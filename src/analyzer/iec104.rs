//! IEC 60870-5-104 (IEC-104) pure-core APCI header parser and post-classification validity gate.
//!
//! Subsystem SS-19, CAP-19 — `analyzer/iec104.rs` (C-27).
//!
//! ## Architecture (ADR-013 Decisions 1, 3, 8)
//!
//! All parse functions in this module are **pure-core free `fn`s** — no `self`, no I/O, no global
//! state mutation. This is a hard constraint for VP-044 Kani formal verification amenability
//! (ADR-013 Decision 8).
//!
//! - `parse_apci_header` — 6-byte APCI header parse; None on short/invalid input
//!   (BC-2.19.001–005); VP-044 Kani target.
//! - `is_valid_iec104_frame` — post-classification validity gate; 2-byte check
//!   (BC-2.19.006); VP-047 cargo-fuzz covered.
//! - `Iec104ParseError` — error type skeleton (extended in STORY-168).
//! - VP-044 Kani harness skeleton under `#[cfg(kani)]` (full proof run: STORY-174).
//!
//! ## Behavioral contracts
//! - BC-2.19.001: `parse_apci_header` returns None for input shorter than 6 bytes.
//! - BC-2.19.002: `parse_apci_header` returns None for start byte ≠ 0x68.
//! - BC-2.19.003: `parse_apci_header` returns None for LEN < 4.
//! - BC-2.19.004: `parse_apci_header` returns None for LEN > 253.
//! - BC-2.19.005: `parse_apci_header` returns Some(ApciHeader) for valid input; CF1–CF4 verbatim.
//! - BC-2.19.006: `is_valid_iec104_frame` is a lightweight 2-byte post-classification gate.
//!
//! ## Architecture compliance (ADR-013 Decision 7 — licensing)
//! Forbidden dependencies (BANNED — licensing violation):
//! - `iec60870-5` crate (proprietary, "NOT FREE for commercial/production use")
//! - Wireshark IEC-104 dissector `packet-104.c` (GPLv2)
//! - lib60870 / MZ Automation (GPLv3)
//!
//! This module is an original Rust implementation derived from IEC 60870-5-104:2006 framing
//! diagrams only. Zero lines are borrowed from any external implementation.

use crate::findings::Finding;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Parsed APCI (Application Protocol Control Information) header for IEC 60870-5-104 frames.
///
/// The APCI header occupies exactly 6 bytes on the wire:
/// - `start` (byte 0): always `0x68` for valid IEC-104 frames (fixed by IEC 60870-5-104 §5.1).
/// - `len`   (byte 1): LEN field; valid range [4, 253]. Total on-wire frame = `len + 2` bytes.
///   Maximum APDU = 255 bytes (LEN=253 + 2 prefix bytes).
/// - `cf1`   (byte 2): Control field octet 1. Low 2 bits discriminate I/S/U frame format.
/// - `cf2`   (byte 3): Control field octet 2.
/// - `cf3`   (byte 4): Control field octet 3.
/// - `cf4`   (byte 5): Control field octet 4.
///
/// Populated by `parse_apci_header` on the accept path (BC-2.19.005 postconditions 1–6).
/// Bytes beyond index 5 are not accessed by `parse_apci_header`.
///
/// ## Integer overflow safety
/// `len + 2` is overflow-free for all `len` in `[4, 253]`: maximum value = 255 (fits in u8).
/// VP-044 Kani harness proves this property (ADR-013 Decision 8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApciHeader {
    /// Start byte; always `0x68` for valid IEC-104 frames (IEC 60870-5-104 §5.1).
    pub start: u8,
    /// LEN field: byte count of the frame following the two-byte start/LEN prefix.
    /// Valid range: [4, 253]. Total on-wire frame = `len + 2` bytes.
    pub len: u8,
    /// Control field octet 1 (CF1). Low 2 bits discriminate frame format:
    /// bit 0 = 0 → I-format; bits 1:0 = 0b01 → S-format; bits 1:0 = 0b11 → U-format.
    pub cf1: u8,
    /// Control field octet 2 (CF2). For I-format: upper byte of N(S) send sequence counter.
    pub cf2: u8,
    /// Control field octet 3 (CF3). For I/S-format: lower byte of N(R) receive sequence counter.
    pub cf3: u8,
    /// Control field octet 4 (CF4). For I/S-format: upper byte of N(R) receive sequence counter.
    pub cf4: u8,
}

