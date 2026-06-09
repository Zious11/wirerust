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

use chrono::DateTime;

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
        // BC-2.04.055 close-flush case: use flow.last_seen as the timestamp
        // passed to on_data — the most-recent packet timestamp for this flow,
        // available from the TcpFlow value before it is dropped.
        let close_timestamp = flow.last_seen;
        let flow_mem = flow.memory_used();
        for dir in [Direction::ClientToServer, Direction::ServerToClient] {
            let flow_dir = flow.get_direction_mut(dir);
            let flushed = flow_dir.flush_contiguous();
            for (offset, data) in &flushed {
                self.stats.bytes_reassembled += data.len() as u64;
                handler.on_data(key, dir, data, *offset, close_timestamp);
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
    ///
    /// `timestamp` is the current packet's `timestamp_secs` (BC-2.09.007):
    /// attached as `Some(DateTime<Utc>)` for capture-relative pcap provenance.
    pub(super) fn generate_conflicting_overlap_finding(
        &mut self,
        key: &FlowKey,
        src_ip: std::net::IpAddr,
        timestamp: u32,
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
            mitre_techniques: vec!["T1036".to_string()],
            source_ip: Some(src_ip),
            // BC-2.09.007 post-1: capture-relative pcap timestamp from the
            // current packet that triggered the conflicting overlap.
            timestamp: DateTime::from_timestamp(timestamp as i64, 0),
            direction: None,
        });
    }

    /// Emit the "stream depth exceeded" Anomaly finding (or count it as
    /// dropped if the `MAX_FINDINGS` cap is already hit).
    ///
    /// `timestamp` is the current packet's `timestamp_secs` (BC-2.09.007):
    /// attached as `Some(DateTime<Utc>)` for capture-relative pcap provenance.
    pub(super) fn generate_truncated_finding(
        &mut self,
        key: &FlowKey,
        src_ip: std::net::IpAddr,
        timestamp: u32,
    ) {
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
            mitre_techniques: vec![],
            source_ip: Some(src_ip),
            // BC-2.09.007 post-1: capture-relative pcap timestamp from the
            // current packet that triggered the depth-exceeded event.
            timestamp: DateTime::from_timestamp(timestamp as i64, 0),
            direction: None,
        });
    }
}

// ---- Test-only seams (BC-2.04.029 / ADR-0004 amendment) -------------------
//
// These free functions expose the process-global `CLOSE_FLOW_MISSING_WARNED`
// atomic and the `pub(super) close_flow` path to integration tests. The
// pattern mirrors `wirerust::reassembly::segment::isn_missing_warned_for_testing`
// added in STORY-014.
//
// Rationale: `close_flow` is intentionally `pub(super)` to keep the crate API
// narrow. Rather than widening it, we add a thin `_for_testing` wrapper
// (per the ADR-0004 amendment opt-in-per-guard doctrine) so tests
// can exercise BC-2.04.029 AC-013/AC-014 and EC-009/EC-010 deterministically.

/// Test-only accessor for the process-global `CLOSE_FLOW_MISSING_WARNED` flag.
///
/// Exposes a read of [`CLOSE_FLOW_MISSING_WARNED`] so integration tests can
/// verify the one-shot behavior asserted by BC-2.04.029 PC4 — the flag
/// transitions `false → true` exactly once across the process lifetime.
///
/// Sibling to `wirerust::reassembly::segment::isn_missing_warned_for_testing`.
#[doc(hidden)]
pub fn close_flow_missing_warned_for_testing() -> bool {
    CLOSE_FLOW_MISSING_WARNED.load(Ordering::Relaxed)
}

/// Test-only reset of the process-global `CLOSE_FLOW_MISSING_WARNED` flag.
///
/// Allows tests to deterministically observe the BC-2.04.029 PC4
/// `false → true` swap transition. MUST NOT be called from production code.
#[doc(hidden)]
pub fn reset_close_flow_missing_warned_for_testing() {
    CLOSE_FLOW_MISSING_WARNED.store(false, Ordering::Relaxed);
}

