---
document_type: convergence-trajectory
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-22T00:00:00Z
cycle: phase-3-tdd
traces_to: STATE.md
---

# Convergence Trajectory — Phase 3 TDD Implementation (Wave-Level)

This file records per-wave wave-level adversarial convergence passes.
Per-story convergence logs are in `cycles/v0.1.0-greenfield-spec/STORY-NNN/` directories.

## Wave-Level Finding Progression

### Wave 1 (STORY-001 + STORY-069) — CLOSED/CONVERGED 2026-05-22

Wave-level convergence: 3 consecutive clean passes achieved. Wave 1 CLOSED.

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W1-1 | 2026-05-22 | 0 blocking | CLEAN 1/3 | — |
| W1-2 | 2026-05-22 | 0 blocking | CLEAN 2/3 | — |
| W1-3 | 2026-05-22 | 0 blocking | CLEAN 3/3 — GATE SATISFIED | develop HEAD at close: b7424b7; 329 tests green |

### Wave 2 (STORY-002 + STORY-003 + STORY-004 + STORY-070) — CLOSED/CONVERGED 2026-05-22

Wave-level convergence: 3 consecutive clean passes achieved. Wave 2 CLOSED.

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W2-1 | 2026-05-22 | 0 blocking | CLEAN 1/3 | — |
| W2-2 | 2026-05-22 | 0 blocking | CLEAN 2/3 | — |
| W2-3 | 2026-05-22 | 0 blocking | CLEAN 3/3 — GATE SATISFIED | develop HEAD at close: 3b2481c; 376 tests green |

### Wave 3 (STORY-071 + STORY-005) — CLOSED/CONVERGED 2026-05-22

Wave-level convergence: 3 consecutive clean passes achieved (pass 1 VERDICT CLEAN; passes 2+3
Nit-only = clean by convergence criterion). Wave 3 CLOSED.

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W3-1 | 2026-05-22 | 0 blocking | CLEAN 1/3 — VERDICT CLEAN | — |
| W3-2 | 2026-05-22 | 0 blocking (Nit only) | CLEAN 2/3 | Nit-only satisfies criterion |
| W3-3 | 2026-05-22 | 0 blocking (Nit only) | CLEAN 3/3 — GATE SATISFIED | develop HEAD at close: f0b5007; 9 stories total Waves 1-3 |

Non-blocking Nits (no action required): process-gap W3.2 (story status:draft not advanced on merge;
first confirmed recurrence here; recorded as process-gap item for codification).

### Wave 4 (STORY-011 + STORY-066) — CLOSED/CONVERGED 2026-05-22

Wave-level convergence: 3 consecutive clean fresh-context passes achieved (all VERDICT: CLEAN;
only non-blocking Nits found). Wave 4 CLOSED.

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W4-1 | 2026-05-22 | 0 blocking (Nits only) | CLEAN 1/3 — VERDICT CLEAN | Fresh-context |
| W4-2 | 2026-05-22 | 0 blocking (Nits only) | CLEAN 2/3 | Fresh-context |
| W4-3 | 2026-05-22 | 0 blocking (Nits only) | CLEAN 3/3 — GATE SATISFIED | Fresh-context; develop HEAD at close: f628c33; 11 stories total Waves 1-4 |

Non-blocking Nits recorded (no action): STORY-011.md N-2 (anchor line-range resync-discipline
note absent, though ranges currently resolve correctly); STORY-071.md changelog row ordering.
Process-gap W3.2 recurred (Wave 4) — STORY-011 and STORY-066 both showed status:draft after merge;
orchestrator fixed proactively before wave-level convergence. Process-gap W4.1 recorded (src edit
in same burst as anchor agents caused immediate staleness; see STATE.md Cycle-Close Follow-Up).

### Wave 5 (STORY-012) — CLOSED/CONVERGED 2026-05-22

Wave-level convergence: 3 consecutive clean fresh-context passes achieved (all VERDICT: CLEAN;
only 2 non-blocking cosmetic Nits found). Wave 5 CLOSED.

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W5-1 | 2026-05-22 | 0 blocking (Nit only) | CLEAN 1/3 — VERDICT CLEAN | Fresh-context |
| W5-2 | 2026-05-22 | 0 blocking (Nit only) | CLEAN 2/3 | Fresh-context |
| W5-3 | 2026-05-22 | 0 blocking (Nit only) | CLEAN 3/3 — GATE SATISFIED | Fresh-context; develop HEAD at close: bbddac6; 12 stories total Waves 1-5 |

Non-blocking cosmetic Nits (no action — may be tidied in a future doc/code sweep):
- N-1: tests/reassembly_engine_tests.rs — redundant inner `use wirerust::decoder::TransportInfo;`
  in the three STORY-012 non-TCP helpers (outer use already covers it).
