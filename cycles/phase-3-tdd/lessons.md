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

## Earlier Wave Lessons (Waves 1-6)

Per-wave process-gap items for Waves 1-6 are recorded in STATE.md Cycle-Close Follow-Up Items
(W1.1, W1.2, W1.3, W2.1–W2.6, W3.1, W3.2, W4.1). Those items were captured as process-gap
table rows in STATE.md before this lessons.md file was created. They are not duplicated here.
