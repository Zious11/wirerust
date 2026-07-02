---
document_type: pipeline-state
project: wirerust
mode: feature
phase: F3-incremental-stories
status: in-progress
current_step: "F3 decomposition: consecutive-clean #1 (Pass-7 CLEAN). Decomposition HELD STABLE. Next: Pass-8 (#2), Pass-9 (#3) → convergence, then F3 consistency audit + human F3 gate."
pipeline: FEATURE-CYCLE
timestamp: 2026-07-02T00:00:00Z

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
story_index_version: "v3.12"
total_stories: 107
story_index_note: "107 stories / 69 waves / 691 pts. STORY-151..154 E-21 feature-protocol-coverage. dependency-graph v3.6 (edges 124, waves 69, acyclicity-proof counts=107). HS-INDEX v2.10 (STORY-154 wave 69 all sites; total 205)."
# Spec versions (current)
bc_index_version: "v2.13"
vp_index_version: "v2.32"
arch_index_version: "v2.11"
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

**F3 decomposition: consecutive-clean #1 (Pass-7 CLEAN, D-346, 2026-07-02). Decomposition HELD STABLE. 3 LOW non-blocking (O-1 F4-carry; O-2/O-3 no action). Next: Pass-8 (#2), Pass-9 (#3) → convergence, then F3 consistency audit + human F3 gate.**

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
| Spec versions | BC-INDEX v2.13 (345 active / 346 on disk) / VP-INDEX v2.32 (43 VPs) / ARCH-INDEX v2.11 / PRD v1.51 |
| Stories | 94 delivered / 107 total (STORY-INDEX v3.12) |

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
| Feature cycle feature-protocol-coverage — F2 (spec-evolution) | **APPROVED (D-338, 2026-07-02)** | Adversarial convergence: 3 consecutive clean passes (P11/P12/P13, 13 total). Input-hash STALE=0 (D-337). Consistency audit PASS. Human gate approved 2026-07-02. 4 LOW items carried into F3. Final spec: BC-INDEX v2.13 / PRD v1.51 / VP-INDEX v2.32 / ARCH-INDEX v2.11 / ss-18 v1.5; 9 BCs, CAP-18, ADR-012, VP-041/042/043. |
| Feature cycle feature-protocol-coverage — F3 (incremental-stories) | **IN PROGRESS** | F3 decomp+holdout DONE (D-339). Pass-1..5 remediated/clean (D-340..344, trajectory 2→3→2→1→0). Pass-5 consecutive-clean #1. Pass-6 REMEDIATED (D-345): 2 MEDIUM + 3 LOW (all cleared); STORY-152 v1.3, STORY-153/154 v1.5, dep-graph v3.6. Counter RESET to 0. Pass-7 CLEAN (D-346): 0 P0/HIGH/MEDIUM; 3 LOW non-blocking; consecutive-clean #1. |

---

## Current Phase Steps (last 5)

