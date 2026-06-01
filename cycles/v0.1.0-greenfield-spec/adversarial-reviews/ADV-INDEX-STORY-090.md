---
document_type: adversarial-review-index
story: STORY-090
cycle: v0.1.0-greenfield-spec
perimeter: 1 (per-story)
target: implementation (tests/summary_story_090_tests.rs vs src/summary.rs)
status: CONVERGED
timestamp: 2026-05-31T23:59:00Z
---

# Adversarial Review Index — STORY-090

**Story:** STORY-090 — Summary Data Model — ingest, Service Hints, unique_hosts, Serialization
(BC-2.12.018..021; E-9; 5pts).
**Mode:** Implementation review, Perimeter 1 (per-story), brownfield-formalization (zero-src).
**Target:** `tests/summary_story_090_tests.rs` (18 tests: 13 AC + 5 EC) against `src/summary.rs`
library module (`pub mod summary`).

## Convergence Summary

Three adversarial passes across two remediation rounds. Test LOGIC was strong throughout
(full mutation matrix caught in each round); convergence blockers were exclusively
traceability/anchoring defects resolved by round 2.

## Passes

| Pass | File | CRIT | HIGH | MED | LOW | New | Novelty | Verdict |
|------|------|------|------|-----|-----|-----|---------|---------|
| 1 (R1) | pass-1-STORY-090.md | 0 | 0 | 3 | 1 | 4 | 1.00 | FINDINGS_REMAIN |
| 2 (R2) | pass-2-STORY-090.md | 0 | 0 | 2 | 0 | 2 | 1.00 | FINDINGS_REMAIN |
| 3 (R3) | pass-3-STORY-090.md | 0 | 0 | 0 | 0 | 0 | 0.00 | **CLEAN** (fresh-context) |

**Trajectory (new findings):** 4 → 2 → 0 (monotonically non-increasing; clean at pass 3).

## Round 1 Findings (Remediated)

| ID | Sev | Category | One-line | Status |
|----|-----|----------|----------|--------|
| ADV-P01-S090-MED-001 | MED | traceability | BC mapping permuted — tests anchor to wrong BC IDs | REMEDIATED — re-anchored to canonical BC-2.12.018..021 |
| ADV-P01-S090-MED-002 | MED | traceability | AC-003 name collides with summary_tests.rs | REMEDIATED — renamed to unique cross-suite name |
| ADV-P01-S090-MED-003 | MED | traceability | AC-004 name collides with summary_tests.rs | REMEDIATED — renamed to unique cross-suite name |
| ADV-P01-S090-LOW-001 | LOW | spec-fidelity | Story header BC list omits BC-2.12.021 | REMEDIATED — header updated to BC-2.12.018..021 |

## Round 2 Findings (Remediated)

| ID | Sev | Category | One-line | Status |
|----|-----|----------|----------|--------|
| ADV-P02-S090-MED-001 | MED | traceability | AC-012 name collides with reporter_tests.rs (different file than R1) | REMEDIATED — renamed; full 18-name cross-suite sweep confirmed zero remaining |
| ADV-P02-S090-MED-002 | MED | traceability | EC-003 mis-anchored to BC-2.12.018; actual target is pc4/BC-2.12.021 | REMEDIATED — retargeted to Serialization BC |

## Mutation-Resistance Ground Truth

All 18 tests (13 AC + 5 EC) were verified against a full mutation matrix covering
BC-2.12.018 (ingest), BC-2.12.019 (service hints), BC-2.12.020 (unique_hosts),
and BC-2.12.021 (serialization). Zero survivors across all mutation classes.
Brownfield zero-src invariant preserved throughout.

## Key Process Observations

1. **Test logic was strong before remediation.** All mutations were killed from pass 1
   onward. Both remediation rounds addressed exclusively traceability/anchoring
   defects (wrong BC citations, name collisions), not behavioral coverage gaps.

2. **Two remediation rounds required for different-file collisions.** Round 1 caught
   summary_tests.rs collisions; round 2 caught reporter_tests.rs collisions. This
   demonstrates that cross-suite sweep scope must be corpus-wide (ALL test files),
   not just the most obvious sibling file. Codified as W27.L3 lesson.

3. **Full 18-name cross-suite uniqueness sweep is mandatory.** After round 2,
   a sweep across ALL test files confirmed zero remaining collisions before pass 3
   dispatch — this is the correct pre-pass verification protocol per
   DF-AC-TEST-NAME-SYNC-001 v2 extended scope.

4. **Library-module stories converge cleanly on direct unit tests.** STORY-090
   used direct unit/integration tests against `src/summary.rs` (not assert_cmd),
   which is the correct pattern for library modules (vs STORY-088/089 which used
   assert_cmd for main.rs). Codified as W27.L2 lesson.

## Convergence Status

**CONVERGED.** Three passes; 3-pass clean at pass 3. BC-5.39.001 ACHIEVED.

- Pass 1 (R1): FINDINGS_REMAIN — 4 findings (traceability/anchoring)
- Pass 2 (R2): FINDINGS_REMAIN — 2 findings (residual collision + EC retarget)
- Pass 3 (R3): **CLEAN** — zero findings, zero survivors

Single-story wave (Wave 27) → per-story convergence == wave-level convergence.
**Wave 27: CLOSED/CONVERGED. Phase 3: COMPLETE (48/48 stories, 27/27 waves).**
