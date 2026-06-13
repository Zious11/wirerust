---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/http.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-06
capability: CAP-06
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3 (2026-06-13): P19-B-08 ss-06 line-anchor re-sync — on_flow_close :540-542→:573-575. Verified against current src/analyzer/http.rs (1044 lines)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.019: on_flow_close Removes Per-Flow State; Reopening Same Key Starts Fresh

## Description

`HttpAnalyzer::on_flow_close` calls `self.flows.remove(flow_key)`. This drops the entire
`HttpFlowState` for the flow including buffers, error counts, and poison flags. If the same
`FlowKey` is subsequently seen in a new TCP connection, a fresh `HttpFlowState` is created
via `entry().or_insert_with(HttpFlowState::new)`. The new state starts with all fields at
their zero/false values.

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. `on_flow_close` is called for that FlowKey (with any CloseReason).

## Postconditions

1. `self.flows.remove(flow_key)` is called; the entry is removed.
2. No other HttpAnalyzer state is modified by `on_flow_close`.
3. A subsequent `on_data` for the same FlowKey creates a brand-new `HttpFlowState::new()`.
4. The new state has: request_poisoned=false, response_poisoned=false, error_count=0,
   counted_as_non_http=false, empty buffers.

## Invariants

1. `on_flow_close` does not affect aggregate counters (`transactions`, `parse_errors`,
   `non_http_flows`, etc.). Those are preserved across flow lifetimes.
2. The CloseReason parameter is ignored by HttpAnalyzer (the parameter is `_reason`).
3. The poison state of a prior flow cannot carry over to a subsequent flow on the same key.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow close on poisoned flow; same key reopened | New flow starts with poison=false |
| EC-002 | Flow close on a FlowKey not in self.flows (benign race) | remove() is a no-op; no panic |
| EC-003 | Flow close called with direction data still in buffer | Buffer is discarded with the state |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Poison a flow (3 errors); on_flow_close; on_data same key with valid request | Method counted; request_poisoned=false | happy-path |
| on_flow_close without prior state | No panic; no state change | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-014 | Poison state cleared after flow close and reopen | unit: test_poison_cleared_after_flow_close |
| VP-014 | on_flow_close cleans up per-flow state | unit: test_flow_close_cleans_up_state |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- flow state cleanup on close is required for correct per-flow HTTP analysis lifecycle |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- on_flow_close is the ONLY reset path) |
| Architecture Module | SS-06 (analyzer/http.rs:573-575, C-12) |
| Stories | STORY-045 |
| Origin BC | BC-HTTP-019 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.015 -- composes with (on_flow_close is the only way to reset poison state)
- BC-2.06.021 -- composes with (cross-flow isolation relies on clean per-flow state)

## Architecture Anchors

- `src/analyzer/http.rs:573-575` -- on_flow_close implementation
- `tests/http_analyzer_tests.rs` -- test_poison_cleared_after_flow_close, test_flow_close_cleans_up_state

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:573-575` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `fn on_flow_close(&mut self, flow_key, _reason) { self.flows.remove(flow_key); }`
- **assertion**: test_poison_cleared_after_flow_close, test_flow_close_cleans_up_state

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flows HashMap (removes one entry) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
