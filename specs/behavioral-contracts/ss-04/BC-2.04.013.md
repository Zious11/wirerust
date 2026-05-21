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

# BC-2.04.013: expire_flows Closes Idle Flows Past flow_timeout_secs

## Description

`TcpReassembler::expire_flows(current_time, handler)` iterates all flows and closes any that
are either already `Closed` (state machine reached Closed without cleanup) or have been idle
for more than `flow_timeout_secs` seconds (`current_time - last_seen > timeout`). Each
expired flow is closed with `CloseReason::Timeout` and `stats.flows_expired` is incremented
for each one. The caller is responsible for passing `current_time` (typically the timestamp
of the packet being processed).

## Preconditions

1. `expire_flows(current_time, handler)` is called with a current timestamp.
2. At least one flow exists with `last_seen + flow_timeout_secs < current_time`, OR a flow
   exists with `state == FlowState::Closed`.

## Postconditions

1. All flows satisfying `state == Closed OR (current_time > last_seen AND current_time -
   last_seen > timeout)` are closed via `close_flow(key, CloseReason::Timeout, handler)`.
2. `stats.flows_expired` increments by the number of flows expired.
3. Each expired flow's remaining buffered data is flushed and `on_flow_close(Timeout)` is
   called.
4. Flows not meeting either expiry condition are untouched.

## Invariants

1. The expiry check uses wrapping-safe subtraction: `current_time > flow.last_seen` is
   checked BEFORE `current_time - flow.last_seen > timeout` to avoid integer underflow.
2. A flow that is already `FlowState::Closed` (e.g., FIN-closed but not yet removed) is also
   expired here. This handles the edge case where close_flow was not called in the same pass.
3. `expire_flows` does NOT expire flows that are still active within the timeout window.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow last_seen 1 second ago; timeout=300 | Not expired |
| EC-002 | Flow last_seen 301 seconds ago; timeout=300 | Expired; flows_expired++ |
| EC-003 | current_time == last_seen (no idle time) | Not expired (current_time - last_seen == 0, not > timeout) |
| EC-004 | Flow with state=Closed | Expired regardless of idle time |
| EC-005 | current_time < last_seen (timestamp wrap or reorder) | current_time > last_seen is false; NOT expired |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Flow with last_seen=0, current_time=400, timeout=300 | Flow expired; flows_expired=1 | happy-path |
| Flow with last_seen=100, current_time=300, timeout=300 | Not expired (300-100=200 <= 300) | edge-case |
| Flow with state=Closed, any timestamps | Expired regardless | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Flows past timeout are closed | unit: create flow, expire with past timeout |
| — | Non-expired flows survive | unit: expire; assert surviving flows count |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- idle flow expiry is required to bound memory use in long-running captures |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:536-552, expire_flows) |
| Stories | S-TBD |
| Origin BC | BC-RAS-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.014 -- related to (total_memory decrements as flows are expired)
- BC-2.04.015 -- related to (max_flows eviction: complementary capacity mechanism)
- BC-2.04.012 -- related to (finalize: end-of-capture expire-all)

## Architecture Anchors

- `src/reassembly/mod.rs:536-552` -- expire_flows: filter + close loop

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:536-552` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `current_time > flow.last_seen && (current_time - flow.last_seen) > timeout`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats, self.total_memory |
| **Deterministic** | yes (given fixed current_time) |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callbacks) |

## Refactoring Notes

No refactoring needed. The guard ordering (current_time > last_seen first) prevents
underflow correctly.
