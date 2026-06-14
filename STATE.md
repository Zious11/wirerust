---
pipeline: FEATURE_MODE_ARP_ANALYZER
phase: feature-F3-story-decomposition-adversarial-convergence
phase_status: "F3 IN PROGRESS — story decomposition complete (STORY-111..115 created); adversarial convergence STRICT 3/3 IN PROGRESS; Pass-26 remediation COMPLETE (Slice A ZERO [5th consec]; Slice B 1 MED [BC-2.15.007 EC-008 frame_len Some(290)→Some(291) — sibling-row miss from v1.1 EC-004 cleanup; matches shipped compute_dnp3_frame_len(254)=291]; Slice C ZERO [3rd consec]; Slice D 1 HIGH [STORY-INDEX total_points 447→457, wave-TOTAL 442→452, epic-TOTAL 447→457, pre-ARP 400→410; per-story rows correct, grand totals stale; propagated to dependency-graph.md + STATE summary; root cause: pre-ARP grand total was 10 low]; ALL REMEDIATED; BC-2.15.007 v1.4; STORY-INDEX v1.5; dep-graph 452/457; STATE summary 457 pts); clean-streak remains 0/3; next = Pass 27 (clean-streak attempt 1/3); user-directed strict continuation"
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
f3_convergence_trajectory: "F3 STRICT WHOLE-CORPUS IN PROGRESS. P1..P21 detail: STATE.md (inline). P22(10 findings: 1H PRD↔HS-INDEX seed-count 26→28 + HS-W44-001 P0/P1 reconcile, 2 MED VP-INDEX 5-BC note + dep-graph-extended 86→84, 7 LOW SS-15 story-anchors-TBD/DNPXX→DNP3/byte-vector/grammar/VP-008 fuzz-filename/MbapFramer; ALL REMEDIATED — SS-15 24 BCs back-filled STORY-106..110, PRD seed-count reconciled v1.22, VP-024+VP-INDEX corrected, dep-graph-extended superseded, DRIFT-PRD-V120-MBAPFRAMER-001 CLOSED). P23(Slice A ZERO, Slice C ZERO; Slice B: 1 false-pos [Suspicious enum valid — 10 variants incl Suspicious+Impact added STORY-109; BC-2.15.013/018/019 CORRECT]; 1 genuine [canonical-frame LEN 9→8 BC-2.15.011/.023; verified against shipped build_detection_frame length_byte=8]; Slice D: 2 genuine [PRD dangling changelog anchor pass-22-f3-convergence + BC-2.16.004 VP-024 Sub-C mis-anchor; PRD v1.23]; ALL genuine REMEDIATED); clean-streak 0/3. P24(Slice A ZERO, Slice C ZERO; Slice B 1 CRIT [BC-2.15.017 DNPXX_ constant — REVERT of Pass-22 self-induced rename regression; DNPXX_ is the shipped name per src/analyzer/dnp3.rs:169 + STORY-110; verified live src]; Slice D 1 LOW [PRD §2.16 BC-2.16.010 title '(11 Keys)' enrichment]; BOTH REMEDIATED, BC-2.15.017 v1.4 + PRD v1.24); clean-streak 0/3. P25(Slice A ZERO [4th consec]; Slice B ZERO [1st clean]; Slice C 1 HIGH [wave-40-44-holdout.md D14→BC-2.16.014 ×3 — F-ARP-C2 sibling-sweep miss in feature/wave-holdout-scenarios/ skeleton] + 3 LOW [STORY-113:293 + HS-008:75 line-pin de-pins; storm frame-count narrative]; Slice D 1 HIGH [PRD v1.24 dangling §[pass-24-f3-convergence] anchor — Pass-24 added note without creating entry] + 1 LOW [ledger gap]; ALL REMEDIATED — pass-24+pass-25 changelog entries created, all 19 PRD anchors resolve); clean-streak 0/3. P26 (Slice A ZERO 5th-consec, Slice C ZERO 3rd-consec; Slice B 1 MED [BC-2.15.007 EC-008 stale frame_len 290→291, sibling-row miss]; Slice D 1 HIGH [STORY-INDEX total_points 447→457/442→452/400→410 pre-ARP; per-story rows correct, grand totals stale; propagated to dependency-graph.md + STATE summary]; ALL REMEDIATED). Corpus 457 pts canonical. NEXT = Pass 27 (clean-streak attempt 1/3)"
f7_convergence_trajectory: "6 fresh-context adversarial passes; final 3 consecutive CONVERGED (0 P0/CRITICAL/HIGH/MEDIUM)"
consistency_audit: CONSISTENT
input_drift_check: "MATCH=23 STALE=44 ERROR=1 (STORY-091 known); ARP stories STORY-111..115 MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835); STALE=44 are pre-existing older greenfield/feature stories whose BC inputs evolved — expected, non-blocking for F3; scan 2026-06-14"
---

