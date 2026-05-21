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

# BC-2.04.011: Both FINs Close Flow with CloseReason::Fin

## Description

TCP connection teardown requires a FIN from each side. The engine uses `fin_count` (a `u8`
that saturates at 255) to track how many FIN-flagged packets have been seen per flow. When
`fin_count >= 2`, `on_fin()` sets `state = FlowState::Closed`. After payload processing for
the packet that triggered the second FIN, `process_packet` detects `state == Closed` and calls
`close_flow(key, CloseReason::Fin, handler)`, incrementing `stats.flows_fin`. Each individual
FIN also transitions the state toward `Closing`.

## Preconditions

1. A TCP FIN packet arrives for a flow.
2. The flow's `fin_count` is at least 1 before this packet (i.e., this is the second FIN
   seen, possibly from either direction).

## Postconditions

1. `flow.fin_count` is now >= 2.
2. `flow.state == FlowState::Closed` (via `on_fin`).
3. `stats.flows_fin` increments by 1.
4. Any remaining contiguous data in both directions is flushed to handler.
5. `handler.on_flow_close(key, CloseReason::Fin)` is called exactly once.
6. The flow is removed from `self.flows`.

## Invariants

1. The first FIN transitions state from `Established` (or `SynSent`) to `Closing`; the second
   FIN transitions from `Closing` to `Closed`.
2. FIN close happens AFTER payload processing for the FIN packet (allowing data carried in the
   FIN segment to be reassembled).
3. `fin_count` uses `saturating_add` -- a flow with more than 255 FINs does not overflow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First FIN (only one seen so far) | state -> Closing; fin_count=1; flow NOT closed |
| EC-002 | Second FIN from same direction (retransmit) | fin_count reaches 2; state -> Closed; flow closed |
| EC-003 | FIN with payload | Payload inserted and flushed; then FIN-close detected after flush |
| EC-004 | FIN on New flow (no handshake) | on_fin from New: state -> Closing; second FIN -> Closed |
| EC-005 | RST and FIN in same packet | RST block runs first (PostHandshake::FlowClosed); FIN block not reached |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Client FIN, then Server FIN | flows_fin=1; on_flow_close(Fin); flow removed | happy-path |
| Client FIN only (never closed by server) | state=Closing; flow remains open until timeout |  edge-case |
| Client FIN retransmit counts as second FIN | flow closed as if both sides sent FIN | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Second FIN (any direction) closes flow | unit: process FIN from C, FIN from S; assert flow removed |
| — | CloseReason::Fin delivered to handler | unit: capture on_flow_close reason |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- FIN-based flow close is required for correct TCP lifecycle management |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:166-173, FIN-close detection; mod.rs:281-287, FIN flag block; flow.rs:248-256, on_fin) |
| Stories | S-TBD |
| Origin BC | BC-RAS-011 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.010 -- related to (RST close; alternative close path)
- BC-2.04.050 -- composes with (state machine transitions for FIN)
- BC-2.04.012 -- related to (finalize closes remaining flows including half-closed ones)

## Architecture Anchors

- `src/reassembly/mod.rs:166-173` -- process_packet: if state==Closed after payload, close_flow(Fin)
- `src/reassembly/mod.rs:281-287` -- FIN flag block: set fin_seen, call on_fin
- `src/reassembly/flow.rs:248-256` -- on_fin: fin_count++; state transitions

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:166-173` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `is_some_and(|f| f.state == FlowState::Closed)` after payload processing
- **assertion**: fin_count uses saturating_add

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callback) |

## Refactoring Notes

No refactoring needed. FIN-close detection after payload processing is intentional (allows
FIN+data to be reassembled).
