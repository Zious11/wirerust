---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.047
bcs:
  - BC-2.04.047
  - BC-2.04.030
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

# VP-010: buffered_bytes Invariant

## Property Statement

The `buffered_bytes` counter on `FlowDirection` mirrors the sum of the lengths of
all `Segment` values currently in the `BTreeMap` at all times:

`buffered_bytes == segments.values().map(|s| s.data.len()).sum()`

This equality holds after every operation that modifies the segment map:
- `insert_segment` (insert, gap-fill, duplicate, conflicting overlap)
- `flush_contiguous` (removes consumed segments from the map)

The counter must not go negative (it is a `usize`). When the buffer is empty,
`buffered_bytes == 0`.

## Source Contract

- **Primary BC:** BC-2.04.047 -- buffered_bytes mirrors segment size sum after all insert/overlap/flush ops
- **Postcondition:** counter == BTreeMap segment length sum at every observable point
- **Related BC:** BC-2.04.030 -- bytes_reassembled equals total bytes delivered to handler at end

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Property testing | proptest | No -- arbitrary operation sequences | All orderings of insert and flush on bounded segment sequences |

## Proof Harness Skeleton

```rust
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::*;

    #[derive(Clone, Debug)]
    enum SegmentOp {
        Insert { offset: u64, data: Vec<u8> },
        Flush,
    }

    fn seg_op_strategy() -> impl Strategy<Value = SegmentOp> {
        prop_oneof![
            (0u64..1000, prop::collection::vec(any::<u8>(), 1..64))
                .prop_map(|(offset, data)| SegmentOp::Insert { offset, data }),
            Just(SegmentOp::Flush),
        ]
    }

    proptest! {
        #[test]
        fn prop_buffered_bytes_mirrors_btreemap_sum(
            ops in prop::collection::vec(seg_op_strategy(), 1..100),
            isn: u64,
        ) {
            let mut dir = FlowDirection::new();
            // Set a dummy ISN so inserts don't return IsnMissing
            dir.set_isn(isn);

            for op in ops {
                match op {
                    SegmentOp::Insert { offset, data } => {
                        let _ = dir.insert_segment(offset, &data, isn);
                    }
                    SegmentOp::Flush => {
                        dir.flush_contiguous(&mut |_bytes| {});
                    }
                }

                // Invariant: counter == actual sum
                let actual_sum: usize = dir.segments()
                    .values()
                    .map(|s| s.data.len())
                    .sum();
                prop_assert_eq!(dir.buffered_bytes(), actual_sum,
                    "buffered_bytes mismatch after op");
            }
        }

        #[test]
        fn prop_buffered_bytes_zero_when_empty(isn: u64) {
            let mut dir = FlowDirection::new();
            dir.set_isn(isn);
            prop_assert_eq!(dir.buffered_bytes(), 0);
            // Insert then flush
            let _ = dir.insert_segment(0, b"hello", isn);
            dir.flush_contiguous(&mut |_| {});
            prop_assert_eq!(dir.buffered_bytes(), 0);
        }
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Unbounded -- proptest generates random op sequences | Shrinking on failure isolates the minimal sequence |
| Proof complexity | Low | Simple counter-vs-sum comparison |
| Tool support | High | FlowDirection is pure; BTreeMap is deterministic |
| Estimated proof time | < 30 seconds for 1000 cases | Fast counter check per operation |

## Source Location

`src/reassembly/segment.rs` -- `FlowDirection.buffered_bytes: usize` field.
Updated in `insert_segment` (add) and `flush_contiguous` (subtract).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
