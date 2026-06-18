---
document_type: story
story_id: STORY-121
epic_id: E-11
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-06-18T00:00:00Z
phase: 3
inputs: []
input-hash: "d41d8cd"
# BC status: pending PO authorship — behavioral_contracts is empty; story must remain
# status: draft until PO authors and anchors canonical BC-S.SS.NNN contracts for this story.
traces_to:
  - .factory/STATE.md
  - .factory/phase-f1-delta-analysis/issue-62-terminal-reporter-enum-modes-delta-analysis.md
points: 3
depends_on: []
blocks: []
behavioral_contracts: []
verification_properties: []
priority: P2
cycle: process-improvement
wave: ~
target_module: .factory/process
subsystems: []
estimated_days: 1
tdd_mode: strict
dispositions:
  - D-099
  - D-100
  - D-101
---

# STORY-121: F1/F2 Story-Input Analysis Docs — Mandatory Numeric Self-Audit + Consuming-Surface Sweep Checklist

> **Disposition target:** D-099 / D-100 / D-101 (STATE.md)
>
> During E-8 / issue #62 F3 story-decomposition, convergence took 10 fix rounds —
> the cycle's most-churned phase. Root cause: the F1 delta-analysis and F2
> spec-evolution docs were declared as STORY-120 `inputs:`. Every consuming-artifact
> fix re-triggered the story's input-hash recompute, which re-entered the fresh-context
> adversarial loop. Stale sub-counts and version-stamps surfaced serially, one
> consuming surface per round (story body, frontmatter comment, dep-graph matrix,
> dep-graph version, F1 headline count, F1 sub-counts, BC sibling wiring, AC
> paraphrase blind-spots). This story codifies the two corrective procedures as
> durable process checkpoints.

## Narrative

- **As a** phase orchestrator or spec steward preparing an F3 story-decomposition dispatch
- **I want** (1) a mandatory numeric self-audit gate for any F1/F2 phase analysis doc
  declared as a story `inputs:` field, and (2) a single post-fix-burst consuming-surface
  sweep checklist covering every artifact surface that must stay in sync after a BC bump
  or fix-burst
- **So that** F3 story-decomposition convergence does not degrade into serial one-finding-
  per-round churn caused by stale counts and version-stamps re-surfacing through the
  input-hash loop, and the cycle's most-churned phase (10 rounds in E-8 / issue #62)
  is not repeated

## Behavioral Contracts

_No BCs authored yet. Status must remain `draft` until PO anchors canonical
BC-S.SS.NNN contracts covering the audit gate and checklist behavior. See frontmatter
comment above._

## Acceptance Criteria

### AC-001 — F1/F2 numeric self-audit gate at authoring time
_(traces to: pending BC authorship — D-099/D-101 codification)_

Before any F1 delta-analysis or F2 spec-evolution document is declared in a story's
`inputs:` frontmatter field, the author MUST perform an exhaustive numeric self-audit:
every count, sub-count, line-list, and construction-site total cited in the document
MUST be reconciled against `grep` ground-truth over the actual source tree. The
reconciliation result (expected vs. actual for each numeric claim) MUST be recorded
in a `## Numeric Self-Audit` section within the F1/F2 document itself before the
document is handed to the story-writer.

**Gate enforcement:** If a `## Numeric Self-Audit` section is absent from an F1/F2
doc at the time it is listed in `inputs:`, the story-writer MUST flag the omission
and refuse to finalize the story's input-hash until the section is present and all
discrepancies resolved.

### AC-002 — Post-fix-burst consuming-surface sweep checklist codified
_(traces to: pending BC authorship — D-099/D-100/D-101 codification)_

After any fix-burst that (a) bumps a BC version, (b) corrects a behavioral contract
wording, or (c) revises a normative count or construction-site list in any spec artifact,
the orchestrator MUST execute the following consuming-surface sweep — in a single atomic
burst — before re-entering adversarial review:

1. **BC files** — every BC touched by the fix-burst: confirm PC/invariant/Architecture-
   Anchor wording is consistent across the full BC file (not just the amended clause).
