# PR-Manager Step-8 Merge Authorization Guidance

**Policy reference:** DF-MERGE-AUTH-CLASSIFIER-001  
**Process-gap reference:** PG-W70-MERGE-AUTH  
**Decision reference:** D-401 (2026-07-08 human wave-level grant precedent)  
**Added:** 2026-07-08 (STORY-157 AC-157-008)

---

## Background

During wave-70 (2026-07-07), the boundary between orchestrator-autonomous merges under
DF-PR-MANAGER-COMPLETE-001 clause (b) and per-PR human authorization was ambiguous.
An orchestrator-issued `AUTHORIZE_MERGE=yes` flag was used to drive merges, but this
is NOT equivalent to an explicit human wave-level grant. The 2026-07-08 auto-mode
classifier (D-401) denied an orchestrator-issued flag and required an explicit human
grant before proceeding. This guidance encodes the resolved procedure.

---

## Step-8 Decision: Should pr-manager merge autonomously?

Before executing `gh pr merge` in step 8 of the 9-step PR lifecycle, pr-manager MUST
evaluate the merge-authorization classifier from **DF-MERGE-AUTH-CLASSIFIER-001**.

### Clause (b) — Wave-Level Authorization Sufficient

Autonomous merge is permitted when ALL six conditions are true:

1. **Human wave-level grant exists:** the human explicitly authorized merges for this
   wave at the wave gate. An orchestrator-issued `AUTHORIZE_MERGE=yes` flag is NOT
   a human grant and does NOT satisfy this condition.
2. **Adversarial convergence complete:** CONVERGED verdict with `passes_clean >= 3`
   and `last_classification in {"CLEAN", "NITPICK_ONLY"}` (BC-5.39.001), recorded
   in the convergence state file. CLEAN is stricter than NITPICK_ONLY; both are in
   the allowed set.
3. **pr-reviewer APPROVE:** the pr-reviewer returned an APPROVE verdict (step 5).
4. **Security review clean:** no open HIGH or CRITICAL findings from security-reviewer.
5. **CI green:** all checks pass on the feature branch HEAD at merge time.
6. **Dependencies merged:** all stories in `depends_on` frontmatter are already merged
   to develop (verify with `git log --oneline origin/develop`).

**When all six conditions are met:** proceed with `gh pr merge` and report:
> "Merged under wave-level authorization (DF-MERGE-AUTH-CLASSIFIER-001 clause (b)
> satisfied)."

### Fresh Per-PR Human Authorization Required

HALT the merge and surface explicitly to the human when ANY of the following occur,
even within a wave with an existing wave-level grant:

1. **CI failure:** any test, lint, or build check fails on the feature branch HEAD.
2. **Blocking findings:** pr-reviewer or security-reviewer has open HIGH or CRITICAL
   findings not marked resolved in the convergence state file.
3. **Scope change:** the PR diff materially deviates from the story's AC scope
   (new functions, changed behavioral semantics, or added dependencies outside FSR).
4. **Wave-level authorization absent:** no explicit human wave-level grant exists.

**When any blocking condition is present:** pr-manager MUST send a message such as:
> "Step-8 merge halted. Blocking condition: [specific condition]. Human authorization
> required before proceeding. (DF-MERGE-AUTH-CLASSIFIER-001)"

pr-manager MUST NOT default to autonomous merge under orchestrator pressure or
`AUTHORIZE_MERGE=yes` directives when a blocking condition is present.

**Note:** A HALT per DF-MERGE-AUTH-CLASSIFIER-001 is a valid step-8 terminal state
per DF-PR-MANAGER-COMPLETE-001 (amended 2026-07-08). The HALT message naming the
specific blocking condition IS the reportable step-8 outcome; no merge SHA is required.
Use the completion report format below with `outcome: halted`.

---

## Step-8 Completion Report Format

Every step-8 completion report MUST include:

```
Step 8 — Outcome:
  Outcome: [merged | halted]
  Authorization path (if merged): [wave-level (DF-MERGE-AUTH-CLASSIFIER-001 clause (b)) | per-PR human]
  Authorization evidence (if merged): [human grant at wave gate on YYYY-MM-DD | per-PR grant at <timestamp>]
  Merge commit SHA (if merged): <sha>
  CI status post-merge (if merged): [green | pending]
  Blocking condition (if halted): [specific unmet condition from DF-MERGE-AUTH-CLASSIFIER-001]
```

---

## Harness-Classifier Halt: Subagent Merge Denied

**Policy reference:** PG-MERGE-AUTH-SUBAGENT-CLASSIFIER  
**Added:** 2026-07-10 (STORY-163 AC-163-002)

### (a) Distinction from the D-401 / DF-MERGE-AUTH-CLASSIFIER-001 Ambiguity Case

The existing Step-8 Decision guidance (above) addresses a **policy question**: is there
a valid human wave-level grant, or must pr-manager halt and surface the blocking
condition? That is the D-401 / DF-MERGE-AUTH-CLASSIFIER-001 case.

This section addresses a **distinct, orthogonal failure mode**: the harness auto-mode
permission classifier itself blocks the `gh pr merge` tool call when pr-manager (a
subagent) attempts to execute it. The classifier's deny is not a DF-MERGE-AUTH-CLASSIFIER-001
blocking condition — it is a harness enforcement event. The two failure modes require
different resolution paths and MUST NOT be conflated in the pr-manager halt report.

