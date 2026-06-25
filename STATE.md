---
pipeline: FEATURE-MODE
phase: F4
phase_status: "F4 Wave 58 FULLY CONVERGED & CLOSED (merged @edce3bd; per-story 3/3 + wave-level 3/3 + regression PASS); NEXT = Wave 59 STORY-132 (CPF/CIP parse + VP-032 Sub-D), pending human go-ahead. STORY-132 obligations: M-001 docs/adr sync, WAVE59-E2E-001, WAVE59-DEADCODE-001."
product: wirerust
mode: feature-mode
timestamp: 2026-06-25T13:00:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
release_yml_run: "28109367603 SUCCESS — 4 binaries"
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"

# Ground-truth HEADs (verified D-237 — 2026-06-25)
develop_head: edce3bd
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 80
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: "Wave 58 FULLY CONVERGED & CLOSED (D-238): per-story 3/3 x2 + wave-level 3/3 + regression PASS @edce3bd. NEXT = Wave 59 STORY-132 — pending human wave-checkpoint approval. Obligations: M-001, WAVE59-E2E-001, WAVE59-DEADCODE-001."

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

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. F1/F2/F3 DONE + HUMAN-APPROVED. F4 TDD Wave 58 FULLY CONVERGED & CLOSED (D-238): STORY-130 (PR #317) + STORY-131 (PR #318) merged @edce3bd; 1955 tests green; clippy/fmt/release clean; per-story convergence 3/3 each; wave-level adversarial convergence 3/3 clean passes (0 HIGH/CRITICAL, BC-5.39.001 MET); consistency audit H-001+L-001 FIXED. stories_delivered=80. STORY-132 obligations logged: M-001 (docs/adr/0010 sync), WAVE59-E2E-001 (reassembler→reporter e2e test), WAVE59-DEADCODE-001 (remove #![allow(dead_code)] on enip.rs). NEXT = Wave 59 STORY-132 — pending human wave-checkpoint approval (D-231 cadence).**

Latest release: v0.10.0 (main `0cbe922`, tag `v0.10.0`). develop=`edce3bd`. stories_delivered=80. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818). GitHub issue #316.

Spec versions: BC-INDEX v1.82 (331 on disk / 330 active; SS-17=26 BCs). ARCH-INDEX v1.8. VP-INDEX v2.11 (VP-032). PRD v1.36. STORY-INDEX v2.8 (91 stories / 61 waves). epics.md v1.8 (E-20).

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603).
- Do NOT convert the 4 `arp.rs` `.expect()` sites to `if-let` — deliberately retained (D-223).
- Do NOT re-run feature-mitre-json-names cycle — CLOSED (D-217). v0.9.4 released.
- Do NOT re-run F1/F2/F3 for feature-enip-v0.11.0 — all CONVERGED + HUMAN-APPROVED (D-228/D-229/D-230/D-231).
- Do NOT re-deliver STORY-130 — MERGED at develop `235ae60` via PR #317 (D-234). 21/21 tests green, clippy/fmt clean, VP-032 Sub-A/B/C Kani harnesses preserved. Demo evidence at docs/demo-evidence/STORY-130/. ADR-0010 shipped.
- Do NOT re-run STORY-131 Pass-1 adversarial remediation — COMPLETE (D-235). M1 EC-007 overload fixed in STORY-131.md. M2 BC-INDEX title fixed (v1.80→v1.81). Boundary doc at cycles/feature-enip-v0.11.0/story-131-132-ondata-boundary.md. Code green @5e61682.
- Do NOT re-run STORY-131 Pass-3 adversarial remediation — COMPLETE (D-236). M-1 warn!/log→eprintln! root fix at ADR-010 Decision 9 + STORY-131/138 sweep. L-1 BC-2.17.023/026 Precondition "N≥1"→"0..=u32::MAX" (v1.0→v1.1). BC-INDEX v1.81→v1.82. STORY-131 input-hash 6d892c4→a119157. Code green @0018a54.
- Do NOT re-deliver STORY-131 — MERGED at develop `edce3bd` via PR #318 (D-237). 3/3 adversarial convergence passes. Wave-58 gate PASSED. Wave-59 dispatch requires human approval (D-231).

### EXACT RESUME POINT — F4 Wave 59 (pending human wave-checkpoint approval)

