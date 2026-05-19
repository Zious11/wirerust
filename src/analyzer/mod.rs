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

use std::collections::BTreeMap;

use serde::Serialize;

use crate::decoder::ParsedPacket;
use crate::findings::Finding;

/// Per-analyzer summary produced at end-of-capture for the reporters.
///
/// LESSON-P2.09 / NFR DET-001: `detail` is a `BTreeMap`, not a
/// `HashMap`, so its JSON serialization is deterministic
/// (alphabetical key order). Two reasons:
///   - Snapshot / golden-file tests over JSON output stay stable.
///   - Diffing two JSON reports across captures or wirerust versions
///     produces minimal, semantically-meaningful diffs.
///
/// The change is a pure observation tightening — analyzer code that
/// previously did `detail.insert(...)` continues to work unchanged
/// because `BTreeMap` exposes the same `insert` / `iter` API.
#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    /// Human-readable analyzer name (e.g. "DNS", "HTTP", "TLS",
    /// "TCP Reassembly").
    pub analyzer_name: String,
    /// Number of packets this analyzer actually processed (the
    /// reassembly engine subtracts non-TCP traffic, etc.).
    pub packets_analyzed: u64,
    /// Analyzer-specific metric key/value pairs. Keys are stable
    /// identifiers (`top_snis`, `ja3_hashes`, `parse_errors`, …);
    /// values are arbitrary serde_json::Value so analyzers can emit
    /// their natural shape (string lists, nested maps, scalars).
    pub detail: BTreeMap<String, serde_json::Value>,
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
