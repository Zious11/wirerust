# STORY-125 Demo Evidence Manifest
## pcapng EPB Parse + Nanosecond Timestamp Resolution (E-19, Wave 53)

All 20 acceptance-criterion tests pass (`cargo test --test bc_2_01_story125_epb_tests`).
Visual recordings produced with VHS 0.11.0. Fixtures built programmatically via `build_demo_fixtures.py`.

---

## Recording 1 — AC-002 + AC-003: Graceful Error Paths (No Panics)

**Files:** `AC-001-002-003-error-paths.gif` / `.webm` / `.tape`

**What it shows:**

- **AC-002 (E-INP-009):** `wirerust analyze epb_before_idb.pcapng` — a 64-byte pcapng with SHB + EPB (no IDB present). The parser returns a structured error:
  ```
  EPB references interface_id=0 but interface table is empty — no IDB has been parsed (E-INP-009)
  ```
  No panic. Exit code non-zero. This was a real crash path before STORY-125.

- **AC-003 (E-INP-010):** `wirerust analyze epb_oob_interface_id.pcapng` — a 84-byte pcapng with SHB + one IDB + EPB with `interface_id=5` (table size=1). The parser returns:
  ```
  EPB interface_id=5 out of range (table size=1) (E-INP-010)
  ```
  No panic. The OOB discriminant (`E-INP-010`) is distinct from the empty-table error (`E-INP-009`).

**ACs covered:** AC-002 (BC-2.01.012 PC5a), AC-003 (BC-2.01.012 PC5b)

---

## Recording 2 — AC-013: Valid Multi-Packet pcapng End-to-End Parse

**Files:** `AC-013-valid-pcapng-parse.gif` / `.webm` / `.tape`

**What it shows:**

`wirerust summary multi_packet.pcapng` — a 192-byte pcapng containing SHB + IDB (default µs resolution) + 3 EPBs. The parser:
- Completes without error
- Reports `Skipped: 3 packets (decode errors)` — the EPB payload is synthetic zeros/patterns, not real Ethernet frames, so the L2 decoder skips them at the decode layer (not the parse layer). This is correct: the pcapng format parsing succeeded; the protocol decoder correctly rejects non-Ethernet payloads.

**AC covered:** AC-013 (BC-2.01.012 PC6 — happy path N-packet order and byte fidelity)

---

## Recording 3 — AC-009 / AC-010 / AC-011: Test Suite (20 Tests, 0 Failed)

**Files:** `AC-009-010-011-test-suite.gif` / `.webm` / `.tape`

**What it shows:**

`cargo test --test bc_2_01_story125_epb_tests` running all 20 EPB/timestamp tests:

```
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured
```

Key tests visible:
- `test_BC_2_01_014_regression_1000x_bug` — pins the F-3 fix: nanosecond pcapng with
  `if_tsresol=9`, timestamp=1,500,000,000 ns ticks → `ts_sec=1`, `ts_usecs=500_000`.
  Before the fix: `ts_sec=1500` (1000x wrong because `DEFAULT_TSRESOL=6` was hardcoded).
- `test_BC_2_01_014_nanosecond_resolution_correct` — verifies the per-interface tsresol lookup.
- `test_BC_2_01_014_usecs_default_matches_classic_pcap` — microsecond default path unaffected.
- `test_BC_2_01_014_e127_no_panic` — base-2 exponent=127 clamp does not panic.

**ACs covered:** AC-009 (F-3 1000x bug regression), AC-010 (µs default path), AC-011 (base-2 exponent coverage)

---

## ACs Not Directly Visible in CLI Output

| AC | Reason |
|----|--------|
| **AC-009 headline (1000x timestamp bug)** | `timestamp_secs`/`timestamp_usecs` fields are internal to `PcapPacket` and not surfaced in any CLI output path. The bug is only observable via the library's public API (`PcapSource::from_pcap_reader`). Demo uses the test `test_BC_2_01_014_regression_1000x_bug` which exercises the exact before/after comparison (`ts_sec` must equal 1, not 1500). |
| **AC-012 (VP-025/VP-027 Kani formal proofs)** | Kani is a `#[cfg(kani)]`-gated harness — it runs only under the Kani model checker, not under `cargo test`. The harnesses exist at `tests/kani_proofs.rs`. No VHS recording is possible without the Kani toolchain. |
| **AC-001 (E-INP-008 body-too-short)** | Shown in the test suite recording via `test_BC_2_01_012_epb_body_short_e_inp_008`. The CLI exits with an error on short EPB bodies; covered by AC-002/AC-003 pattern in error-paths recording. |

---

## Fixture Files

| File | Size | Purpose |
|------|------|---------|
| `epb_before_idb.pcapng` | 64 B | AC-002: triggers E-INP-009 |
| `epb_oob_interface_id.pcapng` | 84 B | AC-003: triggers E-INP-010 |
| `multi_packet.pcapng` | 192 B | AC-013: 3-EPB happy path |
| `ns_resolution_1sec.pcapng` | 148 B | Reference: nanosecond IDB fixture |
| `us_resolution_1sec.pcapng` | 136 B | Reference: microsecond IDB fixture |

Fixtures are minimal hand-crafted pcapng binaries built from the same byte-layout constants used in `tests/bc_2_01_story125_epb_tests.rs`. They do not contain real network traffic.

---

## Test Coverage Summary

```
cargo test --test bc_2_01_story125_epb_tests

running 20 tests
  test_BC_2_01_012_data_bounded_by_captured_len ............. ok (AC-005)
  test_BC_2_01_012_endianness_be_interface_id_and_timestamp .. ok (BE coverage)
  test_BC_2_01_012_epb_body_short_e_inp_008 ................. ok (AC-001)
  test_BC_2_01_012_guard_before_allocate .................... ok (AC-004)
  test_BC_2_01_012_happy_path_n_packet_order_and_byte_fidelity ok (AC-013)
  test_BC_2_01_012_interface_id_bounds_check ................ ok (AC-002, AC-003)
  test_BC_2_01_012_max_boundary_captured_len ................ ok (AC-007)
  test_BC_2_01_012_no_panic_malformed ....................... ok (AC-001 no-panic)
  test_BC_2_01_012_raw_block_path_not_crate_duration ........ ok (AC-008)
  test_BC_2_01_012_zero_byte_captured_len ................... ok (AC-006)
  test_BC_2_01_014_base10_e0_one_tick_per_sec ............... ok
  test_BC_2_01_014_base2_e20_known_vector ................... ok (AC-011)
  test_BC_2_01_014_e127_no_panic ............................ ok (AC-011)
  test_BC_2_01_014_e2e_le_microsecond_correct_timestamp ..... ok
  test_BC_2_01_014_fast_path_saturation_guard ............... ok (AC-010)
  test_BC_2_01_014_invariant_ts_usecs_in_range .............. ok
  test_BC_2_01_014_nanosecond_resolution_correct ............ ok (AC-009)
  test_BC_2_01_014_regression_1000x_bug ..................... ok (AC-009 headline)
  test_BC_2_01_014_saturation_extreme_ticks ................. ok
  test_BC_2_01_014_usecs_default_matches_classic_pcap ....... ok (AC-010)

test result: ok. 20 passed; 0 failed; 0 ignored
```

VP-025 and VP-027 Kani harnesses exist at `tests/kani_proofs.rs` for Phase-6 formal hardening.
