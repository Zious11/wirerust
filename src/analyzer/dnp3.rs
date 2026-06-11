//! DNP3 (IEEE Std 1815-2012) pure-core parser, function-code classifier,
//! per-flow state with carry-buffer frame-walk, and VP-023 Kani harness stubs (SS-15, CAP-15).
//!
//! ## Architecture
//! - `parse_dnp3_dl_header` — pure parse, no validity gate (BC-2.15.001/002/003)
//! - `is_valid_dnp3_frame_header` — 3-point validity gate (BC-2.15.004)
//! - `classify_dnp3_fc` — total FC classification over all 256 u8 values
//!   (BC-2.15.005/006); `_ => Unknown` wildcard guarantees totality; no `unreachable!`
//! - `compute_dnp3_frame_len` — frame-length arithmetic, result in [10, 292]
//!   (BC-2.15.007)
//! - `transport_is_fir` — FIR=1 first-fragment predicate (BC-2.15.008)
//! - `has_user_data` — link-layer control field predicate
//! - `Dnp3FlowState` — per-flow state with carry-buffer frame-walk (implemented in STORY-107)
//! - VP-023 Kani harnesses (sub-properties A, B, C, D) — gated by `#[cfg(kani)]`
//!
//! ## Architecture compliance (ADR-007 Decision 2 / STORY-106 rule set)
//! - Pure-core functions are FREE `fn`s — NOT `impl Dnp3Analyzer` methods.
//!   Kani calls them directly without constructing the analyzer struct.
//! - DEST/SOURCE decoded little-endian ONLY (`u16::from_le_bytes`).
//! - No `unreachable!` in `classify_dnp3_fc` — wildcard `_ => Unknown` is mandatory.
//! - `compute_dnp3_frame_len` uses integer ceil `(u + 15) / 16` — no float math.
//! - `parse_dnp3_dl_header` does NOT check sync or LENGTH validity — separation is
//!   required for VP-023 Sub-A to range over all 2^80 inputs.
//! - This module MUST NOT depend on `crate::analyzer::modbus` or any external DNP3 crate.

#![allow(dead_code)]

use std::collections::HashMap;
use std::net::IpAddr;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Parsed DNP3 data-link layer header (8 header bytes; CRC bytes 8–9 excluded).
///
/// Fields decoded from fixed offsets per IEEE Std 1815-2012 §8.2 and ADR-007 Decision 2:
/// - `start1`      = data[0]  (0x05 for valid DNP3)
/// - `start2`      = data[1]  (0x64 for valid DNP3)
/// - `length`      = data[2]  (LENGTH field; range 5..=255 for valid frames)
/// - `control`     = data[3]
/// - `destination` = u16::from_le_bytes([data[4], data[5]])  (little-endian)
/// - `source`      = u16::from_le_bytes([data[6], data[7]])  (little-endian)
///
/// BC-2.15.001 postconditions 1–6; BC-2.15.003 (LE decode invariant).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dnp3DlHeader {
    /// First sync byte — 0x05 for valid DNP3 frames.
    pub start1: u8,
    /// Second sync byte — 0x64 for valid DNP3 frames.
    pub start2: u8,
    /// LENGTH field (bytes 2): covers CONTROL + DEST + SOURCE + user data.
    /// Valid range: 5..=255.
    pub length: u8,
    /// Link-layer CONTROL octet (byte 3): DIR, PRM, FCB, FCV/DFC bits + FC nibble.
    pub control: u8,
    /// Destination link address, decoded little-endian from bytes 4–5.
    pub destination: u16,
    /// Source link address, decoded little-endian from bytes 6–7.
    pub source: u16,
}

/// Application-layer function-code classification (BC-2.15.005/006).
///
/// Variants:
/// - `Read`       — FC 0x01 (READ)
/// - `Write`      — FC 0x02 (WRITE)
/// - `Control`    — FC set {0x03, 0x04, 0x05, 0x06}
///   (SELECT / OPERATE / DIRECT_OPERATE / DIRECT_OPERATE_NR)
/// - `Restart`    — FC set {0x0D, 0x0E} (COLD_RESTART / WARM_RESTART)
/// - `Management` — remaining DNP3-defined primary FCs (CONFIRM (0x00), IMMED_FREEZE, INITIALIZE_DATA, …)
/// - `Response`   — FC set {0x81, 0x82, 0x83}
///   (RESPONSE / UNSOLICITED_RESPONSE / AUTHENTICATE_RESP)
/// - `Unknown`    — all other FC values (wildcard; guarantees totality per VP-023 Sub-B)
///
/// INVARIANT: `classify_dnp3_fc` MUST contain `_ => Dnp3FcClass::Unknown` as the final
/// match arm. No `unreachable!` is permitted; the wildcard arm is required for the
/// VP-023 Sub-B Kani totality proof.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dnp3FcClass {
    Read,
    Write,
    Control,
    Restart,
    Management,
    Response,
    Unknown,
}

// ---------------------------------------------------------------------------
// Bounded-resource constants (ADR-007 Decision 4)
// ---------------------------------------------------------------------------

/// Maximum outstanding pending control requests per flow for T1691.001
/// request/response correlation.  Oldest entry evicted on overflow.
#[allow(unused)]
pub const MAX_PENDING_REQUESTS: usize = 256;

/// Maximum on-wire DNP3 link frame size; also the per-flow carry-buffer bound
/// (ADR-007 Decision 2).  LENGTH=255 → frame_len = 292 (proven ≤292 by VP-023 Sub-D).
/// This is the **canonical** name, matching BC-2.15.016 postconditions 1–2 and the
/// AC-001..006 test suite.
#[allow(unused)]
pub const MAX_DNP3_FRAME_LEN: usize = 292;

