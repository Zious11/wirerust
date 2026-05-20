---
artifact: L2-cap-06
traces_to: ../domain-spec.md
cap_id: CAP-06
title: HTTP Traffic Analysis
status: descriptive (brownfield) -- reconciled against develop HEAD 0082a0c
reconciled: 2026-05-20
---

# CAP-06: HTTP Traffic Analysis

## What the system does today

`HttpAnalyzer` (E-31, C-12) implements `StreamHandler + StreamAnalyzer`. It buffers
reassembled TCP data per flow direction and drives httparse to parse HTTP/1.x requests and
responses. Parsed fields trigger anomaly detections that emit `Finding` objects.

**Sources:** C-12 analyzer/http.rs (module-decomposition.md L3 Domain Layer). BC-HTTP-001..026.

## Per-flow state: HttpFlowState (E-32)

```
HttpFlowState {
    request_buf:          Vec<u8>  (max MAX_HEADER_BUF = 65,536 bytes)
    response_buf:         Vec<u8>  (max MAX_HEADER_BUF = 65,536 bytes)
    request_poisoned:     bool     (monotonic false->true; never reset within flow)
    response_poisoned:    bool     (monotonic false->true; never reset within flow)
    request_error_count:  u8       (consecutive errors; resets on success)
    response_error_count: u8       (consecutive errors; resets on success)
    counted_as_non_http:  bool     (per-flow latch; prevents double-counting non_http_flows)
}
```

`request_poisoned` and `response_poisoned` are per-direction and strictly monotonic
false->true. They never reset within a flow's lifetime (INV-8).

`request_error_count` / `response_error_count` count CONSECUTIVE errors; they reset to 0 on
a successful parse. `POISON_THRESHOLD = 3` therefore measures consecutive failures, not
cumulative.

`counted_as_non_http` is per-flow (single bool). The first direction to reach POISON_THRESHOLD
increments `non_http_flows` by 1; the second direction's poisoning does NOT increment again.
`non_http_flows` counts flows, not directions.

## Poisoning (CNV-PAT-002)

When `request_error_count >= 3`, `request_poisoned = true`. On subsequent `on_data` calls,
the poisoned direction silently absorbs all bytes (`poisoned_bytes_skipped += data.len()`)
without parsing. Buffer is cleared at poison time.

`HttpAnalyzer` conforms to CNV-PAT-002: silent drops are instrumented via `poisoned_bytes_skipped: u64`.

## Anomaly detections (current behavior post-#71)

All findings now carry `direction: Some(Direction::ClientToServer)` or
`direction: Some(Direction::ServerToClient)` as appropriate (P2.08 / #77).

| Detection | Condition | Finding | MITRE | Direction tag | Source lines |
|---|---|---|---|---|---|
| Path traversal | URI contains `../`, `..%2f`, `..%252f`, or `....//` | Reconnaissance/Likely/High | T1083 | ClientToServer | http.rs:187-203 |
| Web shell access | URI matches web-shell path patterns | Execution/Likely/Medium | T1505.003 | ClientToServer | http.rs:218-233 |
| Admin panel access | URI matches admin-path patterns | Reconnaissance/Inconclusive/Low | T1046 | ClientToServer | http.rs:237-249 |
| Unusual HTTP method | Method in CONNECT/TRACE/DELETE/OPTIONS | Reconnaissance/Inconclusive/Medium | none | ClientToServer | http.rs:253-265 |
| Missing Host (HTTP/1.1) | version==1 AND host.is_none() | Anomaly/Inconclusive/Medium | none | ClientToServer | http.rs:283-301 |
| Empty Host (HTTP/1.1) | version==1 AND host == Some("") | Anomaly/Inconclusive/Medium | none | ClientToServer | http.rs:283-301 |
| Abnormally long URI | uri.len() > 2048 | Execution/Likely/Medium | none | ClientToServer | http.rs:305-317 |
| Empty User-Agent | user_agent.as_deref() == Some("") | Anomaly/Inconclusive/Low | none | ClientToServer | http.rs:344-356 |
| Too many headers (request) | httparse::Error::TooManyHeaders on request parse | Anomaly/Inconclusive/Medium | T1499.002 | ClientToServer | http.rs:416-428 |
| Too many headers (response) | httparse::Error::TooManyHeaders on response parse | Anomaly/Inconclusive/Medium | T1499.002 | ServerToClient | http.rs:475-487 |

**Host detection (fixed by #71 / P0.05):** Both absent Host (`None`) and empty-value Host
(`Some("")`) now fire findings with distinct summary text: "HTTP/1.1 request without Host
header" vs. "HTTP/1.1 request with empty Host header". The former empty-value evasion lane
is now closed.

**UA detection (intentionally asymmetric -- domain-debt O-02):** Only `Some("")` (present-
empty) fires. Absent UA (`None`) does NOT fire. This is a documented design decision
with cited research rationale in http.rs:319-343.

## Host/UA 3-state semantics

`find_header` returns `Option<String>`:
- `None`: header absent.
- `Some("")`: header present with empty or whitespace-only value (after trim).
- `Some(non_empty)`: header present with value.

The `hosts` and `user_agents` HashMaps can accumulate `""` as a key, which serializes to
`{"": N}` in JSON output.

## Statistics tracked

`methods`, `status_codes`, `hosts`, `user_agents`: `HashMap<_, u64>` (bounded by
`MAX_MAP_ENTRIES = 50,000` per map). `uris: Vec<String>` (bounded by `MAX_URIS = 10,000`).
`transactions: u64`. `parse_errors: u64`. `non_http_flows: u64`. `poisoned_bytes_skipped: u64`.

## Findings cardinality

`all_findings: Vec<Finding>` has NO per-flow cap. Only the reassembly engine's findings vec
is capped at 10,000.

## BC references

BC-2.06.001..026. Key: BC-2.06.001..009 (detection logic), BC-2.06.010..016 (poisoning state
machine), BC-2.06.017..022 (stats map update), BC-2.06.023..026 (header overflow).
Component: C-12 (src/analyzer/http.rs) per module-decomposition.md.
