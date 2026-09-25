//! S7comm PDU analyzer — the SS-21 effectful shell built on SS-20's stateless
//! ISO-on-TCP (TPKT/COTP) parsing library.
//!
//! Subsystem SS-21, CAP-21 — `analyzer/s7comm.rs` (created for the first time in
//! STORY-186).
//!
//! ## Architecture (ADR-014 Decisions 1, 2, 8)
//!
//! - **ADR-014 Decision 1**: `src/analyzer/iso_on_tcp.rs` (SS-20) is deliberately
//!   stateless — it exports pure free functions only (`parse_tpkt_header`,
//!   `parse_cotp_header`) and owns no per-flow state of its own. The directional
//!   TPKT/COTP carry buffers required for reassembly live here, on
//!   [`S7commFlowState`] (SS-21), never on a hypothetical `IsoOnTcpFlowState`.
//! - **ADR-014 Decision 2**: no `DispatchTarget::IsoOnTcp` variant is introduced at
//!   any point — SS-20 is a parsing library consumed by [`S7commAnalyzer`], not an
//!   independent dispatch target. (`S7commAnalyzer` is not yet registered with the
//!   dispatcher in this story — that wiring is STORY-193's scope.)
//! - **ADR-014 Decision 8 (WALK-FIRST-RESIDUAL-BOUND)**: [`MAX_S7_ISO_ON_TCP_CARRY_BYTES`]
//!   is derived from the TPKT `length` field's own maximum (`u16::MAX`), not COTP's
//!   254-byte LI maximum. The frame-walk loop in [`S7commAnalyzer::on_data`] runs
//!   unconditionally on `carry ++ incoming_data`; the byte bound applies only to the
//!   leftover partial-frame residual stashed back into carry. No aggregate
//!   `carry.len() + incoming_data.len()` pre-check may exist anywhere (anti-evasion,
//!   mirrors IEC-104 F-172-001 / DNP3 F-B-002).
//!
//! ## Scope
//!
//! STORY-186 proved TPKT/COTP frame extraction, carry-buffer reassembly, and
//! 1-byte resync. STORY-187 builds on that: `on_data`'s frame-walk loop now
//! interprets `CotpHeader::protocol_id` via the four-way dispatch (BC-2.21.002) —
//! CR/CC opposite-direction session tracking (F-01), sticky first-`Some(byte)`-wins
//! protocol classification (F-02), and classic S7comm (`0x32`) header dissection
//! gated on the flow's sticky `classified_protocol == Classic` (F-12) are fully
//! wired. The `Some(0x72)` (S7comm-plus) and unrecognized/`None`-`protocol_id`
//! branches remain deliberate, panic-free structural no-ops — their observable
//! behavior is STORY-190's scope.
//!
//! ## Behavioral contracts
//! - BC-2.20.013: TPKT frames spanning TCP segment boundaries are reassembled via
//!   directional carry buffers using walk-first, residual-bound semantics.
//! - BC-2.20.014: carry buffer bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`;
//!   overflow triggers clear-and-resync with one T0814 per direction.
//! - BC-2.20.015: resync anchor advances exactly 1 byte per iteration on a bad TPKT
//!   version byte (never 2).
//! - BC-2.20.016: frozen `iso_on_tcp.rs` module boundary — verified by this module's
//!   consumer relationship (SS-21 imports SS-20's pure functions; SS-20 gains no
//!   knowledge of SS-21).
//! - BC-2.21.001: `S7commFlowState` owns TPKT/COTP carry buffers, S7comm
//!   classification state (`classified_protocol`, `cr_observed_dir`,
//!   `session_established`), and per-direction malformed-header dedup flags.
//! - BC-2.21.002: `S7commAnalyzer::on_data` four-way dispatch on
//!   `CotpHeader::protocol_id` — CR/CC session tracking, sticky
//!   first-classification-wins, and the classic-dissection sticky-classification
//!   gate (F-12).
//! - BC-2.21.003: `on_flow_close` removes `S7commFlowState` and discards all carry
//!   bytes; no finding is emitted for a flow closing with non-empty carry buffers.
//! - BC-2.21.004: `parse_s7comm_header` returns `None` for input shorter than 10
//!   bytes.
//! - BC-2.21.005: `parse_s7comm_header` defensively rejects `data[0] != 0x32`.
//! - BC-2.21.006: `parse_s7comm_header` extracts the common header fields (ROSCTR,
//!   PDU reference, parameter/data length) for Job/Userdata (10-byte header).
//! - BC-2.21.007: `parse_s7comm_header` returns `None` for an unrecognized ROSCTR
//!   byte, no force-fit.
//! - BC-2.21.008: `parse_s7comm_header` for ROSCTR ∈ {Ack, Ack_Data} requires 12
//!   bytes and extracts `error_class`/`error_code` (2026-09-24 canonical-frame
//!   holdout ruling, DF-CANONICAL-FRAME-HOLDOUT-001).
//! - BC-2.21.009: declared `param_length`/`data_length` are bounds-checked (via the
//!   pure [`s7comm_bounds_ok`] helper) before any parameter/data-block slice.

use std::collections::HashMap;

