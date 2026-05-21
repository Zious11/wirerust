---
pass: 1
scope: story-decomposition
date: 2026-05-21
verdict: NOT_CONVERGED
findings: "1C/3H/3M/2L/2N"
total_findings: 11
convergence_counter: 0/3
---

# Story Adversary Pass 1 — Story Decomposition Review

**Date:** 2026-05-21
**Scope:** Story decomposition (48 STORY-*.md files) + holdout scenarios (HS-INDEX.md) +
dependency graph (dependency-graph.md)
**Verdict:** NOT_CONVERGED — 1C/3H/3M/2L/2N (11 findings total)
**Convergence counter:** 0/3 (Pass 2 next)

## Finding Summary

### CRITICAL

**C-1 — VP-to-Stories matrix mis-anchored (dependency-graph.md)**
The VP-to-Stories cross-reference matrix in `dependency-graph.md` had 11 of 12 rows with
incorrect or missing story references; the full 20-VP x 48-story matrix was absent.
**Remediated:** Matrix regenerated with 20 rows (one per VP); `HS-INDEX.md` added to
`traces_to` for VP-INDEX.

### HIGH

**H-1 — 17 stories had incorrect verification_properties assignments**
`verification_properties` arrays were computed at story-create time from keyword heuristics
rather than ground-truth VP-INDEX.md traceability. 17 of 48 stories had stale, over-assigned,
or under-assigned VP lists.
**Remediated:** All 48 story `verification_properties` fields recomputed against VP-INDEX.md;
17 changed.

**H-2 — HS-INDEX subtotals wrong (By-Epic rollup)**
The By-Epic rollup table in `holdout-scenarios/HS-INDEX.md` had arithmetic errors; subtotals
did not sum to 100.
**Remediated:** By-Epic rollup corrected; all subtotals now sum to 100.

**H-3 — 10 stories missing the `## Behavioral Contracts` body table**
STORY-066, STORY-069, STORY-070, STORY-071, STORY-086, STORY-087, STORY-088, STORY-089,
STORY-090, STORY-096 were missing the required `## Behavioral Contracts` body section with
the inline table of BC IDs and descriptions.
**Remediated:** `## Behavioral Contracts` body tables added to all 10 stories.

### MEDIUM

**M-1 — `depends_on` not back-populated (8 stories)**
Stories STORY-011, STORY-031, STORY-041, STORY-051, STORY-066, STORY-076, STORY-086,
STORY-096 had empty `depends_on` fields despite documented upstream dependencies in the
dependency graph edges.
**Remediated:** `depends_on` back-populated from the canonical edge list for all 8 stories.

**M-2 — `blocks` not back-populated (8 stories)**
Same 8 stories had empty `blocks` fields despite being upstream blockers for downstream stories.
**Remediated:** `blocks` back-populated from the canonical edge list for all 8 stories.

**M-3 — STORY-013 AC-008 does not enumerate all FlowState transitions (untestable)**
AC-008 in STORY-013 referenced FlowState transitions without enumerating them, making the
acceptance criterion untestable.
**Remediated:** AC-008 now explicitly enumerates all 9 FlowState transitions.

### LOW

**L-1 — VP-INDEX not listed in `traces_to` for dependency-graph.md**
`dependency-graph.md` frontmatter was missing `VP-INDEX.md` in `traces_to`, making the VP
cross-reference invisible to traceability tooling.
**Remediated:** `VP-INDEX.md` added to `traces_to` in `dependency-graph.md`.

**L-2 — NUL bytes in spec files (already resolved)**
NUL bytes were found in STORY-070 and STORY-076 during the decomposition-gate remediation
burst (same class as P5-PG). This finding was already resolved before Pass 1 was dispatched;
recorded here for completeness and to close the finding class.
**Status:** ALREADY REMEDIATED (decomposition-gate remediation burst, 2026-05-21).

### NITPICK

**N-1 — Process gap: story-writer workflow lacks template-completeness gate**
The story-writer sub-agent has no automated check that all required sections (including
`## Behavioral Contracts` body table and `depends_on`/`blocks` arrays) are non-empty before
committing. This allowed H-3, M-1, and M-2 to survive to adversarial review.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story defect.

**N-2 — Process gap: story-writer lacks VP-anchoring validation**
VP assignments (`verification_properties`) were derived from heuristics rather than from
VP-INDEX.md ground truth, causing 17 incorrect assignments (H-1). No workflow step validates
VP correctness against the index.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story defect.

**N-3 — Process gap: decomposition-consistency-audit check matrix lacks VP-traceability
and HS-subtotal-arithmetic axes**
The decomposition-consistency-audit (Step F) did not include a VP-to-stories traceability
check (would have caught C-1) or an HS subtotal arithmetic check (would have caught H-2).
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story defect.

## Remediation Status

| Finding | Severity | Remediated? |
|---------|----------|-------------|
| C-1 | CRITICAL | YES — dependency-graph.md VP matrix regenerated |
| H-1 | HIGH | YES — all 48 stories recomputed |
| H-2 | HIGH | YES — HS-INDEX subtotals corrected |
| H-3 | HIGH | YES — 10 stories received BC body tables |
| M-1 | MEDIUM | YES — depends_on back-populated for 8 stories |
| M-2 | MEDIUM | YES — blocks back-populated for 8 stories |
| M-3 | MEDIUM | YES — STORY-013 AC-008 enumerates 9 transitions |
| L-1 | LOW | YES — dependency-graph.md traces_to updated |
| L-2 | LOW | ALREADY RESOLVED (pre-Pass-1) |
| N-1 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-2 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-3 | NITPICK | DEFERRED — cycle-close process-gap codification |

**All blocking findings (C-1, H-1, H-2, H-3, M-1, M-2, M-3, L-1) remediated.**
Pass 2 may be dispatched.

## Process-Gap Follow-Ups (cycle-close codification required)

1. **story-writer template-completeness gate** — add a check that all required sections
   (`## Behavioral Contracts` table, `depends_on`, `blocks`) are non-empty before commit.
2. **story-writer VP-anchoring validation** — validate `verification_properties` against
   VP-INDEX.md ground truth, not keyword heuristics.
3. **decomposition-consistency-audit axes** — extend the Step F audit matrix to include
   VP-traceability correctness and HS-subtotal arithmetic.
