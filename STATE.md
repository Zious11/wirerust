---
pipeline: FEATURE-MODE
phase: F4
phase_status: "F4 Wave 60 — STORY-134 code green @ac04edd; per-story adversarial convergence: Pass-3/4 spec fixes applied (BC-2.17.010 v1.1 F8-001, BC-2.17.008 v1.2 sentinel, ADR-010 Decision 4 roster); re-confirming convergence (3 clean passes needed). BC-INDEX v1.84. STORY-134 input-hash 604b9de→16d03a6 [D-244]."
product: wirerust
mode: feature-mode
timestamp: 2026-06-25T22:15:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
release_yml_run: "28109367603 SUCCESS — 4 binaries"
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"

# Ground-truth HEADs (verified D-242 — 2026-06-25)
develop_head: d562ccc
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 82
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: "Wave 60 STORY-134 per-story adversarial — Pass-3/4 spec fixes applied (D-244): BC-2.17.010 v1.1 (F8-001: command_counts NOT in process_pdu; reattributed to BC-2.17.016 frame-walk PC-0 — last unamended SS-17 BC, F8-001 fully propagated), BC-2.17.008 v1.2 (M-1: error_window_active bool replaces ==0 sentinel), ADR-010 Decision 4 roster (error_window_active field added). Code unchanged @ac04edd (worktree worktree-issue-316-story-134-enip-recon). STORY-134 input-hash 604b9de→16d03a6. BC-INDEX v1.83→v1.84. Re-confirming convergence (3 consecutive clean passes per BC-5.39.001). Wave 59 FULLY CONVERGED & CLOSED (D-242): STORY-132+133 MERGED (PR #319/#320, develop d562ccc)."

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

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. F1/F2/F3 DONE + HUMAN-APPROVED. F4 Wave 59 FULLY CONVERGED & CLOSED (D-242): STORY-132+133 MERGED; develop d562ccc; stories_delivered=82. Wave 60 IN-PROGRESS: STORY-134 code green @ac04edd; Pass-3/4 spec fixes applied (D-244): BC-2.17.010 v1.1 F8-001, BC-2.17.008 v1.2 sentinel, ADR-010 Decision 4 roster. STORY-134 input-hash 604b9de→16d03a6. BC-INDEX v1.84. NEXT = re-confirm convergence (3 clean passes per BC-5.39.001).**

Latest release: v0.10.0 (main `0cbe922`, tag `v0.10.0`). develop=`d562ccc`. stories_delivered=82. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818). GitHub issue #316.

