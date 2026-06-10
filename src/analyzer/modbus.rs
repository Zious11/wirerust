//! Modbus TCP pure-core parser, function-code classifier, per-flow correlation state,
//! and detection-emission engine (SS-14, CAP-14).
//!
//! This module provides the pure, formally-verified core functions for Modbus TCP
//! analysis per BC-2.14.001 through BC-2.14.022 and VP-022 (Kani).
//!
//! ## Architecture
//! - `parse_mbap_header` — pure parse, no validity gate (BC-2.14.001/002)
//! - `is_valid_modbus_adu` — 3-point validity gate (BC-2.14.003/004)
//! - `classify_fc` — total FC classification over all 256 u8 values (BC-2.14.005–008)
//! - `ModbusFlowState` — full per-flow state (BC-2.14.009–012, STORY-103)
//! - `ModbusAnalyzer` — analyzer-level aggregates and `duplicate_inflight_txn` counter
//! - `ModbusAnalyzer::process_pdu` — detection engine stub (STORY-104, BC-2.14.013–022)
//! - `ModbusAnalyzer::summarize` — six-key summary stub (STORY-104, BC-2.14.021)
//! - `MAX_PENDING_TRANSACTIONS` — hard bound of 256 (BC-2.14.012)
//! - `MAX_FINDINGS` — cap at 10,000 findings (BC-2.14.022)
//! - VP-022 Kani harnesses (sub-properties A, B, C) — gated by `#[cfg(kani)]`

use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

/// Parsed Modbus Application Protocol (MBAP) header.
///
/// All fields decoded big-endian from fixed offsets per Modbus.org spec V1.1b3 §4.2:
/// - `transaction_id` at bytes 0–1
/// - `protocol_id`    at bytes 2–3
/// - `length`         at bytes 4–5  (covers Unit ID + PDU, NOT the 6-byte MBAP prefix)
/// - `unit_id`        at byte 6
/// - `function_code`  at byte 7
///
/// BC-2.14.001 postconditions 2–6.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MbapHeader {
    pub transaction_id: u16,
    pub protocol_id: u16,
    /// Byte count of Unit ID + PDU (NOT including the 6-byte prefix TxnID+ProtoID+Length).
    /// Valid range for Modbus: [2, 254]. Full ADU byte count = 6 + length.
    pub length: u16,
    pub unit_id: u8,
    pub function_code: u8,
}

/// Function-code classification result (BC-2.14.005).
///
/// Variants:
/// - `Read`       — data-read FCs: {0x01,0x02,0x03,0x04,0x07,0x0B,0x0C,0x11,0x14,0x18}
/// - `Write`      — state-changing write FCs: {0x05,0x06,0x0F,0x10,0x15,0x16,0x17}
/// - `Diagnostic` — management/tunneling FCs: {0x08,0x2B}
/// - `Exception`  — any FC with high bit set (fc >= 0x80); biconditional (VP-022 sub-C)
/// - `Unknown`    — all remaining FC values (wildcard — guarantees totality)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FunctionCodeClass {
    Read,
    Write,
    Diagnostic,
    Exception,
    Unknown,
}

/// Hard upper bound on pending-table size per flow (BC-2.14.012).
///
/// When `pending.len() >= MAX_PENDING_TRANSACTIONS`, new request insertions are
/// silently dropped (no panic, no eviction of existing entries).
pub const MAX_PENDING_TRANSACTIONS: usize = 256;

/// Maximum number of findings held by `ModbusAnalyzer` (BC-2.14.022).
///
/// When `all_findings.len() >= MAX_FINDINGS`, all subsequent finding-push sites
/// perform a poison-skip: the finding is discarded and `dropped_findings` incremented.
pub const MAX_FINDINGS: usize = 10_000;

/// Maximum bytes held in a per-flow carry buffer (F-105-001 DoS guard).
///
/// A valid Modbus ADU is at most 6 (MBAP prefix) + 254 (max length field) = 260 bytes.
/// If the carry buffer would grow beyond this cap, the stream is treated as non-Modbus
/// (`is_non_modbus = true`), preventing unbounded memory growth on a malicious
/// never-completing stream.
pub const MAX_ADU_CARRY_BYTES: usize = 260;

// ---------------------------------------------------------------------------
// Detection window constants (STORY-104, BC-2.14.016/017/019)
// ---------------------------------------------------------------------------

/// T0831 coordinated-write window width in seconds (BC-2.14.016).
pub const T0831_WINDOW_SECS: u32 = 5;

/// Burst detector window width in seconds (BC-2.14.017 Invariant 1).
pub const WRITE_BURST_WINDOW_SECS: u32 = 1;

/// Sustained detector minimum window duration in seconds (BC-2.14.017 Invariant 2).
pub const WRITE_SUSTAINED_WINDOW_SECS: u32 = 2;

/// Default burst threshold (write-FCs per 1-second window) (BC-2.14.017).
pub const DEFAULT_WRITE_BURST_THRESHOLD: u32 = 20;

/// Default sustained threshold (avg write-FCs/sec over >=2s) (BC-2.14.017).
pub const DEFAULT_WRITE_SUSTAINED_THRESHOLD: u32 = 10;

/// Exception burst threshold: finding fires when same-code count STRICTLY EXCEEDS
/// this value in the 10-second window (BC-2.14.019). Fires on the 6th exception.
pub const EXCEPTION_RATE_THRESHOLD: u32 = 5;

/// Exception-burst window width in seconds (BC-2.14.019).
pub const EXCEPTION_WINDOW_SECS: u32 = 10;

/// Per-flow Modbus analyzer state — authoritative field list (STORY-103, f2-fix-directives §11.4).
///
/// All window-duration arithmetic MUST use `wrapping_sub` on the u32 timestamps
/// (f2-fix-directives §11.5b) — even though no window timers fire in STORY-103, the
/// fields are initialized here so STORY-104 detection logic can write to them.
///
/// `duplicate_inflight_txn` is intentionally on `ModbusAnalyzer` (NOT here) per
/// BC-2.14.009 invariant 6 — it is an analyzer-level diagnostic counter.
#[derive(Default)]
pub struct ModbusFlowState {
    // --- Transaction correlation (BC-2.14.009–012) ---
    /// Bounded pending-request table: (transaction_id, unit_id) -> (function_code, timestamp).
    /// Hard cap: MAX_PENDING_TRANSACTIONS = 256. Drop-not-evict when full.
    pub pending: HashMap<(u16, u8), (u8, u32)>,

    // --- Per-flow aggregate counters ---
    pub write_count: u64,
    pub exception_count: u64,
    pub pdu_count: u64,
    pub last_ts: u32,

    // --- T0806/T0855 burst window (1-second, configurable burst threshold) ---
    pub window_write_count: u32,
    pub window_start_ts: u32,
    pub window_burst_emitted: bool,

    // --- T0806/T0855 sustained window (>=2-second rolling, configurable sustained threshold) ---
    pub sustained_window_start_ts: u32,
    pub sustained_window_write_count: u32,
    pub sustained_burst_emitted: bool,

