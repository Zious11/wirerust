//! Reassembly engine configuration.
//!
//! [`ReassemblyConfig`] holds the resource ceilings that bound the
//! [`crate::reassembly::TcpReassembler`] — per-direction depth, total
//! memcap, idle timeout, concurrent-flow count, per-direction segment
//! count, and the forward receive window. Extracted from `mod.rs` for
//! LESSON-P2.01.

/// Configuration for the TCP reassembly engine.
#[derive(Debug, Clone)]
pub struct ReassemblyConfig {
    /// Maximum bytes to reassemble per-direction before stopping (depth limit).
    pub max_depth: usize,
    /// Maximum total memory across all flows before eviction kicks in.
    pub memcap: usize,
    /// Seconds of inactivity before a flow is considered timed out.
    pub flow_timeout_secs: u32,
    /// Maximum number of concurrent flows tracked. Prevents flow table flooding.
    pub max_flows: usize,
    /// Maximum segments per flow direction. Prevents BTreeMap overhead explosion.
    pub max_segments_per_direction: usize,
    /// Maximum distance (bytes) ahead of base_offset to accept a segment.
    /// Segments beyond this are dropped. Default 1MB matches Suricata/Zeek/Snort.
    pub max_receive_window: usize,
}

impl Default for ReassemblyConfig {
    fn default() -> Self {
        ReassemblyConfig {
            max_depth: 10 * 1024 * 1024,        // 10 MB per direction
            memcap: 1024 * 1024 * 1024,         // 1 GB total
            flow_timeout_secs: 300,             // 5 minutes
            max_flows: 100_000,                 // 100K concurrent flows
            max_segments_per_direction: 10_000, // 10K segments per direction
            max_receive_window: 1_048_576,      // 1 MB forward window
        }
    }
}
