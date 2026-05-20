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

# BC-2.04.012: finalize Flushes All Remaining Flows; Idempotent

## Description

`TcpReassembler::finalize(handler)` is the end-of-capture cleanup method. It closes every
remaining flow in the engine (those not already closed by RST/FIN/timeout), using
`CloseReason::Timeout` for each. A `finalized: bool` latch makes subsequent calls no-ops.
After all flows are closed, `finalize` optionally emits a segment-limit summary finding
(BC-2.04.025). The `impl Drop` tripwire (BC-2.04.012 adjacency) emits a one-shot warning if
the reassembler is dropped without finalize having been called.

## Preconditions

1. `TcpReassembler` has been used to process packets (may have 0 or more open flows).
2. `finalize` has not previously been called (`finalized == false`).

## Postconditions

1. All flows remaining in `self.flows` are closed via `close_flow(key, CloseReason::Timeout,
   handler)`.
2. `self.flows` is empty after finalize completes.
3. `self.finalized == true`.
4. If `stats.segments_segment_limit > 0`, a segment-limit summary finding is pushed
   unconditionally (bypassing MAX_FINDINGS cap).
5. Calling `finalize` a second time: the `if self.finalized { return; }` guard makes it a
   no-op; no flows are double-closed, no extra findings emitted.

## Invariants

1. The `finalized` latch is set to `true` before any flow-closing work; this prevents
   re-entrancy if close_flow triggers a panic.
   The source confirms `self.finalized = true` is set at line 561 before the
   flow loop, ensuring idempotency.
2. The segment-limit finding push in finalize is the ONLY path that bypasses MAX_FINDINGS.
3. `impl Drop` is a DIAGNOSTIC ONLY -- it cannot flush flows because `Drop::drop` has no
   `handler` argument.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero open flows at finalize time | No close_flow calls; finalized=true; possibly emits segment-limit finding |
| EC-002 | N open flows at finalize time | All N closed with Timeout; N on_flow_close callbacks |
| EC-003 | finalize called twice | Second call is immediate no-op (finalized guard) |
| EC-004 | segments_segment_limit == 0 | No segment-limit finding emitted |
| EC-005 | segments_segment_limit > 0 | Segment-limit finding pushed even if findings.len() == MAX_FINDINGS |
| EC-006 | Drop without finalize | One-shot eprintln on stderr; flows NOT flushed |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Engine with 3 open flows; call finalize | 3 on_flow_close(Timeout) calls; flows.is_empty(); finalized=true | happy-path |
| finalize() called twice | Second call no-op; no double callbacks | edge-case |
| segments_segment_limit=5; finalize | findings contains 1 segment-limit finding with count=5 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | After finalize, flows.is_empty() | unit |
| VP-TBD | finalize is idempotent | unit: call twice; assert callbacks fire exactly N times |
| VP-TBD | Segment-limit finding emitted when count > 0 | unit: trigger segment limit; call finalize; assert finding present |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- finalize is the end-of-capture lifecycle contract for the reassembly engine |
| L2 Domain Invariants | INV-7 (Finalize-Once Latch), INV-6 (MAX_FINDINGS cap; finalize bypass) |
| Architecture Module | SS-04 (reassembly/mod.rs:557-591, finalize; mod.rs:677-690, impl Drop) |
| Stories | S-TBD |
| Origin BC | BC-RAS-012 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.025 -- composes with (segment-limit finding from finalize)
- BC-2.04.054 -- composes with (finalize bypass of MAX_FINDINGS)
- BC-2.04.013 -- related to (expire_flows: another cleanup path)

## Architecture Anchors

- `src/reassembly/mod.rs:557-591` -- finalize: latch, flow loop, segment-limit finding
- `src/reassembly/mod.rs:677-690` -- impl Drop: tripwire eprintln

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:557-591` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.finalized { return; }` at top of finalize
- **assertion**: finalized=true set before loop

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.finalized, self.findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callbacks) |

## Refactoring Notes

The `finalized` latch is set at mod.rs:560 (before the flow loop) -- this is correct and
prevents re-entry. No refactoring needed.
