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
current_step: "D-515 DF-VALIDATION-001 BATCH COMPLETE (2026-07-25). 14 PG findings validated (PG-W84-001/002/003/004/005/006/008 + PG-W85-001..005 + PG-W84-010/012). Rollup: 8 DUP / 2 NOVEL-UPSTREAM / 4 LOCAL-CARRY-FORWARD. Upstream: #764 (PG-W84-006) + #765 (PG-W85-001) filed; evidence comments on #749/#681/#663/#626. Steady-state idle. Pipeline ACTIVE. trajectory-tail →0→0→0→0"
current_cycle: "wave-085"
pipeline: ACTIVE
timestamp: 2026-07-25T14:00:00Z
released_version: v0.13.2
released_at: "2026-07-25"
release_tag: v0.13.2
release_tag_object: 9601d711baf72ca30d29be2c289271ade5d027cc
release_commit: 9601d711baf72ca30d29be2c289271ade5d027cc
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.13.2
prior_released_version: v0.13.1
prior_released_at: "2026-07-21"
main_head: 9601d711baf72ca30d29be2c289271ade5d027cc
develop_head: e8841d761f3f25f320f98977618e506e8b41a058
cargo_version_main: "0.13.2"
cargo_version_develop: "0.13.2"
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
stories_delivered: 118
story_index_version: "v3.94"
total_stories: 134
story_index_note: "134 stories / 85 waves / 783 pts. v3.94 (2026-07-24): WAVE-85 GATE CLOSED (D-510) — STORY-181 Dependencies cell corrected '#438'→'—' (CV-W85G-001); BC-2.19.029 v1.4 + BC-2.19.030 v1.3 PO label refreshes (CV-W85G-002); input-hash 22 re-baselined (annotation/index churn, canonical tool). No numeric totals changed. v3.93 (2026-07-24): STORY-181 DELIVERED (D-509, PR #438 5555495b squash-merged to develop 2026-07-24T20:26:06Z, human-executed post-MERGE-AUTH-HALT; DF-MERGE-AUTH-CLASSIFIER-001 satisfied; CI 13/13; pr-reviewer APPROVE cycle 1, 0 blocking; security 0C/0H/0M; Step-4.5 CONVERGED 3/3 D-508); status ready→delivered; wave-85 Delivery Progress 2/2 DELIVERED CLOSED-PENDING-GATE; stories_delivered 117→118. PG-W85-004 NEW. STORY-INDEX v3.92→v3.93. No numeric points/story/wave totals changed. v3.92 (2026-07-24): STORY-180 DELIVERED (D-507, PR #437 421bf572 squash-merged to develop 2026-07-24T18:44:47Z, human-executed post-classifier-halt; DF-MERGE-AUTH-CLASSIFIER-001 satisfied; CI 13/13; stories_delivered 116→117). STORY-INDEX v3.91→v3.92; no numeric totals changed. v3.91 (2026-07-24): WAVE-85 HUMAN STORY-APPROVAL GATE PASSED (D-505) — STORY-180/181 status draft→ready; STORY-INDEX v3.90→v3.91; no numeric totals changed. v3.90 (2026-07-24): pre-gate remediation burst (D-504) — index-body currency corrections: wave count 83→85 (wave-84 STORY-147/166/176 + wave-85 STORY-180/181), dep-graph v3.9→v3.10 (STORY-174→STORY-180 edge, 137→138 acyclic edges), E-22 epic row updated; no numeric story/points totals changed. v3.89 (2026-07-23): STORY-181 title-cell correction (F-P4-001 pass-4 adversary remediation, D-498) — Direction-Keyed Carry Select framing removed from STORY-181 title cell; correct framing Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop now consistent with STORY-181 body (FSR line 262), AC-181-003 trace (line 119), and risk-register.md R-010; no numeric totals changed. v3.88 (2026-07-23): wave-85 STORY-CREATION BURST (D-493) — STORY-180 (IEC-104 timed control-command detection TypeIDs 58–64, E-22, 5 pts, wave 85, BC-2.19.029+030+022 v1.1 regression guard) + STORY-181 (SEC-001 ENIP split-borrow refactor + ROUTE-W74 OBS-1, E-20, 3 pts, wave 85, BC-2.17.016); BC-2.19.022 v1.1 propagation sweep: STORY-170 v2.0→v2.1 (AC-170-005/006 silently-logged range 52–99→{52–57,65–99}, BC table annotated); total_stories 132→134; total_points 775→783; total_waves 84→85; wave-table scheduled 692→700. v3.87 (2026-07-21): Epic table TOTAL cell arithmetic corrected 776→775 (SPEC-009); per-epic sum = 775 = frontmatter total_points; root cause: v3.79 re-scope delta decremented E-11 row (67→66) but TOTAL cell not updated; no other numeric changes; maint-2026-07-21 D-490. STORY-INDEX v3.86→v3.87. v3.86 (2026-07-21): E-16/E-17 ARP stale-draft supersession (D-487, 2026-07-21) — 7 drafts STORY-111..117 status draft→superseded DELIVERED-BY-DRIFT; E-16 v0.7.0 (STORY-111..115, 47 pts, waves 40-44) + E-17 v0.7.0/v0.7.1 (STORY-116/117, 8 pts, waves 45-46); twice-research-validated DF-VALIDATION-001 + human-approved; wave-table scheduled 747→692; total_points 775 unchanged per D-477/D-480 supersession-convention. STORY-INDEX v3.85→v3.86. v3.85 (2026-07-21): WAVE-84 GATE CLOSED (D-486); wave-84 delivery row updated CLOSED-PENDING-GATE→CLOSED (D-486, 2026-07-21); story-file status loci synced (STORY-147/166/176 frontmatter+body status: ready→delivered, three-loci agreement with STORY-INDEX rows at v3.84). No numeric totals changed."
bc_index_version: "v2.37"
vp_index_version: "v2.47"
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
    Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 319 = 181 (dual-margin form). ~319 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-515 DF-VALIDATION-001 BATCH COMPLETE (2026-07-25). Session D-514/D-515: RESUMED (D-514, human-approved) + 14 PG findings research-validated. Rollup: 8 DUPLICATE / 2 NOVEL-UPSTREAM / 4 LOCAL-CARRY-FORWARD / 0 ALREADY-FIXED / 0 INCONCLUSIVE. Report: .factory/planning/df-validation-2026-07-25.md. Upstream: #764 (PG-W84-006) + #765 (PG-W85-001) filed; evidence comments on #749 (PG-W84-001), #681 (PG-W84-003), #663 (PG-W84-008), #626 (PG-W85-004). Pending: DEP-SOAK-FOLLOWUP-2026-07-27 (eligible 2026-07-27, incl. #434/#435/#436); local fixes PG-W85-005 (3 candidates) + PG-W84-010/PG-W85-003 (combined bin/check-green-doc-tense story) + PG-W84-012; ROUTE-W74-OBS-2; PR #407; remaining carry-forwards unchanged. Steady-state idle. Pipeline ACTIVE.**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); **RELEASED v0.13.0 (D-473, 2026-07-18). F1→F7 CONVERGED; CYCLE CLOSED (D-475, 2026-07-18): S-7.02 SATISFIED. D-477: STORY-175/177/178/179 codification VEHICLE CHANGED to upstream (see D-477). D-480: E-11 disposition burst #2 — STORY-091/121/143/155 superseded; STORY-147 v2.0 local survivor. WAVE-84 OPENED (STORY-166/176/147v2, 7 pts, all product-local). D-481: STORY-147 DELIVERED (PR #421 f0cb7374). D-482: STORY-166 DELIVERED (PR #426 fa9be701). D-485: STORY-176 DELIVERED (PR #427 595cdba8) — wave-84 3/3 DELIVERY COMPLETE. D-486: WAVE-84 GATE CLOSED + S-7.02 COMPLETE (2026-07-21). D-487: E-16/E-17 ARP stale-draft supersession; backlog EMPTY. D-488: SESSION WRAP (2026-07-21). D-489: SESSION RESUMED + maintenance sweep maint-2026-07-21 STARTED (2026-07-21). D-490: maint-2026-07-21 COMPLETE (2026-07-21). D-491: v0.13.1 RELEASED (2026-07-21). D-492: SESSION WRAP (2026-07-21). D-493: SESSION RESUMED + WAVE-85 SCOPED (2026-07-23). D-494: WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE (2026-07-23); STORY-180/181 drafted; adversarial convergence next. D-506: STORY-180 Step-4.5 CONVERGED 3/3 (BC-5.39.001). D-507: STORY-180 DELIVERED (PR #437 421bf572, 2026-07-24); stories_delivered 116→117; VP-INDEX v2.47 (CV-008 RESOLVED). D-508: STORY-181 Step-4.5 ADVERSARIAL CONVERGED (2026-07-24) — 3/3 passes clean (P1/P2/P3); BC-5.39.001 SATISFIED. D-509: STORY-181 DELIVERED (PR #438 5555495b, 2026-07-24); stories_delivered 117→118; wave-85 DELIVERY COMPLETE 2/2; CLOSED-PENDING-GATE. D-510: WAVE-85 GATE CLOSED (pending human approval, 2026-07-24). D-511: WAVE-85 GATE APPROVED + CYCLE CLOSED (2026-07-25). S-7.02 COMPLETE. D-512: v0.13.2 RELEASED (2026-07-25).** |
