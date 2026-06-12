//! Content-first stream dispatcher (ADR 0001).
//!
//! Sits between [`crate::reassembly::TcpReassembler`] (which produces
//! contiguous TCP-stream byte ranges) and the per-protocol analyzers
//! ([`HttpAnalyzer`], [`TlsAnalyzer`], [`ModbusAnalyzer`], [`Dnp3Analyzer`]).
//! On the first chunk of each flow, peeks at the leading bytes to decide
//! whether the stream is TLS (`0x16 0x03` record-type-and-version prefix),
//! HTTP (one of the known method tokens), Modbus (port-502 fallback per
//! ADR-005), or DNP3 (port-20000 fallback per ADR-007) and routes all
//! subsequent data on that flow to the matching analyzer. Streams whose
//! content doesn't match any prefix and whose ports don't match any known
//! port are tracked under "unclassified" for the JSON summary.
//!
//! Routing is irrevocable per flow — once classified, a flow stays with
//! its analyzer for the rest of its lifetime to avoid mid-stream
//! protocol confusion attacks.
//!
//! ## Classification Rule Order (BC-2.14.025 / BC-2.15.021, INV-2 content-first)
//!
//! 1. TLS content signature (`0x16 0x03 ...`, len >= 5) → `DispatchTarget::Tls`
//! 2. HTTP method token (`GET `, `POST `, etc.) → `DispatchTarget::Http`
//! 3. Port 443/8443 → `DispatchTarget::Tls`
//! 4. Port 80/8080 → `DispatchTarget::Http`
//! 5. Port 502 → `DispatchTarget::Modbus`  ← Rule 5 (STORY-105, ADR-005)
//! 6. Port 20000 → `DispatchTarget::Dnp3`  ← Rule 6 (STORY-110, ADR-007)
//! 7. No match → `DispatchTarget::None`

use std::collections::HashMap;

use crate::analyzer::dnp3::Dnp3Analyzer;
use crate::analyzer::http::HttpAnalyzer;
use crate::analyzer::modbus::ModbusAnalyzer;
use crate::analyzer::tls::TlsAnalyzer;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchTarget {
    Http,
    Tls,
    /// Port-502 Modbus TCP flows (Rule 5, BC-2.14.025). Added in STORY-105.
    Modbus,
    /// Port-20000 DNP3 TCP flows (Rule 6, BC-2.15.021). Added in STORY-110.
    Dnp3,
    None,
}

/// Default upper bound on classification retries per flow before it
/// is permanently stamped as [`DispatchTarget::None`].
///
/// Picked empirically: a single TCP segment from a long-running TLS or
/// HTTP connection always reveals its protocol in the first 1–2 chunks,
/// and any stream that still hasn't matched after 8 chunks is almost
/// certainly a non-HTTP, non-TLS protocol (SSH, custom binary,
/// encrypted-but-not-TLS) — re-running [`classify`] on every subsequent
/// segment is wasted work and inflates CPU on long-lived flows. See
/// LESSON-P2.11 (`max_classification_attempts` knob).
pub const DEFAULT_MAX_CLASSIFICATION_ATTEMPTS: u32 = 8;

pub struct StreamDispatcher {
    routes: HashMap<FlowKey, DispatchTarget>,
    /// Number of times [`classify`] has returned [`DispatchTarget::None`]
    /// for a given flow. Once a flow's count reaches
    /// `max_classification_attempts`, the dispatcher inserts
    /// `DispatchTarget::None` into `routes` and stops re-classifying.
    classification_attempts: HashMap<FlowKey, u32>,
    /// Hard cap on classification retries per flow. LESSON-P2.11.
    max_classification_attempts: u32,
    http: Option<HttpAnalyzer>,
    tls: Option<TlsAnalyzer>,
    /// Modbus TCP analyzer (STORY-105, BC-2.14.025). Receives data for all
    /// port-502 flows that do not match content rules 1–2 or port rules 3–4.
    modbus: Option<ModbusAnalyzer>,
    /// DNP3 TCP analyzer (STORY-110, BC-2.15.021). Receives data for all
    /// port-20000 flows that do not match content rules 1–2 or port rules 3–5.
    dnp3: Option<Dnp3Analyzer>,
    unclassified_flows: u64,
}