Spec versions: BC-INDEX v1.84 (331 on disk / 330 active; SS-17=26 BCs). ARCH-INDEX v1.8. VP-INDEX v2.11 (VP-032). PRD v1.36. STORY-INDEX v2.8 (91 stories / 61 waves). epics.md v1.8 (E-20).

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603).
- Do NOT convert the 4 `arp.rs` `.expect()` sites to `if-let` — deliberately retained (D-223).
- Do NOT re-run feature-mitre-json-names cycle — CLOSED (D-217). v0.9.4 released.
- Do NOT re-run F1/F2/F3 for feature-enip-v0.11.0 — all CONVERGED + HUMAN-APPROVED (D-228/D-229/D-230/D-231).
- Do NOT re-author STORY-134 stubs/tests/impl — Red Gate @5845ff6 (stubs @25e751e; 18 failing recon tests), Green Gate @f54b9dc (18/18 recon tests pass; full repo green) DONE. Code unchanged @ac04edd.
- Do NOT re-apply STORY-134 Pass-3/4 spec fixes — COMPLETE (D-244). BC-2.17.010 v1.1 (F8-001: command_counts removed from process_pdu; reattributed to BC-2.17.016 frame-walk PC-0 — F8-001 fully propagated), BC-2.17.008 v1.2 (M-1: error_window_active bool replaces ==0 sentinel), ADR-010 Decision 4 roster updated. STORY-134 input-hash 604b9de→16d03a6. BC-INDEX v1.83→v1.84.
- Do NOT re-deliver STORY-130 — MERGED at develop `235ae60` via PR #317 (D-234). 21/21 tests green, clippy/fmt clean, VP-032 Sub-A/B/C Kani harnesses preserved. Demo evidence at docs/demo-evidence/STORY-130/. ADR-0010 shipped.
- Do NOT re-run STORY-131 Pass-1 adversarial remediation — COMPLETE (D-235). M1 EC-007 overload fixed in STORY-131.md. M2 BC-INDEX title fixed (v1.80→v1.81). Boundary doc at cycles/feature-enip-v0.11.0/story-131-132-ondata-boundary.md. Code green @5e61682.
- Do NOT re-run STORY-131 Pass-3 adversarial remediation — COMPLETE (D-236). M-1 warn!/log→eprintln! root fix at ADR-010 Decision 9 + STORY-131/138 sweep. L-1 BC-2.17.023/026 Precondition "N≥1"→"0..=u32::MAX" (v1.0→v1.1). BC-INDEX v1.81→v1.82. STORY-131 input-hash 6d892c4→a119157. Code green @0018a54.
- Do NOT re-deliver STORY-131 — MERGED at develop `edce3bd` via PR #318 (D-237). 3/3 adversarial convergence passes. Wave-58 gate PASSED. Wave-59 dispatch requires human approval (D-231).
- Do NOT re-deliver STORY-132 — MERGED at develop `16d3ce7` via PR #319 (D-239). 3/3 adversarial convergence passes (Pass 2/3/4 clean). 19 cpf_cip tests green. VP-032 Sub-D Kani present. M-001 RESOLVED.
- Do NOT re-run STORY-133 Pass-1 remediation — COMPLETE (D-240). T1693.001 prose corrected in STORY-133.md (name→"Modify Firmware: System Firmware", tactic→IcsInhibitResponseFunction/TA0107). Code fixed @ffca717 (impl + test pin + mitre_tests pin-table + stale-count fn renames + RED-tense scrub). Codified as MITRE-CATALOG-ADR-AUTHORITATIVE-001 in lessons.md.
- Do NOT re-deliver STORY-133 — MERGED at develop `7f040de` via PR #320 (D-241). Per-story adversarial convergence 3/3 (Pass 2/3/4 clean; Pass 1: 2 CRIT + 2 HIGH fixed). 10 mitre_seeding tests green. VP-007 6-step atomic burst complete (T0858/T0816/T1693.001/IcsExecution seeding; EMITTED 17→20, SEEDED 25→28).

### EXACT RESUME POINT — F4 Wave 60 STORY-134 per-story adversarial convergence

STORY-134 worktree HEAD: `ac04edd` (branch `worktree-issue-316-story-134-enip-recon`, base develop `d562ccc`). Code green (18/18 recon tests; BC-2.17.008/010/014; full repo green; clippy/fmt clean). Red Gate @5845ff6 (stubs @25e751e; 18 failing recon tests). Green Gate @f54b9dc (18/18 recon tests pass). STORY-134 input-hash: 16d03a6 (refreshed D-244, Pass-3/4 spec fixes).

**NEXT STEP = dispatch `vsdd-factory:adversary` for STORY-134 per-story convergence (re-confirming after Pass-3/4 spec fixes — need 3 consecutive clean passes per BC-5.39.001).** DO NOT re-run stub-architect/test-writer/implementer — code DONE @ac04edd.

Continue per per-story-delivery.md (convergence → demo-recorder → push → pr-manager 9-step → cleanup). **Halt before merge for human authorization (D-231 cadence).**

Scope confirmed (D-243): STORY-134 owns `process_pdu` + `EnipFlowState`; STORY-137 owns `on_data` frame-walk wiring + `#![allow(dead_code)]` removal (WAVE59-DEADCODE-001/E2E-001 correctly re-targeted to STORY-137).

WAVE59-E2E-001 + WAVE59-DEADCODE-001 RE-TARGETED to STORY-137 (wave 60, BC-2.17.016 frame-walk wiring — STORY-132 adds pure parse fns only, not yet called from on_data; both obligations cannot be fulfilled before that wiring lands).

### Locked design facts (do not re-derive on resume)

