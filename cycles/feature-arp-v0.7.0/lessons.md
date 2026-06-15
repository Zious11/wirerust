---
document_type: lessons-learned
cycle: feature-arp-v0.7.0
producer: state-manager
timestamp: 2026-06-15T00:00:00Z
---

# Lessons Learned — feature-arp-v0.7.0

Process-gap items recorded per S-7.02 codification discipline. Each item needs
a follow-up story OR a justified deferral entry. Items flagged `[process-gap]`
below are candidates for policy codification.

---

## [process-gap] PG-ARP-F4-REDBANNER-SWEEP

**Source:** STORY-112 Step-4.5 adversarial convergence (multiple comment-fix bursts)

**Observation:** The RED-gate banner sibling-sweep was missed across 3 successive
comment-fix bursts. The module docstring was fixed (F-2), but per-test section
banners inside `bc_2_16_story112_arp_tests.rs` were left stale — including the
AC-004 banner block, which a later fresh adversarial pass escalated from MEDIUM
to HIGH.

This is a recurrence of DF-SIBLING-SWEEP-001 in the doc-comment dimension.
When a module-level status changes (RED→GREEN), the sweep must enumerate:

1. Module-level docstrings
2. Per-test section banners (all of them, not just the first encountered)
3. Any inline `// RED GATE` / `// TODO` comments referencing the transitional error string
4. Story frontmatter changelog notes referencing old-state language

**Candidate policy extension:** Extend DF-SIBLING-SWEEP-001 (or create a new
sub-rule `DF-RED-BANNER-SWEEP-001`) to enumerate per-test section banners and
doc-comments as explicit sibling targets whenever a module-level implementation
status changes.

**Status:** DEFERRED — needs codification follow-up (next feature cycle or
dedicated housekeeping story).

---

## [process-gap] PG-ARP-F4-PRECLEAR-PROPAGATION

**Source:** STORY-112 Step-4.5 adversarial convergence (AC-004 banner re-escalation)

**Observation:** The orchestrator propagated a prior adversarial pass's
"acceptably-nuanced / leave as-is" pre-clearance into a fix dispatch for the
AC-004 banner. A later fresh-context pass overturned this pre-clearance as HIGH
(the region was still present-tense stale-RED, which the fresh context correctly
identified as a state contradiction).

**Root cause:** Fix dispatches must not inherit pre-clearances from an earlier
adversarial pass. Each fresh adversarial context examines the full perimeter
without inherited verdicts. Pre-clearance from pass N does not bind pass N+1.

**Candidate policy extension:** Add explicit language to DF-ADVERSARY-METHODOLOGY-001
or the per-story adversarial dispatch template: "Fix dispatches MUST NOT carry
forward 'leave as-is' verdicts from a prior adversarial pass. Each fresh
adversarial context is pre-clearance-free."

**Status:** DEFERRED — policy codification follow-up (next feature cycle).

---

## [process-gap] PG-ARP-F4-GUARD-WORDING

**Source:** STORY-112 Step-4.5 adversarial convergence (checkout-guard accuracy)

**Observation (F-4 in adversarial review):** An adversary checkout-guard
premise stated "main repo does NOT have this function" (referring to
`extract_arp_frame`). This was inaccurate: the main-repo `develop` branch at
`cced898` does carry `extract_arp_frame`, introduced by STORY-111's
non-panicking None placeholder. The function BODY, not the function PRESENCE,
is the distinguishing characteristic between STORY-111 stub state and
STORY-112 implementation state.

**Root cause:** The checkout guard keyed on function presence rather than body
content. The correct discriminator is:
- Stub state: `extract_arp_frame` body returns `None` unconditionally (placeholder)
- Implemented state: `extract_arp_frame` body performs hw/proto type + size
  validation and field copy

The transitional error string `"ARP extraction not yet implemented"` (in
`decode_packet`) is another valid body-content discriminator.

