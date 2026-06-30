---
document_type: pipeline-state
project: wirerust
mode: feature
phase: 7
status: paused
current_step: "F6 targeted hardening IN PROGRESS — PAUSED for session clear. Kani VP-039 (3 proofs, non-vacuous, PASS) + fuzz_tls_reassembly (1.9M execs CLEAN) DONE on branch fix/f6-tls-hardening (cd005f8, pushed). REMAINING: 13 mutation-gap tests → test-writer; RUSTSEC-2026-0190 anyhow bump; F6 fix-PR (human merge); then F7."
pipeline: FEATURE-CYCLE
current_cycle: fix-tls-clienthello-frag
timestamp: 2026-06-30T22:00:00Z

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
# F6 hardening WIP: /Users/zious/Documents/GITHUB/wirerust/.worktrees/f6-tls-hardening [fix/f6-tls-hardening @ cd005f8] — PUSHED to origin
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
bc_index_version: v2.3
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

**ACTIVE FEATURE CYCLE: fix-tls-clienthello-frag — F6 TARGETED HARDENING IN PROGRESS (PAUSED mid-F6 for session clear)**

Phases F1 DONE, F2 HUMAN-APPROVED (D-305), F3 HUMAN-APPROVED (D-306), F4 PASS (D-311), F5 CONVERGED (D-312), F6 IN PROGRESS (D-313, PAUSED — session clear).

**F6 active branch:** `fix/f6-tls-hardening` (worktree `/Users/zious/Documents/GITHUB/wirerust/.worktrees/f6-tls-hardening`, HEAD `cd005f8b990f9396bf34a31b38ef9cdf179faddc`, base develop `8b52046`, PUSHED to origin). Working tree CLEAN. Commits: `d085db2` (fuzz target), `07865cb` (kani proofs), `cd005f8` (fuzz Cargo.lock sync + mutants ignore).

**F6 COMPLETED so far (D-313):**

- **Kani formal proofs: PASS (non-vacuous).** `#[cfg(kani)] mod kani_proofs_vp039` added in `src/analyzer/tls.rs` (additive; kani_proofs_vp005 untouched). 3 harnesses: `verify_drain_loop_cursor_safety` (unwind 5, 5/5 covers), `verify_no_usize_overflow_on_advance`, `verify_carry_bounded_after_append`. All VERIFICATION SUCCESSFUL, NON-VACUOUS (DF-KANI-NONVACUITY-001 satisfied). Proven: cursor in-bounds (no OOB/underflow), ≥4-byte progress (termination), no usize overflow on `consumed += 4+body_len`, carry ≤ MAX_BUF. Documented limit: drain loop modeled (`drain_loop_model`) — HashMap RandomState hasher is FFI CBMC can't symbolically execute; fuzz is the dynamic cross-check.
- **Fuzz: CLEAN.** `fuzz/fuzz_targets/fuzz_tls_reassembly.rs` added (registered in `fuzz/Cargo.toml`). 181s, 1,909,352 execs, 0 crashes/OOM/timeouts. Corpus 826 entries. No artifacts.
- **Security scan:** `cargo audit` = 0 vulns + 1 allowed warning (RUSTSEC-2026-0190 anyhow, PRE-EXISTING, not delta). `cargo deny check` advisories FAILED on RUSTSEC-2026-0190 (anyhow ≤1.0.102 unsoundness) — PRE-EXISTING on develop (delta never touched Cargo.toml/lock). Manual delta pass clean (no unsafe, bounded allocs, DoS bounds intact).
- **Regression at cd005f8:** `cargo test --all-targets` 2220/0; clippy clean; fmt clean; green-doc-tense PASS. Stable build unaffected (kani gated `#[cfg(kani)]`; fuzz is separate crate).
- **Mutation testing: 13 SURVIVING real-gap mutants** (test-coverage blind spots in carry-reassembly delta — NOT production defects; Kani + fuzz confirm production logic is sound). 133 total: 102 caught + 5 timeout-caught + 21 missed (8 provably-equivalent, 13 real gaps) + 5 unviable. The 13 gaps cluster into 6 symmetric C2S/S2C themes (all `src/analyzer/tls.rs`):

  | Theme | Mutant Sites | Description |
  |-------|-------------|-------------|
  | 1 — Step-1 overflow guard exact-MAX_BUF boundary | C2S 829:64 `>→>=`, S2C 998:64 `>→>=` | No test pins carry filled to exactly MAX_BUF (should accept, not clear) |
  | 2 — Step-1 guard arithmetic | C2S 829:41 `+→*`, S2C 998:41 `+→*` | Non-zero carry + large payload arithmetic path |
  | 3 — Decision-4 body_len spoof boundary (body_len==MAX_BUF) | C2S 900:37 `>→>=`, S2C 1036:37 `>→>=` | body_len exactly MAX_BUF not tested |
  | 4 — parse_errors increment (PC-9, non-hello msg_type) | C2S 950:59 `+=→-=` + `+=→*=`, S2C 1079:59 ×2 | No test dispatches non-hello msg_type (0x01 C2S / 0x02 S2C) after partial carry |
  | 5 — S2C body_len high-byte lane | S2C 1030:67 `<<→>>` | No S2C test uses length with non-zero bits 16-23 |
  | 6 — Incomplete-body guard (consumed>0 partial trailing) + exact-fill | S2C 1047:38 `-→+`; C2S 911:38 needs deterministic (proptest-randomized only); S2C did_drop exact-fill 1155:43 `>→>=` | `carry_len - consumed` with consumed>0; C2S mirror 1145:43 IS covered |

