use wirerust::reassembly::flow::FlowDirection;
use wirerust::reassembly::segment::InsertResult;

// =============================================================================
// PROCESS-GLOBAL ATOMIC INTERACTION (Wave 9 wave-level F-W9P1-004 fix)
//
// This integration test binary is COMPILED SEPARATELY from
// reassembly_engine_tests.rs. As a result, the process-global atomics in
// src/reassembly/segment.rs (e.g., ISN_MISSING_WARNED at segment.rs:16
// (declaration; swap site at :54)) and src/reassembly/lifecycle.rs (e.g.,
// CLOSE_FLOW_MISSING_WARNED at lifecycle.rs:31 (declaration; swap site at
// :44)) are SEPARATE INSTANCES per test binary — tests in THIS
// binary do not race against tests in reassembly_engine_tests.rs.
//
// HOWEVER, the sibling-discipline doctrine established in STORY-014
// (reassembly_engine_tests.rs lines 10-26) requires explicit lock acquisition
// when ANY test reads or asserts on the atomic. The tests in this file
// (STORY-015 + STORY-016) currently trigger IsnMissing in a few places but do
// NOT INSPECT the atomic — only the return value is asserted. So the lock is
// not required today.
//
// CONTRACT FOR FUTURE TESTS IN THIS FILE: any new test that reads
// `isn_missing_warned_for_testing()` or calls
// `reset_isn_missing_warned_for_testing()` MUST add an
// `ISN_MISSING_WARNED_LOCK: Mutex<()>` to this file (modeled on
// reassembly_engine_tests.rs lines 10-26) and acquire it as the FIRST line of
// the test body. Failure to do so re-introduces the same intra-binary race
// that STORY-014 documented.
// =============================================================================

#[test]
fn test_insert_single_segment() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(dir.segment_count(), 1);
    assert_eq!(dir.segment_at(1), Some(b"hello".as_slice()));
}

#[test]
fn test_flush_contiguous_single() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);

    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].0, 1); // offset
    assert_eq!(flushed[0].1, b"hello");
    assert_eq!(dir.base_offset, 6); // 1 + 5
    assert_eq!(dir.reassembled_bytes, 5);
    assert!(dir.segments_is_empty());
}

#[test]
fn test_flush_contiguous_ordered() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"aaa", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1004, b"bbb", 10_485_760, 10_000, 10_485_760);

    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 2);
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(flushed[1].1, b"bbb");
    assert_eq!(dir.base_offset, 7); // 1 + 3 + 3
    assert!(dir.segments_is_empty());
}

#[test]
fn test_out_of_order_buffering() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment 2 first (out of order)
    dir.insert_segment(1004, b"bbb", 10_485_760, 10_000, 10_485_760);
    let flushed = dir.flush_contiguous();
    assert!(flushed.is_empty()); // Can't flush — gap at offset 1

    // Now insert segment 1
    dir.insert_segment(1001, b"aaa", 10_485_760, 10_000, 10_485_760);
    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 2); // Both flush now
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(flushed[1].1, b"bbb");
    assert_eq!(dir.base_offset, 7);
}

#[test]
fn test_retransmission_dedup() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Duplicate);
    assert_eq!(dir.segment_count(), 1); // No duplicate stored
    assert_eq!(dir.buffered_bytes(), 5); // counter must not double-count
}

#[test]
fn test_overlap_first_wins() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert "AAABBB" at offset 1
    dir.insert_segment(1001, b"AAABBB", 10_485_760, 10_000, 10_485_760);

    // Overlapping insert: "XXXCC" at offset 4 (overlaps with "BBB" at 4-6)
    let result = dir.insert_segment(1004, b"XXXCC", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Flush and verify: first 6 bytes from original, then "CC" from new
    let flushed = dir.flush_contiguous();
    let all_bytes: Vec<u8> = flushed
        .iter()
        .flat_map(|(_, data)| data.iter().copied())
        .collect();
    assert_eq!(&all_bytes, b"AAABBBCC");
}

#[test]
fn test_overlap_conflicting_data_detected() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAA", 10_485_760, 10_000, 10_485_760);

    // Same range, different data
    let result = dir.insert_segment(1001, b"BBBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::ConflictingOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Original data preserved (first-wins)
    let flushed = dir.flush_contiguous();
    assert_eq!(flushed[0].1, b"AAAA");
}

#[test]
fn test_sequence_wraparound() {
    let mut dir = FlowDirection::new();
    // ISN near wraparound
    dir.set_isn(0xFFFF_FFF0);

    // First data byte at ISN+1 = 0xFFFF_FFF1, offset = 1
    dir.insert_segment(0xFFFF_FFF1, b"before", 10_485_760, 10_000, 10_485_760);
    // Next segment wraps: seq = 0xFFFF_FFF1 + 6 = 0xFFFF_FFF7, offset = 7
    dir.insert_segment(0xFFFF_FFF7, b"wrap", 10_485_760, 10_000, 10_485_760);
    // Another after wrap: seq = 0xFFFF_FFFB, offset = 11
    dir.insert_segment(0xFFFF_FFFB, b"around", 10_485_760, 10_000, 10_485_760);

    let flushed = dir.flush_contiguous();
    let all_bytes: Vec<u8> = flushed
        .iter()
        .flat_map(|(_, data)| data.iter().copied())
        .collect();
    assert_eq!(&all_bytes, b"beforewraparound");
}

// `test_small_segment_tracking` was removed in the LESSON-P2.05
// consecutive-run change: small-segment classification moved out of the
// segment buffer into the engine (`insert_payload_segment`), so it is
// now covered by the engine-level tests in `reassembly_engine_tests.rs`
// (`test_consecutive_small_segments_trip_anomaly` and
// `test_normal_segment_resets_small_segment_run`).

#[test]
fn test_buffered_bytes_after_insert() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);
    dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes(), 5);
    dir.insert_segment(1006, b"world", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes(), 10);
}

#[test]
fn test_buffered_bytes_after_overlap() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);
    dir.insert_segment(1001, b"AAABBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes(), 6);
    dir.insert_segment(1004, b"XXXCC", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes(), 8); // 6 original + 2 gap bytes
}

#[test]
fn test_buffered_bytes_after_flush() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes(), 5);

    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 1);
    assert_eq!(dir.buffered_bytes(), 0);
}

#[test]
fn test_buffered_bytes_partial_flush() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 1 (contiguous) and offset 10 (gap)
    dir.insert_segment(1001, b"aaa", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1010, b"bbb", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes(), 6);

    // Flush only flushes contiguous segment at offset 1
    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(dir.buffered_bytes(), 3); // "bbb" remains buffered
}

#[test]
fn test_depth_limit_truncation() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 100; // small for testing
    let data = vec![b'A'; 80];
    dir.insert_segment(1001, &data, max_depth, 10_000, 10_485_760);
    dir.flush_contiguous();
    assert_eq!(dir.reassembled_bytes, 80);
    assert!(!dir.depth_exceeded);

    // This should be truncated to 20 bytes
    let data2 = vec![b'B'; 50];
    let result = dir.insert_segment(1081, &data2, max_depth, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Truncated);
    assert!(dir.depth_exceeded);

    let flushed = dir.flush_contiguous();
    assert_eq!(flushed[0].1.len(), 20); // truncated from 50 to 20
    assert_eq!(dir.reassembled_bytes, 100);
}

#[test]
fn test_overlap_detection_boundary() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 1, length 5 (covers 1-5)
    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    // Insert segment at offset 10, length 5 (covers 10-14) — no overlap with above
    dir.insert_segment(1010, b"BBBBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.segment_count(), 2);
    assert_eq!(dir.overlap_count, 0);

    // Insert segment at offset 3, length 4 (covers 3-6) — overlaps first, not second
    let result = dir.insert_segment(1003, b"XXXX", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Insert segment at offset 6, length 4 (covers 6-9).
    // The partial-overlap insert above deposited a 1-byte gap at offset 6 ("X"),
    // so this segment overlaps that byte and is PartialOverlap.
    let result = dir.insert_segment(1006, b"CCCC", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(dir.overlap_count, 2);
}

#[test]
fn test_range_boundary_exact_new_end() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 1, length 5 (covers 1-5, ends at 6)
    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);

    // Insert segment starting exactly at the end of the first (offset 6)
    // This should NOT overlap — range(..new_end) must exclude it
    let result = dir.insert_segment(1006, b"BBBBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(dir.overlap_count, 0);
}

#[test]
fn test_out_of_window_segment_rejected() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert normal segment at offset 1 (within any window)
    let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 1_048_576);
    assert_eq!(result, InsertResult::Inserted);

    dir.flush_contiguous(); // base_offset now 6

    // Insert segment far beyond window: base_offset=6 + 1MB + 100 = way out of window
    let far_seq = 1000 + 6 + 1_048_576 + 100; // ISN + base_offset + window + 100
    let result = dir.insert_segment(far_seq as u32, b"evil", 10_485_760, 10_000, 1_048_576);
    assert_eq!(result, InsertResult::OutOfWindow);

    // Segment exactly one byte beyond window should be rejected (off-by-one check)
    let one_past_seq = 1000 + 6 + 1_048_576 + 1; // ISN + base_offset + window + 1
    let result = dir.insert_segment(one_past_seq as u32, b"x", 10_485_760, 10_000, 1_048_576);
    assert_eq!(result, InsertResult::OutOfWindow);

    // Segment exactly at window boundary should be accepted
    let edge_seq = 1000 + 6 + 1_048_576; // ISN + base_offset + window (exactly at boundary)
    let result = dir.insert_segment(edge_seq as u32, b"edge", 10_485_760, 10_000, 1_048_576);
    assert_eq!(result, InsertResult::Inserted);
}

#[test]
fn test_multi_segment_full_coverage_returns_duplicate() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert two adjacent segments: "AAA" at offset 1, "BBB" at offset 4
    dir.insert_segment(1001, b"AAA", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1004, b"BBB", 10_485_760, 10_000, 10_485_760);

    // Insert segment spanning both: "AAABBB" at offset 1
    // Union of existing segments fully covers this — should be Duplicate
    let result = dir.insert_segment(1001, b"AAABBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Duplicate);
}

#[test]
fn test_multi_segment_full_coverage_conflicting_returns_conflict() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert two adjacent segments: "AAA" at offset 1, "BBB" at offset 4
    dir.insert_segment(1001, b"AAA", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1004, b"BBB", 10_485_760, 10_000, 10_485_760);

    // Insert segment spanning both with different data
    // Union covers it but data conflicts — should be ConflictingOverlap
    let result = dir.insert_segment(1001, b"XXXXXX", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::ConflictingOverlap);
}

#[test]
fn test_segment_limit_non_overlap_path() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_segments: usize = 3;

    // Fill up to the segment limit
    dir.insert_segment(1001, b"aaa", 10_485_760, max_segments, 10_485_760);
    dir.insert_segment(1010, b"bbb", 10_485_760, max_segments, 10_485_760);
    dir.insert_segment(1020, b"ccc", 10_485_760, max_segments, 10_485_760);
    assert_eq!(dir.segment_count(), 3);

    // Next non-overlapping insert should return SegmentLimitReached
    let result = dir.insert_segment(1030, b"ddd", 10_485_760, max_segments, 10_485_760);
    assert_eq!(result, InsertResult::SegmentLimitReached);
    assert_eq!(dir.segment_count(), 3); // No new segment added
}