    // --- T0831 coordinated-write window (5-second fixed, not CLI-configurable) ---
    pub t0831_window_start_ts: u32,
    pub t0831_window_write_count: u32,
    pub t0831_burst_emitted: bool,

    // --- BC-2.14.019 exception-burst windows (per exception code) ---
    pub exception_window_counts: HashMap<u8, u32>,
    pub exception_window_start_ts: HashMap<u8, u32>,
    pub exception_burst_emitted: HashMap<u8, bool>,

    // --- Desync safety (Decision 6) ---
    /// When `true`, this flow carries non-Modbus binary data (Protocol ID != 0x0000).
    /// All subsequent `on_data` calls bail immediately without parsing.
    pub is_non_modbus: bool,

    // --- TCP reassembly carry buffer (F-105-001 partial-ADU buffering fix) ---
    /// Bytes left over from a prior `on_data` call that form an incomplete ADU.
    ///
    /// The reassembler delivers each contiguous TCP segment as a separate `on_data`
    /// call; a Modbus ADU (6-byte MBAP prefix + variable PDU) can span two segments.
    /// On every call we prepend `carry` to the incoming data before the walk loop.
    /// When the walk loop cannot find a full ADU in `remaining`, the tail is stashed
    /// here for the next call.
    ///
    /// Bounded by `MAX_ADU_CARRY_BYTES`: if the carry buffer would exceed this cap
    /// (a malformed or adversarially crafted stream whose MBAP `length` field promises
    /// more than one Modbus ADU maximum), we set `is_non_modbus = true` and bail,
    /// preventing unbounded memory growth (DoS guard).
    pub carry: Vec<u8>,
}

impl ModbusFlowState {
    /// Insert a request ADU into the pending table (BC-2.14.009).
    ///
    /// Precondition: caller has verified `classify_fc(fc) != Exception`.
    /// Returns `Some(old_value)` if an existing entry was overwritten (key reuse).
    ///
    /// Enforcement:
    /// - If the key already exists in pending: overwrite and return `Some(old_value)`.
    ///   (Caller increments `duplicate_inflight_txn` when `Some(_)` is returned.)
    /// - If the key is NEW and `pending.len() >= MAX_PENDING_TRANSACTIONS`: silently drop,
    ///   return `None` (BC-2.14.012: drop-not-evict; no panic).
    /// - Otherwise: insert and return `None`.
    pub fn insert_request(
        &mut self,
        txn_id: u16,
        unit_id: u8,
        fc: u8,
        ts: u32,
    ) -> Option<(u8, u32)> {
        let key = (txn_id, unit_id);
        if self.pending.contains_key(&key) {
            // Key already exists — overwrite path; return old value so caller can
            // increment `duplicate_inflight_txn` (BC-2.14.009 invariant 6).
            self.pending.insert(key, (fc, ts))
        } else if self.pending.len() >= MAX_PENDING_TRANSACTIONS {
            // Table at cap — silently drop; do not evict; no panic (BC-2.14.012).
            None
        } else {
            self.pending.insert(key, (fc, ts))
        }
    }

    /// Match a normal (non-exception) response against the pending table (BC-2.14.010).
    ///
    /// Looks up `(txn_id, unit_id)` in pending:
    /// - If found: removes the entry and returns `Some((stored_fc, stored_ts))`.
    ///   The pair is considered closed regardless of whether the response FC echoes the
    ///   request FC (BC-2.14.010 Case B: FC mismatch still closes the pair).
    /// - If not found (orphan): returns `None`, state unchanged.
    pub fn match_response(&mut self, txn_id: u16, unit_id: u8, _fc: u8) -> Option<(u8, u32)> {
        // BC-2.14.010: remove and return on any key match (pair is closed regardless of
        // FC echo match or mismatch — see postcondition 3 "closed regardless of anomaly").
        self.pending.remove(&(txn_id, unit_id))
    }

    /// Attribute an exception response to the original request FC (BC-2.14.011).
    ///
    /// `exception_fc` is the raw exception FC byte (>= 0x80).
    /// Derives `original_fc = exception_fc & 0x7F`.
    ///
    /// Strict FC consistency gate (anti-spoof invariant):
    /// - If found AND `original_fc == stored_fc`: removes entry, returns `Some(original_fc)`.
    /// - If found AND `original_fc != stored_fc` (FC mismatch / spoof guard): does NOT
    ///   remove entry, returns `None` (BC-2.14.011 EC-010).
    /// - If not found (orphan exception): returns `None`.
    ///
    /// Caller MUST increment `exception_count` regardless of return value (BC-2.14.011 post.6).
    pub fn attribute_exception(
        &mut self,
        txn_id: u16,
        unit_id: u8,
        exception_fc: u8,
    ) -> Option<u8> {
        let original_fc = exception_fc & 0x7F;
        let key = (txn_id, unit_id);
        match self.pending.get(&key) {
            Some(&(stored_fc, _)) if stored_fc == original_fc => {
                // FC consistency gate passes — close the pair.
                self.pending.remove(&key);
                Some(original_fc)
            }
            Some(_) => {
                // FC mismatch (spoof guard): do NOT remove the pending entry.
                None
            }
            None => {
                // Orphan exception: no matching pending entry.
                None
            }
        }
    }
}

/// Analyzer-level aggregates for Modbus TCP (STORY-103/104/105, f2-fix-directives §11.3).
///
/// STORY-105: `flows` field added — per-flow `ModbusFlowState` keyed by `FlowKey`.
/// The dispatcher routes all port-502 TCP data here via `StreamHandler::on_data`.
///
/// `duplicate_inflight_txn` is an INTERNAL diagnostic counter (BC-2.14.009 invariant 6).
/// It is NOT surfaced in `summarize()` (BC-2.14.021's six-key contract is unchanged).
pub struct ModbusAnalyzer {
    /// --modbus-write-burst-threshold (default 20): max write-FCs in any 1-second window.
    pub write_burst_threshold: u32,
    /// --modbus-write-sustained-threshold (default 10): max avg write-FCs/sec over >=2s window.
    pub write_sustained_threshold: u32,
    /// Counts how many pending-table entries were overwritten by a duplicate (txn_id, unit_id)
    /// before the original response arrived. INTERNAL — not in summarize().
    pub duplicate_inflight_txn: u64,
    /// Total PDU count across all flows (valid ADUs past the 3-point gate).
    pub total_pdu_count: u64,
    /// Total write-class FC PDUs across all flows (BC-2.14.021).
    pub total_write_count: u64,
    /// Total exception-response PDUs across all flows (BC-2.14.021).
    pub total_exception_count: u64,
    /// Total ADUs that failed the 3-point validity gate (BC-2.14.021).
    pub parse_errors: u64,
    /// Per-FC occurrence counts across all flows.
    pub fn_code_counts: HashMap<u8, u64>,
    /// Accumulated findings — capped at MAX_FINDINGS (BC-2.14.022).
    pub all_findings: Vec<Finding>,
    /// Count of findings silently dropped after MAX_FINDINGS cap was reached (BC-2.14.022).
    pub dropped_findings: u64,
    /// Monotonic counter: incremented once per flow on first PDU insertion (BC-2.14.021).
    /// NOT derived from a flow map length (flows removed on close would give wrong count).
    pub total_flows_analyzed: u64,
    /// Per-flow reassembly state (STORY-105). Keyed by `FlowKey`.
    /// Flows are inserted on first `on_data` and removed on `on_flow_close`.
    flows: HashMap<FlowKey, ModbusFlowState>,
}

