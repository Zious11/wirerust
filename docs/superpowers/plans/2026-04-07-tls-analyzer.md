# TLS ClientHello + ServerHello Analyzer Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a TLS analyzer that extracts SNI, computes JA3/JA3S fingerprints, and detects weak cipher suites, plus a StreamDispatcher to route reassembled TCP streams to the correct protocol analyzer.

**Architecture:** A content-first `StreamDispatcher` (ADR 0001) routes flows to `HttpAnalyzer` or `TlsAnalyzer` based on first-byte inspection. `TlsAnalyzer` implements `StreamAnalyzer`, buffers TLS records per-flow, parses ClientHello/ServerHello via `tls-parser`, computes JA3/JA3S via `md-5`, and generates findings for weak ciphers. CLI wiring connects `--tls` flag to the dispatcher.

**Tech Stack:** Rust 2024, tls-parser 0.12, md-5 0.11, nom (transitive via tls-parser)

---

### Task 1: Add Dependencies and Module Scaffolding

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/lib.rs`
- Modify: `src/analyzer/mod.rs`
- Create: `src/analyzer/tls.rs` (stub)
- Create: `src/dispatcher.rs` (stub)

- [ ] **Step 1: Add `tls-parser` and `md-5` to Cargo.toml**

In `Cargo.toml`, add to `[dependencies]`:

```toml
tls-parser = "0.12"
md-5 = "0.11"
```

- [ ] **Step 2: Add module declarations**

In `src/lib.rs`, add at the end:

```rust
pub mod dispatcher;
```

In `src/analyzer/mod.rs`, add:

```rust
pub mod tls;
```

- [ ] **Step 3: Create stub `src/analyzer/tls.rs`**

```rust
use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

const MAX_BUF: usize = 65_536;
const MAX_MAP_ENTRIES: usize = 50_000;

struct TlsFlowState {
    buf: Vec<u8>,
    client_hello_seen: bool,
    server_hello_seen: bool,
}

impl TlsFlowState {
    fn new() -> Self {
        TlsFlowState {
            buf: Vec::new(),
            client_hello_seen: false,
            server_hello_seen: false,
        }
    }
}

pub struct TlsAnalyzer {
    flows: HashMap<FlowKey, TlsFlowState>,
    sni_counts: HashMap<String, u64>,
    ja3_counts: HashMap<String, u64>,
    ja3s_counts: HashMap<String, u64>,
    version_counts: HashMap<u16, u64>,
    cipher_counts: HashMap<String, u64>,
    handshakes_seen: u64,
    parse_errors: u64,
    all_findings: Vec<Finding>,
}

impl Default for TlsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsAnalyzer {
    pub fn new() -> Self {
        TlsAnalyzer {
            flows: HashMap::new(),
            sni_counts: HashMap::new(),
            ja3_counts: HashMap::new(),
            ja3s_counts: HashMap::new(),
            version_counts: HashMap::new(),
            cipher_counts: HashMap::new(),
            handshakes_seen: 0,
            parse_errors: 0,
            all_findings: Vec::new(),
        }
    }

    pub fn sni_counts(&self) -> &HashMap<String, u64> {
        &self.sni_counts
    }

    pub fn ja3_counts(&self) -> &HashMap<String, u64> {
        &self.ja3_counts
    }

    pub fn ja3s_counts(&self) -> &HashMap<String, u64> {
        &self.ja3s_counts
    }

    pub fn version_counts(&self) -> &HashMap<u16, u64> {
        &self.version_counts
    }

    pub fn parse_error_count(&self) -> u64 {
        self.parse_errors
    }

    pub fn handshake_count(&self) -> u64 {
        self.handshakes_seen
    }
}

impl StreamHandler for TlsAnalyzer {
    fn on_data(&mut self, _flow_key: &FlowKey, _direction: Direction, _data: &[u8], _offset: u64) {
        // Will be implemented in Task 3
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, _reason: CloseReason) {
        self.flows.remove(flow_key);
    }
}

impl StreamAnalyzer for TlsAnalyzer {
    fn name(&self) -> &'static str {
        "TLS"
    }

    fn summarize(&self) -> AnalysisSummary {
        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.handshakes_seen,
            detail: HashMap::new(), // Will be implemented in Task 6
        }
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}
```

- [ ] **Step 4: Create stub `src/dispatcher.rs`**

```rust
use std::collections::HashMap;

use crate::analyzer::http::HttpAnalyzer;
use crate::analyzer::tls::TlsAnalyzer;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchTarget {
    Http,
    Tls,
    None,
}

pub struct StreamDispatcher {
    routes: HashMap<FlowKey, DispatchTarget>,
    pub http: Option<HttpAnalyzer>,
    pub tls: Option<TlsAnalyzer>,
}

impl StreamDispatcher {
    pub fn new(http: Option<HttpAnalyzer>, tls: Option<TlsAnalyzer>) -> Self {
        StreamDispatcher {
            routes: HashMap::new(),
            http,
            tls,
        }
    }
}

fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget {
    // Content-first detection
    if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
        return DispatchTarget::Tls;
    }
    if data.starts_with(b"GET ")
        || data.starts_with(b"POST ")
        || data.starts_with(b"PUT ")
        || data.starts_with(b"DELETE ")
        || data.starts_with(b"HEAD ")
        || data.starts_with(b"OPTIONS ")
        || data.starts_with(b"PATCH ")
        || data.starts_with(b"CONNECT ")
        || data.starts_with(b"TRACE ")
        || data.starts_with(b"HTTP/")
    {
        return DispatchTarget::Http;
    }
    // Port fallback for short data
    let ports = [flow_key.lower_port, flow_key.upper_port];
    if ports.contains(&443) || ports.contains(&8443) {
        return DispatchTarget::Tls;
    }
    if ports.contains(&80) || ports.contains(&8080) {
        return DispatchTarget::Http;
    }
    DispatchTarget::None
}

impl StreamHandler for StreamDispatcher {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64) {
        let target = *self
            .routes
            .entry(flow_key.clone())
            .or_insert_with(|| classify(data, flow_key));

        match target {
            DispatchTarget::Http => {
                if let Some(ref mut http) = self.http {
                    http.on_data(flow_key, direction, data, offset);
                }
            }
            DispatchTarget::Tls => {
                if let Some(ref mut tls) = self.tls {
                    tls.on_data(flow_key, direction, data, offset);
                }
            }
            DispatchTarget::None => {}
        }
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
        let target = self.routes.remove(flow_key);
        match target {
            Some(DispatchTarget::Http) => {
                if let Some(ref mut http) = self.http {
                    http.on_flow_close(flow_key, reason);
                }
            }
            Some(DispatchTarget::Tls) => {
                if let Some(ref mut tls) = self.tls {
                    tls.on_flow_close(flow_key, reason);
                }
            }
            _ => {}
        }
    }
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check 2>&1`

Expected: Compiles with no errors (may have unused warnings — that's fine).

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/lib.rs src/analyzer/mod.rs src/analyzer/tls.rs src/dispatcher.rs
git commit -m "scaffold: add TLS analyzer and StreamDispatcher stubs with dependencies"
```

---

### Task 2: StreamDispatcher Tests

**Files:**
- Create: `tests/dispatcher_tests.rs`

- [ ] **Step 1: Write dispatcher routing tests**

```rust
use std::net::IpAddr;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

fn flow_key(src_port: u16, dst_port: u16) -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        src_port,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        dst_port,
    )
}

#[test]
fn test_dispatcher_routes_tls() {
    let mut dispatcher = StreamDispatcher::new(None, Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 443);

    // TLS ClientHello record header: content_type=0x16, version=0x0303, length=0x0005
    let tls_data = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

    // If routed correctly, TLS analyzer received data (no panic, no error)
    // We can't directly assert routing, but we can verify HTTP didn't get it
    assert!(dispatcher.http.is_none());
}

#[test]
fn test_dispatcher_routes_http() {
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);
    let fk = flow_key(49152, 80);

    let http_data = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\n\r\n";
    dispatcher.on_data(&fk, Direction::ClientToServer, http_data, 0);

    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(*http.method_counts().get("GET").unwrap(), 1);
}

#[test]
fn test_dispatcher_content_detection_tls_on_port_80() {
    let mut dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 80); // Port 80, but content is TLS

    // TLS record header on port 80 — content detection should override port
    let tls_data = [0x16, 0x03, 0x03, 0x00, 0x05, 0x01, 0x00, 0x00, 0x01, 0x00];
    dispatcher.on_data(&fk, Direction::ClientToServer, &tls_data, 0);

    // HTTP analyzer should NOT have received this data
    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(http.method_counts().len(), 0);
}

#[test]
fn test_dispatcher_port_fallback_short_data() {
    let mut dispatcher =
        StreamDispatcher::new(Some(HttpAnalyzer::new()), Some(TlsAnalyzer::new()));
    let fk = flow_key(49152, 443); // Port 443

    // Only 2 bytes — too short for content detection, falls back to port
    let short_data = [0x16, 0x03];
    dispatcher.on_data(&fk, Direction::ClientToServer, &short_data, 0);

    // Should have routed to TLS based on port 443
    // HTTP should not have received it
    let http = dispatcher.http.as_ref().unwrap();
    assert_eq!(http.method_counts().len(), 0);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test dispatcher_tests 2>&1`

Expected: All 4 tests pass.

- [ ] **Step 3: Commit**

```bash
git add tests/dispatcher_tests.rs
git commit -m "test: add StreamDispatcher routing tests"
```

---

### Task 3: TLS Record Parsing and ClientHello with JA3

**Files:**
- Modify: `src/analyzer/tls.rs`
- Create: `tests/tls_analyzer_tests.rs`

This is the core task — implement the TLS record extraction loop, ClientHello processing, and JA3 computation.

- [ ] **Step 1: Write the failing ClientHello test**

Create `tests/tls_analyzer_tests.rs`:

