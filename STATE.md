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
phase: "feature-iec104/F4"
status: in_progress
current_step: "D-458 STORY-173 DELIVERED (PR #408 084ff93 squash-merged to develop, 2026-07-16, human-authorized). 17 adversarial passes; initial 3-clean P12/P13/P14 at 7b2a73e (D-457), then pre-merge LOW-fix burst (LOW#1 flows_analyzed + LOW#2 packets_analyzed + SEC-001 is_valid doc; 0bfc977/5325cf2/3ec6ac1), then fresh 3-clean A/B/C. IEC104-FINDINGS-CAP-001 RESOLVED. CI 13/13. 2604/0. 7/8 IEC-104. stories_delivered=112. Wave-82 SATISFIED. STORY-INDEX v3.72. BC-INDEX v2.33. develop=084ff93. NEXT: STORY-174 wave-83. trajectory-tail →0→0→0→0"
current_cycle: "feature-iec104"
pipeline: IN PROGRESS
timestamp: 2026-07-17T00:02:00Z
# D-458 STORY-173 DELIVERED (PR #408 084ff93, 2026-07-16). 17 adversarial passes; 3-clean A/B/C post-LOW-fixes. IEC104-FINDINGS-CAP-001 RESOLVED. Wave-82 gate SATISFIED. stories_delivered=112. BC-INDEX v2.33. STORY-INDEX v3.72. NEXT: STORY-174 (wave-83).

# Release chain (latest)
released_version: v0.12.1
released_at: "2026-07-13"
release_tag: v0.12.1
release_tag_object: d687a77d911503e67a8d171c00536bd710762bba
release_commit: fedcea4ab17d9b3257c9903636aec0c0fd08f147
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.12.1
prior_released_version: v0.12.0
prior_released_at: "2026-07-10"
# Ground-truth HEADs (updated 2026-07-16 — D-458 STORY-173 DELIVERED; develop=084ff93 (STORY-167+168+169+170+171+172+173, 7 unreleased); DRIFT-BACKMERGE-SQUASH-001 still applies)
main_head: fedcea4ab17d9b3257c9903636aec0c0fd08f147
develop_head: 084ff93
# Cargo.toml version: main=0.12.1; develop=0.12.1 (7 unreleased commits 084ff93 STORY-167+168+169+170+171+172+173; DRIFT-BACKMERGE-SQUASH-001: main fedcea4 not an ancestor of develop 084ff93, histories diverge; trees differ by IEC-104 feature code)
cargo_version_main: "0.12.1"
cargo_version_develop: "0.12.1"
# Open worktrees: main checkout [develop] + .factory [factory-artifacts].
# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
# Story tracking
stories_delivered: 112
story_index_version: "v3.72"
total_stories: 127
story_index_note: "127 stories / 83 waves / 765 pts. v3.72 (2026-07-16): STORY-173 DELIVERED (D-458, PR #408 084ff93, wave-82 SATISFIED; IEC104-FINDINGS-CAP-001 RESOLVED; BC-INDEX v2.33). See cycles/feature-iec104/ for full F2/F3 history."
# Spec versions (current)
bc_index_version: "v2.33"
vp_index_version: "v2.46"
arch_index_version: "v2.19"
prd_version: "v1.56"
epics_version: v2.1
# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-07-11
maintenance_started_at: "2026-07-11"
maintenance_completed_at: "2026-07-11"
maintenance_prior_run: maint-2026-07-09
---

