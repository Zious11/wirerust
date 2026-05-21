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

# BC-2.06.026: Header Values Extracted via from_utf8_lossy.trim(); Raw Bytes Preserved

## Description

The `find_header` function extracts header values using `String::from_utf8_lossy(h.value).trim().to_string()`.
This means: non-UTF-8 bytes in header values are replaced with U+FFFD (lossy conversion);
leading and trailing whitespace is trimmed from the value. The resulting String is stored in
`hosts`, `user_agents`, and passed to detection logic. Critically, the RAW bytes flow from
httparse into Finding evidence fields without escaping at the analyzer layer, as required by
ADR 0003 / INV-4.

## Preconditions

1. A request is being parsed and httparse returns a complete result.
2. The `Host` or `User-Agent` header is present with a non-empty value byte slice.

## Postconditions

1. `find_header(headers, "host")` returns `Some(trimmed_lossy_string)`.
2. Non-ASCII bytes in the header value are replaced by U+FFFD in the stored String.
3. Whitespace (space, tab) is trimmed from both ends of the value.
4. The raw URI (not the header values) is stored in Finding evidence without modification
   (from `parsed.uri`, not filtered through `find_header`).

## Invariants

1. `find_header` performs case-insensitive name matching (`eq_ignore_ascii_case`).
2. `from_utf8_lossy` guarantees the result is valid UTF-8 (replacements inserted).
3. `.trim()` removes ASCII whitespace only (U+0009 tab, U+0020 space, U+000A LF, etc.).
4. ADR 0003 / INV-4: no escape function is called at parse time; raw URI bytes from
   `req.path` flow directly into detection code and eventually into Finding.evidence.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Host: example.com\r\n | hosts["example.com"] (trimmed \r\n NOT included -- httparse strips CRLF) |
| EC-002 | Host:   example.com   \r\n | hosts["example.com"] (spaces trimmed) |
| EC-003 | User-Agent: curl/7.0\x80\r\n | user_agents["curl/7.0\u{FFFD}"] (lossy replacement) |
| EC-004 | Host with tab: \texample.com | hosts["example.com"] (tab trimmed) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Host: example.com (with surrounding spaces) | hosts["example.com"] = 1 | happy-path |
| User-Agent with C1 bytes | stored with U+FFFD replacements | edge-case |
| URI with C1 bytes -> Finding.evidence | raw bytes preserved (not lossy) | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Raw bytes in URI flow through to Finding evidence | integration: test_http_finding_c1_csi_escaped_by_terminal_reporter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 |
| Capability Anchor Justification | CAP-06 ("HTTP Traffic Analysis") per capabilities.md §CAP-06 -- header value extraction and raw byte preservation are core to HTTP analysis data quality |
| L2 Domain Invariants | INV-4 (Raw-data/display-layer separation -- no escape at analyzer) |
| Architecture Module | SS-06 (analyzer/http.rs:70-75, C-12) |
| Stories | S-TBD |
| Origin BC | BC-HTTP-026 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.06.005 -- composes with (raw URI in evidence per ADR 0003)
- BC-2.06.006 -- composes with (raw URI in evidence per ADR 0003)

## Architecture Anchors

- `src/analyzer/http.rs:70-75` -- find_header function
- `tests/reporter_tests.rs` -- test_http_finding_c1_csi_escaped_by_terminal_reporter (pins raw-byte flow)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/http.rs:70-75` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **type constraint**: `String::from_utf8_lossy` -- defined behavior for non-UTF-8 input
- **assertion**: test_http_finding_c1_csi_escaped_by_terminal_reporter (pipeline test)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (no shared state) |
| **Overall classification** | pure |