/// Deprecated alias of [`MAX_DNP3_FRAME_LEN`] (the canonical name).
///
/// STORY-106 introduced `MAX_DNP3_CARRY_BYTES` and STORY-107 scaffolding introduced
/// `MAX_DNP3_FRAME_LEN`; both held the same value (292).  Consolidated in STORY-107 to a
/// single source of truth — this alias is retained only to avoid breaking any external
/// reference and is defined in terms of the canonical constant.
#[deprecated(note = "use MAX_DNP3_FRAME_LEN (canonical name per BC-2.15.016)")]
#[allow(unused)]
pub const MAX_DNP3_CARRY_BYTES: usize = MAX_DNP3_FRAME_LEN;

/// Maximum unique master-station source addresses tracked per flow
/// (BC-2.15.016 postconditions 5–6; ADR-007 Decision 4).
/// Once full, new master source addresses are silently ignored.
#[allow(unused)]
pub const MAX_MASTER_ADDRS: usize = 64;

/// Number of malformed/structural frames within the 300s correlation window
/// that triggers a T0814 low/med-confidence anomaly finding (BC-2.15.024).
#[allow(unused)]
pub const MALFORMED_ANOMALY_THRESHOLD: u64 = 3;

/// Detection window for the Direct-Operate burst guard in seconds (BC-2.15.010).
/// Control-class FC counter and `window_start_ts` reset when elapsed exceeds this.
pub const DETECTION_WINDOW_SECS: u32 = 60;

/// Hard upper bound on findings accumulated in `Dnp3Analyzer.all_findings` (BC-2.15.022).
/// Mirrors `modbus::MAX_FINDINGS` (10_000) — consistent DoS cap across analyzers
/// (BC-2.15.022 Invariant 1 / ADR-007 Decision 2).
pub const MAX_FINDINGS: usize = 10_000;

// ---------------------------------------------------------------------------
// Per-flow state (effectful shell — NOT a Kani target)
// ---------------------------------------------------------------------------

/// Per-flow DNP3 analyzer state.
///
/// Carries the desync latch (`is_non_dnp3`) and the partial-frame accumulation
/// buffer (`carry`) — both implemented in STORY-107 (carry-buffer frame-walk,
/// AC-001..006).  Detection-emission and correlation-window fields are stubs for
/// STORY-108/109; they compile but contain no logic yet.
///
/// BC-2.15.009 (desync bail), ADR-007 Decision 4 (full field list).
#[derive(Default)]
#[allow(dead_code)]
pub struct Dnp3FlowState {
    /// Partial frame accumulation buffer.  Max 292 bytes (ADR-007 Decision 2).
    /// Implemented in STORY-107 (frame-walk, AC-001..006, BC-2.15.016 PC1-4).
    pub carry: Vec<u8>,

    /// Set to `true` on desync (no valid DNP3 sync word in first 16 bytes).
    /// All subsequent `on_data` calls for this flow are no-ops once set.
    /// One-way latch: never reset (BC-2.15.009 Invariant 3).
    pub is_non_dnp3: bool,

    // --- Aggregate counters (STORY-107/108) ---
    /// Counts of each application FC seen in this flow.
    pub fc_counts: HashMap<u8, u64>,
    /// Total frames analyzed.
    pub frame_count: u64,
    /// LIFETIME parse-error counter: incremented for every frame that fails
    /// the three-point validity gate.  NEVER reset (ADR-007 Decision 4).
    pub parse_errors: u64,

    // --- Direct-operate burst window (BC-2.15.010, STORY-108) ---
    pub direct_operate_count: u32,
    pub window_start_ts: u32,
    pub direct_operate_emitted: bool,

    // --- Master address tracking (BC-2.15.010, STORY-108) ---
    pub master_addrs_seen: Vec<u16>,

    // --- Correlation-window state (BC-2.15.011/014/015/024, STORY-109) ---
    /// All six fields below reset together at correlation-window expiry (300s).
    pub restart_event_count: u64,
    pub block_event_count: u64,
    pub pending_requests: HashMap<(u16, u8), u32>,
    pub block_finding_emitted_this_window: bool,
    pub loss_of_control_emitted: bool,
    pub correlation_window_start_ts: u32,
    /// Windowed malformed-frame counter for BC-2.15.024 T0814 threshold.
    pub malformed_in_window: u64,
    /// One-shot T0814 guard for BC-2.15.024.
    pub malformed_anomaly_emitted: bool,
}

// ---------------------------------------------------------------------------
// DNP3 analyzer struct (effectful shell — NOT a Kani target)
// ---------------------------------------------------------------------------

/// DNP3 TCP stream analyzer.
///
/// Holds per-flow state keyed by `FlowKey` and analyzer-level aggregates.
/// The pure-core parsing and classification functions are FREE `fn`s below
/// (not methods) so VP-023 Kani harnesses can call them directly.
///
/// ADR-007 Decision 4 (`Dnp3Analyzer` struct layout).
pub struct Dnp3Analyzer {
    /// Per-flow state.
    pub flows: HashMap<FlowKey, Dnp3FlowState>,
    /// Direct-operate burst threshold.  Exposed via CLI `--dnp3-direct-operate-threshold`.
    pub direct_operate_threshold: u32,
    /// Aggregate function-code distribution across all flows: FC byte → count.
    pub fn_code_counts: HashMap<u8, u64>,
    /// Accumulated findings — capped at MAX_FINDINGS (BC-2.15.022).
    pub all_findings: Vec<Finding>,
}

impl Dnp3Analyzer {
    /// Construct a new `Dnp3Analyzer` with the given direct-operate threshold.
    pub fn new(direct_operate_threshold: u32) -> Self {
        Self {
            flows: HashMap::new(),
            direct_operate_threshold,
            fn_code_counts: HashMap::new(),
            all_findings: Vec::new(),
        }
    }

