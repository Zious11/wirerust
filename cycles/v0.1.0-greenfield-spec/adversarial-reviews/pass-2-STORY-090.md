---
document_type: adversarial-review-pass
story: STORY-090
pass: 2
round: 2
cycle: v0.1.0-greenfield-spec
perimeter: 1 (per-story)
mode: per-story, fresh-context
target: tests/summary_story_090_tests.rs vs src/summary.rs
verdict: FINDINGS_REMAIN
timestamp: 2026-05-31T00:00:00Z
---

# Adversarial Review — STORY-090 Pass 2 (Round 2, post-round-1 remediation)

**Story:** STORY-090 — Summary Data Model — ingest, Service Hints, unique_hosts, Serialization
(BC-2.12.018..021; E-9; 5pts; library module `pub mod summary`).
**Mode:** Brownfield-formalization. ZERO src changes.
**Target:** `tests/summary_story_090_tests.rs` — 18 tests (13 AC + 5 EC) against `src/summary.rs`.
**Context:** Post-round-1 remediation — BC mapping re-anchored to canonical; AC-003/004 names renamed.

## Summary

Pass 2 revealed one residual cross-suite collision: AC-012 test name collided with a test in
`tests/reporter_tests.rs` (not `summary_tests.rs` — a DIFFERENT sibling file than Pass 1 had
caught). Additionally EC-003 was mis-categorized as an edge case of BC-2.12.018 but its actual
behavioral anchor was `pc4` (Serialization); retargeted. A full cross-suite uniqueness sweep
across the entire test corpus (18 summary_story_090 names vs all other test files) confirmed
these were the only remaining collisions.

## Findings

| ID | Sev | Category | One-line |
|----|-----|----------|----------|
| ADV-P02-S090-MED-001 | MED | traceability | AC-012 test name collides with reporter_tests.rs (cross-suite, different file than P1 caught) |
| ADV-P02-S090-MED-002 | MED | traceability | EC-003 anchored to BC-2.12.018 (ingest); actual behavioral target is pc4 (Serialization BC-2.12.021) |

## Verdict

FINDINGS_REMAIN — remediation required before Pass 3.

**Round-2 remediation scope:** Rename AC-012 test to eliminate reporter_tests.rs collision;
retarget EC-003 to pc4/BC-2.12.021; run full 18-name cross-suite uniqueness sweep to confirm
zero remaining collisions across ALL test files.

**Key lesson crystallized (round 2):** Cross-suite uniqueness sweep must cover ALL sibling test
files, not only the most obvious neighbors. Per DF-AC-TEST-NAME-SYNC-001 v2, the sweep scope
is the entire corpus — not just summary_tests.rs or the story's own file.
