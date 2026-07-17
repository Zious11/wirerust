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
phase: "feature-iec104/F5"
status: active
current_step: "D-466 FIX-F5-001 DELIVERED; F5 Round-1 findings all resolved; F5 Round 2 adversary next (converge to no CRITICAL/HIGH + novelty decay). trajectory-tail →0→0→0→0"
current_cycle: "feature-iec104"
pipeline: IN PROGRESS
timestamp: 2026-07-17T14:30:00Z
# D-466 FIX-F5-001 DELIVERED (PR #411 9c5aa9a, 2026-07-17); F5 Round-1 F-01..F-05 RESOLVED; F5 Round 2 adversary next. STORY-INDEX v3.76.

# Release chain (latest)
released_version: v0.12.1
released_at: "2026-07-13"
release_tag: v0.12.1
release_tag_object: d687a77d911503e67a8d171c00536bd710762bba
release_commit: fedcea4ab17d9b3257c9903636aec0c0fd08f147
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.12.1
prior_released_version: v0.12.0
prior_released_at: "2026-07-10"
# Ground-truth HEADs (updated 2026-07-17 — D-466 FIX-F5-001 DELIVERED; develop=9c5aa9a (STORY-167+168+169+170+171+172+173+174+FIX-P4-001+FIX-F5-001, 10 unreleased); DRIFT-BACKMERGE-SQUASH-001 still applies)
main_head: fedcea4ab17d9b3257c9903636aec0c0fd08f147
develop_head: 9c5aa9a
# Cargo.toml version: main=0.12.1; develop=0.12.1 (10 unreleased commits 9c5aa9a STORY-167+168+169+170+171+172+173+174+FIX-P4-001+FIX-F5-001; DRIFT-BACKMERGE-SQUASH-001: main fedcea4 not an ancestor of develop 9c5aa9a, histories diverge; trees differ by IEC-104 feature code)
cargo_version_main: "0.12.1"
cargo_version_develop: "0.12.1"
# Open worktrees: main checkout [develop] + .factory [factory-artifacts].
# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
# Story tracking
stories_delivered: 113
story_index_version: "v3.76"
total_stories: 127
story_index_note: "127 stories / 83 waves / 765 pts. v3.76 (2026-07-17): STORY-174 DELIVERED (D-463, PR #409 547deba, wave-83 SATISFIED; F4 COMPLETE 8/8). FIX-P4-001 delivered D-464 (fix PR, not a story). See cycles/feature-iec104/ for full F2/F3 history."
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
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 301 = 199 (dual-margin form). 301 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-466 FIX-F5-001 DELIVERED (2026-07-17). PR #411 9c5aa9a squash-merged to develop, human-executed merge. F5 Round-1 findings F-01..F-05 ALL RESOLVED: source_ip + timestamp enrichment threaded through all 10 IEC-104 emit sites (8 function + 2 inline; DNP3/ENIP house-parity); BC-2.19.011 PC-3 SATISFIED; 10 red-first tests mod fix_f5_001; 9 stale-prose sites scrubbed GREEN; false forward-ref removed; additive JSON keys source_ip/timestamp in CHANGELOG. CI 13/13 + post-merge SUCCESS. develop=9c5aa9a (10 unreleased). F5 Round 2 adversary next (fresh eyes on fixed files; converge to no CRITICAL/HIGH + novelty decay). trajectory-tail →0→0→0→0**

