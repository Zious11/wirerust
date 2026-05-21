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

# BC-2.04.035: Identical Retransmission Returns Duplicate; Does Not Double-Count

## Description

When a TCP segment's byte range is already fully covered by one or more buffered segments with
IDENTICAL content (same bytes, same positions), `insert_segment` returns
`InsertResult::Duplicate`. The segment's data is discarded, no bytes are added to the buffer,
`buffered_bytes` is unchanged, and `stats.segments_duplicates` is incremented. This is the
normal TCP retransmission path.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. The new segment's byte range `[offset, offset + data.len())` is fully covered by one or
   more existing segments in `self.segments`.
3. The bytes at the overlapping range in the existing segments are IDENTICAL to the new
   segment's bytes at those positions.

## Postconditions

1. Returns `InsertResult::Duplicate`.
2. `self.segments` is unchanged (no new entries added, no existing entries modified).
3. `self.buffered_bytes` is unchanged.
4. `self.overlap_count` is incremented by 1 (overlapping segments always increment the
   overlap counter, even duplicates -- see segment.rs:142).
5. `stats.segments_duplicates` is incremented by 1 (in mod.rs match arm).
6. No finding is emitted.

## Invariants

1. The Duplicate result is only returned when the new segment is FULLY covered AND the bytes
   match. Partial coverage returns PartialOverlap; full coverage with different bytes returns
   ConflictingOverlap.
2. `overlap_count` increments on every overlap (Duplicate, PartialOverlap, ConflictingOverlap)
   -- not just on conflicts. The overlap_alert_threshold check in check_anomaly_thresholds
   fires based on the cumulative overlap_count regardless of overlap type.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Exact same seq, exact same bytes, single existing entry | Duplicate |
| EC-002 | New segment byte range covered by two existing entries (multi-segment coverage) with identical bytes | Duplicate (via the fully_covered path) |
| EC-003 | New segment byte range partially covered but the uncovered gap bytes differ from what's buffered | PartialOverlap (new gap bytes inserted with conflict suppressed because the overlap portion is not full) |
| EC-004 | Same range, one byte differs | ConflictingOverlap (not Duplicate) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Insert b"ABC" at offset 1, then insert b"ABC" at offset 1 again | Second call: Duplicate; buffered_bytes=3 (unchanged from first insert) | happy-path |
| Insert b"ABCDE" at 1, then insert b"BCD" at 2 (sub-range, identical bytes) | Second call: Duplicate; overlap_count=1 | happy-path |
| Insert b"ABC" at 1, then insert b"ABX" at 1 | Second call: ConflictingOverlap (not Duplicate) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | Duplicate: buffered_bytes unchanged | unit: test_retransmission_dedup |
| VP-002 | Duplicate: segments unchanged (no new keys) | unit |
| VP-002 | overlap_count incremented even on Duplicate | unit: verify overlap_count after retransmission |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- duplicate retransmission handling is part of the first-wins overlap policy |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- Duplicate is the benign case where the first-wins bytes happen to match the retransmitted bytes) |
| Architecture Module | SS-04 (reassembly/segment.rs:142-154, C-8) |
| Stories | STORY-016 |
| Origin BC | BC-RAS-035 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.036 -- related to (PartialOverlap: gap bytes are different case)
- BC-2.04.037 -- related to (ConflictingOverlap: full coverage, different bytes)
- BC-2.04.019 -- related to (overlap_count increment feeds the overlap threshold alert)

## Architecture Anchors

- `src/reassembly/segment.rs:142-154` -- fully_covered + has_conflict logic producing Duplicate

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:142-154` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_retransmission_dedup asserts Duplicate result and unchanged buffer
- **guard clause**: `fully_covered` check at segment.rs:145 gates the Duplicate/ConflictingOverlap return

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.overlap_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
