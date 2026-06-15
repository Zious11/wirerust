//! Link-layer / IP / ARP / transport decoder for raw pcap frames.
//!
//! Turns a single captured frame plus its [`pcap_file::DataLink`] into a
//! [`DecodedFrame`] — either a [`DecodedFrame::Ip`] containing a
//! [`ParsedPacket`] with source / destination IP, transport-layer
//! information ([`TransportInfo`]), and the payload slice, or a
//! [`DecodedFrame::Arp`] containing an [`ArpFrame`] with ARP fields and
//! the outer Ethernet source MAC. Application-layer parsing is NOT done
//! here — that is the analyzer / dispatcher pipeline's responsibility.
//!
//! Currently supports Ethernet and Linux-cooked-capture (SLL) link layers
//! via `etherparse::SlicedPacket`. IPv4 and IPv6 are both handled; TCP and
//! UDP transports surface their port / flag tuple, ICMP is reported as the
//! parent protocol with no transport detail, and everything else is reported
//! as `Protocol::Other(proto_num)` with no transport info. ARP frames
//! (EtherType 0x0806) with Ethernet/IPv4 format return `DecodedFrame::Arp`;
//! non-Ethernet/IPv4 ARP frames return `Err("Non-Ethernet/IPv4 ARP frame")`;
//! non-IP non-ARP frames return `Err("No IP layer found")`.
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
//!
//! When the snaplen cut lands *inside* the transport header (not just the
//! payload), the lax parser recovers the IP layer but not the transport
//! layer: such a frame decodes with its IP addresses intact but as
//! `Protocol::Other(ip_protocol)` with `TransportInfo::None` — the honest
//! degraded result, since the ports / flags simply were not captured. The
//! "TCP / UDP surface their port / flag tuple" statement above therefore
//! holds only when the transport header itself was captured.

use std::net::IpAddr;

