---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: Wave 9 wave-level adv pass-1 F-W9P1-003: lifecycle.rs:51 → lifecycle.rs:60 in 2 occurrences — 2026-05-26"
  - "v1.4: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:337-340 → mod.rs:367-368 (total_memory += bytes_added); mod.rs:525-527 → mod.rs:554-556 (total_memory -= flushed bytes). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.014: total_memory Tracks Buffered Bytes; Decrements on Flush and Close

## Description

`TcpReassembler.total_memory: usize` is a running sum of all bytes currently buffered across
all flow directions. It increments when new segment bytes are added to a direction's BTreeMap
(via `insert_payload_segment`) and decrements when bytes are flushed (via `flush_contiguous`)
or when a flow is closed (via `close_flow`). The `total_memory()` accessor exposes this value.
This counter drives the memcap eviction check after each packet.

## Preconditions

1. Segment bytes are being inserted or flushed from flow direction buffers.

## Postconditions

1. After inserting N bytes into a direction's buffer: `total_memory` increases by N.
2. After `flush_contiguous` delivers M bytes: `total_memory` decreases by M.
3. After `close_flow` removes a flow: `total_memory` decreases by the flow's `memory_used()`
   at removal time (all remaining buffered bytes in both directions).
4. `total_memory == sum(flow.memory_used() for all flows in self.flows)` is maintained as
   a debug invariant.

## Invariants

1. `total_memory` is never negative; it is `usize`, so underflow is a bug caught by
   debug_assert (not a panic in release builds).
2. The accounting uses `flow_dir.buffered_bytes` as the authoritative per-direction counter;
   `total_memory` is the cross-flow aggregate.
3. The memcap check in `process_packet` fires AFTER each packet: `if self.total_memory >
   self.config.memcap { self.evict_flows(handler); }`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Insert segment; flush immediately | total_memory goes up then back down to 0 |
| EC-002 | Out-of-order segment buffered; gap not filled for N packets | total_memory stays elevated until gap filled and flushed |
| EC-003 | Flow evicted under memcap pressure | total_memory decremented by evicted flow's memory_used() |
| EC-004 | Zero-length segment insert | total_memory unchanged (InsertResult::Inserted for empty data returns immediately) |
| EC-005 | finalize closes all flows | total_memory reaches 0 after finalize |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Insert 100 bytes | total_memory == 100 | happy-path |
| Insert 100 bytes then flush | total_memory == 0 | happy-path |
| Insert 100 bytes then close_flow | total_memory == 0 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | total_memory == sum(flow.memory_used()) after every operation | proptest: random sequence of inserts/flushes/closes |
| — | total_memory never exceeds memcap + max segment size (bounded by eviction) | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- total_memory tracking is the measurement mechanism for the memcap eviction policy |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:367-368, insert; mod.rs:554-556, flush; lifecycle.rs:60, close_flow) |
| Stories | STORY-020 |
| Origin BC | BC-RAS-014 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.016 -- depends on (memcap check uses total_memory)
- BC-2.04.047 -- related to (per-direction buffered_bytes is the component of total_memory)
- BC-2.04.015 -- related to (eviction uses total_memory)

## Architecture Anchors

- `src/reassembly/mod.rs:367-368` -- total_memory += bytes_added after insert
- `src/reassembly/mod.rs:554-556` -- total_memory -= flushed bytes
- `src/reassembly/lifecycle.rs:60` -- total_memory -= flow_mem on close

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:367-368` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: explicit += and -= on total_memory at all insert/flush/close sites

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.total_memory |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (simple counter mutation) |

## Refactoring Notes

The three sites (insert, flush, close) are consistent. A debug_assert reconciling total_memory
against sum(flow.memory_used()) would catch drift; currently this invariant is unverified at
runtime outside debug builds.

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-05-20 | product-owner | Initial brownfield extraction |
| 1.2 | 2026-05-21 | product-owner | VP back-reference back-fill (P8-DEFER) |
| 1.3 | 2026-05-26 | product-owner | Wave 9 wave-level adv pass-1 F-W9P1-003: lifecycle.rs:51 → lifecycle.rs:60 in 2 occurrences (line 51 is the let-binding capture; line 60 is the actual decrement, shifted by STORY-019 let-else block at lifecycle.rs:42-50). Full BC anchor freshness verified against current source. |
| 1.4 | 2026-06-01 | product-owner | DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:337-340 → mod.rs:367-368 (total_memory += bytes_added); mod.rs:525-527 → mod.rs:554-556 (total_memory -= flushed bytes). |
