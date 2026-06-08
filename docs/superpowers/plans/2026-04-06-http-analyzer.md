# HTTP Analyzer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stream-level HTTP/1.x analysis using the TCP reassembly engine, with httparse for parsing and detection rules for forensics indicators.

**Architecture:** `HttpAnalyzer` implements `StreamHandler` to receive reassembled TCP bytes, accumulates per-flow buffers, parses HTTP requests/responses with `httparse`, generates `Finding`s for suspicious patterns, and produces summary statistics. Auto-enables reassembly when `--http` is passed.

**Tech Stack:** httparse 1.x (zero-alloc HTTP/1.x push parser), existing TCP reassembly engine, existing `Finding`/`AnalysisSummary` types.

---

## File Structure

| File | Action | Responsibility |
|------|--------|---------------|
| `Cargo.toml` | Modify | Add `httparse = "1"` dependency |
| `src/analyzer/http.rs` | Create | HttpAnalyzer struct, StreamHandler impl, parsing, detection, summary |
| `src/analyzer/mod.rs` | Modify | Register `pub mod http;` |
| `src/main.rs` | Modify | Wire up --http flag, auto-reassembly, HttpAnalyzer as handler |
| `tests/http_analyzer_tests.rs` | Create | Unit tests for parsing, detection, summary |
| `tests/http_integration_tests.rs` | Create | Integration test with http-full.cap fixture |

---

### Task 1: HttpAnalyzer Skeleton + Dependency

**Files:**
- Modify: `Cargo.toml`
- Create: `src/analyzer/http.rs`
- Modify: `src/analyzer/mod.rs`
- Create: `tests/http_analyzer_tests.rs`

- [ ] **Step 1: Add httparse dependency**

In `Cargo.toml`, add under `[dependencies]`:

```toml
httparse = "1"
```

- [ ] **Step 2: Write a test that constructs HttpAnalyzer**

Create `tests/http_analyzer_tests.rs`:

```rust
use wirerust::analyzer::http::HttpAnalyzer;

#[test]
fn test_http_analyzer_construction() {
    let analyzer = HttpAnalyzer::new();
    assert_eq!(analyzer.transaction_count(), 0);
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cargo test --test http_analyzer_tests test_http_analyzer_construction`
Expected: FAIL — module doesn't exist.

- [ ] **Step 4: Create the HttpAnalyzer skeleton**

Create `src/analyzer/http.rs`:

```rust
use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

const MAX_HEADER_BUF: usize = 65_536;
const MAX_HEADERS: usize = 96;
const MAX_URIS: usize = 10_000;

struct HttpFlowState {
    request_buf: Vec<u8>,
    response_buf: Vec<u8>,
    pending_method: Option<String>,
}

impl HttpFlowState {
    fn new() -> Self {
        HttpFlowState {
            request_buf: Vec::new(),
            response_buf: Vec::new(),
            pending_method: None,
        }
    }
}

pub struct HttpAnalyzer {
    flows: HashMap<FlowKey, HttpFlowState>,
    methods: HashMap<String, u64>,
    status_codes: HashMap<u16, u64>,
    hosts: HashMap<String, u64>,
    user_agents: HashMap<String, u64>,
    uris: Vec<String>,
    transactions: u64,
    all_findings: Vec<Finding>,
}

impl HttpAnalyzer {
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
        }
    }

    pub fn transaction_count(&self) -> u64 {
        self.transactions
    }
}

impl StreamHandler for HttpAnalyzer {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], _offset: u64) {
        {
            let state = self.flows.entry(flow_key.clone()).or_insert_with(HttpFlowState::new);
            match direction {
                Direction::ClientToServer => {
                    let remaining = MAX_HEADER_BUF.saturating_sub(state.request_buf.len());
                    if remaining > 0 {
                        state.request_buf.extend_from_slice(&data[..data.len().min(remaining)]);
                    }
                }
                Direction::ServerToClient => {
                    let remaining = MAX_HEADER_BUF.saturating_sub(state.response_buf.len());
                    if remaining > 0 {
                        state.response_buf.extend_from_slice(&data[..data.len().min(remaining)]);
                    }
                }
            }
        }
        // Parsing will be added in Tasks 2 and 3
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, _reason: CloseReason) {
        self.flows.remove(flow_key);
    }
}

impl StreamAnalyzer for HttpAnalyzer {
    fn name(&self) -> &'static str {
        "HTTP"
    }

    fn summarize(&self) -> AnalysisSummary {
        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.transactions,
            detail: HashMap::new(), // Populated in Task 5
        }
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}
```