**F6 REMAINING (IN ORDER — do not skip):**

1. **Route 13 mutation gaps to test-writer**: Add deterministic boundary/arithmetic tests in `mod story_144`/`mod story_145`/`mod story_146` (or new `mod f6_hardening`) on `fix/f6-tls-hardening`. Target themes 1–6 above. Observe DF-TEST-NAMESPACE-001 + snapshot-delta discipline.
2. **Re-run cargo-mutants** scoped to delta — confirm 13 gaps killed (effective kill rate → ~100%; only 8 provably-equivalent survivors remain).
3. **Handle RUSTSEC-2026-0190** (anyhow): `cargo update -p anyhow` to ≥1.0.103 to clear `cargo deny` advisories. Pre-existing on develop — decide whether to fix on F6 branch (bundled) or separate PR. Validate per DF-VALIDATION-001.
4. **Deliver F6 via fix-pr-delivery**: push (already pushed) → PR to develop → pr-reviewer + security-reviewer → CI green → HUMAN-AUTHORIZED squash-merge (auto-mode classifier blocks subagent --admin merge; USER must run `gh pr merge`). Then step-9 cleanup + state checkpoint.
5. **F7 delta convergence**: 5-dimension check on delta + full regression on develop + HUMAN gate + RELEASE-VERSION decision (likely v0.11.1 patch).

**Locked design facts (do NOT re-derive on resume):**
- Overflow = clear-and-recover (NO sticky abandon). Per-MESSAGE cap MAX_BUF=65,536. Per-RECORD cap 18,432.
- `handshake_reassembly_overflows` + `buffer_saturation_drops` = TlsAnalyzer u64 aggregates (saturating_add). `TlsFlowState` = `client_hs_carry` + `server_hs_carry` (Vec<u8>). Cursor + single-drain (O(carry_len)).
- Test wrappers in `mod story_NNN` (DF-TEST-NAMESPACE-001). Seams: `client_hello_seen_for_testing`, `client_hs_carry_len_for_testing`, `server_hs_carry_len_for_testing`, `handshake_reassembly_overflow_count`, `fill_buf_for_testing`.
- F5 BCs (7 re-anchored, BC-INDEX v2.3): 001 v2.0, 002 v1.7, 005 v1.8, 038 v2.10, 039 v2.5, 042 v1.5, 043 v1.4.

## RESUME PROCEDURE (execute in order — BLOCKING)

1. Run `vsdd-factory:factory-worktree-health` — PASS required before proceeding.
2. Read `.factory/STATE.md` (this file) and `.factory/cycles/fix-tls-clienthello-frag/cycle-manifest.md`.
3. Verify git state:
   - `git rev-parse origin/develop` = `8b52046421332b6d9796fe93acf69b38960815bd`
   - `git rev-parse origin/fix/f6-tls-hardening` = `cd005f8b990f9396bf34a31b38ef9cdf179faddc`
   - F6 worktree mounted: `ls /Users/zious/Documents/GITHUB/wirerust/.worktrees/f6-tls-hardening/src/analyzer/tls.rs` exists
   - `git -C /Users/zious/Documents/GITHUB/wirerust/.worktrees/f6-tls-hardening status` = clean
