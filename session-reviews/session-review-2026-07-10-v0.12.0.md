---
document_type: session-review
date: 2026-07-10
run_id: v0.12.0-release-chain
path: composite
path_name: "wave-70 + wave-71 + maint-2026-07-08 + issue-triage + wave-72 + maint-2026-07-09 + v0.12.0-release"
product: wirerust
duration: "~4 calendar days (2026-07-07 through 2026-07-10)"
released_version: v0.12.0
prior_released_version: v0.11.5
stories_delivered: 9  # STORY-149 (wave-70) + STORY-150/156/157 (wave-71) + STORY-158/159/160/161 (wave-72)
prs_reviewed: [374, 375, 376, 377, 378, 379, 380, 381, 382, 383, 384, 386, 387, 388, 389, 390, 391, 393, 394]
decisions: [D-393, D-394, D-395, D-396, D-397, D-398, D-399, D-400, D-401, D-402, D-403, D-404, D-405, D-406, D-407, D-408, D-409, D-410, D-411, D-412, D-413, D-414, D-415, D-416, D-417, D-418, D-419, D-420, D-421, D-422]
total_cost: N/A
prior_review: session-review-maint-2026-07-06.md
human_decisions:
  recorded_at: pending
  notes: >-
    Review produced 2026-07-10. Human review within 72h per session-review protocol.
    Proposals pending disposition.
---

# Session Review: wirerust — v0.12.0 Release Chain — 2026-07-10

## Executive Summary

This was the largest single-review scope in the project's history: two feature waves
(70, 71), two maintenance sweeps, one issue-triage run, one out-of-cycle dependency
merge, and the v0.12.0 release — all in four calendar days. The factory handled the
compressed schedule cleanly: all quality gates passed, holdout scores held at or above
threshold (wave-72 achieved perfect 1.00/16 scenarios), and v0.12.0 shipped on schedule
with a BREAKING JSON surface change and histories successfully reunified.

The top three findings are: (1) BREAKING-change stories need a mandatory holdout-
expectation sweep before PR creation — 13 stale scenarios landed as unplanned gate
work; (2) background adversary agent relay remains unreliable for large-output tasks
— synchronous dispatch is the operational fix; (3) the docs-agent content-fabrication
pattern (3 HIGH findings including inverted technical claims) shows that documentation
PRs with heavy context load need the same strict 3/3 adversarial convergence as code PRs.

Prior review's approved proposals (PROP-MAINT-01..04) are all confirmed effective in
this period. The deferred PROP-MAINT-07 (import-style batch refactor story) was not
acted upon and remains open.

---

## Run Overview