#[test]
fn test_segment_limit_gap_loop_full_rejection() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_segments: usize = 2;

    // Insert two segments with a gap between them (offsets 1-3 and 10-12)
    dir.insert_segment(1001, b"AAA", 10_485_760, max_segments, 10_485_760);
    dir.insert_segment(1010, b"BBB", 10_485_760, max_segments, 10_485_760);
    assert_eq!(dir.segment_count(), 2);

    // Now insert a segment that overlaps the first and has a gap to fill (offset 1-6)
    // Gap is at offset 4-6. But segments are at capacity (2), so the gap can't be inserted.
    let result = dir.insert_segment(1001, b"AAAXXX", 10_485_760, max_segments, 10_485_760);
    assert_eq!(result, InsertResult::SegmentLimitReached);
    assert_eq!(dir.segment_count(), 2); // No new segment added
}

#[test]
fn test_segment_limit_gap_loop_partial_insertion() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_segments: usize = 3;

    // Insert two segments: offset 1-3 and offset 10-12, leaving gaps at 4-9 and 13+
    dir.insert_segment(1001, b"AAA", 10_485_760, max_segments, 10_485_760);
    dir.insert_segment(1010, b"BBB", 10_485_760, max_segments, 10_485_760);
    assert_eq!(dir.segment_count(), 2);

    // Insert a 15-byte segment spanning offsets 1-15; it overlaps both existing
    // segments and leaves two insertable gaps: [4,10) and [13,16).
    // Only 1 segment slot available (limit=3, currently 2), so the first gap is
    // inserted and the second gap is dropped when the segment limit is reached.
    let result = dir.insert_segment(
        1001,
        b"AAABBBBBBBBBBCC",
        10_485_760,
        max_segments,
        10_485_760,
    );
    // Segment limit was hit (even though some data was inserted)
    assert_eq!(result, InsertResult::SegmentLimitReached);
    assert_eq!(dir.segment_count(), 3); // One gap inserted, hit limit before second

    // Verify the first gap was filled at offset 4 with 6 bytes covering [4,10)
    assert!(dir.has_segment_at(4));
    // Second gap (starting at offset 13) was NOT inserted
    assert!(!dir.has_segment_at(13));
}

#[test]
fn test_isn_missing_returns_isn_missing() {
    let mut dir = FlowDirection::new();
    // Deliberately do NOT call dir.set_isn() — ISN is None

    let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::IsnMissing);
    assert!(dir.segments_is_empty()); // No data inserted
    assert_eq!(dir.buffered_bytes(), 0);
}

// =============================================================================
// STORY-015: BC-2.04.033 — Single Segment Insertion
// =============================================================================

/// BC-2.04.033 PC1–PC2: Single non-overlapping, in-window segment insertion
/// returns Inserted and stores the segment at its ISN-relative offset.
/// Canonical vector: ISN=1000, seq=1001, data=b"hello" → offset=1, result=Inserted.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_033_single_segment_insert_returns_inserted() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Inserted,
        "BC-2.04.033 PC1: insert_segment must return Inserted for a clean, non-overlapping segment"
    );
    // BC-2.04.033 PC2: stored under ISN-relative offset key = seq - isn = 1.
    assert_eq!(
        dir.segment_at(1),
        Some(b"hello".as_slice()),
        "BC-2.04.033 PC2: segment must be stored at ISN-relative offset 1"
    );
    assert_eq!(
        dir.segment_count(),
        1,
        "BC-2.04.033 PC2: exactly one segment in the BTreeMap"
    );
}

/// BC-2.04.033 PC3: buffered_bytes increases by data.len() after a successful
/// single-segment insertion.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_033_buffered_bytes_increments_after_insert() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(999);

    assert_eq!(
        dir.buffered_bytes(),
        0,
        "BC-2.04.033 PC3: buffered_bytes starts at 0"
    );

    // Canonical vector: ISN=999, seq=1000, data=b"AB" → offset=1, buffered_bytes=2.
    dir.insert_segment(1000, b"AB", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        dir.buffered_bytes(),
        2,
        "BC-2.04.033 PC3: buffered_bytes must increase by data.len() == 2 after insert"
    );
}

// =============================================================================
// STORY-015: BC-2.04.034 — flush_contiguous Consumes from base_offset in Order
// =============================================================================

/// BC-2.04.034 PC2–PC3: flush_contiguous() decrements buffered_bytes and
/// advances base_offset by exactly the total flushed bytes.
/// Also verifies reassembled_bytes increments (BC-2.04.034 PC2).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_034_flush_contiguous_accounting() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(0);
    // Insert 5 bytes at offset 1 (seq=1 with ISN=0).
    dir.insert_segment(1, b"hello", 10_485_760, 10_000, 10_485_760);

    let pre_buffered = dir.buffered_bytes();
    let pre_base = dir.base_offset;
    let pre_reassembled = dir.reassembled_bytes;

    assert_eq!(
        pre_buffered, 5,
        "BC-2.04.034 pre-flush: buffered_bytes must be 5"
    );

    let flushed = dir.flush_contiguous();

    assert_eq!(flushed.len(), 1, "BC-2.04.034: exactly one segment flushed");
    assert_eq!(
        dir.buffered_bytes(),
        0,
        "BC-2.04.034 PC2: buffered_bytes must decrement to 0 after flushing all 5 bytes"
    );
    assert_eq!(
        dir.base_offset,
        pre_base + 5,
        "BC-2.04.034 PC2: base_offset must advance by exactly 5 (total flushed bytes)"
    );
    assert_eq!(
        dir.reassembled_bytes,
        pre_reassembled + 5,
        "BC-2.04.034 PC2: reassembled_bytes must increment by 5"
    );
}

/// BC-2.04.034 PC4: When no segment exists at base_offset, flush_contiguous()
/// returns an empty Vec and leaves base_offset unchanged.
/// Canonical vector: segments={5: "XY"}, base_offset=1 (set by set_isn) → gap at offset 1 →
/// [] returned, base_offset stays at 1.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_034_flush_contiguous_empty_when_no_segment_at_base() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(0);
    // Insert segment at offset 5 — leaves gap at offset 1 (base_offset, set by set_isn).
    dir.insert_segment(5, b"XY", 10_485_760, 10_000, 10_485_760);

    let pre_base = dir.base_offset;
    assert_eq!(
        pre_base, 1,
        "BC-2.04.031 PC2: set_isn(0) yields base_offset=1"
    );
    let pre_reassembled = dir.reassembled_bytes;

    let flushed = dir.flush_contiguous();

    assert!(
        flushed.is_empty(),
        "BC-2.04.034 PC4: flush_contiguous must return empty Vec when no segment at base_offset"
    );
    assert_eq!(
        dir.base_offset, pre_base,
        "BC-2.04.034 PC4: base_offset must not change when no segment at base_offset"
    );
    assert_eq!(
        dir.buffered_bytes(),
        2,
        "BC-2.04.034 PC4: buffered segment (\"XY\") must remain in buffer"
    );
    assert_eq!(
        dir.reassembled_bytes, pre_reassembled,
        "BC-2.04.034 PC4: reassembled_bytes unchanged on no-op flush"
    );
}

/// BC-2.04.034 PC3 (ordering): flush_contiguous() returns segments in
/// ascending offset order regardless of insertion order.
/// Discriminating vector: insert at offsets 10, 20, 30 (via ISN-relative seqs)
/// in insertion order 30, 10, 20; flush must return [(10,"A"),(20,"B"),(30,"C")].
/// A HashMap or insertion-order store would return a different order.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_034_flush_contiguous_returns_ordered_segments() {
    let mut dir2 = wirerust::reassembly::flow::FlowDirection::new();
    dir2.set_isn(9);
    // Offsets: seq=12 → 3, seq=10 → 1, seq=11 → 2. Insert out-of-order.
    dir2.insert_segment(12, b"C", 10_485_760, 10_000, 10_485_760);
    dir2.insert_segment(10, b"A", 10_485_760, 10_000, 10_485_760);
    dir2.insert_segment(11, b"B", 10_485_760, 10_000, 10_485_760);

    let flushed = dir2.flush_contiguous();

    // All three segments are contiguous starting from base_offset=1; all flushed.
    assert_eq!(
        flushed.len(),
        3,
        "BC-2.04.034 PC3: three contiguous segments must all be flushed"
    );
    assert_eq!(
        flushed[0].0, 1,
        "BC-2.04.034 PC3: first flushed segment must have offset 1 (ascending order)"
    );
    assert_eq!(
        flushed[0].1, b"A",
        "BC-2.04.034 PC3: first flushed data must be 'A' (offset 1)"
    );
    assert_eq!(
        flushed[1].0, 2,
        "BC-2.04.034 PC3: second flushed segment must have offset 2"
    );
    assert_eq!(
        flushed[1].1, b"B",
        "BC-2.04.034 PC3: second flushed data must be 'B' (offset 2)"
    );
    assert_eq!(
        flushed[2].0, 3,
        "BC-2.04.034 PC3: third flushed segment must have offset 3"
    );
    assert_eq!(
        flushed[2].1, b"C",
        "BC-2.04.034 PC3: third flushed data must be 'C' (offset 3)"
    );
}

// =============================================================================
// STORY-015: BC-2.04.039 — TCP Sequence Wraparound
// =============================================================================

/// BC-2.04.039 PC1: seq.wrapping_sub(isn) as u64 correctly computes the
/// monotonically-increasing byte offset even when the TCP sequence number
/// wraps around u32::MAX.
/// Discriminating vector: ISN=u32::MAX-2; seq=u32::MAX-1 → offset=1;
/// seq=0 (wrapped) → offset=3; seq=2 → offset=5.
/// A regression to plain seq-isn arithmetic (without wrapping_sub) would produce
/// offsets near u64::MAX for the wrapped values.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_039_sequence_wraparound_correct_offsets() {
    let isn: u32 = u32::MAX - 2;
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(isn);

    // seq = ISN+1 = u32::MAX-1 → offset = 1
    let r1 = dir.insert_segment(u32::MAX - 1, b"A", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        r1,
        InsertResult::Inserted,
        "BC-2.04.039 PC1: first segment must insert"
    );
    assert_eq!(
        dir.segment_at(1),
        Some(b"A".as_slice()),
        "BC-2.04.039 PC1: seq=u32::MAX-1 with ISN=u32::MAX-2 → ISN-relative offset must be 1"
    );

    // seq = ISN+3 = u32::MAX+1 = 0 (wrapped) → offset = 3
    let r2 = dir.insert_segment(0, b"B", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        r2,
        InsertResult::Inserted,
        "BC-2.04.039 PC1: wrapped segment must insert"
    );
    assert_eq!(
        dir.segment_at(3),
        Some(b"B".as_slice()),
        "BC-2.04.039 PC1: seq=0 (wrapped past u32::MAX) with ISN=u32::MAX-2 → offset must be 3"
    );

    // seq = ISN+5 = 2 (wrapped) → offset = 5
    let r3 = dir.insert_segment(2, b"C", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        r3,
        InsertResult::Inserted,
        "BC-2.04.039 PC1: double-wrapped segment must insert"
    );
    assert_eq!(
        dir.segment_at(5),
        Some(b"C".as_slice()),
        "BC-2.04.039 PC1: seq=2 (double wrap) with ISN=u32::MAX-2 → offset must be 5"
    );
}