use crate::analyzer::iso_on_tcp;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::Direction;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum bytes retained in a single direction's carry buffer between `on_data`
/// calls before the residual is treated as adversarial overflow (BC-2.20.014).
///
/// Derived exactly from the TPKT `length` field's own maximum representable value
/// (`u16::MAX`, RFC 1006 §6) — not from COTP's single-byte Length Indicator (max
/// 254). The overflow comparison is strict `>`, never `>=` (BC-2.20.014 Invariant 1).
///
/// Under this module's walk-first framing (BC-2.20.013), the largest residual
/// actually reachable via `on_data` is **65,534** bytes, not 65,535: a
/// declared-`length = 65,535` TPKT frame that is fully available is extracted as a
/// complete frame on the walk, not stashed to carry, so the residual can equal but
/// never exceed 65,534 (BC-2.20.014 Invariant 1 / Edge Case EC-001; see also the
/// `on_data` call-entry comment below, Invariant 5). A residual of exactly 65,535 —
/// the literal value this constant guards against — is therefore unreachable via
/// real traffic; it is exercised only by synthetic tests that inject the carry
/// buffer directly (BC-2.20.014 Edge Case EC-006). The overflow branch below is
/// retained as defense-in-depth against a future design regression, not as a
/// currently-live detection path.
pub const MAX_S7_ISO_ON_TCP_CARRY_BYTES: usize = 65_535;

// ---------------------------------------------------------------------------
// Per-flow state
// ---------------------------------------------------------------------------

/// Per-flow state for [`S7commAnalyzer`] — carry-buffer fields (STORY-186) plus the
/// S7comm classification state and malformed-header dedup flags this story
/// (STORY-187) adds (BC-2.21.001).
///
/// Per BC-2.20.016 postcondition 3, the carry buffers live here (SS-21) and nowhere
/// else — no `IsoOnTcpFlowState` type exists anywhere in the tree, and
/// `carry_c2s`/`carry_s2c` are never merged into a single shared buffer (directional
/// isolation, BC-2.20.013 invariant 3).
#[derive(Debug, Clone, Default)]
pub struct S7commFlowState {
    /// Directional carry buffer for client-to-server bytes not yet resolved into a
    /// complete TPKT frame (BC-2.20.013).
    pub carry_c2s: Vec<u8>,
    /// Directional carry buffer for server-to-client bytes not yet resolved into a
    /// complete TPKT frame (BC-2.20.013).
    pub carry_s2c: Vec<u8>,
    /// Set once a carry-buffer overflow (BC-2.20.014) has been reported for the
    /// client-to-server direction on this flow, so repeated overflow events in this
    /// direction do not each emit a new T0814 finding (BC-2.20.014 postcondition 3).
    pub carry_overflow_reported_c2s: bool,
    /// Set once a carry-buffer overflow (BC-2.20.014) has been reported for the
    /// server-to-client direction on this flow. Independent of
    /// `carry_overflow_reported_c2s` (BC-2.20.014 edge case EC-005).
    pub carry_overflow_reported_s2c: bool,
    /// Set when a COTP CR (BC-2.20.007) is followed by a matching CC (BC-2.20.008) on
    /// this flow (BC-2.21.001 postcondition 1). Classification of the upper-layer
    /// protocol is deferred to the first DT frame regardless of this flag's value
    /// (BC-2.21.002 postcondition 2).
    pub session_established: bool,
    /// Records the direction of the most recently observed COTP CR on this flow
    /// (overwritten by each subsequent CR), so a later CC can be tested for
    /// direction-opposite-ness against it (BC-2.21.001 postcondition 1's
    /// `cr_observed_dir` "at minimum" field permission, F-01 ruling, human-ratified
    /// 2026-09-24). `None` until the first CR is observed on this flow. Once set, it
    /// is never cleared by a matching opposite-direction CC — `dispatch_cotp_frame`'s
    /// `ConnectConfirm` arm only sets `session_established` and never writes back to
    /// this field, so it continues to reflect the most recent CR even after a
    /// successful CR/CC match.
    pub cr_observed_dir: Option<Direction>,
    /// Set at most once, on the first DT frame observed for this flow whose
    /// `protocol_id` is `Some(byte)` — sticky first-classification-wins
    /// (BC-2.21.002 postcondition 6, BC-2.21.001 edge case EC-002). A `protocol_id:
    /// None` DT frame carries no protocol evidence and never consumes "first DT
    /// frame" status (F-02 ruling, human-ratified 2026-09-24) — this field remains
    /// `None` until a DT frame with `Some(byte)` is observed, however many
    /// `protocol_id: None` DT frames precede it.
    pub classified_protocol: Option<S7Protocol>,
    /// Set once a malformed classic S7comm header (BC-2.21.004/007/008/009) has been
    /// reported for the client-to-server direction on this flow, so repeated
    /// malformed-header conditions in this direction do not each emit a new T0814
    /// finding. Distinct from `carry_overflow_reported_c2s` — the two dedup flags
    /// track independent anomaly classes (BC-2.21.001 invariant 2).
    pub malformed_header_reported_c2s: bool,
    /// Set once a malformed classic S7comm header has been reported for the
    /// server-to-client direction on this flow. Independent of
    /// `malformed_header_reported_c2s` (BC-2.21.001 invariant 2).
    pub malformed_header_reported_s2c: bool,
}

// ---------------------------------------------------------------------------
// S7comm data model (STORY-187)
// ---------------------------------------------------------------------------

/// Which upper-layer protocol a flow's COTP DT frames have been classified as, per
/// the `CotpHeader::protocol_id` four-way dispatch (BC-2.21.002, ADR-014 Decision 2).
///
/// `Plus` and `Unclassified` are populated by this story's dispatch skeleton but are
/// not yet fully driven — full behavior for those two branches lands in STORY-190.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum S7Protocol {
    /// `protocol_id == Some(0x32)` — classic S7comm.
    Classic,
    /// `protocol_id == Some(0x72)` — S7comm-plus.
    Plus,
    /// `protocol_id` is `Some(byte)` for a byte other than `0x32`/`0x72` on a DT
    /// frame — unclassified gap (BC-2.21.027). A `protocol_id: None` DT frame never
    /// classifies at all (F-02 ruling) and is never represented by this variant —
    /// see [`S7commFlowState::classified_protocol`].
    Unclassified,
}