- N-2: tests/reassembly_engine_tests.rs — stale "EC-005" comment label in test_ec_004.
Neither affects behavior or CI. Wave 5 GATE SATISFIED. Wave 6 (STORY-013) READY TO DISPATCH.

### Wave 6 (STORY-013) — CLOSED/CONVERGED 2026-05-22

Wave-level convergence: 3 consecutive clean fresh-context passes achieved (all VERDICT: CLEAN;
ZERO findings of any severity across all three passes). Wave 6 CLOSED.

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W6-1 | 2026-05-22 | 0 | CLEAN 1/3 — VERDICT CLEAN | Fresh-context |
| W6-2 | 2026-05-22 | 0 | CLEAN 2/3 | Fresh-context; ZERO findings |
| W6-3 | 2026-05-22 | 0 | CLEAN 3/3 — GATE SATISFIED | Fresh-context; develop HEAD at close: 3e705b5; 13 stories total Waves 1-6; 446 tests green |

Process-gap W4.1 recurred (Wave 6): adding TcpFlow::fin_count() at flow.rs:222-227 shifted all
state-machine methods +7 lines, staling 5 BC file anchors. Two-wave recurrence (Wave 4 + Wave 6)
recorded as W4.1 raised-priority item. Wave 7 (STORY-014): READY TO DISPATCH.

### Wave 7 (STORY-014) — CLOSED/CONVERGED 2026-05-25

