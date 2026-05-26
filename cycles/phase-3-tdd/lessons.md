---
document_type: lessons
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-05-25T00:00:00Z
cycle: phase-3-tdd
traces_to: STATE.md
---

# Lessons Learned — Phase 3 TDD Implementation

Lessons captured per-wave as convergence completes. Tags:
- `[codified]` — follow-up story or structural fix identified and tracked
- `[deferred]` — explicit target release noted; not yet actioned

---

## Wave 7 Lessons (2026-05-25)

### L-W7.1 — No Public-API Surface Gate [deferred — v0.1.0-release]

**Finding ID:** W7.1 (Drift Items table, STATE.md)
**Category:** CI / process-gap
**Observed:** Wave 7 pass-1 process-gap, confirmed pass-2 and pass-4. Two `#[doc(hidden)] pub fn`
test-seam accessors (`isn_missing_warned_for_testing`, `isn_missing_warned`) landed in PR #120
without any CI gate raising a surface-diff alert. No `cargo public-api` or `cargo semver-checks`
job exists in `.github/workflows/ci.yml`.
**Impact:** The addition was intentional and documented via ADR-0004 amendment (PR #121), but the
absence of a gate means future accidental pub-fn additions would go undetected until adversarial
review or manual inspection.
**Target:** v0.1.0-release cycle-close — evaluate `cargo public-api` snapshot job vs. a CLAUDE.md
callout requiring explicit PR annotation for any pub-fn addition.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — v0.1.0-release]

---

### L-W7.2 — Partial-Fix Regression Discipline Recurrence [deferred — Wave 8 cycle-close]

**Finding ID:** W7.2 (Drift Items table, STATE.md)
**Category:** process-discipline / adversarial-workflow
**Observed:** Wave 7 saw partial-fix regression at passes 2, 3, 4, and 5:
- Pass-2: sibling BC anchor missed while fixing the directly-flagged BC
- Pass-3: mod.rs cites missed while sweeping flow.rs
- Pass-4: closing-brace boundary and semantic correspondence missed while fixing start-line
- Pass-5: sibling row missed in HS-014 fix
Each "comprehensive" remediation was too narrowly scoped; the next fresh-context adversary found
the same class of issue in adjacent positions.
**Rule already in place:** every remediation MUST sweep the same axis across the entire affected
surface, not just the directly-flagged item. Wave 7 affirms that this rule is not consistently
followed when context is fresh after a large sweep.
**Recommendation:** remediation bursts should include an explicit checklist step: "have I checked
all instances of the same axis type (same BC file pattern, same mod.rs structure, same anchor
format) across the entire affected module?" before declaring the fix complete.
**Target:** Wave 8 cycle-close — evaluate whether this can be encoded as a pre-remediation
checklist row in the adversarial-review skill or as a state-manager checklist step.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — Wave 8 cycle-close]

---

### L-W7.3 — Out-of-Scope Anchor Drift (src/analyzer + src/decoder) [deferred — Wave 9+]

**Finding ID:** W7.3 (Drift Items table, STATE.md)
**Category:** spec-anchor hygiene
**Observed:** The Wave 7 mega-sweep (commit 6db1772) identified anchor drift in src/analyzer and
src/decoder modules. These were explicitly out of Wave 7 scope (STORY-014 only touches
src/reassembly). The drift exists but is not actively blocking any current story.
**Risk:** If Wave 9+ stories touch src/analyzer (e.g., analyzer subsystem stories STORY-041+),
the stale anchors will surface as adversarial findings in that wave, requiring a remediation burst
at that point. Proactive sweep would prevent this.
**Target:** Wave 9+ — when src/analyzer is next a primary implementation target. If analyzer
stories arrive before Wave 9, reschedule accordingly.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — Wave 9+ (when src/analyzer is next touched)]

---

### L-W4.1-recurrence-4 — Src-Edit-Before-Anchor-Derivation Discipline [deferred — v0.1.0-release]

