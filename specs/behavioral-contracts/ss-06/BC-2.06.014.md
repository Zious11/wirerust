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

# BC-2.06.014: Too Many Headers Emits Anomaly/Inconclusive/Medium Finding (T1499.002)

## Description

When httparse returns `Err(httparse::Error::TooManyHeaders)` -- meaning the request or
response contained more than `MAX_HEADERS = 96` header fields -- the HttpAnalyzer emits an
`Anomaly/Inconclusive/Medium` finding tagged with MITRE T1499.002 (Endpoint Denial of Service:
Service Exhaustion Flood). This is the ONLY parse error that emits a finding. The direction
determines whether the evidence cites "request" or "response". The error also increments
`parse_errors` and `request_error_count` normally (subject to had_success suppression).

## Preconditions

1. An HttpFlowState exists for the FlowKey.
2. httparse returns `Err(httparse::Error::TooManyHeaders)` when attempting to parse the buffer.
3. `had_success == false` in the current loop call.

## Postconditions

1. A Finding is emitted with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Medium
   - mitre_technique: Some("T1499.002")
   - summary: "Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)"
   - evidence: vec!["Direction: request"] or vec!["Direction: response"]
   - direction: Some(Direction::ClientToServer) or Some(Direction::ServerToClient)
2. `parse_errors` incremented by 1.
3. `request_error_count` (or response equivalent) incremented by 1.
4. Buffer cleared.
5. Function returns early.

## Invariants

1. The TooManyHeaders finding fires at the same time as the error-counter increment.
2. `MAX_HEADERS = 96` is the httparse capacity; exceeding it requires 97+ headers in one request.
3. The finding does NOT bypass the usual error-count path; the direction will eventually be
   poisoned if this error repeats.
4. The direction-specific evidence text ("Direction: request" vs "Direction: response") is
   hardcoded, not derived from a Direction enum -- it is a plain string in the evidence vec.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Request with 97 headers | Finding + parse_errors=1 |
| EC-002 | Response with 97 headers | Finding with "Direction: response" evidence |
| EC-003 | TooManyHeaders on 3rd consecutive attempt | Third error also triggers poisoning (BC-2.06.015) AND emits a finding |
| EC-004 | TooManyHeaders after prior success (had_success=true) | Finding NOT emitted; the TooManyHeaders check is nested inside `if !had_success {`, so had_success=true suppresses both the counter increment and the finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Request with 97 headers | Finding(Anomaly/Inconclusive/Medium, T1499.002, evidence=["Direction: request"]) | happy-path |
| Response with 97 headers | Finding(Anomaly/Inconclusive/Medium, T1499.002, evidence=["Direction: response"]) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Too-many-headers request emits T1499.002 finding | unit: test_too_many_headers_generates_finding |
| — | Too-many-headers response emits T1499.002 finding | unit: test_too_many_headers_in_response_generates_finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- excessive-header detection is an HTTP DoS/evasion anomaly finding |
| L2 Domain Invariants | INV-8 (HTTP poisoning is monotonic false-to-true -- TooManyHeaders contributes to error_count) |
| Architecture Module | SS-06 (analyzer/http.rs:416-428, C-12) |
| Stories | STORY-044 |
| Origin BC | BC-HTTP-014 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.013 -- related to (all other errors increment counter without findings; TooManyHeaders is the exception)
- BC-2.06.015 -- composes with (repeated TooManyHeaders advances toward poisoning)

## Architecture Anchors

- `src/analyzer/http.rs:416-428` -- TooManyHeaders arm in request Err block
- `src/analyzer/http.rs:475-487` -- TooManyHeaders arm in response Err block
- `tests/http_analyzer_tests.rs` -- test_too_many_headers_generates_finding, test_too_many_headers_in_response_generates_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:416-428, 475-487` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if e == httparse::Error::TooManyHeaders { self.all_findings.push(...) }`
- **assertion**: test_too_many_headers_generates_finding

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings, parse_errors, error_count |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