/// BC-2.04.039 PC3: After wraparound, flush_contiguous delivers wrapped
/// segments in the correct byte order regardless of arrival order.
/// Discriminating: insert wrapped segments out-of-arrival-order; assert flush
/// delivers in offset order (1,3), not arrival order (3,1).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_039_flush_delivers_wrapped_segments_in_order() {
    let isn: u32 = u32::MAX - 2;
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(isn);

    // Insert in reverse arrival order: seq=0 (offset=3) first, seq=u32::MAX-1
    // (offset=1) second. Flush must deliver offset=1 before offset=3.
    let r_later = dir.insert_segment(0, b"B", 10_485_760, 10_000, 10_485_760);
    assert_eq!(r_later, InsertResult::Inserted);

    // base_offset is 1 (set by set_isn per BC-2.04.031 PC2); segment at offset=3 is buffered (gap at 1,2).
    // No flush yet.
    let flushed_empty = dir.flush_contiguous();
    assert!(
        flushed_empty.is_empty(),
        "BC-2.04.039 PC3: gap at offset 1 (base_offset) must prevent flush of offset-3 segment"
    );

    // Now insert segment at offset=1 (seq=u32::MAX-1) — this is immediately contiguous.
    let r_first = dir.insert_segment(u32::MAX - 1, b"A", 10_485_760, 10_000, 10_485_760);
    assert_eq!(r_first, InsertResult::Inserted);

    // Flush: segment A (offset=1) flushed; then offset=2 is missing (gap before B at offset=3).
    let flushed = dir.flush_contiguous();

    assert_eq!(
        flushed.len(),
        1,
        "BC-2.04.039 PC3: only contiguous prefix (offset=1) flushed; gap at offset=2 stops flush"
    );
    assert_eq!(
        flushed[0].0, 1,
        "BC-2.04.039 PC3: flushed segment must have ISN-relative offset 1"
    );
    assert_eq!(
        flushed[0].1, b"A",
        "BC-2.04.039 PC3: flushed data must be 'A' (the offset-1 segment)"
    );

    // Insert segment at offset=2 (seq=u32::MAX) to bridge the gap; now B at offset=3 flushes.
    let r_bridge = dir.insert_segment(u32::MAX, b"X", 10_485_760, 10_000, 10_485_760);
    assert_eq!(r_bridge, InsertResult::Inserted);

    let flushed2 = dir.flush_contiguous();
    assert_eq!(
        flushed2.len(),
        2,
        "BC-2.04.039 PC3: bridge segment and B must both flush after gap filled"
    );
    assert_eq!(
        flushed2[0].0, 2,
        "BC-2.04.039 PC3: bridge at offset=2 must come first"
    );
    assert_eq!(
        flushed2[1].0, 3,
        "BC-2.04.039 PC3: 'B' segment at offset=3 must come second"
    );
    assert_eq!(
        flushed2[1].1, b"B",
        "BC-2.04.039 PC3: second flushed data must be 'B'"
    );
}

// =============== STORY-016: Overlap Detection (Wave 9) ===============
// BC-2.04.035 / 036 / 038 / 043 / 047
// 14 ACs + 10 ECs — Part A stubs (Red Gate).
// Every test body panics; all must FAIL before implementation begins.
// =====================================================================

// --- AC-001 (BC-2.04.035 postcondition 1) ---
/// Identical retransmission (same range, identical bytes) must return Duplicate.
/// Canonical vector: ISN=1000, seq=1001, data=b"AAAAA" inserted twice.
/// Second insert must return Duplicate.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_035_identical_retransmission_returns_duplicate() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Duplicate,
        "BC-2.04.035 PC1: identical retransmission must return Duplicate"
    );
}

// --- AC-002 (BC-2.04.035 postconditions 2-3) ---
/// After Duplicate result, segments map and buffered_bytes are unchanged.
/// Canonical vector: ISN=1000, seq=1001, data=b"HELLO" (5 bytes).
/// After Duplicate, segment_count() == 1, buffered_bytes() == 5.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_035_duplicate_does_not_change_buffer() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"HELLO", 10_485_760, 10_000, 10_485_760);
    let count_before = dir.segment_count();
    let buffered_before = dir.buffered_bytes();

    let result = dir.insert_segment(1001, b"HELLO", 10_485_760, 10_000, 10_485_760);

    assert_eq!(result, InsertResult::Duplicate);
    assert_eq!(
        dir.segment_count(),
        count_before,
        "BC-2.04.035 PC2: segment_count must be unchanged after Duplicate"
    );
    assert_eq!(
        dir.buffered_bytes(),
        buffered_before,
        "BC-2.04.035 PC3: buffered_bytes must be unchanged after Duplicate"
    );
    // Verify the stored data is the original (first-wins)
    assert_eq!(
        dir.segment_at(1),
        Some(b"HELLO".as_slice()),
        "BC-2.04.035 PC2: original segment data must be preserved"
    );
}

// --- AC-003 (BC-2.04.035 postcondition 4) ---
/// overlap_count increments by 1 even for a Duplicate result.
/// Canonical vector: overlap_count starts at 0; after one Duplicate, overlap_count == 1.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_035_duplicate_increments_overlap_count() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        dir.overlap_count, 0,
        "overlap_count must be 0 before any overlap"
    );

    let result = dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Duplicate);
    assert_eq!(
        dir.overlap_count, 1,
        "BC-2.04.035 PC4: overlap_count must increment by 1 for Duplicate"
    );

    // A second Duplicate must increment to 2
    let result2 = dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result2, InsertResult::Duplicate);
    assert_eq!(
        dir.overlap_count, 2,
        "BC-2.04.035 PC4: overlap_count must increment again for a second Duplicate"
    );
}

// --- AC-004 (BC-2.04.036 postcondition 1) ---
/// Partial overlap (existing + gap bytes) returns PartialOverlap.
/// Canonical vector: ISN=1000.
///   Segment A = seq=1001, data=b"AAAAA" → offset [1,6).
///   Segment B = seq=1004, data=b"XXXXX" → offset [4,9).
///   B overlaps A at [4,6) and has gap at [6,9).
///   Expected result: PartialOverlap.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_036_partial_overlap_returns_partial_overlap() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(1004, b"XXXXX", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::PartialOverlap,
        "BC-2.04.036 PC1: segment with overlap + gap bytes must return PartialOverlap"
    );
}

// --- AC-005 (BC-2.04.036 postconditions 2-3) ---
/// After PartialOverlap, only gap bytes inserted; existing bytes are preserved (first-wins).
/// Setup: A=b"AAAAA" at [1,6), B=b"XXXXX" at [4,9).
/// After insert B: segment at offset 1 must still be b"AAAAA".
/// Gap at [6,9) is filled with b"XXX" (the tail of B beyond A's end).
/// Flush must yield: b"AAAAA" then b"XXX" = b"AAAAAXXX".
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_036_partial_overlap_preserves_existing_bytes() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(1004, b"XXXXX", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);

    // Original segment at offset 1 must be unchanged (first-wins: INV-3)
    assert_eq!(
        dir.segment_at(1),
        Some(b"AAAAA".as_slice()),
        "BC-2.04.036 PC2: first-wins — existing bytes at [1,6) must not be overwritten"
    );

    // Flush and verify the byte stream: AAAAA (original) + XXX (gap fill)
    let flushed = dir.flush_contiguous();
    let all_bytes: Vec<u8> = flushed
        .iter()
        .flat_map(|(_, d)| d.iter().copied())
        .collect();
    assert_eq!(
        &all_bytes, b"AAAAAXXX",
        "BC-2.04.036 PC3: only gap bytes (XXX at [6,9)) must be added; overlap bytes from A preserved"
    );
}

// --- AC-006 (BC-2.04.036 postcondition 4) ---
/// After PartialOverlap, buffered_bytes increases only by the gap byte count.
/// A=b"AAAAA" (5 bytes) at [1,6). After insert, buffered_bytes==5.
/// B=b"XXXXX" (5 bytes) at [4,9). Overlap at [4,6) = 2 bytes, gap at [6,9) = 3 bytes.
/// After PartialOverlap: buffered_bytes must be 5+3=8 (NOT 5+5=10).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_036_partial_overlap_buffered_bytes_gap_only() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        dir.buffered_bytes(),
        5,
        "baseline buffered_bytes after inserting A"
    );

    let result = dir.insert_segment(1004, b"XXXXX", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);

    assert_eq!(
        dir.buffered_bytes(),
        8,
        "BC-2.04.036 PC4: buffered_bytes must increase by exactly 3 gap bytes (not 5)"
    );
}

// --- AC-007 (BC-2.04.036 postcondition 5) ---
/// overlap_count increments by 1 for a PartialOverlap result.
/// A=b"AAAAA" at [1,6), then B=b"XXXXX" at [4,9).
/// overlap_count before: 0. After PartialOverlap: 1.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_036_partial_overlap_increments_overlap_count() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        dir.overlap_count, 0,
        "overlap_count must be 0 before overlap"
    );

    let result = dir.insert_segment(1004, b"XXXXX", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(
        dir.overlap_count, 1,
        "BC-2.04.036 PC5: overlap_count must increment by 1 for PartialOverlap"
    );
}

// --- AC-008 (BC-2.04.038 postcondition 1) ---
/// New segment fully covered by union of 2+ existing segments with matching bytes → Duplicate.
/// Setup: ISN=1000. A=b"ABC" at offset [1,4). B=b"DEF" at offset [4,7).
/// Insert C=b"ABCDEF" at offset [1,7) — union of A+B fully covers [1,7) with matching bytes.
/// Expected: Duplicate.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_038_multi_segment_full_coverage_matching_returns_duplicate() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    // Two adjacent segments covering [1,7)
    dir.insert_segment(1001, b"ABC", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1004, b"DEF", 10_485_760, 10_000, 10_485_760);

    // Insert segment spanning the whole range with matching bytes
    let result = dir.insert_segment(1001, b"ABCDEF", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Duplicate,
        "BC-2.04.038 PC1: new segment fully covered by union of 2 segments with matching bytes must return Duplicate"
    );
}

