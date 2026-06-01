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

## Wave 10 Lessons (2026-05-27)

- W10.L1: DF-SIBLING-SWEEP-001 v1→v4 iteratively refined through STORY-018 passes; v4 final checklist axes include AC-vs-revised-BC, story-EC-vs-BC-EC, test prose stale markers, test name inverted forms, cross-ref target resolution, implementation reachability, ALL ACs whose BCs modified in ANY pass. [codified iteratively]
- W10.L2: DF-PR-MANAGER-COMPLETE-001 insufficient at dispatch level for pr-manager agent; implementer-as-PR-executor (PR #133) RELIABLE workaround. [codified W10-D7]
- W10.L3: Brownfield-formalization surfaces CRITICAL spec/impl mismatches mid-convergence; reachability axis catches systematically. [codified in DF-SIBLING-SWEEP-001 v4]
- W10.L4: Wave-level adversarial structurally necessary for cross-story sibling-discipline (F-W10P1-001 overlap_count + F-W10P1-004 test duplication invisible per-story). [validated]

---

## Wave 11 Lessons (2026-05-27)

### W11.L1 — BC Pre-Merge Re-Anchor Doctrine [codified — 2026-05-27]

**Finding ID:** F-W11P8-001 (doctrine flip during pass-8 convergence)
**Category:** spec-anchor hygiene / process-discipline
**Observed:** STORY-021's test-seam additions (FINALIZE_SKIPPED_WARNED and associated lock statics)
pushed the `impl Drop` block downward across multiple adversarial passes. Line citations for
`impl Drop` drifted through 4 incremental states: 677-690 → 793-807 → 796-810 → 794-808. Each
remediation pass updated STORY-021 story body but left the corresponding BC-2.04.012 anchor at the
pre-pass value, creating a post-merge window where BC anchors pointed to pre-story-merge code.
**Doctrine adopted:** When a story's implementation shifts cited source-code line numbers, the BCs
cited in the story's `bcs:` frontmatter MUST be re-anchored to post-merge (worktree) line numbers
as part of the story's convergence cycle — NOT deferred to a follow-up sweep on develop. Final
anchor: 794-808 in BC-2.04.012 v1.5 (matching worktree-post-STORY-021 source).
**Impact:** Eliminates the post-merge stale-anchor window. BC versions are now authoritative
immediately after merge, not after a separate sweep.
**Codification:** Extended DF-SIBLING-SWEEP-001 (v2 "BC pre-merge re-anchor" bullet group) in
`.factory/policies.yaml`.
**Status:** [codified — 2026-05-27]

---

### W11.L2 — Adversary Methodology Bug: cd Non-Persistent Across Bash Invocations [codified — 2026-05-27]

**Finding ID:** F-W11P5-010 (closed during codification)
**Category:** adversarial-workflow / dispatch-methodology
**Observed:** Wave 11 pass-5 adversary was dispatched with `cd <worktree> && grep ...` patterns.
The `cd` did NOT persist across the adversary's Bash invocations in its read-only profile. The
adversary silently queried the main repo instead of the worktree, produced 2 FALSE-CRITICAL findings
("no STORY-021 implementation exists") that misled the convergence process. The orchestrator
identified the error by cross-referencing expected vs actual line numbers (impl Drop at line 794
in worktree, line 706 in main repo — distinct).
**Impact:** Wasted a full adversarial pass. The false positives required orchestrator-side
methodology debugging before pass-6 could be dispatched.
**Rule adopted:** Orchestrator dispatch prompts to adversary agents MUST use absolute paths for ALL
file operations. `cd <path> && ...` is FORBIDDEN in adversary dispatches except for cargo commands
where cwd is required. Git operations must use `git -C <absolute-path> ...`.
**Verification:** Adversary's first reply MUST include verification output proving worktree vs main
repo distinction (e.g., "impl Drop at line 794 worktree, line 706 main — distinct, methodology OK").
**Codification:** DF-ADVERSARY-METHODOLOGY-001 added to `.factory/policies.yaml`.
**Status:** [codified — 2026-05-27]

---

### W11.L3 — Iterative Line-Citation Drift from Seam-Block Edits [deferred — phase-5]

**Finding ID:** Drift pattern observed across passes 1-8 (4 drift cycles)
**Category:** spec-anchor hygiene
**Observed:** Seam-block edits (adding FINALIZE_SKIPPED_WARNED atomic and associated lock statics)
shifted `impl Drop` and related downstream code across 4 incremental states over the adversarial
convergence cycle. Each test-writer pass that introduced new statics above the impl Drop block
shifted its line number. Citations in STORY-021 story body and BC-2.04.012 required re-verification
after EVERY test-writer pass that touched cited files.
**Pattern:** When seam additions are inserted above cited functions, all subsequent anchors shift.
The drift is invisible until the next adversary re-reads the file. 3 passes of drift before final
convergence at 794-808.
**Recommendation:** After any test-writer pass that inserts code above cited functions, the
orchestrator MUST re-verify all line anchors in STORY-021 body + BCs before declaring a pass
clean. This is a subset of the W11.L1 BC pre-merge re-anchor procedure, applied intra-cycle.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — phase-5]

---

### W11.L4 — Source-Docstring Propagation Gap After Story-Body Changes [deferred — phase-5]

**Finding ID:** F-W11P6-001 (pass-6 adversary finding after pass-5 story-body fix)
**Category:** process-discipline / sibling-sweep
**Observed:** Pass-5 remediated STORY-021 story body (changed "194 sites" → "~130+ sites" and
related factual claims). Pass-6 adversary found 7 docstring sites in src/ and tests/ still
publishing the old content. The DF-SIBLING-SWEEP-001 procedure at pass-5 updated story body
but did not explicitly mandate a cross-file string-match step for src/ and test file docstrings.
**Impact:** One additional adversarial pass (pass-6) required to surface and remediate the
docstring-propagation gap.
**Rule adopted:** When a story body changes a "magic number" or factual claim, story-writer MUST
also identify all source/test-file docstrings that publish the same claim and dispatch test-writer
to update them in the same cycle.
**Codification:** Extended DF-SIBLING-SWEEP-001 (v2 "source-docstring propagation" bullet group)
in `.factory/policies.yaml`.
**Status:** [deferred — phase-5; partial codification in DF-SIBLING-SWEEP-001 v2]

---

### W11.L5 — Implementer-as-PR-Executor Pattern Continues to Outperform pr-manager [codified — confirmed W11]

**Finding ID:** Wave 11 retrospective observation (4th consecutive wave)
**Category:** process-discipline / agent-scope
**Observed:** PR #134 was executed by the implementer (STORY-021 implementer-as-PR-executor) with
all 9 steps completed autonomously — same pattern as PR #133 (Wave 10). The pr-manager agent
stops at APPROVE on PRs where GitHub self-review policy applies. This pattern has now been
observed across 4 consecutive waves: W8 (PRs #122/#123), W9 (PRs #127/#128/#129/#130), W10
(PRs #131/#132, workaround PR #133), W11 (PR #134).
**Root cause:** The pr-manager agent prompt's stop condition treats APPROVE as task completion.
The DF-PR-MANAGER-COMPLETE-001 policy injection improves behavior at the orchestrator dispatch
level but does not fix the underlying agent prompt. The implementer-as-PR-executor workaround
is structurally reliable because the implementer already has all branch/worktree context.
**Recommendation:** Retire pr-manager dispatch in favor of implementer-as-PR-executor for story
PRs. Reserve pr-manager only for PRs where a distinct review agent is required (wave-followup
PRs, chore PRs). The 4-wave evidence base (W8.L4 → W9.L2 → W10.L3 → W11.L5) justifies
formalizing this as the default pattern.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — phase-5; 4-wave recurrence justifies process-change proposal]

---

## Wave 12 Lessons (2026-05-27)

### W12.L1 — Anchor-Completeness EC-Scenario-Match Sub-Rule [codified — 2026-05-27]

**Finding ID:** F-W12P6-001 (pass-6 adversary; BC-2.05.002 EC-001 mis-anchor)
**Category:** spec-anchor hygiene / anchor-completeness doctrine
**Observed:** Wave 12 pass-6 adversary found that BC-2.05.002 EC-001 cited a test using port 9999
to cover an EC row described as "data starts with `b\"GET \"` on port 443". The test exercised the
same parent BC capability (HTTP routing) but used a different port than the EC scenario required.
The EC citation existed but did not match the specific scenario described.
**Doctrine sub-rule adopted:** When an EC row in a BC's Edge Cases table includes a `covered by
<test>` citation, the cited test MUST exercise the SPECIFIC scenario described in the EC row
(e.g., specific port number, specific byte value, specific length boundary) — not just the parent
BC capability. A citation that exercises the capability in a different configuration than the EC
describes is a MEDIUM mis-anchor (citation exists but doesn't match scenario).
**Impact:** Pass-6 catching this reset the clean streak from pass-5. Required a remediation burst.
Subsequent passes-7/8/9 were all CLEAN, confirming the sub-rule was the final convergence blocker.
**Codification:** Extended DF-SIBLING-SWEEP-001 to v3 (EC-scenario-match sub-rule bullet group)
in `.factory/policies.yaml`.
**Status:** [codified — 2026-05-27]

---

### W12.L2 — Anchor-Completeness Sibling-Sweep Must Be Single-Burst Across ALL BCs [codified — 2026-05-27]

**Finding ID:** Process gap observed across passes 3/4/6 (anchor cascade pattern)
**Category:** process-discipline / sibling-sweep
**Observed:** Wave 12 anchor-completeness corrections were applied iteratively:
- Pass-3 fixed BC-2.05.002 but left BC-2.05.001/003 with the same gap → pass-4 caught siblings
- Pass-4 fixed siblings but missed the EC-scenario-match detail in BC-2.05.002 → pass-6 caught it
- Each iteration cost an adversarial pass + remediation burst (~1.5 hours each)
Root cause: the orchestrator dispatched anchor-completeness fixes one-BC-at-a-time, relying on the
adversary to discover remaining sibling gaps. Each discovery reset the 3-clean streak counter.
**Rule adopted:** When anchor-completeness doctrine applies to ONE BC in a story (adding new test
names to Architecture Anchors, fixing EC citations, updating line ranges), the dispatching
orchestrator MUST sweep ALL BCs in the story `bcs:` frontmatter in the SAME burst — not
iteratively across adversarial passes. PO dispatch checklist MUST include: "Sweep ALL BCs in
story `bcs:` frontmatter for the same class of update before committing."
**Codification:** Extended DF-SIBLING-SWEEP-001 to v3 (single-burst-all-BCs rule bullet group)
in `.factory/policies.yaml`.
**Status:** [codified — 2026-05-27]

---

### W12.L3 — Brownfield-Formalization Without Test Seams Converges Faster [deferred — phase-5]

**Finding ID:** Wave 12 retrospective observation (convergence cost comparison)
**Category:** adversarial-workflow / convergence-cost
**Observed:** Wave 12 converged in 9 adversarial passes vs Wave 11's 11 passes. Wave 12
(STORY-031) touched only tests/dispatcher_tests.rs with no src/ modifications and no test seams.
Wave 11 (STORY-021) introduced 3 new test seam statics and modified 4 files including src/, which
caused the iterative line-citation drift pattern (W11.L3) that added passes.
**Pattern:** Test-only stories with no src/ modifications avoid the seam-block insertion cascade
that shifts downstream line numbers. No seam insertions = no anchor drift = fewer line-citation
passes required. The remaining adversarial cost in Wave 12 was driven entirely by anchor-completeness
EC citation quality (passes 1-6) rather than by implementation churn.
**Target:** phase-5 — evaluate whether "test-only story" can be a planning signal for reduced
adversarial budget allocation (e.g., 6 passes reserved vs 10 for src-touching stories).
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — phase-5]

---

### W12.L4 — DF-ADVERSARY-METHODOLOGY-001 Effective Across 9 Passes [codified — confirmed W12]

**Finding ID:** Wave 12 retrospective observation
**Category:** adversarial-workflow / policy-validation
**Observed:** Zero methodology bugs across all 9 adversarial passes in Wave 12. No false positives
from wrong-directory queries. All adversary dispatches used absolute paths per DF-ADVERSARY-METHODOLOGY-001
(codified 2026-05-27 from W11.L2). This is the first full wave under the new policy.
**Validation:** The W11 pass-5 false-CRITICAL finding pattern (adversary grepping main repo instead
of worktree) did not recur in any of Wave 12's 9 passes. Policy enforcement via orchestrator
dispatch template injection is working at the intended dispatch site.
**Status:** [codified — confirmed effective W12; no recurrence]

---

## Wave 13 Lessons (2026-05-27)

### W13.L1 — Brownfield-Formalization Without Test Seams = Even Faster Convergence [deferred — phase-5]

**Finding ID:** Wave 13 retrospective observation (convergence cost comparison)
**Category:** adversarial-workflow / convergence-cost
**Observed:** W11 (STORY-021): 11 passes, 9+4 test seams, atomics, Drop tripwire complexity.
W12 (STORY-031): 9 passes, 0 test seams (W12.L3 codified this), no docstring-shift cascade.
W13 (STORY-032): 5 passes, 0 test seams, 0 src/ changes, indirect-observability throughout.
**Pattern:** Zero src/ changes → fewer line-citation drift findings → fewer remediation cycles → faster convergence. The remaining adversarial cost in W13 was driven by anchor-completeness propagation gaps (passes 1-2), which are spec-quality findings, not implementation churn.
**Implication:** When formalizing impl with sufficient public API for indirect observability (parse_error_count, unclassified_flows, method_counts, max_classification_attempts), prefer that over adding `_for_testing` seams. The W11→W12→W13 trajectory (11→9→5) demonstrates sustained benefit with each wave that avoids src/ seams.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — phase-5; W12.L3 extended with W13 confirmation data]

---

### W13.L2 — DF-SIBLING-SWEEP-001 v3 Single-Burst-All-BCs Effectiveness Validated [codified — confirmed W13]

**Finding ID:** Wave 13 pass-1→pass-2 transition observation
**Category:** adversarial-workflow / policy-validation
**Observed:** Pass-1 produced 8 findings (4 MEDIUM, 4 LOW) largely comprising propagation gaps (BC-005/006 missing test anchors after BC-004 got them; stale 136-148 citations not propagated to story body; FSR row missed 5th test; AC-010 missing for EC-008). The v3 single-burst-all-BCs rule was applied in remediation: ALL BCs in story `bcs:` frontmatter were swept in a single burst.
**Result:** Pass-2 produced only 3 LOW observations (zero MEDIUM) — the v3 rule prevented further cascading sibling-sweep gaps. The subsequent clean streak (P3: NITPICK, P4: CLEAN, P5: CLEAN) confirms the burst was comprehensive.
**Without the rule:** The historical pattern (W12 passes 3/4/6 catching the same gap iteratively) would have repeated, adding 2-3 passes. The rule effectively collapsed that cascade into a single catch-up pass.
**Status:** [codified — confirmed effective W13; second wave of validation (first was W12)]

---

### W13.L3 — Indirect Observability Proxies (Cargo Public API Only) Sufficient for Cache + Retry-Budget Tests [deferred — phase-5]

**Finding ID:** Wave 13 retrospective observation
**Category:** test-design / public-API-sufficiency
**Observed:** All 5 new BC-prefixed tests in STORY-032 used only the public API for indirect observability:
- `unclassified_flows()` — for unclassified flow count assertions
- `max_classification_attempts()` — for retry budget enforcement
- `parse_error_count()` — for parse error tracking
- `method_counts()` — for classification result verification
Zero `#[doc(hidden)] pub fn _for_testing()` seams were added. The adversary confirmed faithfulness: each proxy uniquely identifies the code-path branch the test claims to exercise.
**Pattern:** "Observe analyzer state after dispatch" is preferable to "expose internal HashMap directly" when the public API surface provides sufficient discriminating power for the behaviors being tested.
**Target:** phase-5 — document as a test-design guideline: enumerate public API observability surface before reaching for test seams; seams are opt-in only when no public proxy discriminates the required behavior.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — phase-5]

