//! STORY-004 Phase 3 TDD — Packet Decoding: ICMP, Protocol::Other, app_protocol_hint
//!
//! Formalizes behavioral contracts BC-2.02.010 through BC-2.02.013.
//! Strategy: brownfield-formalization. Every test maps to an AC or edge-case
//! from STORY-004.md and a clause from the named BC.
//!
//! On the first run these are expected to PASS because the implementation
//! in `src/decoder.rs` already satisfies the BCs. Any failure indicates a
//! real gap that the implementer must close.
//!
//! Test naming convention: test_BC_S_SS_NNN_<assertion>()
//!
//! The BC-based naming pattern uses uppercase letters (BC-S.SS.NNN) which
//! violates Rust's snake_case convention. `#![allow(non_snake_case)]` is
//! necessary to satisfy both the factory naming mandate and CI's `-D warnings`.
#![allow(non_snake_case)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pcap_file::DataLink;
use wirerust::decoder::{DecodedFrame, ParsedPacket, Protocol, TransportInfo, decode_packet};

// ---------------------------------------------------------------------------
// Frame-building helpers
//
// Each helper builds a minimal but legally-parseable frame at the Ethernet or
// raw-IP layer. Checksums are zeroed (etherparse does not validate them by
// default for the strict or lax slicers).
// ---------------------------------------------------------------------------

/// ICMPv4 echo-request frame over Ethernet with a non-empty echo data section.
///
/// Layout:
///   Ethernet  (14 bytes): dst-mac, src-mac, EtherType=0x0800
///   IPv4      (20 bytes): IHL=5, proto=0x01 (ICMP), total_length=36
///   ICMPv4    (8 bytes):  type=8 (echo-request), code=0, checksum=0, id=1, seq=1
///   echo data (8 bytes):  classic timestamp/pattern payload (0x01..0x08)
///
/// The 8-byte echo body is present so that `pkt.payload.is_empty()` genuinely
/// tests BC-2.02.010 invariant 3 ("ICMP body bytes are NOT included in
/// ParsedPacket.payload") rather than being vacuously true because the frame
/// carried no body at all.
fn make_icmpv4_echo_request_eth() -> Vec<u8> {
    vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac (broadcast)
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00, // EtherType: IPv4
        // IPv4 header (20 bytes)
        0x45, // version=4, IHL=5
        0x00, // DSCP / ECN
        0x00, 0x24, // total_length = 36 (20 IP + 8 ICMP header + 8 echo data)
        0x00, 0x01, // identification
        0x00, 0x00, // flags / fragment offset
        0x40, // TTL = 64
        0x01, // protocol = 1 (ICMP)
        0x00, 0x00, // header checksum (zeroed; not validated by etherparse)
        0xc0, 0xa8, 0x01, 0x01, // src: 192.168.1.1
        0xc0, 0xa8, 0x01, 0x02, // dst: 192.168.1.2
        // ICMPv4 echo-request header (8 bytes): type=8, code=0
        0x08, // type: echo-request
        0x00, // code: 0
        0x00, 0x00, // checksum (zeroed)
        0x00, 0x01, // identifier = 1
        0x00, 0x01, // sequence number = 1
        // echo data (8 bytes): classic timestamp/pattern bytes
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ]
}

/// Minimal ICMPv4 echo-reply frame over Ethernet.
///
/// Identical to the echo-request except ICMP type=0 (echo-reply).
fn make_icmpv4_echo_reply_eth() -> Vec<u8> {
    let mut pkt = make_icmpv4_echo_request_eth();
    // ICMP type byte is at offset 34 (14 Ethernet + 20 IPv4)
    pkt[34] = 0x00; // type: echo-reply
    pkt
}

