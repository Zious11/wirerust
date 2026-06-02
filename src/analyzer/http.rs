//! HTTP/1.x request analyzer driven by reassembled TCP stream data.
//!
//! Implements [`StreamAnalyzer`] so the [`crate::dispatcher::StreamDispatcher`]
//! can route bytes here once a flow is content-classified as HTTP. Per-flow
//! state is bounded by `MAX_HEADER_BUF`, `MAX_HEADERS`, and `MAX_URIS` to
//! prevent attacker-controlled growth.
//!
//! Anomaly detection covers: directory-traversal URIs, encoded path traversal,
//! upload paths with executable extensions, unusual HTTP methods, missing or
//! empty `Host` headers on HTTP/1.1 (RFC 7230 §5.4), abnormally long URIs,
//! and empty `User-Agent` values. Rationale for the asymmetric missing- vs
//! empty-UA handling is documented inline at rule 7.

use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

const MAX_HEADER_BUF: usize = 65_536;
const MAX_HEADERS: usize = 96;
const MAX_URIS: usize = 10_000;
const MAX_MAP_ENTRIES: usize = 50_000;

struct ParsedRequest {
    bytes_consumed: usize,
    method: String,
    uri: String,
    version: u8,
    host: Option<String>,
    user_agent: Option<String>,
}

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

struct ParsedResponse {
    bytes_consumed: usize,
    status_code: u16,
}

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