---

### W13.L4 — DF-ADVERSARY-METHODOLOGY-001 Perfect Run for 2nd Consecutive Wave [codified — confirmed W13]

**Finding ID:** Wave 13 retrospective observation
**Category:** adversarial-workflow / policy-validation
**Observed:** Zero methodology bugs across all 5 adversarial passes in Wave 13. No false positives from wrong-directory queries. All adversary dispatches used absolute paths per DF-ADVERSARY-METHODOLOGY-001. The verification block in the adversary's first reply provided a cheap self-check that caught no anomalies.
**W12 confirmation:** Wave 12 (9 passes) was the first full wave under the policy — zero methodology bugs. Wave 13 (5 passes) confirms the policy holds at a different pass count and story complexity.
**Pattern:** The absolute-path mandate has zero runtime cost when correctly applied and prevents the entire class of wrong-directory false positives observed in W11-P5. Two consecutive waves of zero methodology bugs indicate the dispatch template injection is reliably working.
**Status:** [codified — confirmed effective W13; second consecutive wave with zero methodology bugs]

---

## Wave 14 Lessons (2026-05-28)

### W14.L1 — Compounding Gains from Doctrine Codification [codified — confirmed W14]

**Finding ID:** Wave 14 retrospective observation (pass-count trend)
**Category:** adversarial-workflow / convergence-cost
**Observed:** W11→W12→W13→W14 pass counts: 11 → 9 → 5 → 4. 64% total reduction over 4 waves. 20% reduction W13→W14.
Each reduction is attributable to a specific doctrine codification:
- W11→W12: DF-ADVERSARY-METHODOLOGY-001 (absolute paths) eliminated methodology-bug passes
- W12→W13: DF-SIBLING-SWEEP-001 v3 single-burst-all-BCs eliminated cascading anchor passes
- W13→W14: Pass-1 remediation burst caught all classes of drift in one shot → Pass-2 straight to CLEAN (no NITPICK_ONLY intermediate), skipping a pass tier entirely
**Pattern:** Doctrine codified in cycle N pays back in cycles N+1, N+2, ... compounding. No single doctrine change explains the full improvement; the cumulative layering of precise policies builds the effect over multiple waves. Early codification (even from a single-wave finding) is justified by the compound-return precedent.
**Validation:** W14 is the 4th consecutive wave of improvement. The trajectory is now consistent enough to be a planning assumption: "each new doctrine applied should reduce per-story pass count by ~1 pass for the next 1-3 waves."
**Status:** [codified — confirmed W14; pattern stable over 4 waves]

---

### W14.L2 — Documented Additive Seams = Acceptable Brownfield Scope [codified — W14]

**Finding ID:** F-W14P1-003 (HIGH scope-creep finding, Pass 1)
**Category:** brownfield-formalization / story-FSR discipline
**Observed:** STORY-033 AC-007 required indirect observability of on_flow_close → flows.remove. The test-writer added 2 `#[doc(hidden)] pub fn active_flows_len_for_testing()` seams in HttpAnalyzer + TlsAnalyzer. These seams were additive-only (no production behavior change) and consistent with ADR-0004 opt-in-per-guard doctrine. However, the initial story dispatch did NOT document the seams in the story FSR (File Structure Reference). Pass-1 adversary flagged this as HIGH scope-creep (code delivered without FSR declaration).
**Pass-1 remediation** added FSR rows + rationale note citing AC-007 + ADR-0004. Pass-2 verified clean.
**Rule confirmed:** Brownfield-formalization CAN add additive seams to non-frozen files, but the story FSR MUST declare them. Test-writer MUST update FSR proactively when a seam-add decision is made (in the same burst as the seam, per DF-SIBLING-SWEEP-001 v3 sibling-sweep).
**Impact:** Zero-cost to fix in Pass-1 remediation; zero recurrence in subsequent passes. FSR declaration fully resolves the scope-creep classification.
**Status:** [codified — FSR-seam-declaration rule confirmed W14; sub-rule of existing brownfield-formalization doctrine]

---

### W14.L3 — Null Verification Properties Is Correct for Some Stories [codified — confirmed W14]

**Finding ID:** Wave 14 Pass 1 + Pass 3 confirmations
**Category:** story-validation / frontmatter discipline
**Observed:** STORY-033 frontmatter: `verification_properties: []`. Pass 1 (HIGH coverage) and Pass 3 (re-confirmation) both verified: BC-2.05.007/008/009 are not in VP-004's `bcs:` list. VP-004 covers BC-2.05.004/005/006 (STORY-031/032). No VP applies to STORY-033's BCs.
**Rule confirmed:** `verification_properties: []` is the truthful value when no VP in `.factory/specs/verification-properties/` cites the story's BCs. It is NOT a frontmatter defect. Adversaries MUST verify VP-INDEX.md + individual VP `bcs:` lists before flagging empty verification_properties as a gap.
**Pattern:** Not every story has a VP anchor. The obligation is to check; the null value is acceptable when no VP applies.
**Status:** [codified — confirmed W14; adversary dispatch templates should include VP-null-is-acceptable note]

---

### W14.L4 — Implementer-as-PR-Executor: 7th Consecutive Validation [codified — confirmed W14]

**Finding ID:** Wave 14 retrospective observation (pattern continuity)
**Category:** process-discipline / agent-scope
**Observed:** PR #137 was executed by the implementer (STORY-033 implementer-as-PR-executor) with all 9 steps completed autonomously — 7th consecutive wave validating this pattern (PRs #131/132/133/134/135/136/137 across Waves 10-14). The pr-manager agent continues to stop at APPROVE on PRs where GitHub self-review policy applies.
**Pattern stability:** 7 consecutive PRs = no regression. The implementer-as-PR-executor workaround is the de facto standard for story PRs in this cycle. DF-PR-MANAGER-COMPLETE-001 policy injection continues to be included in every pr-manager dispatch, but the implementer-as-executor pattern does not require it.
**Pending action:** pr-manager agent prompt-level fix (W9.L3 follow-up) still unresolved. This is not blocking — the workaround is reliable — but the root cause structural gap persists.
**Status:** [codified — confirmed W14; 7-wave validation of implementer-as-PR-executor pattern]

---

## Wave 15 Lessons (2026-05-28)

### W15.L1 — First Multi-Story Wave Since Wave 10 — Parallelism Scales [codified — confirmed W15]

