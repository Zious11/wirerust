---
document_type: verification-property
level: L4
version: "2.1"
status: verified
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
verification_lock: true
proof_completed_date: "2026-06-02"
proof_file_hash: "00e89a767d6a9cf54236b4a48b5fe0367b4a9226cc3a21f47bee5a92831ae478"
verified_at_commit: "0855f25"
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v2.0: Phase-6 verification locked 2026-06-02 @ develop 0855f25. status→verified, verification_lock→true, proof_file_hash set (tests/reassembly_segment_tests.rs)."
  - "v2.1 (2026-06-13, PG-ARP-F2-007 anchor-drift sweep): Source Location line anchors corrected for segment.rs shifts. insert_segment: :39→:189. flush_contiguous: :236→:369. Lock fields unchanged."
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

API notes verified against `src/reassembly/` @ 0082a0c:
- `FlowDirection::set_isn(isn: u32)` -- ISN is `u32`, not u64.
- `FlowDirection::insert_segment(seq: u32, data: &[u8], max_depth: usize, max_segments: usize, max_receive_window: usize) -> InsertResult`
  -- 5 parameters; the first is a TCP sequence number (`u32`), not an offset. For
  tests that want a simple byte offset, pass `isn.wrapping_add(1)` as the seq for
  offset 1, etc. Using `seq = isn + 1` puts the segment at relative offset 1
  (seq_offset wrapping subtraction from ISN).
- `FlowDirection::flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)>` -- returns
  the flushed (offset, data) pairs; takes NO closure argument.
- `FlowDirection::buffered_bytes(&self) -> usize` -- public accessor (line 154).
- `FlowDirection::segments` is `pub(super)` BTreeMap; it is NOT accessible outside
  the `reassembly` module. The invariant check must be done inside the reassembly
  module or via the `debug_assert` in `memory_used()`. For proptest driven from an
  integration test, verify via `memory_used()` (which asserts the invariant in debug
  builds) rather than directly iterating `segments`.

```rust
// Located in src/reassembly/ (needs pub(super) or module-internal access)
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::flow::FlowDirection;
    use super::segment::InsertResult;

    // Probe parameters: reasonable caps that keep tests fast.
    const MAX_DEPTH: usize = 1_000_000;
    const MAX_SEGS: usize = 10_000;
    const MAX_WIN: usize = 1_000_000;

    #[derive(Clone, Debug)]
    enum SegmentOp {
        Insert { seq_delta: u32, data: Vec<u8> },
        Flush,
    }

    fn seg_op_strategy() -> impl Strategy<Value = SegmentOp> {
        prop_oneof![
            (0u32..1000, prop::collection::vec(any::<u8>(), 1..64))
                .prop_map(|(d, data)| SegmentOp::Insert { seq_delta: d, data }),
            Just(SegmentOp::Flush),
        ]
    }

    proptest! {
        #[test]
        fn prop_buffered_bytes_mirrors_btreemap_sum(
            ops in prop::collection::vec(seg_op_strategy(), 1..100),
            isn: u32,
        ) {
            let mut dir = FlowDirection::new();
            // set_isn takes u32 (src/reassembly/flow.rs:136).
            dir.set_isn(isn);

            for op in ops {
                match op {
                    SegmentOp::Insert { seq_delta, data } => {
                        // seq = ISN + 1 + delta => relative offset = 1 + delta
                        let seq = isn.wrapping_add(1).wrapping_add(seq_delta);
                        let _ = dir.insert_segment(
                            seq, &data, MAX_DEPTH, MAX_SEGS, MAX_WIN
                        );
                    }
                    SegmentOp::Flush => {
                        // flush_contiguous returns Vec<(u64, Vec<u8>)>; no closure
                        // (src/reassembly/segment.rs:369).
                        let _ = dir.flush_contiguous();
                    }
                }

                // Invariant: buffered_bytes counter == actual BTreeMap sum.
                // segments is pub(super); sum via memory_used() which contains
                // the debug_assert. In tests within this module, direct access
                // to `dir.segments` is allowed.
                let actual_sum: usize = dir.segments
                    .values()
                    .map(|v| v.len())
                    .sum();
                prop_assert_eq!(dir.buffered_bytes(), actual_sum,
                    "buffered_bytes mismatch after op");
            }
        }

        #[test]
        fn prop_buffered_bytes_zero_when_empty(isn: u32) {
            let mut dir = FlowDirection::new();
            dir.set_isn(isn);
            prop_assert_eq!(dir.buffered_bytes(), 0);
            // Insert then flush -- insert_segment(seq, data, max_depth, max_segs, max_win)
            let seq = isn.wrapping_add(1);
            let _ = dir.insert_segment(seq, b"hello", MAX_DEPTH, MAX_SEGS, MAX_WIN);
            let _ = dir.flush_contiguous();
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

`src/reassembly/flow.rs:90` -- `FlowDirection.buffered_bytes: usize` field (pub(super)).
`src/reassembly/flow.rs:154` -- `FlowDirection::buffered_bytes(&self) -> usize` public accessor.
`src/reassembly/segment.rs:189` -- `FlowDirection::insert_segment(seq: u32, data: &[u8], max_depth: usize, max_segments: usize, max_receive_window: usize) -> InsertResult`.
`src/reassembly/segment.rs:369` -- `FlowDirection::flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)>`.
`src/reassembly/flow.rs:89` -- `segments: BTreeMap<u64, Vec<u8>>` is `pub(super)`; test code
inside the `reassembly` module can access it directly for sum verification.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | 2026-06-02 | formal-verifier |
| Proof first passed | 2026-06-02 | formal-verifier |
| Locked (VERIFIED) | 2026-06-02 | spec-steward (Phase-6 gate) |
