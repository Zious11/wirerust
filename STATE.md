---
pipeline: ACTIVE
phase: FIX-CYCLE-SPEC-DONE
phase_status: "fix-pc-013-014-015 SPEC BURST committed (D-221). BC-2.16.016 NEW + 3 BC bumps + 4 story propagations. Awaiting implementation phase."
product: wirerust
mode: brownfield (fix bundle), feature-mode-lite
timestamp: 2026-06-23T23:30:00Z

# Release chain
released_version: v0.9.4
released_at: "2026-06-23"
release_tag: v0.9.4
release_commit: 96b49e8
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.9.4
release_yml_run: "28053327452 SUCCESS — 4 binaries published"
prior_released_version: v0.9.3
prior_released_at: "2026-06-22"
prior_release_tag: v0.9.3
prior_release_commit: 2dbf461
v092_release_tag: v0.9.2
v092_release_commit: b73b242
v091_release_tag: v0.9.1
v091_release_commit: ad4eec8
v090_release_tag: v0.9.0
v090_release_commit: 986e148

# Ground-truth HEADs (verified at D-219 — 2026-06-23)
develop_head: 0115d0e
main_head: 96b49e8
factory_artifacts_head: a69d757

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_4_completed: "2026-06-01"
phase_5_completed: "2026-06-01"
phase_6_completed: "2026-06-02"
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 78
current_cycle: fix-pc-013-014-015 (OPENED D-219 — 2026-06-23)
current_wave: "FIX-CYCLE-SPEC-DONE (D-221). Spec burst committed. BC-2.16.016 NEW + BC-2.16.004 v1.9 + BC-2.16.010 v1.8 + BC-2.15.020 v1.4 + 4 story propagations. Awaiting implementation."

# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []

# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-22
maintenance_started_at: "2026-06-22"
maintenance_completed_at: "2026-06-23"
maintenance_findings_count: 38
maintenance_blocking: false
maintenance_prior_run_id: maint-2026-06-17
maintenance_prior_run_status: COMPLETE
maintenance_prior_completed_at: "2026-06-17"
maintenance_prior_findings_count: 48
maintenance_prior_blocking: false

# Convergence (archive pointer)
adversary_convergence_counter: SATISFIED
convergence_trajectory: "Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
---

# VSDD Pipeline State — wirerust

## ACTIVE CYCLE — fix-pc-013-014-015 (D-219)

**Prior checkpoints D-203 through D-218 are archived. All decisions from those checkpoints remain final.**

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run the feature-mitre-json-names cycle — CLOSED (D-217). PRs #306 (mitre_attack, merged to develop), #307 (ICS tactic fix), #308 (docs) all merged.
- Do NOT re-cut v0.9.4 — RELEASED (main 96b49e8, tag v0.9.4, 4 binaries, run 28053327452).
- Do NOT re-apply the ICS tactic fix — MitreTactic already has 20 variants (IcsDiscovery/IcsCollection/IcsCommandAndControl added); T0830→IcsCollection/TA0100, T0831→IcsImpact/TA0105 already corrected in src/mitre.rs on develop.
- Do NOT re-post issue-triage validation comments — already posted on #255/#252/#103/#101/#67/#64/#3/#63 last session.
- Do NOT re-run maintenance sweep maint-2026-06-22 — COMPLETE (D-204). PRs #304 + #305 already merged to develop.
- Do NOT reopen D-200-era decision threads (a)/(b)/(c) — all three CLOSED.

### GROUND-TRUTH HEADs (verified at D-219 — 2026-06-23)

- **develop:** `0115d0e` (local == origin/develop; working tree clean). Re-verify: `git rev-parse --short develop` == `0115d0e`.
- **main:** `96b49e8` — PR #309 merge commit (`chore: release v0.9.4`); tag `v0.9.4` on this commit.
- **factory-artifacts:** `a69d757` (pre-cycle-open head). Re-verify: `git -C .factory log -1 --format='%h'`.
- **Open PRs:** None at cycle open. Re-verify: `gh pr list` must return empty.
- **Worktrees:** main repo (develop) + `.factory/` only. No story/fix/feature worktrees open.

### RESUME PROCEDURE (execute in order)

1. Run `vsdd-factory:factory-worktree-health` — BLOCKING. Do not proceed until PASS.
2. Read `.factory/STATE.md` in full.
3. Verify: `git rev-parse --short develop` == `0115d0e` AND `git rev-parse --short main` == `96b49e8`.
4. Verify: `gh pr list` returns empty.
5. Verify: `git worktree list` shows only main repo + `.factory/` (no story/fix/feature worktrees).
6. Confirm both trees clean: `git status` (main repo) and `git -C .factory status`.
7. Active cycle: `fix-pc-013-014-015`. Scope: `.factory/code-delivery/fix-pc-013-014-015/scope.md`. PC-013/014/015 IN-PROGRESS.

