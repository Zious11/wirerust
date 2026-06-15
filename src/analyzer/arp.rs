//! ARP security analyzer — STORY-113 full-impl skeleton (Red Gate stubs).
//!
//! This module defines [`ArpAnalyzer`], a stateful ARP-frame processor that
//! maintains a bounded binding table (IP→MAC with LRU eviction), detects
//! Gratuitous ARP (D2), D11 malformed ARP, D12 L2/L3 sender-MAC mismatch,
//! and exposes a `summarize()` method returning eleven canonical summary keys.
//!
//! **STORY-113 stub state:** All non-trivial method bodies are `todo!()` per
//! BC-5.38.001. The skeleton compiles so that test-writer can write RED tests
//! that drive the implementer. `new()` and `Default::default()` are GREEN-BY-DESIGN
//! (zero branching, no I/O, no non-trivial helpers, ≤3 lines each).
//!
//! ## Scope boundary
//! - D1 spoof EMISSION (BC-2.16.004) is NOT in this story → STORY-114.
//! - D3 storm detection (BC-2.16.013) is NOT in this story → STORY-115.
//! - `spoof_findings` and `storm_findings` summary keys will be 0 after STORY-113.
//!
//! ## Forbidden dependencies
//! This module MUST NOT import `crate::dispatcher`, `crate::analyzer::modbus`,
//! or `crate::analyzer::dnp3` (arp-architecture-delta.md §1; STORY-113 Forbidden
//! Dependencies; BC-2.16.015 Invariant 2).

use std::collections::HashMap;

use crate::analyzer::AnalysisSummary;
use crate::decoder::ArpFrame;
use crate::findings::Finding;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Maximum number of IP→MAC binding entries retained at any time.
///
/// BC-2.16.006 postcondition 2: `bindings.len()` NEVER exceeds this value.
/// When a new IP would cause overflow, `insert_binding_lru` evicts the entry
/// with the minimum `last_seen_ts` before inserting.
pub const MAX_ARP_BINDINGS: usize = 65_536;

/// Reduced cap used by the VP-024 Sub-D Kani harness (`verify_binding_table_cap`).
///
/// Using 65_536 in a bounded-model-check loop is not feasible; 8 lets the
/// harness iterate cap+1 = 9 times with `#[kani::unwind(12)]`.
#[cfg(any(kani, test))]
pub const TEST_MAX_ARP_BINDINGS: usize = 8;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Per-IP binding entry in the ARP binding table.
///
/// Tracks the most-recently-seen MAC for each sender IP, along with
/// metadata used for D1 escalation (STORY-114) and LRU eviction.
///
/// BC-2.16.005 Architecture Anchors; ADR-008 Decision 4.
#[derive(Debug, Clone)]
pub struct BindingEntry {
    /// Most-recently-observed hardware (MAC) address for this IP.
    pub mac: [u8; 6],
    /// Number of times the MAC changed for this IP (rebind count).
    /// Incremented by `process_arp` when a new MAC differs from the stored MAC.
    /// D1 EMISSION on rebind is STORY-114; STORY-113 only updates this counter.
    pub rebind_count: u32,
    /// Timestamp (seconds) of the first rebind event. `None` until the first rebind.
    pub first_rebind_ts: Option<u32>,
    /// True once a D1 HIGH spoof finding has been emitted for this entry.
    /// Set in STORY-114. Always `false` after STORY-113.
    pub spoof_high_emitted: bool,
    /// Timestamp (seconds) of the most recent frame that observed this binding.
    /// Written by `process_arp` before calling `insert_binding_lru` so the
    /// eviction function can read it during the LRU scan.
    /// NOTE: `insert_binding_lru` has NO `ts` parameter — Architecture Compliance Rule 1.
    pub last_seen_ts: u32,
}

