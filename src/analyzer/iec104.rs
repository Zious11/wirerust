//! IEC 60870-5-104 (IEC-104) pure-core APCI header parser and frame-validity predicates.
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
//! - `is_valid_iec104_frame` — standalone pure predicate (BC-2.19.006); VP-047
//!   cargo-fuzz seam. Its equivalent validation is performed inline in the `on_data`
//!   frame-walk loop (start-byte check + LEN-range check). Not wired as a dispatch
//!   gate — port-based classification per ADR-013 Decision 1.
//! - `Iec104ParseError` — error type skeleton (extended in STORY-168).
//! - `parse_asdu` — pure-core ASDU header extraction into broken-out `Asdu` fields
//!   (BC-2.19.015–018; STORY-169); VP-047 fuzz target (no-panic for any input).
//! - `detect_iec104_threats` — effectful TypeID dispatch; emits T1692.001/T0836/T0827/T0814
//!   per TypeID range; appends `[TEST]` on `cot_test` frames
//!   (BC-2.19.017/BC-2.19.019–022; STORY-170).
//! - VP-044 Kani harness skeleton under `#[cfg(kani)]` (full proof run: STORY-174).
//!
//! ## Behavioral contracts
//! - BC-2.19.001: `parse_apci_header` returns None for input shorter than 6 bytes.
//! - BC-2.19.002: `parse_apci_header` returns None for start byte ≠ 0x68.
//! - BC-2.19.003: `parse_apci_header` returns None for LEN < 4.
//! - BC-2.19.004: `parse_apci_header` returns None for LEN > 253.
//! - BC-2.19.005: `parse_apci_header` returns Some(ApciHeader) for valid input; CF1–CF4 verbatim.
//! - BC-2.19.006: `is_valid_iec104_frame` is a standalone pure predicate; its validation is mirrored inline in on_data's frame-walk (not wired as a dispatch gate — port-based classification per ADR-013 Decision 1).
//! - BC-2.19.017: COT T-bit (`cot_test`) drives `[TEST]` tagging in findings.
//! - BC-2.19.019: TypeIDs 45–47 → T1692.001 Possible; TypeIDs 48–51 → T1692.001 + T0836 Possible.
//! - BC-2.19.020: TypeID 105 (C_RP_NA_1) → T0827 Likely.
//! - BC-2.19.021: TypeIDs 100, 101, 103 → no finding (trace-logged only).
//! - BC-2.19.022: TypeID=0 or TypeID in [128, 255] → T0814 Possible.
//!
//! ## Architecture compliance (ADR-013 Decision 7 — licensing)
//! Forbidden dependencies (BANNED — licensing violation):
//! - `iec60870-5` crate (proprietary, "NOT FREE for commercial/production use")
//! - Wireshark IEC-104 dissector `packet-104.c` (GPLv2)
//! - lib60870 / MZ Automation (GPLv3)
//!
//! This module is an original Rust implementation derived from IEC 60870-5-104:2006 framing
//! diagrams only. Zero lines are borrowed from any external implementation.

use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::Direction;

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
// Carry buffer bound (ADR-013 Decision 2; BC-2.19.025)
// ---------------------------------------------------------------------------

/// Maximum bytes held in a directional carry buffer between `on_data` calls.
///
/// The maximum on-wire APCI frame is LEN=253 + 2 prefix bytes = 255 bytes
/// (IEC 60870-5-104 §5.1; ADR-013 Decision 2). A carry buffer exceeding this
/// bound is therefore impossible without malformed or adversarial input; carry
/// accumulation beyond 255 bytes triggers T0814 overflow detection (BC-2.19.025).
///
/// Used by `Iec104Analyzer::on_data` carry-overflow check (STORY-172).
pub const MAX_IEC104_CARRY_BYTES: usize = 255;

/// DoS bound on accumulated IEC-104 findings per analyzer instance.
///
/// Cap enforced at the `on_data` extend step: `local_findings` is truncated to the remaining
/// capacity before merging into `self.all_findings`; the discarded count is added to
/// `self.dropped_findings` (BC-2.19.028; IEC104-FINDINGS-CAP-001; STORY-173).
///
/// Mirrors `MAX_FINDINGS` in `Dnp3Analyzer` (BC-2.15.022) and `EnipAnalyzer` (BC-2.17.022).
/// Same value (10_000) and same silent-cap-drop pattern.
///
/// CALLER NOTE: `detect_iec104_threats` callers MUST enforce this cap externally at the
/// `on_data` extend step. `detect_iec104_threats` itself is unbounded — the caller is
/// responsible for truncation (BC-2.19.028 Invariant 6 / IEC104-FINDINGS-CAP-001).
pub const MAX_IEC104_FINDINGS: usize = 10_000;

// ---------------------------------------------------------------------------
// Per-flow state (STORY-168 — introduces session_started; STORY-171 wires N(S) fields)
// ---------------------------------------------------------------------------