/// The four classic-S7comm ROSCTR ("Remote Operating Service Control") values this
/// story models (BC-2.21.006/007/008), mirroring `CotpTpduType`'s
/// exhaustive-but-bounded design (BC-2.20.011) — this is not exhaustive over all 256
/// `u8` values by design (BC-2.21.007 invariant 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rosctr {
    /// `0x01` — Job (request).
    Job,
    /// `0x02` — Ack (bare acknowledgment; requires the 12-byte extended header,
    /// BC-2.21.008).
    Ack,
    /// `0x03` — Ack_Data (response carrying a parameter/data block; also requires
    /// the 12-byte extended header, BC-2.21.008, 2026-09-24 canonical-frame holdout
    /// ruling DF-CANONICAL-FRAME-HOLDOUT-001).
    AckData,
    /// `0x07` — Userdata.
    Userdata,
}

/// Parsed classic S7comm (protocol-ID `0x32`) common header, as extracted by
/// [`parse_s7comm_header`] (BC-2.21.004 through BC-2.21.008).
///
/// Frozen per ADR-014 Decision 9 item 3's pure-core free-fn design — this is the
/// exact field set BC-2.21.006/008 define, no additional fields.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct S7commHeader {
    /// The ROSCTR value extracted from `data[1]`.
    pub rosctr: Rosctr,
    /// `u16::from_be_bytes([data[4], data[5]])` (BC-2.21.006 postcondition 2).
    pub pdu_reference: u16,
    /// `u16::from_be_bytes([data[6], data[7]])` (BC-2.21.006 postcondition 3). Not yet
    /// validated against the actual remaining bytes in `data` — that is the caller's
    /// BC-2.21.009 obligation.
    pub param_length: u16,
    /// `u16::from_be_bytes([data[8], data[9]])` (BC-2.21.006 postcondition 4). Not yet
    /// validated against the actual remaining bytes in `data` — that is the caller's
    /// BC-2.21.009 obligation.
    pub data_length: u16,
    /// `Some(data[10])` only when `rosctr ∈ {Ack, AckData}`; `None` for Job/Userdata
    /// (BC-2.21.008 postcondition 3, 2026-09-24 canonical-frame holdout ruling
    /// DF-CANONICAL-FRAME-HOLDOUT-001).
    pub error_class: Option<u8>,
    /// `Some(data[11])` only when `rosctr ∈ {Ack, AckData}`; `None` for Job/Userdata
    /// (BC-2.21.008 postcondition 3, 2026-09-24 canonical-frame holdout ruling
    /// DF-CANONICAL-FRAME-HOLDOUT-001).
    pub error_code: Option<u8>,
    /// `10` for Job/Userdata (BC-2.21.006); `12` for Ack/Ack_Data (BC-2.21.008,
    /// 2026-09-24 canonical-frame holdout ruling DF-CANONICAL-FRAME-HOLDOUT-001).
    pub header_len: usize,
}

// ---------------------------------------------------------------------------
// Pure-core parser (STORY-187)
// ---------------------------------------------------------------------------

/// Parses the classic S7comm (protocol-ID `0x32`) common header from `data`, the COTP
/// DT payload slice beginning **at** the already-classified protocol-ID byte
/// (BC-2.21.004 precondition 1, `&tpkt_payload[payload_offset..]` from BC-2.20.009).
///
/// Pure-core free function (ADR-014 Decision 9 item 3) — no I/O, no global state
/// mutation, no side effects, deterministic; MUST NOT read `S7commAnalyzer`'s flow
/// state (this story's Forbidden Dependencies).
///
/// # Contract summary (BC-2.21.004 through BC-2.21.008; Ack_Data grouping per the
/// 2026-09-24 canonical-frame holdout ruling, DF-CANONICAL-FRAME-HOLDOUT-001)
///
/// - `data.len() < 10` -> `None` (BC-2.21.004).
/// - `data[0] != 0x32` -> `None` (BC-2.21.005, defensive caller-hygiene guard).
/// - `data[1] ∈ {0x01, 0x07}` (Job/Userdata) -> `Some(S7commHeader { ..,
///   error_class: None, error_code: None, header_len: 10 })` (BC-2.21.006).
/// - `data[1] ∈ {0x02, 0x03}` (Ack/Ack_Data) -> requires `data.len() >= 12`;
///   `Some(S7commHeader { .., error_class: Some(data[10]), error_code:
///   Some(data[11]), header_len: 12 })`, else `None` (BC-2.21.008).
/// - `data[1] ∉ {0x01, 0x02, 0x03, 0x07}` -> `None`, no force-fit (BC-2.21.007).
///
pub fn parse_s7comm_header(data: &[u8]) -> Option<S7commHeader> {
    // BC-2.21.004: 10-byte minimum guard. No bytes beyond this check are accessed
    // for any data.len() in [0, 9].
    if data.len() < 10 {
        return None;
    }
    // BC-2.21.005: defensive re-check of the protocol-ID byte (caller-hygiene, not
    // a wire-observable anomaly — no Finding is emitted for this path).
    if data[0] != 0x32 {
        return None;
    }

    let pdu_reference = u16::from_be_bytes([data[4], data[5]]);
    let param_length = u16::from_be_bytes([data[6], data[7]]);
    let data_length = u16::from_be_bytes([data[8], data[9]]);

    match data[1] {
        // BC-2.21.006: Job / Userdata — common 10-byte header (Ack_Data moved to the
        // 12-byte Ack/Ack_Data group below per the 2026-09-24 canonical-frame
        // holdout ruling, DF-CANONICAL-FRAME-HOLDOUT-001). ROSCTR mapped directly in
        // each outer match arm (F-16) — no inner re-match, no panic site anywhere in
        // this pure parser.
        0x01 => Some(S7commHeader {
            rosctr: Rosctr::Job,
            pdu_reference,
            param_length,
            data_length,
            error_class: None,
            error_code: None,
            header_len: 10,
        }),
        0x07 => Some(S7commHeader {
            rosctr: Rosctr::Userdata,
            pdu_reference,
            param_length,
            data_length,
            error_class: None,
            error_code: None,
            header_len: 10,
        }),
        // BC-2.21.008 (2026-09-24 canonical-frame holdout ruling,
        // DF-CANONICAL-FRAME-HOLDOUT-001): Ack AND Ack_Data both require the
        // 12-byte extended header (10-byte common header + error class/code) — a
        // real-world Ack_Data (Setup Communication response) parameter block only
        // aligns at byte 12, not byte 10. Ack_Data is grouped here with Ack, not
        // with Job/Userdata's 10-byte group above.
        0x02 | 0x03 => {
            if data.len() < 12 {
                return None;
            }
            let rosctr = if data[1] == 0x02 {
                Rosctr::Ack
            } else {
                Rosctr::AckData
            };
            Some(S7commHeader {
                rosctr,
                pdu_reference,
                param_length,
                data_length,
                error_class: Some(data[10]),
                error_code: Some(data[11]),
                header_len: 12,
            })
        }
        // BC-2.21.007: unrecognized ROSCTR byte — safe-reject, no force-fit.
        _ => None,
    }
}

