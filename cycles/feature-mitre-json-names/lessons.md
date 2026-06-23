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

## Infrastructure-Level

_(none recorded yet)_

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | convergence-clean-tree-guard: per-fix/per-story convergence dispatch must require a `git status --short` CLEAN attestation so the reviewed tree == committed tree; supplied execution evidence must come from the COMMITTED tree (or CI), not an uncommitted working tree | engine / orchestrator | proposed |