**Finding ID:** Wave 15 retrospective observation
**Category:** process-discipline / pipeline-throughput
**Observed:** Wave 15 dispatched STORY-041 (HTTP/1.1 parsing) and STORY-051 (JA3/JA3S GREASE) in parallel — first multi-story wave since Wave 10. Both stories converged and shipped (PRs #139 + #138). Total adversarial cost: 6 passes (STORY-051) + 8 passes (STORY-041) = 14 total passes.
**Pattern:** Multi-story parallelism scales well with the established per-story-delivery pattern. Convergence cost is approximately additive — 14 total passes ≈ 2 single-story wave equivalents. No shared-context interference between parallel stories.
**STORY-051 was 33% faster than STORY-041** (6 vs 8 passes): smaller spec surface (JA3/JA3S GREASE is a well-bounded algorithmic change) + cleaner test layout (test helpers extracted upfront). Size and structure of the spec surface correlate with convergence cost.
**Implication for Wave 16:** 4-story wave (STORY-042/043/044/052) is feasible with parallel dispatch. Expect 24-32 total adversarial passes (6-8 per story × 4). Schedule accordingly.
**Status:** [codified — confirmed W15; multi-story parallelism validated]

---

### W15.L2 — BC-Addition Sibling-Sweep Cascade Pattern [deferred — pending W16 evidence for DF-BC-CASCADE-001]

**Finding ID:** Wave 15 Pass 5 (F-W15P4-001 trigger chain)
**Category:** process-discipline / spec-propagation
**Observed:** BC-2.06.004 v1.5 added invariant 4 (response-side had_success guard) during Round 5 remediation (response to F-W15P4-001). Round 5 fixed the trigger BC + test but did NOT propagate to all sibling locations. Pass 5 caught 5 cascade findings (1M/4L):
- AC-004 trace label (still cited invariant 1, should reflect invariant 1-2 per AC narrative's invariant-2 content)
- Task 6 invariant range (not updated to include invariant 4)
- Architecture Compliance Rules row (had_success rule should reference BC-2.06.004 invariant 4 + AC-007)
- BC-2.06.002 ↔ BC-2.06.004 Related BCs reciprocal cross-refs (not updated)
- BC-2.06.004 Verification Properties table row (missing invariant 4 row)
**Pattern:** "BC-addition sibling-sweep cascade" — when a BC version bump adds a new invariant or postcondition, the remediation burst must propagate to ALL of: (1) related-BC reciprocal cross-refs, (2) VP table row, (3) Architecture Compliance Rules row, (4) Task list invariant range, (5) AC trace labels. Failure to sweep any one of these produces a cascade finding in the next pass.
**Codification candidate:** DF-BC-CASCADE-001 — inject "BC-addition cascade checklist" into PO/story-writer prompt when a BC version bump adds a new invariant. Deferred pending W16 confirmation.
**Status:** [deferred — DF-BC-CASCADE-001 candidate pending W16 evidence; one instance is a data point, not yet a pattern]

---

### W15.L3 — VHS+ffmpeg 8.1 Incompatibility Causes Empty Demo Artifacts [deferred — investigate in W16+]

**Finding ID:** Wave 15 demo recording phase
**Category:** toolchain / demo-quality
**Observed:** STORY-041 demo recordings produced empty .gif/.webm placeholder files. VHS .tape scripts captured correctly and .log files were present, but ffmpeg 8.1 (currently installed) is incompatible with the VHS gif/webm pipeline. Demo visual fidelity was lost for STORY-041. STORY-051 demos were not affected (different tape configuration or timing).
**Impact:** Not blocking. .tape + .log files captured correctly and provide textual evidence. Visual demos for STORY-041 are absent. factory-artifacts commit records placeholders.
**Recommendation:** Investigate ffmpeg version pinning in VHS demo recording setup. Options: (a) pin ffmpeg to a known-compatible version (e.g., 7.x), (b) downgrade VHS to a version compatible with ffmpeg 8.1, (c) switch demo format to text-only (.log) for waves until toolchain issue resolved.
**Target:** Wave 16+ investigation — not blocking current delivery. If persistent, add a toolchain note to CLAUDE.md.
**Status:** [deferred — W16+ investigation; not blocking]

---

### W15.L4 — DF-AC-TEST-NAME-SYNC-001 v1 (W14 Codification) Caught P1 Drift for Both Stories [codified — compounding-gain doctrine validated]

**Finding ID:** Wave 15 Pass 1 — both STORY-041 and STORY-051
**Category:** process-discipline / codification-compounding-gain
**Observed:** DF-AC-TEST-NAME-SYNC-001 v1 (codified in Wave 14 from F-W14P1-011) fired correctly in Wave 15 Pass 1 for both stories. For both STORY-041 and STORY-051, the adversary identified AC "**Test:**" citations that used short-form test names instead of BC-prefixed fn names. These were caught and remediated in the P1 burst, preventing the same class of drift from requiring its own dedicated pass.
**Impact:** Without DF-AC-TEST-NAME-SYNC-001, these findings would have surfaced as MEDIUM at Pass 2 or later, extending the convergence trajectory. The codification prevented re-litigation of the W14 cluster and likely saved 1 pass per story (2 passes total across W15).
**Pattern:** Compounding-gain doctrine validated — a policy codified from W14 data already produced measurable benefit in the next wave. One-wave codification latency.
**Implication:** Early codification of second-wave patterns (W12.L2 precedent → W14 early codification of DF-AC-TEST-NAME-SYNC-001) provides compounding returns faster than deferred codification.
**Status:** [codified — W14 codification effective in W15; compounding-gain doctrine confirmed for second consecutive wave]

---

---

## Wave 16 Lessons (2026-05-29) — Retroactive Convergence

Stories: STORY-042, STORY-043, STORY-044, STORY-052. 4-story wave; retroactive convergence run after stories were merged in a prior session.

### W16.L1 — Merge-Before-Convergence Is Silent; STATE.md Drift Is the Detection Signal [codification candidate — HIGHEST VALUE LESSON]

**Finding ID:** Wave 16 meta-process-gap
**Category:** process-gap / workflow-enforcement
**Observed:** All 4 Wave 16 stories (STORY-042/043/044/052) were merged to develop in a prior session WITHOUT running mandatory per-story + wave-level adversarial convergence. STATE.md was left at `ready_to_dispatch` instead of advancing to `convergence_in_progress` or `closed`. The stale status was the detection signal in the next session.
**Impact:** Retroactive convergence works but is more expensive than in-flight — the adversary must reconstruct context from archived state rather than operating on fresh-dispatch context. Retroactive convergence consumed a full session; in-flight would have been interleaved with delivery.
**Root cause:** The per-story delivery flow allows merge before Step-4.5 adversarial convergence because no enforcement gate blocks the merge-step. STATE.md `stories_delivered` counter was incremented on merge, not on convergence-close.
**Codification follow-up:** Delivery workflow MUST NOT merge before Step-4.5 (adversarial convergence per BC-5.39.001). Add post-merge prohibition or gate to delivery workflow. File as draft codification item; requires DF-VALIDATION-001 research-agent validation before any GitHub issue.
**Status:** [deferred — draft codification item; research-agent validation required per DF-VALIDATION-001]

---

### W16.L2 — LOW-Nit-Rides Policy Is What Enables 3-Clean Streaks to Accumulate [codified — compounding-gain doctrine]

**Finding ID:** Wave 16 convergence policy decision (Pass-4 → Pass-5)
**Category:** convergence-policy
**Observed:** Freezing artifacts after the last blocking (MEDIUM+) fix and letting LOW-severity nits ride without remediation is the mechanism that allows the 3-consecutive-clean streak to accumulate. Across Passes 5/6/7 multiple LOW nits were observed each pass but none triggered remediation. The streak accumulated cleanly. Had the LOW findings been remediated after each pass, each remediation burst would have reset the streak and produced another cycle of cleanup passes.
**Impact:** The LOW-nits-ride policy (established in Wave 14 "NITPICK_ONLY" pass convention) was decisive in Wave 16, where 4 LOW observations per pass were present for 3 consecutive clean passes.
**Pattern:** Only MEDIUM+ findings break the streak. LOW observations are recorded as drift items (wave-close batch) without resetting the convergence counter.
**Status:** [codified — compounding-gain doctrine confirmed; LOW-nits-ride policy effective across W14/W15/W16]

---

### W16.L3 — Remediation Can Introduce Defects; Sibling-Sweep Must Verify Cited Line Numbers [codification candidate]

**Finding ID:** F-W16-S044-P3-001 + F-W16-S052-P2-001 anchor drift
**Category:** process-discipline / sibling-sweep
**Observed:** The Pass-2 factory-only burst tightened BC-2.06.015's anchor to `467-468` but did NOT sweep the consuming STORY-044 Architecture Mapping body (line 124) which still cited `467`. This was caught in Pass-3 as F-W16-S044-P3-001 (MEDIUM). A second instance: a line-anchor correction in a prior burst pointed into the wrong test body — caught in Pass-4.
**Root cause:** DF-SIBLING-SWEEP-001 v3 sweeps BC-to-BC and BC-to-test, but did not explicitly enumerate "consuming-story Architecture-Mapping bodies that cite the same source anchor."
**Codification follow-up:** Extend DF-SIBLING-SWEEP-001 to v4, requiring that on any BC anchor change, the sweep explicitly covers: (1) all existing sweep targets, AND (2) consuming-story Architecture-Mapping table bodies citing the same source anchor. File as draft extension; requires DF-VALIDATION-001 research-agent validation.
**Status:** [deferred — draft codification item; research-agent validation required per DF-VALIDATION-001]

---

### W16.L4 — Read-Only Adversary Profile Cannot Run Cargo; Orchestrator Must Supply Build Evidence [codified — dispatch protocol]

**Finding ID:** Wave-level Pass-1 Lens B (integration) — DIRTY-procedural-only
**Category:** adversarial-methodology / dispatch-protocol
**Observed:** The wave-level integration lens adversary (read-only profile) could not run `cargo test`, `cargo clippy`, or `cargo fmt`. It produced a DIRTY verdict for the procedural inability to verify toolchain state, even though the substantive integration findings were all CLEAN. The orchestrator independently verified at session start (cargo test/clippy/fmt all green; diff=test-only+seam) and annotated the verdict as "substantively CLEAN."
**Impact:** Without orchestrator annotation, a DIRTY-procedural-only verdict could trigger unnecessary remediation passes. The Lens B DIRTY reset the wave-level streak to 0 and required a re-run (Pass-2).
**Dispatch protocol fix:** When dispatching a read-only wave-level adversary, the orchestrator MUST inject build evidence (cargo test output summary, clippy clean confirmation, diff characterization) into the dispatch prompt. The adversary cites injected evidence for toolchain-state claims rather than running tools itself.
**Status:** [codified — dispatch protocol; orchestrator must inject build evidence for read-only adversary dispatches]

---

### W16.L5 — Fresh-Context Adversary Can Produce False-Positive MEDIUM via Incomplete Search [codified — verification protocol]

**Finding ID:** F-W16-WAVE-R2-001 — VP-006 "orphan" false-positive
**Category:** adversarial-methodology / verification-discipline
**Observed:** A fresh-context adversary (wave-level round-2 consistency lens) flagged VP-006 as an "orphan verification property" — claimed it was not referenced in VP-INDEX.md. Investigation confirmed VP-006 exists at `.factory/specs/verification-properties/vp-006-http-poison-monotonicity.md` AND is registered at VP-INDEX.md:54 with citations to BC-2.06.015/016/017. STORY-044 is legitimately the only wave story with a VP. The finding was a CONFIRMED FALSE-POSITIVE caused by the adversary not searching the verification-properties/ directory before making the negative-existence claim.
**Impact:** If accepted uncritically, this false-positive would have triggered a remediation burst touching VP-INDEX.md and STORY-044 with no actual defect to fix. Orchestrator verification of negative-existence claims before routing remediation is essential.
**Protocol fix:** Before accepting any adversary negative-existence claim (e.g., "X is not registered in INDEX"), the orchestrator MUST verify via a filesystem search (`ls` or `grep`). A negative claim without a search transcript is automatically suspect.
**Status:** [codified — verification protocol; orchestrator must verify adversary negative-existence claims before routing remediation]

---

### W16 Process-Gap Codification Follow-Ups (Draft Items)

The following process-gaps from Wave 16 require DF-VALIDATION-001 research-agent validation before any GitHub issue is filed. Recorded here as draft codification items for tracking:

| ID | Gap | Proposed Fix | Priority |
|----|-----|-------------|----------|
| PG-W16-001 | DF-AC-TEST-NAME-SYNC-001 v1 verifies AC `**Test:**` name EXISTENCE but not UNIQUE RESOLUTION nor correct fn-declaration line anchor (F-W16-WAVE-P1-003 + F-W16-S044-P4-001). | Extend policy to v2 requiring unique resolution + correct fn-declaration line. | HIGH |
| PG-W16-002 | Merged stories had no workflow step transitioning story status draft/in-progress → completed on merge (F-W16-S042-P1-001; also caught Wave-15 STORY-041/051 stuck at in-progress). | Add post-merge status-transition step to delivery workflow. | HIGH |
| PG-W16-003 | BC-edit sibling-sweep did not extend to consuming-STORY Architecture-Mapping bodies citing same source anchor (F-W16-S044-P3-001 partial-fix propagation). | Extend DF-SIBLING-SWEEP-001 to v4: add consuming-story body sweep on BC anchor changes. | HIGH |
| PG-W16-004 | No CI gate enforcing zero production callers of `*_for_testing` seams (F-W16-WAVE-P2-003) — convention-only. | Research feasibility of CI grep-gate or dylint. Deferred drift. | MEDIUM |
| PG-W16-005 | 4 stories merged without mandatory per-story + wave-level adversarial convergence; STATE.md left stale (ready_to_dispatch). | Per-story delivery flow must not merge before Step-4.5 adversarial convergence; add gate or enforced protocol step. | CRITICAL (highest-value lesson W16) |

All items require `vsdd-factory:research-agent` validation per policy DF-VALIDATION-001 before any GitHub issue is filed.

---

## Wave 17 Lessons (2026-05-29) — Brownfield Formalization; 3 Stories

Stories: STORY-045 (PR #150), STORY-053 (PR #149), STORY-055 (PR #151). 3-story wave; all 3 per-story converged (3-clean P3-P5, 5 passes each). Wave-level: pass-1 DIRTY → remediation → pass-2 all-3-lenses CLEAN. CONVERGED 2026-05-29.

### W17.L1 — Per-Story Convergence Cannot Catch Cross-Story AC-Citation Sibling-Misses When Legacy Names Resolve [process-gap — PG-W17-001/002]

**Finding ID:** F-W17-WAVE-C-001 / F-W17-WAVE-T-001 (wave-level pass-1, HIGH)
**Category:** process-gap / sibling-sweep / AC-test-name-sync
**Observed:** STORY-055 per-story convergence passed 3-clean P3-P5 because the AC `**Test:**` citations named LEGACY test function names that still resolved (the old short-form names existed alongside the new BC-prefixed names). Only the wave-level cross-story consistency lens — examining ALL 3 W17 stories side-by-side — detected that STORY-055 AC citations did not name the BC-prefixed tests added by the test-writer. The per-story adversary did not flag this because the legacy names resolved without ambiguity per the per-story isolation context.
**Impact:** Wave-level pass-1 produced a HIGH finding (F-W17-WAVE-C-001). STORY-055 v1.2 AC-citation sync was required before pass-2 could run. Wave close was delayed by one remediation burst.
**Root cause:** DF-AC-TEST-NAME-SYNC-001 v2 requires BC-prefixed test citations but cannot enforce the negative — the absence of the NEW prefixed name is undetectable per-story when the old name also resolves. The enforcement gap only manifests cross-story during wave-level review.
**Codification follow-up (PG-W17-002):** A wave-gate pre-close checklist item should sweep ALL same-strategy/same-subsystem wave siblings for AC-test-name-sync before wave close. Propose DF-SIBLING-SWEEP-001 v5 or a new wave-gate-checklist policy (e.g., DF-WAVE-SIBLING-AC-SYNC-001). Requires DF-VALIDATION-001 research-agent validation before filing GitHub issue.
**Status:** [process-gap — PG-W17-001 (recurring, all 3 W17 stories) + PG-W17-002 (new, wave-gate checklist); DF-VALIDATION-001 validation required]

---

### W17.L2 — Brownfield Test-Writer Dispatches MUST Include AC Test-Citation Sync in Same Burst [codification reinforcement]

**Finding ID:** PG-W17-001 (recurrence on all 3 Wave 17 stories)
**Category:** process-gap / dispatch-template / AC-test-name-sync
**Observed:** The recurring AC-test-name-sync defect (first codified W14 → DF-AC-TEST-NAME-SYNC-001 v1, extended W16 → v2) hit ALL 3 Wave 17 stories: STORY-045, STORY-053, and STORY-055. In each case, the brownfield-formalization test-writer dispatch created BC-prefixed test functions WITHOUT simultaneously updating the story AC `**Test:**` citations to reference the new BC-prefixed names. The test-writer burst was dispatched without an explicit instruction to also update the story file.
**Root cause:** The test-writer dispatch template does not mandate a story-file AC sync step. DF-AC-TEST-NAME-SYNC-001 is a policy on the output (citations must match), not on the dispatch (test-writer MUST update story in same burst).
**Codification required:** Test-writer dispatch prompts for brownfield-formalization stories MUST include an explicit step: "After writing BC-prefixed tests, update the story AC `**Test:**` citations in the same burst. Do not leave the story file with legacy test names." This closes the dispatch-template gap. Extends or reinforces DF-AC-TEST-NAME-SYNC-001. Requires DF-VALIDATION-001 validation before filing GitHub issue.
**Status:** [codification reinforcement — PG-W17-001; DF-VALIDATION-001 validation required]

---

### W17.L3 — TLS Test-File Merge Conflict (STORY-053 + STORY-055 Touching Same File) Resolved Cleanly [validated pattern]

**Finding ID:** Wave 17 merge observation
**Category:** merge-management / conflict-resolution
**Observed:** STORY-053 and STORY-055 both added tests to `tests/tls_analyzer_tests.rs`. Merging PR #149 (STORY-053, 83 fns) then PR #151 (STORY-055, +10 fns = 86 total) required keep-both conflict resolution. The sequencing (053 merged first, 055 rebased on result) produced zero test collisions. Final `tls_analyzer_tests.rs` contains 86 functions with no overlap in function names.
**Pattern confirmed:** For same-file TLS test additions, sequencing merges + rebase on the first merge HEAD resolves cleanly when BC-prefixed function names are unique (as required by DF-AC-TEST-NAME-SYNC-001 v2).
**Status:** [validated — keep-both merge resolution pattern for same-file test additions]

---

### W17.L4 — Harness Security Classifier Blocks Default-Branch Merges Without Human Authorization [noted — structural]

**Finding ID:** Wave 17 merge observation
**Category:** process-discipline / security-classifier / merge-authorization
**Observed:** The harness security classifier blocked pr-manager subagents from executing squash-merges to `develop` for PRs #150, #149, and #151. Orchestrator completed the merges directly after receiving explicit human authorization for each PR. This is the same pattern as Wave 16 (W16.L4 → same root cause).
**Pattern:** For all develop-branch story merges, the orchestrator requires explicit human authorization before executing `gh pr merge --squash`. The pr-manager → implementer-as-PR-executor path is the only reliable autonomous path, but even that is blocked for merges to `develop` without human confirmation.
**Status:** [noted — structural behavior; no codification candidate (classifier behavior is by-design)]

---

### W17 Process-Gap Codification Items (Draft)

| ID | Gap | Proposed Fix | Priority |
|----|-----|-------------|----------|
| PG-W17-001 | AC-test-name-sync recurred ALL 3 W17 stories — test-writer dispatch did not include story AC citation sync step | Extend test-writer dispatch template with mandatory AC-sync step; reinforce DF-AC-TEST-NAME-SYNC-001 | HIGH |
| PG-W17-002 | Per-story passes cannot catch wave-sibling AC-sync misses when legacy names resolve — only wave-level caught STORY-055 | Add wave-gate pre-close checklist sweep: all same-strategy/same-subsystem siblings must be AC-sync verified before wave close. Tag: DF-SIBLING-SWEEP-001 v5 or DF-WAVE-SIBLING-AC-SYNC-001 | HIGH |

All items require `vsdd-factory:research-agent` validation per policy DF-VALIDATION-001 before any GitHub issue is filed.

---

## Wave 18 STORY-058 Lessons (2026-05-29)

### W18-S058.L1 — Cross-Artifact Citation Re-Points Must Trigger Full Occurrence Sweep [codified — PG-W18-002 extended]

**Finding ID:** PG-W18-002 (extension from STORY-056); F-S058-P3-001/P4-001/P5-002/P6-002
**Category:** process-gap / sibling-sweep
**Observed:** STORY-058 needed passes 3/4/5/6 to chase the same AC-013 mis-mapping across 3 distinct locations: (1) story FSR row, (2) BC-2.07.033 Proof-Method/Evidence field, (3) test-file index header comment at tls_analyzer_tests.rs. Each pass caught a different occurrence. A 3rd-occurrence stale index comment surfaced at pass 8 (F-S058-P8-001 MED) after prior passes had fixed story and BC. Root cause: the remediation burst for AC-013 re-point did not sweep all occurrence types — story body, story FSR, story test-citation, BC Evidence fields of all `bcs:` BCs, and test-file index/header comments.
**Impact:** 5 extra passes (P3/P4/P5/P6/P8 DIRTY) attributable to incomplete sweep. BC-5.39.001 delayed from P2 to P13.
**Codification:** Extends PG-W18-002. Candidate rule: "test-citation change" checklist must enumerate: story body ACs, story FSR rows, test-file index comments, sibling BC Proof-Method/Evidence columns. DF-VALIDATION-001 applies before filing GitHub issue.
**Status:** [codified — PG-W18-002 extended; candidate codification pending DF-VALIDATION-001]

---

### W18-S058.L2 — Mis-Anchor on Proof-Method Test Persists Across 2 Passes Before Root-Cause Identified [noted]

**Finding ID:** F-S058-P3-001/P4-001 (HIGH — BC-2.07.033 Proof Method cited done-short-circuit test for within-loop-skip claim)
**Category:** BC evidence quality / anchor correctness
**Observed:** BC-2.07.033 v1.2 cited `test_stop_after_handshake` (done-short-circuit test) as Proof Method for its within-loop-skip invariant. Pass 3 flagged HIGH. Pass 4 (fresh context) independently corroborated. Root cause: the BC was written to reference the first relevant test found, but the done-short-circuit and within-loop-skip are distinct behaviors with distinct test names. The fix (v1.3) re-pointed to `test_loop_skip_on_done`, resolved at P5.
**Pattern:** Same HIGH mis-anchor was caught by 2 independent adversary contexts — confirms adversarial consistency rule; high-severity cross-context agreement is reliable.
**Status:** [noted — no new codification; DF-SIBLING-SWEEP-001 already covers anchor correctness]

---

### W18-S058.L3 — BC Arithmetic Errors Caught by Fresh-Context Adversary [noted]

**Finding ID:** F-S058-P3-002 (MED — BC-2.07.029 invariant-2 arithmetic: was `parse_errors − truncated_records`, should be `parse_errors − parse_errors_that_are_also_truncated`)
**Category:** BC correctness / arithmetic
**Observed:** BC-2.07.029 invariant-2 stated the relationship between `parse_errors` and `truncated_records` with incorrect arithmetic. Fresh-context adversary caught this in P3. Fixed in v1.3.
**Pattern:** Arithmetic invariants in BCs benefit from fresh-context review; local familiarity with the counter semantics masks the error.
**Status:** [noted — no new codification]

---

### W18-S058.L4 — Deferred LOW Items Accepted Below-MEDIUM Threshold (BC-5.39.001 Satisfied) [noted]

**Finding ID:** F-S058-P11-001, F-S058-P11-002, F-S058-P12-O1, F-S058-P13-O4
**Category:** quality / threshold
**Observed:** Four LOW/observation items were identified during the clean-streak passes (P11/P12/P13) and accepted as deferred. These are: stale "sync to story after this pass" comment at tls_analyzer_tests.rs:6819; test_nonhandshake_types EC-label set discrepancy (header EC-002/003/004 vs body EC-001-004); BC-2.07.005 anchor off-by-one (726-748 vs 726-747); test_stop_after_handshake cross-story AC labels + STORY-058 FSR pre-existing collision. None above LOW; all below MEDIUM threshold for blocking convergence per BC-5.39.001.
**Status:** [deferred — items remain in STATE.md Cycle-Close Follow-Up / deferred items table]

---

### W18 STORY-058 Process-Gap Codification Items (Draft)

| ID | Gap | Proposed Fix | Priority |
|----|-----|-------------|----------|
| PG-W18-002 (extended) | AC-013 re-point propagated to 3 of 5 artifact locations; missed test-file index comment + sibling BC Evidence columns in same burst. 5 extra passes (P3/P4/P5/P6/P8) resulted. | Add "test-citation change" checklist to story-writer/BC-editor dispatch: enumerate story body, story FSR, BC Evidence for all `bcs:` BCs, test-file index header comments. | HIGH |

All items require `vsdd-factory:research-agent` validation per policy DF-VALIDATION-001 before any GitHub issue is filed.

---

## Earlier Wave Retrospectives (Archived from STATE.md 2026-05-29)

Full retrospective text for Waves 9-17 and the Drift Remediation pass was archived from
STATE.md on 2026-05-29 to keep STATE.md under 450 lines (compaction S-7.02 / content-routing
rule: historical retrospectives go in cycle files, not STATE.md).

### Wave 9 Retrospective (closed 2026-05-26)

- Stories: STORY-016 (6 passes 3D+3C) + STORY-020 (8 passes 5D+3C). PRs: #127-130. Wave-level 6 passes 3D+3C.
- Key outcome: DF-SIBLING-SWEEP-001 codified from W9-D8 (6-recurrence sibling-discipline pattern). DF-PR-MANAGER-COMPLETE-001 codified from W9.L3. Both pre-Wave-10.
- Drift: W9-D5/D12 (LOW); W9-D8/D9 RESOLVED; W9-D1..D4 (LOW template gaps); all require DF-VALIDATION-001 before issue filing.

### Wave 10 Retrospective (closed 2026-05-27)

- Stories: STORY-017 (4 passes 1D+3C) + STORY-018 (9 passes 6D+3C). PRs: #131/132/133. Wave-level 4 passes 1D+3C.
- DF-SIBLING-SWEEP-001 demonstrably effective: STORY-017 cleared in 4 passes. iteratively refined v1→v4.
- 3 brownfield spec/impl mismatches resolved (BC-2.04.041/045/027 v1.3). 1 src/ hardening: overlap_count saturating_add (PR #133).
- Total adversarial: 17 passes (vs Wave 9: 20 = 15% reduction).

### Wave 11 Retrospective (closed 2026-05-27)

- Stories: STORY-021 (finalize lifecycle + MAX_FINDINGS cap + segment-limit summary; brownfield-formalization). PRs: #134 → 3cd3000.
- Adversarial passes: 11 total (passes 9-10-11 CLEAN per BC-5.39.001). 203 new tests. 4 files. Zero production behavior changes.
- Doctrine flip in pass-8: BC pre-merge re-anchor adopted (W11.L1). Methodology bug (pass-5): DF-ADVERSARY-METHODOLOGY-001 added (W11.L2).

### Wave 12 Retrospective (closed 2026-05-27)

- Stories: STORY-031 (content-first classification). PRs: #135 → 1435362. 9 passes (7-8-9 CLEAN per BC-5.39.001).
- EC-scenario-match sub-rule discovered (W12.L1). DF-SIBLING-SWEEP-001 extended to v3.

### Wave 13 Retrospective (closed 2026-05-27)

- Stories: STORY-032 (cache eviction + retry budget + unclassified flow). PRs: #136 → 0d9b16d. 5 passes (3-4-5 CLEAN).
- 44% fewer passes than W12. Zero src/ changes; indirect observability throughout.

### Wave 14 Retrospective (closed 2026-05-28)

- Stories: STORY-033 (active-flows lifecycle). PRs: #137 → 30cd4a6. 4 passes (2-3-4 CLEAN).
- 1 codification: DF-AC-TEST-NAME-SYNC-001 v1. W11→W12→W13→W14 trajectory: 11→9→5→4 passes.

### Wave 15 Retrospective (closed 2026-05-28)

- Stories: STORY-041 (8 passes, 24 BC tests) + STORY-051 (6 passes, 19 BC tests). PRs: #138/#139. First multi-story since W10.
- BC-addition sibling-sweep cascade pattern (W15.L2). 9th+10th implementer-as-PR-executor validations.

### Drift Remediation Retrospective (2026-05-29)

66 tracked (62 original + 4 new) → 57 closed; 10 OPEN. Key: DF-16.B (209 BC files broken citations); 5 new policies codified; develop HEAD advanced to 34e66c7 (PRs #147+#148).
Archive: `.factory/cycles/drift-remediation-2026-05-29/closed-items.md`

---

---

## Wave 18 Lessons (2026-05-29)

Wave 18: STORY-046 (E-4 HTTP, 3pts) + STORY-054 (E-5 TLS, 8pts) + STORY-056 (E-5 TLS, 8pts) + STORY-058 (E-5 TLS, 8pts). 27pts. PRs #152-155. develop HEAD at close: 3f87ac3. Wave-level: round-1 3-lens CLEAN.

### PG-W18-001 — DF-ADVERSARY-METHODOLOGY-001 Recurrence (STORY-054 Pass-10 Wrong-Checkout False-Positive) [deferred — codification-candidate]

**Finding ID:** PG-W18-001 (Cycle-Close Follow-Up, STATE.md)
**Category:** process-gap / adversarial-methodology
**Observed:** STORY-054 pass-10 adversary reviewed develop HEAD instead of feature/STORY-054 worktree, producing 3 false-CRITICAL findings. Pass-11 checkout-guard (branch assertion + grep-count assertion) succeeded and confirmed the false-positive nature of pass-10 findings. This is a recurrence of DF-ADVERSARY-METHODOLOGY-001 (first caught W11).
**Impact:** One full adversary pass wasted. Required pass-11 re-run with explicit checkout verification.
**Recommendation:** Bake checkout-guard (verify branch==feature/STORY-NNN AND a known story-specific grep-count) into every per-story adversary dispatch template. Also: .factory is gitignored in worktrees — dispatch MUST provide absolute main-repo paths for factory artifacts.
**Codification candidate:** DF-ADVERSARY-METHODOLOGY-001 v2 (extend with checkout-guard requirement).
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — codification-candidate; requires DF-VALIDATION-001]

---

### PG-W18-002 — Test-Citation Re-Points Must Trigger Same-Burst Sweep of ALL Occurrences [deferred — codification-candidate]

**Finding ID:** PG-W18-002 (Cycle-Close Follow-Up, STATE.md)
**Category:** process-gap / sibling-sweep discipline
**Observed (STORY-056 original):** Story-anchor fix (F-S056-P3-001) was applied to the story body but did not sweep sibling BCs in the same burst. PG-W18-002 logged.
**Extended (STORY-058):** AC-013 mis-citation existed in 3 locations: (1) STORY-058.md FSR row, (2) BC-2.07.004 Evidence field, (3) tls_analyzer_tests.rs index comment. Each burst fixed only 1-2 locations; adversary found the 3rd each time. STORY-058 needed passes 3/4/5/6/8 to chase the same AC-013 mis-mapping across those 3 locations.
**Root cause:** The existing DF-SIBLING-SWEEP-001 policy covers BC sibling-sweeps and STORY Architecture-Mapping sweeps, but does NOT explicitly enumerate test-file index/header comments as a sweep target.
**Recommendation:** Add explicit checklist rule: when a test-citation (AC test name or AC-NNN reference) changes in ANY of {story FSR, BC Proof-Method, BC Evidence, test-file header comment, test-file index comment}, ALL of those locations must be swept in the SAME burst before declaring the fix complete.
**Codification candidate:** DF-SIBLING-SWEEP-001 v5 (add test-citation sweep locations) or new DF-TEST-CITATION-SWEEP-001.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — codification-candidate; requires DF-VALIDATION-001; detail: W18-S058.L1]

---

### PG-W18-003 — TLS Test Flat Namespace vs HTTP Per-Story Mod Wrappers (Latent Collision Risk) [deferred — codification-candidate]

**Finding ID:** PG-W18-003 (Cycle-Close Follow-Up, STATE.md; from F-W18-WAVE-I-006)
**Category:** process-gap / test-architecture convention
**Observed:** tests/tls_analyzer_tests.rs uses a FLAT namespace — all TLS test functions live at the module root without per-story `mod` groupings. tests/http_analyzer_tests.rs uses per-story `mod` wrappers (the F-W16 collision fix). This divergence means that as more TLS stories are delivered, test-name collisions across stories become a latent risk (the same issue that triggered PR #146 for HTTP tests).
**Impact (current):** No collision yet. Risk increases with each new TLS story added to the flat namespace.
**Impact (future):** A name collision in tls_analyzer_tests.rs would require a retroactive rename PR similar to PR #146, touching ALL existing TLS story tests.
**Recommendation:** Codify one convention (per-story `mod` wrapper) across both analyzer test files. Apply to tls_analyzer_tests.rs proactively before Wave 19+ TLS stories land.
**Codification candidate:** Add DF-TEST-NAMESPACE-CONVENTION-001 or extend DF-AC-TEST-NAME-SYNC-001.
**Follow-up:** Wave-level F-W18-WAVE-C-001/002/003 (cross-story style-convention divergences) fold into this codification.
**Validation required:** research-agent must validate per DF-VALIDATION-001 before filing GitHub issue.
**Status:** [deferred — codification-candidate; requires DF-VALIDATION-001]

---

### Wave 18 Retrospective Summary

- 4 stories, 27pts, PRs #152-155, develop HEAD 3f87ac3.
- Deepest story: STORY-058 (13 passes; AC-013 mis-citation cascade drove passes 3-8).
- Fastest story: STORY-046 (4 passes; isolated HTTP scope, zero cross-story coupling).
- Wave-level: round-1 3-lens CLEAN (no dirty round — improvement vs W16 round-1-dirty, W17 wave-level P1-dirty).
- All zero src changes (brownfield-formalization wave-wide).
- 3 process-gaps codified: PG-W18-001/002/003 (all require DF-VALIDATION-001 before issue filing).
- 4 deferred-LOWs accepted: OBS-7, F-S058-P11-001/002, F-S058-P12-O1, F-S058-P13-O4.
- Phase-4-ENTRY deferred: HS-* re-validation against W18 BC corrections (BC-2.07.002/012/029).

---

## Governance Policy Codification — 2026-05-30

Four process-gap candidates from Waves 18 and the drift-remediation pass were codified into
`.factory/policies.yaml` on 2026-05-30 during a deferred-item cleanup burst.

| New Policy ID | Source Item | Severity | Summary |
|--------------|-------------|----------|---------|
| DF-INPUT-HASH-CANONICAL-001 | PG-HASH-001 | HIGH | input-hash MUST be computed via bin/compute-input-hash (MD5/inputs-order), never hand-computed (sha256/sorted). Story-writer and product-owner dispatch prompts MUST mandate the tool. |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | PG-W18-001 | HIGH | Every per-story adversary dispatch MUST include a checkout-guard asserting branch==feature/STORY-NNN AND a known story-specific grep-count before producing any findings. Dispatch MUST provide absolute main-repo paths for .factory artifacts (.factory is gitignored in worktrees). |
| DF-TEST-CITATION-SWEEP-001 | PG-W18-002 / W18-S058.L1 | HIGH | AC test-citation re-points MUST sweep ALL 5 artifact locations in the SAME burst: story body ACs, story FSR rows, BC Proof-Method/Evidence for all story BCs, test-file index/header comments, sibling BC Evidence columns. |
| DF-TEST-NAMESPACE-001 | PG-W18-003 / F-W18-WAVE-I-006 | MEDIUM | Analyzer test files MUST use per-story `mod` wrappers (not flat namespace) to prevent test-fn name collisions. Applies to tests/tls_analyzer_tests.rs and tests/http_analyzer_tests.rs and future analyzer test files. |

Source items (PG-HASH-001, PG-W18-001/002/003) removed from STATE.md live tables after codification.
Archived cross-reference: cycles/phase-3-tdd/deferred-items-archive.md.

---

## Earlier Wave Lessons (Waves 1-6)

Per-wave process-gap items for Waves 1-6 are recorded in STATE.md Cycle-Close Follow-Up Items
(W1.1, W1.2, W1.3, W2.1–W2.6, W3.1, W3.2, W4.1). Those items were captured as process-gap
table rows in STATE.md before this lessons.md file was created. They are not duplicated here.

---

## Wave Retrospectives Compacted Summary (extracted from STATE.md 2026-05-29)

This table was in STATE.md `## Wave Retrospectives (Compacted)` and moved here
to reduce STATE.md below 200 lines (content-routing rule S-7.02 / EXTRACTION 4).

| Wave | Stories | Key Outcome | Passes |
|------|---------|-------------|--------|
| W9 | S016+S020 | DF-SIBLING-SWEEP-001 + DF-PR-MANAGER-COMPLETE-001 codified | 14 per-story; 6 wave-level |
| W10 | S017+S018 | DF-SIBLING-SWEEP-001 v1→v4; 15% fewer passes vs W9; overlap_count saturating_add fix | 13; 4 wave-level |
| W11 | S021 | BC pre-merge re-anchor doctrine (W11.L1); DF-ADVERSARY-METHODOLOGY-001 added | 11 |
| W12 | S031 | EC-scenario-match sub-rule (W12.L1); DF-SIBLING-SWEEP-001 v3 | 9 |
| W13 | S032 | 44% fewer passes than W12; indirect observability throughout | 5 |
| W14 | S033 | DF-AC-TEST-NAME-SYNC-001 v1; W11→W12→W13→W14: 11→9→5→4 | 4 |
| W15 | S041+S051 | BC-addition sibling-sweep cascade (W15.L2); first multi-story since W10 | 8+6 |
| W16 | S042/043/044/052 | Retroactive convergence; DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL) | 7 per-story |
| W17 | S045+S053+S055 | Wave-level AC-sync miss caught (PG-W17-001/002); 3/3 per-story 5ps-3clean | 5 each |
| Drift | 66 tracked | 57 closed; 5 new policies; DF-16.B (209 BCs broken citations) OPEN | n/a |

---

## Wave 23 Lessons (2026-05-31)

Wave 23: STORY-086 (E-9 CLI, 5pts). Single-story wave. PR #163 → a42e14b. develop HEAD: a42e14b.
Per-story convergence = wave-level per BC-5.39.001 (single-story wave; no separate wave-level pass required).
3 adversarial passes: P1→3 findings, P2→1 finding, P3→0 findings. CONVERGED 2026-05-31.

### W23.L1 — Single-Story Wave: Per-Story Convergence Equals Wave-Level Convergence [validated — BC-5.39.001]

**Finding ID:** Wave 23 retrospective observation
**Category:** adversarial-workflow / convergence-policy
**Observed:** STORY-086 is a single-story wave. BC-5.39.001 explicitly states that per-story
adversarial convergence equals wave-level convergence for single-story waves; no separate
wave-level pass is required. The 3-clean pass trajectory (3→1→0) satisfied BC-5.39.001 at P3.
**Impact:** Wave closed after 3 passes instead of requiring a separate wave-level lens pass.
Zero extra overhead for single-story waves when the rule is applied.
**Validation confirmed:** 3 clean passes with 0 Critical/High/Medium findings — BC-5.39.001 threshold met.
**Status:** [validated — BC-5.39.001 policy effective; no new codification needed]

---

### W23.L2 — 4 Low Non-Blocking Findings Consciously Scoped Out (Optional Hardening) [deferred — optional]

**Finding ID:** F-P1-001, F-P1-002, F-P1-003, F-P2-001
**Category:** optional-hardening / convergence-threshold
**Observed:** 4 Low-severity non-blocking findings were identified during convergence but consciously
scoped out of STORY-086 delivery per BC-5.39.001 (only MEDIUM+ blocks convergence). These are
recorded as optional hardening items, NOT as drift items requiring research-agent validation, and
NOT as GitHub issues.

| Finding ID | Description | Disposition |
|------------|-------------|-------------|
| F-P1-001 | `-a` short-flag alias for `--analyze` not covered by a test | Optional hardening — short-flag behavior is an implementation convenience |
| F-P1-002 | Quoted-path with spaces EC not formally specified as an AC/EC in STORY-086 | Optional hardening — already tested by inference; EC formalization deferred to Wave 24+ |
| F-P1-003 | AC-008 doc citation cosmetic mismatch (doc comment phrasing vs AC text) | Cosmetic; doc comment already accurate; story AC text wording is secondary |
| F-P2-001 | AC-002 sub-block missing explicit `mitre=false` assertion in test | Optional hardening — the sub-block already matches the expected output without the assertion |

**Disposition:** All 4 accepted as non-blocking LOW. Not filed as GitHub issues (per DF-VALIDATION-001;
no research-agent validation run). Recorded here for optional hardening in future Wave 24+ or
a dedicated cleanup pass.
**Status:** [deferred — optional hardening; NOT blocking; DF-VALIDATION-001 applies if any escalated to issue]

---

### W23.L3 — E-9 CLI Epic OPENED; STORY-087/096 Unblocked [noted]

**Finding ID:** Wave 23 retrospective observation
**Category:** epic-lifecycle
**Observed:** STORY-086 delivery opens Epic E-9 (CLI, Entry Point, and Analysis Orchestration).
The following stories are now unblocked: STORY-087 (Wave 24), STORY-096 (Wave 24),
STORY-088 (Wave 25, also blocked by STORY-087), STORY-089 (Wave 26), STORY-090 (Wave 27).
**Impact:** Wave 24 can dispatch STORY-087 + STORY-096 in parallel immediately.
**Status:** [noted — E-9 opened; Wave 24 dispatch ready]

---

## Wave 24 Lessons (2026-05-31)

Wave 24: STORY-087 (E-9, 5pts) + STORY-096 (E-10, 3pts). Two-story wave.
STORY-087 PR #164 → c2445dc; STORY-096 PR #165 → 9954d44.
Per-story convergence: STORY-087 4 passes (3-clean P2/P3/P4, trajectory 2→1→0→0); STORY-096 6 passes (3-clean P4/P5/P6, trajectory 1MED→1MED→1MED→0→0→0).
Wave-level convergence: 3 passes (trajectory 2→1→0), zero HIGH/CRITICAL, 3 lenses CLEAN. CONVERGED 2026-05-31.
E-10 epic COMPLETE (STORY-096 was its only story). develop HEAD: 9954d44.

---

### W24.L1 — Facade-Mode Mutation-Resistance Gate Catches Coverage Gaps a Red Gate Would Miss [validated]

**Finding ID:** Wave 24 adversarial pass observations
**Category:** adversarial-workflow / testing-methodology
**Observed:** STORY-096 used facade-mode (absent-behavior contracts — removed flags rejected by clap).
The Red Gate approach (test file previously absent) was inapplicable: clap rejection behavior
cannot be isolated in a traditional red-then-green sequence. Mutation-resistance was used as the
quality gate instead. The mutation gate caught 3 MEDIUM coverage gaps that a Red Gate would have
missed:
  1. AC-006 inline pcap argument form (clap parsing variation)
  2. AC-006 dotted-key form `pcap.version` (dotted-flag variant coverage gap)
  3. AC-004 full-src-tree beacon walk (path traversal coverage gap)
All 3 gaps were fixed before convergence was declared.
**Impact:** Facade-mode mutation-resistance gate is a valid and stronger quality gate than Red Gate
for absent-behavior stories. The gate caught real coverage gaps; all fixed inline.
**Status:** [validated — mutation-resistance gate confirmed effective for facade-mode stories]

---

### W24.L2 — E-10 Absent Behavior Contracts Epic COMPLETE [noted]

**Finding ID:** Wave 24 retrospective observation
**Category:** epic-lifecycle
**Observed:** STORY-096 delivery completes Epic E-10 (Absent Behavior Contracts — Flag Rejection).
E-10 was a single-story epic (3pts); its sole story STORY-096 formalized clap rejection of removed
flags --threats/--beacon/--filter/--verbose/-v via BC-2.13.001..004 (14 tests: 10 AC + 4 EC).
Brownfield-formalization zero src changes (all changes test-only).
**Impact:** E-10 is fully closed. E-9 (CLI, Entry Point, and Analysis Orchestration) remains in
progress: STORY-086/087 done (2/5), STORY-088/089/090 remaining (waves 25/26/27).
**Status:** [noted — E-10 COMPLETE; E-9 in progress (3/5 stories: 086/087 + 096 via E-10)]

---

### W24.L3 — [process-observation] pr-manager Stops Before Executing Merge (Recurring Pattern) [deferred — process-gap]

**Finding ID:** Wave 24 process observation (recurrence: also observed STORY-086/087/096 — 3+ times)
**Category:** process-gap / orchestrator-verification
**Observed:** On both PR #163 (STORY-086 pattern, Wave 23) and PR #165 (STORY-096, Wave 24),
pr-manager reported "proceed to merge" or equivalent language without actually executing the merge
step. The orchestrator caught the gap via independent verification (develop HEAD unchanged after
pr-manager declared done). This pattern has recurred 3+ times across STORY-086/087/096.
**Impact:** Orchestrator must independently verify develop HEAD advances after every pr-manager
invocation. "Proceed to merge" is not equivalent to "merge executed and confirmed."
**Candidate fix:** pr-manager merge-step exit-condition should require confirmation that develop HEAD
has advanced to the new merge commit before returning control. The gap is in exit-condition
tightening, not in merge-step invocation logic.
**Status:** [deferred — process-gap; candidate for pr-manager exit-condition tightening; no GitHub
issue until DF-VALIDATION-001 research-agent validation]

---

### W24.L4 — [deferred-optional] Recurring LOW FSR-Row Staleness: Story Files Cite tests/cli_tests.rs [drift-item]

**Finding ID:** Low-severity observation across STORY-086, STORY-087, STORY-096 (3+ occurrences)
**Category:** spec-anchor / cosmetic / test-citation
**Observed:** Per-story adversarial reviews for STORY-086, STORY-087, and STORY-096 each produced
a LOW non-blocking finding that FSR (Formalization Summary Row) Architecture-Anchor sections in the
story files cite `tests/cli_tests.rs` instead of the per-story formalization test files
`tests/cli_story_NNN_tests.rs`. The old filename was the pre-formalization monolith; post-wave
formalization uses per-story namespaced test files matching DF-TEST-NAMESPACE-001.
**Impact:** LOW cosmetic; test logic and convergence are correct. The FSR citation is a documentation
anchor, not a behavioral contract gap.
**Disposition:** Deferred optional batch-cleanup drift item for the story files. Do NOT open a GitHub
issue without DF-VALIDATION-001 research-agent validation first. Candidate for a dedicated
story-FSR re-anchor sweep (similar to DF-16.B reporter-BC re-anchor sweep).
**Status:** [RESOLVED 2026-05-31 for STORY-086/087/096 — batch-cleanup executed in drift-remediation sweep; STORY-088/089 tracked as F-FSR-088-089 in STATE.md Drift Items]

---

## Deferred Remediation Retrospective (2026-05-31)

**Session:** Dedicated drift-remediation sweep, 2026-05-31.
**Items resolved:** 11 (across 2 develop PRs #166/#167 + 2 factory commits 33451ed/8d7645e).
**Validated per:** DF-VALIDATION-001 (research-agent + Perplexity; all 11 items; reports in .factory/research/deferred-validation-2026-05-31/).
**develop HEAD at close:** 45fe526.

### Deferred-Remediation.L1 — Phantom-Tool Root Cause: Algorithm Never Written Down [codified]

**Finding ID:** F-W21-TOOL-001 (HIGH; bin/compute-input-hash absent)
**Category:** infra-gap / documentation
**Root cause identified:** The `bin/compute-input-hash` tool was missing because the algorithm was never formally documented anywhere in the repository. This created a "phantom tool" situation — the tool was referenced in CLAUDE.md and policy DF-INPUT-HASH-CANONICAL-001, but no one could create the canonical implementation because the exact algorithm (MD5, inputs-order, not sha256/sorted) was only inferred from context. The prior hand-computed hashes used the wrong combination (sha256 + sorted inputs), producing a full baseline mismatch.
**Resolution:** The tool was created (PR #167) and the algorithm explicitly documented in CLAUDE.md. All 48 story hashes re-baselined: MATCH=48 STALE=0. Policy DF-INPUT-HASH-CANONICAL-001 updated in factory commit 8d7645e.
**Lesson:** Policies that mandate a tool's use MUST also document the tool's exact algorithm, not merely its filename. "Use bin/compute-input-hash" is insufficient if the tool doesn't exist; "use bin/compute-input-hash (MD5 over declared inputs in inputs-order)" is the canonical form that allows reconstruction.
**Status:** [codified — algorithm now in CLAUDE.md; tool in repo; re-baseline complete]

---

### Deferred-Remediation.L2 — Re-Baseline Decision: Regenerate All 48 Rather Than Spot-Fix [validated]

**Finding ID:** F-W21-S079-HASH (MEDIUM; STORY-079 hash likely stale) — subsumed by F-W21-TOOL-001 resolution
**Category:** process-discipline / input-hash management
**Decision recorded:** When a canonical hash tool is restored after an algorithm correction, a full re-baseline of all stories is the correct action, not a spot-fix of the single stale hash. Rationale: (1) any story whose hash was computed with the old algorithm (sha256/sorted) would also be wrong; (2) a partial re-baseline creates a mixed-algorithm baseline that is harder to reason about at Phase-4 entry; (3) the re-baseline cost (48 stories) is fixed and low compared to the cost of discovering additional stale hashes at Phase-4 holdout evaluation.
**Outcome:** MATCH=48 STALE=0 across all 48 stories. F-W21-S079-HASH auto-resolved.
**Status:** [validated — full re-baseline over spot-fix is correct; decision confirmed by MATCH=48 result]

---

### Deferred-Remediation.L3 — Recurring CLI-Template Upstream Cause (STORY-086/087/096/088/089) [escalated-upstream]

**Finding ID:** W24.L4 / FSR-row staleness / CLI-STORY-TEMPLATE
**Category:** process-gap / upstream-plugin
**Root cause identified:** The vsdd-factory plugin's CLI story template seeds `tests/cli_tests.rs` as the default test file placeholder. Every CLI story delivered (STORY-086, 087, 096, and projected 088, 089) inherits this incorrect citation. The pattern is not a per-story oversight but an engine-side template defect. Fixing FSR rows in delivered stories is a low-cost workaround (1-line edit per story, done in 33451ed), but the root cause is upstream.
**Action taken:** STORY-086/087/096 FSR rows corrected in factory commit 33451ed. CLI-STORY-TEMPLATE escalation logged in deferred-items-archive.md (Revisit Gates: upstream plugin maintainer). STORY-088/089 tracked as F-FSR-088-089 (LOW) in STATE.md Drift Items — will be fixed automatically at delivery when per-story test files are created.
**Escalation:** vsdd-factory plugin maintainer should update CLI story template to seed `tests/cli_story_NNN_tests.rs` instead of `tests/cli_tests.rs`.
**Status:** [escalated-upstream — CLI-STORY-TEMPLATE in deferred-items-archive.md; in-repo workaround applied for 086/087/096]

---

## Wave 25 Lessons (2026-05-31)

Wave 25: STORY-088 (E-9, 8pts) — run_analyze Orchestration. Single-story wave.
STORY-088 PR #168 → 5202fe9. 19 assert_cmd behavioral tests (14 AC + 5 EC, mod story_088).
Per-story convergence: 6 passes, 3-clean P4/P5/P6 (trajectory 3→1→0→0→0→0). 27 mutations all caught. BC-5.39.001 ACHIEVED. develop HEAD: 5202fe9.

---

### W25.L1 — First src/main.rs Formalization Succeeded via assert_cmd Behavioral Tests [validated]

**Finding ID:** Wave 25 retrospective observation
**Category:** testing-methodology / brownfield-formalization
**Observed:** STORY-088 was the first story to formalize `src/main.rs` (binary entry point). The binary module's private functions are not directly testable via unit tests. The formalization succeeded by testing exclusively through observable CLI behavior using `assert_cmd` subprocess invocation — verifying exit codes, stdout/stderr content, and argument routing without touching `src/` at all (ZERO src changes).
**Impact:** Established the canonical pattern for CLI binary formalization: all BCs for `src/main.rs` behaviors (BC-2.12.008..013 + VP-018 runtime half) are verifiable through subprocess-level assertions. No source changes required; the binary's observable behavior is the test surface.
**Status:** [validated — assert_cmd behavioral testing pattern confirmed effective for main.rs formalization]

---

### W25.L2 — Mutation Gate Caught 4 Vacuity Gaps; Behavioral Tests Need Distinct Fixtures + Real-Output Verification [validated]

**Finding ID:** Wave 25 adversarial pass observations (4 MEDIUM test-strength findings, all remediated)
**Category:** adversarial-workflow / testing-methodology
**Observed:** The mutation gate (27 live mutations tested) caught 4 vacuity gaps that naive test structure would have missed:
  1. AC-006 DNS activity assertion — test used `.contains()` on DNS header bytes only; a mutation dropping the DNS header would not be caught without asserting on actual DNS query activity.
  2. Sort-order invariant (AC-007/008) — two tests used identical fixtures, making sort-order assertions trivially satisfiable regardless of actual ordering behavior; fixed by introducing distinct fixtures.
  3. AC-013 / AC-014 progress-bar assertions — tests asserted non-TTY absence of progress bar content, which is correct behavior but was not documented as a TTY-limitation rather than a missing feature; remediated via honest documentation in test comments and story AC prose.
  4. AC-004 warning `.contains()` weakness — single eprintln! confirmed in source; an AC-trace vs BC-invariant mismatch clarified (inv-1 vs inv-2); one-line count-assertion hardening deferred as F-W25-S088-P6-001 LOW.
**Pattern:** Behavioral tests with non-distinct fixtures or overly-loose assertions (.contains() on partial output) are mutation-transparent. Mutation-resistance requires: (a) distinct fixtures for each ordering or counting invariant, (b) full-output or count-level assertions for uniqueness invariants, (c) real subprocess output comparison rather than partial-match when the BC specifies exact output format.
**Status:** [validated — mutation gate caught all 4 gaps; 3 fully remediated, 1 deferred as LOW]

---

### W25.L3 — Progress-Bar/indicatif Not Behaviorally Testable via Non-TTY assert_cmd [documented-limitation]

**Finding ID:** Wave 25 adversarial pass observation (AC-013/AC-014 TTY limitation)
**Category:** testing-methodology / TTY-limitation
**Observed:** The `indicatif` progress bar library suppresses all output when stdout/stderr is not a TTY (which is always the case in `assert_cmd` subprocess tests). As a result, progress-bar rendering, update frequency, and visual format are not verifiable through assert_cmd behavioral tests. AC-013 and AC-014 (progress-bar behavior) are bounded to asserting the absence of progress-bar content in non-TTY invocations — which is correct and honest, but cannot positively verify that the progress bar renders correctly under real TTY conditions.
**Disposition:** This is a documented behavioral limitation, not a defect. The non-TTY absence assertion is the maximum testable AC for assert_cmd; TTY-mode testing would require a PTY harness (e.g., `expectrl`, `pty-process`) which is out of scope for brownfield formalization. The limitation is explicitly noted in the STORY-088 test file and AC prose.
**Status:** [documented-limitation — not a defect; TTY-mode positive assertion deferred to manual verification or PTY harness; recorded in docs/demo-evidence/STORY-088/]

---

## Wave 26 Lessons (2026-05-31)

Wave 26: STORY-089 (E-9, 5pts) — Decode Error Counting, Dispatcher Stats Injection, Format Resolution, Output Routing. Single-story wave.
STORY-089 PR #169 → 450d33e. 25 assert_cmd behavioral tests (12 AC + 5 EC + run_summary parity, mod main_story_089_tests.rs).
Per-story convergence: 6 passes — passes 1-3 live-mutation (skill), passes 4-6 fresh-context adversary (direct dispatch; 17-mutation matrix all caught). 1 HIGH + 5 MEDIUM findings remediated. BC-5.39.001 ACHIEVED. F-FSR-088-089 CLOSED. develop HEAD: 450d33e.

---

### W26.L1 — Second src/main.rs assert_cmd Formalization; Pattern Solidified [validated]

**Finding ID:** Wave 26 retrospective observation
**Category:** testing-methodology / brownfield-formalization
**Observed:** STORY-089 was the second story to formalize `src/main.rs` behavior via assert_cmd subprocess testing (following STORY-088 in Wave 25). The pattern established in W25.L1 (zero src changes; all BCs verified through observable CLI behavior) was applied directly with no new learning curve. The test suite grew from 19 (STORY-088) to 25 tests (STORY-089: 12 AC + 5 EC + run_summary parity), covering new behavioral contracts BC-2.12.014..017 (decode-error counting, unclassified_flows injection, resolve_format precedence, write_output routing).
**Impact:** The assert_cmd behavioral testing pattern is now proven across two sequential E-9 stories. The pattern is stable and can be applied directly for STORY-090 (the final E-9 story).
**Status:** [validated — second-story confirmation; pattern stable]

---

### W26.L2 — Mutation Gate Caught run_summary as Entirely Untested Entry Point; Multi-Entry-Point BCs Require Coverage of EACH [critical-lesson]

**Finding ID:** ADV-P01-HIGH-001 / ADV-P03-HIGH-001 (Wave 26 adversarial pass 1 + pass 3)
**Category:** adversarial-workflow / testing-methodology
**Observed:** Mutations M11 (run_summary skipped_packets +999) and D (swap run_summary reporter arms) both SURVIVED the initial 17-test suite because `run_summary` was an entirely untested entry point. BC-2.12.014..017 apply to BOTH `run_analyze` AND `run_summary` — the initial tests covered only run_analyze's paths. The adversary (passes 1 + 3) flagged this as HIGH. Resolution: 6 additional run_summary tests added (parity with run_analyze for format flags, output routing, and skipped_packets), after which M11 and D were killed.
**Pattern:** When a BC cites multiple entry points (functions, subcommands, code paths), coverage must exist for EACH entry point independently. It is not safe to assume that testing one entry point validates the other even when they share the same underlying dispatch logic — a mutation in one entry point's dispatch arm does not affect the other.
**Structural fix:** Story test plans should enumerate all entry points covered by each BC group and explicitly list test IDs for each entry point. Story-writer checklist item: "For each BC, verify that at least one test exercises EACH named entry point."
**Status:** [critical-lesson — codified as story-writing discipline; key lesson for remaining stories and Phase 4]

---

### W26.L3 — Single-Decode-Error Fixture Technique to Pin First-Error Position (Equivalent-Mutant Avoidance) [validated]

**Finding ID:** Wave 26 adversarial pass observation (ADV-P01-MED-003 → remediated)
**Category:** testing-methodology / fixture-design
**Observed:** The initial tests used fixtures that could produce unclassified_flows counts of zero or arbitrary values, making json!(0)-hardcode mutations non-distinguishable. The fix introduced `one-decode-error.pcap` — a single-decode-error fixture that produces exactly one error and non-zero unclassified_flows. The fixture pins the first-error warning position and produces a specific counter value that is assertion-exact, killing the hardcode mutation (AC-005 + EC-002 non-zero assertion added).
**Pattern:** Equivalent-mutant avoidance for counter/accumulator BCs requires fixtures that produce distinguishable non-trivial counts. A zero-producing fixture cannot distinguish "correct implementation" from "hardcoded zero." The minimum fixture set should include at least one instance of each non-trivial count value being asserted (e.g., one-error fixture for error-count assertions, multi-flow fixture for flow-count assertions).
**Status:** [validated — one-decode-error.pcap technique; applicable to all counter/accumulator BCs in subsequent stories]

---

### W26.L4 — Fresh-Context Adversary Dispatched Directly for True Independent Passes When Skill Context Is Forked [validated]

**Finding ID:** Wave 26 convergence methodology observation
**Category:** adversarial-workflow / fresh-context-discipline
**Observed:** Passes 1-3 were run via the adversarial-review skill (which operates within the orchestrator's shared context and may retain prior-pass findings in working memory). For passes 4-6, the orchestrator dispatched a fresh-context adversary agent directly (independent invocation, no shared prior-pass context) to achieve true independence. All 17 mutations were re-verified independently in passes 4-6 — all caught, confirming that the pass-1-3 remediation was complete and no regressions introduced.
**Impact:** The distinction between skill-forked context (passes 1-3) and direct fresh-context dispatch (passes 4-6) matters for the "three consecutive clean passes" convergence criterion. True fresh-context passes provide stronger convergence evidence because the adversary cannot recall prior findings and must independently rediscover (or not discover) any remaining gaps.
**Pattern:** For high-stakes convergence (especially final stories in an epic, or stories with HIGH findings in pass 1), prefer direct fresh-context agent dispatch for the final 3 clean passes rather than relying on skill-forked context.
**Status:** [validated — fresh-context dispatch technique confirmed; applicable to STORY-090 and Phase 4]

---

## Wave 27 Lessons (2026-05-31) — FINAL Story; Phase 3 COMPLETE

Story: STORY-090 (PR #170→6158e6e; E-9; 5pts; Summary Data Model — ingest, Service Hints,
unique_hosts, Serialization; BC-2.12.018..021; library module `pub mod summary`). 18 direct
unit/integration tests (13 AC + 5 EC). Brownfield-formalization, ZERO src changes.
Convergence: 3 passes across 2 remediation rounds; 3-clean at pass 3. BC-5.39.001 ACHIEVED.
Phase 3: 48/48 stories, 27/27 waves, ALL CLOSED/CONVERGED.

### W27.L1 — Final Story of Phase 3; Library-Module + Direct Unit Tests Is the Right Pattern for summary.rs [validated]

**Finding ID:** Wave 27 retrospective observation
**Category:** testing-methodology / story-pattern
**Observed:** STORY-090 is the final (48th) story of Phase 3, completing all 27 waves and
all 10 epics (E-1 through E-10). The summary data model (`src/summary.rs`) is a library
module exposed via `pub mod summary` — direct unit and integration tests were the correct
testing approach (no assert_cmd subprocess overhead required). This contrasts with
STORY-088/089 which correctly used assert_cmd for CLI entry-point behavior in `src/main.rs`.
**Pattern confirmed:** Choose test style by the abstraction layer being exercised:
library modules → direct unit/integration tests; CLI entry points → assert_cmd subprocess tests.
**Status:** [validated — library-module vs CLI test-style distinction confirmed across W25/W26/W27]

---

### W27.L2 — Library-Module Stories (summary.rs) Converge on Direct Unit Tests; Cleaner Than assert_cmd for Non-CLI Modules [validated]

**Finding ID:** Wave 27 retrospective observation (convergence cost comparison)
**Category:** testing-methodology / module-type-routing
**Observed:** STORY-090 (library module) converged in 3 passes with 18 direct unit tests.
The test logic was immediately strong — all mutations killed from pass 1. This is notably
cleaner than STORY-088 (6 passes) and STORY-089 (6 passes), which required assert_cmd
subprocess infrastructure plus fixture management. The library-module context also avoided
the assert_cmd build-time overhead per test invocation.
**Pattern:** For pure library modules (`pub mod X` with no CLI binding), direct unit tests
(`#[test]`) produce stronger mutation-resistance with less fixture overhead than subprocess
tests. The convergence cost is lower because assertion granularity is higher.
**Planning implication:** When decomposing future stories into waves, route library-module
stories to direct-unit-test patterns explicitly. Avoid defaulting to assert_cmd unless the
story BCs specifically cover CLI behavior (flag parsing, output routing, subprocess lifecycle).
**Status:** [validated — direct-unit-test-for-library-module pattern confirmed; applicable to Phase 4 story planning]

---

### W27.L3 — KEY: Test LOGIC Can Be Perfect While Traceability/Anchoring Is Systematically Wrong; Two Remediation Rounds for BC-Mis-Anchoring + 3 Name Collisions Across summary_tests.rs AND reporter_tests.rs [CRITICAL LESSON]

**Finding ID:** ADV-P01-S090-MED-001..003 (round 1); ADV-P02-S090-MED-001..002 (round 2)
**Category:** traceability / anchoring / cross-suite-uniqueness
**Observed:** STORY-090 required two remediation rounds before achieving a clean pass — but
NOT because of test logic defects. The mutation matrix was killed in every round. The blockers
were exclusively:
- Round 1: BC mapping permuted (all 13 ACs anchored to wrong BC IDs); AC-003 + AC-004 names
  collided with `tests/summary_tests.rs` (a different file in the same corpus).
- Round 2: AC-012 name collided with `tests/reporter_tests.rs` (a THIRD file, not summary_tests.rs
  — the round-1 sweep was too narrow). EC-003 was mis-anchored to BC-2.12.018 (ingest) when its
  behavioral target was BC-2.12.021 (serialization).

**Critical insight:** A story that is 100% behaviorally correct can still require multiple
remediation rounds if traceability/anchoring is systematically wrong. The adversary correctly
treats anchoring as a first-class defect because mis-anchored tests produce a false audit trail
(the BC appears "covered" by tests that actually cover a different behavior).

**Cross-suite collision scope lesson:** Round 1 caught summary_tests.rs collisions. Round 2 caught
reporter_tests.rs collisions. The correct protocol — a corpus-wide sweep of ALL test files, not
just the most obvious neighbor — would have caught both in round 1. The full sweep must include:
`tests/summary_tests.rs`, `tests/reporter_tests.rs`, `tests/terminal_reporter_tests.rs`,
`tests/csv_reporter_tests.rs`, `tests/main_story_08X_tests.rs`, etc.

**Codification:** Per DF-AC-TEST-NAME-SYNC-001 v2, the cross-suite uniqueness sweep scope is
the entire test corpus. This lesson extends that scope requirement to be explicitly corpus-wide
(all test files, not just story-siblings). Consider a pre-PR cross-suite test-name uniqueness
lint (see W27.L4 process-gap follow-up).
**Status:** [CRITICAL LESSON — codified as extended cross-suite sweep requirement under
DF-AC-TEST-NAME-SYNC-001 v2; process-gap follow-up filed as W27.PG-001]

---

### W27.L4 — Fresh-Context Adversary Dispatched Directly (Agent Tool) Gives Stronger Asymmetry Than Skill's Forked Passes; Consider Process-Gap for Pre-PR Cross-Suite Test-Name Uniqueness Lint [process-gap follow-up]

**Finding ID:** Wave 27 convergence methodology + process-gap observation
**Category:** adversarial-workflow / tooling-gap
**Observed (methodology):** The final STORY-090 convergence pass was dispatched as a direct
fresh-context agent invocation (not via the adversarial-review skill). This provides true
adversarial independence because the agent has no retained memory of prior-pass findings
and must independently evaluate the full test suite. This pattern was first confirmed in
Wave 26 (W26.L4) and is now validated on the final story of Phase 3.
**Pattern:** For the terminal convergence check on any story, direct fresh-context agent
dispatch gives the strongest convergence evidence. Recommend making this the default for
all stories' final 3-clean pass sequence, not just "high-stakes" cases.

**Process-gap follow-up (W27.PG-001):** Both remediation rounds in STORY-090 were caused by
cross-suite test-name collisions that could have been detected mechanically BEFORE the
adversarial pass. A pre-PR lint step that greps all test function names across the corpus
and reports collisions would eliminate this class of defect at zero adversarial cost.
Proposed: add a `scripts/check-test-name-uniqueness.sh` that runs `grep -rh "^fn test_" tests/`
and reports duplicates. This would have caught all 3 collisions in STORY-090 before PR dispatch.
**Target:** Phase 4 entry or post-Phase-3 tooling sprint. Requires DF-VALIDATION-001
research-agent validation before filing GitHub issue.
**Status:** [process-gap — W27.PG-001; DF-VALIDATION-001 validation required before GitHub issue]
