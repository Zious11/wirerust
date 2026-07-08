# STORY-157 Adversarial Findings Tracker

**Cycle:** wave-71
**Story:** STORY-157 — Input Hash Computation (compute-input-hash tool + CLAUDE.md docs)
**Convergence status:** CONVERGED (2026-07-08) — streak 3/3 (passes 4, 5, 6 CLEAN)
**Criterion:** BC-5.39.001 (three consecutive CLEAN passes)

---

## Pass Summary

| Pass | Date       | Verdict   | Streak | Findings (MED/LOW/NITPICK) | Head Reviewed |
|------|------------|-----------|--------|----------------------------|---------------|
| 1    | 2026-07-08 | NOT-CLEAN | 0      | 3 MED / 0 LOW / 0 NITPICK  | 2927c02       |
| 2    | 2026-07-08 | NOT-CLEAN | 0      | 1 MED / 2 LOW / 0 NITPICK  | 2927c02       |
| 3    | 2026-07-08 | NOT-CLEAN | 0      | 1 MED / 1 LOW / 0 NITPICK  | 2927c02       |
| 4    | 2026-07-08 | CLEAN     | 1      | 0 MED / 2 LOW / 1 NITPICK  | e023e79       |
| 5    | 2026-07-08 | CLEAN     | 2      | 0 / 0 / 0 (zero findings)   | 70d99ad       |
| 6    | 2026-07-08 | CLEAN     | 3      | 0 MED / 3 LOW / 2 NITPICK  | 70d99ad       |

---

## Pass 1 Findings (NOT-CLEAN)

### F-157-P1-001 — MED — Hallucinated hash in Notes
- **Finding:** STORY-157 Notes section cited hash f401b29 which did not correspond to any commit on develop.
- **Disposition:** REMEDIATED
- **Remediation:** STORY-157 updated to v1.5 then v1.6; hash corrected to actual commit value.

### F-157-P1-002 — MED — Classifier NITPICK_ONLY path excluded CLEAN
- **Finding:** policies.yaml CLASSIFIER-001 lacked a NITPICK_ONLY branch, making CLEAN unreachable for passes that returned only nitpick-level observations.
- **Disposition:** REMEDIATED
- **Remediation:** CLASSIFIER-001 amended — NITPICK_ONLY branch added with explicit CLEAN verdict mapping.

### F-157-P1-003 — MED — COMPLETE-001/CLASSIFIER-001 terminal-state gap
- **Finding:** Neither COMPLETE-001 nor CLASSIFIER-001 defined the convergence exit condition (streak >= 3 CLEAN passes). The policies described per-pass behavior but left the terminal state implicit.
- **Disposition:** REMEDIATED
- **Remediation:** COMPLETE-001 v2 added terminal-state section; CLASSIFIER-001 updated with cross-reference.

---

## Pass 2 Findings (NOT-CLEAN)

### F-157-P2-001 — MED — "(current frontmatter)" staleness recurrence
- **Finding:** STORY-157 still contained the placeholder text "(current frontmatter)" after the v1.6 edit, indicating an incomplete field substitution.
- **Disposition:** REMEDIATED
- **Remediation:** STORY-157 updated to v1.7; all placeholder text replaced with live values.

### F-157-P2-002 — LOW — Amendment-field convention violation
- **Finding:** policies.yaml amendment entries lacked the required amendment_date and amendment_reason fields per the established amendment convention.
- **Disposition:** REMEDIATED
- **Remediation:** Amendment fields added to all affected entries in policies.yaml.

### F-157-P2-003 — LOW — Ambiguous a-fortiori wording in COMPLETE-001
- **Finding:** The phrase "even stronger" in COMPLETE-001 v1 read as introducing an additional condition rather than clarifying the existing convergence criterion.
- **Disposition:** REMEDIATED
- **Remediation:** COMPLETE-001 v2 wording revised for clarity; hash updated to 4ca0ad4.

---

## Pass 3 Findings (NOT-CLEAN)

### F-157-P3-001 — MED — Stale RED banner per DF-GREEN-DOC-TENSE-SWEEP
- **Finding:** The compute-input-hash self-test file retained a RED status banner describing a failing assertion that had already been fixed in a prior commit.
- **Disposition:** REMEDIATED
- **Remediation:** Commit e023e79 — banner updated to GREEN; doc tense corrected throughout.

### F-157-P3-002 — LOW — Docstring coverage: 3 of 9
- **Finding:** Only 3 of 9 public functions in the compute-input-hash module had docstrings.
- **Disposition:** REMEDIATED
- **Remediation:** Commit e023e79 — docstrings added to all 9 public functions.

### Pass 3 Observations (ACCEPTED)

| ID      | Summary                                                       | Disposition |
|---------|---------------------------------------------------------------|-------------|
| OBS-001 | Historical clause narrative in COMPLETE-001 clear and useful  | ACCEPTED    |
| OBS-002 | Chain ergonomics in compute-input-hash could be improved      | ACCEPTED (maintenance) |

---

## Pass 4 Findings (CLEAN — streak 1)

### F-157-P4-OBS-001 — LOW — CLAUDE.md historical clause ref casing
- **Disposition:** REMEDIATED — Commit 70d99ad normalized casing.