/// Parse error type for IEC-104 APCI parsing.
///
/// Skeleton — additional variants (e.g., `InvalidStartByte`, `LenOutOfRange`) are scoped
/// to STORY-168. The current single variant covers only the length-reject path.
///
/// BC-2.19.001–004 (all reject paths). Architecture mapping: SS-19 (src/analyzer/iec104.rs C-27).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Iec104ParseError {
    /// The input slice is shorter than the 6-byte APCI header minimum (BC-2.19.001).
    Incomplete,
}

// ---------------------------------------------------------------------------
// Frame format discrimination (STORY-168 — BC-2.19.007–009; ADR-013 Decision 4)
// ---------------------------------------------------------------------------

/// IEC-104 APCI frame format classification.
///
/// Determined by the low 2 bits of CF1 per IEC 60870-5-104 §5.1 and
/// ADR-013 Decision 4. VP-046 proptest verifies totality over all 256 CF1 values
/// (BC-2.19.009 invariant 1; `proptest_vp046_frame_format_totality`; STORY-174 full run).
///
/// ## Variants
/// - `IFormat` — carries an ASDU payload with N(S)/N(R) sequence numbers. `cf1 & 0x01 == 0x00`
///   (BC-2.19.007).
/// - `SFormat` — supervisory-only with N(R) counter; no ASDU payload. `cf1 & 0x03 == 0x01`
///   (BC-2.19.008).
/// - `UFormat` — unnumbered session-control commands (STARTDT/STOPDT/TESTFR). `cf1 & 0x03 == 0x03`
///   (BC-2.19.009).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameFormat {
    /// I-format (Information): bit 0 of CF1 is 0. `cf1 & 0x01 == 0x00` (BC-2.19.007).
    IFormat,
    /// S-format (Supervisory): bits 1:0 of CF1 are `0b01`. `cf1 & 0x03 == 0x01` (BC-2.19.008).
    SFormat,
    /// U-format (Unnumbered): bits 1:0 of CF1 are `0b11`. `cf1 & 0x03 == 0x03` (BC-2.19.009).
    UFormat,
}

// ---------------------------------------------------------------------------
// U-frame canonical CF1 constants (ADR-013 Decision 5)
// ---------------------------------------------------------------------------

/// STARTDT-act CF1 value: activates data transfer; no finding emitted (BC-2.19.010).
pub const U_STARTDT_ACT: u8 = 0x07;

/// STARTDT-con CF1 value: confirms data transfer activation; no finding emitted (BC-2.19.010).
pub const U_STARTDT_CON: u8 = 0x0B;

/// STOPDT-act CF1 value: halts data acquisition.
/// Emits T0881 "Service Stop" with `Verdict::Possible` (if started) or `Verdict::Likely`
/// (if no prior STARTDT; BC-2.19.011/012).
pub const U_STOPDT_ACT: u8 = 0x13;

/// STOPDT-con CF1 value: confirms data transfer deactivation; sets `session_started = false`;
/// no finding on ACT-only MVP (BC-2.19.012).
pub const U_STOPDT_CON: u8 = 0x23;

/// TESTFR-act CF1 value: keepalive request; no finding; session state unchanged (BC-2.19.013).
pub const U_TESTFR_ACT: u8 = 0x43;

/// TESTFR-con CF1 value: keepalive response; no finding; session state unchanged (BC-2.19.013).
pub const U_TESTFR_CON: u8 = 0x83;

// ---------------------------------------------------------------------------
// Per-flow state (STORY-168 — introduces session_started; STORY-171 wires N(S) fields)
// ---------------------------------------------------------------------------

