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
  - "v1.3: Wave 9 STORY-016 adversarial pass-1 fix: F-4 — stale architecture-anchor line range corrected from segment.rs:156-212 to segment.rs:156-199 (line shift from Wave 8 STORY-019 test-seam additions) — 2026-05-26"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.036: First-Wins Overlap: Gap Bytes Added, Existing Bytes Preserved

## Description

When a new segment partially overlaps existing buffered segments (the new segment covers
some already-buffered bytes AND some gap bytes), `insert_segment` applies the first-wins
policy: only the bytes in the gap positions are inserted. The already-buffered bytes at
overlapping positions are preserved unchanged. The function returns `InsertResult::PartialOverlap`.
No finding is emitted for partial overlaps.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. The new segment's byte range partially overlaps existing segments -- that is, some bytes
   are new (gap) and some bytes are at already-buffered positions.
3. The segment is within the receive window and within depth limits.
4. `self.segments.len() < max_segments` (not at the BTreeMap limit).

## Postconditions

1. Returns `InsertResult::PartialOverlap`.
2. Only the gap bytes are inserted into `self.segments` (as one or more new BTreeMap entries
   covering the gap intervals).
3. Existing entries at the overlapping positions are NOT modified.
4. `self.buffered_bytes` increases by the total number of gap bytes inserted.
5. `self.overlap_count` increments by 1.
6. `stats.segments_overlaps` increments (and `stats.segments_inserted` also increments,
   because gap bytes were added; see mod.rs PartialOverlap match arm).
7. No finding is emitted.

## Invariants

1. First-wins policy (INV-3): existing bytes ALWAYS win. A partial overlap never modifies
   any byte that is already buffered.
2. Gap bytes from the new segment are treated as fresh data, not as conflicting; conflict
   detection is only applied to positions where both old and new bytes exist.
3. The `overlap_count` increment applies to ALL overlapping segments (partial or full),
   feeding the aggregate overlap threshold check.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New segment extends an existing one at the end (append overlap) | Gap bytes at the tail appended; head preserved |
| EC-002 | New segment extends an existing one at the start (prepend overlap) | Gap bytes at the head prepended; tail (old bytes) preserved |
| EC-003 | New segment spans two existing segments with a gap between them | Gap bytes between the two old segments filled |
| EC-004 | Gap exists but max_segments limit hit mid-gap insertion | SegmentLimitReached returned instead (BC-2.04.046) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Existing: b"AAA" at 1; New: b"AAABBB" at 1 (gap = "BBB" at 4) | PartialOverlap; "AAA" preserved; "BBB" added at 4; buffered_bytes=6 | happy-path |
| Existing: b"BBB" at 4; New: b"AAABBB" at 1 (gap = "AAA" at 1) | PartialOverlap; "BBB" preserved; "AAA" added at 1 | happy-path |
| Existing: b"AA" at 1, b"CC" at 5; New: b"AABBCC" at 1 (gap = "BB" at 3) | PartialOverlap; "BB" inserted at 3; others preserved | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | PartialOverlap: existing bytes unchanged at overlap positions | unit: test_overlap_first_wins |
| VP-002 | PartialOverlap: buffered_bytes increases by exactly gap bytes count | unit: test_buffered_bytes_after_overlap |
| VP-002 | overlap_count increments by 1 per partial overlap event | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- partial overlap gap-filling is the core of the first-wins TCP reassembly policy |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- this BC is the primary implementation of that invariant for the partial overlap case) |
| Architecture Module | SS-04 (reassembly/segment.rs:156-199, C-8) |
| Stories | STORY-016 |
| Origin BC | BC-RAS-036 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.035 -- related to (Duplicate is the case where gap is empty and bytes match)
- BC-2.04.037 -- related to (ConflictingOverlap is the case where full coverage with mismatching bytes)
- BC-2.04.043 -- related to (adjacent boundary does not count as overlap)

## Architecture Anchors

- `src/reassembly/segment.rs:156-199` -- gap computation and gap insertion loop

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:156-199` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_overlap_first_wins asserts "AAABBBCC" (original "BBB" wins over conflicting bytes)
- **guard clause**: gap computation loop preserves existing segment ranges explicitly

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.segments, self.buffered_bytes, self.overlap_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
