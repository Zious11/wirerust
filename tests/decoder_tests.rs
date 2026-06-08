use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pcap_file::DataLink;
use wirerust::decoder::{Protocol, TransportInfo, decode_packet};

fn make_tcp_packet() -> Vec<u8> {
    vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00, // ethertype: IPv4
        // IPv4 header (20 bytes)
        0x45, 0x00, 0x00, 0x28, 0x00, 0x01, 0x00, 0x00, 0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8, 0x01,
        0x0a, // src: 192.168.1.10
        0xc0, 0xa8, 0x01, 0x01, // dst: 192.168.1.1
        // TCP header (20 bytes)
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x50, 0x02, 0xff, 0xff, 0x00, 0x00, 0x00,
        0x00,
    ]
}

fn make_udp_packet() -> Vec<u8> {
    vec![
        // Ethernet header
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x00,
        // IPv4 header (20 bytes), protocol=UDP (0x11)
        0x45, 0x00, 0x00, 0x1c, 0x00, 0x01, 0x00, 0x00, 0x40, 0x11, 0x00, 0x00, 0x0a, 0x00, 0x00,
        0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
        // UDP header (8 bytes)
        0xd9, 0x03, 0x00, 0x35, // src port 55555, dst port 53
        0x00, 0x08, 0x00, 0x00, // length=8, checksum
    ]
}

fn make_raw_ip_tcp_packet() -> Vec<u8> {
    vec![
        // IPv4 header (20 bytes) — no Ethernet header
        0x45, 0x00, 0x00, 0x28, // version/IHL, DSCP, total length=40
        0x00, 0x01, 0x00, 0x00, // identification, flags/fragment
        0x40, 0x06, 0x00, 0x00, // TTL=64, protocol=TCP, checksum
        0xc0, 0xa8, 0x01, 0x0a, // src: 192.168.1.10
        0xc0, 0xa8, 0x01, 0x01, // dst: 192.168.1.1
        // TCP header (20 bytes)
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, // seq number
        0x00, 0x00, 0x00, 0x00, // ack number
        0x50, 0x02, 0xff, 0xff, // data offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent pointer
    ]
}

#[test]
fn test_decode_tcp_packet() {
    let data = make_tcp_packet();
    let parsed = decode_packet(&data, DataLink::ETHERNET).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));

    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }

    assert_eq!(parsed.protocol, Protocol::Tcp);
}

#[test]
fn test_decode_udp_dns_packet() {
    let data = make_udp_packet();
    let parsed = decode_packet(&data, DataLink::ETHERNET).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));

    match parsed.transport {
        TransportInfo::Udp { src_port, dst_port } => {
            assert_eq!(src_port, 55555);
            assert_eq!(dst_port, 53);
        }
        _ => panic!("Expected UDP"),
    }

    assert_eq!(parsed.protocol, Protocol::Udp);
    assert_eq!(parsed.app_protocol_hint(), Some("DNS"));
}

#[test]
fn test_decode_invalid_packet() {
    let garbage = vec![0x00, 0x01, 0x02];
    assert!(decode_packet(&garbage, DataLink::ETHERNET).is_err());
}

#[test]
fn test_decode_raw_ip_tcp_packet() {
    let data = make_raw_ip_tcp_packet();
    let parsed = decode_packet(&data, DataLink::RAW).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }
}

#[test]
fn test_decode_ipv4_linktype_uses_from_ip() {
    // DataLink::IPV4 should use from_ip(), same as RAW
    let data = make_raw_ip_tcp_packet();
    let parsed = decode_packet(&data, DataLink::IPV4).unwrap();
    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
}

fn make_raw_ipv6_tcp_packet() -> Vec<u8> {
    vec![
        // IPv6 header (40 bytes) — no Ethernet header
        0x60, 0x00, 0x00, 0x00, // version=6, traffic class, flow label
        0x00, 0x14, 0x06, 0x40, // payload length=20, next header=TCP(6), hop limit=64
        // src: 2001:db8::1
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, // dst: 2001:db8::2
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02, // TCP header (20 bytes)
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, // seq number
        0x00, 0x00, 0x00, 0x00, // ack number
        0x50, 0x02, 0xff, 0xff, // data offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent pointer
    ]
}