- [ ] **Step 5: Register the module**

In `src/analyzer/mod.rs`, add after the `pub mod dns;` line:

```rust
pub mod http;
```

- [ ] **Step 6: Run the test**

Run: `cargo test --test http_analyzer_tests`
Expected: PASS.

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml src/analyzer/http.rs src/analyzer/mod.rs tests/http_analyzer_tests.rs
git commit -m "feat: add HttpAnalyzer skeleton with StreamHandler impl"
```

---

### Task 2: HTTP Request Parsing

**Files:**
- Modify: `src/analyzer/http.rs`
- Modify: `tests/http_analyzer_tests.rs`

- [ ] **Step 1: Write test for parsing a simple GET request**

Add to `tests/http_analyzer_tests.rs`:

```rust
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};
use std::net::IpAddr;

fn test_flow_key() -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(), 49153,
        "10.0.0.2".parse::<IpAddr>().unwrap(), 80,
    )
}

#[test]
fn test_parse_get_request() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
    assert_eq!(*analyzer.host_counts().get("example.com").unwrap(), 1);
    assert_eq!(*analyzer.user_agent_counts().get("Mozilla/5.0").unwrap(), 1);
    assert_eq!(analyzer.uri_list(), &["/index.html"]);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test http_analyzer_tests test_parse_get_request`
Expected: FAIL — `method_counts` doesn't exist yet.

- [ ] **Step 3: Write test for HTTP pipelining**

Add to `tests/http_analyzer_tests.rs`:

```rust
#[test]
fn test_parse_pipelined_requests() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let pipelined = b"GET /first HTTP/1.1\r\nHost: a.com\r\n\r\nPOST /second HTTP/1.1\r\nHost: b.com\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, pipelined, 0);

    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
    assert_eq!(*analyzer.method_counts().get("POST").unwrap(), 1);
    assert_eq!(analyzer.uri_list().len(), 2);
}
```

- [ ] **Step 4: Write test for partial request across two data chunks**

Add to `tests/http_analyzer_tests.rs`:

```rust
#[test]
fn test_parse_partial_request() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send request in two parts
    analyzer.on_data(&fk, Direction::ClientToServer, b"GET /page HTTP/1.1\r\nHos", 0);
    assert_eq!(analyzer.method_counts().get("GET"), None); // Not parsed yet

    analyzer.on_data(&fk, Direction::ClientToServer, b"t: example.com\r\n\r\n", 23);
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 1);
}
```

- [ ] **Step 5: Implement request parsing**

In `src/analyzer/http.rs`, add a free function and accessor methods. Add at the top of the file:

```rust
use httparse;
```

Add this free function outside the impl blocks:

```rust
struct ParsedRequest {
    bytes_consumed: usize,
    method: String,
    uri: String,
    version: u8,
    host: Option<String>,
    user_agent: Option<String>,
}

fn parse_one_request(buf: &[u8]) -> Result<Option<ParsedRequest>, ()> {
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
        Err(_) => Err(()),
    }
}