// --- AC-009 (BC-2.04.038 postcondition 2) ---
/// New segment fully covered by union but at least one byte differs → ConflictingOverlap.
/// Setup: A=b"ABC" at [1,4), B=b"DEF" at [4,7).
/// Insert C=b"ABCXEF" at [1,7) — one byte differs (X vs D at offset 4).
/// Expected: ConflictingOverlap.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_038_multi_segment_full_coverage_conflicting_returns_conflicting() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"ABC", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1004, b"DEF", 10_485_760, 10_000, 10_485_760);

    // One byte differs: 'X' at offset 4 conflicts with 'D'
    let result = dir.insert_segment(1001, b"ABCXEF", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::ConflictingOverlap,
        "BC-2.04.038 PC2: new segment fully covered by union but one byte differs must return ConflictingOverlap"
    );
}

// --- AC-010 (BC-2.04.043 postcondition 1) ---
/// Segment whose start == existing segment's end returns Inserted, not PartialOverlap.
/// Canonical vector: ISN=1000. A=b"AAAAA" at [1,6).
/// B starts at seq=1006 → offset=6, which is exactly A's end.
/// Half-open interval check [new_start < existing_end]: 6 < 6 is false → no overlap.
/// Expected: Inserted (not PartialOverlap).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_043_adjacent_segment_returns_inserted_not_overlap() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(1006, b"BBBBB", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Inserted,
        "BC-2.04.043 PC1: segment starting exactly where existing segment ends must return Inserted, not PartialOverlap"
    );
}

// --- AC-011 (BC-2.04.043 postcondition 2) ---
/// overlap_count is NOT incremented for an adjacent (touching, non-overlapping) segment.
/// A=b"AAAAA" at [1,6), then B=b"BBBBB" at offset 6 (adjacent).
/// overlap_count must remain 0 after inserting B.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_043_adjacent_segment_does_not_increment_overlap_count() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        dir.overlap_count, 0,
        "overlap_count must be 0 after clean insert"
    );

    let result = dir.insert_segment(1006, b"BBBBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(
        dir.overlap_count, 0,
        "BC-2.04.043 PC2: overlap_count must NOT increment for adjacent segment (new_start == existing_end)"
    );
}

// --- AC-012 (BC-2.04.047 postcondition 1) ---
/// buffered_bytes == sum(segments.values().map(|v| v.len())) at all times.
///
/// Exercises VP-010: the buffered_bytes counter must mirror actual segment storage.
/// Uses proptest: random insert/flush sequences; after each op, memory_used()
/// triggers the debug_assert that verifies the invariant internally.
/// Additionally, explicit spot-checks verify specific states.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_047_buffered_bytes_mirrors_segment_size_sum() {
    // Deterministic spot-check: multi-step scenario
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    // Step 1: fresh — 0 bytes
    assert_eq!(dir.memory_used(), 0, "invariant at step 1 (empty)");

    // Step 2: insert 5 bytes
    dir.insert_segment(1001, b"HELLO", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.memory_used(), 5, "invariant after inserting 5 bytes");

    // Step 3: insert 3 more bytes (non-contiguous gap)
    dir.insert_segment(1010, b"XYZ", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.memory_used(), 8, "invariant after second insert (gap)");

    // Step 4: Duplicate — buffered_bytes must not change
    dir.insert_segment(1001, b"HELLO", 10_485_760, 10_000, 10_485_760);
    assert_eq!(
        dir.memory_used(),
        8,
        "invariant after Duplicate (no change)"
    );

    // Step 5: PartialOverlap — only gap bytes counted
    // Insert b"HELLOWORLD" at offset 1: A=[1,6), new=[1,11), gap=[6,11)=5 bytes
    dir.insert_segment(1001, b"HELLOWORLD", 10_485_760, 10_000, 10_485_760);
    // After this: offset 1 has "HELLO" (5), gap [6,10) = "WORL" (4 bytes, since XYZ is at offset 10),
    // XYZ at offset 10 (3). Total = 5+4+3=12.
    assert_eq!(
        dir.memory_used(),
        12,
        "invariant after PartialOverlap gap fill"
    );

    // Step 6: flush contiguous — base_offset was 1
    let flushed = dir.flush_contiguous();
    let flushed_bytes: usize = flushed.iter().map(|(_, d)| d.len()).sum();
    assert_eq!(
        dir.memory_used(),
        12 - flushed_bytes,
        "invariant after flush: buffered_bytes == prior - flushed_bytes"
    );
}

// --- AC-013 (BC-2.04.047 postcondition 4) ---
/// buffered_bytes is unchanged for Duplicate, ConflictingOverlap, OutOfWindow, IsnMissing.
/// Tests each non-insert result variant individually.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_047_buffered_bytes_unchanged_for_non_insert_results() {
    // --- Duplicate ---
    {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        dir.set_isn(1000);
        dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
        let baseline = dir.buffered_bytes();
        let result = dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
        assert_eq!(result, InsertResult::Duplicate);
        assert_eq!(
            dir.buffered_bytes(),
            baseline,
            "BC-2.04.047 PC4: buffered_bytes must not change for Duplicate"
        );
    }

    // --- ConflictingOverlap (same range, different bytes) ---
    {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        dir.set_isn(1000);
        dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
        let baseline = dir.buffered_bytes();
        let result = dir.insert_segment(1001, b"BBBBB", 10_485_760, 10_000, 10_485_760);
        assert_eq!(result, InsertResult::ConflictingOverlap);
        assert_eq!(
            dir.buffered_bytes(),
            baseline,
            "BC-2.04.047 PC4: buffered_bytes must not change for ConflictingOverlap"
        );
    }

    // --- OutOfWindow ---
    {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        dir.set_isn(1000);
        dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 1_048_576);
        dir.flush_contiguous(); // base_offset now 6
        let baseline = dir.buffered_bytes();
        // Far beyond window
        let far_seq: u32 = 1000_u32.wrapping_add(6).wrapping_add(1_048_576 + 100);
        let result = dir.insert_segment(far_seq, b"evil", 10_485_760, 10_000, 1_048_576);
        assert_eq!(result, InsertResult::OutOfWindow);
        assert_eq!(
            dir.buffered_bytes(),
            baseline,
            "BC-2.04.047 PC4: buffered_bytes must not change for OutOfWindow"
        );
    }

    // --- IsnMissing ---
    {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        // deliberately no set_isn
        let baseline = dir.buffered_bytes();
        let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
        assert_eq!(result, InsertResult::IsnMissing);
        assert_eq!(
            dir.buffered_bytes(),
            baseline,
            "BC-2.04.047 PC4: buffered_bytes must not change for IsnMissing"
        );
    }
}

// --- AC-014 (BC-2.04.047 postcondition 5) ---
/// After flush_contiguous() flushes N bytes, buffered_bytes decreases by exactly N.
/// Tests two scenarios: full flush and partial flush (gap blocks second segment).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_047_buffered_bytes_decrements_on_flush() {
    // Full flush scenario: one contiguous segment
    {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        dir.set_isn(1000);
        dir.insert_segment(1001, b"HELLOWORLD", 10_485_760, 10_000, 10_485_760);
        let before = dir.buffered_bytes();
        assert_eq!(before, 10);

        let flushed = dir.flush_contiguous();
        let n: usize = flushed.iter().map(|(_, d)| d.len()).sum();
        assert_eq!(n, 10, "full flush must yield 10 bytes");
        assert_eq!(
            dir.buffered_bytes(),
            before - n,
            "BC-2.04.047 PC5: buffered_bytes must decrease by exactly N={} after flush",
            n
        );
        assert_eq!(dir.buffered_bytes(), 0);
    }

    // Partial flush scenario: gap prevents second segment from flushing
    {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        dir.set_isn(1000);
        // segment A at [1,6) and segment B at [10,15) — gap at [6,10)
        dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
        dir.insert_segment(1010, b"BBBBB", 10_485_760, 10_000, 10_485_760);
        let before = dir.buffered_bytes();
        assert_eq!(before, 10, "10 bytes buffered before partial flush");

        let flushed = dir.flush_contiguous();
        let n: usize = flushed.iter().map(|(_, d)| d.len()).sum();
        assert_eq!(
            n, 5,
            "partial flush must yield only 5 bytes (A); B blocked by gap"
        );
        assert_eq!(
            dir.buffered_bytes(),
            before - n,
            "BC-2.04.047 PC5: buffered_bytes must decrease by exactly N={} (partial flush)",
            n
        );
        assert_eq!(dir.buffered_bytes(), 5, "5 bytes (B) must remain buffered");
    }
}

// =============================================================================
// STORY-016: BC-2.04.047 PC1 — buffered_bytes mirrors segment size sum
// VP-010 proptest (AC-012 property-based variant)
// Exercises ≥1000 random insert/flush sequences and asserts the invariant
// via memory_used() after every operation.
// Note: `use proptest::prelude::*` is declared once at the bottom of this file
// (STORY-015 proptest section); the proptest! macro below relies on it.
// =============================================================================

#[derive(Debug, Clone)]
enum OverlapOp {
    Insert { seq_delta: u32, len: u8, fill: u8 },
    Flush,
}

fn overlap_op_strategy() -> impl Strategy<Value = OverlapOp> {
    prop_oneof![
        // seq_delta in [0,30) so segments frequently overlap for rich coverage
        (0u32..30u32, 1u8..20u8, 0u8..=255u8).prop_map(|(delta, len, fill)| {
            OverlapOp::Insert {
                seq_delta: delta,
                len,
                fill,
            }
        }),
        Just(OverlapOp::Flush),
    ]
}

proptest! {
    #![proptest_config(proptest::test_runner::Config::with_cases(1000))]
    /// VP-010 / BC-2.04.047 PC1: For any random sequence of overlapping inserts and
    /// flushes, `buffered_bytes` mirrors `sum(segment sizes)` after EACH operation.
    /// memory_used() enforces this via debug_assert internally; we also assert the
    /// return value equals buffered_bytes() explicitly.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_04_047_proptest_buffered_bytes_mirrors_segment_size_sum(
        ops in proptest::collection::vec(overlap_op_strategy(), 1..=30)
    ) {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        let isn: u32 = 5000;
        dir.set_isn(isn);

        for op in &ops {
            match op {
                OverlapOp::Insert { seq_delta, len, fill } => {
                    let seq = isn.wrapping_add(*seq_delta).wrapping_add(1);
                    let data = vec![*fill; *len as usize];
                    let _ = dir.insert_segment(seq, &data, 10_485_760, 10_000, 10_485_760);
                }
                OverlapOp::Flush => {
                    let _ = dir.flush_contiguous();
                }
            }
            // memory_used() triggers debug_assert internally; also verify return value
            let mu = dir.memory_used();
            prop_assert_eq!(
                mu,
                dir.buffered_bytes(),
                "BC-2.04.047 PC1: memory_used() must equal buffered_bytes()"
            );
        }
    }
}

// --- EC-001 — Exact same seq, exact same bytes → Duplicate ---
/// Canonical vector: ISN=1000, seq=2000, data=b"RETRANSMIT" inserted twice.
/// Second insert must return Duplicate (covers the single-segment fully-covered path).
#[test]
fn test_story_016_ec001_exact_retransmission_duplicate() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(2000, b"RETRANSMIT", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(2000, b"RETRANSMIT", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Duplicate,
        "EC-001: exact same seq and bytes must return Duplicate"
    );
    assert_eq!(dir.overlap_count, 1, "EC-001: overlap_count must be 1");
}

