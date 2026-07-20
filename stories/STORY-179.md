---
document_type: story
story_id: STORY-179
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
  - .factory/maintenance/delivery-doc-currency-protocol.md
inputs:
  - .factory/STATE.md
  - .factory/cycles/feature-iec104/convergence-trajectory.md
input-hash: "41176f4"
---

# STORY-179: Feature-IEC104 Cycle-Close: Session Recovery and Multi-Worktree Verification

**Epic:** E-11 (Tooling and Self-Improvement)
**Status:** superseded
**Wave:** TBD
**Points:** 2
**Priority:** P3

## Narrative

- **As a** orchestrator and pipeline operator on the wirerust project
- **I want** the session-boundary state recovery protocol updated to mandate simultaneous
  verification of ALL git worktrees AND the main develop checkout, and post-agent
  verification scope likewise expanded to cover all worktrees plus the main checkout
- **So that** the class of stray-commit incidents (stray commit `105497f` on the main
  develop checkout, D-458) is caught during recovery rather than discovered mid-wave,
  and agents that commit to the wrong location (main checkout instead of a worktree) are
  detected immediately

## Behavioral Contracts

_(none — E-11 convention: governance-only story; no BCs authored; no Rust source modified)_

## Background

### PG-STATE-RECOVERY-SCOPE and PG-VERIFY-ALL-WORKTREES — stray-commit 105497f incident

During STORY-172 delivery (D-458, 2026-07-15), a fix agent committed changes to the
**main develop checkout** rather than the intended worktree. This produced stray commit
`105497f` on the main develop branch (not in any worktree). The stray commit was
undetected during the post-agent verification pass because:

1. Verification only inspected the active worktree — it did not check the main develop
   checkout.
2. Session recovery at the next session boundary also missed it because the recovery
   procedure checked worktrees but not the main checkout.

The stray commit had to be discarded (not squash-merged into the wave PR) when discovered
during the wave-81 delivery cycle.

**Root cause:** Both session-boundary recovery and post-agent verification had an
implicit assumption that all work lands in worktrees. When an agent commits to the main
develop checkout (which is a valid git operation — the main checkout is always present
alongside any worktrees), neither check caught it.

**Correct scope for both checks:**
```
git worktree list                    # enumerate all worktrees + main checkout
for each location:
  git -C <path> log --oneline -5    # check for unexpected commits
  git -C <path> status              # check for uncommitted changes
```

The main develop checkout is always present at the repo root and MUST be included in
both the recovery and post-agent verification scans.

These are feature-iec104 cycle-execution findings — DF-VALIDATION-001-exempt per the
in-process exemption.

## Acceptance Criteria

### AC-179-001 (traces to PG-STATE-RECOVERY-SCOPE — session-boundary recovery protocol update)

A new maintenance document `.factory/maintenance/session-recovery-protocol.md` is
created (or an equivalent section is added to an appropriate existing maintenance doc)
codifying the expanded session-boundary recovery scope. The document MUST:

(a) **Explicit multi-location scope:** State that session-boundary recovery MUST verify
    ALL of the following simultaneously:
    - Every active git worktree (`git worktree list` enumerates them)
    - The main develop checkout at the repo root

(b) **Recovery procedure:** Provide the canonical recovery steps:
    ```bash
    # Step 1: enumerate all locations
    git worktree list

    # Step 2: for each location (main checkout + each worktree path):
    git -C <path> log --oneline HEAD~5..HEAD   # check for unexpected commits
    git -C <path> status                        # check for uncommitted changes
    git -C <path> branch --show-current         # confirm expected branch
    ```

(c) **Stray-commit handling:** If a commit is found on the main develop checkout that
    was not part of an authorized merge (e.g., a commit from an agent working in what
    it thought was a worktree but committed to the main checkout instead), the operator
    MUST:
    - Record the stray commit SHA and its content
    - Determine if the content is salvageable (cherry-pick to the correct branch) or
      must be discarded
    - Reset the main develop checkout: `git reset --hard origin/develop` (after
      confirming the content decision)
    - Record the incident in the session checkpoint

(d) **Reference:** Cite stray-commit `105497f` incident (D-458, STORY-172 wave-81,
    2026-07-15) as the confirming incident that motivated this protocol.

Verification:
```bash
ls .factory/maintenance/session-recovery-protocol.md
# Must exist

grep -n "worktree list\|main.*checkout\|105497f\|stray" \
  .factory/maintenance/session-recovery-protocol.md
# Must emit non-empty output
```

### AC-179-002 (traces to PG-VERIFY-ALL-WORKTREES — post-agent verification scope)

`.factory/maintenance/delivery-doc-currency-protocol.md` (or the new session-recovery
doc from AC-179-001) is extended with a post-agent verification scope rule. The rule
MUST:

(a) **Scope statement:** After any agent dispatch that may commit code (implementer,
    fix agent, spec-steward in write mode, etc.), post-agent verification MUST span:
    - The intended target worktree (the one the agent was dispatched into)
    - ALL other active worktrees
    - The main develop checkout at the repo root

(b) **Why the main checkout:** An agent that commits to the main develop checkout
    instead of a worktree produces a stray commit that a worktree-only check will miss.
    The main checkout is always present alongside any worktrees and must always be
    in scope.

(c) **Minimum check:** For each location:
    ```bash
    git -C <path> log --oneline HEAD~3..HEAD
    # Look for unexpected commits authored in the last session
    ```