/// ICMPv6 echo-request frame as a raw IPv6 packet (DataLink::IPV6) with a
/// non-empty echo data section.
///
/// Layout:
///   IPv6      (40 bytes): next-header=0x3A (58 = ICMPv6), payload_length=16
///   ICMPv6    (8 bytes):  type=128 (echo-request), code=0, checksum=0, id=1, seq=1
///   echo data (8 bytes):  classic timestamp/pattern payload (0x01..0x08)
///
/// The 8-byte echo body is present so that `pkt.payload.is_empty()` genuinely
/// tests BC-2.02.010 invariant 3 ("ICMP body bytes are NOT included in
/// ParsedPacket.payload") rather than being vacuously true.
fn make_icmpv6_echo_request_raw() -> Vec<u8> {
    vec![
        // IPv6 header (40 bytes)
        0x60, 0x00, 0x00, 0x00, // version=6, traffic-class=0, flow-label=0
        0x00, 0x10, // payload_length = 16 (8 ICMPv6 header + 8 echo data)
        0x3a, // next_header = 58 (ICMPv6)
        0x40, // hop_limit = 64
        // src: 2001:db8::1
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, // dst: 2001:db8::2
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
        // ICMPv6 echo-request header (8 bytes): type=128, code=0
        0x80, // type: 128 (echo-request)
        0x00, // code: 0
        0x00, 0x00, // checksum (zeroed)
        0x00, 0x01, // identifier = 1
        0x00, 0x01, // sequence number = 1
        // echo data (8 bytes): classic timestamp/pattern bytes
        0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08,
    ]
}

/// Minimal ICMPv6 neighbor-solicitation frame as a raw IPv6 packet (DataLink::IPV6).
///
/// Type=135 (neighbor solicitation), code=0.
fn make_icmpv6_neighbor_solicitation_raw() -> Vec<u8> {
    let mut pkt = make_icmpv6_echo_request_raw();
    // ICMPv6 type is at offset 40 (40-byte IPv6 header)
    pkt[40] = 0x87; // type: 135 (neighbor solicitation)
    pkt
}

/// GRE (IP protocol 47) frame over Ethernet with a non-empty protocol body.
///
/// Layout:
///   Ethernet    (14 bytes): EtherType=0x0800
///   IPv4        (20 bytes): proto=0x2F (47 = GRE), total_length=28
///   GRE header  (4 bytes):  minimal header (flags=0, protocol-type=0)
///   GRE payload (4 bytes):  arbitrary encapsulated-protocol body bytes
///
/// The 4-byte body is present so that `pkt.payload.is_empty()` genuinely
/// tests BC-2.02.011 PC3 ("payload is Vec::new()") rather than being
/// vacuously true because the frame carried no body at all.
fn make_gre_eth() -> Vec<u8> {
    vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00, // EtherType: IPv4
        // IPv4 header (20 bytes)
        0x45, // version=4, IHL=5
        0x00, // DSCP / ECN
        0x00, 0x1c, // total_length = 28 (20 IP + 4 GRE header + 4 body)
        0x00, 0x02, // identification
        0x00, 0x00, // flags / fragment offset
        0x40, // TTL = 64
        0x2f, // protocol = 47 (GRE)
        0x00, 0x00, // header checksum (zeroed)
        0x0a, 0x00, 0x00, 0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
        // GRE header (4 bytes): flags=0, protocol-type=0
        0x00, 0x00, // flags and version
        0x00, 0x00, // protocol type
        // GRE payload body (4 bytes): arbitrary encapsulated bytes
        0xde, 0xad, 0xbe, 0xef,
    ]
}

/// ESP (IP protocol 50) frame over Ethernet with a non-empty ciphertext body.
///
/// Layout:
///   Ethernet      (14 bytes): EtherType=0x0800
///   IPv4          (20 bytes): proto=0x32 (50 = ESP), total_length=28
///   ESP SPI       (4 bytes):  Security Parameter Index
///   ESP ciphertext (4 bytes): arbitrary encrypted-payload body bytes
///
/// The 4-byte ciphertext body is present so that `pkt.payload.is_empty()`
/// genuinely tests BC-2.02.011 PC3 ("payload is Vec::new()") rather than
/// being vacuously true because the frame carried no body at all.
fn make_esp_eth() -> Vec<u8> {
    vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00, // EtherType: IPv4
        // IPv4 header (20 bytes)
        0x45, // version=4, IHL=5
        0x00, // DSCP / ECN
        0x00, 0x1c, // total_length = 28 (20 IP + 4 ESP SPI + 4 ciphertext)
        0x00, 0x03, // identification
        0x00, 0x00, // flags / fragment offset
        0x40, // TTL = 64
        0x32, // protocol = 50 (ESP)
        0x00, 0x00, // header checksum (zeroed)
        0x0a, 0x00, 0x00, 0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
        // ESP SPI (4 bytes)
        0x00, 0x00, 0x00, 0x01,
        // ESP ciphertext body (4 bytes): arbitrary encrypted bytes
        0xca, 0xfe, 0xba, 0xbe,
    ]
}