impl ModbusAnalyzer {
    /// Construct a new `ModbusAnalyzer` with the given dual-window thresholds.
    pub fn new(write_burst_threshold: u32, write_sustained_threshold: u32) -> Self {
        Self {
            write_burst_threshold,
            write_sustained_threshold,
            duplicate_inflight_txn: 0,
            total_pdu_count: 0,
            total_write_count: 0,
            total_exception_count: 0,
            parse_errors: 0,
            fn_code_counts: HashMap::new(),
            all_findings: Vec::new(),
            dropped_findings: 0,
            total_flows_analyzed: 0,
            flows: HashMap::new(),
        }
    }

    /// Detection engine: process one Modbus ADU (STORY-104, BC-2.14.013–022).
    ///
    /// Takes ownership of a mutable borrow of the per-flow `ModbusFlowState` and the
    /// raw ADU bytes (already validated by the caller via `is_valid_modbus_adu`).
    ///
    /// Responsibilities:
    /// - Insert request into `flow.pending` (BC-2.14.009); match/attribute responses
    ///   (BC-2.14.010/011); increment `duplicate_inflight_txn` on key overwrite.
    /// - Update aggregate counters: `total_pdu_count`, `total_write_count`, etc.
    /// - Update `fn_code_counts`.
    /// - Run all seven detectors (BC-2.14.013–020): write-class / T0831 / burst /
    ///   sustained / diagnostics / exception-burst / recon / unknown.
    /// - Guard every finding push with `MAX_FINDINGS` poison-skip (BC-2.14.022).
    /// - Return a `Vec<Finding>` containing all findings emitted for this PDU.
    ///
    /// The `timestamp` argument is the pcap-relative capture timestamp (u32 microseconds).
    /// All window elapsed computations use `now_ts.wrapping_sub(window_start_ts)` per
    /// f2-fix-directives §11.5b.
    #[allow(clippy::too_many_arguments)] // 8 params: interface dictated by STORY-105 wiring (FlowKey, flow state, direction, header, fc, raw data, timestamp)
    pub fn process_pdu(
        &mut self,
        flow_key: &FlowKey,
        flow: &mut ModbusFlowState,
        direction: Direction,
        header: &MbapHeader,
        fc: u8,
        data: &[u8],
        timestamp: u32,
    ) -> Vec<Finding> {
        use chrono::DateTime;

        // Convert pcap-relative microsecond u32 to a DateTime<Utc> for Finding.timestamp.
        // Per BC-2.09.007 post.1: DateTime::from_timestamp(ts_sec, ts_usec * 1_000)
        // where ts_sec = timestamp / 1_000_000, ts_usec = timestamp % 1_000_000.
        let ts_sec = (timestamp / 1_000_000) as i64;
        let ts_nsec = (timestamp % 1_000_000) * 1_000;
        let finding_ts = DateTime::from_timestamp(ts_sec, ts_nsec);

        // Resolve source IPs from FlowKey using Modbus port-502 convention
        // (BC-2.14.013 post.1, BC-2.14.017 post.1, BC-2.14.019 post.A/B, BC-2.14.020).
        // The Modbus server always listens on port 502; the client is the other endpoint.
        // FlowKey stores (lower_ip:lower_port, upper_ip:upper_port) canonicalized by
        // (ip, port) tuple order — we check which port is 502 to identify server vs client.
        let client_ip = if flow_key.lower_port() == 502 {
            flow_key.upper_ip()
        } else {
            flow_key.lower_ip()
        };
        let server_ip = if flow_key.lower_port() == 502 {
            flow_key.lower_ip()
        } else {
            flow_key.upper_ip()
        };

        // Helper: push a finding into self.all_findings with MAX_FINDINGS poison-skip.
        // Returns the finding unchanged so it can also be appended to the local return vec.
        // We accumulate into a local vec first, then push in one pass below.

        let mut local_findings: Vec<Finding> = Vec::new();

        // --- total_flows_analyzed: increment once per flow on first PDU (BC-2.14.021 post.3) ---
        if flow.pdu_count == 0 {
            self.total_flows_analyzed += 1;
        }

        // --- Per-flow PDU counter + last timestamp (always, regardless of direction) ---
        flow.pdu_count += 1;
        flow.last_ts = timestamp;

        // --- Analyzer-level PDU counter + FC distribution (always, regardless of direction) ---
        self.total_pdu_count += 1;
        *self.fn_code_counts.entry(fc).or_insert(0) += 1;

        // --- Classify FC ---
        let fc_class = classify_fc(fc);

        // --- Recon detection for FC=0x11 and FC=0x2B/0x0E: direction-independent (BC-2.14.020 EC-010) ---
        // These emit regardless of direction; source_ip uses direction to select client vs server.
        match fc {
            0x11 => {
                // FC=0x11 (Report Server ID) → T0888 recon (BC-2.14.020 post. recon path).
                // Fires for both ClientToServer and ServerToClient (EC-010).
                let src_ip = match direction {
                    Direction::ClientToServer => client_ip,
                    Direction::ServerToClient => server_ip,
                };
                local_findings.push(Finding {
                    category: crate::findings::ThreatCategory::Anomaly,
                    verdict: crate::findings::Verdict::Inconclusive,
                    confidence: crate::findings::Confidence::Medium,
                    summary: format!(
                        "Modbus recon: Report Server ID (FC 0x11) from unit {}",
                        header.unit_id
                    ),
                    evidence: vec![format!(
                        "FC=0x11 TxnID={:#06X} UnitID={}",
                        header.transaction_id, header.unit_id
                    )],
                    mitre_techniques: vec!["T0888".to_string()],
                    source_ip: Some(src_ip),
                    timestamp: finding_ts,
                    direction: Some(direction),
                });
            }
            0x2B => {
                // FC=0x2B MEI: check MEI type byte (data[8]).
                // MEI type 0x0E = Read Device Identification → T0888 (BC-2.14.020 EC-010).
                // MEI type != 0x0E → no T0888 (BC-2.14.020 EC-005).
                // Fires for both directions per EC-010.
                if data.len() >= 9 {
                    let mei_type = data[8];
                    if mei_type == 0x0E {
                        let src_ip = match direction {
                            Direction::ClientToServer => client_ip,
                            Direction::ServerToClient => server_ip,
                        };
                        local_findings.push(Finding {
                            category: crate::findings::ThreatCategory::Anomaly,
                            verdict: crate::findings::Verdict::Inconclusive,
                            confidence: crate::findings::Confidence::Medium,
                            summary: format!(
                                "Modbus recon: Read Device Identification (MEI 0x2B/0x0E) on unit {}",
                                header.unit_id
                            ),
                            evidence: vec![format!(
                                "FC=0x2B MEI type=0x0E TxnID={:#06X} UnitID={}",
                                header.transaction_id, header.unit_id
                            )],
                            mitre_techniques: vec!["T0888".to_string()],
                            source_ip: Some(src_ip),
                            timestamp: finding_ts,
                            direction: Some(direction),
                        });
                    }
                    // MEI type != 0x0E: no T0888.
                }
            }
            // Unknown FC detection: direction-independent (BC-2.14.020 EC-010).
            fc_byte if fc_class == FunctionCodeClass::Unknown => {
                let src_ip = match direction {
                    Direction::ClientToServer => client_ip,
                    Direction::ServerToClient => server_ip,
                };
                local_findings.push(Finding {
                    category: crate::findings::ThreatCategory::Anomaly,
                    verdict: crate::findings::Verdict::Inconclusive,
                    confidence: crate::findings::Confidence::Low,
                    summary: format!(
                        "Modbus unknown function code: 0x{fc_byte:02X} on unit {}",
                        header.unit_id
                    ),
                    evidence: vec![format!(
                        "FC=0x{fc_byte:02X} TxnID={:#06X} UnitID={}",
                        header.transaction_id, header.unit_id
                    )],
                    mitre_techniques: vec![],
                    source_ip: Some(src_ip),
                    timestamp: finding_ts,
                    direction: Some(direction),
                });
            }
            _ => {}
        }

        // --- Branch on direction ---
        match direction {
            Direction::ClientToServer => {
                // REQUEST path
                // --- Insert into pending table (BC-2.14.009) ---
                if fc_class != FunctionCodeClass::Exception {
                    let overwrite =
                        flow.insert_request(header.transaction_id, header.unit_id, fc, timestamp);
                    if overwrite.is_some() {
                        self.duplicate_inflight_txn += 1;
                    }
                }

                match fc_class {
                    FunctionCodeClass::Write => {
                        // -------------------------------------------------------
                        // Write-class detection (BC-2.14.013–016)
                        // -------------------------------------------------------
                        self.total_write_count += 1;
                        flow.write_count += 1; // per-flow counter (BC-2.14.013 post.2)

                        // Determine tag subset per ORCHESTRATOR RULING BC-DISCREPANCY-001:
                        // Register-write set {0x06,0x10,0x16,0x17} → T0836.
                        // Coil-write set     {0x05,0x0F}           → T0835.
                        // All other write FCs (0x15)               → T0855 only.
                        let is_register_write = matches!(fc, 0x06 | 0x10 | 0x16 | 0x17);
                        let is_coil_write = matches!(fc, 0x05 | 0x0F);

                        // -------------------------------------------------------
                        // T0831 inline co-tag logic (BC-2.14.016)
                        // Must run BEFORE building mitre_techniques vec so T0831
                        // can be appended if the condition fires.
                        // Window update FIRST, emission check SECOND (BC-2.14.016 inv2).
                        // T0831 window applies only to register-write FCs {0x06,0x10,0x16,0x17}.
                        // -------------------------------------------------------
                        let mut emit_t0831 = false;
                        if is_register_write {
                            // Check if window has expired (wrapping_sub).
                            let t0831_elapsed = timestamp.wrapping_sub(flow.t0831_window_start_ts);
                            if t0831_elapsed > T0831_WINDOW_SECS * 1_000_000 {
                                // Window expired → reset.
                                flow.t0831_window_start_ts = timestamp;
                                flow.t0831_window_write_count = 1;
                                flow.t0831_burst_emitted = false;
                            } else {
                                // Still within window → increment count FIRST (update-before-check).
                                flow.t0831_window_write_count += 1;
                                // Now check emission: count >= 2 and not yet emitted.
                                if flow.t0831_window_write_count >= 2 && !flow.t0831_burst_emitted {
                                    emit_t0831 = true;
                                    flow.t0831_burst_emitted = true;
                                }
                            }
                        }

                        // Build the canonical mitre_techniques vec.
                        // Canonical order (ADR-006 §13.7 sub-decision 3):
                        //   T0806 > T0855 > T0836 > T0835 > T0831 > T0814 > T0888
                        // For per-PDU write findings: T0855 always first (no T0806 here);
                        // then T0836 or T0835 based on subset; then T0831 if co-tagged.
                        let mut mitre: Vec<String> = Vec::with_capacity(3);
                        mitre.push("T0855".to_string());
                        if is_register_write {
                            mitre.push("T0836".to_string());
                        } else if is_coil_write {
                            mitre.push("T0835".to_string());
                        }
                        if emit_t0831 {
                            mitre.push("T0831".to_string());
                        }

                        // Emit ONE per-PDU write finding (BC-2.14.013 invariant 1).
                        local_findings.push(Finding {
                            category: crate::findings::ThreatCategory::Execution,
                            verdict: crate::findings::Verdict::Likely,
                            confidence: crate::findings::Confidence::Medium,
                            summary: format!(
                                "Modbus write command observed: FC 0x{fc:02X} from unit {}",
                                header.unit_id
                            ),
                            evidence: vec![format!(
                                "FC=0x{fc:02X} TxnID={:#06X} UnitID={} ADU bytes 0..{}",
                                header.transaction_id,
                                header.unit_id,
                                data.len()
                            )],
                            mitre_techniques: mitre,
                            source_ip: Some(client_ip),
                            timestamp: finding_ts,
                            direction: Some(direction),
                        });

                        // -------------------------------------------------------
                        // Burst detector: 1-second window (BC-2.14.017 Invariant 1)
                        // Update window FIRST, then check threshold (wrapping_sub).
                        // -------------------------------------------------------
                        {
                            let burst_elapsed = timestamp.wrapping_sub(flow.window_start_ts);
                            if burst_elapsed > WRITE_BURST_WINDOW_SECS * 1_000_000 {
                                // Window expired → slide forward.
                                flow.window_start_ts = timestamp;
                                flow.window_write_count = 1;
                                flow.window_burst_emitted = false;
                            } else {
                                flow.window_write_count += 1;
                                if flow.window_write_count > self.write_burst_threshold
                                    && !flow.window_burst_emitted
                                {
                                    flow.window_burst_emitted = true;
                                    let elapsed_ms = burst_elapsed / 1_000;
                                    // Burst finding: SEPARATE from per-PDU finding (BC-2.14.013 inv5).
                                    local_findings.push(Finding {
                                        category: crate::findings::ThreatCategory::Execution,
                                        verdict: crate::findings::Verdict::Likely,
                                        confidence: crate::findings::Confidence::High,
                                        summary: format!(
                                            "Modbus write burst: {} writes in {}ms window (unit {}, threshold {}/s)",
                                            flow.window_write_count,
                                            elapsed_ms,
                                            header.unit_id,
                                            self.write_burst_threshold
                                        ),
                                        evidence: vec![format!(
                                            "Burst threshold exceeded: {} write FCs in 1s window; \
                                             window_write_count={} window_start_ts={} threshold={} \
                                             FC=0x{:02X} UnitID={}",
                                            flow.window_write_count,
                                            flow.window_write_count,
                                            flow.window_start_ts,
                                            self.write_burst_threshold,
                                            fc,
                                            header.unit_id
                                        )],
                                        // Canonical order: T0806 first, then T0855.
                                        mitre_techniques: vec![
                                            "T0806".to_string(),
                                            "T0855".to_string(),
                                        ],
                                        source_ip: Some(client_ip),
                                        timestamp: finding_ts,
                                        direction: Some(direction),
                                    });
                                }
                            }
                        }

                        // -------------------------------------------------------
                        // Sustained detector: >=2-second window (BC-2.14.017 Invariant 2)
                        // Truncation-free cross-multiplication (f2-fix-directives §11.5a).
                        // -------------------------------------------------------
                        {
                            if flow.sustained_window_write_count == 0 {
                                // Uninitialized — start window.
                                flow.sustained_window_start_ts = timestamp;
                                flow.sustained_window_write_count = 1;
                                flow.sustained_burst_emitted = false;
                            } else {
                                // Accumulate first (update-before-check).
                                flow.sustained_window_write_count += 1;
                                let elapsed_us =
                                    timestamp.wrapping_sub(flow.sustained_window_start_ts);

                                // Check trigger: elapsed >= 2s AND rate exceeded AND not emitted.
                                // Truncation-free: (count*1_000_000) > (threshold*elapsed_us)
                                // (f2-fix-directives §11.5a)
                                if elapsed_us >= WRITE_SUSTAINED_WINDOW_SECS * 1_000_000
                                    && !flow.sustained_burst_emitted
                                    && (flow.sustained_window_write_count as u64) * 1_000_000
                                        > (self.write_sustained_threshold as u64)
                                            * (elapsed_us as u64)
                                {
                                    flow.sustained_burst_emitted = true;
                                    let elapsed_secs_f = (elapsed_us as f64) / 1_000_000.0;
                                    local_findings.push(Finding {
                                        category: crate::findings::ThreatCategory::Execution,
                                        verdict: crate::findings::Verdict::Likely,
                                        confidence: crate::findings::Confidence::High,
                                        summary: format!(
                                            "Modbus write burst: {} writes over {:.0}s window (unit {}, >{}/s avg)",
                                            flow.sustained_window_write_count,
                                            elapsed_secs_f,
                                            header.unit_id,
                                            self.write_sustained_threshold
                                        ),
                                        evidence: vec![format!(
                                            "Sustained write rate exceeded: {} writes over {:.1} seconds \
                                             (>{}/s average); sustained_window_start_ts={} \
                                             FC=0x{:02X} UnitID={}",
                                            flow.sustained_window_write_count,
                                            elapsed_secs_f,
                                            self.write_sustained_threshold,
                                            flow.sustained_window_start_ts,
                                            fc,
                                            header.unit_id
                                        )],
                                        // Canonical order: T0806 first, then T0855.
                                        mitre_techniques: vec![
                                            "T0806".to_string(),
                                            "T0855".to_string(),
                                        ],
                                        source_ip: Some(client_ip),
                                        timestamp: finding_ts,
                                        direction: Some(direction),
                                    });
                                }

                                // Window slide: reset when elapsed >= WRITE_SUSTAINED_WINDOW_SECS.
                                // (f2-fix-directives §11.5 step 5: ALWAYS reset regardless of
                                // whether a finding fired, to prevent unbounded accumulation.)
                                if elapsed_us >= WRITE_SUSTAINED_WINDOW_SECS * 1_000_000 {
                                    flow.sustained_window_start_ts = timestamp;
                                    flow.sustained_window_write_count = 1;
                                    flow.sustained_burst_emitted = false;
                                }
                            }
                        }
                    }

                    FunctionCodeClass::Diagnostic => {
                        // -------------------------------------------------------
                        // Diagnostics detection (BC-2.14.018 + BC-2.14.019 Path B)
                        // FC 0x08: check sub-function bytes in PDU.
                        //   sub-func 0x0001/0x0004 → T0814 (BC-2.14.018)
                        //   sub-func 0x000A → anti-forensic Anomaly (BC-2.14.019 Path B)
                        // FC 0x2B: handled in the direction-independent recon block above.
                        // -------------------------------------------------------
                        if fc == 0x08 {
                            // PDU layout: MBAP(7 bytes) + FC(1 byte) + sub_func(2 bytes) + data...
                            // Full ADU: data[0..6]=MBAP prefix, data[7]=FC, data[8..9]=sub-func.
                            if data.len() >= 10 {
                                let sub_func = u16::from_be_bytes([data[8], data[9]]);
                                if matches!(sub_func, 0x0001 | 0x0004) {
                                    // BC-2.14.018: DoS sub-functions → T0814 Denial of Service.
                                    local_findings.push(Finding {
                                        category: crate::findings::ThreatCategory::Anomaly,
                                        verdict: crate::findings::Verdict::Likely,
                                        confidence: crate::findings::Confidence::High,
                                        summary: format!(
                                            "Modbus Diagnostics DoS sub-function 0x{sub_func:04X} detected"
                                        ),
                                        evidence: vec![format!(
                                            "FC=0x08 sub-func=0x{sub_func:04X} unit_id=0x{:02X}",
                                            header.unit_id
                                        )],
                                        mitre_techniques: vec!["T0814".to_string()],
                                        source_ip: Some(client_ip),
                                        timestamp: finding_ts,
                                        direction: Some(direction),
                                    });
                                } else if sub_func == 0x000A {
                                    // BC-2.14.019 Path B: Clear Counters → anti-forensic Anomaly.
                                    local_findings.push(Finding {
                                        category: crate::findings::ThreatCategory::Anomaly,
                                        verdict: crate::findings::Verdict::Inconclusive,
                                        confidence: crate::findings::Confidence::Medium,
                                        summary: format!(
                                            "Modbus anti-forensic: Clear Counters (0x08/0x000A) sent to unit {}",
                                            header.unit_id
                                        ),
                                        evidence: vec![format!(
                                            "FC=0x08 SubFunc=0x000A TxnID={:#06X} UnitID={}",
                                            header.transaction_id, header.unit_id
                                        )],
                                        mitre_techniques: vec![],
                                        source_ip: Some(client_ip),
                                        timestamp: finding_ts,
                                        direction: Some(direction),
                                    });
                                }
                                // Other sub-functions: no T0814 (BC-2.14.018 EC-006).
                            }
                        }
                        // FC=0x2B handled in direction-independent recon block above.
                    }

                    FunctionCodeClass::Read => {
                        // Recon detection for FC=0x11 is handled in the direction-independent block
                        // above; no additional action for other Read-class FCs.
                        // FC=0x07 and all other read FCs: no finding.
                    }

                    FunctionCodeClass::Unknown => {
                        // Unknown FC handled in the direction-independent block above.
                    }

                    FunctionCodeClass::Exception => {
                        // Should not occur in ClientToServer direction (exceptions are
                        // server responses). No-op in request path.
                    }
                }
            }

            Direction::ServerToClient => {
                // RESPONSE path
                if fc_class == FunctionCodeClass::Exception {
                    // -------------------------------------------------------
                    // Exception response (BC-2.14.011 + BC-2.14.019)
                    // -------------------------------------------------------
                    self.total_exception_count += 1;
                    flow.exception_count += 1; // per-flow counter (BC-2.14.019 inv4)

                    // Attribute exception to pending request (BC-2.14.011).
                    flow.attribute_exception(header.transaction_id, header.unit_id, fc);

                    // Exception code byte is at data[8] (after 8-byte MBAP+FC prefix).
                    let exc_code = if data.len() >= 9 { data[8] } else { 0xFF };

                    // Per-exception-code burst window (BC-2.14.019).
                    // wrapping_sub for all elapsed computations (f2-fix-directives §11.5b).
                    // CRITICAL: use unwrap_or(timestamp) so new codes get 0 elapsed on first
                    // occurrence, NOT an anchor — the anchor must be inserted in the else branch.
                    let exc_elapsed = timestamp.wrapping_sub(
                        *flow
                            .exception_window_start_ts
                            .get(&exc_code)
                            .unwrap_or(&timestamp),
                    );

                    if exc_elapsed > EXCEPTION_WINDOW_SECS * 1_000_000 {
                        // Window expired → reset (also handles first-time with exc_elapsed=0
                        // via the else branch below for new codes).
                        flow.exception_window_counts.insert(exc_code, 1);
                        flow.exception_window_start_ts.insert(exc_code, timestamp);
                        flow.exception_burst_emitted.insert(exc_code, false);
                    } else {
                        // Accumulate count. For a NEW exception code, or_insert(0) → count = 1.
                        // ALSO seed the window-start timestamp for new codes so subsequent
                        // exceptions measure real elapsed time (BC-2.14.019 EC-005 fix).
                        // Seed window-start timestamp for new exception codes so subsequent
                        // exceptions measure real elapsed time (BC-2.14.019 EC-005 anchor fix).
                        flow.exception_window_start_ts
                            .entry(exc_code)
                            .or_insert(timestamp);
                        let count = flow.exception_window_counts.entry(exc_code).or_insert(0);
                        *count += 1;
                        let cur_count = *count;
                        let emitted = *flow
                            .exception_burst_emitted
                            .get(&exc_code)
                            .unwrap_or(&false);

                        if cur_count > EXCEPTION_RATE_THRESHOLD && !emitted {
                            flow.exception_burst_emitted.insert(exc_code, true);
                            let orig_fc = fc & 0x7F;
                            let summary = match exc_code {
                                0x01 => format!(
                                    "Modbus recon: {} Illegal Function exceptions in window (unit {}) — possible FC scanning",
                                    cur_count, header.unit_id
                                ),
                                0x02 => format!(
                                    "Modbus recon: {} Illegal Data Address exceptions in window (unit {}) — possible register map enumeration",
                                    cur_count, header.unit_id
                                ),
                                _ => format!(
                                    "Modbus exception anomaly: {} exceptions code 0x{exc_code:02X} in window (unit {})",
                                    cur_count, header.unit_id
                                ),
                            };
                            local_findings.push(Finding {
                                category: crate::findings::ThreatCategory::Anomaly,
                                verdict: crate::findings::Verdict::Inconclusive,
                                confidence: crate::findings::Confidence::Medium,
                                summary,
                                evidence: vec![format!(
                                    "exception_fc=0x{fc:02X} exception_code=0x{exc_code:02X} \
                                     window_count={cur_count} original_fc=0x{orig_fc:02X}"
                                )],
                                mitre_techniques: vec![],
                                source_ip: Some(server_ip),
                                timestamp: finding_ts,
                                direction: Some(direction),
                            });
                        }
                    }
                } else {
                    // Normal response: match against pending table.
                    flow.match_response(header.transaction_id, header.unit_id, fc);
                }
            }
        }

        // -------------------------------------------------------
        // MAX_FINDINGS poison-skip (BC-2.14.022)
        // Push all local_findings into self.all_findings with cap guard.
        // -------------------------------------------------------
        for f in &local_findings {
            if self.all_findings.len() >= MAX_FINDINGS {
                self.dropped_findings += 1;
            } else {
                self.all_findings.push(f.clone());
            }
        }

        local_findings
    }

