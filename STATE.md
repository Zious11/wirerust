---
document_type: pipeline-state
project: wirerust
mode: feature
phase: 7
status: in-progress
current_step: "Feature cycle fix-tls-clienthello-frag — F2 APPROVED (incl F-EV-001 counter); F3 story decomposition active"
pipeline: FEATURE-CYCLE
current_cycle: fix-tls-clienthello-frag
timestamp: 2026-06-29T19:00:00Z

# Release chain (latest)
released_version: v0.11.0
released_at: "2026-06-29"
release_tag: v0.11.0
release_tag_object: c50d89e88984df2ba22bd24332a7a2c7d9626f2c
release_commit: 3072e8287b9f7e6621740b6e31f04ae57914d0b9
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.11.0
prior_released_version: v0.10.0
prior_released_at: "2026-06-24"

# Ground-truth HEADs (verified 2026-06-29 via git rev-parse)
main_head: 3072e8287b9f7e6621740b6e31f04ae57914d0b9
develop_head: ab0b3883b8bc942d7d11bacb0e8b2387ecb2b4c0
factory_artifacts_head: c25f7e45e67044ab646146c24e1c495b05ab1160

# Cargo.toml version on main and develop
cargo_version_main: "0.11.0"
cargo_version_develop: "0.11.0"

# Open worktrees (2 stale scratch worktrees remain on disk — can be removed)
# Main checkout: /Users/zious/Documents/GITHUB/wirerust [develop]
# Factory artifacts: /Users/zious/Documents/GITHUB/wirerust/.factory [factory-artifacts]
# STALE (scratch, safe to remove): .worktrees/enip-edgecase-verify @ fd0c7f3 [scratch/enip-edgecase-verify]
# STALE (scratch, safe to remove): .worktrees/enip-f6-hardening @ 447da07 [test/enip-f6-fuzz-harnesses]

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED

# Story tracking
stories_delivered: 91
story_index_version: v3.2
total_stories: 96
story_index_note: "96 stories / 64 waves. STORY-130..142 MERGED. STORY-143 draft (E-11, D-301)."

# Spec versions (current)
bc_index_version: v2.1
vp_index_version: v2.25
arch_index_version: v2.4
prd_version: v1.45
epics_version: v1.8

# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []

# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-22
maintenance_completed_at: "2026-06-23"
---

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**ACTIVE FEATURE CYCLE: fix-tls-clienthello-frag — F2 FULLY APPROVED (D-305, 2026-06-29); F3 STORY DECOMPOSITION ACTIVE**

Phase F2 spec evolution FULLY CONVERGED + HUMAN-APPROVED, including the human-requested F-EV-001 defense-in-depth scope addition (BC-2.07.043 + VP-040). Phase F3 (incremental story decomposition) is the active step.

**F2 delta summary (final, including scope addition):**
- 6 new BCs: BC-2.07.038 (v2.7, reassembly across records), BC-2.07.039 (v2.4, bounded carry clear-and-recover overflow), BC-2.07.040 (v1.3, truncation-safety), BC-2.07.041 (v1.2, per-flow+per-direction isolation), BC-2.07.042 (v1.4, coalesced dispatch), BC-2.07.043 (buffer_saturation_drops counter).
- 3 amended: BC-2.07.001 v1.9, BC-2.07.002 v1.6 (scope expansion to fragmented-then-assembled), BC-2.07.005 v1.7 (silent-truncation Inv-3 superseded; reconciled with BC-2.07.043).
- VP-039 (proptest+unit; 17 harnesses: 4 proptest + 13 unit). VP-040 (6 harnesses, buffer saturation observability). ADR-011 (TLS handshake reassembly design).
- Spec versions: BC-INDEX v2.1, VP-INDEX v2.25 (40 VPs), ARCH-INDEX v2.4, PRD v1.45, SS-07 now 43 BCs. BC total: 337 on disk / 336 active.

