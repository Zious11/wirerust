//! Reassembly engine configuration.
//!
//! [`ReassemblyConfig`] holds the resource ceilings that bound the
//! [`crate::reassembly::TcpReassembler`] — per-direction depth, total
//! memcap, idle timeout, concurrent-flow count, the forward receive
//! window, and the three per-flow-direction anomaly-alert thresholds
//! (LESSON-P2.05). Extracted from `mod.rs` for LESSON-P2.01.
//!
//! ## Anomaly thresholds (LESSON-P2.05)
//!
//! The three `*_alert_threshold` fields control when the engine emits
//! overlap / small-segment / out-of-window Anomaly findings. A research
//! pass against Suricata, Zeek, and Snort established that **no
//! production NIDS exposes a directly-comparable "count occurrences per
//! flow direction and alert at N" threshold** — Suricata acts
//! per-event with exponential backoff, Zeek bounds byte volumes and
//! ships its overlap detector disabled, and Snort's count-based knobs
//! (`overlap_limit`, `small_segments`) both default to 0/disabled.
//! These three values therefore cannot be "calibrated" against prior
//! art; they are conservative engineering defaults, and each field's
//! doc comment records the closest cited reference. They are also CLI-
//! overridable (`--overlap-threshold`, `--small-segment-threshold`,
//! `--out-of-window-threshold`) so operators can tune per-network.

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
    /// Overlapping-segment count, per flow direction, **above which** an
    /// overlap Anomaly finding is emitted.
    ///
    /// LESSON-P2.05: no NIDS ships a count-based default for this —
    /// Suricata acts per-overlap with exponential backoff, Zeek's
    /// overlap detector (`tcp_max_old_segments`) ships disabled. The
    /// closest analogue is Snort's `stream_tcp.overlap_limit`, a real
    /// per-session overlapping-segment counter with a valid range of
    /// 0–255 (default 0/unlimited). The default `50` is a conservative
    /// engineering choice inside that sanctioned range — not a value
    /// any source endorses.
    pub overlap_alert_threshold: u32,
    /// Small (undersized) segment count, per flow direction, **above
    /// which** a small-segment Anomaly finding is emitted.
    ///
    /// LESSON-P2.05: research flagged this default as likely too
    /// permissive. `2048` is the **maximum** of Snort's
    /// `stream_tcp.small_segments.count` knob — whose actual default is
    /// `0`/disabled — not a recommended detection value. A fine-grained
    /// segmentation-evasion stream would need 2 KB+ of payload to trip
    /// it. The value is retained as the default to avoid a behavior
    /// change without false-positive data; operators tuning for evasion
    /// detection should lower it via `--small-segment-threshold` (the
    /// low hundreds is a reasonable starting point).
    pub small_segment_alert_threshold: u32,
    /// Out-of-window segment count, per flow direction, **above which**
    /// an out-of-window Anomaly finding is emitted.
    ///
    /// LESSON-P2.05: no NIDS counts out-of-window *segments*. The
    /// nearest cited analogue is Zeek's
    /// `tcp_max_above_hole_without_any_acks` (16384 *bytes* above a
    /// sequence hole), a different unit. The default `100` is a
    /// conservative engineering default with no count-based prior art.
    pub out_of_window_alert_threshold: u32,
}

impl Default for ReassemblyConfig {
    fn default() -> Self {
        ReassemblyConfig {
            max_depth: 10 * 1024 * 1024,         // 10 MB per direction
            memcap: 1024 * 1024 * 1024,          // 1 GB total
            flow_timeout_secs: 300,              // 5 minutes
            max_flows: 100_000,                  // 100K concurrent flows
            max_segments_per_direction: 10_000,  // 10K segments per direction
            max_receive_window: 1_048_576,       // 1 MB forward window
            overlap_alert_threshold: 50,         // see field doc (LESSON-P2.05)
            small_segment_alert_threshold: 2048, // see field doc (LESSON-P2.05)
            out_of_window_alert_threshold: 100,  // see field doc (LESSON-P2.05)
        }
    }
}