fn find_header(headers: &[httparse::Header<'_>], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| String::from_utf8_lossy(h.value).to_string())
}
```

Add accessor methods to the `impl HttpAnalyzer` block:

```rust
pub fn method_counts(&self) -> &HashMap<String, u64> {
    &self.methods
}

pub fn host_counts(&self) -> &HashMap<String, u64> {
    &self.hosts
}

pub fn user_agent_counts(&self) -> &HashMap<String, u64> {
    &self.user_agents
}

pub fn uri_list(&self) -> &[String] {
    &self.uris
}

pub fn status_code_counts(&self) -> &HashMap<u16, u64> {
    &self.status_codes
}
```

Add a private method to parse requests from the buffer:

```rust
fn try_parse_requests(&mut self, flow_key: &FlowKey) {
    loop {
        let result = self
            .flows
            .get(flow_key)
            .filter(|s| !s.request_buf.is_empty())
            .map(|s| parse_one_request(&s.request_buf));

        match result {
            Some(Ok(Some(parsed))) => {
                *self.methods.entry(parsed.method.clone()).or_insert(0) += 1;
                if let Some(ref h) = parsed.host {
                    *self.hosts.entry(h.clone()).or_insert(0) += 1;
                }
                if let Some(ref ua) = parsed.user_agent {
                    *self.user_agents.entry(ua.clone()).or_insert(0) += 1;
                }
                if self.uris.len() < MAX_URIS {
                    self.uris.push(parsed.uri.clone());
                }

                if let Some(state) = self.flows.get_mut(flow_key) {
                    state.pending_method = Some(parsed.method);
                    state.request_buf.drain(..parsed.bytes_consumed);
                }
            }
            Some(Ok(None)) => return, // Partial — wait for more data
            Some(Err(())) => {
                // Malformed — clear buffer
                if let Some(state) = self.flows.get_mut(flow_key) {
                    state.request_buf.clear();
                }
                return;
            }
            None => return, // No flow state or empty buffer
        }
    }
}
```

Update the `on_data` callback to call parsing. Replace the `// Parsing will be added in Tasks 2 and 3` comment and the closing brace of the `ClientToServer` arm's outer block:

After the buffer append block (after the closing `}`), add:

```rust
match direction {
    Direction::ClientToServer => self.try_parse_requests(flow_key),
    Direction::ServerToClient => {} // Task 3
}
```

The full `on_data` method should now look like:

```rust
fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], _offset: u64) {
    {
        let state = self
            .flows
            .entry(flow_key.clone())
            .or_insert_with(HttpFlowState::new);
        match direction {
            Direction::ClientToServer => {
                let remaining = MAX_HEADER_BUF.saturating_sub(state.request_buf.len());
                if remaining > 0 {
                    state
                        .request_buf
                        .extend_from_slice(&data[..data.len().min(remaining)]);
                }
            }
            Direction::ServerToClient => {
                let remaining = MAX_HEADER_BUF.saturating_sub(state.response_buf.len());
                if remaining > 0 {
                    state
                        .response_buf
                        .extend_from_slice(&data[..data.len().min(remaining)]);
                }
            }
        }
    }
    match direction {
        Direction::ClientToServer => self.try_parse_requests(flow_key),
        Direction::ServerToClient => {} // Task 3
    }
}
```

- [ ] **Step 6: Run all tests**

Run: `cargo test --test http_analyzer_tests`
Expected: All 4 tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/analyzer/http.rs tests/http_analyzer_tests.rs
git commit -m "feat: parse HTTP/1.x requests from reassembled streams with httparse"
```

---

### Task 3: HTTP Response Parsing

**Files:**
- Modify: `src/analyzer/http.rs`
- Modify: `tests/http_analyzer_tests.rs`

- [ ] **Step 1: Write test for response parsing**

Add to `tests/http_analyzer_tests.rs`:

```rust
#[test]
fn test_parse_response() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Send a request first (to set pending_method)
    let request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    // Then a response
    let response = b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 5\r\n\r\nhello";
    analyzer.on_data(&fk, Direction::ServerToClient, response, 0);

    assert_eq!(*analyzer.status_code_counts().get(&200).unwrap(), 1);
    assert_eq!(analyzer.transaction_count(), 1);
}
```

- [ ] **Step 2: Write test for pipelined responses**

Add to `tests/http_analyzer_tests.rs`:

```rust
#[test]
fn test_parse_pipelined_responses() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // Two requests
    let requests = b"GET /a HTTP/1.1\r\nHost: x.com\r\n\r\nGET /b HTTP/1.1\r\nHost: x.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, requests, 0);

    // Two responses
    let responses = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\nHTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, responses, 0);

    assert_eq!(*analyzer.status_code_counts().get(&200).unwrap(), 1);
    assert_eq!(*analyzer.status_code_counts().get(&404).unwrap(), 1);
    assert_eq!(analyzer.transaction_count(), 2);
}
```

- [ ] **Step 3: Implement response parsing**

In `src/analyzer/http.rs`, add a free function:

```rust
struct ParsedResponse {
    bytes_consumed: usize,
    status_code: u16,
}