2. **BC-INDEX** — confirm the BC's version stamp and title row reflect the new version.
3. **spec-changelog** — confirm a changelog entry was added for the BC version bump.
4. **Consuming-story body** — confirm all BC-table version cells in the story body
   match the new BC versions.
5. **Consuming-story frontmatter** — confirm the `# BC status:` comment version
   annotation (if present) matches.
6. **Consuming-story version-table** — confirm any token-budget BC-count or version
   subtable is updated.
7. **dep-graph matrix** — confirm the BC-to-Stories matrix rows carry the correct
   current BC version stamps.
8. **F1/F2 phase analysis docs** — if the bumped BC appears in F1/F2 docs that are
   declared as story `inputs:`, re-run the numeric self-audit (AC-001) to confirm no
   stale count references to the old BC content.

The checklist MUST be applied exhaustively: every BC bumped in the burst, every
consuming-story that lists that BC in its `behavioral_contracts:` or body BC table,
every cross-reference in dep-graph and phase-analysis docs.

### AC-003 — Checklist codified in .factory/policies.yaml
_(traces to: pending BC authorship — D-099/D-100/D-101 codification)_

The two procedures from AC-001 and AC-002 are codified as a named policy entry in
`.factory/policies.yaml`:

- **ID:** `F3-CONVERGENCE-002`
- **Title:** "F1/F2 Story-Input Numeric Self-Audit + Post-Fix-Burst Consuming-Surface Sweep"
- **Trigger:** (a) Any F1/F2 phase analysis document is added to a story's `inputs:` field;
  (b) Any BC version bump or fix-burst during F3 story-decomposition adversarial convergence.
- **Mandatory actions:** The numeric self-audit gate (AC-001) and the 8-step consuming-surface
  sweep (AC-002).
- **References:** `D-099`, `D-100`, `D-101`, `STORY-121`, `STORY-120` (exemplar cycle).
- **Rationale:** E-8 / issue #62 F3 required 10 convergence rounds — the cycle's most-churned
  phase. Root cause documented in STATE.md D-099/D-100/D-101.

### AC-004 — F3 dispatch template updated
_(traces to: pending BC authorship — D-099/D-101 codification)_

The F3 story-decomposition dispatch template (or the orchestrator F3 runbook section in
`.factory/` / VSDD workflow docs) includes a pre-dispatch checklist item:
"If any F1/F2 analysis doc will be listed in a story's `inputs:`, verify a
`## Numeric Self-Audit` section is present and CLEAN before dispatching."

### AC-005 — Exemplar documented
_(traces to: pending BC authorship — D-101 codification)_

The policy entry (AC-003) or a companion note in `.factory/process/` references the
E-8 / issue #62 cycle as the motivating exemplar, with a pointer to STATE.md D-099
through D-102 for the full round-by-round churn log, so future orchestrators can
understand the failure mode concretely rather than abstractly.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| F3-CONVERGENCE-002 policy entry | `.factory/policies.yaml` | Configuration (no code) |
| F3 dispatch template checklist item | `.factory/` orchestrator runbook / VSDD workflow | Documentation (no code) |
| `## Numeric Self-Audit` section in F1/F2 docs | F1/F2 analysis docs at authoring time | Documentation (process gate) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | F1/F2 doc has a `## Numeric Self-Audit` section but it shows unresolved discrepancies | Story-writer must halt and request resolution; partial audit is not sufficient |
| EC-002 | A fix-burst corrects only prose wording (no numeric claim changed, no BC version bumped) | Consuming-surface sweep is still RECOMMENDED but not MANDATORY for that burst |
| EC-003 | A BC is bumped that is NOT referenced in any active story's `inputs:` | Steps 1–7 of AC-002 apply; step 8 (F1/F2 re-audit) is skipped as no story has the BC as an input |
| EC-004 | Multiple BCs are bumped in a single fix-burst | The AC-002 checklist applies to each bumped BC; a single sweep pass covering all BC-bump consequences at once satisfies the "single atomic burst" requirement |
| EC-005 | The consuming story has already been delivered (status: completed) | Stale version-stamps in completed stories are recorded as ACCEPTED DRIFT in STATE.md (see D-088 precedent); the sweep applies only to in-flight (draft/ready) stories |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| `.factory/policies.yaml` (insertion target) | ~2,000 |
| STATE.md D-099/D-100/D-101/D-102 excerpt | ~2,000 |
| F3 dispatch template / orchestrator runbook (if exists) | ~1,500 |
| **Total** | **~8,000** |
| Agent context window | 200K (Sonnet) |
| **Budget usage** | **~4%** |

