//! ARP security analyzer — STORY-113 + STORY-114 + STORY-115 full implementation (GREEN).
//!
//! This module defines [`ArpAnalyzer`], a stateful ARP-frame processor that
//! maintains a bounded binding table (IP→MAC with LRU eviction), detects
//! Gratuitous ARP (D2), D11 malformed ARP, D12 L2/L3 sender-MAC mismatch,
//! D3 ARP storm rate detection, and exposes a `summarize()` method returning
//! eleven canonical summary keys.
//!
//! STORY-113, STORY-114, and STORY-115 are all fully implemented and all tests pass (GREEN).
//! `process_arp` emits D1 spoof findings (BC-2.16.004): MEDIUM on rebind, escalating
//! to HIGH after `spoof_threshold` rebinds within `ARP_FLAP_WINDOW_SECS`. GARP-conflict
//! frames upgrade the GARP finding to MEDIUM and co-emit a D1 finding (BC-2.16.014).
//! D12 L2/L3 mismatch findings carry `mitre_techniques: ["T0830", "T1557.002"]`
//! (BC-2.16.007 PC1). `src/mitre.rs` was updated atomically (VP-007): SEEDED=25, EMITTED=17.
//! D3 storm detection (BC-2.16.008) emits MEDIUM/Anomaly findings with `mitre_techniques: []`
//! (T0814 withheld per DF-VALIDATION-001) when source MAC rate exceeds `storm_rate` threshold.
//!
//! The VP-024 Sub-B/Sub-D Kani harness bodies (`verify_classify_garp_total`,
//! `verify_binding_table_cap`) were filled and formally proven at the F6
//! formal-hardening gate (VP-024 v2.0, verification_lock: true).
//!
//! ## STORY-114 deliverables (implemented)
//! - `ArpAnalyzer::new(spoof_threshold, storm_rate)` — signature extended.
//! - `SPOOF_REBIND_ESCALATION_DEFAULT`, `ARP_FLAP_WINDOW_SECS`, `ARP_STORM_RATE_DEFAULT`
//!   constants.
//! - `--arp-spoof-threshold` CLI flag wired (BC-2.16.012).
//! - `emit_d1_spoof_finding` and `apply_garp_conflict_escalation` fully wired into
//!   `process_arp` (BC-2.16.004 / BC-2.16.014).
//! - VP-007 5-part atomic catalog update: T0830 + T1557.002 seeded and emitted.
//!
//! ## STORY-115 deliverables (implemented)
//! - `detect_storm` — BC-2.16.008 3-step intra-frame sequence wired into `process_arp`.
//! - `insert_storm_counter_lru` — LRU eviction for `storm_counters` (MAX_STORM_COUNTERS=4096).
//! - `--arp-storm-rate` CLI flag wired (BC-2.16.013).
//! - `storm_findings` summary key reflects actual D3 detections (BC-2.16.010 extension).
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

/// Default ARP storm rate threshold (frames/second per source MAC) for D3 detection.
///
/// Overridable via `--arp-storm-rate` (BC-2.16.013; STORY-115).
/// Used as the default value of `ArpAnalyzer.storm_rate` when no CLI override is given.
/// Not an industry standard — wirerust engineering default (BC-2.16.008 Description).
pub const ARP_STORM_RATE_DEFAULT: u32 = 50;

/// Maximum number of per-sender-MAC storm-counter entries retained at any time.
///
/// BC-2.16.008 postcondition 5: `storm_counters.len()` NEVER exceeds this value.
/// When a new source MAC would cause overflow, the entry with the minimum
/// `window_start_ts` is evicted before inserting (LRU heuristic, analogous to
/// `insert_binding_lru` / BC-2.16.006).
pub const MAX_STORM_COUNTERS: usize = 4_096;

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

/// Per-sender-MAC storm-rate tracking entry (D3 — BC-2.16.008).
///
/// Tracks frame count, window start timestamp, and one-shot emission guard
/// per source MAC for ARP storm detection. Managed by `detect_storm` and
/// `insert_storm_counter_lru` in `process_arp`.
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

