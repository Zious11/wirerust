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
  - "v1.3: Wave 6 Ph3 pass-1 re-run adversarial fix F-3: re-synced flow.rs anchors after fin_count() accessor insertion shifted state-machine methods +7 lines; verified mod.rs anchors — product-owner 2026-05-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.004: First SYN Sets Client ISN and Initiator

## Description

When a TCP SYN packet (SYN=true, ACK=false) arrives for a new or existing flow, the engine
records the source endpoint as the initiator and sets the ISN for the client-to-server
direction. The flow state transitions from `New` to `SynSent`. This is the first event in the
standard TCP three-way handshake and establishes the canonical direction tagging for the flow.

## Preconditions

1. A TCP packet with SYN=true and ACK=false arrives.
2. The flow for the packet's FlowKey exists (created by `get_or_create_flow` in the same
   `process_packet` call).
3. `flow.state == FlowState::New` (the flow was just created).

## Postconditions

1. `flow.initiator == Some((packet.src_ip, tcp.src_port))` (set via `set_initiator`; only
   takes effect if initiator was previously None).
2. The direction corresponding to `src_ip:src_port` (ClientToServer) has `isn == Some(tcp.seq)`.
3. `flow.state == FlowState::SynSent` (via `on_syn()`; only transitions from New).
4. `flow.client_to_server.base_offset == 1` (ISN+1 is first data byte).

## Invariants

1. `set_initiator` is idempotent: only sets if currently None. A second SYN on the same flow
   does not change the recorded initiator.
2. `set_isn` is idempotent: only sets if currently None. A retransmitted SYN does not change
   the ISN.
3. `on_syn()` transitions only from `New`; calling it on `Established` or `SynSent` is a
   no-op.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Retransmitted SYN (same flow, SYN again) | set_initiator no-op; set_isn no-op; on_syn no-op (state already SynSent) |
| EC-002 | SYN arrives after SYN+ACK (out-of-order capture) | set_initiator/set_isn both no-op (already set by SYN+ACK); on_syn no-op |
| EC-003 | SYN on new flow when flow table is at max_flows | get_or_create_flow evicts first; SYN processing proceeds normally |
| EC-004 | SYN with payload (unusual; valid TCP) | ISN set; payload processed via insert_payload_segment in same process_packet call |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SYN from 1.1.1.1:5000 to 2.2.2.2:80 | initiator=(1.1.1.1,5000); c2s.isn=tcp.seq; state=SynSent | happy-path |
| Two SYNs from same source (retransmit) | State SynSent; ISN unchanged after second SYN | edge-case |
| SYN then SYN+ACK on same flow | After SYN: initiator=client; After SYN+ACK: state=Established | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | SYN sets initiator and ISN in ClientToServer direction | unit: construct SYN packet; assert flow state post-process |
| VP-009 | Retransmitted SYN does not change ISN or initiator | unit: process two SYNs; assert values unchanged |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- handshake tracking is foundational to correct directional reassembly |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering -- direction tagging depends on initiator identity) |
| Architecture Module | SS-04 (reassembly/mod.rs:257-263, apply_handshake_flags; flow.rs:208-212) |
| Stories | STORY-013 |
| Origin BC | BC-RAS-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.003 -- composes with (FlowKey built before this; initiator within the keyed flow)
- BC-2.04.005 -- composes with (SYN+ACK completes the handshake)
- BC-2.04.050 -- composes with (state machine: New->SynSent)
- BC-2.04.053 -- depends on (direction() uses initiator set here)

## Architecture Anchors

- `src/reassembly/mod.rs:257-263` -- apply_handshake_flags SYN block
- `src/reassembly/flow.rs:208-212` -- set_initiator (idempotent)
- `src/reassembly/flow.rs:136-140` -- set_isn (idempotent; base_offset = 1)
- `src/reassembly/flow.rs:236-240` -- on_syn() state transition

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:257-263` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if tcp.syn && !tcp.ack` block
- **type constraint**: `set_initiator` and `set_isn` enforce idempotency via `if self.xxx.is_none()`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow state within self.flows |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. Logic is clear and isolated in apply_handshake_flags.
