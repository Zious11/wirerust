//! Flow-lifecycle internals for the reassembly engine.
//!
//! Houses the [`crate::reassembly::TcpReassembler`] methods that *retire*
//! flows or *emit segment-anomaly findings* — operations invoked from the
//! engine core in `mod.rs` but conceptually separate from the per-packet
//! hot path:
//!
//! - `close_flow` — flush a single flow's contiguous data, notify the
//!   handler, and remove the flow.
//! - `evict_flows` — shed flows (LRU, non-established first) when the
//!   memcap or `max_flows` ceiling is exceeded.
//! - `generate_conflicting_overlap_finding` / `generate_truncated_finding`
//!   — emit the two segment-level Anomaly findings.
//!
//! All four are `pub(super)` so the engine core can call them. This does
//! NOT widen the crate-facing API: `pub(super)` exposes them only to the
//! parent `reassembly` module, exactly as the original private `fn`s were
//! visible to the single combined `mod.rs`. Extracted for LESSON-P2.01.

use std::sync::atomic::{AtomicBool, Ordering};

use crate::findings::{Confidence, Finding, ThreatCategory, Verdict};
use crate::reassembly::flow::{FlowKey, FlowState};
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};

use super::{MAX_FINDINGS, TcpReassembler};

/// One-shot guard: a `close_flow` call for a key not present in the flow
/// table is a programming error. It warns at most once per process so a
/// recurring bug does not flood stderr.
static CLOSE_FLOW_MISSING_WARNED: AtomicBool = AtomicBool::new(false);

impl TcpReassembler {
    /// Flush remaining contiguous data in both directions, remove the flow,
    /// update memory accounting, and notify the handler.
    pub(super) fn close_flow(
        &mut self,
        key: &FlowKey,
        reason: CloseReason,
        handler: &mut dyn StreamHandler,
    ) {
        let Some(mut flow) = self.flows.remove(key) else {
            debug_assert!(false, "close_flow called for non-existent key: {key}");
            if !CLOSE_FLOW_MISSING_WARNED.swap(true, Ordering::Relaxed) {
                eprintln!(
                    "wirerust: close_flow called for non-existent key: {key} (reason: {reason:?})"
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
    pub(super) fn evict_flows(&mut self, handler: &mut dyn StreamHandler) {
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

    /// Emit the "conflicting TCP segment overlap" Anomaly finding (or
    /// count it as dropped if the `MAX_FINDINGS` cap is already hit).
    pub(super) fn generate_conflicting_overlap_finding(
        &mut self,
        key: &FlowKey,
        src_ip: std::net::IpAddr,
    ) {
        if self.findings.len() >= MAX_FINDINGS {
            self.stats.dropped_findings += 1;
            return;
        }
        self.findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: format!("Conflicting TCP segment overlap on flow {key}"),
            evidence: vec!["Retransmitted segment contains different data".to_string()],
            mitre_technique: Some("T1036".to_string()),
            source_ip: Some(src_ip),
            timestamp: None,
            direction: None,
        });
    }

    /// Emit the "stream depth exceeded" Anomaly finding (or count it as
    /// dropped if the `MAX_FINDINGS` cap is already hit).
    pub(super) fn generate_truncated_finding(&mut self, key: &FlowKey, src_ip: std::net::IpAddr) {
        if self.findings.len() >= MAX_FINDINGS {
            self.stats.dropped_findings += 1;
            return;
        }
        self.findings.push(Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: format!("Stream depth exceeded on flow {key}"),
            evidence: vec![format!("Max depth {} bytes reached", self.config.max_depth)],
            mitre_technique: None,
            source_ip: Some(src_ip),
            timestamp: None,
            direction: None,
        });
    }
}
