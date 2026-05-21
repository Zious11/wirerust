---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/segment.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.033: Single Segment Insertion Returns Inserted; Stored Under Offset Key

## Description

When a non-overlapping, in-window, within-depth TCP segment is inserted into an empty
`FlowDirection` buffer, `insert_segment` returns `InsertResult::Inserted`, stores the
segment data under its ISN-relative byte offset key in the `BTreeMap`, and increments
`buffered_bytes` by the segment's length.

## Preconditions

1. `self.isn` is `Some(isn)` (ISN has been set via `set_isn` or `infer_isn`).
2. `data` is non-empty.
3. The segment does not overlap any existing entry in `self.segments`.
4. The computed offset does not exceed `base_offset + max_receive_window`.
5. `self.segments.len() < max_segments`.
6. `self.reassembled_bytes + self.buffered_bytes + data.len() <= max_depth` (no truncation).

## Postconditions

1. Returns `InsertResult::Inserted`.
2. `self.segments` contains a new entry at key `offset` with value `data.to_vec()`.
3. `self.buffered_bytes` increases by `data.len()`.
4. `self.overlap_count`, `self.out_of_window_count` are unchanged.
5. No finding is emitted.

## Invariants

1. The offset stored as the BTreeMap key is `seq.wrapping_sub(isn) as u64`, where `isn` is
   the direction's ISN. For a SYN-set ISN, data seq `ISN+1` yields offset `1` (base_offset).
2. `buffered_bytes == sum of segment data lengths` is maintained by the debug_assert in
   `FlowDirection::memory_used()`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Segment arrives exactly at base_offset (in-order delivery) | Inserted; will be immediately flushed by flush_contiguous |
| EC-002 | Segment arrives far ahead of base_offset but within window | Inserted at high offset; flush_contiguous will wait for gap fill |
| EC-003 | Empty data slice | Returns Inserted immediately (early-return before any checks) |
| EC-004 | segments BTreeMap already has entries but no overlap | Inserted alongside existing entries |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ISN=0, seq=1, data=b"hello", max_depth=1000, max_segments=1000 | Inserted; segments[1]=b"hello"; buffered_bytes=5 | happy-path |
| ISN=999, seq=1000, data=b"AB" | Inserted; segments[1]=b"AB"; buffered_bytes=2 | happy-path |
| ISN=0, seq=1, data=b"" (empty) | Inserted (early return) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Inserted result: segment stored at correct offset key | unit: test_insert_single_segment |
| — | buffered_bytes == segment data length after single insert | unit: test_buffered_bytes_after_insert |
| — | Empty data returns Inserted without any BTreeMap mutation | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- single segment insertion is the base case of TCP reassembly's segment buffering |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- this BC is the no-overlap baseline that all overlap variants are compared against) |
| Architecture Module | SS-04 (reassembly/segment.rs:214-231, C-8) |
| Stories | STORY-015 |
| Origin BC | BC-RAS-033 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.034 -- depends on (flush_contiguous reads what this inserts)
- BC-2.04.035 -- related to (Duplicate is the result when same offset is re-inserted)
- BC-2.04.047 -- related to (buffered_bytes accounting invariant)

## Architecture Anchors

- `src/reassembly/segment.rs:214-231` -- no-overlap insertion path
- `src/reassembly/segment.rs:47-49` -- empty-data early return

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:214-231` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_insert_single_segment asserts Inserted result and correct segment storage
- **guard clause**: `if data.is_empty() { return InsertResult::Inserted; }` at segment.rs:47

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.segments, self.buffered_bytes |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |

## Refactoring Notes

No refactoring needed -- pure in-memory mutation of BTreeMap. Suitable for Kani verification of buffered_bytes invariant.
