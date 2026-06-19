---
document_type: cycle-manifest
cycle_id: feature-story-119-grouped-collapse
cycle_type: feature
version: v0.9.0
status: CLOSED
started: 2026-06-18T00:00:00Z
completed: 2026-06-19T00:00:00Z
producer: orchestrator
---

# Cycle Manifest: feature-story-119-grouped-collapse (Feature-Mode) — CLOSED

## Scope

**STORY-119 — grouped-mode finding-collapse (E-18 / issue #259 tail)**

Adds collapse behaviour to `--mitre` grouped mode. Bundled with STORY-120
(FindingsRender enum→struct migration) and STORY-122 (struct reshape slice-A).

Type design decision (D-110): reshape `FindingsRender` from a 3-variant enum
into a struct of two orthogonal enums (`struct FindingsRender { grouping:
Grouping, collapse: Collapse }`). CLI change: `--mitre` collapses by default;
`--no-collapse` suppresses collapse in both grouped and flat modes.

F1 delta-analysis artifact: `.factory/phase-f1-delta-analysis/story-119-grouped-mode-collapse-delta-analysis.md`
Research backing type-design decision: `.factory/research/story-119-render-mode-typedesign.md`

## Phase Progress (CLOSED)

| Phase | Status | Date |
|-------|--------|------|
| F1 — Delta-Analysis | **COMPLETE** | 2026-06-18 |
| F2 — Spec-Evolution | **CONVERGED 3/3** (7eb9f09; 6 rounds) | 2026-06-18 |
| F3 — Incremental Stories (resplit D-120) | **CONVERGED 3/3** (8fa9ff9; 5 rounds) | 2026-06-19 |
| F4 — Delta-Implementation | **COMPLETE** (PRs #268/#269) | 2026-06-19 |
| F5 — Scoped-Adversarial | **CONVERGED 3/3** (adcf4e9; 2 rounds) | 2026-06-19 |
| F6 — Targeted-Hardening | **HARDENED** (mutation 85%, regression ~1700/0) | 2026-06-19 |
| F7 — Delta-Convergence | **CONVERGED 3/3** (1c89b52; 5 rounds) | 2026-06-19 |
| Release v0.9.0 | **RELEASED** (main 986e148; tag v0.9.0; 4 binaries) | 2026-06-19 |
| Release v0.9.1 | **RELEASED** (main ad4eec8; tag v0.9.1; doc patch) | 2026-06-19 |
| Release v0.9.2 | **RELEASED** (main b73b242; tag v0.9.2 obj a298dbe; DNP3 determinism patch) | 2026-06-19 |

## Delivered

| Metric | Value |
|--------|-------|
| Stories delivered | STORY-120 (PR #266/267), STORY-122 (PR #268), STORY-119/B (PR #269) |
| BCs created | 5 new BCs (BC-2.11.030–034); SS-11 29→34; total 293 (+ revisions) |
| VPs created | None new (F6 mutation guard; VP-012 proptest extended) |
| Holdout scenarios | None new (existing holdouts gated) |
| Adversarial passes (F2) | 6 rounds (3/3 SATISFIED; 7eb9f09) |
| Adversarial passes (F3) | 5 rounds (3/3 SATISFIED; 8fa9ff9) |
| Adversarial passes (F5) | 2 rounds (3/3 SATISFIED; adcf4e9) |
| Adversarial passes (F7) | 5 rounds (3/3 SATISFIED; 1c89b52) |
| Final holdout satisfaction | N/A (F4-level per-story 3/3 CLEAN; no new F4 wave) |
| Release versions | v0.9.0 + v0.9.1 + v0.9.2 |

## Key Decisions

| Decision | Summary |
|----------|---------|
| D-110 | TYPE DESIGN: FindingsRender struct-of-orthogonal-enums; --mitre collapses by default; --no-collapse dual-scope |
| D-111..D-118 | F2 spec-evolution: 5 new BCs; 3 residuals→F3; gate SATISFIED (D-118) |
| D-119..D-122 | F3 decomposition: monolithic→D-120 resplit; STORY-122/A + STORY-119/B; F3 gate APPROVED (D-122) |
| D-123..D-125 | F4 delivery: STORY-122 (PR #268, 8696448); STORY-119/B (PR #269, 181d5e2) |
| D-126..D-128 | F5 scoped-adversarial: Round-1 3 findings → Round-2 3/3 CLEAN |
| D-129..D-132 | F7 convergence: Rounds 1-5; R5 CONVERGED 3/3 on 1c89b52 |
| D-133 | v0.9.0 RELEASED; E-18 cycle CLOSED; DNS-TUNNELING-COVERAGE-001 filed |
| D-134 | v0.9.1 RELEASED (doc/help patch; PRs #277/#278) |
| D-135 | v0.9.2 RELEASED (DNP3 determinism fix; PRs #279/#280; BUG-DNP3-CONTROL-OP-DETERMINISM-001 CLOSED) |

Detail for D-131..D-135: `decisions-archive.md` in this cycle directory.

## Open Items Filed From This Cycle

- **DNS-TUNNELING-COVERAGE-001** (OPEN INFORMATIONAL): dns-tunnel-iodine.pcap fixtures ready; human decision pending on feature scope.
- **STORY-121** (OPEN DRAFT): E-11 process-gap codification; D-127 orchestrator relay-trust; post-fixburst consuming-artifact sweep; narrow-leak-checker CI gate. Human decision on scope pending.
- **pcapng reader support**: newly motivated (large TLS captures only available as pcapng). Candidate feature.

## Lessons

See `lessons.md` in this directory.

## Session Checkpoints

Archived checkpoints in `session-checkpoints.md`.
