---
document_type: pipeline-state
project: wirerust
mode: feature
phase: F2-spec-evolution
status: paused
current_step: "F2 adversarial Pass-8: NOT-CLEAN, 1 HIGH (coverage_gaps JSON schema contradiction BC-2.12.023 PC-3 flat-dict vs BC-2.12.024 PC-5 object) — remediated (D-331). BC-INDEX v2.12, PRD v1.51, ARCH-INDEX v2.10, VP-INDEX v2.31. Entering Pass-9 (0/3 consecutive clean passes)."
pipeline: FEATURE-CYCLE
timestamp: 2026-07-01T18:30:00Z

# Release chain (latest)
released_version: v0.11.1
released_at: "2026-07-01"
release_tag: v0.11.1
release_tag_object: e8a8a2d4e7cd03e337b066859586e2c610208888
release_commit: 4e2b28529ae196785ce6a0baed522b9939f929ea
release_url: https://github.com/Zious11/wirerust/releases/tag/v0.11.1
prior_released_version: v0.11.0
prior_released_at: "2026-06-29"
# Ground-truth HEADs (verified 2026-07-01 — PR #347 main merge + #348 develop back-merge)
main_head: 4e2b28529ae196785ce6a0baed522b9939f929ea
develop_head: 3a60317965e62bef9895e857c8a26fc3b8d03ad0
# Cargo.toml version on main and develop (in sync)
cargo_version_main: "0.11.1"
cargo_version_develop: "0.11.1"
# Open worktrees: main checkout [develop] + .factory [factory-artifacts]. release/back-merge worktrees removed.
# Pipeline completion
bootstrapped: 2026-05-19T16:56:48Z
adversary_gate: SATISFIED
adversary_convergence_counter: SATISFIED
# Story tracking
stories_delivered: 94
story_index_version: v3.10
total_stories: 103
story_index_note: "103 stories / 66 waves. STORY-148/149/150 added (maint-2026-07-01). IDX-003 total_points reconciled 656→659. develop=3a60317."
# Spec versions (current)
bc_index_version: "v2.12"
vp_index_version: "v2.31"
arch_index_version: "v2.10"
prd_version: "v1.51"
epics_version: v1.8
# DTU
dtu_required: false
dtu_assessment: 2026-05-20
dtu_clones_built: n/a
dtu_services: []
# Maintenance
maintenance_run: COMPLETE
maintenance_run_id: maint-2026-07-01
maintenance_started_at: "2026-07-01"
maintenance_completed_at: "2026-07-01"
maintenance_prior_run: maint-2026-06-22
---

# VSDD Pipeline State — wirerust

## EXACT RESUME POINT

**F2 adversarial Pass-8 REMEDIATED (D-331). 1 HIGH (F-F2P8-001: coverage_gaps JSON schema contradiction — BC-2.12.023 PC-3 flat-dict vs BC-2.12.024 PC-5 {caveat_l2, entries[]} object); BC-2.12.023 reconciled to authoritative object form. BC-INDEX v2.12, PRD v1.51, ARCH-INDEX v2.10, VP-INDEX v2.31. Entering Pass-9 (0/3 consecutive clean passes). See Session Resume Checkpoint below.**

---

## Project Metadata

| Field | Value |
|-------|-------|
| Project | wirerust |
| Mode | feature (cycle: feature-protocol-coverage; F2 spec evolution) |
| Version | 0.11.1 (released) |
| Main HEAD | `4e2b285` (full: `4e2b28529ae196785ce6a0baed522b9939f929ea`) |
| Develop HEAD | `3a60317` (full: `3a60317965e62bef9895e857c8a26fc3b8d03ad0`) |
| Tag v0.11.1 | commit `4e2b285`; tag object `e8a8a2d4` |
| GitHub release | https://github.com/Zious11/wirerust/releases/tag/v0.11.1 (Latest, not draft) |
| Factory artifacts HEAD | see `git -C .factory log -1 --format='%h %s'` |
| Spec versions | BC-INDEX v2.12 (345 active / 346 on disk) / VP-INDEX v2.31 (43 VPs) / ARCH-INDEX v2.10 / PRD v1.51 |
| Stories | 94 delivered / 103 total (STORY-INDEX v3.10) |

---

## Phase Progress

