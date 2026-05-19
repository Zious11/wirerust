//! Reassembly engine counters.
//!
//! [`ReassemblyStats`] is the plain-counter struct accumulated by the
//! [`crate::reassembly::TcpReassembler`] over a capture and surfaced
//! through its `summarize()` method. Extracted from `mod.rs` for
//! LESSON-P2.01.

/// Counters exposed by the reassembly engine.
#[derive(Debug, Clone, Default)]
pub struct ReassemblyStats {
    pub packets_processed: u64,
    pub packets_tcp: u64,
    pub packets_skipped_non_tcp: u64,
    pub flows_total: u64,
    pub flows_partial: u64,
    pub flows_expired: u64,
    pub flows_rst: u64,
    pub flows_fin: u64,
    pub segments_inserted: u64,
    pub segments_duplicates: u64,
    pub segments_overlaps: u64,
    pub segments_out_of_window: u64,
    pub segments_segment_limit: u64,
    pub segments_depth_exceeded: u64,
    pub bytes_reassembled: u64,
    pub evictions: u64,
    /// Anomaly findings that were suppressed because `self.findings` had
    /// already reached `MAX_FINDINGS`. Exposed in `summarize()` under
    /// `dropped_findings` so JSON consumers can detect when the cap was
    /// hit and per-flow signal was silently lost — see LESSON-P1.01.
    pub dropped_findings: u64,
}