// --- EC-002 — Range covered by 2 adjacent (contiguous) segments, matching bytes → Duplicate ---
/// ISN=1000. A=b"AAA" at [1,4). B=b"BBB" at [4,7) (adjacent, contiguous — no gap between them).
/// Insert C=b"AAABBB" at [1,7) with matching bytes → Duplicate.
/// This exercises the gap-computation path where sorted ranges collapse to no gaps.
#[test]
fn test_story_016_ec002_adjacent_union_coverage_duplicate() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAA", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1004, b"BBB", 10_485_760, 10_000, 10_485_760);

    let result = dir.insert_segment(1001, b"AAABBB", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Duplicate,
        "EC-002: new range covered by union of 2 segments with matching bytes must return Duplicate"
    );
}

// --- EC-003 — Same range, one byte differs → ConflictingOverlap ---
/// ISN=1000. Insert b"AAAAA" then re-insert b"AAAXA" — one byte differs at position 3.
/// Expected: ConflictingOverlap (same range, single-segment fully covers new).
#[test]
fn test_story_016_ec003_same_range_one_byte_differs_conflicting() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let result = dir.insert_segment(1001, b"AAAXA", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::ConflictingOverlap,
        "EC-003: same range with one byte differing must return ConflictingOverlap"
    );
    // Original bytes must be preserved (first-wins)
    assert_eq!(
        dir.segment_at(1),
        Some(b"AAAAA".as_slice()),
        "EC-003: original bytes must be preserved after ConflictingOverlap"
    );
}

// --- EC-004 — New segment extends existing at the end (append) → PartialOverlap, tail gap added ---
/// ISN=1000. A=b"AAAAA" at [1,6). B=b"AAAAABBB" at [1,9) — extends A by 3 tail bytes.
/// Overlap at [1,6), gap at [6,9). Expected: PartialOverlap.
/// buffered_bytes increases by 3 (tail gap only).
#[test]
fn test_story_016_ec004_append_extension_partial_overlap() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let before = dir.buffered_bytes();

    let result = dir.insert_segment(1001, b"AAAAABBB", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::PartialOverlap,
        "EC-004: segment extending existing at end must return PartialOverlap"
    );
    assert_eq!(
        dir.buffered_bytes(),
        before + 3,
        "EC-004: buffered_bytes must increase by 3 (tail gap bytes only)"
    );
    // first-wins: A's original bytes at offset 1 must be preserved
    assert_eq!(
        dir.segment_at(1),
        Some(b"AAAAA".as_slice()),
        "EC-004: first-wins preserves A's original AAAAA at offset 1"
    );
    // tail gap at offset 6 must be filled with B's tail bytes b\"BBB\"
    assert_eq!(
        dir.segment_at(6),
        Some(b"BBB".as_slice()),
        "EC-004: tail gap at offset 6 must be filled with b\"BBB\""
    );
}

// --- EC-005 — New segment extends existing at the start (prepend) → PartialOverlap, head gap added ---
/// ISN=1000. A=b"AAAAA" at offset [4,9) (seq=1004). B=b"BBBAAAAA" at [1,9) (seq=1001).
/// Overlap at [4,9), gap at [1,4). Expected: PartialOverlap.
/// buffered_bytes increases by 3 (head gap only).
#[test]
fn test_story_016_ec005_prepend_extension_partial_overlap() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 4 (seq=1004)
    dir.insert_segment(1004, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    let before = dir.buffered_bytes();
    assert_eq!(before, 5, "5 bytes buffered after first insert");

    // Insert segment spanning [1,9): 3 head bytes (gap) + 5 overlap bytes
    let result = dir.insert_segment(1001, b"BBBAAAAA", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::PartialOverlap,
        "EC-005: segment prepending existing must return PartialOverlap"
    );
    assert_eq!(
        dir.buffered_bytes(),
        before + 3,
        "EC-005: buffered_bytes must increase by 3 (head gap bytes only)"
    );
    // Head gap at offset 1 must be filled with b"BBB"
    assert_eq!(
        dir.segment_at(1),
        Some(b"BBB".as_slice()),
        "EC-005: head gap at offset 1 must be filled with b\"BBB\""
    );
    // first-wins: A's original bytes at offset 4 must be preserved (overlap region not overwritten)
    assert_eq!(
        dir.segment_at(4),
        Some(b"AAAAA".as_slice()),
        "EC-005: first-wins preserves A's original AAAAA at offset 4"
    );
}

// --- EC-006 — New segment spans two existing segments with a gap between → gap bytes filled ---
/// ISN=1000. A=b"AAA" at [1,4). B=b"BBB" at [7,10). Gap at [4,7).
/// Insert C=b"AAAXYZBB" at [1,9) — spans A and part of B, fills gap [4,7).
/// Expected: PartialOverlap; buffered_bytes increases by 3 (gap only).
#[test]
fn test_story_016_ec006_spans_two_segments_gap_filled() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    // A at offset 1, length 3: covers [1,4)
    dir.insert_segment(1001, b"AAA", 10_485_760, 10_000, 10_485_760);
    // B at offset 7, length 3: covers [7,10)
    dir.insert_segment(1007, b"BBB", 10_485_760, 10_000, 10_485_760);
    let before = dir.buffered_bytes();
    assert_eq!(before, 6, "6 bytes buffered before bridging insert");

    // Insert C covering [1,10): "AAA" + "XYZ" (gap at [4,7)) + "BBB" (overlaps B)
    let result = dir.insert_segment(1001, b"AAAXYZBB", 10_485_760, 10_000, 10_485_760);

    // C covers [1,9): overlaps A at [1,4), has gap at [4,7) = "XYZ" (3 bytes),
    // overlaps B at [7,9) (2 bytes of B's 3).
    assert_eq!(
        result,
        InsertResult::PartialOverlap,
        "EC-006: segment spanning two existing segments with gap must return PartialOverlap"
    );
    // Gap at [4,7) must be filled
    assert!(
        dir.has_segment_at(4),
        "EC-006: gap at offset 4 must be filled"
    );
    // gap bytes at offset 4 must be b"XYZ" (C[3..6])
    assert_eq!(
        dir.segment_at(4),
        Some(b"XYZ".as_slice()),
        "EC-006: gap bytes at offset 4 must be b\"XYZ\""
    );
    // buffered_bytes increased by 3 (the gap bytes at [4,7))
    assert_eq!(
        dir.buffered_bytes(),
        before + 3,
        "EC-006: buffered_bytes must increase by 3 (gap bytes only)"
    );
    // first-wins: A's original bytes at offset 1 must be preserved
    assert_eq!(
        dir.segment_at(1),
        Some(b"AAA".as_slice()),
        "EC-006: first-wins preserves A's original AAA at offset 1"
    );
    // first-wins: B's original bytes at offset 7 must be preserved
    assert_eq!(
        dir.segment_at(7),
        Some(b"BBB".as_slice()),
        "EC-006: first-wins preserves B's original BBB at offset 7"
    );
}

// --- EC-007 — Segment B starts exactly where segment A ends → Inserted, overlap_count unchanged ---
/// ISN=1000. A=b"AAAAA" ends at offset 6. B starts at seq=1006 → offset 6.
/// Condition new_start(6) < existing_end(6) is false (6 < 6 = false) → no overlap.
/// Expected: Inserted, overlap_count stays 0.
#[test]
fn test_story_016_ec007_exact_adjacency_inserted_not_overlap() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    // Segment B starts at exactly the byte AFTER A ends
    let result = dir.insert_segment(1006, b"BBBBB", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Inserted,
        "EC-007: B starting where A ends must return Inserted"
    );
    assert_eq!(
        dir.overlap_count, 0,
        "EC-007: overlap_count must remain 0 for exact adjacency"
    );
}

// --- EC-008 — Segment B starts one byte before segment A ends → overlap detected ---
/// ISN=1000. A=b"AAAAA" at [1,6). B starts at seq=1005 → offset 5 (one byte before A ends).
/// Condition new_start(5) < existing_end(6): 5 < 6 is true → overlap detected.
/// Expected: PartialOverlap (B has 1 overlap byte and N-1 gap bytes) or
/// ConflictingOverlap/Duplicate if bytes match/differ for the overlap portion.
/// B=b"XBBBB" (5 bytes) → offset 5 to 10. Overlap at [5,6) = "X" vs "A" → conflict in overlap byte.
/// But only 1 byte overlaps and B has 4 gap bytes → PartialOverlap with data inserted for [6,10).
#[test]
fn test_story_016_ec008_one_byte_before_end_is_overlap() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AAAAA", 10_485_760, 10_000, 10_485_760);
    // B at offset 5 (1 byte before A ends at offset 6): overlaps A's last byte
    let result = dir.insert_segment(1005, b"XBBBB", 10_485_760, 10_000, 10_485_760);

    // B at [5,10) overlaps A at [1,6) → overlap range [5,6) = 1 byte.
    // new byte at [5] is 'X', existing is 'A' — conflict in overlap region.
    // Gap at [6,10) = b"BBBB" (4 bytes from B[1..5]) gets inserted.
    // Since there IS a gap (had_gap=true), result is PartialOverlap (not ConflictingOverlap).
    assert_eq!(
        result,
        InsertResult::PartialOverlap,
        "EC-008: B starting 1 byte before A ends must detect overlap (PartialOverlap with gap)"
    );
    assert_eq!(
        dir.overlap_count, 1,
        "EC-008: overlap_count must be 1 after overlap is detected"
    );
    assert!(
        dir.has_segment_at(6),
        "EC-008: gap segment at offset 6 must exist"
    );
    assert_eq!(
        dir.segment_at(6),
        Some(&b"BBBB"[..]),
        "EC-008: gap bytes at offset 6 must be B[1..5] = b\"BBBB\""
    );
    assert_eq!(
        dir.segment_at(1),
        Some(&b"AAAAA"[..]),
        "EC-008: first-wins preserves A's original AAAAA at offset 1"
    );
}

// --- EC-009 — Three segments covering new range jointly, all bytes match → Duplicate ---
/// ISN=1000. A=b"AA" at [1,3). B=b"BB" at [3,5). C=b"CC" at [5,7).
/// Insert D=b"AABBCC" at [1,7) — all 3 segments jointly cover D with matching bytes.
/// Expected: Duplicate.
#[test]
fn test_story_016_ec009_three_segment_union_coverage_duplicate() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"AA", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1003, b"BB", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1005, b"CC", 10_485_760, 10_000, 10_485_760);

    // Insert spanning all three with matching bytes
    let result = dir.insert_segment(1001, b"AABBCC", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Duplicate,
        "EC-009: new segment fully covered by union of 3 segments with matching bytes must return Duplicate"
    );
}