| Phase | Status | Notes |
|-------|--------|-------|
| Phase 0–7 + v0.1.0..v0.5.0 | RELEASED | Greenfield through MITRE v19 remap |
| Feature DNP3 (E-8) + v0.6.0..v0.11.0 | RELEASED | Details: cycles/ subdirs |
| Maintenance maint-2026-06-22 | COMPLETE 2026-06-23 | 38 observations; 0 blocking |
| Maintenance maint-2026-07-01 | **COMPLETE 2026-07-01** | PRs #349+#350 merged; develop=3a60317; STORY-148/149/150 drafted |
| Feature cycle fix-tls-clienthello-frag — F1 | DONE | delta-analysis.md committed |
| Feature cycle fix-tls-clienthello-frag — F2 | APPROVED (D-305, 2026-06-29) | 6 new BCs + 3 amended + VP-039 + VP-040 + ADR-011 |
| Feature cycle fix-tls-clienthello-frag — F3 | APPROVED (D-306, 2026-06-29) | STORY-144..146; STORY-INDEX v3.6; HS-F4-001..012 |
| Feature cycle fix-tls-clienthello-frag — F4 | **DONE/PASS** | Holdout 0.904 mean, 8/8 must-pass; HS-F4-001 artifact-fidelity fix |
| Feature cycle fix-tls-clienthello-frag — F5 | **DONE/CONVERGED** | 5 passes; BC-completeness 60/60, 0 P0; BC-INDEX v2.3 |
| Feature cycle fix-tls-clienthello-frag — F6 | **DONE** | Kani VP-039 3 proofs PASS; fuzz 1.9M execs clean; 100% real-gap mutation kill (mod f6_hardening, 12 tests); anyhow 1.0.103 (RUSTSEC-2026-0190 cleared). PRs #345+#346 merged. develop=52907bc. |
| Feature cycle fix-tls-clienthello-frag — F7 | **DONE/CONVERGED (D-316)** | v0.11.1 released (PR #347 main, #348 back-merge); S-7.02 SATISFIED; cycle CLOSED. |
| Feature cycle feature-protocol-coverage — F1 (delta-analysis) | **DONE** | Artifacts: `.factory/phase-f1-delta-analysis/feature-protocol-coverage-delta-analysis.md` + `affected-files.txt` + `feature-protocol-coverage-research.md`. Impact: 5 source files (new SS-18 `src/protocols.rs`, `dispatcher.rs`, `cli.rs`, `main.rs`, `lib.rs`). 9 new BCs / 2 amended / 2 new VPs (VP-041/VP-042) / 1 new ADR (ADR-012) / new subsystem SS-18. ~5 stories / ~23 pts / 3 waves. Regression risk MEDIUM (dispatcher carries VP-004 Kani harnesses). |
| Feature cycle feature-protocol-coverage — F2 (spec-evolution) | **IN PROGRESS** | Spec-layer DONE (D-323). Pass-1: NOT-CLEAN (14) REMEDIATED (D-324). Pass-2: NOT-CLEAN (8: 1H,4M,3L) REMEDIATED (D-325). Pass-3: NOT-CLEAN (4: 2H,2M) REMEDIATED (D-326). Pass-4: NOT-CLEAN (3: 2H,1M) REMEDIATED (D-327). Pass-5: NOT-CLEAN (6: 1H,3M,2L) REMEDIATED (D-328). Pass-6: NOT-CLEAN (1H+1M+2L version-metadata hygiene; core spec CONTENT converged) REMEDIATED (D-329). Pass-7: NOT-CLEAN (3H+1M PRD-narrative + ARCH-INDEX registry propagation gaps) REMEDIATED (D-330). Pass-8: NOT-CLEAN (1H coverage_gaps JSON schema BC-2.12.023 vs BC-2.12.024) REMEDIATED (D-331). BC-INDEX v2.12, PRD v1.51, ARCH-INDEX v2.10. Entering Pass-9 (0/3 consecutive clean). Finding trajectory: 14→8→4→3→4→4→4→1(H). |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **F2 adversarial Pass-8: NOT-CLEAN, 1 HIGH (coverage_gaps JSON schema contradiction) — remediated** | **DONE (D-331)** | HIGH F-F2P8-001: BC-2.12.023 PC-3 used flat-dict form (`"<transport>/<port>": {count, state}`) contradicting BC-2.12.024 PC-5 authoritative object form (`{caveat_l2, entries[]}`); flat-dict structurally incompatible with mandatory caveat_l2 field (BC-2.12.024 Inv-1) and per-entry collision_note. BC-2.12.023 PC-3 corrected to authoritative object form; PC-3 now references BC-2.12.024 PC-5 as field-level authority. BC-INDEX v2.12. All other axes CLEAN. |
| **F2 adversarial Pass-7: NOT-CLEAN, 3 HIGH + 1 MEDIUM (PRD-narrative + ARCH-INDEX registry propagation gaps) — ALL remediated** | **DONE (D-330)** | HIGH F-F2P7-001: ARCH-INDEX Subsystem Registry SS-05 9→11 (BC-2.05.010..011 never propagated), SS-12 21→24 (BC-2.12.022..024 never propagated). HIGH F-F2P7-002: PRD §2.18 L2 caveat had only 4 protocols; 5th Ethernet POWERLINK (0x88AB) added. HIGH F-F2P7-003: PRD §2.18 narrative + ARCH-INDEX SS-18 comment still cited VP-041 as single harness; corrected to two harnesses (oracle_cross_check + partition_invariant). MEDIUM F-F2P7-004: BC-2.18.003/004 partition-harness non-vacuity mislabel corrected (partition holds trivially; oracle_cross_check is the non-vacuous guard). ARCH-INDEX v2.10 (also swept SS-11 34→35 BC-2.11.035, SS-16 15→16 BC-2.16.016 — pre-existing ARCH-INDEX-COUNT-DRIFT-001 CLOSED). PRD v1.51. BC-INDEX v2.11. |
| **F2 adversarial Pass-6: NOT-CLEAN, 1 HIGH + 1 MED + 2 LOW (all version-metadata hygiene) — ALL remediated; core spec CONTENT converged** | **DONE (D-329)** | HIGH F-F2P6-001: BC-2.18.004 file v1.0 vs BC-INDEX v1.1 — bumped to v1.1. MED F-F2P6-002: BC-2.05.010 under-versioned at v1.0 vs index v1.2. Full 9-BC sweep: BC-2.18.001→v1.3, .002→v1.1, .004→v1.1, BC-2.05.010→v1.2, BC-2.12.023→v1.1. LOW F-F2P6-003/004: arch-delta snapshot disclaimer + Decision 10 + path fix. BC-INDEX v2.10. Adversary confirmed core spec CONTENT converged. |
| **F2 adversarial Pass-4: NOT-CLEAN, 3 findings (2 HIGH, 1 MED) — ALL remediated** | **DONE (D-327)** | HIGH F-F2P4-001: BC-2.05.011 phantom DispatchTarget::Arp/::Dns variants removed; real enum {Http,Tls,Modbus,Dnp3,Enip,None}; EC-008 reframed. HIGH F-F2P4-002: VP-042 false UDP-via-dispatcher clause deleted ×6 locations. MED F-F2P4-003: coverage-matrix (u16,u16)→(TransportProto,u16). BC-INDEX v2.8. |
| **F2 adversarial Pass-5: NOT-CLEAN, 4 substantive findings (1 HIGH, 3 MED) + 2 LOW — ALL remediated** | **DONE (D-328)** | HIGH F-F2P5-001: SUPPORTED_PORTS reframed — not pure classify() mirror; DNS/53 = decode-loop, ARP = DecodedFrame::Arp, PERMANENT BY DESIGN. MED F-F2P5-002/003: arch-delta synced. MED F-F2P5-004: BC-2.12.024 tri-state transport-aware. BC-INDEX v2.9, PRD v1.50, ss-18 v1.4. |

---

## Decisions Log

D-001..D-301: see `cycles/*/decisions-archive.md` (greenfield → feature-enip-v0.11.0)

| ID | Decision | Date |
|----|----------|------|
| D-302 | Dependabot PRs #325+#311 merged. develop `a2d8c13`. | 2026-06-29 |
| D-303 | Cycle `fix-tls-clienthello-frag` started. Full F1-F7. Maintenance paused. | 2026-06-29 |
| D-304 | F2 CONVERGED: 5 new BCs + 2 amended + VP-039 + ADR-011. | 2026-06-29 |
| D-305 | F2 APPROVED + F-EV-001 scope: BC-2.07.043 + VP-040. BC-INDEX v2.1, PRD v1.45. | 2026-06-29 |
| D-306 | F3 APPROVED. STORY-144..146; STORY-INDEX v3.6; HS-F4-001..012. Pre-F4 PASS. | 2026-06-29 |
| D-307 | STORY-144 MERGED PR #341 `0986e878`. SEC-001 DoS fixed. Wave 65 DONE. stories_delivered=92. | 2026-06-29 |
| D-308 | Session paused at STORY-145 mid-TDD (Red Gate `f60c0e0`, branch pushed). VP-INDEX corrected to v2.28. | 2026-06-30 |
| D-309 | STORY-145 MERGED PR #343 squash `d3d2e19`. Per-story convergence 5 passes, APPROVE. stories_delivered=93. | 2026-06-30 |
| D-310 | STORY-146 MERGED PR #344 squash `8b52046`. Per-story convergence multi-pass. stories_delivered=94. Wave 66 COMPLETE. | 2026-06-30 |
| D-311 | F4 holdout PASS (mean 0.904 ≥ 0.85; must-pass 8/8). HS-F4-001 verdict B+C. BC-2.07.038 v2.8. BC-INDEX v2.2. | 2026-06-30 |
| D-312 | F5 scoped adversarial CONVERGED. 60/60 BC-completeness. BC-2.07.038 v2.10. Re-anchor 7 BCs. BC-INDEX v2.3. | 2026-06-30 |
| D-313 | F6 targeted hardening IN PROGRESS (paused for session clear). Kani VP-039 (3 proofs, non-vacuous) + fuzz (1.9M execs clean). 13 mutation-gap tests remain. RUSTSEC-2026-0190 open. | 2026-06-30 |
| D-314 | F6 DONE. PR #345 merged (squash d7f0ef4): 12 mutation-gap tests mod f6_hardening — 100% real-gap kill (13 gaps closed; 2 provably-equiv survivors at tls.rs:950:59 documented). PR #346 merged (squash 52907bc): anyhow 1.0.102→1.0.103, RUSTSEC-2026-0190 cleared, cargo deny PASS. F6-MUTATION-GAPS-001 RESOLVED. RUSTSEC-2026-0190 RESOLVED. SEC-002/SEC-006 closed-by-design (mod f6_hardening themes 1+2+6 pin exact-MAX_BUF + clear-and-recover). develop=52907bc. F7 next. | 2026-07-01 |
| D-315 | Gitflow merge-settings alignment. Enabled allow_merge_commit=true repo-wide; main branch protection required_linear_history=false (accepts gitflow merge commits for releases + back-merges). develop keeps required_linear_history=true (squash-only, D-289 preserved). Refines D-289 + D-290. Root-caused B1: v0.11.0 squash into main left branches diverged; recurred because back-merge was skipped and squash-into-main prevented shared ancestry. | 2026-07-01 |
| D-316 | Cycle fix-tls-clienthello-frag CLOSED / CONVERGED. Released v0.11.1 (PR #347 gitflow merge into main `4e2b285`; tag `v0.11.1` object `e8a8a2d4`; GH Release published, 4 assets, NOT crates.io per D-300). Back-merged to develop PR #348 squash `ba6fbd8`. Both at 0.11.1 in sync. F6: Kani VP-039 3 non-vacuous proofs; fuzz 1.9M clean; 100% real-gap mutation kill (13/13; 2 dead-code survivors ADR-011). PRs #341/#343/#344/#345/#346/#347/#348. S-7.02 SATISFIED (STORY-147 PG-MUTANTS-JOBS-001; PG-BC-ANCHOR-VALIDATION-001 + DF-KANI-NONVACUITY-001-PROPTEST-GAP justified-deferred). | 2026-07-01 |
| D-317 | Maintenance run maint-2026-07-01 STARTED. D-303 pause lifted. Sweeps: dep/supply-chain, security, code-quality/pattern, doc/comment-drift, spec/anchor-drift, performance (6 total; UI/design-drift skipped — CLI only). develop @ ba6fbd8, v0.11.1. Log: `.factory/cycles/maint-2026-07-01/maintenance-log.md`. | 2026-07-01 |
| D-318 | maint-2026-07-01 COMPLETE. 2 doc cleanup PRs merged (#349 squash b451c481 — 9 stale RED-tense/todo!() comments; #350 squash 3a60317 — README ENIP+TLS-reassembly docs + ADR-011 promoted to docs/adr/0011 + CLAUDE.md ADR list 0010+0011). develop=3a60317. SEC-005/006 (ENIP on_flow_close unwired + DNP3 flow-map unbounded) → STORY-148 (E-20, 5 pts). Perf regression PERF-001/002/003-005 + benchmark gap → STORY-149 (E-11, 5 pts). TLS-DRAIN-DUP-001 (~220-line C2S/S2C duplication) → STORY-150 (E-11, 5 pts). Spec/anchor drift BC-ANCHOR-DRIFT-OUTOFCYCLE-001 expanded (12 stale sites, exact fixes captured), ARCH-INDEX-COUNT-DRIFT-001, TLS-SUMMARIZE-MAPTYPE-001, SEC-004/007, SEC-001-ENIP, MAINT-SC-001 deferred to backlog. IDX-003 total_points reconciled 656→659 (STORY-121 3 pts never added at v2.0). Audit/deny/pins clean. 0 STALE input-hashes (STORY-148/149/150 have inputs:[]). | 2026-07-01 |
| D-319 | Session paused for clear at 2026-07-01; durable resume checkpoint written. Pipeline at rest, no active cycle. | 2026-07-01 |
| D-320 | Feature cycle `feature-protocol-coverage` STARTED (feature: list protocols wirerust does NOT dissect — both static coverage report + dynamic undissected-traffic detection). F1 delta-analysis DONE. Scope gate APPROVED by human with research-backed decisions: OQ-1 = ICS + core-IT curated catalog (~28-32 entries, category-tagged); OQ-2 = new CoverageGapsSummary report section using Suricata-style known/unknown/failed states; OQ-5 = TCP+UDP dynamic detection this cycle (BACnet/IP udp/47808 is Tier-1, must be flaggable — expands STORY-153 beyond F1's TCP-only assumption); OQ-3 = terminal + --json; OQ-4 = default to explicit --coverage-gaps flag (do NOT auto-enable under analyze --all), to confirm in F2. Research report `.factory/phase-f1-delta-analysis/feature-protocol-coverage-research.md` (25+ ICS protocols tiered; L2/multicast protocols GOOSE/SV/PROFINET-RT flagged as port-undetectable; port-102 collision S7comm/MMS/ICCP noted; hand-curated catalog confirmed correct — no auto-source). Entering F2 spec evolution. | 2026-07-01 |
| D-321 | F2 design-layer DONE (SS-18, ADR-012, VP-041/042, index bumps ARCH v2.6 / VP v2.30). Session paused for clear before F2 spec-layer (BCs+PRD). Scope-drift F2-SCOPE-DRIFT-UDP-001 logged: ADR-012 dec #7 says TCP-only but D-320/OQ-5 approved TCP+UDP — reconcile on resume before authoring BC-2.05/BC-2.12. | 2026-07-01 |
| D-322 | F2-SCOPE-DRIFT-UDP-001 RESOLVED. ADR-012 Decision 6 corrected TCP-only→TCP+UDP (D-320 OQ-5 approved scope). Decision 3a updated (TCP-only caveat→L2/multicast structural caveat). Consequences updated: HashMap key type changed from (u16, u16) direction-normalized port pair to (TransportProto, u16) 2-tuple. SS-18 Dynamic Detection Scope section, Bounded-Resource Note, and Subsystem Purpose updated. ARCH-INDEX v2.6→v2.7 (Bounded-Resource note, ADR-012 row, SS-18 registry comment, modified log). module-decomposition.md C-21 updated with new field and TransportProto note. architecture-delta.md OQ-5/SS-05/decisions/BC-2.05 anchoring/mandatory caveat all reconciled. BACnet/IP UDP/47808 is now flaggable. Product-owner unblocked for BC-2.05.010/011, BC-2.12.022/024. VP-042 flagged for harness key-type amendment at story-authoring time (property description valid; harnesses use new (TransportProto, u16) type). VP-041 unaffected. | 2026-07-01 |
| D-323 | F2 spec-layer authored & integrated. 9 BCs (BC-2.18.001..004, BC-2.05.010..011, BC-2.12.022..024) with (TransportProto,u16) keying + UDP counting; CAP-18; PRD v1.46; BC-INDEX v2.4. Deferred-to-F3: AMB-001-ARP-ETHERTYPE, AMB-002-JSON-FLAG-SCOPE. Next: F2 adversarial spec convergence. | 2026-07-01 |
| D-324 | F2 adversarial spec Pass-1 complete + remediated. Findings F-F2P1-001..014 all fixed. P0 F-F2P1-002: BC-2.05.010 false no-UDP-dissector premise removed + DNS-53 mis-count + UDP key min(src,dst). HIGH: GOOSE ethertype 34992→35000 (0x88B8), ProtocolCategory L2-variant removed ({ICS,IT} only), ARP iff-invariant weakened to one-way implication. Design: VP-043 added (main.rs UDP path, proptest, P1), VP-041 oracle-reframe (proptest_vp041_oracle_cross_check), UDP key min(src,dst), --ics-only not shipping, tri-state known-supported, HART-IP single-canonical UDP:5094. BC-INDEX v2.5, PRD v1.47, VP-INDEX v2.31, ARCH-INDEX v2.8. Next: Pass-2 fresh-context adversary (need 3 clean passes). | 2026-07-01 |
| D-325 | F2 adversarial spec Pass-2 complete + remediated. Findings F-F2P2-001..008 all fixed. HIGH F-F2P2-001: BC-2.18.003/ss-18 false 'VP-041 detects classify() drift' claim corrected to match ADR-012 Dec-5 (unenforced convention). NEW ADR-012 Decision 10: UDP gap-classification (can_decode) evaluated regardless of enable_dns when --coverage-gaps active (prevents false known-supported signals). VP-041 2nd harness (proptest_vp041_partition_invariant) propagated to BCs+PRD RTM. POWERLINK 0x88AB/34987 externally verified (IEEE RA registry, HIGH) — [unverified] removed; V1 0x3E3F excluded. L2 caveat now names 5 protocols. BC-INDEX v2.6, PRD v1.48, ARCH-INDEX v2.9. Next: Pass-3 fresh-context adversary (0 consecutive clean; need 3). | 2026-07-01 |
| D-326 | F2 adversarial spec Pass-3 complete + remediated. F-F2P3-001..004 all fixed. HIGH F-F2P3-001: BC-2.18.001 EC-001 ARP LinkLayer self-contradiction (ARP is a supported LinkLayer entry) reworded — EC-001 now states no L2/multicast PROTOCOL entries (GOOSE, SV, PROFINET-RT/DCP, EtherCAT, POWERLINK) in --supported set, explicitly noting ARP is the sole transport=LinkLayer entry that IS supported. PC-5 amended (LinkLayer entries with ethertype=None render — in EtherType column). HIGH F-F2P3-002: BC-2.05.010/011 + PRD RTM §2.18.B cited phantom VP-043 harness (proptest_vp043_udp_counter_exactness); replaced with 2 canonical harnesses per VP-INDEX v2.31 (proptest_vp043_total_count_equals_n + proptest_vp043_no_increment_on_classified_udp) in 4 BC locations + PRD RTM. MEDIUM F-F2P3-003: cap-18 §Key caveats L2 list updated to include Ethernet POWERLINK as 5th entry. MEDIUM F-F2P3-004: BC-INDEX BC-2.05.010 comment (Udp,dst_port)→min(src,dst) + VP-043 split; BC-2.05.011 comment VP-042 Sub-A/B/C→VP-042 (TCP)+VP-043 (UDP). BC-INDEX v2.7, PRD v1.49. Finding trajectory 13→8→4 (converging). Counter: 0 consecutive clean. Next: Pass-4 fresh-context adversary (need 3 consecutive clean). | 2026-07-01 |
| D-327 | F2 adversarial spec Pass-4 complete + remediated. F-F2P4-001..003 all fixed. HIGH F-F2P4-001: BC-2.05.011 referenced non-existent DispatchTarget::Arp/::Dns — real enum is {Http,Tls,Modbus,Dnp3,Enip,None}; ARP handled outside dispatcher via DecodedFrame::Arp; EC-008 corrected (TCP/53 None-target DOES increment — no DispatchTarget::Dns exists). HIGH F-F2P4-002: VP-042 text falsely claimed UDP-via-dispatcher (Udp,…) keys in 5 locations across VP-INDEX (×2), verification-architecture.md (×2), verification-coverage-matrix.md (×2) — contradicted TCP-only dispatcher invariant + ADR-012 Dec-6; deleted in all 6 locations; VP-042 now uniformly TCP-only (UDP is VP-043's sole responsibility). MEDIUM F-F2P4-003: verification-coverage-matrix.md stale (u16,u16) key type corrected to (TransportProto,u16); per-port-pair wording corrected to per-(TransportProto,u16)-key. BC-INDEX v2.8. PRD stays at v1.49 (no phantom variants found in PRD). VP-INDEX v2.31, ARCH-INDEX v2.9 unchanged (wording-only deletions, no version bump). Finding trajectory 13→8→4→3 (converging). Counter: 0 consecutive clean. Next: Pass-5 fresh-context adversary (need 3 consecutive clean). | 2026-07-01 |
| D-328 | F2 adversarial spec Pass-5 complete + remediated. HIGH F-F2P5-001: SUPPORTED_PORTS reframed — it is NOT a pure mirror of classify(); port 53 (DNS) and ARP are dissected outside classify() by design (decode-loop / DecodedFrame::Arp), permanent-not-drift; BC-2.18.003 doc-comment obligation reframed to 'dissection path'; contradiction with BC-2.05.011 EC-008 resolved. MEDIUM F-F2P5-002/003: architecture-delta working doc synced (false VP-041 non-vacuity claim removed, POWERLINK 5th L2 added, VP-041 2-harness count). MEDIUM F-F2P5-004: BC-2.12.024 tri-state lookup now transport-aware (Tcp,53)→unknown, (Tcp,47808)→unknown. LOW: PRD RTM title 'the' fix; F3 forward obligation (main.rs can_decode enable_dns decoupling) recorded. BC-INDEX v2.9, PRD v1.50, ss-18 v1.4. Finding trajectory 14→8→4→3→4 (converging on quality; HIGH count 3→2→2→1→1). Counter: 0 consecutive clean. Next: Pass-6 fresh-context adversary (need 3 consecutive clean). | 2026-07-02 |
| D-329 | F2 adversarial spec Pass-6 complete + remediated. Adversary confirmed core spec CONTENT converged (scope, BC logic, symbol grounding, protocol constants, VP non-vacuity, cross-doc counts all clean). Remaining findings were version-metadata hygiene: HIGH F-F2P6-001 BC-2.18.004 file v1.0 vs index v1.1; MEDIUM F-F2P6-002 BC-2.05.010 under-versioned. Full 9-BC version sweep reconciled 5 BCs (BC-2.18.001→v1.3, .002→v1.1, .004→v1.1, BC-2.05.010→v1.2, BC-2.12.023→v1.1); all file versions now match BC-INDEX rows. LOW F-F2P6-003/004: architecture-delta given a HISTORICAL SNAPSHOT disclaimer (durable fix — points to canonical docs) + Decision 10 summary + path fix. BC-INDEX v2.10. Finding trajectory 14→8→4→3→4→(1H+1M+2L). Counter: 0 consecutive clean. Next: Pass-7 fresh-context adversary (need 3 consecutive clean). | 2026-07-01 |
| D-330 | F2 adversarial spec Pass-7 complete + remediated. Systematic propagation-sweep-gap surfaced: PRD §2.18 narrative + ARCH-INDEX subsystem registry were never swept by BC-centric earlier passes. HIGH F-F2P7-001: ARCH-INDEX SS-05 9→11 (BC-2.05.010..011 added), SS-12 21→24 (BC-2.12.022..024 added) — registry counts were never updated when BCs were authored. HIGH F-F2P7-002: PRD §2.18 L2 caveat had only 4 protocols; Ethernet POWERLINK (0x88AB) added as 5th (consistent with BCs v2.2→pass-2 and ADR-012 Decision 3a). HIGH F-F2P7-003: PRD §2.18 narrative + ARCH-INDEX SS-18 registry comment cited VP-041 as "single harness"; corrected to "two harnesses proptest_vp041_oracle_cross_check + proptest_vp041_partition_invariant". MEDIUM F-F2P7-004: BC-2.18.003/004 partition-harness "non-vacuous / oracle computed independently" mislabeling corrected — partition holds trivially (unsupported = KNOWN \ supported by definition); oracle_cross_check is the non-vacuous guard. Full sweep also closed ARCH-INDEX-COUNT-DRIFT-001: SS-11 34→35 (BC-2.11.035, never propagated from maint-2026-06-22 sweep) + SS-16 15→16 (BC-2.16.016, never propagated from Pass-1 ARP additions). ARCH-INDEX v2.10, PRD v1.51, BC-INDEX v2.11. Counter: 0 consecutive clean. Next: Pass-8. | 2026-07-01 |
| D-331 | F2 adversarial spec Pass-8 complete + remediated. HIGH F-F2P8-001: coverage_gaps --json schema contradiction — BC-2.12.023 PC-3 flat-dict (`"<transport>/<port>": { count, state }`) vs BC-2.12.024 PC-5 authoritative object form (`{ "caveat_l2": <string>, "entries": [ { transport, port, count, state, name?, collision_note? } ] }`); flat-dict structurally incompatible with mandatory caveat_l2 field (BC-2.12.024 Inv-1) and per-entry collision_note (port-102 four-way collision); reconciled BC-2.12.023 PC-3 to the authoritative object form; PC-3 now references BC-2.12.024 PC-5 as field-level authority. Adversary confirmed ALL other axes clean (protocol values, counts, index/version propagation, ARCH registry, PRD narrative, VP arithmetic, non-vacuity, anchors). BC-INDEX v2.12. Finding trajectory 14→8→4→3→4→4→4→1 (single HIGH). Counter: 0 consecutive clean. Next: Pass-9 (spec appears converged pending re-confirmation). | 2026-07-01 |

---

## Skip Log

| Step | Justification |
|------|---------------|
| crates.io publish (v0.11.0) | Human declined at D-300 — not published |
| Holdout formal eval HS-110..122 | Deferred post-release per D-267; 10/13 behaviors covered by unit tests |
| DTU creation | Not required (passive analyzer; no external service calls) — D-dtu-assessment 2026-05-20 |

---

## Blocking Issues

| ID | Summary | Priority | Owner | Status |
|----|---------|----------|-------|--------|
| F2-SCOPE-DRIFT-UDP-001 | ADR-012 Decision 6 corrected from TCP-only to TCP+UDP dynamic detection. All docs reconciled. (TransportProto, u16) keying consistent. | HIGH | architect | **RESOLVED 2026-07-01** |

---

## Open Items / Backlog (DF-VALIDATION-001-gated unless noted)

| ID | Summary | Priority | Status |
|----|---------|----------|--------|
| PG-MUTANTS-JOBS-001 | `cargo mutants --jobs 8` masks survivors. | MEDIUM | **CODIFIED → STORY-147** (draft, E-11, 3 pts) |
| SEC-005 + SEC-006 | ENIP on_flow_close unwired (CWE-400 DoS); DNP3 flow-map no cap. | MEDIUM | **→ STORY-148** (E-20, 5 pts, draft) |
| PERF-001/002 + BENCHMARK-GAP-001 | TLS carry-path +10.3% regression; HashMap + Vec alloc hotspots; no fragmented-handshake fixture. | HIGH | **→ STORY-149** (E-11, 5 pts, draft) |
| TLS-DRAIN-DUP-001 | ~220-line C2S/S2C drain-loop duplication in tls.rs. | MEDIUM | **→ STORY-150** (E-11, 5 pts, draft) |
| BC-ANCHOR-DRIFT-OUTOFCYCLE-001 | 12 stale tls.rs anchor sites; exact fixes in maintenance-log.md. | LOW | Deferred — next sweep or fold into STORY-150 |
| ARCH-INDEX-COUNT-DRIFT-001 | SS-11 34→35, SS-16 15→16; SS-sum 334→336. | LOW | **RESOLVED 2026-07-01** (ARCH-INDEX v2.10 full registry sweep; fixed in Pass-7 remediation) |
| TLS-SUMMARIZE-MAPTYPE-001 | BC-2.07.043 PC-4 HashMap vs impl BTreeMap; VP-040 Sub-D wording. | LOW | Deferred — spec-only gap |
| SEC-004 + SEC-007 | 7+ counter `+= 1` → saturating_add; clippy hygiene MQ-003/004/005. | LOW | Deferred — trivial PR candidate |
| PG-BC-ANCHOR-VALIDATION-001 | No automated anchor validation; 12 stale sites maint-2026-07-01. | LOW | Deferred — STORY-091 tooling candidate |
| DF-KANI-NONVACUITY-001-PROPTEST-GAP | No proptest/unit analog for DF-KANI-NONVACUITY-001. | LOW | Justified deferral — next Kani VP |
| DF-CANONICAL-FRAME-HOLDOUT-001-F3-OBLIGATION | **F3 MUST add canonical-value ACs for port/ethertype-asserting BCs**: BC-2.18.001/002 (GOOSE ethertype=35000, HART-IP port=5094), BC-2.12.024 (--coverage-gaps canonical output example). **AMENDED (D-325):** F3 must also add a test asserting POWERLINK EtherType == 0x88AB (34987); V1 0x3E3F intentionally excluded as obsolete. Port/ethertype values verified correct at Pass-2 (POWERLINK IEEE RA registry HIGH-confidence). Forward obligation only. Failure to add these ACs leaves concrete-value correctness untested in holdout. | HIGH | story-writer (F3 decomposition) | **OPEN — F3 prerequisite** |
| SEC-001-ENIP | Unsafe split-borrow enip.rs `on_data`. | MEDIUM | v0.12.0 candidate |
| TLS-FILLBUF-PUBLIC-SEAM-001 + MAINT-SC-001 | fill_buf_for_testing seam (W7.1); indicatif patch + 41 transitive updates; 8 stale deny.toml entries. | LOW | W7.1 backlog / optional dep-refresh |
| PG-F2-ARCHDELTA-SYNC-001 | [process-gap] Phase-delta working docs drift across adversary passes (F-F2P6-003); mitigated via historical-snapshot disclaimer on arch-delta; consider codifying a policy that phase-delta docs either stay synced or carry a snapshot disclaimer — capture at cycle-close lessons (S-7.02). | LOW | cycle-close retrospective |
| PG-F2-NARRATIVE-SWEEP-001 | [process-gap] PRD §2.18 narrative + ARCH-INDEX subsystem registry are non-BC artifacts that BC-centric remediation sweeps miss; DF-SIBLING-SWEEP / DF-CONSISTENCY-AUDIT sweeps should explicitly include PRD narrative blocks + ARCH-INDEX subsystem registry counts as sweep targets (surfaced F-F2P7-001/002/003). | LOW | cycle-close retrospective (S-7.02 lessons) |

Detail: `cycles/feature-enip-v0.11.0/decisions-archive` + `cycles/maint-2026-07-01/maintenance-log.md`.

---

## Session Resume Checkpoint

**F2 adversarial Pass-8 REMEDIATED (D-331). 1 HIGH (F-F2P8-001: coverage_gaps JSON schema contradiction — BC-2.12.023 PC-3 flat-dict vs BC-2.12.024 PC-5 {caveat_l2, entries[]} object; BC-2.12.023 reconciled to authoritative object form). BC-INDEX v2.12, PRD v1.51, ARCH-INDEX v2.10, VP-INDEX v2.31. Entering Pass-9 (0/3 consecutive clean passes). Finding trajectory: 14→8→4→3→4→4→4→1(H).**

- **Ground truth:** develop=`3a60317` (full `3a60317965e62bef9895e857c8a26fc3b8d03ad0`), main=`4e2b285` (full `4e2b28529ae196785ce6a0baed522b9939f929ea`, v0.11.1). factory-artifacts HEAD: `git -C .factory log -1 --format='%h %s'`. No open PRs. Worktrees: main checkout [develop] + .factory [factory-artifacts] only.
- **F2 design-layer artifacts (DONE — D-321/D-322):**
  - SS-18: `.factory/specs/architecture/ss-18-protocol-coverage-catalog.md` (v1.4)
  - ADR-012 (Decision 5 reframed + Decision 10): `.factory/specs/architecture/decisions/ADR-012-protocol-coverage-catalog.md`
  - VP-041 (2 harnesses: oracle_cross_check + partition_invariant), VP-042 (TCP-only), VP-043 (UDP main.rs, 2 harnesses)
  - Index: ARCH-INDEX v2.10; VP-INDEX v2.31.
- **F2 spec-layer artifacts (DONE — D-323 through D-331 Pass-8 remediated):**
  - BC-2.18.001..004 (SS-18): BC-2.18.003 v1.3, BC-2.18.004 v1.2. BC-2.05.010..011 (SS-05), BC-2.12.022..024 (SS-12), CAP-18
  - BC-2.12.023 v1.2 (Pass-8 F-F2P8-001: coverage_gaps JSON schema corrected to {caveat_l2, entries[]} object form)
  - BC-INDEX v2.12 (345 active / 346 on disk); PRD v1.51; ARCH-INDEX v2.10 (SS-05=11, SS-12=24, SS-11=35, SS-16=16; sum=345)
  - Deferred-to-F3: AMB-001-ARP-ETHERTYPE, AMB-002-JSON-FLAG-SCOPE; DF-CANONICAL-FRAME-HOLDOUT-001 forward obligation (AMENDED for POWERLINK test).
- **RESUME PROCEDURE (strictly ordered):**
  1. Run `vsdd-factory:factory-worktree-health` — PASS required before any other step.
  2. Read `.factory/STATE.md` (this file) — confirm Pass-8 REMEDIATED state.
  3. Verify git ground truth: `origin/develop=3a60317`, `origin/main=4e2b285`, no open PRs.
  4. Dispatch Pass-9 fresh-context adversary (cannot see Pass-1..Pass-8 reports).
  5. Continue adversary passes until 3 consecutive clean passes, then human F2 gate approval, then F3.

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001.

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Not on crates.io (D-300). Squash-only on develop (D-289). Branch protection (D-290/D-315).
- Cycle `fix-tls-clienthello-frag` CLOSED (D-316). maint-2026-07-01 CLOSED (D-318). Cycle `feature-protocol-coverage` STARTED (D-320). F1 DONE. F2 design-layer DONE (D-321). Blocker F2-SCOPE-DRIFT-UDP-001 RESOLVED (D-322). F2 spec-layer DONE (D-323): 9 BCs, CAP-18. F2 adversarial Pass-1 REMEDIATED (D-324): 14 findings fixed, BC-INDEX v2.5, PRD v1.47, VP-043, VP-INDEX v2.31, ARCH-INDEX v2.8. F2 adversarial Pass-2 REMEDIATED (D-325): 8 findings fixed, BC-INDEX v2.6, PRD v1.48, ARCH-INDEX v2.9. F2 adversarial Pass-3 REMEDIATED (D-326): 4 findings fixed (ARP EC-001 self-contradiction, VP-043 phantom harness, cap-18 POWERLINK, BC-INDEX citation), BC-INDEX v2.7, PRD v1.49. F2 adversarial Pass-4 REMEDIATED (D-327): 3 findings fixed (BC-2.05.011 phantom DispatchTarget::Arp/::Dns, VP-042 false UDP-via-dispatcher clause ×6 locations, coverage-matrix key type), BC-INDEX v2.8. F2 adversarial Pass-5 REMEDIATED (D-328): 6 findings fixed (SUPPORTED_PORTS reframe, BC-2.18.003 v1.2, BC-2.12.024 v1.1 transport-aware tri-state, arch-delta working doc sync, PRD RTM title), BC-INDEX v2.9, PRD v1.50. F2 adversarial Pass-6 REMEDIATED (D-329): 1H+1M+2L version-metadata hygiene; 9-BC version sweep (5 bumped); arch-delta snapshot disclaimer, BC-INDEX v2.10. F2 adversarial Pass-7 REMEDIATED (D-330): 3H+1M PRD-narrative + ARCH-INDEX registry propagation gaps; ARCH-INDEX v2.10 (SS-05=11, SS-12=24, SS-11=35, SS-16=16); PRD v1.51; BC-INDEX v2.11; ARCH-INDEX-COUNT-DRIFT-001 CLOSED. F2 adversarial Pass-8 REMEDIATED (D-331): 1H coverage_gaps JSON schema contradiction (BC-2.12.023 PC-3 flat-dict vs BC-2.12.024 PC-5 object); BC-2.12.023 v1.2; BC-INDEX v2.12. Entering Pass-9 (0/3 consecutive clean). Finding trajectory: 14→8→4→3→4→4→4→1(H).
