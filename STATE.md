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
develop_head: 616897e
current_cycle: v0.1.0-greenfield-spec
current_wave: 20
wave_19_status: CLOSED
wave_19_started: "2026-05-29"
wave_19_closed: "2026-05-29"
wave_19_stories: STORY-057 (E-5 TLS, 8pts, src/analyzer/tls.rs, BC-2.07.022..028; SNI edge cases)
wave_19_points: 8
wave_19_prs: "#156 (STORY-057 → 616897e)"
wave_19_per_story_convergence: "STORY-057: 6 passes; 3/3 clean streak on passes 4/5/6 (BC-5.39.001 ACHIEVED). Trajectory: P1-DIRTY(5:1HIGH-tautological-AC002-baseline+1MED-misanchor-NameType+2LOW-coverage+1NIT)→P2-DIRTY(2MED:NameType-classifier-reach+capacity-asymmetry)→P3-DIRTY(2MED:EC004-arm3-fidelity+large-SNI-16384-canonical)→P4-CLEAN(2NIT-comment)→P5-CLEAN(2LOW-comment;1-accepted-documented-intent EC-004-illustrative-NameType)→P6-CLEAN(0). Frozen 7854a13→merged 616897e."
wave_19_wave_level_convergence: "single-story wave; per-story convergence == wave-level convergence per BC-5.39.001"
wave_19_delivery: "PR #156 squash-merged → 616897e 2026-05-29; brownfield-formalization, ZERO src changes; 114 tls_analyzer_tests green; full suite 903 passed/0 failed; all 8 CI green (Audit/Clippy/Deny/Format/Fuzz-build/Semantic-PR/Test/Trust-boundary); security CLEAN; pr-reviewer APPROVED 1 cycle (2 non-blocking NITs); worktree + local branch removed; demo evidence docs/demo-evidence/STORY-057/"
wave_19_pg_enforcement: "[PG-W17-001] AC-test-name-sync verified both directions across all 6 adversarial passes; clean"
stories_delivered: 38
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "Full Phase-1→W19 trajectory archived in cycles/phase-3-tdd/convergence-trajectory.md. Latest: W19-S057 6ps-3clean(P4/P5/P6); W19-CONVERGED-2026-05-29."
consistency_audit: CONSISTENT
input_drift_check: CLEAN (50 holdout-scenario hashes bumped 2026-05-29 — non-semantic story-citation drift; Wave-18 story files STORY-046/054/056/058 bumped for AC-citation sync + FSR enumeration + AC-002 reachability clarification; zero src/production changes wave-wide; holdout scenarios are black-box behavioral specs not referencing test-fn names; Wave-19 STORY-057 test-only formalization; zero src/production changes; story-citation/AC-sync bump may apply — verify at Phase-4 entry)
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
wave_history_archived: "cycles/phase-3-tdd/wave-history.md (waves 1-18 detail fields; extracted 2026-05-29)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-19 CLOSED/CONVERGED; 38 stories delivered.
38 stories delivered (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020/017/018/021/031/032/033/041/051/042/043/044/052/045/053/055/046/054/056/058/057).
Wave 19 CLOSED 2026-05-29 — 1/1 story (8pts); STORY-057 6 passes 3/3 clean streak (P4/P5/P6); BC-5.39.001 ACHIEVED. PR #156 squash-merged → 616897e; brownfield-formalization, ZERO src changes; 903 tests green. PG-W17-001 AC-test-name-sync enforcement clean. NEXT: Wave 20.
develop HEAD: 616897e (PR #156 squash-merged 2026-05-29; cargo test --all-targets 903 passed/0 failed). All 8 CI checks green.

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
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-19 CLOSED/CONVERGED; 38 stories delivered (develop HEAD 616897e); Wave 19 CLOSED 2026-05-29 (1 story 8pts; STORY-057 6ps-3clean; BC-5.39.001 ACHIEVED) | Finding progression W19-S057: P1(5findings)→P2(2)→P3(2)→P4(0+2NIT)→P5(0+2LOW)→P6(0); 1LOW-accepted-documented-intent; NEXT: Wave 20 dispatch |
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
| 20–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 18 — STORY-058 tests written + per-story convergence | **COMPLETE** 2026-05-29 | BC-5.39.001 ACHIEVED: 13 passes; 3-clean streak P11/P12/P13 on frozen code 4c252f3. 114 tls_analyzer_tests + 4 tls_integration_tests green; zero src changes. BC-2.07.004/005/029/033/035 remediated; STORY-058 v1.3 AC-sync. 4 deferred-LOW accepted. [PG-W18-002 extended]. |
| Wave 18 — STORY-058 demos + PR merged | **COMPLETE** 2026-05-29 | PR #155 squash-merged → 3f87ac3. All 8 CI green. Security CLEAN. PR review APPROVED 1 cycle. Worktree + local branch removed. Wave 18 4/4 DELIVERED. |
| Wave 18 — Wave-level adversarial convergence + CLOSE | **COMPLETE** 2026-05-29 | 3-lens fresh-context (consistency/integration-static/traceability) on frozen 3f87ac3 — ALL CLEAN round-1. BC-5.39.001 ACHIEVED. No dirty round. input-drift CLEAN (50 HS hashes bumped non-semantic). Wave 18 CLOSED. PG-W18-001/002/003 logged. |
| Wave 19 — STORY-057 tests written + per-story convergence | **COMPLETE** 2026-05-29 | BC-5.39.001 ACHIEVED: 6 passes; 3-clean streak P4/P5/P6 on frozen code 7854a13. 114 tls_analyzer_tests green; zero src changes (brownfield-formalization). 1HIGH-tautological-AC002-baseline + 5MED remediated across P1-P3. 1LOW accepted/documented-intent (EC-004-illustrative-NameType). PG-W17-001 AC-test-name-sync enforcement verified both directions all 6 passes; clean. |
| Wave 19 — STORY-057 PR merged + CLOSED | **COMPLETE** 2026-05-29 | PR #156 squash-merged → 616897e. All 8 CI green (Audit/Clippy/Deny/Format/Fuzz-build/Semantic-PR/Test/Trust-boundary). 903 tests passed/0 failed. Security CLEAN. PR review APPROVED 1 cycle (2 non-blocking NITs). Worktree + local branch removed. Demo evidence docs/demo-evidence/STORY-057/. Wave 19 CLOSED. |
| Wave 20 — dispatch | **NEXT** | Propose Wave 20 stories when ready. |

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

## Session Resume Checkpoint (2026-05-29 — Wave 19 CLOSED; next = Wave 20)

1. Waves 1-19 CLOSED/CONVERGED. develop HEAD: 616897e (PR #156 squash-merged 2026-05-29). All 8 CI checks green. 38 stories delivered.
2. Wave 19 CLOSED 2026-05-29: 1/1 story (8pts). STORY-057 PR#156→616897e (6ps-3clean P4/P5/P6; BC-5.39.001 ACHIEVED). Brownfield-formalization, ZERO src changes. 1HIGH+5MED remediated across P1-P3. 1LOW accepted/documented-intent (EC-004-illustrative-NameType). 903 tests green.
3. PG-W17-001 [AC-test-name-sync] enforcement verified both directions across all 6 adversarial passes; clean. Process-gap Pass-2 F-006 (execution evidence not supplied to adversary) resolved by supplying git-diff-stat + cargo-test evidence in passes 3-6; governing policy DF-ADVERSARY-TOOLCHAIN-PAIRING-001 already exists — no new follow-up story or deferral entry required.
4. input-drift: Wave-19 STORY-057 test-only formalization; zero src/production changes; story-citation/AC-sync bump may apply — verify at Phase-4 entry. Prior input-drift CLEAN: TOTAL=153 MATCH=153 STALE=0.
5. Process-gaps from Wave 18 still open: PG-W18-001 (checkout-guard codification), PG-W18-002 (test-citation sweep checklist), PG-W18-003 (TLS flat-ns latent collision). All require DF-VALIDATION-001 before issue filing.
6. Phase-4-ENTRY deferred: HS-* semantic re-validation against W18 BC corrections at Phase-4 entry (non-blocking). Deferred LOWs: OBS-7, F-S058-P11-001/002, F-S058-P12-O1, F-S058-P13-O4.
7. NEXT: Wave 20 — propose stories when ready. Prior checkpoint archived: cycles/phase-3-tdd/session-checkpoints.md.

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

## Blocking Issues

None open.

## Drift Items

All items below require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Closed items archived in `.factory/cycles/drift-remediation-2026-05-29/closed-items.md`.

| ID | Finding | Category | Target | Status |
|----|---------|----------|--------|--------|
| DF-16.B | [spec-gap, MEDIUM, bulk-mechanical] ~209 BC files across SS-02..SS-13 still have broken `capabilities.md §CAP-NN` citations (SS-01 8 files fixed 2026-05-29). Single bulk find-replace `capabilities.md §CAP-NN` → `domain/capabilities/cap-NN-<slug>.md`; per-cap slug mapping present in `.factory/specs/domain/capabilities/`. | spec-gap | dedicated bulk sweep | OPEN |
| W9-D2 | [process-gap, ESCALATE-UPSTREAM] story-writer template Task #2 wording "Verify Red Gate" incompatible with brownfield-formalization. NOT fixable in this repo; escalate to plugin maintainer. | process-gap | plugin-maintainer | OPEN — ESCALATE-UPSTREAM |
| W9-D3 | [process-gap, ESCALATE-UPSTREAM] story template lacks per-AC VP trace column. NOT fixable in this repo; escalate to plugin maintainer. | process-gap | plugin-maintainer | OPEN — ESCALATE-UPSTREAM |
| W9-D4 | [process-gap, ESCALATE-UPSTREAM] story Token Budget template hardcodes "200K for Sonnet". NOT fixable in this repo; escalate to plugin maintainer. | process-gap | plugin-maintainer | OPEN — ESCALATE-UPSTREAM |
| W9-D12 | [spec-gap, needs-PO-intent] `packets_dropped_capacity` stats counter absent (BC-2.04.015 PC-6 observability). Awaiting PO adjudication: add counter vs document omission. | spec-gap | phase-5 PO | OPEN — AWAITING-PO |
| W10-D10-sibling | [test-quality, LOW] tests/reassembly_engine_tests.rs:~14143 `test_story_018_ec008` re-implements the 10,000-flow fill loop inline (should use `fill_findings_to_cap`). Target: next reassembly-test touch. | test-quality | next reassembly touch | OPEN |
| F-DRIFT-C-001 | [cosmetic, LOW] Stale doc-comment in src/analyzer/http.rs `truncate_uri` test: "5 'é' = 10 bytes" vs actual "éééé" 4-char fixture; logic correct. Target: next http-test PR (develop branch). | cosmetic | next http-test touch | OPEN |
| PG-HASH-001 | [process-gap, HIGH] input-hash was hand-computed (sha256/sorted) this session by story-writer, diverging from `bin/compute-input-hash` (MD5/inputs-order); 12 stories left tool-stale. FIXED 2026-05-29 (scan confirms MATCH=48 STALE=0). Policy-codification candidate: DF-INPUT-HASH-CANONICAL-001 — record at next governance pass. Story-writer/PO prompts should mandate the tool. | process-gap | next governance pass | OPEN — CODIFICATION-PENDING |

## Cycle-Close Follow-Up Items (OPEN)

Most items from Waves 1-16 closed during drift-remediation-2026-05-29. Closed items archived in
`.factory/cycles/drift-remediation-2026-05-29/closed-items.md`.

| ID | Item | Priority |
|----|------|----------|
| W1.3/W2.5 **[RECURRING Waves 1-16]** | No pipeline gate advances story status draft/in-progress → completed on merge. Requires plugin-level fix (vsdd-factory story-writer template); not fixable in this repo. This session (F-DRIFT3B-001): 16 stories manually reconciled across Waves 3-13 (STORY-033 + 016/017/018/019/020/021/031/032/005/011/012/013/014/015/066/071). Root cause unfixed (upstream plugin). | P1 — ESCALATE-UPSTREAM |
| W7.1 | No public-API surface gate for `pub fn` additions. Candidate: `cargo public-api` CI job. Deferred: requires nightly + committed baseline, 2-PR setup. Documented in CLAUDE.md. | P2 — DEFERRED |
| PG-W18-001 | [process-gap] DF-ADVERSARY-METHODOLOGY-001 recurred in STORY-054 pass-10 — adversary reviewed develop instead of feature/STORY-054 worktree; produced 3 false-CRITICAL findings. Pass-11 checkout-guard (branch assertion + grep-count assertion) succeeded. Candidate codification: bake checkout-guard (verify branch==feature/STORY-NNN AND a known story-specific grep-count) into every per-story adversary dispatch template. Also: .factory is gitignored in worktrees — dispatch MUST provide absolute main-repo paths for factory artifacts. REQUIRES DF-VALIDATION-001 research-agent validation before GitHub issue. | P2 — CODIFICATION-CANDIDATE |
| OBS-7 | [deferred-LOW, STORY-056 P9] AC-007 / BC-2.07.020 inv2: "JSON reporter must RFC-8259 escape lossy summary string" clause is a downstream reporter-subsystem claim, not testable within STORY-056's src/analyzer/tls.rs scope. Defer to wave-gate integration check — reporter subsystem not in scope for brownfield-formalization stories. REQUIRES DF-VALIDATION-001 before any issue filing. | P3 — DEFERRED-WAVE-GATE |
| PG-W18-002 | [process-gap, HIGH] Story-anchor fixes (F-S056-P3-001) must trigger same-burst sibling-BC sweep. EXTENDED (STORY-058): AC-test-citation re-points must also sweep story FSR rows + sibling BC Evidence columns + test-file index/header comments in the SAME burst. STORY-058 needed passes 3/4/5/6/8 to chase same AC-013 mis-mapping across 3 locations. Candidate codification: "test-citation change" checklist. REQUIRES DF-VALIDATION-001 before GitHub issue. Detail: cycles/phase-3-tdd/lessons.md (W18-S058.L1). | P2 — CODIFICATION-CANDIDATE |
| PG-W18-003 | [process-gap, MEDIUM] TLS test file (tests/tls_analyzer_tests.rs) uses FLAT namespace while HTTP (tests/http_analyzer_tests.rs) uses per-story `mod` wrappers (the F-W16 collision fix). Latent name-collision risk for future TLS stories — codify one convention (per-story mod wrapper) across both analyzer test files. From F-W18-WAVE-I-006. REQUIRES DF-VALIDATION-001 before GitHub issue. | P2 — CODIFICATION-CANDIDATE |
| Phase-4-ENTRY | [deferred-review] Holdout scenarios HS-* must be semantically re-validated against Wave-18 reachability/arithmetic BC corrections (BC-2.07.002 EC-004 SSL2-ServerHello-rejection, BC-2.07.012 reachability, BC-2.07.029 arithmetic) at Phase-4 holdout-evaluation entry — confirm no scenario asserts pre-correction behavior. Non-blocking for Phase-3 wave close (zero src changes; observable behavior unchanged). | P2 — DEFERRED-PHASE-4-ENTRY |
| F-S058-P11-001 | [deferred-LOW] Stale "sync to story after this pass" comment at tls_analyzer_tests.rs:6819. Target: next tls-test PR. | P3 — DEFERRED |
| F-S058-P11-002 | [deferred-LOW] test_nonhandshake_types EC-label header lists EC-002/003/004 but body covers EC-001-004. Cosmetic inconsistency. Target: next tls-test PR. | P3 — DEFERRED |
| F-S058-P12-O1 | [deferred-LOW] BC-2.07.005 anchor 726-748 vs actual 726-747 (off-by-one). Target: next BC-2.07.005 touch. | P3 — DEFERRED |
| F-S058-P13-O4 | [deferred-LOW] test_stop_after_handshake cross-story AC labels + STORY-058 FSR inclusion — pre-existing collision documented in STORY-058 v1.2. Target: wave-gate or Phase-5. | P3 — DEFERRED |

Historical process-gap items from Phase 1 (P1.1–P1.3, P3-PG, P4-PG1/2/3, P5-PG, P8-DEFER,
P10-PG, P-CITE-PG): archived in `.factory/cycles/v0.1.0-greenfield-spec/convergence-trajectory.md`.

## Governance Policy

Full policy text: `.factory/policies.yaml` (canonical). Prose detail archived: `cycles/phase-3-tdd/governance-policy-detail.md`.

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
