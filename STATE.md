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
current_step: "D-476 PR #414 ADOPTED (2026-07-19). ArcavenAE scorecard-guard squash-merged to develop (fcd57dcb; human-executed 2026-07-19T01:54:40Z). Security-triaged SAFE-WITH-CHANGES; F1 CWE-494 RESOLVED-CLEAN (all 4 SHA↔tag MATCH; harden-runner v2.19.4 NOT AFFECTED); CI 13/13 incl. action-pin-gate. Workflow inert until SCORECARD_ENABLED set. PR-414-FORK-SCORECARD CLOSED/ADOPTED. SCORECARD-ENABLEMENT-RUNBOOK carry-forward added. PR #407 governance OPEN (disjoint files). Pipeline ACTIVE (steady-state). trajectory-tail →0→0→0→0"
current_cycle: "feature-iec104"
pipeline: ACTIVE
timestamp: 2026-07-19T02:05:00Z
released_version: v0.13.0
released_at: "2026-07-18"
release_tag: v0.13.0
release_tag_object: 03f35e4f0499dde0bcdb7a79dff9844ec57f1cdb
release_commit: 67a06b6f82654d2af79d023b15ac56ab03182ffd
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.13.0
prior_released_version: v0.12.1
prior_released_at: "2026-07-13"
main_head: 67a06b6f82654d2af79d023b15ac56ab03182ffd
develop_head: fcd57dcbd8b13074ffb57086f5f179dc30f1d026
cargo_version_main: "0.13.0"
cargo_version_develop: "0.13.0"
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
stories_delivered: 113
story_index_version: "v3.77"
total_stories: 132
story_index_note: "132 stories / 83 waves / 777 pts. v3.77 (2026-07-18): STORY-175..179 added (E-11 cycle-close, 5 draft stories, 12 pts). B-001/B-002 FIXED (PRD v1.57, BC-2.19.002 v1.3). STORY-167 v1.1 AC propagation. 13-story input-hash re-baseline BENIGN. See cycles/feature-iec104/ for full history."
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
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 263 = 237 (dual-margin form). 263 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-476 PR #414 ADOPTED (2026-07-19). Scorecard-guard squash-merged to develop (fcd57dcb; human-executed 2026-07-19T01:54:40Z). Security-triaged SAFE-WITH-CHANGES; F1 RESOLVED-CLEAN; CI 13/13 incl. action-pin-gate. Workflow inert until SCORECARD_ENABLED set. Pipeline ACTIVE (steady-state). Resume: /vsdd-factory:next-step. trajectory-tail →0→0→0→0**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); **RELEASED v0.13.0 (D-473, 2026-07-18). F1→F7 CONVERGED; CYCLE CLOSED (D-475, 2026-07-18): S-7.02 SATISFIED, 9 PGs → STORY-175..179 (12 pts), B-001/B-002 FIXED (PRD v1.57 + BC-2.19.002 v1.3), PR #419 82ad2ed merged. Pipeline ACTIVE.** |
| Version | 0.13.0 (released 2026-07-18; main=67a06b6; develop=fcd57dc — D-476 PR #414 scorecard-guard adopted (2026-07-19); DRIFT-BACKMERGE-SQUASH-001 retained; trees reconciled at v0.13.0) |
| Main HEAD | `67a06b6f82654d2af79d023b15ac56ab03182ffd` |
| Develop HEAD | `fcd57dcbd8b13074ffb57086f5f179dc30f1d026` — D-476 PR #414 scorecard-guard adopted (2026-07-19); DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 |
| Stories | 113 delivered / 132 total (STORY-INDEX v3.77, dep-graph v3.9, 777 pts) |
| **Last Updated** | 2026-07-19 — D-476 PR #414 scorecard-guard ADOPTED. Pipeline ACTIVE (steady-state). trajectory-tail →0→0→0→0 |

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
| **feature-iec104 cycle-close (S-7.02)** | **CLOSED (D-475)** | 9 PGs → STORY-175..179 (12 pts, E-11 epic); B-001/B-002 FIXED; PR #419 82ad2ed; STORY-INDEX v3.77; 132 stories / 777 pts |

---

## Convergence Status

Per-story F4 convergence details archived to `cycles/feature-iec104/convergence-trajectory.md`.
F5 phase-level trajectory: 5 rounds, code frozen R2, `5H/M→2M→1H→1M→1L(NB)` — CONVERGED (D-468).

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | **CLOSED (D-475, 2026-07-18)** — v0.13.0 RELEASED (D-473); F1→F7 CONVERGED (D-470); S-7.02 SATISFIED: 9 PGs → STORY-175..179; B-001/B-002 FIXED; PR #419 82ad2ed; Pipeline ACTIVE | develop (fcd57dc) |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-476 PR #414 ADOPTED (2026-07-19). ArcavenAE ci/scorecard-guard squash-merged to develop fcd57dcb (human-executed 2026-07-19T01:54:40Z). Security-triaged SAFE-WITH-CHANGES (report: .factory/planning/pr-414-security-triage.md); F1 CWE-494 RESOLVED-CLEAN (all 4 SHA↔tag MATCH via GitHub API; harden-runner v2.19.4 NOT AFFECTED by any advisory). CI 13/13 SUCCESS incl. action-pin-gate. Workflow inert until SCORECARD_ENABLED=true set. PR #407 governance OPEN/UNAFFECTED (disjoint files, no overlap). SCORECARD-ENABLEMENT-RUNBOOK carry-forward added; PR-414-FORK-SCORECARD CLOSED.** | **ADOPTED (D-476)** | develop=fcd57dc. trajectory-tail →0→0→0→0 |
| **D-475 feature-iec104 CYCLE-CLOSE (2026-07-18). S-7.02 checklist SATISFIED: 9 process-gaps codified into 5 draft stories STORY-175..179 (E-11 epic, 12 pts; STORY-INDEX v3.77); B-001/B-002 doc nits FIXED (PRD v1.57, BC-2.19.002 v1.3 + title cascade, BC-INDEX v2.34); STORY-167 v1.1 AC propagation; IEC104-DEMO-TYPEID45-MISLABEL DELIVERED via docs PR #419 82ad2edd12ad1f9dad61a03a4760d4112d45ccc2 squash-merged to develop (human-executed merge; pr-reviewer APPROVE 0 findings; CI 13/13; step-8 halt per PG-MERGE-AUTH-SUBAGENT-CLASSIFIER, human-direct merge — pattern reconfirmed); STORY-164/165 input-hashes re-baselined BENIGN (canonical tool; 132/0 scan); DRIFT-SPRINT-STATE-FIELD-FORM-001 pre-resolved (sprint-state.yaml already absent); mutants.out residue deleted. feature-iec104 declared CLOSED. Pipeline ACTIVE (resumed from D-474 pause).** | **CLOSED (D-475)** | S-7.02 SATISFIED. lessons.md written. All codified PGs removed from carry-forwards. Open: PR #407 governance, PR #414 triage, STORY-166 + STORY-175..179 wave-TBD. trajectory-tail →0→0→0→0 |
| **D-474 SESSION WRAP (2026-07-18). Human-requested pipeline pause at clean milestone. v0.13.0 RELEASED (feature-iec104 F1→F7 CONVERGED + shipped). All session PRs (#409-418) merged. No in-flight work. Pipeline PAUSED by human direction.** | **PAUSED (D-474)** | Clean stop post-v0.13.0. trajectory-tail →0→0→0→0 |
| **D-473 v0.13.0 RELEASED (2026-07-18). Release PR #417 release/0.13.0→main merged 67a06b6 (human --merge). Tag v0.13.0 (object 03f35e4f); release.yml SUCCESS; GitHub Release published + 4 platform binaries. Back-merge PR #418 SQUASH → develop af3ecbd. 13 commits released. DRIFT-BACKMERGE-SQUASH-001 retained.** | **RELEASED (D-473)** | main=67a06b6, develop=af3ecbd, both 0.13.0. trajectory-tail →0→0→0→0 |
| **D-472 PR #407 (external fork ArcavenAE/wirerust) security-triaged (2026-07-18): SAFE-WITH-CHANGES, 0 blocking vulns. 3 required-if-adopted changes. DEFERRED by human — PR left OPEN; governance question unresolved. Triage: .factory/planning/pr-407-security-triage.md.** | **TRIAGED SAFE-WITH-CHANGES (D-472)** | PR left OPEN, DEFERRED by human. trajectory-tail →0→0→0→0 |

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
| D-476 | PR #414 ADOPTED (2026-07-19). ArcavenAE fork ci/scorecard-guard squash-merged to develop (fcd57dcbd8b13074ffb57086f5f179dc30f1d026; human-executed 2026-07-19T01:54:40Z). Security-triaged SAFE-WITH-CHANGES (triage: .factory/planning/pr-414-security-triage.md; F1 CWE-494 RESOLVED-CLEAN: all 4 SHA↔tag mappings MATCH via GitHub API, harden-runner v2.19.4 NOT AFFECTED by any advisory — dated section appended to triage file 2026-07-19). CI 13/13 SUCCESS incl. action-pin-gate. Adds .github/workflows/scorecards.yml; workflow inert until SCORECARD_ENABLED=true set. F2 (CWE-200) and enablement runbook noted in SCORECARD-ENABLEMENT-RUNBOOK carry-forward. PR #407 governance OPEN/UNAFFECTED (disjoint files, no overlap verified). | 2026-07-19 |

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
| DRIFT-BACKMERGE-SQUASH-001 | v0.12.1 back-merge PR #400 was squash-merged; v0.13.0 cut re-encountered this drift resolved-forward. Back-merge #418 also squash-merged per human choice. main (67a06b6) NOT ancestor of develop (fcd57dc). Trees ARE identical for released content — history-only divergence. DRIFT PERSISTS. | v0.12.1 release (D-436, 2026-07-13); re-encountered v0.13.0 (D-473, 2026-07-18) | resolve at a future release via true-merge back-merge if desired (human deferred; squash pattern retained D-473) |
| DRIFT-VP039-BC207038-TLS-TODO-001 | VP-INDEX carries stale present-tense "PO must add BC-2.07.038 postcondition/EC + Red-Gate test name" TODOs for VP-039 (TLS reassembly). Out of feature-iec104 scope. | feature-iec104 F2 review (D-438, 2026-07-14) | SS-07 TLS owner — next TLS maintenance sweep |

---

## Active Carry-Forwards

| ID | Summary | Target |
|----|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11 (human decision). | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review 1 NIT deferred from wave-74 gate (human-ratified); joins wave-75 NIT. | Next bin-touch PR |
| PERF-RERUN-001 | AC-149-003 quiescent re-run pending (load avg 52.57 at maint-2026-07-11; human deferred). | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred from maint-2026-07-11; next feature wave. | Next feature wave or maintenance |
| STORY-166 | E-11, 3 pts, wave-TBD, hash b56924f; S-7.02 carry from wave-75. | Next wave scheduling |
| F3-handoff cleanup | F-F3P12-002 (STORY-151 pointer note), F-F3P13-002 (STORY-154 frontmatter SS-05), F-F3P17-001 (STORY-154 cross-layer trace). | F4 implementation per-story |
| SEC-001-S158 / SEC-002-S158 | CWE-22 LOW advisories in `bin/lint-cycle-artifact` (deferred until mandatory CI wiring). DF-VALIDATION-001-gated. | bin/lint-cycle-artifact CI wiring |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant, DEFERRED) TypeIDs 58–64 (timed control variants) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Evasion gap. DF-VALIDATION-001 required before filing any GitHub issue. | Follow-on detection story (new BC + detection arm) |
| PR-407-FORK-RELEASE-OPS | External ArcavenAE PR #407 security-triaged SAFE-WITH-CHANGES (D-472; triage at .factory/planning/pr-407-security-triage.md); DEFERRED — governance decision pending. Resume without re-running security review. | Governance decision when authorized |
| SCORECARD-ENABLEMENT-RUNBOOK | Before setting SCORECARD_ENABLED=true: document that publish_results:true publishes security-posture data to OpenSSF public API (F2, CWE-200, LOW); optional harden-runner bump v2.19.4→v2.20.0 via Dependabot. PR #414 ADOPTED (D-476). | Whenever scorecard is enabled |

---

## Session Resume Checkpoint

**D-476 PR #414 ADOPTED (2026-07-19). Pipeline ACTIVE (steady-state). trajectory-tail →0→0→0→0**

Prior checkpoint (D-475 feature-iec104 CYCLE-CLOSE, 2026-07-18) archived to `cycles/feature-iec104/session-checkpoints.md`.

- **Date:** 2026-07-19. Position: D-476 PR #414 ADOPTED; scorecard-guard workflow merged to develop (fcd57dcb); pipeline ACTIVE (steady-state). main=67a06b6 (v0.13.0), develop=fcd57dc. DRIFT-BACKMERGE-SQUASH-001 retained.
- **Ground truth:** main = `67a06b6f82654d2af79d023b15ac56ab03182ffd` (v0.13.0); develop = `fcd57dcbd8b13074ffb57086f5f179dc30f1d026` (D-476 PR #414 scorecard-guard; human squash-merge 2026-07-19T01:54:40Z). DRIFT-BACKMERGE-SQUASH-001 still applies.
- **In-flight work:** NONE. No stories mid-TDD, no open factory PRs, no story worktrees, no adversarial loop active.
- **Open items (not blocking):** (a) PR #407 governance decision (SAFE-WITH-CHANGES, DEFERRED — triage at `.factory/planning/pr-407-security-triage.md`, do NOT re-run security review); (b) STORY-166 (E-11, 3 pts, wave-TBD) still drafted/undelivered; (c) STORY-175..179 (5 draft stories, E-11 epic, 12 pts, wave-TBD) awaiting wave scheduling.
- **Pending human decisions:** PR #407 disposition; wave scheduling for STORY-166 + STORY-175..179; SCORECARD_ENABLED enablement (see SCORECARD-ENABLEMENT-RUNBOOK carry-forward).
- **Spec versions:** BC-INDEX v2.34 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.57 / STORY-INDEX v3.77 / dep-graph v3.9 (137 edges).
- **Resume command:** `/vsdd-factory:next-step` (reads STATE.md, continues from checkpoint). Pipeline ACTIVE — ready for next wave scheduling or PR triage.

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
| **feature-iec104 cycle-close lessons** | `cycles/feature-iec104/lessons.md` (9 process-gaps [codified] → STORY-175..179; D-475, 2026-07-18) |
| **Phase Progress granular rows (F4 waves/adversary/fixes)** | `cycles/feature-iec104/phase-progress-archive.md` (D-451 burst, wave-79..83, STORY-172/173/174 per-story adversary, FIX-P4-001/F5-001..004) |
| **Convergence Trajectory (F4 per-story + F5 phase)** | `cycles/feature-iec104/convergence-trajectory.md` |
| feature-iec104 F2 convergence report | `cycles/feature-iec104/adversarial/f2-convergence-report.md` (12 passes, CONVERGED P10/P11/P12, D-438) |
| feature-iec104 F2 gate review (first-frame guard) | `cycles/feature-iec104/adversarial/f2-first-frame-guard-review.md` (CLEAN; 2 LOW applied; D-439) |
| feature-iec104 MITRE pin confirmation | `cycles/feature-iec104/research/f2-mitre-pin-confirmation.md` (8 techniques CONFIRMED-AT-v19.1; D-439) |
| Session checkpoints (feature-iec104, all prior) | `cycles/feature-iec104/session-checkpoints.md` (waves 76–83 era + D-471 E2E + D-472 PR #407 + D-473 v0.13.0 RELEASED + D-474 SESSION WRAP + D-475 CYCLE-CLOSE checkpoints) |
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
| feature-iec104 F5 adversarial reviews | `.factory/phase-f5-adversarial/round-1-review.md` through `round-5-review.md`; `convergence-summary.md` (D-468) |
| feature-iec104 F6 gate verdict + hardening artifacts | `.factory/phase-f6-hardening/f6-gate-verdict-iec104.md` (D-469 PASS); `kani-results.md`, `fuzz-results.md`, `mutation-results.md`, `security-scan-results.md` |
| feature-iec104 F7 convergence artifacts | `.factory/phase-f7-convergence/delta-convergence-report.md` (D-470 CONVERGED); `traceability-chain-delta.md`; `consistency-audit.md` |