## Tasks (MANDATORY)

1. [ ] Read `.factory/policies.yaml` to identify the next available policy ID in the
       `F3-CONVERGENCE` namespace (or confirm `F3-CONVERGENCE-002` is available)
2. [ ] Add policy entry `F3-CONVERGENCE-002` to `.factory/policies.yaml` per AC-003,
       referencing D-099/D-100/D-101/STORY-121/STORY-120
3. [ ] Locate the F3 story-decomposition dispatch template or orchestrator F3 runbook
       section (in `.factory/` or VSDD workflow docs) and add the pre-dispatch checklist
       item per AC-004; if no template file exists, create a minimal
       `.factory/process/f3-dispatch-checklist.md` documenting the two new gates
4. [ ] Add a `## Numeric Self-Audit` section template to the F1/F2 doc author guidance
       (either a process note in the policy entry or a new `.factory/process/f1-f2-authoring-guide.md`)
       that specifies the required section format and reconciliation table structure
5. [ ] Verify the policy entry references all required IDs: D-099, D-100, D-101, STORY-121, STORY-120
6. [ ] Record this story's completion in STATE.md under the D-099/D-100/D-101 disposition block

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-091 (E-11 anchor tooling) | Python 3 stdlib-only; policy entry in `.factory/policies.yaml` pattern established for `ANCHOR-VALIDATION-001` | `ANCHOR-VALIDATION-001` policy entry format: ID / policy text / references block | Process-gap stories have no BCs at authoring time — status must remain `draft` per Spec-First Gate S-7.01 |

E-8 / issue #62 exemplar: 10 adversarial rounds for STORY-120 (most-churned phase in
this cycle). The F1 delta-analysis was listed in STORY-120's `inputs:` field
(`input-hash` = `3d76a93`). Every edit to the F1 doc re-triggered an input-hash
recompute, which re-entered the adversarial convergence loop with fresh context.
Stale items surfaced one-per-round: round-7 fixed the F1 headline count (35→28),
round-8 fixed a second F1 occurrence (OQ-3), round-9 fixed three F1 sub-counts
(§6/§2/§10). The BC version-stamp drift (rounds 5–6) was a parallel consuming-surface
problem of the same class. Both classes are addressed by this story's AC-001 + AC-002.

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| Policy entry format must match existing `ANCHOR-VALIDATION-001` entry style | STORY-091 pattern | Read `.factory/policies.yaml` before writing the new entry |
| `behavioral_contracts: []` with `status: draft` — story must NOT be promoted to `ready` without PO-authored BCs | Spec-First Gate S-7.01 | Frontmatter `status: draft` + `# BC status:` comment |
| No new Rust production code — this story delivers only policy/process artifacts | E-11 tooling-only boundary | No modifications to `src/` tree |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Python 3 | ≥ 3.8 (matches existing `bin/` tooling) | Only if a companion process-validation script is added |
| `.factory/policies.yaml` | current schema | Policy registration target |

_No new library dependencies — this story delivers process documentation and policy
entries only._

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `.factory/policies.yaml` | **modify** | Add `F3-CONVERGENCE-002` policy entry (AC-003) |
| `.factory/process/f3-dispatch-checklist.md` | **create** (if no F3 template exists) | Pre-dispatch checklist with numeric self-audit gate and consuming-surface sweep (AC-004) |
| `.factory/process/f1-f2-authoring-guide.md` | **create** (if no authoring guide exists) | `## Numeric Self-Audit` section template and reconciliation table format (AC-004) |
