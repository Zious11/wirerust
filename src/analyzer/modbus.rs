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
use crate::reassembly::handler::Direction;

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

/// Analyzer-level aggregates for Modbus TCP (STORY-103/104, f2-fix-directives §11.3).
///
/// Flow states are passed in directly to `process_pdu` (STORY-104); dispatcher wiring
/// (the `HashMap<FlowKey, ModbusFlowState>`) is deferred to STORY-105.
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
    ///
    /// NOT YET IMPLEMENTED — stub for Red Gate (STORY-104 TDD step 1).
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
        // Suppress unused-parameter warnings in the stub — all params used in impl.
        let _ = (flow_key, flow, direction, header, fc, data, timestamp);
        todo!("STORY-104: implement process_pdu detection engine (BC-2.14.013–022)")
    }

    /// Produce the six-key `AnalysisSummary` (STORY-104, BC-2.14.021).
    ///
    /// Keys (authoritative set):
    ///   `pdu_count`, `write_count`, `exception_count`, `parse_errors`,
    ///   `function_code_distribution`, `dropped_findings`.
    ///
    /// NOT YET IMPLEMENTED — stub for Red Gate (STORY-104 TDD step 1).
    pub fn summarize(&self) -> AnalysisSummary {
        todo!("STORY-104: implement summarize() returning six-key AnalysisSummary (BC-2.14.021)")
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
