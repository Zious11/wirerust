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

# BC-2.06.013: Non-HTTP Bytes Increment parse_errors; No Token-Error Findings

## Description

When httparse returns an `Err` (e.g., invalid token, malformed request line) and no prior
successful parse has occurred in that loop call (`had_success == false`), `parse_errors` is
incremented by 1 and `request_error_count` advances toward the poison threshold. Critically,
no finding is emitted for parse errors themselves -- only the statistics counter is affected.
The `TooManyHeaders` special case (BC-2.06.014) is the only error path that emits a finding.

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. `request_buf` contains bytes that are not parseable as an HTTP request header.
3. httparse returns an Err value OTHER than TooManyHeaders.
4. `had_success == false` (no prior successful parse in this loop call).

## Postconditions

1. `parse_errors` incremented by 1.
2. `request_error_count` incremented by 1 (saturating, up to u8::MAX).
3. `request_buf` is cleared.
4. No finding is pushed to `all_findings` (no token-error-findings policy).
5. `try_parse_requests` returns early.

## Invariants

1. `had_success` suppresses error counting for body bytes that follow a complete header
   (http.rs:364, 403-408). Body bytes after a parsed header do NOT increment `parse_errors`.
2. `TooManyHeaders` is the only Err variant that also emits a finding (BC-2.06.014).
3. Buffer is cleared on error; the offending bytes are discarded.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Binary garbage bytes | parse_errors=1; no finding emitted |
| EC-002 | SSH protocol bytes (non-HTTP) | parse_errors=1; no finding |
| EC-003 | HTTP body bytes after complete header | NOT counted (had_success=true suppresses) |
| EC-004 | TooManyHeaders error | parse_errors=1 AND a finding is emitted (BC-2.06.014) |
| EC-005 | Three consecutive non-HTTP buffers | parse_errors=3; direction poisoned (BC-2.06.015) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| on_data(b"SSH-2.0-OpenSSH") to request direction | parse_errors=1; findings empty | happy-path |
| on_data(b"\xff\xfe binary garbage") | parse_errors=1; findings empty | happy-path |
| Normal request + body bytes | parse_errors=0 (had_success suppresses body errors) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Non-HTTP bytes increment parse_errors without findings | unit: test_parse_error_increments_counter |
| — | Response parse error also increments parse_errors | unit: test_parse_error_in_response |
| — | Body bytes after success do not inflate parse_errors | unit: test_body_bytes_do_not_inflate_parse_errors |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- parse error counting and no-finding-on-error policy is part of HTTP analysis resilience |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- error_count drives poisoning) |
| Architecture Module | SS-06 (analyzer/http.rs:403-434, C-12) |
| Stories | STORY-044 |
| Origin BC | BC-HTTP-013 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.014 -- related to (TooManyHeaders is the one error that also emits a finding)
- BC-2.06.015 -- composes with (error_count reaching 3 triggers poisoning)
- BC-2.06.020 -- composes with (had_success suppresses error counting for body bytes)

## Architecture Anchors

- `src/analyzer/http.rs:403-434` -- Err arm in try_parse_requests
- `tests/http_analyzer_tests.rs` -- test_parse_error_increments_counter, test_parse_error_in_response

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:403-434` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if !had_success { self.parse_errors += 1; ... }`
- **assertion**: test_parse_error_increments_counter

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates parse_errors, HttpFlowState.request_error_count, request_buf |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
