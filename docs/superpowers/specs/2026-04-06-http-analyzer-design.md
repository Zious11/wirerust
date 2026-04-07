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
- Pre-allocate 96-element header arrays (covers real-world HTTP; `Status::Partial` returned if exceeded)

### Per-Flow State

`HashMap<FlowKey, HttpFlowState>` where each flow tracks:
- `request_buf: Vec<u8>` — accumulates client→server bytes until a complete request header is parsed
- `response_buf: Vec<u8>` — accumulates server→client bytes
- `pending_method: Option<String>` — last request method, to pair with response

**Buffer cap:** 64KB per direction per flow. HTTP headers exceeding 64KB are themselves anomalous and are skipped rather than buffered indefinitely.

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
      │   append to request_buf
      │   loop: httparse::Request::parse(&request_buf)
      │     Complete(n) → extract method, uri, version, headers
      │                  → store pending_method for response pairing
      │                  → check detection rules → generate findings
      │                  → update summary counters
      │                  → drain n bytes, continue loop
      │     Partial → break, wait for more data
      │
      └── direction == ServerToClient:
          append to response_buf
          loop: httparse::Response::parse(&response_buf)
            Complete(n) → extract status_code, reason, headers
                        → pair with pending_method
                        → update summary counters
                        → drain n bytes, continue loop
            Partial → break, wait for more data

  → on_flow_close(flow_key, reason)
    → remove flow state from HashMap
```

## Detection Rules

Each produces a `Finding` with the listed severity. Conservative by design — forensics tools surface indicators, analysts decide.

| Pattern | Category | Verdict | Confidence | Details |
|---------|----------|---------|------------|---------|
| Path traversal (`../`, `..%2f`, `..%252f`, `....//`) | Reconnaissance | Likely | High | URI decoded and checked |
| Web shell paths (`/shell`, `/cmd`, `/c99`, `/r57`, `/webshell`, `/backdoor`) | Execution | Likely | Medium | Substring match on URI |
| Admin panel paths (`/wp-admin`, `/admin`, `/phpmyadmin`, `/manager`) | Reconnaissance | Inconclusive | Low | Common scan targets |
| Unusual methods (CONNECT, TRACE, DELETE, OPTIONS) | Reconnaissance | Inconclusive | Medium | Rarely seen in normal traffic |
| Missing Host header (HTTP/1.1) | Anomaly | Inconclusive | Medium | Violates HTTP/1.1 spec, common in exploits |
| Long URI (> 2048 chars) | Execution | Likely | Medium | Buffer overflow / fuzzing indicator |
| Empty User-Agent | Anomaly | Inconclusive | Low | Scripts and tools often omit it |

## Summary Output

After all packets are processed, `HttpAnalyzer::summarize()` returns:

```json
{
  "analyzer_name": "HTTP",
  "packets_analyzed": 142,
  "detail": {
    "transactions": 42,
    "methods": {"GET": 35, "POST": 7},
    "status_codes": {"200": 30, "404": 5, "301": 4, "500": 3},
    "top_hosts": ["example.com", "api.target.com"],
    "top_uris": ["/index.html", "/api/v1/users", "/login"],
    "user_agents": {"Mozilla/5.0...": 35, "curl/7.68": 7}
  }
}
```

Terminal reporter displays this in the existing `ANALYZER: HTTP` section format.

## Dependencies

Add to `Cargo.toml`:
```toml
httparse = "1"
```

## Testing

**Unit tests (crafted byte streams):**
1. Parse simple GET request → verify method, URI, Host, User-Agent
2. Parse HTTP response → verify status code, reason
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
