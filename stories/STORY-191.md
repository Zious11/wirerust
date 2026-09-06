---
document_type: story
level: ops
story_id: STORY-191
title: "S7comm Download-Session Correlation State Machine + T0843/T0889/T0821 New MITRE Techniques (Six-Part Atomic + Two New MitreTactic Variants)"
epic_id: E-23
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-09-06T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 8
priority: P1
cycle: feature-s7comm
wave: 94
target_module: analyzer/s7comm
subsystems: [SS-10, SS-21]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-s7comm
depends_on: [STORY-190]
blocks: [STORY-192]
behavioral_contracts: [BC-2.21.029, BC-2.21.030, BC-2.21.031, BC-2.21.032]
verification_properties: [VP-007]
inputs:
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.029.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.030.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.031.md
  - .factory/specs/behavioral-contracts/ss-21/BC-2.21.032.md
  - .factory/specs/architecture/ARCH-INDEX.md
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/research/s7comm-mitre-ics-tagging.md
input-hash: "61fdbe2"
---

> **tdd_mode:** `strict` — full TDD Iron Law enforced.

# STORY-191: S7comm Download-Session Correlation State Machine + T0843/T0889/T0821 New MITRE Techniques

## Narrative

**As a** security engineer using wirerust to detect S7comm-based PLC program deployment
and controller-tasking manipulation,
**I want** a per-flow download-session correlation state machine that tracks
RequestDownload -> DownloadBlock(xN) -> DownloadEnded sequences with block-type-hint
capture, and the three new MITRE ATT&CK for ICS techniques (T0843 Program Download,
T0889 Modify Program, T0821 Modify Controller Tasking) correctly seeded and emitted,
**so that** a completed download session — the single richest evidence signal this
feature produces — is correctly attributed with defensible confidence.

This is the first "part B2" (MITRE emission) story in this epic. It introduces
`MAX_S7COMM_FINDINGS` (first referenced here) and performs the six-part atomic
`mitre.rs` catalog update, following the ADR-013/ADR-010 precedent exactly.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.21.029 | Download-Session Correlation State Machine, With Block-Type Hint Capture | State-only substrate; no MITRE emission of its own |
| BC-2.21.030 | Completed Download Session Emits T0843 Program Download Finding | Base per-session finding, NEW technique |
| BC-2.21.031 | T0889 Modify Program Co-Tagged on Download Completion, or Standalone on Block Activate/Delete | Two independent emission paths, NEW technique |
| BC-2.21.032 | T0821 Modify Controller Tasking Co-Tagged, Gated by Block-Type Decodability | Reuses existing tactic, NEW technique |

## Acceptance Criteria

### AC-191-001: Download-session state machine transitions correctly through Idle/InProgress/Completed
(traces to BC-2.21.029 postcondition 3)
- Given `S7commFlowState.download_state == Idle` and a `RequestDownload` frame is
  classified
- When the state machine processes it
- Then it transitions to `InProgress { blocks_seen: 0, block_type_hint }`, where
  `block_type_hint` is decoded from the Request Download parameter block's filename-style
  field: `"OB"` -> `OrganizationBlock`; any other 2-char code -> `OtherBlockType(code)`;
  unparseable/absent -> `Undeterminable`
- **Test:** `test_BC_2_21_029_idle_to_in_progress_on_request_download`

### AC-191-002: DownloadBlock increments blocks_seen while InProgress; carries hint unchanged
(traces to BC-2.21.029 postcondition 5)
- Given `download_state == InProgress { blocks_seen, block_type_hint }` and a
  `DownloadBlock` frame is classified
- When the state machine processes it
- Then it transitions to `InProgress { blocks_seen: blocks_seen + 1, block_type_hint }` —
  the hint is carried, never re-decoded from a Download Block frame
- **Test:** `test_BC_2_21_029_download_block_increments_blocks_seen`

### AC-191-003: A new RequestDownload while InProgress or Completed abandons the prior session without emission
(traces to BC-2.21.029 postcondition 4)
- Given `download_state ∈ {InProgress { .. }, Completed { .. }}` and a new
  `RequestDownload` frame is classified
