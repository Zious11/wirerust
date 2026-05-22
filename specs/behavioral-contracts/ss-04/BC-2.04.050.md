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

# BC-2.04.050: Flow State Machine: New->SynSent->Established->Closing->Closed

## Description

`TcpFlow` implements a finite state machine with five states: `New`, `SynSent`,
`Established`, `Closing`, and `Closed`. Transitions are driven by the four handshake events:
`on_syn()` (SYN without ACK), `on_syn_ack()` (SYN+ACK), `on_fin()` (FIN from either
direction), and `on_rst()` (RST). `on_data_without_syn()` directly transitions `New ->
Established` for mid-stream joins.

## Preconditions

1. A `TcpFlow` is created with initial `state = FlowState::New`.

## Postconditions

State transition table:

| Event | From | To |
|-------|------|----|
| on_syn() | New | SynSent |
| on_syn() | SynSent | SynSent (no-op guard) |
| on_syn_ack() | SynSent | Established |
| on_syn_ack() | New | Established (server-first SYN+ACK without prior SYN) |
| on_data_without_syn() | New | Established (+ partial=true) |
| on_fin() (first) | Established | Closing |
| on_fin() (first) | SynSent | Closing |
| on_fin() (second, fin_count >= 2) | any | Closed |
| on_rst() | any | Closed |

States not in the table are unchanged (no-op).

## Invariants

1. `on_syn()` only transitions from `New` (guard: `if self.state == FlowState::New`).
2. `on_syn_ack()` transitions from `SynSent` OR `New` (server-first scenario).
3. `on_data_without_syn()` only transitions from `New`.
4. `fin_count` uses `saturating_add(1)` to prevent u8 overflow.
5. Once in `Closed`, no transition occurs (RST and FIN from Closed are effectively no-ops
   because the flow is removed from the table before the next packet).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SYN seen before SYN+ACK (normal handshake) | New -> SynSent -> Established |
| EC-002 | SYN+ACK without prior SYN | New -> Established directly |
| EC-003 | on_syn() when state is SynSent | No-op (guard prevents double-transition) |
| EC-004 | Two FINs from the same direction | fin_count >= 2 after first two; state -> Closed |
| EC-005 | RST received from Established | Closed immediately |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| New flow, on_syn() | state=SynSent | happy-path |
| SynSent flow, on_syn_ack() | state=Established | happy-path |
| Established flow, on_fin() once | state=Closing; fin_count=1 | happy-path |
| Closing flow, on_fin() again | state=Closed; fin_count=2 | happy-path |
| New flow, on_rst() | state=Closed | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | All state transitions match the table above | unit: test_flow_state_transitions |
| VP-009 | on_syn() no-op when not in New state | unit |
| VP-009 | fin_count saturating_add prevents overflow | unit: boundary test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- the TCP flow state machine is the lifecycle management core of the reassembly engine |
| L2 Domain Invariants | INV-7 (Finalize-once latch -- the state machine drives flow close events that finalize calls) |
| Architecture Module | SS-04 (reassembly/flow.rs:236-266, C-7) |
| Stories | STORY-013 |
| Origin BC | BC-RAS-050 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.004 -- depends on (SYN processing calls on_syn)
- BC-2.04.005 -- depends on (SYN+ACK processing calls on_syn_ack)
- BC-2.04.010 -- depends on (RST processing calls on_rst)
- BC-2.04.011 -- depends on (FIN processing calls on_fin)
- BC-2.04.052 -- depends on (mid-stream join calls on_data_without_syn)
- BC-2.04.051 -- composes with (RST transitions to Closed from any state)

## Architecture Anchors

- `src/reassembly/flow.rs:236-266` -- on_syn, on_syn_ack, on_data_without_syn, on_fin, on_rst

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:236-266` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_flow_state_transitions (reassembly_flow_tests)
- **type constraint**: FlowState enum ensures only valid states exist

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.state, self.fin_count, self.partial |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |

## Refactoring Notes

No refactoring needed -- pure state machine transitions. Suitable for Kani formal verification of the state machine reachability and transition completeness.