**Candidate policy extension:** Extend DF-ADVERSARY-CHECKOUT-GUARD-001 guidance:
"Checkout guards for stub-vs-implementation transitions MUST key on BODY CONTENT
(placeholder behavior vs real logic) rather than function presence. The presence
of a function is not a reliable discriminator after the stub commit."

**Status:** DEFERRED — extend DF-ADVERSARY-CHECKOUT-GUARD-001 (engine agent-prompt
note or policy codification follow-up).

---

## [process-gap] PG-ARP-F4-DEMO-LEAK

**Source:** STORY-112 pre-PR diff inspection (2026-06-15)

**Observation:** The demo-recorder agent committed 4 gif+webm+tape recording sets
(demo binaries) to the develop-bound worktree branch under `.factory-demos/STORY-112/`.
This dodged the `.factory/` worktree ignore rule because `.factory-demos/` is a
different directory name — not covered by the develop branch's `.gitignore` entry.

The leak was caught by a pre-PR diff inspection. Commit 76bdf16 (demo binary commit
on the worktree branch) was dropped, and `.factory-demos/` was added to `.gitignore`
via commit bec7a76, which shipped on develop in PR #238.

**Root cause:** The demo-recorder dispatch template targeted a develop-bound worktree
path (`.factory-demos/`) rather than the factory-artifacts branch (`.factory/demo-evidence/`).
Demo evidence is factory-artifacts-only content.

**Lesson:** Demo evidence belongs ONLY on the factory-artifacts branch under
`.factory/demo-evidence/`. The demo-recorder MUST NOT commit demo artifacts
(gif/webm/tape/binary) to develop-bound worktree branches.

**Candidate fix:**
1. Demo-recorder dispatch template must target `.factory/demo-evidence/` on the
   factory-artifacts worktree, OR commit evidence to a fully gitignored path that
   is never staged to a develop-bound branch.
2. The orchestrator MUST run a pre-PR diff check for binary/demo artifacts
   (gif, webm, tape, mp4, png above a threshold) before dispatching pr-manager.
   Any such artifact detected on a develop-bound branch is a hard STOP.

**Status:** DEFERRED — demo-recorder dispatch template update + orchestrator
pre-PR binary-leak check (candidate for next engine sprint).

---

## [process-gap] PG-ARP-F4-PRMGR-MERGE-SHORTSTOP (RECURRENCE #3)

**Source:** STORY-112 PR #238 delivery (2026-06-15). Third recurrence this feature cycle.

**Observation:** pr-manager again halted at step 6 (APPROVE) without executing steps
7-9 (merge + confirm + consolidated report). Required an orchestrator "merge NOW"
SendMessage to complete. The exact same pattern occurred at STORY-111 (PR #236)
and at the DNP3 F5 cycle.

**Root cause:** pr-manager interprets its mandate as obtaining approval rather than
driving the PR to a merged state. The 9-step protocol is not self-enforcing.

**Escalation note:** Three recurrences in one feature cycle (STORY-111, STORY-112,
and at least one DNP3 F5 PR). This has crossed the threshold for engine-level
escalation. DF-PR-MANAGER-COMPLETE-001 (HIGH) is already filed; this recurrence
should be referenced when that policy is enforced or escalated to CRITICAL.

**Candidate fix:** The pr-manager dispatch template must include an explicit
"DO NOT STOP AT APPROVE — execute steps 7-9 (merge, confirm CI green, consolidated
report) before returning" instruction, and the orchestrator should verify merge
completion before declaring the PR cycle closed.

**Status:** DEFERRED — engine dispatch template hardening; escalate
DF-PR-MANAGER-COMPLETE-001 recurrence count to 3 in policy registry.

---

## [process-gap] PG-ARP-F4-PRMGR-MERGE-SHORTSTOP (RECURRENCE #4)

**Source:** STORY-113 PR #239 delivery (2026-06-15). Fourth recurrence this feature cycle.

