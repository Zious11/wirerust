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
current_step: "D-457 STORY-173 per-story adversarial CONVERGED 3-clean (BC-5.39.001) 2026-07-16. 14 passes; streak P12/P13/P14; trajectory P1(1H+3doc)→P2(1M+1N)→P3/P4/P5(doc-tense NITs)→P6(CLEAN)→P7(1N stale-cardinality)→P8(4N stale seeded-counts)→P9/P10(CLEAN)→P11(1N non-discriminating test)→P12/P13/P14(CLEAN). Production code FROZEN/CLEAN since P2. Advisory A-12-01 accepted non-blocking. 3 new process-gaps (PG-DOC-CURRENCY-SWEEP/PG-ADVERSARY-IDLE-NO-REPORT/PG-ADVERSARY-SEVERITY-CALIBRATION). STORY-INDEX v3.71. Next: demos → push → PR to develop. trajectory-tail →1→0→0→0"
current_cycle: "feature-iec104"
pipeline: IN PROGRESS
timestamp: 2026-07-16T23:59:01Z
# D-457 STORY-173 per-story adversarial CONVERGED 3-clean (BC-5.39.001) 2026-07-16. 14 passes streak P12/P13/P14. All findings remediated; advisory A-12-01 accepted. 3 process-gaps (PG-DOC-CURRENCY-SWEEP/PG-ADVERSARY-IDLE-NO-REPORT/PG-ADVERSARY-SEVERITY-CALIBRATION). STORY-INDEX v3.71. Next: demos → push → PR.

# Release chain (latest)
released_version: v0.12.1
released_at: "2026-07-13"
release_tag: v0.12.1
release_tag_object: d687a77d911503e67a8d171c00536bd710762bba
release_commit: fedcea4ab17d9b3257c9903636aec0c0fd08f147
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.12.1
prior_released_version: v0.12.0
prior_released_at: "2026-07-10"
# Ground-truth HEADs (updated 2026-07-15 — D-456 STORY-173 pre-delivery fidelity remediation COMPLETE; develop=d64e5fe (STORY-167+168+169+170+171+172, 6 unreleased); DRIFT-BACKMERGE-SQUASH-001 still applies)
main_head: fedcea4ab17d9b3257c9903636aec0c0fd08f147
develop_head: d64e5fe
# Cargo.toml version: main=0.12.1; develop=0.12.1 (6 unreleased commits d64e5fe STORY-167+168+169+170+171+172; DRIFT-BACKMERGE-SQUASH-001: main fedcea4 not an ancestor of develop d64e5fe, histories diverge; trees differ by IEC-104 feature code)
cargo_version_main: "0.12.1"
cargo_version_develop: "0.12.1"
# Open worktrees: main checkout [develop] + .factory [factory-artifacts].
# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
# Story tracking
stories_delivered: 111
story_index_version: "v3.70"
total_stories: 127
story_index_note: "127 stories / 83 waves / 765 pts. v3.70 (2026-07-15): STORY-173 BC-realigned v2.0 pre-delivery D-456 (SR-173-01..08; BC-2.19.028 findings-cap added; hash f3d3673; bc active 377→378). See cycles/feature-iec104/ for full F2/F3 history."
# Spec versions (current)
bc_index_version: "v2.32"
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
  Hard cap (500 lines) margin from soft-target = 500 - 200 = 300; margin from actual = 500 - 275 = 225 (dual-margin form). 275 lines (wc-l).
  Hard cap: 500 lines.
