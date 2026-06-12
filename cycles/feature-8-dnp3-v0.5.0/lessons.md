---
document_type: lessons-learned
level: ops
version: "1.0"
status: complete
producer: state-manager
timestamp: 2026-06-11T00:00:00Z
cycle: "feature-8-dnp3-v0.5.0"
session_dates: "2026-06-10 to 2026-06-11"
decisions: "D-047..D-054"
inputs: [STATE.md]
traces_to: STATE.md
---

# Lessons Learned — Feature #8 DNP3 / v0.5.0 Release

Durable lessons from the 2026-06-10/11 session covering:
- v0.5.0 release (MITRE ATT&CK-ICS v19 revocation fix, issue #222)
- Feature #8 DNP3 F1 scope, F2 spec (SS-15, ADR-007, VP-023), F3 decomposition
  (STORY-106..110, E-15, waves 35-39)

---

## Process-Level

### PG-5 [process-gap] Sibling-Sweep Must Cover ADRs, domain-debt, cap-10, and Design Drafts

**Tag:** `[process-gap]`

**What happened:** When a technique-ID, enum variant, or count changes, multiple HIGH
adversarial findings (this session: ADR-005/006 emission tables, cap-10 counts) surfaced
because the sibling-sweep that accompanied the change did not include:
- `docs/architecture/decisions/ADR-*.md` emission tables
- `domain-debt.md` technique references
- `cap-10-*` capability files with counts
- `docs/superpowers/specs/` and `docs/superpowers/plans/` design drafts

Each of these is a mandatory sweep target when any technique ID or enum variant is
added, removed, or renamed.

**Rule (DF-SIBLING-SWEEP-001 expansion):** When a technique-ID/enum/count changes,
the sibling-sweep checklist MUST enumerate: `docs/architecture/decisions/ADR-*.md`,
`domain-debt.md`, `cap-10-*`, and `docs/superpowers/` design drafts. Missing any one
of these is a HIGH finding in the next adversarial pass.

_Discovered: F2 adversarial pass, 2026-06-10_

---

### PG-7 [process-gap] BC-INDEX, PRD, and ARCH-INDEX Titles Must Be Updated in the Same Burst as Spec Changes

**Tag:** `[process-gap]`

**What happened:** Multiple adversarial passes caught stale title/count fields in
BC-INDEX, PRD, and ARCH-INDEX that were not updated in the same burst that added or
renamed a field/variant. This became a recurring HIGH finding across F2 and F3.

**Rule:** Any burst that changes a domain-spec section title, BC count, story count,
or wave count MUST update BC-INDEX, PRD headings, and ARCH-INDEX in the same commit.
These index files are mandatory co-change targets — not optional sibling sweeps.

_Discovered: F2/F3 adversarial passes, 2026-06-10_

---

### PG-8 [process-gap] Orphaned-Struct-Field / Partial-Fix-Propagation — Most Recurring This Session

**Tag:** `[process-gap]`

**Frequency:** Recurred across F2 HIGH-1/2, must-add C-1/C-2, and F3 C-1/M-2/F-PG2.
Every remediation burst that added or renamed a flow-state field or a count triggered
a follow-up adversarial pass because the fix was partial.

**What a partial fix looks like:**
- A new/renamed field appears in one copy of the struct sketch (e.g., architecture-delta)
  but not in the companion ADR struct sketch, or vice versa.
- The single reset-owner BC is not updated to include the new field in its reset list.
- All index/PRD/spec-changelog entries are not updated in the same burst.

**Rule (in-burst sibling checklist for new/renamed flow-state field OR
release-target/count change):**
When any burst adds or renames a flow-state field or changes a release-target/count, the
burst MUST atomically update ALL of:
1. Both struct copies (architecture-delta + ADR struct sketch).
2. The single reset-owner BC's reset list.
3. BC-INDEX, PRD, and ARCH-INDEX title/count fields.
4. spec-changelog entry.
5. grep-to-exhaustion across the full repo for the old field/count string before closing.

**Global grep-to-exhaustion (orchestrator-run) is the mechanism that breaks the partial-fix
cycle.** It must be run after any multi-file sweep, not left as an optional step.

_Discovered: F2/F3 adversarial passes, 2026-06-10/11_

---

### PG-9 [process-gap] Research-Validation Passes Catch Real Shipped Defects and Self-Hallucinations

**Tag:** `[pattern-confirmation]`

**What happened (v0.5.0):** A DNP3 research pass incidentally discovered that the shipped
v0.4.0 release was emitting T0855/T0856 — both REVOKED in ATT&CK-ICS v19.0. This was a
genuine shipped-release defect that surfaced only because a fresh research pass questioned
the project's own pinned assumptions. Two independent research passes (DF-VALIDATION-001)
confirmed the revocation; the research agent also caught and discarded its own
deep-research hallucinations via primary-source cross-check.

Adversarial fresh-context passes throughout F2/F3 repeatedly caught real defects (fabricated
technique name in 5 sites, window-reset contradiction, panic-prone arithmetic, un-buildable
orphaned fields) that prior passes and the original author missed.

**Pattern confirmed:** Independent research-validation is not overhead — it is the mechanism
that surfaces defects in assumptions the project has held for multiple waves. DF-VALIDATION-001
(no issue from unvalidated finding) and the adversary-independence wall are working as intended.

_Confirmed: v0.5.0 MITRE v19 remap, F2/F3 adversarial passes, 2026-06-10/11_

---

## Open Follow-Ups (carry into next session)

| Item | Category | Priority |
|------|----------|----------|
| F3 human-gate review — 3 open questions (decomposition granularity, VP-023 placement, linear-chain parallelism) | gate | HIGH |
| F4 start: wave 35 STORY-106 parse-core + VP-023 Kani authoring | next-step | HIGH |
| 6 open Dependabot PRs | maintenance | MEDIUM |
| PCAP-CORPUS-001 (local E2E pcap corpus expansion) | maintenance | MEDIUM |
| Roadmap issues #3, #4, #6 | roadmap | MEDIUM |
| DRIFT-F2-COUNT-001 (F2 count field drift) | drift | LOW |
| DRIFT-SUPERPOWERS-001 (superpowers design draft sync) | drift | LOW |
| PG-5/7/8 codification into policy candidates (DF-SIBLING-SWEEP-001 expansion + new PG-8 rule) | policy | MEDIUM |

---

## Policy Candidates

| Lesson | Proposed Policy / Rule | Scope | Status |
|--------|------------------------|-------|--------|
| PG-5 | Expand DF-SIBLING-SWEEP-001: ADRs + domain-debt + cap-10 + superpowers drafts as mandatory sweep targets | Adversarial-pass entry checklist | proposed |
| PG-7 | BC-INDEX / PRD / ARCH-INDEX titles are mandatory co-change targets (same burst as spec changes) | All burst checklists | proposed |
| PG-8 | In-burst sibling checklist for new/renamed flow-state field or count change (both struct copies + reset-owner BC + indices + spec-changelog + grep-to-exhaustion) | Burst authoring rules | proposed |
| PG-F4-F5-001 | DF-CANONICAL-FRAME-HOLDOUT-001: protocol framing invariants require at least one holdout using canonical spec byte sequence | Holdout authorship, F3 decomposition | proposed |
| PG-F4-F5-002 | Expand DF-ADVERSARY-METHODOLOGY-001: F5 must include BC-completeness sweep (is every BC Invariant implemented?) | F5 holistic review checklist | proposed |
| PG-F4-F5-003 | Embed checkout-guard block verbatim in adversary dispatch template (DF-ADVERSARY-CHECKOUT-GUARD-001 enforcement) | Orchestrator dispatch template | proposed |
| PG-F4-F5-004 | Orchestrator must run cargo fmt before direct-committing agent-authored code | Orchestrator commit procedure | proposed |
| PG-F4-F5-005 | Update pr-manager prompt to emit consolidated report (PR#, pr-reviewer, security, CI) per DF-PR-MANAGER-COMPLETE-001 | pr-manager agent prompt | proposed |
| PG-F4-F5-006 | Pre-implementation agentic-sliced design review as default step for protocol/security-invariant features | F5 remediation playbook | proposed (IP-3) |

---

## F4-F6 Session Lessons (appended 2026-06-12)

_Session arc: F4 delivery (waves 37-39, STORY-108/109/110) + F5 scoped adversarial +
F5 remediation + F6 formal hardening. Full analysis: `session-review-f4-f6.md`._

---

### PG-F4-F5-001 [process-gap] Canonical-Frame Holdout Cross-Validation for Protocol Framing Invariants

**Tag:** `[process-gap]`

**What happened:** The `is_master_frame` DIR-bit mask was wrong (0x10 instead of 0x80,
where DIR is bit 7 per IEEE 1815 DNP3). The bug was latent since STORY-107 (wave 36) and
survived approximately 30 per-story adversarial passes across STORY-107, 108, 109, and 110.
Root cause: the per-story adversary reviews code against the project's own BC and tests.
When the BC (PC5) and the tests both used the same wrong mask (an internally self-consistent
error), the adversary had no external reference to detect it.

The fix was found by the F5 agentic-sliced pre-implementation review, which cross-checked
the mask against the DNP3 link-layer bit layout in IEEE 1815 independently.

**Lesson:** Per-story adversarial cannot detect protocol-spec level errors that are
internally self-consistent between code, tests, and project BCs. The only reliable
mechanism is cross-validation against the authoritative protocol specification, either
via a holdout scenario that uses a canonical spec byte sequence, or via a fresh-context
adversary given the spec as a reference.

**Rule:** For any protocol-framing invariant (direction bits, function codes, frame
types, magic bytes), at least one holdout scenario MUST use a canonical byte sequence
from the authoritative protocol specification — NOT a value derived from the project's
own BCs. The holdout scenario must cite the spec section and the canonical byte value
explicitly. Stories implementing protocol-framing invariants must require this in their
AC.

**Proposed policy:** `DF-CANONICAL-FRAME-HOLDOUT-001` in `.factory/policies.yaml`.

**Follow-up recommendation:** Add to F3 story decomposition checklist. Warrant: YES —
this is a self-improvement story candidate (or policy-file change) before the next
protocol feature begins.

_Discovered: F5 agentic-sliced pre-impl adversarial review, 2026-06-12_

---

### PG-F4-F5-002 [process-gap] BC-Set-Completeness Check in F5 Holistic Review

**Tag:** `[process-gap]`

**What happened:** BC-2.15.010 Invariant 5 (unexpected-source detection) was entirely
absent from the implementation after 5 stories were delivered. The per-story adversarial
passes correctly verified that existing detection branches satisfied their own BCs. None
of the 34 per-story passes checked whether Invariant 5 had any corresponding code path.
The F5 holistic adversarial review found the gap by scanning the full delta for coverage
of each BC Invariant.

**Lesson:** Per-story adversarial reviews correctness of what exists. Completeness of
what was specified (is every BC Invariant implemented?) requires a different check — a
scan of the BC corpus against the implementation, not a scan of the implementation
against each story's BCs.

**Rule:** F5 scoped adversarial review MUST include a mandatory first step: for each BC
in the feature's spec that specifies a detection invariant, guard condition, or emission
requirement, confirm that a corresponding implementation path EXISTS in the feature delta.
A BC invariant with no corresponding implementation path is a P0 BLOCKER finding.

**Proposed policy change:** Expand `DF-ADVERSARY-METHODOLOGY-001` to include the
completeness sweep as a numbered step.

**Follow-up recommendation:** Warrant: YES — add to DF-ADVERSARY-METHODOLOGY-001 scope
before next feature-level F5 review.

_Discovered: F5 holistic adversarial delta review, 2026-06-12_

---

### PG-F4-F5-003 [process-gap] Pre-Implementation Adversarial Design Review for High-Risk Features

**Tag:** `[codified]`

**What happened:** Before TDD implementation of F5 remediations, an agentic-sliced
adversarial review was run in 3 parallel slices (F-001 design, F-003 design,
spec-consistency). The review found 2 design BLOCKERs before any code was written:

1. F-A-001: DIR-bit mask error (described under PG-F4-F5-001 above)
2. F-B-002: Overflow `clear+return` in the proposed resync design created a data-loss
   path (Crain-Sistrunk evasion vector) that was not visible in the initial design sketch

Both required ARCHITECT REVISION-2 directives. The estimated savings: 5-8 convergence
passes that would have been needed to discover these flaws post-implementation.

**Lesson:** For designs touching protocol framing invariants, multi-path detection logic,
or security-invariant BCs, a pre-implementation adversarial design review is high-leverage.
The cost is one additional step before TDD; the benefit is avoiding design-rework in the
middle of the convergence cycle.

**Rule:** When F5 holistic review identifies issues involving (a) protocol framing
invariants, (b) detection logic with security-invariant BCs (T1692.001/T0827 class),
or (c) multi-path byte-walk algorithms, dispatch an agentic-sliced pre-implementation
adversarial design review (2-4 slices, one per concern area) BEFORE TDD authoring begins.
Each slice receives: the adjudication directive, the relevant BCs, and the authoritative
protocol spec section. Any BLOCKER finding triggers ARCHITECT REVISION before coding begins.

**Follow-up recommendation:** Warrant: YES — codify as a Step 0 in the F5 remediation
playbook template with trigger conditions listed above.

_Discovered/confirmed: F5 pre-impl adversarial design review, 2026-06-12_

---

### PG-F4-F5-004 [process-gap] Mandatory Adversary Checkout-Guard in Worktree Dispatch

**Tag:** `[process-gap]`

**What happened:** F5 convergence pass P9 reviewed `develop` HEAD (pre-fix state) instead
of the feature worktree (post-fix state). The adversary produced 3 BLOCKER findings
describing code that had already been fixed in the worktree. The orchestrator caught the
error by cross-checking the file paths cited; P10 was re-dispatched on the correct
worktree and returned CLEAN.

Impact: 1 wasted convergence pass (~20-30 minutes). If uncaught, would have triggered
unnecessary rework.

**Lesson:** An adversary reviewing the wrong codebase produces invalid findings. Policy
DF-ADVERSARY-CHECKOUT-GUARD-001 exists and is correct, but it must be embedded in the
dispatch template — not added post-hoc when a failure is suspected. The later passes
(P10) used the guard explicitly and worked correctly.

**Rule (operational):** The orchestrator's adversarial dispatch template MUST include
a checkout-guard block as the FIRST instruction in every worktree-based review pass:

1. Run `git -C <worktree_path> log -1 --format='%H %s'` and log the result.
2. Confirm the HEAD SHA matches the expected worktree HEAD provided by the orchestrator.
3. Spot-check: read the file that contained the most recently fixed BLOCKER finding;
   confirm the fix is present (cite the line checked).
4. If either check fails: STOP. Report "CHECKOUT GUARD FAILED — invalid context." Do
   NOT produce findings. The pass is invalid and must be re-dispatched.

**Follow-up recommendation:** Warrant: YES — update the adversarial dispatch template
to embed this block verbatim (not as a reminder, but as mandatory instructions).

_Discovered: F5 convergence pass P9, 2026-06-12_

---

### PG-F4-F5-005 [process-gap] Orchestrator Must Run cargo fmt Before Direct-Committing Agent-Authored Code

**Tag:** `[process-gap]`

**What happened:** CI Format gate failed on PRs #228 (STORY-109) and #231 (F6) because
the orchestrator committed agent-authored test changes via direct git, bypassing the
authoring agent's `cargo fmt` step. The authoring agent runs `cargo fmt` as part of its
delivery step; the direct-commit path does not. Frequency: 2/5 PRs (40%) in this session.

**Lesson:** When the orchestrator takes code from an agent's output and commits it
directly (without routing through the agent's normal delivery step), the format step is
skipped. This predictably produces CI Format gate failures.

**Rule:** When committing agent-authored code via direct git (not via the authoring
agent's own commit step), the orchestrator MUST run `cargo fmt` (or `cargo fmt --check`
as a gate) before the commit. Skipping this step WILL produce a CI Format gate failure.
The rule applies to: test changes received from the test-writer agent, fixup patches
received from the implementer agent, or any code received as text output rather than
via the agent's normal commit path.

**Follow-up recommendation:** Warrant: YES — add to orchestrator commit procedure as a
mandatory pre-commit check. This is a simple procedure change (1 line: `cargo fmt`
before `git commit`).

_Discovered: STORY-109 PR #228 and F6 PR #231 CI failures, 2026-06-12_

---

### PG-F4-F5-006 [process-gap] pr-manager Must Return Consolidated Report (Policy Non-Compliance)

**Tag:** `[process-gap]`

**What happened:** Across multiple PRs in this session (#227, #228, #229, #230),
pr-manager returned only the security review verdict. The pr-reviewer approval verdict,
PR number, and CI status were not included, requiring the orchestrator to dispatch
pr-reviewer separately and query CI status manually each time.

Policy DF-PR-MANAGER-COMPLETE-001 already requires a consolidated report. The agent is
not complying with an existing policy. This is an agent-prompt gap.

**Lesson:** Policies are only enforced when embedded in the agent prompt. DF-PR-MANAGER-
COMPLETE-001 exists in policies.yaml but the pr-manager agent prompt does not require
the consolidated output format.

**Rule (operational):** The pr-manager agent prompt MUST require the following
consolidated report as the mandatory final section of every run:

```
## Consolidated Gate Report
- PR number: #NNN
- pr-reviewer verdict: [APPROVE / REQUEST_CHANGES / REJECT]
- security verdict: [APPROVE_WITH_NOTES / APPROVE / REJECT] (N CRITICAL, N HIGH, N MED, N LOW)
- CI status: [GREEN / FAILING] (job list with pass/fail per job)
- Merge commit: <SHA> (if merged)
- Blocking items: <list or "none">
```

**Follow-up recommendation:** Warrant: YES — update pr-manager agent prompt immediately.
DF-PR-MANAGER-COMPLETE-001 already exists; this is a compliance fix, not a new policy.

---

## F7 Delta-Convergence + v0.6.0 Release Lessons (appended 2026-06-12)

_Session arc: F7 delta-convergence (6 fresh-context adversarial passes; final 3 consecutive
CONVERGED) + consistency audit + v0.6.0 release (PR #234 → main 3e29891; tag v0.6.0; 4 binaries).
Decisions D-063..D-064. Full analysis: `session-review-f7-release.md`._

---

### PG-F7-001 [process-gap] Input-Hash Re-Stamp Must Follow BC Version Bumps in the Same Burst

**Tag:** `[process-gap]`

**What happened:** F5 and F6 bumped BC versions on BCs 2.15.009, 2.15.010, 2.15.014, and
2.15.016. The `bin/compute-input-hash --scan` step was run at F6 completion and recorded
MATCH=62/STALE=0 in STATE.md. However, a live scan at F7 entry revealed STALE=4 (STORY-106,
STORY-107, STORY-108, STORY-110). The STATE.md value was recorded after a partial re-stamp
pass that missed 4 stories because those stories reference the bumped BCs in their `inputs:`
lists but were not individually re-stamped after the BC content changes that occurred during
F5 adversarial remediation.

**Impact:** STATE.md recorded a convergence it did not actually have. The F7 pre-pass had to
remediate the drift before proceeding — adding an extra burst before the first adversarial pass.

**Rule:** Any BC version bump (content change, not merely a prose clarification) MUST be
followed in the SAME burst by:
1. Running `bin/compute-input-hash --scan` to identify all stories with newly-STALE hashes.
2. Running `bin/compute-input-hash --write --scan` to re-stamp them.
3. Committing the re-stamped stories to factory-artifacts AND recording the resulting
   MATCH=N/STALE=0 in STATE.md.

F4, F5, and F6 gate checklists MUST include a live `bin/compute-input-hash --scan` run as
a mandatory gate step rather than trusting the last STATE.md-recorded value. The recorded
value becomes stale the moment any `inputs:` file changes.

**Proposed policy:** Add a sub-rule to DF-INPUT-HASH-CANONICAL-001: "A BC version bump is a
mandatory trigger for a full --scan + --write cycle in the same burst."

_Discovered: F7 pre-pass live scan, 2026-06-12_

---

### PG-F7-002 [process-gap] Holdout Assertion Re-Validation When Implementation Behavior Changes

**Tag:** `[process-gap]`

**What happened:** F5 remediation introduced the inline-resync behavior (F-F5-003 adjudication,
ADJ-001 Addendum Q2): when an unexpected carry overflows the counter window, the counter resets
to 0 rather than the prior value. Unit tests (BC-2.15.016) were updated to reflect this
behavior. Holdout HS-W36-001, however, retained its original carry assertion (carry==292)
rather than the post-resync value of 0. The F7 adversarial pass caught this as F-CC-001 [HIGH].

The pattern: a behavior-changing fix was followed by unit-test updates but NOT by a holdout
re-validation sweep. The holdout and the unit tests diverged on the same behavior.

**Rule:** When an adjudication changes implementation behavior (not merely clarifies it),
the remediation burst MUST include a mandatory step: grep the holdout corpus for every
scenario that touches the changed code path and validate each holdout assertion against the
new behavior. A holdout assertion that conflicts with updated unit tests is a failing [HIGH]
finding at the next adversarial pass. This step is separate from and in addition to the unit
test update.

**Proposed policy:** Add to F5 remediation playbook: "After adjudication-driven behavior
change, run: grep the holdout corpus for any scenario asserting on the affected detection
output. Validate each match against the post-adjudication behavior. Update or document
justification for each assertion."

_Discovered: F7 pass 1 finding F-CC-001, 2026-06-12_

---

### PG-F7-003 [process-gap] Adjudication Must Verify That Governing BC Text Matches the Ruling

**Tag:** `[process-gap]`

**What happened:** ADJ-001 ruled that BC-2.15.009 described "initial delivery only" behavior
and that "BC-2.15.009 does not need updating." At the time the ruling was written, BC-2.15.009
Invariant 1 text described a "cross-segment 16-byte bail" that was explicitly NOT the
implemented behavior. The BC text was stale relative to the adjudicated/implemented behavior.
ADJ-001's statement that the BC needed no update was wrong because the BC text already
contradicted the ruling — but no one re-read the BC text when writing ADJ-001.

Finding F-S1-001 [HIGH] surfaced this at F7. BC-2.15.009 required a v1.3 update to remove
the never-implemented cross-segment bail language and align with ADJ-001.

**Rule:** When authoring an adjudication, the adjudicating agent MUST:
1. Read the current text of every BC named in the ruling (not rely on memory of the BC).
2. Confirm the BC text matches the ruling's description of current behavior.
3. If the BC text contradicts the ruling, update the BC as part of the adjudication burst —
   the adjudication MUST NOT claim "BC does not need updating" without having read the BC.

**Proposed policy:** Add to F5 adjudication authorship rules: "Before finalizing an
adjudication, Read() the current content of each governing BC. If the BC Invariant text
contradicts the ruling, the BC requires a version bump in the same burst."

_Discovered: F7 pass 1 finding F-S1-001, 2026-06-12_

---

### PG-F7-004 [process-gap] Partial-Fix Sibling Sweep Must Propagate to BC-INDEX Titles and Story Body Notes

**Tag:** `[process-gap]`

**What happened:** F-S1-001 was filed and remediated: BC-2.15.009 text was updated (v1.3),
removing the stale cross-segment bail language. The fix was correct on the BC body. However,
a re-pass (F-N-001 [HIGH]) caught that the BC-INDEX title for BC-2.15.009 still reflected the
old behavior description, and the STORY-106 body note still cited the old Invariant 1 wording.
The partial-fix created a sibling regression that was caught only one pass later.

**Rule:** This is an instance of DF-SIBLING-SWEEP-001 that deserves a concrete protocol-BC
sub-rule: when a protocol BC Invariant text is corrected, the mandatory sweep MUST include:
1. BC-INDEX entry title for the corrected BC.
2. All story body sections (Notes, AC rows, Architecture Mapping) that cite the corrected
   Invariant text.
3. Any holdout scenario notes that reference the invariant by description.

"The BC body fix is not complete until its BC-INDEX title and all story citations are
consistent with the new text."

_Discovered: F7 pass 2 finding F-N-001, 2026-06-12_

---

### PG-F7-005 [process-gap] Story Status Lifecycle Must Advance to Completed at Delivery/Merge

**Tag:** `[process-gap]`

**What happened:** F-CC-002 [HIGH] found that STORY-106 through STORY-110 status was still
marked `draft` in both the story body frontmatter and the STORY-INDEX, despite all five
stories having been delivered via merged PRs (#225–#229). Additionally, wave 37-39 delivery
rows were absent from the STORY-INDEX.

**Rule:** The per-story delivery close-out procedure MUST include:
1. Update story frontmatter status from `draft` to `completed`.
2. Update STORY-INDEX status column to `completed` with the merge commit SHA.
3. Add or verify the wave delivery row exists in the STORY-INDEX for the story's wave.

This is a mandatory step in the story-delivery close-out, not optional housekeeping. A
story merged to develop but still marked draft creates a false picture of the backlog and
will be flagged as [HIGH] at the next convergence audit.

_Discovered: F7 consistency audit finding F-CC-002, 2026-06-12_

---

### PG-F7-006 [process-gap] Doc Roadmap Accuracy Must Be Updated at Delivery, Not at Release

**Tag:** `[process-gap]`

**What happened:** F-CC-003 [HIGH] and F-CC-004 [HIGH] found that the README listed shipped
DNP3 and Modbus analyzers as "planned/roadmap" features, and the CHANGELOG had no DNP3 entry
despite DNP3 having been fully delivered and hardened. These gaps required last-minute docs
PRs (#232 and #233) during F7 to pass the docs dimension of the 5-dim convergence gate.

**Rule:** Shipping a feature (i.e., merging the final story of the feature to develop) MUST
trigger a same-burst documentation update:
1. Move the feature from the README "Planned" / "Roadmap" section to the implemented features
   section.
2. Add a CHANGELOG entry in the `[Unreleased]` section describing the feature.

These are mandatory delivery close-out steps, not deferred to release-prep. If they are
deferred, they create a mismatch between develop's code and its docs that will fail the F7
docs convergence dimension and require emergency PRs. Deferring to release-prep also makes
the CHANGELOG unreliable during the interval between delivery and release.

_Discovered: F7 pass 1 findings F-CC-003/F-CC-004, 2026-06-12_

---

### PG-F7-007 [process-gap] Async Workflow Verification Before Reporting CI/Release Asset State

**Tag:** `[process-gap]`

**What happened:** During v0.6.0 release, a devops agent reported "release.yml does not
exist / no binaries built" after the tag was pushed. In fact, release.yml existed and did
build 4 binaries — the workflow was triggered by the tag push and had not yet completed when
the agent checked. The agent reported an absence based on a snapshot taken before the
async workflow finished.

**Rule:** Before reporting the absence of CI workflow outputs or release assets (binaries,
attestations, release notes), an agent MUST:
1. Verify the workflow exists by reading `.github/workflows/` — not by checking whether
   outputs have appeared yet.
2. If the workflow is tag-triggered, wait at least one polling cycle (or check
   `gh run list --workflow=release.yml --limit=5`) to confirm the run was triggered.
3. Report the run ID and status rather than the presence or absence of artifacts when the
   run may still be in progress.

Reporting "missing" for an in-flight async workflow is a false alarm that can cause
unnecessary remediation actions.

_Discovered: v0.6.0 release devops check, 2026-06-12_

---

### PG-F7-008 [meta-lesson] "Phase Marked COMPLETE in STATE" Does Not Mean Converged

**Tag:** `[meta-lesson]` `[process-gap]`

**What happened:** This session provides the clearest evidence yet for the load-bearing
nature of fresh-context F7 convergence audits. At F7 entry, the STATE.md read:
- F4 COMPLETE, F5 COMPLETE (adversary gate 3/3), F6 HARDENED
- input_drift MATCH=62/STALE=0

A fresh-context audit found:
- STALE=4 (input-hash not as claimed) — PG-F7-001
- F-S2-001 CRITICAL: canonical holdout derived circularly — PG-F4-F5-001 violation surviving
  into F7
- F-S1-001 HIGH: BC text contradicting the governing adjudication
- F-N-001 HIGH: partial fix not propagated
- F-CC-001 HIGH: holdout assertion contradicting implemented behavior
- F-CC-002/003/004 HIGH: status and docs gaps

Six passes and multiple remediation rounds were required before the first 3-consecutive-CLEAN
streak. These were not cosmetic findings; they included a CRITICAL policy violation
(DF-CANONICAL-FRAME-HOLDOUT-001 violated in a holdout that ADJ-001 Addendum Q2 was specifically
written to address).

**Meta-rule:** The F7 fresh-context convergence audit is not a rubber-stamp. It is a separate
quality gate that operates with different context from the phases that preceded it. Prior
"COMPLETE/HARDENED" markings in STATE.md record that the phase's own checklist was satisfied,
not that a fresh adversary will find nothing. The 3-consecutive-CLEAN streak of 6 passes was
needed precisely because prior in-context phases share context with the artifacts they produce.

Fresh-context adversarial divergence from STATE.md's recorded convergence is expected, not
alarming. The F7 gate MUST NOT be shortened on the basis of prior STATE.md phase markings.

_Observed pattern: F7 delta-convergence, 2026-06-12 — confirmed across DNP3 (6 passes),
greenfield (14 passes), and MITRE v19 maintenance (3 passes)._

---

## Open Follow-Ups — F7 + Release (carry into next session)

| Item | Category | Priority | Codification target |
|------|----------|----------|---------------------|
| PG-F7-001: BC-version-bump → input-hash re-stamp rule | policy sub-rule | HIGH | DF-INPUT-HASH-CANONICAL-001 addendum |
| PG-F7-002: Holdout re-validation after behavior-changing adjudication | playbook step | HIGH | F5 remediation playbook |
| PG-F7-003: Adjudication must read and verify BC text | adjudication rules | HIGH | F5 adjudication authorship checklist |
| PG-F7-004: BC-INDEX + story-body sibling sweep on protocol BC text fix | DF-SIBLING-SWEEP extension | HIGH | DF-SIBLING-SWEEP-001 protocol-BC sub-rule |
| PG-F7-005: Story status lifecycle close-out | delivery close-out | MEDIUM | Per-story delivery checklist |
| PG-F7-006: Docs update at delivery, not at release | delivery close-out | MEDIUM | Feature-delivery close-out checklist |
| PG-F7-007: Async workflow verification before reporting absence | agent dispatch rules | LOW | Orchestrator devops checklist |
| PG-F7-008: Meta-lesson — F7 gate is load-bearing, not a rubber stamp | culture/process | (standing practice) | Session-review template note |
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 engine follow-up (version_sources in human_approval_prompt template) | engine deferred | MEDIUM | Factory engine template; DEFERRED — no self-improvement story yet |

---

## Policy Candidates (F7 + Release)

| Lesson | Proposed Policy / Rule | Scope | Status |
|--------|------------------------|-------|--------|
| PG-F7-001 | DF-INPUT-HASH-CANONICAL-001 sub-rule: BC version bump triggers mandatory --scan+--write in same burst | F4/F5/F6 gate checklists; all BC-edit dispatches | proposed |
| PG-F7-002 | F5 remediation playbook Step: grep holdout corpus for scenarios touching changed code path; validate assertions against new behavior | F5 remediation playbook | proposed |
| PG-F7-003 | Adjudication authorship rule: Read() every named BC before finalizing "BC needs no update" | F5 adjudication dispatch | proposed |
| PG-F7-004 | DF-SIBLING-SWEEP-001 protocol-BC sub-rule: BC-INDEX title + story-body notes are mandatory sweep targets when BC Invariant text changes | DF-SIBLING-SWEEP-001 v5 | proposed |
| PG-F7-005 | Per-story delivery close-out: story frontmatter + STORY-INDEX status MUST advance to completed at merge | Delivery close-out checklist | proposed |
| PG-F7-006 | Feature delivery close-out: README planned→implemented + CHANGELOG Unreleased entry in the delivery burst | Feature delivery close-out | proposed |
