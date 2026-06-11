//! DNP3 (IEEE Std 1815-2012) pure-core parser, function-code classifier,
//! per-flow state skeleton, and VP-023 Kani harness stubs (SS-15, CAP-15).
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
//! - `Dnp3FlowState` — per-flow state skeleton (desync latch + carry placeholder)
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
/// - `Management` — remaining DNP3-defined primary FCs (IMMED_FREEZE, INITIALIZE_DATA, …)
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

/// Maximum bytes held in a per-flow carry buffer (ADR-007 Decision 2).
/// LENGTH=255 → frame_len = 292.  Carry buffer is bounded to this value.
#[allow(unused)]
pub const MAX_DNP3_CARRY_BYTES: usize = 292;

/// Number of malformed/structural frames within the 300s correlation window
/// that triggers a T0814 low/med-confidence anomaly finding (BC-2.15.024).
#[allow(unused)]
pub const MALFORMED_ANOMALY_THRESHOLD: u64 = 3;

// ---------------------------------------------------------------------------
// Per-flow state (effectful shell — NOT a Kani target)
// ---------------------------------------------------------------------------

/// Per-flow DNP3 analyzer state.
///
/// Carries the desync latch (`is_non_dnp3`) and the partial-frame accumulation
/// buffer (`carry`).  Additional correlation-window and detection-emission fields
/// are stubs for STORY-107/108/109; they compile but contain no logic yet.
///
/// BC-2.15.009 (desync bail), ADR-007 Decision 4 (full field list).
#[derive(Default)]
#[allow(dead_code)]
pub struct Dnp3FlowState {
    /// Partial frame accumulation buffer.  Max 292 bytes (ADR-007 Decision 2).
    /// Populated in STORY-107; declared here as a carry placeholder.
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

    // --- Direct-operate burst window (BC-2.15.011, STORY-108) ---
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
}

impl Dnp3Analyzer {
    /// Construct a new `Dnp3Analyzer` with the given direct-operate threshold.
    pub fn new(direct_operate_threshold: u32) -> Self {
        Self {
            flows: HashMap::new(),
            direct_operate_threshold,
            fn_code_counts: HashMap::new(),
        }
    }

    /// Process a chunk of reassembled TCP stream data for the given flow.
    ///
    /// Desync bail (BC-2.15.009): if `flow.is_non_dnp3` is already set, returns
    /// immediately without doing any work.
    ///
    /// FIR=1 gate (BC-2.15.008): application-layer FC is extracted only from
    /// first-fragment transport segments (`transport_octet & 0x40 != 0`).
    ///
    /// Full frame-walk and carry-buffer management are implemented in STORY-107.
    pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], _ts: u32) {
        // Look up (or create) the per-flow state entry.
        let flow = self.flows.entry(flow_key).or_default();

        // BC-2.15.009 postcondition 5: if the desync latch is already set, this
        // on_data call is an immediate no-op — no parsing, no metrics, no findings.
        if flow.is_non_dnp3 {
            return;
        }

        // BC-2.15.009: check for valid DNP3 sync word [0x05, 0x64] at offset 0 of
        // the incoming data within the first 16-byte window.
        //
        // v1 implementation (STORY-106 scope):
        //   - If data has at least 2 bytes and data[0..2] != [0x05, 0x64], set
        //     is_non_dnp3 = true and return immediately (no-op).
        //   - If data[0..2] == [0x05, 0x64], the flow is valid DNP3 — do not bail.
        //   - The carry buffer is not populated in STORY-106 (STORY-107 scope).
        //
        // The 16-byte check is: if the first data delivered contains no valid sync
        // at offset 0, bail. This matches the test requirement: 16-byte delivery
        // with no [0x05, 0x64] at byte 0 triggers is_non_dnp3 = true.
        if data.len() >= 2 {
            if data[0] != 0x05 || data[1] != 0x64 {
                // No valid DNP3 sync word at offset 0 — desync bail.
                flow.is_non_dnp3 = true;
                return;
            }
            // Valid sync observed. Continue processing (STORY-107 will add frame walk).
        } else if !data.is_empty() {
            // Less than 2 bytes — cannot determine sync yet. Defer (STORY-107 carry logic).
            // For STORY-106 scope: treat < 2 bytes as insufficient data, no bail yet.
        }

        // BC-2.15.008 FIR=1 gate: extract application FC from FIR=1 transport fragments.
        // For STORY-106 scope, basic frame counting + FC extraction is included here
        // to satisfy AC-008 (tested via transport_is_fir pure fn) and the desync bail.
        // Full carry-buffer frame-walk is STORY-107 scope.
        //
        // Minimum frame: 10-byte header + 1 transport byte + 2 app bytes = 13 bytes for
        // FC extraction. If we have at least 13 bytes and FIR=1, extract the App FC.
        if data.len() >= 13 {
            // Bytes 0-9: header (10 bytes); byte 10: transport octet; byte 11: app ctrl;
            // byte 12: app FC.
            let transport_octet = data[10];
            // STORY-106 scope: frame_count counts sync-valid DELIVERIES, not gate-validated frames.
            //  on_data does NOT call is_valid_dnp3_frame_header here — per-frame validity gating
            //  before counting is part of the STORY-107 frame-walk. (adv Pass-2 B1)
            flow.frame_count += 1;

            // BC-2.15.008 precondition 2 / Invariant 4 / EC-005 (adv Pass-3 F-P3-001):
            // The transport+application layer is present ONLY when the link CONTROL field FC
            // nibble (CONTROL & 0x0F) is CONFIRMED_USER_DATA (0x03) or UNCONFIRMED_USER_DATA
            // (0x04). Other link FCs (e.g. RESET_LINK = 0x00) carry NO transport/app payload;
            // descending into the app layer for those frames is incorrect.
            let control = data[3]; // link CONTROL byte
            if transport_is_fir(transport_octet) && has_user_data(control) {
                // STORY-107 scope: this offset assumes the minimum single-block frame
                // (no interior CRC blocks). Multi-block CRC-block stripping and correct
                // payload_buf indexing are STORY-107 scope (ADR-007 Decision 3).
                let app_fc = data[12];
                *flow.fc_counts.entry(app_fc).or_insert(0) += 1;
                *self.fn_code_counts.entry(app_fc).or_insert(0) += 1;
            }
        } else if data.len() >= 10 {
            // A DNP3 application frame needs >=13 bytes (10 header + 1 transport + 2 app) to carry
            //  an app FC at byte 12. Valid-sync frames of 10-12 bytes have no app FC to extract, so
            //  they are counted but not extracted (correct). Multi-block / short-frame handling is
            //  STORY-107 frame-walk scope. (adv Pass-2 B2)
            // Valid sync present, frame_count can be incremented for short frames.
            // STORY-106 scope: frame_count counts sync-valid DELIVERIES, not gate-validated frames.
            //  on_data does NOT call is_valid_dnp3_frame_header here — per-frame validity gating
            //  before counting is part of the STORY-107 frame-walk. (adv Pass-2 B1)
            flow.frame_count += 1;
        }
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