| Step | Status | Notes |
|------|--------|-------|
| **F3 adversarial story Pass-7 CLEAN — consecutive-clean #1 on stable decomposition** | **DONE (D-346)** | 0 P0/HIGH/MEDIUM. 3 LOW non-blocking: O-1 (STORY-153 AC-153-005 udp_unclassified_counts scope clarification → run_analyze() function scope, before outermost target loop; F4-carry), O-2 (BC-2.12.024 frozen 'supported: bool' already reconciled via derived check in STORY-152 v1.3 + STORY-154 v1.3; no action), O-3 (VP-043 module=main.rs in VP-INDEX vs udp_gap_key seam in dispatcher.rs — benign F2/F3 layering, SEAM CONTRACT documents it; no action). Decomposition HELD STABLE. Consecutive-clean #1. |
| **F3 adversarial story Pass-6 REMEDIATED (all LOW cleared) — entering Pass-7** | **DONE (D-345)** | 2 MEDIUM + 3 LOW ALL cleared (counter RESET to 0). MEDIUM F-F3P6-001: STORY-153 wave-67 independent-compile gap — `coverage_gaps: bool` scalar param introduced on run_analyze() default-false; STORY-154 flips call-site to `*coverage_gaps`. MEDIUM F-F3P6-002: STORY-152 missing DERIVED-value NOTE for `supported` column/JSON field — added to AC-152-003 + AC-152-007. LOW F-F3P6-003 (new() 5-arg shorthand), F-F3P6-004=F-F3P5-001 (dep-graph phantom ProtocolsArgs/AnalyzeArgs → inline Commands variants), F-F3P6-005=F-F3P5-002 (args.coverage_gaps → scalar param). STORY-152 v1.3, STORY-153/154 v1.5, dep-graph v3.6. |
| **F3 adversarial story Pass-5 CLEAN (#1 consecutive) — entering Pass-6** | **DONE (D-344)** | Pass-5 CLEAN: 0 P0/HIGH. 2 LOW prose-precision findings DEFERRED to F4 (decomposition held stable). F-F3P5-001: dep-graph:277 phantom ProtocolsArgs/AnalyzeArgs. F-F3P5-002: STORY-154 AC-154-002 args.coverage_gaps phantom struct ref. Consecutive-clean count = 1 (Pass-5). Blocking-count trajectory: 2→3→2→1→0. |
| **F3 adversarial story Pass-4 REMEDIATED — entering Pass-5** | **DONE (D-343)** | 1 HIGH + 2 MEDIUM. HIGH F-F3P4-001: STORY-153 UDP counter binary-private in main.rs → VP-043 vacuous; extracted `pub fn udp_gap_key()` seam in dispatcher.rs (SEAM CONTRACT; main.rs loop calls it; BC-2.05.010 satisfied). MEDIUM F-F3P4-002: STORY-154 AC-154-002 heading had forbidden new(coverage_gaps) param → `.with_coverage_gaps(...)` builder. MEDIUM F-F3P4-003: STORY-154 unit tests mis-located + duplicate names → inline `#[cfg(test)] mod story_154_unit` in main.rs + `_unit` suffix. Obs-1: STORY-152/154 VP-relevance notes added. STORY-153/154 v1.4, STORY-152 v1.2. Counter: 0 clean (trajectory 2→3→2→1). |
| **F3 adversarial story Pass-3 REMEDIATED — entering Pass-4** | **DONE (D-342)** | 2 HIGH + 3 MEDIUM. HIGH F-F3P3-001: STORY-154 AC-154-006 phantom KnownProtocol.supported field → derived check. HIGH F-F3P3-002: HS-INDEX STORY-154 wave 68→69 (6 sites + range 67-69). MEDIUM F-F3P3-003: STORY-153 UDP += 1 → saturating_add. MEDIUM F-F3P3-004: dep-graph acyclicity-proof 73/93→107 (3 locs). MEDIUM F-F3P3-005: HS-INDEX total 182→205. STORY-153/154 v1.3, dep-graph v3.5, HS-INDEX v2.10. Counter: 0 clean. |

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
| D-333 | F2 adversarial spec Pass-10 = CLEAN (zero HIGH/CRITICAL; Pass-9 + Pass-10 both clean). Pass-10 confirmed real src/dispatcher.rs on_flow_close guard is the 5-analyzer form (incl enip) matching BC-2.05.010 — no drift. Applied 2 non-blocking MEDIUM derived-doc propagation fixes for quality: F-F2P10-001 (cap-18 '5 LinkLayer'→'5 L2/multicast + ARP=6th LinkLayer entry'), F-F2P10-002 (module-decomposition.md added C-25 ENIP [pre-existing gap] + C-26 protocols.rs PLANNED; preamble 24→26; counts now agree with ARCH-INDEX). Fixing after a clean pass re-baselines consecutive-clean to 0. Next: Pass-11 (targeting 3 consecutive clean; findings now decaying to peripheral derived docs — core spec converged per Pass-9/10). | 2026-07-01 |
| D-334 | F2 adversarial spec Pass-11 = CLEAN. Only 2 LOW findings, DEFERRED (not fixed) to hold the spec stable for consecutive-clean accumulation: F-F2P11-001 (BC-2.05.010 TCP-path references flow_key.src_port/dst_port; real FlowKey exposes lower_port()/upper_port() accessors — self-correcting since min(src,dst)==lower_port() on pre-canonicalized FlowKey; fix to flow_key.lower_port() at F3), F-F2P11-002 (BC-2.05.011 EC-002 illustrative label 'Http/502' should be 'Modbus/502' — port-502 routes to DispatchTarget::Modbus; expected output unaffected). CONSECUTIVE-CLEAN COUNT = 1 (Pass-11). Rationale for deferral: passes 9/10/11 each surfaced ~2 new peripheral nits; core spec converged at Pass-9/10; continuing to fix resets the 3-consecutive requirement indefinitely. Deferred LOW items carried as F3 story-writer inputs. Next: Pass-12 (#2) on the SAME unchanged spec state; Pass-13 (#3) → convergence. | 2026-07-01 |
| D-335 | F2 adversarial spec Pass-12 = CLEAN. Independently re-derived the same 2 LOW deferred items (F-F2P12-001=F-F2P11-001 FlowKey lower_port(); F-F2P12-002=F-F2P11-002 EC-002 Modbus/502 label) — confirms spec stability, no new defects. New non-blocking observation: ARCH-INDEX Document Map (~line 147) describes module-criticality.md as 'all 24 components' but system now has 26 (C-1..C-26) — same derived-doc-lag class as PG-F2-DERIVED-DOC-SWEEP-001; folded into that process-gap. CONSECUTIVE-CLEAN COUNT = 2 (Pass-11, Pass-12). Spec held byte-stable. Next: Pass-13 (#3) → convergence. | 2026-07-01 |
| D-336 | F2 ADVERSARIAL SPEC CONVERGENCE ACHIEVED. 3 consecutive clean fresh-context passes (Pass-11/12/13), zero HIGH/CRITICAL, spec byte-stable (factory-artifacts HEAD 038bcb3 unchanged across the 3 passes). Full HIGH-decay trajectory across 13 passes: 14→8→4→3→4→4→4→1→0→0→0→0→0. Deferred F3-carry items (LOW, non-blocking): F-F2P11-001/F-F2P13-001 (BC-2.05.010 TCP path use flow_key.lower_port() not src_port/dst_port), F-F2P11-002/F-F2P13-002 (BC-2.05.011 EC-002 label Http/502→Modbus/502), ARCH-INDEX-DOCMAP-COMPONENT-COUNT-001 (Document Map '24 components'→26), and NEW F-F2P13-OBS-VP042D (VP-042 sub-property (d) 'both counters consistent' is described in ADR-012 Decision 6 Clarification + VP-INDEX but not mapped to a dedicated named harness — folded into total_count_equals_n's precondition; F3/F6 to either add a (d) assertion or drop the (d) enumeration to 3 sub-properties). Next: mandatory phase-gate fresh-context consistency-validator audit + input-hash drift check, THEN human F2 gate. | 2026-07-01 |
| D-332 | F2 adversarial spec Pass-9 = CLEAN (zero HIGH/CRITICAL; HIGH decay 3→1→0 across Pass-7/8/9). Fixed 3 non-blocking findings for spec quality rather than carry a known MEDIUM PC/Inv contradiction into convergence: F-F2P9-001 (BC-2.18.001 PC-6 port-102 footnote made conditional, consistent with Inv-3/ECs/test), F-F2P9-003 (BC-2.05.010 increment-site pinned INSIDE analyzer-present guard per new ADR-012 Decision 6 Clarification; VP-042(d) precondition added), F-F2P9-002 (ss-18 changelog wording). Fixing after a clean pass re-baselines the consecutive-clean count to 0. BC-INDEX v2.13, ARCH-INDEX v2.11, VP-INDEX v2.32, ss-18 v1.5. Next: Pass-10 (targeting 3 consecutive clean on the corrected, contradiction-free spec). | 2026-07-01 |
| D-331 | F2 adversarial spec Pass-8 complete + remediated. HIGH F-F2P8-001: coverage_gaps --json schema contradiction — BC-2.12.023 PC-3 flat-dict (`"<transport>/<port>": { count, state }`) vs BC-2.12.024 PC-5 authoritative object form (`{ "caveat_l2": <string>, "entries": [ { transport, port, count, state, name?, collision_note? } ] }`); flat-dict structurally incompatible with mandatory caveat_l2 field (BC-2.12.024 Inv-1) and per-entry collision_note (port-102 four-way collision); reconciled BC-2.12.023 PC-3 to the authoritative object form; PC-3 now references BC-2.12.024 PC-5 as field-level authority. Adversary confirmed ALL other axes clean (protocol values, counts, index/version propagation, ARCH registry, PRD narrative, VP arithmetic, non-vacuity, anchors). BC-INDEX v2.12. Finding trajectory 14→8→4→3→4→4→4→1 (single HIGH). Counter: 0 consecutive clean. Next: Pass-9 (spec appears converged pending re-confirmation). | 2026-07-01 |
| D-337 | F2 pre-gate input-hash drift resolved. 10 delivered stories (STORY-002/003/004/005/076/077/078/079/080/129) went STALE because they list prd.md as an input and F2 changed prd.md v1.45→v1.51. Re-baselined mechanically via bin/compute-input-hash --write --scan (no version bump — hash rewrite is mechanical per DF-INPUT-HASH-CANONICAL-001). Post-fix scan: MATCH=95 STALE=0 ERROR=3. 3 PRE-EXISTING ERRORs (NOT F2-caused) logged for maintenance: STORY-001 (inputs cite retired BC-2.01.004 → should re-point to superseding BC-2.01.009), STORY-091 + STORY-121 (no inputs: block). These predate feature-protocol-coverage and do NOT block F2. | 2026-07-01 |
| D-338 | F2 (spec-evolution) HUMAN GATE APPROVED (2026-07-02) → proceed to F3 story decomposition. Pre-gate satisfied: adversarial convergence (3 consecutive clean passes, 13 total), fresh-context consistency audit PASS (corpus consistently integrated), input-hash drift resolved (10 stories re-baselined, STALE=0). Human elected to CARRY the 4 deferred LOW items into F3 (not fix-now): F-F2P11/13-001 (BC-2.05.010 flow_key.lower_port()), F-F2P11/13-002 (BC-2.05.011 EC-002 Modbus/502 label), ARCH-INDEX-DOCMAP-COMPONENT-COUNT-001 (24→26), F-F2P13-OBS-VP042D (VP-042 sub-property (d) harness). Final F2 spec state: BC-INDEX v2.13 / PRD v1.51 / VP-INDEX v2.32 / ARCH-INDEX v2.11 / ss-18 v1.5; 9 BCs, CAP-18, ADR-012, VP-041/042/043. | 2026-07-02 |
| D-339 | F3 decomposition + holdout authoring COMPLETE (2026-07-02). 4 stories STORY-151..154 (32 pts, waves 67/68, E-21, diamond-acyclic): 151=SS-18 catalog (BC-2.18.003/004, VP-041), 152=protocols subcommand (BC-2.12.022+BC-2.18.001/002), 153=SS-05 dispatcher+main.rs counters (BC-2.05.010/011, VP-042/043), 154=--coverage-gaps+CoverageGapsSummary (BC-2.12.023/024). Canonical-value ACs added per DF-CANONICAL-FRAME-HOLDOUT-001; 4 deferred F3-carry LOW items folded in (ARCH-INDEX 24→26 in STORY-151; lower_port() + EC-002 label + VP-042(d)=3-subs in STORY-153). 10 holdout scenarios HS-123..132 (all must_pass, 7 canonical-value; every framing/port/ethertype BC has >=1 canonical must-pass scenario). STORY-INDEX v3.11, dependency-graph v3.2, HS-INDEX v2.7. F1's 5th 'hardening' story dropped (VP harnesses are Red-Gate tests in impl stories). Point estimate 23->32 (F2 detail). Input-hash STALE=0 post-BC-anchor-edits. Next: F3 adversarial story convergence (3 clean passes) -> human F3 gate. | 2026-07-02 |
| D-340 | F3 adversarial story Pass-1: NOT-CLEAN (1 P0, 1 HIGH, 5 MEDIUM + LOW) — ALL remediated. P0 F-F3P1-001: EtherCAT(0x88A4/34980)+PROFINET-DCP(0x8892/34962) canonical EtherType had no test/holdout — added story tests (STORY-151) + HS-124 Cases F/G (value+wrong-value guards, IEEE-RA cited); SV(0x88BA/35002) test added for symmetry (F-F3P1-006). HIGH F-F3P1-002: STORY-153 phantom udp_header → real TransportInfo::Udp in DecodedFrame::Ip arm, can_decode outside enable_dns (ADR-012 Dec-10). MEDIUM: --json Option<Option<PathBuf>> (F-F3P1-003); port-502 dataless trap→9999 neutral + annotation (F-F3P1-004); removed misplaced VP-042(d) Task 0+note from STORY-151 (F-F3P1-005); dep-graph total_stories 93→107 (F-F3P1-007). All 5 L2 EtherTypes now have symmetric story canonical tests + holdout coverage. dep-graph v3.3, HS-124 v2.0, HS-INDEX v2.8. Counter: 0 consecutive clean. Next: F3 Pass-2. | 2026-07-02 |
| D-341 | F3 adversarial story Pass-2: NOT-CLEAN (1 CRITICAL, 2 HIGH, 2 MEDIUM + LOW) — ALL remediated. CRITICAL F-F3P2-001: STORY-153 AC-153-003 had regressed unclassified_flows to be gated on coverage_gaps_enabled (would zero it on normal runs, break BC-2.05.009 + HS-040/HS-095) — fixed to match ADR-012 Decision 6 Clarification (unclassified_flows inside analyzer-guard ONLY; new per-port counter in inner coverage_gaps_enabled). HIGH F-F3P2-002: STORY-151 ARCH-INDEX F3-carry mis-anchored (Document Map already 26 since F2; real stale '24' is module-criticality.md row; C-25=enip.rs not reader.rs) — re-targeted. HIGH F-F3P2-003: 9 E-21 holdouts had duplicate input-hash keys (stale 'tbd' seed + computed) — invalid YAML; placeholder removed + hashes refreshed. MEDIUM F-F3P2-004: StreamDispatcher gains coverage_gaps via builder with_coverage_gaps() (zero call-site blast radius) not a new() param. MEDIUM F-F3P2-005: STORY-152→154 file-sequencing edge added (both edit cli.rs/main.rs/integration_tests.rs); STORY-154→wave 69. dep-graph v3.4, STORY-INDEX v3.12, HS-INDEX v2.9. Counter: 0 consecutive clean. Next: F3 Pass-3. | 2026-07-02 |
| D-342 | F3 adversarial story Pass-3: NOT-CLEAN (2 HIGH, 3 MEDIUM) — ALL remediated. HIGH F-F3P3-001: STORY-154 AC-154-006 referenced phantom KnownProtocol.supported field (struct has no such field; supportedness is DERIVED via SUPPORTED_PORTS ∩ canonical_ports ∨ name==ARP) — fixed to derived check: `Some(p) if p.canonical_ports.iter().any(|cp| SUPPORTED_PORTS.contains(cp)) \|\| p.name == "ARP"` (mirrors supported_protocols() predicate per STORY-151 AC-151-005 / BC-2.18.003); vocabulary bullets updated; NOTE block added. HIGH F-F3P3-002: HS-INDEX stale STORY-154 wave 68 (F-F3P2-005 re-wave to 69 not propagated) — fixed 6 sites + range 67-69. MEDIUM F-F3P3-003: STORY-153 UDP snippet non-compiling `+= 1` on Entry return → `let c = …; *c = c.saturating_add(1)` (TCP sibling fixed in v1.2, UDP was not). MEDIUM F-F3P3-004: dep-graph acyclicity-proof stale 73/93 node counts → 107 (3 locations: Wave Schedule intro, E-21 Cycle-Check callout, Acyclicity Proof bullet). MEDIUM F-F3P3-005: HS-INDEX closing Note total 182 → 205. STORY-153/154 v1.3, dep-graph v3.5, HS-INDEX v2.10. Counter: 0 consecutive clean (Pass-1/2/3 all NOT-CLEAN; finding blocking-count 2→3→2). Next: F3 Pass-4. | 2026-07-02 |
| D-343 | F3 adversarial story Pass-4: NOT-CLEAN (1 HIGH, 2 MEDIUM) — ALL remediated. HIGH F-F3P4-001: STORY-153 UDP counter was main.rs-local (binary-private) → VP-043 harnesses/unit tests unreachable/vacuous; fixed by extracting library-visible seam `pub fn udp_gap_key(parsed, dns_handles) -> Option<(TransportProto,u16)>` in dispatcher.rs (SEAM CONTRACT note added, mirrors VP-039/040 pattern); main.rs decode loop calls `wirerust::dispatcher::udp_gap_key()` seam — BC-2.05.010 counter-in-main.rs-loop still satisfied (DF-KANI-NONVACUITY-001). MEDIUM F-F3P4-002: STORY-154 AC-154-002 heading named forbidden `StreamDispatcher::new(coverage_gaps_enabled=true)` param form (v1.3 body/Task already correct) → corrected to `.with_coverage_gaps(...)` builder heading consistent with body. MEDIUM F-F3P4-003: STORY-154 unit tests for `lookup_protocol_state()` were in `tests/` (binary-private fn unreachable from integration tests) and had name collision with identically-named integration tests → moved to inline `#[cfg(test)] mod story_154_unit { ... }` in `src/main.rs`; 4 unit test names given `_unit` suffix (DF-AC-TEST-NAME-SYNC-001). Obs-1 resolved: STORY-152 VP-relevance note + STORY-154 VP-relevance note added clarifying VP-041/042/043 are regression/relevance references only. STORY-153/154 v1.4, STORY-152 v1.2. Counter: 0 consecutive clean (blocking-count trajectory 2→3→2→1). Next: F3 Pass-5. | 2026-07-02 |
| D-344 | F3 adversarial story Pass-5 = CLEAN. Blocking-count decayed 2→3→2→1→0. Only 2 LOW prose-precision findings, DEFERRED (not fixed) to hold the decomposition stable: F-F3P5-001 (dependency-graph.md:277 edge justification cites phantom `ProtocolsArgs`/`AnalyzeArgs` — real cli.rs uses inline Commands struct-variants; the stories themselves are correct; fix to 'inline Commands::Protocols/Analyze variant' at F4), F-F3P5-002 (STORY-154 AC-154-002 'run_analyze() wires args.coverage_gaps' — run_analyze takes scalar params; thread coverage_gaps as a new param like enable_dns + apply .with_coverage_gaps() at the single StreamDispatcher::new site main.rs:306; F4 clarification). CONSECUTIVE-CLEAN COUNT = 1 (Pass-5). Rationale for deferral: applying the F2 lesson (fixing first-clean-pass LOW findings resets the 3-consecutive requirement indefinitely); both are F4-resolvable prose-precision items. Next: Pass-6 (#2) on SAME unchanged decomposition; Pass-7 (#3) → convergence. | 2026-07-02 |
| D-345 | F3 adversarial story Pass-6: CLEAN by HIGH/CRITICAL threshold but surfaced 2 genuine MEDIUM + 3 LOW — ALL cleared in one burst to reach a truly-clean decomposition (consecutive-clean counter RESET to 0). MEDIUM F-F3P6-001: STORY-153 wave-67 independent-compile gap — coverage_gaps binding origin unspecified; fixed by STORY-153 introducing `coverage_gaps: bool` run_analyze() param default-false (STORY-154 flips source to --coverage-gaps flag). MEDIUM F-F3P6-002: STORY-152 missing 'supportedness DERIVED' note (sibling of F-F3P3-001) — added to AC-152-003/007. LOW F-F3P6-003 (new() 5-arg shorthand), F-F3P6-004=F-F3P5-001 (dep-graph phantom ProtocolsArgs/AnalyzeArgs), F-F3P6-005=F-F3P5-002 (args.coverage_gaps→scalar param) — all cleared; F-F3P5-001/002 deferred items now RESOLVED. STORY-152 v1.3, STORY-153/154 v1.5, dep-graph v3.6. Counter: 0 (decomposition now free of all known LOW; targeting 3 truly-clean consecutive passes). Next: F3 Pass-7. | 2026-07-02 |
| D-346 | F3 adversarial story Pass-7 = CLEAN (zero P0/CRITICAL/HIGH/MEDIUM; novelty LOW — converged). 3 LOW observations, all non-blocking: O-1 (STORY-153 AC-153-005 'declare udp_unclassified_counts before the packet loop' — clarify to run_analyze() function scope, before outermost target loop; data-flow already forces correct placement; F4-carry clarification), O-2 (BC-2.12.024 frozen field-notation 'supported: bool' — already reconciled in STORY-152 v1.3 + STORY-154 v1.3 via derived check; no action), O-3 (VP-043 module=main.rs in frozen VP-INDEX vs STORY-153 udp_gap_key seam in dispatcher.rs — benign F2/F3 layering, SEAM CONTRACT documents it; no action). CONSECUTIVE-CLEAN COUNT = 1 (Pass-7) on the fully-cleared decomposition (all prior LOW resolved in Pass-6). Deferring O-1 to F4 (no spec change; hold decomposition stable). Next: Pass-8 (#2), Pass-9 (#3) → convergence. | 2026-07-02 |

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
| DF-CANONICAL-FRAME-HOLDOUT-001-F3-OBLIGATION | Canonical-value ACs added in STORY-151..154; 7 canonical-value holdout scenarios authored (HS-124..126, HS-129..132). POWERLINK 0x88AB/34987 canonical assertion present. | HIGH | story-writer | **RESOLVED D-339 2026-07-02** |
| F4-FIXTURE-NEED-001 | HS-127..132 require crafted pcap fixtures at F4 eval time (fixture-builder step or pre-built fixtures); HS-132 needs public BACnet/IP corpus (Wireshark SampleCaptures + documented fallbacks). HS-123..126 are pcap-independent. | LOW | F4 evaluator | **OPEN — F4-carry** |
| SEC-001-ENIP | Unsafe split-borrow enip.rs `on_data`. | MEDIUM | v0.12.0 candidate |
| TLS-FILLBUF-PUBLIC-SEAM-001 + MAINT-SC-001 | fill_buf_for_testing seam (W7.1); indicatif patch + 41 transitive updates; 8 stale deny.toml entries. | LOW | W7.1 backlog / optional dep-refresh |
| PG-F2-ARCHDELTA-SYNC-001 | [process-gap] Phase-delta working docs drift across adversary passes (F-F2P6-003); mitigated via historical-snapshot disclaimer on arch-delta; consider codifying a policy that phase-delta docs either stay synced or carry a snapshot disclaimer — capture at cycle-close lessons (S-7.02). | LOW | cycle-close retrospective |
| PG-F2-NARRATIVE-SWEEP-001 | [process-gap] PRD §2.18 narrative + ARCH-INDEX subsystem registry are non-BC artifacts that BC-centric remediation sweeps miss; DF-SIBLING-SWEEP / DF-CONSISTENCY-AUDIT sweeps should explicitly include PRD narrative blocks + ARCH-INDEX subsystem registry counts as sweep targets (surfaced F-F2P7-001/002/003). | LOW | cycle-close retrospective (S-7.02 lessons) |
| PG-F2-DERIVED-DOC-SWEEP-001 | [process-gap] Pass-N count/label fixes to ss-18/ARCH-INDEX repeatedly not swept to derived prose docs (cap-18, module-decomposition); module-decomposition lagged component additions across feature-enip (C-25) and feature-protocol-coverage (C-26). Consider a mechanized component-inventory ↔ ARCH-INDEX-count consistency check (mirror validate-vp-consistency.sh). Surfaced Pass-10. For S-7.02 cycle-close lessons. | LOW | cycle-close retrospective (S-7.02) |
| F-F2P11-001 | BC-2.05.010 TCP-path references flow_key.src_port/dst_port; real FlowKey exposes lower_port()/upper_port() accessors — self-correcting since min(src,dst)==lower_port() on pre-canonicalized FlowKey; fix to flow_key.lower_port() at implementation time. | LOW | F3-carry (spec-layer polish deferred to implementation) |
| F-F2P11-002 | BC-2.05.011 EC-002 illustrative label 'Http/502' should be 'Modbus/502' — port-502 routes to DispatchTarget::Modbus; expected output unaffected by the label error. | LOW | F3-carry (spec-layer polish deferred to implementation) |
| ARCH-INDEX-DOCMAP-COMPONENT-COUNT-001 | ARCH-INDEX Document Map (~line 147) describes module-criticality.md as 'all 24 components' — system now has 26 (C-1..C-26; C-25 EnipAnalyzer + C-26 protocols.rs). Same derived-doc-lag class as PG-F2-DERIVED-DOC-SWEEP-001. Batch with derived-doc consistency sweep at F3. | LOW | F3-carry (folded into PG-F2-DERIVED-DOC-SWEEP-001) |
| F-F2P13-OBS-VP042D | VP-042 sub-property (d) 'both counters consistent' is described in ADR-012 Decision 6 Clarification + VP-INDEX but not mapped to a dedicated named harness — folded into total_count_equals_n's precondition; F3/F6 to either add a (d) assertion or drop the (d) enumeration to 3 sub-properties. | LOW | F3-F6-carry |
| INPUT-HASH-ERROR-STORIES-001 | STORY-001 retired-BC input reference (→BC-2.01.009); STORY-091/STORY-121 missing inputs: block. Pre-existing; validate via research-agent, fix in a maintenance sweep. | LOW | **OPEN — maintenance backlog (DF-VALIDATION-001-gated)** |
| PG-F3-HOLDOUT-HASH-DUP-001 | [process-gap, S-7.02] Holdout authoring template appends computed input-hash without removing the 'tbd' seed → duplicate YAML keys (9/10 E-21 holdout files). Surfaced F3 Pass-2 as F-F3P2-003 (HIGH). Recommend a lint (bin/compute-input-hash --scan variant or pre-commit grep) failing on >1 '^input-hash:' key or literal 'tbd' in holdout files. | LOW | cycle-close retrospective (S-7.02) |
| F-F3P5-001 | dependency-graph.md:277 edge justification cites phantom `ProtocolsArgs`/`AnalyzeArgs` types — real cli.rs uses inline Commands struct-variants (Commands::Protocols { ... } / Commands::Analyze { ... }); the stories themselves are correct; fix the edge prose to 'inline Commands::Protocols/Analyze struct-variant' at F4 implementation time. | LOW | **RESOLVED as F-F3P6-004 (Pass-6 remediation, dep-graph v3.6)** |
| F-F3P5-002 | STORY-154 AC-154-002 states 'run_analyze() wires args.coverage_gaps' — run_analyze takes scalar params, not an args struct; at implementation time thread coverage_gaps as a new scalar param like enable_dns and apply .with_coverage_gaps() at the single StreamDispatcher::new site in main.rs:306; F4 clarification. | LOW | **RESOLVED as F-F3P6-005 (Pass-6 remediation, STORY-153/154 v1.5)** |
| BC-STORY-ANCHOR-TBD-001 | The 9 feature BCs' '## Story Anchor' section still reads 'TBD (F3 story decomposition)' while their Traceability 'Stories' rows are correctly populated (STORY-15x). Back-fill the Story Anchor sections during F4 pre-merge BC re-anchor (changing frozen BC content now would cascade story input-hashes; deferred intentionally per DF-SIBLING-SWEEP-001 W11.L1). | LOW | F4 pre-merge re-anchor carry |
| F3-ADV-P7-O1 | STORY-153 AC-153-005 udp_unclassified_counts declaration scope — clarify 'run_analyze() function scope, before outermost target loop' at F4 implementation (data-flow already forces correct placement; annotation clarification only). Surfaced F3 Pass-7 as O-1; non-blocking. | LOW | F4-carry (no spec change; decomposition held stable) |

Detail: `cycles/feature-enip-v0.11.0/decisions-archive` + `cycles/maint-2026-07-01/maintenance-log.md`.

---

## Session Resume Checkpoint

**F3 adversarial story Pass-7 CLEAN (D-346, 2026-07-02): 0 P0/CRITICAL/HIGH/MEDIUM; 3 LOW non-blocking (O-1 F4-carry; O-2/O-3 no action). Consecutive-clean #1 on fully-cleared decomposition. Decomposition HELD STABLE. stories_delivered=94 (STORY-151..154 still draft). Next: Pass-8 (#2) → Pass-9 (#3) → convergence, then F3 consistency audit + human F3 gate.**

- **Ground truth:** develop=`3a60317` (full `3a60317965e62bef9895e857c8a26fc3b8d03ad0`), main=`4e2b285` (full `4e2b28529ae196785ce6a0baed522b9939f929ea`, v0.11.1). factory-artifacts HEAD: `git -C .factory log -1 --format='%h %s'`. No open PRs. Worktrees: main checkout [develop] + .factory [factory-artifacts] only.
- **F3 story artifacts (byte-stable, Pass-7 unchanged):**
  - STORY-151 (`.factory/stories/STORY-151.md`): v1.2 — unchanged
  - STORY-152 (`.factory/stories/STORY-152.md`): v1.3 — unchanged
  - STORY-153 (`.factory/stories/STORY-153.md`): v1.5 — unchanged
  - STORY-154 (`.factory/stories/STORY-154.md`): v1.5 — unchanged
  - dep-graph v3.6 (edges 124, waves 69); STORY-INDEX v3.12 unchanged
  - Holdouts: HS-123..132 unchanged; HS-INDEX v2.10 unchanged
- **F3 ADVERSARIAL CONVERGENCE PROCEDURE (strictly ordered):**
  1. Run `vsdd-factory:factory-worktree-health` — PASS required before any other step.
  2. Read `.factory/STATE.md` — confirm D-346 present, consecutive-clean count=1 (Pass-7), decomposition byte-stable.
  3. Dispatch F3 adversarial story Pass-8 (fresh-context, #2 of 3) — same decomposition (byte-stable).
  4. Dispatch Pass-9 (#3) → on convergence, present human F3 gate summary → await approval → proceed to F4 holdout eval.

---

## Governance Policy

Full policy text: `.factory/policies.yaml`. 17 active policies — critical: DF-SIBLING-SWEEP-001
v4, DF-CONVERGENCE-BEFORE-MERGE-001, DF-CANONICAL-FRAME-HOLDOUT-001.

---

## Notes

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Not on crates.io (D-300). Squash-only on develop (D-289). Branch protection (D-290/D-315).
- Cycle `fix-tls-clienthello-frag` CLOSED (D-316). maint-2026-07-01 CLOSED (D-318). Cycle `feature-protocol-coverage` STARTED (D-320). F1 DONE. F2 design-layer DONE (D-321). Blocker F2-SCOPE-DRIFT-UDP-001 RESOLVED (D-322). F2 spec-layer DONE (D-323): 9 BCs, CAP-18. F2 adversarial Passes 1–13 complete: trajectory 14→8→4→3→4→4→4→1→0→0→0→0→0; BC-INDEX v2.13; PRD v1.51; VP-INDEX v2.32; ARCH-INDEX v2.11; ss-18 v1.5. BC-5.39.001 SATISFIED. F2 HUMAN GATE APPROVED (D-338, 2026-07-02). F3 story decomposition COMPLETE (D-339, 2026-07-02). F3 Pass-1 REMEDIATED (D-340): 7 findings — EtherCAT/PROFINET/SV canonical tests, STORY-153 UDP real-symbols, dep-graph v3.3, HS-124 v2.0, HS-INDEX v2.8. **F3 Pass-2 REMEDIATED (D-341, 2026-07-02): 1 CRITICAL + 2 HIGH + 2 MEDIUM + LOW — unclassified_flows regression fix, ARCH-INDEX re-anchor, builder API, 9 holdout YAML fix, dep-graph v3.4, STORY-INDEX v3.12, HS-INDEX v2.9. F3 Pass-3 REMEDIATED (D-342, 2026-07-02): 2 HIGH + 3 MEDIUM — STORY-154 derived supportedness, STORY-153 UDP saturating_add, dep-graph v3.5, HS-INDEX v2.10. F3 Pass-4 REMEDIATED (D-343, 2026-07-02): 1 HIGH + 2 MEDIUM — STORY-153 udp_gap_key seam (VP-043 non-vacuity), STORY-154 builder heading + unit-test disambiguation (story_154_unit _unit suffix), STORY-152 v1.2 VP-relevance note. **F3 Pass-5 CLEAN (D-344, 2026-07-02): 0 P0/HIGH; 2 LOW deferred to F4. Consecutive-clean #1 (trajectory 2→3→2→1→0). Decomposition HELD STABLE. F3 Pass-6 REMEDIATED (D-345, 2026-07-02): 2 MEDIUM + 3 LOW ALL cleared. STORY-152 v1.3, STORY-153/154 v1.5, dep-graph v3.6. F-F3P5-001/002 RESOLVED. Counter RESET to 0. F3 Pass-7 CLEAN (D-346, 2026-07-02): 0 P0/HIGH/MEDIUM; 3 LOW non-blocking (O-1 F4-carry; O-2/O-3 no action). Consecutive-clean #1 on fully-cleared decomposition. Next: Pass-8 (#2).**