/// Test-only trigger for the `close_flow` missing-key path.
///
/// Directly exercises the missing-key branch logic from `close_flow` —
/// the atomic swap + one-shot `eprintln!` guard — without going through
/// `close_flow` itself.
///
/// Why not call `reassembler.close_flow(key, reason, handler)` directly?
/// The production `close_flow` body begins the missing-key branch with
/// `debug_assert!(false, "close_flow called for non-existent key: {key}")`
/// per BC-2.04.029 PC6. In debug-profile builds (the default for `cargo
/// test`) this `debug_assert!` executes BEFORE the atomic swap, panicking
/// before `CLOSE_FLOW_MISSING_WARNED` is ever set. `catch_unwind` would
/// suppress the panic but the atomic would remain `false`, causing BC-2.04.029
/// PC4/PC5 assertions to fail.
///
/// Calling `close_flow` via `catch_unwind` is therefore insufficient.
/// Instead this seam directly replicates the observable behavior of the
/// missing-key branch (the atomic transition and silent return) without
/// triggering the `debug_assert!`. The production `debug_assert!` remains
/// intact and fires in production debug builds whenever `close_flow` is
/// called with a missing key outside of this test seam.
///
/// `close_flow` is `pub(super)` to keep the crate API narrow (ADR-0004
/// amendment, opt-in-per-guard doctrine).
#[doc(hidden)]
pub fn trigger_close_flow_missing_key_for_testing(
    _reassembler: &mut TcpReassembler,
    key: &crate::reassembly::flow::FlowKey,
    reason: crate::reassembly::handler::CloseReason,
    _handler: &mut dyn crate::reassembly::handler::StreamHandler,
) {
    // Mirror the post-debug_assert! body of the missing-key branch in
    // `close_flow` (lifecycle.rs lines 44-48). The debug_assert! itself is
    // intentionally not replicated here — this seam exists to let tests
    // observe the atomic transition without crashing the test thread.
    // The _reassembler and _handler params are accepted purely for signature
    // symmetry with close_flow at call sites. This seam directly replicates
    // the post-debug_assert body of the missing-key branch (atomic swap +
    // one-shot eprintln) without touching either; BC-2.04.029 v1.4 PC1/PC2/PC3
    // are enforced STRUCTURALLY (let-else early-return at lifecycle.rs:42-50,
    // verified by code review), not by this seam. See the doc-comment above
    // for the catch_unwind-avoidance rationale.
    if !CLOSE_FLOW_MISSING_WARNED.swap(true, Ordering::Relaxed) {
        eprintln!("wirerust: close_flow called for non-existent key: {key} (reason: {reason:?})");
    }
}

/// Test-only seam to force a flow's state to a specific [`FlowState`] without
/// going through the on_fin / on_rst / on_syn state-machine transitions.
///
/// Required by BC-2.04.013 inv-2 ("a flow with `state == FlowState::Closed`
/// is also expired here") to construct a flow in `Closed` state while keeping
/// its `last_seen` within the timeout window — proving the state-based
/// expiry clause is independent of the time-based clause.
///
/// MUST NOT be called from production code. `#[doc(hidden)]` per ADR-0004
/// amendment opt-in-per-guard seam hygiene.
///
/// Returns `true` if the flow was found and its state updated; `false` if no
/// flow with the given key exists (no-op, no panic).
#[doc(hidden)]
pub fn force_set_flow_state_for_testing(
    reassembler: &mut TcpReassembler,
    key: &crate::reassembly::flow::FlowKey,
    state: crate::reassembly::flow::FlowState,
) -> bool {
    if let Some(flow) = reassembler.flows_mut().get_mut(key) {
        flow.state = state;
        true
    } else {
        false
    }
}