fn parse_one_response(buf: &[u8]) -> Result<Option<ParsedResponse>, ()> {
    let mut headers = [httparse::EMPTY_HEADER; MAX_HEADERS];
    let mut resp = httparse::Response::new(&mut headers);
    match resp.parse(buf) {
        Ok(httparse::Status::Complete(n)) => Ok(Some(ParsedResponse {
            bytes_consumed: n,
            status_code: resp.code.unwrap_or(0),
        })),
        Ok(httparse::Status::Partial) => Ok(None),
        Err(_) => Err(()),
    }
}
```

Add a private method for the response parsing loop:

```rust
fn try_parse_responses(&mut self, flow_key: &FlowKey) {
    loop {
        let result = self
            .flows
            .get(flow_key)
            .filter(|s| !s.response_buf.is_empty())
            .map(|s| parse_one_response(&s.response_buf));

        match result {
            Some(Ok(Some(parsed))) => {
                *self.status_codes.entry(parsed.status_code).or_insert(0) += 1;
                self.transactions += 1;

                if let Some(state) = self.flows.get_mut(flow_key) {
                    state.pending_method = None;
                    state.response_buf.drain(..parsed.bytes_consumed);
                }
            }
            Some(Ok(None)) => return,
            Some(Err(())) => {
                if let Some(state) = self.flows.get_mut(flow_key) {
                    state.response_buf.clear();
                }
                return;
            }
            None => return,
        }
    }
}
```

Update the `on_data` match for `ServerToClient`. Change:

```rust
Direction::ServerToClient => {} // Task 3
```

to:

```rust
Direction::ServerToClient => self.try_parse_responses(flow_key),
```

- [ ] **Step 4: Run all tests**

Run: `cargo test --test http_analyzer_tests`
Expected: All 6 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/analyzer/http.rs tests/http_analyzer_tests.rs
git commit -m "feat: parse HTTP/1.x responses and track request-response transactions"
```

---

### Task 4: Detection Rules

**Files:**
- Modify: `src/analyzer/http.rs`
- Modify: `tests/http_analyzer_tests.rs`

- [ ] **Step 1: Write detection tests**

Add to `tests/http_analyzer_tests.rs`:

```rust
use wirerust::findings::{Confidence, ThreatCategory, Verdict};

#[test]
fn test_detect_path_traversal() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /../../etc/passwd HTTP/1.1\r\nHost: target.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Reconnaissance);
    assert_eq!(findings[0].verdict, Verdict::Likely);
    assert_eq!(findings[0].confidence, Confidence::High);
}

#[test]
fn test_detect_encoded_traversal() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /..%2f..%2fetc/passwd HTTP/1.1\r\nHost: target.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let findings = analyzer.findings();
    assert!(!findings.is_empty(), "Should detect encoded path traversal");
}

#[test]
fn test_detect_webshell_path() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /uploads/shell.php HTTP/1.1\r\nHost: target.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Execution);
}

#[test]
fn test_detect_unusual_method() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"CONNECT proxy.example.com:443 HTTP/1.1\r\nHost: proxy.example.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, ThreatCategory::Reconnaissance);
}

#[test]
fn test_detect_missing_host_header() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    // HTTP/1.1 without Host header
    let request = b"GET /path HTTP/1.1\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let findings = analyzer.findings();
    assert!(
        findings.iter().any(|f| f.category == ThreatCategory::Anomaly),
        "Should detect missing Host header"
    );
}

#[test]
fn test_no_findings_for_normal_request() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    assert!(analyzer.findings().is_empty(), "Normal request should produce no findings");
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test http_analyzer_tests test_detect`
Expected: FAIL — findings are always empty.

- [ ] **Step 3: Implement detection rules**

In `src/analyzer/http.rs`, add the detection method to `impl HttpAnalyzer`:

```rust
fn check_request_detections(&mut self, parsed: &ParsedRequest, flow_key: &FlowKey) {
    let uri_lower = parsed.uri.to_lowercase();

    // Path traversal (including URL-encoded variants)
    if uri_lower.contains("../")
        || uri_lower.contains("..%2f")
        || uri_lower.contains("..%252f")
        || uri_lower.contains("....//")
    {
        self.all_findings.push(Finding {
            category: ThreatCategory::Reconnaissance,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: format!("Path traversal in URI: {}", truncate_uri(&parsed.uri, 120)),
            evidence: vec![format!("URI: {}", parsed.uri)],
            mitre_technique: Some("T1083".to_string()),
            source_ip: None,
            timestamp: None,
        });
    }

    // Web shell paths
    let shell_patterns = ["/shell", "/cmd", "/c99", "/r57", "/webshell", "/backdoor"];
    if shell_patterns.iter().any(|p| uri_lower.contains(p)) {
        self.all_findings.push(Finding {
            category: ThreatCategory::Execution,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: format!("Possible web shell access: {}", truncate_uri(&parsed.uri, 120)),
            evidence: vec![format!("URI: {}", parsed.uri)],
            mitre_technique: Some("T1505.003".to_string()),
            source_ip: None,
            timestamp: None,
        });
    }

    // Admin panel paths
    let admin_patterns = ["/wp-admin", "/admin", "/phpmyadmin", "/manager"];
    if admin_patterns.iter().any(|p| uri_lower.contains(p)) {
        self.all_findings.push(Finding {
            category: ThreatCategory::Reconnaissance,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: format!("Admin panel access: {}", truncate_uri(&parsed.uri, 120)),
            evidence: vec![format!("URI: {}", parsed.uri)],
            mitre_technique: Some("T1046".to_string()),
            source_ip: None,
            timestamp: None,
        });
    }

    // Unusual HTTP methods
    let unusual_methods = ["CONNECT", "TRACE", "DELETE", "OPTIONS"];
    if unusual_methods.contains(&parsed.method.as_str()) {
        self.all_findings.push(Finding {
            category: ThreatCategory::Reconnaissance,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Medium,
            summary: format!("Unusual HTTP method: {}", parsed.method),
            evidence: vec![format!("{} {}", parsed.method, parsed.uri)],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
        });
    }

    // Missing Host header (HTTP/1.1 requires it)
    if parsed.version == 1 && parsed.host.is_none() {
        self.all_findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Medium,
            summary: "HTTP/1.1 request without Host header".to_string(),
            evidence: vec![format!("{} {}", parsed.method, parsed.uri)],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
        });
    }

    // Long URI (> 2048 chars)
    if parsed.uri.len() > 2048 {
        self.all_findings.push(Finding {
            category: ThreatCategory::Execution,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: format!("Abnormally long URI ({} chars)", parsed.uri.len()),
            evidence: vec![format!("URI prefix: {}", truncate_uri(&parsed.uri, 200))],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
        });
    }

    // Empty User-Agent
    if parsed.user_agent.as_deref() == Some("") {
        self.all_findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: "Empty User-Agent header".to_string(),
            evidence: vec![format!("{} {}", parsed.method, parsed.uri)],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
        });
    }
}
```

Add the helper function outside the impl blocks:

```rust
fn truncate_uri(uri: &str, max_len: usize) -> &str {
    if uri.len() <= max_len {
        uri
    } else {
        &uri[..max_len]
    }
}
```

Now integrate detection into `try_parse_requests`. In the `Some(Ok(Some(parsed)))` arm, after the URI is pushed to `self.uris` and before the flow state update, add:

```rust
self.check_request_detections(&parsed, flow_key);
```

But wait — `parsed` is moved when we access its fields above. We need to restructure. Change the `Some(Ok(Some(parsed)))` arm to pass `&parsed` to detection before consuming fields. The full arm becomes:

```rust
Some(Ok(Some(parsed))) => {
    *self.methods.entry(parsed.method.clone()).or_insert(0) += 1;
    if let Some(ref h) = parsed.host {
        *self.hosts.entry(h.clone()).or_insert(0) += 1;
    }
    if let Some(ref ua) = parsed.user_agent {
        *self.user_agents.entry(ua.clone()).or_insert(0) += 1;
    }
    if self.uris.len() < MAX_URIS {
        self.uris.push(parsed.uri.clone());
    }

    self.check_request_detections(&parsed, flow_key);

    if let Some(state) = self.flows.get_mut(flow_key) {
        state.pending_method = Some(parsed.method);
        state.request_buf.drain(..parsed.bytes_consumed);
    }
}
```

- [ ] **Step 4: Add Finding imports to test file**

At the top of `tests/http_analyzer_tests.rs`, ensure these imports exist:

```rust
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::findings::{Confidence, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};
use std::net::IpAddr;
```

- [ ] **Step 5: Run all tests**

Run: `cargo test --test http_analyzer_tests`
Expected: All 12 tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/analyzer/http.rs tests/http_analyzer_tests.rs
git commit -m "feat: add HTTP forensics detection rules (traversal, webshell, methods, anomalies)"
```

---

### Task 5: Summary Output

**Files:**
- Modify: `src/analyzer/http.rs`
- Modify: `tests/http_analyzer_tests.rs`

- [ ] **Step 1: Write test for summary output**

Add to `tests/http_analyzer_tests.rs`:

```rust
use wirerust::reassembly::handler::{CloseReason, StreamAnalyzer};