-->

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**D-454 COMPLETE (2026-07-15): STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001). 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6. Findings: F-172-001/002 HIGH remediated D-452; F-172-201 LOW prose precision remediated D-453; F-172-301 NIT stale-citation remediated fec9bfa. Deferred: F-172-003 LOW STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. Spec: BC-2.19.025 v1.3 / BC-INDEX v2.31 / ADR-013 / SS-19 v1.9 / ARCH-INDEX v2.19. Worktree fec9bfa; 2584/0; 26 story_172 tests. develop=1a64380; stories_delivered=110. STORY-INDEX v3.68. Next: demos → push → PR to develop. trajectory-tail →1→0→0→0**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | Feature Mode — feature-iec104 (IEC 60870-5-104, TCP 2404); F4 IN PROGRESS; **wave-80 DELIVERED (D-448); D-454 STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001); 5 of 8 IEC-104 story-items delivered (STORY-167+168+169+170+171); STORY-172 wave-81 adversarial CONVERGED (demos/PR pending)** |
| Version | 0.12.1 (released 2026-07-13; main=fedcea4; develop=1a64380 — 5 unreleased commits; DRIFT-BACKMERGE-SQUASH-001) |
| Main HEAD | `fedcea4ab17d9b3257c9903636aec0c0fd08f147` |
| Develop HEAD | `1a64380` — PR #405 STORY-171 squash 2026-07-15; DRIFT-BACKMERGE-SQUASH-001 |
| Spec versions | BC-INDEX v2.31 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 |
| Stories | 110 delivered / 127 total (STORY-INDEX v3.68, dep-graph v3.9, 765 pts) |
| **Last Updated** | 2026-07-15 — D-454 STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001); 6 passes streak P4/P5/P6; fec9bfa; 2584/0. STORY-INDEX v3.68. Next: demos → push → PR to develop. trajectory-tail →1→0→0→0 |

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
| feature-iec104 — F4 (delta-implementation) | **IN PROGRESS** | Wave-76..80 DELIVERED (D-441/443/445/447/448): STORY-167..171 PRs #401-405. **D-454 STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001); 6 passes streak P4/P5/P6; worktree fec9bfa; 2584/0. Demos/PR next.** 5/8 stories done. trajectory-tail →1→0→0→0 |
| feature-iec104 — D-451 spec-remediation burst | **COMPLETE** | SR-172-01 BLOCKING (FlowId→FlowKey); SR-172-02 MEDIUM (carry-overflow discard-all-new); SR-172-03 MEDIUM (malformed-LEN EMIT-WITH-DEDUP, research-validated). 3rd F3-DECOMPOSITION-BC-FIDELITY — CODIFY-NOW. |
| feature-iec104 — F4 per-story adversary (STORY-172) | **CONVERGED (D-454)** | 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6 (BC-5.39.001 SATISFIED). F-172-001/002 HIGH remediated D-452; F-172-201 LOW remediated D-453; F-172-301 NIT remediated fec9bfa. Deferred: F-172-003 LOW STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. |
| feature-iec104 — F4 per-story adversary pass-4 (STORY-171) | CONVERGED | Trajectory 4 passes, 3-clean P2/P3/P4 (BC-5.39.001 SATISFIED); Pass-1 F-171-001 MEDIUM (stale header, PG-REDGREEN 4th) + F-171-002 (PC-C2 coverage) remediated 27bb678; trajectory-tail →2→0→0→0 |
| feature-iec104 — F4 fix burst (STORY-167 P1) | COMPLETE | Pass-1 findings remediated; commit 557b6a8; re-ran P2/P3/P4 clean |
| feature-iec104 — F4 fix burst (STORY-169 P1) | COMPLETE | Pass-1 MEDIUM (stale todo!() docstring) remediated 0debf98; re-ran P2/P3/P4 clean |
| feature-iec104 — F4 fix burst (STORY-170 P1+P2) | COMPLETE | F-170-001 MEDIUM (CASDU/first_ioa context, BC-2.19.019 PC3/BC-2.19.020 PC2) remediated P1; F-P2-L1 LOW (stale Red-Gate test-module header) remediated P2 |
| feature-iec104 — F4 fix burst (STORY-171 P1) | COMPLETE | F-171-001 MEDIUM (stale header, PG-REDGREEN 4th) + F-171-002 (PC-C2 coverage) remediated 27bb678; re-ran P2/P3/P4 clean |
| feature-iec104 — wave-79 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P3/P4/P5) on STORY-170 diff == wave-level adversarial; CI 13/13 develop 0bd93f8; D-447 |
| feature-iec104 — wave-80 gate | DELIVERED & SATISFIED | Single-story wave; per-story 3-clean (P2/P3/P4) on STORY-171 diff == wave-level adversarial; CI 13/13 develop 1a64380; D-448 |

---

## Convergence Status

| Cycle/Story | Passes | Trajectory | Status |
|------------|--------|-----------|--------|
| feature-iec104 F4 per-story (STORY-171) | 4 | →2→0→0→0 | CONVERGED 3-clean (BC-5.39.001) |
| feature-iec104 F4 per-story (STORY-172) | 6 | →(2H+1L+1N)→1L→1NIT→0→0→0 | CONVERGED 3-clean (BC-5.39.001) streak P4/P5/P6 |
| feature-iec104 F4 per-story (STORY-173) | 14 | →(1H+3doc)→(1M+1N)→NITs→CLEAN(P6)→1N→4N→CLEAN(P9/P10)→1N→CLEAN(P12/P13/P14) | CONVERGED 3-clean (BC-5.39.001) streak P12/P13/P14 |

---

## Concurrent Cycles