    /// Process a chunk of reassembled TCP stream data for the given flow.
    ///
    /// STORY-107 restructures the STORY-106 skeleton into the real carry-buffer
    /// frame-walk (ADR-007 Decision 2; BC-2.15.016).  The pipeline is:
    ///
    /// 1. **Desync bail FIRST** (BC-2.15.009): if `flow.is_non_dnp3` is already set,
    ///    immediate no-op — carry is NOT touched (EC-004).  On the first delivery, if
    ///    the leading bytes are not the DNP3 sync word `[0x05, 0x64]`, latch and bail.
    /// 2. **Accumulate into carry** with a 292-byte cap (AC-001/EC-003): bytes beyond
    ///    `MAX_DNP3_FRAME_LEN` are discarded and `parse_errors` is incremented once per
    ///    overflowing `on_data` call.
    /// 3. **Frame-walk** (`while` loop, EC-002): consume every complete frame from the head
    ///    of `flow.carry`.  Each frame is gate-validated **before** it is counted
    ///    (SEC-106-001 / adv-B1 gate-before-count; BC-2.15.004).
    ///
    /// FIR=1 + user-data gate (BC-2.15.008): the application FC is extracted only from
    /// first-fragment transport segments (`transport_octet & 0x40 != 0`) whose link
    /// CONTROL nibble is a user-data FC (`has_user_data`).
    pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32) {
        // Look up (or create) the per-flow state entry.
        // Clone flow_key here so it remains available for source_ip resolution in
        // detection branches below (BC-2.15.010/011/012 PC3).
        let flow = self.flows.entry(flow_key.clone()).or_default();

        // --- Step 1: desync bail FIRST (BC-2.15.009; EC-004) -----------------------
        // PC5: if the desync latch is already set, this on_data call is an immediate
        // no-op — no parsing, no metrics, no findings, and the carry is NOT touched.
        if flow.is_non_dnp3 {
            return;
        }

        // BC-2.15.009: a flow is DNP3 only if its first delivered bytes begin with the
        // sync word [0x05, 0x64] at offset 0 (v1 checks offset 0 only).  We apply this
        // check on the FIRST delivery (carry still empty); once a flow has accepted any
        // bytes into carry it is an established DNP3 flow and we do not re-bail.
        if flow.carry.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64) {
            // No valid DNP3 sync word at offset 0 — desync bail. Carry NOT touched.
            flow.is_non_dnp3 = true;
            return;
        }

        // --- Step 2: accumulate into carry with the 292-byte cap (AC-001 / EC-003) --
        // BC-2.15.016 postconditions 1–2: append incoming bytes; if the carry would
        // exceed MAX_DNP3_FRAME_LEN (292), append only up to 292 and DISCARD the excess,
        // incrementing the LIFETIME parse_errors counter once for the overflow event.
        let remaining_capacity = MAX_DNP3_FRAME_LEN - flow.carry.len();
        if data.len() > remaining_capacity {
            flow.carry.extend_from_slice(&data[..remaining_capacity]);
            // Excess bytes beyond 292 are discarded; record one overflow (BC-2.15.016 PC2).
            flow.parse_errors += 1;
        } else {
            flow.carry.extend_from_slice(data);
        }

        // --- Step 3: frame-walk — consume every complete frame from carry's head ----
        // STORY-103 lesson: use a WHILE loop, not an if — a single on_data may carry
        // multiple complete frames (EC-002).
        loop {
            // Guard: need at least 3 bytes to read the LENGTH byte at carry[2].
            if flow.carry.len() < 3 {
                break;
            }

            // SYNC GATE FIRST: the head of an established DNP3 carry must begin with the
            // sync word [0x05, 0x64]. If it does not, the carry is mis-aligned corruption;
            // stop the walk and leave the carry intact (no count, no drain, no parse_error
            // beyond any overflow already recorded above). This is the resync hold-point —
            // the bounded carry (≤292) guarantees no unbounded growth. (BC-2.15.004 sync.)
            // Resync policy (STORY-107 v1, adv Pass-1 F-2): an invalid LENGTH (<5) drains 1 byte to resync;
            //  a mid-carry sync-word loss holds the carry until the 292 cap (then overflow-discards). Both
            //  paths guarantee progress/termination (each non-break iteration drains >=1 byte; carry bounded
            //  <=292). Byte-walk resync on mid-carry sync-loss is deferred to a later detection story.
            if flow.carry[0] != 0x05 || flow.carry[1] != 0x64 {
                break;
            }

            // VALIDITY GATE: LENGTH must yield a computable frame length (LENGTH ≥ 5).
            // compute_dnp3_frame_len returns None for LENGTH < 5 (gate-before-count;
            // SEC-106-001 / adv-B1 / EC-006). On failure: increment parse_errors and
            // advance the carry by one byte to attempt re-sync (BC-2.15.004 PC4 /
            // BC-2.15.008 EC-006). VP-023 Sub-D bounds the returned frame_len to [10, 292].
            let length_byte = flow.carry[2];
            let frame_len = match compute_dnp3_frame_len(length_byte) {
                Some(fl) => fl,
                None => {
                    // Invalid LENGTH (< 5): structural parse error. Advance one byte.
                    flow.parse_errors += 1;
                    flow.carry.drain(..1);
                    continue;
                }
            };

            // Not enough bytes for a complete frame yet — leave the partial in carry (EC-001).
            if flow.carry.len() < frame_len {
                break;
            }

            // Parse the gate-validated header. parse returns Some because
            // carry.len() >= frame_len >= 10; sync and LENGTH≥5 were just validated, so
            // is_valid_dnp3_frame_header holds — the failure arm is defensive only.
            let header = match parse_dnp3_dl_header(&flow.carry[..frame_len]) {
                Some(h) if is_valid_dnp3_frame_header(&h) => h,
                _ => {
                    flow.parse_errors += 1;
                    flow.carry.drain(..frame_len);
                    continue;
                }
            };

            // --- Valid, gate-passed frame: now genuinely count it (BC-2.15.016 PC7). ---
            flow.frame_count += 1;

            // BC-2.15.016 PC5–6: master-direction (DIR=1) frame → record its source
            // address in master_addrs_seen, deduplicated and capped at MAX_MASTER_ADDRS.
            if is_master_frame(header.control)
                && !flow.master_addrs_seen.contains(&header.source)
                && flow.master_addrs_seen.len() < MAX_MASTER_ADDRS
            {
                flow.master_addrs_seen.push(header.source);
            }

            // BC-2.15.008 FIR=1 + user-data gate: extract the application FC only from
            // first-fragment transport segments whose link CONTROL nibble is a user-data FC.
            // Offsets within a single-block frame: byte 10 = transport octet, byte 11 =
            // application control, byte 12 = application FC. Raw carry still holds the
            // header/data-block CRC octets (ADR-007 Decision 3 — CRC not stripped here).
            if frame_len >= 13 && has_user_data(header.control) {
                let transport_octet = flow.carry[10];
                if transport_is_fir(transport_octet) {
                    let app_fc = flow.carry[12];
                    *flow.fc_counts.entry(app_fc).or_insert(0) += 1;
                    *self.fn_code_counts.entry(app_fc).or_insert(0) += 1;

                    // --- Detection branches (STORY-108, BC-2.15.010/011/012) -------
                    // Borrow-checker note: `flow` borrows `self.flows`; we cannot call
                    // `&mut self` methods while `flow` is held.  Instead, we pass the
                    // mutable sub-fields of self directly as separate references.
                    let dest = header.destination;
                    let src = header.source;

                    match classify_dnp3_fc(app_fc) {
                        Dnp3FcClass::Control => {
                            // BC-2.15.010: seed pending_requests for Control-class FCs
                            // (STORY-108 relocates STORY-107 seed here onto gate-validated frame).
                            let app_seq = flow.carry[11] & 0x0F;
                            Self::insert_pending_request(flow, (dest, app_seq), ts);

                            // Detection burst branch.
                            Self::detect_control_class_burst_split(
                                flow,
                                &mut self.all_findings,
                                self.direct_operate_threshold,
                                app_fc,
                                ts,
                                dest,
                                src,
                                &flow_key,
                            );
                        }
                        Dnp3FcClass::Restart => {
                            Self::detect_restart_split(
                                flow,
                                &mut self.all_findings,
                                app_fc,
                                dest,
                                src,
                                ts,
                                &flow_key,
                            );
                        }
                        Dnp3FcClass::Write => {
                            Self::detect_write_split(
                                &mut self.all_findings,
                                dest,
                                src,
                                ts,
                                &flow_key,
                            );
                        }
                        _ => {}
                    }
                }
            }

            // Drain the consumed frame from the head of carry (BC-2.15.016 PC3–4).
            // VP-023 Sub-D guarantees frame_len ∈ [10, 292] and we checked carry.len() >=
            // frame_len above, so this drain can never index out of bounds (AC-006).
            flow.carry.drain(..frame_len);
        }
    }

    // -----------------------------------------------------------------------
    // Detection branches (STORY-108, Tasks 3–7).
    //
    // These are associated functions (not methods) to avoid the Rust borrow
    // conflict between `flow` (borrowed from `self.flows`) and `self`.  Each
    // receives the minimum mutable sub-state it needs.
    // -----------------------------------------------------------------------

    /// Control-class burst detection branch (BC-2.15.010).
    ///
    /// Increments `flow.direct_operate_count`, seeds `flow.window_start_ts` on
    /// the first FC in a window, resets on window expiry, and pushes exactly one
    /// T1692.001 `Finding` when `count > threshold` and the one-shot guard is clear.
    ///
    /// All timestamp arithmetic uses `wrapping_sub` (overflow-checks=true in release;
    /// EC-008 out-of-order pcap safety). Cap check: `findings.len() < MAX_FINDINGS`.
    // 8 args is one above the default clippy limit (7); adding flow_key for BC-2.15.010 PC3
    // source_ip resolution is the minimal change. A refactor into a context struct is tracked
    // as a future cleanup but is out of scope for this adversarial fix (F-108-P1-001).
    #[allow(clippy::too_many_arguments)]
    fn detect_control_class_burst_split(
        flow: &mut Dnp3FlowState,
        findings: &mut Vec<Finding>,
        direct_operate_threshold: u32,
        app_fc: u8,
        now_ts: u32,
        dest: u16,
        src: u16,
        flow_key: &FlowKey,
    ) {
        // BC-2.15.010 postcondition 4: check window expiry BEFORE incrementing.
        // When elapsed > DETECTION_WINDOW_SECS, reset to a fresh window seeded
        // by this incoming FC.
        if flow.direct_operate_count > 0
            && now_ts.wrapping_sub(flow.window_start_ts) > DETECTION_WINDOW_SECS
        {
            flow.direct_operate_count = 1;
            flow.window_start_ts = now_ts;
            flow.direct_operate_emitted = false;
            // Window reset — count=1 never exceeds threshold; return.
            return;
        }

        // BC-2.15.010 postcondition 1: increment counter.
        flow.direct_operate_count += 1;

        // BC-2.15.010 postcondition 2: seed window_start_ts on first FC in window.
        if flow.direct_operate_count == 1 {
            flow.window_start_ts = now_ts;
        }

        // BC-2.15.010 postcondition 3: emit finding when threshold exceeded and guard clear.
        if flow.direct_operate_count > direct_operate_threshold
            && !flow.direct_operate_emitted
            && now_ts.wrapping_sub(flow.window_start_ts) <= DETECTION_WINDOW_SECS
            && findings.len() < MAX_FINDINGS
        {
            let count = flow.direct_operate_count;
            let elapsed = now_ts.wrapping_sub(flow.window_start_ts);
            let threshold = direct_operate_threshold;
            // BC-2.15.010 PC3: resolve master endpoint from FlowKey.
            let master_ip = Self::resolve_master_ip(flow_key);
            findings.push(Finding {
                category: crate::findings::ThreatCategory::Execution,
                verdict: crate::findings::Verdict::Likely,
                confidence: crate::findings::Confidence::Medium,
                summary: format!(
                    "DNP3 unauthorized control command burst: {count} control FCs \
                     in {elapsed}s window (threshold {threshold})"
                ),
                evidence: vec![format!("FC=0x{app_fc:02X} dest={dest:#06X} src={src:#06X}")],
                mitre_techniques: vec!["T1692.001".to_string()],
                source_ip: Some(master_ip),
                timestamp: chrono::DateTime::from_timestamp(now_ts as i64, 0),
                direction: None,
            });
            flow.direct_operate_emitted = true;
        }
    }

    /// Restart-command detection branch (BC-2.15.011).
    ///
    /// Pushes one T0814 `Finding` per occurrence (no threshold guard).
    /// Increments `flow.restart_event_count` UNCONDITIONALLY — even when capped.
    /// Cap check: `findings.len() < MAX_FINDINGS` evaluated before push.
    fn detect_restart_split(
        flow: &mut Dnp3FlowState,
        findings: &mut Vec<Finding>,
        app_fc: u8,
        dest: u16,
        src: u16,
        now_ts: u32,
        flow_key: &FlowKey,
    ) {
        // BC-2.15.011 postcondition 1: push ONE Finding per occurrence (cap gated).
        if findings.len() < MAX_FINDINGS {
            let name = match app_fc {
                0x0D => "COLD_RESTART",
                0x0E => "WARM_RESTART",
                _ => "RESTART",
            };
            // BC-2.15.011 PC1: resolve master endpoint from FlowKey.
            let master_ip = Self::resolve_master_ip(flow_key);
            findings.push(Finding {
                category: crate::findings::ThreatCategory::Execution,
                verdict: crate::findings::Verdict::Likely,
                confidence: crate::findings::Confidence::High,
                summary: format!(
                    "DNP3 restart command observed: FC 0x{app_fc:02X} ({name}) \
                     from src={src:#06X} to dest={dest:#06X}"
                ),
                evidence: vec![format!("FC=0x{app_fc:02X} dest={dest:#06X} src={src:#06X}")],
                mitre_techniques: vec!["T0814".to_string()],
                source_ip: Some(master_ip),
                timestamp: chrono::DateTime::from_timestamp(now_ts as i64, 0),
                direction: None,
            });
        }

        // BC-2.15.011 postcondition 2 / Architecture Compliance Rule 3:
        // restart_event_count is incremented UNCONDITIONALLY (even when capped).
        flow.restart_event_count += 1;

        // T0827 co-emission placeholder: STORY-109 inserts derived T0827 push HERE,
        // after the T0814 push, ensuring most-specific-first ordering (BC-2.15.013).
    }

    /// WRITE-command detection branch (BC-2.15.012).
    ///
    /// Pushes one T0836 `Finding` per occurrence. T0836 only — NOT T1692.001
    /// (ADR-007 Decision 5 / Architecture Compliance Rule 2).
    /// Cap check: `findings.len() < MAX_FINDINGS` evaluated before push.
    fn detect_write_split(
        findings: &mut Vec<Finding>,
        dest: u16,
        src: u16,
        now_ts: u32,
        flow_key: &FlowKey,
    ) {
        if findings.len() < MAX_FINDINGS {
            // BC-2.15.012 PC1: resolve master endpoint from FlowKey.
            let master_ip = Self::resolve_master_ip(flow_key);
            findings.push(Finding {
                category: crate::findings::ThreatCategory::Execution,
                verdict: crate::findings::Verdict::Likely,
                confidence: crate::findings::Confidence::Medium,
                summary: format!(
                    "DNP3 WRITE command observed: parameter modification \
                     from src={src:#06X} to dest={dest:#06X}"
                ),
                evidence: vec![format!("FC=0x02 (WRITE) dest={dest:#06X} src={src:#06X}")],
                mitre_techniques: vec!["T0836".to_string()],
                source_ip: Some(master_ip),
                timestamp: chrono::DateTime::from_timestamp(now_ts as i64, 0),
                direction: None,
            });
        }
    }

    /// Produce the DNP3 analyzer summary (BC-2.15.020).
    ///
    /// Aggregates across all flows: `function_code_distribution` (from
    /// `self.fn_code_counts`, zero-count FCs suppressed), `control_operation_counts`
    /// (per-flow `direct_operate_count`), `total_frames`, `total_parse_errors`,
    /// `flows_analyzed`. Returns a populated `AnalysisSummary` even when zero flows
    /// were processed (all counts zero, maps empty — BC-2.15.020 postcondition 2).
    ///
    /// Does NOT emit new findings (BC-2.15.020 Invariant 3).
    pub fn summarize(&self) -> AnalysisSummary {
        use std::collections::BTreeMap;

        let flows_analyzed = self.flows.len() as u64;
        let total_frames: u64 = self.flows.values().map(|f| f.frame_count).sum();
        let total_parse_errors: u64 = self.flows.values().map(|f| f.parse_errors).sum();

        // BC-2.15.020 postcondition 1: function_code_distribution — only FCs with count > 0.
        // Keys are decimal strings of the FC byte (e.g. "5" for 0x05 DIRECT_OPERATE).
        let function_code_distribution: BTreeMap<String, u64> = self
            .fn_code_counts
            .iter()
            .filter(|&(_, count)| *count > 0)
            .map(|(&fc, &count)| (fc.to_string(), count))
            .collect();

        // BC-2.15.020 postcondition 1: control_operation_counts — per-flow direct_operate_count.
        // Keys are the flow index (0-based) as string to produce a stable JSON object.
        let control_operation_counts: BTreeMap<String, u64> = self
            .flows
            .values()
            .enumerate()
            .map(|(i, flow)| (i.to_string(), flow.direct_operate_count as u64))
            .collect();

        let mut detail = BTreeMap::new();
        detail.insert(
            "function_code_distribution".to_string(),
            serde_json::to_value(function_code_distribution)
                .unwrap_or(serde_json::Value::Object(Default::default())),
        );
        detail.insert(
            "control_operation_counts".to_string(),
            serde_json::to_value(control_operation_counts)
                .unwrap_or(serde_json::Value::Object(Default::default())),
        );
        detail.insert(
            "total_frames".to_string(),
            serde_json::Value::Number(total_frames.into()),
        );
        detail.insert(
            "total_parse_errors".to_string(),
            serde_json::Value::Number(total_parse_errors.into()),
        );
        detail.insert(
            "flows_analyzed".to_string(),
            serde_json::Value::Number(flows_analyzed.into()),
        );

        AnalysisSummary {
            analyzer_name: "DNP3".to_string(),
            packets_analyzed: total_frames,
            detail,
        }
    }

    /// Resolve the DNP3 master (command-originator) endpoint from the flow key.
    ///
    /// **Port-heuristic-only resolution.** DNP3 outstations listen on port 20000
    /// by convention; the opposite endpoint is therefore the master:
    ///
    /// - `lower_port == 20000` → lower endpoint is the outstation, upper is the master.
    /// - otherwise             → upper endpoint is the outstation, lower is the master.
    ///
    /// **Known limitation:** this heuristic is correct for standard DNP3 flows where
    /// exactly one endpoint is on port 20000. It cannot disambiguate when NEITHER
    /// endpoint is on 20000 (non-standard outstation port or proxied capture) — in
    /// that case the function silently returns `lower_ip`, which may or may not be
    /// the actual master.
    ///
    /// **Direction deferral:** this function does NOT use the TCP `Direction` signal
    /// that sibling analyzers (modbus, http, tls) receive, because `Dnp3Analyzer::on_data`
    /// is not yet wired into the dispatcher and does not accept a `direction` argument.
    /// Direction-aware resolution — analogous to `src/analyzer/modbus.rs` ~355–382,
    /// where `direction` selects `client_ip` vs `server_ip` — is deferred to the
    /// DNP3 dispatcher-integration story that adds the `DispatchTarget::Dnp3` arm and
    /// threads `direction` into `on_data`.
    fn resolve_master_ip(flow_key: &FlowKey) -> IpAddr {
        if flow_key.lower_port() == 20000 {
            flow_key.upper_ip()
        } else {
            flow_key.lower_ip()
        }
    }

    /// Insert a pending Control-class request into `flow.pending_requests` with the
    /// DoS-safe bound from BC-2.15.016 postconditions 8–10.
    ///
    /// The map NEVER exceeds `MAX_PENDING_REQUESTS` (256) entries. When the map is full
    /// and the `key` is not already present, the entry with the smallest `request_ts`
    /// (oldest) is evicted **before** the new entry is inserted (ties broken arbitrarily
    /// per PC9). The evicted entry is silently dropped — it generates NO T1691.001
    /// timeout event (PC10).
    fn insert_pending_request(flow: &mut Dnp3FlowState, key: (u16, u8), request_ts: u32) {
        // If the key already exists we are overwriting in place — no growth, no eviction.
        if flow.pending_requests.len() >= MAX_PENDING_REQUESTS
            && !flow.pending_requests.contains_key(&key)
        {
            // Evict the entry with the minimum request_ts (oldest). min_by_key over the
            // (key, ts) pairs; tie-breaking is implementation-defined (BC-2.15.016 PC9).
            if let Some((&oldest_key, _)) = flow
                .pending_requests
                .iter()
                .min_by_key(|&(_, &request_ts)| request_ts)
            {
                flow.pending_requests.remove(&oldest_key);
            }
        }
        flow.pending_requests.insert(key, request_ts);
    }
}

