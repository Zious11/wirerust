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

# BC-2.04.043: Adjacent Segments at Exact Boundary Do Not Count as Overlap

## Description

Two TCP segments are adjacent (not overlapping) if the first segment ends exactly where the
second begins: `first_end == second_start`. The overlap detection logic in `insert_segment`
uses half-open interval comparison `new_start < existing_end && new_end > existing_offset`,
which excludes exact-boundary adjacency. A segment that starts exactly where an existing
segment ends does NOT trigger `has_overlap = true` and is inserted cleanly.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. An existing segment occupies byte range `[existing_offset, existing_end)`.
3. A new segment starts at exactly `existing_end` (i.e., `new_start == existing_end`).
4. The new segment does not overlap any other existing segment.

## Postconditions

1. Returns `InsertResult::Inserted` (not PartialOverlap or Duplicate).
2. `self.overlap_count` is NOT incremented.
3. The new segment is stored at `new_start` in `self.segments`.
4. `self.buffered_bytes` increases by `data.len()`.

## Invariants

1. The half-open interval test `new_start < existing_end && new_end > existing_offset` is
   the canonical overlap check. Adjacency (`new_start == existing_end`) satisfies
   `new_start < existing_end` as False for a segment immediately following (since
   `new_start == existing_end` means `new_start < existing_end` is false).
   Wait -- re-check: if `new_start == existing_end`, then `new_start < existing_end` is
   FALSE; the condition is not entered; no overlap recorded. This is the correct behavior.
2. After adjacent segments are inserted, `flush_contiguous` delivers them in order because
   the BTreeMap keys are `existing_offset` and `existing_end = new_start`, and the flush
   loop advances `base_offset` incrementally.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Segment B starts exactly where segment A ends | Inserted; no overlap |
| EC-002 | Segment B starts one byte before segment A ends | Overlap detected (partial or conflicting) |
| EC-003 | Segment B starts one byte after segment A ends | Gap exists; Inserted; no overlap; flush waits for gap fill |
| EC-004 | Three adjacent segments chained | Each Inserted; flush_contiguous delivers all three |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Existing: b"ABC" at offset 1 (end=4); New: b"DE" at offset 4 | Inserted; overlap_count=0 | happy-path |
| Existing: b"ABC" at offset 1 (end=4); New: b"XY" at offset 3 (overlap by 1) | PartialOverlap or ConflictingOverlap; overlap_count=1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | Adjacent segment (new_start == existing_end) returns Inserted, not Overlap | unit: test_range_boundary_exact_new_end |
| VP-002 | overlap_count not incremented for adjacent segments | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- correct boundary handling prevents false-positive overlap detection in normal sequential TCP flows |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- adjacency is not overlap; this BC ensures the policy is not over-triggered) |
| Architecture Module | SS-04 (reassembly/segment.rs:118, C-8) |
| Stories | STORY-016 |
| Origin BC | BC-RAS-043 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.035 -- related to (Duplicate: same range, exact match)
- BC-2.04.036 -- related to (PartialOverlap: when there IS a true byte-range overlap)
- BC-2.04.034 -- related to (flush_contiguous delivers adjacent segments in order)

## Architecture Anchors

- `src/reassembly/segment.rs:118` -- overlap check: `new_start < existing_end && new_end > existing_offset`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:118` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_range_boundary_exact_new_end verifies no overlap for adjacent segments
- **type constraint**: half-open interval math is the standard interval overlap test

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.segments, self.buffered_bytes (for the insert path) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