    /// Produce the six-key `AnalysisSummary` (STORY-104, BC-2.14.021).
    ///
    /// Keys (authoritative set — exactly six):
    ///   `pdu_count`, `write_count`, `exception_count`, `parse_errors`,
    ///   `function_code_distribution`, `dropped_findings`.
    ///
    /// `function_code_distribution` is a JSON object with keys formatted as
    /// "0x{:02X}" (uppercase hex, zero-padded, "0x" prefix) per BC-2.14.021 invariant 3.
    /// Zero-count FC entries are suppressed (BC-2.14.021 post.2 invariant 2).
    pub fn summarize(&self) -> AnalysisSummary {
        use std::collections::BTreeMap;

        let mut detail: BTreeMap<String, serde_json::Value> = BTreeMap::new();

        detail.insert(
            "pdu_count".to_string(),
            serde_json::json!(self.total_pdu_count),
        );
        detail.insert(
            "write_count".to_string(),
            serde_json::json!(self.total_write_count),
        );
        detail.insert(
            "exception_count".to_string(),
            serde_json::json!(self.total_exception_count),
        );
        detail.insert(
            "parse_errors".to_string(),
            serde_json::json!(self.parse_errors),
        );
        detail.insert(
            "dropped_findings".to_string(),
            serde_json::json!(self.dropped_findings),
        );

        // function_code_distribution: "0x{FC:02X}" → count (zero-count suppressed).
        let mut dist: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
        for (&fc, &count) in &self.fn_code_counts {
            if count > 0 {
                dist.insert(format!("0x{fc:02X}"), serde_json::json!(count));
            }
        }
        detail.insert(
            "function_code_distribution".to_string(),
            serde_json::Value::Object(dist),
        );

        AnalysisSummary {
            // BC-2.14.021 post.3: lowercase "modbus" matches "http" and "tls" analyzer convention.
            analyzer_name: "modbus".to_string(),
            packets_analyzed: self.total_pdu_count,
            detail,
        }
    }
}

