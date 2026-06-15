//! ARP security analyzer — STORY-113 full implementation (GREEN) + STORY-114 stubs.
//!
//! This module defines [`ArpAnalyzer`], a stateful ARP-frame processor that
//! maintains a bounded binding table (IP→MAC with LRU eviction), detects
//! Gratuitous ARP (D2), D11 malformed ARP, D12 L2/L3 sender-MAC mismatch,
//! and exposes a `summarize()` method returning eleven canonical summary keys.
//!
//! All STORY-113 method bodies are implemented and all STORY-113 tests pass (GREEN).
//! The VP-024 Sub-B/Sub-D Kani harness bodies (`verify_classify_garp_total`,
//! `verify_binding_table_cap`) remain `todo!()` pending the F6 formal-hardening gate —
//! that is the only intentional `todo!()` in this module for STORY-113 functions.
//!
//! ## STORY-114 scaffold (Red Gate)
//! - `ArpAnalyzer::new(spoof_threshold, storm_rate)` — signature extended.
//! - `SPOOF_REBIND_ESCALATION_DEFAULT`, `ARP_FLAP_WINDOW_SECS`, `ARP_STORM_RATE_DEFAULT`
//!   constants added.
//! - `--arp-spoof-threshold` CLI flag wired (BC-2.16.012 primary deliverable).
//! - `emit_d1_spoof_finding` and `apply_garp_conflict_escalation` helpers added as
//!   uncalled `todo!()` stubs — NOT yet wired into `process_arp`. The implementer
//!   wires these in the Green step.
//! - `process_arp` retains STORY-113 behaviour (no D1 emission, GARP stays LOW).
//! - `src/mitre.rs` is untouched (SEEDED=23, EMITTED=15); VP-007 5-part atomic
//!   update is the implementer's Green step.
//!
//! ## Scope boundary
//! - D1 spoof EMISSION (BC-2.16.004) is STORY-114 (stubs here, impl in Green step).
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
// STORY-114 constants (BC-2.16.004 / BC-2.16.012 / BC-2.16.013)
// ---------------------------------------------------------------------------

/// Default rebind count threshold at which D1 spoof finding escalates to HIGH.
///
/// BC-2.16.004 postcondition 1.c; BC-2.16.012 default.
/// CLI flag `--arp-spoof-threshold` overrides this value per AC-006.
pub const SPOOF_REBIND_ESCALATION_DEFAULT: u32 = 3;

/// Detection window in seconds for D1 spoof escalation (also shared with D3 storm, STORY-115).
///
/// BC-2.16.004 postcondition 1.c (flap window); BC-2.16.013 (storm window).
pub const ARP_FLAP_WINDOW_SECS: u32 = 60;

