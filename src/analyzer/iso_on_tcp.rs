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
//! ## Scope
//!
//! - STORY-184 delivered the TPKT (RFC 1006) outer framing header: `TpktHeader` and
//!   `parse_tpkt_header` (BC-2.20.001–004; VP-048).
//! - STORY-185 adds the COTP (ISO 8073 / ITU-T X.224) inner TPDU header:
//!   `CotpHeader`, `CotpTpduType`, `parse_cotp_header` (BC-2.20.005–012; VP-049).
//!
//! - `parse_tpkt_header` — 4-byte TPKT header parse; `None` on short/invalid input
//!   (BC-2.20.001–004); VP-048 Kani target.
//! - `parse_cotp_header` — COTP TPDU-type parse (CR/CC/DT) plus verbatim `protocol_id`
//!   extraction; `None` on short/invalid/unrecognized input (BC-2.20.005–012); VP-049
//!   Kani target.
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
//! - BC-2.20.005: `parse_cotp_header` returns `None` for input shorter than 2 bytes.
//! - BC-2.20.006: `parse_cotp_header` returns `None` when the Length Indicator declares
//!   more bytes than are present (LI-truncation).
//! - BC-2.20.007: `parse_cotp_header` recognizes Connect Request (CR) TPDUs.
//! - BC-2.20.008: `parse_cotp_header` recognizes Connect Confirm (CC) TPDUs.
//! - BC-2.20.009: `parse_cotp_header` recognizes DT TPDUs with a non-empty payload and
//!   extracts `protocol_id`.
//! - BC-2.20.010: `parse_cotp_header` recognizes DT TPDUs with an empty payload
//!   (`protocol_id: None`).
//! - BC-2.20.011: `parse_cotp_header` returns `None` for an unrecognized TPDU-type
//!   code (high nibble not in `{0xE0, 0xD0, 0xF0}`).
//! - BC-2.20.012: `protocol_id` is extracted verbatim, never interpreted (frozen SS-20
//!   to SS-21 boundary) — `parse_cotp_header` never compares the extracted byte against
//!   any specific value.
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
// COTP (ISO 8073 / ITU-T X.224) data types — STORY-185
// ---------------------------------------------------------------------------
//
// Frozen interface (ADR-014 Decision 1, Decision 2 disambiguation table, Decision 9).
// `iso_on_tcp.rs` (SS-20) performs zero interpretation of `protocol_id` — the
// upper-layer-protocol disambiguation table (ADR-014 Decision 2) lives entirely in
// `S7commAnalyzer` (SS-21), built starting in STORY-186/STORY-187.

/// The three COTP (ISO 8073) TPDU types this parser discriminates.
///
/// Frozen per ADR-014 Decision 1 — exactly these 3 variants. This is deliberately not
/// an exhaustive enumeration of all ISO 8073 TPDU codes (DR, DC, ED, AK, EA, RJ, ER, and
/// others exist on the wire but are not modeled — BC-2.20.011 requires that any of
/// those 13 remaining high-nibble values causes `parse_cotp_header` to return `None`
/// rather than being force-fit into one of these three variants).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CotpTpduType {
    /// CR — Connect Request. Session establishment; no upper-layer payload.
    ConnectRequest,
    /// CC — Connect Confirm. Session establishment; no upper-layer payload.
    ConnectConfirm,
    /// DT — Data Transfer. Carries an upper-layer payload, optionally prefixed by a
    /// single protocol-ID byte (see [`CotpHeader::protocol_id`]).
    DataTransfer,
}

/// Parsed COTP (ISO 8073 / ITU-T X.224) header, as extracted from the TPKT payload
/// slice (i.e. `data[4..length]` from an already-accepted `TpktHeader`,
/// BC-2.20.004's accept path).
///
/// Frozen per ADR-014 Decision 1 — exactly these three fields, no additional fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CotpHeader {
    /// Which of the three modeled TPDU types this frame is.
    pub tpdu_type: CotpTpduType,
    /// The raw, uninterpreted protocol-ID byte immediately following the COTP
    /// fixed-and-variable header, when present.
    ///
    /// `Some(byte)` only for a [`CotpTpduType::DataTransfer`] TPDU whose payload is
    /// non-empty (BC-2.20.009); `None` for `ConnectRequest`/`ConnectConfirm` (no
    /// upper-layer payload exists yet — BC-2.20.007, BC-2.20.008) or when the DT
    /// payload is empty (BC-2.20.010).
    ///
    /// This byte is extracted **verbatim** — `parse_cotp_header` never compares it
    /// against any specific value (BC-2.20.012, ADR-014 Decision 2). SS-21
    /// (`S7commAnalyzer`) owns all disambiguation of this value.
    pub protocol_id: Option<u8>,
    /// Byte offset into `tpkt_payload` where the upper-layer payload begins
    /// (`1 + LI`, where `LI` is the Length Indicator at `tpkt_payload[0]`).
    pub payload_offset: usize,
}