/// BC-2.21.009 / F-14: pure, public (`pub fn`) caller-side bounds check — `true` iff
/// `data_len >= header.header_len + header.param_length as usize + header.data_length
/// as usize` (checked addition; the sum cannot overflow `usize` on any wirerust
/// target, BC-2.21.009 invariant 1). Extracted as a standalone `pub fn` (rather than
/// inlined at the `on_data` call site only) so the VP-051 Kani harness can call it
/// directly, independent of the effectful `S7commAnalyzer::dispatch_classic_s7comm`
/// call site (human ruling, STORY-187 per-story adversarial pass 1, F-14,
/// 2026-09-24).
///
/// Pure-core free function: no I/O, no global state, no side effects.
pub fn s7comm_bounds_ok(header: &S7commHeader, data_len: usize) -> bool {
    let declared_total = header
        .header_len
        .checked_add(header.param_length as usize)
        .and_then(|sum| sum.checked_add(header.data_length as usize));
    matches!(declared_total, Some(total) if data_len >= total)
}

// ---------------------------------------------------------------------------
// Analyzer
// ---------------------------------------------------------------------------

/// S7comm (SS-21) effectful shell: owns per-flow state and drives the TPKT/COTP
/// frame-walk loop built on SS-20's pure parse functions.
///
/// Not yet registered with the dispatcher (`DispatchTarget::S7comm` wiring is
/// STORY-193's scope) — TPKT/COTP frame extraction, carry management, and resync
/// are proven in isolation, and `protocol_id` dispatch (session tracking, sticky
/// classification, and gated classic-header dissection) is wired on top of it.
#[derive(Debug, Default)]
pub struct S7commAnalyzer {
    /// Per-flow state, keyed by the canonical [`FlowKey`].
    pub flows: HashMap<FlowKey, S7commFlowState>,
    /// Findings accumulated across all flows processed by this analyzer: the T0814
    /// carry-overflow finding (BC-2.20.014 postcondition 3) and the T0814
    /// malformed classic-S7comm-header finding (parse failure or bounds-check
    /// failure, BC-2.21.004/007/008/009, F-11).
    pub findings: Vec<Finding>,
}

impl S7commAnalyzer {
    /// Construct a new, empty `S7commAnalyzer`.
    pub fn new() -> Self {
        Self {
            flows: HashMap::new(),
            findings: Vec::new(),
        }
    }

