# Red Gate Log — STORY-125

**Story:** STORY-125 — EPB Parse and 64-bit Timestamp Normalization  
**Epic:** E-19  
**Wave:** 53  
**Date:** 2026-06-20  
**Phase:** F3 (TDD Red Gate)

## Summary

Red Gate verification completed. All new STORY-125 tests compile and fail for the
correct reasons. STORY-123 and STORY-124 existing tests remain GREEN.

## Test Files Written

- `/tests/bc_2_01_story125_epb_tests.rs` — 20 tests total (10 RED, 10 GREEN)
- `/tests/kani_proofs.rs` — VP-025 and VP-027 Kani harnesses (gated `#[cfg(kani)]`)

## Compilation

```
cargo check --all-targets → Finished (0 errors, 0 warnings)
```

Both new test files compile cleanly under the stable toolchain.
`tests/kani_proofs.rs` compiles to an empty module under `cargo check` (all
content is `#[cfg(kani)]`-gated), satisfying the "must compile under normal
toolchain" requirement.

## Red Gate Results

### Tests That FAIL (RED) — 10 tests

| Test Name | Failure Reason | Why This Is Correct |
|-----------|---------------|---------------------|
| `test_BC_2_01_014_usecs_default_matches_classic_pcap` | `todo!()` panic in `pcapng_timestamp_to_secs_usecs` | Function is a stub; not yet implemented |
| `test_BC_2_01_014_fast_path_saturation_guard` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_nanosecond_resolution_correct` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_base10_e0_one_tick_per_sec` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_e127_no_panic` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_base2_e20_known_vector` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_saturation_extreme_ticks` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_invariant_ts_usecs_in_range` | `todo!()` panic | Same stub |
| `test_BC_2_01_014_regression_1000x_bug` | Assertion: `ts_sec=1500 == 1500` (detected the 1000× bug) | EPB arm hard-codes DEFAULT_TSRESOL=6; ns ticks produce ts_sec=1500 instead of correct ts_sec=1 |
| `test_BC_2_01_012_interface_id_bounds_check` | Assertion: empty-table message doesn't contain "interface table is empty" | Current message format differs from BC-2.01.012 PC5a required format; AND OOB-on-non-empty (E-INP-010) discriminant is missing |

### The F-3 1000× Bug (Current Wrong vs Expected)

```
test_BC_2_01_014_regression_1000x_bug:

  Input:  if_tsresol=9 (nanoseconds), ts_low=1_500_000_000

  CURRENT WRONG BEHAVIOR (hardcoded DEFAULT_TSRESOL=6):
    ticks_per_sec = 1_000_000 (µs)  ← WRONG for ns capture
    ts_sec = 1_500_000_000 / 1_000_000 = 1500  ← 1000× too large!
    ts_usecs = 0

  CORRECT BEHAVIOR (after implementation with per-interface if_tsresol=9):
    ticks_per_sec = 1_000_000_000 (ns)
    ts_sec = 1_500_000_000 / 1_000_000_000 = 1
    ts_usecs = 500_000 (500 ms)
```

### Tests That PASS (GREEN) — 10 tests

These pass because they exercise already-implemented behavior (from STORY-123/124):