impl StreamDispatcher {
    /// Construct a dispatcher with optional HTTP, TLS, Modbus, and DNP3 analyzers.
    ///
    /// Pass `modbus: Some(analyzer)` to enable port-502 flow routing (STORY-105).
    /// Pass `modbus: None` to leave Modbus disabled (default-off per BC-2.14.023).
    /// Pass `dnp3: Some(analyzer)` to enable port-20000 flow routing (STORY-110).
    /// Pass `dnp3: None` to leave DNP3 disabled (default-off per BC-2.15.021).
    pub fn new(
        http: Option<HttpAnalyzer>,
        tls: Option<TlsAnalyzer>,
        modbus: Option<ModbusAnalyzer>,
        dnp3: Option<Dnp3Analyzer>,
    ) -> Self {
        StreamDispatcher {
            routes: HashMap::new(),
            classification_attempts: HashMap::new(),
            max_classification_attempts: DEFAULT_MAX_CLASSIFICATION_ATTEMPTS,
            http,
            tls,
            modbus,
            dnp3,
            unclassified_flows: 0,
        }
    }

    /// Override the per-flow classification-retry cap. Useful for
    /// tests that need to exercise the give-up branch with small
    /// inputs, or for callers that need to widen the cap to
    /// accommodate unusual mid-stream-join captures.
    ///
    /// A value of `0` effectively disables classification entirely
    /// (every flow becomes `DispatchTarget::None` on the first chunk).
    pub fn with_max_classification_attempts(mut self, max_attempts: u32) -> Self {
        self.max_classification_attempts = max_attempts;
        self
    }

    pub fn unclassified_flows(&self) -> u64 {
        self.unclassified_flows
    }

    /// Returns the configured per-flow classification-retry cap.
    pub fn max_classification_attempts(&self) -> u32 {
        self.max_classification_attempts
    }

    /// Returns a reference to the HTTP analyzer, if one was configured.
    pub fn http_analyzer(&self) -> Option<&HttpAnalyzer> {
        self.http.as_ref()
    }

    /// Returns a reference to the TLS analyzer, if one was configured.
    pub fn tls_analyzer(&self) -> Option<&TlsAnalyzer> {
        self.tls.as_ref()
    }

    /// Returns a reference to the Modbus analyzer, if one was configured.
    ///
    /// BC-2.14.025 §P4: mirrors `tls_analyzer()` shape.
    pub fn modbus_analyzer(&self) -> Option<&ModbusAnalyzer> {
        self.modbus.as_ref()
    }

    /// Moves the TLS analyzer out of the dispatcher, consuming the slot.
    ///
    /// Intended for callers that need ownership of the analyzer after
    /// processing is complete (e.g., to collect results after the capture
    /// loop finishes).
    ///
    /// After this call the internal slot is permanently `None`. Any subsequent
    /// [`StreamHandler::on_data`] calls will no longer route data to the TLS
    /// analyzer — there is no re-insertion path. Only call this once the
    /// capture loop has finished.
    pub fn take_tls_analyzer(&mut self) -> Option<TlsAnalyzer> {
        self.tls.take()
    }

    /// Moves the Modbus analyzer out of the dispatcher, consuming the slot.
    ///
    /// BC-2.14.025 §P4: mirrors `take_tls_analyzer()` — uses `Option::take()`,
    /// leaving `self.modbus = None` permanently. After this call, all Modbus
    /// dispatch arms are no-ops. Call ONCE, post-`reassembler.finalize()`.
    pub fn take_modbus_analyzer(&mut self) -> Option<ModbusAnalyzer> {
        self.modbus.take()
    }