### F-157-P4-OBS-003 — LOW — Missing Python 3.10+ floor in CLAUDE.md
- **Disposition:** REMEDIATED — Commit 70d99ad added Python 3.10+ minimum version floor.

### F-157-P4-OBS-002 — LOW — DF-INPUT-HASH-CANONICAL-001 cross-ref extension — DEFERRED
- **Finding:** CLAUDE.md would benefit from a cross-reference to DF-INPUT-HASH-CANONICAL-001 in policies.yaml.
- **Disposition:** DEFERRED TO MAINTENANCE BACKLOG
- **Rationale:** Editorial improvement; no correctness impact. Out of scope for STORY-157 delivery.

### F-157-P4-NITPICK-001 — NITPICK — --write touches MATCH files — DEFERRED
- **Finding:** The --write flag in compute-input-hash rewrites files even when their stored hash already matches the computed hash (no-op write). Pre-existing behavior, not introduced by STORY-157.
- **Disposition:** DEFERRED TO MAINTENANCE BACKLOG
- **Rationale:** Pre-existing behavior; touched files are idempotent. Optimization deferred.

---

## Pass 5 Findings (CLEAN — streak 2)

Zero findings at any severity level. Clean pass with no observations.

---

## Pass 6 Findings (CLEAN — streak 3 — CONVERGED)

### F-157-P6-OBS-001 — LOW — Cross-half doc window
- **Disposition:** ACCEPTED — Existing cross-half convention; no action required.

### F-157-P6-OBS-002 — LOW — FSR row site drift
- **Disposition:** ACCEPTED — Within AC-157-002 latitude.

### F-157-P6-OBS-003 — NITPICK — Token-budget weight underweight
- **Disposition:** ACCEPTED — Cosmetic; no action required.

### F-157-P6-OBS-004 — LOW — sed path-depth portability — DEFERRED
- **Finding:** The sed path-depth example in maintenance/demo-evidence-scrub-gate.md may not generalize to deeply nested paths on Linux distributions.
- **Disposition:** DEFERRED TO MAINTENANCE BACKLOG

### F-157-P6-OBS-005 — LOW — macOS/BSD sed specificity — DEFERRED
- **Finding:** Scrub-gate example uses BSD sed syntax not portable to GNU sed (Linux default).
- **Disposition:** DEFERRED TO MAINTENANCE BACKLOG

### F-157-P6-OBS-006 — LOW — exec-namespace __name__ guard absent — DEFERRED
- **Finding:** compute-input-hash module lacks if __name__ == "__main__": guard.
- **Disposition:** DEFERRED TO MAINTENANCE BACKLOG
- **Note:** Out of scope for STORY-157 delivery.

---

## Deferred-to-Maintenance Summary

| ID                   | Severity | Summary                                                    | Deferred From |
|----------------------|----------|------------------------------------------------------------|---------------|
| F-157-P4-OBS-002     | LOW      | DF-INPUT-HASH-CANONICAL-001 cross-ref in CLAUDE.md         | Pass 4        |
| F-157-P4-NITPICK-001 | NITPICK  | --write touches MATCH files (pre-existing, optimize later) | Pass 4        |
| F-157-P6-OBS-004     | LOW      | sed path-depth portability in scrub-gate example           | Pass 6        |
| F-157-P6-OBS-005     | LOW      | macOS/BSD sed vs GNU sed portability                       | Pass 6        |
| F-157-P6-OBS-006     | LOW      | exec-namespace __name__ guard absent in compute-input-hash | Pass 6        |

---

## Wave-71 Session Process Observations (Cycle Close)

### PROC-OBS-001 — Teammate mailbox one-message-per-activation latency [PROCESS-GAP]
- **Observation:** Agent-mailbox model delivered one message per activation cycle; orchestrator issued approximately 6 nudges across wave-71 to advance blocked agents waiting for follow-up messages in the same turn.
- **Impact:** Each nudge introduced a full activation-cycle delay. Cumulative delay across 3 stories was approximately 6 activation cycles.
- **Mitigation:** None structural during wave-71; nudging was manual. Recommendation: batch multi-step instructions into single activation messages.

### PROC-OBS-002 — Partial-fix propagation class recurred [PROCESS-GAP, largely mitigated]
- **Observation:** Across passes 1-3 for STORY-157, partial-fix propagation was the dominant finding class: a fix applied to one file but the same issue persisting in sibling files.
- **Root cause:** Implementer agents performing targeted single-file edits without sweeping siblings.
- **Mitigation:** DF-SIBLING-SWEEP-001 grep-list dispatch discipline adopted mid-wave. Recurrence rate dropped to near-zero after discipline was adopted.
- **Residual risk:** Discipline is procedural (relies on orchestrator dispatch), not structural (no automated gate).

### PROC-OBS-003 — Adversary wrong-tree read mitigated [RESOLVED]
- **Observation:** During STORY-156 Pass 4, the adversary agent read from the wrong worktree, producing findings based on stale file state.
- **Mitigation:** Tree-discipline preamble added to adversary dispatch prompts. Codified as DF-ADVERSARY-CHECKOUT-GUARD-002 in policies.yaml.
- **Status:** RESOLVED — no recurrences after guard was applied.

---

*Generated: 2026-07-08 | STORY-157 | wave-71 | State Manager*
