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
develop_head: b23c6d3
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
wave_8_status: ready_to_dispatch
current_wave: 7
stories_delivered: 14
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "17→13→7→19→8→3→13→7→4→6→1→6→5→3→4→3→5→5→2→4→3→0→3→0→4→SWEEP68→5→SWEEP48→1→0→0→3→0→0→0|W7-story:8ps-3clean|W7-wave:8ps-3clean"
consistency_audit: CONSISTENT
input_drift_check: CLEAN
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-7 CLOSED/CONVERGED; Wave 8 READY TO DISPATCH.
14 stories delivered across Waves 1-7 (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014).
STORY-014: PR #120 squash-merged → bc5d23e (2026-05-25). 17 tests + 2 #[doc(hidden)] test seams;
mid-stream join + ISN management + IsnMissing guard. ADR-0004 amended via PR #121 → b23c6d3.
Per-story convergence: 8 passes, 3-clean streak passes 6/7/8. Wave-level convergence: 8 passes,
3-clean streak passes 6/7/8. Wave 7 CLOSED/CONVERGED. STORY-015/019 unblocked — Wave 8 READY.
develop HEAD: b23c6d3 (PR #121 — ADR-0004 amendment merged 2026-05-25).

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** 463 passing on develop (446 + 17 new STORY-014 BC tests). `cargo fmt --check`,
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
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-7 CLOSED/CONVERGED; 14 stories delivered; Wave 8 READY TO DISPATCH | Wave-level convergence detail: cycles/phase-3-tdd/convergence-trajectory.md |
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
| 8–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 7 — STORY-014 delivery (PR #120) | **COMPLETE** 2026-05-25 | squash-merged → bc5d23e; 17 tests + 2 #[doc(hidden)] test seams; mid-stream join + ISN management + IsnMissing guard; per-story 8 passes, 3-clean streak passes 6/7/8 |
| Wave 7 — ADR-0004 amendment (PR #121) | **COMPLETE** 2026-05-25 | squash-merged → b23c6d3; documents test-seam exception for #[doc(hidden)] pub fn accessors |
| Wave 7 — wave-level adversarial convergence | **COMPLETE** 2026-05-25 | 8 passes; 3/3 clean streak passes 6/7/8; factory-artifacts: c2a0181 → 4a200d0 → 5e6cc59 → 6db1772 → aeacc6a → 6d9c1fc |
| Wave-gate — Wave 7 | **CLOSED** 2026-05-25 | develop HEAD b23c6d3; 14 stories total Waves 1-7; sprint-state STORY-014 → done; STORY-015/019 → pending |
| Wave 8 — dispatch ready | **READY** 2026-05-25 | STORY-015 (TCP half-close) + STORY-019 (reassembly limits) both unblocked |

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

## Session Resume Checkpoint (2026-05-25 — Wave 7 CLOSED; Wave 8 READY)

1. Waves 1-7 all CLOSED/CONVERGED — 14 stories delivered.
   STORY-001/069/002/003/004/070/071/005/011/066/012/013/014 all merged to develop.
2. Wave 7 STORY-014 DELIVERED: PR #120 squash-merged → bc5d23e (2026-05-25). 17 tests +
   2 #[doc(hidden)] test seams (mid-stream join + ISN management + IsnMissing guard).
   Per-story convergence 8 passes, 3-clean streak passes 6/7/8.
   ADR-0004 amended via PR #121 → b23c6d3 (2026-05-25) — documents test-seam exception.
   Wave-level convergence 8 passes, 3-clean streak passes 6/7/8. Wave 7 CLOSED/CONVERGED.
   develop HEAD: b23c6d3. 463 tests.
3. STORY-015 and STORY-019 unblocked (blocked_by [STORY-013, STORY-014]; both now done).
   sprint-state.yaml: STORY-015 status=pending, STORY-019 status=pending. current_wave=7.
4. 4 cycle-close drift items logged (W7.1–W7.4) — see Drift Items below. All require
   research-agent validation per DF-VALIDATION-001 before any GitHub issue is filed.
5. NEXT: Dispatch Wave 8 (STORY-015 + STORY-019) — per-story adversarial → deliver → wave-gate.

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

## Cycle-Close Follow-Up Items

Process-gap codification required before cycle can be declared closed:

| ID | Item | Priority |
|----|------|----------|
| W1.1 | Wave-gate dispatch pre-check: add `git pull origin develop` verification step before wave-gate adversarial review. Observed: local develop stale after `gh pr merge`. | P1 |
| W1.2 | Brownfield static-assertion tests must anchor to non-test code or use line-range verification; `assert!(content.contains(...))` does not distinguish test-only vs production paths. | P2 |
| W1.3 | Deliver-story skill must emit a state-manager update command on completion to flip story status. | P1 |
| W2.1 | VP-anchored file-existence tests must also assert at least one structural invariant of file content. | P2 |
| W2.2 | CI jobs guarding VP-anchored verification properties must include a smoke assertion (e.g., `-runs=100`). | P2 |
| W2.3 | Story frontmatter should include `bc_versions:` map listing each cited BC and version at authoring time. | Minor |
| W2.5 | Deliver-story skill must flip story `status: draft` → `completed` on PR merge (same as W1.3; recurrence in Wave 2). | P1 |
| W2.6 | Cargo.toml pins `rust-version = "1.91"` while CLAUDE.md states "requires Rust 1.85+"; reconcile in a maintenance sweep. | Minor |
| W3.1 | Test-naming `ecNNN` suffix tracks story EC IDs, not BC EC IDs — drift risk. Raised STORY-005 pass-8. Do NOT file GitHub issue until research-agent validates (DF-VALIDATION-001). | Minor |
| W3.2 **[CONFIRMED RECURRING — Waves 3+4+5]** | No pipeline gate advances story `status: draft` → `completed` on merge. Recurred in Wave 3 (STORY-005/071), Wave 4 (STORY-011/066), and Wave 5 (STORY-012). Three-wave recurrence confirms P1 priority. Do NOT file GitHub issue until research-agent validates (DF-VALIDATION-001). | **P1 — RAISED PRIORITY** |
| W4.1 **[CONFIRMED RECURRING — Waves 4+6+7 (recurrence #4)]** | Src edits that shift line counts must land and commit BEFORE anchor re-derivation agents are dispatched; anchor agents must re-read from disk, not use offsets computed in the same burst. Wave 7 pass-4: mega-sweep at 6db1772 verified function-start lines but missed closing-brace boundary correctness and description-vs-line-range semantic correspondence. Three-wave recurrence (4, 6, 7). Cycle-close codification candidate: enhanced anchor-validation pre-commit hook or scripted sweep tool checking both bounds AND semantic descriptions. Do NOT file GitHub issue until research-agent validates (DF-VALIDATION-001). | **P1 — RAISED PRIORITY (recurrence #4 — Wave 7)** |
| W7.1 **[NEW — Wave 7 pass-1]** | No public-API surface gate: project has no `cargo public-api`/`cargo semver-checks` CI gate. Two `#[doc(hidden)] pub fn` test-seam accessors landed without surface-diff visibility. Cycle-close codification candidate: add `cargo public-api` snapshot CI job OR document in CLAUDE.md that pub-fn additions require explicit PR call-out. Target: v0.1.0-release. Do NOT file GitHub issue until research-agent validates (DF-VALIDATION-001). | P2 |
| W7.2 **[NEW — Wave 7 passes 4+5+6+7, recurring]** | Partial-fix regression discipline recurrence: every "comprehensive" remediation in Wave 7 tended to be too narrowly scoped — sibling BCs missed (pass-2), mod.rs cites missed (pass-3), closing-brace + semantic missed (pass-4), sibling-row missed in HS-014 fix (pass-5). Each fresh-context adversary found the same class of issue in adjacent positions. Rule affirmed: every remediation MUST sweep the same axis across the entire affected surface. This is already in established rules; Wave 7 confirms the rule is not consistently followed. Target: cycle-close codification. Do NOT file GitHub issue until research-agent validates (DF-VALIDATION-001). | P2 |
| W7.3 **[NEW — Wave 7 mega-sweep, out-of-scope]** | Out-of-scope src/analyzer + src/decoder anchor drift noted by mega-sweep at 6db1772 but explicitly NOT in Wave 7 scope. Future scoped sweep needed when analyzer subsystem changes are anchored (Wave 9+ or whenever src/analyzer is next touched). Do NOT file GitHub issue until research-agent validates (DF-VALIDATION-001). | P3 — Wave 9+ |

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
