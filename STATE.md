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
current_step: "D-503 WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). 9-pass fresh-context adversarial convergence COMPLETE: streak P7/P8/P9 = 3/3 clean (pass-9 0C/0H/0M/0L — FULLY CLEAN); BC-5.39.001 SATISFIED; DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. F-W85S-P9-001 LOW (BC-2.19.019 parity reciprocity back-refs to BC-2.19.029/030 added, v1.1→v1.2) APPLIED. Package CONVERGED: BC-2.19.029/030 v1.1, BC-2.19.022 v1.1, BC-2.19.028 v1.1, STORY-180/181 (draft, ready for human story-approval gate), HS-133..136, PRD v1.59, STORY-INDEX v3.89, BC-INDEX v2.35, HS-INDEX v2.17. Human /wrap — pipeline PAUSED before human story-approval gate. develop=dc7331fb unchanged. trajectory-tail →0→0→0→0"
current_cycle: "wave-085"
pipeline: PAUSED
timestamp: 2026-07-23T23:27:00Z
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
story_index_version: "v3.89"
total_stories: 134
story_index_note: "134 stories / 85 waves / 783 pts. v3.89 (2026-07-23): STORY-181 title-cell correction (F-P4-001 pass-4 adversary remediation, D-498) — Direction-Keyed Carry Select framing removed from STORY-181 title cell; correct framing Eliminate *mut EnipFlowState Raw Pointer in PDU Dispatch Loop now consistent with STORY-181 body (FSR line 262), AC-181-003 trace (line 119), and risk-register.md R-010; no numeric totals changed. v3.88 (2026-07-23): wave-85 STORY-CREATION BURST (D-493) — STORY-180 (IEC-104 timed control-command detection TypeIDs 58–64, E-22, 5 pts, wave 85, BC-2.19.029+030+022 v1.1 regression guard) + STORY-181 (SEC-001 ENIP split-borrow refactor + ROUTE-W74 OBS-1, E-20, 3 pts, wave 85, BC-2.17.016); BC-2.19.022 v1.1 propagation sweep: STORY-170 v2.0→v2.1 (AC-170-005/006 silently-logged range 52–99→{52–57,65–99}, BC table annotated); total_stories 132→134; total_points 775→783; total_waves 84→85; wave-table scheduled 692→700. v3.87 (2026-07-21): Epic table TOTAL cell arithmetic corrected 776→775 (SPEC-009); per-epic sum = 775 = frontmatter total_points; root cause: v3.79 re-scope delta decremented E-11 row (67→66) but TOTAL cell not updated; no other numeric changes; maint-2026-07-21 D-490. STORY-INDEX v3.86→v3.87. v3.86 (2026-07-21): E-16/E-17 ARP stale-draft supersession (D-487, 2026-07-21) — 7 drafts STORY-111..117 status draft→superseded DELIVERED-BY-DRIFT; E-16 v0.7.0 (STORY-111..115, 47 pts, waves 40-44) + E-17 v0.7.0/v0.7.1 (STORY-116/117, 8 pts, waves 45-46); twice-research-validated DF-VALIDATION-001 + human-approved; wave-table scheduled 747→692; total_points 775 unchanged per D-477/D-480 supersession-convention. STORY-INDEX v3.85→v3.86. v3.85 (2026-07-21): WAVE-84 GATE CLOSED (D-486); wave-84 delivery row updated CLOSED-PENDING-GATE→CLOSED (D-486, 2026-07-21); story-file status loci synced (STORY-147/166/176 frontmatter+body status: ready→delivered, three-loci agreement with STORY-INDEX rows at v3.84). No numeric totals changed. v3.84 (2026-07-20): STORY-176 DELIVERED (D-485, PR #427 595cdba8 squash-merged to develop, human-executed merge 2026-07-20T21:46:45Z under explicit per-PR human authorization, DF-MERGE-AUTH-CLASSIFIER-001 satisfied; wave-84 #421/#426/#427 pattern match); status ready→delivered; wave-84 Delivery Progress row updated (3/3 DELIVERED — STORY-147 ✓, STORY-166 ✓, STORY-176 ✓; CLOSED-PENDING-GATE); CI 13/13 PASS (new \"Bin selftest suites\" step); pr-reviewer APPROVE (1 cycle, 0 blocking, 3 NITs accepted; self-authored PR — COMMENTED event + pr-review.md = review of record); security APPROVE (0C/0H/0M/1L pre-existing SEC-001 CWE-22); 8-pass Step-4.5 adversary CONVERGED P6/P7/P8 (BC-5.39.001 SATISFIED); story v2.7/6ec8772. stories_delivered 115→116. No numeric points/story/wave totals changed (status transition only)."
bc_index_version: "v2.35"
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
    Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 308 = 192 (dual-margin form). ~308 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-503 WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). 9-pass fresh-context convergence COMPLETE: streak P7/P8/P9 = 3/3 clean, zero open HIGH/CRITICAL (BC-5.39.001 + DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED). F-W85S-P9-001 LOW closed (BC-2.19.019 v1.1→v1.2, reciprocal parity back-refs to BC-2.19.029/030). wave-85 spec+story package CONVERGED, ZERO open findings. Pipeline PAUSED before consistency-validator audit + human story-approval gate. develop=dc7331fb unchanged. trajectory-tail →0→0→0→0.**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); **RELEASED v0.13.0 (D-473, 2026-07-18). F1→F7 CONVERGED; CYCLE CLOSED (D-475, 2026-07-18): S-7.02 SATISFIED. D-477: STORY-175/177/178/179 codification VEHICLE CHANGED to upstream (see D-477). D-480: E-11 disposition burst #2 — STORY-091/121/143/155 superseded; STORY-147 v2.0 local survivor. WAVE-84 OPENED (STORY-166/176/147v2, 7 pts, all product-local). D-481: STORY-147 DELIVERED (PR #421 f0cb7374). D-482: STORY-166 DELIVERED (PR #426 fa9be701). D-485: STORY-176 DELIVERED (PR #427 595cdba8) — wave-84 3/3 DELIVERY COMPLETE. D-486: WAVE-84 GATE CLOSED + S-7.02 COMPLETE (2026-07-21). D-487: E-16/E-17 ARP stale-draft supersession; backlog EMPTY. D-488: SESSION WRAP (2026-07-21). D-489: SESSION RESUMED + maintenance sweep maint-2026-07-21 STARTED (2026-07-21). D-490: maint-2026-07-21 COMPLETE (2026-07-21). D-491: v0.13.1 RELEASED (2026-07-21). D-492: SESSION WRAP (2026-07-21). D-493: SESSION RESUMED + WAVE-85 SCOPED (2026-07-23); IEC104-TIMED-CMD-GAP-001 research in flight; SEC-001 + ROUTE-W74 pulled into wave-85. D-494: WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE (2026-07-23); STORY-180/181 drafted; adversarial convergence next. D-495: WAVE-85 ADVERSARIAL PASS 1 → REMEDIATED (2026-07-23); STORY-181 re-anchored enip.rs:992-999; HS-INDEX v2.16; adversary pass 2 next.** |