// ---------------------------------------------------------------------------
// StreamHandler + StreamAnalyzer impls (STORY-105, BC-2.14.025)
// ---------------------------------------------------------------------------

impl StreamHandler for ModbusAnalyzer {
    /// Receive a reassembled TCP data chunk for a port-502 flow.
    ///
    /// Parses MBAP headers out of `data`, applying the 3-point validity gate
    /// per BC-2.14.003/004 (`is_valid_modbus_adu`). For each valid ADU, calls
    /// `process_pdu`. Multiple ADUs per call are handled by advancing through
    /// the buffer in a loop.
    ///
    /// ### Partial-ADU carry buffer (F-105-001)
    ///
    /// The reassembler delivers each contiguous TCP segment as a SEPARATE `on_data`
    /// call. A Modbus ADU (6-byte MBAP prefix + PDU, up to 260 bytes total) can span
    /// two TCP segments. Without a carry buffer this caused:
    ///   - The first segment to be processed as a TRUNCATED ADU
    ///   - The second segment's bytes to be misparsed as a fresh ADU
    ///   - Silent corruption / parse_errors / premature `is_non_modbus` disable
    ///
    /// The fix: prepend `flow.carry` to `data` before the walk loop. When the walk
    /// loop encounters incomplete data (< 8 bytes for MBAP header, or < `adu_len` bytes
    /// for a full ADU), stash the remainder into `flow.carry` and break. On the next
    /// call the carry is prepended again, completing the ADU.
    ///
    /// DoS guard: if `flow.carry` would grow beyond `MAX_ADU_CARRY_BYTES` (260 bytes,
    /// one maximum Modbus ADU), the stream is marked `is_non_modbus` to prevent
    /// unbounded memory growth on a malicious never-completing stream.
    ///
    /// ### Borrow-checker note
    /// `process_pdu` takes `&mut self` AND `&mut ModbusFlowState`. To satisfy the
    /// borrow checker, the flow state is removed from `self.flows`, mutated via
    /// `process_pdu`, then re-inserted. This is safe because `process_pdu` never
    /// touches `self.flows`.
    ///
    /// No-panic guarantee: all attacker-controlled byte lengths are guarded by
    /// `parse_mbap_header` (returns `None` on short data) and the ADU-length
    /// bounds check before slicing. No `unwrap()` on attacker bytes.
    fn on_data(
        &mut self,
        flow_key: &FlowKey,
        direction: Direction,
        data: &[u8],
        _offset: u64,
        timestamp: u32,
    ) {
        // Retrieve or create per-flow state.
        // We need to take the flow out of self.flows to call process_pdu
        // (which takes &mut self + &mut flow_state) without violating the borrow rules.
        let mut flow = self.flows.remove(flow_key).unwrap_or_default();

        // Desync bail: if a previous packet on this flow had protocol_id != 0,
        // skip all further processing (BC-2.14.003 / Decision 6 desync policy).
        if flow.is_non_modbus {
            self.flows.insert(flow_key.clone(), flow);
            return;
        }

        // F-105-001: Prepend carry buffer to incoming data so partial ADUs that
        // spanned two TCP segments are completed before the walk loop runs.
        // We build a combined buffer only when carry is non-empty to avoid an
        // allocation on the common (carry-empty) fast path.
        let combined: Vec<u8>;
        let buf: &[u8] = if flow.carry.is_empty() {
            data
        } else {
            combined = flow
                .carry
                .iter()
                .copied()
                .chain(data.iter().copied())
                .collect();
            &combined
        };
        // Clear the carry — it is now folded into `buf`. Any unconsumed tail will
        // be re-stashed at the end of the loop.
        flow.carry.clear();

        // Walk the buffer: parse and dispatch each ADU in the chunk.
        // Modbus TCP ADU layout: 6-byte MBAP prefix + PDU (length-1 bytes of FC+data).
        // Full ADU byte count = 6 (MBAP prefix without FC) + header.length (includes UnitID).
        // ADU total size = 6 + header.length; minimum valid = 6 + 2 = 8 bytes.
        let mut pos = 0usize;
        while pos < buf.len() {
            let remaining = &buf[pos..];

            // Parse the MBAP header (needs at least 8 bytes: 6 prefix + 1 UnitID + 1 FC).
            // If fewer than 8 bytes remain, the ADU is incomplete — stash the tail into
            // carry and break. The next on_data call will complete it.
            let header = match parse_mbap_header(remaining) {
                Some(h) => h,
                None => {
                    // F-105-001: partial MBAP header — carry the tail forward.
                    // DoS guard: if the tail exceeds the cap, treat as non-Modbus.
                    if remaining.len() > MAX_ADU_CARRY_BYTES {
                        flow.is_non_modbus = true;
                        self.parse_errors += 1;
                    } else {
                        flow.carry.extend_from_slice(remaining);
                    }
                    break;
                }
            };

            // 3-point validity gate (BC-2.14.003/004).
            if !is_valid_modbus_adu(&header) {
                // Invalid protocol_id or out-of-range length.
                // Per the desync policy (BC-2.14.003): if protocol_id != 0, mark the flow
                // as non-Modbus so future chunks bail immediately.
                if header.protocol_id != 0x0000 {
                    flow.is_non_modbus = true;
                }
                self.parse_errors += 1;
                break; // Cannot safely advance: length field is invalid.
            }

            // Compute full ADU byte count: 6-byte MBAP prefix + header.length bytes.
            // header.length covers UnitID (1 byte) + PDU (FC + data). Minimum = 2.
            let adu_len = 6usize + header.length as usize;

            // F-105-001: if the full ADU is not yet present, stash the tail in carry
            // and break. The next on_data call will complete it.
            if remaining.len() < adu_len {
                // DoS guard: adu_len is at most 6+254=260 (bounded by is_valid_modbus_adu).
                // remaining.len() < adu_len <= 260, so this is always within cap.
                // We check the cap anyway for defense-in-depth.
                if remaining.len() > MAX_ADU_CARRY_BYTES {
                    flow.is_non_modbus = true;
                    self.parse_errors += 1;
                } else {
                    flow.carry.extend_from_slice(remaining);
                }
                break;
            }

            // Full ADU is present. Slice exactly adu_len bytes and dispatch.
            let adu_bytes = &remaining[..adu_len];
            let fc = header.function_code;

            // Dispatch to the detection engine.
            // process_pdu takes &mut self (for counters/findings) and &mut flow (for per-flow
            // state). We pass the flow as a local mut, then re-insert after the loop.
            self.process_pdu(
                flow_key, &mut flow, direction, &header, fc, adu_bytes, timestamp,
            );

            // Advance past exactly this ADU.
            pos += adu_len;
        }

        // Re-insert the (possibly mutated) flow state (with updated carry).
        self.flows.insert(flow_key.clone(), flow);
    }