- When the state machine processes it
- Then the prior session (complete or incomplete) is abandoned without emission; the new
  `RequestDownload` starts a fresh `InProgress { blocks_seen: 0, block_type_hint }` with
  its own decoded hint — an abandoned incomplete session never retroactively emits
- **Test:** `test_BC_2_21_029_new_request_download_abandons_prior_session`

### AC-191-004: DownloadEnded while InProgress transitions to Completed, including zero-block sessions
(traces to BC-2.21.029 postcondition 7)
- Given `download_state == InProgress { blocks_seen, block_type_hint }` and a
  `DownloadEnded` frame is classified
- When the state machine processes it
- Then it transitions to `Completed { blocks_seen, block_type_hint }` — this transition
  is the single event BC-2.21.030/031/032 key their emission on; a zero-block session
  (`blocks_seen == 0`) still transitions to `Completed`
- **Test:** `test_BC_2_21_029_download_ended_transitions_to_completed_including_zero_block`

### AC-191-005: Out-of-sequence DownloadBlock/DownloadEnded frames leave download_state unchanged
(traces to BC-2.21.029 postconditions 6, 8)
- Given `download_state ∈ {Idle, Completed { .. }}` and a `DownloadBlock` or
  `DownloadEnded` frame is classified (no active session)
- When the state machine processes it
- Then `download_state` is unchanged — the frame is still classified at the B1 layer,
  but does not start or extend a session at this state-machine layer
- **Test:** `test_BC_2_21_029_out_of_sequence_frames_no_state_change`

### AC-191-006: After a Completed transition is consumed, download_state resets to Idle
(traces to BC-2.21.029 postcondition 9)
- Given `download_state` just transitioned to `Completed { .. }` and BC-2.21.030's
  emission has been consumed
- When the next frame arrives
- Then `download_state` has reset to `Idle` — the flow is ready for a new session
- **Test:** `test_BC_2_21_029_state_resets_to_idle_after_completion`

### AC-191-007: Flow close discards an in-progress session without emission
(traces to BC-2.21.029 postcondition 10)
- Given `download_state == InProgress { .. }` and the flow closes
- When `on_flow_close` runs
- Then `download_state` is discarded along with the rest of `S7commFlowState` — an
  in-progress session that never completes before flow close never emits
- **Test:** `test_BC_2_21_029_flow_close_discards_in_progress_session`

### AC-191-008: Completed download session emits exactly one T0843-tagged Finding
(traces to BC-2.21.030 postcondition 1)
- Given `download_state` transitions from `InProgress { blocks_seen, block_type_hint }`
  to `Completed { .. }` and `self.all_findings.len() < MAX_S7COMM_FINDINGS`
- When the completion is processed
- Then exactly one `Finding` is pushed: `category: LateralMovement`,
  `verdict: Likely`, `confidence: High`, `mitre_techniques: vec!["T0843"]` (BC-2.21.031
  unconditionally appends `"T0889"` to this same vec; BC-2.21.032 conditionally appends
  `"T0821"`)
- A `blocks_seen == 0` session still emits (traces to BC-2.21.030 postcondition 3)
- A new completed session on the same flow emits a NEW finding — no one-shot guard
  beyond the state machine itself (traces to BC-2.21.030 postcondition 2)
- **Test:** `test_BC_2_21_030_completed_session_emits_t0843`,
  `test_BC_2_21_030_zero_block_session_still_emits`

### AC-191-009: T0889 unconditionally co-tagged on every download completion (path a)
(traces to BC-2.21.031 postcondition 1)
- Given path (a) — BC-2.21.030's completed-download precondition holds
- When the finding is constructed
- Then `"T0889"` is appended to the SAME `Finding` object BC-2.21.030 pushes — no
  separate `Finding` is created; category/verdict/confidence/summary/evidence remain as
  BC-2.21.030 defines them
- **Test:** `test_BC_2_21_031_t0889_co_tagged_on_download_completion`

### AC-191-010: T0889 fires standalone on bare BlockActivate/BlockDelete with no preceding download
(traces to BC-2.21.031 postcondition 2)
- Given `S7ClassicFunction::PlcControl(service)` where `service ∈ {BlockActivate,
  BlockDelete}`, path (a) did NOT fire for this same frame, and
  `self.all_findings.len() < MAX_S7COMM_FINDINGS`
