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

# BC-2.06.020: HTTP Body Bytes After Header Completion Do Not Inflate parse_errors

## Description

After `try_parse_requests` successfully parses a complete HTTP request header (`had_success =
true`), the remaining bytes in the buffer (which are HTTP body data, not headers) are
attempted with `parse_one_request`. If this attempt fails (because body data is not valid HTTP
headers), the `had_success` flag suppresses both `parse_errors` increment and
`request_error_count` advance. This prevents body bytes from inflating parse_errors or
inadvertently triggering the poison threshold.

## Preconditions

1. A complete HTTP request has been parsed in this call, setting `had_success = true`.
2. Remaining bytes in `request_buf` after draining the header are body data (not a second request header).
3. `parse_one_request` returns `Err` when called on the body bytes.

## Postconditions

1. `parse_errors` is NOT incremented.
2. `request_error_count` is NOT incremented.
3. `request_buf` is cleared.
4. The loop exits (return).

## Invariants

1. `had_success` is a local bool initialized to `false` per `try_parse_requests` invocation.
2. The suppression is unconditional for any Err when `had_success == true`; this is a
   deliberate "body bytes after success" tolerance per the source comment (http.rs:362-364).
3. The TooManyHeaders finding check (inside the `if !had_success` block) is also suppressed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | POST request header + JSON body in same on_data | Header parsed (had_success=true); body fails parse silently |
| EC-002 | Pipelined: req1 + req2 -- req2 is an error | req1 counted; second error suppressed if had_success=true |
| EC-003 | Two errors before any success | Both counted (had_success still false) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| POST / HTTP/1.1\r\n...\r\n\r\n{"json":"body"} | parse_errors=0 (body bytes not counted as error) | happy-path |
| 2 error buffers + valid header | parse_errors=2; then header success; then body does not add more | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Body bytes after header success do not inflate parse_errors | unit: test_body_bytes_do_not_inflate_parse_errors |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- had_success suppression is part of the HTTP parsing resilience design |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- had_success prevents body bytes from advancing toward threshold) |
| Architecture Module | SS-06 (analyzer/http.rs:362-408, C-14) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-020 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.013 -- composes with (parse error counting is gated by had_success)
- BC-2.06.015 -- related to (body bytes cannot inadvertently trigger poisoning)

## Architecture Anchors

- `src/analyzer/http.rs:362-364` -- had_success local variable and initialization
- `src/analyzer/http.rs:403-408` -- `if !had_success` guard before error counting
- `tests/http_analyzer_tests.rs` -- test_body_bytes_do_not_inflate_parse_errors

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:362-408` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if !had_success { self.parse_errors += 1; ... }`
- **assertion**: test_body_bytes_do_not_inflate_parse_errors

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | conditionally mutates parse_errors and error_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
