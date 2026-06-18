---
document_type: story
story_id: STORY-119
epic_id: E-18
version: "1.0"
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
  - BC-2.11.025
  - BC-2.11.026
verification_properties: []
tdd_mode: strict
target_module: reporter/terminal
subsystems: [SS-11]
estimated_days: 2
feature_id: e18-finding-collapse
github_issue: 259
wave: ~
deferred: true
deferred_reason: "v0.8.0 scope boundary: grouped/--mitre collapse deferred per F1 delta analysis §4 (locked design). STORY-119 exists to resolve forward-references in BC-2.11.013 Invariant 4 and BC-2.11.025 Invariant 5. Full decomposition in a future feature cycle."
# BC status: pending full BC authorship for grouped-mode collapse (future cycle).
# behavioral_contracts listed here are the BCs that forward-reference STORY-119;
# additional BCs for grouped-mode collapse will be authored at cycle start.
# tdd_mode: strict — default; will be confirmed at full decomposition time.
# Subsystem anchor: SS-11 owns this story's scope because grouped-mode collapse
#   is a display-layer extension of reporter/terminal.rs — the core SS-11 module.
# DEFERRED: Do NOT dispatch to F4 implementer. Do NOT schedule in v0.8.0 wave plan.
# This stub exists solely to anchor the BC forward-references and avoid dangling
# STORY-119 citations in BC-2.11.013 Invariant 4 and BC-2.11.025 Invariant 5.
inputs: []
input-hash: TBD
---

# STORY-119: Terminal Finding-Collapse — Grouped Mode / --mitre (DEFERRED)

**DEFERRED — NOT scheduled for v0.8.0 or v0.9.0.**

> **Note (2026-06-18, F3 fix-burst):** `depends_on` updated from `[STORY-118]` to
> `[STORY-120]`. This story's collapse work will build on STORY-120's `FindingsRender`
> enum rather than the removed `collapse_findings: bool` field from STORY-118. At full
> decomposition time, ACs should reference `FindingsRender::Grouped` (not the old bool)
> and the Architecture Mapping should reflect the enum vocabulary.

