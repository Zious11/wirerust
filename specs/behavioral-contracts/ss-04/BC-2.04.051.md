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

# BC-2.04.051: RST Transitions State to Closed from Any Prior State

## Description

`TcpFlow::on_rst()` unconditionally sets `self.state = FlowState::Closed`, regardless of
the current state. Unlike `on_syn()` and `on_fin()` which have state guards, `on_rst()` has
no guard: a RST from `New`, `SynSent`, `Established`, `Closing`, or `Closed` all result in
`Closed`. The engine processes the RST in `apply_handshake_flags`, calls `on_rst()`, increments
`stats.flows_rst`, and immediately calls `close_flow(key, CloseReason::Rst, handler)`.

## Preconditions

1. A TCP segment with the RST flag set is received on a flow in any state.

## Postconditions

1. `self.state = FlowState::Closed`.
2. The engine returns `PostHandshake::FlowClosed` from `apply_handshake_flags`.
3. `stats.flows_rst` increments.
4. `close_flow(key, CloseReason::Rst, handler)` is called, flushing remaining data and
   notifying the handler.
5. The flow is removed from `self.flows`.

## Invariants

1. `on_rst()` has no state guard: `self.state = FlowState::Closed` is executed unconditionally
   (flow.rs:264).
2. The RST close is performed before payload processing in `process_packet` (the
   `PostHandshake::FlowClosed` return prevents payload processing).
3. A RST-closed flow is counted in `stats.flows_rst`, not in `stats.flows_fin`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RST received on New flow (SYN not yet seen) | state=Closed; flows_rst++ |
| EC-002 | RST received on Established flow | state=Closed; flows_rst++ |
| EC-003 | RST received on Closing flow | state=Closed; flows_rst++ |
| EC-004 | RST received simultaneously with payload | Payload is NOT processed (PostHandshake::FlowClosed returned) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Established flow, RST packet | state=Closed; flows_rst=1; CloseReason::Rst emitted | happy-path |
| New flow (no SYN), RST packet | state=Closed; flows_rst=1 | edge-case |
| SynSent flow, RST packet | state=Closed; flows_rst=1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-009 | on_rst() transitions to Closed from any state | unit: test_flow_rst_from_any_state |
| VP-009 | RST closes flow with CloseReason::Rst | unit: test_rst_closes_flow |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- RST handling is a critical TCP lifecycle event in the reassembly engine |
| L2 Domain Invariants | INV-7 (Finalize-once latch -- RST is one of the paths that triggers close_flow, which is also called by finalize) |
| Architecture Module | SS-04 (reassembly/flow.rs:264-266, C-7; reassembly/mod.rs:273-279, C-6) |
| Stories | STORY-013 |
| Origin BC | BC-RAS-051 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.050 -- composes with (RST is one of the state machine transitions)
- BC-2.04.010 -- composes with (the engine-level RST handling calling on_rst)

## Architecture Anchors

- `src/reassembly/flow.rs:264-266` -- on_rst() implementation
- `src/reassembly/mod.rs:273-279` -- engine RST handling calling on_rst and close_flow

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:264-266` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_flow_rst_from_any_state (reassembly_flow_tests) and test_rst_closes_flow (engine tests)
- **guard clause**: absence of state guard in on_rst() (unlike on_syn which checks New state)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.state |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
