---
artifact: vp-index
traces_to: .factory/specs/architecture/ARCH-INDEX.md
version: "2.9"
status: active
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified: "2026-06-09: Phase-F6 — VP-021 (timestamp-provenance-threading) locked/verified @ develop 256a490. status draft→verified, verification_lock→true. test_sufficient_count 5→6. All 21 VPs now verified; draft count 1→0. | 2026-06-09: F2 delta issue #7 — VP-022 added (Modbus MBAP parse safety; draft; Kani; P1; analyzer/modbus.rs). total 21→22, p1 7→8, kani 8→9, draft 0→1. | 2026-06-09: F2 fix (consistency BLOCKING-1 / F-MED-006) — VP-022 catalog-row Verified BCs reconciled 6→8 (added BC-2.14.005, BC-2.14.008) to match VP-022 frontmatter and the architect's canonical BC map; no VP-count change. | 2026-06-09: F7 consistency fix F1 — VP-022 locked/verified at F6 (Kani 4/4 SUCCESSFUL @ develop 68a3306); propagate lock: status draft→verified, verification_lock→true. draft count 1→0; verified count 21→22. Mirrors VP-021 lock propagation pattern. | 2026-06-10: F2 delta issue #8 — VP-023 added (DNP3 data-link parse safety and FC classification; draft; Kani; P1; analyzer/dnp3.rs). total 22→23, p1 8→9, kani 9→10, draft 0→1. 4 harnesses: verify_parse_dnp3_dl_header_safety (sub-A), verify_is_valid_dnp3_frame_gate (sub-C), verify_classify_dnp3_fc_total (sub-B), verify_compute_dnp3_frame_len (sub-D). | 2026-06-10: H-3/H-4 coherence fixes (issue #8) — VP-023 Verified-BCs scope clarified: BC-2.15.001..007 only; BC-2.15.008 and BC-2.15.009 explicitly excluded (unit-test-only, not Kani obligations). VP-023 draft→verified lifecycle note added documenting F6 lock obligation and count transition (verified 22→23, draft 1→0) mirroring VP-021/VP-022 pattern. No VP counts changed. | 2026-06-12: Phase-F6 — VP-023 (DNP3 data-link frame parse safety and FC classification) locked/verified @ develop e685664. status draft→verified, verification_lock→true. verified count 22→23, draft count 1→0. | 2026-06-12: F2 delta ARP security analyzer — VP-024 added (ARP frame parse safety and binding-table invariant; draft; Kani primary + proptest Sub-C; P1; src/analyzer/arp.rs + src/decoder.rs). total 23→24, p1 9→10, kani 10→11, draft 0→1. | 2026-06-13: Corpus-wide consistency audit remediation (VP-1): VP-023 lifecycle note qualified — 'Total VP count (23), Kani count (10), P1 count (9)' were pre-VP-024 values at time of VP-023 lock; updated to reflect VP-024 addition: total 23→24, Kani 10→11, P1 9→10. Version bump 2.0→2.1. | 2026-06-13: Pass-15 A-01 reconciliation — VP-024 Verified-BCs catalog-row corrected to BC-2.16.001, .002, .003, .005, .006 (5 BCs; .004 excluded); BC-2.16.007 removed from formal scope per vp-024-arp-parse-safety.md v1.1 (F-A04) which is authoritative source of truth. Footnote [^vp024-bc-scope] updated to clarify BC-2.16.007 is satisfied by unit test (STORY-113), not Kani, and is NOT a VP-024 Kani-verified BC. No VP counts changed. Version bump 2.1→2.2. | 2026-06-19: F2 pcapng reader remediation (ADR-009 rev 4) — VP-025 through VP-030 added (pcapng framing BCs; draft; SS-01 reader.rs). Resolves C-3/DF-CANONICAL-FRAME-HOLDOUT-001. VP-025 Kani (timestamp totality, BC-2.01.014); VP-026 Kani (SHB parse safety, BC-2.01.010); VP-027 Kani (EPB parse safety, BC-2.01.012); VP-028 cargo-fuzz (pcapng reader no-panic, BC-2.01.017); VP-029 proptest (block-walk skip correctness, BC-2.01.015); VP-030 proptest (multi-IDB agreement totality, BC-2.01.018). total 24→30, p1 10→16, kani 11→14, fuzz 1→2, proptest 7→9, draft 0→6. Version bump 2.2→2.3. | 2026-06-19: Pass-2 adversarial remediation (ADR-009 rev 5) — I-1: VP-025/026/027 module column re-anchored from 'reader.rs' to 'reader.rs (pcapng_pure_core fns)' — Kani targets pure-core sub-functions (pcapng_timestamp_to_secs_usecs, pure SHB-body decode, pure EPB field decode), NOT the effectful from_pcap_reader<R: Read> entry point. VP-028/029/030 module anchor unchanged (proptest/fuzz correctly target the integration layer). I-2: footnote [^vp025-027-module-anchor] added documenting VP-025 Kani unwind-bound requirement (Option A: precomputed lookup table preferred; Option B: #[kani::unwind(128)]); must be resolved before STORY-125 F3 decomposition. No VP counts changed. Version bump 2.3→2.4. | 2026-06-19: Pass-3 adversarial remediation (ADR-009 rev 6 / Decision 18 / M-2) — VP-031 added (SPB captured-len computation correctness; proptest; P1; reader.rs (pcapng_pure_core fns); draft; BC-2.01.013). Fills SPB framing VP gap per DF-CANONICAL-FRAME-HOLDOUT-001: cargo-fuzz VP-028 covers no-panic but cannot express arithmetic relationship between original_len, snaplen, and returned slice length. VP-031 provides the missing arithmetic correctness property. total 30→31, p1 16→17, proptest 9→10, draft 6→7. Version bump 2.4→2.5. | 2026-06-19: Pass-4 adversarial remediation (ADR-009 rev 7 / H-3) — VP-030 RESTATED: domain narrowed from 'any sequence of IDB linktype u16 values' to 'WHITELISTED DataLink values only' (non-whitelisted values short-circuit to E-INP-001 before the conflict check is ever reached; the original domain included unreachable sequences). Comparison unit pinned to DataLink (not raw u16). Property restated: all-equal whitelisted DataLink → Ok; first-differing whitelisted DataLink → Err(E-INP-011) on that IDB; non-whitelisted → E-INP-001 (out of VP-030 scope). No VP counts changed (31 total; proptest 10; draft 7). Version bump 2.5→2.6. | 2026-06-20: Pass-5 adversarial remediation (ADR-009 rev 8) — VP property updates only; no count changes (total 31, Kani 14, proptest 10, fuzz 2, integration-unit 5 unchanged). VP-025: property amended to require explicit ts_sec saturation guard (`.min(u32::MAX)`) and a large-ts_high Kani vector where ticks/ticks_per_sec > u32::MAX (M-3 / BC-2.01.014 µs fast path). VP-027: property amended to explicitly classify EPB padding-overrun and bound-by-body checks as Err(E-INP-008) NOT E-INP-010 (C-1 / Decision 20 clarification — these are wirerust body-decode failures after crate framing, not crate framing failures). VP-031: property domain narrowed — snaplen parameter DROPPED from the captured_len formula; formula is now min(original_len, body.len() as u32) per Decision 9 rev 8 amendment (H-3 + M-2 SPB snaplen asymmetry fix; matches EPB which also ignores snaplen). No new VPs. Version bump 2.6→2.7. | 2026-06-20: Pass-6 adversarial remediation (ADR-009 rev 9) — VP property updates only; no count changes (total 31, Kani 14, proptest 10, fuzz 2, integration-unit 5 unchanged). VP-031: formula CORRECTED from min(original_len, body.len() as u32) to min(original_len, body.len() as u32 - 4) per Decision 22 (canonical spb_data_available definition; rev 8 formula failed to subtract the 4-byte original_len header from the body; F-H2/F-H3). VP-027: property extended to assert error DISCRIMINANT for interface_id checks — empty table → Err(E-INP-009); OOB on non-empty table → Err(E-INP-010); slash notation '(→ E-INP-009 / E-INP-010)' declared ambiguous and REPLACED with two explicit cases (Decision 22 / F-H4). No new VPs. Version bump 2.7→2.8. | 2026-06-21: F5 fix (F-F5P1-001) — VP-027 harness converted from tautological stub to genuine non-vacuous proof (PR #287, develop=97c66b0). Pure `decode_epb_body` extracted from the EPB arm (src/reader.rs); canonical harness `reader::kani_proofs::vp027_epb_parse_safety` reports cargo kani VERIFICATION SUCCESSFUL (687 checks) with confirmed non-vacuity flip. For BMC tractability the proof uses an `EpbDecodeError` discriminant twin (`decode_epb_body_discriminant`) that mirrors the production decode path line-by-line; twin verified FAITHFUL in PR review. VP-027 status draft→active (real proof backs it; not yet locked pending F6 lock gate). No VP count change (already counted as Kani VP-027). NOTE: twin-drift risk recorded as SEC-001 — a `#[cfg(test)]` equivalence smoke test is a tracked follow-up obligation to detect divergence between the twin and the production `decode_epb_body`. Version bump 2.8→2.9."
total_vps: 31
p0_count: 8
p1_count: 17
test_sufficient_count: 6
kani_count: 14
proptest_count: 10
fuzz_count: 2
integration_unit_count: 5
---

