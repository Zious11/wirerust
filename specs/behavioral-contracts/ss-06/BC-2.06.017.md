---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.017: Poisoning is Per-Direction; Poisoned Request Does Not Affect Response

## Description

`request_poisoned` and `response_poisoned` are independent booleans in `HttpFlowState`.
Poisoning in the ClientToServer direction (request) has no effect on the ServerToClient
direction (response) and vice versa. A flow can have one direction poisoned while the other
continues parsing normally. The two directions maintain separate buffers, separate error
counts, and separate poison flags.

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. The request direction has been poisoned (`request_poisoned == true`).
3. The response direction has `response_poisoned == false`.

## Postconditions

1. Subsequent `on_data` calls with `Direction::ServerToClient` continue to parse responses
   normally (not bypassed by the request-side poison flag).
2. `response_error_count` is independent of `request_error_count`.
3. `response_poisoned` remains false until its own threshold is reached.

## Invariants

1. `request_poisoned` only gates `Direction::ClientToServer` data (http.rs:509-511).
2. `response_poisoned` only gates `Direction::ServerToClient` data (http.rs:521-523).
3. The two flags are independent boolean fields in `HttpFlowState`; neither sets the other.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Request poisoned (3 errors); response receives valid HTTP | Response parsed normally |
| EC-002 | Both directions independently reach poison threshold | Both poisoned; non_http_flows incremented only once (BC-2.06.018) |
| EC-003 | Response poisoned; request receives valid HTTP | Request parsed normally |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 3 request-direction errors -> request_poisoned; then valid response | transactions incremented normally | happy-path |
| 3 response-direction errors -> response_poisoned; then valid request | method/URI counted normally | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-006 | Poisoned request direction does not affect response parsing | unit: test_poison_request_does_not_affect_response |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- per-direction poisoning isolation is required for correct HTTP analysis in bidirectional flows |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- per direction) |
| Architecture Module | SS-06 (analyzer/http.rs:509-523, C-12) |
| Stories | STORY-044 |
| Origin BC | BC-HTTP-017 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.015 -- composes with (poisoning transition for each direction)
- BC-2.06.018 -- composes with (non_http_flows counts the flow once, not once per direction)

## Architecture Anchors

- `src/analyzer/http.rs:509-511` -- request_poisoned early-return
- `src/analyzer/http.rs:521-523` -- response_poisoned early-return
- `tests/http_analyzer_tests.rs` -- test_poison_request_does_not_affect_response

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:509-523` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: separate `if state.request_poisoned` and `if state.response_poisoned` branches
- **assertion**: test_poison_request_does_not_affect_response

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads request_poisoned, response_poisoned |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
