---
document_type: story
level: ops
story_id: STORY-190
title: "S7comm-plus DetectionOnly Framing + Session-Setup Metadata + Unclassified-Gap Completion (protocol_id Dispatch Totality)"
epic_id: E-23
version: "1.0"
status: ready
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 5
priority: P1
cycle: feature-s7comm
wave: 93
target_module: analyzer/s7comm
subsystems: [SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-189]
blocks: [STORY-191]
behavioral_contracts: [BC-2.21.024, BC-2.21.025, BC-2.21.026, BC-2.21.027, BC-2.21.028]
verification_properties: [VP-053]
inputs:
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.024.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.025.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.026.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.027.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.028.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
input-hash: "64414bc"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-190: S7comm-plus DetectionOnly Framing + Session-Setup Metadata + Unclassified-Gap Completion

## Narrative

**As a** security analyst using wirerust to inspect all TCP/102 traffic, including
S7comm-plus, IEC 61850 MMS, ICCP/TASE.2, and any unparseable COTP payload,
**I want** `S7commAnalyzer` to observe S7comm-plus sessions at the framing level with
bounded, pre-TLS session-setup metadata capture, and to leave every non-classic,
non-S7comm-plus port-102 flow correctly and honestly unclassified,
**so that** the four-way `protocol_id` dispatch (started in STORY-187) is fully total,
and the single most load-bearing correctness property in this feature — non-S7comm
port-102 traffic is never misattributed to S7comm — is proven.

This story completes `S7commAnalyzer::on_data`'s dispatch on `CotpHeader::protocol_id`:
the `Some(0x72)` and `Some(other)`/unparseable-COTP branches left as placeholders in
STORY-187.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.21.024 | S7comm-plus DT Frame Classified as Observed Session — Framing-Level Only | `Some(0x72)` branch, no function-code decode |
| BC-2.21.025 | S7comm-plus Unencrypted Session-Setup Handshake Metadata Observation | Bounded pre-TLS metadata capture |
| BC-2.21.026 | TLS-Wrapped S7comm-plus Defers Entirely to SS-07 | Explicit non-goal boundary |
| BC-2.21.027 | DT Frame With protocol_id: Some(other) Left Unclassified — Never Force-Fit | Load-bearing never-misattribute property |
| BC-2.21.028 | Unparseable COTP DT Payload Receives Same Unclassified-Gap Treatment | Completes the unclassified-gap outcome space |

## Acceptance Criteria

### AC-190-001: `Some(0x72)` DT frames classify as observed S7comm-plus, framing-level only
(traces to BC-2.21.024 postcondition 1)
- Given `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer,
  protocol_id: Some(0x72), .. })`
- When `on_data` dispatches this frame
- Then `S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Plus)` if not
  already set (sticky first-classification)
- The frame contributes to an "observed S7comm-plus session" count; it does not register
  the flow as `known-supported` in the protocol catalog (traces to BC-2.21.024
  postcondition 2)
- No bytes beyond `protocol_id` are interpreted as an S7comm-plus function code, object
  ID, or any other semantic structure — the classic `S7ClassicFunction` classification
  surface is never applied to a `0x72` frame (traces to BC-2.21.024 postcondition 3)
- **Test:** `test_BC_2_21_024_s7comm_plus_framing_only_classification`

### AC-190-002: Bounded pre-TLS session-setup metadata observation
(traces to BC-2.21.025 postconditions 1-2)
- Given a flow classified `S7Protocol::Plus` and no TLS signature yet observed on it,
  with at least 1 byte following `protocol_id` in the DT payload
- When `on_data` processes this frame
- Then the message-type/opcode byte (immediately following `protocol_id`) is extracted
  and recorded as raw metadata — not matched against any semantic table, not classified
  into a named enum variant
- No more than the fixed, small bounded window is read; no attempt is made to parse an
  object ID, service ID, or payload field (traces to BC-2.21.025 postcondition 4)
- **Test:** `test_BC_2_21_025_session_setup_metadata_bounded_window`

### AC-190-003: TLS detection halts further S7comm-plus metadata observation for that flow
(traces to BC-2.21.026 postcondition 1)
- Given a flow classified `S7Protocol::Plus` on which SS-07's existing TLS-handshake
  detection subsequently observes a TLS signature
- When further DT frames arrive on that flow
- Then `S7commAnalyzer` performs no further byte-level reads or metadata extraction on
  this flow's payload — the `classified_protocol: Some(Plus)` tag and any pre-TLS
  metadata already recorded remain unchanged; no new observation is added
- No integrity or anti-replay material is ever interpreted, TLS-wrapped or not (traces
  to BC-2.21.026 postcondition 2)
- **Test:** `test_BC_2_21_026_tls_detected_halts_further_observation`

### AC-190-004: `Some(other)` for other not in {0x32, 0x72} is left unclassified, never force-fit
(traces to BC-2.21.027 postcondition 1)
- Given `parse_cotp_header` returns `Some(CotpHeader { tpdu_type: DataTransfer,
  protocol_id: Some(other), .. })` where `other` is not `0x32` or `0x72`
