---
document_type: verification-property
level: L4
version: "1.0"
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

# VP-015: TCP Sequence Number Wraparound

## Property Statement

The segment buffer correctly handles TCP sequence number wraparound across the
32-bit boundary (0xFFFF_FFFF -> 0x0000_0000):

1. A segment starting at sequence number 0xFFFF_FFFE with 4 bytes of data
   (covering sequence numbers 0xFFFF_FFFE, 0xFFFF_FFFF, 0x0000_0000, 0x0000_0001)
   is inserted and flushed correctly, delivering all 4 bytes in order.

2. A subsequent segment starting at sequence number 0x0000_0002 is correctly
   identified as adjacent (not a gap) and flushed in sequence.

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
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_sequence_wraparound_arithmetic() {
        // ISN is set so that the wraparound falls within our segment
        let isn: u32 = 0xFFFF_FFFE;  // ISN just before the boundary
        let mut dir = FlowDirection::new();
        dir.set_isn(isn as u64);

        // Segment at seq=isn covers bytes at byte-offsets 0,1,2,3
        // seq_to_offset(0xFFFF_FFFE) = 0
        // seq_to_offset(0xFFFF_FFFF) = 1
        // seq_to_offset(0x0000_0000) = 2  (wrapped)
        // seq_to_offset(0x0000_0001) = 3  (wrapped)
        let data = [0xAAu8, 0xBB, 0xCC, 0xDD];
        let result = dir.insert_segment(0, &data, isn as u64);
        assert!(matches!(result, InsertResult::Inserted));

        // Flush and verify all 4 bytes delivered
        let mut delivered = Vec::new();
        dir.flush_contiguous(&mut |b| delivered.extend_from_slice(b));
        assert_eq!(delivered.len(), 4);
        assert_eq!(delivered[0], 0xAA);
        assert_eq!(delivered[3], 0xDD);

        // Next segment immediately after the wraparound boundary
        let data2 = [0xEEu8, 0xFF];
        let result2 = dir.insert_segment(4, &data2, isn as u64);
        assert!(!matches!(result2, InsertResult::ConflictingOverlap));
    }

    #[kani::proof]
    fn verify_seq_to_offset_at_wraparound() {
        // Verify that the seq_to_offset conversion handles the mod-2^32 case
        // by testing symbolic inputs near the boundary
        let isn: u32 = kani::any();
        kani::assume(isn > 0xFFFF_FF00u32); // Near the boundary

        let seq_after_wrap: u32 = isn.wrapping_add(300); // Crosses 0
        let expected_offset: u64 = 300;

        let computed = seq_to_offset(seq_after_wrap, isn);
        assert_eq!(computed, expected_offset,
            "seq_to_offset incorrect at wraparound");
    }
}
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Specific arithmetic at u32 boundary; Kani handles this exactly |
| Proof complexity | Medium | Requires `seq_to_offset` to be accessible as a pure function |
| Tool support | High | Wraparound arithmetic is deterministic; no external state |
| Estimated proof time | < 2 minutes | Pure arithmetic; no BTreeMap iteration needed |

## Source Location

`src/reassembly/segment.rs` -- sequence-number-to-byte-offset conversion.
The ISN is the reference point; `offset = (seq - isn) mod 2^32` using `u32::wrapping_sub`.

Existing test: BC-2.04.039 is [PLANNED]; the Kani proof provides the verification.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| Created | 2026-05-20 | architect |
| Proof harness committed | null | formal-verifier |
| Proof first passed | null | formal-verifier |
| Locked (VERIFIED) | null | formal-verifier |
