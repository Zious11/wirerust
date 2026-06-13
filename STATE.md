---
pipeline: FEATURE_MODE_ARP_ANALYZER
phase: feature-F2-strict-whole-corpus-convergence
phase_status: "0/3 — Pass 15 (Claude) REMEDIATED; resume = Pass 16 (via agy, additional quota)"
active_feature: "arp-analyzer"
feature_arp_status: "F1 Delta Analysis PASSED (human-gated 2026-06-12) — DecodedFrame integration, ADR-008 planned, F2→F7 authorized; release target v0.7.0"
feature_8_status: "v0.6.0 RELEASED 2026-06-12 — DNP3 TCP analyzer; F7 5-dim CONVERGED; tag v0.6.0 + 4 binaries"
product: wirerust
mode: brownfield
timestamp: 2026-06-13T00:00:00Z
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
develop_head: 31d1231
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
arp_f2_adversary_convergence_counter: 0/3  # STRICT WHOLE-CORPUS mode — zero findings any severity across entire spec corpus required; P15 NOT_CLEAN→REMEDIATED (8 findings incl holdout-layer field-rename C-01..03 + inv-01 YAML regression C-04 + VP-024 scope A-01 + 3 more); remediation does not advance counter; Pass 16 via agy next
arp_f2_convergence_trajectory: "15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5→~18→~8→~22(P14: 2C/5H NEW corpus-debt; trend broke; ARP delta clean 6th pass)→P15(8 findings: holdout-layer field-rename + regression; REMEDIATED) — 0/3 STRICT WHOLE-CORPUS; 15 passes; P14 REMEDIATED; P15 REMEDIATED (8 findings: holdout-scenarios field-rename 16 files C-01/02/03 + inv-01 YAML regression + VP-024 scope + 3 more; consistency CONSISTENT 7 dims); next=Pass 16 via agy (additional quota). Detail: phase-f5-adversarial/arp-f2-convergence-trajectory.md"
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT
input_drift_check: "MATCH=62 STALE=0 ERROR=1 (STORY-091 known); STORY-106 d0ef956 / STORY-109 cf0bb94 re-stamped"
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.6.0 RELEASED (DNP3 TCP analyzer, issue #8). Feature: ARP security analyzer + etherparse 0.16→0.20 migration (F1 PASSED 2026-06-12, D-066); release target v0.7.0. F2 spec evolution IN PROGRESS — adversarial convergence 0/3 STRICT WHOLE-CORPUS mode (human-elected 2026-06-13; 15 passes completed; Pass 15 REMEDIATED — 8 findings: holdout-scenarios field-rename sweep 16 files [C-01/02/03] + inv-01 YAML regression [C-04] + VP-024 scope [A-01] + 3 more; consistency CONSISTENT all 7 dims; next = Pass 16 via agy/Gemini CLI, additional quota provided).**

**Summary:** 63 stories (48 greenfield + 4 F-cycle + 11 F3-new), 400 pts. 268 BCs (244 pre-F2 + 24 SS-15), 23 VPs (22+VP-023 ALL LOCKED), 1496 tests green, holdout 0.967. develop HEAD 31d1231; main HEAD 3e29891 (v0.6.0). ARP feature: F1 approved — est. 18-24 new BCs (SS-16), 1 revised BC, VP-024, ADR-008, 5-6 stories (E-16), 3-5 holdout scenarios. MITRE T0830 (primary) + T1557.002 (secondary).

Post-release sweep 2026-06-12: 5 dep bumps merged (#203/#204/#207/#235/#206), #202/#205 closed; develop 31d1231; etherparse 0.20 folded into ARP feature cycle (IN-PROGRESS).

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
| Feature: ARP analyzer — F2 Spec Evolution | **IN PROGRESS** — adversarial convergence 0/3 STRICT WHOLE-CORPUS (human-elected 2026-06-13); Pass 15 REMEDIATED; next = Pass 16 via agy (Gemini CLI, cross-family, additional quota) | 4-slice method; 15 passes; ARP delta SETTLED (clean 6 consecutive); P14 REMEDIATED (22 findings: mitre_techniques field-rename corpus sweep + O-01 closure + architect ×2 + PO ×10); P15 REMEDIATED (8 findings: holdout-scenarios field-rename 16 files C-01/02/03 + inv-01 YAML regression C-04 + VP-024 scope A-01 + 3 more; consistency CONSISTENT 7 dims); trajectory: `phase-f5-adversarial/arp-f2-convergence-trajectory.md` |

## Session Resume Checkpoint (2026-06-13 — F2 STRICT WHOLE-CORPUS CONVERGENCE, Pass 15 REMEDIATED; next = Pass 16 via agy)

**Previous checkpoint (2026-06-13 — Pass 14 REMEDIATED) archived to:
`cycles/feature-arp-v0.7.0/session-checkpoints.md`**

### POSITION

- **Project:** wirerust. Mode: brownfield/feature. Active feature: ARP security analyzer +
  etherparse 0.16→0.20 migration (issue #9). Release target v0.7.0.
- **Pipeline phase:** Feature Mode **F2 (Spec Evolution) — adversarial convergence**, IN PROGRESS.
- **F1 delta analysis:** PASSED (human-gated 2026-06-12, D-066).
- **F2 spec authoring:** COMPLETE (SS-16 15 BCs, ADR-008, VP-024, arp-architecture-delta,
  cap-10/SS-10 catalogue deltas, PRD §2.16/O-04, error-taxonomy ARP rows, HS-INDEX ARP seeds).
- **F2 adversarial convergence:** STRICT WHOLE-CORPUS mode (human-elected 2026-06-13).
  Bar = 3 consecutive passes with ZERO findings of ANY severity, including LOW, across the
  ENTIRE spec corpus — not just the ARP delta. Counter **0/3**.
- **15 adversarial passes + 1 corpus consistency audit run. Pass 15 REMEDIATED.**
  Pass 14 (22 findings): mitre_techniques field-rename corpus sweep + O-01 closure propagation +
  architect ×2 + PO ×10 + consistency audit CONSISTENT (7 dims). ARP delta clean 6th consecutive.
  Pass 15 (8 findings): holdout-scenarios field-rename sweep [16 HS files; C-01/02/03 — sibling
  layer MISSED by Pass-14 sweep] + inv-01 YAML duplicate `version:` key [C-04 regression] +
  VP-024 BC-scope reconciliation [A-01] + BC-2.11.024 Evidence + BC-INDEX pin + interface-definitions.
  Consistency audit CONSISTENT all 7 dimensions (including new holdout-scenarios field-rename
  saturation dimension and inv-01 YAML structure dimension). Two process-gaps noted:
  PG-ARP-F2-003 (holdout-scenarios must be in sweep perimeter) and PG-ARP-F2-004
  (YAML frontmatter version bumps must replace-in-place, not append).

### VERIFIED SHAs (re-verify live on resume — do NOT trust as current-HEAD values)

| Ref | Value at checkpoint | How to re-verify |
|-----|--------------------|--------------------|
| develop HEAD | `31d1231` | `git rev-parse --short HEAD` (on develop) |
| main HEAD | `3e29891` | `git log main -1 --format='%h %s'` |
| tag v0.6.0 | annotated → commit 3e29891 | `git show v0.6.0 --format='%h' -s` |
| factory-artifacts HEAD | re-verify live | `git -C .factory log -1 --format='%h %s'` |
| released_version | v0.6.0 | — |
| Open PRs | none | `gh pr list --state open` |
| Working tree | clean | `git status --short` |

develop == origin/develop at checkpoint. No open PRs. Working tree clean.

### WHAT IS CONVERGED (do NOT re-litigate)

The ARP F2 delta core is CONVERGED — zero ARP-specific defects for 5 consecutive passes
(P9–P13). This includes: SS-16 BCs BC-2.16.001..015, ADR-008, VP-024, arp-architecture-delta,
cap-10/SS-10 catalogue deltas, PRD §2.16/O-04, error-taxonomy ARP rows, HS-INDEX ARP seeds.
Pass 13 Slice B verified all 283 BC H1 titles clean.

Remaining findings each pass are pre-existing corpus debt in non-ARP docs and occasional
remediation-churn residue. Trajectory decaying: corpus-audit 9 → P12 ~18 → P13 ~8.

### HOW TO RESUME (runbook, strict order)

1. Run `vsdd-factory:factory-worktree-health`. **BLOCKING — do not proceed if this fails.**
2. Read `STATE.md` (this file) + `.factory/phase-f5-adversarial/arp-f2-convergence-trajectory.md`
   (full per-pass history + current artifact versions table, post-Pass-14 remediation).
3. Confirm develop==origin/develop, working tree clean, no open PRs.
4. **Next action: whole-corpus Pass 16 via `agy` (Gemini CLI).**
   Run `agy` (/opt/homebrew/bin/agy, -p print mode) as cross-family adversary (Gemini model,
   cognitive diversity per human directive 2026-06-13). Additional individual quota provided by
   human (prior RESOURCE_EXHAUSTED 429 that blocked Pass 15 attempt reset ~5 days; Pass 15 was
   run via Claude adversary per human fallback direction). Monitor for RESOURCE_EXHAUSTED; if
   quota dies, fall back to Claude vsdd-factory:adversary as in Pass 15. Use step-budgeted
   scoped calls (~40 planner steps per slice to avoid agentic cap). Dispatch 4 FRESH-CONTEXT
   slices in parallel, STRICT mode (report EVERY finding of ANY severity), each covering its
   whole-corpus partition:
   - **Slice A** = ALL architecture/ (ARCH-INDEX, module-decomposition, module-criticality,
     tooling-selection, dependency-graph, api-surface, purity-boundary-map,
     arp-architecture-delta, ADR-0001..0008) + ALL verification-properties/ (VP-INDEX,
     verification-architecture, verification-coverage-matrix, all vp-*.md).
     Route findings → architect.
   - **Slice B** = BC-INDEX + ALL ss-01..ss-16 BC bodies (283 total).
     Route findings → product-owner.
   - **Slice C** = ALL domain/ (domain-spec, capabilities/cap-*, entities/ent-*,
     invariants/inv-*) + prd-supplements + HS-INDEX + ss-10 + stories/STORY-INDEX + sample
     stories + mitre-arp research files.
     Route findings → product-owner (architect for VP/mitre.rs facts).
   - **Slice D** = PRD + all indexes (BC-INDEX, ARCH-INDEX, VP-INDEX, HS-INDEX, STORY-INDEX) +
     spec-changelog ledger completeness + cross-doc master counts.
     Route findings → product-owner.
   - Inject policy rubric from `.factory/policies.yaml`.
   - Mark STORY-114 PLANNED forward-declarations NON-BLOCKING (see below).
5. Aggregate verdicts. If all 4 slices CLEAN → that is 1 of 3 consecutive clean; record and
   continue. If any finding → remediate (architect-bucket first, then product-owner,
   sequential to avoid changelog version races; PO owns spec-changelog and records
   architect's exact version bumps), then re-run. Repeat until 3 consecutive all-slice-clean.
6. On 3/3 clean → run `vsdd-factory:consistency-validator` final audit → F2→F3 human gate
   (present structured approval) → F3 story decomposition (STORY-111..115).

### NON-BLOCKING / EXPECTED (do NOT treat as findings)

- **STORY-114 PLANNED forward-declarations:** src/mitre.rs at SEEDED=23/EMITTED=15 (target
  25/17); decode_packet returns Result<ParsedPacket> (target Result<DecodedFrame>); no
  src/analyzer/arp.rs yet; ADR-008 status proposed; VP-024 status draft. These are correct —
  the code is implemented in F4/STORY-114, NOT in F2. PLANNED markers are in
  BC-2.10.005/007/008, cap-10, arp-architecture-delta §5.0.
- **Brownfield IcsImpact display obligation:** src/mitre.rs:91 IcsImpact Display "Impact (ICS)"
  vs spec canonical "Impact" — tracked for STORY-114 F4 adjudication (arp-architecture-delta
  §5.0 brownfield-debt table; ADR-007 drift-note). NON-BLOCKING in F2.

### RECURRING DEFECT CLASSES (sweep proactively before each pass)

- **Stale src/mitre.rs line anchors** — canonical: technique_info @128-182; `_ => return None`
  @179; technique_tactic @192-194; all_tactics_in_report_order slice @100-120; Display impl
  @72-95; ICS arms @89-91; extract_sni @247, match @252-266. Many docs cite pre-refactor lines.
- **Count drift** — canonical: 283 BCs total; 24 VPs (Kani 11/proptest 7/fuzz 1/int-unit 5;
  P0 8/P1 10/test-suff 6); 17 MitreTactic variants (14E+3 ICS incl IcsImpact); SEEDED
  25/EMITTED 17/CAT-ONLY 8 PLANNED (current src 23/15); 24 components C-1..C-24; ARP holdout
  26/24/2; summarize 11 keys.
- **Stale version-pin citations** — doc cites "BC-X vN" lagging file; BC-INDEX inline
  annotation lag; H1↔BC-INDEX title sync.
- **Changelog ledger completeness** — every frontmatter version recorded, no placeholders or
  phantom paths.
- **Index registration completeness** — PRD §2/§7 ↔ BC-INDEX ↔ ARCH-INDEX must match.

### OFF-RAMP OPTION (human may elect)

Narrow the strict gate back to the ARP-F2 delta perimeter (already converged), declare F2
CONVERGED, proceed to F2→F3 gate, and track remaining pre-existing corpus debt as a separate
maintenance sweep. Human chose full whole-corpus on 2026-06-13; this remains available if
they revise.

### KEY ARTIFACT POINTERS

- Trajectory + per-pass detail + current artifact versions table (post-Pass-13):
  `.factory/phase-f5-adversarial/arp-f2-convergence-trajectory.md`
- Corpus audit (systematic debt classes + remediation worklist):
  `.factory/phase-f5-adversarial/corpus-consistency-audit-2026-06-13.md`
  (NOTE: file listed in STATE.md; verify existence on resume — may be embedded in trajectory
  or under a slightly different name)
- F1 delta analysis: `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md`
- F1 MITRE research: `.factory/phase-f1-delta-analysis/mitre-arp-research.md`
- Feature #8 DNP3 lessons (process-gaps): `cycles/feature-8-dnp3-v0.5.0/lessons.md`
- Decisions log: D-066 (F1 gate approval) in STATE.md Decisions Log below.
- Adversary agent is read-only (cannot persist its own reports) — orchestrator persists
  findings; known [process-gap] (PG candidate: adversary report-write or
  orchestrator-persist).
- Archived prior session checkpoint: `cycles/feature-arp-v0.7.0/session-checkpoints.md`

## Decisions Log

D-001..D-046 archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md`.
D-047..D-054 full text archived: `cycles/v0.1.0-greenfield-spec/decisions-archive.md` (Feature #8 / v0.5.0 section).

| ID | Decision | Date |
|----|----------|------|
| D-047 | Feature #8 DNP3 F1 gate APPROVED — full F1-F7, TCP-only, DispatchTarget::Dnp3 (port-20000 Rule 6), SS-15, VP-023, ADR-007. MITRE: T1692.001/T1691.001/T0827/T0814/T0836. | 2026-06-10 |
| D-048 | MITRE v19 revocation defect (T0855→T1692.001, T0856→T1692.002) — fix-first (issue #222); DNP3 paused; corrected MITRE IDs locked. | 2026-06-10 |
| D-049 | MITRE v19 remap CONVERGED — 3-pass adversarial. | 2026-06-10 |
| D-050 | MITRE v19 remap MERGED to develop via PR #223 (33de854); issue #222 CLOSED. | 2026-06-10 |
| D-051 | v0.5.0 RELEASED (gitflow-proper: release/0.5.0 → PR #224 → main c2df1b5; tag v0.5.0; run 27313698900). | 2026-06-10 |
| D-052 | Feature #8 F2 spec evolution CONVERGED — SS-15 22 BCs + ADR-007 + VP-023; 5-pass adversarial. | 2026-06-10 |
| D-053 | Feature #8 F2 gate COMPLETE — 2 must-add BCs (BC-023/024); SS-15 now 24 BCs / 268 total; 3 thresholds CONFIRMED. | 2026-06-10 |
| D-054 | Feature #8 F3 story decomposition CONVERGED — 5 stories STORY-106..110, E-15, 47 pts, waves 35-39, 22 holdout scenarios. | 2026-06-10 |
| D-055 | Feature #8 F3 human gate PASSED — 5 stories accepted; VP placements; strictly-linear chain. F4 TDD authorized. | 2026-06-11 |
| D-056 | STORY-106 DELIVERED — PR #225 d0f3586. VP-023 4/4 Kani SUCCESSFUL. | 2026-06-11 |
| D-057 | STORY-107 DELIVERED — PR #226 ebb4751. Carry-walk gate-before-count; STORY-106 frames wire-valid. | 2026-06-11 |
| D-058 | STORY-108 DELIVERED — PR #227 9c03fde. 5-pass adversarial 3/3 CLEAN. DRIFT-DNP3-DIRECTION-001 recorded. | 2026-06-11 |
| D-059 | STORY-109 DELIVERED — PR #228 34443f9. 13-pass 3/3 CLEAN; MitreTactic::IcsImpact; VP-007 seed. | 2026-06-12 |
| D-060 | STORY-110 DELIVERED — PR #229 ddfa576. Rule 6 + CLI flags + VP-004 oracle. F4 COMPLETE. | 2026-06-12 |
| D-061 | Feature #8 F5 COMPLETE — PR #230 e685664. 4 issues fixed (DIR-bit P0; unexpected-source P0; IcsImpact display; resync). 10-pass 3/3 CLEAN. | 2026-06-12 |
| D-062 | Feature #8 F6 HARDENED — PR #231 a125c69. 9/9 Kani; 89% mut; 3.19M fuzz/0; VP-023 LOCKED v1.5; VP-004 relocked. 4/4 F6 obligations SATISFIED. | 2026-06-12 |
| D-063 | Feature #8 F7 CONVERGED — 5-dim delta convergence after remediation of F-S2-001 (canonical-frame IEEE 1815 provenance: holdout HS-W37-002 + test, PR #232), F-S1-001 (BC-2.15.009 v1.3 initial-delivery-only reconciliation + BC-INDEX/STORY-106 propagation), F-PG-001 (HS-INDEX feature-holdout indexing), F-CC-001 (HS-W36-001 stale carry assertion), F-CC-002 (STORY-106..110 status drift), F-CC-003/004 (README/CHANGELOG DNP3 docs, PR #233). 6 fresh-context adversarial passes; final 3 consecutive CONVERGED. develop f217f27. NEXT = human gate → v0.6.0. | 2026-06-12 |
| D-064 | v0.6.0 RELEASED — gitflow release/0.6.0 → PR #234 → main 3e29891; fixup fb3935c; tag v0.6.0; GitHub Release WITH 4 binaries (release.yml auto-build); develop merge-back 04f8ccb. DNP3 TCP analyzer is the headline feature. | 2026-06-12 |
| D-065 | Dependabot sweep post-v0.6.0 COMPLETE — #203 serde_json/#204 assert_cmd/#207 clap/#206 rayon routine bumps merged; #235 manual SHA-pin actions/checkout 6.0.3 (replacing tag-ref #202); #205 etherparse 0.16→0.20 closed and deferred as migration story (new drift DRIFT-ETHERPARSE-0.20-MIGRATION-001). develop 31d1231. | 2026-06-12 |
| D-066 | Feature ARP analyzer F1 gate APPROVED — full F1-F7, release target v0.7.0. Integration via DecodedFrame{Ip,Arp} enum from decode_packet (new ADR-008); ArpAnalyzer owns bounded IP↔MAC binding table; zero structural impact on existing 5 analyzers/reassembly/dispatcher. etherparse 0.20 migration is sub-delta A (SliceError::Len removed; 2 non-exhaustive NetSlice/LaxNetSlice match breaks; DecodedFrame return-type change). Estimate: 18-24 new BCs (SS-16), 1 revised BC (BC-2.02.009), VP-024, ADR-008, 5-6 stories (E-16), 3-5 holdout scenarios. MITRE: T0830 (ICS AiTM, primary) + T1557.002 (Enterprise ARP Cache Poisoning, secondary) — validated ATT&CK v19.1. Detections approved: ARP spoof/cache-poisoning + gratuitous ARP + ARP storm/rate anomaly + research-agent pass for additional detections. DRIFT-ETHERPARSE-0.20-MIGRATION-001 folded into this cycle (IN-PROGRESS). | 2026-06-12 |

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
