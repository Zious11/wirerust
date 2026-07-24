---
document_type: pipeline-state
level: ops
version: "2.0"
producer: state-manager
inputs: []
input-hash: "[live-state]"
traces_to: ""
project: wirerust
mode: maintenance
phase: "steady-state"
status: active
current_step: "D-506 STORY-180 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 4 passes; streak P2/P3/P4 = 3/3; BC-5.39.001 SATISFIED. Trajectory 3M(P1)→3L(P2)→1L(P3)→1L(P4). Remediation commits a0087033/e40955f1/0502c642. Demo evidence ccec1711 (8 artifacts, PG-W70-DEMO-SCRUB PASSED). BC-INDEX v2.37. PG-W85-003 filed. Next: pr-manager full 9-step PR lifecycle (STORY-180). trajectory-tail →0→0→0→0"
current_cycle: "wave-085"
pipeline: ACTIVE
timestamp: 2026-07-24T08:11:00Z
released_version: v0.13.1
released_at: "2026-07-21"
release_tag: v0.13.1
release_tag_object: 47b7d23c137483de37aa7705617749f5f9d37b07
release_commit: 47b7d23c137483de37aa7705617749f5f9d37b07
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.13.1
prior_released_version: v0.13.0
prior_released_at: "2026-07-18"
main_head: 47b7d23c137483de37aa7705617749f5f9d37b07
develop_head: dc7331fbe3a41fc2b74084dafd8553c3009d7c2e
cargo_version_main: "0.13.1"
cargo_version_develop: "0.13.1"
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
stories_delivered: 116
story_index_version: "v3.91"
total_stories: 134
story_index_note: "134 stories / 85 waves / 783 pts. v3.91 (2026-07-24): WAVE-85 HUMAN STORY-APPROVAL GATE PASSED (D-505) — STORY-180/181 status draft→ready; STORY-INDEX v3.90→v3.91; no numeric totals changed. v3.90 (2026-07-24): pre-gate remediation burst (D-504) — index-body currency corrections: wave count 83→85 (wave-84 STORY-147/166/176 + wave-85 STORY-180/181), dep-graph v3.9→v3.10 (STORY-174→STORY-180 edge, 137→138 acyclic edges), E-22 epic row updated (v3.10 acyclic 138 edges); no numeric story/points totals changed. v3.89 (2026-07-23): STORY-181 title-cell correction (F-P4-001 pass-4 adversary remediation, D-498) — Direction-Keyed Carry Select framing removed from STORY-181 title cell; correct framing Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop now consistent with STORY-181 body (FSR line 262), AC-181-003 trace (line 119), and risk-register.md R-010; no numeric totals changed. v3.88 (2026-07-23): wave-85 STORY-CREATION BURST (D-493) — STORY-180 (IEC-104 timed control-command detection TypeIDs 58–64, E-22, 5 pts, wave 85, BC-2.19.029+030+022 v1.1 regression guard) + STORY-181 (SEC-001 ENIP split-borrow refactor + ROUTE-W74 OBS-1, E-20, 3 pts, wave 85, BC-2.17.016); BC-2.19.022 v1.1 propagation sweep: STORY-170 v2.0→v2.1 (AC-170-005/006 silently-logged range 52–99→{52–57,65–99}, BC table annotated); total_stories 132→134; total_points 775→783; total_waves 84→85; wave-table scheduled 692→700. v3.87 (2026-07-21): Epic table TOTAL cell arithmetic corrected 776→775 (SPEC-009); per-epic sum = 775 = frontmatter total_points; root cause: v3.79 re-scope delta decremented E-11 row (67→66) but TOTAL cell not updated; no other numeric changes; maint-2026-07-21 D-490. STORY-INDEX v3.86→v3.87. v3.86 (2026-07-21): E-16/E-17 ARP stale-draft supersession (D-487, 2026-07-21) — 7 drafts STORY-111..117 status draft→superseded DELIVERED-BY-DRIFT; E-16 v0.7.0 (STORY-111..115, 47 pts, waves 40-44) + E-17 v0.7.0/v0.7.1 (STORY-116/117, 8 pts, waves 45-46); twice-research-validated DF-VALIDATION-001 + human-approved; wave-table scheduled 747→692; total_points 775 unchanged per D-477/D-480 supersession-convention. STORY-INDEX v3.85→v3.86. v3.85 (2026-07-21): WAVE-84 GATE CLOSED (D-486); wave-84 delivery row updated CLOSED-PENDING-GATE→CLOSED (D-486, 2026-07-21); story-file status loci synced (STORY-147/166/176 frontmatter+body status: ready→delivered, three-loci agreement with STORY-INDEX rows at v3.84). No numeric totals changed. v3.84 (2026-07-20): STORY-176 DELIVERED (D-485, PR #427 595cdba8 squash-merged to develop, human-executed merge 2026-07-20T21:46:45Z under explicit per-PR human authorization, DF-MERGE-AUTH-CLASSIFIER-001 satisfied; wave-84 #421/#426/#427 pattern match); status ready→delivered; wave-84 Delivery Progress row updated (3/3 DELIVERED — STORY-147 ✓, STORY-166 ✓, STORY-176 ✓; CLOSED-PENDING-GATE); CI 13/13 PASS (new \"Bin selftest suites\" step); pr-reviewer APPROVE (1 cycle, 0 blocking, 3 NITs accepted; self-authored PR — COMMENTED event + pr-review.md = review of record); security APPROVE (0C/0H/0M/1L pre-existing SEC-001 CWE-22); 8-pass Step-4.5 adversary CONVERGED P6/P7/P8 (BC-5.39.001 SATISFIED); story v2.7/6ec8772. stories_delivered 115→116. No numeric points/story/wave totals changed (status transition only)."
bc_index_version: "v2.37"
vp_index_version: "v2.46"
arch_index_version: "v2.20"
prd_version: "v1.59"
epics_version: v2.1
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-07-21
maintenance_started_at: "2026-07-21"
maintenance_completed_at: "2026-07-21"
maintenance_prior_run: maint-2026-07-11
---