### OPEN ITEMS (backlog — non-blocking, no active work)

| ID | Summary | Status |
|----|---------|--------|
| DRIFT-UNCOMMITTED-TEST-EDITS-001 | [MEDIUM, process-gap]: F5 committed only src/mitre.rs; 3 test files were working-tree edits; CI caught on push. Engine policy candidate: convergence-clean-tree-guard. | DEFERRED MEDIUM — engine codification |
| DRIFT-BC-TEMPLATE-EC-VP-MAP-001 | [LOW, process-gap]: BC template EC table can have more rows than VP/test-name table. Engine/template concern, not product defect. | DEFERRED LOW — engine/template |
| DRIFT-MITRE-SUBSET-COUNT-TESTS-001 | [LOW]: mitre/multitag dual-count subset tests (21/13 vs 25/17) — pre-existing cruft, no correctness impact. | DEFERRED LOW — future maintenance |
| DRIFT-ARP-DEMO-FIXTURE-001 | [LOW]: No ARP pcap fixture; T0830→TA0100 unit-tested but not demoed live. | DEFERRED LOW — future cycle |
| PO-BACKLOG-MAINT-2026-06-22 | DNP3/ARP/Modbus/finding-collapse holdout coverage gap (73 declared seeds, 0 files) + HS-064/075/090/098/108 staleness. Human scope decision needed. | OPEN — product-owner / human |
| PC-013 | ARP internal HashMap invariant `.expect()` sites in arp.rs — fail-safe degradation (not packet panics). BC-2.16.004 v1.9 AUTHORED (D-221). | SPEC-DONE — awaiting implementation |
| PC-014 | dnp3: key MISNAMED — `total_parse_errors` → `parse_errors` BREAKING rename. BC-2.15.020 v1.4 AUTHORED (D-220/D-221); STORY-108 AC-010 updated. | SPEC-DONE — awaiting implementation |
| PC-015 | ARP findings unbounded (no cap) — undocumented design, not hidden cap. BC-2.16.016 NEW + BC-2.16.010 v1.8 AUTHORED (D-221). | SPEC-DONE — awaiting implementation |
| ADV-4 | ci.yml audit comment rationale lost (LOW). | DEFERRED LOW |
| DRIFT-READER-ADR-CITATION-001 | reader.rs ADR citation numbers (LOW). | DEFERRED LOW |
| SEC-008 | Residual unbounded EPB accumulation on `from_pcap_reader` STREAM path (not CLI-reachable). DF-VALIDATION-001 required before GitHub issue. | DEFERRED — latent |
| PERF-REASM-NFR-001 | Formal NFR/VP for reassembly per-packet CPU O(1) amortised. | BACKLOG |
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only; tunneling detection is a human feature scope decision. | OPEN — human decision |
| STORY-121 | E-11 process-gap follow-ups. Open draft — human decision on scope. | OPEN DRAFT |
| INPUT-HASH-ERROR-PRESTORY | STORY-001/091/121 persistent ERROR from `bin/compute-input-hash --scan` (pre-existing). | BACKLOG |
| INPUT-HASH-STALE | STORY-002..005/076..080/101/120 STALE (pre-existing). | BACKLOG |
| ENGINE-IMPROVEMENT-BACKLOG | ~18 engine proposals pending human review, incl. pr-manager shortstop PAT-001; lessons.md Lessons 1 & 2 / policy candidates convergence-clean-tree-guard + magic-number-sweep-on-count-change. Pointer: `cycles/feature-mitre-json-names/session-review.md`. | BACKLOG — human review |
| ISSUE-TRIAGE-OPEN-9 | 9 open GitHub issues triaged: keep-open #255/#252/#103/#101/#67/#63/#3; reframe-needed #6 (rayon obsolete) and #4 (narrow to SQLite — CSV shipped). | OPEN — product-owner |

**Resolved — do not reopen:** maint-2026-06-22, O-07, DEP-001/005, DOC-001..009, F-MAJ-001, CORPUS-OBS-PCAPNG-IFFCSLEN-001, decision-threads (a)/(b)/(c), PERF-REASM-DOS-001, all F6 items, feature-mitre-json-names F1-F7 (D-206..D-217). **SPEC VERSIONS (at D-221):** prd.md v1.35, nfr-catalog v2.4, VP-INDEX v2.10 (31/31), BC-INDEX v1.72 (305 BCs / 304 active), STORY-INDEX v2.7 (82/57/526 pts). MitreTactic: 20 variants.

---

## Status

**PIPELINE ACTIVE. Fix cycle fix-pc-013-014-015 SPEC-DONE (D-221, 2026-06-23). PC-013/014/015 specs committed. Awaiting implementation.**

