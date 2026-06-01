---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
phase: 1c
traces_to: .factory/specs/architecture/ARCH-INDEX.md
source_bc: BC-2.04.039
bcs:
  - BC-2.04.039
module: src/reassembly/segment.rs
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - date: 2026-06-01
    actor: product-owner
    reason: "Fix prose inconsistency: Property Statement items 1-2 corrected to match harness/code (ISN=0xFFFF_FFFE, first segment at isn+1=0xFFFF_FFFF covering offsets 1-4, adjacent segment at seq=0x0000_0003 offset 5)"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-015: TCP Sequence Number Wraparound

## Property Statement

The segment buffer correctly handles TCP sequence number wraparound across the
32-bit boundary (0xFFFF_FFFF -> 0x0000_0000):

1. With ISN = 0xFFFF_FFFE (so `base_offset = 1` after `set_isn`), a 4-byte segment
   starting at sequence number `isn+1 = 0xFFFF_FFFF` (offset 1) crosses the 32-bit
   boundary, covering sequence numbers 0xFFFF_FFFF, 0x0000_0000, 0x0000_0001,
   0x0000_0002 at offsets 1, 2, 3, 4 respectively. The segment is inserted and
   flushed correctly, delivering all 4 bytes in order.

2. A subsequent segment starting at sequence number 0x0000_0003 (offset 5) is
   correctly identified as adjacent (not a gap, not an overlap) and flushed in
   sequence.

3. The buffer arithmetic correctly converts from TCP sequence space (mod 2^32)
   to monotonic byte offset space (u64) using the ISN as the reference point.

## Source Contract

- **Primary BC:** BC-2.04.039 -- TCP sequence wraparound across 32-bit boundary reassembles correctly
- **Postcondition:** Bytes spanning the 32-bit SEQ boundary are delivered in correct order

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Model checking | Kani | Yes -- single wraparound boundary; small data size | The specific wraparound arithmetic at seq=0xFFFF_FFFF |

Kani is preferred here because the wraparound arithmetic is a deterministic
computation over specific numeric values. A bounded model check of the offset
conversion from `seq_space` to `u64` offset is more precise than proptest.

## Proof Harness Skeleton

```rust
// NOTE: seq_offset (src/reassembly/segment.rs:32) is a module-private fn.
// The harness must live in src/reassembly/segment.rs itself (or in a
// #[cfg(kani)] submodule of that file) to access it. The harness below
// is written for placement inside segment.rs.

#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use crate::reassembly::flow::FlowDirection;

    // Reasonable test-harness limits -- keep Kani state space small.
    const MAX_DEPTH: usize = 65536;
    const MAX_SEGMENTS: usize = 64;
    const MAX_WINDOW: usize = 65536;

    #[kani::proof]
    fn verify_sequence_wraparound_arithmetic() {
        // ISN is set so the first data byte lands at offset 1 (base_offset=1
        // after set_isn, per flow.rs:139).  seq_offset(isn+1, isn) = 1.
        // We pick ISN = 0xFFFF_FFFE so seq 0xFFFF_FFFF maps to offset 1,
        // seq 0x0000_0000 (wrap) maps to offset 2, seq 0x0000_0001 to 3,
        // seq 0x0000_0002 to 4.
        let isn: u32 = 0xFFFF_FFFE;
        let mut dir = FlowDirection::new();
        dir.set_isn(isn); // set_isn takes u32 (flow.rs:136)

        // Segment starting at seq = isn+1 = 0xFFFF_FFFF (4 bytes).
        // seq_offset(0xFFFF_FFFF, 0xFFFF_FFFE) = 1  (no wrap yet)
        // seq_offset(0x0000_0000, 0xFFFF_FFFE) = 2  (wraps)
        // seq_offset(0x0000_0001, 0xFFFF_FFFE) = 3
        // seq_offset(0x0000_0002, 0xFFFF_FFFE) = 4
        // 4-byte segment at seq=0xFFFF_FFFF covers offsets 1..5.
        let data = [0xAAu8, 0xBB, 0xCC, 0xDD];
        let result = dir.insert_segment(
            0xFFFF_FFFFu32,  // seq: u32
            &data,           // data: &[u8]
            MAX_DEPTH,       // max_depth: usize
            MAX_SEGMENTS,    // max_segments: usize
            MAX_WINDOW,      // max_receive_window: usize
        );
        assert!(matches!(result, InsertResult::Inserted));

        // flush_contiguous returns Vec<(u64, Vec<u8>)>; no callback.
        let flushed = dir.flush_contiguous();
        let delivered: Vec<u8> = flushed.into_iter().flat_map(|(_, d)| d).collect();
        assert_eq!(delivered.len(), 4);
        assert_eq!(delivered[0], 0xAA);
        assert_eq!(delivered[3], 0xDD);

        // Next segment immediately after the 4-byte span: seq = 0x0000_0003,
        // offset = seq_offset(3, 0xFFFF_FFFE) = 5.  Should be adjacent.
        let data2 = [0xEEu8, 0xFF];
        let result2 = dir.insert_segment(
            0x0000_0003u32,
            &data2,
            MAX_DEPTH,
            MAX_SEGMENTS,
            MAX_WINDOW,
        );
        assert!(!matches!(result2, InsertResult::ConflictingOverlap));
    }

    #[kani::proof]
    fn verify_seq_offset_at_wraparound() {
        // seq_offset is the private fn at segment.rs:32:
        //   fn seq_offset(seq: u32, isn: u32) -> u64 {
        //       seq.wrapping_sub(isn) as u64
        //   }
        // Test with symbolic ISN near the 32-bit boundary.
        let isn: u32 = kani::any();
        kani::assume(isn > 0xFFFF_FF00u32); // near the boundary

        let delta: u32 = kani::any();
        kani::assume(delta <= 300);

        let seq_after_wrap: u32 = isn.wrapping_add(delta);
        let expected_offset: u64 = delta as u64;

        let computed = seq_offset(seq_after_wrap, isn);
        assert_eq!(computed, expected_offset,
            "seq_offset incorrect at wraparound");
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Specific arithmetic at u32 boundary; Kani handles this exactly |
| Proof complexity | Medium | Requires `seq_offset` (segment.rs:32) to be accessible; harness must be co-located in segment.rs |
| Tool support | High | Wraparound arithmetic is deterministic; no external state |
| Estimated proof time | < 2 minutes | Pure arithmetic; no BTreeMap iteration needed |

## Source Location

`src/reassembly/segment.rs:32` -- private function `seq_offset(seq: u32, isn: u32) -> u64`
implements the sequence-number-to-byte-offset conversion using `seq.wrapping_sub(isn) as u64`.
`src/reassembly/segment.rs:39-46` -- `FlowDirection::insert_segment` (5 params: seq, data,
max_depth, max_segments, max_receive_window; no `isn` parameter -- ISN is stored on `self`).
`src/reassembly/flow.rs:136` -- `FlowDirection::set_isn(&mut self, isn: u32)` (takes `u32`).
`src/reassembly/segment.rs:236` -- `FlowDirection::flush_contiguous(&mut self) -> Vec<(u64, Vec<u8>)>`.

Existing test: BC-2.04.039 is [PLANNED]; the Kani proof provides the verification.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
