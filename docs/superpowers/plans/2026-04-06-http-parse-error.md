# HTTP Parse Error Counter Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an HTTP parse error counter and TooManyHeaders security finding to the HTTP analyzer.

**Architecture:** Change `parse_one_request`/`parse_one_response` return types to preserve `httparse::Error`, add aggregate counter to `HttpAnalyzer`, generate a `Finding` for `TooManyHeaders`, surface counter in `summarize()`.

**Tech Stack:** Rust 2024, httparse 1.10.1, serde_json

---

### Task 1: Change Return Types and Add Counter + Accessor

**Files:**
- Modify: `src/analyzer/http.rs:22-55` (parse functions), `src/analyzer/http.rs:86-115` (struct + new)

- [ ] **Step 1: Change `parse_one_request` return type**

In `src/analyzer/http.rs`, change the function signature and error arm:

```rust
fn parse_one_request(buf: &[u8]) -> Result<Option<ParsedRequest>, httparse::Error> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut req = httparse::Request::new(&mut headers);
    match req.parse(buf) {
        Ok(httparse::Status::Complete(n)) => Ok(Some(ParsedRequest {
            bytes_consumed: n,
            method: req.method.unwrap_or("").to_string(),
            uri: req.path.unwrap_or("").to_string(),
            version: req.version.unwrap_or(1),
            host: find_header(req.headers, "host"),
            user_agent: find_header(req.headers, "user-agent"),
        })),
        Ok(httparse::Status::Partial) => Ok(None),
        Err(e) => Err(e),
    }
}
```

- [ ] **Step 2: Change `parse_one_response` return type**

Same pattern:

```rust
fn parse_one_response(buf: &[u8]) -> Result<Option<ParsedResponse>, httparse::Error> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut resp = httparse::Response::new(&mut headers);
    match resp.parse(buf) {
        Ok(httparse::Status::Complete(n)) => Ok(Some(ParsedResponse {
            bytes_consumed: n,
            status_code: resp.code.unwrap_or(0),
        })),
        Ok(httparse::Status::Partial) => Ok(None),
        Err(e) => Err(e),
    }
}
```

- [ ] **Step 3: Add `parse_errors` field to `HttpAnalyzer` struct**

Add after the `all_findings` field:

```rust
pub struct HttpAnalyzer {
    flows: HashMap<FlowKey, HttpFlowState>,
    methods: HashMap<String, u64>,
    status_codes: HashMap<u16, u64>,
    hosts: HashMap<String, u64>,
    user_agents: HashMap<String, u64>,
    uris: Vec<String>,
    transactions: u64,
    all_findings: Vec<Finding>,
    parse_errors: u64,
}
```

- [ ] **Step 4: Initialize `parse_errors` in `new()`**

Add `parse_errors: 0` to the constructor:

```rust
    pub fn new() -> Self {
        HttpAnalyzer {
            flows: HashMap::new(),
            methods: HashMap::new(),
            status_codes: HashMap::new(),
            hosts: HashMap::new(),
            user_agents: HashMap::new(),
            uris: Vec::new(),
            transactions: 0,
            all_findings: Vec::new(),
            parse_errors: 0,
        }
    }
```

- [ ] **Step 5: Add `parse_error_count()` accessor**

Add after `status_code_counts()` (around line 139):

```rust
    pub fn parse_error_count(&self) -> u64 {
        self.parse_errors
    }
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check 2>&1`

Expected: Compiler errors in `try_parse_requests` and `try_parse_responses` because the match arms still expect `Err(())` but now receive `Err(httparse::Error)`. This is expected — Task 2 will fix these.

- [ ] **Step 7: Commit**

```bash
git add src/analyzer/http.rs
git commit -m "refactor: change parse helpers to return httparse::Error, add parse_errors counter"
```

### Task 2: Update Callers with Error Handling and TooManyHeaders Finding

