//! Link-layer / IP / transport decoder for raw pcap frames.
//!
//! Turns a single captured frame plus its [`pcap_file::DataLink`] into a
//! [`ParsedPacket`] containing source / destination IP, transport-layer
//! information ([`TransportInfo`]), and the payload slice. Application-layer
//! parsing is NOT done here — that is the analyzer / dispatcher pipeline's
//! responsibility.
//!
//! Currently supports Ethernet and Linux-cooked-capture (SLL) link layers
//! via `etherparse::SlicedPacket`. IPv4 and IPv6 are both handled; TCP and
//! UDP transports surface their port / flag tuple, ICMP is reported as the
//! parent protocol with no transport detail, and everything else is reported
//! as `Protocol::Other(proto_num)` with no transport info.
//!
//! ## Snaplen-truncated captures
//!
//! Each frame is parsed strictly first ([`etherparse::SlicedPacket`]), which
//! validates the IPv4 `total_length` / IPv6 `payload_length` header fields
//! against the bytes actually captured. A `tcpdump -s` capture truncates
//! packets below their on-wire length, so those fields legitimately over-run
//! the captured slice and the strict parser fails with a *length* error
//! ([`etherparse::err::packet::SliceError::Len`]). **Only that error class**
//! triggers a re-parse with [`etherparse::LaxSlicedPacket`], which clamps
//! lengths to the captured bytes — matching how Wireshark and tcpdump dissect
//! truncated captures. Any other strict error (bad header version, bad IHL,
//! bad TCP data-offset, ...) is genuine structural corruption and the packet
//! is rejected, exactly as the strict parser intended — lax recovery is never
//! applied to a malformed packet. Strict-first keeps full validation for the
//! common (untruncated) case; lax parsing is only ever the truncation fallback.

use std::net::IpAddr;

use anyhow::{Result, anyhow};
use etherparse::err::packet::SliceError;
use etherparse::{
    EtherType, IpNumber, LaxNetSlice, LaxSlicedPacket, NetSlice, SlicedPacket, TransportSlice,
};
use pcap_file::DataLink;
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

/// Linux SLL ("cooked capture") header length in bytes. The protocol-type
/// field occupies the final two bytes (big-endian).
const SLL_HEADER_LEN: usize = 16;

/// `(source IP, destination IP, IP-layer protocol number)` — the subset of
/// the network layer the decoder needs, extracted uniformly from either the
/// strict or the lax slice types.
type IpTriple = (IpAddr, IpAddr, IpNumber);

pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket> {
    // Strict parse first: it validates the IPv4 `total_length` / IPv6
    // `payload_length` fields against the captured bytes and catches
    // genuinely malformed packets.
    let strict = match datalink {
        DataLink::ETHERNET => SlicedPacket::from_ethernet(data),
        DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => SlicedPacket::from_ip(data),
        DataLink::LINUX_SLL => SlicedPacket::from_linux_sll(data),
        other => return Err(anyhow!("Unsupported link type: {other:?}")),
    };

    match strict {
        // Strict succeeded with a usable IP layer — the common path.
        Ok(slice) => match &slice.net {
            Some(net) => Ok(build_parsed(
                strict_ip_triple(net),
                &slice.transport,
                data.len(),
            )),
            // Strict parsed cleanly but found no IP layer — a non-IP
            // frame (e.g. ARP). Lax parsing cannot conjure an IP layer
            // that is not present, so reject directly.
            None => Err(anyhow!("No IP layer found")),
        },
        // Strict failed on a *length* error — the signature of a
        // snaplen-truncated capture, where a header length field
        // legitimately over-runs the captured bytes. ONLY this error
        // class is retried with the lax slicer, which clamps lengths to
        // the captured slice (as Wireshark and tcpdump do). A structural
        // error is handled by the arm below.
        Err(SliceError::Len(_)) => {
            let lax = lax_parse(data, datalink)?;
            match &lax.net {
                Some(net) => Ok(build_parsed(lax_ip_triple(net), &lax.transport, data.len())),
                // Truncated past the IP header itself — undecodable.
                None => Err(anyhow!("No IP layer found")),
            }
        }
        // Any other strict error is genuine structural corruption (bad
        // header version, bad IHL, bad TCP data-offset, ...). Reject it,
        // exactly as the strict parser intended — lax recovery here
        // would admit malformed packets a forensics tool should drop.
        Err(e) => Err(anyhow!("Parse error: {e}")),
    }
}