- When `on_data` dispatches this frame
- Then `S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Unclassified)`
  — distinct from both `Classic` and `Plus`, never left ambiguous as `None` once a DT
  frame has actually been inspected
- No bytes beyond `protocol_id` are read or interpreted for this frame (traces to
  BC-2.21.027 postcondition 2)
- This flow's traffic is never counted toward S7comm's supported coverage in any report
  (traces to BC-2.21.027 postcondition 3)
- Sticky-first-classification applies uniformly: once set to `Unclassified`, it remains
  so for the flow's lifetime even if a later frame carries `Some(0x32)` or `Some(0x72)`
  (traces to BC-2.21.027 postcondition 4)
- **Test:** `test_BC_2_21_027_other_protocol_id_never_misattributed`

### AC-190-005: Unparseable COTP DT payload receives identical unclassified-gap treatment
(traces to BC-2.21.028 postcondition 1)
- Given `parse_cotp_header(tpkt_payload)` returns `None` for a frame that was not itself
  a TPKT-level reject (a complete TPKT frame was extracted, but its COTP payload could
  not be parsed into a recognized `CotpHeader`)
- When `on_data` dispatches this frame
- Then `S7commFlowState.classified_protocol` is set to `Some(S7Protocol::Unclassified)`
  if not already set; the frame is never counted as S7comm traffic in any report (traces
  to BC-2.21.028 postcondition 2)
- No bytes beyond what `parse_cotp_header` already inspected and rejected are further
  interpreted by SS-21 (traces to BC-2.21.028 postcondition 3)
- This outcome is indistinguishable, from any surfaced report, from AC-190-004's
  `Some(other)` outcome (traces to BC-2.21.028 postcondition 4)
- **Test:** `test_BC_2_21_028_unparseable_cotp_same_unclassified_treatment`

### AC-190-006: The full protocol_id four-way dispatch is total, mutually exclusive, and never misattributes non-S7comm traffic
(traces to BC-2.21.027 postcondition 1, BC-2.21.028 postcondition 1 — VP-053 totality obligation)
- Given every possible `parse_cotp_header` return value: `None`; `Some` with
  `tpdu_type ∈ {ConnectRequest, ConnectConfirm}`; `Some(0x32)` DataTransfer; `Some(0x72)`
  DataTransfer; `Some(other)` for the 254 remaining `u8` values, or `protocol_id: None`
  on an empty-payload DT
- When `on_data` dispatches every case
- Then exactly one outcome applies for each, and for ALL 254 `other` values plus the
  unparseable-COTP case, the resulting flow's `classified_protocol` is
  `Some(Unclassified)`, never `Some(Classic)` or `Some(Plus)` — no proximity-based
  fallback exists anywhere in the dispatch
- **Test:** `proptest_vp053_protocol_id_dispatch_totality` (completes the skeleton
  started in STORY-187; full non-vacuous run in STORY-194)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `S7Protocol::Plus`, `S7Protocol::Unclassified` (fully driven) | SS-21 data model | `src/analyzer/s7comm.rs` | N/A |
| S7comm-plus session-setup metadata field(s) on `S7commFlowState` | SS-21 per-flow state | `src/analyzer/s7comm.rs` | Mutable state |
| `S7commAnalyzer::on_data` (dispatch completion) | SS-21 effectful shell | `src/analyzer/s7comm.rs` | Effectful |

Subsystem anchor: SS-21 owns this story's scope — completing the four-way `protocol_id`
dispatch and the S7comm-plus/unclassified outcomes is the load-bearing correctness
property ADR-014 names explicitly per ARCH-INDEX.md §SS-21.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `S7commAnalyzer::on_data` (dispatch completion) | effectful-shell | Mutates `S7commFlowState.classified_protocol` and session-setup metadata fields; reads (but does not mutate) SS-07's TLS-detection signal |

## VP-053 Proptest Obligation (completes the skeleton started in STORY-187)

**Harness:** `proptest_vp053_protocol_id_dispatch_totality` (completed in this story)
**Method:** proptest
**Priority:** P0

This is, per the VP registration, "the highest-value correctness proof in this feature."
Non-vacuity requirement: the proptest generator MUST sweep all 256 `u8` `protocol_id`
values (not just `0x32`/`0x72`) and assert the resulting `classified_protocol` for each
of the 254 "other" values is `Some(Unclassified)`. Full non-vacuous run in STORY-194.

## Tasks

- [ ] Extend `S7commFlowState` with a bounded session-setup metadata field (e.g.
      `s7_plus_session_metadata: Option<Vec<u8>>` or an equivalent small fixed-size
      capture) per BC-2.21.025's bounded-window contract