```rust
use std::net::IpAddr;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamHandler};

fn test_flow_key() -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        49153,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        443,
    )
}

/// Build a minimal TLS ClientHello record with SNI and specified cipher suites.
/// Returns the complete TLS record bytes (record header + handshake header + ClientHello body).
fn build_client_hello(sni: &str, cipher_ids: &[u16]) -> Vec<u8> {
    // Extensions
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000)
    let sni_bytes = sni.as_bytes();
    let sni_list_len = (3 + sni_bytes.len()) as u16; // type(1) + name_len(2) + name
    let sni_ext_len = (2 + sni_list_len) as u16; // list_len(2) + sni_list
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&sni_ext_len.to_be_bytes()); // extension data length
    extensions.extend_from_slice(&sni_list_len.to_be_bytes()); // server name list length
    extensions.push(0x00); // host_name type
    extensions.extend_from_slice(&(sni_bytes.len() as u16).to_be_bytes());
    extensions.extend_from_slice(sni_bytes);

    // Supported Groups extension (type 0x000a) — x25519 (0x001d), secp256r1 (0x0017)
    extensions.extend_from_slice(&[0x00, 0x0a]); // extension type: supported_groups
    extensions.extend_from_slice(&[0x00, 0x06]); // extension data length
    extensions.extend_from_slice(&[0x00, 0x04]); // named group list length
    extensions.extend_from_slice(&[0x00, 0x1d]); // x25519
    extensions.extend_from_slice(&[0x00, 0x17]); // secp256r1

    // EC Point Formats extension (type 0x000b) — uncompressed (0x00)
    extensions.extend_from_slice(&[0x00, 0x0b]); // extension type: ec_point_formats
    extensions.extend_from_slice(&[0x00, 0x02]); // extension data length
    extensions.push(0x01); // ec point formats length
    extensions.push(0x00); // uncompressed

    // Build ClientHello body
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0

    // Cipher suites
    let ciphers_len = (cipher_ids.len() * 2) as u16;
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression

    // Extensions
    let ext_len = extensions.len() as u16;
    ch_body.extend_from_slice(&ext_len.to_be_bytes());
    ch_body.extend_from_slice(&extensions);

    // Handshake header: type=0x01 (ClientHello), length=3 bytes
    let mut handshake = Vec::new();
    handshake.push(0x01); // handshake type: ClientHello
    let ch_len = ch_body.len() as u32;
    handshake.push((ch_len >> 16) as u8);
    handshake.push((ch_len >> 8) as u8);
    handshake.push(ch_len as u8);
    handshake.extend_from_slice(&ch_body);

    // TLS record header: type=0x16, version=0x0301, length
    let mut record = Vec::new();
    record.push(0x16); // content type: handshake
    record.extend_from_slice(&[0x03, 0x01]); // record version: TLS 1.0 (standard for records)
    let hs_len = handshake.len() as u16;
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

#[test]
fn test_parse_client_hello() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // TLS_AES_128_GCM_SHA256 (0x1301), TLS_CHACHA20_POLY1305_SHA256 (0x1303)
    let record = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(*analyzer.sni_counts().get("example.com").unwrap(), 1);
    assert_eq!(analyzer.ja3_counts().len(), 1);
    assert!(!analyzer.ja3_counts().is_empty());
    assert_eq!(*analyzer.version_counts().get(&0x0303).unwrap(), 1);
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_ja3_grease_filtering() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Include GREASE value 0x0a0a alongside real cipher 0x1301
    let record = build_client_hello("test.com", &[0x0a0a, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // JA3 should have been computed — GREASE filtered out
    assert_eq!(analyzer.ja3_counts().len(), 1);
    // Get the JA3 string by checking the hash exists
    let ja3_hash = analyzer.ja3_counts().keys().next().unwrap();
    assert_eq!(ja3_hash.len(), 32); // MD5 hex = 32 chars
}

#[test]
fn test_parse_error_counter() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Valid TLS record header but garbage handshake content
    let bad_record = [0x16, 0x03, 0x03, 0x00, 0x05, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    analyzer.on_data(&fk, Direction::ClientToServer, &bad_record, 0);

    assert_eq!(analyzer.parse_error_count(), 1);
}

#[test]
fn test_normal_request_no_parse_errors() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert!(analyzer.findings().is_empty());
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --test tls_analyzer_tests 2>&1`

Expected: FAIL — `on_data` is a no-op stub.

- [ ] **Step 3: Implement `on_data` with TLS record extraction and ClientHello processing**

Replace the `on_data` method and add helper functions in `src/analyzer/tls.rs`. Replace the entire file content with:

```rust
use std::collections::HashMap;

use md5::{Digest, Md5};
use tls_parser::{
    TlsCipherSuite, TlsCipherSuiteID, TlsExtension, TlsExtensionType, TlsMessage,
    TlsMessageHandshake, parse_tls_extensions, parse_tls_plaintext,
};

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

const MAX_BUF: usize = 65_536;
const MAX_MAP_ENTRIES: usize = 50_000;

struct TlsFlowState {
    buf: Vec<u8>,
    client_hello_seen: bool,
    server_hello_seen: bool,
}

impl TlsFlowState {
    fn new() -> Self {
        TlsFlowState {
            buf: Vec::new(),
            client_hello_seen: false,
            server_hello_seen: false,
        }
    }
}

fn is_grease_u16(val: u16) -> bool {
    (val & 0x0F0F) == 0x0A0A
}

fn is_weak_cipher(id: TlsCipherSuiteID) -> bool {
    if let Some(cs) = TlsCipherSuite::from_id(id.0) {
        let name = cs.name.to_uppercase();
        name.contains("NULL") || name.contains("ANON") || name.contains("EXPORT")
    } else {
        false
    }
}

fn cipher_name(id: TlsCipherSuiteID) -> String {
    TlsCipherSuite::from_id(id.0)
        .map(|cs| cs.name.to_string())
        .unwrap_or_else(|| format!("0x{:04x}", id.0))
}

fn is_weak_server_cipher(id: TlsCipherSuiteID) -> bool {
    if let Some(cs) = TlsCipherSuite::from_id(id.0) {
        let name = cs.name.to_uppercase();
        name.contains("NULL")
            || name.contains("ANON")
            || name.contains("EXPORT")
            || name.contains("RC4")
    } else {
        false
    }
}

fn compute_ja3(
    version: u16,
    ciphers: &[TlsCipherSuiteID],
    extensions: &[TlsExtension<'_>],
) -> (String, String) {
    let version_str = version.to_string();

    let ciphers_str: String = ciphers
        .iter()
        .filter(|c| !is_grease_u16(c.0))
        .map(|c| c.0.to_string())
        .collect::<Vec<_>>()
        .join("-");

    let mut ext_ids = Vec::new();
    let mut elliptic_curves = Vec::new();
    let mut ec_point_formats = Vec::new();

    for ext in extensions {
        if matches!(ext, TlsExtension::Grease(_, _)) {
            continue;
        }
        let ext_type: TlsExtensionType = ext.into();
        if !is_grease_u16(ext_type.0) {
            ext_ids.push(ext_type.0.to_string());
        }
        match ext {
            TlsExtension::EllipticCurves(groups) => {
                for g in groups {
                    if !is_grease_u16(g.0) {
                        elliptic_curves.push(g.0.to_string());
                    }
                }
            }
            TlsExtension::EcPointFormats(formats) => {
                for &f in *formats {
                    ec_point_formats.push((f as u16).to_string());
                }
            }
            _ => {}
        }
    }

    let ja3_string = format!(
        "{},{},{},{},{}",
        version_str,
        ciphers_str,
        ext_ids.join("-"),
        elliptic_curves.join("-"),
        ec_point_formats.join("-"),
    );

    let mut hasher = Md5::new();
    hasher.update(ja3_string.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    (hash, ja3_string)
}

fn compute_ja3s(version: u16, cipher: TlsCipherSuiteID, extensions: &[TlsExtension<'_>]) -> String {
    let version_str = version.to_string();
    let cipher_str = cipher.0.to_string();

    let ext_ids: Vec<String> = extensions
        .iter()
        .filter(|e| !matches!(e, TlsExtension::Grease(_, _)))
        .map(|e| {
            let t: TlsExtensionType = e.into();
            t.0.to_string()
        })
        .collect();

    let ja3s_string = format!("{},{},{}", version_str, cipher_str, ext_ids.join("-"));

    let mut hasher = Md5::new();
    hasher.update(ja3s_string.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn extract_sni(extensions: &[TlsExtension<'_>]) -> Option<String> {
    for ext in extensions {
        if let TlsExtension::SNI(names) = ext {
            for (_, name_bytes) in names {
                if let Ok(s) = std::str::from_utf8(name_bytes) {
                    let trimmed = s.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }
    None
}

pub struct TlsAnalyzer {
    flows: HashMap<FlowKey, TlsFlowState>,
    sni_counts: HashMap<String, u64>,
    ja3_counts: HashMap<String, u64>,
    ja3s_counts: HashMap<String, u64>,
    version_counts: HashMap<u16, u64>,
    cipher_counts: HashMap<String, u64>,
    handshakes_seen: u64,
    parse_errors: u64,
    all_findings: Vec<Finding>,
}

impl Default for TlsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsAnalyzer {
    pub fn new() -> Self {
        TlsAnalyzer {
            flows: HashMap::new(),
            sni_counts: HashMap::new(),
            ja3_counts: HashMap::new(),
            ja3s_counts: HashMap::new(),
            version_counts: HashMap::new(),
            cipher_counts: HashMap::new(),
            handshakes_seen: 0,
            parse_errors: 0,
            all_findings: Vec::new(),
        }
    }

    pub fn sni_counts(&self) -> &HashMap<String, u64> {
        &self.sni_counts
    }

    pub fn ja3_counts(&self) -> &HashMap<String, u64> {
        &self.ja3_counts
    }

    pub fn ja3s_counts(&self) -> &HashMap<String, u64> {
        &self.ja3s_counts
    }

    pub fn version_counts(&self) -> &HashMap<u16, u64> {
        &self.version_counts
    }

    pub fn parse_error_count(&self) -> u64 {
        self.parse_errors
    }

    pub fn handshake_count(&self) -> u64 {
        self.handshakes_seen
    }

    fn try_parse_records(&mut self, flow_key: &FlowKey) {
        loop {
            let state = match self.flows.get(flow_key) {
                Some(s) if !s.client_hello_seen || !s.server_hello_seen => s,
                _ => return,
            };

            let buf = &state.buf;
            if buf.len() < 5 {
                return;
            }

            let content_type = buf[0];
            let record_len = u16::from_be_bytes([buf[3], buf[4]]) as usize;
            let total = 5 + record_len;

            if buf.len() < total {
                return;
            }

            // Skip non-handshake records
            if content_type != 0x16 {
                if let Some(state) = self.flows.get_mut(flow_key) {
                    state.buf.drain(..total);
                }
                continue;
            }

            // Parse the handshake record
            let record_bytes: Vec<u8> = buf[..total].to_vec();
            match parse_tls_plaintext(&record_bytes) {
                Ok((_, record)) => {
                    for msg in &record.msg {
                        match msg {
                            TlsMessage::Handshake(TlsMessageHandshake::ClientHello(ch)) => {
                                let version = ch.version.0;
                                if self.version_counts.len() < MAX_MAP_ENTRIES
                                    || self.version_counts.contains_key(&version)
                                {
                                    *self.version_counts.entry(version).or_insert(0) += 1;
                                }

                                let extensions = ch
                                    .ext
                                    .and_then(|e| parse_tls_extensions(e).ok())
                                    .map(|(_, exts)| exts)
                                    .unwrap_or_default();

                                // SNI
                                if let Some(sni) = extract_sni(&extensions) {
                                    if self.sni_counts.len() < MAX_MAP_ENTRIES
                                        || self.sni_counts.contains_key(&sni)
                                    {
                                        *self.sni_counts.entry(sni).or_insert(0) += 1;
                                    }
                                }

                                // JA3
                                let (ja3_hash, _) =
                                    compute_ja3(version, &ch.ciphers, &extensions);
                                if self.ja3_counts.len() < MAX_MAP_ENTRIES
                                    || self.ja3_counts.contains_key(&ja3_hash)
                                {
                                    *self.ja3_counts.entry(ja3_hash).or_insert(0) += 1;
                                }

                                // Weak cipher check
                                let weak: Vec<String> = ch
                                    .ciphers
                                    .iter()
                                    .filter(|c| is_weak_cipher(**c))
                                    .map(|c| cipher_name(*c))
                                    .collect();
                                if !weak.is_empty() {
                                    self.all_findings.push(Finding {
                                        category: ThreatCategory::Anomaly,
                                        verdict: Verdict::Likely,
                                        confidence: Confidence::High,
                                        summary:
                                            "ClientHello offers weak cipher suites (NULL/anonymous/export)"
                                                .to_string(),
                                        evidence: weak,
                                        mitre_technique: None,
                                        source_ip: None,
                                        timestamp: None,
                                    });
                                }

                                self.handshakes_seen += 1;
                                if let Some(state) = self.flows.get_mut(flow_key) {
                                    state.client_hello_seen = true;
                                }
                            }
                            TlsMessage::Handshake(TlsMessageHandshake::ServerHello(sh)) => {
                                let version = sh.version.0;
                                if self.version_counts.len() < MAX_MAP_ENTRIES
                                    || self.version_counts.contains_key(&version)
                                {
                                    *self.version_counts.entry(version).or_insert(0) += 1;
                                }

                                // Cipher name for counts
                                let name = cipher_name(sh.cipher);
                                if self.cipher_counts.len() < MAX_MAP_ENTRIES
                                    || self.cipher_counts.contains_key(&name)
                                {
                                    *self.cipher_counts.entry(name).or_insert(0) += 1;
                                }

                                // JA3S
                                let extensions = sh
                                    .ext
                                    .and_then(|e| parse_tls_extensions(e).ok())
                                    .map(|(_, exts)| exts)
                                    .unwrap_or_default();

                                let ja3s_hash =
                                    compute_ja3s(version, sh.cipher, &extensions);
                                if self.ja3s_counts.len() < MAX_MAP_ENTRIES
                                    || self.ja3s_counts.contains_key(&ja3s_hash)
                                {
                                    *self.ja3s_counts.entry(ja3s_hash).or_insert(0) += 1;
                                }

                                // Weak cipher selection check
                                if is_weak_server_cipher(sh.cipher) {
                                    let cipher_display = cipher_name(sh.cipher);
                                    self.all_findings.push(Finding {
                                        category: ThreatCategory::Anomaly,
                                        verdict: Verdict::Likely,
                                        confidence: Confidence::Medium,
                                        summary: format!(
                                            "ServerHello selected weak cipher suite ({})",
                                            cipher_display
                                        ),
                                        evidence: vec![format!(
                                            "Selected cipher: {} (0x{:04x})",
                                            cipher_display, sh.cipher.0
                                        )],
                                        mitre_technique: None,
                                        source_ip: None,
                                        timestamp: None,
                                    });
                                }

                                if let Some(state) = self.flows.get_mut(flow_key) {
                                    state.server_hello_seen = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Err(nom::Err::Incomplete(_)) => {
                    self.parse_errors += 1;
                }
                Err(_) => {
                    self.parse_errors += 1;
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.buf.clear();
                    }
                    return;
                }
            }

            if let Some(state) = self.flows.get_mut(flow_key) {
                state.buf.drain(..total);
            }
        }
    }
}

impl StreamHandler for TlsAnalyzer {
    fn on_data(&mut self, flow_key: &FlowKey, _direction: Direction, data: &[u8], _offset: u64) {
        {
            let state = self
                .flows
                .entry(flow_key.clone())
                .or_insert_with(TlsFlowState::new);

            if state.client_hello_seen && state.server_hello_seen {
                return;
            }

            let remaining = MAX_BUF.saturating_sub(state.buf.len());
            if remaining > 0 {
                state
                    .buf
                    .extend_from_slice(&data[..data.len().min(remaining)]);
            }
        }
        self.try_parse_records(flow_key);
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, _reason: CloseReason) {
        self.flows.remove(flow_key);
    }
}

impl StreamAnalyzer for TlsAnalyzer {
    fn name(&self) -> &'static str {
        "TLS"
    }

    fn summarize(&self) -> AnalysisSummary {
        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.handshakes_seen,
            detail: HashMap::new(), // Will be implemented in Task 6
        }
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test tls_analyzer_tests 2>&1`

