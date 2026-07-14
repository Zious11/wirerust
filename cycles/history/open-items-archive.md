# Open Items Archive — Resolved / Historical

Extracted from STATE.md on 2026-07-14 (compact-state, D-444).

Contains: resolved Blocking Issues, resolved/stale Open Items / Backlog rows from
feature-protocol-coverage era and prior maintenance cycles. All items below are RESOLVED,
DOWNGRADED, CLOSED, REFUTED, or ACCEPTED — not active tracking items.

---

## Resolved Blocking Issues

| ID | Summary | Priority | Owner | Status |
|----|---------|----------|-------|--------|
| F2-SCOPE-DRIFT-UDP-001 | ADR-012 Decision 6 corrected from TCP-only to TCP+UDP dynamic detection. All docs reconciled. (TransportProto, u16) keying consistent. | HIGH | architect | **RESOLVED 2026-07-01** |

---

## Resolved Open Items / Backlog (feature-protocol-coverage era + maintenance)

| ID | Summary | Final Status |
|----|---------|-------------|
| SEC-005 + SEC-006 | ENIP on_flow_close unwired (CWE-401+CWE-770); DNP3 flow-map no cap+on_flow_close. | **RESOLVED (D-383, PR #362 / issue #342 CLOSED 2026-07-06). STORY-148 SUPERSEDED (D-399).** |
| PERF-001/002 + BENCHMARK-GAP-001 | TLS carry-path +10.3% regression; no fragmented-handshake fixture. | **→ STORY-149 DELIVERED (PR #374, D-395, 2026-07-07). Issue #360 CLOSED.** |
| TLS-DRAIN-DUP-001 | ~220-line C2S/S2C drain-loop duplication in tls.rs. | **RESOLVED — STORY-150 DELIVERED (PR #379, D-402, 2026-07-08)** |
| BC-ANCHOR-DRIFT-OUTOFCYCLE-001 | 12 stale tls.rs anchor sites. | **FOLDED into STORY-150 v1.3 AC-150-006 (D-398, wave 71)** |
| ARCH-INDEX-COUNT-DRIFT-001 | SS-11 34→35, SS-16 15→16. | **RESOLVED 2026-07-01 (ARCH-INDEX v2.10)** |
| DF-CANONICAL-FRAME-HOLDOUT-001-F3-OBLIGATION | Canonical-value ACs + 7 canonical-value holdout scenarios (HS-124..126, HS-129..132). | **RESOLVED D-339 2026-07-02** |
| F4-FIXTURE-NEED-001 | HS-127..132 require crafted pcap fixtures at F4 eval time. | **RESOLVED (D-373, 2026-07-04) — 8 pcaps in `.factory/holdout-fixtures/`** |
| SEC-001-ENIP | Unsafe split-borrow enip.rs `on_data`. | **DOWNGRADED to LOW (D-383, 2026-07-06) — sound-as-written; tech-debt** |
| SEC-001-STORY153 | `unclassified_port_counts` ceiling ~131,072 keys; no doc-comment. | **Issue #361 filed (docs: add ceiling doc-comment). Validated LOW.** |
| SEC-004 + SEC-007 | 7+ counter `+= 1` → saturating_add cosmetic. | **DOWNGRADED to LOW cosmetic (D-383) — FALSE-POSITIVE on overflow (u64).** |
| TLS-FILLBUF-PUBLIC-SEAM-001 + MAINT-SC-001 | fill_buf_for_testing seam (W7.1); indicatif patch. | **W7.1 backlog / optional dep-refresh** |
| ARCH-INDEX-DOCMAP-COMPONENT-COUNT-001 | ARCH-INDEX Document Map '24 components' → system now has 26. | **RESOLVED (ARCH-INDEX v2.12, D-362, 2026-07-03)** |
| F-F2P13-OBS-VP042D | VP-042 sub-property (d) not mapped to dedicated harness. | **RESOLVED (D-375, 2026-07-04) — (d) dropped; VP-042 = 3 harnesses A/B/C** |
| INPUT-HASH-ERROR-STORIES-001 | STORY-001/091/121 input-hash anomalies. | **REFUTED-CLOSED (maint-2026-07-08 DF-VALIDATION-001 triage)** |
| F-F2P11-001 | BC-2.05.010 TCP-path references flow_key.src_port/dst_port. | **RESOLVED (BC-2.05.010 v1.4 lower_port().min(upper_port()), D-375)** |
| F-F2P11-002 | BC-2.05.011 EC-002 label 'Http/502' should be 'Modbus/502'. | **RESOLVED (BC-2.05.011 EC-002 label corrected in F5 sweep)** |
| F-F3P5-001 | dependency-graph.md:277 phantom ProtocolsArgs/AnalyzeArgs types. | **RESOLVED as F-F3P6-004 (dep-graph v3.6)** |
| F-F3P5-002 | STORY-154 AC-154-002 'run_analyze() wires args.coverage_gaps'. | **RESOLVED as F-F3P6-005 (STORY-153/154 v1.5)** |
| BC-STORY-ANCHOR-TBD-001 | 9 feature BCs' Story Anchor section reads 'TBD (F3 story decomposition)'. | **Resolved at F4 story delivery.** |
| F3-ADV-P7-O1 | STORY-153 AC-153-005 udp_unclassified_counts declaration scope clarification. | **Addressed at F4 implementation (data-flow forces correct placement).** |
| F-F3P9-001 | STORY-152 run_protocols stdout-only vs path-routing clarification. | **RESOLVED (STORY-152 v1.5, D-366 — AC-152-002 prose reconciled)** |
| HS-INDEX-ENIP-WAVE-DRIFT-001 | HS-INDEX ENIP feature section waves '63-68' vs dep-graph E-20 waves 58-61. | **CONFIRMED — DEFERRED Route C. Batch into next spec-coherence sweep.** |
| F-F3P10-001 | STORY-153 no Red-Gate test asserting unclassified_flows fires when coverage_gaps_enabled=false. | **Applied at F4 implementation; covered by holdouts HS-040/HS-095.** |
| VP042D-FROZEN-RESIDUAL-001 | VP-INDEX VP-042 sub-property (d) residual in frozen doc. | **RESOLVED (D-375, 2026-07-04) — (d) label dropped from VP-INDEX + ADR-012** |
| BC-2.05.010-LOWERPORT-WORDING-001 | Frozen BC-2.05.010 references non-existent flow_key.src_port/dst_port. | **RESOLVED (D-375, 2026-07-04) — BC-2.05.010 v1.4 lower_port().min(upper_port())** |
| F-F3P12-001 | STORY-151 test_BC_2_18_003_supported_ports_mirror excludes port 53 unnecessarily. | **RESOLVED (STORY-151 v1.5, D-362, 2026-07-03)** |
| F-F3P13-001 | STORY-152 AC-152-002 prose over-claims --json=path file routing. | **RESOLVED (STORY-152 v1.5, D-366 — stdout-only reconcile per frozen BC)** |
| F-F3P18-O2 | STORY-154 render path must re-lookup KNOWN_PROTOCOLS for name. | **RESOLVED (D-371, 2026-07-04 — applied at STORY-154 F4 implementation)** |
| F-F3P18-O1 | Frozen BC-2.12.024 PC-4 uses 'supported: false' field notation. | **RESOLVED-STALE — same as BC-2.12.024-PC4-PHANTOM-SUPPORTED-001, resolved D-375** |
| BC-2.05.010-EC006-UNREACHABLE-001 | BC-2.05.010 EC-006 (Tcp,502)==2 unreachable since classify() routes 502→Modbus. | **Phase-5 BC reconciliation (DF-VALIDATION-001-gated). Deferred.** |
| STORY-152-GLOBAL-FLAG-NOOP-001 | Global --csv/--output-format json silently no-op under protocols. | **RESOLVED (D-370, F-W68-01 fix, PR #354 0e700a9)** |
| BC-2.12.022-FWFIX-SYNC-001 | BC-2.12.022 v1.0 lags shipped --json=PATH file-routing behavior. | **RESOLVED (D-375 — BC-2.12.022 v1.1 synced)** |
| PG-F5-RECONCILE-INCOMPLETE-001 | Spec-reconciliation missed Invariant 2 + phantom variant-shape + BC-INDEX title-cell in first burst. | **RESOLVED (D-377 — 3rd/final sweep completed; checklist codified)** |
| PG-SPEC-FRESHNESS-ON-FIX-001 | No gate ties BC version to shipped CLI flag-matrix when a wave-level fix adds behavior. | **cycle-close retrospective — logged in feature-protocol-coverage lessons** |
| PG-HELP-PROVENANCE-CLI-DOC-001 | clap doc-comments MUST NOT contain internal factory IDs. | **Codified in implementer checklist; applied at STORY-154.** |
| STORY-154-DNS53-TCP-GAP-001 | (Tcp,53) DNS-over-TCP must be genuine gap. | **RESOLVED (D-371 — applied at STORY-154 F4 implementation)** |
| STORY-154-CAN-DECODE-HOIST-001 | Single can_decode hoist in main.rs. | **RESOLVED (D-371 — applied at STORY-154 F4 implementation)** |
| EPICS-TOTAL-BCS-DRIFT-001 | epics.md total_bcs 337 vs BC-INDEX 345 active. | **CONFIRMED — DEFERRED Route C. Batch into next spec-coherence sweep.** |
| STORY-154-ALL-COVERAGEGAPS-TEST-001 | No analyze --all --coverage-gaps combined integration test. | **RESOLVED (D-379, PR #356 commit abc048e)** |
| STORY-154-TESTCOUNT-COMMENT-001 | tests/integration_tests.rs ~line 1161 stale count 20→21 tests. | **LOW cosmetic — addressed in maint follow-up.** |
| STORY-154-WEAK-UNKNOWN-ASSERT-001 | 2 terminal tests use bare contains("unknown"). | **RESOLVED (D-379, PR #356 commit f90dfb8 — tightened to line-level checks)** |
| BC-2.12.024-PC4-PHANTOM-SUPPORTED-001 | BC-2.12.024 PC-4 references phantom `supported:` field. | **RESOLVED (D-375 — BC-2.12.024 v1.2; derived predicate applied)** |
| STORY-153-RUNANALYZE-DOC-STALE-001 | `src/main.rs` run_analyze coverage_gaps doc-comment stale. | **RESOLVED (D-379, PR #356 commit 7fbb57c)** |
| STORY-154-LOOKUP-ARP-DEADCLAUSE-001 | `lookup_protocol_state` ARP disjunct provably unreachable. | **RESOLVED (D-379, PR #356 commit 0fdaa29)** |
| STORY-148-BASIS-RESOLVED-001 | STORY-148 drafted as fix vehicle for SEC-005+SEC-006; PR #362 closed those findings. | **RESOLVED (D-399 — STORY-148 SUPERSEDED by PR #362). Row closed.** |
| DNP3-CLOSEDFLOW-REOPEN-REUSE-001 | Same 5-tuple closes/re-opens within capture → Vec lists flow twice. | **OPEN — DF-VALIDATION-001-gated (research-agent validation required)** |
| SEC-008 + SEC-009 | `closed_flow_direct_operates` Vec not cleared; `CloseReason` dropped. | **OPEN — documented acceptable; no action required.** |
| SILENT-LIMIT-GAPS-001..004 | 4 observability gaps: ARP evictions, Modbus drops, TLS+HTTP dropped_map_entries. | **RESOLVED (D-385, PR #365, develop cc2a87c, 2026-07-06)** |
| MODBUS-INVALID-ADU-LATCH-NOT-A-GAP | Modbus invalid-ADU latch proposed as silent gap. | **REJECTED (D-385 — `parse_errors` already surfaces it)** |
| HTTP-AC008-NEG-TEST-001 | Add negative regression test for dropped_map_entries. | **RESOLVED (D-386, PR #366)** |
| EVICTION-NO-FINDING-NEG-TEST-001 | Regression tests that eviction/drop emit no Finding. | **RESOLVED (D-386, PR #366)** |
| ARP-BINDINGS-EVICT-PRECHECK-COSMETIC-001 | ARP bindings-evicted pre-check duplicated. | **RESOLVED (D-386, PR #366 — insert_binding_lru returns bool; 2 call sites deduped)** |
| REBIND-COUNT-SATURATING-001 | `rebind_count` uses plain `+=` not `saturating_add`. | **RESOLVED — folded into PR #384 PF-001 sweep (c4eb1f4 2026-07-08)** |
| SEC-W71-001 | CWE-22 path traversal in `bin/compute-input-hash`. | **FILED — GitHub issue #392 (2026-07-09)** |
| SEC-W71-002 + SEC-W71-003 | Wave-71 security LOW observations. | **ACCEPTED — no issue to file** |
| CR-W71-001 | Code review MINOR + 3 NITs from wave-71 (no code-review.md written — PG-W71-CODEREVIEW-ARTIFACT). | **CLOSED-UNVERIFIABLE (maint-2026-07-08). PG codified to STORY-158 AC-158-006.** |
| STALE-INPUT-HASH-076-101 | STORY-076 + STORY-101 stale hashes due to BC-2.11.001 v1.9 cascade. | **RESOLVED (D-412 — mechanically re-baselined; MATCH=112 STALE=0)** |
| F-W72-P15-L01 | dep-graph v3.8 frontmatter totals stale. | **RESOLVED at wave-72 close.** |
| CD-03-RC-01 | STORY-INDEX release-mapping note predates wave-72 v0.12.0 targeting. | **RESOLVED — v0.12.0 released (D-422).** |
| II-02-BC-INDEX-BUMP-ASYMMETRY | BC-INDEX bump asymmetry for pre-delivery BC amendments. | **RESOLVED (D-412 — BC-INDEX v2.22 committed)** |
| SC-01-TEMPLATE-REGISTRY | Template-registry entry absent for wave-72 story template variant. | **RESOLVED — STORY-158 delivery accepted absence; advisory closed.** |
| CC-01-STORY-161-TDD-MODE | STORY-161 tdd_mode:strict on governance-only E-11 story. | **RESOLVED (D-413 — confirmed consistent with E-11 stories that include test-writing ACs)** |
| PG-GITFLOW-SQUASH-BACKMERGE | Squash back-merges sever main/develop shared history. | **MITIGATED (D-422 — fast-forward back-merge resolves divergence at v0.12.0).** |

---

## Archived Notes Section

From STATE.md Notes section (pre-compaction):

- `.factory/` is a `factory-artifacts` orphan-branch worktree, gitignored from `develop`.
- Not on crates.io (D-300). Squash-only on develop (D-289). Branch protection (D-290/D-315).
- Cycle `fix-tls-clienthello-frag` CLOSED (D-316). maint-2026-07-01 CLOSED (D-318). Cycle `feature-protocol-coverage` STARTED (D-320) / CLOSED (D-382, 2026-07-05). v0.11.2 RELEASED (PR #358/tag v0.11.2). S-7.02 satisfied (STORY-155). F2 HUMAN GATE APPROVED (D-338). F3 story decomposition COMPLETE (D-339). F3 Passes 1–18 complete: all findings catalogued above. F3 ADVERSARIAL STORY CONVERGENCE ACHIEVED (Pass-16/17/18; BC-5.39.001 SATISFIED). See `cycles/history/decision-log-archive.md` for full narrative.

---

## Active items carried forward (NOT in this archive)

The following items are still active and appear in STATE.md Active Carry-Forwards:
- SEC-001-S168 (STORY-172)
- SEC-001-S158, SEC-002-S158 (advisory, bin/lint-cycle-artifact CI wiring)
- ROUTE-BC-DEFERRED-2026-07-11, ROUTE-W74-DEFERRED, PERF-RERUN-001
- DRIFT-BACKMERGE-SQUASH-001, DRIFT-VP039-BC207038-TLS-TODO-001
- RETRANSMIT-NS-FALSEPOS-001
- STORY-166, F3-handoff cleanup items
