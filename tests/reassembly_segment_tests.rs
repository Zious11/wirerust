use wirerust::reassembly::flow::FlowDirection;
use wirerust::reassembly::segment::InsertResult;

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

    // base_offset starts at 0; segment at offset=3 is buffered (gap at 0,1,2).
    // No flush yet.
    let flushed_empty = dir.flush_contiguous();
    assert!(
        flushed_empty.is_empty(),
        "BC-2.04.039 PC3: gap at offset 0 must prevent flush of offset-3 segment"
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