    /// Returns a reference to the DNP3 analyzer, if one was configured.
    ///
    /// BC-2.15.021: mirrors `modbus_analyzer()` shape.
    pub fn dnp3_analyzer(&self) -> Option<&Dnp3Analyzer> {
        self.dnp3.as_ref()
    }

    /// Moves the DNP3 analyzer out of the dispatcher, consuming the slot.
    ///
    /// BC-2.15.021 Invariant 5: mirrors `take_modbus_analyzer()` — uses
    /// `Option::take()`, leaving `self.dnp3 = None` permanently. After this
    /// call, all DNP3 dispatch arms are no-ops. Call ONCE,
    /// post-`reassembler.finalize()`.
    pub fn take_dnp3_analyzer(&mut self) -> Option<Dnp3Analyzer> {
        self.dnp3.take()
    }
}

fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget {
    // Rule 1 (content: TLS): TLS record header signature.
    if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
        return DispatchTarget::Tls;
    }
    // Rule 2 (content: HTTP): HTTP method token prefix.
    if data.starts_with(b"GET ")
        || data.starts_with(b"POST ")
        || data.starts_with(b"PUT ")
        || data.starts_with(b"DELETE ")
        || data.starts_with(b"HEAD ")
        || data.starts_with(b"OPTIONS ")
        || data.starts_with(b"PATCH ")
        || data.starts_with(b"CONNECT ")
        || data.starts_with(b"TRACE ")
        || data.starts_with(b"HTTP/")
    {
        return DispatchTarget::Http;
    }
    // Port fallback rules (checked after content rules — BC-2.14.025 INV-2).
    let ports = [flow_key.lower_port(), flow_key.upper_port()];
    // Rule 3: TLS port fallback (443/8443).
    if ports.contains(&443) || ports.contains(&8443) {
        return DispatchTarget::Tls;
    }
    // Rule 4: HTTP port fallback (80/8080).
    if ports.contains(&80) || ports.contains(&8080) {
        return DispatchTarget::Http;
    }
    // Rule 5: Modbus port (502 — IANA-assigned, ADR-005). Fires AFTER all
    // content rules and TLS/HTTP port fallbacks. TLS ClientHello or HTTP GET
    // on port 502 will have already matched Rules 1 or 2 above (BC-2.14.025).
    //
    // Gemini MEDIUM investigation (STORY-105 adversarial review, accepted):
    // A TLS/HTTP flow on port 502 whose FIRST on_data chunk is < 5 bytes will
    // reach Rule 5 and be committed to Modbus before content rules 1-2 can
    // evaluate. This is CONSISTENT with the behavior of port rules 3-4: a flow
    // on port 443 (Rule 3) or port 80 (Rule 4) with a tiny first chunk is
    // similarly committed to TLS/HTTP before content is inspectable.
    // The classification-retry mechanism (max_classification_attempts / None-caching)
    // applies ONLY to the DispatchTarget::None path — it is not a defer-until-content
    // mechanism for successful classifications. Port-fallback rules commit
    // irrevocably on first presentation, uniformly across all three protocols.
    // Verdict: ACCEPTED — no defect, no code change required.
    if ports.contains(&502) {
        return DispatchTarget::Modbus;
    }
    // Rule 6: DNP3 port (20000 — IANA-assigned, ADR-007 Decision 1). Fires AFTER all
    // content rules and TLS/HTTP/Modbus port fallbacks. TLS ClientHello or HTTP GET
    // on port 20000 will have already matched Rules 1 or 2 above (BC-2.15.021 INV-2).
    // VP-004 oracle obligation: classify_oracle in #[cfg(kani)] mod kani_proofs has the
    // identical arm at the identical position (BC-2.15.021 Invariant 3, STORY-110 AC-005,
    // same-commit requirement per ADR-007 Decision 1).
    if ports.contains(&20000) {
        return DispatchTarget::Dnp3;
    }
    // Rule 7: no match.
    DispatchTarget::None
}

