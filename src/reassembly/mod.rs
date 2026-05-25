//! TCP stream reassembly engine.
//!
//! Owns the [`TcpReassembler`] type plus the per-flow ([`flow`]) and
//! per-segment ([`segment`]) state, and the [`handler::StreamHandler`] /
//! [`handler::StreamAnalyzer`] interfaces that downstream protocol
//! analyzers (HTTP, TLS) implement.
//!
//! ## Module layout (LESSON-P2.01)
//!
//! The engine is split across four files within this directory:
//! - `mod.rs` (here) — the [`TcpReassembler`] type and the per-packet
//!   hot path: [`TcpReassembler::process_packet`] decomposed into named
//!   steps, plus `expire_flows` / `finalize` / `summarize` / accessors.
//! - `config.rs` — [`ReassemblyConfig`] (re-exported).
//! - `stats.rs` — [`ReassemblyStats`] (re-exported).
//! - `lifecycle.rs` — flow-retirement and finding-emission internals
//!   (`close_flow`, `evict_flows`, `generate_*_finding`).
//!
//! ## Design highlights
//!
//! - **First-wins overlap policy.** When a retransmitted segment carries
//!   different bytes from the one already buffered at the same offset,
//!   the buffered bytes are kept and a "conflicting overlap" Anomaly
//!   finding is emitted. Mirrors Suricata's default behavior.
//! - **Per-direction latched alerts** for overlap / small-segment /
//!   out-of-window thresholds. The latches flip even when the
//!   `MAX_FINDINGS` cap suppresses the finding (LESSON-P1.01), so
//!   `dropped_findings` counts distinct anomalies — not packets.
//! - **`impl Drop` lifecycle tripwire** that emits a one-shot eprintln
//!   if [`TcpReassembler::finalize`] was not called, catching the
//!   `?`-bail bypass at runtime (LESSON-P0.03).

pub mod flow;
pub mod handler;
pub mod segment;

mod config;
pub mod lifecycle;
mod stats;

pub use config::ReassemblyConfig;
pub use stats::ReassemblyStats;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::analyzer::AnalysisSummary;
use crate::decoder::{ParsedPacket, Protocol, TransportInfo};
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::{FlowKey, FlowState, TcpFlow};
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};
use crate::reassembly::segment::InsertResult;

const MAX_FINDINGS: usize = 10_000;

// The overlap / small-segment / out-of-window anomaly thresholds were
// module-level `const`s; they are now per-engine configuration fields
// (`ReassemblyConfig::*_alert_threshold`, see `config.rs`) so they can
// be tuned via the CLI — LESSON-P2.05.

/// Plural suffix for a count: `""` for exactly one, `"s"` otherwise.
///
/// LESSON-P3.02: extracted from the inline ternary that formatted the
/// segment-limit summary finding, giving the pluralization form one
/// named home for any future count-bearing message to reuse.
fn plural_s(count: u64) -> &'static str {
    if count == 1 { "" } else { "s" }
}

static FINALIZE_SKIPPED_WARNED: AtomicBool = AtomicBool::new(false);

/// TCP header fields extracted from a packet.
///
/// Carried between the `process_packet` sub-steps as a named struct
/// rather than a positional 7-tuple (LESSON-P2.01 readability cleanup).
struct TcpFields {
    src_port: u16,
    dst_port: u16,
    seq: u32,
    syn: bool,
    ack: bool,
    fin: bool,
    rst: bool,
}

/// Outcome of handshake-flag handling: whether `process_packet` should
/// continue to payload handling or stop because the flow was RST-closed.
#[derive(Debug, PartialEq, Eq)]
enum PostHandshake {
    /// No RST — proceed to payload processing.
    Continue,
    /// An RST flushed and removed the flow; stop processing this packet.
    FlowClosed,
}

/// The main TCP reassembly engine.
pub struct TcpReassembler {
    config: ReassemblyConfig,
    flows: HashMap<FlowKey, TcpFlow>,
    stats: ReassemblyStats,
    findings: Vec<Finding>,
    total_memory: usize,
    finalized: bool,
}

