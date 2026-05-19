//! Stream-handler / stream-analyzer interfaces.
//!
//! The [`StreamHandler`] trait is the callback surface invoked by
//! [`crate::reassembly::TcpReassembler`] when contiguous TCP-stream data
//! becomes available. The [`StreamAnalyzer`] super-trait additionally
//! exposes name / findings / summarize hooks so the dispatcher can
//! treat per-protocol analyzers uniformly. [`Direction`] and
//! [`CloseReason`] are the shared event vocabulary.

use serde::Serialize;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;

/// Per-flow byte direction inside a TCP stream.
///
/// LESSON-P2.08: implements `Serialize` so it can be carried on a
/// [`crate::findings::Finding`] and surfaced in JSON output. The
/// variant names serialize as their CamelCase identifiers
/// ("ClientToServer", "ServerToClient") via serde's default
/// representation for fieldless enums.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum Direction {
    /// Bytes flowing from the initiator (typically the client that
    /// emitted SYN) toward the responder.
    ClientToServer,
    /// Bytes flowing from the responder back to the initiator.
    ServerToClient,
}

/// Reason a TCP flow was closed (informational; carried in the
/// `on_flow_close` callback).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum CloseReason {
    /// Mutual FIN handshake completed normally.
    Fin,
    /// One side sent RST.
    Rst,
    /// `flow_timeout_secs` elapsed without activity.
    Timeout,
    /// Engine evicted the flow under memcap pressure.
    MemoryPressure,
}

pub trait StreamHandler {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64);

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason);
}

pub trait StreamAnalyzer: StreamHandler {
    fn name(&self) -> &'static str;
    fn summarize(&self) -> AnalysisSummary;
    fn findings(&self) -> Vec<Finding>;
}