/// Fixed-capacity array surrogate of `insert_binding_lru` for the VP-024 Sub-D
/// Kani cap-invariant harness.
///
/// **Why an array and not the `BTreeMap` surrogate (`insert_binding_lru_btree`)
/// in the Kani harness:** CBMC (Kani's backend) cannot symbolically execute
/// `std::collections::BTreeMap` within a practical resource budget. Empirically
/// (Kani 0.67 / CBMC, this environment), even *three* plain `BTreeMap::insert`
/// calls with no eviction exhaust CBMC's memory during SSA conversion
/// (`VERIFICATION:- FAILED`, "CBMC appears to have run out of memory"); the full
/// `insert_binding_lru_btree` cap+1 sequence was unresolved after 45+ minutes.
/// The std B-tree's raw-pointer node machinery and rebalancing loops are the
/// bottleneck — incidental to the property under proof, which is the pure
/// arithmetic cap invariant `len <= cap`. VP-024 explicitly states the cap
/// invariant "is a purely arithmetic property independent of which
/// ordered/unordered map is used; the proof is valid for the production
/// `HashMap` by substitution." This surrogate substitutes a CBMC-tractable
/// fixed-capacity array for the same arithmetic invariant.
///
/// **Algorithmic fidelity:** this function reproduces the exact three-branch
/// decision logic of `insert_binding_lru` / `insert_binding_lru_btree`:
///   1. key already present  → update MAC in place, length unchanged;
///   2. else, length >= cap  → evict the entry with the minimum `last_seen_ts`
///      (one removal), then insert (net length unchanged);
///   3. else                 → insert (length + 1).
///
/// `len` therefore never exceeds `cap`. Entries are stored as
/// `(ip, mac, last_seen_ts)` triples in a `[_; N]` backing array with an explicit
/// `len`; new entries initialize `last_seen_ts = 0`, matching `BindingEntry`.
///
/// `N` is the array capacity (must be >= `cap`). Gated `#[cfg(any(kani, test))]`
/// — NOT present in the production binary. ADR-008 Decision 5; VP-024 Sub-D.
#[cfg(any(kani, test))]
pub fn insert_binding_lru_array<const N: usize>(
    entries: &mut [([u8; 4], [u8; 6], u32); N],
    len: &mut usize,
    ip: [u8; 4],
    mac: [u8; 6],
    cap: usize,
) {
    // Branch 1: key already present → update MAC in place (length unchanged).
    let mut j = 0usize;
    while j < *len {
        if entries[j].0 == ip {
            entries[j].1 = mac;
            return;
        }
        j += 1;
    }
    // Branch 2: new key at capacity → evict the min-last_seen_ts entry.
    if *len >= cap {
        // Find the index of the minimum last_seen_ts among the first `len` entries.
        let mut min_idx = 0usize;
        let mut k = 1usize;
        while k < *len {
            if entries[k].2 < entries[min_idx].2 {
                min_idx = k;
            }
            k += 1;
        }
        // Remove by swapping the last live entry into the evicted slot.
        entries[min_idx] = entries[*len - 1];
        *len -= 1;
    }
    // Branch 3 (and the post-eviction insert): append the new entry.
    entries[*len] = (ip, mac, 0);
    *len += 1;
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
/// escalation are fully implemented.
///
/// STORY-115 implements D3 storm detection (BC-2.16.008): `detect_storm` is
/// wired into `process_arp`; `--arp-storm-rate` is wired (BC-2.16.013).
pub struct ArpAnalyzer {
    /// IP→MAC binding table. Cap enforced at MAX_ARP_BINDINGS via LRU eviction.
    /// Production substrate per Architecture Compliance Rule 2.
    pub bindings: HashMap<[u8; 4], BindingEntry>,
    /// Per-sender-MAC storm tracking. Cap enforced at MAX_STORM_COUNTERS via LRU eviction.
    /// Managed by `detect_storm` / `insert_storm_counter_lru` (D3 — BC-2.16.008).
    pub storm_counters: HashMap<[u8; 6], StormCounter>,

    // --- STORY-114 threshold fields (BC-2.16.012 / BC-2.16.013) ---
    /// D1 spoof escalation threshold: number of rebinds within ARP_FLAP_WINDOW_SECS
    /// before a HIGH finding is emitted. Overridable via `--arp-spoof-threshold`.
    /// BC-2.16.012; default = SPOOF_REBIND_ESCALATION_DEFAULT = 3.
    pub spoof_threshold: u32,
    /// ARP storm rate threshold (frames/second per source MAC) for D3 detection.
    /// Used by `detect_storm`. Overridable via `--arp-storm-rate` (BC-2.16.013).
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
    /// D3 storm findings emitted. Incremented by `detect_storm` via `process_arp` (STORY-115).
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
    ///   (BC-2.16.013). Overridable via `--arp-storm-rate` (STORY-115).
    ///   Pass `ARP_STORM_RATE_DEFAULT` (= 50) when no CLI override.
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
    /// (d) D3 storm detection — BC-2.16.008. Runs for every frame that passes the
    ///     zero/broadcast sender_ip filter above, keyed on sender_mac. Executed BEFORE
    ///     the GARP/D1/D12 branching so that GARP floods are covered uniformly with
    ///     all other frame types. Exactly one call per frame.
    /// (e) check D2 GARP (is_gratuitous_arp — BC-2.16.003);
    ///     - if GARP AND binding conflict (BC-2.16.014): escalate GARP LOW→MEDIUM,
    ///       attach T0830/T1557.002, co-emit D1 finding (Steps 1–3 via emit_d1_spoof_finding);
    ///       Step 4 (MAC update) occurs after both findings are produced.
    ///     - if GARP with no binding conflict: LOW finding, no D1, no MITRE.
    /// (f)/(g) For non-GARP frames: update binding table; if rebind (MAC change),
    ///     emit D1 finding via emit_d1_spoof_finding (Steps 1–3);
    ///     Step 4 (MAC update) occurs last — Architecture Compliance Rule 1.
    /// (h) return Vec of findings (D2/D12/D1/D3).
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

        // (d) D3 storm detection — BC-2.16.008.
        // Runs for every frame that passes the zero/broadcast sender_ip filter above,
        // keyed on sender_mac. Executed before the GARP/D1 branching so GARP floods
        // are detected uniformly with all other ARP frame types (BC-2.16.008 PC1).
        if let Some(storm_finding) = self.detect_storm(sender_mac, timestamp_secs) {
            self.storm_findings += 1;
            findings.push(storm_finding);
        }

        // (e) D2 GARP check — BC-2.16.003 / BC-2.16.014.
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
                    summary: "D2: Gratuitous ARP (GARP) with binding conflict — \
                              sender_ip equals target_ip and conflicts with an existing \
                              IP\u{2192}MAC binding"
                        .to_string(),
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
    /// `storm_findings` reflects the number of D3 storm detections since the last `new()`
    /// (incremented by `detect_storm` in STORY-115; 0 until D3 fires).
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
            verdict: if confidence == Confidence::High {
                Verdict::Likely
            } else {
                Verdict::Possible
            },
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

    // -----------------------------------------------------------------------
    // STORY-115 D3 storm detection helpers (GREEN — wired into process_arp).
    // -----------------------------------------------------------------------

    /// Evict the oldest entry from `storm_counters` when the cap is reached,
    /// then insert a new `StormCounter` for `source_mac`.
    ///
    /// Analogous to `insert_binding_lru` for the binding table. Eviction key
    /// is the minimum `window_start_ts` (LRU heuristic per BC-2.16.008 PC5).
    ///
    /// One-in-one-out: `storm_counters.len()` never exceeds `MAX_STORM_COUNTERS`.
    /// BC-2.16.008 PC5; BC-2.16.008 Invariant 6.
    fn insert_storm_counter_lru(&mut self, source_mac: [u8; 6], timestamp_secs: u32) {
        if !self.storm_counters.contains_key(&source_mac)
            && self.storm_counters.len() >= MAX_STORM_COUNTERS
        {
            // Evict the entry with the minimum window_start_ts (LRU heuristic).
            let oldest_mac = self
                .storm_counters
                .iter()
                .min_by_key(|(_, e)| e.window_start_ts)
                .map(|(k, _)| *k);
            if let Some(k) = oldest_mac {
                self.storm_counters.remove(&k);
            }
        }
        self.storm_counters.insert(
            source_mac,
            StormCounter {
                count_in_window: 1,
                window_start_ts: timestamp_secs,
                storm_emitted: false,
            },
        );
    }

    /// D3 ARP storm detection for a single frame from `source_mac`.
    ///
    /// Implements the exact three-step intra-frame sequence from BC-2.16.008 PC1–PC4:
    ///
    /// Step 1 — window-expiry check and initialization:
    ///   - If source MAC not in storm_counters OR elapsed > ARP_FLAP_WINDOW_SECS:
    ///     (re)initialize: count_in_window=1, window_start_ts=ts, storm_emitted=false.
    ///     Proceed to Step 3 (no Step 2 increment).
    ///   - Otherwise (existing entry, window still active): proceed to Step 2.
    ///
    /// Step 2 — increment: count_in_window += 1 (window active path only).
    ///
    /// Step 3 — rate evaluation:
    ///   rate = count_in_window / max(1, ts - window_start_ts).
    ///   If rate >= self.storm_rate AND !storm_emitted: emit MEDIUM/Anomaly Finding
    ///   with mitre_techniques: [] (T0814 withheld per DF-VALIDATION-001).
    ///   Set storm_emitted = true (one-shot guard per BC-2.16.008 Invariant 1).
    ///
    /// Returns `Some(Finding)` when the storm threshold is met, `None` otherwise.
    ///
    /// BC-2.16.008 PC1–PC4, PC3 Note 6 (max(1,...) denominator, ARP-AMB-003 RESOLVED).
    fn detect_storm(&mut self, source_mac: [u8; 6], timestamp_secs: u32) -> Option<Finding> {
        use crate::findings::{Confidence, ThreatCategory, Verdict};

        // Step 1 — window-expiry check and initialization.
        let in_window = if let Some(entry) = self.storm_counters.get(&source_mac) {
            let elapsed = timestamp_secs.saturating_sub(entry.window_start_ts);
            elapsed <= ARP_FLAP_WINDOW_SECS
        } else {
            false
        };

        if !in_window {
            // First-ever observation OR window expired: (re)initialize via LRU insert.
            self.insert_storm_counter_lru(source_mac, timestamp_secs);
            // count_in_window = 1 set in insert_storm_counter_lru; proceed to Step 3.
        } else {
            // Step 2 — in-window increment.
            if let Some(entry) = self.storm_counters.get_mut(&source_mac) {
                entry.count_in_window += 1;
            }
        }

        // Step 3 — rate evaluation.
        let entry = self.storm_counters.get_mut(&source_mac)?;
        let elapsed = timestamp_secs.saturating_sub(entry.window_start_ts);
        let denominator = elapsed.max(1) as u64;
        let rate = entry.count_in_window / denominator;

        if rate >= self.storm_rate as u64 && !entry.storm_emitted {
            entry.storm_emitted = true;

            let mac = source_mac;
            let frame_count = entry.count_in_window;
            let window_secs = elapsed;
            let rate_pps = rate;

            Some(Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Possible,
                confidence: Confidence::Medium,
                summary: format!(
                    "D3: ARP storm detected — high ARP frame rate from source MAC \
                     {:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                    mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                ),
                evidence: vec![
                    format!(
                        "source_mac={:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
                        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
                    ),
                    format!("frame_count={frame_count}"),
                    format!("window_secs={window_secs}"),
                    format!("rate_pps={rate_pps}"),
                ],
                // T0814 withheld per DF-VALIDATION-001 / BC-2.16.008 Invariant 3.
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
        } else {
            None
        }
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
    //! STORY-113 test suite — unit + proptest tests (originally RED in the stub phase,
    //! now fully GREEN).
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

    /// AC-008 (BC-2.16.006 PC2), array surrogate: `insert_binding_lru_array` — the
    /// CBMC-tractable fixed-capacity surrogate exercised by the VP-024 Sub-D Kani
    /// harness `verify_binding_table_cap` — enforces the same `len <= cap`
    /// invariant as `insert_binding_lru_btree`, and agrees with it on the final
    /// length after a cap+1 distinct-IP insertion sequence. This concrete test
    /// keeps the array surrogate honest (algorithmic parity with the BTreeMap
    /// surrogate) and prevents it from being dead code under `cargo test`.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_006_binding_table_cap_enforced_array_surrogate() {
        const CAP: usize = TEST_MAX_ARP_BINDINGS; // 8
        let mut entries: [([u8; 4], [u8; 6], u32); CAP] = [([0u8; 4], [0u8; 6], 0u32); CAP];
        let mut len: usize = 0;

        for i in 0u8..=(CAP as u8) {
            let ip: [u8; 4] = [10, 0, 0, i + 1];
            let mac: [u8; 6] = [i; 6];
            insert_binding_lru_array(&mut entries, &mut len, ip, mac, CAP);
            // Stamp a distinct last_seen_ts so the eviction min-scan is deterministic.
            if len > 0 {
                entries[len - 1].2 = i as u32;
            }
            assert!(
                len <= CAP,
                "array surrogate: len must NEVER exceed cap={CAP}. After {} inserts, len={len}.",
                i + 1,
            );
        }
        // After cap+1 insertions, length must be exactly cap (one eviction occurred),
        // matching insert_binding_lru_btree's behavior in the test above.
        assert_eq!(
            len,
            CAP,
            "array surrogate: after inserting cap+1={} distinct IPs, len must equal cap={CAP}.",
            CAP + 1,
        );
    }

    /// Production `insert_binding_lru` (HashMap) eviction-boundary fidelity.
    ///
    /// The existing cap test uses the BTreeMap surrogate; `process_arp` only drives
    /// the production HashMap function at the real cap (MAX_ARP_BINDINGS = 65_536),
    /// which is infeasible to exercise at the boundary in a unit test. This test
    /// calls `insert_binding_lru` directly with a small cap so its eviction branch
    /// (`bindings.len() >= cap`) and last-write-wins update branch are observably
    /// covered — exact-state assertions on which key is evicted and which MAC wins.
    #[test]
    fn test_insert_binding_lru_hashmap_eviction_boundary() {
        const CAP: usize = 3;
        let mut bindings: HashMap<[u8; 4], BindingEntry> = HashMap::new();
        let ip = |n: u8| [10, 0, 0, n];
        let mac = |n: u8| [n; 6];

        // Fill to cap with distinct IPs; stamp ascending last_seen_ts (ip(0) oldest).
        for n in 0u8..3 {
            insert_binding_lru(&mut bindings, ip(n), mac(n), CAP);
            bindings.get_mut(&ip(n)).unwrap().last_seen_ts = n as u32;
        }
        assert_eq!(bindings.len(), CAP, "filled to cap");

        // Last-write-wins: re-insert an existing IP updates MAC, len unchanged.
        insert_binding_lru(&mut bindings, ip(1), mac(88), CAP);
        assert_eq!(bindings.len(), CAP, "update-in-place keeps len");
        assert_eq!(
            bindings.get(&ip(1)).map(|e| e.mac),
            Some(mac(88)),
            "MAC updated"
        );

        // New key at capacity evicts the minimum-last_seen_ts entry (ip(0), ts=0).
        insert_binding_lru(&mut bindings, ip(9), mac(9), CAP);
        assert_eq!(bindings.len(), CAP, "eviction keeps len at cap");
        assert!(
            !bindings.contains_key(&ip(0)),
            "ip(0) (min ts) must be evicted"
        );
        assert!(
            bindings.contains_key(&ip(9)),
            "new key present after eviction"
        );
        assert!(bindings.contains_key(&ip(1)), "non-min ip(1) survives");
        assert!(bindings.contains_key(&ip(2)), "non-min ip(2) survives");
    }

    /// Array-surrogate algorithmic fidelity: exercises all three branches of
    /// `insert_binding_lru_array` with exact-state assertions, so the lookup-loop
    /// bound, the eviction-trigger comparison, and the min-scan loop bound are all
    /// observably tested (mutation-kill coverage for the surrogate the VP-024
    /// Sub-D Kani harness depends on).
    #[test]
    fn test_insert_binding_lru_array_branch_fidelity() {
        const CAP: usize = 3;
        let mut entries: [([u8; 4], [u8; 6], u32); CAP] = [([0u8; 4], [0u8; 6], 0u32); CAP];
        let mut len = 0usize;

        let ip = |n: u8| [10, 0, 0, n];
        let mac = |n: u8| [n; 6];

        // Branch 3 (append): three distinct IPs fill the table to cap, last_seen_ts
        // ascending (entry 0 is the oldest).
        for n in 0u8..3 {
            insert_binding_lru_array(&mut entries, &mut len, ip(n), mac(n), CAP);
            entries[len - 1].2 = n as u32; // ts = 0,1,2
        }
        assert_eq!(len, 3, "three distinct inserts must fill to cap");
        // Each inserted IP is present with its MAC (kills the lookup `j < len` bound:
        // a broken lookup loop would mis-route a later re-insert).
        for n in 0u8..3 {
            let found = entries[..len].iter().find(|e| e.0 == ip(n));
            assert_eq!(
                found.map(|e| e.1),
                Some(mac(n)),
                "ip {n} must map to mac {n}"
            );
        }

        // Branch 1 (update-in-place): re-insert an EXISTING IP with a new MAC. len
        // must stay 3 and only the MAC changes. Kills the lookup-loop bound and the
        // early-return: without a correct `while j < len` scan the existing key is
        // not found and a spurious eviction+append would occur.
        insert_binding_lru_array(&mut entries, &mut len, ip(1), mac(99), CAP);
        assert_eq!(len, 3, "update-in-place must not change len");
        let updated = entries[..len].iter().find(|e| e.0 == ip(1));
        assert_eq!(updated.map(|e| e.1), Some(mac(99)), "in-place MAC update");
        // No IP was evicted: all three originals still present.
        for n in 0u8..3 {
            assert!(
                entries[..len].iter().any(|e| e.0 == ip(n)),
                "ip {n} must survive an in-place update of another key"
            );
        }

        // Branch 2 (evict min-last_seen_ts): a NEW IP at capacity evicts the entry
        // with the smallest last_seen_ts. Entry for ip(0) has ts=0 (the minimum), so
        // it must be the one evicted. Kills the eviction-trigger `>= cap` and the
        // min-scan `k < len` / `<` comparison.
        insert_binding_lru_array(&mut entries, &mut len, ip(7), mac(7), CAP);
        assert_eq!(len, 3, "eviction keeps len at cap");
        assert!(
            !entries[..len].iter().any(|e| e.0 == ip(0)),
            "min-last_seen_ts entry ip(0) must be evicted"
        );
        assert!(
            entries[..len].iter().any(|e| e.0 == ip(7)),
            "newly inserted ip(7) must be present after eviction"
        );
        // The non-minimum entries (ip(1) updated, ip(2)) must NOT be evicted.
        assert!(entries[..len].iter().any(|e| e.0 == ip(1)));
        assert!(entries[..len].iter().any(|e| e.0 == ip(2)));

        // Branch 2, minimum at a NON-ZERO index: the min-scan loop must actually
        // iterate (`while k < len`) and compare (`entries[k].2 < entries[min_idx].2`)
        // to locate it. A fresh table is built so that the smallest last_seen_ts sits
        // at index 1, NOT index 0 — this distinguishes a correct scan from one whose
        // loop bound or comparison is mutated (which would leave min_idx = 0 and
        // evict the wrong entry).
        let mut e2: [([u8; 4], [u8; 6], u32); CAP] = [([0u8; 4], [0u8; 6], 0u32); CAP];
        let mut l2 = 0usize;
        // ts layout by index: [0]=5, [1]=1 (the minimum), [2]=3.
        let ts = [5u32, 1u32, 3u32];
        for n in 0u8..3 {
            insert_binding_lru_array(&mut e2, &mut l2, ip(n), mac(n), CAP);
            e2[l2 - 1].2 = ts[n as usize];
        }
        // Insert a new key at capacity: the minimum-ts entry (index 1, ip(1)) must be
        // evicted; ip(0) and ip(2) must survive.
        insert_binding_lru_array(&mut e2, &mut l2, ip(8), mac(8), CAP);
        assert_eq!(l2, 3, "eviction keeps len at cap (min at non-zero index)");
        assert!(
            !e2[..l2].iter().any(|e| e.0 == ip(1)),
            "the minimum-last_seen_ts entry at index 1 (ip(1)) must be evicted"
        );
        assert!(
            e2[..l2].iter().any(|e| e.0 == ip(0)),
            "ip(0) (ts=5, not the min) must survive"
        );
        assert!(
            e2[..l2].iter().any(|e| e.0 == ip(2)),
            "ip(2) (ts=3, not the min) must survive"
        );
        assert!(
            e2[..l2].iter().any(|e| e.0 == ip(8)),
            "newly inserted ip(8) must be present"
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
        // STORY-113 TDD sibling-sweep update: assertion updated from mitre=[] (wave 42)
        // to mitre=["T0830","T1557.002"] (wave 43 final state, delivered by STORY-114
        // VP-007 5-part atomic update).
        let mut d12_techs = mismatch_finding.mitre_techniques.clone();
        d12_techs.sort();
        assert_eq!(
            d12_techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-009 / BC-2.16.007 PC1 (wave 43 sibling-sweep): D12 Finding mitre_techniques \
             must be [\"T0830\", \"T1557.002\"] (delivered by STORY-114 VP-007 5-part atomic update). \
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
    /// shape, enforcing BC-2.16.009 PC3 (F-113-01 fix).
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

        // record_malformed returns Vec<Finding> (F-113-01 interface fix).
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
             value ({packet_len}) — verifies that record_malformed includes packet_len in \
             evidence (F-113-01 fix). Evidence: {:?}",
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
// STORY-114 tests (BC-2.16.004, BC-2.16.014, BC-2.16.007 MITRE)
// ---------------------------------------------------------------------------

/// STORY-114 test suite (originally RED in the stub phase, now fully GREEN).
///
/// Each test verifies a behavioral contract delivered by STORY-114:
///   - `process_arp` emits D1 spoof findings with correct confidence, category, MITRE
///     techniques, and evidence (BC-2.16.004).
///   - GARP-conflict frames escalate GARP to MEDIUM and co-emit a D1 finding (BC-2.16.014).
///   - D12 mismatch findings carry mitre_techniques=["T0830","T1557.002"] (BC-2.16.007 PC1).
///   - T0830 and T1557.002 are seeded in src/mitre.rs (SEEDED=25, EMITTED=17, VP-007 atomic).
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
    use crate::findings::{Confidence, ThreatCategory, Verdict};

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
    /// Verifies D1 first rebind emits MEDIUM with T0830/T1557.002 and IP+old+new MAC evidence.
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

        // Assert a D1 spoof finding is present in the returned findings.
        assert!(
            d1.is_some(),
            "AC-001 / BC-2.16.004 PC1: process_arp must emit a D1 spoof Finding on first \
             rebind (MAC_A → MAC_B). Got {} finding(s): {:?}. \
             D1 spoof finding must be emitted via emit_d1_spoof_finding.",
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
    /// Verifies D1 escalates to HIGH at the rebind threshold; rebind=1 and rebind=2
    /// emit MEDIUM; rebind=3 emits HIGH (spoof_high_emitted → true).
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
            "AC-002 / BC-2.16.004: rebind 1 must emit D1 finding."
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
            "AC-002 / BC-2.16.004: rebind 2 must emit D1 finding."
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
            "AC-002 / BC-2.16.004: rebind 3 must emit D1 finding."
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
    /// Verifies the one-shot HIGH guard: once HIGH is emitted, further rebinds within
    /// the same flap window produce MEDIUM, never a second HIGH.
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
            "AC-003 setup: rebind 3 must emit D1 finding."
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
            "AC-003 / BC-2.16.004 PC4: 4th rebind must emit D1 finding."
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
    /// Verifies the flap window resets after 60 s: rebind after 61 s emits MEDIUM
    /// as rebind_count=1 (fresh window).
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
        assert!(high_d1.is_some(), "AC-004 setup: 3rd rebind must emit D1.");

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
             must emit D1 finding."
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
    /// Verifies that spoof_threshold=1 causes the first rebind to emit HIGH immediately.
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
            "AC-005 / BC-2.16.004 EC-008: threshold=1 first rebind must emit D1 finding."
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
    /// Verifies that a GARP frame conflicting with the binding table upgrades the GARP
    /// finding from LOW to MEDIUM and attaches T0830+T1557.002 (BC-2.16.014 PC1).
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
             when binding conflict exists (MAC_A in table, frame has MAC_B). Got {:?}.",
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
    /// Verifies that a GARP-that-conflicts frame produces exactly 2 findings:
    /// GARP upgraded to MEDIUM and D1 co-emitted (BC-2.16.014 PC2/PC5 Invariant 2).
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
             (GARP MEDIUM + D1 MEDIUM). Got {} finding(s): {:?}.",
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
    /// Verifies that when a GARP-that-conflicts frame is the rebind-threshold hit,
    /// D1 escalates to HIGH while GARP stays MEDIUM (BC-2.16.014 EC-004).
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
             exactly 2 findings. Got {} finding(s).",
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
    /// Verifies D1 evidence contains the conflicting IP, old MAC, and new MAC
    /// (BC-2.16.004 PC1.e; applies at both MEDIUM and HIGH confidence).
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
            "AC-016 / BC-2.16.004 PC1.e: D1 finding must be emitted."
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

    /// AC-017 (BC-2.16.007 PC1): D12 mismatch findings carry
    /// mitre_techniques: ["T0830", "T1557.002"] AND retain confidence=MEDIUM,
    /// category=Anomaly, and evidence fields eth_mac + arp_sender_mac + sender_ip.
    ///
    /// This test supersedes the STORY-113 assertion (which expected mitre_techniques=[]).
    /// The VP-007 5-part atomic update (co-committed with STORY-114) added T0830+T1557.002
    /// to src/mitre.rs and wired them into the D12 emission branch.
    ///
    /// Verifies D12 L2/L3 mismatch finding carries T0830+T1557.002 after the VP-007
    /// catalog update (BC-2.16.007 PC1; SEEDED=25, EMITTED=17).
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
        let mut techs = d12.mitre_techniques.clone();
        techs.sort();
        assert_eq!(
            techs,
            vec!["T0830".to_string(), "T1557.002".to_string()],
            "AC-017 / BC-2.16.007 PC1: D12 finding must carry mitre_techniques \
             [\"T0830\", \"T1557.002\"] (STORY-114 VP-007 atomic update: catalog seeding \
             and mitre vec wiring in D12 emission branch). Got: {:?}",
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

    // -----------------------------------------------------------------------
    // G1 / D-075 — BC-2.16.004 lines 45/74/118: HIGH D1 finding carries
    // Verdict::Likely (not Verdict::Possible).
    // -----------------------------------------------------------------------

    /// G1 / D-075 (BC-2.16.004 lines 45/74/118): a D1 ARP-spoof finding that
    /// escalates to HIGH confidence MUST carry `verdict: Verdict::Likely`.
    ///
    /// BC-2.16.004 line 45  (precondition): rebind_count >= spoof_threshold AND
    ///   elapsed <= ARP_FLAP_WINDOW_SECS AND !spoof_high_emitted
    ///   is the HIGH escalation condition.
    /// BC-2.16.004 line 74  (postcondition): HIGH confidence finding has
    ///   `verdict = Verdict::Likely`.
    /// BC-2.16.004 line 118 (invariant): verdict=Likely iff confidence=High on D1.
    ///
    /// Canonical test vector: threshold=3, rebind_count=3 within 60 s →
    ///   confidence=High, verdict=Likely.
    ///
    /// Regression guard for D-075 / BC-2.16.004: a HIGH-confidence D1 ARP-spoof
    /// finding MUST carry `Verdict::Likely` (MEDIUM carries `Verdict::Possible`).
    /// The fix (merged in D-075) introduced the conditional in
    /// `emit_d1_spoof_finding_impl`. This test FAILS if a future refactor routes
    /// HIGH confidence back to `Verdict::Possible`.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_d1_high_confidence_carries_verdict_likely() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        // Seed initial binding: 10.0.0.1 → MAC_A at ts=0
        seed_binding(&mut analyzer, 0);

        // Rebind 1 (count→1, MEDIUM — below threshold=3)
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_B), 2);
        // Rebind 2 (count→2, MEDIUM — below threshold=3)
        let _ = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_C), 5);
        // Rebind 3 (count→3, HIGH — count == threshold=3, elapsed=8s <= 60s)
        let findings = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_D), 8);

        // Locate the D1 spoof finding (mitre T0830 or "spoof" in summary)
        let d1 = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
                || f.summary.to_lowercase().contains("rebind")
        });
        assert!(
            d1.is_some(),
            "G1 / D-075 / BC-2.16.004 PC1.c: 3rd rebind must emit a D1 spoof finding. \
             Got {} finding(s): {:?}.",
            findings.len(),
            findings
                .iter()
                .map(|f| (&f.confidence, &f.summary))
                .collect::<Vec<_>>()
        );
        let d1 = d1.unwrap();

        // Regression guard: confidence must be HIGH (verifies escalation path is exercised).
        // This assertion is GREEN: the escalation logic is already correct for confidence.
        assert_eq!(
            d1.confidence,
            Confidence::High,
            "G1 / D-075 setup assertion: 3rd rebind at threshold=3 must produce HIGH confidence \
             (BC-2.16.004 PC1.c). Got {:?}.",
            d1.confidence
        );

        // Regression guard (BC-2.16.004 lines 74/118):
        // A HIGH D1 finding MUST carry Verdict::Likely (displays "LIKELY", serializes "Likely").
        // D-075 introduced the conditional in emit_d1_spoof_finding_impl that routes
        // HIGH confidence to Verdict::Likely. This FAILS if that conditional is removed
        // or a refactor reverts HIGH-confidence D1 findings to Verdict::Possible.
        assert_eq!(
            d1.verdict,
            Verdict::Likely,
            "G1 / D-075 / BC-2.16.004 line 74+118: HIGH D1 finding must carry \
             Verdict::Likely. Got {:?}. \
             Root cause: emit_d1_spoof_finding_impl hardcodes Verdict::Possible (line 774) \
             for both HIGH and MEDIUM D1 findings.",
            d1.verdict
        );
    }

    /// G1 / D-075 regression guard (BC-2.16.004 line 118):
    /// a D1 finding at MEDIUM confidence (below threshold) must carry
    /// `verdict: Verdict::Possible`.
    ///
    /// This assertion is GREEN now and must remain GREEN after the HIGH-path fix.
    /// It guards against overcorrection (raising all D1 verdicts to Likely).
    ///
    /// Canonical test vector: threshold=3, rebind_count=1 → confidence=Medium,
    /// verdict=Possible.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_d1_medium_confidence_carries_verdict_possible() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        // Seed initial binding: 10.0.0.1 → MAC_A at ts=0
        seed_binding(&mut analyzer, 0);

        // Rebind 1 (count→1, MEDIUM — below threshold=3)
        let findings = analyzer.process_arp_for_test(&make_reply(IP_A, MAC_B), 2);

        let d1 = findings.iter().find(|f| {
            f.mitre_techniques.contains(&"T0830".to_string())
                || f.summary.to_lowercase().contains("spoof")
                || f.summary.to_lowercase().contains("rebind")
        });
        assert!(
            d1.is_some(),
            "G1 / D-075 regression guard: first rebind must emit a D1 spoof finding. \
             Got {} finding(s): {:?}.",
            findings.len(),
            findings
                .iter()
                .map(|f| (&f.confidence, &f.summary))
                .collect::<Vec<_>>()
        );
        let d1 = d1.unwrap();

        // Confidence must be MEDIUM (below threshold).
        assert_eq!(
            d1.confidence,
            Confidence::Medium,
            "G1 / D-075 regression guard: first rebind must be MEDIUM (count=1 < threshold=3). \
             Got {:?}.",
            d1.confidence
        );

        // Verdict must be Possible for MEDIUM D1 findings (BC-2.16.004 line 118 invariant).
        // This assertion is GREEN now and must remain GREEN after the HIGH fix.
        assert_eq!(
            d1.verdict,
            Verdict::Possible,
            "G1 / D-075 regression guard / BC-2.16.004 line 118: MEDIUM D1 finding must \
             carry Verdict::Possible (not Likely). Got {:?}.",
            d1.verdict
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-115 — D3 ARP Storm Detection, --arp-storm-rate, storm_findings
// BC-2.16.008, BC-2.16.013, BC-2.16.010 cross-story extension
// ---------------------------------------------------------------------------

/// Per-story namespace wrapper per DF-TEST-NAMESPACE-001.
/// All STORY-115 unit tests live in this module (sibling of story_114).
#[cfg(test)]
mod story_115 {
    use super::*;
    use crate::decoder::ArpFrame;
    use crate::findings::{Confidence, ThreatCategory};

    // -----------------------------------------------------------------------
    // Frame builder helpers for storm tests
    // -----------------------------------------------------------------------

    /// Build a normal ARP Request from the given sender MAC and sender IP.
    ///
    /// Uses RFC 826 canonical values: op=1 (Request), htype=0x0001, ptype=0x0800,
    /// hlen=6, plen=4. outer_src_mac matches sender_mac (no D12 mismatch).
    fn storm_request(sender_mac: [u8; 6], sender_ip: [u8; 4]) -> ArpFrame {
        ArpFrame {
            operation: 1,
            sender_mac,
            sender_ip,
            target_mac: [0u8; 6],
            target_ip: [192, 168, 0, 1],
            outer_src_mac: Some(sender_mac),
            packet_len: 42,
        }
    }

    /// Canonical storm source MAC used across AC-001..014 tests.
    const STORM_MAC: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
    /// Canonical storm source IP used across tests.
    const STORM_IP: [u8; 4] = [10, 0, 0, 1];
    /// Alternate storm source MAC used for AC-011 custom-rate test.
    const ALT_MAC: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
    /// Alternate storm source IP.
    const ALT_IP: [u8; 4] = [10, 0, 0, 2];

    // -----------------------------------------------------------------------
    // AC-001 — BC-2.16.008 PC1 first-observation initializes StormCounter
    // -----------------------------------------------------------------------

    /// Verifies that the first frame from a never-before-seen source MAC initializes
    /// a StormCounter with count_in_window=1 and emits no storm finding because
    /// count=1 < storm_rate=50 (BC-2.16.008 PC1, Step 1 first-observation path).
    #[test]
    fn test_storm_first_observation_no_finding() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);
        let findings = analyzer.process_arp(&frame, 100);

        // No storm finding on first observation: count=1 < rate=50.
        let storm_findings: Vec<_> = findings
            .iter()
            .filter(|f| {
                matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
            })
            .collect();
        assert!(
            storm_findings.is_empty(),
            "AC-001 / BC-2.16.008 PC1: no storm finding must be emitted on the first \
             observation of a source MAC (count=1 < rate=50). Got {} storm-like finding(s).",
            storm_findings.len()
        );

        // storm_counters must have exactly 1 entry for the new MAC.
        assert_eq!(
            analyzer.storm_counters.len(),
            1,
            "AC-001 / BC-2.16.008 PC1: storm_counters must have 1 entry after first \
             observation. Got {}.",
            analyzer.storm_counters.len()
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 — BC-2.16.008 PC2 window-active increment
    // -----------------------------------------------------------------------

    /// Verifies that a second frame from the same source MAC within the 60-second window
    /// increments count_in_window by 1 via Step 2 (BC-2.16.008 PC2).
    #[test]
    fn test_storm_in_window_increments_count() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        // First frame at ts=100 — initializes counter (count=1)
        let _ = analyzer.process_arp(&frame, 100);
        // Second frame at ts=100 (still in-window) — must increment to count=2
        let _ = analyzer.process_arp(&frame, 100);

        let entry = analyzer.storm_counters.get(&STORM_MAC).expect(
            "AC-002 / BC-2.16.008 PC2: storm_counters must contain entry for STORM_MAC \
             after two frames.",
        );
        assert_eq!(
            entry.count_in_window, 2,
            "AC-002 / BC-2.16.008 PC2: count_in_window must be 2 after two frames in \
             the same window (first=1 init + 1 increment). Got {}.",
            entry.count_in_window
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 — BC-2.16.008 PC3 storm finding emitted at threshold
    // -----------------------------------------------------------------------

    /// Verifies a D3 storm finding is emitted at the rate threshold with
    /// confidence=MEDIUM, category=Anomaly, and mitre_techniques=[] (empty).
    /// Evidence must contain source_mac, frame_count, window_secs, rate_pps.
    ///
    /// Canonical vector: 50 frames all at ts=100 → rate=50/max(1,0)=50/1=50 >= 50.
    #[test]
    fn test_storm_finding_emitted_at_threshold() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        let mut storm_finding: Option<crate::findings::Finding> = None;
        for _ in 0..50 {
            let findings = analyzer.process_arp(&frame, 100);
            for f in findings {
                if matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    storm_finding = Some(f);
                }
            }
        }

        let f = storm_finding.expect(
            "AC-003 / BC-2.16.008 PC3: a D3 storm finding must be emitted after 50 frames \
             at ts=100 (rate=50/1=50 >= storm_rate=50). Got None.",
        );

        // confidence == MEDIUM (BC-2.16.008 PC3)
        assert_eq!(
            f.confidence,
            Confidence::Medium,
            "AC-003 / BC-2.16.008 PC3: D3 storm finding must have confidence=MEDIUM. \
             Got {:?}",
            f.confidence
        );

        // category == Anomaly (BC-2.16.008 PC3)
        assert!(
            matches!(f.category, ThreatCategory::Anomaly),
            "AC-003 / BC-2.16.008 PC3: D3 storm finding must have category=Anomaly. \
             Got {:?}",
            f.category
        );

        // mitre_techniques == [] (DF-VALIDATION-001 — T0814 withheld)
        assert!(
            f.mitre_techniques.is_empty(),
            "AC-003 / BC-2.16.008 PC3 / DF-VALIDATION-001: D3 storm finding must have \
             mitre_techniques=[] (empty). T0814 is withheld. Got: {:?}",
            f.mitre_techniques
        );

        // Evidence must contain source_mac, frame_count, window_secs, rate_pps
        let ev = f.evidence.join(" ");
        assert!(
            ev.to_lowercase().contains("source_mac")
                || ev.to_lowercase().contains("aa:bb:cc:dd:ee:ff")
                || ev.to_lowercase().contains("aa-bb-cc-dd-ee-ff"),
            "AC-003 / BC-2.16.008 PC3: D3 finding evidence must contain source_mac. \
             Got: {:?}",
            f.evidence
        );
        assert!(
            ev.to_lowercase().contains("frame_count") || ev.to_lowercase().contains("count"),
            "AC-003 / BC-2.16.008 PC3: D3 finding evidence must contain frame_count. \
             Got: {:?}",
            f.evidence
        );
        assert!(
            ev.to_lowercase().contains("window_secs") || ev.to_lowercase().contains("window"),
            "AC-003 / BC-2.16.008 PC3: D3 finding evidence must contain window_secs. \
             Got: {:?}",
            f.evidence
        );
        assert!(
            ev.to_lowercase().contains("rate_pps") || ev.to_lowercase().contains("rate"),
            "AC-003 / BC-2.16.008 PC3: D3 finding evidence must contain rate_pps. \
             Got: {:?}",
            f.evidence
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 — BC-2.16.008 PC4 one-shot guard prevents second finding
    // -----------------------------------------------------------------------

    /// Verifies that after storm_emitted=true, additional frames from the same MAC
    /// in the same window do NOT trigger further storm findings (BC-2.16.008 PC4,
    /// Invariant 1 — one-shot per window).
    #[test]
    fn test_storm_one_shot_guard_prevents_second_finding() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        // Send 60 frames at ts=100 (first storm fires at frame 50; frames 51-60 must
        // not produce additional storm findings due to one-shot guard).
        let mut storm_count = 0usize;
        for _ in 0..60 {
            let findings = analyzer.process_arp(&frame, 100);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    storm_count += 1;
                }
            }
        }

        assert_eq!(
            storm_count, 1,
            "AC-004 / BC-2.16.008 PC4 / Invariant 1: exactly 1 storm finding must be \
             emitted for 60 frames in the same window (one-shot guard). Got {storm_count}."
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 — BC-2.16.008 PC1 Step-1 window expiry resets counter
    // -----------------------------------------------------------------------

    /// Verifies that when timestamp_secs - window_start_ts > ARP_FLAP_WINDOW_SECS=60,
    /// the counter resets: count_in_window=1, window_start_ts=new_ts,
    /// storm_emitted=false (BC-2.16.008 PC1 window-expiry branch).
    #[test]
    fn test_storm_window_expiry_resets_counter() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        // Send 30 frames at ts=100 (count=30, no storm yet)
        for _ in 0..30 {
            let _ = analyzer.process_arp(&frame, 100);
        }

        // Now send 1 frame at ts=162 (elapsed = 162-100=62 > 60 → window expired)
        let _ = analyzer.process_arp(&frame, 162);

        let entry = analyzer
            .storm_counters
            .get(&STORM_MAC)
            .expect("AC-005 / BC-2.16.008 PC1: storm_counters must contain STORM_MAC entry.");

        assert_eq!(
            entry.count_in_window, 1,
            "AC-005 / BC-2.16.008 PC1: after window expiry (elapsed=62 > 60), \
             count_in_window must reset to 1. Got {}.",
            entry.count_in_window
        );
        assert_eq!(
            entry.window_start_ts, 162,
            "AC-005 / BC-2.16.008 PC1: after window expiry, window_start_ts must \
             reset to the new timestamp (162). Got {}.",
            entry.window_start_ts
        );
        assert!(
            !entry.storm_emitted,
            "AC-005 / BC-2.16.008 PC1: after window expiry, storm_emitted must \
             reset to false."
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 — BC-2.16.008 PC3 Note: same-second denominator is max(1,0)=1
    // -----------------------------------------------------------------------

    /// Verifies the rate formula uses max(1, ts - window_start_ts): when all 50
    /// frames arrive at ts=100 (same second as window_start_ts), the denominator
    /// is max(1,0)=1, giving rate=50/1=50 >= 50, which triggers the finding.
    /// No divide-by-zero occurs (ARP-AMB-003 RESOLVED; BC-2.16.008 PC3 Note 6).
    #[test]
    fn test_storm_same_second_denominator_is_1() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        // All 50 frames at ts=100 (same second as window_start_ts=100)
        let mut got_storm = false;
        for _ in 0..50 {
            let findings = analyzer.process_arp(&frame, 100);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    got_storm = true;
                }
            }
        }

        assert!(
            got_storm,
            "AC-006 / BC-2.16.008 PC3 Note: 50 frames all at ts=100 must trigger a \
             storm finding (rate=50/max(1,0)=50/1=50 >= 50). The denominator guard \
             max(1,...) prevents divide-by-zero AND allows same-second burst detection. \
             Got no storm finding."
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 — BC-2.16.008 EC-001/EC-002: 49 below threshold, 50 at threshold
    // -----------------------------------------------------------------------

    /// Verifies that 49 frames at ts=100 (rate=49/1=49 < 50) produce no storm
    /// finding, and that 50 frames at ts=100 (rate=50/1=50 >= 50) produce one
    /// (BC-2.16.008 EC-001 and EC-002 canonical vectors).
    #[test]
    fn test_storm_49_below_threshold_50_at_threshold() {
        // --- 49 frames: no storm ---
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        let mut got_storm_49 = false;
        for _ in 0..49 {
            let findings = analyzer.process_arp(&frame, 100);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    got_storm_49 = true;
                }
            }
        }
        assert!(
            !got_storm_49,
            "AC-007 / BC-2.16.008 EC-001: 49 frames at ts=100 must NOT trigger a storm \
             finding (rate=49/1=49 < 50)."
        );

        // --- 50 frames: storm fires on the 50th ---
        let mut analyzer2 = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let mut got_storm_50 = false;
        for _ in 0..50 {
            let findings = analyzer2.process_arp(&frame, 100);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    got_storm_50 = true;
                }
            }
        }
        assert!(
            got_storm_50,
            "AC-007 / BC-2.16.008 EC-002: 50 frames at ts=100 must trigger a storm \
             finding (rate=50/1=50 >= 50)."
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 — BC-2.16.008 EC-009/EC-010: window boundary exact 60 and 61
    // -----------------------------------------------------------------------

    /// Verifies the window boundary condition: elapsed=60 is still in-window (<=60),
    /// elapsed=61 triggers a reset (>60) (BC-2.16.008 EC-009 and EC-010).
    #[test]
    fn test_storm_window_boundary_60_in_window_61_expired() {
        let frame = storm_request(STORM_MAC, STORM_IP);

        // --- elapsed=60: still in-window, count continues accumulating ---
        let mut analyzer_60 = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        // First frame at ts=100: window_start_ts=100, count=1
        let _ = analyzer_60.process_arp(&frame, 100);
        // Frame at ts=160: elapsed=60 <= 60 → still in-window; increment to count=2
        let _ = analyzer_60.process_arp(&frame, 160);
        let entry_60 = analyzer_60
            .storm_counters
            .get(&STORM_MAC)
            .expect("AC-008: storm_counters must contain STORM_MAC.");
        assert_eq!(
            entry_60.window_start_ts, 100,
            "AC-008 / BC-2.16.008 EC-009: elapsed=60 must NOT reset the window; \
             window_start_ts must remain 100. Got {}.",
            entry_60.window_start_ts
        );
        assert_eq!(
            entry_60.count_in_window, 2,
            "AC-008 / BC-2.16.008 EC-009: elapsed=60 is still in-window; \
             count_in_window must be 2 (1 init + 1 increment). Got {}.",
            entry_60.count_in_window
        );

        // --- elapsed=61: window expired, counter resets ---
        let mut analyzer_61 = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        // Prime counter to a high count to demonstrate expiry discards it
        for _ in 0..30 {
            let _ = analyzer_61.process_arp(&frame, 100);
        }
        // Frame at ts=161: elapsed=61 > 60 → window expired; resets to count=1
        let _ = analyzer_61.process_arp(&frame, 161);
        let entry_61 = analyzer_61
            .storm_counters
            .get(&STORM_MAC)
            .expect("AC-008: storm_counters must contain STORM_MAC after expiry frame.");
        assert_eq!(
            entry_61.count_in_window, 1,
            "AC-008 / BC-2.16.008 EC-010: elapsed=61 > 60 must reset count_in_window to 1. \
             Got {}.",
            entry_61.count_in_window
        );
        assert_eq!(
            entry_61.window_start_ts, 161,
            "AC-008 / BC-2.16.008 EC-010: elapsed=61 must reset window_start_ts to 161. \
             Got {}.",
            entry_61.window_start_ts
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 — BC-2.16.008 EC-011 / Invariant 2: late-burst suppression
    // -----------------------------------------------------------------------

    /// Verifies that 49 frames at ts=100 followed by 50 frames at ts=159 (same window,
    /// window_start_ts=100) produce NO storm finding: at ts=159, count=99,
    /// elapsed=59, rate=99/59≈1.68 < 50 (BC-2.16.008 Invariant 2 — accepted limitation).
    ///
    /// Documents the average-since-window-start limitation.
    #[test]
    fn test_storm_late_burst_suppression_accepted_limitation() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        // 49 frames at ts=100 (count=49, rate=49/1=49 < 50 — no storm)
        let mut got_storm = false;
        for _ in 0..49 {
            let findings = analyzer.process_arp(&frame, 100);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    got_storm = true;
                }
            }
        }

        // 50 more frames at ts=159 (count=99, elapsed=59, rate≈1.68 < 50 — no storm)
        for _ in 0..50 {
            let findings = analyzer.process_arp(&frame, 159);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    got_storm = true;
                }
            }
        }

        assert!(
            !got_storm,
            "AC-009 / BC-2.16.008 EC-011 / Invariant 2: 49 frames at ts=100 + 50 frames \
             at ts=159 must NOT trigger a storm finding (rate=99/59≈1.68 < 50). \
             This is the accepted average-since-window-start limitation."
        );
    }

    // -----------------------------------------------------------------------
    // AC-010 — BC-2.16.008 PC5: storm counter cap MAX_STORM_COUNTERS=4096
    // -----------------------------------------------------------------------

    /// Verifies that storm_counters.len() never exceeds MAX_STORM_COUNTERS=4096
    /// when 4097 distinct source MACs each send one frame (BC-2.16.008 PC5,
    /// Invariant 5 — LRU eviction analogous to binding table).
    #[test]
    fn test_storm_counter_cap_enforced() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);

        for i in 0u32..=4096u32 {
            // Build a unique MAC from i (4097 distinct MACs: i=0..=4096)
            let mac: [u8; 6] = [
                0xDE,
                0xAD,
                ((i >> 24) & 0xFF) as u8,
                ((i >> 16) & 0xFF) as u8,
                ((i >> 8) & 0xFF) as u8,
                (i & 0xFF) as u8,
            ];
            let frame = storm_request(mac, [10, 1, (i >> 8) as u8, (i & 0xFF) as u8]);
            let _ = analyzer.process_arp(&frame, i);

            assert!(
                analyzer.storm_counters.len() <= MAX_STORM_COUNTERS,
                "AC-010 / BC-2.16.008 PC5: storm_counters.len() must never exceed \
                 MAX_STORM_COUNTERS={} at any point. After {} MACs, len={}.",
                MAX_STORM_COUNTERS,
                i + 1,
                analyzer.storm_counters.len()
            );
        }

        assert_eq!(
            analyzer.storm_counters.len(),
            MAX_STORM_COUNTERS,
            "AC-010 / BC-2.16.008 PC5: after 4097 distinct MACs, storm_counters.len() \
             must equal MAX_STORM_COUNTERS={} (one eviction on the 4097th). Got {}.",
            MAX_STORM_COUNTERS,
            analyzer.storm_counters.len()
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 — BC-2.16.013 PC1/2: ArpAnalyzer::new uses storm_rate parameter
    // -----------------------------------------------------------------------

    /// Verifies that ArpAnalyzer::new(spoof_threshold, storm_rate=10) uses storm_rate=10
    /// for D3 detection: 10 frames at ts=200 → rate=10/1=10 >= 10 → storm finding
    /// (BC-2.16.013 PC1, EC-001; canonical vector row 6 from BC-2.16.008).
    #[test]
    fn test_storm_custom_rate_10() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 10);
        let frame = storm_request(ALT_MAC, ALT_IP);

        let mut got_storm = false;
        for _ in 0..10 {
            let findings = analyzer.process_arp(&frame, 200);
            for f in &findings {
                if matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    got_storm = true;
                }
            }
        }

        assert!(
            got_storm,
            "AC-011 / BC-2.16.013 PC1: with storm_rate=10, 10 frames at ts=200 must \
             trigger a storm finding (rate=10/max(1,0)=10/1=10 >= 10). Got no storm."
        );
    }

    // -----------------------------------------------------------------------
    // AC-013 — BC-2.16.010 cross-story: storm_findings key non-zero after D3
    // -----------------------------------------------------------------------

    /// Verifies that after a D3 storm finding is emitted, summarize()["storm_findings"] > 0
    /// (BC-2.16.010 cross-story extension: storm_findings VALUE wired in STORY-115).
    #[test]
    fn test_summarize_storm_findings_key_non_zero_after_detection() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        // Send 50 frames at ts=100 to trigger D3 (rate=50/1=50 >= 50)
        for _ in 0..50 {
            let _ = analyzer.process_arp(&frame, 100);
        }

        let summary = analyzer.summarize();
        let storm_val = summary
            .detail
            .get("storm_findings")
            .and_then(|v| v.as_u64())
            .expect(
                "AC-013 / BC-2.16.010: summarize() must contain 'storm_findings' key. \
                 Key is missing.",
            );

        assert!(
            storm_val > 0,
            "AC-013 / BC-2.16.010 cross-story: storm_findings must be > 0 after a D3 \
             detection fires (50 frames at ts=100 → rate=50 >= threshold=50). Got \
             {storm_val}."
        );
    }

    // -----------------------------------------------------------------------
    // AC-014 — BC-2.16.008 Invariant 3 / DF-VALIDATION-001: empty MITRE
    // -----------------------------------------------------------------------

    /// Verifies that every D3 storm finding has mitre_techniques=[] and that
    /// T0814 is explicitly NOT present (BC-2.16.008 Invariant 3, DF-VALIDATION-001).
    #[test]
    fn test_d3_finding_has_empty_mitre_techniques() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);
        let frame = storm_request(STORM_MAC, STORM_IP);

        let mut d3_findings: Vec<crate::findings::Finding> = Vec::new();
        for _ in 0..50 {
            let findings = analyzer.process_arp(&frame, 100);
            for f in findings {
                if matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    d3_findings.push(f);
                }
            }
        }

        assert!(
            !d3_findings.is_empty(),
            "AC-014 / BC-2.16.008 Invariant 3: at least one D3 storm finding must be \
             emitted (50 frames at ts=100 → rate=50 >= 50). Got none."
        );

        for f in &d3_findings {
            assert!(
                f.mitre_techniques.is_empty(),
                "AC-014 / BC-2.16.008 Invariant 3 / DF-VALIDATION-001: D3 finding \
                 mitre_techniques must be [] (empty). T0814 is withheld per \
                 DF-VALIDATION-001. Got: {:?}",
                f.mitre_techniques
            );

            assert!(
                !f.mitre_techniques.iter().any(|t| t == "T0814"),
                "AC-014 / BC-2.16.008 Invariant 3 / DF-VALIDATION-001: T0814 must NOT \
                 be present in any D3 storm finding's mitre_techniques. Got: {:?}",
                f.mitre_techniques
            );
        }
    }
    // -----------------------------------------------------------------------
    // F1/C1 — Regression guard: GARP flood must trigger D3 storm (GREEN — detect_storm
    // runs before the GARP branch in process_arp, so GARP floods are covered)
    // -----------------------------------------------------------------------

    /// Regression guard (BC-2.16.008 keys D3 on sender_mac for ALL ARP frames, no GARP
    /// exclusion): detect_storm runs before the GARP branch in process_arp, so GARP floods
    /// are covered. This test asserts that a 50-GARP flood at ts=100 emits a D3
    /// MEDIUM/Anomaly finding (sender_ip == target_ip, same sender_mac,
    /// rate=50/max(1,0)=50 >= storm_rate=50). If this test regresses it means the
    /// call-site ordering in process_arp was changed so that GARP frames bypass detect_storm.
    #[test]
    fn test_storm_detected_for_garp_flood() {
        // Construct 50 GARP frames: sender_ip == target_ip (GARP condition),
        // same sender_mac, RFC 826 canonical values: op=1, htype=0x0001, ptype=0x0800,
        // hlen=6, plen=4.
        let garp_mac: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
        let garp_ip: [u8; 4] = [10, 0, 0, 1];
        // GARP: sender_ip == target_ip
        let garp_frame = ArpFrame {
            operation: 1,
            sender_mac: garp_mac,
            sender_ip: garp_ip,
            target_mac: [0u8; 6],
            target_ip: garp_ip, // sender_ip == target_ip → GARP
            outer_src_mac: Some(garp_mac),
            packet_len: 42,
        };

        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);

        // Process all 50 GARP frames at ts=100.
        // Rate = 50 / max(1, 100-100) = 50/1 = 50 >= storm_rate=50 → D3 must fire.
        let mut d3_storm_finding: Option<crate::findings::Finding> = None;
        for _ in 0..50 {
            let findings = analyzer.process_arp(&garp_frame, 100);
            for f in findings {
                // D3 storm finding: confidence MEDIUM, category Anomaly, storm keyword.
                // Distinguish from D2 GARP LOW findings (those have confidence LOW).
                if matches!(f.confidence, Confidence::Medium)
                    && matches!(f.category, ThreatCategory::Anomaly)
                    && (f.summary.to_lowercase().contains("storm")
                        || f.summary.to_lowercase().contains("d3"))
                {
                    d3_storm_finding = Some(f);
                }
            }
        }

        // Assert the D3 storm finding IS emitted.
        // Regression: if this fails, the call-site ordering in process_arp was changed so
        // that GARP frames no longer reach detect_storm, re-introducing the GARP exclusion
        // that BC-2.16.008 forbids.
        let d3 = d3_storm_finding.expect(
            "Regression / BC-2.16.008: a D3 storm finding (confidence=MEDIUM, \
             category=Anomaly, storm keyword) must be emitted after 50 GARP frames from \
             one source MAC at ts=100 (rate=50/1=50 >= storm_rate=50). \
             BC-2.16.008 specifies no GARP exclusion from storm detection. \
             Failure means process_arp was changed so GARP floods no longer reach \
             detect_storm, breaking BC-2.16.008 coverage for GARP frames.",
        );

        // Verify confidence == MEDIUM.
        assert_eq!(
            d3.confidence,
            Confidence::Medium,
            "F1/C1 / BC-2.16.008: D3 storm finding from GARP flood must have \
             confidence=MEDIUM. Got {:?}.",
            d3.confidence
        );

        // Verify category == Anomaly.
        assert!(
            matches!(d3.category, ThreatCategory::Anomaly),
            "F1/C1 / BC-2.16.008: D3 storm finding from GARP flood must have \
             category=Anomaly. Got {:?}.",
            d3.category
        );

        // Verify mitre_techniques == [] (DF-VALIDATION-001 — T0814 withheld).
        assert!(
            d3.mitre_techniques.is_empty(),
            "F1/C1 / BC-2.16.008 Invariant 3 / DF-VALIDATION-001: D3 storm finding \
             must have mitre_techniques=[] (empty). Got: {:?}.",
            d3.mitre_techniques
        );

        // Verify evidence contains source_mac, frame_count, window_secs, rate_pps
        // (BC-2.16.008 PC3: evidence fields).
        let ev = d3.evidence.join(" ");
        assert!(
            ev.to_lowercase().contains("source_mac")
                || ev.to_lowercase().contains("aa:bb:cc:dd:ee:ff")
                || ev.to_lowercase().contains("aa-bb-cc-dd-ee-ff"),
            "F1/C1 / BC-2.16.008: D3 finding evidence must contain source_mac. \
             Got: {:?}.",
            d3.evidence
        );
        assert!(
            ev.to_lowercase().contains("frame_count") || ev.to_lowercase().contains("count"),
            "F1/C1 / BC-2.16.008: D3 finding evidence must contain frame_count. \
             Got: {:?}.",
            d3.evidence
        );
        assert!(
            ev.to_lowercase().contains("window_secs") || ev.to_lowercase().contains("window"),
            "F1/C1 / BC-2.16.008: D3 finding evidence must contain window_secs. \
             Got: {:?}.",
            d3.evidence
        );
        assert!(
            ev.to_lowercase().contains("rate_pps") || ev.to_lowercase().contains("rate"),
            "F1/C1 / BC-2.16.008: D3 finding evidence must contain rate_pps. \
             Got: {:?}.",
            d3.evidence
        );
    }

    // -----------------------------------------------------------------------
    // F-1 Regression guard (GREEN): insert_storm_counter_lru HAS a contains_key
    // guard (mirroring insert_binding_lru), so re-initialising an ALREADY-PRESENT
    // MAC at cap does NOT evict an innocent (min-window_start_ts) MAC.
    // -----------------------------------------------------------------------

    /// Regression guard (GREEN): `insert_storm_counter_lru` (arp.rs) HAS a `contains_key`
    /// guard, mirroring `insert_binding_lru`. When the map is at MAX_STORM_COUNTERS and
    /// detect_storm calls `insert_storm_counter_lru` to re-initialize an already-present
    /// MAC whose window has expired, the function performs an in-place overwrite and does
    /// NOT evict the min-window_start_ts entry (the LRU "innocent" MAC). This test asserts
    /// the guard holds; it would FAIL if a regression removed the contains_key check.
    ///
    /// Setup:
    ///   1. Fill storm_counters to exactly MAX_STORM_COUNTERS=4096 distinct source MACs,
    ///      one frame each, with varied window_start_ts so the entries have a clear LRU
    ///      ordering. The first MAC inserted (ts=0) is the "innocent" min-window_start_ts
    ///      victim and is deliberately DISTINCT from the MAC we will re-initialize.
    ///   2. Pick one ALREADY-PRESENT MAC (NOT the innocent one) and send it another frame
    ///      at timestamp_secs = window_start_ts + ARP_FLAP_WINDOW_SECS + 1, so its window
    ///      has expired and detect_storm calls insert_storm_counter_lru.
    ///
    /// Expected (correct behaviour, enforced by the contains_key guard):
    ///   (a) storm_counters.len() == MAX_STORM_COUNTERS (no net growth or shrinkage).
    ///   (b) The innocent min-window_start_ts MAC is STILL present (NOT evicted).
    ///
    /// A regression that removed the contains_key guard would cause assertion (b) to FAIL:
    /// the re-init of the already-present MAC would trigger spurious eviction of the innocent
    /// min-window_start_ts MAC even though the insert did not need to grow the map.
    ///
    /// BC-2.16.008 PC5 / Invariant 6 — spurious-eviction regression guard.
    #[test]
    fn test_storm_lru_no_spurious_eviction_on_existing_mac_reinit() {
        let mut analyzer = ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, 50);

        // Step 1: populate storm_counters to exactly MAX_STORM_COUNTERS=4096 distinct
        // source MACs. Each MAC gets one frame at ts=i, giving varied window_start_ts.
        // MAC i=0 will have window_start_ts=0, i.e. the minimum — it is the LRU victim.
        //
        // We use the same scheme as test_storm_counter_cap_enforced (AC-010):
        //   mac = [0xDE, 0xAD, byte3, byte2, byte1, byte0] derived from i.
        //
        // MAC i=0 ([0xDE,0xAD,0x00,0x00,0x00,0x00]) → "innocent" LRU victim (ts=0).
        // MAC i=1 ([0xDE,0xAD,0x00,0x00,0x00,0x01]) → the MAC we will re-initialize.
        //
        // Use distinct sender IPs per MAC to avoid binding-table interactions.
        let innocent_mac: [u8; 6] = [0xDE, 0xAD, 0x00, 0x00, 0x00, 0x00];
        let reinit_mac: [u8; 6] = [0xDE, 0xAD, 0x00, 0x00, 0x00, 0x01];

        for i in 0u32..(MAX_STORM_COUNTERS as u32) {
            let mac: [u8; 6] = [
                0xDE,
                0xAD,
                ((i >> 24) & 0xFF) as u8,
                ((i >> 16) & 0xFF) as u8,
                ((i >> 8) & 0xFF) as u8,
                (i & 0xFF) as u8,
            ];
            // Unique IP per MAC (avoids binding conflicts).
            let ip: [u8; 4] = [
                10,
                ((i >> 16) & 0xFF) as u8,
                ((i >> 8) & 0xFF) as u8,
                (i & 0xFF) as u8,
            ];
            let frame = storm_request(mac, ip);
            let _ = analyzer.process_arp(&frame, i);
        }

        // Confirm the map is exactly at cap.
        assert_eq!(
            analyzer.storm_counters.len(),
            MAX_STORM_COUNTERS,
            "Setup: storm_counters must be at MAX_STORM_COUNTERS={} before the re-init \
             step. Got {}.",
            MAX_STORM_COUNTERS,
            analyzer.storm_counters.len()
        );

        // Confirm the innocent MAC (i=0, ts=0) is present — it is the LRU victim.
        assert!(
            analyzer.storm_counters.contains_key(&innocent_mac),
            "Setup: innocent MAC {innocent_mac:?} must be present in storm_counters before \
             re-init.",
        );

        // Confirm reinit_mac (i=1, ts=1) is present with window_start_ts=1.
        let reinit_entry_ts = analyzer
            .storm_counters
            .get(&reinit_mac)
            .expect("Setup: reinit_mac must be present in storm_counters.")
            .window_start_ts;

        // Step 2: send reinit_mac another frame at timestamp = window_start_ts + ARP_FLAP_WINDOW_SECS + 1
        // so the window has expired (elapsed > ARP_FLAP_WINDOW_SECS=60).
        // detect_storm takes the !in_window path and calls insert_storm_counter_lru.
        // The map is at cap (4096), so the buggy code evicts innocent_mac (min ts=0)
        // even though reinit_mac is already present and the insert is a key-overwrite.
        let expired_ts = reinit_entry_ts + ARP_FLAP_WINDOW_SECS + 1;
        let reinit_ip: [u8; 4] = [10, 0, 0, 1];
        let reinit_frame = storm_request(reinit_mac, reinit_ip);
        let _ = analyzer.process_arp(&reinit_frame, expired_ts);

        // Assertion (a): map length must still be MAX_STORM_COUNTERS (re-init is an
        // in-place overwrite; the innocent MAC must not have been evicted to make room).
        assert_eq!(
            analyzer.storm_counters.len(),
            MAX_STORM_COUNTERS,
            "F-1 spurious-eviction regression / BC-2.16.008 PC5: re-initialising an \
             already-present MAC at cap must not change storm_counters.len(). Expected \
             {}, got {}.",
            MAX_STORM_COUNTERS,
            analyzer.storm_counters.len()
        );

        // Assertion (b): the innocent min-window_start_ts MAC must still be present.
        // Regression guard: if the contains_key guard were removed, this assertion would FAIL
        // because insert_storm_counter_lru would evict the innocent MAC instead of overwriting
        // in place (unlike insert_binding_lru which has always had the guard).
        assert!(
            analyzer.storm_counters.contains_key(&innocent_mac),
            "F-1 spurious-eviction regression / BC-2.16.008 PC5: the innocent \
             min-window_start_ts MAC {innocent_mac:?} was spuriously evicted when \
             re-initialising already-present MAC {reinit_mac:?} at cap. \
             insert_storm_counter_lru must check contains_key before evicting, as \
             insert_binding_lru does.",
        );
    }
} // mod story_115

