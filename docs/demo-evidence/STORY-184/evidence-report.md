# Evidence Report — STORY-184

**Story:** STORY-184: S7comm TPKT Core Parser: `parse_tpkt_header` Pure-Core Free
Function + VP-048 Kani Skeleton
**Wave:** 87
**Date:** 2026-09-06
**Branch:** feature/STORY-184-tpkt-header-parser
**Product type:** Library (pure-core free function — no CLI/web surface; the S7comm
dispatch wiring that consumes this module is a later story, mirroring STORY-173's role
for STORY-167's IEC-104 APCI parser)

---

## Full Test Suite: 30/30 PASS

Command:
```
cargo test --test iso_on_tcp_tests
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iso_on_tcp_tests.rs (target/debug/deps/iso_on_tcp_tests-...)

running 30 tests
test story_184::proptests::test_BC_2_20_004_proptest_accepted_length_matches_decoded_bytes ... ok
test story_184::proptests::test_BC_2_20_004_proptest_matches_independent_oracle ... ok
test story_184::test_BC_2_20_001_invariant_no_panic_on_truncated_inputs ... ok
test story_184::test_BC_2_20_001_returns_none_for_empty_slice ... ok
test story_184::test_BC_2_20_001_returns_none_for_one_byte ... ok
test story_184::test_BC_2_20_001_returns_none_for_three_bytes_canonical_vector ... ok
test story_184::test_BC_2_20_001_returns_none_for_two_bytes ... ok
test story_184::test_BC_2_20_002_bad_version_short_circuits_before_length_decode ... ok
test story_184::test_BC_2_20_002_invariant_no_panic_across_version_byte_sample ... ok
test story_184::test_BC_2_20_002_returns_none_for_version_0x00_canonical_vector ... ok
test story_184::test_BC_2_20_002_returns_none_for_version_0x04_off_by_one_canonical_vector ... ok
test story_184::test_BC_2_20_002_returns_none_for_version_0xFF_canonical_vector ... ok
test story_184::test_BC_2_20_003_invariant_no_panic_across_sub_minimum_lengths ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_five_below_rfc_minimum ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_four_below_rfc_minimum ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_one_canonical_vector ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_six_boundary_below_rfc_minimum ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_three_off_by_one_canonical_vector ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_two ... ok
test story_184::test_BC_2_20_003_returns_none_for_length_zero_canonical_vector ... ok
test story_184::test_BC_2_20_004_exact_length_match_no_trailing_bytes ... ok
test story_184::test_BC_2_20_004_four_way_partition_is_exhaustive ... ok
test story_184::test_BC_2_20_004_reserved_byte_nonzero_parses_identically_to_zero ... ok
test story_184::test_BC_2_20_004_trailing_bytes_beyond_declared_length_still_accepted_canonical_vector ... ok
test story_184::test_BC_2_20_004_valid_input_returns_some_header_length_65535_max_canonical_vector ... ok
test story_184::test_BC_2_20_004_valid_input_returns_some_header_length_7_canonical_vector ... ok
test story_184::test_rfc1006_s6_length_four_below_minimum_returns_none ... ok
test story_184::test_rfc1006_s6_minimum_valid_length_holdout ... ok
test story_184::test_rfc1006_s6_ten_byte_tpkt_holdout ... ok
test story_184::test_rfc1006_s6_wide_length_field_holdout ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s
```

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|----------------|---------|
| AC-184-001 | `parse_tpkt_header` returns None for input shorter than 4 bytes | BC-2.20.001 | 5 | `AC-001-short-input-rejection.md` | PASS |
| AC-184-002 | `parse_tpkt_header` returns None for version byte != 0x03 | BC-2.20.002 | 5 | `AC-002-bad-version-byte.md` | PASS |
| AC-184-003 | `parse_tpkt_header` returns None for decoded length < 7 (RFC 1006 §6 minimum) | BC-2.20.003 | 9 | `AC-003-length-floor-rejection.md` | PASS |
| AC-184-004 | `parse_tpkt_header` returns Some(TpktHeader) for valid input; length in [7, 65535] exact, reserved byte ignored | BC-2.20.004 | 9 | `AC-004-valid-accept-path.md` | PASS |
| AC-184-005 | The four `parse_tpkt_header` outcomes are jointly exhaustive and mutually exclusive | BC-2.20.004 invariant 3 | 2 | `AC-005-four-way-partition.md` | PASS |
| AC-184-006 | VP-048 Kani harness skeleton compiles | VP-048 | verify (grep + `cargo check` + `cargo clippy`) | `AC-006-vp048-kani-skeleton.md` | PASS |

**Total test-based coverage: 30/30 (all AC-184-001..005); AC-184-006 verified by
source-level inspection (no cargo test target — full Kani proof execution is
STORY-194's obligation, not counted against the 30).**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Test Names |
|----|-----|-----------|------------|
| AC-184-001 | BC-2.20.001 | 5 | returns_none_for_empty_slice, returns_none_for_one_byte, returns_none_for_two_bytes, returns_none_for_three_bytes_canonical_vector, invariant_no_panic_on_truncated_inputs |
| AC-184-002 | BC-2.20.002 | 5 | returns_none_for_version_0x00_canonical_vector, returns_none_for_version_0x04_off_by_one_canonical_vector, returns_none_for_version_0xFF_canonical_vector, bad_version_short_circuits_before_length_decode, invariant_no_panic_across_version_byte_sample |
| AC-184-003 | BC-2.20.003 | 9 | returns_none_for_length_zero_canonical_vector, returns_none_for_length_one_canonical_vector, returns_none_for_length_two, returns_none_for_length_three_off_by_one_canonical_vector, returns_none_for_length_four_below_rfc_minimum, returns_none_for_length_five_below_rfc_minimum, returns_none_for_length_six_boundary_below_rfc_minimum, invariant_no_panic_across_sub_minimum_lengths, test_rfc1006_s6_length_four_below_minimum_returns_none (independent holdout) |
| AC-184-004 | BC-2.20.004 | 9 | valid_input_returns_some_header_length_7_canonical_vector, valid_input_returns_some_header_length_65535_max_canonical_vector, reserved_byte_nonzero_parses_identically_to_zero, exact_length_match_no_trailing_bytes, trailing_bytes_beyond_declared_length_still_accepted_canonical_vector, test_rfc1006_s6_minimum_valid_length_holdout, test_rfc1006_s6_ten_byte_tpkt_holdout, test_rfc1006_s6_wide_length_field_holdout (independent holdouts), proptest_accepted_length_matches_decoded_bytes |
| AC-184-005 | BC-2.20.004 invariant 3 | 2 | four_way_partition_is_exhaustive, proptest_matches_independent_oracle |
| AC-184-006 | VP-048 | 0 (source-level verification) | N/A — see `AC-006-vp048-kani-skeleton.md` |

**Test-count cross-check:** 5 + 5 + 9 + 9 + 2 = 30, matching the full `cargo test
--test iso_on_tcp_tests` run above exactly (test-by-test row-verified against the
raw test-runner output, not just the aggregate count).

---

## VP-048 Kani Skeleton Evidence

**Source file:** `src/analyzer/iso_on_tcp.rs`, line 145
**Harness name:** `verify_parse_tpkt_header_safety`
**Property anchored:** A (no panic for any symbolic input, `len <= 300`)

Skeleton presence confirmed via:
```
grep -n "cfg(kani)" src/analyzer/iso_on_tcp.rs
```
Output:
```
108:/// `#[cfg(kani)]` skeleton below is scoped to check only no-panic/bounds-safety over
145:#[cfg(kani)]
```

`cargo check` and `cargo clippy --all-targets -- -D warnings` both pass clean (harness
excluded from normal compilation by the `#[cfg(kani)]` gate). Full proof run,
including the AC-184-005 exhaustiveness assertions: STORY-194.

---

## Recording Method

This is a pure-core library story (no CLI binary, no web UI — the analyzer is not yet
wired to the CLI; `s7comm.rs` / SS-21 dispatch wiring is a later story per ADR-014
Decision 1). Per the demo-recording skill's library/test-harness mode, evidence is
captured as:
- Annotated `cargo test` output transcripts grouped by AC, matching the STORY-167
  (IEC-104 `parse_apci_header`) precedent this story's shape mirrors
- Inline canonical-vector tables sourced from the BC-2.20.001-004 specifications
- Source-level grep verification plus `cargo check`/`cargo clippy` output for the
  VP-048 Kani skeleton (AC-184-006)

VHS/Playwright recordings are not applicable — no interactive surface exists at this
story's scope.

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-short-input-rejection.md` | AC-184-001 (BC-2.20.001) |
| `AC-002-bad-version-byte.md` | AC-184-002 (BC-2.20.002) |
| `AC-003-length-floor-rejection.md` | AC-184-003 (BC-2.20.003) |
| `AC-004-valid-accept-path.md` | AC-184-004 (BC-2.20.004) |
| `AC-005-four-way-partition.md` | AC-184-005 (BC-2.20.004 invariant 3) |
| `AC-006-vp048-kani-skeleton.md` | AC-184-006 (VP-048) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files in this directory were authored using repo-relative
paths only; any
`cargo check`/`cargo clippy` output line naming the crate root was rewritten to the
`<repo>` placeholder before inclusion, per the gate's documented pattern table.

Result: **zero content matches** — no absolute host paths present in any evidence file
in this directory.

Gate status: **PASSED** (2026-09-06).