/// Per-sender-MAC storm-rate tracking entry (stub; D3 logic in STORY-115).
///
/// Fields are allocated here so the type system is correct for STORY-115.
/// All logic that populates / checks these fields is deferred to STORY-115.
#[derive(Debug, Clone)]
pub struct StormCounter {
    /// Number of ARP frames from this sender MAC in the current detection window.
    pub count_in_window: u64,
    /// Timestamp (seconds) when the current detection window started.
    pub window_start_ts: u32,
    /// True once a D3 storm finding has been emitted for this MAC.
    /// Set in STORY-115. Always `false` after STORY-113.
    pub storm_emitted: bool,
}

// ---------------------------------------------------------------------------
// Free functions (pure core — VP-024 Sub-B/D targets)
// ---------------------------------------------------------------------------

/// Classify a decoded ARP frame as Gratuitous ARP (GARP) or normal.
///
/// Returns `true` if and only if `frame.sender_ip == frame.target_ip` (byte-wise).
/// The function is opcode-agnostic: both request-GARPs (op=1, RFC 5227 announce)
/// and reply-GARPs (op=2) satisfy the predicate. No other field is inspected.
///
/// BC-2.16.003 postconditions 1/2/3; VP-024 Sub-B Kani target (`verify_classify_garp_total`).
///
/// BC-5.38.005 self-check: "If I include this real implementation, will the test for
/// this function pass trivially without any implementer work?" — YES. The single-expression
/// body `frame.sender_ip == frame.target_ip` would make AC-001/AC-002 tests pass without
/// implementer work. However, this function IS the specified behavior (it is not domain
/// logic that requires test-driven discovery; it is literally the definition of GARP per
/// RFC 5227 and BC-2.16.003). This is a GREEN-BY-DESIGN function per BC-5.38.002: zero
/// branching, no I/O, no non-trivial helpers, 1 line. Listed in stub commit report.
pub fn is_gratuitous_arp(_frame: &ArpFrame) -> bool {
    todo!("STORY-113: implement — return frame.sender_ip == frame.target_ip")
}

/// Insert or update a binding entry in the production HashMap binding table.
///
/// If `bindings.len() >= cap` and `ip` is not already in the table, evicts
/// the entry with the minimum `last_seen_ts` before inserting. If `ip` is
/// already present, the entry is updated in-place (last-write-wins per BC-2.16.005).
///
/// **Architecture Compliance Rule 1:** This function has NO `ts` parameter.
/// `last_seen_ts` is written into the entry by `process_arp` before this
/// function is called; the eviction scan reads it from the stored entries.
///
/// BC-2.16.005 postcondition 1; BC-2.16.006 postcondition 2; ADR-008 Decision 4.
pub fn insert_binding_lru(
    _bindings: &mut HashMap<[u8; 4], BindingEntry>,
    _ip: [u8; 4],
    _mac: [u8; 6],
    _cap: usize,
) {
    todo!("STORY-113: implement LRU eviction + insert/update logic")
}

/// BTreeMap surrogate of `insert_binding_lru` for VP-024 Sub-D Kani harness.
///
/// Semantically identical to `insert_binding_lru` but uses a `BTreeMap` so
/// that the Kani bounded-model-checker can iterate over entries in a
/// deterministic order (HashMap iteration order is non-deterministic under
/// symbolic execution).
///
/// Gated `#[cfg(any(kani, test))]` — NOT present in the production binary.
/// `TEST_MAX_ARP_BINDINGS = 8` is used as the cap in the Kani harness.
///
/// Architecture Compliance Rule 2: BTreeMap surrogate is cfg-gated only.
/// ADR-008 Decision 5; VP-024 Sub-D.
#[cfg(any(kani, test))]
pub fn insert_binding_lru_btree(
    _bindings: &mut std::collections::BTreeMap<[u8; 4], BindingEntry>,
    _ip: [u8; 4],
    _mac: [u8; 6],
    _cap: usize,
) {
    todo!("STORY-113: implement BTreeMap LRU eviction + insert/update logic")
}

