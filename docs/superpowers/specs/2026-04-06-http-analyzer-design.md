# HTTP Traffic Analyzer Design

**Issue:** #1 — Add HTTP traffic analyzer
**Scope:** Stream-level HTTP/1.x analysis using the TCP reassembly engine with `httparse`.

## Problem

wirerust has no HTTP analysis capability. The `--http` flag exists in the CLI but does nothing. HTTP is the most common application protocol in forensics captures and the first thing analysts look for during incident response.

## Approach

Stream-level analysis via the `StreamAnalyzer` trait and TCP reassembly engine, not per-packet inspection. Industry standard tools (Wireshark, Zeek, Suricata) all use TCP stream reassembly for HTTP because headers frequently span multiple TCP segments (~30-40% of real-world requests).

The `httparse` crate (v1.10.1) provides zero-allocation push parsing of HTTP/1.x requests and responses from raw bytes, with `Status::Partial`/`Status::Complete(n)` for incremental stream processing.

## Architecture

### HttpAnalyzer (`src/analyzer/http.rs`)

Implements `StreamAnalyzer` (which extends `StreamHandler`). Receives reassembled TCP byte streams from the reassembly engine and parses HTTP/1.x transactions.

**Stream callbacks:**
- `on_data(flow_key, ClientToServer, data, offset)` — append to per-flow request buffer, parse requests
- `on_data(flow_key, ServerToClient, data, offset)` — append to per-flow response buffer, parse responses
- `on_flow_close(flow_key, reason)` — clean up per-flow state

**Parsing uses `httparse`:**
- `httparse::Request::new(&mut headers).parse(&buf)` for requests
- `httparse::Response::new(&mut headers).parse(&buf)` for responses
- After `Status::Complete(n)`, drain `n` bytes and parse again (handles HTTP pipelining)
- `Status::Partial` means wait for more data
- Pre-allocate 96-element header arrays (covers real-world HTTP). If exceeded, `httparse` returns `Error::TooManyHeaders`; the analyzer treats this as a parse error, clears the direction buffer, and — unless the error is absorbed by **body-byte tolerance** (see "Per-Flow State" below) — emits an `Anomaly`/`Inconclusive` finding (possible DoS or header-based attack, MITRE T1499.002).

### Per-Flow State

`HashMap<FlowKey, HttpFlowState>` where each flow tracks:
- `request_buf: Vec<u8>` — accumulates client→server bytes until a complete request header is parsed
- `response_buf: Vec<u8>` — accumulates server→client bytes
- `request_poisoned`, `response_poisoned: bool` — once a direction trips the parse-error threshold, subsequent bytes in that direction are skipped
- `request_error_count`, `response_error_count: u8` — consecutive parse errors per direction; poisoning fires at `POISON_THRESHOLD` and resets on successful parse
- `counted_as_non_http: bool` — ensures a flow that poisons in both directions is only counted once in `non_http_flows`

**Buffer cap:** 64KB per direction per flow. HTTP headers exceeding 64KB are themselves anomalous and are skipped rather than buffered indefinitely.

Request/response pairing is not tracked — responses are counted independently. The summary's `transactions` counter increments once per successfully parsed response, so on captures with missing responses (mid-stream joins, dropped traffic) the count reflects responses only, not request/response pairs.

**Body-byte tolerance:** Once at least one message has been successfully parsed in the same `on_data` call, any parse errors encountered afterward in that same call are treated as trailing body bytes (since the analyzer does not track `Content-Length`/`Transfer-Encoding`). Such errors do not increment `parse_errors`, do not emit `TooManyHeaders` findings, and do not count toward `POISON_THRESHOLD`. This keeps normal HTTP traffic from tripping the poisoning logic.

### CLI Wiring

Auto-enable reassembly when `--http` (or `--all`) is passed. In `run_analyze()`:

```
let needs_reassembly = cli.reassemble || enable_http;  // was: cli.reassemble only
```

`NullHandler` is replaced with `HttpAnalyzer` when HTTP analysis is enabled. After processing:
- `http_analyzer.findings()` feeds into `all_findings`
- `http_analyzer.summarize()` feeds into `analyzer_summaries`

No new CLI flags needed — `--http` already exists in `Commands::Analyze`.

If `--no-reassemble` is explicitly passed alongside `--http`, reassembly is skipped and HTTP analysis is silently disabled (reassembly is required for stream-level parsing).

## Data Flow

```
Reassembly Engine
  → on_data(flow_key, direction, data, offset)
    → HttpAnalyzer
      ├── direction == ClientToServer:
      │   append to request_buf (skip if request_poisoned)
      │   loop: httparse::Request::parse(&request_buf)
      │     Complete(n) → extract method, uri, version, Host, User-Agent
      │                  → check detection rules → generate findings
      │                  → update summary counters (methods, hosts, user_agents, uris)
      │                  → drain n bytes, reset request_error_count, continue loop
      │     Partial → break, wait for more data
      │     Err(_) → clear buffer and return; unless absorbed by body-byte
      │              tolerance (see Per-Flow State): increment
      │              request_error_count; if TooManyHeaders also emit anomaly
      │              finding; poison direction once consecutive errors reach
      │              POISON_THRESHOLD
      │
      └── direction == ServerToClient:
          append to response_buf (skip if response_poisoned)
          loop: httparse::Response::parse(&response_buf)
            Complete(n) → extract status_code
                        → update summary counters (transactions, status_codes)
                        → drain n bytes, reset response_error_count, continue loop
            Partial → break, wait for more data
            Err(_) → clear buffer and return; same body-byte-tolerance-aware
                     poisoning path as requests

  → on_flow_close(flow_key, reason)
    → remove flow state from HashMap
```