// ---------------------------------------------------------------------------
// Pure-core functions — VP-023 Kani targets (BC-2.15.001..007)
//
// These are FREE `fn`s (not `impl` methods) so Kani harnesses call them
// directly without constructing the analyzer struct.
// ---------------------------------------------------------------------------

/// Parse the DNP3 data-link layer header from a raw byte slice.
///
/// Returns `None` when `data.len() < 10` (no panic — BC-2.15.002).
/// Returns `Some(Dnp3DlHeader)` when `data.len() >= 10`, decoding six fields
/// from fixed byte offsets.  Does NOT validate the sync word or LENGTH range —
/// that is the responsibility of `is_valid_dnp3_frame_header` (BC-2.15.001).
///
/// **DEST and SOURCE are decoded LITTLE-ENDIAN** per IEEE Std 1815-2012
/// (ADR-007 Decision 2; BC-2.15.003 LE invariant).
///
/// BC-2.15.001 / BC-2.15.002 / BC-2.15.003. VP-023 Sub-property A.
pub fn parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader> {
    if data.len() < 10 {
        return None;
    }
    Some(Dnp3DlHeader {
        start1: data[0],
        start2: data[1],
        length: data[2],
        control: data[3],
        // Little-endian decode — BC-2.15.003 LE invariant; ADR-007 Decision 2.
        destination: u16::from_le_bytes([data[4], data[5]]),
        source: u16::from_le_bytes([data[6], data[7]]),
    })
}