// ---------------------------------------------------------------------------
// BC-2.16.016 test — ARP Findings Output is Unbounded (characterization)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod bc_2_16_016 {
    //! BC-2.16.016 characterization test: `process_arp` returns a `Vec<Finding>`
    //! with NO upper bound. Processing >10,000 rebind-triggering frames must
    //! produce >10,000 findings with no MAX_FINDINGS cap applied.
    //!
    //! This test PASSES on the current codebase (the no-cap invariant is already
    //! in place) and acts as a regression guard: if a MAX_FINDINGS cap is ever
    //! accidentally added to the ARP path, this test will fail.
    //!
    //! DF-TEST-NAMESPACE-001: tests wrapped in `mod bc_2_16_016`.
    //! DF-AC-TEST-NAME-SYNC-001: function name follows BC-S.SS.NNN convention.

    use super::*;
    use crate::decoder::ArpFrame;

    /// Build a minimal ARP Reply (op=2) with the given sender_ip, sender_mac,
    /// and matching outer_src_mac. Used for characterization-test frame synthesis.
    ///
    /// target_ip is fixed to `192.168.0.1` — outside the `10.x.x.x` range used
    /// for sender_ip in the no-cap test, so sender_ip never equals target_ip and
    /// no GARP detection fires (D2 GARP requires sender_ip == target_ip per RFC 826).
    fn make_reply_frame(sender_ip: [u8; 4], sender_mac: [u8; 6]) -> ArpFrame {
        ArpFrame {
            operation: 2,
            sender_mac,
            sender_ip,
            target_mac: [0u8; 6],
            target_ip: [192, 168, 0, 1],
            outer_src_mac: Some(sender_mac),
            packet_len: 42,
        }
    }

    /// BC-2.16.016 characterization test: ARP findings Vec is NOT capped.
    ///
    /// Processes N = 10,001 distinct sender IPs × 2 frames each (alternating
    /// MACs), for a total of 20,002 frames. With `spoof_threshold=1`, each
    /// second frame triggers a D1 rebind finding immediately. Expected:
    /// `all_findings.len() == 10_001` (exactly one D1 per IP) and
    /// `all_findings.len() > 10_000` (the no-cap invariant is satisfied).
    ///
    /// Postcondition 1 of BC-2.16.016: the Vec<Finding> returned across all
    /// `process_arp` calls is NOT truncated by any MAX_FINDINGS constant.
    ///
    /// Regression guard (BC-2.16.016 Invariant 5): if a MAX_FINDINGS cap is
    /// ever added to the ARP path, `all_findings.len()` will plateau at that
    /// cap and the `== 10_001` assertion will fail.
    ///
    /// This test is expected to PASS on the current codebase.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_016_arp_findings_vec_has_no_cap() {
        // storm_rate=u32::MAX suppresses all D3 findings so the count is purely D1.
        let mut analyzer = ArpAnalyzer::new(1, u32::MAX);

        const MAC_A: [u8; 6] = [0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
        const MAC_B: [u8; 6] = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];
        const N: usize = 10_001; // number of distinct sender IPs

        let mut all_findings: Vec<Finding> = Vec::new();

        for i in 0..N {
            // Synthesize a unique sender_ip in the range 10.0.X.Y (N <= 65535 so
            // the address space fits without wrapping into broadcast addresses).
            let hi = (i / 256) as u8;
            let lo = (i % 256) as u8;
            let sender_ip: [u8; 4] = [10, 0, hi, lo];
            let ts = i as u32;

            // Frame 1 for this IP: first observation — inserts binding, no D1 finding.
            let frame1 = make_reply_frame(sender_ip, MAC_A);
            let f1 = analyzer.process_arp(&frame1, ts);
            all_findings.extend(f1);

            // Frame 2 for this IP: rebind (MAC_A → MAC_B).
            // spoof_threshold=1 → rebind_count(1) >= threshold(1) → HIGH D1 emitted.
            let frame2 = make_reply_frame(sender_ip, MAC_B);
            let f2 = analyzer.process_arp(&frame2, ts);
            all_findings.extend(f2);
        }

        // BC-2.16.016 Postcondition 1: findings Vec must NOT be truncated.
        // With N=10,001 IPs each producing exactly one D1 finding on rebind,
        // the total must equal N — not be capped at 10,000 (the reassembly
        // MAX_FINDINGS constant that applies only to HTTP/TLS/Modbus/DNP3).
        assert_eq!(
            all_findings.len(),
            N,
            "BC-2.16.016 PC1: all_findings.len() must equal N={N} (one D1 finding per IP \
             rebind, no MAX_FINDINGS cap). Got {}. If this is 10,000, a MAX_FINDINGS cap \
             has been accidentally introduced on the ARP path.",
            all_findings.len()
        );

        // Belt-and-suspenders: explicitly assert > 10,000 to document the no-cap intent.
        assert!(
            all_findings.len() > 10_000,
            "BC-2.16.016 PC1 (no-cap invariant): all_findings.len() must exceed 10,000 \
             (the reassembly MAX_FINDINGS constant) to prove the ARP path is unbounded. \
             Got {}.",
            all_findings.len()
        );

        // BC-2.16.016 PC2: no dropped_findings counter — ArpAnalyzer has no such field.
        // Confirmed implicitly: `all_findings.len() == N` with no dropped_findings signal.

        // BC-2.16.016 PC3: summarize() must NOT contain a dropped_findings key.
        let summary = analyzer.summarize();
        assert!(
            !summary.detail.contains_key("dropped_findings"),
            "BC-2.16.016 PC3 / BC-2.16.010 Invariant 1: summarize() must NOT emit a \
             'dropped_findings' key. Found unexpected key. Keys present: {:?}",
            summary.detail.keys().collect::<Vec<_>>()
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
    /// Body filled and proven at the F6 formal-hardening gate (VP-024 v2.0,
    /// verification_lock: true).
    #[kani::proof]
    fn verify_classify_garp_total() {
        let frame = ArpFrame {
            operation: kani::any(),
            sender_mac: kani::any(),
            sender_ip: kani::any(),
            target_mac: kani::any(),
            target_ip: kani::any(),
            outer_src_mac: kani::any(),
            packet_len: kani::any(),
        };
        // No panic for any symbolic ArpFrame (pure boolean predicate).
        let is_garp = is_gratuitous_arp(&frame);
        // Biconditional totality: GARP iff sender_ip == target_ip, for ALL
        // operation values (opcode-agnostic — operation is fully symbolic).
        assert!(is_garp == (frame.sender_ip == frame.target_ip));
    }

    /// VP-024 Sub-D: binding table cap invariant (BC-2.16.006).
    ///
    /// Asserts `len <= cap` after each insert, including the cap→cap+1 boundary
    /// where LRU eviction fires. `TEST_MAX_ARP_BINDINGS = 8`; cap+1 = 9 inserts
    /// (0..=8), matching the VP-024 Sub-D skeleton's iteration count.
    ///
    /// CONTAINER SUBSTITUTION (CBMC tractability — documented deviation from the
    /// skeleton's `insert_binding_lru_btree`): CBMC cannot symbolically execute
    /// `std::collections::BTreeMap` within a practical resource budget. A probe
    /// confirmed that even three plain `BTreeMap::insert` calls exhaust CBMC's
    /// memory during SSA conversion ("CBMC appears to have run out of memory");
    /// the `insert_binding_lru_btree` sequence did not resolve after 45+ minutes.
    /// This harness therefore exercises `insert_binding_lru_array`, a CBMC-
    /// tractable fixed-capacity-array surrogate that reproduces the IDENTICAL
    /// three-branch eviction algorithm (update-in-place / evict-min-last_seen_ts /
    /// append). VP-024 Proof Method authorizes this: the cap invariant "is a
    /// purely arithmetic property independent of which ordered/unordered map is
    /// used; the proof is valid for the production HashMap by substitution."
    /// `insert_binding_lru_btree` remains in the tree and is exercised by the
    /// concrete unit test `test_BC_2_16_006_binding_table_cap_enforced`.
    ///
    /// SYMBOLIC INPUTS: MACs are symbolic (`kani::any()`), per the VP-024
    /// symbolic-input table. IPs are distinct per iteration (every insert is a
    /// genuine new key reaching the eviction boundary). `last_seen_ts` is set to a
    /// distinct per-iteration value so the min-scan eviction is well-defined; the
    /// proved obligation is the count invariant `len <= cap`, independent of which
    /// entry is chosen for eviction (LRU target-correctness is unit-tested, not
    /// Kani-proven — VP-024 Sub-D scope note).
    ///
    /// UNWIND BOUND: `#[kani::unwind(12)]` bounds the cap+1 = 9-iteration outer
    /// loop and the inner min-scan / lookup loops (each <= cap = 8 iterations).
    ///
    /// AC-019; runs at F6 formal-hardening gate.
    #[kani::proof]
    #[kani::unwind(12)]
    fn verify_binding_table_cap() {
        const CAP: usize = TEST_MAX_ARP_BINDINGS; // 8
        // Backing array sized to cap (the surrogate never holds more than `cap`
        // live entries, so capacity == cap is sufficient).
        let mut entries: [([u8; 4], [u8; 6], u32); CAP] = [([0u8; 4], [0u8; 6], 0u32); CAP];
        let mut len: usize = 0;
        // Insert cap+1 = 9 frames with distinct IPs. The cap invariant must hold
        // after every insert, including the boundary transition at cap and cap+1
        // where LRU eviction fires.
        let mut i = 0u8;
        while i <= (CAP as u8) {
            let ip: [u8; 4] = [0, 0, 0, i];
            let mac: [u8; 6] = kani::any();
            // Distinct, increasing last_seen_ts so the eviction min-scan is total.
            insert_binding_lru_array(&mut entries, &mut len, ip, mac, CAP);
            // Stamp the freshly-inserted/updated entry's last_seen_ts to keep the
            // LRU ordering well-defined (mirrors process_arp writing last_seen_ts
            // after insert). The cap invariant does not depend on this value.
            if len > 0 {
                entries[len - 1].2 = i as u32;
            }
            // BC-2.16.006: table never exceeds the cap.
            assert!(len <= CAP);
            i += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// BC-2.16.004 v1.10 Invariant 6 — by-construction `.expect()` regression guards
// ---------------------------------------------------------------------------
//
// These tests cover EC-011 and EC-012 from BC-2.16.004 v1.10 and serve as
// regression guards for the four `.expect()` sites in `process_arp` and
// `emit_d1_spoof_finding_impl`.  No production code change is involved —
// the `.expect()` calls remain as-is per the v1.10 spec decision.
//
// ## By-Construction Invariant (BC-2.16.004 v1.10 Invariant 6)
//
// All four `.expect()` sites are provably unreachable under invariant-preserving
// execution in single-threaded safe Rust.  Each site is a loud, self-documenting
// tripwire — the correct idiom for this class of guarantee:
//
//   - Lines 555/576: `has_conflict` is derived by `bindings.get(…)`, and
//     `bindings.get_mut(…)` executes in the same `process_arp` invocation with
//     no interleaving opportunity to remove the entry.  Entry is present by
//     construction; `.expect()` cannot fire.
//
//   - Line 642: `emit_d1_spoof_finding_impl` returns, then `bindings.get_mut(…)`
//     is called for the Step 4 MAC update.  No removal can occur between those
//     two statements in single-threaded code.  Entry is present by construction;
//     `.expect()` cannot fire.
//
//   - Line 827: Step 2 sets `first_rebind_ts = Some(timestamp_secs)` before
//     Step 3's `.expect()` runs.  Even after a flap-window reset (which clears
//     the field to `None`), Step 2 immediately re-sets it.  `first_rebind_ts`
//     is always `Some` at the Step 3 `.expect()` site; `.expect()` cannot fire.
//
// ## Role of These Tests
//
// The tests in this module exercise the real production paths (entry always
// present, `first_rebind_ts` always `Some` at the relevant point) and assert
// correct finding counts and severities.  They are regression guards: if a
// future refactor breaks a by-construction invariant and causes a finding to
// be dropped, duplicated, or emitted at the wrong severity, these tests catch
// it before the `.expect()` tripwire would ever need to fire in production.
//
// DF-TEST-NAMESPACE-001: wrapped in `mod bc_2_16_004_inv6`.

#[cfg(test)]
mod bc_2_16_004_inv6 {
    //! BC-2.16.004 v1.10 Invariant 6 — by-construction `.expect()` regression guards.
    //!
    //! EC-011: GARP-conflict path (`has_conflict == true`) — entry is present by
    //!         construction; exercises `.expect()` sites at lines 555 and 576.
    //! EC-012: `emit_d1_spoof_finding_impl` — `first_rebind_ts` is always `Some` at
    //!         Step 3 by construction (Step 2 sets it unconditionally); exercises the
    //!         `.expect()` site at line 827.
    //! Line 642 (non-GARP rebind Step 4 MAC-update re-borrow) — entry is present by
    //!         construction; exercised via the normal non-GARP rebind path.
    //!
    //! All five tests exercise real production paths (the by-construction invariant
    //! always holds) and assert correct finding counts and severities.  They serve as
    //! regression guards: a future refactor that drops a finding, duplicates one, or
    //! changes severity will be caught here before the `.expect()` tripwire fires.

    use super::*;
    use crate::decoder::ArpFrame;
    use crate::findings::Confidence;

    // -----------------------------------------------------------------------
    // Shared constants (match story_114 canonical vectors — BC-2.16.004 table)
    // -----------------------------------------------------------------------

    const IP_A: [u8; 4] = [10, 0, 0, 1];
    const MAC_A: [u8; 6] = [0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA];
    const MAC_B: [u8; 6] = [0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB];
    const MAC_C: [u8; 6] = [0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC];

    /// Normal (non-GARP) ARP Reply — outer_src_mac matches sender_mac to avoid D12.
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

    /// GARP Reply (op=2, sender_ip == target_ip) — outer_src_mac matches sender_mac.
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
    // EC-011: GARP-conflict path — `.expect()` sites at lines 555 and 576
    // -----------------------------------------------------------------------

    /// BC-2.16.004 v1.10 / EC-011 — GARP-conflict path regression guard.
    ///
    /// Exercises the `.expect("has_conflict implies entry exists")` site at line
    /// 555 and `.expect("entry must still exist")` site at line 576.
    ///
    /// ## By-construction invariant (STOP — read before modifying this test)
    ///
    /// Sites 555 and 576 are provably unreachable: `has_conflict` is derived from
    /// `bindings.get(&sender_ip)` and `get_mut()` executes in the same
    /// `process_arp` invocation with no interleaving opportunity to remove the
    /// entry.  The entry is always present when these sites execute; the `.expect()`
    /// calls are loud tripwires, not fail-safes.
    ///
    /// This test verifies that the GARP-conflict path produces the expected 2
    /// findings (D2 MEDIUM + D1 MEDIUM at first rebind) and correct MAC update,
    /// guarding against future regressions in finding count or severity that would
    /// indicate the invariant has been broken.
    ///
    /// Note on test name: the name implies a "missing entry" scenario from the
    /// superseded v1.9 framing.  Under v1.10, the test exercises the production
    /// path where the entry IS present (by construction).  The name is kept for
    /// EC-011 anchor continuity per the spec.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_expect_site_no_panic_on_missing_entry() {
        // Establish initial binding: IP_A → MAC_A (no conflict yet).
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);
        let seed = make_reply(IP_A, MAC_A);
        let seed_findings = analyzer.process_arp(&seed, 1_000);
        // Seed must not emit a D1 spoof finding (first observation → no rebind).
        let seed_spoof_count = seed_findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0830".to_string())
                    || f.summary.to_lowercase().contains("spoof")
                    || f.summary.to_lowercase().contains("rebind")
            })
            .count();
        assert_eq!(
            seed_spoof_count, 0,
            "EC-011 setup: first observation of IP_A must not emit a D1 spoof finding. \
             Got {seed_spoof_count} spoof finding(s).",
        );

        // Verify the binding was created (entry exists before the GARP-conflict frame).
        assert!(
            analyzer.bindings.contains_key(&IP_A),
            "EC-011 setup: binding for IP_A must exist after seeding. \
             bindings.len() = {}",
            analyzer.bindings.len()
        );

        // Send a GARP frame with a DIFFERENT MAC — this triggers the conflict path
        // (`has_conflict == true`) in `process_arp`, exercising `.expect()` at lines
        // 555 (first `get_mut`) and 576 (second `get_mut` for Step 4 MAC update).
        //
        // Both `.expect()` calls succeed because the entry is present by construction
        // (the by-construction invariant holds; these sites are provably unreachable).
        let garp_conflict = make_garp(IP_A, MAC_B);
        let findings = analyzer.process_arp(&garp_conflict, 2_000);

        // BC-2.16.014 / BC-2.16.004: GARP-that-conflicts must emit exactly 2 findings:
        //   (1) D2 GARP upgraded to MEDIUM with T0830/T1557.002
        //   (2) D1 spoof finding (MEDIUM at first rebind, threshold=3)
        let d2_findings: Vec<_> = findings
            .iter()
            .filter(|f| f.summary.to_lowercase().contains("garp"))
            .collect();
        let d1_findings: Vec<_> = findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0830".to_string())
                    && (f.summary.to_lowercase().contains("spoof")
                        || f.summary.to_lowercase().contains("rebind"))
            })
            .collect();

        assert_eq!(
            d2_findings.len(),
            1,
            "EC-011 / BC-2.16.014: GARP-conflict must emit exactly 1 D2 (GARP) finding. \
             Got {} D2 finding(s) in {:?}",
            d2_findings.len(),
            findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
        );
        assert_eq!(
            d1_findings.len(),
            1,
            "EC-011 / BC-2.16.004: GARP-conflict must co-emit exactly 1 D1 (spoof) finding. \
             Got {} D1 finding(s) in {:?}",
            d1_findings.len(),
            findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
        );

        // D2 must be MEDIUM (upgraded from LOW per BC-2.16.014 PC1).
        assert_eq!(
            d2_findings[0].confidence,
            Confidence::Medium,
            "EC-011 / BC-2.16.014: D2 GARP-conflict finding must be MEDIUM. \
             Got {:?}",
            d2_findings[0].confidence
        );

        // D1 must be MEDIUM (first rebind, rebind_count=1 < threshold=3).
        assert_eq!(
            d1_findings[0].confidence,
            Confidence::Medium,
            "EC-011 / BC-2.16.004 PC1.d: D1 co-emitted with GARP-conflict must be \
             MEDIUM (rebind_count=1 < spoof_threshold=3). Got {:?}",
            d1_findings[0].confidence
        );

        // Both D2 and D1 must carry T0830 and T1557.002 (BC-2.16.014 PC1 / BC-2.16.004 PC1.e).
        assert!(
            d2_findings[0]
                .mitre_techniques
                .contains(&"T0830".to_string()),
            "EC-011 / BC-2.16.014: D2 GARP-conflict must carry T0830. \
             Got techniques: {:?}",
            d2_findings[0].mitre_techniques
        );
        assert!(
            d1_findings[0]
                .mitre_techniques
                .contains(&"T1557.002".to_string()),
            "EC-011 / BC-2.16.004 PC1.e: D1 spoof finding must carry T1557.002. \
             Got techniques: {:?}",
            d1_findings[0].mitre_techniques
        );

        // MAC must have been updated (Step 4 ran via line 576 — the second `.expect()` site).
        let entry = analyzer
            .bindings
            .get(&IP_A)
            .expect("EC-011: binding for IP_A must still exist after GARP-conflict processing.");
        assert_eq!(
            entry.mac, MAC_B,
            "EC-011 / BC-2.16.004 PC1.f / line 576: MAC must be updated to MAC_B after \
             GARP-conflict Step 4. Got {:?}",
            entry.mac
        );
    }

    // -----------------------------------------------------------------------
    // EC-011 sibling: GARP-conflict HIGH escalation — both lines 555 and 576
    // -----------------------------------------------------------------------

    /// BC-2.16.004 v1.10 / EC-011 sibling — GARP-conflict HIGH-escalation regression guard.
    ///
    /// With `spoof_threshold=1`, the first rebind immediately escalates to HIGH.
    /// Exercises the Step 4 MAC-update path at line 576 after
    /// `apply_garp_conflict_escalation_impl` sets `spoof_high_emitted = true`.
    /// Both `.expect()` sites (555 and 576) are exercised with the entry present
    /// by construction (invariant holds).  Guards against regressions in D1 HIGH
    /// finding count, severity, and MAC-update correctness on the GARP-conflict path.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_expect_site_no_panic_garp_conflict_high_escalation() {
        // spoof_threshold=1: first rebind immediately escalates to HIGH.
        let mut analyzer = ArpAnalyzer::new(1, ARP_STORM_RATE_DEFAULT);

        // Seed: IP_A → MAC_A (no rebind).
        let seed = make_reply(IP_A, MAC_A);
        let _ = analyzer.process_arp(&seed, 1_000);

        // GARP conflict with MAC_B: rebind_count→1, threshold=1 → HIGH D1 co-emitted.
        // Exercises lines 555 and 576 on the GARP-conflict path.
        let garp_conflict = make_garp(IP_A, MAC_B);
        let findings = analyzer.process_arp(&garp_conflict, 2_000);

        let d1_findings: Vec<_> = findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0830".to_string())
                    && (f.summary.to_lowercase().contains("spoof")
                        || f.summary.to_lowercase().contains("rebind"))
            })
            .collect();

        assert_eq!(
            d1_findings.len(),
            1,
            "EC-011 HIGH path: GARP-conflict with threshold=1 must co-emit exactly 1 D1 \
             spoof finding. Got {} in {:?}",
            d1_findings.len(),
            findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
        );

        // With threshold=1: rebind_count=1 >= threshold=1, elapsed=0 <= 60s → HIGH.
        assert_eq!(
            d1_findings[0].confidence,
            Confidence::High,
            "EC-011 HIGH path / BC-2.16.004 PC1.c: D1 co-emitted with threshold=1 GARP \
             must be HIGH. Got {:?}",
            d1_findings[0].confidence
        );

        // MAC must be updated (Step 4 at line 576 ran).
        let entry = analyzer
            .bindings
            .get(&IP_A)
            .expect("EC-011 HIGH path: binding must exist after GARP-conflict HIGH processing.");
        assert_eq!(
            entry.mac, MAC_B,
            "EC-011 HIGH path / line 576: MAC must be updated to MAC_B after Step 4. \
             Got {:?}",
            entry.mac
        );
    }

    // -----------------------------------------------------------------------
    // EC-012: `emit_d1_spoof_finding_impl` direct call — `.expect()` site at line 827
    // -----------------------------------------------------------------------

    /// BC-2.16.004 v1.10 / EC-012 — `emit_d1_spoof_finding_impl` regression guard
    /// (entry with `first_rebind_ts = None` on entry, set to `Some` by Step 2).
    ///
    /// ## By-construction invariant (STOP — read before modifying this test)
    ///
    /// Site 827 (`entry.first_rebind_ts.expect("set in Step 2")`) is provably
    /// unreachable: Step 2 sets `first_rebind_ts = Some(timestamp_secs)` before
    /// Step 3's `.expect()` runs, regardless of the entry's initial state.  If a
    /// flap-window reset clears the field to `None` earlier in the same call, Step 2
    /// immediately re-sets it.  `first_rebind_ts` is always `Some` at the Step 3
    /// `.expect()` site; the `.expect()` is a loud tripwire, not a fail-safe.
    ///
    /// This test calls `emit_d1_spoof_finding_impl` directly with an entry that has
    /// `first_rebind_ts = None` on entry.  It verifies that Step 2 correctly sets
    /// the field, Step 3 proceeds without panic, and the function returns a valid
    /// MEDIUM D1 finding.  Guards against a future reordering of Steps 2 and 3.
    ///
    /// Note on test name: the name reflects the v1.9 "missing entry" framing.
    /// Under v1.10, `first_rebind_ts` is always `Some` at Step 3 by construction;
    /// the test title is kept for EC-012 anchor continuity per the spec.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_expect_site_no_panic_emit_d1_first_rebind_ts_none() {
        // Construct a BindingEntry where first_rebind_ts = None.
        // This simulates the state at the start of a fresh rebind window.
        let mut entry = BindingEntry {
            mac: MAC_A,
            rebind_count: 0,
            first_rebind_ts: None, // <-- the "missing" field site 827 would expect
            spoof_high_emitted: false,
            last_seen_ts: 1_000,
        };

        // Call emit_d1_spoof_finding_impl directly (private fn, accessible in same file).
        //
        // By construction: Step 2 sets `first_rebind_ts = Some(2_000)` before
        // Step 3's `.expect()` runs → no panic.  The test asserts: (a) no panic;
        // (b) a Finding is returned; (c) the finding is MEDIUM (rebind_count starts
        // at 0, Step 1 → 1, threshold=3 → MEDIUM).
        let finding = ArpAnalyzer::emit_d1_spoof_finding_impl(
            &mut entry,
            IP_A,
            MAC_A, // old_mac
            MAC_B, // new_mac
            2_000, // timestamp_secs
            SPOOF_REBIND_ESCALATION_DEFAULT,
        );

        // A valid Finding must be returned (no panic, no None/Option).
        assert!(
            finding.summary.to_lowercase().contains("spoof")
                || finding.summary.to_lowercase().contains("rebind"),
            "EC-012 / BC-2.16.004 PC1.e: emit_d1_spoof_finding_impl must return a D1 \
             finding summarizing the rebind event. Got summary: {:?}",
            finding.summary
        );

        // Step 1 incremented rebind_count to 1; threshold=3 → MEDIUM (not HIGH).
        assert_eq!(
            finding.confidence,
            Confidence::Medium,
            "EC-012 / BC-2.16.004 PC1.d: first rebind (rebind_count=1 < threshold=3) must \
             produce MEDIUM confidence. Got {:?}",
            finding.confidence
        );

        // first_rebind_ts must have been set by Step 2 (BC-2.16.004 PC1.b).
        assert_eq!(
            entry.first_rebind_ts,
            Some(2_000),
            "EC-012 / BC-2.16.004 PC1.b: Step 2 must set first_rebind_ts = Some(2000). \
             Got {:?}",
            entry.first_rebind_ts
        );

        // MITRE techniques must include T0830 and T1557.002 (BC-2.16.004 PC1.e).
        assert!(
            finding.mitre_techniques.contains(&"T0830".to_string()),
            "EC-012 / BC-2.16.004 PC1.e: D1 finding must carry T0830. \
             Got: {:?}",
            finding.mitre_techniques
        );
        assert!(
            finding.mitre_techniques.contains(&"T1557.002".to_string()),
            "EC-012 / BC-2.16.004 PC1.e: D1 finding must carry T1557.002. \
             Got: {:?}",
            finding.mitre_techniques
        );
    }

    /// BC-2.16.004 v1.10 / EC-012 sibling — flap-window-reset regression guard.
    ///
    /// Calls `emit_d1_spoof_finding_impl` with `first_rebind_ts = Some(very_old_ts)`
    /// that triggers the flap-window RESET (BC-2.16.004 PC5), clearing
    /// `first_rebind_ts` to `None` inside the function before Step 2 re-sets it.
    ///
    /// This is the closest white-box scenario to `first_rebind_ts` being `None`
    /// immediately before Step 2 executes — the reset clears it, Step 2 immediately
    /// sets it again.  Site 827's `.expect()` is provably safe because Step 2 always
    /// runs between the reset and Step 3, maintaining the by-construction invariant.
    ///
    /// Verifies:
    ///   (a) No panic (invariant holds; `.expect()` site is provably unreachable).
    ///   (b) After the reset, `rebind_count` is reset to 0, then incremented to 1 by
    ///       Step 1.
    ///   (c) `first_rebind_ts` is set to the new timestamp by Step 2.
    ///   (d) Finding is MEDIUM (fresh window, rebind_count=1 < threshold=3).
    ///
    /// Guards against regressions in flap-window-reset + Step 2 ordering that would
    /// expose the Step 3 `.expect()` to a `None` value.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_expect_site_no_panic_emit_d1_after_flap_window_reset() {
        // Entry state: prior rebind window started at ts=0, current ts=100 > 60s window.
        let very_old_ts: u32 = 0;
        let current_ts: u32 = 100; // elapsed = 100 > ARP_FLAP_WINDOW_SECS=60 → reset fires

        let mut entry = BindingEntry {
            mac: MAC_A,
            rebind_count: 5,                    // would be > 0 before reset
            first_rebind_ts: Some(very_old_ts), // will be cleared by reset
            spoof_high_emitted: true,           // will be reset to false
            last_seen_ts: 50,
        };

        // Call emit_d1_spoof_finding_impl.  Expected sequence inside:
        //   - Flap-window reset fires (elapsed=100 > 60): rebind_count→0,
        //     first_rebind_ts→None, spoof_high_emitted→false.
        //   - Step 1: rebind_count → 1.
        //   - Step 2: first_rebind_ts is None → set to Some(100).
        //   - Step 3: first_rebind_ts.expect("set in Step 2") → Some(100) → no panic.
        //   - rebind_count=1 < threshold=3 → MEDIUM.
        let finding = ArpAnalyzer::emit_d1_spoof_finding_impl(
            &mut entry,
            IP_A,
            MAC_A, // old_mac
            MAC_C, // new_mac
            current_ts,
            SPOOF_REBIND_ESCALATION_DEFAULT,
        );

        // No panic: function returned successfully.
        assert!(
            finding.summary.to_lowercase().contains("spoof")
                || finding.summary.to_lowercase().contains("rebind"),
            "EC-012 flap-reset path: emit_d1_spoof_finding_impl must return a valid D1 finding \
             after a flap-window reset. Got summary: {:?}",
            finding.summary
        );

        // After reset + Step 1: rebind_count must be 1.
        assert_eq!(
            entry.rebind_count, 1,
            "EC-012 flap-reset path / BC-2.16.004 PC5+PC1.a: after reset, rebind_count \
             must be 1 (reset→0, Step 1→1). Got {}",
            entry.rebind_count
        );

        // Step 2 must have set first_rebind_ts to current_ts.
        assert_eq!(
            entry.first_rebind_ts,
            Some(current_ts),
            "EC-012 flap-reset path / BC-2.16.004 PC1.b: Step 2 must set \
             first_rebind_ts = Some({current_ts}) after reset. Got {:?}",
            entry.first_rebind_ts
        );

        // MEDIUM (fresh window, count=1 < threshold=3).
        assert_eq!(
            finding.confidence,
            Confidence::Medium,
            "EC-012 flap-reset path / BC-2.16.004 PC1.d: post-reset first rebind must \
             be MEDIUM (count=1 < threshold=3). Got {:?}",
            finding.confidence
        );
    }

    // -----------------------------------------------------------------------
    // Site line 642 — non-GARP rebind path Step 4 re-borrow
    // -----------------------------------------------------------------------

    /// BC-2.16.004 v1.10 / line 642 — non-GARP rebind Step 4 MAC-update regression guard.
    ///
    /// Exercises `.expect("entry must still exist")` at line 642 (the second
    /// `bindings.get_mut(&sender_ip)` call for the Step 4 MAC update after
    /// `emit_d1_spoof_finding_impl` returns on the non-GARP rebind path).
    ///
    /// ## By-construction invariant
    ///
    /// Line 642's `.expect()` is provably unreachable: between the first `get_mut`
    /// (line 617, guarded by `if let`) and the second `get_mut` (lines 641-642),
    /// there is no opportunity to remove the entry in single-threaded safe Rust.
    /// The entry is present by construction; the `.expect()` is a loud tripwire.
    ///
    /// This test verifies the full non-GARP rebind path completes successfully:
    ///   (1) Exactly one D1 finding emitted (surrounding code at site 642 ran).
    ///   (2) MAC updated to the new value (Step 4 via line 642 completed).
    ///   (3) `rebind_count` incremented correctly.
    ///
    /// Guards against future regressions in finding count, MAC-update correctness,
    /// or rebind_count state on the non-GARP rebind path.
    #[test]
    #[allow(non_snake_case)]
    fn test_BC_2_16_004_expect_site_no_panic_non_garp_rebind_step4_reborrow() {
        let mut analyzer =
            ArpAnalyzer::new(SPOOF_REBIND_ESCALATION_DEFAULT, ARP_STORM_RATE_DEFAULT);

        // Seed: IP_A → MAC_A (no rebind).
        let seed = make_reply(IP_A, MAC_A);
        let _ = analyzer.process_arp(&seed, 1_000);

        // Rebind: MAC_A → MAC_B.  Exercises:
        //   - `emit_d1_spoof_finding_impl` at line 626 (site via EC-012).
        //   - `bindings.get_mut(&sender_ip).expect(…)` at line 641-642 for MAC update.
        let rebind = make_reply(IP_A, MAC_B);
        let findings = analyzer.process_arp(&rebind, 2_000);

        // Exactly one D1 spoof finding must be emitted.
        let d1_count = findings
            .iter()
            .filter(|f| {
                f.mitre_techniques.contains(&"T0830".to_string())
                    || f.summary.to_lowercase().contains("spoof")
                    || f.summary.to_lowercase().contains("rebind")
            })
            .count();
        assert_eq!(
            d1_count,
            1,
            "line 642 path / BC-2.16.004 PC1: non-GARP rebind must emit exactly 1 D1 \
             spoof finding (including Step 4 re-borrow at line 642). Got {} finding(s) \
             in {:?}",
            d1_count,
            findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
        );

        // MAC must have been updated by Step 4 (line 642 path ran).
        let entry = analyzer
            .bindings
            .get(&IP_A)
            .expect("line 642 path: binding must exist after non-GARP rebind.");
        assert_eq!(
            entry.mac, MAC_B,
            "line 642 path / BC-2.16.004 PC1.f: MAC must be updated to MAC_B after \
             Step 4 MAC-update at line 642. Got {:?}",
            entry.mac
        );

        // rebind_count must be 1 after the first rebind.
        assert_eq!(
            entry.rebind_count, 1,
            "line 642 path / BC-2.16.004 PC1.a: rebind_count must be 1 after first rebind. \
             Got {}",
            entry.rebind_count
        );
    }
} // mod bc_2_16_004_inv6