| Metric | Value | Benchmark | Status |
|--------|-------|-----------|--------|
| Total cost | N/A (no cost-summary.md) | — | — |
| Duration | ~4 calendar days | — | — |
| Stories delivered | 9 (STORY-149, 150, 156, 157, 158, 159, 160, 161) | — | PASS |
| Story points delivered | 58 pts (STORY-149: 7 + wave-71: 13 + wave-72: 29 + other: ~9) | — | — |
| Waves closed | 3 (70, 71, 72) | — | — |
| Versions released | v0.11.5 (2026-07-07), v0.12.0 (2026-07-10) | — | — |
| Adversary passes total | ~38 (W70:5, W71:7, maint-08:~13, W72:4, maint-09:6, release:~3) | — | — |
| Gate-fix PRs | 2 (wave-71 PR #381, wave-72 PR #391) | — | PAT-WGFIX |
| Adversary NOT-CLEAN counts (waves) | W70: 2, W71: 2, W72: 1 | — | IMPROVING |
| Holdout mean (wave-70) | 0.920 | ≥ 0.85 | PASS |
| Holdout mean (wave-71) | 0.970 | ≥ 0.85 | PASS |
| Holdout mean (wave-72) | 1.000 (perfect) | ≥ 0.85 | PASS |
| Full-suite test count | 2,392 (at wave-72 close) | ≥ prior | PASS |
| Spec coherence (maint-09) | 33/33 PASS | 33/33 | PASS |
| Holdout scenarios active | 132 | all active | PASS |
| Input-hash scan | MATCH=115 STALE=0 | STALE=0 | PASS |
| Stories_delivered adjudicated | 101 (corrected from 106) | direct row count | RESOLVED |
| Process gap stories created | STORY-158, STORY-162, STORY-163 | ≥1/wave | PASS |

---

## 1. Cost Analysis

**No explicit cost data available.** `cost-summary.md` is absent or unpopulated.
The analysis below uses session-marker density and adversary-pass counts as proxy
cost indicators.

### Session density by date (from sidecar-learning.md)

| Date | Session markers | Inferred activity |
|------|-----------------|-------------------|
| 2026-07-06 | ~52 markers | maint-2026-07-06 (prior review period) |
| 2026-07-07 | ~28 markers | v0.11.5 release (D-393) + wave-70 gate (D-396) + wave-71 delivery start |
| 2026-07-08 | ~160+ markers | Wave-71 close (D-404) + maint-2026-07-08 (D-406) + triage-2026-07-08 (D-407) + wave-72 delivery start |
| 2026-07-09 | ~90 markers | Wave-72 close (D-415, D-416) + maint-2026-07-09 (D-418..D-420) |
| 2026-07-10 | ~60 markers | v0.12.0 release (D-421, D-422) + cleanup |

2026-07-08 is the highest-density day by ~2×, consistent with the fact that both
wave-71 close (7 adversary passes) and maint-2026-07-08 (13 adversary passes) landed
on the same day.

### Highest-cost operations (estimated)

1. **maint-2026-07-08 DF-VALIDATION-001 triage** — 10 research-agent invocations + codebase-analyzer passes; this class is expensive but high-value (10/10 CONFIRMED).
2. **Wave-71 adversary convergence** — 7 passes for 3 stories including one process-gap codification story (STORY-157); above-average for a 3-story wave.
3. **maint-2026-07-09 Route A** — 6 adversary passes for a documentation PR; 3 finding rounds before streak. Docs PRs are not cheap when the docs-agent fabricates content.
4. **Wave-72 holdout repair** — 13 stale scenario repairs were unplanned gate work triggered by the BREAKING JSON change in STORY-160. This work occurred at gate time rather than during delivery.

### Cost efficiency observation

**Wave-72 was the most cost-efficient delivery wave:** 4 stories, 29 pts, only 4 adversary passes (1 NOT-CLEAN → 3 CLEAN streak), perfect holdout, and 5 PRs merged. The wave-72 gate-fix PR #391 bundled 4 separate classes of fixes (CI guard, security, code-review, BREAKING changelog placement) into one PR — efficient batching.

**Recommendation:** Track a formal `cost_per_story_point` metric once cost-summary.md is populated. The absence of cost data is the single largest gap in the review infrastructure.

---

## 2. Timing Analysis

**Wall-clock span:** 4 calendar days (2026-07-07 to 2026-07-10).

### Activity sequence (approximate)

| Activity | Date | Decisions | Notes |
|----------|------|-----------|-------|
| v0.11.5 release | 2026-07-07 | D-393 | PR to main + tag + back-merge |
| Wave-70 gate close | 2026-07-07 | D-395, D-396 | 5 adversary passes; PRs #374–#377 |
| Wave-71 delivery | 2026-07-07/08 | D-397..D-403 | STORY-150/156/157 TDD + PR lifecycle |
| Wave-71 gate + close | 2026-07-08 | D-404 | 7 passes; gate-fix PR #381 |
| maint-2026-07-08 | 2026-07-08 | D-406 | 7 sweeps + 11 DF-VALIDATION-001 items; PRs #382–#384 |
| triage-2026-07-08 | 2026-07-08 | D-407 | 10 issues validated; #385 created |
| Wave-72 delivery | 2026-07-08/09 | D-408..D-414 | STORY-158/159/160/161 TDD + PRs |
| Dependabot PR #386 | 2026-07-09 | D-417 | indicatif 0.18.5→0.18.6 (out-of-cycle) |
| Wave-72 gate + close | 2026-07-09 | D-415, D-416 | 4 passes; gate-fix PR #391 |
| maint-2026-07-09 | 2026-07-09/10 | D-418..D-420 | Route A (PR #393) + Route B + STORY-163 |
| v0.12.0 release | 2026-07-10 | D-421, D-422 | PR #394; histories reunified |

**Bottleneck:** The 2026-07-08 session was the densest, combining wave-71 close + maint-2026-07-08 + triage into a single day. Session marker analysis shows ~40-minute clusters followed by ~10-minute gaps — consistent with heavy agent-dispatch cycles.

**Back-merge timing:** v0.11.5 back-merge used squash (PG-GITFLOW-SQUASH-BACKMERGE created); v0.12.0 back-merge used fast-forward (histories reunified). The squash resolution required a join commit before the v0.12.0 release PR could merge cleanly. This adds latency to every release that follows a squash back-merge.

**PR cycle times (maintenance PRs):**
- PR #382 (docs): Created and merged same-session (2026-07-08)
- PR #383 (test/fix): Created and merged same-session
- PR #384 (refactor): Created and merged same-session
- PR #393 (docs): Created 2026-07-09, merged same-day; 6 adversary passes required

**Recommendation:** Flag squash back-merges at the time they occur (not post-hoc) — each one creates unresolved-histories debt that must be paid before the next release. PG-GITFLOW-SQUASH-BACKMERGE (candidate for STORY-163 codification) captures this.

---

## 3. Convergence Analysis

### Wave adversarial convergence (all waves this period)

| Wave | Stories | Total Passes | NOT-CLEAN | Streak | Trajectory |
|------|---------|-------------|-----------|--------|------------|
| Wave-70 | STORY-149 | 5 | 2 | 3/3 (W3-W4-W5) | 0→1→1→0→0 |
| Wave-71 | STORY-150/156/157 | 7 | 2 | 3/3 (P5-P6-P7) | 1→0→0→1→0→0→0 |
| Wave-72 | STORY-158/159/160/161 | 4 | 1 | 3/3 (P2-P3-P4) | 1→0→0→0 |

**Trend: Improving.** Wave-70: 5 passes. Wave-71: 7 passes (multi-story including
process-gap story; partial convergence reset at P4 due to evidence.md identity gap).
Wave-72: 4 passes (fastest convergence in any wave this period). The adversary is
converging faster — fewer novel structural defects remain, and STORY-158's
codifications (CHANGELOG gate, cycle-artifact identity lint) closed the classes that
caused NOT-CLEAN findings in wave-71.

**Wave-71 P4 reset:** The P4 NOT-CLEAN (evidence.md identity headers) was a factory-
artifact gap, not a code defect. It was caught at wave scope because per-story
adversary passes don't review neighboring evidence.md files. This class of finding is
now gated by STORY-158 AC-158-002 (lint-cycle-artifact); should not recur.

**Maintenance convergence:**

| Run | Route | Passes | NOT-CLEAN | Result |
|-----|-------|--------|-----------|--------|
| maint-2026-07-08 | PR #382 | 3 | 0 | CONVERGED |
| maint-2026-07-08 | PR #383 | 3 | 0 | CONVERGED |
| maint-2026-07-08 | PR #384 | 3 | 0 | CONVERGED |
| maint-2026-07-09 | Route A PR #393 | 6 | 3 | CONVERGED |

**Route A is an outlier:** 6 passes to converge for a documentation PR is
disproportionate. The 3 finding rounds (one per pass before streak) were all HIGH
severity: fabricated vocabulary, fabricated ADR citation, inverted technical claim.
This finding class is traced to the docs-agent content-fabrication pattern (see
Section 4). The strict 3/3 convergence requirement was necessary and caught all
three defects, but cost 3× the baseline. Docs PRs with heavy context load are
adversary-expensive.

---

## 4. Agent Behavior Analysis

### Per-agent behavior summary

**Wave-level adversary (all waves):**
- Wave-72 P1: caught action-pin-gate CI no-op (scan of 0 files) — structurally
  identical to STORY-158 AC-158-004 but in a different file. Demonstrates value of
  wave-scope adversary passes over per-story scope (Wave-72 Lesson 1). No issues.

**Per-story adversary (all waves):**
- Wave-71 per-story passes: no stale-state recurrences (GUARD-002 checkout-guard
  effective; confirms PG-S149-001 fix from STORY-157). No issues.
- Wave-72 per-story passes: no BREAKING holdout impact detected per-story — STORY-160's
  holdout expectation staleness was only caught at wave gate. This is the expected
  scope limitation (per-story adversary doesn't scan `.factory/holdout-scenarios/`).

**DF-VALIDATION-001 research agent (maint-2026-07-08):**
- 11 items: 4 CONFIRMED, 1 REFUTED, 1 UNVERIFIABLE, 1 ALREADY-RESOLVED, 1 CLOSED-BY-
  DESIGN, 1 NOT-DEFECT, 2 CONFIRMED-RESIDUAL. All verdicts substantively correct.
- Catch rate of 4/11 CONFIRMED (plus 2 CONFIRMED-RESIDUAL) is reasonable for a
  mixed-origin triage load.

**Docs-writer agent (maint-2026-07-09 Route A):** ANOMALOUS
- 3 HIGH findings attributable to agent content fabrication:
  - F-RA-P1-001: invented tri-state vocabulary not in any BC or spec
  - F-RA-P2-001: fabricated ADR-0008 citation for a document that does not exist
  - F-RA-P3-001: inverted VLAN/QinQ/MACsec technical claim (docs stated "no findings"
    when ground-truth code `src/decoder.rs` confirmed findings ARE produced)
- All three were caught by adversary convergence before merge. The inverted VLAN claim
  (F-RA-P3-001) was verifiable against source code — the adversary's code-trace
  correctly found the discrepancy. The ADR fabrication was caught by checking whether
  ADR-0008 exists.
- Root cause: docs agents under context pressure will paraphrase incorrectly, fill
  gaps with invented content, and adopt incorrect vocabulary from ambient context.
  Heavy documentation tasks that reference multiple source artifacts simultaneously
  are high-fabrication-risk.

**Background adversary relay (maint-2026-07-08 L-005):** ANOMALOUS
- 2 incidents of background-dispatched adversary agents completing analysis internally
  but failing to emit reports to the session.
- Workaround (synchronous re-dispatch) resolved both incidents.
- This is now documented as ENGINE-NOTE ADVERSARY-RELAY-UNRELIABLE-001.

**pr-manager sub-agent (maint-2026-07-09 PG-MERGE-AUTH-SUBAGENT-CLASSIFIER):** NEW FAILURE MODE
- pr-manager received merge authorization via teammate-message relay from orchestrator
- Harness permission classifier blocked the merge because the authorization signal
  was not visible in the main conversation thread
- Orchestrator executed `gh pr merge` in the main thread under direct user authorization
- This is correct harness behavior (authorization must be in the main thread), but the
  factory dispatch protocol did not account for this; the pr-manager was dispatched
  expecting to complete the merge autonomously
- Codified: PG-MERGE-AUTH-SUBAGENT-CLASSIFIER → STORY-163 AC-163-002

**State-manager:** 
- Correct ordering maintained throughout (state_manager_runs_last).
- stories_delivered counter adjudication (106→101) was a significant corrective action
  — the counter had drifted 5 units over multiple prior cycles, now corrected via
  direct row count evidence (D-419).

---

## 5. Gate Outcome Analysis

| Gate | First Try? | Outcome | Notes |
|------|-----------|---------|-------|
| Wave-70 integration gate | NO (2 NOT-CLEAN) | PASS | 5 passes; gate-fix bundles in PR #376, #377 |
| Wave-71 integration gate | NO (2 NOT-CLEAN) | PASS | 7 passes; gate-fix PR #381 (CHANGELOG) |
| Wave-72 integration gate | NO (1 NOT-CLEAN) | PASS | 4 passes; gate-fix PR #391 (CI guard + security + CR) |
| v0.11.5 release PR | YES | PASS | Squash back-merge created PG-GITFLOW-SQUASH-BACKMERGE |
| v0.12.0 release PR | YES | PASS | CI 11/11; fast-forward back-merge; histories reunified |
| maint-2026-07-08 PR #382 | YES (3/3) | PASS | HIGH inverted ARP tuning caught pre-merge |
| maint-2026-07-08 PR #383 | YES (3/3) | PASS | SEC-010/011 resolved |
| maint-2026-07-08 PR #384 | YES (3/3) | PASS | 109 saturating_add conversions |
| maint-2026-07-09 PR #393 | NO (3 finding rounds) | PASS | 3 HIGH docs fabrications caught |
| DF-VALIDATION-001 (triage-07-08) | n/a | 10/10 CONFIRMED | All 10 issues correctly verdicted |
| Holdout re-eval (wave-72 gate, after repairs) | n/a | PASS 16/16 | 13 scenarios repaired by product-owner |

**First-try pass rate:** 5 of 11 gates (45%). This is below the benchmark of 60% from
prior cycles. However, "no first-try" here means "adversary found something before
merge" — which is the correct outcome. The 3 finding rounds in Route A (docs) are the
primary driver of the lower rate. The wave gates did not pass first try (by design —
the gate-level adversary is expected to find things per-story scope misses).

**Human override frequency:** All 4 counted instances of human merge execution (wave-70
+ maint-2026-07-08 × 3 + maint-2026-07-09 × 1) followed the correct DF-PR-MANAGER-
COMPLETE-001 exception pathway. These are not overrides — they are the policy.

**New gate required:** A "holdout-expectation sweep" gate on any BREAKING-change story
before PR creation (Wave-72 Lesson 2). Not currently enforced — 13 unplanned repairs
landed at wave gate. Candidate PROP-V0.12.0-01.

---

## 6. Wall Integrity Analysis

**All information asymmetry walls held during this period.**

**Wave-level holdout evaluation (wave-72):**
- Holdout evaluator was dispatched with public API surface + scenario files only.
- The 13 stale-expectation repairs were performed by the product-owner from BC shape
  (outward-visible contracts), not from implementation internals. Wall intact.

**Adversary independence (all waves):**
- GUARD-002 checkout-guard confirmed effective for wave-71 (PROC-OBS-001): no stale-
  state recurrences. Per-story adversary passes used verified HEAD SHA in preamble.
- Wave-72 adversary passes: 4 passes, no stale-SHA false alarms (vs PAT-009 pattern
  from prior cycles). The verified-SHA dispatch (PROP-E18-02) appears effective.

**DF-VALIDATION-001 research agent:**
- maint-2026-07-08: correctly reviewed develop HEAD only; did not see fix-phase worktrees.
- No instances of research agent referencing implementation-phase artifacts.

**PG-MERGE-AUTH-SUBAGENT-CLASSIFIER:**
- The pr-manager subagent attempted merge after receiving authorization via teammate-
  message relay. The harness classifier's block was a CORRECT wall enforcement —
  merge authorization must not flow through agent relay chains. No wall integrity
  violation; the system worked correctly.

**Observation — wave-72 code-review artifact (PROP-MAINT-01 follow-through):**
- Wave-72 gate wrote `cycles/wave-72/wave-gate/code-review.md` with full enumeration
  of 5 MINOR + 4 NIT findings with dispositions. This is PG-W71-CODEREVIEW-ARTIFACT
  fix in action (STORY-158 AC-158-006). Wave-71 gap not repeated.

---

## 7. Quality Signal Analysis

### Holdout scores

| Wave | Mean | Min Must-Pass | Scenarios | Status |
|------|------|--------------|-----------|--------|
| Wave-70 | 0.920 | 0.80 | 15 TLS | PASS (gate ≥ 0.85) |
| Wave-71 | 0.970 | 0.80 (HS-062 fixture-limited) | 7 | PASS |
| Wave-72 | 1.000 | 1.00/16 | 16 | PERFECT |

**Wave-72 holdout at 1.000 perfect** — the highest score in any wave since feature-
protocol-coverage F4 holdout (D-374, also 1.00/10). This reflects the primarily
process/codification nature of wave-72 stories (STORY-158/159/160/161 are E-11 or
documentation/codification types with limited behavioral delta, except STORY-160's
JSON casing change which is fully specified).

**Holdout staleness (STORY-160 BREAKING change):**
- 13 holdout scenarios stale after STORY-160's JSON enum casing + schema_version change
- None detected during per-story delivery; all caught at wave gate
- Product-owner repaired HS-021/024/032/033/034/035/050/054/059/064/065/074/075
- Root cause: no delivery protocol step requires a holdout-expectation sweep for
  BREAKING stories before PR creation (Wave-72 Lesson 2)
- Post-repair verification: all 132 scenarios active/CLEAN in maint-2026-07-09 sweep 4

### Security

| Finding | Status | Notes |
|---------|--------|-------|
| SEC-W71-001 (CWE-22, bin/compute-input-hash) | FILED #392 | Path traversal; factory-artifacts write access required; LOW severity |
| SEC-W72-001 (CWE-200, tilde paths in demo scrub) | FIXED PR #391 | Fixed at wave-72 gate |
| SEC-W72-002, SEC-W72-003 | DEFERRED LOW | DF-VALIDATION-001-gated |
| SEC-010, SEC-011 (wave-70 carry) | RESOLVED PR #383 | test/bench-only guards added |

**No exploitable findings.** All resolved or correctly gated.

### Formal verification / mutation

- VP-039 (Kani, wave-71): spot-check 12/12 (existing proofs confirmed; no new proofs required in wave-71 scope)
- VP-041/042/043 (Kani, feature-protocol-coverage): ALL PROVEN in prior F6; not re-run this period
- Mutation kill rate: not explicitly measured this period (maintenance/codification scope; no new hot paths)
- maint-2026-07-09 pattern sweep: CLEAN (0 new `+=` sites; PF-001 resolution confirmed holding)

### Spec coherence trajectory

| Run | Pass/Fail | FAIL criteria |
|-----|-----------|--------------|
| maint-2026-07-06 | 3 FAIL | C8, C27, C29 |
| maint-2026-07-08 | 33/33 PASS | 0 FAIL |
| maint-2026-07-09 | 33/33 PASS | 0 FAIL |

Two consecutive 33/33 PASS results. The spec coherence criteria are holding after the
FIX-D burst from maint-2026-07-06 resolved the phantom VP-INDEX entries and BC-2.16.016.

### Test count

| Point in time | Test count |
|---------------|-----------|
| v0.9.0 release | ~1700 |
| wave-70 gate | 2367 |
| wave-71 gate | 2378 |
| wave-72 gate | 2392 |

Consistent upward trend (+25 per wave on average). No test regressions.

---

## 8. Pattern Detection (Cross-Run)

### New patterns identified this run

| ID | Name | Occurrences | Severity | Status |
|----|------|-------------|----------|--------|
| PAT-010 | wave-gate-fix-pr-recurring | 2/3 waves (W71 + W72) | MEDIUM | NEW — propose codification |
| PAT-011 | breaking-change-holdout-staleness | 1 (STORY-160 / 13 scenarios) | HIGH | NEW — codification candidate (Wave-72 Lesson 2) |
| PAT-012 | docs-agent-content-fabrication | 1 (maint-2026-07-09 Route A, 3 HIGH) | HIGH | NEW — adversary 3/3 required |
| PAT-013 | background-adversary-relay-unreliable | 2 incidents (maint-07-08 L-005) | MEDIUM | NEW — synchronous dispatch required |
| PAT-014 | subagent-merge-auth-relay-blocked | 1 (maint-07-09 PG-MERGE-AUTH-SUBAGENT-CLASSIFIER) | MEDIUM | NEW — STORY-163 AC-163-002 |

### Updates to existing patterns

| ID | Name | Prior Status | Update |
|----|------|-------------|--------|
| PAT-001 | pr-manager-shortstop | OPEN | Continues: all wave merges via orchestrator `gh` + explicit human auth relay; PAT-014 is a related but distinct incident |
| PAT-002 | doc-tense-recurrence | OPEN | No recurrences observed this period (wave-70/71/72 lessons silent on this axis) |
| PAT-009 | adversary-stale-git-ref-false-alarm | OPEN | ZERO false alarms this period; verified-SHA dispatch (PROP-E18-02) appears effective across wave-71 + wave-72; recommend advancing to RESOLVED-EFFECTIVE |

### Follow-through on prior review (session-review-maint-2026-07-06.md) recommendations

| Prior Item | Action Expected | Status |
|-----------|----------------|--------|
| PROP-MAINT-01 (incremental-write mandate) | Applied to sweep 8 dispatch | CONFIRMED APPLIED — Sweep 8 completed first-try in maint-2026-07-08 (vs 3 attempts prior) |
| PROP-MAINT-02 (canonical-ID re-read mandate) | Applied to sweep 8 dispatch | CONFIRMED APPLIED — No register ID corrections needed in maint-2026-07-08 (vs 1 correction prior) |
| PROP-MAINT-03 (spec-coherence reads artifacts) | Applied to sweep 7 dispatch | CONFIRMED APPLIED — No cross-report inconsistencies in maint-2026-07-08 (vs 1 prior) |
| PROP-MAINT-04 (deduplication check) | Applied to sweep 8 dispatch | APPLIED — No false-new pre-existing items surfaced maint-2026-07-08 |
| PROP-MAINT-05 (advisory-race triage annotation) | DEFERRED | No advisory races this period; still valid deferral |
| PROP-MAINT-06 (mechanism-verification for HIGH sweep-3) | DEFERRED | Not triggered this period; still valid deferral |
| PROP-MAINT-07 (import-style batch story) | DEFERRED to wave 71+ | NOT ACTED ON — wave-72 process-gap store is full; carry to next cycle |
| PROP-MAINT-08 (holdout freshness exact-key-count heuristic) | DEFERRED | Not triggered this period; Wave-72 Lesson 2 is related but broader |
| ASM-CAND-003 escalation threshold | ACCEPTED via PR #382 | RESOLVED — README Known Limitations section added |
| ASM-CAND-009 escalation threshold | ACCEPTED via PR #382 | RESOLVED |
| TD-MAINT-RISK-REGISTRY-BACKFILL (P1) | Formal registry required | RESOLVED (D-420) — risk-register.md + assumptions.md authored |
| ADV-4 OVERDUE (ci.yml build-dep-chain comment) | Human disposition required | NOT ADDRESSED — still open; carry forward as P2 |
| STORY-149 perf regression +14.0% | Verify at wave 70 | STORY-149 DELIVERED; TLS drain refactor (STORY-150) cleared +1.8% within noise; AC-149-003 INDETERMINATE (noise-suspect environment) |

**Score: 7/11 items resolved or confirmed effective. 2 still deferred (PROP-MAINT-07, PROP-MAINT-08). 1 not addressed (ADV-4). 1 open (AC-149-003 INDETERMINATE).**

---

## 9. Governance Policy Audit

**Existing policy enforcement check:**

| Policy | Enforcement Status | Notes |
|--------|--------------------|-------|
| `append_only_numbering` | PASS | No IDs renumbered; canonical IDs preserved throughout |
| `state_manager_runs_last` | PASS | All STATE.md updates followed artifact commits |
| `df_validation_001` | PASS | triage-2026-07-08: 10/10 validated before issue filing; maint-2026-07-08: all 11 items validated; SEC-W71-001 not filed until validated |
| `df_convergence_before_merge_001` | PASS | All PRs achieved 3/3 consecutive-clean before merge |
| `df_sibling_sweep_001` | PASS (with one recovery) | Wave-71 P4 catch: single-file fix would have missed siblings; grep-list remediation applied |
| `bc_h1_is_title_source_of_truth` | PASS | No BC title changes requiring cross-file cascade untracked |
| `vp_index_is_vp_catalog_source_of_truth` | PASS | VP-INDEX v2.35→v2.39; VP-024 re-lock triple-verified |
| `lift_invariants_to_bcs` | PASS | No orphan domain invariants introduced |
| `prd_is_source_of_truth_bc_authoring` | PASS | BC-2.11.036/037 authored from PRD v1.51 |

**New policy candidates from this period:**

1. **BREAKING-CHANGE-HOLDOUT-SWEEP-001** — Any story tagged BREAKING (or changing observable output format) MUST include a holdout-expectation sweep as an explicit delivery step before PR creation. Root: STORY-160 shipped 13 stale holdout expectations undetected. Recurrence threshold EXCEEDED at 1 (this is a high-severity gap).

2. **DOCS-ADVERSARY-3-PASS-001** — Any PR primarily touching README, --help, or operator-guidance text MUST use strict 3/3 adversarial convergence (same as code PRs). Root: maint-2026-07-09 Route A caught 3 HIGH fabrications. Precedent from maint-2026-07-08 L-002 (strict 3/3 for maintenance PRs touching operator text). Recommended to formalize as policy.

3. **SUBAGENT-MERGE-AUTH-MAIN-THREAD-001** — Merge authorization for any pr-manager sub-agent MUST originate in the main conversation thread. Relay via teammate-message is insufficient for harness classifier. Root: PG-MERGE-AUTH-SUBAGENT-CLASSIFIER; codified STORY-163 AC-163-002.

4. **BACKMERGE-FAST-FORWARD-001** — Back-merges from main to develop after a release MUST be fast-forward (no squash). Squash back-merges sever shared history and require a join commit before the next release. Root: PG-GITFLOW-SQUASH-BACKMERGE; operationally resolved in v0.12.0; STORY-163 candidate for formal codification.

---

## Improvement Proposals

### Proposal 1: BREAKING-change story delivery protocol — holdout-expectation sweep (PROP-V0.12.0-01)
- **Category:** workflow / delivery-protocol
- **Priority:** HIGH (P1)
- **Evidence:** STORY-160 BREAKING JSON change produced 13 stale holdout expectations; none caught during per-story delivery; all landed as unplanned gate work. Product-owner estimated significant repair effort at wave gate. (Wave-72 Lesson 2, PROC-OBS-W72-002)
- **Recommendation:** Add a mandatory checklist item to the BREAKING-change story template and the implementer delivery checklist: "Before opening the PR, run the holdout evaluator against the story's output changes. If any holdout scenario expectations are stale, repair them and include the repairs in the PR body." Marked `holdout-expectations-sweep: COMPLETE`. This is applicable to any story that changes observable output format (JSON schema, enum casing, text layout), not only explicitly BREAKING-tagged stories.
- **Affected files:** `.factory/templates/story-template.md` (implementer checklist section), possibly a dispatch reminder in the per-story-delivery orchestrator sequence
- **Risk:** Low. Adds a single pre-PR step; changes no test logic. The repair work already happens at gate time — this merely moves it earlier.

### Proposal 2: Strict 3/3 adversarial convergence for operator-guidance documentation PRs (PROP-V0.12.0-02)
- **Category:** convergence / policy
- **Priority:** HIGH (P1)
- **Evidence:** maint-2026-07-09 Route A (PR #393) caught 3 HIGH findings (fabricated vocabulary, fabricated ADR citation, inverted VLAN technical claim) under strict 3/3 convergence. maint-2026-07-08 L-002 caught 1 HIGH inverted ARP tuning advice under the same standard. Fabrication risk is high for docs agents with heavy context loads; 3/3 convergence is the only reliable catch mechanism.
- **Recommendation:** Formalize as policy DOCS-ADVERSARY-3-PASS-001: any PR primarily touching README, --help, or operator-guidance text MUST use strict 3/3 adversarial convergence. Add to maintenance-config.yaml fix-phase dispatch guidance and the per-story delivery checklist for any story with documentation-only deliverables.
- **Affected files:** `.factory/maintenance-config.yaml` (fix-phase dispatch template — already implicitly applied; formalize), `.factory/policies.yaml` (add DOCS-ADVERSARY-3-PASS-001)
- **Risk:** Low. Increases average adversary cost for documentation PRs by ~1-2 passes; prevents HIGH-severity user-facing defects.

### Proposal 3: Synchronous adversary dispatch as default for maintenance PRs (PROP-V0.12.0-03)
- **Category:** agent / dispatch-template
- **Priority:** MEDIUM (P2)
- **Evidence:** maint-2026-07-08 L-005: 2 incidents of background-dispatched adversary agents completing analysis but failing to emit reports. Both required synchronous re-dispatch. Background dispatch for adversary tasks with large output is unreliable.
- **Recommendation:** Add to maintenance-config.yaml adversary dispatch instructions: "Dispatch adversary agents with `run_in_background: false` for all maintenance-PR review passes. Background dispatch is acceptable only for clearly bounded tasks with short output." Also record ENGINE-NOTE ADVERSARY-RELAY-UNRELIABLE-001 as a known limitation.
- **Affected files:** `.factory/maintenance-config.yaml` (adversary dispatch section)
- **Risk:** Very low. Synchronous dispatch is functionally equivalent; the only cost is loss of parallelism, which is acceptable for single-PR convergence passes.

### Proposal 4: VHS tape templates — viewport-safe output patterns (PROP-V0.12.0-04)
- **Category:** tooling / demo-evidence
- **Priority:** MEDIUM (P2)
- **Evidence:** All 4 wave-71 VHS demo recordings failed with Wait+Screen timeout. Root cause: wirerust's unicode box-drawing output fills VHS 0.11.0's visible viewport before the match pattern scrolls in. Transcript fallback applied, but VHS recordings retain the value of being runnable/replayable. (Wave-71 Lesson 5)
- **Recommendation:** Update VHS tape templates to include: (a) a comment explaining the Wait+Screen viewport limitation for full-output commands; (b) a recommended mitigation pattern for wirerust commands (`| grep "key-pattern"` or `--json` with short-field `jq` extraction). Add this as a note to `.factory/maintenance/demo-evidence-scrub-gate.md` or a new `.factory/maintenance/vhs-recording-guidance.md`.
- **Affected files:** New `.factory/maintenance/vhs-recording-guidance.md`, existing VHS tape templates
- **Risk:** Very low. Documentation/template change; no behavioral changes.

### Proposal 5: E-11 story template — hook-divergence false-positive advisory note (PROP-V0.12.0-05)
- **Category:** template / tooling
- **Priority:** LOW (P3)
- **Evidence:** Wave-72 process-gap ledger F-W72-P1-005: E-11 stories use `inputs: []` producing hash `d41d8cd`; the bash hook's divergent algorithm (PG-HASH-HOOK-DIVERGENCE) may emit false-positive drift warnings. The interaction is undocumented in the E-11 template. (wave-72 process-gap-ledger.md)
- **Recommendation:** Add a note to the E-11 story template: "E-11 stories use `inputs: []`. The canonical Python tool produces `d41d8cd` (MD5 of empty bytes). Any hook discrepancy is a known false positive per PG-HASH-HOOK-DIVERGENCE — treat as advisory-only."
- **Affected files:** E-11 story template (if separate from the main template), CLAUDE.md Known Tool Divergences section
- **Risk:** Very low. Documentation only.

### Proposal 6: PAT-009 adversary-stale-git-ref — advance to RESOLVED-EFFECTIVE (PROP-V0.12.0-06)
- **Category:** pattern / convergence
- **Priority:** LOW (P3)
- **Evidence:** ZERO stale-SHA false alarms across wave-71 (7 passes) and wave-72 (4 passes). The confirmed mitigation (orchestrator supplies verified `git rev-parse HEAD` in adversary preamble; PROP-E18-02) has been applied without a single recurrence over 11 consecutive passes.
- **Recommendation:** Update PAT-009 status from OPEN to RESOLVED-EFFECTIVE. Continue the verified-SHA dispatch practice as standard. Remove PAT-009 from active monitoring concern.
- **Affected files:** `.factory/session-reviews/pattern-database.yaml` (PAT-009 status update)
- **Risk:** None. If a future false alarm occurs, re-open; the mitigation remains in place.

### Proposal 7: PROP-MAINT-07 carry-forward — PC-001/013/007/015 batch refactor story (PROP-V0.12.0-07 — DEFER)
- **Category:** story decomposition / tech-debt
- **Priority:** MEDIUM (P2) — carry from PROP-MAINT-07
- **Evidence:** Pattern-debt carrying forward since maint-2026-07-06; wave-72 pattern scan found 0 new findings (PF-001 resolved). PC-001/020 (DNP3/ENIP StreamHandler arch gap), PC-002/013 (import style), PC-007/015 (BTreeMap imports) remain open. The import-style items are low-blast-radius cosmetic.
- **Recommendation:** Draft a batch refactor story for PC-002/013 + PC-007/015 (import style) at wave-73+. Keep PC-001/020 as a separate story (architectural scope). This carry-forward is now 2 reviews old; schedule at wave-73.
- **Affected files:** New story file + STORY-INDEX update
- **Risk:** Low blast radius (import-style only; no behavioral changes).

---

## Metrics for Next Run

**From this run's proposals (if PROP-V0.12.0-01 applied before next BREAKING-change story):**
- Holdout repairs at wave gate for BREAKING-change stories: target 0 (baseline: 13 this run)
- Adversary finding rounds for docs PRs under 3/3 convergence: target ≤ 1 (baseline: 3 this run)

**Convergence targets:**
- Wave adversary passes to streak: target ≤ 4 (benchmark: W70=5, W71=7, W72=4; trend improving)
- Wave adversary NOT-CLEAN count: target ≤ 1 (benchmark: W70=2, W71=2, W72=1; improving)

**Quality targets:**
- Holdout mean ≥ 0.85 (sustained; last 3 waves: 0.920/0.970/1.000)
- Spec coherence 33/33 PASS (second consecutive; target third consecutive)
- Input-hash STALE=0 (second consecutive; target sustained)
- Test count ≥ 2392

**Carry-forward items for next maintenance run:**

| Item | Source | Priority |
|------|---------|----------|
| ADV-4 OVERDUE (ci.yml build-dep-chain comment) | tech-debt register | P2 — human disposition 2+ runs overdue |
| AC-149-003 INDETERMINATE | wave-70/71/72 | Standing — quiescent benchmark re-run needed |
| PROP-MAINT-07 carry (PC-001/013 import-style batch) | improvement-backlog | P2 — 2 reviews old |
| PROP-V0.12.0-01 (BREAKING holdout sweep) | this review | P1 — implement before next BREAKING story |
| SEC-W71-001 filed #392 | GitHub issue | P2 — CWE-22; fix before bin/compute-input-hash next touched |
| Route-C deferrals | maint-07-08 + 07-09 | LOW — HS-INDEX ENIP wave drift, epics.md BC tally, SC-PERSIST-001/002/003 |
| F-W72G-P3-OBS-002 MEDIUM (BC-tally 337 vs 347) | wave-72 process-gap-ledger | MEDIUM — pre-existing; route C |
| STORY-162/163 scheduling | STATE.md | P2 — wave-TBD drafts awaiting scheduling |
| ISSUE-102-PREMATURE-CLOSE-001 triage | STATE.md | P2 — DF-VALIDATION-001-gated |
| BACKMERGE-FAST-FORWARD-001 codification | PG-GITFLOW-SQUASH-BACKMERGE | P3 — STORY-163 candidate |
| Wave-72 process-gap-ledger items (12 deferred) | cycles/wave-72/process-gap-ledger.md | LOW/MINOR — next maintenance sweep |

*Session review authored by session-reviewer agent (2026-07-10, v0.12.0 release chain). Covers D-393..D-422, waves 70/71/72, maint-2026-07-08/09, triage-2026-07-08, v0.11.5 + v0.12.0 releases. Human decisions pending within 72h.*
