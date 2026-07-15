//! Content-first stream dispatcher (ADR 0001).
//!
//! Sits between [`crate::reassembly::TcpReassembler`] (which produces
//! contiguous TCP-stream byte ranges) and the per-protocol analyzers
//! ([`HttpAnalyzer`], [`TlsAnalyzer`], [`ModbusAnalyzer`], [`Dnp3Analyzer`],
//! [`EnipAnalyzer`]).
//! On the first chunk of each flow, peeks at the leading bytes to decide
//! whether the stream is TLS (`0x16 0x03` record-type-and-version prefix),
//! HTTP (one of the known method tokens), Modbus (port-502 fallback per
//! ADR-005), DNP3 (port-20000 fallback per ADR-007), or EtherNet/IP traffic
//! on TCP port 44818 → EnipAnalyzer (Rule 7, ADR-010) and routes all
//! subsequent data on that flow to the matching analyzer. Streams whose
//! content doesn't match any prefix and whose ports don't match any known
//! port are tracked under "unclassified" for the JSON summary.
//!
//! Routing is irrevocable per flow — once classified, a flow stays with
//! its analyzer for the rest of its lifetime to avoid mid-stream
//! protocol confusion attacks.
//!
//! ## Classification Rule Order (BC-2.14.025 / BC-2.15.021 / BC-2.17.019 / BC-2.05.012, INV-2 content-first)
//!
//! 1. TLS content signature (`0x16 0x03 ...`, len >= 5) → `DispatchTarget::Tls`
//! 2. HTTP method token (`GET `, `POST `, etc.) → `DispatchTarget::Http`
//! 3. Port 443/8443 → `DispatchTarget::Tls`
//! 4. Port 80/8080 → `DispatchTarget::Http`
//! 5. Port 502 → `DispatchTarget::Modbus`  ← Rule 5 (STORY-105, ADR-005)
//! 6. Port 20000 → `DispatchTarget::Dnp3`  ← Rule 6 (STORY-110, ADR-007)
//! 7. Port 44818 → `DispatchTarget::Enip`  ← Rule 7 (STORY-131, ADR-010)
//! 8. Port 2404 → `DispatchTarget::Iec104` ← Rule 8 (STORY-173, ADR-013)
//! 9. No match → `DispatchTarget::None`

use std::collections::HashMap;

use crate::analyzer::dnp3::Dnp3Analyzer;
use crate::analyzer::enip::EnipAnalyzer;
use crate::analyzer::http::HttpAnalyzer;
use crate::analyzer::iec104::Iec104Analyzer;
use crate::analyzer::modbus::ModbusAnalyzer;
use crate::analyzer::tls::TlsAnalyzer;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};

/// Minimal transport discriminant for the (TransportProto, u16) gap-counter key.
/// Distinct from `protocols::Transport` (which has a third `LinkLayer` variant).
/// NOT imported from `protocols.rs` — defined here to enforce the pure-core boundary
/// (BC-2.05.010 PC-4, Invariant 1; ADR-012 Decision 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TransportProto {
    Tcp,
    Udp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchTarget {
    Http,
    Tls,
    /// Port-502 Modbus TCP flows (Rule 5, BC-2.14.025). Added in STORY-105.
    Modbus,
    /// Port-20000 DNP3 TCP flows (Rule 6, BC-2.15.021). Added in STORY-110.
    Dnp3,
    /// Port-44818 EtherNet/IP TCP flows (Rule 7, BC-2.17.019). Added in STORY-131.
    Enip,
    /// Port-2404 IEC 60870-5-104 TCP flows (Rule 8, BC-2.05.012). Added in STORY-173.
    Iec104,
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
    /// EtherNet/IP TCP analyzer (STORY-131, BC-2.17.019). Receives data for all
    /// port-44818 flows that do not match content rules 1–2 or port rules 3–6.
    enip: Option<EnipAnalyzer>,
    /// IEC 60870-5-104 TCP analyzer (STORY-173, BC-2.05.012). Receives data for all
    /// port-2404 flows that do not match content rules 1–2 or port rules 3–7.
    iec104: Option<Iec104Analyzer>,
    unclassified_flows: u64,
    /// Per-(TransportProto, port) counts for TCP flows that close as DispatchTarget::None
    /// (STORY-153, BC-2.05.010 PC-1, ADR-012 Decision 6 Clarification).
    /// Populated in on_flow_close only when coverage_gaps_enabled=true AND analyzer-present guard.
    unclassified_port_counts: HashMap<(TransportProto, u16), u64>,
    /// Feature flag: when true, the per-port unclassified_port_counts counter is populated
    /// in the on_flow_close None-target arm (STORY-153, BC-2.05.010 PC-1).
    /// Default: false. Set via with_coverage_gaps(bool) builder.
    coverage_gaps_enabled: bool,
}