| Cycle | Status | Branch |
|-------|--------|--------|
| feature-iec104 | IN PROGRESS — F4 STORY-172 wave-81; D-454 per-story adversarial CONVERGED 3-clean (BC-5.39.001); worktree fec9bfa; demos/PR next | develop |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **D-454 STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-15). 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6. Worktree fec9bfa; 2584/0; 26 story_172 tests. Findings remediated: F-172-001/002 HIGH (D-452); F-172-201 LOW prose precision (D-453); F-172-301 NIT stale-citation (fec9bfa). Deferred: F-172-003 LOW VP-045 vacuity STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. PG-SPEC-VERSION-CITATION-CURRENCY noted (spec-version bumps must include src/ + CHANGELOG in citation-currency set). STORY-INDEX v3.67→v3.68.** | **CONVERGED (D-454)** | Demos/PR next. trajectory-tail →1→0→0→0 |
| **D-453 STORY-172 Pass-2 adversarial → 1L F-172-201 prose precision REMEDIATED (2026-07-15). Entry-check vs residual-after-walk equivalents proved; no observable divergence. Spec: BC-2.19.025 v1.2→v1.3 / BC-INDEX v2.30→v2.31 / SS-19 v1.8→v1.9 / ARCH-INDEX v2.18→v2.19. STORY-172 v3.0→v3.1 hash 938645f. Code unchanged 4dc85c4; 2584/0. STORY-INDEX v3.67. Pass 3 pending.** | **REMEDIATED (D-453)** | trajectory-tail →0→0→4→1 |
| **D-452 STORY-172 Pass-1 adversarial remediation (2026-07-15). F-172-001 HIGH (aggregate carry pre-check = Ptacek/Newsham evasion channel + false-positive source; WALK-FIRST-RESIDUAL-BOUND research-validated per Zeek/Suricata/Snort3 + DNP3 F-B-002 internal precedent + ADR-013 Decision 3) + F-172-002 HIGH (dispatch wiring regression-unguarded; 6 effect tests added) remediated. F-172-003 LOW (VP-045 proptests vacuous) DEFERRED STORY-174. F-172-004 NIT (story_168 stale header) → PG-REDGREEN-SIBLING-SWEEP. Spec chain: BC-2.19.025 v1.1→v1.2 (walk-first residual-bound; vectors i/ii/iii; carry_overflow_reported_c2s/s2c dedup flags; f815431) / BC-INDEX v2.29→v2.30 / ADR-013 Decision 2 rewritten / SS-19 v1.7→v1.8 / ARCH-INDEX v2.17→v2.18. STORY-172 v2.0→v3.0 hash 246add6. Worktree 4dc85c4; 2584/0. STORY-INDEX v3.66.** | **REMEDIATED (D-452)** | trajectory-tail →0→0→0→4 |
| **D-451 pre-STORY-172 spec-remediation burst (2026-07-15). AC↔BC fidelity check DRIFT-FOUND: SR-172-01 BLOCKING (FlowId→FlowKey nonexistent; real type FlowKey); SR-172-02 MEDIUM (carry-overflow discard-all-new semantics per BC-2.19.025 canonical vectors 1+255→1 / 200+100→200); SR-172-03 MEDIUM (BC-2.19.026 PC4 vs ADR-013 contradiction — research-validated EMIT-WITH-DEDUP per CVE-2023-5768/Snort3/Wireshark/Zeek). BC-2.19.027 v1.1 / BC-2.19.026 v1.6 / BC-INDEX v2.29 / ADR-013 reconciled / SS-19 v1.7 / ARCH-INDEX v2.17. STORY-172 v2.0 hash af0f732. 3rd F3-DECOMPOSITION-BC-FIDELITY — CODIFY-NOW. STORY-INDEX v3.65.** | **COMPLETE (D-451)** | Pipeline RESUMED. STORY-172 delivery next (wave-81). trajectory-tail →2→0→0→0 |
| **STORY-171 DELIVERED (D-448, 2026-07-15). PR #405 1a64380; CI 13/13; pre-delivery AC↔BC fidelity check NO drift; adversarial CONVERGED 4 passes streak P2/P3/P4 (BC-5.39.001); F-171-001 MEDIUM (stale header, PG-REDGREEN 4th) + F-171-002 (PC-C2 coverage) remediated 27bb678; security PASS (0 CRIT/HIGH); AI APPROVE (1 cycle, 0 blocking); 166 iec104 tests; demo 8 artifacts scrub PASS; worktree+branch cleaned. RETRANSMIT-NS-FALSEPOS-001 resolved (EC-007 fail-closed). PG-REDGREEN-COMMENT-CLEANUP: 4th occurrence — CODIFY-NOW. 5th of 8 IEC-104 stories. stories_delivered=110; STORY-INDEX v3.64. develop=1a64380.** | **DELIVERED (D-448)** | Wave-80 DELIVERED & gate-satisfied. |

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
| D-447 | STORY-170 DELIVERED (PR #404 0bd93f8, 2026-07-15, human-authorized). Story BC-realigned v2.0 pre-impl (F3-drift: FALSE-POSITIVE interrogation→T0827 bug fixed per BC-2.19.021; T0827 Likely; cot_test [TEST]; reserved-scope). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 5 passes, streak P3/P4/P5; F-170-001 MEDIUM (CASDU/first_ioa context, BC-2.19.019 PC3/BC-2.19.020 PC2) + F-P2-L1 LOW (stale Red-Gate test-header) remediated. Security CLEAN (0 CRIT/HIGH/MEDIUM; 4 INFO accepted); AI APPROVE (0 blocking, 2 NIT accepted); CI 13/13; 136 iec104 tests; demo 6 artifacts scrub PASS; worktree+branch cleaned. 4th of 8 IEC-104 stories. develop=0bd93f8; stories_delivered=109; STORY-INDEX v3.63. PG-REDGREEN-COMMENT-CLEANUP: 3rd occurrence — READY-TO-CODIFY (codification obligation: CI/pre-commit grep guard for stale Red-Gate phrases OR implementer cleanup checklist). | 2026-07-15 |
| D-448 | STORY-171 DELIVERED (PR #405 1a64380, 2026-07-15, human-authorized). Pre-delivery AC↔BC fidelity check: NO drift (Option<u16> first-frame model already BC-faithful). Per-story adversarial CONVERGED 3-clean (BC-5.39.001): 4 passes, streak P2/P3/P4; Pass-1 F-171-001 MEDIUM (stale header, PG-REDGREEN 4th occurrence) + F-171-002 (PC-C2 coverage) remediated 27bb678. Security PASS (0 CRIT/HIGH); AI APPROVE (1 cycle, 0 blocking); CI 13/13; 166 iec104 tests; demo 8 artifacts scrub PASS; worktree+branch cleaned. 5th of 8 IEC-104 stories. RETRANSMIT-NS-FALSEPOS-001 resolved (documented EC-007, fail-closed per INV-3). develop=1a64380; stories_delivered=110; STORY-INDEX v3.64. PG-REDGREEN-COMMENT-CLEANUP: 4th occurrence — CODIFY-NOW (strongly recommend CI/pre-commit grep-guard as STORY-174 hardening item or E-11 governance follow-up; 4 confirmed recurrences: STORY-167/169/170/171). | 2026-07-15 |
| D-449 | Session /wrap (human-requested, 2026-07-15) — pipeline PAUSED at feature-iec104 F4, 5/8 stories delivered (STORY-167..171); develop=1a64380 (5 unreleased); .factory tree clean (sidecar-learning.md committed in wrap commit); docs/adr/0013 untracked-on-develop flagged for STORY-173/docs-commit (recoverable from factory-artifacts mirror); no in-flight TDD, no open PRs, no story worktrees. input-hash scan MATCH=125 STALE=2 (STORY-164/165 pre-existing PG-HASH-HOOK-DIVERGENCE noise, non-blocking). Resume: /vsdd-factory:next-step → STORY-172. | 2026-07-15 |
| D-450 | Post-merge late-review findings captured (PR #404/#405 already merged; verdicts corroborated APPROVE+PASS+CI-13/13). 3 deferred items recorded: IEC104-TIMED-CMD-GAP-001 (timed control TypeIDs 58–64 detection gap, sec-review-170 L-001, follow-on story, DF-VALIDATION-001-gated), IEC104-FINDINGS-CAP-001 (unbounded findings Vec, sec-review-170 M-001, → STORY-173 dispatcher cap), IEC104-FINDING-DIRECTION-001 (Finding.direction None though known, pr-review-171 MINOR-2, → STORY-172/173 cleanup). Pipeline remains PAUSED (wrap). | 2026-07-15 |
| D-451 | Pre-STORY-172 spec-remediation burst COMPLETE (2026-07-15, pipeline RESUMED). AC↔BC fidelity check DRIFT-FOUND — 3rd F3-DECOMPOSITION-BC-FIDELITY occurrence: SR-172-01 BLOCKING (FlowId→FlowKey; nonexistent type in original story); SR-172-02 MEDIUM (carry-overflow discard-all-new canonical vectors 1+255→1, 200+100→200; per BC-2.19.025); SR-172-03 MEDIUM (BC-2.19.026 PC4 vs ADR-013 contradiction — research-validated EMIT-WITH-DEDUP; evidence: CVE-2023-5768 malformed-APDU-length DoS, Snort3 IEC104_BAD_LENGTH, Wireshark iec104.apdu_invalid_len, Zeek weird sampling; report: `.factory/cycles/feature-iec104/research/sr-172-03-malformed-len-validation.md`). SEC-001-S168 coverage CONFIRMED. Human decisions: EMIT-WITH-DEDUP ratified; sibling on_data signature (flow_key, data, ts, direction) adopted. Spec updates: BC-2.19.027 v1.0→v1.1 (FlowId→FlowKey); BC-2.19.026 v1.5→v1.6 (PC4 EMIT-WITH-DEDUP + inv5 + EC-006/007/008); BC-INDEX v2.28→v2.29; ADR-013 Decision 3 steps 3–4 reconciled; SS-19 v1.6→v1.7; ARCH-INDEX v2.16→v2.17. STORY-172 realigned v2.0: FlowId→FlowKey; carry-overflow discard-all-new; AC-172-004 EMIT-WITH-DEDUP + AC-172-008 (3 dedup tests); Iec104FlowState 7 fields; inputs 5→12; input-hash af0f732 (canonical Python tool). STORY-INDEX v3.64→v3.65. F3-DECOMPOSITION-BC-FIDELITY: CODIFY-NOW (3 confirmed occurrences; vehicle: cycle-close or E-11 governance follow-up). | 2026-07-15 |
| D-452 | STORY-172 Pass-1 adversarial remediation COMPLETE (2026-07-15). Pass-1 findings: F-172-001 HIGH (aggregate carry pre-check = Ptacek/Newsham evasion channel + false-positive source; genuine BC-2.19.025 vs BC-2.19.026 PC1 spec contradiction) + F-172-002 HIGH (dispatch wiring regression-unguarded; no effect assertions) + F-172-003 LOW (VP-045 proptests vacuous; DEFERRED STORY-174) + F-172-004 NIT (story_168 stale header; joins PG-REDGREEN-SIBLING-SWEEP). Research-validated: WALK-FIRST-RESIDUAL-BOUND (Zeek/Suricata/Snort3 parse-what-fits precedent; DNP3 F-B-002 internal precedent; ADR-013 Decision 3 already walk-first). Spec chain: BC-2.19.025 v1.1→v1.2 (walk-first residual-bound; new canonical vectors i/ii/iii replacing defective 1+255→1/200+100→200; carry_overflow_reported_c2s/s2c separate dedup flags; input-hash f815431) / BC-INDEX v2.29→v2.30 / ADR-013 Decision 2 rewritten / SS-19 v1.7→v1.8 / ARCH-INDEX v2.17→v2.18. STORY-172 v2.0→v3.0 hash 246add6. Code: b90a834 (F-172-002 dispatch tests) → afa6d14 (DF-SIBLING-SWEEP-001 strengthened) → 45b9384 (Red: 3 v1.2 vectors) → 4dc85c4 (Green: walk-first impl + CHANGELOG); 2584/0; clippy/fmt clean. STORY-INDEX v3.65→v3.66. Convergence: Pass 1 →2H+1L+1N; Pass 2 pending (streak target 3-clean). | 2026-07-15 |
| D-453 | STORY-172 Pass-2 adversarial → 1L F-172-201 prose precision REMEDIATED (2026-07-15). Entry-check vs residual-after-walk: equivalents proved; no observable divergence. Spec chain: BC-2.19.025 v1.2→v1.3 (PC-3/Inv-2 entry-check formulation) / BC-INDEX v2.30→v2.31 / SS-19 v1.8→v1.9 / ARCH-INDEX v2.18→v2.19. STORY-172 v3.0→v3.1 hash 938645f. Code unchanged 4dc85c4; 2584/0. STORY-INDEX v3.66→v3.67. Pass 3 pending (streak target 3-clean). | 2026-07-15 |
| D-454 | STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-15). 6 passes total; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6. Worktree HEAD fec9bfa; 2584/0; 26 story_172 tests. Findings remediated: F-172-001 HIGH walk-first (D-452); F-172-002 HIGH dispatch assertions (D-452); F-172-201 LOW prose precision (D-453); F-172-301 NIT stale-citation (fec9bfa). Deferred: F-172-003 LOW (VP-045 proptest vacuity → STORY-174); F-172-004 NIT (pre-existing story_168 header → PG-REDGREEN-SIBLING-SWEEP wave-gate). Process-gap noted: PG-SPEC-VERSION-CITATION-CURRENCY (F-172-301 — spec-version bumps must include src/ comments + CHANGELOG in citation-currency set; also: adversary idle-without-report recurring behavior). STORY-INDEX v3.67→v3.68. Next: demos → push → PR to develop. | 2026-07-15 |
| D-455 | STORY-172 DELIVERED (PR #406 d64e5fe squash-merged to develop, 2026-07-15, human-authorized per DF-MERGE-AUTH-CLASSIFIER-001). Per-story adversarial CONVERGED 3-clean (D-454). Security PASS — SEC-001-S168 carry-bound FULLY MITIGATED (deferred finding closed). pr-reviewer APPROVE (F1 direction/timestamp=None deferred; F2 unreachable-guard accepted by design). CI 13/13. Demos 9 artifacts/8 ACs scrub PASS. ADR-0013 committed to develop (c5b098f) — UNTRACKED-DEVELOP-ADR RESOLVED. develop=d64e5fe (6 unreleased: STORY-167..172); stories_delivered 110→111. Wave-81 gate SATISFIED. STORY-INDEX v3.68→v3.69. | 2026-07-15 |
| D-456 | STORY-173 pre-delivery AC↔BC fidelity check DRIFT-FOUND (2 BLOCKING/3 MEDIUM/3 LOW) — 4th F3-DECOMPOSITION-BC-FIDELITY (story-decomposition imprecision; ADR-013/code were correct). SR-173-01 BLOCKING (T0881 tactic string "impact" → MitreTactic::IcsInhibitResponseFunction, would not compile); SR-173-02 BLOCKING security GAP (IEC104-FINDINGS-CAP-001 uncovered though assigned here). Remediation: product-owner created BC-2.19.028 v1.0 (per-session findings cap MAX_IEC104_FINDINGS=10_000 mirroring DNP3 BC-2.15.022/ENIP BC-2.17.022; dropped_findings counter; anchor PC-2); BC-INDEX v2.31→v2.32. STORY-173 realigned to v2.0 (SR-173-01..08; AC-173-007 cap + AC-173-008 dispatcher wiring; input-hash f3d3673). STORY-INDEX v3.69→v3.70. | 2026-07-15 |
| D-457 | STORY-173 per-story adversarial CONVERGED 3-clean (BC-5.39.001) (2026-07-16). 14 passes; streak P12/P13/P14; final HEAD 7b2a73e; 2602/0. Trajectory P1(1H F-173-001 + 3 doc)→P2(1M+1N)→P3/P4/P5(doc-tense NITs)→P6 CLEAN→P7(1N stale protocols.rs cardinality)→P8(4N stale mitre.rs seeded-count)→P9/P10 CLEAN→P11(1N non-discriminating EMITTED_IDS test)→P12/P13/P14 CLEAN. Code FROZEN/CLEAN since P2; post-P2 tail = doc-accuracy + test-cosmetic reviewer-variance (whole-src doc sweeps + severity calibration at P12). Fix commits 11f695c/366b176/6a3a372/a652464/b4cca90/7462e9c/5363be6/a73a3b9/f6b91f1/7b2a73e; demo 3d22003. Advisory A-12-01 accepted. Process-gaps: PG-DOC-CURRENCY-SWEEP, PG-ADVERSARY-IDLE-NO-REPORT, PG-ADVERSARY-SEVERITY-CALIBRATION, PG-STATE-RECOVERY-SCOPE (→ cycle-close). STORY-INDEX v3.70→v3.71. | 2026-07-16 |

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
| SEC-001-S168 | LOW (downgraded from MEDIUM in STORY-169 review): MAX_IEC104_CARRY_BYTES bound not enforced on carry-append path. Inert in STORY-168+169+171 (no append path yet). | STORY-172 (carry-append path wired there) |
| STORY-166 | E-11, 3 pts, wave-TBD, hash b56924f; S-7.02 carry from wave-75. | Next wave after feature-iec104 F4 |
| F3-handoff cleanup | F-F3P12-002 (STORY-151 pointer note), F-F3P13-002 (STORY-154 frontmatter SS-05), F-F3P17-001 (STORY-154 cross-layer trace). | F4 implementation per-story |
| SEC-001-S158 / SEC-002-S158 | CWE-22 LOW advisories in `bin/lint-cycle-artifact` (deferred until mandatory CI wiring). DF-VALIDATION-001-gated. | bin/lint-cycle-artifact CI wiring |
| PG-REDGREEN-COMMENT-CLEANUP | Stub-era Red-Gate phrase surviving into GREEN delivery: STORY-167 (stale Kani comment P1) + STORY-169 (stale todo!() docstring P1) + STORY-170 (stale Red-Gate test-module header F-P2-L1) + **STORY-171 (stale header F-171-001 Pass-1)**. **4 occurrences — CODIFY-NOW.** Codification: add CI/pre-commit grep guard that FAILS if implemented function or test module contains stale Red-Gate phrases ("MUST FAIL", "Body is `todo!()`", "is a todo!() stub", "Therefore: `todo!()`"). Candidate vehicle: STORY-174 hardening wave or dedicated E-11 governance follow-up. PG-REDGREEN-SIBLING-SWEEP (sibling headers in STORY-168/169) still queued. | STORY-174 hardening or E-11 governance follow-up |
| PG-REDGREEN-SIBLING-SWEEP | Already-merged baseline stale Red-Gate test-module headers: `tests/iec104_analyzer_tests.rs` mod story_168 (~L662-663, L1498-1499) + mod story_169 (~L1544) contain false "MUST FAIL/todo!() stub" comments. Inert at runtime but confusing to reviewers. F-172-004 NIT (story_168 stale header) joins this item. Sweep at the feature wave-gate or a maintenance touch. | feature-iec104 wave-gate or next maintenance |
| F3-DECOMPOSITION-BC-FIDELITY | **3 CONFIRMED occurrences: STORY-169** (flat vs broken-out fields; wrong guards) **+ STORY-170** (false-positive T0827; confidence Possible→Likely; reserved-TypeID scope; naming) **+ STORY-172** (FlowId→FlowKey nonexistent; carry-overflow discard-all-new semantics; malformed-LEN PC4 contradiction). All corrected pre-delivery. **CODIFY-NOW (D-451).** Codification: mandatory pre-delivery AC↔BC fidelity check as F3/F4 gate step. Vehicle: cycle-close or E-11 governance follow-up. | STORY-173-174 pre-delivery checks + cycle-close codification |
| F-172-003 | VP-045 proptests vacuity (STORY-172 Pass-1 F-172-003 LOW): carrier loop covers no meaningful shrinkage paths; proptest framework calls without domain generators. DEFERRED — STORY-174 formal hardening target. Advisory-LOW; does not block Pass 2. | STORY-174 (formal hardening wave) |
| IEC104-TIMED-CMD-GAP-001 | (DETECTION GAP, security-relevant, DEFERRED) TypeIDs 58–64 (timed control variants C_SC_TA_1=58, C_DC_TA_1=59, C_RC_TA_1=60, C_SE_TA_1=61, C_SE_TB_1=62, C_SE_TC_1=63, C_BO_TA_1=64 — time-tagged equivalents of 45–51) fall into detect_iec104_threats `_` silent arm; no T1692.001/T0836 findings emitted. Out of scope per BC-2.19.019 (lists only 45–51); NOT a STORY-170 defect. Evasion gap: control commands via timed variants bypass detection. Source: sec-review-170 L-001 (PR #404). DF-VALIDATION-001 required before filing any GitHub issue (research-agent must validate on current develop first). | Follow-on detection story (new BC + detection arm for TypeIDs 58–64, or feature-cycle extension) |
| IEC104-FINDINGS-CAP-001 | (CWE-400/770, DEFERRED to STORY-173) detect_iec104_threats pushes into an unbounded &mut Vec<Finding> with no cap. Function is O(1)/call but the dispatcher must enforce a per-session/per-flow findings cap before wiring detect_iec104_threats into live on_data. Source: sec-review-170 M-001 (PR #404). Related to SEC-001-S168 carry-bound theme. | STORY-173 (dispatch integration): add findings-cap enforcement + document cardinality bound in fn doc comment |
| IEC104-FINDING-DIRECTION-001 | (code-quality, MINOR, DEFERRED) track_ns_desync (STORY-171) leaves Finding.direction = None while direction IS known (formats direction into evidence string instead). Finding.direction: Option<Direction> exists per LESSON-P2.08 for JSON consumers to distinguish client/server anomalies. Populating Some(direction) is more idiomatic. Source: pr-review-171 MINOR-2 (PR #405). | STORY-172/173 or maintenance touch. Consider whether detect_iec104_threats / process_u_frame findings should carry direction once dispatcher provides it (STORY-173). |
| PG-SPEC-VERSION-CITATION-CURRENCY | Spec-version bumps must include src/ comments and CHANGELOG entries in the citation-currency sweep set (surfaced by F-172-301 NIT, D-454). Recurring adversary idle-without-report behavior also flagged for lessons ledger. | cycle-close lessons codification |

---

## Session Resume Checkpoint

**D-454 COMPLETE (2026-07-15): STORY-172 per-story adversarial CONVERGED 3-clean (BC-5.39.001). 6 passes; trajectory →(2H+1L+1N)→1L→1NIT→0→0→0; streak P4/P5/P6. Findings: F-172-001/002 HIGH remediated D-452; F-172-201 LOW prose precision remediated D-453; F-172-301 NIT stale-citation remediated fec9bfa. Deferred: F-172-003 LOW STORY-174; F-172-004 NIT PG-REDGREEN-SIBLING-SWEEP. Spec: BC-2.19.025 v1.3 / BC-INDEX v2.31 / ADR-013 / SS-19 v1.9 / ARCH-INDEX v2.19. Worktree fec9bfa; 2584/0; 26 story_172 tests. develop=1a64380; stories_delivered=110. STORY-INDEX v3.68. Next: demos → push → PR to develop. trajectory-tail →1→0→0→0**

Prior checkpoint (D-453 STORY-172 Pass-2 remediation, 2026-07-15) archived to `cycles/feature-iec104/session-checkpoints.md`.

- **Date:** 2026-07-15. Position: feature-iec104 F4 delta-implementation IN PROGRESS; D-454 per-story adversarial CONVERGED 3-clean; STORY-172 v3.1 hash 938645f; worktree fec9bfa. Demos/PR next. trajectory-tail →1→0→0→0
- **Ground truth (source):** main = `fedcea4ab17d9b3257c9903636aec0c0fd08f147`; develop = `1a64380`. DRIFT-BACKMERGE-SQUASH-001 still applies. 5 unreleased commits: STORY-167 (PR #401 e65e0d6) + STORY-168 (PR #402 b720fd96) + STORY-169 (PR #403 ac01d9f2) + STORY-170 (PR #404 0bd93f8) + STORY-171 (PR #405 1a64380). STORY-172 in worktree fec9bfa (adversarial CONVERGED; not yet merged).
- **Wave status:** Wave-76 DELIVERED (D-441): STORY-167. Wave-77 DELIVERED (D-443): STORY-168. Wave-78 DELIVERED (D-445): STORY-169. Wave-79 DELIVERED (D-447): STORY-170. Wave-80 DELIVERED (D-448): STORY-171. Wave-81 IN PROGRESS: STORY-172 (TDD complete; adversarial CONVERGED 3-clean D-454; worktree fec9bfa; demos/PR pending). 5 of 8 IEC-104 stories merged to develop.
- **Remaining delivery sequence:** STORY-172 (wave-81, carry buffers + frame-walk + on_flow_close; v3.1 realigned D-453; adversarial CONVERGED D-454; **SEC-001-S168 carry bound enforced here**; demos → push → PR next) → STORY-173 (wave-82, dispatch integration + T0881 six-part atomic + Rule 8 + --iec104 + SUPPORTED_PORTS; **land docs/adr/0013**; **IEC104-FINDINGS-CAP-001 lands here**) → STORY-174 (wave-83, formal hardening + PG-REDGREEN-COMMENT-CLEANUP grep-guard + **F-172-003 VP-045 proptest fix**) → F5 → F6 → F7 → release cut. stories_delivered=110.
- **Pre-delivery recommendation:** STORY-172 demos next, then push branch and open PR. Run AC↔BC check for STORY-173/174 before coding.
- **Carry-forwards:** ROUTE-BC-DEFER-2026-07-11; ROUTE-W74-DEFERRED; PERF-RERUN-001; SEC-001; SEC-001-S168 (LOW, STORY-172 carry-append path); STORY-166 (E-11, 3 pts, wave-TBD, hash b56924f); **STORY-164/165 input-hash STALE=2** (pre-existing PG-HASH-HOOK-DIVERGENCE noise, non-blocking); **F-172-003** (VP-045 vacuity → STORY-174); **IEC104-TIMED-CMD-GAP-001** (timed control TypeIDs 58–64 detection gap, DF-VALIDATION-001-gated); **IEC104-FINDINGS-CAP-001** (unbounded findings Vec → STORY-173); **IEC104-FINDING-DIRECTION-001** (Finding.direction None → STORY-172/173); **PG-SPEC-VERSION-CITATION-CURRENCY** (spec-version bumps must include src/ comments + CHANGELOG in citation-currency sweep → cycle-close codification).
- **Process-gaps CODIFY-NOW:** PG-REDGREEN-COMMENT-CLEANUP (4 occurrences: STORY-167+169+170+171; CODIFY-NOW per D-448; candidate: STORY-174 or E-11); PG-REDGREEN-SIBLING-SWEEP (stale headers in STORY-168+169 + F-172-004 NIT; sweep at feature wave-gate); **F3-DECOMPOSITION-BC-FIDELITY (3 confirmed: STORY-169+170+172; CODIFY-NOW per D-451; mandatory pre-delivery AC↔BC check as F3/F4 gate step)**; **PG-SPEC-VERSION-CITATION-CURRENCY (F-172-301 — spec-version bumps must include src/ comments + CHANGELOG in citation-currency set; target: cycle-close codification)**.
- **UNTRACKED-DEVELOP-ADR:** `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md` is UNTRACKED on the develop working tree. **ACTION:** commit docs/adr/0013 to develop via STORY-173 PR or a dedicated `docs(adr): add ADR-0013` commit. Do NOT commit directly to develop outside a PR.
- **Spec versions:** BC-INDEX v2.31 / VP-INDEX v2.46 / ARCH-INDEX v2.19 / PRD v1.56 / SS-19 v1.9 / STORY-INDEX v3.68 / dep-graph v3.9 (137 edges).
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