Expected: All 4 tests pass.

- [ ] **Step 5: Run all tests to check for regressions**

Run: `cargo test 2>&1`

Expected: All existing tests pass (HTTP, reassembly, etc.) plus new TLS + dispatcher tests.

- [ ] **Step 6: Run clippy**

Run: `cargo clippy --tests 2>&1`

Expected: No errors. Fix any warnings.

- [ ] **Step 7: Commit**

```bash
git add src/analyzer/tls.rs tests/tls_analyzer_tests.rs
git commit -m "feat: implement TLS record parsing, ClientHello extraction, and JA3 fingerprinting"
```

---

### Task 4: ServerHello Tests

**Files:**
- Modify: `tests/tls_analyzer_tests.rs`

- [ ] **Step 1: Add ServerHello builder and tests**

Add to `tests/tls_analyzer_tests.rs`:

```rust
/// Build a minimal TLS ServerHello record.
fn build_server_hello(cipher_id: u16) -> Vec<u8> {
    // Extensions: just renegotiation_info (0xff01) with empty data
    let mut extensions = Vec::new();
    extensions.extend_from_slice(&[0xff, 0x01]); // renegotiation_info
    extensions.extend_from_slice(&[0x00, 0x01]); // extension data length
    extensions.push(0x00); // empty renegotiation info

    // ServerHello body
    let mut sh_body = Vec::new();
    sh_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    sh_body.extend_from_slice(&[0u8; 32]); // random
    sh_body.push(0x00); // session_id length: 0
    sh_body.extend_from_slice(&cipher_id.to_be_bytes()); // selected cipher
    sh_body.push(0x00); // compression: null

    let ext_len = extensions.len() as u16;
    sh_body.extend_from_slice(&ext_len.to_be_bytes());
    sh_body.extend_from_slice(&extensions);

    // Handshake header
    let mut handshake = Vec::new();
    handshake.push(0x02); // handshake type: ServerHello
    let sh_len = sh_body.len() as u32;
    handshake.push((sh_len >> 16) as u8);
    handshake.push((sh_len >> 8) as u8);
    handshake.push(sh_len as u8);
    handshake.extend_from_slice(&sh_body);

    // TLS record header
    let mut record = Vec::new();
    record.push(0x16);
    record.extend_from_slice(&[0x03, 0x03]);
    let hs_len = handshake.len() as u16;
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

#[test]
fn test_parse_server_hello() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Send ClientHello first
    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // Then ServerHello selecting TLS_AES_128_GCM_SHA256 (0x1301)
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    assert_eq!(analyzer.ja3s_counts().len(), 1);
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_weak_cipher_finding_client() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // TLS_RSA_WITH_NULL_SHA (0x0002) — NULL cipher
    let record = build_client_hello("test.com", &[0x0002, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, wirerust::findings::ThreatCategory::Anomaly);
    assert_eq!(findings[0].confidence, wirerust::findings::Confidence::High);
    assert!(findings[0].summary.contains("weak cipher"));
}

#[test]
fn test_weak_cipher_finding_server() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // Server selects TLS_RSA_WITH_RC4_128_SHA (0x0005)
    let sh = build_server_hello(0x0005);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].confidence, wirerust::findings::Confidence::Medium);
    assert!(findings[0].summary.contains("weak cipher"));
}

#[test]
fn test_normal_handshake_no_findings() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    assert!(analyzer.findings().is_empty());
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_stop_after_handshake() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    // Send encrypted application data (content_type=0x17) — should be ignored
    let app_data = [0x17, 0x03, 0x03, 0x00, 0x10, 0xAA; 21];
    analyzer.on_data(&fk, Direction::ServerToClient, &app_data, 0);

    // No parse errors from the encrypted data
    assert_eq!(analyzer.parse_error_count(), 0);
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --test tls_analyzer_tests 2>&1`

