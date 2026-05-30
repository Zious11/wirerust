---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: Wave 6 Ph3 pass-1 re-run adversarial fix F-3: re-synced flow.rs anchors after fin_count() accessor insertion shifted state-machine methods +7 lines; verified mod.rs anchors — product-owner 2026-05-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.052: on_data_without_syn: New->Established; partial=true

## Description

When the first observed packet for a flow carries data but no SYN flag, the engine calls
`TcpFlow::on_data_without_syn()` to handle a mid-stream join. This method transitions the
flow state from `New` to `Established` and sets `self.partial = true`. The engine then calls
`set_initiator` and `infer_isn` to complete the flow setup, and increments `stats.flows_partial`.

## Preconditions

1. `self.state == FlowState::New` (the flow was just created and no SYN was seen).
2. The packet has a non-empty payload (data without SYN).

## Postconditions

1. `self.state = FlowState::Established`.
2. `self.partial = true`.
3. The engine increments `stats.flows_partial`.
4. The engine calls `flow.set_initiator(packet.src_ip, tcp.src_port)` to set the initiator
   as the source of the first data packet.
5. The engine calls `flow.get_direction_mut(dir).infer_isn(tcp.seq)` to set the inferred ISN.

## Invariants

1. `on_data_without_syn()` only transitions from `New` (guard: `if self.state == FlowState::New`).
   If state is already Established (e.g., a second data-without-syn packet arrives), the
   method is a no-op.
2. `partial = true` is permanent once set; it is never reset to `false`.
3. The `flows_partial` counter tracks flows that were joined mid-stream, enabling forensic
   analysis of capture gaps.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow is New; first packet is data (no SYN) | state=Established; partial=true |
| EC-002 | Flow already in Established state; data packet arrives | on_data_without_syn is a no-op (guard prevents re-transition) |
| EC-003 | SYN seen before any data (normal handshake) | on_data_without_syn is never called; partial stays false |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| New flow, data packet (no SYN) | state=Established, partial=true, flows_partial=1 | happy-path |
| Established flow, data packet | state=Established (unchanged), partial unchanged | edge-case |
| Normal SYN + SYN+ACK flow, then data | state=Established, partial=false | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | on_data_without_syn: state=Established, partial=true | unit: test_mid_stream_pickup |
| VP-009 | on_data_without_syn is a no-op when not in New state | unit |
| VP-009 | flows_partial counter increments for each mid-stream join | unit: test_mid_stream_no_syn |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- mid-stream join handling is a core requirement for real-world TCP capture analysis |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering -- initiator is set here for mid-stream flows) |
| Architecture Module | SS-04 (reassembly/flow.rs:248-253, C-7; reassembly/mod.rs:306-312, C-6) |
| Stories | STORY-013 |
| Origin BC | BC-RAS-052 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.009 -- composes with (engine-level mid-stream join handling)
- BC-2.04.050 -- composes with (on_data_without_syn is a state machine transition)
- BC-2.04.031 -- depends on (infer_isn is called after on_data_without_syn)

## Architecture Anchors

- `src/reassembly/flow.rs:248-253` -- on_data_without_syn implementation
- `src/reassembly/mod.rs:306-312` -- engine calling on_data_without_syn for New flows with data

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:248-253` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_mid_stream_pickup (reassembly_flow_tests)
- **assertion**: test_mid_stream_no_syn (reassembly_engine_tests) asserts stats.flows_partial==1
- **guard clause**: `if self.state == FlowState::New` in on_data_without_syn

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.state, self.partial |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