# wirerust Verification Properties Index

> **Source of truth:** This file is the authoritative catalog of all VP-NNN
> verification properties. Any change to a VP (addition, retirement, module
> reassignment, tool change, phase reassignment, count change) MUST propagate
> in the same burst to:
> - `.factory/specs/architecture/verification-architecture.md` (Provable Properties
>   Catalog tables, P0/P1 enumeration lists, summary counts)
> - `.factory/specs/architecture/verification-coverage-matrix.md` (VP-to-Module
>   table rows, per-module counts, Totals row)

## Summary

| Total VPs | P0 | P1 | Test-Sufficient |
|-----------|----|----|-----------------|
| 31 | 8 | 17 | 6 |

| Tool | Count | VP IDs |
|------|-------|--------|
| Kani | 14 | VP-001, VP-002, VP-003, VP-004, VP-005, VP-007, VP-009, VP-015, VP-022, VP-023, VP-024, VP-025, VP-026, VP-027 |
| proptest | 10 | VP-006, VP-010, VP-011, VP-012, VP-013, VP-014, VP-021, VP-029, VP-030, VP-031 |
| cargo-fuzz | 2 | VP-008, VP-028 |
| integration/unit | 5 | VP-016, VP-017, VP-018, VP-019, VP-020 |

> VP-005 uses Kani as its primary and sole counted tool. VP-021 uses integration +
> proptest; counted under proptest. VP-022 uses Kani only. VP-023 uses Kani only.
> VP-024 uses Kani (primary, counted) + proptest (Sub-C); counted under Kani per
> convention. VP-025, VP-026, VP-027 use Kani only. VP-028 uses cargo-fuzz only.
> VP-029, VP-030, and VP-031 use proptest only. Each VP is counted exactly once.
> Totals: 14+10+2+5 = 31.

