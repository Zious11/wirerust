# Lessons Learned — maint-2026-07-09

**Run ID:** maint-2026-07-09
**Date:** 2026-07-10
**Author:** session-reviewer (state-manager record)
**Format:** [codified] = AC added to a story; [observation] = noted but not yet codified

---

## Lessons

### L-001 — Docs-Dispatch Without Ground-Truth Citation Mandate (PG-RA-P3-ARP-REC006-INVERSION-001)

**Status:** [codified] → STORY-163 AC-163-001

**Observation:** During the maint-2026-07-09 Route A adversarial convergence, adversary Pass 3 (finding F-RA-P3-001) exposed that the routeA-docs-writer dispatch had paraphrased a one-line sweep recommendation (REC-006) and produced a README claim that was factually inverted: the draft text stated that VLAN/QinQ/MACsec-tagged ARP frames produce no findings, when `src/decoder.rs` D-078/D-078b provably handles those frames via the lax path and does produce findings for them. The inversion was caught by adversary Pass 3 and fixed before merge; however, the dispatch pattern remains a latent hazard for any future docs-remediation task that touches behavioral claims.

**Root cause:** The dispatch task contained only the one-line REC-006 summary from the sweep report. The docs-writer was not required to cite a ground-truth file:line anchor for each behavioral claim before producing the output, and was not told which source files to Read for verification.

**Codified:** STORY-163 AC-163-001 — create `.factory/maintenance/docs-writer-dispatch-guidance.md` codifying the ground-truth citation mandate for docs-remediation dispatches (scope, mandate, inversion-prevention rule, verification template, REC-006/F-RA-P3-001 concrete example).

---

### L-002 — Harness-Classifier Halt: Subagent Merge Denied (PG-MERGE-AUTH-SUBAGENT-CLASSIFIER)

**Status:** [codified] → STORY-163 AC-163-002

**Observation:** During the PR #393 merge step (2026-07-10), the harness auto-mode permission classifier denied `gh pr merge` when executed by pr-manager (a subagent). The classifier's denial was correct: human consent for the merge was present only as a relayed message in the subagent's teammate-message context, not as a visible authorization in the main conversation thread. This is a new failure mode not covered by the existing `pr-manager-merge-auth-guidance.md` (DF-MERGE-AUTH-CLASSIFIER-001 companion), which addresses whether pr-manager should attempt a merge, but does not specify what happens when the harness itself denies the attempt.

**Root cause:** The existing guidance (DF-MERGE-AUTH-CLASSIFIER-001, STORY-157 AC-157-008) covers the `AUTHORIZE_MERGE=yes` ambiguity (human vs. orchestrator grant) but does not cover the distinct case where the harness permission system itself blocks the merge tool call because authorization is not visible in the calling agent's conversation thread. These are two different failure modes: the first is a policy question (should we merge?), the second is a harness enforcement question (can this agent execute the merge call?).

**Resolution (this run):** The orchestrator executed `gh pr merge` in the main conversation thread under direct user authorization given in that thread; pr-manager then completed step-9 cleanup after merge confirmation.

**Codified:** STORY-163 AC-163-002 — add "Harness-Classifier Halt: Subagent Merge Denied" section to `.factory/maintenance/pr-manager-merge-auth-guidance.md`.

---

### L-003 — README ARP Schema Inaccuracy (PG-W-README-JSON-SCHEMA)

**Status:** [register] → tech-debt-register row PG-W-README-JSON-SCHEMA (P3, OPEN)

**Observation:** README § ARP section describes `arp_summary` as a nested JSON key, but ARP counters live flat in `analyzers[i].detail` (`src/analyzer/arp.rs` vs README ARP section). DNP3 wording was corrected in PR #393 but ARP remains inaccurate. The ENIP-nested vs flat-detail schema asymmetry should also be reconciled (fix README ARP wording, or nest counters for real). Source: adversary F-RA-P2-002 [process-gap], maint-2026-07-09.

**Disposition:** Registered as tech-debt row PG-W-README-JSON-SCHEMA (P3, OPEN) — target: next docs PR or maintenance run. No codification needed (content defect, not a process gap requiring a story AC).

---

## Summary Table

| ID | Type | Codified? | Story/AC |
|----|------|-----------|----------|
| L-001 | Docs-dispatch citation mandate gap | YES | STORY-163 AC-163-001 |
| L-002 | Harness-classifier subagent merge halt | YES | STORY-163 AC-163-002 |
| L-003 | README ARP schema inaccuracy | Register (no codification) | tech-debt PG-W-README-JSON-SCHEMA |
