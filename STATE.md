---
pipeline: FEATURE-MODE
phase: F4
phase_status: "SAFE-TO-CLEAR (D-251). F4 Wave 60 STORY-136 per-story convergence ACHIEVED @b003547 (BC-5.39.001 MET; 0H/0C x3). NEXT = demo-recorder (in-progress) → push → pr-manager 9-step PR (halt before merge per D-231). Resume per RESUME PROCEDURE."
product: wirerust
mode: feature-mode
timestamp: 2026-06-26T02:00:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
release_yml_run: "28109367603 SUCCESS — 4 binaries"
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"

# Ground-truth HEADs (verified D-250 — 2026-06-26)
develop_head: 84be2fb
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 84
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: "Wave 60 STORY-136 per-story convergence ACHIEVED @b003547 (D-251). STORY-134 MERGED PR #323 @e330ccc (D-247); STORY-135 MERGED PR #324 @84be2fb (D-249); stories_delivered=84. STORY-136 demo-recorder in-progress → push → pr-manager."

# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []

# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-22
maintenance_completed_at: "2026-06-23"
maintenance_blocking: false

# Convergence
adversary_convergence_counter: SATISFIED
convergence_trajectory: "Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
---

# VSDD Pipeline State — wirerust

## Status

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. F4 Wave 60 IN-PROGRESS. SESSION PAUSED (D-251). STORY-130-135 DELIVERED+MERGED. STORY-136 per-story convergence ACHIEVED @b003547 (BC-5.39.001 MET); demo-recorder in-progress. SAFE-TO-CLEAR.**

Latest release: v0.10.0 (main `0cbe922`, tag `v0.10.0`). develop=`84be2fb`. stories_delivered=84. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818). GitHub issue #316.

Spec versions: BC-INDEX v1.84 (331 on disk / 330 active; SS-17=26 BCs). ARCH-INDEX v1.8. VP-INDEX v2.11 (VP-032). PRD v1.36. STORY-INDEX v2.8 (91 stories / 61 waves). epics.md v1.8 (E-20).

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603).
- Do NOT convert the 4 `arp.rs` `.expect()` sites to `if-let` — deliberately retained (D-223).
- Do NOT re-run F1/F2/F3 for feature-enip-v0.11.0 — all CONVERGED + HUMAN-APPROVED (D-228/D-229/D-230/D-231).
- Do NOT re-deliver STORY-130 — MERGED PR #317 @235ae60 (D-234).
- Do NOT re-deliver STORY-131 — MERGED PR #318 @edce3bd (D-237).
- Do NOT re-deliver STORY-132 — MERGED PR #319 @16d3ce7 (D-239).
- Do NOT re-deliver STORY-133 — MERGED PR #320 @7f040de (D-241).
- Do NOT re-deliver STORY-134 — MERGED PR #323 @e330ccc (D-247). 20 recon tests green (T0846/T0888; BC-2.17.008/010/014). Per-story M/N/O 3/3 achieved (D-246). input-hash 16d03a6. BC-2.17.010 v1.1 (F8-001: command_counts removed from process_pdu; reattributed to BC-2.17.016 frame-walk PC-0), BC-2.17.008 v1.2 (error_window_active bool), ADR-010 Decision 4 roster updated. BC-INDEX v1.83→v1.84. Green-doc-tense gate CI live.
- Do NOT re-deliver STORY-135 — MERGED PR #324 @84be2fb (D-249). 16 command_detections tests (T0858/T0816/T0836; BC-2.17.011/012/013). Per-story 5/6/7 3/3 achieved (D-248). input-hash ae2d871. Green-doc-tense gate 22 patterns / self-test 54. GREEN-DOC-TENSE-GATE-COVERAGE-001 RESOLVED.
- Do NOT re-run STORY-136 stub-architect — Red Gate @1b5d300 DONE (D-250).
- Do NOT re-author STORY-136 input-hash — 0846e0e DONE (committed 5bb327c, D-250).
- Do NOT re-run STORY-136 TDD or per-story adversarial convergence — CONVERGED @b003547 (D-251). Trajectory: 2H→0H(1MED)→CLEAN→CLEAN→CLEAN; BC-5.39.001 MET. 10/10 connection_lifecycle tests pass; clippy/fmt/input-hash MATCH.