**Finding ID:** W4.1 (Drift Items table, STATE.md; recurrence #4 in Wave 7)
**Category:** anchor-derivation workflow / process-discipline
**Observed (Wave 7 recurrence):** Wave 7 pass-4 mega-sweep at 6db1772 verified function-start
lines but missed (a) closing-brace boundary correctness and (b) description-vs-line-range semantic
correspondence. The anchor agent computed offsets correctly for the first line of each function
but did not verify that the last line of the anchor range was still accurate after editing.
**Prior recurrences:** Wave 4 (original W4.1), Wave 6 (TcpFlow::fin_count() +7 shift), Wave 7
(mega-sweep missed closing-brace and semantic checks) — recurrence #4.
**Pattern:** The existing rule ("src edits commit first, then anchor agents re-read from disk") is
being followed for start-line derivation but NOT for end-line verification or description
semantic accuracy. The gap is in the sweep validator's coverage, not in agent ordering.
**Codification candidates:**
1. Enhanced anchor-validation pre-commit hook that checks both start and end line of each range
2. Scripted spec-anchor sweep tool that verifies (a) start-line matches function signature,
   (b) end-line matches closing brace, (c) description text matches function purpose
**Target:** v0.1.0-release — four-wave recurrence justifies investing in tooling before the
analyzer/decoder waves (Wave 9+) which will have far more anchors to maintain.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — v0.1.0-release (codification priority raised by recurrence #4)]

---

## Wave 8 Lessons (2026-05-26)

### L-W8.1 — Stale Local Develop Causes Spurious Adversary Findings [deferred — Wave 9 cycle-open]

**Finding ID:** W8.1 (Drift Items table, STATE.md)
**Category:** process-discipline / adversarial-workflow
**Observed:** Wave 8 wave-level pass-3 produced FALSE-POSITIVE F-1/F-2 HIGH findings. Root cause:
local develop branch was 1 commit behind origin when the adversary was dispatched — the adversary
read stale source code and flagged behavior already corrected in the missing commit.
**Impact:** Wasted a full adversary pass + remediation investigation cycle. The false positives
were not identified until careful cross-referencing of the PR merge history.
**Recommendation:** Orchestrator MUST run `git pull origin develop --ff-only` (cwd:
`/Users/zious/Documents/GITHUB/wirerust`) BEFORE every wave-level adversary dispatch. Alternatively,
the adversary agent itself should sync at session start. This is a P1 structural gap — the W1.1
pre-check recommendation covers per-story dispatches but was not consistently applied at wave-level.
**Target:** Wave 9 cycle-open — enforce as a pre-dispatch gate step before any adversary is launched.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — Wave 9 cycle-open / pre-dispatch gate enforcement]

---

### L-W8.2 — ADR Amendment Dialect Drift [deferred — v0.1.0-release]

**Finding ID:** W8.2 (Drift Items table, STATE.md)
**Category:** spec-anchor hygiene / ADR compliance
**Observed:** Wave 8 wave-level pass-2 found 3 source comments in code delivered by STORY-019
PR #122 referencing "(choice (b))" vocabulary — a design-choice framing that ADR-0004 amendments
(PR #124) never enumerated. ADR-0004 uses "opt-in per-guard" prose, not "(choice (a))/(choice (b))"
framing. The implementer invented their own vocabulary; the ADR-amendment process did not catch it
because the amendment was authored post-implementation.
**Impact:** Spec-comment mismatch; adversary correctly flagged as a doctrinal accuracy gap. Required
a chore PR (#125) to align source comments with ADR vocabulary.
**Recommendation:** Enforcement-mode reviews of any new source comments that reference ADR doctrines
should verify the cited vocabulary exists verbatim in the ADR. ADR amendments must be drafted or
reviewed alongside the implementation PR, not after merge.
**Target:** v0.1.0-release — consider adding a CLAUDE.md callout requiring ADR-reference source
comments to be verified against the cited ADR text.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — v0.1.0-release]

---

### L-W8.3 — Wave-Level Adversarial Cost Escalation [deferred — Wave 9 cycle-close]

**Finding ID:** W8.3 (Drift Items table, STATE.md)
**Category:** process-discipline / adversarial-workflow
**Observed:** Wave 8 wave-level required 9 passes before achieving a 3-clean streak, vs Wave 7's 8.
Each pass through passes 1-6 routinely found 1-2 findings, typically a partial-fix regression of
the prior pass's fix. The pattern: each remediation fixed the directly-flagged item but missed an
adjacent position (sibling BC, sibling section, sibling comment, sibling enum arm).
**Root cause analysis:** Likely (b) — the W7.2 partial-fix regression discipline is genuinely
systemic and needs structural enforcement, not pass-by-pass detection and remediation. The wave
being larger (2 stories + 3 chore PRs vs 1 story) amplifies the effect but doesn't explain the
underlying mechanics.
**Codification candidate:** "Pass-N+1 must NOT find issues already partially-addressed in pass-N's
remediation" — this is a testable invariant. One implementation: before dispatching pass N+1, the
adversary should be given pass-N's finding IDs and told to verify each was FULLY resolved (all
instances of the same axis type), not just the cited one.
**Target:** Wave 9 cycle-close — evaluate structural mechanism for partial-fix regression detection.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — Wave 9 cycle-close]

---

### L-W8.4 — W7.2 Partial-Fix Regression Discipline Recurrence in Wave 8 [deferred — cycle-close codification]

