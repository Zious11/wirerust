//! Statistics-only DNS analyzer (BC-2.08.004).
//!
//! Matches port-53 traffic on both TCP and UDP. For each packet it inspects
//! the DNS header QR bit (byte 2, bit 7) to classify the packet as a query
//! (QR = 0) or a response (QR = 1), then increments the corresponding
//! counter. Payloads shorter than 12 bytes (the minimum DNS header length)
//! are classified as responses via the length guard in `is_query`.
//!
//! `summarize()` reports two counters — `dns_queries` and `dns_responses` —
//! in the [`AnalysisSummary`] detail map.
//!
//! `analyze()` unconditionally returns an empty `Vec<Finding>`. DNS anomaly
//! detection (qname parsing, DGA-class entropy, NXDOMAIN spike detection,
//! confidence-leveled findings) does not exist in this implementation and is
//! explicitly out of scope (see BC-2.08.004).

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
