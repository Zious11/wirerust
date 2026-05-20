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

# BC-2.06.012: Well-Formed HTTP Request Produces Zero Findings

## Description

A syntactically valid, semantically normal HTTP/1.1 request (standard method, short URI, valid
Host header, non-empty User-Agent) triggers no anomaly detections and produces zero findings.
This is the baseline/negative case that validates the absence of false positives on clean
traffic. All counters are updated normally (method, host, UA, URI), but `all_findings` remains
empty for the flow.

## Preconditions

1. A complete HTTP/1.1 request has been parsed.
2. Method is one of: GET, POST, PUT, PATCH, HEAD.
3. URI length <= 2048.
4. URI does not contain path traversal, web-shell, or admin-panel patterns.
5. Host header is present and non-empty.
6. User-Agent header is either absent or non-empty.

## Postconditions

1. `all_findings` gains zero new entries from this request.
2. `methods`, `hosts`, `user_agents`, `uris` counters updated normally.
3. `parse_errors` is unchanged.

## Invariants

1. All anomaly detections are independently gated; none fires on clean input.
2. Zero findings is the expected steady-state for legitimate HTTP traffic.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | GET /index.html HTTP/1.1 with Host and UA | Zero findings |
| EC-002 | POST /api/data HTTP/1.1 with JSON body bytes following | Zero findings from header; body bytes handled by had_success guard |
| EC-003 | HTTP/1.0 GET with no Host | Zero findings (HTTP/1.0 exempt from Host check) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n | findings.is_empty() == true | happy-path |
| POST /submit HTTP/1.1\r\nHost: example.com\r\nContent-Length: 5\r\n\r\nhello | Zero findings from header parse | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Normal request produces no findings | unit: test_no_findings_for_normal_request |
| VP-TBD | Normal request does not increment parse_errors | unit: test_normal_request_no_parse_errors |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- zero-findings for normal traffic is the baseline correctness guarantee for HTTP analysis |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:183-357, C-12) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-012 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.005 through BC-2.06.011 -- related to (each anomaly detection is the positive case; this is the negative)
- BC-2.06.013 -- related to (non-HTTP bytes trigger error counter, not findings)

## Architecture Anchors

- `tests/http_analyzer_tests.rs` -- test_no_findings_for_normal_request, test_normal_request_no_parse_errors

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:183-357` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_no_findings_for_normal_request

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads all_findings (no mutation for clean traffic) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