**Finding ID:** W8.4 (Drift Items table, STATE.md; recurrence of W7.2)
**Category:** process-discipline / adversarial-workflow
**Observed:** The W7.2 partial-fix regression pattern recurred specifically in Wave 8 adversarial
passes:
- Pass-1: sibling-BC enforcement-mode propagation gap — fixed the flagged BC but sibling BCs with
  the same enforcement-mode field were not updated to match
- Pass-4: within-BC sibling-section propagation gap — fixed the PC body but ECs, Canonical Test
  Vectors, and Verification Properties sections were not updated to reflect the same factual reality
- Pass-5: ADR-amendment narrative inaccuracy — the amendment prose described the implementation
  correctly but the linked source comments used non-ADR vocabulary (see L-W8.2)
- Pass-6: BC↔test correspondence gap — test names and assertions did not reflect the final BC
  version after all amendments
**Pattern confirmed:** The rule "sweep the same axis across the entire affected surface" is
consistently violated when a remediation burst is scoped to the single flagged item.
**Codification target:** A pre-commit or pre-merge sibling-discipline checklist for any BC update.
The checklist should verify that ALL sibling sections of the updated BC (Description, Pre-conditions,
ECs, Canonical Test Vectors, Verification Properties) reflect the same factual reality as the PC
body, AND that all other BCs in the same subsystem sharing the same enforcement-mode field have
been reviewed for consistency.
**Target:** cycle-close codification — this is the second wave with confirmed recurrence (W7 + W8),
justifying encoding as a mandatory checklist step rather than a best-practice reminder.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — cycle-close codification; target: before Wave 10 dispatch]

---

## Wave 9 Lessons (2026-05-26)

### W9.L1 — Wave-Level Adversarial as Distinct Convergence Loop [codified — confirmed Wave 9]

**Finding ID:** Wave 9 retrospective observation
**Category:** adversarial-workflow / process-validation
**Observed:** Wave 9 wave-level adversarial passes 1/2/3 surfaced INTEGRATION findings that per-story
passes could not surface: cross-BC sibling-regressions where fixing STORY-016 or STORY-020 in
isolation looked complete but the joint invariant test coverage had a gap; joint invariant assertions
not written because each per-story adversary only reviewed its own story's test file; BC↔story↔test
propagation gaps (PC-7 in BC-2.04.015 had no corresponding AC trace in STORY-020; BC-2.04.016
was missing the sibling PC-5 entirely).
**Impact:** Wave-level is not redundant — it finds a structurally distinct class of finding. Per-story
convergence is necessary but not sufficient for wave convergence. Both loops required.
**Status:** [codified — confirmed by Wave 9 results]

---

### W9.L2 — Sibling-Discipline Pattern Self-Perpetuates Under Semantic Remediations [deferred — BEFORE Wave 10]

**Finding ID:** W9-D8 (Drift Items table, STATE.md; CRITICAL escalation 2026-05-26)
**Category:** process-discipline / adversarial-workflow
**Observed:** The sibling-discipline pattern (W7.2 / W8.4 recurrence) fired 6 consecutive times in
Wave 9: STORY-020 per-story passes 2/3/4 (3 cycles) + wave-level passes 1/2/3 (3 cycles). Each
semantic remediation (new PC, new AC, new EC, new test vector) created a new sibling-regression
discovered by the next fresh-context adversary. The pattern BROKE at wave-level pass 4 because
passes 1-3's final fixes were trivial text-only (memcap=12→4, PC-N→PC-5, placeholder→real ref) —
trivial fixes don't create adjacent semantic siblings.
**Pattern mechanics:** Semantic remediation adds a new structural element (PC, AC, EC) →
fresh-context adversary notices the NEW element's sibling positions are not updated → sibling fix
creates another new element → repeat. The loop terminates only when no new elements are introduced.
**Structural fix:** Sibling-sweep MUST be an explicit mandatory checklist step for EVERY remediation
burst that introduces a new PC/AC/EC/Canonical-Test-Vector/VP, not a best-practice reminder. The
checklist must be enforced at story-writer, test-writer, and PO remediation dispatch sites.
**Cumulative evidence:** ~10+ sibling-regression findings across waves 7/8/9; 5 consecutive adversary
cycles before W9 wave-level pass 4; W9-D8 escalated from P1 to CRITICAL.
**Target:** BEFORE Wave 10 dispatch — codification of mandatory sibling-sweep checklist in adversarial
and remediation skill prompts. Research-agent validation required per DF-VALIDATION-001 before
filing GitHub issue.
**Status:** [deferred — CRITICAL; target: before Wave 10 dispatch]

---

### W9.L4 — W9-D8 Codified as DF-SIBLING-SWEEP-001 [codified — 2026-05-26]