## Complete VP Catalog

| VP-ID | Title | Module | Tool | Phase | Status | Verified BCs |
|-------|-------|--------|------|-------|--------|--------------|
| VP-001 | FlowKey Canonical Ordering | reassembly/flow.rs | Kani | P0 | verified | BC-2.04.003, BC-2.04.053 |
| VP-002 | First-Wins Overlap Policy | reassembly/segment.rs | Kani | P0 | verified | BC-2.04.018, BC-2.04.035, BC-2.04.036, BC-2.04.037, BC-2.04.038, BC-2.04.043 |
| VP-003 | MAX_FINDINGS Cap with Finalize Bypass | reassembly/mod.rs | Kani | P0 | verified | BC-2.04.024, BC-2.04.054 |
| VP-004 | Content-First Dispatch Precedence | dispatcher.rs | Kani | P0 | verified | BC-2.05.001, BC-2.05.002, BC-2.05.003, BC-2.05.004, BC-2.05.005, BC-2.05.006 |
| VP-005 | SNI 4-Way Ordered Classification | analyzer/tls.rs | Kani | P0 | verified | BC-2.07.013, BC-2.07.014, BC-2.07.015, BC-2.07.016, BC-2.07.017, BC-2.07.019, BC-2.07.037 |
| VP-006 | HTTP Poison Monotonicity | analyzer/http.rs | proptest | P1 | verified | BC-2.06.015, BC-2.06.016, BC-2.06.017 |
| VP-007 | MITRE Technique ID Format and Catalog Completeness | mitre.rs | Kani | P0 | verified | BC-2.10.005, BC-2.10.006, BC-2.10.007, BC-2.10.008 |
| VP-008 | decode_packet Never Panics on Arbitrary Input | decoder.rs | cargo-fuzz | P0 | verified | BC-2.02.007, BC-2.02.008, BC-2.02.009 |
| VP-009 | FlowState Machine Validity | reassembly/flow.rs | Kani | P0 | verified | BC-2.04.004, BC-2.04.005, BC-2.04.050, BC-2.04.051, BC-2.04.052 |
| VP-010 | buffered_bytes Invariant | reassembly/segment.rs | proptest | P1 | verified | BC-2.04.047, BC-2.04.030 |
| VP-011 | flush_contiguous Monotonicity | reassembly/segment.rs | proptest | P1 | verified | BC-2.04.034, BC-2.04.007, BC-2.04.008 |
| VP-012 | escape_for_terminal Correctness | reporter/terminal.rs | proptest | P1 | verified | BC-2.11.007, BC-2.11.008, BC-2.11.009, BC-2.11.010, BC-2.11.011, BC-2.11.012 |
| VP-013 | JA3 GREASE Filter Correctness | analyzer/tls.rs | proptest | P1 | verified | BC-2.07.006, BC-2.07.007, BC-2.07.008 |
| VP-014 | HttpAnalyzer Cross-Flow Isolation | analyzer/http.rs | proptest | P1 | verified | BC-2.06.021, BC-2.06.019 |
| VP-015 | TCP Sequence Number Wraparound | reassembly/segment.rs | Kani | P1 | verified | BC-2.04.039 |
| VP-016 | MITRE Tactic Grouping Order | reporter/terminal.rs | integration | test-sufficient | verified | BC-2.11.013, BC-2.11.014, BC-2.11.015, BC-2.10.003, BC-2.10.004 |
| VP-017 | JsonReporter Key-Order Determinism | reporter/json.rs | integration | test-sufficient | verified | BC-2.11.001, BC-2.11.003 |
| VP-018 | CLI Reassemble / No-Reassemble Mutual Exclusion | cli.rs | integration | test-sufficient | verified | BC-2.12.007, BC-2.12.009 |
| VP-019 | DNS Analyzer Is Statistics-Only (Never Emits Findings) | analyzer/dns.rs | unit | test-sufficient | verified | BC-2.08.001, BC-2.08.002, BC-2.08.003, BC-2.08.004 |
| VP-020 | CSV Injection Neutralization | reporter/csv.rs | unit | test-sufficient | verified | BC-2.11.021 |
| VP-021 | Timestamp Provenance Threading | reassembly/mod.rs | integration+proptest | test-sufficient | verified | BC-2.09.007, BC-2.04.055 |
| VP-022 | Modbus MBAP Parse Safety and Function-Code Boundary Classification | analyzer/modbus.rs | Kani | P1 | verified | BC-2.14.001, BC-2.14.002, BC-2.14.003, BC-2.14.004, BC-2.14.005, BC-2.14.006, BC-2.14.007, BC-2.14.008 |
| VP-023 | DNP3 Data-Link Frame Parse Safety and Function-Code Classification | analyzer/dnp3.rs | Kani | P1 | verified | BC-2.15.001, BC-2.15.002, BC-2.15.003, BC-2.15.004, BC-2.15.005, BC-2.15.006, BC-2.15.007 [^vp023-bc-scope] |
| VP-024 | ARP Frame Parse Safety and Binding-Table Invariant | analyzer/arp.rs + decoder.rs | Kani | P1 | verified | BC-2.16.001, BC-2.16.002, BC-2.16.003, BC-2.16.005, BC-2.16.006 [^vp024-bc-scope] |
| VP-025 | pcapng Timestamp Conversion Totality (saturation-locked) | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | draft | BC-2.01.014 |
| VP-026 | pcapng SHB Parse Safety and Byte-Order Detection | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | draft | BC-2.01.010 |
| VP-027 | pcapng EPB Parse Safety, interface_id Discriminant, and Padding-Overrun Classification | reader.rs (pcapng_pure_core fns) [b] | Kani | P1 | active [c] | BC-2.01.012 |
| VP-028 | pcapng Reader No-Panic (Full Path Fuzz) | reader.rs | cargo-fuzz | P1 | draft | BC-2.01.017 |
| VP-029 | pcapng Block-Walk Skip Correctness and Forward Progress | reader.rs | proptest | P1 | draft | BC-2.01.015 |
| VP-030 | pcapng Multi-IDB Linktype Agreement Totality (WHITELISTED domain) | reader.rs | proptest | P1 | draft | BC-2.01.018 |
| VP-031 | pcapng SPB Captured-Len Computation Correctness (body.len()-4 formula) | reader.rs (pcapng_pure_core fns) [b] | proptest | P1 | draft | BC-2.01.013 |

