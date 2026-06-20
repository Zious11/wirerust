---
document_type: lessons-learned
level: ops
version: "1.0"
status: in-progress
producer: state-manager
timestamp: 2026-06-19T04:00:00Z
cycle: feature-pcapng-reader
inputs: [STATE.md]
input-hash: pending
traces_to: STATE.md
---

# Lessons Learned — feature-pcapng-reader

<!-- Durable lessons from this cycle for future VSDD factory runs.
     Organized by category: agent-level, process-level, infrastructure-level.
     Each lesson is numbered continuously and includes the pass/burst
     where it was discovered. -->

## Agent-Level

1. **[process-gap] State-manager must not edit spec/ADR content** — During the D-139 commit burst the state-manager directly edited ADR content (mergecap hint wording) in addition to state tracking files. State-manager scope is restricted to state/index files (STATE.md, cycle logs, manifests, burst-logs). Spec and ADR content is the architect's lane. The ADR-009 edit had to be reverted by the architect and re-applied correctly, adding a reconciliation round-trip. Always route spec/ADR edits through the architect agent.
   _Discovered: D-139 burst, 2026-06-19. Reconciled: D-140, 2026-06-19._

## Process-Level

<!-- Reserved for process-level lessons discovered during this cycle. -->

## Infrastructure-Level

<!-- Reserved for infrastructure-level lessons discovered during this cycle. -->

## Policy Candidates

| Lesson | Proposed Policy | Scope | Status |
|--------|----------------|-------|--------|
| 1 | DF-STATE-MANAGER-SCOPE-001: state-manager must not write spec/ADR content | State-manager agent boundaries | proposed |