    /// Finalize a Modbus flow on close (BC-2.14.012 / on_flow_close).
    ///
    /// Removes the per-flow state from `self.flows`. Any pending-table entries
    /// are silently discarded (the flow is gone; no partial-pair findings emitted
    /// on close — this matches HTTP/TLS behavior and BC-2.14.012 semantics).
    fn on_flow_close(&mut self, flow_key: &FlowKey, _reason: CloseReason) {
        self.flows.remove(flow_key);
    }
}

impl StreamAnalyzer for ModbusAnalyzer {
    fn name(&self) -> &'static str {
        "modbus"
    }

    fn summarize(&self) -> crate::analyzer::AnalysisSummary {
        // Delegate to the inherent method (same logic).
        ModbusAnalyzer::summarize(self)
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}

/// Parse the 7-byte MBAP header from a reassembled TCP byte slice.
///
/// Returns `Some(MbapHeader)` when `data.len() >= 8` (7-byte header + 1-byte FC
/// minimum), `None` otherwise. This function is PURE — no validity gate on
/// `protocol_id` or `length` (those belong to `is_valid_modbus_adu`).
///
/// BC-2.14.001 (accept path) + BC-2.14.002 (truncation safety / reject path).
/// VP-022 sub-property A Kani target.
pub fn parse_mbap_header(data: &[u8]) -> Option<MbapHeader> {
    // BC-2.14.002: need at least 7-byte MBAP header + 1-byte function code = 8 bytes.
    // The len >= 8 guard makes data[0..7] safe (no out-of-bounds access below).
    if data.len() < 8 {
        return None;
    }
    Some(MbapHeader {
        transaction_id: u16::from_be_bytes([data[0], data[1]]),
        protocol_id: u16::from_be_bytes([data[2], data[3]]),
        length: u16::from_be_bytes([data[4], data[5]]),
        unit_id: data[6],
        function_code: data[7],
    })
}