This story is a forward-reference stub only. It resolves the BC citations in
BC-2.11.013 Invariant 4 ("Collapse within grouped/`--mitre` mode is deferred to
STORY-119 (future cycle)") and BC-2.11.025 Invariant 5 ("Grouped-mode collapse is
deferred to a future cycle (see STORY-119)"). Full story decomposition will occur at
the start of the future feature cycle that targets grouped-mode collapse.

---

## Narrative (high-level scope only — ACs deferred)

- **As a** network security analyst using `wirerust analyze --mitre`
- **I want** repeated identical findings within the same MITRE tactic bucket to be
  collapsed with a ` (xN)` count suffix
- **So that** I can quickly assess the volume of repeated findings per tactic without
  wading through thousands of identical grouped-mode lines

**Scope note (v0.8.0/v0.9.0 boundary):** The `--mitre` grouped rendering path
(`render_findings_grouped` at terminal.rs:272-323) renders findings individually in
v0.8.0 and v0.9.0. In v0.9.0 (STORY-120), the dispatch becomes `match self.render {
FindingsRender::Grouped => render_findings_grouped(...), ... }` — the grouped arm
explicitly bypasses the collapse pass (BC-2.11.025 Invariant 5 / BC-2.11.013 Invariant 4).
This story will extend collapse into the grouped path in a future cycle.

## Future-Cycle Scope (non-normative sketch)

- Extend collapse into per-tactic buckets in `render_findings_grouped`
- Apply the same `(category, verdict, confidence, summary)` key within each bucket
- Count suffix ` (xN)` on grouped finding lines when N≥2 within a bucket
- Singleton findings within a bucket render without suffix (backward compatible)
- K=3 evidence sampling per group (same rule as STORY-118)
- The `FindingsRender::Grouped` variant (from STORY-120) controls grouped-mode dispatch; collapse behavior within grouped mode will be gated on this variant (replaces the old `collapse_findings: bool` field removed by STORY-120)

## Dependencies

- `depends_on: [STORY-120]` — STORY-119 depends on STORY-120 because STORY-120
  introduces the `FindingsRender` enum (`Grouped`, `FlatCollapsed`, `FlatExpanded`) that
  replaces the v0.8.0 bool fields (`show_mitre_grouping`, `collapse_findings`). STORY-119's
  implementer should build against the enum vocabulary established by STORY-120 — checking
  `FindingsRender::Grouped` in the dispatch match — rather than against the removed bool
  fields. The underlying collapse infrastructure (`CollapseKey`, `collapse_findings_pass`,
  `COLLAPSE_EVIDENCE_SAMPLES`) still comes from STORY-118 (which STORY-120 depends on
  transitively). STORY-119 cannot be built before STORY-120 ships (v0.9.0 boundary).
- `blocks: []` — No downstream stories depend on STORY-119 in v0.9.0.

## Behavioral Contracts (forward references only — not yet fully authored)

| BC | Version | Forward-Reference Clause |
|----|---------|--------------------------|
| BC-2.11.013 | v1.11 | Invariant 4: "Collapse within grouped/`--mitre` mode is deferred to STORY-119 (future cycle)" |
| BC-2.11.025 | v1.6 | Invariant 5: "Grouped-mode collapse is deferred to a future cycle (see STORY-119)" |
| BC-2.11.026 | v1.8 | Postcondition 4: "The grouped/`--mitre` path MUST NOT emit a ` (xN)` suffix on any finding, regardless of group size (BC-2.11.013 Invariant 4)" |

Additional BCs governing grouped-mode collapse behavior will be authored at full
decomposition time (future feature cycle). The above BCs confirm the STORY-119
forward-reference is resolved; they do not constitute a complete AC set.

## Acceptance Criteria

**DEFERRED — Not written for this stub.**

Full ACs will be written during the future-cycle F2 (spec evolution) and F3 (story
decomposition) phases when grouped-mode collapse is scoped. At that time, each AC will
trace to a BC postcondition per the standard story format.

## Architecture Mapping (preliminary)

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `render_findings_grouped` (extension) | `src/reporter/terminal.rs:272-323` | Pure core |
| Per-bucket collapse pass (new) | `src/reporter/terminal.rs` | Pure core |
| `render: FindingsRender` field (from STORY-120; replaces v0.8.0 `collapse_findings: bool`) | `src/reporter/terminal.rs` | Pure data |

## Token Budget Estimate

**DEFERRED.** Estimated 8 points / 2 days when fully decomposed. The grouped-mode path
is ~51 lines (`render_findings_grouped`) vs. the flat path; the extension will be
similar in complexity to STORY-118's flat-mode collapse. Full estimate at decomposition time.

## Tasks

**DEFERRED — Not written for this stub.**

Full tasks will be written during the future-cycle F3 decomposition pass. The
prerequisite is: (1) STORY-118 merged and CI green, (2) new BCs authored for
grouped-mode collapse behavior (at minimum: grouped-mode collapse key semantics,
per-bucket count suffix rules, tactic-bucket ordering interaction).

## Previous Story Intelligence

**N/A — DEFERRED stub. See STORY-118 for predecessor context.**

When this story is activated, the predecessor is STORY-118. Key lessons from STORY-118:
- Vec accumulator (not HashMap) for collapse pass
- Suffix appended before colorization
- escape_for_terminal called directly in collapse wrapper

## Architecture Compliance Rules

**DEFERRED — Not written for this stub.**

At full decomposition time, key rules will include:
- Collapse within a tactic bucket must preserve per-bucket first-occurrence order
- The `--mitre` grouped path must remain structurally separate from the flat path
- The ` (xN)` suffix on grouped findings must follow the same color-ladder as flat mode

## Forbidden Dependencies

- This stub MUST NOT be dispatched to the F4 implementer in v0.8.0.
- Any PR against `render_findings_grouped` for collapse purposes before this story is
  formally activated (status: draft → ready) is a scope violation.

## Library & Framework Requirements

**DEFERRED.** Same library set as STORY-118 (no new dependencies expected).

## File Structure Requirements

**DEFERRED.** Expected files at decomposition time:
- `src/reporter/terminal.rs` — Modify `render_findings_grouped` (terminal.rs:272-323)
- `tests/reporter_terminal_tests.rs` — Add `mod story_119` block

## Dependency Rationale

- `depends_on: [STORY-120]` — STORY-119 (grouped-mode collapse) builds on the
  `FindingsRender` enum introduced by STORY-120 (v0.9.0). The enum replaces the
  v0.8.0 `show_mitre_grouping` + `collapse_findings` bool pair; STORY-119's implementer
  must dispatch on `FindingsRender::Grouped` (the enum variant), not on the removed bool.
  STORY-120 is the direct predecessor. STORY-118's collapse infrastructure (`CollapseKey`,
  `collapse_findings_pass`, `COLLAPSE_EVIDENCE_SAMPLES`) is transitively available through
  STORY-120's dependency chain.
- `blocks: []` — STORY-119 is a future-cycle tail. No v0.9.0 or earlier stories
  depend on grouped-mode collapse.