<!--
  STATE.md SIZE BUDGET (per D-421(c)):
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 280 = 220 (dual-margin form). 280 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-458 COMPLETE (2026-07-16): STORY-173 DELIVERED (PR #408 084ff93, wave-82 SATISFIED). 17 adversarial passes; initial 3-clean P12/P13/P14 (D-457), pre-merge LOW-fix burst (LOW#1/LOW#2/SEC-001; 0bfc977/5325cf2/3ec6ac1), fresh 3-clean A/B/C. IEC104-FINDINGS-CAP-001 RESOLVED (CWE-400/770; MAX_IEC104_FINDINGS=10_000). 2604/0; develop=084ff93; stories_delivered=112. BC-INDEX v2.33. STORY-INDEX v3.72. NEXT: STORY-174 (wave-83, formal hardening + VP-045 non-vacuity + PG-REDGREEN grep-guard). trajectory-tail →0→0→0→0**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); F4 IN PROGRESS; **wave-82 DELIVERED (D-458); STORY-173 DELIVERED; 7 of 8 IEC-104 story-items delivered (STORY-167..173); IEC104-FINDINGS-CAP-001 RESOLVED; STORY-174 wave-83 next** |
| Version | 0.12.1 (released 2026-07-13; main=fedcea4; develop=084ff93 — 7 unreleased commits; DRIFT-BACKMERGE-SQUASH-001) |
| Main HEAD | `fedcea4ab17d9b3257c9903636aec0c0fd08f147` |
| Develop HEAD | `084ff93` — PR #408 STORY-173 squash 2026-07-16; DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 |
| Stories | 112 delivered / 127 total (STORY-INDEX v3.72, dep-graph v3.9, 765 pts) |
| **Last Updated** | 2026-07-16 — D-458 STORY-173 DELIVERED (PR #408 084ff93, wave-82 SATISFIED). 17 passes; 3-clean A/B/C post-LOWfix; IEC104-FINDINGS-CAP-001 RESOLVED; 2604/0. STORY-INDEX v3.72. trajectory-tail →0→0→0→0 |

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
| feature-iec104 — F4 (delta-implementation) | **IN PROGRESS** | Waves 76–82 DELIVERED (D-441/443/445/447/448/455/458): STORY-167..173 PRs #401-408. **D-458 STORY-173 DELIVERED; IEC104-FINDINGS-CAP-001 RESOLVED; 17 passes 3-clean A/B/C; 2604/0.** 7/8 stories done. trajectory-tail →0→0→0→0 |
| feature-iec104 — D-451 spec-remediation burst | **COMPLETE** | SR-172-01 BLOCKING (FlowId→FlowKey); SR-172-02 MEDIUM (carry-overflow discard-all-new); SR-172-03 MEDIUM (malformed-LEN EMIT-WITH-DEDUP, research-validated). 3rd F3-DECOMPOSITION-BC-FIDELITY — CODIFY-NOW. |
| feature-iec104 — F4 per-story adversary (STORY-172) | **CONVERGED (D-454)** | 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6 (BC-5.39.001 SATISFIED). F-172-001/002 HIGH remediated D-452; F-172-201 LOW remediated D-453; F-172-301 NIT remediated fec9bfa. Deferred: F-172-003 LOW STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. |
| feature-iec104 — F4 per-story adversary (STORY-173) | **CONVERGED (D-457/D-458)** | 17 passes total; initial streak P12/P13/P14 (D-457); pre-merge LOW fix burst (3 LOWs fixed; see next row); fresh 3-clean A/B/C (D-458). IEC104-FINDINGS-CAP-001 RESOLVED. BC-2.19.006 v1.2; BC-INDEX v2.33. |
| feature-iec104 — STORY-173 pre-merge LOW fix burst | **COMPLETE** | 3 LOWs FIXED pre-merge (human approved all 3): LOW#1 flows_analyzed real cumulative counter (mirrors ENIP; 0bfc977); LOW#2 packets_analyzed valid-APDU frame counter (mirrors DNP3; 5325cf2); SEC-001/A-173-A-01 is_valid_iec104_frame doc overstated gate role + BC-2.19.006 v1.1→v1.2; BC-INDEX v2.32→v2.33 (3ec6ac1). 2602→2604 tests. Triggered fresh A/B/C re-convergence. |
| feature-iec104 — wave-79 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P3/P4/P5) on STORY-170 diff == wave-level adversarial; CI 13/13 develop 0bd93f8; D-447 |
| feature-iec104 — wave-80 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P2/P3/P4) on STORY-171 diff == wave-level adversarial; CI 13/13 develop 1a64380; D-448 |
| feature-iec104 — wave-81 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P4/P5/P6) on STORY-172 diff == wave-level adversarial; CI 13/13 develop d64e5fe; D-455 |
| feature-iec104 — wave-82 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean A/B/C (17 total passes) on STORY-173 diff == wave-level adversarial; CI 13/13 develop 084ff93; D-458 |

---

## Convergence Status

