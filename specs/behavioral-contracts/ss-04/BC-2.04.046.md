---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: PG-ARP-F2-007 ss-04-full re-anchor: segment.rs:308-332 → segment.rs:308-332 (gap-insertion loop with mid-loop limit check); segment.rs:311 → segment.rs:311. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.046: Segments Map Fills Mid-Loop: Partial Insertion

## Description

When a new segment has multiple gaps relative to existing buffered segments, and the
BTreeMap fills to `max_segments` capacity PARTWAY through the gap-insertion loop, some
gap bytes are inserted (before the limit is hit) and later gap bytes are dropped. The
function returns `InsertResult::SegmentLimitReached` with a partial insertion: `buffered_bytes`
has increased by the bytes that were inserted before the limit was hit, and `segments.len()`
equals `max_segments`.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. The new segment has two or more gap intervals relative to existing segments.
3. After inserting some gap intervals, `self.segments.len() == max_segments` (the limit is
   hit mid-loop at the segment-limit check inside the loop at segment.rs:311).

## Postconditions

1. Returns `InsertResult::SegmentLimitReached`.
2. `self.segments` contains the partial insertion: gap intervals up to the point where the
   limit was hit are in the map.
3. `self.buffered_bytes` has increased by the bytes from the successfully-inserted gaps only.
4. `segments_exhausted = true` (the loop broke early).
5. `self.overlap_count` is incremented by 1.

## Invariants

1. Partial insertion is irreversible: the engine does not roll back partially-inserted gaps
   on SegmentLimitReached. This is a best-effort insertion.
2. The gap loop iterates gaps in ascending offset order (from `sorted_ranges`), so earlier
   gaps are filled before later gaps; later gaps are the ones dropped.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First gap fills last slot; second gap cannot be inserted | SegmentLimitReached; first gap in BTreeMap; second gap dropped |
| EC-002 | All gaps can be inserted (limit not reached) | PartialOverlap (not this BC) |
| EC-003 | No gaps at all (fully covered) | Duplicate or ConflictingOverlap (fully_covered path; not this BC) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| max_segments=2, existing: b"A" at 1, b"C" at 3; new: b"ABCD" at 1 (gaps at 2 and 4); segments fills after first gap | SegmentLimitReached; b"B" at 2 inserted; b"D" at 4 dropped; buffered_bytes increased by 1 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Partial insertion: some gaps filled, later gaps dropped when limit hit | unit: test_segment_limit_gap_loop_partial_insertion |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- mid-loop partial insertion is an edge case of the segment buffer overflow protection |
| L2 Domain Invariants | INV-6 (bounded-resource design -- max_segments caps BTreeMap size even mid-loop) |
| Architecture Module | SS-04 (reassembly/segment.rs:308-332, C-8) |
| Stories | STORY-018 |
| Origin BC | BC-RAS-046 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.044 -- related to (full rejection on non-overlap path)
- BC-2.04.045 -- related to (full rejection on overlap path when all gaps blocked)
- BC-2.04.047 -- related to (buffered_bytes accounting for partial insertion)

## Architecture Anchors

- `src/reassembly/segment.rs:308-332` -- gap-insertion loop with mid-loop limit check

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:308-332` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_segment_limit_gap_loop_partial_insertion

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.segments, self.buffered_bytes, self.overlap_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