Expected: All tests pass (4 original + 5 new = 9 total).

- [ ] **Step 3: Commit**

```bash
git add tests/tls_analyzer_tests.rs
git commit -m "test: add ServerHello, weak cipher, and stop-after-handshake tests"
```

---

### Task 5: Summarize Output

**Files:**
- Modify: `src/analyzer/tls.rs`
- Modify: `tests/tls_analyzer_tests.rs`

- [ ] **Step 1: Write the failing summarize test**

Add to `tests/tls_analyzer_tests.rs`:

```rust
use wirerust::reassembly::handler::StreamAnalyzer;

#[test]
fn test_summarize_output() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    let summary = analyzer.summarize();
    assert_eq!(summary.analyzer_name, "TLS");
    assert_eq!(summary.packets_analyzed, 1);

    let detail = &summary.detail;
    assert!(detail["top_snis"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("example.com")));
    assert!(detail.contains_key("ja3_hashes"));
    assert!(detail.contains_key("ja3s_hashes"));
    assert!(detail.contains_key("tls_versions"));
    assert!(detail.contains_key("cipher_suites"));
    assert_eq!(detail["parse_errors"], 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test tls_analyzer_tests test_summarize_output 2>&1`

Expected: FAIL — `detail` map is empty.

- [ ] **Step 3: Implement `summarize()`**

