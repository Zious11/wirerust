---
document_type: story
story_id: "STORY-018"
epic_id: "E-2"
version: "2.1"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.023.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.027.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.040.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.041.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.042.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.044.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.045.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.046.md
input-hash: "6b02e4d"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-015, STORY-016]
blocks: [STORY-021]
behavioral_contracts: [BC-2.04.023, BC-2.04.027, BC-2.04.040, BC-2.04.041, BC-2.04.042, BC-2.04.044, BC-2.04.045, BC-2.04.046]
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 10
target_module: reassembly
subsystems: [SS-04]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-RES-003
  - NFR-RES-004
  - NFR-RES-005
  - NFR-RES-009
  - NFR-RES-010
  - NFR-MNT-008
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-018`

# STORY-018: Resource Bounds — Depth Truncation, Out-of-Window Rejection, and Segment Limit Enforcement

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to enforce hard resource limits per direction — depth truncation when streams exceed `max_depth`, out-of-window rejection for segments beyond the receive window, segment-map cap at `max_segments_per_direction`, and accurate tracking of rejected segments via dedicated stats counters
- **So that** the engine cannot be memory-exhausted by adversarial or malformed captures and resource-consumption events are observable in the statistics output

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.041 | Depth Truncation: Segment Crossing max_depth is Truncated | Core truncation mechanics in insert_segment |
| BC-2.04.023 | Truncated Segment Emits Anomaly/Inconclusive/Low Finding | Engine-level finding generation for Truncated result |
| BC-2.04.027 | segments_depth_exceeded Tracks Fully-Rejected Segments After Depth Hit | Counter for post-truncation rejections |
| BC-2.04.042 | Segment Beyond max_receive_window Returns OutOfWindow | Out-of-window rejection and counter |
| BC-2.04.040 | Small-Segment Counter Increments Per Direction | Consecutive-run counter update rules |
| BC-2.04.044 | Segments Map Full: Non-Overlapping Insert Returns SegmentLimitReached | Non-overlap path cap enforcement |
| BC-2.04.045 | Segments Map Full: Overlapping Insert Returns SegmentLimitReached | Overlap path cap enforcement |
| BC-2.04.046 | Segments Map Fills Mid-Loop: Partial Insertion | Partial insertion when limit hit mid-loop |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.041 postcondition 1)
- When `reassembled_bytes + buffered_bytes + data.len() > max_depth` and `depth_exceeded == false`, `insert_segment` returns `InsertResult::Truncated`.
- **NOTE:** When `allowed == 0`, the result is `DepthExceeded` (see AC-002 NOTE and BC-2.04.041 EC-004) — AC-001's antecedent is reachable in that case but its consequent does not hold; AC-002's NOTE captures the operative boundary.
- **Test:** `test_BC_2_04_041_depth_truncation_returns_truncated()`

### AC-002 (traces to BC-2.04.041 postcondition 2-4)
- After a `Truncated` result, only `allowed = max_depth.saturating_sub(reassembled_bytes + buffered_bytes)` bytes are stored (not the full payload), and `buffered_bytes` increases by exactly `allowed`.
- **NOTE:** When `allowed == 0` (buffer exactly at `max_depth` at entry), the result is `DepthExceeded` (not `Truncated`); see BC-2.04.041 Description + EC-004.
- **Test:** `test_BC_2_04_041_truncated_stores_only_allowed_bytes()`

### AC-003 (traces to BC-2.04.041 postcondition 5 and invariant 1)
- After `Truncated`, `depth_exceeded == true` and all subsequent inserts for that direction return `InsertResult::DepthExceeded` (not `Truncated` again).
- **Test:** `test_BC_2_04_041_depth_exceeded_flag_set_after_truncated()`

### AC-004 (traces to BC-2.04.023 postcondition 1)
- The engine emits one `Anomaly/Inconclusive/Low` finding with `mitre_technique: None`, `summary: "Stream depth exceeded on flow <key>"`, and `evidence: ["Max depth N bytes reached"]` where N matches `config.max_depth`.
- **Test:** `test_BC_2_04_023_truncated_finding_emitted()`

### AC-005 (traces to BC-2.04.023 postcondition 2 and invariant 2)
- When `findings.len() >= MAX_FINDINGS` at the time of truncation, no finding is pushed and `stats.dropped_findings` increments instead.
- **Test:** `test_BC_2_04_023_truncated_finding_dropped_at_cap()`

### AC-006 (traces to BC-2.04.023 invariant 3)
- The truncated finding sets `source_ip: Some(packet.src_ip)` but `direction: None`.
- **Test:** `test_BC_2_04_023_truncated_finding_has_source_ip_no_direction()`

### AC-007 (traces to BC-2.04.027 v1.3 postcondition 1-2 + Description paths a/b/c)
- For each segment that returns `InsertResult::DepthExceeded` — whether via the pre-existing `depth_exceeded` flag (path a), the `remaining_depth == 0` check at `segment.rs:80-86` (path b, first-call sets flag during this call), or the `allowed == 0` check at `segment.rs:94-98` (path c, first-call sets flag during this call) — `stats.segments_depth_exceeded` increments by 1 and no bytes are stored.
- **NOTE:** Paths (b) and (c) are the first-call cases where the segment itself triggers the flag transition; see BC-2.04.027 v1.3 EC-005 for the canonical allowed==0 example.
- **Test:** `test_BC_2_04_027_depth_exceeded_counter_increments()`

### AC-008 (traces to BC-2.04.027 postcondition 4 and invariant 2)
- `DepthExceeded` segments do not change `total_memory` or `buffered_bytes`.
- **Test:** `test_BC_2_04_027_depth_exceeded_does_not_affect_memory()`

### AC-009 (traces to BC-2.04.027 edge case EC-004)
- Depth exceedance is per-direction: if the client-to-server direction exceeds `max_depth`, the server-to-client direction continues to accept segments normally.
- **Test:** `test_BC_2_04_027_depth_exceeded_is_per_direction()`

### AC-010 (traces to BC-2.04.042 postcondition 1 and invariant 1)
- When a segment's computed offset exceeds `base_offset.saturating_add(max_receive_window as u64)`, `insert_segment` returns `InsertResult::OutOfWindow` and no bytes are stored.
- **Test:** `test_BC_2_04_042_out_of_window_returns_out_of_window()`

### AC-011 (traces to BC-2.04.042 postcondition 4)
- `out_of_window_count` increments by 1 for each out-of-window segment.
- **Test:** `test_BC_2_04_042_out_of_window_count_increments()`

### AC-012 (traces to BC-2.04.042 edge case EC-001)
- A segment at exactly `base_offset + max_receive_window` (the boundary, not beyond) is accepted with `InsertResult::Inserted` (boundary is exclusive: `offset > window`, not `>=`).
- **Test:** `test_BC_2_04_042_segment_at_exact_window_boundary_is_inserted()`

### AC-013 (traces to BC-2.04.040 postcondition 1-2)
- After a segment with `payload.len() < small_segment_max_bytes`, `flow_dir.small_segment_run` increments by 1 (saturating). After a segment with `payload.len() >= small_segment_max_bytes`, `small_segment_run` resets to 0.
- **Test:** `test_BC_2_04_040_small_segment_run_increments_and_resets()`

### AC-014 (traces to BC-2.04.040 postcondition 3 and invariant 1)
- `small_segment_run` is tracked independently per direction (client-to-server and server-to-client have separate counters).
- **Test:** `test_BC_2_04_040_small_segment_run_is_per_direction()`

### AC-015 (traces to BC-2.04.040 invariant 1 and edge cases EC-004)
- Results `OutOfWindow`, `SegmentLimitReached`, `DepthExceeded`, and `IsnMissing` do NOT update `small_segment_run` (neither increment nor reset).
- **Test:** `test_BC_2_04_040_excluded_results_do_not_update_small_segment_run()`

### AC-016 (traces to BC-2.04.044 postcondition 1 and invariant 1)
- When `segments.len() >= max_segments` and the new segment does not overlap any existing entry, `insert_segment` returns `InsertResult::SegmentLimitReached` and `segments.len()` is unchanged.
- **Test:** `test_BC_2_04_044_segment_limit_non_overlapping_path()`

### AC-017 (traces to BC-2.04.044 edge case EC-001)
- When `segments.len() == max_segments - 1`, a new non-overlapping segment is inserted successfully (not rejected).
- **Test:** `test_BC_2_04_044_segment_one_below_limit_is_inserted()`

### AC-018 (traces to BC-2.04.045 v1.3 postcondition 3 and invariant 2 — mid-loop overlap_count claim)
- When `segments.len() < max_segments` at entry, the new segment overlaps existing entries, AND the BTreeMap fills to `max_segments` during the gap-insertion loop (such that one or more LATER gaps are dropped), `insert_segment` returns `SegmentLimitReached` and `overlap_count` is incremented by 1 (overlap detection at `segment.rs:143` fires before the mid-loop limit guard at `segment.rs:178-180`).
- **NOTE:** This AC scopes specifically to the `overlap_count` increment claim. The byte-accounting and partial-insertion semantics of the same scenario are covered by AC-019 (BC-2.04.046). The "zero gaps inserted before SegmentLimitReached fires" state is structurally unreachable — the early-exit guard at `segment.rs:70-72` prevents entry at `len == max_segments`, so at least one gap MUST be inserted before the mid-loop guard can fire.
- **NOTE:** For the EARLY-EXIT path (`segments.len() >= max_segments` at entry), see EC-007 — overlap detection does NOT run, so `overlap_count` is NOT incremented.
- **Test:** `test_BC_2_04_045_segment_limit_overlapping_path_increments_overlap_count()`

### AC-019 (traces to BC-2.04.046 postcondition 1-3 and invariant 1)
- When the BTreeMap fills to `max_segments` mid-way through a multi-gap insertion, `SegmentLimitReached` is returned, earlier gaps are in the map, later gaps are dropped, `buffered_bytes` has increased only by the bytes of the inserted gaps, and `overlap_count` increments by 1 (per BC-2.04.046 PC5 mid-loop overlap-detection).
- **Test:** `test_BC_2_04_046_segment_limit_partial_insertion_mid_loop()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| insert_segment depth check and truncation | src/reassembly/segment.rs:79-104 | pure-core |
| insert_segment out-of-window check | src/reassembly/segment.rs:63-67 | pure-core |
| insert_segment segment-limit check (non-overlap) | src/reassembly/segment.rs:70-72 | pure-core |
| insert_segment segment-limit check (overlap loop) | src/reassembly/segment.rs:175-199 | pure-core |
| generate_truncated_finding | src/reassembly/lifecycle.rs:120-136 | effectful-shell (mutates findings/stats) |
| small_segment_run update in insert_payload_segment | src/reassembly/mod.rs:385-399 | effectful-shell (mutates flow_dir) |
| DepthExceeded match arm | src/reassembly/mod.rs:416-418 | effectful-shell (mutates stats) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Segment for which `reassembled + buffered + data.len() == max_depth` (i.e., `total_after == max_depth`, equals not exceeds) | Inserted; no Truncated result; no finding (check at `segment.rs:93` is `>`, not `>=`) |
| EC-002 | Segment crosses depth limit by 1 byte | Truncated; 1 byte dropped; finding emitted |
| EC-003 | Two segments after depth hit | Both return DepthExceeded; segments_depth_exceeded=2 |
| EC-004 | Segment at exact receive window boundary | Inserted (boundary is exclusive: `>` not `>=`) |
| EC-005 | Segment 1 byte beyond receive window | OutOfWindow; out_of_window_count=1 |
| EC-006 | base_offset near u64::MAX | saturating_add prevents overflow; OutOfWindow returned correctly |
| EC-007 | segments.len() == max_segments, new segment is pure overlap (no gap) | SegmentLimitReached — early guard at `segment.rs:70-72` fires unconditionally at function entry, before overlap detection. See BC-2.04.045 v1.3 EC-002 for the corrected semantics; legacy Duplicate/ConflictingOverlap outcomes only occur when the map is BELOW capacity at entry. |
| EC-008 | Truncated at MAX_FINDINGS cap | dropped_findings++; no finding pushed |
| EC-009 | OutOfWindow result after 2 small segments | small_segment_run unchanged at 2 |
| EC-010 | DepthExceeded result after 3 small segments | small_segment_run unchanged at 3 |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/segment.rs (depth, window, segment-limit checks) | pure-core | No I/O; BTreeMap mutation only |
| src/reassembly/lifecycle.rs (generate_truncated_finding) | effectful-shell | Mutates self.findings and stats.dropped_findings |
| src/reassembly/mod.rs (DepthExceeded arm, small_segment_run update) | effectful-shell | Mutates stats and flow_dir fields |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,500 |
| BC files (8 BCs) | ~8,000 |
| src/reassembly/segment.rs (depth/window/limit sections ~lines 63-104, 175-199) | ~2,500 |
| src/reassembly/lifecycle.rs (generate_truncated_finding ~lines 120-136) | ~500 |
| src/reassembly/mod.rs (DepthExceeded arm, small_segment_run ~lines 385-418) | ~1,000 |
| Test files | ~5,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~21,500** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~10.7%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 19 ACs in `tests/reassembly_segment_tests.rs` and `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Test depth truncation: exactly-at-limit (Inserted), 1-byte-over (Truncated), post-depth (DepthExceeded)
5. [ ] Test per-direction independence: exceed depth on C2S, verify S2C still accepts
6. [ ] Test out-of-window boundary: exactly at `base+window` is Inserted; `base+window+1` is OutOfWindow
7. [ ] Test segment limit mid-loop partial insertion (BC-2.04.046 canonical test vector: max_segments=2, two gaps, first fills last slot)
8. [ ] Test small_segment_run exclusion list: OutOfWindow/DepthExceeded/SegmentLimitReached do not update run counter
9. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | Overlap detection tests in reassembly_segment_tests.rs | buffered_bytes is the authoritative per-direction byte counter | half-open interval check: `offset > window` not `>= window` for out-of-window |
| STORY-015 | BTreeMap key is ISN-relative u64 offset | flush_contiguous decrements buffered_bytes | wrapping_sub cast to u64 is the correct offset arithmetic |
| STORY-014 | ISN must be set before insert_segment fires | depth_exceeded is per-direction, not per-flow | ISN_MISSING_WARNED is process-wide |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| OutOfWindow check runs BEFORE segment-limit and depth checks | BC-2.04.042 invariant 1 | Code review: check order in segment.rs:63 before 70 before 79 |
| Segment-limit check runs BEFORE depth check | BC-2.04.044 invariant 1 | Code review: segment.rs:70 before 79 |
| `depth_exceeded` is permanent once set for a direction (never reset) | BC-2.04.041 invariant 1 | Test: verify third segment after Truncated also returns DepthExceeded |
| Truncated finding uses `source_ip` but NOT `direction` | BC-2.04.023 invariant 3 | Test: assert direction field is None in finding |
| `small_segment_run` uses `saturating_add(1)` (not wrapping add) | BC-2.04.040 invariant 3 | Code review: grep saturating_add in mod.rs:385-399 |
| Truncated finding evidence: exactly `["Max depth N bytes reached"]` | BC-2.04.023 postcondition 1 | Test: assert evidence vec == expected string with max_depth substituted |
| OutOfWindow boundary is exclusive: `offset > window`, not `>=` | BC-2.04.042 precondition 3 | Test: AC-012 boundary test |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | BTreeMap, AtomicBool, saturating_sub, saturating_add |
| proptest | from Cargo.toml | Property-based tests for memory invariants (if needed) |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/segment.rs` | verify (lines 63-104, 175-199) | Out-of-window, depth, and segment-limit checks |
| `src/reassembly/lifecycle.rs` | verify (lines 120-136) | generate_truncated_finding |
| `src/reassembly/mod.rs` | verify (lines 385-418) | small_segment_run update and DepthExceeded counter |
| `tests/reassembly_segment_tests.rs` | modify | Add AC-001 through AC-019 (segment-level tests) |
| `tests/reassembly_engine_tests.rs` | modify | Add engine-level truncation and finding tests |

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.1 | 2026-05-21 | story-writer | Initial draft |
| 1.2 | 2026-05-26 | story-writer | Wave 10 STORY-018 pass-1 fixes: F-002 (CRITICAL) — added NOTE to AC-002 disambiguating the allowed==0 case (see BC-2.04.041 v1.x Description + EC-004 + EC-005); F-006 (MED) — EC-001 row revised from ambiguous 'exactly at max_depth' to explicit 'total_after == max_depth' boundary (matches segment.rs:93 `>` check, not `>=`). DF-SIBLING-SWEEP-001 sweep performed. |
| 1.3 | 2026-05-26 | story-writer | Wave 10 STORY-018 pass-2 fixes (sibling-regression of pass-1 v1.3 BC fix that didn't propagate to story EC table): F-PASS2-001 (MED) — EC-007 rewritten to match BC-2.04.045 v1.3 EC-002 (SegmentLimitReached via early guard at segment.rs:70-72); O-PASS2-002 (LOW) — Task 7 parenthetical reference corrected from EC-010 to BC-2.04.046 canonical test vector. Sweep checklist expanded per DF-SIBLING-SWEEP-001 to grep story EC tables when downstream BC EC tables change. |
| 1.4 | 2026-05-26 | story-writer | Wave 10 STORY-018 pass-4 fixes: F-PASS4-001 (MED, exactly the W10-D6 process-gap pattern) — AC-018 rewritten to use mid-loop entry condition (`segments.len() < max_segments` at entry) matching BC-2.04.045 v1.3 PC3 explicit bifurcation; added NOTE referencing EC-007 for the early-exit path. O-PASS4-002 (LOW) — AC-019 augmented with overlap_count increment assertion per BC-2.04.046 PC5. DF-SIBLING-SWEEP-001 v2 refinement applied: sweep included AC text against revised BC PCs/INVs (not just EC tables) — W10-D6 codification target validated by this fix. |
| 1.5 | 2026-05-26 | story-writer | Wave 10 STORY-018 pass-5 fixes (sibling-regression of pass-4 v2-refined sweep gaps): F-PASS5-001 (MED) — dropped non-existent "EC-005" from AC-002 NOTE (BC-2.04.041 v1.3 EC table has only EC-001..EC-004); F-PASS5-002 (MED, within-story half) — AC-018 "no gap can be inserted" tail removed (structurally unreachable per early-guard analysis); AC-018 rescoped to "overlap_count claim only" with cross-ref to AC-019 for partial-insertion accounting; O-PASS5-001 (LOW) — AC-001 NOTE added mirroring AC-002 NOTE for allowed==0 boundary. DF-SIBLING-SWEEP-001 v3 refinement applied: sweep now includes (a) cross-reference target resolution verification, (b) implementation-reachability reasoning. W10-D7 codification target validated (BC-2.04.045 v1.3 PC2 "or no gaps fit at all" unreachable branch deferred to wave-gate). |
| 1.7 | 2026-05-28 | story-writer | W10-D8 propagation: BC-2.04.045 v1.4 removed "or no gaps fit at all" from PC2 (structurally unreachable — early-guard at segment.rs:70-72 prevents entry at len>=max_segments). Body sweep confirmed no live occurrence of the removed phrase (only historical changelog reference at v1.5 remains — permitted). BC-2.04.041 v1.4 added forensic data-loss note (PC7); BC-2.04.045 v1.4 PC3 overlap_count claim unchanged from STORY-018 v1.6 AC-018. No body AC text required edits. input-hash bumped 41b6ae2→b67d7fb. DF-SIBLING-SWEEP-001: grep confirmed no stale "or no gaps fit at all" in live body sections.
| 1.8 | 2026-05-29 | state-manager | input-hash corrected via canonical bin/compute-input-hash --update (prior value `b67d7fb` was hand-computed sha256 over sorted inputs-file list; tool uses MD5 over inputs-order file list). New value: `41b6ae2`. Also: BC-2.04.045 v1.5 (cap-04 citation + mid-loop anchor correction) applied this burst. |
| 2.0 | 2026-06-01 | story-writer | DF-SIBLING-SWEEP-001 story-body mod.rs re-anchor to HEAD e0451ef (Phase-5 anchor-class closure): Architecture Mapping small_segment_run 356-370→385-399; DepthExceeded arm 387-389→416-418; Token Budget range 356-389→385-418; Architecture Compliance Rule grep range 356-370→385-399; File Structure verify range 356-389→385-418. |
| 1.9 | 2026-05-29 | state-manager | status reconciled to completed per sprint-state.yaml (merge_commit f4963ba wave 10); F-DRIFT3B-001/PG-W16-002. |
| 1.6 | 2026-05-26 | story-writer | Wave 10 STORY-018 pass-6 fix: F-PASS6-001 (MED, recursive sweep-gap pattern — DF-SIBLING-SWEEP-001 v3 axis 'implementation-reachability reasoning' was applied to current-pass ACs but not to ACs whose BCs were modified in PRIOR passes this wave) — AC-007 antecedent rewritten to enumerate all 3 BC-2.04.027 v1.3 paths (a/b/c) for DepthExceeded; added NOTE pointing at first-call cases (paths b/c) and BC-2.04.027 v1.3 EC-005. DF-SIBLING-SWEEP-001 v4 refinement applied: sweep now broadens to ALL ACs whose anchored BCs were modified in any pass of any cycle since the AC was last reviewed. |