impl TcpReassembler {
    pub fn new(config: ReassemblyConfig) -> Self {
        assert!(config.max_depth > 0, "max_depth must be > 0");
        assert!(config.memcap > 0, "memcap must be > 0");
        assert!(config.max_flows > 0, "max_flows must be > 0");
        assert!(
            config.max_segments_per_direction > 0,
            "max_segments_per_direction must be > 0"
        );
        assert!(
            config.max_receive_window > 0,
            "max_receive_window must be > 0"
        );
        TcpReassembler {
            config,
            flows: HashMap::new(),
            stats: ReassemblyStats::default(),
            findings: Vec::new(),
            total_memory: 0,
            finalized: false,
        }
    }

    /// Process a single parsed packet through the reassembly engine.
    ///
    /// The work is decomposed into named sub-steps (LESSON-P2.01); each
    /// re-acquires the flow handle from the table, mirroring the
    /// original combined function's existing per-section re-lookups.
    pub fn process_packet(
        &mut self,
        packet: &ParsedPacket,
        timestamp: u32,
        handler: &mut dyn StreamHandler,
    ) {
        self.stats.packets_processed += 1;

        // Skip non-TCP packets; extract TCP fields; build the flow key.
        let Some((key, tcp)) = self.extract_tcp_context(packet) else {
            return;
        };
        self.stats.packets_tcp += 1;

        // Get-or-create the flow, evicting under capacity pressure.
        if !self.get_or_create_flow(&key, timestamp, handler) {
            return;
        }

        // SYN / SYN+ACK / RST / FIN handling. RST closes the flow.
        if self.apply_handshake_flags(packet, &key, &tcp, handler) == PostHandshake::FlowClosed {
            return;
        }

        // Payload: insert the segment, check anomaly thresholds, flush.
        if !packet.payload.is_empty() {
            let dir = self.insert_payload_segment(packet, &key, &tcp);
            self.check_anomaly_thresholds(packet, &key, dir);
            self.flush_contiguous_data(&key, dir, handler);
        }

        // Remove a flow that FIN-closed during this packet, after its
        // final payload has been processed.
        if self
            .flows
            .get(&key)
            .is_some_and(|f| f.state == FlowState::Closed)
        {
            self.stats.flows_fin += 1;
            self.close_flow(&key, CloseReason::Fin, handler);
        }

        // Evict flows if the memcap was exceeded by this packet.
        if self.total_memory > self.config.memcap {
            self.evict_flows(handler);
        }
    }

    /// Reject non-TCP packets, pull the TCP header fields, and build the
    /// canonical flow key. Returns `None` (counting the packet as
    /// skipped, when it is genuinely non-TCP) if the packet cannot be
    /// processed as a TCP segment.
    fn extract_tcp_context(&mut self, packet: &ParsedPacket) -> Option<(FlowKey, TcpFields)> {
        if packet.protocol != Protocol::Tcp {
            self.stats.packets_skipped_non_tcp += 1;
            return None;
        }
        let tcp = match &packet.transport {
            TransportInfo::Tcp {
                src_port,
                dst_port,
                seq_number,
                syn,
                ack,
                fin,
                rst,
            } => TcpFields {
                src_port: *src_port,
                dst_port: *dst_port,
                seq: *seq_number,
                syn: *syn,
                ack: *ack,
                fin: *fin,
                rst: *rst,
            },
            _ => return None,
        };
        let key = FlowKey::new(packet.src_ip, tcp.src_port, packet.dst_ip, tcp.dst_port);
        Some((key, tcp))
    }

    /// Ensure a [`TcpFlow`] exists for `key`, creating one (and evicting
    /// under `max_flows` pressure) if needed, and stamp `last_seen`.
    /// Returns `false` if the flow table is still at capacity after
    /// eviction and the packet must therefore be dropped.
    fn get_or_create_flow(
        &mut self,
        key: &FlowKey,
        timestamp: u32,
        handler: &mut dyn StreamHandler,
    ) -> bool {
        if !self.flows.contains_key(key) {
            // Enforce max_flows limit
            if self.flows.len() >= self.config.max_flows {
                self.evict_flows(handler);
                if self.flows.len() >= self.config.max_flows {
                    // Still at capacity after eviction — drop this packet
                    return false;
                }
            }
            let flow = TcpFlow::new(key.clone(), timestamp);
            self.flows.insert(key.clone(), flow);
            self.stats.flows_total += 1;
        }

        let flow = self.flows.get_mut(key).unwrap();
        flow.last_seen = timestamp;
        true
    }

