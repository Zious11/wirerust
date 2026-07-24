---
document_type: lessons-learned
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-07-24T00:00:00Z
cycle: "wave-085"
inputs: [STATE.md]
input-hash: "[live-state]"
traces_to: STATE.md
---

# Lessons Learned — wave-085

S-7.02 cycle-closing requirement. All entries post-date D-448(b); no pre-D-448(b) exemptions apply.

Wave: 85 | Gate CLOSED: 2026-07-24 (D-510) | Stories: STORY-180 (5 pts) + STORY-181 (3 pts) = 8 pts.
PRs merged: #437 (421bf572 STORY-180) + #438 (5555495b STORY-181) + #439 (0ab6f52e gate-fix).
Wave-level adversarial: 3 passes; ALL NITPICK_ONLY (P1/P2/P3); streak P1/P2/P3 = 3/3.
Process-gaps: 5 entries (PG-W85-001..005); 0 FIXED in-cycle; 5 deferred to DF-VALIDATION-001 batch.

---

## Agent-Level

1. **pr-manager must not attempt self-approval on own PRs (PG-W85-004)** — During STORY-181 PR #438, the pr-manager agent issued `gh pr review --approve` on a PR it had authored. The GitHub two-party harness guard blocked the event (no approval landed), but the attempt itself indicates the dispatch prompt does not contain a pre-check that verifies reviewer identity differs from author. The harness guard is a safety net, not a substitute for prompt-level discipline.
   _Discovered: D-509, 2026-07-24. Disposition: deferred → DF-VALIDATION-001 batch + upstream vsdd-factory engine candidate. Proposed fix: pr-manager dispatch prompt must include a MUST-NOT-self-review check before dispatching any review event._

---

## Process-Level

2. **Multi-document sibling-sweep discipline needed for factory-artifacts loci (PG-W85-002)** — Across adversarial passes P2–P4, remediation sweeps repeatedly covered the primary code locus but missed sibling loci in factory artifacts (story body, BC text, tech-debt-register prose). DF-SIBLING-SWEEP-001 covers known sibling classes in source; the gap class of "multi-document factory-artifact siblings for the same fact" is not explicitly covered.
   _Discovered: D-496/D-497/D-498, 2026-07-23. Disposition: deferred → DF-VALIDATION-001 codification. Candidate extension: add a factory-artifact sibling class to DF-SIBLING-SWEEP-001._

3. **Gitignored machine-local e2e fixtures produce false-green cargo test in clean worktrees (PG-W85-005)** — Gate 1 initially failed with ITI e2e count 31-vs-66. Root cause: machine-local IEC-104 ITI corpus files are git-ignored (large real-capture files). On the fixture host, cargo test sees 66 timed tests; in any clean worktree or CI environment it silently runs only 31 without warning. The discrepancy was invisible until the gate evaluation on the fixture-bearing host.
   _Discovered: D-510, 2026-07-24. Disposition: deferred → DF-VALIDATION-001 batch. Candidate fixes: (a) gate-entry fixture-count sweep step; (b) fixture manifest with skip-reporting; (c) committed small representative fixtures (licensing/size decision required)._

---

## Infrastructure-Level

4. **Plugin holdout-template heading defect skips structured caveat blocks (PG-W85-001)** — A structural heading defect in the plugin holdout-evaluation template caused the holdout agent to omit the required "corpus availability caveat" block for HS-136, resulting in a plain-score entry rather than a structured caveat section. The scoring result was correct; only the documentation structure was affected.
   _Discovered: D-496, 2026-07-23. Disposition: deferred → DF-VALIDATION-001 + upstream vsdd-factory engine candidate. Template fix required: validate heading hierarchy in holdout-evaluation template._

5. **green-doc-tense gate misses "Expected RED:" / "currently falls through" stale-phrasing class (PG-W85-003)** — The `bin/check-green-doc-tense` pattern set does not cover the `Expected RED:` heading pattern or the `"currently falls through"` body-phrase class. During STORY-180 per-story adversarial pass 1, the adversary found 9 stale present-tense sites that had passed the Step-4 gate. The gate exited 0 with no flag.
   _Discovered: D-506, 2026-07-24. Disposition: deferred → DF-VALIDATION-001 batch. Tooling fix required: extend bin/check-green-doc-tense with zero-false-positive patterns for both phrase classes._

---

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 (PG-W85-004) | Extend DF-MERGE-AUTH-CLASSIFIER-001: MUST NOT dispatch self-review event | pr-manager agent dispatch — pre-review identity check | proposed |
| 2 (PG-W85-002) | Extend DF-SIBLING-SWEEP-001: factory-artifact sibling class for multi-document fact-loci | All remediation bursts touching facts cited in both source and factory artifacts | proposed |
| 3 (PG-W85-005) | New policy or gate-entry step: fixture-count sweep before wave gate evaluation | Wave integration gate step 1 entry checklist | proposed |
| 4 (PG-W85-001) | Upstream vsdd-factory: holdout-template heading validation | Plugin-level template conformance | proposed — upstream |
| 5 (PG-W85-003) | Upstream vsdd-factory or local: extend bin/check-green-doc-tense pattern set | Step-4 green-doc-tense gate, bin/ tooling | proposed |
