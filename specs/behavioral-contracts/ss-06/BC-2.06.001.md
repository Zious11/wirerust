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

# BC-2.06.001: Parse Complete HTTP/1.1 Request with Method/URI/Version/Host/UA

## Description

When a complete HTTP/1.1 (or HTTP/1.0) request header block arrives in the per-direction
buffer, `HttpAnalyzer` drives httparse to extract the method, URI, HTTP version byte, Host
header value, and User-Agent header value. All five fields are stored in a `ParsedRequest`
struct and passed to detection logic and aggregate statistics. A parsed request does not
increment `transactions`; only a parsed response does (per BC-2.06.004).

## Preconditions

1. An HttpFlowState exists (or is created on first call) for the FlowKey.
2. The request direction is NOT poisoned (`request_poisoned == false`).
3. The accumulated `request_buf` contains a complete HTTP request header block (httparse
   returns `Status::Complete`).
4. The buffer contains at most `MAX_HEADERS = 96` header lines.

## Postconditions

1. `methods` map gains an entry (or increments) for the parsed method string, subject to
   `MAX_MAP_ENTRIES = 50,000` cap.
2. If Host header is present and non-empty, `hosts` map gains an entry for the trimmed value,
   subject to cap.
3. If User-Agent header is present, `user_agents` map gains an entry for the trimmed value,
   subject to cap.
4. If `uris.len() < MAX_URIS (10,000)`, the URI is appended to the `uris` Vec.
5. The bytes consumed by the header are drained from `request_buf`; remaining bytes stay.
6. `request_error_count` is reset to 0.
7. Anomaly detections (`check_request_detections`) are invoked on the parsed fields.

## Invariants

1. Header field values are extracted via `String::from_utf8_lossy(h.value).trim().to_string()`
   (find_header at http.rs:70-75). Non-UTF-8 bytes in header values are replaced with U+FFFD.
2. The method string is extracted from `req.method` (ASCII token; httparse guarantees ASCII).
3. Raw bytes are NOT escaped at parse time; ADR 0003 / INV-4 applies.
4. `transactions` is NOT incremented on request parse -- only on response (BC-2.06.004).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | HTTP/1.0 request (version byte == 0) | Parsed normally; Host checks exempt (INV-8 / version==1 gate) |
| EC-002 | Request with both Host and User-Agent headers | Both values stored in respective maps |
| EC-003 | Request with no User-Agent header | user_agent field is None; no UA map entry added |
| EC-004 | Request with no Host header (HTTP/1.0) | host field is None; no hosts map entry; no finding |
| EC-005 | Pipelined requests: two complete headers in one buffer | Loop in try_parse_requests handles both; each processed independently |
| EC-006 | Partial request (incomplete header) | httparse returns Partial; buffer retained; no stats updated |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/7.0\r\n\r\n | methods["GET"]=1, hosts["example.com"]=1, user_agents["curl/7.0"]=1, uris=["/index.html"] | happy-path |
| POST /submit HTTP/1.1\r\nHost: example.com\r\n\r\n | methods["POST"]=1, hosts["example.com"]=1, no UA entry | happy-path |
| GET /partial (no trailing \r\n\r\n) | Buffer retained; no stats change | edge-case |
| GET /a HTTP/1.1\r\n...\r\n\r\nGET /b HTTP/1.1\r\n...\r\n\r\n (pipelined) | methods["GET"]=2, uris=["/a","/b"] | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Complete request yields all five fields extracted | unit: test_parse_get_request |
| — | Partial request is buffered without stats change | unit: test_parse_partial_request |
| — | Pipelined requests processed independently | unit: test_parse_pipelined_requests |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- request parsing is the core data-extraction behavior of HTTP analysis |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation) |
| Architecture Module | SS-06 (analyzer/http.rs:35-49, C-12) |
| Stories | STORY-041 |
| Origin BC | BC-HTTP-001 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.002 -- composes with (pipelined-request loop extends this parsing)
- BC-2.06.003 -- composes with (partial request buffering before this completes)
- BC-2.06.004 -- related to (transactions incremented on response, not request)
- BC-2.06.015 -- related to (parse errors transition error_count; success resets it)

## Architecture Anchors

- `src/analyzer/http.rs:35-50` -- parse_one_request function
- `src/analyzer/http.rs:359-438` -- try_parse_requests loop
- `tests/http_analyzer_tests.rs` -- test_parse_get_request, test_parse_pipelined_requests

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:35-50, 359-438` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: httparse::Request fields have defined lifetimes and types
- **assertion**: test_parse_get_request, test_parse_pipelined_requests

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates HttpAnalyzer.methods, hosts, user_agents, uris, flows |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
