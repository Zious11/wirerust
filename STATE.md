---
pipeline: FEATURE-MODE
phase: F4
phase_status: "SAFE-TO-CLEAR (D-232). F4 Wave 58 STORY-130 mid-TDD (Red Gate done @1f9c656; test-writer in-flight at pause). Resume per RESUME PROCEDURE."
product: wirerust
mode: feature-mode
timestamp: 2026-06-25T00:00:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
release_yml_run: "28109367603 SUCCESS — 4 binaries"
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"

# Ground-truth HEADs (verified D-232 — 2026-06-25)
develop_head: ff4b82b
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 78
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: Wave 58 — STORY-130 mid-TDD (Red Gate @1f9c656; test-writer in-flight at pause). STORY-131 PENDING.

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

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. F1/F2/F3 DONE + HUMAN-APPROVED. F4 TDD Implementation IN-PROGRESS: Wave 58 PAUSED mid-STORY-130 (Red Gate done; test-writer in-flight). SAFE-TO-CLEAR (D-232).**

Latest release: v0.10.0 (main `0cbe922`, tag `v0.10.0`). develop=`ff4b82b`. stories_delivered=78. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818). GitHub issue #316.

Spec versions: BC-INDEX v1.79 (331 on disk / 330 active; SS-17=26 BCs). ARCH-INDEX v1.8. VP-INDEX v2.11 (VP-032). PRD v1.36. STORY-INDEX v2.8 (91 stories / 61 waves). epics.md v1.8 (E-20).

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603).
- Do NOT convert the 4 `arp.rs` `.expect()` sites to `if-let` — deliberately retained (D-223).
- Do NOT re-run feature-mitre-json-names cycle — CLOSED (D-217). v0.9.4 released.
- Do NOT re-run F1/F2/F3 for feature-enip-v0.11.0 — all CONVERGED + HUMAN-APPROVED (D-228/D-229/D-230/D-231).
- Do NOT re-author STORY-130 Red Gate stubs — already committed at `1f9c656` on branch `worktree-issue-316-story-130-enip-pure-core-parse`.

### EXACT RESUME POINT — F4 Wave 58, STORY-130 mid-TDD

STORY-130 worktree: `.worktrees/STORY-130-enip-pure-core-parse`, branch `worktree-issue-316-story-130-enip-pure-core-parse`, base develop `ff4b82b`. Red Gate stub commit `1f9c656` (enip.rs stubs + tests; cargo check/clippy GREEN; 14 tests FAIL).

**On resume:** `git -C .worktrees/STORY-130-enip-pure-core-parse log --oneline -5`
- If a `test(enip): STORY-130 ... failing tests` commit IS present → next step is the **implementer** (TDD to green).
- If NO / partial → re-run the **test-writer** first (tests/enip_analyzer_tests.rs, mod parse_header, BC-2.17.001-004).

Then continue STORY-130: implementer → per-story adversarial convergence (Step 4.5, BC-5.39.001, 3 clean passes) → demo-recorder → push → pr-manager (9-step PR) → worktree cleanup.

After STORY-130: STORY-131 (dispatch Rule 7 + CLI flags, BC-2.17.019/020/023/026) → Wave-58 integration gate → **REPORT TO HUMAN** (wave-by-wave cadence D-231).

### Locked design facts (do not re-derive on resume)

ENIP header LITTLE-endian (`from_le_bytes`). `is_valid_enip_frame` single-arg (command-only). `EnipCommandClass` 10 payloadless variants. `CipServiceClass` 15 (0x0A=MultipleServicePacket). `CipHeader={service,request_path}`. `CpfItem={type_id,data}`. `general_status`=byte-2 on 0x00B2 responses. 0x00B2-only CIP detection (0x00B1 deferred v0.12.0). Write-burst default 50 / error-burst 5 strict `>` (51st/6th); both CLI-overridable. T0814 windowed >=3/300s; carry-overflow runs `check_t0814` BEFORE latching `is_non_enip`. `command_counts` SINGLE site=frame-walk (BC-2.17.016 PC-0, counts all incl Unknown). `process_pdu` owns `pdu_count`. `flows_analyzed`→`on_flow_close`. Summary canonical key `parse_errors`. MAX_ENIP_CARRY_BYTES=600, MAX_FINDINGS=10000. MITRE pin ics-attack-19.1. EMITTED 17→20, SEEDED 25→28, catalogue-only 8. Counters u64, window timestamps u32 seconds.

