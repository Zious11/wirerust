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

# BC-2.04.030: bytes_reassembled Equals Total Bytes Delivered to Handler

## Description

`ReassemblyStats.bytes_reassembled` accumulates the total number of bytes delivered to the
`StreamHandler` via `on_data` callbacks over the lifetime of the reassembler. At the end of
a capture (after `finalize()`), `bytes_reassembled` equals the sum of all `data.len()` values
passed to every `handler.on_data` invocation across all flows, in both directions.

## Preconditions

1. `TcpReassembler` has processed one or more packets.
2. `finalize()` has been called (so all remaining buffered data has been flushed).
3. `stats().bytes_reassembled` is read after finalize completes.

## Postconditions

1. `stats().bytes_reassembled == sum of data.len() over all on_data(flow_key, dir, data, offset) calls`.
2. The counter is incremented in two places:
   - `flush_contiguous_data()` (mod.rs:530) on each segment flushed during live processing.
   - `close_flow()` (lifecycle.rs:56) on each segment flushed during flow closure.
3. The counter is never decremented.
4. Duplicate segments and out-of-window segments do NOT contribute to this count (they are
   discarded before flush).

## Invariants

1. `bytes_reassembled` is monotonically non-decreasing throughout the reassembler lifetime.
2. Bytes counted as `bytes_reassembled` are exactly the bytes the handler received; there is
   no double-counting between the flush path and the close path.
3. Only the contiguous prefix that was actually flushed is counted; buffered-but-not-yet-
   flushed bytes are NOT included until they are flushed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Out-of-order segment buffered but never flushed (flow closed mid-gap) | Bytes NOT counted in bytes_reassembled (never delivered to handler) |
| EC-002 | Duplicate retransmission | Bytes NOT counted (Duplicate result; no flush of duplicate) |
| EC-003 | Segment truncated at max_depth boundary | Only the truncated (allowed) bytes are flushed and counted |
| EC-004 | finalize() flushes remaining buffered contiguous data | Those bytes ARE counted (close_flow calls flush_contiguous) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 sequential 100-byte segments, in-order, finalize() called | bytes_reassembled == 300 | happy-path |
| 1 segment + 1 duplicate retransmission + finalize() | bytes_reassembled == 100 (duplicate not counted) | edge-case |
| 2 out-of-order segments: seg2 arrives first, seg1 fills gap | bytes_reassembled == 200 after both arrive and flush | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-010 | bytes_reassembled == sum of on_data len args after finalize | unit: test_finalize_bytes_reassembled_consistent |
| VP-010 | bytes_reassembled is monotonically non-decreasing | proptest: any packet sequence yields non-negative delta |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- bytes_reassembled is the primary accounting metric for data delivered by the TCP reassembly engine |
| L2 Domain Invariants | (none -- pure accounting invariant) |
| Architecture Module | SS-04 (reassembly/mod.rs:531, C-6; reassembly/lifecycle.rs:56, C-15) |
| Stories | STORY-012 |
| Origin BC | BC-RAS-030 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.014 -- related to (total_memory tracks buffered bytes; bytes_reassembled tracks delivered bytes)
- BC-2.04.007 -- depends on (flush_contiguous is the delivery mechanism)
- BC-2.04.012 -- depends on (finalize triggers final flush path)

## Architecture Anchors

- `src/reassembly/mod.rs:531` -- bytes_reassembled increment in flush_contiguous_data
- `src/reassembly/lifecycle.rs:56` -- bytes_reassembled increment in close_flow

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:531` and `src/reassembly/lifecycle.rs:56` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_finalize_bytes_reassembled_consistent verifies final count matches handler-received bytes
- **guard clause**: bytes_reassembled incremented only on successful flush (after flush_contiguous returns non-empty)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.stats.bytes_reassembled |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed (stateful mutation) |
