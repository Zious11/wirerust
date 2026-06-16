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
//! (EtherType 0x0806) are routed to `extract_arp_frame`, which is fully
//! implemented in STORY-112: a size-guard checks the 28-byte minimum, then all
//! 7 fields (hardware type, protocol type, hardware/protocol length, operation,
//! sender/target MAC and IP) are extracted. Ethernet/IPv4 ARP produces
//! `Ok(DecodedFrame::Arp(...))`. Any non-Ethernet/IPv4 ARP (malformed header
//! fields) produces `Err("Non-Ethernet/IPv4 ARP frame")` on **both** the strict
//! and lax paths (D-078/D-078b, BC-2.16.009 v1.6 — path-independence, D11).
//! `Err("truncated ARP frame")` is reserved exclusively for genuine snaplen
//! truncation: valid Ethernet/IPv4 header fields present but variable section
//! cut short, or frame too short to read the fixed header at all.
//! Non-IP non-ARP frames return `Err("No IP layer found")`.
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
use etherparse::err::Layer;
use etherparse::err::packet::SliceError;
use etherparse::{
    ArpHardwareId, ArpPacketSlice, EtherType, IpNumber, LaxNetSlice, LaxSlicedPacket, NetSlice,
    SlicedPacket, TransportSlice,
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
    pub operation: u16, // 1 = Request, 2 = Reply
    pub sender_mac: [u8; 6],
    pub sender_ip: [u8; 4],
    pub target_mac: [u8; 6],
    pub target_ip: [u8; 4],
    pub outer_src_mac: Option<[u8; 6]>, // Ethernet frame src MAC; None for SLL
    pub packet_len: usize,
}