#[test]
fn test_summarize_produces_complete_output() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET /page HTTP/1.1\r\nHost: example.com\r\nUser-Agent: TestBot\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);

    let response = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
    analyzer.on_data(&fk, Direction::ServerToClient, response, 0);

    let summary = analyzer.summarize();
    assert_eq!(summary.analyzer_name, "HTTP");
    assert_eq!(summary.packets_analyzed, 1); // 1 transaction

    let detail = &summary.detail;
    assert_eq!(detail["transactions"], 1);
    assert_eq!(detail["methods"]["GET"], 1);
    assert_eq!(detail["status_codes"]["200"], 1);
    assert!(detail["top_hosts"].as_array().unwrap().contains(&serde_json::json!("example.com")));
    assert!(detail["user_agents"]["TestBot"].as_u64().unwrap() == 1);
}

#[test]
fn test_flow_close_cleans_up_state() {
    let mut analyzer = HttpAnalyzer::new();
    let fk = test_flow_key();

    let request = b"GET / HTTP/1.1\r\nHost: x.com\r\n\r\n";
    analyzer.on_data(&fk, Direction::ClientToServer, request, 0);
    analyzer.on_flow_close(&fk, CloseReason::Fin);

    // After close, new data on same flow starts fresh
    analyzer.on_data(&fk, Direction::ClientToServer, b"GET /new HTTP/1.1\r\nHost: y.com\r\n\r\n", 0);
    assert_eq!(*analyzer.method_counts().get("GET").unwrap(), 2);
    assert_eq!(*analyzer.host_counts().get("y.com").unwrap(), 1);
}
```

- [ ] **Step 2: Implement the full summarize method**

In `src/analyzer/http.rs`, replace the `summarize` method in the `StreamAnalyzer` impl:

```rust
fn summarize(&self) -> AnalysisSummary {
    let mut detail = HashMap::new();

    detail.insert(
        "transactions".to_string(),
        serde_json::json!(self.transactions),
    );
    detail.insert("methods".to_string(), serde_json::json!(self.methods));
    detail.insert(
        "status_codes".to_string(),
        serde_json::json!(
            self.status_codes
                .iter()
                .map(|(k, v)| (k.to_string(), *v))
                .collect::<HashMap<String, u64>>()
        ),
    );

    // Top hosts (up to 20)
    let mut top_hosts: Vec<_> = self.hosts.iter().collect();
    top_hosts.sort_by(|a, b| b.1.cmp(a.1));
    let top_hosts: Vec<&str> = top_hosts.iter().take(20).map(|(k, _)| k.as_str()).collect();
    detail.insert("top_hosts".to_string(), serde_json::json!(top_hosts));

    // Top URIs (up to 20)
    let top_uris: Vec<&str> = self.uris.iter().take(20).map(|s| s.as_str()).collect();
    detail.insert("top_uris".to_string(), serde_json::json!(top_uris));

    detail.insert(
        "user_agents".to_string(),
        serde_json::json!(self.user_agents),
    );

    AnalysisSummary {
        analyzer_name: self.name().to_string(),
        packets_analyzed: self.transactions,
        detail,
    }
}
```

- [ ] **Step 3: Run all tests**

Run: `cargo test --test http_analyzer_tests`
Expected: All 14 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/analyzer/http.rs tests/http_analyzer_tests.rs
git commit -m "feat: add HTTP analyzer summary with methods, status codes, hosts, user-agents"
```

---

### Task 6: Wire Up main.rs + Integration Test

**Files:**
- Modify: `src/main.rs`
- Create: `tests/http_integration_tests.rs`

- [ ] **Step 1: Write integration test**

Create `tests/http_integration_tests.rs`:

```rust
use std::io::Cursor;

use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reassembly::handler::StreamAnalyzer;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

#[test]
fn test_http_analysis_with_fixture() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/http-full.cap")).unwrap();

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut http_analyzer = HttpAnalyzer::new();

    for raw in &source.packets {
        if let Ok(parsed) = decode_packet(&raw.data, source.datalink) {
            reassembler.process_packet(&parsed, raw.timestamp_secs, &mut http_analyzer);
        }
    }
    reassembler.finalize(&mut http_analyzer);

    let summary = http_analyzer.summarize();
    assert_eq!(summary.analyzer_name, "HTTP");

    // http-full.cap should have at least some HTTP transactions
    assert!(
        http_analyzer.transaction_count() > 0,
        "Expected HTTP transactions from http-full.cap, got 0"
    );

    // Should have at least one GET method
    assert!(
        http_analyzer.method_counts().contains_key("GET"),
        "Expected GET requests in http-full.cap"
    );
}
```

