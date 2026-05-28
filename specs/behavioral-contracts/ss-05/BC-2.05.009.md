---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - v1.3: W14 Pass 1 remediation: Description past-tense (pass-3 R4 finding closed by STORY-033), VP confidence MED→HIGH with concrete test, Architecture Anchors add STORY-033 BC-prefixed tests — 2026-05-28
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.05.009: on_flow_close Removes Route Entry and Forwards Close

## Description

`StreamDispatcher::on_flow_close` performs two side effects:
1. **Route removal:** `classification_attempts.remove(flow_key)` and `routes.remove(flow_key)`
   are called, cleaning up all dispatcher state for the flow.
2. **Analyzer forwarding:** If the removed route was `Some(DispatchTarget::Http)`, the close is
   forwarded to `http.on_flow_close`; if `Some(DispatchTarget::Tls)`, forwarded to `tls.on_flow_close`.
   Unclassified flows (`None` or `Some(None)`) do NOT forward to any analyzer.

Both side effects are atomic from the dispatcher's perspective (single function call). The
route-removal side-effect is indirectly verified by the `test_unclassified_flows_counter` test.
The analyzer-forward side-effect was a pass-3 R4 finding; it has been independently formalized
and closed by STORY-033 via `test_BC_2_05_009_flow_close_forwards_to_http_analyzer`.

## Preconditions

1. `on_flow_close` is called for a FlowKey with any CloseReason.

## Postconditions

1. `self.classification_attempts.remove(flow_key)` is called (removes attempt counter).
2. `let target = self.routes.remove(flow_key)` -- route entry removed; returns the prior route.
3. If `target == Some(DispatchTarget::Http)`:
   - `if let Some(ref mut http) = self.http { http.on_flow_close(flow_key, reason); }` (safe pattern; no panic on None).
4. If `target == Some(DispatchTarget::Tls)`:
   - `if let Some(ref mut tls) = self.tls { tls.on_flow_close(flow_key, reason); }` (safe pattern; no panic on None).
5. If `target == None || target == Some(DispatchTarget::None)`:
   - `self.unclassified_flows` is incremented (if analyzers are configured).
   - No analyzer receives the close event.

## Invariants

1. `routes.remove()` is always called unconditionally, regardless of flow classification.
2. `classification_attempts.remove()` is also always called (clears retry state).
3. Each flow contributes its close event to exactly one destination (one analyzer or the
   unclassified counter).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Http-classified flow closed | Http.on_flow_close called; route removed |
| EC-002 | Tls-classified flow closed | Tls.on_flow_close called; route removed |
| EC-003 | Unclassified flow (no cached route) closed | unclassified_flows++; no analyzer close call |
| EC-004 | Close called for FlowKey not in routes | remove() is a no-op; None branch executes; unclassified++ |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Classify as Http; on_flow_close | http.flows no longer contains the FlowKey | happy-path |
| Unclassified flow; on_flow_close | unclassified_flows=1 | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Route removed on flow close (route-remove side effect) | HIGH (partial): test_unclassified_flows_counter indirectly pins route-remove via counter |
| — | Analyzer-forward close side effect | HIGH: independently formalized by STORY-033 test_BC_2_05_009_flow_close_forwards_to_http_analyzer (active_flows_len_for_testing assertions post-close for both Http and Tls classified flows) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 |
| Capability Anchor Justification | CAP-05 ("Content-First Protocol Dispatch") per capabilities.md §CAP-05 -- on_flow_close cleans up per-flow dispatcher state and propagates close events to wrapped analyzers |
| L2 Domain Invariants | INV-2 (Content-first dispatch precedence -- route removal ensures clean per-flow lifecycle) |
| Architecture Module | SS-05 (dispatcher.rs:171-194, C-21) |
| Stories | STORY-033 |
| Origin BC | BC-DSP-009 (pass-3 ingestion corpus, MEDIUM confidence -- route-remove side-effect HIGH per R4; analyzer-forward side-effect MEDIUM per R4) |

## Related BCs

- BC-2.05.005 -- composes with (cached route is the thing being removed)
- BC-2.05.007 -- composes with (unclassified_flows increment is part of this function)

## Architecture Anchors

- `src/dispatcher.rs:171-194` -- on_flow_close implementation
- `src/dispatcher.rs:175-176` -- classification_attempts.remove and routes.remove
- `tests/dispatcher_tests.rs` -- test_unclassified_flows_counter (indirectly pins route-remove), test_BC_2_05_009_flow_close_forwards_to_http_analyzer (independently verifies Http and Tls analyzer-forward side effects via active_flows_len_for_testing post-close), test_BC_2_05_009_flow_close_for_unknown_flow_key (verifies None branch increments unclassified_flows for unknown FlowKey)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/dispatcher.rs:171-194` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: match on `routes.remove(flow_key)` result
- **test-vector**: analyzer-forward path independently verified by STORY-033 test_BC_2_05_009_flow_close_forwards_to_http_analyzer; both Http and Tls routes confirmed via active_flows_len_for_testing assertions post-close

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates routes, classification_attempts, unclassified_flows, and downstream analyzer state |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
