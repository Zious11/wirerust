use std::net::IpAddr;

use anyhow::{Result, anyhow};
use etherparse::SlicedPacket;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Other(u8),
}

#[derive(Debug, Clone)]
pub enum TransportInfo {
    Tcp {
        src_port: u16,
        dst_port: u16,
        seq_number: u32,
        syn: bool,
        ack: bool,
        fin: bool,
        rst: bool,
    },
    Udp {
        src_port: u16,
        dst_port: u16,
    },
    None,
}

#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub protocol: Protocol,
    pub transport: TransportInfo,
    pub payload: Vec<u8>,
    pub packet_len: usize,
}

impl ParsedPacket {
    /// Guess the application-layer protocol based on port numbers.
    pub fn app_protocol_hint(&self) -> Option<&'static str> {
        let dst = match &self.transport {
            TransportInfo::Tcp { dst_port, .. } => *dst_port,
            TransportInfo::Udp { dst_port, .. } => *dst_port,
            TransportInfo::None => return None,
        };
        let src = match &self.transport {
            TransportInfo::Tcp { src_port, .. } => *src_port,
            TransportInfo::Udp { src_port, .. } => *src_port,
            TransportInfo::None => return None,
        };

        match (src, dst) {
            (53, _) | (_, 53) => Some("DNS"),
            (80, _) | (_, 80) => Some("HTTP"),
            (443, _) | (_, 443) => Some("TLS"),
            (22, _) | (_, 22) => Some("SSH"),
            (445, _) | (_, 445) => Some("SMB"),
            (502, _) | (_, 502) => Some("Modbus"),
            (20000, _) | (_, 20000) => Some("DNP3"),
            _ => None,
        }
    }
}

pub fn decode_packet(data: &[u8]) -> Result<ParsedPacket> {
    let sliced = SlicedPacket::from_ethernet(data).map_err(|e| anyhow!("Parse error: {e}"))?;

    let (src_ip, dst_ip, ip_protocol) = match &sliced.net {
        Some(etherparse::NetSlice::Ipv4(ipv4)) => {
            let header = ipv4.header();
            (
                IpAddr::V4(header.source_addr()),
                IpAddr::V4(header.destination_addr()),
                header.protocol(),
            )
        }
        Some(etherparse::NetSlice::Ipv6(ipv6)) => {
            let header = ipv6.header();
            (
                IpAddr::V6(header.source_addr()),
                IpAddr::V6(header.destination_addr()),
                ipv6.payload().ip_number,
            )
        }
        None => return Err(anyhow!("No IP layer found")),
    };

    let (protocol, transport) = match &sliced.transport {
        Some(etherparse::TransportSlice::Tcp(tcp)) => (
            Protocol::Tcp,
            TransportInfo::Tcp {
                src_port: tcp.source_port(),
                dst_port: tcp.destination_port(),
                seq_number: tcp.to_header().sequence_number,
                syn: tcp.syn(),
                ack: tcp.ack(),
                fin: tcp.fin(),
                rst: tcp.rst(),
            },
        ),
        Some(etherparse::TransportSlice::Udp(udp)) => (
            Protocol::Udp,
            TransportInfo::Udp {
                src_port: udp.source_port(),
                dst_port: udp.destination_port(),
            },
        ),
        Some(etherparse::TransportSlice::Icmpv4(_) | etherparse::TransportSlice::Icmpv6(_)) => {
            (Protocol::Icmp, TransportInfo::None)
        }
        None => (Protocol::Other(ip_protocol.0), TransportInfo::None),
    };

    let payload = match &sliced.transport {
        Some(etherparse::TransportSlice::Tcp(tcp)) => tcp.payload().to_vec(),
        Some(etherparse::TransportSlice::Udp(udp)) => udp.payload().to_vec(),
        _ => Vec::new(),
    };

    Ok(ParsedPacket {
        src_ip,
        dst_ip,
        protocol,
        transport,
        payload,
        packet_len: data.len(),
    })
}
