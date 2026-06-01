---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.3: Wave 8 wave-level adv-pass-1 F-1 HIGH closure (S-7.01 sibling-BC propagation, W7.2 recurrence #5): PC3 enforcement-mode notation — \"remaining buffered data flushed in close_flow\" is structurally a defense-in-depth invariant (per-packet flush at mod.rs:162 drains buffer pre-close); enforced via code-review of close_flow flush loop body at lifecycle.rs:52-59; on_flow_close(Timeout) invocation and stats.flows_expired increment are automated-test-verifiable via STORY-019 AC-009/010/011/012. Mirrors BC-2.04.010 v1.5 PC2 + BC-2.04.029 v1.4 + ADR-0004 amendment precedent. — 2026-05-26"
  - "v1.4: Wave 8 wave-level adv-pass-2 F-1 MEDIUM closure (S-7.01 sibling-discipline): added PC5 documenting the force_set_flow_state_for_testing test seam (lifecycle.rs:232-244) — required by STORY-019 AC-012 to discriminate the Closed-state OR-branch of expire_flows (invariant 2). Mirrors BC-2.04.029 v1.4 PC7 pattern; authorized under ADR-0004 Amendment 2 state-injection seam class. — 2026-05-26"
  - "v1.5: Phase-4 HS-043 scope decision: added PC0 (caller obligation) explicitly requiring expire_flows to be invoked from the production per-packet processing path with the packet timestamp — closes the 'tested directly but never called in production' wiring gap identified in holdout-finding-triage-2026-06-01.md. Direct test-only invocations do not satisfy the production wiring requirement. — 2026-06-01"
  - "v1.6: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:536-552 → mod.rs:593-609 (expire_flows public fn); mod.rs:162 → mod.rs:191 (flush_contiguous_data call in process_packet, cited in PC3 enforcement note). — 2026-06-01"
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

**PC0 (Caller Obligation — Production Wiring Requirement):** `expire_flows` MUST be called
from the production per-packet processing path — either inside `process_packet` or in the
per-packet loop in `src/main.rs` — with the arriving packet's `timestamp_secs` as
`current_time`. Calling `expire_flows` only from test code does NOT satisfy this contract.
The production binary must invoke `expire_flows` so that idle flows are actually expired
during real captures. This requirement is the basis of the capability-anchor memory bound
("idle flow expiry is required to bound memory use in long-running captures"). See
`.factory/specs/phase-4-hs043-scope-decision.md` for the governing ruling.

1. `expire_flows(current_time, handler)` is called with a current timestamp from the
   production per-packet path (PC0 above).
2. At least one flow exists with `last_seen + flow_timeout_secs < current_time`, OR a flow
   exists with `state == FlowState::Closed`.

## Postconditions

1. All flows satisfying `state == Closed OR (current_time > last_seen AND current_time -
   last_seen > timeout)` are closed via `close_flow(key, CloseReason::Timeout, handler)`.
2. `stats.flows_expired` increments by the number of flows expired.
3. Each expired flow's remaining buffered data is flushed and `on_flow_close(Timeout)` is
   called. (Enforcement: in the current engine architecture, the per-packet flush at
   `src/reassembly/mod.rs:191` already delivers all contiguous-prefix data BEFORE
   `expire_flows` invokes `close_flow`. The `flush_contiguous` loop at
   `src/reassembly/lifecycle.rs:52-59` inside `close_flow` is therefore structurally a
   defense-in-depth invariant — cannot be triggered to deliver under current engine semantics.
   PC3's flush sub-property is enforced via code-review of the close_flow flush loop body's
   presence; the `on_flow_close(Timeout)` invocation and `stats.flows_expired` increment are
   automated-test-verifiable via STORY-019 AC-009/010/011/012 (mirrors BC-2.04.010 v1.5 PC2 /
   BC-2.04.029 v1.4 PC1-PC3 / ADR-0004 amendment enforcement-mode precedent).)
4. Flows not meeting either expiry condition are untouched.
- **PC5 (Test Seam):** A `#[doc(hidden)] pub fn force_set_flow_state_for_testing(reassembler: &mut TcpReassembler, key: &FlowKey, state: FlowState) -> bool` accessor in `src/reassembly/lifecycle.rs` allows tests to directly mutate a flow's state without going through the state machine. Returns `true` if the flow was found and updated; `false` if no flow with the given key exists. Required by STORY-019 AC-012 to discriminate the state-based OR-branch of expire_flows (invariant 2) from the time-based clause — the test constructs a flow in `FlowState::Closed` with `last_seen` well within the timeout window, proving the state-based clause fires independently.

  **Hygiene constraints:** `#[doc(hidden)]` (kept out of `cargo doc`); `_for_testing` suffix flags intent; MUST NOT be called from production code paths. Authorized as a NEW test-seam class (state-injection, NOT warning-guard) under ADR-0004 Amendment 2 (2026-05-26) "opt-in per-guard, gated by BC-driven need" doctrine.

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
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- idle flow expiry is required to bound memory use in long-running captures |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:593-609, expire_flows) |
| Stories | STORY-019 |
| Origin BC | BC-RAS-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.014 -- related to (total_memory decrements as flows are expired)
- BC-2.04.015 -- related to (max_flows eviction: complementary capacity mechanism)
- BC-2.04.012 -- related to (finalize: end-of-capture expire-all)

## Architecture Anchors

- `src/reassembly/mod.rs:593-609` -- expire_flows: filter + close loop

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:593-609` |
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