| Cycle/Story | Passes | Trajectory | Status |
|------------|--------|-----------|--------|
| feature-iec104 F4 per-story (STORY-171) | 4 | →2→0→0→0 | CONVERGED 3-clean (BC-5.39.001) |
| feature-iec104 F4 per-story (STORY-172) | 6 | →(2H+1L+1N)→1L→1NIT→0→0→0 | CONVERGED 3-clean (BC-5.39.001) streak P4/P5/P6 |
| feature-iec104 F4 per-story (STORY-173) | 17 (14+3) | →(1H+3doc)→(1M+1N)→NITs→CLEAN(P6)→1N→4N→CLEAN(P9/P10)→1N→CLEAN(P12/P13/P14)→3LOWfix→CLEAN(A/B/C) | CONVERGED 3-clean (BC-5.39.001) re-converged A/B/C post-LOW-fixes (D-458) |

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | IN PROGRESS — F4 STORY-173 wave-82 DELIVERED (D-458); 7/8 IEC-104 stories done; STORY-174 wave-83 next | develop |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-458 STORY-173 DELIVERED (2026-07-16). PR #408 084ff93 squash-merged to develop, human-authorized. 17 adversarial passes; initial 3-clean P12/P13/P14 (D-457); pre-merge LOW-fix burst (LOW#1 flows_analyzed + LOW#2 packets_analyzed + SEC-001 is_valid doc/BC-2.19.006 v1.2; 0bfc977/5325cf2/3ec6ac1); fresh 3-clean A/B/C on 3ec6ac1. IEC104-FINDINGS-CAP-001 RESOLVED. CI 13/13. 2604/0. 7/8 IEC-104. stories_delivered 111→112. Wave-82 gate SATISFIED. 5 new process-gaps. BC-INDEX v2.33. STORY-INDEX v3.72. develop=084ff93.** | **DELIVERED (D-458)** | Wave-82 gate SATISFIED. trajectory-tail →0→0→0→0 |
| **D-457 STORY-173 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 14 passes; streak P12/P13/P14; final HEAD 7b2a73e; 2602/0. Trajectory P1(1H+3doc)→P2(1M+1N)→P3-P5(NITs)→P6 CLEAN→P7(1N)→P8(4N)→P9/P10 CLEAN→P11(1N)→P12/P13/P14 CLEAN. Code FROZEN since P2. Advisory A-12-01 accepted. 4 process-gaps. STORY-INDEX v3.70→v3.71.** | **CONVERGED (D-457)** | Pre-merge LOW-fix burst next (human approved). trajectory-tail →1→0→0→0 |
| **D-456 STORY-173 pre-delivery AC↔BC fidelity check DRIFT-FOUND (D-456, 2026-07-15). 2 BLOCKING/3 MEDIUM/3 LOW. SR-173-01 BLOCKING (T0881 tactic string); SR-173-02 BLOCKING (IEC104-FINDINGS-CAP-001 uncovered). BC-2.19.028 v1.0 created (MAX_IEC104_FINDINGS=10_000; dropped_findings); BC-INDEX v2.31→v2.32. STORY-173 realigned v2.0 (SR-173-01..08; AC-173-007/008). STORY-INDEX v3.70.** | **COMPLETE (D-456)** | 4th F3-DECOMPOSITION-BC-FIDELITY. trajectory-tail →0→0→0→0 |
| **D-455 STORY-172 DELIVERED (PR #406 d64e5fe squash-merged to develop, 2026-07-15, human-authorized). Per-story adversarial CONVERGED 3-clean (D-454). Security PASS — SEC-001-S168 carry-bound FULLY MITIGATED. pr-reviewer APPROVE. CI 13/13. Demos 9 artifacts/8 ACs scrub PASS. ADR-0013 committed to develop. develop=d64e5fe (6 unreleased: STORY-167..172); stories_delivered 110→111. Wave-81 gate SATISFIED. STORY-INDEX v3.68→v3.69.** | **DELIVERED (D-455)** | Wave-81 SATISFIED. trajectory-tail →0→0→0→0 |
| **D-454 STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-15). 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6. Worktree fec9bfa; 2584/0; 26 story_172 tests. F-172-001/002 HIGH (D-452); F-172-201 LOW (D-453); F-172-301 NIT (fec9bfa). Deferred: F-172-003 LOW VP-045 vacuity STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. STORY-INDEX v3.67→v3.68.** | **CONVERGED (D-454)** | Demos/PR next. trajectory-tail →1→0→0→0 |

## Decisions Log