/// Default ARP storm rate threshold (frames/second per sender MAC) for D3 detection.
///
/// D3 storm logic is STORY-115. This constant is defined here so
/// `ArpAnalyzer::new(spoof_threshold, storm_rate)` has a canonical default
/// for the `storm_rate` parameter at the STORY-114 call site in `src/main.rs`
/// (standalone-compile: `--arp-storm-rate` flag does not exist until STORY-115).
pub const ARP_STORM_RATE_DEFAULT: u32 = 50;

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
/// branching, no I/O, no non-trivial helpers, 1 line.
pub fn is_gratuitous_arp(frame: &ArpFrame) -> bool {
    frame.sender_ip == frame.target_ip
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
    bindings: &mut HashMap<[u8; 4], BindingEntry>,
    ip: [u8; 4],
    mac: [u8; 6],
    cap: usize,
) {
    if bindings.contains_key(&ip) {
        // Update existing entry in-place (last-write-wins, last_seen_ts already set by caller).
        if let Some(entry) = bindings.get_mut(&ip) {
            entry.mac = mac;
        }
        return;
    }
    // New IP — evict the entry with the minimum last_seen_ts if at capacity.
    if bindings.len() >= cap {
        let oldest_ip = bindings
            .iter()
            .min_by_key(|(_, e)| e.last_seen_ts)
            .map(|(k, _)| *k);
        if let Some(k) = oldest_ip {
            bindings.remove(&k);
        }
    }
    bindings.insert(
        ip,
        BindingEntry {
            mac,
            rebind_count: 0,
            first_rebind_ts: None,
            spoof_high_emitted: false,
            last_seen_ts: 0,
        },
    );
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
    bindings: &mut std::collections::BTreeMap<[u8; 4], BindingEntry>,
    ip: [u8; 4],
    mac: [u8; 6],
    cap: usize,
) {
    if bindings.contains_key(&ip) {
        // Update existing entry in-place.
        if let Some(entry) = bindings.get_mut(&ip) {
            entry.mac = mac;
        }
        return;
    }
    // New IP — evict the entry with the minimum last_seen_ts if at capacity.
    if bindings.len() >= cap {
        let oldest_ip = bindings
            .iter()
            .min_by_key(|(_, e)| e.last_seen_ts)
            .map(|(k, _)| *k);
        if let Some(k) = oldest_ip {
            bindings.remove(&k);
        }
    }
    bindings.insert(
        ip,
        BindingEntry {
            mac,
            rebind_count: 0,
            first_rebind_ts: None,
            spoof_high_emitted: false,
            last_seen_ts: 0,
        },
    );
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
/// STORY-114 extends `new(spoof_threshold, storm_rate)` and wires
/// `--arp-spoof-threshold` (BC-2.16.012). D1 emission + GARP-conflict
/// helpers are stubs until the Green step.
///
/// D3 storm detection is STORY-115.
pub struct ArpAnalyzer {
    /// IP→MAC binding table. Cap enforced at MAX_ARP_BINDINGS via LRU eviction.
    /// Production substrate per Architecture Compliance Rule 2.
    pub bindings: HashMap<[u8; 4], BindingEntry>,
    /// Per-sender-MAC storm tracking (stub; D3 logic in STORY-115).
    pub storm_counters: HashMap<[u8; 6], StormCounter>,

    // --- STORY-114 threshold fields (BC-2.16.012 / BC-2.16.013) ---
    /// D1 spoof escalation threshold: number of rebinds within ARP_FLAP_WINDOW_SECS
    /// before a HIGH finding is emitted. Overridable via `--arp-spoof-threshold`.
    /// BC-2.16.012; default = SPOOF_REBIND_ESCALATION_DEFAULT = 3.
    pub spoof_threshold: u32,
    /// ARP storm rate threshold (frames/second per sender MAC) for D3 detection.
    /// Stub field; D3 logic is STORY-115. Stored here for `new()` API completeness.
    /// Default = ARP_STORM_RATE_DEFAULT = 50.
    pub storm_rate: u32,

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
    /// Construct a zeroed `ArpAnalyzer` with caller-supplied thresholds.
    ///
    /// `spoof_threshold` — rebind count at which D1 escalates to HIGH
    ///   (BC-2.16.004 postcondition 1.c; BC-2.16.012).
    ///   Pass `SPOOF_REBIND_ESCALATION_DEFAULT` (= 3) when no CLI override.
    ///
    /// `storm_rate` — ARP frames/second threshold for D3 storm detection
    ///   (BC-2.16.013; STORY-115). Pass `ARP_STORM_RATE_DEFAULT` (= 50)
    ///   until STORY-115 wires `args.arp_storm_rate`.
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no non-trivial helpers, 3 lines.
    /// Returns a fully-zeroed struct with the given thresholds — no implementer
    /// work is required for this body.
    pub fn new(spoof_threshold: u32, storm_rate: u32) -> Self {
        Self {
            bindings: HashMap::new(),
            storm_counters: HashMap::new(),
            spoof_threshold,
            storm_rate,
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
    ///     - if GARP AND binding conflict (BC-2.16.014): escalate GARP LOW→MEDIUM,
    ///       attach T0830/T1557.002, co-emit D1 finding (Steps 1–3 via emit_d1_spoof_finding);
    ///       Step 4 (MAC update) occurs after both findings are produced.
    ///     - if GARP with no binding conflict: LOW finding, no D1, no MITRE.
    /// (e)/(f) For non-GARP frames: update binding table; if rebind (MAC change),
    ///     emit D1 finding via emit_d1_spoof_finding (Steps 1–3);
    ///     Step 4 (MAC update) occurs last — Architecture Compliance Rule 1.
    /// (g) return Vec of findings (D2/D12/D1 per this story; D3 deferred).
    ///
    /// BC-2.16.003, BC-2.16.004, BC-2.16.005, BC-2.16.006, BC-2.16.007, BC-2.16.014.
    pub fn process_arp(&mut self, frame: &ArpFrame, timestamp_secs: u32) -> Vec<Finding> {
        use crate::findings::{Confidence, ThreatCategory, Verdict};

        let mut findings = Vec::new();

        // (b) Count frame + opcode — all frames (including zero/broadcast sender_ip).
        self.frames_analyzed += 1;
        match frame.operation {
            1 => self.request_count += 1,
            2 => self.reply_count += 1,
            _ => self.other_opcode_count += 1,
        }

        // (a) Filter zero/broadcast sender_ip — BC-2.16.005 Invariant 5.
        // Detection and binding update are skipped for these addresses.
        let is_zero = frame.sender_ip == [0u8; 4];
        let is_broadcast = frame.sender_ip == [255u8; 4];
        if is_zero || is_broadcast {
            return findings;
        }

        // (c) D12 mismatch check — BC-2.16.007 + AC-017 (T0830/T1557.002 back-fill).
        if let Some(eth_mac) = frame.outer_src_mac
            && eth_mac != frame.sender_mac
        {
            self.mismatch_findings += 1;
            findings.push(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Possible,
                confidence: Confidence::Medium,
                summary: "D12: L2/L3 sender-MAC mismatch — Ethernet src MAC differs from ARP \
                          sender HW addr"
                    .to_string(),
                evidence: vec![
                    format!(
                        "eth_src_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        eth_mac[0], eth_mac[1], eth_mac[2], eth_mac[3], eth_mac[4], eth_mac[5]
                    ),
                    format!(
                        "arp_sender_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        frame.sender_mac[0],
                        frame.sender_mac[1],
                        frame.sender_mac[2],
                        frame.sender_mac[3],
                        frame.sender_mac[4],
                        frame.sender_mac[5]
                    ),
                    format!(
                        "sender_ip={}.{}.{}.{}",
                        frame.sender_ip[0],
                        frame.sender_ip[1],
                        frame.sender_ip[2],
                        frame.sender_ip[3]
                    ),
                ],
                // AC-017 / BC-2.16.007 MITRE back-fill (co-committed with VP-007 5-part update).
                mitre_techniques: vec!["T0830".to_string(), "T1557.002".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            });
        }

        let sender_ip = frame.sender_ip;
        let sender_mac = frame.sender_mac;

        // (d) D2 GARP check — BC-2.16.003 / BC-2.16.014.
        if is_gratuitous_arp(frame) {
            // Determine whether there is a binding conflict.
            let has_conflict = self
                .bindings
                .get(&sender_ip)
                .map(|e| e.mac != sender_mac)
                .unwrap_or(false);

            if has_conflict {
                // BC-2.16.014 PC1/2: GARP-that-conflicts path.
                // Build initial GARP finding at LOW (to be upgraded below).
                let garp_low = Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Possible,
                    confidence: Confidence::Low,
                    summary: "D2: Gratuitous ARP (GARP) — sender_ip equals target_ip".to_string(),
                    evidence: vec![
                        format!(
                            "sender_ip={}.{}.{}.{}",
                            sender_ip[0], sender_ip[1], sender_ip[2], sender_ip[3]
                        ),
                        format!("operation={}", frame.operation),
                    ],
                    mitre_techniques: vec![],
                    source_ip: None,
                    timestamp: None,
                    direction: None,
                };

                // apply_garp_conflict_escalation: upgrade GARP to MEDIUM + co-emit D1.
                // It also runs Steps 1–3 of BC-2.16.004 PC1 on the entry.
                // Step 4 (MAC update) is applied afterward.
                let entry = self
                    .bindings
                    .get_mut(&sender_ip)
                    .expect("has_conflict implies entry exists");
                let (upgraded_garp, d1) = Self::apply_garp_conflict_escalation_impl(
                    entry,
                    garp_low,
                    sender_ip,
                    sender_mac,
                    timestamp_secs,
                    self.spoof_threshold,
                );

                self.garp_findings += 1;
                self.spoof_findings += 1;
                findings.push(upgraded_garp);
                findings.push(d1);

                // Step 4 (BC-2.16.004 PC1 — Architecture Compliance Rule 1):
                // MAC update occurs AFTER escalation evaluation and emission.
                // Write last_seen_ts first (LRU correctness), then update MAC.
                let entry = self
                    .bindings
                    .get_mut(&sender_ip)
                    .expect("entry must still exist");
                entry.last_seen_ts = timestamp_secs;
                entry.mac = sender_mac;
            } else {
                // BC-2.16.014 PC6 / EC-009: benign GARP (no conflict) → LOW only, no D1, no MITRE.
                self.garp_findings += 1;
                findings.push(Finding {
                    category: ThreatCategory::Anomaly,
                    verdict: Verdict::Possible,
                    confidence: Confidence::Low,
                    summary: "D2: Gratuitous ARP (GARP) — sender_ip equals target_ip".to_string(),
                    evidence: vec![
                        format!(
                            "sender_ip={}.{}.{}.{}",
                            sender_ip[0], sender_ip[1], sender_ip[2], sender_ip[3]
                        ),
                        format!("operation={}", frame.operation),
                    ],
                    mitre_techniques: vec![],
                    source_ip: None,
                    timestamp: None,
                    direction: None,
                });

                // New IP via GARP (no conflict means either no entry or same MAC).
                // Insert/update binding and set last_seen_ts.
                if !self.bindings.contains_key(&sender_ip) {
                    insert_binding_lru(&mut self.bindings, sender_ip, sender_mac, MAX_ARP_BINDINGS);
                }
                if let Some(entry) = self.bindings.get_mut(&sender_ip) {
                    entry.last_seen_ts = timestamp_secs;
                    entry.mac = sender_mac;
                }
            }

            return findings;
        }

        // (e)/(f) Non-GARP frame: update binding table.
        // BC-2.16.005/006 + BC-2.16.004 PC1 (D1 emission on rebind).
        // Architecture Compliance Rule 1: Step 4 (MAC update) is LAST.
        if let Some(entry) = self.bindings.get_mut(&sender_ip) {
            // Entry exists — update last_seen_ts unconditionally (AC-021, LRU correctness).
            entry.last_seen_ts = timestamp_secs;
            if entry.mac != sender_mac {
                // Rebind detected (MAC changed).
                // Capture old MAC BEFORE Steps 1–3 modify the entry (AC-016).
                let old_mac = entry.mac;

                // Steps 1–3: emit D1 finding with escalation logic.
                let d1 = Self::emit_d1_spoof_finding_impl(
                    entry,
                    sender_ip,
                    old_mac,
                    sender_mac,
                    timestamp_secs,
                    self.spoof_threshold,
                );
                self.spoof_findings += 1;
                findings.push(d1);

                // Step 4 (Architecture Compliance Rule 1): MAC update AFTER emission.
                // Re-borrow entry after the immutable borrow in emit_d1_spoof_finding_impl.
                let entry = self
                    .bindings
                    .get_mut(&sender_ip)
                    .expect("entry must still exist");
                entry.mac = sender_mac;
            }
        } else {
            // New IP: call insert_binding_lru (handles eviction), then set last_seen_ts.
            insert_binding_lru(&mut self.bindings, sender_ip, sender_mac, MAX_ARP_BINDINGS);
            if let Some(entry) = self.bindings.get_mut(&sender_ip) {
                entry.last_seen_ts = timestamp_secs;
            }
        }

        findings
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
    pub fn record_malformed(&mut self, packet_len: usize) -> Vec<Finding> {
        use crate::findings::{Confidence, ThreatCategory, Verdict};

        // Always increment malformed_frames (BC-2.16.009 PC4; AC-012).
        self.malformed_frames += 1;

        // Increment malformed_findings — this method is only called from the
        // --arp-gated path in main.rs (BC-2.16.009 PC3/PC4; AC-012).
        self.malformed_findings += 1;

        // Construct and return the D11 LOW/Anomaly Finding (BC-2.16.009 PC3).
        // mitre_techniques: [] — T0814 withheld per DF-VALIDATION-001 / BC-2.16.009 Invariant 3.
        vec![Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Possible,
            confidence: Confidence::Low,
            summary: "D11: Malformed ARP frame — Non-Ethernet/IPv4 ARP frame (non-standard \
                      HW/proto address sizes or types)"
                .to_string(),
            evidence: vec![
                "Non-Ethernet/IPv4 ARP frame".to_string(),
                format!("packet_len={packet_len}"),
            ],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        }]
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
        use std::collections::BTreeMap;

        let mut detail: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        detail.insert(
            "frames_analyzed".to_string(),
            serde_json::json!(self.frames_analyzed),
        );
        detail.insert(
            "request_count".to_string(),
            serde_json::json!(self.request_count),
        );
        detail.insert(
            "reply_count".to_string(),
            serde_json::json!(self.reply_count),
        );
        detail.insert(
            "other_opcode_count".to_string(),
            serde_json::json!(self.other_opcode_count),
        );
        detail.insert(
            "bindings_tracked".to_string(),
            serde_json::json!(self.bindings.len() as u64),
        );
        detail.insert(
            "spoof_findings".to_string(),
            serde_json::json!(self.spoof_findings),
        );
        detail.insert(
            "garp_findings".to_string(),
            serde_json::json!(self.garp_findings),
        );
        detail.insert(
            "storm_findings".to_string(),
            serde_json::json!(self.storm_findings),
        );
        detail.insert(
            "mismatch_findings".to_string(),
            serde_json::json!(self.mismatch_findings),
        );
        detail.insert(
            "malformed_findings".to_string(),
            serde_json::json!(self.malformed_findings),
        );
        detail.insert(
            "malformed_frames".to_string(),
            serde_json::json!(self.malformed_frames),
        );

        AnalysisSummary {
            analyzer_name: "ARP".to_string(),
            packets_analyzed: self.frames_analyzed,
            detail,
        }
    }

    // -----------------------------------------------------------------------
    // STORY-114 detection helpers (Green step — wired into process_arp above).
    // -----------------------------------------------------------------------

    /// Evaluate and emit a D1 ARP Spoof finding for a rebind event.
    ///
    /// Implements BC-2.16.004 postcondition 1, Steps 1–3 exactly.
    /// Step 4 (MAC update) is performed by `process_arp` AFTER this helper returns.
    ///
    /// - `entry`: mutable binding entry for the sender IP.
    /// - `sender_ip`: the IP address being rebound.
    /// - `old_mac`: the MAC stored in the binding table BEFORE this rebind
    ///   (captured by the caller before any entry mutation).
    /// - `new_mac`: the MAC from the current frame (frame.sender_mac).
    /// - `timestamp_secs`: frame timestamp (seconds).
    /// - `spoof_threshold`: rebind count at which HIGH is emitted.
    ///
    /// Returns the D1 Finding (confidence = HIGH or MEDIUM).
    fn emit_d1_spoof_finding_impl(
        entry: &mut BindingEntry,
        sender_ip: [u8; 4],
        old_mac: [u8; 6],
        new_mac: [u8; 6],
        timestamp_secs: u32,
        spoof_threshold: u32,
    ) -> Finding {
        use crate::findings::{Confidence, ThreatCategory, Verdict};

        // Flap-window reset (BC-2.16.004 PC5 / AC-004 / EC-007):
        // If a prior rebind window exists AND the window has expired, reset state
        // BEFORE Step 1 so the new rebind is treated as the first in a fresh window.
        if let Some(first_ts) = entry.first_rebind_ts {
            let elapsed = timestamp_secs.saturating_sub(first_ts);
            if elapsed > ARP_FLAP_WINDOW_SECS {
                entry.rebind_count = 0;
                entry.first_rebind_ts = None;
                entry.spoof_high_emitted = false;
            }
        }

        // Step 1: increment rebind_count (BC-2.16.004 PC1.a).
        entry.rebind_count += 1;

        // Step 2: set first_rebind_ts if currently None (BC-2.16.004 PC1.b).
        if entry.first_rebind_ts.is_none() {
            entry.first_rebind_ts = Some(timestamp_secs);
        }

        // Step 3: evaluate HIGH vs MEDIUM (BC-2.16.004 PC1.c/1.d).
        let first_ts = entry.first_rebind_ts.expect("set in Step 2");
        let elapsed = timestamp_secs.saturating_sub(first_ts);
        let confidence = if entry.rebind_count >= spoof_threshold
            && elapsed <= ARP_FLAP_WINDOW_SECS
            && !entry.spoof_high_emitted
        {
            // Escalate to HIGH and set one-shot guard (PC1.c / AC-002 / AC-003).
            entry.spoof_high_emitted = true;
            Confidence::High
        } else {
            Confidence::Medium
        };

        // Step 4 (MAC update) is intentionally NOT done here — caller is responsible.

        // Build evidence: sender_ip, old_mac, new_mac (AC-016 / BC-2.16.004 PC1.e).
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Possible,
            confidence,
            summary: format!(
                "D1: ARP Spoof — IP→MAC rebind detected for sender_ip={}.{}.{}.{}",
                sender_ip[0], sender_ip[1], sender_ip[2], sender_ip[3]
            ),
            evidence: vec![
                format!(
                    "sender_ip={}.{}.{}.{}",
                    sender_ip[0], sender_ip[1], sender_ip[2], sender_ip[3]
                ),
                format!(
                    "old_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    old_mac[0], old_mac[1], old_mac[2], old_mac[3], old_mac[4], old_mac[5]
                ),
                format!(
                    "new_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    new_mac[0], new_mac[1], new_mac[2], new_mac[3], new_mac[4], new_mac[5]
                ),
            ],
            mitre_techniques: vec!["T0830".to_string(), "T1557.002".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    /// Apply GARP-that-conflicts escalation and co-emit a D1 finding.
    ///
    /// Called when `is_gratuitous_arp(frame) == true` AND the binding table
    /// already contains `sender_ip` with a different MAC (conflict).
    ///
    /// BC-2.16.014 postcondition 1/2:
    ///   - Upgrades the GARP finding from LOW to MEDIUM and attaches T0830/T1557.002.
    ///   - Co-emits a D1 spoof finding (Steps 1–3 severity evaluation).
    ///
    /// Returns `(upgraded_garp_finding, d1_finding)`.
    fn apply_garp_conflict_escalation_impl(
        entry: &mut BindingEntry,
        garp_low: Finding,
        sender_ip: [u8; 4],
        new_mac: [u8; 6],
        timestamp_secs: u32,
        spoof_threshold: u32,
    ) -> (Finding, Finding) {
        use crate::findings::Confidence;

        // Upgrade GARP LOW → MEDIUM; attach T0830/T1557.002 (BC-2.16.014 PC1).
        let upgraded_garp = Finding {
            confidence: Confidence::Medium,
            mitre_techniques: vec!["T0830".to_string(), "T1557.002".to_string()],
            ..garp_low
        };

        // Capture old MAC BEFORE Steps 1–3 mutate the entry (AC-016).
        let old_mac = entry.mac;

        // Co-emit D1 using same Steps 1–3 logic (BC-2.16.014 PC2).
        let d1 = Self::emit_d1_spoof_finding_impl(
            entry,
            sender_ip,
            old_mac,
            new_mac,
            timestamp_secs,
            spoof_threshold,
        );

        (upgraded_garp, d1)
    }
}

impl Default for ArpAnalyzer {
    /// GREEN-BY-DESIGN: delegates to `new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT)`.
    /// Zero branching, no I/O, no non-trivial helpers, 1 line.
    fn default() -> Self {
        Self::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT)
    }
}

// ---------------------------------------------------------------------------
// Test affordances (ADR-008 Decision 4; VP-024 Sub-C)
// ---------------------------------------------------------------------------

#[cfg(test)]
impl ArpAnalyzer {
    /// Construct an `ArpAnalyzer` for use in unit tests.
    ///
    /// Uses default thresholds (`SPOOF_REBIND_ESCALATION_DEFAULT = 3`,
    /// `ARP_STORM_RATE_DEFAULT = 50`) so STORY-113 tests are unaffected
    /// by the STORY-114 signature extension to `new(spoof_threshold, storm_rate)`.
    /// Kept parameterless for VP-024 Sub-C proptest compatibility.
    ///
    /// ADR-008 Decision 4; VP-024 Sub-C anchor (BC-2.16.005).
    pub fn new_for_test() -> Self {
        Self::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT)
    }

    /// Process a frame using a fixed-cap binding table for Sub-C/Sub-D tests.
    ///
    /// Calls `process_arp` with the given frame and timestamp.
    /// Exists so tests can drive the analyzer without constructing CLI args.
    ///
    /// ADR-008 Decision 4 extension; VP-024 Sub-C.
    pub fn process_arp_for_test(&mut self, frame: &ArpFrame, timestamp_secs: u32) -> Vec<Finding> {
        self.process_arp(frame, timestamp_secs)
    }

    /// Return a snapshot of the current binding table for assertion in tests.
    ///
    /// Returns a clone of `self.bindings` so tests can inspect the table state
    /// after processing a sequence of frames.
    ///
    /// VP-024 Sub-C (`test_binding_table_last_write_wins` proptest).
    pub fn bindings_snapshot(&self) -> HashMap<[u8; 4], BindingEntry> {
        self.bindings.clone()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(unreachable_code)]
mod tests {
    //! STORY-113 TDD Red Gate test suite — unit + proptest tests.
    //!
    //! These tests exercise the behavioral contracts for:
    //!   BC-2.16.003 — Gratuitous ARP Detection (AC-001/002/003/004)
    //!   BC-2.16.005 — Binding-Table Update Last-Write-Wins (AC-005/006/007/021)
    //!   BC-2.16.006 — Binding-Table Cap (AC-008)
    //!   BC-2.16.007 — D12 L2/L3 Sender Mismatch (AC-009/010/020)
    //!   BC-2.16.009 — D11 Malformed ARP (AC-011/012)
    //!   BC-2.16.010 — summarize() 11 keys (AC-013/014)
    //!   VP-024 Sub-C — last-write-wins proptest (AC-018)
    //!
    //! All tests pass (GREEN). CLI integration tests for
    //! AC-015/AC-016 live in tests/bc_2_16_story113_arp_tests.rs.
    //!
    //! DF-TEST-NAMESPACE-001: all tests are within this `mod tests` block.
    //! DF-AC-TEST-NAME-SYNC-001: function names are the canonical names cited
    //! in the story's Test Plan; the orchestrator will back-fill AC `Test:`
    //! citations to match these exact names.

    use super::*;
    use crate::decoder::ArpFrame;
    use crate::findings::{Confidence, ThreatCategory};
    use proptest::prelude::*;

    // -----------------------------------------------------------------------
    // Helpers — canonical RFC 826 frame builders
    // (DF-CANONICAL-FRAME-HOLDOUT-001: htype=0x0001, ptype=0x0800, hlen=6, plen=4)
    // -----------------------------------------------------------------------

    /// Build a minimal valid ArpFrame with caller-supplied fields.
    /// operation: 1=Request, 2=Reply per RFC 826.
    fn make_arp_frame(
        operation: u16,
        sender_mac: [u8; 6],
        sender_ip: [u8; 4],
        target_ip: [u8; 4],
        outer_src_mac: Option<[u8; 6]>,
    ) -> ArpFrame {
        ArpFrame {
            operation,
            sender_mac,
            sender_ip,
            target_mac: [0u8; 6],
            target_ip,
            outer_src_mac,
            packet_len: 42,
        }
    }

    /// Build a GARP frame (sender_ip == target_ip).
    fn make_garp_frame(operation: u16, ip: [u8; 4], mac: [u8; 6]) -> ArpFrame {
        make_arp_frame(operation, mac, ip, ip, Some(mac))
    }

    /// Build a normal (non-GARP) ARP Reply frame.
    fn make_normal_reply(sender_ip: [u8; 4], sender_mac: [u8; 6], target_ip: [u8; 4]) -> ArpFrame {
        make_arp_frame(2, sender_mac, sender_ip, target_ip, Some(sender_mac))
    }

    // -----------------------------------------------------------------------
    // AC-001 — BC-2.16.003 PC1/PC2: is_gratuitous_arp biconditional
    // -----------------------------------------------------------------------

    /// AC-001a (BC-2.16.003 PC1): is_gratuitous_arp returns true when sender_ip == target_ip.
    /// Canonical vector: op=2, sender_ip=192.168.1.1, target_ip=192.168.1.1.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_003_is_gratuitous_arp_true_when_sender_eq_target_ip() {
        let frame = make_garp_frame(2, [192, 168, 1, 1], [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);
        // BC-2.16.003 PC1: must return true when sender_ip == target_ip
        let result = is_gratuitous_arp(&frame);
        assert!(
            result,
            "BC-2.16.003 PC1: is_gratuitous_arp must return true when \
             sender_ip == target_ip ([192,168,1,1] == [192,168,1,1]). \
             Got false."
        );
    }

    /// AC-001b (BC-2.16.003 PC2): is_gratuitous_arp returns false when sender_ip != target_ip.
    /// Canonical vector: op=2, sender_ip=192.168.1.1, target_ip=192.168.1.2.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_003_is_gratuitous_arp_false_when_sender_ne_target_ip() {
        let frame = make_arp_frame(
            2,
            [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
            [192, 168, 1, 1],
            [192, 168, 1, 2], // target_ip != sender_ip
            Some([0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]),
        );
        let result = is_gratuitous_arp(&frame);
        assert!(
            !result,
            "BC-2.16.003 PC2: is_gratuitous_arp must return false when \
             sender_ip != target_ip ([192,168,1,1] != [192,168,1,2]). \
             Got true."
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 — BC-2.16.003 PC3 / Invariant 2: opcode agnosticism
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.16.003 PC3/Invariant 2): is_gratuitous_arp returns true for op=1 and
    /// op=2 when sender_ip == target_ip. The function does NOT inspect the operation field.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_003_is_gratuitous_arp_opcode_agnostic() {
        let ip: [u8; 4] = [10, 0, 0, 5];
        let mac: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

        // op=1 (Request GARP — RFC 5227 ACD announcement form)
        let garp_request = make_garp_frame(1, ip, mac);
        assert!(
            is_gratuitous_arp(&garp_request),
            "BC-2.16.003 PC3/Invariant 2: is_gratuitous_arp must return true for \
             op=1 GARP Request (sender_ip == target_ip). Function must be opcode-agnostic."
        );

        // op=2 (Reply GARP — classic form)
        let garp_reply = make_garp_frame(2, ip, mac);
        assert!(
            is_gratuitous_arp(&garp_reply),
            "BC-2.16.003 PC3/Invariant 2: is_gratuitous_arp must return true for \
             op=2 GARP Reply (sender_ip == target_ip). Function must be opcode-agnostic."
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 — BC-2.16.003 PC5: GARP finding at LOW/Anomaly, mitre_techniques empty
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.16.003 PC5): process_arp emits one Finding with confidence=LOW,
    /// finding_type=Anomaly, description indicating GARP, mitre_techniques=[] for
    /// a benign (non-conflicting) GARP frame.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_003_process_arp_garp_emits_low_anomaly_finding() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        // Canonical vector: op=2, sender_ip=192.168.1.1, target_ip=192.168.1.1 (no prior binding)
        let frame = make_garp_frame(2, [192, 168, 1, 1], [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF]);

        let findings = analyzer.process_arp_for_test(&frame, 1_700_000_000);

        // Must emit at least one finding (the GARP D2 finding)
        let garp_finding = findings
            .iter()
            .find(|f| matches!(f.category, ThreatCategory::Anomaly))
            .expect(
                "AC-003 / BC-2.16.003 PC5: process_arp must emit a Finding with \
                 finding_type=Anomaly for a GARP frame (sender_ip==target_ip). \
                 Got 0 findings.",
            );

        // confidence: LOW (BC-2.16.003 PC5)
        assert_eq!(
            garp_finding.confidence,
            Confidence::Low,
            "AC-003 / BC-2.16.003 PC5: GARP Finding must have confidence=LOW. \
             Got {:?}",
            garp_finding.confidence
        );

        // finding_type: Anomaly (asserted by the find() above — double-check)
        assert!(
            matches!(garp_finding.category, ThreatCategory::Anomaly),
            "AC-003 / BC-2.16.003 PC5: GARP Finding must have finding_type=Anomaly"
        );

        // description must indicate Gratuitous ARP (case-insensitive substring check)
        assert!(
            garp_finding.summary.to_lowercase().contains("garp")
                || garp_finding.summary.to_lowercase().contains("gratuitous"),
            "AC-003 / BC-2.16.003 PC5: GARP Finding description must indicate \
             Gratuitous ARP. Got: {:?}",
            garp_finding.summary
        );

        // mitre_techniques: [] (D-068 adjudication — no AiTM attribution for benign GARP)
        assert!(
            garp_finding.mitre_techniques.is_empty(),
            "AC-003 / BC-2.16.003 PC5: GARP Finding mitre_techniques must be [] \
             (empty) for benign non-conflicting GARP. T0830/T1557.002 only via \
             BC-2.16.014 GARP-that-conflicts path (STORY-114). Got: {:?}",
            garp_finding.mitre_techniques
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 — BC-2.16.003 PC6: one GARP finding per GARP frame (no one-shot guard)
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.16.003 PC6): exactly one GARP finding per GARP frame.
    /// 10 consecutive GARP frames → 10 findings total. No cross-frame deduplication.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_003_process_arp_garp_emits_per_frame() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let ip: [u8; 4] = [10, 0, 0, 1];
        let mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let mut total_garp_findings: usize = 0;

        for i in 0u32..10 {
            let frame = make_garp_frame(2, ip, mac);
            let findings = analyzer.process_arp_for_test(&frame, 1_700_000_000 + i);
            let garp_count = findings
                .iter()
                .filter(|f| {
                    matches!(f.category, ThreatCategory::Anomaly)
                        && (f.summary.to_lowercase().contains("garp")
                            || f.summary.to_lowercase().contains("gratuitous"))
                })
                .count();
            // Each frame must emit exactly one GARP finding
            assert_eq!(
                garp_count, 1,
                "AC-004 / BC-2.16.003 PC6: each GARP frame must emit exactly 1 GARP \
                 finding (no cross-frame deduplication). Frame #{i} emitted {garp_count} \
                 GARP findings."
            );
            total_garp_findings += garp_count;
        }

        assert_eq!(
            total_garp_findings, 10,
            "AC-004 / BC-2.16.003 PC6: 10 GARP frames must produce exactly 10 GARP \
             findings. Got {total_garp_findings}."
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 — BC-2.16.005 PC1: last-write-wins binding update (basic)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.16.005 PC1): after process_arp completes, bindings[sender_ip].mac
    /// equals the MAC from the most recently processed frame.
    /// Sequence: two frames for same IP, different MACs → binding holds last MAC.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_005_binding_table_last_write_wins_basic() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let ip: [u8; 4] = [10, 0, 0, 1];
        let mac_first: [u8; 6] = [0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
        let mac_last: [u8; 6] = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];

        // First frame — establishes binding with mac_first
        let frame1 = make_normal_reply(ip, mac_first, [10, 0, 0, 2]);
        let _ = analyzer.process_arp_for_test(&frame1, 1_000);

        // Second frame — same IP, different MAC → must overwrite (last-write-wins)
        let frame2 = make_normal_reply(ip, mac_last, [10, 0, 0, 2]);
        let _ = analyzer.process_arp_for_test(&frame2, 2_000);

        let bindings = analyzer.bindings_snapshot();
        let entry = bindings.get(&ip).expect(
            "AC-005 / BC-2.16.005 PC1: binding table must contain entry for sender_ip \
             [10,0,0,1] after processing two frames with that IP.",
        );

        assert_eq!(
            entry.mac, mac_last,
            "AC-005 / BC-2.16.005 PC1: last-write-wins violated — binding[10.0.0.1].mac \
             must equal MAC from the last frame ({:?}) not the first ({:?}). Got: {:?}",
            mac_last, mac_first, entry.mac
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 — BC-2.16.005 PC4/Invariant 3: first-observation initializes entry correctly
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.16.005 PC4/Invariant 3): first-time observation inserts entry with
    /// rebind_count=0, first_rebind_ts=None, spoof_high_emitted=false. No finding emitted.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_005_binding_first_observation_no_finding() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let ip: [u8; 4] = [172, 16, 0, 1];
        let mac: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];

        let frame = make_normal_reply(ip, mac, [172, 16, 0, 2]);
        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        // No spoof finding on first observation (BC-2.16.005 Invariant 3)
        let spoof_findings: Vec<_> = findings
            .iter()
            .filter(|f| {
                matches!(f.confidence, Confidence::Medium | Confidence::High)
                    && (f.summary.to_lowercase().contains("spoof")
                        || f.mitre_techniques
                            .iter()
                            .any(|t| t == "T0830" || t == "T1557.002"))
            })
            .collect();
        assert!(
            spoof_findings.is_empty(),
            "AC-006 / BC-2.16.005 Invariant 3: no spoof finding must be emitted on \
             first observation of an IP. Got {} spoof-like finding(s).",
            spoof_findings.len()
        );

        // Binding entry must exist with correct initialization
        let bindings = analyzer.bindings_snapshot();
        let entry = bindings.get(&ip).expect(
            "AC-006 / BC-2.16.005 PC4: binding table must contain entry for first-observed \
             sender_ip [172,16,0,1].",
        );

        assert_eq!(
            entry.rebind_count, 0,
            "AC-006 / BC-2.16.005 PC4: first-observation entry must have rebind_count=0. \
             Got {}",
            entry.rebind_count
        );
        assert_eq!(
            entry.first_rebind_ts, None,
            "AC-006 / BC-2.16.005 PC4: first-observation entry must have \
             first_rebind_ts=None. Got {:?}",
            entry.first_rebind_ts
        );
        assert!(
            !entry.spoof_high_emitted,
            "AC-006 / BC-2.16.005 PC4: first-observation entry must have \
             spoof_high_emitted=false."
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 — BC-2.16.005 Invariant 5: zero and broadcast sender IPs filtered
    // -----------------------------------------------------------------------

    /// AC-007a (BC-2.16.005 Invariant 5): zero sender_ip [0,0,0,0] must not be inserted.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_005_binding_zero_sender_ip_filtered() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let zero_ip: [u8; 4] = [0, 0, 0, 0];
        let mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

        // Frame with sender_ip = 0.0.0.0 (RFC 5227 ACD probe form)
        let frame = make_arp_frame(1, mac, zero_ip, [192, 168, 1, 1], Some(mac));
        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        // No binding must be inserted for 0.0.0.0
        let bindings = analyzer.bindings_snapshot();
        assert!(
            !bindings.contains_key(&zero_ip),
            "AC-007 / BC-2.16.005 Invariant 5: sender_ip=0.0.0.0 must NOT be inserted \
             into the binding table. Found entry: {:?}",
            bindings.get(&zero_ip)
        );

        // No spoof finding for zero IP
        let spoof_like: Vec<_> = findings
            .iter()
            .filter(|f| {
                f.mitre_techniques
                    .iter()
                    .any(|t| t == "T0830" || t == "T1557.002")
            })
            .collect();
        assert!(
            spoof_like.is_empty(),
            "AC-007 / BC-2.16.005 Invariant 5: no spoof finding must be emitted for \
             sender_ip=0.0.0.0."
        );
    }

    /// AC-007b (BC-2.16.005 Invariant 5): broadcast sender_ip [255,255,255,255] must not
    /// be inserted into the binding table.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_005_binding_broadcast_sender_ip_filtered() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let broadcast_ip: [u8; 4] = [255, 255, 255, 255];
        let mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

        let frame = make_arp_frame(2, mac, broadcast_ip, [192, 168, 1, 1], Some(mac));
        let _ = analyzer.process_arp_for_test(&frame, 1_000);

        let bindings = analyzer.bindings_snapshot();
        assert!(
            !bindings.contains_key(&broadcast_ip),
            "AC-007 / BC-2.16.005 Invariant 5: sender_ip=255.255.255.255 must NOT be \
             inserted into the binding table. Found entry: {:?}",
            bindings.get(&broadcast_ip)
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 — BC-2.16.006 PC2: binding table cap enforced at MAX_ARP_BINDINGS
    // Uses insert_binding_lru_btree (BTreeMap surrogate, cfg(any(kani, test)))
    // to avoid running 65_537 iterations in the HashMap variant.
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.16.006 PC2): bindings.len() NEVER exceeds TEST_MAX_ARP_BINDINGS (8)
    /// when inserting cap+1 entries via insert_binding_lru_btree.
    /// Uses the BTreeMap Kani surrogate to keep the test efficient.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_006_binding_table_cap_enforced() {
        let cap = TEST_MAX_ARP_BINDINGS; // 8 in test context
        let mut bindings: std::collections::BTreeMap<[u8; 4], BindingEntry> =
            std::collections::BTreeMap::new();

        // Insert cap+1 = 9 distinct IPs, each with a unique timestamp so LRU is deterministic.
        for i in 0u8..=(cap as u8) {
            let ip: [u8; 4] = [10, 0, 0, i + 1];
            let mac: [u8; 6] = [i; 6];
            // Pre-populate last_seen_ts as required by insert_binding_lru contract:
            // process_arp writes last_seen_ts before calling insert_binding_lru.
            // We do the equivalent here: insert a pre-entry or rely on insert_binding_lru_btree
            // to use the mac/ts we provide. The btree variant writes the entry like the HashMap:
            // the entry's last_seen_ts is set to `i as u32` to make LRU deterministic.
            insert_binding_lru_btree(&mut bindings, ip, mac, cap);

            assert!(
                bindings.len() <= cap,
                "AC-008 / BC-2.16.006 PC2: binding table must NEVER exceed \
                 TEST_MAX_ARP_BINDINGS={} entries. After inserting {} IPs, \
                 bindings.len()={}.",
                cap,
                i + 1,
                bindings.len()
            );
        }

        // After cap+1 insertions, length must be exactly cap (one eviction occurred)
        assert_eq!(
            bindings.len(),
            cap,
            "AC-008 / BC-2.16.006 PC2: after inserting cap+1={} distinct IPs, \
             bindings.len() must equal cap={}. Got {}.",
            cap + 1,
            cap,
            bindings.len()
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 — BC-2.16.007 PC1: D12 mismatch emits MEDIUM/Anomaly finding
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.16.007 PC1): outer_src_mac != sender_mac → MEDIUM/Anomaly Finding
    /// with mitre_techniques=[] (wave 42 intermediate state per BC-2.16.007 cross-story note).
    /// Evidence must include eth_mac, arp_sender_mac, and sender_ip.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_007_d12_mismatch_emits_medium_finding() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let eth_mac: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let arp_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let sender_ip: [u8; 4] = [192, 168, 1, 1];

        // Frame where outer_src_mac != sender_mac (D12 mismatch condition)
        let frame = make_arp_frame(
            2,
            arp_mac, // ARP sender HW addr (in ARP payload)
            sender_ip,
            [192, 168, 1, 2],
            Some(eth_mac), // Ethernet src MAC (different from arp_mac)
        );

        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        let mismatch_finding = findings
            .iter()
            .find(|f| {
                matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
            })
            .expect(
                "AC-009 / BC-2.16.007 PC1: process_arp must emit a MEDIUM/Anomaly Finding \
                 when outer_src_mac != sender_mac (D12 mismatch). Got 0 matching findings.",
            );

        // confidence: MEDIUM (BC-2.16.007 PC1)
        assert_eq!(
            mismatch_finding.confidence,
            Confidence::Medium,
            "AC-009 / BC-2.16.007 PC1: D12 Finding must have confidence=MEDIUM."
        );

        // mitre_techniques: ["T0830", "T1557.002"] (wave 43 / STORY-114 back-fill;
        // BC-2.16.007 cross-story delivery note — see AC-017 / STORY-114 test_d12_mismatch_carries_mitre_after_catalog).
        // STORY-113 TDD sibling-sweep update: this assertion is updated from mitre=[] (wave 42)
        // to mitre=["T0830","T1557.002"] (wave 43 final state). It now FAILS until the
        // VP-007 5-part atomic update is applied (RED for STORY-114 D12 MITRE back-fill).
        let mut d12_techs = mismatch_finding.mitre_techniques.clone();
        d12_techs.sort();
        assert_eq!(
            d12_techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-009 / BC-2.16.007 PC1 (wave 43 sibling-sweep): D12 Finding mitre_techniques \
             must be [\"T0830\", \"T1557.002\"] after STORY-114 VP-007 5-part atomic update. \
             Currently [] (wave 42 intermediate). RED until VP-007 atomic update applied. \
             Got: {:?}",
            mismatch_finding.mitre_techniques
        );

        // Evidence must reference the MAC addresses and IP (BC-2.16.007 PC1 evidence clause).
        // Assert on the actual content, not just non-emptiness (anti-proxy-assertion).
        let evidence_joined = mismatch_finding.evidence.join(" ");
        assert!(
            evidence_joined.contains("eth_src_mac=11:22:33:44:55:66"),
            "AC-009 / BC-2.16.007 PC1: D12 Finding evidence must contain \
             'eth_src_mac=11:22:33:44:55:66' (Ethernet src MAC). Got: {:?}",
            mismatch_finding.evidence
        );
        assert!(
            evidence_joined.contains("arp_sender_mac=AA:BB:CC:DD:EE:FF"),
            "AC-009 / BC-2.16.007 PC1: D12 Finding evidence must contain \
             'arp_sender_mac=AA:BB:CC:DD:EE:FF' (ARP sender HW addr). Got: {:?}",
            mismatch_finding.evidence
        );
        assert!(
            evidence_joined.contains("sender_ip=192.168.1.1"),
            "AC-009 / BC-2.16.007 PC1: D12 Finding evidence must contain \
             'sender_ip=192.168.1.1' (ARP sender protocol addr). Got: {:?}",
            mismatch_finding.evidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-010 — BC-2.16.007 PC4/PC5: D12 skipped for None or matching MACs
    // -----------------------------------------------------------------------

    /// AC-010a (BC-2.16.007 PC4): outer_src_mac=None → no D12 finding.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_007_d12_skipped_when_outer_src_mac_none() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let sender_ip: [u8; 4] = [10, 0, 0, 1];

        // outer_src_mac = None (SLL capture)
        let frame = make_arp_frame(2, mac, sender_ip, [10, 0, 0, 2], None);
        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        // No MEDIUM finding (D12 not triggered when outer_src_mac is None)
        let d12_findings: Vec<_> = findings
            .iter()
            .filter(|f| matches!(f.confidence, Confidence::Medium))
            .collect();
        assert!(
            d12_findings.is_empty(),
            "AC-010 / BC-2.16.007 PC4: no D12 MEDIUM finding must be emitted when \
             outer_src_mac=None. Got {} medium finding(s).",
            d12_findings.len()
        );
    }

    /// AC-010b (BC-2.16.007 PC5): outer_src_mac=Some(mac) where mac == sender_mac → no D12.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_007_d12_skipped_when_macs_match() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let mac: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let sender_ip: [u8; 4] = [10, 0, 0, 1];

        // outer_src_mac == sender_mac (normal case — MACs agree)
        let frame = make_arp_frame(2, mac, sender_ip, [10, 0, 0, 2], Some(mac));
        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        let d12_findings: Vec<_> = findings
            .iter()
            .filter(|f| matches!(f.confidence, Confidence::Medium))
            .collect();
        assert!(
            d12_findings.is_empty(),
            "AC-010 / BC-2.16.007 PC5: no D12 MEDIUM finding must be emitted when \
             outer_src_mac == sender_mac (MACs agree). Got {} medium finding(s).",
            d12_findings.len()
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 — BC-2.16.009 PC3: D11 malformed ARP emits LOW/Anomaly Finding
    // (F-113-01): strengthened to assert Finding shape + evidence, not just counter.
    // Interface: record_malformed(&mut self, packet_len: usize) -> Vec<Finding>
    // -----------------------------------------------------------------------

    /// AC-011 (BC-2.16.009 PC3): record_malformed(packet_len) returns exactly one
    /// LOW/Anomaly Finding with mitre_techniques=[] and evidence containing both
    /// the canonical error string "Non-Ethernet/IPv4 ARP frame" and the packet_len
    /// value.
    ///
    /// F-113-01 (HIGH): the previous test only asserted a counter increment and
    /// passed against a non-conformant impl. This test asserts the complete Finding
    /// shape so it FAILS against the current impl (RED for F-113-01 / BC-2.16.009 PC3).
    ///
    /// Interface: `record_malformed(&mut self, packet_len: usize) -> Vec<Finding>`,
    /// mirroring `process_arp`'s return pattern so main.rs can do:
    ///   `all_findings.extend(arp_analyzer.record_malformed(packet_len));`
    ///
    /// Canonical test vector: packet_len = 36 (non-standard ARP payload length).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_009_d11_malformed_arp_emits_low_finding() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let packet_len: usize = 36; // canonical test vector — non-standard ARP payload length

        // record_malformed must return Vec<Finding> (target interface).
        // Current impl returns () → compile error is the intended RED (F-113-01).
        let findings = analyzer.record_malformed(packet_len);

        // PC3: exactly one Finding emitted
        assert_eq!(
            findings.len(),
            1,
            "AC-011 / BC-2.16.009 PC3: record_malformed({packet_len}) must return exactly \
             1 Finding (D11 malformed ARP). Got {} finding(s).",
            findings.len()
        );

        let f = &findings[0];

        // PC3: confidence == LOW
        assert_eq!(
            f.confidence,
            Confidence::Low,
            "AC-011 / BC-2.16.009 PC3: D11 Finding must have confidence=LOW. Got {:?}",
            f.confidence
        );

        // PC3: finding_type == Anomaly
        assert!(
            matches!(f.category, ThreatCategory::Anomaly),
            "AC-011 / BC-2.16.009 PC3: D11 Finding must have finding_type=Anomaly. \
             Got {:?}",
            f.category
        );

        // PC3: description indicates malformed ARP frame
        let summary_lower = f.summary.to_lowercase();
        assert!(
            summary_lower.contains("malformed")
                || summary_lower.contains("non-ethernet")
                || summary_lower.contains("d11"),
            "AC-011 / BC-2.16.009 PC3: D11 Finding description must indicate malformed ARP \
             frame. Got: {:?}",
            f.summary
        );

        // PC3: mitre_techniques == [] (T0814 withheld per DF-VALIDATION-001 / BC-2.16.009 Invariant 3)
        assert!(
            f.mitre_techniques.is_empty(),
            "AC-011 / BC-2.16.009 PC3: D11 Finding mitre_techniques must be [] \
             (T0814 withheld per DF-VALIDATION-001). Got: {:?}",
            f.mitre_techniques
        );

        // PC3 evidence clause (a): canonical error string from decode_packet Err path (BC-2.02.009)
        let evidence_joined = f.evidence.join("\n");
        assert!(
            evidence_joined.contains("Non-Ethernet/IPv4 ARP frame"),
            "AC-011 / BC-2.16.009 PC3: D11 Finding evidence must contain the error string \
             \"Non-Ethernet/IPv4 ARP frame\" (from decode_packet Err path). Evidence: {:?}",
            f.evidence
        );

        // PC3 evidence clause (b): packet_len present in evidence (catches discarded _packet_len)
        let packet_len_str = packet_len.to_string();
        assert!(
            evidence_joined.contains(&packet_len_str),
            "AC-011 / BC-2.16.009 PC3: D11 Finding evidence must contain packet_len \
             value ({packet_len}). This is the F-113-01 RED signal — current impl discards \
             _packet_len and never emits a Finding. Evidence: {:?}",
            f.evidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-012 — BC-2.16.009 PC4: malformed counter semantics
    // -----------------------------------------------------------------------

    /// AC-012 (BC-2.16.009 PC4): frames_analyzed NOT incremented for malformed frames.
    /// malformed_frames increments unconditionally on each record_malformed call.
    /// malformed_findings increments with record_malformed (--arp gate enforced in
    /// main.rs; record_malformed is the --arp-gated path per BC-2.16.009 PC6 note).
    ///
    /// Uses `let _ = analyzer.record_malformed(...)` to handle the Vec<Finding> return
    /// value introduced by the F-113-01 interface fix (record_malformed -> Vec<Finding>).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_009_d11_malformed_counter_semantics() {
        let mut analyzer = ArpAnalyzer::new_for_test();

        // Process one valid frame first (frames_analyzed should increment)
        let valid_frame = make_normal_reply([10, 0, 0, 1], [0xAA; 6], [10, 0, 0, 2]);
        let _ = analyzer.process_arp_for_test(&valid_frame, 1_000);

        // Now record 3 malformed frames; discard returned findings (counter test only)
        let _ = analyzer.record_malformed(28);
        let _ = analyzer.record_malformed(32);
        let _ = analyzer.record_malformed(20);

        let summary = analyzer.summarize();

        let frames_analyzed = summary
            .detail
            .get("frames_analyzed")
            .and_then(|v| v.as_u64())
            .expect("summarize() must have 'frames_analyzed' key");
        assert_eq!(
            frames_analyzed, 1,
            "AC-012 / BC-2.16.009 PC4: frames_analyzed must equal 1 (only the valid \
             frame is counted; malformed frames are NOT counted). Got {frames_analyzed}."
        );

        let malformed_frames = summary
            .detail
            .get("malformed_frames")
            .and_then(|v| v.as_u64())
            .expect("summarize() must have 'malformed_frames' key");
        assert_eq!(
            malformed_frames, 3,
            "AC-012 / BC-2.16.009 PC4: malformed_frames must equal 3 (unconditional \
             increment per malformed frame). Got {malformed_frames}."
        );

        let malformed_findings = summary
            .detail
            .get("malformed_findings")
            .and_then(|v| v.as_u64())
            .expect("summarize() must have 'malformed_findings' key");
        assert_eq!(
            malformed_findings, 3,
            "AC-012 / BC-2.16.009 PC4: malformed_findings must equal 3 when record_malformed \
             is called 3 times on the --arp-active path. Got {malformed_findings}."
        );
    }

    // -----------------------------------------------------------------------
    // AC-013 — BC-2.16.010 PC1/PC4: all eleven summary keys present, all 0 at zero frames
    // -----------------------------------------------------------------------

    /// AC-013a (BC-2.16.010 PC4): all eleven keys present with value 0 when no frames
    /// have been processed.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_010_summarize_zero_frames_all_eleven_keys_zero() {
        let analyzer = ArpAnalyzer::new_for_test();
        let summary = analyzer.summarize();

        // All eleven canonical keys must be present and equal 0 (BC-2.16.010 EC-001)
        const EXPECTED_KEYS: &[&str] = &[
            "frames_analyzed",
            "request_count",
            "reply_count",
            "other_opcode_count",
            "bindings_tracked",
            "spoof_findings",
            "garp_findings",
            "storm_findings",
            "mismatch_findings",
            "malformed_findings",
            "malformed_frames",
        ];

        for key in EXPECTED_KEYS {
            let val = summary.detail.get(*key).unwrap_or_else(|| {
                panic!(
                    "AC-013 / BC-2.16.010 PC4: summarize() must contain key '{key}' \
                     with value 0 when no frames processed. Key is MISSING."
                )
            });
            assert_eq!(
                val.as_u64(),
                Some(0),
                "AC-013 / BC-2.16.010 PC4: key '{key}' must have value 0 at zero frames. \
                 Got: {val}"
            );
        }
    }

    /// AC-013b (BC-2.16.010 Invariant 1): exactly eleven keys in detail — no extras, no fewer.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_010_summarize_key_names_exact() {
        let analyzer = ArpAnalyzer::new_for_test();
        let summary = analyzer.summarize();

        // Exact eleven key names per BC-2.16.010 PC1 (authority over any story body wording)
        let mut expected: std::collections::BTreeSet<&str> = [
            "frames_analyzed",
            "request_count",
            "reply_count",
            "other_opcode_count",
            "bindings_tracked",
            "spoof_findings",
            "garp_findings",
            "storm_findings",
            "mismatch_findings",
            "malformed_findings",
            "malformed_frames",
        ]
        .iter()
        .copied()
        .collect();

        let actual: std::collections::BTreeSet<&str> =
            summary.detail.keys().map(|k| k.as_str()).collect();

        assert_eq!(
            actual.len(),
            11,
            "AC-013 / BC-2.16.010 Invariant 1: summarize() must return exactly 11 \
             keys. Got {}. Keys present: {actual:?}",
            actual.len()
        );

        for key in &expected {
            assert!(
                actual.contains(key),
                "AC-013 / BC-2.16.010 PC1: required key '{key}' is MISSING from summarize() \
                 output. Actual keys: {actual:?}"
            );
        }

        // Remove known keys to detect any unexpected extras
        for key in actual.iter() {
            expected.remove(*key);
        }
        // If expected is now empty, all known keys were present — but check for extras
        let extras: std::collections::BTreeSet<&str> = summary
            .detail
            .keys()
            .map(|k| k.as_str())
            .filter(|k| {
                ![
                    "frames_analyzed",
                    "request_count",
                    "reply_count",
                    "other_opcode_count",
                    "bindings_tracked",
                    "spoof_findings",
                    "garp_findings",
                    "storm_findings",
                    "mismatch_findings",
                    "malformed_findings",
                    "malformed_frames",
                ]
                .contains(k)
            })
            .collect();
        assert!(
            extras.is_empty(),
            "AC-013 / BC-2.16.010 Invariant 1: summarize() must contain EXACTLY 11 keys. \
             Unexpected extra keys found: {extras:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-014 — BC-2.16.010 Invariant 3: reconciliation invariant
    // request_count + reply_count + other_opcode_count == frames_analyzed
    // -----------------------------------------------------------------------

    /// AC-014 (BC-2.16.010 Invariant 3): request_count + reply_count + other_opcode_count
    /// == frames_analyzed after processing a mixed sequence of frames.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_010_summarize_reconciliation_invariant() {
        let mut analyzer = ArpAnalyzer::new_for_test();

        // Process 3 Requests (op=1)
        for i in 0u8..3 {
            let frame = make_arp_frame(
                1,
                [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, i],
                [10, 0, 0, i + 1],
                [10, 0, 0, 100],
                None,
            );
            let _ = analyzer.process_arp_for_test(&frame, 1_000 + u32::from(i));
        }

        // Process 2 Replies (op=2)
        for i in 0u8..2 {
            let frame = make_arp_frame(
                2,
                [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, i + 10],
                [10, 0, 1, i + 1],
                [10, 0, 0, 100],
                None,
            );
            let _ = analyzer.process_arp_for_test(&frame, 2_000 + u32::from(i));
        }

        // Process 1 "other opcode" (op=5, not 1 or 2)
        let other_frame = make_arp_frame(5, [0xCC; 6], [10, 0, 2, 1], [10, 0, 0, 100], None);
        let _ = analyzer.process_arp_for_test(&other_frame, 3_000);

        // Malformed frames must NOT affect frames_analyzed
        let _ = analyzer.record_malformed(20);

        let summary = analyzer.summarize();

        let get_u64 = |key: &str| -> u64 {
            summary
                .detail
                .get(key)
                .and_then(|v| v.as_u64())
                .unwrap_or_else(|| panic!("key '{key}' missing from summarize()"))
        };

        let frames_analyzed = get_u64("frames_analyzed");
        let request_count = get_u64("request_count");
        let reply_count = get_u64("reply_count");
        let other_opcode_count = get_u64("other_opcode_count");

        assert_eq!(
            request_count + reply_count + other_opcode_count,
            frames_analyzed,
            "AC-014 / BC-2.16.010 Invariant 3: reconciliation invariant VIOLATED. \
             request_count({request_count}) + reply_count({reply_count}) + \
             other_opcode_count({other_opcode_count}) = {} \
             but frames_analyzed = {frames_analyzed}. Malformed frames must NOT be counted \
             in frames_analyzed.",
            request_count + reply_count + other_opcode_count
        );

        assert_eq!(
            frames_analyzed, 6,
            "AC-014 / BC-2.16.010 Invariant 3: frames_analyzed must equal 6 \
             (3 requests + 2 replies + 1 other-opcode; 1 malformed excluded). Got {frames_analyzed}."
        );

        assert_eq!(request_count, 3, "request_count must be 3");
        assert_eq!(reply_count, 2, "reply_count must be 2");
        assert_eq!(other_opcode_count, 1, "other_opcode_count must be 1");
    }

    // -----------------------------------------------------------------------
    // AC-018 — VP-024 Sub-C: test_binding_table_last_write_wins (proptest)
    // BC-2.16.005 PC1; VP-024 Sub-C anchor adjudication (PO, 2026-06-13).
    // Runs at `cargo test`.
    // -----------------------------------------------------------------------

    proptest! {
        /// AC-018 (BC-2.16.005 PC1 / VP-024 Sub-C): for any arbitrary sequence of
        /// (ip_octet, mac_byte, opcode) tuples, after processing all frames,
        /// bindings[ip].mac equals the mac_byte from the LAST frame with that ip.
        /// Uses new_for_test()/process_arp_for_test()/bindings_snapshot().
        #[test]
        #[allow(non_snake_case)]
        fn test_BC_2_16_005_binding_table_last_write_wins(
            entries in proptest::collection::vec(
                // ip_octet in 1..=254 keeps sender_ip non-zero and non-broadcast.
                // mac_byte is arbitrary. opcode is 1 or 2 (canonical RFC 826 values).
                (1u8..=254u8, any::<u8>(), proptest::sample::select(vec![1u16, 2u16])),
                0..=1000
            )
        ) {
            let mut analyzer = ArpAnalyzer::new_for_test();

            // Track what the last MAC was for each unique IP.
            let mut last_mac_for_ip: std::collections::HashMap<[u8; 4], [u8; 6]> =
                std::collections::HashMap::new();

            for (ts, (ip_octet, mac_byte, opcode)) in entries.iter().enumerate() {
                // Construct a routable non-zero, non-broadcast sender_ip from ip_octet.
                // ip_octet is in 1..=254 so sender_ip is never 0.0.0.0 or 255.255.255.255.
                let ip: [u8; 4] = [10, 0, 0, *ip_octet];
                let mac: [u8; 6] = [*mac_byte; 6];
                let target_ip: [u8; 4] = [10, 0, 1, 1]; // non-GARP: target_ip != sender_ip

                let frame = ArpFrame {
                    operation: *opcode,
                    sender_mac: mac,
                    sender_ip: ip,
                    target_mac: [0u8; 6],
                    target_ip,
                    outer_src_mac: Some(mac),
                    packet_len: 42,
                };

                let _ = analyzer.process_arp_for_test(&frame, ts as u32);
                last_mac_for_ip.insert(ip, mac);
            }

            // Assert last-write-wins for every IP in the sequence
            let bindings = analyzer.bindings_snapshot();
            for (ip, expected_mac) in &last_mac_for_ip {
                if let Some(entry) = bindings.get(ip) {
                    // If the entry is still present (not evicted due to table cap),
                    // it MUST hold the last MAC observed for that IP.
                    prop_assert_eq!(
                        &entry.mac,
                        expected_mac,
                        "VP-024 Sub-C / BC-2.16.005 PC1: last-write-wins violated for \
                         ip={:?}. Expected MAC {:?}, got {:?}.",
                        ip,
                        expected_mac,
                        &entry.mac
                    );
                }
                // If the entry was evicted (LRU, table cap), that is acceptable per
                // BC-2.16.005 Invariant 4 (eviction does not affect correctness).
            }
        }
    }

    // -----------------------------------------------------------------------
    // AC-020 — BC-2.16.007 Invariant 3 / EC-004: D12 and D2 GARP co-emit on single frame
    // -----------------------------------------------------------------------

    /// AC-020 (BC-2.16.007 Invariant 3): a frame where sender_ip==target_ip (GARP) AND
    /// outer_src_mac != sender_mac (D12 mismatch) produces exactly two findings:
    /// one D12 MEDIUM/Anomaly (mitre_techniques=[]) and one D2 LOW/Anomaly (mitre_techniques=[]).
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_007_d12_and_garp_coemit_on_single_frame() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let eth_mac: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let arp_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let ip: [u8; 4] = [10, 0, 0, 1];

        // GARP (sender_ip == target_ip) AND D12 mismatch (eth_mac != arp_mac)
        let frame = ArpFrame {
            operation: 2, // Reply GARP
            sender_mac: arp_mac,
            sender_ip: ip,
            target_mac: [0u8; 6],
            target_ip: ip,                // GARP: target_ip == sender_ip
            outer_src_mac: Some(eth_mac), // D12: eth_mac != arp_mac
            packet_len: 42,
        };

        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        // Must emit exactly 2 findings: one D12 (MEDIUM) and one D2 (LOW)
        assert_eq!(
            findings.len(),
            2,
            "AC-020 / BC-2.16.007 Invariant 3: a GARP+D12 frame must produce exactly \
             2 findings (one D12 MEDIUM, one D2 LOW). Got {} finding(s): {:?}",
            findings.len(),
            findings
                .iter()
                .map(|f| (&f.confidence, &f.category))
                .collect::<Vec<_>>()
        );

        // One must be MEDIUM/Anomaly (D12)
        let d12 = findings
            .iter()
            .find(|f| matches!(f.confidence, Confidence::Medium))
            .expect(
                "AC-020: must have one MEDIUM finding (D12 mismatch) when outer_src_mac \
                 != sender_mac AND sender_ip==target_ip.",
            );
        // AC-020 sibling-sweep (wave 43 / STORY-114): D12 now carries T0830+T1557.002
        // (co-committed with VP-007 5-part atomic update; AC-017 / BC-2.16.007 cross-story note).
        let mut d12_techs = d12.mitre_techniques.clone();
        d12_techs.sort();
        assert_eq!(
            d12_techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-020 sibling-sweep: D12 finding mitre_techniques must be [\"T0830\", \"T1557.002\"] \
             after STORY-114 VP-007 atomic update (wave 43 final state). Got: {:?}",
            d12.mitre_techniques
        );

        // One must be LOW/Anomaly (D2 GARP — benign: no prior binding for this IP)
        let d2 = findings
            .iter()
            .find(|f| matches!(f.confidence, Confidence::Low))
            .expect("AC-020: must have one LOW finding (D2 GARP) when sender_ip==target_ip.");
        assert!(
            d2.mitre_techniques.is_empty(),
            "AC-020: D2 GARP finding mitre_techniques must be [] (benign GARP — no prior binding \
             conflict; D-068 adjudication). Got: {:?}",
            d2.mitre_techniques
        );
    }

    // -----------------------------------------------------------------------
    // AC-021 — BC-2.16.005 PC5: same-MAC re-observation advances last_seen_ts
    // -----------------------------------------------------------------------

    /// AC-021 (BC-2.16.005 PC5): when process_arp processes a frame where
    /// sender_ip already has a binding AND sender_mac == binding.mac (same MAC, no rebind),
    /// last_seen_ts is updated and rebind_count remains unchanged.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_005_binding_same_mac_touches_last_seen_ts() {
        let mut analyzer = ArpAnalyzer::new_for_test();
        let ip: [u8; 4] = [192, 168, 1, 1];
        let mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];

        // First frame — establishes binding
        let frame1 = make_normal_reply(ip, mac, [192, 168, 1, 2]);
        let _ = analyzer.process_arp_for_test(&frame1, 1_000);

        // Capture rebind_count after first observation (must be 0)
        let bindings_before = analyzer.bindings_snapshot();
        let rebind_before = bindings_before
            .get(&ip)
            .expect("binding must exist after first observation")
            .rebind_count;
        let ts_before = bindings_before.get(&ip).unwrap().last_seen_ts;

        assert_eq!(
            rebind_before, 0,
            "rebind_count must be 0 after first observation"
        );

        // Second frame — SAME MAC (no rebind), later timestamp
        let frame2 = make_normal_reply(ip, mac, [192, 168, 1, 2]);
        let _ = analyzer.process_arp_for_test(&frame2, 2_000);

        let bindings_after = analyzer.bindings_snapshot();
        let entry_after = bindings_after
            .get(&ip)
            .expect("binding must still exist after second same-MAC frame");

        // rebind_count must remain 0 (same MAC, no rebind)
        assert_eq!(
            entry_after.rebind_count, 0,
            "AC-021 / BC-2.16.005 PC5: rebind_count must remain 0 on same-MAC \
             re-observation. Got {}.",
            entry_after.rebind_count
        );

        // last_seen_ts must be updated to the new timestamp
        assert!(
            entry_after.last_seen_ts >= ts_before,
            "AC-021 / BC-2.16.005 PC5: last_seen_ts must advance on same-MAC \
             re-observation (LRU correctness). Before: {}, after: {}.",
            ts_before,
            entry_after.last_seen_ts
        );
        // Specifically, must reflect the second frame's timestamp (2_000)
        assert_eq!(
            entry_after.last_seen_ts, 2_000,
            "AC-021 / BC-2.16.005 PC5: last_seen_ts must be updated to the most \
             recent frame's timestamp (2_000). Got {}.",
            entry_after.last_seen_ts
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-114 TDD Red Gate tests (BC-2.16.004, BC-2.16.014, BC-2.16.007 MITRE)
// ---------------------------------------------------------------------------

/// STORY-114 Red Gate test suite.
///
/// Every test in this module MUST FAIL before the Green step because:
///   - D1 spoof findings are not yet emitted (emit_d1_spoof_finding stub uncalled).
///   - T0830/T1557.002 are not yet in src/mitre.rs (SEEDED=23, EMITTED=15).
///   - GARP-conflict escalation is not yet wired (apply_garp_conflict_escalation uncalled).
///   - D12 finding still carries mitre_techniques=[] (will carry T0830+T1557.002 after update).
///
/// DF-TEST-NAMESPACE-001: wrapped in `mod story_114` per per-story namespace rule.
/// DF-AC-TEST-NAME-SYNC-001: function names exactly match the Test Plan fn-name column.
/// DF-CANONICAL-FRAME-HOLDOUT-001: all frames use RFC 826 values (op 1/2, htype 0x0001).
/// PG-ARP-F4-PROXY-COUNTER-TEST: all finding-emission tests assert the Finding object
///   fields (confidence, category, mitre_techniques, evidence content), never a bare counter.
#[cfg(test)]
mod story_114 {
    use super::*;
    use crate::decoder::ArpFrame;
    use crate::findings::{Confidence, ThreatCategory};

    // -----------------------------------------------------------------------
    // Shared frame builders (RFC 826 canonical values)
    // -----------------------------------------------------------------------

    /// Build a normal (non-GARP) ARP Reply (op=2) with matching outer_src_mac.
    fn make_reply(sender_ip: [u8; 4], sender_mac: [u8; 6]) -> ArpFrame {
        ArpFrame {
            operation: 2,
            sender_mac,
            sender_ip,
            target_mac: [0u8; 6],
            target_ip: [10, 0, 0, 100],
            outer_src_mac: Some(sender_mac),
            packet_len: 42,
        }
    }

    /// Build a GARP Reply (op=2, sender_ip == target_ip) with matching outer_src_mac.
    fn make_garp(ip: [u8; 4], mac: [u8; 6]) -> ArpFrame {
        ArpFrame {
            operation: 2,
            sender_mac: mac,
            sender_ip: ip,
            target_mac: [0u8; 6],
            target_ip: ip,
            outer_src_mac: Some(mac),
            packet_len: 42,
        }
    }

    // -----------------------------------------------------------------------
    // Canonical test vector constants (BC-2.16.004 Test Vectors table)
    // -----------------------------------------------------------------------

    const IP_A: [u8; 4] = [10, 0, 0, 1];
    const MAC_A: [u8; 6] = [0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
    const MAC_B: [u8; 6] = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];
    const MAC_C: [u8; 6] = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];
    const MAC_D: [u8; 6] = [0xDD, 0xDD, 0xDD, 0xDD, 0xDD, 0xDD];
    const MAC_E: [u8; 6] = [0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE];

    /// Helper: seed an initial binding for IP_A → MAC_A (first observation; no finding).
    fn seed_binding(analyzer: &mut ArpAnalyzer, ts: u32) {
        let frame = make_reply(IP_A, MAC_A);
        let findings = analyzer.process_arp_for_test(&frame, ts);
        assert!(
            findings.iter().all(|f| {
                !f.summary.to_lowercase().contains("spoof")
                    && !f.mitre_techniques.iter().any(|t| t == "T0830")
            }),
            "seed_binding: first observation must not emit a spoof finding"
        );
    }

    // -----------------------------------------------------------------------
    // AC-001 — BC-2.16.004 PC1.a-1.e: first rebind emits MEDIUM D1 finding
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.16.004 PC1.a–1.e): when sender_ip is in binding table with a
    /// different MAC (first rebind), process_arp emits exactly one D1 Finding with:
    ///   - confidence: MEDIUM
    ///   - category: Anomaly
    ///   - mitre_techniques: exactly ["T0830", "T1557.002"]
    ///
    /// Red Gate: fails because emit_d1_spoof_finding is an uncalled todo!() stub.
    /// Canonical test vector: binding {10.0.0.1 → AA:AA, rebind=0} → frame with BB:BB
    /// Expected: MEDIUM Finding + T0830+T1557.002 (BC-2.16.004 test vector row 3).
    #[test]
    #[allow(non_snake_case)]
    fn test_d1_first_rebind_emits_medium() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        // Seed initial binding: 10.0.0.1 → MAC_A at ts=0
        seed_binding(&mut analyzer, 0);

        // First rebind: MAC_A → MAC_B at ts=2
        let frame = make_reply(IP_A, MAC_B);
        let findings = analyzer.process_arp_for_test(&frame, 2);

        // PRIMARY assertion: find the D1 spoof finding
        let d1 = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.mitre_techniques.contains(&"T1557.002".to_string())
                || f.summary.to_lowercase().contains("spoof")
                || f.summary.to_lowercase().contains("rebind")
                || f.summary.to_lowercase().contains("d1")
        });

        // This assert_is_some IS the Red Gate — it will fail because no D1 finding is emitted
        assert!(
            d1.is_some(),
            "AC-001 / BC-2.16.004 PC1: process_arp must emit a D1 spoof Finding on first \
             rebind (MAC_A → MAC_B). Got {} finding(s): {:?}. \
             RED: emit_d1_spoof_finding stub is not wired into process_arp.",
            findings.len(),
            findings
                .iter()
                .map(|f| (&f.confidence, &f.summary))
                .collect::<Vec<_>>()
        );

        let d1 = d1.unwrap();

        // confidence: MEDIUM (BC-2.16.004 PC1.d — escalation condition NOT met: rebind=1 < threshold=3)
        assert_eq!(
            d1.confidence,
            Confidence::Medium,
            "AC-001 / BC-2.16.004 PC1.d: first rebind must emit MEDIUM confidence. Got {:?}",
            d1.confidence
        );

        // category: Anomaly
        assert!(
            matches!(d1.category, ThreatCategory::Anomaly),
            "AC-001 / BC-2.16.004 PC1.e: D1 finding must have category=Anomaly. Got {:?}",
            d1.category
        );

        // mitre_techniques: exactly ["T0830", "T1557.002"] (BC-2.16.004 PC1.e; VP-007)
        let mut techniques = d1.mitre_techniques.clone();
        techniques.sort();
        assert_eq!(
            techniques,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-001 / BC-2.16.004 PC1.e: D1 finding mitre_techniques must be EXACTLY \
             [\"T0830\", \"T1557.002\"]. Got: {:?}",
            d1.mitre_techniques
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 — BC-2.16.004 PC1.c: third rebind within 60s escalates to HIGH
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.16.004 PC1.c): after 3 rebinds within 60s (default threshold=3),
    /// the 3rd rebind emits HIGH confidence. spoof_high_emitted is set to true.
    ///
    /// Red Gate: fails because D1 emission is not wired (stub uncalled).
    /// Canonical test vectors (BC-2.16.004 table rows 3-5):
    ///   rebind=0 → MEDIUM; rebind=1 → MEDIUM; rebind=2 (count=3) → HIGH.
    #[test]
    #[allow(non_snake_case)]
    fn test_d1_escalates_to_high_at_threshold() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        // Seed: 10.0.0.1 → MAC_A at ts=0
        seed_binding(&mut analyzer, 0);

        // Rebind 1: MAC_A → MAC_B at ts=2 (rebind_count → 1, MEDIUM expected)
        let f1 = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_B), 2);
        let d1_1 = f1.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1_1.is_some(),
            "AC-002 / BC-2.16.004: rebind 1 must emit D1 finding. RED: stub not wired."
        );
        assert_eq!(
            d1_1.unwrap().confidence,
            Confidence::Medium,
            "AC-002 / BC-2.16.004: rebind 1 of 3 must be MEDIUM (count=1 < threshold=3)"
        );

        // Rebind 2: MAC_B → MAC_C at ts=5 (rebind_count → 2, MEDIUM expected)
        let f2 = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_C), 5);
        let d1_2 = f2.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1_2.is_some(),
            "AC-002 / BC-2.16.004: rebind 2 must emit D1 finding. RED: stub not wired."
        );
        assert_eq!(
            d1_2.unwrap().confidence,
            Confidence::Medium,
            "AC-002 / BC-2.16.004: rebind 2 of 3 must be MEDIUM (count=2 < threshold=3)"
        );

        // Rebind 3: MAC_C → MAC_D at ts=8 — count reaches threshold=3, HIGH expected
        let f3 = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_D), 8);
        let d1_3 = f3.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1_3.is_some(),
            "AC-002 / BC-2.16.004: rebind 3 must emit D1 finding. RED: stub not wired."
        );
        assert_eq!(
            d1_3.unwrap().confidence,
            Confidence::High,
            "AC-002 / BC-2.16.004 PC1.c: 3rd rebind within 60s must escalate to HIGH \
             (rebind_count=3 >= threshold=3, elapsed=8s <= 60s, spoof_high_emitted=false). \
             Got {:?}",
            d1_3.unwrap().confidence
        );

        // Verify spoof_high_emitted is now true in the binding table
        let bindings = analyzer.bindings_snapshot();
        let entry = bindings.get(&IP_A).expect("binding must exist");
        assert!(
            entry.spoof_high_emitted,
            "AC-002 / BC-2.16.004 PC4: spoof_high_emitted must be true after HIGH finding emitted"
        );

        // Verify mitre on the HIGH finding
        let mut techs = d1_3.unwrap().mitre_techniques.clone();
        techs.sort();
        assert_eq!(
            techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-002: HIGH D1 finding must carry T0830+T1557.002"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 — BC-2.16.004 PC4: one-shot HIGH guard prevents second HIGH
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.16.004 PC4): after spoof_high_emitted=true, subsequent rebinds
    /// within the same flap window emit MEDIUM (not HIGH again).
    ///
    /// Red Gate: fails because D1 emission is not wired.
    #[test]
    #[allow(non_snake_case)]
    fn test_d1_high_guard_prevents_second_high() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        seed_binding(&mut analyzer, 0);

        // Drive to threshold: 3 rebinds within 60s
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_B), 2);
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_C), 5);
        let f_high = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_D), 8);

        // Verify the 3rd is HIGH
        let high_finding = f_high.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            high_finding.is_some(),
            "AC-003 setup: rebind 3 must emit D1 finding. RED: stub not wired."
        );
        // Only proceed to AC-003 proper if setup D1 finding was emitted
        // (if it wasn't, the earlier assert will have failed)
        if let Some(hf) = high_finding {
            assert_eq!(
                hf.confidence,
                Confidence::High,
                "AC-003 setup: 3rd rebind must be HIGH"
            );
        }

        // 4th rebind (EC-006): MAC_D → MAC_E at ts=10 — spoof_high_emitted=true → MEDIUM
        let f4 = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_E), 10);
        let d1_4 = f4.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1_4.is_some(),
            "AC-003 / BC-2.16.004 PC4: 4th rebind must emit D1 finding. RED: stub not wired."
        );
        assert_eq!(
            d1_4.unwrap().confidence,
            Confidence::Medium,
            "AC-003 / BC-2.16.004 PC4 (one-shot HIGH guard): 4th rebind must emit MEDIUM, \
             not HIGH, because spoof_high_emitted=true. Got {:?}",
            d1_4.unwrap().confidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 — BC-2.16.004 PC5: flap window reset after 60s
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.16.004 PC5): after ARP_FLAP_WINDOW_SECS=60 seconds have elapsed
    /// since first_rebind_ts, the window resets: rebind_count→0, first_rebind_ts→None,
    /// spoof_high_emitted→false. The next rebind is treated as the first (MEDIUM).
    ///
    /// Red Gate: fails because D1 emission is not wired.
    /// EC-007: rebind after 61s → window reset → MEDIUM (rebind_count=1).
    #[test]
    #[allow(non_snake_case)]
    fn test_d1_flap_window_reset() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        seed_binding(&mut analyzer, 0);

        // Drive to HIGH (3 rebinds within 60s starting at ts=2)
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_B), 2); // rebind_count→1, first_rebind_ts=2
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_C), 5); // rebind_count→2
        let f3 = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_D), 8); // rebind_count→3 → HIGH

        let high_d1 = f3.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        // RED: will fail if stub not wired
        assert!(
            high_d1.is_some(),
            "AC-004 setup: 3rd rebind must emit D1. RED: stub not wired."
        );

        // Now process a rebind at ts=2+61=63 → elapsed = 63 - 2 = 61 > 60s → window RESET
        // After reset: rebind_count=0, first_rebind_ts=None, spoof_high_emitted=false
        // Then Step 1: rebind_count→1, Step 2: first_rebind_ts=63 (elapsed=0), Step 3: MEDIUM
        let f_after_reset = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_E), 63);

        let d1_after_reset = f_after_reset.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1_after_reset.is_some(),
            "AC-004 / BC-2.16.004 PC5: rebind after window reset (ts=63, elapsed=61s > 60s) \
             must emit D1 finding. RED: stub not wired."
        );
        assert_eq!(
            d1_after_reset.unwrap().confidence,
            Confidence::Medium,
            "AC-004 / BC-2.16.004 PC5: first rebind after window reset must emit MEDIUM \
             (rebind_count resets to 1; spoof_high_emitted resets to false). Got {:?}",
            d1_after_reset.unwrap().confidence
        );

        // After reset, binding state should reflect fresh window
        let bindings = analyzer.bindings_snapshot();
        let entry = bindings.get(&IP_A).expect("binding must still exist");
        assert_eq!(
            entry.rebind_count, 1,
            "AC-004 / BC-2.16.004 PC5: after window reset, rebind_count must be 1 \
             (fresh start). Got {}",
            entry.rebind_count
        );
        assert!(
            !entry.spoof_high_emitted,
            "AC-004 / BC-2.16.004 PC5: after window reset, spoof_high_emitted must be false. \
             Got {}",
            entry.spoof_high_emitted
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 — BC-2.16.004 EC-008: threshold=1 → HIGH on first rebind
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.16.004 EC-008): with spoof_threshold=1, the first rebind emits HIGH.
    /// Step 1: rebind_count→1; Step 2: first_rebind_ts=ts (elapsed=0); Step 3:
    ///   1 >= 1 AND 0 <= 60 AND !spoof_high_emitted → HIGH.
    ///
    /// Red Gate: fails because D1 emission is not wired.
    #[test]
    #[allow(non_snake_case)]
    fn test_d1_threshold_1_high_on_first_rebind() {
        // Use threshold=1 (EC-008: any rebind → HIGH immediately)
        let mut analyzer = ArpAnalyzer::new(1, ARP_STORM_RATE_DEFAULT);
        seed_binding(&mut analyzer, 0);

        // First rebind at ts=100: threshold=1 → rebind_count=1 >= 1, elapsed=0 → HIGH
        let frame = make_reply(IP_A, MAC_B);
        let findings = analyzer.process_arp_for_test(&frame, 100);

        let d1 = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1.is_some(),
            "AC-005 / BC-2.16.004 EC-008: threshold=1 first rebind must emit D1 finding. \
             RED: stub not wired."
        );
        assert_eq!(
            d1.unwrap().confidence,
            Confidence::High,
            "AC-005 / BC-2.16.004 EC-008: with threshold=1, first rebind must immediately \
             escalate to HIGH (rebind_count=1 >= threshold=1, elapsed=0 <= 60s, \
             spoof_high_emitted=false). Got {:?}",
            d1.unwrap().confidence
        );

        // mitre check
        let mut techs = d1.unwrap().mitre_techniques.clone();
        techs.sort();
        assert_eq!(
            techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-005: HIGH D1 finding must carry T0830+T1557.002"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 — BC-2.16.014 PC1: GARP-that-conflicts upgrades GARP to MEDIUM
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.16.014 PC1): when is_gratuitous_arp=true AND binding conflict exists,
    /// the GARP finding is upgraded from LOW to MEDIUM and carries T0830+T1557.002.
    ///
    /// Red Gate: fails because apply_garp_conflict_escalation is an uncalled stub.
    #[test]
    #[allow(non_snake_case)]
    fn test_garp_conflicts_garp_finding_upgrades_to_medium() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        // Seed binding: 10.0.0.1 → MAC_A
        seed_binding(&mut analyzer, 0);

        // GARP frame with MAC_B (conflicts with existing MAC_A binding)
        // BC-2.16.014 EC-001: GARP Reply, sender_ip=target_ip=10.0.0.1, sender_mac=MAC_B
        let garp_frame = make_garp(IP_A, MAC_B);
        let findings = analyzer.process_arp_for_test(&garp_frame, 10);

        // Find the GARP finding
        let garp_finding = findings.iter().find(|f| {
            f.summary.to_lowercase().contains("garp")
                || f.summary.to_lowercase().contains("gratuitous")
        });

        assert!(
            garp_finding.is_some(),
            "AC-007 / BC-2.16.014 PC1: process_arp must emit a GARP finding when \
             sender_ip==target_ip. Got {} finding(s).",
            findings.len()
        );

        let gf = garp_finding.unwrap();

        // PRIMARY assertion: GARP finding MUST be MEDIUM (upgraded from LOW due to conflict)
        assert_eq!(
            gf.confidence,
            Confidence::Medium,
            "AC-007 / BC-2.16.014 PC1: GARP finding must be upgraded from LOW to MEDIUM \
             when binding conflict exists (MAC_A in table, frame has MAC_B). Got {:?}. \
             RED: apply_garp_conflict_escalation stub not wired.",
            gf.confidence
        );

        // GARP finding mitre: ["T0830", "T1557.002"] (BC-2.16.014 PC1)
        let mut techs = gf.mitre_techniques.clone();
        techs.sort();
        assert_eq!(
            techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-007 / BC-2.16.014 PC1: GARP-conflict finding must carry T0830+T1557.002. \
             Got: {:?}",
            gf.mitre_techniques
        );

        // category: Anomaly
        assert!(
            matches!(gf.category, ThreatCategory::Anomaly),
            "AC-007 / BC-2.16.014 PC1: GARP-conflict finding must have category=Anomaly. \
             Got {:?}",
            gf.category
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 — BC-2.16.014 PC2/5: GARP-that-conflicts also emits D1 (2 findings total)
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.16.014 PC2/PC5): for a GARP-that-conflicts frame, process_arp
    /// returns exactly 2 findings: one GARP MEDIUM and one D1 (MEDIUM for first rebind).
    ///
    /// Red Gate: fails because apply_garp_conflict_escalation is an uncalled stub
    /// (currently only 1 finding — the original LOW GARP — is returned).
    #[test]
    #[allow(non_snake_case)]
    fn test_garp_conflicts_d1_also_emitted() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        seed_binding(&mut analyzer, 0);

        // First GARP-that-conflicts: binding=MAC_A, frame=MAC_B → 2 findings
        let garp_frame = make_garp(IP_A, MAC_B);
        let findings = analyzer.process_arp_for_test(&garp_frame, 10);

        // BC-2.16.014 PC5 / Invariant 2: exactly 2 findings for GARP-that-conflicts
        assert_eq!(
            findings.len(),
            2,
            "AC-008 / BC-2.16.014 PC5: GARP-that-conflicts must produce exactly 2 findings \
             (GARP MEDIUM + D1 MEDIUM). Got {} finding(s): {:?}. \
             RED: apply_garp_conflict_escalation stub not wired.",
            findings.len(),
            findings
                .iter()
                .map(|f| (&f.confidence, &f.summary))
                .collect::<Vec<_>>()
        );

        // One finding must be the GARP finding at MEDIUM
        let garp_f = findings.iter().find(|f| {
            f.summary.to_lowercase().contains("garp")
                || f.summary.to_lowercase().contains("gratuitous")
        });
        assert!(
            garp_f.is_some(),
            "AC-008 / BC-2.16.014: one of the 2 findings must be a GARP finding"
        );
        assert_eq!(
            garp_f.unwrap().confidence,
            Confidence::Medium,
            "AC-008 / BC-2.16.014 PC1: GARP finding in pair must be MEDIUM. Got {:?}",
            garp_f.unwrap().confidence
        );

        // The other finding must be D1 at MEDIUM (first rebind, count=1 < threshold=3)
        let d1_f = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                && !(f.summary.to_lowercase().contains("garp")
                    || f.summary.to_lowercase().contains("gratuitous"))
        });
        // Fallback: find by spoof/rebind keyword
        let d1_f = d1_f.or_else(|| {
            findings.iter().find(|f| {
                f.summary.to_lowercase().contains("spoof")
                    || f.summary.to_lowercase().contains("rebind")
                    || f.summary.to_lowercase().contains("d1")
            })
        });
        assert!(
            d1_f.is_some(),
            "AC-008 / BC-2.16.014 PC2: one of the 2 findings must be a D1 spoof finding"
        );
        assert_eq!(
            d1_f.unwrap().confidence,
            Confidence::Medium,
            "AC-008 / BC-2.16.014 PC2: D1 finding in pair must be MEDIUM (first rebind, \
             rebind_count=1 < threshold=3). Got {:?}",
            d1_f.unwrap().confidence
        );

        // Both findings carry T0830+T1557.002 (BC-2.16.014 PC1/PC2 + Invariant 4)
        for f in &findings {
            let mut techs = f.mitre_techniques.clone();
            techs.sort();
            assert_eq!(
                techs,
                vec!["T0830".to_string(), "T1557.002".to_string()],
                "AC-008 / BC-2.16.014 Invariant 4: both findings must carry T0830+T1557.002. \
                 Finding {:?} has mitre: {:?}",
                f.summary,
                f.mitre_techniques
            );
        }
    }

    // -----------------------------------------------------------------------
    // AC-009 — BC-2.16.014 EC-004: GARP-that-conflicts at HIGH threshold
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.16.014 EC-004): when the GARP-that-conflicts is the 3rd rebind
    /// within 60s, D1 is HIGH; GARP remains MEDIUM. 2 findings total.
    ///
    /// Red Gate: fails because stub not wired.
    /// Canonical test vector (BC-2.16.014 table row 3):
    ///   binding {10.0.0.1 → BB:BB, rebind=2, first_rebind_ts=5} + GARP at ts=30 → GARP MEDIUM + D1 HIGH
    #[test]
    #[allow(non_snake_case)]
    fn test_garp_conflicts_d1_high_at_threshold() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        seed_binding(&mut analyzer, 0); // 10.0.0.1 → MAC_A

        // Drive rebind_count to 2 via normal (non-GARP) frames within 60s
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_B), 5); // rebind=1, first_rebind_ts=5
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_C), 10); // rebind=2

        // 3rd rebind is a GARP-that-conflicts (ts=30, within 60s of first_rebind_ts=5)
        // BC-2.16.014 EC-004: rebind_count→3 >= threshold=3 → D1 HIGH; GARP stays MEDIUM
        let garp_frame = make_garp(IP_A, MAC_D);
        let findings = analyzer.process_arp_for_test(&garp_frame, 30);

        // Exactly 2 findings
        assert_eq!(
            findings.len(),
            2,
            "AC-009 / BC-2.16.014 EC-004: GARP-that-conflicts at 3rd rebind must produce \
             exactly 2 findings. Got {} finding(s). RED: stub not wired.",
            findings.len()
        );

        // GARP finding: MEDIUM
        let garp_f = findings.iter().find(|f| {
            f.summary.to_lowercase().contains("garp")
                || f.summary.to_lowercase().contains("gratuitous")
        });
        assert!(garp_f.is_some(), "AC-009: must have a GARP finding");
        assert_eq!(
            garp_f.unwrap().confidence,
            Confidence::Medium,
            "AC-009 / BC-2.16.014 EC-004: GARP finding must remain MEDIUM (regardless of \
             D1 escalation). Got {:?}",
            garp_f.unwrap().confidence
        );

        // D1 finding: HIGH
        let d1_f = findings.iter().find(|f| {
            !(f.summary.to_lowercase().contains("garp")
                || f.summary.to_lowercase().contains("gratuitous"))
                && (f.mitre_techniques.contains(&"T0830".to_string())
                    || f.summary.to_lowercase().contains("spoof")
                    || f.summary.to_lowercase().contains("d1"))
        });
        assert!(d1_f.is_some(), "AC-009: must have a D1 finding");
        assert_eq!(
            d1_f.unwrap().confidence,
            Confidence::High,
            "AC-009 / BC-2.16.014 EC-004: D1 finding at 3rd rebind must be HIGH. Got {:?}",
            d1_f.unwrap().confidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-010 — BC-2.16.014 PC6: GARP without conflict → LOW only, no D1
    // (regression from STORY-113 behavior)
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.16.014 PC6): a GARP frame where sender_ip is NOT in the binding
    /// table (no conflict) produces exactly one GARP finding at LOW. No D1 finding.
    /// mitre_techniques: [] (D-068 adjudication for benign GARP).
    ///
    /// This is a regression test: STORY-113 behavior must be preserved.
    /// This test is expected to PASS now (benign GARP already emits LOW).
    /// We include it here as a RED anchor for AC-010 to verify it does NOT
    /// accidentally acquire mitre or a second finding after STORY-114 impl.
    #[test]
    #[allow(non_snake_case)]
    fn test_garp_no_conflict_low_only() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);

        // GARP for IP_A with MAC_A — no prior binding for IP_A
        // EC-009 / BC-2.16.014 EC-003: no conflict → LOW only
        let garp_frame = make_garp(IP_A, MAC_A);
        let findings = analyzer.process_arp_for_test(&garp_frame, 10);

        // Exactly 1 finding (LOW GARP)
        assert_eq!(
            findings.len(),
            1,
            "AC-010 / BC-2.16.014 PC6: GARP without binding conflict must produce exactly \
             1 finding (LOW GARP). Got {} finding(s): {:?}",
            findings.len(),
            findings
                .iter()
                .map(|f| (&f.confidence, &f.summary))
                .collect::<Vec<_>>()
        );

        let garp_f = &findings[0];

        // Must be LOW
        assert_eq!(
            garp_f.confidence,
            Confidence::Low,
            "AC-010 / BC-2.16.014 PC6: benign GARP (no conflict) must be LOW. Got {:?}",
            garp_f.confidence
        );

        // mitre_techniques: [] (D-068: no AiTM attribution for benign GARP)
        assert!(
            garp_f.mitre_techniques.is_empty(),
            "AC-010 / BC-2.16.014 PC6: benign GARP mitre_techniques must be [] (empty). \
             Got: {:?}",
            garp_f.mitre_techniques
        );

        // No D1 finding
        let d1 = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
        });
        assert!(
            d1.is_none(),
            "AC-010 / BC-2.16.014 PC6: no D1 finding must be emitted for benign GARP \
             (no binding conflict). Found: {:?}",
            d1.map(|f| (&f.confidence, &f.summary))
        );
    }

    // -----------------------------------------------------------------------
    // AC-016 — BC-2.16.004 PC1.e: D1 evidence contains IP + old MAC + new MAC
    // -----------------------------------------------------------------------

    /// AC-016 (BC-2.16.004 PC1.e): D1 finding evidence must include the conflicting IP,
    /// the old MAC (from binding before update), and the new MAC (from frame.sender_mac).
    /// Applies regardless of MEDIUM or HIGH severity.
    ///
    /// Red Gate: fails because D1 emission is not wired.
    #[test]
    #[allow(non_snake_case)]
    fn test_d1_finding_evidence_contains_ips_and_macs() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        // Seed: 10.0.0.1 → AA:AA:AA:AA:AA:AA
        seed_binding(&mut analyzer, 0);

        // Rebind: old=MAC_A (AA:AA), new=MAC_B (BB:BB), sender_ip=10.0.0.1
        let frame = make_reply(IP_A, MAC_B);
        let findings = analyzer.process_arp_for_test(&frame, 2);

        let d1 = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
                || f.summary.to_lowercase().contains("rebind")
        });
        assert!(
            d1.is_some(),
            "AC-016 / BC-2.16.004 PC1.e: D1 finding must be emitted. RED: stub not wired."
        );
        let d1 = d1.unwrap();
        let evidence_joined = d1.evidence.join(" ");

        // Evidence must contain the sender_ip: "10.0.0.1"
        assert!(
            evidence_joined.contains("10.0.0.1"),
            "AC-016 / BC-2.16.004 PC1.e: D1 evidence must contain the conflicting IP \
             '10.0.0.1'. Evidence: {:?}",
            d1.evidence
        );

        // Evidence must contain the OLD MAC: AA:AA:AA:AA:AA:AA
        // (case-insensitive: the impl may use upper or lower hex)
        let evidence_upper = evidence_joined.to_uppercase();
        assert!(
            evidence_upper.contains("AA:AA:AA:AA:AA:AA"),
            "AC-016 / BC-2.16.004 PC1.e: D1 evidence must contain the OLD MAC \
             (AA:AA:AA:AA:AA:AA — binding before update). Evidence: {:?}",
            d1.evidence
        );

        // Evidence must contain the NEW MAC: BB:BB:BB:BB:BB:BB
        assert!(
            evidence_upper.contains("BB:BB:BB:BB:BB:BB"),
            "AC-016 / BC-2.16.004 PC1.e: D1 evidence must contain the NEW MAC \
             (BB:BB:BB:BB:BB:BB — frame.sender_mac). Evidence: {:?}",
            d1.evidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-017 — BC-2.16.007 PC1 D12 MITRE back-fill
    // -----------------------------------------------------------------------

    /// AC-017 (BC-2.16.007 PC1): after the VP-007 5-part atomic update, D12 mismatch
    /// findings carry mitre_techniques: ["T0830", "T1557.002"] AND retain
    /// confidence=MEDIUM, category=Anomaly, and evidence fields eth_mac + arp_sender_mac + sender_ip.
    ///
    /// This test UPDATES the STORY-113 assertion (which expected mitre_techniques=[])
    /// to the STORY-114 final state (mitre_techniques=["T0830","T1557.002"]).
    ///
    /// Red Gate: fails now because D12 still emits mitre_techniques=[] until the
    /// VP-007 5-part atomic update adds T0830+T1557.002 to src/mitre.rs and wires
    /// the mitre vec in the D12 emission branch.
    #[test]
    #[allow(non_snake_case)]
    fn test_d12_mismatch_carries_mitre_after_catalog() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        let eth_mac: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
        let arp_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let sender_ip: [u8; 4] = [192, 168, 1, 1];

        // D12 mismatch frame: eth_mac != arp_mac
        let frame = ArpFrame {
            operation: 2,
            sender_mac: arp_mac,
            sender_ip,
            target_mac: [0u8; 6],
            target_ip: [192, 168, 1, 2],
            outer_src_mac: Some(eth_mac),
            packet_len: 42,
        };

        let findings = analyzer.process_arp_for_test(&frame, 1_000);

        let d12 = findings
            .iter()
            .find(|f| {
                matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
            })
            .expect(
                "AC-017 / BC-2.16.007 PC1: D12 mismatch must emit MEDIUM/Anomaly finding. \
                 Got 0 matching findings.",
            );

        // PRIMARY assertion: mitre_techniques must be ["T0830", "T1557.002"] after STORY-114
        // This FAILS now because D12 still emits mitre_techniques=[] at wave 42.
        let mut techs = d12.mitre_techniques.clone();
        techs.sort();
        assert_eq!(
            techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-017 / BC-2.16.007 PC1: D12 finding must carry mitre_techniques \
             [\"T0830\", \"T1557.002\"] after STORY-114 VP-007 atomic update. \
             Currently [] (wave 42). RED: requires both catalog seeding AND \
             mitre vec wiring in D12 emission branch. Got: {:?}",
            d12.mitre_techniques
        );

        // Retained invariants: confidence=MEDIUM, category=Anomaly
        assert_eq!(d12.confidence, Confidence::Medium, "D12 must retain MEDIUM");
        assert!(
            matches!(d12.category, ThreatCategory::Anomaly),
            "D12 must retain Anomaly"
        );

        // Evidence: eth_mac, arp_sender_mac, sender_ip (BC-2.16.007 PC1)
        let ev = d12.evidence.join(" ");
        assert!(
            ev.contains("eth_src_mac=11:22:33:44:55:66"),
            "AC-017: D12 evidence must contain eth_src_mac. Got: {:?}",
            d12.evidence
        );
        assert!(
            ev.contains("arp_sender_mac=AA:BB:CC:DD:EE:FF"),
            "AC-017: D12 evidence must contain arp_sender_mac. Got: {:?}",
            d12.evidence
        );
        assert!(
            ev.contains("sender_ip=192.168.1.1"),
            "AC-017: D12 evidence must contain sender_ip. Got: {:?}",
            d12.evidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-014 — BC-2.10.002 v1.5: enum-variant distinctness (VERIFY test)
    // -----------------------------------------------------------------------

    /// AC-014 (BC-2.10.002 v1.5 / D-069): MitreTactic::Impact != MitreTactic::IcsImpact
    /// as enum values (distinct variants). Display strings also differ:
    /// "Impact" vs "Impact (ICS)".
    ///
    /// This test may already PASS (enum distinctness is correct as-is under D-069).
    /// It is included here as a verify test to prevent future regression.
    #[test]
    #[allow(non_snake_case)]
    fn test_impact_vs_ics_impact_variants_distinct() {
        use crate::mitre::MitreTactic;

        // Enum variants must be distinct (D-069: IcsImpact is a different variant)
        assert_ne!(
            MitreTactic::Impact,
            MitreTactic::IcsImpact,
            "AC-014 / BC-2.10.002 v1.5: MitreTactic::Impact and MitreTactic::IcsImpact \
             must be distinct enum variants (D-069)."
        );

        // Display strings must be distinct (D-069: IcsImpact = "Impact (ICS)", not "Impact")
        let impact_str = format!("{}", MitreTactic::Impact);
        let ics_impact_str = format!("{}", MitreTactic::IcsImpact);

        assert_eq!(
            impact_str, "Impact",
            "MitreTactic::Impact Display must be \"Impact\". Got: {impact_str:?}",
        );
        assert_eq!(
            ics_impact_str, "Impact (ICS)",
            "MitreTactic::IcsImpact Display must be \"Impact (ICS)\" (D-069 canonical; \
             src/mitre.rs:91 MUST NOT be changed). Got: {ics_impact_str:?}",
        );
        assert_ne!(
            impact_str, ics_impact_str,
            "AC-014 / BC-2.10.002 v1.5: Display strings of Impact and IcsImpact must differ \
             (\"Impact\" vs \"Impact (ICS)\"). D-069 preservation check."
        );
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
