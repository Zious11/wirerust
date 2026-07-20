---
document_type: per-story-convergence-report
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-20T06:00:00Z
phase: step-4.5-per-story-adversarial
inputs: []
input-hash: "[live-state]"
traces_to: STATE.md
story: STORY-166
cycle: wave-084
passes_total: 10
verdict: CONVERGED
criterion: BC-5.39.001
clean_streak: [P8, P9, P10]
final_head: 55b39152
base: "49255464"
story_version: "1.3"
---

# Convergence Report — STORY-166 (compact)

## Pipeline Run: 2026-07-20
## Product: wirerust — STORY-166 Citation Symbol-Anchor Assertion (E-11, wave-084)
## Iterations: 10

---

## Verdict: CONVERGED — BC-5.39.001 SATISFIED (3 consecutive clean passes: P8/P9/P10)

## Trajectory

`NIT(3L) -> 1M+1L -> 1M+2L -> 2M+1L -> NIT(0) -> NIT(1L) -> 1M[PG] -> NIT(2L) -> NIT(0) -> NIT(0)`

| Pass | Verdict | HIGH | MED | LOW | Code Tip | Part A (prior-pass fix verification) |
|------|---------|------|-----|-----|----------|----------------------------------------|
| P1 | NITPICK_ONLY | 0 | 0 | 3 | 6c297fe5 | — (first pass) |
| P2 | FAIL_FINDINGS | 0 | 1 | 1 | 3510032c | 3/3 VERIFIED-FIXED |
| P3 | FAIL_FINDINGS | 0 | 1 | 2 | 55b39152 | 2/2 VERIFIED-FIXED |
| P4 | FAIL_FINDINGS | 0 | 2 | 1 | 55b39152 (unchanged) | 3/3 VERIFIED-FIXED |
| P5 | NITPICK_ONLY | 0 | 0 | 0 | 55b39152 (unchanged) | 3/3 VERIFIED-FIXED |
| P6 | NITPICK_ONLY | 0 | 0 | 1 | 55b39152 (unchanged) | — |
| P7 | FAIL_FINDINGS | 0 | 1 [process-gap] | 0 | 55b39152 (unchanged) | — |
| P8 | NITPICK_ONLY | 0 | 0 | 2 (carried) | 55b39152 (unchanged) | 1/1 VERIFIED-FIXED — streak 1/3 |
| P9 | NITPICK_ONLY | 0 | 0 | 0 (new) | 55b39152 (unchanged) | streak 2/3 |
| P10 | NITPICK_ONLY | 0 | 0 | 0 | 55b39152 (unchanged) | streak 3/3, CONVERGED |

---

## Headline Narrative

Code was written and frozen early: the implementation converged to tip **55b39152**
by Pass 3 and held unchanged through Pass 10 — every finding from Pass 4 onward was
governance-prose or documentation, with zero further source-code churn.

The pivotal finding was **Pass-7 F-S166P7-001** (MEDIUM, process-gap): the adversary
caught a **Pass-3-era fix regression** in `demo-evidence-scrub-gate.md`'s CI-guard
example. The grep-based guard exits `2` on a missing `.factory/` path — a condition
that fires even when leaks **are** present in `docs/`, producing a **false-green**
result that would have silently defeated the gate. This was orchestrator-probe
execution-verified (not just inspected), then fixed with a path-guarded loop in
factory-artifacts commit `eef569c9787fba7d29e8dfe7be6cbbe0e9ce434e`. Per human
directive, the root-cause class (governance-doc CI examples not validated against
develop/factory-artifacts branch topology) is routed to the upstream vehicle rather
than a further local story amendment.

Passes 1-4 closed out residual test-assertion, CHANGELOG-enumeration, and
Previous-Story-Intelligence-currency findings, all adversary-verified fixed before
the next pass opened. Passes 5-10 held code tip `55b39152` fixed, closing on three
consecutive NITPICK_ONLY passes (BC-5.39.001's 3-clean-streak criterion — STORY-147
precedent).

Factory-doc track fixes across the cycle: **F-S166P3-001** (T23 count assertion
tightened to exact PASS line), **F-S166P4-002** (scrub-gate trigger predicates
harmonized with extended scope, factory-artifacts commit `9fa2072e`), and
**F-S166P7-001** (CI-guard false-green fix, factory-artifacts commit `eef569c9`).

Story spec evolved **v1.2 -> v1.3** across the 10 passes (Pass-4 currency fixes:
Previous-Story-Intelligence points claim aligned to the v1.1 3-pt re-estimate
F-S166P4-001; stale draft-status self-reference dropped F-S166P4-003).

## Non-Blocking Residuals

- **F-S166P6-001** (LOW): lone-CR line-model divergence — untested/undocumented edge
  in the line-counting model.
- **F-S166P8-001** (LOW): colon-in-anchor + `\S+`-greediness siblings — untested/
  undocumented edge in the anchor-parsing regex.
- **F-S166P8-002** (LOW): pre-existing harness empty-list latent behavior, dating to
  the STORY-164/165 era; not introduced by STORY-166.
- **Background line-anchor staleness** (LOW): deferred to the wave-84 gate currency
  sweep.
- **Line-33 base-command carve-out** (LOW): documented, non-blocking.
- **">= 25" floor phrasing** (LOW): non-blocking prose nit.

All six residuals are non-blocking and carried for gate ratification, not
convergence blockers.

## Process Gaps Noted for Cycle Close

- **F-S166P7-001** [process-gap]: governance-doc CI examples not validated against
  develop/factory-artifacts branch topology (scrub-gate CI-guard exit-2 false-green
  regression) — human directive: route the durable fix upstream.

## Traceability

- Full pass-by-pass state: `adversary-convergence-state.json` (this directory)
- Story: `.factory/stories/STORY-166.md` (v1.3)
- STORY-INDEX: `.factory/stories/STORY-INDEX.md`
- Related factory-artifacts commits: `9fa2072e` (F-S166P4-002), `eef569c9` (F-S166P7-001)
