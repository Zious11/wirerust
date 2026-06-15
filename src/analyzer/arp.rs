//! ARP security analyzer stub — STORY-112.
//!
//! This module defines [`ArpAnalyzer`], a stateful ARP-frame processor that is
//! instantiated once per `run_analyze()` call and receives every [`ArpFrame`]
//! that `decode_packet` routes as `DecodedFrame::Arp`. In STORY-112 the full
//! detection logic (binding table, GARP, D11 malformed, D12 mismatch, storm
//! counters) is NOT yet implemented; `process_arp` is a no-op that returns
//! `vec![]`. Full implementation lands in STORY-113 (binding table + GARP +
//! D11/D12) and STORY-114 (spoof escalation + MITRE) and STORY-115 (D3 storm).
//!
//! ## Forbidden dependency
//!
//! This module MUST NOT import `crate::dispatcher`. `ArpAnalyzer` does not
//! implement `ProtocolAnalyzer` or `StreamAnalyzer` and never goes through the
//! `StreamDispatcher` (arp-architecture-delta.md §1; AC-010 / BC-2.16.015
//! Invariant 2; Forbidden Dependencies in STORY-112).

use crate::decoder::ArpFrame;
use crate::findings::Finding;

/// Stub ARP security analyzer.
///
/// Receives [`ArpFrame`] values from `main.rs` after `decode_packet` produces
/// `DecodedFrame::Arp`. In STORY-112 this is a no-op stub; the real detection
/// logic (binding table, GARP, D11, D12, storm counters) is added in
/// STORY-113..115. The stub compiles and the `process_arp` no-op is wired in
/// `main.rs` unconditionally per BC-2.16.015 postconditions 5/6 (AC-008).
///
/// `--arp` flag-gating is added in STORY-113 (BC-2.16.011).
/// `--arp-spoof-threshold` is added in STORY-114.
/// `--arp-storm-rate` is added in STORY-115.
/// `summarize()` is added in STORY-113.
pub struct ArpAnalyzer {
    // STORY-113 adds: bindings, storm_counters, spoof_threshold, storm_rate, etc.
    // Stub has no fields — parameterless new() per AC-010.
}

impl ArpAnalyzer {
    /// Construct a parameterless stub `ArpAnalyzer`.
    ///
    /// In STORY-112 the constructor takes no arguments because `--arp-spoof-threshold`
    /// (STORY-114) and `--arp-storm-rate` (STORY-115) CLI flags do not yet exist.
    /// STORY-113 will add the full binding-table state fields and STORY-114/115 will
    /// add the threshold parameters to this constructor signature.
    ///
    /// GREEN-BY-DESIGN: zero branching, no I/O, no helpers, 1 line. The constructor
    /// simply creates an empty struct; no implementer work is required for this body.
    pub fn new() -> ArpAnalyzer {
        ArpAnalyzer {}
    }

    /// Process a decoded ARP frame — no-op stub returning `vec![]`.
    ///
    /// This is the **final STORY-112 deliverable form** for `process_arp`: the
    /// method signature is locked (it must match this exact signature in STORY-113
    /// when real logic replaces the no-op body). The no-op body returning `vec![]`
    /// is the correct STORY-112 deliverable per AC-008/AC-010 and BC-2.16.015
    /// postconditions 5/6 — the wiring in `main.rs` is verified to call this and
    /// receive an empty findings vec.
    ///
    /// STORY-113 replaces this body with real detection logic (binding table update,
    /// GARP detection, D11 malformed finding emission, D12 mismatch detection).
    ///
    /// GREEN-BY-DESIGN: returns `vec![]` — zero branching, no I/O, no helpers,
    /// 1 line. Tests asserting `vec![]` will pass immediately; this is intentional
    /// because the no-op is itself the specified STORY-112 behavior. The real-logic
    /// ACs (AC-001..AC-007, AC-012) test `extract_arp_frame` and `decode_packet`,
    /// which remain as None-returning placeholders, keeping those tests RED.
    pub fn process_arp(&mut self, _frame: &ArpFrame, _timestamp_secs: u32) -> Vec<Finding> {
        vec![]
    }
}

impl Default for ArpAnalyzer {
    /// GREEN-BY-DESIGN: delegates to `new()` — zero branching, no I/O, no helpers,
    /// 1 line. Required so `#[derive]` users and clippy `new_without_default` don't
    /// complain.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(kani)]
mod kani_proofs {
    use super::*;

    // Sub-property B: GARP detection totality (BC-2.16.003)
    // Harness skeleton — STORY-113 implements is_gratuitous_arp; bodies filled in by
    // formal-verifier at F6 gate. Bodies are todo!() per BC-5.38.001.
    // This block is only compiled under cargo kani; it is invisible to cargo check /
    // cargo test on the stable toolchain (Cargo.toml registers cfg(kani) as expected).

    #[kani::proof]
    fn verify_classify_garp_total() {
        todo!("STORY-113 implements is_gratuitous_arp — harness body filled at F6 gate")
    }

    // Sub-property D: MAX_ARP_BINDINGS cap (BC-2.16.006)
    // Harness skeleton — STORY-113 implements insert_binding_lru_btree; body filled at F6.
    #[kani::proof]
    #[kani::unwind(12)]
    fn verify_binding_table_cap() {
        todo!("STORY-113 implements insert_binding_lru_btree — harness body filled at F6 gate")
    }
}