**Finding ID:** W9-D8 (Drift Items table, STATE.md; resolved via codification 2026-05-26)
**Category:** process-discipline / policy-codification
**Observed:** W9-D8 (sibling-discipline process-gap, CRITICAL) was codified as policy
DF-SIBLING-SWEEP-001 in `.factory/policies.yaml` before Wave 10 dispatch. Concrete
sibling-sweep checklists were added per artifact type (STORY, BC, TEST). Orchestrator-side
enforcement: inject checklist into every remediation dispatch under
"## Sibling-Sweep Checklist (MANDATORY per DF-SIBLING-SWEEP-001)" heading.
**Validation test for Wave 10:** Did the codification break the 6-cycle recurrence pattern?
Expected: YES — if orchestrator dispatch templates are updated to inject the checklist, each
remediation burst will self-sweep before the next adversary pass, preventing sibling-regressions
from surfacing as new findings. If Wave 10 shows zero sibling-discipline findings across per-story
and wave-level passes, the codification is effective. If the pattern recurs, escalate to structural
tooling (pre-commit hook or script).
**W9.L3 status:** W9.L3 (pr-manager merge-step gap) remains OPEN — separate codification target.
That is not a blocker for Wave 10 dispatch.
**Status:** [codified — 2026-05-26]

---

### W9.L3 — PR Manager Stops at Approve; Merge Step Requires Orchestrator [deferred — cycle-close codification]

**Finding ID:** Wave 9 retrospective observation (recurring; also observed Waves 8+9)
**Category:** process-discipline / agent-scope
**Observed:** The pr-manager agent consistently stops after receiving a pr-reviewer APPROVE verdict
and reports completion without executing the merge step. This recurred across PRs #122, #123, #126,
#127, #128, #129, #130 (spanning Waves 8 and 9). Each time, the orchestrator had to explicitly
invoke the merge step as a separate action.
**Impact:** Every PR close requires an extra orchestrator round-trip. At 4 PRs per wave, this is
a predictable latency multiplier. Over a 27-wave cycle it represents ~108 unnecessary handoff
interruptions.
**Root cause:** The pr-manager agent prompt does not explicitly enumerate "steps 7-9 are
non-optional; report back ONLY after merge commit SHA is confirmed." The agent treats APPROVE as a
natural stopping point because that is the end of its review-coordination loop.
**Codification candidate:** Update pr-manager agent prompt to include: "Steps 7 (approve), 8 (merge),
and 9 (confirm merge commit SHA) are all non-optional and sequential. Do NOT report back to the
orchestrator until the merge commit SHA is confirmed on develop. APPROVE alone is NOT task completion."
**Target:** Cycle-close codification — update pr-manager agent prompt before Wave 10. Research-agent
validation required per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [codified — DF-PR-MANAGER-COMPLETE-001 in .factory/policies.yaml on 2026-05-26]

---

### W9.L5 — W9.L3 Codified as DF-PR-MANAGER-COMPLETE-001 [codified — 2026-05-26]

**Finding ID:** W9.L3 (closed during codification)
**Category:** process-discipline / policy-codification
**Observed:** W9.L3 (pr-manager merge-step gap; recurring across 7 PRs in Waves 8-9) was codified
as policy DF-PR-MANAGER-COMPLETE-001 in `.factory/policies.yaml` before Wave 10 dispatch. The policy
enumerates concrete completion criteria for steps 7 (handle approval), 8 (squash merge), and 9
(post-merge cleanup) with exact `gh pr merge <#> --squash --admin --delete-branch` command template.
**Orchestrator enforcement:** Inject policy under "## PR Completion Policy (MANDATORY per
DF-PR-MANAGER-COMPLETE-001)" heading into every pr-manager dispatch, with `<#>` and `<branch>`
substituted. If pr-manager still reports back at step 6, orchestrator may re-dispatch with
"complete steps 7-9 NOW" or execute merge directly and tag as policy violation.
**Validation test for Wave 10:** Did the codification eliminate orchestrator merge-intervention?
Expected: pr-manager completes steps 7-9 autonomously per the injected policy with no extra
orchestrator round-trips. If the pattern recurs in Wave 10 despite injection, escalate as
agent-prompt-defect bug to vsdd-factory plugin maintainer.
**Status:** [codified — 2026-05-26]

---

## Earlier Wave Lessons (Waves 1-6)

Per-wave process-gap items for Waves 1-6 are recorded in STATE.md Cycle-Close Follow-Up Items
(W1.1, W1.2, W1.3, W2.1–W2.6, W3.1, W3.2, W4.1). Those items were captured as process-gap
table rows in STATE.md before this lessons.md file was created. They are not duplicated here.
