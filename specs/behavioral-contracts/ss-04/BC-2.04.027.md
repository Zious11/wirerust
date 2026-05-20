---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/mod.rs
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

# BC-2.04.027: segments_depth_exceeded Tracks Fully-Rejected Segments After Depth Hit

## Description

`ReassemblyStats.segments_depth_exceeded: u64` is incremented in the `InsertResult::DepthExceeded`
match arm within `insert_payload_segment`. This result is returned by `insert_segment` when
the `depth_exceeded` flag is already set on the direction (meaning a prior Truncated event
already hit the depth boundary and all subsequent segments for this direction are fully
rejected). This counter reflects segments that were fully discarded (no bytes stored) due to
the per-direction depth limit having been reached. It is distinct from `Truncated` events
(partial insertion).

## Preconditions

1. `insert_segment` returns `InsertResult::DepthExceeded`.
2. The direction's `depth_exceeded` flag is `true` (set by a prior Truncated event or by the
   zero-remaining-depth check).

## Postconditions

1. `stats.segments_depth_exceeded` increments by 1.
2. The segment payload is entirely discarded; no bytes are stored in the BTreeMap.
3. No Finding is emitted for individual DepthExceeded events (only the Truncated-path emits
   a per-event finding).
4. `total_memory` does not change (no bytes were added).

## Invariants

1. `segments_depth_exceeded` is monotonically non-decreasing.
2. `DepthExceeded` never contributes to `bytes_added` or `total_memory`.
3. A DepthExceeded result does NOT count as a small segment (not inserted into the OOO window;
   excluded from small_segment_run updates in insert_payload_segment).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First segment hits depth limit (Truncated, not DepthExceeded) | segments_inserted++; Truncated finding emitted; segments_depth_exceeded NOT incremented |
| EC-002 | Second segment after depth hit | segments_depth_exceeded++ |
| EC-003 | 1000 segments after depth hit | segments_depth_exceeded += 1000 |
| EC-004 | DepthExceeded in s2c direction; c2s direction still under depth | c2s continues normally; s2c all rejected |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| max_depth=100; 120-byte segment (truncated), then 10-byte segment | segments_depth_exceeded=1 for 2nd segment | happy-path |
| max_depth=100; 1000 segments after depth hit | segments_depth_exceeded=1000 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | segments_depth_exceeded increments for each post-depth segment | unit: exceed depth; send N more segments; assert count |
| VP-TBD | DepthExceeded does not affect total_memory | unit: assert total_memory unchanged after depth-exceeded segments |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- depth-exceeded tracking enables observability of segments dropped by the depth bound |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:386-389, DepthExceeded match arm; segment.rs:80-88, DepthExceeded check) |
| Stories | S-TBD |
| Origin BC | BC-RAS-027 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.041 -- depends on (depth truncation logic in insert_segment)
- BC-2.04.023 -- related to (Truncated finding; the event that sets depth_exceeded flag)
- BC-2.04.028 -- related to (segments_depth_exceeded surfaced in summarize)

## Architecture Anchors

- `src/reassembly/mod.rs:386-389` -- DepthExceeded match arm: segments_depth_exceeded++
- `src/reassembly/segment.rs:80-88` -- DepthExceeded return when depth_exceeded is already set

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:386-389` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: explicit counter increment in InsertResult::DepthExceeded match arm

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates stats.segments_depth_exceeded |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe |
| **Overall classification** | mixed (simple counter mutation) |

## Refactoring Notes

No refactoring needed.
