---
pipeline: FEATURE_MODE
phase: F2
phase_status: "FEATURE MODE issue #62 — F3 CONVERGED (3/3). STORY-120 (TerminalReporter FindingsRender enum migration, 28 construction sites, wave 48, E-8) implementation-ready. AWAITING F3 HUMAN GATE before F4 delta-implementation."
active_feature: "E-8 / #62 TerminalReporter enum-of-modes refactor — F1..F3 COMPLETE; STORY-120 implementation-ready (28 sites, wave 48); STORY-119 depends_on [STORY-120]; F3 adversarial gate SATISFIED 3/3 (f034ca2); AWAITING F3 HUMAN GATE → F4; release target v0.9.0"
feature_arp_status: "v0.7.0 RELEASED 2026-06-16 — ARP Security Analyzer (E-16, issue #9); PR #256 dd8e142; tag v0.7.0; 4 binaries (aarch64-apple-darwin, x86_64-apple-darwin, x86_64-pc-windows-msvc, x86_64-unknown-linux-gnu)"
feature_8_status: "v0.6.0 RELEASED 2026-06-12 — DNP3 TCP analyzer; F7 5-dim CONVERGED; tag v0.6.0 + 4 binaries"
product: wirerust
mode: brownfield
timestamp: 2026-06-18T23:30:00Z
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
e8_f3_story_adversary_convergence_counter: "3/3 SATISFIED — F3 story-decomposition adversarial gate SATISFIED (frozen corpus f034ca2; Round-10 triple A/B/C all CLEAN, zero MEDIUM+; all 3 confirmed STORY-120 implementer-ready + byte-identical). Convergence took 10 rounds — most-churned phase; churn was documentation/bookkeeping in consuming artifacts (esp. F1 delta-analysis sub-counts re-entering via input-hash), NOT implementation-readiness (established early)."
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
| E-8 / #62 F3 story decomposition — CONVERGED | **F3 CONVERGED — adversarial gate SATISFIED 3/3 (f034ca2); awaiting human gate** | STORY-120 (28 sites, wave 48, 16+1 ACs, depends_on [], input-hash 3d76a93) implementation-ready. 10 fix rounds (D-092..D-102). Churn: documentation/bookkeeping in consuming artifacts (F1 sub-counts via input-hash), NOT implementation-readiness. AWAITING F3 HUMAN GATE → F4. |

## Session Resume Checkpoint (2026-06-18 — COLD-RESUME — FEATURE MODE E-8 / #62; F1 ✅ / F2 ✅ CONVERGED 3/3 / F3 ✅ CONVERGED 3/3 — AWAITING F3 HUMAN GATE; NEXT = F4)

**Previous checkpoint (2026-06-18 — F3 round-9 fix-burst COMPLETE; round-10 re-streak 0/3 pending) archived to:
`.factory/cycles/feature-collapse-v0.8.0/session-checkpoints.md`**

### A. EXACT PIPELINE POSITION