/// 3-point Modbus ADU validity gate.
///
/// Returns `true` iff:
/// 1. `h.protocol_id == 0x0000`  (BC-2.14.003)
/// 2. `h.length >= 2`            (BC-2.14.004 lower bound)
/// 3. `h.length <= 254`          (BC-2.14.004 upper bound; PDU max = 253 bytes, Length = 1+253=254)
///
/// BC-2.14.003 + BC-2.14.004. Called by `on_data` (STORY-103) after a successful parse.
/// VP-022 sub-property A gate biconditional target.
pub fn is_valid_modbus_adu(h: &MbapHeader) -> bool {
    h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254
}

/// Classify a Modbus function code into one of five risk/type classes.
///
/// Total function over all 256 u8 values — never panics, no unreachable arm
/// (BC-2.14.005 invariant 1). Exception pre-guard fires first (BC-2.14.006).
///
/// Classification order (matches must be checked in this order):
/// 1. `fc >= 0x80`  → `Exception`  (pre-guard, BC-2.14.006)
/// 2. Write set     → `Write`      (BC-2.14.007)
/// 3. Diagnostic    → `Diagnostic` (BC-2.14.008)
/// 4. Read set      → `Read`       (BC-2.14.005 post.2)
/// 5. `_`           → `Unknown`    (wildcard, totality guarantee)
///
/// VP-022 sub-properties B (totality + set membership) and C (exception biconditional).
pub fn classify_fc(fc: u8) -> FunctionCodeClass {
    // BC-2.14.006: exception pre-guard — must fire BEFORE any other match arm.
    // Any fc with the high bit set is an exception response; recover original via fc & 0x7F.
    if fc >= 0x80 {
        return FunctionCodeClass::Exception;
    }

    match fc {
        // BC-2.14.007: Write-class FCs — state-changing operations (exactly 7 members).
        0x05 | 0x06 | 0x0F | 0x10 | 0x15 | 0x16 | 0x17 => FunctionCodeClass::Write,

        // BC-2.14.008: Diagnostic-class FCs — management/tunneling (exactly 2 members).
        0x08 | 0x2B => FunctionCodeClass::Diagnostic,

        // BC-2.14.005 post.2: Read-class FCs (exactly 10 members).
        0x01 | 0x02 | 0x03 | 0x04 | 0x07 | 0x0B | 0x0C | 0x11 | 0x14 | 0x18 => {
            FunctionCodeClass::Read
        }

        // BC-2.14.005 invariant 1: wildcard arm — totality guarantee, no panic.
        _ => FunctionCodeClass::Unknown,
    }
}

