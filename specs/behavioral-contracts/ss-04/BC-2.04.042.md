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

# BC-2.04.042: Segment Beyond max_receive_window Returns OutOfWindow

## Description

When a TCP segment's computed offset exceeds `base_offset + max_receive_window`, the segment
is rejected and `InsertResult::OutOfWindow` is returned. The segment is not stored.
`out_of_window_count` is incremented. The out-of-window check uses `saturating_add` to
prevent overflow when `base_offset + max_receive_window` would exceed `u64::MAX`.

## Preconditions

1. `self.isn` is `Some(isn)`.
2. `offset = seq.wrapping_sub(isn) as u64`.
3. `offset > base_offset.saturating_add(max_receive_window as u64)`.

## Postconditions

1. Returns `InsertResult::OutOfWindow`.
2. `self.segments` is unchanged.
3. `self.buffered_bytes` is unchanged.
4. `self.out_of_window_count = self.out_of_window_count.saturating_add(1)`.
5. No depth check, no segment-limit check (OutOfWindow is returned before those checks).

## Invariants

1. The OutOfWindow check runs BEFORE the segment-limit and depth checks (segment.rs:63-67
   is the first check after ISN validation). A segment that is both out-of-window AND would
   exceed the depth limit is rejected as OutOfWindow, not as DepthExceeded.
2. `out_of_window_count` increments feed the cumulative threshold in `check_anomaly_thresholds`
   (BC-2.04.021).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Segment at exactly base_offset + max_receive_window | Accepted (boundary is exclusive: `> window`, not `>=`) |
| EC-002 | Segment one byte beyond the window boundary | OutOfWindow |
| EC-003 | base_offset near u64::MAX (saturating_add prevents overflow) | OutOfWindow returned without panic |
| EC-004 | Valid segment after several out-of-window segments | Inserted normally; out_of_window_count retains its accumulated value |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| base_offset=1, max_receive_window=1048576, seq yields offset=1048577 | OutOfWindow; out_of_window_count=1 | happy-path |
| base_offset=1, max_receive_window=1048576, seq yields offset=1048576 (= base+window, exclusive) | Inserted (within window) | edge-case |
| base_offset=1, max_receive_window=1048576, seq yields offset=2 | Inserted | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Segment exactly at base+window is accepted (exclusive boundary) | unit: test_out_of_window_segment_rejected |
| — | Segment beyond base+window returns OutOfWindow | unit: test_out_of_window_segment_rejected_by_engine |
| — | out_of_window_count increments per out-of-window segment | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- out-of-window segment rejection enforces the forward receive window resource bound |
| L2 Domain Invariants | INV-6 (bounded-resource design -- max_receive_window prevents accepting arbitrarily far-ahead segments) |
| Architecture Module | SS-04 (reassembly/segment.rs:63-67, C-8) |
| Stories | S-TBD |
| Origin BC | BC-RAS-042 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.021 -- composes with (out_of_window_count feeds the threshold alert)
- BC-2.04.022 -- related to (per-direction alert latch for out-of-window finding)

## Architecture Anchors

- `src/reassembly/segment.rs:63-67` -- out-of-window check: `if offset > self.base_offset.saturating_add(max_receive_window as u64)`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:63-67` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_out_of_window_segment_rejected (segment tests); test_out_of_window_segment_rejected_by_engine (engine tests)
- **guard clause**: explicit out-of-window guard at segment.rs:63

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.out_of_window_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
