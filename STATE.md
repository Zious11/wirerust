---
pipeline: PHASE_4_HOLDOUT_EVALUATION
phase: phase-4-holdout-evaluation
product: wirerust
mode: brownfield
timestamp: 2026-05-31T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_started: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_3_to_4_gate: PASSED
phase_4_started: "2026-06-01"
develop_head: 6158e6e
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
wave_20_24_detail: "cycles/phase-3-tdd/wave-history.md"
stories_delivered: 48
wave_26_status: CLOSED
wave_26_summary: "single-story (STORY-089 PR#169→450d33e; E-9 decode-error counting + dispatcher unclassified_flows injection + resolve_format precedence + write_output routing); 25 tests (12 AC+5 EC+run_summary parity); BC-2.12.014..017; brownfield-formalization ZERO src changes; 6-pass convergence 3-clean(P4/P5/P6); 17-mutation matrix all caught across run_analyze AND run_summary; 1 HIGH remediated (run_summary untested→6 tests), 5 MEDIUM remediated; F-FSR-088-089 CLOSED; BC-5.39.001 ACHIEVED; per-story==wave-level (BC-5.39.001)"
wave_27_status: CLOSED
wave_27_summary: "FINAL story (STORY-090 PR#170→6158e6e; E-9 Summary Data Model — ingest/ServiceHints/unique_hosts/Serialization); 18 direct unit/integration tests (13 AC+5 EC); BC-2.12.018..021; library module pub mod summary; ZERO src changes; 3-pass convergence (2 remediation rounds — traceability/anchoring only, test logic strong throughout); all 18 mutations killed; cross-suite uniqueness sweep (full corpus) confirmed zero collisions; BC-5.39.001 ACHIEVED; per-story==wave-level (BC-5.39.001); E-9 COMPLETE; PHASE 3 COMPLETE"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3
adversary_gate: SATISFIED
convergence_trajectory: "Full Phase-1→W27 trajectory archived in cycles/phase-3-tdd/convergence-trajectory.md. Latest: ...W26-CLOSED(PR#169→450d33e;BC-2.12.014..017;6ps-3clean;17-mutation-matrix)|W27-S090:3ps-3clean(R1/R2/R3;4→2→0;BC-2.12.018..021;18t;2-remediation-rounds-traceability;corpus-uniqueness-sweep)|W27-CONVERGED-CLOSED(single-story;per-story==wave-level;BC-5.39.001;PR#170→6158e6e)|PHASE-3-COMPLETE(48/48;27/27)"
consistency_audit: CONSISTENT
input_drift_check: CLEAN (Phase-4-entry gate confirmed: bin/compute-input-hash --scan = MATCH=48/STALE=0; zero stale hashes across full corpus)
phase_2_input_hash_drift_check: CLEAN
phase_2_input_hash_drift_check_total: 153
wave_history_archived: "cycles/phase-3-tdd/wave-history.md (waves 1-22 detail fields; waves 1-18 extracted 2026-05-29; waves 20/22 extracted 2026-05-31; wave table rows 1-21 extracted 2026-05-31)"
---

# VSDD Pipeline State — wirerust

## Status

