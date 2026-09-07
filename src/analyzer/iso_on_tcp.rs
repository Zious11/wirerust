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
//! - BC-2.20.003: `parse_tpkt_header` returns `None` for decoded length field < 7
//!   (RFC 1006 §6's stated minimum packet length; malformed, includes zero-length and
//!   header-only lengths 4-6).
//! - BC-2.20.004: `parse_tpkt_header` returns `Some(TpktHeader)` for valid input
//!   (happy path); reserved byte (`data[1]`) is never validated; accept range is
//!   `[7, 65535]`; `length == 65535` is a legal accept.
//!
//! ## Architecture compliance (ADR-014 Decision 4 — licensing)
//! Forbidden dependencies (BANNED/AVOID — licensing violation or unclear provenance):
//! - `rusty-cotp`, `rusty-tpkt`, `tpkt`, `copt` crates (unclear/non-standard license)
//! - `s7`, `s7-comm`, `s7-client` crates (non-standard custom license grant)
//! - Wireshark, Snap7, or libnodave source of any kind (GPL/LGPL — banned)
//!
//! This module is an original Rust implementation derived directly from RFC 1006 §6
//! (a freely implementable open specification). Zero lines are borrowed from any
//! external implementation.

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Parsed TPKT (RFC 1006) header — the outer 4-byte framing layer present on every TCP
/// segment carrying ISO-on-TCP traffic (S7comm, IEC 61850 MMS, ICCP/TASE.2 on TCP/102).
///
/// The TPKT header occupies exactly 4 bytes on the wire (RFC 1006 §6):
/// - `version` (byte 0): always `0x03` for a valid TPKT packet.
/// - *(reserved, byte 1)*: not surfaced by this struct — never validated by
///   `parse_tpkt_header` (BC-2.20.004 invariant 1).
/// - `length` (bytes 2–3): big-endian `u16`, total TPKT packet length **including**
///   this 4-byte header. Valid range on the accept path: `[7, 65535]` (RFC 1006 §6's
///   stated minimum packet length is 7: a 4-byte TPKT header plus a 3-byte minimum
///   COTP).
///
/// Frozen per ADR-014 Decision 1 — exactly these two fields, no `reserved` field
/// surfaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TpktHeader {
    /// TPKT version byte; always `3` for a valid TPKT packet (RFC 1006 §6).
    pub version: u8,
    /// Total TPKT packet length in bytes, including this 4-byte header.
    /// Valid range on the accept path: `[7, 65535]` (RFC 1006 §6 minimum packet
    /// length = 7).
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
/// - `None` if `data.len() < 4` (BC-2.20.001) — the structural read-guard: 4 bytes are
///   needed just to read a TPKT header's fields off the wire, independent of RFC
///   conformance.
/// - `None` if `data[0] != 0x03` (BC-2.20.002); the length field is never decoded in
///   this case.
/// - `None` if the big-endian `u16` decoded from `data[2..4]` is `< 7` (BC-2.20.003) —
///   the RFC 1006 §6 length-floor: a valid TPKT packet's declared length must be at
///   least 7 (4-byte TPKT header + 3-byte minimum COTP).
/// - `Some(TpktHeader { version: 3, length })` otherwise, where `length` is exactly the
///   big-endian `u16` decoded from `data[2..4]`, in `[7, 65535]` (BC-2.20.004). The
///   reserved byte at `data[1]` is never inspected.
///
/// These four outcomes are jointly exhaustive and mutually exclusive by construction
/// (BC-2.20.004 invariant 3; AC-184-005). Formalizing that partition is the VP-048 Kani
/// obligation: the assertions are added and executed in STORY-194 (formal hardening); the
/// `#[cfg(kani)]` skeleton below is scoped to check only no-panic/bounds-safety over
/// symbolic input — its proof is executed in STORY-194 (not run in this story).
pub fn parse_tpkt_header(data: &[u8]) -> Option<TpktHeader> {
    if data.len() < 4 {
        return None;
    }
    if data[0] != 0x03 {
        return None;
    }
    let length = u16::from_be_bytes([data[2], data[3]]);
    // RFC 1006 §6 states the minimum legal TPKT packet length is 7 (4-byte TPKT header +
    // 3-byte minimum COTP). This is distinct from the 4-byte structural read-guard above
    // (data.len() < 4): that guard is about having enough bytes to READ a header at all,
    // while this length-floor is about whether the packet's OWN DECLARED length is a
    // valid TPKT/COTP packet per the RFC. A declared length of 4-6 is header-only (or
    // near-header-only) with no room for even a minimal COTP PDU, and is rejected here.
    if length < 7 {
        return None;
    }
    Some(TpktHeader {
        version: 0x03,
        length,
    })
}

// ---------------------------------------------------------------------------
// VP-048 Kani proof — parse_tpkt_header safety (ADR-014 Decision 9)
// ---------------------------------------------------------------------------
//
// SCOPE: this harness covers only `parse_tpkt_header`. `parse_cotp_header`'s Kani
// obligation is VP-049 (STORY-185); the combined no-panic frame-walk loop is
// VP-050/VP-055.
//
// `parse_tpkt_header` is now `todo!()`-free (STORY-184). This harness compiles and is
// ready to run under `cargo kani`; the full VP-048 proof execution and evidence capture
// targeting all four BC-2.20.001-004 outcomes is STORY-194's obligation (formal-verifier
// step), per the module-level scope note above.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-048: `parse_tpkt_header` must not panic for any input, up to the bounded
    /// length (`len <= 300`).
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
