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
  - "v1.3: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:379-381 → mod.rs:416-418 (ConflictingOverlap engine match arm). — 2026-06-01"
  - "v1.4: PG-ARP-F2-007 ss-04-full re-anchor: segment.rs:286-303 → segment.rs:286-303 (fully_covered + has_conflict gate); segment.rs:286 → segment.rs:286; mod.rs:416-418 → mod.rs:416-418 (ConflictingOverlap engine match arm). — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.037: Same-Range Conflicting Overlap Returns ConflictingOverlap; Original Wins

## Description

When a new segment's byte range is fully covered by one or more existing buffered segments
AND at least one byte at the overlapping positions DIFFERS from the buffered bytes,
`insert_segment` returns `InsertResult::ConflictingOverlap`. The original buffered bytes are
preserved (first-wins, INV-3). No new bytes are added to the buffer. The engine match arm
in `mod.rs` then emits an `Anomaly/Likely/High` finding tagged T1036.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. The new segment's byte range `[offset, offset + data.len())` is fully covered by existing
   segments (the union of existing segment ranges covers the entire new range).
3. At least one overlapping byte position has a different byte in the new segment vs. the
   existing buffer (i.e., `has_conflict == true`).

## Postconditions

1. Returns `InsertResult::ConflictingOverlap`.
2. `self.segments` is unchanged (no new entries, no modifications to existing bytes).
3. `self.buffered_bytes` is unchanged.
4. `self.overlap_count` is incremented by 1.
5. The engine (`mod.rs`) emits one `Anomaly/Likely/High` finding tagged T1036 (subject to
   MAX_FINDINGS cap) -- this is outside the scope of `insert_segment` itself.

## Invariants

1. First-wins policy (INV-3): the conflicting new bytes are NEVER accepted. The original
   buffered bytes are authoritative.
2. `ConflictingOverlap` is distinct from `Duplicate` (same range, same bytes) and
   `PartialOverlap` (partial coverage with some new bytes).
3. The T1036 finding is emitted by the engine match arm (lifecycle.rs via mod.rs:416-418),
   not inside `insert_segment` itself. `insert_segment` only classifies the result.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Single-byte range, single byte differs | ConflictingOverlap |
| EC-002 | Multi-byte range, only first byte differs | ConflictingOverlap (any conflict triggers it) |
| EC-003 | New segment fully covered by two existing non-contiguous segments | ConflictingOverlap if any of the overlapping bytes differ |
| EC-004 | Same range, ALL bytes identical | Duplicate (not ConflictingOverlap) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Existing: b"ABC" at 1; New: b"AXC" at 1 (byte 2 differs) | ConflictingOverlap; b"ABC" preserved at 1 | happy-path |
| Existing: b"ABC" at 1; New: b"ABC" at 1 (identical) | Duplicate (not ConflictingOverlap) | edge-case |
| Existing: b"AA" at 1, b"BB" at 3; New: b"AAXX" at 1 (fully covered, b"BB" conflicts) | ConflictingOverlap | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | ConflictingOverlap: original bytes unchanged in segments | unit: test_overlap_conflicting_data_detected |
| VP-002 | ConflictingOverlap: buffered_bytes unchanged | unit |
| VP-002 | ConflictingOverlap: overlap_count incremented | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- ConflictingOverlap detection is the primary forensic signal of TCP evasion in the reassembly engine |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- ConflictingOverlap is the canonical enforcement point) |
| Architecture Module | SS-04 (reassembly/segment.rs:286-303, C-8) |
| Stories | STORY-017 |
| Origin BC | BC-RAS-037 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.018 -- composes with (the engine emits the T1036 finding when this result is returned)
- BC-2.04.035 -- related to (Duplicate is the same-range case with no conflict)
- BC-2.04.036 -- related to (PartialOverlap is the partial-coverage case)

## Architecture Anchors

- `src/reassembly/segment.rs:286-303` -- fully_covered + has_conflict gate for ConflictingOverlap/Duplicate return
- `src/reassembly/mod.rs:416-418` -- engine match arm calling generate_conflicting_overlap_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:286-303` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_overlap_conflicting_data_detected asserts ConflictingOverlap result
- **guard clause**: `if fully_covered { return if has_conflict { ConflictingOverlap } else { Duplicate } }` at segment.rs:286

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.overlap_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