- When the frame is processed
- Then exactly ONE `Finding` is pushed: `category: Persistence`, `verdict: Likely`,
  `confidence: Medium`, `mitre_techniques: vec!["T0889"]`
- No one-shot guard on path (b): each standalone activate/delete frame generates one
  finding (traces to BC-2.21.031 postcondition 3)
- **Test:** `test_BC_2_21_031_t0889_standalone_on_block_activate_delete`

### AC-191-011: T0821 co-tagged when block_type_hint is OrganizationBlock; suppressed when OtherBlockType; low-confidence-noted when Undeterminable
(traces to BC-2.21.032 postconditions 1-3)
- Given a completed download session with `block_type_hint == OrganizationBlock`
- When the finding is constructed
- Then `"T0821"` is appended to the same finding, with the evidence field noting the OB
  decode (traces to BC-2.21.032 postcondition 1)
- Given `block_type_hint == OtherBlockType(code)`
- Then T0821 is NOT appended — T0843/T0889 are unaffected (traces to BC-2.21.032
  postcondition 2)
- Given `block_type_hint == Undeterminable`
- Then `"T0821"` is appended but the shared `confidence` field is NOT downgraded from
  BC-2.21.030's `High`; instead the evidence string notes the reduced-confidence caveat
  in prose (traces to BC-2.21.032 postcondition 3)
- **Test:** `test_BC_2_21_032_t0821_gated_by_block_type_hint` (one case per hint variant)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `S7DownloadSessionState` enum | SS-21 per-flow state | `src/analyzer/s7comm.rs` | N/A (`Idle`, `InProgress`, `Completed`) |
| `S7BlockTypeHint` enum | SS-21 data model | `src/analyzer/s7comm.rs` | N/A (`OrganizationBlock`, `OtherBlockType`, `Undeterminable`) |
| `S7commFlowState.download_state` field | SS-21 per-flow state | `src/analyzer/s7comm.rs` | Mutable state |
| `const MAX_S7COMM_FINDINGS: usize = 10_000` | SS-21 constants | `src/analyzer/s7comm.rs` | N/A (first introduced here) |
| Download-session emission logic | SS-21 effectful shell | `src/analyzer/s7comm.rs` | Effectful |
| `MitreTactic::IcsLateralMovement`, `MitreTactic::IcsPersistence` (NEW variants) | SS-10 MITRE catalog | `src/mitre.rs` | N/A |
| `technique_info("T0843"/"T0889"/"T0821")` (NEW arms) | SS-10 MITRE catalog | `src/mitre.rs` | N/A |

Subsystem anchors:
- SS-21 owns the download-session state machine and emission logic per ARCH-INDEX.md
  §SS-21.
- SS-10 owns the MITRE catalog per ARCH-INDEX.md §SS-10 — this story performs the
  six-part atomic obligation for T0843/T0889/T0821 (VP-007 amendment).

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Download-session state transition logic | effectful-shell | Mutates `S7commFlowState.download_state`, pushes `Finding` objects |
| `S7DownloadSessionState`, `S7BlockTypeHint` | pure-core | Plain data types |
| `technique_info`, `SEEDED_TECHNIQUE_IDS`, `EMITTED_IDS` (mitre.rs) | pure-core | Static catalog lookups; no I/O |

## T0843/T0889/T0821 Six-Part Atomic Commit (ADR-014 Decision 5, VP-007)

**CRITICAL: all six changes below MUST land in a SINGLE git commit. No partial delivery.**