<!--
  STATE.md SIZE BUDGET (per D-421(c)):
    Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 300 = 200 (dual-margin form). ~300 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-506 STORY-180 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 4 fresh-context passes; streak P2/P3/P4 = 3/3 clean; zero open HIGH/CRIT; BC-5.39.001 SATISFIED. Trajectory 3M(P1)→3L(P2)→1L(P3)→1L(P4). Remediation commits a0087033/e40955f1/0502c642 on feature branch; head ccec1711 (demo evidence, 8 artifacts, PG-W70-DEMO-SCRUB PASSED). Red Gate PASSED (21 red/227 green). BC-2.19.029 v1.3 / BC-2.19.030 v1.2 (label refresh); BC-INDEX v2.37. PG-W85-003 filed. Pipeline ACTIVE — pr-manager full 9-step PR lifecycle next (STORY-180). trajectory-tail →0→0→0→0**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); **RELEASED v0.13.0 (D-473, 2026-07-18). F1→F7 CONVERGED; CYCLE CLOSED (D-475, 2026-07-18): S-7.02 SATISFIED. D-477: STORY-175/177/178/179 codification VEHICLE CHANGED to upstream (see D-477). D-480: E-11 disposition burst #2 — STORY-091/121/143/155 superseded; STORY-147 v2.0 local survivor. WAVE-84 OPENED (STORY-166/176/147v2, 7 pts, all product-local). D-481: STORY-147 DELIVERED (PR #421 f0cb7374). D-482: STORY-166 DELIVERED (PR #426 fa9be701). D-485: STORY-176 DELIVERED (PR #427 595cdba8) — wave-84 3/3 DELIVERY COMPLETE. D-486: WAVE-84 GATE CLOSED + S-7.02 COMPLETE (2026-07-21). D-487: E-16/E-17 ARP stale-draft supersession; backlog EMPTY. D-488: SESSION WRAP (2026-07-21). D-489: SESSION RESUMED + maintenance sweep maint-2026-07-21 STARTED (2026-07-21). D-490: maint-2026-07-21 COMPLETE (2026-07-21). D-491: v0.13.1 RELEASED (2026-07-21). D-492: SESSION WRAP (2026-07-21). D-493: SESSION RESUMED + WAVE-85 SCOPED (2026-07-23); IEC104-TIMED-CMD-GAP-001 research in flight; SEC-001 + ROUTE-W74 pulled into wave-85. D-494: WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE (2026-07-23); STORY-180/181 drafted; adversarial convergence next.** |
