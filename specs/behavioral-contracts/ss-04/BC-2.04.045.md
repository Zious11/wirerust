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

# BC-2.04.045: Segments Map Full: Overlapping Insert Returns SegmentLimitReached

## Description

When `self.segments.len() >= max_segments` AND the incoming segment overlaps existing entries
but all of its byte range is fully covered (no gaps exist), `insert_segment` returns
`InsertResult::SegmentLimitReached`. The segment-limit check inside the gap-insertion loop
(at segment.rs:178) fires when gaps are found but cannot be inserted because the map is at
capacity. When there are no gaps (fully covered), the fully-covered path at segment.rs:201
returns `SegmentLimitReached` when `segments_exhausted` is set.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. `self.segments.len() >= max_segments` at the time the gap-insertion loop runs.
3. The new segment has gaps relative to the existing buffer (i.e., it is NOT fully covered).
4. All gap positions hit the segment-limit guard inside the loop.

## Postconditions

1. Returns `InsertResult::SegmentLimitReached`.
2. `self.segments` may be unchanged (if no gaps were filled before the limit was hit) or
   partially modified (see BC-2.04.046 for the partial insertion variant).
3. `self.overlap_count` is incremented by 1 (overlap was detected before the limit check).
4. `stats.segments_segment_limit` is incremented by the engine.

## Invariants

1. This is distinct from the non-overlapping path (BC-2.04.044): the segment-limit check
   inside the gap loop (segment.rs:178) fires after overlap detection has already incremented
   `overlap_count`.
2. `overlap_count` is incremented even when the segment is ultimately rejected by the limit,
   because the overlap was detected before the limit check.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Overlap detected, gap found, first gap insertion hits limit | SegmentLimitReached; overlap_count=1 |
| EC-002 | Segment is fully covered by existing segments (no gaps), map is full | Depends on fully_covered path: Duplicate or ConflictingOverlap (the non-gap path bypasses the limit guard) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| segments at max, new segment has gap, all gap insertions blocked by limit | SegmentLimitReached; overlap_count incremented | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | SegmentLimitReached on overlapping insert when map full and gap cannot be inserted | unit: test_segment_limit_gap_loop_full_rejection |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- overlapping segment limit handling is part of the BTreeMap overflow protection in TCP reassembly |
| L2 Domain Invariants | INV-6 (bounded-resource design -- max_segments prevents unbounded BTreeMap growth) |
| Architecture Module | SS-04 (reassembly/segment.rs:175-179, C-8) |
| Stories | S-TBD |
| Origin BC | BC-RAS-045 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.044 -- related to (non-overlapping path for same condition)
- BC-2.04.046 -- related to (partial insertion when some gaps filled before limit hit)
- BC-2.04.025 -- composes with (finalize summary finding triggered by segments_segment_limit counter)

## Architecture Anchors

- `src/reassembly/segment.rs:175-179` -- segment-limit check inside gap-insertion loop

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:175-179` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_segment_limit_gap_loop_full_rejection

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.overlap_count, may insert into self.segments |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
