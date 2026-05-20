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

# BC-2.06.008: Unusual HTTP Methods Emit Reconnaissance/Inconclusive/Medium Finding

## Description

When the parsed HTTP request method is one of `CONNECT`, `TRACE`, `DELETE`, or `OPTIONS`,
the HttpAnalyzer emits a `Reconnaissance/Inconclusive/Medium` finding with no MITRE technique
assigned. These methods are flagged because they are uncommon in standard web traffic and
have known abuse scenarios (CONNECT for tunneling, TRACE for XST, DELETE for destructive
actions, OPTIONS for fingerprinting).

## Preconditions

1. A complete HTTP request has been parsed.
2. `parsed.method` (as a str) is exactly one of: "CONNECT", "TRACE", "DELETE", "OPTIONS".
3. Method comparison is case-sensitive (httparse preserves method casing).

## Postconditions

1. A Finding is emitted with:
   - category: Reconnaissance
   - verdict: Inconclusive
   - confidence: Medium
   - mitre_technique: None
   - summary: "Unusual HTTP method: <method>"
   - evidence: vec!["<method> <uri>"]
   - direction: Some(Direction::ClientToServer)
2. The finding fires per-request.
3. Standard methods (GET, POST, PUT, PATCH, HEAD) do NOT trigger this detection.

## Invariants

1. Method matching is an exact slice comparison (`unusual_methods.contains(&parsed.method.as_str())`).
2. The comparison is case-sensitive; "delete" (lowercase) would NOT match "DELETE".
3. `mitre_technique` is `None` for this finding (no MITRE ID assigned).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Method = "DELETE" | Finding emitted |
| EC-002 | Method = "CONNECT" | Finding emitted |
| EC-003 | Method = "OPTIONS" | Finding emitted |
| EC-004 | Method = "TRACE" | Finding emitted |
| EC-005 | Method = "GET" | No finding |
| EC-006 | Method = "PATCH" | No finding |
| EC-007 | Method = "delete" (lowercase) | No finding (case-sensitive match) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| DELETE /resource HTTP/1.1\r\n... | Finding(Reconnaissance/Inconclusive/Medium, mitre=None) | happy-path |
| OPTIONS * HTTP/1.1\r\n... | Finding(Reconnaissance/Inconclusive/Medium, mitre=None) | happy-path |
| GET /index HTTP/1.1\r\n... | No unusual-method finding | happy-path |
| CONNECT proxy.example.com:443 HTTP/1.1\r\n... | Finding(Reconnaissance/Inconclusive/Medium) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Unusual method emits Reconnaissance/Inconclusive/Medium finding | unit: test_detect_unusual_method |
| VP-TBD | GET/POST/PUT/HEAD/PATCH do not emit unusual-method finding | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- unusual HTTP method detection is one of the HTTP anomaly findings |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:251-265, C-14) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-008 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.001 -- composes with (method is extracted during request parse)

## Architecture Anchors

- `src/analyzer/http.rs:251-265` -- unusual methods detection block
- `tests/http_analyzer_tests.rs` -- test_detect_unusual_method

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:251-265` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `unusual_methods.contains(&parsed.method.as_str())`
- **assertion**: test_detect_unusual_method

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
