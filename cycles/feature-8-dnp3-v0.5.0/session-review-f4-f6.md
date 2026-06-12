---
document_type: session-review
level: ops
version: "1.0"
status: complete
producer: session-reviewer
model: claude-sonnet-4-6 (adversary tier)
timestamp: 2026-06-12T00:00:00Z
cycle: feature-8-dnp3-v0.5.0
phases_covered: [feature-f4, feature-f5, feature-f6]
stories: [STORY-108, STORY-109, STORY-110]
prs_merged: ["#227", "#228", "#229", "#230", "#231"]
develop_head_at_review: a125c69
---

# Session Review — Feature #8 DNP3 F4/F5/F6
## Wave 37-39, Remediation, and Formal Hardening

_Produced post-pipeline; covers the session arc from STORY-108 (wave 37) delivery
through F6 formal hardening (PR #231 merged a125c69). Written from a fresh-context
adversary perspective — does not share context with the build agents._

---

## Executive Summary

Feature #8 F4 delivered 3 stories (STORY-108, 109, 110) via per-story TDD with
step-4.5 adversarial convergence. F5 scoped adversarial review found 2 P0 blockers
— one entirely unimplemented holdout feature, one latent protocol-bit bug — before
any rework was committed (via an agentic-sliced pre-implementation design review).
F6 formal hardening reached HARDENED: 9/9 Kani successful, 89% mutation kill, 3.19M
fuzz executions, 0 crashes. The session also surfaced 4 actionable process gaps that
have systemic implications beyond this feature.

**Top result:** F6 HARDENED is the correct exit state. The implementation is sound.

---

## 1. Convergence Metrics

### Per-Story Adversarial Passes (F4 Step-4.5)

| Story | Wave | Total Passes | Clean Streak | Genuine Functional Defects Found | BC Spec Bumps |
|-------|------|-------------|--------------|----------------------------------|---------------|
| STORY-106 | 35 | 7 | 3/3 (P5-P7) | 3 MAJOR (FC edge-cases, min-length guard, wrong transport bit mask) | VP-023 v1.4, BC-2.15.005 v1.2 |
| STORY-107 | 36 | 3 | 3/3 (P1-P3) | 1 (offline-pcap flows uncapped — accepted) | — |
| STORY-108 | 37 | 5 | 3/3 (P3-P5) | 2 (source_ip/timestamp None violation; master-resolution test vacuity) | — |
| STORY-109 | 38 | 13 | 3/3 (P11-P13) | 2 genuine functional defects (window-seed spurious-reset; T0827 emission gap F-P9-001) | BC-2.15.016 v1.2, BC-2.15.014 v1.6 |
| STORY-110 | 39 | 6 | 3/3 (P4-P6) | 1 (phantom test/Kani citation) | — |

**Total F4 adversarial passes:** 34 across 5 stories.
**Mean passes to convergence:** 6.8 (median: 6). STORY-109 is a significant outlier
at 13 passes — attributable to genuine complexity (two detection paths, one spec
evolution mid-delivery, two regression fixes for prior stories).

**Genuine functional defects found by per-story adversarial:** 8 across 5 stories.
Per-story adversarial is working as intended for within-story correctness.

### F5 Scoped Adversarial (Holistic Feature Review)

| Finding | Severity | Type | Caught By |
|---------|----------|------|-----------|
| F-A-001: DIR-bit mask 0x10 vs 0x80 (latent since STORY-107) | P0 BLOCKER | Pre-existing protocol bug | F5 agentic-sliced pre-impl design review |
| F-F5-001: Unexpected-source detection entirely absent | P0 BLOCKER | Missing whole feature | F5 holistic delta review |
| F-F5-002: IcsImpact/Impact display collision | MEDIUM | Display defect | F5 holistic delta review |
| F-F5-003: Resync arm silent data-loss (Crain-Sistrunk evasion) | HIGH | Security evasion vector | F5 holistic delta review |

Both P0 BLOCKERs would have caused holdout scenario failures (HS-W37-002: unexpected
source; any master-frame-dependent detection: DIR-bit). Neither was catchable by
per-story adversarial because the error was internally self-consistent — the tests and
the BCs agreed with each other while both being wrong about the protocol specification.

### F5 Convergence Trajectory

| Pass | Result | Notes |
|------|--------|-------|
| Pre-impl slice A | 2 DESIGN BLOCKERs (F-A-001, F-B-002) | Caught before a line of code was written — ARCHITECT REVISION-2 required |
| P1-P5 | BLOCKED | Design corrections propagating |
| P6 | CLEAN 1/3 | — |
| P7 | BLOCKED | Regression |
| P8 | CLEAN 2/3 | — |
| P9 | [INVALID — CHECKOUT GUARD FAILURE] | Adversary reviewed develop not worktree; 3 false BLOCKERs |
| P10 | CLEAN 3/3 — CONVERGED | — |

**Effective passes:** 10 total (1 invalid). Convergence took longer than median due
to the scale of the design changes and the P9 checkout-guard failure wasting one pass.

### F6 Formal Hardening Quality Signals

| Signal | Result | Target | Status |
|--------|--------|--------|--------|
| Kani proof runs | 9/9 SUCCESSFUL | 9 | PASS |
| Kani counterexamples | 0 | 0 | PASS |
| Mutation kill rate | 89% (91.8% incl. timeouts) | >85% | PASS |
| Logic-critical mutant survivors | 0 | 0 | PASS |
| Fuzz executions | 3.19M | >1M | PASS |
| Fuzz crashes | 0 | 0 | PASS |
| Regression tests | 1495 green | all | PASS |
| Clippy | CLEAN | CLEAN | PASS |

F6 verdict: HARDENED. All 4 F6 obligations satisfied including the new obligation
from F5 (VP-023 re-validation under corrected 0x80 mask). No counterexamples found,
confirming the DIR-bit fix introduced no proof regression.

---

## 2. Holistic vs. Per-Story Review — Value Comparison

This session provides the clearest possible evidence of the value differential
between per-story adversarial convergence and holistic/feature-level review.

**34 per-story adversarial passes** caught 8 genuine within-story defects. None of
the 34 passes found:
- The DIR-bit mask bug (latent since STORY-107)
- The entirely absent unexpected-source detection (BC-2.15.010 Invariant 5)
- The Crain-Sistrunk resync evasion vector

**1 F5 holistic review pass** (plus 1 agentic-sliced pre-impl design review slice)
caught all four F5 issues including both P0 BLOCKERs.

The root cause of the per-story blind spot is structural: per-story passes review
code against its own BC and its own tests. When the BC and the tests share the same
wrong assumption (e.g., wrong bitmask; "this detection exists" when it does not),
the adversary has no external reference to detect the error. The only mechanisms that
can catch this class of defect are:

1. A holdout scenario written from the protocol spec (not derived from the story BC)
   using canonical byte sequences that exercise the invariant independently.
2. A holistic/feature-level review that checks the BC corpus for completeness (is
   every BC Invariant actually implemented somewhere?) rather than checking each story's
   implementation against its own BC.
3. A fresh-context adversary with access to the authoritative protocol specification
   (IEEE 1815 DNP3) — not just the project's BC corpus.

The F5 pre-impl slice review caught the DIR-bit bug by doing exactly this: reading
the protocol bit layout independently and cross-checking against the code, without
being anchored to the project's existing BC text.

**Quantified cost of the latent DIR-bit bug:** the bug was present for approximately
30 adversarial passes (STORY-107 through STORY-110 and into early F5). Had it reached
holdout evaluation, it would have produced a P0 failure requiring a full
re-implementation and re-convergence cycle. The pre-impl design review caught it at
the lowest possible intervention cost.

---

## 3. Agent Behavior Analysis

### 3a. Per-Story Adversarial Agent — Behavior (6 Process Observations)

**PG-F4-F5-001: Per-story adversarial has a structural blind spot for protocol-spec
level correctness.**

Evidence: DIR-bit mask (0x10 vs 0x80) survived 30 passes. Root cause: adversary
reviews code against project BCs and tests; cannot detect errors that are internally
self-consistent between code + BC + tests. No tier violation; the agent did its
assigned job correctly. This is a scope/methodology gap, not an agent failure.

Status of codification: partially captured in F5-remediation/lessons.md (PG-F5-DIRBIT-001).
Not yet in policies.yaml as a formal protocol-invariant rule. Recommendation: add
a rule requiring at least one canonical-spec holdout per protocol framing invariant.

---

**PG-F4-F5-002: BC-set completeness (is every BC Invariant implemented?) is not
verified by per-story adversarial.**

Evidence: BC-2.15.010 Invariant 5 (unexpected-source detection) was never implemented.
The adversary for STORY-108 reviewed `detect_control_class_burst_split` against
BC-2.15.010 and found the burst path correct — it did not check whether the Invariant
5 path existed at all. This is a within-scope correctness review; checking for entirely
absent features requires a completeness audit at the BC level, not the story level.

Status of codification: NOT yet codified. Recommendation: F5 holistic review MUST
include a BC completeness sweep — for each BC that specifies a detection invariant,
confirm the corresponding branch/function exists in the code (not just that the
existing code satisfies the BC). File as a mandatory F5 checklist item.

---

**PG-F4-F5-003: Checkout-guard failure (P9) produced 3 false BLOCKERs.**

Evidence: F5 convergence pass P9 reviewed develop HEAD (pre-fix) instead of the
feature worktree (post-fix). Its BLOCKER findings described code that the worktree
had already corrected. The orchestrator caught the error by cross-checking the file
paths the adversary cited; P10 (re-run on correct worktree) returned CLEAN.

Impact: 1 wasted convergence pass. No merge-before-converge risk (the correct worktree
was eventually reviewed). However, had the orchestrator not cross-checked paths, P9
would have triggered unnecessary rework and added another pass cycle.

Status of codification: captured in F5-remediation/lessons.md (PG-F5-CHECKOUT-001).
DF-ADVERSARY-CHECKOUT-GUARD-001 policy exists. Later passes implemented the guard
explicitly and it worked. The gap is in initial dispatch — the guard must be built into
the adversary dispatch template, not added post-hoc when a failure is suspected.

---

**PG-F4-F5-004: CI Format gate failures from direct-commit of agent-authored code
without cargo fmt.**

Evidence: CI Format gate failed twice (STORY-109 PR #228, F6 PR #231) because the
orchestrator committed agent-authored test changes via direct git, bypassing the
step where the authoring agent would have run `cargo fmt`. The authoring agent runs
`cargo fmt` as part of its delivery step; the direct-commit path skips this.

Impact: extra CI cycle per occurrence; no correctness impact (format-only). Frequency:
2/5 PRs (40%) hit this issue in this session. This is a predictable recurring failure
whenever the orchestrator takes code from an agent's output and commits it directly.

Status of codification: NOT yet in policies.yaml. Recommendation: add a rule:
"when committing agent-authored code via direct git (not via the authoring agent's
own commit), the orchestrator MUST run `cargo fmt --check` or `cargo fmt` before the
commit. Failure to do so WILL produce a CI Format gate failure."

---

**PG-F4-F5-005: pr-manager returned only the security review to the orchestrator.**

Evidence: Across multiple PRs in this session, pr-manager surfaced the security review
verdict but did not include: the pr-reviewer approval/rejection verdict, the PR number,
or CI status. The orchestrator was required to dispatch pr-reviewer separately and query
CI status manually for each PR (#227, #228, #229, #230).

Impact: extra orchestrator round-trips per story delivery. In a 5-story feature with
multiple PRs, this compounds. DF-PR-MANAGER-COMPLETE-001 already exists in
policies.yaml (added in a prior session), which means pr-manager is not compliant with
an existing policy.

Status of codification: policy DF-PR-MANAGER-COMPLETE-001 exists but pr-manager is
not honoring it. This is an agent-prompt gap, not a process gap. Recommendation:
the pr-manager agent prompt must be updated to emit a consolidated report (PR number,
pr-reviewer verdict, security verdict, CI job status, merge commit) as its final
output — in that order, every time.

---

**PG-F4-F5-006: Input-hash blast radius when a shared spec file is edited mid-cycle.**

Evidence: BC-2.15.016 was bumped (v1.2, v1.3) mid-delivery to accommodate spec
evolution during STORY-109 and the DIR-bit fix. Each bump cascaded an input-hash
STALE on every story whose `inputs:` list includes BC-2.15.016. A full `--scan` at
STORY-109 delivery showed MATCH=62/STALE=0 only after manual regen of 5+ stories.
The PRD edit associated with F5 cascaded across ~57 stories with PRD as a shared input.

Impact: no correctness issue (the regen mechanism works), but the blast radius is
large when a widely-shared input (PRD, core BCs) changes. Each regen requires a
factory-artifacts commit; with ~57 stories affected this is a significant mechanical
overhead.

Status of codification: DRIFT-F2-COUNT-001 and general input-hash cascade patterns
are acknowledged but not codified as an operational procedure. Recommendation:
document a "shared-input edit procedure" that batches all story hash regens into a
single commit when a widely-shared input file (PRD, core-spec BC) is updated.

---

### 3b. Pre-Implementation Agentic-Sliced Design Review — Pattern Confirmation

The agentic-sliced pre-implementation adversarial review (3 parallel slices: F-001
design, F-003 design, spec-consistency) caught 2 design BLOCKERs before any code
was written:

- F-A-001: DIR-bit mask 0x10 → 0x80 (blocked the entire F-001 feature from working)
- F-B-002: Overflow `clear+return` data-loss path in the proposed F-003 design (DoS/
  evasion vector in the resync accounting)

Both required ARCHITECT REVISION-2 directives before TDD began. Without this step,
the implementation would have been built on flawed designs, discovered the bugs mid-
convergence (P4-P7 typically), required rework, and likely needed 5+ additional passes.

**Estimated savings:** 5-8 adversarial passes (approximately 1-2 hours of pipeline
time). Actual cost: 1 extra pre-impl review step (approximately 20 minutes).

**Pattern conclusion:** Pre-implementation adversarial design review is high-leverage
for stories touching: protocol framing invariants, multi-path detection logic,
security-invariant BCs (T1692.001/T0827 class), or any design where the proposed
algorithm has non-obvious ordering dependencies. The benefit:cost ratio is strongly
positive.

---

### 3c. Formal Verification Agent (Kani) — Behavior

The Kani verification agent performed correctly. 9/9 harnesses successful including
the re-validation run after the DIR-bit fix (F6 obligation 4). The DIR-bit fix
(`is_master_frame` is effectful shell, not inside VP-023 pure-core) produced no
proof regression. The F6 agent correctly identified that the pure-core harnesses
are insulated from the mask change.

Mutation testing correctly found 0 logic-critical survivors. Survivor #6
(window-seeding gap) was killed by a new unit test authored in PR #231 —
appropriate escalation path.

---

## 4. Gate Outcome Analysis

| Gate | Story/Phase | Outcome | Notes |
|------|-------------|---------|-------|
| Red Gate | STORY-106 | PASS 1st try | 32 tests red as expected |
| Step-4.5 convergence | STORY-106 | 7 passes (longer than typical) | 3 MAJOR issues in P1 |
| Step-4.5 convergence | STORY-107 | 3 passes | Smoothest story in this session |
| Step-4.5 convergence | STORY-108 | 5 passes | BC violation + test vacuity caught |
| Step-4.5 convergence | STORY-109 | 13 passes | Outlier — 2 functional defects + spec evolution |
| Step-4.5 convergence | STORY-110 | 6 passes | VP-004 oracle/production sync: CLEAN throughout |
| F5 holistic adversarial | Feature #8 delta | 2 P0 BLOCKERs, 2 HIGH/MEDIUM | Both BLOCKERs caught by pre-impl design review |
| F5 convergence | F5 remediation | 10 passes (1 invalid P9) | CONVERGED at P10 |
| F6 Kani | VP-023 ×4 + VP-007 ×4 + VP-004 ×1 | 9/9 SUCCESSFUL | 0 counterexamples; 0 regressions |
| F6 mutation | DNP3 delta | 89% kill; 0 logic survivors | PASS |
| F6 fuzz | fuzz_dnp3_parse | 3.19M / 0 crashes | PASS |
| CI | #227, #228, #229, #230, #231 | All GREEN (after fmt fixes) | 2 PRs required fmt fix before CI passed |
| Security | All PRs | APPROVE_WITH_NOTES (no CRITICAL/HIGH) | DRIFT-SEMGREP-001 recorded (non-blocking) |

**Gate first-try rate:** 3/5 stories converged within or at median pass count (≤6).
STORY-109 significantly exceeded median (13 passes) due to genuine complexity. F5
convergence took 10 passes including 1 wasted (P9 checkout failure).

**Human override count:** 0 in F4-F6. All gate outcomes were accepted as-is.

**Human corrections:** 2 (both fmt-related CI failures where the orchestrator had
to re-push after running cargo fmt; not a correctness reversal).

---

## 5. Wall Integrity Analysis

**Per-story adversarial passes:** adversary walls held. Each story adversary received
fresh context; no cross-story leakage of prior findings detected in convergence records.

**F5 pre-impl slice review:** 3 parallel slices reviewed independent aspects of the
design without sharing context with each other. Two BLOCKERs were found in separate
slices (F-001 design slice caught DIR-bit; F-003 design slice caught overflow data-
loss). No evidence of cross-slice contamination.

**P9 checkout-guard failure** was a wall-type failure: the adversary reviewed the wrong
artifact (develop instead of the feature worktree). This is not an information-asymmetry
wall failure but a context-loading failure — the adversary anchored to the wrong codebase.
The DF-ADVERSARY-CHECKOUT-GUARD-001 policy addresses this by requiring explicit worktree
HEAD attestation at pass start. The policy was honored in later passes (P10 returned
CLEAN after explicit re-dispatch on the correct worktree).

**Holdout independence:** the F5 findings were generated before any implementation
began (for the P0 pre-impl slice) and from a fresh holistic review without reference
to per-story adversary findings. Holdout scenarios (HS-W37-002) were cross-validated
against the architect adjudication independently. No wall leak detected.

**Assessment: walls held with one process exception (P9 checkout error, not an asymmetry
violation).**

---

## 6. Quality Signal Analysis

| Signal | Value | Interpretation |
|--------|-------|----------------|
| Step-4.5 functional defect rate | 1.6 genuine functional defects / story (8 total / 5 stories) | Healthy — adversary is finding real issues |
| F5 P0 blocker rate | 2 P0 BLOCKERs / feature | Higher than ideal — indicates per-story scope is insufficient for feature-level completeness |
| F5 security evasion vector | 1 (F-F5-003 resync Crain-Sistrunk) | Caught before release — correct outcome |
| Kani counterexamples | 0 | Strong signal: pure-core arithmetic is correct |
| Mutation kill rate | 89% (0 logic survivors) | Acceptable; above 85% threshold |
| Fuzz crashes | 0 @ 3.19M execs | Strong signal: parse path is panic-free |
| Test count at delivery | 1495 | Growing (was 1126 at Phase 7 greenfield) |
| CI Format failures | 2/5 PRs (40%) | Elevated — direct-commit-without-fmt pattern is recurring |

**Holdout pre-validation note:** Phase 4 holdout evaluation has not yet run for
Feature #8 (F7 delta convergence is NEXT). The F5 adversarial review caught the
P0 holdout blocker (HS-W37-002) before holdout execution — which is the intended
preventive function of F5. Holdout results remain to be observed at F7.

---

## 7. Cost and Timing Summary

_Precise token counts are not available in the artifact corpus. The following is
an evidence-based qualitative assessment._

**Highest-cost story:** STORY-109 (13 adversarial passes). Contributing factors:
two genuine functional defects requiring spec evolution plus two regression fixes
for prior stories. The per-story adversarial agent did its job — the pass count
reflects real complexity, not process inefficiency.

**Highest-leverage cost ratio:** F5 pre-impl agentic-sliced design review. One
additional step (3 slices) caught 2 design BLOCKERs before any code was written,
preventing an estimated 5-8 additional convergence passes that would have been
needed post-implementation. This is the session's clearest cost-efficiency win.

**Wasted cost:** P9 checkout-guard failure consumed 1 full convergence pass
producing findings against the wrong codebase. With a checkout guard in place,
this pass would either have been caught at attestation time (saving the full pass)
or would have been a legitimate CLEAN finding.

**Recurring CI cost:** 2 Format gate failures (PRs #228, #231) required re-push.
Each adds a CI cycle (approximately 5-10 minutes). Trivially prevented by running
`cargo fmt` before direct-committing agent output.

**SEMGREP absence:** DRIFT-SEMGREP-001 records that semgrep is not installed on
the build host. F6 security relied on manual reviews (all CLEAN). Manual security
review is more expensive in time than automated SAST. Installing semgrep eliminates
this gap at low cost.

---

## 8. Pattern Detection (Cross-Run)

Comparing this session against the prior lessons.md entries (PG-5, PG-7, PG-8
from the F2/F3 session):

**Recurring pattern — partial-fix propagation (PG-8):** Also present in F5, where
BC-2.15.010 v1.5 required separate title-sync passes (P3-P4 BLOCKED on cascading
title-sync gaps). The grep-to-exhaustion discipline reduces but does not eliminate
this pattern. It recurs whenever there are many co-change targets.

**New pattern — protocol-spec vs. project-spec divergence:** This session introduces
a new category not present in prior lessons: a protocol-level correctness error
(wrong bitmask from the DNP3 spec) that was invisible to project-internal review
but immediately visible to spec cross-validation. This pattern is specific to
protocol analyzers and will recur with each new protocol implemented.

**Improving trend — VP-004 oracle/production sync:** The STORY-105 lesson (VP-004
oracle and production classify must be updated atomically) was correctly applied
to STORY-110. VP-004 oracle/production sync was CLEAN throughout all 6 STORY-110
passes. Lesson codification is working.

**Improving trend — pr-review dispatch:** pr-reviewer was dispatched correctly
for each PR, but required manual orchestrator dispatch due to pr-manager not
returning consolidated output (PG-F4-F5-005). The correctness outcome was correct
(all PRs approved before merge); the efficiency gap remains.

**Stable pattern — holdout pre-screening by F5 holistic review:** This is the
third feature-level cycle and the F5 holistic review has caught P0 issues
before holdout every time. The pattern is working as designed.

---

## 9. Prioritized Improvement Proposals

### IP-1 (CRITICAL / Immediate): Mandatory Canonical-Frame Holdout for Protocol Framing Invariants

**Category:** Process rule + holdout authorship requirement
**Priority:** P0 — prevents the single most expensive class of latent defect
**Evidence:** DIR-bit mask bug survived 30+ per-story adversarial passes. Would have
caused P0 holdout failure. Caught only by cross-validating against IEEE 1815 DNP3 spec.

**Recommendation:** Add the following to the F3 story-decomposition checklist AND to
the holdout-scenario authorship rules:

> "For any protocol-framing invariant (direction bits, function codes, frame types,
> magic bytes), at least one holdout scenario MUST use a canonical byte sequence from
> the authoritative protocol specification (e.g., IEEE 1815 for DNP3, Modbus
> Application Protocol spec for Modbus) — NOT a value derived from the project's own
> BC. This cross-validates the implementation against the external spec, not against
> itself. The holdout scenario must explicitly cite the spec section and the canonical
> byte value. Stories that implement protocol-framing invariants MUST include this
> requirement in their AC."

**Affected artifacts:** holdout-scenario template, F3 decomposition checklist, story
authorship rules.
**Risk:** Low (additive rule; no existing process changed).
**Recommendation for codification:** Add as a mandatory item in
`.factory/policies.yaml` under a new policy `DF-CANONICAL-FRAME-HOLDOUT-001`.

---

### IP-2 (HIGH / Before next protocol feature): F5 BC-Completeness Sweep as Mandatory Step

**Category:** F5 holistic review checklist addition
**Priority:** P1 — prevents "absent whole feature" class of P0 holdout failure
**Evidence:** BC-2.15.010 Invariant 5 was entirely unimplemented. Per-story adversarial
cannot detect absent features (it reviews what exists). F5 holistic review found it.

**Recommendation:** Add a mandatory first step to every F5 scoped adversarial review:

> "BC-Completeness Sweep: for each BC in the feature's spec that specifies a detection
> invariant, guard condition, or emission requirement, confirm that a corresponding
> implementation path (function, branch, or condition) EXISTS in the feature delta.
> The adversary must grep the implementation for the BC ID or the invariant's key
> variable name. A BC invariant with no corresponding implementation path is a P0
> BLOCKER finding, not a MEDIUM finding."

**Affected artifacts:** F5 adversarial review template, DF-ADVERSARY-METHODOLOGY-001
scope expansion.
**Risk:** Low (additive checklist step; adds ~10% to F5 review time).
**Recommendation for codification:** Expand DF-ADVERSARY-METHODOLOGY-001 to include
the completeness sweep as a numbered step. This is a policy-prompt change (not a
new policy).

---

### IP-3 (HIGH / Before next convergence cycle): Pre-Implementation Adversarial Design
Review as Default Step for High-Risk Features

**Category:** Process step addition (F5 remediation entry)
**Priority:** P1 — proved its value in this session (caught 2 design BLOCKERs before
coding); risk-adjusted ROI is strongly positive for protocol/security features
**Evidence:** Agentic-sliced pre-impl review saved estimated 5-8 passes. Cost: 1 extra
step. Both BLOCKERs (DIR-bit, overflow data-loss) required REVISION-2 before TDD.

**Recommendation:** Codify as a default step in the F5 remediation playbook:

> "When F5 holistic review identifies issues touching: (a) protocol framing invariants,
> (b) detection logic with security-invariant BCs, or (c) multi-path byte-walk
> algorithms, dispatch an agentic-sliced pre-implementation adversarial design review
> (2-4 slices, one per concern area) BEFORE TDD authoring begins. Each slice receives:
> the adjudication directive, the relevant BC(s), and the authoritative protocol spec
> section. This review checks the DESIGN for soundness, not the implementation. Any
> BLOCKER finding triggers ARCHITECT REVISION before coding begins."

**Affected artifacts:** F5 playbook, orchestrator dispatch script.
**Risk:** Low (additive step; does not change existing convergence protocol).
**Recommendation for codification:** Add to cycles/ F5-remediation template as
Step 0 with trigger conditions.

---

### IP-4 (HIGH / Immediate, recurring): Checkout-Guard as Mandatory First Step in
All Worktree Adversarial Dispatches

**Category:** Agent dispatch protocol / policy enforcement
**Priority:** P1 — P9 false BLOCKERs cost 1 full convergence pass; if uncaught could
have triggered unnecessary rework
**Evidence:** F5 P9 reviewed develop instead of feature worktree. Existing policy
DF-ADVERSARY-CHECKOUT-GUARD-001 was not enforced in the initial dispatch template.

**Recommendation:** The orchestrator's adversarial dispatch template must include a
mandatory checkout-guard block as the first agent instruction in every worktree-based
review pass:

```
CHECKOUT GUARD (mandatory — complete before any analysis):
1. Run: git -C <worktree_path> log -1 --format='%H %s'
2. Confirm the HEAD SHA matches the expected worktree HEAD (provided by orchestrator).
3. Spot-check: read the file that contained the most recently fixed BLOCKER finding.
   Confirm that the fix is present (cite the line you checked).
4. If either check fails: STOP. Report "CHECKOUT GUARD FAILED — invalid context."
   Do NOT produce findings. The pass is invalid and must be re-dispatched.
```

This is already in DF-ADVERSARY-CHECKOUT-GUARD-001. The gap is the dispatch template
not including it. Policy enforcement is only as strong as its injection into the
dispatch prompt.

**Affected artifacts:** orchestrator dispatch template for Step-4.5 worktree reviews,
F5 remediation worktree review dispatch.
**Risk:** None (the guard prevents invalid passes; it does not change valid ones).
**Recommendation for codification:** The policy text already exists. Action: update
the adversarial dispatch template to embed the checkout-guard block verbatim.

---

### IP-5 (MEDIUM / Next PR cycle): pr-manager Consolidated Report Compliance

**Category:** Agent prompt fix (pr-manager)
**Priority:** P2 — creates recurring orchestrator overhead; policy already exists
**Evidence:** pr-manager returned only the security review on multiple PRs (#227, #228,
#229, #230). Orchestrator dispatched pr-reviewer separately each time. DF-PR-MANAGER-
COMPLETE-001 requires a consolidated report.

**Recommendation:** Update the pr-manager agent prompt to output a consolidated
report in this format as the mandatory final section of every run:

```
## Consolidated Gate Report
- PR number: #NNN
- pr-reviewer verdict: [APPROVE / REQUEST_CHANGES / REJECT]
- security verdict: [APPROVE_WITH_NOTES / APPROVE / REJECT] (N CRITICAL, N HIGH, N MED, N LOW)
- CI status: [GREEN / FAILING] (job list with pass/fail per job)
- Merge commit: <SHA> (if merged)
- Blocking items: <list or "none">
```

This is a prompt change, not a process change. The agent is not complying with an
existing policy (DF-PR-MANAGER-COMPLETE-001). Prompt update is the fix.

**Affected artifacts:** pr-manager agent prompt.
**Risk:** None.
**Recommendation for codification:** Update pr-manager prompt. Flag as a self-
improvement story candidate if the prompt lives in a versioned template file.

---

### IP-6 (LOW / Deferred): SEMGREP Installation and Automated SAST Integration

**Category:** Toolchain
**Priority:** P3 — non-blocking; manual reviews are covering the gap
**Evidence:** DRIFT-SEMGREP-001 (F6 hardening record). Security reviews in F4/F5/F6
relied entirely on manual review (all CLEAN). Semgrep is not installed on the build host.

**Recommendation:** Install semgrep as part of the CI toolchain. Add a non-blocking
semgrep step to the F6 hardening playbook. Promote to blocking once a clean baseline
is established.

**Affected artifacts:** CI workflow, F6 playbook.
**Risk:** Low (start as non-blocking/informational gate).

---

### IP-7 (LOW / Before next shared-input edit): Shared-Input Edit Procedure

**Category:** Operational procedure
**Priority:** P3 — no correctness impact; reduces mechanical overhead
**Evidence:** PRD edit cascaded input-hash STALE to ~57 stories. BC-2.15.016 bumps
cascaded to 5+ stories. Each regen requires factory-artifacts commits.

**Recommendation:** Document a "shared-input edit procedure" in the factory ops guide:
when editing a file that appears in many stories' `inputs:` lists (PRD, core BCs),
run `bin/compute-input-hash --write --scan` in a single batch AFTER all edits are
complete (not after each individual edit), and commit all regens in one factory-
artifacts commit with a clear message describing the shared-input change.

**Affected artifacts:** factory ops guide, state-manager instructions.
**Risk:** None (operational procedure change only).

---

## 10. Summary of What Worked Well

These items are NOT improvement proposals — they worked correctly and should be
explicitly noted as strengths to preserve:

1. **3-consecutive-CLEAN convergence gate (BC-5.39.001):** Caught genuine functional
   defects in every story (8 total). The gate is calibrated correctly.

2. **Architect adjudication for design disputes:** Two ARCHITECT REVISION-2 directives
   were required in F5. The adjudication mechanism resolved ambiguous design decisions
   cleanly, preventing implementation stalls.

3. **F6 formal verification scope:** Kani harnesses for VP-023 (parse-safety) and
   VP-007 (catalog) provided independent verification of the pure-core functions.
   The F6 obligation to re-validate Kani under the corrected DIR-bit mask was correctly
   added and correctly satisfied.

4. **VP-004 oracle/production sync lesson applied correctly:** STORY-110 applied the
   STORY-105 lesson (oracle and production must update atomically in one commit).
   VP-004 oracle sync was CLEAN throughout all 6 STORY-110 passes. Lesson codification
   is working as intended.

5. **DF-CONVERGENCE-BEFORE-MERGE-001 compliance:** No story was merged before reaching
   3 consecutive CLEAN passes. F5 remediation was not merged until P10 CLEAN (despite
   the pressure of 4 issues to fix). The policy held.

6. **F5 holistic review as P0 catchment:** For the third feature cycle, F5 holistic
   review caught issues that per-story adversarial cannot. The two-tier review model
   (per-story for correctness; holistic for completeness and emergent issues) is
   working as designed.

---

## Appendix A — Convergence Pass Counts Summary

| Phase | Story/Scope | Passes | CLEAN streak position |
|-------|------------|--------|-----------------------|
| F4 Step-4.5 | STORY-106 | 7 | P5-P7 |
| F4 Step-4.5 | STORY-107 | 3 | P1-P3 |
| F4 Step-4.5 | STORY-108 | 5 | P3-P5 |
| F4 Step-4.5 | STORY-109 | 13 | P11-P13 |
| F4 Step-4.5 | STORY-110 | 6 | P4-P6 |
| F5 pre-impl design | F-001/F-003 slices | 1 (3 parallel) | N/A (design-only) |
| F5 scoped adversarial | Feature #8 delta | 10 (1 invalid P9) | P6, P8, P10 |
| F6 Kani | 9 harnesses | 1 run | N/A (formal proof) |

**Session total adversarial passes:** 50 (34 F4 + 1 pre-impl + 10 F5 + 5 for STORY-106
which spans F4 start).

---

## Appendix B — BC Spec Versions Changed This Session

| BC | Version at Session Start | Version at F6 COMPLETE | Change |
|----|--------------------------|------------------------|--------|
| BC-2.15.005 | v1.1 | v1.2 | 0x00 CONFIRM→Management locked |
| BC-2.15.009 | v1.1 | v1.2 | Prose clarification (F5) |
| BC-2.15.010 | v1.2 | v1.5 | EC-009/010/011 + dual-gate H1 (unexpected-source); mask correction |
| BC-2.15.014 | v1.5 | v1.6 | T1691.001 evidence format canonicalized |
| BC-2.15.016 | v1.1 | v1.3 | EC-007 resync carry-forward (v1.2); PC5 mask 0x10→0x80 (v1.3) |
| BC-2.15.024 | v1.2 | v1.3 | Three-path resync accounting (F5) |
| VP-023 | draft v1.3 | locked v1.5 | Verification lock at e685664 |
| VP-004 | locked (Rules 1-5) | relocked (Rules 1-6) | Rules 5/6 added |
| VP-007 | draft | locked | Kani Sub-B/C/D 4/4 SUCCESSFUL |

---

_Session review complete. NEXT: F7 delta convergence → v0.6.0._