    /// Apply SYN / SYN+ACK / RST / FIN semantics to the flow. An RST
    /// flushes and removes the flow; the return value tells
    /// `process_packet` whether to stop ([`PostHandshake::FlowClosed`])
    /// or continue to payload handling ([`PostHandshake::Continue`]).
    fn apply_handshake_flags(
        &mut self,
        packet: &ParsedPacket,
        key: &FlowKey,
        tcp: &TcpFields,
        handler: &mut dyn StreamHandler,
    ) -> PostHandshake {
        let flow = self.flows.get_mut(key).unwrap();

        // SYN (without ACK) — client initiating.
        if tcp.syn && !tcp.ack {
            flow.set_initiator(packet.src_ip, tcp.src_port);
            let dir = flow.direction(packet.src_ip, tcp.src_port);
            flow.get_direction_mut(dir).set_isn(tcp.seq);
            flow.on_syn();
        }

        // SYN+ACK — server responding, so the initiator is the *destination*.
        if tcp.syn && tcp.ack {
            flow.set_initiator(packet.dst_ip, tcp.dst_port);
            let dir = flow.direction(packet.src_ip, tcp.src_port);
            flow.get_direction_mut(dir).set_isn(tcp.seq);
            flow.on_syn_ack();
        }

        // RST — flush salvageable data, close, and remove.
        if tcp.rst {
            flow.on_rst();
            self.stats.flows_rst += 1;
            self.close_flow(key, CloseReason::Rst, handler);
            return PostHandshake::FlowClosed;
        }

        // FIN — mark the direction; if both FINs are now seen the flow
        // is removed after payload processing (see `process_packet`).
        if tcp.fin {
            let dir = flow.direction(packet.src_ip, tcp.src_port);
            flow.get_direction_mut(dir).fin_seen = true;
            flow.on_fin();
        }

        PostHandshake::Continue
    }

