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
  - "v1.3: Wave 6 Ph3 pass-1 re-run adversarial fix F-3: re-synced flow.rs anchors after fin_count() accessor insertion shifted state-machine methods +7 lines; verified mod.rs anchors — product-owner 2026-05-22"
  - "v1.4: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:265-271 → mod.rs:295-300 (SYN+ACK block in apply_handshake_flags). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.005: SYN+ACK Marks Server as Responder; State Transitions to Established

## Description

When a TCP SYN+ACK packet (SYN=true, ACK=true) arrives for a flow, the engine identifies the
DESTINATION as the initiator (because the destination sent the original SYN that this SYN+ACK
is responding to), sets the ISN for the server-to-client direction, and transitions the flow
state to `Established`. This completes the two-step SYN / SYN+ACK sequence and marks the flow
as ready to accept data segments.

## Preconditions

1. A TCP packet with SYN=true and ACK=true arrives.
2. A flow for the packet's FlowKey exists in the engine.
3. The flow state is typically `SynSent` (saw the client SYN) or `New` (SYN+ACK seen first,
   mid-capture join).

## Postconditions

1. `flow.initiator == Some((packet.dst_ip, tcp.dst_port))` -- the DESTINATION is the initiator
   (set via `set_initiator`; only takes effect if initiator was previously None).
2. The direction corresponding to the SENDER (`src_ip:src_port`, which is ServerToClient)
   has `isn == Some(tcp.seq)` (set via `set_isn`; idempotent).
3. `flow.state == FlowState::Established` (via `on_syn_ack()`).
4. `flow.server_to_client.base_offset == 1`.

## Invariants

1. The SYN+ACK block runs IF `tcp.syn && tcp.ack`. It is NOT gated by `else if`; it can
   execute in the same packet as the plain-SYN block, but TCP semantics guarantee no real
   packet is simultaneously a pure SYN and a SYN+ACK -- the code is structurally safe.
2. `set_initiator` and `set_isn` are idempotent; if the SYN was processed first, the initiator
   is already set from the SYN's dst_ip and the SYN+ACK merely sets the server ISN.
3. `on_syn_ack()` transitions from either `SynSent` or `New` -- handles mid-capture where only
   SYN+ACK is seen.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SYN+ACK without prior SYN (mid-capture) | initiator = dst_ip:dst_port (the SYN sender inferred); state -> Established |
| EC-002 | SYN+ACK retransmission | set_initiator no-op; set_isn no-op; on_syn_ack no-op if already Established |
| EC-003 | SYN+ACK when flow state is Established (resync) | on_syn_ack is no-op for Established state |
| EC-004 | SYN+ACK with payload | ISN set; payload processed in insert_payload_segment in same call |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SYN from C, then SYN+ACK from S | After SYN+ACK: state=Established; s2c.isn=SYN+ACK seq; initiator=C | happy-path |
| SYN+ACK without prior SYN | state=Established; initiator=dst; s2c.isn set | edge-case |
| Retransmitted SYN+ACK | state unchanged; isn unchanged | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | SYN+ACK sets server ISN and state=Established | unit: process SYN then SYN+ACK; assert flow.state and s2c.isn |
| VP-009 | SYN+ACK-first (no prior SYN) still reaches Established | unit: process SYN+ACK alone |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- SYN+ACK handling completes the handshake model and enables bidirectional data reassembly |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering -- initiator identity set here) |
| Architecture Module | SS-04 (reassembly/mod.rs:295-300, apply_handshake_flags; flow.rs:242-246) |
| Stories | STORY-013 |
| Origin BC | BC-RAS-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.004 -- composes with (SYN precedes SYN+ACK in normal handshake)
- BC-2.04.050 -- composes with (state machine: SynSent->Established)
- BC-2.04.009 -- related to (mid-stream join; data-without-SYN path)

## Architecture Anchors

- `src/reassembly/mod.rs:295-300` -- SYN+ACK block in apply_handshake_flags
- `src/reassembly/flow.rs:242-246` -- on_syn_ack() state transition

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:295-300` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if tcp.syn && tcp.ack` block
- **type constraint**: `on_syn_ack` only transitions from SynSent or New

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow state within self.flows |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. SYN+ACK block is symmetrical to SYN block and clearly isolated.
