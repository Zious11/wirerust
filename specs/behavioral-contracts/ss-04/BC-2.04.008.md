---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.008: Out-of-Order Segments Buffer Until Gap Filled Then Flush

## Description

When a TCP segment arrives with a sequence number ahead of `base_offset` (leaving a gap),
the segment is stored in the BTreeMap under its ISN-relative offset key. It is NOT delivered
to the handler immediately. When a later segment fills the gap, `flush_contiguous` can
advance `base_offset` past the previously-buffered segment, delivering it in order. This is
the core out-of-order reassembly behavior.

## Preconditions

1. A TCP segment arrives with an ISN-relative offset > `base_offset` (gap exists).
2. The segment is within `max_receive_window` bytes of `base_offset`.
3. The flow has an ISN set for the relevant direction.
4. `segments.len() < max_segments_per_direction`.

## Postconditions

1. The segment is stored in `segments[offset]`.
2. `buffered_bytes` increases by the segment length.
3. `total_memory` increases by the segment length.
4. `on_data` is NOT called for this segment yet.
5. When a later segment arrives at `base_offset` (filling the gap), `flush_contiguous`
   delivers both the fill segment and all previously-buffered contiguous segments, advancing
   `base_offset` across all of them.

## Invariants

1. The BTreeMap key is the ISN-relative offset, ensuring ordered iteration.
2. Segments stored before a gap are never lost unless the flow is closed/evicted.
3. Multiple gaps may exist simultaneously; each is resolved independently as fill segments
   arrive.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Fill segment arrives immediately after out-of-order | flush_contiguous delivers both in sequence |
| EC-002 | Gap never filled (flow timeout/evict) | Buffered segments flushed in close_flow via flush_contiguous |
| EC-003 | Multiple gaps at once | Each gap resolved independently; contiguous prefix flushed after each fill |
| EC-004 | Fill segment itself is out-of-order relative to a deeper gap | Partial flush advances to next gap only |
| EC-005 | Segment arrives that fills gap but overlaps with existing segment | Handled by overlap logic (PartialOverlap or ConflictingOverlap); flush proceeds for non-conflicting portions |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Segments arrive in order 1,2,3 | All flushed immediately as they arrive | happy-path |
| Segments arrive as 2,1 | 2 buffered on arrival; 1 arrives -> both flushed | out-of-order |
| Segments arrive as 3,2,1 | 3,2 buffered; 1 arrives -> all three flushed in order | out-of-order |
| Gap never filled (close_flow) | close_flow::flush_contiguous delivers segments up to gap | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Out-of-order segments delivered in ISN-relative order after gap fill | unit: insert 2,1; assert on_data order |
| VP-TBD | buffered_bytes accounting correct throughout | proptest: random insert order; assert buffered_bytes == sum(segment lengths) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- out-of-order buffering is the primary function that justifies the BTreeMap segment store |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/segment.rs:214-231, no-overlap insert; segment.rs:236-248, flush_contiguous) |
| Stories | S-TBD |
| Origin BC | BC-RAS-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.007 -- composes with (flush_contiguous delivers buffered segments)
- BC-2.04.033 -- composes with (single segment insertion formal spec)
- BC-2.04.042 -- related to (out-of-window rejects segments too far ahead)
- BC-2.04.044 -- related to (segment limit reached before buffer fills)

## Architecture Anchors

- `src/reassembly/segment.rs:214-231` -- no-overlap insert path: stores in BTreeMap
- `src/reassembly/segment.rs:236-248` -- flush_contiguous: delivers contiguous prefix

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:214-231` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: BTreeMap provides O(log n) ordered storage by offset key
- **assertion**: flush_contiguous: while segments.remove(&base_offset) exists

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates FlowDirection.segments and buffered_bytes |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful buffer mutation) |

## Refactoring Notes

The insert + flush logic has a clear separation. The BTreeMap could be replaced with a
VecDeque for certain access patterns, but the current structure enables gap detection
naturally.