// --- EC-010 — Empty data slice → Inserted (early-return before overlap checks) ---
/// ISN=1000. Insert empty slice at any seq.
/// Expected: Inserted (early-return at data.is_empty() check in segment.rs).
/// segment_count and buffered_bytes remain 0.
#[test]
fn test_story_016_ec010_empty_data_returns_inserted() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let result = dir.insert_segment(1001, b"", 10_485_760, 10_000, 10_485_760);

    assert_eq!(
        result,
        InsertResult::Inserted,
        "EC-010: empty data slice must return Inserted (early-return path)"
    );
    assert_eq!(
        dir.segment_count(),
        0,
        "EC-010: no segment stored for empty data"
    );
    assert_eq!(
        dir.buffered_bytes(),
        0,
        "EC-010: buffered_bytes must remain 0 for empty data"
    );
    assert_eq!(
        dir.overlap_count, 0,
        "EC-010: overlap_count must not change for empty data"
    );
}

// =============================================================================
// STORY-015: BC-2.04.007 inv-3 — base_offset is monotonically non-decreasing
// VP-011 proptest
// =============================================================================

use proptest::prelude::*;

#[derive(Debug, Clone)]
enum SegOp {
    Insert { seq_delta: u32, len: u8 },
    Flush,
}

fn seg_op_strategy() -> impl Strategy<Value = SegOp> {
    prop_oneof![
        (0u32..20u32, 1u8..16u8).prop_map(|(delta, len)| SegOp::Insert {
            seq_delta: delta,
            len
        }),
        Just(SegOp::Flush),
    ]
}

proptest! {
    /// VP-011 / BC-2.04.007 inv-3: For any sequence of in-window inserts and flushes around a
    /// mid-range ISN, BC-2.04.007 invariant 3 holds: `base_offset` is monotonically
    /// non-decreasing. Wraparound monotonicity is covered separately by AC-016/AC-017
    /// (`test_BC_2_04_039_*`); out-of-window monotonicity is covered by
    /// `test_out_of_window_segment_rejected`.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_04_007_base_offset_is_monotonic(ops in proptest::collection::vec(seg_op_strategy(), 1..=20)) {
        let mut dir = wirerust::reassembly::flow::FlowDirection::new();
        let isn: u32 = 1000;
        dir.set_isn(isn);

        let mut prev_base: u64 = dir.base_offset;
        for op in &ops {
            match op {
                SegOp::Insert { seq_delta, len } => {
                    let seq = isn.wrapping_add(*seq_delta).wrapping_add(1);
                    let data = vec![0u8; *len as usize];
                    let _ = dir.insert_segment(seq, &data, 10_485_760, 10_000, 10_485_760);
                }
                SegOp::Flush => {
                    let _ = dir.flush_contiguous();
                }
            }
            assert!(
                dir.base_offset >= prev_base,
                "BC-2.04.007 inv-3: base_offset decreased from {} to {} — monotonicity violated",
                prev_base,
                dir.base_offset
            );
            prev_base = dir.base_offset;
        }
    }
}

// =============== STORY-018: Resource Bounds — Segment-Level (Wave 10) ===============
// BCs: 2.04.041 (depth truncation), 2.04.042 (out-of-window), 2.04.044 (segment limit
// non-overlap), 2.04.045 (segment limit overlap), 2.04.046 (partial insertion mid-loop)
// ACs: 001, 002, 003, 010, 011, 012, 016, 017, 018, 019
// ECs: 001, 002, 003, 004, 005, 006, 007, 009, 010
// All test bodies panic — Red Gate (Part A stubs).
// ======================================================================================

// --- AC-001 (BC-2.04.041 postcondition 1) ---
/// When reassembled_bytes + buffered_bytes + data.len() > max_depth and
/// depth_exceeded == false, insert_segment returns InsertResult::Truncated.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_041_depth_truncation_returns_truncated() {
    // Canonical vector (from story spec):
    //   max_depth=10; insert 5 bytes (buffered=5); insert 8 more bytes →
    //   reassembled(0) + buffered(5) + new(8) = 13 > 10 → Truncated.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    // First insert: 5 bytes at offset 1 (within depth, buffered=5)
    let r1 = dir.insert_segment(1001, b"AAAAA", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r1,
        InsertResult::Inserted,
        "AC-001 precondition: first 5-byte insert must succeed (total=5 <= max_depth=10)"
    );
    assert_eq!(
        dir.buffered_bytes(),
        5,
        "AC-001 precondition: buffered_bytes must be 5 after first insert"
    );
    assert!(
        !dir.depth_exceeded,
        "AC-001 precondition: depth_exceeded must be false"
    );

    // Second insert: 8 bytes → 0 + 5 + 8 = 13 > 10 → Truncated
    let r2 = dir.insert_segment(
        1006,
        b"BBBBBBBB",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        r2,
        InsertResult::Truncated,
        "BC-2.04.041 PC1: reassembled(0) + buffered(5) + new(8) = 13 > max_depth(10) must return Truncated"
    );
}

// --- AC-002 (BC-2.04.041 postconditions 2-4) ---
/// After a Truncated result, only allowed = max_depth.saturating_sub(reassembled_bytes
/// + buffered_bytes) bytes are stored, and buffered_bytes increases by exactly allowed.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_041_truncated_stores_only_allowed_bytes() {
    // Canonical vector:
    //   max_depth=10; insert 5 bytes (buffered=5); insert 8 more bytes →
    //   allowed = 10 - (0 + 5) = 5; buffered_bytes must increase by exactly 5.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    // First insert: 5 bytes at offset 1
    dir.insert_segment(1001, b"AAAAA", max_depth, max_segments, max_receive_window);
    assert_eq!(
        dir.buffered_bytes(),
        5,
        "AC-002 precondition: 5 bytes buffered"
    );

    // Second insert: 8 bytes; only allowed=5 should be stored
    let r = dir.insert_segment(
        1006,
        b"BBBBBBBB",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(r, InsertResult::Truncated, "AC-002: must return Truncated");

    // buffered_bytes should have increased by exactly 5 (= allowed = 10 - 5)
    assert_eq!(
        dir.buffered_bytes(),
        10,
        "BC-2.04.041 PC3-4: buffered_bytes must be exactly max_depth after Truncated (5 original + 5 allowed)"
    );

    // Flush and verify only 5 bytes of the truncated segment were stored
    let flushed = dir.flush_contiguous();
    let all_bytes: usize = flushed.iter().map(|(_, d)| d.len()).sum();
    assert_eq!(
        all_bytes, 10,
        "BC-2.04.041 PC2: total flushed bytes must be exactly max_depth (5 original + 5 allowed)"
    );
}

// --- AC-003 (BC-2.04.041 postcondition 5 and invariant 1) ---
/// After Truncated, depth_exceeded == true and all subsequent inserts return
/// InsertResult::DepthExceeded (not Truncated again).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_041_depth_exceeded_flag_set_after_truncated() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    // Insert 5 bytes then trigger Truncated
    dir.insert_segment(1001, b"AAAAA", max_depth, max_segments, max_receive_window);
    let r = dir.insert_segment(
        1006,
        b"BBBBBBBB",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        r,
        InsertResult::Truncated,
        "AC-003 precondition: Truncated must fire"
    );

    // BC-2.04.041 PC5: depth_exceeded must be true after Truncated
    assert!(
        dir.depth_exceeded,
        "BC-2.04.041 PC5: depth_exceeded must be true after Truncated"
    );

    // BC-2.04.041 INV-1: all subsequent inserts must return DepthExceeded, not Truncated
    let r2 = dir.insert_segment(1020, b"CCCC", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r2,
        InsertResult::DepthExceeded,
        "BC-2.04.041 INV-1: second insert after Truncated must return DepthExceeded (not Truncated again)"
    );

    // Third insert still returns DepthExceeded (flag is permanent)
    let r3 = dir.insert_segment(1030, b"DDDD", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r3,
        InsertResult::DepthExceeded,
        "BC-2.04.041 INV-1: third insert after Truncated must also return DepthExceeded"
    );
}

// --- AC-010 (BC-2.04.042 postcondition 1 and invariant 1) ---
/// When a segment's computed offset exceeds base_offset.saturating_add(max_receive_window
/// as u64), insert_segment returns InsertResult::OutOfWindow and no bytes are stored.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_042_out_of_window_returns_out_of_window() {
    // Setup: ISN=1000, so base_offset=1, max_receive_window=100.
    // A segment at seq=1000+1+100+1=1102 has offset=102 which exceeds base_offset(1)+window(100)=101.
    // So offset 102 > 101 → OutOfWindow.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 100;

    // base_offset=1, window=100 → limit = 1 + 100 = 101.
    // offset of a segment at seq=1102 is seq.wrapping_sub(ISN) as u64 = 1102 - 1000 = 102.
    // 102 > 101 → OutOfWindow.
    let result = dir.insert_segment(1102, b"evil", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result,
        InsertResult::OutOfWindow,
        "BC-2.04.042 PC1: offset(102) > base_offset(1) + window(100) must return OutOfWindow"
    );

    // Verify no bytes stored
    assert_eq!(
        dir.buffered_bytes(),
        0,
        "BC-2.04.042 INV-1: buffered_bytes must remain 0 after OutOfWindow"
    );
    assert!(
        dir.segments_is_empty(),
        "BC-2.04.042 INV-1: no segment must be stored"
    );
}

// --- AC-011 (BC-2.04.042 postcondition 4) ---
/// out_of_window_count increments by 1 for each out-of-window segment.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_042_out_of_window_count_increments() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 100;

    assert_eq!(
        dir.out_of_window_count, 0,
        "AC-011 precondition: count starts at 0"
    );

    // First out-of-window segment: offset=102 > limit=101
    let r1 = dir.insert_segment(1102, b"x", max_depth, max_segments, max_receive_window);
    assert_eq!(r1, InsertResult::OutOfWindow, "AC-011: first OOW segment");
    assert_eq!(
        dir.out_of_window_count, 1,
        "BC-2.04.042 PC4: out_of_window_count must be 1 after first OOW segment"
    );

    // Second out-of-window segment: count must increment again
    let r2 = dir.insert_segment(1200, b"y", max_depth, max_segments, max_receive_window);
    assert_eq!(r2, InsertResult::OutOfWindow, "AC-011: second OOW segment");
    assert_eq!(
        dir.out_of_window_count, 2,
        "BC-2.04.042 PC4: out_of_window_count must be 2 after second OOW segment"
    );
}

// --- AC-012 (BC-2.04.042 edge case EC-001) ---
/// A segment at exactly base_offset + max_receive_window (the boundary) is accepted
/// with InsertResult::Inserted (boundary is exclusive: offset > window, not >=).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_042_segment_at_exact_window_boundary_is_inserted() {
    // ISN=1000 → base_offset=1. max_receive_window=100 → window_limit = 1 + 100 = 101.
    // A segment at ISN-relative offset=101 has seq = 1000 + 101 = 1101.
    // The check is: offset > window_limit → 101 > 101 is FALSE → Inserted.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 100;

    // Segment at exactly the boundary (offset == base + window): must be Inserted
    let result = dir.insert_segment(
        1101,
        b"boundary",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        result,
        InsertResult::Inserted,
        "BC-2.04.042 EC-001: segment at offset == base_offset + window must be Inserted (exclusive boundary: `>` not `>=`)"
    );

    // One byte beyond the boundary (offset = 102 > 101): must be OutOfWindow
    let result2 = dir.insert_segment(1102, b"x", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result2,
        InsertResult::OutOfWindow,
        "BC-2.04.042 EC-001: segment one byte beyond boundary must be OutOfWindow"
    );
}