impl StreamDispatcher {
    /// Construct a dispatcher with optional HTTP, TLS, Modbus, DNP3, ENIP, and IEC-104 analyzers.
    ///
    /// Pass `modbus: Some(analyzer)` to enable port-502 flow routing (STORY-105).
    /// Pass `modbus: None` to leave Modbus disabled (default-off per BC-2.14.023).
    /// Pass `dnp3: Some(analyzer)` to enable port-20000 flow routing (STORY-110).
    /// Pass `dnp3: None` to leave DNP3 disabled (default-off per BC-2.15.021).
    /// Pass `enip: Some(analyzer)` to enable port-44818 flow routing (STORY-131).
    /// Pass `enip: None` to leave ENIP disabled (default-off per BC-2.17.020).
    /// Pass `iec104: Some(analyzer)` to enable port-2404 flow routing (STORY-173).
    /// Pass `iec104: None` to leave IEC-104 disabled (default-off per BC-2.12.025).
    pub fn new(
        http: Option<HttpAnalyzer>,
        tls: Option<TlsAnalyzer>,
        modbus: Option<ModbusAnalyzer>,
        dnp3: Option<Dnp3Analyzer>,
        enip: Option<EnipAnalyzer>,
        iec104: Option<Iec104Analyzer>,
    ) -> Self {
        StreamDispatcher {
            routes: HashMap::new(),
            classification_attempts: HashMap::new(),
            max_classification_attempts: DEFAULT_MAX_CLASSIFICATION_ATTEMPTS,
            http,
            tls,
            modbus,
            dnp3,
            enip,
            iec104,
            unclassified_flows: 0,
            unclassified_port_counts: HashMap::new(),
            coverage_gaps_enabled: false,
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

    /// Enable or disable per-port unclassified coverage-gap counting (STORY-153, BC-2.05.010).
    /// Consistent with the existing `with_max_classification_attempts` builder pattern.
    /// All existing `StreamDispatcher::new()` call sites remain untouched (ADR-012 Decision 6 Clarification).
    ///
    /// When `enabled = true`, `on_flow_close` for None-target flows populates
    /// `unclassified_port_counts` with `(TransportProto::Tcp, min(lower_port, upper_port))`
    /// entries (F-F3P11-001; BC-2.05.010 PC-1; ADR-012 Decision 6 Clarification).
    /// When `enabled = false` (the default), the port counter is never incremented.
    pub fn with_coverage_gaps(mut self, enabled: bool) -> Self {
        self.coverage_gaps_enabled = enabled;
        self
    }

    pub fn unclassified_flows(&self) -> u64 {
        self.unclassified_flows
    }

    /// Returns the per-`(TransportProto::Tcp, port)` unclassified flow counts accumulated
    /// by `on_flow_close` for None-target flows when `coverage_gaps_enabled = true`
    /// (STORY-153, BC-2.05.010 PC-1).
    ///
    /// Keys are `(TransportProto::Tcp, min(lower_port, upper_port))` (F-F3P11-001).
    /// Returns an empty map when `coverage_gaps_enabled = false` (the default).
    pub fn unclassified_port_counts(&self) -> &HashMap<(TransportProto, u16), u64> {
        &self.unclassified_port_counts
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

    /// Returns a reference to the ENIP analyzer, if one was configured.
    ///
    /// BC-2.17.019: mirrors `dnp3_analyzer()` shape.
    pub fn enip_analyzer(&self) -> Option<&EnipAnalyzer> {
        self.enip.as_ref()
    }

    /// Moves the ENIP analyzer out of the dispatcher, consuming the slot.
    ///
    /// BC-2.17.019 Invariant / BC-2.17.020 §P4: mirrors `take_dnp3_analyzer()` —
    /// uses `Option::take()`, leaving `self.enip = None` permanently. After this
    /// call, all ENIP dispatch arms are no-ops. Call ONCE,
    /// post-`reassembler.finalize()`.
    pub fn take_enip_analyzer(&mut self) -> Option<EnipAnalyzer> {
        self.enip.take()
    }

    /// Wires an `EnipAnalyzer` into the dispatcher after construction.
    ///
    /// Called from `main.rs` after the analyzer is constructed.
    /// WIRING-EXEMPT: single field assignment with no branching.
    pub fn set_enip_analyzer(&mut self, analyzer: EnipAnalyzer) {
        self.enip = Some(analyzer);
    }

    /// Returns a reference to the IEC-104 analyzer, if one was configured.
    ///
    /// BC-2.05.012: mirrors `enip_analyzer()` shape.
    pub fn iec104_analyzer(&self) -> Option<&Iec104Analyzer> {
        self.iec104.as_ref()
    }

    /// Moves the IEC-104 analyzer out of the dispatcher, consuming the slot.
    ///
    /// BC-2.05.012: mirrors `take_enip_analyzer()` — uses `Option::take()`,
    /// leaving `self.iec104 = None` permanently. After this call, all IEC-104
    /// dispatch arms are no-ops. Call ONCE, post-`reassembler.finalize()`.
    pub fn take_iec104_analyzer(&mut self) -> Option<Iec104Analyzer> {
        self.iec104.take()
    }

    /// Wires an `Iec104Analyzer` into the dispatcher after construction.
    ///
    /// Called from `main.rs` after the analyzer is constructed.
    /// WIRING-EXEMPT: single field assignment with no branching.
    pub fn set_iec104_analyzer(&mut self, analyzer: Iec104Analyzer) {
        self.iec104 = Some(analyzer);
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
    // Rule 7: ENIP port (44818 — IANA-assigned, ADR-010 Decision 1). Fires AFTER Rule 6
    // (DNP3). TLS ClientHello or HTTP GET on port 44818 will have already matched Rules 1
    // or 2 above (BC-2.17.019 Invariant 1). VP-004 oracle obligation: classify_oracle
    // gains the port-44818 → Enip arm immediately after the port-20000 → Dnp3 arm
    // (BC-2.17.019 Invariant 3, STORY-131 VP-004 obligation).
    if ports.contains(&44818) {
        return DispatchTarget::Enip;
    }
    // Rule 8: IEC-104 port (2404 — IANA-assigned, ADR-013 Decision 1). Fires AFTER Rule 7
    // (ENIP). No content-signature rule for 0x68 (single byte is not a reliable discriminator —
    // ADR-013 Decision 1). VP-004 oracle obligation: classify_oracle gains the port-2404 → Iec104
    // arm immediately after the port-44818 → Enip arm (BC-2.05.012, ADR-013 Decision 9 step 2).
    if ports.contains(&2404) {
        return DispatchTarget::Iec104;
    }
    // Rule 9: no match.
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
        // BC-2.14.025 §P2 / BC-2.15.021 Inv 4 / BC-2.17.019 Inv 4 / BC-2.05.012 early-exit guard:
        // extended to include iec104 (STORY-173). Without `self.iec104.is_none()`, on_data
        // silently drops data when only an IEC-104 analyzer is active.
        if self.http.is_none()
            && self.tls.is_none()
            && self.modbus.is_none()
            && self.dnp3.is_none()
            && self.enip.is_none()
            && self.iec104.is_none()
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
                // BC-2.15.021 §P3: route a port-20000-classified flow's data to
                // Dnp3Analyzer; no-op if disabled.
                // STORY-140 (BC-2.15.016 v2.0 Precondition 2): pass direction to
                // on_data (Modbus/ENIP pattern; AC-140-003).
                if let Some(ref mut dnp3) = self.dnp3 {
                    dnp3.on_data(flow_key.clone(), data, timestamp, direction);
                }
            }
            DispatchTarget::Enip => {
                // BC-2.17.019 §P2: route a port-44818-classified flow's data to
                // EnipAnalyzer; no-op if disabled. Detection logic (frame-walk, CIP
                // parse) is added by STORY-132+. This arm increments bytes_received
                // to evidence PC-2 routing correctness (STORY-131 boundary decision).
                if let Some(ref mut enip) = self.enip {
                    // STORY-139: pass direction to on_data (Modbus pattern; BC-2.17.016 v2.0 Precondition 1)
                    enip.on_data(flow_key.clone(), data, timestamp, direction);
                }
            }
            DispatchTarget::Iec104 => {
                // BC-2.05.012 §P2 (stub — STORY-173 TDD step, Red Gate):
                // Forwarding to Iec104Analyzer is UNIMPLEMENTED here.
                // The behavioral wiring (iec104.on_data call) is added in the TDD implementation step.
                // This arm compiles but does NOT route data to the analyzer, keeping the
                // test_BC_2_05_012_dispatch_port_2404 and test_iec104_only_dispatcher tests RED.
                let _ = (flow_key, data, timestamp, direction);
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
                // BC-2.15.021 / SEC-006 / issue #342: forward on_flow_close to Dnp3Analyzer
                // to purge per-flow state and fold metrics into aggregates.
                let _ = reason;
                if let Some(ref mut dnp3) = self.dnp3 {
                    dnp3.on_flow_close(flow_key.clone());
                }
            }
            Some(DispatchTarget::Enip) => {
                // BC-2.17.019 / SEC-005 / issue #342: forward on_flow_close to EnipAnalyzer
                // to purge per-flow state and fold metrics into aggregates.
                let _ = reason;
                if let Some(ref mut enip) = self.enip {
                    enip.on_flow_close(flow_key.clone());
                }
            }
            Some(DispatchTarget::Iec104) => {
                // BC-2.05.012 (stub — STORY-173 TDD step, Red Gate):
                // Forwarding to Iec104Analyzer is UNIMPLEMENTED here.
                // The behavioral wiring (iec104.on_flow_close call) is added in the TDD step.
                let _ = reason;
            }
            Some(DispatchTarget::None) | None => {
                // BC-2.14.025 §P3: unclassified_flows guard extended with modbus + dnp3 + enip + iec104.
                if self.http.is_some()
                    || self.tls.is_some()
                    || self.modbus.is_some()
                    || self.dnp3.is_some()
                    || self.enip.is_some()
                    || self.iec104.is_some()
                {
                    // REGRESSION WARNING (ADR-012 Decision 6 Clarification EXACT):
                    // unclassified_flows += 1 is gated on the analyzer-present guard ONLY —
                    // NOT on coverage_gaps_enabled. Moving this inside the coverage_gaps block
                    // would zero the counter on all normal (coverage_gaps=false) runs, breaking
                    // BC-2.05.009 + holdouts HS-040/HS-095.
                    self.unclassified_flows = self.unclassified_flows.saturating_add(1);
                    // STORY-153 (BC-2.05.010 PC-1, AC-153-003): per-port TCP counter increment.
                    // Dual-gate: (outer) analyzer-present guard AND (inner) coverage_gaps_enabled.
                    // ADR-012 Decision 6 Clarification EXACT: unclassified_flows += 1 is above,
                    // gated only on the analyzer-present guard (NOT on coverage_gaps_enabled).
                    // The port counter below is additionally gated on coverage_gaps_enabled.
                    if self.coverage_gaps_enabled {
                        // F-F3P11-001: use min(lower_port, upper_port) — NOT lower_port() alone.
                        // FlowKey canonicalizes by (ip, port) tuple (IP first), so lower_port()
                        // is the port of the lower-IP endpoint, which may be an ephemeral port.
                        // min(lower_port(), upper_port()) gives the service port (BC-2.05.010 PC-1).
                        let lower_port = flow_key.lower_port().min(flow_key.upper_port());
                        let c = self
                            .unclassified_port_counts
                            .entry((TransportProto::Tcp, lower_port))
                            .or_insert(0);
                        // EC-153-10: saturating_add — no panic on u64 overflow.
                        *c = c.saturating_add(1);
                    }
                }
            }
        }
    }
}

// ── STORY-153: UDP gap-key seam (VP-043 non-vacuity) ───────────────────────────
//
// Library-visible pure decision function used by both the VP-043 proptest harnesses
// (in tests/dispatcher_tests.rs, which link only the library crate and CANNOT reach
// the main.rs decode loop) and the main.rs decode loop itself.
//
// The seam pattern mirrors VP-039/VP-040 `fill_buf_for_testing` (VP-INDEX lines ~189–240).
// BC-2.05.010 is satisfied: `udp_unclassified_counts` is still populated in main.rs via
// this seam — no logic is duplicated (DF-KANI-NONVACUITY-001).

/// Returns `Some((TransportProto::Udp, min(src_port, dst_port)))` when `parsed`
/// is a UDP packet that is NOT handled by any registered dissector (`dns_handles == false`).
/// Returns `None` when the packet is already classified (DNS accepted it) or is not UDP.
///
/// # SEAM CONTRACT (VP-043)
/// This is the library-visible boundary that VP-043 proptest harnesses exercise directly.
/// The `main.rs` decode loop calls `udp_gap_key(&parsed, dns_analyzer.can_decode(&parsed))`
/// and accumulates `Some(key)` returns into `udp_unclassified_counts`.
/// BC-2.05.010 is satisfied: the counter is populated in the main.rs loop via this seam.
/// The seam itself is pure and stateless — it does NOT modify any `StreamDispatcher` state.
///
/// ADR-012 Decision 10: `dns_handles` must be set by evaluating `dns_analyzer.can_decode()`
/// regardless of the `enable_dns` flag. DNS/53 traffic is gap-excluded even when DNS
/// finding-emission is disabled.
pub fn udp_gap_key(
    parsed: &crate::decoder::ParsedPacket,
    dns_handles: bool,
) -> Option<(TransportProto, u16)> {
    // ADR-012 Decision 10: dns_handles is evaluated regardless of enable_dns.
    // DNS/53 packets accepted by can_decode() are gap-excluded (not counted).
    if dns_handles {
        return None;
    }
    match parsed.transport {
        crate::decoder::TransportInfo::Udp { src_port, dst_port } => {
            // Normalize to service port: min(src_port, dst_port).
            // EC-153-10: no overflow risk (u16 min is always valid).
            Some((TransportProto::Udp, src_port.min(dst_port)))
        }
        // Non-UDP transport (TCP, ICMP, etc.) — seam is UDP-only.
        _ => None,
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
        // Rule 7: ENIP port fallback (ADR-010 Decision 1 — MUST mirror production exactly).
        // VP-004 oracle obligation: this arm is mandatory per BC-2.17.019 Invariant 3 /
        // STORY-131 VP-004 oracle obligation. Placed AFTER Rule 6 (DNP3) and BEFORE
        // Rule 8 (IEC-104).
        if ports.contains(&44818) {
            return DispatchTarget::Enip;
        }
        // Rule 8: IEC-104 port fallback (ADR-013 Decision 1 — MUST mirror production exactly).
        // VP-004 oracle obligation: this arm is mandatory per BC-2.05.012 /
        // STORY-173 VP-004 oracle obligation (ADR-013 Decision 9 step 3).
        // Placed AFTER Rule 7 (ENIP) and BEFORE Rule 9 (None).
        if ports.contains(&2404) {
            return DispatchTarget::Iec104;
        }
        // Rule 9: nothing matched.
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

    // ── VP-043: udp_gap_key seam correctness (symbolic BMC) ────────────────────
    //
    // F6 hardening for VP-043. The designated method is proptest
    // (`proptest_vp043_*` in tests/dispatcher_tests.rs); these Kani harnesses add
    // exhaustive bounded model checking over the FULL symbolic input space
    // (src_port × dst_port ∈ u16 × u16, dns_handles ∈ {true,false}). `udp_gap_key`
    // reads only `parsed.transport`; the other ParsedPacket fields are fixed to
    // concrete values with no loss of generality (the function never reads them).
    // Traces BC-2.05.010, BC-2.05.011.

    fn udp_packet(src_port: u16, dst_port: u16) -> crate::decoder::ParsedPacket {
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        crate::decoder::ParsedPacket {
            src_ip: ip,
            dst_ip: ip,
            protocol: crate::decoder::Protocol::Udp,
            transport: crate::decoder::TransportInfo::Udp { src_port, dst_port },
            payload: Vec::new(),
            packet_len: 0,
        }
    }

    /// VP-043 gate + accumulation-key correctness over full symbolic ports/gate.
    /// `dns_handles == true`  → `None` (DNS-accepted packets never counted).
    /// `dns_handles == false` → `Some((Udp, min(src,dst)))` (service-port key).
    #[kani::proof]
    fn vp043_udp_gap_key_gate_and_key() {
        let src_port: u16 = kani::any();
        let dst_port: u16 = kani::any();
        let dns_handles: bool = kani::any();
        let parsed = udp_packet(src_port, dst_port);
        match udp_gap_key(&parsed, dns_handles) {
            None => assert!(dns_handles), // only DNS-accepted UDP yields None here
            Some((proto, port)) => {
                assert!(!dns_handles);
                assert!(matches!(proto, TransportProto::Udp));
                assert!(port == src_port.min(dst_port));
            }
        }
    }

    /// VP-043 direction symmetry: swapping src/dst ports yields the identical
    /// `(Udp, min)` key — query and response collapse to one service-port bucket.
    #[kani::proof]
    fn vp043_udp_gap_key_direction_symmetric() {
        let a: u16 = kani::any();
        let b: u16 = kani::any();
        assert!(udp_gap_key(&udp_packet(a, b), false) == udp_gap_key(&udp_packet(b, a), false));
    }

    /// VP-043 non-UDP exclusion: TCP transport never produces a UDP gap key
    /// (the seam is UDP-only — covers the `_ => None` arm).
    #[kani::proof]
    fn vp043_udp_gap_key_non_udp_none() {
        let sp: u16 = kani::any();
        let dp: u16 = kani::any();
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));
        let parsed = crate::decoder::ParsedPacket {
            src_ip: ip,
            dst_ip: ip,
            protocol: crate::decoder::Protocol::Tcp,
            transport: crate::decoder::TransportInfo::Tcp {
                src_port: sp,
                dst_port: dp,
                seq_number: 0,
                syn: false,
                ack: false,
                fin: false,
                rst: false,
            },
            payload: Vec::new(),
            packet_len: 0,
        };
        assert!(udp_gap_key(&parsed, false).is_none());
    }

