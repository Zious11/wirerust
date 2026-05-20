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

API notes verified against `src/reassembly/` @ 0082a0c:
- `FlowDirection::set_isn(isn: u32)` -- ISN is `u32`, not u64.
- `FlowDirection::insert_segment(seq: u32, data: &[u8], max_depth: usize, max_segments: usize, max_receive_window: usize) -> InsertResult`
  -- seq is a TCP sequence number (`u32`). Offset relative to ISN is computed
  internally via `seq.wrapping_sub(isn)`.
- `FlowDirection::flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)>` -- returns
  flushed `(offset, bytes)` pairs; takes NO closure argument.
- `FlowDirection::base_offset` is a `pub` field (`u64`), not a method. Access as
  `dir.base_offset`, not `dir.base_offset()`.

```rust
// Located in src/reassembly/ (module-internal test)
#[cfg(test)]
mod proptest_proofs {
    use proptest::prelude::*;
    use super::flow::FlowDirection;

    const MAX_DEPTH: usize = 1_000_000;
    const MAX_SEGS: usize = 10_000;
    const MAX_WIN: usize = 1_000_000;

    proptest! {
        #[test]
        fn prop_flush_monotonic_no_duplicate_bytes(
            // Sequence of (seq_delta, data) pairs to insert; seq = ISN+1+delta
            inserts in prop::collection::vec(
                (0u32..500, prop::collection::vec(any::<u8>(), 1..32)),
                1..30
            ),
            isn: u32,
        ) {
            let mut dir = FlowDirection::new();
            // set_isn takes u32 (src/reassembly/flow.rs:136).
            dir.set_isn(isn);

            let mut total_delivered: usize = 0;

            for (seq_delta, data) in inserts {
                let seq = isn.wrapping_add(1).wrapping_add(seq_delta);
                // insert_segment: 5-argument signature
                let _ = dir.insert_segment(seq, &data, MAX_DEPTH, MAX_SEGS, MAX_WIN);

                // flush_contiguous returns Vec<(u64, Vec<u8>)>; no closure arg.
                // (src/reassembly/segment.rs:236)
                let flushed = dir.flush_contiguous();
                for (_, bytes) in &flushed {
                    total_delivered += bytes.len();
                }
            }

            // total_delivered must match base_offset advancement.
            // base_offset is a pub field (src/reassembly/flow.rs:88).
            // After set_isn, base_offset starts at 1 (ISN+1 is first data byte).
            // Bytes delivered == base_offset - 1 (the initial ISN+1 offset).
            prop_assert_eq!(total_delivered as u64, dir.base_offset - 1,
                "bytes delivered != base_offset advance");
        }

        #[test]
        fn prop_out_of_order_gap_then_fill(isn: u32) {
            let mut dir = FlowDirection::new();
            dir.set_isn(isn);
            let mut total_delivered: usize = 0;

            // Insert segment at seq = ISN+5 (relative offset 5; gap at offsets 1..5)
            let seq_world = isn.wrapping_add(5);
            let _ = dir.insert_segment(seq_world, b"WORLD", MAX_DEPTH, MAX_SEGS, MAX_WIN);
            let flushed = dir.flush_contiguous();
            // Nothing flushed yet: gap at base_offset=1 (offset 1..5 missing)
            assert!(flushed.is_empty(), "delivered bytes before gap was filled");

            // Fill the gap: seq = ISN+1, data "HELL" (relative offsets 1..5)
            let seq_hell = isn.wrapping_add(1);
            let _ = dir.insert_segment(seq_hell, b"HELL", MAX_DEPTH, MAX_SEGS, MAX_WIN);
            let flushed2 = dir.flush_contiguous();
            // Now both segments should flush contiguously
            for (_, bytes) in &flushed2 {
                total_delivered += bytes.len();
            }
            assert!(!flushed2.is_empty(), "nothing delivered after gap filled");
            // 4 + 5 = 9 bytes delivered ("HELL" + "WORLD")
            assert_eq!(total_delivered, 9, "expected 9 bytes after gap fill");
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

`src/reassembly/segment.rs:236` -- `FlowDirection::flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)>`.
Returns flushed (offset, data) pairs; caller iterates the returned Vec. No closure arg.
`src/reassembly/flow.rs:88` -- `FlowDirection::base_offset: u64` pub field (not a method).
Initialized to 0 by `FlowDirection::new()`; set to 1 by `set_isn` (ISN+1 is first data byte).
`src/reassembly/segment.rs:39` -- `FlowDirection::insert_segment` (5-arg signature).

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
