# Red Gate Log — STORY-128

**Story:** STORY-128 — main.rs Per-File Error Isolation Loop (Catch-and-Continue)
**Epic:** E-19
**Wave:** 56
**Date:** 2026-06-20
**Phase:** F3 (TDD Red Gate)

## Summary

Red Gate verification completed. All 10 isolation tests are RED against the
pre-refactor `?`-aborting loop. 2 non-isolation tests pass correctly (all-good
batch and single-file fail-closed, which should be GREEN both before and after
refactor). STORY-123..127 existing tests remain GREEN.

## Test File Written

- `/tests/bc_2_01_018_story128_tests.rs` — 12 tests total (10 RED, 2 GREEN)

## Compilation

```
cargo clippy --all-targets -- -D warnings → Finished (0 errors, 0 warnings)
cargo fmt --check → Finished (0 differences)
```

All tests compile cleanly. No warnings.

## Red Gate Results

### cargo test --all-targets output summary

```
test result: ok. 87 passed; 0 failed   (lib unit tests — STORY-123..127 unaffected)
test result: ok. 2 passed; 0 failed    (integration test suite #1 — no regressions)
test result: ok. 3 passed; 0 failed    (integration test suite #2 — no regressions)
test result: FAILED. 2 passed; 10 failed  (bc_2_01_018_story128_tests — RED gate)
```

### Tests That FAIL (RED) — 10 tests

All fail because the current main.rs loop uses `?` to propagate reader errors,
which aborts the batch on the first bad file.

| Test Name | Failure Assertion | Failure Reason (Current Code) |
|-----------|-------------------|-------------------------------|
| `test_BC_2_01_018_per_file_isolation_continues_on_error` | `stdout contains "Skipped: 1 packets"` | Current code aborts on `a-conflict.pcapng` (E-INP-011 `?`-propagated); `b-valid.pcapng` never processed; stdout empty |
| `test_BC_2_01_018_einp011_does_not_abort_batch` | `stdout contains "Skipped: 1 packets"` | Same abort pattern; `2-good.pcapng` never processed |
| `test_BC_2_01_018_any_reader_error_isolated` | `stdout contains "Skipped: 1 packets"` | `a-truncated.pcapng` (E-INP-008) causes abort; `b-valid.pcapng` never processed |
| `test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation` | `stderr contains "b-shb-only.pcapng"` AND `stderr contains "0 packets"` | (1) Abort on `a-conflict.pcapng` prevents `b-shb-only.pcapng` from being processed; (2) zero-packet notice not yet implemented in main.rs |
| `test_BC_2_01_018_order_independence_bad_file_first` | `stdout contains "Skipped: 2 packets"` | `a-bad.pcapng` sorts first; abort leaves `b-good.pcapng` and `c-good.pcapng` unprocessed |
| `test_BC_2_01_018_order_independence_bad_file_last` | `stdout contains "Skipped: 2 packets"` | Good files processed in the closure but `capture_result?` at line 315 causes run_analyze to return Err before `write_output` — stdout empty even though 2 packets were seen in the closure |
| `test_BC_2_01_018_order_independence_bad_file_middle` | `stdout contains "Skipped: 2 packets"` | `b-bad.pcapng` aborts after `a-good.pcapng` completes; `c-good.pcapng` never processed; stdout empty (same capture_result? abort) |
| `test_BC_2_01_018_all_bad_batch_no_panic_exit_one` | `stderr contains "b-bad.pcapng"` | Only `a-bad.pcapng` reported (first bad file via `?` propagation); `b-bad.pcapng` never processed |
| `test_BC_2_01_018_summary_subcommand_per_file_isolation` | `stdout contains "Skipped: 1 packets"` | `run_summary` has same `?` pattern (line 412-413); `a-bad.pcapng` abort prevents `b-good.pcapng` processing |
| `test_BC_2_01_018_zero_packet_notice_decision19_lone_valid_file` | `stderr contains "shb-only.pcapng"` AND `stderr contains "0 packets"` | Zero-packet notice (ADR-009 Decision 19) not implemented in main.rs; SHB-only file returns Ok but no notice is emitted |

