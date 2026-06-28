---
pipeline: FEATURE-MODE
phase: F7
phase_status: "STORY-140 DNP3 (EC-X1/EC-X2) MERGED to develop (PR #335 squash, b6d7a01). Both EC-X1/EC-X2 fixes now on develop (ENIP #334 + DNP3 #335). v0.11.0 READY TO RELEASE — HELD pending human go-ahead."
product: wirerust
mode: feature-mode
timestamp: 2026-06-28T18:00:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"

# Ground-truth HEADs (verified D-288 — 2026-06-28)
develop_head: b6d7a01
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

# Active worktrees
worktree_scratch: ".worktrees/enip-edgecase-verify @ fd0c7f3 [scratch/enip-edgecase-verify] — keep for reference"
worktree_orphan: ".worktrees/enip-f6-hardening @ 447da07 [test/enip-f6-fuzz-harnesses] — orphan, safe to remove"

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 89
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: "Wave 63 — STORY-140 DNP3 MERGED (b6d7a01). v0.11.0 READY TO RELEASE (ENIP+DNP3 both on develop), HELD pending human go-ahead."

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

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. STORY-139 MERGED (D-277, develop `99a06f4`, PR #334). STORY-140 DNP3 sibling fix MERGED to develop via PR #335 squash (b6d7a01, D-288). BOTH EC-X1/EC-X2 release-blockers resolved on develop. v0.11.0 READY TO RELEASE — HELD pending explicit human go-ahead (D-260/D-278).**

**HUMAN DIRECTIVE (D-260/D-278): HALT — do NOT run the release pipeline / tag / publish without explicit human go-ahead. v0.11.0 ships ENIP+DNP3 together; both now on develop. Release requires explicit human instruction.**

Latest release: v0.10.0 (main `0cbe922`). develop=`b6d7a01`. stories_delivered=89. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818 + DNP3 sibling EC-X1/EC-X2 fix). GitHub issue #316. v0.11.0 HELD pending human release go-ahead.

Spec versions: BC-INDEX v1.85 (332 on disk / 331 active; SS-17=26 BCs + EC-010 amendments). ARCH-INDEX v1.8. VP-INDEX v2.13 (36 VPs: VP-033..036 added). PRD v1.36. STORY-INDEX v2.9 (92 stories / 62 waves). epics.md v1.8 (E-20, Wave 62).

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603).
- Do NOT re-deliver STORY-130..138 — all MERGED (D-234..D-259). stories_delivered=87 (pre-STORY-139).
- Do NOT re-merge ENIP E2E real-pcap PR #333 — MERGED (D-269) fd0c7f3.
- Do NOT re-run F5/F6/F7 for Wave 61 — F5 CONVERGED (D-263), F6 PASSED (D-265), F7 HUMAN-APPROVED (D-267).
- Do NOT re-create RULING-EDGECASE-001 — exists at `.factory/cycles/feature-enip-v0.11.0/RULING-EDGECASE-001-direction-and-clock.md`.
- Do NOT re-run F2/F3/F4/F5/F6/F7 for STORY-139 — all COMPLETE + MERGED (D-271..D-277).
- Do NOT re-baseline input-hashes for STORY-130..139 — done @4f4dc76 (D-276, MATCH=89/STALE=0); re-baselined again @c99d7b6 after ADR-010 amendment.
- Do NOT re-deliver STORY-139 — MERGED (D-277, develop 99a06f4). PR #334. CI 11/11 green.
- Do NOT cut v0.11.0 — HELD (D-260/D-278); ENIP+DNP3 ship together on explicit human go-ahead.
- Do NOT re-run STORY-140 F4 — GREEN @1dda26b (D-279).
- Do NOT re-introduce the `< 3` frame-walk guard or drop parse_errors counting (D-281 regression).
- Do NOT restore wrapping_sub at block-timeout (D-280).
- Do NOT re-run STORY-140 F5 — CONVERGED @e16ee56 (D-282).
- Do NOT re-run STORY-140 F6 — PASS @499c778 (D-283). Do NOT re-chase the 3 Group-D MAX_FINDINGS DoS-cap mutants (accepted impractical). fuzz_dnp3_parse 4-arg fix committed b40d1d9.
- Do NOT re-run STORY-140 F4-F7 — F7 CONVERGED + human-approved @560efd3 (D-285). Do NOT re-run F7 delta-convergence or human gate.
- Do NOT merge STORY-140 or release v0.11.0 without explicit human go-ahead (D-285). STORY-140 PR #335 OPEN + READY TO MERGE @7169963 — awaiting merge go-ahead only (D-287).
- Do NOT re-open/re-create STORY-140 PR — #335 MERGED b6d7a01 (D-288). Do NOT re-merge STORY-140.
- Do NOT release v0.11.0 without explicit human go-ahead. Repo is squash-only (D-289) — do NOT re-enable merge-commit without human instruction.

