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

```rust
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    #[kani::proof]
    fn verify_first_wins_conflicting_overlap() {
        let mut dir = FlowDirection::new();
        // Insert original bytes at offset 0
        let original: [u8; 4] = kani::any();
        let result1 = dir.insert_segment(0, &original, /* isn */ 0);
        assert!(matches!(result1, InsertResult::Inserted));

        // Insert conflicting bytes at same offset (must differ in at least one position)
        let conflicting: [u8; 4] = kani::any();
        kani::assume(conflicting != original); // ensure a difference exists

        let result2 = dir.insert_segment(0, &conflicting, 0);
        assert!(matches!(result2, InsertResult::ConflictingOverlap));

        // Original bytes must still be in buffer unchanged
        let buffered = dir.read_at(0, 4); // test accessor
        assert_eq!(buffered, original);
    }

    #[kani::proof]
    fn verify_duplicate_does_not_change_buffer() {
        let mut dir = FlowDirection::new();
        let bytes: [u8; 4] = kani::any();
        let _ = dir.insert_segment(0, &bytes, 0);
        // Insert identical bytes again
        let result = dir.insert_segment(0, &bytes, 0);
        assert!(matches!(result, InsertResult::Duplicate));
    }

    #[kani::proof]
    fn verify_adjacent_boundary_not_overlap() {
        let mut dir = FlowDirection::new();
        let bytes_a: [u8; 4] = kani::any();
        let bytes_b: [u8; 4] = kani::any();
        let _ = dir.insert_segment(0, &bytes_a, 0);
        // Segment starting exactly at end of first segment -- should NOT be overlap
        let result = dir.insert_segment(4, &bytes_b, 0);
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
