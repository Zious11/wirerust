---
document_type: story
story_id: STORY-119
epic_id: E-18
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-06-17T00:00:00Z
phase: f3
points: 8
priority: P2
depends_on: [STORY-120]
blocks: []
behavioral_contracts:
  - BC-2.11.013
  - BC-2.11.014
  - BC-2.11.016
  - BC-2.11.025
  - BC-2.11.026
  - BC-2.11.027
  - BC-2.11.028
  - BC-2.11.030
  - BC-2.11.031
  - BC-2.11.032
  - BC-2.11.033
  - BC-2.11.034
verification_properties: []
tdd_mode: strict
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 2
feature_id: e18-finding-collapse
github_issue: 259
wave: ~
deferred: false
deferred_reason: "F1/F2 complete; full AC/task decomposition pending F3."
# Subsystem anchor: SS-11 owns this story's scope because grouped-mode collapse
#   is a display-layer extension of reporter/terminal.rs — the core SS-11 module.
# Version 1.1 changes: F2 round-2 de-stale: struct vocabulary, full 9-BC set,
#   current anchors, deferred flag removed; full AC/task decomposition pending F3.
inputs: []
input-hash: TBD
---

# STORY-119: Terminal Finding-Collapse — Grouped Mode / --mitre

> **Status (2026-06-18, F2 round-2 de-stale):** F1/F2 are complete. The BCs that
> previously forward-referenced this story have now been authored (see Behavioral
> Contracts table below). Full AC/task decomposition will occur in F3. `depends_on`
> remains `[STORY-120]` — STORY-119 builds on the `FindingsRender` struct introduced
> by STORY-120.

This story implements grouped-mode collapse: per-tactic-bucket deduplication with
a ` (xN)` count suffix for repeated findings when `--mitre` grouped rendering is
active. The governing BCs are now authored and listed below. Full ACs and tasks
are pending F3 decomposition.

---

## Narrative (high-level scope only — ACs deferred)

- **As a** network security analyst using `wirerust analyze --mitre`
- **I want** repeated identical findings within the same MITRE tactic bucket to be
  collapsed with a ` (xN)` count suffix
- **So that** I can quickly assess the volume of repeated findings per tactic without
  wading through thousands of identical grouped-mode lines

**Scope note:** As of v0.9.0 (STORY-120), the dispatch is `match self.render.grouping {
Grouping::Grouped => render_findings_grouped(...), ... }`. For the
`{Grouping::Grouped, Collapse::Collapsed}` variant pair, collapse is not yet applied
within per-bucket groups — that is precisely what this story delivers. The
`{Grouping::Grouped, Collapse::Expanded}` variant remains suffix-free and is not
modified by this story. See BC-2.11.025 and BC-2.11.013 for the invariant clauses
that were previously forward-references and are now authored.

## Implementation Scope (non-normative sketch — full ACs in F3)

- Extend collapse into per-tactic buckets in `render_findings_grouped` (~432-483)
- Apply the same `(category, verdict, confidence, summary)` key within each bucket
- Count suffix ` (xN)` on grouped finding lines when N≥2 within a bucket
- Singleton findings within a bucket render without suffix (backward compatible)
- K=3 evidence sampling per group (same rule as STORY-118)
- Dispatch gated on `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`
  (struct form introduced by STORY-120); `{Grouping::Grouped, Collapse::Expanded}` renders
  findings individually and is unaffected by this story

## Dependencies