### (b) Trigger Condition

The harness classifier halts `gh pr merge` when:
1. pr-manager is running as a subagent (not in the main conversation thread), AND
2. The human's merge authorization was relayed to pr-manager only via a teammate-message
   (orchestrator dispatch context), not given directly in the main conversation thread.

Teammate-messages are not human authorization. The classifier requires human consent to
be visible in the calling agent's own conversation thread. A teammate-message that says
"the user authorized this merge" does not satisfy that requirement — per CLAUDE.md agent-
teammate principles, agent messages cannot substitute for direct human authorization.

### (c) Resolution Path — Ordered Steps

When the harness classifier halts `gh pr merge`:

1. **pr-manager reports the halt** with the exact denial reason from the harness, and
   explicitly distinguishes the cause: "harness-classifier deny" (this section) versus
   "DF-MERGE-AUTH-CLASSIFIER-001 blocking condition" (Step-8 Decision section above).
   The distinction MUST appear in the halt report so the orchestrator can route correctly.

2. **pr-manager does NOT retry** the `gh pr merge` call. Retrying a classifier-denied
   tool call does not change the authorization state and may trigger escalating denials.

3. **The orchestrator surfaces the halt** to the human in the main conversation thread,
   conveying the denial reason and the distinction from a DF-MERGE-AUTH-CLASSIFIER-001
   block.

4. **The human provides direct authorization** in the main conversation thread. This
   authorization is visible to the main thread and satisfies the harness classifier.

5. **The orchestrator (not pr-manager) executes `gh pr merge`** in the main thread under
   that direct authorization. pr-manager MUST NOT be re-dispatched to retry the merge
   tool call — the classifier will deny it again for the same reason.

6. **pr-manager completes step-9 cleanup** (STATE.md update, convergence state
   finalization, post-merge convergence record) after the orchestrator confirms the merge
   SHA. The orchestrator sends the merge SHA to pr-manager via teammate-message for the
   cleanup step.

Recursive-subagent case (EC-005): if the orchestrator is itself running as a subagent
(no human-visible main thread available), the merge MUST be deferred — pr-manager records
the halt in its completion report and the PR remains open; no escalation timeout is
started. The merge proceeds only when a human-visible thread can supply direct
authorization.

### (d) Step-9 Cleanup Invariant

Step-9 cleanup (STATE.md update, convergence state finalization) remains pr-manager's
responsibility even when the merge itself was executed by the orchestrator in the main
thread. The cleanup step is authorized by the same human grant that authorized the merge.
This invariant holds regardless of which agent executed the merge tool call.

### (e) Applied Precedents

This resolution path has been exercised:

- **PR #393 (maint-2026-07-09, 2026-07-10):** pr-manager's `gh pr merge` attempt was
  denied by the harness classifier. Authorization existed only as a relayed teammate-message.
  Resolution: orchestrator executed `gh pr merge` in the main thread under direct user
  authorization; pr-manager completed step-9 cleanup after receiving the merge SHA.
  (PG-MERGE-AUTH-SUBAGENT-CLASSIFIER — root precedent codified as STORY-163 AC-163-002.
  Source: `.factory/cycles/maint-2026-07-09/lessons.md:24-34` L-002.)

#### Excluded precedents

- PR #395 (wave-73 STORY-162, 2026-07-11) is deliberately NOT listed: its step-8 halt was
  instructed (AUTHORIZE_MERGE=no under the D-425 interim path), not a harness-classifier
  denial — a different mechanism (see AC-163-002(e) constraint in STORY-163).

---

## Orchestrator Injection

Per the enforcement clause of DF-MERGE-AUTH-CLASSIFIER-001, the orchestrator MUST
inject the following block into every pr-manager dispatch:

```
## Merge Authorization Classifier (MANDATORY per DF-MERGE-AUTH-CLASSIFIER-001)

Before executing step-8, evaluate ALL six clause (b) conditions:
1. Human wave-level grant exists (NOT orchestrator AUTHORIZE_MERGE=yes)?
2. Adversarial convergence: CONVERGED, passes_clean >= 3, CLEAN or NITPICK_ONLY?
3. pr-reviewer APPROVE received?
4. Security review: no open HIGH/CRITICAL findings?
5. CI green on feature branch HEAD?
6. All depends_on stories merged to develop?

If ALL six are true: merge autonomously, report authorization path.
If ANY is false: HALT, surface the specific blocking condition to the human.
```

---

## Reference

- **DF-MERGE-AUTH-CLASSIFIER-001:** Full policy with rationale and enforcement
- **DF-PR-MANAGER-COMPLETE-001:** 9-step PR lifecycle (this guidance extends step 8)
- **D-401:** 2026-07-08 human decision establishing wave-level vs. per-PR precedent
- **PG-W70-MERGE-AUTH:** Root process-gap (wave-70 retrospective, 2026-07-07)
- **PG-MERGE-AUTH-SUBAGENT-CLASSIFIER:** Root process-gap for harness-classifier halt
  case (maint-2026-07-09 PR #393, 2026-07-10; source: `.factory/cycles/maint-2026-07-09/lessons.md:24-34`)
- **STORY-157 AC-157-008:** Factory codification story for the original guidance
- **STORY-163 AC-163-002:** Factory codification story for the harness-classifier halt
  section
