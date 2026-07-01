---
document_type: pipeline-state
project: wirerust
mode: feature
phase: feature-ready
status: idle
current_step: "PIPELINE AT REST — no active cycle. Session paused 2026-07-01."
pipeline: FEATURE-CYCLE
timestamp: 2026-07-01T00:00:00Z

# Release chain (latest)
released_version: v0.11.1
released_at: "2026-07-01"
release_tag: v0.11.1
release_tag_object: e8a8a2d4e7cd03e337b066859586e2c610208888
release_commit: 4e2b28529ae196785ce6a0baed522b9939f929ea
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.11.1
prior_released_version: v0.11.0
prior_released_at: "2026-06-29"
# Ground-truth HEADs (verified 2026-07-01 — PR #347 main merge + #348 develop back-merge)
main_head: 4e2b28529ae196785ce6a0baed522b9939f929ea
develop_head: 3a60317965e62bef9895e857c8a26fc3b8d03ad0
# Cargo.toml version on main and develop (in sync)
cargo_version_main: "0.11.1"
cargo_version_develop: "0.11.1"
# Open worktrees: main checkout [develop] + .factory [factory-artifacts]. release/back-merge worktrees removed.
# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
# Story tracking
stories_delivered: 94
story_index_version: v3.10
total_stories: 103
story_index_note: "103 stories / 66 waves. STORY-148/149/150 added (maint-2026-07-01). IDX-003 total_points reconciled 656→659. develop=3a60317."
# Spec versions (current)
bc_index_version: v2.3
vp_index_version: v2.28
arch_index_version: v2.5
prd_version: v1.45
epics_version: v1.8
# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-07-01
maintenance_started_at: "2026-07-01"
maintenance_completed_at: "2026-07-01"
maintenance_prior_run: maint-2026-06-22
---

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**PIPELINE AT REST — no active cycle. See Session Resume Checkpoint below.**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | feature (post-maint-2026-07-01; idle) |
| Version | 0.11.1 (released) |
| Main HEAD | `4e2b285` (full: `4e2b28529ae196785ce6a0baed522b9939f929ea`) |
| Develop HEAD | `3a60317` (full: `3a60317965e62bef9895e857c8a26fc3b8d03ad0`) |
| Tag v0.11.1 | commit `4e2b285`; tag object `e8a8a2d4` |
| GitHub release | https://github.com/Zious11/wirerust/releases/tag/v0.11.1 (Latest, not draft) |
| Factory artifacts HEAD | see `git -C .factory log -1 --format='%h %s'` |
| Spec versions | BC-INDEX v2.3 / VP-INDEX v2.28 (40 VPs) / ARCH-INDEX v2.5 / PRD v1.45 |
| Stories | 94 delivered / 103 total (STORY-INDEX v3.10) |

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0–7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0..v0.11.0 | RELEASED | Details: cycles/ subdirs |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking |
| Maintenance maint-2026-07-01 | **COMPLETE 2026-07-01** | PRs #349+#350 merged; develop=3a60317; STORY-148/149/150 drafted |
| Feature cycle fix-tls-clienthello-frag — F1 | DONE | delta-analysis.md committed |
| Feature cycle fix-tls-clienthello-frag — F2 | APPROVED (D-305, 2026-06-29) | 6 new BCs + 3 amended + VP-039 + VP-040 + ADR-011 |
| Feature cycle fix-tls-clienthello-frag — F3 | APPROVED (D-306, 2026-06-29) | STORY-144..146; STORY-INDEX v3.6; HS-F4-001..012 |
| Feature cycle fix-tls-clienthello-frag — F4 | **DONE/PASS** | Holdout 0.904 mean, 8/8 must-pass; HS-F4-001 artifact-fidelity fix |
| Feature cycle fix-tls-clienthello-frag — F5 | **DONE/CONVERGED** | 5 passes; BC-completeness 60/60, 0 P0; BC-INDEX v2.3 |
| Feature cycle fix-tls-clienthello-frag — F6 | **DONE** | Kani VP-039 3 proofs PASS; fuzz 1.9M execs clean; 100% real-gap mutation kill (mod f6_hardening, 12 tests); anyhow 1.0.103 (RUSTSEC-2026-0190 cleared). PRs #345+#346 merged. develop=52907bc. |
| Feature cycle fix-tls-clienthello-frag — F7 | **DONE/CONVERGED (D-316)** | v0.11.1 released (PR #347 main, #348 back-merge); S-7.02 SATISFIED; cycle CLOSED. |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| F7: delta convergence | DONE/CONVERGED | v0.11.1 released (#347 main + #348 back-merge, ba6fbd8). S-7.02 SATISFIED. Cycle CLOSED (D-316). |
| Maint maint-2026-07-01: detection | DONE | 6 parallel sweeps complete; findings documented in maintenance-log.md |
| Maint maint-2026-07-01: doc PRs | DONE | PR #349 (9 stale comments, squash b451c481); PR #350 (docs+ADR-011, squash 3a60317) merged |
| Maint maint-2026-07-01: stories drafted | DONE | STORY-148/149/150 drafted; STORY-INDEX v3.10 (103 stories); IDX-003 reconciled |
| **Maint maint-2026-07-01: CLOSED** | **DONE (D-318)** | develop=3a60317. Idle. |

---

## Decisions Log

D-001..D-301: see `cycles/*/decisions-archive.md` (greenfield → feature-enip-v0.11.0)

| ID | Decision | Date |
|----|----------|------|
| D-302 | Dependabot PRs #325+#311 merged. develop `a2d8c13`. | 2026-06-29 |
| D-303 | Cycle `fix-tls-clienthello-frag` started. Full F1-F7. Maintenance paused. | 2026-06-29 |
| D-304 | F2 CONVERGED: 5 new BCs + 2 amended + VP-039 + ADR-011. | 2026-06-29 |
| D-305 | F2 APPROVED + F-EV-001 scope: BC-2.07.043 + VP-040. BC-INDEX v2.1, PRD v1.45. | 2026-06-29 |
| D-306 | F3 APPROVED. STORY-144..146; STORY-INDEX v3.6; HS-F4-001..012. Pre-F4 PASS. | 2026-06-29 |
| D-307 | STORY-144 MERGED PR #341 `0986e878`. SEC-001 DoS fixed. Wave 65 DONE. stories_delivered=92. | 2026-06-29 |
| D-308 | Session paused at STORY-145 mid-TDD (Red Gate `f60c0e0`, branch pushed). VP-INDEX corrected to v2.28. | 2026-06-30 |
| D-309 | STORY-145 MERGED PR #343 squash `d3d2e19`. Per-story convergence 5 passes, APPROVE. stories_delivered=93. | 2026-06-30 |
| D-310 | STORY-146 MERGED PR #344 squash `8b52046`. Per-story convergence multi-pass. stories_delivered=94. Wave 66 COMPLETE. | 2026-06-30 |
| D-311 | F4 holdout PASS (mean 0.904 ≥ 0.85; must-pass 8/8). HS-F4-001 verdict B+C. BC-2.07.038 v2.8. BC-INDEX v2.2. | 2026-06-30 |
| D-312 | F5 scoped adversarial CONVERGED. 60/60 BC-completeness. BC-2.07.038 v2.10. Re-anchor 7 BCs. BC-INDEX v2.3. | 2026-06-30 |
| D-313 | F6 targeted hardening IN PROGRESS (paused for session clear). Kani VP-039 (3 proofs, non-vacuous) + fuzz (1.9M execs clean). 13 mutation-gap tests remain. RUSTSEC-2026-0190 open. | 2026-06-30 |
| D-314 | F6 DONE. PR #345 merged (squash d7f0ef4): 12 mutation-gap tests mod f6_hardening — 100% real-gap kill (13 gaps closed; 2 provably-equiv survivors at tls.rs:950:59 documented). PR #346 merged (squash 52907bc): anyhow 1.0.102→1.0.103, RUSTSEC-2026-0190 cleared, cargo deny PASS. F6-MUTATION-GAPS-001 RESOLVED. RUSTSEC-2026-0190 RESOLVED. SEC-002/SEC-006 closed-by-design (mod f6_hardening themes 1+2+6 pin exact-MAX_BUF + clear-and-recover). develop=52907bc. F7 next. | 2026-07-01 |
| D-315 | Gitflow merge-settings alignment. Enabled allow_merge_commit=true repo-wide; main branch protection required_linear_history=false (accepts gitflow merge commits for releases + back-merges). develop keeps required_linear_history=true (squash-only, D-289 preserved). Refines D-289 + D-290. Root-caused B1: v0.11.0 squash into main left branches diverged; recurred because back-merge was skipped and squash-into-main prevented shared ancestry. | 2026-07-01 |
| D-316 | Cycle fix-tls-clienthello-frag CLOSED / CONVERGED. Released v0.11.1 (PR #347 gitflow merge into main `4e2b285`; tag `v0.11.1` object `e8a8a2d4`; GH Release published, 4 assets, NOT crates.io per D-300). Back-merged to develop PR #348 squash `ba6fbd8`. Both at 0.11.1 in sync. F6: Kani VP-039 3 non-vacuous proofs; fuzz 1.9M clean; 100% real-gap mutation kill (13/13; 2 dead-code survivors ADR-011). PRs #341/#343/#344/#345/#346/#347/#348. S-7.02 SATISFIED (STORY-147 PG-MUTANTS-JOBS-001; PG-BC-ANCHOR-VALIDATION-001 + DF-KANI-NONVACUITY-001-PROPTEST-GAP justified-deferred). | 2026-07-01 |
| D-317 | Maintenance run maint-2026-07-01 STARTED. D-303 pause lifted. Sweeps: dep/supply-chain, security, code-quality/pattern, doc/comment-drift, spec/anchor-drift, performance (6 total; UI/design-drift skipped — CLI only). develop @ ba6fbd8, v0.11.1. Log: `.factory/cycles/maint-2026-07-01/maintenance-log.md`. | 2026-07-01 |
| D-318 | maint-2026-07-01 COMPLETE. 2 doc cleanup PRs merged (#349 squash b451c481 — 9 stale RED-tense/todo!() comments; #350 squash 3a60317 — README ENIP+TLS-reassembly docs + ADR-011 promoted to docs/adr/0011 + CLAUDE.md ADR list 0010+0011). develop=3a60317. SEC-005/006 (ENIP on_flow_close unwired + DNP3 flow-map unbounded) → STORY-148 (E-20, 5 pts). Perf regression PERF-001/002/003-005 + benchmark gap → STORY-149 (E-11, 5 pts). TLS-DRAIN-DUP-001 (~220-line C2S/S2C duplication) → STORY-150 (E-11, 5 pts). Spec/anchor drift BC-ANCHOR-DRIFT-OUTOFCYCLE-001 expanded (12 stale sites, exact fixes captured), ARCH-INDEX-COUNT-DRIFT-001, TLS-SUMMARIZE-MAPTYPE-001, SEC-004/007, SEC-001-ENIP, MAINT-SC-001 deferred to backlog. IDX-003 total_points reconciled 656→659 (STORY-121 3 pts never added at v2.0). Audit/deny/pins clean. 0 STALE input-hashes (STORY-148/149/150 have inputs:[]). | 2026-07-01 |
| D-319 | Session paused for clear at 2026-07-01; durable resume checkpoint written. Pipeline at rest, no active cycle. | 2026-07-01 |

---

## Skip Log

| Step | Justification |
|------|---------------|
| crates.io publish (v0.11.0) | Human declined at D-300 — not published |
| Holdout formal eval HS-110..122 | Deferred post-release per D-267; 10/13 behaviors covered by unit tests |
| DTU creation | Not required (passive analyzer; no external service calls) — D-dtu-assessment 2026-05-20 |

---

## Blocking Issues

*None.*

---

## Open Items / Backlog (DF-VALIDATION-001-gated unless noted)

| ID | Summary | Priority | Status |
|----|---------|----------|--------|
| PG-MUTANTS-JOBS-001 | `cargo mutants --jobs 8` masks survivors. | MEDIUM | **CODIFIED → STORY-147** (draft, E-11, 3 pts) |
| SEC-005 + SEC-006 | ENIP on_flow_close unwired (CWE-400 DoS); DNP3 flow-map no cap. | MEDIUM | **→ STORY-148** (E-20, 5 pts, draft) |
| PERF-001/002 + BENCHMARK-GAP-001 | TLS carry-path +10.3% regression; HashMap + Vec alloc hotspots; no fragmented-handshake fixture. | HIGH | **→ STORY-149** (E-11, 5 pts, draft) |
| TLS-DRAIN-DUP-001 | ~220-line C2S/S2C drain-loop duplication in tls.rs. | MEDIUM | **→ STORY-150** (E-11, 5 pts, draft) |
| BC-ANCHOR-DRIFT-OUTOFCYCLE-001 | 12 stale tls.rs anchor sites; exact fixes in maintenance-log.md. | LOW | Deferred — next sweep or fold into STORY-150 |
| ARCH-INDEX-COUNT-DRIFT-001 | SS-11 34→35, SS-16 15→16; SS-sum 334→336. | LOW | Deferred — next sweep |
| TLS-SUMMARIZE-MAPTYPE-001 | BC-2.07.043 PC-4 HashMap vs impl BTreeMap; VP-040 Sub-D wording. | LOW | Deferred — spec-only gap |
| SEC-004 + SEC-007 | 7+ counter `+= 1` → saturating_add; clippy hygiene MQ-003/004/005. | LOW | Deferred — trivial PR candidate |
| PG-BC-ANCHOR-VALIDATION-001 | No automated anchor validation; 12 stale sites maint-2026-07-01. | LOW | Deferred — STORY-091 tooling candidate |
| DF-KANI-NONVACUITY-001-PROPTEST-GAP | No proptest/unit analog for DF-KANI-NONVACUITY-001. | LOW | Justified deferral — next Kani VP |
| SEC-001-ENIP | Unsafe split-borrow enip.rs `on_data`. | MEDIUM | v0.12.0 candidate |
| TLS-FILLBUF-PUBLIC-SEAM-001 + MAINT-SC-001 | fill_buf_for_testing seam (W7.1); indicatif patch + 41 transitive updates; 8 stale deny.toml entries. | LOW | W7.1 backlog / optional dep-refresh |

Detail: `cycles/feature-enip-v0.11.0/decisions-archive` + `cycles/maint-2026-07-01/maintenance-log.md`.

---

## Session Resume Checkpoint

**PIPELINE AT REST — no active cycle. Session paused for clear at 2026-07-01. Safe to clear and resume.**

- **Ground truth:** develop=`3a60317` (full `3a60317965e62bef9895e857c8a26fc3b8d03ad0`), main=`4e2b285` (full `4e2b28529ae196785ce6a0baed522b9939f929ea`, v0.11.1 released + tagged). factory-artifacts HEAD: run `git -C .factory log -1 --format='%h %s'`. No open PRs. No stray worktrees (main checkout [develop] + .factory [factory-artifacts] only).
- **Last completed work:** cycle fix-tls-clienthello-frag CLOSED (v0.11.1 released, D-316); gitflow merge-settings aligned (D-315); maintenance run maint-2026-07-01 COMPLETE (D-318 — 2 doc PRs #349/#350 merged, 3 follow-up stories STORY-148/149/150 drafted).
- **RESUME PROCEDURE (BLOCKING, in order):** (1) run `vsdd-factory:factory-worktree-health` — PASS required; (2) read `.factory/STATE.md` (this file); (3) verify git ground truth (origin/develop=`3a60317`, origin/main=`4e2b285`, no open PRs, no stray worktrees); (4) pipeline is idle — await human direction. No in-flight work to resume.
- **RECOMMENDED NEXT CANDIDATES (not started; await human):** STORY-148 (SEC-005/006 ENIP on_flow_close DoS — highest value), STORY-149 (TLS carry perf recovery), STORY-150 (TLS drain DRY refactor + Kani re-run); plus deferred backlog in `cycles/maint-2026-07-01/maintenance-log.md` (12 spec anchor sites, ARCH-INDEX-COUNT-DRIFT-001, TLS-SUMMARIZE-MAPTYPE-001, SEC-004/007 counter hygiene, SEC-001-ENIP v0.12.0, dep-refresh). Optional: session-reviewer for this run.

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001.

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Not on crates.io (D-300). Squash-only on develop (D-289). Branch protection (D-290/D-315).
- Cycle `fix-tls-clienthello-frag` CLOSED (D-316). maint-2026-07-01 CLOSED (D-318). Idle.
