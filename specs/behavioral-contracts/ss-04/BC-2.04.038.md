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
  - "v1.3: Wave 9 STORY-016 adversarial pass-1 fix: F-3 — stale architecture-anchor line range corrected from segment.rs:199-212 to segment.rs:201-212 (line shift from Wave 8 STORY-019 test-seam additions) — 2026-05-26"
  - "v1.4: Wave 9 STORY-016 adv pass-2 F-7 (sibling-discipline regression of pass-1 F-3): invariant 1 prose anchor 'segment.rs:201' → 'segment.rs:204 (within block at 201-212)' (line 201 is comment, line 204 is the actual return) — 2026-05-26"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.038: Multi-Segment Full Coverage Returns Duplicate or ConflictingOverlap

## Description

When a new segment's byte range is fully covered by the UNION of two or more existing
buffered segments (no single segment fully covers it, but together they do), and there is no
gap remaining, `insert_segment` still returns `Duplicate` (if all covered bytes match) or
`ConflictingOverlap` (if any covered byte differs). The "fully covered by union" path is
handled by the same `fully_covered` check that handles single-segment coverage.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. Two or more existing segments in `self.segments` whose union completely covers the new
   segment's byte range `[new_start, new_end)`.
3. No gap within the new range exists that is not covered by some existing segment.

## Postconditions

1. If all bytes at overlapping positions match: returns `InsertResult::Duplicate`.
2. If any byte at an overlapping position differs: returns `InsertResult::ConflictingOverlap`.
3. `self.segments` is unchanged in either case.
4. `self.buffered_bytes` is unchanged.
5. `self.overlap_count` is incremented by 1.

## Invariants

1. The `fully_covered` computation (segment.rs:145) checks whether any single existing range
   covers `[new_start, new_end)` entirely: `es <= new_start && ee >= new_end`. Multi-segment
   union coverage is handled by the gap-computation path that produces an empty gaps vec,
   which then falls through to the `!had_gap` return arm at segment.rs:204 (within the block at 201-212).
2. First-wins (INV-3) applies identically here: multi-segment union does not change the
   conflict detection or preservation semantics.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two existing segments cover new range with gap between them filled elsewhere | ConflictingOverlap or Duplicate depending on byte content |
| EC-002 | New segment fits exactly in the gap between two existing segments with no overlap | PartialOverlap or Inserted (not this BC) |
| EC-003 | Three segments jointly covering the new range; all bytes match | Duplicate |
| EC-004 | Three segments jointly covering; one byte differs | ConflictingOverlap |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Existing: b"AB" at 1, b"CD" at 3; New: b"ABCD" at 1 (same bytes) | Duplicate | happy-path |
| Existing: b"AB" at 1, b"CD" at 3; New: b"ABXX" at 1 (b"XX" conflicts with b"CD") | ConflictingOverlap | happy-path |
| Existing: b"AA" at 1, b"BB" at 3, b"CC" at 5; New: b"AABBCC" at 1 (all match) | Duplicate | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | Multi-segment full coverage with matching bytes returns Duplicate | unit: test_multi_segment_full_coverage_returns_duplicate |
| VP-002 | Multi-segment full coverage with conflicting bytes returns ConflictingOverlap | unit: test_multi_segment_full_coverage_conflicting_returns_conflict |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- multi-segment coverage handling is part of the first-wins overlap policy completeness |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- applies identically to multi-segment coverage as to single-segment coverage) |
| Architecture Module | SS-04 (reassembly/segment.rs:201-212, C-8) |
| Stories | STORY-016 |
| Origin BC | BC-RAS-038 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.035 -- related to (Duplicate single-segment coverage case)
- BC-2.04.037 -- related to (ConflictingOverlap single-segment coverage case)
- BC-2.04.036 -- related to (PartialOverlap: when there IS a gap in the coverage union)

## Architecture Anchors

- `src/reassembly/segment.rs:201-212` -- the `!had_gap` path returning Duplicate or ConflictingOverlap

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:201-212` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_multi_segment_full_coverage_returns_duplicate and test_multi_segment_full_coverage_conflicting_returns_conflict

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.overlap_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