```
Six-part atomic for T0843/T0889/T0821 (ADR-014 Decision 5):
1. "T0843", "T0889", "T0821" added to SEEDED_TECHNIQUE_IDS array (29 -> 32 entries)
2. SEEDED_TECHNIQUE_ID_COUNT bumped 29 -> 32
3. technique_info("T0843") -> ("Program Download", MitreTactic::IcsLateralMovement)
   technique_info("T0889") -> ("Modify Program", MitreTactic::IcsPersistence)
   technique_info("T0821") -> ("Modify Controller Tasking", MitreTactic::IcsExecution)
4. MitreTactic gains two NEW variants: IcsLateralMovement (tactic_id "TA0109", Display
   "Lateral Movement (ICS)") and IcsPersistence (tactic_id "TA0110", Display
   "Persistence (ICS)") — added to the enum, its Display impl, its tactic_id() impl, and
   all_tactics_in_report_order(), in this same commit. T0821 requires NO enum change —
   it reuses the pre-existing MitreTactic::IcsExecution.
5. EMITTED_IDS array gains "T0843", "T0889", "T0821" entries
6. vp007_catalog_drift_guard #[test] passes at count=32; verify_all_seeded_ids_resolve
   passes at count=32; verify_all_emitted_ids_resolve Kani harness passes for all three
```

**Pitfall to avoid**: the codebase's `MitreTactic` enum already has an Enterprise
`LateralMovement` variant (TA0008) and an Enterprise `Persistence` variant — these are
DISTINCT from the new ICS-specific `IcsLateralMovement` (TA0109) and `IcsPersistence`
(TA0110) variants this story adds. T0843/T0889 MUST use the new `Ics*` variants, never
the pre-existing Enterprise ones.

## Tasks

- [ ] Define `pub enum S7DownloadSessionState { Idle, InProgress { blocks_seen: u32,
      block_type_hint: S7BlockTypeHint }, Completed { blocks_seen: u32, block_type_hint:
      S7BlockTypeHint } }`
- [ ] Define `pub enum S7BlockTypeHint { OrganizationBlock, OtherBlockType(String),
      Undeterminable }`
- [ ] Add `download_state: S7DownloadSessionState` field to `S7commFlowState`
- [ ] Implement the state-transition logic per BC-2.21.029's postconditions 3-10
- [ ] Define `const MAX_S7COMM_FINDINGS: usize = 10_000;` (mirrors DNP3/ENIP/IEC-104
      `MAX_FINDINGS`)
- [ ] Implement the T0843 base-finding emission on `Completed` transition (BC-2.21.030)
- [ ] Implement T0889's two paths: unconditional co-tag on download completion, and
      standalone emission on bare `BlockActivate`/`BlockDelete` (BC-2.21.031)
- [ ] Implement T0821's gated co-tag on `block_type_hint` (BC-2.21.032)
- [ ] `src/mitre.rs`: perform the six-part atomic T0843/T0889/T0821 commit exactly as
      specified above, in ONE commit
- [ ] Write unit tests: one per AC, named `test_BC_2_21_029_*` .. `test_BC_2_21_032_*`
- [ ] Write `test_mitre_t0843_t0889_t0821_six_part_atomic` — verifies
      `SEEDED_TECHNIQUE_IDS.len() == SEEDED_TECHNIQUE_ID_COUNT == 32`,
      `technique_info` resolves all three new IDs to the correct tactic, and
      `EMITTED_IDS` contains all three
- [ ] Extend `tests/fixtures/mk_s7comm_pcap.py` with a complete download-session sequence
      (`RequestDownload` -> `DownloadBlock` xN -> `DownloadEnded`) with an `"OB"`
      filename-style hint
- [ ] Verify `cargo test --all-targets` passes
- [ ] Verify `cargo test vp007_catalog_drift_guard` passes (count=32, `#[test]`, not Kani)
- [ ] Verify `cargo test verify_all_seeded_ids_resolve` passes at count=32
- [ ] Verify `cargo kani --harness verify_all_emitted_ids_resolve` passes for
      T0843/T0889/T0821 (full run deferred to STORY-194's VP-007 re-run; a scoped local
      run here is recommended but not blocking)