/// Per-flow state for the IEC-104 passive analyzer (SS-19, ADR-013).
///
/// Five fields per the SS-19 architecture shard (v1.6). `session_started` and the carry
/// buffers are introduced and wired in STORY-168. `last_ns_c2s`/`last_ns_s2c` are declared
/// here per the SS-19 field inventory but their behavior is wired in STORY-171 (N(S)
/// desync detection, BC-2.19.024).
///
/// ## Fields
/// - `carry_c2s`: reassembly carry buffer for C→S direction; max `MAX_IEC104_CARRY_BYTES`
///   = 255 bytes (BC-2.19.025–027).
/// - `carry_s2c`: reassembly carry buffer for S→C direction; same bound.
/// - `session_started`: `true` iff a STARTDT-act/con has been observed without a subsequent
///   STOPDT-con (BC-2.19.010/012). Initialized `false` via `Default`.
/// - `last_ns_c2s`: last observed 15-bit N(S) in the C→S direction. `None` = no I-frame
///   seen yet; first I-frame sets baseline without emitting a desync finding (BC-2.19.024;
///   STORY-171).
/// - `last_ns_s2c`: last observed 15-bit N(S) in the S→C direction. Same semantics
///   (STORY-171).
#[derive(Debug, Default)]
pub struct Iec104FlowState {
    /// Directional carry buffer for client-to-server APCI stream reassembly.
    /// Max 255 bytes (`MAX_IEC104_CARRY_BYTES`; BC-2.19.025). Wired in STORY-171+.
    pub carry_c2s: Vec<u8>,
    /// Directional carry buffer for server-to-client APCI stream reassembly.
    /// Max 255 bytes. Wired in STORY-171+.
    pub carry_s2c: Vec<u8>,
    /// STARTDT/STOPDT session state flag.
    /// `true` after STARTDT-act (`0x07`) or STARTDT-con (`0x0B`).
    /// `false` after STOPDT-con (`0x23`) or at flow initialization.
    /// Governs T0881 confidence level in STOPDT-act handling (BC-2.19.010–012).
    pub session_started: bool,
    /// Last observed N(S) send-sequence counter in C→S direction.
    /// `None` before the first I-frame in this direction (BC-2.19.024). Wired in STORY-171.
    pub last_ns_c2s: Option<u16>,
    /// Last observed N(S) send-sequence counter in S→C direction.
    /// `None` before the first I-frame in this direction (BC-2.19.024). Wired in STORY-171.
    pub last_ns_s2c: Option<u16>,
}

// ---------------------------------------------------------------------------
// Pure-core frame format classifier (VP-046 proptest target — ADR-013 Decision 4)
// ---------------------------------------------------------------------------

/// Classify the IEC-104 APCI frame format from CF1 octet 1.
///
/// Pure-core free function — no I/O, no global state, no side effects. Total over all 256
/// u8 CF1 values: every input maps to exactly one `FrameFormat` variant with no unhandled
/// case and no panic (BC-2.19.007–009; ADR-013 Decision 4).
///
/// ## Classification rule
/// Examines only the low 2 bits of CF1 (`cf1 & 0x03`):
/// - `cf1 & 0x01 == 0x00` (bit 0 = 0) → `FrameFormat::IFormat` (BC-2.19.007)
/// - `cf1 & 0x03 == 0x01` (bits1:0 = 0b01) → `FrameFormat::SFormat` (BC-2.19.008)
/// - `cf1 & 0x03 == 0x03` (bits1:0 = 0b11) → `FrameFormat::UFormat` (BC-2.19.009)
///
/// Note: `cf1 & 0x03 == 0x02` maps to I-format (bit 0 = 0) per IEC 60870-5-104 §5.1;
/// the I-format guard `cf1 & 0x01 == 0x00` absorbs both `0bXX00` and `0bXX10` values.
///
/// ## Purity and VP-046 seam
/// Not amenable to Kani (pure discriminant over the full u8 range; proptest is correct tool).
/// VP-046 `proptest_vp046_frame_format_totality` exercises all 256 CF1 values exhaustively
/// (anchored STORY-168; full run STORY-174). See ADR-013 §Trade-offs.
pub fn classify_frame_format(cf1: u8) -> FrameFormat {
    // I-format: bit 0 of CF1 is 0 (absorbs both 0bXX00 and 0bXX10 values).
    // Check bit 0 first; the 0x01 mask is strictly narrower than the 0x03 mask.
    if cf1 & 0x01 == 0x00 {
        FrameFormat::IFormat
    } else if cf1 & 0x03 == 0x01 {
        // S-format: bits 1:0 = 0b01
        FrameFormat::SFormat
    } else {
        // U-format: bits 1:0 = 0b11 (the only remaining case; cf1 & 0x03 == 0x03).
        // Totality proof: all 256 u8 values are covered — no unhandled case, no panic
        // (BC-2.19.009 invariant 1; VP-046 proptest exhaustively verifies this;
        // ADR-013 Decision 4).
        FrameFormat::UFormat
    }
}