| Version | 0.13.1 (released 2026-07-21; main=47b7d23c; develop=dc7331fb — D-491 v0.13.1 RELEASED (PR #432 dev-tooling patch + PR #433 true-merge back-merge, 2026-07-21)) |
| Main HEAD | `47b7d23c137483de37aa7705617749f5f9d37b07` |
| Develop HEAD | `dc7331fbe3a41fc2b74084dafd8553c3009d7c2e` — D-491 v0.13.1 RELEASED (PR #433 true-merge back-merge, 2026-07-21) |
| Spec versions | BC-INDEX v2.37 / VP-INDEX v2.46 / ARCH-INDEX v2.20 / PRD v1.59 |
| Stories | 116 delivered / 134 total (STORY-INDEX v3.91, dep-graph v3.10, 783 pts) |
| **Last Updated** | 2026-07-24 — D-506 STORY-180 STEP-4.5 ADVERSARIAL CONVERGED. 4 passes streak 3/3; BC-5.39.001 SATISFIED. BC-INDEX v2.37. PG-W85-003 filed. pr-manager PR lifecycle next. trajectory-tail →0→0→0→0 |

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
| feature-iec104 — F5 (scoped adversarial) | **CONVERGED (D-468)** | 5 rounds; pass-5 NITPICK_ONLY (0 CRIT/HIGH/MED; 1 LOW non-blocking); code frozen R2 (9c5aa9a); BC-completeness 31/31 + canonical-frame 19 byte-exact clean |
| feature-iec104 — F6 (targeted-hardening) | **PASS (D-469)** | Kani/fuzz/mutation/audit/regression all green; VPs re-run post-fix on b36b884 |
| feature-iec104 — F7 (delta-convergence) | **CONVERGED (D-470)** | 5/5 dims PASS; holdout 0.99 RELEASE-READY |
| v0.13.0 RELEASED | RELEASED 2026-07-18 | PR #417 67a06b6 main + tag v0.13.0 + GH release 4 assets; back-merge #418; IEC-104 F1-F7 |
| **feature-iec104 cycle-close (S-7.02)** | **CLOSED (D-475)** | 9 PGs → STORY-175..179 (12 pts, E-11 epic); B-001/B-002 FIXED; PR #419 82ad2ed; STORY-INDEX v3.77. **D-477: STORY-175/177/178/179 vehicle changed to upstream per D-477.** |
| **Wave 84 (E-11 mini-wave: STORY-166/176/147v2)** | **CLOSED (D-486, 2026-07-21)** | 7 pts. STORY-147 (PR #421 f0cb7374) + STORY-166 (PR #426 fa9be701) + STORY-176 (PR #427 595cdba8). 3 gate-fix PRs #428/429/430. develop=1e967bad. S-7.02 COMPLETE. trajectory-tail →0→0→0→0 |
| pass-84 adversary (wave-84 gate, 6 passes, streak P4/P5/P6) | CONVERGED (D-486) | pass-1 1M; pass-2 M/L-batch; pass-3 1L; pass-4 NITPICK_ONLY; pass-5 NITPICK_ONLY; pass-6 NITPICK_ONLY (streak 3/3). Code frozen 1e967bad. DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. trajectory-tail →0→0→0→0 |
| Wave-84 gate fix burst (3 gate-fix PRs) | CLOSED (D-486) | Gate fix burst: #428 82105d02 (F-W84G-P1-001 MEDIUM TLS doctest); #429 39b30cb1 (CR-002/005/006 + SEC-003 FIXED); #430 1e967bad (F-W84G-P3-001 LOW STORY-147 inline comment). develop=1e967bad. |
| **Maintenance maint-2026-07-21** | **COMPLETE (D-490, 2026-07-21)** | 8 sweeps (S6=DTU SKIP, S9=a11y SKIP). 5 PRs merged: #422-425 Dependabot batch + #431 IEC-104 doc-drift. ARCH-INDEX v2.20 + STORY-INDEX v3.87 + HS-INDEX v2.14. Register v2.0. develop=6c47c0ef. |
| **v0.13.1 RELEASED** | **RELEASED 2026-07-21** | PR #432 47b7d23c main + tag v0.13.1 (lightweight) + GH release 4 assets; back-merge #433 TRUE-MERGE dc7331fb; DRIFT-BACKMERGE-SQUASH-001 RESOLVED. |
| **Wave 85 (IEC-104 completion mini-wave)** | **ACTIVE (D-506, 2026-07-24)** | Spec-evolution + story-creation COMPLETE (D-494). P1-P6 REMEDIATED (D-495..500). Pass-7/8/9 CLEAN 3/3 (D-501..503): CONVERGED. BC-5.39.001 SATISFIED. D-504: BC-INDEX v2.36 + STORY-INDEX v3.90. D-505: HUMAN STORY-APPROVAL GATE PASSED; STORY-180/181 ready; STORY-INDEX v3.91. D-506: STORY-180 per-story adversarial CONVERGED (2026-07-24) — 4 passes, streak P2/P3/P4, trajectory `3M(P1)→3L(P2)→1L(P3)→1L(P4)`; demo ccec1711 (8 artifacts, PG-W70-DEMO-SCRUB PASSED); BC-INDEX v2.37; PG-W85-003 filed. develop=dc7331fb. trajectory-tail →0→0→0→0 |

---

## Convergence Status

Per-story F4 convergence details archived to `cycles/feature-iec104/convergence-trajectory.md`.
F5 phase-level trajectory: 5 rounds, code frozen R2, `5H/M→2M→1H→1M→1L(NB)` — CONVERGED (D-468).
Wave-84 gate-level adversarial trajectory (6 passes, code frozen 1e967bad): `1M→M/L-batch→1L→0→0→0` — CONVERGED (D-486). Streak P4/P5/P6.
Wave-85 story adversarial trajectory (CONVERGED): `1C+2H+4M+2L(P1)→3M/1L(P2)→1M(P3)→1H(P4)→NITPICK/1L(P5)→1M/2L(P6)→NITPICK/2L(P7 1/3)→CLEAN/0(P8 2/3)→NITPICK/1L-closed(P9 3/3) → CONVERGED 3/3 (P7/P8/P9)` — BC-5.39.001 SATISFIED. trajectory-tail →0→0→0→0.
Wave-85 STORY-180 per-story adversarial trajectory (CONVERGED D-506): `3M(P1)→NITPICK/3L(P2)→NITPICK/1L(P3)→NITPICK/1L(P4) → CONVERGED 3/3 (P2/P3/P4)` — BC-5.39.001 SATISFIED. Commits a0087033/e40955f1/0502c642. Demo head ccec1711. trajectory-tail →0→0→0→0.

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | **CLOSED (D-475, 2026-07-18)** — v0.13.0 RELEASED (D-473); F1→F7 CONVERGED (D-470); S-7.02 SATISFIED. D-477: STORY-175/177/178/179 vehicles changed to upstream; STORY-176 v2.0 + STORY-166 local survivors | develop (1e967bad) |
| wave-084 (E-11 mini-wave) | **CLOSED (D-486, 2026-07-21)** — 3/3 DELIVERED + gate CLOSED; S-7.02 COMPLETE; 12 PG-W84 entries (3 FIXED / 9 deferred to DF-VALIDATION-001 batch). develop=1e967bad (PR #430 gate-fix final). trajectory-tail →0→0→0→0 | develop (1e967bad, D-486 gate-close) |
| wave-085 (IEC-104 completion mini-wave) | **ACTIVE (D-506, 2026-07-24)** — STORY-180 per-story adversarial CONVERGED 3/3 (D-506); demo evidence ccec1711 committed; BC-INDEX v2.37; PG-W85-003 filed; pr-manager PR lifecycle next (STORY-180). trajectory-tail →0→0→0→0 | develop (dc7331fb, unchanged) |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-506 STORY-180 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 4 fresh-context passes; streak P2/P3/P4 = 3/3; BC-5.39.001 SATISFIED. Trajectory 3M(P1)→3L(P2)→1L(P3)→1L(P4). Commits a0087033/e40955f1/0502c642; demo head ccec1711 (8 artifacts, PG-W70-DEMO-SCRUB PASSED). Red Gate PASSED (21 red/227 green). BC-2.19.029 v1.3 / BC-2.19.030 v1.2 (label refresh); BC-INDEX v2.37. PG-W85-003 filed. pr-manager PR lifecycle next (STORY-180). trajectory-tail →0→0→0→0** | **ACTIVE (D-506)** | STORY-180 Step-4.5 CONVERGED. Demo evidence committed. BC-INDEX v2.37. PR lifecycle next. trajectory-tail →0→0→0→0 |
| **D-505 WAVE-85 HUMAN STORY-APPROVAL GATE PASSED (2026-07-24). STORY-180/181 approved for Phase 3 TDD per-story delivery (STORY-180 first — dep on delivered STORY-174; then STORY-181). Structured review questions presented (TypeID 58-64 scope, SEC-001 anchor enip.rs:992-999, ROUTE-W74 OBS-2 left pending, MITRE parity mapping) — human approved both without changes. STORY-180 v1.1 / STORY-181 v1.1 status ready. STORY-INDEX v3.91. Pipeline ACTIVE. trajectory-tail →0→0→0→0** | **COMPLETE (D-505)** | Human gate PASSED. STORY-180/181 status ready. STORY-INDEX v3.91. trajectory-tail →0→0→0→0 |
| **D-504 WAVE-85 PRE-GATE REMEDIATION BURST (2026-07-24). BC-INDEX v2.35→v2.36: CV-001/002/003 changelog entries, total count 379/378→381/380, row annotations BC-2.19.019/028/029/030 updated. STORY-INDEX v3.89→v3.90: wave count 83→85, dep-graph v3.9→v3.10 (STORY-174→STORY-180 edge, 137→138 acyclic edges), E-22 epic row updated. CV-004/005 story-anchor fills APPLIED. CV-008 DEFERRED: VP-047 source_bc deferred to STORY-180 delivery. Input hashes rebaselined (STORY-170 096877a, STORY-180 8ddf419). Pipeline ACTIVE. trajectory-tail →0→0→0→0** | **COMPLETE (D-504)** | Pre-gate remediation burst COMPLETE. BC-INDEX v2.36. STORY-INDEX v3.90. CV-008 deferred. trajectory-tail →0→0→0→0 |
| **D-503 WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). Streak P7/P8/P9 = 3/3. BC-5.39.001 + DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. F-W85S-P9-001 LOW closed. wave-85 spec+story package CONVERGED, ZERO open findings. Pipeline PAUSED. develop=dc7331fb. trajectory-tail →0→0→0→0** | **COMPLETE (D-503)** | wave-85 CONVERGED 3/3. Superseded by D-504. trajectory-tail →0→0→0→0 |
| **D-502 WAVE-85 ADVERSARIAL PASS 8 → FULLY CLEAN (2026-07-23). Pass-8: 0 CRIT/HIGH/MED/LOW. Clean-pass streak 2/3. trajectory-tail →0→0→0→0** | **COMPLETE (D-502)** | Pass-8 FULLY CLEAN. Clean streak 2/3. trajectory-tail →0→0→0→0 |


## Decisions Log

| ID | Decision | Date |
|----|----------|------|
| D-001..D-301 (exhaustive). Greenfield through feature-enip-v0.11.0; see cycles/*/decisions-archive.md for full range. | — | — |
| D-302..D-436 (exhaustive). Fix-tls through feature-protocol-coverage through v0.12.1; see cycles/history/decision-log-archive.md for full range. | — | — |
| D-437..D-458 (exhaustive). feature-iec104 F1 engine triage through F4 delivery; see cycles/feature-iec104/decisions-archive.md for full range. | — | — |
| D-460 | Session RESUMED (human-approved, 2026-07-16). Worktree health PASS; develop=084ff93 verified; no story worktrees; only open PR is external #407 (deferred post-wave-83 by human). STORY-174 wave-83 begins with research-agent validation. | 2026-07-16 |
| D-461 | STORY-174 pre-delivery realignment COMPLETE (research-validated, human-approved 2026-07-16). DF-VALIDATION-001 research 2 passes (all HIGH confidence). STORY-174 v2.0 input-hash de9d14e→27c86aa. STORY-INDEX v3.72→v3.73. | 2026-07-16 |
| D-462 | STORY-174 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 7 passes; streak P5/P6/P7; final HEAD e62701f; 2600+/0 tests. Story v2.2; STORY-INDEX v3.75. | 2026-07-16 |
| D-463 | STORY-174 DELIVERED (PR #409 547deba squash-merged, 2026-07-17, human-direct merge after TWO subagent-classifier halts). CI 13/13. 8/8 IEC-104 stories delivered. stories_delivered 112→113. Wave-83 gate SATISFIED. | 2026-07-17 |
| D-464 | FIX-P4-001 DELIVERED (PR #410 7e95f71 squash-merged, 2026-07-17, human-executed merge). IEC104-FINDING-DIRECTION-001 RESOLVED. CI 13/13. develop=7e95f71. | 2026-07-17 |
| D-465 | feature-iec104 F5 scoped adversarial OPENED (2026-07-17). Round 1 @ 7e95f71: BC-completeness 31/31 PASS; canonical-frame 19 invariants byte-exact; 1H+4M findings → FIX-F5-001. | 2026-07-17 |
| D-466 | FIX-F5-001 DELIVERED (PR #411 9c5aa9a squash-merged, 2026-07-17). source_ip + timestamp enrichment; 10 red-first tests; 9 stale-prose sites scrubbed. CI 13/13. develop=9c5aa9a. | 2026-07-17 |
| D-467 | F5 Rounds 2-3 (2026-07-17). R2 code CONVERGED + 2 MEDIUM doc findings → FIX-F5-002 (#412 b356545). R3: F-B1 HIGH fabricated FIX-P4-001 demo-evidence → FIX-F5-003 (PG-DEMO-JSON-FABRICATION root cause confirmed). | 2026-07-17 |
| D-468 | feature-iec104 F5 CONVERGED (2026-07-17). 5 rounds. FIX-F5-002/003/004 DELIVERED. R5 NITPICK_ONLY. BC-completeness 31/31 + canonical-frame 19 byte-exact. develop=b36k884. | 2026-07-17 |
| D-469 | feature-iec104 F6 targeted hardening PASS (2026-07-17). Kani/fuzz/mutation/audit/regression all green. cargo-mutants iec104.rs 95.9%. No BLOCKERs. | 2026-07-17 |
| D-470 | feature-iec104 F7 delta convergence CONVERGED (2026-07-17). 5/5 dims PASS; holdout 0.99 RELEASE-READY. RELEASE HELD (human direction) — v0.13.0 cut deferred. | 2026-07-17 |
| D-471 | E2E IEC-104 coverage merged (PR #416 0b65e8e, 2026-07-17, human-executed merge). 4 real captures + tests/iec104_e2e_real_pcaps_tests.rs. CI 13/13. | 2026-07-17 |
| D-472 | PR #407 security-triaged (2026-07-18): SAFE-WITH-CHANGES. DEFERRED by human — governance decision pending. Triage: .factory/planning/pr-407-security-triage.md. | 2026-07-18 |
| D-473 | v0.13.0 RELEASED (2026-07-18). Release PR #417 67a06b6 main; tag v0.13.0; GH release 4 assets; back-merge #418 af3ecbd develop. 13 commits released. DRIFT-BACKMERGE-SQUASH-001 retained. | 2026-07-18 |
| D-474 | SESSION WRAP (2026-07-18). Human-requested pipeline pause at clean milestone post-v0.13.0 release. Pipeline PAUSED. | 2026-07-18 |
| D-475 | feature-iec104 CYCLE-CLOSE (2026-07-18). S-7.02 checklist SATISFIED: 9 PGs → STORY-175..179; B-001/B-002 FIXED; PR #419 82ad2edd merged. feature-iec104 CLOSED. | 2026-07-18 |
| D-476 | PR #414 ADOPTED (2026-07-19). ArcavenAE fork ci/scorecard-guard squash-merged to develop fcd57dcb (human-executed 2026-07-19T01:54:40Z). CI 13/13 incl. action-pin-gate. | 2026-07-19 |
| D-477 | UPSTREAM-ROUTING (2026-07-19). E-11 process-gap codification redirected from local STORY-175..179 to upstream drbothen/vsdd-factory. STORY-176 v2.0 local survivor. STORY-INDEX v3.78. | 2026-07-19 |
| D-478 | DEP-SOAK DELIVERED (2026-07-19). PR #420 squash-merged to develop 49255464 (human-executed). Lockfile-only: 24 distinct version-pair changes. CI 13/13. | 2026-07-19 |
| D-479 | SESSION WRAP (2026-07-19). Human-requested pause post-D-478 dep-soak. Pipeline PAUSED. | 2026-07-19 |
| D-480 | E-11 DISPOSITION BURST DELIVERED (2026-07-19). STORY-091/121/143/155 → superseded; STORY-147 → v2.0 SPLIT survivor; WAVE-84 OPENED (STORY-166/176/147v2, 7 pts). STORY-INDEX v3.78→v3.79. | 2026-07-19 |
| D-481 | STORY-147 DELIVERED (PR #421 f0cb7374 squash-merged 2026-07-20, human-executed). 8-pass Step-4.5 adversary CONVERGED P6/P7/P8. stories_delivered 113→114. | 2026-07-20 |
| D-482 | STORY-166 DELIVERED (PR #426 fa9be701 squash-merged 2026-07-20, human-executed). 10-pass Step-4.5 adversary CONVERGED P8/P9/P10. stories_delivered 114→115. | 2026-07-20 |
| D-483 | SESSION WRAP (2026-07-20). Human-requested pause. D-480..D-482 (exhaustive). Pipeline PAUSED. | 2026-07-20 |
| D-484 | Session RESUMED (human-approved, 2026-07-20) from D-483 pause. Worktree health PASS. STORY-176 v2.2 per-story delivery next. Dependabot PRs #422-425 deferred. Pipeline ACTIVE. | 2026-07-20 |
| D-485 | STORY-176 DELIVERED (PR #427 595cdba8 squash-merged to develop 2026-07-20T21:46:45Z, human-executed). CI 13/13. Step-4.5 adversary CONVERGED P6/P7/P8 (BC-5.39.001 SATISFIED). Story v2.7. STORY-INDEX v3.83→v3.84. stories_delivered 115→116. Wave-84 DELIVERY COMPLETE (3/3). | 2026-07-20 |
| D-486 | WAVE-84 GATE CLOSED + S-7.02 COMPLETE (2026-07-21). Integration gate 6-gate all-pass: Gate 1 PASS (2640 tests/94 suites, develop 1e967bad, clippy/fmt clean, 5 bin/ Python self-tests pass); Gate 3 PASS/CONVERGED (6 passes, streak P4/P5/P6, DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED; gate-fix PRs #428/#429/#430); Gate 3b PASS (consistency 4MED/3LOW addressed); Gate 4 PASS (STORY-147/166/176 demo evidence on develop). S-7.02 cycle-close COMPLETE: 12 PG-W84 entries — PG-W84-007/009/011 FIXED in-cycle; others deferred. gate-summary.md + code-review.md + lessons.md authored. STORY-INDEX v3.84→v3.85. develop=1e967bad. WAVE-84 CLOSED. | 2026-07-21 |
| D-487 | E-16/E-17 ARP STALE-DRAFT SUPERSESSION (2026-07-21). STORY-111..115 (E-16, 47 pts, waves 40-44) + STORY-116/117 (E-17, 8 pts, waves 45-46) status draft→superseded DELIVERED-BY-DRIFT. Twice research-validated (DF-VALIDATION-001), human-approved. Wave-table scheduled 747→692; total_points 775 unchanged. STORY-INDEX v3.85→v3.86. Backlog now EMPTY. | 2026-07-21 |
| D-488 | SESSION WRAP (2026-07-21). Human-requested pause at clean idle milestone. D-484..D-487 (exhaustive). Backlog EMPTY. Pipeline PAUSED. | 2026-07-21 |
| D-489 | Session RESUMED + maintenance sweep maint-2026-07-21 STARTED (2026-07-21, human-approved). Worktree health PASS. Human scope: dep-soak eligibility from upstream RELEASE DATE; NO carry-forwards. Sweeps 1-5,7,8 dispatched; S6 DTU SKIP; S9 a11y SKIP. | 2026-07-21 |
| D-490 | maint-2026-07-21 COMPLETE (2026-07-21). 8 sweeps total (S6=DTU SKIP, S9=a11y SKIP). DOC-011 HIGH fixed (PR #431 6c47c0ef, human-executed). Dependabot #422-425 batch-merged. Holdouts repaired HS-087/123/125/132. ARCH-INDEX v2.20. STORY-INDEX v3.86→v3.87. Tech-debt register v2.0. develop=6c47c0ef. | 2026-07-21 |
| D-491 | v0.13.1 RELEASED (2026-07-21). Release PR #432 47b7d23c squash-merged to main (human-merged). Tag v0.13.1 (lightweight). GH release 4 assets. Back-merge PR #433 TRUE-MERGE dc7331fb to develop (human decision). DRIFT-BACKMERGE-SQUASH-001 RESOLVED. | 2026-07-21 |
| D-492 | SESSION WRAP (2026-07-21). Human-requested pause (/wrap). D-489..D-491 (exhaustive). No in-flight work. Pipeline PAUSED. trajectory-tail →0→0→0→0 | 2026-07-21 |
| D-493 | Session RESUMED + WAVE-85 SCOPED (human-approved, 2026-07-23). Resumed from D-492 pause. Worktree health PASS (factory-artifacts a1676f0d in-sync). Option A selected: wave-85 IEC-104 completion mini-wave. Scope: IEC104-TIMED-CMD-GAP-001 + SEC-001 ENIP split-borrow + ROUTE-W74. Pipeline ACTIVE. | 2026-07-23 |
| D-494 | WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE (2026-07-23). BC-2.19.029/030 NEW v1.0; BC-2.19.022 v1.0→v1.1; BC-INDEX v2.34→v2.35; HS-133..136 (HS-INDEX v2.15); prd.md §2.19.E/H updated. STORY-180/181 drafted; STORY-170 v2.0→v2.1; STORY-INDEX v3.87→v3.88. | 2026-07-23 |
| D-495 | WAVE-85 ADVERSARIAL PASS 1 → REMEDIATED (2026-07-23). 1 CRIT / 2 HIGH / 4 MED / 2 LOW. CRITICAL F-W85S-P1-001: STORY-181 mis-anchored SEC-001; re-anchored to enip.rs:992-999. F-P1-002/003 (HIGH): tech-debt-register + HS-136 count=0 fixed. HS-INDEX v2.15→v2.16. | 2026-07-23 |
| D-496 | WAVE-85 ADVERSARIAL PASS 2 → REMEDIATED (2026-07-23). 0 CRIT/HIGH / 3 MED / 1 LOW. F-P2-001 STORY-170:62 silently-logged range corrected; F-P2-002/003 HS-136 dropped BC-2.19.028 + Case D jq fixed; F-P2-004 HS-135 LEN corrected. HS-INDEX v2.16→v2.17. PG-W85-001 adjudicated (template defect). | 2026-07-23 |
| D-497 | WAVE-85 ADVERSARIAL PASS 3 → REMEDIATED (2026-07-23). 0 CRIT/HIGH / 1 MED. F-P3-001: STORY-170 AC-170-005 Note missing [1,44] segment corrected; all 11 silent-set loci now consistent. | 2026-07-23 |
| D-498 | WAVE-85 ADVERSARIAL PASS 4 → REMEDIATED (2026-07-23). 0 CRIT / 1 HIGH. F-P4-001: STORY-181 3 loci retained REJECTED framing; all fixed + risk-register R-010 swept. PG-W85-002 flagged. | 2026-07-23 |
| D-499 | WAVE-85 ADVERSARIAL PASS 5 → CLEAN NITPICK_ONLY (2026-07-23). 0 CRIT/HIGH/MED / 1 LOW. FIRST CLEAN PASS (streak 1/3). F-P5-001 (LOW) REC-004 get_disjoint harmonized to take-remove-reinsert. | 2026-07-23 |
| D-500 | WAVE-85 ADVERSARIAL PASS 6 → REMEDIATED (2026-07-23). 0 CRIT/HIGH / 1 MED / 2 LOW — ALL PRE-EXISTING. Adversary CERTIFIED wave-85 timed-command package. Fixed: F-P6-001 TypeID-105 Possible→Likely; F-P6-002 stale v0.12.0 labels; F-P6-003 prd §2.19 header re-tensed. PRD v1.58→v1.59. Streak RESET to 0/3. | 2026-07-23 |
| D-501 | WAVE-85 ADVERSARIAL PASS 7 → CLEAN NITPICK_ONLY (2026-07-23). 0 CRIT/HIGH/MED / 2 LOW. Clean streak 1/3. F-P7-001/002/003 LOW residues swept. | 2026-07-23 |
| D-502 | WAVE-85 ADVERSARIAL PASS 8 → FULLY CLEAN (2026-07-23). 0 findings at any severity. Clean streak 2/3. | 2026-07-23 |
| D-503 | WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). Streak P7/P8/P9 = 3/3 clean. BC-5.39.001 + DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. F-W85S-P9-001 LOW closed (BC-2.19.019 v1.1→v1.2). Pipeline PAUSED. develop=dc7331fb unchanged. trajectory-tail →0→0→0→0 | 2026-07-23 |
| D-504 | WAVE-85 PRE-GATE REMEDIATION BURST (2026-07-24). BC-INDEX v2.35→v2.36: CV-001..005; total BC count text corrected 379/378→381/380. STORY-INDEX v3.89→v3.90: wave count 83→85, dep-graph v3.9→v3.10, E-22 epic row updated. CV-008 DEFERRED: VP-047 source_bc deferred to STORY-180 delivery. STORY-170 input-hash 7873f11→096877a; STORY-180 input-hash c0fad6c→8ddf419. Pipeline ACTIVE. | 2026-07-24 |
| D-505 | WAVE-85 HUMAN STORY-APPROVAL GATE PASSED (2026-07-24). Both stories approved for Phase 3 per-story delivery (STORY-180 first — dep on delivered STORY-174; then STORY-181). Structured review questions presented (TypeID 58-64 scope, SEC-001 anchor enip.rs:992-999, ROUTE-W74 OBS-2 left pending, MITRE parity mapping) — human approved both without changes. STORY-180 v1.1 / STORY-181 v1.1 status ready. STORY-INDEX v3.91. Next: per-story delivery STORY-180 (worktree → stubs → failing tests → TDD → Step-4.5 adversarial → demos → PR). | 2026-07-24 |
| D-506 | STORY-180 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 4 fresh-context passes; streak P2/P3/P4 = 3/3 clean; zero open HIGH/CRIT; BC-5.39.001 SATISFIED. Trajectory 3M(P1)→3L(P2)→1L(P3)→1L(P4). Remediation commits a0087033/e40955f1/0502c642 on feature branch; demo head ccec1711 (8 artifacts, PG-W70-DEMO-SCRUB PASSED). Red Gate PASSED (21 red/227 green). BC-2.19.029 v1.3 / BC-2.19.030 v1.2 (draft→ready label refresh, F-180-P4-001); BC-INDEX v2.36→v2.37. PG-W85-003 NEW: bin/check-green-doc-tense pattern set misses 'Expected RED:'/'currently falls through' stale-RED phrasing class (adversary pass-1 observation) — queued DF-VALIDATION-001 batch. Convergence state + report: cycles/wave-085/STORY-180/. Next: pr-manager full 9-step PR lifecycle (STORY-180). | 2026-07-24 |

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
| DRIFT-BACKMERGE-SQUASH-001 | **RESOLVED (D-491, 2026-07-21).** v0.13.1 back-merge PR #433 TRUE-MERGE (dc7331fb to develop); main (47b7d23c) IS ancestor of develop (dc7331fb). | v0.12.1 release → RESOLVED D-491 (2026-07-21) | RESOLVED — true-merge PR #433. Archive at next compact. |
| DRIFT-VP039-BC207038-TLS-TODO-001 | VP-INDEX carries stale TODOs for VP-039 (TLS reassembly). Out of feature-iec104 scope. | feature-iec104 F2 review (D-438) | SS-07 TLS owner — next TLS maintenance sweep |
| STORY-INDEX-IN-INPUTS-CHURN | Stories listing STORY-INDEX.md as input (STORY-164/165) re-stale on every index version bump. | D-477 → D-483 | Human decision: structural fix pending |
| PG-W84-UPSTREAM-BATCH | PG-W84-001/002/003/004/005/006/008 (7 upstream engine gaps). DF-VALIDATION-001 research required before filing. | wave-084 S-7.02 (D-486) | DF-VALIDATION-001 research pass (next available) |
| PG-W84-LOCAL-BATCH | PG-W84-010/012 (2 product-local gaps). DF-VALIDATION-001 research required. | wave-084 S-7.02 (D-486) | DF-VALIDATION-001 research pass |
| PG-W85-001 | Plugin-level template+hook defect. NOT per-file fix; does NOT block wave-85 convergence. | wave-085 pass-2 (D-496) | DF-VALIDATION-001 + upstream |
| PG-W85-002 | Recurring remediation-sweep locus-coverage gap. Flag for cycle-close codification. | wave-085 P2-P4 (D-496/497/498) | Cycle-close codification / DF-VALIDATION-001 |
| PG-W85-003 | bin/check-green-doc-tense pattern set misses 'Expected RED:'/'currently falls through' stale-RED phrasing class — allowed F-180-P1-003 (9 stale present-tense sites) to pass the gate undetected at Step 4. | wave-085 STORY-180 pass-1 adversary (D-506) | DF-VALIDATION-001 batch |

---

## Active Carry-Forwards

| ID | Summary | Target |
|---|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11. | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review NIT deferred wave-74; OBS-1 → AC-181-004 STORY-181 (wave-85); OBS-2 remains open. | STORY-181 (OBS-1); OBS-2 per ROUTE-W74-OBS-2 |
| ROUTE-W74-OBS-2 | ROUTE-W74 OBS-2 not absorbed by STORY-166/181. Pending human scope decision. | Next wave or maintenance run |
| PERF-RERUN-001 | AC-149-003 re-run PASS at maint-2026-07-21. Remains OPEN per human scope decision D-490. | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred maint-2026-07-11. Pulled into wave-85; absorbed into STORY-181. | STORY-181 (wave-85) |
| IEC104-TIMED-CMD-GAP-001 | TypeIDs 58–64 detection gap. CONFIRMED HIGH. STORY-180 drafted. | STORY-180 (wave-85) |
| PR-407-FORK-RELEASE-OPS | External ArcavenAE PR #407 SAFE-WITH-CHANGES (D-472); DEFERRED — governance pending. | Governance decision when authorized |
| SCORECARD-ENABLEMENT-RUNBOOK | Before setting SCORECARD_ENABLED=true: document CWE-200 publish_results:true risk. | Whenever scorecard is enabled |
| DEP-SOAK-FOLLOWUP-2026-07-27 | 17 not-yet-soaked crates eligible 2026-07-21..27. Run next soak on/after 2026-07-27. | Next maintenance run on/after 2026-07-27 |
| ROUTE-DOC-DEFER-2026-07-21 | PR #431 review residuals: ADR-0001 Consequences (LOW), ADR-0002 Deviations (NIT), ADR-0012 stale 'supported: 7 protocols' (LOW). | Next doc sweep |
| CV-008 | VP-047 source_bc annotation incomplete: BC-2.19.029/030 not yet added. Deferred from D-504. | STORY-180 delivery (wave 85) |

---

## Session Resume Checkpoint

**D-506 STORY-180 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 4 fresh-context passes; streak P2/P3/P4 = 3/3; BC-5.39.001 SATISFIED. Trajectory 3M(P1)→3L(P2)→1L(P3)→1L(P4). Commits a0087033/e40955f1/0502c642; demo head ccec1711 (8 artifacts, PG-W70-DEMO-SCRUB PASSED). BC-INDEX v2.37. PG-W85-003 filed. Pipeline ACTIVE — pr-manager full 9-step PR lifecycle next (STORY-180). trajectory-tail →0→0→0→0**

Prior checkpoints archived to `cycles/feature-iec104/session-checkpoints.md` and `cycles/wave-084/session-checkpoints.md` and `cycles/wave-085/session-checkpoints.md`.

- **Date:** 2026-07-24. Position: STORY-180 Step-4.5 adversarial CONVERGED (D-506); PR lifecycle (pr-manager) is NEXT.
- **Convergence counter:** STORY-180 BC-5.39.001 3/3 SATISFIED (P2/P3/P4) — do NOT re-run per-story adversarial on resume.
- **In-flight:** NONE. All bursts committed. Tree clean (factory-artifacts updated this burst).
- **PENDING NEXT STEPS (in order) on resume:** (a) pr-manager full 9-step PR lifecycle (STORY-180 feature branch → PR → CI → review → merge); (b) per-story delivery STORY-181; (c) wave-85 integration gate; (d) cycle-close: codify PG-W85-001 + PG-W85-002 + PG-W85-003.
- **Ground truth:** develop=dc7331fb, main=47b7d23c (v0.13.1). Feature branch HEAD ccec1711 (demo evidence). Only open product PR: external #407 (DEFERRED).
- **Pending human decisions:** PR #407 governance; STORY-INDEX-IN-INPUTS-CHURN; DEP-SOAK-FOLLOWUP-2026-07-27; PERF-RERUN-001; ROUTE-BC-DEFER; ROUTE-DOC-DEFER-2026-07-21; CV-008 (deferred to STORY-180 delivery); ROUTE-W74-OBS-2.
- **Spec versions:** BC-INDEX v2.37 / VP-INDEX v2.46 / ARCH-INDEX v2.20 / PRD v1.59 / STORY-INDEX v3.91 / HS-INDEX v2.17 / dep-graph v3.10.
- **Resume command:** `/vsdd-factory:next-step`

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001 v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001.

---

## Historical Content

| Content | Location |
|---------|----------|
| **Decision Log D-302..D-436 (exhaustive)** | `cycles/history/decision-log-archive.md` |
| **feature-iec104 decisions archive (exhaustive)** | `cycles/feature-iec104/decisions-archive.md` (D-437 through D-458) |
| **Resolved Open Items (pre-feature-iec104)** | `cycles/history/open-items-archive.md` |
| **Resolved Carry-Forwards (feature-iec104)** | `cycles/feature-iec104/blocking-issues-resolved.md` |
| **feature-iec104 cycle-close lessons** | `cycles/feature-iec104/lessons.md` |
| **feature-iec104 burst log** | `cycles/feature-iec104/burst-log.md` |
| **Phase Progress granular rows (F4 waves/adversary/fixes)** | `cycles/feature-iec104/phase-progress-archive.md` |
| **Convergence Trajectory (F4 per-story + F5 phase)** | `cycles/feature-iec104/convergence-trajectory.md` |
| feature-iec104 F2 convergence report | `cycles/feature-iec104/adversarial/f2-convergence-report.md` |
| Session checkpoints (feature-iec104, all prior) | `cycles/feature-iec104/session-checkpoints.md` |
| Wave 71–75 + Wave 82–83 gate files | `cycles/wave-{71..75,82,83}/wave-gate/` |
| **Wave 84 gate artifacts** | `cycles/wave-084/wave-gate/` (gate-summary.md D-486, code-review.md) |
| **Wave-84 lessons (S-7.02 COMPLETE)** | `cycles/wave-084/lessons.md` |
| **Wave-084 burst log + session checkpoints** | `cycles/wave-084/burst-log.md` + `cycles/wave-084/session-checkpoints.md` |
| STORY-147/166/176 per-story convergence reports | `cycles/wave-084/STORY-{147,166,176}/convergence-report.md` |
| **STORY-180 per-story convergence report** | `cycles/wave-085/STORY-180/convergence-report.md` + `adversary-convergence-state.json` |
| feature-iec104 F5–F7 adversarial/hardening/convergence | `.factory/phase-f5-adversarial/` + `.factory/phase-f6-hardening/` + `.factory/phase-f7-convergence/` |
