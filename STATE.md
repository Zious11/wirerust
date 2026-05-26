---
pipeline: PHASE_3_TDD_IMPLEMENTATION
phase: phase-3-tdd-implementation
product: wirerust
mode: brownfield
timestamp: 2026-05-22T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_started: "2026-05-21"
develop_head: e237747
wave_1_closed: "2026-05-22"
wave_2_closed: "2026-05-22"
wave_3_closed: "2026-05-22"
wave_4_closed: "2026-05-22"
wave_5_closed: "2026-05-22"
wave_5_status: closed
wave_5_wave_level_convergence: "3/3 clean fresh-context passes (all VERDICT: CLEAN; only 2 non-blocking cosmetic Nits)"
wave_6_closed: "2026-05-22"
wave_6_status: closed
wave_6_per_story_delivery: complete
wave_6_wave_level_convergence: "3/3 clean fresh-context passes (all VERDICT: CLEAN; ZERO findings of any severity across all three passes)"
wave_7_closed: "2026-05-25"
wave_7_status: closed
wave_7_per_story_delivery: complete
wave_7_per_story_convergence: "8 passes; 3/3 clean streak on passes 6/7/8"
wave_7_wave_level_convergence: "8 passes; 3/3 clean streak on passes 6/7/8"
wave_8_closed: "2026-05-26"
wave_8_status: closed
wave_8_per_story_delivery: complete
wave_8_per_story_convergence: "STORY-019: 8 passes; 3/3 clean streak on passes 6/7/8 (14 findings remediated); STORY-015: 8 passes; 3/3 clean streak on passes 6/7/8 (14 findings remediated)"
wave_8_wave_level_convergence: "9 passes; 3/3 clean streak on passes 7/8/9 (12 findings remediated; 3 develop PRs + 4 factory BC commits)"
wave_9_status: closed
wave_9_started: "2026-05-26"
wave_9_closed: "2026-05-26"
wave_9_stories: STORY-016 + STORY-020
wave_9_pr_count: 4
wave_9_prs: "#127, #128, #129, #130"
wave_9_per_story_convergence: "STORY-016: 6 passes (DIRTY×3 + CLEAN×3); STORY-020: 8 passes (DIRTY×5 + CLEAN×3)"
wave_9_wave_level_convergence: "6 passes (DIRTY×3 + CLEAN×3; passes 4/5/6 clean)"
wave_10_status: in_progress
wave_10_started: "2026-05-26"
wave_10_stories: STORY-017 + STORY-018
wave_10_notes: "Parallel-eligible (disjoint src line ranges; both touch tests/reassembly_engine_tests.rs — STORY-018 also touches reassembly_segment_tests.rs). Will require rebase on second PR if conflicts. New policies ACTIVE: DF-SIBLING-SWEEP-001 + DF-PR-MANAGER-COMPLETE-001 (validating whether codifications reduce Wave 9 recurrence patterns)."
current_wave: 10
stories_delivered: 18
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0→0→0|W7-story:8ps-3clean|W7-wave:8ps-3clean|W8-S019-story:8ps-3clean(14rem)|W8-S015-story:8ps-3clean(14rem)|W8-wave:9ps-3clean(12rem)|W9-S016-story:6ps-3clean(24rem)|W9-S020-story:8ps-3clean(13rem)|W9-wave:6ps-3clean(11rem)"
consistency_audit: CONSISTENT
input_drift_check: CLEAN
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-9 CLOSED/CONVERGED; Wave 10 IN PROGRESS.
18 stories delivered across Waves 1-9 (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020).
Wave 9 CLOSED 2026-05-26: STORY-016 (PR #127 → d636285, overlap detection, 24 tests + 1 proptest) +
STORY-020 (PR #128 → 8cb907e, memory mgmt, 25 tests + 1 proptest + 1 additive #[doc(hidden)] seam).
Wave-level remediation: PR #129 → 2037f88 (F-W9P1-001 + F-W9P1-004) + PR #130 → e237747 (F-W9P2-001 + F-W9P2-003).
Wave-level convergence: 6 passes (DIRTY×3 + CLEAN×3; passes 4/5/6 clean; 11 findings remediated).
Wave 10 DISPATCHED 2026-05-26: STORY-017 (conflict + evasion detection; BCs 2.04.018/019/020/021/022/037) +
STORY-018 (resource bounds; BCs 2.04.023/027/040/041/042/044/045/046); parallel worktrees active.
New policies ACTIVE: DF-SIBLING-SWEEP-001 + DF-PR-MANAGER-COMPLETE-001.
develop HEAD: e237747 (PR #130 — wave-followup-2 merged 2026-05-26).

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** passing on develop (Wave 8 stories delivered). `cargo fmt --check`,
`cargo clippy`, `cargo test --all-targets` all green. CI: 7 checks including `fuzz-build` job
(pinned `nightly-2026-05-21` + `cargo-fuzz 0.13.1` + `timeout-minutes: 25` after PR #111 hotfix;
the nightly pin is a deliberate periodic-maintenance item — do NOT enable automated
dependency bumping for it).

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements; 33 adversary passes; trajectory: `17→…→0→0→0` (detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md) |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 48 stories / 10 epics / 27 waves / 100 holdout scenarios / 282 points; story-adversary 3/3 (10 passes) SATISFIED; input-hash drift CLEAN (153/153) |
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-9 CLOSED/CONVERGED; 18 stories delivered; Wave 10 IN PROGRESS (STORY-017 + STORY-018 dispatched 2026-05-26) | Wave-level convergence detail: cycles/phase-3-tdd/convergence-trajectory.md |
| Phase 4 — Holdout Evaluation | NOT STARTED | — |
| Phase 5 — Adversarial Refinement | NOT STARTED | — |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 3 — Current Wave Status

| Wave | Stories | Status | develop HEAD at Close | Notes |
|------|---------|--------|----------------------|-------|
| 1 | STORY-001, STORY-069 | CLOSED/CONVERGED | b7424b7 | 329 tests |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | CLOSED/CONVERGED | 3b2481c | 376 tests; fuzz-build CI |
| 3 | STORY-071, STORY-005 | CLOSED/CONVERGED | f0b5007 | CI hotfix #112; chore #115 |
| 4 | STORY-011, STORY-066 | CLOSED/CONVERGED | f628c33 | 394 tests |
| 5 | STORY-012 | **CLOSED/CONVERGED** | bbddac6 | 415 tests; 3/3 clean wave-level passes |
| 6 | STORY-013 | **CLOSED/CONVERGED** | 3e705b5 | PR #119 squash-merged 2026-05-22; 31 BC tests; per-story 3/3 clean; wave-level 3/3 CLEAN (ZERO findings) |
| 7 | STORY-014 | **CLOSED/CONVERGED** | b23c6d3 | PR #120 squash-merged 2026-05-25; 17 tests + 2 doc(hidden) seams; ADR-0004 amended PR #121; per-story 8 passes 3/3 clean streak; wave-level 8 passes 3/3 clean streak |
| 8 | STORY-019, STORY-015 | **CLOSED/CONVERGED** | 4b9b85f | PR #122 (STORY-019) + PR #123 (STORY-015) squash-merged 2026-05-26; ADR-0004 v2 PRs #124/#125/#126; per-story 8 passes each (3/3 clean); wave-level 9 passes 3/3 clean streak; 4 drift items logged |
| 9 | STORY-016, STORY-020 | **CLOSED/CONVERGED** 2026-05-26 | e237747 | PR #127 (STORY-016, 24 tests+1 proptest) + PR #128 (STORY-020, 25 tests+1 proptest+1 seam) + PR #129 + PR #130 (wave-followup-1/2); per-story 14 passes total (S016: 6; S020: 8); wave-level 6 passes (DIRTY×3+CLEAN×3); 11 findings remediated; W9-D8 CRITICAL; 632 tests passing |
| 10–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 9 — wave-level adversarial passes 1–3 (DIRTY) | **REMEDIATED** 2026-05-26 | P1: 5 findings (4 MED rem. + W9-D12 deferred) → PRs #129/#130 + factory commits a3e8927/f5330a4/152acbb. P2: 3 MED sibling-regressions all rem. P3: 3 MED sibling-regressions all rem as trivial text edits (no new PR needed; BC-2.04.016 v1.4 + STORY-020 v1.9). W9-D8 escalated to CRITICAL after P3. |
| Wave 9 — wave-level adversarial passes 4/5/6 (CLEAN) | **CONVERGED** 2026-05-26 | Passes 4/5/6 all VERDICT: CLEAN — zero findings. 3-clean streak satisfied (BC-5.39.001). Wave-level CONVERGED. Total wave-level: 6 passes (DIRTY×3 + CLEAN×3), 11 findings remediated. |
| Wave-gate — Wave 9 | **CLOSED** 2026-05-26 | develop HEAD e237747; 18 stories total Waves 1-9; 632 tests passing; STORY-017 + STORY-018 unblocked; Wave 10 READY TO DISPATCH. |
| Wave 9 → Wave 10 — W9-D8 CODIFIED | **COMPLETE** 2026-05-26 | W9-D8 codified as DF-SIBLING-SWEEP-001 in .factory/policies.yaml. Orchestrator policy-rubric injection now extended to all remediation dispatches. Wave 10 dispatch unblocked. W9.L3 (pr-manager merge-step gap) remains OPEN — separate codification target. |
| Wave 9 → Wave 10 — W9.L3 CODIFIED | **COMPLETE** 2026-05-26 | W9.L3 codified as DF-PR-MANAGER-COMPLETE-001 in .factory/policies.yaml. pr-manager merge-step completion now enforced policy; orchestrator must inject policy with concrete gh CLI template into every pr-manager dispatch. Both pre-Wave-10 codification targets resolved. |
| Wave 10 IN PROGRESS | **IN PROGRESS** 2026-05-26 | Wave 10 opened 2026-05-26; STORY-017 + STORY-018 dispatched in parallel; new policies DF-SIBLING-SWEEP-001 + DF-PR-MANAGER-COMPLETE-001 will be injected at remediation + pr-manager dispatches respectively. |

## Spec Package Summary (Phase 1 — PASSED)

| Artifact | Location | Count |
|----------|----------|-------|
| L2 Domain Specification | `.factory/specs/domain/` | 20 shards |
| L3 PRD | `.factory/specs/prd.md` | 1 file |
| Behavioral Contracts | `.factory/specs/behavioral-contracts/ss-01..ss-13/` | 217 BCs across 12 subsystems |
| BC Index | `.factory/specs/behavioral-contracts/BC-INDEX.md` | 1 file |
| Architecture Package | `.factory/specs/architecture/` | 9 files + ARCH-INDEX.md |
| Module Criticality | `.factory/specs/module-criticality.md` | 1 file |
| DTU Assessment | `.factory/specs/dtu-assessment.md` | DTU_REQUIRED: false |
| Verification Properties | `.factory/specs/verification-properties/vp-001..vp-020` | 20 VPs + VP-INDEX.md |
| PRD Supplements | `.factory/specs/prd-supplements/` | 4 files |

Full Phase 1 convergence detail: `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`

## Session Resume Checkpoint (2026-05-26 — Wave 9 CLOSED)

1. Waves 1-9 all CLOSED/CONVERGED — 18 stories delivered.
   STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020 all merged to develop.
2. Wave 9 CLOSED: STORY-016 (PR #127 → d636285, 24 tests + 1 proptest, overlap detection) +
   STORY-020 (PR #128 → 8cb907e, 25 tests + 1 proptest + 1 additive #[doc(hidden)] seam, memory mgmt).
   Wave-level remediation: PR #129 → 2037f88 (F-W9P1-001 + F-W9P1-004) + PR #130 → e237747
   (F-W9P2-001 + F-W9P2-003). develop HEAD: e237747.
   Per-story passes: STORY-016 6 passes (DIRTY×3 + CLEAN×3); STORY-020 8 passes (DIRTY×5 + CLEAN×3).
   Wave-level passes: 6 total (DIRTY×3 + CLEAN×3; passes 4/5/6 clean; 11 findings remediated).
   Test suite: 632 tests passing on develop.
3. Wave 10 READY TO DISPATCH: sprint-state.yaml current_wave=10.
   STORY-017: status=pending (deps STORY-015 done W8 + STORY-016 done W9).
   STORY-018: status=pending (same deps).
4. Drift items W9-D1..W9-D8 + W9-D12 logged — see Drift Items table. All require
   research-agent validation per DF-VALIDATION-001 before any GitHub issue is filed.
   W9-D8 RESOLVED: codified as DF-SIBLING-SWEEP-001 in .factory/policies.yaml (2026-05-26,
   pre-Wave-10). W9-D9: RESOLVED (F-W9P1-003; factory commit a3e8927).
5. NEXT: Dispatch Wave 10 (STORY-017 + STORY-018). Both pre-Wave-10 codification targets resolved:
   W9-D8 → DF-SIBLING-SWEEP-001 (2026-05-26) and W9.L3 → DF-PR-MANAGER-COMPLETE-001 (2026-05-26).
   Orchestrator to inject DF-SIBLING-SWEEP-001 checklist into all remediation dispatches AND
   DF-PR-MANAGER-COMPLETE-001 (with concrete gh pr merge command template) into every pr-manager dispatch.

## Wave Retrospectives

### Wave 9 Retrospective (closed 2026-05-26)

**Stories:** STORY-016 (overlap detection, BCs 2.04.035/036/038/043/047): per-story 6 passes
(3 DIRTY + 3 CLEAN). STORY-020 (memory mgmt, BCs 2.04.014-017): per-story 8 passes (5 DIRTY + 3 CLEAN).

**PRs:** 4 merged (#127, #128, #129, #130) — 2 story PRs + 2 wave-level remediation PRs.

**Adversarial cost:** Per-story 14 passes total (S016: 6; S020: 8). Wave-level 6 passes (3 DIRTY + 3
CLEAN). Cumulative passes Wave 9: 20.

**Findings remediated:** Per-story ~37 (24 STORY-016 + 13 STORY-020). Wave-level 11 (5+3+3 across
passes 1/2/3).

**Sibling-discipline pattern (W9-D8):** Fired 6 consecutive times (STORY-020 per-story P2/P3/P4 +
wave-level P1/P2/P3). Broke at wave-level pass 4 (trivial text-only fixes). W9-D8 RESOLVED:
codified as DF-SIBLING-SWEEP-001 in .factory/policies.yaml (2026-05-26, pre-Wave-10).

**Brownfield-formalization integrity:** Zero production behavior changes. 1 additive
`#[doc(hidden)] pub fn flows_memory_sum_for_testing` seam (STORY-020 AC-004; opt-in per ADR-0004 v2).

**Notable artifacts:** BC-2.04.015 v1.5 PC-7 + BC-2.04.016 v1.4 PC-5 (data-loss-on-MemoryPressure
semantics); AC-013 PATH 1/PATH 2 bifurcation; AC-012 covers all 4 non-Established states.

**Active drift items:** W9-D5 (LOW), W9-D8 (RESOLVED — DF-SIBLING-SWEEP-001 codified 2026-05-26), W9-D11 (LOW), W9-D12 (LOW pending intent),
W9-D1..D4 (LOW story template gaps), W9-D6/D7 (LOW line-citation). W9-D9: RESOLVED.
W9.L3: RESOLVED — codified as DF-PR-MANAGER-COMPLETE-001 in .factory/policies.yaml (2026-05-26, pre-Wave-10).

Full retrospective detail: `.factory/cycles/phase-3-tdd/lessons.md` (W9 lessons)

## Decisions Log

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-001 | Brownfield mode (target == reference) | 2026-05-19 | No parallel reference repo; in-repo formalization only |
| D-002 | DTU not required | 2026-05-20 | No external service clones needed per dtu-assessment |
| D-003 | CI hotfix: cargo audit shell step | 2026-05-22 | rustsec/audit-check@v2.0.0 fails on push events; PR #111 |
| D-004 | Nightly pin nightly-2026-05-21 is periodic-maintenance | 2026-05-22 | Bumping requires verifying fuzz build; do NOT automate |
| D-005 | Demo recordings local-only (gitignored) | 2026-05-22 | factory-artifacts gitignores cycles/**/demos/; 49 prior files untracked |

## Blocking Issues

None open.

## Drift Items

| ID | Finding | Category | Target Phase | Validation Status |
|----|---------|----------|-------------|-------------------|
| DF-16.A | BC-2.01.001..008 anchor capability CAP-01; CAP-02 (Link-Type Gating) also describes this behavior; capabilities.md not found under .factory/specs/. Capability column may be under-specified or capabilities.md archived/renamed. | architectural | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W9-D1 | STORY-016 F-9 — BC-2.04.047 PC4 should enumerate Truncated/DepthExceeded/SegmentLimitReached behavior for completeness; not blocking | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W9-D2 | STORY-016 F-10 [process-gap] — story-writer template Task #2 wording "Verify Red Gate" incompatible with brownfield-formalization (no Red Gate possible); needs template revision | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W9-D3 | STORY-016 F-11 [process-gap] — story template lacks per-AC VP trace column; needs template enhancement | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W9-D4 | STORY-016 F-12 — story Token Budget template hardcodes "200K for Sonnet"; needs parameterization or removal | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W9-D5 | STORY-020 F-004 — AC-005 test cannot distinguish "evict_flows called but exits immediately" from "never called"; would require evict_flows_calls_for_testing seam; acceptable since production code is observable in behavior | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W9-D8 | **[RESOLVED 2026-05-26]** [process-gap] Sibling-discipline pattern codified as DF-SIBLING-SWEEP-001 in `.factory/policies.yaml` on 2026-05-26 (pre-Wave-10). Orchestrator now injects sibling-sweep checklist into every remediation dispatch. Original finding: STORY-020 per-story P2/P3/P4 + wave-level P1/P2/P3 = 6 total recurrences; each remediation created a new sibling-regression next pass. | process-gap | **RESOLVED** | **RESOLVED** 2026-05-26 — Codified as DF-SIBLING-SWEEP-001 in .factory/policies.yaml. No GitHub issue required (structural codification is the resolution). |
| W9-D9 | BC-2.04.014 + STORY-020 anchor drift: lifecycle.rs:51 cited but correct anchor is lifecycle.rs:60. 3 occurrences in STORY-020 (Architecture Mapping, Token Budget, File Structure tables); 2 occurrences in BC-2.04.014. | spec-gap | — | **RESOLVED** 2026-05-26 by F-W9P1-003 remediation — BC-2.04.014 v1.3 + STORY-020 v1.7; factory commit a3e8927 |
| W9-D12 | F-W9P1-005 (LOW pending intent): `packets_dropped_capacity` stats counter not present in production code. Observability gap on BC-2.04.015 PC-6 drop event. May be intentional design (stats.evictions counts evicted flows, not refused new flows). Requires intent adjudication before filing as issue. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |

## Cycle-Close Follow-Up Items

Process-gap codification required before cycle can be declared closed:

| ID | Item | Priority |
|----|------|----------|
| W1.1 | Wave-gate dispatch: verify `git pull origin develop` before adversarial review. | P1 |
| W1.2 | Brownfield static-assertion tests must anchor to non-test code or use line-range verification. | P2 |
| W1.3/W2.5 **[RECURRING Waves 1-5]** | No pipeline gate advances story status on merge. 5-wave recurrence. Req. research-agent validation. | P1 |
| W2.1 | VP-anchored file-existence tests must assert ≥1 structural content invariant. | P2 |
| W2.2 | CI VP-anchored jobs must include smoke assertion (e.g., `-runs=100`). | P2 |
| W2.3 | Story frontmatter should include `bc_versions:` map at authoring time. | Minor |
| W2.6 | Cargo.toml `rust-version = "1.91"` vs CLAUDE.md "requires Rust 1.85+"; reconcile. | Minor |
| W3.1 | Test-naming `ecNNN` suffix tracks story EC IDs not BC EC IDs — drift risk. Req. research-agent. | Minor |
| W4.1 **[RECURRING Waves 4+6+7 — recurrence #4]** | Anchor agents must re-read from disk after src edits; sweep must verify end-line AND description semantics, not only start-line. Cycle-close codification: anchor-validation hook checking both bounds AND semantic descriptions. Req. research-agent. | P1 |
| W7.1 | No public-API surface gate for pub fn additions. Candidate: `cargo public-api` CI job. Req. research-agent. | P2 |
| W7.2/W8.4 **[RECURRING Waves 7+8 — W7.2 recurrence in W8]** | Partial-fix regression: every remediation must sweep entire axis surface. W8 instances: sibling-BC enforcement-mode (p1), within-BC sibling-section (p4), ADR-narrative (p5), BC↔test (p6). Codification: pre-merge sibling-discipline checklist for BC updates. Req. research-agent. | P1 |
| W7.3 | Out-of-scope anchor drift in src/analyzer + src/decoder. Proactive sweep when Wave 9+ touches analyzer. Req. research-agent. | P3 |
| W8.1 | Stale local develop caused FALSE-POSITIVE F-1/F-2 HIGH in wave-level pass-3. Orchestrator MUST `git pull origin develop --ff-only` before each wave-level adversary dispatch. Req. research-agent. | P1 |
| W8.2 | ADR amendment dialect drift: STORY-019 src comments used "(choice (b))" vocab not present in ADR-0004. Enforcement reviews must verify cited vocab exists in ADR. Req. research-agent. | P2 |
| W8.3 | Wave-level adversarial cost escalation (9 passes vs 7's 8). Likely W7.2 pattern at wave scale. Codification: pass-N+1 must-not-recur assertion. Req. research-agent. | P2 |

Historical process-gap items from Phase 1 (P1.1–P1.3, P3-PG, P4-PG1/2/3, P5-PG, P8-DEFER,
P10-PG, P-CITE-PG): archived in `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.

## Governance Policy

**DF-VALIDATION-001** (commit 9b6efd3, `.factory/policies.yaml`): every
deferred/open finding must be research-agent validated before filing as a
GitHub issue. Pointer in `CLAUDE.md` on `develop` via PR #99 (0082a0c).

**DF-SIBLING-SWEEP-001** (added 2026-05-26, `.factory/policies.yaml`): every
remediation dispatch to story-writer, test-writer, or product-owner MUST
include an explicit sibling-sweep checklist. Orchestrator MUST inject the
checklist under "## Sibling-Sweep Checklist (MANDATORY per DF-SIBLING-SWEEP-001)"
into every dispatch prompt. Derived from W9-D8 (6 consecutive recurrences in
Wave 9). Severity: CRITICAL.

**DF-PR-MANAGER-COMPLETE-001** (added 2026-05-26, `.factory/policies.yaml`):
pr-manager MUST complete steps 7-9 (handle approval, squash merge, post-merge
cleanup) before reporting back to the orchestrator. APPROVE verdict is step 6
of 9 — NOT the stopping point. Orchestrator MUST inject this policy with the
concrete `gh pr merge <#> --squash --admin --delete-branch` command template
under "## PR Completion Policy (MANDATORY per DF-PR-MANAGER-COMPLETE-001)"
into every pr-manager dispatch. Derived from W9.L3 (7 consecutive PRs
#122/123/126/127/128/129/130 across Waves 8-9). Severity: HIGH.

## Tech Debt (Open)

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| O-07 | `rayon` declared in Cargo.toml but unused in `src/` — dead dependency | P2 | adversarial pass 1 (LOW finding) |
| O-08 | `src/analyzer/dns.rs` module doc-comment stale — references removed behavior | P3 | adversarial pass 29 (observation O-1); recorded in domain-debt.md |

Full register: `.factory/tech-debt-register.md`

## Open Issues (from Phase 0 / deferred findings)

| Issue | Summary |
|-------|---------|
| #100 | `Finding.timestamp` always None; thread pcap timestamps |
| #101 | Empirically characterize anomaly-threshold FP rates |
| #102 | Cap weak-cipher ClientHello evidence Vec, CWE-405 |
| #103 | Bidirectional size-symmetry discriminator for small-segment detector |
| #104 | Surface control bytes in non-ASCII SNI summary, BC-TLS-037 |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- SS-03 gap in BC numbering is intentional (subsystem not applicable).
- Phase 0 canonical ground truth: `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`.
- Wave-level convergence history: `.factory/cycles/phase-3-tdd/convergence-trajectory.md`.
- Phase 1 adversary pass detail (33 passes): `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.
- Phase 2 story-adversary pass detail (10 passes): `.factory/cycles/v0.1.0-greenfield-spec/story-adversary-pass-*.md`.
