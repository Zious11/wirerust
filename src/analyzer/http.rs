use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};

const MAX_HEADER_BUF: usize = 65_536;
const MAX_HEADERS: usize = 96;
const MAX_URIS: usize = 10_000;

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
                    if let Some(state) = self.flows.get_mut(flow_key) {
                        state.request_buf.clear();
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
            Direction::ServerToClient => {} // Task 3
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
