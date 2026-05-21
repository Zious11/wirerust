---
pass: 5
scope: story-decomposition
date: 2026-05-21
verdict: NOT_CONVERGED
findings: "0C/1H/1M/3L/2N"
total_findings: 7
convergence_counter: 0/3
---

# Story Adversary Pass 5 — Story Decomposition Review

**Date:** 2026-05-21
**Scope:** Story decomposition (STORY-*.md files) + holdout scenarios (HS-001..100.md, HS-INDEX.md)
+ dependency graph (dependency-graph.md) + STORY-INDEX.md + BC-2.04.039.md
**Verdict:** NOT_CONVERGED — 0C/1H/1M/3L/2N (7 findings total; 2 blocking HIGH/MEDIUM findings
all remediated)
**Convergence counter:** 0/3 (streak does not advance; Pass 6 next)

## Pass-4 Regressions Check

All Pass-4 blocking findings (H-1 VP-018 trace annotation, H-2 BC-INDEX arithmetic
self-contradiction, H-3 STORY-086 provenance note, M-1 HS-051..100 concrete inputs, M-2
HS-INDEX Count column, M-3 STORY-088 AC-010, M-4 STORY-096 tdd_mode + Red Gate, M-5 STORY-003
cargo-fuzz AC) verified clean. No regressions on any prior finding.

## Adversary Structural Backbone Verification

The adversary performed an explicit structural backbone audit before raising findings. The
following dimensions were verified as fully clean:

- **VP anchoring:** All VP trace annotations present on AC rows where a VP is the primary
  behavioral-contract witness. H-1 from Pass 4 (missing VP-018 annotation) confirmed corrected
  and no new gaps found.
- **BC traceability:** All 217 BCs traced to at least one story in STORY-INDEX.md. No
  untraced BCs.
- **Dependency edges:** 77 edges in dependency-graph.md; graph verified acyclic; wave
  assignments consistent with graph topology.
- **Wave computation:** All 48 stories assigned to waves 1–27; wave membership consistent
  with sprint-state.yaml.
- **AC quality:** Acceptance criteria across all sampled stories (STORY-001..010, STORY-022..053,
  STORY-054..096) verified to follow standard predicate form; no missing mandatory fields.
- **HS rollups:** HS-INDEX.md Count column verified consistent with scenario-to-wave assignments
  as clarified in Pass-4 M-2 remediation.

Seven lower-tier findings were raised after the structural backbone passed.

## Finding Summary

### HIGH

**H-1 — dependency-graph.md wave-schedule Description cells (48 rows) mismatched to story H1 titles**
The wave-schedule table in `dependency-graph.md` contained 48 Description cells that did not
match the canonical story H1 titles as written in the individual STORY-NNN.md files and in
`stories/STORY-INDEX.md`. The descriptions were close paraphrases or earlier drafts of the story
titles, but any divergence breaks the traceability contract: the wave-schedule table is an index
artifact that must cite the authoritative story title verbatim so that automated tooling (and
human auditors) can match rows without ambiguity. With 48 Description cells all deviating to some
degree, the table functioned as a parallel, unvalidated title registry rather than a derivative
of the canonical STORY-INDEX.
**Remediated:** All 48 Description cells in `dependency-graph.md` wave-schedule table corrected
to match the H1 titles from the corresponding STORY-NNN.md files. `stories/STORY-INDEX.md` was
used as the cross-reference source; STORY-INDEX.md H1 titles were also spot-verified against
individual STORY-NNN.md H1 headings.

### MEDIUM

**M-1 — HS-094 BC citation list in HS-INDEX.md contained incorrect BC identifiers**
The HS-INDEX.md row for HS-094 listed a BC citation set that included identifiers from
subsystems unrelated to the scenario's behavioral scope. HS-094 exercises the CSV injection
neutralization behavior, which is governed exclusively by BC-2.12.005. The citation list
contained at least one extraneous BC reference (a holdover from an earlier batch-copy phase
that was not caught in Passes 1–4 because HS-INDEX audits had focused on Count-column semantics
and must-pass/should-pass classification rather than per-row BC accuracy).
**Remediated:** HS-094 BC citation list in `HS-INDEX.md` corrected to `BC-2.12.005` only.
Extraneous BC references removed.

### LOW