- [ ] Complete `S7commAnalyzer::on_data`'s four-way dispatch (started in STORY-187):
  - `Some(0x72)` DT -> set `classified_protocol = Some(Plus)` (sticky-first); if no TLS
    signature yet observed on this flow, extract the bounded session-setup metadata
    window (BC-2.21.024/025)
  - TLS signature observed on a `Plus`-classified flow -> cease further metadata
    extraction for that flow (BC-2.21.026)
  - `Some(other)` for `other ∉ {0x32, 0x72}`, or `protocol_id: None` on an empty-payload
    DT -> set `classified_protocol = Some(Unclassified)` (sticky-first) (BC-2.21.027)
  - `parse_cotp_header` returns `None` (unparseable COTP) -> identical
    `Unclassified` treatment (BC-2.21.028)
- [ ] Wire a read-only check against SS-07's existing TLS-handshake detection signal for
      the flow (no modification to SS-07's own behavior)
- [ ] Complete `proptest_vp053_protocol_id_dispatch_totality`: sweep all 256 `u8`
      `protocol_id` values plus the `None`/session-TPDU/unparseable-COTP cases
- [ ] Write unit tests: one per AC, named `test_BC_2_21_024_*` .. `test_BC_2_21_028_*`
- [ ] Extend `tests/fixtures/mk_s7comm_pcap.py` with an S7comm-plus (`0x72`) framing
      skeleton per ADR-014 Decision 7
- [ ] Verify `cargo test` passes
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the S7comm-plus
      framing-only observation and the completed protocol_id dispatch, before creating
      the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.21.024 | Multiple `0x72` DT frames on the same flow | Each reaffirms/continues the observed session; no re-classification logic needed beyond sticky-first |
| EC-002 | BC-2.21.025 | DT payload immediately following `protocol_id` is empty (0 bytes) | No metadata extracted — bounded window requires at least 1 byte present |
| EC-003 | BC-2.21.026 | TLS ClientHello observed mid-session on a `Plus`-classified flow | All subsequent DT frames on this flow are skipped for metadata purposes; SS-07 continues independently |
| EC-004 | BC-2.21.027 | A flow's first DT frame is `Some(0x01)` (simulating MMS), a later frame is `Some(0x32)` | `classified_protocol` remains `Unclassified` for the flow's lifetime — sticky-first-classification never re-evaluates |
| EC-005 | BC-2.21.028 | A COTP DR (Disconnect Request, unrecognized TPDU type) frame on port 102 | `parse_cotp_header` returns `None`; treated identically to EC-004's outcome |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~5,200 |
| BC-2.21.024-028 (5 BCs) | ~5,500 |
| ADR-014 (Decisions 2, 6) | ~6,000 |
| src/analyzer/s7comm.rs (from STORY-187/188/189) | ~7,000 |
| Test file delta | ~3,000 |
| **Total** | **~26,700** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~13%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-187 | Four-way dispatch skeleton wired for session TPDUs and classic `0x32`; `Some(0x72)` and `Some(other)` routed to a `todo!()`-free placeholder | Sticky-first-classification pattern established for `classified_protocol` | This story is the correctness capstone for the dispatch — a bug here silently misattributes MMS/ICCP/S7comm-plus traffic to S7comm, inflating the `Supported` coverage count incorrectly; the proptest sweep over all 256 `u8` values (not spot-checking a handful) is not optional |

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 2**: "a COTP DT-TPDU on port 102 whose protocol-ID is not `0x32` or
  `0x72` must never be misattributed to S7comm" — the load-bearing correctness property
  this entire story exists to guarantee.
- **ADR-014 Decision 6**: S7comm-plus is "observed, not dissected" — framing-level
  classification plus bounded, pre-TLS session-setup metadata only. No
  `S7commPlusAnalyzer`, no function-code catalog, no object/service dissection. TLS-
  wrapped S7comm-plus defers entirely to the existing TLS analyzer (SS-07); wirerust
  performs no decryption attempt of any kind.
- Pure/effectful boundary: the dispatch completion is entirely within `on_data`'s
  effectful shell; no new pure-core functions are introduced by this story.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Match patterns, `Option<Vec<u8>>` |
| proptest | 1 (pinned in `Cargo.toml`) | VP-053 full-sweep totality proptest |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/s7comm.rs` | MODIFY | Complete the four-way `protocol_id` dispatch; add S7comm-plus session-setup metadata field and bounded extraction |
| `tests/s7comm_analyzer_tests.rs` | MODIFY | Add BC-2.21.024-028 unit tests + complete `proptest_vp053_protocol_id_dispatch_totality` |
| `tests/fixtures/mk_s7comm_pcap.py` | MODIFY | Add S7comm-plus (`0x72`) framing skeleton frame |

## Forbidden Dependencies

- No `S7commPlusAnalyzer`, no S7comm-plus function-code catalog, no object/service
  dissection — explicitly out of scope per ADR-014 Decision 6
- `S7commAnalyzer` MUST NOT attempt to decrypt or interpret TLS-wrapped S7comm-plus
  traffic under any circumstance

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — S7comm-plus DetectionOnly framing, bounded session-setup metadata, TLS-handoff boundary, unclassified-gap completion, VP-053 completion, AC-190-001..006. |
