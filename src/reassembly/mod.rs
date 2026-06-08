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
    /// Last timestamp at which an expiry sweep was performed.
    ///
    /// `expire_flows` is called from `process_packet` only when `timestamp`
    /// has advanced past this value, limiting the O(n) sweep to at most once
    /// per unique second of stream time while ensuring no idle flow escapes
    /// expiry for longer than one sweep period (BC-2.04.013 v1.5 PC0).
    last_expiry_sweep_secs: u32,
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
            last_expiry_sweep_secs: 0,
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

        // BC-2.04.013 v1.5 PC0 — idle-flow expiry wiring.
        //
        // Sweep whenever stream time advances past the last sweep timestamp.
        // This limits the O(n) scan to at most once per unique second of
        // capture time (effectively once per second of real traffic), while
        // guaranteeing that a flow idle for exactly (timeout + 1) seconds is
        // caught by the next packet that arrives at any later timestamp.
        //
        // Uses expire_idle_by_timeout (time-based only) rather than the public
        // expire_flows (which also removes FlowState::Closed flows). process_packet
        // already handles Closed-state removal inline after FIN processing, so the
        // hot-path sweep must not also trigger the Closed-state branch of expire_flows
        // — doing so would prematurely remove flows that are Closed only because of
        // test seams or FIN-partial paths, breaking the eviction-order invariant
        // (BC-2.04.017). expire_flows remains correct as a direct-call API for
        // scenarios that need both cleanup modes (e.g. offline tools, tests).
        if timestamp > self.last_expiry_sweep_secs {
            self.last_expiry_sweep_secs = timestamp;
            self.expire_idle_by_timeout(timestamp, handler);
        }

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
            self.flush_contiguous_data(&key, dir, handler, timestamp);
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
    ///
    /// `timestamp` is the current packet's `timestamp_secs` (BC-2.04.055 hot-path
    /// case): the pcap capture-relative Unix epoch seconds of the packet that
    /// triggered this flush.
    fn flush_contiguous_data(
        &mut self,
        key: &FlowKey,
        dir: Direction,
        handler: &mut dyn StreamHandler,
        timestamp: u32,
    ) {
        let flow = self.flows.get_mut(key).unwrap();
        let flow_dir = flow.get_direction_mut(dir);
        let before_flush = flow_dir.buffered_bytes;
        let flushed = flow_dir.flush_contiguous();
        self.total_memory -= before_flush - flow_dir.buffered_bytes;

        for (offset, data) in &flushed {
            self.stats.bytes_reassembled += data.len() as u64;
            handler.on_data(key, dir, data, *offset, timestamp);
        }
    }

    /// Expire flows that have been idle longer than the configured timeout,
    /// using only the time-based condition (strict greater-than).
    ///
    /// This is the hot-path variant called from [`Self::process_packet`]. It
    /// deliberately omits the `FlowState::Closed` OR-clause that the public
    /// [`Self::expire_flows`] includes, because `process_packet` already handles
    /// `Closed`-state removal inline after FIN processing. Applying the Closed
    /// clause here would prematurely remove flows that are Closed only via test
    /// seams or FIN-partial paths, violating the eviction-order invariant
    /// (BC-2.04.017). The public `expire_flows` retains both clauses for
    /// direct-call use cases (offline tools, manual lifecycle management).
    fn expire_idle_by_timeout(&mut self, current_time: u32, handler: &mut dyn StreamHandler) {
        let timeout = self.config.flow_timeout_secs;
        let expired_keys: Vec<FlowKey> = self
            .flows
            .iter()
            .filter(|(_, flow)| {
                current_time > flow.last_seen && (current_time - flow.last_seen) > timeout
            })
            .map(|(key, _)| key.clone())
            .collect();

        for key in expired_keys {
            self.stats.flows_expired += 1;
            self.close_flow(&key, CloseReason::Timeout, handler);
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

    /// Return the number of flows currently tracked.
    ///
    /// Exposed for testing so AC-012 and similar tests can assert the flow
    /// table is empty without needing access to the private `flows` field.
    pub fn flow_count(&self) -> usize {
        self.flows.len()
    }

    /// Test-only: return the sum of memory_used() across all flows.
    /// Used by test_BC_2_04_014_total_memory_equals_sum_of_flow_memory
    /// to assert the total_memory invariant.
    ///
    /// This seam observes the BC-2.04.014 invariant:
    ///   total_memory == sum(flow.memory_used() for all flows)
    /// without breaking encapsulation of the private `flows` field.
    #[doc(hidden)]
    pub fn flows_memory_sum_for_testing(&self) -> usize {
        self.flows.values().map(|f| f.memory_used()).sum()
    }

    /// Mutable access to the flow table for test-only seams in `lifecycle.rs`.
    ///
    /// `pub(crate)` keeps this invisible outside the crate; the production
    /// API is unchanged. Required by `force_set_flow_state_for_testing` in
    /// `lifecycle.rs` (ADR-0004 amendment, opt-in-per-guard doctrine).
    pub(crate) fn flows_mut(&mut self) -> &mut HashMap<FlowKey, TcpFlow> {
        &mut self.flows
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

// Kani formal-verification harnesses for VP-003 (MAX_FINDINGS cap with finalize
// bypass, BC-2.04.024 / BC-2.04.054). Gated behind `#[cfg(kani)]` (set only by
// `cargo kani`), so invisible to the normal build/test/clippy pipeline.
//
// DESIGN NOTE: these proofs reason over a symbolic `findings.len()` (a `usize`)
// rather than constructing a real `TcpReassembler`. `TcpReassembler::new`
// builds a `HashMap<FlowKey, TcpFlow>` and a `ReassemblyConfig` holding `Vec`s;
// CBMC's model of the std `HashMap`/allocator leaves its checks UNDETERMINED
// (and slow), which is the same std-collection limitation that makes BTreeMap
// proofs intractable. The VP-003 cap invariant is a pure property of
// `findings.len()` versus `MAX_FINDINGS` and `stats.dropped_findings` — it does
// not depend on flow state or finding contents. Each proof body is therefore a
// byte-for-byte transcription of the production guard / bypass idiom operating
// on a symbolic length, which is sound (the production guards read nothing but
// `len()`) and fast. The end-to-end `Vec<Finding>` glue is exercised by the
// (green) reassembly test-suite, which includes explicit MAX_FINDINGS-cap tests.
//
// The guard appears in two logically equivalent spellings across the five
// emission sites: a positive form in mod.rs (`if len < MAX_FINDINGS { push }
// else { dropped += 1 }`) and an early-return form in lifecycle.rs
// (`if len >= MAX_FINDINGS { dropped += 1; return } ... push`). Both partition
// on `len < MAX_FINDINGS` with identical effect, so a single transcription
// covers all five.
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-003 invariants 1, 3, 4: a single guarded emission can never push the
    /// length above MAX_FINDINGS, and a blocked push increments `dropped_findings`.
    ///
    /// Proved INDUCTIVELY (loop-free): start from a symbolic length already
    /// satisfying the invariant (`current_len <= MAX_FINDINGS`), apply ONE
    /// guarded emission, and show the invariant is preserved. The five
    /// production guard sites use two LOGICALLY EQUIVALENT spellings:
    ///   - mod.rs:461/495/524 — positive form
    ///     `if findings.len() < MAX_FINDINGS { push } else { dropped += 1 }`
    ///   - lifecycle.rs:101/121 — early-return form
    ///     `if findings.len() >= MAX_FINDINGS { dropped += 1; return } ... push`
    /// Both partition on `len < MAX_FINDINGS` and have identical effect on
    /// `(len, dropped)`; the transcription below models that common effect. All
    /// sites only ever run on a vector already satisfying the invariant, so this
    /// single inductive step proves the invariant across any number of emissions.
    ///
    /// Bounded domain: `current_len` symbolic over the whole valid range
    /// [0, MAX_FINDINGS]. The body is an exact transcription of the guard idiom.
    #[kani::proof]
    fn verify_guarded_push_preserves_cap() {
        let current_len: usize = kani::any();
        kani::assume(current_len <= MAX_FINDINGS); // invariant holds on entry
        let dropped_before: usize = kani::any();
        kani::assume(dropped_before < usize::MAX); // avoid spurious counter overflow

        // Exact transcription of the production guard arithmetic.
        let (new_len, new_dropped) = if current_len < MAX_FINDINGS {
            (current_len + 1, dropped_before)
        } else {
            (current_len, dropped_before + 1)
        };

        // Cap invariant preserved after the guarded emission.
        assert!(new_len <= MAX_FINDINGS);

        if current_len < MAX_FINDINGS {
            assert!(new_len == current_len + 1);
            assert!(new_dropped == dropped_before);
        } else {
            // At the cap: push suppressed, exactly one drop counted.
            assert!(new_len == current_len);
            assert!(new_dropped == dropped_before + 1);
        }
    }

    /// VP-003 invariants 2 & 4: the finalize bypass (mod.rs:630) is an
    /// UNCONDITIONAL single push with no cap guard, so it adds exactly one
    /// finding and is the only path that may produce `len == MAX_FINDINGS + 1`.
    ///
    /// Transcription of the bypass: from any symbolic length already within the
    /// invariant, one unconditional `+1` is applied. We show the post-bypass
    /// length is at most `MAX_FINDINGS + 1`, with equality exactly when the
    /// bypass fires at the cap — establishing the `+1` ceiling.
    #[kani::proof]
    fn verify_finalize_bypass_adds_at_most_one() {
        let current_len: usize = kani::any();
        kani::assume(current_len <= MAX_FINDINGS); // invariant holds before finalize

        // The bypass push is unconditional (no `if len < MAX_FINDINGS`).
        let after = current_len + 1;

        // Post-finalize ceiling: never exceeds MAX_FINDINGS + 1.
        assert!(after <= MAX_FINDINGS + 1);
        // The ceiling is reachable ONLY when the bypass fires at exactly the cap.
        assert!((after == MAX_FINDINGS + 1) == (current_len == MAX_FINDINGS));
    }

    /// VP-003 invariant 1 (loop form, below the cap): a run of guarded emissions
    /// from empty never exceeds the cap. The guard is memoryless (its branch
    /// depends only on the current length), so a small bounded loop, together
    /// with the inductive proof above, covers the full range. Models the length
    /// with a plain counter applying the exact guard.
    ///
    /// NOTE: with `n <= 6` and `MAX_FINDINGS == 10_000` this loop never reaches
    /// the cap, so the final `len == n` is the "no drop fired" arm. The drop
    /// path at the boundary is exercised by
    /// `verify_guarded_push_loop_saturates_at_cap` below, which starts at the
    /// cap minus one.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_guarded_push_loop_never_exceeds_cap() {
        let n: usize = kani::any();
        kani::assume(n <= 6);

        let mut len: usize = 0;
        for _ in 0..n {
            // Exact guard idiom: push only while under the cap.
            if len < MAX_FINDINGS {
                len += 1;
            }
            assert!(len <= MAX_FINDINGS);
        }
        // Below the cap throughout: no emission suppressed, so len == n.
        assert!(len == n);
    }

    /// VP-003 invariant 1 + 3 (loop form, AT the cap boundary): a run of guarded
    /// emissions that begins one below the cap saturates at exactly MAX_FINDINGS
    /// and counts every subsequent suppressed emission as a dropped finding.
    /// This formally exercises the boundary the below-cap loop cannot reach: the
    /// guard's drop arm (`else { dropped += 1 }`) actually fires here.
    ///
    /// Bounded domain: start `len = MAX_FINDINGS - 1`, run `n` guarded emissions
    /// for symbolic `n` in [1, 6]. The first emission lifts `len` to the cap;
    /// every later one is suppressed, so `dropped == n - 1`. `n <= 6` keeps the
    /// unwind cheap; the guard is memoryless so any `n >= 1` exhibits the same
    /// saturation behavior, making the small bound representative.
    #[kani::proof]
    #[kani::unwind(8)]
    fn verify_guarded_push_loop_saturates_at_cap() {
        let n: usize = kani::any();
        kani::assume((1..=6).contains(&n));

        let mut len: usize = MAX_FINDINGS - 1;
        let mut dropped: usize = 0;
        for _ in 0..n {
            // Exact guard idiom, including the drop arm.
            if len < MAX_FINDINGS {
                len += 1;
            } else {
                dropped += 1;
            }
            // Never exceeds the cap even once saturated.
            assert!(len <= MAX_FINDINGS);
        }
        // First emission reached the cap; all n-1 later ones were suppressed.
        assert!(len == MAX_FINDINGS);
        assert!(dropped == n - 1);
    }
}

// ---- Test-only seams (STORY-021 / ADR-0004 amendment) ----------------------
//
// These seams expose process-global atomics and private engine state to
// integration tests without widening the production API. All follow the
// `#[doc(hidden)] pub fn` append pattern established by STORY-014 / STORY-019.
// MUST NOT be called from production code.

impl TcpReassembler {
    /// Test-only accessor for the process-global `FINALIZE_SKIPPED_WARNED` flag.
    ///
    /// Exposes a read of [`FINALIZE_SKIPPED_WARNED`] so integration tests can
    /// verify the one-shot Drop tripwire behavior defined by BC-2.04.012 EC-006:
    /// the flag transitions `false → true` exactly once per process lifetime when
    /// an un-finalized reassembler is dropped. Sibling to
    /// `wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing`.
    #[doc(hidden)]
    pub fn finalize_skipped_warned_for_testing() -> bool {
        FINALIZE_SKIPPED_WARNED.load(Ordering::Relaxed)
    }

    /// Test-only reset of the process-global `FINALIZE_SKIPPED_WARNED` flag.
    ///
    /// Allows tests to deterministically observe the BC-2.04.012 EC-006
    /// `false → true` swap transition by resetting the atomic before each
    /// relevant test. MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn reset_finalize_skipped_warned_for_testing() {
        FINALIZE_SKIPPED_WARNED.store(false, Ordering::Relaxed);
    }

    /// Test-only swap of the process-global `FINALIZE_SKIPPED_WARNED` flag.
    ///
    /// Mirrors [`AtomicBool::swap`] semantics so callers can read the current
    /// value and reset it atomically in one operation. Used by AC-004
    /// (`test_BC_2_04_012_drop_without_finalize_emits_warning`) for the
    /// reset → drop → swap observation pattern.
    ///
    /// **Scope limitation (per STORY-021 Architecture Compliance Rules,
    /// Option B):** This seam does NOT provide unique per-Drop attribution
    /// under cargo's parallel test scheduler. The `reassembly_engine_tests`
    /// binary contains ~130+ sites that drop un-finalized `TcpReassembler`
    /// instances without holding `FINALIZE_SKIPPED_WARNED_LOCK`. A `true`
    /// return value confirms the production `Drop` hook fired **somewhere
    /// in this process**, not specifically at the caller's drop site. The
    /// test reliably detects total removal of the Drop hook in a fresh
    /// single-binary process; unique attribution requires `--test-threads=1`
    /// or stderr capture.
    ///
    /// MUST NOT be called from production code.
    #[doc(hidden)]
    pub fn swap_finalize_skipped_warned_for_testing(value: bool) -> bool {
        FINALIZE_SKIPPED_WARNED.swap(value, Ordering::Relaxed)
    }

    /// Test-only setter for `stats.segments_segment_limit`.
    ///
    /// Directly injects a segment-limit counter value so tests can exercise
    /// the finalize segment-limit summary path (BC-2.04.025, BC-2.04.026,
    /// BC-2.04.054) without constructing adversarial packet sequences to
    /// overflow `max_segments_per_direction`. Production behavior is unchanged:
    /// the counter is still only incremented by the packet-processing hot path.
    ///
    /// NOTE (F-W11P1-009 / BC-2.04.026 EC-002): This seam bypasses the engine's
    /// normal increment path (`process_packet` → `SegmentLimitReached`). It does
    /// NOT violate BC-2.04.026 EC-002, which forbids increments *during* finalize;
    /// this seam sets the counter *before* finalize is called. The trust boundary
    /// is enforced by convention (`#[doc(hidden)]` + `_for_testing` suffix);
    /// production code MUST NOT call this.
    #[doc(hidden)]
    pub fn set_segments_segment_limit_for_testing(&mut self, count: u64) {
        self.stats.segments_segment_limit = count;
    }

    /// Test-only direct push of a [`Finding`] into the engine's findings vec.
    ///
    /// Bypasses all cap guards so tests can pre-fill the findings vec to
    /// arbitrary lengths (e.g. exactly `MAX_FINDINGS - 1` or `MAX_FINDINGS`)
    /// to exercise the boundary conditions in BC-2.04.024 and BC-2.04.054
    /// without running O(10_000) packet-processing iterations.
    ///
    /// This seam does NOT update `stats.dropped_findings`; it pushes
    /// unconditionally, mirroring the finalize segment-limit bypass path.
    #[doc(hidden)]
    pub fn push_finding_for_testing(&mut self, finding: Finding) {
        self.findings.push(finding);
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
