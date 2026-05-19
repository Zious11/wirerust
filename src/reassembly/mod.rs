//! TCP stream reassembly engine.
//!
//! Owns the [`TcpReassembler`] type plus the per-flow ([`flow`]) and
//! per-segment ([`segment`]) state, and the [`handler::StreamHandler`] /
//! [`handler::StreamAnalyzer`] interfaces that downstream protocol
//! analyzers (HTTP, TLS) implement.
//!
//! Design highlights:
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

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::analyzer::AnalysisSummary;
use crate::decoder::{ParsedPacket, Protocol, TransportInfo};
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::{FlowKey, FlowState, TcpFlow};
use crate::reassembly::handler::{CloseReason, StreamHandler};
use crate::reassembly::segment::InsertResult;

const OVERLAP_ALERT_THRESHOLD: u32 = 50;
const SMALL_SEGMENT_ALERT_THRESHOLD: u32 = 2048;
const OUT_OF_WINDOW_ALERT_THRESHOLD: u32 = 100;
const MAX_FINDINGS: usize = 10_000;

static CLOSE_FLOW_MISSING_WARNED: AtomicBool = AtomicBool::new(false);
static FINALIZE_SKIPPED_WARNED: AtomicBool = AtomicBool::new(false);

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
    pub fn process_packet(
        &mut self,
        packet: &ParsedPacket,
        timestamp: u32,
        handler: &mut dyn StreamHandler,
    ) {
        self.stats.packets_processed += 1;

        // 1. Skip non-TCP packets
        if packet.protocol != Protocol::Tcp {
            self.stats.packets_skipped_non_tcp += 1;
            return;
        }

        // 2. Extract TCP fields
        let (src_port, dst_port, seq, syn, ack, fin, rst) = match &packet.transport {
            TransportInfo::Tcp {
                src_port,
                dst_port,
                seq_number,
                syn,
                ack,
                fin,
                rst,
            } => (*src_port, *dst_port, *seq_number, *syn, *ack, *fin, *rst),
            _ => return,
        };

        self.stats.packets_tcp += 1;

        // 3. Build the flow key
        let key = FlowKey::new(packet.src_ip, src_port, packet.dst_ip, dst_port);

        // 4. Get or create flow
        if !self.flows.contains_key(&key) {
            // Enforce max_flows limit
            if self.flows.len() >= self.config.max_flows {
                self.evict_flows(handler);
                if self.flows.len() >= self.config.max_flows {
                    // Still at capacity after eviction — drop this packet
                    return;
                }
            }
            let flow = TcpFlow::new(key.clone(), timestamp);
            self.flows.insert(key.clone(), flow);
            self.stats.flows_total += 1;
        }

        // Work with the flow
        let flow = self.flows.get_mut(&key).unwrap();
        flow.last_seen = timestamp;

        // 5. Handle SYN (without ACK) -- client initiating
        if syn && !ack {
            flow.set_initiator(packet.src_ip, src_port);
            let dir = flow.direction(packet.src_ip, src_port);
            flow.get_direction_mut(dir).set_isn(seq);
            flow.on_syn();
        }

        // 6. Handle SYN+ACK -- server responding
        if syn && ack {
            // The responder is sending SYN+ACK, so the initiator is the *destination*
            flow.set_initiator(packet.dst_ip, dst_port);
            let dir = flow.direction(packet.src_ip, src_port);
            flow.get_direction_mut(dir).set_isn(seq);
            flow.on_syn_ack();
        }

        // 7. Handle RST — flush salvageable data, close, and remove
        if rst {
            flow.on_rst();
            self.stats.flows_rst += 1;
            self.close_flow(&key, CloseReason::Rst, handler);
            return;
        }

        // 8. Handle FIN
        if fin {
            let dir = flow.direction(packet.src_ip, src_port);
            flow.get_direction_mut(dir).fin_seen = true;
            flow.on_fin();
            // Note: if state is now Closed (both FINs seen), the flow will be
            // removed after payload processing below (step 10).
        }

        // 9. Handle payload
        let payload = &packet.payload;
        if !payload.is_empty() {
            // If no SYN was seen (mid-stream join), infer state
            if flow.state == FlowState::New {
                flow.on_data_without_syn();
                flow.set_initiator(packet.src_ip, src_port);
                let dir = flow.direction(packet.src_ip, src_port);
                flow.get_direction_mut(dir).infer_isn(seq);
                self.stats.flows_partial += 1;
            }

            let dir = flow.direction(packet.src_ip, src_port);

            // Ensure ISN is set for this direction even on established flows
            // (e.g., server direction when only SYN was seen, not SYN+ACK)
            if flow.get_direction_mut(dir).isn.is_none() {
                flow.get_direction_mut(dir).infer_isn(seq);
            }

            let flow_dir = flow.get_direction_mut(dir);
            let before_insert = flow_dir.buffered_bytes;
            let result = flow_dir.insert_segment(
                seq,
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

            match result {
                InsertResult::Inserted => self.stats.segments_inserted += 1,
                InsertResult::Duplicate => self.stats.segments_duplicates += 1,
                InsertResult::PartialOverlap => {
                    self.stats.segments_overlaps += 1;
                    self.stats.segments_inserted += 1;
                }
                InsertResult::ConflictingOverlap => {
                    self.stats.segments_overlaps += 1;
                    self.generate_conflicting_overlap_finding(&key, packet.src_ip);
                }
                InsertResult::Truncated => {
                    self.stats.segments_inserted += 1;
                    self.generate_truncated_finding(&key, packet.src_ip);
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

            // Check anomaly thresholds on the direction
            let flow = self.flows.get_mut(&key).unwrap();
            let flow_dir = flow.get_direction_mut(dir);
            // LESSON-P1.01: the per-direction alert latches now flip
            // unconditionally once their threshold trips, even when the
            // finding push is suppressed by the MAX_FINDINGS cap. This
            // prevents re-evaluating the same threshold on every
            // subsequent packet (which would also miscount as multiple
            // dropped_findings rather than one) and lets the
            // dropped_findings counter accurately reflect distinct
            // anomalies lost to the cap.
            if flow_dir.overlap_count > OVERLAP_ALERT_THRESHOLD && !flow_dir.overlap_alert_fired {
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
                    });
                } else {
                    self.stats.dropped_findings += 1;
                }
            }
            if flow_dir.small_segment_count > SMALL_SEGMENT_ALERT_THRESHOLD
                && !flow_dir.small_segment_alert_fired
            {
                flow_dir.small_segment_alert_fired = true;
                if self.findings.len() < MAX_FINDINGS {
                    self.findings.push(Finding {
                        category: ThreatCategory::Anomaly,
                        verdict: Verdict::Inconclusive,
                        confidence: Confidence::Medium,
                        summary: format!(
                            "Excessive small segments ({}) on flow {}",
                            flow_dir.small_segment_count, key
                        ),
                        evidence: vec!["Possible IDS evasion".into()],
                        mitre_technique: None,
                        source_ip: Some(packet.src_ip),
                        timestamp: None,
                    });
                } else {
                    self.stats.dropped_findings += 1;
                }
            }
            if flow_dir.out_of_window_count > OUT_OF_WINDOW_ALERT_THRESHOLD
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
                        summary: format!(
                            "Excessive out-of-window segments ({}) on flow {}",
                            count, key
                        ),
                        evidence: vec![format!(
                            "max_receive_window={} bytes; possible misconfiguration, evasion, or capture corruption",
                            window
                        )],
                        mitre_technique: None,
                        source_ip: Some(packet.src_ip),
                        timestamp: None,
                    });
                } else {
                    self.stats.dropped_findings += 1;
                }
            }

            // Flush contiguous data
            let flow = self.flows.get_mut(&key).unwrap();
            let flow_dir = flow.get_direction_mut(dir);
            let before_flush = flow_dir.buffered_bytes;
            let flushed = flow_dir.flush_contiguous();
            self.total_memory -= before_flush - flow_dir.buffered_bytes;

            for (offset, data) in &flushed {
                self.stats.bytes_reassembled += data.len() as u64;
                handler.on_data(&key, dir, data, *offset);
            }
        }

        // 10. Remove FIN-closed flows after processing their final payload
        if self
            .flows
            .get(&key)
            .is_some_and(|f| f.state == FlowState::Closed)
        {
            self.stats.flows_fin += 1;
            self.close_flow(&key, CloseReason::Fin, handler);
        }

        // 12. Evict flows if memcap exceeded
        if self.total_memory > self.config.memcap {
            self.evict_flows(handler);
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
                    "{} segment{} dropped due to per-flow segment count limit",
                    count,
                    if count == 1 { "" } else { "s" }
                ),
                evidence: vec![
                    "Segment count limit prevents BTreeMap overhead explosion".into(),
                    "May indicate segmentation-based evasion attempt".into(),
                ],
                mitre_technique: None,
                source_ip: None,
                timestamp: None,
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
    pub fn summarize(&self) -> AnalysisSummary {
        let mut detail = HashMap::new();
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

    // --- Private helpers ---

    /// Flush remaining contiguous data in both directions, remove the flow,
    /// update memory accounting, and notify the handler.
    fn close_flow(&mut self, key: &FlowKey, reason: CloseReason, handler: &mut dyn StreamHandler) {
        use crate::reassembly::handler::Direction;
        let Some(mut flow) = self.flows.remove(key) else {
            debug_assert!(false, "close_flow called for non-existent key: {}", key);
            if !CLOSE_FLOW_MISSING_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "wirerust: close_flow called for non-existent key: {} (reason: {:?})",
                    key, reason
                );
            }
            return;
        };
        let flow_mem = flow.memory_used();
        for dir in [Direction::ClientToServer, Direction::ServerToClient] {
            let flow_dir = flow.get_direction_mut(dir);
            let flushed = flow_dir.flush_contiguous();
            for (offset, data) in &flushed {
                self.stats.bytes_reassembled += data.len() as u64;
                handler.on_data(key, dir, data, *offset);
            }
        }
        self.total_memory -= flow_mem;
        handler.on_flow_close(key, reason);
    }

    /// Evict flows when memcap is exceeded.
    /// Strategy: evict non-established flows first (sorted by LRU),
    /// then established flows by LRU.
    fn evict_flows(&mut self, handler: &mut dyn StreamHandler) {
        // Sort once, then evict from the sorted list until under memcap
        let mut candidates: Vec<(FlowKey, bool, u32)> = self
            .flows
            .iter()
            .map(|(key, flow)| {
                let is_established = flow.state == FlowState::Established;
                (key.clone(), is_established, flow.last_seen)
            })
            .collect();

        // Sort: non-established first, then by oldest last_seen
        candidates.sort_by(|a, b| {
            a.1.cmp(&b.1) // false (non-established) < true (established)
                .then(a.2.cmp(&b.2)) // older first
        });

        for (key, _, _) in &candidates {
            if self.total_memory <= self.config.memcap && self.flows.len() <= self.config.max_flows
            {
                break;
            }
            self.stats.evictions += 1;
            self.close_flow(key, CloseReason::MemoryPressure, handler);
        }
    }

    fn generate_conflicting_overlap_finding(&mut self, key: &FlowKey, src_ip: std::net::IpAddr) {
        if self.findings.len() >= MAX_FINDINGS {
            self.stats.dropped_findings += 1;
            return;
        }
        self.findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: format!("Conflicting TCP segment overlap on flow {}", key),
            evidence: vec!["Retransmitted segment contains different data".to_string()],
            mitre_technique: Some("T1036".to_string()),
            source_ip: Some(src_ip),
            timestamp: None,
        });
    }

    fn generate_truncated_finding(&mut self, key: &FlowKey, src_ip: std::net::IpAddr) {
        if self.findings.len() >= MAX_FINDINGS {
            self.stats.dropped_findings += 1;
            return;
        }
        self.findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: format!("Stream depth exceeded on flow {}", key),
            evidence: vec![format!("Max depth {} bytes reached", self.config.max_depth)],
            mitre_technique: None,
            source_ip: Some(src_ip),
            timestamp: None,
        });
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