    /// Insert the packet payload into the per-direction segment buffer,
    /// inferring mid-stream-join state when no SYN was seen, and update
    /// the segment-class counters. Returns the flow [`Direction`] the
    /// payload belongs to, for the threshold-check and flush steps.
    fn insert_payload_segment(
        &mut self,
        packet: &ParsedPacket,
        key: &FlowKey,
        tcp: &TcpFields,
    ) -> Direction {
        let payload = &packet.payload;
        let flow = self.flows.get_mut(key).unwrap();

        // If no SYN was seen (mid-stream join), infer state.
        if flow.state == FlowState::New {
            flow.on_data_without_syn();
            flow.set_initiator(packet.src_ip, tcp.src_port);
            let dir = flow.direction(packet.src_ip, tcp.src_port);
            flow.get_direction_mut(dir).infer_isn(tcp.seq);
            self.stats.flows_partial += 1;
        }

        let dir = flow.direction(packet.src_ip, tcp.src_port);

        // Ensure ISN is set for this direction even on established flows
        // (e.g., server direction when only SYN was seen, not SYN+ACK).
        if flow.get_direction_mut(dir).isn.is_none() {
            flow.get_direction_mut(dir).infer_isn(tcp.seq);
        }

        let small_segment_max_bytes = self.config.small_segment_max_bytes;
        let flow_dir = flow.get_direction_mut(dir);
        let before_insert = flow_dir.buffered_bytes;
        let result = flow_dir.insert_segment(
            tcp.seq,
            payload,
            self.config.max_depth,
            self.config.max_segments_per_direction,
            self.config.max_receive_window,
        );
        debug_assert!(
            flow_dir.buffered_bytes >= before_insert,
            "insert_segment decreased buffered_bytes: before={} after={}",
            before_insert,
            flow_dir.buffered_bytes
        );
        let bytes_added = flow_dir.buffered_bytes.saturating_sub(before_insert);
        self.total_memory += bytes_added;

        // Maintain the consecutive small-segment run for this direction
        // (LESSON-P2.05). "Small" is a pure property of the payload
        // size, so it is classified here rather than inside the segment
        // buffer. The run reflects every segment that *reached the
        // reassembly window* — including duplicates and overlaps
        // (`Duplicate` / `PartialOverlap` / `ConflictingOverlap` /
        // `Truncated`), which are real received segments even though
        // they are not newly stored. Only segments turned away *before*
        // the window — out-of-window, segment-limit, depth-exceeded — and
        // the IsnMissing programming-error case are neutral, as are pure
        // ACKs (empty payload): none extend or reset the run. A
        // normal-sized data segment resets it: segmentation-evasion
        // shows up as a long *unbroken* run of tiny segments, whereas
        // benign interactive traffic interleaves them with normal-sized
        // segments.
        if !payload.is_empty()
            && !matches!(
                result,
                InsertResult::OutOfWindow
                    | InsertResult::SegmentLimitReached
                    | InsertResult::DepthExceeded
                    | InsertResult::IsnMissing
            )
        {
            if payload.len() < usize::from(small_segment_max_bytes) {
                flow_dir.small_segment_run = flow_dir.small_segment_run.saturating_add(1);
            } else {
                flow_dir.small_segment_run = 0;
            }
        }

        match result {
            InsertResult::Inserted => self.stats.segments_inserted += 1,
            InsertResult::Duplicate => self.stats.segments_duplicates += 1,
            InsertResult::PartialOverlap => {
                self.stats.segments_overlaps += 1;
                self.stats.segments_inserted += 1;
            }
            InsertResult::ConflictingOverlap => {
                self.stats.segments_overlaps += 1;
                self.generate_conflicting_overlap_finding(key, packet.src_ip);
            }
            InsertResult::Truncated => {
                self.stats.segments_inserted += 1;
                self.generate_truncated_finding(key, packet.src_ip);
            }
            InsertResult::DepthExceeded => {
                self.stats.segments_depth_exceeded += 1;
            }
            InsertResult::SegmentLimitReached => {
                self.stats.segments_segment_limit += 1;
                // Partial insertion: some gap bytes were inserted before the limit
                if bytes_added > 0 {
                    self.stats.segments_overlaps += 1;
                    self.stats.segments_inserted += 1;
                }
            }
            InsertResult::OutOfWindow => {
                self.stats.segments_out_of_window += 1;
            }
            InsertResult::IsnMissing => {
                // Programming error — ISN should always be set before insert.
                // eprintln already emitted in insert_segment.
            }
        }

        dir
    }