### EXACT RESUME POINT — F4 Wave 60 STORY-136

**F4 Wave 60 — STORY-136 per-story convergence ACHIEVED @b003547** (worktree `worktree-issue-316-story-136-enip-lifecycle`, base develop `84be2fb`; input-hash `0846e0e` MATCH).

**NEXT STEP = demo-recorder (in-progress) per STORY-136 ACs.** Convergence ACHIEVED (D-251). After demo: push worktree branch → pr-manager 9-step PR (halt before merge for human auth per D-231) → merge+cleanup. Do NOT re-run TDD, stub-architect, input-hash, or per-story adversarial convergence.

### Remaining-work map (after resume)

STORY-136 (finish: demo-recorder in-progress → push → pr-manager 9-step PR → merge+cleanup) → STORY-137 (on_data frame-walk wiring BC-2.17.016; carries WAVE59-E2E-001/WAVE59-DEADCODE-001; removes `#![allow(dead_code)]`; wires parse fns; `command_counts` single-increment site) → Wave-60 integration gate (regression + consistency audit + 3-pass wave-level convergence) → Wave 61 STORY-138 (session summary) → Wave-61 gate → F5 scoped-adversarial → F6 formal hardening (VP-032/VP-004/VP-007 Kani; cargo-fuzz F-P9-002 `parse_cip_header`/`parse_cpf_items`) → F7 delta-convergence + human gate → release v0.11.0.

### RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` + `cycles/feature-enip-v0.11.0/cycle-manifest.md` in full.
3. Verify: `git rev-parse --short develop` == `84be2fb`.
4. Verify: `git -C .worktrees/STORY-136-enip-lifecycle log --oneline -1` == `b003547 ...` (per-story convergence achieved @b003547).
5. Run `gh pr list` — expect Dependabot #311 open (non-blocking); PRs #317/#318/#319/#320/#323/#324 MERGED; no open ENIP PRs.
6. Continue demo-recorder for STORY-136 per EXACT RESUME POINT above, then push → pr-manager (9-step, halt before merge per D-231).

### Locked design facts (do not re-derive on resume)

ENIP header LITTLE-endian (`from_le_bytes`). `is_valid_enip_frame` single-arg (command-only). `EnipCommandClass` 10 payloadless variants. `CipServiceClass` 15 (0x0A=MultipleServicePacket). `CipHeader={service,request_path}`. `CpfItem={type_id,data}`. `general_status`=byte-2 on 0x00B2 responses. 0x00B2-only CIP detection (0x00B1 deferred v0.12.0). Write-burst default 50 / error-burst 5 strict `>` (51st/6th); both CLI-overridable. T0814 windowed >=3/300s; carry-overflow runs `check_t0814` BEFORE latching `is_non_enip`. `command_counts` SINGLE site=frame-walk (BC-2.17.016 PC-0, counts all incl Unknown). `process_pdu` owns `pdu_count`. `flows_analyzed`→`on_flow_close`. Summary canonical key `parse_errors`. MAX_ENIP_CARRY_BYTES=600, MAX_FINDINGS=10000. MITRE pin ics-attack-19.1. EMITTED 17→20, SEEDED 25→28, catalogue-only 8. Counters u64, window timestamps u32 seconds.

