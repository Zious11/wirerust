# Decision Log Archive — D-302..D-436

Extracted from STATE.md on 2026-07-14 (compact-state, D-444).

Covers: feature fix-tls-clienthello-frag, maint-2026-07-01, feature-protocol-coverage (E-21),
out-of-cycle fixes, v0.11.x releases, maint-2026-07-06..07-11, wave-70..wave-75, v0.12.0, v0.12.1.

D-001..D-301 (exhaustive): see `cycles/*/decisions-archive.md` (greenfield → feature-enip-v0.11.0).

---

## fix-tls-clienthello-frag + maint-2026-07-01 (D-302..D-319)

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

---

## feature-protocol-coverage (E-21) F1–F7 (D-320..D-382)

| ID | Decision | Date |
|----|----------|------|
| D-320 | Feature cycle `feature-protocol-coverage` STARTED. F1 delta-analysis DONE. Scope gate APPROVED by human: OQ-1=ICS+core-IT curated catalog (~28-32 entries); OQ-2=CoverageGapsSummary Suricata-style; OQ-5=TCP+UDP dynamic detection this cycle (BACnet/IP udp/47808 Tier-1); OQ-3=terminal+--json; OQ-4=default --coverage-gaps flag (not auto-enabled under analyze --all). Research report: `.factory/phase-f1-delta-analysis/feature-protocol-coverage-research.md`. Entering F2. | 2026-07-01 |
| D-321 | F2 design-layer DONE (SS-18, ADR-012, VP-041/042, index bumps ARCH v2.6 / VP v2.30). Session paused for clear before F2 spec-layer. Scope-drift F2-SCOPE-DRIFT-UDP-001 logged. | 2026-07-01 |
| D-322 | F2-SCOPE-DRIFT-UDP-001 RESOLVED. ADR-012 Decision 6 corrected TCP-only→TCP+UDP (D-320 OQ-5 approved scope). SS-18 + module-decomposition.md reconciled. BACnet/IP UDP/47808 now flaggable. | 2026-07-01 |
| D-323 | F2 spec-layer authored & integrated. 9 BCs (BC-2.18.001..004, BC-2.05.010..011, BC-2.12.022..024) with (TransportProto,u16) keying + UDP counting; CAP-18; PRD v1.46; BC-INDEX v2.4. Deferred-to-F3: AMB-001-ARP-ETHERTYPE, AMB-002-JSON-FLAG-SCOPE. Next: F2 adversarial spec convergence. | 2026-07-01 |
| D-324 | F2 adversarial spec Pass-1 complete + remediated. Findings F-F2P1-001..014 all fixed. P0 F-F2P1-002: BC-2.05.010 false no-UDP-dissector premise removed + DNS-53 mis-count + UDP key min(src,dst). HIGH: GOOSE ethertype 34992→35000, ProtocolCategory L2-variant removed, ARP iff-invariant weakened. VP-043 added. BC-INDEX v2.5, PRD v1.47, VP-INDEX v2.31, ARCH-INDEX v2.8. | 2026-07-01 |
| D-325 | F2 adversarial spec Pass-2 complete + remediated. Findings F-F2P2-001..008 all fixed. HIGH F-F2P2-001: BC-2.18.003/ss-18 false VP-041 claim corrected. NEW ADR-012 Decision 10: UDP gap-classification evaluated regardless of enable_dns. POWERLINK 0x88AB/34987 externally verified. BC-INDEX v2.6, PRD v1.48, ARCH-INDEX v2.9. | 2026-07-01 |
| D-326 | F2 adversarial spec Pass-3 complete + remediated. Findings F-F2P3-001..004 all fixed. HIGH F-F2P3-001: BC-2.18.001 EC-001 ARP LinkLayer self-contradiction resolved. HIGH F-F2P3-002: phantom VP-043 harness replaced with 2 canonical harnesses. BC-INDEX v2.7, PRD v1.49. | 2026-07-01 |
| D-327 | F2 adversarial spec Pass-4 complete + remediated. Findings F-F2P4-001..003 all fixed. HIGH F-F2P4-001: BC-2.05.011 phantom DispatchTarget::Arp/::Dns corrected. HIGH F-F2P4-002: VP-042 false UDP-via-dispatcher claim deleted from 6 locations. BC-INDEX v2.8. | 2026-07-01 |
| D-328 | F2 adversarial spec Pass-5 complete + remediated. HIGH F-F2P5-001: SUPPORTED_PORTS reframed (dissection path, not just classify() mirror). MEDIUM F-F2P5-002/003: architecture-delta working doc synced. BC-INDEX v2.9, PRD v1.50, ss-18 v1.4. Finding trajectory 14→8→4→3→4. | 2026-07-02 |
| D-329 | F2 adversarial spec Pass-6 complete + remediated. Core spec CONTENT converged. Remaining findings were version-metadata hygiene. Full 9-BC version sweep reconciled 5 BCs. BC-INDEX v2.10. Finding trajectory (1H+1M+2L). | 2026-07-01 |
| D-330 | F2 adversarial spec Pass-7 complete + remediated. Systematic propagation-sweep-gap surfaced. HIGH F-F2P7-001: ARCH-INDEX SS-05 9→11, SS-12 21→24. HIGH F-F2P7-002: PRD §2.18 L2 caveat POWERLINK added. HIGH F-F2P7-003: VP-041 "two harnesses" correction. ARCH-INDEX v2.10, PRD v1.51, BC-INDEX v2.11. | 2026-07-01 |
| D-331 | F2 adversarial spec Pass-8 complete + remediated. HIGH F-F2P8-001: coverage_gaps --json schema contradiction (flat-dict vs object form) — reconciled BC-2.12.023 PC-3 to authoritative object form. BC-INDEX v2.12. Finding trajectory single HIGH. | 2026-07-01 |
| D-332 | F2 adversarial spec Pass-9 = CLEAN (zero HIGH/CRITICAL; HIGH decay 3→1→0 across Pass-7/8/9). Fixed 3 non-blocking findings for spec quality. BC-INDEX v2.13, ARCH-INDEX v2.11, VP-INDEX v2.32, ss-18 v1.5. | 2026-07-01 |
| D-333 | F2 adversarial spec Pass-10 = CLEAN (zero HIGH/CRITICAL; Pass-9 + Pass-10 both clean). Applied 2 non-blocking MEDIUM derived-doc propagation fixes. Consecutive-clean 0 (fixing after clean resets). | 2026-07-01 |
| D-334 | F2 adversarial spec Pass-11 = CLEAN. Only 2 LOW findings DEFERRED (not fixed). CONSECUTIVE-CLEAN COUNT = 1 (Pass-11). | 2026-07-01 |
| D-335 | F2 adversarial spec Pass-12 = CLEAN. Same 2 LOW re-derived — confirms spec stability. CONSECUTIVE-CLEAN COUNT = 2 (Pass-11, Pass-12). | 2026-07-01 |
| D-336 | F2 ADVERSARIAL SPEC CONVERGENCE ACHIEVED. 3 consecutive clean passes (Pass-11/12/13). Full HIGH-decay 14→8→4→3→4→4→4→1→0→0→0→0→0. Deferred F3-carry items (LOW, non-blocking). | 2026-07-01 |
| D-337 | F2 pre-gate input-hash drift resolved. 10 delivered stories STALE due to prd.md v1.45→v1.51 change. Re-baselined mechanically via bin/compute-input-hash --write --scan. Post-fix: MATCH=95 STALE=0 ERROR=3 (3 pre-existing). | 2026-07-01 |
| D-338 | F2 (spec-evolution) HUMAN GATE APPROVED (2026-07-02) → proceed to F3. Pre-gate: adversarial convergence 3 consecutive clean (13 total), consistency audit PASS, input-hash STALE=0. Human elected to CARRY 4 deferred LOW items into F3. Final F2: BC-INDEX v2.13 / PRD v1.51 / VP-INDEX v2.32 / ARCH-INDEX v2.11 / ss-18 v1.5. | 2026-07-02 |
| D-339 | F3 decomposition + holdout authoring COMPLETE (2026-07-02). 4 stories STORY-151..154 (32 pts, waves 67/68/69, E-21, diamond-acyclic). 10 holdout scenarios HS-123..132 (all must_pass, 7 canonical-value). STORY-INDEX v3.11, dependency-graph v3.2, HS-INDEX v2.7. | 2026-07-02 |
| D-340 | F3 adversarial story Pass-1: NOT-CLEAN (1 P0, 1 HIGH, 5 MEDIUM + LOW) — ALL remediated. P0 F-F3P1-001: EtherCAT/PROFINET-DCP canonical EtherType no test/holdout → added story tests + HS-124 Cases F/G. dep-graph v3.3, HS-124 v2.0, HS-INDEX v2.8. | 2026-07-02 |
| D-341 | F3 adversarial story Pass-2: NOT-CLEAN (1 CRITICAL, 2 HIGH, 2 MEDIUM + LOW) — ALL remediated. CRITICAL F-F3P2-001: STORY-153 unclassified_flows regression fix. HIGH F-F3P2-002/003: ARCH-INDEX re-anchor, 9 holdout YAML dup fix. dep-graph v3.4, STORY-INDEX v3.12, HS-INDEX v2.9. | 2026-07-02 |
| D-342 | F3 adversarial story Pass-3: NOT-CLEAN (2 HIGH, 3 MEDIUM) — ALL remediated. HIGH F-F3P3-001: STORY-154 derived supportedness fix. HIGH F-F3P3-002: HS-INDEX stale STORY-154 wave 69. dep-graph v3.5, HS-INDEX v2.10. | 2026-07-02 |
| D-343 | F3 adversarial story Pass-4: NOT-CLEAN (1 HIGH, 2 MEDIUM) — ALL remediated. HIGH F-F3P4-001: STORY-153 UDP counter library-visible seam extracted (VP-043 non-vacuity). STORY-153/154 v1.4, STORY-152 v1.2. | 2026-07-02 |
| D-344 | F3 adversarial story Pass-5 = CLEAN. 0 P0/HIGH; 2 LOW deferred. Blocking-count trajectory 2→3→2→1→0. CONSECUTIVE-CLEAN COUNT = 1 (Pass-5). | 2026-07-02 |
| D-345 | F3 adversarial story Pass-6: CLEAN by HIGH threshold but 2 MEDIUM + 3 LOW — ALL cleared. Counter RESET to 0. STORY-152 v1.3, STORY-153/154 v1.5. F-F3P5-001/002 RESOLVED. | 2026-07-02 |
| D-346 | F3 adversarial story Pass-7 = CLEAN (zero P0/HIGH/MEDIUM; novelty LOW). 3 non-blocking LOW obs. CONSECUTIVE-CLEAN COUNT = 1 (Pass-7) on fully-cleared decomposition. | 2026-07-02 |
| D-347 | F3 adversarial story Pass-8: CLEAN by HIGH threshold but 3 MEDIUM + 2 LOW (all F4-breaking, STORY-152 under-swept) — ALL cleared. Counter RESET to 0. STORY-151 v1.3, STORY-152 v1.4, STORY-153/154 v1.6. | 2026-07-02 |
| D-348 | F3 adversarial story Pass-9 = CLEAN (zero P0/HIGH; novelty LOW). 1 LOW DEFERRED (F-F3P9-001 protocols --json path-routing). CONSECUTIVE-CLEAN COUNT = 1 (Pass-9). | 2026-07-02 |
| D-349 | F3 adversarial story Pass-10 = CLEAN (zero P0/HIGH; novelty LOW). 1 MEDIUM F-F3P10-001 (NOT F4-breaking) DEFERRED as F4 test-writer directive. CONSECUTIVE-CLEAN COUNT = 2 (Pass-9, Pass-10). | 2026-07-02 |
| D-350 | F3 adversarial story Pass-11: NOT-CLEAN (1 CRITICAL + 1 LOW) — remediated. CRITICAL F-F3P11-001: STORY-153 TCP gap-counter key bare lower_port() → lower_port().min(upper_port()) (real bug — IP-first canonicalization). STORY-153/154 v1.7. Counter RESET to 0. | 2026-07-02 |
| D-351 | F3 adversarial story Pass-12 = CLEAN (zero P0/HIGH/MEDIUM; novelty LOW). Pass-11 TCP fix verified. CONSECUTIVE-CLEAN COUNT = 1 (Pass-12). | 2026-07-02 |
| D-352 | F3 adversarial story Pass-13 = CLEAN (zero P0/HIGH/MEDIUM; novelty LOW). CONSECUTIVE-CLEAN COUNT = 2 (Pass-12, Pass-13). | 2026-07-02 |
| D-353 | F3 adversarial story Pass-14: CLEAN by HIGH threshold but 1 MEDIUM F4-breaker (F-F3P14-001 STORY-154 unreachable port-502 red-gate test → CLI-reachable absent-guard + unit-only). STORY-154 v1.8. Counter RESET to 0. | 2026-07-02 |
| D-354 | F3 adversarial story Pass-15: NOT-CLEAN by mis-anchoring policy. 1 MEDIUM F-F3P15-001: STORY-151 C-25 `src/enip.rs` → `src/analyzer/enip.rs` (3 sites). HS hash cascade: HS-123/125 re-baselined. STORY-151 v1.4. Counter RESET to 0. | 2026-07-02 |
| D-355 | F3 adversarial story Pass-16 = CLEAN (zero P0/CRITICAL/HIGH/mis-anchor; 3 non-blocking re-derivations of already-deferred F4-carry items). CONSECUTIVE-CLEAN COUNT = 1 (Pass-16, final run). | 2026-07-02 |
| D-356 | F3 adversarial story Pass-17 = CLEAN (zero P0/CRITICAL/HIGH/mis-anchor; novelty LOW). CONSECUTIVE-CLEAN COUNT = 2 (Pass-16, Pass-17, final run). | 2026-07-02 |
| D-357 | F3 ADVERSARIAL STORY CONVERGENCE ACHIEVED. 3 consecutive clean passes (Pass-16/17/18). 18 total passes; caught CRITICAL TCP min-of-ports keying bug (F-F3P11-001) + wave-67 independent-compile gap + unreachable port-502 red-gate + phantom symbols + non_snake_case CI-gate misses + C-25 path mis-anchor. BC-5.39.001 SATISFIED. STORY-151..154 v1.4/1.4/1.7/1.8. | 2026-07-02 |
| D-358 | F3 pre-gate GAP-1 (epics.md drift) fixed. epics.md v2.0→v2.1: added E-21 epic section; total_bcs 328→337. STATE epics_version v1.8→v2.1. Input-hash drift check CLEAN (MATCH=99 STALE=0). | 2026-07-02 |
| D-359 | F3 (story decomposition) HUMAN GATE APPROVED (2026-07-02) → proceed to F4 delta implementation. Human elected AUTONOMOUS F4. F4 carry refinements catalogued. | 2026-07-02 |
| D-360 | F4 wave 67 in progress. STORY-151 (branch feature/story-151-protocol-catalog) + STORY-153 (branch feature/story-153-unclassified-counters) both IMPLEMENTED + GREEN in worktrees. Per-story adversarial Pass-1 CLEAN for both. | 2026-07-02 |
| D-361 | SESSION WRAP (human-requested, 2026-07-02). Pipeline PAUSED at F4 wave-67 per-story convergence. STORY-151 (pushed @550170d) + STORY-153 (pushed @6c3b3c3) both implemented+green; adversarial Pass-1 CLEAN+remediated. develop untouched (3a60317). | 2026-07-02 |
| D-362 | F4 wave-67 per-story adversarial Pass-2 BOTH CLEAN. STORY-153 MEDIUM-1 (VP-042 Sub-A range widened) remediated; STORY-151 MEDIUM-2 (AC-151-005 port-53 carve-out removed). STORY-151 convergence counter = 1/3. F-F3P12-001 RESOLVED. | 2026-07-03 |
| D-363 | F4 Wave-67 per-story adversarial CONVERGED for BOTH stories (BC-5.39.001 satisfied). STORY-151 @550170d: 3 consecutive clean. STORY-153 @ff91fd8: 3 consecutive clean. PRs created+reviewed+READY-TO-MERGE. Pipeline PAUSED at Wave-67 merge gate. | 2026-07-03 |
| D-364 | F4 Wave-67 STORY-151 + STORY-153 MERGED to develop 2026-07-03. PR #351 → 963a69a; PR #352 → b285feb. develop HEAD = b285feb. Integration gate GREEN. stories_delivered 94→96 (STORY-151 + STORY-153). | 2026-07-03 |
| D-365 | Wave-67 wave-level adversarial convergence 3/3 SATISFIED on develop b285feb. Integration verified: SUPPORTED_PORTS ↔ classify() set-equal; protocols::Transport vs dispatcher::TransportProto pure-core boundary intact. WAVE 67 GATE PASS. | 2026-07-03 |
| D-366 | F4 Wave 68 (STORY-152 protocols subcommand) IN PROGRESS. Worktree @7abd9e8; Red Gate + impl GREEN. Per-story adversarial Pass-1 NOT-CLEAN → REMEDIATED. 24 story_152 tests green. F-F3P9-001 + F-F3P13-001 RESOLVED. | 2026-07-03 |
| D-367 | STORY-152 (Wave 68) per-story adversarial convergence ACHIEVED (BC-5.39.001). 3 consecutive clean passes on worktree tip d34a05f. 25 story_152 tests green. Canonical values independently verified. STORY-152-GLOBAL-FLAG-NOOP-001 noted for phase-5. | 2026-07-03 |
| D-368 | STORY-152 PR #353 READY-TO-MERGE. Branch tip d34a05f→c4b14f7 (3 `///` doc-comment BC IDs stripped per Help-provenance-gate). CI 11/11 PASS. AI PR review APPROVE. PG-HELP-PROVENANCE-CLI-DOC-001 filed. Pipeline PAUSED at Wave-68 human merge gate. | 2026-07-03 |
| D-369 | PR #353 SQUASH-MERGED → develop 5c4437a. Wave-68 wave-level adversarial: SPLIT VERDICT (Pass-1 CLEAN / Pass-2 NOT-CLEAN-HIGH / Pass-3 CLEAN). F-W68-01 HIGH: `wirerust protocols --json=<path>` silently wrote JSON to stdout instead of file. Fix branch fix/protocols-json-output-routing @7ab197a: run_protocols routes JSON through resolve_format/write_output pipeline. All gates green. | 2026-07-03 |
| D-370 | Wave-68 wave-level adversarial convergence ACHIEVED (3 consecutive clean on develop 0e700a9, post-F-W68-01-fix). F-W68-01 CONFIRMED RESOLVED. PR #354 squash-merged → develop 0e700a9. Integration gate GREEN: 85 suites 0 failures. WAVE 68 COMPLETE / WAVE GATE PASS. stories_delivered 96→97. | 2026-07-04 |
| D-371 | STORY-154 (Wave 69, --coverage-gaps / CoverageGapsSummary) per-story adversarial convergence 3/3 ACHIEVED (BC-5.39.001). Worktree @a5f8e52. 25 tests green. Canonical values BACnet/IP UDP 47808 + TCP/102 four-protocol collision independently verified. Forward-notes APPLIED. | 2026-07-04 |
| D-372 | F4 DELTA-IMPLEMENTATION COMPLETE / WAVE-69 COMPLETE / WAVE GATE PASS. Wave-69 wave-level adversarial 3/3 on develop cad7024. STORY-154 delivered PR #355. All 4 E-21 stories merged. Integration gate GREEN (85 suites). stories_delivered 97→98. | 2026-07-04 |
| D-373 | F4 fixture prep COMPLETE. 8 independent pcap fixtures crafted in `.factory/holdout-fixtures/` from canonical specs. F4-FIXTURE-NEED-001 RESOLVED. | 2026-07-04 |
| D-374 | F4 HOLDOUT EVALUATION GATE PASS. Mean satisfaction 1.00 (gate ≥ 0.85); min must-pass 1.00 (gate ≥ 0.6); 10/10 must_pass. Zero behavioral gaps. All canonical values verified. Phase → F5-scoped-adversarial. | 2026-07-04 |
| D-375 | F5 spec-wording reconciliation COMPLETE. Reconciled 4 frozen-spec drift items: BC-2.12.022 v1.1, BC-2.12.024 v1.2, BC-2.05.010 v1.4, VP-042(d) residual removed. BC-INDEX v2.13→v2.14; VP-INDEX v2.32→v2.33. | 2026-07-04 |
| D-376 | F5 RECONCILIATION COMPLETION (2nd sweep). D-375 was incomplete; 3 residual MEDIUM drifts caught (ZERO code defects). BC-2.12.022 v1.2 phantom variant fixed; BC-2.05.010 v1.5 phantom min(src_port,dst_port) → accessor. VP-INDEX v2.33→v2.34. | 2026-07-04 |
| D-377 | F5 RECONCILIATION FINAL CLOSURE (3rd/final sweep). Exhaustive whole-tree grep confirmed ZERO live phantom. BC-INDEX v2.15→v2.16. PG-F5-RECONCILE-INCOMPLETE-001 RESOLVED. Input-hashes: MATCH=99 STALE=0 ERROR=3 (pre-existing). | 2026-07-04 |
| D-378 | F5 SCOPED-ADVERSARIAL CONVERGENCE ACHIEVED (feature-protocol-coverage / E-21, 2026-07-04). 8 fresh-context passes. CODE ZERO findings every pass; BC-completeness 9/9; canonical values re-derived every pass; SS-18 purity + VP-041/042/043 non-vacuity confirmed. All findings were spec-doc reconciliation-drift, now closed. | 2026-07-04 |
| D-379 | F6 carry findings RESOLVED. PR #356 squash-merged to develop 3727578d. 4 findings closed: STORY-153-RUNANALYZE-DOC-STALE-001, STORY-154-LOOKUP-ARP-DEADCLAUSE-001, STORY-154-ALL-COVERAGEGAPS-TEST-001, STORY-152/154-WEAK-UNKNOWN-ASSERT-001. | 2026-07-04 |
| D-380 | F6 (targeted-hardening) DONE/PASS. PR #357 squash-merged to develop 6da5456. VP-041/042/043 ALL PROVEN. 5 Kani harnesses VERIFICATION SUCCESSFUL (0/103 checks). fuzz_coverage_gap_classify 2,049,292 execs/301s/0 crashes. cargo-mutants E-21 delta kill rate 100%. develop=6da5456. | 2026-07-04 |
| D-381 | F7 DELTA-CONVERGENCE PRE-GATE COMPLETE (2026-07-05). All 5 convergence dimensions SATISFIED. Pre-gate reconciliations: P0-001 (STORY-INDEX v3.12→v3.13 status updates) + P2-001 (ARCH-INDEX stale comment removed). STORY-INDEX v3.13. Cycle AWAITING FINAL HUMAN APPROVAL GATE. | 2026-07-05 |
| D-382 | Feature cycle feature-protocol-coverage (E-21) CONVERGED/CLOSED (human gate approved 2026-07-05). v0.11.2 RELEASED: PR #358 (merge 96ef1ff); tag v0.11.2 (obj 2852165d); GitHub Release Latest. Back-merge PR #359 (squash 4a9eba3). S-7.02 SATISFIED: STORY-155 filed. STORY-INDEX v3.14. Pipeline IDLE. | 2026-07-05 |