impl StreamHandler for StreamDispatcher {
    fn on_data(
        &mut self,
        flow_key: &FlowKey,
        direction: Direction,
        data: &[u8],
        offset: u64,
        timestamp: u32,
    ) {
        // BC-2.14.025 §P2 / BC-2.15.021 Inv 4 early-exit guard: extended to include dnp3.
        // Without `self.dnp3.is_none()`, on_data silently drops data when only a DNP3
        // analyzer is active (AC-003 of STORY-110).
        if self.http.is_none() && self.tls.is_none() && self.modbus.is_none() && self.dnp3.is_none()
        {
            return;
        }

        // Classification cache + retry-budget enforcement (LESSON-P2.11):
        //   - If the flow is already in `routes`, use the cached target
        //     (covers both successful classifications AND flows that
        //     hit the retry cap and were stamped `None`).
        //   - Otherwise run [`classify`]; on success cache the result;
        //     on failure increment the attempt count and, if we've hit
        //     `max_classification_attempts`, cache `None` so future
        //     chunks short-circuit the work.
        let target = if let Some(&cached) = self.routes.get(flow_key) {
            cached
        } else {
            let target = classify(data, flow_key);
            if target == DispatchTarget::None {
                let count = self
                    .classification_attempts
                    .entry(flow_key.clone())
                    .or_insert(0);
                *count = count.saturating_add(1);
                if *count >= self.max_classification_attempts {
                    // Give up: persistently route to `None` so we
                    // stop calling `classify` on every chunk.
                    self.routes.insert(flow_key.clone(), DispatchTarget::None);
                    self.classification_attempts.remove(flow_key);
                }
            } else {
                self.routes.insert(flow_key.clone(), target);
                self.classification_attempts.remove(flow_key);
            }
            target
        };

        match target {
            DispatchTarget::Http => {
                if let Some(ref mut http) = self.http {
                    http.on_data(flow_key, direction, data, offset, timestamp);
                }
            }
            DispatchTarget::Tls => {
                if let Some(ref mut tls) = self.tls {
                    tls.on_data(flow_key, direction, data, offset, timestamp);
                }
            }
            DispatchTarget::Modbus => {
                // BC-2.14.025 §P2: route to ModbusAnalyzer; no-op if disabled.
                if let Some(ref mut modbus) = self.modbus {
                    modbus.on_data(flow_key, direction, data, offset, timestamp);
                }
            }
            DispatchTarget::Dnp3 => {
                // BC-2.15.021 §P3: route to Dnp3Analyzer; no-op if disabled.
                // STORY-110 stub: wiring of direction/offset/timestamp into
                // Dnp3Analyzer::on_data is the implementer's TDD task.
                if let Some(ref mut dnp3) = self.dnp3 {
                    dnp3.on_data(flow_key.clone(), data, timestamp);
                }
            }
            DispatchTarget::None => {}
        }
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
        // Clean up both the routing cache and the retry-attempt
        // counter (LESSON-P2.11) so closing a flow returns the
        // dispatcher to its pre-classification state for that key.
        self.classification_attempts.remove(flow_key);
        let target = self.routes.remove(flow_key);
        match target {
            Some(DispatchTarget::Http) => {
                if let Some(ref mut http) = self.http {
                    http.on_flow_close(flow_key, reason);
                }
            }
            Some(DispatchTarget::Tls) => {
                if let Some(ref mut tls) = self.tls {
                    tls.on_flow_close(flow_key, reason);
                }
            }
            Some(DispatchTarget::Modbus) => {
                // BC-2.14.025 §P3: route on_flow_close to ModbusAnalyzer.
                if let Some(ref mut modbus) = self.modbus {
                    modbus.on_flow_close(flow_key, reason);
                }
            }
            Some(DispatchTarget::Dnp3) => {
                // BC-2.15.021: route on_flow_close to Dnp3Analyzer (no-op if disabled).
                // Dnp3Analyzer does not implement StreamHandler; no forwarding needed.
                let _ = reason;
            }
            Some(DispatchTarget::None) | None => {
                // BC-2.14.025 §P3: unclassified_flows guard extended with modbus + dnp3.
                if self.http.is_some()
                    || self.tls.is_some()
                    || self.modbus.is_some()
                    || self.dnp3.is_some()
                {
                    self.unclassified_flows += 1;
                }
            }
        }
    }
}