/// Per-flow state for the IEC-104 passive analyzer (SS-19, ADR-013).
///
/// Nine fields per the SS-19 architecture shard. `session_started` and the carry
/// buffers are introduced and wired in STORY-168. `last_ns_c2s`/`last_ns_s2c` are declared
/// and wired in STORY-171 (N(S) desync detection, BC-2.19.024). The two malformed-LEN
/// per-direction dedup flags and the two carry-overflow per-direction dedup flags are
/// fully wired in STORY-172 (BC-2.19.026 invariant 5; BC-2.19.025 invariants 4–5).
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
/// - `malformed_len_reported_c2s`: one-shot dedup flag; set on first malformed-LEN in C→S
///   direction; prevents T0814 re-emission on subsequent occurrences (BC-2.19.026 invariant 5;
///   STORY-172).
/// - `malformed_len_reported_s2c`: same dedup flag for S→C direction (BC-2.19.026 invariant 5;
///   STORY-172).
#[derive(Debug, Default)]
pub struct Iec104FlowState {
    /// Directional carry buffer for client-to-server APCI stream reassembly.
    /// Max 255 bytes (`MAX_IEC104_CARRY_BYTES`; BC-2.19.025). Wired in STORY-172.
    pub carry_c2s: Vec<u8>,
    /// Directional carry buffer for server-to-client APCI stream reassembly.
    /// Max 255 bytes. Wired in STORY-172.
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
    /// One-shot dedup flag for malformed-LEN T0814 emission in C→S direction.
    /// Set on the first valid-0x68-start but out-of-range-LEN frame in C→S.
    /// Once set, subsequent malformed-LEN frames in C→S advance the cursor silently.
    /// Never reset within a flow lifetime (BC-2.19.026 invariant 5; STORY-172).
    pub malformed_len_reported_c2s: bool,
    /// One-shot dedup flag for malformed-LEN T0814 emission in S→C direction.
    /// Same semantics as `malformed_len_reported_c2s` but for S→C.
    /// The two flags are independent — C→S and S→C dedup states never cross
    /// (BC-2.19.026 invariant 5; STORY-172).
    pub malformed_len_reported_s2c: bool,
    /// One-shot dedup flag for carry-residual-overflow T0814 emission in C→S direction.
    /// Set on the first carry-overflow event in C→S; suppresses T0814 re-emission for that
    /// direction within the flow lifetime. SEPARATE from `malformed_len_reported_c2s` so
    /// that the two anomaly classes cannot suppress each other (BC-2.19.025 invariant 4;
    /// F-172-001; STORY-172). Wired in `on_data` carry-overflow check.
    pub carry_overflow_reported_c2s: bool,
    /// One-shot dedup flag for carry-residual-overflow T0814 emission in S→C direction.
    /// Same semantics as `carry_overflow_reported_c2s` but for S→C direction.
    /// (BC-2.19.025 invariant 4; F-172-001; STORY-172). Wired in `on_data` carry-overflow check.
    pub carry_overflow_reported_s2c: bool,
    /// Count of complete valid parsed APDUs seen on this flow (start-byte 0x68 + LEN in
    /// [4,253] + full frame available). Incremented once per successful `parse_apci_header`
    /// call in the `on_data` frame-walk loop. Does NOT count bad-start-byte advances,
    /// malformed-LEN stubs, or carry-stashed partial frames.
    /// Folded into `Iec104Analyzer::total_frames_closed` by `on_flow_close`;
    /// summed over open flows by `summarize()` for `packets_analyzed`.
    /// (BC-2.19.028 observability; STORY-173 LOW#2.)
    pub frame_count: u64,
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

/// Standalone pure predicate: checks whether a byte slice begins with a valid IEC-104
/// APCI start byte and a LEN byte in the conformant range (BC-2.19.006).
///
/// Returns `true` iff:
/// - `data.len() >= 2` (can read start byte and LEN)
/// - `data[0] == 0x68` (IEC-104 start byte)
/// - `4 <= data[1] <= 253` (LEN in valid range)
///
/// Returns `false` for empty slice, one-byte slice, wrong start byte, or out-of-range LEN.
///
/// ## Design note (SEC-001 / ADR-013 Decisions 1, 2, 3)
/// This function is a **unit-tested and VP-047-fuzz-covered pure predicate**. It is NOT
/// called in the production `on_data` frame-walk path and MUST NOT be wired there:
/// the equivalent validation is performed inline in `on_data` — start-byte check and
/// LEN-range check — as required by the walk-first residual-bound anti-evasion semantics
/// (BC-2.19.025/BC-2.19.026, F-172-001, ADR-013 Decision 2). Adding a delivery-level
/// 0x68 pre-gate using this function would re-open the Ptacek/Newsham evasion hole and
/// break cross-segment carry. Flows reach `on_data` via port-2404-based classification
/// (ADR-013 Decision 1), not content-level gating.
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
// ASDU data model (STORY-169 — BC-2.19.015–018; ADR-013 Decision 3)
// ---------------------------------------------------------------------------

/// Parsed ASDU (Application Service Data Unit) with broken-out DUI header fields,
/// extracted from an IEC-104 I-format frame body.
///
/// All nine fields are semantically broken out from the raw bytes — no packed `vsq: u8`
/// or `cot: u16` fields are present (ADR-013 Decision 3; STORY-169 §Forbidden Dependencies).
///
/// ## Field layout (per IEC 60870-5-104:2006 §8.6)
///
/// | Field            | Source bytes        | Expression                         | BC ref        |
/// |------------------|--------------------|------------------------------------|---------------|
/// | `type_id`        | `asdu_body[0]`      | verbatim                           | BC-2.19.016 §1 |
/// | `sq`             | `asdu_body[1]`      | `(byte & 0x80) != 0`               | BC-2.19.016 §2 |
/// | `count`          | `asdu_body[1]`      | `byte & 0x7F`                      | BC-2.19.016 §3 |
/// | `cot_cause`      | `asdu_body[2]`      | `byte & 0x3F`                      | BC-2.19.017 §1 |
/// | `cot_pn`         | `asdu_body[2]`      | `(byte & 0x40) != 0`               | BC-2.19.017 §2 |
/// | `cot_test`       | `asdu_body[2]`      | `(byte & 0x80) != 0`               | BC-2.19.017 §3 |
/// | `cot_originator` | `asdu_body[3]`      | verbatim (0 = no originator)       | BC-2.19.017 §4 |
/// | `casdu`          | `asdu_body[4..=5]`  | `u16::from_le_bytes([b4, b5])`     | BC-2.19.018 §1 |
/// | `first_ioa`      | `asdu_body[6..=8]`  | `Some(u32 LE + 0-pad)` or `None`   | BC-2.19.018 §2 |
///
/// ## `first_ioa` semantics (BC-2.19.018 postconditions 2–3)
/// `Some(24-bit LE zero-extended to u32)` when `count > 0` AND `asdu_body.len() >= 9`.
/// `None` when `count == 0` (no objects declared) or `asdu_body.len() < 9` (bytes unavailable).
///
/// ## Populated by
/// [`parse_asdu`], which enforces the 6-byte DUI minimum-length guard (BC-2.19.015).
///
/// ## Architecture compliance (ADR-013 Decisions 3, 7, 8)
/// - Pure-core data struct: no behaviour, no I/O, no finding emission.
/// - VP-047 cargo-fuzz target (no-panic for all extraction paths; ADR-013 Decision 8).
///   `parse_asdu` is NOT a VP-044 Kani target; it is covered by VP-047 only.
/// - Forbidden dependencies: `iec60870-5`, Wireshark, lib60870 (ADR-013 Decision 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asdu {
    /// Type Identification (TypeID): identifies the ASDU content type
    /// (e.g., M_SP_NA_1 = 1, C_SC_NA_1 = 45, C_IC_NA_1 = 100, C_RP_NA_1 = 105).
    /// Value range: 0–255; TypeID 0 is undefined per IEC 60870-5-104 (passed through).
    /// Source: `asdu_body[0]`. BC-2.19.016 postcondition 1.
    pub type_id: u8,
    /// SQ (Sequence qualifier) flag from VSQ byte (bit 7 of `asdu_body[1]`).
    /// `true` = contiguous sequence of IOs starting at `first_ioa`; `false` = each IO has its own address.
    /// BC-2.19.016 postcondition 2; AC-169-002.
    pub sq: bool,
    /// Count of information objects (bits 6:0 of `asdu_body[1]`, range 0–127).
    /// 0 = no information objects present (valid, unusual).
    /// BC-2.19.016 postcondition 3; AC-169-002.
    pub count: u8,
    /// COT cause code (bits 5:0 of `asdu_body[2]`, range 0–63).
    /// Common values: 3 = spontaneous, 6 = activation, 7 = activation confirmation.
    /// BC-2.19.017 postcondition 1; AC-169-003.
    pub cot_cause: u8,
    /// COT P/N flag (bit 6 of `asdu_body[2]`).
    /// `true` = negative confirmation; `false` = positive confirmation.
    /// BC-2.19.017 postcondition 2; AC-169-003.
    pub cot_pn: bool,
    /// COT T (test) flag (bit 7 of `asdu_body[2]`).
    /// `true` = test transmission; caller may suppress or tag findings `[TEST]` per
    /// BC-2.19.017 invariant 1.
    /// BC-2.19.017 postcondition 3; AC-169-003.
    pub cot_test: bool,
    /// COT originator address (`asdu_body[3]`). 0 = no originator defined.
    /// BC-2.19.017 postcondition 4; AC-169-003.
    pub cot_originator: u8,
    /// Common Address of ASDU (16-bit little-endian from `asdu_body[4..=5]`).
    /// Identifies the RTU/IED (0 = undefined per spec; extracted without rejection).
    /// BC-2.19.018 postcondition 1; AC-169-004.
    pub casdu: u16,
    /// First Information Object Address — 24-bit LE from `asdu_body[6..=8]`, zero-padded to u32.
    /// `Some(ioa)` when `count > 0` AND `asdu_body.len() >= 9`; `None` otherwise.
    /// Range when present: `[0, 16_777_215]` (max 24-bit value 0xFFFFFF).
    /// BC-2.19.018 postconditions 2–3; AC-169-005.
    pub first_ioa: Option<u32>,
}

