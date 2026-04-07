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

pub struct StreamDispatcher {
    routes: HashMap<FlowKey, DispatchTarget>,
    pub http: Option<HttpAnalyzer>,
    pub tls: Option<TlsAnalyzer>,
}

impl StreamDispatcher {
    pub fn new(http: Option<HttpAnalyzer>, tls: Option<TlsAnalyzer>) -> Self {
        StreamDispatcher {
            routes: HashMap::new(),
            http,
            tls,
        }
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
    let ports = [flow_key.lower_port, flow_key.upper_port];
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

        // Don't cache None — allow reclassification on next on_data with more bytes
        let target = if let Some(&cached) = self.routes.get(flow_key) {
            cached
        } else {
            let target = classify(data, flow_key);
            if target != DispatchTarget::None {
                self.routes.insert(flow_key.clone(), target);
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
            _ => {}
        }
    }
}