/// Three-point post-classification validity gate.
///
/// Returns `true` IFF ALL of:
/// 1. `h.start1 == 0x05`  (first sync byte)
/// 2. `h.start2 == 0x64`  (second sync byte; together = 0x0564 DNP3 sync word)
/// 3. `h.length >= 5`     (LENGTH minimum per DNP3 spec)
///
/// Operates on an already-parsed `Dnp3DlHeader` struct; no slice indexing,
/// no panic possible.  Biconditional: true iff all three conditions hold.
///
/// BC-2.15.004. VP-023 Sub-property C.
pub fn is_valid_dnp3_frame_header(h: &Dnp3DlHeader) -> bool {
    h.start1 == 0x05 && h.start2 == 0x64 && h.length >= 5
}

/// Classify a DNP3 application-layer function code.
///
/// Total over all 256 `u8` values — the final match arm MUST be `_ => Unknown`.
/// No `unreachable!` is permitted (VP-023 Sub-B Kani totality proof relies on the
/// wildcard arm).
///
/// Classification sets (BC-2.15.006):
/// - Read:       {0x01}
/// - Write:      {0x02}
/// - Control:    {0x03, 0x04, 0x05, 0x06}
/// - Restart:    {0x0D, 0x0E}
/// - Management: {0x00, 0x07..=0x0C, 0x0F..=0x1A} (other defined primary FCs)
/// - Response:   {0x81, 0x82, 0x83}
/// - Unknown:    all remaining values (wildcard)
///
/// BC-2.15.005 / BC-2.15.006. VP-023 Sub-property B.
pub fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass {
    match fc {
        // Read set (BC-2.15.006 postcondition 8).
        0x01 => Dnp3FcClass::Read,
        // Write set (BC-2.15.006 postcondition 7).
        0x02 => Dnp3FcClass::Write,
        // Control set: SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR
        // (BC-2.15.006 postconditions 1–4; contiguous range 0x03..=0x06).
        0x03..=0x06 => Dnp3FcClass::Control,
        // Management set — CONFIRM and all defined primary FCs not in other sets.
        // 0x00 = CONFIRM (BC-2.15.005 canonical vector; BC-2.15.006 EC-005)
        // 0x07..=0x0C = IMMED_FREEZE through FREEZE_AT_TIME_NR
        // 0x0F..=0x1A = INITIALIZE_DATA through various management FCs
        // (BC-2.15.006 EC-009: 0x0F INITIALIZE_DATA is Management, NOT Restart)
        0x00 | 0x07..=0x0C | 0x0F..=0x1A => Dnp3FcClass::Management,
        // Restart set: COLD_RESTART / WARM_RESTART
        // (BC-2.15.006 postconditions 5–6).
        0x0D | 0x0E => Dnp3FcClass::Restart,
        // Response set: RESPONSE / UNSOLICITED_RESPONSE / AUTHENTICATE_RESP
        // (BC-2.15.006 postconditions 9–11).
        0x81..=0x83 => Dnp3FcClass::Response,
        // Wildcard: all remaining values → Unknown.
        // NO `unreachable!` — this wildcard arm is required for VP-023 Sub-B totality.
        _ => Dnp3FcClass::Unknown,
    }
}