Story input-hashes (verified): STORY-130 dc8a2c9, 131 c9970ba, 132 9df8cea, 133 7104101, 134 c82d3ff, 135 4ba6bb1, 136 2af89b5, 137 92f15e6, 138 0053018.

### RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — BLOCKING. Do not proceed until PASS.
2. Read `.factory/STATE.md` + `cycles/feature-enip-v0.11.0/cycle-manifest.md` in full.
3. Verify: `git rev-parse --short develop` == `ff4b82b` AND `git rev-parse --short main` == `0cbe922`.
4. Verify: `gh pr list` (expect only Dependabot #311 open, non-blocking).
5. Check STORY-130 worktree git log (decide test-writer vs implementer per EXACT RESUME POINT above).
6. Resume F4 Wave 58 / STORY-130.

### Remaining F4 work (waves 58-61)

Wave 58: STORY-130 (finish) + STORY-131 → wave gate → human checkpoint.
Wave 59: STORY-132 (CPF/CIP parse + VP-032 Sub-D), STORY-133 (MITRE seeding + VP-007 6-part atomic burst).
Wave 60: STORY-134/135/136/137. Wave 61: STORY-138.

F4 obligations (all waves): 12 pcap fixtures (HS-110..122 minus HS-121 which is synthetic); BC frontmatter input-hash writes; STORY-133 EMITTED/SEEDED baseline reverify vs live src/mitre.rs; docs/adr/0010-*.md uncommitted on develop working tree → commit with first ENIP code PR.

### OPEN ITEMS (backlog — non-blocking)

| ID | Summary | Status |
|----|---------|--------|
| ENGINE-PROPAGATION-GREP-GATE-001 | Mechanical changed-value sibling-grep gate; from feature-enip-v0.11.0 F2. Human decision needed before cycle CLOSE. | OPEN — human review |
| DEPENDABOT-311 | PR #311 (actions/checkout 6.0.3→7.0.0) unreviewed. | OPEN — human triage |
| PO-BACKLOG-MAINT-2026-06-22 | DNP3/ARP/Modbus/finding-collapse holdout coverage gap + stale HS. Human scope decision needed. | OPEN — product-owner |
| DNS-TUNNELING-COVERAGE-001 | DNS analyzer statistics-only; tunneling detection is a human feature scope decision. | OPEN — human decision |
| ISSUE-TRIAGE-OPEN-9 | 9 open GitHub issues triaged (see decisions-archive). | OPEN — product-owner |
| SEC-008 / PERF-REASM-NFR-001 | Residual latent items — DF-VALIDATION-001-gated. | DEFERRED |
| ENGINE-IMPROVEMENT-BACKLOG | ~18 engine proposals; lessons.md Lessons 1 & 2. | BACKLOG — human review |

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
| Feature EtherNet/IP + CIP — F4 | **IN-PROGRESS — Wave 58 PAUSED mid-STORY-130 (D-232 SAFE-TO-CLEAR)** | Red Gate @1f9c656. test-writer in-flight at pause. Wave-by-wave cadence (D-231): gates at waves 58/59/60/61. |

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
| D-232 | SAFE-TO-CLEAR. F4 Wave 58 STORY-130 mid-TDD: Red Gate stub @1f9c656 done; test-writer was in-flight (tests/enip_analyzer_tests.rs, mod parse_header, BC-2.17.001-004) at pause. All F1/F2/F3 artifacts durable on factory-artifacts. Resume per RESUME PROCEDURE in this STATE.md. Detail: cycles/feature-enip-v0.11.0/decisions-archive.md. | 2026-06-25 |

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- STORY-INDEX.md authoritative (91 stories / 61 waves / 592 pts — v2.8). STORY-130..138 flat in stories/, input-hash MATCH.
- F6 fuzz obligation: `parse_cip_header` + `parse_cpf_items` cargo-fuzz (F-P9-002, from F2 adversarial pass 9).
- Deferred LOW (non-blocking): BC-2.17.010 Description "per-occurrence" → fix to one-shot (PO); dep-graph STORY-133→137 T0814 rationale prose imprecision.
- Issues: #104/#102/#64 CLOSED; all actions SHA-pinned; dtolnay/rust-toolchain @stable/@nightly exempted.
