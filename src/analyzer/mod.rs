pub mod dns;

use std::collections::HashMap;

use serde::Serialize;

use crate::decoder::ParsedPacket;
use crate::findings::Finding;

#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    pub analyzer_name: String,
    pub packets_analyzed: u64,
    pub detail: HashMap<String, serde_json::Value>,
}

pub trait ProtocolAnalyzer {
    /// Human-readable name for this analyzer.
    fn name(&self) -> &'static str;

    /// Return true if this analyzer handles the given packet.
    fn can_decode(&self, packet: &ParsedPacket) -> bool;

    /// Process a packet. Returns any findings (threats, anomalies).
    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding>;

    /// Produce a summary after all packets have been processed.
    fn summarize(&self) -> AnalysisSummary;
}
