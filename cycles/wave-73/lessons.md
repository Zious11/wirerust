# Lessons Learned — wave-73

S-7.02 cycle-closing requirement: lessons recorded here for human review and
engine-improvement triage.

Wave: 73 | Gate passed: 2026-07-11 (D-428) | Wave CLOSED: 2026-07-11 (D-428, human-approved) | Stories: STORY-162, STORY-163 (5 pts).
PRs merged: #395 (b5e1e15, STORY-162). STORY-163 factory-artifacts-only (D-427, no develop PR).
Adversarial convergence: 6 passes; streak 3/3 (P4/P5/P6); trajectory 4→2→1→0→1(nits-refuted)→0.

---

## Lesson 1 — [codified] STORY-164 AC-164-001 — Corpus-Wide Status-Drift Is a Wave-Level Gate Class

**Observation:**

Wave-73 adversary Pass 3 (F-W73G-P3-001 HIGH) found that STORY-158 and STORY-159
had stale status fields at multiple loci (frontmatter, body, index). Investigating
further, the same class of status-vocabulary inconsistency was present across 38
factory story files — stories with `status: completed` in one location and `status:
delivered` or `status: merged` in another, or stories with body-text status labels
that did not match the STORY-INDEX catalog column.

The root cause was dual: (1) no canonicalized status-vocabulary legend existed in
STORY-INDEX to enforce a single authoritative set of values; (2) story amendments
(version bumps, AC updates) updated one locus but not all. The 38-file sweep was
executed at P3; all files were corrected.

**Lesson:**

A STORY-INDEX status-vocabulary legend (a machine-checkable mapping of allowed
status values and their semantics) prevents ambiguous or inconsistent status labeling
from accumulating silently across cycles. Without such a legend, each story amendment
risks introducing a new informal variant that the next adversary pass must discover
and classify.

Codification: STORY-164 AC-164-001 adds the STORY-INDEX status-vocabulary legend
(PG-W73-STATUS-VOCAB). Delivery of STORY-164 is required to fully close this gap.

**Tags:** `codified`, `corpus-sweep`, `status-vocabulary`, `wave-gate-value`

---

## Lesson 2 — [codified] STORY-164 AC-164-002 — Citation-Mandate Stories Must Self-Validate

**Observation:**

STORY-163 (the citation-mandate story, AC-163-001 docs-dispatch citation mandate)
was delivered with fabricated anchor references in its own authoring-evidence.md.
The anchor table cited non-existent line numbers in pr-manager-merge-auth-guidance.md.
This was caught at P1 of the STORY-163 per-story adversary as F-S163P1-001 CRITICAL.

This is a meta-irony: the story that mandates citation accuracy had inaccurate
citations in its own evidence. The root cause is that no mechanical preflight tool
existed to validate that cited line numbers resolve to actual content in the
referenced files. The adversary acted as the validation mechanism in this instance,
but the fabrication could have persisted if the adversary had not been dispatched.

**Lesson:**

Any story that codifies or mandates citation practices MUST be verified by a
mechanical preflight tool before delivery, not only by adversarial review. A
`bin/validate-citations` script that resolves every anchor reference (file:line)
in authoring-evidence and implementation documents to an actual match in the
referenced file would catch fabricated anchors before adversary dispatch.

The broader principle: stories that codify process constraints should themselves
satisfy stronger validation of those constraints than ordinary stories — the mandate
must self-validate.

Codification: STORY-164 AC-164-002 adds `bin/validate-citations` preflight validator
(PG-W73-CITATION-VALIDATOR). Delivery of STORY-164 is required to fully close this gap.

**Tags:** `codified`, `citation-validation`, `meta-irony`, `self-referential-mandate`

---

## Lesson 3 — [codified] STORY-164 AC-164-003 — Changelog-Gate Is Presence-Only (Content Not Asserted)

**Observation:**

During STORY-162 delivery, adversary Pass 5 (P5 process-gap observation) noted that
the `changelog-gate` CI job (introduced by STORY-158 AC-158-001) checks only that
an `[Unreleased]` CHANGELOG entry EXISTS — it does not assert anything about the
content of that entry. A story could satisfy the gate with a placeholder entry
(e.g., `[Unreleased]\n- placeholder`) that provides no useful release-note information.

The root cause: AC-158-001's gate was designed as a presence gate (catching the
class of "CHANGELOG entirely absent" failures that triggered PG-W71-CHANGELOG),
not a content-quality gate. Content accuracy was left to reviewer judgment in the
PR review. While this is a reasonable first cut, it leaves a gap: a low-effort
CHANGELOG entry passes CI even if it has no behavioral content.

**Lesson:**