// ── VP-004: Content-First Dispatch Precedence (Kani proofs) ────────────────────
//
// Formal verification of the `classify` precedence rules and the two-phase
// `DispatchTarget::None` caching behavior (LESSON-P2.11). These harnesses are
// strictly `#[cfg(kani)]`-gated: they are invisible to the normal build,
// `cargo test`, and clippy. They are exercised only under `cargo kani`, which
// auto-provides the `kani` crate.
//
// Source of truth: `classify` (this file, ~line 155) and `on_data` (~line 185).
#[cfg(kani)]
mod kani_proofs {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    /// VP-004 rule 1: a TLS record-header signature (`0x16 0x03 ...` with
    /// `len >= 5`) routes to TLS *regardless of port number*. We pin the
    /// flow key's ports to the HTTP fallback ports (80, 8080) to demonstrate
    /// that content wins over the port-fallback rule that would otherwise
    /// select HTTP.
    ///
    /// BOUND/SOUNDNESS: `data` is a symbolic 5-byte array. The signature check
    /// in `classify` reads only `data.len() >= 5 && data[0] && data[1]`; the
    /// remaining 3 bytes (`data[2..5]`) are irrelevant to the rule-1 branch, so
    /// a 5-byte array fully covers the precondition with no loss of generality.
    /// Ports 80/9000 (canonicalized: lower=80) are the strongest adversarial
    /// case for "content beats port".
    #[kani::proof]
    fn verify_tls_signature_beats_port() {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let key = FlowKey::new(ip, 80, ip, 9000); // lower_port == 80 (HTTP fallback)
        let b2: u8 = kani::any();
        let b3: u8 = kani::any();
        let b4: u8 = kani::any();
        let data: [u8; 5] = [0x16, 0x03, b2, b3, b4];
        assert!(matches!(classify(&data, &key), DispatchTarget::Tls));
    }

