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
//! ## Scope of this story (STORY-186)
//!
//! This story proves TPKT/COTP frame extraction, carry-buffer reassembly, and
//! 1-byte resync only. Protocol-specific dispatch on the extracted
//! `CotpHeader::protocol_id` (the four-way protocol-ID dispatch contract) is
//! **not** built here — that is STORY-187's scope. `on_data`'s frame-walk loop
//! dispatches each extracted frame to `iso_on_tcp::parse_cotp_header` and then stops;
//! it does not yet interpret `protocol_id`.
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
//! - BC-2.21.003: `on_flow_close` removes `S7commFlowState` and discards all carry
//!   bytes; no finding is emitted for a flow closing with non-empty carry buffers.

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
    /// Set exactly once, on the first DT frame observed for this flow (any
    /// `protocol_id` value, including `None`) — sticky first-classification-wins
    /// (BC-2.21.002 postcondition 6, BC-2.21.001 edge case EC-002). Remains `None`
    /// until the first DT frame is observed.
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
    /// `protocol_id` is `None` or any value other than `0x32`/`0x72` on a DT frame —
    /// unclassified gap (BC-2.21.027).
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
    /// `0x03` — Ack_Data (response carrying a parameter/data block).
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
    /// `Some(data[10])` only when `rosctr == Ack`; `None` for every other ROSCTR value
    /// (BC-2.21.008 postcondition 3).
    pub error_class: Option<u8>,
    /// `Some(data[11])` only when `rosctr == Ack`; `None` for every other ROSCTR value
    /// (BC-2.21.008 postcondition 3).
    pub error_code: Option<u8>,
    /// `10` for Job/Ack_Data/Userdata (BC-2.21.006); `12` for Ack (BC-2.21.008).
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
/// # Contract summary (BC-2.21.004 through BC-2.21.008)
///
/// - `data.len() < 10` -> `None` (BC-2.21.004).
/// - `data[0] != 0x32` -> `None` (BC-2.21.005, defensive caller-hygiene guard).
/// - `data[1] ∈ {0x01, 0x03, 0x07}` (Job/Ack_Data/Userdata) -> `Some(S7commHeader {
///   .., error_class: None, error_code: None, header_len: 10 })` (BC-2.21.006).
/// - `data[1] == 0x02` (Ack) -> requires `data.len() >= 12`; `Some(S7commHeader {
///   .., error_class: Some(data[10]), error_code: Some(data[11]), header_len: 12 })`,
///   else `None` (BC-2.21.008).
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
        // BC-2.21.006: Job / Ack_Data / Userdata — common 10-byte header.
        0x01 | 0x03 | 0x07 => {
            let rosctr = match data[1] {
                0x01 => Rosctr::Job,
                0x03 => Rosctr::AckData,
                0x07 => Rosctr::Userdata,
                _ => unreachable!("data[1] is one of 0x01/0x03/0x07 in this match arm"),
            };
            Some(S7commHeader {
                rosctr,
                pdu_reference,
                param_length,
                data_length,
                error_class: None,
                error_code: None,
                header_len: 10,
            })
        }
        // BC-2.21.008: Ack requires 12 bytes (10-byte common header + error
        // class/code).
        0x02 => {
            if data.len() < 12 {
                return None;
            }
            Some(S7commHeader {
                rosctr: Rosctr::Ack,
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

// ---------------------------------------------------------------------------
// Analyzer
// ---------------------------------------------------------------------------

/// S7comm (SS-21) effectful shell: owns per-flow state and drives the TPKT/COTP
/// frame-walk loop built on SS-20's pure parse functions.
///
/// Not yet registered with the dispatcher (`DispatchTarget::S7comm` wiring is
/// STORY-193's scope) — this story creates the analyzer and proves extraction,
/// carry management, and resync in isolation.
#[derive(Debug, Default)]
pub struct S7commAnalyzer {
    /// Per-flow state, keyed by the canonical [`FlowKey`].
    pub flows: HashMap<FlowKey, S7commFlowState>,
    /// Findings accumulated across all flows processed by this analyzer (e.g. the
    /// T0814 carry-overflow finding, BC-2.20.014 postcondition 3).
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
    /// This story's dispatch on `CotpHeader::protocol_id` is a no-op placeholder —
    /// classification lands in STORY-187.
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
    /// Per this story's scope, the `Some(0x32)` classic branch is fully wired to
    /// [`Self::dispatch_classic_s7comm`], and the CR/CC session-tracking branch
    /// (BC-2.21.002 postcondition 2) updates `session_established` on CC. The
    /// `None`-from-`parse_cotp_header` branch (unclassified-gap, BC-2.21.028) and the
    /// `Some(0x72)`/`Some(other)`/`None`-protocol_id DT branches (BC-2.21.027) remain
    /// structural no-ops with no divergent/panicking body — their observable behavior
    /// is STORY-190's scope, per this story's Tasks list. The sticky-first-classification assignment
    /// (BC-2.21.002 postcondition 6) for DT frames is fully wired via
    /// [`Self::classify_first_dt_frame`] (AC-187-005).
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
                    // (BC-2.21.002 postcondition 2). `S7commFlowState` carries no
                    // dedicated "CR observed, awaiting CC" field (AC-187-001's field
                    // set is exhaustive), so a bare CR alone leaves
                    // `session_established` untouched.
                }
                iso_on_tcp::CotpTpduType::ConnectConfirm => {
                    // A CC observed on this flow marks the session established
                    // (BC-2.21.001 postcondition 1, AC-187-003). No protocol
                    // classification occurs here (BC-2.21.002 postcondition 2).
                    state.session_established = true;
                }
                iso_on_tcp::CotpTpduType::DataTransfer => {
                    Self::classify_first_dt_frame(state, header.protocol_id);
                    match header.protocol_id {
                        Some(0x32) => {
                            let payload = &tpkt_payload[header.payload_offset..];
                            Self::dispatch_classic_s7comm(state, payload, direction, ts, findings);
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
    /// (any `protocol_id` value, including `None`), `classified_protocol` is set
    /// exactly once, per ADR-014 Decision 2's four-row disambiguation table:
    /// `Some(0x32)` -> `Classic`, `Some(0x72)` -> `Plus`, anything else (including
    /// `None`) -> `Unclassified`. Subsequent DT frames on the same flow never
    /// overwrite it, even if their `protocol_id` differs.
    fn classify_first_dt_frame(state: &mut S7commFlowState, protocol_id: Option<u8>) {
        if state.classified_protocol.is_some() {
            return;
        }
        let protocol = match protocol_id {
            Some(0x32) => S7Protocol::Classic,
            Some(0x72) => S7Protocol::Plus,
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
        let Some(header) = parse_s7comm_header(payload) else {
            // BC-2.21.004/007/008: length-reject or unrecognized-ROSCTR — malformed
            // header, dedup-guarded T0814.
            Self::report_malformed_header(state, direction, ts, findings);
            return;
        };

        // BC-2.21.009: the declared param_length/data_length are bounds-checked
        // against the bytes actually remaining in `payload` before any
        // parameter/data-block slice is ever constructed. Checked arithmetic
        // (BC-2.21.009 invariant 1) — `header_len` (10 or 12) plus two `u16` values
        // cannot overflow `usize` on any wirerust target, but the checked form makes
        // that safety explicit rather than assumed, and the release profile's
        // `overflow-checks = true` would otherwise panic on any future regression.
        let declared_total = header
            .header_len
            .checked_add(header.param_length as usize)
            .and_then(|sum| sum.checked_add(header.data_length as usize));

        match declared_total {
            Some(total) if payload.len() >= total => {
                // Bounds check passes. Function-code/Userdata classification
                // (BC-2.21.010 onward) is out of this story's scope (STORY-188/189).
            }
            _ => {
                // Declared lengths exceed the bytes actually present (or, in the
                // unreachable overflow case, `checked_add` returned `None`) — treated
                // identically to a malformed header (BC-2.21.009 postcondition 2).
                Self::report_malformed_header(state, direction, ts, findings);
            }
        }
    }

    /// Emits one T0814 (Anomaly/Possible/Medium) for a malformed classic S7comm
    /// header/bounds condition, deduplicated per flow direction via
    /// `malformed_header_reported_c2s`/`_s2c` (BC-2.21.001 postcondition 1) — shared
    /// by BC-2.21.004 (too-short), BC-2.21.007 (unrecognized ROSCTR), BC-2.21.008
    /// (truncated Ack), and BC-2.21.009 (declared-length/available-bytes mismatch),
    /// since all four conditions answer the same question: "was this frame's S7comm
    /// header/declared structure internally consistent?"
    fn report_malformed_header(
        state: &mut S7commFlowState,
        direction: Direction,
        ts: u32,
        findings: &mut Vec<Finding>,
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
            summary: "Malformed classic S7comm header: the header could not be parsed, \
                      or its declared param_length/data_length exceed the bytes \
                      actually present (T0814; BC-2.21.004/007/008/009)"
                .to_string(),
            evidence: vec!["classic S7comm header parse/bounds check failed".to_string()],
            mitre_techniques: vec!["T0814".to_string()],
            source_ip: None,
            timestamp,
            direction: Some(direction),
        });
    }
}
