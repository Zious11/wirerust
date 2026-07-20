---
document_type: pipeline-state
level: ops
version: "2.0"
producer: state-manager
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: wirerust
mode: steady-state
phase: "steady-state"
status: active
current_step: "Wave-84 in progress: STORY-147 DELIVERED (D-481); next STORY-166 per-story delivery (await human go); trajectory-tail →0→0→0→0"
current_cycle: "wave-084"
pipeline: ACTIVE
timestamp: 2026-07-20T03:06:00Z
released_version: v0.13.0
released_at: "2026-07-18"
release_tag: v0.13.0
release_tag_object: 03f35e4f0499dde0bcdb7a79dff9844ec57f1cdb
release_commit: 67a06b6f82654d2af79d023b15ac56ab03182ffd
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.13.0
prior_released_version: v0.12.1
prior_released_at: "2026-07-13"
main_head: 67a06b6f82654d2af79d023b15ac56ab03182ffd
develop_head: f0cb7374e51ed486cf72ef3ca1694be24169815a
cargo_version_main: "0.13.0"
cargo_version_develop: "0.13.0"
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
stories_delivered: 114
story_index_version: "v3.81"
total_stories: 132
story_index_note: "132 stories / 84 waves / 775 pts. v3.81 (2026-07-20): STORY-147 DELIVERED (D-481, PR #421 f0cb7374 squash-merged, human-executed under DF-MERGE-AUTH-CLASSIFIER-001); status ready→delivered; wave-84 Delivery Progress row (1/3 DELIVERED); stories_delivered 113→114. No points/story/wave totals changed. v3.80 (2026-07-19): STORY-147 title cell updated to '.cargo/mutants.toml Timeout Floor' (title-only cascade from STORY-147 v2.1→v2.2 spec-route remediation, Step-4.5 adversarial findings F-S147P1-002/-004/-005; no points/status/wave/epic change). v3.79 (2026-07-19): E-11 upstream re-scope burst #2 (D-480, human-approved) — STORY-091 superseded-OBSOLETE (no filing; delivered-by-drift via bin/validate-citations STORY-164 + STORY-166 symbol-at-line assertion); STORY-121/143/155 superseded (upstream drbothen/vsdd-factory #582/#695(NEW)/#290); STORY-147 v2.0 re-scoped local SPLIT survivor (engine half →#654; 3→2 pts). E-11 67→66; total_points 776→775. WAVE-84 OPENED: STORY-166+STORY-176+STORY-147v2 (7 pts, draft→ready). v3.78 (2026-07-19): STORY-175/177/178/179 superseded (upstream #690/#494/#461/#686/#682/#305/#655/#396); STORY-176 v2.0 local survivor 2 pts; E-11 68→67. See cycles/feature-iec104/ + planning/e11-stale-draft-disposition-plan.md for full history."
bc_index_version: "v2.34"
vp_index_version: "v2.46"
arch_index_version: "v2.19"
prd_version: "v1.57"
epics_version: v2.1
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-07-11
maintenance_started_at: "2026-07-11"
maintenance_completed_at: "2026-07-11"
maintenance_prior_run: maint-2026-07-09
---

