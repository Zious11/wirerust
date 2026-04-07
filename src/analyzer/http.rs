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

fn find_header(headers: &[httparse::Header<'_>], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| String::from_utf8_lossy(h.value).to_string())
}

struct HttpFlowState {
    request_buf: Vec<u8>,
    response_buf: Vec<u8>,
}

impl HttpFlowState {
    fn new() -> Self {
        HttpFlowState {
            request_buf: Vec::new(),
            response_buf: Vec::new(),
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
            });
        }

        // 5. Missing Host header (HTTP/1.1 requires it)
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
            });
        }

        // 7. Empty User-Agent
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

    fn try_parse_requests(&mut self, flow_key: &FlowKey) {
        loop {
            let result = self
                .flows
                .get(flow_key)
                .filter(|s| !s.request_buf.is_empty())
                .map(|s| parse_one_request(&s.request_buf));

            match result {
                Some(Ok(Some(parsed))) => {
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
                    }
                }
                Some(Ok(None)) => return, // Partial — wait for more data
                Some(Err(())) => {
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

        let mut top_hosts: Vec<_> = self.hosts.iter().collect();
        top_hosts.sort_by(|a, b| b.1.cmp(a.1));
        let top_hosts: Vec<&str> = top_hosts.iter().take(20).map(|(k, _)| k.as_str()).collect();
        detail.insert("top_hosts".to_string(), serde_json::json!(top_hosts));

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

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}
