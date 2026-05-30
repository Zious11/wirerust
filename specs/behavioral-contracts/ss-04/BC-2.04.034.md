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

# BC-2.04.034: flush_contiguous Consumes Segments from base_offset in Order

## Description

`FlowDirection::flush_contiguous()` removes and returns all segments forming a contiguous
prefix starting at `self.base_offset`. Segments at higher offsets that would leave a gap are
NOT flushed. Each flushed segment decrements `buffered_bytes` and advances `base_offset` by
the segment's length. The returned `Vec<(u64, Vec<u8>)>` contains `(offset, data)` pairs in
ascending offset order.

## Preconditions

1. The direction's `segments` BTreeMap may contain zero or more entries.
2. `self.base_offset` points to the expected next byte offset.

## Postconditions

1. All contiguous segments starting at `base_offset` are removed from `self.segments`.
2. For each flushed segment `(offset, data)`:
   - `self.buffered_bytes -= data.len()`
   - `self.base_offset += data.len() as u64`
   - `self.reassembled_bytes += data.len()`
3. Returns `Vec<(u64, Vec<u8>)>` in ascending offset order.
4. If no segment exists at `base_offset`, returns empty Vec (no-op).
5. Segments at offsets beyond the first gap are NOT flushed.

## Invariants

1. After flush_contiguous, `segments.get(&base_offset)` returns `None` (the contiguous
   prefix has been removed).
2. `buffered_bytes` equals the sum of remaining (not-yet-flushed) segment lengths.
3. `base_offset` is monotonically increasing; it never decreases.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Empty segments BTreeMap | Returns empty Vec; base_offset unchanged |
| EC-002 | Single segment exactly at base_offset | Returns [(base_offset, data)]; base_offset advances |
| EC-003 | Two adjacent segments at base_offset and base_offset+len | Both flushed in one call; returns 2-element Vec |
| EC-004 | Gap: segment at base_offset, then gap, then segment further ahead | Only first segment flushed; second left in buffer |
| EC-005 | Three-segment chain with no gaps | All three flushed in order |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| segments={1: "ABC"}, base_offset=1 | [(1, "ABC")]; base_offset=4; buffered_bytes=0 | happy-path |
| segments={1: "AB", 3: "CD"}, base_offset=1 | [(1, "AB"), (3, "CD")]; base_offset=5 | happy-path |
| segments={5: "XY"}, base_offset=1 (gap) | []; base_offset=1 unchanged | edge-case |
| segments={}, base_offset=1 | []; base_offset=1 unchanged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-011 | flush_contiguous returns segments in ascending offset order | unit: test_flush_contiguous_ordered |
| VP-011 | buffered_bytes decrements by exactly the flushed byte count | unit: test_flush_contiguous_single |
| VP-011 | base_offset advances by exactly the flushed byte count | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- flush_contiguous is the mechanism by which reassembled data is delivered; it is the output contract of TCP stream reassembly |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- data flushed is the already-buffered first-wins bytes) |
| Architecture Module | SS-04 (reassembly/segment.rs:234-248, C-8) |
| Stories | STORY-015 |
| Origin BC | BC-RAS-034 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.007 -- composes with (flush_contiguous is the implementation of in-order flush)
- BC-2.04.030 -- related to (bytes flushed here are counted in bytes_reassembled)
- BC-2.04.047 -- related to (buffered_bytes accounting updated by this function)

## Architecture Anchors

- `src/reassembly/segment.rs:234-248` -- flush_contiguous implementation

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:234-248` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_flush_contiguous_single and test_flush_contiguous_ordered verify output structure
- **type constraint**: BTreeMap iteration is in ascending key order by definition

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.segments, self.buffered_bytes, self.base_offset, self.reassembled_bytes |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |

## Refactoring Notes

No refactoring needed -- pure in-memory BTreeMap mutation. Suitable for Kani verification of the base_offset advancement invariant and the buffered_bytes decrement invariant.