// ---------------------------------------------------------------------------
// Effectful U-format session state machine (ADR-013 Decision 5)
// ---------------------------------------------------------------------------

/// Process a U-format APCI frame, update per-flow session state, and return a finding.
///
/// Effectful free function — mutates `Iec104FlowState::session_started` and may return
/// `Some(Finding)`. Called by the dispatcher after `classify_frame_format` returns
/// `FrameFormat::UFormat` (ADR-013 Decision 5; BC-2.19.010–014).
///
/// ## ACT-only MVP dispatch table (ADR-013 Decision 5)
///
/// | CF1    | Command      | State change              | Finding emitted                    |
/// |--------|--------------|---------------------------|------------------------------------|
/// | `0x07` | STARTDT-act  | `session_started = true`  | None (BC-2.19.010)                 |
/// | `0x0B` | STARTDT-con  | `session_started = true`  | None (BC-2.19.010)                 |
/// | `0x13` | STOPDT-act   | `session_started = false` | T0881 `Possible` (was started) /   |
/// |        |              |                           | T0881 `Likely` (not started)       |
/// |        |              |                           | (BC-2.19.011/012)                  |
/// | `0x23` | STOPDT-con   | `session_started = false` | None (ACT-only MVP; BC-2.19.012)   |
/// | `0x43` | TESTFR-act   | unchanged                 | None (BC-2.19.013)                 |
/// | `0x83` | TESTFR-con   | unchanged                 | None (BC-2.19.013)                 |
/// | other  | Non-canonical| unchanged                 | T0814 `Possible` (BC-2.19.014;     |
/// |        | U-frame      |                           | CVE-2026-1773)                     |
///
/// ## Technique IDs
/// - `"T0881"` — "Service Stop" (`IcsInhibitResponseFunction`): STOPDT-act detection.
///   Catalog entry added atomically in STORY-173 per ADR-013 Decision 9/10 (BC-2.10.010).
/// - `"T0814"` — "Denial of Service": non-canonical U-frame CF1 (CVE-2026-1773).
///
/// ## Purity boundary (ADR-013 Decision 4)
/// `classify_frame_format` is pure; `process_u_frame` is effectful. These two functions
/// MUST remain separate — `classify_frame_format` MUST NOT read or write `Iec104FlowState`.
pub fn process_u_frame(state: &mut Iec104FlowState, cf1: u8) -> Option<Finding> {
    use crate::findings::{Confidence, ThreatCategory, Verdict};

    // L1: defensive guard — process_u_frame is only valid for U-format CF1 values
    // (bits1:0 = 0b11). This assertion fails fast in debug builds if a mis-dispatch
    // occurs (e.g., STORY-173 dispatcher sends an I/S-format frame here by mistake).
    // Compiled out in release builds (debug_assert is a no-op in --release).
    debug_assert!(
        classify_frame_format(cf1) == FrameFormat::UFormat,
        "process_u_frame called with non-U-format CF1: {cf1:#04x}"
    );

    match cf1 {
        // STARTDT-act / STARTDT-con: activate data transfer; session goes live; no finding.
        // Idempotent: if already started, session_started remains true (BC-2.19.010).
        U_STARTDT_ACT | U_STARTDT_CON => {
            state.session_started = true;
            None
        }

        // STOPDT-act: halt data acquisition; emit T0881 "Service Stop" (BC-2.19.011/012).
        // Confidence depends on whether the session was previously started:
        //   - session_started=true  → Verdict::Possible  (normal stop, but warrant attention)
        //   - session_started=false → Verdict::Likely    (stop without prior start is anomalous;
        //                            BC-2.19.012 postcondition 3: note added to evidence)
        U_STOPDT_ACT => {
            let was_started = state.session_started;
            let verdict = if was_started {
                Verdict::Possible
            } else {
                Verdict::Likely
            };
            state.session_started = false;
            // BC-2.19.012 postcondition 3: when emitted on the Likely path (no prior STARTDT),
            // include a distinguishing note in evidence so analysts can identify cold-start
            // STOPDT-act without correlating the session timeline themselves.
            let mut evidence = vec![format!("CF1=0x{cf1:02X} (STOPDT-act)")];
            if !was_started {
                evidence.push("STOPDT received without prior STARTDT on this flow".to_string());
            }
            Some(Finding {
                category: ThreatCategory::Impact,
                verdict,
                confidence: Confidence::Medium,
                summary: format!(
                    "IEC-104 STOPDT-act received: CF1=0x{cf1:02X} — \
                     ICS data-transfer service stop request observed \
                     (T0881 inhibit-response-function; BC-2.19.011/012)"
                ),
                evidence,
                mitre_techniques: vec!["T0881".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
        }

        // STOPDT-con: confirms deactivation; sets session_started=false; no finding
        // (ACT-only MVP per ADR-013 Decision 5; BC-2.19.012).
        U_STOPDT_CON => {
            state.session_started = false;
            None
        }

        // TESTFR-act / TESTFR-con: keepalive; no finding; session state unchanged
        // (BC-2.19.013).
        U_TESTFR_ACT | U_TESTFR_CON => None,

        // Non-canonical U-frame CF1: any value with bits1:0=0b11 that is not one of the
        // six canonical commands is a protocol anomaly (CVE-2026-1773; BC-2.19.014).
        // Fail-closed: session state is NOT advanced (invariant 1).
        _ => Some(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Possible,
            confidence: Confidence::Medium,
            summary: format!(
                "IEC-104 non-canonical U-frame CF1=0x{cf1:02X}: \
                 CF1 bits1:0=0b11 but not in canonical set \
                 {{0x07,0x0B,0x13,0x23,0x43,0x83}} — \
                 potential CVE-2026-1773 denial-of-service attack (T0814; BC-2.19.014)"
            ),
            evidence: vec![format!(
                "CF1=0x{cf1:02X} not in canonical U-frame set \
                 {{STARTDT-act=0x07, STARTDT-con=0x0B, STOPDT-act=0x13, \
                 STOPDT-con=0x23, TESTFR-act=0x43, TESTFR-con=0x83}}"
            )],
            mitre_techniques: vec!["T0814".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }),
    }
}

// ---------------------------------------------------------------------------
// Pure-core free functions (VP-044 Kani targets — ADR-013 Decision 8)
// ---------------------------------------------------------------------------

/// Parse the 6-byte APCI header from a byte slice.
///
/// Returns `Some(ApciHeader)` iff ALL of the following hold:
/// 1. `data.len() >= 6` (BC-2.19.001)
/// 2. `data[0] == 0x68` — IEC-104 start byte (BC-2.19.002)
/// 3. `data[1] >= 4`   — LEN lower bound (BC-2.19.003)
/// 4. `data[1] <= 253` — LEN upper bound (BC-2.19.004)
///
/// Returns `None` for any input that fails any of the above guards.
///
/// On the accept path, returns `Some(ApciHeader { start: data[0], len: data[1],
/// cf1: data[2], cf2: data[3], cf3: data[4], cf4: data[5] })`. Bytes beyond
/// index 5 are never accessed (BC-2.19.005 postcondition 6).
///
/// ## Purity
/// Pure-core free function: no I/O, no global state, no side effects.
/// Same input always produces the same output. VP-044 Kani target (ADR-013 Decision 8).
///
/// ## Arithmetic safety
/// For any returned `Some(h)`: `h.len` is in `[4, 253]`; `h.len as usize + 2` is in
/// `[6, 255]` — no integer overflow. Proved by VP-044 Kani harness (STORY-174).
pub fn parse_apci_header(data: &[u8]) -> Option<ApciHeader> {
    // BC-2.19.001: must have at least 6 bytes for the full APCI header.
    if data.len() < 6 {
        return None;
    }
    // BC-2.19.002: start byte must be 0x68 (fixed by IEC 60870-5-104 §5.1).
    if data[0] != 0x68 {
        return None;
    }
    let len = data[1];
    // BC-2.19.003: LEN must be at least 4 (4 control octets; minimum U-frame with no ASDU).
    if len < 4 {
        return None;
    }
    // BC-2.19.004: LEN must be at most 253 (so that len+2 ≤ 255, no u8 overflow).
    if len > 253 {
        return None;
    }
    // BC-2.19.005: all guards passed — extract CF1–CF4 verbatim from bytes [2..6].
    // Bytes beyond index 5 are not accessed (postcondition 6).
    Some(ApciHeader {
        start: 0x68,
        len,
        cf1: data[2],
        cf2: data[3],
        cf3: data[4],
        cf4: data[5],
    })
}

/// Post-classification validity gate for IEC-104 frames (BC-2.19.006).
///
/// Returns `true` iff:
/// - `data.len() >= 2` (can read start byte and LEN)
/// - `data[0] == 0x68` (IEC-104 start byte)
/// - `4 <= data[1] <= 253` (LEN in valid range)
///
/// Returns `false` for empty slice, one-byte slice, wrong start byte, or out-of-range LEN.
///
/// Called on port-2404-dispatched flows as a lightweight guard that compensates for
/// false-positive port classification, without polluting the `classify()` rule table with
/// a single-byte content signature (ADR-013 Decision 1).
///
/// ## Consistency guarantee (BC-2.19.006 invariant 2)
/// Any input for which this returns `true` AND `data.len() >= 6` will produce
/// `Some(ApciHeader)` from `parse_apci_header`.
///
/// ## Purity
/// Pure-core free function: no I/O, no side effects. VP-047 cargo-fuzz target.
pub fn is_valid_iec104_frame(data: &[u8]) -> bool {
    // BC-2.19.006: need at least 2 bytes to read start byte and LEN.
    data.len() >= 2 && data[0] == 0x68 && data[1] >= 4 && data[1] <= 253
}

// ---------------------------------------------------------------------------
// VP-044 Kani proof harness skeleton (ADR-013 Decision 8; STORY-167)
//
// Full Kani proof run targeting all five properties is STORY-174.
// parse_apci_header is fully implemented (BC-2.19.001-005). This #[cfg(kani)]
// harness asserts VP-044 Property A (no panic on any bounded symbolic input),
// Property B (returned total frame LEN+2 in [6,255]), and Property C (LEN in
// [4,253]) per ADR-013 Decision 8. STORY-174 wires the actual `cargo kani`
// execution into CI (this skeleton establishes the harness seam).
// ---------------------------------------------------------------------------
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // VP-044: parse_apci_header arithmetic safety.
    //
    // SCOPE: this harness covers only parse_apci_header.
    // parse_asdu field extraction, N(S)/N(R) counter tracking, and on_data-loop
    // no-panic are covered by VP-047 (cargo-fuzz), not this harness.
    // classify_frame_format totality over all 256 CF1 values is covered by
    // VP-046 (proptest), not Kani.
    //
    // Properties proved in STORY-174 (full run):
    //   A — no panic for any symbolic input (implicit: returns without panicking)
    //   B — total frame length `h.len as usize + 2` is in [6, 255]
    //   C — `h.len` is in [4, 253]
    #[kani::proof]
    fn verify_parse_apci_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 260); // BOUND=260 per ADR-013 Decision 8 / BC-2.19.001
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        // Must not panic for any input (Property A):
        let _ = parse_apci_header(&data);
        if let Some(h) = parse_apci_header(&data) {
            // Property B: total frame length is in [6, 255]
            let total = h.len as usize + 2;
            kani::assert(total >= 6, "APCI total frame >= 6");
            kani::assert(total <= 255, "APCI total frame <= 255");
            // Property C: len field in valid range
            kani::assert(h.len >= 4, "LEN >= 4");
            kani::assert(h.len <= 253, "LEN <= 253");
        }
    }
}