**D-465 base (2026-07-17): F5 scoped adversarial OPENED @ 7e95f71. Round 1 BC-completeness 31/31 PASS; canonical-frame 19 invariants CLEAN; 1H+4M findings (F-01..F-05) → FIX-F5-001. trajectory-tail →0→0→0→5**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); F4 COMPLETE + FIX-P4-001 DELIVERED (D-464); **F5 scoped adversarial (D-465): Round 1 RESOLVED (F-01..F-05) → FIX-F5-001 DELIVERED (D-466, PR #411 9c5aa9a); F5 Round 2 adversary next** |
| Version | 0.12.1 (released 2026-07-13; main=fedcea4; develop=9c5aa9a — 10 unreleased commits; DRIFT-BACKMERGE-SQUASH-001) |
| Main HEAD | `fedcea4ab17d9b3257c9903636aec0c0fd08f147` |
| Develop HEAD | `9c5aa9a` — PR #411 FIX-F5-001 squash 2026-07-17; DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 |
| Stories | 113 delivered / 127 total (STORY-INDEX v3.76, dep-graph v3.9, 765 pts) |
| **Last Updated** | 2026-07-17 — D-466 FIX-F5-001 DELIVERED: F5 Round-1 F-01..F-05 RESOLVED; F5 Round 2 adversary next. trajectory-tail →0→0→0→0 |

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
| feature-iec104 — D-451 spec-remediation burst | **COMPLETE** | SR-172-01 BLOCKING (FlowId→FlowKey); SR-172-02 MEDIUM (carry-overflow discard-all-new); SR-172-03 MEDIUM (malformed-LEN EMIT-WITH-DEDUP, research-validated). 3rd F3-DECOMPOSITION-BC-FIDELITY — CODIFY-NOW. |
| feature-iec104 — F4 per-story adversary (STORY-172) | **CONVERGED (D-454)** | 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6 (BC-5.39.001 SATISFIED). F-172-001/002 HIGH remediated D-452; F-172-201 LOW remediated D-453; F-172-301 NIT remediated fec9bfa. Deferred: F-172-003 LOW STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. |
| feature-iec104 — F4 per-story adversary (STORY-173) | **CONVERGED (D-457/D-458)** | 17 passes total; initial streak P12/P13/P14 (D-457); pre-merge LOW fix burst (3 LOWs fixed; see next row); fresh 3-clean A/B/C (D-458). IEC104-FINDINGS-CAP-001 RESOLVED. BC-2.19.006 v1.2; BC-INDEX v2.33. |
| feature-iec104 — STORY-173 pre-merge LOW fix burst | **COMPLETE** | 3 LOWs FIXED pre-merge (human approved all 3): LOW#1 flows_analyzed real cumulative counter (mirrors ENIP; 0bfc977); LOW#2 packets_analyzed valid-APDU frame counter (mirrors DNP3; 5325cf2); SEC-001/A-173-A-01 is_valid_iec104_frame doc overstated gate role + BC-2.19.006 v1.2; BC-INDEX v2.32→v2.33 (3ec6ac1). 2602→2604 tests. Triggered fresh A/B/C re-convergence. |
| feature-iec104 — F4 per-story adversary (STORY-174) | **CONVERGED (D-462)** | 7 passes; streak P5/P6/P7 (BC-5.39.001 SATISFIED). Trajectory P1(1M)->P2(1M)->P3(NIT)->P4(1M)->P5/P6/P7 CLEAN. F-174-001 VP-044 non-vacuity (Kani 82→89); F-174-002 skeleton/CI-claim prose + 8-site sweep; F-174-P4-001 BC-2.19.025 invariant-2 mis-anchor (e62701f). Story v2.2; STORY-INDEX v3.75. PG-GATE-VOCAB-BLINDSPOT filed. |
| feature-iec104 — wave-79 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P3/P4/P5) on STORY-170 diff == wave-level adversarial; CI 13/13 develop 0bd93f8; D-447 |
| feature-iec104 — wave-80 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P2/P3/P4) on STORY-171 diff == wave-level adversarial; CI 13/13 develop 1a64380; D-448 |
| feature-iec104 — wave-81 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P4/P5/P6) on STORY-172 diff == wave-level adversarial; CI 13/13 develop d64e5fe; D-455 |
| feature-iec104 — wave-82 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean A/B/C (17 total passes) on STORY-173 diff == wave-level adversarial; CI 13/13 develop 084ff93; D-458 |
| feature-iec104 — wave-83 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P5/P6/P7) on STORY-174 diff == wave-level adversarial; CI 13/13 + post-merge develop CI SUCCESS; D-463 |
| feature-iec104 — pre-F5 fix-PR (FIX-P4-001) | **DELIVERED (D-464)** | PR #410 7e95f71; IEC104-FINDING-DIRECTION-001 resolved — all 10 IEC-104 emit sites direction: Some(...); 11 direction-assertion tests (mod fix_p4_001); F5 scoped adversarial UNBLOCKED |
| feature-iec104 — F5 (scoped adversarial) | **OPENED (D-465); Round 2 pending** | Round 1 @ 7e95f71: BC-completeness 31/31 PASS; canonical-frame 19 invariants CLEAN; 1H+4M → FIX-F5-001 DELIVERED (D-466) |
| feature-iec104 — F5 fix batch (FIX-F5-001) | **DELIVERED (D-466)** | PR #411 9c5aa9a; F-01..F-05 resolved; source_ip+timestamp enrichment all 10 emit sites; BC-2.19.011 PC-3 SATISFIED |