// ---------------------------------------------------------------------------
// COTP parse function — STORY-185
// ---------------------------------------------------------------------------

/// Parse a COTP (ISO 8073 / ITU-T X.224) TPDU header from `tpkt_payload`, the byte
/// slice following an already-accepted 4-byte TPKT header.
///
/// Pure-core free function (ADR-014 Decision 9) — no I/O, no global state mutation,
/// no side effects, deterministic. VP-049 Kani P0 target.
///
/// # Returns
///
/// - `None` if `tpkt_payload.len() < 2` (BC-2.20.005) — the minimum readable COTP
///   prefix is the Length Indicator (LI, offset 0) plus the TPDU-code byte (offset 1).
/// - `None` if `tpkt_payload.len() < 1 + LI` (BC-2.20.006) — the LI declares more
///   bytes than are present (LI-truncation guard); no out-of-bounds index for any `u8`
///   LI value, including `0`.
/// - `Some(CotpHeader { tpdu_type: ConnectRequest, protocol_id: None, payload_offset })`
///   if `tpkt_payload[1] & 0xF0 == 0xE0` (BC-2.20.007), where
///   `payload_offset == 1 + LI`.
/// - `Some(CotpHeader { tpdu_type: ConnectConfirm, protocol_id: None, payload_offset })`
///   if `tpkt_payload[1] & 0xF0 == 0xD0` (BC-2.20.008), where
///   `payload_offset == 1 + LI`.
/// - `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: Some(tpkt_payload[payload_offset]),
///   payload_offset })` if `tpkt_payload[1] & 0xF0 == 0xF0` and
///   `tpkt_payload.len() > payload_offset` (BC-2.20.009) — `protocol_id` is the
///   trailing byte, extracted verbatim.
/// - `Some(CotpHeader { tpdu_type: DataTransfer, protocol_id: None, payload_offset })`
///   if `tpkt_payload[1] & 0xF0 == 0xF0` and `tpkt_payload.len() == payload_offset`
///   exactly (BC-2.20.010) — no out-of-bounds index at `tpkt_payload[payload_offset]`.
/// - `None` for any other high-nibble value (BC-2.20.011) — the 13 remaining ISO 8073
///   TPDU codes (DR, DC, ED, AK, EA, RJ, ER, and others) are never modeled and never
///   force-fit into CR, CC, or DT.
///
/// These six outcomes are jointly exhaustive and mutually exclusive by construction
/// over all 16 high-nibble values (BC-2.20.011 invariant 3; AC-185-008). Formalizing
/// that partition, plus the `protocol_id` totality property (BC-2.20.012), is the
/// VP-049 Kani obligation: the assertions are added and executed in STORY-194 (formal
/// hardening); the `#[cfg(kani)]` skeleton below is scoped to check only
/// no-panic/bounds-safety over symbolic input — its proof is executed in STORY-194
/// (not run in this story).
pub fn parse_cotp_header(tpkt_payload: &[u8]) -> Option<CotpHeader> {
    if tpkt_payload.len() < 2 {
        return None;
    }
    let li = tpkt_payload[0] as usize;
    if tpkt_payload.len() < 1 + li {
        return None;
    }
    let payload_offset = 1 + li;
    let tpdu_code = tpkt_payload[1];
    match tpdu_code & 0xF0 {
        0xE0 => Some(CotpHeader {
            tpdu_type: CotpTpduType::ConnectRequest,
            protocol_id: None,
            payload_offset,
        }),
        0xD0 => Some(CotpHeader {
            tpdu_type: CotpTpduType::ConnectConfirm,
            protocol_id: None,
            payload_offset,
        }),
        0xF0 => {
            let protocol_id = if tpkt_payload.len() > payload_offset {
                Some(tpkt_payload[payload_offset])
            } else {
                None
            };
            Some(CotpHeader {
                tpdu_type: CotpTpduType::DataTransfer,
                protocol_id,
                payload_offset,
            })
        }
        _ => None,
    }
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

    /// VP-049: `parse_cotp_header` must not panic for any input, up to the bounded
    /// length (`len <= 300`).
    ///
    /// SCOPE (this story): no-panic / bounds-safety only, mirroring the VP-048 harness
    /// pattern above. The full VP-049 proof obligation — TPDU-type classification
    /// exhaustiveness over all 16 high-nibble values (BC-2.20.011 invariant 3) and
    /// protocol-ID-extraction totality over all 256 `u8` values (BC-2.20.012) — is
    /// deferred to STORY-194 (formal hardening), per this story's Kani obligation note.
    #[kani::proof]
    fn verify_parse_cotp_header_safety() {
        let len: usize = kani::any();
        kani::assume(len <= 300);
        let mut data = vec![0u8; len];
        for b in data.iter_mut() {
            *b = kani::any();
        }
        // Must not panic for any input, including the LI-truncation bounds check:
        let _ = parse_cotp_header(&data);
    }
}
