---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/flow.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 9 STORY-016 adversarial pass-1 fix: F-2 — stale architecture-anchor line range corrected from segment.rs:194-198, 223-226 to segment.rs:196 and 225 (line shift from Wave 8 STORY-019 test-seam additions) — 2026-05-26"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.047: buffered_bytes Mirrors Segment Size Sum After All Operations

## Description

`FlowDirection.buffered_bytes` is a running total of the byte lengths of all segments
currently in the `segments` BTreeMap. After every insert, flush, or overlap operation,
`buffered_bytes` equals `self.segments.values().map(|v| v.len()).sum::<usize>()`. A
`debug_assert` in `FlowDirection::memory_used()` verifies this invariant in debug builds.

## Preconditions

1. A `FlowDirection` has been created and one or more segment operations have occurred.

## Postconditions

1. At all times: `buffered_bytes == sum of self.segments.values().map(|v| v.len())`.
2. After `insert_segment` (Inserted path): `buffered_bytes` increases by `data.len()`.
3. After `insert_segment` (PartialOverlap path): `buffered_bytes` increases by gap bytes
   inserted only (not the full segment length).
4. After `insert_segment` (Duplicate/ConflictingOverlap/OutOfWindow/IsnMissing): `buffered_bytes`
   is unchanged.
5. After `flush_contiguous()` flush of N bytes: `buffered_bytes` decreases by N.

## Invariants

1. `buffered_bytes` is NEVER negative (it is `usize`; underflow would panic in debug builds).
2. The `debug_assert` at `flow.rs:171-176` fires in debug builds if the counter drifts.
3. The `total_memory` at the engine level (`mod.rs`) mirrors `sum of buffered_bytes across
   all active flows`, maintained by adding `bytes_added` on insert and subtracting flush
   bytes on flush (mod.rs:339, 527).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Partial overlap inserts 3 gap bytes out of a 10-byte segment | buffered_bytes increases by 3 |
| EC-002 | flush_contiguous flushes 5 bytes | buffered_bytes decreases by 5 |
| EC-003 | Duplicate insert (no new bytes) | buffered_bytes unchanged |
| EC-004 | flush_contiguous on empty segments | buffered_bytes unchanged (already 0) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Insert 5-byte segment | buffered_bytes=5 | happy-path |
| Insert 5-byte segment, then flush 5 bytes | buffered_bytes=0 | happy-path |
| Insert 5-byte segment, duplicate 5-byte retransmission | buffered_bytes=5 (unchanged by duplicate) | edge-case |
| Insert 5-byte segment, partial overlap: 3 gap bytes | buffered_bytes=8 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-010 | buffered_bytes == sum of segment lengths after each operation | unit: test_buffered_bytes_after_insert, _after_overlap, _after_flush, _partial_flush |
| VP-010 | debug_assert fires on drift | debug-build test |
| VP-010 | buffered_bytes never underflows on flush of 0 bytes | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- buffered_bytes accuracy is the foundation of the memory accounting invariant used by memcap eviction |
| L2 Domain Invariants | INV-6 (bounded-resource design -- buffered_bytes feeds total_memory which is compared against memcap) |
| Architecture Module | SS-04 (reassembly/flow.rs:170-177, C-7; reassembly/segment.rs, C-8) |
| Stories | STORY-016 |
| Origin BC | BC-RAS-047 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.014 -- composes with (total_memory = sum of all flow buffered_bytes)
- BC-2.04.034 -- depends on (flush_contiguous decrements buffered_bytes)
- BC-2.04.033 -- depends on (insert increments buffered_bytes)

## Architecture Anchors

- `src/reassembly/flow.rs:170-177` -- memory_used() with debug_assert for buffered_bytes consistency
- `src/reassembly/segment.rs:196 and 225` -- buffered_bytes increment sites in insert_segment

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:170-177` and `src/reassembly/segment.rs` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_buffered_bytes_after_insert, _after_overlap, _after_flush, _partial_flush
- **assertion**: debug_assert in memory_used() fires on drift in debug builds

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads self.buffered_bytes and self.segments (for debug_assert) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |

## Refactoring Notes

The debug_assert in memory_used() is an excellent verification hook. For formal verification,
a Kani proof can verify the invariant holds for all sequences of insert/flush operations with
small bounds.
