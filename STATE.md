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
develop_head: 3cd3000
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
wave_10_status: closed
wave_10_started: "2026-05-26"
wave_10_closed_date: "2026-05-26"
wave_10_stories: STORY-017 + STORY-018
wave_10_pr_count: 3
wave_10_prs: "#131, #132, #133"
wave_10_per_story_convergence: "STORY-017: 4 passes (1D+3C); STORY-018: 9 passes (6D+3C)"
wave_10_wave_level_convergence: "4 passes (1 DIRTY + 3 CLEAN; passes 2/3/4 clean; 3 findings remediated + 6 deferred)"
wave_11_status: closed
wave_11_closed_date: "2026-05-27"
wave_11_stories: STORY-021
wave_11_pr_count: 1
wave_11_prs: "#134"
wave_11_per_story_convergence: "STORY-021: 11 passes; passes 9-10-11 clean (BC-5.39.001 ACHIEVED)"
wave_11_wave_level_convergence: "11 passes (pass-1: 15→pass-2: 14→pass-3: 5→pass-4: 11→pass-5: 12 (10 false/methodology bug)→pass-6: 4→pass-7: 3→passes 9-10-11: 0); CONVERGENCE ACHIEVED"
wave_12_status: ready_to_dispatch
current_wave: 12
stories_delivered: 21
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0→0→0|W7-story:8ps-3clean|W7-wave:8ps-3clean|W8-S019-story:8ps-3clean(14rem)|W8-S015-story:8ps-3clean(14rem)|W8-wave:9ps-3clean(12rem)|W9-S016-story:6ps-3clean(24rem)|W9-S020-story:8ps-3clean(13rem)|W9-wave:6ps-3clean(11rem)|W10-S017-story:4ps-3clean(MERGED-PR#131)|W10-S018-story:9ps-3clean(MERGED-PR#132)|W10-wave:4ps-3clean(1HIGH+4MED+4LOW->1src+2BC+6def)|W10-CONVERGED-2026-05-26|W11-S021-story:11ps-3clean(15→14→5→11→12→4→3→0→0→0->MERGED-PR#134-3cd3000)|W11-CONVERGED-2026-05-27-BC-5.39.001-ACHIEVED"
consistency_audit: CONSISTENT
input_drift_check: CLEAN
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-11 CLOSED/CONVERGED; Wave 12 ready to dispatch.
21 stories delivered across Waves 1-11 (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020/017/018/021).
Wave 11 CLOSED 2026-05-27: STORY-021 (PR #134 → 3cd3000, 11 passes; passes 9-10-11 CLEAN per BC-5.39.001).
Implementation strategy: brownfield-formalization (zero production behavior changes).
Diff scope: src/reassembly/mod.rs +88, src/analyzer/http.rs +33, src/analyzer/tls.rs +33, tests/reassembly_engine_tests.rs +1290 — 4 files.
Test count: 203/203 reassembly_engine_tests pass on develop; all 15 test binaries pass (0 failures).
Doctrine adopted: BC pre-merge re-anchor (W11.L1) — BC anchors re-anchored to post-merge line numbers as part of convergence cycle.
4 process-gap codifications applied (DF-SIBLING-SWEEP-001 v2 extended + DF-ADVERSARY-METHODOLOGY-001 new).
develop HEAD: 3cd3000 (PR #134 — Wave 11 STORY-021 2026-05-27).

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
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-11 CLOSED/CONVERGED; 21 stories delivered; Wave 12 ready to dispatch (STORY-031 pending) | Wave-level convergence detail: cycles/phase-3-tdd/convergence-trajectory.md |
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
| 10 | STORY-017, STORY-018 | **CLOSED/CONVERGED** 2026-05-27 | 211143e (PR #133 — wave-level fix) | STORY-017 MERGED PR #131 (4 passes 1D+3C; 24 tests + 9 ECs). STORY-018 MERGED PR #132 (9 passes 6D+3C; resource bounds). Wave-level 4 passes (1D+3C; 3 findings remediated + 6 deferred). 17 adversarial passes total (15% reduction vs Wave 9: 20). |
| 11 | STORY-021 | **CLOSED/CONVERGED** 2026-05-27 | 3cd3000 (PR #134) | STORY-021 MERGED PR #134 (11 passes; 9-10-11 CLEAN per BC-5.39.001). Brownfield-formalization: +88/+33/+33/+1290 lines, 4 files, 203 new tests. BC pre-merge re-anchor doctrine adopted (W11.L1). Methodology bug caught (W11.L2). 4 process-gap codifications applied. |
| 12–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 10 — per-story adversarial pass-4/5 remediation (burst-3) | **COMPLETE** 2026-05-26 | STORY-017 CONVERGED+MERGED PR #131 → ced10aa (4 passes 1D+3C; 24 tests + 9 ECs; 33% pass reduction vs W9-S016). STORY-018 pass-4 DIRTY (1 MED W10-D6 pattern at deeper layer) → v1.4 remediated. STORY-018 pass-5 DIRTY (2 MED: cross-ref target validation + implementation reachability) → v1.5 remediated. DF-SIBLING-SWEEP-001 v3 applied. W10-D7 + W10-D8 filed. factory-artifacts HEAD d74e7b2. |
| Wave 10 — wave-level pass-1 remediation (burst-4) | **COMPLETE** 2026-05-26 | STORY-018 MERGED PR #132 → 211143e (20 stories delivered). Wave-level pass-1 DIRTY: F-W10P1-001 REMEDIATED PR #133 (overlap_count saturating_add); F-W10P1-002/003 REMEDIATED BC-2.04.022/027 anchors. W10-D10..D14 DEFERRED. factory-artifacts HEAD b989cf2. |
| Wave 11 OPEN — STORY-021 dispatched | **COMPLETE** 2026-05-27 | STORY-021 (finalize lifecycle + MAX_FINDINGS cap + segment-limit summary) implemented and converged. 11 adversarial passes; passes 9-10-11 CLEAN per BC-5.39.001. BC pre-merge re-anchor doctrine adopted (W11.L1). Methodology bug caught at pass-5 (W11.L2). |
| Wave 11 — PR #134 merged | **COMPLETE** 2026-05-27 | STORY-021 MERGED PR #134 → 3cd3000 (implementer-as-PR-executor; all 9 steps autonomous). 21 stories delivered. W11.L3: line-citation drift across 4 cycles (677→690→793→807→796→810→794→808). W11.L4: source-docstring propagation gap. W11.L5: pr-manager stops-at-APPROVE structural gap confirmed. |
| Wave 11 CLOSED — 4 process-gap codifications applied | **COMPLETE** 2026-05-27 | DF-SIBLING-SWEEP-001 extended to v2 (BC pre-merge re-anchor, lock-discipline propagation, story-writer post-AC-add sibling-sweep, source-docstring propagation). DF-ADVERSARY-METHODOLOGY-001 added (absolute-path requirement in adversary dispatches). Wave 12 ready to dispatch (STORY-031 pending). |

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

## Session Resume Checkpoint (2026-05-27 — Wave 11 CLOSED)

1. Waves 1-11 all CLOSED/CONVERGED — 21 stories delivered.
   STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020/017/018/021 all merged to develop.
   STORY-021 PR #134 → 3cd3000 (11 passes; passes 9-10-11 CLEAN per BC-5.39.001; brownfield-formalization).
2. Wave 11 CLOSED 2026-05-27. Convergence: 11 passes (15→14→5→11→12→4→3→0→0→0).
   Pass-5 methodology bug (cd non-persistent): 10 false findings discarded; real findings = 2.
   BC pre-merge re-anchor doctrine adopted. impl Drop citation: 794-808 (worktree-post-STORY-021).
3. 4 process-gap codifications committed to .factory/policies.yaml:
   - DF-SIBLING-SWEEP-001 extended (BC pre-merge re-anchor + lock-discipline + post-AC-add sweep + docstring propagation)
   - DF-ADVERSARY-METHODOLOGY-001 added (absolute-path mandatory in adversary dispatches)
4. Drift items W11-D1..D7 require research-agent validation per DF-VALIDATION-001 before GitHub issue filing.
5. develop HEAD: 3cd3000. All 15 test binaries pass; 203/203 reassembly_engine_tests green.
6. NEXT: Wave 12 — dispatch STORY-031 (dependency STORY-021 now done).
   Inject DF-SIBLING-SWEEP-001 v2 + DF-ADVERSARY-METHODOLOGY-001 at all remediation and adversary dispatches.

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

### Wave 10 Retrospective (closed 2026-05-27)
- Stories: STORY-017 (4 passes 1D+3C) + STORY-018 (9 passes 6D+3C)
- PRs: #131 STORY-017 (ced10aa) + #132 STORY-018 (f4963ba) + #133 wave-followup (211143e overlap_count saturating_add HIGH)
- Wave-level: 4 passes (1D + 3C at passes 2/3/4)
- Total adversarial: 17 passes (vs Wave 9: 20 = 15% reduction)
- DF-SIBLING-SWEEP-001 demonstrably effective: STORY-017 cleared in 4 passes vs equivalent Wave 9 stories needing 6-8; iteratively refined v1→v4 through STORY-018 passes
- DF-PR-MANAGER-COMPLETE-001 partial: pr-manager stops at APPROVE on PRs #131/#132; implementer-as-PR-executor on PR #133 worked autonomously (W10-D7 workaround)
- 3 brownfield spec/impl mismatches resolved (BC-2.04.041/045/027 v1.3)
- 1 src/ hardening: overlap_count saturating_add (PR #133)
- Active drift: W10-D1..D14 + Wave 9 carryover; all REQUIRE DF-VALIDATION-001 research-agent validation before issue filing

### Wave 11 Retrospective (closed 2026-05-27)

- Stories: STORY-021 (finalize lifecycle + MAX_FINDINGS cap + segment-limit summary; brownfield-formalization)
- PRs: #134 STORY-021 → 3cd3000 (implementer-as-PR-executor; all 9 steps autonomous)
- Adversarial passes: 11 total (pass-1: 15 → pass-2: 14 → pass-3: 5 → pass-4: 11 → pass-5: 12 [10 false/methodology bug] → pass-6: 4 → pass-7: 3 → passes 9-10-11: 0)
- Convergence gate: BC-5.39.001 ACHIEVED at pass 11 (3 consecutive clean passes)
- Implementation scope: src/reassembly/mod.rs +88, src/analyzer/http.rs +33, src/analyzer/tls.rs +33, tests/reassembly_engine_tests.rs +1290 — 4 files, zero production behavior changes
- Test results: 203/203 reassembly_engine_tests pass; all 15 test binaries pass (0 failures)
- Adversarial discovery cost trend: pass-1: 15 → pass-2: 14 → pass-3: 5 → pass-4: 11 → pass-5: 12 (10 false from methodology bug) → pass-6: 4 → pass-7: 3 → passes 9-10-11: 0
- Doctrine flip in pass-8 (BC pre-merge re-anchor): when story shifts cited source-line numbers, BCs in story `bcs:` frontmatter are re-anchored to post-merge line numbers as part of convergence cycle — not deferred. Zero post-merge stale-anchor window. Final impl Drop anchor: 794-808.
- Methodology bug (pass-5): adversary `cd <worktree>` did not persist across Bash invocations; adversary grepped main repo instead of worktree; produced 2 FALSE-CRITICAL findings. Orchestrator caught. Future dispatches hardened with DF-ADVERSARY-METHODOLOGY-001.
- Iterative drift pattern: impl Drop line citation drifted across 4 cycles (677-690 → 793-807 → 796-810 → 794-808) due to seam-block edits shifting downstream code.
- Implementer-as-PR-executor pattern: validated again (PR #134; all 9 steps autonomous; W10-D7 codification holds)
- 4th consecutive wave confirming pr-manager stops-at-APPROVE structural gap (W8.L4 → W9.L2 → W10.L3 → W11.L5)
- Process-gap codifications applied: DF-SIBLING-SWEEP-001 extended to v2 (4 new bullet groups); DF-ADVERSARY-METHODOLOGY-001 added
- Active drift items: W11-D1..D7 (all LOW/deferred); all REQUIRE DF-VALIDATION-001 research-agent validation before issue filing
- Demo evidence: 6 .gif + 6 .tape + evidence-report.md, 708 KB, local-only at .factory/cycles/v0.1.0-greenfield-spec/wave-11/story-021/demos/ (gitignored)

Full retrospective detail: `.factory/cycles/phase-3-tdd/lessons.md` (W11 lessons)

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
| W10-D2 | STORY-017 pass-2 F-PASS2-LOW-A (LOW): line 187 Architecture Compliance Rule cites BC-2.04.022 invariant 1; correct citation is PC-1/INV-2. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D3 | STORY-017 pass-2 F-PASS2-LOW-B (LOW): BC-2.04.019 anchor mod.rs:430-449 is off-by-one from actual implementation lines. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D4 | STORY-017 pass-2 F-PASS2-LOW-C (LOW): BC-2.04.022 Source Evidence inner-line citations have mixed semantics (some character-level, some token-level). | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D5 | STORY-018 pass-1 F-003 carry-forward (LOW): AC-005 uses 3 distinct execution flows but EC-002 covers only the same-flow case; the other 2 flows lack coverage. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D6 | [process-gap] DF-SIBLING-SWEEP-001 v1 checklist does not explicitly enumerate BC→story-EC, BC→test-prose, and BC→test-name propagation paths. Surfaced in STORY-018 pass-2 (3 MED sibling-regressions of BC-2.04.045 v1.3 fix that passed BC→BC sweep but missed story/test paths). Policy v2 codification candidate post-Wave 10. | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D7 | [process-gap] DF-PR-MANAGER-COMPLETE-001 v1 enforcement insufficient at dispatch-prompt level — pr-manager STOPPED at APPROVE step on STORY-017 PR #131 despite explicit policy injection. Root cause: policy enforcement lives only in orchestrator dispatch text, not in pr-manager agent prompt itself. Codification candidate: edit pr-manager agent prompt in vsdd-factory plugin source, not just dispatch-time injection. RESOLUTION CANDIDATE — implementer-as-PR-executor proven reliable on PR #133 (autonomous through merge); recommend retiring pr-manager dispatch in favor of implementer for PR completion. | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D8 | [spec-gap, defer-as-integration] BC-2.04.045 v1.3 PC2 "or no gaps fit at all" wording is structurally unreachable per early-guard analysis — the STORY-018 v1.5 pass-5 implementation-reachability finding. Should be removed at wave-gate to prevent re-propagation to STORY-021 + future SS-04 stories. Deferred as integration item. | spec-gap | wave-gate | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D10 | F-W10P1-004 (MED): STORY-018 AC-005 and EC-008 test coverage is duplicated — fill_findings_to_cap helper exists but test uses manual duplication pattern instead. Refactor opportunity reduces test code volume without changing coverage. Source: STORY-018 tests/reassembly_segment_tests.rs. | test-quality | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D11 | F-W10P1-005 (MED): No AC pins evidence strings for small-segment + overlap findings in STORY-018 — only AC-012 covers out-of-window (OOW) case. Small-segment and overlap finding evidence strings lack acceptance criteria anchors, leaving spec coverage gap for those two finding types. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D12 | O-W10P1-001 (LOW): BC-2.04.018 PC2 parenthetical overgeneralizes direction:None — states behavior applies to "all direction values" but direction:None is not a valid direction for per-flow findings. PC2 wording may confuse future story implementers. Wave-gate fix candidate. | spec-gap | wave-gate | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D13 | O-W10P1-002 (LOW): Truncated path overlap detection skips bytes beyond `allowed` without any security note — bytes past the depth limit are silently discarded; this is intentional behavior but the BC lacks a security-implication note for analysts reviewing partial-stream reassembly. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W10-D14 | O-W10P1-003 (LOW): No AC in STORY-018 verifies direction:None for ConflictingOverlap finding — the BC states direction:None is valid for flow-level findings but no test AC anchors this invariant, leaving it unverifiable from the story alone. | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D1 | BC-2.04.025/012/026 VP-NNN identifiers are "—" (pre-existing placeholder, F-W11P2-013). | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D2 | Trust-boundary CI lint to forbid `_for_testing(` calls in src/ (F-W11P2-015). | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D3 | Adversary read-only profile cannot run toolchain (axis G) — orchestrator must pair adversary with toolchain runner (F-W11P4-010). | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D4 | Adversary read-only profile cannot run compute-input-hash --check (F-W11P4-011). | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D5 | Orchestrator dispatch scope-of-change should be generated from `git diff --stat` actuals rather than manual description (F-W11P5-010). | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D6 | Cargo.toml rust-version="1.91" vs CLAUDE.md says 1.85+ (pre-existing, observed wave-level; see also W2.6). | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
| W11-D7 | AC-007 vs AC-007b — test name `test_BC_2_04_024_engine_cap_at_exactly_10000` covers AC-007 as sub-assertion of AC-007b's test (18 functions for 19 ACs). | spec-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |

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

**DF-ADVERSARY-METHODOLOGY-001** (added 2026-05-27, `.factory/policies.yaml`):
All file operations in adversary dispatch prompts MUST use absolute paths.
The `cd <path> && ...` pattern is FORBIDDEN in adversary dispatches because
the cd does not persist across Bash invocations in some agent profiles,
causing the adversary to query the wrong filesystem. Git operations must use
`git -C <absolute-path> ...`. Derived from W11.L2 (pass-5 adversary grepped
main repo instead of worktree; produced 2 FALSE-CRITICAL findings).
Severity: HIGH.

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