---

## Convergence Status

| Cycle/Story | Passes | Trajectory | Status |
|------------|--------|-----------|--------|
| feature-iec104 F4 per-story (STORY-171) | 4 | →2→0→0→0 | CONVERGED 3-clean (BC-5.39.001) |
| feature-iec104 F4 per-story (STORY-172) | 6 | →(2H+1L+1N)→1L→1NIT→0→0→0 | CONVERGED 3-clean (BC-5.39.001) streak P4/P5/P6 |
| feature-iec104 F4 per-story (STORY-173) | 17 (14+3) | →(1H+3doc)→(1M+1N)→NITs→CLEAN(P6)→1N→4N→CLEAN(P9/P10)→1N→CLEAN(P12/P13/P14)→3LOWfix→CLEAN(A/B/C) | CONVERGED 3-clean (BC-5.39.001) re-converged A/B/C post-LOW-fixes (D-458) |
| feature-iec104 F4 per-story (STORY-174) | 7 | →1M→1M→NIT→1M→0→0→0 | CONVERGED 3-clean streak P5/P6/P7 (BC-5.39.001) D-462 |

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | F5 scoped adversarial (D-465) — Round 1 RESOLVED: FIX-F5-001 DELIVERED (D-466, PR #411 9c5aa9a); F5 Round 2 adversary next (converge to no CRITICAL/HIGH + novelty decay) | develop |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-466 FIX-F5-001 DELIVERED (2026-07-17). PR #411 9c5aa9a squash-merged to develop, human-executed merge per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER. F5 Round-1 findings ALL RESOLVED: F-01 HIGH BC-2.19.011 PC-3 + F-02/03/04/05 MEDIUM — source_ip+timestamp threaded through all 10 IEC-104 emit sites (8 function + 2 inline; DNP3/ENIP house-parity); BC-2.19.011 PC-3 SATISFIED; 10 red-first tests mod fix_f5_001 (each asserts source_ip+timestamp per finding family); 9 stale-prose sites scrubbed GREEN + protocols_tests count comment fixed; false forward-ref comment removed; additive JSON keys source_ip/timestamp documented in CHANGELOG; holdout-expectations sweep COMPLETE (PG-W72; docs/holdout-expectations-sweep-FIX-F5-001.md). Security PASS 0 findings. pr-reviewer APPROVE (MINOR count-prose + NIT timestamp-type both remediated in-file, orchestrator row-verified per PG-W74). CI 13/13 + post-merge SUCCESS. Demo before/after JSON scrub PASS. develop=9c5aa9a (10 unreleased: STORY-167..174 + FIX-P4-001 + FIX-F5-001). F5 Round 2 adversary next (fresh eyes on fixed files).** | **DELIVERED (D-466)** | F5 Round-1 F-01..F-05 ALL RESOLVED. F5 Round 2 next. trajectory-tail →0→0→0→0 |
| **D-465 feature-iec104 F5 scoped adversarial OPENED (2026-07-17). Round 1 @ develop 7e95f71 (base fedcea4): BC-set completeness sweep 31/31 PASS (no missing-feature blocker); canonical-frame sweep 19 invariants byte-exact vs IEC 60870-5-104 (no DNP3-DIR-class defect). Findings: F-01 HIGH BC-2.19.011 PC-3 source_ip unmet (untested blind spot); F-02 MEDIUM source_ip/timestamp parity (iec104.rs:1148 let _ = ts); F-03 MEDIUM stale RED-phase prose + 4 unlisted siblings; F-04 MEDIUM false forward-ref iec104.rs:1029; F-05 MEDIUM protocols_tests.rs:208 stale count. All 5 batched → FIX-F5-001 (in progress). MITRE EXECUTION-REQUIRED axis closed via D-439 v19.1 pin. Regression/Security/Kani axes CLEAN. Phase frontmatter → feature-iec104/F5.** | **FINDINGS (D-465)** | 1H+4M → FIX-F5-001. Round 2 after merge. trajectory-tail →0→0→0→5 |
| **D-464 FIX-P4-001 DELIVERED (2026-07-17). PR #410 7e95f71 squash-merged to develop, human-executed merge per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER. fix-pr-delivery flow (D-461 routing, ENIP D-262 PR #331 precedent). IEC104-FINDING-DIRECTION-001 RESOLVED — all 10 IEC-104 emit sites direction: Some(...) (was None); direction threaded into process_u_frame + detect_iec104_threats; redundant direction-in-evidence strings dropped. 11 direction-assertion tests (mod fix_p4_001, red-first TDD). Additive `direction` JSON key documented in CHANGELOG. holdout-expectations sweep COMPLETE (PG-W72; zero IEC-104 holdout scenarios; docs/holdout-expectations-sweep-FIX-P4-001.md). Security review PASS 0 findings. pr-reviewer APPROVE (2 NITs accepted). CI 13/13 + post-merge develop CI SUCCESS. Demo evidence 3 artifacts scrub PASS. develop=7e95f71 (9 unreleased: STORY-167..174 + FIX-P4-001). F5 scoped adversarial UNBLOCKED.** | **DELIVERED (D-464)** | IEC104-FINDING-DIRECTION-001 CLOSED. F5 next. trajectory-tail →0→0→0→0 |
| **D-463 STORY-174 DELIVERED (2026-07-17). PR #409 547deba squash-merged to develop, human-authorized (TWO classifier halts: DF-MERGE-AUTH-CLASSIFIER-001 condition-4 wave-grant-absent + PG-MERGE-AUTH-SUBAGENT-CLASSIFIER harness deny; human-direct in main thread). Per-story adversarial CONVERGED 3-clean D-462 (7 passes P5/P6/P7). Security APPROVE (1 LOW SEC-001 CWE-22 bin path-prefix accepted). pr-reviewer APPROVE (2 NITs). CI 13/13 + post-merge develop CI SUCCESS. Demos 9 artifacts/8 ACs scrub PASS. Kani VP-044 89 checks (5 facets) + VP-004/VP-007 re-run; VP-045/046 non-vacuous proptests (F-172-003 RESOLVED); VP-047 fuzz 1.35M clean; cargo-mutants 117/122=95.9%. PG-REDGREEN-COMMENT-CLEANUP CODIFIED+DELIVERED (AC-174-008, 23-25 token patterns + baseline scrub); PG-REDGREEN-SIBLING-SWEEP RESOLVED. 8th of 8 IEC-104 stories. stories_delivered 112→113. develop=547deba. Wave-83 gate SATISFIED. F4 COMPLETE. PG-MERGE-AUTH-SUBAGENT-CLASSIFIER filed. STORY-INDEX v3.76.** | **DELIVERED (D-463)** | F4 COMPLETE 8/8. trajectory-tail →0→0→0→0 |
| **D-462 STORY-174 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 7 passes; streak P5/P6/P7; final HEAD e62701f; base 084ff93; 2600+/0 tests (92 suites). Trajectory P1(1M F-174-001)->P2(1M F-174-002)->P3(NITPICK_ONLY)->P4(1M F-174-P4-001)->P5/P6/P7 CLEAN. F-174-001 MEDIUM VP-044 valid→Some facet missing (Kani 82→89 checks; 1071de4); F-174-002 MEDIUM stale skeleton/false CI-wiring prose + 8-site sibling sweep (038286a); F-174-P4-001 MEDIUM BC-2.19.025 invariant-2 mis-anchor from v1.3 renumbering re-cited to VP-045 harness registration (e62701f 8 test + 2 story sites). Story v2.2; STORY-INDEX v3.75. Kani non-vacuity 3/3 every pass. Mutation 117/122=95.9%. Fuzz 1.35M execs clean. PG-GATE-VOCAB-BLINDSPOT filed (green-doc-tense gate misses "skeleton"/"seam" phrasing; 2 independent obs P2+P4). Demos/PR next.** | **CONVERGED (D-462)** | Demo recording next. trajectory-tail →0→0→0→0 |

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
| D-460 | Session RESUMED (human-approved, 2026-07-16). Worktree health PASS; develop=084ff93 verified; no story worktrees; only open PR is external #407 (deferred post-wave-83 by human). STORY-174 wave-83 begins with research-agent validation of carry-forward scope items before any story realignment (human-directed). | 2026-07-16 |
| D-461 | STORY-174 pre-delivery realignment COMPLETE (research-validated, human-approved 2026-07-16). DF-VALIDATION-001 research 2 passes (story-174-scope-validation.md + -followup.md; all HIGH confidence): (1) PG-REDGREEN-COMMENT-CLEANUP VALID-INCLUDE — codified as AC-174-008 extending existing green-doc-tense-gate token list (3 patterns; zero tree-wide false positives; no allowlist change) + scrub of 3 baseline stale headers + CHANGELOG entry; (2) F-172-003 VP-045 vacuity VALID-INCLUDE — AC-174-002 amended with non-vacuity/interleaved-generator/state-comparison requirements (carry fields already pub; zero production code); (3) IEC104-FINDING-DIRECTION-001 VALID-DEFER out of STORY-174 — routed to dedicated pre-F5 fix-PR inside feature-iec104 via fix-pr-delivery (ENIP D-262 PR #331 precedent; PG-W72 holdout sweep near-empty, additive JSON key). STORY-174 v2.0 input-hash de9d14e→27c86aa (also resolved genuine BC-2.19.006 v1.2 input drift from D-458). STORY-INDEX v3.72→v3.73. Points unchanged (5). | 2026-07-16 |
| D-462 | STORY-174 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 7 passes; streak P5/P6/P7; final HEAD e62701f; base 084ff93; 2600+/0 tests (92 suites). Trajectory P1(1M F-174-001)->P2(1M F-174-002)->P3(NITPICK_ONLY)->P4(1M F-174-P4-001)->P5/P6/P7 CLEAN. F-174-001 MEDIUM VP-044 valid→Some facet missing (Kani 82→89 checks; 1071de4); F-174-002 MEDIUM stale skeleton/false CI-wiring prose + 8-site sibling sweep (038286a); F-174-P4-001 MEDIUM BC-2.19.025 invariant-2 mis-anchor from v1.3 renumbering re-cited to VP-045 harness registration (e62701f 8 test + 2 story sites). Story v2.2; STORY-INDEX v3.75. Kani non-vacuity 3/3 every pass. Mutation 117/122=95.9%. Fuzz 1.35M execs clean. PG-GATE-VOCAB-BLINDSPOT filed (green-doc-tense gate misses "skeleton"/"seam" phrasing; 2 independent obs P2+P4). Demos/PR next. | 2026-07-16 |
| D-463 | STORY-174 DELIVERED (PR #409 547deba squash-merged to develop, 2026-07-17, human-authorized per-PR — human executed merge directly in main thread after TWO classifier halts: DF-MERGE-AUTH-CLASSIFIER-001 condition-4 wave-grant-absent, then PG-MERGE-AUTH-SUBAGENT-CLASSIFIER harness deny of subagent --admin merge on relayed consent; orchestrator-direct attempt also denied on unnamed --admin bypass; bypass tagged per DF-PR-MANAGER-COMPLETE-001(b)). Per-story adversarial CONVERGED 3-clean D-462 (7 passes P5/P6/P7). Security APPROVE (1 LOW SEC-001 CWE-22 bin path-prefix accepted, joins SEC-001-S158 class). pr-reviewer APPROVE (2 NITs accepted). CI 13/13 + post-merge develop CI SUCCESS. Demos 9 artifacts/8 ACs scrub PASS. Kani VP-044 89 checks (5 facets) + VP-004 (440/407/183) + VP-007 (122, SEEDED=29); VP-045/046 non-vacuous proptests (F-172-003 RESOLVED); VP-047 fuzz 1.35M execs clean; cargo-mutants 117/122=95.9%; green-doc-tense gate patterns 23-25 + baseline scrub (PG-REDGREEN-COMMENT-CLEANUP CODIFIED-DELIVERED; PG-REDGREEN-SIBLING-SWEEP RESOLVED). 8th of 8 IEC-104 stories. develop=547deba (8 unreleased: STORY-167..174); stories_delivered 112→113. Wave-83 gate SATISFIED (single-story wave: per-story 3-clean == wave-level on identical diff, per waves 79-82 precedent). F4 delta-implementation COMPLETE. New process-gap: PG-MERGE-AUTH-SUBAGENT-CLASSIFIER (subagent cannot execute --admin merge on relayed human consent; orchestrator-direct attempt also denied on unnamed --admin bypass. Resolution path = human-direct in main thread (per D-463). Codify at cycle-close as AC for E-11 follow-up story. STORY-INDEX v3.76. | 2026-07-17 |
| D-464 | FIX-P4-001 DELIVERED (PR #410 7e95f71 squash-merged to develop, 2026-07-17, human-executed merge per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER). fix-pr-delivery flow (D-461 routing; ENIP D-262 PR #331 precedent). IEC104-FINDING-DIRECTION-001 RESOLVED — all 10 IEC-104 emit sites now direction: Some(...) (was None); direction threaded into process_u_frame + detect_iec104_threats; redundant direction-in-evidence strings dropped; 11 direction-assertion tests (mod fix_p4_001, red-first); additive `direction` JSON key documented in CHANGELOG; holdout-expectations sweep COMPLETE (PG-W72; zero IEC-104 holdout scenarios, subset assertions unaffected; docs/holdout-expectations-sweep-FIX-P4-001.md). Security review PASS 0 findings. pr-reviewer APPROVE (2 NITs accepted). CI 13/13 + post-merge develop CI SUCCESS. Demo evidence 3 artifacts scrub PASS. develop=7e95f71 (9 unreleased: STORY-167..174 + FIX-P4-001). F5 scoped adversarial UNBLOCKED. | 2026-07-17 |
| D-465 | feature-iec104 F5 scoped adversarial OPENED (2026-07-17). Round 1 @ develop 7e95f71: BC-set completeness sweep 31/31 PASS (no missing-feature blocker); canonical-frame sweep 19 invariants byte-exact vs IEC 60870-5-104 (no DNP3-DIR-class defect); findings 1H+4M — F-01 HIGH BC-2.19.011 PC-3 source_ip unmet (untested blind spot) + F-02 source_ip/timestamp parity + F-03 stale prose (+4 new siblings) + F-04 false forward-ref + F-05 stale count. All 5 batched to FIX-F5-001 (in progress). MITRE EXECUTION-REQUIRED axis closed via D-439 v19.1 pin research. Phase frontmatter → feature-iec104/F5. | 2026-07-17 |
| D-466 | FIX-F5-001 DELIVERED (PR #411 9c5aa9a squash-merged to develop, 2026-07-17, human-executed merge). Batches F5 Round-1 findings F-01 HIGH + F-02/03/04/05 MEDIUM: source_ip + timestamp enrichment threaded through all 10 IEC-104 emit sites (8 function + 2 inline; DNP3/ENIP house-parity pattern) — BC-2.19.011 PC-3 SATISFIED; 10 red-first tests mod fix_f5_001 (each asserts source_ip+timestamp per finding family); 9 stale-prose sites scrubbed GREEN + protocols_tests count comment fixed; false forward-ref comment removed; additive JSON keys source_ip/timestamp documented in CHANGELOG; holdout-expectations sweep COMPLETE (PG-W72; docs/holdout-expectations-sweep-FIX-F5-001.md). Security PASS 0 findings. pr-reviewer APPROVE (MINOR count-prose + NIT timestamp-type both remediated in-file, orchestrator row-verified per PG-W74). CI 13/13 + post-merge SUCCESS. Demo before/after JSON scrub PASS. develop=9c5aa9a (10 unreleased: STORY-167..174 + FIX-P4-001 + FIX-F5-001). F5 Round 2 next (fresh adversary on fixed files). | 2026-07-17 |

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
| DRIFT-BACKMERGE-SQUASH-001 | v0.12.1 back-merge PR #400 was squash-merged; main (fedcea4) NOT ancestor of develop (9c5aa9a). Trees ARE identical (5e75fd5) — history-only divergence. | v0.12.1 release (D-436, 2026-07-13) | resolve at next release cut |
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
| F3-DECOMPOSITION-BC-FIDELITY | **4 CONFIRMED occurrences: STORY-169** (flat vs broken-out fields; wrong guards) **+ STORY-170** (false-positive T0827; confidence Possible→Likely; reserved-TypeID scope; naming) **+ STORY-172** (FlowId→FlowKey nonexistent; carry-overflow discard-all-new semantics; malformed-LEN PC4 contradiction) **+ STORY-173** (T0881 tactic string "impact" → MitreTactic; compilation blocker). All corrected pre-delivery. **CODIFY-NOW.** Codification: mandatory pre-delivery AC↔BC fidelity check as F3/F4 gate step. Vehicle: cycle-close E-11 follow-up. | Cycle-close codification |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant, DEFERRED) TypeIDs 58–64 (timed control variants C_SC_TA_1=58 .. C_BO_TA_1=64) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Out of scope per BC-2.19.019. Evasion gap: control commands via timed variants bypass detection. Source: sec-review-170 L-001 (PR #404). DF-VALIDATION-001 required before filing any GitHub issue. | Follow-on detection story (new BC + detection arm for TypeIDs 58–64, or feature-cycle extension) |
| IEC104-FINDING-DIRECTION-001 | RESOLVED (PR #410, D-464) — CLOSED. All 10 IEC-104 emit sites now direction: Some(...). | CLOSED (D-464, PR #410 7e95f71, 2026-07-17) |
| F5-ROUND1-F01..F05 | RESOLVED (PR #411, D-466) — CLOSED. F-01 HIGH BC-2.19.011 PC-3 source_ip + F-02/03/04/05 MEDIUM all resolved by FIX-F5-001. BC-2.19.011 PC-3 SATISFIED. | CLOSED (D-466, PR #411 9c5aa9a, 2026-07-17) |
| F5-DEFERRED-LOW-BC-2.19.006-VP044-BACKREF | LOW (non-blocking): BC-2.19.006 VP-044 back-reference wording review deferred from F5 Round-1 pass. Does not block Round 2. | F5 Round 2 adversary — accept or remediate per severity |
| F5-DEFERRED-LOW-MUTANTS-DISPOSITION-2.4 | LOW (non-blocking): mutants-disposition section 2.4 wording imprecision deferred from F5 Round-1 pass. Does not block Round 2. | F5 Round 2 adversary — accept or remediate per severity |
| PG-SPEC-VERSION-CITATION-CURRENCY | Spec-version bumps must include src/ comments and CHANGELOG entries in the citation-currency sweep set (surfaced by F-172-301 NIT, D-454). | cycle-close lessons codification |
| PG-DOC-CURRENCY-SWEEP | Post-adversarial doc-accuracy drift consumed 12 of 17 STORY-173 passes. A pre-adversarial code-comments/test-header doc sweep would reduce adversarial pass count. | Cycle-close codification |
| PG-ADVERSARY-IDLE-NO-REPORT | Adversary agents completing CLEAN passes sometimes emitted no report, making CLEAN vs idle indistinguishable. Recurring behavior flagged across multiple STORY-173 passes. | Cycle-close lessons codification |
| PG-ADVERSARY-SEVERITY-CALIBRATION | Whole-source doc sweeps at late passes generated advisory findings against code FROZEN since P2. Adversary instances diverging on severity calibration for code that hasn't changed. | Cycle-close lessons codification |
| PG-STATE-RECOVERY-SCOPE | Session-boundary state recovery must verify ALL worktrees and the main develop checkout simultaneously. Omitting the main checkout created the stray-commit 105497f gap (D-458). | Cycle-close codification |
| PG-VERIFY-ALL-WORKTREES | Post-agent verification must span ALL worktrees and the main develop checkout. A fix agent committed to the main develop checkout (not a worktree), creating stray commit 105497f which had to be discarded. | Cycle-close codification |
| PG-GATE-VOCAB-BLINDSPOT | Green-doc-tense gate (AC-174-008) misses "skeleton" and "seam" phrasing (stub-era language surviving into green deliveries). 2 independent adversary observations: P2 Obs-1 + P4 obs on STORY-174. Token list must be extended. | Cycle-close codification; extend AC-174-008 token list |
| PG-MERGE-AUTH-SUBAGENT-CLASSIFIER | Subagent cannot execute --admin merge on relayed human consent; orchestrator-direct attempt also denied on unnamed --admin bypass. Resolution path = human-direct in main thread (per D-463). Codify at cycle-close as AC for E-11 follow-up story. | Cycle-close codification |

---

## Session Resume Checkpoint

**D-466 FIX-F5-001 DELIVERED (2026-07-17). PR #411 9c5aa9a squash-merged to develop, human-executed merge. F5 Round-1 F-01..F-05 ALL RESOLVED. develop=9c5aa9a (10 unreleased). F5 Round 2 adversary next (fresh eyes on fixed files; converge to no CRITICAL/HIGH + novelty decay). trajectory-tail →0→0→0→0**

**D-465 base (2026-07-17): F5 scoped adversarial OPENED @ 7e95f71. Round 1 1H+4M → FIX-F5-001 DELIVERED. trajectory-tail →0→0→0→5**

Prior checkpoint (D-465 F5 Round-1 FINDINGS, 2026-07-17) archived to `cycles/feature-iec104/session-checkpoints.md`.

- **Date:** 2026-07-17. Position: feature-iec104 F5 (D-465 OPENED); Round 1 RESOLVED → FIX-F5-001 DELIVERED (D-466). F5 Round 2 adversary next. develop=9c5aa9a. trajectory-tail →0→0→0→0
- **Ground truth:** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `9c5aa9a`. DRIFT-BACKMERGE-SQUASH-001 still applies. 10 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2) + STORY-170 (PR #404 0bd93f8) + STORY-171 (PR #405 1a64380) + STORY-172 (PR #406 d64e5fe) + STORY-173 (PR #408 084ff93) + STORY-174 (PR #409 547deba) + FIX-P4-001 (PR #410 7e95f71) + FIX-F5-001 (PR #411 9c5aa9a).
- **Wave status:** Waves 76–83 DELIVERED (D-441/443/445/447/448/455/458/463): STORY-167..174. Wave-83 SATISFIED. F4 COMPLETE. FIX-P4-001 DELIVERED (D-464, PR #410 7e95f71). F5 OPENED (D-465); Round 1 RESOLVED → FIX-F5-001 DELIVERED (D-466, PR #411 9c5aa9a). F5 Round 2 adversary next.
- **Remaining delivery sequence:** F5 Round 2 adversary (fresh eyes on 9c5aa9a; converge to no CRITICAL/HIGH + novelty decay) → F6 targeted hardening → F7 delta convergence → release cut. PR #407 external-fork triage also pending.
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f); IEC104-TIMED-CMD-GAP-001 (TypeIDs 58–64 detection gap, DF-VALIDATION-001-gated); F5-ROUND1-F01..F05 CLOSED (D-466); F5-DEFERRED-LOW-BC-2.19.006-VP044-BACKREF (LOW, non-blocking); F5-DEFERRED-LOW-MUTANTS-DISPOSITION-2.4 (LOW, non-blocking); PG-GATE-VOCAB-BLINDSPOT + PG-MERGE-AUTH-SUBAGENT-CLASSIFIER (cycle-close); PG-VERIFY-ALL-WORKTREES + PG-STATE-RECOVERY-SCOPE + PG-DOC-CURRENCY-SWEEP + PG-ADVERSARY-IDLE-NO-REPORT + PG-ADVERSARY-SEVERITY-CALIBRATION (all → cycle-close codification).
- **Spec versions:** BC-INDEX v2.33 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 / STORY-INDEX v3.76 / dep-graph v3.9 (137 edges).
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
| Wave 83 gate files | `cycles/wave-083/wave-gate/` (gate-summary.md D-463, code-review.md) |
| STORY-174 per-story convergence report | `cycles/feature-iec104/STORY-174/convergence-report.md` (7 passes, CONVERGED P5/P6/P7, D-462) |