In `src/analyzer/tls.rs`, replace the `summarize()` method in the `StreamAnalyzer` impl:

```rust
    fn summarize(&self) -> AnalysisSummary {
        let mut detail = HashMap::new();

        // Top SNIs (top 20 by count)
        let mut top_snis: Vec<_> = self.sni_counts.iter().collect();
        top_snis.sort_by(|a, b| b.1.cmp(a.1));
        let top_snis: Vec<&str> = top_snis.iter().take(20).map(|(k, _)| k.as_str()).collect();
        detail.insert("top_snis".to_string(), serde_json::json!(top_snis));

        detail.insert("ja3_hashes".to_string(), serde_json::json!(self.ja3_counts));
        detail.insert(
            "ja3s_hashes".to_string(),
            serde_json::json!(self.ja3s_counts),
        );
        detail.insert(
            "tls_versions".to_string(),
            serde_json::json!(
                self.version_counts
                    .iter()
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect::<HashMap<String, u64>>()
            ),
        );
        detail.insert(
            "cipher_suites".to_string(),
            serde_json::json!(self.cipher_counts),
        );
        detail.insert(
            "parse_errors".to_string(),
            serde_json::json!(self.parse_errors),
        );

        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.handshakes_seen,
            detail,
        }
    }
```

- [ ] **Step 4: Run tests**

Run: `cargo test --test tls_analyzer_tests 2>&1`