### EXACT RESUME POINT — STORY-140 MERGED, v0.11.0 RELEASE HELD

**STORY-140 MERGED to develop @b6d7a01 (D-288). PR #335 squash-merged. 24 files. Worktree .worktrees/dnp3-direction-clock + branch fix/dnp3-direction-and-clock removed (local+remote); 17 stale refs pruned. stories_delivered=88→89. Both EC-X1/EC-X2 release-blockers resolved on develop (ENIP STORY-139 #334 @99a06f4, DNP3 STORY-140 #335 @b6d7a01).**

**ONLY remaining action: v0.11.0 RELEASE — HELD pending explicit human go-ahead.**

Release pipeline (requires explicit human go-ahead): cut release/0.11.0 from develop b6d7a01 → version bump + CHANGELOG → PR to main → tag v0.11.0 → GitHub release. NOTE: repo is squash-only (D-289) — the release PR to main will squash; confirm main handling with human if gitflow merge-commit preferred.

Do NOT release v0.11.0 without explicit human instruction.

### RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file).
3. Verify: develop HEAD is `b6d7a01`; PR #335 MERGED; worktree dnp3-direction-clock GONE.
4. Wait for explicit human go-ahead before running release pipeline.

### Locked design facts (do not re-derive on resume)

ENIP header LITTLE-endian (`from_le_bytes`). `EnipCommandClass` 10 variants. `CipServiceClass` 15. `CipHeader={service,request_path}`. `CpfItem={type_id,data}`. `general_status`=byte-2 on 0x00B2 responses. 0x00B2-only CIP (0x00B1 deferred v0.12.0). Write-burst 50/error-burst 5 strict `>`. T0814 windowed >=3/300s. MAX_ENIP_CARRY_BYTES=600. MAX_FINDINGS=10000. MITRE pin ics-attack-19.1. Counters u64, window timestamps u32 seconds. `EnipFlowState` now has: `carry_c2s`, `carry_s2c` (replacing `carry`), `malformed_window_start_ts` (replacing `window_start_ts`). `on_data` signature: `(flow_key, data, timestamp, direction: Direction)`. 74 call-sites updated to `Direction::ClientToServer` at red-gate.

EC-X1: cross-direction carry splice — confirmed HIGH release-blocker. EC-X2: clock-backwards `wrapping_sub` → window reset — confirmed HIGH release-blocker. Both adjudicated RULING-EDGECASE-001. Repro tests: `.worktrees/enip-edgecase-verify/tests/scratch_ecx1_ecx2_repro.rs`. DNP3 shares both patterns → DRIFT-DNP3-DIRECTION-001 + DRIFT-DNP3-CLOCK-001 — now IN SCOPE v0.11.0 as STORY-140 (human D-278; atomic fix with ENIP).

Story input-hashes: STORY-130 63fac3a, STORY-131 ce92886, STORY-132 c33dff8, STORY-133 661f504, STORY-134 16d03a6, STORY-135 ae2d871, STORY-136 0846e0e, STORY-137 f4c8390, STORY-138 0f60353 (all MATCH). STORY-139 input-hash: 581b0fd (re-baselined 4f4dc76 after BC-2.17.016 additive NOTE; D-276). STORY-140 input-hash: b3a4fd0 (re-baselined a915faa after SS-15 BC v2.x amendments + BC-2.15.014 fix; D-286).

### OPEN ITEMS (backlog — non-blocking unless marked)