/// The result of a successful `decode_packet` call.
///
/// IP frames (IPv4 and IPv6) become `Ip(ParsedPacket)`. The `Arp(ArpFrame)`
/// variant is produced by the fully-implemented `extract_arp_frame` introduced
/// in STORY-112: Ethernet/IPv4 ARP yields `Ok(DecodedFrame::Arp(...))`;
/// non-Ethernet/IPv4 (malformed) ARP yields `Err("Non-Ethernet/IPv4 ARP frame")`
/// on both strict and lax paths (D-078/D-078b, BC-2.16.009 v1.6 — D11);
/// genuine snaplen truncation (valid fixed-header fields, short variable section)
/// yields `Err("truncated ARP frame")`. Non-IP non-ARP frames are errors, not
/// `Ok` variants.
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
            // AC-005 / BC-2.02.009 Invariant 1 — ARP three-way dispatch arm.
            // outer_src_mac is extracted from slice.link for D12 mismatch detection
            // (STORY-113). extract_arp_frame is fully implemented in STORY-112:
            // size-guard + 7-field extraction; Some(f) → Ok(DecodedFrame::Arp(f));
            // None (non-Ethernet/IPv4 ARP) → Err("Non-Ethernet/IPv4 ARP frame").
            // (ADR-008 Decision 3; BC-2.02.009 v1.6 Invariant 1.)
            Some(NetSlice::Arp(arp)) => {
                let outer_src_mac: Option<[u8; 6]> = slice.link.as_ref().and_then(|l| {
                    if let etherparse::LinkSlice::Ethernet2(eth) = l {
                        Some(eth.source())
                    } else {
                        None
                    }
                });
                match extract_arp_frame(arp, outer_src_mac, data.len()) {
                    Some(f) => Ok(DecodedFrame::Arp(f)),
                    // AC-012 / BC-2.16.015 / BC-2.16.009 PC3: non-Eth/IPv4 ARP
                    // (hw_addr_type!=ETHERNET, proto_addr_type!=IPv4, hw_addr_size!=6,
                    // or proto_addr_size!=4) → decode-layer Err with diagnostic string.
                    None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame")),
                }
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
                // Lax ARP routing arm — NOT unreachable! (BC-2.02.009 Invariant 2,
                // ADR-008 Decision 3 v2.1; arp-architecture-delta §2.2 v1.16).
                // This decode_packet lax arm IS reachable (live routing):
                // decode_packet intercepts Some(LaxNetSlice::Arp(_)) here before
                // calling lax_ip_triple, which carries the unreachable! arm.
                // Snaplen-truncated ARP frames yield Some(LaxNetSlice::Arp(_)) from
                // the lax parser and reach this arm. outer_src_mac is extracted from
                // lax.link; extract_arp_frame is fully implemented in STORY-112:
                // Some(f) → Ok(DecodedFrame::Arp(f)); None (non-Eth/IPv4 type)
                // → Err("Non-Ethernet/IPv4 ARP frame") per D-078b / BC-2.16.009 PC3
                // (path-independence: same D11-trigger string as the strict arm).
                Some(LaxNetSlice::Arp(arp)) => {
                    let outer_src_mac: Option<[u8; 6]> = lax.link.as_ref().and_then(|l| {
                        if let etherparse::LinkSlice::Ethernet2(eth) = l {
                            Some(eth.source())
                        } else {
                            None
                        }
                    });
                    match extract_arp_frame(arp, outer_src_mac, data.len()) {
                        Some(f) => Ok(DecodedFrame::Arp(f)),
                        // D-078b / BC-2.16.009 v1.6 PC3: non-Eth/IPv4 ARP via the lax
                        // parse path must produce the same D11-trigger string as the
                        // strict arm (path-independence). "Non-Ethernet/IPv4 ARP frame"
                        // is matched by main.rs → record_malformed → D11 finding.
                        None => Err(anyhow!("Non-Ethernet/IPv4 ARP frame")),
                    }
                }
                Some(net) => Ok(DecodedFrame::Ip(build_parsed(
                    lax_ip_triple(net),
                    &lax.transport,
                    data.len(),
                ))),
                // Lax parser could not reconstruct a net layer slice.
                //
                // D-078 / BC-2.16.009 PC3 / BC-2.16.015 PC-7a/7b:
                // When stop_err == Layer::Arp, the lax slicer failed at
                // ArpPacketSlice::from_slice — the ARP payload was too short
                // for the declared hw/proto sizes.  Two distinct situations
                // arrive here:
                //
                //   (a) MALFORMED: the ARP fixed header (8 bytes) is readable
                //       and reveals bad field values (hlen != 6 or plen != 4
                //       or htype != 0x0001 or ptype != 0x0800).  The frame is
                //       structurally invalid and must route to D11:
                //       → Err("Non-Ethernet/IPv4 ARP frame").
                //
                //   (b) GENUINE TRUNCATION: the fixed header declares valid
                //       Ethernet/IPv4 field values but the variable section
                //       was cut short by snaplen.  Must stay a decode-error:
                //       → Err("truncated ARP frame").
                //
                //   (c) HEADER NOT READABLE: the frame is so short that even
                //       the 8-byte fixed header cannot be read (no field info
                //       available).  Treat conservatively as truncation:
                //       → Err("truncated ARP frame").
                //
                // Offset derivation: for Ethernet2 link frames the ARP payload
                // starts at byte 14 (6+6+2 Ethernet header) PLUS the sum of all
                // link-extension header lengths from `lax.link_exts` (F-1 /
                // BC-2.16.015 v1.6 / BC-2.16.009 v1.7 / D-078 VLAN-offset fix).
                // A single 802.1Q VLAN tag adds 4 bytes (TCI + inner EtherType),
                // QinQ adds 8, and MACsec adds its variable header length — all
                // handled via `LaxLinkExtSlice::header_len()` without hardcoding.
                // Non-Ethernet or absent link layers use conservative case (c).
                //
                // All byte accesses use bounds-checked .get() — no panics on
                // attacker-controlled frame lengths.
                None => {
                    let is_arp_truncation = lax
                        .stop_err
                        .as_ref()
                        .is_some_and(|(_, layer)| *layer == Layer::Arp);
                    if is_arp_truncation {
                        // Determine the ARP payload offset from the link header.
                        //
                        // F-1 / BC-2.16.015 v1.6 / BC-2.16.009 v1.7 / D-078 VLAN-offset fix:
                        // For Ethernet2 frames the ARP payload starts at the Ethernet2 base
                        // (14 bytes: 6 dst + 6 src + 2 EtherType) PLUS the summed on-wire
                        // header length of all link-extension headers in `lax.link_exts`
                        // (e.g. 4 bytes per 802.1Q/802.1ad VLAN tag, variable for MACsec).
                        // Using `LaxLinkExtSlice::header_len()` is correct for QinQ (two
                        // 4-byte tags → +8) and MACsec (variable length) without hardcoding.
                        // For any other link layer, fall back to conservative truncation (case c).
                        let arp_offset: Option<usize> =
                            lax.link.as_ref().and_then(|link| match link {
                                etherparse::LinkSlice::Ethernet2(_) => {
                                    let link_exts_len: usize =
                                        lax.link_exts.iter().map(|ext| ext.header_len()).sum();
                                    Some(14 + link_exts_len)
                                }
                                // `_` covers any future LinkSlice variants added by etherparse;
                                // conservatively treat as unreadable (case c).
                                _ => None,
                            });

                        // Attempt to read the 8-byte ARP fixed header and classify the frame.
                        //
                        // ARP fixed header layout (RFC 826):
                        //   [0..2]  htype  — hardware address type (0x0001 = Ethernet)
                        //   [2..4]  ptype  — protocol address type (0x0800 = IPv4)
                        //   [4]     hlen   — hardware address length (6 for Ethernet MAC)
                        //   [5]     plen   — protocol address length (4 for IPv4)
                        //   [6..8]  oper   — operation (not inspected here)
                        //
                        // If offset is known and frame has >= offset+8 bytes, peek the header.
                        let malformed = arp_offset.is_some_and(|offset| {
                            data.get(offset..offset + 8).is_some_and(|arp_hdr| {
                                let htype = u16::from_be_bytes([arp_hdr[0], arp_hdr[1]]);
                                let ptype = u16::from_be_bytes([arp_hdr[2], arp_hdr[3]]);
                                let hlen = arp_hdr[4];
                                let plen = arp_hdr[5];
                                // Non-Ethernet/IPv4 field values indicate a malformed frame
                                // (case a). All-valid values with short variable section is
                                // genuine truncation (case b) — not malformed.
                                htype != 0x0001 || ptype != 0x0800 || hlen != 6 || plen != 4
                            })
                        });

                        if malformed {
                            // Case (a): bad fixed-header field(s) visible → D11 (BC-2.16.009
                            // PC3 / D-078).  Same error string as the strict path so that the
                            // single main.rs dispatch condition covers both paths.
                            Err(anyhow!("Non-Ethernet/IPv4 ARP frame"))
                        } else {
                            // Case (b) or (c): valid fields or unreadable header → decode-error
                            // (BC-2.16.015 PC-7b / AC-007).
                            Err(anyhow!("truncated ARP frame"))
                        }
                    } else {
                        Err(anyhow!("No IP layer found"))
                    }
                }
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
/// Returns `Some(ArpFrame)` when `hw_addr_type == ETHERNET`, `proto_addr_type == IPV4`,
/// `hw_addr_size == 6`, and `proto_addr_size == 4` (Ethernet/IPv4 ARP); returns `None`
/// for any frame that fails the type or size check. `None` signals a non-Ethernet/IPv4
/// (malformed) frame: **both** the strict and lax call-sites return
/// `Err("Non-Ethernet/IPv4 ARP frame")` → D11 (BC-2.16.009 v1.6, D-078/D-078b
/// path-independence). `Err("truncated ARP frame")` is returned by the lax path
/// only for genuine truncation (valid fixed-header fields but unreadable/short
/// variable section), which never reaches this function.
///
/// This function is a **pure core** function (BC-2.16.015, VP-024 Sub-A):
/// - No I/O, no global state, no panic for any valid `ArpPacketSlice` input.
/// - All field copies are byte-exact from the `ArpPacketSlice` accessors.
/// - `outer_src_mac` and `packet_len` are passed through unchanged.
/// - Extraction is opcode-agnostic (BC-2.16.001 Invariant 4): any operation
///   value (1=Request, 2=Reply, 0=undefined, other) extracts successfully as
///   long as hw/proto sizes pass the size guard.
///
/// **Forbidden:** `src/decoder.rs` MUST NOT import `src/analyzer/arp.rs` (AC-010 /
/// arp-architecture-delta §2.1 decode-vs-analysis separation boundary).
pub fn extract_arp_frame(
    arp: &ArpPacketSlice<'_>,
    outer_src_mac: Option<[u8; 6]>,
    packet_len: usize,
) -> Option<ArpFrame> {
    // Guard: only Ethernet/IPv4 ARP is admitted (BC-2.16.001 PC2-PC5,
    // BC-2.16.009 PC3a/PC3b, EC-007/EC-008, VP-024 Sub-A).
    // hw_addr_type must be ETHERNET (0x0001); proto_addr_type must be IPv4 (0x0800);
    // hw_addr_size must be 6; proto_addr_size must be 4.
    // Any frame failing either the type check or the size check returns None.
    // No panic: all comparisons are on u8/newtype fields.
    if arp.hw_addr_type() != ArpHardwareId::ETHERNET
        || arp.proto_addr_type() != EtherType::IPV4
        || arp.hw_addr_size() != 6
        || arp.proto_addr_size() != 4
    {
        return None;
    }

    // At this point from_slice has already validated that the slice is at least
    // 8 + 6*2 + 4*2 = 28 bytes, so sender_hw_addr() yields exactly 6 bytes,
    // sender_protocol_addr() yields exactly 4 bytes, and so on. The try_from
    // conversions cannot fail here (the sizes are guaranteed by the hw/proto
    // size check above and by ArpPacketSlice's own length validation).
    // We use expect() with an actionable message (not unwrap) per coding rules;
    // in practice these are provably infallible given the size guard above.
    let sender_mac_bytes = arp.sender_hw_addr();
    let sender_ip_bytes = arp.sender_protocol_addr();
    let target_mac_bytes = arp.target_hw_addr();
    let target_ip_bytes = arp.target_protocol_addr();

    // Convert &[u8] slices to fixed-size arrays. These are provably infallible:
    // hw_addr_size()==6 and proto_addr_size()==4 guarantee the slice lengths.
    let sender_mac = <[u8; 6]>::try_from(sender_mac_bytes)
        .expect("sender_hw_addr is guaranteed 6 bytes when hw_addr_size==6");
    let sender_ip = <[u8; 4]>::try_from(sender_ip_bytes)
        .expect("sender_protocol_addr is guaranteed 4 bytes when proto_addr_size==4");
    let target_mac = <[u8; 6]>::try_from(target_mac_bytes)
        .expect("target_hw_addr is guaranteed 6 bytes when hw_addr_size==6");
    let target_ip = <[u8; 4]>::try_from(target_ip_bytes)
        .expect("target_protocol_addr is guaranteed 4 bytes when proto_addr_size==4");

    Some(ArpFrame {
        operation: arp.operation().0,
        sender_mac,
        sender_ip,
        target_mac,
        target_ip,
        outer_src_mac,
        packet_len,
    })
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
        // This arm is a compile-exhaustiveness guard only.
        // In practice, decode_packet's Err(SliceError::Len(_)) arm matches
        // Some(LaxNetSlice::Arp(_)) BEFORE calling lax_ip_triple (see above),
        // so lax_ip_triple is never called with an ARP slice at runtime.
        // The arm exists so the match is exhaustive under all possible inputs
        // (AC-005 / BC-2.02.009 Invariant 2 — lax ARP is handled in decode_packet,
        // not here). This is a code-logic invariant: if this arm executes it
        // indicates a caller error, but it is provably unreachable via decode_packet.
        LaxNetSlice::Arp(_) => unreachable!(
            "ARP frames are routed in decode_packet's Err(SliceError::Len) arm \
             before lax_ip_triple is called — this arm is a compile-safety guard \
             (BC-2.02.009 Invariant 2; arp-architecture-delta §2.2)"
        ),
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

/// VP-024 Sub-A Kani harness skeletons — STORY-112 (AC-011).
///
/// All three harnesses target `extract_arp_frame` directly (pure-core function).
/// Bodies are `todo!()` per BC-5.38.001 — the real bodies are the formal-verifier's
/// work at the F6 gate (VP-024 verification_lock: false until then).
///
/// These blocks are only compiled under `cargo kani` (the `kani` cfg is registered
/// in `Cargo.toml` [lints.rust] so `cargo check --all-targets` on the stable
/// toolchain never sees these items and compilation is unaffected).
#[cfg(kani)]
mod kani_proofs {
    use super::*;

    const ARP_ETH_IPV4_LEN: usize = 28; // minimum wire length for Ethernet/IPv4 ARP

    /// VP-024 Sub-A harness 1: `extract_arp_frame` never panics for any valid
    /// `ArpPacketSlice` and any `outer_src_mac`. Proves no-panic / OOB-freedom.
    /// BC-2.16.001 postcondition 1 (safety), BC-2.16.002 postcondition 1 (safety).
    ///
    /// Body is `todo!()` — real harness body filled by formal-verifier at F6 gate.
    #[kani::proof]
    fn verify_extract_arp_frame_safety() {
        todo!(
            "VP-024 Sub-A safety harness — body filled by formal-verifier at F6 gate \
             (STORY-112 stub: AC-011)"
        )
    }

    /// VP-024 Sub-A harness 2: for a well-formed Ethernet/IPv4 ARP buffer,
    /// `extract_arp_frame` returns `Some(ArpFrame)` with fields byte-exactly copied
    /// from the `ArpPacketSlice` accessors. BC-2.16.001 postconditions 2–8.
    ///
    /// Body is `todo!()` — real harness body filled by formal-verifier at F6 gate.
    /// F4 obligation: add `kani::cover!` reachability assertion before F6 lock
    /// (see VP-024 v1.4 vacuous-satisfiability note).
    #[kani::proof]
    fn verify_extract_arp_frame_eth_ipv4_correctness() {
        todo!(
            "VP-024 Sub-A correctness harness — body filled by formal-verifier at F6 gate \
             (STORY-112 stub: AC-011). F4 obligation: add kani::cover! reachability check."
        )
    }

    /// VP-024 Sub-A harness 3: `extract_arp_frame` returns `None` (no panic) for any
    /// ARP frame that fails the full four-part type+size guard introduced by D-077:
    ///
    /// - `hw_addr_type() != ArpHardwareId::ETHERNET` (non-Ethernet hardware type), OR
    /// - `proto_addr_type() != EtherType::IPV4` (non-IPv4 protocol type), OR
    /// - `hw_addr_size() != 6` (bad hardware address length), OR
    /// - `proto_addr_size() != 4` (bad protocol address length).
    ///
    /// The harness name predates D-077 (which widened the reject contract beyond size
    /// checks to also cover type rejection); the name is retained per VP-024 v1.9
    /// scope note to avoid churn on the harness identifier.
    ///
    /// References: BC-2.16.001 PC2-PC5, BC-2.16.009 PC3a-PC3d, EC-007/EC-008.
    ///
    /// Body is `todo!()` — real harness body filled by formal-verifier at F6 gate.
    /// F4 obligation: confirm `from_slice` accepts non-Ethernet/non-IPv4 buffers (Ok
    /// arm reachable) or restructure to use `kani::cover!` before F6 lock
    /// (see VP-024 v1.9 vacuous-satisfiability note).
    #[kani::proof]
    fn verify_extract_arp_frame_none_on_bad_size() {
        todo!(
            "VP-024 Sub-A negative harness — body filled by formal-verifier at F6 gate \
             (STORY-112 stub: AC-011). F4 obligation: resolve vacuous-satisfiability risk."
        )
    }
}