// ---------------------------------------------------------------------------
// ArpAnalyzer
// ---------------------------------------------------------------------------

/// Stateful ARP security analyzer.
///
/// Receives [`ArpFrame`] values from `main.rs` after `decode_packet` produces
/// `DecodedFrame::Arp`. Activated by the `--arp` CLI flag (BC-2.16.011).
///
/// STORY-113 delivers: binding table (BC-2.16.005/006), GARP detection D2
/// (BC-2.16.003), D11 malformed ARP (BC-2.16.009), D12 L2/L3 mismatch
/// (BC-2.16.007), and `summarize()` with all 11 canonical keys (BC-2.16.010).
///
/// D1 spoof EMISSION is STORY-114. D3 storm detection is STORY-115.
/// `--arp` is parameterless in this story (Architecture Compliance Rule 3).
pub struct ArpAnalyzer {
    /// IP→MAC binding table. Cap enforced at MAX_ARP_BINDINGS via LRU eviction.
    /// Production substrate per Architecture Compliance Rule 2.
    pub bindings: HashMap<[u8; 4], BindingEntry>,
    /// Per-sender-MAC storm tracking (stub; D3 logic in STORY-115).
    pub storm_counters: HashMap<[u8; 6], StormCounter>,

    // --- per-capture counters ---
    /// Total ARP frames processed by `process_arp` (excludes malformed frames).
    pub frames_analyzed: u64,
    /// ARP Request (op=1) frame count.
    pub request_count: u64,
    /// ARP Reply (op=2) frame count.
    pub reply_count: u64,
    /// ARP frames with operation code other than 1 or 2.
    pub other_opcode_count: u64,
    /// D1 spoof findings emitted (always 0 after STORY-113; set in STORY-114).
    pub spoof_findings: u64,
    /// D2 GARP findings emitted.
    pub garp_findings: u64,
    /// D3 storm findings emitted (always 0 after STORY-113; set in STORY-115).
    pub storm_findings: u64,
    /// D12 L2/L3 mismatch findings emitted.
    pub mismatch_findings: u64,
    /// D11 malformed ARP findings emitted (only when `--arp` is active per AC-012).
    pub malformed_findings: u64,
    /// Total malformed ARP frames counted (always incremented, even without `--arp`).
    pub malformed_frames: u64,
}