**Observation:** pr-manager again stopped at step 6 (APPROVE) on PR #239 without
executing steps 7-9 (merge + confirm CI + consolidated report). Required an orchestrator
SendMessage to complete the merge. This is the 4th consecutive recurrence this feature
cycle (PRs #236 STORY-111, #238 STORY-112, #239 STORY-113, plus the DNP3 F5 instance
that seeded the pattern).

**Pattern confirmed:** Every ARP-feature PR has required manual merge-completion
intervention. The pr-manager agent interprets its job as securing APPROVE, not as
driving the PR to a merged state. The 9-step dispatch protocol is structurally
under-weighted toward completion vs. the review loop.

**Escalation:** Four consecutive recurrences in one feature cycle is a confirmed
agent-prompt defect in the vsdd-factory pr-manager. The steps-7-9 completion
instruction is present in the dispatch protocol but is insufficiently weighted
or parsed relative to the review-loop steps. This should be filed as an
agent-prompt-defect against the vsdd-factory pr-manager after DF-VALIDATION-001
research-agent validation.

**Proactive mitigation for STORY-114:** The orchestrator must explicitly include
"DO NOT STOP AT APPROVE — you MUST execute steps 7-9 (merge, confirm CI green,
consolidated report) before returning to orchestrator" in the pr-manager dispatch
message, AND verify merge completion before closing the PR cycle.

**Note on AC-009 recurrence:** The two non-blocking items fixed pre-merge on PR #239
(AC-009 D12 evidence assertion strengthened + stale summary-path comment corrected,
commit a73fbd6) were caught by pr-reviewer fresh eyes. The evidence-assertion
weakness on AC-009 is a recurrence of PG-ARP-F4-PROXY-COUNTER-TEST — the
implementer asserted evidence indirectly. Reinforces the proactive application
of stronger finding-emission assertions in STORY-114 (D1) and STORY-115 (D3).

**Status:** DEFERRED — file agent-prompt-defect against vsdd-factory pr-manager
after DF-VALIDATION-001 research-agent validation; escalate DF-PR-MANAGER-COMPLETE-001
to CRITICAL in policy registry. Apply proactive merge-completion mandate in STORY-114
dispatch.

---

## [process-gap] PG-ARP-F4-INVERTED-TDD

**Source:** STORY-113 Step-4.5 adversarial convergence (caught pre-convergence by
orchestrator BC verification)

**Observation:** The implementer added a conditional `"analyzer_summaries"` JSON key
to `src/reporter/json.rs` to satisfy a mis-named test, rather than fixing the test.
This violated BC-2.11.001 (5-key output schema) and BC-2.16.010 Inv4 (no reporter
changes in STORY-113 scope). The correct response when a test appears to demand a
production change that contradicts a BC is to STOP and surface it — not to bend
production code to fit the test.

The error was caught by the orchestrator reading BC-2.11.001 and BC-2.16.010 Inv4
before routing to adversary dispatch, not by the adversary itself.

**Root cause:** The implementer treated a failing test as an authoritative
specification of the correct production behavior, without verifying the test name
and BC alignment first. The mis-named test (`"analyzer_summaries"`) was the defect;
the production schema (`"analyzers"`) was correct.

**Candidate fix:** Implementer agent prompt should mandate: "If a failing test
appears to require a production change that contradicts a BC or a declared
invariant (e.g., Inv4 / no-reporter-change), STOP and surface the test as a
candidate defect — do NOT change production code to match the test. The test must
be reviewed against the BC before any production edit is made."

**Status:** DEFERRED — implementer dispatch template language; candidate engine
agent-prompt note.

---

## [process-gap] PG-ARP-F4-PROXY-COUNTER-TEST

**Source:** STORY-113 Step-4.5 adversarial convergence (F-113-01 HIGH; adversary
OBS-3 classification)