**Pipeline:** PHASE 4 — Holdout Evaluation IN PROGRESS; Phase 3 COMPLETE (48/48, 27/27 waves); develop 6158e6e.
Phase 3→4 gate PASSED 2026-06-01 (input-drift CLEAN MATCH=48/STALE=0; consistency audit READY 217/217 BCs + 100/100 HS CLEAR + 20/20 VPs consistent; human-approved). D-001/D-002 RESOLVED; Phase-4-ENTRY deferred item CLOSED.

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
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves, all CLOSED/CONVERGED; E-1..E-10 ALL COMPLETE; develop HEAD 6158e6e (PR#170); BC-5.39.001 ACHIEVED across all waves; trajectory detail: cycles/phase-3-tdd/convergence-trajectory.md |
| Phase 4 — Holdout Evaluation | **IN PROGRESS** STARTED 2026-06-01 | Pass criteria: mean satisfaction >= 0.85, must-pass >= 0.6; dtu_required: false — evaluate against 100 HS scenarios directly |
| Phase 5 — Adversarial Refinement | NOT STARTED | — |
| Phase 6 — Formal Hardening | NOT STARTED | — |
| Phase 7 — Convergence | NOT STARTED | — |

## Phase 3 — Current Wave Status

| Wave | Stories | Status | develop HEAD at Close | Notes |
|------|---------|--------|----------------------|-------|
| 1–21 | 41 stories | CLOSED/CONVERGED | see wave-history | Full per-wave detail: cycles/phase-3-tdd/wave-history.md |
| 22 | STORY-078 + STORY-080 | **CLOSED/CONVERGED** 2026-05-30 | c127c1c (PR #162 docs; PRs #160/#161/#162) | STORY-078: 3ps-3clean(P1/P2/P3); STORY-080: 3-clean P7/P8/P9; 3/3 wave-level lenses CLEAN; 28 tests (16 terminal+12 csv); E-8 epic COMPLETE (BC-2.11.001..024) |
| 23 | STORY-086 | **CLOSED/CONVERGED** 2026-05-31 | a42e14b (PR #163) | single-story; 3-clean P1/P2/P3 (3→1→0); 15 BC-prefixed CLI tests; BC-2.12.001/002/003/006; E-9 CLI epic OPENED; 4 Low non-blocking |
| 24 | STORY-087 + STORY-096 | **CLOSED/CONVERGED** 2026-05-31 | 9954d44 (PRs #164/#165) | S087: 4ps-3clean(P2/P3/P4;2→1→0→0;BC-2.12.004/005/007;16t); S096: 6ps-3clean(P4/P5/P6;1MED→1MED→1MED→0→0→0;BC-2.13.001..004;14t;facade-mutation-gate); wave-level 3ps(2→1→0); E-10 COMPLETE |
| 25 | STORY-088 | **CLOSED/CONVERGED** 2026-05-31 | 5202fe9 (PR #168) | single-story; first src/main.rs formalization via assert_cmd; 19 tests (14 AC+5 EC); BC-2.12.008..013+VP-018; 6ps-3clean(P4/P5/P6; 3→1→0→0→0→0); 27 mutations caught; BC-5.39.001 ACHIEVED |
| 26 | STORY-089 | **CLOSED/CONVERGED** 2026-05-31 | 450d33e (PR #169) | single-story; 6ps-3clean(P4/P5/P6); 25 tests (12 AC+5 EC+run_summary parity); BC-2.12.014..017; 17-mutation matrix all caught; 1 HIGH+5 MEDIUM remediated; F-FSR-088-089 CLOSED; BC-5.39.001 ACHIEVED |
| 27 | STORY-090 | **CLOSED/CONVERGED** 2026-05-31 | 6158e6e (PR #170) | single-story FINAL; 3ps-3clean(R1/R2/R3); 18 tests (13 AC+5 EC); BC-2.12.018..021; library module; ZERO src changes; 2 remediation rounds (traceability only); corpus uniqueness sweep; BC-5.39.001 ACHIEVED; E-9 COMPLETE; PHASE 3 COMPLETE |

## Phase 3 — Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 26 — CLOSED | **COMPLETE** 2026-05-31 | Single-story wave; per-story convergence == wave-level (BC-5.39.001). F-FSR-088-089 CLOSED. 47 stories. E-9 4/5 (086/087/088/089 done). |
| Wave 27 — STORY-090 converged (3 passes, 2 remediation rounds) | **COMPLETE** 2026-05-31 | R1: 4 findings (BC permuted + 2 name collisions in summary_tests.rs); R2: 2 findings (AC-012 collision reporter_tests.rs + EC-003 retarget); R3: CLEAN. Test logic strong throughout; all 18 mutations killed each round. BC-5.39.001 ACHIEVED. |
| Wave 27 — STORY-090 PR merged | **COMPLETE** 2026-05-31 | PR #170 squash-merged → 6158e6e. 18 tests (13 AC+5 EC); BC-2.12.018..021 formalized. 8/8 CI green. Security CLEAN. pr-reviewer APPROVE cycle 1. 10/13 ACs observable in demos. |
| Wave 27 — CLOSED | **COMPLETE** 2026-05-31 | Single-story FINAL wave; per-story convergence == wave-level (BC-5.39.001). E-9 COMPLETE (5/5 stories: 086/087/088/089/090). 48/48 stories delivered. |
| Phase 3 — COMPLETE | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves, all CLOSED/CONVERGED. All 10 epics complete. NEXT: Phase 3→4 gate (see below). |

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

## Session Resume Checkpoint (2026-06-01 — PHASE 4 IN PROGRESS; Phase 3 COMPLETE)

1. Phase 3→4 gate PASSED 2026-06-01. Pipeline now in PHASE_4_HOLDOUT_EVALUATION. develop HEAD: 6158e6e (unchanged; no src changes in Phase 4 setup). All 8 CI checks green.
2. Phase 4 task: evaluate 100 holdout scenarios (HS-001..HS-100) against delivered codebase. Pass criteria: mean satisfaction >= 0.85, must-pass >= 0.6. dtu_required: false — no clone setup needed; evaluate directly.
3. Consistency audit confirmed (report: cycles/phase-3-tdd/phase-4-entry-consistency-audit.md): 217/217 BCs covered; 100/100 HS scenarios CLEAR of pre-Wave-18-correction behavior; 20/20 VPs consistent.
4. D-001 RESOLVED (STORY-053 EC fixed f368f53). D-002 RESOLVED (this burst: STORY-057/076/077/078/079/080 statuses completed + STORY-INDEX wave rows 3-22 backfilled).
5. Open drift item: F-W25-S088-P6-001 LOW (AC-004 warning-once inv-2 count assertion; test-strength only; accepted or target Phase-5; per DF-VALIDATION-001 no issue without research-agent validation).
6. Prior checkpoint (Phase 3 COMPLETE; next = Phase 3→4 gate) archived: cycles/phase-3-tdd/session-checkpoints.md.

## Phase 3→4 Gate — PASSED 2026-06-01

(a) Input-drift CLEAN (MATCH=48/STALE=0). (b) Consistency audit READY (report: cycles/phase-3-tdd/phase-4-entry-consistency-audit.md; 217/217 BCs covered, 100/100 HS CLEAR, 20/20 VPs consistent). (c) Human approval GRANTED. D-001 RESOLVED (STORY-053 EC fixed f368f53). D-002 RESOLVED (this burst: 6 story statuses + wave rows 3-22 backfilled). Phase-4-ENTRY deferred item CLOSED.

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
| F-W25-S088-P6-001 | [test-strength, LOW] AC-004 warning uses .contains() so a doubled eprintln! (BC-2.12.009 inv-2 "warning printed once") would not be caught. Invariant HOLDS in source (single pre-loop emission, adversary-verified P6); AC-004 traces to PC-5/inv-1 not inv-2 — not a traceability defect. Optional one-line count-assertion hardening; target: STORY-090 touch (next main.rs-adjacent story) or accepted. Per DF-VALIDATION-001, no GitHub issue without research-agent validation. | test-strength | STORY-090 delivery (wave 27) or accept | OPEN |

## Cycle-Close Follow-Up Items (OPEN)

Most items from Waves 1-16 closed during drift-remediation-2026-05-29. Items from the 2026-05-31 drift-remediation sweep (11 items) archived in `.factory/cycles/drift-remediation-2026-05-29/closed-items.md`.
Externally-blocked / phase-gated items (W9-D2/D3/D4, W1.3/W2.5, W7.1, Phase-4-ENTRY, F-S058-P13-O4, CLI-STORY-TEMPLATE) archived to cycles/phase-3-tdd/deferred-items-archive.md.

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