    /// Process a chunk of reassembled TCP stream data for `flow_key`, in `direction`.
    ///
    /// Implements the BC-2.20.013 walk-first, residual-bound frame-walk loop:
    ///
    /// 1. Overflow check at entry on the directional carry (before appending the new
    ///    delivery or walking) per BC-2.20.014's walk-first-residual-bound semantics —
    ///    never an aggregate `carry.len() + data.len()` pre-check (BC-2.20.013
    ///    postcondition 2, invariant 1).
    /// 2. `working = carry[direction] ++ data`; repeatedly call
    ///    `iso_on_tcp::parse_tpkt_header(&working[cursor..])`: a complete frame
    ///    (`Some(header)` and enough trailing bytes) is extracted and dispatched to
    ///    `iso_on_tcp::parse_cotp_header`, `cursor` advances by `header.length`, and
    ///    the loop continues; a declared-but-incomplete frame or a `None` result
    ///    breaks the loop.
    /// 3. On a bad-version-byte reject (or immediately after a carry-overflow clear),
    ///    the shared 1-byte resync sub-routine (BC-2.20.015; see
    ///    [`Self::resync_one_byte`]) advances the cursor and retries.
    /// 4. Whatever remains after the loop terminates is stashed to `carry[direction]`.
    ///
    /// Each extracted frame's `CotpHeader` is then routed through the BC-2.21.002
    /// four-way dispatch (see [`Self::dispatch_cotp_frame`]): CR/CC frames update
    /// session-tracking state, and Data Transfer frames drive sticky protocol
    /// classification and, for classic (`0x32`) S7comm on a sticky-Classic flow,
    /// header dissection via [`parse_s7comm_header`].
    pub fn on_data(&mut self, flow_key: FlowKey, data: &[u8], ts: u32, direction: Direction) {
        use crate::findings::{Confidence, ThreatCategory, Verdict};

        // Collect frame-walk findings locally to avoid a borrow conflict between the
        // per-flow state entry (below) and `self.findings`.
        let mut local_findings: Vec<Finding> = Vec::new();

        {
            let state = self.flows.entry(flow_key).or_default();

            // BC-2.20.014 precondition 2 / postconditions 1-4: overflow check on the
            // directional carry ALONE, before the current delivery is appended and the
            // walk begins (walk-first, residual-bound semantics — BC-2.20.013
            // postcondition 2, invariant 1: no aggregate carry+delivery pre-check).
            //
            // DEFENSE-IN-DEPTH, NOT LIVE DETECTION (human ruling, Option B,
            // 2026-09-07): under the current walk-first + 1-byte-resync + u16
            // length-cap design, this branch is unreachable via `on_data`. The
            // frame-walk loop below stashes at most a declared-but-incomplete TPKT
            // frame to carry, and a TPKT `length` field is a `u16` (max 65,535 —
            // `MAX_S7_ISO_ON_TCP_CARRY_BYTES`), so the residual can equal but never
            // exceed 65,534 — one byte short of that literal bound, since a residual
            // of exactly 65,535 would itself have been a complete, dispatchable frame
            // on the walk that produced it (walk-first framing, BC-2.20.014
            // Invariant 1); a bad-version-byte reject resyncs 1 byte at a time
            // rather than accumulating carry. BC-2.20.014 v1.1 formalizes this as
            // Invariant 5: the directional carry is provably `<= 65,534` bytes
            // on entry to `on_data` (strictly less than `MAX_S7_ISO_ON_TCP_CARRY_BYTES`,
            // since a carry of exactly 65,535 would itself have been a complete,
            // dispatchable frame on the walk that produced it), so `carry.len() >
            // MAX_S7_ISO_ON_TCP_CARRY_BYTES` never evaluates true by construction.
            //
            // The check — and its placement at call-entry on the directional carry,
            // reconciled as correct/equivalent per BC-2.20.014 v1.1 Invariant 5 — is
            // retained anyway, guarding only against a *future* design regression
            // (e.g. a change that stashes more than one incomplete frame's worth of
            // bytes to carry, or relaxes the resync step size). It is intentionally
            // not deleted: removing it would silently drop the safety net the next
            // time this module's invariants change. Do not read `local_findings`
            // ever containing a T0814 in this story's test suite as evidence the
            // branch is live; it is not exercised by `on_data` today.
            {
                let (carry, reported) = if direction == Direction::ClientToServer {
                    (&mut state.carry_c2s, &mut state.carry_overflow_reported_c2s)
                } else {
                    (&mut state.carry_s2c, &mut state.carry_overflow_reported_s2c)
                };
                if carry.len() > MAX_S7_ISO_ON_TCP_CARRY_BYTES {
                    // Clear, not truncate (BC-2.20.014 invariant 2) — the oversized
                    // residual has no reliable frame boundary to preserve.
                    carry.clear();
                    if !*reported {
                        // T0814-emission branch — DEFENSE-IN-DEPTH, NOT LIVE DETECTION
                        // (see the enclosing overflow-check comment above and
                        // BC-2.20.014 v1.1 Invariant 5): unreachable via `on_data`
                        // under the current walk-first/1-byte-resync/u16-length-cap
                        // design, retained only against a future design regression.
                        // The `chrono::DateTime` timestamp conversion is deferred to
                        // this rare branch (rather than computed unconditionally at
                        // the top of `on_data`) since it is otherwise-unreachable and
                        // its only consumer is this `Finding`.
                        *reported = true;
                        let timestamp = chrono::DateTime::from_timestamp(ts as i64, 0);
                        local_findings.push(Finding {
                            category: ThreatCategory::Anomaly,
                            verdict: Verdict::Possible,
                            confidence: Confidence::Medium,
                            summary: format!(
                                "S7comm/ISO-on-TCP directional carry residual overflow: carry \
                                 buffer exceeded MAX_S7_ISO_ON_TCP_CARRY_BYTES={MAX_S7_ISO_ON_TCP_CARRY_BYTES} \
                                 — adversarial or non-conformant byte sequence; carry cleared \
                                 and the walk resyncs on this delivery (T0814; BC-2.20.014)"
                            ),
                            evidence: vec![format!(
                                "carry overflow (>{MAX_S7_ISO_ON_TCP_CARRY_BYTES}); carry cleared"
                            )],
                            mitre_techniques: vec!["T0814".to_string()],
                            source_ip: None,
                            timestamp,
                            direction: Some(direction),
                        });
                    }
                    // Carry is now cleared; the walk proceeds on the delivery alone
                    // (fresh-start resync, not a permanent desync latch — BC-2.20.014
                    // postcondition 2).
                }
            }

            // BC-2.20.013 precondition 3: working = carry[direction] ++ incoming_data.
            let mut working: Vec<u8> = if direction == Direction::ClientToServer {
                std::mem::take(&mut state.carry_c2s)
            } else {
                std::mem::take(&mut state.carry_s2c)
            };
            working.extend_from_slice(data);

            // Frame-walk loop (BC-2.20.013 postcondition 1). Runs unconditionally on
            // the full working buffer — no aggregate byte-count bound is ever applied
            // here; only the leftover residual stashed back to carry is bounded
            // (BC-2.20.014), checked above at call entry.
            let mut cursor = 0usize;
            loop {
                if working.len() - cursor < 4 {
                    // Fewer than 4 bytes remain: cannot even attempt a TPKT header
                    // read. Stash the remainder to carry below (BC-2.20.015
                    // postcondition 3(b)).
                    break;
                }
                match iso_on_tcp::parse_tpkt_header(&working[cursor..]) {
                    Some(header) => {
                        let total = header.length as usize;
                        if working.len() - cursor >= total {
                            // Complete TPKT frame: dispatch to parse_cotp_header, then
                            // to this story's four-way protocol_id dispatch skeleton
                            // (BC-2.21.002), and advance past it (BC-2.20.013
                            // postcondition 1a).
                            let frame = &working[cursor..cursor + total];
                            let tpkt_payload = &frame[4..];
                            let cotp = iso_on_tcp::parse_cotp_header(tpkt_payload);
                            Self::dispatch_cotp_frame(
                                state,
                                cotp,
                                tpkt_payload,
                                direction,
                                ts,
                                &mut local_findings,
                            );
                            cursor += total;
                        } else {
                            // Declared-but-incomplete: stash the entire partial frame
                            // (including its parsed header) to carry (BC-2.20.013
                            // postcondition 1b).
                            break;
                        }
                    }
                    None => {
                        // Bad version byte (or a rejected length field, EC-004): resync
                        // via the shared 1-byte-advance sub-routine (BC-2.20.015),
                        // reused verbatim whether reached from an ordinary mid-stream
                        // reject or the post-carry-overflow fresh-start walk above
                        // (BC-2.20.015 invariant 3 / AC-186-008 — there is exactly one
                        // resync implementation).
                        cursor = Self::resync_one_byte(&working, cursor);
                    }
                }
            }

            let remainder = working[cursor..].to_vec();
            if direction == Direction::ClientToServer {
                state.carry_c2s = remainder;
            } else {
                state.carry_s2c = remainder;
            }
        }

        self.findings.extend(local_findings);
    }

