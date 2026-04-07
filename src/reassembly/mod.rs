pub mod flow;
pub mod handler;
pub mod segment;

use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::decoder::{ParsedPacket, Protocol, TransportInfo};
use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::{FlowKey, FlowState, TcpFlow};
use crate::reassembly::handler::{CloseReason, StreamHandler};
use crate::reassembly::segment::InsertResult;

const OVERLAP_ALERT_THRESHOLD: u32 = 50;
const SMALL_SEGMENT_ALERT_THRESHOLD: u32 = 2048;
const MAX_FINDINGS: usize = 10_000;

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
            }

            // Check anomaly thresholds on the direction
            let flow = self.flows.get_mut(&key).unwrap();
            let flow_dir = flow.get_direction_mut(dir);
            if flow_dir.overlap_count > OVERLAP_ALERT_THRESHOLD
                && !flow_dir.overlap_alert_fired
                && self.findings.len() < MAX_FINDINGS
            {
                flow_dir.overlap_alert_fired = true;
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
            }
            if flow_dir.small_segment_count > SMALL_SEGMENT_ALERT_THRESHOLD
                && !flow_dir.small_segment_alert_fired
                && self.findings.len() < MAX_FINDINGS
            {
                flow_dir.small_segment_alert_fired = true;
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
            eprintln!("wirerust: close_flow called for non-existent key: {}", key);
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