    /// VP-004 full precedence ladder, exhaustive over a symbolic 8-byte prefix
    /// and fully symbolic 16-bit ports. Re-derives the spec's expected target
    /// independently of `classify`'s internal branch wiring and asserts
    /// equality, so this proves the *entire* decision function (rules 1–5),
    /// not just the TLS-beats-port corollary.
    ///
    /// BOUND/SOUNDNESS:
    ///  - `data` is a symbolic `[u8; 8]` (CR-004). 8 bytes is the length of the
    ///    longest discriminating method token — `"OPTIONS "` and `"CONNECT "`
    ///    are exactly 8 bytes — so EVERY method token in `classify`
    ///    (`GET `, `POST `, `PUT `, `DELETE `, `HEAD `, `OPTIONS `, `PATCH `,
    ///    `CONNECT `, `TRACE `, `HTTP/`) is now fully matchable by the symbolic
    ///    input, closing the gap left by the earlier 5-byte bound (which could
    ///    not realize DELETE/OPTIONS/PATCH/CONNECT/TRACE). The reference oracle
    ///    replicates the EXACT same `starts_with` set so production and oracle
    ///    agree on every input with no divergence.
    ///  - Ports are fully symbolic `u16` (all 65536 values each), so the
    ///    443/8443/80/8080 fallback arms, the port-502 Modbus arm, and the `None`
    ///    arm are all covered.
    ///  - Rule 5 (port 502 → Modbus) is added to the oracle, mirroring production
    ///    exactly per BC-2.14.025 §P5 (critical: oracle MUST mirror production).
    fn classify_oracle(data: &[u8; 8], lower: u16, upper: u16) -> DispatchTarget {
        // Rule 1: TLS content signature.
        if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
            return DispatchTarget::Tls;
        }
        // Rule 2: HTTP method token (identical set/order to production).
        if data.starts_with(b"GET ")
            || data.starts_with(b"POST ")
            || data.starts_with(b"PUT ")
            || data.starts_with(b"DELETE ")
            || data.starts_with(b"HEAD ")
            || data.starts_with(b"OPTIONS ")
            || data.starts_with(b"PATCH ")
            || data.starts_with(b"CONNECT ")
            || data.starts_with(b"TRACE ")
            || data.starts_with(b"HTTP/")
        {
            return DispatchTarget::Http;
        }
        // Rule 3: port fallback (TLS ports take precedence over HTTP ports,
        // matching production's branch ordering).
        let ports = [lower, upper];
        if ports.contains(&443) || ports.contains(&8443) {
            return DispatchTarget::Tls;
        }
        if ports.contains(&80) || ports.contains(&8080) {
            return DispatchTarget::Http;
        }
        // Rule 5: Modbus port fallback (ADR-005 — MUST mirror production exactly).
        // Placed AFTER Rules 3–4 and BEFORE Rule 6 (BC-2.14.025 §P5).
        if ports.contains(&502) {
            return DispatchTarget::Modbus;
        }
        // Rule 6: DNP3 port fallback (ADR-007 Decision 1 — MUST mirror production exactly).
        // VP-004 oracle obligation: this arm is mandatory per BC-2.15.021 Invariant 3 /
        // STORY-110 AC-005. Placed AFTER Rule 5 and BEFORE Rule 7 (None).
        if ports.contains(&20000) {
            return DispatchTarget::Dnp3;
        }
        // Rule 7: nothing matched.
        DispatchTarget::None
    }

    #[kani::proof]
    fn verify_content_first_precedence_exhaustive() {
        let port_a: u16 = kani::any();
        let port_b: u16 = kani::any();
        // IPs are irrelevant to `classify` (it reads only ports). Fix them so
        // canonicalization is driven purely by the symbolic ports.
        let ip = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
        let key = FlowKey::new(ip, port_a, ip, port_b);

        let b0: u8 = kani::any();
        let b1: u8 = kani::any();
        let b2: u8 = kani::any();
        let b3: u8 = kani::any();
        let b4: u8 = kani::any();
        let b5: u8 = kani::any();
        let b6: u8 = kani::any();
        let b7: u8 = kani::any();
        let data: [u8; 8] = [b0, b1, b2, b3, b4, b5, b6, b7];

        let got = classify(&data, &key);
        let want = classify_oracle(&data, key.lower_port(), key.upper_port());
        assert!(got == want);

        // Spell out the headline corollary explicitly for readability:
        // a TLS signature always wins, never mind the port.
        if data[0] == 0x16 && data[1] == 0x03 {
            assert!(matches!(got, DispatchTarget::Tls));
        }
    }

    /// Single-flow-key model of `on_data`'s cache/counter state machine
    /// (this file, the `else` branch of the `routes.get` check). It mirrors the
    /// production transitions on lines ~185–202 EXACTLY for the rule-6 `None`
    /// path, but on a single `(route, attempts)` pair instead of the two
    /// `HashMap<FlowKey, _>`s.
    ///
    /// WHY MODELLED, NOT DRIVEN THROUGH `on_data`: the real dispatcher keys its
    /// state on `HashMap<FlowKey, _>`. `std::collections::HashMap`'s default
    /// `RandomState` seeds itself via the OS RNG (`CCRandomGenerateBytes` on
    /// macOS), a foreign C function Kani cannot symbolically execute — driving
    /// `on_data` therefore aborts with a Kani-unsupported-FFI error, NOT a
    /// property failure. (Confirmed empirically before switching to this model.)
    /// Per-key, the HashMap is just "presence + value"; an `Option` captures the
    /// identical semantics — `entry().or_insert(0); *c = c.saturating_add(1)`
    /// becomes `*attempts.get_or_insert(0) = ...`, `routes.insert` becomes
    /// `route = Some(..)`, `remove` becomes `= None`, `contains_key` becomes
    /// `.is_some()`. This is the same faithful-restatement tactic VP-005 uses
    /// for tls-parser. The transition source below is a line-for-line port.
    fn step_none_path(
        route: &mut Option<DispatchTarget>,
        attempts: &mut Option<u32>,
        max: u32,
    ) -> DispatchTarget {
        // Precondition of this model: `classify` returned `None` (rule-6 path).
        // Cached route short-circuits (mirrors `if let Some(&cached) = routes.get`).
        if let Some(cached) = *route {
            return cached;
        }
        let target = DispatchTarget::None; // classify(...) == None on this path
        // target == None branch of on_data:
        let count = attempts.get_or_insert(0);
        *count = count.saturating_add(1);
        if *count >= max {
            *route = Some(DispatchTarget::None); // routes.insert(key, None)
            *attempts = None; // classification_attempts.remove(key)
        }
        target
    }

    /// VP-004 two-phase `None`-caching (LESSON-P2.11), proven for the ENTIRE
    /// production-relevant cap range via a SYMBOLIC `cap` (CR-002). For each call
    /// `i` (1-based) on the rule-6 `None` path:
    ///   Phase A (i < cap): attempts -> Some(i), route stays uncached (`None`).
    ///   Phase B (i == cap): route = Some(None) permanently, attempts cleared.
    ///   Phase C (i > cap): cached `None` short-circuits — route frozen at
    ///                      Some(None), attempts stays cleared (no re-classify).
    ///
    /// BOUND/SOUNDNESS:
    ///  - `cap` is SYMBOLIC over `1..=DEFAULT_MAX_CLASSIFICATION_ATTEMPTS` (the
    ///    full configurable range; default is 8). `cap == 0` is excluded because
    ///    `with_max_classification_attempts(0)` is documented as a degenerate
    ///    "disable classification" mode that caches `None` on the first call —
    ///    a separate behavior, not the multi-phase retry property under test.
    ///  - The loop runs a FIXED `DEFAULT_MAX_CLASSIFICATION_ATTEMPTS + 1` (= 9)
    ///    iterations regardless of `cap`, so it always observes at least one
    ///    post-cap (phase C) call for every cap in range. `#[kani::unwind(11)]`
    ///    fully unrolls it. Within the loop each phase is checked against the
    ///    symbolic `cap`, so the proof covers cap = 1, 2, ..., 8 simultaneously.
    ///  - The model `step_none_path` is a line-for-line port of `on_data`'s rule-6
    ///    branch (see doc above); the only abstraction is HashMap-by-key -> Option,
    ///    exact for a single key. The companion proofs prove WHEN `classify`
    ///    returns `None`; this proves what the cache/counter then do.
    #[kani::proof]
    #[kani::unwind(11)]
    fn verify_none_two_phase_caching() {
        let cap: u32 = kani::any();
        kani::assume(cap >= 1 && cap <= DEFAULT_MAX_CLASSIFICATION_ATTEMPTS);

        let mut route: Option<DispatchTarget> = None;
        let mut attempts: Option<u32> = None;

        // Drive one extra call beyond the maximum possible cap so every cap in
        // range exercises phases A, B, and C.
        for i in 1..=(DEFAULT_MAX_CLASSIFICATION_ATTEMPTS + 1) {
            let t = step_none_path(&mut route, &mut attempts, cap);
            assert!(matches!(t, DispatchTarget::None)); // always None on rule-6 path

            if i < cap {
                // Phase A: under cap — not cached, counter == i.
                assert!(route.is_none());
                assert!(attempts == Some(i));
            } else {
                // Phase B (i == cap) and Phase C (i > cap): cached permanently,
                // counter cleared and never re-created.
                assert!(matches!(route, Some(DispatchTarget::None)));
                assert!(attempts.is_none());
            }
        }
    }
}
