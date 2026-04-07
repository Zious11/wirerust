use wirerust::reassembly::flow::FlowDirection;
use wirerust::reassembly::segment::InsertResult;

#[test]
fn test_insert_single_segment() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let result = dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(dir.segments.len(), 1);
    assert_eq!(dir.segments.get(&1), Some(&b"hello".to_vec()));
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
    assert!(dir.segments.is_empty());
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
    assert!(dir.segments.is_empty());
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
    assert_eq!(dir.segments.len(), 1); // No duplicate stored
    assert_eq!(dir.buffered_bytes, 5); // counter must not double-count
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

#[test]
fn test_small_segment_tracking() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert small segments
    for i in 0..5u32 {
        let seq = 1001 + i;
        dir.insert_segment(seq, b"a", 10_485_760, 10_000, 10_485_760);
    }

    assert_eq!(dir.small_segment_count, 5);
}

#[test]
fn test_buffered_bytes_after_insert() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);
    dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes, 5);
    dir.insert_segment(1006, b"world", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes, 10);
}

#[test]
fn test_buffered_bytes_after_overlap() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);
    dir.insert_segment(1001, b"AAABBB", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes, 6);
    dir.insert_segment(1004, b"XXXCC", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes, 8); // 6 original + 2 gap bytes
}

#[test]
fn test_buffered_bytes_after_flush() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    dir.insert_segment(1001, b"hello", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes, 5);

    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 1);
    assert_eq!(dir.buffered_bytes, 0);
}

#[test]
fn test_buffered_bytes_partial_flush() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert segment at offset 1 (contiguous) and offset 10 (gap)
    dir.insert_segment(1001, b"aaa", 10_485_760, 10_000, 10_485_760);
    dir.insert_segment(1010, b"bbb", 10_485_760, 10_000, 10_485_760);
    assert_eq!(dir.buffered_bytes, 6);

    // Flush only flushes contiguous segment at offset 1
    let flushed = dir.flush_contiguous();
    assert_eq!(flushed.len(), 1);
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(dir.buffered_bytes, 3); // "bbb" remains buffered
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
    assert_eq!(dir.segments.len(), 2);
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
    assert_eq!(dir.segments.len(), 3);

    // Next non-overlapping insert should return SegmentLimitReached
    let result = dir.insert_segment(1030, b"ddd", 10_485_760, max_segments, 10_485_760);
    assert_eq!(result, InsertResult::SegmentLimitReached);
    assert_eq!(dir.segments.len(), 3); // No new segment added
}

#[test]
fn test_segment_limit_gap_loop_full_rejection() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_segments: usize = 2;

    // Insert two segments with a gap between them (offsets 1-3 and 10-12)
    dir.insert_segment(1001, b"AAA", 10_485_760, max_segments, 10_485_760);
    dir.insert_segment(1010, b"BBB", 10_485_760, max_segments, 10_485_760);
    assert_eq!(dir.segments.len(), 2);

    // Now insert a segment that overlaps the first and has a gap to fill (offset 1-6)
    // Gap is at offset 4-6. But segments are at capacity (2), so the gap can't be inserted.
    let result = dir.insert_segment(1001, b"AAAXXX", 10_485_760, max_segments, 10_485_760);
    assert_eq!(result, InsertResult::SegmentLimitReached);
    assert_eq!(dir.segments.len(), 2); // No new segment added
}

#[test]
fn test_segment_limit_gap_loop_partial_insertion() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_segments: usize = 3;

    // Insert two segments: offset 1-3 and offset 10-12, leaving gaps at 4-9 and 13+
    dir.insert_segment(1001, b"AAA", 10_485_760, max_segments, 10_485_760);
    dir.insert_segment(1010, b"BBB", 10_485_760, max_segments, 10_485_760);
    assert_eq!(dir.segments.len(), 2);

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
    assert_eq!(dir.segments.len(), 3); // One gap inserted, hit limit before second

    // Verify the first gap was filled at offset 4 with 6 bytes covering [4,10)
    assert!(dir.segments.contains_key(&4));
    // Second gap (starting at offset 13) was NOT inserted
    assert!(!dir.segments.contains_key(&13));
}
