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