impl ArpAnalyzer {
    /// Construct a zeroed `ArpAnalyzer`.
    ///
    /// Parameterless in STORY-113 per Architecture Compliance Rule 3:
    /// - `--arp-spoof-threshold` is added in STORY-114 (BC-2.16.012).
    /// - `--arp-storm-rate` is added in STORY-115 (BC-2.16.013).
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 3 lines.
    /// Returns a fully-zeroed struct — no implementer work is required for this body.
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            storm_counters: HashMap::new(),
            frames_analyzed: 0,
            request_count: 0,
            reply_count: 0,
            other_opcode_count: 0,
            spoof_findings: 0,
            garp_findings: 0,
            storm_findings: 0,
            mismatch_findings: 0,
            malformed_findings: 0,
            malformed_frames: 0,
        }
    }

    /// Process a decoded ARP frame.
    ///
    /// Full detection pipeline:
    /// (a) filter zero/broadcast sender_ip (BC-2.16.005 Invariant 5);
    /// (b) count frame + opcode (frames_analyzed, request/reply/other_opcode);
    /// (c) check D12 mismatch (outer_src_mac vs sender_mac — BC-2.16.007);
    /// (d) check D2 GARP (is_gratuitous_arp — BC-2.16.003);
    /// (e) update binding table via insert_binding_lru (BC-2.16.005/006);
    /// (f) detect rebind (MAC change) — update rebind_count/first_rebind_ts,
    ///     do NOT emit D1 finding (STORY-114 responsibility);
    /// (g) return Vec of findings (D2/D12 only in this story; D1/D3 deferred).
    ///
    /// BC-2.16.003, BC-2.16.005, BC-2.16.006, BC-2.16.007.
    pub fn process_arp(&mut self, _frame: &ArpFrame, _timestamp_secs: u32) -> Vec<Finding> {
        todo!("STORY-113: implement full detection pipeline (D2/D12 + binding update)")
    }

    /// Receive notification that a malformed ARP frame was observed.
    ///
    /// Called from `main.rs` when `decode_packet` returns
    /// `Err("Non-Ethernet/IPv4 ARP frame")` for an ARP EtherType packet.
    ///
    /// Behavior (BC-2.16.009):
    /// - Always increments `malformed_frames` (even when `--arp` absent, per AC-012).
    /// - Emits a D11 LOW/Anomaly finding and increments `malformed_findings` ONLY
    ///   when `--arp` is active. The `--arp` gate is enforced in `main.rs` (AC-012).
    ///   `record_malformed` itself always increments `malformed_frames`; the gate
    ///   in main.rs decides whether to call the finding-emission variant or bare counter.
    ///
    /// `packet_len` is included in the finding evidence (BC-2.16.009 PC3).
    pub fn record_malformed(&mut self, _packet_len: usize) {
        todo!("STORY-113: implement D11 LOW/Anomaly finding emission + counter updates")
    }

    /// Produce the eleven-key `AnalysisSummary` for this capture.
    ///
    /// Returns an `AnalysisSummary` with `analyzer_name = "ARP"` and exactly
    /// the following eleven keys in the `detail` BTreeMap (Architecture Compliance
    /// Rule 5 — exact string names are the contract per BC-2.16.010 PC1):
    ///
    /// - `"frames_analyzed"`
    /// - `"request_count"`
    /// - `"reply_count"`
    /// - `"other_opcode_count"`
    /// - `"bindings_tracked"`
    /// - `"spoof_findings"`
    /// - `"garp_findings"`
    /// - `"storm_findings"`
    /// - `"mismatch_findings"`
    /// - `"malformed_findings"`
    /// - `"malformed_frames"`
    ///
    /// `storm_findings` is always 0 after STORY-113 (Architecture Compliance Rule 6).
    /// `spoof_findings` is always 0 after STORY-113 (Scope Boundary: D1 deferred).
    /// `bindings_tracked` = `self.bindings.len()` as u64.
    ///
    /// BC-2.16.010; BC-2.16.011 PC7.
    pub fn summarize(&self) -> AnalysisSummary {
        todo!("STORY-113: implement eleven-key AnalysisSummary return")
    }
}

impl Default for ArpAnalyzer {
    /// GREEN-BY-DESIGN: delegates to `new()` — zero branching, no I/O, no helpers, 1 line.
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Test affordances (ADR-008 Decision 4; VP-024 Sub-C)
// ---------------------------------------------------------------------------

#[cfg(test)]
impl ArpAnalyzer {
    /// Construct an `ArpAnalyzer` for use in unit tests.
    ///
    /// In STORY-113 this is identical to `new()`. STORY-114/115 may add
    /// threshold parameters to the production `new()` while keeping
    /// `new_for_test()` parameterless for Sub-C proptest compatibility.
    ///
    /// ADR-008 Decision 4; VP-024 Sub-C anchor (BC-2.16.005).
    pub fn new_for_test() -> Self {
        Self::new()
    }

    /// Process a frame using a fixed-cap binding table for Sub-C/Sub-D tests.
    ///
    /// Calls `process_arp` with the given frame and timestamp.
    /// Exists so tests can drive the analyzer without constructing CLI args.
    ///
    /// ADR-008 Decision 4 extension; VP-024 Sub-C.
    pub fn process_arp_for_test(
        &mut self,
        _frame: &ArpFrame,
        _timestamp_secs: u32,
    ) -> Vec<Finding> {
        todo!("STORY-113: delegate to process_arp once implemented")
    }