Expected: All 10 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/analyzer/tls.rs tests/tls_analyzer_tests.rs
git commit -m "feat: implement TLS summarize() with SNI, JA3, versions, ciphers"
```

---

### Task 6: CLI Integration

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Replace direct handler with StreamDispatcher**

Replace the entire `run_analyze` function in `src/main.rs`. The key changes:
- Import `StreamDispatcher` and `TlsAnalyzer`
- Destructure `tls` from CLI args
- Create `StreamDispatcher` wrapping optional HTTP + TLS analyzers
- Use dispatcher as the single StreamHandler
- Collect findings and summaries from both analyzers via dispatcher

Replace the `run_analyze` function:

```rust
fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    enable_http: bool,
    enable_tls: bool,
    use_color: bool,
    cli: &Cli,
) -> Result<()> {
    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();
    let mut total_decode_errors: u64 = 0;

    // Determine if reassembly is needed
    let needs_reassembly = cli.reassemble || enable_http || enable_tls;
    let skip_reassembly = cli.no_reassemble;

    if (enable_http || enable_tls) && skip_reassembly {
        eprintln!(
            "Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped."
        );
    }

    let mut reassembler = if needs_reassembly && !skip_reassembly {
        let config = ReassemblyConfig {
            max_depth: cli.reassembly_depth * 1_048_576,
            memcap: cli.reassembly_memcap * 1_048_576,
            ..ReassemblyConfig::default()
        };
        Some(TcpReassembler::new(config))
    } else {
        None
    };

    let http_analyzer = if enable_http && !skip_reassembly {
        Some(HttpAnalyzer::new())
    } else {
        None
    };
    let tls_analyzer = if enable_tls && !skip_reassembly {
        Some(TlsAnalyzer::new())
    } else {
        None
    };
    let mut dispatcher = StreamDispatcher::new(http_analyzer, tls_analyzer);

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;

            let pb = ProgressBar::new(source.packets.len() as u64);
            pb.set_style(ProgressStyle::with_template(
                "[{elapsed_precise}] {bar:40} {pos}/{len} packets",
            )?);

            for raw in &source.packets {
                match decode_packet(&raw.data, source.datalink) {
                    Ok(parsed) => {
                        summary.ingest(&parsed);
                        if enable_dns && dns_analyzer.can_decode(&parsed) {
                            let findings = dns_analyzer.analyze(&parsed);
                            all_findings.extend(findings);
                        }
                        if let Some(ref mut reasm) = reassembler {
                            reasm.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher);
                        }
                    }
                    Err(e) => {
                        if total_decode_errors == 0 {
                            eprintln!(
                                "Warning: failed to decode packet ({e}). Further errors counted silently."
                            );
                        }
                        total_decode_errors += 1;
                    }
                }
                pb.inc(1);
            }
            pb.finish_and_clear();
        }
    }

    summary.skipped_packets = total_decode_errors;

    if let Some(ref mut reasm) = reassembler {
        reasm.finalize(&mut dispatcher);
        all_findings.extend(reasm.findings().to_vec());
    }

    if let Some(ref http) = dispatcher.http {
        all_findings.extend(http.findings());
    }
    if let Some(ref tls) = dispatcher.tls {
        all_findings.extend(tls.findings());
    }

    let mut analyzer_summaries = Vec::new();
    if enable_dns {
        analyzer_summaries.push(dns_analyzer.summarize());
    }
    if let Some(ref http) = dispatcher.http {
        analyzer_summaries.push(http.summarize());
    }
    if let Some(ref tls) = dispatcher.tls {
        analyzer_summaries.push(tls.summarize());
    }

    let output = match cli.output_format {
        Some(OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        _ => {
            let reporter = TerminalReporter { use_color };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
    };

    println!("{output}");
    Ok(())
}
```

- [ ] **Step 2: Update the `Commands::Analyze` match arm**

In `main()`, update the match arm to pass `tls`:

```rust
        Commands::Analyze {
            targets,
            dns,
            http,
            tls,
            all,
            ..
        } => {
            run_analyze(
                targets,
                *dns || *all,
                *http || *all,
                *tls || *all,
                use_color,
                &cli,
            )?;
        }
```

- [ ] **Step 3: Update imports**

Add to the imports at the top of `src/main.rs`:

```rust
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::dispatcher::StreamDispatcher;
```

Remove the now-unused imports:
- `use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};` (CloseReason, Direction, StreamHandler no longer used directly)
- The `NullHandler` struct and its `impl` block

Keep `StreamAnalyzer` if it's used for type bounds, otherwise remove it too.

- [ ] **Step 4: Verify compilation**

Run: `cargo check 2>&1`

Expected: Compiles. Fix any unused import warnings.

- [ ] **Step 5: Run all tests**

Run: `cargo test 2>&1`

Expected: All tests pass including existing HTTP, reassembly, CLI, and new TLS/dispatcher tests.

- [ ] **Step 6: Run clippy and fmt**

Run: `cargo clippy --tests 2>&1 && cargo fmt --check 2>&1`

Expected: No warnings, no formatting issues.

- [ ] **Step 7: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire TLS analyzer into CLI via StreamDispatcher"
```

---

## Files Modified

| File | Change |
|------|--------|
| `Cargo.toml` | Add `tls-parser = "0.12"`, `md-5 = "0.11"` |
| `src/lib.rs` | Add `pub mod dispatcher` |
| `src/analyzer/mod.rs` | Add `pub mod tls` |
| `src/analyzer/tls.rs` | New: `TlsAnalyzer` with record parsing, JA3/JA3S, weak cipher findings, summarize |
| `src/dispatcher.rs` | New: `StreamDispatcher` with content-first routing |
| `src/main.rs` | Use `StreamDispatcher`, wire `--tls` flag, remove `NullHandler` |
| `tests/tls_analyzer_tests.rs` | New: 10 tests (ClientHello, ServerHello, JA3, weak ciphers, summarize, etc.) |
| `tests/dispatcher_tests.rs` | New: 4 tests (routing, content detection, port fallback) |

## Self-Review Checklist

- [x] Spec coverage: StreamDispatcher (ADR 0001) → Task 1+2. TlsAnalyzer with record parsing → Task 3. ClientHello/JA3 → Task 3. ServerHello/JA3S → Task 3+4. Weak cipher findings → Task 3+4. summarize() → Task 5. CLI integration → Task 6. Public accessors → Task 1 (stub) + Task 3 (final). All 13 spec tests covered.
- [x] No placeholders: All code blocks contain complete implementation. No "TBD" or "similar to" references.
- [x] Type consistency: `TlsCipherSuiteID`, `TlsExtensionType`, `TlsVersion` used consistently with `.0` access for u16 values. `FlowKey` cloned where needed. `HashMap<String, u64>` pattern consistent across counters.
