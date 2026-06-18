---
document_type: cycle-manifest
cycle_id: feature-story-119-grouped-collapse
cycle_type: feature
version: v0.9.0-bundled
status: in-progress
started: 2026-06-18T00:00:00Z
completed: null
producer: orchestrator
---

# Cycle Manifest: feature-story-119-grouped-collapse (Feature-Mode)

## Scope

**STORY-119 — grouped-mode finding-collapse (E-18 / issue #259 tail)**

Adds collapse behaviour to `--mitre` grouped mode. Depends on STORY-120
(FindingsRender enum migration, merged develop a4263c73 via PR #266). Ships
bundled with the unreleased Cargo 0.9.0 develop line — no separate release tag
until the human gate authorizes.

Type design decision (D-110): reshape `FindingsRender` from a 3-variant enum
into a struct of two orthogonal enums (`struct FindingsRender { grouping:
Grouping, collapse: Collapse }`). CLI change: `--mitre` collapses by default;
`--no-collapse` suppresses collapse in both grouped and flat modes.

F1 delta-analysis artifact: `.factory/phase-f1-delta-analysis/story-119-grouped-mode-collapse-delta-analysis.md`
Research backing type-design decision: `.factory/research/story-119-render-mode-typedesign.md`

## Phase Progress

| Phase | Status | Date |
|-------|--------|------|
| F1 — Delta-Analysis | **COMPLETE + gate-approved** | 2026-06-18 |
| F2 — Spec-Evolution | IN PROGRESS | — |
| F3 — Incremental Stories | PENDING | — |
| F4 — Delta-Implementation | PENDING | — |
| F5 — Scoped-Adversarial | PENDING | — |
| F6 — Targeted-Hardening | PENDING | — |
| F7 — Delta-Convergence | PENDING | — |

## Delivered (to be updated)

| Metric | Value |
|--------|-------|
| Stories delivered | STORY-119 |
| BCs created | TBD (F2) |
| VPs created | TBD |
| Holdout scenarios | TBD |
| Total cost | TBD |
| Adversarial passes | 0 (F1 only so far) |
| Final holdout satisfaction | TBD |
| Release version | v0.9.0 (bundled — HELD pending human gate) |

## Key Decisions

| Decision | Summary |
|----------|---------|
| D-110 | TYPE DESIGN: FindingsRender reshapen to struct-of-orthogonal-enums; --mitre collapses by default; --no-collapse dual-scope; no release yet |

## Notes

- v0.9.0 release is HELD per D-109 (human-deferred). STORY-119 bundles into the
  unreleased develop line.
- DRIFT-62-MAIN495-DOC-001 to be fixed on develop within this cycle (D-109).
- STORY-121 (E-11 process-gap) remains filed as draft; no action needed during
  STORY-119 F1.
</content>
</invoke>