## Detection Rules

Each produces a `Finding` with the listed severity. Conservative by design — forensics tools surface indicators, analysts decide.

| Pattern | Category | Verdict | Confidence | Details |
|---------|----------|---------|------------|---------|
| Path traversal (`../`, `..%2f`, `..%252f`, `....//`) | Reconnaissance | Likely | High | Case-insensitive substring match on the raw URI — single- and double-encoded variants are matched literally, URIs are not percent-decoded |
| Web shell paths (`/shell.php`, `/shell.asp`, `/shell.jsp`, `/cmd.php`, `/cmd.asp`, `/cmd.jsp`, `/c99.php`, `/r57.php`, `/webshell`, `/backdoor`) | Execution | Likely | Medium | Case-insensitive substring match on the URI |
| Admin panel paths (`/wp-admin`, `/admin`, `/phpmyadmin`, `/manager`) | Reconnaissance | Inconclusive | Low | Common scan targets |
| Unusual methods (CONNECT, TRACE, DELETE, OPTIONS) | Reconnaissance | Inconclusive | Medium | Rarely seen in normal traffic |
| Missing Host header (HTTP/1.1) | Anomaly | Inconclusive | Medium | Violates HTTP/1.1 spec, common in exploits |
| Long URI (> 2048 chars) | Execution | Likely | Medium | Buffer overflow / fuzzing indicator |
| Empty User-Agent | Anomaly | Inconclusive | Low | Fires only when the `User-Agent` header is present but its value is empty; a missing header is not currently flagged |
| Excessive HTTP headers (>96) | Anomaly | Inconclusive | Medium | Emitted from the `TooManyHeaders` parse-error path, in either direction |

## Summary Output

After all packets are processed, `HttpAnalyzer::summarize()` returns an `AnalysisSummary` shaped like the following. `packets_analyzed` is set to the same value as `detail.transactions` — the analyzer counts parsed HTTP transactions (responses), not TCP packets.

```json
{
  "analyzer_name": "HTTP",
  "packets_analyzed": 42,
  "detail": {
    "transactions": 42,
    "methods": {"GET": 35, "POST": 7},
    "status_codes": {"200": 30, "404": 5, "301": 4, "500": 3},
    "top_hosts": ["example.com", "api.target.com"],
    "recent_uris": ["/index.html", "/api/v1/users", "/login"],
    "user_agents": {"Mozilla/5.0...": 35, "curl/7.68": 7},
    "parse_errors": 0,
    "non_http_flows": 0,
    "poisoned_bytes_skipped": 0
  }
}
```

Key shape notes:
- `top_hosts` is a top-20 list sorted by count.
- `recent_uris` is the first 20 URIs seen, in arrival order (not a "top" list). Capped at `MAX_URIS` internally.
- `parse_errors` is the global total of non-suppressed parse errors across all flows — each error event increments it, regardless of whether the flow eventually poisons. (Body-byte errors, per "Body-byte tolerance" above, are suppressed and do not increment this counter.)
- `non_http_flows` counts flows where at least one direction hit `POISON_THRESHOLD` consecutive parse errors.
- `poisoned_bytes_skipped` is the cumulative size of data dropped from poisoned directions.

Terminal reporter displays this in the existing `ANALYZER: HTTP` section format.

## Dependencies

Add to `Cargo.toml`:
```toml
httparse = "1"
```

## Testing

**Unit tests (crafted byte streams):**
1. Parse simple GET request → verify method, URI, Host, User-Agent
2. Parse HTTP response → verify status code is recorded (no reason/headers extraction)
3. Pipelining: two requests in one buffer → both parsed correctly
4. Partial data: incomplete request → no crash, waits for more
5. Buffer cap: 64KB+ headers → skipped gracefully
6. Detection: URI with `../` → path traversal Finding generated
7. Detection: CONNECT method → unusual method Finding generated
8. Detection: missing Host header → Finding generated
9. Summary counters: methods, status codes, hosts all tracked

**Integration test:**
- `http-full.cap` fixture — verify HttpAnalyzer produces transactions and summary with methods/status codes

**Existing tests unaffected** — `NullHandler` path still works when `--http` is not passed.

## Not In Scope

- **HTTP/2 or HTTP/3** — different wire format, separate analyzer
- **Response body parsing** — track Content-Length for size, don't parse body content
- **File extraction/carving** — future feature (`--carve` flag like pcapper)
- **TLS decryption** — requires keylog file, separate feature
- **Chunked body decoding** — httparse detects Transfer-Encoding header; body decoding deferred