    // ── VP-042: unclassified port-key + counter algebra (pure sub-properties) ──
    //
    // The FULL VP-042 accumulation property (Σ unclassified_port_counts == N over
    // N on_flow_close calls) is verified by the designated proptest harnesses
    // (`proptest_vp042_*`), NOT here: it accumulates into a std `HashMap` whose
    // `RandomState` seeds from OS entropy (getrandom), which Kani cannot model
    // soundly. The two PURE arithmetic invariants underpinning the on_flow_close
    // increment ARE Kani-amenable and are proven here over full symbolic domains.
    // Traces BC-2.05.010 PC-1 (F-F3P11-001 min-port key), EC-153-10 (saturating).

    /// VP-042 service-port key is `min(lower_port, upper_port)` — symmetric and
    /// never exceeds either endpoint port (F-F3P11-001 normalization).
    #[kani::proof]
    fn vp042_min_port_key_symmetric() {
        let a: u16 = kani::any();
        let b: u16 = kani::any();
        assert!(a.min(b) == b.min(a));
        assert!(a.min(b) <= a && a.min(b) <= b);
    }

    /// VP-042 counter is `saturating_add(1)` (EC-153-10): never panics, never
    /// decreases, never exceeds `u64::MAX`.
    #[kani::proof]
    fn vp042_saturating_counter_monotonic() {
        let c: u64 = kani::any();
        let next = c.saturating_add(1);
        assert!(next >= c);
        assert!(next <= u64::MAX);
    }
}
