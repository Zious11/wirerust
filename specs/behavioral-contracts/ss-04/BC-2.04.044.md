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
  - "v1.3: PG-ARP-F2-007 ss-04-full re-anchor: segment.rs:220-222 → segment.rs:220-222 (segment-limit check on non-overlap path); segment.rs:220 → segment.rs:220. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.044: Segments Map Full: Non-Overlapping Insert Returns SegmentLimitReached

## Description

When `self.segments.len() >= max_segments` at the start of `insert_segment` (the segment
count check occurs after the out-of-window check but before the depth check), a new segment
that does NOT overlap any existing entry is rejected and `InsertResult::SegmentLimitReached`
is returned. No data is inserted and no counters other than `stats.segments_segment_limit`
(tracked by the engine) are modified.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. `self.segments.len() >= max_segments` (the BTreeMap is at capacity).
3. The new segment's offset does not overlap any existing entry (non-overlapping case).
4. The segment is within the receive window.

## Postconditions

1. Returns `InsertResult::SegmentLimitReached`.
2. `self.segments` is unchanged.
3. `self.buffered_bytes` is unchanged.
4. No anomaly counter (`overlap_count`, `out_of_window_count`) is modified.
5. The engine increments `stats.segments_segment_limit` and may trigger the finalize summary
   finding at end-of-capture (BC-2.04.025).

## Invariants

1. The segment-limit check (`self.segments.len() >= max_segments`) at segment.rs:220 runs
   BEFORE the depth check. A segment that is both at the limit AND would exceed depth is
   rejected as SegmentLimitReached, not DepthExceeded.
2. `max_segments` defaults to 10,000 (`max_segments_per_direction` in `ReassemblyConfig`),
   preventing BTreeMap overhead explosion on adversarial flows.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | segments.len() == max_segments - 1 (not yet full) | Inserted normally |
| EC-002 | segments.len() == max_segments (full) | SegmentLimitReached |
| EC-003 | Segment limit reached, then the flow is closed and segments flushed | After close, new flow for same key starts fresh with empty segments |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| segments filled to max_segments, new non-overlapping segment | SegmentLimitReached; segments.len() unchanged | happy-path |
| segments at max_segments - 1, new segment | Inserted | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SegmentLimitReached when segments at max_segments on non-overlap path | unit: test_segment_limit_non_overlap_path; test_max_segments_per_direction |
| — | segments.len() does not increase after SegmentLimitReached | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- segment limit enforcement is the BTreeMap-overflow protection in the TCP reassembly buffer |
| L2 Domain Invariants | INV-6 (bounded-resource design -- max_segments_per_direction caps the BTreeMap size) |
| Architecture Module | SS-04 (reassembly/segment.rs:220-222, C-8) |
| Stories | STORY-018 |
| Origin BC | BC-RAS-044 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.025 -- composes with (finalize emits summary finding when segments_segment_limit > 0)
- BC-2.04.045 -- related to (overlapping insert also returns SegmentLimitReached when map full)
- BC-2.04.046 -- related to (partial insertion when map fills mid-loop)

## Architecture Anchors

- `src/reassembly/segment.rs:220-222` -- segment-limit check on non-overlap path

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:220-222` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_segment_limit_non_overlap_path and test_max_segments_per_direction

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads self.segments.len() |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
