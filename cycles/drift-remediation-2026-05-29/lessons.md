---
document_type: lessons
cycle: drift-remediation-2026-05-29
date: 2026-05-29
produced_by: state-manager
---

# Drift Remediation 2026-05-29 — Lessons Learned

## DR.L1 — Validate-Before-Fix Pays Off (DF-VALIDATION-001)

A research-agent validation pass before remediation cut the 62-item backlog substantially:
approximately 13 items were already-resolved, invalid, or duplicate before any remediation
work began. DF-VALIDATION-001 prevented wasted effort and false issue-filing. The policy
investment (Phase 0 / PR #99) paid clear dividends at scale.

**Pattern:** When a backlog of deferred findings accumulates across multiple waves, a bulk
validation pass eliminates phantom work before execution.

## DR.L2 — pr-manager APPROVE Recurrence (DF-PR-MANAGER-COMPLETE-001 Upstream)

pr-manager AGAIN stopped at APPROVE on PR #147, triggering a manual orchestrator merge.
This is the Nth recurrence of DF-PR-MANAGER-COMPLETE-001 (W9.L3, W10.L3, W11.L5, confirmed
again here). The policy is correct and injected at dispatch time, but the root-cause fix
requires an agent-prompt change in the vsdd-factory plugin source — not in this repo.

**Pattern:** Policy enforcement that lives only in dispatch-injection text is fragile.
Structural fixes (agent-prompt changes) are upstream; document the workaround clearly.

**Action:** W10-D7 escalation to plugin maintainer confirmed necessary.

## DR.L3 — Feasibility Probe Before Treating Coverage Items as Real Gaps

A documented "coverage gap" (F-W16-S052-P2-002 / BC-2.07.001 EC-002 inner Err arm) turned
out to be UNREACHABLE dead code on investigation — the nom `many0`/`complete` semantics
prevent the branch from being exercised through the public on_data API. The item was
reclassified WONT-FIX-BY-DESIGN.

**Pattern:** Before treating a "needs-test" finding as a real spec gap, probe feasibility:
Can the trigger condition actually be induced through the public API? If not, the branch
is defensive dead code, not a coverage gap. Research-agent feasibility checks on coverage
items should be the first step.

## DR.L4 — Bulk Template Defects Require Repo-Wide Sweep, Not Per-Item Handling

SS-01's capabilities.md citation fix (DF-16.A, 8 files) surfaced a 209-file project-wide
blast radius (DF-16.B) — a single broken citation template at Phase-1a authoring propagated
to every BC across SS-02..SS-13. Fixing one SS per wave would take 12+ waves.

**Pattern:** When a structural defect in a template or authoring tool affects a corpus
of generated files, the correct response is a single bulk find-replace sweep across the
entire corpus, not per-item version-bump handling. DF-16.B is queued for a dedicated
bulk mechanical sweep rather than incremental fixes.

**Action:** DF-16.B added to open Drift Items with explicit "bulk sweep" disposition.
