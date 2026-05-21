---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/dispatcher.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-05
capability: CAP-05
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.007: unclassified_flows Increments Only at on_flow_close

## Description

`unclassified_flows` is a u64 counter that increments only in `on_flow_close`, and only when
the flow being closed had no cached route (either `None` from `routes.remove()` or
`Some(DispatchTarget::None)` -- both represent unclassified). The counter does NOT increment
during `on_data`. Classified flows (Http or Tls route) do not contribute to this counter.
The counter is exposed via `dispatcher.unclassified_flows()` and injected into the reassembly
summary detail map (BC-2.12.015).

## Preconditions

1. `on_flow_close` is called for a FlowKey.
2. `routes.remove(flow_key)` returns either `None` (no route entry) or
   `Some(DispatchTarget::None)` (cached None from retry-cap logic).
3. At least one of `self.http` or `self.tls` is configured (the counter does not increment
   for unconfigured dispatchers -- dispatcher.rs:188-191).

## Postconditions

1. `self.unclassified_flows` incremented by 1.
2. No analyzer's `on_flow_close` is called (since the flow was unclassified).

## Invariants

1. `unclassified_flows` is a monotonically increasing u64 counter; never decrements.
2. Classified flows (Http or Tls) do NOT increment `unclassified_flows` on close.
3. The counter increments only when at least one analyzer is configured (guard at
   dispatcher.rs:188-191: `if self.http.is_some() || self.tls.is_some()`).
4. Flows with no data (SYN-only, no content) may land here, making this metric potentially
   misleading for handshake-only flows (noted in cap-05).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow classified as Http then closed | Http.on_flow_close called; unclassified NOT incremented |
| EC-002 | Flow never classified (no data sent) | unclassified_flows=1 on close |
| EC-003 | Flow with None-cached route closed | unclassified_flows=1 on close |
| EC-004 | Dispatcher has no analyzers configured; unclassified flow closed | unclassified NOT incremented |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Send unknown bytes; on_flow_close | unclassified_flows=1 | happy-path |
| Classify as HTTP; on_flow_close | unclassified_flows=0 | happy-path |
| Two unclassified flows closed | unclassified_flows=2 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | unclassified_flows increments only at close for unclassified flows | unit: test_unclassified_flows_counter |
| — | Classified flow close does not increment unclassified | unit: test_classified_flow_not_counted_as_unclassified |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- unclassified_flows counter is the observability metric for flows the dispatcher could not classify |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- None flows are the unclassified population) |
| Architecture Module | SS-05 (dispatcher.rs:171-194, C-21) |
| Stories | STORY-033 |
| Origin BC | BC-DSP-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.05.004 -- composes with (None flows are the source of unclassified counts)
- BC-2.05.009 -- composes with (on_flow_close is where the increment happens)

## Architecture Anchors

- `src/dispatcher.rs:171-194` -- on_flow_close implementation
- `src/dispatcher.rs:188-191` -- unclassified_flows increment guard
- `tests/dispatcher_tests.rs` -- test_unclassified_flows_counter, test_classified_flow_not_counted_as_unclassified

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:171-194` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `Some(DispatchTarget::None) | None => { if self.http.is_some() || self.tls.is_some() { self.unclassified_flows += 1; } }`
- **assertion**: test_unclassified_flows_counter

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates unclassified_flows, routes, classification_attempts |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