**Files:**
- Modify: `src/analyzer/http.rs:264-339` (try_parse_requests, try_parse_responses)

- [ ] **Step 1: Write the failing test for parse error counter**

Add to `tests/http_analyzer_tests.rs`:

```rust
#[test]
fn test_parse_error_increments_counter() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // "NOT_HTTP\r\n\r\n" triggers httparse::Error::Token
    analyzer.on_data(&fk, Direction::ClientToServer, b"NOT_HTTP\r\n\r\n", 0);

    assert_eq!(analyzer.parse_error_count(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test http_analyzer_tests test_parse_error_increments_counter 2>&1`

Expected: FAIL — `parse_error_count()` returns 0 because `try_parse_requests` doesn't increment it yet.

- [ ] **Step 3: Update `try_parse_requests` error arm**

Replace the `Some(Err(()))` arm (lines 301-306) with:

```rust
                Some(Err(e)) => {
                    self.parse_errors += 1;
                    if e == httparse::Error::TooManyHeaders {
                        self.all_findings.push(Finding {
                            category: ThreatCategory::Anomaly,
                            verdict: Verdict::Inconclusive,
                            confidence: Confidence::Medium,
                            summary: "Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)".to_string(),
                            evidence: vec!["Direction: request".to_string()],
                            mitre_technique: Some("T1499.002".to_string()),
                            source_ip: None,
                            timestamp: None,
                        });
                    }
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.request_buf.clear();
                    }
                    return;
                }
```

- [ ] **Step 4: Update `try_parse_responses` error arm**

Replace the `Some(Err(()))` arm (lines 330-335) with:

```rust
                Some(Err(e)) => {
                    self.parse_errors += 1;
                    if e == httparse::Error::TooManyHeaders {
                        self.all_findings.push(Finding {
                            category: ThreatCategory::Anomaly,
                            verdict: Verdict::Inconclusive,
                            confidence: Confidence::Medium,
                            summary: "Excessive HTTP headers exceeded parser limit (possible DoS or header-based attack)".to_string(),
                            evidence: vec!["Direction: response".to_string()],
                            mitre_technique: Some("T1499.002".to_string()),
                            source_ip: None,
                            timestamp: None,
                        });
                    }
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.response_buf.clear();
                    }
                    return;
                }
```

- [ ] **Step 5: Run the parse error test**

Run: `cargo test --test http_analyzer_tests test_parse_error_increments_counter 2>&1`

Expected: PASS

- [ ] **Step 6: Run all existing tests to verify no regressions**

Run: `cargo test --test http_analyzer_tests 2>&1`

Expected: All 15 tests pass (14 existing + 1 new).

- [ ] **Step 7: Commit**

```bash
git add src/analyzer/http.rs tests/http_analyzer_tests.rs
git commit -m "feat: increment parse_errors counter and generate TooManyHeaders finding"
```

### Task 3: Add `parse_errors` to `summarize()` Output

**Files:**
- Modify: `src/analyzer/http.rs:384-420` (summarize method)

- [ ] **Step 1: Write the failing test for summarize**

Add to `tests/http_analyzer_tests.rs`:

```rust
#[test]
fn test_parse_error_in_summarize() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    analyzer.on_data(&fk, Direction::ClientToServer, b"NOT_HTTP\r\n\r\n", 0);

    let summary = analyzer.summarize();
    assert_eq!(summary.detail["parse_errors"], 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test http_analyzer_tests test_parse_error_in_summarize 2>&1`

Expected: FAIL — `parse_errors` key not in detail map.

- [ ] **Step 3: Add `parse_errors` to `summarize()`**

In `summarize()`, add after the `user_agents` insert (around line 413):

```rust
        detail.insert(
            "parse_errors".to_string(),
            serde_json::json!(self.parse_errors),
        );
```

- [ ] **Step 4: Run the summarize test**

Run: `cargo test --test http_analyzer_tests test_parse_error_in_summarize 2>&1`

Expected: PASS