/// Compute the total on-wire frame length for a given DNP3 LENGTH field value.
///
/// Returns `None` for `length < 5` (minimum valid LENGTH is 5).
/// For `length` in `5..=255`:
/// ```text
/// num_user_octets = (length as usize) - 5
/// num_data_blocks = (num_user_octets + 15) / 16   // integer ceil
/// frame_len       = 5 + (length as usize) + 2 * num_data_blocks
/// ```
/// Result is always in `[10, 292]`; no overflow (ADR-007 Decision 2).
/// Uses integer ceil — no floating-point arithmetic.
///
/// BC-2.15.007. VP-023 Sub-property D.
pub fn compute_dnp3_frame_len(length: u8) -> Option<usize> {
    if length < 5 {
        return None;
    }
    let u = (length as usize) - 5;
    let blocks = u.div_ceil(16); // integer ceil(u / 16) — BC-2.15.007, no float
    Some(5 + length as usize + 2 * blocks)
}

/// Returns `true` when the transport-layer FIR (First) bit is set in the
/// transport octet (`transport_octet & 0x40 != 0`).
///
/// A FIR=1 transport segment carries the start of a new application-layer
/// message; the application FC is at `payload_buf[2]`.  FIR=0 segments are
/// continuation fragments (BC-2.15.008).
///
/// BC-2.15.008. Unit test only (not a Kani target).
pub fn transport_is_fir(transport_octet: u8) -> bool {
    transport_octet & 0x40 != 0
}

