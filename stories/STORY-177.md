---
document_type: story
story_id: STORY-177
epic_id: E-11
version: "1.0"
status: superseded
producer: story-writer
timestamp: 2026-07-18T00:00:00Z
phase: f7
level: feature
cycle: feature-iec104
points: 2
priority: P3
depends_on: []
blocks: []
# BC status: E-11 convention — governance-only story; no BCs authored
behavioral_contracts: []
verification_properties: []
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
target_module: .factory/maintenance/
subsystems: []
estimated_days: 1
wave: "TBD"
traces_to:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
  - .factory/maintenance/pr-manager-merge-auth-guidance.md
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "62d13e0"
---

# STORY-177: Feature-IEC104 Cycle-Close: Agent Dispatch and Reporting Discipline

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** superseded
**Wave:** TBD
**Points:** 2
**Priority:** P3

## Narrative

- **As a** orchestrator, spec-steward, and pipeline operator on the wirerust project
- **I want** two agent dispatch discipline rules codified: (1) the merge-authorization
  guidance updated with D-463 evidence showing that both subagent `--admin` merges on
  relayed consent AND orchestrator-direct unnamed `--admin` bypasses are denied, and
  (2) a rule that ALL agent types (adversary, spec-steward, implementer, or otherwise)
  must emit a final status report before idling or completing a pass — re-confirmed by
  a 2026-07-18 spec-steward dispatch going idle without reporting
- **So that** merge-auth failures are handled immediately by human-direct merge in the
  main thread, and silent idle conditions are distinguishable from genuine CLEAN passes
  without requiring session-context interrogation

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

### PG-MERGE-AUTH-SUBAGENT-CLASSIFIER — merge-auth resolution path update

STORY-163 AC-163-002 codified the first version of merge-authorization guidance:
subagents halt at MERGE-READY and escalate to the human thread for merge execution.

D-463 (STORY-174, 2026-07-17) revealed two additional failure modes not covered by the
original guidance:

1. **Orchestrator-direct `--admin` bypass denied:** When the orchestrator attempted
   an `--admin` merge directly (not via a subagent), this was also denied. The
   `--admin` flag does not provide authorization for non-human principals.

2. **Named `--admin` bypass also denied:** A second attempt using a named bypass was
   also rejected. No form of `--admin` merge relayed from an agent (subagent or
   orchestrator) provides the authorization that only direct human action does.

Resolution path confirmed: **human-direct merge in the main thread** is the only valid
execution path when `DF-MERGE-AUTH-CLASSIFIER-001` conditions are not met (wave grant
absent or condition-4 unmet).

PR #419 (2026-07-18) re-confirmed this resolution: the merge was executed human-direct
in the main thread after two classifier halts (same pattern as D-463).

The existing `.factory/maintenance/pr-manager-merge-auth-guidance.md` must be updated
to document both failure modes and the confirmed resolution path.

### PG-ADVERSARY-IDLE-NO-REPORT — agents must emit a final report before idling

During STORY-173 adversarial convergence (multiple passes), adversary agent instances
completed their review pass but emitted no explicit report — making it impossible to
distinguish a CLEAN pass (no findings) from an idle session (no work done). The operator
had to interrogate the session context to determine which condition held.

On 2026-07-18, a spec-steward dispatch went idle without reporting — confirming this is
not adversary-specific but a general agent discipline gap.

**Rule:** All agent types dispatched in the pipeline (adversary, spec-steward, implementer,
test-writer, etc.) MUST emit a final status report before the session ends. A CLEAN pass
must produce at minimum a "Pass N: CLEAN — zero findings" report line. An idle agent that
produces no output is non-conforming regardless of whether any findings exist.

This is a feature-iec104 cycle-execution finding + 2026-07-18 confirming occurrence —
DF-VALIDATION-001-exempt per the in-process exemption.

## Acceptance Criteria

### AC-177-001 (traces to PG-MERGE-AUTH-SUBAGENT-CLASSIFIER — merge-auth guidance update)