/// Re-parse a frame with `etherparse`'s lax slicer, which clamps header
/// length fields to the captured slice instead of rejecting the packet.
fn lax_parse(data: &[u8], datalink: DataLink) -> Result<LaxSlicedPacket<'_>> {
    match datalink {
        DataLink::ETHERNET => {
            LaxSlicedPacket::from_ethernet(data).map_err(|e| anyhow!("Parse error: {e}"))
        }
        DataLink::RAW | DataLink::IPV4 | DataLink::IPV6 => {
            LaxSlicedPacket::from_ip(data).map_err(|e| anyhow!("Parse error: {e}"))
        }
        DataLink::LINUX_SLL => {
            // etherparse 0.16 has no `LaxSlicedPacket::from_linux_sll`. The
            // SLL header is a fixed 16 bytes whose final two bytes hold the
            // protocol type (an `EtherType`); hand the remainder to the lax
            // ether-type slicer, which is infallible.
            let proto = data
                .get(SLL_HEADER_LEN - 2..SLL_HEADER_LEN)
                .ok_or_else(|| anyhow!("Parse error: SLL header truncated"))?;
            let ether_type = EtherType::from(u16::from_be_bytes([proto[0], proto[1]]));
            Ok(LaxSlicedPacket::from_ether_type(
                ether_type,
                &data[SLL_HEADER_LEN..],
            ))
        }
        other => Err(anyhow!("Unsupported link type: {other:?}")),
    }
}

/// Extract the [`IpTriple`] from a strict network slice.
fn strict_ip_triple(net: &NetSlice<'_>) -> IpTriple {
    match net {
        NetSlice::Ipv4(ipv4) => {
            let header = ipv4.header();
            (
                IpAddr::V4(header.source_addr()),
                IpAddr::V4(header.destination_addr()),
                header.protocol(),
            )
        }
        NetSlice::Ipv6(ipv6) => {
            let header = ipv6.header();
            (
                IpAddr::V6(header.source_addr()),
                IpAddr::V6(header.destination_addr()),
                ipv6.payload().ip_number,
            )
        }
    }
}

/// Extract the [`IpTriple`] from a lax network slice.
fn lax_ip_triple(net: &LaxNetSlice<'_>) -> IpTriple {
    match net {
        LaxNetSlice::Ipv4(ipv4) => {
            let header = ipv4.header();
            (
                IpAddr::V4(header.source_addr()),
                IpAddr::V4(header.destination_addr()),
                header.protocol(),
            )
        }
        LaxNetSlice::Ipv6(ipv6) => {
            let header = ipv6.header();
            (
                IpAddr::V6(header.source_addr()),
                IpAddr::V6(header.destination_addr()),
                ipv6.payload().ip_number,
            )
        }
    }
}

/// Assemble a [`ParsedPacket`] from an extracted [`IpTriple`] and the
/// transport slice, which is the same `TransportSlice` type on both the
/// strict and lax parse paths.
fn build_parsed(
    ip: IpTriple,
    transport: &Option<TransportSlice<'_>>,
    packet_len: usize,
) -> ParsedPacket {
    let (src_ip, dst_ip, ip_protocol) = ip;

    let (protocol, transport_info) = match transport {
        Some(TransportSlice::Tcp(tcp)) => (
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
        Some(TransportSlice::Udp(udp)) => (
            Protocol::Udp,
            TransportInfo::Udp {
                src_port: udp.source_port(),
                dst_port: udp.destination_port(),
            },
        ),
        Some(TransportSlice::Icmpv4(_) | TransportSlice::Icmpv6(_)) => {
            (Protocol::Icmp, TransportInfo::None)
        }
        None => (Protocol::Other(ip_protocol.0), TransportInfo::None),
    };

    let payload = match transport {
        Some(TransportSlice::Tcp(tcp)) => tcp.payload().to_vec(),
        Some(TransportSlice::Udp(udp)) => udp.payload().to_vec(),
        _ => Vec::new(),
    };

    ParsedPacket {
        src_ip,
        dst_ip,
        protocol,
        transport: transport_info,
        payload,
        packet_len,
    }
}