**L-1 — STORY-INDEX.md title for STORY-054 diverged from STORY-054.md H1**
The STORY-INDEX.md row title for STORY-054 was a paraphrase of the authoritative H1 in
`STORY-054.md`, not a verbatim copy. Consistent with H-1 pattern (canonical H1 vs. paraphrase
drift), but scoped to the index file rather than the dependency-graph wave table.
**Remediated:** STORY-INDEX.md row title for STORY-054 aligned to the verbatim H1 from
`STORY-054.md`.

**L-2 — STORY-INDEX.md titles for STORY-057 and STORY-002 diverged from story H1 headings**
Same pattern as L-1 but for STORY-057 and STORY-002: STORY-INDEX.md row titles were paraphrases
rather than verbatim H1 copies.
**Remediated:** STORY-INDEX.md row titles for STORY-057 and STORY-002 aligned to verbatim H1
headings from the respective STORY-NNN.md files.

**L-3 — BC-2.04.039.md proof method listed as "unit" instead of "Kani"**
`BC-2.04.039.md` listed its verification proof method as `unit` in the Verification Properties
block. VP-015, which governs this BC, specifies Kani model-checking as the proof method. The
`unit` value was a residual default from the VP back-fill burst (P8-DEFER); at that time the
proof-method field was not updated to match the VP's prescribed method, only the VP ID
cross-reference was added.
**Remediated:** BC-2.04.039.md proof method corrected from `unit` to `Kani`.

### NITPICK

**N-1 — Process gap: dependency-graph.md Description cells not mechanically validated against story H1 titles**
H-1 demonstrates that the wave-schedule Description cells in `dependency-graph.md` are not
machine-validated against STORY-INDEX.md or individual STORY-NNN.md H1 headings. A linter that
asserts Description == canonical story H1 (post-strip whitespace) would have caught all 48
divergences at dependency-graph construction time.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story content
defect once remediated.

**N-2 — Process gap: STORY-INDEX.md title accuracy not validated against story H1 headings**
L-1 and L-2 demonstrate that STORY-INDEX.md row titles drift from canonical STORY-NNN.md H1
headings without machine detection. The same linter as N-1 extended to STORY-INDEX.md would
close this gap.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story content
defect once remediated.

## Remediation Status

| Finding | Severity | Remediated? |
|---------|----------|-------------|
| H-1 | HIGH | YES — all 48 Description cells in dependency-graph.md corrected to match story H1 titles |
| M-1 | MEDIUM | YES — HS-094 BC citation list in HS-INDEX.md corrected to BC-2.12.005 only |
| L-1 | LOW | YES — STORY-INDEX.md STORY-054 title aligned to verbatim H1 |
| L-2 | LOW | YES — STORY-INDEX.md STORY-057 and STORY-002 titles aligned to verbatim H1s |
| L-3 | LOW | YES — BC-2.04.039.md proof method corrected from unit to Kani |
| N-1 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-2 | NITPICK | DEFERRED — cycle-close process-gap codification |

**Both blocking findings (1H + 1M) remediated. All 3 LOW findings remediated. 2 process-gap
NITPICKs deferred for cycle-close codification.**
Pass 6 may be dispatched.

## Finding Trajectory Across Story-Review Passes

| Pass | Findings | Severity Breakdown | Verdict |
|------|----------|--------------------|---------|
| 1 | 11 | 1C/3H/3M/2L/2N | NOT_CONVERGED |
| 2 | 7 | 0C/1H/2M/2L/2N | NOT_CONVERGED |
| 3 | 6 | 0C/1H/1M/2L/2N | NOT_CONVERGED (1 invalid) |
| 4 | 15 | 0C/3H/5M/4L/3N | NOT_CONVERGED (NON-MONOTONIC — scope expansion) |
| 5 | 7 | 0C/1H/1M/3L/2N | NOT_CONVERGED |

**Trajectory note:** Pass 5 returns to a monotonically declining blocking-severity count
(2 blocking vs. Pass 4's 8 blocking). The structural backbone (VP anchoring, BC traceability,
dependency edges, wave computation, AC quality, HS rollups) passed clean. All 5-pass findings
are presentation-consistency defects (title drift, BC citation accuracy, proof-method residual)
rather than behavioral or traceability gaps. Convergence counter remains 0/3; Pass 6 next.
