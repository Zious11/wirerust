use wirerust::reassembly::flow::FlowDirection;
use wirerust::reassembly::segment::{flush_contiguous, insert_segment, InsertResult};

#[test]
fn test_insert_single_segment() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let result = insert_segment(&mut dir, 1001, b"hello", 10_485_760);
    assert_eq!(result, InsertResult::Inserted);
    assert_eq!(dir.segments.len(), 1);
    assert_eq!(dir.segments.get(&1), Some(&b"hello".to_vec()));
}

#[test]
fn test_flush_contiguous_single() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"hello", 10_485_760);

    let flushed = flush_contiguous(&mut dir);
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

    insert_segment(&mut dir, 1001, b"aaa", 10_485_760);
    insert_segment(&mut dir, 1004, b"bbb", 10_485_760);

    let flushed = flush_contiguous(&mut dir);
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
    insert_segment(&mut dir, 1004, b"bbb", 10_485_760);
    let flushed = flush_contiguous(&mut dir);
    assert!(flushed.is_empty()); // Can't flush — gap at offset 1

    // Now insert segment 1
    insert_segment(&mut dir, 1001, b"aaa", 10_485_760);
    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed.len(), 2); // Both flush now
    assert_eq!(flushed[0].1, b"aaa");
    assert_eq!(flushed[1].1, b"bbb");
    assert_eq!(dir.base_offset, 7);
}

#[test]
fn test_retransmission_dedup() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"hello", 10_485_760);
    let result = insert_segment(&mut dir, 1001, b"hello", 10_485_760);
    assert_eq!(result, InsertResult::Duplicate);
    assert_eq!(dir.segments.len(), 1); // No duplicate stored
}

#[test]
fn test_overlap_first_wins() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert "AAABBB" at offset 1
    insert_segment(&mut dir, 1001, b"AAABBB", 10_485_760);

    // Overlapping insert: "XXXCC" at offset 4 (overlaps with "BBB" at 4-6)
    let result = insert_segment(&mut dir, 1004, b"XXXCC", 10_485_760);
    assert_eq!(result, InsertResult::PartialOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Flush and verify: first 6 bytes from original, then "CC" from new
    let flushed = flush_contiguous(&mut dir);
    let all_bytes: Vec<u8> = flushed.iter().flat_map(|(_, data)| data.iter().copied()).collect();
    assert_eq!(&all_bytes, b"AAABBBCC");
}

#[test]
fn test_overlap_conflicting_data_detected() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    insert_segment(&mut dir, 1001, b"AAAA", 10_485_760);

    // Same range, different data
    let result = insert_segment(&mut dir, 1001, b"BBBB", 10_485_760);
    assert_eq!(result, InsertResult::ConflictingOverlap);
    assert_eq!(dir.overlap_count, 1);

    // Original data preserved (first-wins)
    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed[0].1, b"AAAA");
}

#[test]
fn test_sequence_wraparound() {
    let mut dir = FlowDirection::new();
    // ISN near wraparound
    dir.set_isn(0xFFFF_FFF0);

    // First data byte at ISN+1 = 0xFFFF_FFF1, offset = 1
    insert_segment(&mut dir, 0xFFFF_FFF1, b"before", 10_485_760);
    // Next segment wraps: seq = 0xFFFF_FFF1 + 6 = 0xFFFF_FFF7, offset = 7
    insert_segment(&mut dir, 0xFFFF_FFF7, b"wrap", 10_485_760);
    // Another after wrap: seq = 0xFFFF_FFFB, offset = 11
    insert_segment(&mut dir, 0xFFFF_FFFB, b"around", 10_485_760);

    let flushed = flush_contiguous(&mut dir);
    let all_bytes: Vec<u8> = flushed.iter().flat_map(|(_, data)| data.iter().copied()).collect();
    assert_eq!(&all_bytes, b"beforewraparound");
}

#[test]
fn test_small_segment_tracking() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    // Insert small segments
    for i in 0..5u32 {
        let seq = 1001 + i;
        insert_segment(&mut dir, seq, &[b'a'], 10_485_760);
    }

    assert_eq!(dir.small_segment_count, 5);
}

#[test]
fn test_depth_limit_truncation() {
    let mut dir = FlowDirection::new();
    dir.set_isn(1000);

    let max_depth: usize = 100; // small for testing
    let data = vec![b'A'; 80];
    insert_segment(&mut dir, 1001, &data, max_depth);
    flush_contiguous(&mut dir);
    assert_eq!(dir.reassembled_bytes, 80);
    assert!(!dir.depth_exceeded);

    // This should be truncated to 20 bytes
    let data2 = vec![b'B'; 50];
    let result = insert_segment(&mut dir, 1081, &data2, max_depth);
    assert_eq!(result, InsertResult::Truncated);
    assert!(dir.depth_exceeded);

    let flushed = flush_contiguous(&mut dir);
    assert_eq!(flushed[0].1.len(), 20); // truncated from 50 to 20
    assert_eq!(dir.reassembled_bytes, 100);
}