## P0 Properties (required before Phase 5 gate)

- VP-001: FlowKey canonical ordering (INV-1)
- VP-002: First-wins overlap policy (INV-3)
- VP-003: MAX_FINDINGS cap with finalize bypass (INV-6)
- VP-004: Content-first dispatch precedence (INV-2)
- VP-005: SNI 4-way ordered classification (INV-5)
- VP-007: MITRE technique ID format completeness (INV-9)
- VP-008: decode_packet no-panic property
- VP-009: FlowState machine validity

## P1 Properties (required before Phase 6 hardening)

- VP-006: HTTP poison monotonicity (INV-8)
- VP-010: buffered_bytes invariant
- VP-011: flush_contiguous monotonicity
- VP-012: escape_for_terminal correctness (ADR 0003)
- VP-013: JA3 GREASE filter
- VP-014: HttpAnalyzer cross-flow isolation
- VP-015: TCP sequence wraparound
- VP-022: Modbus MBAP parse safety and function-code boundary classification [NEW — SS-14]
- VP-023: DNP3 data-link frame parse safety and function-code classification [NEW — SS-15]
- VP-024: ARP frame parse safety and binding-table invariant [NEW — SS-16]
- VP-025: pcapng timestamp conversion totality — no panic, ts_usecs in [0,999999], ts_sec saturated (.min(u32::MAX)), for all (u32, u32, u8); MUST include large-ts_high Kani vector (ticks/ticks_per_sec > u32::MAX) to lock saturation [NEW — SS-01 pcapng, ADR-009 rev 4; amended rev 8 / M-3]
- VP-026: pcapng SHB parse safety and byte-order detection [NEW — SS-01 pcapng, ADR-009 rev 4]
- VP-027: pcapng EPB parse safety, interface_id discriminant (empty table → E-INP-009; OOB non-empty table → E-INP-010; two distinct cases, not slash notation), guard-before-allocate, and padding-overrun/bound-by-body → Err(E-INP-008) [NEW — SS-01 pcapng, ADR-009 rev 4; amended rev 8 / C-1; discriminant split rev 9 / Decision 22 / F-H4; PROOF FIXED — F-F5P1-001, PR #287 @ develop 97c66b0]. Harness: `reader::kani_proofs::vp027_epb_parse_safety`. Verification approach: pure `decode_epb_body` (extracted from EPB arm, src/reader.rs) called with symbolic body + interface table; BMC tractability via `EpbDecodeError` discriminant twin `decode_epb_body_discriminant` (twin FAITHFUL line-by-line per PR review). 687 checks, VERIFICATION SUCCESSFUL, non-vacuity confirmed via deliberate-flip negative test. NOTE (SEC-001 twin-drift risk): a `#[cfg(test)]` equivalence smoke test asserting `decode_epb_body` and `decode_epb_body_discriminant` agree on all error discriminants for a representative input set is a TRACKED FOLLOW-UP obligation; until present, divergence between twin and production path is detectable only by re-running `cargo kani`.
- VP-028: pcapng reader no-panic (cargo-fuzz, F6 hardening deliverable) [NEW — SS-01 pcapng, ADR-009 rev 4]
- VP-029: pcapng block-walk skip correctness and forward progress [NEW — SS-01 pcapng, ADR-009 rev 4]
- VP-030: pcapng multi-IDB linktype agreement totality — RESTATED (ADR-009 rev 7 / H-3): domain = WHITELISTED DataLink values only; non-whitelisted → E-INP-001 (out of VP-030 scope); comparison unit = DataLink not raw u16 [NEW — SS-01 pcapng, ADR-009 rev 4; restated rev 7]
- VP-031: pcapng SPB captured-len computation correctness — proptest arithmetic invariant for min(original_len, body.len() as u32 - 4) = min(original_len, spb_data_available); formula CORRECTED from rev 8 (body.len() → body.len()-4 per Decision 22; rev 8 formula failed to subtract the 4-byte original_len header); snaplen DROPPED (Decision 9 rev 8); fills SPB framing VP gap per DF-CANONICAL-FRAME-HOLDOUT-001 [NEW — SS-01 pcapng, ADR-009 rev 6; amended rev 8 / Decision 9; formula corrected rev 9 / Decision 22 / F-H2 / F-H3]

## Test-Sufficient Properties (VP-016..VP-021)

These six properties are verified by standard Rust integration or unit tests.
No standalone formal proof harness (Kani) is required; VP-021 additionally uses proptest.

| VP-ID | Verification method |
|-------|-------------------|
| VP-016 | Integration test: fixed finding sets; tactic order assertion |
| VP-017 | Integration test: determinism round-trip; C0 escape check |
| VP-018 | CLI test (assert_cmd): mutual exclusion exit code |
| VP-019 | Unit test: empty Vec<Finding> assertion for all DNS packets |
| VP-020 | Unit test: injection character prefix check in CSV output |
| VP-021 | Integration test (end-to-end hot-path + close-flush + segment-limit-None) + proptest (all-u32 timestamp range + cross-flow isolation) — tests/timestamp_threading_tests.rs |

[^vp024-bc-scope]: VP-024 formal (Kani/proptest) Verified-BCs are BC-2.16.001, BC-2.16.002,
BC-2.16.003, BC-2.16.005, and BC-2.16.006 only. BC-2.16.004 (D1 ARP spoof / rebind escalation)
is NOT a VP-024 Verified BC — it is primary-owned by STORY-114 (wave 43), which runs after
STORY-113. BC-2.16.007 (D12 L2/L3 sender mismatch detection) is verified by unit test in
STORY-113 (stateless single-packet comparison) and is NOT a VP-024 Kani-verified BC. Both
BC-2.16.004 and BC-2.16.007 were removed from the bcs: frontmatter array in
vp-024-arp-parse-safety.md v1.1 (F-A04). VP-024's Kani scope is Sub-A (BC-2.16.001,
BC-2.16.002), Sub-B (BC-2.16.003), Sub-D (BC-2.16.006). Sub-C (proptest
test_binding_table_last_write_wins) has PRIMARY anchor BC-2.16.005 (binding-table last-write-wins
semantics, implemented in STORY-113). BC-2.16.004 (D1 ARP spoof detection) is INDIRECTLY
supported by Sub-C: the last-write-wins property (BC-2.16.005) is the substrate that the
spoof-detection rebind escalation (BC-2.16.004, primary STORY-114) depends upon. Sub-C
discharges BC-2.16.005 directly and supports BC-2.16.004 indirectly; BC-2.16.004 is not
in VP-024's formal BC scope.

