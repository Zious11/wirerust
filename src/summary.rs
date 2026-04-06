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
