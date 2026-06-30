---
document_type: pipeline-state
project: wirerust
mode: feature
phase: 7
status: in-progress
current_step: "F4 holdout PASS (0.904 mean, 8/8 must-pass). HS-F4-001-FRAMEC finding RESOLVED (artifact-fidelity, no code change). NEXT: F5 scoped adversarial."
pipeline: FEATURE-CYCLE
current_cycle: fix-tls-clienthello-frag
timestamp: 2026-06-30T14:00:00Z

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
develop_head: 8b52046421332b6d9796fe93acf69b38960815bd

# Cargo.toml version on main and develop
cargo_version_main: "0.11.0"
cargo_version_develop: "0.11.0"

# Open worktrees
# Main checkout: /Users/zious/Documents/GITHUB/wirerust [develop]
# Factory artifacts: /Users/zious/Documents/GITHUB/wirerust/.factory [factory-artifacts]
# (story-145 + story-146 worktrees removed — PRs #343/#344 merged, branches deleted)

# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED

# Story tracking
stories_delivered: 94
story_index_version: v3.8
total_stories: 99
story_index_note: "99 stories / 66 waves. STORY-130..142 MERGED. STORY-143 draft (E-11, D-301). STORY-144 MERGED (wave 65). STORY-145 MERGED (wave 66, PR #343). STORY-146 MERGED (wave 66, PR #344, 8b52046). Wave 66 COMPLETE."

# Spec versions (current)
bc_index_version: v2.2
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

**ACTIVE FEATURE CYCLE: fix-tls-clienthello-frag — F5 SCOPED ADVERSARIAL NEXT**

Phases F1 DONE, F2 HUMAN-APPROVED (D-305), F3 HUMAN-APPROVED (D-306), F4 PASS (D-311).

**Wave 65 DONE:** STORY-144 MERGED PR #341, squash `0986e878`. stories_delivered 91→92.

**Wave 66 DONE (D-309, D-310):** STORY-145 MERGED PR #343, squash `d3d2e19` (5-pass convergence,
3 clean on `ae45d54`). stories_delivered 92→93. STORY-146 MERGED PR #344, squash `8b52046`
(multi-pass convergence, 3 clean on `bb29117`; found+fixed doc-tense, saturating_add SEC-003
sibling-parity, EC-C1/EC-C3 coverage, 6→8 test-count header drift, accessor visibility parity,
EC-C1 docstring attribution). pr-reviewer APPROVE + security-reviewer CLEAR; 11/11 CI green.
stories_delivered 93→94.

**Wave-66 integration gate PASS:** Full regression on develop `8b52046` = 2220/0; clippy/fmt green.

**F4 holdout PASS (D-311):** Mean 0.904 ≥ 0.85; 8/8 must-pass ≥ 0.6. 10/12 independently
verified; HS-F4-010/012 seam-gated proxy-verified. HS-F4-001 Frame C finding triaged →
verdict B+C (artifact-fidelity / conformant-leniency): BC-2.07.038 v2.8 + holdout corrected;
no code change. STORY-144 input-hash `3dfe20c`→`9b4284b`; STORY-145 `88e29c9`→`9a82e5d`.

