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
            detail: HashMap::new(),
        }
    }

    fn findings(&self) -> Vec<Finding> {
        self.all_findings.clone()
    }
}