// ---------------------------------------------------------------------------
// Pure-core ASDU parser (VP-047 fuzz target — ADR-013 Decision 8)
// ---------------------------------------------------------------------------

/// Parse the ASDU DUI header fields from an I-format ASDU body slice.
///
/// Returns `Some(Asdu)` iff `asdu_body.len() >= 6` — the 6-byte DUI minimum:
/// TypeID(1) + VSQ(1) + COT(2) + CASDU(2) = 6 bytes (BC-2.19.015 invariant 1; AC-169-001).
/// Returns `None` for any shorter input, without accessing any byte beyond the bounds
/// and without panicking.
///
/// ## Field extraction mapping (when `Some` is returned)
///
/// | Field            | Expression                                                  | BC           |
/// |------------------|-------------------------------------------------------------|--------------|
/// | `type_id`        | `asdu_body[0]`                                              | BC-2.19.016  |
/// | `sq`             | `(asdu_body[1] & 0x80) != 0`                                | BC-2.19.016  |
/// | `count`          | `asdu_body[1] & 0x7F`                                       | BC-2.19.016  |
/// | `cot_cause`      | `asdu_body[2] & 0x3F`                                       | BC-2.19.017  |
/// | `cot_pn`         | `(asdu_body[2] & 0x40) != 0`                                | BC-2.19.017  |
/// | `cot_test`       | `(asdu_body[2] & 0x80) != 0`                                | BC-2.19.017  |
/// | `cot_originator` | `asdu_body[3]`                                              | BC-2.19.017  |
/// | `casdu`          | `u16::from_le_bytes([asdu_body[4], asdu_body[5]])`          | BC-2.19.018  |
/// | `first_ioa`      | `Some(u32::from_le_bytes([b6, b7, b8, 0]))` when eligible   | BC-2.19.018  |
///
/// ## `first_ioa` eligibility (BC-2.19.018 postconditions 2–3; AC-169-005)
/// `Some(...)` only when `count > 0` AND `asdu_body.len() >= 9`; `None` in all other cases.
///
/// ## ASDU body offset
/// The caller must pass the ASDU body slice starting at CF1 offset 4 within the APCI data
/// (i.e., `&apci_data[4..]` where `apci_data.len() == header.len as usize`).
/// ASDU body length = `header.len - 4` (LEN covers CF1–CF4 + ASDU).
/// See STORY-169 §Previous Story Intelligence and ADR-013 §ASDU payload offset.
///
/// ## Minimum-length guard (BC-2.19.015; AC-169-001)
/// Guard is `< 6` (NOT `< 10`). The 6-byte DUI minimum covers TypeID + VSQ + COT(2) + CASDU(2).
/// IOA bytes (6–8) are conditional — only accessed when `count > 0 && len >= 9`.
///
/// ## Purity (AC-169-006 / BC-2.19.015 invariant 2)
/// Pure-core free function: no I/O, no finding emission, no mutation of any state.
/// The caller (STORY-170 effectful shell) emits T0814 on `None`.
///
/// ## VP-047 fuzz seam (ADR-013 Decision 8)
/// This is a VP-047 cargo-fuzz target (no-panic for any input). It is NOT a VP-044
/// Kani target — `parse_apci_header` is the Kani target; ASDU extraction is VP-047 only.
pub fn parse_asdu(asdu_body: &[u8]) -> Option<Asdu> {
    // BC-2.19.015: minimum-length guard — DUI requires at least 6 bytes:
    // TypeID(1) + VSQ(1) + COT(2) + CASDU(2) = 6. Return None without accessing any byte.
    // The caller (STORY-170 effectful shell) emits T0814 on None (AC-169-001).
    if asdu_body.len() < 6 {
        return None;
    }

    // BC-2.19.016: TypeID verbatim from byte 0; VSQ broken out from byte 1 (AC-169-002).
    let type_id = asdu_body[0];
    let sq = (asdu_body[1] & 0x80) != 0;
    let count = asdu_body[1] & 0x7F;

    // BC-2.19.017: COT broken out from bytes 2–3 (AC-169-003).
    let cot_cause = asdu_body[2] & 0x3F;
    let cot_pn = (asdu_body[2] & 0x40) != 0;
    let cot_test = (asdu_body[2] & 0x80) != 0;
    let cot_originator = asdu_body[3];

    // BC-2.19.018 postcondition 1: CASDU as 16-bit little-endian from bytes 4–5 (AC-169-004).
    let casdu = u16::from_le_bytes([asdu_body[4], asdu_body[5]]);

    // BC-2.19.018 postconditions 2–3: first_ioa is Some(24-bit LE zero-extended to u32) only
    // when count > 0 AND asdu_body.len() >= 9; None otherwise (AC-169-005).
    let first_ioa = if count > 0 && asdu_body.len() >= 9 {
        Some(u32::from_le_bytes([
            asdu_body[6],
            asdu_body[7],
            asdu_body[8],
            0,
        ]))
    } else {
        None
    };

    Some(Asdu {
        type_id,
        sq,
        count,
        cot_cause,
        cot_pn,
        cot_test,
        cot_originator,
        casdu,
        first_ioa,
    })
}