Wave-level convergence: 8 passes; 3/3 clean streak on passes 6/7/8. Wave 7 CLOSED.
Per-story convergence: 8 passes; 3/3 clean streak on passes 6/7/8.
ADR-0004 amended (PR #121 → b23c6d3) to document #[doc(hidden)] test-seam exception.

Factory-artifacts commits this wave: c2a0181 (STORY-014 input-hash bump) → 4a200d0 (BC-2.04.009
+ VP-009 anchor fix, pass-1 partial) → 5e6cc59 (comprehensive flow.rs anchor sweep, pass-2) →
6db1772 (mega-sweep all src/reassembly anchors, pass-3) → aeacc6a (HS-014 BC-2.04.048 coverage
+ mod.rs anchor closing-brace fixes, pass-4) → 6d9c1fc (HS-014 PC labels, pass-5).

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W7-story-1 | 2026-05-25 | multiple anchors | BLOCKED | BC-2.04.009 + VP-009 anchor gaps; partial fix 4a200d0 |
| W7-story-2 | 2026-05-25 | anchor drift | BLOCKED | flow.rs sweep 5e6cc59; sibling BCs missed in pass-1 |
| W7-story-3 | 2026-05-25 | Minor F-1 (race) | REMEDIATED | race condition fixed; per-story 1/3 |
| W7-story-4 | 2026-05-25 | anchor closing-brace | BLOCKED | mega-sweep 6db1772; mod.rs cites missed |
| W7-story-5 | 2026-05-25 | LOW F-1 (doc) | REMEDIATED | doc fix; HS-014 PC labels 6d9c1fc |
| W7-story-6 | 2026-05-25 | 0 | CLEAN 1/3 — post-doc-fix | Fresh-context |
| W7-story-7 | 2026-05-25 | 0 | CLEAN 2/3 | Fresh-context |
| W7-story-8 | 2026-05-25 | 0 | CLEAN 3/3 — PER-STORY GATE SATISFIED | Fresh-context; PR #120 dispatched |
| W7-wave-1 | 2026-05-25 | CRITICAL F-1 + HIGH F-2 | BLOCKED | spec anchor regressions; swept |
| W7-wave-2 | 2026-05-25 | HIGH F-1 + MEDIUM F-2 | BLOCKED | mod.rs anchors + closing-brace; mega-swept aeacc6a |
| W7-wave-3 | 2026-05-25 | 4 findings | BLOCKED | HS-014 + mod.rs closing-brace; fixed aeacc6a |
| W7-wave-4 | 2026-05-25 | MEDIUM F-1 | BLOCKED | HS-014 PC labels; fixed 6d9c1fc |
| W7-wave-5 | 2026-05-25 | MEDIUM F-1 (sibling row) | BLOCKED | sibling-row missed in HS-014 fix |
| W7-wave-6 | 2026-05-25 | 0 | CLEAN 1/3 — post-pass-5 | Fresh-context |
| W7-wave-7 | 2026-05-25 | 0 | CLEAN 2/3 | Fresh-context |
| W7-wave-8 | 2026-05-25 | 0 | CLEAN 3/3 — WAVE GATE SATISFIED | Fresh-context; develop HEAD at close: b23c6d3; 14 stories total Waves 1-7 |

Cycle-close drift items logged: W7.1 (no public-API surface gate), W7.2 (partial-fix regression
discipline recurrence), W7.3 (out-of-scope analyzer/decoder anchor drift), W4.1 raised to
recurrence #4. All require research-agent validation per DF-VALIDATION-001 before issue filing.

### Wave 8 (STORY-019 + STORY-015) — CLOSED/CONVERGED 2026-05-26

Per-story convergence (STORY-019): 8 passes; 3/3 clean streak on passes 6/7/8 (14 findings remediated).
Per-story convergence (STORY-015): 8 passes; 3/3 clean streak on passes 6/7/8 (14 findings remediated).
Wave-level convergence: 9 passes; 3/3 clean streak on passes 7/8/9 (12 findings remediated).
Remediation vehicles: 3 develop PRs (#124 ADR-0004 v2 amendment, #125 chore cleanup, #126 ADR visibility fix)
+ 4 factory BC commits (BC-2.04.029 v1.4, BC-2.04.013 v1.4, BC-2.04.011 v1.5, BC-2.04.010 v1.5/v1.6, BC-2.04.039 v1.4).
Factory-artifacts key commits this wave: c4d6a1f (STORY-019 v1.2 anchor refresh), e49bdae (STORY-015 +
STORY-019 input-hash bumps), + BC update commits + STORY-019 v1.3/1.4/1.5 + STORY-015 v1.2 amendments.

Per-story STORY-019 passes:

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W8-S019-story-1 | 2026-05-26 | enforcement-mode sibling-BC gap | BLOCKED | sibling-BC propagation gap; factory BC commits |
| W8-S019-story-2 | 2026-05-26 | within-BC sibling-section gap | BLOCKED | BC body vs ECs/CVs sync; factory BC commits |
| W8-S019-story-3 | 2026-05-26 | ADR-narrative accuracy | BLOCKED | STORY-019 v1.3 amendment |
| W8-S019-story-4 | 2026-05-26 | BC↔test correspondence | BLOCKED | STORY-019 v1.4 amendment |
| W8-S019-story-5 | 2026-05-26 | minor doc | BLOCKED | STORY-019 v1.5 amendment |
| W8-S019-story-6 | 2026-05-26 | 0 | CLEAN 1/3 | Fresh-context |
| W8-S019-story-7 | 2026-05-26 | 0 | CLEAN 2/3 | Fresh-context |
| W8-S019-story-8 | 2026-05-26 | 0 | CLEAN 3/3 — PER-STORY GATE SATISFIED | Fresh-context; PR #122 dispatched |

Per-story STORY-015 passes:

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W8-S015-story-1 | 2026-05-26 | enforcement-mode sibling-BC gap | BLOCKED | sibling-BC propagation gap; factory BC commits |
| W8-S015-story-2 | 2026-05-26 | within-BC sibling-section gap | BLOCKED | BC body vs ECs/CVs sync; factory BC commits |
| W8-S015-story-3 | 2026-05-26 | ADR-narrative accuracy | BLOCKED | STORY-015 v1.2 amendment |
| W8-S015-story-4 | 2026-05-26 | BC↔test correspondence | BLOCKED | factory BC commits |
| W8-S015-story-5 | 2026-05-26 | minor doc | BLOCKED | STORY-015 v1.2 amendment |
| W8-S015-story-6 | 2026-05-26 | 0 | CLEAN 1/3 | Fresh-context |
| W8-S015-story-7 | 2026-05-26 | 0 | CLEAN 2/3 | Fresh-context |
| W8-S015-story-8 | 2026-05-26 | 0 | CLEAN 3/3 — PER-STORY GATE SATISFIED | Fresh-context; PR #123 dispatched |

Wave-level passes:

| Pass | Date | Findings | Verdict | Notes |
|------|------|----------|---------|-------|
| W8-wave-1 | 2026-05-26 | sibling-BC enforcement-mode gaps | BLOCKED | PR remediation + factory BC commits |
| W8-wave-2 | 2026-05-26 | ADR vocabulary drift (W8.2) | BLOCKED | PR #125 chore: source-comment alignment |
| W8-wave-3 | 2026-05-26 | FALSE-POSITIVES F-1/F-2 HIGH | STALE-SOURCE | Caused by stale local develop (W8.1); git pull resolved |
| W8-wave-4 | 2026-05-26 | within-BC sibling-section gap | BLOCKED | factory BC commits |
| W8-wave-5 | 2026-05-26 | ADR-narrative accuracy | BLOCKED | PR #124 ADR-0004 v2 amendment |
| W8-wave-6 | 2026-05-26 | enum visibility gap | BLOCKED | PR #126 ADR visibility-widening fix → 4b9b85f |
| W8-wave-7 | 2026-05-26 | 0 | CLEAN 1/3 | Fresh-context |
| W8-wave-8 | 2026-05-26 | 0 | CLEAN 2/3 | Fresh-context |
| W8-wave-9 | 2026-05-26 | 0 | CLEAN 3/3 — WAVE GATE SATISFIED | Fresh-context; develop HEAD at close: 4b9b85f; 16 stories total Waves 1-8 |

Cycle-close drift items logged: W8.1 (stale-local-develop false positives), W8.2 (ADR amendment dialect
drift), W8.3 (wave-level adversarial cost escalation), W8.4 (W7.2 partial-fix regression recurrence in W8).
All require research-agent validation per DF-VALIDATION-001 before issue filing.

## Wave-Level Summary

| Wave | Stories | Gate Status | develop HEAD at Close | Stories Cumulative |
|------|---------|-------------|----------------------|--------------------|
| 1 | STORY-001, STORY-069 | CLOSED/CONVERGED | b7424b7 | 2 |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | CLOSED/CONVERGED | 3b2481c | 6 |
| 3 | STORY-071, STORY-005 | CLOSED/CONVERGED | f0b5007 | 8 |
| 4 | STORY-011, STORY-066 | CLOSED/CONVERGED | f628c33 | 10 |
| 5 | STORY-012 | CLOSED/CONVERGED | bbddac6 | 12 |
| 6 | STORY-013 | CLOSED/CONVERGED | 3e705b5 | 13 |
| 7 | STORY-014 | CLOSED/CONVERGED | b23c6d3 | 14 |
| 8 | STORY-019, STORY-015 | CLOSED/CONVERGED | 4b9b85f | 16 |

Wave 9 (STORY-016 + STORY-020): READY TO DISPATCH. develop HEAD: 4b9b85f.

---

## Wave 18 Per-Story Convergence (2026-05-29)

### STORY-058 Per-Story Convergence — 13 passes; BC-5.39.001 ACHIEVED

Frozen code: feature/STORY-058 HEAD 4c252f3. 5 test commits total.
114 tls_analyzer_tests + 4 tls_integration_tests green; zero src changes.

| Pass | Verdict | Key Findings |
|------|---------|-------------|
| P1 | DIRTY | 2 MED: buffer-cap literal (RESIDUE_CAP 65535) unasserted in test + AC-013 mis-citation in story FSR |
| P2 | CLEAN | 0 findings |
| P3 | DIRTY | 1 HIGH: BC-2.07.033 Proof Method cites done-short-circuit test for within-loop-skip claim. 1 MED: BC-2.07.029 invariant-2 arithmetic (`parse_errors − truncated_records` incorrect) |
| P4 | DIRTY | Corroborated P3-HIGH (BC-2.07.033 mis-anchor, independent fresh context) |
| P5 | DIRTY | 1 MED: BC-2.07.035 Evidence field stale (references generic tests not dedicated on_flow_close). 1 MED: BC-2.07.033 internal xref to BC-2.07.034 inconsistent post-v1.3 |
| P6 | DIRTY | Corroborated BC-2.07.035 evidence + 1 MED: AC-002 story↔BC contradiction (defensive/by-inspection qualification missing from story) |
| P7 | CLEAN | 0 findings |
| P8 | DIRTY | 1 MED: 3rd-occurrence stale AC-013 index comment at tls_analyzer_tests.rs (story body + BC fixed; test index missed) |
| P9 | CLEAN | 0 findings |
| P10 | CLEAN | 0 findings |
| P11 | CLEAN (streak=1) | 4 LOW/OBS accepted: F-S058-P11-001 (stale comment:6819), F-S058-P11-002 (EC-label set), F-S058-P12-O1 (anchor off-by-one), F-S058-P13-O4 (cross-story collision) |
| P12 | CLEAN (streak=2) | 0 findings above LOW |
| P13 | CLEAN (streak=3) | BC-5.39.001 ACHIEVED |

**Trajectory shorthand:** `|W18-S058-story:13ps-3clean(P11/P12/P13;2MED+1HIGH-mis-anchor+3MED-cross-artifact+1MED-3rd-occurrence-rem;deepest-W18)`

Factory artifacts remediated: STORY-058.md v1.1→v1.2→v1.3, BC-2.07.004 v1.3, BC-2.07.005 v1.3, BC-2.07.029 v1.3, BC-2.07.033 v1.3→v1.4, BC-2.07.035 v1.3.
Process gap extended: PG-W18-002 (test-citation change checklist). Deferred: F-S058-P11-001/002, F-S058-P12-O1, F-S058-P13-O4 (all LOW).
Next: STORY-058 demos + PR.