fn find_header(headers: &[httparse::Header<'_>], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| String::from_utf8_lossy(h.value).trim().to_string())
}

/// Number of consecutive parse errors before poisoning a direction.
/// Set > 1 to tolerate mid-stream joins where the first segment(s)
/// are body data from a transfer that started before the capture.
const POISON_THRESHOLD: u8 = 3;

struct HttpFlowState {
    request_buf: Vec<u8>,
    response_buf: Vec<u8>,
    request_poisoned: bool,
    response_poisoned: bool,
    request_error_count: u8,
    response_error_count: u8,
    counted_as_non_http: bool,
}

impl HttpFlowState {
    fn new() -> Self {
        HttpFlowState {
            request_buf: Vec::new(),
            response_buf: Vec::new(),
            request_poisoned: false,
            response_poisoned: false,
            request_error_count: 0,
            response_error_count: 0,
            counted_as_non_http: false,
        }
    }
}

fn truncate_uri(uri: &str, max_len: usize) -> &str {
    if uri.len() <= max_len {
        uri
    } else {
        &uri[..uri.floor_char_boundary(max_len)]
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
    parse_errors: u64,
    non_http_flows: u64,
    poisoned_bytes_skipped: u64,
}

impl Default for HttpAnalyzer {
    fn default() -> Self {
        Self::new()
    }
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
            parse_errors: 0,
            non_http_flows: 0,
            poisoned_bytes_skipped: 0,
        }
    }

    pub fn transaction_count(&self) -> u64 {
        self.transactions
    }

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

    pub fn parse_error_count(&self) -> u64 {
        self.parse_errors
    }

    pub fn poisoned_bytes_skipped(&self) -> u64 {
        self.poisoned_bytes_skipped
    }

    fn check_request_detections(&mut self, parsed: &ParsedRequest, _flow_key: &FlowKey) {
        let uri_lower = parsed.uri.to_lowercase();

        // 1. Path traversal (including URL-encoded variants)
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
                direction: Some(Direction::ClientToServer),
            });
        }

        // 2. Web shell paths (specific file extensions to reduce false positives)
        let shell_patterns = [
            "/shell.php",
            "/shell.asp",
            "/shell.jsp",
            "/cmd.php",
            "/cmd.asp",
            "/cmd.jsp",
            "/c99.php",
            "/r57.php",
            "/webshell",
            "/backdoor",
        ];
        if shell_patterns.iter().any(|p| uri_lower.contains(p)) {
            self.all_findings.push(Finding {
                category: ThreatCategory::Execution,
                verdict: Verdict::Likely,
                confidence: Confidence::Medium,
                summary: format!(
                    "Possible web shell access: {}",
                    truncate_uri(&parsed.uri, 120)
                ),
                evidence: vec![format!("URI: {}", parsed.uri)],
                mitre_technique: Some("T1505.003".to_string()),
                source_ip: None,
                timestamp: None,
                direction: Some(Direction::ClientToServer),
            });
        }

        // 3. Admin panel paths
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
                direction: Some(Direction::ClientToServer),
            });
        }

        // 4. Unusual HTTP methods
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
                direction: Some(Direction::ClientToServer),
            });
        }

        // 5. Missing or empty Host header on HTTP/1.1.
        //
        // RFC 7230 §5.4 (and successor RFC 9112 §3.2) require an HTTP/1.1
        // request to carry exactly one non-empty `Host` field-value; both
        // absent-Host and empty-value-Host are equally non-compliant and
        // are documented evasion lanes in front-end/back-end request-
        // smuggling research (PortSwigger; Node.js CVE-2022-35256). The
        // closest comparator tool (Suricata) surfaces these as two
        // separate events (sids 2221014 `http.missing_host_header` and
        // 2221028 `http.request_header_host_invalid`); we fold both into
        // one Anomaly finding but disambiguate via the summary text so
        // downstream analysts can grep either case.
        //
        // Note: `find_header` already trims whitespace from header
        // values, so `Some("")` here covers both `Host:\r\n` and
        // `Host:   \r\n`.
        if parsed.version == 1 {
            let host_anomaly_summary = match parsed.host.as_deref() {
                None => Some("HTTP/1.1 request without Host header"),
                Some("") => Some("HTTP/1.1 request with empty Host header"),
                Some(_) => None,
            };
            if let Some(summary) = host_anomaly_summary {
                self.all_findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Inconclusive,
                    confidence: Confidence::Medium,
                    summary: summary.to_string(),
                    evidence: vec![format!("{} {}", parsed.method, parsed.uri)],
                    mitre_technique: None,
                    source_ip: None,
                    timestamp: None,
                    direction: Some(Direction::ClientToServer),
                });
            }
        }

        // 6. Long URI (> 2048 chars)
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
                direction: Some(Direction::ClientToServer),
            });
        }

        // 7. Empty User-Agent (deliberately asymmetric with the Host
        //    check above — only `Some("")` fires, absent UA is ignored).
        //
        // Rationale for not firing on absent UA:
        //   - Many legitimate clients omit UA entirely (cron jobs,
        //     internal microservices, healthchecks, embedded HTTP
        //     libraries). Snort ships its "POLICY-OTHER HTTP Request
        //     missing user-agent" rule (sid 1:38130) **disabled by
        //     default** for this reason — it is treated as a policy
        //     violation, not a malicious-traffic indicator.
        //   - Empty-UA, in contrast, is a stronger signal. Kheir (2015,
        //     "Malware Detection Using HTTP User-Agent Discrepancy
        //     Identification") reports ~24% of malware samples in a
        //     181k-sample Totalhash corpus emit an empty UA. Real
        //     browsers always populate UA, and common tools (curl,
        //     wget, Python `requests`) send a default string when one
        //     is not overridden.
        //   - Suricata's `http-events.rules` ships no built-in UA
        //     presence/emptiness anomaly at all; both detections are
        //     left to rule-authors via `http.user_agent` content
        //     matching, which we do not do here.
        //
        // If a policy-mode "missing UA" finding is later desired, it
        // should be added as a separate, lower-confidence finding
        // rather than collapsing the two cases.
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
                direction: Some(Direction::ClientToServer),
            });
        }
    }

    fn try_parse_requests(&mut self, flow_key: &FlowKey) {
        // Track whether we've successfully parsed headers this call. After a
        // successful parse, remaining bytes are likely HTTP body data (we don't
        // handle Content-Length/Transfer-Encoding). Suppress error counting for
        // body-byte-induced failures to avoid inflating the counter on normal traffic.
        let mut had_success = false;
        loop {
            let result = self
                .flows
                .get(flow_key)
                .filter(|s| !s.request_buf.is_empty())
                .map(|s| parse_one_request(&s.request_buf));

            match result {
                Some(Ok(Some(parsed))) => {
                    had_success = true;
                    if self.methods.len() < MAX_MAP_ENTRIES
                        || self.methods.contains_key(&parsed.method)
                    {
                        *self.methods.entry(parsed.method.clone()).or_insert(0) += 1;
                    }
                    if let Some(ref h) = parsed.host
                        && (self.hosts.len() < MAX_MAP_ENTRIES || self.hosts.contains_key(h))
                    {
                        *self.hosts.entry(h.clone()).or_insert(0) += 1;
                    }
                    if let Some(ref ua) = parsed.user_agent
                        && (self.user_agents.len() < MAX_MAP_ENTRIES
                            || self.user_agents.contains_key(ua))
                    {
                        *self.user_agents.entry(ua.clone()).or_insert(0) += 1;
                    }
                    if self.uris.len() < MAX_URIS {
                        self.uris.push(parsed.uri.clone());
                    }

                    self.check_request_detections(&parsed, flow_key);

                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.request_buf.drain(..parsed.bytes_consumed);
                        state.request_error_count = 0;
                    }
                }
                Some(Ok(None)) => return, // Partial — wait for more data
                Some(Err(e)) => {
                    if !had_success {
                        self.parse_errors += 1;
                        if let Some(state) = self.flows.get_mut(flow_key) {
                            state.request_error_count = state.request_error_count.saturating_add(1);
                            if state.request_error_count >= POISON_THRESHOLD {
                                state.request_poisoned = true;
                                if !state.counted_as_non_http {
                                    state.counted_as_non_http = true;
                                    self.non_http_flows += 1;
                                }
                            }
                        }
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
                            direction: Some(Direction::ClientToServer),
                            });
                        }
                    }
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.request_buf.clear();
                    }
                    return;
                }
                None => return,
            }
        }
    }

    fn try_parse_responses(&mut self, flow_key: &FlowKey) {
        let mut had_success = false;
        loop {
            let result = self
                .flows
                .get(flow_key)
                .filter(|s| !s.response_buf.is_empty())
                .map(|s| parse_one_response(&s.response_buf));

            match result {
                Some(Ok(Some(parsed))) => {
                    had_success = true;
                    *self.status_codes.entry(parsed.status_code).or_insert(0) += 1;
                    self.transactions += 1;

                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.response_buf.drain(..parsed.bytes_consumed);
                        state.response_error_count = 0;
                    }
                }
                Some(Ok(None)) => return,
                Some(Err(e)) => {
                    if !had_success {
                        self.parse_errors += 1;
                        if let Some(state) = self.flows.get_mut(flow_key) {
                            state.response_error_count =
                                state.response_error_count.saturating_add(1);
                            if state.response_error_count >= POISON_THRESHOLD {
                                state.response_poisoned = true;
                                if !state.counted_as_non_http {
                                    state.counted_as_non_http = true;
                                    self.non_http_flows += 1;
                                }
                            }
                        }
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
                            direction: Some(Direction::ServerToClient),
                            });
                        }
                    }
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.response_buf.clear();
                    }
                    return;
                }
                None => return,
            }
        }
    }
}