### Key Failure Mode Detail: `order_independence_bad_file_last`

This case is instructive because the two good files ARE processed within the
immediately-invoked closure before the bad file causes a bail. However:

1. The closure returns `Err` after the bad file.
2. `capture_result?` at line 315 propagates that `Err` and causes `run_analyze`
   to return early.
3. `write_output` at line 396 is NEVER called.
4. stdout is empty — the terminal report with "Skipped: 2 packets" is never
   printed.

This proves the isolation failure is NOT just about skipping files — even when
good files are processed first, their results are lost because the error
propagation discards the summary output.

### Tests That PASS (GREEN) — 2 tests

These test scenarios that MUST pass both before and after refactor.

| Test Name | Why GREEN Now | Why It Will Stay GREEN |
|-----------|--------------|------------------------|
| `test_BC_2_01_018_all_good_batch_exit_zero` | All good files → no reader Err → closure Ok → write_output called → exit 0 | Refactor replaces `?` with match; Ok arm is unchanged |
| `test_BC_2_01_018_invariant1_reader_fail_closed_preserved` | Single bad file → `?` propagates → run_analyze returns Err → exit 1; path appears in stderr anyhow chain | Post-refactor: single file → Err arm → eprintln("error: <path>: ..."); any_error=true; loop ends → exit 1. Still exit 1, path still in stderr. |

Note on `invariant1_reader_fail_closed_preserved`: the stderr format changes
after refactor (anyhow chain vs. per-file eprintln), but the test only asserts
`stderr contains "single-bad.pcapng"` and `failure()`, so it stays GREEN after
the refactor implements the per-file format.

### Previously Existing Tests (STORY-123..127) — ALL GREEN

```
cargo test --test bc_2_01_story123_pcapng_tests → 31 passed; 0 failed
cargo test --test bc_2_01_story124_idb_tests    → 27 passed; 0 failed
cargo test --test bc_2_01_story125_epb_tests    → 20 passed; 0 failed
cargo test --test bc_2_01_story126_spb_tests    → 29 passed; 0 failed
cargo test --test bc_2_12_011_story127_tests    →  9 passed; 0 failed
```

No regressions in any prior story's test suite.

## AC-to-Test Mapping

| AC | Test Name | Status |
|----|-----------|--------|
| AC-001 (per-file isolation) | `test_BC_2_01_018_per_file_isolation_continues_on_error` | RED |
| AC-002 (E-INP-011 isolation) | `test_BC_2_01_018_einp011_does_not_abort_batch` | RED |
| AC-003 (all error classes) | `test_BC_2_01_018_any_reader_error_isolated` | RED |
| AC-004 (zero-packet notice) | `test_BC_2_01_018_zero_packet_notice_not_suppressed_by_isolation` | RED |
| Additional (order independence: bad first) | `test_BC_2_01_018_order_independence_bad_file_first` | RED |
| Additional (order independence: bad last) | `test_BC_2_01_018_order_independence_bad_file_last` | RED |
| Additional (order independence: bad middle) | `test_BC_2_01_018_order_independence_bad_file_middle` | RED |
| Additional (all-bad batch) | `test_BC_2_01_018_all_bad_batch_no_panic_exit_one` | RED |
| Additional (run_summary isolation) | `test_BC_2_01_018_summary_subcommand_per_file_isolation` | RED |
| Additional (Decision 19 standalone) | `test_BC_2_01_018_zero_packet_notice_decision19_lone_valid_file` | RED |
| EC-002 (all-good batch) | `test_BC_2_01_018_all_good_batch_exit_zero` | GREEN |
| Inv1 (reader fail-closed) | `test_BC_2_01_018_invariant1_reader_fail_closed_preserved` | GREEN |

## BC Ambiguities Noted

### Ambiguity 1: Exit-code format vs. per-file eprintln format

The `invariant1_reader_fail_closed_preserved` test currently passes with the
CURRENT code's anyhow-chain stderr format ("Error: Failed to read ...\n\nCaused
by: ...") because the test only asserts `stderr contains "single-bad.pcapng"`.
Post-refactor, the format changes to "error: single-bad.pcapng: ..." (the
per-file eprintln format per AC-001). Both contain the filename, so the test
stays GREEN. The implementer should be aware that the error message FORMAT
changes — test passes either way, but the format is BC-normative.

