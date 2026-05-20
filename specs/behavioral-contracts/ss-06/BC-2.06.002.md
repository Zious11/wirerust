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

# BC-2.06.002: Parse Pipelined Requests with Independent Per-Request Counting

## Description

`try_parse_requests` operates as an inner loop, calling `parse_one_request` on the tail of
`request_buf` after each successful parse drains the consumed bytes. This loop continues as
long as `request_buf` is non-empty and httparse returns `Complete`. Each iteration processes
one request independently, updating method/host/UA/URI counters and triggering anomaly
detection separately per request. The loop exits when the buffer is exhausted (returns
`None`), partially filled (`Partial`), or an error occurs.

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. The request direction is NOT poisoned.
3. `request_buf` contains bytes from multiple back-to-back complete HTTP request headers.

## Postconditions

1. Each complete request in the buffer is parsed as an independent transaction.
2. Method/host/UA/URI counters are incremented once per request.
3. Anomaly detections are triggered once per request (e.g., path traversal fires per request).
4. The drained bytes after each parsed request expose the next request's bytes for the next
   loop iteration.
5. The loop exits cleanly when no more complete requests remain; partial bytes are retained.

## Invariants

1. `request_error_count` is reset to 0 after each successful parse within the loop.
2. The `had_success` flag prevents error counting for body-bytes that follow a successfully
   parsed header (http.rs:364).
3. Each request's detection and counting is isolated; findings do NOT aggregate across requests
   in a single call.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Three complete requests in one on_data call | All three parsed; method count incremented 3 times |
| EC-002 | Two complete requests + one partial | Two parsed; partial retained in buffer |
| EC-003 | Single complete request | Loop runs once, exits on next iteration with empty/partial buffer |
| EC-004 | Both pipelined requests trigger path traversal | Two separate findings emitted |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /a HTTP/1.1\r\nHost: h\r\n\r\nGET /b HTTP/1.1\r\nHost: h\r\n\r\n | methods["GET"]=2, uris=["/a","/b"] | happy-path |
| POST /login HTTP/1.1\r\n...\r\n\r\nGET /admin HTTP/1.1\r\n...\r\n\r\n | methods["POST"]=1, methods["GET"]=1; admin-panel finding emitted for /admin only | happy-path |
| GET /a HTTP/1.1\r\n...\r\n\r\n + (partial header for /b) | methods["GET"]=1, uris=["/a"]; partial retained | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Each pipelined request counted independently | unit: test_parse_pipelined_requests |
| VP-TBD | Loop stops cleanly on partial; partial retained | unit: test_parse_partial_request |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- pipelined request handling is a required behavior for HTTP/1.1 analysis |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:359-437, C-12) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.001 -- composes with (per-request parsing logic is the inner step)
- BC-2.06.003 -- composes with (partial exit condition is the complement)
- BC-2.06.020 -- related to (body bytes after a parsed header invoke had_success guard)

## Architecture Anchors

- `src/analyzer/http.rs:359-437` -- try_parse_requests loop
- `tests/http_analyzer_tests.rs` -- test_parse_pipelined_requests

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:359-437` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `while` loop with drain-and-retry pattern
- **assertion**: test_parse_pipelined_requests

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates HttpAnalyzer and HttpFlowState |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