**Locked design decisions (do NOT re-derive on resume):**
- OVERFLOW POLICY = clear-and-recover (Policy A, NO sticky-abandon flag). Chosen over abandon to deny permanent per-flow blinding. Research: `.factory/research/TLS-REASSEMBLY-OVERFLOW-POLICY.md` (Ptacek/Newsham; Suricata CVE-2019-18792).
- Per-MESSAGE body_len cap = MAX_BUF=65,536 (raised from 18,432; Go maxHandshake parity).
- Per-RECORD cap stays 18,432 (BC-2.07.004).
- Parse boundary = `tls_parser::parse_tls_message_handshake` (NOT `parse_tls_plaintext`).
- `handshake_reassembly_overflows` = TlsAnalyzer u64 AGGREGATE counter (not per-flow), surfaced in `summarize()`.
- Per-flow ceiling 4×MAX_BUF (post-on_data-return residue).
- `TlsFlowState` gains `client_hs_carry` + `server_hs_carry` (Vec<u8>) only.
- F-EV-001 defense-in-depth IMPLEMENTED IN SPEC (human pulled into cycle): BC-2.07.043 buffer_saturation_drops — TlsAnalyzer u64 aggregate counter incremented when on_data buffer-append discards bytes (condition data.len() > remaining, covering partial AND full drop), increment HOISTED after the &mut state block (borrow constraint), surfaced in summarize(), no finding/no parse_errors; test seam fill_buf_for_testing(&mut self, &FlowKey, Direction, usize). BC-2.07.005 v1.7 reconciled (silent-truncation Inv-3 superseded). Three distinct telemetry counters now: parse_errors+truncated_records (record-oversize BC-2.07.004), handshake_reassembly_overflows (carry overflow BC-2.07.039), buffer_saturation_drops (TCP-buffer saturation BC-2.07.043).

**Next action:** Execute `vsdd-factory:phase-f3-incremental-stories`.

- v0.11.0 released (D-300, 2026-06-29). main=`3072e828`, develop=`ab0b388`. crates.io not published.
- Two stale scratch worktrees on disk (`.worktrees/enip-edgecase-verify`, `.worktrees/enip-f6-hardening`) — safe to remove when convenient.

**OPEN HUMAN QUESTION (D-301, non-blocking):** Should the corrected `[0.11.0]` CHANGELOG entry be fast-tracked onto `main` now, or wait for the next gitflow back-merge? No functional impact either way. Awaiting answer.

## RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file).
3. Verify: `git rev-parse origin/main` = `3072e8287b9f7e6621740b6e31f04ae57914d0b9`; `git rev-parse origin/develop` = `ab0b3883b8bc942d7d11bacb0e8b2387ecb2b4c0`; `git tag -l v0.11.0` exists.
4. Active cycle: `fix-tls-clienthello-frag`. F2 FULLY APPROVED (D-305) — F3 story decomposition ACTIVE. Read `.factory/cycles/fix-tls-clienthello-frag/cycle-manifest.md` for scope + phase status.
5. Maintenance sweeps PAUSED. Do not initiate maintenance work during this cycle.
6. Next action: Execute `vsdd-factory:phase-f3-incremental-stories`.
7. Non-blocking open question: main CHANGELOG fast-track (D-301) — re-surface if human asks.

## Locked design facts (do not re-derive on resume)

