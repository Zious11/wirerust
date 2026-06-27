---
pipeline: FEATURE-MODE
phase: F4
phase_status: "F4 RED-GATE DONE — EC-X1/EC-X2 fix-delta (STORY-139) in progress on worktree enip-direction-clock @63c119a — 9 RED / 170 GREEN"
product: wirerust
mode: feature-mode
timestamp: 2026-06-27T22:00:00Z

# Release chain (latest)
released_version: v0.10.0
released_at: "2026-06-24"
release_tag: v0.10.0
release_commit: 0cbe922
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.10.0
prior_released_version: v0.9.4
prior_released_at: "2026-06-23"

# Ground-truth HEADs (verified D-270 — 2026-06-27)
develop_head: fd0c7f3
main_head: 0cbe922
factory_artifacts_head: (run `git -C .factory log -1 --format='%h'`)

# Active worktrees (verified D-270)
worktree_fix: ".worktrees/enip-direction-clock @ 63c119a [fix/enip-direction-and-clock] — ACTIVE"
worktree_scratch: ".worktrees/enip-edgecase-verify @ fd0c7f3 [scratch/enip-edgecase-verify] — keep for reference"
worktree_orphan: ".worktrees/enip-f6-hardening @ 447da07 [test/enip-f6-fuzz-harnesses] — orphan, safe to remove"

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED

# Story tracking
stories_delivered: 87
current_cycle: feature-enip-v0.11.0 (D-228, 2026-06-24)
current_wave: "Wave 62 OPEN — STORY-139 EC-X1/EC-X2 fix-delta (per-direction carry + saturating window). F4 RED GATE DONE @63c119a (9 RED / 170 GREEN)."

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

**PIPELINE FEATURE-MODE. Cycle `feature-enip-v0.11.0` OPEN. v0.11.0 EtherNet/IP converged + human-approved (D-267). HELD AT RELEASE (D-260). EC-X1/EC-X2 confirmed release-blockers — fix-delta STORY-139 in progress (Wave 62, F4 red-gate done).**

**HUMAN DIRECTIVE (D-260): HALT — do NOT run the release pipeline / tag / publish without explicit human go-ahead.**

Latest release: v0.10.0 (main `0cbe922`). develop=`fd0c7f3`. stories_delivered=87. Target: v0.11.0 (SS-17 EtherNet/IP + CIP TCP/44818). GitHub issue #316. v0.11.0 STAYS HELD until STORY-139 (EC-X1/EC-X2) merges.

Spec versions: BC-INDEX v1.85 (332 on disk / 331 active; SS-17=26 BCs + EC-010 amendments). ARCH-INDEX v1.8. VP-INDEX v2.12 (34 VPs: VP-033/034 added). PRD v1.36. STORY-INDEX v2.9 (92 stories / 62 waves). epics.md v1.8 (E-20, Wave 62).

### WARNING — DO NOT REDO (on resume)

- Do NOT re-run fix cycle fix-pc-013-014-015 — CLOSED (D-226). v0.10.0 released.
- Do NOT re-cut v0.10.0 — RELEASED (main `0cbe922`, tag `v0.10.0`, run 28109367603).
- Do NOT re-deliver STORY-130..138 — all MERGED (D-234..D-259). stories_delivered=87.
- Do NOT re-merge ENIP E2E real-pcap PR #333 — MERGED (D-269) fd0c7f3. develop=`fd0c7f3`.
- Do NOT re-run F5/F6/F7 for Wave 61 — F5 CONVERGED (D-263), F6 PASSED (D-265), F7 HUMAN-APPROVED (D-267).
- Do NOT re-create RULING-EDGECASE-001 — exists at `.factory/cycles/feature-enip-v0.11.0/RULING-EDGECASE-001-direction-and-clock.md`.
- Do NOT re-run F2 spec evolution for STORY-139 — DONE + RE-VALIDATED CLEAN (factory-artifacts commits 936361e/654db0e/b15ab17/38fa910/1e48035/6e876c8/a2db4f3).
- Do NOT re-run F3 for STORY-139 — authored + registered (input-hash 759464a MATCH; STORY-INDEX v2.9 wave 62).
- Do NOT redo F4 red-gate setup — DONE on `.worktrees/enip-direction-clock` @63c119a: crate compiles, clippy clean, 9 STORY-139 tests RED, 170 existing GREEN.

### EXACT RESUME POINT — F4 IMPLEMENTER on enip-direction-clock @63c119a

**SESSION PAUSED at F4 implementation of STORY-139 EC-X1/EC-X2 fix-delta (D-270, 2026-06-27).**

Worktree: `.worktrees/enip-direction-clock` branch `fix/enip-direction-and-clock` base `fd0c7f3`.
Red-gate commit: `63c119a`. State: 9 STORY-139 tests RED / 170 existing tests GREEN / crate compiles / clippy clean.

**3 stub points to fill (in priority order):**

