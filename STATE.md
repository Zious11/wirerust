---
document_type: pipeline-state
project: wirerust
mode: feature
phase: 7
status: complete
current_step: "IDLE — v0.11.0 released; awaiting human direction on v0.12.0 scope"
pipeline: FEATURE-MODE
current_cycle: feature-enip-v0.11.0
timestamp: 2026-06-29T14:20:00Z

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
develop_head: a2d8c13ff9e23f49d5ab93ab6453da4442658bcc
factory_artifacts_head: d67eb274d9d5900b68a26180399d7b8aaccb6ce5

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
bc_index_version: v1.88
vp_index_version: v2.14
arch_index_version: v1.8
prd_version: v1.36
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

**PIPELINE IDLE — v0.11.0 RELEASED (D-300, 2026-06-29). Nothing in flight.**

- v0.11.0 released: release PR #337 squash-merged → main `3072e828`. Annotated tag `v0.11.0` pushed. GitHub release published (marked Latest). develop synced via PR #338 → `ecbcd268`; then PRs #339+#340 (CHANGELOG corrections) landed → `ab0b388`.
- Dependabot triage DONE (D-302, 2026-06-29): PR #325 (softprops/action-gh-release 3.0.1) squash-merged `a715437`; PR #311 (actions/checkout v7.0.0, major) squash-merged `a2d8c13`. develop now at `a2d8c13`. main unchanged at `3072e828`.
- main + develop both carry Cargo.toml version `0.11.0`. All CI checks green.
- crates.io publish: declined by human — not published.
- Cycle `feature-enip-v0.11.0` CLOSED (D-300). All four EC-X1/EC-X2 fixes shipped: ENIP (STORY-139, PR #334), DNP3 EC-X1/X2 (STORY-140, PR #335), Modbus EC-X1/X2 (STORY-141, PR #336), DNP3 desync-latch (STORY-142, PR #336).
- stories_delivered = 91. STORY-INDEX v3.2 (96 stories total; STORY-143 draft added for v0.12.0).
- No open PRs requiring action. Two stale scratch worktrees on disk (`.worktrees/enip-edgecase-verify`, `.worktrees/enip-f6-hardening`) — safe to remove; no active work.

**ONE OPEN HUMAN QUESTION (D-301):**
Should the corrected complete `[0.11.0]` CHANGELOG entry be fast-tracked onto `main` now (via a docs-only PR), or left to ride to the next gitflow back-merge at v0.12.0 release time? The complete entry is on develop (`ab0b388`). main currently has the original short v0.11.0 entry. No functional impact either way. **Awaiting human answer — not yet received.**

## RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file).
3. Verify: `git rev-parse origin/main` = `3072e8287b9f7e6621740b6e31f04ae57914d0b9`; `git rev-parse origin/develop` = `a2d8c13ff9e23f49d5ab93ab6453da4442658bcc`; `git tag -l v0.11.0` exists.
4. Pipeline is IDLE. Re-surface the open human question (main CHANGELOG fast-track y/n) if unanswered.
5. Next work: v0.12.0 planning — start with research-agent validation of TLS ClientHello fragmentation finding (CRIT candidate, see backlog below).

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
| Develop HEAD | `a2d8c13` (full: `a2d8c13ff9e23f49d5ab93ab6453da4442658bcc`) |
| Tag v0.11.0 | commit `3072e828`; tag object `c50d89e8` |
| GitHub release | https://github.com/Zious11/wirerust/releases/tag/v0.11.0 (Latest, not draft) |
| Factory artifacts HEAD | `d67eb274` |
| Spec versions | BC-INDEX v1.88 / VP-INDEX v2.14 (38 VPs) / ARCH-INDEX v1.8 / PRD v1.36 |
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

---

## Current Phase Steps

| Step | Status | Notes |
|------|--------|-------|
| Wave-64 F7 converged | DONE (D-298) | BC-INDEX v1.88; VP-INDEX v2.14 |
| Wave-64 PR #336 merged | DONE (D-299, 2026-06-28) | develop `a13b5c5`; stories_delivered=91 |
| v0.11.0 release | DONE (D-300, 2026-06-29) | main `3072e828`; tag pushed; GitHub release Latest |
| Post-release CHANGELOG corrections | DONE (D-301, 2026-06-29) | PR #339 + #340; develop `ab0b388` |
| Dependabot triage #325 + #311 | DONE (D-302, 2026-06-29) | PR #325 `a715437`, PR #311 `a2d8c13`; develop `a2d8c13` |
| Pipeline IDLE | **CURRENT** | Awaiting human direction on v0.12.0 scope |

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

| ID | Summary | Priority | Status |
|----|---------|----------|--------|
| TLS-CLIENTHELLO-FRAG-001 | ClientHello fragmented across TLS records → SNI/JA3 evasion (tls.rs:763-792). No record reassembly. RECOMMENDED FIRST — validate via research-agent. | CRIT CANDIDATE | DF-VALIDATION-001-gated |
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
**State:** IDLE post-v0.11.0 + Dependabot triage complete

### What was done this session
- v0.11.0 RELEASED (D-300): four EC-X1/EC-X2 fixes (ENIP/DNP3/Modbus/DNP3-desync) + ENIP analyzer epic.
- Post-release CHANGELOG corrections (D-301): PRs #339+#340 applied complete [0.11.0] entry to develop.
- GitHub release notes edited (40 ENIP/MITRE markers confirmed, still Latest).
- STORY-143 (draft, E-11) created for release-changelog hardening.
- decisions-archive.md extended D-228..D-301 (was D-228..D-266).
- STATE.md compacted: historical "Do NOT re-X" block and verbose inline decisions (D-270..D-299) archived to cycles/feature-enip-v0.11.0/decisions-archive.md.
- Dependabot triage (D-302): PRs #325 + #311 squash-merged; develop advanced to `a2d8c13`. main unchanged at `3072e828`.

### Open question requiring human response
Should the corrected `[0.11.0]` CHANGELOG entry be fast-tracked onto `main` now via a docs-only PR, or left to ride to the next gitflow back-merge at v0.12.0 release time? (no functional impact either way)

### Next candidate work
1. Answer the open question above.
2. Research-agent validation of TLS-CLIENTHELLO-FRAG-001 (CRIT candidate — recommended first for v0.12.0 scoping).
3. v0.12.0 planning based on backlog + research-agent findings.

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
- BC-INDEX v1.88 (BC-2.14.002 v2.1 errata anchor reconciliation, D-298). VP-INDEX v2.14 (38 VPs; VP-037 range 0..6, D-298).