- ENIP header LITTLE-endian (`from_le_bytes`). `EnipCommandClass` 10 variants. `CipServiceClass` 15. `EnipFlowState` fields: `carry_c2s`, `carry_s2c` (replacing `carry`), `malformed_window_start_ts` (replacing `window_start_ts`). `on_data` sig: `(flow_key, data, timestamp, direction: Direction)`.
- Modbus needs NO `on_data` sig change (direction already threaded). Sustained-window `>=` at modbus.rs:670 is an INTENTIONAL min-duration gate — PRESERVE. Modbus carry-cap STRUCTURALLY UNREACHABLE (max operand 259 < 260=MAX_ADU_CARRY_BYTES; active_carry.clear() at L1075 drains before walk). 6 cargo-mutants survivors EQUIVALENT — do NOT attempt to kill them.
- DNP3 desync fix REQUIRES `frame_count==0` guard — both-carries-empty-only is INCOMPLETE (missed sub-case ii where a completed frame drains carry).
- STORY-104 AC-006 "wrapping_sub" text superseded by saturating_sub precedent (spec corrected in STORY-141).
- EC-X1: cross-direction carry splice — CONFIRMED HIGH. EC-X2: clock-backwards `wrapping_sub` → window reset — CONFIRMED HIGH. Both FIXED in STORY-139..142.
- 2 .factory STORY-104 red-gate-LOG citations of old test names are historical run-logs — no update needed.

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | feature (post-greenfield) |
| Version | 0.11.0 (released) |
| Main HEAD | `3072e828` (full: `3072e8287b9f7e6621740b6e31f04ae57914d0b9`) |
| Develop HEAD | `ab0b388` (full: `ab0b3883b8bc942d7d11bacb0e8b2387ecb2b4c0`) |
| Tag v0.11.0 | commit `3072e828`; tag object `c50d89e8` |
| GitHub release | https://github.com/Zious11/wirerust/releases/tag/v0.11.0 (Latest, not draft) |
| Factory artifacts HEAD | see `git -C .factory log -1 --format='%h %s'` |
| Spec versions | BC-INDEX v2.1 / VP-INDEX v2.25 (40 VPs) / ARCH-INDEX v2.4 / PRD v1.45 |
| Stories | 91 delivered / 96 total (STORY-INDEX v3.2) |

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19 |
| Phase 1 — Spec Crystallization | PASSED 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs |
| Phase 2 — Story Decomposition | PASSED 2026-05-21 | 49 stories / 11 epics / 27 waves |
| Phase 3 — TDD Implementation | PASSED 2026-05-31 | 48/48 stories, 27/27 waves |
| Phase 4 — Holdout Evaluation | PASSED 2026-06-01 | mean 0.949 |
| Phase 5 — Adversarial Refinement | PASSED 2026-06-01 | 3/3 adversary gate SATISFIED |
| Phase 6 — Formal Hardening | PASSED 2026-06-02 | 8 Kani VPs; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0 | RELEASED 2026-06-12 | Detail: cycles/feature-8-dnp3-v0.5.0/ |
| Feature ARP (E-16) + v0.7.0 | RELEASED 2026-06-16 | STORY-111..115; VP-024 LOCKED |
| E-17 ARP QinQ/MACsec + v0.7.1 | RELEASED 2026-06-17 | STORY-116/117 |
| E-18 finding-collapse + v0.8.0 | RELEASED 2026-06-17 | SS-11=29 BCs |
| E-18/E-8 STORY-119 + v0.9.0 | RELEASED 2026-06-19 | 293 BCs; tag v0.9.0 |
| v0.9.1/v0.9.2 patches | RELEASED 2026-06-19 | Doc/help + DNP3 determinism |
| Feature pcapng-reader + v0.9.3 | RELEASED 2026-06-22 | 10 new BCs, VP-INDEX v2.10 |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking |
| Feature mitre-json-names + v0.9.4 | RELEASED 2026-06-23 | BC-INDEX v1.71 |
| Fix cycle fix-pc-013-014-015 + v0.10.0 | RELEASED 2026-06-24 | tag v0.10.0 `0cbe922` |
| Feature EtherNet/IP (Waves 58-64) + v0.11.0 | **RELEASED 2026-06-29 (D-300)** | tag v0.11.0 `3072e828`. Detail: cycles/feature-enip-v0.11.0/ |
| Feature cycle fix-tls-clienthello-frag — F1 | DONE | delta-analysis.md; architect completed |
| Feature cycle fix-tls-clienthello-frag — F2 | **FULLY APPROVED (D-305, 2026-06-29)** | 6 new BCs (incl BC-2.07.043) + 3 amended (incl BC-2.07.005 v1.7) + VP-039 + VP-040 + ADR-011; scope addition F-EV-001 defense-in-depth approved |
| Feature cycle fix-tls-clienthello-frag — F3 | **ACTIVE** | Story decomposition in progress |