/// Returns `true` when the link-layer CONTROL field's function-code nibble
/// (`control & 0x0F`) is CONFIRMED_USER_DATA (0x03) or UNCONFIRMED_USER_DATA (0x04) —
/// the only link FCs that carry a transport+application payload
/// (BC-2.15.008 precondition 2 / Invariant 4). The DIR and PRM bits are NOT inspected.
///
/// Used to decide whether the frame body after the header CRC contains a
/// transport octet + application data.
///
/// Unit test only (not a Kani target).
pub fn has_user_data(control: u8) -> bool {
    let link_fc = control & 0x0F;
    link_fc == 0x03 || link_fc == 0x04
}

/// Returns `true` when the link-layer CONTROL field has the DIR bit set
/// (`control & 0x10 != 0`), indicating a master-direction frame (DIR=1).
///
/// Used by the master-address tracking logic (BC-2.15.016 postconditions 5–6)
/// to decide whether the frame's source address should be recorded in
/// `flow.master_addrs_seen`.  Implemented in STORY-107 Task 5.
///
/// Unit test only (not a Kani target).
#[allow(unused)]
pub fn is_master_frame(control: u8) -> bool {
    // BC-2.15.016 postcondition 5 (PC5): DIR bit is bit 4 (mask 0x10). DIR=1 → master.
    control & 0x10 != 0
}

