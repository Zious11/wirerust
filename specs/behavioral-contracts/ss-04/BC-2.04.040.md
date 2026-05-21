---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.040: Small-Segment Counter Increments Per Direction

## Description

The `small_segment_run` field on `FlowDirection` tracks the length of the CURRENT CONSECUTIVE
RUN of undersized segments per direction. After each segment insertion (for results other than
OutOfWindow, SegmentLimitReached, DepthExceeded, and IsnMissing), if the payload length is
less than `small_segment_max_bytes`, the run counter increments by 1 (saturating). If the
payload length is >= `small_segment_max_bytes`, the run counter resets to 0. This is a
consecutive-run model (redesign per LESSON-P2.05 / #92/#93), not a cumulative count.

## Preconditions

1. A non-empty TCP payload is being processed.
2. The `InsertResult` is NOT one of: `OutOfWindow`, `SegmentLimitReached`, `DepthExceeded`,
   `IsnMissing` (those are excluded from the run accounting).
3. `small_segment_max_bytes` is the configured threshold (default: 16 bytes).

## Postconditions

1. If `payload.len() < small_segment_max_bytes`:
   - `flow_dir.small_segment_run = flow_dir.small_segment_run.saturating_add(1)`.
2. If `payload.len() >= small_segment_max_bytes`:
   - `flow_dir.small_segment_run = 0`.
3. The counter is maintained independently per direction (client-to-server vs server-to-client).
4. Empty payloads (pure ACKs) are never processed (the engine checks `!payload.is_empty()`
   before calling `insert_payload_segment`).

## Invariants

1. `small_segment_run` is a consecutive-run counter, not a cumulative counter. Any normal-
   sized segment resets it to zero, modeling the principle that an unbroken run of tiny
   segments is the evasion signal, not the total count.
2. The counter is maintained regardless of the small_segment_ignore_ports exemption; the
   exemption is applied only at the alert-emission stage in `check_anomaly_thresholds`.
3. `saturating_add(1)` prevents u32 overflow if a run exceeds u32::MAX.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | 5 consecutive 1-byte segments | small_segment_run = 5 after 5 segments |
| EC-002 | 5 small segments then 1 large segment | small_segment_run = 0 after the large segment |
| EC-003 | 5 small segments then 1 large then 3 small | small_segment_run = 3 (run reset then 3 new small) |
| EC-004 | OutOfWindow result (excluded from run accounting) | small_segment_run unchanged |
| EC-005 | Empty payload (pure ACK) | Neither incremented nor reset (empty payload guard) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 x 1-byte payloads, small_segment_max_bytes=16 | small_segment_run=3 | happy-path |
| 3 x 1-byte then 1 x 100-byte | small_segment_run=0 after large segment | edge-case |
| InsertResult=OutOfWindow after 2 small segments | small_segment_run unchanged at 2 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | small_segment_run increments on small payload | unit: test_small_segment_tracking |
| — | small_segment_run resets to 0 on normal-sized payload | unit |
| — | OutOfWindow does not affect small_segment_run | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- small-segment detection is part of TCP reassembly anomaly detection |
| L2 Domain Invariants | INV-3 (related -- small-segment detection uses reassembly window results) |
| Architecture Module | SS-04 (reassembly/mod.rs:356-370, C-6; reassembly/flow.rs:101, C-7) |
| Stories | S-TBD |
| Origin BC | BC-RAS-040 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.020 -- composes with (small_segment_run feeds the threshold alert in check_anomaly_thresholds)
- BC-2.04.022 -- related to (per-direction alert latch for small-segment finding)

## Architecture Anchors

- `src/reassembly/mod.rs:356-370` -- small_segment_run update logic in insert_payload_segment
- `src/reassembly/flow.rs:101` -- small_segment_run field on FlowDirection

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:356-370` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_small_segment_tracking (reassembly_segment_tests)
- **guard clause**: explicit InsertResult exclusion list in mod.rs:357-363

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow_dir.small_segment_run |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
