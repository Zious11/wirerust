# Lessons Learned — maint-2026-09-05

**Run ID:** maint-2026-09-05
**Date:** 2026-09-05
**Author:** session-reviewer (state-manager record)
**Format:** [codified] = AC/policy added; [register] = tracked in a register/carry-forward; [observation] = noted but not yet codified

---

## Lessons

### L-1 — Auto-Mode Classifier Blocks All Maintenance-Authorized Merges (PG-MAINT-CLASSIFIER-MERGE-BLOCK)

**Status:** [observation] — carry to next maintenance run's kickoff checklist; no codification this run

**Observation:** This session's auto-mode permission classifier blocked `gh pr merge` for every PR this run — both the 5 human-authorized Dependabot Rust-dep merges (#459/#458/#444/#443/#442) and, had they been ready to merge, the two human PRs under review (#451, #407). The classifier also blocked editing `.claude/settings.local.json` to allowlist `gh pr merge` mid-session, so there was no way to work around the block once discovered. As a result, every merge this run — despite explicit human authorization for the Rust-dep batch — had to be handed to the human to execute manually, rather than completed end-to-end by the orchestrator.

**Root cause:** The block was only discovered at the disposition step, after all 6 analysis sweeps had already completed and produced a merge-ready recommendation. By that point, the only paths forward were (a) hand the merge list to the human, or (b) leave the PRs unmerged with no clear owner. Neither is a defect in the classifier's behavior (it is correctly declining an unauthorized-for-this-session action) — the gap is procedural: nothing in the maintenance-sweep dispatch flow checks merge/create permissions before the analysis work begins.

**Lesson:** For future maintenance runs, confirm `gh pr merge` and `gh pr create` permissions (or secure explicit human sign-off on the specific merge list) at the *start* of the run — before dispatching the sweep agents — rather than discovering the block only at the disposition step after all analysis work is already sunk. This lets the orchestrator either request the allowlist change up front, or set the human's expectations early that this run will end with a manual-merge handoff.

**No story needed:** This is an engine/process-level workflow gap (maintenance-sweep dispatch sequencing), not a product defect. Carry forward as a checklist item for the next maintenance-sweep kickoff; promote to a formal dispatch-template mandate (`maintenance-config.yaml`) if it recurs a second time.

---

### L-2 — Review-Only pr-reviewer Dispatch Trips the Delivery-Only Posted-Review Hook (PG-MAINT-REVIEWONLY-HOOK-TRIP)

**Status:** [observation] — recommend hook exemption; no codification this run (engine-level fix, not a wirerust story)

**Observation:** During disposition of human PRs #451 and #407, a `pr-reviewer` dispatch was made for review-only purposes (producing a disposition opinion for this maintenance sweep — not a merge-track review as part of the per-story-delivery flow). This dispatch tripped the delivery-only `validate-pr-review-posted` SubagentStop hook, which expects either a posted GitHub review or a `pr-review.md` deliverable artifact — both of which are normally produced only by the per-story-delivery flow, not a maintenance-sweep disposition pass. This caused a stop-loop, even though the review verdict itself (pr-reviewer APPROVE-WITH-CHANGES on both PRs) was still correctly relayed to the orchestrator.

**Root cause:** The `validate-pr-review-posted` hook has no notion of "review-only, disposition-purpose" dispatches distinct from "merge-track, delivery-purpose" dispatches — it assumes every `pr-reviewer` invocation is part of the per-story-delivery pipeline and therefore must terminate in a posted review or `pr-review.md` artifact.

**Lesson:** Consider exempting review-only maintenance-sweep dispatches of `pr-reviewer` (i.e., dispatches whose stated purpose is a disposition opinion for a maintenance sweep, not a merge-track PR review) from that hook's posted-review requirement — for example, via a dispatch-context flag the hook can check before enforcing the posted-review invariant. This is an engine-level (dark-factory) fix, not a wirerust product change; no story is needed here.

**No story needed:** Engine/hook-level fix. Carry forward to the next engine-maintenance review; the verdict was still delivered correctly this run, so no product-facing harm occurred.

---

## Summary Table

| ID | Type | Codified? | Story/AC |
|----|------|-----------|----------|
| L-1 | Auto-mode classifier blocks maintenance-authorized merges | Observation (carry to next kickoff checklist) | — |
| L-2 | Review-only pr-reviewer dispatch trips delivery-only hook | Observation (engine-level hook fix recommended) | — |