1. **Per-direction carry select/stash [EC-X1 / BC-2.17.016 v2.0 Inv-7/EC-010]:** Use `carry_c2s` when `direction == Direction::ClientToServer`, `carry_s2c` when `direction == Direction::ServerToClient`. A partial frame in one direction MUST NEVER be spliced with the other direction's bytes. (Stub currently uses `carry_c2s` for both → RED on AC-139-001/003/005.)
2. **Direction-based src_ip/dest_ip [AC-139-002]:** Replace `resolve_enip_client_ip()` with direction-based assignment: `ClientToServer → src_ip=client, dest_ip=server`; `ServerToClient → src_ip=server, dest_ip=client`. Findings emit `direction: Some(...)`. Mirror Modbus pattern.
3. **Saturating window monotonicity [EC-X2 / BC-2.17.008/012/018]:** Replace all `wrapping_sub` window-expiry checks with `saturating_sub`. Pin malformed `window_start_ts` guard to strict `> 300` (not `>= 300`). Field renamed `malformed_window_start_ts`. (Stub retains wrapping_sub + `>=300` → RED on AC-139-006..009.)

**After green:** micro-commit with STORY-139 AC citations; `cargo test --all-targets` full GREEN; `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt --check` clean.

**Then (sequential):**
- F5: Scoped adversarial, 3 clean passes on fix branch.
- F6: Implement VP-033 (carry-direction isolation proptest) + VP-034 (window monotonic proptest); re-run Kani 11/11 (verify VP-033/034 + existing); cargo-fuzz on fix branch; mutation delta on changed functions.
- F7: Delta-convergence. At human gate: confirm DNP3 sibling scope decision (DRIFT-DNP3-DIRECTION-001 + clock; ruling: ENIP-now/DNP3-v0.12.0 — needs human confirm before close).
- Merge fix PR into develop → re-assess v0.11.0 release posture. EC-X1/EC-X2 are release-blockers; v0.11.0 stays held until they merge.

### RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file) + `.factory/cycles/feature-enip-v0.11.0/RULING-EDGECASE-001-direction-and-clock.md` + `.factory/stories/STORY-139.md`.
3. Verify: `git rev-parse --short develop` == `fd0c7f3`; worktree `.worktrees/enip-direction-clock` on `fix/enip-direction-and-clock` @`63c119a`; `cargo test --all-targets` = 9 RED / 170 GREEN (confirm before touching code).
4. Dispatch F4 IMPLEMENTER to `.worktrees/enip-direction-clock` to fill the 3 stub points per EXACT RESUME POINT above.

### Locked design facts (do not re-derive on resume)

ENIP header LITTLE-endian (`from_le_bytes`). `EnipCommandClass` 10 variants. `CipServiceClass` 15. `CipHeader={service,request_path}`. `CpfItem={type_id,data}`. `general_status`=byte-2 on 0x00B2 responses. 0x00B2-only CIP (0x00B1 deferred v0.12.0). Write-burst 50/error-burst 5 strict `>`. T0814 windowed >=3/300s. MAX_ENIP_CARRY_BYTES=600. MAX_FINDINGS=10000. MITRE pin ics-attack-19.1. Counters u64, window timestamps u32 seconds. `EnipFlowState` now has: `carry_c2s`, `carry_s2c` (replacing `carry`), `malformed_window_start_ts` (replacing `window_start_ts`). `on_data` signature: `(flow_key, data, timestamp, direction: Direction)`. 74 call-sites updated to `Direction::ClientToServer` at red-gate.

EC-X1: cross-direction carry splice — confirmed HIGH release-blocker. EC-X2: clock-backwards `wrapping_sub` → window reset — confirmed HIGH release-blocker. Both adjudicated RULING-EDGECASE-001. Repro tests: `.worktrees/enip-edgecase-verify/tests/scratch_ecx1_ecx2_repro.rs`. DNP3 shares both patterns → DRIFT-DNP3-DIRECTION-001 (v0.12.0 sibling; human-confirm at F7).

Story input-hashes: STORY-130 63fac3a, STORY-131 ce92886, STORY-132 c33dff8, STORY-133 661f504, STORY-134 16d03a6, STORY-135 ae2d871, STORY-136 0846e0e, STORY-137 f4c8390, STORY-138 0f60353 (all MATCH). STORY-139 input-hash: 759464a (MATCH at F3 registration).

### OPEN ITEMS (backlog — non-blocking unless marked)

| ID | Summary | Status |
|----|---------|--------|
| STORY-139 | EC-X1/EC-X2 fix-delta: per-direction carry + saturating window. **RELEASE-BLOCKER.** | F4 IN PROGRESS @63c119a |
| DRIFT-DNP3-DIRECTION-001 | DNP3 shares EC-X1 + EC-X2 patterns; fix in v0.12.0 sibling follow-up. Human-confirm at F7. | DEFERRED — v0.12.0 |
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
| EDGE-CASE-HUNT-COVERAGE-IDEAS | SendUnitData path, multi-0x00B2, ForwardOpen response suppression tests (from EC hunt). | BACKLOG — optional |
| STORY-137-UNSAFE-SPLIT-BORROW | [LOW] unsafe split-borrow in process_pdu. Sound; consider safe refactor. | OPEN — v0.12.0 |

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
| Feature EtherNet/IP — EC-X1/EC-X2 fix-delta (Wave 62) — F4 | **RED-GATE DONE — IN PROGRESS** | `.worktrees/enip-direction-clock` @63c119a. 9 RED / 170 GREEN. |

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

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Active cycle: `cycles/feature-enip-v0.11.0/` (cycle-manifest.md, decisions-archive.md D-228+). Issue #316.
- STORY-INDEX.md authoritative (92 stories / 62 waves — v2.9). STORY-130..138 completed. STORY-139 authored (Wave 62). stories_delivered=87 (STORY-139 not yet merged).
- F6 fuzz harness (F-P9-002) MERGED — PR #332 @f17d270 on develop (D-265).