// ---------------------------------------------------------------------------
// VP-022 Kani formal-verification harnesses (sub-properties A, B, C).
// Gated by #[cfg(kani)] — not compiled in normal builds; run via `cargo kani`.
// Harness structure from VP-022 proof skeleton (architecture-delta §2.8).
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // ---- Sub-property A (part 1): parse_mbap_header safety (BC-2.14.001/002) ----
    // Symbolic input: [u8; 12] array + symbolic len <= 12. Proves:
    //   - no panic / no OOB for any (bytes, len) combination
    //   - None iff len < 8
    //   - Some with correct BE field decode when len >= 8
    #[kani::proof]
    fn verify_parse_mbap_header_safety() {
        const MAX_LEN: usize = 12;
        let buf: [u8; MAX_LEN] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        let data = &buf[..len];

        let parsed = parse_mbap_header(data);

        if len < 8 {
            assert!(parsed.is_none());
        } else {
            let h = parsed.expect("len>=8 must parse to Some");
            assert!(h.transaction_id == u16::from_be_bytes([data[0], data[1]]));
            assert!(h.protocol_id == u16::from_be_bytes([data[2], data[3]]));
            assert!(h.length == u16::from_be_bytes([data[4], data[5]]));
            assert!(h.unit_id == data[6]);
            assert!(h.function_code == data[7]);
        }
    }

    // ---- Sub-property A (part 2): is_valid_modbus_adu gate biconditional ----
    // (BC-2.14.003/004): gate is true IFF proto==0 && 2<=len<=254.
    #[kani::proof]
    fn verify_is_valid_modbus_adu_gate() {
        let h = MbapHeader {
            transaction_id: kani::any(),
            protocol_id: kani::any(),
            length: kani::any(),
            unit_id: kani::any(),
            function_code: kani::any(),
        };
        let ok = is_valid_modbus_adu(&h);
        assert!(ok == (h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254));
    }

    // ---- Sub-property B: classify_fc totality (BC-2.14.005/007/008) ----
    // Symbolic fc: u8 (all 256 values). Proves no panic + totality + correct set
    // membership + Unknown-for-undefined FCs. The full biconditional expected-mapping
    // approach means a bug that returns e.g. Read for fc=0x09 (undefined) would be
    // caught — the previous one-sided `if` guards + tautological variant-exhaustion
    // check could not detect such a mapping error.
    #[kani::proof]
    fn verify_classify_fc_total() {
        let fc: u8 = kani::any();
        let class = classify_fc(fc);

        // Compute the expected classification for every possible u8 value.
        // Match order mirrors classify_fc: Exception pre-guard first, then Write,
        // Diagnostic, Read, Unknown. FC sets taken verbatim from the implementation.
        let expected = if fc >= 0x80 {
            FunctionCodeClass::Exception
        } else if matches!(fc, 0x05 | 0x06 | 0x0F | 0x10 | 0x15 | 0x16 | 0x17) {
            FunctionCodeClass::Write
        } else if matches!(fc, 0x08 | 0x2B) {
            FunctionCodeClass::Diagnostic
        } else if matches!(
            fc,
            0x01 | 0x02 | 0x03 | 0x04 | 0x07 | 0x0B | 0x0C | 0x11 | 0x14 | 0x18
        ) {
            FunctionCodeClass::Read
        } else {
            FunctionCodeClass::Unknown
        };

        assert!(class == expected);
    }

    // ---- Sub-property C: exception biconditional + mask invariant (BC-2.14.006) ----
    // Proves the biconditional: classify_fc returns Exception IFF fc has the high bit set.
    // Also proves the mask invariant: (fc & 0x7F) < 0x80 — i.e., the consumer-side
    // high-bit-stripping operation that recovers the original FC always clears the bit.
    // Note: FunctionCodeClass::Exception carries no payload; original-FC recovery is a
    // consumer-side computation (fc & 0x7F), not a function output — so there is nothing
    // further to assert about a returned value here.
    #[kani::proof]
    fn verify_classify_fc_exception_iff_high_bit() {
        let fc: u8 = kani::any();
        assert!((classify_fc(fc) == FunctionCodeClass::Exception) == (fc >= 0x80));
        if fc >= 0x80 {
            let original_fc = fc & 0x7F;
            // Meaningful: proves the mask always clears the high bit.
            assert!(original_fc < 0x80);
        }
    }
}
