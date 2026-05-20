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

# BC-2.04.010: RST Closes Flow Immediately with CloseReason::Rst

## Description

When a TCP RST packet arrives for a flow, the engine calls `flow.on_rst()` to mark the flow
closed, increments `stats.flows_rst`, calls `close_flow(key, CloseReason::Rst, handler)` to
flush any remaining buffered data and notify the handler, and then returns
`PostHandshake::FlowClosed` to `process_packet`, which skips all further payload handling for
this packet. The flow is removed from the `flows` HashMap by `close_flow`.

## Preconditions

1. A TCP packet with RST=true arrives.
2. A flow for the packet's FlowKey exists in the engine.

## Postconditions

1. `stats.flows_rst` increments by 1.
2. Any remaining contiguous data in both directions is flushed to the handler via `on_data`
   calls (in `close_flow`).
3. `handler.on_flow_close(key, CloseReason::Rst)` is called exactly once.
4. The flow is removed from `self.flows`.
5. `total_memory` decrements by the bytes freed from the flow's buffers.
6. Payload processing for the RST packet itself is skipped (even if the RST packet carries
   data, that data is not processed).

## Invariants

1. RST closes the flow regardless of current state (New, SynSent, Established, Closing).
   `on_rst()` unconditionally sets `state = FlowState::Closed`.
2. `close_flow` is idempotent in effect: if the key is not found (already closed), a one-shot
   warning is emitted and the call returns without error (per BC-2.04.029).
3. `PostHandshake::FlowClosed` return value prevents any payload from the RST packet being
   processed after the flow is removed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RST on New flow (no data seen) | Flow closed; no data flushed; on_flow_close(Rst) called |
| EC-002 | RST on Established flow with buffered data | Buffered data flushed; then on_flow_close(Rst) |
| EC-003 | RST with payload | Payload ignored; close happens; payload not inserted |
| EC-004 | RST on already-missing key | close_flow emits one-shot warning; no panic |
| EC-005 | Both SYN+ACK and RST in same packet (malformed) | Both blocks execute: on_syn_ack then on_rst; RST wins (state=Closed) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Established flow receives RST | flows_rst=1; on_flow_close(Rst); flow removed | happy-path |
| New flow receives RST (no handshake) | flows_rst=1; on_flow_close(Rst); flow removed | edge-case |
| Flow with 100 bytes buffered receives RST | 100 bytes flushed; then on_flow_close(Rst) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | RST always results in flow removal | unit: process RST; assert flows.is_empty() |
| VP-TBD | CloseReason::Rst delivered to handler | unit: capture on_flow_close reason |
| VP-TBD | Buffered data flushed before on_flow_close | unit: buffer data, send RST, assert on_data before on_flow_close |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- RST handling is a required part of the TCP flow lifecycle |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:272-278, RST block; lifecycle.rs:36-62, close_flow) |
| Stories | S-TBD |
| Origin BC | BC-RAS-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.011 -- related to (FIN-based close; alternative close path)
- BC-2.04.012 -- related to (finalize: third close path)
- BC-2.04.051 -- composes with (RST state transition)
- BC-2.04.029 -- related to (missing-key warning in close_flow)

## Architecture Anchors

- `src/reassembly/mod.rs:272-278` -- RST block in apply_handshake_flags
- `src/reassembly/lifecycle.rs:36-62` -- close_flow: flush + remove + on_flow_close
- `src/reassembly/flow.rs:257-259` -- on_rst: unconditional state=Closed

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:272-278` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if tcp.rst` block with explicit counter increment and close_flow call
- **type constraint**: PostHandshake enum ensures process_packet stops after RST

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats, self.total_memory |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callback) |

## Refactoring Notes

No refactoring needed. RST handling is clean and isolated in apply_handshake_flags.
