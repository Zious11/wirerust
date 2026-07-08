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
- **STORY-157 AC-157-008:** Factory codification story for this guidance
