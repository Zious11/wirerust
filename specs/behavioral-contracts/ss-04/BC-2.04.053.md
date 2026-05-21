---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.053: TcpFlow::direction Returns ClientToServer When src Matches Initiator

## Description

`TcpFlow::direction(src_ip, src_port)` returns `Direction::ClientToServer` when
`self.initiator == Some((src_ip, src_port))`, and `Direction::ServerToClient` otherwise
(including when `self.initiator` is `None`). The initiator is set by `set_initiator`, which
records the first caller and ignores subsequent calls (idempotent). For SYN flows, the
initiator is the source of the SYN; for mid-stream joins, the initiator is the source of the
first data packet; for SYN+ACK flows, the initiator is the DESTINATION of the SYN+ACK.

## Preconditions

1. `self.initiator` is either `None` or `Some((ip, port))` set by a prior `set_initiator` call.
2. `direction(src_ip, src_port)` is called with the source IP and port of the current packet.

## Postconditions

1. If `self.initiator == Some((src_ip, src_port))`: returns `Direction::ClientToServer`.
2. Otherwise (including `initiator == None`): returns `Direction::ServerToClient`.

## Invariants

1. `set_initiator` is idempotent: once set, subsequent calls with different values are
   ignored (`if self.initiator.is_none()`).
2. When `initiator` is `None` (e.g., no SYN ever seen AND no data before ISN was inferred),
   all packets return `ServerToClient` -- a conservative default. This edge case should not
   occur in practice because `set_initiator` is called before `direction` in all code paths.
3. The SYN+ACK handling in `apply_handshake_flags` sets initiator to `packet.dst_ip, tcp.dst_port`
   (the destination of the SYN+ACK is the initiator, i.e., the original SYN sender).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | src == initiator | ClientToServer |
| EC-002 | src != initiator | ServerToClient |
| EC-003 | initiator == None | ServerToClient (fallback) |
| EC-004 | SYN+ACK: initiator set to dst (not src) of SYN+ACK | direction() called with SYN+ACK src returns ServerToClient |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| initiator=1.2.3.4:1000, direction(1.2.3.4, 1000) | ClientToServer | happy-path |
| initiator=1.2.3.4:1000, direction(5.6.7.8, 80) | ServerToClient | happy-path |
| initiator=None, direction(any) | ServerToClient | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-001 | direction == ClientToServer iff src matches initiator | unit: test_flow_direction_determines_client_server |
| VP-001 | initiator=None returns ServerToClient | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- direction determination is required for correct bidirectional data delivery in TCP stream reassembly |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering -- direction is orthogonal to canonical key ordering; a client can be either lower or upper endpoint) |
| Architecture Module | SS-04 (reassembly/flow.rs:214-220, C-7) |
| Stories | STORY-013 |
| Origin BC | BC-RAS-053 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.006 -- composes with (bidirectional data delivery uses direction to tag on_data calls)
- BC-2.04.004 -- depends on (set_initiator is called during SYN processing)
- BC-2.04.052 -- depends on (set_initiator called during on_data_without_syn)

## Architecture Anchors

- `src/reassembly/flow.rs:214-220` -- direction() and set_initiator() implementations

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:214-220` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_flow_direction_determines_client_server (reassembly_flow_tests)
- **type constraint**: Direction enum with exactly two variants enforces binary classification

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads self.initiator |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (shared read) |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed -- pure function of self.initiator. Suitable for Kani proof: `direction(ip, port) == ClientToServer iff initiator == Some((ip, port))`.
