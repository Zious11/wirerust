---
artifact: vp-index
traces_to: .factory/specs/architecture/ARCH-INDEX.md
version: "2.0"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified: "2026-06-02: Phase-6 gate close — all 20 VPs locked (status draft→verified, verification_lock→true). VP-INDEX status→verified. module-criticality frozen. develop@0855f25."
total_vps: 20
p0_count: 8
p1_count: 7
test_sufficient_count: 5
kani_count: 8
proptest_count: 6
fuzz_count: 1
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
| 20 | 8 | 7 | 5 |

| Tool | Count | VP IDs |
|------|-------|--------|
| Kani | 8 | VP-001, VP-002, VP-003, VP-004, VP-005, VP-007, VP-009, VP-015 |
| proptest | 6 | VP-006, VP-010, VP-011, VP-012, VP-013, VP-014 |
| cargo-fuzz | 1 | VP-008 |
| integration/unit | 5 | VP-016, VP-017, VP-018, VP-019, VP-020 |

> VP-005 (SNI 4-way ordered classification) uses Kani as its primary and sole
> counted tool. Each VP is counted exactly once. Totals: 8+6+1+5 = 20.

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

## Test-Sufficient Properties (VP-016..VP-020)

These five properties are verified by standard Rust integration or unit tests.
No formal proof harness (Kani/proptest) is required.

| VP-ID | Verification method |
|-------|-------------------|
| VP-016 | Integration test: fixed finding sets; tactic order assertion |
| VP-017 | Integration test: determinism round-trip; C0 escape check |
| VP-018 | CLI test (assert_cmd): mutual exclusion exit code |
| VP-019 | Unit test: empty Vec<Finding> assertion for all DNS packets |
| VP-020 | Unit test: injection character prefix check in CSV output |

## Consistency Invariants (machine-enforced by validate-vp-consistency.sh)

- VP-INDEX total (20) must equal verification-architecture.md row count (20)
- VP-INDEX total (20) must equal verification-coverage-matrix.md VP row count (20)
- verification-coverage-matrix.md Totals row: Kani(8) + proptest(6) + fuzz(1) + integration/unit(5) = 20
- P0 count (8) + P1 count (7) + test-sufficient (5) = 20

## File Naming Convention

VP files: `vp-NNN-<short-slug>.md` where NNN is zero-padded to 3 digits.
All VP files reside in `.factory/specs/verification-properties/`.