[^vp023-bc-scope]: VP-023 Verified-BCs are intentionally scoped to BC-2.15.001..007 only.
BC-2.15.008 (FIR=1 gating / single-fragment short-circuit) and BC-2.15.009 (desync
bail-out / reject-until-SYN) are unit-test-only obligations — they exercise stateful
runtime behaviour that is not amenable to bounded Kani model-checking. These two BCs
are correctly excluded from VP-023 and carry no Kani harness obligation.

## VP-023 Lifecycle Note (draft → verified at F6) — COMPLETED 2026-06-12

VP-023 transitioned from `status: draft` to `verified` at Phase F6 hardening. All
four Kani harnesses ran green at develop@e685664 (lock commit aa469bd on
factory-artifacts), mirroring the VP-021 and VP-022 lock propagation pattern:

- `verify_parse_dnp3_dl_header_safety` (sub-A)
- `verify_classify_dnp3_fc_total` (sub-B)
- `verify_is_valid_dnp3_frame_gate` (sub-C)
- `verify_compute_dnp3_frame_len` (sub-D)

The Consistency Invariants block counts shifted from "verified 22 / draft 1" to
"verified 23 / draft 0". Total VP count was 23 at the time of VP-023 lock — now 24
after VP-024 addition (2026-06-12); Kani count 10→11; P1 count 9→10. The lock
itself did not change totals; the subsequent VP-024 addition did.