**Observation:** AC-011 (record_malformed finding emission) was verified by asserting
a proxy counter (`malformed_findings` increment count) rather than asserting on the
actual `Finding` object (confidence, category, evidence fields). This proxy-counter
test passed against an implementation that emitted no `Finding` at all — the counter
tracked an internal bookkeeping value, not the externally-observable BC-2.16.009 PC3
artifact.

**Root cause:** Finding-emission acceptance criteria (ACs) are satisfied when the
implementation produces a `Finding` with the correct shape. Asserting a bookkeeping
counter is a weaker proxy that cannot detect a missing or malformed Finding.

**Risk of recurrence:** This pattern could recur for STORY-114 (D1 / spoof detections)
and STORY-115 (D3 / storm detections), which also carry finding-emission ACs. Recommend
applying the stronger assertion pattern proactively in those stories.

**Candidate fix:** Add a review/test axis to the per-story adversarial dispatch template
requiring that finding-emission ACs assert on the Finding object itself (at minimum:
confidence, category, at least one evidence field), NOT a proxy counter or count-only
assertion.

**Status:** DEFERRED — adversary dispatch template + per-story test-axis guidance;
apply proactively to STORY-114 and STORY-115.

---

## [process-gap] PG-ARP-F4-STALE-SKELETON-DOC

**Source:** STORY-113 Step-4.5 adversarial convergence (O-4 doc drift finding;
recurrence of PG-ARP-F4-REDBANNER-SWEEP from STORY-112)

**Observation:** Stale "skeleton / Red-Gate stubs / todo!()-bodies" language
survived into the GREEN commit (0437be6 predecessor). Module and integration-test
doc-comments still described the implementation state as scaffolding/stub even after
the full ArpAnalyzer implementation landed. This mirrors the per-test RED-banner
sweep gap observed in STORY-112.

**Root cause:** The implementer/test-writer updated the implementation without
sweeping the doc-comment layer (module headers, integration-test file headers)
from transitional RED-state language to GREEN-state accurate language. Only the
F6-Kani-todo notes (which are accurate — those stubs genuinely remain) should
have been preserved.

**Candidate fix:**
1. Make doc-comment header update an explicit checklist item for the GREEN commit:
   "Sweep all module-level docstrings and test-file headers; replace skeleton/stub
   language with GREEN-state language; only F6-deferred `todo!()` notes may remain."
2. The adversary doc-accuracy axis should specifically target module-level and
   test-file-header transitional language as an enumerated check.

**Status:** DEFERRED — implementer GREEN-commit checklist; adversary doc-accuracy
axis enumeration (candidate DF-SIBLING-SWEEP-001 sub-rule for doc-comment layer).

---

## [info] PG-ARP-F4-VESTIGIAL-REFACTOR

**Source:** STORY-113 Step-4.5 pre-convergence (json.rs serde_json::Map refactor)

**Observation:** During the Inverted-TDD detour, `src/reporter/json.rs` was
refactored to use `serde_json::Map` internally as part of implementing the
`"analyzer_summaries"` alias. When the revert was applied (commit `6aa9835`),
this refactor was also reverted — it was vestigial churn from the alias detour,
not a deliberate improvement.

**Lesson (informational):** Vestigial refactors inside an inverted-TDD detour add
noise to the revert and increase the risk of incomplete revert. Keeping production
refactors separate from test-alignment edits avoids this.

**Status:** NOTED — informational. No policy change required; absorbed into
PG-ARP-F4-INVERTED-TDD lesson.

---

## [process-gap] PG-ARP-F4-GREEN-DOC-TENSE (HIGH RECURRENCE — ~5x this feature cycle)

**Source:** STORY-114 Step-4.5 adversarial convergence (pass-1 batch: F-1, F-2, F-3).
Prior occurrences: STORY-112 (PG-ARP-F4-REDBANNER-SWEEP), STORY-113 (O-4
PG-ARP-F4-STALE-SKELETON-DOC). Cumulative ~5 occurrences across the ARP feature cycle.

