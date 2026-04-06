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
