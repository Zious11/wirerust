---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.004: Parse HTTP/1.1 Responses with Status Code Counting

## Description

When a complete HTTP response header block arrives in the `response_buf` for a flow,
`try_parse_responses` extracts the status code and increments both `transactions` and
`status_codes[status_code]`. Responses are the only parse path that advances `transactions`.
The response direction supports the same buffering and pipelined-loop semantics as requests
(BC-2.06.003 and BC-2.06.002), and the same poisoning mechanics apply.

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. The response direction (`ServerToClient`) is NOT poisoned.
3. `response_buf` contains a complete HTTP response header (httparse returns `Complete`).

## Postconditions

1. `transactions` incremented by 1.
2. `status_codes` map gains an entry (or increments) for the numeric status code (http.rs:452).
   There is NO `MAX_MAP_ENTRIES` guard on `status_codes`; the key type is u16 which provides a
   natural practical limit of 65535 distinct status codes (see BC-2.06.024 Invariants).
3. The bytes consumed are drained from `response_buf`.
4. `response_error_count` is reset to 0.
5. No anomaly detection is triggered from response parse (all detections are request-side).

## Invariants

1. `transactions` counts parsed HTTP RESPONSES, not requests. The `summarize()` method maps
   `packets_analyzed = self.transactions`, so the summary's packet count is response-based.
2. Response parsing never emits findings for content-based anomalies (no check_response_detections
   function; detections only exist on the request path).
3. `status_codes` can store status_code=0 if httparse returns `code: None` (unwrap_or(0)).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | HTTP/1.1 200 OK response | transactions=1, status_codes[200]=1 |
| EC-002 | HTTP 500 response | transactions=1, status_codes[500]=1 |
| EC-003 | Two pipelined responses | transactions=2, status_codes incremented twice |
| EC-004 | Partial response (no terminal \r\n\r\n) | Buffer retained; transactions unchanged |
| EC-005 | Response with httparse code==None | status_codes[0] incremented; transactions incremented |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| HTTP/1.1 200 OK\r\n\r\n | transactions=1, status_codes[200]=1 | happy-path |
| HTTP/1.1 404 Not Found\r\n\r\n | transactions=1, status_codes[404]=1 | happy-path |
| HTTP/1.1 200 OK\r\n\r\nHTTP/1.1 304 Not Modified\r\n\r\n (pipelined) | transactions=2, status_codes[200]=1, status_codes[304]=1 | edge-case |
| HTTP/1.1 200 (partial, no \r\n\r\n) | transactions=0; buffer held | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Response parse increments transactions and status_codes | unit: test_parse_response |
| VP-TBD | Pipelined responses increment transactions multiple times | unit: test_parse_pipelined_responses |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- response status code tracking is part of HTTP traffic analysis statistics |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:440-497, C-12) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-004 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.001 -- related to (request parsing is analogous; requests do NOT increment transactions)
- BC-2.06.023 -- composes with (summarize maps packets_analyzed = transactions)

## Architecture Anchors

- `src/analyzer/http.rs:440-497` -- try_parse_responses function
- `src/analyzer/http.rs:450-452` -- transactions increment and status_codes update
- `tests/http_analyzer_tests.rs` -- test_parse_response, test_parse_pipelined_responses

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:450-452` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_parse_response, test_parse_pipelined_responses

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates transactions, status_codes, HttpFlowState.response_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