| ID | Summary | Status |
|----|---------|--------|
| STORY-139 | EC-X1/EC-X2 fix-delta: per-direction carry + saturating window. | MERGED — develop `99a06f4` (D-277, PR #334) |
| STORY-140 | DNP3 sibling EC-X1/EC-X2 fix (carry-split + saturating_sub). | MERGED — develop `b6d7a01` (D-288, PR #335 squash) |
| DRIFT-DNP3-DIRECTION-001 | DNP3 EC-X1 pattern (carry-direction-split) — sibling of ENIP fix. | RESOLVED + MERGED (D-288, STORY-140 b6d7a01) |
| DRIFT-DNP3-CLOCK-001 | DNP3 EC-X2 pattern (wrapping_sub clock reset) — sibling of ENIP fix. | RESOLVED + MERGED (D-288, STORY-140 b6d7a01) |
| DEVELOP-BRANCH-PROTECTION | develop has no GitHub branch-protection (404); squash-required enforced via allowed-merge-methods only; consider adding required-status-checks protection. | BACKLOG (D-289) |
| BC-2.15.014-LINE-CITATION | EC-006 + v2.0 changelog stale source-line citation 984-991 → verified post-STORY-140 line 1173-1200. | RESOLVED (D-286, commit eb406d1, no version bump) |
| INPUT-HASH-DRIFT-STORY-140 | Input-hash mechanical re-baseline after SS-15 BC v2.x amendments + BC-2.15.014 fix. | RESOLVED (D-286, commit a915faa): STORY-140 d498e66→b3a4fd0; MATCH=90/STALE=0. |
| F-W60-002 | `bytes_received` BC-2.17.016 v1.1→v1.2 clarification (PC-5 exemption + Invariant 7). | DEFERRED — cycle close |
| BC-PROSE-LOW-RESIDUALS | BC-2.17.001 Inv-4 + prd.md singular `carry`; BC-2.17.018 PC-1 singular `carry`; VP-034 title-label drift. | OPEN — cycle close |
| ENGINE-PROPAGATION-GREP-GATE-001 | Mechanical changed-value sibling-grep gate. Human decision before cycle CLOSE. | OPEN — human review |
| SS-17-BC-INPUT-HASH-BACKFILL | BC-2.17.007+ carry `input-hash: TBD`. Evaluate at cycle close. | DEFERRED — cycle close |
| GREEN-DOC-TENSE-GATE-PATTERN-GAP-001 | `bin/check-green-doc-tense` misses "These tests MUST FAIL" / "will pass once" forms. | OPEN — engine backlog |
| DEPENDABOT-311 | PR #311 (actions/checkout 6.0.3→7.0.0) unreviewed. | OPEN — human triage |
| DEPENDABOT-325 | Dependabot PR #325 unreviewed. | OPEN — human triage |
| ENIP-UNCONNECTED-SEND-UNWRAP-001 | Metasploit Unconnected_Send/0x52 unwrap for real-world attack detection. DF-VALIDATION-001-gated. | DEFERRED — v0.12.0 |
| F-138-P1-002 | BC-2.17.016 PC-0 wording ambiguity (non-blocking). | OPEN — cycle close |
| F6-MUTANTS-FULL-RUN | Confirm 0-missed on full 241-mutant run (21 caught/0 missed at F6 gate). | OPEN — human confirm at F7 |
| INPUT-HASH-PREEXIST-DRIFT-14 | 14 non-ENIP released stories (STORY-002/003/004/005/071/076-080/100/101/120/129) were pre-existing input-hash-stale before the BC-2.17.016 edit; swept clean in re-baseline 4f4dc76. Likely benign (prior-cycle BC doc-edits); confirm no real un-regenerated drift. | BACKLOG — audit (DF-VALIDATION-001-gated) |
| INPUT-HASH-3-STRUCTURAL-ERRORS | STORY-001 (retired-BC ref), STORY-091, STORY-121 (missing inputs: blocks) report compute-input-hash ERROR; pre-existing, unrelated to this edit. | BACKLOG |
| ENIP-CARRY-CAP-V0.12.0-REDESIGN | BC-2.17.016 PC-4 carry-overflow cap is unreachable dead code (RULING-137-002); v0.12.0 quarantine-mechanism redesign should make it reachable or remove it. | DEFERRED — v0.12.0 |
| EDGE-CASE-HUNT-COVERAGE-IDEAS | SendUnitData path, multi-0x00B2, ForwardOpen response suppression tests (from EC hunt). | BACKLOG — optional |
| STORY-137-UNSAFE-SPLIT-BORROW | [LOW] unsafe split-borrow in process_pdu. Sound; consider safe refactor. | OPEN — v0.12.0 |
| D4-001 | BC-2.17.018 Architecture Anchors missing explicit malformed_window_start_ts row (doc-completeness). | BACKLOG — next BC sweep |
| D6-001 | consistency-validator reported VP-025..031 body files apparently absent from disk (7 files); pre-existing, NOT introduced by this burst; likely validator directory-path artifact since v0.9.3 owns those VPs and is released+closed. | BACKLOG — verify (DF-VALIDATION-001-gated) |
| RULING-EDGECASE-001-STALE-ANCHORS | ruling §2.4 cites pre-fix enip.rs line numbers (1312/1129/821) + §1.3/§2.4 use old field name malformed_window_start; immutable-ruling forensic refs, no action. | OBSERVATION — no action |

All GitHub-issue creation DF-VALIDATION-001-gated.

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
| Feature ARP (E-16) + v0.7.0 | RELEASED 2026-06-16 | STORY-111..115; VP-024 LOCKED. |
| E-17 ARP QinQ/MACsec + v0.7.1 | RELEASED 2026-06-17 | STORY-116/117; tag v0.7.1 |
| E-18 finding-collapse + v0.8.0 | RELEASED 2026-06-17 | SS-11=29 BCs. |
| E-18/E-8 STORY-119 + v0.9.0 | RELEASED 2026-06-19 | 293 BCs; tag v0.9.0. |
| v0.9.1/v0.9.2 patches | RELEASED 2026-06-19 | Doc/help + DNP3 determinism. |
| Feature pcapng-reader + v0.9.3 | RELEASED + CLOSED 2026-06-22 (D-201) | 10 new BCs, VP-INDEX v2.10. |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking. |
| Feature mitre-json-names + v0.9.4 | RELEASED + CLOSED 2026-06-23 (D-217) | BC-INDEX v1.71. |
| Fix cycle fix-pc-013-014-015 + v0.10.0 | CONVERGED + RELEASED + CLOSED 2026-06-24 (D-226) | tag v0.10.0 0cbe922. |
| Feature EtherNet/IP — F1/F2/F3 (Wave 58-61) | CONVERGED + HUMAN-APPROVED (D-228..D-231) | 26 BCs, 9 stories STORY-130..138. |
| Feature EtherNet/IP — F4..F7 (Wave 61) | F7 HUMAN-APPROVED + HOLDOUT EVAL PASS + ENIP E2E MERGED (D-267..D-269). HOLD AT RELEASE. | 2093/0/81 GREEN @fd0c7f3. |
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F2 | DONE + RE-VALIDATED CLEAN | BC-2.17.016 v2.0/008 v1.3/012 v1.2/018 v1.1; VP-033/034; BC-INDEX v1.85; VP-INDEX v2.12 (34). |
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F3 | DONE | STORY-139 authored @a2db4f3; input-hash 759464a MATCH. |
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F4 | DONE @3c688ff (179 GREEN / 0 RED) | `.worktrees/enip-direction-clock` @3c688ff. |
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F5 | CONVERGED @0607b82 (2107/0) | 6 adversary passes; findings 4→3→2→0; consistency audit CONSISTENT. |
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F6 | PASS @cee85c0 (Kani 36/37, fuzz 0-crash, mutation 23/23 in-scope, 3 equivalent; 2112/0) | Input-hash re-baseline 4f4dc76 (MATCH=89/STALE=0). |
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F7 + MERGE | **MERGED to develop — PR #334 (D-277, 99a06f4). stories_delivered=88. CI 11/11 green.** | pr-reviewer 0 findings; security-reviewer 0 CRITICAL/0 HIGH. ADR-010 D4 amended; input-hash re-baselined c99d7b6. |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F1 | DONE | RULING-DNP3-SIBLING-001 (88d41fd). Identified DNP3 carry-split + saturating_sub regions in dnp3.rs / SS-15. |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F2 | DONE | 4 SS-15 BCs amended (e04809d: BC-2.15.016 v2.0/010 v1.8/014 v2.1/015 v2.0); ADR-007 amended (1e39373); VP-035/VP-036 + indexes (ab3c270, VP-INDEX v2.13/36 VPs). |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F3 | DONE | STORY-140 authored (6d6e3a3, E-15 Wave 63, input-hash d498e66). |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F4 | **F4 GREEN @1dda26b (2128/0)** | Worktree from develop 99a06f4. 208 call-sites threaded. parse_errors-resync regression caught+fixed (D-281). clippy/fmt clean; 0 wrapping_sub; singular carry gone; resolve_master_ip gone. |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F5 | **F5 CONVERGED @e16ee56 (2129/0)** | 6 fresh-context adversary passes (findings 2 MED→3 MED+1 LOW→0→1 LOW→2 LOW→0): passes 3/4/5 + confirming pass all zero-HIGH/CRITICAL/mis-anchor. Commit chain: 1dda26b→ac8f2b3→5bc6caa→9972037→e16ee56. 24/24 BC clauses; VP-035/036 genuine non-vacuous proptests; AC-140-002b discriminating. |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F6 | **F6 PASS @499c778 (2168/0)** | Kani 36/37 (all 4 DNP3 framing harnesses PROVED + non-vacuous; 1 orthogonal reader harness still-solving). cargo-fuzz fuzz_dnp3_parse 5.18M execs/0 crashes (4-arg fix b40d1d9). Mutation: Group A/B/C CAUGHT (two remediation bursts: 7bcbbaa 28 tests → verifier re-run found 11 Group-A survivors → 499c778 11 targeted tests VERIFIED via cargo-mutants); 3 Group-D MAX_FINDINGS DoS-cap accepted impractical. VP-035 2/2, VP-036 6/6. |
| Feature DNP3 sibling EC-X1/EC-X2 (Wave 63) — STORY-140 F7 + MERGE | **MERGED to develop — PR #335 squash (D-288, b6d7a01). stories_delivered=89.** | pr-reviewer APPROVE (0 findings); security-reviewer APPROVE (0 CRITICAL/HIGH/MEDIUM, 4 LOW non-blocking). Worktree .worktrees/dnp3-direction-clock + branch fix/dnp3-direction-and-clock removed. STORY-140 input-hash b3a4fd0. v0.11.0 READY TO RELEASE — HELD pending human go-ahead. Repo squash-only policy set (D-289). |

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md`
D-206..D-217: `cycles/feature-mitre-json-names/decisions-archive.md`
D-219..D-226: `cycles/fix-pc-013-014-015/decisions-archive.md`
D-228..D-269: `cycles/feature-enip-v0.11.0/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-270 | SESSION PAUSE at F4 implementation of EC-X1/EC-X2 fix-delta (STORY-139, Wave 62). Worktree `.worktrees/enip-direction-clock` @63c119a (`fix/enip-direction-and-clock`, base `fd0c7f3`): red-gate complete — crate compiles, clippy clean, 9 STORY-139 tests RED, 170 existing GREEN. 3 stub points pending: (1) per-direction carry select (EC-X1/BC-2.17.016 v2.0); (2) direction-based src_ip/dest_ip (AC-139-002); (3) saturating_sub + strict `> 300` window (EC-X2/BC-2.17.008/012/018). Durable resume checkpoint written. factory-artifacts HEAD at time of pause: see `git -C .factory log -1 --format='%h'`. | 2026-06-27 |
| D-271 | F4 implementation of STORY-139 (EC-X1/EC-X2 fix-delta) COMPLETE on `.worktrees/enip-direction-clock` branch `fix/enip-direction-and-clock`, impl commit `3c688ff` (base red-gate `63c119a`). All 3 stub points filled: (1) per-direction carry select/stash carry_c2s/carry_s2c [EC-X1/BC-2.17.016 v2.0 Inv-7]; (2) direction-based src_ip inline, resolve_enip_client_ip removed [AC-139-002/DRIFT-ENIP-DIRECTION-001]; (3) saturating_sub + strict >300 window [EC-X2/EC-X4/BC-2.17.008/012/018]. Green gate: cargo test --all-targets 179 GREEN / 0 RED; clippy -D warnings clean; fmt clean. Compliance verified: zero wrapping_sub, zero singular carry:, resolve_enip_client_ip removed (comments only). STEP 0 red-gate baseline confirmed 9 RED / 170 GREEN. Not yet pushed/PR'd/merged. Next: F5 scoped adversarial. | 2026-06-27 |
| D-272 | STORY-139 F4→F5 fix-burst chain on worktree enip-direction-clock: impl 3c688ff → F-139-01 doc + green-doc-tense 69cbf18 → carry-clear removal 046cc41 → operator-pin test rewrite 573a977 → VP-033/034 real proptests + resolve_enip_client_ip prose sweep d0b7b78 → EC-X1 false-positive guard assertion + STORY-137 break/continue doc-tense fc29f2f → summarize_drainage doc-tense 0607b82. F-139-02 (malformed-window carry-clear) ADJUDICATED by architect (RULING-EDGECASE-001 addendum): REMOVE carry-clear (restore BC-2.17.018 PC-5 3-reset minimal-fix) + rewrite test_malformed_window_operator_pin_boundary to oversized-declared-frame path (no carry residue); no BC/AC amendment. O-3 (AC-139-002 direction:Some) ADJUDICATED: out-of-scope per RULING §1.4 (per-flow aggregates); AC-139-002 amended to state direction field NOT populated (deferred v0.12.0). F-139-03 dispatcher path (stream_dispatcher.rs→src/dispatcher.rs) fixed in STORY-139 + ruling. | 2026-06-27 |
| D-273 | STORY-139 F5 scoped-adversarial CONVERGED. 6 fresh-context adversary passes (findings decayed 4→3→2→0): passes 4/5/6 + final confirming pass all CLEAN (zero HIGH/CRITICAL, zero mis-anchors). Post-fixburst consistency audit (consistency-validator, 6 dimensions) verdict CONSISTENT — malformed-window code matches BC-2.17.018 PC-5 exactly (3 resets, no carry-clear), 2 remaining carry-clears are cap-overflow only (BC-2.17.016 PC-4), input-hash 759464a MATCH. OBS-1 (proptest_ prefix on 2 deterministic VP-034 guards) ADJUDICATED ACCEPTED: names spec-cited by AC-139-005 + VP-034 spec, annotated GREEN-BY-DESIGN, renaming would churn 3 artifacts/2 branches. v0.11.0 stays HELD (D-260) until STORY-139 merges. | 2026-06-27 |
| D-274 | STORY-139 F6 targeted hardening PASS. Worktree commits since F5: 25f8b4a (5 boundary-hardening tests killing carry-direction-select 948 + error/write-window boundary mutants 1152/1336), cee85c0 (cap-check doc-comment corrected). Kani: 36/37 proved (all 5 ENIP harnesses PROVED + non-vacuous; struct/signature change broke nothing; the 1 outstanding harness vp025_timestamp_totality_base10 is in reader.rs — non-delta, still-solving CBMC SAT, NOT a regression). cargo-fuzz fuzz_enip_cip_parse: 14.99M execs / 0 crashes. cargo-mutants delta: 23/28 caught; of 5 survivors, 3 (enip.rs:953 ×2 + 967, the carry-overflow cap+clear) are PROVEN-EQUIVALENT permanent survivors and 2 (465 STORY-138, 1335 STORY-135) are pre-existing out-of-scope. In-scope killable kill-rate = 23/23 = 100%. Regression 2112/0; VP-033 2/2, VP-034 6/6. | 2026-06-27 |
| D-275 | ARCHITECT RULING (RULING-EDGECASE-001 addendum, 3rd adjudication): the ENIP carry-overflow cap (enip.rs:953/967, BC-2.17.016 PC-4, MAX_ENIP_CARRY_BYTES=600) is STRUCTURALLY UNREACHABLE — RULING-137-002 §1 proves max carry after any on_data call is 599 bytes (all 3 frame-walk paths exhausted). The 3 cargo-mutants survivors are CONFIRMED EQUIVALENT (excluded from kill denominator). RULING-EDGECASE-001 §3's Path-A reachability claim is RETRACTED (was a self-correction that was itself wrong; RULING-137-002 §1 governs). Resolution (b1): keep cap-check as belt-and-suspenders dead code with corrected comment (impl commit cee85c0); BC-2.17.016 PC-4 additive errata NOTE + EC-004/canonical-test-vector annotated superseded (PO commit a748971, no version bump); orchestrator escalated rather than accept test-writer's unverified equivalent-mutant claim — proof confirmed it AND surfaced the ruling contradiction. | 2026-06-27 |
| D-276 | Input-hash mechanical re-baseline after BC-2.17.016 additive NOTE (story-writer commit 4f4dc76, factory-artifacts). STORY-139: 759464a→581b0fd. `bin/compute-input-hash --write --scan` rewrote 24 stale stories total → MATCH=89 STALE=0. NOTE: only STORY-130..139 relate to the BC-2.17.016 edit; the other 14 (STORY-002/003/004/005/071/076-080/100/101/120/129) were PRE-EXISTING stale from prior released-cycle BC doc-edits, swept clean in the same re-baseline (benign — all released/closed v0.1-v0.10). 3 pre-existing structural ERRORs (STORY-001 retired-BC-ref, STORY-091, STORY-121 missing inputs blocks) unrelated → backlog. No story versions bumped; only input-hash fields touched. | 2026-06-27 |
| D-277 | STORY-139 (ENIP EC-X1/EC-X2 detection-correctness fix) MERGED to develop via PR #334 (merge commit 99a06f4; develop HEAD 99a06f4). Both reviewers APPROVE (pr-reviewer 0 findings; security-reviewer 0 CRITICAL/0 HIGH, 1 MEDIUM = documented-unreachable carry-cap per RULING-137-002 non-blocking, 2 LOW pre-existing). CI 11/11 green (run 28302628291). Pre-merge: ADR-010 Decision 4 amended (worktree 5d5181f in PR + .factory da8adf5); input-hash re-baseline c99d7b6 (STORY-130..139; STORY-139=16e5c27). Worktree enip-direction-clock + branch fix/enip-direction-and-clock removed. F7 human-approved. stories_delivered=88. | 2026-06-27 |
| D-278 | HUMAN DIRECTIVE (F7 gate, Q2): fix DNP3 ATOMICALLY with ENIP for v0.11.0 — DRIFT-DNP3-DIRECTION-001 (carry splice) + DRIFT-DNP3-CLOCK-001 (wrapping_sub clock reset) are NO LONGER deferred to v0.12.0; they are now IN SCOPE for v0.11.0 as a sibling fix (new STORY-140, full F1-F7 cycle, mirroring RULING-EDGECASE-001 applied to dnp3.rs / SS-15). HUMAN DIRECTIVE (Q4): v0.11.0 STAYS HELD after the ENIP merge — separate explicit release go-ahead required (extends D-260). v0.11.0 ships ENIP + DNP3 together once both land on develop and human approves release. | 2026-06-27 |
| D-279 | STORY-140 DNP3 F1-F4 complete. F1 RULING-DNP3-SIBLING-001 (88d41fd). F2: 4 SS-15 BCs amended (e04809d: BC-2.15.016 v2.0/010 v1.8/014 v2.1/015 v2.0), ADR-007 amended (.factory 1e39373), VP-035/VP-036 registered + indexes (ab3c270, VP-INDEX v2.13/36 VPs). F3 STORY-140 authored (6d6e3a3, E-15 Wave 63, input-hash d498e66). F4 worktree .worktrees/dnp3-direction-clock from develop 99a06f4: red-gate scaffolding 7a225aa (208 test call-sites threaded) + RED tests b761033, impl af66b9d, block-timeout saturating_sub a5ca673, test fixes 28b5673, regression-fix 1dda26b. 2128 GREEN/0 RED. clippy/fmt clean; 0 live wrapping_sub; singular carry gone; resolve_master_ip gone. | 2026-06-27 |
| D-280 | ARCHITECT adjudications during STORY-140 F4 (RULING-DNP3-SIBLING-001 author): (1) AC-140-002 test was DEFECTIVE (master placed on outstation port 20000) → rebuilt to standard topology (outstation:20000, master:54321 ephemeral), C2S→source=master=upper_ip; port-heuristic+direction formula correct (mirrors ENIP). (2) block-timeout (BC-2.15.014/T1691.001) → saturating_sub (consistent with RULING-EDGECASE-001 §2.2 ENIP precedent); old STORY-109 AC-014 test_pending_request_timeout_wrapping_sub SUPERSEDED → renamed test_pending_request_timeout_no_spurious_fire_on_rollover_or_backwards_ts (asserts no-spurious-fire + forward-clock companion); BC-2.15.014 v2.1 EC-009 stands; STORY-109.md:133 citation updated (59e7688). | 2026-06-27 |
| D-281 | REGRESSION caught during STORY-140 F4 (orchestrator refused to accept test-writer's 'pre-existing' dismissal; devops bisect confirmed develop@99a06f4 GREEN vs worktree RED). Carry-split refactor changed the dnp3 frame-walk loop guard `< 3` → `< 10`, silently dropping parse_errors increments in the junk-at-clean-boundary / LENGTH-gate resync path (3 f5_resync_accounting tests under-counted parse_errors by 1). Fixed (1dda26b) via did_process_in_this_call context tracking (dnp3.rs:442/455/495/535) — restores one parse_error per structural event without breaking AC-140-001 partial-stash. No tests dropped (2128 = develop 2112 + 16 new). | 2026-06-27 |
| D-282 | STORY-140 DNP3 F5 scoped-adversarial CONVERGED. Worktree commit chain after F4-green: 1dda26b → ac8f2b3 (5 stale wrapping_sub doc-comments) → 5bc6caa (VP-036 Sub-B/C made genuine on_data-driven proptests, F-140-002) → 9972037 (Sub-D rollover values/rationale corrected now_ts=500→wrapping=506, EC-008 test renamed saturating_sub, correlation prose, AC-140-002b discriminating ServerToClient src-ip case) → e16ee56 (EC-008 header + Sub-C operator-pin proptest doc honesty). 6 fresh-context adversary passes (findings 2 MED → 3 MED+1 LOW → 0 → 1 LOW → 2 LOW → 0): passes 3/4/5 + confirming pass all zero-HIGH/CRITICAL/mis-anchor; all MED/LOW resolved. Spec-prose fixes on factory-artifacts: VP-036/STORY-140 Sub-D rationale (7722617), STORY-109 AC-014 citation supersession (59e7688). 24/24 BC clauses covered; VP-035/036 genuine non-vacuous proptests; did_process_in_this_call regression-fix confirmed sound; AC-140-002b genuinely discriminates direction from port-heuristic. | 2026-06-27 |
| D-283 | STORY-140 DNP3 F6 targeted hardening PASS @499c778. Kani: 36/37 proved (all 4 DNP3 framing harnesses PROVED + non-vacuous; struct/signature change broke nothing; 1 orthogonal still-solving reader harness). cargo-fuzz fuzz_dnp3_parse: 5.18M execs/0 crashes (+ pre-existing fuzz-harness on_data 4-arg signature gap found+fixed+committed b40d1d9 — DF-SIBLING-SWEEP miss in STORY-140's call-site sweep; now drives both directions). Mutation delta: first remediation (7bcbbaa, 28 tests) claimed structural-kill but VERIFIER re-run found 11 Group-A survivors still missed; orchestrator routed back; second remediation (499c778, 11 targeted tests in dnp3_f6_story140_group_a_survivors.rs) VERIFIED via actual cargo-mutants re-run — all 11 Group-A (carry-cap arithmetic 409/410, resync byte-walk sync-match 467/479/511/555) CAUGHT, plus Group B (8 src_ip attribution) + Group C (1 window boundary) caught. Only 3 Group-D MAX_FINDINGS DoS-cap off-by-one (1504/1538/1597) remain — accepted impractical (needs 10k findings; pre-existing, mirrors modbus.rs). Regression 2168/0; VP-035 2/2, VP-036 6/6. | 2026-06-27 |
| D-284 | PROCESS NOTE: STORY-140 F6 mutation gate required TWO orchestrator verification interventions — (1) refused 'pre-existing' dismissal of 3 f5_resync_accounting RED tests → devops bisect proved STORY-140 regression (D-281); (2) refused test-writer 'structurally killed' claim → formal-verifier cargo-mutants re-run found 11 Group-A survivors. Both caught real gaps. Reinforces DF-ADVERSARY-TOOLCHAIN-PAIRING-001 / verify-don't-trust on mutation + regression claims. | 2026-06-27 |
| D-285 | STORY-140 DNP3 F7 delta-convergence CONVERGED (consistency-validator: 5/5 convergence dims + 6/6 consistency, 1 minor non-blocking BC-2.15.014 line-citation finding). docs/adr/0007 develop-tree copy amended on worktree (560efd3) — both ADR-007 copies now consistent with code. HUMAN GATE outcome: (Q1) APPROVE STORY-140 convergence but HOLD MERGE — separate explicit go-ahead required before merging fix/dnp3-direction-and-clock → develop; (Q2) v0.11.0 STAYS HELD; (Q3) fix both backlog items now. STORY-140 worktree @560efd3, 2168/0 green, ready to merge on go-ahead. | 2026-06-28 |
| D-286 | BACKLOG FIXES (human-approved, both done): (1) BC-2.15.014 EC-006 + v2.0 changelog stale source-line citation 984-991 → verified post-STORY-140 line 1173-1200 (commit eb406d1, no version bump, no logic change). (2) Input-hash mechanical re-baseline after SS-15 BC v2.x amendments + the BC-2.15.014 fix (commit a915faa): STORY-140 d498e66→b3a4fd0, STORY-106..110 rebaselined; full scan MATCH=90 STALE=0 (3 pre-existing ERROR rows STORY-001/091/121 unrelated). STORY-140 authoritative input-hash is now b3a4fd0. | 2026-06-28 |
| D-287 | STORY-140 PR #335 (https://github.com/Zious11/wirerust/pull/335) OPENED targeting develop; branch fix/dnp3-direction-and-clock pushed (HEAD 7169963 — 560efd3 + per-AC demo-evidence commit). Human chose 'Push + open PR, hold merge': pr-manager ran lifecycle steps 1-7 and STOPPED before the squash-merge (DF-PR-MANAGER-COMPLETE-001 overridden by explicit human hold D-285). Gates: cargo test 2168/0, clippy/fmt clean; CI 11/11 green; pr-reviewer APPROVE (0 findings); security-reviewer APPROVE (0 CRITICAL/HIGH/MEDIUM, 4 LOW non-blocking, CWE-191 wrapping_sub-underflow confirmed resolved at all 8 sites); dependency STORY-139 #334 MERGED. READY TO MERGE — merge command held: `gh pr merge 335 --squash --delete-branch`. Branch + worktree intact. | 2026-06-28 |
| D-288 | STORY-140 (DNP3 EC-X1/EC-X2 sibling fix) MERGED to develop via PR #335 — SQUASH merge commit b6d7a01 (develop ff-pulled 99a06f4→b6d7a01, 24 files). Human gave merge go-ahead lifting D-285 hold. Worktree .worktrees/dnp3-direction-clock + branch fix/dnp3-direction-and-clock removed (local+remote); 17 stale refs pruned. stories_delivered 88→89. Both EC-X1/EC-X2 release-blockers now resolved on develop (ENIP STORY-139 #334 @99a06f4, DNP3 STORY-140 #335 @b6d7a01). | 2026-06-28 |
| D-289 | REPO MERGE POLICY: per human directive, repo Zious11/wirerust set to SQUASH-ONLY (allow_squash_merge=true, allow_merge_commit=false, allow_rebase_merge=false, delete_branch_on_merge=true) — squash-and-merge now REQUIRED for develop (and repo-wide). Supersedes the prior merge-commit-only policy (STORY-139 #334 used a merge commit under the old policy). FLAG: this is repo-wide, so the upcoming release/0.11.0 → main PR will also squash (diverges from CLAUDE.md gitflow merge-commit release flow — develop↔main sync should compare by tag/cherry, not commit ancestry). Note: develop currently has NO GitHub branch-protection rules (404 Branch not protected) — squash-required is enforced via allowed-merge-methods only; CI is workflow-side. Consider adding develop branch protection (required status checks) as a future hardening item. | 2026-06-28 |

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- STORY-INDEX.md authoritative (92 stories / 63 waves — v2.9). STORY-130..140 completed + merged. stories_delivered=89. STORY-140 (DNP3 sibling fix, Wave 63) MERGED @b6d7a01 (D-288). v0.11.0 READY TO RELEASE — HELD pending human go-ahead.
- Repo squash-only policy set (D-289). Worktree .worktrees/dnp3-direction-clock + branch fix/dnp3-direction-and-clock removed.
- F6 fuzz harness (F-P9-002) MERGED — PR #332 @f17d270 on develop (D-265).