    /// Remove `flow_key`'s [`S7commFlowState`], discarding any carry bytes with no
    /// finding emitted (BC-2.21.003). A no-op if no state exists for `flow_key`.
    pub fn on_flow_close(&mut self, flow_key: FlowKey) {
        self.flows.remove(&flow_key);
    }

    /// Shared 1-byte resync sub-routine (BC-2.20.015).
    ///
    /// Reused verbatim for both an ordinary bad-version-byte reject encountered
    /// mid-stream and the post-carry-overflow resync (BC-2.20.014) — there is exactly
    /// one resync implementation, not two (BC-2.20.015 invariant 3 / AC-186-008).
    ///
    /// Advances `cursor` by exactly 1 byte per iteration (never 2, BC-2.20.015
    /// invariant 1) until either `iso_on_tcp::parse_tpkt_header` succeeds at the new
    /// offset, or fewer than 4 bytes remain (BC-2.20.015 postcondition 3(b)), at which
    /// point the caller stashes the remainder to carry per the ordinary
    /// incomplete-frame path (BC-2.20.013).
    ///
    /// Returns the new cursor position.
    fn resync_one_byte(working: &[u8], cursor: usize) -> usize {
        let mut cursor = cursor;
        while working.len() - cursor >= 4
            && iso_on_tcp::parse_tpkt_header(&working[cursor..]).is_none()
        {
            cursor += 1;
        }
        cursor
    }

    /// BC-2.21.002 four-way dispatch on `parse_cotp_header`'s return value — the
    /// single integration point between SS-20's frame extraction and SS-21's
    /// protocol-specific dissection.
    ///
    /// `tpkt_payload` is `frame[4..]` (the bytes `parse_cotp_header` was called on);
    /// `cotp` is that call's result.
    ///
    /// The CR/CC session-tracking branch (BC-2.21.002 postcondition 2) updates
    /// `session_established` only on a CC observed in the direction OPPOSITE a
    /// previously-recorded CR on this flow (F-01 ruling, BC-2.21.001
    /// postcondition 1) — a bare CR, a CC with no prior CR, an out-of-order CC, and
    /// a same-direction CC all leave it untouched. The sticky-first-classification
    /// assignment (BC-2.21.002 postcondition 6) for DT frames is fully wired via
    /// [`Self::classify_first_dt_frame`] (AC-187-005; a `protocol_id: None` DT
    /// frame never classifies, F-02). The `Some(0x32)` classic branch is fully
    /// wired to [`Self::dispatch_classic_s7comm`], but gated on the flow's STICKY
    /// `classified_protocol == Some(Classic)` (F-12, BC-2.21.002 postcondition 3 /
    /// invariant 4) — a flow already sticky-classified `Plus`/`Unclassified` is
    /// never dissected, even on a later `0x32`-leading DT frame. The
    /// `None`-from-`parse_cotp_header` branch (unclassified-gap, BC-2.21.028), the
    /// `Some(0x72)` DT branch (S7comm-plus framing-only path, BC-2.21.024/025/026,
    /// STORY-190 scope), and the `Some(other)`/`None`-protocol_id DT branches
    /// (unclassified gap, BC-2.21.027) remain structural no-ops with no
    /// divergent/panicking body — their observable behavior is STORY-190's scope.
    fn dispatch_cotp_frame(
        state: &mut S7commFlowState,
        cotp: Option<iso_on_tcp::CotpHeader>,
        tpkt_payload: &[u8],
        direction: Direction,
        ts: u32,
        findings: &mut Vec<Finding>,
    ) {
        match cotp {
            None => {
                // Unclassified gap (BC-2.21.028): `parse_cotp_header` could not
                // interpret this COTP payload. Routed to a STORY-190 placeholder
                // no-op per this story's Tasks list — never counted as S7comm, never
                // force-fit into a recognized branch.
            }
            Some(header) => match header.tpdu_type {
                iso_on_tcp::CotpTpduType::ConnectRequest => {
                    // Session tracking only, no protocol classification
                    // (BC-2.21.002 postcondition 2). Records/updates the pending
                    // CR direction so a later CC on this flow can be tested for
                    // direction-opposite-ness (F-01 ruling, BC-2.21.001
                    // postcondition 1) -- a bare CR alone never sets
                    // `session_established` itself.
                    state.cr_observed_dir = Some(direction);
                }
                iso_on_tcp::CotpTpduType::ConnectConfirm => {
                    // A CC observed on this flow marks the session established
                    // ONLY when a CR was previously recorded in the OPPOSITE
                    // direction (F-01 ruling, BC-2.21.001 postcondition 1,
                    // AC-187-003). A CC with no prior CR, a CC before any CR
                    // (out-of-order -- matching is forward-looking from the CR
                    // only, never retroactive from the CC), or a CC in the SAME
                    // direction as the recorded CR all leave `session_established`
                    // untouched. No protocol classification occurs here
                    // (BC-2.21.002 postcondition 2).
                    if let Some(cr_dir) = state.cr_observed_dir
                        && cr_dir != direction
                    {
                        state.session_established = true;
                    }
                }
                iso_on_tcp::CotpTpduType::DataTransfer => {
                    Self::classify_first_dt_frame(state, header.protocol_id);
                    match header.protocol_id {
                        Some(0x32) => {
                            // F-12 gate (BC-2.21.002 postcondition 3 / invariant 4):
                            // classic dissection fires only when the flow's STICKY
                            // classified_protocol is (or, by this very frame's own
                            // first-classification above, becomes) Classic — never
                            // merely because the current frame's raw protocol_id
                            // byte is 0x32. A flow already sticky-classified Plus or
                            // Unclassified by an earlier DT frame must never fall
                            // into classic dissection, even on a later 0x32-leading
                            // frame (ADR-014 Decision 2 no-misattribution guarantee,
                            // F-12 ruling, human-ratified 2026-09-24).
                            if state.classified_protocol == Some(S7Protocol::Classic) {
                                let payload = &tpkt_payload[header.payload_offset..];
                                Self::dispatch_classic_s7comm(
                                    state, payload, direction, ts, findings,
                                );
                            }
                        }
                        Some(0x72) => {
                            // S7comm-plus framing-only path (BC-2.21.024/025/026) —
                            // completed structurally in STORY-190. Deliberate
                            // no-divergent-body placeholder no-op per this story's
                            // Tasks list.
                        }
                        _ => {
                            // Unrecognized protocol_id, or an empty DT payload
                            // (protocol_id: None, BC-2.20.010) — unclassified gap
                            // (BC-2.21.027). Deliberate no-divergent-body placeholder
                            // no-op per this story's Tasks list.
                        }
                    }
                }
            },
        }
    }