<!--
  STATE.md SIZE BUDGET (per D-421(c)):
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 279 = 221 (dual-margin form). 279 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**STORY-147 DELIVERED (2026-07-20, D-481). PR #421 squash-merged to develop f0cb7374e51ed486cf72ef3ca1694be24169815a (human-executed merge, 2026-07-20T02:40:53Z, explicit per-PR authorization; DF-MERGE-AUTH-CLASSIFIER-001 satisfied). Feature branch feature/STORY-147-mutation-testing-defaults deleted (remote + local); worktree .worktrees/STORY-147 removed. CI 13/13 (Semantic PR check recovered after a GitHub-declared Minor Service Outage delayed it ~2h). Dual pr-reviewer APPROVE; security CLEAN. Step-4.5 adversary CONVERGED P6/P7/P8 (8 passes; Pass-1 F-S147P1-002 caught a placebo config — repo-root `mutants.toml`/`jobs` key silently never read by cargo-mutants; execution-verified pivot to `.cargo/mutants.toml` `minimum_test_timeout=300`); spec v2.1→v2.8. STORY-INDEX v3.80→v3.81 (status ready→delivered; wave-84 Delivery Progress row 1/3 DELIVERED). stories_delivered 113→114. Evidence artifacts at `.factory/code-delivery/STORY-147/` (committed f2b5dcfe). Process-gaps ledgered for cycle-close: stale-inline-version-marker recurrence, sub-agent message-routing breakage (relay-through-orchestrator workaround; also caused security-review.md artifact backfill f2b5dcfe), burst-log template understatement. Wave-84 IN PROGRESS: STORY-166 and STORY-176 remain ready, un-started, no dependency edges among the three. Resume: STORY-166 per-story delivery next (await human go). trajectory-tail →0→0→0→0**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); **RELEASED v0.13.0 (D-473, 2026-07-18). F1→F7 CONVERGED; CYCLE CLOSED (D-475, 2026-07-18): S-7.02 SATISFIED, 9 PGs → STORY-175..179 (12 pts), B-001/B-002 FIXED (PRD v1.57 + BC-2.19.002 v1.3), PR #419 82ad2ed merged. D-477: STORY-175/177/178/179 codification VEHICLE CHANGED to upstream (see D-477). D-480: E-11 disposition burst #2 — STORY-091/121/143/155 superseded (upstream-routed or OBSOLETE); STORY-147 v2.0 local survivor. WAVE-84 OPENED (STORY-166/176/147v2, 7 pts, all product-local). D-481: STORY-147 DELIVERED (PR #421 f0cb7374, 2026-07-20) — wave-84 1/3 DELIVERED; STORY-166/STORY-176 remain ready. Pipeline ACTIVE.** |
| Version | 0.13.0 (released 2026-07-18; main=67a06b6; develop=f0cb7374 — D-481 STORY-147 PR #421 squash-merged (2026-07-20); DRIFT-BACKMERGE-SQUASH-001 retained) |
| Main HEAD | `67a06b6f82654d2af79d023b15ac56ab03182ffd` |
| Develop HEAD | `f0cb7374e51ed486cf72ef3ca1694be24169815a` — D-481 STORY-147 PR #421 squash-merged (2026-07-20); DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 |
| Stories | 114 delivered / 132 total (STORY-INDEX v3.81, dep-graph v3.9, 775 pts) |
| **Last Updated** | 2026-07-20 — STORY-147 DELIVERED (D-481, PR #421 f0cb7374 squash-merged); wave-84 1/3 DELIVERED. STORY-INDEX v3.81; stories_delivered 114. Pipeline ACTIVE. develop=f0cb7374. trajectory-tail →0→0→0→0 |

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0–7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0..v0.11.0 | RELEASED | Details: cycles/ subdirs |
| Maintenance maint-2026-06-22 + maint-2026-07-01 | COMPLETE | Details: cycles/ subdirs |
| Feature fix-tls-clienthello-frag (F1–F7) | CONVERGED/CLOSED | v0.11.1 released (D-316); see cycles/fix-tls-clienthello-frag/ |
| Feature feature-protocol-coverage E-21 (F1–F7) | CONVERGED/CLOSED | v0.11.2 released (D-382); see cycles/feature-protocol-coverage/ |
| Out-of-cycle + v0.11.3..v0.11.5 + maint-2026-07-06 | COMPLETE/RELEASED | D-383..D-393 (exhaustive); see cycles/history/decision-log-archive.md |
| Wave 70 (STORY-149) + maint-2026-07-08 + issue triage | COMPLETE | D-394..D-407 (exhaustive); see cycles/history/decision-log-archive.md |
| Wave 71 (v0.12.0: STORY-150/156/157) | CLOSED (D-404) | PRs #378-381; develop=b642c0f; 3/3 adversary; S-7.02 SATISFIED |
| Wave 72 (v0.12.0: STORY-158/159/160/161) | CLOSED (D-416) | PRs #387-391+gate-fix #391; gate all-green; S-7.02 SATISFIED |
| v0.12.0 RELEASED | RELEASED 2026-07-10 | PR #394 fedcea4; BREAKING JSON BC-2.11.036/037; histories reunified |
| Wave 73: STORY-162/163 | CLOSED (D-428) | PRs #395 b5e1e15 + factory-only D-427; adversary 6-pass streak 3/3 |
| Maintenance maint-2026-07-11 | COMPLETE (D-429) | PR #396 6779be6; 17 findings; ROUTES B/C DEFERRED |
| Wave 74: STORY-164 | CLOSED (D-432) | PR #397 d6e3be8; adversary 13-pass streak 3/3; ROUTE-W74-DEFERRED |
| Wave 75: STORY-165 | CLOSED (D-435) | PR #398 fa646ed; adversary 7-pass streak 3/3; S-7.02: STORY-166 drafted |
| v0.12.1 RELEASED | RELEASED 2026-07-13 | PR #399 fedcea4 main; back-merge #400 7b11b83; DRIFT-BACKMERGE-SQUASH-001 |
| feature-iec104 — F1 (delta-analysis) | DONE/APPROVED (2026-07-14) | 30 new BCs; SS-19; ADR-013; VP-044..047 |
| feature-iec104 — F2 (spec-evolution) | **APPROVED (D-439) CLOSED** | BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56; Option<u16>; MITRE ics-attack-19.1 confirmed |
| feature-iec104 — F3 (incremental-stories) | **APPROVED (D-440)** | STORY-167..174 (8 stories/36 pts/waves 76–83); dep-graph v3.9 (137 edges) |
| feature-iec104 — F4 (delta-implementation) | **COMPLETE (D-463)** | Waves 76–83 DELIVERED (D-441/443/445/447/448/455/458/463): STORY-167..174 PRs #401-409. 8/8. trajectory-tail →0→0→0→0 |
| feature-iec104 — STORY-173 pre-merge LOW fix burst | **COMPLETE** | 3 LOWs FIXED pre-merge (human approved all 3): flows_analyzed counter; packets_analyzed counter; is_valid_iec104_frame doc gate role. BC-INDEX v2.32→v2.33 (3ec6ac1). 2602→2604 tests. Triggered fresh A/B/C re-convergence. |
| **Other granular F4 rows (D-451 burst, per-story adversary STORY-172/173/174, wave gates 76–83, FIX-P4-001, FIX-F5-001..004)** | ARCHIVED | `cycles/feature-iec104/phase-progress-archive.md` |
| feature-iec104 — F5 (scoped adversarial) | **CONVERGED (D-468)** | 5 rounds; pass-5 NITPICK_ONLY (0 CRIT/HIGH/MED; 1 LOW non-blocking); code frozen R2 (9c5aa9a); BC-completeness 31/31 + canonical-frame 19 byte-exact clean |
| feature-iec104 — F6 (targeted-hardening) | **PASS (D-469)** | Kani/fuzz/mutation/audit/regression all green; VPs re-run post-fix on b36b884 |
| feature-iec104 — F7 (delta-convergence) | **CONVERGED (D-470)** | 5/5 dims PASS; holdout 0.99 RELEASE-READY |
| E2E IEC-104 coverage (human-directed, post-F7) | **MERGED (D-471)** | PR #416 0b65e8e; 4 fixtures + analyzer-level real-pcap test |
| v0.13.0 RELEASED | RELEASED 2026-07-18 | PR #417 67a06b6 main + tag v0.13.0 + GH release 4 assets; back-merge #418; IEC-104 F1-F7 |
| **feature-iec104 cycle-close (S-7.02)** | **CLOSED (D-475)** | 9 PGs → STORY-175..179 (12 pts, E-11 epic); B-001/B-002 FIXED; PR #419 82ad2ed; STORY-INDEX v3.77; 132 stories / 777 pts. **D-477 annotation: STORY-175/177/178/179 codification VEHICLE CHANGED to upstream drbothen/vsdd-factory issues/comments per D-477 (D-475 history preserved — vehicle changed, not rewritten); STORY-176 v2.0 local survivor (2 pts).** |
| **Wave 84 (E-11 mini-wave: STORY-166/176/147v2)** | **OPENED (D-480); DELIVERY IN PROGRESS (1/3 DELIVERED)** | Plan gate approved (human, 2026-07-19); 7 pts, all product-local; no dependency edges among the three stories. STORY-147 DELIVERED (D-481, PR #421 f0cb7374, 2026-07-20; 8-pass Step-4.5 adversary CONVERGED P6/P7/P8; dual pr-reviewer APPROVE; security CLEAN; CI 13/13). STORY-166 + STORY-176 remain ready, un-started. |

---

## Convergence Status

Per-story F4 convergence details archived to `cycles/feature-iec104/convergence-trajectory.md`.
F5 phase-level trajectory: 5 rounds, code frozen R2, `5H/M→2M→1H→1M→1L(NB)` — CONVERGED (D-468).

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | **CLOSED (D-475, 2026-07-18)** — v0.13.0 RELEASED (D-473); F1→F7 CONVERGED (D-470); S-7.02 SATISFIED: 9 PGs → upstream drbothen/vsdd-factory per D-477 (STORY-175/177/178/179 superseded); STORY-176 v2.0 + STORY-166 local survivors; B-001/B-002 FIXED; Pipeline ACTIVE | develop (f0cb7374) |
| wave-084 (E-11 mini-wave) | **OPENED (D-480, 2026-07-19); DELIVERY IN PROGRESS (1/3 DELIVERED)** — STORY-147 DELIVERED (D-481, PR #421 f0cb7374, 2026-07-20; 8-pass adversary CONVERGED P6/P7/P8); STORY-166 v1.2 + STORY-176 v2.1 remain ready, un-started; 7 pts, all product-local; plan gate approved (human); no dependency edges among the three stories | develop (f0cb7374, D-481 STORY-147 merge) |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **STORY-147 DELIVERED (2026-07-20, D-481). PR #421 squash-merged to develop f0cb7374e51ed486cf72ef3ca1694be24169815a (human-executed, 2026-07-20T02:40:53Z, explicit per-PR authorization; DF-MERGE-AUTH-CLASSIFIER-001 satisfied). Feature branch + worktree .worktrees/STORY-147 removed. CI 13/13 (Semantic PR recovered after GitHub-declared Minor Service Outage delayed it ~2h). Dual pr-reviewer APPROVE; security CLEAN. Step-4.5 adversary CONVERGED P6/P7/P8 (8 passes; Pass-1 caught placebo config — repo-root mutants.toml/jobs key never read by cargo-mutants; execution-verified pivot to .cargo/mutants.toml minimum_test_timeout=300); spec v2.1→v2.8. STORY-INDEX v3.80→v3.81 (status ready→delivered; wave-84 row 1/3 DELIVERED). stories_delivered 113→114. Evidence artifacts at .factory/code-delivery/STORY-147/ (committed f2b5dcfe). Process-gaps ledgered for cycle-close: stale-inline-version-marker recurrence, sub-agent message-routing breakage (relay-through-orchestrator workaround; also caused security-review.md artifact backfill f2b5dcfe), burst-log template understatement.** | **DELIVERED (D-481)** | develop=f0cb7374. Resume: STORY-166 per-story delivery next (await human go). trajectory-tail →0→0→0→0 |
| **STORY-147 Step-4.5 adversarial review CONVERGED (2026-07-19). 8 passes; clean streak P6/P7/P8 (BC-5.39.001 SATISFIED); final code tip 7ff84f56 (unchanged P6→P8, doc/comment-only fixes). Pass-1 F-S147P1-002 (HIGH, corroborated by F-S147P1-004/-005) caught a placebo config: repo-root `mutants.toml` `jobs=1` is silently never read by cargo-mutants (only `.cargo/mutants.toml` is read by default) and `jobs` is not a valid Config field (deny_unknown_fields — would abort every run); confirmed by execution probes against installed cargo-mutants 27.0.0 plus 27.1.0 docs/source research. Pivoted design to a `.cargo/mutants.toml` `minimum_test_timeout=300` timeout floor. Story retitled v2.1→v2.2 (".cargo/mutants.toml Timeout Floor"); spec evolved v2.1→v2.8 across 8 passes (STORY-INDEX v3.79→v3.80, title-cascade only, no pts/status change). All substantive findings adversary-verified fixed in Part A chains; 1 non-blocking LOW residual (F-S147P8-001, doc-only) carried for gate ratification. Details: cycles/wave-084/STORY-147/convergence-report.md + adversary-convergence-state.json. Step 5 demo evidence dispatched.** | **CONVERGED (Step-4.5)** | STORY-147 v2.8; STORY-INDEX v3.80. Resume: STORY-147 Step 5 demo evidence in-flight. trajectory-tail →0→0→0→0 |
| **Housekeeping burst (2026-07-19, folded into current_step — no new D-number). sprint-state.yaml registered wave-84 entries for STORY-147/STORY-166/STORY-176 (status: pending, blocked_by: [] — no dependency edges; STORY-147 branch/worktree fields populated for in-flight dispatch). Story-writer ride-alongs landed: STORY-147 v2.0→v2.1 + STORY-176 v2.1→v2.2 (both add a "Token Budget Estimate" section per per-story-delivery.md Token Budget Check; no AC or scope content change). input-hash re-baselined STORY-175/176/177/178/179 (canonical tool only) — cause: all five list `.factory/STATE.md` in `inputs:`, re-staled by the D-480 STATE.md commit (stored 62d13e0 vs computed 072239d before this re-baseline); 2nd re-baseline for this cluster in one day. Final scan MATCH=132 STALE=0. Wave-84 delivery IN PROGRESS: STORY-147 Step 2 (stubs) dispatched.** | **COMPLETE (housekeeping)** | Resume: STORY-147 Step 2 (stubs) in-flight. trajectory-tail →0→0→0→0 |
| **D-480 E-11 DISPOSITION BURST DELIVERED (2026-07-19, resumed from D-479 pause; all items human-approved). DF-VALIDATION-001 research pass (research-agent) over 5 stale E-11 drafts (STORY-091/121/143/147/155) → disposition plan (planning/e11-stale-draft-disposition-plan.md, dupe-checked against D-477's 465-issue corpus). Upstream filings (github-ops, redaction-verified): NEW issue drbothen/vsdd-factory#695 (STORY-143, x-ref #580); evidence comments on #582 (STORY-121, comment-5016995736, x-ref #396), #654 (STORY-147 engine half, comment-5016997563), #290 (STORY-155, comment-5016998745, x-ref #600). STORY-091: no filing (OBSOLETE — verification core delivered by bin/validate-citations STORY-164 + STORY-166 symbol-at-line assertion; residual --scan layer represented upstream #622/#603/#396). Story-writer burst: STORY-091/121/143/155 status draft→superseded; STORY-147 v2.0 SPLIT survivor "Repo-Local Mutation-Testing Defaults: mutants.toml (jobs=1) + CLAUDE.md Guidance" (retitled v2.2) (3→2 pts, AC-147-001..004, engine half →#654). WAVE-84 OPENED (plan gate approved, human): STORY-166 v1.2 + STORY-176 v2.1 + STORY-147 v2.0, 7 pts, all product-local, draft→ready, no dependency edges. STORY-INDEX v3.78→v3.79 (7 rows updated; E-11 67→66 [only STORY-147's -1 delta]; total_points 776→775; total_waves 83→84; arithmetic verified: wave-scheduled 747 + exclusions 28 = 775). input-hash final scan MATCH=132 STALE=0; re-baselined STORY-164/165 (4th re-baseline — STORY-INDEX-IN-INPUTS-CHURN), STORY-176 (edited), STORY-175/177/178/179 (pre-existing drift, benign). Incidental fixes: STORY-091 unescaped-pipe table bug + missing template fields; STORY-143/155 body Status-line loci agreement corrected.** | **COMPLETE (D-480)** | STORY-INDEX v3.79. Wave-84 OPENED, 3 ready stories. trajectory-tail →0→0→0→0 |
| **D-479 SESSION WRAP (2026-07-19). Human-requested pause at clean milestone post-D-478 dep-soak. Sessions D-475..D-478 (exhaustive) delivered (feature-iec104 CLOSED; v0.13.0 released; dep-soak PR #420 merged). No in-flight work. Pipeline PAUSED. Resume: /vsdd-factory:next-step.** | **PAUSED (D-479)** | steady-state post-dep-soak. trajectory-tail →0→0→0→0 |

## Decisions Log

| ID | Decision | Date |
|----|----------|------|
| D-001..D-301 (exhaustive). Greenfield through feature-enip-v0.11.0; see cycles/*/decisions-archive.md for full range. | — | — |
| D-302..D-436 (exhaustive). Fix-tls through feature-protocol-coverage through v0.12.1; see cycles/history/decision-log-archive.md for full range. | — | — |
| D-437..D-458 (exhaustive). feature-iec104 F1 engine triage through F4 delivery; see cycles/feature-iec104/decisions-archive.md for full range. | — | — |
| D-460 | Session RESUMED (human-approved, 2026-07-16). Worktree health PASS; develop=084ff93 verified; no story worktrees; only open PR is external #407 (deferred post-wave-83 by human). STORY-174 wave-83 begins with research-agent validation. | 2026-07-16 |
| D-461 | STORY-174 pre-delivery realignment COMPLETE (research-validated, human-approved 2026-07-16). DF-VALIDATION-001 research 2 passes (all HIGH confidence): PG-REDGREEN-COMMENT-CLEANUP VALID-INCLUDE → AC-174-008; F-172-003 VP-045 vacuity VALID-INCLUDE → AC-174-002 amended; IEC104-FINDING-DIRECTION-001 VALID-DEFER → pre-F5 fix-PR (D-464). STORY-174 v2.0 input-hash de9d14e→27c86aa. STORY-INDEX v3.72→v3.73. | 2026-07-16 |
| D-462 | STORY-174 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 7 passes; streak P5/P6/P7; final HEAD e62701f; 2600+/0 tests. Story v2.2; STORY-INDEX v3.75. Kani non-vacuity 3/3 every pass. Mutation 117/122=95.9%. Fuzz 1.35M execs clean. PG-GATE-VOCAB-BLINDSPOT filed. | 2026-07-16 |
| D-463 | STORY-174 DELIVERED (PR #409 547deba squash-merged, 2026-07-17, human-direct merge after TWO subagent-classifier halts). Per-story adversarial CONVERGED D-462. CI 13/13. 8/8 IEC-104 stories delivered. stories_delivered 112→113. Wave-83 gate SATISFIED. | 2026-07-17 |
| D-464 | FIX-P4-001 DELIVERED (PR #410 7e95f71 squash-merged, 2026-07-17, human-executed merge). IEC104-FINDING-DIRECTION-001 RESOLVED — 10 emit sites direction: Some(...); 11 direction-assertion tests; additive `direction` JSON key; holdout-expectations sweep COMPLETE. CI 13/13. develop=7e95f71. | 2026-07-17 |
| D-465 | feature-iec104 F5 scoped adversarial OPENED (2026-07-17). Round 1 @ 7e95f71: BC-completeness 31/31 PASS; canonical-frame 19 invariants byte-exact; 1H+4M findings → FIX-F5-001. | 2026-07-17 |
| D-466 | FIX-F5-001 DELIVERED (PR #411 9c5aa9a squash-merged, 2026-07-17). source_ip + timestamp enrichment; 10 red-first tests; 9 stale-prose sites scrubbed; additive JSON keys in CHANGELOG; holdout-expectations sweep COMPLETE. CI 13/13. develop=9c5aa9a. | 2026-07-17 |
| D-467 | F5 Rounds 2-3 (2026-07-17). R2 code CONVERGED + 2 MEDIUM doc findings → FIX-F5-002 (#412 b356545). R3: F-B1 HIGH fabricated FIX-P4-001 demo-evidence → FIX-F5-003 (PG-DEMO-JSON-FABRICATION root cause confirmed). | 2026-07-17 |
| D-468 | feature-iec104 F5 CONVERGED (2026-07-17). 5 rounds. FIX-F5-002/003/004 DELIVERED. R5 NITPICK_ONLY (0 CRIT/HIGH/MED; 1 LOW non-blocking TypeID-45 prose). BC-completeness 31/31 + canonical-frame 19 byte-exact. develop=b36b884. | 2026-07-17 |
| D-469 | feature-iec104 F6 targeted hardening PASS (2026-07-17). Kani/fuzz/mutation/audit/regression all green. cargo-mutants iec104.rs 95.9%. No BLOCKERs. | 2026-07-17 |
| D-470 | feature-iec104 F7 delta convergence CONVERGED (2026-07-17). 5/5 dims PASS; holdout 0.99 RELEASE-READY. RELEASE HELD (human direction) — v0.13.0 cut deferred. | 2026-07-17 |
| D-471 | E2E IEC-104 coverage merged (PR #416 0b65e8e, 2026-07-17, human-executed merge). 4 real captures + tests/iec104_e2e_real_pcaps_tests.rs. CI 13/13. | 2026-07-17 |
| D-472 | PR #407 security-triaged (2026-07-18): SAFE-WITH-CHANGES. DEFERRED by human — governance decision pending. Triage: .factory/planning/pr-407-security-triage.md. | 2026-07-18 |
| D-473 | v0.13.0 RELEASED (2026-07-18). Release PR #417 67a06b6 main; tag v0.13.0; GH release 4 assets; back-merge #418 af3ecbd develop. 13 commits released. DRIFT-BACKMERGE-SQUASH-001 retained. | 2026-07-18 |
| D-474 | SESSION WRAP (2026-07-18). Human-requested pipeline pause at clean milestone post-v0.13.0 release. All session PRs (#409-418) merged. No in-flight work. Factory artifacts committed to factory-artifacts. Pipeline PAUSED. | 2026-07-18 |
| D-475 | feature-iec104 CYCLE-CLOSE (2026-07-18). S-7.02 checklist SATISFIED: 9 process-gaps codified into 5 draft stories STORY-175..179 (E-11 epic, 12 pts; STORY-INDEX v3.77); B-001/B-002 doc nits FIXED (PRD v1.57, BC-2.19.002 v1.3 + title cascade, BC-INDEX v2.34); STORY-167 v1.1 AC propagation; IEC104-DEMO-TYPEID45-MISLABEL DELIVERED via docs PR #419 82ad2edd12ad1f9dad61a03a4760d4112d45ccc2 squash-merged to develop (human-executed merge; pr-reviewer APPROVE 0 findings; CI 13/13; step-8 halt per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER, human-direct merge — pattern reconfirmed); STORY-164/165 input-hashes re-baselined BENIGN (canonical tool; 132/0 scan); DRIFT-SPRINT-STATE-FIELD-FORM-001 pre-resolved (sprint-state.yaml already absent); mutants.out residue deleted. develop FF to 82ad2edd12ad1f9dad61a03a4760d4112d45ccc2. feature-iec104 declared CLOSED. Pipeline ACTIVE (resumed from D-474 pause). | 2026-07-18 |
| D-476 | PR #414 ADOPTED (2026-07-19). ArcavenAE fork ci/scorecard-guard squash-merged to develop (fcd57dcbd8b13074ffb57086f5f179dc30f1d026; human-executed 2026-07-19T01:54:40Z). Security-triaged SAFE-WITH-CHANGES (triage: .factory/planning/pr-414-security-triage.md; F1 CWE-494 RESOLVED-CLEAN: all 4 SHA↔tag mappings MATCH via GitHub API, harden-runner v2.19.4 NOT AFFECTED by any advisory — dated section appended to triage file 2026-07-19). CI 13/13 SUCCESS incl. action-pin-gate. Adds .github/workflows/scorecards.yml; workflow inert until SCORECARD_ENABLED=true set. F2 (CWE-200) and enablement runbook noted in SCORECARD-ENABLEMENT-RUNBOOK carry-forward. PR #407 governance OPEN/UNAFFECTED (disjoint files, no overlap verified). NOTE: planning/vsdd-factory-upstream-issues.md rode along in this commit (d4d690b6); provenance = the github-ops issue dump prepared for the D-477 upstream-routing effort. | 2026-07-19 |
| D-477 | UPSTREAM-ROUTING (2026-07-19). E-11 process-gap codification redirected from local wirerust stories (STORY-175..179) to upstream drbothen/vsdd-factory. DF-VALIDATION-001 research pass: 465 issues scanned, 33 bodies read (planning/upstream-codification-filing-plan.md incl. REDACTED section — human chose identifier-redacted publication). Filed NEW upstream issue #690 (validate-count-propagation E-11→"11" tokenizer false-positive; body redacted post-hoc) + 7 redacted evidence comments on #494/#461/#686/#682/#305/#655/#396. 2 confirmed duplicates no-action (#457, #637). STORY-175/177/178/179 → superseded (files retained, Disposition sections cite upstream URLs). STORY-176 v2.0 → local product survivor "Feature-IEC104 Cycle-Close: Local Gate + Tooling Hygiene Sweeps" (2 pts, 3 ACs: skeleton/seam gate tokens, input-hash post-delivery re-baseline protocol note, .gitignore mutants.out* glob). STORY-166 classified PRODUCT-LOCAL no-action (engine ACs already upstream at #638/#635 since wave-75). STORY-INDEX v3.78 (132 stories / 776 pts; E-11 68→67; wave TOTAL 120/740 unchanged). STORY-164/165 input-hashes re-baselined BENIGN (canonical tool; 132/0 scan; 3rd re-baseline in 2 days — STORY-INDEX-IN-INPUTS-CHURN drift item filed). | 2026-07-19 |
| D-478 | DEP-SOAK DELIVERED (2026-07-19). PR #420 "build(deps): soaked dependency bumps 2026-07-19" squash-merged to develop 492554642c7d4a3251df128789fd5f149fd2b0a7 (human-executed, 2026-07-19T18:01:50Z; per-PR explicit human instruction per DF-MERGE-AUTH-CLASSIFIER-001, D-417 precedent). Lockfile-only: 24 distinct version-pair changes / 26 version movements (hashbrown 2→1 consolidation; etherparse 0.20.3 direct dep; libc/log/memchr/indexmap/zerocopy et al., all soaked ≥8d per D-417 protocol); 18 obsolete WASM-tooling crate versions removed (getrandom@0.4 resolution change; deps 193→175). cargo audit 0 advisories + deny 4/4 clean. pr-reviewer APPROVE, PG-W74 row-verify 4/4. CI 13/13. Deferred: 17 not-yet-soaked candidates (eligible 2026-07-21..27: serde/clap/regex/syn/anyhow/etc.) + 4 soaked-but-blocked (js-sys/wasm-bindgen/web-sys via futures-* 0.3.33; shlex via cc 1.3.0). DEP-SOAK-FOLLOWUP-2026-07-27 carry-forward added. | 2026-07-19 |
| D-479 | SESSION WRAP (2026-07-19). Human-requested pause at clean milestone post-D-478 dep-soak. Sessions D-475..D-478 (exhaustive) delivered in this session: feature-iec104 CYCLE-CLOSE (D-475); PR #414 adopted (D-476); upstream-routing (D-477); dep-soak PR #420 merged (D-478). No in-flight work; no story worktrees; no adversarial loop; no abandoned sub-agents. Pipeline PAUSED by human direction. Pending human decisions: PR #407 governance; E-11 mini-wave scheduling (STORY-166 + STORY-176 v2.0); STORY-INDEX-IN-INPUTS-CHURN structural decision. DEP-SOAK-FOLLOWUP-2026-07-27 and SCORECARD-ENABLEMENT-RUNBOOK carry-forwards active. | 2026-07-19 |
| D-480 | E-11 DISPOSITION BURST DELIVERED (2026-07-19, resumed from D-479 pause; all items human-approved). DF-VALIDATION-001 research pass (research-agent) over 5 stale E-11 drafts (STORY-091/121/143/147/155) → disposition plan (planning/e11-stale-draft-disposition-plan.md, dupe-checked against D-477's 465-issue corpus). Upstream filings (github-ops, redaction-verified): NEW issue drbothen/vsdd-factory#695 (STORY-143, x-ref #580); evidence comments on #582 (STORY-121, x-ref #396), #654 (STORY-147 engine half), #290 (STORY-155, x-ref #600). STORY-091: no filing (OBSOLETE — verification core delivered by bin/validate-citations STORY-164 + STORY-166 symbol-at-line assertion). Story-writer burst: STORY-091/121/143/155 → superseded; STORY-147 → v2.0 SPLIT survivor (3→2 pts, engine half →#654). WAVE-84 OPENED: STORY-166 v1.2 + STORY-176 v2.1 + STORY-147 v2.0 (7 pts, all product-local, draft→ready, plan gate approved). STORY-INDEX v3.78→v3.79 (132/84/775 pts; E-11 67→66; arithmetic verified 747+28=775). input-hash final scan MATCH=132 STALE=0. Incidental fixes: STORY-091 table/template gaps; STORY-143/155 Status-line loci corrected. | 2026-07-19 |
| D-481 | STORY-147 DELIVERED (PR #421 f0cb7374 squash-merged 2026-07-20, human-executed under explicit per-PR authorization). 8-pass Step-4.5 adversary CONVERGED P6/P7/P8; Pass-1 caught placebo config (repo-root mutants.toml/jobs key never read by cargo-mutants; execution-verified pivot to .cargo/mutants.toml minimum_test_timeout=300); spec v2.1→v2.8; dual APPROVE; security CLEAN; CI 13/13 after GitHub Minor Service Outage delayed Semantic PR ~2h; stories_delivered 113→114. Process-gaps ledgered for cycle-close: stale-inline-version-marker recurrence, sub-agent message-routing breakage (relay-through-orchestrator workaround; also caused security-review.md artifact backfill f2b5dcfe), burst-log template understatement. | 2026-07-20 |

---

## Skip Log

| Step | Justification |
|------|---------------|
| crates.io publish (v0.11.0) | Human declined at D-300 — not published |
| Holdout formal eval HS-110..122 | Deferred post-release per D-267; 10/13 behaviors covered by unit tests |
| DTU creation | Not required (passive analyzer; no external service calls) — D-dtu-assessment 2026-05-20 |

---

## Blocking Issues

| ID | Issue | Severity | Owner | Resolution |
|----|-------|----------|-------|------------|
| (none) | No open blocking issues. | — | — | — |

---

## Drift Items

| ID | Summary | Source | Target |
|----|---------|--------|--------|
| DRIFT-BACKMERGE-SQUASH-001 | v0.12.1 back-merge PR #400 was squash-merged; v0.13.0 cut re-encountered this drift resolved-forward. Back-merge #418 also squash-merged per human choice. main (67a06b6) NOT ancestor of develop (f0cb7374). Trees ARE identical for released content — history-only divergence. DRIFT PERSISTS. | v0.12.1 release (D-436, 2026-07-13); re-encountered v0.13.0 (D-473, 2026-07-18) | resolve at a future release via true-merge back-merge if desired (human deferred; squash pattern retained D-473) |
| DRIFT-VP039-BC207038-TLS-TODO-001 | VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039 (TLS reassembly). Out of feature-iec104 scope. | feature-iec104 F2 review (D-438, 2026-07-14) | SS-07 TLS owner — next TLS maintenance sweep |
| STORY-INDEX-IN-INPUTS-CHURN | Stories listing STORY-INDEX.md as an input (at minimum STORY-164/165) re-stale on every index version bump; 4 re-baselines in 3 days (D-480 was the 4th — 2026-07-17, 2026-07-18, 2026-07-19×2). Separately, STORY-175..179 list `.factory/STATE.md` as an input, re-staling on EVERY factory commit — this housekeeping burst is the 2nd re-baseline for that cluster in one day (stored 62d13e0 vs computed 072239d across all five before re-baseline). Structural fix (remove index/STATE.md from inputs lists) awaits human decision, now covering both the STORY-INDEX.md-in-inputs cluster (164/165) and the STATE.md-in-inputs cluster (175..179). Related upstream discussion #672/#314. | D-477 (2026-07-19, #3) → D-480 (2026-07-19, #4) → housekeeping burst (2026-07-19, STATE.md cluster re-baseline #2) | Human decision: remove STORY-INDEX.md/STATE.md from affected story inputs lists — still pending after 4+2 re-baselines |

---

## Active Carry-Forwards

| ID | Summary | Target |
|----|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11 (human decision). | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review 1 NIT deferred from wave-74 gate (human-ratified); joins wave-75 NIT. | Next bin-touch PR |
| PERF-RERUN-001 | AC-149-003 quiescent re-run pending (load avg 52.57 at maint-2026-07-11; human deferred). | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred from maint-2026-07-11; next feature wave. | Next feature wave or maintenance |
| F3-handoff cleanup | F-F3P12-002 (STORY-151 pointer note), F-F3P13-002 (STORY-154 frontmatter SS-05), F-F3P17-001 (STORY-154 cross-layer trace). | F4 implementation per-story |
| SEC-001-S158 / SEC-002-S158 | CWE-22 LOW advisories in `bin/lint-cycle-artifact` (deferred until mandatory CI wiring). DF-VALIDATION-001-gated. | bin/lint-cycle-artifact CI wiring |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant, DEFERRED) TypeIDs 58–64 (timed control variants) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Evasion gap. DF-VALIDATION-001 required before filing any GitHub issue. | Follow-on detection story (new BC + detection arm) |
| PR-407-FORK-RELEASE-OPS | External ArcavenAE PR #407 security-triaged SAFE-WITH-CHANGES (D-472; triage at .factory/planning/pr-407-security-triage.md); DEFERRED — governance decision pending. Resume without re-running security review. | Governance decision when authorized |
| SCORECARD-ENABLEMENT-RUNBOOK | Before setting SCORECARD_ENABLED=true: document that publish_results:true publishes security-posture data to OpenSSF public API (F2, CWE-200, LOW); optional harden-runner bump v2.19.4→v2.20.0 via Dependabot (SHA bf7454d06d71f1098171f2acdf0cd4708d7b5920 available; Dependabot [github-actions ecosystem, 7d cooldown] expected to auto-PR the re-pin; manual re-pin only if no PR within ~48h of 2026-07-19). PR #414 ADOPTED (D-476). | Whenever scorecard is enabled |
| DEP-SOAK-FOLLOWUP-2026-07-27 | 17 not-yet-soaked crates become eligible 2026-07-21..27 (serde/clap/regex/syn/anyhow/etc.) + 4 soaked-but-blocked (js-sys/wasm-bindgen/web-sys via futures-* 0.3.33; shlex via cc 1.3.0). Run next soak sweep on/after 2026-07-27 to catch all in one pass. | Next maintenance run on/after 2026-07-27 |

---

## Session Resume Checkpoint

**STORY-147 DELIVERED (2026-07-20, D-481). PR #421 f0cb7374 squash-merged to develop; 8-pass Step-4.5 adversary CONVERGED P6/P7/P8; dual pr-reviewer APPROVE; security CLEAN; CI 13/13. Wave-84 1/3 DELIVERED — STORY-166/STORY-176 remain ready, un-started. Pipeline ACTIVE. trajectory-tail →0→0→0→0**

Prior checkpoints archived to `cycles/feature-iec104/session-checkpoints.md` and `cycles/wave-084/session-checkpoints.md`.

- **Date:** 2026-07-20. Position: wave-84 delivery IN PROGRESS (1/3 delivered), pipeline ACTIVE. Next step: STORY-166 per-story delivery (await human go); STORY-176 also ready, un-started; no dependency edges among the three.
- **Ground truth:** main = `67a06b6f82654d2af79d023b15ac56ab03182ffd` (v0.13.0); develop = `f0cb7374e51ed486cf72ef3ca1694be24169815a` (D-481 STORY-147 PR #421 squash-merged 2026-07-20); factory-artifacts = this burst commit. DRIFT-BACKMERGE-SQUASH-001 still applies.
- **In-flight work:** Wave-84 delivery IN PROGRESS (1/3 delivered). STORY-147 DELIVERED and closed (worktree + branch removed). No open factory PRs, no adversarial loop active. STORY-166/STORY-176 not started.
- **Pending human decisions:** (a) PR #407 governance (triage preserved at `planning/pr-407-security-triage.md` — do NOT re-run); (b) STORY-INDEX-IN-INPUTS-CHURN structural decision (remove STORY-INDEX.md/STATE.md from affected story inputs lists? now 4+2 re-baselines across the two clusters); (c) F-S147P8-001 non-blocking LOW residual (doc-only) — for gate ratification; (d) go-ahead for STORY-166 per-story delivery.
- **Dated follow-ups:** DEP-SOAK-FOLLOWUP-2026-07-27 (17 deferred + 4 blocked bumps eligible 2026-07-21..27); harden-runner v2.20.0 Dependabot re-pin watch (~48h from 2026-07-19, manual re-pin SHA `bf7454d06d71f1098171f2acdf0cd4708d7b5920` if absent) — both unchanged this burst.
- **Spec versions:** BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 / STORY-INDEX v3.81 / dep-graph v3.9 (137 edges).
- **Resume command:** `/vsdd-factory:next-step` (STORY-166 per-story delivery next, await human go).

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001.

---

## Historical Content

| Content | Location |
|---------|----------|
| **Decision Log D-302..D-436 (exhaustive)** | `cycles/history/decision-log-archive.md` (fix-tls through feature-protocol-coverage through v0.12.1) |
| **feature-iec104 decisions archive (exhaustive)** | `cycles/feature-iec104/decisions-archive.md` (D-437 through D-458: F1 engine triage through F4 delivery) |
| **Resolved Open Items (pre-feature-iec104)** | `cycles/history/open-items-archive.md` |
| **Resolved Carry-Forwards (feature-iec104)** | `cycles/feature-iec104/blocking-issues-resolved.md` (IEC104-FINDING-DIRECTION-001; F5R2-01/02/F-B1/F-B2; D-475 codified PGs) |
| **feature-iec104 cycle-close lessons** | `cycles/feature-iec104/lessons.md` (9 process-gaps [codified] → STORY-175..179 per D-475; D-477: vehicles changed to upstream — see lessons.md D-477 entry; D-478: dep-soak process lessons appended) |
| **feature-iec104 burst log (D-475 archived Current Phase Steps row)** | `cycles/feature-iec104/burst-log.md` (created D-480; archives the D-475 row rolled out under the last-5 rule; this housekeeping burst rolled the D-476 row out under the same rule) |
| **Phase Progress granular rows (F4 waves/adversary/fixes)** | `cycles/feature-iec104/phase-progress-archive.md` (D-451 burst, wave-79..83, STORY-172/173/174 per-story adversary, FIX-P4-001/F5-001..004) |
| **Convergence Trajectory (F4 per-story + F5 phase)** | `cycles/feature-iec104/convergence-trajectory.md` |
| feature-iec104 F2 convergence report | `cycles/feature-iec104/adversarial/f2-convergence-report.md` (12 passes, CONVERGED P10/P11/P12, D-438) |
| feature-iec104 F2 gate review (first-frame guard) | `cycles/feature-iec104/adversarial/f2-first-frame-guard-review.md` (CLEAN; 2 LOW applied; D-439) |
| feature-iec104 MITRE pin confirmation | `cycles/feature-iec104/research/f2-mitre-pin-confirmation.md` (8 techniques CONFIRMED-AT-v19.1; D-439) |
| Session checkpoints (feature-iec104, all prior) | `cycles/feature-iec104/session-checkpoints.md` (waves 76–83 era + D-471 E2E + D-472 PR #407 + D-473 v0.13.0 RELEASED + D-474 SESSION WRAP + D-475 CYCLE-CLOSE + D-476 PR#414 + D-477 upstream-routing + D-478 dep-soak + D-479 session-wrap checkpoints, all archived) |
| Wave 75 gate files | `cycles/wave-75/wave-gate/` (gate-summary.md D-435, code-review.md, findings.md) |
| Wave 75 lessons + process-gap ledger | `cycles/wave-75/lessons.md` + `cycles/wave-75/process-gap-ledger.md` |
| Wave 74 gate files | `cycles/wave-74/wave-gate/` (gate-summary.md D-432) |
| Wave 73 gate files + lessons | `cycles/wave-73/wave-gate/` + `cycles/wave-73/lessons.md` |
| Wave 72 gate files + lessons | `cycles/wave-72/wave-gate/` (gate-summary.md, code-review.md, demo-evidence/) + `cycles/wave-72/lessons.md` |
| Wave 71 gate files + lessons + checkpoints | `cycles/wave-71/wave-gate/` + `cycles/wave-71/session-checkpoints.md` + `cycles/wave-71/lessons.md` |
| Wave 70 gate files + checkpoints | `cycles/wave-70-story-149/wave-gate/` + `cycles/wave-70-story-149/session-checkpoints.md` |
| Burst history + lessons (maint-2026-07-06) | `cycles/maint-2026-07-06/burst-log.md` + `cycles/maint-2026-07-06/lessons.md` |
| Session checkpoints + lessons (maint-2026-07-08) | `cycles/maint-2026-07-08/session-checkpoints.md` + `cycles/maint-2026-07-08/lessons.md` |
| Wave 82 gate files | `cycles/wave-082/wave-gate/` (gate-summary.md D-458, code-review.md) |
| Wave 83 gate files | `cycles/wave-083/wave-gate/` (gate-summary.md D-463, code-review.md) |
| STORY-174 per-story convergence report | `cycles/feature-iec104/STORY-174/convergence-report.md` (7 passes, CONVERGED P5/P6/P7, D-462) |
| STORY-147 per-story convergence report | `cycles/wave-084/STORY-147/convergence-report.md` + `adversary-convergence-state.json` (8 passes, CONVERGED P6/P7/P8, 2026-07-19) |
| STORY-147 delivery evidence (PR #421, D-481) | `cycles/wave-084/STORY-147/` + `.factory/code-delivery/STORY-147/` (committed f2b5dcfe) |
| feature-iec104 F5 adversarial reviews | `.factory/phase-f5-adversarial/round-1-review.md` through `round-5-review.md`; `convergence-summary.md` (D-468) |
| feature-iec104 F6 gate verdict + hardening artifacts | `.factory/phase-f6-hardening/f6-gate-verdict-iec104.md` (D-469 PASS); `kani-results.md`, `fuzz-results.md`, `mutation-results.md`, `security-scan-results.md` |
| feature-iec104 F7 convergence artifacts | `.factory/phase-f7-convergence/delta-convergence-report.md` (D-470 CONVERGED); `traceability-chain-delta.md`; `consistency-audit.md` |
| D-476 Current Phase Steps row (rolled out under last-5 rule by this housekeeping burst) | `cycles/feature-iec104/burst-log.md` |
| D-477 Current Phase Steps row (rolled out under last-5 rule by the STORY-147 Step-4.5 convergence row) | `cycles/wave-084/burst-log.md` |
| D-478 Current Phase Steps row (rolled out under last-5 rule by the D-481 STORY-147 DELIVERED row) | `cycles/wave-084/burst-log.md` |
| Session Resume Checkpoint superseded by D-481 (STORY-147 Step-4.5/Step-5 in-flight checkpoint) | `cycles/wave-084/session-checkpoints.md` |
| Wave-84 session checkpoints (housekeeping-burst checkpoint, superseded) | `cycles/wave-084/session-checkpoints.md` |