    /// Emit the per-direction overlap / small-segment / out-of-window
    /// Anomaly findings whose thresholds have just been crossed.
    ///
    /// LESSON-P1.01: the per-direction alert latches flip
    /// unconditionally once their threshold trips, even when the finding
    /// push is suppressed by the `MAX_FINDINGS` cap. This prevents
    /// re-evaluating the same threshold on every subsequent packet
    /// (which would also miscount as multiple `dropped_findings` rather
    /// than one) and lets the `dropped_findings` counter accurately
    /// reflect distinct anomalies lost to the cap.
    fn check_anomaly_thresholds(&mut self, packet: &ParsedPacket, key: &FlowKey, dir: Direction) {
        // Snapshot the configured thresholds before borrowing the flow
        // (LESSON-P2.05 — these are now `ReassemblyConfig` fields).
        let overlap_threshold = self.config.overlap_alert_threshold;
        let small_segment_threshold = self.config.small_segment_alert_threshold;
        let out_of_window_threshold = self.config.out_of_window_alert_threshold;

        let flow = self.flows.get_mut(key).unwrap();
        let flow_dir = flow.get_direction_mut(dir);

        if flow_dir.overlap_count > overlap_threshold && !flow_dir.overlap_alert_fired {
            flow_dir.overlap_alert_fired = true;
            if self.findings.len() < MAX_FINDINGS {
                self.findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Likely,
                    confidence: Confidence::Medium,
                    summary: format!(
                        "Excessive segment overlaps ({}) on flow {}",
                        flow_dir.overlap_count, key
                    ),
                    evidence: vec!["Possible evasion attempt".into()],
                    mitre_technique: Some("T1036".into()),
                    source_ip: Some(packet.src_ip),
                    timestamp: None,
                    direction: Some(dir),
                });
            } else {
                self.stats.dropped_findings += 1;
            }
        }
        // LESSON-P2.05 follow-up: a flow is exempt from small-segment
        // detection when EITHER endpoint port is in the configured
        // interactive-port ignore list (telnet, rlogin, ...) — those
        // protocols emit benign runs of tiny segments. The list scan is
        // the last `&&` term, so it runs only once the run has actually
        // crossed the threshold, not on every packet.
        if flow_dir.small_segment_run > small_segment_threshold
            && !flow_dir.small_segment_alert_fired
            && !self
                .config
                .small_segment_ignore_ports
                .iter()
                .any(|&p| p == key.lower_port() || p == key.upper_port())
        {
            flow_dir.small_segment_alert_fired = true;
            if self.findings.len() < MAX_FINDINGS {
                self.findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Inconclusive,
                    confidence: Confidence::Medium,
                    summary: format!(
                        "Excessive consecutive small segments ({}) on flow {}",
                        flow_dir.small_segment_run, key
                    ),
                    evidence: vec![
                        "Long unbroken run of undersized TCP segments; possible \
                         segmentation-based IDS evasion"
                            .into(),
                    ],
                    mitre_technique: None,
                    source_ip: Some(packet.src_ip),
                    timestamp: None,
                    direction: Some(dir),
                });
            } else {
                self.stats.dropped_findings += 1;
            }
        }
        if flow_dir.out_of_window_count > out_of_window_threshold
            && !flow_dir.out_of_window_alert_fired
        {
            flow_dir.out_of_window_alert_fired = true;
            let count = flow_dir.out_of_window_count;
            let window = self.config.max_receive_window;
            if self.findings.len() < MAX_FINDINGS {
                self.findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Inconclusive,
                    confidence: Confidence::Low,
                    summary: format!("Excessive out-of-window segments ({count}) on flow {key}"),
                    evidence: vec![format!(
                        "max_receive_window={window} bytes; possible misconfiguration, evasion, or capture corruption"
                    )],
                    mitre_technique: None,
                    source_ip: Some(packet.src_ip),
                    timestamp: None,
                    direction: Some(dir),
                });
            } else {
                self.stats.dropped_findings += 1;
            }
        }
    }

    /// Flush the now-contiguous prefix of the direction's buffer to the
    /// handler and update memory accounting.
    fn flush_contiguous_data(
        &mut self,
        key: &FlowKey,
        dir: Direction,
        handler: &mut dyn StreamHandler,
    ) {
        let flow = self.flows.get_mut(key).unwrap();
        let flow_dir = flow.get_direction_mut(dir);
        let before_flush = flow_dir.buffered_bytes;
        let flushed = flow_dir.flush_contiguous();
        self.total_memory -= before_flush - flow_dir.buffered_bytes;

        for (offset, data) in &flushed {
            self.stats.bytes_reassembled += data.len() as u64;
            handler.on_data(key, dir, data, *offset);
        }
    }

    /// Expire flows that have been idle longer than the configured timeout.
    pub fn expire_flows(&mut self, current_time: u32, handler: &mut dyn StreamHandler) {
        let timeout = self.config.flow_timeout_secs;
        let expired_keys: Vec<FlowKey> = self
            .flows
            .iter()
            .filter(|(_, flow)| {
                flow.state == FlowState::Closed
                    || (current_time > flow.last_seen && (current_time - flow.last_seen) > timeout)
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.stats.flows_expired += 1;
            self.close_flow(&key, CloseReason::Timeout, handler);
        }
    }

    /// Close all remaining flows (called at end of capture).
    /// Generates summary-level findings for notable reassembly events.
    /// Must only be called once; subsequent calls are no-ops.
    pub fn finalize(&mut self, handler: &mut dyn StreamHandler) {
        if self.finalized {
            return;
        }
        self.finalized = true;

        let all_keys: Vec<FlowKey> = self.flows.keys().cloned().collect();
        for key in all_keys {
            self.close_flow(&key, CloseReason::Timeout, handler);
        }

        // Generate summary-level finding for segment limit hits.
        // Pushed unconditionally (at most 1 finding) to avoid being silently
        // dropped when per-flow findings have filled the MAX_FINDINGS cap.
        let count = self.stats.segments_segment_limit;
        if count > 0 {
            self.findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Inconclusive,
                confidence: Confidence::Medium,
                summary: format!(
                    "{count} segment{} dropped due to per-flow segment count limit",
                    plural_s(count),
                ),
                evidence: vec![
                    "Segment count limit prevents BTreeMap overhead explosion".into(),
                    "May indicate segmentation-based evasion attempt".into(),
                ],
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }
    }

    /// Return a reference to current stats.
    pub fn stats(&self) -> &ReassemblyStats {
        &self.stats
    }

    /// Return any anomaly findings generated during reassembly.
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    /// Returns `true` once [`Self::finalize`] has been called. Exposed so
    /// callers and tests can observe the lifecycle invariant that
    /// `impl Drop` defends.
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Return the current total memory used by all flow buffers.
    pub fn total_memory(&self) -> usize {
        self.total_memory
    }

    /// Produce an AnalysisSummary for the reassembly engine stats.
    ///
    /// LESSON-P2.09: the returned `AnalysisSummary::detail` is a
    /// `BTreeMap`, so the JSON output's analyzer-summary section is
    /// alphabetically ordered and deterministic across runs.
    pub fn summarize(&self) -> AnalysisSummary {
        let mut detail: std::collections::BTreeMap<String, serde_json::Value> =
            std::collections::BTreeMap::new();
        let s = &self.stats;
        detail.insert("packets_processed".into(), s.packets_processed.into());
        detail.insert(
            "packets_skipped_non_tcp".into(),
            s.packets_skipped_non_tcp.into(),
        );
        detail.insert("flows_total".into(), s.flows_total.into());
        detail.insert("flows_partial".into(), s.flows_partial.into());
        detail.insert("flows_fin".into(), s.flows_fin.into());
        detail.insert("flows_rst".into(), s.flows_rst.into());
        detail.insert("flows_completed".into(), (s.flows_fin + s.flows_rst).into());
        detail.insert("flows_expired".into(), s.flows_expired.into());
        detail.insert("evictions".into(), s.evictions.into());
        detail.insert("segments_inserted".into(), s.segments_inserted.into());
        detail.insert("segments_duplicates".into(), s.segments_duplicates.into());
        detail.insert("segments_overlaps".into(), s.segments_overlaps.into());
        detail.insert(
            "segments_out_of_window".into(),
            s.segments_out_of_window.into(),
        );
        detail.insert(
            "segments_segment_limit".into(),
            s.segments_segment_limit.into(),
        );
        detail.insert(
            "segments_depth_exceeded".into(),
            s.segments_depth_exceeded.into(),
        );
        detail.insert("bytes_reassembled".into(), s.bytes_reassembled.into());
        detail.insert("dropped_findings".into(), s.dropped_findings.into());
        AnalysisSummary {
            analyzer_name: "TCP Reassembly".into(),
            packets_analyzed: s.packets_tcp,
            detail,
        }
    }
}