- `depends_on: [STORY-120]` — STORY-119 depends on STORY-120 because STORY-120
  introduces the `FindingsRender` struct (`grouping: Grouping`, `collapse: Collapse`) that
  replaces the v0.8.0 bool fields (`show_mitre_grouping`, `collapse_findings`). STORY-119's
  implementer dispatches on `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`
  rather than on the removed bool fields. The underlying collapse infrastructure
  (`CollapseKey`, `collapse_findings_pass`, `COLLAPSE_EVIDENCE_SAMPLES`) comes from
  STORY-118 (transitively available through STORY-120's dependency chain).
  STORY-119 cannot be built before STORY-120 ships.
- `blocks: []` — No downstream stories depend on STORY-119.

## Behavioral Contracts (authored — F2 complete)

All BCs governing grouped-mode collapse are now authored. The former forward-reference
clauses in BC-2.11.013 Invariant 4 and BC-2.11.025 Invariant 5 have been superseded by
the current BC versions listed below. STORY-119 is the implementing story for all of these.

| BC | Version | Role for STORY-119 |
|----|---------|-------------------|
| BC-2.11.013 | v1.14 | Core grouped-mode collapse semantics (primary) |
| BC-2.11.014 | v2.0 | Collapse key definition reused in grouped path |
| BC-2.11.016 | v1.9 | Evidence sampling (K=3) applies to grouped buckets |
| BC-2.11.025 | v1.11 | `{Grouped, Collapsed}` variant dispatch rules |
| BC-2.11.026 | v1.13 | `{Grouped, Expanded}` remains suffix-free |
| BC-2.11.027 | v1.7 | Count suffix format ` (xN)` in grouped output |
| BC-2.11.028 | v1.9 | Per-bucket first-occurrence ordering preserved |
| BC-2.11.030 | v1.1 | Backward compatibility: singleton findings in bucket |
| BC-2.11.031 | v1.2 | Color-ladder rules for suffix in grouped mode |
| BC-2.11.032 | v1.2 | Structural separation of grouped and flat paths |
| BC-2.11.033 | v1.2 | Tactic-bucket ordering interaction with collapse |
| BC-2.11.034 | v1.2 | `render_findings_grouped_collapsed` dispatch target |

Full AC-to-BC tracing will be authored in F3 decomposition.

## Acceptance Criteria

**Pending F3 full decomposition.**

F1/F2 are complete and all governing BCs are now authored (see Behavioral Contracts
table above). Full ACs — each tracing to a BC postcondition/invariant per the standard
story format — will be written during F3 story decomposition in this cycle.

## Architecture Mapping (preliminary)

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `render_findings_grouped` (extension, ~432-483) | `src/reporter/terminal.rs` | Pure core |
| `render_findings_grouped_collapsed` (new, F4-pending) | `src/reporter/terminal.rs` | Pure core |
| Per-bucket collapse pass (new) | `src/reporter/terminal.rs` | Pure core |
| `render: FindingsRender { grouping: Grouping, collapse: Collapse }` (from STORY-120) | `src/reporter/terminal.rs` | Pure data |

## Token Budget Estimate

Preliminary estimate: 8 points / 2 days. The grouped-mode path (`render_findings_grouped`
~432-483, ~51 lines) is comparable in scope to STORY-118's flat-mode collapse. A
refined per-subtask breakdown with context-window sizing will be produced in F3
decomposition once full ACs and tasks are written.

## Tasks

**Pending F3 full decomposition.**

BCs are now authored (prerequisite met). Full tasks will be written during the F3
decomposition pass in this cycle. At that point the key prerequisite — STORY-120
(enum refactor) merged and CI green — must also hold before F4 dispatch.

## Previous Story Intelligence

Predecessor: STORY-118 (flat-mode collapse). Key lessons from STORY-118 applicable here:
- Vec accumulator (not HashMap) for collapse pass
- Suffix appended before colorization
- escape_for_terminal called directly in collapse wrapper

## Architecture Compliance Rules

**Preliminary (full rules in F3 decomposition):**

- Collapse within a tactic bucket must preserve per-bucket first-occurrence order
  (BC-2.11.028)
- The `--mitre` grouped path must remain structurally separate from the flat path
  (BC-2.11.032)
- The ` (xN)` suffix on grouped findings must follow the same color-ladder as flat mode
  (BC-2.11.031)
- `{Grouping::Grouped, Collapse::Expanded}` MUST NOT emit a ` (xN)` suffix under any
  circumstances (BC-2.11.026)

## Forbidden Dependencies

- This story MUST NOT be dispatched to the F4 implementer before STORY-120 has merged
  and CI is green (the `FindingsRender` struct is a hard prerequisite).
- Any PR against `render_findings_grouped` for collapse purposes before this story
  transitions `status: draft → ready` (full ACs authored in F3) is a scope violation.
- The grouped-path implementation MUST NOT import or call the flat-mode render path
  (`render_findings_flat` or `render_findings_flat_collapsed`) — paths must remain
  structurally separate per BC-2.11.032.

## Library & Framework Requirements

Same library set as STORY-118 (no new crate dependencies expected). Exact version pins
will be confirmed from `dependency-graph.md` during F3 decomposition — do not invent
version numbers.

## File Structure Requirements

Expected files (to be confirmed in F3 decomposition):
- `src/reporter/terminal.rs` — Extend `render_findings_grouped` (~432-483); add
  `render_findings_grouped_collapsed` function
- `tests/reporter_terminal_tests.rs` — Add `mod story_119` block

## Dependency Rationale

- `depends_on: [STORY-120]` — STORY-119 (grouped-mode collapse) builds on the
  `FindingsRender` struct introduced by STORY-120 (`grouping: Grouping`, `collapse: Collapse`).
  The struct replaces the v0.8.0 `show_mitre_grouping` + `collapse_findings` bool pair;
  STORY-119's implementer dispatches on
  `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`,
  not on removed bools. STORY-120 is the direct predecessor. STORY-118's collapse
  infrastructure (`CollapseKey`, `collapse_findings_pass`, `COLLAPSE_EVIDENCE_SAMPLES`)
  is transitively available through STORY-120's dependency chain.
  Subsystem anchor: SS-11 owns this story's scope because grouped-mode collapse is a
  display-layer extension of reporter/terminal.rs — the core SS-11 module.
- `blocks: []` — No downstream stories currently depend on grouped-mode collapse.