---

## Out-of-cycle fixes + v0.11.3..v0.11.5 + maint-2026-07-06 (D-383..D-393)

| ID | Decision | Date |
|----|----------|------|
| D-383 | DF-VALIDATION-001 triage of deferred security/perf findings. SEC-005+SEC-006 CONFIRMED deduped-into-#342. Issue #342 FIX delivered: TDD regression tests (6958048), ENIP wiring fix (0d73892), DNP3 on_flow_close+summarize aggregation (e954dda). PR #362 squash-merged → develop ae931245. Issue #342 CLOSED 2026-07-06. | 2026-07-06 |
| D-384 | v0.11.3 RELEASED (2026-07-06). Smoke-test GO recorded. PR #363 (merge 6785716); tag v0.11.3 (obj 57381877). Back-merge PR #364 (squash a85c6f7). Cargo 0.11.3. Pipeline IDLE. | 2026-07-06 |
| D-385 | Silent-limit audit COMPLETE + observability counters DELIVERED (2026-07-06). 4 genuine observability gaps confirmed. Fix: 4 counters with saturating_add surfaced via summarize()/JSON/terminal. 8 BCs amended; BC-INDEX → v2.18. PR #365 squash-merged → develop cc2a87c. | 2026-07-06 |
| D-386 | v0.11.4 follow-up hardening DELIVERED + v0.11.4 RELEASED (2026-07-06). PR #366 (3 LOW items resolved). PR #367 (release/0.11.4 → main, merge f0f2136); tag v0.11.4 (obj e6ee614). Back-merge PR #368 (squash f7460b4). develop NO unreleased commits. Pipeline IDLE. | 2026-07-06 |
| D-387 | SESSION WRAP (human-requested, 2026-07-06). Pipeline PAUSED at IDLE post v0.11.4 release. No in-flight work. | 2026-07-06 |
| D-388 | Maintenance run maint-2026-07-06 STARTED. Sweeps 1-5,7,8,11 applicable; 6/9/10 N/A. Baseline: develop f7460b4, main f0f2136, v0.11.4 released. Log: `.factory/cycles/maint-2026-07-06/maintenance-log.md`. | 2026-07-06 |
| D-390 | maint-2026-07-06 FIX PHASE COMPLETE. FIX-A (docs PR #369: DOC-009+DOC-010) + FIX-B (code PR #370: PC-016/017 DNP3 counter gaps; BC-INDEX v2.19) + FIX-C (4 holdout repairs; HS-INDEX v2.12) + FIX-D (10 VP shards + module-criticality v1.6 + STORY-156 + STORY-149 wave 70 + STORY-INDEX v3.15) committed. | 2026-07-06 |
| D-391 | maint-2026-07-06 STATE-FINAL (2026-07-06). PRs #369 (e40fe8a), #370 (d3e153c), #371 (359726b) merged. develop HEAD d3e153c (3 unreleased). tech-debt-register v1.4. Pipeline PAUSED. | 2026-07-06 |
| D-392 | maint-2026-07-06 session review COMPLETE. 4 proposals adopted into .factory/maintenance-config.yaml dispatch_templates. 4 proposals appended to improvement-backlog.md. Release v0.11.5 IN PROGRESS. | 2026-07-06 |
| D-393 | v0.11.5 release chain COMPLETE. PR #373 (squash 19569ae) MERGED 2026-07-07T02:17:57Z. develop HEAD=19569ae; Cargo 0.11.5. Remote branches cleaned. Pipeline IDLE. | 2026-07-07 |

---

## Wave-70 + maint-2026-07-08 + issue triage (D-394..D-407)

| ID | Decision | Date |
|----|----------|------|
| D-394 | Wave 70 STARTED; human approved 2026-07-07. STORY-149 pre-story baseline captured: reassembly/tls.pcap slope 25.880 µs (+11.16% vs May-19 anchor 23.281 µs). AC-149-003 post-story target ≤ 24.445 µs. | 2026-07-07 |
| D-395 | STORY-149 DELIVERED (2026-07-07). PR #374 merged 116100d. stories_delivered=99. AC-149-003 PASS (23.841 µs, +2.41% vs May-19 anchor). Issue #360 CLOSED. Wave 70 integration gate PENDING. | 2026-07-07 |
| D-396 | Wave 70 CLOSED — 5-pass wave adversarial streak 3/3. PRs #374/#375/#376/#377 merged; develop=87035da. Gate all PASS/APPROVE. STORY-157 drafted (S-7.02). Pipeline IDLE. | 2026-07-07 |
| D-397 | Session resumed 2026-07-07. Human decisions: (a) next work = wave-71/v0.12.0 planning; (b) merge-authorization question folded into STORY-157. | 2026-07-07 |
| D-398 | Wave-71 scope gate APPROVED (human, 2026-07-07): STORY-150 v1.3 + STORY-156 v1.1 + STORY-157 v1.2, 13 pts, single parallel wave. Pre-gate input-hash drift: MATCH=102 STALE=0. | 2026-07-07 |
| D-399 | STORY-148 SUPERSEDED by PR #362. STORY-148-BASIS-RESOLVED-001 CLOSED. | 2026-07-07 |
| D-400 | Pre-gate input-hash drift check: MATCH=102 STALE=0. PG-HASH-HOOK-DIVERGENCE + PG-HASH-INLINE-COMMENT codified into STORY-157. | 2026-07-07 |
| D-401 | Human granted WAVE-LEVEL merge authorization for wave-71 story PRs (2026-07-08). STORY-156 DELIVERED: PR #378 squash e2c2b33 merged 2026-07-08T13:30:49Z; stories_delivered=100. | 2026-07-08 |
| D-402 | STORY-150 DELIVERED. PR #379 squash 9d0d175 merged. TLS-DRAIN-DUP-001 RESOLVED. BC-ANCHOR-DRIFT-OUTOFCYCLE-001 fully resolved. stories_delivered=101. STORY-INDEX v3.21. | 2026-07-08 |
| D-403 | STORY-157 DELIVERED. PR #380 squash 11c37b6 merged. stories_delivered=102. First PR under DF-MERGE-AUTH-CLASSIFIER-001 clause (b) with dual-outcome completion report. wave-70 process gaps fully codified. STORY-INDEX v3.22. | 2026-07-08 |
| D-404 | WAVE 71 CLOSED (human-approved 2026-07-08). Gate: all 7 steps green; wave adversary CONVERGED 3/3 (7 passes). S-7.02 satisfied via STORY-158. develop=b642c0f. Pipeline PAUSED. | 2026-07-08 |
| D-405 | Maintenance run maint-2026-07-08 STARTED (2026-07-08). Sweeps 1,2,3,4,5,7 dispatched in parallel; DTU/a11y N/A. DF-VALIDATION-001 research-agent triage dispatched for deferred backlog. | 2026-07-08 |
| D-406 | maint-2026-07-08 COMPLETE (2026-07-08). PRs #382 (624bae3), #383 (3ebd801), #384 (c4eb1f4) — strict 3/3 adversarial convergence each. STORY-158 v1.1 amended (AC-158-006). STORY-159 drafted. SEC-W71-001 CONFIRMED CWE-22. develop=c4eb1f4 (8 unreleased). | 2026-07-08 |
| D-407 | Issue-backlog triage triage-2026-07-08 COMPLETE. 10/10 issues validated (DF-VALIDATION-001, all CONFIRMED). #101 + #4 closed. #385 filed (SQLite). 6 validated-backlog items. | 2026-07-08 |

---

## Wave-72 + maint-2026-07-09 + v0.12.0 (D-408..D-423)

| ID | Decision | Date |
|----|----------|------|
| D-408 | Wave 72 story decomposition CONVERGED + HUMAN APPROVED (2026-07-09). 15 adversarial passes (P13/P14/P15 clean 3/3); ~100 findings fixed. BC-5.39.001 SATISFIED. LMR-001/002/003 codified. dep-graph v3.8 (128 edges). STORY-INDEX v3.28. | 2026-07-09 |
| D-409 | STORY-158 delivery COMPLETE through PR-open; merge HELD by human. Per-story convergence 7 passes P5/P6/P7 clean 3/3 (BC-5.39.001). PR #387 opened (branch feature/STORY-158-changelog-gate-cycle-lint, HEAD c4831bc); CI 12/12 green. | 2026-07-09 |
| D-410 | Merge hold D-409 RELEASED by human. PR #387 squash-merged to develop at 75c5ba5 (2026-07-09T16:41:36Z). STORY-158 DELIVERED. stories_delivered=103. STORY-159/160 UNBLOCKED. | 2026-07-09 |
| D-411 | STORY-159 DELIVERED (2026-07-09). PR #388 squash-merged d410b8d. Per-story adversarial CONVERGED. CI 12/12 green. stories_delivered=104. STORY-161 UNBLOCKED. | 2026-07-09 |
| D-412 | STORY-160 DELIVERED (2026-07-09). PR #389 squash-merged 704fd2e. F4 AC-160-010 spec amendment (BC-2.11.001 v1.9 + BC-INDEX v2.22). Step-4.5 convergence P1/P2/P3 CLEAN. BREAKING JSON change staged for v0.12.0. stories_delivered=105. | 2026-07-09 |
| D-413 | STORY-161 DELIVERED + WAVE-72 DELIVERY COMPLETE (2026-07-09). PR #390 squash-merged 80fbb64. VP-INDEX v2.38→v2.39 (Multi-File Proof Anchor Algorithm). proof_file_hash 48296b21 independent cross-verification. Step-4.5 CONVERGED P1/P2/P3 NITPICK_ONLY. stories_delivered=106. Wave-72 DELIVERY COMPLETE. | 2026-07-09 |
| D-414 | Wave-72 integration gate mid-gate bookkeeping. Suite PASS (2,392/0). PR #391 squash-merged 44f8c9c (action-pin-gate scan-guard + wave-72 gate fixes). code-review.md written at cycles/wave-72/wave-gate/code-review.md (AC-158-006 PG-W71-CODEREVIEW-ARTIFACT). | 2026-07-09 |
| D-415 | Wave-72 integration gate PASSED. All 8 dimensions green: suite 2,392/0; adversary CONVERGED 3/3 (P2/P3/P4); code-review APPROVE-WITH-COMMENTS; security PASS; consistency PASS; holdout PASS mean 1.00 (16/16 must-pass); demos PASS. S-7.02 SATISFIED: STORY-162 drafted. STORY-INDEX v3.32. | 2026-07-09 |
| D-416 | Wave-72 CLOSED (2026-07-09, human-approved). 4 stories + gate-fix delivered. S-7.02: STORY-162 drafted (wave-TBD, E-11, 3 pts). develop=44f8c9c (13 unreleased). Pipeline IDLE. | 2026-07-09 |
| D-417 | Dependabot PR #386 (indicatif 0.18.5→0.18.6) squash-merged to develop at 716054a (2026-07-09, per-PR explicit human instruction). CI 12/12 green (first dependabot PR through wave-72 changelog-gate). develop=716054a. | 2026-07-09 |
| D-418 | Maintenance run maint-2026-07-09 STARTED (2026-07-09, human-requested resume). Sweeps: 1 deps, 2 doc-drift, 3 patterns, 4 holdouts, 5 perf, 7 spec-coherence, 8 tech-debt-register. develop=716054a (14 unreleased). | 2026-07-09 |
| D-419 | maint-2026-07-09 gate APPROVED (human, 2026-07-09): Route A docs PR + Route B factory fixes EXECUTE. SEC-W71-001 filing APPROVED. TD-MAINT-RISK-REGISTRY-BACKFILL scheduled. stories_delivered adjudicated 106→101 (direct row count). | 2026-07-09 |
| D-420 | maint-2026-07-09 CLOSED (2026-07-10): 8 sweeps 0 HIGH/CRIT; Route A PR #393 squash-merged e3ca2bc (6 adversary passes, 3 HIGH findings fixed, 3/3 CONVERGED); Route B 27a4002 (SC-001 fixed); SEC-W71-001 FILED #392; TD-MAINT-RISK-REGISTRY-BACKFILL RESOLVED f41f517; STORY-163 drafted; STORY-INDEX v3.35 (116 stories / 719 pts). develop=e3ca2bc. | 2026-07-10 |
| D-421 | v0.12.0 release run OPENED (human-selected 2026-07-10 post-maint close). Scope: 15 unreleased commits on develop e3ca2bc. Flow: release/0.12.0 → PR to main → human-gated tag → GitHub Release → back-merge. | 2026-07-10 |
| D-422 | v0.12.0 RELEASED + CHAIN COMPLETE (2026-07-10). PR #394 merge f1e0c36; tag v0.12.0; GitHub Release Latest 4 binaries. BREAKING JSON surface change shipped (BC-2.11.036/037). Back-merge: develop FAST-FORWARDED to f1e0c36 (histories REUNIFIED). main==develop==f1e0c36. Pipeline PAUSED. | 2026-07-10 |
| D-423 | Session review session-review-2026-07-10-v0.12.0 COMPLETE (2026-07-10). 4 artifacts written. Key outcomes: PROP-V0.12.0-01 (BREAKING-change holdout-expectation pre-PR sweep), PROP-V0.12.0-02 (strict 3/3 adversarial for docs PRs), PROP-V0.12.0-03 (synchronous adversary dispatch for maintenance PRs), PAT-009 adversary stale git-ref false alarms RESOLVED-EFFECTIVE. | 2026-07-10 |

---

## Wave-73..Wave-75 + v0.12.1 (D-424..D-436)

| ID | Decision | Date |
|----|----------|------|
| D-424 | Wave-73 OPENED. STORY-162 + STORY-163 assigned to wave-73. STORY-163 input-hash drift resolved (1cd3179→e1ad659). STORY-INDEX v3.36. input-hash scan MATCH=116 STALE=0. Human wave-plan approval PENDING. | 2026-07-10 |
| D-425 | Wave-73 plan gate APPROVED (human, 2026-07-10). Delivery order STORY-162 → STORY-163 confirmed. STORY-162 delivery STARTED. | 2026-07-10 |
| D-426 | STORY-162 DELIVERED (2026-07-11): PR #395 squash-merged b5e1e155 (2026-07-11T03:16:04Z). Per-story adversarial 5 passes streak 3/3 (P3/P4/P5). VP-INDEX v2.39→v2.40. stories_delivered 101→102. | 2026-07-11 |
| D-427 | STORY-163 DELIVERED (2026-07-11): factory-artifacts-only (E-11 convention). docs-writer-dispatch-guidance.md CREATED (AC-163-001) + pr-manager-merge-auth-guidance.md AMENDED (AC-163-002). Per-story adversarial 5 passes streak 3/3 (P3/P4/P5). Input-hash: MATCH=116 STALE=0. stories_delivered 102→103. | 2026-07-11 |
| D-428 | Wave-73 CLOSED (human-approved, 2026-07-11). Gate all-green. Adversary 6-pass CONVERGED streak 3/3 (P4/P5/P6) trajectory 4→2→1→0→1(nits-refuted)→0. S-7.02 SATISFIED: STORY-164 drafted. STORY-INDEX v3.43 (117 stories/722 pts). | 2026-07-11 |
| D-429 | Maintenance run maint-2026-07-11 COMPLETE (2026-07-11). 8 sweeps 0 CRITICAL/HIGH. Route A: PR #396 squash-merged 6779be6 (human-merged, 17 findings fixed, adversary 3/3 CONVERGED). Routes B/C DEFERRED → ROUTE-BC-DEFERRED-2026-07-11. STORY-164 v1.1 AC-164-005 added. PERF-RERUN-001 OPEN. SEC-001 DEFERRED next feature wave. STORY-INDEX v3.44 (117 stories/723 pts). develop=6779be6. | 2026-07-11 |
| D-430 | Wave-74 OPENED (plan gate approved, human, 2026-07-11). Pre-gate citation validation caught F-VAL-164-001 fabricated anchor, fixed v1.2→v1.3. STORY-164 v1.3 assigned+ready (wave 74, E-11, 4 pts, 5 ACs). STORY-INDEX v3.45. | 2026-07-11 |
| D-431 | STORY-164 DELIVERED. PR #397 squash-merged d6e3be8 (2026-07-11T23:04:56Z, human-authorized). CI 12/12 (changelog-gate exercised). Per-story adversarial convergence: 8 passes, streak 3/3 (P6/P7/P8). Story spec final v1.16 @ 1a02b00. stories_delivered 103→104. | 2026-07-11 |
| D-432 | Wave-74 CLOSED (human-approved, 2026-07-12). Gate all-green: cargo 904/0; adversary 13 passes streak 3/3 (W11/W12/W13) trajectory 2→0→1→1→0→1→0→3→1→1→0→2n→1n; code-review 2 MINOR + 4 NIT DEFERRED (ROUTE-W74-DEFERRED); security PASS; holdout N/A; input-hash MATCH=118 STALE=0. S-7.02: STORY-165 drafted. develop=d6e3be8 (3 unreleased). Pipeline PAUSED. | 2026-07-12 |
| D-433 | Wave-75 OPENED (plan gate approved, human, 2026-07-13). STORY-165 v1.1 (3 pts, E-11, 4 ACs); consistency-audit 2 MINORs fixed; input-hash 23d6614; STORY-INDEX v3.52 (75 waves, 704 pts); sprint-state.yaml wave-75 entry added. | 2026-07-13 |
| D-434 | STORY-165 DELIVERED. PR #398 squash-merged fa646ed (2026-07-13, human-authorized). CI 13/13 green (bin-selftest first-ever run AC-165-001). Per-story adversarial: 9 passes streak 3/3 (P7/P8/P9). PG-W74-PRDESC-ROW-VERIFY first compliant execution. stories_delivered 104→105. | 2026-07-13 |
| D-435 | Wave-75 CLOSED (human-approved, 2026-07-13). Gate all-green: CI 13/13; adversary 7 passes streak 3/3 (W5/W6/W7) CONVERGED; code-review 0 BLOCKING/MAJOR/MINOR, 1 NIT DEFERRED (ROUTE-W74-DEFERRED); currency sweep COMPLETE MATCH=119 STALE=0; holdout N/A; input-hash MATCH=119 STALE=0. S-7.02: STORY-166 drafted. develop==fa646ed (4 unreleased). | 2026-07-13 |
| D-436 | v0.12.1 RELEASED (human-authorized 2026-07-13). PR #399 merged to main fedcea4ab17d9b3257c9903636aec0c0fd08f147; tag v0.12.1 object d687a77d911503e67a8d171c00536bd710762bba; GitHub Release Latest 4 binaries. Back-merge PR #400 squash 7b11b83; DRIFT-BACKMERGE-SQUASH-001. Branches cleaned. Release content: b5e1e15 STORY-162 + 6779be6 maint-2026-07-11 + d6e3be8 STORY-164 + fa646ed STORY-165 + version bump ec019b3. | 2026-07-13 |