ENIP header LITTLE-endian (`from_le_bytes`). `is_valid_enip_frame` single-arg (command-only). `EnipCommandClass` 10 payloadless variants. `CipServiceClass` 15 (0x0A=MultipleServicePacket). `CipHeader={service,request_path}`. `CpfItem={type_id,data}`. `general_status`=byte-2 on 0x00B2 responses. 0x00B2-only CIP detection (0x00B1 deferred v0.12.0). Write-burst default 50 / error-burst 5 strict `>` (51st/6th); both CLI-overridable. T0814 windowed >=3/300s; carry-overflow runs `check_t0814` BEFORE latching `is_non_enip`. `command_counts` SINGLE site=frame-walk (BC-2.17.016 PC-0, counts all incl Unknown). `process_pdu` owns `pdu_count`. `flows_analyzed`→`on_flow_close`. Summary canonical key `parse_errors`. MAX_ENIP_CARRY_BYTES=600, MAX_FINDINGS=10000. MITRE pin ics-attack-19.1. EMITTED 17→20, SEEDED 25→28, catalogue-only 8. Counters u64, window timestamps u32 seconds.

Story input-hashes (verified): STORY-130 272738c→e3c0a6a (D-237 H-001 rehash), 131 a119157 (refreshed D-236), 132 738d0b0 (refreshed Wave-59 delivery start; MATCH at merge), 133 7104101→350dcf3 (refreshed Wave-59 delivery start — ADR-010 D-233/D-236 cosmetic inputs; STORY-133 body unchanged), 134 c82d3ff→604b9de→16d03a6 (604b9de: Wave-60 delivery start, ADR-010 changes; 16d03a6: D-244 Pass-3/4 spec fixes, BC-2.17.008/010/ADR-010 amended). STORY-135..138 STALE — pending F4 per-story refresh (do NOT refresh until each story's delivery wave).

### RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — BLOCKING. Do not proceed until PASS.
2. Read `.factory/STATE.md` + `cycles/feature-enip-v0.11.0/cycle-manifest.md` in full.
3. Verify: `git rev-parse --short develop` == `d562ccc` AND `git rev-parse --short main` == `0cbe922`.
4. Verify: `gh pr list` (expect Dependabot #311 open, non-blocking; PRs #317+#318+#319+#320+#321+#322 MERGED).
5. **REPORT TO HUMAN** — Wave-59 CONVERGED & CLOSED (D-242); Wave-60 approval required before dispatching (D-231 cadence).
6. On approval: begin Wave-60 (STORY-134/135/136/137 — recon detections, command detections, connection lifecycle, frame walk robustness).

### Remaining F4 work (waves 59-61)

Wave 58: COMPLETE (STORY-130+131 merged, D-237/D-238).
Wave 59: FULLY CONVERGED & CLOSED (D-242) — STORY-132 MERGED PR #319 D-239; STORY-133 MERGED PR #320 D-241; develop d562ccc; wave-level 3/3 passes D/E/F; PRs #321+#322 remediation; BC-2.17.012 v1.1 TA-id fix.
Wave 60: STORY-134/135/136/137 (WAVE59-E2E-001 + WAVE59-DEADCODE-001 re-targeted here, must land in STORY-137). Wave 61: STORY-138.

F4 obligations (remaining waves): 12 pcap fixtures (HS-110..122 minus HS-121 which is synthetic); BC frontmatter input-hash writes; M-001 RESOLVED (docs/adr/0010 synced in PR #319). EMITTED/SEEDED baseline confirmed: EMITTED=20, SEEDED=28 (STORY-133 VP-007 complete).

STORY-134 input-hash refreshed (c82d3ff→604b9de, Wave-60 delivery start). STORY-135..138 remain STALE — to be refreshed as F4 obligation per story delivery.

### OPEN ITEMS (backlog — non-blocking)

| ID | Summary | Status |
|----|---------|--------|
| ENGINE-PROPAGATION-GREP-GATE-001 | Mechanical changed-value sibling-grep gate; from feature-enip-v0.11.0 F2. Human decision needed before cycle CLOSE. | OPEN — human review |
| GREEN-DOC-TENSE-TEST-HEADER-STORY | Self-improvement story for GREEN-DOC-TENSE-TEST-HEADER-001 codified process-gap (D-239): add mechanical test-module-header tense check. JUSTIFIED DEFERRAL: codified in lessons.md; follow-up story scope to be set at cycle close alongside STORY-091/STORY-121 wave assignment. Target: resolve before feature-enip-v0.11.0 cycle CLOSE (S-7.02). | DEFERRED — lessons.md codified; story scope pending cycle close |
| DEPENDABOT-311 | PR #311 (actions/checkout 6.0.3→7.0.0) unreviewed. | OPEN — human triage |
| PO-BACKLOG-MAINT-2026-06-22 | DNP3/ARP/Modbus/finding-collapse holdout coverage gap + stale HS. Human scope decision needed. | OPEN — product-owner |
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only; tunneling detection is a human feature scope decision. | OPEN — human decision |
| ISSUE-TRIAGE-OPEN-9 | 9 open GitHub issues triaged (see decisions-archive). | OPEN — product-owner |
| SEC-008 / PERF-REASM-NFR-001 | Residual latent items — DF-VALIDATION-001-gated. | DEFERRED |
| SS-17-BC-INPUT-HASH-BACKFILL | BC-2.17.007 (and likely other SS-17 BC files) carry `input-hash: TBD` — authored before the input-hash convention was applied to BC files. Pre-existing; not a Wave-59 regression. Evaluate at cycle close. | DEFERRED — cycle close |
| ENGINE-IMPROVEMENT-BACKLOG | ~18 engine proposals; lessons.md Lessons 1 & 2. | BACKLOG — human review |
| WAVE-60-TEST-DOC-SWEEP | (a) M-1 — stale renamed-test reference in tests/bc_2_15_110_dnp3_dispatcher_tests.rs:25 doc-comment (cites old test_vp007_seeded_23_emitted_15; fn now test_vp007_story110_seeded_and_emitted_subset_resolves); (b) S1 — residual RED-tense in tests/bc_2_16_story114_arp_tests.rs test_t0830_and_t1557_002_resolves_in_catalog (lines ~44/59/76/93); (c) O-W59-1 — vestigial "staged catalog entry missing" comment in tests/enip_analyzer_tests.rs:1923. All develop-tree test-doc-prose; batch into Wave-60 doc sweep. | OPEN — Wave-60 |
| GREEN-DOC-TENSE-GATE-COVERAGE-001 | bin/check-green-doc-tense scans only `//` comment lines, NOT assert!/assert_eq! message string literals NOR doc-comment test-name references. Recommended: extend gate to scan assert-message string-literal content + add DF-AC-TEST-NAME-SYNC check (backtick-quoted `test_*` in doc-comments must resolve to a fn in same file). Defer to cycle close per S-7.02 alongside other gate/lesson items. | DEFERRED — cycle close |

All GitHub-issue creation DF-VALIDATION-001-gated.

**Resolved — do not reopen:** PC-013/014/015, maint-2026-06-22, all F6 items, feature-mitre-json-names F1-F7, fix-pc-013-014-015.

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
| Feature EtherNet/IP + CIP — F4 | **IN-PROGRESS — Wave 59 CLOSED (D-242); Wave 60 IN-PROGRESS: STORY-134 code green @ac04edd; Pass-3/4 spec fixes applied (D-244); re-confirming convergence.** | STORY-130: PR #317 @235ae60; 3/3 passes. STORY-131: PR #318 @edce3bd; 3/3 passes. STORY-132: PR #319 @16d3ce7; 3/3 passes; 19 cpf_cip tests; VP-032 Sub-D Kani. STORY-133: PR #320 @7f040de; 3/3 passes (Pass-1 2CRIT+2HIGH fixed); 10 mitre_seeding tests; VP-007 complete (EMITTED 20, SEEDED 28). STORY-134: worktree ac04edd; Red @5845ff6; Green @f54b9dc; 18/18 recon tests (T0846/T0888 Pattern A+B; BC-2.17.008/010/014); Pass-3/4 spec fixes applied (BC-2.17.010 v1.1 F8-001, BC-2.17.008 v1.2 M-1 sentinel, ADR-010 Decision 4 roster); input-hash 604b9de→16d03a6; re-confirming convergence. Wave-59 trajectory: `1→2→2→0→0→0`. Convergence detail: cycles/feature-enip-v0.11.0/convergence-trajectory.md. |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md`
D-206..D-217: `cycles/feature-mitre-json-names/decisions-archive.md`
D-219..D-226: `cycles/fix-pc-013-014-015/decisions-archive.md`
D-228..D-231: `cycles/feature-enip-v0.11.0/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-232 | SAFE-TO-CLEAR. F4 Wave 58 STORY-130 mid-TDD: Red Gate @1f9c656 + failing tests @09d5be9 (tests/enip_analyzer_tests.rs, mod parse_header, 21 tests, 0 passed / 21 failed via todo!() panics, clippy clean, all AC Test: citations matched per DF-AC-TEST-NAME-SYNC) DONE. NEXT = implementer. All F1/F2/F3 artifacts durable on factory-artifacts. Resume per RESUME PROCEDURE in this STATE.md. Detail: cycles/feature-enip-v0.11.0/decisions-archive.md. | 2026-06-25 |
| D-233 | STORY-130 mid-TDD adversarial convergence in progress. Code green at `42de2d0` (21/21 tests, clippy/fmt clean, VP-032 Sub-A/B/C Kani harnesses preserved). Pass 1 = 1 HIGH (DF-GREEN-DOC-TENSE, fixed @42de2d0 in develop worktree, no factory-artifacts impact). Pass 2 = PASS: 0 HIGH/CRITICAL, 1 MEDIUM F-130-P2-001 (BC-2.17.002→v1.1 field-count 10→6 + ADR-010 §Decision 8 "6 fields" fix; STORY-130 input-hash dc8a2c9→272738c; BC-INDEX v1.79→v1.80). Convergence counter: 1 clean pass (Pass 2); need 2 more consecutive clean passes per BC-5.39.001. NEXT = adversarial Pass 3. | 2026-06-25 |
| D-234 | STORY-130 per-story delivery COMPLETE. Adversarial convergence ACHIEVED: 3 consecutive clean passes (Pass 2/3/4, 0 HIGH/CRITICAL, BC-5.39.001 MET). Pass 1: 1 HIGH (DF-GREEN-DOC-TENSE, fixed). Pass 2: 1 MEDIUM (BC-2.17.002/ADR-010 field-count 10→6, fixed). Pass 4: 1 LOW (AC-130-001 postcondition citation precision "1-9" vs "1-8") — non-blocking, logged as deferred LOW backlog item. Code: 21/21 tests green, clippy/fmt clean, VP-032 Sub-A/B/C Kani harnesses preserved. SEC-002 latent-panic hardening applied (try_into().expect() → byte-literal array). Demo evidence: docs/demo-evidence/STORY-130/. ADR-0010 (F4 obligation) shipped. Merged via PR #317, develop HEAD now 235ae60 (merge-commit strategy). Worktree cleaned up. NEXT = STORY-131. | 2026-06-25 |
| D-235 | STORY-131/132 on_data boundary decision (architect authoritative). STORY-131 (Wave 58) implements minimal `EnipAnalyzer::on_data` (bytes_received counter only) + dispatcher Rule 7 + CLI flags + reassembly guard. CIP frame-walk/CPF/findings/VP-032 Sub-D deferred to STORY-132 (Wave 59). Rationale: PC-2 wiring guarantee requires non-panicking on_data (DNP3 precedent); white-box classify() tests alone insufficient for BC-2.17.019 PC-2. bytes_received counter is stable across STORY-131→132 transition; STORY-132 extends alongside it, does not remove. Boundary doc: cycles/feature-enip-v0.11.0/story-131-132-ondata-boundary.md. STORY-131 Pass-1 adversarial: 1 HIGH DF-GREEN-DOC-TENSE (dispatch test docs — fixed @5e61682) + 2 MEDIUM (M1 STORY-131.md EC-007 overload fixed, M2 BC-INDEX BC-2.17.020 title sync v1.80→v1.81 fixed). Code green @5e61682 (15/15 dispatch + 21/21 parse, clippy/fmt clean, VP-004 oracle 44818 arm present). Convergence in progress: Pass 2 clean; Pass 3 running; need 3 consecutive clean passes (BC-5.39.001). | 2026-06-25 |
| D-236 | STORY-131 adversarial Pass 3 = PASS (0 HIGH/CRITICAL) with 1 MEDIUM [process-gap] M-1 (false warn!/log requirement: ADR-010 Decision 9 root + STORY-131 + STORY-138 propagation — all fixed to eprintln!/no-log-crate convention) + 2 LOW (L-1 BC-2.17.023/026 Precondition "N≥1" vs 0-accepted — fixed to 0..=u32::MAX v1.0→v1.1; L-2 dispatcher.rs module-doc ENIP omission — fixed @0018a54). Code green @0018a54 (15/15 dispatch + 21/21 parse, clippy/fmt clean). BC-INDEX v1.81→v1.82. STORY-131 input-hash 6d892c4→a119157. M-1 codified as [codified] WARN-LOG-CRATE-001 in cycles/feature-enip-v0.11.0/lessons.md. In-place fix sufficient; re-evaluate at cycle close (S-7.02). STORY-132..138 remain STALE (pending F4 per-story refresh — ADR-010 is input to all; do NOT refresh until delivery wave). | 2026-06-25 |
| D-237 | Wave-58 (STORY-130+131) delivered+merged to develop@edce3bd; regression PASS (1955 tests green, clippy/fmt/release clean, ENIP surface present). Per-story convergence 3/3 each. Consistency-audit H-001 FIXED (STORY-130 input-hash 272738c→e3c0a6a — D-236 ADR-010 Decision-9 eprintln! change was a declared input of STORY-130 but only STORY-131's hash was refreshed in the D-236 burst). Consistency-audit L-001 FIXED (STORY-INDEX.md STORY-130/131 status draft→completed; Wave-58 delivery-progress row draft→DELIVERED & CLOSED). M-001 OUTSTANDING and deferred to STORY-132 PR obligation: sync docs/adr/0010-ethernet-ip-cip-stream-dispatch.md (public copy) to .factory ADR-010 (field count 10→6 line ~598; Decision-9 eprintln! wording line ~697). stories_delivered: 79→80. | 2026-06-25 |
| D-238 | Wave-58 wave-level adversarial convergence ACHIEVED: 3 consecutive clean passes (W58-P1/P2/P3, all 0 HIGH/CRITICAL, BC-5.39.001 MET) reviewing integrated develop@edce3bd. Integration verified: STORY-130 parse ↔ STORY-131 dispatch seam coherent; 5-arg StreamDispatcher::new ripple complete; both exhaustive DispatchTarget matches (on_data, on_flow_close) + classify_oracle updated with Enip arm; sibling routing (HTTP/TLS/Modbus/DNP3) unaffected; reporter take_enip_analyzer integration symmetric with DNP3; early-exit guard includes self.enip.is_none(). Wave 58 FULLY CLOSED (regression PASS + per-story 3/3×2 + consistency audit + wave-level 3/3). STORY-132 obligations logged: M-001 (docs/adr/0010 sync: field-count 10→6 line ~598, Decision-9 eprintln! wording line ~697), WAVE59-E2E-001 (combined e2e test: HTTP+TLS+Modbus+DNP3+ENIP armed, port-44818 traffic through reassembler→dispatcher→take_enip_analyzer→reporter), WAVE59-DEADCODE-001 (remove #![allow(dead_code)] on src/analyzer/enip.rs when STORY-132 wires parse functions into on_data frame-walk). Wave 59 (STORY-132) pending human go-ahead (D-231 cadence). | 2026-06-25 |
| D-239 | STORY-132 per-story delivery COMPLETE. Adversarial convergence ACHIEVED: 3/3 (Pass 2/3/4 clean, 0 HIGH/CRITICAL, BC-5.39.001 MET). Pass 1: 1 HIGH (DF-GREEN-DOC-TENSE test-module header — fixed; recurrence codified as GREEN-DOC-TENSE-TEST-HEADER-001 in lessons.md). Pass 3: 1 LOW (Vec::with_capacity amplification factor → capped). Pass 4: 1 LOW (test PC citations). BCs: BC-2.17.005/006/007/009 (CPF item walk + CIP header parse + path extraction). VP-032 Sub-D Kani harnesses present (run at F6). F-P9-002 fuzz obligation doc comments present (harnesses deferred to F6). 19 cpf_cip tests green. M-001 RESOLVED: docs/adr/0010-ethernet-ip-cip-stream-dispatch.md synced to .factory ADR-010 (field count + eprintln! guard). Merged via PR #319, develop HEAD now 16d3ce7 (merge-commit strategy). stories_delivered: 80→81. WAVE59-E2E-001 + WAVE59-DEADCODE-001 re-targeted to STORY-137 (wave 60, BC-2.17.016 frame-walk — STORY-132 adds pure parse fns only, not yet wired into on_data). Process-gap codified: GREEN-DOC-TENSE-TEST-HEADER-001 (recurred 3× STORY-130/131/132); justified deferral for self-improvement story recorded in STATE.md OPEN ITEMS (target: STORY-133+ dispatch; resolve with STORY-091/STORY-121 wave assignment at cycle close). NEXT = STORY-133. | 2026-06-25 |
| D-240 | STORY-133 adversarial Pass-1 REMEDIATED — Pass 2+ pending. Pass-1: 2 CRITICAL + 2 HIGH. Root cause: STORY-133 prose carried wrong MITRE catalog mapping for T1693.001 — name was "Exploit Public-Facing Application: EtherNet/IP" (Enterprise technique, wrong) vs ADR-010 Decision 7 authoritative "Modify Firmware: System Firmware"; tactic was IcsInitialAccess (wrong) vs IcsInhibitResponseFunction/TA0107 (correct). Implementation followed the wrong prose; test pinned the wrong name; no tactic-gate existed. ALL FIXED at code commit `ffca717` (impl + test pin + mitre_tests.rs authoritative-TA-id pin-table extended with T1693.001→TA0107 + stale-count fn renames + RED-tense scrub). Story prose corrected in this factory-artifacts burst: 4 T1693.001 references at lines ~68, ~123, ~147–148, ~224. Sibling sweep confirmed no other stories/BCs carry the defect (STORY-INDEX/dependency-graph/BC-2.17.007 cite T1693.001 by ID only). STORY-133 input-hash UNAFFECTED (story body is not its own input). VP-007 invariants intact: SEEDED 28, EMITTED 20, T1693.001 excluded from EMITTED, revoked T0855/T0856/T0857 absent. Codified as [codified][process-gap] MITRE-CATALOG-ADR-AUTHORITATIVE-001 in cycles/feature-enip-v0.11.0/lessons.md. Justified deferral: name-correctness pin evaluation deferred to cycle close (tactic-id pin now in place; name-pin cost/benefit evaluated at cycle close per S-7.02). | 2026-06-25 |
| D-241 | STORY-133 per-story delivery COMPLETE. Adversarial convergence ACHIEVED: 3/3 (Pass 2/3/4 clean, 0 HIGH/CRITICAL, BC-5.39.001 MET). Pass 1: 2 CRITICAL + 2 HIGH (T1693.001 wrong name/tactic — all fixed, D-240). VP-007 6-step atomic burst SATISFIED: T0858/T0816/T1693.001 seeded in technique_info(); SEEDED 25→28; SEEDED_TECHNIQUE_ID_COUNT 25→28; EMITTED_IDS 17→20 (T0858/T0816/T0846); IcsExecution variant added (Display "Execution (ICS)", tactic_id "TA0104"); `cargo test mitre` all 10 mitre_seeding tests green. Wave-59 regression PASS (1984 tests, clippy/fmt/release clean on develop 7f040de). Wave-59 consistency audit: 0 HIGH/CRITICAL (FINDING-3 RESOLVED/false-positive: docs/adr/0010 confirmed tracked on develop, blob 09134be; FINDING-4 deferred: SS-17-BC-INPUT-HASH-BACKFILL, input-hash TBD on BC files authored pre-convention, not a regression). Merged via PR #320, develop HEAD now 7f040de (merge-commit strategy). stories_delivered: 81→82. NEXT = Wave 60 (STORY-134/135/136/137) pending wave-level adversarial convergence + human checkpoint (D-231 cadence). | 2026-06-25 |
| D-242 | Wave 59 FULLY CONVERGED & CLOSED. STORY-132+133 merged (PR #319/#320), regression PASS on develop d562ccc (full suite green, green-doc-tense CI gate now live on develop). Per-story adversarial convergence 3/3 each (STORY-132: passes 2/3/4 clean; STORY-133: passes 2/3/4 clean). Wave-level adversarial convergence 3/3 (confirmation passes D/E/F all 0 HIGH/CRITICAL on develop d562ccc). Remediation history: C-1 (T0846 stale `write_burst_emitted` guard cross-story regression — fixed PR #321 + green-doc-tense CI gate); F-WAVE59-C-001/M-2 (stale cross-story count-snapshot prose + RED-tense test comments — fixed PR #322); F-W59-M01 (BC-2.17.012 TA-id wrong TA0105→TA0106 — fixed in this factory-artifacts burst: BC-2.17.012 v1.0→v1.1, BC-INDEX v1.82→v1.83). Full SS-17 detection-BC MITRE-tuple audit: BC-2.17.010/011/013/014/018 all correct; only 012 was wrong. Pre-empts STORY-133-class wrong-spec defect in Wave-60. Follow-ups logged: WAVE-60-TEST-DOC-SWEEP (M-1 stale test-name ref + S1 RED-tense + O-W59-1 vestigial comment) and GREEN-DOC-TENSE-GATE-COVERAGE-001 (extend bin/check-green-doc-tense to scan assert-message literals + test-name backtick refs). | 2026-06-25 |
| D-243 | STORY-134 Green Gate reached. process_pdu + EnipFlowState implemented at worktree `worktree-issue-316-story-134-enip-recon`; HEAD f54b9dc. 18/18 recon tests pass; full repo green; clippy/fmt/green-doc-tense clean. Implements: T0846 ListIdentity one-shot, T0888 Pattern A (Identity-read per-occurrence) + Pattern B (error-burst >threshold one-shot), CIP error-window accumulation (10s), is_non_enip suppression gate. Scope confirmed: STORY-134 owns process_pdu + EnipFlowState; command_counts NOT touched; on_data NOT wired (STORY-137 owns frame-walk per BC-2.17.016 — stub-architect confirmed). BCs covered: BC-2.17.008/010/014. Red Gate @5845ff6 (stubs @25e751e; 18 failing recon tests). NEXT = per-story adversarial convergence Pass 1 (3 consecutive clean passes per BC-5.39.001). Halt before merge per D-231 cadence. | 2026-06-25 |
| D-244 | STORY-134 per-story adversarial convergence Pass-3/4 spec fixes APPLIED. Pass-3 found 2 HIGH spec contradictions (F-134-P3-001: BC-2.17.010 process_pdu pseudo-code commanded command_counts increment — contradicts F8-001; F-134-P3-002: BC-2.17.010 Architecture Anchor pseudo-code re-stated same). Pass-4 found 1 MEDIUM (M-1: BC-2.17.008 PC-2 used `error_window_start_ts==0` as unseeded sentinel — fails when first error arrives at pcap-relative ts=0). ALL resolved via SPEC corrections; code @ac04edd was already correct (implementation pre-empted the spec). BC-2.17.010 v1.0→v1.1: F8-001 amendment applied — PC-1 command_counts increment removed from process_pdu entirely; reattributed to BC-2.17.016 frame-walk (on_data PC-0); Architecture Anchor updated; PC-3 corrected; modified: entry added. F8-001 now fully propagated — BC-2.17.010 was the last unamended SS-17 BC. BC-2.17.008 v1.1→v1.2: PC-2 sentinel fix — replaced `error_window_start_ts==0` with `flow.error_window_active == false`; PC-4 guard updated; EC-008 added (first-error at ts=0 is a valid seed); Architecture Anchors updated. ADR-010 Decision 4 EnipFlowState roster: `error_window_active: bool` field + doc-comment added. STORY-134.md: AC-134-001/002 + Architecture Mapping + Tasks aligned. STORY-134 input-hash 604b9de→16d03a6 (DF-INPUT-HASH-CANONICAL-001). BC-INDEX v1.83→v1.84. Convergence reset: re-confirming (3 clean passes needed). Lesson codified: F8-001-PROPAGATION-COMPLETENESS in cycles/feature-enip-v0.11.0/lessons.md (deferred-to-cycle-close evaluation). | 2026-06-25 |

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- STORY-INDEX.md authoritative (91 stories / 61 waves / 592 pts — v2.8). STORY-130+131+132+133 completed (Waves 58-59, D-237/D-239/D-241). STORY-134..138 draft (waves 60-61). Input-hashes: STORY-130 e3c0a6a, STORY-131 a119157, STORY-132 738d0b0, STORY-133 350dcf3, STORY-134 16d03a6 (D-244 — all MATCH); STORY-135..138 STALE (pending F4 delivery).
- F6 fuzz obligation: `parse_cip_header` + `parse_cpf_items` cargo-fuzz (F-P9-002, from F2 adversarial pass 9).
- Deferred LOW (non-blocking): BC-2.17.010 Description "per-occurrence" → fix to one-shot (PO); dep-graph STORY-133→137 T0814 rationale prose imprecision.
- Issues: #104/#102/#64 CLOSED; all actions SHA-pinned; dtolnay/rust-toolchain @stable/@nightly exempted.