    /// BC-2.21.002 postcondition 6 / BC-2.21.001 edge case EC-002:
    /// sticky-first-classification-wins. On the first DT frame observed for a flow
    /// whose `protocol_id` is `Some(byte)`, `classified_protocol` is set exactly
    /// once, per ADR-014 Decision 2's four-row disambiguation table: `Some(0x32)` ->
    /// `Classic`, `Some(0x72)` -> `Plus`, any other `Some(byte)` -> `Unclassified`.
    /// Subsequent DT frames on the same flow never overwrite it, even if their
    /// `protocol_id` differs.
    ///
    /// A `protocol_id: None` DT frame (empty payload, BC-2.20.010) carries NO
    /// protocol evidence and never classifies -- it does not consume the flow's
    /// "first DT frame" status, so classification remains deferred to a later DT
    /// frame (if any) that carries `Some(byte)` (F-02 ruling, human-ratified
    /// 2026-09-24, BC-2.21.002 postcondition 6 / edge case EC-004).
    fn classify_first_dt_frame(state: &mut S7commFlowState, protocol_id: Option<u8>) {
        let Some(byte) = protocol_id else {
            // No protocol evidence; leaves classified_protocol (and "first DT
            // frame" status) untouched regardless of its current value (F-02).
            return;
        };
        if state.classified_protocol.is_some() {
            return;
        }
        let protocol = match byte {
            0x32 => S7Protocol::Classic,
            0x72 => S7Protocol::Plus,
            _ => S7Protocol::Unclassified,
        };
        state.classified_protocol = Some(protocol);
    }

