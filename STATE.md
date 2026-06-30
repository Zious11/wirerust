---
document_type: pipeline-state
project: wirerust
mode: feature
phase: 7
status: in-progress
current_step: "PAUSED for session-clear — F4 wave 66, STORY-145 mid-TDD (Red Gate established; NEXT: implementer wires ServerToClient carry drain)"
pipeline: FEATURE-CYCLE
current_cycle: fix-tls-clienthello-frag
timestamp: 2026-06-30T00:00:00Z

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
develop_head: 0986e8787abf909cd08af422b0b75f845d72616a

# Cargo.toml version on main and develop
cargo_version_main: "0.11.0"
cargo_version_develop: "0.11.0"

# Open worktrees
# Main checkout: /Users/zious/Documents/GITHUB/wirerust [develop]
# Factory artifacts: /Users/zious/Documents/GITHUB/wirerust/.factory [factory-artifacts]
# ACTIVE: .worktrees/story-145-tls-serverhello-symmetry @ f60c0e0 [feature/story-145-tls-serverhello-symmetry] — Red Gate established

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED

# Story tracking
stories_delivered: 92
story_index_version: v3.6
total_stories: 99
story_index_note: "99 stories / 65 waves. STORY-130..142 MERGED. STORY-143 draft (E-11, D-301). STORY-144 MERGED (wave 65). STORY-145..146 authored (F3, wave 66)."

# Spec versions (current)
bc_index_version: v2.1
vp_index_version: v2.28
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

**ACTIVE FEATURE CYCLE: fix-tls-clienthello-frag — F4 TDD DELTA IMPLEMENTATION — wave 66 STORY-145 MID-TDD (RED GATE ESTABLISHED)**

Phases F1 DONE, F2 HUMAN-APPROVED (D-305), F3 HUMAN-APPROVED (D-306). Phase F4 active
(driven autonomously; human receives reports at wave/phase boundaries).

**Wave 65 DONE:** STORY-144 MERGED PR #341, squash `0986e878`, 11/11 CI green. stories_delivered 91→92. develop HEAD `0986e878`.

**Wave 66 — STORY-145 IN PROGRESS (mid-TDD, Red Gate established):**