---

## Current Phase Steps

| Step | Status | Notes |
|------|--------|-------|
| Phase F1 — Delta Analysis | DONE | Architect completed; delta-analysis.md committed |
| Phase F2 — Spec Evolution | **FULLY APPROVED (D-305, 2026-06-29)** | 6 new BCs + 3 amended + VP-039 + VP-040 + ADR-011; F-EV-001 defense-in-depth scope addition approved |
| Phase F3 — Incremental Stories | **ACTIVE** (current) | Story decomposition in progress |
| Phase F4 — TDD Delta Implementation | PENDING | |
| Phase F5 — Scoped Adversarial Review | PENDING | |
| Phase F6 — Targeted Hardening | PENDING | |
| Phase F7 — Delta Convergence | PENDING | Version decision at gate |

---

## Decisions Log

D-001..D-054: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`
D-055..D-130: `cycles/feature-collapse-v0.8.0/decisions-archive.md`
D-131..D-135: `cycles/feature-story-119-grouped-collapse/decisions-archive.md`
D-136..D-202: `cycles/feature-pcapng-reader/decisions-archive.md`
D-206..D-217: `cycles/feature-mitre-json-names/decisions-archive.md`
D-219..D-226: `cycles/fix-pc-013-014-015/decisions-archive.md`
D-228..D-301: `cycles/feature-enip-v0.11.0/decisions-archive.md`

| ID | Decision | Date |
|----|----------|------|
| D-300 | v0.11.0 RELEASED. PR #337 squash→main `3072e828`. Tag v0.11.0 pushed. GitHub release published (Latest). develop synced PR #338 `ecbcd268`. crates.io: not published (human declined). Cycle CLOSED. | 2026-06-29 |
| D-301 | POST-RELEASE: v0.11.0 CHANGELOG corrected (PRs #339+#340; develop `ab0b388`). GitHub release notes updated (40 ENIP/MITRE markers confirmed). STORY-143 (draft, E-11) created. STORY-INDEX v3.2 (96 stories). main CHANGELOG will catch up on next gitflow back-merge. Open question: fast-track main CHANGELOG now vs wait — not yet answered. | 2026-06-29 |
| D-302 | Dependabot triage: PRs #325 (softprops/action-gh-release 3.0.0→3.0.1, squash-merged 14:10:55Z, merge commit `a715437`) and #311 (actions/checkout 6.0.3→7.0.0 major, squash-merged 14:14:34Z, merge commit `a2d8c13`) merged to develop after 7-day soak verification per .github/dependabot.yml cooldown policy (#311: 11-day soak). CI 22/22 green on both. SHA-pinning preserved. No breaking impact (plain checkout usage, not fork pull_request_target pattern). develop now `a2d8c13`. Human-approved merge. | 2026-06-29 |
| D-303 | Started Feature-Mode cycle `fix-tls-clienthello-frag` for TLS-CLIENTHELLO-FRAG-001 (validated CONFIRMED, severity HIGH). Human chose full F1-F7 VSDD process; release version deferred to F7 convergence (not v0.12.0 or v0.11.1 yet). Maintenance sweeps paused for cycle duration. develop at `a2d8c13`. | 2026-06-29 |
| D-304 | Phase F2 spec evolution CONVERGED for fix-tls-clienthello-frag (TLS handshake reassembly). 5 new BCs (BC-2.07.038-042) + 2 amended (BC-2.07.001 v1.9, BC-2.07.002 v1.6) + VP-039 (17 harnesses) + ADR-011. Overflow policy = clear-and-recover (human-approved via research, D-303 cycle); per-message cap 65,536; parse boundary = `parse_tls_message_handshake`. 3+ clean adversary passes after 12 fix bursts. Awaiting F2 human approval gate. | 2026-06-29 |
| D-305 | Phase F2 APPROVED (D-304 converged). Human approved + pulled the F-EV-001 defense-in-depth counter into the cycle: BC-2.07.043 (buffer_saturation_drops) + BC-2.07.005 v1.7 + VP-040 (6 harnesses) authored and converged via scoped adversarial passes. F-EV-001 (client_buf saturation) validated NOT-EXPLOITABLE; the counter makes the tail-drop primitive non-silent (pre-empts P1/P2). SS-07 43 BCs, VP total 40, BC-INDEX v2.1, PRD v1.45. Proceeding to F3. | 2026-06-29 |

---

## Skip Log

| Step | Justification |
|------|---------------|
| crates.io publish (v0.11.0) | Human declined at D-300 — not published |
| Holdout formal eval HS-110..122 | Deferred post-release per D-267; 10/13 behaviors covered by unit tests |
| DTU creation | Not required (passive analyzer; no external service calls) — D-dtu-assessment 2026-05-20 |

---

## Blocking Issues

*None. Pipeline IDLE.*

---

## Open Items / Backlog (v0.12.0 candidates — all DF-VALIDATION-001-gated)

> **MAINTENANCE SWEEPS PAUSED** — cycle `fix-tls-clienthello-frag` in progress (D-303). Do not initiate new maintenance sweep runs until F7 gate.

| ID | Summary | Priority | Status |
|----|---------|----------|--------|
| TLS-CLIENTHELLO-FRAG-001 | ClientHello fragmented across TLS records → SNI/JA3 evasion (tls.rs:763-792). No record reassembly. Severity revised CRIT-candidate → HIGH (validated). | HIGH | IN PROGRESS (cycle fix-tls-clienthello-frag; F2 CONVERGED) |
| F-EV-001 | client_buf TCP-buffer saturation silent blinding. Research-validated NOT-EXPLOITABLE on develop (`.factory/research/F-EV-001-clientbuf-saturation-validation.md` — residue ceiling ≤18,437 < MAX_BUF; oversize records telemetered; no segment coalescing). DF-VALIDATION-001 SATISFIED. Defense-in-depth counter BC-2.07.043 (buffer_saturation_drops) + VP-040 implemented in spec this cycle (D-305). F-EV-003 (done() whole-flow ServerHello silent-loss) is same class — not exploitable. | LOW | **RESOLVED** — defense-in-depth counter implemented in spec (D-305); BC-2.07.043 / VP-040 authored |
| PRD-HEADER-VERSION-POINTER-LAG | [LOW residual — F3 sweep] PRD §2.7.1 subsection header version-pointer may lag current frontmatter version (v1.42→ now should reflect v1.45). Cosmetic only; body content current. | LOW | F3 sweep |
| BC-2.07.043-PC4-BTREEMAP-LABEL | [NIT residual — F3 sweep] BC-2.07.043 PC-4 prose says HashMap but real type is BTreeMap<String,serde_json::Value> (non-load-bearing label mismatch). | NIT | F3 sweep |
| SEAM-SIG-HISTORICAL-RESIDUE | [LOW residual — non-actionable] Two historical changelog entries (prd.md, spec-changelog) retain old by-value seam signature — accurate historical residue, not a defect. | LOW | No action needed (accurate history) |
| DF-KANI-NONVACUITY-001-PROPTEST-GAP | [process-gap] DF-KANI-NONVACUITY-001 has no proptest/unit-test analog in policies.yaml. Proptest non-vacuity defects this cycle (Sub-F near-vacuity, Sub-C wrong-path) would have been caught earlier by codified proptest-analog policy. Candidate for policy-add at cycle close (per Cycle-Closing Checklist S-7.02). | LOW | Candidate — cycle close |
| SEC-001 | Unsafe split-borrow in src/analyzer/enip.rs `on_data` — sound under invariant, but should refactor to modbus.rs owned-borrow pattern. Pre-existing PR #334. | MEDIUM | v0.12.0 candidate |
| STORY-143 | Draft story (E-11, 3 pts): harden release-changelog to enumerate full `<prev-tag>..HEAD` PR range (policy DF-RELEASE-CHANGELOG-RANGE-001 candidate). | LOW | Draft — not yet scheduled |
| EDGE-CASE-HUNT-2026-06-28 | ~30 candidates across all analyzers. Register: cycles/feature-enip-v0.11.0/EDGE-CASE-HUNT-REGISTER-2026-06-28.md. 4 CRIT, ~9 HIGH, MED/LOW. All DF-VALIDATION-001-gated. | MIXED | Candidates — validation-gated |
| DESIGN-TIMESTAMP-MONOTONICITY | Design note: cycles/feature-enip-v0.11.0/DESIGN-TIMESTAMP-MONOTONICITY.md. Informs v0.12.0 planning. | REF | Archive (afd7dbb) |
| DESIGN-CROSS-DIRECTION-STATE | Design note: cycles/feature-enip-v0.11.0/DESIGN-CROSS-DIRECTION-STATE.md. Informs v0.12.0 planning. | REF | Archive (afd7dbb) |
| DNP3-DOC-COMMENT-STALE-ACTIVE-CARRY | [NIT] Stale doc comment in src/analyzer/dnp3.rs referencing old `active_carry!` pattern. Doc-only. DF-VALIDATION-001-gated. | NIT | v0.12.0 |
| MODBUS-DEBUG-ASSERT-CARRY-CLEAR | [SEC-002 LOW] Add `debug_assert!(dir_carry.is_empty())` after carry clear in modbus.rs. Defensive correctness. DF-VALIDATION-001-gated. | LOW | v0.12.0 |
| PROCESS-WATCH: false-structurally-killed claim | "False structurally-killed mutation claim" gap hit 2 consecutive waves (63+64). A 3rd triggers DF-ADVERSARY-TOOLCHAIN-PAIRING-001 policy-text update. | PROCESS | Watch |
| DEPENDABOT-311 | PR #311 (actions/checkout 6.0.3→7.0.0 major). | LOW | MERGED 2026-06-29 (D-302, merge commit `a2d8c13`) |
| DEPENDABOT-325 | PR #325 (softprops/action-gh-release 3.0.0→3.0.1). | LOW | MERGED 2026-06-29 (D-302, merge commit `a715437`) |
| ENIP-CARRY-CAP-V0.12.0-REDESIGN | BC-2.17.016 PC-4 carry-overflow cap is unreachable dead code (RULING-137-002). v0.12.0 redesign should make reachable or remove. Mirrors MODBUS-CARRY-CAP. | BACKLOG | v0.12.0 |
| F6-MUTANTS-FULL-RUN | Confirm 0-missed on full 241-mutant run for ENIP (21 caught/0 missed at F6 gate — confirm at F7). | BACKLOG | Human confirm |
| BC-PROSE-LOW-RESIDUALS | BC-2.17.001 Inv-4 + prd.md singular `carry`; BC-2.17.018 PC-1 singular `carry`; VP-034 title-label drift. | LOW | Cycle close / v0.12.0 |

All GitHub-issue creation DF-VALIDATION-001-gated (policies.yaml).

---

## Session Resume Checkpoint

**Date:** 2026-06-29
**State:** Feature cycle `fix-tls-clienthello-frag` — Phase F2 FULLY APPROVED (D-305, incl F-EV-001 scope addition); F3 story decomposition ACTIVE

### What was done this session
- v0.11.0 RELEASED (D-300) + post-release corrections (D-301) + Dependabot triage (D-302).
- TLS-CLIENTHELLO-FRAG-001 research-validated CONFIRMED HIGH; DF-VALIDATION-001 SATISFIED.
- Feature cycle `fix-tls-clienthello-frag` initialized (D-303); F1 delta analysis completed.
- Phase F2 spec evolution executed and CONVERGED: 5 new BCs (BC-2.07.038-042), 2 amended BCs (BC-2.07.001 v1.9, BC-2.07.002 v1.6), VP-039 (17 harnesses), ADR-011. 12 fix bursts; 3+ clean adversary passes. BC-INDEX v1.98, VP-INDEX v2.21, ARCH-INDEX v2.3, PRD v1.43.
- F-EV-001 research-validated NOT-EXPLOITABLE. Process gap DF-KANI-NONVACUITY-001-PROPTEST-GAP filed.
- Phase F2 HUMAN-APPROVED (D-305). Human pulled F-EV-001 defense-in-depth into cycle: BC-2.07.043 (buffer_saturation_drops), BC-2.07.005 v1.7, VP-040 (6 harnesses) authored + converged. BC-INDEX v2.1, VP-INDEX v2.25 (40 VPs), ARCH-INDEX v2.4, PRD v1.45, SS-07 43 BCs. BC total 337 on disk / 336 active.
- Maintenance sweeps PAUSED.

### Open question (non-blocking)
Should the corrected `[0.11.0]` CHANGELOG entry be fast-tracked onto `main` now via a docs-only PR, or left to ride to the next gitflow back-merge? No functional impact either way.

### Next action
Execute `vsdd-factory:phase-f3-incremental-stories`.

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. Active policies (17): DF-VALIDATION-001 (HIGH), DF-SIBLING-SWEEP-001 v4 (CRITICAL), DF-PR-MANAGER-COMPLETE-001 (HIGH), DF-ADVERSARY-METHODOLOGY-001 (HIGH), DF-AC-TEST-NAME-SYNC-001 v2 (MEDIUM), DF-CONVERGENCE-BEFORE-MERGE-001 (CRITICAL), DF-DEVELOP-FRESHNESS-001 v2 (HIGH), DF-ADVERSARY-TOOLCHAIN-PAIRING-001 (MEDIUM), DF-INPUT-HASH-CANONICAL-001 (HIGH), DF-ADVERSARY-CHECKOUT-GUARD-001 (HIGH), DF-TEST-CITATION-SWEEP-001 (HIGH), DF-TEST-NAMESPACE-001 (MEDIUM), DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 (HIGH), DF-CANONICAL-FRAME-HOLDOUT-001 (CRITICAL), DF-BC-COMPLETENESS-SWEEP-001 (HIGH), DF-GREEN-DOC-TENSE-SWEEP v2 (HIGH), DF-KANI-NONVACUITY-001 (HIGH).

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Closed cycle: `cycles/feature-enip-v0.11.0/` (decisions-archive.md D-228..D-301, CLOSED D-300).
- STORY-INDEX.md authoritative (96 stories / 64 waves — v3.2). STORY-130..142 all MERGED. stories_delivered=91. STORY-143 added draft (E-11, D-301).
- v0.11.0 RELEASED (D-300, 2026-06-29). main=`3072e828`, develop=`a2d8c13` (post-Dependabot triage D-302), tag v0.11.0. crates.io not published.
- Repo squash-only policy set (D-289). Branch protection on develop + main (D-290).
- SEC-001 (unsafe split-borrow enip.rs `on_data`, MEDIUM, pre-existing PR #334) in backlog as v0.12.0 candidate (D-300).
- BC-INDEX v1.88 (BC-2.14.002 v2.1 errata anchor reconciliation, D-298). VP-INDEX v2.14 (38 VPs; VP-037 range 0..6, D-298). Cycle fix-tls-clienthello-frag F2 (initial convergence): BC-INDEX v1.98 (5 new BCs + 2 amended in SS-07; SS-07 42 BCs), VP-INDEX v2.21 (39 VPs; VP-039 added, 17 harnesses), ARCH-INDEX v2.3 (ADR-011 added), PRD v1.43. F2 scope addition (D-305): BC-INDEX v2.1 (BC-2.07.043 + BC-2.07.005 v1.7; SS-07 43 BCs; 337 on disk / 336 active), VP-INDEX v2.25 (40 VPs; VP-040 added, 6 harnesses), ARCH-INDEX v2.4, PRD v1.45.