    /// BC-2.21.002 postcondition 3: classic S7comm (`protocol_id == Some(0x32)`)
    /// dissection entry point. Calls [`parse_s7comm_header`] on `payload` (the DT
    /// payload slice beginning at `payload_offset`) and applies the BC-2.21.009
    /// caller-side bounds check (`data.len() >= header_len + param_length +
    /// data_length`) before any parameter/data-block slice — malformed headers and
    /// bounds failures emit one T0814 per flow direction via
    /// `malformed_header_reported_c2s`/`_s2c` (BC-2.21.001).
    ///
    fn dispatch_classic_s7comm(
        state: &mut S7commFlowState,
        payload: &[u8],
        direction: Direction,
        ts: u32,
        findings: &mut Vec<Finding>,
    ) {
        // F-15: this function is only ever reached from the `Some(0x32)` DT arm
        // above, whose `payload` is `&tpkt_payload[header.payload_offset..]` with
        // `header.protocol_id == Some(tpkt_payload[header.payload_offset])`
        // (BC-2.20.009) -- i.e. `payload[0]` is always the already-classified
        // protocol-ID byte. Debug-only self-check of that call-site invariant;
        // never a production-reachable panic (release builds omit this entirely).
        debug_assert_eq!(
            payload.first(),
            Some(&0x32u8),
            "dispatch_classic_s7comm must only be called with a payload beginning \
             at the already-classified 0x32 protocol-ID byte"
        );

        let Some(header) = parse_s7comm_header(payload) else {
            // BC-2.21.004/007/008: length-reject or unrecognized-ROSCTR — malformed
            // header, dedup-guarded T0814. F-11: the specific reject reason (and,
            // where applicable, declared-vs-available byte counts) is classified
            // separately for evidence purposes only -- it never influences the
            // accept/reject decision itself, which remains `parse_s7comm_header`'s
            // sole responsibility.
            let reason = Self::classify_malformed_header_reason(payload);
            Self::report_malformed_header(state, direction, ts, findings, &reason);
            return;
        };

        // BC-2.21.009 / F-14: the declared param_length/data_length are
        // bounds-checked against the bytes actually remaining in `payload` before
        // any parameter/data-block slice is ever constructed. Delegates to the
        // extracted pure helper [`s7comm_bounds_ok`] rather than re-inlining the
        // checked-arithmetic comparison here, so this call site and the VP-051
        // Kani harness share a single source of truth for the bounds decision.
        if s7comm_bounds_ok(&header, payload.len()) {
            // Bounds check passes. Function-code/Userdata classification
            // (BC-2.21.010 onward) is out of this story's scope (STORY-188/189).
        } else {
            // Declared lengths exceed the bytes actually present (or, in the
            // unreachable overflow case, the sum would have overflowed `usize`) —
            // treated identically to a malformed header (BC-2.21.009
            // postcondition 2). F-11: report the declared-vs-available byte counts.
            let declared =
                header.header_len as u64 + header.param_length as u64 + header.data_length as u64;
            let reason = format!(
                "declared param_length/data_length exceed available bytes: \
                 declared {declared} (header_len={} + param_length={} + data_length={}), \
                 available {} (BC-2.21.009)",
                header.header_len,
                header.param_length,
                header.data_length,
                payload.len()
            );
            Self::report_malformed_header(state, direction, ts, findings, &reason);
        }
    }

    /// F-11: classifies *why* [`parse_s7comm_header`] rejected `payload`, purely to
    /// produce specific T0814 evidence text -- this classification never affects
    /// the accept/reject decision itself, which remains `parse_s7comm_header`'s
    /// sole responsibility (this function is only ever called after that function
    /// has already returned `None` for the same `payload`).
    fn classify_malformed_header_reason(payload: &[u8]) -> String {
        if payload.len() < 10 {
            return format!(
                "header too short: {} byte(s) available, 10 required (BC-2.21.004)",
                payload.len()
            );
        }
        if payload[0] != 0x32 {
            // Unreachable via dispatch_classic_s7comm's only call site (guarded by
            // the debug_assert_eq! in that function, F-15) -- retained as a
            // defensive fallback so this classifier never panics or mis-labels a
            // future, differently-guarded call site. Because this call site
            // guarantees payload[0] == 0x32, this branch never actually executes
            // today, so BC-2.21.005 postcondition 3 (no Finding emitted for a
            // direct, non-dispatch-gated reject) is not violated in practice --
            // the string it would produce is unreachable, not observed.
            return format!(
                "unexpected protocol-ID byte 0x{:02x} (expected 0x32, BC-2.21.005)",
                payload[0]
            );
        }
        match payload[1] {
            0x02 => format!(
                "truncated Ack header: {} byte(s) available, 12 required (BC-2.21.008)",
                payload.len()
            ),
            0x03 => format!(
                "truncated Ack_Data header: {} byte(s) available, 12 required \
                 (BC-2.21.008, DF-CANONICAL-FRAME-HOLDOUT-001)",
                payload.len()
            ),
            other => format!("unrecognized ROSCTR byte 0x{other:02x} (BC-2.21.007)"),
        }
    }

    /// Emits one T0814 (Anomaly/Possible/Medium) for a malformed classic S7comm
    /// header/bounds condition, deduplicated per flow direction via
    /// `malformed_header_reported_c2s`/`_s2c` (BC-2.21.001 postcondition 1) — shared
    /// by BC-2.21.004 (too-short), BC-2.21.007 (unrecognized ROSCTR), BC-2.21.008
    /// (truncated Ack, truncated Ack_Data), and BC-2.21.009 (declared-length/
    /// available-bytes mismatch), since all five conditions answer the same
    /// question: "was this frame's S7comm header/declared structure internally
    /// consistent?"
    ///
    /// `reason` (F-11) is the specific, human-readable cause -- e.g. "header too
    /// short: 4 byte(s) available, 10 required" or (for header_len=10,
    /// param_length=3, data_length=5, available=12) "declared param_length/
    /// data_length exceed available bytes: declared 18 (header_len=10 +
    /// param_length=3 + data_length=5), available 12 (BC-2.21.009)" -- computed
    /// by the caller via [`Self::classify_malformed_header_reason`] (parse
    /// failures) or inline (bounds-check failure). It never influences dedup or
    /// the T0814 emission decision, only the finding's summary/evidence text.
    fn report_malformed_header(
        state: &mut S7commFlowState,
        direction: Direction,
        ts: u32,
        findings: &mut Vec<Finding>,
        reason: &str,
    ) {
        use crate::findings::{Confidence, ThreatCategory, Verdict};

        let reported = if direction == Direction::ClientToServer {
            &mut state.malformed_header_reported_c2s
        } else {
            &mut state.malformed_header_reported_s2c
        };
        if *reported {
            return;
        }
        *reported = true;

        let timestamp = chrono::DateTime::from_timestamp(ts as i64, 0);
        findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Possible,
            confidence: Confidence::Medium,
            summary: format!(
                "Malformed classic S7comm header: {reason} (T0814; \
                 BC-2.21.004/007/008/009)"
            ),
            evidence: vec![reason.to_string()],
            mitre_techniques: vec!["T0814".to_string()],
            source_ip: None,
            timestamp,
            direction: Some(direction),
        });
    }
}