4. Proceed with F6 REMAINING step 1 (dispatch test-writer for 13 mutation-gap themes above).
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
| Spec versions | BC-INDEX v2.3 / VP-INDEX v2.28 (40 VPs) / ARCH-INDEX v2.4 / PRD v1.45 |
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
| Feature cycle fix-tls-clienthello-frag — F5 | **DONE/CONVERGED** | 5 passes; BC-completeness 60/60, 0 P0; F-01/F-03 BC-2.07.038 v2.10 reconciliation; F-F5-001 re-anchor 7 BCs; BC-INDEX v2.3 |
| Feature cycle fix-tls-clienthello-frag — F6 | **IN PROGRESS (PAUSED mid-F6, session clear)** | Kani VP-039 3 proofs PASS (non-vacuous); fuzz 1.9M execs CLEAN; 13 mutation-gap tests REMAIN; on fix/f6-tls-hardening cd005f8 (pushed) |
| Feature cycle fix-tls-clienthello-frag — F7 | PENDING | Version decision at gate |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| F4: holdout-evaluation | DONE/PASS | Mean 0.904, 8/8 must-pass; 10/12 directly verified; HS-F4-010/012 seam-gated proxy-verified |
| F4: HS-F4-001 Frame C triage | DONE | Verdict B+C (artifact-fidelity): BC-2.07.038 v2.8 + holdout corrected; no code change |
| F4: BC-INDEX v2.2 committed | DONE | bc_index_version v2.1→v2.2 (BC-2.07.038 v2.8 correction) |
| F5: scoped adversarial | DONE/CONVERGED | 5 passes; 60/60 BC-completeness; 0 P0; BC-2.07.038 v2.10 + re-anchor 7 BCs; BC-INDEX v2.3; STORY-052/053/058/144/145/146 hashes updated |
| F6: kani proofs (VP-039) | DONE | 3 harnesses, non-vacuous, PASS — on fix/f6-tls-hardening d085db2+07865cb |
| F6: fuzz target | DONE | fuzz_tls_reassembly.rs; 1.9M execs, 0 crashes — cd005f8 |
| F6: security scan | DONE (note) | cargo audit 0 vulns; cargo deny advisories FAIL on RUSTSEC-2026-0190 anyhow (PRE-EXISTING, not delta) |
| **F6: mutation-gap tests** | **NEXT (IN PROGRESS)** | 13 surviving gaps, 6 themes — dispatch test-writer on fix/f6-tls-hardening |

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
| D-312 | F5 scoped adversarial CONVERGED. BC-completeness sweep all-covered (60/60 clauses, 0 P0 BLOCKER across 8 cycle BCs). Findings resolved: F-01 (BC-2.07.038 Inv-4 done()-scope over-claim) + F-03 (PC-3 direction-gating) reconciled → BC-2.07.038 v2.10; F-F5-001 (stale architecture anchors) → re-anchor sweep 7 SS-07 BCs (001 v2.0, 002 v1.7, 005 v1.8, 038 v2.10, 039 v2.5, 042 v1.5, 043 v1.4) verified against tls.rs. F-02 (fill_buf_for_testing mutating doc-hidden pub seam) → W7.1 backlog. PAUSED before F6 per human. BC-INDEX v2.2→v2.3. STORY-052/053/058/144/145/146 input-hashes updated (→ 85f5eb7/024eede/f18801e/52fb717/f2748f9/6134dfc). | 2026-06-30 |
| D-313 | F6 targeted hardening IN PROGRESS (paused for session clear). Kani VP-039 carry-drain safety proofs (3 harnesses, non-vacuous, PASS) + fuzz_tls_reassembly (1.9M execs clean) added on fix/f6-tls-hardening (cd005f8, pushed). Mutation testing: 13 surviving real-gap mutants (6 C2S/S2C symmetric themes — test blind spots, not defects; Kani+fuzz confirm production logic sound). Security: RUSTSEC-2026-0190 anyhow advisory pre-existing (cargo deny advisories FAIL), out of delta. Regression 2220/0. | 2026-06-30 |

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
| TLS-CLIENTHELLO-FRAG-001 | ClientHello + ServerHello fragmentation → SNI/JA3/JA3S evasion. Severity HIGH. | HIGH | Code merged (STORY-144/145/146). F4 PASS. F5 CONVERGED. **F6 IN PROGRESS (PAUSED mid-F6, D-313)** |
| F6-MUTATION-GAPS-001 | F6 WIP: 13 surviving real-gap mutants across 6 C2S/S2C symmetric themes in src/analyzer/tls.rs. Themes: (1) Step-1 overflow guard exact-MAX_BUF boundary 829:64/998:64; (2) Step-1 guard arithmetic 829:41/998:41; (3) Decision-4 body_len==MAX_BUF 900:37/1036:37; (4) parse_errors PC-9 non-hello dispatch 950:59/1079:59; (5) S2C body_len high-byte lane 1030:67; (6) incomplete-body guard consumed>0 and did_drop exact-fill 1047:38/1155:43 (C2S 911:38 needs deterministic test, 1145:43 covered). Test-coverage gaps, not production defects (Kani+fuzz confirm logic sound). Fix: dispatch test-writer on fix/f6-tls-hardening before F6 fix-PR. | HIGH (F6 WIP) | IN PROGRESS — test-writer step next |
| RUSTSEC-2026-0190 | anyhow ≤1.0.102 unsoundness (freshly-published advisory). `cargo deny check` advisories FAIL on develop and F6 branch. PRE-EXISTING — delta never touched Cargo.toml/lock. Fix: `cargo update -p anyhow` to ≥1.0.103. Decide: bundle with F6 branch or separate Dependabot-style PR. Validate per DF-VALIDATION-001. | MEDIUM | Open — not yet addressed |
| TLS-FILLBUF-PUBLIC-SEAM-001 | `fill_buf_for_testing` is a mutating `#[doc(hidden)] pub` library-surface method; first mutating test-seam on the public API; should be on W7.1 `cargo public-api` baseline when it lands (F5 F-02). | LOW | W7.1 backlog (deferred from F5 per D-312) |
| BC-ANCHOR-DRIFT-OUTOFCYCLE-001 | Out-of-cycle stale tls.rs anchors found during F5 re-anchor sweep: BC-2.07.004:124 (:319→:339), BC-2.07.028:109 (:413-515→:455-558), STORY-054:127 (:519-539→:600-621). Fix in next maintenance sweep. | LOW | Maintenance sweep (not this cycle) |
| PG-BC-ANCHOR-VALIDATION-001 | No automated line-anchor validation in the spec pipeline; line-anchor drift recurs every cycle that grows tls.rs (F-F5-001 this cycle; precedent PG-ARP-*). Follow-up: CI/maintenance check resolving cited symbol line-ranges, or policy to prefer symbol-only anchors. Per S-7.02: open follow-up story OR record as justified deferral before cycle CLOSE. | LOW | Process-gap — open before cycle close (S-7.02) |
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
**State:** PAUSED mid-F6 for session clear — Feature cycle `fix-tls-clienthello-frag`, Phase F6 targeted hardening IN PROGRESS

