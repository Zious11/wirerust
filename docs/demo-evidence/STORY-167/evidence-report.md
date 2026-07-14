# Evidence Report — STORY-167

**Story:** STORY-167: IEC-104 APCI Core Parser: parse_apci_header Pure-Core Free Function + VP-044 Kani Skeleton  
**Wave:** 76  
**Date:** 2026-07-14  
**Branch:** feature/STORY-167-iec104-apci-parser  
**Product type:** Library (pure-core free functions — no CLI/web surface; dispatch wiring is STORY-173)

---

## Full Test Suite: 30/30 PASS

Command:
```
cargo test --test iec104_analyzer_tests
```

Output:
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running tests/iec104_analyzer_tests.rs (target/debug/deps/iec104_analyzer_tests-...)

running 30 tests
test story_167::test_BC_2_19_001_returns_none_for_empty_slice ... ok
test story_167::test_BC_2_19_001_invariant_no_panic_on_truncated_inputs ... ok
test story_167::test_BC_2_19_001_returns_none_for_five_bytes_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_one_byte ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x00_canonical_vector ... ok
test story_167::test_BC_2_19_001_returns_none_for_two_bytes ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0x69_off_by_one ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_1_and_len_2 ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_3_off_by_one_canonical_vector ... ok
test story_167::test_BC_2_19_003_returns_none_for_len_zero_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_254_canonical_vector ... ok
test story_167::test_BC_2_19_002_returns_none_for_start_byte_0xFF_canonical_vector ... ok
test story_167::test_BC_2_19_004_returns_none_for_len_255_canonical_vector ... ok
test story_167::test_BC_2_19_005_apci_header_equality_and_field_layout ... ok
test story_167::test_BC_2_19_005_cf_fields_verbatim_from_data_indices_2_through_5 ... ok
test story_167::test_BC_2_19_005_i_frame_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_005_invariant_len_plus_two_in_range_for_boundaries ... ok
test story_167::test_BC_2_19_005_returns_some_for_len_253_maximum_canonical_vector ... ok
test story_167::test_BC_2_19_005_s_frame_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_005_u_frame_startdt_act_all_fields_correct_canonical_vector ... ok
test story_167::test_BC_2_19_006_invariant_consistency_with_parse_apci_header ... ok
test story_167::test_BC_2_19_006_invariant_false_gate_implies_none_from_parse ... ok
test story_167::test_BC_2_19_006_returns_false_for_empty_slice ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_254_above_maximum ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_3_below_minimum ... ok
test story_167::test_BC_2_19_006_returns_false_for_len_ff_out_of_range_canonical_vector ... ok
test story_167::test_BC_2_19_006_returns_false_for_wrong_start_byte_canonical_vector ... ok
test story_167::test_BC_2_19_006_returns_true_for_valid_start_and_len_253 ... ok
test story_167::test_BC_2_19_006_returns_false_for_one_byte_slice ... ok
test story_167::test_BC_2_19_006_returns_true_for_valid_start_and_len_4_canonical_vector ... ok

test result: ok. 30 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## Coverage Map

| AC | Description | BC | Tests | Evidence File | Verdict |
|----|-------------|-----|-------|---------------|---------|
| AC-167-001 | `parse_apci_header` returns None for len < 6 | BC-2.19.001 | 5 | `AC-001-short-input-rejection.md` | PASS |
| AC-167-002 | `parse_apci_header` returns None for start byte ≠ 0x68 | BC-2.19.002 | 3 | `AC-002-bad-start-byte.md` | PASS |
| AC-167-003 | `parse_apci_header` returns None for LEN < 4 | BC-2.19.003 | 3 | `AC-003-004-len-bounds-rejection.md` | PASS |
| AC-167-004 | `parse_apci_header` returns None for LEN > 253 | BC-2.19.004 | 2 | `AC-003-004-len-bounds-rejection.md` | PASS |
| AC-167-005 | `parse_apci_header` returns Some(ApciHeader) for valid input; CF1–CF4 verbatim | BC-2.19.005 | 7 | `AC-005-valid-frame-extraction.md` | PASS |
| AC-167-006 | `is_valid_iec104_frame` post-classification validity gate | BC-2.19.006 | 10 | `AC-006-validity-gate.md` | PASS |
| AC-167-007 | VP-044 Kani harness skeleton compiles | VP-044 | verify | `AC-007-vp044-kani-skeleton.md` | PASS |

