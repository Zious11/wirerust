---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: F-001/F-003/F-005 remediation — Description, PC2, PC3, EC-002, INV-2 corrected for early-guard vs mid-loop path distinction; new EC-003 added; DF-SIBLING-SWEEP-001 — 2026-05-26"
  - "v1.4: W10-D8 fix — PC2 tail 'or no gaps fit at all' removed. The early-guard at segment.rs:70-72 prevents entry when len>=max_segments; therefore at the mid-loop guard there must be >=1 gap already computed. The 'no gaps fit at all' case is structurally unreachable via this path. — 2026-05-28"
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
`InsertResult::SegmentLimitReached`. Two code paths reach this result: (a) the **unconditional
early guard** at `segment.rs:70-72` fires whenever `self.segments.len() >= max_segments` at
function entry — this gates ALL inserts at capacity, including pure-overlap (fully covered)
segments that would otherwise return Duplicate/ConflictingOverlap; (b) the **mid-loop guard**
at `segment.rs:178-180` fires when the map fills during gap insertion (see BC-2.04.046 for
the partial-insertion variant).

## Preconditions

1. `self.isn` is `Some(isn)`.
2. `self.segments.len() < max_segments` at function entry (so the early guard at segment.rs:70-72 does not fire), AND `self.segments.len() == max_segments` at the time the mid-loop guard at segment.rs:178 fires (after at least one gap was inserted in the same call).
3. The new segment has gaps relative to the existing buffer (i.e., it is NOT fully covered).
4. All gap positions hit the segment-limit guard inside the loop.

## Postconditions

1. Returns `InsertResult::SegmentLimitReached`.
2. `self.segments` may be unchanged (if no gaps were filled before the limit was hit) or
   partially modified (see BC-2.04.046 for the partial insertion variant).
3. `self.overlap_count` is incremented by 1 **only in the mid-loop path** (segment.rs:143 fires before the gap-loop limit guard at segment.rs:178). In the **early-exit path** (segment.rs:70-72 fires before overlap detection), `overlap_count` is NOT incremented; the rejection happens too early to detect the overlap.
4. `stats.segments_segment_limit` is incremented by the engine.

## Invariants

1. This is distinct from the non-overlapping path (BC-2.04.044): the segment-limit check
   inside the gap loop (segment.rs:178) fires after overlap detection has already incremented
   `overlap_count`.
2. `overlap_count` is incremented when the limit is hit **mid-loop** because overlap was already
   detected at that point. When the limit fires at the **entry guard** (segment.rs:70-72),
   overlap detection has not yet run, so `overlap_count` is NOT incremented. Callers cannot
   use `overlap_count` to infer overlap from a `SegmentLimitReached` result without knowing
   the entry-vs-mid-loop path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Overlap detected, gap found, first gap insertion hits limit | SegmentLimitReached; overlap_count=1 |
| EC-002 | Segment is fully covered by existing segments (no gaps), map is full at entry | `SegmentLimitReached` — the early guard at `segment.rs:70-72` fires before overlap detection, so the fully-covered path is not reached. (The Duplicate/ConflictingOverlap results only occur when the map is below capacity at entry.) |
| EC-003 | Segment overlaps existing segments but map full at entry | `SegmentLimitReached` (early-exit path); `overlap_count` NOT incremented because overlap detection didn't run |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| segments at max, new segment has gap, all gap insertions blocked by limit | SegmentLimitReached; overlap_count incremented | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SegmentLimitReached on overlapping insert when map full and gap cannot be inserted | unit: test_segment_limit_gap_loop_full_rejection |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- overlapping segment limit handling is part of the BTreeMap overflow protection in TCP reassembly |
| L2 Domain Invariants | INV-6 (bounded-resource design -- max_segments prevents unbounded BTreeMap growth) |
| Architecture Module | SS-04 (reassembly/segment.rs:175-179, C-8) |
| Stories | STORY-018 |
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