(d) **Incomplete verification:** A post-agent verification that checks only the active
    worktree without checking the main develop checkout is INCOMPLETE and must not be
    recorded as a clean verification pass.

(e) **Reference:** Stray-commit `105497f` incident (D-458, 2026-07-15) as confirming
    evidence.

Verification:
```bash
grep -n "worktree\|main.*checkout\|post-agent\|PG-VERIFY-ALL" \
  .factory/maintenance/delivery-doc-currency-protocol.md
# Must emit non-empty output containing the scope rule
```

## Architecture Mapping

| Component | File | Branch |
|-----------|------|--------|
| Session recovery protocol | `.factory/maintenance/session-recovery-protocol.md` (create) | factory-artifacts |
| Post-agent verification scope rule | `.factory/maintenance/delivery-doc-currency-protocol.md` (amend) | factory-artifacts |

No Rust source files in `src/`, no `tests/`, no `Cargo.toml` changes. No `bin/` changes.
No develop PR required (factory-artifacts only).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | No active worktrees — developer is working in the main checkout only | `git worktree list` returns one entry (the main checkout); recovery and post-agent verification apply to it as the sole location |
| EC-002 | A worktree is removed (pruned) between agent dispatch and verification | `git worktree list` will not show pruned worktrees; this is not an error — the verification scope is the set of currently-listed worktrees plus the main checkout |
| EC-003 | Agent was dispatched read-only (no commit expected) | Post-agent verification still recommended as a lightweight check; a read-only dispatch that produces an unexpected commit indicates a bug in the dispatch |
| EC-004 | Main develop checkout has authorized commits from a recently merged PR | These appear in `git log` with their PR merge SHA; distinguishable from stray commits by their PR association and committer identity |

## Tasks

1. **Create session-recovery-protocol.md (AC-179-001):** New maintenance doc with
   multi-location scope statement, canonical recovery procedure, stray-commit handling,
   and `105497f` incident citation. Factory-artifacts branch commit.

2. **Extend delivery-doc-currency-protocol.md (AC-179-002):** Add post-agent
   verification scope rule covering all worktrees + main checkout. Factory-artifacts
   branch commit.

3. **Update CLAUDE.md Project References table:** Add `session-recovery-protocol.md`
   row (same pattern as other `.factory/maintenance/` docs listed there). Develop branch.

4. **Register in STORY-INDEX.md:** Add STORY-179 row (draft, E-11, wave-TBD).
   Factory-artifacts branch commit.

> **Note for implementer:** Tasks 1 and 2 are factory-artifacts branch commits. Task 3
> (CLAUDE.md) is a develop-branch change (no CHANGELOG entry required — CLAUDE.md is not
> in the AC-158-001 trigger set). Task 3 can be batched with the CLAUDE.md changes from
> STORY-178 AC-178-001d if the two stories are delivered in the same wave.

## Previous Story Intelligence

- **STORY-165 AC-165-003 (wave-75):** Established `delivery-doc-currency-protocol.md`
  as the canonical delivery sweep document. STORY-179 AC-179-002 amends the same
  document with a post-agent verification scope rule.
- **No direct predecessor for AC-179-001:** The session-recovery-protocol.md is a new
  document. The closest precedent is the session checkpoint format established across
  multiple waves (documented in STATE.md session-checkpoints archive).

## Notes

- **S-7.02 disposition:** Creating this story at draft status codifies two paired
  feature-iec104 cycle-execution process gaps: PG-STATE-RECOVERY-SCOPE and
  PG-VERIFY-ALL-WORKTREES (both from the stray-commit `105497f` incident, D-458,
  2026-07-15; per task instruction "these two can be one story").
- **DF-VALIDATION-001 gate:** Both gaps are in-process execution findings.
  DF-VALIDATION-001-exempt per the in-process exemption.
- **No behavioral contract required:** E-11 convention.
- **CLAUDE.md reference row:** Both new docs (`session-recovery-protocol.md` and the
  delivery-doc-currency-protocol.md amendment) MUST be reflected in CLAUDE.md Project
  References table.

## Disposition

**Status:** superseded — routed upstream 2026-07-19

Both ACs address session-boundary recovery and post-agent verification scope — these are
engine-level behaviors governing how the orchestrator inspects worktrees across all
vsdd-factory projects. The stray-commit `105497f` incident (D-458) was a wirerust execution
incident, but the protocol change (expanding verification scope to include the main develop
checkout alongside all worktrees) is an engine orchestrator discipline, not a wirerust-specific
file change.

| AC | Upstream Disposition |
|----|---------------------|
| AC-179-001 (session-recovery-protocol.md multi-location scope) | drbothen/vsdd-factory#655 evidence comment, 2026-07-19 |
| AC-179-002 (post-agent verification scope rule) | drbothen/vsdd-factory#655 evidence comment, 2026-07-19 |

This story file is retained on disk for traceability. No further wirerust delivery expected.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-18 | story-writer | Initial authorship — feature-iec104 cycle-close S-7.02: PG-STATE-RECOVERY-SCOPE + PG-VERIFY-ALL-WORKTREES (AC-179-001 session-recovery-protocol.md multi-location scope + AC-179-002 delivery-doc post-agent verification scope; stray-commit 105497f D-458 incident cited). Two gaps in one story per task instruction. |
