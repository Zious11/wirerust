---
pass: 3
scope: story-decomposition
date: 2026-05-21
verdict: NOT_CONVERGED
findings: "0C/1H/1M/2L/2N"
total_findings: 6
convergence_counter: 0/3
---

# Story Adversary Pass 3 — Story Decomposition Review

**Date:** 2026-05-21
**Scope:** Story decomposition (STORY-*.md files) + holdout scenarios (HS-*.md, HS-INDEX.md) +
dependency graph (dependency-graph.md) + STORY-INDEX.md
**Verdict:** NOT_CONVERGED — 0C/1H/1M/2L/2N (6 findings total)
**Convergence counter:** 0/3 (Pass 4 next)

## Pass-2 Regressions Check

All Pass-2 HIGH and MEDIUM findings (H-1 `--format json` flag, M-1 depends_on/blocks symmetry,
M-2 VP-INDEX Verified BCs) were verified clean. No regressions on any prior finding.

## Finding Summary

### HIGH

**F-1 — STORY-INDEX epic-points table self-contradiction**
The Stories-by-Epic summary table in `STORY-INDEX.md` listed per-epic point totals that did not
sum to the claimed 282-point total. E-2 showed 69 points (correct pre-Pass-2 count; 4 points were
added when M-1 remediation introduced corrected story splits) and E-5 showed 55 points (similarly
stale). The row sum with stale values resolved to a number inconsistent with the 282-point total
stated in STATE.md, the dependency graph, and STORY-INDEX.md header.
**Remediated:** E-2 updated 69→73 and E-5 updated 55→58. All rows now sum to 282, consistent
with the canonical story-point total.

### MEDIUM

**F-2 — STORY-013 transitive-dependency inconsistency across 5 artifacts**
STORY-013 listed `depends_on: [STORY-001, STORY-012]` in its frontmatter, recording both a
direct dependency on STORY-012 and a transitive dependency on STORY-001 (because STORY-012
already depends on STORY-001). The factory dependency model uses direct edges only; transitive
edges must not be materialized in story frontmatter, the sprint-state.yaml `blocked_by` field,
or the wave-schedule. This inconsistency propagated across 5 artifacts:
  1. `STORY-013.md` frontmatter `depends_on`
  2. `stories/sprint-state.yaml` `blocked_by` list for STORY-013
  3. `cycles/v0.1.0-greenfield-spec/wave-schedule.md` Blocked By column for STORY-013
  4. `STORY-001.md` `blocks` field (listed STORY-013 transitively)
  5. `stories/STORY-INDEX.md` Dependencies column for STORY-013

**Remediated:** All 5 artifacts corrected to direct-edges-only:
  - `STORY-013.md` `depends_on: [STORY-012]` (STORY-001 removed)
  - `sprint-state.yaml` `blocked_by: [STORY-012]` (STORY-001 removed)
  - `wave-schedule.md` Blocked By → `STORY-012` (STORY-001 removed)
  - `STORY-001.md` `blocks:` — STORY-013 removed (not a direct downstream of STORY-001)
  - `STORY-INDEX.md` Dependencies column → `STORY-012` only

### LOW

**F-3 — STORY-044 template drift (INVALID FINDING)**
Initial review flagged STORY-044 as having drift from story template conventions. On closer
inspection, the adversary had compared STORY-044 against E-4 sibling stories that themselves
were authored in a slightly earlier template revision. STORY-044 matches its E-4 peers in all
structural respects; no template drift exists relative to the correct comparison baseline.
**Disposition:** INVALID FINDING — no change needed. The finding arose from comparing against
wrong-epic siblings during initial sweep; the correct intra-epic comparison (E-4 peers) shows
no deviation.

**F-4 — HS-094 filler BC linkage**
`HS-094-cli-overlap-threshold-range-enforced.md` listed `BC-2.12.004` in its `behavioral_contracts`
frontmatter array and in its linkage table. BC-2.12.004 governs `--max-packet-size`, which is
not exercised by the HS-094 threshold-range scenario. The linkage was a filler citation carried
over from initial scenario authoring rather than a genuine behavioral reference.
**Remediated:** BC-2.12.004 removed from both the `behavioral_contracts` frontmatter array and
the linkage table in `HS-094-cli-overlap-threshold-range-enforced.md`.

### NITPICK

**N-1 — Process gap: story-point totals in STORY-INDEX summary table not machine-validated**
F-1 arose because the per-epic point totals in STORY-INDEX.md are authored manually and not
computed from the individual STORY-NNN.md `points:` frontmatter values. No automated check
asserts that the summary table is internally consistent or matches the story-file ground truth.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story content
defect.

**N-2 — Process gap: transitive-edge materialization not enforced**
F-2 arose because there is no validator asserting that `depends_on` and `blocked_by` fields
contain only direct edges. Transitive edges silently propagate across story files, sprint-state,
and wave-schedule without detection.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story or
dependency-graph defect.

## Remediation Status

| Finding | Severity | Remediated? |
|---------|----------|-------------|
| F-1 | HIGH | YES — E-2 69→73, E-5 55→58; STORY-INDEX epic-points rows sum to 282 |
| F-2 | MEDIUM | YES — direct-edges-only enforced across 5 artifacts (STORY-013, STORY-001, sprint-state.yaml, wave-schedule.md, STORY-INDEX.md) |
| F-3 | LOW | INVALID — adversary compared against wrong-epic siblings; STORY-044 already matches E-4 peers; no change made |
| F-4 | LOW | YES — BC-2.12.004 removed from HS-094 frontmatter and linkage table |
| N-1 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-2 | NITPICK | DEFERRED — cycle-close process-gap codification |

**All valid blocking findings (F-1 HIGH, F-2 MEDIUM, F-4 LOW) remediated. F-3 invalid (no
change needed). 2 process-gap NITPICKs deferred.**
Pass 4 may be dispatched.

## Finding Trajectory Across Story-Review Passes

| Pass | Findings | Severity Breakdown | Verdict |
|------|----------|--------------------|---------|
| 1 | 11 | 1C/3H/3M/2L/2N | NOT_CONVERGED |
| 2 | 7 | 0C/1H/2M/2L/2N | NOT_CONVERGED |
| 3 | 6 | 0C/1H/1M/2L/2N | NOT_CONVERGED (1 invalid) |

Trajectory showing decay in blocking-severity findings: CRITICAL eliminated after pass 1;
HIGH count decaying 3→1→1 (with Pass 3 H reduced to 0 valid unresolved); MEDIUM decaying
3→2→1. Convergence is trending; Pass 4 next.
