# Red Gate Log — STORY-015

**Timestamp:** 2026-05-25  
**Agent:** test-writer  
**Story:** STORY-015 — In-Order Delivery, Out-of-Order Buffering, and Bidirectional Direction Tagging

## Phase 1: Anchor Verification

Story claimed:
- `src/reassembly/segment.rs:31-248` (seq_offset, insert_segment, flush_contiguous)
- `src/reassembly/mod.rs:517-533` (flush_contiguous_data with on_data callback)

**Actual anchors:**
- `segment.rs:32` — `fn seq_offset(seq: u32, isn: u32) -> u64`
- `segment.rs:39` — `pub fn insert_segment(...)`
- `segment.rs:236` — `pub fn flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)>`
- `mod.rs:517` — `fn flush_contiguous_data(...)`

**Result: NO DRIFT.** All anchors within the claimed ranges. Lines 31-248 and 517-533 are accurate.

## Phase 2: Red Gate Results

### Stub Phase

26 stub test functions added across two files:
- `tests/reassembly_segment_tests.rs`: 8 stubs (AC-011..AC-017 + AC-008 proptest)
- `tests/reassembly_engine_tests.rs`: 16 stubs (AC-001..AC-010 + 7 EC tests)

**`cargo test --all-targets --no-run`:** SUCCEEDED (all stubs compiled)

**Running stubs:** ALL FAILED with "STORY-015 stub — Red Gate" as required.

Red Gate: VERIFIED.

## Phase 3: Real Test Bodies

All 26 stubs replaced with BC-anchored assertions. Two test failures encountered during development:

1. **`test_BC_2_04_006_on_data_offset_is_isn_relative`**: Initial design used seq=1003 with ISN=1000, creating a gap (base_offset=1, segment at offset=3). Fixed: use seq=1001 (offset=1, contiguous). BC-2.04.006 PC3 correctly asserts offset==1 not raw seq==1001.

2. **`test_BC_2_04_006_directions_are_independent`**: Initial design used mid-stream ISN inference for s2c, which caused the OOO segment to be immediately contiguous. Fixed: use full SYN/SYN-ACK handshake (both ISNs set), then insert OOO segments in both directions at non-base offsets.

## Final Test Results

| File | Tests Added | Passing |
|------|-------------|---------|
| `tests/reassembly_segment_tests.rs` | 8 | 8 |
| `tests/reassembly_engine_tests.rs` | 16 | 16 |
| **Total** | **24** | **24** |

Plus 1 proptest function (`test_BC_2_04_007_base_offset_is_monotonic`) that executes 256 random cases per run.

**`cargo test --all-targets`:** All tests pass (0 failures).  
**`cargo clippy --all-targets -- -D warnings`:** Clean.  
**`cargo fmt --check`:** Clean.

## BC Coverage

| BC | ACs Covered | Tests |
|----|-------------|-------|
| BC-2.04.006 | AC-001, AC-002, AC-003, AC-004, AC-005 | 6 engine tests |
| BC-2.04.007 | AC-006, AC-007, AC-008 | 1 engine test + 1 engine test + 1 proptest |
| BC-2.04.008 | AC-009, AC-010 | 2 engine tests |
| BC-2.04.033 | AC-011, AC-012 | 2 segment tests |
| BC-2.04.034 | AC-013, AC-014, AC-015 | 3 segment tests |
| BC-2.04.039 | AC-016, AC-017 | 2 segment tests |

## EC Coverage

| EC | Test |
|----|------|
| EC-001 | `test_BC_2_04_007_ec001_in_order_no_buffering` |
| EC-002 | `test_BC_2_04_007_ec002_gap_stops_flush` |
| EC-003 | Covered by `test_BC_2_04_008_gap_fill_delivers_all_contiguous` |
| EC-004 | `test_BC_2_04_006_ec004_empty_payload_not_inserted` |
| EC-005 | `test_BC_2_04_007_ec005_multiple_contiguous_delivered_separately` |
| EC-006 | `test_BC_2_04_008_ec006_three_segment_ooo_321` |
| EC-007 | Covered by `test_BC_2_04_007_gap_halts_flush` |
| EC-008 | `test_BC_2_04_039_ec008_isn_near_max_btreemap_keys_monotonic` |
| EC-009 | `test_BC_2_04_034_ec009_flush_empty_btreemap_no_change` |

## VP Coverage

| VP | Test |
|----|------|
| VP-011 | `test_BC_2_04_007_base_offset_is_monotonic` (proptest, 256 random cases) |
| VP-015 | `test_BC_2_04_039_sequence_wraparound_correct_offsets`, `test_BC_2_04_039_flush_delivers_wrapped_segments_in_order` |

## No src/ Modifications Required

This is a brownfield-formalization story. All tests exercise existing implementation via the public API. No new src/ accessors were needed.

## Status: DONE

Handed off to implementer: all tests pass against existing implementation. No regressions. No missing BC coverage.