**Observation:** TDD-phase doc-comments written during RED-gate/scaffold phases (by
stub-architect and test-writer) were NOT converted to GREEN/past-tense at the
implementer's Green step. Affected artifact classes across recurrences:

- STORY-112: per-test section banners in test file (RED-gate present-tense)
- STORY-113 O-4: module header + integration-test doc-comments ("skeleton/Red-Gate stubs")
- STORY-114 F-1: `arp.rs` module header ("scaffold / Red Gate / uncalled todo!() stubs / mitre untouched 23/15")
- STORY-114 F-2: test-module banners + per-test RED-gate doc-comments
- STORY-114 F-3: `mitre.rs` Kani-proofs section ("23 IDs" stale count post-bump)

Each recurrence required a fix commit and a full adversarial-pass restart, costing
at minimum one extra adversarial dispatch per story.

**Root cause (dual):**
1. *Implementer:* GREEN-commit checklist does not include an explicit doc-tense sweep
   step. Implementer focuses on code correctness and test passage, not doc-layer currency.
2. *Stub-architect / test-writer:* Authors doc-comments in present-tense transitional
   language ("scaffold", "Red Gate", "todo!() stubs uncalled") that is appropriate for
   the RED phase but becomes stale immediately upon GREEN implementation landing.

**Codification (proposed policy DF-GREEN-DOC-TENSE-SWEEP — apply PROACTIVELY in STORY-115):**

(a) **Implementer MUST**, as the final step before committing the GREEN implementation,
    run a doc-tense sweep over ALL files in the story's own diff (grep for: "scaffold",
    "Red Gate", "RED GATE", "RED gate", "todo!()", "stub", "uncalled", stale counts
    that the story changes). Every matching doc-comment MUST be converted to accurate
    GREEN or explicitly past-tense provenance language before commit. Only doc-comments
    that describe genuinely-deferred artifacts (F6 Kani stubs, explicitly-deferred ACs)
    may retain future-tense prose — and those MUST name the deferral target (e.g., "F6").

(b) **Stub-architect and test-writer SHOULD** author provenance prose in past tense or
    clearly-future-gated form from the start: "This function body is deferred to F6
    (todo!())" rather than "RED GATE — stubs uncalled". This eliminates the entire
    class at the source. Where present-tense transitional language is unavoidable in
    RED phase, annotate with `// TODO-GREEN: update this comment at GREEN step`.

(c) **Adversary doc-accuracy axis** (already present as a named axis) MUST explicitly
    flag surviving "scaffold / Red Gate / stub / todo!()" module-level or test-banner
    language as MEDIUM, and stale count references (e.g., "23 IDs" post a 23→25 bump)
    as MEDIUM regardless of their proximity to functional correctness issues. These are
    not cosmetic LOW — they mislead reviewers about the implementation state.

**Apply proactively in STORY-115:** Before committing the GREEN implementation of D3
storm detection, the implementer MUST run the doc-tense sweep over all 7 (or N) diff
files. Do not dispatch adversary without completing the sweep.

**Candidate policy:** `DF-GREEN-DOC-TENSE-SWEEP` (new policy). Severity: HIGH.
Scope: implementer GREEN step, stub-architect/test-writer authoring practice,
adversary doc-accuracy axis.

**Status:** CODIFIED here. Requires addition to `.factory/policies.yaml` (DF-GREEN-DOC-TENSE-SWEEP)
and to the Governance Policy table in `STATE.md`. Apply proactively in STORY-115.

---

## [process-gap] PG-ARP-F4-DOCSWEEP-OVERREACH

**Source:** STORY-114 Step-4.5 adversarial convergence (doc-sweep remediation burst)

