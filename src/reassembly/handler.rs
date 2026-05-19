//! Stream-handler / stream-analyzer interfaces.
//!
//! The [`StreamHandler`] trait is the callback surface invoked by
//! [`crate::reassembly::TcpReassembler`] when contiguous TCP-stream data
//! becomes available. The [`StreamAnalyzer`] super-trait additionally
//! exposes name / findings / summarize hooks so the dispatcher can
//! treat per-protocol analyzers uniformly. [`Direction`] and
//! [`CloseReason`] are the shared event vocabulary.

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reassembly::flow::FlowKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    ClientToServer,
    ServerToClient,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseReason {
    Fin,
    Rst,
    Timeout,
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