use anyhow::{Result, anyhow};
// `SliceError::Len` is the strict-parser error class the truncation
// fallback keys on (see `decode_packet`). That discrimination is part
// of the etherparse 0.20 API contract; `Cargo.toml` constrains the
// dependency to `0.20.x` accordingly. `SliceError::Len` is confirmed
// present and unchanged in 0.20.1+ — `test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing`
// and `test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered`
// act as the contract tests for it.
use etherparse::err::packet::SliceError;
use etherparse::{
    ArpPacketSlice, EtherType, IpNumber, LaxNetSlice, LaxSlicedPacket, NetSlice, SlicedPacket,
    TransportSlice,
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

/// ARP frame extracted from a decoded Ethernet/IPv4 ARP packet.
///
/// Defined in `src/decoder.rs` (not in `src/analyzer/arp.rs`) per the
/// decode-vs-analysis separation boundary (BC-2.16.015, arp-architecture-delta §2.1).
/// `outer_src_mac` carries the Ethernet frame's source MAC for D12 mismatch detection
/// (STORY-113). It is `None` for non-Ethernet (SLL) captures.
#[derive(Debug, Clone, PartialEq)]
pub struct ArpFrame {
    pub operation: u16,               // 1 = Request, 2 = Reply
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
    pub outer_src_mac: Option<[u8; 6]>, // Ethernet frame src MAC; None for SLL
    pub packet_len: usize,
}

/// The result of a successful `decode_packet` call.
///
/// IP frames (IPv4 and IPv6) become `Ip(ParsedPacket)`; Ethernet/IPv4 ARP
/// frames become `Arp(ArpFrame)`. Non-Ethernet/IPv4 ARP frames and non-IP
/// non-ARP frames are both errors, not `Ok` variants.
#[derive(Debug, Clone)]
pub enum DecodedFrame {
    Ip(ParsedPacket),
    Arp(ArpFrame),
}

/// Linux SLL ("cooked capture") header length in bytes. The protocol-type
/// field occupies the final two bytes (big-endian).
const SLL_HEADER_LEN: usize = 16;

/// `(source IP, destination IP, IP-layer protocol number)` — the subset of
/// the network layer the decoder needs, extracted uniformly from either the
/// strict or the lax slice types.
type IpTriple = (IpAddr, IpAddr, IpNumber);

pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<DecodedFrame> {
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
            Some(NetSlice::Arp(_arp)) => {
                // STORY-111 stub: ARP three-way dispatch — real routing is
                // implemented in STORY-112 (extract_arp_frame full impl).
                // The outer_src_mac extraction from slice.link and the call
                // to extract_arp_frame are STORY-112's job.
                todo!("STORY-111: ARP extraction in strict Ok arm — implement in STORY-112")
            }
            Some(net) => Ok(DecodedFrame::Ip(build_parsed(
                strict_ip_triple(net),
                &slice.transport,
                data.len(),
            ))),
            // Strict parsed cleanly but found no IP layer — a non-IP
            // frame (e.g. LLDP). Lax parsing cannot conjure an IP layer
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
                Some(LaxNetSlice::Arp(_arp)) => {
                    // STORY-111 stub: lax ARP routing — full impl is STORY-112.
                    // Per arp-architecture-delta §2.2, outer_src_mac is extracted
                    // from lax.link, then extract_arp_frame is called. This arm
                    // is NOT unreachable! — truncated ARP yields LaxNetSlice::Arp.
                    todo!("STORY-111: lax ARP routing in Err(SliceError::Len) arm — implement in STORY-112")
                }
                Some(net) => Ok(DecodedFrame::Ip(build_parsed(
                    lax_ip_triple(net),
                    &lax.transport,
                    data.len(),
                ))),
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

/// Extract an [`ArpFrame`] from an etherparse `ArpPacketSlice`.
///
/// Returns `Some(ArpFrame)` for Ethernet/IPv4 ARP (hw_addr_size=6, proto_addr_size=4,
/// hw_type=0x0001, proto_type=0x0800); returns `None` for non-Ethernet/IPv4 ARP frames
/// (signals the caller to return `Err("Non-Ethernet/IPv4 ARP frame")`).
///
/// STORY-111 stub: body is `todo!()`. Full implementation is STORY-112.
/// BC-5.38.005 self-check: if this body were real, AC-001 and AC-002 tests would pass
/// trivially — therefore todo!() is required here to maintain the Red Gate.
#[allow(dead_code)]
pub fn extract_arp_frame(
    _arp: &ArpPacketSlice<'_>,
    _outer_src_mac: Option<[u8; 6]>,
    _packet_len: usize,
) -> Option<ArpFrame> {
    todo!("STORY-111 stub: extract_arp_frame — implement in STORY-112")
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
            // etherparse 0.20 has no `LaxSlicedPacket::from_linux_sll`. The
            // SLL header is a fixed 16 bytes whose final two bytes hold the
            // protocol type (an `EtherType`); hand the remainder to the lax
            // ether-type slicer, which is infallible.
            //
            // The `.get(..)` guard below is defensive: in practice an SLL
            // frame shorter than 16 bytes fails the *strict* parse with a
            // non-`Len` error and is rejected before `lax_parse` is ever
            // reached, so the truncated-header branch is not expected to
            // fire — but bounding the slice keeps it panic-free regardless.
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
        // Compile-safety arm only — ARP frames are routed out of decode_packet's
        // strict Ok(slice) arm before strict_ip_triple is ever called.
        // This arm is never reached at runtime (ADR-008 Decision 3, BC-2.02.009
        // Invariant 2). unreachable! is correct here per AC-006.
        NetSlice::Arp(_) => unreachable!("ARP frames are routed before strict_ip_triple"),
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
        // Explicit routing arm — NOT unreachable! Snaplen-truncated ARP frames
        // yield LaxNetSlice::Arp from etherparse 0.20's lax parser and DO reach
        // this function. An unreachable! here would be a reachable runtime panic,
        // violating VP-008/VP-024 Sub-A (ADR-008 Decision 3 v1.6, BC-2.02.009
        // Invariant 2, AC-007). STORY-112 replaces this todo! with real routing.
        LaxNetSlice::Arp(_) => {
            todo!("STORY-111 lax ARP routing — implement real routing to extract_arp_frame in STORY-112")
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