// ---------------------------------------------------------------------------
// ASDU threat detection (STORY-170 — BC-2.19.017/019–022; ADR-013 Decision 8)
// ---------------------------------------------------------------------------

/// Classify an IEC-104 ASDU by TypeID and push security findings into `findings`.
///
/// Effectful free function — reads `asdu.type_id` and `asdu.cot_test`; appends zero or
/// more [`Finding`]s to `findings`. Called by the dispatcher after [`parse_asdu`] returns
/// `Some(asdu)` on an I-format frame (ADR-013 Decision 8).
///
/// ## TypeID dispatch (AC-170-006 — exhaustive, no fallthrough)
///
/// | Range / value      | Technique(s)            | Verdict  | BC ref      |
/// |--------------------|-------------------------|----------|-------------|
/// | 45–47 (C_SC/DC/RC) | T1692.001               | Possible | BC-2.19.019 |
/// | 48–51 (C_SE/C_BO)  | T1692.001 + T0836       | Possible | BC-2.19.019 |
/// | 105 (C_RP_NA_1)    | T0827 Loss of Control   | Likely   | BC-2.19.020 |
/// | 100, 101, 103      | none (trace-logged)     | —        | BC-2.19.021 |
/// | 0 or 128–255       | T0814 DoS anomaly       | Possible | BC-2.19.022 |
/// | 1–127 (unhandled)  | none (silently logged)  | —        | BC-2.19.022 |
///
/// When `asdu.cot_test == true`, ` [TEST]` is appended to every emitted finding's
/// `summary` field (BC-2.19.017 invariant 1; AC-170-007).
///
/// ## Purity boundary (ADR-013 Decision 8)
/// [`parse_asdu`] is pure-core; `detect_iec104_threats` is effectful-shell. Finding
/// emission must NOT occur inside `parse_asdu`.
///
/// ## VP-047 seam
/// No-panic for all TypeID values is a VP-047 cargo-fuzz property (`fuzz_iec104_parser`).
/// This function is NOT a VP-044 Kani target.
///
/// ## Findings cap
/// This function is UNBOUNDED — it appends findings without checking `MAX_IEC104_FINDINGS`.
/// The caller (`Iec104Analyzer::on_data`) MUST enforce the cap at the extend step by
/// truncating `local_findings` before merging into `self.all_findings`
/// (BC-2.19.028 Invariant 6 / IEC104-FINDINGS-CAP-001).
pub fn detect_iec104_threats(asdu: &Asdu, findings: &mut Vec<Finding>) {
    use crate::findings::{Confidence, ThreatCategory, Verdict};

    let type_id = asdu.type_id;
    // Record where findings start so we can apply [TEST] tagging only to new entries.
    let start_idx = findings.len();

    match type_id {
        // TypeIDs 45–47 (C_SC_NA_1, C_DC_NA_1, C_RC_NA_1): switching commands.
        // Emit T1692.001 "Unauthorized Message: Command Message" — Possible.
        // No T0836 emitted: switching commands are binary control, not parameter writes
        // (BC-2.19.019 postcondition 1; invariant 2; AC-170-001).
        45..=47 => {
            // BC-2.19.019 postcondition 3 (F-170-001): include CASDU and first_ioa as
            // target-address context so analysts can identify which RTU/IED and IO address
            // was targeted by the control command (BC-2.19.019 PC3; BC-2.19.020 PC2).
            let mut evidence = vec![
                format!("TypeID={type_id} is a switching control command (C_SC/C_DC/C_RC)"),
                format!("CASDU={}", asdu.casdu),
            ];
            if let Some(ioa) = asdu.first_ioa {
                evidence.push(format!("first_ioa={ioa}"));
            }
            findings.push(Finding {
                category: ThreatCategory::Impact,
                verdict: Verdict::Possible,
                confidence: Confidence::Medium,
                summary: format!(
                    "IEC-104 control command TypeID={type_id} (C_SC/C_DC/C_RC): \
                     switching control command observed on passive monitor \
                     (T1692.001 unauthorized command message; BC-2.19.019)"
                ),
                evidence,
                mitre_techniques: vec!["T1692.001".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }

        // TypeIDs 48–51 (C_SE_NA_1, C_SE_NB_1, C_SE_NC_1, C_BO_NA_1):
        // set-point and bitstring write commands.
        // Emit T1692.001 Possible (command-message indicator for all control TypeIDs)
        // AND T0836 Possible (parameter/value modification unique to set-point/bitstring).
        // (BC-2.19.019 postconditions 1–2; AC-170-001).
        48..=51 => {
            // BC-2.19.019 postcondition 3 (F-170-001): CASDU/first_ioa target-address context
            // for both co-emitted findings (T1692.001 + T0836).
            let mut ev1 = vec![
                format!(
                    "TypeID={type_id} is a set-point/bitstring write command (C_SE_NA/NB/NC or C_BO)"
                ),
                format!("CASDU={}", asdu.casdu),
            ];
            if let Some(ioa) = asdu.first_ioa {
                ev1.push(format!("first_ioa={ioa}"));
            }
            let mut ev2 = vec![
                format!("TypeID={type_id} is a set-point/bitstring write; parameter modification"),
                format!("CASDU={}", asdu.casdu),
            ];
            if let Some(ioa) = asdu.first_ioa {
                ev2.push(format!("first_ioa={ioa}"));
            }
            findings.push(Finding {
                category: ThreatCategory::Impact,
                verdict: Verdict::Possible,
                confidence: Confidence::Medium,
                summary: format!(
                    "IEC-104 control command TypeID={type_id} (C_SE/C_BO): \
                     set-point or bitstring write command observed on passive monitor \
                     (T1692.001 unauthorized command message; BC-2.19.019)"
                ),
                evidence: ev1,
                mitre_techniques: vec!["T1692.001".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
            findings.push(Finding {
                category: ThreatCategory::Impact,
                verdict: Verdict::Possible,
                confidence: Confidence::Medium,
                summary: format!(
                    "IEC-104 parameter modification TypeID={type_id} (C_SE/C_BO): \
                     set-point or bitstring write modifies ICS control parameters \
                     (T0836 modify parameter; BC-2.19.019 postcondition 2)"
                ),
                evidence: ev2,
                mitre_techniques: vec!["T0836".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }

        // TypeID 105 (C_RP_NA_1 — Reset Process Command).
        // Emit T0827 "Loss of Control" — Likely (NOT Possible; BC-2.19.020 v1.1 correction).
        // Only T0827 is emitted — not T1692.001 (reset is session management, not parameter change).
        // (BC-2.19.020 postcondition 1; invariant 1; AC-170-002).
        105 => {
            // BC-2.19.020 postcondition 2 (F-170-001): CASDU/first_ioa target-address context.
            let mut evidence = vec![
                "TypeID=105 (C_RP_NA_1: Reset Process Command)".to_string(),
                format!("CASDU={}", asdu.casdu),
            ];
            if let Some(ioa) = asdu.first_ioa {
                evidence.push(format!("first_ioa={ioa}"));
            }
            findings.push(Finding {
                category: ThreatCategory::Impact,
                verdict: Verdict::Likely,
                confidence: Confidence::Medium,
                summary: "IEC-104 Reset Process command TypeID=105 (C_RP_NA_1): \
                     potential unauthorized RTU/IED process reset observed; \
                     adversarial use causes equipment to revert to default state \
                     (T0827 loss of control; BC-2.19.020)"
                    .to_string(),
                evidence,
                mitre_techniques: vec!["T0827".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }

        // TypeIDs 100 (C_IC_NA_1), 101 (C_CI_NA_1), 103 (C_CS_NA_1):
        // General Interrogation, Counter Interrogation, Clock Synchronization.
        // Benign administrative commands — no finding emitted (BC-2.19.021 postcondition 1;
        // invariant 1; AC-170-003). Logged at trace level by the caller's dispatcher.
        100 | 101 | 103 => {
            // No finding: interrogation and clock-sync are benign operational messages
            // (BC-2.19.021 postcondition 1). This arm is explicit to prevent silent
            // fallthrough into the reserved-TypeID detection path for adjacent values.
        }

        // TypeID=0 (undefined per IEC 60870-5-104) or TypeIDs 128–255 (private-use/reserved):
        // Emit T0814 "Denial of Service" — Possible.
        // TypeIDs in [1, 127] that are not in the explicit detection sets above are
        // silently logged (no finding) — they are defined-but-unhandled TypeIDs per
        // IEC 60870-5-104 and should not produce anomaly findings (BC-2.19.022 invariant 1;
        // AC-170-004).
        0 | 128..=255 => {
            // BC-2.19.022 postcondition 1 (F-170-001): CASDU/first_ioa target-address context.
            let mut evidence = vec![
                format!(
                    "TypeID={type_id} is {} per IEC 60870-5-104",
                    if type_id == 0 {
                        "undefined (TypeID 0 has no assigned meaning)"
                    } else {
                        "in the private-use/reserved range [128, 255]"
                    }
                ),
                format!("CASDU={}", asdu.casdu),
            ];
            if let Some(ioa) = asdu.first_ioa {
                evidence.push(format!("first_ioa={ioa}"));
            }
            findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Possible,
                confidence: Confidence::Medium,
                summary: format!(
                    "IEC-104 reserved or invalid TypeID={type_id}: \
                     undefined or private-use TypeID indicates implementation error \
                     or adversarial protocol probe \
                     (T0814 denial of service; BC-2.19.022)"
                ),
                evidence,
                mitre_techniques: vec!["T0814".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }

        // Defined-but-unhandled TypeIDs in [1, 127] not covered by the arms above:
        // TypeIDs 1–44 (monitoring direction), 52–99, 102 (C_RD_NA_1), 104, 106–127.
        // No finding emitted — silently logged (BC-2.19.022 invariant 1; AC-170-005).
        _ => {
            // Silently logged: defined TypeID not in any detection set.
            // No finding emitted (BC-2.19.022 invariant 1).
        }
    }

    // BC-2.19.017 invariant 1 (AC-170-007): when cot_test=true, append " [TEST]" to
    // every finding emitted by this call. Applied to the slice of newly-pushed findings
    // only — existing entries in `findings` from prior calls are not modified.
    if asdu.cot_test {
        for f in &mut findings[start_idx..] {
            f.summary.push_str(" [TEST]");
        }
    }
}

// ---------------------------------------------------------------------------
// N(S)/N(R) sequence number extraction (STORY-171 — BC-2.19.023; ADR-013 Decision 6)
// ---------------------------------------------------------------------------

/// Extract N(S) 15-bit send sequence number from I-format CF1/CF2 control field bytes.
///
/// Pure-core free function — no state mutation, no I/O. Called before any state
/// mutation on I-format frames (BC-2.19.023 postcondition 1; ADR-013 Decision 6).
///
/// ## Extraction formula (BC-2.19.023 postcondition 1)
/// `ns = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)` — result in [0, 32767].
///
/// ## VP-047 seam
/// No-panic for all (cf1, cf2) u8 inputs is a VP-047 cargo-fuzz property
/// (`fuzz_iec104_parser`; BC-2.19.023 invariant 2).
pub fn extract_ns(cf1: u8, cf2: u8) -> u16 {
    // BC-2.19.023 postcondition 1: ns = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7).
    // Result is always in [0, 32767] — no additional masking needed (BC-2.19.023 invariant 1).
    ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)
}

/// Extract N(R) 15-bit receive sequence number from I/S-format CF3/CF4 control field bytes.
///
/// Pure-core free function — no state mutation, no I/O. N(R) is computed but NOT
/// stored in `Iec104FlowState` (BC-2.19.023 postcondition 4; ADR-013 Decision 6).
///
/// ## Extraction formula (BC-2.19.023 postcondition 2)
/// `nr = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)` — result in [0, 32767].
///
/// ## VP-047 seam
/// No-panic for all (cf3, cf4) u8 inputs is a VP-047 cargo-fuzz property
/// (`fuzz_iec104_parser`; BC-2.19.023 invariant 2).
pub fn extract_nr(cf3: u8, cf4: u8) -> u16 {
    // BC-2.19.023 postcondition 2: nr = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7).
    // Same formula as extract_ns but applied to CF3/CF4 (BC-2.19.023 postconditions 1–2).
    // N(R) is transient — caller holds it; NOT stored in Iec104FlowState (postcondition 4).
    ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)
}

// ---------------------------------------------------------------------------
// N(S) gap detection + Option<u16> first-frame guard (STORY-171 — BC-2.19.024)
// ---------------------------------------------------------------------------

/// Track per-direction N(S) and emit T1692.001 Possible on gap > k=12.
///
/// Effectful free function — mutates `Iec104FlowState::last_ns_c2s` or
/// `last_ns_s2c` (selected by `direction`) and may return `Some(Finding)`.
/// Called by the dispatcher on each I-format frame after `extract_ns`
/// (ADR-013 Decision 6; BC-2.19.024).
///
/// ## Three-path dispatch (BC-2.19.024)
///
/// | State (`last_ns_dir`) | Gap      | Action                                           |
/// |-----------------------|----------|--------------------------------------------------|
/// | `None` (first frame)  | N/A      | Set `Some(current_ns)`; return `None` (Path A)   |
/// | `Some(prev)`, gap ≤ 12 | ≤ 12   | Update `Some(current_ns)`; return `None` (Path B) |
/// | `Some(prev)`, gap > 12 | > 12   | Update + return `Some(T1692.001 Possible)` (Path C) |
///
/// ## 15-bit modular arithmetic (BC-2.19.024 invariant 1)
/// Gap = `current_ns.wrapping_sub(prev) & 0x7FFF`. The `& 0x7FFF` mask is
/// mandatory — `wrapping_sub` wraps at 2^16, not 2^15; plain subtraction is WRONG.
///
/// ## Direction field selection (BC-2.19.023 postcondition 3; AC-171-007)
/// `Direction::ClientToServer` → `state.last_ns_c2s`
/// `Direction::ServerToClient` → `state.last_ns_s2c`
/// The two fields are mutated independently — no cross-direction mixing.
///
/// ## VP-045 proptest seam
/// VP-045 verifies directional isolation: last_ns_c2s and last_ns_s2c updated
/// independently (AC-171-007; STORY-172 anchors proptest; full run STORY-174).
pub fn track_ns_desync(
    state: &mut Iec104FlowState,
    current_ns: u16,
    direction: Direction,
) -> Option<Finding> {
    use crate::findings::{Confidence, ThreatCategory, Verdict};

    // Select the directional field by direction parameter (BC-2.19.023 postcondition 3;
    // AC-171-007). The two fields are mutated independently — no cross-direction mixing.
    let last_ns_dir = match direction {
        Direction::ClientToServer => &mut state.last_ns_c2s,
        Direction::ServerToClient => &mut state.last_ns_s2c,
    };

    match *last_ns_dir {
        // Path A — first I-frame (state None): set baseline; NO finding unconditionally.
        // Handles mid-capture starts where the first observed N(S) is arbitrary (not 0);
        // any gap relative to an assumed zero baseline would be a false positive
        // (BC-2.19.024 postconditions A1–A2; invariant 3; ADR-013 Decision 6).
        None => {
            *last_ns_dir = Some(current_ns);
            None
        }

        // Path B/C — subsequent I-frame (state Some(prev)): compute 15-bit modular gap.
        Some(prev) => {
            // BC-2.19.024 invariant 1: gap MUST use wrapping_sub + & 0x7FFF mask.
            // wrapping_sub wraps at 2^16 (65536), not 2^15 (32768); the mask collapses
            // the result to the 15-bit N(S) range. Plain subtraction is WRONG here.
            let gap = current_ns.wrapping_sub(prev) & 0x7FFF;
            // Update state before returning finding (BC-2.19.024 postcondition C3:
            // state is always updated even when a finding is emitted).
            *last_ns_dir = Some(current_ns);

            if gap > 12 {
                // Path C: gap > k=12 → T1692.001 "Unauthorized Message: Command Message"
                // with Verdict::Possible (BC-2.19.024 postcondition C1; ADR-013 Decision 6).
                // source_ip and timestamp left None — enriched in STORY-173.
                Some(Finding {
                    category: ThreatCategory::Impact,
                    verdict: Verdict::Possible,
                    confidence: Confidence::Medium,
                    summary: format!(
                        "IEC-104 N(S) sequence desync: N(S)={current_ns} prev={prev} \
                         gap={gap} > k=12 — sequence-number desynchronization detected; \
                         possible replay injection or adversarial manipulation \
                         (T1692.001 unauthorized command message; BC-2.19.024)"
                    ),
                    evidence: vec![
                        format!(
                            "N(S) gap={gap} exceeds k=12 window \
                             (current_ns={current_ns}, prev_ns={prev})"
                        ),
                        format!("direction={direction:?}"),
                    ],
                    mitre_techniques: vec!["T1692.001".to_string()],
                    source_ip: None,
                    timestamp: None,
                    direction: None,
                })
            } else {
                // Path B: gap ≤ k=12 — state updated, no finding (BC-2.19.024 postcondition B).
                None
            }
        }
    }
}

// ---------------------------------------------------------------------------
// IEC-104 analyzer struct + flow lifecycle (STORY-172 — BC-2.19.025/026/027)
// ---------------------------------------------------------------------------

/// IEC-104 TCP stream analyzer.
///
/// Holds per-flow [`Iec104FlowState`] keyed by [`FlowKey`]. The pure-core parse and
/// classification free functions (`parse_apci_header`, `parse_asdu`, `classify_frame_format`,
/// etc.) are NOT methods — they remain free `fn`s for VP-044 Kani amenability
/// (ADR-013 Decision 8).
///
/// ## Subsystem
/// SS-19; ADR-013 Decision 8 (effectful-shell / pure-core separation).
///
/// ## BC anchors
/// - BC-2.19.025: directional carry buffers bounded at `MAX_IEC104_CARRY_BYTES` = 255
/// - BC-2.19.026: frame-walk loop processes multiple APDUs per `on_data` call
/// - BC-2.19.027: `on_flow_close` removes `Iec104FlowState` and discards carry bytes
pub struct Iec104Analyzer {
    /// Per-flow IEC-104 analyzer state, keyed by canonicalized TCP 4-tuple.
    pub flows: HashMap<FlowKey, Iec104FlowState>,
    /// Accumulated findings from `on_data` calls across all flows.
    /// Tests inspect this field to assert T0814 emission counts and attributes.
    /// Mirrors the `Dnp3Analyzer::all_findings` / `EnipAnalyzer::all_findings` pattern.
    pub all_findings: Vec<Finding>,
    /// Count of findings silently dropped because `all_findings` reached `MAX_IEC104_FINDINGS`.
    ///
    /// Initialized to 0. Incremented at the `on_data` extend step whenever
    /// `local_findings` is truncated to the remaining capacity
    /// (BC-2.19.028 PC-3/PC-4; IEC104-FINDINGS-CAP-001; STORY-173).
    /// Surfaced in `summarize()` as `detail["dropped_findings"]`.
    pub dropped_findings: u64,
    /// Count of flows removed via `on_flow_close` (i.e., closed flows).
    ///
    /// Added to `self.flows.len()` by `summarize()` to compute total `flows_analyzed`
    /// (closed + still-open). Mirrors `EnipAnalyzer::flows_analyzed` /
    /// `Dnp3Analyzer::closed_flows_count` pattern. (BC-2.19.028 observability; STORY-173 LOW#1.)
    pub flows_analyzed: u64,
    /// Aggregate `frame_count` from flows removed via `on_flow_close`.
    ///
    /// Added to the per-open-flow sum by `summarize()` for `packets_analyzed`.
    /// Mirrors `Dnp3Analyzer::total_frames_closed` pattern.
    /// (BC-2.19.028 observability; STORY-173 LOW#2.)
    pub total_frames_closed: u64,
}

impl Default for Iec104Analyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl Iec104Analyzer {
    /// Construct a new `Iec104Analyzer` with an empty flow state map and findings list.
    pub fn new() -> Self {
        Self {
            flows: HashMap::new(),
            all_findings: Vec::new(),
            dropped_findings: 0,
            flows_analyzed: 0,
            total_frames_closed: 0,
        }
    }

    /// Process a chunk of reassembled TCP stream data for the given flow.
    ///
    /// Effectful shell per ADR-013 Decision 8. VP-047 cargo-fuzz target (`fuzz_iec104_parser`).
    ///
    /// WALK-FIRST RESIDUAL-BOUND semantics (BC-2.19.025 v1.3, F-172-001, ADR-013 Decision 2):
    /// No aggregate pre-check on `carry.len() + delivery.len()`. Frame extraction always
    /// completes before any carry-bound reaction. The only pre-walk check is on the directional
    /// carry alone: if carry.len() > MAX_IEC104_CARRY_BYTES (adversarial state injection;
    /// unreachable from conformant traffic), carry is cleared and ONE T0814 emitted per
    /// direction (dedup flags carry_overflow_reported_{c2s/s2c}; BC-2.19.025 invariants 4–5).
    /// The delivery is always walked regardless (anti-evasion invariant 2).
    ///
    /// Frame-walk loop (BC-2.19.026 / ADR-013 Decision 3):
    /// - Drain directional carry into working buffer; extend with delivery.
    /// - Loop: bad-start-byte → advance 1, no finding; malformed-LEN → advance 2,
    ///   emit T0814 EMIT-WITH-DEDUP on first occurrence per direction (BC-2.19.026
    ///   invariant 5); valid frame → parse + dispatch + advance LEN+2; insufficient data
    ///   → stash remaining to carry and return.
    ///
    /// BC-2.19.025 / BC-2.19.026 / STORY-172.
    ///
    /// ## Findings cap (BC-2.19.028; IEC104-FINDINGS-CAP-001; STORY-173)
    /// At the extend step, `local_findings` is truncated to the remaining capacity
    /// (`MAX_IEC104_FINDINGS - self.all_findings.len()` slots) before merging; discarded count
    /// is added to `self.dropped_findings`. Per-flow state continues updating regardless.
    pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction) {
        use crate::findings::{Confidence, ThreatCategory, Verdict};
        let _ = ts;

        // Collect frame-walk findings locally to avoid borrow conflicts between
        // self.flows (via state) and self.all_findings during the loop.
        let mut local_findings: Vec<Finding> = Vec::new();

        {
            let state = self.flows.entry(flow_key).or_default();

            // BC-2.19.025 v1.3 carry-overflow check (F-172-001, WALK-FIRST-RESIDUAL-BOUND):
            // Check the directional carry alone — NOT the aggregate carry + delivery.
            // A carry exceeding MAX_IEC104_CARRY_BYTES is adversarial or non-conformant state;
            // conformant on_data calls always stash ≤ 254 bytes as residual (a 255-byte prefix
            // is a complete max-size frame and is walked off, not stashed). Clear the carry and
            // emit ONE T0814 on the first overflow per direction via the carry-overflow dedup
            // flag (distinct from malformed_len_reported_* per BC-2.19.025 invariant 4).
            // The delivery is always walked regardless (walk-first anti-evasion clause;
            // BC-2.19.025 invariant 2; ADR-013 Decision 2).
            {
                let (carry, reported) = if direction == Direction::ClientToServer {
                    (&mut state.carry_c2s, &mut state.carry_overflow_reported_c2s)
                } else {
                    (&mut state.carry_s2c, &mut state.carry_overflow_reported_s2c)
                };
                if carry.len() > MAX_IEC104_CARRY_BYTES {
                    carry.clear();
                    if !*reported {
                        *reported = true;
                        local_findings.push(Finding {
                            category: ThreatCategory::Anomaly,
                            verdict: Verdict::Possible,
                            confidence: Confidence::Medium,
                            summary: format!(
                                "IEC-104 directional carry residual overflow: carry buffer \
                                 exceeded MAX_IEC104_CARRY_BYTES={MAX_IEC104_CARRY_BYTES} — \
                                 adversarial or non-conformant byte sequence; carry cleared \
                                 and analyzer resyncs on next delivery \
                                 (T0814; BC-2.19.025 v1.3 F-172-001)"
                            ),
                            evidence: vec![format!(
                                "carry overflow (>{}); direction={:?}; carry cleared",
                                MAX_IEC104_CARRY_BYTES, direction
                            )],
                            mitre_techniques: vec!["T0814".to_string()],
                            source_ip: None,
                            timestamp: None,
                            direction: None,
                        });
                    }
                    // Carry is now cleared; walk proceeds on delivery only (walk-first preserved).
                }
            }

            // Build working buffer: drain directional carry first, then append delivery.
            // BC-2.19.025 invariant 1: carries are never mixed across directions.
            let mut buf: Vec<u8> = if direction == Direction::ClientToServer {
                state.carry_c2s.drain(..).collect()
            } else {
                state.carry_s2c.drain(..).collect()
            };
            buf.extend_from_slice(data);

            // Frame-walk loop (BC-2.19.026 postconditions 1–4; ADR-013 Decision 3).
            let mut pos = 0;
            while pos < buf.len() {
                // Bad start byte: advance 1; no finding; carry NOT cleared (BC-2.19.026
                // postcondition 4 bad-start-byte arm; ADR-013 Decision 3).
                if buf[pos] != 0x68 {
                    pos += 1;
                    continue;
                }

                // Valid 0x68 start byte found. Need at least 2 bytes to read LEN.
                if buf.len() - pos < 2 {
                    // Only the start byte remains — insufficient to determine LEN.
                    // Stash as carry and exit.
                    let remaining = &buf[pos..];
                    if direction == Direction::ClientToServer {
                        state.carry_c2s.extend_from_slice(remaining);
                    } else {
                        state.carry_s2c.extend_from_slice(remaining);
                    }
                    break;
                }

                let len = buf[pos + 1];

                // Malformed LEN: 0x68 start byte but LEN outside [4, 253].
                // Advance 2 bytes (skip APCI stub). EMIT-WITH-DEDUP: emit ONE T0814
                // on the first occurrence per direction; silent resync thereafter.
                // (BC-2.19.026 invariant 5; ADR-013 Decision 3; SR-172-03.)
                if !(4u8..=253).contains(&len) {
                    let reported = if direction == Direction::ClientToServer {
                        &mut state.malformed_len_reported_c2s
                    } else {
                        &mut state.malformed_len_reported_s2c
                    };
                    if !*reported {
                        *reported = true;
                        local_findings.push(Finding {
                            category: ThreatCategory::Anomaly,
                            verdict: Verdict::Possible,
                            confidence: Confidence::Medium,
                            summary: format!(
                                "IEC-104 malformed LEN byte: 0x68 start byte followed by \
                                 LEN={len:#04x} ({len}) outside valid range [4, 253] — \
                                 protocol anomaly or adversarial framing attack \
                                 (T0814; BC-2.19.026 invariant 5)"
                            ),
                            evidence: vec![format!(
                                "LEN={len} not in [4, 253]; start byte=0x68 at buffer offset {pos}"
                            )],
                            mitre_techniques: vec!["T0814".to_string()],
                            source_ip: None,
                            timestamp: None,
                            direction: None,
                        });
                    }
                    pos += 2;
                    continue;
                }

                // Valid LEN in [4, 253]: check whether the complete frame is available.
                let frame_len = len as usize + 2;
                if buf.len() - pos < frame_len {
                    // Insufficient data: stash remaining bytes into directional carry.
                    // Residual is always ≤ frame_len − 1 ≤ 254 bytes for conformant traffic.
                    let remaining = &buf[pos..];
                    if direction == Direction::ClientToServer {
                        state.carry_c2s.extend_from_slice(remaining);
                    } else {
                        state.carry_s2c.extend_from_slice(remaining);
                    }
                    break;
                }

                // Complete valid frame: parse APCI header and dispatch to per-format handlers.
                let frame = &buf[pos..pos + frame_len];
                if let Some(header) = parse_apci_header(frame) {
                    // Count every successfully parsed APDU (BC-2.19.028 observability;
                    // STORY-173 LOW#2). Incremented here — after parse_apci_header succeeds
                    // and before format dispatch — so bad-start-byte skips and malformed-LEN
                    // stubs are never counted.
                    state.frame_count = state.frame_count.saturating_add(1);
                    match classify_frame_format(header.cf1) {
                        FrameFormat::UFormat => {
                            if let Some(f) = process_u_frame(state, header.cf1) {
                                local_findings.push(f);
                            }
                        }
                        FrameFormat::IFormat => {
                            // ASDU body starts at byte 6 of the frame (after the 6-byte APCI
                            // header: start + LEN + CF1 + CF2 + CF3 + CF4).
                            let asdu_body = &frame[6..];
                            if let Some(asdu) = parse_asdu(asdu_body) {
                                detect_iec104_threats(&asdu, &mut local_findings);
                            }
                            let ns = extract_ns(header.cf1, header.cf2);
                            if let Some(f) = track_ns_desync(state, ns, direction) {
                                local_findings.push(f);
                            }
                        }
                        FrameFormat::SFormat => {
                            // S-format: supervisory-only, no ASDU, no finding emitted.
                        }
                    }
                }
                pos += frame_len;
            }
        }

        // BC-2.19.028 PC-2 / IEC104-FINDINGS-CAP-001: cap at MAX_IEC104_FINDINGS.
        let remaining_cap = MAX_IEC104_FINDINGS.saturating_sub(self.all_findings.len());
        if local_findings.len() > remaining_cap {
            self.dropped_findings = self
                .dropped_findings
                .saturating_add((local_findings.len() - remaining_cap) as u64);
            local_findings.truncate(remaining_cap);
        }
        self.all_findings.extend(local_findings);
    }

    /// Produce the IEC-104 analyzer summary.
    ///
    /// Aggregates analyzer-level counters into an `AnalysisSummary`. The detail map
    /// MUST include key `"dropped_findings"` (BC-2.19.028 PC-5 / AC-173-007).
    ///
    /// `flows_analyzed` = closed flows (accumulated in `self.flows_analyzed` by
    /// `on_flow_close`) + still-open flows (`self.flows.len()`). Mirrors the
    /// ENIP `flows_analyzed` and DNP3 `closed_flows_count` patterns (STORY-173 LOW#1).
    ///
    /// `packets_analyzed` = complete valid parsed APDUs across all closed flows
    /// (`self.total_frames_closed`) plus the sum of `frame_count` over still-open flows.
    /// Mirrors the DNP3 `total_frames_closed` + open-flow sum pattern (STORY-173 LOW#2).
    ///
    /// Does NOT emit new findings (no side effects).
    ///
    /// Traces: BC-2.19.028 Postcondition 5; AC-173-007; STORY-173 F-173-001.
    pub fn summarize(&self) -> AnalysisSummary {
        use std::collections::BTreeMap;

        // Closed flows + still-open flows (STORY-173 LOW#1).
        let flows_analyzed = self.flows_analyzed.saturating_add(self.flows.len() as u64);

        // Closed-flow frame totals + still-open flow frame totals (STORY-173 LOW#2).
        let packets_analyzed = self
            .total_frames_closed
            .saturating_add(self.flows.values().map(|f| f.frame_count).sum::<u64>());

        let mut detail: BTreeMap<String, serde_json::Value> = BTreeMap::new();

        // BC-2.19.028 PC-5 / AC-173-007: MUST be present; value is monotonically
        // non-decreasing across on_data calls (zero when no cap event has fired).
        detail.insert(
            "dropped_findings".to_string(),
            serde_json::Value::Number(self.dropped_findings.into()),
        );
        // Observability: closed + open flow count (STORY-173 LOW#1).
        detail.insert(
            "flows_analyzed".to_string(),
            serde_json::Value::Number(flows_analyzed.into()),
        );
        // Observability: retained findings count (capped at MAX_IEC104_FINDINGS).
        detail.insert(
            "total_findings".to_string(),
            serde_json::Value::Number((self.all_findings.len() as u64).into()),
        );

        AnalysisSummary {
            analyzer_name: "IEC-104".to_string(),
            packets_analyzed,
            detail,
        }
    }

    /// Remove per-flow state for a closed flow, discarding carry bytes silently.
    ///
    /// Postconditions (BC-2.19.027):
    /// 1. `self.flows.remove(&flow_key)` removes `Iec104FlowState` for the flow.
    /// 2. `carry_c2s` and `carry_s2c` are dropped (memory freed) as part of state removal.
    /// 3. No finding is emitted for normal flow close.
    /// 4. Unknown `flow_key` is a no-op — no panic (BC-2.19.027 postcondition 4).
    ///
    /// On successful removal (postcondition 6 — STORY-173 LOW#1/LOW#2):
    /// - `self.flows_analyzed` is incremented by 1 (closed-flow count).
    /// - The flow's `frame_count` is folded into `self.total_frames_closed`.
    ///
    /// BC-2.19.027 / STORY-172 / STORY-173.
    pub fn on_flow_close(&mut self, flow_key: FlowKey) {
        if let Some(flow) = self.flows.remove(&flow_key) {
            self.flows_analyzed = self.flows_analyzed.saturating_add(1);
            self.total_frames_closed = self.total_frames_closed.saturating_add(flow.frame_count);
        }
    }
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
