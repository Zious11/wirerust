---
pipeline: FEATURE_MODE_ARP_ANALYZER
phase: feature-F4-delta-implementation
phase_status: "F4 IN PROGRESS — STORY-111 DELIVERED (PR #236 cced898); STORY-112 (wave 41) Step-4.5 CONVERGED (3/3 clean logic passes; 1512 tests green; final HEAD c68964d; PR pending); NEXT = demo-recorder → pr-manager (STORY-112 PR)."
active_feature: "arp-analyzer"
feature_arp_status: "F1 Delta Analysis PASSED (human-gated 2026-06-12) — DecodedFrame integration, ADR-008 planned, F2→F7 authorized; release target v0.7.0"
feature_8_status: "v0.6.0 RELEASED 2026-06-12 — DNP3 TCP analyzer; F7 5-dim CONVERGED; tag v0.6.0 + 4 binaries"
product: wirerust
mode: brownfield
timestamp: 2026-06-14T00:00:00Z
bootstrapped: 2026-05-19T16:56:48Z
phase_0_completed: 2026-05-19T20:00:00Z
phase_1_completed: "2026-05-21"
phase_2_completed: "2026-05-21"
phase_3_completed: "2026-05-31"
phase_4_completed: "2026-06-01"
phase_5_completed: "2026-06-01"
phase_6_completed: "2026-06-02"
phase_7_to_release_gate: "PASSED (human-approved 2026-06-09 — D-045)"
adversary_gate: SATISFIED
develop_head: cced898
main_head: 3e29891
released_version: v0.6.0
released_at: "2026-06-12"
release_tag: v0.6.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.6.0
release_commit: 3e29891
prior_released_version: v0.5.0
prior_released_at: "2026-06-10"
prior_release_tag: v0.5.0
prior_release_commit: c2df1b5
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 57
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
arp_f2_adversary_convergence_counter: 3/3 CONVERGED  # Pass 31/32/33 consecutive CLEAN; F2 strict-whole-corpus adversarial gate SATISFIED
arp_f3_adversary_convergence_counter: 3/3 CONVERGED  # Passes 36/37/38 consecutive CLEAN; F3 strict-whole-corpus adversarial gate SATISFIED
arp_f2_convergence_trajectory: "15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5→~18→~8→~22(P14: 2C/5H NEW corpus-debt; trend broke; ARP delta clean 6th pass)→P15(8 findings: holdout-layer field-rename + regression; REMEDIATED)→P16(7: 0C/0H, sibling-sweep misses; REMEDIATED; Slice B CLEAN)→P17(10: holdout MITRE-counts + module-decomposition peer; REMEDIATED; Slice B CLEAN 2nd)→P18(9: ss-05 anchor-drift + indicatif + STORY-INDEX; 0C/3H; REMEDIATED; arp.rs+holdout pre-flush verified clean)→P19(15: corpus-wide anchor-drift; 0C/8H; PARTIAL — ss-07-full+remaining-BC pending)→ batch2: ss-07-full(35 BCs)+ss-04-partial(21 BCs)+ss-11(10 BCs); ss-01/02/08/13 CLEAN; ss-04-remainder+ss-12 to Pass-20 — REMEDIATED → P20(7: anchor-drift flushed, ss-04/ss-12 closed; 0C/1H; Slices A+C CLEAN; REMEDIATED) → P21(5 cosmetic; 0C/0H; A+C CLEAN; REMEDIATED) → P22(5 valid; 0C/0H; cosmetic; version-pin hardened; REMEDIATED) → P23(5; B/C/D CLEAN; Slice-A only; 0C/0H; REMEDIATED) → P24(4: D-01 DNP3-C24 sweep genuine + 3 self-induced; 0C/1H; B+C CLEAN; REMEDIATED) → P25(2; A/B/C CLEAN; changelog-path flush; 0C/0H; REMEDIATED) → P26 CLEAN 1/3 (all 4 slices zero findings; corpus-wide debt flushed P14-25) → P27 reset 1/3→0/3 (HS-008 kill-chain + HS-INDEX pin; holdout-pin-hardened) → P28 CLEAN 1/3 (restart after P27 reset) → P29 reset 1/3→0/3 (DNP3 T1692.001 + PRD FC-0x17 content gaps; REMEDIATED) → P30 (4 HIGH genuine: FlowKey accessor + STORY input-hash dup + ADR-006 FC0x17; REMEDIATED) → P31 CLEAN 1/3 (restart; P30 HIGH fixes held; all 4 slices zero findings) → P32 CLEAN 2/3 (2nd consecutive) → P33 CLEAN 3/3 CONVERGED (F2 strict-whole-corpus gate satisfied after 33 passes). Detail: phase-f5-adversarial/arp-f2-convergence-trajectory.md"
f3_convergence_trajectory: "F3 STRICT WHOLE-CORPUS CONVERGED 3/3 — GATE SATISFIED. Full per-pass detail P1-P38: phase-f5-adversarial/arp-f3-convergence-trajectory.md. P31 FULLY CLEAN (clean-streak 0/3→1/3). P32 reset (STORY-115 storm_findings field; REMEDIATED). P33 reset (BC-2.15.024 parse_errors→malformed_in_window; REMEDIATED). POST-P33 SS-15 FLUSH (6 findings). P34 reset (changelog Artifacts table; REMEDIATED). P35 reset (changelog line-pins; de-pin sweep). P36 FULLY CLEAN (clean-streak 0/3→1/3). P37 FULLY CLEAN (clean-streak 1/3→2/3). P38 FULLY CLEAN — all 4 slices ZERO; A 17th-consec, B converged, C converged, D converged; mount-guards PASSED; clean-streak 2/3→3/3. **F3 STRICT WHOLE-CORPUS ADVERSARIAL GATE SATISFIED** (Passes 36/37/38 consecutive CLEAN). Total: 38 passes."
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT
input_drift_check: "MATCH=23 STALE=44 ERROR=1 (STORY-091 known); ARP stories: STORY-111=d05149f MATCH, STORY-112=8a4d566 MATCH, STORY-113 STALE (stored a767d96/computed 7c61bae), STORY-114 STALE (stored e2f1c95/computed 5705a10), STORY-115 STALE (stored 5ca9835/computed 2e0eca2); STORY-113/114/115 went STALE after arp-architecture-delta v1.16 (D-072) — re-stamp before STORY-113 delivery (non-blocking for STORY-112); scan 2026-06-14."
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.6.0 RELEASED (DNP3 TCP analyzer, issue #8). Feature: ARP security analyzer + etherparse 0.16→0.20 migration (F1 PASSED 2026-06-12, D-066); release target v0.7.0. F2 CONVERGED (P33 CLEAN; 3/3 strict-whole-corpus). F3 CONVERGED 3/3 (Passes 36/37/38). F4 IN PROGRESS — STORY-111 DELIVERED (PR #236 cced898; wave 40; D-073); STORY-112 Step-4.5 CONVERGED (c68964d; 1512 tests green; PR pending). NEXT = demo-recorder → pr-manager (STORY-112).**

**Summary:** 68 stories (48 greenfield + 1 tooling + 19 feature-cycle), 457 pts. 283 BCs (244 pre-F2 + 24 SS-15 + 15 SS-16 ARP), 24 VPs (23 locked + VP-024 ARP draft), 1496 tests green, holdout 0.967. develop HEAD cced898; main HEAD 3e29891 (v0.6.0). ARP feature: F1 approved — SS-16 (18-24 new BCs), VP-024, ADR-008, E-16 (5-6 stories). MITRE T0830+T1557.002. F4 wave 40 (STORY-111): etherparse 0.20 + DecodedFrame + ArpFrame + symmetric-unreachable design + non-panicking extract_arp_frame placeholder delivered PR #236.

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0 — Brownfield Ingestion | PASSED | 2026-05-19T20:00:00Z |
| Phase C — Lesson Backlog Remediation | PASSED | 30/30 lessons; PRs #69–#99 |
| Phase 1 — Spec Crystallization | **PASSED** 2026-05-21 | 20 L2 shards, 217 BCs, 20 VPs, 4 supplements |
| Phase 2 — Story Decomposition | **PASSED** 2026-05-21 | 49 stories / 11 epics / 27 waves; input-hash drift CLEAN |
| Phase 3 — TDD Implementation | **PASSED** 2026-05-31 | 48/48 stories, 27/27 waves; develop HEAD 6158e6e |
| Phase 4 — Holdout Evaluation | **PASSED** 2026-06-01 | mean 0.949; detail: cycles/v0.1.0-greenfield-spec/ |
| Phase 5 — Adversarial Refinement | **PASSED** 2026-06-01 | Adversary gate 3/3; trajectory: P1-P14 GATE |
| Phase 6 — Formal Hardening | **PASSED** 2026-06-02 | 8 Kani VPs proven; fuzz 21.7M/0; 20 VPs LOCKED |
| Phase 7 — Convergence | **PASSED + RELEASED** 2026-06-08 | 1126 tests; consistency 8/8 CONSISTENT |
| Release v0.1.0..v0.4.0 | **RELEASED** | v0.1.0 greenfield; v0.2.0 timestamp; v0.3.0 multi-tag; v0.4.0 Modbus |
| Maintenance MITRE v19 remap (issue #222) | **RELEASED in v0.5.0** 2026-06-10 | 3-pass adversarial CONVERGED; PR #223→develop; PR #224→main |
| Release v0.5.0 | **RELEASED** 2026-06-10 | c2df1b5; 4 binaries; run 27313698900 SUCCESS |
| Feature #8 DNP3 — F2 Spec Evolution | **COMPLETE** 2026-06-10 | SS-15 24 BCs; 268 total; MITRE 23/15/8 |
| Feature #8 DNP3 — F3 Story Decomposition | **PASSED** (human-gated 2026-06-11) | 5 stories STORY-106..110; linear chain; VP placements |
| Feature #8 DNP3 — F4 Delta Implementation | **COMPLETE** 2026-06-12 | waves 35-39 / stories 106-110 ALL DELIVERED |
| Feature #8 DNP3 — F5 Scoped Adversarial + Remediation | **COMPLETE** 2026-06-12 | PR #230 e685664; 4 issues fixed; 10-pass 3/3 CLEAN |
| Feature #8 DNP3 — F6 Formal Hardening | **COMPLETE** 2026-06-12 | PR #231 a125c69; 9/9 Kani; 89% mut kill; VP-023 LOCKED |
| Feature #8 DNP3 — F7 Delta Convergence | **CONVERGED** 2026-06-12 | 5-dim convergence; 6 fresh-context passes (final 3 consecutive CONVERGED); PRs #232/#233; BC-2.15.009 v1.3 |
| Release v0.6.0 | **RELEASED** 2026-06-12 | PR #234 (release/0.6.0 → main 3e29891); fixup fb3935c; tag v0.6.0; 4 binaries (release.yml); develop merge-back 04f8ccb |
| Maintenance: Dependabot sweep (post-v0.6.0) | **COMPLETE** 2026-06-12 | 5 PRs merged (#203/#204/#207/#235/#206), 2 closed (#202 superseded, #205 deferred); develop 31d1231 |
| Feature: ARP analyzer — F1 Delta Analysis | **PASSED** (human-gated 2026-06-12) | DecodedFrame{Ip,Arp} integration, ADR-008 planned, F2→F7 authorized; artifacts: `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md` |
| Feature: ARP analyzer — F2 Spec Evolution | **CONVERGED 3/3** (Pass 33, 2026-06-13); 33 passes total; P31/P32/P33 consecutive CLEAN; F2 strict-whole-corpus adversarial gate SATISFIED | 4-slice method; ARP delta SETTLED P9+; corpus-wide debt flushed P14-25; P26/P28/P31/P32/P33 CLEAN; P27/P29/P30 reset cycles surfaced+fixed genuine defects; trajectory: `phase-f5-adversarial/arp-f2-convergence-trajectory.md` |
| Feature: ARP analyzer — F3 Story Decomposition | **CONVERGED 3/3** (Passes 36/37/38, 38 passes total incl. post-P26/P33 consistency flushes); F3 STRICT WHOLE-CORPUS ADVERSARIAL GATE SATISFIED; F3 human gate PASSED (D-070, 2026-06-14) | STORY-111..115 (E-16, 47 pts, linear chain); 15 SS-16 BCs; waves 40-44 holdouts; HS-INDEX v1.7; wave-schedule v1.3; SS-15 fully de-NEW-ed; corpus canonical 457 pts; trajectory: phase-f5-adversarial/arp-f3-convergence-trajectory.md |
| Feature: ARP analyzer — F4 Delta Implementation | **IN PROGRESS** — STORY-111 DELIVERED (PR #236 cced898; wave 40); STORY-112 Step-4.5 CONVERGED (final HEAD c68964d; 1512 tests green; 3/3 clean logic passes; BC-5.39.001; PR pending; wave 41); NEXT = demo-recorder → pr-manager (STORY-112) | per-story TDD; waves 40-44; v0.7.0 target |

## Session Resume Checkpoint (2026-06-15 — F4 ARP DELTA-IMPLEMENTATION; STORY-112 Step-4.5 CONVERGED c68964d; NEXT = demo-recorder → pr-manager)

**Previous checkpoint (2026-06-14 — F4 IN PROGRESS; STORY-111 DELIVERED PR #236 cced898; STORY-112 STUB COMMITTED 0227d9c; NEXT = test-writer) archived to:
`cycles/feature-arp-v0.7.0/session-checkpoints.md`**

### A. EXACT PIPELINE POSITION

- **Project:** wirerust. Mode: FEATURE. Active feature: ARP security analyzer + etherparse
  0.16→0.20 migration. GitHub issue #9. Release target: **v0.7.0**.
- **F1 PASSED** (human-gated 2026-06-12, D-066).
- **F2 CONVERGED 3/3** — Passes 31/32/33 consecutive CLEAN; strict whole-corpus gate SATISFIED.
- **F3 CONVERGED 3/3** — Passes 36/37/38 consecutive CLEAN; 38 passes total; gate SATISFIED.
  F3 human gate PASSED (2026-06-14, D-070) — STORY-111..115 accepted as-is.
- **F4 Delta-Implementation: IN PROGRESS — AUTHORIZED (D-070). Linear chain STORY-111→112→113→114→115; waves 40-44.**
  - **Wave 40 / STORY-111: DELIVERED** — PR #236 merged to develop (merge commit cced898; repo
    enforces merge-commits only, not squash). D-073. Delivered: etherparse 0.20 migration,
    DecodedFrame{Ip,Arp} enum, ArpFrame struct, decode_packet→Result<DecodedFrame>,
    symmetric-unreachable ARP decode (D-072), non-panicking extract_arp_frame placeholder,
    BC-2.02.009 v1.7, VP-008 fuzz-harness return-type update. 53 suites green; clippy/fmt clean.
    Worktree for STORY-111 removed.
  - **Wave 41 / STORY-112: Step-4.5 CONVERGED.** Final HEAD `c68964d` on branch
    `worktree-issue-9-story-112-arp-extract-frame` (base `cced898`). 3/3 clean logic passes
    (frozen diff at 365dbeb); 1512 tests passed / 0 failed; rustfmt 1.9.0-stable (CI-matched).
    All 10 AC banners GREEN. 4 comment-only fix commits resolved non-blocking findings
    (F-1/F-2/F-3/Residual-F-1). VP-024 Sub-A Kani harnesses deferred to F6 (todo!()
    skeletons; verification_lock:false; D-062 precedent). STORY-112.md v1.4 committed
    (92797a2). input-hash: 8a4d566 (unchanged). **NEXT = demo-recorder → pr-manager (9-step PR).**
  - STORY-113/114/115: NOT STARTED.
- **Decisions active: D-047..D-073; do NOT re-adjudicate D-068/D-069/D-071/D-072/D-073.**
- **F3-OBL-STORY114-001/002/003 REVOKED** (D-069).

### B. INPUT-HASH STATUS

| Story | Stored | Computed | Status |
|-------|--------|----------|--------|
| STORY-111 | d05149f | d05149f | MATCH — DELIVERED |
| STORY-112 | 8a4d566 | 8a4d566 | MATCH |
| STORY-113 | a767d96 | 7c61bae | **STALE** |
| STORY-114 | e2f1c95 | 5705a10 | **STALE** |
| STORY-115 | 5ca9835 | 2e0eca2 | **STALE** |

STORY-113/114/115 went STALE because arp-architecture-delta v1.16 (D-072) is an input to all
ARP stories. Re-stamp before STORY-113 delivery (non-blocking for STORY-112). Verify stories
carry no stale "lax_ip_triple NOT unreachable / explicit routing" framing before re-stamping
(D-072 symmetric-unreachable is authoritative).

### C. DECISIONS CONFIRMED ACTIVE (do not re-adjudicate)

- **D-068:** Benign GARP emits `mitre_techniques: []`; T0830/T1557.002 only on GARP-that-conflicts
  (BC-2.16.014). Research-backed (`.factory/research/arp-garp-mitre-attribution.md`).
- **D-069:** IcsImpact Display = "Impact (ICS)" — CORRECT. SUPERSEDES D-067. src/mitre.rs:91 CORRECT.
- **D-072:** Symmetric-unreachable design authoritative: decode_packet intercepts ARP in both strict
  and lax arms; both `strict_ip_triple` AND `lax_ip_triple` have provably-dead `unreachable!` ARP arms.
  arp-architecture-delta v1.16, ADR-008 v2.1, BC-2.02.009 v1.7, BC-2.16.015 v1.3.
- **D-073:** STORY-111 DELIVERED — PR #236 merged develop cced898.

### D. DURABLE MITIGATIONS / SCOPE NOTES (preserved for adversary dispatches)

- BC-note citations are intentionally version-less — do not flag missing versions.
- vp-007 numeric "23 seeded / 15 emitted" = current src; "25 / 17" = PLANNED post-STORY-114.
- `storm_findings` is the canonical ArpAnalyzer field (STORY-113:254 + BC-2.16.010).
- STORY-115 v1.1 uses `storm_findings` (not `storm_findings_count`) — CORRECT (P32).
- `ThreatCategory::Suspicious` is VALID (10 variants; P23 FALSE POSITIVE).
- `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` is canonical shipped constant — do NOT rename in spec.
- `prd.md §~298 §[pass-13-2026-06-13]` is intentional immutable-history prose.
- `wave-40-44-holdout.md "D14" = BC-2.16.014`. `mitre-arp-additional-detections.md "D14"` = deferred Unicast ARP candidate — do NOT flag.
- Canonical point total = 457 (410 pre-ARP + 47 ARP); wave-table excl STORY-091 = 452; tooling = 5.
- SS-15 reset set = SIX windowed fields; `parse_errors` is LIFETIME-only — NEVER in reset set.
- BC-2.15.014 EC-006 + Invariant 7 enumerate exactly SIX fields (post-P33 flush v2.0).
- SS-15 FC 0x13 = SAVE_CONFIGURATION (IEEE 1815-2012); SAVE_CONFIG only in sealed changelog.
- Reciprocal Related-BCs complete (post-P33): BC-2.15.014↔016, 016↔010, 015↔024, 022↔016.
- Src citations are SYMBOL-ANCHORED — do NOT flag missing line numbers; do NOT re-add them.
- All spec-changelog ACTIVE-zone (2026-06-12+) entries carry Artifacts-changed tables (P34 flush).
- All spec-changelog ACTIVE-zone entries are de-pinned — remaining numerics are sealed history/src-refs.
- T1692.001 is canonical (not T0855). T0855 appears only in sealed remap-history prose.
- VP-023 Kani scope = BC-2.15.001..007 ONLY (BC-2.15.008 is unit-test-only; no Kani harness).
- F4 dead_code lint for deferred fields `storm_rate` (STORY-114) / `storm_counters` (STORY-113) is OUT OF F3 SCOPE — carry to F4 implementer.
- All process-gap justifications archived to §F of prior checkpoint and Drift Items table.

### E. RESUME PROCEDURE

**Step 1 (BLOCKING):** `vsdd-factory:factory-worktree-health`

**Step 2 — Verify SHAs (ground truth; re-verify live):**
```
git -C /Users/zious/Documents/GITHUB/wirerust rev-parse --short HEAD
# expect: cced898 (or newer if work advanced)
git -C /Users/zious/Documents/GITHUB/wirerust/.factory log -1 --format='%h %s'
# expect: this commit or newer
gh pr list --state open
# expect: none (OR a STORY-112 PR if work advanced since checkpoint)
```

**Step 3 — STORY-112 convergence verification (source of truth = git):**
```
git -C /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112 log --oneline -3
# expect: c68964d (or later) — final HEAD after all comment-fix commits
git -C /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112 status --short
# expect: clean (no output)
cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112 && cargo test --all-targets
# expect: 1512 passed / 0 failed
```
- Step-4.5 CONVERGED at c68964d (3/3 clean logic passes; 1512 tests green).
  NEXT = **demo-recorder** → **pr-manager 9-step PR** → worktree cleanup.
- If git log/status/cargo test diverge from above → re-verify live state before dispatching
  demo-recorder.

**Step 4 — Before STORY-113 delivery:**
- Re-stamp STORY-113/114/115 input-hashes:
  `bin/compute-input-hash --write .factory/stories/STORY-113.md` (and 114, 115).
- Verify each story has NO stale "lax_ip_triple NOT unreachable / explicit routing" framing
  (D-072 symmetric-unreachable is authoritative); fix before re-stamping if present.

**Step 5 — CI/toolchain note (PG-ARP-F4-CI-FMT-TOOLCHAIN):**
Before opening any PR, run `rustup update stable` + `cargo fmt --all --check` to match CI
rolling-stable (dtolnay/rust-toolchain@stable; local must match).
pr-manager must complete steps 7-9 + consolidated report (PG-ARP-F4-PRMGR-MERGE-SHORTSTOP).

**Step 6 — Mid-cycle policy:**
User directed strict per-story adversarial convergence (Step-4.5, 3/3). Do NOT re-deliver
STORY-111. Do NOT revert D-070/D-071/D-072/D-073.

### F. KEY ARTIFACT POINTERS

- ARP architecture delta: `.factory/specs/architecture/arp-architecture-delta.md` (v1.16)
- ADR-008: `.factory/specs/architecture/adr-008.md` (Decision 3 v2.1)
- F2 convergence trajectory (33 passes): `.factory/phase-f5-adversarial/arp-f2-convergence-trajectory.md`
- F3 convergence trajectory (38 passes): `.factory/phase-f5-adversarial/arp-f3-convergence-trajectory.md`
- F1 delta analysis: `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md`
- STORY-112: `.factory/stories/STORY-112.md`; worktree: `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112`
- Archived checkpoints: `.factory/cycles/feature-arp-v0.7.0/session-checkpoints.md`

## Decisions Log

D-001..D-054 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (D-047..D-054 in Feature #8 / v0.5.0 section).

| ID | Decision | Date |
|----|----------|------|
| D-055 | Feature #8 F3 human gate PASSED — 5 stories accepted; VP placements; strictly-linear chain. F4 TDD authorized. | 2026-06-11 |
| D-056 | STORY-106 DELIVERED — PR #225 d0f3586. VP-023 4/4 Kani SUCCESSFUL. | 2026-06-11 |
| D-057 | STORY-107 DELIVERED — PR #226 ebb4751. Carry-walk gate-before-count; STORY-106 frames wire-valid. | 2026-06-11 |
| D-058 | STORY-108 DELIVERED — PR #227 9c03fde. 5-pass adversarial 3/3 CLEAN. DRIFT-DNP3-DIRECTION-001 recorded. | 2026-06-11 |
| D-059 | STORY-109 DELIVERED — PR #228 34443f9. 13-pass 3/3 CLEAN; MitreTactic::IcsImpact; VP-007 seed. | 2026-06-12 |
| D-060 | STORY-110 DELIVERED — PR #229 ddfa576. Rule 6 + CLI flags + VP-004 oracle. F4 COMPLETE. | 2026-06-12 |
| D-061 | Feature #8 F5 COMPLETE — PR #230 e685664. 4 issues fixed (DIR-bit P0; unexpected-source P0; IcsImpact display; resync). 10-pass 3/3 CLEAN. | 2026-06-12 |
| D-062 | Feature #8 F6 HARDENED — PR #231 a125c69. 9/9 Kani; 89% mut; 3.19M fuzz/0; VP-023 LOCKED v1.5; VP-004 relocked. 4/4 F6 obligations SATISFIED. | 2026-06-12 |
| D-063 | Feature #8 F7 CONVERGED — 5-dim delta; 6 fresh-context adversarial passes (final 3/3 CONVERGED); F-S2-001/F-S1-001/F-PG-001/F-CC-001..004 remediated (PRs #232/#233). develop f217f27. | 2026-06-12 |
| D-064 | v0.6.0 RELEASED — gitflow release/0.6.0 → PR #234 → main 3e29891; fixup fb3935c; tag v0.6.0; GitHub Release WITH 4 binaries (release.yml auto-build); develop merge-back 04f8ccb. DNP3 TCP analyzer is the headline feature. | 2026-06-12 |
| D-065 | Dependabot sweep post-v0.6.0 COMPLETE — #203 serde_json/#204 assert_cmd/#207 clap/#206 rayon routine bumps merged; #235 manual SHA-pin actions/checkout 6.0.3 (replacing tag-ref #202); #205 etherparse 0.16→0.20 closed and deferred as migration story (new drift DRIFT-ETHERPARSE-0.20-MIGRATION-001). develop 31d1231. | 2026-06-12 |
| D-066 | Feature ARP analyzer F1 gate APPROVED — full F1-F7, release target v0.7.0. DecodedFrame{Ip,Arp} integration (ADR-008); ArpAnalyzer bounded IP↔MAC table; etherparse 0.20 sub-delta A. SS-16 (18-24 BCs), VP-024, ADR-008, E-16 (5-6 stories). MITRE T0830+T1557.002. Detections: spoof/cache-poison + GARP + storm/rate + research-agent additional. DRIFT-ETHERPARSE-0.20-MIGRATION-001 folded in. | 2026-06-12 |
| D-067 | IcsImpact Display adjudication — canonical Display = "Impact" (spec correct; BC-2.10.002 PC3/PC4, PRD §85/823, cap-10, spec-changelog unanimous). src/mitre.rs:91 "Impact (ICS)" is DEVIANT (introduced F-F5-002 as "No BC change" tactical test fix). " (ICS)" suffix does NOT break merge-by-name report bucketing (terminal.rs render_findings_grouped keys on MitreTactic enum variant, not Display string); severity LOW (terminal section-header label only). F2 SPEC CHANGE: NONE — F2 3/3 strict-whole-corpus convergence preserved unaffected. Fix folded into STORY-114 (obligations: 1-line mitre.rs:91 fix; HS-008:75 "Impact (ICS)"→"Impact"; Display unit test; two-bucket enum-level report test). F2→F3 gate condition SATISFIED. **SUPERSEDED BY D-069.** | 2026-06-13 |
| D-068 | Benign gratuitous ARP emits mitre_techniques: [] (LOW/Anomaly severity); T0830 + T1557.002 apply ONLY when GARP conflicts with binding table (BC-2.16.014). Research-backed: MITRE ATT&CK v19.1 T1557.002/DET0387 + T0830; arpwatch/Zeek/Suricata all gate techniques on conflict-detection. Corrected latent over-tagging defect in BC-2.16.003 (→v1.7) and ADR-008 (→v2.0). Propagated to §3.3/STORY-113 AC-003, holdouts, and error-taxonomy. | 2026-06-14 |
| D-069 | IcsImpact Display canonical = "Impact (ICS)" (distinct from Enterprise "Impact" TA0040). SUPERSEDES D-067. Research-backed: MITRE TA0040 (Enterprise Impact) vs TA0105 (ICS Impact) are distinct tactic families; WCAG 2.4.6 requires unique headings/labels. src/mitre.rs:91 "Impact (ICS)" is CORRECT — not deviant. STORY-114 D-067 revert obligations (F3-OBL-STORY114-001/002/003) REVOKED. 2 shipped DNP3 F5 distinctness tests preserved (they test enum-variant identity, not Display string equality). Spec side corrected: BC-2.10.002 (→v1.5), PRD §85/882, ADR-007, arp-architecture-delta §5.0. | 2026-06-14 |
| D-070 | Feature ARP F3 human gate PASSED (2026-06-14) — STORY-111..115 (E-16, 47 pts) accepted as-is; F3 strict whole-corpus adversarial convergence SATISFIED (3/3, Passes 36/37/38; 38 passes total + 3 consistency flushes). F4 delta-implementation AUTHORIZED: per-story TDD on the linear chain STORY-111→112→113→114→115; release target v0.7.0. etherparse 0.16→0.20 migration folded into STORY-111 (sub-delta A). Human review questions (scope/MITRE/linear-chain/etherparse) — no changes requested; approved as-is. | 2026-06-14 |
| D-073 | STORY-111 DELIVERED — PR #236 MERGED to develop (merge commit cced898; repo allows merge-commits only). etherparse 0.20 migration + DecodedFrame{Ip,Arp} enum + ArpFrame struct + decode_packet→Result<DecodedFrame> + symmetric-unreachable ARP decode (D-072) + non-panicking extract_arp_frame placeholder + BC-2.02.009 v1.7 + VP-008 fuzz-harness return-type update. 53 test suites green; clippy/fmt clean; Step-4.5 adversarial 3/3. CI Format failure fixed by aligning local toolchain to CI rolling-stable (rustfmt 1.8.0→1.9.0). pr-reviewer APPROVE. Wave 40 complete. NEXT = STORY-112 (extract_arp_frame + ArpAnalyzer stub + main.rs DecodedFrame wiring + VP-024 Sub-A Kani). | 2026-06-14 |
| D-072 | F4-surfaced ARP decode design inconsistency (2026-06-14) — STORY-111 implementation revealed arp-architecture-delta §2.2 contained an internally-inconsistent + non-type-implementable lax design (BLOCK 1 'lax_ip_triple routes ARP / NOT unreachable!' vs authoritative BLOCK 2 'decode_packet intercepts ARP'). lax_ip_triple returns IpTriple and cannot route ARP. Architect ruled SYMMETRIC design authoritative: decode_packet routes ARP in both strict Ok(slice) and lax Err(SliceError::Len) arms; both strict_ip_triple AND lax_ip_triple have provably-dead unreachable! ARP arms; VP-008/VP-024 Sub-A no-panic via decode_packet interception + panic-free extract_arp_frame. Reconciled: arp-architecture-delta v1.16, ADR-008 Decision 3 v2.1, BC-2.02.009 v1.7, BC-2.16.015 v1.3, STORY-111 v1.4 (hash d05149f), STORY-112 v1.3 (hash 8a4d566). SUPERSEDES BC-2.16.015 v1.2 'lax NOT unreachable' fix (which had aligned to the inconsistent BLOCK-1 prose). STORY-111 GREEN implementation was correct throughout and matches the reconciled design — no code change. 2nd F4-surfaced spec defect after D-071; reinforces strict-TDD value. | 2026-06-14 |
| D-071 | F4-surfaced STORY-111 decomposition fix (2026-06-14) — strict-TDD stub-architect Red-Gate (BC-5.38.005 self-check) caught that STORY-111 ACs (001/002/004/007/008) asserted STORY-112's extract_arp_frame end-to-end ARP-decode behavior, unsatisfiable within STORY-111's §6 scaffolding scope and duplicative of STORY-112 AC-006/007/004. Re-scoped STORY-111→v1.1 (scaffolding-only ACs AC-003/005/005b/006/009/010 + AC-005b non-panicking extract_arp_frame placeholder preserving VP-008); added STORY-112 AC-012→v1.1 (decode_packet-level Err("Non-Ethernet/IPv4 ARP frame")) closing the one coverage gap. BC-2.02.009 unedited (primary STORY-111; ARP-Ok postcondition behaviorally satisfied in STORY-112 — story-level framing). Both stories input-hash MATCH (d5bda72/268f53f — body-only edits). 38 strict F3 passes did not catch this (AC-satisfiable-within-dependency-scope feasibility check, not spec-internal-consistency) — validates strict-TDD value. Scoped adversarial re-review of STORY-111/112 in progress before resuming TDD. Worktree stub commit 4e22ef9 was based on old over-scoped STORY-111 and must be re-aligned (extract_arp_frame todo!()→non-panicking placeholder). **F4 scoped post-fix adversarial re-review (2026-06-14) surfaced 7 findings (1 HIGH: BC-2.16.015 lax_ip_triple unreachable! mis-anchor — sibling-propagation gap from BC-2.02.009 v1.6 correction, would have caused a reachable VP-008/VP-024 Sub-A violating panic; 2 MED + 4 LOW: AC-seam/type/count issues). ALL 7 remediated: BC-2.16.015→v1.2 (lax unreachable! → explicit routing; Architecture Anchors corrected), STORY-111→v1.2 (AC-005 seam + AC-005b type + Task-8 + coverage-map), STORY-112→v1.2 (AC count AC-001..AC-011→AC-001..AC-012; input-hash 268f53f→c8c1a64). BC-2.02.009 anchoring judged coherent — no BC split needed. BC-2.16.015 lax-unreachable mis-anchor shows the BC-2.02.009 v1.6 sibling-sweep did not reach SS-16 sibling BCs — cross-subsystem sibling sweep gap. Confirming scoped re-review in progress; on clean → resume STORY-111 TDD.** | 2026-06-14 |

## Blocking Issues

None open.

## Drift Items / Tech Debt Pointers

All items require DF-VALIDATION-001 research-agent validation before GitHub issue filing.
Full tech-debt register: `.factory/tech-debt-register.md`.

| ID | Summary | Status |
|----|---------|--------|
| ADV-HS043-P02-MED-001 | Idle-flow expiry monotonic watermark stalls on multi-epoch captures | ACCEPTED — gated on live-capture support |
| O-07 | rayon declared in Cargo.toml but unused | OPEN P2 |
| O-08 | dns.rs module doc-comment stale | OPEN P3 |
| F-W25-S088-P6-001 | AC-004 warning .contains() — weaker than count-assertion | OPEN — target next main.rs touch or accept |
| RUSTSEC-2026-0097 | rand 0.8.5 unsound (transitive via tls-parser→phf 0.11); upstream-only fix | ACCEPTED-TRANSITIVE |
| FE-001 | pcapng input format not supported — v2 idea | deferred / v2 |
| ACTION-PIN-001 | dtolnay/rust-toolchain @stable/@nightly exempt in pin gate | OPEN P3 |
| PCAP-CORPUS-001 | E2E pcap test-corpus storage backend — PR #221 landed; large pcaps gitignored | TABLED — human decision pending |
| MITRE-V19-REMAP-001 | T0855/T0856 remap; PR #223 develop; PR #224 main | CLOSED — RELEASED in v0.5.0 |
| DRIFT-F2-COUNT-001 | Stale "15 seeded IDs" count in BC-2.10.006.md, prd-supplements, HS-008/009 | DEFERRED |
| DRIFT-SUPERPOWERS-001 | docs/superpowers/ carries stale pre-F2 catalog | DEFERRED |
| SEC-106-001..002 | CWE-129 gate-before-count; CWE-400 MAX_MASTER_ADDRS cap | SATISFIED |
| STORY-107-CARRY-001 | BC EC-004/EC-006/PC4 deferrals; multi-block indexing | SATISFIED |
| DRIFT-DNP3-DIRECTION-001 | source_ip resolution port-heuristic-only; direction-aware deferred post-v0.6.0 | DEFERRED |
| DRIFT-MITRE-EMITTED-LABEL-001 | kani EMITTED_IDS T0835/T0831 over-label; system-level | DEFERRED LOW |
| DRIFT-BC-2.15.024-EC006-PROSE-001 | EC-006 prose vs BC-2.15.009 PC5 conflict; PO prose-refresh | DEFERRED LOW |
| DRIFT-SEMGREP-001 | semgrep absent; manual CLEAN; non-blocking | DEFERRED LOW |
| DRIFT-ENGINE-CHECKOUT-GUARD-001 | adversary dispatch template missing checkout-guard; engine fix needed | ENGINE-NOTE HIGH |
| DRIFT-ENGINE-PRMGR-REPORT-001 | pr-manager omitting consolidated report on 4/5 PRs; engine fix needed | ENGINE-NOTE MEDIUM |
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 | release-config.yaml human-prose fields refreshed this burst (human_approval_prompt version-agnostic; test counts updated to 1496 / 23 VPs; release.yml stale note corrected); engine template follow-up (version_sources) DEFERRED | PARTIALLY RESOLVED |
| DRIFT-DNP3-DOC-T0814-COMPLETENESS-001 | RESOLVED in v0.6.0 — README/CHANGELOG T0814 ENABLE/DISABLE_UNSOLICITED trigger sources added on release branch; also corrected pre-existing README "—" technique error for unsolicited-response row → T0814 | RESOLVED |
| DRIFT-BC-INPUTHASH-TBD-001 | all 24 SS-15 BC files carry input-hash:TBD; compute-input-hash scopes to .factory/stories/ per CLAUDE.md; by-design; known/accepted, non-blocking | BY-DESIGN LOW |
| PG-F7-001 | BC version bump must re-stamp all consuming stories in same burst; F4/F5/F6 gates run live compute-input-hash --scan not trust STATE value. Policy candidate: DF-INPUT-HASH-CANONICAL-001 sub-rule. Backing: lessons.md PG-F7-001. | DEFERRED — next feature cycle |
| PG-F7-002 | After behavior-changing adjudication, grep + re-validate all holdout assertions on the changed path against impl. Policy candidate: F5 remediation playbook step. Backing: lessons.md PG-F7-002. | DEFERRED — next feature cycle |
| PG-F7-003 | Adjudicating agent must Read() current BC text and verify each Invariant before writing "BC needs no update". Engine agent-prompt note. Backing: lessons.md PG-F7-003. | DEFERRED — engine agent-prompt note |
| PG-F7-004 | DF-SIBLING-SWEEP v5: BC Invariant text change must sweep BC-INDEX titles + consuming-story body notes. Policy candidate: DF-SIBLING-SWEEP-001 protocol-BC sub-rule. Backing: lessons.md PG-F7-004. | DEFERRED — next feature cycle |
| PG-F7-005 | Story status (body frontmatter + STORY-INDEX) advances to completed at merge, not draft. Add to per-story delivery close-out. Backing: lessons.md PG-F7-005. | DEFERRED — engine workflow note |
| PG-F7-006 | Shipping a feature moves README planned→implemented + adds CHANGELOG Unreleased entry at delivery, not release scramble. Backing: lessons.md PG-F7-006. | DEFERRED — engine workflow note |
| PG-F7-007 | Agents must check gh run list for in-flight tag-triggered workflows before reporting missing CI/release assets. Backing: lessons.md PG-F7-007. | DEFERRED LOW — engine devops checklist note |
| DRIFT-ETHERPARSE-0.20-MIGRATION-001 | etherparse 0.20 adds Arp variants to NetSlice/LaxNetSlice/InternetSlice; non-exhaustive match at src/decoder.rs:210,232. Folded into ARP analyzer feature cycle (D-066, sub-delta A). | IN-PROGRESS — ARP feature cycle |
| PG-ARP-F2-003 | Pass-14 field-rename sweep scoped to .factory/specs/ only — MISSED .factory/holdout-scenarios/ sibling layer; 16 HS files caught at Pass 15. DF-SIBLING-SWEEP must include holdout-scenarios in propagation perimeter for any Finding-schema change. | DEFERRED — policy codification |
| PG-ARP-F2-004 | PO burst appended second `version:` YAML frontmatter key (inv-01) instead of replacing existing one, introducing malformed YAML caught at Pass 15 (C-04). Version bumps must replace-in-place; pre-commit dup-key lint recommended. | DEFERRED — policy codification |
| PG-ARP-F2-005 | Sweep globs must cover sibling naming variants (chunk*-eval.md missed chunk3-reeval.md; caught Pass 16). Partial-fix discipline: when fixing one of N instances of same defect, enumerate ALL siblings before committing (ADR-005 :74 missed after :108 fixed; api-surface STORY-114 introduced when arp-architecture-delta already cited STORY-111). | DEFERRED — policy codification |
| PG-ARP-F2-006 | Holdout-scenarios carry count assertions (tactic/seeded/emitted/cat-only) that drift when feature cycles change the MITRE catalog. HS-008/009/025 carried greenfield-era counts (16 tactics/15 seeded/5 emitted/9 cat-only) not swept across DNP3 cycle. F-cycle close-out must sweep holdout count-assertions. Candidate: extend DF-CANONICAL-FRAME-HOLDOUT-001 / holdout-maintenance policy. | DEFERRED — policy codification |
| PG-ARP-F2-007 | src-line-anchor drift class — feature cycles that insert code into a shared file (dispatcher.rs via Modbus/DNP3) leave EVERY citing BC's anchors stale. F-cycle close-out must re-run anchor-resync sweep across ALL BCs citing touched src files (dispatcher.rs→ss-04/ss-05; mitre.rs→ss-10 [done]; findings.rs→ss-09; reassembly/http/tls→ss-04/06/07 [verify P19]). Candidate: anchor-drift lint or F-cycle anchor-resync checklist. | DEFERRED — policy codification |
| PG-ARP-F2-008 | Under STRICT zero-any-severity whole-corpus, each remediation burst can introduce ~1-2 new trivial propagation/whitespace items (version-pin lag, blank-line residue) the next pass flags — asymptotic. Mitigation: drop brittle current-state version-pins (done P22 D-01), and treat sustained 0 CRIT/HIGH/MED as practical convergence if LOW-cosmetic churn persists. (Human elected strict 3/3; continuing per directive.) Corpus substantively converged as of P22. | NOTED — asymptotic LOW churn; version-pin hardening done |
| PG-ARP-F2-009 | F5 FlowKey-accessor fix (v2.2) swept 5 of 7 ss-14 direction-resolution BCs but missed 018/020; sibling-sweep completeness for code-vs-spec API fixes must enumerate ALL sibling BCs in same subsystem before closing. STORY input-hash dup-key (TBD placeholder + real hash both present) is a new frontmatter-validity defect class — pre-commit dup-key lint recommended. Caught at Pass 30; REMEDIATED. | DEFERRED — policy codification |
| DRIFT-PRD-V120-MBAPFRAMER-001 | PRD v1.20 delta:285 "C-23 was MbapFramer" historical rationale was factually wrong — no MbapFramer component ever existed; ss-15/DNP3 was renumbered C-23→C-24 when ARP took C-23. | RESOLVED — PRD v1.22 corrected MbapFramer prose (Pass-22 burst 2026-06-14) |
| F3-OBL-STORY114-001/002/003 | D-067 obligations (mitre.rs:91 "Impact"→"Impact (ICS)", HS-008 alignment, test obligations). **ALL REVOKED by D-069 (2026-06-14): src/mitre.rs:91 "Impact (ICS)" is CORRECT — canonical Display; no revert required. VP-007 obligation in STORY-114 unchanged.** | REVOKED — superseded by D-069 |
| DNPXX-SOURCE-RENAME-001 | src constant `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` is an ugly placeholder-style name shipped in v0.6.0; candidate code-cleanup rename DNPXX_→DNP3_ (6 files: dnp3.rs/cli.rs/main.rs/tests + arp-delta + STORY-110) — OUT OF F3 SCOPE (spec/story only); defer to a code-quality maintenance sweep. Requires DF-VALIDATION-001 research-agent validation before GitHub issue filing. | DEFERRED LOW |
| DF-SIBLING-SWEEP-CROSS-SS-001 | F-cycle BC-invariant corrections that change routing semantics shared across subsystems (e.g., strict/lax unreachable! vs explicit-routing pattern in SS-02+SS-16) MUST sweep cross-subsystem sibling BCs, not only the originating subsystem. BC-2.02.009 v1.6 sibling-sweep missed BC-2.16.015 SS-16 sibling; caught at F4 scoped re-review as HIGH would-be-panic. Candidate: extend DF-SIBLING-SWEEP-001 with a cross-subsystem enumeration rule for shared decode-architecture invariants. (PG-ARP-F4-SIBLING-SWEEP-CROSS-SUBSYSTEM) | DEFERRED — policy codification |
| DRIFT-ARP-STORY-113-115-HASH-STALE | STORY-113/114/115 input-hashes stale after arp-architecture-delta v1.16 (D-072): stored a767d96/e2f1c95/5ca9835 vs computed 7c61bae/5705a10/2e0eca2. Re-stamp with `bin/compute-input-hash --write` before STORY-113 delivery; verify no stale "lax NOT unreachable" framing first. Non-blocking for STORY-112. | OPEN — pre-STORY-113 gate action |
| PG-ARP-F4-REDBANNER-SWEEP | RED-gate banner sibling-sweep missed across 3 successive comment-fix bursts (module docstring fixed but per-test banners + AC-004 block left stale). Recurrence of DF-SIBLING-SWEEP-001 in the doc-comment dimension. Candidate: extend sibling-sweep checklist to enumerate per-test section banners + doc-comments as explicit targets when module-level status changes. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — policy codification follow-up |
| PG-ARP-F4-PRECLEAR-PROPAGATION | Orchestrator propagated a prior adversary pass's "leave as-is" pre-clearance into a fix dispatch (AC-004 banner), which a later fresh pass overturned as HIGH. Lesson: fix dispatches MUST NOT pre-clear regions based on an earlier pass's judgment; each fresh adversary examines the full perimeter pre-clearance-free. Candidate: DF-ADVERSARY-METHODOLOGY-001 language extension. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — policy codification follow-up |
| PG-ARP-F4-GUARD-WORDING | Checkout-guard premise "main repo does NOT have this function" was inaccurate (main repo has the STORY-111 None placeholder). Guard must key on BODY CONTENT (placeholder None vs real extraction) / the transitional error string, not function presence. Candidate: extend DF-ADVERSARY-CHECKOUT-GUARD-001. Detail: `cycles/feature-arp-v0.7.0/lessons.md`. | DEFERRED — extend DF-ADVERSARY-CHECKOUT-GUARD-001 |

## Deferred Next-Work Backlog

**1. PCAP-CORPUS-001:** R2/B2/Drive-SA — TABLED, human decision pending.

**2. Roadmap (post-DNP3):** #3 C2 beaconing | #4 CSV+SQLite reporters | #6 rayon parallel (relates to O-07).

**3. etherparse 0.20 migration:** DRIFT-ETHERPARSE-0.20-MIGRATION-001 — folded into ARP analyzer feature cycle (D-066, sub-delta A); IN-PROGRESS.

## Governance Policy

Full policy text: `.factory/policies.yaml`.

| Policy | Severity |
|--------|----------|
| DF-VALIDATION-001 | HIGH |
| DF-SIBLING-SWEEP-001 (v4) | CRITICAL |
| DF-PR-MANAGER-COMPLETE-001 | HIGH |
| DF-ADVERSARY-METHODOLOGY-001 | HIGH |
| DF-AC-TEST-NAME-SYNC-001 (v2) | MEDIUM |
| DF-CONVERGENCE-BEFORE-MERGE-001 | CRITICAL |
| DF-DEVELOP-FRESHNESS-001 | HIGH |
| DF-ADVERSARY-TOOLCHAIN-PAIRING-001 | MEDIUM |
| DF-INPUT-HASH-CANONICAL-001 | HIGH |
| DF-ADVERSARY-CHECKOUT-GUARD-001 | HIGH |
| DF-TEST-CITATION-SWEEP-001 | HIGH |
| DF-TEST-NAMESPACE-001 | MEDIUM |
| DF-CONSISTENCY-AUDIT-POST-FIXBURST-001 | HIGH |
| DF-CANONICAL-FRAME-HOLDOUT-001 | CRITICAL |
| DF-BC-COMPLETENESS-SWEEP-001 | HIGH |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`; F6 hardening `cycles/feature-8-dnp3-v0.5.0/F6-hardening/`.
- Issues: #104/#102 CLOSED (PRs #194/#195), #100 RELEASED v0.2.0, #101 OPEN-DEBT, #103 DEFERRED. Dependabot sweep 2026-06-12 cleared all v0.6.0-era PRs (5 merged: #203/#204/#207/#235/#206; 2 closed: #202 superseded by #235, #205 etherparse deferred — see DRIFT-ETHERPARSE-0.20-MIGRATION-001). All actions SHA-pinned (actions/checkout now at df4cb1c # v6.0.3); pin gate enforced (PR #196, PR #235).
