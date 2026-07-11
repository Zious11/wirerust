---
document_type: story
story_id: STORY-163
epic_id: E-11
version: "1.2"
status: ready
producer: story-writer
timestamp: 2026-07-10T00:30:00Z
phase: f7
level: feature
cycle: wave-73
points: 2
priority: P3
depends_on: []
blocks: []
# BC status: pending PO authorship
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: .factory/maintenance/
subsystems: []
estimated_days: 1
wave: "73"
traces_to:
  - .factory/code-delivery/maint-2026-07-09/pr-review.md
  - .factory/maintenance/sweep-report-2026-07-09.md
  - .factory/maintenance/pr-manager-merge-auth-guidance.md
input-hash: "e1ad659"
inputs:
  - .factory/code-delivery/maint-2026-07-09/pr-review.md
  - .factory/maintenance/sweep-report-2026-07-09.md
  - .factory/maintenance/pr-manager-merge-auth-guidance.md
---

# STORY-163: maint-2026-07-09 cycle-closing: docs-dispatch citation mandate + subagent merge-auth resolution path

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** ready
**Wave:** 73
**Points:** 2
**Priority:** P3

## Narrative

- **As a** spec-steward, orchestrator operator, and future docs-remediation dispatcher on
  the wirerust project
- **I want** two maint-2026-07-09 process gaps codified into durable project artifacts: a
  ground-truth citation mandate added to the docs-remediation dispatch guidance, and a
  subagent-merge-halted resolution path appended to the pr-manager merge-authorization
  guidance
- **So that** future docs-remediation dispatches cannot produce factually inverted
  user-facing claims from one-line finding summaries, and so that when the harness
  classifier halts a subagent `gh pr merge`, the orchestrator and pr-manager have a
  documented, unambiguous resolution path that does not require human improvisation

## Behavioral Contracts

