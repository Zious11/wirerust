//! Per-protocol analyzers and the shared [`ProtocolAnalyzer`] trait.
//!
//! Each protocol-specific submodule (`dns`, `http`, `tls`) inspects either
//! raw [`crate::decoder::ParsedPacket`]s ([`ProtocolAnalyzer`]) or reassembled
//! TCP stream data ([`crate::reassembly::handler::StreamAnalyzer`]) and emits
//! [`crate::findings::Finding`]s plus an [`AnalysisSummary`].
//!
//! `AnalysisSummary` is the universal shape consumed by both
//! [`crate::reporter::terminal::TerminalReporter`] and
//! [`crate::reporter::json::JsonReporter`]; analyzer-specific metric
//! key/value pairs live in `detail` so the reporter does not need to know
//! per-protocol schemas.

pub mod dns;
pub mod http;
pub mod tls;

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
