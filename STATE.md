---
pipeline: PHASE_3_TDD_IMPLEMENTATION
phase: phase-3-tdd-implementation
product: wirerust
mode: brownfield
timestamp: 2026-05-22T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_started: "2026-05-21"
develop_head: 41ab24d
current_cycle: v0.1.0-greenfield-spec
current_wave: 22
wave_21_status: CLOSED
wave_21_closed: "2026-05-30"
wave_21_stories: "STORY-077 (TerminalReporter, 8pts, BC-2.11.006..012) + STORY-079 (CsvReporter, 5pts, BC-2.11.020..022)"
wave_21_points: 13
wave_21_prs: "#158 (STORY-077 → 594567c), #159 (STORY-079 → 41ab24d)"
wave_21_per_story_convergence: "STORY-077: 3ps-3clean(P1/P2/P3); STORY-079: 3-clean P11/P12/P13 (13 passes; all DIRTY were spec-side citation/proof-method drift — CRLF→LF BC-2.11.020, VP-020 manual→unit, proptest→unit family sweep, test-file citation, EC-002→EC-004; test artifact clean from P2). BC-5.39.001 per-story ACHIEVED both."
wave_21_wave_level_convergence: "3-lens fresh-context (consistency/integration-static/traceability). R1 DIRTY(2HIGH VP-proof-method + 1MED FSR-citation)→remediated; R2 integration-static+traceability CLEAN, consistency DIRTY(1MED CAP-11 casing)→remediated; R3 consistency CLEAN. Net 3/3 lenses CLEAN. BC-5.39.001 ACHIEVED. Notable: wave-level pass surfaced SS-11 reporter VP proof-method family-harmonization (VP-012/016/017 swept to match VP-020 correction) — sibling-sweep completion."
wave_21_delivery: "Both PRs squash-merged → develop 41ab24d 2026-05-30; brownfield-formalization, ZERO src changes; 27 reporter tests (14 terminal + 13 csv); full suite 942 passed/0 failed; all 8 CI green; security CLEAN; both pr-reviewer APPROVED; worktrees + branches removed; demo evidence docs/demo-evidence/STORY-077|079/; first multi-story reporter wave (E-8/SS-11)."
wave_21_pg_enforcement: "[PG-W17-001] AC-test-name-sync clean both stories; [DF-TEST-NAMESPACE-001] mod story_077/story_079 applied; [DF-ADVERSARY-CHECKOUT-GUARD-001] content-based guard used throughout; [DF-SIBLING-SWEEP-001] SS-11 reporter VP family harmonized."
wave_20_status: CLOSED
wave_20_started: "2026-05-29"
wave_20_closed: "2026-05-29"
wave_20_stories: STORY-076 (E-8 reporter, SS-11, 5pts, reporter/json; BC-2.11.001..005; JsonReporter structure/skipped_packets/RFC-8259 byte handling)
wave_20_points: 5
wave_20_prs: "#157 (STORY-076 → e5cb2b1)"
wave_20_per_story_convergence: "STORY-076: 5 passes; 3/3 clean streak on passes 3/4/5 (BC-5.39.001 ACHIEVED). Trajectory: P1-DIRTY(1HIGH-DEL-non-escape+2MED-Cyrillic/C1+2LOW)→P2-DIRTY(1MED-over-broad-\\u04-guard-self-inflicted+1LOW)→P3/P4/P5-CLEAN(0). All test-level; resolved via discriminating escaped-form-absence assertions scoped to fixture codepoints. Frozen d7c4a91→merged e5cb2b1."
wave_20_wave_level_convergence: "single-story wave; per-story convergence == wave-level convergence per BC-5.39.001"
wave_20_delivery: "PR #157 squash-merged → e5cb2b1 2026-05-29; brownfield-formalization, ZERO src changes; 40 reporter_json_tests green; full suite 915 passed/0 failed; all 8 CI green; security CLEAN; pr-reviewer APPROVED 1 cycle (1 non-blocking NIT); worktree + branch removed; demo evidence docs/demo-evidence/STORY-076/; VP-017 deferred to Phase-6 (proptest)"
wave_20_pg_enforcement: "[PG-W17-001] AC-test-name-sync verified both directions across all 5 adversarial passes; clean. First reporter-subsystem (SS-11) story; opened E-8 epic."
stories_delivered: 41
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "Full Phase-1→W21 trajectory archived in cycles/phase-3-tdd/convergence-trajectory.md. Latest: ...W20-S076-DELIVERED(PR#157→e5cb2b1;2026-05-29)|W20-CONVERGED-CLOSED-2026-05-29|W21-S077-story:3ps-3clean(P1/P2/P3;zero-findings;terminal-C1-escaping)|W21-S077-DELIVERED(PR#158→594567c)|W21-S079-story:3-clean-P11/P12/P13(13ps;spec-side-drift-cascade-rem;csv-injection)|W21-S079-DELIVERED(PR#159→41ab24d)|W21-WAVE-3lens:R1-DIRTY(VP-method+FSR)→R2-2clean+1dirty(casing)→R3-CLEAN;3/3-lenses-CLEAN;SS-11-VP-family-harmonized|W21-CONVERGED-CLOSED-2026-05-30"
consistency_audit: CONSISTENT
input_drift_check: CLEAN (Wave-20 STORY-076 test-only formalization; zero src/production changes; reporter/json subsystem — no holdout-scenario hash impact; Wave-19 story-citation/AC-sync bump may apply — verify at Phase-4 entry)
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
wave_history_archived: "cycles/phase-3-tdd/wave-history.md (waves 1-18 detail fields; extracted 2026-05-29)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-21 CLOSED/CONVERGED; 41 stories delivered. NEXT: Wave 22.
41 stories delivered (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020/017/018/021/031/032/033/041/051/042/043/044/052/045/053/055/046/054/056/058/057/076/077/079).
Wave 21 CLOSED 2026-05-30 — STORY-077 (PR #158→594567c, 3ps-3clean P1/P2/P3) + STORY-079 (PR #159→41ab24d, 13ps 3-clean P11/P12/P13). Both BC-5.39.001 ACHIEVED. Wave-level 3/3 lenses CLEAN. SS-11 VP family harmonized (VP-012/016/017).
develop HEAD: 41ab24d (PR #159 squash-merged 2026-05-30; cargo test --all-targets 942 passed/0 failed). All 8 CI checks green.

**Mode:** brownfield (in-repo: target == reference).

**Test suite:** passing on develop (Wave 8 stories delivered). `cargo fmt --check`,
`cargo clippy`, `cargo test --all-targets` all green. CI: 8 checks (semantic-pr, test, clippy, fmt, fuzz-build, audit, deny, trust-boundary; `fuzz-build` pinned `nightly-2026-05-21` + `cargo-fuzz 0.13.1` + `timeout-minutes: 25` after PR #111 hotfix; `trust-boundary` added PR #148;
the nightly pin is a deliberate periodic-maintenance item — do NOT enable automated
dependency bumping for it).

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements; 33 adversary passes; trajectory: `17→…→0→0→0` (detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md) |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 48 stories / 10 epics / 27 waves / 100 holdout scenarios / 282 points; story-adversary 3/3 (10 passes) SATISFIED; input-hash drift CLEAN (153/153) |
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-21 CLOSED/CONVERGED (41 stories; develop HEAD 41ab24d; 942 tests green); NEXT: Wave 22 | W21 wave-level 3/3 lenses CLEAN; BC-5.39.001 ACHIEVED both stories; SS-11 VP family harmonized |
| Phase 4 — Holdout Evaluation | NOT STARTED | — |
| Phase 5 — Adversarial Refinement | NOT STARTED | — |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 3 — Current Wave Status

| Wave | Stories | Status | develop HEAD at Close | Notes |
|------|---------|--------|----------------------|-------|
| 1 | STORY-001, STORY-069 | CLOSED/CONVERGED | b7424b7 | 329 tests |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | CLOSED/CONVERGED | 3b2481c | 376 tests; fuzz-build CI |
| 3 | STORY-071, STORY-005 | CLOSED/CONVERGED | f0b5007 | CI hotfix #112; chore #115 |
| 4 | STORY-011, STORY-066 | CLOSED/CONVERGED | f628c33 | 394 tests |
| 5 | STORY-012 | **CLOSED/CONVERGED** | bbddac6 | 415 tests; 3/3 clean wave-level passes |
| 6 | STORY-013 | **CLOSED/CONVERGED** | 3e705b5 | PR #119 squash-merged 2026-05-22; 31 BC tests; per-story 3/3 clean; wave-level 3/3 CLEAN (ZERO findings) |
| 7 | STORY-014 | **CLOSED/CONVERGED** | b23c6d3 | PR #120 squash-merged 2026-05-25; 17 tests + 2 doc(hidden) seams; ADR-0004 amended PR #121; per-story 8 passes 3/3 clean streak; wave-level 8 passes 3/3 clean streak |
| 8 | STORY-019, STORY-015 | **CLOSED/CONVERGED** | 4b9b85f | PR #122 (STORY-019) + PR #123 (STORY-015) squash-merged 2026-05-26; ADR-0004 v2 PRs #124/#125/#126; per-story 8 passes each (3/3 clean); wave-level 9 passes 3/3 clean streak; 4 drift items logged |
| 9 | STORY-016, STORY-020 | **CLOSED/CONVERGED** 2026-05-26 | e237747 | PR #127 (STORY-016, 24 tests+1 proptest) + PR #128 (STORY-020, 25 tests+1 proptest+1 seam) + PR #129 + PR #130 (wave-followup-1/2); per-story 14 passes total (S016: 6; S020: 8); wave-level 6 passes (DIRTY×3+CLEAN×3); 11 findings remediated; W9-D8 CRITICAL; 632 tests passing |
| 10 | STORY-017, STORY-018 | **CLOSED/CONVERGED** 2026-05-27 | 211143e (PR #133 — wave-level fix) | STORY-017 MERGED PR #131 (4 passes 1D+3C; 24 tests + 9 ECs). STORY-018 MERGED PR #132 (9 passes 6D+3C; resource bounds). Wave-level 4 passes (1D+3C; 3 findings remediated + 6 deferred). 17 adversarial passes total (15% reduction vs Wave 9: 20). |
| 11 | STORY-021 | **CLOSED/CONVERGED** 2026-05-27 | 3cd3000 (PR #134) | STORY-021 MERGED PR #134 (11 passes; 9-10-11 CLEAN per BC-5.39.001). Brownfield-formalization: +88/+33/+33/+1290 lines, 4 files, 203 new tests. BC pre-merge re-anchor doctrine adopted (W11.L1). Methodology bug caught (W11.L2). 4 process-gap codifications applied. |
| 12 | STORY-031 | **CLOSED/CONVERGED** 2026-05-27 | 1435362 (PR #135) | STORY-031 MERGED PR #135 (brownfield-formalization: tests/dispatcher_tests.rs only; 22 tests; 9 passes, 7-8-9 CLEAN per BC-5.39.001). Anchor-completeness EC-scenario-match sub-rule discovered (W12.L1). 2 process-gap codifications applied to policies.yaml. |
| 13 | STORY-032 | **CLOSED/CONVERGED** 2026-05-27 | 0d9b16d (PR #136) | STORY-032 MERGED PR #136 (brownfield-formalization: tests/dispatcher_tests.rs only; +444/-0 lines, 27 tests; 5 passes, 3-4-5 CLEAN per BC-5.39.001). 44% fewer passes than W12. Zero src/ changes; indirect observability throughout. 4 lessons recorded (W13.L1-L4); 0 new codifications. |
| 14 | STORY-033 | **CLOSED/CONVERGED** 2026-05-28 | 30cd4a6 (PR #137) | STORY-033 MERGED PR #137 (brownfield-formalization: tests/dispatcher_tests.rs +367/-0 lines; src/analyzer/http.rs +12, src/analyzer/tls.rs +12 additive seams; 6 new BC-prefixed tests, 33 total; 4 passes, 2-3-4 CLEAN per BC-5.39.001). 20% fewer passes than W13. 1 codification (DF-AC-TEST-NAME-SYNC-001 v1). 4 lessons recorded (W14.L1-L4). |
| 15 | STORY-041, STORY-051 | **CLOSED/CONVERGED** 2026-05-28 | cb322dc (PR #139 — STORY-041) / 945034d (PR #138 — STORY-051) | First multi-story wave since W10. STORY-041: 8 passes, 3/3 clean streak, 24 BC-prefixed tests. STORY-051: 6 passes, 3/3 clean streak, 19 BC-prefixed tests + 2 test helpers. BC-addition sibling-sweep cascade pattern (W15.L2). 9th+10th implementer-as-PR-executor validations. |
| 16 | STORY-042, STORY-043, STORY-044, STORY-052 | **CLOSED/CONVERGED** 2026-05-29 | fa17dec (PR #146) | PRs #140-146. Retroactive convergence. Per-story: S052(P3-P5), S042(P4-P6), S043(P4-P6), S044(P5-P7). Wave-level R2: 3-lens×3-pass CLEAN; 1 false-positive MEDIUM (VP-006 "orphan") dismissed. BC-5.39.001 ACHIEVED. 5 W16 lessons recorded. |
| 17 | STORY-045, STORY-053, STORY-055 | **CLOSED/CONVERGED** 2026-05-29 | 9633b0d (PR #151 — STORY-055) | PRs #150 (STORY-045), #149 (STORY-053), #151 (STORY-055). Per-story all 3 CONVERGED (3-clean P3-P5, 5 passes each). Wave-level: P1 DIRTY (F-W17-WAVE-C-001/T-001 HIGH — AC-sync sibling-miss) → remediated (STORY-055 v1.2) → P2 3-lens CLEAN. BC-5.39.001 ACHIEVED. 4 lessons (W17.L1-L4). [PG-W17-001/002] codification pending. |
| 18 | STORY-046 (E-4 HTTP, 3pts), STORY-054 (E-5 TLS, 8pts), STORY-056 (E-5 TLS, 8pts), STORY-058 (E-5 TLS, 8pts) | **CLOSED/CONVERGED** 2026-05-29 | 3f87ac3 (STORY-058 PR #155; develop HEAD) | 27pts. PRs #152-155. Wave-level: 3-lens CLEAN round-1 (consistency/integration-static/traceability) on frozen 3f87ac3; BC-5.39.001 ACHIEVED; no dirty round. PG-W18-001/002/003 logged. input-drift: CLEAN (50 HS hashes bumped non-semantic). |
| 19 | STORY-057 (E-5 TLS, 8pts) | **CLOSED/CONVERGED** 2026-05-29 | 616897e (PR #156) | 1 story. 6 passes, 3/3 clean streak P4/P5/P6; BC-5.39.001 ACHIEVED. Brownfield-formalization, ZERO src changes; 114 tls_analyzer_tests + full 903-test suite green. 1HIGH+5MED remediated across P1-P3; 1LOW accepted/documented-intent. PG-W17-001 AC-test-name-sync clean. |
| 20 | STORY-076 (E-8 reporter, SS-11, 5pts) | **CLOSED/CONVERGED** 2026-05-29 | e5cb2b1 (PR #157) | 1 story. 5 passes, 3/3 clean streak P3/P4/P5; BC-5.39.001 ACHIEVED. Brownfield-formalization, ZERO src changes; 40 reporter_json_tests + full 915-test suite green. 1HIGH+3MED remediated P1-P2; 1MED self-inflicted by remediation. First SS-11 reporter story; E-8 epic opened. VP-017 deferred Phase-6. |
| 21 | STORY-077 (TerminalReporter, 8pts) + STORY-079 (CsvReporter, 5pts) | **CLOSED/CONVERGED** 2026-05-30 | 41ab24d (PR #159 — STORY-079) | PRs #158/#159. STORY-077: 3ps-3clean (P1/P2/P3); STORY-079: 13ps 3-clean (P11/P12/P13; spec-side drift cascade). Wave-level: 3-lens R1-DIRTY(VP-method+FSR)→R2-2clean+1dirty(casing)→R3-CLEAN; 3/3 CLEAN. 27 tests (14 terminal+13 csv). SS-11 VP family harmonized (VP-012/016/017). BC-5.39.001 ACHIEVED. |
| 22–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 21 — STORY-077 tests written + per-story convergence | **COMPLETE** 2026-05-30 | BC-5.39.001 ACHIEVED: 3 passes P1/P2/P3 CLEAN (zero findings). 14 reporter_terminal_tests (BC-2.11.006..012). Brownfield-formalization ZERO src. VP-012 deferred Phase-6. |
| Wave 21 — STORY-077 PR merged | **COMPLETE** 2026-05-30 | PR #158 squash-merged → 594567c. 929 tests green. All 8 CI green. Security CLEAN. Worktree + branch removed. |
| Wave 21 — STORY-079 tests written + per-story convergence | **COMPLETE** 2026-05-30 | BC-5.39.001 ACHIEVED: 13 passes; 3-clean P11/P12/P13. All DIRTY passes spec-side (CRLF→LF, VP-020 manual→unit, proptest→unit family sweep, FSR citation, EC-002→EC-004). Test artifact clean from P2. 13 reporter_csv_tests (BC-2.11.020..022). VP-020 unit in-story. |
| Wave 21 — STORY-079 PR merged + wave-level convergence | **COMPLETE** 2026-05-30 | PR #159 squash-merged → 41ab24d. 942 tests green. All 8 CI green. Wave-level 3-lens: R1-DIRTY(2HIGH VP-proof-method+1MED FSR-citation)→remediated; R2 integration-static+traceability CLEAN, consistency DIRTY(1MED CAP-11 casing)→remediated; R3 consistency CLEAN. 3/3 CLEAN. SS-11 VP family harmonized (VP-012/016/017). BC-5.39.001 ACHIEVED. Wave 21 CLOSED. |
| Wave 22 — dispatch | **NEXT** | STORY-078 (blocked_by STORY-077) + STORY-080 (blocked_by STORY-079) unblocked. develop HEAD 41ab24d. |

## Spec Package Summary (Phase 1 — PASSED)

| Artifact | Location | Count |
|----------|----------|-------|
| L2 Domain Specification | `.factory/specs/domain/` | 20 shards |
| L3 PRD | `.factory/specs/prd.md` | 1 file |
| Behavioral Contracts | `.factory/specs/behavioral-contracts/ss-01..ss-13/` | 217 BCs across 12 subsystems |
| BC Index | `.factory/specs/behavioral-contracts/BC-INDEX.md` | 1 file |
| Architecture Package | `.factory/specs/architecture/` | 9 files + ARCH-INDEX.md |
| Module Criticality | `.factory/specs/module-criticality.md` | 1 file |
| DTU Assessment | `.factory/specs/dtu-assessment.md` | DTU_REQUIRED: false |
| Verification Properties | `.factory/specs/verification-properties/vp-001..vp-020` | 20 VPs + VP-INDEX.md |
| PRD Supplements | `.factory/specs/prd-supplements/` | 4 files |

Full Phase 1 convergence detail: `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`

## Session Resume Checkpoint (2026-05-30 — Wave 21 CLOSED; next = Wave 22)

1. Waves 1-21 CLOSED/CONVERGED. develop HEAD: 41ab24d (PR #159 squash-merged 2026-05-30). All 8 CI checks green. 41 stories delivered.
2. Wave 21 CLOSED 2026-05-30: STORY-077 (PR#158→594567c; 3ps-3clean P1/P2/P3; BC-5.39.001 ACHIEVED; 14 reporter_terminal_tests; ZERO src changes) + STORY-079 (PR#159→41ab24d; 13ps 3-clean P11/P12/P13; BC-5.39.001 ACHIEVED; 13 reporter_csv_tests; ZERO src changes). Full suite 942 passed/0 failed.
3. Wave-level convergence ACHIEVED: 3-lens fresh-context. R1 DIRTY (2HIGH VP proof-method divergence + 1MED FSR citation) → remediated. R2 integration-static + traceability CLEAN; consistency DIRTY (1MED BC-2.11.007 CAP-11 casing) → remediated. R3 consistency CLEAN. Net 3/3 lenses CLEAN. SS-11 reporter VP family harmonized: VP-012/016/017 proof_method corrected to match VP-020. Spec corrections committed to factory-artifacts.
4. Deferred open items: F-W21-S079-HASH (stale input-hash; TOOL-MISSING), F-W21-TOOL-001 (bin/compute-input-hash absent), F-W21-VP-METHOD (VP-018/019 proof_method — sweep at next SS-12/SS-08 touch). Cycle-close note: DF-SIBLING-SWEEP-001 "sweep sibling VPs in same subsystem when correcting proof_method" codification candidate — handled-this-wave; consider adding a bullet to policies.yaml.
5. NEXT: Wave 22 — STORY-078 (blocked_by STORY-077) + STORY-080 (blocked_by STORY-079). Both unblocked. Prior checkpoint archived: cycles/phase-3-tdd/session-checkpoints.md.

## Wave Retrospectives

Compacted summary table + full prose: `.factory/cycles/phase-3-tdd/lessons.md` (archived 2026-05-29 — content-routing rule S-7.02).

## Decisions Log

| ID | Decision | Date | Rationale |
|----|----------|------|-----------|
| D-001 | Brownfield mode (target == reference) | 2026-05-19 | No parallel reference repo; in-repo formalization only |
| D-002 | DTU not required | 2026-05-20 | No external service clones needed per dtu-assessment |
| D-003 | CI hotfix: cargo audit shell step | 2026-05-22 | rustsec/audit-check@v2.0.0 fails on push events; PR #111 |
| D-004 | Nightly pin nightly-2026-05-21 is periodic-maintenance | 2026-05-22 | Bumping requires verifying fuzz build; do NOT automate |
| D-005 | Demo recordings local-only (gitignored) | 2026-05-22 | factory-artifacts gitignores cycles/**/demos/; 49 prior files untracked |
| D-006 | [correction 2026-05-29/30] Wave-20/STORY-076 real merge SHA is e5cb2b1 (PR #157). Two earlier recorded SHAs were wrong and have been corrected: a8f3d21 (phantom, pre-merge write) and 4d9e1c7 (transient pre-resolution id). Root cause: post-merge state written before pr-manager's authoritative merge SHA was confirmed; rectified. | 2026-05-29 | Orchestrator supplied SHA before actual merge; real merge commit confirmed e5cb2b1 on origin/develop |
| D-007 | Deferred-item cleanup: DF-16.B closed (bulk 209-BC sweep commit b17c5f0; 0 remaining broken citations); OBS-7 closed (covered by STORY-076 BC-2.11.003 / test_BC_2_11_003_c0_esc_escaped_in_json; PR #157→e5cb2b1); 4 governance candidates codified to policies.yaml (DF-INPUT-HASH-CANONICAL-001, DF-ADVERSARY-CHECKOUT-GUARD-001, DF-TEST-CITATION-SWEEP-001, DF-TEST-NAMESPACE-001); 6 externally-blocked items archived to cycles/phase-3-tdd/deferred-items-archive.md (W9-D2/D3/D4 upstream-plugin, W9-D12 awaiting-PO, W1.3/W2.5 upstream, W7.1 public-api, Phase-4-ENTRY, F-S058-P13-O4). | 2026-05-30 | STATE.md deferred-item cleanup burst; no information lost |
| D-008 | [2026-05-30] STORY-079 input BC-2.11.020 corrected v1.2→v1.3 (CRLF→LF). STORY-079 input-hash NOT recomputed because canonical bin/compute-input-hash is missing from repo (DF-INPUT-HASH-CANONICAL-001 forbids hand-compute). Logged F-W21-S079-HASH + F-W21-TOOL-001; input-hash re-validated at Phase-4 gate after tool restore. Decision: do not block STORY-079 per-story convergence on a stale-hash finding that cannot be mechanically resolved and is gated for Phase-4 anyway (zero src/behavioral impact; test↔spec sync intact; AC test-name citations unchanged). | 2026-05-30 | STORY-079 Pass-1 adversarial review F-002; unblocking per-story convergence on non-mechanical, phase-gated gap |

## Blocking Issues

None open.

## Drift Items

All items below require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Closed items archived in `.factory/cycles/drift-remediation-2026-05-29/closed-items.md`.
Externally-blocked / phase-gated items (W9-D2/D3/D4 upstream-plugin, W9-D12 awaiting-PO, W1.3/W2.5 upstream, W7.1 public-api, Phase-4-ENTRY, F-S058-P13-O4) archived to cycles/phase-3-tdd/deferred-items-archive.md — revisit at their named gate/phase.

| ID | Finding | Category | Target | Status |
|----|---------|----------|--------|--------|
| W10-D10-sibling | [test-quality, LOW] tests/reassembly_engine_tests.rs:~14143 `test_story_018_ec008` re-implements the 10,000-flow fill loop inline (should use `fill_findings_to_cap`). Target: next reassembly-test touch. | test-quality | next reassembly touch | OPEN |
| F-DRIFT-C-001 | [cosmetic, LOW] Stale doc-comment in src/analyzer/http.rs `truncate_uri` test: "5 'é' = 10 bytes" vs actual "éééé" 4-char fixture; logic correct. Target: next http-test PR (develop branch). | cosmetic | next http-test touch | OPEN |
| F-S058-P12-O1 | [deferred-LOW] BC-2.07.005 anchor 726-748 vs actual 726-747 (off-by-one). Target: next BC-2.07.005 touch. | spec-gap | next BC-2.07.005 touch | OPEN |
| F-W21-S079-HASH | [process-gap, MEDIUM] STORY-079 input-hash "903f0d0" likely stale after input BC-2.11.020 changed v1.2→v1.3 (CRLF→LF correction, 2026-05-30). Cannot recompute: canonical `bin/compute-input-hash` tool is ABSENT from repo (DF-INPUT-HASH-CANONICAL-001 forbids hand-compute). Re-validate at Phase-4 input-drift gate once the tool is restored. | process-gap | Phase-4 entry / tool-restore | OPEN — TOOL-MISSING |
| F-W21-TOOL-001 | [infra-gap, HIGH] Canonical input-hash tool `bin/compute-input-hash` (referenced by CLAUDE.md + policy DF-INPUT-HASH-CANONICAL-001) does NOT exist in the repo. All input-hash freshness checks are currently un-runnable; this is the likely root of PG-HASH-001 (prior hand-computation). Restore/author the tool (MD5 over declared inputs in inputs-order per policy) before relying on any input-hash drift gate. | infra-gap | tooling | OPEN — BLOCKS-HASH-VALIDATION |
| F-W21-VP-METHOD | [spec-consistency, LOW] VP-018 (cli.rs/SS-12) + VP-019 (dns.rs/SS-08) have proof_method frontmatter (`manual`) diverging from VP-INDEX/body (`integration`/`unit`) + consuming BC VP-table rows — same pattern fixed for VP-012/016/017 in the SS-11 reporter family (Wave-21). Out of SS-11/Wave-21 scope; sweep when their owning subsystem/story is next touched, or in a dedicated VP-method-consistency pass. | spec-consistency | next SS-12/SS-08 touch or dedicated pass | OPEN |

## Cycle-Close Follow-Up Items (OPEN)

Most items from Waves 1-16 closed during drift-remediation-2026-05-29. Closed items archived in
`.factory/cycles/drift-remediation-2026-05-29/closed-items.md`.
Externally-blocked / phase-gated items (PG-W18-001/002/003 codified → policies.yaml, OBS-7 closed → STORY-076, W1.3/W2.5 upstream, W7.1 public-api, Phase-4-ENTRY, F-S058-P13-O4) archived to cycles/phase-3-tdd/deferred-items-archive.md.

| ID | Item | Priority |
|----|------|----------|
| F-S058-P11-001 | [deferred-LOW] Stale "sync to story after this pass" comment at tls_analyzer_tests.rs:6819. Target: next tls-test PR. | P3 — DEFERRED |
| F-S058-P11-002 | [deferred-LOW] test_nonhandshake_types EC-label header lists EC-002/003/004 but body covers EC-001-004. Cosmetic inconsistency. Target: next tls-test PR. | P3 — DEFERRED |
| W20-NIT-001 | [deferred-LOW, STORY-076 PR#157] optional future U+0080 C1-boundary test for JsonReporter byte handling. Target: next reporter-test PR. | P3 — DEFERRED |

Historical process-gap items from Phase 1 (P1.1–P1.3, P3-PG, P4-PG1/2/3, P5-PG, P8-DEFER,
P10-PG, P-CITE-PG): archived in `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.

## Governance Policy

Full policy text: `.factory/policies.yaml` (canonical). Prose detail archived: `cycles/phase-3-tdd/governance-policy-detail.md`.
4 policies codified 2026-05-30 from PG-HASH-001 + PG-W18-001/002/003 (detail: cycles/phase-3-tdd/lessons.md).

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | required-before-issue |
| DF-SIBLING-SWEEP-001 (v1→v4) | CRITICAL |
| DF-PR-MANAGER-COMPLETE-001 | HIGH |
| DF-ADVERSARY-METHODOLOGY-001 | HIGH |
| DF-AC-TEST-NAME-SYNC-001 (v2) | MEDIUM |
| DF-CONVERGENCE-BEFORE-MERGE-001 | CRITICAL |
| DF-DEVELOP-FRESHNESS-001 | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | HIGH |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |

## Tech Debt (Open)

| ID | Description | Priority | Source |
|----|-------------|----------|--------|
| O-07 | `rayon` declared in Cargo.toml but unused in `src/` — dead dependency | P2 | adversarial pass 1 (LOW finding) |
| O-08 | `src/analyzer/dns.rs` module doc-comment stale — references removed behavior | P3 | adversarial pass 29 (observation O-1); recorded in domain-debt.md |

Full register: `.factory/tech-debt-register.md`

## Open Issues (from Phase 0 / deferred findings)

| Issue | Summary |
|-------|---------|
| #100 | `Finding.timestamp` always None; thread pcap timestamps |
| #101 | Empirically characterize anomaly-threshold FP rates |
| #102 | Cap weak-cipher ClientHello evidence Vec, CWE-405 |
| #103 | Bidirectional size-symmetry discriminator for small-segment detector |
| #104 | Surface control bytes in non-ASCII SNI summary, BC-TLS-037 |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- SS-03 gap in BC numbering is intentional (subsystem not applicable).
- Phase 0 canonical ground truth: `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`.
- Wave-level convergence history: `.factory/cycles/phase-3-tdd/convergence-trajectory.md`.
- Phase 1 adversary pass detail (33 passes): `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.
- Phase 2 story-adversary pass detail (10 passes): `.factory/cycles/v0.1.0-greenfield-spec/story-adversary-pass-*.md`.
- Wave 1-18 per-wave detail fields: `.factory/cycles/phase-3-tdd/wave-history.md`.