### Ambiguity 2: Zero-packet notice for classic pcap vs. pcapng

ADR-009 Decision 19 mandates the zero-packet notice for BOTH pcapng ("0 packets
read from pcapng file") AND classic pcap ("0 packets read from pcap file").
The `test_BC_2_01_018_zero_packet_notice_decision19_lone_valid_file` test uses
an SHB-only pcapng file, so the pcapng format is tested. A separate test for
classic pcap zero-packet notice is NOT included in STORY-128's test plan (it
would require a valid zero-packet classic pcap fixture). The PO should confirm
whether STORY-128 or a separate story owns the classic pcap zero-packet notice.

### Ambiguity 3: Zero-packet notice format pinning

The tests assert `stderr contains "0 packets"` but do NOT pin the exact format
"notice: <filename>: 0 packets read from pcapng file". The exact format is the
implementer's choice consistent with ADR-009 Decision 19. If tighter pinning is
desired, the implementer should update the test after implementing to use:
`.stderr(predicate::str::contains("notice: "))` or the exact format.

## Red Gate Verdict

RED GATE: PASS

- 10 new tests FAIL (correct: they target the unimplemented isolation loop)
- 2 new tests PASS (correct: they test pre-existing correct behavior)
- 0 regressions in STORY-123..127 existing tests (116 tests, all GREEN)
- Test file COMPILES with zero errors and zero warnings under:
  - `cargo check --all-targets`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --check`

The implementation may now proceed. Implementer must:
1. In `run_analyze` (src/main.rs ~line 239-303): add `let mut any_error = false;`
   before the inner `for path in &pcap_files` loop, replace `from_file(path)?`
   with explicit `match`, Err arm: `eprintln!("error: {}: {e:#}", path.display());
   any_error = true; continue;`, Ok arm: add `if source.packets.is_empty() {
   eprintln!("notice: {}: 0 packets read from pcapng file", path.display()); }`.
   After the closure, add: `if any_error { std::process::exit(1); }`.
2. In `run_summary` (src/main.rs ~line 409-433): apply the same pattern.
3. DO NOT modify `src/reader.rs`.

---

## Addendum: PC6 Notice Format Tests (adversarial review C-1/M-1/H-1)

**Date:** 2026-06-20
**Scope:** 10 additional tests pinning the FULL BC-2.01.009 PC6 notice format
          (OPB clause, generic-skip segment, classic-pcap wording, gate suppression).

These tests were added AFTER the isolation loop implementation landed — the
isolation tests above are now GREEN. The new PC6 format tests are RED against
the current bare notice implementation.

### Current State (implementation delivered, notice format incomplete)

The original 12 STORY-128 tests are now all GREEN (isolation loop implemented).
The 10 new PC6 format tests are RED because `src/main.rs` emits the bare notice:
  `eprintln!("notice: {}: 0 packets read from pcapng file", path.display())`
Missing: OPB clause, generic-skip segment, classic-pcap "pcap file" wording.

### New Tests (10 total: 8 RED, 2 GREEN)

| Test Name | Status | Failure Reason |
|-----------|--------|---------------|
| `test_BC_2_01_009_pc6_opb_clause_analyze` | RED | "obsolete" absent from bare notice |
| `test_BC_2_01_009_pc6_opb_clause_summary` | RED | "obsolete" absent from bare notice |
| `test_BC_2_01_009_pc6_generic_skip_segment_analyze` | RED | "skipped as unsupported" absent from bare notice |
| `test_BC_2_01_009_pc6_generic_skip_segment_summary` | RED | "skipped as unsupported" absent from bare notice |
| `test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_analyze` | RED | Both "skipped as unsupported" and "obsolete" absent |
| `test_BC_2_01_009_pc6_both_segments_nrb_plus_opb_summary` | RED | Both "skipped as unsupported" and "obsolete" absent |
| `test_BC_2_01_009_pc6_classic_pcap_wording_analyze` | RED | Notice says "pcapng file" for .pcap input; `NOT contains("pcapng file")` fails |
| `test_BC_2_01_009_pc6_classic_pcap_wording_summary` | RED | Same hardcoded "pcapng file" in run_summary notice |
| `test_BC_2_01_009_pc6_neither_segment_shb_only_analyze` | GREEN | Bare notice has no parenthetical — gate-suppression is vacuously correct |
| `test_BC_2_01_009_pc6_neither_segment_shb_only_summary` | GREEN | Same |

### Exact Cargo Output

```
test result: FAILED. 14 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out
```

The 14 passing = 12 original STORY-128 tests + 2 neither-segment gate tests.
The 8 failing = OPB-clause (x2), generic-skip (x2), both-segments (x2), classic-wording (x2).

### What Each Failing Test Currently Emits vs. Expects

For ALL 8 failing tests, the current notice emitted is:
  `"notice: <tempdir>/<file>: 0 packets read from pcapng file"`

- OPB-clause tests expect: `contains("obsolete")` → ABSENT → FAIL
- Generic-skip tests expect: `contains("skipped as unsupported")` → ABSENT → FAIL
- Both-segments tests expect: `contains("skipped as unsupported")` AND `contains("obsolete")` → BOTH ABSENT → FAIL
- Classic-wording tests expect: `contains("pcap file")` AND `NOT contains("pcapng file")` → second NOT assertion FAILS because actual output IS "pcapng file"

### PC6 Format Strings Pinned

**Generic segment** (emitted when G = skipped_blocks - opb_skipped > 0):
  `"(G block(s) skipped as unsupported)"`
  Pinned by HS-108 Case B: `"(2 block(s) skipped as unsupported)"`

**OPB clause** (emitted when opb_skipped > 0):
  `"(includes N obsolete Packet Block(s) whose data was not analyzed; re-save with mergecap)"`
  Pinned by BC-2.01.009 PC6 v1.7 canonical text; HS-108 Cases D/E key substrings: "obsolete", "mergecap", count.

**Classic-pcap wording**:
  `"0 packets read from pcap file"` (NOT "pcapng file")
  Pinned by BC-2.01.009 PC6 EC-009 / ADR-009 Decision 19 rev 8.

**Gate suppression** (when both G==0 and opb_skipped==0):
  Notice is bare: `"notice: <file>: 0 packets read from pcapng file"` with NO parenthetical.
  Pinned by BC-2.01.009 EC-010 / HS-108 Case F.

### Fixtures Built

| Fixture Function | File Contents | skipped_blocks | opb_skipped | G |
|-----------------|---------------|---------------|-------------|---|
| `shb_idb_one_opb_bytes()` | SHB + IDB + 1 OPB (32 bytes) | 1 | 1 | 0 |
| `shb_idb_two_unknown_blocks_bytes()` | SHB + IDB + 2 unknown (type 0x99) | 2 | 0 | 2 |
| `shb_idb_two_nrb_one_opb_bytes()` | SHB + IDB + 2 NRB + 1 OPB | 3 | 1 | 2 |
| `empty_classic_pcap_bytes()` | 24-byte global header only (magic 0xA1B2C3D4 LE) | n/a | n/a | n/a |

OPB wire layout: type=0x00000002, btl=32, body=20 bytes (interface_id:2 + drops_count:2 + ts_high:4 + ts_low:4 + captured_len:4 + original_len:4), all zeros.
NRB wire layout: type=0x00000004, btl=16, body=4 bytes (record_type:2=0, record_length:2=0).
Unknown block: type=0x00000099, btl=20, body=8 bytes (AA BB CC DD EE FF 00 11).
Classic pcap global header: magic=D4C3B2A1 (LE), version 2.4, thiszone=0, sigfigs=0, snaplen=65535, network=1.

### STORY-123..127 Regression Status

All 116 prior tests remain GREEN:
```
bc_2_01_story123_pcapng_tests → 31 passed; 0 failed
bc_2_01_story124_idb_tests    → 27 passed; 0 failed
bc_2_01_story125_epb_tests    → 20 passed; 0 failed
bc_2_01_story126_spb_tests    → 29 passed; 0 failed
bc_2_12_011_story127_tests    →  9 passed; 0 failed
```