| Version | 0.13.2 (released 2026-07-25; main=9601d711; develop=e8841d76 — D-512 v0.13.2 RELEASED (patch, human-directed)) |
| Main HEAD | `9601d711baf72ca30d29be2c289271ade5d027cc` |
| Develop HEAD | `e8841d761f3f25f320f98977618e506e8b41a058` — D-512 v0.13.2 RELEASED (2026-07-25); back-merge PR #441 TRUE-MERGE |
| Spec versions | BC-INDEX v2.37 / VP-INDEX v2.47 / ARCH-INDEX v2.20 / PRD v1.59 |
| Stories | 118 delivered / 134 total (STORY-INDEX v3.94, dep-graph v3.10, 783 pts) |
| **Last Updated** | 2026-07-25 — D-515 DF-VALIDATION-001 BATCH COMPLETE (2026-07-25). 14 PG findings validated; 8 DUP / 2 NOVEL-UPSTREAM / 4 LOCAL-CARRY-FORWARD. Pipeline ACTIVE. trajectory-tail →0→0→0→0 |

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
| **Wave 85 (IEC-104 completion mini-wave)** | **CLOSED (D-511, 2026-07-25)** | Spec-evolution + story-creation COMPLETE (D-494). Story-level adversarial CONVERGED (D-501..503, 9 passes, streak P7/P8/P9). STORY-180 DELIVERED (D-507, PR #437 421bf572). STORY-181 DELIVERED (D-509, PR #438 5555495b). 2/2 DELIVERY COMPLETE. Gate-fix PR #439 0ab6f52e (ITI e2e 31→66). Gate-summary + code-review + lessons.md authored (D-510). Human gate-ratified (D-511, 2026-07-25): streak P1/P2/P3 accepted, PG-W85-005 deferral, HS-136 0.9 caveat, holdout runs as wave integration demos. develop=0ab6f52e. S-7.02 COMPLETE. trajectory-tail →0→0→0→0 |
| pass-85 gate adversary (wave-85 gate, 3 passes, streak P1/P2/P3) | CONVERGED (D-510) | All 3 passes NITPICK_ONLY. Zero CRIT/HIGH/MED. Trajectory: NITPICK/1L(P1)→NITPICK/0(P2)→NITPICK/2L-factory+1I(P3). Code frozen 0ab6f52e. DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. trajectory-tail →0→0→0→0 |
| **v0.13.2 RELEASED** | **RELEASED 2026-07-25** | PR #440 9601d711 main (human-merged); tag v0.13.2 (lightweight); GH release 4 assets; back-merge PR #441 TRUE-MERGE e8841d76 develop (ancestry PASS — no DRIFT-BACKMERGE-SQUASH recurrence). Ships: IEC-104 timed-command detection (TypeIDs 58-64) + SEC-001 ENIP unsafe elimination + gate-fix. Version 0.13.1→0.13.2. |

---

## Convergence Status

Per-story F4 convergence details archived to `cycles/feature-iec104/convergence-trajectory.md`.
F5 phase-level trajectory: 5 rounds, code frozen R2, `5H/M→2M→1H→1M→1L(NB)` — CONVERGED (D-468).
Wave-84 gate-level adversarial trajectory (6 passes, code frozen 1e967bad): `1M→M/L-batch→1L→0→0→0` — CONVERGED (D-486). Streak P4/P5/P6.
Wave-85 story adversarial trajectory (CONVERGED): `1C+2H+4M+2L(P1)→3M/1L(P2)→1M(P3)→1H(P4)→NITPICK/1L(P5)→1M/2L(P6)→NITPICK/2L(P7 1/3)→CLEAN/0(P8 2/3)→NITPICK/1L-closed(P9 3/3) → CONVERGED 3/3 (P7/P8/P9)` — BC-5.39.001 SATISFIED. trajectory-tail →0→0→0→0.
Wave-85 STORY-180 per-story adversarial trajectory (CONVERGED D-506): `3M(P1)→NITPICK/3L(P2)→NITPICK/1L(P3)→NITPICK/1L(P4) → CONVERGED 3/3 (P2/P3/P4)` — BC-5.39.001 SATISFIED. Commits a0087033/e40955f1/0502c642. Demo head ccec1711. trajectory-tail →0→0→0→0.
Wave-85 STORY-180 DELIVERED (D-507, 2026-07-24): PR #437 421bf572 squash-merged to develop; stories_delivered 116→117; VP-047 source_bc updated (CV-008 RESOLVED). VP-INDEX v2.47.
Wave-85 STORY-181 per-story adversarial trajectory (CONVERGED D-508): `NITPICK/2L(P1)→NITPICK/2L(P2)→CLEAN/0(P3) → CONVERGED 3/3 (P1/P2/P3)` — BC-5.39.001 SATISFIED. Commits 224311a1/13491355/e9572820 + sweeps 294168fa/093ff519. O-181-P3-001 theoretical non-blocking. trajectory-tail →0→0→0→0.
Wave-85 STORY-181 DELIVERED (D-509, 2026-07-24): PR #438 5555495b squash-merged to develop; stories_delivered 117→118; SEC-001 RESOLVED (zero unsafe in enip.rs); ROUTE-W74 OBS-1 RESOLVED (AC-181-004). WAVE-85 DELIVERY COMPLETE (2/2). CLOSED-PENDING-GATE.
Wave-85 gate-level adversarial trajectory (3 passes, code frozen 0ab6f52e): `NITPICK/1L(P1)→NITPICK/0(P2)→NITPICK/2L-factory+1I(P3) → CONVERGED 3/3 (P1/P2/P3)` — DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. trajectory-tail →0→0→0→0.

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | **CLOSED (D-475, 2026-07-18)** — v0.13.0 RELEASED (D-473); F1→F7 CONVERGED (D-470); S-7.02 SATISFIED. D-477: STORY-175/177/178/179 vehicles changed to upstream; STORY-176 v2.0 + STORY-166 local survivors | develop (1e967bad) |
| wave-084 (E-11 mini-wave) | **CLOSED (D-486, 2026-07-21)** — 3/3 DELIVERED + gate CLOSED; S-7.02 COMPLETE; 12 PG-W84 entries (3 FIXED / 9 deferred to DF-VALIDATION-001 batch). develop=1e967bad (PR #430 gate-fix final). trajectory-tail →0→0→0→0 | develop (1e967bad, D-486 gate-close) |
| wave-085 (IEC-104 completion mini-wave) | **CLOSED (D-511, 2026-07-25)** — STORY-180 DELIVERED (D-507, PR #437 421bf572); STORY-181 DELIVERED (D-509, PR #438 5555495b). 2/2 DELIVERY COMPLETE. Gate-fix PR #439 0ab6f52e. Gate 3 adversary CONVERGED 3/3 (NITPICK_ONLY P1/P2/P3). Holdout mean 0.98. S-7.02 COMPLETE. Human gate-ratified (D-511, 2026-07-25). **v0.13.2 RELEASED (D-512, 2026-07-25)** from this wave. develop=e8841d76. trajectory-tail →0→0→0→0 | develop (e8841d76, D-512 RELEASED) |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-515 DF-VALIDATION-001 BATCH COMPLETE (2026-07-25). Research-agent validated 14 PG findings. Rollup: 8 DUPLICATE / 2 NOVEL-UPSTREAM / 4 LOCAL-CARRY-FORWARD. Report: planning/df-validation-2026-07-25.md. Upstream: #764 (PG-W84-006) + #765 (PG-W85-001) filed; evidence comments on #749 (PG-W84-001), #681 (PG-W84-003), #663 (PG-W84-008), #626 (PG-W85-004). Steady-state idle. Pipeline ACTIVE.** | **COMPLETE (D-515)** | DF-VALIDATION-001 batch complete. 8 DUP / 2 upstream filed / 4 local carry-forwards. |
| **D-514 SESSION RESUMED (human-approved, 2026-07-25) from D-513 pause. Worktree health PASS. Human selected DF-VALIDATION-001 batch as session work.** | **COMPLETE (D-514)** | Session resumed. DF-VALIDATION-001 batch selected. |
| **D-513 SESSION WRAP (2026-07-25). Human /wrap at clean post-v0.13.2 milestone. Session D-504..D-512 (exhaustive): pre-gate consistency audit (D-504, 8 findings remediated); D-505 human story gate PASSED; STORY-180 DELIVERED (D-507, PR #437 421bf572); STORY-181 DELIVERED (D-509, PR #438 5555495b; SEC-001 CLOSED); gate-fix PR #439 0ab6f52e; wave-085 gate CONVERGED 3/3 + CLOSED (D-511, S-7.02 COMPLETE); v0.13.2 RELEASED (D-512, main=9601d711; back-merge PR #441 ancestry PASS). No in-flight work; no story worktrees; no abandoned sub-agent steps; all product branches merged and deleted. Pipeline PAUSED.** | **COMPLETE (D-513)** | Session wrap. wave-085 CLOSED; v0.13.2 RELEASED; Pipeline PAUSED. trajectory-tail →0→0→0→0 |
| **D-512 v0.13.2 RELEASED (2026-07-25). Patch bump (human-directed). Release PR #440 9601d711 main (human-merged); tag v0.13.2 (lightweight); GH release 4 assets; back-merge PR #441 TRUE-MERGE e8841d76 develop (human-authorized gh pr merge --merge; ancestry PASS — no DRIFT-BACKMERGE-SQUASH recurrence). Ships wave-85: IEC-104 timed-command detection (TypeIDs 58-64) + SEC-001 ENIP unsafe elimination + gate-fix. CR-004 CHANGELOG trim applied at release cut. Version 0.13.1→0.13.2. main=9601d711. develop=e8841d76. Pipeline ACTIVE at idle. trajectory-tail →0→0→0→0** | **COMPLETE (D-512)** | v0.13.2 RELEASED. PR #440 main (human-merged); back-merge PR #441 TRUE-MERGE. Ships wave-85 content. trajectory-tail →0→0→0→0 |
| **D-511 WAVE-85 GATE APPROVED + CYCLE CLOSED (2026-07-25). Human gate: all 6 gates ratified — streak P1/P2/P3 accepted; PG-W85-005 deferral (3 candidate fixes remain open per lessons.md §PG-W85-005) accepted; HS-136 0.9 corpus caveat (not a product defect) accepted; holdout real-capture runs accepted as wave integration demos. Wave-085 CLOSED. S-7.02 COMPLETE (PG-W85-001..005 dispositioned in cycles/wave-085/lessons.md). develop=0ab6f52e. Pipeline ACTIVE at idle. Backlog: ROUTE-W74-OBS-2, PG-W84+PG-W85 DF-VALIDATION-001 batches, DEP-SOAK-FOLLOWUP-2026-07-27, Dependabot #434/#435 deferred to soak, PR #407 governance, PERF-RERUN-001, ROUTE-BC/DOC defers. Next: human choice (v0.14.0 release candidacy — two [Unreleased] entries incl. one Added feature — minor bump candidate). trajectory-tail →0→0→0→0** | **COMPLETE (D-511)** | WAVE-85 GATE APPROVED + CYCLE CLOSED. S-7.02 COMPLETE. Pipeline idle. trajectory-tail →0→0→0→0 |


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
| D-506 | STORY-180 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 4 fresh-context passes; streak P2/P3/P4 = 3/3 clean; zero open HIGH/CRIT; BC-5.39.001 SATISFIED. Trajectory 3M(P1)→3L(P2)→1L(P3)→1L(P4). Remediation commits a0087033/e40955f1/0502c642 on feature branch; demo head ccec1711 (8 artifacts, PG-W70-DEMO-SCRUB PASSED). Red Gate PASSED (21 red/227 green). BC-2.19.029 v1.3 / BC-2.19.030 v1.2 (draft→ready label refresh, F-180-P4-001); BC-INDEX v2.36→v2.37. PG-W85-003 NEW: bin/check-green-doc-tense pattern set misses 'Expected RED:'/'currently falls through' stale-RED phrasing class (adversary pass-1 observation) — queued DF-VALIDATION-001 batch. pr-manager PR lifecycle next (STORY-180). trajectory-tail →0→0→0→0 | 2026-07-24 |
| D-507 | STORY-180 DELIVERED (2026-07-24). PR #437 421bf572 squash-merged to develop 2026-07-24T18:44:47Z, human-executed post-classifier-halt. DF-MERGE-AUTH-CLASSIFIER-001 satisfied (wave-84 pattern #421/#426/#427/#437). CI 13/13 green. pr-reviewer APPROVE cycle 1 (0 blocking; self-authored COMMENTED+review-findings.md = review of record). Security CLEAN. Step-4.5 CONVERGED 3/3 (D-506, BC-5.39.001). stories_delivered 116→117. STORY-INDEX v3.91→v3.92 (D-507). VP-INDEX v2.46→v2.47: VP-047 source_bc += BC-2.19.029/030 (CV-008 RESOLVED). STORY-180 input-hash rebaselined 8ddf419→e87befe (canonical Python; BC-2.19.029 v1.3 + BC-2.19.030 v1.2 changed). Worktree cleaned. Next: per-story delivery STORY-181. | 2026-07-24 |
| D-508 | STORY-181 STEP-4.5 ADVERSARIAL CONVERGED (2026-07-24). 3 fresh-context passes; streak P1/P2/P3 = 3/3 clean; zero open HIGH/CRIT; BC-5.39.001 SATISFIED. Trajectory NITPICK/2L(P1)→NITPICK/2L(P2)→CLEAN/0(P3). Remediation sweeps 294168fa (P1: pdu_queue invariant + flow_key param doc) + 093ff519 (P2: RULING-137-002 cross-ref + line ~1033 precision). O-181-P3-001 LOW theoretical (panic-unwind debug_assert-only, compiled out in release) accepted non-blocking. SEC-001 fix verified: zero unsafe in enip.rs. ROUTE-W74 OBS-1 closed (AC-181-004). Convergence state + report: cycles/wave-085/STORY-181/. Next: demo evidence then pr-manager 9-step lifecycle. | 2026-07-24 |
| D-509 | STORY-181 DELIVERED (2026-07-24). PR #438 5555495bbcdb3b0d4088a21c77aa6cc24e9ce7f3 squash-merged to develop 2026-07-24T20:26:06Z, human-executed post-MERGE-AUTH-HALT. DF-MERGE-AUTH-CLASSIFIER-001 satisfied (same pattern #421/#426/#427/#437/#438). CI 13/13 green. pr-reviewer APPROVE cycle 1 (0 blocking; self-authored COMMENTED+review-findings.md = review of record; issuecomment-5073978095). Security 0C/0H/0M. Step-4.5 CONVERGED 3/3 (D-508, BC-5.39.001). stories_delivered 117→118. STORY-INDEX v3.92→v3.93. WAVE-85 DELIVERY COMPLETE (2/2). CLOSED-PENDING-GATE. PG-W85-004 NEW: pr-manager attempted `gh pr review --approve` on self-authored PR #438 — blocked by two-party harness guard; no approval event landed (orchestrator-verified reviews list empty); flag as process-gap → DF-VALIDATION-001 batch + upstream candidate. Worktree cleaned. Next: wave-85 integration gate. | 2026-07-24 |
| D-510 | WAVE-85 GATE CLOSED (2026-07-24, pending human approval). Gate-fix PR #439 0ab6f52e (ITI e2e 31→66, derived decomposition +35 timed = 15×TypeID-58/59 + 10×TypeID-61/63×2; T1692.001 46, T0836 20). 6-gate results: G1 PASS after gate-fix #439 0ab6f52e; G2 SKIP (DTU n/a); G3 adversary CONVERGED 3/3 (P1/P2/P3, all NITPICK_ONLY, zero CRIT/HIGH/MED; DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED); G3b security APPROVE 0C/0H/0M/0L + consistency 3 MINOR (CV-W85G-001/002/003) fixed + code review 0 MAJOR/1 MINOR/5 NIT dispositioned; G4 demo evidence PASS (STORY-180 8 artifacts + STORY-181 5 artifacts, scrub PASSED); G5 holdout PASS mean 0.98 (HS-133/134/135 1.0, HS-136 0.9 corpus caveat not a product defect, ENIP HS-118/120 1.0 no regression). Input-hash re-baseline 22 stories (annotation/index churn). Final scan MATCH=134 STALE=0. S-7.02: PG-W85-001..005 all dispositioned in cycles/wave-085/lessons.md. BC-2.19.029 v1.4 / BC-2.19.030 v1.3 (PO edits, CV-W85G-002). STORY-INDEX v3.93→v3.94. tech-debt-register v2.1→v2.2 (ROUTE-W74 OBS-1 resolved, SEC-001 line-cite 992-999→993-1000, CR-W85G-001 deferred). develop=0ab6f52e frozen. Next: human wave-85 gate approval, then wave-085 cycle CLOSED. | 2026-07-24 |
| D-511 | WAVE-85 GATE APPROVED + CYCLE CLOSED (human gate, 2026-07-25). Structured questions presented — all accepted: streak P1/P2/P3 pedigree accepted; PG-W85-005 deferral (3 candidate fixes in lessons.md) accepted; HS-136 0.9 corpus caveat (not a product defect) accepted; holdout real-capture runs accepted as wave integration demos. Wave-085 CLOSED. S-7.02 COMPLETE (PG-W85-001..005 dispositioned per cycles/wave-085/lessons.md). develop=0ab6f52e. Session D-504..D-511 (exhaustive): D-504 pre-gate remediation, D-505 story gate, STORY-180 DELIVERED (#437), STORY-181 DELIVERED (#438, SEC-001 closed), gate-fix #439, wave gate CONVERGED+CLOSED. Backlog: ROUTE-W74-OBS-2 (human scope decision), PG-W84+PG-W85 DF-VALIDATION-001 batches, DEP-SOAK-FOLLOWUP-2026-07-27 (dated), Dependabot #434/#435 deferred to soak, PR #407 governance, PERF-RERUN-001, ROUTE-BC/DOC defers. Pipeline ACTIVE at idle; next work is human choice (release candidacy v0.14.0: two [Unreleased] entries incl. one Added feature — minor bump candidate). | 2026-07-25 |
| D-512 | v0.13.2 RELEASED (2026-07-25). Patch bump (human-directed). Release PR #440 9601d711 main (human-merged); tag v0.13.2 (lightweight); GH release 4 assets; back-merge PR #441 TRUE-MERGE e8841d76 develop (human-authorized gh pr merge --merge; ancestry PASS — no DRIFT-BACKMERGE-SQUASH recurrence). Ships wave-85: IEC-104 timed-command detection (TypeIDs 58-64) + SEC-001 ENIP unsafe elimination + gate-fix. CR-004 disposition executed at release cut. Version 0.13.1→0.13.2. | 2026-07-25 |
| D-513 | SESSION WRAP (2026-07-25). Human-requested pause at clean post-v0.13.2 milestone. Session D-504..D-512 (exhaustive): consistency audit remediation; D-505 story gate; STORY-180/181 DELIVERED (PRs #437/#438); wave-85 gate-fix #439; wave-085 gate CONVERGED+CLOSED (D-511, S-7.02 COMPLETE); v0.13.2 RELEASED (D-512). Backlog recorded in checkpoint. No in-flight work, no story worktrees, no factory lock. Pipeline PAUSED. | 2026-07-25 |
| D-514 | Session RESUMED (human-approved, 2026-07-25) from D-513 pause. Worktree health PASS. Human selected DF-VALIDATION-001 batch as session work. | 2026-07-25 |
| D-515 | DF-VALIDATION-001 BATCH COMPLETE (2026-07-25). Research-agent validated 14 PG findings. Rollup: 8 DUP / 2 NOVEL-UPSTREAM (filed #764 PG-W84-006, #765 PG-W85-001) / 4 LOCAL-CARRY-FORWARD (PG-W84-010, PG-W84-012, PG-W85-003, PG-W85-005). Evidence comments: #749 (PG-W84-001), #681 (PG-W84-003), #663 (PG-W84-008), #626 (PG-W85-004). Report: planning/df-validation-2026-07-25.md. | 2026-07-25 |

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
| PG-W84-UPSTREAM-BATCH | **RESOLVED (D-515, 2026-07-25).** Research-validated: 001 DUP #749 (comment posted); 002 DUP #457; 003 DUP #681 (comment posted); 004 DUP #572; 005 DUP #651/#626; 006 FILED #764; 008 DUP #663 (comment posted). | wave-084 S-7.02 (D-486) | RESOLVED — archive at next compact. |
| PG-W84-LOCAL-BATCH | **VALIDATED LOCAL-CARRY-FORWARD (D-515, 2026-07-25).** PG-W84-010 (bin/check-green-doc-tense) + PG-W84-012 (bin-selftest required-status-check) confirmed product-local; no upstream filing. PG-W84-010 must ship as combined story with PG-W85-003. | wave-084 S-7.02 (D-486) | Local carry-forwards — story delivery |
| PG-W85-001 | **RESOLVED (D-515, 2026-07-25).** NOVEL-UPSTREAM — filed drbothen/vsdd-factory#765. | wave-085 pass-2 (D-496) | RESOLVED — archive at next compact. |
| PG-W85-002 | **RESOLVED-DUPLICATE (D-515, 2026-07-25).** Class covered by #470/#507/#216. Local DF-SIBLING-SWEEP-001 extension proposal recorded in cycles/wave-085/lessons.md codification table. | wave-085 P2-P4 (D-496/497/498) | RESOLVED — archive at next compact. |
| PG-W85-003 | **VALIDATED LOCAL-CARRY-FORWARD (D-515, 2026-07-25).** bin/check-green-doc-tense pattern gap. MUST ship as one combined story with PG-W84-010 (both target bin/check-green-doc-tense). | wave-085 STORY-180 pass-1 (D-506) | Combined local story — bin/check-green-doc-tense |
| PG-W85-004 | **RESOLVED-DUPLICATE (D-515, 2026-07-25).** Covered by #626 (primary) + #696/#651. Observed-attempt evidence posted as comment on #626. | wave-085 D-509 (2026-07-24) | RESOLVED — archive at next compact. |
| PG-W85-005 | **VALIDATED LOCAL-CARRY-FORWARD (D-515, 2026-07-25).** Gitignored machine-local e2e fixtures cause false-green. Substantive open local fix; 3 candidate fixes per cycles/wave-085/lessons.md §PG-W85-005. | wave-085 gate G1 (D-510) | Local carry-forward — 3 candidate fixes |

---

## Active Carry-Forwards

| ID | Summary | Target |
|---|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11. | Next maintenance run |
| ROUTE-W74-DEFERRED | **RESOLVED (D-509, 2026-07-24)** — Code-review NIT deferred wave-74; OBS-1 absorbed by STORY-181 AC-181-004 (PR #438 delivered). OBS-2 remains open per ROUTE-W74-OBS-2 row. | RESOLVED (OBS-1); OBS-2 per ROUTE-W74-OBS-2 |
| ROUTE-W74-OBS-2 | ROUTE-W74 OBS-2 not absorbed by STORY-166/181. Pending human scope decision. | Next wave or maintenance run |
| PERF-RERUN-001 | AC-149-003 re-run PASS at maint-2026-07-21. Remains OPEN per human scope decision D-490. | Next maintenance run |
| SEC-001 | **RESOLVED (D-509, 2026-07-24)** — SEC-001-ENIP (split-borrow) deferred maint-2026-07-11; absorbed into STORY-181 (wave-85). PR #438 5555495b delivered, zero unsafe in enip.rs. | CLOSED |
| PR-407-FORK-RELEASE-OPS | External ArcavenAE PR #407 SAFE-WITH-CHANGES (D-472); DEFERRED — governance pending. | Governance decision when authorized |
| SCORECARD-ENABLEMENT-RUNBOOK | Before setting SCORECARD_ENABLED=true: document CWE-200 publish_results:true risk. | Whenever scorecard is enabled |
| DEP-SOAK-FOLLOWUP-2026-07-27 | 17 not-yet-soaked crates eligible 2026-07-21..27; Dependabot #434/#435/#436 included. Run next soak on/after 2026-07-27. | Next maintenance run on/after 2026-07-27 |
| ROUTE-DOC-DEFER-2026-07-21 | PR #431 review residuals: ADR-0001 Consequences (LOW), ADR-0002 Deviations (NIT), ADR-0012 stale 'supported: 7 protocols' (LOW). | Next doc sweep |

---

## Session Resume Checkpoint

**D-515 DF-VALIDATION-001 BATCH COMPLETE (2026-07-25). Session D-514/D-515: RESUMED (D-514, human-approved) + 14 PG findings research-validated. Rollup: 8 DUPLICATE / 2 NOVEL-UPSTREAM / 4 LOCAL-CARRY-FORWARD / 0 ALREADY-FIXED / 0 INCONCLUSIVE. Report: .factory/planning/df-validation-2026-07-25.md. Upstream: #764 (PG-W84-006) + #765 (PG-W85-001). Steady-state idle. Pipeline ACTIVE.**

Prior checkpoints archived to `cycles/feature-iec104/session-checkpoints.md` and `cycles/wave-084/session-checkpoints.md` and `cycles/wave-085/session-checkpoints.md`.

- **Date:** 2026-07-25. Position: steady-state idle, DF-VALIDATION-001 batch COMPLETE; no active wave; no in-flight work.
- **Convergence counters:** NONE active. Wave-85 story-level + gate-level both SATISFIED and closed. Do NOT re-run either.
- **In-flight:** NONE. All bursts committed. Tree clean.
- **PENDING NEXT STEPS (in order) on resume:** (a) DEP-SOAK-FOLLOWUP-2026-07-27 (eligible on/after 2026-07-27; includes Dependabot #434/#435/#436); (b) local fix candidates: PG-W85-005 (3 candidate fixes per lessons.md §PG-W85-005) + PG-W84-010/PG-W85-003 combined story (bin/check-green-doc-tense) + PG-W84-012 (bin-selftest required-status-check); (c) ROUTE-W74-OBS-2 human scope decision; (d) PR #407 governance; (e) PERF-RERUN-001; (f) ROUTE-BC-DEFER + ROUTE-DOC-DEFER-2026-07-21; (g) STORY-INDEX-IN-INPUTS-CHURN structural fix (pending human decision).
- **Ground truth:** develop=e8841d761f3f25f320f98977618e506e8b41a058 (back-merge PR #441 TRUE-MERGE; main IS ancestor), main=9601d711baf72ca30d29be2c289271ade5d027cc (v0.13.2). Cargo 0.13.2 on both branches. No open product worktrees. Open PRs: external #407 (DEFERRED) + Dependabot #434/#435/#436 (deferred to DEP-SOAK-FOLLOWUP-2026-07-27).
- **Pending human decisions:** DEP-SOAK-FOLLOWUP-2026-07-27; PR #407 governance; ROUTE-W74-OBS-2; STORY-INDEX-IN-INPUTS-CHURN; PERF-RERUN-001; ROUTE-BC-DEFER; ROUTE-DOC-DEFER-2026-07-21.
- **Note:** main-repo untracked bin/__pycache__/ is a harmless Python build artifact; candidate .gitignore addition at next hygiene sweep. DF-VALIDATION-001 batch COMPLETE — PG items dispositioned in Drift Items table.
- **Spec versions:** BC-INDEX v2.37 / VP-INDEX v2.47 / ARCH-INDEX v2.20 / PRD v1.59 / STORY-INDEX v3.94 / HS-INDEX v2.17 / dep-graph v3.10.
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
| **STORY-181 per-story convergence report** | `cycles/wave-085/STORY-181/convergence-report.md` + `adversary-convergence-state.json` |
| **Wave-85 gate artifacts** | `cycles/wave-085/wave-gate/` (gate-summary.md D-510, code-review.md) |
| **Wave-85 lessons (S-7.02 COMPLETE)** | `cycles/wave-085/lessons.md` |
| feature-iec104 F5–F7 adversarial/hardening/convergence | `.factory/phase-f5-adversarial/` + `.factory/phase-f6-hardening/` + `.factory/phase-f7-convergence/` |
