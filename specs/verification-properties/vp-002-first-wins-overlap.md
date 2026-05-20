---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.018
bcs:
  - BC-2.04.018
  - BC-2.04.035
  - BC-2.04.036
  - BC-2.04.037
  - BC-2.04.038
  - BC-2.04.043
module: src/reassembly/segment.rs
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-002: First-Wins Overlap Policy

## Property Statement

When `insert_segment` is called with bytes that overlap an already-buffered range:

1. If the new bytes are identical to the buffered bytes in the overlapping range:
   `InsertResult::Duplicate` is returned; the buffer is unchanged.
2. If the new bytes DIFFER from the buffered bytes in the overlapping range:
   `InsertResult::ConflictingOverlap` is returned; the ORIGINAL buffered bytes
   are preserved (the new bytes lose); the buffer content at the overlapping
   positions is unchanged.
3. Gap-fill bytes (positions not yet buffered) are always accepted regardless of
   whether the same segment also contains conflicting overlap at other positions.
4. Adjacent segments that meet exactly at a byte boundary do NOT constitute overlap.

The "first byte received wins" policy is the forensic-correctness guarantee for
TCP evasion attack detection. A conflicting overlap is always surfaced as an
`InsertResult::ConflictingOverlap` that the engine converts to a finding.

## Source Contract

- **Primary BC:** BC-2.04.018 -- Conflicting overlap emits Anomaly/Likely/High finding with MITRE T1036
- **Postcondition:** `InsertResult::ConflictingOverlap` returned; original bytes preserved in buffer
- **Invariant:** INV-3 (First-Wins Overlap Policy, inv-01-core-invariants.md)
- **Related BC:** BC-2.04.035 -- Identical retransmission returns Duplicate; does not double-count bytes
- **Related BC:** BC-2.04.036 -- First-wins overlap: gap bytes added, existing bytes preserved
- **Related BC:** BC-2.04.037 -- Same-range conflicting overlap returns ConflictingOverlap; original wins
- **Related BC:** BC-2.04.038 -- Multi-segment full coverage returns Duplicate or ConflictingOverlap
- **Related BC:** BC-2.04.043 -- Adjacent segments at exact boundary do not count as overlap

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- small fixed-size byte arrays (4 bytes); bounded segment count | All insert orderings for 4-byte segments at bounded offsets |

## Proof Harness Skeleton

// Real signature (src/reassembly/segment.rs:39-46):
//   pub fn insert_segment(
//       &mut self,
//       seq: u32,
//       data: &[u8],
//       max_depth: usize,
//       max_segments: usize,
//       max_receive_window: usize,
//   ) -> InsertResult
//
// ISN is stored inside FlowDirection (self.isn: Option<u32>); there is NO `isn`
// parameter on insert_segment. Set the ISN via dir.set_isn(isn) or dir.infer_isn(seq)
// before the first insert.
//
// Available read accessors on FlowDirection (src/reassembly/flow.rs:150-177):
//   segment_at(offset: u64) -> Option<&[u8]>
//   has_segment_at(offset: u64) -> bool
//   segment_count() -> usize
//   buffered_bytes() -> usize
//   segments_is_empty() -> bool
// There is NO read_at method.

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // Bounded constants for Kani tractability
    const TEST_MAX_DEPTH: usize = 65_536;
    const TEST_MAX_SEGMENTS: usize = 1_024;
    const TEST_MAX_WINDOW: usize = 65_536;

    #[kani::proof]
    fn verify_first_wins_conflicting_overlap() {
        let mut dir = FlowDirection::new();
        // Set ISN so that seq=0 maps to offset=1 (ISN+1 is first data byte).
        // Use infer_isn(1) which sets isn=0, base_offset=1.
        // For simplicity use seq=1 so offset=seq.wrapping_sub(isn)=1.
        dir.set_isn(0);
        // Insert original bytes at seq=1 (offset=1)
        let original: [u8; 4] = kani::any();
        let result1 = dir.insert_segment(
            1,
            &original,
            TEST_MAX_DEPTH,
            TEST_MAX_SEGMENTS,
            TEST_MAX_WINDOW,
        );
        assert!(matches!(result1, InsertResult::Inserted));

        // Insert conflicting bytes at same seq (must differ in at least one position)
        let conflicting: [u8; 4] = kani::any();
        kani::assume(conflicting != original); // ensure a difference exists

        let result2 = dir.insert_segment(
            1,
            &conflicting,
            TEST_MAX_DEPTH,
            TEST_MAX_SEGMENTS,
            TEST_MAX_WINDOW,
        );
        assert!(matches!(result2, InsertResult::ConflictingOverlap));

        // Original bytes must still be in buffer unchanged.
        // Use segment_at(offset) to read the buffered bytes at ISN-relative offset 1.
        let buffered = dir.segment_at(1).expect("segment must exist after insert");
        assert_eq!(buffered, &original[..]);
    }

    #[kani::proof]
    fn verify_duplicate_does_not_change_buffer() {
        let mut dir = FlowDirection::new();
        dir.set_isn(0);
        let bytes: [u8; 4] = kani::any();
        let _ = dir.insert_segment(1, &bytes, TEST_MAX_DEPTH, TEST_MAX_SEGMENTS, TEST_MAX_WINDOW);
        // Insert identical bytes again
        let result = dir.insert_segment(1, &bytes, TEST_MAX_DEPTH, TEST_MAX_SEGMENTS, TEST_MAX_WINDOW);
        assert!(matches!(result, InsertResult::Duplicate));
    }

    #[kani::proof]
    fn verify_adjacent_boundary_not_overlap() {
        let mut dir = FlowDirection::new();
        dir.set_isn(0);
        let bytes_a: [u8; 4] = kani::any();
        let bytes_b: [u8; 4] = kani::any();
        // seq=1 -> offset=1; insert 4 bytes covering offsets 1..5
        let _ = dir.insert_segment(1, &bytes_a, TEST_MAX_DEPTH, TEST_MAX_SEGMENTS, TEST_MAX_WINDOW);
        // seq=5 -> offset=5; starts exactly at end of first segment -- NOT overlap
        let result = dir.insert_segment(5, &bytes_b, TEST_MAX_DEPTH, TEST_MAX_SEGMENTS, TEST_MAX_WINDOW);
        assert!(!matches!(result, InsertResult::ConflictingOverlap));
        assert!(!matches!(result, InsertResult::Duplicate));
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | 4-byte arrays; Kani handles this well |
| Proof complexity | Medium | BTreeMap operations; overlap logic has multiple branches |
| Tool support | High | FlowDirection is pure; no I/O or global state |
| Estimated proof time | 2-5 minutes | BTreeMap iteration within Kani's bounded model |

## Source Location

`src/reassembly/segment.rs` -- `FlowDirection::insert_segment` and `flush_contiguous`.

Overlap logic: the BTreeMap range query checks for existing entries whose byte ranges
intersect the new segment's `[offset, offset+len)` range.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
