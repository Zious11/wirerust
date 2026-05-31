---
document_type: story
story_id: "STORY-017"
epic_id: "E-2"
version: "1.4"
status: completed
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.018.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.019.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.020.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.021.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.022.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.037.md
input-hash: "256089c"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-015, STORY-016]
blocks: [STORY-021]
behavioral_contracts: [BC-2.04.018, BC-2.04.019, BC-2.04.020, BC-2.04.021, BC-2.04.022, BC-2.04.037]
verification_properties: [VP-002]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 10
target_module: reassembly
subsystems: [SS-04]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-017`

# STORY-017: Conflict and Evasion Detection — T1036 Findings and One-Shot Anomaly Latches

## Narrative
- **As a** forensic analyst investigating TCP evasion attacks
- **I want** the reassembly engine to emit a `T1036/Anomaly/Likely/High` finding on every conflicting byte overlap, emit one-shot threshold alerts for excessive overlaps, small segments, and out-of-window segments per direction, and ensure each alert fires at most once per (flow, direction) regardless of how many more anomalous segments arrive
- **So that** IDS-evasion and segment-splicing attacks are reliably detected with one finding per event, and alert flooding is bounded

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.018 | Conflicting Overlap Emits Anomaly/Likely/High Finding with MITRE T1036 | Per-event conflict finding |
| BC-2.04.019 | Excessive Overlaps Emit One-Shot T1036 Finding | Cumulative overlap threshold alert |
| BC-2.04.020 | Excessive Small Segments Emit One-Shot Finding | Small-segment consecutive run alert |
| BC-2.04.021 | Excessive Out-of-Window Segments Emit One-Shot Low Finding | OOW cumulative threshold alert |
| BC-2.04.022 | Per-Direction Alert Fires At Most Once Per Flow (Sticky Latch) | Latch-before-cap pattern |
| BC-2.04.037 | Same-Range Conflicting Overlap Returns ConflictingOverlap; Original Wins | insert_segment classification |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.037 postcondition 1)
- When a new segment's byte range is fully covered by existing segments AND at least one byte differs, `insert_segment` returns `InsertResult::ConflictingOverlap`.
- **Test:** `test_BC_2_04_037_conflicting_bytes_returns_conflicting_overlap()`

### AC-002 (traces to BC-2.04.037 postcondition 2-3)
- After `ConflictingOverlap`, `self.segments` is unchanged and `self.buffered_bytes` is unchanged (original bytes are preserved).
- **Test:** `test_BC_2_04_037_conflicting_overlap_original_bytes_preserved()`

### AC-003 (traces to BC-2.04.018 postcondition 2)
- When `InsertResult::ConflictingOverlap` is returned, the engine emits exactly one Finding with: category=Anomaly, verdict=Likely, confidence=High, `mitre_technique=Some("T1036")`, a summary containing the FlowKey display string, and `direction: None`.
- **Test:** `test_BC_2_04_018_conflicting_overlap_emits_t1036_finding()`

### AC-004 (traces to BC-2.04.018 postcondition 3)
- The original buffered bytes are NOT replaced; the conflicting new bytes are discarded. The finding is informational only.
- **Test:** `test_BC_2_04_018_conflicting_overlap_first_wins()`

### AC-005 (traces to BC-2.04.018 postcondition 4)
- Each `ConflictingOverlap` event produces one finding (not latched); successive conflicts each produce their own finding (subject to MAX_FINDINGS cap).
- **Test:** `test_BC_2_04_018_multiple_conflicts_each_produce_finding()`

### AC-006 (traces to BC-2.04.019 postcondition 1)
- When `flow_dir.overlap_count > config.overlap_alert_threshold` (strictly greater) AND `overlap_alert_fired == false`, the engine emits one Finding with: category=Anomaly, verdict=Likely, confidence=Medium, `mitre_technique=Some("T1036")`, and evidence containing `["Possible evasion attempt"]`.
- **Test:** `test_BC_2_04_019_overlap_threshold_emits_medium_t1036_finding()`

### AC-007 (traces to BC-2.04.019 postcondition 4)
- After the overlap threshold alert fires, no further overlap-threshold findings are emitted for that (flow, direction) pair, regardless of additional overlapping segments.
- **Test:** `test_BC_2_04_019_overlap_threshold_alert_fires_at_most_once()`

### AC-008 (traces to BC-2.04.019 edge case EC-001)
- When `overlap_count == threshold` exactly (not strictly greater), no alert fires.
- **Test:** `test_BC_2_04_019_overlap_count_at_threshold_does_not_alert()`

### AC-009 (traces to BC-2.04.020 postcondition 1-2)
- When `small_segment_run > config.small_segment_alert_threshold` AND `small_segment_alert_fired == false` AND neither endpoint port is in `small_segment_ignore_ports`, the engine emits one Finding with: category=Anomaly, verdict=Inconclusive, confidence=Medium, `mitre_technique=None`, and evidence containing `["Long unbroken run of undersized TCP segments; possible segmentation-based IDS evasion"]`.
- **Test:** `test_BC_2_04_020_small_segment_run_emits_finding()`

### AC-010 (traces to BC-2.04.020 invariant 2)
- If EITHER endpoint port is in `small_segment_ignore_ports`, no small-segment alert is emitted for that flow regardless of run length.
- **Test:** `test_BC_2_04_020_port_exempt_flow_never_alerts()`

### AC-011 (traces to BC-2.04.021 postcondition 1-2)
- When `out_of_window_count > config.out_of_window_alert_threshold` AND `out_of_window_alert_fired == false`, the engine emits one Finding with: category=Anomaly, verdict=Inconclusive, confidence=Low, `mitre_technique=None`, and evidence containing the `max_receive_window` value.
- **Test:** `test_BC_2_04_021_out_of_window_threshold_emits_finding()`

### AC-012 (traces to BC-2.04.021 invariant 3)
- The evidence string format for the OOW alert is exactly: `"max_receive_window={window} bytes; possible misconfiguration, evasion, or capture corruption"`.
- **Test:** `test_BC_2_04_021_oow_evidence_string_format()`

### AC-013 (traces to BC-2.04.022 postcondition 1)
- The sticky latch (`overlap_alert_fired`, `small_segment_alert_fired`, `out_of_window_alert_fired`) is set to `true` BEFORE the MAX_FINDINGS cap check. Even if the cap suppresses the finding, the latch is set.
- **Test:** `test_BC_2_04_022_latch_fires_before_cap_check()`

### AC-014 (traces to BC-2.04.022 postcondition 3)
- Once a latch is set for a (flow, direction) pair, subsequent threshold crossings for that alert type are no-ops (no finding emitted, no `dropped_findings` increment from re-evaluation).
- **Test:** `test_BC_2_04_022_latch_prevents_re_evaluation()`

### AC-015 (traces to BC-2.04.022 invariant 3)
- The maximum possible threshold findings for a single bidirectional flow is 6 (3 alert types × 2 directions); both directions can each fire all three alerts independently.
- **Test:** `test_BC_2_04_022_max_6_threshold_findings_per_flow()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| insert_segment (ConflictingOverlap path) | src/reassembly/segment.rs | pure-core |
| generate_conflicting_overlap_finding | src/reassembly/lifecycle.rs | effectful-shell (mutates findings) |
| check_anomaly_thresholds | src/reassembly/mod.rs | effectful-shell (mutates latch + findings) |
| FlowDirection alert latch fields | src/reassembly/flow.rs | pure-core (data) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | ConflictingOverlap when findings.len() == MAX_FINDINGS | Finding silently dropped; dropped_findings++ |
| EC-002 | ConflictingOverlap immediately after another | Second finding emitted (not latched) |
| EC-003 | overlap_count == threshold exactly | No threshold alert (strictly greater) |
| EC-004 | overlap_count == threshold + 1 | Alert fires |
| EC-005 | Small-segment run reset by normal-sized segment | No alert after reset |
| EC-006 | Port 23 (telnet) in ignore list; 1000 small segments | No alert |
| EC-007 | OOW alert fires when findings cap is full | Latch set; dropped_findings++ |
| EC-008 | ClientToServer latch set; ServerToClient still unlocked | S2C can fire independently |
| EC-009 | Duplicate overlap result | overlap_count++ but no ConflictingOverlap finding |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/segment.rs | pure-core | Returns classification without emitting findings |
| src/reassembly/lifecycle.rs (generate_conflicting_overlap_finding) | effectful-shell | Mutates self.findings, self.stats |
| src/reassembly/mod.rs (check_anomaly_thresholds) | effectful-shell | Mutates alert latches, self.findings |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (6 BCs) | ~6,000 |
| src/reassembly/mod.rs (check_anomaly_thresholds ~lines 420-512) | ~2,000 |
| src/reassembly/lifecycle.rs (generate_conflicting_overlap_finding ~lines 96-120) | ~800 |
| src/reassembly/segment.rs (ConflictingOverlap ~lines 142-154) | ~500 |
| Test files | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~17,300** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 15 ACs in `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Test latch-before-cap (AC-013): fill findings to MAX_FINDINGS, then trigger threshold; assert latch=true and dropped_findings==1
5. [ ] Test per-direction independence (AC-015): trigger all 3 alerts in both directions; assert exactly 6 findings
6. [ ] Test port-exempt flows (AC-010): configure ignore list with test port; send many small segments
7. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | overlap_count increments on Duplicate too | buffered_bytes debug_assert verifies consistency | Half-open interval determines adjacency (not overlap) |
| STORY-015 | flush_contiguous delivers segments per segment (not merged) | Base offset is monotonic | direction() returns ServerToClient when initiator=None |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Latch set BEFORE cap check (LESSON-P1.01) | BC-2.04.022 PC-1/INV-2 | Code review: latch assignment before `if findings.len() < MAX_FINDINGS` |
| ConflictingOverlap finding: confidence=High (not Medium) | BC-2.04.018 postcondition 2 | Test: assert finding.confidence == High |
| Overlap threshold finding: confidence=Medium | BC-2.04.019 postcondition 2 | Test: assert finding.confidence == Medium |
| OOW threshold finding: confidence=Low | BC-2.04.021 postcondition 2 | Test: assert finding.confidence == Low |
| Small-segment alert: no MITRE technique | BC-2.04.020 postcondition 2 | Test: assert finding.mitre_technique == None |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | AtomicBool, saturating arithmetic |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/mod.rs` | verify (lines 379-512) | InsertResult match arms + check_anomaly_thresholds |
| `src/reassembly/lifecycle.rs` | verify (lines 96-136) | generate_conflicting_overlap_finding, generate_truncated_finding |
| `src/reassembly/segment.rs` | verify (lines 142-154) | ConflictingOverlap/Duplicate classification |
| `src/reassembly/flow.rs` | verify (lines 86-108) | Alert latch fields on FlowDirection |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-015 |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.2 | 2026-05-28 | story-writer | W10-D2: Architecture Compliance Rule "Latch set BEFORE cap check" corrected from "BC-2.04.022 invariant 1" → "BC-2.04.022 PC-1/INV-2" (INV-2 is the correct invariant for the monotonic latch; PC-1 is the postcondition establishing latch-before-cap ordering; "invariant 1" was stale). W10-D11: AC-006 evidence string pinned to `["Possible evasion attempt"]`; AC-009 evidence string pinned to `["Long unbroken run of undersized TCP segments; possible segmentation-based IDS evasion"]` — anchoring test-writer assertions added this wave. W10-D14: AC-003 extended to assert `direction: None` on ConflictingOverlap finding, matching test-writer assertion `assert_eq!(f.direction, None)`. BC-2.04.019 v1.4 anchor fix (mod.rs:430-450) already cited in AC-006 via overlap block reference. input-hash bumped 9ddb8b7→7a32070. DF-SIBLING-SWEEP-001: full body sweep performed; no stale BC-2.04.022 invariant-1 or evidence-string occurrences remain.
| 1.3 | 2026-05-29 | state-manager | input-hash corrected via canonical bin/compute-input-hash --update (prior value `7a32070` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `9ddb8b7`. |
| 1.4 | 2026-05-29 | state-manager | status reconciled to completed per sprint-state.yaml (merge_commit ced10aa wave 10); F-DRIFT3B-001/PG-W16-002. |
| 1.1 | 2026-05-21 | story-writer | Initial story version |