- **Project:** wirerust. **Mode:** FEATURE_MODE — E-8 / issue #62 TerminalReporter FindingsRender enum-of-modes refactor. Release target v0.9.0.
- **Phase:** **F1 ✅ / F2 ✅ CONVERGED 3/3 / F3 ✅ CONVERGED 3/3 (10 rounds; corpus f034ca2). AWAITING F3 HUMAN GATE. Next = F4 delta-implementation (TDD).** No F4 work started.
- **Refactor summary:** Replace TerminalReporter's 3 render bools (`show_mitre_grouping`, `collapse_findings`, and the logical third) with `FindingsRender { Grouped, FlatCollapsed, FlatExpanded }` enum (keep `use_color`, `show_hosts_breakdown` as orthogonal bools). 28 construction sites. Byte-identical output required.
- **Latest release:** v0.8.0 — finding-collapse (E-18, issue #259, STORY-118). Tag v0.8.0 on main 73034da.

### B. EXACT SHAs / WORKTREE STATE (verified 2026-06-18)

- **develop HEAD:** `bec13ba` == origin/develop (`chore: merge main (v0.8.0) back into develop`).
- **main HEAD:** `73034da` (`chore: release v0.8.0`). Tag `v0.8.0` annotated.
- **factory-artifacts HEAD:** run `git -C /Users/zious/Documents/GITHUB/wirerust/.factory log -1 --format='%h %s'` — this burst advances it; record the new HEAD post-commit.
- **STORY-120 input-hash:** `3d76a93` (MATCH verified).
- **UNCOMMITTED on develop working tree: `docs/adr/0003-reporting-pipeline-layering.md` (M)** — F2+F3 spec changes (Render-Mode Enum subsection, Binding Rule 5, F3 in-scope-param migration-map correction). Backed up at `.factory/cycles/feature-collapse-v0.8.0/issue-62-adr-0003-f2f3.patch`. This change rides into develop via the STORY-120 F4 PR.
- **Active worktrees:** EXACTLY 2 — main repo (develop at `/Users/zious/Documents/GITHUB/wirerust`), `.factory/` (factory-artifacts).
- **Open PRs:** NONE.

### C. RESUME PROCEDURE (COLD-RESUME — follow verbatim in order)

**Step 1 (BLOCKING):** Run `/vsdd-factory:factory-worktree-health` before any other action.

**Step 2 — Verify SHAs:**
```
git -C /Users/zious/Documents/GITHUB/wirerust rev-parse --short HEAD      # expect bec13ba
git -C /Users/zious/Documents/GITHUB/wirerust rev-parse --short main       # expect 73034da
git -C /Users/zious/Documents/GITHUB/wirerust tag -l v0.8.0                # must exist
git -C /Users/zious/Documents/GITHUB/wirerust/.factory log -1 --format='%h %s'  # factory-artifacts HEAD
gh pr list --state open                                                     # expect empty
```

**Step 3 — Verify uncommitted ADR change is present:**
```
git -C /Users/zious/Documents/GITHUB/wirerust status --short
```
Expected output contains: ` M docs/adr/0003-reporting-pipeline-layering.md`

If ABSENT: re-apply the patch:
```
git -C /Users/zious/Documents/GITHUB/wirerust apply .factory/cycles/feature-collapse-v0.8.0/issue-62-adr-0003-f2f3.patch
```

### D. WHAT IS COMPLETE — DO NOT REDO

- **F1 delta-analysis:** COMPLETE (full numeric audit; 28 construction sites: 2 src/main.rs + 7 reporter_terminal_tests + 17 reporter_tests + 1 dnp3_f5 + 1 bc_2_09_100). Artifact: `.factory/phase-f1-delta-analysis/issue-62-terminal-reporter-enum-modes-delta-analysis.md`.
- **F2 spec-evolution:** COMPLETE (D-088–D-091). 12 unique SS-11 BCs re-anchored to FindingsRender enum (authoritative versions: BC-2.11.010 v1.9, 013 v1.12, 014 v1.7, 015 v1.8, 016 v1.7, 017 v1.14, 019 v1.8, 025 v1.9, 026 v1.10, 027 v1.5, 028 v1.6, 029 v1.5). BC-INDEX v1.42 (current). ADR-0003 amended (uncommitted on develop — see §B). HS-081 recomputed 9df8300 MATCH. STORY-077/078/118 FROZEN as-built (D-088). F2 adversarial gate SATISFIED 3/3 (frozen corpus 60d8392).
- **F3 story decomposition:** COMPLETE (D-092–D-102; 10 fix rounds). STORY-120 created (E-8, wave 48, 3 pts, 17 ACs, depends_on [], input-hash 3d76a93). STORY-119 re-pointed depends_on [STORY-120]. dep-graph v2.1 (intra 74 / cross 21 / total 95; no cycle). STORY-INDEX: 73 stories / 48 waves / 476 total-points; E-8 has 6 stories / 32 points. F3 adversarial gate SATISFIED 3/3 (frozen corpus f034ca2; Round-10 A/B/C all CLEAN, zero MEDIUM+).
- **Decisions D-088..D-102** cover this cycle (archived in decisions log above).

### E. NEXT ACTIONS (in order — do NOT skip steps)

1. **BLOCKING on resume:** run `/vsdd-factory:factory-worktree-health`; verify SHAs per §C; confirm uncommitted ADR change is present (re-apply patch if absent).
2. **F3 HUMAN GATE is pending** — 3 questions for the human: (a) approve to proceed to F4 TDD? (b) open a self-improvement follow-up story for the F1-input-doc churn process-gap (D-099/D-100/D-101) or defer? (c) confirm v0.9.0 release target. **Do NOT start F4 until the human approves.**
3. **On human approval → F4 delta-implementation (TDD):**
   - Dispatch test-writer: stub `pub enum FindingsRender` + failing tests for AC-001 derives + AC-003 dispatch-exhaustive in a `mod story_120` block.
   - Dispatch implementer: TDD the 28-site migration per AC-007/Task-7 + Task-7b comment sweep; full existing suite as byte-identical regression gate.
   - Per-story delivery: demo, PR, review, merge. ADR-0003 patch rides in via the F4 PR.
4. **After F4 delivery:** F5 scoped-adversarial → F6 formal hardening → F7 convergence → release v0.9.0.

### F. KEY ARTIFACT POINTERS

- Story: `.factory/stories/STORY-120.md`
- Depends: `.factory/stories/STORY-119.md` (depends_on [STORY-120])
- F1 delta-analysis: `.factory/phase-f1-delta-analysis/issue-62-terminal-reporter-enum-modes-delta-analysis.md`
- F2 PRD-delta: `.factory/phase-f2-spec-evolution/issue-62-prd-delta.md`
- Research: `.factory/research/issue-62-enum-modes-design-validation.md`
- ADR: `docs/adr/0003-reporting-pipeline-layering.md` (uncommitted; patch: `.factory/cycles/feature-collapse-v0.8.0/issue-62-adr-0003-f2f3.patch`)
- BCs: `.factory/specs/behavioral-contracts/ss-11/BC-2.11.0{10,13,14,15,16,17,19,25,26,27,28,29}.md`
- Decisions detail: D-088..D-102 in Decisions Log above
- ADR patch re-apply: `git -C /Users/zious/Documents/GITHUB/wirerust apply .factory/cycles/feature-collapse-v0.8.0/issue-62-adr-0003-f2f3.patch`

## Decisions Log

D-001..D-054 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (D-047..D-054 in Feature #8 / v0.5.0 section).
D-055..D-091 archived: `cycles/feature-collapse-v0.8.0/decisions-archive.md` (Feature #8 DNP3 / ARP / E-17 / v0.8.0 / E-8 F2 cycle decisions; D-080..D-091 compacted 2026-06-18 F3-R9 burst).

| ID | Decision | Date |
|----|----------|------|
| D-092 | Issue #62 F3 — STORY-120 created as sole enum-migration carrier (28 construction sites: 2 src/main.rs + 7 reporter_terminal_tests + 17 reporter_tests + 1 dnp3_f5 + 1 bc_2_09_100 param-helper; wave 48; depends_on []). STORY-119 re-pointed to depend on STORY-120. F3 round-1 adversarial+consistency review caught: CRITICAL Grouped/FlatExpanded mis-split (wrong-but-compiling variant — only cargo test detects); HIGH census error (35→28, double-counted fn-signatures); HIGH dead test citation; 2 MEDIUM AC quality gaps; 4 MINOR issues. ALL FIXED in fix-burst. F3 adversarial convergence pending (0/3). | 2026-06-18 |
| D-093 | Issue #62 F3 round-1 triple caught CRITICAL in STORY-120 AC-005 (+ originating ADR-0003 migration map): prescribed `render: if *mitre ... else if !no_collapse ...` at run_analyze construction site, but those vars are out of scope there — only main() owns them. Adjudicated option (a): keep run_analyze signature UNCHANGED, build enum from in-scope bool params show_mitre_grouping/collapse_findings; collapse_findings_from_flag UNCHANGED. ADR-0003 migration map + STORY-120 AC-005/Task5 both corrected. AC-001 doc-comment ADR byte-match fixed. dep-graph acyclicity prose 71→72. STORY-120 input-hash ca8e753→cfa60a9. ADR-0003 on develop tree (uncommitted). Process-gap: AC code blocks MUST reference only variables in scope at the cited file:line (PG-62-F3-AC-SCOPE). F3 convergence re-streak pending. | 2026-06-18 |
| D-094 | Issue #62 F3 round-2 triple (each pass 1 distinct MEDIUM, CRITICAL/census/wiring confirmed clean): (1) BC-2.11.014/015/016/017 missing explicit AC trace clauses — added to AC-003/AC-014 with "(traces to BC-2.11.014/015/016/017)" clauses + BC↔Body cross-check reconciled; (2) dep-graph mis-bucketed STORY-120 cross-epic edges as intra-E-18 — corrected to intra_epic_edges 74/cross_epic_edges 21 (total_edges 95 unchanged), v2.0 changelog + summary table + subheadings corrected; (3) STORY-120 lacked a test-comment-sweep task — added Task 7b (grep sweep for stale field-referencing comments) + AC-017 (no stale comments, DF-GREEN-DOC-TENSE/SIBLING-SWEEP guard, cargo check/test cannot catch). STORY-120 now 17 ACs, input-hash cfa60a9 MATCH (inputs unchanged). Convergence re-streak pending. | 2026-06-18 |
| D-095 | Issue #62 F3 round-3 triple found CRITICAL: round-2 AC-trace completeness fix added BC-2.11.015/016 trace descriptions semantically INVERTED — BC-015 mislabeled "colorization" (actual: Uncategorized bucket); BC-016 mislabeled "uncategorized" (actual: em-dash expansion); plus 12 BC body-table titles were scrambled/truncated copies. Root cause: story-writer wrote descriptions from memory rather than reading BC postconditions. Fixed round-4: orchestrator extracted verbatim canonical H1 titles + actual PC-1 text and handed paste-ready to story-writer. Also fixed: dep-graph version-stamp lag (1.9→2.0); AC-017/Task-7b comment-sweep falsifiable (full 24-target census + 13-entry explicit EXEMPT allow-list); colorization attribution removed. STORY-120 input-hash cfa60a9 MATCH. Process-gap PG-62-F3-AC-DESC-FROM-SOURCE recorded. | 2026-06-18 |
| D-096 | Issue #62 F3 round-4 triple: story BODY converged (Pass A + B CLEAN); Pass C found 2 MEDIUM in the anchored BCs — BC-2.11.028 still prescribed the out-of-scope `*mitre`/`!no_collapse` wiring (the ROOT that propagated to ADR + AC-005, both already fixed) and BC-2.11.019/025/026 anchored FINDINGS dispatch at stale 149-162 (actual 185-207, ~38-line drift, pre-existing). Fixed: BC-2.11.028 v1.6 (in-scope params `show_mitre_grouping`/`collapse_findings` at PC3/Inv1/Inv6/Architecture-Anchor); BC-2.11.019 v1.8 / BC-2.11.025 v1.9 / BC-2.11.026 v1.10 (re-anchor 149-162→185-207). STORY-120 input-hash 2012512. Demonstrates F3 source cross-check catching latent F2 BC defects. Round-5 re-streak pending. | 2026-06-18 |
| D-097 | Issue #62 F3 round-5 bookkeeping-sync: round-4 BC version bumps (019 v1.8/025 v1.9/026 v1.10/028 v1.6) created propagation drift into STORY-120 body BC-table version cells + frontmatter `# BC status:` comment (HIGH) and STORY-119 forward-ref BC table (MED). AC-017 EXEMPT list mis-classified lines 3345/3358 (construction-site comments that Task 7 rewrites — they belong in Forward-Facing Sweep Targets, not EXEMPT). Fixed via orchestrator-supplied authoritative version set + exhaustive grep reconciliation across both stories (zero mismatches). input-hash 2012512 unchanged (documentation-only edits). Recurring root: BC version bump must sweep consuming-story body version-stamps in addition to file:line anchors — reinforces PG-62-F2-BOOKKEEPING-SWEEP / PG-62-F3-AC-DESC-FROM-SOURCE family. Round-6 re-streak pending. | 2026-06-18 |
| D-098 | Issue #62 F3 round-6 triple (2 CLEAN + 1 MEDIUM): BC-2.11.029 Architecture-Anchor still carried the out-of-scope `*mitre`/`!no_collapse` wiring expression — identical to the BC-2.11.028 defect fixed in round-4, but the round-4 sibling-sweep (DF-SIBLING-SWEEP-001) covered only the dispatch-anchor (149-162) sweep, not the wiring-expression sweep. Fixed: BC-2.11.029 v1.5 (in-scope params `show_mitre_grouping`/`collapse_findings`); exhaustive grep across all 12 BCs confirms zero remaining defect wiring expressions. STORY-120 .029 stamp synced, input-hash 1cd1be8. Reinforces DF-SIBLING-SWEEP-001: a fix must sweep ALL siblings for the SAME defect class, not just the named instance. Round-7 re-streak pending. | 2026-06-18 |
| D-099 | Issue #62 F3 round-7 triple (1 CLEAN + 2 MEDIUM): Pass A found F1 delta-analysis census still cited "35 construction sites" (ground-truth 28; 9 locations corrected: census table reporter_terminal 12→7/dnp3 2→1/bc_2_09 2→1, total line, §6 intro, §7, summary table, §8 test-count, §9 OQ-5/OQ-6, §10 top-risks). Pass B found dep-graph BC-to-Stories matrix carrying stale version stamps for BCs bumped in rounds 4–6. Fixed: dep-graph matrix stamps synced (BC-019 v1.8/025 v1.9/026 v1.10/028 v1.6/029 v1.5); dep-graph v2.0→v2.1; STORY-120 input-hash 1cd1be8→776490b. Exhaustive grep confirms only frozen/historical artifacts retain old stamps (STORY-118 per D-088; prd.md/epics/STATE/changelog narration). F3 churn root: each BC bump creates distinct consuming surfaces that must be swept together — codification candidate: single post-BC-bump sweep checklist (BC files, BC-INDEX, spec-changelog, consuming-story body+frontmatter+version-table, dep-graph matrix, F1/F2 phase docs). Round-8 re-streak pending. | 2026-06-18 |
| D-100 | Issue #62 F3 round-8 triple (all 3 passes confirmed implementer-success + byte-identical output; residuals documentation-only): F1 OQ-3 retained the 10th '35' occurrence the round-7 9-location sweep missed (doubly wrong — '35' + 'all in test files' when 2 are in src/main.rs); anchor line 187-205 corrected. AC-017's field-name grep was blind to two paraphrased provenance comments (dnp3:1074, bc_2_09:694) — AC-017 now runs a dual grep (field-name + paraphrase pattern); Forward-Facing Sweep Targets expanded to 26 total. STORY-120 input-hash 776490b→6e4d628. F3 has run 8 fix rounds; the churn is the documented PG-259-F2-ADVERSARY-CHURN / consuming-surface-sweep pattern — codification candidate D-099 stands. NOTE for orchestrator: if round-9 surfaces only further documentation-hygiene residuals (no implementer-blocking defect), escalate to human with recommend-accept. | 2026-06-18 |
| D-101 | Issue #62 F3 round-9 triple: ALL THREE passes converged on a single identical finding (F1 §6 migration table reporter_tests Grouped undercount 4 vs ground-truth 6) and all three confirmed the STORY-120 body is correct, self-sufficient, and implementer-ready (byte-identical output) — strong convergence signal. Root cause: the F1 delta-analysis accumulated stale sub-counts (35→28 headline fixed across rounds 7-8 but embedded sub-counts in §6/§2/§10 lagged). Resolved by an EXHAUSTIVE F1 numeric audit reconciling every count/line-list against grep ground-truth (3 fixes: §6 reporter_tests Grouped "4 sites"→"6 sites (1001,1036,1071,1106,1155,1192)" + FlatExpanded "most"→"11 sites"; §2 line-132 "9 BCs need updates"→8 (BC-2.11.018 is no-change); §10 line-513 "9 BCs"→8). STORY-120 input-hash 3d76a93. F3 has run 9 fix rounds — the most-churned phase of this cycle; the F1-delta-analysis as a STORY-120 input meant every F1 edit re-triggered input-hash + fresh re-derivation. Codification candidate (extends D-099/D-100): F1/F2 phase analysis docs that are declared story inputs MUST pass a full numeric self-audit vs grep ground-truth at authoring time, since they re-enter the convergence loop via input-hash. | 2026-06-18 |
| D-102 | Issue #62 F3 story-decomposition CONVERGED — adversarial gate SATISFIED 3/3 on frozen corpus f034ca2 (Round-10 A/B/C all CLEAN). STORY-120 (16+1 ACs, 28 construction sites, depends_on [], wave 48, input-hash 3d76a93) verified implementation-ready: in-scope-param wiring (option a), behavior-preserving byte-identical, complete BC traceability, AC-017 dual-grep comment-sweep falsifiable. F3 required 10 fix rounds — the cycle's most-churned phase. Root cause (codified D-099/D-100/D-101): the F1 delta-analysis is a STORY-120 input, so every consuming-artifact fix re-triggered input-hash recompute + fresh adversarial re-derivation, and stale sub-counts/version-stamps surfaced serially one consuming surface per round (story body, frontmatter comment, dep-graph matrix, dep-graph version, F1 headline count, F1 sub-counts, BC-029 sibling wiring, AC-017 paraphrase blind-spot). Resolved decisively by exhaustive orchestrator-supplied authoritative-value sweeps. Pipeline awaiting F3 HUMAN GATE → F4. | 2026-06-18 |

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
| PG-62-F3-AC-DESC-FROM-SOURCE | F3 round-3 adversary caught CRITICAL: round-2 AC-trace descriptions were written from memory and were semantically INVERTED for BC-2.11.015/016. Root cause: story-writer paraphrased BC postconditions rather than reading the actual BC file and copying the canonical PC-1 text verbatim. Policy candidate: AC trace descriptions for BC citations MUST be copied verbatim from the cited BC's actual postcondition text, never paraphrased. Companion to PG-62-F3-AC-SCOPE. | OPEN — policy codification pending (MEDIUM) |

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