- [ ] **Step 5: Run all tests**

Run: `cargo test --test http_analyzer_tests 2>&1`

Expected: All 16 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/analyzer/http.rs tests/http_analyzer_tests.rs
git commit -m "feat: surface parse_errors in summarize() output"
```

### Task 4: Add Remaining Tests (TooManyHeaders Finding, Response Error, Buffer Recovery, No-Error Baseline)

**Files:**
- Modify: `tests/http_analyzer_tests.rs`

- [ ] **Step 1: Write `test_too_many_headers_generates_finding`**

This test programmatically builds a request with 97 headers to exceed `MAX_HEADERS=96`:

```rust
#[test]
fn test_too_many_headers_generates_finding() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Build a request with 97 headers to exceed MAX_HEADERS (96)
    let mut request = b"GET / HTTP/1.1\r\n".to_vec();
    for i in 0..97 {
        request.extend_from_slice(format!("X-Header-{i}: value\r\n").as_bytes());
    }
    request.extend_from_slice(b"\r\n");

    analyzer.on_data(&fk, Direction::ClientToServer, &request, 0);

    assert_eq!(analyzer.parse_error_count(), 1);
    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Anomaly);
    assert_eq!(findings[0].verdict, Verdict::Inconclusive);
    assert_eq!(findings[0].confidence, Confidence::Medium);
    assert_eq!(
        findings[0].mitre_technique.as_deref(),
        Some("T1499.002")
    );
    assert!(findings[0].summary.contains("Excessive HTTP headers"));
    assert!(findings[0].evidence[0].contains("request"));
}
```

- [ ] **Step 2: Write `test_parse_error_in_response`**

```rust
#[test]
fn test_parse_error_in_response() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send valid request first
    let request = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    // Send malformed response
    analyzer.on_data(&fk, Direction::ServerToClient, b"NOT_HTTP\r\n\r\n", 0);

    assert_eq!(analyzer.parse_error_count(), 1);
}
```

- [ ] **Step 3: Write `test_parse_error_clears_buffer_and_continues`**

```rust
#[test]
fn test_parse_error_clears_buffer_and_continues() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // First: malformed request (triggers error, clears buffer)
    analyzer.on_data(&fk, Direction::ClientToServer, b"GARBAGE\r\n\r\n", 0);
    assert_eq!(analyzer.parse_error_count(), 1);

    // Second: valid request (should parse successfully on fresh buffer)
    let valid = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, valid, 0);

    assert_eq!(analyzer.parse_error_count(), 1); // no new errors
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
}
```

- [ ] **Step 4: Write `test_normal_request_no_parse_errors`**

```rust
#[test]
fn test_normal_request_no_parse_errors() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request =
        b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert!(analyzer.findings().is_empty());
}
```

- [ ] **Step 5: Run all tests**

Run: `cargo test --test http_analyzer_tests 2>&1`

Expected: All 20 tests pass (14 existing + 6 new).

- [ ] **Step 6: Run clippy and fmt**

Run: `cargo clippy --tests 2>&1 && cargo fmt --check 2>&1`

Expected: No warnings, no formatting issues.

- [ ] **Step 7: Commit**

```bash
git add tests/http_analyzer_tests.rs
git commit -m "test: add parse error counter, TooManyHeaders finding, and recovery tests"
```

## Files Modified

| File | Change |
|------|--------|
| `src/analyzer/http.rs` | Return type change, counter field, error handling, summarize, accessor |
| `tests/http_analyzer_tests.rs` | 6 new tests |

## Self-Review Checklist

- [x] Spec coverage: all 5 spec items mapped to tasks (return type, counter, summarize, finding, accessor)
- [x] No placeholders: all code blocks contain complete implementation
- [x] Type consistency: `httparse::Error` used consistently, `parse_errors: u64` matches accessor return type
- [x] Test coverage: counter increment, response error, TooManyHeaders finding, buffer recovery, summarize output, no-error baseline