Wave 58 FULLY CONVERGED & CLOSED (D-238). STORY-130 merged @235ae60 (PR #317). STORY-131 merged @edce3bd (PR #318). develop=edce3bd. Regression: 1955 tests green, clippy/fmt/release clean, ENIP surface present. Per-story convergence 3/3 each. Wave-level adversarial convergence 3/3 clean passes (0 HIGH/CRITICAL). Consistency audit H-001+L-001 fixed (D-237).

STORY-132 delivery obligations (all must be addressed in Wave-59 PR):
- M-001: sync docs/adr/0010-ethernet-ip-cip-stream-dispatch.md to .factory ADR-010 (field-count 10→6 line ~598; Decision-9 eprintln! wording line ~697)
- WAVE59-E2E-001: combined e2e test arming HTTP+TLS+Modbus+DNP3+ENIP, port-44818 traffic through reassembler→dispatcher→take_enip_analyzer→reporter (add when STORY-132 lands real findings)
- WAVE59-DEADCODE-001: remove #![allow(dead_code)] on src/analyzer/enip.rs when STORY-132 wires parse functions into on_data frame-walk

**On resume — REPORT TO HUMAN for wave-59 approval before dispatching STORY-132.** D-231 cadence: wave-by-wave human checkpoint required before each wave dispatch.

### Locked design facts (do not re-derive on resume)

ENIP header LITTLE-endian (`from_le_bytes`). `is_valid_enip_frame` single-arg (command-only). `EnipCommandClass` 10 payloadless variants. `CipServiceClass` 15 (0x0A=MultipleServicePacket). `CipHeader={service,request_path}`. `CpfItem={type_id,data}`. `general_status`=byte-2 on 0x00B2 responses. 0x00B2-only CIP detection (0x00B1 deferred v0.12.0). Write-burst default 50 / error-burst 5 strict `>` (51st/6th); both CLI-overridable. T0814 windowed >=3/300s; carry-overflow runs `check_t0814` BEFORE latching `is_non_enip`. `command_counts` SINGLE site=frame-walk (BC-2.17.016 PC-0, counts all incl Unknown). `process_pdu` owns `pdu_count`. `flows_analyzed`→`on_flow_close`. Summary canonical key `parse_errors`. MAX_ENIP_CARRY_BYTES=600, MAX_FINDINGS=10000. MITRE pin ics-attack-19.1. EMITTED 17→20, SEEDED 25→28, catalogue-only 8. Counters u64, window timestamps u32 seconds.

Story input-hashes (verified): STORY-130 272738c→e3c0a6a (D-237 H-001 rehash; D-236 ADR-010 Decision-9 eprintln! fix not propagated in D-236 burst), 131 a119157 (refreshed D-236), 132..138 STALE — pending F4 per-story refresh (ADR-010 M-1 fix is an input to all; do NOT refresh until each story's delivery wave).

### RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — BLOCKING. Do not proceed until PASS.
2. Read `.factory/STATE.md` + `cycles/feature-enip-v0.11.0/cycle-manifest.md` in full.
3. Verify: `git rev-parse --short develop` == `edce3bd` AND `git rev-parse --short main` == `0cbe922`.
4. Verify: `gh pr list` (expect Dependabot #311 open, non-blocking; PRs #317+#318 MERGED).
5. **REPORT TO HUMAN** — Wave-59 approval required before dispatching STORY-132 (D-231 cadence).
6. On approval: create STORY-132 worktree from develop `edce3bd`, branch `worktree-issue-316-story-132-cip-parse`, then begin Red Gate. Deliver docs/adr/0010 M-001 fix within the same PR (sync "10 fields"→"6 fields" line ~598 and Decision-9 eprintln! wording line ~697 in public docs/adr/0010-ethernet-ip-cip-stream-dispatch.md).

### Remaining F4 work (waves 59-61)

Wave 58: COMPLETE (STORY-130+131 merged, D-237).
Wave 59: STORY-132 (CPF/CIP parse + VP-032 Sub-D + M-001 docs/adr/0010 sync + WAVE59-E2E-001 + WAVE59-DEADCODE-001), STORY-133 (MITRE seeding + VP-007 6-part atomic burst) — pending human approval.
Wave 60: STORY-134/135/136/137. Wave 61: STORY-138.

F4 obligations (remaining waves): 12 pcap fixtures (HS-110..122 minus HS-121 which is synthetic); BC frontmatter input-hash writes; STORY-133 EMITTED/SEEDED baseline reverify vs live src/mitre.rs; docs/adr/0010-ethernet-ip-cip-stream-dispatch.md public copy sync (M-001 — deliver in STORY-132 PR).

STORY-132..138 remain STALE — to be refreshed as F4 obligation per story delivery.

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
| Feature EtherNet/IP + CIP — F4 | **IN-PROGRESS — Wave 58 FULLY CONVERGED & CLOSED (D-238): STORY-130+131 merged @edce3bd; 1955 tests green; per-story 3/3 each; wave-level 3/3 clean (0 HIGH/CRITICAL). Wave 59 pending human checkpoint.** | STORY-130: merged PR #317 @235ae60; 3/3 clean passes; input-hash e3c0a6a. STORY-131: merged PR #318 @edce3bd; 3/3 clean passes; input-hash a119157. Wave-level adversarial: `0→0→0` (BC-5.39.001 MET). Consistency audit H-001+L-001 fixed. M-001+WAVE59-E2E-001+WAVE59-DEADCODE-001 deferred to STORY-132. stories_delivered=80. Convergence trajectory: cycles/feature-enip-v0.11.0/convergence-trajectory.md. |

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

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- STORY-INDEX.md authoritative (91 stories / 61 waves / 592 pts — v2.8). STORY-130+131 completed (Wave 58, D-237). STORY-132..138 draft (waves 59-61). Input-hashes: STORY-130 e3c0a6a, STORY-131 a119157 (MATCH); STORY-132..138 STALE (pending F4 delivery).
- F6 fuzz obligation: `parse_cip_header` + `parse_cpf_items` cargo-fuzz (F-P9-002, from F2 adversarial pass 9).
- Deferred LOW (non-blocking): BC-2.17.010 Description "per-occurrence" → fix to one-shot (PO); dep-graph STORY-133→137 T0814 rationale prose imprecision.
- Issues: #104/#102/#64 CLOSED; all actions SHA-pinned; dtolnay/rust-toolchain @stable/@nightly exempted.