#[test]
fn test_decode_ipv6_tcp_packet() {
    let data = make_raw_ipv6_tcp_packet();
    let parsed = decode_packet(&data, DataLink::IPV6).unwrap();

    assert_eq!(
        parsed.src_ip,
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1))
    );
    assert_eq!(
        parsed.dst_ip,
        IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2))
    );
    assert_eq!(parsed.protocol, Protocol::Tcp);
    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }
}

fn make_linux_sll_tcp_packet() -> Vec<u8> {
    let mut pkt = Vec::new();
    // Linux SLL header (16 bytes)
    pkt.extend_from_slice(&[0x00, 0x00]); // packet type: sent by us
    pkt.extend_from_slice(&[0x00, 0x01]); // ARPHRD_ETHER
    pkt.extend_from_slice(&[0x00, 0x06]); // link-layer address length
    pkt.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00]); // address (padded)
    pkt.extend_from_slice(&[0x08, 0x00]); // protocol type: IPv4
    // IPv4 header (20 bytes)
    pkt.extend_from_slice(&[
        0x45, 0x00, 0x00, 0x28, // version/IHL, total length=40
        0x00, 0x01, 0x00, 0x00, // identification, flags/fragment
        0x40, 0x06, 0x00, 0x00, // TTL=64, protocol=TCP, checksum
        0x0a, 0x00, 0x00, 0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
    ]);
    // TCP header (20 bytes)
    pkt.extend_from_slice(&[
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, // seq number
        0x00, 0x00, 0x00, 0x00, // ack number
        0x50, 0x02, 0xff, 0xff, // data offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent pointer
    ]);
    pkt
}

#[test]
fn test_decode_linux_sll_tcp_packet() {
    let data = make_linux_sll_tcp_packet();
    let parsed = decode_packet(&data, DataLink::LINUX_SLL).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }
}

// --- Snaplen-truncated captures -----------------------------------------
//
// A `tcpdump -s 96` capture truncates each packet below its on-wire
// length, but the IPv4 `total_length` field still reports the full
// original size. `etherparse`'s strict parser rejects that as a length
// inconsistency; `decode_packet` must fall back to lax parsing and
// recover the headers from the bytes that were captured.

/// `make_tcp_packet` with the IPv4 `total_length` field overwritten to
/// claim a full 1500-byte packet, simulating snaplen truncation. The
/// field is at frame offset 16..18 (Ethernet 14 + IPv4 bytes 2..4).
fn make_snaplen_truncated_eth_tcp() -> Vec<u8> {
    let mut pkt = make_tcp_packet();
    pkt[16] = 0x05;
    pkt[17] = 0xdc; // total_length = 1500, far past the captured bytes
    pkt
}

#[test]
fn test_decode_snaplen_truncated_ethernet_recovers_via_lax_parsing() {
    let data = make_snaplen_truncated_eth_tcp();
    let parsed = decode_packet(&data, DataLink::ETHERNET)
        .expect("snaplen-truncated Ethernet frame must decode via the lax fallback");

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }
}

#[test]
fn test_decode_snaplen_truncated_raw_ip_recovers_via_lax_parsing() {
    // RAW datalink: IPv4 `total_length` is at frame offset 2..4.
    let mut data = make_raw_ip_tcp_packet();
    data[2] = 0x05;
    data[3] = 0xdc;
    let parsed = decode_packet(&data, DataLink::RAW)
        .expect("snaplen-truncated raw-IP frame must decode via the lax fallback");

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
}

#[test]
fn test_decode_snaplen_truncated_linux_sll_recovers_via_lax_parsing() {
    // SLL datalink: IPv4 starts after the 16-byte SLL header, so the
    // `total_length` field is at frame offset 18..20. This also
    // exercises the manual `from_ether_type` lax path, since
    // `etherparse` has no `LaxSlicedPacket::from_linux_sll`.
    let mut data = make_linux_sll_tcp_packet();
    data[18] = 0x05;
    data[19] = 0xdc;
    let parsed = decode_packet(&data, DataLink::LINUX_SLL)
        .expect("snaplen-truncated SLL frame must decode via the lax fallback");

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
    assert_eq!(parsed.protocol, Protocol::Tcp);
}

