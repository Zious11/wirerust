---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:517-533 → mod.rs:546-562 (flush_contiguous_data fn definition). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.006: Bidirectional Data Delivered with Correct Direction Tag

## Description

Reassembled data bytes are delivered to the `StreamHandler` via `on_data(key, direction, data,
offset)`. The `direction` parameter is `Direction::ClientToServer` when the data came from the
initiator, and `Direction::ServerToClient` when it came from the responder. Direction
classification is determined by `TcpFlow::direction(src_ip, src_port)` which compares the
packet's source address against the stored `initiator` field. Both directions are independent
segment buffers; flushing one never affects the other.

## Preconditions

1. A flow with at least one direction's ISN set exists in the engine.
2. Contiguous segments starting at `base_offset` are present in the direction's buffer.
3. A call to `flush_contiguous` produces at least one flushed chunk.

## Postconditions

1. `handler.on_data` is called with `direction == ClientToServer` for bytes originating from
   the initiator endpoint.
2. `handler.on_data` is called with `direction == ServerToClient` for bytes originating from
   the responder endpoint.
3. The `offset` parameter in each `on_data` call is the ISN-relative stream offset of the
   first byte of that chunk.
4. `stats.bytes_reassembled` is incremented by the total bytes delivered across all `on_data`
   calls in both directions.

## Invariants

1. Direction tagging is stable for the lifetime of the flow: the `initiator` field is set once
   (idempotent) and never changed.
2. Client-to-server and server-to-client buffers are fully independent `FlowDirection` instances;
   a flush in one direction never drains the other.
3. Bytes are delivered in ISN-relative order within a direction; cross-direction ordering is
   not guaranteed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Only client-to-server data present | on_data called only with ClientToServer; ServerToClient direction produces no callbacks |
| EC-002 | Only server-to-client data present | on_data called only with ServerToClient |
| EC-003 | Both directions data arrive interleaved | Each direction gets correct tag; order within direction is ISN-relative |
| EC-004 | Mid-stream join (no SYN seen) | initiator set from first data packet source; direction tagging same as normal |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SYN from C, SYN+ACK from S, data from C | on_data called with ClientToServer | happy-path |
| SYN from C, SYN+ACK from S, data from S | on_data called with ServerToClient | happy-path |
| Both directions data in same PCAP | Both directions delivered with correct tags independently | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Data from initiator always tagged ClientToServer | unit: process c2s and s2c packets; assert direction in on_data callbacks |
| — | bytes_reassembled equals sum of all on_data lengths | unit: capture all on_data callbacks; assert sum == stats.bytes_reassembled |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- correct direction tagging is a fundamental correctness requirement for protocol analyzers that interpret stream content |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering; direction tagging requires known initiator) |
| Architecture Module | SS-04 (reassembly/mod.rs:546-562, flush_contiguous_data; flow.rs:214-220, direction()) |
| Stories | STORY-015 |
| Origin BC | BC-RAS-006 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.004 -- depends on (initiator set by SYN processing)
- BC-2.04.007 -- composes with (in-order flush delivers the tagged data)
- BC-2.04.053 -- composes with (direction() logic drives tagging)

## Architecture Anchors

- `src/reassembly/mod.rs:546-562` -- flush_contiguous_data; on_data callback with dir
- `src/reassembly/flow.rs:214-220` -- TcpFlow::direction(src_ip, src_port)
- `src/reassembly/handler.rs:49` -- StreamHandler::on_data signature

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:546-562` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: Direction enum (ClientToServer / ServerToClient) enforced by type system
- **guard clause**: direction() comparison against stored initiator

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (handler callback is an in-process call) |
| **Global state access** | mutates stats.bytes_reassembled and flow buffer |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callback) |

## Refactoring Notes

No refactoring needed. The direction-tagging logic is clean and isolated in TcpFlow::direction.
