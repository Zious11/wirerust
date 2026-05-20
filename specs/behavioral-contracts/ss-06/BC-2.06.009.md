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

# BC-2.06.009: HTTP/1.1 Missing or Empty Host Emits Anomaly/Inconclusive/Medium Finding

## Description

For HTTP/1.1 requests (`parsed.version == 1`), the HttpAnalyzer checks the Host header value.
Both absent Host (`host == None`) and empty-value Host (`host == Some("")`) are RFC 7230
violations and emit an `Anomaly/Inconclusive/Medium` finding with distinct summary text. HTTP/1.0
requests are exempt. The two cases are disambiguated in the summary ("without Host header" vs
"with empty Host header") so downstream analysts can distinguish the evasion pattern. Fixed by
PR #71 which closed the empty-value evasion lane.

## Preconditions

1. A complete HTTP/1.1 request has been parsed (`version == 1`).
2. One of:
   - `host == None` (Host header entirely absent), OR
   - `host == Some("")` (Host header present but value is empty after trim).

## Postconditions

1. A Finding is emitted with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Medium
   - mitre_technique: None
   - summary: either "HTTP/1.1 request without Host header" (absent) or
     "HTTP/1.1 request with empty Host header" (present-empty)
   - evidence: vec!["<method> <uri>"]
   - direction: Some(Direction::ClientToServer)
2. The finding fires per-request.
3. HTTP/1.0 requests (`version == 0`) never emit this finding.

## Invariants

1. The `version` field from httparse is 0 for HTTP/1.0, 1 for HTTP/1.1.
2. `find_header` already trims whitespace; `Host:   \r\n` (whitespace-only value) produces
   `Some("")` and triggers the empty-host finding.
3. `mitre_technique` is `None` for this finding.
4. Both cases are RFC non-compliant per RFC 7230 section 5.4 (and RFC 9112 section 3.2).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | HTTP/1.1 with Host: example.com | No finding |
| EC-002 | HTTP/1.1 with no Host header | Finding: "without Host header" |
| EC-003 | HTTP/1.1 with Host: (empty) | Finding: "with empty Host header" |
| EC-004 | HTTP/1.1 with Host:   (whitespace only) | Finding: "with empty Host header" (trim produces "") |
| EC-005 | HTTP/1.0 with no Host header | No finding (version==0 is exempt) |
| EC-006 | HTTP/1.0 with empty Host | No finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET / HTTP/1.1\r\n\r\n (no Host) | Finding(Anomaly/Inconclusive/Medium, "without Host header") | happy-path |
| GET / HTTP/1.1\r\nHost: \r\n\r\n (empty Host) | Finding(Anomaly/Inconclusive/Medium, "with empty Host header") | happy-path |
| GET / HTTP/1.0\r\n\r\n (no Host, HTTP/1.0) | No finding | edge-case |
| GET / HTTP/1.1\r\nHost: example.com\r\n\r\n | No host-anomaly finding | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | HTTP/1.1 missing Host emits finding | unit: test_detect_missing_host_header |
| VP-TBD | HTTP/1.0 missing Host does not emit finding | unit |
| VP-TBD | Empty Host value also emits finding | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- missing/empty Host header detection is a core HTTP protocol compliance anomaly finding |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:283-302, C-14) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.001 -- composes with (host field extracted during request parse)
- BC-2.06.011 -- related to (UA detection is similarly asymmetric: None vs Some(""))

## Architecture Anchors

- `src/analyzer/http.rs:283-302` -- host anomaly detection block (None/empty/non-empty 3-state match)
- `tests/http_analyzer_tests.rs` -- test_detect_missing_host_header

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:283-302` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `match parsed.host.as_deref() { None => Some("without Host"), Some("") => Some("with empty Host"), Some(_) => None }`
- **assertion**: test_detect_missing_host_header

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