// --- AC-016 (BC-2.04.044 postcondition 1 and invariant 1) ---
/// When segments.len() >= max_segments and the new segment does not overlap any existing
/// entry, insert_segment returns InsertResult::SegmentLimitReached and segments.len()
/// is unchanged.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_044_segment_limit_non_overlapping_path() {
    // Setup: max_segments=3; insert 3 non-overlapping segments to fill the map.
    // Then insert a 4th non-overlapping segment → SegmentLimitReached.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 3;
    let max_receive_window: usize = 10_485_760;

    // Fill the segment map to capacity (3 non-overlapping, non-contiguous segments)
    dir.insert_segment(1001, b"AAA", max_depth, max_segments, max_receive_window);
    dir.insert_segment(1010, b"BBB", max_depth, max_segments, max_receive_window);
    dir.insert_segment(1020, b"CCC", max_depth, max_segments, max_receive_window);
    assert_eq!(
        dir.segment_count(),
        3,
        "AC-016 precondition: 3 segments fill the map"
    );

    // New non-overlapping segment (at seq=1030, offset=30, no overlap with existing)
    let result = dir.insert_segment(1030, b"DDD", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result,
        InsertResult::SegmentLimitReached,
        "BC-2.04.044 PC1: non-overlapping insert at full capacity must return SegmentLimitReached"
    );
    assert_eq!(
        dir.segment_count(),
        3,
        "BC-2.04.044 INV-1: segments.len() must be unchanged after SegmentLimitReached"
    );
}

// --- AC-017 (BC-2.04.044 edge case EC-001) ---
/// When segments.len() == max_segments - 1, a new non-overlapping segment is inserted
/// successfully (not rejected).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_044_segment_one_below_limit_is_inserted() {
    // max_segments=3; insert 2 segments (== max - 1), then insert a 3rd → Inserted.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 3;
    let max_receive_window: usize = 10_485_760;

    // Insert 2 segments (one below limit)
    dir.insert_segment(1001, b"AAA", max_depth, max_segments, max_receive_window);
    dir.insert_segment(1010, b"BBB", max_depth, max_segments, max_receive_window);
    assert_eq!(
        dir.segment_count(),
        2,
        "AC-017 precondition: 2 segments = max_segments - 1"
    );

    // Third segment at exactly max_segments - 1 → must be Inserted (not rejected)
    let result = dir.insert_segment(1020, b"CCC", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result,
        InsertResult::Inserted,
        "BC-2.04.044 EC-001: insert when segments.len() == max_segments - 1 must return Inserted"
    );
    assert_eq!(
        dir.segment_count(),
        3,
        "AC-017: 3 segments in map after successful insert"
    );
}

// --- AC-018 (BC-2.04.045 postcondition 1 and invariant 2) ---
/// When segments.len() >= max_segments and the new segment overlaps existing entries but
/// gaps cannot be inserted, insert_segment returns SegmentLimitReached and overlap_count
/// is incremented (overlap was detected before the mid-loop limit check).
///
/// Implementation note: the early segment-limit check (before overlap detection) only fires
/// for NON-overlapping segments (line 70-72 in segment.rs fires when segments.len() >= max_segments
/// BEFORE the overlap loop). For overlapping inserts, the mid-loop check (inside the gap
/// insertion loop at line 178) is the relevant gate.
///
/// To exercise BC-2.04.045 INV-2 (overlap detected before limit fires):
///   max_segments=3; 2 segments in map (early check: 2 < 3, passes).
///   New segment overlaps one existing segment AND has 2 gaps.
///   After inserting first gap (map now has 3 = max_segments), second gap can't be inserted →
///   mid-loop SegmentLimitReached, but overlap_count was already incremented during detection.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_045_segment_limit_overlapping_path_increments_overlap_count() {
    // max_segments=3; 2 segments in map at offsets [1,4) and [10,13).
    // New segment: offset [1,16) = 15 bytes (overlaps [1,4) and [10,13), gaps at [4,10) and [13,16)).
    // Overlap detected → overlap_count++ (before gap insertion loop).
    // Gap [4,10) inserted → map now has 3 = max_segments.
    // Gap [13,16): 3 >= 3 → SegmentLimitReached.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 3;
    let max_receive_window: usize = 10_485_760;

    // Insert 2 segments: offset [1,4) and offset [10,13)
    dir.insert_segment(1001, b"AAA", max_depth, max_segments, max_receive_window);
    dir.insert_segment(1010, b"BBB", max_depth, max_segments, max_receive_window);
    assert_eq!(
        dir.segment_count(),
        2,
        "AC-018 precondition: 2 segments in map"
    );
    assert_eq!(dir.overlap_count, 0, "AC-018 precondition: overlap_count=0");

    // New segment: seq=1001, 15 bytes covering [1,16). Overlaps both existing segments.
    // Gaps: [4,10) = 6 bytes and [13,16) = 3 bytes.
    let result = dir.insert_segment(
        1001,
        b"AAABBBBBBBBBBCC",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        result,
        InsertResult::SegmentLimitReached,
        "BC-2.04.045 PC1: must return SegmentLimitReached after hitting mid-loop segment limit"
    );
    assert_eq!(
        dir.overlap_count, 1,
        "BC-2.04.045 INV-2: overlap_count must increment (overlap detected before mid-loop limit check)"
    );
    // Segment count: 2 original + 1 gap (first gap [4,10) was inserted before limit hit) = 3
    assert_eq!(
        dir.segment_count(),
        3,
        "AC-018: exactly 3 segments (2 original + first gap inserted before limit)"
    );
}

// --- AC-019 (BC-2.04.046 postconditions 1-3 and invariant 1) ---
/// When the BTreeMap fills to max_segments mid-way through a multi-gap insertion,
/// SegmentLimitReached is returned, earlier gaps are in the map, later gaps are dropped,
/// and buffered_bytes has increased only by the bytes of the inserted gaps.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_046_segment_limit_partial_insertion_mid_loop() {
    // Setup: max_segments=3; insert 2 segments leaving a gap between them.
    // Then insert a segment that spans both existing segments plus a tail extension.
    // This creates TWO gaps to fill; the first gap consumes the last slot → SegmentLimitReached.
    // Earlier gap: inserted (in map). Later gap: dropped.
    // buffered_bytes increases only by the inserted gap bytes.
    //
    // Canonical layout:
    //   Existing: offset [1,4) = b"AAA", offset [10,13) = b"BBB"  (2 of 3 slots used)
    //   New: ISN+1 to ISN+15 = b"AAABBBBBBBBBBCC" (15 bytes, seq=1001, offset=[1,16))
    //   Gap 1: [4,10) = 6 bytes (fills last slot → segment count = 3)
    //   Gap 2: [13,16) = 3 bytes (cannot insert, limit hit) → dropped
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 3;
    let max_receive_window: usize = 10_485_760;

    // Insert 2 segments: offset [1,4) and offset [10,13)
    dir.insert_segment(1001, b"AAA", max_depth, max_segments, max_receive_window);
    dir.insert_segment(1010, b"BBB", max_depth, max_segments, max_receive_window);
    let before_buffered = dir.buffered_bytes();
    assert_eq!(
        before_buffered, 6,
        "AC-019 precondition: 6 bytes buffered (3+3)"
    );
    assert_eq!(dir.segment_count(), 2, "AC-019 precondition: 2 segments");

    // Insert spanning segment with 2 insertable gaps
    let result = dir.insert_segment(
        1001,
        b"AAABBBBBBBBBBCC",
        max_depth,
        max_segments,
        max_receive_window,
    );

    // BC-2.04.046 PC1: SegmentLimitReached is returned
    assert_eq!(
        result,
        InsertResult::SegmentLimitReached,
        "BC-2.04.046 PC1: must return SegmentLimitReached when limit hit mid-loop"
    );

    // BC-2.04.046 PC2: earlier gap [4,10) is in the map; later gap [13,16) is NOT
    assert_eq!(
        dir.segment_count(),
        3,
        "BC-2.04.046 PC2: exactly one gap inserted → segment_count must be 3 (2 original + 1 gap)"
    );
    assert!(
        dir.has_segment_at(4),
        "BC-2.04.046 PC2: first gap at offset 4 must be present in the map"
    );
    assert!(
        !dir.has_segment_at(13),
        "BC-2.04.046 PC2: second gap at offset 13 must NOT be in the map (dropped)"
    );

    // BC-2.04.046 PC3: buffered_bytes increased only by the 6 inserted gap bytes
    assert_eq!(
        dir.buffered_bytes(),
        before_buffered + 6,
        "BC-2.04.046 PC3: buffered_bytes must increase by exactly 6 (gap [4,10)) — NOT by 9 (both gaps)"
    );
}

// --- EC-001 (depth: segment exactly at max_depth, no truncation needed) ---
/// Segment exactly at max_depth (no truncation needed): Inserted; no Truncated result.
#[test]
fn test_story_018_ec001_segment_exactly_at_max_depth_is_inserted() {
    // max_depth=10; insert exactly 10 bytes → reassembled(0)+buffered(0)+new(10) = 10,
    // which is NOT > 10, so no Truncated. The check is > (strictly greater than).
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    let result = dir.insert_segment(
        1001,
        b"AAAAAAAAAA",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        result,
        InsertResult::Inserted,
        "EC-001: exactly max_depth bytes must return Inserted (check is >, not >=)"
    );
    assert!(
        !dir.depth_exceeded,
        "EC-001: depth_exceeded must remain false"
    );
    assert_eq!(dir.buffered_bytes(), 10, "EC-001: all 10 bytes stored");
}

// --- EC-002 (depth: segment crosses depth limit by 1 byte) ---
/// Segment crosses depth limit by 1 byte: depth is exceeded; depth_exceeded=true.
///
/// Implementation note: when allowed=0 (buffer is already full to max_depth),
/// the code returns DepthExceeded (not Truncated), because there is no data to
/// truncate — allowed=0 means we'd store zero bytes, which is a pure rejection.
/// Truncated is returned only when allowed > 0 (partial data can be stored).
/// This test verifies the boundary: exactly-full buffer + 1 new byte → DepthExceeded.
#[test]
fn test_story_018_ec002_segment_crosses_depth_by_one_byte_truncated() {
    // max_depth=10; insert 10 bytes (exactly at depth, buffer full);
    // then insert 1 more byte → allowed=0 → DepthExceeded (not Truncated).
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    // Fill exactly to depth limit (10 bytes)
    let r1 = dir.insert_segment(
        1001,
        b"AAAAAAAAAA",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        r1,
        InsertResult::Inserted,
        "EC-002 precondition: 10 bytes inserted"
    );
    assert_eq!(
        dir.buffered_bytes(),
        10,
        "EC-002 precondition: 10 bytes buffered"
    );

    // Insert 1 more byte → allowed = max_depth - (reassembled + buffered) = 10 - (0+10) = 0.
    // When allowed == 0, the implementation returns DepthExceeded (segment is fully rejected).
    let r2 = dir.insert_segment(1011, b"B", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r2,
        InsertResult::DepthExceeded,
        "EC-002: when buffer is full (allowed=0), 1 new byte returns DepthExceeded (pure rejection, not partial truncation)"
    );

    // buffered_bytes unchanged (no bytes stored)
    assert_eq!(
        dir.buffered_bytes(),
        10,
        "EC-002: buffered_bytes must be unchanged at 10 (no bytes stored when allowed=0)"
    );
    assert!(dir.depth_exceeded, "EC-002: depth_exceeded must be true");
}

