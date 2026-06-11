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
///                  (SELECT / OPERATE / DIRECT_OPERATE / DIRECT_OPERATE_NR)
/// - `Restart`    — FC set {0x0D, 0x0E} (COLD_RESTART / WARM_RESTART)
/// - `Management` — remaining DNP3-defined primary FCs (IMMED_FREEZE, INITIALIZE_DATA, …)
/// - `Response`   — FC set {0x81, 0x82, 0x83}
///                  (RESPONSE / UNSOLICITED_RESPONSE / AUTHENTICATE_RESP)
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

impl Default for Dnp3FlowState {
    fn default() -> Self {
        Self {
            carry: Vec::new(),
            is_non_dnp3: false,
            fc_counts: HashMap::new(),
            frame_count: 0,
            parse_errors: 0,
            direct_operate_count: 0,
            window_start_ts: 0,
            direct_operate_emitted: false,
            master_addrs_seen: Vec::new(),
            restart_event_count: 0,
            block_event_count: 0,
            pending_requests: HashMap::new(),
            block_finding_emitted_this_window: false,
            loss_of_control_emitted: false,
            correlation_window_start_ts: 0,
            malformed_in_window: 0,
            malformed_anomaly_emitted: false,
        }
    }
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
        todo!(
            "STORY-107: implement carry-buffer frame walk, desync bail, FIR gate \
             (flow_key={flow_key:?}, data.len={})",
            data.len()
        )
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
    todo!("STORY-106: implement fixed-offset decode with LE u16 for DEST/SRC (data.len={})", data.len())
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
    todo!("STORY-106: implement 3-clause boolean (start1==0x05 && start2==0x64 && length>=5), h={h:?}")
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
/// - Management: {0x07..=0x0C, 0x0F..=0x1A} (other defined primary FCs)
/// - Response:   {0x81, 0x82, 0x83}
/// - Unknown:    all remaining values (wildcard)
///
/// BC-2.15.005 / BC-2.15.006. VP-023 Sub-property B.
pub fn classify_dnp3_fc(fc: u8) -> Dnp3FcClass {
    todo!("STORY-106: implement match with _ => Unknown wildcard (fc=0x{fc:02X})")
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
    todo!("STORY-106: implement formula 5 + length + 2*ceil((length-5)/16) (length={length})")
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
    todo!("STORY-106: return transport_octet & 0x40 != 0 (transport_octet=0x{transport_octet:02X})")
}

/// Returns `true` when the link-layer CONTROL field indicates a primary frame
/// that carries user data (DIR=1, PRM=1, FC=0x03 or 0x04).
///
/// Used to decide whether the frame body after the header CRC contains a
/// transport octet + application data.
///
/// Unit test only (not a Kani target).
pub fn has_user_data(control: u8) -> bool {
    todo!("STORY-106: implement DIR/PRM/FC check from CONTROL octet (control=0x{control:02X})")
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
    // Symbolic input: [u8; 12] array + symbolic len <= 12. Proves:
    //   - no panic / no OOB for any (bytes, len) combination
    //   - None iff len < 10
    //   - Some with correct LE field decode when len >= 10
    #[kani::proof]
    fn verify_parse_dnp3_dl_header_safety() {
        todo!("STORY-106: wire VP-023 Sub-A harness per vp-023-dnp3-parse-safety.md skeleton")
    }

    // ---- Sub-property B: classify_dnp3_fc totality + set membership (BC-2.15.005/006) ----
    //
    // Symbolic input: fc: u8 (all 256 values). Proves totality (no panic,
    // returns a defined variant) and Read/Write/Control/Restart/Response set membership.
    #[kani::proof]
    fn verify_classify_dnp3_fc_total() {
        todo!("STORY-106: wire VP-023 Sub-B harness per vp-023-dnp3-parse-safety.md skeleton")
    }

    // ---- Sub-property C: validity gate biconditional (BC-2.15.004) ----
    //
    // Symbolic input: fully symbolic Dnp3DlHeader. Proves gate is true IFF
    // start1==0x05 && start2==0x64 && length>=5.
    #[kani::proof]
    fn verify_is_valid_dnp3_frame_gate() {
        todo!("STORY-106: wire VP-023 Sub-C harness per vp-023-dnp3-parse-safety.md skeleton")
    }

    // ---- Sub-property D: frame_len arithmetic (BC-2.15.007) ----
    //
    // Symbolic input: length: u8 (all 256 values). Proves None for length<5;
    // correct formula for length>=5; result in [10, 292]; no panic/overflow.
    #[kani::proof]
    fn verify_compute_dnp3_frame_len() {
        todo!("STORY-106: wire VP-023 Sub-D harness per vp-023-dnp3-parse-safety.md skeleton")
    }
}