impl StreamHandler for HttpAnalyzer {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], _offset: u64) {
        {
            let state = self
                .flows
                .entry(flow_key.clone())
                .or_insert_with(HttpFlowState::new);
            match direction {
                Direction::ClientToServer => {
                    if state.request_poisoned {
                        self.poisoned_bytes_skipped += data.len() as u64;
                        return;
                    }
                    let remaining = MAX_HEADER_BUF.saturating_sub(state.request_buf.len());
                    if remaining > 0 {
                        state
                            .request_buf
                            .extend_from_slice(&data[..data.len().min(remaining)]);
                    }
                }
                Direction::ServerToClient => {
                    if state.response_poisoned {
                        self.poisoned_bytes_skipped += data.len() as u64;
                        return;
                    }
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
            Direction::ServerToClient => self.try_parse_responses(flow_key),
        }
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
        // LESSON-P2.09: BTreeMap so the JSON output keys are
        // alphabetically ordered and deterministic across runs.
        let mut detail: std::collections::BTreeMap<String, serde_json::Value> =
            std::collections::BTreeMap::new();

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

        let mut top_hosts: Vec<_> = self.hosts.iter().collect();
        top_hosts.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        let top_hosts: Vec<&str> = top_hosts.iter().take(20).map(|(k, _)| k.as_str()).collect();
        detail.insert("top_hosts".to_string(), serde_json::json!(top_hosts));

        let recent_uris: Vec<&str> = self.uris.iter().take(20).map(|s| s.as_str()).collect();
        detail.insert("recent_uris".to_string(), serde_json::json!(recent_uris));

        detail.insert(
            "user_agents".to_string(),
            serde_json::json!(self.user_agents),
        );
        detail.insert(
            "parse_errors".to_string(),
            serde_json::json!(self.parse_errors),
        );
        detail.insert(
            "non_http_flows".to_string(),
            serde_json::json!(self.non_http_flows),
        );
        detail.insert(
            "poisoned_bytes_skipped".to_string(),
            serde_json::json!(self.poisoned_bytes_skipped),
        );

        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.transactions,
            detail,
        }
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}

// ---- Test-only seams (STORY-021 adversarial-pass-1 remediation / F-W11P1-002) ----
//
// These seams expose `HttpAnalyzer`-internal state to integration tests so they
// can verify that `HttpAnalyzer` does NOT apply the engine-local `MAX_FINDINGS`
// cap (BC-2.04.024 invariant 4). All follow the `#[doc(hidden)] pub fn` append
// pattern established in `src/reassembly/mod.rs`. MUST NOT be called from
// production code.
impl HttpAnalyzer {
    /// Test-only accessor for the count of accumulated findings.
    ///
    /// Exposes `self.all_findings.len()` so integration tests can verify
    /// that `HttpAnalyzer` does NOT apply the `MAX_FINDINGS` cap used by
    /// `TcpReassembler` (BC-2.04.024 invariant 4 / AC-007b — analyzer non-cap).
    /// The analyzer pushes to `all_findings` unconditionally — there is no local cap.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn all_findings_len_for_testing(&self) -> usize {
        self.all_findings.len()
    }