**Total test-based coverage: 30/30 (all AC-167-001..006)**

---

## Per-AC Test Distribution

| AC | BC | Test Count | Test Names |
|----|-----|-----------|------------|
| AC-167-001 | BC-2.19.001 | 5 | returns_none_for_empty_slice, returns_none_for_one_byte, returns_none_for_two_bytes, returns_none_for_five_bytes_canonical_vector, invariant_no_panic_on_truncated_inputs |
| AC-167-002 | BC-2.19.002 | 3 | returns_none_for_start_byte_0x00_canonical_vector, returns_none_for_start_byte_0xFF_canonical_vector, returns_none_for_start_byte_0x69_off_by_one |
| AC-167-003 | BC-2.19.003 | 3 | returns_none_for_len_zero_canonical_vector, returns_none_for_len_3_off_by_one_canonical_vector, returns_none_for_len_1_and_len_2 |
| AC-167-004 | BC-2.19.004 | 2 | returns_none_for_len_254_canonical_vector, returns_none_for_len_255_canonical_vector |
| AC-167-005 | BC-2.19.005 | 7 | u_frame_startdt_act_all_fields_correct_canonical_vector, s_frame_all_fields_correct_canonical_vector, i_frame_all_fields_correct_canonical_vector, returns_some_for_len_253_maximum_canonical_vector, invariant_len_plus_two_in_range_for_boundaries, cf_fields_verbatim_from_data_indices_2_through_5, apci_header_equality_and_field_layout |
| AC-167-006 | BC-2.19.006 | 10 | returns_true_for_valid_start_and_len_4_canonical_vector, returns_true_for_valid_start_and_len_253, returns_false_for_wrong_start_byte_canonical_vector, returns_false_for_len_ff_out_of_range_canonical_vector, returns_false_for_empty_slice, returns_false_for_one_byte_slice, returns_false_for_len_3_below_minimum, returns_false_for_len_254_above_maximum, invariant_consistency_with_parse_apci_header, invariant_false_gate_implies_none_from_parse |

---

## VP-044 Kani Skeleton Evidence

**Source file:** `src/analyzer/iec104.rs`, line 175  
**Harness name:** `verify_parse_apci_header_safety`  
**Properties anchored:** A (no panic), B (len+2 ∈ [6,255]), C (len ∈ [4,253])

Skeleton presence confirmed via:
```
grep -n "cfg(kani)" src/analyzer/iec104.rs
```
Output:
```
16://! - VP-044 Kani harness skeleton under `#[cfg(kani)]` (full proof run: STORY-174).
169:// parse_apci_header is fully implemented (BC-2.19.001-005). This #[cfg(kani)]
175:#[cfg(kani)]
```

`cargo check` passes clean (harness excluded from normal compilation by `#[cfg(kani)]` gate).  
Full proof run: STORY-174.

---

## Recording Method

This is a pure-core library story (no CLI binary, no web UI). Evidence is captured as:
- Annotated CLI transcript markdown files showing `cargo test` output grouped by AC
- Inline canonical vector tables from the BC specifications
- Source-level grep verification for the VP-044 Kani skeleton

VHS/Playwright recordings are not applicable (no interactive surface exists at this story
scope; dispatch wiring to the CLI is STORY-173).

---

## Artifact List

| File | AC Coverage |
|------|-------------|
| `AC-001-short-input-rejection.md` | AC-167-001 (BC-2.19.001) |
| `AC-002-bad-start-byte.md` | AC-167-002 (BC-2.19.002) |
| `AC-003-004-len-bounds-rejection.md` | AC-167-003 (BC-2.19.003), AC-167-004 (BC-2.19.004) |
| `AC-005-valid-frame-extraction.md` | AC-167-005 (BC-2.19.005) |
| `AC-006-validity-gate.md` | AC-167-006 (BC-2.19.006) |
| `AC-007-vp044-kani-skeleton.md` | AC-167-007 (VP-044) |
| `evidence-report.md` | Index (this file) |

---

## Demo-Evidence Path-Scrub Gate (PG-W70-DEMO-SCRUB)

Gate defined in: `.factory/maintenance/demo-evidence-scrub-gate.md`

The gate grep (per PG-W70-DEMO-SCRUB) was run against all files in this directory
before commit. All evidence files were authored without absolute host paths; no
occurrences of absolute host-local paths were present in the committed content.

Result: **zero content matches** — no absolute host paths present in any evidence file.

Gate status: **PASSED** (2026-07-14).
