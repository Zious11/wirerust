---
document_type: adversary-pass-report
level: ops
version: "1.0"
story: STORY-149
cycle: wave-70-story-149
pass: 2
date: "2026-07-07"
worktree_head_reviewed: a02eb6f
classification: FINDINGS
producer: adversary (wave-70)
traces_to: STORY-149
bc_gating: BC-5.39.001
---

# Adversary Pass 2 Report — STORY-149

## Checkout Guard

**Result: PASS**

Worktree head reviewed: `a02eb6f`. Branch `feature/STORY-149-tls-carry-perf` confirmed at
correct head. No uncommitted changes. Grep-counts cross-check passed.

## Classification

**FINDINGS** — 1 finding: 1 LOW. Zero behavior-preservation defects. Zero phantom fixes.
Zero regressions from Pass-1 remediation.

Counted NOT-clean per convergence policy: remediation was applied (comment-only, 4 sites).
Clean streak remains 0 after pass 2.

## Pass-1 Fix Verification

All 5 actionable Pass-1 remediation targets confirmed FIXED with no phantom fixes or
regressions. Verification summary:

| Finding | Status | Notes |
|---------|--------|-------|
| F-S149P1-001 — assertion gameability | FIXED | `== 1` assertion confirmed; two companion tests present; no regression |
| F-S149P1-002 — stale Kani refs | FIXED | STORY-149 v1.2 structural references correct at `a02eb6f` |
| F-S149P1-003 — mem::replace wording | FIXED | Implementation uses `mem::take`; however, see F-S149P2-001 below for residual comment drift |
| F-S149P1-004 — fixture duplication | FIXED | Shared helper confirmed extracted; no test divergence risk |
| F-S149P1-005 — no-alloc overstatement | FIXED | Hot-path qualification confirmed in v1.2 |
| F-S149P1-006 — missing mod wrappers | FIXED | `#[cfg(test)] mod` wrappers present in both new test files |

## Re-Verified Axes

Pass 2 re-derived the following behavioral axes independently against `a02eb6f`:

1. **Symmetry** — take/restore pattern correctly symmetric across all carry-path entry
   points; `mem::take` clears carry before processing and restores on incomplete reassembly
   without double-borrow or state leak.
2. **Purge** — carry state correctly cleared on oversized-record rejection and on parse
   error; no stale carry survives an error path.
3. **Summarize** — convergence-report summary path (finding aggregation across passes)
   consistent with prior session; no new divergence introduced by `a02eb6f`.
4. **include! mechanism** — test macro expansion paths verified clean; the `include!`-based
   fixture loading in the test suite correctly resolves at compile time with no path drift.
5. **Kani alignment** — structural references in STORY-149 v1.2 correctly point to
   post-restructure module layout; Kani harness locations are reachable.

## Findings Detail

### LOW

**F-S149P2-001** — `mem::replace` doc drift (comment residuals)

The Pass-1 fix at `d18632c` correctly updated the implementation to use `mem::take` and
resolved the principal wording ambiguity in the implementation guidance. However, two
inline comments in the carry-path implementation (`src/`) and two comments in the test
file (`tests/bc_149_fragmented_fixture_tests.rs`) still referenced the old
"mem::replace swap pattern" phrase. These were not part of the original test-writer or
implementer remediation sweep.

This is a documentation hazard only — the code is correct and uses `mem::take`. No
functional defect. The comment drift would mislead future maintainers reading the code
alongside git history.

Remediation route: comment-only update at 4 sites via sibling sweep.

Remediated in: `208b2d4` (implementer; 4 sites updated via sibling sweep).

## Behavior-Preservation Verdict

**ZERO behavior-preservation defects.** Pass-1 restructure confirmed stable at `a02eb6f`.
No regressions introduced by any Pass-1 remediation commit.

## Convergence State

Pass 2 complete. Clean streak: 0 / 3 required (remediation applied — counted NOT-clean).
Pass 3 pending against remediation head `208b2d4`.