- [ ] **Step 2: Run integration test to verify it passes**

Run: `cargo test --test http_integration_tests`
Expected: PASS (HttpAnalyzer receives stream data from reassembly engine).

- [ ] **Step 3: Wire up main.rs**

In `src/main.rs`, make these changes:

**Add import** at the top (after the dns import):

```rust
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::reassembly::handler::StreamAnalyzer;
```

**Update the `Commands::Analyze` match** to extract `http`:

Change:

```rust
Commands::Analyze {
    targets, dns, all, ..
} => {
    run_analyze(targets, *dns || *all, use_color, &cli)?;
}
```

to:

```rust
Commands::Analyze {
    targets, dns, http, all, ..
} => {
    run_analyze(targets, *dns || *all, *http || *all, use_color, &cli)?;
}
```

**Update `run_analyze` signature** to accept `enable_http`:

```rust
fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    enable_http: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
```

**Update the reassembly decision** — change:

```rust
let needs_reassembly = cli.reassemble; // Will expand when HTTP/TLS analyzers added
```

to:

```rust
let needs_reassembly = cli.reassemble || enable_http;
```

**Replace NullHandler with conditional HttpAnalyzer** — replace:

```rust
struct NullHandler;
impl StreamHandler for NullHandler {
    fn on_data(&mut self, _: &FlowKey, _: Direction, _: &[u8], _: u64) {}
    fn on_flow_close(&mut self, _: &FlowKey, _: CloseReason) {}
}
let mut stream_handler = NullHandler;
```

with:

```rust
struct NullHandler;
impl StreamHandler for NullHandler {
    fn on_data(&mut self, _: &FlowKey, _: Direction, _: &[u8], _: u64) {}
    fn on_flow_close(&mut self, _: &FlowKey, _: CloseReason) {}
}
let mut null_handler = NullHandler;
let mut http_analyzer = if enable_http && !skip_reassembly {
    Some(HttpAnalyzer::new())
} else {
    None
};
```

**Update process_packet call** — change:

```rust
if let Some(ref mut reasm) = reassembler {
    reasm.process_packet(&parsed, raw.timestamp_secs, &mut stream_handler);
}
```

to:

```rust
if let Some(ref mut reasm) = reassembler {
    match http_analyzer {
        Some(ref mut http) => {
            reasm.process_packet(&parsed, raw.timestamp_secs, http);
        }
        None => {
            reasm.process_packet(&parsed, raw.timestamp_secs, &mut null_handler);
        }
    }
}
```

**Update finalize call** — change:

```rust
if let Some(ref mut reasm) = reassembler {
    reasm.finalize(&mut stream_handler);
    all_findings.extend(reasm.findings().to_vec());
}
```

to:

```rust
if let Some(ref mut reasm) = reassembler {
    match http_analyzer {
        Some(ref mut http) => {
            reasm.finalize(http);
            all_findings.extend(http.findings());
        }
        None => {
            reasm.finalize(&mut null_handler);
        }
    }
    all_findings.extend(reasm.findings().to_vec());
}
```

**Update analyzer_summaries** — change:

```rust
let analyzer_summaries = if enable_dns {
    vec![dns_analyzer.summarize()]
} else {
    vec![]
};
```

to:

```rust
let mut analyzer_summaries = Vec::new();
if enable_dns {
    analyzer_summaries.push(dns_analyzer.summarize());
}
if let Some(ref http) = http_analyzer {
    analyzer_summaries.push(http.summarize());
}
```

- [ ] **Step 4: Run full test suite**

Run: `cargo test`
Expected: All tests pass.

Run: `cargo clippy -- -D warnings`
Expected: Clean.

Run: `cargo fmt --check`
Expected: Clean (run `cargo fmt` if not).

- [ ] **Step 5: Smoke test with fixture**

Run: `cargo run -- analyze tests/fixtures/http-full.cap --http`
Expected: Should show HTTP analyzer output with transactions, methods, status codes.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs tests/http_integration_tests.rs
git commit -m "feat: wire up --http flag with auto-reassembly and HttpAnalyzer"
```
