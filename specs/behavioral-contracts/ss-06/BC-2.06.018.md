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
  - "v1.3 (2026-06-13): P19-B-08 ss-06 line-anchor re-sync — counted_as_non_http latch :410-413→:429-432; field decl :89→:91. Verified against current src/analyzer/http.rs (1044 lines)."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.018: non_http_flows Counts Flow Once Even if Both Directions Poisoned

## Description

`non_http_flows` is a flow-level counter, not a direction-level counter. When the first
direction (request or response) reaches `POISON_THRESHOLD` consecutive errors and is poisoned,
`non_http_flows` is incremented by 1 and `counted_as_non_http` is set to `true` for that
flow. If the second direction subsequently also reaches the poison threshold, `non_http_flows`
is NOT incremented again because `counted_as_non_http` is already true. The latch ensures
each flow contributes at most 1 to `non_http_flows`.

## Preconditions

1. An HttpFlowState exists for the FlowKey with `counted_as_non_http == false`.
2. A direction reaches `request_error_count >= POISON_THRESHOLD (3)` for the first time.

## Postconditions

1. `non_http_flows` incremented by 1.
2. `counted_as_non_http = true` (latch set).
3. If the SECOND direction subsequently also reaches poison threshold:
   - The direction is poisoned.
   - `counted_as_non_http` is already true.
   - `non_http_flows` is NOT incremented again.

## Invariants

1. `counted_as_non_http` is a per-flow latch (one bool per HttpFlowState).
2. `non_http_flows` counts flows, not directions.
3. The latch is checked before incrementing: `if !state.counted_as_non_http { state.counted_as_non_http = true; self.non_http_flows += 1; }` (http.rs:429-432).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Only request direction poisoned | non_http_flows=1 |
| EC-002 | Both directions poisoned | non_http_flows=1 (not 2) |
| EC-003 | Two separate flows, each having one direction poisoned | non_http_flows=2 |
| EC-004 | Same flow closed and reopened; second instance also poisoned | non_http_flows=2 (new HttpFlowState per open) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Flow A: 3 request errors | non_http_flows=1 | happy-path |
| Flow A: 3 request errors + 3 response errors | non_http_flows=1 (not 2) | happy-path |
| Flow A: 3 request errors; Flow B: 3 request errors | non_http_flows=2 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | non_http_flows counts per-flow, not per-direction | unit: test_non_http_flows_counts_per_flow_not_direction |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- per-flow non_http_flows counting is part of HTTP analysis statistics |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- counted_as_non_http latch is part of this) |
| Architecture Module | SS-06 (analyzer/http.rs:429-432, C-12) |
| Stories | STORY-044 |
| Origin BC | BC-HTTP-018 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.015 -- composes with (poisoning is the trigger for non_http_flows increment)
- BC-2.06.017 -- composes with (per-direction independence means both can be poisoned)
- BC-2.06.023 -- composes with (non_http_flows appears in summarize() detail map)

## Architecture Anchors

- `src/analyzer/http.rs:429-432` -- counted_as_non_http latch and non_http_flows increment
- `src/analyzer/http.rs:91` -- counted_as_non_http field in HttpFlowState
- `tests/http_analyzer_tests.rs` -- test_non_http_flows_counts_per_flow_not_direction

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:429-432` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if !state.counted_as_non_http { ... self.non_http_flows += 1; }`
- **assertion**: test_non_http_flows_counts_per_flow_not_direction

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates counted_as_non_http, non_http_flows |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
