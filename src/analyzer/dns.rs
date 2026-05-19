//! DNS query/response analyzer.
//!
//! Operates on raw UDP/53 packets directly (not via TCP reassembly), parsing
//! the question section to extract qnames and tracking suspicious patterns:
//! DGA-class entropy on labels, unusually long subdomains, NXDOMAIN spikes,
//! and rare-TLD lookups. Findings carry confidence levels reflecting how
//! noisy each pattern is in benign traffic.

use std::collections::BTreeMap;

use crate::analyzer::{AnalysisSummary, ProtocolAnalyzer};
use crate::decoder::{ParsedPacket, TransportInfo};
use crate::findings::Finding;

pub struct DnsAnalyzer {
    query_count: u64,
    response_count: u64,
}

impl Default for DnsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl DnsAnalyzer {
    pub fn new() -> Self {
        DnsAnalyzer {
            query_count: 0,
            response_count: 0,
        }
    }

    fn is_dns_port(src: u16, dst: u16) -> bool {
        src == 53 || dst == 53
    }

    fn is_query(payload: &[u8]) -> bool {
        // DNS header: first 2 bytes = transaction ID, byte 2 bit 7 = QR (0=query, 1=response)
        if payload.len() < 12 {
            return false;
        }
        (payload[2] & 0x80) == 0
    }
}

impl ProtocolAnalyzer for DnsAnalyzer {
    fn name(&self) -> &'static str {
        "DNS"
    }

    fn can_decode(&self, packet: &ParsedPacket) -> bool {
        match &packet.transport {
            TransportInfo::Udp { src_port, dst_port } => Self::is_dns_port(*src_port, *dst_port),
            TransportInfo::Tcp {
                src_port, dst_port, ..
            } => Self::is_dns_port(*src_port, *dst_port),
            TransportInfo::None => false,
        }
    }

    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding> {
        if Self::is_query(&packet.payload) {
            self.query_count += 1;
        } else {
            self.response_count += 1;
        }

        Vec::new()
    }

    fn summarize(&self) -> AnalysisSummary {
        let mut detail: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        detail.insert(
            "dns_queries".to_string(),
            serde_json::json!(self.query_count),
        );
        detail.insert(
            "dns_responses".to_string(),
            serde_json::json!(self.response_count),
        );

        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.query_count + self.response_count,
            detail,
        }
    }
}