_(none -- E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

maint-2026-07-09 Route A adversarial convergence and the PR #393 merge step surfaced two
process gaps. S-7.02 (cycle-close requirement) mandates codification of recurring process
gaps as follow-up stories.

### PG-RA-P3-ARP-REC006-INVERSION-001 -- docs-dispatch without ground-truth citation mandate

During the maint-2026-07-09 Route A adversarial convergence, adversary Pass 3
(finding F-RA-P3-001, severity equivalent to a blocking factual inversion) exposed that
the routeA-docs-writer dispatch had paraphrased the one-line sweep recommendation
REC-006 ("README § Known Limitations: add one sentence on MACsec/VLAN ARP offset
detection limitation") and produced a README claim that was factually **inverted**: the
draft text stated that VLAN/QinQ/MACsec-tagged ARP frames produce no findings, when
`src/decoder.rs` provably handles those frames by applying the correct VLAN-header offset
via `LaxLinkExtSlice::header_len()` (D-078/D-078b, `BC-2.16.009`/`BC-2.16.015`) and
does produce findings for them.

Root cause: the dispatch task contained only the one-line REC-006 summary from the
sweep report. The docs-writer was not required to cite a ground-truth file:line anchor
for each behavioral claim before producing the output, and was not told which source
files to Read for verification. The inversion was caught by adversary Pass 3 and fixed
before merge; however, the dispatch pattern remains a latent hazard for any future
docs-remediation task that touches behavioral claims.

Evidence:
- `F-RA-P3-001` (maint-2026-07-09 Route A adversary Pass 3, inverted VLAN/QinQ/MACsec
  claim): `.factory/code-delivery/maint-2026-07-09/pr-review.md` §Adversarial
  Convergence Evidence Reviewed (lines confirming F-RA-P3-001 as a prior finding
  resolved before merge).
- REC-006 original recommendation: `.factory/maintenance/sweep-report-2026-07-09.md`
  §Risk-Assumption Monitoring carry-forward items (REC-006 "README § Known Limitations:
  add one sentence on MACsec/VLAN ARP offset detection limitation") and §Route A table.
- Ground-truth location: `src/decoder.rs` D-078/D-078b comments (lines 22, 157, 266,
  291-313) confirming VLAN/QinQ/MACsec ARP frames are handled by the lax path.

### PG-MERGE-AUTH-SUBAGENT-CLASSIFIER -- subagent merge halted; resolution path undocumented

During the PR #393 merge step (2026-07-10, maint-2026-07-09 Route A delivery), the
harness auto-mode permission classifier denied `gh pr merge` when executed by pr-manager
(a subagent). The classifier's denial was correct: human consent for the merge was
present only as a relayed message in the subagent's teammate-message context, not as a
visible authorization in the main conversation thread. This is a new failure mode not
covered by the existing `pr-manager-merge-auth-guidance.md` (DF-MERGE-AUTH-CLASSIFIER-001
companion), which addresses the question of whether pr-manager should attempt a merge,
but does not specify what happens when the harness itself denies the attempt.

pr-manager correctly refused to retry after the denial and reported the halt with a
diagnosis. The resolution was: the orchestrator executed `gh pr merge` in the main
conversation thread under direct user authorization given in that thread; pr-manager
then completed step-9 cleanup after merge confirmation.

Root cause: the existing guidance (DF-MERGE-AUTH-CLASSIFIER-001, STORY-157
AC-157-008) covers the `AUTHORIZE_MERGE=yes` ambiguity (human vs. orchestrator grant)
but does not cover the distinct case where the harness permission system itself blocks
the merge tool call because authorization is not visible in the calling agent's
conversation thread. These are two different failure modes: the first is a policy
question (should we merge?), the second is a harness enforcement question (can this
agent execute the merge call?).

Evidence:
- DF-MERGE-AUTH-CLASSIFIER-001 companion doc (the file to update):
  `.factory/maintenance/pr-manager-merge-auth-guidance.md`.
- Resolution performed 2026-07-10: orchestrator executed `gh pr merge` in the main
  thread under direct user authorization; pr-manager completed step-9 cleanup.

## Acceptance Criteria

### AC-163-001 (traces to PG-RA-P3-ARP-REC006-INVERSION-001 -- docs-dispatch citation mandate)

A new guidance document `.factory/maintenance/docs-writer-dispatch-guidance.md` is
created codifying the ground-truth citation mandate for docs-remediation dispatches.
The document MUST:

(a) Define the scope: applies to any dispatch of a technical-writer or docs-writer agent
    (or equivalent instruction to any agent) to produce or modify user-facing
    documentation from finding summaries, sweep recommendations, or other compressed
    descriptions of behavioral properties.

(b) State the **ground-truth citation mandate**: every behavioral claim in the produced
    text MUST be traceable to a specific `file:line` or spec anchor (ADR section,
    BC-S.SS.NNN, VP-NNN property statement, or equivalent) that the writer Read during
    the task. The dispatch MUST name the expected ground-truth source files explicitly
    (e.g., `src/decoder.rs`, the relevant ADR, or the BC file) so the writer can Read
    them before drafting any behavioral claim.

(c) State the **inversion-prevention rule**: one-line finding summaries from sweep
    reports or recommendation lists are sufficient to identify what to document, but are
    NOT sufficient as sole inputs for drafting behavioral claims. The writer must verify
    the actual code or spec behavior against the file:line anchor before writing.

(d) Include a **verification template**: a short block the orchestrator MUST include in
    every docs-remediation dispatch, requiring the writer to list each behavioral claim
    alongside its file:line anchor before submitting the draft.

(e) Include a **concrete application example** using REC-006 / F-RA-P3-001 as the
    illustrating case: the one-line summary said "MACsec ARP limitation"; the correct
    ground-truth check was `src/decoder.rs` D-078/D-078b comments confirming the lax
    path handles VLAN/QinQ/MACsec-tagged ARP frames and produces findings for them; the
    correct README claim therefore describes the limitation boundary accurately rather
    than inverting it.

(f) Reference: `PG-RA-P3-ARP-REC006-INVERSION-001`, finding `F-RA-P3-001`
    (maint-2026-07-09 Route A adversary Pass 3), and this story (STORY-163).

Verification:
```bash
test -f .factory/maintenance/docs-writer-dispatch-guidance.md
grep -n "ground-truth\|citation mandate\|file:line\|inversion" \
  .factory/maintenance/docs-writer-dispatch-guidance.md
```
must emit non-empty output containing the mandate text and REC-006/F-RA-P3-001 example.

### AC-163-002 (traces to PG-MERGE-AUTH-SUBAGENT-CLASSIFIER -- pr-manager guidance update)

`.factory/maintenance/pr-manager-merge-auth-guidance.md` (DF-MERGE-AUTH-CLASSIFIER-001
companion) gains a new section titled **"Harness-Classifier Halt: Subagent Merge Denied"**
that documents the resolution path when the harness permission system itself blocks the
`gh pr merge` tool call. The new section MUST:

(a) Distinguish this failure mode from the existing `AUTHORIZE_MERGE=yes` ambiguity case
    (D-401): the existing case is a policy question (is there a valid human grant?); the
    new case is a harness enforcement question (the harness denied the tool call because
    human consent was not visible in the subagent's conversation thread).

(b) State the trigger condition: the harness auto-mode classifier halts `gh pr merge`
    when executed by a subagent (pr-manager) and the human authorization was relayed
    only via teammate-message, not given directly in the main conversation thread.

(c) State the resolution path in ordered steps:
    1. pr-manager reports the halt with the exact denial reason and a diagnosis
       distinguishing "harness-classifier deny" from "DF-MERGE-AUTH-CLASSIFIER-001
       blocking condition" (they are different).
    2. pr-manager does NOT retry the merge call.
    3. The orchestrator surfaces the halt to the human in the main conversation thread.
    4. The human provides direct authorization in the main conversation thread.
    5. The orchestrator (not pr-manager) executes `gh pr merge` in the main thread
       under that direct authorization.
    6. pr-manager completes step-9 cleanup (post-merge state update, convergence record)
       after the orchestrator confirms the merge SHA.

(d) State the invariant: step-9 cleanup (STATE.md update, convergence state finalization)
    remains pr-manager's responsibility even when the merge itself was executed by the
    orchestrator. The cleanup step is authorized by the same human grant that authorized
    the merge.

(e) Cite: `PG-MERGE-AUTH-SUBAGENT-CLASSIFIER` (maint-2026-07-09, 2026-07-10 PR #393
    merge), `D-401` (prior wave-70 precedent), `DF-MERGE-AUTH-CLASSIFIER-001`, and
    this story (STORY-163).

Verification:
```bash
grep -n "Harness-Classifier\|subagent.*denied\|SUBAGENT\|PG-MERGE-AUTH-SUBAGENT" \
  .factory/maintenance/pr-manager-merge-auth-guidance.md
```
must emit non-empty output containing the new section heading and STORY-163 citation.

## Architecture Mapping

| Component | File | Pure/Effectful |
|-----------|------|---------------|
| Docs-dispatch citation mandate | `.factory/maintenance/docs-writer-dispatch-guidance.md` (new) | Documentation |
| Subagent merge-halt resolution path | `.factory/maintenance/pr-manager-merge-auth-guidance.md` (amend) | Documentation |

No Rust source files, no tests in `tests/`, no CI configuration, no Cargo.toml.

## Purity Classification

| File | Classification | Reason |
|------|---------------|--------|
| `docs-writer-dispatch-guidance.md` | Documentation artifact | Governance prose; no code |
| `pr-manager-merge-auth-guidance.md` | Documentation artifact | Governance prose; no code |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Docs-dispatch where the finding summary is accurate and no behavioral claim is made | Citation mandate still applies: writer must confirm no behavioral claim exists before omitting citations; if even one behavioral claim is present, cite it |
| EC-002 | Dispatch to a writer for changelog or release-notes (not behavioral-claim docs) | Guidance scope is "user-facing documentation from behavioral properties"; pure change-log enumeration without behavioral claims is out of scope unless behavioral claims appear |
| EC-003 | Harness classifier halts a merge outside of the docs-sweep context (any story PR) | AC-163-002 guidance applies generally; the section is not scoped to maint sweeps only |
| EC-004 | pr-manager receives a teammate-message that looks like human authorization | Teammate-messages are not human authorization (consistent with CLAUDE.md agent-teammate section); the guidance must make this explicit |
| EC-005 | Orchestrator is itself a subagent and cannot execute the merge in a "main thread" | The merge must be deferred until a human-visible thread can supply the authorization; pr-manager reports the block without escalation timeout |

## Tasks

1. **Create docs-writer-dispatch-guidance.md (AC-163-001):** Write
   `.factory/maintenance/docs-writer-dispatch-guidance.md` from scratch, covering all
   five required elements (scope, mandate, inversion-prevention rule, verification
   template, and REC-006/F-RA-P3-001 example). Use the same header style as
   `pr-manager-merge-auth-guidance.md` (policy reference block, background section,
   rule sections, reference block at end). Verify the REC-006 example accuracy against
   `src/decoder.rs` D-078/D-078b comments before writing.

2. **Amend pr-manager-merge-auth-guidance.md (AC-163-002):** Append the new
   "Harness-Classifier Halt: Subagent Merge Denied" section to the existing guidance
   document. Place it after the existing "Step-8 Decision" section and before
   "Orchestrator Injection." Update the Reference block at the end to include
   `PG-MERGE-AUTH-SUBAGENT-CLASSIFIER` and `STORY-163`.

3. **Verify no product code changes.** Confirm zero changes to `src/`, `tests/`,
   `.github/`, or `Cargo.toml`. The diff must touch only
   `.factory/maintenance/docs-writer-dispatch-guidance.md` (new) and
   `.factory/maintenance/pr-manager-merge-auth-guidance.md` (amended). Both live on
   the `factory-artifacts` branch.

> **Note for implementer:** Both target files live in `.factory/maintenance/` on the
> `factory-artifacts` branch. Neither appears in a `develop`-targeted PR diff. Commit
> both files to `factory-artifacts` in the same delivery burst. No develop PR is required
> unless CLAUDE.md needs a Project References row (not required for this story).

## Previous Story Intelligence

Lessons from analogous governance/tooling stories in E-11:

- **STORY-157 (wave-71, E-11, 3 pts):** Created `pr-manager-merge-auth-guidance.md`
  (AC-157-008) to codify the wave-70 merge-auth process gap. STORY-163 AC-163-002 extends
  that same document with the new harness-classifier halt case. Follow the same file
  structure (policy reference header, background, rule sections, reference block).
- **STORY-162 (wave-TBD, E-11, 3 pts):** Wave-72 process-gap codifications
  (LMR-003 + check-green-doc-tense tests). STORY-163 follows the same S-7.02 pattern
  but targets factory-artifacts-only artifacts (no develop PR needed for either AC).
- **STORY-158 (wave-72, E-11, 3 pts):** Governance docs that avoid a flaky CI stub.
  Both STORY-163 ACs are documentation-only; no CI gate added (consistent with no-flaky-
  stub policy).

## Architecture Compliance Rules

- This story modifies/creates ONLY: `.factory/maintenance/docs-writer-dispatch-guidance.md`
  (new) and `.factory/maintenance/pr-manager-merge-auth-guidance.md` (amended). Both are
  factory-artifacts branch files.
- No production Rust, no CI YAML, no CLAUDE.md, no Cargo.toml, no story files other than
  STORY-163 itself.
- The REC-006/F-RA-P3-001 example in AC-163-001(e) MUST accurately describe the actual
  code behavior at `src/decoder.rs` D-078/D-078b — read the file before writing the
  example; do not paraphrase from the sweep report summary (that is precisely the failure
  this story codifies against).

## Library & Framework Requirements

- No code dependencies. Pure documentation artifacts.
- No new library versions required. No Rust toolchain changes.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `.factory/maintenance/docs-writer-dispatch-guidance.md` | Create | Ground-truth citation mandate; factory-artifacts branch |
| `.factory/maintenance/pr-manager-merge-auth-guidance.md` | Modify | Add subagent merge-halt section; factory-artifacts branch |

## Token Budget Estimate

| Component | Estimated tokens |
|-----------|-----------------|
| Story spec (this file) | ~2.5 k |
| `pr-manager-merge-auth-guidance.md` (existing, to understand structure) | ~0.5 k |
| `src/decoder.rs` D-078/D-078b relevant lines for example accuracy check | ~0.3 k |
| New guidance doc draft (docs-writer-dispatch-guidance.md) | ~0.8 k |
| **Total** | **~4.1 k** |

Well within context window. No story split required.

## Notes

- **DF-VALIDATION-001 gate:** Both process gaps originate from maint-2026-07-09
  in-process execution findings: F-RA-P3-001 from the Route A adversarial convergence
  Pass 3 (in-process finding caught before merge) and PG-MERGE-AUTH-SUBAGENT-CLASSIFIER
  from the PR #393 merge step (in-process harness behavior, 2026-07-10). Both are
  in-process execution findings — DF-VALIDATION-001-exempt per the in-process exemption
  (same pattern as STORY-162 Notes, STORY-159 Notes, STORY-158 Notes).
- **S-7.02 disposition:** Creating this story at draft status codifies two
  maint-2026-07-09 process-gap findings (F-RA-P3-001 / PG-RA-P3-ARP-REC006-INVERSION-001,
  and PG-MERGE-AUTH-SUBAGENT-CLASSIFIER) for the S-7.02 maint-2026-07-09 cycle-close
  obligation.
- **No behavioral contract required:** E-11 convention (epics.md E-11: "BCs: none
  authored yet -- status: draft; pending PO authorship").
- **Two distinct merge-auth failure modes.** This story is careful to distinguish the
  D-401 case (orchestrator `AUTHORIZE_MERGE=yes` is not a human grant) from the new
  PG-MERGE-AUTH-SUBAGENT-CLASSIFIER case (harness classifier denies the tool call
  itself). The existing guidance already covers D-401; AC-163-002 only adds the new
  harness-classifier halt case.
- **Precedent:** STORY-163 follows the same E-11 pattern: cycle process-gap follow-up
  encoding lessons into project governance and tooling (STORY-157 → wave-70; STORY-158
  → wave-71; STORY-162 → wave-72; STORY-163 → maint-2026-07-09).

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-07-10 | story-writer | Status draft→ready — wave-73 plan gate approved (D-425). |
| 1.1 | 2026-07-10 | story-writer | Input-hash refresh after D-420 run-close updates to input files; citations re-validated, no semantic drift. Wave and cycle assigned: wave-73 (wave-73 opening). |
| 1.0 | 2026-07-10 | story-writer | Initial authorship -- maint-2026-07-09 process-gap codifications: PG-RA-P3-ARP-REC006-INVERSION-001 (F-RA-P3-001, docs-dispatch citation mandate) + PG-MERGE-AUTH-SUBAGENT-CLASSIFIER (PR #393, subagent merge-halt resolution path). S-7.02 maint-2026-07-09 cycle-close. |