/// Build a `ParsedPacket` directly with a TCP transport carrying the given
/// src/dst ports. Used for app_protocol_hint port-table tests where we want
/// to bypass the byte-level decode and exercise the hint logic in isolation.
fn tcp_packet_with_ports(src_port: u16, dst_port: u16) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            seq_number: 0,
            syn: false,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: Vec::new(),
        packet_len: 54,
    }
}

/// Build a `ParsedPacket` directly with a UDP transport carrying the given
/// src/dst ports.
fn udp_packet_with_ports(src_port: u16, dst_port: u16) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Udp,
        transport: TransportInfo::Udp { src_port, dst_port },
        payload: Vec::new(),
        packet_len: 42,
    }
}

/// Build a `ParsedPacket` with `TransportInfo::None` (ICMP / Other).
fn none_transport_packet(protocol: Protocol) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol,
        transport: TransportInfo::None,
        payload: Vec::new(),
        packet_len: 42,
    }
}

// ===========================================================================
// AC-001 — BC-2.02.010 postcondition 1
//
// An IPv4 ICMP echo-request frame decoded via decode_packet produces
// ParsedPacket { protocol: Protocol::Icmp, transport: TransportInfo::None, payload: [] }.
// ===========================================================================
#[test]
fn test_BC_2_02_010_icmpv4_protocol_icmp() {
    let data = make_icmpv4_echo_request_eth();
    let DecodedFrame::Ip(pkt) = decode_packet(&data, DataLink::ETHERNET)
        .expect("ICMPv4 echo-request Ethernet frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        pkt.protocol,
        Protocol::Icmp,
        "BC-2.02.010 PC1: ICMPv4 must produce Protocol::Icmp"
    );
    assert!(
        matches!(pkt.transport, TransportInfo::None),
        "BC-2.02.010 PC2: ICMPv4 must produce TransportInfo::None"
    );
    assert!(
        pkt.payload.is_empty(),
        "BC-2.02.010 PC3: ICMP payload must be empty Vec"
    );
    assert_eq!(
        pkt.src_ip,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)),
        "src IP must be preserved"
    );
    assert_eq!(
        pkt.dst_ip,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 2)),
        "dst IP must be preserved"
    );
}

// ===========================================================================
// AC-002 — BC-2.02.010 invariant 1
//
// Both ICMPv4 (IP protocol 1) and ICMPv6 (IP protocol 58) produce
// Protocol::Icmp — there is no Protocol::Icmpv6 variant.
// ===========================================================================
#[test]
fn test_BC_2_02_010_icmpv4_and_icmpv6_both_produce_protocol_icmp() {
    // ICMPv4 over Ethernet
    let icmpv4_data = make_icmpv4_echo_request_eth();
    let DecodedFrame::Ip(icmpv4_pkt) =
        decode_packet(&icmpv4_data, DataLink::ETHERNET).expect("ICMPv4 frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        icmpv4_pkt.protocol,
        Protocol::Icmp,
        "BC-2.02.010 invariant 1: ICMPv4 must map to Protocol::Icmp"
    );
    assert!(matches!(icmpv4_pkt.transport, TransportInfo::None));

    // ICMPv6 over raw IPv6
    let icmpv6_data = make_icmpv6_echo_request_raw();
    let DecodedFrame::Ip(icmpv6_pkt) =
        decode_packet(&icmpv6_data, DataLink::IPV6).expect("ICMPv6 frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        icmpv6_pkt.protocol,
        Protocol::Icmp,
        "BC-2.02.010 invariant 1: ICMPv6 must also map to Protocol::Icmp (no separate Icmpv6 variant)"
    );
    assert!(
        matches!(icmpv6_pkt.transport, TransportInfo::None),
        "ICMPv6 must also produce TransportInfo::None"
    );

    // Confirm both resolve to the same discriminant (compile-time evidence that
    // Icmpv6 is not a separate variant)
    assert_eq!(icmpv4_pkt.protocol, icmpv6_pkt.protocol);
}

