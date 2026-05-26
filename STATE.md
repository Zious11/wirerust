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
develop_head: 4b9b85f
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
wave_9_status: in_progress
wave_9_started: "2026-05-26"
wave_9_stories: STORY-016 + STORY-020
wave_9_notes: "parallel-eligible (disjoint files: segment.rs+flow.rs vs mod.rs+lifecycle.rs)"
current_wave: 9
stories_delivered: 16
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0→0→0|W7-story:8ps-3clean|W7-wave:8ps-3clean|W8-S019-story:8ps-3clean(14rem)|W8-S015-story:8ps-3clean(14rem)|W8-wave:9ps-3clean(12rem)"
consistency_audit: CONSISTENT
input_drift_check: CLEAN
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-8 CLOSED/CONVERGED; Wave 9 IN PROGRESS.
16 stories delivered across Waves 1-8 (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015).
STORY-019: PR #122 squash-merged → c141c094 (2026-05-26). RST/FIN/expire_flows/missing-key lifecycle;
per-story 8 passes, 3-clean streak passes 6/7/8 (14 findings remediated).
STORY-015: PR #123 squash-merged → 6c53fc14 (2026-05-26). In-order delivery + OOO + bidirectional +
wraparound; per-story 8 passes, 3-clean streak passes 6/7/8 (14 findings remediated).
ADR-0004 v2 amendment: PR #124 → d6614d22 (CLOSE_FLOW_MISSING_WARNED seams + state-injection seam class).
Wave-level convergence: 9 passes, 3-clean streak passes 7/8/9 (12 findings remediated; 3 develop PRs +
4 factory BC commits). Wave 8 CLOSED/CONVERGED. STORY-016/020 unblocked — Wave 9 READY.
develop HEAD: 4b9b85f (PR #126 — ADR-0004 visibility-widening enumeration fix merged 2026-05-26).

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
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-8 CLOSED/CONVERGED; 16 stories delivered; Wave 9 IN PROGRESS (STORY-016 + STORY-020 dispatched 2026-05-26) | Wave-level convergence detail: cycles/phase-3-tdd/convergence-trajectory.md |
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
| 9 | STORY-016, STORY-020 | **IN PROGRESS** 2026-05-26 | — | STORY-016: overlap detection (BCs 2.04.035/036/038/043/047); STORY-020: memory management (BCs 2.04.014-017); parallel-eligible (disjoint files: segment.rs+flow.rs vs mod.rs+lifecycle.rs) |
| 10–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 8 — wave-level adversarial convergence | **COMPLETE** 2026-05-26 | 9 passes; 3/3 clean streak passes 7/8/9; 12 findings remediated; 3 develop PRs + 4 factory BC commits; 4 drift items logged (W8.1–W8.4) |
| Wave-gate — Wave 8 | **CLOSED** 2026-05-26 | develop HEAD 4b9b85f; 16 stories total Waves 1-8; STORY-016/020 unblocked |
| Wave 9 — dispatch | **COMPLETE** 2026-05-26 | STORY-016 (overlap detection; BCs 2.04.035/036/038/043/047; worktree-story-016) + STORY-020 (memory management; BCs 2.04.014-017; worktree-story-020); parallel-eligible (disjoint files) |
| Wave 9 — burst-1 spec+story revisions | **COMPLETE** 2026-05-26 | BC-2.04.015 v1.3 (EC-004 fix + DESIGN INTENT inv4 + EC-005); BC-2.04.036/038/047 v1.3 (anchor drift fix); STORY-016 v1.2 (EC-002 "adjacent (contiguous)"); STORY-020 v1.2 (EC-005 revised + EC-011 dual-pressure); input-hashes refreshed (STORY-016: 8cc2ec9→3f3509b; STORY-020: c859b44→6527afc) |
| Wave 9 — per-story adversarial passes 1–4 + bursts 3–6 | **IN PROGRESS** 2026-05-26 | STORY-016: pass-1 11 findings → burst-1 remediated (W9-D1..W9-D4 deferred); pass-2 DIRTY → burst-3 (BC-2.04.043 v1.3, BC-2.04.047 v1.4, BC-2.04.038 v1.4, STORY-016 v1.3); pass-3 DIRTY → burst-5 (STORY-016 v1.4); pass-4 DIRTY → pass-5 dispatched. STORY-016 converged: 3 consecutive clean passes P4+P5+P6 (BC-5.39.001). 6 total passes (DIRTY×3+CLEAN×3). STORY-020: pass-1 5 findings → 3 actioned, W9-D5 deferred; pass-2 DIRTY → burst-4 (STORY-020 v1.3); pass-3 DIRTY → burst-5 (STORY-020 v1.4); pass-4 DIRTY 4 findings + 6 LOW obs → burst-6 (STORY-020 v1.5 + BC-2.04.015 v1.4); pass-5 dispatched in parallel with this commit. |
| Wave 9 — STORY-016 CONVERGED | **COMPLETE** 2026-05-26 | 6 passes total (DIRTY×3 + CLEAN×3); per BC-5.39.001 convergence criterion satisfied. STORY-016 ready for demo + PR. |
| Wave 9 — sibling-discipline pattern (W9-D8 filed) | **OBSERVATION** 2026-05-26 | Sibling-discipline pattern recurred in STORY-020 passes 2/3/4 (3-cycle recurrence). Cumulative evidence: ~10 sibling-regression findings across waves 7–9. W9-D8 filed as [process-gap] drift item. Requires research-agent validation per DF-VALIDATION-001. |
| Wave 9 — wave-level adversarial pass-1 | **REMEDIATED** 2026-05-26 | DIRTY 5 findings: F-W9P1-001 MED (joint invariant assertions) → test-writer 0aba23c; F-W9P1-002 MED (BC-2.04.015 PC gap MemoryPressure) → BC-2.04.015 v1.5; F-W9P1-003 MED (lifecycle.rs anchor drift) → BC-2.04.014 v1.3 + STORY-020 v1.7; F-W9P1-004 MED [process-gap] (ISN_MISSING_WARNED_LOCK docstring) → test-writer 0aba23c; F-W9P1-005 LOW deferred → W9-D12. Factory commit a3e8927. Pass-2 + pass-3 dispatched. |

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

## Session Resume Checkpoint (2026-05-26 — Wave 9 burst-9 COMPLETE)

1. Waves 1-8 all CLOSED/CONVERGED — 16 stories delivered.
   STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015 all merged to develop.
2. Wave 8 CLOSED: STORY-019 (PR #122 → c141c094) + STORY-015 (PR #123 → 6c53fc14) + ADR-0004 v2
   PRs #124/#125/#126. develop HEAD: 4b9b85f. Wave-level convergence: 9 passes, 3-clean streak
   passes 7/8/9 (12 findings remediated).
3. Wave 9 IN PROGRESS: sprint-state.yaml current_wave=9.
   STORY-016: status=CONVERGED (per BC-5.39.001), branch=worktree-story-016.
     6 passes total: DIRTY(P1)/DIRTY(P2)/DIRTY(P3)/CLEAN(P4)/CLEAN(P5)/CLEAN(P6). 3-clean streak satisfied.
     READY for demo + PR (pr-manager to push w9-followup branch).
   STORY-020: status=CONVERGED (per-story), branch=w9-followup, worktree=.worktrees/w9-followup.
     Scope: memory management; BCs 2.04.014-017; depends on STORY-019.
     Wave-level pass-1 5 findings → 4 MED remediated + 1 LOW deferred (W9-D12).
     Burst-9 factory commit a3e8927: BC-2.04.015 v1.5 (PC-7) + BC-2.04.014 v1.3 + STORY-020 v1.7.
     Test-writer (0aba23c, w9-followup): F-W9P1-001 assertions + proptest + F-W9P1-004 docstring.
     input-hash: 6527afc → da8045f (refreshed after BC-2.04.014 + BC-2.04.015 changes).
   Sibling-discipline recurrence in STORY-020: W9-D8 filed. Wave-level pass-2 + pass-3 dispatched.
4. STORY-017/018: still blocked (blocked_by includes STORY-016 PR not yet merged).
5. Drift items W9-D1..W9-D8 + W9-D12 logged — see Drift Items table. All require
   research-agent validation per DF-VALIDATION-001 before any GitHub issue is filed.
   W9-D9 (anchor drift BC-2.04.014/STORY-020): RESOLVED by F-W9P1-003 — factory commit a3e8927.
6. NEXT: receive wave-level pass-2 + pass-3 results; remediate if DIRTY; target 3-clean streak.
   pr-manager to push w9-followup PR (STORY-020 src fixes + wave-level remediation).

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
| W9-D8 | [process-gap] Sibling-discipline pattern recurred in STORY-020 passes 2/3/4 (3-cycle recurrence within a single story). Cumulative evidence: ~10 sibling-regression findings across waves 7–9. Codification needed: sibling-sweep should be an explicit checklist step in story-writer/test-writer/PO remediation prompts, not a hopeful adversary-axis catch. MED severity (will be re-classified at convergence if pattern recurs in wave-level adversary). | process-gap | phase-5 | REQUIRES vsdd-factory:research-agent validation per policy DF-VALIDATION-001 before any GitHub issue is filed |
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