| Test Name | Why GREEN Now | Will It Stay GREEN After Implementation? |
|-----------|--------------|------------------------------------------|
| `test_BC_2_01_012_epb_body_short_e_inp_008` | Body-length check already in EPB arm (lines 802-809) | Yes — same assertion |
| `test_BC_2_01_012_no_panic_malformed` | Same body-length check | Yes |
| `test_BC_2_01_012_guard_before_allocate` | PC6a already coded (lines 861-867) | Yes |
| `test_BC_2_01_012_data_bounded_by_captured_len` | captured_len slicing already correct | Yes |
| `test_BC_2_01_012_zero_byte_captured_len` | Zero-byte packet already works | Yes |
| `test_BC_2_01_012_max_boundary_captured_len` | Boundary already correct | Yes |
| `test_BC_2_01_012_raw_block_path_not_crate_duration` | ts_low=1_000_000 with DEFAULT_TSRESOL=6 → ts_sec=1 (coincidentally correct since default=6) | Yes — implementation must preserve this |
| `test_BC_2_01_014_e2e_le_microsecond_correct_timestamp` | Explicit if_tsresol=6 IDB + EPB; current code uses DEFAULT=6 → same result | Yes — implementation must produce same result for if_tsresol=6 |
| `test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity` | 16-packet encounter order already correct; test doesn't assert timestamps | Yes |
| `test_BC_2_01_012_endianness_be_interface_id_and_timestamp` | BE ts_high=1 correctly decoded → ts_sec=4294 with inline DEFAULT_TSRESOL=6 formula | Yes — same calculation via pcapng_timestamp_to_secs_usecs(1, 0, 6) |

### Previously Existing Tests (STORY-123 + STORY-124) — ALL GREEN

```
cargo test --test bc_2_01_story123_pcapng_tests → 31 passed; 0 failed
cargo test --test bc_2_01_story124_idb_tests → 27 passed; 0 failed
```

No regressions in STORY-123 or STORY-124 test suites.

## BC Ambiguities Noted

### Ambiguity 1: E-INP-009 Message Format (PC5a)

The BC-2.01.012 PC5a requires the exact message:
  `"EPB references interface_id=<id> but interface table is empty — no IDB has been parsed"`

The CURRENT implementation (reader.rs line 812-815) produces:
  `"pcapng EPB encountered before any IDB has been parsed (E-INP-009: no interface table entry)"`

Both contain "E-INP-009" but the current message does not match the BC-required format.
The test `test_BC_2_01_012_interface_id_bounds_check` asserts the BC-required format
and FAILS (correct RED behavior). The implementer must update the message to match PC5a.

### Ambiguity 2: OOB-on-Non-Empty (E-INP-010) Missing

The current EPB arm has NO check for `interface_id >= interfaces.len()` on a non-empty
table. The code only checks `interfaces.is_empty()` (E-INP-009) and then proceeds to
index into the interface table without bounds-checking. This is the AC-003 / PC5b
implementation gap. Sub-test B of `test_BC_2_01_012_interface_id_bounds_check`
fails because the OOB case returns Ok (not Err with E-INP-010).

### Ambiguity 3: VP-027 Requires Extracted `decode_epb_body`

VP-027's Kani harness requires calling the EPB decode function directly (not via I/O
stream). The implementer MUST extract the EPB body decode logic into a separate
pure function (e.g., `pub fn decode_epb_body(body: &[u8], interfaces: &[InterfaceInfo],
endianness: SectionEndianness) -> Result<RawPacket>`) and export it (at least
`#[doc(hidden)] pub`) for the Kani harness to call. The current inline match-arm
implementation cannot be reached from Kani without I/O.

The VP-027 harness in `tests/kani_proofs.rs` is STRUCTURAL (uses modeled assertions)
and is ready to be wired to the real function call once it is extracted. Phase-6
action item for the implementer.

## Red Gate Verdict

RED GATE: PASS

- 10 new tests FAIL (correct: they target unimplemented or wrong behavior)
- 10 new tests PASS (correct: they test already-implemented behavior that STORY-125 must not break)
- 0 regressions in STORY-123/124 existing tests
- Both test files COMPILE with zero errors under `cargo check --all-targets`
- `kani_proofs.rs` compiles to empty module under normal toolchain (non-kani build)

The implementation may now proceed. Implement tests one at a time, starting with
`pcapng_timestamp_to_secs_usecs` (the `todo!()` stub) — all timestamp-helper tests
will then turn GREEN. Then wire the EPB arm to call the helper with per-interface
`if_tsresol`, which will turn the regression test GREEN. Finally fix the E-INP-009/010
discriminant split to turn the bounds-check test GREEN.