// ===========================================================================
// AC-003 — BC-2.02.010 postcondition 4
//
// app_protocol_hint() on an ICMP ParsedPacket (with TransportInfo::None)
// returns None.
// ===========================================================================
#[test]
fn test_BC_2_02_010_icmp_app_protocol_hint_none() {
    let data = make_icmpv4_echo_request_eth();
    let DecodedFrame::Ip(pkt) =
        decode_packet(&data, DataLink::ETHERNET).expect("ICMPv4 frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert!(
        matches!(pkt.transport, TransportInfo::None),
        "pre-condition: ICMP packet must have TransportInfo::None"
    );
    assert_eq!(
        pkt.app_protocol_hint(),
        None,
        "BC-2.02.010 PC4: app_protocol_hint must return None for ICMP"
    );
}

// ===========================================================================
// AC-004 — BC-2.02.011 postcondition 1
//
// A packet with IP protocol 47 (GRE) produces
// ParsedPacket { protocol: Protocol::Other(47), transport: TransportInfo::None, payload: [] }.
// ===========================================================================
#[test]
fn test_BC_2_02_011_gre_protocol_other() {
    let data = make_gre_eth();
    let DecodedFrame::Ip(pkt) = decode_packet(&data, DataLink::ETHERNET)
        .expect("GRE Ethernet frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        pkt.protocol,
        Protocol::Other(47),
        "BC-2.02.011 PC1: GRE (IP proto 47) must produce Protocol::Other(47)"
    );
    assert!(
        matches!(pkt.transport, TransportInfo::None),
        "BC-2.02.011 PC2: GRE packet must produce TransportInfo::None"
    );
    assert!(
        pkt.payload.is_empty(),
        "BC-2.02.011 PC3: GRE packet payload must be empty Vec"
    );
    assert_eq!(
        pkt.src_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        "BC-2.02.011 PC5: src IP must still be preserved for Other packets"
    );
    assert_eq!(
        pkt.dst_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        "BC-2.02.011 PC5: dst IP must still be preserved for Other packets"
    );
}

