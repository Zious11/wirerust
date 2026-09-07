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
/// 254). The overflow comparison is strict `>`, never `>=` (BC-2.20.014 Edge Case
/// EC-001 / Invariant 1): a residual of exactly 65,535 bytes from a still-incomplete,
/// conformant `length = 65,535` frame is legitimate, not overflow.
pub const MAX_S7_ISO_ON_TCP_CARRY_BYTES: usize = 65_535;

// ---------------------------------------------------------------------------
// Per-flow state
// ---------------------------------------------------------------------------

/// Minimal per-flow state for [`S7commAnalyzer`] — carry-buffer fields only.
///
/// STORY-187 extends this struct with the classification/dedup fields its own
/// scope requires. Per BC-2.20.016 postcondition 3, these carry buffers live here
/// (SS-21) and nowhere else — no `IsoOnTcpFlowState` type exists anywhere in the
/// tree, and `carry_c2s`/`carry_s2c` are never merged into a single shared buffer
/// (directional isolation, BC-2.20.013 invariant 3).
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
            // exceed the bound; a bad-version-byte reject resyncs 1 byte at a time
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
                            // Complete TPKT frame: dispatch to parse_cotp_header and
                            // advance past it (BC-2.20.013 postcondition 1a). This
                            // story's protocol_id dispatch is a no-op placeholder —
                            // classification lands in STORY-187.
                            let frame = &working[cursor..cursor + total];
                            let _ = iso_on_tcp::parse_cotp_header(&frame[4..]);
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
}
