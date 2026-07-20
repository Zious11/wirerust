---
document_type: red-gate-log
level: ops
version: "1.0"
status: draft
producer: test-writer
timestamp: 2026-07-20T04:38:53
phase: 3
inputs: []
input-hash: "d41d8cd"
traces_to: "STORY-166"
stub_architect_agent: "n/a (NO-STUBS verdict)"
stub_compile_verified: true
test_writer_agent: "test-writer"
red_gate_verified: true
---

# Red Gate Log: Wave 84 / STORY-166

Story: STORY-166 v1.2 "Wave-75 cycle-closing: citation symbol-at-line
assertion, demo-evidence scrub scope extension (project half)"
Branch: `feature/STORY-166-citation-symbol-anchor`
Worktree: `.worktrees/STORY-166`
Base SHA: `f0cb7374` (develop, post-STORY-147)

## Summary
| Story | Tests Written | All Fail (Red)? | Gate |
|-------|-------------|-----------------|------|
| STORY-166 | 3 (T23, T24, T25 / AC-166-001e) | Yes — 2 failed / 1 backward-compat control passed | PASSED |

## Stubs Created

### STORY-166: Citation Symbol-at-Line Assertion

**Step 2 verdict: NO-STUBS.** This story extends the existing
`bin/validate-citations` tool. The unextended tool IS the stub state: the
current `_CITATION_RE` regex rejects the new third field (symbol anchor) as
`MALFORMED`. The existing `_run_with_real_files()` harness hosts the new
tests unmodified — no additional scaffolding was needed. Baseline
(pre-existing test suite) was 22/22 green prior to any test authoring. No
stub commit was made — this is a valid, documented outcome per Step 2
guidance.

## Red Gate Verification

### STORY-166

Commit: `54d3fc78` — "test(STORY-166): add failing tests T23/T24 +
backward-compat control T25 (Red Gate, AC-166-001e)"
92 insertions, tests-only.

Orchestrator-verified run: 23 passed / 2 failed.

- AC-166-001e: `test_T23_anchor_present_passes` — FAIL (expected) —
  "expected exit 0, got 1" (both anchored citations rejected `MALFORMED`,
  including EC-002 regex-special anchor `arr[0]`)
- AC-166-001e: `test_T24_anchor_absent_symbol_not_at_line` — FAIL
  (expected) — "expected 'SYMBOL NOT AT LINE:' failure-class prefix in
  output, got stdout='MALFORMED: ...'" — deliberately discriminates
  right-exit-code-wrong-reason from the correct failure mode
- AC-166-001e: `test_T25_bare_citation_still_passes` — PASS by design
  (backward-compat control; asserts pre-existing bare-citation behavior is
  unchanged by the new tests' presence)

## Regression Check

Independent orchestrator-verified run in the worktree, 2026-07-20:

| Existing Tests | Status |
|---------------|--------|
| 22 pre-existing tests | all pass unchanged |
| New tests (T23, T24, T25) | 1 passed (T25, by design) / 2 failed (T23, T24) — clean AssertionErrors, no crashes |

**RED GATE: PASSED** — correctly red. T23 and T24 fail for substantive,
AC-traceable reasons (T23: anchored citations wrongly rejected; T24: right
exit code but wrong failure-class message); T25 passes by design as the
backward-compat control; no pre-existing test broke.

## Hand-Off to Implementer

- Stories ready for implementation: STORY-166
- Implementation guidance: Extend `_CITATION_RE` (and downstream parsing)
  in `bin/validate-citations` to accept an optional third field (symbol
  anchor), including regex-special characters in the anchor (EC-002, e.g.
  `arr[0]`). When an anchor is present, verify the symbol actually occurs
  at the cited line and emit a `SYMBOL NOT AT LINE:` failure-class message
  (not `MALFORMED:`) when it does not. When no anchor is present, preserve
  existing bare-citation behavior exactly (T25). Re-run
  `_run_with_real_files()` to confirm T23/T24 go green with zero
  regressions across the 22 pre-existing tests.
