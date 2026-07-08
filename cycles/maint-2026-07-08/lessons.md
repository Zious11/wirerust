# Lessons Learned — maint-2026-07-08

**Run ID:** maint-2026-07-08
**Date:** 2026-07-08
**Author:** session-reviewer (state-manager record)
**Format:** [codified] = AC added to a story; [observation] = noted but not yet codified

---

## Lessons

### L-001 — Wave Gate Code-Review Artifact Protocol (PG-W71-CODEREVIEW-ARTIFACT)

**Status:** [codified] → STORY-158 v1.1 AC-158-006

**Observation:** The wave-71 gate code-review dimension concluded "APPROVE — CR-W71-001 MINOR + 3 NITs" but no dedicated code-review report file was written to `cycles/wave-71/wave-gate/code-review.md`. The gate summary was the only artifact mentioning the MINOR. During DF-VALIDATION-001 triage this run, the MINOR was unverifiable — no evidence of which file, which line, or what the finding was. The 3 NITs were recoverable from per-story PR reviews but required manual excavation.

**Root cause:** The wave-gate code-review dimension protocol did not mandate that output be persisted to a dedicated file before closing the gate.

**Codified:** STORY-158 v1.1 added AC-158-006: "Every wave gate code-review dimension output MUST be written to `cycles/wave-NNN/wave-gate/code-review.md` before the gate is declared PASS. The file must enumerate every MINOR and NIT with: (a) source file, (b) line number or function name, (c) severity, (d) description. A gate summary row citing 'CR-X MINOR + N NITs' without a corresponding code-review.md is incomplete and leaves findings unverifiable for future DF-VALIDATION-001 triage."

---

### L-002 — Strict 3/3 Adversarial Convergence for Maintenance PRs

**Status:** [observation] — candidate for codification in maintenance checklist

**Observation:** All three maint-2026-07-08 PRs used strict 3/3 consecutive-clean fresh-context adversarial convergence (compared to the wave gate's standard 3/3 for feature PRs). This gate requirement was human-mandated for this run.

**Evidence that 3/3 caught substantive defects the prior lighter gate missed:**
- PR #382 Pass 1 missed an operator-boundary drift in arp.rs rustdoc; Pass 2 caught it.
- PR #382 passes identified an INVERTED ARP tuning rationale in README + --help: stated "decrease threshold" for false positives, correct is "increase threshold". This is a HIGH user-facing defect that would have shipped without the additional passes.
- ~7 extra adversary passes total across the 3 PRs caught 1 HIGH user-facing doc defect (inverted tuning advice).

**Implication:** The marginal cost of 3/3 vs 2/3 for maintenance PRs is low (~2-3 additional adversary passes per PR). The benefit is material — HIGH-severity documentation defects with user-facing impact are catchable. Recommend making strict 3/3 the default for any maintenance PR touching README, --help, or operator-guidance text.

---

### L-003 — Grep-Derived Finding Counts Must Be Re-Run by the Fixer (DF-SIBLING-SWEEP-001)

**Status:** [observation] — re-confirmation of DF-SIBLING-SWEEP-001

**Observation:** Pattern sweep (sweep-3) reported ~48 plain `+=` sites for PF-001. The fixer (PR #384) independently grep-ran the codebase and found 109 actual sites — more than double the reported count. The sweep report had enumerated a subset (file-by-file scan stopped at a representative set per file; `src/analyzer/dnp3.rs` alone contributed 25+ sites).

**Root cause:** Sweep reports enumerate representative examples per file but do not guarantee exhaustive counts. The fixer trusted the enumeration table rather than re-running `grep -rn "+=\s*1" src/`.

**Rule (re-confirmed):** The count in a sweep finding is a lower bound. The fixer MUST re-run the grep before writing the fix to establish the actual exhaustive count. The fix PR must state the re-run count, not the sweep-report count.

---

### L-004 — Subagent Permission Wall: pr-manager Cannot Execute gh pr merge

**Status:** [observation] — documented in DF-MERGE-AUTH-CLASSIFIER-001 / PR merge auth guidance

**Observation:** The pr-manager sub-agent cannot execute `gh pr merge` under the auto-mode classifier. All three maint-2026-07-08 merges required either main-session execution or human intervention. This is a known constraint (documented in `.factory/maintenance/pr-manager-merge-auth-guidance.md`), but each maintenance session must plan merge-step execution accordingly.

**Implication:** When planning a maintenance run, allocate one merge step per PR for main-session or human execution. Do not schedule merges as background pr-manager tasks — they will stall.

---

### L-005 — Background Adversary Agents: Relay Unreliability

**Status:** [observation] — new ENGINE-NOTE ADVERSARY-RELAY-UNRELIABLE-001

**Observation:** During this maintenance run, background-dispatched adversary agents were observed going idle without relaying their final reports back to the orchestrator. Two incidents: (1) an adversary agent for PR #382 Pass 2 completed its analysis internally but did not emit a report to the session; (2) a similar incident during PR #383 adversarial convergence. In both cases the synchronous workaround (re-dispatching the adversary in foreground mode) resolved the issue but added latency.

**Root cause:** Background agent completion notification is unreliable when the adversary agent exits cleanly but its output is buffered and not relayed. The pattern is specific to adversary agents with large output.

**Rule:** Dispatch adversary agents synchronously (`run_in_background: false`) for maintenance PRs. The reliability benefit outweighs the parallelism loss. Background dispatch is acceptable only for clearly bounded, short-output tasks.

---

## Summary Table

| ID | Type | Codified? | Story/AC |
|----|------|-----------|----------|
| L-001 | Artifact protocol gap | YES | STORY-158 v1.1 AC-158-006 |
| L-002 | Convergence gate calibration | Observation | — |
| L-003 | Sweep count re-run discipline | Observation (re-confirm) | — |
| L-004 | Permission wall — pr-manager | Observation | DF-MERGE-AUTH-CLASSIFIER-001 |
| L-005 | Background adversary relay | Observation | ENGINE-NOTE ADVERSARY-RELAY-UNRELIABLE-001 |