## Consistency Invariants (machine-enforced by validate-vp-consistency.sh)

- VP-INDEX total (31) must equal verification-architecture.md row count (31)
- VP-INDEX total (31) must equal verification-coverage-matrix.md VP row count (31)
- verification-coverage-matrix.md Totals row: Kani(14) + proptest(10) + fuzz(2) + integration/unit(5) = 31
- P0 count (8) + P1 count (17) + test-sufficient (6) = 31; draft count 6 (VP-025, VP-026, VP-028..031); active 1 (VP-027); verified 24

> Note: VP-025, VP-026, and VP-028..VP-031 are status=draft pending BC revisions by the
> PO (ADR-009 rev 4/5/6/7/8/9 PO BC-Change Dispatch) and F3 story decomposition. They
> will transition to verified at F6 hardening per the standard lifecycle (VP-022/023/024
> pattern). VP-027 is status=active: the tautological stub harness was replaced with a
> genuine non-vacuous proof of `decode_epb_body` (F-F5P1-001, PR #287 @ develop 97c66b0);
> VP-027 transitions to verified at the F6 lock gate per the standard lifecycle.
> VP-030 was restated in ADR-009 rev 7 (H-3): domain narrowed to WHITELISTED DataLink
> values only; tool/phase/status/counts unchanged.
> Rev 8 property amendments (no count change): VP-025 saturation vector added (M-3);
> VP-027 padding-overrun/bound-by-body → E-INP-008 explicit (C-1); VP-031 snaplen
> dropped from domain, formula is now min(original_len, body.len() as u32) (Decision 9
> rev 8 / H-3 + M-2).
> Rev 9 property amendments (no count change): VP-031 formula CORRECTED —
> min(original_len, body.len() as u32 - 4) replaces min(original_len, body.len() as u32);
> rev 8 formula failed to subtract the 4-byte original_len header; canonical symbol
> spb_data_available = body.len() - 4 (Decision 22 / F-H2 / F-H3). VP-027 property
> extended to assert error discriminant — empty table → E-INP-009; OOB non-empty →
> E-INP-010; slash notation removed (Decision 22 / F-H4).
> F5 fix (2026-06-21, no count change): VP-027 harness rewritten from tautological stub
> to real decode_epb_body call with symbolic inputs; status draft→active; discriminant-twin
> verification approach via decode_epb_body_discriminant (SEC-001 twin-drift follow-up
> tracked). See modified: history entry for F-F5P1-001.

[^vp025-027-module-anchor]: **VP-025 / VP-026 / VP-027 module anchor clarification (I-1 resolution,
ADR-009 rev 5).** The Kani target for these three VPs is NOT `from_pcap_reader<R: Read>`
(effectful: I/O, generic Read impl). Kani operates only on pure-core functions (no I/O,
no global state). The correct anchor is the **pure-core helper functions** extracted from
`reader.rs` or colocated as `#[cfg(kani)]`-only targets within it:
- VP-025 → `pcapng_timestamp_to_secs_usecs(u32, u32, u8) -> (u32, u32)` (pure arithmetic)
- VP-026 → pure SHB body-decode function (takes `&[u8]` body slice, returns parse result)
- VP-027 → pure EPB fixed-field-decode function (takes `&[u8]`, interface table size;
  returns parsed fields or Err)
All three are deterministic, take only scalar/slice inputs, and perform no I/O. They are
the correct Kani harness targets per BC-2.01.014 §Purity Classification. The module
label `reader.rs (pcapng_pure_core fns)` means these harness targets live in the
`src/reader.rs` compilation unit but are pure-core sub-functions, NOT the top-level
effectful `from_pcap_reader` entry point. VP-028 (cargo-fuzz) correctly targets
`from_pcap_reader` — the effectful entry point — which is appropriate for fuzzing but
not for Kani. VP-029 and VP-030 (proptest) target pure predicate/aggregation logic
extracted from the block-walk and multi-IDB policy layers, also in `reader.rs`.

**VP-025 Kani provability note (I-2 resolution, ADR-009 rev 5):** The base-10 branch
of `pcapng_timestamp_to_secs_usecs` currently calls `10u64.checked_pow(e as u32)` which
is iterative. With symbolic `e`, the VP-025 Kani harness MUST carry `#[kani::unwind(128)]`
OR the implementation must use a precomputed lookup table for e∈[0,19] (preferred —
eliminates the loop entirely, making the proof trivially bounded). Without one of these,
Kani's default unwind=1 produces a vacuous (false-pass) proof. See ADR-009 rev 5
VP-025 Kani Provability Note for full analysis. This MUST be resolved before STORY-125
F3 story decomposition; the choice must be reflected in BC-2.01.014's implementation
notes.

[c]: **VP-027 status=active (post F-F5P1-001 fix, 2026-06-21).** Status `active` means:
the harness `reader::kani_proofs::vp027_epb_parse_safety` calls the real
`decode_epb_body` (pure-core, src/reader.rs) over symbolic inputs and asserts the
E-INP-008/E-INP-009/E-INP-010 discriminants. `cargo kani` reports VERIFICATION SUCCESSFUL
(687 checks) with confirmed non-vacuity (deliberate-flip negative test). Not yet
`verified` (that transition occurs at F6 lock gate per the VP-022/023/024 pattern).
BMC tractability is via the `EpbDecodeError` discriminant twin `decode_epb_body_discriminant`
which mirrors the production path line-by-line (twin faithfulness confirmed in PR review).
SEC-001 twin-drift risk: a `#[cfg(test)]` equivalence smoke test is a TRACKED FOLLOW-UP
obligation. Cite F-F5P1-001 adjudication and PR #287 as the authority.

## File Naming Convention

VP files: `vp-NNN-<short-slug>.md` where NNN is zero-padded to 3 digits.
All VP files reside in `.factory/specs/verification-properties/`.