# VSDD Pipeline State — wirerust

## Status

**wirerust v0.6.0 RELEASED (DNP3 TCP analyzer, issue #8). Feature: ARP security analyzer + etherparse 0.16→0.20 migration (F1 PASSED 2026-06-12, D-066); release target v0.7.0. F2 CONVERGED (P33 CLEAN; 3/3 strict-whole-corpus). F3 ARP story decomposition: STORY-111..115 CREATED (epic E-16, 47 pts). D-068: benign GARP emits mitre_techniques: [] (BC-2.16.003 v1.7 + ADR-008 v2.0). D-069 SUPERSEDES D-067: IcsImpact Display canonical = "Impact (ICS)" (research-backed); src/mitre.rs:91 CORRECT as-is; F3-OBL-STORY114-001/002/003 REVOKED. F3 adversarial convergence STRICT 3/3 IN PROGRESS: Pass-26 remediation COMPLETE (Slice A ZERO [5th consec]; Slice B 1 MED [BC-2.15.007 EC-008 frame_len Some(290)→Some(291) — sibling-row miss from v1.1 EC-004 cleanup; matches shipped compute_dnp3_frame_len(254)=291]; Slice C ZERO [3rd consec]; Slice D 1 HIGH [STORY-INDEX total_points 447→457/wave-total 442→452/epic-total 447→457/pre-ARP 400→410 — per-story rows correct, grand totals stale; propagated to dep-graph + STATE summary]); ALL REMEDIATED; clean-streak remains 0/3; NEXT = Pass 27 (clean-streak attempt 1/3).**

**Summary:** 68 stories (48 greenfield + 1 tooling + 19 feature-cycle), 457 pts. 283 BCs (244 pre-F2 + 24 SS-15 + 15 SS-16 ARP), 24 VPs (23 locked + VP-024 ARP draft, locks at STORY-113 F6), 1496 tests green, holdout 0.967. develop HEAD 31d1231; main HEAD 3e29891 (v0.6.0). ARP feature: F1 approved — est. 18-24 new BCs (SS-16), 1 revised BC, VP-024, ADR-008, 5-6 stories (E-16), 3-5 holdout scenarios. MITRE T0830 (primary) + T1557.002 (secondary).

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
| Feature: ARP analyzer — F3 Story Decomposition | **IN PROGRESS** — stories created; adversarial convergence STRICT 3/3 in progress; Pass-26 remediation COMPLETE; clean-streak **0/3**; user-directed strict continuation | STORY-111..115 (E-16, 47 pts, linear chain); 15 SS-16 BCs; waves 40-44 holdouts; BC-2.15.007 v1.4 EC-008 290→291; STORY-INDEX v1.5 totals 447→457/442→452/400→410; dep-graph 452/457; corpus canonical 457 pts; next = Pass 27 (clean-streak attempt 1/3) |

## Session Resume Checkpoint (2026-06-14 — F3 ARP adversarial convergence — Pass-26 remediation complete; strict 3/3 in progress)

**Previous checkpoint (2026-06-14 — F3 ARP adversarial convergence — Pass-25 remediation complete) archived to:
`cycles/feature-arp-v0.7.0/session-checkpoints.md`**

### A. EXACT PIPELINE POSITION

- **Project:** wirerust. Mode: FEATURE MODE. Active feature: ARP security analyzer +
  etherparse 0.16→0.20 migration. GitHub issue #9. Release target: **v0.7.0**.
- **F1 Delta Analysis:** PASSED (human-gated 2026-06-12, D-066).
- **F2 Spec Evolution:** CONVERGED — STRICT WHOLE-CORPUS adversarial loop, 3/3 consecutive
  clean passes (Pass 31/32/33 CLEAN). F2 gate SATISFIED. D-068 + D-069 applied post-F2
  (spec corrections, no F2 re-convergence required).
- **F3 Story Decomposition:** STORIES CREATED. STORY-111..115 (epic E-16, 47 pts) exist;
  HS-INDEX waves 40-44 + holdout scenarios authored; STORY-INDEX/dependency-graph/wave-schedule
  updated. All 5 ARP stories MATCH: STORY-111=d5bda72, STORY-112=268f53f, STORY-113=a767d96,
  STORY-114=e2f1c95, STORY-115=5ca9835.
- **F3 Adversarial Convergence:** STRICT WHOLE-CORPUS, IN PROGRESS.
  **Pass-26 remediation is COMPLETE. Clean-streak: 0/3.**
  **USER DIRECTIVE (2026-06-14): CONTINUE STRICT 3/3 indefinitely.**
  **NEXT ACTION: Run Pass 27 (clean-streak attempt 1/3).**

### B. F3 CONVERGENCE STATUS

- Method: STRICT WHOLE-CORPUS, 4 fresh-context slices per pass:
  - Slice A = architecture + VPs
  - Slice B = all 283 BC bodies
  - Slice C = domain / holdout / MITRE / stories
  - Slice D = PRD + indexes + changelog ledger
- Bar: ZERO findings of ANY severity (incl LOW) across ALL 4 slices; 3 CONSECUTIVE
  fully-clean passes required. Adversary: CLAUDE (Agent tool, `vsdd-factory:adversary`).
  agy is UNUSABLE (40-step cap + quota-exhausted). Absolute paths, no cd (DF-ADVERSARY-METHODOLOGY-001).
- **Pass 17** = first fully-clean pass. **Pass 18** broke streak (VP title-sync).
  **Passes 19/20/21/22/23/24/25/26** each surfaced+remediated genuine items.
  **Pass-26 remediation COMPLETE** (Slice A ZERO [5th consec]; Slice B 1 MED [BC-2.15.007 EC-008 frame_len Some(290)→Some(291) — sibling-row miss from v1.1 EC-004 cleanup; matches shipped compute_dnp3_frame_len(254)=291]; Slice C ZERO [3rd consec]; Slice D 1 HIGH [STORY-INDEX total_points 447→457, wave-TOTAL 442→452, epic-TOTAL 447→457, pre-ARP 400→410 — per-story rows correct, grand totals stale; propagated to dependency-graph.md + STATE summary; root cause: pre-ARP grand total was 10 low]; ALL REMEDIATED; BC-2.15.007 v1.4; STORY-INDEX v1.5; dep-graph 452/457; STATE summary 457 pts).
- 3 deep flush audits completed and cleared: (1) dependency-graph whole-file audit;
  (2) VP-layer + index title-sync audit; (3) story-completeness (BC-PC→AC→test) audit.
  Wave-28-34 dependency-graph gap closed.
- Canonical corpus facts (feed to each adversary dispatch):
  - BCs: 283 total (244 pre-F2 + 24 SS-15 + 15 SS-16 ARP)
  - VPs: 24 total (22 pre-F2 + VP-023 DNP3 + VP-024 ARP draft)
  - MitreTactic variants: 17 (14 Enterprise + 3 ICS: IcsInhibitResponseFunction, IcsImpairProcessControl, IcsImpact)
  - Components: 24 total (C-22 Modbus SHIPPED, C-23 ARP PLANNED, C-24 DNP3 SHIPPED)
  - `Finding.mitre_techniques`: `Vec<String>` + 3 Option fields (`source_ip`, `timestamp`, `direction`)
  - O-01: CLOSED; SEEDED 25 / EMITTED 17 / CAT-ONLY 8 (PLANNED targets; current src 23/15)

### C. DECISIONS CONFIRMED ACTIVE (do not re-adjudicate)

- **D-068 (2026-06-14):** Benign gratuitous ARP emits mitre_techniques: [] (LOW/Anomaly);
  T0830/T1557.002 only on GARP-that-conflicts (BC-2.16.014). Research-backed
  (`.factory/research/arp-garp-mitre-attribution.md`). BC-2.16.003 v1.8 + ADR-008 corrected.
- **D-069 (2026-06-14):** IcsImpact Display canonical = "Impact (ICS)" (distinct from Enterprise
  "Impact" TA0040). SUPERSEDES D-067. src/mitre.rs:91 CORRECT as-is. F3-OBL-STORY114-001/002/003
  REVOKED. Research-backed (`.factory/research/mitre-impact-tactic-disambiguation.md` + WCAG 2.4.6).
  BC-2.10.002/PRD/ADR-007/arp-delta §5.0 corrected.

### D. CURRENT ARTIFACT VERSIONS (for resume verification)

| Artifact | Version |
|----------|---------|
| arp-architecture-delta.md | v1.15 |
| BC-2.16.003 | v1.8 |
| BC-2.16.004 | v1.7 |
| BC-2.15.007 | v1.4 (EC-008 frame_len Some(290)→Some(291); sibling-row miss from v1.1 EC-004 cleanup; matches shipped compute_dnp3_frame_len(254)=291) |
| BC-2.16.007 | v1.3 |
| BC-2.16.010 | v1.7 |
| BC-2.10.002 | v1.5 |
| BC-2.10.005 | v1.10 |
| vp-007 | v2.6 |
| vp-008 | v2.2 |
| vp-024 | v1.7 |
| verification-architecture.md | updated (VP-024 module, VP-008 note, fuzz-filename) |
| tooling-selection.md | updated (fuzz target name fuzz_decode_packet) |
| VP-INDEX.md | updated (Pass-15 A-01 5-BC note corrected) |
| prd.md | v1.24 (BC-2.16.010 row title now includes "(11 Keys)" enrichment — title-sync with BC H1 + BC-INDEX, Criterion-75; v1.24 delta records Pass-24 fixes) |
| BC-2.15.011 | v1.5 (canonical-frame LEN 9→8; verified against shipped build_detection_frame length_byte=8) |
| BC-2.15.017 | v1.4 (REVERT Pass-22 erroneous rename: DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT is canonical shipped name per src/analyzer/dnp3.rs:169 + cli.rs:16/183 + main.rs:192 + STORY-110; broken v1.3 "DNP3_" rename superseded) |
| BC-2.15.023 | v1.4 (canonical-frame LEN 9→8; DF-SIBLING-SWEEP-001 sibling to BC-2.15.011 fix) |
| BC-2.15.001..024 | all 24 story-anchors back-filled to STORY-106..110 |
| STORY-INDEX.md | v1.5 (total_points 447→457, wave-TOTAL 442→452, epic-TOTAL 447→457, pre-ARP 400→410; all 68 per-story rows verified 0 mismatches) |
| dependency-graph.md | updated (Total story points 442/447 → 452/457) |
| dep-graph-extended.md | SUPERSEDED (edge-count 86→84; pointer to dependency-graph.md) |

STORY-114 inputs include BC-2.16.007 (cross-story). All 16 SS-16 BC story-anchors back-filled.
All 24 SS-15 BC story-anchors back-filled to STORY-106..110 (Pass-22 burst).
STATE.md F3-OBL-STORY114-001/002/003 = REVOKED (superseded by D-069).
DRIFT-PRD-V120-MBAPFRAMER-001 = RESOLVED (PRD v1.22 corrected MbapFramer prose).

### E. DURABLE MITIGATIONS / SCOPE NOTES (feed to each adversary dispatch)

- BC-note citations are intentionally VERSION-LESS (e.g. "BC-2.16.007's cross-story delivery
  note") — do not flag missing versions.
- vp-007 NUMERIC current-code "23 seeded / 15 emitted" claims are CORRECT (live src 23/15);
  SEEDED 25/EMITTED 17 are PLANNED post-STORY-114. Only BC-title-QUOTES reflect forward "25".
- F4 implementation dead_code lint handling for deferred fields storm_rate (STORY-114) /
  storm_counters (STORY-113) is OUT OF F3 SCOPE — carry to F4 implementer.
- BC-INDEX inline version-comments are informational; only the title column is load-bearing.
- arp-delta §7 changelog has an enforced ascending-order convention.
- **ThreatCategory::Suspicious is a VALID enum variant** (10 variants total; Suspicious + Impact
  added in STORY-109, src/findings.rs:122). BC-2.15.018 (broadcast anomaly, dnp3.rs:1077) and
  BC-2.15.019 (unsolicited anomaly, dnp3.rs:1207) both use ThreatCategory::Suspicious correctly.
  BC-2.15.013/018/019 category assignments are CORRECT — do NOT flag as invalid. (Pass-23 Slice B
  Finding 1 adjudicated FALSE POSITIVE 2026-06-14.)
- **DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT is the CANONICAL SHIPPED constant name**
  (src/analyzer/dnp3.rs:169, cli.rs:16/183, main.rs:192, STORY-110:91/133/158). "DNPXX" is an
  odd-but-real shipped symbol, NOT a typo for "DNP3". Do NOT flag it as a placeholder and do NOT
  rename it in spec — spec must match code. Pass-22 adversary asserted DNPXX→DNP3 typo without
  checking src; remediation renamed without checking src; the 2-pass self-induced regression was
  caught and reverted at Pass-24. (Pass-24 Slice B Finding 1, BC-2.15.017 v1.4, 2026-06-14.)
- **prd.md immutable-history prose** at §~298 quotes a previously-corrected non-existent anchor
  `§[pass-13-2026-06-13]`; this is documented historical prose, NOT a live dangling anchor. Do
  NOT flag as a defect — it is intentional immutable-history preservation.
- **wave-40-44-holdout.md "D14" references corrected to BC-2.16.014** (Pass-25 burst). "D14" was a
  non-existent detection alias reintroduced by a sibling-sweep miss in the F-ARP-C2 purge. All
  three occurrences now read BC-2.16.014 (canonical BC identifier). NOTE: `mitre-arp-additional-detections.md`
  DOES legitimately contain a different "D14" = deferred 'Unicast ARP request' candidate — that
  entry is CORRECT and must NOT be flagged. The wave-40-44-holdout.md instances were the incorrect
  ones; those are now fixed.
- **All PRD §[changelog anchors] resolve as of Pass-25 burst.** spec-changelog.md now contains
  both [pass-24-f3-convergence-2026-06-14] and [pass-25-f3-convergence-2026-06-14] entries.
  All 19 live PRD §[anchor] citations verified to resolve (excluding the intentional sealed-history
  quote at prd.md:298). Zero dangling anchors.
- **Canonical point total = 457 (410 pre-ARP E-1..E-15 + 47 E-16 ARP); wave-table excl
  STORY-091 = 452; STORY-091 tooling = 5.** Prior values 447/442/400 were stale (pre-ARP grand
  total was 10 low; propagated before ARP +47 was added). Do NOT flag 457/452/410 as incorrect.
  All 68 per-story points-cells verified to match story files (0 mismatches; Pass-26 Slice D).

### F. PROCESS-GAP CODIFICATION BACKLOG (for cycle-close)

- **PG-ARP-F3-DNPXX (Pass-24):** constant/symbol-name "typo" findings MUST be verified against
  shipped src/ and the anchoring story BEFORE remediation. Pass-22 adversary asserted
  DNPXX→DNP3 typo without checking src; orchestrator remediated without checking src;
  introduced a 2-pass regression caught at Pass-24 only because the "verify against shipped
  source" guard was added. Codification candidate: adversary + remediation dispatch must include
  "grep src/ for the symbol" before any rename.
- PG: VP Source-Contract title-quotes must be in the consistency-audit sweep enumeration.
- PG: changelog ascending-order lint candidate (regressed twice).
- PG: F3 gate must force every Task-named BC into frontmatter/inputs or a documented
  cross-story-extension with owning AC+test (D12 back-fill gap class).
- PG: BC-INDEX inline version-comment lag class.
- **PG-ARP-F3-ANCHOR (4th instance — Pass-25):** EVERY burst that adds a PRD delta-note citing
  `spec-changelog §[anchor]` MUST create that changelog entry in the SAME burst. The burst MUST
  end with a grep verifying all PRD `§[anchor]` references resolve. Now enforced in remediation
  dispatches. This class has recurred in Pass-21, Pass-22, Pass-23, and Pass-25 (Pass-24 burst
  added delta-note referencing §[pass-24-f3-convergence-2026-06-14] without creating the entry;
  caught at Pass-25 and remediated). Candidate codification: validate all `§[anchor]`
  cross-references in prd.md as part of consistency audit or a dedicated anchor-resolution lint step.
- **PG-ARP-F3-SIBLING-SKELETON (Pass-25):** Detection-ID/field corrections must sweep
  `.factory/feature/wave-holdout-scenarios/` skeleton holdouts as a SEPARATE layer from
  `.factory/holdout-scenarios/`. The F-ARP-C2 D14 purge swept holdout-scenarios/ but missed
  feature/wave-holdout-scenarios/wave-40-44-holdout.md. Candidate codification: extend
  DF-SIBLING-SWEEP-001 to enumerate feature/wave-holdout-scenarios/ as a required propagation
  perimeter for any detection-ID change.
- **PG-ARP-F3-INDEX-TOTAL (Pass-26):** STORY-INDEX grand-total (total_points, wave-TOTAL,
  epic-TOTAL, pre-ARP subtotal) MUST be recomputed as the sum of per-story `points:` fields
  whenever any story's points change. Stale hand-entered totals drifted 10 pts undetected across
  multiple cycles (pre-ARP total was 400, correct value is 410; propagated stale 447/442 into
  STORY-INDEX header, dependency-graph.md, and STATE.md summary). Candidate gate: F3-entry
  recompute check that sums all `points:` frontmatter values and diffs against STORY-INDEX
  grand totals before adversarial convergence begins.

### G. DEFERRED ITEMS (must not be lost)

- Process-gap codification backlog PG-ARP-F2-003..009 — deferred to F7/cycle-close.
- DRIFT-PRD-V120-MBAPFRAMER-001 — RESOLVED (Pass-22 burst; PRD v1.22 corrects MbapFramer prose).
- F3-OBL-STORY114-001/002/003 REVOKED by D-069 — no revert required.

### H. RESUME COMMAND

1. `vsdd-factory:factory-worktree-health` (BLOCKING — do not proceed if fails).
2. `git -C /Users/zious/Documents/GITHUB/wirerust/.factory log -1 --format='%h %s'` (confirm factory HEAD).
3. `git rev-parse --short HEAD` on develop (expect `31d1231` or newer, clean).
4. `python3 /Users/zious/Documents/GITHUB/wirerust/bin/compute-input-hash --scan` —
   confirm STORY-111..115 MATCH (d5bda72/268f53f/a767d96/e2f1c95/5ca9835).
5. **Resume: dispatch Pass 26 = 4 fresh-context adversary slices (strict whole-corpus,
   zero-any-severity); clean-streak 0/3 → need 3 consecutive clean.**
   DO NOT re-run F1/F2 (converged). DO NOT revert D-068/D-069.

### I. KEY ARTIFACT POINTERS

- ARP architecture delta: `/Users/zious/Documents/GITHUB/wirerust/.factory/specs/architecture/arp-architecture-delta.md`
- F2 convergence trajectory (33 passes): `/Users/zious/Documents/GITHUB/wirerust/.factory/phase-f5-adversarial/arp-f2-convergence-trajectory.md`
- F1 delta analysis: `/Users/zious/Documents/GITHUB/wirerust/.factory/phase-f1-delta-analysis/arp-analyzer-delta-analysis.md`
- Archived checkpoints: `/Users/zious/Documents/GITHUB/wirerust/.factory/cycles/feature-arp-v0.7.0/session-checkpoints.md`

### J. VERIFIED SHAs (re-verify live on resume — snapshot only)

| Ref | Value at checkpoint | Re-verify command |
|-----|--------------------|--------------------|
| develop HEAD | `31d1231` | `git rev-parse --short HEAD` (on develop) |
| main HEAD | `3e29891` | `git log main -1 --format='%h %s'` |
| tag v0.6.0 | annotated → commit `3e29891` | `git show v0.6.0 --format='%h' -s` |
| factory-artifacts HEAD | verify live | `git -C /Users/zious/Documents/GITHUB/wirerust/.factory log -1 --format='%h %s'` |
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
| D-067 | IcsImpact Display adjudication — canonical Display = "Impact" (spec correct; BC-2.10.002 PC3/PC4, PRD §85/823, cap-10, spec-changelog unanimous). src/mitre.rs:91 "Impact (ICS)" is DEVIANT (introduced F-F5-002 as "No BC change" tactical test fix). " (ICS)" suffix does NOT break merge-by-name report bucketing (terminal.rs render_findings_grouped keys on MitreTactic enum variant, not Display string); severity LOW (terminal section-header label only). F2 SPEC CHANGE: NONE — F2 3/3 strict-whole-corpus convergence preserved unaffected. Fix folded into STORY-114 (obligations: 1-line mitre.rs:91 fix; HS-008:75 "Impact (ICS)"→"Impact"; Display unit test; two-bucket enum-level report test). F2→F3 gate condition SATISFIED. **SUPERSEDED BY D-069.** | 2026-06-13 |
| D-068 | Benign gratuitous ARP emits mitre_techniques: [] (LOW/Anomaly severity); T0830 + T1557.002 apply ONLY when GARP conflicts with binding table (BC-2.16.014). Research-backed: MITRE ATT&CK v19.1 T1557.002/DET0387 + T0830; arpwatch/Zeek/Suricata all gate techniques on conflict-detection. Corrected latent over-tagging defect in BC-2.16.003 (→v1.7) and ADR-008 (→v2.0). Propagated to §3.3/STORY-113 AC-003, holdouts, and error-taxonomy. | 2026-06-14 |
| D-069 | IcsImpact Display canonical = "Impact (ICS)" (distinct from Enterprise "Impact" TA0040). SUPERSEDES D-067. Research-backed: MITRE TA0040 (Enterprise Impact) vs TA0105 (ICS Impact) are distinct tactic families; WCAG 2.4.6 requires unique headings/labels. src/mitre.rs:91 "Impact (ICS)" is CORRECT — not deviant. STORY-114 D-067 revert obligations (F3-OBL-STORY114-001/002/003) REVOKED. 2 shipped DNP3 F5 distinctness tests preserved (they test enum-variant identity, not Display string equality). Spec side corrected: BC-2.10.002 (→v1.5), PRD §85/882, ADR-007, arp-architecture-delta §5.0. | 2026-06-14 |

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
| F3-OBL-STORY114-001 | D-067 carry-forward: STORY-114 must fix src/mitre.rs:91 — change `MitreTactic::IcsImpact => "Impact (ICS)"` to `MitreTactic::IcsImpact => "Impact"` (1-line fix; adjudicated 2026-06-13). **REVOKED by D-069 (2026-06-14): src/mitre.rs:91 "Impact (ICS)" is CORRECT; no revert.** | REVOKED — superseded by D-069 |
| F3-OBL-STORY114-002 | D-067 carry-forward: when STORY-114 fixes src/mitre.rs:91, also update `.factory/holdout-scenarios/HS-008-*.md:75` — change `"Impact (ICS)"` → `"Impact"` to align holdout assertion with corrected code. **REVOKED by D-069 (2026-06-14): HS-008-*.md:75 stays "Impact (ICS)"; no revert.** | REVOKED — superseded by D-069 |
| F3-OBL-STORY114-003 | D-067 carry-forward: STORY-114 test obligations — (a) Display unit test: `assert_eq!(format!("{}", MitreTactic::IcsImpact), "Impact")`; (b) two-bucket report test confirming Impact vs IcsImpact bucket distinctly despite identical Display string (keyed on enum variant); (c) HS-008:75 alignment verified by test. **REVOKED by D-069 (2026-06-14): canonical Display is "Impact (ICS)"; D-067 test obligations nullified. VP-007 obligation in STORY-114 unchanged.** | REVOKED — superseded by D-069 |
| DNPXX-SOURCE-RENAME-001 | src constant `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT` is an ugly placeholder-style name shipped in v0.6.0; candidate code-cleanup rename DNPXX_→DNP3_ (6 files: dnp3.rs/cli.rs/main.rs/tests + arp-delta + STORY-110) — OUT OF F3 SCOPE (spec/story only); defer to a code-quality maintenance sweep. Requires DF-VALIDATION-001 research-agent validation before GitHub issue filing. | DEFERRED LOW |

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