/// Lifecycle tripwire: warn (once per process) if a reassembler is dropped
/// without [`TcpReassembler::finalize`] having been called.
///
/// `finalize` requires a `&mut dyn StreamHandler`, which `Drop::drop`
/// cannot accept, so this `impl Drop` cannot itself flush pending flows
/// or emit summary-level findings. What it *can* do is make the
/// "forgot-to-finalize" bug loud at runtime, including on unwind from
/// a `?`-propagated `Err` further up the call stack — which is the
/// failure mode that closes LESSON-P0.03 in the brownfield-ingest
/// synthesis (architecture smell #9, "no-Drop / finalize-fragile").
///
/// The companion to this tripwire is the `run_analyze` control flow in
/// `src/main.rs`, which wraps the fallible per-target loop so that
/// `finalize` is reached before any `Err` escapes the function. The two
/// pieces together — guaranteed finalize on the happy path + a noisy
/// runtime check for regressions — replace the original silent skip.
impl Drop for TcpReassembler {
    fn drop(&mut self) {
        if !self.finalized && !FINALIZE_SKIPPED_WARNED.swap(true, Ordering::Relaxed) {
            eprintln!(
                "wirerust: TcpReassembler dropped without calling finalize() \
                 — {} flow(s) and {} byte(s) of buffered segment state were \
                 discarded without producing summary-level findings. This \
                 indicates a control-flow bug; further occurrences in this \
                 process will be suppressed.",
                self.flows.len(),
                self.total_memory,
            );
        }
    }
}
