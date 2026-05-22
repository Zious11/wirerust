# Red Gate Log — STORY-001 Phase 3

**Story:** STORY-001 — PCAP File Ingestion (Link-Type Gating, Eager Packet Load, Error Surfaces)
**Cycle:** v0.1.0-greenfield-spec
**Wave:** 1
**Implementation strategy:** brownfield-formalization
**Date:** 2026-05-21
**Test file:** `tests/bc_2_01_story001_tests.rs`
**Agent:** test-writer

> **Correction — 2026-05-21 (adversary pass 5):** Initial log recorded 19 tests;
> the correct count is 20. `test_BC_2_01_008_permission_denied_error`
> (`#[cfg(unix)]`-gated) was omitted from the Per-Test Results table and all
> count references. All three occurrences corrected below.
>
> **Correction — 2026-05-21 (adversary pass 6):** Per-Test Results row and BC
> Coverage Map entry for `test_BC_2_01_007_truncated_packet_error` previously
> credited the test with exercising BC-2.01.007 invariant 1 (all-or-nothing) and
> postcondition 2 (no partial Vec) by test exercise. Both are in fact structurally
> guaranteed by the `Result<PcapSource>` return type — `Err` and `Ok(partial)` are
> mutually exclusive shapes. The log now records this distinction accurately.
>
> **Correction — 2026-05-21 (adversary passes 8/9/10 — comprehensive accuracy pass):**
> (1) Per-Test Results: "AC" column renamed to "Traces to" to honestly cover the mix
> of AC-NNN, EC-NNN, and VP references it contains; `permission_denied` row "Traces
> to" cell corrected from a bare BC clause reference to `EC-002 (BC-2.01.008)` — the
> BC-2.01.008 edge-case ID for "file exists but not readable" (STORY-001 has no
> permission-denied EC in its own edge-case table; EC-007 there is "Non-existent file
> path", a different scenario).
> (2) BC Coverage Map: every clause annotated as either "(test-exercised)",
> "(structural / type-guaranteed)", or "(exercised as test input)" per the actual
> test inventory. BC-2.01.002 invariant 1 (eager load — no streaming API) corrected
> to structural/architectural; BC-2.01.002 postcondition 4 annotated with shared-test
> reference and type-guarantee note; BC-2.01.001 invariant 3 softened to "exercised
> as test input". See table below for full detail.

---

## Summary

All 20 new tests PASS on the first run. This is expected: the
implementation strategy is `brownfield-formalization`, which means
`src/reader.rs` already exists and is being formally covered by tests
for the first time. Every test confirms existing correct behavior; no
implementation gaps were found.

`cargo clippy --all-targets -- -D warnings` exits 0.
`cargo test --all-targets` exits 0 (261+ tests across all modules).

---

## Red Gate Disposition

| Disposition | Count |
|-------------|-------|
| PASS (brownfield-confirm) | 20 |
| FAIL (implementation gap) | 0 |

---

## Per-Test Results

| Test Name | Traces to | BC clause(s) exercised | Result | Disposition |
|-----------|-----------|------------------------|--------|-------------|
| `test_BC_2_01_001_accepts_all_five_link_types` | AC-001 | BC-2.01.001 postcondition 1 | PASS | brownfield-confirm |
| `test_BC_2_01_001_rejects_unsupported_link_type` | AC-002 | BC-2.01.001 postcondition 2 | PASS | brownfield-confirm |
| `test_BC_2_01_002_packet_count_and_order` | AC-003 | BC-2.01.002 postcondition 1, postcondition 2 (data bytes) | PASS | brownfield-confirm |
| `test_BC_2_01_002_timestamp_preserved_microsecond` | AC-004 | BC-2.01.002 postcondition 2 + BC-2.01.005 postcondition 1 | PASS | brownfield-confirm |
| `test_BC_2_01_002_timestamp_preserved_nanosecond` | AC-004 | BC-2.01.002 postcondition 2 + BC-2.01.005 postcondition 2 | PASS | brownfield-confirm |
| `test_BC_2_01_003_zero_packet_pcap` | AC-005 | BC-2.01.003 postcondition 1 | PASS | brownfield-confirm |
| `test_BC_2_01_004_rejects_pcapng` | AC-006 | BC-2.01.004 postcondition 1 | PASS | brownfield-confirm |
| `test_BC_2_01_005_nanosecond_resolution_conversion` | AC-007 | BC-2.01.005 postcondition 2 | PASS | brownfield-confirm |
| `test_BC_2_01_006_corrupt_header_error_message` | AC-008 | BC-2.01.006 postcondition 1 | PASS | brownfield-confirm |
| `test_BC_2_01_007_truncated_packet_error` | AC-009 | BC-2.01.007 postcondition 1 (test-exercised); postcondition 2 + invariant 1 structural / type-guaranteed by `Result<PcapSource>` return type, not test-exercised | PASS | brownfield-confirm |
| `test_BC_2_01_008_file_not_found_error` | AC-010 | BC-2.01.008 postcondition 2 | PASS | brownfield-confirm |
| `test_BC_2_01_008_permission_denied_error` _(#[cfg(unix)]; runs on Unix CI runners, not compiled on non-Unix)_ | EC-002 (BC-2.01.008) | BC-2.01.008 postcondition 2 (permission-denied path) | PASS | brownfield-confirm |
| `test_BC_2_01_001_proptest_non_whitelist_link_type_rejected` | VP: BC-2.01.001 invariant 2 | BC-2.01.001 postcondition 2, invariant 2 (proptest, 1000 cases, full u32 range) | PASS | brownfield-confirm |
| `test_BC_2_01_001_proptest_whitelist_link_type_accepted` | VP: BC-2.01.001 postcondition 1 | BC-2.01.001 postcondition 1 (proptest, whitelist) | PASS | brownfield-confirm |
| `test_BC_2_01_005_ts_sec_u32_max_stored_as_is` | EC-005 (STORY-001) | BC-2.01.002 EC-006 + BC-2.01.005 EC-002 | PASS | brownfield-confirm |
| `test_BC_2_01_001_raw_and_ipv4_both_accepted` | EC-008 (STORY-001) | BC-2.01.001 EC-004 | PASS | brownfield-confirm |
| `test_BC_2_01_003_zero_packet_linux_sll` | BC-2.01.003 EC-002 | BC-2.01.003 postcondition 1 (LINUX_SLL variant) | PASS | brownfield-confirm |
| `test_BC_2_01_005_zero_timestamp_nanosecond` | BC-2.01.005 TV | BC-2.01.005 postcondition 2 (ts_sec=0, ts_frac=0) | PASS | brownfield-confirm |
| `test_BC_2_01_006_truncated_header_error_message` | AC-008 (variant: EC-002) | BC-2.01.006 EC-002 (valid-magic-but-truncated header) | PASS | brownfield-confirm |
| `test_BC_2_01_008_from_file_delegates_to_from_pcap_reader` | AC-010 (delegation) | BC-2.01.008 postcondition 1 | PASS | brownfield-confirm |

---

## Fixture Analysis

No new fixture files were written to disk. All fixtures required for
the 8 BCs were generated programmatically inside the test binary:

| Fixture scenario | Source |
|-----------------|--------|
| ETHERNET (link 1) | inline bytes |
| RAW (link 101) | inline bytes |
| LINUX_SLL (link 113) | inline bytes |
| IPV4 (link 228) | inline bytes |
| IPV6 (link 229) | inline bytes |
| IEEE 802.11 (link 105, unsupported) | inline bytes |
| Zero-packet pcap (24-byte header only) | inline bytes |
| Nanosecond-resolution pcap (magic 0xa1b23c4d) | inline bytes |
| Truncated packet record | inline bytes |
| Non-existent file path | runtime ephemeral path |
| pcapng (smb3.pcapng) | existing fixture |
| Valid multi-packet pcap | inline bytes |

The existing `tests/fixtures/tls.pcap` (ETHERNET) is also used in
`test_BC_2_01_008_from_file_delegates_to_from_pcap_reader` to confirm
`from_file` and `from_pcap_reader` produce identical results.

Note: No existing fixture has link type 229 (IPV6) or 113 (LINUX_SLL)
at the file level; those link-type variants are covered by inline
pcap bytes in the test, not by a named fixture file.

---

## BC Coverage Map

| BC | Clauses covered | Coverage mode |
|----|----------------|---------------|
| BC-2.01.001 | postcondition 1 (test-exercised: `test_BC_2_01_001_accepts_all_five_link_types`); postcondition 2 (test-exercised: `test_BC_2_01_001_rejects_unsupported_link_type`, proptest); postcondition 3 (structural — follows from pc-1 + pc-2 by return type); invariant 1 (test-exercised: proptest 1000 cases confirm whitelist size behaviorally); invariant 2 (test-exercised: proptest no-panic property across full u32 range); invariant 3 (exercised as test input — numeric constants are owned by the pcap-file crate and validated by pc-1/pc-2 tests using those values, not by a separate assertion on the encoding) | mixed |
| BC-2.01.002 | postcondition 1 (test-exercised: `test_BC_2_01_002_packet_count_and_order`); postcondition 2 (test-exercised: timestamp fields in `test_BC_2_01_002_timestamp_preserved_microsecond`/`_nanosecond`; data bytes in `test_BC_2_01_002_packet_count_and_order`); postcondition 3 (test-exercised: order verified by distinct-payload assertion in `test_BC_2_01_002_packet_count_and_order`); postcondition 4 (shares `test_BC_2_01_007_truncated_packet_error`; all-or-nothing part structural / type-guaranteed by `Result<PcapSource>`); invariant 1 (structural / architectural — eager-load is an API design property; no streaming return type exists; not separately test-exercised) | mixed |
| BC-2.01.003 | postcondition 1 (test-exercised: `test_BC_2_01_003_zero_packet_pcap`); postcondition 2 (test-exercised: no error returned); postcondition 3 (test-exercised: no panic — implicit in test passing); invariant 1 (test-exercised: `packets.is_empty()` assertion confirms empty Vec is a valid state) | test-exercised |
| BC-2.01.004 | postcondition 1 (test-exercised: `test_BC_2_01_004_rejects_pcapng`); postcondition 2 (test-exercised: no packets returned — implicit in Err return); postcondition 3 (test-exercised: no panic — implicit in test passing) | test-exercised |
| BC-2.01.005 | postcondition 1 (test-exercised: `test_BC_2_01_002_timestamp_preserved_microsecond`); postcondition 2 (test-exercised: `test_BC_2_01_002_timestamp_preserved_nanosecond`, `test_BC_2_01_005_nanosecond_resolution_conversion`); postcondition 3 (structural — u32 field type enforced by the compiler, not a runtime assertion); invariant 1 (test-exercised: `test_BC_2_01_005_ts_sec_u32_max_stored_as_is` confirms no normalization); invariant 2 (structural — u32 capacity boundary; confirmed as accepted limitation by type, not tested at Y2106) | mixed |
| BC-2.01.006 | postcondition 1 (test-exercised: `test_BC_2_01_006_corrupt_header_error_message` — empty bytes + garbage magic; `test_BC_2_01_006_truncated_header_error_message` — valid-magic-but-truncated); postcondition 2 (test-exercised: no packets returned — implicit in Err return); postcondition 3 (test-exercised: no panic — implicit in tests passing); invariant 1 (test-exercised: both sub-cases use the same context string, confirming it is stable across error origins) | test-exercised |
| BC-2.01.007 | postcondition 1 (test-exercised: `test_BC_2_01_007_truncated_packet_error` — error returned with correct context string); postcondition 2 (structural / type-guaranteed by `Result<PcapSource>` return type — `Err` and `Ok(partial)` are mutually exclusive; not separately test-exercised); invariant 1 (structural / type-guaranteed — same reasoning as postcondition 2) | mixed |
| BC-2.01.008 | postcondition 1 (test-exercised: `test_BC_2_01_008_from_file_delegates_to_from_pcap_reader` — packet count, datalink, timestamps, and data all compared); postcondition 2 (test-exercised: `test_BC_2_01_008_file_not_found_error` — missing file; `test_BC_2_01_008_permission_denied_error` — unreadable file [unix only]); invariant 1 (structural — delegation is a code-structure property confirmed by code review; the equivalence test exercises it behaviorally) | mixed |

---

## Implementation Gaps Found

None. All 8 behavioral contracts are fully satisfied by `src/reader.rs`
as it exists in this worktree. No modifications to `src/` are required
for STORY-001.

---

## Compilation and Lint

| Check | Result |
|-------|--------|
| `cargo check --tests` | OK |
| `cargo clippy --all-targets -- -D warnings` | OK (0 errors, 0 warnings) |
| `cargo test --all-targets` | OK (all tests pass) |

Note: The BC-based test naming pattern (`test_BC_S_SS_NNN_xxx`) uses
uppercase letters that violate Rust's `non_snake_case` lint. The file
opens with `#![allow(non_snake_case)]` to satisfy both the factory
naming mandate and CI's `-D warnings` gate. This is the correct
resolution per the agent's operating rules.
