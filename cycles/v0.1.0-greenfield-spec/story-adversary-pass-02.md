---
pass: 2
scope: story-decomposition
date: 2026-05-21
verdict: NOT_CONVERGED
findings: "0C/1H/2M/2L/2N"
total_findings: 7
convergence_counter: 0/3
---

# Story Adversary Pass 2 — Story Decomposition Review

**Date:** 2026-05-21
**Scope:** Story decomposition (STORY-*.md files) + holdout scenarios (HS-*.md, HS-INDEX.md) +
dependency graph (dependency-graph.md) + VP-INDEX.md
**Verdict:** NOT_CONVERGED — 0C/1H/2M/2L/2N (7 findings total)
**Convergence counter:** 0/3 (Pass 3 next)

## VP-Anchoring Check (Pass 1 CRITICAL confirmed clean)

Pass 1 C-1 (VP-to-Stories matrix mis-anchored) was remediated before this pass. The VP-to-Stories
cross-reference matrix in `dependency-graph.md` was verified correct: 20 VP rows, all story
references accurate, `HS-INDEX.md` present in `traces_to`. No regression on this finding.
VP-anchoring is confirmed clean.

## Finding Summary

### HIGH

**H-1 — 31 holdout scenarios reference nonexistent `--format json` flag (61 occurrences)**
Across 28 HS-*.md files, the CLI flag `--format json` appears 61 times in scenario scripts,
expected commands, and verification steps. The actual CLI flag is `--output-format json`
(see BC-2.12 series, cap-12, and src/cli.rs). The flag `--format` does not exist; running
these scenarios verbatim would produce a clap parse error rather than testing the intended
behavior. This is a HIGH finding because it renders a significant portion of the holdout
scenario suite untestable as written.
**Remediated:** All 61 occurrences replaced with `--output-format json` across the 28 affected
HS files (HS-026 through HS-050, HS-064, HS-073, HS-075).

### MEDIUM

**M-1 — `depends_on`/`blocks` asymmetry in 16 stories (missing cross-epic `blocks` edges)**
After Pass 1 corrected 8 intra-epic dependency pairs, a second class of asymmetry remained:
16 stories had correct `depends_on` entries but were missing the corresponding `blocks` entry
on the upstream story (or vice versa), primarily across epic boundaries. The dependency graph
edges are the canonical source; story frontmatter must mirror them bidirectionally.
**Remediated:** `depends_on`/`blocks` fields corrected for all 16 affected stories
(STORY-005, STORY-011, STORY-013, STORY-014, STORY-017, STORY-018, STORY-019, STORY-021,
STORY-033, STORY-046, STORY-052, STORY-055, STORY-057, STORY-058, STORY-071, STORY-080)
to achieve full bidirectional symmetry with the canonical edge list.

**M-2 — VP-INDEX "Primary BCs" column header ambiguous; 6 VP entries list BC subsets**
The VP-INDEX catalog column header was titled "Primary BCs", implying a curated subset rather
than the full set of BCs that the VP verifies. Additionally, 6 VP entries listed only a subset
of BCs from their corresponding VP file's `bcs:` frontmatter arrays, creating a traceability
gap. This finding also closes Phase-1 finding F-5 (consistency audit NITPICK: VP-INDEX column
header discrepancy).
**Remediated:** Column header renamed "Verified BCs". The 6 VP entries whose BC listings were
incomplete expanded to match the full `bcs:` arrays from their VP files:
- VP-001: added BC-2.04.001, BC-2.04.002
- VP-003: added BC-2.04.025, BC-2.04.026
- VP-005: added BC-2.07.018, BC-2.07.020, BC-2.07.021
- VP-009: added BC-2.04.006, BC-2.04.007
- VP-016: added BC-2.10.001, BC-2.10.002
- VP-020: added BC-2.11.022, BC-2.11.023, BC-2.11.024

### LOW

**L-1 — STORY-069 cites out-of-scope BC (BC-2.12.008 in SS-11 story)**
STORY-069 is an SS-11 (CSV Reporter) story. Its `## Behavioral Contracts` table listed
BC-2.12.008, which belongs to SS-12 (CLI). The correct BC for the CSV header-row guarantee
is within the SS-11 BC range. The citation was sourced from keyword-match heuristics at
story-creation time rather than from the BC-INDEX ground truth.
**Remediated:** BC-2.12.008 replaced with the correct SS-11 BC citation sourced from the
BC-INDEX and BC body review.

**L-2 — Redundant edge STORY-011→STORY-013 in dependency-graph.md**
`dependency-graph.md` contained a duplicate directed edge STORY-011→STORY-013. The edge
appears once as a direct dependency and again through a transitive path; the direct listing
was redundant given the explicit edge set. Removing it corrects the edge count.
**Remediated:** Redundant edge removed. Total edge count: 78→77 (63 intra-epic + 14
cross-epic). Both story frontmatter entries updated to reflect the corrected edge set.

### NITPICK

**N-1 — Process gap: no automated `--format` / `--output-format` flag validator for HS files**
The 61 occurrences of `--format json` across 28 HS files were not caught by the decomposition-
gate consistency audit (Step F) because no check validates CLI flag strings in HS scenario
bodies against the actual clap-parsed flag list. This is the same tooling-gap class as
P-CITE-PG (stale literal references, no machine validator).
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a story or
scenario content defect.

**N-2 — Process gap: VP-INDEX BC-listing completeness not validated against VP file `bcs:` arrays**
M-2 required manual cross-referencing of VP-INDEX rows against 20 VP file frontmatter arrays.
No automated check exists to assert that each VP-INDEX "Verified BCs" listing exactly matches
the corresponding VP file's `bcs:` array. This gap allowed 6 VP entries to carry truncated BC
lists from the initial authoring sweep.
**Disposition:** [process-gap] — deferred for cycle-close codification. Not a VP or BC defect.

## Remediation Status

| Finding | Severity | Remediated? |
|---------|----------|-------------|
| H-1 | HIGH | YES — 61 occurrences of `--format json` replaced with `--output-format json` across 28 HS files |
| M-1 | MEDIUM | YES — `depends_on`/`blocks` symmetry corrected for 16 stories |
| M-2 | MEDIUM | YES — VP-INDEX column renamed "Verified BCs"; 6 VP BC listings expanded to full set; closes Phase-1 F-5 |
| L-1 | LOW | YES — STORY-069 out-of-scope BC cite re-sourced to correct SS-11 BC |
| L-2 | LOW | YES — redundant STORY-011→STORY-013 edge removed; edge count 78→77 (63 intra + 14 cross) |
| N-1 | NITPICK | DEFERRED — cycle-close process-gap codification |
| N-2 | NITPICK | DEFERRED — cycle-close process-gap codification |

**All blocking findings (H-1, M-1, M-2, L-1, L-2) remediated.**
Pass 3 may be dispatched.

## Edge Count Note

After L-2 remediation, the canonical edge count is **77** (63 intra-epic + 14 cross-epic).
Any reference to "78 edges" in STATE.md, dependency-graph.md, or STORY-INDEX.md reflects the
pre-Pass-2 count and must be updated to 77.