**NEXT STEP ON RESUME:** F5 scoped adversarial — fresh-context multi-pass adversarial review
of the cycle DELTA (STORY-144 #341 + STORY-145 #343 + STORY-146 #344 on develop `8b52046`),
with the MANDATORY DF-BC-COMPLETENESS-SWEEP-001 first step (enumerate all cycle BCs SS-07:
BC-2.07.038/040/041/042/043 + amended 001/002/005, verify each Invariant/PC has an
implementation path in the delta). Then F6 targeted hardening → F7 delta convergence +
HUMAN gate + RELEASE-VERSION decision (likely v0.11.1 patch).

**Locked design facts (do NOT re-derive on resume):**
- Overflow = clear-and-recover (NO sticky abandon). Per-MESSAGE cap MAX_BUF=65,536.
  Per-RECORD cap 18,432. Parse boundary = `tls_parser::parse_tls_message_handshake`.
- `handshake_reassembly_overflows` + `buffer_saturation_drops` = TlsAnalyzer u64 aggregates
  (saturating_add), surfaced in `summarize()`. `TlsFlowState` = `client_hs_carry` +
  `server_hs_carry` (Vec<u8>). Cursor + single-drain (O(carry_len), SEC-001 DoS fix).
- Test wrappers in `mod story_NNN` (DF-TEST-NAMESPACE-001); strip 5-byte record header.
  Seams: `client_hello_seen_for_testing`, `client_hs_carry_len_for_testing`,
  `server_hs_carry_len_for_testing`, `handshake_reassembly_overflow_count`, `fill_buf_for_testing`.
- STORY-145: BC-2.07.041 v1.2, BC-2.07.002 v1.6; VP-039 Sub-E; HS-F4-007/008/009; hash `88e29c9`. MERGED PR #343 `d3d2e19`.
- STORY-146: BC-2.07.043 v1.3, BC-2.07.005 v1.7; VP-040 (6 canonical + 2 EC-coverage = 8 tests;
  `fill_buf_for_testing`); HS-F4-010/011/012; hash `6d9da65`. MERGED PR #344 `8b52046`.

## RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file) and `.factory/cycles/fix-tls-clienthello-frag/cycle-manifest.md`.
3. Verify git state:
   - `git rev-parse origin/develop` = `8b52046421332b6d9796fe93acf69b38960815bd`
   - `git rev-parse origin/main` = `3072e8287b9f7e6621740b6e31f04ae57914d0b9`
   - No story worktrees open (all removed after merge)
4. F5 entry: dispatch **vsdd-factory:phase-f5-scoped-adversarial** — fresh-context
   multi-pass adversarial review of the cycle DELTA (STORY-144/145/146 on develop `8b52046`).
   MANDATORY first step: DF-BC-COMPLETENESS-SWEEP-001 (enumerate all cycle BCs SS-07:
   BC-2.07.038/040/041/042/043 + amended 001/002/005, verify each Invariant/PC has an
   implementation path in the delta).
5. Maintenance sweeps PAUSED. Do not initiate during this cycle (D-303).
6. Non-blocking open question: main CHANGELOG fast-track (D-301) — re-surface if human asks.

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | feature (post-greenfield) |
| Version | 0.11.0 (released) |
| Main HEAD | `3072e828` (full: `3072e8287b9f7e6621740b6e31f04ae57914d0b9`) |
| Develop HEAD | `8b52046` (full: `8b52046421332b6d9796fe93acf69b38960815bd`) |
| Tag v0.11.0 | commit `3072e828`; tag object `c50d89e8` |
| GitHub release | https://github.com/Zious11/wirerust/releases/tag/v0.11.0 (Latest, not draft) |
| Factory artifacts HEAD | see `git -C .factory log -1 --format='%h %s'` |
| Spec versions | BC-INDEX v2.2 / VP-INDEX v2.28 (40 VPs) / ARCH-INDEX v2.4 / PRD v1.45 |
| Stories | 94 delivered / 99 total (STORY-INDEX v3.8) |

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
| Feature cycle fix-tls-clienthello-frag — F4 | **DONE/PASS** | Holdout 0.904 mean, 8/8 must-pass; HS-F4-001-FRAMEC artifact-fidelity correction (BC-2.07.038 v2.8 + holdout corrected; no code change) |
| Feature cycle fix-tls-clienthello-frag — F5 | **ACTIVE/NEXT** | Scoped adversarial — cycle DELTA (STORY-144/145/146 on develop `8b52046`) |
| Feature cycle fix-tls-clienthello-frag — F6 | PENDING | |
| Feature cycle fix-tls-clienthello-frag — F7 | PENDING | Version decision at gate |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| Wave 66: Integration gate | DONE | Full regression PASS on develop `8b52046` (2220/0); clippy/fmt green |
| F4: holdout-evaluation | DONE/PASS | Mean 0.904, 8/8 must-pass; 10/12 directly verified; HS-F4-010/012 seam-gated proxy-verified |
| F4: HS-F4-001 Frame C triage | DONE | Verdict B+C (artifact-fidelity): BC-2.07.038 v2.8 + holdout corrected; no code change; STORY-144/145 hashes recomputed |
| F4: BC-INDEX v2.2 committed | DONE | bc_index_version v2.1→v2.2 (BC-2.07.038 v2.8 correction) |
| **F5: scoped adversarial** | **NEXT** | Fresh-context multi-pass review of cycle DELTA (STORY-144/145/146); DF-BC-COMPLETENESS-SWEEP-001 first |

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
| D-309 | STORY-145 MERGED PR #343 squash `d3d2e19`. Per-story convergence: 5 passes (1-2 found doc-tense MEDIUM + DoS-guard coverage MEDIUM, both fixed; 3 clean on ae45d54). pr-reviewer + security-reviewer APPROVE; 11/11 CI green. Human-authorized merge (auto-mode classifier required direct user merge). stories_delivered=93. | 2026-06-30 |
| D-310 | STORY-146 MERGED PR #344 squash `8b52046`. Per-story convergence: multi-pass (found+fixed doc-tense, saturating_add SEC-003 sibling-parity, EC-C1/EC-C3 coverage, 6→8 test-count header drift, accessor visibility parity, EC-C1 docstring attribution; 3 clean on `bb29117`). pr-reviewer APPROVE + security-reviewer CLEAR; 11/11 CI green. Human-authorized merge. stories_delivered=94. Wave 66 COMPLETE; integration gate PASS (2220/0). | 2026-06-30 |
| D-311 | F4 holdout PASS (mean 0.904 ≥ 0.85; must-pass 8/8 ≥ 0.6). HS-F4-001 Frame C contradiction triaged by research-agent → verdict B+C (artifact-fidelity / conformant-leniency): BC-2.07.038 v2.7→v2.8 (Frame C input corrected to 0xcc body, PC-9 example updated, NOTE on degenerate-all-zero accepted case added) + holdout HS-F4-001 Frame C corrected to match; no code change. STORY-144 input-hash `3dfe20c`→`9b4284b`; STORY-145 `88e29c9`→`9a82e5d` (recomputed via bin/compute-input-hash). BC-INDEX v2.1→v2.2. | 2026-06-30 |

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
| TLS-CLIENTHELLO-FRAG-001 | ClientHello + ServerHello fragmentation → SNI/JA3/JA3S evasion. Severity HIGH. | HIGH | Code merged (STORY-144/145/146). F4 holdout PASS. **F5 scoped adversarial NEXT** |
| TLS-FRAMEC-TEST-DOC-001 | Verify/align comment-docstring of `test_BC_2_07_038_canonical_frame_rfc8446_s4` Frame C (~tests/tls_analyzer_tests.rs:9619) to accurately describe the 0xcc session-id>32 vector (test is behaviorally correct + green; doc-comment fidelity item). | LOW | Fold into F6 hardening or doc fix-PR (requires develop code PR) |
| TLS-CH-STRICT-VALIDATION-001 | Consider strict structural validation rejecting empty-cipher-suite / trailing-padded ClientHellos as an evasion anomaly (deliberate change against lenient-fingerprinting norm; separate maintenance item). | LOW | NOT this cycle — separate maintenance item |
| SEC-002 | Narrow non-RFC overflow window [MAX_BUF-3, MAX_BUF] — clears-and-recovers rather than assembling. Low exploitability. | LOW | F6 hardening |
| SEC-004 | parse_errors plain `+=` in tls.rs — theoretical u64 overflow. Cosmetic. | LOW | Maintenance sweep |
| DONE-MID-LOOP-CROSS-DIRECTION | done()-mid-loop cross-direction carry interaction. Pre-existing. | LOW | Wave-gate review |
| DF-KANI-NONVACUITY-001-PROPTEST-GAP | No proptest/unit analog for DF-KANI-NONVACUITY-001 in policies.yaml. | LOW | Policy-add at cycle close (S-7.02) |
| SEC-001-ENIP | Unsafe split-borrow enip.rs `on_data` (pre-existing). | MEDIUM | v0.12.0 candidate |
| STORY-143 | Draft story (E-11, 3 pts): harden release-changelog PR-range enumeration. | LOW | Draft — not scheduled |
| EDGE-CASE-HUNT-2026-06-28 | ~30 candidates. Register: cycles/feature-enip-v0.11.0/EDGE-CASE-HUNT-REGISTER-2026-06-28.md. | MIXED | Validation-gated |
| D-301-CHANGELOG | main CHANGELOG will catch up on next gitflow back-merge. Open question: fast-track? | LOW | Non-blocking open question |
| SEC-006 | TLS handshake carry Step-1 guard uses strict `>`, allowing carry to reach exactly MAX_BUF (65,536 B). Pre-existing & symmetric from STORY-144 (not a STORY-145 regression). CWE-400. | LOW | Defer to F6 hardening alongside SEC-002 |
| TLS-DRAIN-DUP-001 | ~85 lines C2S/S2C drain-loop duplication in src/analyzer/tls.rs (intentional symmetric mirror per spec; borrow-checker tradeoff). Refactor follow-up story candidate. | LOW | Validation-gated (DF-VALIDATION-001) |
| TLS-STALE-COMMENT-001 | Stale C2S comment "STORY-145 scope (not reachable here)" now reachable in S2C arm; plus STORY-144 test-seam RED-tense doc at src/analyzer/tls.rs ~1323/1344. Doc chore. | LOW | Maintenance sweep |
| TLS-SILENT-COMMENT-001 | Stale comment in `tests/tls_analyzer_tests.rs::test_buffer_overflow_silent_no_counters` ("no counters") now inaccurate since `buffer_saturation_drops` increments on that path. Out of STORY-146 diff. | LOW | Maintenance sweep / future TLS story |
| TLS-SUMMARIZE-MAPTYPE-001 | BC-2.07.043 PC-4 / AC-146-005 describe detail map as `HashMap<String, Value>` but impl uses `BTreeMap` (LESSON-P2.09 deterministic ordering). Descriptive wording shared by all sibling keys; consider BC-wording normalization. | LOW | Spec sweep (architectural defer) |

Full backlog with resolved/archived items: `cycles/feature-enip-v0.11.0/` decisions-archive and STATE.md prior to D-308 (accessible via `git -C .factory log`).

---

## Session Resume Checkpoint

**Date:** 2026-06-30
**State:** F4 holdout PASS; F5 scoped adversarial next — Feature cycle `fix-tls-clienthello-frag`, Phase F5

### Exact state at checkpoint

- develop HEAD: `8b52046421332b6d9796fe93acf69b38960815bd` (short `8b52046`)
- F4 holdout PASS: mean 0.904, 8/8 must-pass; HS-F4-001 Frame C finding resolved (artifact-fidelity)
- BC-2.07.038 at v2.8; BC-INDEX at v2.2
- STORY-144 input-hash: `9b4284b` (was `3dfe20c`); STORY-145: `9a82e5d` (was `88e29c9`)
- All story worktrees removed; no open worktrees

### Next action (first thing to do on resume)

Dispatch **vsdd-factory:phase-f5-scoped-adversarial** — fresh-context multi-pass adversarial
review of cycle DELTA (STORY-144 #341 + STORY-145 #343 + STORY-146 #344 on develop `8b52046`).
MANDATORY first step: DF-BC-COMPLETENESS-SWEEP-001 (enumerate all cycle BCs SS-07:
BC-2.07.038/040/041/042/043 + amended 001/002/005, verify each Invariant/PC has an
implementation path in the delta).

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001. See policies.yaml for
full list.

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- STORY-INDEX.md: 99 stories / 66 waves (v3.8). stories_delivered=94. STORY-144/145/146 MERGED (wave 65+66). Wave 66 COMPLETE. F4 holdout PASS.
- v0.11.0 RELEASED 2026-06-29. main=`3072e828`, develop=`8b52046` (wave 66 gate). Not on crates.io.
- BC-INDEX v2.2 (BC-2.07.038 v2.8 Frame-C fidelity fix). STORY-144/145 input-hashes updated.
- Squash-only policy (D-289). Branch protection develop + main (D-290).
