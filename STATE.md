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
status: active
current_step: "STORY-170 BC-realigned v2.0 (D-446, 2026-07-14). Pre-delivery drift corrected: FALSE-POSITIVE bug fixed (C_IC/C_CI/C_CS interrogation TypeIDs 100/101/103 were speccing T0827, BC-2.19.021 says no-finding); AC-170-002 Possible→Likely (BC-2.19.020); reserved-TypeID scope corrected (BC-2.19.022); AsduHeader→Asdu naming; cot_test [TEST]-tagging AC added (BC-2.19.017 inv1). input-hash 7c3c35c (canonical). STORY-INDEX v3.62. F3-DECOMPOSITION-BC-FIDELITY: 2 confirmed occurrences. NEXT: STORY-170 code cycle (wave-79, worktree → stub → test → impl → adversarial → demo → PR) from develop ac01d9f2. trajectory-tail →0→0→0→0"
current_cycle: "feature-iec104"
pipeline: PAUSED
timestamp: 2026-07-15T00:04:00Z
# STORY-170 BC-realigned v2.0 (D-446, 2026-07-14); STORY-INDEX v3.62; F3-DECOMPOSITION-BC-FIDELITY 2 confirmed occurrences.

# Release chain (latest)
released_version: v0.12.1
released_at: "2026-07-13"
release_tag: v0.12.1
release_tag_object: d687a77d911503e67a8d171c00536bd710762bba
release_commit: fedcea4ab17d9b3257c9903636aec0c0fd08f147
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.12.1
prior_released_version: v0.12.0
prior_released_at: "2026-07-10"
# Ground-truth HEADs (updated 2026-07-14 — D-445 STORY-169 DELIVERED: PR #403 ac01d9f2 squash-merged to develop; main=fedcea4 unchanged; develop now 3 unreleased commits ahead (STORY-167+168+169); DRIFT-BACKMERGE-SQUASH-001 still applies)
main_head: fedcea4ab17d9b3257c9903636aec0c0fd08f147
develop_head: ac01d9f2
# Cargo.toml version: main=0.12.1; develop=0.12.1 (3 unreleased commits ac01d9f2 STORY-167+168+169; DRIFT-BACKMERGE-SQUASH-001: main fedcea4 not an ancestor of develop ac01d9f2, histories diverge; trees differ by IEC-104 feature code)
cargo_version_main: "0.12.1"
cargo_version_develop: "0.12.1"
# Open worktrees: main checkout [develop] + .factory [factory-artifacts]. No open release/* or chore/backmerge-* branches.
# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
# Story tracking
stories_delivered: 108
story_index_version: "v3.62"
total_stories: 127
story_index_note: "127 stories / 83 waves / 765 pts. v3.62 (2026-07-14): STORY-170 BC-realigned v2.0 (D-446); no stories_delivered change. See cycles/feature-iec104/ for full F2/F3 history."
# Spec versions (current)
bc_index_version: "v2.28"
vp_index_version: "v2.46"
arch_index_version: "v2.16"
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
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 254 = 246 (dual-margin form). 254 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**STORY-170 v2.0 BC-realigned (D-446, 2026-07-14). Pre-delivery drift corrected (FALSE-POSITIVE + confidence + scope + naming + cot_test AC); ready for code cycle. develop=ac01d9f2; stories_delivered=108; STORY-INDEX v3.62. NEXT: STORY-170 code cycle (wave-79, worktree → stub → test → impl → adversarial → demo → PR). trajectory-tail →0→0→0→0.**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); F4 IN PROGRESS; wave-78 DELIVERED (D-445, 2026-07-14, PR #403); 3 of 8 IEC-104 story-items (E-22) delivered (STORY-167+168+169); STORY-170 v2.0 BC-realigned (D-446); NEXT: STORY-170 code cycle (wave-79) |
| Version | 0.12.1 (released 2026-07-13; main=fedcea4; develop=ac01d9f2 — 3 unreleased commits; DRIFT-BACKMERGE-SQUASH-001) |
| Main HEAD | `fedcea4ab17d9b3257c9903636aec0c0fd08f147` |
| Develop HEAD | `ac01d9f2` — PR #403 STORY-169 squash 2026-07-14; DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 |
| Stories | 108 delivered / 127 total (STORY-INDEX v3.62, dep-graph v3.9, 765 pts) |
| **Last Updated** | 2026-07-14 — STORY-170 BC-realigned v2.0 (D-446); pre-delivery drift corrected (FALSE-POSITIVE + confidence + scope); STORY-INDEX v3.62. trajectory-tail →0→0→0→0 |

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
| feature-iec104 — F4 (delta-implementation) | **IN PROGRESS** | Wave-76 DELIVERED (D-441): STORY-167 PR #401 e65e0d6. Wave-77 DELIVERED (D-443): STORY-168 PR #402 b720fd96. Wave-78 DELIVERED (D-445): STORY-169 PR #403 ac01d9f2; BC-realigned v1.1; 3-clean adversary; CI 13/13; wave-78 gate SATISFIED. STORY-170 v2.0 BC-realigned (D-446). NEXT: STORY-170 code cycle (wave-79). |
| feature-iec104 — F4 per-story adversary pass-4 (STORY-169) | CONVERGED | Trajectory 4→0→0→0; 3-clean streak P2/P3/P4 (BC-5.39.001 SATISFIED); trajectory-tail →0→0→0→0 |
| feature-iec104 — F4 fix burst (STORY-167 P1) | COMPLETE | Pass-1 findings remediated; commit 557b6a8; re-ran P2/P3/P4 clean |
| feature-iec104 — F4 fix burst (STORY-169 P1) | COMPLETE | Pass-1 MEDIUM (stale todo!() docstring) remediated 0debf98; re-ran P2/P3/P4 clean |

---

## Convergence Status

| Cycle/Story | Passes | Trajectory | Status |
|------------|--------|-----------|--------|
| feature-iec104 F4 per-story (STORY-169) | 4 | 4→0→0→0 | CONVERGED 3-clean (BC-5.39.001) |

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | ACTIVE — F4 IN PROGRESS (STORY-170 BC-realigned v2.0, code cycle NEXT); trajectory-tail →0→0→0→0 | develop |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **STORY-170 v2.0 BC-realigned (D-446, 2026-07-14). Pre-delivery drift corrected: FALSE-POSITIVE bug (C_IC/C_CI/C_CS interrogation TypeIDs 100/101/103 were speccing T0827 Possible, BC-2.19.021 says benign/no-finding); AC-170-002 confidence Possible→Likely (BC-2.19.020); reserved-TypeID scope corrected TypeID=0 or [128,255] (BC-2.19.022); AC-170-007 cot_test [TEST]-tagging added (BC-2.19.017 inv1); AsduHeader→Asdu naming; input-hash 7c3c35c (canonical). STORY-INDEX v3.62.** | **BC-REALIGNED (D-446)** | Wave-79 code cycle NEXT: worktree → stub → test → impl → adversarial → demo → PR. |
| **STORY-169 DELIVERED (D-445, 2026-07-14). PR #403 ac01d9f2; CI 13/13; BC-realigned v1.1 (parse_asdu/Asdu, min-6, first_ioa Option); SEC-001 LOW carry-bound deferred STORY-172; AI APPROVE (1 MINOR+1 NIT accepted); adversarial CONVERGED 4 passes streak 3/3 (P2/P3/P4); wave-78 gate SATISFIED. stories_delivered=108; STORY-INDEX v3.61. develop=ac01d9f2 (3 unreleased).** | **DELIVERED (D-445)** | Wave-78 DELIVERED. |
| **STORY-168 DELIVERED (D-443, 2026-07-14). PR #402 b720fd96; CI 13/13; SEC-001-S168 MEDIUM carry-bound deferred STORY-172; AI APPROVE (3 NIT); adversarial CONVERGED 4 passes streak 3/3 (P2/P3/P4); wave-77 gate SATISFIED. stories_delivered=107; STORY-INDEX v3.60. develop=b720fd96 (2 unreleased).** | **DELIVERED (D-443)** | Wave-77 DELIVERED. |
| **STORY-167 DELIVERED (D-441, 2026-07-14). PR #401 e65e0d6; CI 13/13 green; security CLEAN; AI APPROVE (2 MINOR/3 NIT accepted); adversarial CONVERGED 4 passes streak 3/3 (P2/P3/P4); wave-76 gate: single-story wave — per-story 3-clean satisfies wave-level. Demo 7 artifacts, scrub PASS. stories_delivered=106; STORY-INDEX v3.59.** | **DELIVERED (D-441)** | Wave-76 DELIVERED. |
| **feature-iec104 F3 story decomposition DONE (D-440, 2026-07-14). STORY-167..174 (IEC-104 Passive Analyzer E-22, 8 stories, 36 pts, waves 76–83). Serialized: one story/wave due to src/analyzer/iec104.rs file contention. dep-graph v3.9 acyclic (137 edges). STORY-INDEX v3.58 (127 stories/765 pts). Plan gate APPROVED (human).** | **APPROVED (D-440)** | F3 approved. F4 delivery started. |

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
| D-446 | STORY-170 pre-delivery BC-realignment v2.0 (2026-07-14). Corrected significant F3-decomposition drift caught pre-code: (1) AsduHeader→Asdu/extract_asdu_header→parse_asdu naming (STORY-169 delivered broken-out Asdu struct); (2) FALSE-POSITIVE bug fixed — interrogation/clock-sync C_IC/C_CI/C_CS (TypeIDs 100/101/103) were speccing T0827 Possible, BC-2.19.021 says benign/no-finding; (3) AC-170-002 confidence Possible→Likely (BC-2.19.020); (4) reserved-TypeID scope corrected to TypeID=0 or [128,255] (BC-2.19.022); (5) dispatch table corrected (45-47 T1692.001 only; 48-51 +T0836; 105 T0827 Likely; 128-255 T0814); (6) AC-170-007 cot_test [TEST]-tagging added (BC-2.19.017 inv1); (7) BC-2.19.017 added to inputs; input-hash 7c3c35c (canonical; story-writer set d4fcb27 via hook, corrected per PG-HASH-HOOK-DIVERGENCE). STORY-INDEX v3.62. Reinforces F3-DECOMPOSITION-BC-FIDELITY (2nd confirmed occurrence; STORY-169 field-shape/guard drift + STORY-170 false-positive/confidence/scope). Recommend pre-delivery AC↔BC fidelity check for STORY-171-174. | 2026-07-14 |

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
| RETRANSMIT-NS-FALSEPOS-001 | Backwards/retransmitted IEC-104 I-frame N(S) computes large 15-bit gap → possible T1692.001 false-positive. Pre-existing in gap arithmetic (not introduced by first-frame guard). | feature-iec104 F2 gate (D-439, 2026-07-14) | F3 implementer + F4 holdout (STORY-171) |

---

## Active Carry-Forwards

| ID | Summary | Target |
|----|---------|--------|
| ROUTE-BC-DEFER-2026-07-11 | Routes B/C deferred from maint-2026-07-11 (human decision). | Next maintenance run |
| ROUTE-W74-DEFERRED | Code-review 1 NIT deferred from wave-74 gate (human-ratified); joins wave-75 NIT. | Next bin-touch PR |
| PERF-RERUN-001 | AC-149-003 quiescent re-run pending (load avg 52.57 at maint-2026-07-11; human deferred). | Next maintenance run |
| SEC-001 | SEC-001-ENIP (split-borrow) deferred from maint-2026-07-11; next feature wave. | Next feature wave or maintenance |
| SEC-001-S168 | LOW (downgraded from MEDIUM in STORY-169 review): MAX_IEC104_CARRY_BYTES bound not enforced on carry-append path. Inert in STORY-168+169 (no append path yet). | STORY-172 (carry-append path wired there) |
| STORY-166 | E-11, 3 pts, wave-TBD, hash b56924f; S-7.02 carry from wave-75. | Next wave after feature-iec104 F4 |
| F3-handoff cleanup | F-F3P12-002 (STORY-151 pointer note), F-F3P13-002 (STORY-154 frontmatter SS-05), F-F3P17-001 (STORY-154 cross-layer trace). | F4 implementation per-story |
| SEC-001-S158 / SEC-002-S158 | CWE-22 LOW advisories in `bin/lint-cycle-artifact` (deferred until mandatory CI wiring). DF-VALIDATION-001-gated. | bin/lint-cycle-artifact CI wiring |
| PG-REDGREEN-COMMENT-CLEANUP | Stub-era comment/docstring surviving into GREEN delivery — STORY-167 (stale Kani comment Pass-1) + STORY-169 (stale todo!() docstring Pass-1). 2 occurrences. If 3rd occurs (STORY-170+), codify: implementer Red→Green cleanup checklist OR grep guard for `todo!()`/`Body is \`todo!()\`` in doc comments. | cycle-close lessons |
| F3-DECOMPOSITION-BC-FIDELITY | 2 CONFIRMED occurrences: STORY-169 (flat vs broken-out fields; wrong guards) + STORY-170 (false-positive T0827 for C_IC/C_CI/C_CS interrogation; confidence Possible→Likely; reserved-TypeID scope; naming). Both corrected pre-delivery. Recommend: before delivering STORY-171/172/173/174, run pre-delivery AC↔BC field/behavior-fidelity check (story-writer diff of each AC against BC postconditions) — cheap, caught real bugs twice. Consider codifying as F3 gate step at cycle-close. | STORY-171-174 pre-delivery checks + cycle-close lessons |

---

## Session Resume Checkpoint

**STORY-170 v2.0 BC-realigned (D-446, 2026-07-14) — ready for code cycle. develop=ac01d9f2; stories_delivered=108; STORY-INDEX v3.62. NEXT: STORY-170 code cycle (wave-79, worktree → stub → test → impl → adversarial → demo → PR).**

Prior checkpoint (STORY-167+168+169 delivered, 2026-07-14) archived to `cycles/feature-iec104/session-checkpoints.md`.

- **Date:** 2026-07-14. Position: feature-iec104 F4 delta-implementation IN PROGRESS; STORY-170 v2.0 BC-realigned (D-446); pipeline PAUSED pre-wave-79 coding. trajectory-tail →0→0→0→0.
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `ac01d9f2`. DRIFT-BACKMERGE-SQUASH-001 still applies. 3 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2).
- **Wave status:** Wave-76 DELIVERED (D-441): STORY-167. Wave-77 DELIVERED (D-443): STORY-168. Wave-78 DELIVERED (D-445): STORY-169 (BC-realigned v1.1). STORY-170 v2.0 BC-realigned (D-446, pre-delivery).
- **Remaining F4:** STORY-170 (wave-79, control-command detection; v2.0 BC-aligned, ready for coding) through STORY-174 (wave-83). stories_delivered=108.
- **Pre-delivery recommendation:** Run AC↔BC field/behavior-fidelity check for STORY-171/172/173/174 before coding — cheap, caught real bugs in STORY-169 and STORY-170 (F3-DECOMPOSITION-BC-FIDELITY, 2 confirmed occurrences).
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; SEC-001-S168 (LOW, carry-append path inert; STORY-172); STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f).
- **Process-gaps:** PG-REDGREEN-COMMENT-CLEANUP (2 occurrences: STORY-167+169); F3-DECOMPOSITION-BC-FIDELITY (2 confirmed: STORY-169+170; recommend pre-delivery AC↔BC check for STORY-171-174).
- **Spec versions:** BC-INDEX v2.28 / VP-INDEX v2.46 / ARCH-INDEX v2.16 / PRD v1.56 / SS-19 v1.6 / STORY-INDEX v3.62 / dep-graph v3.9 (137 edges).
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
