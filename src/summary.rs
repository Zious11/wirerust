//! Capture-level rollup totals.
//!
//! [`Summary`] aggregates packet count, byte count, decode-skipped packets,
//! the set of unique source/destination IPs, the protocol-count map, and
//! the inferred service distribution (port-tuple-based, see
//! [`crate::decoder::ParsedPacket::app_protocol_hint`]). Both reporters
//! ([`crate::reporter::terminal`] and [`crate::reporter::json`]) consume
//! this type unchanged; the terminal reporter's per-host breakdown
//! (LESSON-P1.03) reads from [`Summary::unique_hosts`].

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use serde::Serialize;

use crate::decoder::{ParsedPacket, Protocol};

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_packets: u64,
    pub total_bytes: u64,
    pub skipped_packets: u64,
    hosts: HashSet<IpAddr>,
    protocols: HashMap<Protocol, u64>,
    /// Inferred service distribution, keyed by service name.
    ///
    /// LESSON-P3.01 — known divergence: this map is **port-based**. It
    /// is built from [`ParsedPacket::app_protocol_hint`], which infers a
    /// service purely from the TCP/UDP port tuple (53→DNS, 80→HTTP,
    /// 443→TLS, 22→SSH, ...). The stream dispatcher (ADR 0001) is, by
    /// contrast, **content-first**: it identifies a protocol from the
    /// reassembled bytes regardless of port. The two can therefore
    /// disagree — e.g. HTTP served on port 8080 contributes nothing
    /// here (8080 is not a known port hint) yet is dispatched as HTTP
    /// by content. Treat `services` as a cheap port-based triage
    /// estimate, not an authoritative classification.
    services: HashMap<String, u64>,
}

impl Default for Summary {
    fn default() -> Self {
        Self::new()
    }
}

impl Summary {
    pub fn new() -> Self {
        Summary {
            total_packets: 0,
            total_bytes: 0,
            skipped_packets: 0,
            hosts: HashSet::new(),
            protocols: HashMap::new(),
            services: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, packet: &ParsedPacket) {
        self.total_packets += 1;
        self.total_bytes += packet.packet_len as u64;
        self.hosts.insert(packet.src_ip);
        self.hosts.insert(packet.dst_ip);
        *self.protocols.entry(packet.protocol).or_insert(0) += 1;

        if let Some(svc) = packet.app_protocol_hint() {
            *self.services.entry(svc.to_string()).or_insert(0) += 1;
        }
    }

    pub fn unique_hosts(&self) -> Vec<IpAddr> {
        let mut hosts: Vec<_> = self.hosts.iter().copied().collect();
        hosts.sort();
        hosts
    }

    pub fn protocol_counts(&self) -> &HashMap<Protocol, u64> {
        &self.protocols
    }

    pub fn service_counts(&self) -> &HashMap<String, u64> {
        &self.services
    }
}