`.factory/maintenance/pr-manager-merge-auth-guidance.md` is updated to document the
expanded D-463 evidence. The update MUST:

(a) **Document both new failure modes:** Add a subsection or clearly delimited note
    stating that both of the following are denied and should not be attempted:
    - Subagent `--admin` merge on relayed human consent (existing guidance from AC-163-002)
    - Orchestrator-direct `--admin` merge (new, confirmed D-463 2026-07-17)
    - Any named `--admin` bypass from an agent (new, confirmed D-463 2026-07-17)

(b) **Affirm confirmed resolution path:** The guidance MUST state that the ONLY valid
    execution path when the DF-MERGE-AUTH-CLASSIFIER-001 conditions are unmet is
    **human-direct merge in the main thread**. The human opens the merge dialog or
    executes `gh pr merge` directly — no agent relay, no `--admin`, no bypass.

(c) **Cite D-463 and PR #419:** Both incidents MUST be cited as confirming evidence
    (D-463: STORY-174 wave-83, 2026-07-17; PR #419 re-confirmation: 2026-07-18).

(d) **Note the classifier halt as correct behavior:** The guidance MUST affirm that
    DF-MERGE-AUTH-CLASSIFIER-001 condition-4 (wave-grant-absent) halts are correct
    behavior, not a bug. The classifier protecting against unauthorized merges is
    functioning as designed.

Verification:
```bash
grep -n "D-463\|--admin\|orchestrator-direct\|human-direct\|PR #419" \
  .factory/maintenance/pr-manager-merge-auth-guidance.md
# Must emit non-empty output referencing both failure modes
```

### AC-177-002 (traces to PG-ADVERSARY-IDLE-NO-REPORT — agent-generic report-before-idle rule)

A new maintenance document `.factory/maintenance/agent-reporting-discipline.md` is
created (or an equivalent section is added to an appropriate existing maintenance doc)
codifying the agent-generic report-before-idle rule. The document MUST state:

(a) **Scope:** All agent types dispatched in the vsdd-factory pipeline on this project
    (adversary, spec-steward, implementer, test-writer, code-reviewer, demo-recorder, etc.)
    are subject to this rule. It is not limited to adversary agents.

(b) **Rule:** Every agent session MUST emit a final status report before the session ends
    or the agent idles. The minimum acceptable report is one of:
    - `"Pass N: CLEAN — zero findings"` (for review/adversary passes)
    - `"Pass N: FINDINGS — [count] finding(s): [severity list]"` (for passes with findings)
    - `"Task complete: [one-line summary of output]"` (for non-review dispatches)

(c) **Failure mode:** An agent that completes its work and then idles without emitting
    any report is non-conforming. The operator must treat an idle-without-report as an
    unknown state (potentially CLEAN, potentially idle — indistinguishable without
    interrogation). This is equivalent to a subprocess that exits without a status code.

(d) **Operator action:** If an agent idles without reporting, the operator MUST
    interrogate the agent's session to determine the actual state before recording the
    pass outcome. The interrogation result is then recorded in the session checkpoint.
    A fresh re-dispatch may be required if interrogation is inconclusive.

(e) **Confirming occurrences:** STORY-173 adversarial passes (multiple instances,
    2026-07-15/16) + spec-steward dispatch 2026-07-18 — both cited as confirming
    evidence that the gap is cross-agent-type.

Verification:
```bash
ls .factory/maintenance/agent-reporting-discipline.md
# Must exist

grep -n "CLEAN\|report\|idle\|Pass N" .factory/maintenance/agent-reporting-discipline.md
# Must emit non-empty output containing the rule
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Merge-auth guidance update | `.factory/maintenance/pr-manager-merge-auth-guidance.md` (amend) | factory-artifacts |
| Agent reporting discipline rule | `.factory/maintenance/agent-reporting-discipline.md` (create) | factory-artifacts |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. No `bin/` changes.
No develop PR required for either AC (factory-artifacts only).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | An agent emits a partial or truncated report (e.g., session cut off mid-sentence) | Operator must treat as idle-without-report and interrogate; a truncated report does not satisfy AC-177-002(b) |
| EC-002 | DF-MERGE-AUTH-CLASSIFIER-001 condition-4 is met (wave grant present) | Normal flow; subagent merge may be authorized. AC-177-001 applies only when conditions are unmet and the classifier halts |
| EC-003 | An agent dispatched for a purely read-only lookup (no review, no code changes) | A one-line task-complete summary still satisfies AC-177-002(b); the rule applies to all dispatches |

## Tasks

1. **Update pr-manager-merge-auth-guidance.md (AC-177-001):** Add D-463 failure modes,
   confirm human-direct resolution, cite PR #419 re-confirmation. Factory-artifacts
   branch commit.

2. **Create agent-reporting-discipline.md (AC-177-002):** New maintenance doc with
   agent-generic report-before-idle rule, scope, minimum report format, failure mode,
   operator action, confirming occurrences. Factory-artifacts branch commit.

3. **Register in STORY-INDEX.md:** Add STORY-177 row (draft, E-11, wave-TBD).
   Factory-artifacts branch commit.

> **Note for implementer:** Both ACs are factory-artifacts branch commits only — no
> develop PR required. This story extends/complements STORY-163 AC-163-002 (which covered
> the original subagent halt rule); AC-177-001 updates the same guidance doc with D-463
> evidence, and AC-177-002 adds the agent-generic report discipline that STORY-163 did
> not cover.

## Previous Story Intelligence

- **STORY-163 AC-163-002 (wave-73):** Original subagent merge-halt codification.
  STORY-177 AC-177-001 is an UPDATE to that same guidance doc — not a replacement.
  The original AC-163-002 content remains valid; AC-177-001 adds D-463 evidence and
  the orchestrator-direct bypass failure mode.
- **STORY-157 AC-157-004 (wave-71):** Original merge-authorization procedure codification.
  The guidance chain is STORY-157 → STORY-163 → STORY-177; each adds evidence.

## Notes

- **S-7.02 disposition:** Creating this story at draft status codifies two
  feature-iec104 cycle-execution process gaps: PG-MERGE-AUTH-SUBAGENT-CLASSIFIER
  (D-463 two failure modes + PR #419 re-confirmation, 2026-07-18) and
  PG-ADVERSARY-IDLE-NO-REPORT (STORY-173 adversarial passes + 2026-07-18 spec-steward
  confirming occurrence; generalized to all agent types per instruction).
- **DF-VALIDATION-001 gate:** Both gaps are in-process execution findings.
  DF-VALIDATION-001-exempt per the in-process exemption.
- **No behavioral contract required:** E-11 convention.
- **CLAUDE.md reference row:** The new `agent-reporting-discipline.md` maintenance doc
  MUST be added to the CLAUDE.md Project References table (same pattern as
  `pr-manager-merge-auth-guidance.md` and other maintenance docs listed there).

## Disposition

**Status:** superseded — routed upstream 2026-07-19

Both ACs address engine-level agent dispatch discipline. The merge-authorization guidance
lives in the vsdd-factory plugin's orchestrator prompts and hooks; the report-before-idle
rule likewise governs engine-level agent behavior across all projects. Neither fix lands
in wirerust repository files.

| AC | Upstream Disposition |
|----|---------------------|
| AC-177-001 (merge-auth guidance D-463 update) | drbothen/vsdd-factory#461 evidence comment, 2026-07-19 |
| AC-177-002 (agent-generic report-before-idle rule) | Confirmed duplicate of drbothen/vsdd-factory#457; no new issue filed, 2026-07-19 |

This story file is retained on disk for traceability. No further wirerust delivery expected.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: PG-MERGE-AUTH-SUBAGENT-CLASSIFIER (AC-177-001 merge-auth guidance D-463 update + PR #419 re-confirmation) + PG-ADVERSARY-IDLE-NO-REPORT (AC-177-002 agent-generic report-before-idle rule, generalized from adversary to all agent types). |