    /// Test-only direct push of a [`Finding`] into the analyzer's findings vec.
    ///
    /// Bypasses the normal analyzer detection logic so tests can pre-fill
    /// `all_findings` to arbitrary lengths (e.g. > 10,000) to verify that
    /// `HttpAnalyzer` has NO local cap analogous to `TcpReassembler::MAX_FINDINGS`
    /// (BC-2.04.024 invariant 4 / AC-007b — analyzer non-cap companion test).
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn push_finding_for_testing(&mut self, finding: Finding) {
        self.all_findings.push(finding);
    }

    /// Test-only accessor: number of active per-flow state entries.
    ///
    /// Exposes `self.flows.len()` so integration tests can verify that
    /// `HttpAnalyzer::on_flow_close` removes the per-flow state (BC-2.05.009
    /// analyzer-forward side effect / STORY-033 AC-007). A flow that has been
    /// closed must no longer appear in the `flows` map.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn active_flows_len_for_testing(&self) -> usize {
        self.flows.len()
    }

    /// Test-only accessor: current byte length of the request-direction buffer
    /// for the given `flow_key`.
    ///
    /// Exposes `state.request_buf.len()` so integration tests can assert exact
    /// buffer-cap boundary behaviour (BC-2.06.022 / STORY-045 AC-006/007,
    /// EC-005 and EC-006).  Returns `None` when the flow has no live state.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn request_buf_len_for_testing(&self, flow_key: &FlowKey) -> Option<usize> {
        self.flows.get(flow_key).map(|s| s.request_buf.len())
    }

    /// Test-only accessor: current byte length of the response-direction buffer
    /// for the given `flow_key`.
    ///
    /// Mirrors `request_buf_len_for_testing` for the `ServerToClient` direction
    /// so integration tests can assert exact buffer-cap boundary behaviour for
    /// responses (BC-2.06.022 invariant 3 / STORY-045 F-W17-S045-P1-005).
    /// Returns `None` when the flow has no live state.
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn response_buf_len_for_testing(&self, flow_key: &FlowKey) -> Option<usize> {
        self.flows.get(flow_key).map(|s| s.response_buf.len())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::truncate_uri;

    /// BC-2.06.010 invariant 2: `truncate_uri` must not split a UTF-8 codepoint.
    ///
    /// Exercises `floor_char_boundary` on a multibyte URI where the requested
    /// byte limit falls in the middle of a 2-byte codepoint ('é', U+00E9).
    #[test]
    fn test_BC_2_06_010_truncate_uri_multibyte_two_byte_codepoint() {
        // 'é' encodes as [0xC3, 0xA9] (2 bytes).  A string of 4 'é' characters
        // is 8 bytes.  A limit of 3 falls mid-codepoint (after byte 2 of the
        // second 'é').  floor_char_boundary(3) must round down to 2.
        let uri = "éééé"; // 4 × 2 = 8 bytes
        let max_len = 3; // falls between byte 2 and 3, i.e. mid-second-codepoint
        let result = truncate_uri(uri, max_len);

        // (a) result length must not exceed max_len bytes
        assert!(
            result.len() <= max_len,
            "truncate_uri must produce at most {max_len} bytes; got {} bytes",
            result.len()
        );
        // (b) result must be valid UTF-8 (no panic above proves no codepoint split)
        assert!(
            std::str::from_utf8(result.as_bytes()).is_ok(),
            "truncate_uri must produce valid UTF-8"
        );
        // (c) result must be strictly shorter than the input (input > max_len)
        assert!(
            result.len() < uri.len(),
            "truncate_uri must shorten the URI when input ({} bytes) > max_len ({max_len})",
            uri.len()
        );
        // (d) exact value: floor_char_boundary(3) on 'é'*4 = 2 (one 'é')
        assert_eq!(
            result, "é",
            "truncate_uri must return exactly one 'é' (2 bytes)"
        );
    }

    /// BC-2.06.010 invariant 2: `truncate_uri` must not split a UTF-8 codepoint.
    ///
    /// Exercises `floor_char_boundary` with a 4-byte emoji codepoint ('🎯', U+1F3AF).
    /// A limit of 5 falls mid-codepoint (after 1 byte of the second emoji).
    #[test]
    fn test_BC_2_06_010_truncate_uri_multibyte_four_byte_codepoint() {
        // '🎯' encodes as [0xF0, 0x9F, 0x8E, 0xAF] (4 bytes).
        // Two emojis = 8 bytes.  A limit of 5 falls after byte 1 of the second
        // emoji — floor_char_boundary(5) must round down to 4 (one full emoji).
        let uri = "🎯🎯"; // 8 bytes
        let max_len = 5;
        let result = truncate_uri(uri, max_len);

        assert!(
            result.len() <= max_len,
            "truncate_uri must produce at most {max_len} bytes; got {} bytes",
            result.len()
        );
        assert!(
            std::str::from_utf8(result.as_bytes()).is_ok(),
            "truncate_uri must produce valid UTF-8"
        );
        assert!(
            result.len() < uri.len(),
            "truncate_uri must shorten the URI when input ({} bytes) > max_len ({max_len})",
            uri.len()
        );
        // floor_char_boundary(5) on '🎯🎯' = 4 (one full emoji)
        assert_eq!(
            result, "🎯",
            "truncate_uri must return exactly one '🎯' (4 bytes)"
        );
    }

    /// BC-2.06.010 invariant 2: when the URI fits within max_len bytes,
    /// `truncate_uri` must return the full URI unchanged.
    #[test]
    fn test_BC_2_06_010_truncate_uri_multibyte_no_truncation_when_fits() {
        let uri = "éé"; // 4 bytes
        let max_len = 10;
        let result = truncate_uri(uri, max_len);
        assert_eq!(
            result, uri,
            "truncate_uri must return full URI when it fits within max_len"
        );
    }
}