Latest release: v0.9.4 (main `96b49e8`, tag `v0.9.4`, 4 binaries, run 28053327452). develop=0115d0e. stories_delivered=78. No open PRs. Active cycle: fix-pc-013-014-015.

Scope: post-release defect fixes PC-013 (ARP `.expect()` invariant sites), PC-014 (dnp3 key rename `total_parse_errors` → `parse_errors`, BREAKING), PC-015 (ARP unbounded findings, undocumented). BCs AUTHORED (D-221): BC-2.16.016 NEW v1.0; BC-2.16.004 v1.9; BC-2.16.010 v1.8; BC-2.15.020 v1.4. Story propagation complete: STORY-113 v1.3, STORY-114 v1.5, STORY-108 v1.2, STORY-115 v1.5. BC-INDEX v1.72 (305 BCs / 304 active). D-220: PC-014 rename human-approved.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog | PASSED | 30/30 lessons; PRs #69-#99 |
| Phase 1 — Spec Crystallization | PASSED 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs |
| Phase 2 — Story Decomposition | PASSED 2026-05-21 | 49 stories / 11 epics / 27 waves |
| Phase 3 — TDD Implementation | PASSED 2026-05-31 | 48/48 stories, 27/27 waves |
| Phase 4 — Holdout Evaluation | PASSED 2026-06-01 | mean 0.949; detail: cycles/v0.1.0-greenfield-spec/ |
| Phase 5 — Adversarial Refinement | PASSED 2026-06-01 | Adversary gate 3/3 SATISFIED |
| Phase 6 — Formal Hardening | PASSED 2026-06-02 | 8 Kani VPs; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0 | RELEASED 2026-06-12 | SS-15 24 BCs; F7 5-dim; tag v0.6.0. Detail: cycles/feature-8-dnp3-v0.5.0/ |
| Feature ARP (E-16) + v0.7.0 | RELEASED 2026-06-16 | STORY-111..115; VP-024 LOCKED. Detail: cycles/feature-arp-v0.7.0/ |
| E-17 ARP QinQ/MACsec + v0.7.1 | RELEASED 2026-06-17 | STORY-116/117; tag v0.7.1 b98a72f |
| Maintenance maint-2026-06-17 | COMPLETE 2026-06-17 | 2 PRs (#261/#262); 5 deferred; 0 blocking |
| E-18 finding-collapse (STORY-118) + v0.8.0 | RELEASED 2026-06-17 | STORY-118; SS-11=29 BCs. Detail: cycles/feature-collapse-v0.8.0/ |
| E-18/E-8 STORY-119 cycle (F1-F7) + v0.9.0 | RELEASED + CLOSED 2026-06-19 | STORY-120/122/119; 293 BCs; tag v0.9.0 986e148. Detail: cycles/feature-story-119-grouped-collapse/ |
| v0.9.1 patch | RELEASED 2026-06-19 | Doc/help; PRs #277/#278; tag v0.9.1 ad4eec8 |
| v0.9.2 patch | RELEASED 2026-06-19 | DNP3 determinism + E2E fixtures; PRs #279/#280; tag v0.9.2 b73b242 |
| Feature pcapng-reader (FE-001) + v0.9.3 | RELEASED + CLOSED 2026-06-22 (D-201) | F1-F7 CONVERGED+HUMAN-APPROVED (D-194). 10 new BCs, VP-INDEX v2.10. PR #302 → main 2dbf461. 4 binaries. |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking; F-MAJ-001 fixed (a6efb23); PR #304 (e458ce2) + PR #305 (e4abbe2). |
| Feature mitre-json-names (issue #64) + v0.9.4 | **RELEASED + CLOSED 2026-06-23 (D-217)** | F1-F7 CONVERGED. mitre_attack enrichment (STORY-129) + ICS tactic fix (F5 F-1). 20 MitreTactic variants. 5 BCs bumped. BC-INDEX v1.71 (303 BCs). PR #306/307/308/309. tag v0.9.4 96b49e8. 4 binaries. stories_delivered=78. |
| Fix cycle fix-pc-013-014-015 | **SPEC-DONE (D-221, 2026-06-23)** | PC-013/014/015 spec burst committed. BC-2.16.016 NEW (v1.0); BC-2.16.004 v1.9; BC-2.16.010 v1.8; BC-2.15.020 v1.4 (BREAKING rename, D-220). STORY-113 v1.3 (+AC-022), STORY-114 v1.5 (+AC-018), STORY-108 v1.2 (AC-010 updated), STORY-115 v1.5. BC-INDEX v1.72 (305 BCs / 304 active). Awaiting implementation. |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md` (archived at cycle close)
D-206..D-217: `cycles/feature-mitre-json-names/decisions-archive.md` (archived at cycle close)

| ID | Decision | Date |
|----|----------|------|
| D-203 | SESSION PAUSED — SAFE TO CLEAR. All three D-200-era threads closed. Pipeline quiesced: no open PRs, no active cycle, no in-flight work. | 2026-06-22 |
| D-204 | Maintenance sweep maint-2026-06-22 COMPLETE. 0 blocking. PR #304 (e458ce2) + PR #305 (e4abbe2) merged. F-MAJ-001 fixed (a6efb23). 2 LOWs deferred; 1 engine-note. | 2026-06-23 |
| D-205 | SAFE-TO-CLEAR checkpoint refreshed. Ground truth: main=2dbf461, develop=e4abbe2, PRs=0, worktrees=main+.factory. Pipeline quiesced. | 2026-06-23 |
| D-217 | v0.9.4 RELEASED. PR #309 (release/0.9.4) merged to main 96b49e8; annotated tag v0.9.4; release.yml run 28053327452 SUCCESS, 4 binaries published; develop back-merged 0115d0e. Feature cycle feature-mitre-json-names CONVERGED + RELEASED + CLOSED: delivered mitre_attack JSON enrichment (issue #64, STORY-129) + ICS-matrix tactic-ID correctness fix (F5 F-1, incl. T0830/T0831 corrections). stories_delivered=78. Pipeline quiesced. | 2026-06-23 |
| D-218 | SAFE-TO-CLEAR checkpoint written. Session that delivered v0.9.4 (feature-mitre-json-names cycle: issue #64 mitre_attack + ICS tactic-ID correctness fix) is complete and CLOSED. Ground truth: develop=0115d0e, main=96b49e8, v0.9.4 released, 0 open PRs, worktrees=main+.factory only, pipeline quiesced. Safe to clear the session. | 2026-06-23 |
| D-219 | Human selected backlog item "fix bundle PC-013/14/15" as next work. Pipeline leaving QUIESCED into fix cycle `fix-pc-013-014-015`. F1 delta analysis (product-owner) corrected all three STATE.md labels: PC-013 = internal HashMap invariant .expect() sites in arp.rs (not packet panics), PC-014 = key MISNAMED (total_parse_errors) not missing, PC-015 = no findings cap exists (undocumented design, not hidden cap). Ground truth at open: develop=0115d0e, main=96b49e8, factory-artifacts=a69d757. | 2026-06-23 |
| D-220 | PC-014 resolution decision = RENAME dnp3 output key `total_parse_errors` → `parse_errors` to match sibling analyzers (HTTP/TLS/Modbus). This is a BREAKING JSON output change — requires CHANGELOG entry and a minor version bump at release. Human-approved. | 2026-06-23 |
| D-221 | fix-pc-013-014-015 SPEC BURST committed. All three PC specs anchored: BC-2.16.004 v1.9 (PC-013 fail-safe degradation invariant 6, EC-011/EC-012, arp.rs lines 555/576/642/827); BC-2.15.020 v1.4 (PC-014 BREAKING key rename `total_parse_errors`→`parse_errors`, D-220 confirmed); BC-2.16.016 NEW v1.0 + BC-2.16.010 v1.8 (PC-015 ARP unbounded findings, intentional no-cap documented). Story propagation: STORY-113 v1.3 (+AC-022), STORY-114 v1.5 (+AC-018), STORY-108 v1.2 (AC-010 updated — PC-014 key rename now anchored), STORY-115 v1.5. BC-INDEX v1.72 (305 BCs / 304 active / SS-16=16). Phase: FIX-CYCLE-OPEN → FIX-CYCLE-SPEC-DONE. | 2026-06-23 |

## Governance Policy

Full policy text: `.factory/policies.yaml`.

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | HIGH |
| DF-SIBLING-SWEEP-001 (v4) | CRITICAL |
| DF-PR-MANAGER-COMPLETE-001 | HIGH |
| DF-ADVERSARY-METHODOLOGY-001 | HIGH |
| DF-AC-TEST-NAME-SYNC-001 (v2) | MEDIUM |
| DF-CONVERGENCE-BEFORE-MERGE-001 | CRITICAL |
| DF-DEVELOP-FRESHNESS-001 (v2) | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | MEDIUM |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |
| DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 | HIGH |
| DF-CANONICAL-FRAME-HOLDOUT-001 | CRITICAL |
| DF-BC-COMPLETENESS-SWEEP-001 | HIGH |
| DF-GREEN-DOC-TENSE-SWEEP (v2) | HIGH |
| DF-KANI-NONVACUITY-001 | HIGH |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`.
- Issues: #104/#102/#64 CLOSED; all actions SHA-pinned; dtolnay/rust-toolchain @stable/@nightly exempted.
- STORY-INDEX.md authoritative (82 stories / 57 waves / 526 pts — v2.7).
- Cycle artifacts: `cycles/feature-mitre-json-names/` (decisions-archive.md D-206..D-217, cycle-manifest.md, f6-hardening.md, lessons.md, session-review.md).
