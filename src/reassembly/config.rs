//! Reassembly engine configuration.
//!
//! [`ReassemblyConfig`] holds the resource ceilings that bound the
//! [`crate::reassembly::TcpReassembler`] — per-direction depth, total
//! memcap, idle timeout, concurrent-flow count, the forward receive
//! window, and the five anomaly-detection fields (LESSON-P2.05).
//! Extracted from `mod.rs` for LESSON-P2.01.
//!
//! ## Anomaly thresholds (LESSON-P2.05)
//!
//! Five fields control the overlap / small-segment / out-of-window
//! Anomaly findings. A research pass against Suricata, Zeek, and Snort
//! established that **no production NIDS ships an enabled, directly-
//! comparable detector**: Suricata reassembles and inspects the result
//! rather than alerting on segment-size distribution, Zeek ships no
//! small-segment notice, and Snort's count-based knobs (`overlap_limit`,
//! `small_segments`) both default to 0/disabled. These values are
//! therefore conservative engineering defaults, not values calibrated
//! against prior art; each field's doc comment records the closest
//! cited reference. All five are CLI-overridable (`--overlap-threshold`,
//! `--small-segment-threshold`, `--small-segment-max-bytes`,
//! `--small-segment-ignore-ports`, `--out-of-window-threshold`) so
//! operators can tune per-network.

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
    /// Length of a *consecutive run* of small (undersized) segments, per
    /// flow direction, **above which** a small-segment Anomaly finding
    /// is emitted. A segment counts as small when its payload is shorter
    /// than [`Self::small_segment_max_bytes`]; a normal-sized segment
    /// resets the run to zero.
    ///
    /// LESSON-P2.05: TCP segmentation-evasion (e.g. `fragroute tcp_seg
    /// 1`) shows up as a long *unbroken* run of tiny segments, whereas
    /// benign interactive traffic (telnet / rlogin keystrokes)
    /// interleaves tiny segments with normal-sized ones. A
    /// consecutive-run counter — in the spirit of Snort's
    /// `stream_tcp.small_segments` — separates the two far better than
    /// the cumulative count this field previously held. The run resets
    /// on *any single* normal-sized segment, so an attacker who splices
    /// one `>= small_segment_max_bytes` segment into the run evades the
    /// detector; a port-independent directional-symmetry discriminator
    /// (tracked as a follow-up) would be more robust. No NIDS publishes
    /// a recommended value (Snort ships the feature disabled); `100` is
    /// a conservative engineering default: low enough to catch a
    /// 1-byte-segmented exploit payload (typically 300–900 segments),
    /// high enough to tolerate ordinary interactive bursts. Tune via
    /// `--small-segment-threshold`.
    pub small_segment_alert_threshold: u32,
    /// Payload-size cutoff: a TCP segment carrying fewer than this many
    /// payload bytes is classified as "small" for the
    /// [`Self::small_segment_alert_threshold`] run counter. Empty
    /// segments (pure ACKs) are never counted either way.
    ///
    /// LESSON-P2.05: the closest prior art is the size parameter of
    /// Snort's `stream_tcp.small_segments`, operator-chosen in the range
    /// 1–2048 (the documentation examples use `15`). `16` is a
    /// conservative default — small enough to flag deliberate
    /// fragmentation, large enough to also catch an attacker who splits
    /// into 8–15 byte segments rather than literal 1-byte ones. Setting
    /// it to `0` disables small-segment detection entirely. The
    /// `--small-segment-max-bytes` flag enforces Snort's 0–2048 range.
    pub small_segment_max_bytes: u16,
    /// Ports on which small-segment detection is suppressed. A flow is
    /// exempt when **either** endpoint port appears here — a small-segment
    /// run on these ports is benign interactive traffic, not evasion.
    ///
    /// LESSON-P2.05: the analogue is the `ignore_ports` parameter of
    /// Snort 2.9's `stream5` `small_segments` (a space-separated port
    /// list). The default `[23, 513]` covers telnet and rlogin — both
    /// send one 1-byte segment per keystroke and so produce long benign
    /// runs of tiny segments. SSH (22) is deliberately **not** included:
    /// its encrypted keystroke packets are ~28–96 bytes, above the
    /// `small_segment_max_bytes` cutoff, so they never count as small —
    /// excluding 22 would only create a detection blind spot. (If the
    /// cutoff is ever raised above ~28, reconsider adding 22.) An empty
    /// list disables the exemption. Tune via `--small-segment-ignore-ports`.
    pub small_segment_ignore_ports: Vec<u16>,
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
            max_depth: 10 * 1024 * 1024,               // 10 MB per direction
            memcap: 1024 * 1024 * 1024,                // 1 GB total
            flow_timeout_secs: 300,                    // 5 minutes
            max_flows: 100_000,                        // 100K concurrent flows
            max_segments_per_direction: 10_000,        // 10K segments per direction
            max_receive_window: 1_048_576,             // 1 MB forward window
            overlap_alert_threshold: 50,               // see field doc (LESSON-P2.05)
            small_segment_alert_threshold: 100,        // see field doc (LESSON-P2.05)
            small_segment_max_bytes: 16,               // see field doc (LESSON-P2.05)
            small_segment_ignore_ports: vec![23, 513], // telnet, rlogin (LESSON-P2.05)
            out_of_window_alert_threshold: 100,        // see field doc (LESSON-P2.05)
        }
    }
}