    /// Return a snapshot of the current binding table for assertion in tests.
    ///
    /// Returns a clone of `self.bindings` so tests can inspect the table state
    /// after processing a sequence of frames.
    ///
    /// VP-024 Sub-C (`test_binding_table_last_write_wins` proptest).
    pub fn bindings_snapshot(&self) -> HashMap<[u8; 4], BindingEntry> {
        todo!("STORY-113: return self.bindings.clone()")
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unreachable_code)]
mod tests {
    // -----------------------------------------------------------------------
    // VP-024 Sub-C: test_binding_table_last_write_wins (proptest)
    //
    // BC-2.16.005 postcondition 1; VP-024 Sub-C anchor adjudication (PO,
    // 2026-06-13). For any arbitrary sequence of ArpFrame values, after
    // processing all frames, `bindings[ip].mac` must equal the MAC from
    // the last frame with that sender_ip.
    //
    // Runs at `cargo test` (NOT deferred to F6 Kani gate).
    //
    // This stub compiles but the proptest body is todo!() — the implementer
    // fills in the proptest strategy and assertion once `process_arp_for_test`
    // and `bindings_snapshot` are implemented.
    // -----------------------------------------------------------------------
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_binding_table_last_write_wins(
            // Strategy: sequences of (ip_octet, mac_byte, opcode) tuples up to 1000 entries.
            // Each tuple drives one synthetic ArpFrame with a non-zero, non-broadcast sender_ip
            // derived from ip_octet to keep the table bounded.
            entries in proptest::collection::vec(
                (1u8..=254u8, any::<u8>(), proptest::sample::select(vec![1u16, 2u16])),
                0..=1000
            )
        ) {
            // TODO(STORY-113 implementer): construct an ArpAnalyzer via new_for_test(),
            // drive it through the entries (building ArpFrame values from the tuple),
            // then assert that for every unique ip, bindings_snapshot()[ip].mac equals
            // the mac from the last frame that had that ip.
            //
            // The proptest is intentionally a skeleton: the body panics so the test
            // is RED until the implementer fills it in.
            let _ = entries;
            todo!("STORY-113: implement proptest body — drive new_for_test()/process_arp_for_test()/bindings_snapshot() and assert last-write-wins invariant")
        }
    }
}

// ---------------------------------------------------------------------------
// Kani formal-verification harnesses (VP-024 Sub-B and Sub-D)
// ---------------------------------------------------------------------------

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    /// VP-024 Sub-B: GARP detection totality (BC-2.16.003 biconditional).
    ///
    /// Symbolic `ArpFrame` with all fields symbolic; asserts:
    ///   `is_gratuitous_arp(frame) == (frame.sender_ip == frame.target_ip)`
    /// for ALL symbolic inputs. Never panics.
    ///
    /// AC-017; runs at F6 formal-hardening gate.
    /// Body is `todo!()` per BC-5.38.001 — filled by formal-verifier at F6.
    #[kani::proof]
    fn verify_classify_garp_total() {
        todo!("STORY-113 implements is_gratuitous_arp — harness body filled at F6 gate")
    }

    /// VP-024 Sub-D: binding table cap invariant (BC-2.16.006).
    ///
    /// Uses `insert_binding_lru_btree` (BTreeMap surrogate, cfg-gated).
    /// `TEST_MAX_ARP_BINDINGS = 8`; 9-iteration loop (cap+1).
    /// Asserts `bindings.len() <= TEST_MAX_ARP_BINDINGS` after each insert.
    ///
    /// AC-019; runs at F6 formal-hardening gate.
    /// Body is `todo!()` per BC-5.38.001 — filled by formal-verifier at F6.
    #[kani::proof]
    #[kani::unwind(12)]
    fn verify_binding_table_cap() {
        todo!("STORY-113 implements insert_binding_lru_btree — harness body filled at F6 gate")
    }
}