// VP-006: HTTP Poison Monotonicity (BC-2.06.015/016/017, INV-8).
//
// Property: within a single flow's lifetime the per-direction poison flags are
// monotonically false->true and per-direction isolated. The flags themselves
// are private; the public observable is `poisoned_bytes_skipped()`, which
// increments for every byte fed to a direction AFTER it is poisoned and never
// decreases (http.rs:509-512, 521-524). These harnesses assert monotonicity and
// per-direction isolation via that public observable.
#[cfg(test)]
mod vp_006_proptest_proofs {
    use super::*;
    use proptest::prelude::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[derive(Clone, Debug)]
    enum ParseEvent {
        ValidRequest,  // bytes that parse as a complete HTTP request
        InvalidBytes,  // bytes that cause a parse error
        ValidResponse, // bytes that parse as a complete HTTP response
    }

    fn test_flow_key() -> FlowKey {
        let c = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
        let s = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
        FlowKey::new(c, 54321, s, 80)
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

        // VP-006 monotonicity: across any ordering of valid/invalid request and
        // valid response chunks, `poisoned_bytes_skipped()` is monotonically
        // non-decreasing. Once a direction poisons, the counter only grows.
        #[test]
        fn prop_poison_bytes_skipped_monotonic(
            events in prop::collection::vec(
                prop_oneof![
                    Just(ParseEvent::ValidRequest),
                    Just(ParseEvent::InvalidBytes),
                    Just(ParseEvent::ValidResponse),
                ],
                1..50
            )
        ) {
            let mut analyzer = HttpAnalyzer::new();
            let key = test_flow_key();
            let mut prev_skipped: u64 = 0;

            for event in &events {
                let data: &[u8] = match event {
                    ParseEvent::ValidRequest => {
                        b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n"
                    }
                    ParseEvent::InvalidBytes => b"\xFF\xFE garbage",
                    ParseEvent::ValidResponse => b"HTTP/1.1 200 OK\r\n\r\n",
                };
                let dir = match event {
                    ParseEvent::ValidResponse => Direction::ServerToClient,
                    _ => Direction::ClientToServer,
                };
                analyzer.on_data(&key, dir, data, 0);

                let now_skipped = analyzer.poisoned_bytes_skipped();
                prop_assert!(
                    now_skipped >= prev_skipped,
                    "poisoned_bytes_skipped decreased: was {} now {} (events: {:?})",
                    prev_skipped,
                    now_skipped,
                    events
                );
                prev_skipped = now_skipped;
            }
        }

        // VP-006 per-direction isolation: poisoning the request direction
        // (>= POISON_THRESHOLD consecutive request errors) must NOT cause a
        // subsequent valid response's bytes to be counted as skipped. The
        // response direction is independent and was never poisoned.
        #[test]
        fn prop_poison_per_direction_isolated(req_errors in 3usize..=10) {
            let mut analyzer = HttpAnalyzer::new();
            let key = test_flow_key();

            // Drive the request direction past POISON_THRESHOLD (3).
            for _ in 0..req_errors {
                analyzer.on_data(&key, Direction::ClientToServer, b"\xFF\xFE garbage", 0);
            }
            let skipped_after_req_poison = analyzer.poisoned_bytes_skipped();

            // The request side must actually be poisoned for this to be a
            // meaningful test: with >= 3 consecutive errors, later request bytes
            // would be skipped. Confirm by feeding more request garbage.
            analyzer.on_data(&key, Direction::ClientToServer, b"\xFF\xFE more", 0);
            prop_assert!(
                analyzer.poisoned_bytes_skipped() > skipped_after_req_poison,
                "request direction was not poisoned after {} consecutive errors",
                req_errors
            );
            let skipped_after_more_req = analyzer.poisoned_bytes_skipped();

            // Now feed a valid response. The response direction is independent
            // and unpoisoned, so its bytes must NOT be added to the skipped
            // counter.
            let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
            analyzer.on_data(&key, Direction::ServerToClient, resp, 0);
            prop_assert_eq!(
                analyzer.poisoned_bytes_skipped(),
                skipped_after_more_req,
                "response-direction bytes were skipped due to request-side poison"
            );
        }
    }
}