// --- EC-003 (depth: two segments after depth hit) ---
/// Two segments after depth hit: both return DepthExceeded; depth_exceeded flag is
/// permanent after first Truncated result.
#[test]
fn test_story_018_ec003_two_segments_after_depth_hit_both_depth_exceeded() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 5;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    // Trigger Truncated
    dir.insert_segment(1001, b"AA", max_depth, max_segments, max_receive_window);
    let r_trunc = dir.insert_segment(1003, b"BBBBBB", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r_trunc,
        InsertResult::Truncated,
        "EC-003 precondition: Truncated fired"
    );
    assert!(
        dir.depth_exceeded,
        "EC-003 precondition: depth_exceeded=true"
    );

    // First post-Truncated segment: DepthExceeded
    let r1 = dir.insert_segment(1020, b"CCC", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r1,
        InsertResult::DepthExceeded,
        "EC-003: first segment after Truncated must return DepthExceeded"
    );

    // Second post-Truncated segment: also DepthExceeded
    let r2 = dir.insert_segment(1030, b"DDD", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r2,
        InsertResult::DepthExceeded,
        "EC-003: second segment after Truncated must return DepthExceeded (flag is permanent)"
    );
}

// --- EC-004 (out-of-window: segment at exact receive window boundary) ---
/// Segment at exact receive window boundary is Inserted (boundary is exclusive: > not >=).
#[test]
fn test_story_018_ec004_segment_at_exact_window_boundary_inserted() {
    // Distinct from AC-012: this uses a different ISN and window size to prove
    // generality. ISN=500 → base_offset=1; max_receive_window=50 → limit=51.
    // seq = 500 + 51 = 551 → offset=51 = limit exactly → must be Inserted (exclusive boundary).
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(500);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 50;

    // offset=51 is exactly at base_offset(1)+window(50)=51 → should be Inserted
    let result = dir.insert_segment(551, b"EDGE", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result,
        InsertResult::Inserted,
        "EC-004: segment at exact window boundary (offset == base + window) must be Inserted"
    );

    // offset=52 is one beyond limit → OutOfWindow
    let result2 = dir.insert_segment(552, b"X", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result2,
        InsertResult::OutOfWindow,
        "EC-004: segment one beyond boundary (offset > base + window) must be OutOfWindow"
    );
}

// --- EC-005 (out-of-window: segment 1 byte beyond receive window) ---
/// Segment 1 byte beyond receive window: OutOfWindow; out_of_window_count=1.
#[test]
fn test_story_018_ec005_segment_one_byte_beyond_window_out_of_window() {
    // ISN=1000 → base_offset=1; max_receive_window=100 → limit=101.
    // offset=102 (1 beyond limit) → OutOfWindow; out_of_window_count increments to 1.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 100;

    // offset = 1000+102 - 1000 = 102 > limit(101) → OutOfWindow
    let result = dir.insert_segment(1102, b"X", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result,
        InsertResult::OutOfWindow,
        "EC-005: 1 byte beyond window must return OutOfWindow"
    );
    assert_eq!(
        dir.out_of_window_count, 1,
        "EC-005: out_of_window_count must be 1"
    );
    assert_eq!(dir.buffered_bytes(), 0, "EC-005: no bytes stored");
}

// --- EC-006 (out-of-window: base_offset near u64::MAX) ---
/// base_offset near u64::MAX: saturating_add prevents overflow; OutOfWindow returned correctly.
#[test]
fn test_story_018_ec006_base_offset_near_u64_max_saturating_add() {
    // We cannot set base_offset directly (it's derived from ISN and flush).
    // Instead, use a concrete ISN near u32::MAX and verify the saturating_add
    // behavior: when base_offset is large and window overflows u64, the check
    // uses saturating_add which caps at u64::MAX. Any segment offset < u64::MAX
    // will therefore be <= u64::MAX and accepted (not rejected) unless its
    // actual offset is truly beyond the non-saturated window.
    //
    // Practical approach: set ISN near u32::MAX so the base_offset after set_isn
    // is 1 (from ISN+1 arithmetic), then use a window small enough that we can
    // construct an out-of-window segment. This tests that the code path compiles
    // and returns the right result — the interesting edge here is when
    // base_offset itself is near u64::MAX, but we can't reach that in unit
    // testing without manually hacking the field. The compile-time correctness
    // of saturating_add is the critical safety property; the functional
    // test of out-of-window is already covered by EC-005 and AC-010.
    //
    // We directly verify: with a very large window (near usize::MAX cast to u64),
    // saturating_add must not overflow and must accept segments at reasonable offsets.
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(0xFFFF_FF00_u32);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    // usize::MAX window → saturating_add(base_offset, usize::MAX as u64) = u64::MAX (saturated)
    // Any segment offset <= u64::MAX should be accepted
    let max_receive_window: usize = usize::MAX;

    // seq=ISN+1=0xFFFF_FF01 → offset=1. With window=usize::MAX, limit = u64::MAX (saturated).
    // offset(1) > u64::MAX? No → Inserted.
    let result = dir.insert_segment(
        0xFFFF_FF01_u32,
        b"X",
        max_depth,
        max_segments,
        max_receive_window,
    );
    assert_eq!(
        result,
        InsertResult::Inserted,
        "EC-006: saturating_add must prevent overflow — segment within any sane offset must not be rejected"
    );
    assert_eq!(
        dir.out_of_window_count, 0,
        "EC-006: out_of_window_count must remain 0 when segment is within the saturated window"
    );
}

// --- EC-007 (segment limit: pure overlap, no gap) ---
/// STORY-018 EC-007 / BC-2.04.045 v1.3 EC-002: when `segments.len() >= max_segments`
/// at function entry, the early guard at `segment.rs:70-72` fires unconditionally —
/// including for pure-overlap segments that would otherwise return Duplicate /
/// ConflictingOverlap when the map is below capacity. This is the brownfield-confirmed
/// semantics formalized in BC-2.04.045 v1.3.
#[test]
fn test_story_018_ec007_full_map_pure_overlap_returns_segment_limit_reached() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 2;
    let max_receive_window: usize = 10_485_760;

    // Insert 2 adjacent segments filling the map: [1,4) and [4,7)
    dir.insert_segment(1001, b"AAA", max_depth, max_segments, max_receive_window);
    dir.insert_segment(1004, b"BBB", max_depth, max_segments, max_receive_window);
    assert_eq!(dir.segment_count(), 2, "EC-007 precondition: map is full");

    // Insert a new segment fully covered by union of the two existing (matching bytes).
    // BC-2.04.045 v1.3 EC-002: early guard at segment.rs:70-72 fires unconditionally
    // when segments.len() >= max_segments, returning SegmentLimitReached before any
    // overlap detection — even for pure-overlap inserts that add no new entries.
    let result = dir.insert_segment(1001, b"AAABBB", max_depth, max_segments, max_receive_window);
    assert_eq!(
        result,
        InsertResult::SegmentLimitReached,
        "EC-007: early guard fires before overlap detection when map is full (BC-2.04.045 v1.3 EC-002)"
    );
}

// --- EC-009 (small_segment_run: OutOfWindow result after 2 small segments) ---
/// OutOfWindow result after 2 small segments: small_segment_run unchanged at 2.
/// Tests that the out_of_window path on FlowDirection does not reset the run counter.
///
/// Note: small_segment_run is maintained by the *engine* in insert_payload_segment
/// (mod.rs:356-370), not by insert_segment in segment.rs. At the segment level,
/// FlowDirection.small_segment_run starts at 0 and is never modified by insert_segment.
/// This test verifies that an OutOfWindow result does NOT modify small_segment_run
/// (the field stays 0 since insert_segment never touches it — modification is the
/// engine shell's responsibility).
#[test]
fn test_story_018_ec009_out_of_window_does_not_change_small_segment_run() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 10_485_760;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 100; // small window

    // Manually set small_segment_run to 2 to simulate state after 2 small segments
    dir.small_segment_run = 2;

    // Now send an OutOfWindow segment (offset=102 > limit=101)
    let r = dir.insert_segment(1102, b"X", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r,
        InsertResult::OutOfWindow,
        "EC-009: segment must be OutOfWindow"
    );

    // small_segment_run must be unchanged at 2 (insert_segment does not touch it)
    assert_eq!(
        dir.small_segment_run, 2,
        "EC-009: OutOfWindow result must not change small_segment_run (stays at 2)"
    );
}

// --- EC-010 (small_segment_run: DepthExceeded result after 3 small segments) ---
/// DepthExceeded result after 3 small segments: small_segment_run unchanged at 3.
/// Verifies the segment-level depth_exceeded path does not touch the run counter.
///
/// Note: like EC-009, small_segment_run is an engine-level counter (mod.rs:356-370),
/// not a segment-level one. At the insert_segment level, the field is not read or
/// written. This test verifies that DepthExceeded does not modify small_segment_run
/// by testing directly on FlowDirection with a pre-set counter value.
#[test]
fn test_story_018_ec010_depth_exceeded_does_not_change_small_segment_run() {
    let mut dir = wirerust::reassembly::flow::FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 5;
    let max_segments: usize = 10_000;
    let max_receive_window: usize = 10_485_760;

    // Trigger depth_exceeded: insert 3 bytes then a 5-byte segment → Truncated → depth_exceeded=true
    dir.insert_segment(1001, b"AAA", max_depth, max_segments, max_receive_window);
    let r_trunc = dir.insert_segment(1004, b"BBBBB", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r_trunc,
        InsertResult::Truncated,
        "EC-010 precondition: Truncated fired"
    );
    assert!(
        dir.depth_exceeded,
        "EC-010 precondition: depth_exceeded=true"
    );

    // Manually set small_segment_run to 3 to simulate state after 3 small segments
    dir.small_segment_run = 3;

    // Now send a DepthExceeded segment
    let r = dir.insert_segment(1020, b"CCC", max_depth, max_segments, max_receive_window);
    assert_eq!(
        r,
        InsertResult::DepthExceeded,
        "EC-010: segment must return DepthExceeded"
    );

    // small_segment_run must be unchanged at 3 (insert_segment does not touch it)
    assert_eq!(
        dir.small_segment_run, 3,
        "EC-010: DepthExceeded result must not change small_segment_run (stays at 3)"
    );
}