Story input-hashes: STORY-130 e3c0a6a, STORY-131 a119157, STORY-132 738d0b0, STORY-133 350dcf3, STORY-134 16d03a6, STORY-135 ae2d871 (all MATCH at merge); STORY-136 0846e0e (MATCH — refreshed Wave-60 delivery start); STORY-137..138 STALE (pending F4 delivery — do NOT refresh until each story's delivery wave).

### OPEN ITEMS (backlog — non-blocking)

| ID | Summary | Status |
|----|---------|--------|
| ENGINE-PROPAGATION-GREP-GATE-001 | Mechanical changed-value sibling-grep gate; from feature-enip-v0.11.0 F2. Human decision needed before cycle CLOSE. | OPEN — human review |
| WAVE-60-TEST-DOC-SWEEP | (a) stale renamed-test ref in dnp3_dispatcher_tests.rs:25; (b) RED-tense in arp_tests.rs:~44/59/76/93; (c) vestigial comment in enip_analyzer_tests.rs:1923; (d) STORY-134.md AC flow_key prose vs actual signature (LOW); (e) redundant `service & 0x80 == 0` re-check in enip.rs (LOW). Batch into Wave-60 doc sweep. | OPEN — Wave-60 |
| SS-17-BC-INPUT-HASH-BACKFILL | BC-2.17.007 (and likely other SS-17 BC files) carry `input-hash: TBD`. Pre-existing; evaluate at cycle close. | DEFERRED — cycle close |
| GREEN-DOC-TENSE-GATE-COVERAGE-001 | RESOLVED (gate 22 patterns on develop). Residual: assert-message/test-name-backtick scan deferred to cycle close per S-7.02. | RESOLVED — residual DEFERRED cycle close |
| mitre.rs:358 stale BC-2.17.012 label | T0816 label cites old BC-2.17.012 annotation (cross-story cleanup, non-blocking). | OPEN — Wave-60 doc sweep |
| DEPENDABOT-311 | PR #311 (actions/checkout 6.0.3→7.0.0) unreviewed. | OPEN — human triage |
| PO-BACKLOG-MAINT-2026-06-22 | DNP3/ARP/Modbus/finding-collapse holdout coverage gap + stale HS. Human scope decision needed. | OPEN — product-owner |
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only; tunneling detection is a human feature scope decision. | OPEN — human decision |
| SEC-008 / PERF-REASM-NFR-001 | Residual latent items — DF-VALIDATION-001-gated. | DEFERRED |
| ENGINE-IMPROVEMENT-BACKLOG | ~18 engine proposals; lessons.md Lessons 1 & 2. | BACKLOG — human review |

All GitHub-issue creation DF-VALIDATION-001-gated.

**Resolved — do not reopen:** PC-013/014/015, maint-2026-06-22, all F6 items, feature-mitre-json-names F1-F7, fix-pc-013-014-015, GREEN-DOC-TENSE-GATE-COVERAGE-001 (gate live on develop).

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase 1 — Spec Crystallization | PASSED 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs |
| Phase 2 — Story Decomposition | PASSED 2026-05-21 | 49 stories / 11 epics / 27 waves |
| Phase 3 — TDD Implementation | PASSED 2026-05-31 | 48/48 stories, 27/27 waves |
| Phase 4 — Holdout Evaluation | PASSED 2026-06-01 | mean 0.949 |
| Phase 5 — Adversarial Refinement | PASSED 2026-06-01 | Adversary gate 3/3 SATISFIED |
| Phase 6 — Formal Hardening | PASSED 2026-06-02 | 8 Kani VPs; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0 | RELEASED 2026-06-12 | SS-15 24 BCs. Detail: cycles/feature-8-dnp3-v0.5.0/ |
| Feature ARP (E-16) + v0.7.0 | RELEASED 2026-06-16 | STORY-111..115; VP-024 LOCKED. Detail: cycles/feature-arp-v0.7.0/ |
| E-17 ARP QinQ/MACsec + v0.7.1 | RELEASED 2026-06-17 | STORY-116/117; tag v0.7.1 |
| E-18 finding-collapse (STORY-118) + v0.8.0 | RELEASED 2026-06-17 | SS-11=29 BCs. Detail: cycles/feature-collapse-v0.8.0/ |
| E-18/E-8 STORY-119 cycle + v0.9.0 | RELEASED 2026-06-19 | 293 BCs; tag v0.9.0. Detail: cycles/feature-story-119-grouped-collapse/ |
| v0.9.1/v0.9.2 patches | RELEASED 2026-06-19 | Doc/help + DNP3 determinism; tags v0.9.1/v0.9.2 |
| Feature pcapng-reader (FE-001) + v0.9.3 | RELEASED + CLOSED 2026-06-22 (D-201) | F1-F7 CONVERGED. 10 new BCs, VP-INDEX v2.10. Detail: cycles/feature-pcapng-reader/ |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking; PRs #304/#305. |
| Feature mitre-json-names (issue #64) + v0.9.4 | RELEASED + CLOSED 2026-06-23 (D-217) | 5 BCs bumped. BC-INDEX v1.71 (303). PRs #306-309. tag v0.9.4. |
| Fix cycle fix-pc-013-014-015 + v0.10.0 | **CONVERGED + RELEASED + CLOSED 2026-06-24 (D-226)** | BC-INDEX v1.73 (305). PRs #310-315. tag v0.10.0 0cbe922. |
| Feature EtherNet/IP + CIP (issue #316) — F1/F2/F3 | **CONVERGED + HUMAN-APPROVED (D-228/D-230/D-231)** | 26 BCs (BC-2.17.001..026). 9 stories STORY-130..138 (E-20, 66 pts, waves 58-61). 13 holdouts HS-110..122. ADR-010, VP-032, SS-17. Detail: cycles/feature-enip-v0.11.0/ |
| Feature EtherNet/IP + CIP — F4 | **IN-PROGRESS — Wave 60 SESSION PAUSED (D-251): STORY-130-135 MERGED; STORY-136 convergence ACHIEVED @b003547; demo-recorder in-progress; stories_delivered=84.** | STORY-134 PR #323 @e330ccc MERGED (D-247); STORY-135 PR #324 @84be2fb MERGED (D-249). STORY-136 per-story convergence ACHIEVED @b003547 (D-251); trajectory `2H→0H(1MED)→CLEAN→CLEAN→CLEAN`. Wave-59 trajectory: `1→2→2→0→0→0`. Convergence detail: cycles/feature-enip-v0.11.0/convergence-trajectory.md. |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md`
D-206..D-217: `cycles/feature-mitre-json-names/decisions-archive.md`
D-219..D-226: `cycles/fix-pc-013-014-015/decisions-archive.md`
D-228..D-250: `cycles/feature-enip-v0.11.0/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-251 | SESSION PAUSE — STORY-136 per-story adversarial convergence ACHIEVED @b003547 (BC-5.39.001 MET). Trajectory: 2H→0H(1MED)→CLEAN→CLEAN→CLEAN (3 consecutive clean passes). 10/10 connection_lifecycle tests pass; clippy/fmt clean; input-hash 0846e0e MATCH. Demo-recorder in-progress. NEXT = demo → push → pr-manager 9-step PR (halt before merge per D-231). Do NOT re-run TDD or convergence for STORY-136. | 2026-06-26 |

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- STORY-INDEX.md authoritative (91 stories / 61 waves / 592 pts — v2.8). STORY-130+131+132+133+134+135 completed (Waves 58-60 partial, D-237/D-239/D-241/D-247/D-249). STORY-136..138 draft (waves 60-61).
- F6 fuzz obligation: `parse_cip_header` + `parse_cpf_items` cargo-fuzz (F-P9-002, from F2 adversarial pass 9).
- Deferred LOW (non-blocking): BC-2.17.010 Description "per-occurrence" → fix to one-shot (PO); dep-graph STORY-133→137 T0814 rationale prose imprecision.
- Issues: #104/#102/#64 CLOSED; all actions SHA-pinned; dtolnay/rust-toolchain @stable/@nightly exempted.