- Worktree: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/story-145-tls-serverhello-symmetry`
- Branch: `feature/story-145-tls-serverhello-symmetry` — PUSHED to origin
- Base: develop `0986e878`
- Commits on branch: `389c648` (S-145.00 stubs), `f60c0e0` (S-145.01 Red-Gate tests)
- **Red Gate status:** ESTABLISHED. Tests `proptest_vp039_direction_isolation` and
  `test_BC_2_07_041_cross_flow_isolation` FAIL (ServerToClient carry drain NOT yet wired).
  136 existing tests GREEN. `cargo check`, `clippy -Dwarnings`, `cargo fmt --check` all CLEAN.

**NEXT STEP ON RESUME:** Dispatch the **implementer** to wire the ServerToClient carry drain:
1. Factor STORY-144's inline ClientToServer drain loop (see `src/analyzer/tls.rs`) into a
   **direction-parameterized helper** per AC-145-001 (ref BC-2.07.041 v1.2).
2. Add the server arm dispatching `handle_server_hello` via `server_hs_carry`.
3. Make the 2 story_145 Red-Gate tests pass; keep all 138 tests green.
4. `clippy -Dwarnings` and `cargo fmt --check` CLEAN.

**After implementer:** per-story adversarial convergence (3 clean passes, BC-5.39.001) →
demo-recorder (per-AC demos) → push → pr-manager (9-step PR, NON-DESTRUCTIVE
`update-branch`/`--auto`, NO force-push) → squash-merge → worktree cleanup.

**STORY-146 (buffer-saturation telemetry) — NOT STARTED.** Deliver AFTER STORY-145 merges
(both touch `src/analyzer/tls.rs` — sequential). Create new worktree from updated develop.

**After wave 66 both merged:** wave-66 integration gate (full regression on develop) →
F4 build-verification + holdout-evaluation (HS-F4-001..012 at
`.factory/cycles/fix-tls-clienthello-frag/holdout-scenarios.md`) → F5 scoped adversarial →
F6 targeted hardening → F7 delta convergence + HUMAN gate + RELEASE-VERSION decision
(deferred; likely v0.11.1 patch, NOT v0.12.0).

**Locked design facts (do NOT re-derive on resume):**
- Overflow = clear-and-recover (NO sticky abandon). Per-MESSAGE cap MAX_BUF=65,536.
  Per-RECORD cap 18,432. Parse boundary = `tls_parser::parse_tls_message_handshake`.
- `handshake_reassembly_overflows` + `buffer_saturation_drops` = TlsAnalyzer u64 aggregates
  (saturating_add), surfaced in `summarize()`. `TlsFlowState` = `client_hs_carry` +
  `server_hs_carry` (Vec<u8>). Cursor + single-drain (O(carry_len), SEC-001 DoS fix).
- Test wrappers in `mod story_NNN` (DF-TEST-NAMESPACE-001); strip 5-byte record header.
  Seams: `client_hello_seen_for_testing`, `client_hs_carry_len_for_testing`,
  `server_hs_carry_len_for_testing`, `handshake_reassembly_overflow_count`, `fill_buf_for_testing`.
- STORY-145: BC-2.07.041 v1.2, BC-2.07.002 v1.6; VP-039 Sub-E; HS-F4-007/008/009; hash `88e29c9`.
- STORY-146: BC-2.07.043 v1.3, BC-2.07.005 v1.7; VP-040 (6 harnesses + `fill_buf_for_testing`);
  HS-F4-010/011/012; hash `6d9da65`.

## RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file) and `.factory/cycles/fix-tls-clienthello-frag/cycle-manifest.md`.
3. Verify git state:
   - `git rev-parse origin/develop` = `0986e8787abf909cd08af422b0b75f845d72616a`
   - `git rev-parse origin/main` = `3072e8287b9f7e6621740b6e31f04ae57914d0b9`
   - Worktree `.worktrees/story-145-tls-serverhello-symmetry` on `feature/story-145-tls-serverhello-symmetry`
   - Branch tip HEAD = `f60c0e0` (Red-Gate tests commit)
4. Confirm Red Gate: run `cargo test --all-targets` from the story-145 worktree.
   - EXPECTED: 2 tests FAIL (`proptest_vp039_direction_isolation`, `test_BC_2_07_041_cross_flow_isolation`); 136 PASS.
5. Dispatch **implementer** agent as described in EXACT RESUME POINT above.
6. Maintenance sweeps PAUSED. Do not initiate during this cycle (D-303).
7. Non-blocking open question: main CHANGELOG fast-track (D-301) — re-surface if human asks.

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | feature (post-greenfield) |
| Version | 0.11.0 (released) |
| Main HEAD | `3072e828` (full: `3072e8287b9f7e6621740b6e31f04ae57914d0b9`) |
| Develop HEAD | `0986e878` (full: `0986e8787abf909cd08af422b0b75f845d72616a`) |
| Tag v0.11.0 | commit `3072e828`; tag object `c50d89e8` |
| GitHub release | https://github.com/Zious11/wirerust/releases/tag/v0.11.0 (Latest, not draft) |
| Factory artifacts HEAD | see `git -C .factory log -1 --format='%h %s'` |
| Spec versions | BC-INDEX v2.1 / VP-INDEX v2.28 (40 VPs) / ARCH-INDEX v2.4 / PRD v1.45 |
| Stories | 92 delivered / 99 total (STORY-INDEX v3.6) |

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0–7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0..v0.11.0 | RELEASED | Details: cycles/ subdirs |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking |
| Feature cycle fix-tls-clienthello-frag — F1 | DONE | delta-analysis.md committed |
| Feature cycle fix-tls-clienthello-frag — F2 | APPROVED (D-305, 2026-06-29) | 6 new BCs + 3 amended + VP-039 + VP-040 + ADR-011 |
| Feature cycle fix-tls-clienthello-frag — F3 | APPROVED (D-306, 2026-06-29) | STORY-144..146; STORY-INDEX v3.6; HS-F4-001..012 |
| Feature cycle fix-tls-clienthello-frag — F4 | **ACTIVE** | Wave 65 DONE (STORY-144 PR #341 `0986e878`). Wave 66: STORY-145 mid-TDD (Red Gate established); STORY-146 pending |
| Feature cycle fix-tls-clienthello-frag — F5 | PENDING | |
| Feature cycle fix-tls-clienthello-frag — F6 | PENDING | |
| Feature cycle fix-tls-clienthello-frag — F7 | PENDING | Version decision at gate |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 65: STORY-144 TDD delivery | DONE | PR #341 squash `0986e878`, 11/11 CI green. SEC-001 DoS fix included. stories_delivered 91→92 |
| Wave 65: Integration gate | DONE | Full regression PASS on develop `0986e878` |
| Wave 66: STORY-145 stubs | DONE | Commit `389c648` on branch feature/story-145-tls-serverhello-symmetry |
| Wave 66: STORY-145 Red-Gate tests | DONE | Commit `f60c0e0`; 2 tests FAIL (expected), 136 PASS |
| **Wave 66: STORY-145 implementation** | **NEXT** | Wire ServerToClient carry drain; make 2 story_145 tests pass; keep 138 green |

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
| D-302 | Dependabot PRs #325+#311 merged. develop `a2d8c13`. | 2026-06-29 |
| D-303 | Cycle `fix-tls-clienthello-frag` started. Full F1-F7. Maintenance paused. | 2026-06-29 |
| D-304 | F2 CONVERGED: 5 new BCs + 2 amended + VP-039 + ADR-011. | 2026-06-29 |
| D-305 | F2 APPROVED + F-EV-001 scope: BC-2.07.043 + VP-040. BC-INDEX v2.1, PRD v1.45. | 2026-06-29 |
| D-306 | F3 APPROVED. STORY-144..146; STORY-INDEX v3.6; HS-F4-001..012. Pre-F4 PASS. | 2026-06-29 |
| D-307 | STORY-144 MERGED PR #341 `0986e878`. SEC-001 DoS fixed. Wave 65 DONE. stories_delivered=92. | 2026-06-29 |
| D-308 | Session paused at STORY-145 mid-TDD (Red Gate `f60c0e0`, branch pushed). Resume checkpoint written. VP-INDEX corrected to v2.28. | 2026-06-30 |

---

## Skip Log

| Step | Justification |
|------|---------------|
| crates.io publish (v0.11.0) | Human declined at D-300 — not published |
| Holdout formal eval HS-110..122 | Deferred post-release per D-267; 10/13 behaviors covered by unit tests |
| DTU creation | Not required (passive analyzer; no external service calls) — D-dtu-assessment 2026-05-20 |

---

## Blocking Issues

*None.*

---

## Open Items / Backlog (DF-VALIDATION-001-gated unless noted)

> **MAINTENANCE SWEEPS PAUSED** — cycle `fix-tls-clienthello-frag` in progress (D-303).

| ID | Summary | Priority | Status |
|----|---------|----------|--------|
| TLS-CLIENTHELLO-FRAG-001 | ClientHello fragmented across TLS records → SNI/JA3 evasion. Severity HIGH. | HIGH | **IN PROGRESS** — F4 wave 66 STORY-145 mid-TDD |
| SEC-002 | Narrow non-RFC overflow window [MAX_BUF-3, MAX_BUF] — clears-and-recovers rather than assembling. Low exploitability. | LOW | F6 hardening |
| SEC-004 | parse_errors plain `+=` in tls.rs — theoretical u64 overflow. Cosmetic. | LOW | Maintenance sweep |
| DONE-MID-LOOP-CROSS-DIRECTION | done()-mid-loop cross-direction carry interaction. Pre-existing. | LOW | Wave-gate review |
| DF-KANI-NONVACUITY-001-PROPTEST-GAP | No proptest/unit analog for DF-KANI-NONVACUITY-001 in policies.yaml. | LOW | Policy-add at cycle close (S-7.02) |
| SEC-001-ENIP | Unsafe split-borrow enip.rs `on_data` (pre-existing). | MEDIUM | v0.12.0 candidate |
| STORY-143 | Draft story (E-11, 3 pts): harden release-changelog PR-range enumeration. | LOW | Draft — not scheduled |
| EDGE-CASE-HUNT-2026-06-28 | ~30 candidates. Register: cycles/feature-enip-v0.11.0/EDGE-CASE-HUNT-REGISTER-2026-06-28.md. | MIXED | Validation-gated |
| D-301-CHANGELOG | main CHANGELOG will catch up on next gitflow back-merge. Open question: fast-track? | LOW | Non-blocking open question |

Full backlog with resolved/archived items: `cycles/feature-enip-v0.11.0/` decisions-archive and STATE.md prior to D-308 (accessible via `git -C .factory log`).

---

## Session Resume Checkpoint

**Date:** 2026-06-30
**State:** PAUSED for session clear — Feature cycle `fix-tls-clienthello-frag`, Phase F4, wave 66, STORY-145 mid-TDD

### Exact state at pause

- Worktree: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/story-145-tls-serverhello-symmetry`
- Branch HEAD: `f60c0e0` — `test(S-145.01): faithful VP-039 Sub-E Red Gate tests`
- Branch PUSHED to `origin/feature/story-145-tls-serverhello-symmetry`
- Failing tests: `proptest_vp039_direction_isolation`, `test_BC_2_07_041_cross_flow_isolation`
- Cause: ServerToClient carry drain path not yet wired in `src/analyzer/tls.rs`

### Next action (first thing to do on resume)

Dispatch implementer to wire the ServerToClient carry drain per the EXACT RESUME POINT
section above. Full 9-step per-story delivery follows (adversarial → demos → push → PR →
squash-merge → cleanup → STORY-146).

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001. See policies.yaml for
full list.

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- STORY-INDEX.md: 99 stories / 65 waves (v3.6). stories_delivered=92. STORY-145 mid-TDD.
- v0.11.0 RELEASED 2026-06-29. main=`3072e828`, develop=`0986e878`. Not on crates.io.
- Squash-only policy (D-289). Branch protection develop + main (D-290).
