---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-23T08:00:00Z
cycle: feature-mitre-json-names
inputs: [STATE.md]
input-hash: ""
traces_to: STATE.md
---

# Lessons Learned — feature-mitre-json-names

<!-- Durable lessons from this cycle for future VSDD factory runs.
     Organized by category: agent-level, process-level, infrastructure-level.
     Each lesson is numbered continuously and includes the pass/burst
     where it was discovered. -->

## Agent-Level

_(none recorded yet)_

## Process-Level

1. **Commit ALL story-relevant working-tree changes (tests + src) atomically** — During the F5
   ICS fix, the implementer committed only `src/mitre.rs` (719816e), leaving the test-writer's
   corrected assertions in `tests/bc_2_09_100_multitag_tests.rs`,
   `tests/bc_2_16_story114_arp_tests.rs`, and `tests/reporter_json_tests.rs` as UNCOMMITTED
   working-tree edits. The 3 per-fix adversarial passes read the working tree (saw correct
   values, reported CLEAN/green) while the committed SHAs (74a48ea/cf22de9) still carried the
   OLD wrong assertions. The divergence was caught only when PR #307 CI failed on the pushed
   (committed) state; pr-manager committed the corrections as 96f0afc. Outcome: final merged
   state 96f0afc is correct and CI-green, and matches what the adversaries reviewed.
   _Discovered: F5 ICS fix burst, 2026-06-23_
   _DRIFT-UNCOMMITTED-TEST-EDITS-001 [process-gap, MEDIUM]_

2. **Variant/count changes must trigger a repo-wide magic-number grep across ALL spec layers, not just the directly-edited BCs** — The F5 BC update (PO) bumped BC-2.10.002/003/007 + BC-2.16.004 to reflect the 20-variant MitreTactic enum (14 Enterprise + 6 ICS) but did NOT sweep the L2 domain specs (cap-10/11, ent-04/05), VP-016, NFR-catalog, test-vectors, prd.md, module-criticality, or BC-2.10.004 which independently asserted the OLD tactic-variant count ('17 variants / 14 Enterprise + 3 ICS'). The per-fix adversarial passes reviewed only the directly-changed files; the F7 consistency-validator was scoped to the feature delta and also missed the broad count references in those sibling artifacts. The incomplete propagation was caught only by an orchestrator-run wide grep for the magic number '17' across all spec layers. Guard: variant/count changes must trigger a repo-wide magic-number grep across ALL spec layers (BCs, VPs, NFR catalog, test vectors, prd.md, domain specs, module-criticality) before the fix burst is considered complete.
   _Discovered: F7 consistency sweep, 2026-06-23_
   _D-215 — DF-SIBLING-SWEEP-001_

## Infrastructure-Level

_(none recorded yet)_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | convergence-clean-tree-guard: per-fix/per-story convergence dispatch must require a `git status --short` CLEAN attestation so the reviewed tree == committed tree; supplied execution evidence must come from the COMMITTED tree (or CI), not an uncommitted working tree | engine / orchestrator | proposed |
| 2 | magic-number-sweep-on-count-change: whenever a variant/count value changes in any BC, the sweep protocol (DF-SIBLING-SWEEP-001) must be explicitly applied to ALL spec layers (BCs + VPs + L2 domain specs + NFR catalog + test-vectors + prd.md + module-criticality) before the fix is committed, not just the directly-modified BCs | orchestrator / PO / DF-SIBLING-SWEEP-001 | proposed |
