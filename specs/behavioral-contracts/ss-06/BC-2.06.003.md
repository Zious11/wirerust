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
  - "v1.3 (2026-05-28): W15 Pass-3 remediation — F-W15P3-006; corrected 459→460 in Architecture Module row and Source Evidence Path to match verified Partial arm location; line anchor reconciled."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.003: Partial Requests Buffered Until Complete; Not Counted Until Full

## Description

When httparse returns `Status::Partial` for the accumulated `request_buf`, `try_parse_requests`
exits the loop immediately without updating any counters or triggering detections. The partial
bytes remain in `request_buf` and are joined with subsequent `on_data` bytes. A request is only
counted and inspected when httparse returns `Status::Complete`. The same behavior applies to
responses via `try_parse_responses`.

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. The request direction is NOT poisoned.
3. `request_buf` contains an incomplete HTTP request header (no terminal `\r\n\r\n` yet).
4. httparse returns `Status::Partial` when called on the current buffer.

## Postconditions

1. No method, host, UA, or URI counters are updated.
2. No anomaly detection is triggered.
3. `request_buf` retains the partial bytes unchanged.
4. `request_error_count` is NOT incremented (partial is not an error).
5. On a subsequent `on_data`, new bytes are appended and parsing retried.

## Invariants

1. `Status::Partial` is distinct from `Err`; it does not increment `parse_errors` and does
   not advance `request_error_count` toward the poison threshold (INV-8).
2. `request_buf` growth is bounded by `MAX_HEADER_BUF = 65,536` bytes (BC-2.06.022).
3. A request that never completes before `on_flow_close` is silently discarded (the
   `HttpFlowState` is dropped in `on_flow_close`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First on_data has 10 bytes (request header start) | Buffer holds 10 bytes; no count change |
| EC-002 | Second on_data completes the request | Combined buffer parsed; counters updated |
| EC-003 | Partial request followed by a parse error | Error counted against error_count; partial dropped |
| EC-004 | Partial request whose full size exceeds MAX_HEADER_BUF | Bytes past cap dropped; completion may never arrive; silently discarded on close |
| EC-005 | TCP flow closes with partial request in buffer | Buffer discarded; no partial-request count or error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| on_data("GET /test HTTP/1.1\r\nHost: ") | methods empty; buffer holds partial | edge-case |
| on_data("GET /test HTTP/1.1\r\nHost: ") then on_data("h.com\r\n\r\n") | methods["GET"]=1 | happy-path |
| on_data (partial response) then on_data (completes response) | transactions incremented on second call | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Partial input leaves counters unchanged | unit: test_parse_partial_request |
| — | Partial + completion increments counter exactly once | unit: test_partial_response_reassembly |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- buffering partial request headers is required for correct HTTP parsing over TCP stream data |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:402, 460, C-12) |
| Stories | STORY-041 |
| Origin BC | BC-HTTP-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.001 -- composes with (partial is the pre-condition for eventual completion)
- BC-2.06.002 -- composes with (pipelined loop exits on Partial)
- BC-2.06.015 -- related to (Partial is not an error; does not advance error_count)
- BC-2.06.022 -- composes with (buffer is bounded; partial beyond cap is lost)

## Architecture Anchors

- `src/analyzer/http.rs:402` -- `Some(Ok(None)) => return` (Partial arm, requests)
- `src/analyzer/http.rs:460` -- `Some(Ok(None)) => return` (Partial arm, responses)
- `tests/http_analyzer_tests.rs` -- test_parse_partial_request, test_partial_response_reassembly

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:402, 460` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `Some(Ok(None)) => return` arm in try_parse_requests loop
- **assertion**: test_parse_partial_request

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads HttpFlowState.request_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