### Exact state at checkpoint

- develop HEAD: `8b52046421332b6d9796fe93acf69b38960815bd` (short `8b52046`). main HEAD: `3072e8287b9f7e6621740b6e31f04ae57914d0b9` (v0.11.0).
- F6 WIP branch: `fix/f6-tls-hardening`, HEAD `cd005f8b990f9396bf34a31b38ef9cdf179faddc`, PUSHED to `origin/fix/f6-tls-hardening`. Worktree: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/f6-tls-hardening`. Working tree CLEAN.
- F6 DONE: Kani VP-039 (3 proofs, non-vacuous, PASS) — commit `07865cb`; fuzz_tls_reassembly (1.9M execs, 0 crashes) — commit `d085db2`+`cd005f8`.
- F6 REMAINING: 13 mutation-gap tests (6 themes — see EXACT RESUME POINT above); RUSTSEC-2026-0190 anyhow bump; F6 fix-PR (human-authorized merge); then F7.
- stories_delivered: 94. BC-INDEX v2.3. VP-INDEX v2.28. No story worktrees open.

### Next action (first thing on resume — BLOCKING sequence)

1. `vsdd-factory:factory-worktree-health` — PASS required.
2. Verify `origin/fix/f6-tls-hardening` = `cd005f8b990f9396bf34a31b38ef9cdf179faddc`.
3. Verify F6 worktree mounted and clean (path above).
4. Dispatch test-writer for 13 mutation-gap themes (6 themes, exact mutant sites in EXACT RESUME POINT) on `fix/f6-tls-hardening`.
5. Re-run cargo-mutants scoped to delta to confirm gaps killed.
6. Handle RUSTSEC-2026-0190 (anyhow bump decision).
7. fix-pr-delivery → F6 PR → human-authorized merge → F7 delta convergence.

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001. See policies.yaml for
full list.

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- STORY-INDEX.md: 99 stories / 66 waves (v3.8). stories_delivered=94. STORY-144/145/146 MERGED (wave 65+66). Wave 66 COMPLETE. F4 PASS. F5 CONVERGED. F6 IN PROGRESS / PAUSED (D-313).
- v0.11.0 RELEASED 2026-06-29. main=`3072e828`, develop=`8b52046`. Not on crates.io.
- F6 WIP on fix/f6-tls-hardening cd005f8 (pushed). Kani+fuzz DONE. 13 mutation-gap tests remain.
- BC-INDEX v2.3. STORY-052/053/058/144/145/146 input-hashes updated. Squash-only policy (D-289). Branch protection develop + main (D-290).