- [ ] Add a CHANGELOG entry under `[Unreleased] > Added` describing the three new MITRE
      techniques and the download-session correlation, before creating the PR

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.21.029 | `RequestDownload` decodes a filename hint of `"DB"` (Data Block, not OB) | `OtherBlockType("DB")` |
| EC-002 | BC-2.21.029 | Filename field absent or truncated | `Undeterminable` |
| EC-003 | BC-2.21.030 | `all_findings.len() == MAX_S7COMM_FINDINGS` already | No finding pushed for this session's completion (cap enforced) |
| EC-004 | BC-2.21.031 | `BlockActivate` frame IS the same frame that completes a download session | Path (a) fires (co-tag on the existing finding); path (b) does NOT also fire a second, separate finding for the same frame |
| EC-005 | BC-2.21.032 | Mutation test: T0821 emitted for an `OtherBlockType` hint (regression) | MUST be caught — this is the canonical mutation surviving-arm this story's tests must kill |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~6,800 |
| BC-2.21.029-032 (4 BCs, high density) | ~7,500 |
| ADR-014 Decision 5, s7comm-mitre-ics-tagging.md | ~7,000 |
| src/analyzer/s7comm.rs (from STORY-187-190) | ~8,000 |
| src/mitre.rs (existing) | ~5,000 |
| Test file delta + fixture generator extension | ~4,000 |
| **Total** | **~38,300** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~19%** |

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-188 | `S7ClassicFunction::RequestDownload`/`DownloadBlock`/`DownloadEnded`/`PlcControl(BlockActivate or BlockDelete)` classified per-frame | Per-frame classification exists; this story is the first to correlate across frames within a flow | The Download/Upload triad disjointness (STORY-188/VP-054) must hold — this story's state machine only reacts to `RequestDownload`/`DownloadBlock`/`DownloadEnded`, never to any Upload-triad classification |

This is the codebase's sixth MITRE six-part-atomic obligation (after IEC-104's T0881,
ENIP's T0858/T0816/T1693.001, DNP3's T0809/T0836) — follow the exact commit-atomicity
discipline those precedents established; a partial landing (e.g. `EMITTED_IDS` missing
one of the three IDs) fails the Kani drift-guard.

## Architecture Compliance Rules

Extracted from `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md`:
- **ADR-014 Decision 5**: T0843/T0889/T0821 are the three NEW MITRE techniques this
  feature seeds. Live-page tactic verification: T0843 = Lateral Movement (TA0109), T0889
  = Persistence (TA0110), T0821 = Execution (TA0104, reuses `IcsExecution`). Two new
  `MitreTactic` variants required for TA0109/TA0110 — no existing variant covers either.
  Version pin retained at `ics-attack-19.1` (no bump).
- Findings cap (`MAX_S7COMM_FINDINGS = 10_000`) mirrors the DNP3/ENIP/IEC-104 precedent
  and is introduced here, at the first story that emits findings, not deferred to
  dispatcher integration.
- Pure/effectful boundary: the state machine and emission logic are effectful (mutate
  flow state, push findings); `mitre.rs`'s `technique_info`/catalog lookups remain pure.

## Library & Framework Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | Enum state machine, `Vec<Finding>` |
| kani | Latest via `cargo kani` | `verify_all_emitted_ids_resolve` scoped local run |

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `src/analyzer/s7comm.rs` | MODIFY | Add `S7DownloadSessionState`, `S7BlockTypeHint`, `MAX_S7COMM_FINDINGS`, download-session state machine, T0843/T0889/T0821 emission logic |
| `src/mitre.rs` | MODIFY | Six-part atomic: `SEEDED_TECHNIQUE_IDS`/`SEEDED_TECHNIQUE_ID_COUNT` 29->32, two new `MitreTactic` variants, three `technique_info` arms, `EMITTED_IDS` extension |
| `tests/s7comm_analyzer_tests.rs` | MODIFY | Add BC-2.21.029-032 unit tests |
| `tests/mitre_tests.rs` (or equivalent existing MITRE test file) | MODIFY | Add the six-part atomic verification test for T0843/T0889/T0821 |
| `tests/fixtures/mk_s7comm_pcap.py` | MODIFY | Add complete download-session sequence with OB filename hint |

## Forbidden Dependencies

- PARTIAL T0843/T0889/T0821 registration is forbidden: if any of the six parts is
  missing at commit time, the Kani drift-guard will fail — this is the enforcement
  mechanism, not a soft guideline
- The new `IcsLateralMovement`/`IcsPersistence` `MitreTactic` variants MUST NOT be
  confused with the pre-existing Enterprise `LateralMovement`/`Persistence` variants

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-09-06 | story-writer | Initial authorship — download-session correlation state machine, T0843/T0889/T0821 six-part atomic MITRE registration, two new MitreTactic variants, AC-191-001..011. |
