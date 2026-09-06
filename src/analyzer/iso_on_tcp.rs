//! ISO-on-TCP (TPKT/COTP) framing layer — pure-core header parsers.
//!
//! Subsystem SS-20, CAP-20 — `analyzer/iso_on_tcp.rs`.
//!
//! ## Architecture (ADR-014 Decision 1, Decision 9)
//!
//! This module is a **standalone, protocol-agnostic** framing layer, separate from the
//! S7comm PDU dissector (`src/analyzer/s7comm.rs`, SS-21, not yet created — STORY-186).
//! Per ADR-014 Decision 1's frozen interface, `iso_on_tcp.rs` exports **pure free
//! functions only**:
//!
//! - No `impl StreamAnalyzer` block of any kind — `S7commAnalyzer` (SS-21) is the sole
//!   consumer of this module's functions, and the frozen module-boundary contract is
//!   verified structurally starting in STORY-186.
//! - No per-flow state of its own — the TPKT/COTP directional carry buffers
//!   (`carry_c2s`/`carry_s2c`) live on `S7commFlowState` (SS-21), not here.
//! - No dependency on `dispatcher.rs`, mirroring `protocols.rs`'s documented
//!   pure-core-leaf discipline.
//!
//! All parse functions in this module are **pure-core free `fn`s** — no `self`, no I/O,
//! no global state mutation. This is a hard constraint for VP-048 Kani formal
//! verification amenability (ADR-014 Decision 9).
//!
//! ## Scope of this story (STORY-184)
//!
//! This story covers **only** the TPKT (RFC 1006) outer framing header:
//! `TpktHeader` and `parse_tpkt_header`. COTP (ISO 8073 / ITU-T X.224) parsing —
//! `CotpHeader`, `CotpTpduType`, `parse_cotp_header` — is explicitly out of scope here
//! and is delivered by STORY-185 (VP-049). See ADR-014 Decision 9's scope note.
//!
//! - `parse_tpkt_header` — 4-byte TPKT header parse; `None` on short/invalid input
//!   (BC-2.20.001–004); VP-048 Kani target.
//!
//! ## Behavioral contracts
//! - BC-2.20.001: `parse_tpkt_header` returns `None` for input shorter than 4 bytes.
//! - BC-2.20.002: `parse_tpkt_header` returns `None` for version byte != 0x03.
//! - BC-2.20.003: `parse_tpkt_header` returns `None` for decoded length field < 4
//!   (malformed, includes zero-length).
//! - BC-2.20.004: `parse_tpkt_header` returns `Some(TpktHeader)` for valid input
//!   (happy path); reserved byte (`data[1]`) is never validated; `length == 65535` is a
//!   legal accept.
//!
//! ## Architecture compliance (ADR-014 Decision 4 — licensing)
//! Forbidden dependencies (BANNED/AVOID — licensing violation or unclear provenance):
//! - `rusty-cotp`, `rusty-tpkt`, `tpkt`, `copt` crates (unclear/non-standard license)
//! - `s7`, `s7-comm`, `s7-client` crates (non-standard custom license grant)
//! - Wireshark, Snap7, or libnodave source of any kind (GPL/LGPL — banned)
//!
//! This module is an original Rust implementation derived directly from RFC 1006 §5
//! (a freely implementable open specification). Zero lines are borrowed from any
//! external implementation.

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Parsed TPKT (RFC 1006) header — the outer 4-byte framing layer present on every TCP
/// segment carrying ISO-on-TCP traffic (S7comm, IEC 61850 MMS, ICCP/TASE.2 on TCP/102).
///
/// The TPKT header occupies exactly 4 bytes on the wire (RFC 1006 §5):
/// - `version` (byte 0): always `0x03` for a valid TPKT packet.
/// - *(reserved, byte 1)*: not surfaced by this struct — never validated by
///   `parse_tpkt_header` (BC-2.20.004 invariant 1).
/// - `length` (bytes 2–3): big-endian `u16`, total TPKT packet length **including**
///   this 4-byte header. Valid range on the accept path: `[4, 65535]`.
///
/// Frozen per ADR-014 Decision 1 — exactly these two fields, no `reserved` field
/// surfaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TpktHeader {
    /// TPKT version byte; always `3` for a valid TPKT packet (RFC 1006 §5).
    pub version: u8,
    /// Total TPKT packet length in bytes, including this 4-byte header.
    /// Valid range on the accept path: `[4, 65535]`.
    pub length: u16,
}

// ---------------------------------------------------------------------------
// Parse functions
// ---------------------------------------------------------------------------

/// Parse a TPKT (RFC 1006) header from the start of `data`.
///
/// Pure-core free function (ADR-014 Decision 9) — no I/O, no global state mutation,
/// no side effects, deterministic. VP-048 Kani P0 target.
///
/// # Returns
///
/// - `None` if `data.len() < 4` (BC-2.20.001).
/// - `None` if `data[0] != 0x03` (BC-2.20.002); the length field is never decoded in
///   this case.
/// - `None` if the big-endian `u16` decoded from `data[2..4]` is `< 4` (BC-2.20.003).
/// - `Some(TpktHeader { version: 3, length })` otherwise, where `length` is exactly the
///   big-endian `u16` decoded from `data[2..4]`, in `[4, 65535]` (BC-2.20.004). The
///   reserved byte at `data[1]` is never inspected.
///
/// These four outcomes are jointly exhaustive and mutually exclusive over all possible
/// `data` inputs (BC-2.20.004 invariant 3; AC-184-005). Full exhaustiveness is proven by
/// the VP-048 Kani harness below (full proof run: STORY-194).
pub fn parse_tpkt_header(data: &[u8]) -> Option<TpktHeader> {
    if data.len() < 4 {
        return None;
    }
    todo!("STORY-184: implement version/length checks per BC-2.20.002-004")
}

// ---------------------------------------------------------------------------
// VP-048 Kani proof — parse_tpkt_header safety (ADR-014 Decision 9)
// ---------------------------------------------------------------------------
//
// SCOPE: this harness covers only `parse_tpkt_header`. `parse_cotp_header`'s Kani
// obligation is VP-049 (STORY-185); the combined no-panic frame-walk loop is
// VP-050/VP-055.
//
// Skeleton only — `parse_tpkt_header` currently `todo!()`s, so this harness cannot
// pass yet. The full proof run targeting all four BC-2.20.001-004 outcomes is executed
// in STORY-194, once the `todo!()`-free implementation lands (STORY-184 implementer
// step).
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-048: `parse_tpkt_header` must not panic for any input, of any length.
    #[kani::proof]
    fn verify_parse_tpkt_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 300);
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        // Must not panic for any input:
        let _ = parse_tpkt_header(&data);
    }
}
