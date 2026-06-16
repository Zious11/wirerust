---
document_type: session-review
cycle: feature-arp-v0.7.0
producer: session-reviewer (T1 read-only adversary model)
pipeline_path: feature-mode F1..F7 + release
released_version: v0.7.0
released_at: 2026-06-16
run_scope: "F4 holdout through F7 delta convergence + v0.7.0 release (resumed mid-F4 delta-impl complete)"
review_date: 2026-06-16
baseline_available: false  # first dedicated ARP-cycle session review
---

# Session Review — feature-arp-v0.7.0 (ARP Security Analyzer)

Post-pipeline analysis covering the resumed F4→F7 arc and v0.7.0 release for the ARP
Security Analyzer (E-16, issue #9). This review is read-only analysis against factory
artifacts; no artifacts were modified during review.

---

## Executive Summary

The ARP feature cycle delivered a high-confidence security analyzer through a genuinely
adversarial pipeline. The multi-lens defense-in-depth did exactly what it was designed
to do: different lenses caught different defects, including a CRITICAL security boundary
omission (D-077) that survived 4 adversary passes and holdout undetected. The quality
signals at release are strong — holdout mean 1.0, 98.9% mutation kill, 5/5 Kani
SUCCESSFUL, 16.2M fuzz executions / 0 crashes.

The session also generated a dense catalog of process inefficiencies. The pr-manager
agent has a 100% shortstop rate (6/6 PRs required orchestrator rescue), fix-induced
regressions turned a single LOW finding into 3 PRs and a MEDIUM defect, and doc-tense
recurrence persisted despite codification. These are addressable with targeted agent
and workflow changes. The following review quantifies each dimension and maps 12
improvement proposals to priorities.

---

## 1. Cost Analysis

**Data available:** Qualitative — no cost-summary.md with per-agent token counts for
this cycle (cost-summary.md was not populated for the ARP feature run). This dimension
is assessed from artifact volume and agent dispatch counts.

**Agent dispatch density (F4 convergence through release — estimated from burst-log and
commit trail):**

| Phase segment | Agents dispatched | PRs | Commits (develop) |
|---|---|---|---|
| F4 wave-level adversarial (3/3 + re-streak) | adversary x7, consistency-validator x3, research-agent x1, spec-writer x3, story-writer x2, implementer x2, pr-reviewer x2, pr-manager x6 | #242..#246 (5) | ~15 |
| F4 holdout + D-075/D-076/D-077 | holdout-evaluator, adversary x4, implementer x2, pr-reviewer x2, pr-manager x3 | #243..#245 (3) | ~10 |
| F5 scoped-adversarial (2 resets + 3/3) | adversary x5, spec-writer x4, story-writer x2, implementer x3, pr-reviewer x3, pr-manager x3 | #247..#249 (3) | ~10 |
| F6 formal hardening | kani-runner, fuzz-runner, mutant-runner, security-reviewer, pr-reviewer, pr-manager | #250..#251 (2) | ~5 |
| F7 convergence + consistency remediation | consistency-validator x4, holistic-adversary x1, spec-writer x2, story-writer x1, implementer x1 | (spec-only PRs) | ~12 |
| Release (release/0.7.0 branch, PR #256) | pr-manager, release | #256 (1) | ~3 |

**Most expensive segments (estimated):** F5 was the most agent-intensive segment due to
three streak resets, each requiring spec correction, story update, implementation, PR
review, and merge. The doc-only fix PRs (#244, #246, #251) each required full pr-manager
+ pr-reviewer cycles for 1-5 line changes — disproportionate overhead given
branch-protection requires a PR for every change.

**Efficiency concern:** The doc-tense fix loop (roughly 7 recurrences over the ARP
feature) each generated at least one extra adversarial pass restart + a fix PR. If each
restart costs ~2 agent dispatches and each doc-fix PR costs ~2 agent dispatches
(pr-reviewer + pr-manager), 7 recurrences = ~28 incremental agent dispatches that a
GREEN-commit doc sweep would have avoided.

**Self-review cost:** This session review is estimated at <5% of total pipeline run
cost given it requires only read operations over existing artifacts.

---

## 2. Timing Analysis

**Wall clock:** The full F4→F7→release arc was completed in approximately 2 days
(2026-06-15 to 2026-06-16). F6 alone was noted in context as running ~4.4h.

**Bottlenecks identified:**

1. **F5 streak resets — 2 resets adding ~3-4h.** O-A adjudication (LOW) drove 3 PRs
   and a MEDIUM regression. Each reset required: spec correction, story update, implement
   + test, pr-review, pr-manager (with shortstop rescue), state-manager commit. With
   the pr-manager shortstop adding ~1 manual intervention per PR, each F5 reset cycle
   took roughly 60-90 minutes.

2. **pr-manager shortstop — 6 manual interventions.** Each intervention requires the
   orchestrator to notice the shortstop, issue a "merge NOW" SendMessage, and wait for
   merge + CI confirmation. Conservatively 5-10 minutes per event, ~30-60 minutes total
   across the session. More significantly, it breaks the orchestrator's flow and adds
   cognitive overhead.

3. **F2/F3 convergence (pre-session) — 38 passes over 2+ days.** Not in scope of this
   session but establishes the pattern: the factory's adversary convergence loop is the
   longest-running component and dominates total wall clock for large spec corpora. The
   ARP cycle ran 71 total adversary passes across F2+F3 before reaching the F4 handoff.

4. **Branch-protection PR overhead for doc-only fixes.** Three PRs (#244, #246, #251)
   were exclusively doc fixes (1-5 lines). Each required a branch, PR creation, CI
   (~3-5 min), pr-reviewer dispatch, pr-manager dispatch (with shortstop), and merge
   confirmation. Estimated 20-30 minutes per doc-fix PR. These could not be batched
   given the sequential dependency on CI green status.

**Parallelization:** The F4 adversary re-streak passes explicitly ran sequentially for
fresh-context independence — this is correct protocol. The F7 consistency gaps were
found and remediated in batches. No obvious missed parallelization was detected in the
reviewed session.

---

## 3. Convergence Analysis

### F4 Wave-Level Adversarial

**Trajectory:** `Pass 1 (1 MEDIUM) → remediated D-074 → P1/3 CLEAN → P2/3 CLEAN → P3/3
CLEAN GATE (fee71ee) → [holdout D-075 + human-directed re-streak D-077 CRITICAL RESET]
→ Re-streak P1/3 CLEAN → P2/3 CLEAN → P3/3 CLEAN GATE (bcb1bd6)`

Two full streaks were required: 7 adversary passes total for wave-level convergence.
The second streak was human-directed after holdout surfaced a defect (D-075) and the
human identified a security boundary risk (D-077) that warranted deep probing. This is
a legitimate use of the re-streak mechanism — not a process failure.

**Key convergence insight:** D-077 (CRITICAL — half-implemented type-reject boundary)
was self-consistent across impl + unit tests + Kani harness + all prior adversary
passes. The fresh re-streak with explicit BC-completeness-sweep focus on the FULL
precondition set (including negative/reject branches) was the only mechanism that found
it. This validates the BC-completeness-sweep methodology for security-critical code.

### F4 Holdout Evaluation

**Score:** Initial 0.997, post-D-075 15/15 mean 1.0. G1 failure (D1 verdict field
value `Possible` vs `Likely`) was caught by the static adversary (3/3 scenarios) but
missed by the consistency-validator (checked structure, not field value). This is a
tool-complementarity finding: the consistency-validator is a structural checker, not a
value-semantics checker. The holdout scenario's static assertions are the right tool for
value-level verification.

### F5 Scoped Adversarial

**Trajectory:** `P1 CLEAN (O-A LOW obs) → P2 CLEAN → [D-078 + D-078b FIX RESET] →
F-1 MEDIUM (2d2fadf) → [F-1 FIX RESET] → P1/3 CLEAN → P2/3 CLEAN → P3/3 CLEAN
GATE (079013d)`

Three streak resets. First two from code changes (O-A fix + D-078b completion). Third
from F-1 fix-induced regression. Total: 5 adversary passes + 3 streak resets = 8
effective pass slots to reach the F5 gate.

**Concern:** F5 produced more total adversary passes than F4 wave-level convergence,
due entirely to the LOW→fix cascade. The O-A finding was legitimate but LOW severity.
The fix required hand-rolling offset logic that introduced a MEDIUM regression. A
valid alternative adjudication was available: document O-A as a known detection
quality gap with a deferred fix, avoiding the cascade entirely.

### F6 Formal Hardening

5/5 Kani SUCCESSFUL, 46/46 project-wide. Fuzz 16.2M/0. Mutation 98.9% (1 benign
missed by design). F6 formal hardening quality signals are excellent. No convergence
issues in F6.

### F7 Delta Convergence

5-dimensional convergence on first attempt. Consistency-validator found 4 gaps (VP-024
v2.3 residual + consuming-artifact drift) that the holistic adversary missed. This is
a recurring pattern — the holistic adversary operates at semantic level and misses
mechanical propagation drift. The consistency-validator is the correct tool for
symbol-propagation checking.

**Finding trend:** Within F7, the gaps were all propagation-class (symbol rename not
reaching all consumers, version not synced), not behavioral defects. This is the
expected convergence pattern for a mature feature — behavioral issues should be flushed
by F4-F5, with only mechanical propagation remaining for F7.

---

## 4. Agent Behavior Analysis

### pr-manager — CRITICAL DEFECT (6/6 shortstop)

The pr-manager agent stopped at step 6 (APPROVE) on every ARP-feature PR (#236, #238,
#239, #240, #241) plus a prior DNP3 F5 PR — 6/6 recurrences with 0 passes. The
proactive "DO NOT STOP AT APPROVE — execute steps 7-9" instruction injected into the
STORY-115 dispatch did NOT prevent recurrence #6. This confirms the defect is
structural in the agent's dispatch-protocol weighting, not addressable by
per-dispatch instruction injection.

**Assessment of the explicit-instruction mitigation:** The mitigation partially
addressed the cognitive framing issue but did not resolve the structural problem.
Steps 7-9 appear insufficiently weighted relative to the review loop in the agent's
understanding of its mandate. The explicit instruction may have reduced the depth of
shortstop (e.g., quicker to complete after orchestrator nudge) but did not achieve
self-completion.

**Verdict:** The explicit-instruction mitigation worked marginally (no regressions from
earlier to later PRs) but failed to achieve the target behavior (unsupervised
completion). The prompt needs structural change, not instruction addition.

### Implementer — Inverted-TDD instance (STORY-113)

The implementer modified production code (json.rs) to satisfy a mis-named test instead
of flagging the contradiction. This was caught by the orchestrator reading BCs before
adversary dispatch, not by any automated check. The error is a single instance with
contextual explanation (the test name strongly implied the production shape was wrong),
but it represents a known risk class in test-driven development.

### Test-writer / Implementer — doc-tense (HIGH recurrence, ~7x)

Every ARP story (STORY-111 through 115) had at least one doc-tense finding across its
adversary passes. The doc-tense pattern recurred after codification (DF-GREEN-DOC-TENSE-SWEEP
added to policies.yaml), demonstrating that policy-text addition alone does not change
agent behavior. This is the most clear-cut case in this cycle for agent-prompt-level
enforcement being necessary.

### Agents over-delivering beyond mandate

Two instances noted:
1. Demo-recorder committed binary demo artifacts to a develop-bound worktree branch
   (`.factory-demos/`) rather than factory-artifacts (`.factory/demo-evidence/`). Caught
   by pre-PR diff inspection; rolled back.
2. STORY-114 doc-sweep dispatch over-reached to 13 out-of-scope test files (prior-cycle
   doc debt). Rolled back; scope restored.

Both are dispatch-scope violations — the agents followed their logical interpretation of
the task (sweep for the pattern) without the scope constraint. This is a dispatch-template
deficiency, not an agent reasoning failure per se.

### Concurrent adversary independence

No confirmed wall leaks were detected in this session. The F4 re-streak passes were
explicitly run as independent fresh-context dispatches, with Pass 3/3 run solo. No
cross-referencing between passes was documented in the trajectory files.

One technical note: the git-ref staleness (packed-ref lag) caused an adversary to
report a stale SHA while correctly reviewing the right file contents. This was detected
and documented. It is an infrastructure artifact, not a wall integrity issue.

---

## 5. Gate Outcome Analysis

| Gate | First-try pass? | Notes |
|---|---|---|
| F4 wave-level adversarial Pass 1 | NO | D-074 MEDIUM found; reset |
| F4 wave-level adversarial 3/3 CLEAN | YES (fee71ee) | Then invalidated by post-convergence work |
| F4 Holdout | PARTIAL (0.997 → FIX → 1.0) | G1 D-075 verdict defect required fix |
| F4 adversary re-streak 3/3 CLEAN | YES (bcb1bd6) | After human-directed D-077 probe |
| F5 scoped adversarial | NO (3 resets) | O-A LOW → 2 streak resets; F-1 → 1 more |
| F6 Kani 5/5 | YES | No harness failures |
| F6 fuzz | YES | 16.2M/0 crashes |
| F6 mutation | YES (98.9%) | 1 benign MISSED by design |
| F7 5-dim convergence | YES | 4 consistency gaps found + fixed in same burst |
| Release v0.7.0 CI | YES | release.yml run 27645784901 SUCCESS |

**Overall first-try gate pass rate for this session:** 6/10 gates passed first-try
(60%). The 4 non-first-try gates all involved adversarial or holdout quality checks
finding genuine defects — this is correct pipeline behavior, not a process failure.
The F5 resets are the only case where alternative adjudication (document LOW vs fix)
might have improved efficiency without quality loss.

**Human override frequency:** One significant human override — the human directed a
full 3-pass RE-STREAK after the initial F4 gate satisfied (fee71ee). This was an
out-of-protocol action that found D-077 CRITICAL. The override was high-value and
appropriate given the security criticality of the extract_arp_frame type-reject
boundary. No human overrides overturned a quality signal downward.

---

## 6. Wall Integrity Analysis

**Overall assessment: INTACT**

The session review found no evidence of information asymmetry wall leaks in this
session. Specific checks:

- F4 holdout scenarios (HS-xxx wave-40-44) appeared to be used correctly as
  pre-specified scenarios, with the evaluator not having access to implementation
  rationale.
- F5 adversary passes were explicitly run as independent fresh-context dispatches with
  no context from prior passes documented in pass 2/3 reports.
- F7 consistency-validator operated independently from the holistic adversary and found
  different gaps — evidence of independent lens operation.

**Area for attention:** When concurrent same-type adversaries run (e.g., multiple
independent passes dispatched in rapid succession), the potential for context
cross-contamination through shared session state exists. The F4 re-streak Pass 3/3 was
explicitly run solo to address this. This practice should be documented as a protocol
requirement, not just a one-time precaution.

**Adversary vs. consistency-validator finding complementarity:** The holistic F7
adversary missed all 4 mechanical-propagation gaps that the fresh consistency-validator
found. This is not a wall failure — it is a tool-scope difference. Adversaries operate
at semantic/behavioral level; consistency-validators operate at symbol-propagation level.
Both are necessary; neither substitutes for the other.

---

## 7. Quality Signal Analysis

| Signal | Target | Actual | Assessment |
|---|---|---|---|
| Holdout mean satisfaction | >= 0.85 | 1.0 (post-D-075 fix, 15/15) | EXCELLENT |
| Mutation kill rate (ARP delta) | >= 95% (CRITICAL module) | 98.9% (1 benign MISSED by design) | PASS |
| Kani harnesses SUCCESSFUL | 5/5 (VP-024) | 5/5 (46/46 project-wide) | PASS |
| Fuzz crashes | 0 | 0 (16.2M execs) | PASS |
| Security findings | 0 exploitable | RUSTSEC-2026-0097 transitive BUILD-only | PASS |
| Spec coherence (F7 consistency) | CONSISTENT | CONSISTENT (post-remediation) | PASS |
| 1592 tests / 0 failures | GREEN | GREEN | PASS |

**Notable quality outcomes:**

The 5-dimensional F7 convergence passed cleanly with no behavioral defects remaining —
all 4 F7 gaps were propagation-class. This indicates the upstream quality filters
(F4 3/3 + holdout + re-streak + F5 3/3) successfully flushed behavioral issues before
formal hardening. The Kani proofs cover the security-critical parse-safety and LRU-bound
properties that are hardest to test exhaustively.

**F6 Sub-D surrogate note:** The 1 benign MISSED mutant (`<` vs `<=` tie-break in
the insert_binding_lru_array array surrogate) is an accepted design artifact of the
surrogate approach. It is out-of-scope for Kani by design and does not represent a
real production safety gap. The surrogate's correctness relative to the production
BTreeMap LRU is covered by the branch-fidelity test.

**The D-077 finding is the most significant quality signal of this cycle.** A CRITICAL
security boundary omission (type-reject validation on ARP frames) was self-consistent
across 4 independent adversary passes, holdout, and the Kani stub (which was a stub at
the time). Only the human-directed focused re-streak with explicit BC-completeness
protocol found it. This demonstrates the essential role of human judgment in directing
adversarial resources toward the highest-risk boundaries.

---

## 8. Pattern Detection (Cross-Run)

No pattern-database.yaml exists (first cycle with a session review). The following
observations establish baselines for future comparison:

**Pattern: pr-manager shortstop.** Recurred across DNP3 F5 (1x) and all 5 ARP-feature
story PRs. 6 total occurrences before this review. Baseline = 100% recurrence rate.
This is not a new finding at this point but it has not been fixed. The explicit-
instruction mitigation attempted in STORY-115 failed. The pattern requires structural
prompt change.

**Pattern: doc-tense recurrence.** Across ARP stories 111-115: 7 instances of
RED-gate/scaffold/stub prose surviving into adversary-surfaced GREEN commits. After
codification (DF-GREEN-DOC-TENSE-SWEEP), it recurred immediately in D-075 regression
test. Baseline = policy-text codification does not prevent recurrence; requires
agent-prompt / pre-commit enforcement.

**Pattern: fix-induced regression.** This cycle produced the first documented
LOW→FIX→MEDIUM cascade (O-A → D-078 → F-1). No baseline from prior cycles but the
pattern is now documented in PG-ARP-FIX-MECHANISM-FIRST.

**Pattern: consumer-sweep gaps after fixburst.** PG-ARP-FIXBURST-CONSUMER-SWEEP and
PG-CONSISTENCY-AUDIT-CONSUMER-SWEEP are two instances in this cycle alone of a rename
or lock not propagating to all consumers. DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 exists
but was not applied proactively. The pattern of reactive consumer-sweep (caught by
fresh consistency-validator) rather than proactive sweep is established.

**Pattern: holdout catching value-semantics defects.** D-075 (Verdict::Possible vs
Likely) was caught by holdout but missed by consistency-validator. This confirms the
complementary nature of the two tools. The static adversary scenarios within holdout
explicitly assert field values, which the consistency-validator does not do.

**Positive pattern: 3-pass multi-lens effectiveness.** The GARP-storm bypass (C1 in
STORY-115, missed by pass 1, found at pass 2/3) and D-077 (found by human-directed
re-streak) both provide concrete evidence that the 3-pass requirement catches real
structural gaps that single-pass review misses. These should be referenced when
justifying the 3-pass overhead to stakeholders.

---

## What Went Well

1. **Defense-in-depth delivered real value.** Four separate lenses caught four separate
   defect classes: F4 wave-level adversarial found D-074 (CLI degenerate-0), holdout
   found D-075 (Verdict value), human-directed re-streak found D-077 (CRITICAL type
   boundary), and F5 found F-1 (fix-induced VLAN regression). None of these was caught
   by the lens that came immediately before it in the pipeline. The multi-lens design
   is working.

2. **Holdout mean 1.0 is an exceptional result.** 15/15 scenarios at full satisfaction
   post-remediation. The ARP analyzer meets the RFC-826 canonical frame test and all
   15 adversarially-designed holdout scenarios.

3. **F6 quality signals are strong.** 5/5 Kani, 16.2M fuzz/0, 98.9% mutation kill.
   For a security-critical network analyzer, these are appropriate levels of formal
   assurance.

4. **The re-streak mechanism worked as designed.** After the initial F4 gate satisfied
   (fee71ee), the human correctly directed a focused re-streak on the highest-risk
   security boundary. D-077 CRITICAL was found. This is exactly the intended use of
   the human judgment gate between automated convergence phases.

5. **Process gaps were codified immediately.** DF-GREEN-DOC-TENSE-SWEEP was codified
   in policies.yaml before STORY-115 began. PG-ARP-FIX-MECHANISM-FIRST, PG-ARP-FIXBURST-
   CONSUMER-SWEEP, and other gaps were documented in lessons.md in real time. The
   factory's self-documentation is functioning.

6. **Release machinery worked.** PR #256, tag v0.7.0, release.yml 4 binaries — all
   on the first attempt without CI failures or release config issues.

7. **Research-agent validation caught the D-074 convention.** The threshold-0 rejection
   convention was validated HIGH confidence before the fix was implemented, ensuring the
   fix aligned with established CLI patterns across the codebase.

---

## What Was Costly / Inefficient

1. **pr-manager shortstop: 6 manual interventions @ 100% rate** — most consistent source
   of orchestrator friction in this session.

2. **O-A LOW → 3 PRs + MEDIUM regression (F-1)** — the fix for a LOW detection quality
   gap cost more effort than a MEDIUM defect would have, and introduced a MEDIUM defect.
   The fix-adjudication decision (FIX vs DOCUMENT) was made without assessing the
   fix-induced-regression risk.

3. **Doc-tense recurrence @ 7 instances** — each recurrence cost at least 1 adversary
   pass restart + 1 fix PR. Estimated ~28 extra agent dispatches and ~7 extra PRs across
   the ARP cycle.

4. **Spec written before mechanism verified (D-078)** — spec was written twice (BC
   v1.4→v1.5→v1.6) as the incorrect mechanism hypothesis ("lax builds slice") was
   discovered to be impossible. A 5-minute code probe before spec writing would have
   eliminated one correction cycle.

5. **Consumer-sweep gaps requiring reactive F7 discovery** — the VP-024 Sub-D rename
   and F6 lock did not propagate to all consumers. A proactive post-fixburst consumer
   sweep would have found these before F7.

6. **Doc-only PRs with full PR overhead** — PRs #244, #246, #251 were 1-5 line doc
   fixes that each required a full branch-PR-CI-review-merge cycle (~20-30 min each).
   Branch protection is non-negotiable but the overhead is real.

---

## Improvement Proposals

### PROP-01 [CRITICAL / agent] Structural fix for pr-manager shortstop

**Pattern:** PG-ARP-F4-PRMGR-MERGE-SHORTSTOP

**Evidence:** 6/6 (100%) recurrence across ARP + DNP3 F5 PRs. Proactive instruction
injection failed for STORY-115. Steps 7-9 (merge + confirm CI + consolidated report)
are not executing without orchestrator intervention.

**Root cause hypothesis:** The pr-manager agent interprets its terminal success state
as "APPROVE obtained" rather than "PR merged and CI confirmed." Steps 7-9 are deprioritized
relative to the review loop (steps 1-6) in the agent's dispatch-protocol weighting.

**Recommendation:** Restructure the pr-manager dispatch template to position merge
completion as the primary success criterion, not APPROVE. Specifically:

  (a) Move the success statement to read: "Your task is COMPLETE only when the PR
      has been MERGED to develop AND CI is confirmed GREEN. APPROVE is an intermediate
      state, not completion."

  (b) Add a blocking instruction: "DO NOT RETURN to orchestrator after step 6
      (APPROVE). Continue to step 7 (merge), step 8 (confirm CI green), step 9
      (consolidated report). If any step fails, report the failure — do not silently
      stop."

  (c) Require explicit "MERGED: <SHA>" in the agent's final response before the
      orchestrator considers the dispatch complete.

**Files affected:** vsdd-factory pr-manager dispatch template.

**Routing:** Agent prompt update. Requires DF-VALIDATION-001 research-agent validation
before filing as GitHub issue.

**Risk:** LOW — additive constraint, does not change review behavior.

**Priority:** P1 (CRITICAL, blocks every PR delivery; 100% recurrence rate)

---

### PROP-02 [HIGH / agent] Enforce doc-tense sweep at agent-prompt level, not policy-text level

**Pattern:** PG-ARP-F4-GREEN-DOC-TENSE, PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE

**Evidence:** 7 recurrences across ARP stories 111-115. Policy-text codification
(DF-GREEN-DOC-TENSE-SWEEP) added before STORY-115; recurred immediately in STORY-115
D-075 regression test.

**Root cause:** The policy exists in policies.yaml (a factory-artifact reference file)
but is not injected into the implementer/test-writer/stub-architect agent dispatches as
a live behavioral instruction. Agents don't read policies.yaml before committing.

**Recommendation:**

  (a) Inject DF-GREEN-DOC-TENSE-SWEEP as a literal checklist step in the implementer
      GREEN-commit dispatch template: "Before committing, grep the story diff for:
      scaffold | Red Gate | RED GATE | todo!() | stub | uncalled | stale counts.
      Every match must be replaced with accurate GREEN prose or explicitly deferred.
      This is BLOCKING — do not commit until the grep returns 0 matches on these
      patterns in story-scoped files."

  (b) Inject the REDTEST sub-rule into the test-writer dispatch: "Regression-test
      doc-comments MUST use regression-guard framing (present-tense accurate at GREEN
      time), not RED-gate framing ('currently fails', 'MUST FAIL until X'). Write
      these from the start, not as a post-commit correction."

  (c) Add a pre-commit hook (if hooks are feasible in the development environment)
      that greps new test functions for these patterns and emits a WARNING. This
      catches the pattern even if the agent ignores the dispatch instruction.

**Files affected:** Implementer dispatch template, test-writer dispatch template,
optionally a pre-commit hook configuration.

**Routing:** Agent prompt update + potential hook.

**Risk:** LOW-MEDIUM — may require tuning the grep patterns to avoid false positives
on legitimate "F6 deferred todo!()" annotations.

**Priority:** P1 (HIGH, 7 recurrences, each costing at least 1 adversary restart)

---

### PROP-03 [HIGH / workflow] LOW-finding risk-adjusted adjudication gate

**Pattern:** PG-ARP-FIX-MECHANISM-FIRST (D-F1 meta-lesson)

**Evidence:** O-A LOW finding (lax-arm D11 evasion) was adjudicated FIX without
assessing fix-induced-regression risk. The fix required hand-rolling offset logic
that correctly handled only baseline Ethernet2, missing VLAN. Cost: 3 PRs + MEDIUM
regression (F-1) + 2 extra F5 streak resets.

**Recommendation:** Add a LOW-finding adjudication step to the orchestrator's decision
playbook:

  "Before adjudicating a LOW finding as FIX, assess:
   (a) Does the fix require hand-rolling logic that the library already abstracts
       (e.g., offset computation, slice building)? If yes, treat as MEDIUM risk.
   (b) Does the fix touch a decode/parse path with protocol variant sensitivity
       (VLAN, QinQ, MACsec, extension headers)? If yes, require variant coverage
       test cases in the fix spec BEFORE implementation begins.
   (c) If both (a) and (b) are true, consider DOCUMENT (record as known limitation
       with a deferred FU issue) vs FIX. The fix-induced-regression risk may exceed
       the value of closing the LOW gap."

  Also: "Before writing the fix spec, READ the actual code path to verify the
  mechanism. Do not spec a fix based on a mechanism hypothesis without code verification."

**Files affected:** Orchestrator adjudication playbook / Cycle-Closing Checklist.

**Routing:** Orchestration pattern / workflow note.

**Risk:** LOW — informational gate, human still makes the decision.

**Priority:** P2 (HIGH — prevents fix-induced regression class)

---

### PROP-04 [HIGH / workflow] Proactive post-fixburst consumer-sweep mandate

**Pattern:** PG-ARP-FIXBURST-CONSUMER-SWEEP, PG-CONSISTENCY-AUDIT-CONSUMER-SWEEP

**Evidence:** Both the VP-024 v1.8 harness rename and the F6 lock failed to propagate
to all consuming artifacts. In both cases the gap was caught reactively by a fresh
consistency-validator, not proactively by the fix agent.

**Recommendation:** Extend DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 to be proactively
applied at the fix agent (spec-writer/implementer) level, not just discovered by
the consistency-validator:

  "Whenever a symbol name, version label, or canonical identifier changes in a BC,
  VP, or architecture document, the agent MUST, before closing the fix burst, grep
  the following artifact classes for the old symbol/version and update all hits:
  (1) All consuming BCs (.factory/specs/bc/)
  (2) All consuming stories (.factory/stories/)
  (3) verification-coverage-matrix.md
  (4) All consuming architecture deltas
  (5) All consuming VP files
  This sweep is BLOCKING — the fix burst is NOT complete until all consumers are
  updated."

**Files affected:** spec-writer dispatch template, DF-CONSISTENCY-AUDIT-POST-FIXBURST-001
policy text.

**Routing:** Agent prompt update + policy clarification.

**Risk:** LOW — additive sweep step.

**Priority:** P2 (HIGH — prevents F7 cleanup overhead for every feature cycle)

---

### PROP-05 [HIGH / agent] Pre-PR binary-leak diff check in orchestrator

**Pattern:** PG-ARP-F4-DEMO-LEAK

**Evidence:** Demo-recorder committed 4 gif+webm+tape artifacts to develop-bound
worktree branch (`.factory-demos/`) dodging `.factory/` gitignore. Caught by manual
pre-PR diff inspection.

**Recommendation:** Add an explicit pre-PR checklist step to the orchestrator's delivery
playbook:

  "Before dispatching pr-manager, run: `git -C <worktree> diff develop --name-only |
  grep -E '\\.(gif|webm|mp4|png|tape|bin|exe)$'`. If any binary artifacts appear on
  the diff, this is a HARD STOP — do not proceed to PR. Remove the binaries from the
  branch and verify `.gitignore` covers the pattern. Demo artifacts belong ONLY on
  factory-artifacts under `.factory/demo-evidence/`."

Also: Update demo-recorder dispatch template to target `.factory/demo-evidence/` on
the factory-artifacts worktree exclusively.

**Files affected:** Demo-recorder dispatch template, orchestrator pre-PR checklist.

**Routing:** Workflow/template update.

**Risk:** LOW — detection-only check.

**Priority:** P2 (HIGH — binary artifacts in develop would be a repository hygiene
failure)

---

### PROP-06 [MEDIUM / agent] Implement BC-completeness-sweep checklist for negative/reject branches

**Pattern:** PG-ARP-F4-TYPE-BRANCH-NARROWING

**Evidence:** D-077 (CRITICAL — type-reject boundary omission) survived 4 adversary
passes because impl + unit tests + Kani stub were all self-consistently incomplete.
A BC-completeness-sweep that explicitly targets EACH BC's FULL precondition set
(including negative/reject branches) is what eventually found it.

**Recommendation:** Extend DF-BC-COMPLETENESS-SWEEP-001 to make the negative-branch
check an enumerated required axis:

  "For each BC reviewed, the adversary MUST explicitly verify:
  (a) Happy-path behavior (PC1 etc.) — currently done
  (b) Error/reject branches (EC-001, EC-002, Err paths) — NEW explicit requirement
  (c) Boundary conditions (boundary preconditions, inclusive vs exclusive comparisons)
  (d) Type-field gates (hw type, proto type, link type) for protocol parsers

  For item (b): if a BC has EC-N rows, grep the implementation for all return Err paths
  and verify each maps to the correct EC. Missing Err paths (functions that should
  reject but don't) are the highest-risk omission class."

**Files affected:** DF-BC-COMPLETENESS-SWEEP-001 policy text, adversary dispatch template.

**Routing:** Policy text update + adversary dispatch template.

**Risk:** LOW — additive axis in existing sweep.

**Priority:** P2 (MEDIUM — prevents CRITICAL self-consistent omission class)

---

### PROP-07 [MEDIUM / agent] Mechanism-first verification before spec writing

**Pattern:** PG-ARP-FIX-MECHANISM-FIRST

**Evidence:** D-078 fix spec was written from an incorrect mechanism hypothesis ("lax
builds slice + extract None" — mechanically impossible). Spec required two correction
cycles (v1.4→v1.5→v1.6) as the correct mechanism was discovered during implementation.

**Recommendation:** Add to the fix-adjudication and spec-writing playbook:

  "Before writing a fix spec for any code path, the orchestrator or spec-writer MUST
  Read() the relevant code to verify the actual mechanism. Do not spec a fix from a
  mechanism hypothesis. Specifically:
  (a) For decoder/parse paths: Read the exact function body, not just the signature.
  (b) For library-delegating paths: Check which library API is actually called and
      what it returns — do not assume the return shape.
  (c) When fixing one arm of a match/if: Read ALL sibling arms in the same function
      and check whether the same fix is needed for each. Sweep ALL siblings before
      closing the fix burst."

**Files affected:** Orchestrator adjudication playbook, spec-writer dispatch template.

**Routing:** Workflow/template note.

**Risk:** LOW.

**Priority:** P2 (MEDIUM — prevents spec-correction cycles)

---

### PROP-08 [MEDIUM / agent] Implementer must not change production for a contradicting test

**Pattern:** PG-ARP-F4-INVERTED-TDD

**Evidence:** STORY-113 implementer modified json.rs to satisfy a mis-named test rather
than flagging the contradiction. Caught by orchestrator BC verification pre-adversary.

**Recommendation:** Add to implementer dispatch template:

  "If a failing test appears to require a production change that contradicts a declared
  BC or Invariant (e.g., Inv4 'no reporter changes in this story'), STOP. Do NOT
  change production code to match the test. Instead: (1) read the relevant BC, (2)
  determine whether the test or the spec is wrong, (3) surface the discrepancy to the
  orchestrator with: 'Test X appears to require production change Y, but BC-N.N.N
  Inv-K prohibits it. Is the test wrong or should the BC be updated?' The test is a
  defect candidate until the BC alignment is confirmed."

**Files affected:** Implementer dispatch template.

**Routing:** Agent prompt update.

**Risk:** LOW — additive check, doesn't change success-path behavior.

**Priority:** P3 (MEDIUM)

---

### PROP-09 [MEDIUM / workflow] Scope guard in all remediation dispatch templates

**Pattern:** PG-ARP-F4-DOCSWEEP-OVERREACH

**Evidence:** STORY-114 doc-sweep reached 13 out-of-scope files (prior-cycle debt).
Required a revert commit.

**Recommendation:** Add a universal scope header to all remediation and doc-sweep
dispatch templates:

  "SCOPE CONSTRAINT (BLOCKING): All file edits MUST be limited to files listed in
  `git diff develop..HEAD --name-only` for this story's worktree. Files outside this
  set are OUT OF SCOPE. If you encounter the same anti-pattern in an out-of-scope
  file, record it as a follow-up item (e.g., FU-REPO-WIDE-DOC-DEBT) but do NOT edit
  the file. Violating this constraint contaminates the story's convergence snapshot."

**Files affected:** Remediation dispatch template, doc-sweep dispatch template.

**Routing:** Template update.

**Risk:** LOW.

**Priority:** P3 (MEDIUM — prevents scope contamination and revert overhead)

---

### PROP-10 [MEDIUM / quality] Finding-emission ACs must assert on Finding object, not proxy counter

**Pattern:** PG-ARP-F4-PROXY-COUNTER-TEST

**Evidence:** AC-011 in STORY-113 was verified by asserting a proxy counter
(`malformed_findings` increment), which passed against an implementation emitting no
Finding. PR-reviewer caught the gap at STORY-113 delivery; not caught by adversary.

**Recommendation:** Add a test-quality axis to the adversary dispatch template for
finding-emission ACs:

  "For any AC that requires the implementation to emit a Finding: verify the test
  asserts on the Finding object itself — at minimum: (a) the finding appears in the
  output collection, (b) the confidence field has the expected value, (c) at least
  one evidence field is non-empty. A test that asserts only a count or proxy counter
  (e.g., `malformed_findings_count += 1`) without asserting the Finding shape is
  insufficient and MUST be flagged as MEDIUM."

**Files affected:** Adversary dispatch template, test-writer dispatch template.

**Routing:** Agent prompt update.

**Risk:** LOW.

**Priority:** P3 (MEDIUM)

---

### PROP-11 [LOW / quality] Cross-subsystem sibling sweep for shared decode-architecture invariants

**Pattern:** DF-SIBLING-SWEEP-CROSS-SS-001

**Evidence:** BC-2.02.009 v1.6 sibling-sweep missed BC-2.16.015 SS-16 (cross-subsystem
sibling with shared unreachable! pattern). Caught at F4 scoped re-review as a
HIGH would-be-panic.

**Recommendation:** Extend DF-SIBLING-SWEEP-001 with a cross-subsystem enumeration rule:

  "When a BC correction changes a decode-architecture invariant that is shared across
  subsystems (e.g., strict vs lax decode routing, unreachable! vs explicit-routing
  patterns, error-return conventions), the sibling sweep MUST include ALL BCs across
  ALL subsystems that reference the same source function or the same decode-architecture
  pattern — not only BCs in the originating subsystem. Identify cross-subsystem
  siblings via: grep .factory/specs/bc/ for the function name or invariant keyword."

**Files affected:** DF-SIBLING-SWEEP-001 policy text.

**Routing:** Policy text update.

**Risk:** LOW — additive enumeration step.

**Priority:** P4 (LOW)

---

### PROP-12 [LOW / pattern-db] Establish session-review infrastructure

**Pattern:** First-run observation — no pattern-database.yaml, benchmarks.yaml, or
session-reviews directory exists.

**Evidence:** This is the first dedicated session review for the wirerust factory.
No cross-run comparison is possible without prior baselines.

**Recommendation:** Create the `.factory/session-reviews/` directory structure with:
  - `pattern-database.yaml` seeded from this review's patterns (pr-manager shortstop,
    doc-tense recurrence, fix-induced regression, consumer-sweep gap)
  - `benchmarks.yaml` seeded with this cycle's quality signals (holdout mean 1.0,
    98.9% mutation kill, adversary passes per phase, gate first-try pass rate 60%)
  - `improvement-backlog.md` for proposals deferred from this review

**Files affected:** New files in `.factory/session-reviews/`.

**Routing:** State-manager writes after this review.

**Risk:** NONE — infrastructure only, no behavior change.

**Priority:** P4 (LOW — enables future cross-run analysis)

---

## Proposal Priority Summary

| ID | Category | Priority | Pattern | Warrant Story/Issue? |
|---|---|---|---|---|
| PROP-01 | agent (pr-manager) | P1 CRITICAL | PG-ARP-F4-PRMGR-MERGE-SHORTSTOP | YES — after DF-VALIDATION-001 research-agent validation |
| PROP-02 | agent (implementer/test-writer) | P1 HIGH | PG-ARP-F4-GREEN-DOC-TENSE-RECURRENCE | YES — agent-prompt update story |
| PROP-03 | workflow (adjudication) | P2 HIGH | PG-ARP-FIX-MECHANISM-FIRST | JUSTIFIED DEFERRAL — codify in Cycle-Closing Checklist, no standalone story needed |
| PROP-04 | workflow + agent | P2 HIGH | PG-ARP-FIXBURST-CONSUMER-SWEEP | YES — policy update + template change story |
| PROP-05 | workflow (orchestrator) | P2 HIGH | PG-ARP-F4-DEMO-LEAK | YES — orchestrator pre-PR check |
| PROP-06 | agent (adversary) | P2 MEDIUM | PG-ARP-F4-TYPE-BRANCH-NARROWING | YES — adversary dispatch template update |
| PROP-07 | agent + workflow | P2 MEDIUM | PG-ARP-FIX-MECHANISM-FIRST | JUSTIFIED DEFERRAL — folds into PROP-03 |
| PROP-08 | agent (implementer) | P3 MEDIUM | PG-ARP-F4-INVERTED-TDD | YES — implementer dispatch template |
| PROP-09 | workflow (templates) | P3 MEDIUM | PG-ARP-F4-DOCSWEEP-OVERREACH | YES — remediation template scope guard |
| PROP-10 | quality (adversary+test) | P3 MEDIUM | PG-ARP-F4-PROXY-COUNTER-TEST | YES — adversary dispatch template axis |
| PROP-11 | quality (policy) | P4 LOW | DF-SIBLING-SWEEP-CROSS-SS-001 | JUSTIFIED DEFERRAL — append to DF-SIBLING-SWEEP-001 at next policy review |
| PROP-12 | infrastructure | P4 LOW | (first-run) | NO — state-manager action |

---

## Justified Deferrals

- **PROP-03/PROP-07 (fix-mechanism-first):** The fix-adjudication playbook change is
  an orchestrator practice, not an agent prompt change. It should be added to the
  Cycle-Closing Checklist and internal orchestration notes. No standalone story required
  since it's process knowledge, not code.

- **PROP-11 (cross-subsystem sibling sweep):** Append to DF-SIBLING-SWEEP-001 at the
  next policy review cycle. Low urgency — the pattern is now understood and documented
  in lessons.md.

---

## Dimension Gaps

- **Cost Analysis (dimension 1):** Quantitative per-agent token cost data was not
  available (cost-summary.md not populated for this cycle). Cost analysis was qualitative
  only. Recommend populating cost-summary.md with token counts in future cycles to
  enable quantitative efficiency analysis.

- **F2/F3 convergence (pre-session scope):** This review covers the F4→release arc.
  The F2/F3 convergence (71 passes, 2+ days) was the longest phase but pre-dates the
  session scope. A separate review would be needed to analyze F2/F3 efficiency.