// ===========================================================================
// AC-005 — BC-2.02.011 invariant 1
//
// Protocol::Other(u8) preserves the raw IP protocol byte.
// GRE (proto 47) => Other(47); ESP (proto 50) => Other(50).
// ===========================================================================
#[test]
fn test_BC_2_02_011_protocol_other_preserves_byte() {
    let gre_data = make_gre_eth();
    let DecodedFrame::Ip(gre_pkt) =
        decode_packet(&gre_data, DataLink::ETHERNET).expect("GRE frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };
    assert_eq!(
        gre_pkt.protocol,
        Protocol::Other(47),
        "GRE (proto 47) must produce Other(47)"
    );

    let esp_data = make_esp_eth();
    let DecodedFrame::Ip(esp_pkt) =
        decode_packet(&esp_data, DataLink::ETHERNET).expect("ESP frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };
    assert_eq!(
        esp_pkt.protocol,
        Protocol::Other(50),
        "ESP (proto 50) must produce Other(50)"
    );
}

// ===========================================================================
// AC-006 — BC-2.02.012 postcondition 3
//
// app_protocol_hint() returns the correct service string for all 7 recognized
// ports (tested once as src-port and once as dst-port for each service).
// ===========================================================================
#[test]
fn test_BC_2_02_012_app_protocol_hint_all_seven_ports() {
    // Each entry: (src_port, dst_port, expected_hint)
    // The "src wins" sub-case uses a well-known port as src and an unknown port
    // as dst; the "dst wins" sub-case uses an unknown src and the known port as dst.
    let cases: &[(u16, u16, &str)] = &[
        // DNS (53)
        (53, 9999, "DNS"),
        (9999, 53, "DNS"),
        // HTTP (80)
        (80, 9999, "HTTP"),
        (9999, 80, "HTTP"),
        // TLS (443)
        (443, 9999, "TLS"),
        (9999, 443, "TLS"),
        // SSH (22)
        (22, 9999, "SSH"),
        (9999, 22, "SSH"),
        // SMB (445)
        (445, 9999, "SMB"),
        (9999, 445, "SMB"),
        // Modbus (502)
        (502, 9999, "Modbus"),
        (9999, 502, "Modbus"),
        // DNP3 (20000)
        (20000, 9999, "DNP3"),
        (9999, 20000, "DNP3"),
    ];

    for (src_port, dst_port, expected) in cases {
        // TCP variant
        let tcp_pkt = tcp_packet_with_ports(*src_port, *dst_port);
        assert_eq!(
            tcp_pkt.app_protocol_hint(),
            Some(*expected),
            "BC-2.02.012 PC3 [TCP src={src_port} dst={dst_port}]: expected Some({expected:?})"
        );

        // UDP variant — same port table must apply for UDP
        let udp_pkt = udp_packet_with_ports(*src_port, *dst_port);
        assert_eq!(
            udp_pkt.app_protocol_hint(),
            Some(*expected),
            "BC-2.02.012 PC3 [UDP src={src_port} dst={dst_port}]: expected Some({expected:?})"
        );
    }
}

// ===========================================================================
// AC-007 — BC-2.02.012 postcondition 2
//
// app_protocol_hint() returns None for any port not in the 7-entry table.
// ===========================================================================
#[test]
fn test_BC_2_02_012_app_protocol_hint_unknown_port_returns_none() {
    let pkt = tcp_packet_with_ports(9999, 8080);
    assert_eq!(
        pkt.app_protocol_hint(),
        None,
        "BC-2.02.012 PC2: unknown ports must return None"
    );

    // Also verify a UDP packet with unrecognized ports returns None
    let udp_pkt = udp_packet_with_ports(5555, 5555);
    assert_eq!(
        udp_pkt.app_protocol_hint(),
        None,
        "BC-2.02.012 EC-004: src=5555, dst=5555 must return None"
    );
}

// ===========================================================================
// AC-008 — BC-2.02.012 postcondition 4
//
// When both src and dst ports are known but different (src=80, dst=443),
// the first matching match arm wins: Some("HTTP") is returned (80 arm fires
// before 443 arm).
// ===========================================================================
#[test]
fn test_BC_2_02_012_app_protocol_hint_match_order() {
    // src=80 (HTTP), dst=443 (TLS) — HTTP arm must win
    let pkt = tcp_packet_with_ports(80, 443);
    assert_eq!(
        pkt.app_protocol_hint(),
        Some("HTTP"),
        "BC-2.02.012 PC4 / EC-003: src=80, dst=443 must return Some(\"HTTP\") \
         because the 80 arm precedes the 443 arm in match order"
    );
}

// ===========================================================================
// BC-2.02.012 postcondition 4 — non-adjacent arm ordering (MINOR-1 pin)
//
// When src=53 (DNS, arm 1) and dst=502 (Modbus, arm 6) both match, the DNS
// arm fires first and returns Some("DNS"). A regression that reordered the
// Modbus arm above the DNS arm would flip this result to Some("Modbus") and
// be caught here. This pins a second independent point of the match-order
// ordering beyond the adjacent 80/443 pair exercised in AC-008.
// ===========================================================================
#[test]
fn test_BC_2_02_012_app_protocol_hint_match_order_dns_beats_modbus() {
    // src=53 (DNS, arm 1), dst=502 (Modbus, arm 6) — DNS arm must win
    let pkt = tcp_packet_with_ports(53, 502);
    assert_eq!(
        pkt.app_protocol_hint(),
        Some("DNS"),
        "BC-2.02.012 PC4: src=53, dst=502 must return Some(\"DNS\") \
         because the DNS arm (arm 1) precedes the Modbus arm (arm 6)"
    );
}

// ===========================================================================
// AC-009 — BC-2.02.013 postcondition 1
//
// app_protocol_hint() returns None immediately when transport = TransportInfo::None,
// without consulting the port table.
// ===========================================================================
#[test]
fn test_BC_2_02_013_transport_none_returns_none_hint() {
    // Direct construction (BC-2.02.013 EC-003 canonical test vector)
    let pkt = none_transport_packet(Protocol::Icmp);
    assert_eq!(
        pkt.app_protocol_hint(),
        None,
        "BC-2.02.013 PC1: TransportInfo::None must always return None from app_protocol_hint"
    );

    // Also verify for a Protocol::Other packet
    let other_pkt = none_transport_packet(Protocol::Other(47));
    assert_eq!(
        other_pkt.app_protocol_hint(),
        None,
        "BC-2.02.013 PC1: Protocol::Other with TransportInfo::None must also return None"
    );
}

// ===========================================================================
// EC-001 — STORY-004 edge cases + BC-2.02.010 EC-002
//
// ICMPv4 echo reply (type 0) produces Protocol::Icmp, TransportInfo::None.
// ===========================================================================
#[test]
fn test_BC_2_02_010_EC_001_icmpv4_echo_reply_protocol_icmp() {
    let data = make_icmpv4_echo_reply_eth();
    let DecodedFrame::Ip(pkt) =
        decode_packet(&data, DataLink::ETHERNET).expect("ICMPv4 echo-reply frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(pkt.protocol, Protocol::Icmp);
    assert!(matches!(pkt.transport, TransportInfo::None));
    assert!(pkt.payload.is_empty());
}

// ===========================================================================
// EC-002 — STORY-004 edge cases + BC-2.02.010 EC-003
//
// ICMPv6 neighbor solicitation (type 135) produces Protocol::Icmp,
// TransportInfo::None. The src IP is IPv6.
// ===========================================================================
#[test]
fn test_BC_2_02_010_EC_002_icmpv6_neighbor_solicitation_protocol_icmp() {
    let data = make_icmpv6_neighbor_solicitation_raw();
    let DecodedFrame::Ip(pkt) = decode_packet(&data, DataLink::IPV6)
        .expect("ICMPv6 neighbor-solicitation frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        pkt.protocol,
        Protocol::Icmp,
        "ICMPv6 type 135 must produce Protocol::Icmp"
    );
    assert!(matches!(pkt.transport, TransportInfo::None));
    assert!(pkt.payload.is_empty());
    // BC-2.02.010 EC-004: src IP is V6
    assert!(
        matches!(pkt.src_ip, IpAddr::V6(_)),
        "ICMPv6 packet must carry IPv6 src address"
    );
    assert_eq!(
        pkt.src_ip,
        IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1))
    );
}

// ===========================================================================
// EC-003 — STORY-004 edge cases + BC-2.02.011 EC-001
//
// GRE (proto 47) produces Protocol::Other(47), TransportInfo::None.
// (Duplicate of AC-004, here as a named edge-case anchor per the story spec.)
// ===========================================================================
#[test]
fn test_BC_2_02_011_EC_003_gre_protocol_other_47() {
    let data = make_gre_eth();
    let DecodedFrame::Ip(pkt) =
        decode_packet(&data, DataLink::ETHERNET).expect("GRE frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(pkt.protocol, Protocol::Other(47));
    assert!(matches!(pkt.transport, TransportInfo::None));
}

// ===========================================================================
// EC-004 — STORY-004 edge cases + BC-2.02.011 EC-002
//
// ESP (proto 50) produces Protocol::Other(50), TransportInfo::None.
// (Duplicate of AC-005's ESP sub-case, here as a named edge-case anchor.)
// ===========================================================================
#[test]
fn test_BC_2_02_011_EC_004_esp_protocol_other_50() {
    let data = make_esp_eth();
    let DecodedFrame::Ip(pkt) =
        decode_packet(&data, DataLink::ETHERNET).expect("ESP frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(pkt.protocol, Protocol::Other(50));
    assert!(matches!(pkt.transport, TransportInfo::None));
}

// ===========================================================================
// EC-005 — STORY-004 edge cases + BC-2.02.012 EC-001
//
// src=53, dst=9999 => app_protocol_hint() returns Some("DNS").
// ===========================================================================
#[test]
fn test_BC_2_02_012_EC_005_src_53_dst_unknown_returns_dns() {
    let pkt = udp_packet_with_ports(53, 9999);
    assert_eq!(
        pkt.app_protocol_hint(),
        Some("DNS"),
        "BC-2.02.012 EC-001: src=53, dst=9999 must return Some(\"DNS\")"
    );
}

// ===========================================================================
// EC-006 — STORY-004 edge cases + BC-2.02.012 EC-002
//
// src=9999, dst=53 => app_protocol_hint() returns Some("DNS").
// ===========================================================================
#[test]
fn test_BC_2_02_012_EC_006_src_unknown_dst_53_returns_dns() {
    let pkt = udp_packet_with_ports(9999, 53);
    assert_eq!(
        pkt.app_protocol_hint(),
        Some("DNS"),
        "BC-2.02.012 EC-002: src=9999, dst=53 must return Some(\"DNS\")"
    );
}

// ===========================================================================
// EC-007 — STORY-004 edge cases + BC-2.02.012 EC-003 (complement direction)
//
// src=443, dst=80 => Some("HTTP").
//
// AC-008 already covers src=80, dst=443 (the (80, _) arm fires on src).
// This test exercises the complement: src=443 (TLS), dst=80 (HTTP). The match
// evaluates arm 2 as `(80, _) | (_, 80)`: the first alternative `(80, _)`
// fails (src=443 ≠ 80), but the second `(_, 80)` succeeds (dst=80). So arm 2
// still fires and returns Some("HTTP"), confirming that the 80 arm beats the
// 443 arm regardless of which side carries which port number.
// ===========================================================================
#[test]
fn test_BC_2_02_012_EC_007_src_443_dst_80_match_order_http_still_wins() {
    let pkt = tcp_packet_with_ports(443, 80);
    assert_eq!(
        pkt.app_protocol_hint(),
        Some("HTTP"),
        "BC-2.02.012 EC-003 complement: src=443, dst=80 must return Some(\"HTTP\") \
         because the (_, 80) alternative of the 80 arm fires before the 443 arm"
    );
}

// ===========================================================================
// EC-008 — STORY-004 edge cases + BC-2.02.013 invariant 1
//
// TransportInfo::None with any IP protocol => app_protocol_hint() always None.
// ===========================================================================
#[test]
fn test_BC_2_02_013_EC_008_transport_none_always_none_for_any_protocol() {
    let protocols = [
        Protocol::Icmp,
        Protocol::Other(47),
        Protocol::Other(50),
        Protocol::Other(0),
        Protocol::Other(255),
        // Defense-in-depth: Protocol::Tcp/Udp paired with TransportInfo::None is an
        // unreachable synthetic state in the real decoder (a truncated TCP transport
        // header decodes to Protocol::Other(6), not Protocol::Tcp — see BC-2.02.011
        // EC-003). These entries confirm that app_protocol_hint reads only
        // self.transport, so even a hypothetical Protocol/transport mismatch yields
        // None rather than attempting a port lookup on a missing transport.
        Protocol::Tcp,
        Protocol::Udp,
    ];

    for proto in &protocols {
        let pkt = none_transport_packet(*proto);
        assert_eq!(
            pkt.app_protocol_hint(),
            None,
            "BC-2.02.013 invariant 1: TransportInfo::None must always return None \
             regardless of Protocol variant (got non-None for {proto:?})"
        );
    }
}