| ID | Decision | Date |
|----|----------|------|
| D-001..D-301 (exhaustive). Greenfield through feature-enip-v0.11.0; see cycles/*/decisions-archive.md for full range. | — | — |
| D-302..D-436 (exhaustive). Fix-tls through feature-protocol-coverage through v0.12.1; see cycles/history/decision-log-archive.md for full range. | — | — |
| D-437 | Engine/project triage (human-directed, 2026-07-14): 4 engine issues filed in drbothen/vsdd-factory (#635 streak persistence, #636 demo-recorder scrub, #637 input-hash hook divergence, #638 finding-ID canonicalization; all DF-VALIDATION-001-validated); STORY-166 re-scoped v1.1 5→3 pts (AC-002/004 moved to engine, AC-003 narrowed); STORY-INDEX v3.57 (729 pts); PG-HASH-HOOK-DIVERGENCE now tracked upstream as #637. | 2026-07-14 |
| D-438 | feature-iec104 F2 spec-evolution CONVERGED (2026-07-14). 12 adversarial passes (3-clean streak P10/P11/P12), 12 fresh-context consistency audits. Research-agent canonical-fact validation at P1. Final spec: BC-INDEX v2.28 (30 new BCs); PRD v1.56; VP-INDEX v2.46 (VP-044..047); ARCH-INDEX v2.16 (SS-19 + ADR-0013). Input-hash audit STALE=0. 4 F3-handoff items. Convergence report: `.factory/cycles/feature-iec104/adversarial/f2-convergence-report.md`. | 2026-07-14 |
| D-439 | feature-iec104 F2 gate APPROVED WITH first-frame-guard mandate (human, 2026-07-14). Option<u16> applied: SS-19 v1.6, BC-2.19.023 v1.2, BC-2.19.024 v1.3. ADR-013 Decision 6 first-frame baseline added. 27 BC-2.19.* input-hashes recomputed a153144. MITRE ATT&CK ICS v19.1 pin confirmed. Scoped re-verify CLEAN. RETRANSMIT-NS-FALSEPOS-001 carried to F3. F2 CLOSED. Review: `.factory/cycles/feature-iec104/adversarial/f2-first-frame-guard-review.md`. | 2026-07-14 |
| D-440 | feature-iec104 F3 story decomposition COMPLETE (2026-07-14). STORY-167..174 registered (E-22 IEC-104 Passive Analyzer, 8 stories, 36 pts, waves 76–83). Serialized: one story/wave due to src/analyzer/iec104.rs contention; 170-171 file-seq edge per F-F3P2-005 precedent. dep-graph v3.9 acyclic (137 edges). STORY-INDEX v3.58 (127 stories/765 pts; 83 waves). F3 handoff: BC-2.10.010 EMITTED harness; RETRANSMIT-NS-FALSEPOS-001 carried to STORY-171. input-hash scan MATCH=127 STALE=0. Plan gate APPROVED (human). | 2026-07-14 |
| D-441 | STORY-167 DELIVERED (PR #401 e65e0d6, 2026-07-14, human-authorized merge). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 4 passes, streak P2/P3/P4; Pass-1 findings remediated in commit 557b6a8. Security CLEAN. AI/pr-reviewer APPROVE (2 MINOR accepted + 3 NIT). CI 13/13 green. Demo 7 artifacts scrub PASS. Wave-76: single-story wave — per-story 3-clean satisfies wave-level (identical diff). develop=e65e0d6; stories_delivered=106; STORY-INDEX v3.59. | 2026-07-14 |
| D-442 | STORY-168 CHECKPOINT (2026-07-14) — GREEN (64/64 iec104 tests). Worktree `.worktrees/STORY-168` (local, NOT pushed). Branch: `feature/STORY-168-iec104-frame-discrimination-session-sm`. Scope: frame discrimination + session SM (STARTDT/STOPDT/TESTFR U-frame). Iec104FlowState 5 fields including last_ns_c2s/last_ns_s2c: Option<u16> (SS-19 v1.6). N(S) tracking UNWIRED — STORY-171. No dispatch wiring — STORY-173. (Superseded by D-443.) | 2026-07-14 |
| D-443 | STORY-168 DELIVERED (PR #402 b720fd96, 2026-07-14, human-authorized merge). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 4 passes, streak P2/P3/P4. SEC-001-S168 MEDIUM (MAX_IEC104_CARRY_BYTES enforcement — carry fields declared-only, no append path, inert in STORY-168) DEFERRED to STORY-172. SEC-002/003 accepted. AI APPROVE (3 NIT, NIT-1 fixed). CI 13/13 green. Demo 6 artifacts docs/demo-evidence/STORY-168/ scrub PASS. Wave-77 gate SATISFIED. develop=b720fd96; stories_delivered=107; STORY-INDEX v3.60. | 2026-07-14 |
| D-444 | STATE.md compacted (vsdd-factory:compact-state, 2026-07-14) — historical Decision Log rows D-302..D-436 (exhaustive) archived to `cycles/history/decision-log-archive.md`; resolved Open Items archived to `cycles/history/open-items-archive.md`; Notes section archived; STATE.md slimmed from ~490 lines to 247 lines (wc-l); all structural validators satisfied. Zero information loss (Historical Content index updated). Pre-STORY-169 maintenance. | 2026-07-14 |
| D-445 | STORY-169 DELIVERED (PR #403 ac01d9f2, 2026-07-14, human-authorized). Story realigned to BCs pre-impl (v1.1, F3-drift correction: parse_asdu/Asdu broken-out, min-6, first_ioa Option). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 4 passes, streak P2/P3/P4; Pass-1 MEDIUM (stale todo!() docstring) remediated 0debf98. Security 0 CRIT/HIGH/MEDIUM (SEC-001 LOW carry-bound deferred STORY-172); AI APPROVE (0 blocking/major, 1 MINOR demo-format+1 NIT out-of-scope accepted); CI 13/13; demo 6 artifacts scrub PASS. 3rd of 8 IEC-104 stories. develop=ac01d9f2. PG-REDGREEN-COMMENT-CLEANUP: 2nd occurrence; F3-DECOMPOSITION-BC-FIDELITY tracked. STORY-170 pre-known drift: AsduHeader→Asdu rename (4 sites) + cot_test AC. | 2026-07-14 |
| D-446 | STORY-170 pre-delivery BC-realignment v2.0 (2026-07-14). Corrected significant F3-decomposition drift caught pre-code: (1) AsduHeader→Asdu/extract_asdu_header→parse_asdu naming (STORY-169 delivered broken-out Asdu struct); (2) FALSE-POSITIVE bug fixed — interrogation/clock-sync C_IC/C_CI/C_CS (TypeIDs 100/101/103) were speccing T0827 Possible, BC-2.19.021 says benign/no-finding; (3) AC-170-002 confidence Possible→Likely (BC-2.19.020); (4) reserved-TypeID scope corrected to TypeID=0 or [128,255] (BC-2.19.022); (5) dispatch table corrected (45-47 T1692.001 only; 48-51 +T0836; 105 T0827 Likely; 128-255 T0814); (6) AC-170-007 cot_test [TEST]-tagging added (BC-2.19.017 inv1); (7) BC-2.19.017 added to inputs; input-hash 7c3c35c (canonical; story-writer set d4fcb27 via hook, corrected per PG-HASH-HOOK-DIVERGENCE). STORY-INDEX v3.62. Reinforces F3-DECOMPOSITION-BC-FIDELITY (2nd confirmed occurrence). | 2026-07-14 |
| D-447 | STORY-170 DELIVERED (PR #404 0bd93f8, 2026-07-15, human-authorized). Story BC-realigned v2.0 pre-impl (F3-drift: FALSE-POSITIVE interrogation→T0827 bug fixed per BC-2.19.021; T0827 Likely; cot_test [TEST]; reserved-scope). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 5 passes, streak P3/P4/P5; F-170-001 MEDIUM (CASDU/first_ioa context) + F-P2-L1 LOW (stale Red-Gate test-header) remediated. Security CLEAN (0 CRIT/HIGH/MEDIUM; 4 INFO accepted); AI APPROVE (0 blocking, 2 NIT accepted); CI 13/13; 136 iec104 tests; demo 6 artifacts scrub PASS; worktree+branch cleaned. 4th of 8 IEC-104 stories. develop=0bd93f8; stories_delivered=109; STORY-INDEX v3.63. PG-REDGREEN-COMMENT-CLEANUP: 3rd occurrence — READY-TO-CODIFY. | 2026-07-15 |
| D-448 | STORY-171 DELIVERED (PR #405 1a64380, 2026-07-15, human-authorized). Pre-delivery AC↔BC fidelity check: NO drift (Option<u16> first-frame model already BC-faithful). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 4 passes, streak P2/P3/P4; Pass-1 F-171-001 MEDIUM (stale header, PG-REDGREEN 4th occurrence) + F-171-002 (PC-C2 coverage) remediated 27bb678. Security PASS (0 CRIT/HIGH); AI APPROVE (1 cycle, 0 blocking); CI 13/13; 166 iec104 tests; demo 8 artifacts scrub PASS; worktree+branch cleaned. 5th of 8 IEC-104 stories. RETRANSMIT-NS-FALSEPOS-001 resolved (documented EC-007, fail-closed per INV-3). develop=1a64380; stories_delivered=110; STORY-INDEX v3.64. PG-REDGREEN-COMMENT-CLEANUP: 4th occurrence — CODIFY-NOW. | 2026-07-15 |
| D-449 | Session /wrap (human-requested, 2026-07-15) — pipeline PAUSED at feature-iec104 F4, 5/8 stories delivered (STORY-167..171); develop=1a64380 (5 unreleased); .factory tree clean; docs/adr/0013 untracked-on-develop flagged for STORY-173/docs-commit; no in-flight TDD, no open PRs, no story worktrees. Resume: /vsdd-factory:next-step → STORY-172. | 2026-07-15 |
| D-450 | Post-merge late-review findings captured (PR #404/#405 already merged; verdicts corroborated APPROVE+PASS+CI-13/13). 3 deferred items recorded: IEC104-TIMED-CMD-GAP-001 (timed control TypeIDs 58–64 detection gap, sec-review-170 L-001, follow-on story, DF-VALIDATION-001-gated), IEC104-FINDINGS-CAP-001 (unbounded findings Vec, sec-review-170 M-001, → STORY-173 dispatcher cap), IEC104-FINDING-DIRECTION-001 (Finding.direction None though known, pr-review-171 MINOR-2, → STORY-172/173 cleanup). Pipeline remains PAUSED (wrap). | 2026-07-15 |
| D-451 | Pre-STORY-172 spec-remediation burst COMPLETE (2026-07-15, pipeline RESUMED). AC↔BC fidelity check DRIFT-FOUND — 3rd F3-DECOMPOSITION-BC-FIDELITY occurrence: SR-172-01 BLOCKING (FlowId→FlowKey); SR-172-02 MEDIUM (carry-overflow discard-all-new canonical vectors); SR-172-03 MEDIUM (BC-2.19.026 PC4 vs ADR-013 contradiction — research-validated EMIT-WITH-DEDUP; CVE-2023-5768/Snort3/Wireshark/Zeek). Spec updates: BC-2.19.027 v1.1 / BC-2.19.026 v1.6 / BC-INDEX v2.29 / ADR-013 reconciled / SS-19 v1.7 / ARCH-INDEX v2.17. STORY-172 realigned v2.0; input-hash af0f732. STORY-INDEX v3.65. | 2026-07-15 |
| D-452 | STORY-172 Pass-1 adversarial remediation COMPLETE (2026-07-15). F-172-001 HIGH (aggregate carry pre-check = Ptacek/Newsham evasion channel; WALK-FIRST-RESIDUAL-BOUND research-validated) + F-172-002 HIGH (dispatch wiring regression-unguarded; 6 effect tests added) + F-172-003 LOW (VP-045 proptests vacuous; DEFERRED STORY-174) + F-172-004 NIT (story_168 stale header; joins PG-REDGREEN-SIBLING-SWEEP). Spec chain: BC-2.19.025 v1.2 / BC-INDEX v2.30 / ADR-013 Decision 2 rewritten / SS-19 v1.8 / ARCH-INDEX v2.18. STORY-172 v3.0 hash 246add6; 2584/0. STORY-INDEX v3.66. | 2026-07-15 |
| D-453 | STORY-172 Pass-2 adversarial → 1L F-172-201 prose precision REMEDIATED (2026-07-15). Entry-check vs residual-after-walk equivalents proved. Spec chain: BC-2.19.025 v1.3 / BC-INDEX v2.31 / SS-19 v1.9 / ARCH-INDEX v2.19. STORY-172 v3.1 hash 938645f. Code unchanged 4dc85c4; 2584/0. STORY-INDEX v3.67. Pass 3 pending. | 2026-07-15 |
| D-454 | STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-15). 6 passes total; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6. Worktree HEAD fec9bfa; 2584/0; 26 story_172 tests. Findings remediated: F-172-001/002 HIGH (D-452); F-172-201 LOW (D-453); F-172-301 NIT (fec9bfa). Deferred: F-172-003 LOW VP-045 vacuity → STORY-174; F-172-004 NIT → PG-REDGREEN-SIBLING-SWEEP. STORY-INDEX v3.68. | 2026-07-15 |
| D-455 | STORY-172 DELIVERED (PR #406 d64e5fe squash-merged to develop, 2026-07-15, human-authorized per DF-MERGE-AUTH-CLASSIFIER-001). Per-story adversarial CONVERGED 3-clean (D-454). Security PASS — SEC-001-S168 carry-bound FULLY MITIGATED (deferred finding closed). pr-reviewer APPROVE (F1 direction/timestamp=None deferred; F2 unreachable-guard accepted by design). CI 13/13. Demos 9 artifacts/8 ACs scrub PASS. ADR-0013 committed to develop (c5b098f). develop=d64e5fe (6 unreleased: STORY-167..172); stories_delivered 110→111. Wave-81 gate SATISFIED. STORY-INDEX v3.68→v3.69. | 2026-07-15 |
| D-456 | STORY-173 pre-delivery AC↔BC fidelity check DRIFT-FOUND (2 BLOCKING/3 MEDIUM/3 LOW) — 4th F3-DECOMPOSITION-BC-FIDELITY (story-decomposition imprecision; ADR-013/code were correct). SR-173-01 BLOCKING (T0881 tactic string "impact" → MitreTactic::IcsInhibitResponseFunction, would not compile); SR-173-02 BLOCKING security GAP (IEC104-FINDINGS-CAP-001 uncovered though assigned here). Remediation: product-owner created BC-2.19.028 v1.0 (per-session findings cap MAX_IEC104_FINDINGS=10_000 mirroring DNP3 BC-2.15.022/ENIP BC-2.17.022; dropped_findings counter; anchor PC-2); BC-INDEX v2.31→v2.32. STORY-173 realigned to v2.0 (SR-173-01..08; AC-173-007 cap + AC-173-008 dispatcher wiring; input-hash f3d3673). STORY-INDEX v3.69→v3.70. | 2026-07-15 |
| D-457 | STORY-173 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 14 passes; streak P12/P13/P14; final HEAD 7b2a73e; 2602/0. Trajectory P1(1H F-173-001 + 3 doc)→P2(1M+1N)→P3/P4/P5(doc-tense NITs)→P6 CLEAN→P7(1N stale protocols.rs cardinality)→P8(4N stale mitre.rs seeded-count)→P9/P10 CLEAN→P11(1N non-discriminating EMITTED_IDS test)→P12/P13/P14 CLEAN. Code FROZEN/CLEAN since P2; post-P2 tail = doc-accuracy + test-cosmetic reviewer-variance. Fix commits 11f695c/366b176/6a3a372/a652464/b4cca90/7462e9c/5363be6/a73a3b9/f6b91f1/7b2a73e; demo 3d22003. Advisory A-12-01 accepted. Process-gaps: PG-DOC-CURRENCY-SWEEP, PG-ADVERSARY-IDLE-NO-REPORT, PG-ADVERSARY-SEVERITY-CALIBRATION, PG-STATE-RECOVERY-SCOPE. STORY-INDEX v3.70→v3.71. | 2026-07-16 |
| D-458 | STORY-173 DELIVERED (PR #408 084ff93 squash-merged to develop, 2026-07-16, human-authorized per DF-MERGE-AUTH-CLASSIFIER-001). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 17 total passes; initial 3-clean P12/P13/P14 at 7b2a73e (D-457); pre-merge LOW-fix burst (LOW#1 flows_analyzed + LOW#2 packets_analyzed + SEC-001 is_valid doc/BC-2.19.006 v1.2; commits 0bfc977/5325cf2/3ec6ac1; 2604/0 tests); fresh 3-clean A/B/C re-convergence on 3ec6ac1. IEC104-FINDINGS-CAP-001 RESOLVED (CWE-400/770; MAX_IEC104_FINDINGS=10_000; BC-2.19.028 anchor). Security PASS. pr-reviewer APPROVE. CI 13/13. Demos 9 artifacts/8 ACs scrub PASS. Stray commit 105497f (sec001 fix agent to main checkout) discarded. 7th of 8 IEC-104 stories. develop=084ff93 (7 unreleased: STORY-167..173); stories_delivered 111→112. Wave-82 gate SATISFIED. 5 process-gaps filed (PG-DOC-CURRENCY-SWEEP/PG-ADVERSARY-IDLE-NO-REPORT/PG-ADVERSARY-SEVERITY-CALIBRATION/PG-STATE-RECOVERY-SCOPE/PG-VERIFY-ALL-WORKTREES). BC-INDEX v2.33. STORY-INDEX v3.72. | 2026-07-16 |

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
| DRIFT-SPRINT-STATE-FIELD-FORM-001 | sprint-state.yaml uses both `merge_sha:` and `merge_commit:` field names inconsistently across wave entries. Vestigial file (STORY-INDEX is authoritative wave registry). | wave-75 gate S-7.02 justified deferral (D-435) | vestigial-file retirement at next housekeeping pass |
| DRIFT-BACKMERGE-SQUASH-001 | v0.12.1 back-merge PR #400 was squash-merged; main (fedcea4) NOT ancestor of develop (7b11b83). Trees ARE identical (5e75fd5) — history-only divergence. | v0.12.1 release (D-436, 2026-07-13) | resolve at next release cut |
| DRIFT-VP039-BC207038-TLS-TODO-001 | VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039 (TLS reassembly). Out of feature-iec104 scope. | feature-iec104 F2 review (D-438, 2026-07-14) | SS-07 TLS owner — next TLS maintenance sweep |

---

## Active Carry-Forwards

| ID | Summary | Target |
|----|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11 (human decision). | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review 1 NIT deferred from wave-74 gate (human-ratified); joins wave-75 NIT. | Next bin-touch PR |
| PERF-RERUN-001 | AC-149-003 quiescent re-run pending (load avg 52.57 at maint-2026-07-11; human deferred). | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred from maint-2026-07-11; next feature wave. | Next feature wave or maintenance |
| STORY-166 | E-11, 3 pts, wave-TBD, hash b56924f; S-7.02 carry from wave-75. | Next wave after feature-iec104 F4 |
| F3-handoff cleanup | F-F3P12-002 (STORY-151 pointer note), F-F3P13-002 (STORY-154 frontmatter SS-05), F-F3P17-001 (STORY-154 cross-layer trace). | F4 implementation per-story |
| SEC-001-S158 / SEC-002-S158 | CWE-22 LOW advisories in `bin/lint-cycle-artifact` (deferred until mandatory CI wiring). DF-VALIDATION-001-gated. | bin/lint-cycle-artifact CI wiring |
| PG-REDGREEN-COMMENT-CLEANUP | Stub-era Red-Gate phrase surviving into GREEN delivery: STORY-167 (P1) + STORY-169 (P1) + STORY-170 (F-P2-L1) + STORY-171 (F-171-001 Pass-1) + **STORY-173 (A-12-01 test header)**. **5 occurrences — CODIFY-NOW.** Codification: CI/pre-commit grep guard that FAILS if implemented function or test module contains stale Red-Gate phrases. Candidate vehicle: STORY-174 hardening wave. PG-REDGREEN-SIBLING-SWEEP (sibling headers in STORY-168/169) still queued. | STORY-174 hardening |
| PG-REDGREEN-SIBLING-SWEEP | Already-merged baseline stale Red-Gate test-module headers: `tests/iec104_analyzer_tests.rs` mod story_168 (~L662-663, L1498-1499) + mod story_169 (~L1544) contain false "MUST FAIL/todo!() stub" comments. Inert at runtime but confusing to reviewers. F-172-004 NIT (story_168 stale header) joins this item. Sweep at the feature wave-gate or a maintenance touch. | feature-iec104 wave-gate or next maintenance |
| F3-DECOMPOSITION-BC-FIDELITY | **4 CONFIRMED occurrences: STORY-169** (flat vs broken-out fields; wrong guards) **+ STORY-170** (false-positive T0827; confidence Possible→Likely; reserved-TypeID scope; naming) **+ STORY-172** (FlowId→FlowKey nonexistent; carry-overflow discard-all-new semantics; malformed-LEN PC4 contradiction) **+ STORY-173** (T0881 tactic string "impact" → MitreTactic; compilation blocker). All corrected pre-delivery. **CODIFY-NOW.** Codification: mandatory pre-delivery AC↔BC fidelity check as F3/F4 gate step. Vehicle: STORY-174 or cycle-close E-11 follow-up. | STORY-174 pre-delivery check + cycle-close codification |
| F-172-003 | VP-045 proptests vacuity (STORY-172 Pass-1 F-172-003 LOW): carrier loop covers no meaningful shrinkage paths; proptest framework calls without domain generators. DEFERRED — STORY-174 formal hardening target. | STORY-174 (formal hardening wave) |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant, DEFERRED) TypeIDs 58–64 (timed control variants C_SC_TA_1=58 .. C_BO_TA_1=64) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Out of scope per BC-2.19.019 (lists only 45–51); NOT a STORY-170 defect. Evasion gap: control commands via timed variants bypass detection. Source: sec-review-170 L-001 (PR #404). DF-VALIDATION-001 required before filing any GitHub issue. | Follow-on detection story (new BC + detection arm for TypeIDs 58–64, or feature-cycle extension) |
| IEC104-FINDING-DIRECTION-001 | (code-quality, MINOR, DEFERRED) track_ns_desync (STORY-171) leaves Finding.direction = None while direction IS known (formats direction into evidence string instead). Finding.direction: Option<Direction> exists per LESSON-P2.08 for JSON consumers to distinguish client/server anomalies. Source: pr-review-171 MINOR-2 (PR #405). | STORY-174 or maintenance touch. Consider whether detect_iec104_threats / process_u_frame findings should carry direction once dispatcher provides it. |
| PG-SPEC-VERSION-CITATION-CURRENCY | Spec-version bumps must include src/ comments and CHANGELOG entries in the citation-currency sweep set (surfaced by F-172-301 NIT, D-454). | cycle-close lessons codification |
| PG-DOC-CURRENCY-SWEEP | Post-adversarial doc-accuracy drift consumed 12 of 17 STORY-173 passes (P3..P14 minus the CLEAN passes). A pre-adversarial code-comments/test-header doc sweep (analogous to delivery-doc AC-165-003) would reduce adversarial pass count. | STORY-174 or cycle-close codification |
| PG-ADVERSARY-IDLE-NO-REPORT | Adversary agents completing CLEAN passes sometimes emitted no report, making CLEAN vs idle indistinguishable from orchestrator logs. Recurring behavior flagged across multiple STORY-173 passes. | cycle-close lessons codification |
| PG-ADVERSARY-SEVERITY-CALIBRATION | Whole-source doc sweeps at late passes (P12) generated advisory findings against code FROZEN since P2. Adversary instances diverging on severity calibration for code that hasn't changed. | cycle-close lessons codification |
| PG-STATE-RECOVERY-SCOPE | Session-boundary state recovery must verify ALL worktrees and the main develop checkout simultaneously. Omitting the main checkout created the stray-commit 105497f gap (D-458). | cycle-close codification |
| PG-VERIFY-ALL-WORKTREES | Post-agent verification must span ALL worktrees and the main develop checkout. A fix agent committed to the main develop checkout (not a worktree), creating stray commit 105497f which had to be discarded. | cycle-close codification |

---

## Session Resume Checkpoint

**D-458 COMPLETE (2026-07-16): STORY-173 DELIVERED (PR #408 084ff93, wave-82 SATISFIED). 17 adversarial passes (initial P12/P13/P14 + pre-merge LOW-fix burst + A/B/C re-convergence). IEC104-FINDINGS-CAP-001 RESOLVED (CWE-400/770; MAX_IEC104_FINDINGS=10_000). 2604/0; develop=084ff93; stories_delivered=112. BC-INDEX v2.33. STORY-INDEX v3.72. NEXT: STORY-174 (wave-83). trajectory-tail →0→0→0→0**

Prior checkpoint (D-457 STORY-173 per-story adversarial CONVERGED, 2026-07-16) archived to `cycles/feature-iec104/session-checkpoints.md`.

- **Date:** 2026-07-16. Position: feature-iec104 F4 delta-implementation IN PROGRESS; D-458 STORY-173 DELIVERED; develop=084ff93 (7 unreleased). STORY-174 wave-83 next. trajectory-tail →0→0→0→0
- **Ground truth:** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `084ff93`. DRIFT-BACKMERGE-SQUASH-001 still applies. 7 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2) + STORY-170 (PR #404 0bd93f8) + STORY-171 (PR #405 1a64380) + STORY-172 (PR #406 d64e5fe) + STORY-173 (PR #408 084ff93).
- **Wave status:** Waves 76–82 DELIVERED (D-441/443/445/447/448/455/458): STORY-167..173. Wave 83 IN PROGRESS: STORY-174 (TDD next). 7 of 8 IEC-104 stories merged to develop.
- **Remaining delivery sequence:** STORY-174 (wave-83, formal hardening + PG-REDGREEN-COMMENT-CLEANUP grep-guard + F-172-003 VP-045 proptest fix + VP-044 Kani + VP-047 Fuzz + cargo-mutants) → F5 scoped adversarial → F6 targeted hardening → F7 delta convergence → release cut.
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f); F-172-003 (VP-045 vacuity → STORY-174); IEC104-TIMED-CMD-GAP-001 (TypeIDs 58–64 detection gap, DF-VALIDATION-001-gated); IEC104-FINDING-DIRECTION-001 (Finding.direction None → STORY-174); PG-REDGREEN-COMMENT-CLEANUP (5 occurrences CODIFY-NOW → STORY-174 grep-guard); PG-VERIFY-ALL-WORKTREES + PG-STATE-RECOVERY-SCOPE + PG-DOC-CURRENCY-SWEEP + PG-ADVERSARY-IDLE-NO-REPORT + PG-ADVERSARY-SEVERITY-CALIBRATION (all → cycle-close codification).
- **Spec versions:** BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 / STORY-INDEX v3.72 / dep-graph v3.9 (137 edges).
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
| **Resolved Open Items (pre-feature-iec104)** | `cycles/history/open-items-archive.md` |
| feature-iec104 F2 convergence report | `cycles/feature-iec104/adversarial/f2-convergence-report.md` (12 passes, CONVERGED P10/P11/P12, D-438) |
| feature-iec104 F2 gate review (first-frame guard) | `cycles/feature-iec104/adversarial/f2-first-frame-guard-review.md` (CLEAN; 2 LOW applied; D-439) |
| feature-iec104 MITRE pin confirmation | `cycles/feature-iec104/research/f2-mitre-pin-confirmation.md` (8 techniques CONFIRMED-AT-v19.1; D-439) |
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