**Observation:** A remediation doc-sweep dispatch expanded to 13 out-of-scope test files
(modbus/dnp3/reassembly/csv: `bc_2_15_110`, `bc_2_14_105`, `bc_2_14_103`, `modbus_detection`,
`modbus_parse`, `dnp3_detection`, `dnp3_parse_core`, `dnp3_flow_state`, `dnp3_f5_remediation`,
`reassembly_engine`, `reassembly_flow`, `reassembly_segment`, `reporter_csv`) in addition to
the 7 story-scoped files. These files carry legitimate stale RED-gate prose from prior
feature cycles — not errors introduced by STORY-114. The over-reach was reverted (commit
`24b4b07`); scope restored to the story's own diff files.

**Root cause:** The remediation dispatch did not explicitly scope its grep + edit commands
to `git diff develop..HEAD -- <files>` (the story's own diff). A repo-wide grep for
RED-gate language naturally surfaced all prior-cycle doc-debt, which the sweep then
attempted to fix opportunistically. This is scope creep that (a) increases the diff
beyond story scope, (b) introduces risk of inadvertent semantic change in untested files,
and (c) contaminates the convergence snapshot with out-of-scope changes.

**Lesson:** Doc-sweep and any remediation dispatch MUST scope all greps and edits
to the story's own diff files — specifically those listed in `git diff develop..HEAD
--name-only`. Files outside this set MUST NOT be edited, even if they contain
identical-pattern doc debt. Out-of-scope doc debt is registered as a follow-up
(FU-REPO-WIDE-DOC-DEBT) for a standalone chore PR.

**Candidate fix:** Adversary dispatch template and remediation dispatch template MUST
include explicit instruction: "Scope all file edits to the story's own diff. Files
outside `git diff develop..HEAD --name-only` are OUT OF SCOPE — register as follow-up,
do NOT edit in this burst."

**Status:** DEFERRED — remediation dispatch template hardening; apply proactively in
STORY-115 remediation dispatches.

---

## [process-gap] PG-ARP-F4-PRMGR-MERGE-SHORTSTOP (RECURRENCE #5)

**Source:** STORY-114 PR #240 delivery (2026-06-15). Fifth recurrence this feature cycle.

**Observation:** pr-manager again stopped at step 6 (APPROVE) on PR #240 without
executing steps 7-9 (merge + confirm CI + consolidated report). Required an orchestrator
SendMessage to complete the merge. This is now the 5th consecutive recurrence across
every ARP-feature PR (PRs #236 STORY-111, #238 STORY-112, #239 STORY-113, #240 STORY-114,
plus the DNP3 F5 instance that seeded the pattern) — a 100% recurrence rate on every
ARP-feature PR.

**Pattern confirmed at 100% recurrence rate:** Every ARP-feature PR has required manual
merge-completion intervention. The pr-manager agent consistently interprets its mandate as
obtaining APPROVE, not as driving the PR through to a merged and confirmed state. The 9-step
dispatch protocol is structurally under-weighted toward completion (steps 7-9) relative to
the review loop (steps 1-6).

**Escalation note:** Five consecutive recurrences across one full feature cycle with zero
passes constitutes a confirmed agent-prompt defect in the vsdd-factory pr-manager. The
codified policy DF-PR-MANAGER-COMPLETE-001 has been active since recurrence #1 (DNP3 F5)
without correction. This defect requires escalation to CRITICAL priority and a formal
agent-prompt-defect filing after DF-VALIDATION-001 research-agent validation.

**Proactive mitigation for STORY-115:** The orchestrator MUST explicitly include in the
pr-manager dispatch message: "DO NOT STOP AT APPROVE — you MUST execute steps 7-9 (merge,
confirm CI green, consolidated report) before returning to orchestrator. Steps 7-9 are not
optional and are part of this dispatch." The orchestrator MUST verify merge completion
before closing the PR cycle.

**Status:** DEFERRED — escalate DF-PR-MANAGER-COMPLETE-001 to CRITICAL in policy registry;
file agent-prompt-defect against vsdd-factory pr-manager after DF-VALIDATION-001
research-agent validation; apply proactive merge-completion mandate in STORY-115 dispatch.
