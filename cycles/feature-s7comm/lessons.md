---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-09-07T02:15:00Z
cycle: "feature-s7comm"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Lessons Learned — feature-s7comm

<!-- Durable lessons from this cycle for future VSDD factory runs.
     Organized by category: agent-level, process-level, infrastructure-level.
     Each lesson is numbered continuously and includes the pass/burst
     where it was discovered. -->

## Agent-Level

_(none recorded this cycle)_

## Process-Level

1. **[process-gap] PG-CHECK-GREEN-DOC-TENSE-BLINDSPOT** — `bin/check-green-doc-tense` missed
   the "Every test … MUST FAIL" and "until the STORY-184 implementer delivers" RED-tense
   phrasings during STORY-184's per-story review (F-184-P1-003). These phrasings are
   semantically RED (describing not-yet-implemented behavior) but did not match any of the
   linter's existing TIER-1 patterns. This is a gate-coverage gap, not a one-off miss —
   the same phrase shapes are plausible in any future story's test-header prose. Candidate
   disposition: a self-improvement story adding these phrase shapes as new TIER-1 patterns,
   or a documented, justified deferral if the phrase class is judged too narrow to warrant a
   dedicated pattern. Per DF-VALIDATION-001, any GitHub issue filed from this finding requires
   research-agent validation first.
   _Discovered: STORY-184 per-story adversarial pass 1, 2026-09-06/07._

2. **[process-gap] PG-CANONICAL-HOLDOUT-NOT-AC-ENFORCED** — `DF-CANONICAL-FRAME-HOLDOUT-001`
   (the policy requiring an RFC-independent canonical-frame holdout test per parser story) is
   not enforced by any acceptance-criterion-level automated check. As a direct consequence,
   three defects escaped early detection during STORY-184: the missing RFC-independent holdout
   test itself, a §5/§6 RFC-section citation error, and the min-7-vs-4 minimum-length
   divergence (see `DEFERRED-BC-2.20.005-STALE-LEN4` in STATE.md Active Carry-Forwards) — none
   were caught until deep in per-story adversarial review, well after the story's initial TDD
   implementation. Candidate disposition: add an AC-level enforcement mechanism (a checklist
   item, a lint rule, or a story-template gate) that fails a story's Red Gate or Step-4.5 entry
   if no canonical-frame holdout test is present for a story that implements a wire-format
   parser. Per DF-VALIDATION-001, any GitHub issue filed from this finding requires
   research-agent validation first.
   _Discovered: STORY-184 per-story adversarial review (mid-story RFC-min-7 rework), 2026-09-06/07._

## Infrastructure-Level

_(none recorded this cycle)_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | Extend `bin/check-green-doc-tense` TIER-1 patterns with the "MUST FAIL" / "until the STORY-NNN implementer delivers" phrase shapes | Doc-tense gate coverage | proposed |
| 2 | AC-level enforcement of `DF-CANONICAL-FRAME-HOLDOUT-001` (Red Gate or Step-4.5 entry check for a canonical-frame holdout test on parser stories) | Story-template / gate discipline | proposed |
