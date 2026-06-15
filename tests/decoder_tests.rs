use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pcap_file::DataLink;
use wirerust::decoder::{DecodedFrame, Protocol, TransportInfo, decode_packet};

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET).unwrap() else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET).unwrap() else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::RAW).unwrap() else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::IPV4).unwrap() else {
        panic!("expected IP DecodedFrame")
    };
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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::IPV6).unwrap() else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::LINUX_SLL).unwrap() else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("snaplen-truncated Ethernet frame must decode via the lax fallback")
    else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::RAW)
        .expect("snaplen-truncated raw-IP frame must decode via the lax fallback")
    else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::LINUX_SLL)
        .expect("snaplen-truncated SLL frame must decode via the lax fallback")
    else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET).expect("must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::IPV6)
        .expect("snaplen-truncated IPv6 frame must decode via the lax fallback")
    else {
        panic!("expected IP DecodedFrame")
    };

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
    let DecodedFrame::Ip(parsed) = decode_packet(truncated, DataLink::ETHERNET)
        .expect("a frame truncated mid-TCP-header must still decode its IP layer")
    else {
        panic!("expected IP DecodedFrame")
    };

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

// ---------------------------------------------------------------------------
// AC-003 / BC-2.02.009 postcondition 3 (Path 3: non-IP non-ARP unchanged)
//
// STORY-111 v1.3 — renamed from `test_decode_arp_frame_reports_no_ip_layer`.
// The old test included an ARP subtest which is now STORY-112 territory.
// This test exercises ONLY non-IP non-ARP frames (LLDP EtherType 0x88CC and
// custom EtherType 0x9000). Both must return Err containing "No IP layer found".
// ARP frames (EtherType 0x0806) are excluded: their routing is handled by the
// STORY-111 ARP arm stub; their full behavioral assertion belongs to STORY-112.
//
// BC-2.02.009 postcondition 3 (Path 3): when slice.net == None after a
// successful strict parse of a non-IP non-ARP frame, decode_packet returns
// Err(anyhow!("No IP layer found")).
// ---------------------------------------------------------------------------
#[test]
fn test_decode_non_ip_non_arp_frame_returns_no_ip_error() {
    // LLDP frame: EtherType 0x88CC — non-IP, non-ARP. etherparse parses the
    // Ethernet header cleanly but net == None; the strict-path None arm fires.
    let lldp_frame = vec![
        0x01, 0x80, 0xc2, 0x00, 0x00, 0x0e, // dst mac (LLDP multicast)
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x88, 0xcc, // EtherType: LLDP (0x88CC)
        // Minimal LLDP TLV payload (4 bytes — enough to form a parseable frame)
        0x02, 0x07, 0x04, 0x00, // Chassis ID TLV (type=1, length=7 truncated here)
    ];
    let lldp_err = decode_packet(&lldp_frame, DataLink::ETHERNET).unwrap_err();
    assert!(
        lldp_err.to_string().contains("No IP layer found"),
        "AC-003: LLDP frame (EtherType 0x88CC) must return Err containing \
         'No IP layer found' (BC-2.02.009 Path 3); got: {lldp_err}"
    );

    // Custom EtherType 0x9000 — non-IP, non-ARP. etherparse parses the
    // Ethernet header (14 bytes MAC+EtherType) and sets net=None.
    let custom_frame = vec![
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac (broadcast)
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x90, 0x00, // EtherType: 0x9000 (custom, non-IP, non-ARP)
        0x01, 0x02, 0x03, 0x04, // arbitrary payload
    ];
    let custom_err = decode_packet(&custom_frame, DataLink::ETHERNET).unwrap_err();
    assert!(
        custom_err.to_string().contains("No IP layer found"),
        "AC-003: custom EtherType 0x9000 frame must return Err containing \
         'No IP layer found' (BC-2.02.009 Path 3); got: {custom_err}"
    );
}

// ---------------------------------------------------------------------------
// AC-005b / BC-2.02.009 invariant 5 / VP-008
//
// STORY-111 v1.3 — NEW test (Red Gate for AC-005b).
//
// Verifies that `decode_packet` does NOT panic when called with a well-formed
// Ethernet/IPv4 ARP frame (EtherType 0x0806, htype=Ethernet, ptype=IPv4,
// hlen=6, plen=4). The result value is NOT asserted — any Ok or Err outcome
// is acceptable. Only a runtime panic constitutes a failure here.
//
// RED GATE: This test FAILS on the stub (commit 4e22ef9) because
// `extract_arp_frame` is `todo!()` which panics, and the strict Ok arm also
// has a `todo!()`. The implementer makes this test pass by replacing both
// `todo!()` bodies with non-panicking code (the placeholder `extract_arp_frame`
// returns `None`; the ARP arm maps `None` to a temporary
// `Err(anyhow!("ARP extraction not yet implemented"))`).
//
// Do NOT assert Ok(DecodedFrame::Arp(...)) with real field values — that is
// STORY-112 scope.
//
// Canonical test vector: EC-001 from STORY-111.md — 42-byte Ethernet/IPv4
// ARP Request input to `decode_packet`. BC-2.02.009 Invariant 5 / VP-008.
// ---------------------------------------------------------------------------
#[test]
fn test_decode_arp_shaped_input_does_not_panic() {
    // Well-formed 42-byte Ethernet/IPv4 ARP Request.
    // Layout:
    //   [0..6]   dst MAC (broadcast)
    //   [6..12]  src MAC (00:11:22:33:44:55)
    //   [12..14] EtherType: ARP (0x0806)
    //   [14..42] ARP payload: htype=0x0001 (Ethernet), ptype=0x0800 (IPv4),
    //            hlen=6, plen=4, oper=1 (Request), sender hw+ip, target hw+ip
    let arp_frame = vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst: broadcast
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src: 00:11:22:33:44:55
        0x08, 0x06, // EtherType: ARP
        // ARP payload (28 bytes)
        0x00, 0x01, // htype: Ethernet (1)
        0x08, 0x00, // ptype: IPv4 (0x0800)
        0x06, // hlen: 6
        0x04, // plen: 4
        0x00, 0x01, // oper: Request (1)
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // sender hw addr
        0xc0, 0xa8, 0x01, 0x0a, // sender proto addr: 192.168.1.10
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // target hw addr (unknown in Request)
        0xc0, 0xa8, 0x01, 0x01, // target proto addr: 192.168.1.1
    ];
    assert_eq!(arp_frame.len(), 42, "pre-condition: ARP frame is 42 bytes");

    // Use catch_unwind to detect a panic from the todo!() stub.
    // On the stub (4e22ef9), this PANICS (Red Gate — test must fail).
    // After the implementer ships the non-panicking placeholder, the call
    // returns Ok or Err without panic and this test passes.
    let result = std::panic::catch_unwind(|| decode_packet(&arp_frame, DataLink::ETHERNET));
    assert!(
        result.is_ok(),
        "AC-005b / VP-008: decode_packet must NOT panic on a well-formed ARP \
         frame (EtherType 0x0806, htype=Ethernet, ptype=IPv4, hlen=6, plen=4). \
         The STORY-111 non-panicking placeholder must route this to a non-panic \
         Err('ARP extraction not yet implemented'). \
         Got a panic: {:?}",
        result.unwrap_err()
    );
    // The result value is intentionally not asserted — any Ok or Err is acceptable.
    // Ok(DecodedFrame::Arp(...)) with real field values is STORY-112 territory.
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
