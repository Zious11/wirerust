use std::collections::BTreeMap;
use std::net::IpAddr;

use crate::reassembly::handler::Direction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FlowKey {
    lower_ip: IpAddr,
    lower_port: u16,
    upper_ip: IpAddr,
    upper_port: u16,
}

impl FlowKey {
    pub fn lower_ip(&self) -> IpAddr {
        self.lower_ip
    }

    pub fn lower_port(&self) -> u16 {
        self.lower_port
    }

    pub fn upper_ip(&self) -> IpAddr {
        self.upper_ip
    }

    pub fn upper_port(&self) -> u16 {
        self.upper_port
    }

    pub fn new(ip_a: IpAddr, port_a: u16, ip_b: IpAddr, port_b: u16) -> Self {
        // Canonicalize by (ip, port) tuple comparison — keeps IP+port paired together.
        // This is critical: sorting independently would merge different connections.
        if (ip_a, port_a) <= (ip_b, port_b) {
            FlowKey {
                lower_ip: ip_a,
                lower_port: port_a,
                upper_ip: ip_b,
                upper_port: port_b,
            }
        } else {
            FlowKey {
                lower_ip: ip_b,
                lower_port: port_b,
                upper_ip: ip_a,
                upper_port: port_a,
            }
        }
    }
}

impl std::fmt::Display for FlowKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}:{} → {}:{}",
            self.lower_ip, self.lower_port, self.upper_ip, self.upper_port
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    New,
    SynSent,
    Established,
    Closing,
    Closed,
}

#[derive(Debug)]
pub struct FlowDirection {
    pub isn: Option<u32>,
    pub base_offset: u64,
    pub(super) segments: BTreeMap<u64, Vec<u8>>,
    pub(super) buffered_bytes: usize,
    pub reassembled_bytes: usize,
    pub overlap_count: u32,
    pub overlap_alert_fired: bool,
    pub small_segment_count: u32,
    pub small_segment_alert_fired: bool,
    pub fin_seen: bool,
    pub rst_seen: bool,
    pub depth_exceeded: bool,
}

impl Default for FlowDirection {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowDirection {
    pub fn new() -> Self {
        FlowDirection {
            isn: None,
            base_offset: 0,
            segments: BTreeMap::new(),
            buffered_bytes: 0,
            reassembled_bytes: 0,
            overlap_count: 0,
            overlap_alert_fired: false,
            small_segment_count: 0,
            small_segment_alert_fired: false,
            fin_seen: false,
            rst_seen: false,
            depth_exceeded: false,
        }
    }

    pub fn set_isn(&mut self, isn: u32) {
        if self.isn.is_none() {
            self.isn = Some(isn);
            self.base_offset = 1; // ISN+1 is first data byte
        }
    }

    pub fn infer_isn(&mut self, first_seq: u32) {
        if self.isn.is_none() {
            self.isn = Some(first_seq.wrapping_sub(1));
            self.base_offset = 1;
        }
    }

    pub fn segment_count(&self) -> usize {
        self.segments.len()
    }

    pub fn buffered_bytes(&self) -> usize {
        self.buffered_bytes
    }

    pub fn segments_is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn segment_at(&self, offset: u64) -> Option<&[u8]> {
        self.segments.get(&offset).map(|v| v.as_slice())
    }

    pub fn has_segment_at(&self, offset: u64) -> bool {
        self.segments.contains_key(&offset)
    }

    pub fn memory_used(&self) -> usize {
        debug_assert_eq!(
            self.buffered_bytes,
            self.segments.values().map(|v| v.len()).sum::<usize>(),
            "buffered_bytes counter drifted from actual segment sizes"
        );
        self.buffered_bytes
    }
}

#[derive(Debug)]
pub struct TcpFlow {
    pub key: FlowKey,
    pub client_to_server: FlowDirection,
    pub server_to_client: FlowDirection,
    pub state: FlowState,
    pub partial: bool,
    pub first_seen: u32,
    pub last_seen: u32,
    initiator: Option<(IpAddr, u16)>,
    fin_count: u8,
}

impl TcpFlow {
    pub fn new(key: FlowKey, timestamp: u32) -> Self {
        TcpFlow {
            key,
            client_to_server: FlowDirection::new(),
            server_to_client: FlowDirection::new(),
            state: FlowState::New,
            partial: false,
            first_seen: timestamp,
            last_seen: timestamp,
            initiator: None,
            fin_count: 0,
        }
    }

    pub fn set_initiator(&mut self, ip: IpAddr, port: u16) {
        if self.initiator.is_none() {
            self.initiator = Some((ip, port));
        }
    }

    pub fn direction(&self, src_ip: IpAddr, src_port: u16) -> Direction {
        if self.initiator == Some((src_ip, src_port)) {
            Direction::ClientToServer
        } else {
            Direction::ServerToClient
        }
    }

    pub fn get_direction_mut(&mut self, dir: Direction) -> &mut FlowDirection {
        match dir {
            Direction::ClientToServer => &mut self.client_to_server,
            Direction::ServerToClient => &mut self.server_to_client,
        }
    }

    pub fn on_syn(&mut self) {
        if self.state == FlowState::New {
            self.state = FlowState::SynSent;
        }
    }

    pub fn on_syn_ack(&mut self) {
        if self.state == FlowState::SynSent || self.state == FlowState::New {
            self.state = FlowState::Established;
        }
    }

    pub fn on_data_without_syn(&mut self) {
        if self.state == FlowState::New {
            self.state = FlowState::Established;
            self.partial = true;
        }
    }

    pub fn on_fin(&mut self) {
        self.fin_count = self.fin_count.saturating_add(1);
        if self.fin_count >= 2 {
            self.state = FlowState::Closed;
        } else if self.state == FlowState::Established || self.state == FlowState::SynSent {
            self.state = FlowState::Closing;
        }
    }

    pub fn on_rst(&mut self) {
        self.state = FlowState::Closed;
    }

    pub fn memory_used(&self) -> usize {
        self.client_to_server.memory_used() + self.server_to_client.memory_used()
    }
}