// ---------------------------------------------------------------------------
// VP-023 Kani formal-verification harnesses (sub-properties A, B, C, D).
// Gated by #[cfg(kani)] — not compiled in normal builds; run via `cargo kani`.
// Harness structure from VP-023 proof skeleton (vp-023-dnp3-parse-safety.md).
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // ---- Sub-property A: parse_dnp3_dl_header safety (BC-2.15.001/002/003) ----
    //
    // MAX_LEN = 12 covers: the len<10 reject band (0..=9), the minimum accept
    // (len==10), and lengths with a couple of user bytes visible (11..=12) to
    // ensure sub-B/C paths remain representable. No allocation, no loop.
    const MAX_LEN: usize = 12;

    #[kani::proof]
    fn verify_parse_dnp3_dl_header_safety() {
        let buf: [u8; MAX_LEN] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= MAX_LEN);
        let data = &buf[..len];

        // (A.3) No panic / no OOB: calling over the symbolic slice proves
        // absence of out-of-bounds indexing for every length 0..=12.
        let parsed = parse_dnp3_dl_header(data);

        // (A.1) len<10 => None ; (A.2) len>=10 => Some.
        if len < 10 {
            assert!(parsed.is_none());
        } else {
            let h = parsed.expect("len>=10 must parse to Some");
            // (A.2) field decode correctness.
            assert!(h.start1 == data[0]);
            assert!(h.start2 == data[1]);
            assert!(h.length == data[2]);
            assert!(h.control == data[3]);
            // Little-endian DEST/SOURCE (BC-2.15.003).
            assert!(h.destination == u16::from_le_bytes([data[4], data[5]]));
            assert!(h.source == u16::from_le_bytes([data[6], data[7]]));
        }
    }

    // ---- Sub-property C: validity gate biconditional (BC-2.15.004) ----
    #[kani::proof]
    fn verify_is_valid_dnp3_frame_gate() {
        let h = Dnp3DlHeader {
            start1: kani::any(),
            start2: kani::any(),
            length: kani::any(),
            control: kani::any(),
            destination: kani::any(),
            source: kani::any(),
        };
        let ok = is_valid_dnp3_frame_header(&h);
        // Gate is true IFF sync matches AND LENGTH >= 5.
        assert!(ok == (h.start1 == 0x05 && h.start2 == 0x64 && h.length >= 5));
    }

    // ---- Sub-property B: classify_dnp3_fc totality + set membership (BC-2.15.005/006) ----
    //
    // Symbolic input: a single u8 (all 256 values). The match is exhaustive by
    // construction; "no panic" + a returned variant proves totality.
    #[kani::proof]
    fn verify_classify_dnp3_fc_total() {
        let fc: u8 = kani::any();
        let class = classify_dnp3_fc(fc); // must return for every u8

        // Read set (BC-2.15.006).
        if matches!(fc, 0x01) {
            assert!(class == Dnp3FcClass::Read);
        }
        // Write set (BC-2.15.006).
        if matches!(fc, 0x02) {
            assert!(class == Dnp3FcClass::Write);
        }
        // Control set (BC-2.15.006 — SELECT/OPERATE/DIRECT_OPERATE/DIRECT_OPERATE_NR).
        if matches!(fc, 0x03 | 0x04 | 0x05 | 0x06) {
            assert!(class == Dnp3FcClass::Control);
        }
        // Restart set (BC-2.15.006 — COLD_RESTART/WARM_RESTART).
        if matches!(fc, 0x0D | 0x0E) {
            assert!(class == Dnp3FcClass::Restart);
        }
        // Management set (BC-2.15.006 EC-005/006/009; BC-2.15.005 canonical vector 0x00).
        // 0x00 = CONFIRM (LOCKED: CONFIRM → Management per VP-023 v1.4);
        // 0x07..=0x0C = IMMED_FREEZE..FREEZE_AT_TIME_NR;
        // 0x0F..=0x1A = INITIALIZE_DATA and remaining defined primary FCs.
        if matches!(fc, 0x00 | 0x07..=0x0C | 0x0F..=0x1A) {
            assert!(class == Dnp3FcClass::Management);
        }
        // Response set (BC-2.15.006).
        if matches!(fc, 0x81 | 0x82 | 0x83) {
            assert!(class == Dnp3FcClass::Response);
        }
        // Totality witness: returned value is one of the defined variants.
        assert!(matches!(
            class,
            Dnp3FcClass::Read
                | Dnp3FcClass::Write
                | Dnp3FcClass::Control
                | Dnp3FcClass::Restart
                | Dnp3FcClass::Management
                | Dnp3FcClass::Response
                | Dnp3FcClass::Unknown
        ));
    }

    // ---- Sub-property D: frame_len arithmetic (BC-2.15.007) ----
    //
    // Symbolic input: a single u8 (all 256 LENGTH values).
    // Proves: None for length<5; correct formula for length>=5; result in [10,292].
    #[kani::proof]
    fn verify_compute_dnp3_frame_len() {
        let length: u8 = kani::any();
        let result = compute_dnp3_frame_len(length);

        if length < 5 {
            // (D.1) Below minimum: must return None.
            assert!(result.is_none());
        } else {
            // (D.2) Valid range: formula must hold and result in [10, 292].
            let fl = result.expect("length>=5 must return Some");
            let u = (length as usize) - 5;
            let blocks = (u + 15) / 16; // ceil(u / 16)
            let expected = 5 + (length as usize) + 2 * blocks;
            assert!(fl == expected);
            // (D.3) Bounds invariant.
            assert!(fl >= 10);
            assert!(fl <= 292);
        }
    }
}