#[test]
fn test_decode_snaplen_truncated_clamps_payload_to_captured_bytes() {
    // Header stack plus 10 real payload bytes, with `total_length`
    // claiming a 1500-byte packet. Lax parsing must clamp the payload
    // to the bytes actually present rather than trusting the field —
    // otherwise it would over-read past the buffer.
    let mut data = make_tcp_packet();
    data.extend_from_slice(&[0xde, 0xad, 0xbe, 0xef, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06]);
    data[16] = 0x05;
    data[17] = 0xdc;
    let parsed = decode_packet(&data, DataLink::ETHERNET).expect("must decode");

    assert_eq!(
        parsed.payload.len(),
        10,
        "lax parsing must clamp the TCP payload to the captured bytes, \
         not to the inflated IPv4 total_length"
    );
}

#[test]
fn test_decode_snaplen_truncated_ipv6_recovers_via_lax_parsing() {
    // The IPv6 `payload_length` field sits at frame offset 4..6 (no
    // Ethernet header on a RAW capture). Inflating it past the captured
    // bytes makes the strict parser fail with the same length error an
    // IPv4 `total_length` over-run produces — this confirms the lax
    // fallback covers IPv6 truncation too, not only IPv4.
    let mut data = make_raw_ipv6_tcp_packet();
    data[4] = 0x05;
    data[5] = 0xdc; // payload_length = 1500, far past the captured bytes
    let parsed = decode_packet(&data, DataLink::IPV6)
        .expect("snaplen-truncated IPv6 frame must decode via the lax fallback");

    assert_eq!(parsed.protocol, Protocol::Tcp);
    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }
}

#[test]
fn test_decode_truncation_inside_tcp_header_degrades_to_other() {
    // A frame physically cut *inside* the TCP header — only 10 of the
    // 20 header bytes captured (44-byte buffer = Ethernet 14 + IPv4 20 +
    // 10 TCP). The lax fallback recovers the IP layer but cannot recover
    // the transport layer, so the packet decodes with its IP addresses
    // intact but as `Protocol::Other(6)` with no transport detail. This
    // pins the documented degraded-decode behavior for a snaplen cut
    // that lands within the transport header rather than the payload.
    let full = make_tcp_packet();
    let truncated = &full[..44];
    let parsed = decode_packet(truncated, DataLink::ETHERNET)
        .expect("a frame truncated mid-TCP-header must still decode its IP layer");

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(
        parsed.protocol,
        Protocol::Other(6),
        "a captured-but-incomplete TCP header must degrade to Other(<ip-proto>)"
    );
    assert!(
        matches!(parsed.transport, TransportInfo::None),
        "an incomplete TCP header yields no transport detail"
    );
}

// --- Error-path discipline ----------------------------------------------
//
// The strict→lax fallback must apply lax recovery ONLY to snaplen
// truncation (a strict *length* error), never to structural corruption.
// These tests pin both the non-IP-frame branch and the
// corruption-is-rejected branch.

#[test]
fn test_decode_arp_frame_reports_no_ip_layer() {
    // An ARP frame (ethertype 0x0806) parses cleanly at the link layer
    // but has no IP layer. `decode_packet` must reject it with
    // "No IP layer found" — not attempt a lax recovery.
    let mut frame = vec![
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac (broadcast)
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x06, // ethertype: ARP
    ];
    frame.extend_from_slice(&[
        0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, // htype/ptype/hlen/plen/oper
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // sender mac
        0xc0, 0xa8, 0x01, 0x0a, // sender ip
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // target mac
        0xc0, 0xa8, 0x01, 0x01, // target ip
    ]);
    let err = decode_packet(&frame, DataLink::ETHERNET).unwrap_err();
    assert!(
        err.to_string().contains("No IP layer found"),
        "an ARP frame must report 'No IP layer found'; got: {err}"
    );
}

#[test]
fn test_decode_structurally_corrupt_packet_is_rejected_not_lax_recovered() {
    // A frame with a valid IPv4 header but an invalid TCP data-offset
    // (0, below the minimum of 5) is structural corruption, not snaplen
    // truncation. The strict parser rejects it with a non-length error;
    // `decode_packet` must NOT fall back to lax recovery (which would
    // admit the malformed packet) — it must reject with "Parse error".
    let mut frame = make_tcp_packet();
    // TCP data-offset/reserved byte is at frame offset 14 + 20 + 12 = 46.
    frame[46] = 0x00; // data offset 0 — structurally invalid
    let err = decode_packet(&frame, DataLink::ETHERNET).unwrap_err();
    assert!(
        err.to_string().contains("Parse error"),
        "a structurally-corrupt packet must be rejected as a parse error, \
         not lax-recovered; got: {err}"
    );
}
