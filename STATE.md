---
pipeline: FEATURE_MODE
phase: F2
phase_status: "FEATURE MODE issue #62 — F3 round-2 fix-burst complete: STORY-120 AC-005 CRITICAL scope error fixed (out-of-scope *mitre/no_collapse → in-scope show_mitre_grouping/collapse_findings); ADR-0003 migration map corrected; AC-001 doc-comments aligned to ADR; collapse_findings_from_flag declared UNCHANGED; dep-graph acyclicity prose 71→72. F3 convergence re-streak pending."
active_feature: "E-8 / #62 TerminalReporter enum-of-modes refactor — F1..F3 IN PROGRESS; STORY-120 created (28 sites, wave 48); STORY-119 depends_on [STORY-120]; F3 adversary re-streak pending after round-2 fix-burst; release target v0.9.0"
feature_arp_status: "v0.7.0 RELEASED 2026-06-16 — ARP Security Analyzer (E-16, issue #9); PR #256 dd8e142; tag v0.7.0; 4 binaries (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu)"
feature_8_status: "v0.6.0 RELEASED 2026-06-12 — DNP3 TCP analyzer; F7 5-dim CONVERGED; tag v0.6.0 + 4 binaries"
product: wirerust
mode: brownfield
timestamp: 2026-06-18T13:00:00Z
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-06-17
maintenance_started_at: "2026-06-17"
maintenance_completed_at: "2026-06-17"
maintenance_findings_count: 48
maintenance_critical_count: 0
maintenance_blocking: false
maintenance_fixes_applied: 2
maintenance_fixes_deferred: 5
maintenance_fixes_pending: 0
maintenance_report: ".factory/maintenance/sweep-report-2026-06-17.md"
maintenance_sweep_progress:
  dependency-audit: COMPLETE
  doc-drift: COMPLETE
  pattern-consistency: COMPLETE
  holdout-freshness: COMPLETE
  performance-regression: COMPLETE
  spec-coherence: COMPLETE
  tech-debt-register: COMPLETE
  risk-assumption-monitoring: COMPLETE
  DTU-fidelity: N/A
  accessibility-regression: N/A
  design-drift: N/A
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
develop_head: bec13ba
develop_head_confirmed: "bec13ba (chore: merge main (v0.8.0) back into develop — gitflow sync, branch-protection bypass per prior-cycle convention)"
arp_f6_hardening_status: "COMPLETE — 5/5 Kani SUCCESSFUL (46/46 project-wide), VP-024 v2.3 LOCKED, fuzz VP-008 16.2M/0, mutants 98.9%"
arp_f7_convergence_status: "CONVERGED — 5-dim met; v0.7.0 RELEASED"
arp_followups_status: "DISPOSITIONED — item 5 fixed (BC-2.10.007 v1.8 de-PLANNED 25/17); issues #252-255 filed (post-release); CR-001/CR-002/FU-STORM-NEW-ATTR/BC-2.10-COUNT-POSTMERGE dropped/resolved. RELEASE-READY."
factory_artifacts_head: see git -C .factory log -1  # updated by this burst
main_head: 73034da
released_version: v0.8.0
released_at: "2026-06-17"
release_tag: v0.8.0
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.8.0
release_commit: 73034da
release_yml_run: "27732692087 COMPLETED conclusion=success — 4 binaries CONFIRMED PUBLISHED: wirerust-v0.8.0-aarch64-apple-darwin.tar.gz, wirerust-v0.8.0-x86_64-apple-darwin.tar.gz, wirerust-v0.8.0-x86_64-pc-windows-msvc.zip, wirerust-v0.8.0-x86_64-unknown-linux-gnu.tar.gz. GitHub Release isDraft=false at https://github.com/Zious11/wirerust/releases/tag/v0.8.0"
prior_released_version: v0.7.1
prior_released_at: "2026-06-17"
prior_release_tag: v0.7.1
prior_release_commit: b98a72f
current_cycle: v0.1.0-greenfield-spec
current_wave: 27 (FINAL — CLOSED)
stories_delivered: 70  # STORY-INDEX total_stories: 70 (68 merged + STORY-116/117 delivered v0.7.1; STORY-118 delivered v0.8.0)
wave_history_detail: "cycles/phase-3-tdd/wave-history.md (all waves 1-27)"
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
adversary_convergence_counter: 3/3  # Pass 14 CONVERGENCE_REACHED; clean-streak 3/3; ADVERSARY GATE SATISFIED
e8_f2_adversary_convergence_counter: "3/3 SATISFIED — GATE SATISFIED (frozen corpus 4231b6b; Passes 15/16/17 run in parallel; each zero MEDIUM-or-above; 17 passes total)."
e8_f3_adversary_convergence_counter: "3/3 SATISFIED — GATE SATISFIED (frozen corpus bdd531a; Passes V/W/X run in parallel; each zero MEDIUM-or-above; 8 parallel triples / 24 passes total)."
e8_f2_spec_evolution_adversary_convergence_counter: "3/3 SATISFIED — F2 spec-evolution adversarial gate SATISFIED (frozen corpus 60d8392; Round-4 triple A/B/C all CLEAN, zero MEDIUM+). Convergence took 4 rounds: R1 (5 findings), R2 (1 MEDIUM F-A2-01), R3 (1 MEDIUM F-R3A-01), R4 (3/3 CLEAN). Recurring root cause PG-62-F2-BOOKKEEPING-SWEEP-001 (post-fix-burst bookkeeping propagation)."
e8_f3_story_adversary_convergence_counter: "0/3 — round-2 re-streak pending after F3 CRITICAL fix (AC-005/ADR run_analyze scope; STORY-120 input-hash cfa60a9). Gate NOT SATISFIED."
e8_f4_wave_adversary_convergence_counter: "3/3 SATISFIED (passes 1/2/3 clean on develop 5f7cd1b)"
e8_f5_scoped_adversary_convergence_counter: "3/3 SATISFIED (passes 1/2/3 clean on develop 5f7cd1b)"
e8_f6_hardening_status: "HARDENED — no new VP; regression 1641/1641; VP-012 proptest pass; Kani/fuzz unaffected; collapse-delta mutation 100% kill; audit/deny clean"
e8_f7_convergence_status: "CONVERGED — 5-dim MET on develop 5f7cd1b; holistic adversarial: impl ship-ready 3/3"
e8_f4_holdout_status: "PASS — mean 1.00 / critical-min 1.00 (11 CLI scenarios)"
arp_f4_wave_adversary_convergence_counter: 3/3 CONVERGED (re-streak on bcb1bd6) — F4 wave-level adversarial gate SATISFIED
arp_f5_scoped_adversary_convergence_counter: "3/3 CONVERGED — F5 scoped-adversarial gate SATISFIED (2026-06-16, develop 079013d)"
convergence_trajectory: "P1-P14 greenfield GATE-SATISFIED; MITRE-222 3-pass CONVERGED. Detail: cycles/v0.1.0-greenfield-spec/convergence-trajectory.md"
arp_f2_adversary_convergence_counter: 3/3 CONVERGED  # P31/P32/P33 consecutive CLEAN; F2 strict-whole-corpus gate SATISFIED
arp_f3_adversary_convergence_counter: 3/3 CONVERGED  # Passes 36/37/38 consecutive CLEAN; F3 gate SATISFIED (STORY-111..115 E-16)
e17_f3_adversary_convergence_counter: 3/3 SATISFIED  # genuine — 3 verified fresh-context CLEAN passes on dd34205; each zero MEDIUM+
e17_f4_wave_adversary_convergence_counter: "3/3 SATISFIED — GATE SATISFIED (cb2bf06; passes a2c9149c/afec0575/a6c3e1ba)"
e17_f5_scoped_adversary_convergence_counter: "3/3 SATISFIED (cb2bf06; a4b70a59/a97d26e3/ac72bce2) — E-17 F5 gate SATISFIED 2026-06-17"
e17_f7_convergence_status: "CONVERGED — 5-dim MET; F7 holistic adversarial 3/3 (cb2bf06); release-ready v0.7.1"
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT  # post-F7-consistency-remediation; F1-F4 ALL REMEDIATED 2026-06-16
input_drift_check: "F7-followup-dispositions burst (2026-06-16): STORY-071/100/111/112/113/114/115 ALL MATCH. Non-ARP/non-BC-2.10.007 STALE pre-existing; does NOT block release."
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.8.0 RELEASED 2026-06-17 — terminal finding-collapse (E-18, issue #259, STORY-118). F1-F7 CONVERGED AND CLOSED. PR #265 (release/0.8.0 → main 73034da); tag v0.8.0; release.yml run 27732692087 SUCCESS — 4 binaries PUBLISHED. GitHub Release https://github.com/Zious11/wirerust/releases/tag/v0.8.0 (isDraft=false). develop HEAD bec13ba. Pipeline STEADY_STATE/IDLE — await new feature or steady-state task. STORY-119 (grouped-mode collapse) DEFERRED to future cycle.**

## Maintenance Run (maint-2026-06-17)

**Status:** COMPLETE. **Verdict:** NON-BLOCKING — 48 findings, zero CRITICAL, zero CVEs.
**Report:** `.factory/maintenance/sweep-report-2026-06-17.md`
**Delivered:** PR #261 (closes #254) + PR #262 (docs ARP/DNP3/Modbus + ADR-0005/0006/0007). develop HEAD c03a38b after maintenance PRs.
**Deferred (5 items):** TD-MAINT-PC001-DNP3-STREAMTRAIT, TD-MAINT-PC006-MODBUS-NAME-CASING, TD-MAINT-PC003-DNP3-DROPPED-COUNTER, TD-MAINT-PERF-ARP-HOTPATH, TD-MAINT-RISK-REGISTRY-BACKFILL.

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
| Feature #8 DNP3 — F2..F7 + Release v0.6.0 | **RELEASED** 2026-06-12 | SS-15 24 BCs; 268 total; F5 3/3; F6 VP-023 LOCKED; F7 5-dim CONVERGED; PR #234→main 3e29891; tag v0.6.0; 4 binaries. Detail: cycles/feature-8-dnp3-v0.5.0/ |
| Maintenance: Dependabot sweep (post-v0.6.0) | **COMPLETE** 2026-06-12 | 5 PRs merged (#203/#204/#207/#235/#206), 2 closed; develop 31d1231 |
| Feature: ARP analyzer (E-16) — F1..F7 + Release v0.7.0 | **RELEASED** 2026-06-16 | STORY-111..115; 15 SS-16 BCs; F5 3/3; F6 5/5 Kani VP-024 LOCKED; F7 CONVERGED; PR #256→main dd8e142; tag v0.7.0; 4 binaries. Detail: cycles/feature-arp-v0.7.0/ |
| E-17: ARP QinQ/MACsec (issue #253) — F1..F7 + Release v0.7.1 | **RELEASED** 2026-06-17 | STORY-116/117; test-only delta; F7 holistic 3/3; PR #258→develop; PR #260→main b98a72f; tag v0.7.1; 4 binaries. Detail: cycles/feature-arp-v0.7.0/ §E-17 sections |
| Reactive fix: issue #220 Modbus burst-window display | **CLOSED** 2026-06-17 | PR #263 5ed8077; BC-2.14.017 v2.6; spec 8d5446d |
| Maintenance maint-2026-06-17 | **COMPLETE** 2026-06-17 | 2 PRs delivered (#261/#262); 5 items deferred; develop c03a38b |
| Feature E-18 / #259 finding-collapse — F1..F7 + Release v0.8.0 | **RELEASED** 2026-06-17 | STORY-118; 9 new BCs SS-11=29; total 288 BCs; F5 3/3; F6 mutation 100%; F7 5-dim CONVERGED; PR #264→develop 5f7cd1b; PR #265→main 73034da; tag v0.8.0; 4 binaries; run 27732692087. STORY-119 DEFERRED. Per-phase detail: cycles/feature-collapse-v0.8.0/phase-progress-archive.md |
| Feature E-8 / #62 TerminalReporter enum-modes — F1..F2 COMPLETE | **F1..F2 COMPLETE — F2 adversarial gate SATISFIED 3/3 (60d8392)** | F2 fix-burst 2026-06-18: 12 unique SS-11 BCs re-anchored; BC-INDEX v1.42; ADR-0003 v0.9.0 subsection; PRD-delta (12 BCs + run_summary site); HS-081 9df8300 MATCH; STORY-077/078/118 FROZEN (D-088). 4 rounds to convergence (R1: 5 findings; R2: 1 MEDIUM; R3: 1 MEDIUM; R4: 3/3 CLEAN). |
| E-8 / #62 F3 story decomposition — IN PROGRESS | **F3 STORY-120 created; round-1 + round-2 FIXED; convergence pending (0/3)** | STORY-120 created (enum migration carrier, 28 construction sites, wave 48, E-8, 16 ACs, 3 pts, depends_on []). STORY-119 re-pointed to depends_on [STORY-120]. Round-1: 1 CRITICAL + 2 HIGH + 2 MEDIUM + 4 MINOR ALL FIXED (D-092). Round-2: CRITICAL scope error in AC-005/ADR-0003 (out-of-scope *mitre/no_collapse at run_analyze; adj: in-scope bools show_mitre_grouping/collapse_findings); AC-001 doc-comments ADR-aligned; collapse_findings_from_flag UNCHANGED; dep-graph prose 71→72 (D-093). STORY-120 input-hash cfa60a9. Convergence re-streak pending. |

## Session Resume Checkpoint (2026-06-18 — FEATURE MODE E-8 / #62; F3 IN PROGRESS — STORY-120 round-2 fix-burst COMPLETE; convergence re-streak 0/3 pending)

**Previous checkpoint (2026-06-18 — F3 round-1 fix-burst COMPLETE) archived to:
`.factory/cycles/feature-arp-v0.7.0/session-checkpoints.md`**

### A. EXACT PIPELINE POSITION

- **Project:** wirerust. **Mode:** FEATURE_MODE — E-8 / issue #62 TerminalReporter enum-of-modes refactor.
- **Phase:** F3 incremental story decomposition IN PROGRESS. STORY-120 round-2 fix-burst applied (D-093). F3 convergence re-streak: 0/3 — gate NOT SATISFIED.
- **Latest release:** v0.8.0 — finding-collapse (E-18, issue #259, STORY-118). Tag v0.8.0 on main 73034da.
- **develop HEAD:** bec13ba == origin/develop (ADR-0003 round-2 fix pending PR on develop tree — see Step 3).
- **main HEAD:** 73034da (chore: release v0.8.0).
- **factory-artifacts HEAD:** run `git -C .factory log -1 --format='%H'`
- **Active worktrees:** EXACTLY 2 — main repo (develop at /Users/zious/Documents/GITHUB/wirerust), `.factory/` (factory-artifacts).
- **Open PRs:** NONE (ADR-0003 fix is uncommitted on develop tree; needs PR before F3 adversarial).

### B. RESUME PROCEDURE (COLD-RESUME — follow verbatim)

**Step 1 (BLOCKING):** Run `vsdd-factory:factory-worktree-health` before any other action.

**Step 2 — Verify SHAs:**
- `git rev-parse --short HEAD` → expect `bec13ba`
- `git rev-parse --short main` → expect `73034da`
- `git tag -l v0.8.0` → must exist
- `git -C .factory rev-parse --short HEAD` → must match factory-artifacts HEAD above
- `gh pr list --state open` → expect empty (or 1 if ADR-0003 PR was opened)

**Step 3 — WHAT IS COMPLETE (do NOT redo):**
- v0.8.0 FULLY RELEASED (D-087). E-18 #259 CLOSED. STORY-119 DEFERRED (now re-pointed to depends_on [STORY-120]).
- F1 delta-analysis for E-8 / #62 COMPLETE. Artifact: `.factory/phase-f1-delta-analysis/issue-62-terminal-reporter-enum-modes-delta-analysis.md`.
- F2 spec-evolution COMPLETE (D-088–D-091): 12 SS-11 BCs re-anchored; ADR-0003 amended; HS-081 MATCH; STORY-077/078/118 FROZEN. Gate SATISFIED 3/3 (60d8392).
- F3 round-1: STORY-120 created (28 construction sites, wave 48, 16 ACs, 3 pts, depends_on []). ALL FIXED (D-092): census 35→28, Grouped/FlatExpanded split, AC-005 citation.
- F3 round-2: CRITICAL AC-005/ADR-0003 scope error fixed (D-093): prescribed *mitre/no_collapse vars are out of scope at run_analyze; adjudicated in-scope bools show_mitre_grouping/collapse_findings used instead; collapse_findings_from_flag UNCHANGED; AC-001 doc-comments ADR-aligned; dep-graph acyclicity prose 71→72. STORY-120 input-hash cfa60a9. ADR-0003 fix is on develop tree (docs/adr/0003-reporting-pipeline-layering.md — uncommitted/needs PR). ARCH-INDEX.md updated on factory-artifacts.

**Step 4 — NEXT ACTIONS:**
1. Commit and PR the ADR-0003 fix on develop (docs/adr/0003-reporting-pipeline-layering.md) — this is a doc-only change.
2. Run F3 adversary re-streak: dispatch 3 fresh-context passes on STORY-120 post-round-2-fix corpus. Gate requires 3 consecutive CLEAN (zero MEDIUM+). Counter currently 0/3.

### C. KEY ARTIFACT POINTERS

- F1 delta-analysis: `.factory/phase-f1-delta-analysis/issue-62-terminal-reporter-enum-modes-delta-analysis.md`
- F2 PRD-delta: `.factory/phase-f2-spec-evolution/issue-62-prd-delta.md`
- SS-11 BCs: `.factory/specs/behavioral-contracts/ss-11/BC-2.11.*.md`
- STORY-120: `.factory/stories/STORY-120.md`
- E-18 #259 cycle detail: `cycles/feature-collapse-v0.8.0/phase-progress-archive.md`
- Tech-debt register: `.factory/tech-debt-register.md`
- GitHub Release v0.8.0: https://github.com/Zious/wirerust/releases/tag/v0.8.0

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
| D-067 | IcsImpact Display adjudication — canonical Display = "Impact" (spec correct; BC-2.10.002 PC3/PC4, PRD §85/823, cap-10, spec-changelog unanimous). src/mitre.rs:91 "Impact (ICS)" is DEVIANT (introduced F-F5-002 as "No BC change" tactical test fix). " (ICS)" suffix does NOT break merge-by-name report bucketing. Fix folded into STORY-114. **SUPERSEDED BY D-069.** | 2026-06-13 |
| D-068 | Benign gratuitous ARP emits mitre_techniques: [] (LOW/Anomaly severity); T0830 + T1557.002 apply ONLY when GARP conflicts with binding table (BC-2.16.014). Corrected latent over-tagging defect in BC-2.16.003 (→v1.7) and ADR-008 (→v2.0). | 2026-06-14 |
| D-069 | IcsImpact Display canonical = "Impact (ICS)" (distinct from Enterprise "Impact" TA0040). SUPERSEDES D-067. Research-backed: MITRE TA0040 (Enterprise Impact) vs TA0105 (ICS Impact) are distinct tactic families. src/mitre.rs:91 "Impact (ICS)" is CORRECT. STORY-114 D-067 revert obligations REVOKED. | 2026-06-14 |
| D-070 | Feature ARP F3 human gate PASSED (2026-06-14) — STORY-111..115 (E-16, 47 pts) accepted; F3 strict whole-corpus adversarial convergence SATISFIED (3/3, Passes 36/37/38; 38 passes total). F4 delta-implementation AUTHORIZED: linear chain STORY-111→112→113→114→115; release target v0.7.0. | 2026-06-14 |
| D-071 | F4-surfaced STORY-111 decomposition fix (2026-06-14) — strict-TDD stub-architect Red-Gate (BC-5.38.005 self-check) caught that STORY-111 ACs asserted STORY-112's extract_arp_frame end-to-end ARP-decode behavior, unsatisfiable within STORY-111's scaffolding scope. Re-scoped STORY-111→v1.1 + STORY-112 AC-012→v1.1. Both stories input-hash MATCH (d5bda72/268f53f). **F4 scoped post-fix adversarial re-review:** 7 findings ALL remediated (1 HIGH BC-2.16.015 lax_ip_triple unreachable! mis-anchor + 2 MED + 4 LOW). | 2026-06-14 |
| D-072 | F4-surfaced ARP decode design inconsistency (2026-06-14) — arp-architecture-delta §2.2 inconsistent BLOCK 1 vs authoritative BLOCK 2. Architect ruled SYMMETRIC design authoritative: decode_packet routes ARP in both strict + lax arms. Reconciled: arp-architecture-delta v1.16, ADR-008 Decision 3 v2.1, BC-2.02.009 v1.7, BC-2.16.015 v1.3, STORY-111 v1.4, STORY-112 v1.3. SUPERSEDES BC-2.16.015 v1.2 fix. | 2026-06-14 |
| D-073 | STORY-111 DELIVERED — PR #236 MERGED to develop (merge commit cced898). etherparse 0.20 migration + DecodedFrame{Ip,Arp} enum + ArpFrame struct + decode_packet→Result<DecodedFrame> + VP-008 fuzz-harness return-type update. 53 test suites green; clippy/fmt clean; Step-4.5 adversarial 3/3. pr-reviewer APPROVE. Wave 40 complete. | 2026-06-14 |
| D-074 | Reject `--arp-storm-rate 0` and `--arp-spoof-threshold 0` at CLI with fail-fast `anyhow::bail!`. BC-2.16.008 v1.7→v1.8 (EC-006), BC-2.16.012 v1.2→v1.3, BC-2.16.013 v1.2→v1.3. PR #242 merged develop fee71ee. Surfaced by F4 wave-level adversarial Pass 1 finding F-ARP-F4P1-001 (MEDIUM). | 2026-06-15 |
| D-075 | HIGH-confidence D1 ARP-spoof finding carries `Verdict::Likely` (was `Verdict::Possible`). Holdout-caught defect. BC-2.16.004 L45/L74/L118. PR #243 (merge 4ee7a9d). | 2026-06-15 |
| D-076 | D-075 regression-test doc-comments corrected from present-tense RED prose to past-tense regression-guard framing. Recurrence of DF-GREEN-DOC-TENSE-SWEEP sub-rule d / PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE — codified policy text alone did not prevent recurrence. PR #244 (merge 52437f8). | 2026-06-15 |
| D-077 | CRITICAL: `extract_arp_frame` now rejects non-Ethernet hw type (`hw_addr_type != ETHERNET`) and non-IPv4 proto type (`proto_addr_type != IPV4`). BC-2.16.001 PC2/PC3, BC-2.16.009 PC3a/3b/EC-001/EC-002. Half-implemented D11 security boundary caught by F4 3-pass adversary re-streak. Security review PASS (CWE-20, panic-free). PR #245 (merge 6abcd8f). F4 adversary counter RESET to 0/3; re-streak restarted. | 2026-06-15 |
| D-078 | F5 O-A finding adjudicated FIX: lax `None` arm now bounds-checked-peeks raw 8-byte ARP fixed header. Closes CWE-693 D11-evasion. BC-2.16.009 v1.4→v1.6, BC-2.16.015 v1.3→v1.5, STORY-111 v1.4→v1.6, STORY-112 v1.4→v1.6. PR #247 (merge 92c1561). | 2026-06-15/16 |
| D-078b | Completion — sibling lax `Some(LaxNetSlice::Arp)` arm also routes extract_arp_frame returning None to D11. Structurally unreachable via integration. PR #248 (merge 2d2fadf). F5 streak reset to 0/3. | 2026-06-16 |
| D-F1 | F5 Pass 1/3 (re-run on 2d2fadf) found F-1 MEDIUM: D-078 lax None-arm peek hard-coded Ethernet2 offset 14, ignoring `lax.link_exts` — VLAN-tagged ARP false-positive D11. Fix: `arp_offset = 14 + lax.link_exts.iter().map(|ext| ext.header_len()).sum()`. BC-2.16.015 v1.5→v1.6, BC-2.16.009 v1.6→v1.7. PR #249 (merge 079013d). F5 counter reset to 0/3. | 2026-06-16 |
| D-080 | Issue #220 CLOSED — reactive fix. Modbus write-burst summary cosmetic: `elapsed_secs` → `window_secs`. BC-2.14.017 v2.6 (PC1 + EC-011). PR #263 5ed8077. 9/9 CI green. Spec commit 8d5446d. | 2026-06-17 |
| D-081 | Steady-state issue triage 2026-06-17 — 3 issues validated GENUINE via DF-VALIDATION-001: #259 finding-collapse (HIGH — terminal reporter aggregation); #255 JSON snake_case (improvement); #252 VP-024 proof_file_hash (governance). GitHub label `protocol:arp` created. | 2026-06-17 |
| D-082 | STORY-118 (#259 finding-collapse, flat mode) IMPLEMENTED + per-story adversarial gate (BC-5.39.001) SATISFIED 3/3 (2026-06-17). Final clean triple G/H/I ALL CLEAN (zero MEDIUM+). Convergence: T1→c349859; T2→f240900; T3→b847915; T4 ALL CLEAN. 37 story_118 tests ALL PASS. | 2026-06-17 |
| D-083 | Feature #259 F4 COMPLETE (2026-06-17) — wave-47 adversarial 3/3 GATE SATISFIED + holdout PASS (mean 1.00, 11 CLI-producible P0 scenarios). HS-W47-005/010 (reporter-boundary synthetics) out-of-black-box-scope. Pipeline advances F4 → F5. | 2026-06-17 |
| D-084 | Feature #259 F5 CONVERGED 3/3 — scoped-adversarial gate SATISFIED (2026-06-17, develop 5f7cd1b). Three independent fresh-context CLEAN passes; each zero MEDIUM+. BC-set completeness sweep PASS (all 9 BCs). Panic-safety + injection-closure + display-layer-only + determinism confirmed. Pipeline advances F5 → F6. | 2026-06-17 |
| D-085 | Feature #259 F6 HARDENED (2026-06-17, develop 5f7cd1b). Full regression 1641/1641 PASS. VP-012 escape proptest (4 × 1000 cases) PASS. Kani/fuzz UNAFFECTED. Mutation 100% kill (6/6). cargo audit: 0 new advisories. cargo deny OK. clippy/fmt CLEAN. Pipeline advances F6 → F7. | 2026-06-17 |
| D-086 | Feature #259 F7 CONVERGED (2026-06-17, develop 5f7cd1b). 5-dim ALL MET. F7 holistic adversarial gate: 3 fresh-context passes all agree implementation is ship-ready. Release-packaging gaps (PG-F7-006) to apply on release/0.8.0 branch. Pipeline advances F7 → RELEASE PREP v0.8.0. | 2026-06-17 |
| D-087 | wirerust v0.8.0 RELEASED 2026-06-17 — E-18 #259 finding-collapse F1-F7 CONVERGED AND CLOSED. Release PR #265 (release/0.8.0 → main 73034da); annotated tag v0.8.0; run 27732692087 SUCCESS; 4 binaries; GitHub Release isDraft=false. Cargo.toml 0.8.0 + CHANGELOG [0.8.0] on develop. develop merge-back bec13ba. STORY-119 (grouped-mode collapse) DEFERRED. All process-gap dispositions tracked — cycle CLOSED. Pipeline = STEADY_STATE/IDLE. | 2026-06-17 |
| D-088 | Issue #62 F2 spec-evolution: completed/shipped stories (STORY-077/078/118) are FROZEN as immutable as-built records — NOT retroactively re-anchored to the v0.9.0 FindingsRender enum (human-adjudicated 2026-06-18). The enum vocabulary lives only in living specs (BCs/ADR/holdouts) and the future STORY-120. Two F2 adversary passes (5 then 6 findings) drove: full SS-11 BC sweep (13 BCs re-anchored, not the initially-claimed 8), PRD-delta scope correction (8→12 BCs), HS-081 input-hash recompute, and revert of the over-eager completed-story body sweep. | 2026-06-18 |
| D-089 | Issue #62 F2 round-2 bookkeeping reconciliation (2026-06-18): Round-2 adversary Pass A found F-A2-01 MEDIUM — spec-changelog Scope line "Architecture unchanged." contradicted ADR-0003 amendment. Exhaustive bookkeeping self-audit applied: (1) spec-changelog Scope corrected + bookkeeping-correction note + ADR-0003/ARCH-INDEX rows added to Version Summary table; (2) PRD-delta Verification section extended with run_summary → FlatCollapsed (inert) construction site; (3) BC count clarified: 12 unique SS-11 BCs (.025 bumped twice; "13" in prior entries was a miscount). Round-3 re-streak pending. | 2026-06-18 |
| D-090 | Issue #62 F2 round-3 bookkeeping fix (2026-06-18): Round-3 adversary Pass A found F-R3A-01 MEDIUM — PRD-delta BCs-Touched table .029 cell showed v1.3 (stale) while the actual BC file was at v1.4. Fixed: PRD-delta .029 row updated to show v1.3→v1.4 transition. Orchestrator then mechanically cross-checked all 12 BCs-Touched table "After" version cells against their BC files — 12/12 MATCH confirmed. 4th consecutive round where Pass A found exactly one stale-bookkeeping cell while Passes B/C were CLEAN; orchestrator broke the loop by doing a full-table mechanical sweep. PG-62-F2-BOOKKEEPING-SWEEP-001 reinforced (4th data point). Round-4 re-streak pending on fully reconciled corpus. | 2026-06-18 |
| D-091 | Issue #62 F2 spec-evolution CONVERGED — F2 adversarial gate SATISFIED 3/3 on frozen corpus 60d8392 (Round-4 A/B/C all CLEAN). 12 SS-11 BCs re-anchored to FindingsRender{Grouped,FlatCollapsed,FlatExpanded}; ADR-0003 amended (Render-Mode Enum subsection + Binding Rule 5, v0.9.0 semver); ARCH-INDEX updated; HS-081 input-hash 9df8300; STORY-077/078/118 frozen as-built (D-088). Two LOW cosmetic observations deferred to next doc-sweep: BC-2.11.026 PC-6 line-anchor off-by-one (DRIFT-62-BC026-PC6-LINEANCHOR-001); BC-2.11.028 EC-count change-prose undercount (DRIFT-62-BC028-ECCOUNT-PROSE-001). Pipeline advances F2 → F3. | 2026-06-18 |
| D-092 | Issue #62 F3 — STORY-120 created as sole enum-migration carrier (28 construction sites: 2 src/main.rs + 7 reporter_terminal_tests + 17 reporter_tests + 1 dnp3_f5 + 1 bc_2_09_100 param-helper; wave 48; depends_on []). STORY-119 re-pointed to depend on STORY-120. F3 round-1 adversarial+consistency review caught: CRITICAL Grouped/FlatExpanded mis-split (wrong-but-compiling variant — only cargo test detects); HIGH census error (35→28, double-counted fn-signatures); HIGH dead test citation; 2 MEDIUM AC quality gaps; 4 MINOR issues. ALL FIXED in fix-burst. F3 adversarial convergence pending (0/3). | 2026-06-18 |
| D-093 | Issue #62 F3 round-1 triple caught CRITICAL in STORY-120 AC-005 (+ originating ADR-0003 migration map): prescribed `render: if *mitre ... else if !no_collapse ...` at run_analyze construction site, but those vars are out of scope there — only main() owns them. Adjudicated option (a): keep run_analyze signature UNCHANGED, build enum from in-scope bool params show_mitre_grouping/collapse_findings; collapse_findings_from_flag UNCHANGED. ADR-0003 migration map + STORY-120 AC-005/Task5 both corrected. AC-001 doc-comment ADR byte-match fixed. dep-graph acyclicity prose 71→72. STORY-120 input-hash ca8e753→cfa60a9. ADR-0003 on develop tree (uncommitted). Process-gap: AC code blocks MUST reference only variables in scope at the cited file:line (PG-62-F3-AC-SCOPE). F3 convergence re-streak pending. | 2026-06-18 |

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
| DRIFT-ENGINE-RELEASECONFIG-STALE-001 | release-config.yaml human-prose fields refreshed this burst; engine template follow-up (version_sources) DEFERRED | PARTIALLY RESOLVED |
| DRIFT-BC-INPUTHASH-TBD-001 | all 24 SS-15 BC files carry input-hash:TBD; by-design; non-blocking | BY-DESIGN LOW |
| PG-F7-001..007 | Feature-cycle process-gap policies (see each PG entry); detail: cycles/feature-arp-v0.7.0/lessons.md | DEFERRED — next feature cycle |
| PG-ARP-F2-003..009 | ARP F2 process-gap policies (sibling-sweep, anchor-drift, YAML dup-key, etc.) | DEFERRED — policy codification |
| PG-ARP-F4-* | ARP F4 process-gap items (banner-sweep, preclear-propagation, guard-wording, demo-leak, pr-mgr shortstop, inverted-TDD, proxy-counter, stale-skeleton, green-doc-tense, type-branch-narrowing, multipass-value, docsweep-overreach) | Various — see tech-debt-register; detail: cycles/feature-arp-v0.7.0/lessons.md |
| PG-ARP-FIXBURST-CONSUMER-SWEEP | VP-024 v1.8 harness rename didn't sweep 11 consuming artifacts; reverted via PR #246. | OPEN — policy codification |
| PG-ARP-FIX-MECHANISM-FIRST | F5 O-A: spec written from incorrect mechanism hypothesis; LOW-fix cascaded into 3 PRs + MEDIUM regression. | OPEN — process-gap codification |
| PG-CONSISTENCY-AUDIT-CONSUMER-SWEEP | F6 lock + Sub-D surrogate rename did NOT propagate to all consuming artifacts. | OPEN — policy strengthening |
| DRIFT-VP024-BTREEMAP-PROSE-001 | VP-024 Feasibility ~line 582 still reads 'BTreeMap'; shipped substrate is fixed-capacity array surrogate v2.3. | DEFERRED LOW |
| DRIFT-E17-VERSIONLABEL-LAG-001 | verification-coverage-matrix ~48/137 and e17 test-file doc-comments cite initial-burst BC versions. | DEFERRED LOW |
| PG-E17-AGENT-SCOPE-CREEP-001 | Two sub-agents made unrequested out-of-scope edits mid-adversarial-pass, breaking frozen-corpus premise. | ENGINE-NOTE MEDIUM |
| PG-E17-ADVERSARY-HANG-001 | Three adversarial-pass sub-agents hung silently (~60 min each) across E-17 cycle. | ENGINE-NOTE HIGH |
| DRIFT-E16-EPICS-SUMMARY-GAP-001 | epics.md "Estimated Story Count Summary" table omits Epic E-16 (5 stories). | DEFERRED LOW |
| DRIFT-E16-BC-BACKLINK-GAP-001 | BC-2.16.009/BC-2.16.015 Traceability "Stories:" lists omit STORY-114/STORY-115. | DEFERRED LOW |
| DRIFT-EPICS-REGISTRY-STRUCTURAL-001 | epics.md pre-existing structural debt: "12 Subsystems" heading omits SS-14/SS-15/SS-16; E-13/E-14/E-16 epic body sections missing. | DEFERRED LOW |
| PG-E17-STATEMGR-FABRICATED-VERDICT-001 | State-manager burst recorded CLEAN adversary-pass verdict with no real adversary result. ENGINE-NOTE. | ENGINE-NOTE HIGH |
| DRIFT-ADR0007-D2-PROSE-001 | ADR-0007 Decision 2 prose contains arithmetic-walk thinking artifact (pr-reviewer nit on PR #262). | LOW — doc-cleanup; target: next doc sweep |
| DRIFT-BC2-14-017-CR003-001 | CR-003 (PR #263): companion test for elapsed_secs==1 distinct-second burst absent. Non-blocking. | LOW — polish; target: next Modbus test pass |
| PG-MAINT-WORKTREE-PATHGUARD-001 | Maint-2026-06-17 fix agent's edits landed in main repo instead of assigned worktree. ENGINE-NOTE LOW. | ENGINE-NOTE DEFERRED |
| PG-259-F2-ADVERSARY-CHURN | F2 #259 took 17 passes; root cause was BC over-specification of internal call structure. Parallel-triple approach evidence. | RECORDED — codification deferred |
| PG-259-F3-SIBLING-SWEEP-CROSS-ARTIFACT | F3 #259: fixing one artifact (story OR holdout) missed sibling (BC + HS) co-sweep. | RECORDED — codification pending |
| PG-259-F3-BC-CONTENT-INPUTHASH | BC-2.11.025 content change required recomputing STORY-118 input-hash (policy exists but not enforced at burst level). | RECORDED — enforcement gap |
| PG-259-F3-HOLDOUT-PRODUCIBILITY | F3 #259: blind-CLI holdouts must pin request headers to suppress incidental co-emissions. | RECORDED — holdout authoring policy gap |
| PG-259-F4-PERSTORY-CHURN | F4 #259 per-story adversarial found minor defects each triple (test-quality + hygiene; impl logic correct from first GREEN). | RECORDED — codification pending |
| DRIFT-HS-W47-JSON-CMD-001 | wave-47-holdout HS-W47-007/008 command examples use `--json <pcap>` / `--csv <pcap>`; correct invocation needs `-- <pcap>`. Fix in future holdout-maintenance pass. | DEFERRED LOW |
| DRIFT-RUNANALYZE-REASSEMBLYCONFIG-MUTANTS-001 | F6 mutation run surfaced 2 pre-existing SURVIVING mutants in run_analyze ReassemblyConfig field init. Out of #259 scope. | LOW — deferred |
| FU-JSON-CASING | Align serde enum casing to snake_case (ECS/OCSF best-practice). Touches BC-2.09.004/BC-2.11.001/ADR-0003. | FILED #255 — post-release |
| FU-BC-2.10.007-MARKER | BC-2.10.007 PLANNED marker. | FIXED — BC-2.10.007 v1.8 de-PLANNED 23→25; factory-artifacts commit 147aa63 (2026-06-16) |
| DNPXX-SOURCE-RENAME-001 | `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` placeholder-style name shipped v0.6.0; candidate rename DNPXX_→DNP3_. | DEFERRED LOW |
| DF-SIBLING-SWEEP-CROSS-SS-001 | F-cycle BC-invariant corrections sharing routing semantics across subsystems MUST sweep cross-subsystem sibling BCs. | DEFERRED — policy codification |
| PG-ARP-F4-REDTEST-DOC-TENSE-RECURRENCE | PG-ARP-F4-REDTEST-DOC-TENSE recurred in D-075 despite codification. Agent-prompt/hook strengthening needed. | OPEN — agent-prompt/hook strengthening needed |
| PG-ARP-F4-TYPE-BRANCH-NARROWING | impl + unit tests + Kani consistently omitted hw/proto type-reject branch (D-077), self-consistent omission invisible to 4 adversary passes + holdout. | OPEN — DF-BC-COMPLETENESS-SWEEP policy extension |
| DRIFT-62-FROZEN-STORY-INPUTHASH-001 | STORY-077/078/118 input-hashes are STALE after #62 BC re-anchoring (BCs referenced by those stories now carry FindingsRender enum vocabulary). | ACCEPTED — frozen as-built completed-story records; not re-anchored per D-088; does NOT block Phase-4 (#62 cycle delivers STORY-120 only) |
| PG-62-F2-BOOKKEEPING-SWEEP-001 | F2 re-anchor burst migrated normative behavioral text but initially skipped bookkeeping surfaces (story version-tables, PRD-delta BC table, input-hashes) and under-counted the BC set (claimed 8, actual 13). Root cause: F1/F2 accepted the BC list without a mechanical SS-11 grep sweep. | RECORDED — codification follow-up: F2 dispatch template should mandate `grep -rn 'show_mitre_grouping\|collapse_findings'` reconcile step |
| DRIFT-62-BC026-PC6-LINEANCHOR-001 | BC-2.11.026 PC-6 cites terminal.rs:209-221; authoritative is :209-222 per BC-2.11.018. Pre-existing; untouched by #62. | DEFERRED — next doc-sweep |
| DRIFT-62-BC028-ECCOUNT-PROSE-001 | Changelog/PRD-delta describe BC-2.11.028 EC changes as "EC-001..005" but EC-010 was also enum-updated. Artifact is correct; change-prose undercounts. | DEFERRED — next doc-sweep |
| PG-62-F3-AC-SCOPE | F3 round-1 adversary caught CRITICAL: AC-005 code block prescribed vars (*mitre, no_collapse) that are out of scope at the cited construction site (run_analyze). Root cause: story-writer did not verify variable scope at each cited file:line. Policy candidate: AC code blocks MUST reference only variables provably in scope at the cited file:line anchor. Ties to PG-62-F2-BOOKKEEPING-SWEEP-001 family (multi-phase fresh-context audits surface latent spec defects). | OPEN — policy codification |

## Deferred Next-Work Backlog

**1. PCAP-CORPUS-001:** R2/B2/Drive-SA — TABLED, human decision pending.

**2. Roadmap (post-DNP3):** #3 C2 beaconing | #4 CSV+SQLite reporters | #6 rayon parallel (relates to O-07).

**3. STORY-119 (grouped-mode finding-collapse):** natural next feature candidate post-v0.8.0.

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
| DF-GREEN-DOC-TENSE-SWEEP (v1) | HIGH (CODIFIED policies.yaml 2026-06-15; sub-rule REDTEST-DOC-TENSE added STORY-115) |

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Artifact pointers: Phase 0 synthesis `.factory/semport/wirerust/wirerust-pass-8-deep-synthesis.md`; wave history `cycles/phase-3-tdd/convergence-trajectory.md`; phase 4 holdout `cycles/v0.1.0-greenfield-spec/phase-4-holdout-eval-summary.md`; F6 hardening `cycles/feature-8-dnp3-v0.5.0/F6-hardening/`.
- Issues: #104/#102 CLOSED (PRs #194/#195), #100 RELEASED v0.2.0, #101 OPEN-DEBT, #103 DEFERRED. Dependabot sweep 2026-06-12 cleared all v0.6.0-era PRs. All actions SHA-pinned (actions/checkout at df4cb1c # v6.0.3); pin gate enforced.
- Picked up issue #253 (QinQ/MACsec ARP decoder fixtures); DF-VALIDATION-001 = GENUINE/OPEN on 480f8ae; validation at research/issue-253-qinq-macsec-validation.md; delivery scope = QinQ fixtures (assert) + MACsec probe-only.
