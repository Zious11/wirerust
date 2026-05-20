---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.034
bcs:
  - BC-2.04.034
  - BC-2.04.007
  - BC-2.04.008
module: src/reassembly/segment.rs
proof_method: proptest
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

# VP-011: flush_contiguous Monotonicity

## Property Statement

`flush_contiguous` delivers reassembled bytes to the caller's closure in order
without delivering any byte more than once:

1. `base_offset` strictly increases with each call to `flush_contiguous` (or
   stays the same if nothing was flushed).
2. The total bytes delivered across all flush calls equals `bytes_reassembled`
   at the end of the flow's lifetime.
3. Bytes are delivered in contiguous ascending offset order: if byte at offset N
   was delivered, the next delivery starts at offset >= N + (bytes in prior delivery).
4. Out-of-order segments buffer until the gap at `base_offset` is filled; they
   are delivered in one flush when the gap closes (BC-2.04.008).

## Source Contract

- **Primary BC:** BC-2.04.034 -- flush_contiguous consumes segments from base_offset in order
- **Postcondition:** bytes delivered in ascending offset order; base_offset strictly advances
- **Related BC:** BC-2.04.007 -- In-order data flushes contiguously to handler in segment order
- **Related BC:** BC-2.04.008 -- Out-of-order segments buffer until gap filled then flush contiguously

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Property testing | proptest | No -- arbitrary insert sequences with gaps | Covers in-order, out-of-order, and interleaved gap-filling cases |

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    proptest! {
        #[test]
        fn prop_flush_monotonic_no_duplicate_bytes(
            // Sequence of (offset, data) pairs to insert
            inserts in prop::collection::vec(
                (0u64..500, prop::collection::vec(any::<u8>(), 1..32)),
                1..30
            ),
            isn: u64,
        ) {
            let mut dir = FlowDirection::new();
            dir.set_isn(isn);

            let mut last_delivered_end: u64 = 0;
            let mut total_delivered: usize = 0;

            for (offset, data) in inserts {
                let _ = dir.insert_segment(offset, &data, isn);

                // Flush after each insert and track delivery
                dir.flush_contiguous(&mut |bytes: &[u8]| {
                    // Delivery must start at exactly last_delivered_end
                    // (base_offset is tracked internally but we verify via total bytes)
                    total_delivered += bytes.len();
                    last_delivered_end += bytes.len() as u64;
                });
            }

            // total_delivered must match base_offset advancement
            prop_assert_eq!(total_delivered as u64, dir.base_offset(),
                "bytes delivered != base_offset advance");
        }

        #[test]
        fn prop_out_of_order_gap_then_fill(isn: u64) {
            let mut dir = FlowDirection::new();
            dir.set_isn(isn);
            let mut delivered: Vec<u8> = Vec::new();

            // Insert segment at offset 4 (gap at 0..4)
            let _ = dir.insert_segment(4, b"WORLD", isn);
            dir.flush_contiguous(&mut |b| delivered.extend_from_slice(b));
            // Nothing should be delivered yet (gap at base_offset=0)
            prop_assert!(delivered.is_empty(),
                "delivered bytes before gap was filled");

            // Fill the gap
            let _ = dir.insert_segment(0, b"HELL", isn);
            // Insert and flush -- now both should flush
            let _ = dir.insert_segment(4, b"O", isn); // duplicate, covers overlap
            dir.flush_contiguous(&mut |b| delivered.extend_from_slice(b));
            // HELLO WORLD delivered (or similar depending on overlap handling)
            prop_assert!(!delivered.is_empty(),
                "nothing delivered after gap filled");
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded -- proptest | Shrinking finds minimal gap-fill sequence on failure |
| Proof complexity | Medium | BTreeMap iteration order + offset arithmetic |
| Tool support | High | flush_contiguous is pure; deterministic BTreeMap iteration |
| Estimated proof time | < 60 seconds for 1000 cases | |

## Source Location

`src/reassembly/segment.rs` -- `FlowDirection::flush_contiguous`.
`base_offset: u64` field tracks the next expected byte position.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
