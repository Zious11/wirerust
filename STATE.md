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
develop_head: c127c1c
current_cycle: v0.1.0-greenfield-spec
current_wave: 23
wave_20_22_detail: "cycles/phase-3-tdd/wave-history.md"
stories_delivered: 43
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "Full Phase-1→W22 trajectory archived in cycles/phase-3-tdd/convergence-trajectory.md. Latest: ...W21-CONVERGED-CLOSED-2026-05-30|W22-S078-story:3ps-3clean(P1/P2/P3;terminal-MITRE-grouping)|W22-S078-DELIVERED(PR#160→bf16c0b)|W22-S080-story:3-clean-P7/P8/P9(FSR+timestamp-lock+test-harden-cascade)|W22-S080-DELIVERED(PR#161→1ecf114)|W22-WAVE-3lens:R1-consistency+integration-CLEAN,traceability-DIRTY(F-W22-T1-demo-rollup+S078-FSR)→remediated(PR#162-docs→c127c1c)→R2-traceability-CLEAN;3/3-lenses-CLEAN|W22-CONVERGED-CLOSED-2026-05-30|E-8-REPORTER-EPIC-COMPLETE(BC-2.11.001..024)"
consistency_audit: CONSISTENT
input_drift_check: CLEAN (Wave-20 STORY-076 test-only formalization; zero src/production changes; reporter/json subsystem — no holdout-scenario hash impact; Wave-19 story-citation/AC-sync bump may apply — verify at Phase-4 entry)
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
wave_history_archived: "cycles/phase-3-tdd/wave-history.md (waves 1-22 detail fields; waves 1-18 extracted 2026-05-29; waves 20/22 extracted 2026-05-31; wave table rows 1-21 extracted 2026-05-31)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE_3_TDD_IMPLEMENTATION — Waves 1-22 CLOSED/CONVERGED; Wave 23 NEXT. 43 stories delivered.
43 stories delivered (STORY-001/069/002/003/004/070/071/005/011/066/012/013/014/019/015/016/020/017/018/021/031/032/033/041/051/042/043/044/052/045/053/055/046/054/056/058/057/076/077/079/078/080).
Wave 22 CLOSED — STORY-078 (PR#160→bf16c0b) + STORY-080 (PR#161→1ecf114) + docs PR#162→c127c1c. 3/3 wave-level lenses CLEAN. E-8 reporter epic COMPLETE (BC-2.11.001..024 all formalized).
develop HEAD: c127c1c (docs PR #162 squash-merged 2026-05-30; full suite green; 970 tests). All 8 CI checks green. NEXT: Wave 23 (STORY-086).

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
| Phase 3 — TDD Implementation | **IN PROGRESS** — Waves 1-22 CLOSED/CONVERGED; Wave 23 NEXT (43 stories; develop HEAD c127c1c; 970 tests green); E-8 reporter epic COMPLETE | W22: 3/3 wave-level lenses CLEAN; BC-2.11.001..024 all formalized; NEXT: Wave 23 (STORY-086) |
| Phase 4 — Holdout Evaluation | NOT STARTED | — |
| Phase 5 — Adversarial Refinement | NOT STARTED | — |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 3 — Current Wave Status

| Wave | Stories | Status | develop HEAD at Close | Notes |
|------|---------|--------|----------------------|-------|
| 1–21 | 41 stories | CLOSED/CONVERGED | see wave-history | Full per-wave detail: cycles/phase-3-tdd/wave-history.md |
| 22 | STORY-078 + STORY-080 | **CLOSED/CONVERGED** 2026-05-30 | c127c1c (PR #162 docs; PRs #160/#161/#162) | STORY-078: 3ps-3clean(P1/P2/P3); STORY-080: 3-clean P7/P8/P9; 3/3 wave-level lenses CLEAN; 28 tests (16 terminal+12 csv); E-8 epic COMPLETE (BC-2.11.001..024) |
| 23 | STORY-086 | **NEXT** | — | unblocked (STORY-080 done); develop HEAD c127c1c |
| 24–27 | (remaining) | NOT STARTED | — | — |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 22 — STORY-080 per-story convergence | **COMPLETE** 2026-05-30 | BC-5.39.001 ACHIEVED: 3-clean P7/P8/P9 (FSR-citation + BC-2.11.024 timestamp Z→+00:00 lock + test-hardening; P1-P6 DIRTY spec-side drift). 12 reporter_csv_tests (BC-2.11.023..024). Brownfield-formalization ZERO src. |
| Wave 22 — STORY-080 PR merged | **COMPLETE** 2026-05-30 | PR #161 squash-merged → 1ecf114. 970 tests green. All 8 CI green. Security CLEAN. pr-reviewer APPROVED. Demo evidence docs/demo-evidence/STORY-080/. Worktree + branch removed. |
| Wave 22 — wave-level convergence | **COMPLETE** 2026-05-30 | 3-lens fresh-context. R1: consistency+integration-static CLEAN; traceability DIRTY (F-W22-T1 demo-evidence epic-rollup wrong AC counts + STORY-078 FSR-citation). Remediated: PR#162 docs→c127c1c + STORY-078 v1.3 FSR. R2: traceability CLEAN. 3/3 lenses CLEAN. BC-5.39.001 ACHIEVED. |
| Wave 22 — CLOSED | **COMPLETE** 2026-05-30 | develop HEAD c127c1c (docs PR #162). 43 stories. E-8 reporter epic COMPLETE (JSON/Terminal/CSV; BC-2.11.001..024 all formalized). [process-observation] F-W22-T1: demo-recorder fabricated cross-story epic-rollup table with wrong AC counts — demo-recorder should cross-check against STORY-INDEX or omit unverified rollup numbers. |
| Wave 23 — dispatch | **NEXT** | STORY-086 unblocked. develop HEAD c127c1c. |

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

## Session Resume Checkpoint (2026-05-30 — Wave 22 CLOSED; next = Wave 23)

1. Waves 1-22 CLOSED/CONVERGED. develop HEAD: c127c1c (docs PR #162 squash-merged 2026-05-30). All 8 CI checks green. 43 stories delivered.
2. Wave 22 delivery complete: STORY-078 (PR#160→bf16c0b; 3ps-3clean P1/P2/P3; 16 reporter_terminal_tests; BC-2.11.013..019; VP-016 in-story) + STORY-080 (PR#161→1ecf114; 3-clean P7/P8/P9 after FSR-citation + BC-2.11.024 timestamp-Z→+00:00 lock + test-hardening cascade; 12 reporter_csv_tests; BC-2.11.023..024) + docs PR#162→c127c1c. Full suite 970 passed/0 failed. 3/3 wave-level lenses CLEAN. E-8 reporter epic COMPLETE (JSON/Terminal/CSV; BC-2.11.001..024 all formalized).
3. NEXT: Wave 23 — STORY-086 (unblocked; develop HEAD c127c1c). No active worktrees or in-flight branches.
4. Open drift items carried forward: F-W22-BC-ANCHOR (LOW; SS-11 reporter BC anchor staleness — dedicated sweep), F-W21-S079-HASH (MEDIUM; TOOL-MISSING), F-W21-TOOL-001 (HIGH; bin/compute-input-hash absent), F-W21-VP-METHOD (LOW; VP-018/019 proof_method), F-DRIFT-C-001, F-S058-P12-O1.
5. Process-observation from Wave 22 wave-level: demo-recorder fabricated cross-story epic-rollup table with wrong AC counts (F-W22-T1). Demo-recorder should cross-check rollup numbers against STORY-INDEX or omit. Not yet codified to policies.yaml.
6. Prior checkpoint archived: cycles/phase-3-tdd/session-checkpoints.md.

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
| F-W22-BC-ANCHOR | [spec-anchor, LOW, bulk-mechanical] SS-11 reporter BC Architecture-Anchor sections (BC-2.11.001..024) cite stale pre-formalization test file `tests/reporter_tests.rs` + pre-formalization test names that no longer exist post brownfield-formalization (Waves 20-22). Should re-anchor to the per-story formalization tests (reporter_{json,terminal,csv}_tests.rs, mod story_NNN, BC-prefixed names). Pre-existing; per-story test diffs correctly scoped + converged. Dedicated reporter-BC re-anchor sweep (like DF-16.B). | spec-anchor | dedicated reporter-BC re-anchor sweep | OPEN |

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
- Wave 1-22 per-wave detail fields: `.factory/cycles/phase-3-tdd/wave-history.md`.
