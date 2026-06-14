---
pipeline: FEATURE_MODE_ARP_ANALYZER
phase: feature-F2-strict-whole-corpus-convergence
phase_status: "F2 CONVERGED (3/3 strict-whole-corpus, Pass 33); D-067 adjudicated (IcsImpact Display = 'Impact', code deviant src/mitre.rs:91, fix folded into STORY-114, NO F2 spec change); F2→F3 gate condition SATISFIED; next = F3 ARP story decomposition (STORY-111..115)"
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
arp_f2_adversary_convergence_counter: 3/3 CONVERGED  # Pass 31/32/33 consecutive CLEAN; F2 strict-whole-corpus adversarial gate SATISFIED
arp_f2_convergence_trajectory: "15→20→~8→~15→~6→~4→~4→~7→~4→~6→~5→~18→~8→~22(P14: 2C/5H NEW corpus-debt; trend broke; ARP delta clean 6th pass)→P15(8 findings: holdout-layer field-rename + regression; REMEDIATED)→P16(7: 0C/0H, sibling-sweep misses; REMEDIATED; Slice B CLEAN)→P17(10: holdout MITRE-counts + module-decomposition peer; REMEDIATED; Slice B CLEAN 2nd)→P18(9: ss-05 anchor-drift + indicatif + STORY-INDEX; 0C/3H; REMEDIATED; arp.rs+holdout pre-flush verified clean)→P19(15: corpus-wide anchor-drift; 0C/8H; PARTIAL — ss-07-full+remaining-BC pending)→ batch2: ss-07-full(35 BCs)+ss-04-partial(21 BCs)+ss-11(10 BCs); ss-01/02/08/13 CLEAN; ss-04-remainder+ss-12 to Pass-20 — REMEDIATED → P20(7: anchor-drift flushed, ss-04/ss-12 closed; 0C/1H; Slices A+C CLEAN; REMEDIATED) → P21(5 cosmetic; 0C/0H; A+C CLEAN; REMEDIATED) → P22(5 valid; 0C/0H; cosmetic; version-pin hardened; REMEDIATED) → P23(5; B/C/D CLEAN; Slice-A only; 0C/0H; REMEDIATED) → P24(4: D-01 DNP3-C24 sweep genuine + 3 self-induced; 0C/1H; B+C CLEAN; REMEDIATED) → P25(2; A/B/C CLEAN; changelog-path flush; 0C/0H; REMEDIATED) → P26 CLEAN 1/3 (all 4 slices zero findings; corpus-wide debt flushed P14-25) → P27 reset 1/3→0/3 (HS-008 kill-chain + HS-INDEX pin; holdout-pin-hardened) → P28 CLEAN 1/3 (restart after P27 reset) → P29 reset 1/3→0/3 (DNP3 T1692.001 + PRD FC-0x17 content gaps; REMEDIATED) → P30 (4 HIGH genuine: FlowKey accessor + STORY input-hash dup + ADR-006 FC0x17; REMEDIATED) → P31 CLEAN 1/3 (restart; P30 HIGH fixes held; all 4 slices zero findings) → P32 CLEAN 2/3 (2nd consecutive) → P33 CLEAN 3/3 CONVERGED (F2 strict-whole-corpus gate satisfied after 33 passes). Detail: phase-f5-adversarial/arp-f2-convergence-trajectory.md"
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT
input_drift_check: "MATCH=62 STALE=0 ERROR=1 (STORY-091 known); STORY-106 d0ef956 / STORY-109 cf0bb94 re-stamped"
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.6.0 RELEASED (DNP3 TCP analyzer, issue #8). Feature: ARP security analyzer + etherparse 0.16→0.20 migration (F1 PASSED 2026-06-12, D-066); release target v0.7.0. F2 ARP spec evolution CONVERGED — 3/3 strict-whole-corpus adversarial gate SATISFIED (Pass 33 CLEAN; 33 passes total; P31/P32/P33 consecutive CLEAN). D-067 adjudicated: IcsImpact Display canonical = "Impact"; src/mitre.rs:91 "Impact (ICS)" is deviant; fix folded into STORY-114; NO F2 spec change — F2 3/3 convergence preserved. F2→F3 gate condition SATISFIED. Next = F3 ARP story decomposition (STORY-111..115).**

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
| Feature: ARP analyzer — F2 Spec Evolution | **CONVERGED 3/3** (Pass 33, 2026-06-13); 33 passes total; P31/P32/P33 consecutive CLEAN; F2 strict-whole-corpus adversarial gate SATISFIED | 4-slice method; ARP delta SETTLED P9+; corpus-wide debt flushed P14-25; P26/P28/P31/P32/P33 CLEAN; P27/P29/P30 reset cycles surfaced+fixed genuine defects; trajectory: `phase-f5-adversarial/arp-f2-convergence-trajectory.md` |

## Session Resume Checkpoint (2026-06-13 — F3 ARP STORY DECOMPOSITION; cold-resume hardened)

**Previous checkpoint (2026-06-13 — F2 CONVERGED 3/3; D-067 adjudicated) archived to:
`cycles/feature-arp-v0.7.0/session-checkpoints.md`**

### A. EXACT PIPELINE POSITION

- **Project:** wirerust. Mode: FEATURE MODE. Active feature: ARP security analyzer +
  etherparse 0.16→0.20 migration. GitHub issue #9. Release target: **v0.7.0**.
- **F1 Delta Analysis:** PASSED (human-gated 2026-06-12, D-066).
- **F2 Spec Evolution:** CONVERGED — STRICT WHOLE-CORPUS adversarial loop, 3/3 consecutive
  clean passes. Final clean pass: Pass 33. Factory HEAD before D-067 decision-record: `6c36b10`.
  Current factory HEAD: `4fb17e6` (D-067 decision-record only; F2 corpus unchanged).
- **D-067 adjudicated 2026-06-13:** IcsImpact Display canonical = "Impact" (spec correct);
  src/mitre.rs:91 "Impact (ICS)" deviant; severity LOW; fix folded into STORY-114.
  F2 NO SPEC CHANGE — F2 3/3 convergence preserved. Gate condition SATISFIED.
- **F2→F3 human gate:** APPROVED by user, conditioned on IcsImpact adjudication.
  Adjudication COMPLETE (D-067). Gate condition SATISFIED.
- **NEXT ACTION:** Launch F3 — story-writer creates STORY-111..115 per
  `.factory/specs/architecture/arp-architecture-delta.md` §6, plus wave-40..44 holdout skeleton.

### B. F3 WORK DEFINITION

**Decomposition source:** `.factory/specs/architecture/arp-architecture-delta.md` §6.
Target: 5 stories (STORY-111..115), epic E-16, strict linear chain. Est. 18-24 ARP BCs (SS-16).

**Per-story carry-forward obligations (embed into story acceptance criteria):**

- **STORY-111:** etherparse 0.16→0.20 upgrade (sub-delta A); VP-008 fuzz harness return-type
  update: `decode_packet` → `DecodedFrame` (SliceError::Len removed; 2 non-exhaustive
  NetSlice/LaxNetSlice match breaks).
- **STORY-114:** (a) D-067 code fix — `src/mitre.rs:91`: change `"Impact (ICS)"` → `"Impact"`;
  (b) `.factory/holdout-scenarios/HS-008-*.md:75`: change `"Impact (ICS)"` → `"Impact"`;
  (c) Three D-067 test obligations: Display unit test `assert_eq!(format!("{}", MitreTactic::IcsImpact), "Impact")`;
  two-bucket enum-level report test confirming Impact vs IcsImpact bucketed distinctly;
  HS-008 alignment verified by test. (d) VP-007 5-part atomic obligation.

**Additional F3 scope items:**
- VP-024 kani::cover obligation (to be placed before F6 formal hardening).
- `src/mitre.rs` forward-decl targets: SEEDED 23→25, EMITTED 15→17 (current src ships 23/15;
  targets 25/17 are ARP feature additions).
- `decode_packet` currently returns `Result<ParsedPacket>` (target: `Result<DecodedFrame>`);
  no `src/analyzer/arp.rs` yet; ADR-008 status proposed; VP-024 status draft.
  These are PLANNED forward-declarations, NOT inconsistencies — do not flag as defects.

### C. GOVERNANCE AND CONVERGENCE PARAMETERS (for F3 adversarial loop)

- **Adversarial method:** STRICT WHOLE-CORPUS, 4 fresh-context slices:
  - Slice A = architecture + VPs
  - Slice B = all 283 BC bodies
  - Slice C = domain / holdout / MITRE / stories
  - Slice D = PRD + indexes + changelog ledger
- **Bar:** ZERO findings of ANY severity (including LOW) across ALL 4 slices.
  3 CONSECUTIVE fully-clean passes required for convergence.
- **Policy DF-ADVERSARY-METHODOLOGY-001:** adversary dispatches use ABSOLUTE paths; NO `cd`.
- **Adversary tooling:** agy (Gemini/Antigravity) CLI is UNUSABLE headless (40-step agentic
  cap; --conversation stall; content-paste hang) AND quota-exhausted (429, ~5-day reset).
  Use CLAUDE adversary (Agent tool, `vsdd-factory:adversary`) for F3 passes.
- **Canonical corpus facts (feed to each adversary dispatch):**
  - BCs: 283 total (244 pre-F2 + 24 SS-15 + 15 SS-16 ARP)
  - VPs: 24 total (22 pre-F2 + VP-023 DNP3 + VP-024 ARP draft)
  - MitreTactic variants: 17 (14 Enterprise + 3 ICS: IcsLateralMovement, IcsImpact, IcsInitialAccess)
  - Components: 24 total (C-1..C-24; C-22 Modbus SHIPPED, C-23 ARP PLANNED, C-24 DNP3 SHIPPED)
  - `Finding.mitre_techniques`: `Vec<String>` + 3 Option fields (`source_ip`, `timestamp`, `direction`)
  - O-01: CLOSED
  - SEEDED 25 / EMITTED 17 / CAT-ONLY 8 (PLANNED targets; current src 23/15)

### D. DEFERRED AND CYCLE-CLOSE ITEMS (must not be lost)

- **Process-gap codification backlog:** PG-ARP-F2-003..009 — deferred to F7/cycle-close.
  DRIFT-PRD-V120-MBAPFRAMER-001 (PRD v1.20 "C-23 was MbapFramer" wrong historical rationale) —
  fix in F3 cycle or maintenance sweep; LOW cosmetic; does NOT block F3.
- **D-067 obligations:** F3-OBL-STORY114-001/002/003 recorded in Drift Items table below.
  These are the authoritative carry-forward; the story-writer MUST reference them.

### E. RESUME COMMAND

To resume after cold context clear:

1. `git -C .factory log -1 --format='%h %s'` — confirm factory HEAD (expect `4fb17e6` or newer).
2. `git rev-parse --short HEAD` on develop — confirm `31d1231` or newer; `git status` clean.
3. `vsdd-factory:factory-worktree-health` — BLOCKING; do not proceed if fails.
4. Dispatch `vsdd-factory:story-writer` to create STORY-111..115 (split into create + integrate
   sub-bursts, since >8 artifacts expected including holdout skeleton for waves 40..44).
5. Run state-manager LAST in the burst (after story-writer completes).

**DO NOT re-run F2 adversarial passes. F2 is CONVERGED. The gate is SATISFIED.**

### F. KEY ARTIFACT POINTERS

- ARP architecture delta (F3 decomposition source): `.factory/specs/architecture/arp-architecture-delta.md`
- F2 convergence trajectory (33 passes): `.factory/phase-f5-adversarial/arp-f2-convergence-trajectory.md`
- F1 delta analysis: `.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md`
- F1 MITRE research: `.factory/phase-f1-delta-analysis/mitre-arp-research.md`
- Corpus consistency audit: `.factory/phase-f5-adversarial/corpus-consistency-audit-2026-06-13.md`
- Decisions log: D-066 (F1 gate), D-067 (IcsImpact) in Decisions Log below.
- Archived prior checkpoints: `cycles/feature-arp-v0.7.0/session-checkpoints.md`
- DNP3 feature lessons (process-gap reference): `cycles/feature-8-dnp3-v0.5.0/lessons.md`

### G. VERIFIED SHAs (re-verify live on resume — treat as snapshot, not current-HEAD)

| Ref | Value at checkpoint | Re-verify command |
|-----|--------------------|--------------------|
| develop HEAD | `31d1231` | `git rev-parse --short HEAD` (on develop) |
| main HEAD | `3e29891` | `git log main -1 --format='%h %s'` |
| tag v0.6.0 | annotated → commit `3e29891` | `git show v0.6.0 --format='%h' -s` |
| factory-artifacts HEAD | `4fb17e6` | `git -C .factory log -1 --format='%h %s'` |
| released_version | v0.6.0 | — |
| open PRs | none | `gh pr list --state open` |
| working tree | clean | `git status --short` |

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
| D-067 | IcsImpact Display adjudication — canonical Display = "Impact" (spec correct; BC-2.10.002 PC3/PC4, PRD §85/823, cap-10, spec-changelog unanimous). src/mitre.rs:91 "Impact (ICS)" is DEVIANT (introduced F-F5-002 as "No BC change" tactical test fix). " (ICS)" suffix does NOT break merge-by-name report bucketing (terminal.rs render_findings_grouped keys on MitreTactic enum variant, not Display string); severity LOW (terminal section-header label only). F2 SPEC CHANGE: NONE — F2 3/3 strict-whole-corpus convergence preserved unaffected. Fix folded into STORY-114 (obligations: 1-line mitre.rs:91 fix; HS-008:75 "Impact (ICS)"→"Impact"; Display unit test; two-bucket enum-level report test). F2→F3 gate condition SATISFIED. | 2026-06-13 |

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
| DRIFT-PRD-V120-MBAPFRAMER-001 | PRD v1.20 delta:285 "C-23 was MbapFramer" historical rationale is factually wrong — no MbapFramer component ever existed; ss-15/DNP3 was renumbered C-23→C-24 when ARP took C-23. Non-blocking (corrected-from-prose exemption; P33 Slice D). Cosmetic cleanup only. | DEFERRED LOW — fix in F3 cycle or maintenance |
| F3-OBL-STORY114-001 | D-067 carry-forward: STORY-114 must fix src/mitre.rs:91 — change `MitreTactic::IcsImpact => "Impact (ICS)"` to `MitreTactic::IcsImpact => "Impact"` (1-line fix; adjudicated 2026-06-13). | OPEN — target STORY-114 (F4) |
| F3-OBL-STORY114-002 | D-067 carry-forward: when STORY-114 fixes src/mitre.rs:91, also update `.factory/holdout-scenarios/HS-008-*.md:75` — change `"Impact (ICS)"` → `"Impact"` to align holdout assertion with corrected code. | OPEN — target STORY-114 (F4) |
| F3-OBL-STORY114-003 | D-067 carry-forward: STORY-114 test obligations — (a) Display unit test: `assert_eq!(format!("{}", MitreTactic::IcsImpact), "Impact")`; (b) two-bucket report test confirming Impact vs IcsImpact bucket distinctly despite identical Display string (keyed on enum variant); (c) HS-008:75 alignment verified by test. Attach to STORY-114 acceptance criteria in F3 decomposition. | OPEN — target STORY-114 (F3 story authoring) |

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
