//! Content-first stream dispatcher (ADR 0001).
//!
//! Sits between [`crate::reassembly::TcpReassembler`] (which produces
//! contiguous TCP-stream byte ranges) and the per-protocol analyzers
//! ([`HttpAnalyzer`], [`TlsAnalyzer`]). On the first chunk of each flow,
//! peeks at the leading bytes to decide whether the stream is TLS
//! (`0x16 0x03` record-type-and-version prefix) or HTTP (one of the
//! known method tokens) and routes all subsequent data on that flow to
//! the matching analyzer. Streams whose content doesn't match either
//! prefix are tracked under "unclassified" for the JSON summary.
//!
//! Routing is irrevocable per flow — once classified, a flow stays with
//! its analyzer for the rest of its lifetime to avoid mid-stream
//! protocol confusion attacks.

use std::collections::HashMap;

use crate::analyzer::http::HttpAnalyzer;
use crate::analyzer::tls::TlsAnalyzer;
use crate::reassembly::flow::FlowKey;
use crate::reassembly::handler::{CloseReason, Direction, StreamHandler};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DispatchTarget {
    Http,
    Tls,
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
    unclassified_flows: u64,
}

impl StreamDispatcher {
    pub fn new(http: Option<HttpAnalyzer>, tls: Option<TlsAnalyzer>) -> Self {
        StreamDispatcher {
            routes: HashMap::new(),
            classification_attempts: HashMap::new(),
            max_classification_attempts: DEFAULT_MAX_CLASSIFICATION_ATTEMPTS,
            http,
            tls,
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
}

fn classify(data: &[u8], flow_key: &FlowKey) -> DispatchTarget {
    // Content-first detection
    if data.len() >= 5 && data[0] == 0x16 && data[1] == 0x03 {
        return DispatchTarget::Tls;
    }
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
    // Port fallback for short data
    let ports = [flow_key.lower_port(), flow_key.upper_port()];
    if ports.contains(&443) || ports.contains(&8443) {
        return DispatchTarget::Tls;
    }
    if ports.contains(&80) || ports.contains(&8080) {
        return DispatchTarget::Http;
    }
    DispatchTarget::None
}

impl StreamHandler for StreamDispatcher {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64) {
        if self.http.is_none() && self.tls.is_none() {
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
                    http.on_data(flow_key, direction, data, offset);
                }
            }
            DispatchTarget::Tls => {
                if let Some(ref mut tls) = self.tls {
                    tls.on_data(flow_key, direction, data, offset);
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
            Some(DispatchTarget::None) | None => {
                if self.http.is_some() || self.tls.is_some() {
                    self.unclassified_flows += 1;
                }
            }
        }
    }
}