A changelog-gate content assertion (e.g., minimum character count on the entry
body, or a check that the entry contains at least one change description line)
would close the gap between a presence gate and a content-quality gate. This need
not be expensive — a simple line-count or grep for non-heading, non-empty lines
under `[Unreleased]` would prevent placeholder entries from passing silently.

Codification: STORY-164 AC-164-003 adds a changelog-gate content assertion
(PG-W73-CHANGELOG-GATE-CONTENT). Delivery of STORY-164 is required to fully close this gap.

**Tags:** `codified`, `changelog-gate`, `content-assertion`, `process-gap`

---

## Lesson 4 — [observation] Negative-Evidence Claims Require Second-Method Verification

**Observation:**

Wave-73 adversary Pass 5 filed F-W73G-P5-001 claiming that ≤2 demo artifacts
existed on develop, suggesting the STORY-162 demo evidence was insufficient. The
finding was refuted by orchestrator ground truth: 5 demo artifacts were confirmed
present and the scrub gate had already passed at STORY-162 delivery (D-426).

Root cause: the adversary's scan relied on a glob that underscoped the artifact
locations. The adversary reported a negative-evidence claim ("X does not exist")
without verifying by a second independent method.

**Lesson:**

Negative-evidence claims — "file X is absent", "artifact Y does not exist", "count
is less than N" — carry a higher false-positive risk than positive findings (which
assert something present). Before filing a negative-evidence claim as a finding, the
adversary (or any reviewing agent) should verify by at least two independent methods:
(1) the primary scan/glob, and (2) a direct targeted lookup (e.g., `ls` of the
specific path, `find` with explicit path, or a prior delivery record in STATE.md).

This lesson has no codification story; it is recorded here as an explicit
process-improvement observation for future adversary dispatch guidance.

**Tags:** `observation`, `negative-evidence`, `false-finding`, `verification-method`

---

## Lesson 5 — [applied] Instructed-Halt vs. Classifier-Denial Path Used Cleanly

**Observation:**

Wave-73 plan gate (D-425) involved merging STORY-162 PR #395 to develop. The
PG-MERGE-AUTH-SUBAGENT-CLASSIFIER interim path (codified in STORY-163 AC-163-002)
specified that develop-PR merges must be executed by the orchestrator in the main
thread under direct human authorization, not by a subagent.

In practice, this distinction was navigated cleanly: the orchestrator executed the
merge in the main thread, and the human provided direct authorization. No
subagent-classifier denial was triggered, no relay failure occurred, and no
escalation was needed.

**Lesson:**

The instructed-halt path (orchestrator acts in main thread under direct human auth)
is operationally distinct from the classifier-denial path (subagent requests action
that requires elevated permissions and is denied). The D-425 wave-73 plan gate
demonstrated the instructed-halt path works cleanly when correctly followed: the
orchestrator does not attempt to delegate the merge to a subagent, and the human
provides the authorization directly in the conversation.

Codification: AC-163-002 in STORY-163 already codifies this path. The lesson here
is that the path worked as designed with zero friction — confirming the codification
is correct and the protocol is viable at scale.

**Tags:** `applied`, `merge-auth`, `classifier-denial`, `instructed-halt`, `protocol-validated`

---

## Gate-Close Confirmation — Wave-73 CLOSED (D-428, 2026-07-11)

Wave-73 officially CLOSED by human approval (D-428, 2026-07-11). All gate dimensions
PASS per gate-summary.md.

**Codification status summary (S-7.02):**

| Lesson | Tag | Codification vehicle |
|--------|-----|---------------------|
| Lesson 1 — Corpus-Wide Status-Drift | `[codified] STORY-164 AC-164-001` | STORY-164 (wave-TBD) adds STORY-INDEX status-vocabulary legend (PG-W73-STATUS-VOCAB) |
| Lesson 2 — Citation-Mandate Self-Validation | `[codified] STORY-164 AC-164-002` | STORY-164 (wave-TBD) adds bin/validate-citations preflight (PG-W73-CITATION-VALIDATOR) |
| Lesson 3 — Changelog-Gate Content | `[codified] STORY-164 AC-164-003` | STORY-164 (wave-TBD) adds changelog-gate content assertion (PG-W73-CHANGELOG-GATE-CONTENT) |
| Lesson 4 — Negative-Evidence Second Method | `[observation]` | No dedicated story; recorded for adversary dispatch guidance |
| Lesson 5 — Instructed-Halt Path Validated | `[applied]` | AC-163-002 (STORY-163, already delivered) codifies the path |

wave-72 Lesson-2 (BREAKING holdout sweep / PROP-V0.12.0-01) remains `[candidate-codification — next maintenance]`; not addressed in wave-73 scope.
