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
  - "v1.3: F-W16-WAVE-P1-001 — update Verification Properties + Architecture Anchors to renamed STORY-043 formalization test (test_BC_2_06_010_detect_long_uri); Evidence Types legacy citation intentionally preserved — 2026-05-28"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.010: URI Greater Than 2048 Chars Emits Execution/Likely/Medium Finding

## Description

When the parsed HTTP request URI length exceeds 2048 characters, the HttpAnalyzer emits an
`Execution/Likely/Medium` finding. Abnormally long URIs are a common vector for buffer-overflow
attempts, SQL injection payloads, and command injection attacks. The finding summary includes
the character count; the evidence includes the first 200 characters of the URI (truncated).
No MITRE technique ID is assigned.

## Preconditions

1. A complete HTTP request has been parsed.
2. `parsed.uri.len() > 2048` (byte length, not character count -- ASCII-only URIs per HTTP spec
   means byte length equals character count for well-formed URIs).

## Postconditions

1. A Finding is emitted with:
   - category: Execution
   - verdict: Likely
   - confidence: Medium
   - mitre_technique: None
   - summary: "Abnormally long URI (<N> chars)" where N = uri.len()
   - evidence: vec!["URI prefix: <truncate_uri(uri, 200)>"]
   - direction: Some(Direction::ClientToServer)
2. The finding fires per-request.

## Invariants

1. Threshold is strictly greater-than: uri.len() == 2048 does NOT fire; uri.len() == 2049 does.
2. Evidence is truncated to 200 characters via `truncate_uri` (UTF-8 char boundary safe).
3. Summary includes the exact byte count (not the truncated length).
4. `mitre_technique` is `None` for this finding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | URI length = 2048 | No finding (not strictly greater) |
| EC-002 | URI length = 2049 | Finding emitted |
| EC-003 | URI length = 10000 | Finding emitted; evidence shows first 200 chars |
| EC-004 | Long URI also contains ../  | Both long-URI and path-traversal findings emitted |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /<2049 A chars> HTTP/1.1\r\n... | Finding(Execution/Likely/Medium, "Abnormally long URI (2049 chars)") | happy-path |
| GET /<2048 A chars> HTTP/1.1\r\n... | No long-URI finding | edge-case |
| GET /<5000 chars including ../> HTTP/1.1\r\n... | Two findings: long-URI + path-traversal | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | URI > 2048 chars emits Execution/Likely/Medium finding | unit: test_BC_2_06_010_detect_long_uri |
| — | URI of exactly 2048 does not emit long-URI finding | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per domain/capabilities/cap-06-http-analysis.md -- abnormally long URI detection is one of the HTTP anomaly findings |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:304-317, C-12) |
| Stories | STORY-043 |
| Origin BC | BC-HTTP-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.001 -- composes with (URI extracted during request parse)

## Architecture Anchors

- `src/analyzer/http.rs:304-317` -- long URI detection block
- `tests/http_analyzer_tests.rs` -- test_BC_2_06_010_detect_long_uri (in mod bc_2_06_043_formalization)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:304-317` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if parsed.uri.len() > 2048`
- **assertion**: test_detect_long_uri

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates all_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