| Version | 0.13.1 (released 2026-07-21; main=47b7d23c; develop=dc7331fb — D-491 v0.13.1 RELEASED (PR #432 dev-tooling patch + PR #433 true-merge back-merge, 2026-07-21)) |
| Main HEAD | `47b7d23c137483de37aa7705617749f5f9d37b07` |
| Develop HEAD | `dc7331fbe3a41fc2b74084dafd8553c3009d7c2e` — D-491 v0.13.1 RELEASED (PR #433 true-merge back-merge, 2026-07-21) |
| Spec versions | BC-INDEX v2.35 / VP-INDEX v2.46 / ARCH-INDEX v2.20 / PRD v1.59 |
| Stories | 116 delivered / 134 total (STORY-INDEX v3.89, dep-graph v3.9, 783 pts) |
| **Last Updated** | 2026-07-23 — D-503 WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP. 9-pass fresh-context adversarial convergence COMPLETE: streak P7/P8/P9 = 3/3 clean, zero open findings. BC-5.39.001 SATISFIED. F-W85S-P9-001 LOW closed (BC-2.19.019 v1.2). Pipeline PAUSED before consistency-validator audit + human story-approval gate. trajectory: `1C+2H+4M+2L(P1)→3M/1L(P2)→1M(P3)→1H(P4)→NITPICK/1L(P5)→1M/2L(P6)→NITPICK/2L(P7 CLEAN 1/3)→CLEAN/0(P8 CLEAN 2/3)→NITPICK/1L-closed(P9 CLEAN 3/3) → CONVERGED 3/3` trajectory-tail →0→0→0→0 |

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
| **v0.13.1 RELEASED** | **RELEASED 2026-07-21** | PR #432 47b7d23c main + tag v0.13.1 (lightweight) + GH release 4 assets (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu); back-merge #433 TRUE-MERGE dc7331fb; DRIFT-BACKMERGE-SQUASH-001 RESOLVED. |
| **Wave 85 (IEC-104 completion mini-wave)** | **CONVERGED/PAUSED (D-503, 2026-07-23)** | Spec-evolution + story-creation COMPLETE (D-494). P1-P6 REMEDIATED (D-495..500). Pass-5 CLEAN (streak reset by P6 MED). Pass-7 CLEAN (D-501): streak 1/3. Pass-8 CLEAN (D-502): streak 2/3. Pass-9 CLEAN (D-503): 0C/0H/0M/1L NITPICK — F-W85S-P9-001 LOW (BC-2.19.019 v1.2 parity back-refs) CLOSED. CONVERGED 3/3. BC-5.39.001 SATISFIED. Pipeline PAUSED before consistency-validator audit + human story-approval gate. develop=dc7331fb (unchanged). trajectory: `1C+2H+4M+2L(P1)→3M/1L(P2)→1M(P3)→1H(P4)→NITPICK/1L(P5)→1M/2L(P6)→NITPICK/2L(P7 CLEAN 1/3)→CLEAN/0(P8 CLEAN 2/3)→NITPICK/1L-closed(P9 CLEAN 3/3) → CONVERGED 3/3` |

---

## Convergence Status

Per-story F4 convergence details archived to `cycles/feature-iec104/convergence-trajectory.md`.
F5 phase-level trajectory: 5 rounds, code frozen R2, `5H/M→2M→1H→1M→1L(NB)` — CONVERGED (D-468).
Wave-84 gate-level adversarial trajectory (6 passes, code frozen 1e967bad): `1M→M/L-batch→1L→0→0→0` — CONVERGED (D-486). Streak P4/P5/P6.
Wave-85 story adversarial trajectory (CONVERGED): `1C+2H+4M+2L(P1)→3M/1L(P2)→1M(P3)→1H(P4)→NITPICK/1L(P5 CLEAN 1/3)→1M/2L-preexisting(P6)→NITPICK/2L(P7 CLEAN 1/3)→CLEAN/0(P8 CLEAN 2/3)→NITPICK/1L-closed(P9 CLEAN 3/3) → CONVERGED 3/3 (P7/P8/P9)` — BC-5.39.001 SATISFIED. DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. Pipeline PAUSED. trajectory-tail →0→0→0→0.

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | **CLOSED (D-475, 2026-07-18)** — v0.13.0 RELEASED (D-473); F1→F7 CONVERGED (D-470); S-7.02 SATISFIED. D-477: STORY-175/177/178/179 vehicles changed to upstream; STORY-176 v2.0 + STORY-166 local survivors | develop (1e967bad) |
| wave-084 (E-11 mini-wave) | **CLOSED (D-486, 2026-07-21)** — 3/3 DELIVERED + gate CLOSED; S-7.02 COMPLETE; 12 PG-W84 entries (3 FIXED / 9 deferred to DF-VALIDATION-001 batch). develop=1e967bad (PR #430 gate-fix final). trajectory-tail →0→0→0→0 | develop (1e967bad, D-486 gate-close) |
| wave-085 (IEC-104 completion mini-wave) | **CONVERGED/PAUSED (D-503, 2026-07-23)** — 9-pass story-level adversarial COMPLETE; streak P7/P8/P9 3/3; ZERO open HIGH/CRITICAL; BC-5.39.001 SATISFIED; F-W85S-P9-001 LOW closed (BC-2.19.019 v1.2); pipeline PAUSED before consistency-validator audit + human story-approval gate | develop (dc7331fb, unchanged) |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-503 WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). 9-pass fresh-context convergence COMPLETE: streak P7/P8/P9 = 3/3 clean, zero open HIGH/CRITICAL (BC-5.39.001 + DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED). F-W85S-P9-001 LOW closed (BC-2.19.019 v1.1→v1.2, reciprocal parity back-refs to BC-2.19.029/030 added; COMPLETED fix, verified present). wave-85 spec+story package CONVERGED, ZERO open findings: BC-2.19.019 v1.2 / BC-2.19.022 v1.1 / BC-2.19.028 v1.1 / BC-2.19.029 v1.1 / BC-2.19.030 v1.0; STORY-180/181 (draft, ready for human story-approval gate); STORY-170 v2.1; HS-133..136; PRD v1.59; STORY-INDEX v3.89; BC-INDEX v2.35; HS-INDEX v2.17. Human /wrap at converged milestone BEFORE consistency-validator audit + human story-approval gate. Pipeline PAUSED. develop=dc7331fb unchanged (spec/story phase; no product code touched). trajectory-tail →0→0→0→0.** | **PAUSED (D-503)** | wave-85 CONVERGED 3/3. Pipeline PAUSED before consistency-validator audit + human story-approval gate. trajectory-tail →0→0→0→0 |
| **D-502 WAVE-85 ADVERSARIAL PASS 8 → FULLY CLEAN (2026-07-23). Pass-8 adversary (spec+story @ c7ef4b15, fresh context): 0 CRIT / 0 HIGH / 0 MED / 0 LOW — zero findings at any severity; novelty NONE. Independent re-derivation reconciled exactly: TypeID-range enumeration (silent set {1–44,52–57,65–99,102,104,106–127}; TypeID-105 Likely), SEC-001 five-locus framing (enip.rs:992-999 *mut/take-remove-reinsert; 825-829 already-safe), APCI LEN byte-recompute (HS-133 0x15; HS-134 0x17/0x17/0x19/0x18 with C_BO_TA_1 NO-QOS; HS-135 0x0E/0x15/0x17), HS-136 timed/untimed jq filters vs iec104.rs:764/805, count=0 Inv-3 + asdu.count field (no live vsq.count), BC-2.19.028/029/030 reciprocity, index arithmetic (STORY-INDEX 134/783, BC-INDEX 380/381, HS-INDEX 209), AC↔BC traces, EC-cites, canonical-frame coverage, green-doc-tense. Prior-pass fixes (F-P4-001/P6/P7) confirmed fully propagated, no reopening. Clean-pass streak 2/3. Next: adversary pass 9 (fresh context) — final pass for BC-5.39.001 3/3 convergence. trajectory-tail →0→0→0→0** | **COMPLETE (D-502)** | Pass-8 FULLY CLEAN (0 findings). Clean-pass streak 2/3. Adversary pass-9 next (fresh context, final). trajectory-tail →0→0→0→0 |
| **D-501 WAVE-85 ADVERSARIAL PASS 7 → CLEAN (NITPICK_ONLY) + LOW residues swept (2026-07-23). Pass-7 adversary (spec+story @ 2635ac6b, fresh context): 0 CRIT / 0 HIGH / 0 MED / 2 LOW + 1 pre-existing out-of-scope obs — FIRST CLEAN PASS of the restarted streak (clean-pass streak 1/3; wave-85 timed-command package re-certified byte-accurate, anchor-exact, internally coherent). LOW residues fixed with exhaustive PG-W85-002-closing sweep: F-P7-001 (BC-2.19.029 v1.0→v1.1 PC5 backticked non-existent `vsq.count` → "(VSQ object count / `asdu.count`)"; Asdu struct iec104.rs:559-572 has flat count:u8, sq:bool, no vsq subfield); F-P7-002 (BC-2.19.028 v1.0→v1.1 Related-BCs +029/030 reciprocal — reciprocity matrix now fully symmetric all 6 directional pairs); F-P7-003 (REC-007/R-CAND-011 stale v0.12.0 label → "Deferred — not yet scheduled", unrelated-to-SEC-001 currency fix). Exhaustive sweeps ALL clean: (1) backticked `vsq.` field-path grep across all 30 SS-19 BCs + STORY-170/180/181 = EMPTY (only changelog mentions remain); (2) BC-2.19.028/029/030 reciprocity matrix symmetric; (3) live v0.12.0-candidate labels in risk docs = EMPTY. No BC-INDEX bump (body/Related-BC edits, no index-structural field change). Clean-pass streak 1/3. Next: adversary pass 8 (fresh context) — need P8/P9 clean for 3/3 BC-5.39.001 convergence. trajectory-tail →0→0→0→0** | **COMPLETE (D-501)** | Pass-7 CLEAN (NITPICK_ONLY). LOW residues swept. No BC-INDEX bump. Clean-pass streak 1/3. Superseded by D-502. trajectory-tail →0→0→0→0 |
| **D-500 WAVE-85 ADVERSARIAL PASS 6 → REMEDIATED (2026-07-23). Pass-6 adversary (spec+story @ 92c28620, fresh context): 0 CRIT / 0 HIGH / 1 MED / 2 LOW — ALL THREE PRE-EXISTING (predate wave-85; reside in §2.19 PRD block / risk files, NOT in wave-85 change set). Adversary CERTIFIED the wave-85 timed-command package (BC-2.19.029/030, STORY-180/181, HS-133..136, BC-2.19.022 v1.1) "byte-accurate, anchor-exact, internally coherent — genuinely converged on its own scope". Fixed (spec-currency hygiene): F-P6-001 (MED) prd §2.19 TypeID-105 verdict Possible→Likely (BC-2.19.020 v1.1 + iec104.rs:847 Verdict::Likely + STORY-170 already correct; PRD summary was stale drift); F-P6-002 (LOW) stale "v0.12.0 candidate" SEC-001 labels → "target: wave-85 / STORY-181" in risk-register R-010 + risk-assumption-monitoring R-CAND-010; F-P6-003 (LOW) prd §2.19 header re-tensed (base IEC-104 shipped v0.13.0; wave-85 timed-command delta scoped not-yet-delivered). Plus minor: STORY-180 AC-180-008 field-name asdu.vsq.count→asdu.count (verified vs iec104.rs:572 — Asdu has no vsq subfield). PRD v1.58→v1.59. Sibling sweeps clean (all other v0.12.0/Possible hits are historical archives or different findings). STORY-180 hash c0fad6c unchanged. Clean-pass streak RESET to 0/3 (pass-6 had substantive MED). Next: adversary pass 7 (fresh context) — need 3 consecutive clean passes P7/P8/P9. trajectory-tail →0→0→0→0** | **COMPLETE (D-500)** | Pass-6 REMEDIATED (pre-existing only). Adversary CERTIFIED wave-85 package. PRD v1.59. Clean-pass streak RESET to 0/3. Superseded by D-501. trajectory-tail →0→0→0→0 |
| **D-499 WAVE-85 ADVERSARIAL PASS 5 → CLEAN (NITPICK_ONLY) + nit remediated (2026-07-23). Pass-5 adversary (spec+story @ 574325fc, fresh context): 0 CRIT / 0 HIGH / 0 MED / 1 LOW — FIRST CLEAN PASS (clean-pass streak 1/3; DF-CONVERGENCE-BEFORE-MERGE-001 zero-HIGH/CRIT criterion met). 12+ axes independently re-verified clean (TypeID enums, SEC-001 anchor+framing all loci, APCI LEN byte-recompute, jq filters, count=0 Inv-3, BC-2.19.028 orphan-free, index arithmetic, AC↔BC, EC cites, canonical-frame, RED-tense). F-P5-001 (LOW) REC-004 in risk-assumption-monitoring.md:468 recommended inapt/inconsistent get_disjoint technique — harmonized to take-remove-reinsert pattern (superseded by STORY-181). Micro-sweep: 2nd get_disjoint hit in research/deferred-security-perf-validation-2026-07.md:33 correctly left as historical dated snapshot (2026-07-06). Next: adversary pass 6 (fresh context) — need 2 more clean passes for BC-5.39.001 3/3 streak. trajectory-tail →0→0→0→0** | **COMPLETE (D-499)** | Pass-5 CLEAN (NITPICK_ONLY). F-P5-001 LOW REC-004 harmonized. Superseded by D-500. trajectory-tail →0→0→0→0 |


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
| D-486 | WAVE-84 GATE CLOSED + S-7.02 COMPLETE (2026-07-21). Integration gate 6-gate all-pass: Gate 1 PASS (2640 tests/94 suites, develop `1e967bad`, clippy/fmt clean, 5 bin/ Python self-tests pass); Gate 2 SKIP (dtu_required:false, passive analyzer); Gate 3 PASS/CONVERGED (6 passes, streak P4/P5/P6, DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED; gate-fix PRs #428 82105d02 / #429 39b30cb1 / #430 1e967bad); Gate 3b PASS (consistency 4MED/3LOW addressed; code-reviewer 0 MAJOR/3 MINOR/6 NIT; security APPROVE 0C/0H/0M); Gate 4 PASS (STORY-147/166/176 demo evidence on develop); Gate 5 SKIP (CI/tooling/factory-process wave, no product behavior change). S-7.02 cycle-close COMPLETE: 12 PG-W84 entries — PG-W84-007/009/011 FIXED in-cycle; PG-W84-001/002/003/004/005/006/008/010/012 deferred to DF-VALIDATION-001 batch (see cycles/wave-084/lessons.md [codified]/[deferred] entries). gate-summary.md + code-review.md + lessons.md authored. STORY-INDEX v3.84→v3.85 (wave-84 delivery row CLOSED; story-file loci synced: STORY-147/166/176 status ready→delivered). develop=1e967bad. WAVE-84 CLOSED. | 2026-07-21 |
| D-487 | E-16/E-17 ARP STALE-DRAFT SUPERSESSION (2026-07-21). STORY-111..115 (E-16, 47 pts, waves 40-44) + STORY-116/117 (E-17, 8 pts, waves 45-46) status draft→superseded DELIVERED-BY-DRIFT. Twice research-validated (DF-VALIDATION-001; planning/e16-e17-arp-draft-disposition-plan.md), human-approved. Wave-table scheduled 747→692 (55 pts / 7 stories excluded from scheduled); total_points 775 unchanged; epic totals unchanged per D-477/D-480 supersession-convention. Arithmetic: 692 + 83 (exclusion sum) = 775. E-16 and E-17 marked DELIVERED/CLOSED in epic table. STORY-INDEX v3.85→v3.86. Backlog now EMPTY; no wave-85 scheduled. develop=1e967bad (unchanged — factory-only burst). | 2026-07-21 |
| D-488 | SESSION WRAP (2026-07-21). Human-requested pause at clean idle milestone. Session D-484..D-487 (exhaustive): STORY-176 DELIVERED (PR #427); wave-84 gate CLOSED (adversary 3-clean P4/P5/P6, gate-fix PRs #428/#429/#430); E-16/E-17 ARP 7-draft supersession (STORY-111..117, delivered-by-drift, twice-validated, human-approved). Backlog EMPTY; no wave-85 scheduled; no in-flight work. Pipeline PAUSED. | 2026-07-21 |
| D-489 | Session RESUMED + maintenance sweep maint-2026-07-21 STARTED (2026-07-21, human-approved). Worktree health PASS; develop=1e967bad verified; open PRs = Dependabot #422-425 + external #407 (both deferred, verified). Maintenance sweep maint-2026-07-21 STARTED (human-selected from idle work menu). Human scope decisions: (a) dep-soak eligibility measured from upstream RELEASE DATE, not Dependabot PR open date — security-relevant bumps considered regardless of soak; (b) NO carry-forwards pulled in (PERF-RERUN-001, Routes B/C, PG-W84 DF-VALIDATION-001 all remain at their stated targets). Sweeps 1-5,7,8 dispatched; Sweep 6 DTU SKIP (dtu_required:false); Sweep 9 a11y SKIP (no UI). | 2026-07-21 |
| D-490 | maint-2026-07-21 COMPLETE (2026-07-21). 8 sweeps total (S6=DTU SKIP, S9=a11y SKIP). DOC-011 HIGH fixed same run (PR #431 6c47c0efa64fbdd319d91aab66210854d0b5b455 squash-merged to develop, human-executed post-classifier-halt; IEC-104 README + ADR-0001/0002/0012/CLAUDE.md). Dependabot #422-425 batch-merged (orchestrator-executed). Holdouts repaired HS-087/123/125/132 (HS-INDEX v2.14). ARCH-INDEX v2.19→v2.20 (SS-19 BC count 27→28, SPEC-008). STORY-INDEX v3.86→v3.87 (epic TOTAL cell 776→775, SPEC-009). Tech-debt register v1.9→v2.0 (15 new rows, 10 resolutions). Human scope: ROUTE-BC deferred; PERF-RERUN-001 valid env but open per human; doc residuals logged as ROUTE-DOC-DEFER-2026-07-21. develop=6c47c0ef. | 2026-07-21 |
| D-491 | v0.13.1 RELEASED (2026-07-21). Dev-tooling patch (green-doc-tense patterns 26-29, validate-citations path:line:anchor, gitignore mutants guard, IEC-104 doc-drift batch). Release PR #432 47b7d23c137483de37aa7705617749f5f9d37b07 squash-merged to main (human-merged). Tag v0.13.1 (lightweight, orchestrator-pushed; tag_object = commit SHA). GH release 4 assets (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu). Back-merge PR #433 TRUE-MERGE dc7331fbe3a41fc2b74084dafd8553c3009d7c2e to develop (human decision). DRIFT-BACKMERGE-SQUASH-001 RESOLVED: main IS ancestor of develop (git merge-base --is-ancestor PASS), first time since v0.12.1/D-436. cargo_version 0.13.0→0.13.1. | 2026-07-21 |
| D-492 | SESSION WRAP (2026-07-21). Human-requested pause (/wrap) at clean post-release milestone. Session D-489..D-491 (exhaustive): maint-2026-07-21 COMPLETE + v0.13.1 RELEASED + back-merge drift resolved. Backlog candidates recorded in checkpoint. No in-flight work, no story worktrees, no factory lock. Pipeline PAUSED. trajectory-tail →0→0→0→0 | 2026-07-21 |
| D-493 | Session RESUMED + WAVE-85 SCOPED (human-approved, 2026-07-23). Resumed from D-492 pause. Worktree health PASS (factory-artifacts a1676f0d in-sync); ground truth verified: develop=dc7331fb, main=47b7d23c (v0.13.1); only open PR = external #407 (DEFERRED, unchanged). Human selected Option A: wave-85 IEC-104 completion mini-wave. Scope: (1) IEC104-TIMED-CMD-GAP-001 detection story — DF-VALIDATION-001 research validation DISPATCHED (research-agent, in flight; report target .factory/planning/iec104-timed-cmd-gap-validation.md); (2) IEC-104 holdout scenario authoring; (3) SEC-001 ENIP split-borrow refactor — human re-triage: PULLED INTO WAVE-85 (target-passed resolved); (4) ROUTE-W74 deferred NIT — human re-triage: PULLED INTO WAVE-85 (target-passed resolved). Options B (PG-W84 research pass), C (hygiene batches), D (dep-soak, dated 2026-07-27) NOT selected — remain at stated targets. Pipeline ACTIVE. | 2026-07-23 |
| D-494 | WAVE-85 SPEC-EVOLUTION + STORY-CREATION COMPLETE (2026-07-23). Research: IEC104-TIMED-CMD-GAP-001 CONFIRMED HIGH (DF-VALIDATION-001, planning/iec104-timed-cmd-gap-validation.md). PO burst: BC-2.19.029 (timed switching 58-60 → T1692.001) + BC-2.19.030 (timed set-point/bitstring 61-64 → T1692.001+T0836) NEW v1.0; BC-2.19.022 v1.0→v1.1 (silent set narrowed to {52-57, 65-99}); BC-INDEX v2.34→v2.35; HS-133..136 authored (HS-INDEX v2.14→v2.15); prd.md §2.19.E rows + §2.19.H BC-2.19.028 backfill + v1.57/v1.58 changelog entries (hook warnings advisory-only per PG-HASH-HOOK-DIVERGENCE). Story burst: STORY-180 (E-22, 5 pts, detection, BC-2.19.029/030/022) + STORY-181 (E-20, 3 pts, SEC-001 ENIP split-borrow + ROUTE-W74 OBS-1 residual as AC-181-004) drafted; STORY-170 v2.0→v2.1 propagation (annotation-only); STORY-INDEX v3.87→v3.88 (134 stories / 783 pts; wave-85 = STORY-180/181, 8 pts). ROUTE-W74 disposition: primary items absorbed by STORY-166 (wave-84 delivered); residual OBS-1 → AC-181-004; OBS-2 carry-forward. Next: wave-85 story adversarial convergence (3 clean passes) then human story-approval gate. | 2026-07-23 |
| D-495 | WAVE-85 ADVERSARIAL PASS 1 → REMEDIATED (2026-07-23). Pass-1 adversary (spec+story package @ 2202c5b3): 1 CRIT / 2 HIGH / 4 MED / 2 LOW. CRITICAL F-W85S-P1-001 (orchestrator-verified against src/analyzer/enip.rs): STORY-181 mis-anchored SEC-001 to an already-safe carry split-borrow (825-829 uses std::mem::take); real+only unsafe is self/self.flows split-borrow via *mut EnipFlowState at enip.rs:992-999. STORY-181 rewritten to target 992-999 with take-remove-reinsert fix + specific grep exit gate. F-P1-002 (HIGH): tech-debt-register SEC-001 description corrected (+sibling fix in risk-assumption-monitoring.md via DF-SIBLING-SWEEP-001). F-P1-003 (HIGH): HS-136 count=0 contradiction fixed (count-independent per BC-2.19.029/030 Invariant 3, not Inv 2). F-P1-004/006/007/009 (MED/LOW): HS-135 BC-2.19.017 frontmatter, Fixture Creation Obligation sections added to HS-133/134/135, APCI LEN bytes recomputed (HS-133 0x13→0x15; HS-134 A/B 0x12→0x17, C 0x13→0x19, D 0x12→0x18 + C_BO_TA_1 QOS field removed per IEC 60870-5-101 Table 8), BC-2.19.028 dropped from HS-133/134. F-P1-008 (LOW): STORY-170 modified-note softened. F-P1-005 (MED)[process-gap]: DISPUTED/NON-FIX — '## Category: real-world-corpus' heading is a template-mandated structural section (validate-template-compliance hook exit_code=2), not a copy-paste artifact; PO rebuttal accepted pending pass-2 fresh-context confirmation. HS-INDEX v2.15→v2.16. STORY-181 hash 8253122 (unchanged). Next: adversary pass 2 (fresh context). | 2026-07-23 |
| D-496 | WAVE-85 ADVERSARIAL PASS 2 → REMEDIATED (2026-07-23). Pass-2 adversary (spec+story @ 304bb465, fresh context): 0 CRIT / 0 HIGH / 3 MED / 1 LOW / 1 process-gap. NO merge-blocker (zero HIGH/CRIT). Fixes: F-P2-001 (MED) STORY-170:62 silently-logged range corrected {1–57,65–99,...}→{1–44,52–57,65–99,...} (was wrongly folding handled 45–51); 17-site sibling sweep confirmed no other stale ranges. F-P2-002 (MED) HS-136 dropped mis-cited BC-2.19.028 (Inv-3 text mismatch + DoS-cap not exercised by any case); now absent across all HS-133..136 + HS-INDEX. F-P2-003 (MED) HS-136 Case D dead jq regex (_NA/_NB/_NC mnemonics match nothing) fixed to negate timed-mnemonic set (parity with Case A); iec104.rs:764/805 confirm actual summaries use C_SC/C_DC/C_RC and C_SE/C_BO. F-P2-004 (LOW) HS-135 Case C/D frame LEN 0x0B→0x0E. HS-INDEX v2.16→v2.17. STORY-170 hash 7873f11 (unchanged). F-P2-005 (MED)[process-gap]: ADJUDICATED as plugin-level template+hook defect — holdout-scenario-template.md + validate-template-compliance.sh treat '## Category: real-world-corpus' as unconditionally-required (only 6/136 files carry it; HS-122/132 lack it), forcing a contradictory heading on non-corpus files. NOT a per-file fix; does NOT block wave-85 convergence. NEW process-gap PG-W85-001 → DF-VALIDATION-001 + upstream drbothen/vsdd-factory. Next: adversary pass 3 (fresh context); need 3 clean/nitpick-only consecutive passes for BC-5.39.001 convergence. | 2026-07-23 |
| D-497 | WAVE-85 ADVERSARIAL PASS 3 → REMEDIATED (2026-07-23). Pass-3 adversary (spec+story @ dcc8cc06, fresh context): 0 CRIT / 0 HIGH / 1 MED. NO merge-blocker. F-P3-001 (MED): STORY-170 AC-170-005 Note (lines 105-106) dropped the [1,44] monitoring-direction segment — partial-fix residual of pass-2 (line 62 BC-table fixed, this sibling Note locus missed). Corrected to {1–44, 52–57, 65–99, 102, 104, 106–127}; exhaustive in-file sweep confirmed all 11 STORY-170 silent-set loci now consistent; cross-file clean. STORY-170 hash 7873f11 (unchanged). 12 other review axes independently re-verified clean by pass-3 (SEC-001 anchor, APCI LEN byte-recompute, jq filters, count=0, BC-2.19.028 orphan-free, TypeID/technique maps, index arithmetic, AC↔BC traces, canonical-frame coverage). Clean-pass counter still 0/3. Next: adversary pass 4 (fresh context). | 2026-07-23 |
| D-498 | WAVE-85 ADVERSARIAL PASS 4 → REMEDIATED (2026-07-23). Pass-4 adversary (spec+story @ 097c3dd1, fresh context): 0 CRIT / 1 HIGH / 0 MED. F-P4-001 (HIGH): STORY-181 body correctly targeted the *mut EnipFlowState PDU-dispatch-loop fix but 3 loci retained the REJECTED "Direction-Keyed Carry Select" framing — STORY-INDEX:334 title (canonical registry), STORY-181:262 FSR normative cell (pointed implementer at the 825-829 carry region the story forbids touching), STORY-181:119 AC-181-003 trace. Second-order propagation tail of pass-1 CRITICAL F-P1-001. All 3 fixed; story-writer 27-hit exhaustive sweep caught a 4th locus (risk-register.md R-010 same stale framing) — fixed; 23 remaining hits verified correct (already-safe notes / unrelated TLS/DNP3/Modbus carry logic). STORY-181 full-section consistency confirmed. Hashes: STORY-181 8253122 (unchanged), risk-register 0447a72→865986f. Pass-4 also independently re-verified 12 axes clean (TypeID enums, SEC-001 anchor, APCI LEN, jq filters, count=0, BC-2.19.028 orphan-free, arithmetic, AC↔BC, EC cites, canonical-frame, RED-tense). Clean-pass counter still 0/3. NOTE: 3 consecutive passes (P2/P3/P4) found partial-fix propagation residuals (STORY-170:62 → STORY-170:105 → STORY-181 title/FSR/trace) — recurring remediation-sweep locus-coverage gap; flag PG-W85-002 for cycle-close codification (remediation sweeps must cover index titles, FSR cells, AC traces, and cross-spec risk-register loci, not just the cited line). Next: adversary pass 5 (fresh context). | 2026-07-23 |
| D-499 | WAVE-85 ADVERSARIAL PASS 5 → CLEAN (NITPICK_ONLY) + nit remediated (2026-07-23). Pass-5 adversary (spec+story @ 574325fc, fresh context): 0 CRIT / 0 HIGH / 0 MED / 1 LOW — FIRST CLEAN PASS (clean-pass streak 1/3; DF-CONVERGENCE-BEFORE-MERGE-001 zero-HIGH/CRIT criterion met). 12+ axes independently re-verified clean (TypeID enums, SEC-001 anchor+framing all loci, APCI LEN byte-recompute, jq filters, count=0 Inv-3, BC-2.19.028 orphan-free, index arithmetic, AC↔BC, EC cites, canonical-frame, RED-tense). F-P5-001 (LOW) REC-004 in risk-assumption-monitoring.md:468 recommended inapt/inconsistent get_disjoint technique — harmonized to take-remove-reinsert pattern (superseded by STORY-181). Micro-sweep: 2nd get_disjoint hit in research/deferred-security-perf-validation-2026-07.md:33 correctly left as historical dated snapshot (2026-07-06). Next: adversary pass 6 (fresh context) — need 2 more clean passes for BC-5.39.001 3/3 streak. | 2026-07-23 |
| D-500 | WAVE-85 ADVERSARIAL PASS 6 → REMEDIATED (2026-07-23). Pass-6 adversary (spec+story @ 92c28620, fresh context): 0 CRIT / 0 HIGH / 1 MED / 2 LOW — ALL THREE PRE-EXISTING (predate wave-85; reside in §2.19 PRD block / risk files, NOT in wave-85 change set). Adversary CERTIFIED the wave-85 timed-command package (BC-2.19.029/030, STORY-180/181, HS-133..136, BC-2.19.022 v1.1) "byte-accurate, anchor-exact, internally coherent — genuinely converged on its own scope". Fixed (spec-currency hygiene, since adversary re-flags them each pass as anchor sources): F-P6-001 (MED) prd §2.19 TypeID-105 verdict Possible→Likely (BC-2.19.020 v1.1 + iec104.rs:847 Verdict::Likely + STORY-170 already correct; PRD summary was stale drift); F-P6-002 (LOW) stale "v0.12.0 candidate" SEC-001 labels → "target: wave-85 / STORY-181" in risk-register R-010 + risk-assumption-monitoring R-CAND-010; F-P6-003 (LOW) prd §2.19 header re-tensed (base IEC-104 shipped v0.13.0; wave-85 timed-command delta scoped not-yet-delivered). Plus minor: STORY-180 AC-180-008 field-name asdu.vsq.count→asdu.count (verified vs iec104.rs:572 — Asdu has no vsq subfield). PRD v1.58→v1.59. Sibling sweeps clean (all other v0.12.0/Possible hits are historical archives or different findings). STORY-180 hash c0fad6c unchanged. Clean-pass streak RESET to 0/3 (pass-6 had substantive MED). Next: adversary pass 7 (fresh context) — need 3 consecutive clean passes P7/P8/P9. | 2026-07-23 |
| D-501 | WAVE-85 ADVERSARIAL PASS 7 → CLEAN (NITPICK_ONLY) + LOW residues swept (2026-07-23). Pass-7 adversary (spec+story @ 2635ac6b, fresh context): 0 CRIT / 0 HIGH / 0 MED / 2 LOW + 1 pre-existing out-of-scope obs — FIRST CLEAN PASS of the restarted streak (clean-pass streak 1/3; wave-85 timed-command package re-certified byte-accurate, anchor-exact, internally coherent). LOW residues fixed with exhaustive PG-W85-002-closing sweep: F-P7-001 (BC-2.19.029 v1.0→v1.1 PC5 backticked non-existent `vsq.count` → "(VSQ object count / `asdu.count`)"; Asdu struct iec104.rs:559-572 has flat count:u8, sq:bool, no vsq subfield); F-P7-002 (BC-2.19.028 v1.0→v1.1 Related-BCs +029/030 reciprocal — reciprocity matrix now fully symmetric all 6 directional pairs); F-P7-003 (REC-007/R-CAND-011 stale v0.12.0 label → "Deferred — not yet scheduled", unrelated-to-SEC-001 currency fix). Exhaustive sweeps ALL clean: (1) backticked `vsq.` field-path grep across all 30 SS-19 BCs + STORY-170/180/181 = EMPTY (only changelog mentions remain); (2) BC-2.19.028/029/030 reciprocity matrix symmetric; (3) live v0.12.0-candidate labels in risk docs = EMPTY. No BC-INDEX bump (body/Related-BC edits, no index-structural field change). Clean-pass streak 1/3. Next: adversary pass 8 (fresh context) — need P8/P9 clean for 3/3 BC-5.39.001 convergence. | 2026-07-23 |
| D-502 | WAVE-85 ADVERSARIAL PASS 8 → FULLY CLEAN (2026-07-23). Pass-8 adversary (spec+story @ c7ef4b15, fresh context): 0 CRIT / 0 HIGH / 0 MED / 0 LOW — zero findings at any severity; novelty NONE. Independent re-derivation reconciled exactly: TypeID-range enumeration (silent set {1–44,52–57,65–99,102,104,106–127}; TypeID-105 Likely), SEC-001 five-locus framing (enip.rs:992-999 *mut/take-remove-reinsert; 825-829 already-safe), APCI LEN byte-recompute (HS-133 0x15; HS-134 0x17/0x17/0x19/0x18 with C_BO_TA_1 NO-QOS; HS-135 0x0E/0x15/0x17), HS-136 timed/untimed jq filters vs iec104.rs:764/805, count=0 Inv-3 + asdu.count field (no live vsq.count), BC-2.19.028/029/030 reciprocity, index arithmetic (STORY-INDEX 134/783, BC-INDEX 380/381, HS-INDEX 209), AC↔BC traces, EC-cites, canonical-frame coverage, green-doc-tense. Prior-pass fixes (F-P4-001/P6/P7) confirmed fully propagated, no reopening. Clean-pass streak 2/3. Next: adversary pass 9 (fresh context) — final pass for BC-5.39.001 3/3 convergence. | 2026-07-23 |
| D-503 | WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). 9-pass fresh-context convergence COMPLETE: streak P7/P8/P9 = 3/3 clean, zero open HIGH/CRITICAL (BC-5.39.001 + DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED). F-W85S-P9-001 LOW closed (BC-2.19.019 v1.1→v1.2, reciprocal parity back-refs to BC-2.19.029/030 added; COMPLETED fix, verified present). wave-85 spec+story package CONVERGED, ZERO open findings: BC-2.19.019 v1.2 / BC-2.19.022 v1.1 / BC-2.19.028 v1.1 / BC-2.19.029 v1.1 / BC-2.19.030 v1.0; STORY-180/181 (draft, ready for human story-approval gate); STORY-170 v2.1; HS-133..136; PRD v1.59; STORY-INDEX v3.89; BC-INDEX v2.35; HS-INDEX v2.17. Human /wrap at converged milestone BEFORE the (not-yet-run) fresh-context consistency-validator audit + human story-approval gate. Pipeline PAUSED. develop=dc7331fb unchanged (spec/story phase; no product code touched). trajectory-tail →0→0→0→0. | 2026-07-23 |

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
| DRIFT-BACKMERGE-SQUASH-001 | **RESOLVED (D-491, 2026-07-21).** v0.13.1 back-merge PR #433 TRUE-MERGE (dc7331fb to develop); main (47b7d23c) IS ancestor of develop (dc7331fb) — git merge-base --is-ancestor PASS, first time since v0.12.1/D-436. Prior history: v0.12.1 back-merge #400 squash-merged; v0.13.0 back-merge #418 squash-merged. | v0.12.1 release (D-436, 2026-07-13) → RESOLVED D-491 (2026-07-21) | RESOLVED — true-merge PR #433 dc7331fb. Archive at next compact. |
| DRIFT-VP039-BC207038-TLS-TODO-001 | VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039 (TLS reassembly). Out of feature-iec104 scope. | feature-iec104 F2 review (D-438, 2026-07-14) | SS-07 TLS owner — next TLS maintenance sweep |
| STORY-INDEX-IN-INPUTS-CHURN | Stories listing STORY-INDEX.md as an input (STORY-164/165) re-stale on every index version bump; 4+ re-baselines in 3 days. Separately, STORY-175..179 list `.factory/STATE.md` as an input, re-staling on every factory commit — 3+ re-baselines. Structural fix (remove index/STATE.md from inputs lists) awaits human decision. Related upstream discussion #672/#314. | D-477 (2026-07-19) → D-483 (2026-07-20, STATE.md cluster) | Human decision: remove STORY-INDEX.md/STATE.md from affected story inputs lists — still pending |
| PG-W84-UPSTREAM-BATCH | PG-W84-001/002/003/004/005/006/008 (7 upstream drbothen/vsdd-factory engine gaps from wave-84 S-7.02): stale-inline-version-marker recurrence, sub-agent message-routing breakage, burst-log template understatement, STATE.md hook-cascade friction, validate-pr-review-posted false-positive on self-authored PRs, pr-manager step-9 pressure before merge, PR-description commit-count drift. DF-VALIDATION-001 research-agent validation required before filing. See cycles/wave-084/lessons.md [deferred] entries. | wave-084 S-7.02 (D-486, 2026-07-21) | DF-VALIDATION-001 research pass (next available) |
| PG-W84-LOCAL-BATCH | PG-W84-010/012 (2 product-local gaps from wave-84 S-7.02): gate scan Rust-only blind spot for bin/*.py prose; bin-selftest CI job not in develop required-status-checks. DF-VALIDATION-001 research-agent validation required before filing as GitHub issues. | wave-084 S-7.02 (D-486, 2026-07-21) | DF-VALIDATION-001 research pass (next available) / next branch-protection review |
| PG-W85-001 | Plugin-level template+hook defect: holdout-scenario-template.md + validate-template-compliance.sh treat '## Category: real-world-corpus' as unconditionally-required. Only 6/136 HS files carry it; HS-122/132 lack it; forces contradictory heading on non-corpus files. Adjudicated wave-85 pass-2 (D-496) — NOT a per-file fix; does NOT block wave-85 convergence. | wave-085 pass-2 D-496, 2026-07-23 | DF-VALIDATION-001 + upstream drbothen/vsdd-factory |
| PG-W85-002 | Recurring remediation-sweep locus-coverage gap: passes P2/P3/P4 each found stale framing not covered by the preceding sweep (STORY-170:62 → STORY-170:105 → STORY-181 title/FSR/trace + risk-register.md R-010). Remediation sweeps must cover index titles, FSR cells, AC traces, and cross-spec risk-register loci, not just the cited line. Flag for cycle-close codification. | wave-085 P2-P4 (D-496/497/498, 2026-07-23) | Cycle-close codification / DF-VALIDATION-001 |

---

## Active Carry-Forwards

| ID | Summary | Target |
|---|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11 (human decision). | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review NIT deferred from wave-74 gate; OBS-1 residual → AC-181-004 in STORY-181 (wave-85, drafted D-494); OBS-2 residual remains open (see ROUTE-W74-OBS-2 below). Primary items absorbed by STORY-166 (wave-84, delivered). | STORY-181 (OBS-1 AC-181-004, wave-85); OBS-2 per ROUTE-W74-OBS-2 |
| ROUTE-W74-OBS-2 | ROUTE-W74 observability residual OBS-2 — not absorbed by STORY-166 or STORY-181 (STORY-181 covers OBS-1 only). Remains open pending human scope decision on whether OBS-2 warrants a standalone story or defers to next maintenance run. | Human scope decision; next wave or maintenance run |
| PERF-RERUN-001 | AC-149-003 re-run PASS at maint-2026-07-21 (env VALID 0.26/core, 23.659µs — first valid env reading; prior load avg 52.57). Remains OPEN per human scope decision D-490. | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred from maint-2026-07-11. Pulled into wave-85 (D-493); absorbed into STORY-181 (wave-85, drafted D-494). STORY-181 re-anchored to enip.rs:992-999 (D-495). | STORY-181 (wave-85, drafted D-494; re-anchored D-495) |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant) TypeIDs 58–64 (timed control variants) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Evasion gap. CONFIRMED HIGH (DF-VALIDATION-001, planning/iec104-timed-cmd-gap-validation.md). Detection story drafted as STORY-180. | STORY-180 (wave-85, drafted D-494) |
| PR-407-FORK-RELEASE-OPS | External ArcavenAE PR #407 security-triaged SAFE-WITH-CHANGES (D-472; triage at .factory/planning/pr-407-security-triage.md); DEFERRED — governance decision pending. Resume without re-running security review. | Governance decision when authorized |
| SCORECARD-ENABLEMENT-RUNBOOK | Before setting SCORECARD_ENABLED=true: document CWE-200 publish_results:true risk; harden-runner PR #423 merged under D-490 (soaked); DEP-SOAK-FOLLOWUP-2026-07-27 now covers crate soak only (github-actions merged under D-490). PR #414 ADOPTED (D-476). | Whenever scorecard is enabled |
| DEP-SOAK-FOLLOWUP-2026-07-27 | 17 not-yet-soaked crates eligible 2026-07-21..27 (serde/clap/regex/syn/anyhow/etc.) + 4 soaked-but-blocked (js-sys/wasm-bindgen/web-sys via futures-* 0.3.33; shlex via cc 1.3.0). Dependabot github-actions PRs #422-425 merged under D-490 (cargo-deny-action 2.1.1, harden-runner 2.20.0, action-gh-release 3.0.2, codeql upload-sarif 4.37.0). Crate soak only remains. Run next soak sweep on/after 2026-07-27. | Next maintenance run on/after 2026-07-27 |
| ROUTE-DOC-DEFER-2026-07-21 | PR #431 (IEC-104 doc-drift) review residuals deferred from D-490: ADR-0001 Consequences minimal IEC-104 context (LOW), ADR-0002 Deviations heading presence (NIT), ADR-0012 stale 'supported: 7 protocols'/port 2404 vs 102 (LOW). All non-blocking. | Next doc sweep |

---

## Session Resume Checkpoint

**D-503 WAVE-85 STORY-LEVEL ADVERSARIAL CONVERGED + SESSION WRAP (2026-07-23). 9-pass fresh-context adversarial convergence COMPLETE: streak P7/P8/P9 = 3/3 clean (P9: 0C/0H/0M/1L NITPICK — F-W85S-P9-001 LOW BC-2.19.019 v1.2 parity back-refs CLOSED). BC-5.39.001 SATISFIED. DF-CONVERGENCE-BEFORE-MERGE-001 SATISFIED. Converged package: BC-2.19.019 v1.2 / BC-2.19.022 v1.1 / BC-2.19.028 v1.1 / BC-2.19.029 v1.1 / BC-2.19.030 v1.0; STORY-180/181 (draft, ready for human story-approval gate); STORY-170 v2.1; HS-133..136; PRD v1.59; STORY-INDEX v3.89; BC-INDEX v2.35; HS-INDEX v2.17. Pipeline PAUSED before consistency-validator audit + human story-approval gate. trajectory-tail →0→0→0→0.**

Prior checkpoints archived to `cycles/feature-iec104/session-checkpoints.md` and `cycles/wave-084/session-checkpoints.md` and `cycles/wave-085/session-checkpoints.md`.

- **Date:** 2026-07-23. Position: wave-85 IEC-104 completion mini-wave; story-level adversarial CONVERGED (9 passes, streak P7/P8/P9 3/3, ZERO open findings); PAUSED before consistency-validator audit + human story-approval gate.
- **Convergence counter:** BC-5.39.001 3/3 SATISFIED — do NOT re-run story-level adversarial on resume.
- **In-flight:** NONE. All bursts committed. Tree clean expected post-wrap.
- **PENDING NEXT STEPS (in order) on resume:** (a) run fresh-context consistency-validator full-corpus audit (MANDATED before human gate — NOT yet run this session); (b) present HUMAN story-approval gate for STORY-180/181 with structured questions (scope completeness of TypeID 58-64 coverage, BC-2.19.029/030 anchor correctness, story coverage gaps, MITRE technique-ID consistency); (c) on approval → Phase 3 TDD per-story-delivery (STORY-180 detection arms 58-60→T1692.001 / 61-64→T1692.001+T0836 with neighbor-silence regression; STORY-181 SEC-001 *mut EnipFlowState take-remove-reinsert refactor at enip.rs:992-999 + ROUTE-W74 OBS-1); (d) cycle-close: codify PG-W85-001 (plugin template defect) + PG-W85-002 (remediation-sweep locus-coverage).
- **Ground truth:** develop=dc7331fb, main=47b7d23c (v0.13.1); factory-artifacts=this wrap commit. No product code changed this session. Only open product PR: external #407 (DEFERRED).
- **Pending human decisions:** PR #407 governance; STORY-INDEX-IN-INPUTS-CHURN structural fix; DEP-SOAK-FOLLOWUP-2026-07-27 (dated, on/after 2026-07-27); PERF-RERUN-001; ROUTE-BC-DEFER; ROUTE-DOC-DEFER-2026-07-21.
- **Spec versions:** BC-INDEX v2.35 / VP-INDEX v2.46 / ARCH-INDEX v2.20 / PRD v1.59 / STORY-INDEX v3.89 / HS-INDEX v2.17 / dep-graph v3.9.
- **Resume command:** `/vsdd-factory:next-step`

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
| **feature-iec104 cycle-close lessons** | `cycles/feature-iec104/lessons.md` (9 PGs [codified] → STORY-175..179 per D-475; D-477 vehicle change; D-478 dep-soak lessons) |
| **feature-iec104 burst log** | `cycles/feature-iec104/burst-log.md` (archives D-475/D-476 CPS rows rolled out under last-5 rule) |
| **Phase Progress granular rows (F4 waves/adversary/fixes)** | `cycles/feature-iec104/phase-progress-archive.md` (D-451 burst, wave-79..83, STORY-172/173/174 per-story adversary, FIX-P4-001/F5-001..004) |
| **Convergence Trajectory (F4 per-story + F5 phase)** | `cycles/feature-iec104/convergence-trajectory.md` |
| feature-iec104 F2 convergence report | `cycles/feature-iec104/adversarial/f2-convergence-report.md` (12 passes, CONVERGED P10/P11/P12, D-438) |
| Session checkpoints (feature-iec104, all prior) | `cycles/feature-iec104/session-checkpoints.md` (waves 76–83 era + D-471 through D-479 session wrap checkpoints, all archived) |
| Wave 75 gate files | `cycles/wave-75/wave-gate/` (gate-summary.md D-435, code-review.md, findings.md) |
| Wave 74 gate files | `cycles/wave-74/wave-gate/` (gate-summary.md D-432) |
| Wave 73 gate files + lessons | `cycles/wave-73/wave-gate/` + `cycles/wave-73/lessons.md` |
| Wave 72 gate files + lessons | `cycles/wave-72/wave-gate/` (gate-summary.md, code-review.md, demo-evidence/) + `cycles/wave-72/lessons.md` |
| Wave 71 gate files + lessons + checkpoints | `cycles/wave-71/wave-gate/` + `cycles/wave-71/session-checkpoints.md` + `cycles/wave-71/lessons.md` |
| Wave 70 gate files + checkpoints | `cycles/wave-70-story-149/wave-gate/` + `cycles/wave-70-story-149/session-checkpoints.md` |
| Wave 82 gate files | `cycles/wave-082/wave-gate/` (gate-summary.md D-458, code-review.md) |
| Wave 83 gate files | `cycles/wave-083/wave-gate/` (gate-summary.md D-463, code-review.md) |
| **Wave 84 gate artifacts** | `cycles/wave-084/wave-gate/` (gate-summary.md D-486, code-review.md) |
| **Wave-84 lessons (S-7.02 COMPLETE)** | `cycles/wave-084/lessons.md` (12 entries: 3 [codified] PG-W84-007/009/011; 9 [deferred] to DF-VALIDATION-001 batch) |
| **Wave-84 process-gap ledger** | `cycles/wave-084/process-gap-ledger.md` (12 PG-W84 entries; S-7.02 COMPLETE declared D-486) |
| STORY-147 per-story convergence report | `cycles/wave-084/STORY-147/convergence-report.md` + `adversary-convergence-state.json` (8 passes, CONVERGED P6/P7/P8) |
| STORY-166 per-story convergence report | `cycles/wave-084/STORY-166/convergence-report.md` + `adversary-convergence-state.json` (10 passes, CONVERGED P8/P9/P10) |
| STORY-176 per-story convergence report | `cycles/wave-084/STORY-176/convergence-report.md` + `adversary-convergence-state.json` (8 passes, CONVERGED P6/P7/P8; BC-5.39.001 SATISFIED) |
| **Wave-084 burst log** | `cycles/wave-084/burst-log.md` (archives rolled-out CPS rows D-477 through D-492; D-488 CPS row archived by D-493 burst) |
| Wave-084 session checkpoints (all archived) | `cycles/wave-084/session-checkpoints.md` (D-481 through D-492 superseded checkpoints) |
| feature-iec104 F5 adversarial reviews | `.factory/phase-f5-adversarial/round-1-review.md` through `round-5-review.md`; `convergence-summary.md` (D-468) |
| feature-iec104 F6 gate verdict + hardening artifacts | `.factory/phase-f6-hardening/f6-gate-verdict-iec104.md` (D-469 PASS) |
| feature-iec104 F7 convergence artifacts | `.factory/phase-f7-convergence/delta-convergence-report.md` (D-470 CONVERGED) |
