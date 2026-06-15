//! STORY-002 Phase 3 TDD — Packet Decoding: Ethernet, RAW/IPV4, IPv6 Link-Layer Paths
//!
//! This file formalizes the behavioral contracts BC-2.02.001 through BC-2.02.005.
//! Strategy: brownfield-formalization. Every test matches an AC from STORY-002.md and
//! a clause from the named BC. On the first run these are expected to PASS (the
//! implementation already satisfies the BCs); if any fail, a gap in `src/decoder.rs`
//! is flagged in the Phase 3 report.
//!
//! Test naming convention: test_BC_S_SS_NNN_<assertion>()
//!
//! ## Coverage overlap note
//!
//! `tests/decoder_tests.rs` contains pre-existing ad-hoc tests of the same
//! `src/decoder.rs` code paths. Those tests are retained unchanged under the
//! brownfield-formalization strategy: they provide regression continuity and are
//! not superseded by this file. This file adds BC-anchored, traceability-tagged
//! coverage that maps every behavioral-contract clause to a named test; the two
//! sets are complementary.
//!
//! The BC-based naming pattern uses uppercase letters (BC-S.SS.NNN) which
//! violates Rust's snake_case convention. `#![allow(non_snake_case)]` is
//! necessary to satisfy both the factory naming mandate and CI's `-D warnings`.
#![allow(non_snake_case)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pcap_file::DataLink;
use proptest::prelude::*;
use wirerust::decoder::{DecodedFrame, Protocol, TransportInfo, decode_packet};

// ---------------------------------------------------------------------------
// Frame builders — synthetic packet bytes constructed inline.
//
// All frames are hand-crafted Ethernet II / IPv4 / TCP (or UDP), or bare
// IPv4/IPv6 (no Ethernet header) for the RAW/IPV4/IPV6 link-type paths.
// No external fixture files are used.
// ---------------------------------------------------------------------------

/// Build a minimal Ethernet II / IPv4 / TCP frame.
///
/// Layout:
///   Ethernet header   : 14 bytes
///   IPv4 header       : 20 bytes  (IHL=5, no options)
///   TCP header        : 20 bytes  (data-offset=5)
///   Payload           : `payload` bytes (may be empty)
///
/// Arguments control the TCP control flags and payload so that individual
/// AC / edge-case tests can request the exact header state they need.
#[allow(clippy::too_many_arguments)]
fn make_eth_ipv4_tcp(
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack_num: u32,
    flags: u8, // TCP flags byte (RFC 793 offset 13): CWR ECE URG ACK PSH RST SYN FIN
    //           Bit positions (LSB=0): FIN=0 SYN=1 RST=2 ACK=4
    payload: &[u8],
) -> Vec<u8> {
    // Total IP length: 20 (IP hdr) + 20 (TCP hdr) + payload
    let ip_total: u16 = 20 + 20 + payload.len() as u16;

    let mut frame = Vec::new();

    // Ethernet II header (14 bytes)
    frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst MAC
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // src MAC
    frame.extend_from_slice(&[0x08, 0x00]); // EtherType: IPv4

    // IPv4 header (20 bytes)
    frame.push(0x45); // version=4, IHL=5
    frame.push(0x00); // DSCP/ECN
    frame.extend_from_slice(&ip_total.to_be_bytes()); // total length
    frame.extend_from_slice(&[0x00, 0x01]); // identification
    frame.extend_from_slice(&[0x00, 0x00]); // flags/fragment offset
    frame.push(0x40); // TTL = 64
    frame.push(0x06); // protocol = TCP
    frame.extend_from_slice(&[0x00, 0x00]); // checksum (zero; etherparse does not verify)
    frame.extend_from_slice(&src_ip); // source IP
    frame.extend_from_slice(&dst_ip); // destination IP

    // TCP header (20 bytes, data offset = 5)
    frame.extend_from_slice(&src_port.to_be_bytes());
    frame.extend_from_slice(&dst_port.to_be_bytes());
    frame.extend_from_slice(&seq.to_be_bytes()); // sequence number
    frame.extend_from_slice(&ack_num.to_be_bytes()); // acknowledgement number
    frame.push(0x50); // data offset = 5 (top nibble), reserved = 0
    frame.push(flags); // control flags
    frame.extend_from_slice(&[0xff, 0xff]); // window size
    frame.extend_from_slice(&[0x00, 0x00]); // checksum
    frame.extend_from_slice(&[0x00, 0x00]); // urgent pointer

    // Payload
    frame.extend_from_slice(payload);

    frame
}

/// Build a bare IPv4 / TCP frame (no Ethernet header) for DataLink::RAW or IPV4.
fn make_raw_ipv4_tcp(
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    flags: u8,
    payload: &[u8],
) -> Vec<u8> {
    let ip_total: u16 = 20 + 20 + payload.len() as u16;

    let mut frame = Vec::new();

    // IPv4 header (20 bytes)
    frame.push(0x45);
    frame.push(0x00);
    frame.extend_from_slice(&ip_total.to_be_bytes());
    frame.extend_from_slice(&[0x00, 0x01]);
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.push(0x40); // TTL
    frame.push(0x06); // TCP
    frame.extend_from_slice(&[0x00, 0x00]); // checksum
    frame.extend_from_slice(&src_ip);
    frame.extend_from_slice(&dst_ip);

    // TCP header (20 bytes)
    frame.extend_from_slice(&src_port.to_be_bytes());
    frame.extend_from_slice(&dst_port.to_be_bytes());
    frame.extend_from_slice(&seq.to_be_bytes());
    frame.extend_from_slice(&0u32.to_be_bytes()); // ack number
    frame.push(0x50); // data offset = 5
    frame.push(flags);
    frame.extend_from_slice(&[0xff, 0xff]);
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.extend_from_slice(&[0x00, 0x00]);

    frame.extend_from_slice(payload);

    frame
}

/// Build an Ethernet II / IPv4 / UDP frame.
fn make_eth_ipv4_udp(
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    payload: &[u8],
) -> Vec<u8> {
    // UDP total length: 8 (header) + payload
    let udp_len: u16 = 8 + payload.len() as u16;
    let ip_total: u16 = 20 + udp_len;

    let mut frame = Vec::new();

    // Ethernet II (14 bytes)
    frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    frame.extend_from_slice(&[0x08, 0x00]);

    // IPv4 (20 bytes)
    frame.push(0x45);
    frame.push(0x00);
    frame.extend_from_slice(&ip_total.to_be_bytes());
    frame.extend_from_slice(&[0x00, 0x02]);
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.push(0x40); // TTL
    frame.push(0x11); // UDP
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.extend_from_slice(&src_ip);
    frame.extend_from_slice(&dst_ip);

    // UDP (8 bytes)
    frame.extend_from_slice(&src_port.to_be_bytes());
    frame.extend_from_slice(&dst_port.to_be_bytes());
    frame.extend_from_slice(&udp_len.to_be_bytes());
    frame.extend_from_slice(&[0x00, 0x00]); // checksum

    frame.extend_from_slice(payload);

    frame
}

/// Build a bare IPv6 / TCP frame (no Ethernet header) for DataLink::RAW or IPV6.
fn make_raw_ipv6_tcp(
    src_ip: [u16; 8],
    dst_ip: [u16; 8],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    flags: u8,
    payload: &[u8],
) -> Vec<u8> {
    // IPv6 payload length: 20 (TCP header) + payload
    let payload_len: u16 = 20 + payload.len() as u16;

    let [pl_hi, pl_lo] = payload_len.to_be_bytes();
    // IPv6 fixed header (40 bytes): version/TC/flow, payload len, next-hdr=TCP, hop limit
    let mut frame = vec![
        0x60u8, 0x00, 0x00, 0x00, // version=6, traffic class, flow label
        pl_hi, pl_lo, // payload length
        0x06,  // next header = TCP
        0x40,  // hop limit = 64
    ];

    // Source IPv6 address (16 bytes)
    for seg in &src_ip {
        frame.extend_from_slice(&seg.to_be_bytes());
    }
    // Destination IPv6 address (16 bytes)
    for seg in &dst_ip {
        frame.extend_from_slice(&seg.to_be_bytes());
    }

    // TCP header (20 bytes)
    frame.extend_from_slice(&src_port.to_be_bytes());
    frame.extend_from_slice(&dst_port.to_be_bytes());
    frame.extend_from_slice(&seq.to_be_bytes());
    frame.extend_from_slice(&0u32.to_be_bytes()); // ack number
    frame.push(0x50); // data offset = 5
    frame.push(flags);
    frame.extend_from_slice(&[0xff, 0xff]);
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.extend_from_slice(&[0x00, 0x00]);

    frame.extend_from_slice(payload);

    frame
}

/// Build a bare IPv6 / UDP frame (no Ethernet header) for DataLink::RAW or IPV6.
///
/// Used by BC-2.02.005 EC-002 coverage (IPv6 + UDP path).
fn make_raw_ipv6_udp(
    src_ip: [u16; 8],
    dst_ip: [u16; 8],
    src_port: u16,
    dst_port: u16,
    payload: &[u8],
) -> Vec<u8> {
    // UDP total length: 8 (header) + payload
    let udp_len: u16 = 8 + payload.len() as u16;
    // IPv6 payload length = UDP header + UDP payload
    let ipv6_payload_len = udp_len;

    let [pl_hi, pl_lo] = ipv6_payload_len.to_be_bytes();
    // IPv6 fixed header (40 bytes): next-header = 17 (UDP)
    let mut frame = vec![
        0x60u8, 0x00, 0x00, 0x00, // version=6, traffic class, flow label
        pl_hi, pl_lo, // payload length
        0x11,  // next header = UDP
        0x40,  // hop limit = 64
    ];

    // Source IPv6 address (16 bytes)
    for seg in &src_ip {
        frame.extend_from_slice(&seg.to_be_bytes());
    }
    // Destination IPv6 address (16 bytes)
    for seg in &dst_ip {
        frame.extend_from_slice(&seg.to_be_bytes());
    }

    // UDP header (8 bytes)
    frame.extend_from_slice(&src_port.to_be_bytes());
    frame.extend_from_slice(&dst_port.to_be_bytes());
    frame.extend_from_slice(&udp_len.to_be_bytes());
    frame.extend_from_slice(&[0x00, 0x00]); // checksum (zero; not verified by etherparse)

    // UDP payload
    frame.extend_from_slice(payload);

    frame
}

/// Build a bare IPv6 / ICMPv6 Echo Request frame for DataLink::RAW or IPV6.
///
/// Used by BC-2.02.005 EC-003 coverage (IPv6 + ICMPv6 path).
/// ICMPv6 Echo Request: type=128, code=0, checksum=0, id=1, seq=1.
fn make_raw_ipv6_icmpv6(src_ip: [u16; 8], dst_ip: [u16; 8]) -> Vec<u8> {
    // ICMPv6 Echo Request body: type(1) + code(1) + checksum(2) + id(2) + seq(2) = 8 bytes
    let icmpv6_body: [u8; 8] = [
        0x80, // type = 128 (Echo Request)
        0x00, // code = 0
        0x00, 0x00, // checksum (zero; not verified by etherparse)
        0x00, 0x01, // identifier = 1
        0x00, 0x01, // sequence = 1
    ];

    let ipv6_payload_len: u16 = icmpv6_body.len() as u16;
    let [pl_hi, pl_lo] = ipv6_payload_len.to_be_bytes();

    // IPv6 fixed header (40 bytes): next-header = 58 (ICMPv6)
    let mut frame = vec![
        0x60u8, 0x00, 0x00, 0x00, // version=6, TC, flow label
        pl_hi, pl_lo, // payload length
        0x3a,  // next header = ICMPv6
        0x40,  // hop limit = 64
    ];

    for seg in &src_ip {
        frame.extend_from_slice(&seg.to_be_bytes());
    }
    for seg in &dst_ip {
        frame.extend_from_slice(&seg.to_be_bytes());
    }

    frame.extend_from_slice(&icmpv6_body);

    frame
}

// TCP flags byte constants (RFC 793 bit positions in the 8-bit flags field).
// Byte layout (offset 13): CWR ECE URG ACK PSH RST SYN FIN
// Bit positions (LSB=0):   FIN=0 SYN=1 RST=2 PSH=3 ACK=4 URG=5 ECE=6 CWR=7
const TCP_FIN: u8 = 0x01;
const TCP_SYN: u8 = 0x02;
const TCP_RST: u8 = 0x04;
const TCP_ACK: u8 = 0x10;
const TCP_SYN_ACK: u8 = TCP_SYN | TCP_ACK;
const TCP_PURE_ACK: u8 = TCP_ACK;

// ---------------------------------------------------------------------------
// AC-001 / BC-2.02.001 postcondition 2, 3, 4, 5
//
// Given a valid Ethernet II / IPv4 / TCP frame and datalink = ETHERNET,
// decode_packet returns Ok(ParsedPacket) with:
//   - src_ip / dst_ip as IpAddr::V4 matching the IPv4 header
//   - protocol = Protocol::Tcp
//   - transport = TransportInfo::Tcp with correct port and flag values
//   - payload contains the TCP segment payload bytes
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_001_ethernet_ipv4_tcp_decode() {
    // Canonical test vector: Ethernet/IPv4/TCP SYN with a 4-byte payload.
    let payload = [0xde, 0xad, 0xbe, 0xef];
    let src_ip_bytes = [192, 168, 1, 10];
    let dst_ip_bytes = [10, 0, 0, 1];
    let src_port: u16 = 54321;
    let dst_port: u16 = 443;
    let seq: u32 = 0x00_00_00_01;

    let data = make_eth_ipv4_tcp(
        src_ip_bytes,
        dst_ip_bytes,
        src_port,
        dst_port,
        seq,
        0,
        TCP_SYN,
        &payload,
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("valid Ethernet/IPv4/TCP frame must decode without error")
    else {
        panic!("expected IP frame")
    };

    // BC-2.02.001 postcondition 2: IpAddr::V4 matching IPv4 header
    assert_eq!(
        parsed.src_ip,
        IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)),
        "src_ip must be IpAddr::V4(192.168.1.10)"
    );
    assert_eq!(
        parsed.dst_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        "dst_ip must be IpAddr::V4(10.0.0.1)"
    );

    // BC-2.02.001 postcondition 3: Protocol::Tcp
    assert_eq!(parsed.protocol, Protocol::Tcp, "protocol must be Tcp");

    // BC-2.02.001 postcondition 4: TransportInfo::Tcp with correct values
    match &parsed.transport {
        TransportInfo::Tcp {
            src_port: sp,
            dst_port: dp,
            seq_number,
            syn,
            ack,
            fin,
            rst,
        } => {
            assert_eq!(*sp, src_port, "src_port must match frame bytes");
            assert_eq!(*dp, dst_port, "dst_port must match frame bytes");
            assert_eq!(*seq_number, seq, "seq_number must match frame bytes");
            assert!(*syn, "SYN flag must be set");
            assert!(!*ack, "ACK flag must not be set on pure SYN");
            assert!(!*fin, "FIN flag must not be set");
            assert!(!*rst, "RST flag must not be set");
        }
        other => panic!("Expected TransportInfo::Tcp, got: {other:?}"),
    }

    // BC-2.02.001 postcondition 5: payload contains TCP segment payload bytes
    assert_eq!(
        parsed.payload,
        payload.to_vec(),
        "payload must contain the TCP segment data"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.02.001 postcondition 6 + invariant 1
//
// For any successfully decoded frame, ParsedPacket.packet_len equals data.len()
// (the total frame byte length including all headers).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_001_packet_len_is_total_frame_length() {
    // Use several frame sizes to pin the invariant beyond a single case.
    let cases: &[(&[u8], &str)] = &[
        (&[], "empty payload"),
        (&[0xAA; 10], "10-byte payload"),
        (&[0xBB; 100], "100-byte payload"),
    ];

    for (payload, label) in cases {
        let data = make_eth_ipv4_tcp(
            [192, 168, 1, 10],
            [10, 0, 0, 1],
            12345,
            80,
            1,
            0,
            TCP_PURE_ACK,
            payload,
        );
        let expected_len = data.len();
        let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
            .unwrap_or_else(|e| panic!("decode failed for {label}: {e}"))
        else {
            panic!("expected IP frame for {label}")
        };
        assert_eq!(
            parsed.packet_len, expected_len,
            "packet_len must equal data.len() ({expected_len}) for {label}"
        );
    }
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.02.002 postcondition 2, 3, 4, 6
//
// Given Ethernet/IPv4/UDP with dst_port = 53, decode_packet returns Ok with
// protocol = Protocol::Udp (PC2), transport = TransportInfo::Udp { src_port, dst_port } (PC3),
// payload equal to the UDP body bytes (PC4), and app_protocol_hint() returns Some("DNS") (PC6).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_002_udp_dns_port_hint() {
    // BC-2.02.002 canonical test vector: dst_port = 53 (DNS query)
    let data = make_eth_ipv4_udp(
        [10, 0, 0, 1],
        [8, 8, 8, 8],
        55555, // src_port (ephemeral)
        53,    // dst_port = DNS
        b"dns-query-bytes",
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("Ethernet/IPv4/UDP frame must decode successfully")
    else {
        panic!("expected IP frame")
    };

    // BC-2.02.002 postcondition 2: Protocol::Udp
    assert_eq!(parsed.protocol, Protocol::Udp, "protocol must be Udp");

    // BC-2.02.002 postcondition 3: TransportInfo::Udp with correct ports
    match &parsed.transport {
        TransportInfo::Udp { src_port, dst_port } => {
            assert_eq!(*src_port, 55555, "src_port must be 55555");
            assert_eq!(*dst_port, 53, "dst_port must be 53 (DNS)");
        }
        other => panic!("Expected TransportInfo::Udp, got: {other:?}"),
    }

    // BC-2.02.002 postcondition 4: payload contains the UDP datagram body bytes
    assert_eq!(
        parsed.payload,
        b"dns-query-bytes".to_vec(),
        "payload must equal the UDP body bytes placed in the frame"
    );

    // BC-2.02.002 postcondition 6: app_protocol_hint returns Some("DNS") when dst_port = 53
    assert_eq!(
        parsed.app_protocol_hint(),
        Some("DNS"),
        "app_protocol_hint must return Some(\"DNS\") when dst_port = 53"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.02.002 postcondition 2, 6 (src_port = 53 direction)
//
// Given a UDP frame with src_port = 53 (DNS response direction),
// protocol is Protocol::Udp (PC2) and app_protocol_hint() returns Some("DNS") (PC6).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_002_udp_dns_src_port_hint() {
    // BC-2.02.002 canonical test vector: src_port = 53 (DNS response)
    let data = make_eth_ipv4_udp(
        [8, 8, 8, 8],
        [10, 0, 0, 1],
        53,    // src_port = DNS server responding
        55555, // dst_port = client ephemeral port
        b"dns-response-bytes",
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("DNS response frame must decode successfully")
    else {
        panic!("expected IP frame")
    };

    assert_eq!(parsed.protocol, Protocol::Udp);

    // BC-2.02.002 invariant 2: either src or dst port = 53 triggers DNS hint
    assert_eq!(
        parsed.app_protocol_hint(),
        Some("DNS"),
        "app_protocol_hint must return Some(\"DNS\") when src_port = 53"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.02.003 postcondition 2, 3, 4
//
// Given raw IPv4 TCP bytes (no link-layer header) with datalink = RAW,
// decode_packet returns Ok(ParsedPacket) where src_ip and dst_ip are
// IpAddr::V4 values (PC2), protocol is Protocol::Tcp (PC3), and
// TransportInfo::Tcp carries correct ports/flags/packet_len/payload (PC4).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_003_raw_ipv4_tcp_decode() {
    // BC-2.02.003 canonical test vector: RAW IPv4 TCP
    let src_ip_bytes = [172, 16, 0, 1];
    let dst_ip_bytes = [172, 16, 0, 2];
    let src_port: u16 = 8080;
    let dst_port: u16 = 9090;
    let payload = [0x01, 0x02, 0x03];

    let data = make_raw_ipv4_tcp(
        src_ip_bytes,
        dst_ip_bytes,
        src_port,
        dst_port,
        42,
        TCP_SYN,
        &payload,
    );

    let DecodedFrame::Ip(parsed) =
        decode_packet(&data, DataLink::RAW).expect("RAW IPv4 TCP frame must decode successfully")
    else {
        panic!("expected IP frame")
    };

    // BC-2.02.003 postcondition 2: IpAddr::V4 values
    assert_eq!(
        parsed.src_ip,
        IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1)),
        "RAW path: src_ip must be IpAddr::V4(172.16.0.1)"
    );
    assert_eq!(
        parsed.dst_ip,
        IpAddr::V4(Ipv4Addr::new(172, 16, 0, 2)),
        "RAW path: dst_ip must be IpAddr::V4(172.16.0.2)"
    );

    // BC-2.02.003 postcondition 3: Protocol::Tcp
    assert_eq!(parsed.protocol, Protocol::Tcp);

    // BC-2.02.003 postcondition 4: TransportInfo::Tcp with correct values
    match &parsed.transport {
        TransportInfo::Tcp {
            src_port: sp,
            dst_port: dp,
            syn,
            ..
        } => {
            assert_eq!(*sp, src_port);
            assert_eq!(*dp, dst_port);
            assert!(*syn, "SYN flag must be set");
        }
        other => panic!("Expected TransportInfo::Tcp, got: {other:?}"),
    }

    // BC-2.02.003 postcondition 4: packet_len equals data.len()
    assert_eq!(
        parsed.packet_len,
        data.len(),
        "RAW path: packet_len must equal data.len()"
    );

    // BC-2.02.003 postcondition 4: payload correct
    assert_eq!(parsed.payload, payload.to_vec());
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.02.004 postcondition 2, 3
//
// Calling decode_packet with the same IPv4 TCP byte slice using DataLink::RAW
// and then DataLink::IPV4 produces field-for-field identical ParsedPacket values.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_004_raw_and_ipv4_identical() {
    // BC-2.02.004 canonical test vector: same bytes, two DataLink variants
    let data = make_raw_ipv4_tcp(
        [10, 20, 30, 40],
        [50, 60, 70, 80],
        11111,
        22222,
        99,
        TCP_SYN_ACK,
        b"some-payload",
    );

    let DecodedFrame::Ip(from_raw) =
        decode_packet(&data, DataLink::RAW).expect("RAW decode must succeed")
    else {
        panic!("expected IP frame")
    };
    let DecodedFrame::Ip(from_ipv4) =
        decode_packet(&data, DataLink::IPV4).expect("IPV4 decode must succeed")
    else {
        panic!("expected IP frame")
    };

    // BC-2.02.004 postcondition 2: field-for-field identical results
    assert_eq!(
        from_raw.src_ip, from_ipv4.src_ip,
        "src_ip must be identical for RAW and IPV4"
    );
    assert_eq!(
        from_raw.dst_ip, from_ipv4.dst_ip,
        "dst_ip must be identical for RAW and IPV4"
    );
    assert_eq!(
        from_raw.protocol, from_ipv4.protocol,
        "protocol must be identical for RAW and IPV4"
    );
    assert_eq!(
        from_raw.packet_len, from_ipv4.packet_len,
        "packet_len must be identical for RAW and IPV4"
    );
    assert_eq!(
        from_raw.payload, from_ipv4.payload,
        "payload must be identical for RAW and IPV4"
    );

    // Transport fields must also match.
    match (&from_raw.transport, &from_ipv4.transport) {
        (
            TransportInfo::Tcp {
                src_port: sp_r,
                dst_port: dp_r,
                seq_number: seq_r,
                syn: syn_r,
                ack: ack_r,
                fin: fin_r,
                rst: rst_r,
            },
            TransportInfo::Tcp {
                src_port: sp_i,
                dst_port: dp_i,
                seq_number: seq_i,
                syn: syn_i,
                ack: ack_i,
                fin: fin_i,
                rst: rst_i,
            },
        ) => {
            assert_eq!(sp_r, sp_i, "src_port must match");
            assert_eq!(dp_r, dp_i, "dst_port must match");
            assert_eq!(seq_r, seq_i, "seq_number must match");
            assert_eq!(syn_r, syn_i, "syn flag must match");
            assert_eq!(ack_r, ack_i, "ack flag must match");
            assert_eq!(fin_r, fin_i, "fin flag must match");
            assert_eq!(rst_r, rst_i, "rst flag must match");
        }
        (a, b) => panic!("Both should be TransportInfo::Tcp; got RAW={a:?} IPV4={b:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.02.005 postcondition 2, 3
//
// Given raw IPv6 TCP bytes with datalink = RAW, decode_packet returns
// Ok(ParsedPacket) where src_ip is IpAddr::V6 with the correct address
// (PC2) and dst_ip is IpAddr::V6 with the correct address (PC3).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_005_raw_ipv6_tcp_decode() {
    // BC-2.02.005 canonical test vector: RAW/IPv6/TCP
    // Addresses: 2001:db8::1 -> 2001:db8::2
    let src_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 1];
    let dst_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 2];

    let data = make_raw_ipv6_tcp(src_segs, dst_segs, 50000, 443, 1, TCP_SYN, &[]);

    let DecodedFrame::Ip(parsed) =
        decode_packet(&data, DataLink::RAW).expect("RAW IPv6 TCP frame must decode successfully")
    else {
        panic!("expected IP frame")
    };

    // BC-2.02.005 postcondition 2: src_ip is IpAddr::V6
    match parsed.src_ip {
        IpAddr::V6(addr) => assert_eq!(
            addr,
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1),
            "src_ip must be 2001:db8::1"
        ),
        IpAddr::V4(_) => panic!("src_ip must be IpAddr::V6 for IPv6 packet, got V4"),
    }

    // BC-2.02.005 postcondition 3: dst_ip is IpAddr::V6
    match parsed.dst_ip {
        IpAddr::V6(addr) => assert_eq!(
            addr,
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 2),
            "dst_ip must be 2001:db8::2"
        ),
        IpAddr::V4(_) => panic!("dst_ip must be IpAddr::V6 for IPv6 packet, got V4"),
    }
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.02.005 postcondition 4, 5, 6
//
// For an IPv6 TCP frame, protocol = Protocol::Tcp, transport =
// TransportInfo::Tcp { ... } with correct port and flag values,
// and packet_len equals data.len().
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_005_ipv6_tcp_transport() {
    let src_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 10];
    let dst_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 20];
    let src_port: u16 = 60000;
    let dst_port: u16 = 8080;
    let seq: u32 = 0xDEAD_BEEF;

    let data = make_raw_ipv6_tcp(
        src_segs, dst_segs, src_port, dst_port, seq, TCP_SYN, b"hello",
    );

    let DecodedFrame::Ip(parsed) =
        decode_packet(&data, DataLink::RAW).expect("RAW IPv6 TCP frame must decode successfully")
    else {
        panic!("expected IP frame")
    };

    // BC-2.02.005 postcondition 4: Protocol::Tcp
    assert_eq!(
        parsed.protocol,
        Protocol::Tcp,
        "protocol must be Tcp for IPv6/TCP"
    );

    // BC-2.02.005 postcondition 5: TransportInfo::Tcp with correct values
    match &parsed.transport {
        TransportInfo::Tcp {
            src_port: sp,
            dst_port: dp,
            seq_number,
            syn,
            ack,
            ..
        } => {
            assert_eq!(*sp, src_port, "src_port must match");
            assert_eq!(*dp, dst_port, "dst_port must match");
            assert_eq!(*seq_number, seq, "seq_number must match");
            assert!(*syn, "SYN flag must be set");
            assert!(!*ack, "ACK flag must not be set on pure SYN");
        }
        other => panic!("Expected TransportInfo::Tcp for IPv6, got: {other:?}"),
    }

    // BC-2.02.005 postcondition 6: packet_len equals data.len()
    assert_eq!(
        parsed.packet_len,
        data.len(),
        "packet_len must equal data.len() for IPv6 TCP"
    );
}

// ---------------------------------------------------------------------------
// Story Task 8 / BC-2.02.001 invariant 1 + BC-2.02.005 postcondition 6
//
// Property-based test: packet_len == data.len() for all successfully decoded
// frames across all supported link types and packet sizes. Generates 1000
// cases using proptest.
//
// The generator covers Ethernet/IPv4/TCP, RAW/IPv4/TCP, and RAW/IPv6/TCP
// frames with randomised payload sizes in 0..=255 bytes. The packet_len
// invariant must hold for every frame that decodes without error.
// ---------------------------------------------------------------------------
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 1000,
        ..ProptestConfig::default()
    })]

    #[test]
    fn test_BC_2_02_001_proptest_packet_len_equals_data_len(
        payload_size in 0usize..=255usize,
        // Select link path: 0 = Ethernet/IPv4, 1 = RAW/IPv4, 2 = RAW/IPv6
        path in 0u8..=2u8,
    ) {
        let payload: Vec<u8> = vec![0xAB; payload_size];

        let (data, link) = match path {
            0 => (
                make_eth_ipv4_tcp(
                    [192, 168, 1, 1], [10, 0, 0, 1],
                    1234, 5678, 1, 0, TCP_PURE_ACK, &payload,
                ),
                DataLink::ETHERNET,
            ),
            1 => (
                make_raw_ipv4_tcp(
                    [172, 16, 0, 1], [172, 16, 0, 2],
                    4321, 8765, 2, TCP_SYN, &payload,
                ),
                DataLink::RAW,
            ),
            _ => (
                make_raw_ipv6_tcp(
                    [0x2001, 0x0db8, 0, 0, 0, 0, 0, 1],
                    [0x2001, 0x0db8, 0, 0, 0, 0, 0, 2],
                    9000, 80, 3, TCP_SYN, &payload,
                ),
                DataLink::RAW,
            ),
        };

        let expected_len = data.len();

        // Every frame produced by the builders above is well-formed; decode_packet
        // must succeed. If it returns Err the test fails loudly, catching regressions
        // that silently reject valid frames (n1 fix: no longer a silent no-op).
        let result = decode_packet(&data, link);
        prop_assert!(
            result.is_ok(),
            "decode_packet must succeed for well-formed frame (path={path}): {:?}",
            result.as_ref().unwrap_err()
        );
        let DecodedFrame::Ip(parsed) = result.unwrap() else {
            panic!("expected IP frame")
        };
        prop_assert_eq!(
            parsed.packet_len,
            expected_len,
            "packet_len ({}) must equal data.len() ({})",
            parsed.packet_len,
            expected_len
        );
    }
}

// ---------------------------------------------------------------------------
// Edge-case tests EC-001..EC-007 from STORY-002.md
// ---------------------------------------------------------------------------

/// Story EC-001 / BC-2.02.001 EC-001: TCP SYN packet (syn=true, ack=false, fin=false, rst=false).
///
/// TransportInfo::Tcp with exactly syn=true and all other control flags false.
#[test]
fn test_BC_2_02_001_ec001_tcp_syn_only_flags() {
    let data = make_eth_ipv4_tcp(
        [192, 168, 1, 1],
        [192, 168, 1, 2],
        45000,
        80,
        1234,
        0,
        TCP_SYN, // SYN only
        &[],
    );

    let DecodedFrame::Ip(parsed) =
        decode_packet(&data, DataLink::ETHERNET).expect("SYN frame must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            assert!(*syn, "EC-001: syn must be true");
            assert!(!*ack, "EC-001: ack must be false");
            assert!(!*fin, "EC-001: fin must be false");
            assert!(!*rst, "EC-001: rst must be false");
        }
        other => panic!("EC-001: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

/// Story EC-002 / BC-2.02.001 EC-003: TCP pure ACK (no payload) — payload is empty Vec; Ok returned.
#[test]
fn test_BC_2_02_001_ec003_tcp_pure_ack_empty_payload() {
    let data = make_eth_ipv4_tcp(
        [192, 168, 1, 1],
        [192, 168, 1, 2],
        45001,
        80,
        5000,
        1,
        TCP_PURE_ACK, // ACK only, no payload
        &[],
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("EC-002: pure ACK frame must decode successfully (Ok returned)")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        parsed.payload,
        Vec::<u8>::new(),
        "EC-002: payload must be empty for pure ACK frame"
    );

    match &parsed.transport {
        TransportInfo::Tcp {
            ack, syn, fin, rst, ..
        } => {
            assert!(*ack, "EC-002: ack flag must be set");
            assert!(!*syn, "EC-002: syn must not be set on pure ACK");
            assert!(!*fin, "EC-002: fin must not be set");
            assert!(!*rst, "EC-002: rst must not be set");
        }
        other => panic!("EC-002: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

/// Story EC-003 / BC-2.02.002 EC-005: UDP port = 80 → app_protocol_hint() returns Some("HTTP").
#[test]
fn test_BC_2_02_002_ec003_udp_port_80_http_hint() {
    let data = make_eth_ipv4_udp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        50000,
        80, // HTTP
        b"get-request",
    );

    let DecodedFrame::Ip(parsed) =
        decode_packet(&data, DataLink::ETHERNET).expect("EC-003: UDP port 80 frame must decode")
    else {
        panic!("expected IP frame")
    };

    assert_eq!(
        parsed.app_protocol_hint(),
        Some("HTTP"),
        "EC-003: app_protocol_hint must return Some(\"HTTP\") for port 80"
    );
}

/// Story EC-004 / BC-2.02.002 EC-003: UDP port = 9999 (unknown) → app_protocol_hint() returns None.
#[test]
fn test_BC_2_02_002_ec004_udp_unknown_port_no_hint() {
    let data = make_eth_ipv4_udp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        50001,
        9999, // unknown service
        b"some-data",
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("EC-004: UDP frame with unknown port must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    assert_eq!(
        parsed.app_protocol_hint(),
        None,
        "EC-004: app_protocol_hint must return None for unknown port 9999"
    );
}

/// Story EC-005 / BC-2.02.004 EC-001 / BC-2.02.003 EC-001: DataLink::IPV4 with IPv4 TCP — identical result to DataLink::RAW.
#[test]
fn test_BC_2_02_004_ec005_ipv4_datalink_identical_to_raw() {
    let data = make_raw_ipv4_tcp(
        [10, 0, 1, 1],
        [10, 0, 1, 2],
        31000,
        443,
        7,
        TCP_SYN,
        b"payload-ec005",
    );

    let DecodedFrame::Ip(from_raw) =
        decode_packet(&data, DataLink::RAW).expect("EC-005: RAW must decode")
    else {
        panic!("expected IP DecodedFrame")
    };
    let DecodedFrame::Ip(from_ipv4) =
        decode_packet(&data, DataLink::IPV4).expect("EC-005: IPV4 must decode")
    else {
        panic!("expected IP DecodedFrame")
    };

    // BC-2.02.004 postcondition 3: zero observable difference
    assert_eq!(
        from_raw.src_ip, from_ipv4.src_ip,
        "EC-005: src_ip must be identical"
    );
    assert_eq!(
        from_raw.dst_ip, from_ipv4.dst_ip,
        "EC-005: dst_ip must be identical"
    );
    assert_eq!(
        from_raw.protocol, from_ipv4.protocol,
        "EC-005: protocol must be identical"
    );
    assert_eq!(
        from_raw.payload, from_ipv4.payload,
        "EC-005: payload must be identical"
    );
    assert_eq!(
        from_raw.packet_len, from_ipv4.packet_len,
        "EC-005: packet_len must be identical"
    );
}

/// Story EC-006 / BC-2.02.005 EC-004: IPv6 loopback (::1) — decoded normally; IpAddr::V6(::1).
#[test]
fn test_BC_2_02_005_ec006_ipv6_loopback_decoded_normally() {
    // ::1 = [0, 0, 0, 0, 0, 0, 0, 1] as u16 segments
    let loopback: [u16; 8] = [0, 0, 0, 0, 0, 0, 0, 1];

    let data = make_raw_ipv6_tcp(loopback, loopback, 12345, 8080, 1, TCP_SYN, &[]);

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::RAW)
        .expect("EC-006: IPv6 loopback frame must decode normally")
    else {
        panic!("expected IP DecodedFrame")
    };

    match parsed.src_ip {
        IpAddr::V6(addr) => {
            assert_eq!(
                addr,
                Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
                "EC-006: src_ip must be ::1"
            );
        }
        IpAddr::V4(_) => panic!("EC-006: src_ip must be IpAddr::V6 for IPv6 loopback"),
    }
    match parsed.dst_ip {
        IpAddr::V6(addr) => {
            assert_eq!(
                addr,
                Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1),
                "EC-006: dst_ip must be ::1"
            );
        }
        IpAddr::V4(_) => panic!("EC-006: dst_ip must be IpAddr::V6 for IPv6 loopback"),
    }

    assert_eq!(
        parsed.protocol,
        Protocol::Tcp,
        "EC-006: protocol must be Tcp"
    );
}

/// Story EC-007 / BC-2.02.005 EC-005: IPv6 with extension headers — etherparse traverses them; TCP surfaced.
///
/// Extension header tested: Hop-By-Hop Options (next-header=0) prefixed before TCP.
///
/// This test constructs a minimal IPv6 + Hop-By-Hop (next header = 0) + TCP frame.
/// The HBH header has next-header = TCP (6) and a pad2 option (one pad byte), making
/// the extension header 8 bytes total (the minimum unit).
#[test]
fn test_BC_2_02_005_ec007_ipv6_extension_headers_tcp_surfaced() {
    // IPv6 Hop-By-Hop Options header layout (8 bytes = minimum):
    //   byte 0: next header = 6 (TCP)
    //   byte 1: hdr ext len = 0 (meaning 8 bytes total including these 2)
    //   bytes 2-7: options (pad6 = PadN with len 4, or 6 x Pad1)
    //   Pad1 option = 0x00 (type byte only, no length/value)
    let hbh: [u8; 8] = [
        0x06, // next header = TCP
        0x00, // hdr ext len = 0 (8 bytes total)
        0x00, // Pad1
        0x00, // Pad1
        0x00, // Pad1
        0x00, // Pad1
        0x00, // Pad1
        0x00, // Pad1
    ];

    let src_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 5];
    let dst_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 6];
    let src_port: u16 = 44444;
    let dst_port: u16 = 22;
    let seq: u32 = 100;

    // Build TCP header manually
    let tcp: [u8; 20] = {
        let mut t = [0u8; 20];
        t[0..2].copy_from_slice(&src_port.to_be_bytes());
        t[2..4].copy_from_slice(&dst_port.to_be_bytes());
        t[4..8].copy_from_slice(&seq.to_be_bytes());
        // ack = 0
        t[12] = 0x50; // data offset = 5
        t[13] = TCP_SYN;
        t[14] = 0xff; // window high
        t[15] = 0xff; // window low
        t
    };

    // IPv6 payload = HBH extension header (8 bytes) + TCP header (20 bytes)
    let payload_len: u16 = (hbh.len() + tcp.len()) as u16;

    let [pl_hi, pl_lo] = payload_len.to_be_bytes();
    // IPv6 fixed header (40 bytes): next-header = 0 (Hop-by-Hop Options)
    let mut frame = vec![
        0x60u8, 0x00, 0x00, 0x00, // version=6, TC, flow label
        pl_hi, pl_lo, // payload length
        0x00,  // next header = Hop-by-Hop Options
        0x40,  // hop limit
    ];

    for seg in &src_segs {
        frame.extend_from_slice(&seg.to_be_bytes());
    }
    for seg in &dst_segs {
        frame.extend_from_slice(&seg.to_be_bytes());
    }

    // Hop-by-Hop extension header
    frame.extend_from_slice(&hbh);

    // TCP header
    frame.extend_from_slice(&tcp);

    let DecodedFrame::Ip(parsed) = decode_packet(&frame, DataLink::RAW)
        .expect("EC-007: IPv6 frame with HBH extension header must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    // etherparse traverses extension headers; TCP must be surfaced
    assert_eq!(
        parsed.protocol,
        Protocol::Tcp,
        "EC-007: Protocol must be Tcp after traversing extension headers"
    );
    match &parsed.transport {
        TransportInfo::Tcp {
            src_port: sp,
            dst_port: dp,
            syn,
            ..
        } => {
            assert_eq!(*sp, src_port, "EC-007: src_port must match");
            assert_eq!(*dp, dst_port, "EC-007: dst_port must match");
            assert!(*syn, "EC-007: SYN must be set");
        }
        other => {
            panic!("EC-007: expected TransportInfo::Tcp after extension headers, got: {other:?}")
        }
    }
}

// ---------------------------------------------------------------------------
// Additional port-table completeness test
//
// Story Task 7: verify the app_protocol_hint 7-entry port table is complete:
// ports 53 (DNS), 80 (HTTP), 443 (TLS), 22 (SSH), 445 (SMB), 502 (Modbus),
// 20000 (DNP3).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_002_app_protocol_hint_full_port_table() {
    let port_hints: &[(u16, &str)] = &[
        (53, "DNS"),
        (80, "HTTP"),
        (443, "TLS"),
        (22, "SSH"),
        (445, "SMB"),
        (502, "Modbus"),
        (20000, "DNP3"),
    ];

    for (port, expected_hint) in port_hints {
        // Use UDP dst_port for each entry; src-port checking is covered by AC-004.
        let data = make_eth_ipv4_udp([10, 0, 0, 1], [10, 0, 0, 2], 55000, *port, b"probe");
        let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
            .unwrap_or_else(|e| panic!("port {port} frame decode failed: {e}"))
        else {
            panic!("expected IP frame for port {port}")
        };

        assert_eq!(
            parsed.app_protocol_hint(),
            Some(*expected_hint),
            "port {port} must map to hint \"{}\"",
            expected_hint
        );
    }
}

// ---------------------------------------------------------------------------
// m1 — BC-2.02.005 EC-002: RAW IPv6 / UDP with dst_port = 53
//
// The BC-2.02.005 canonical test vector "RAW/IPv6/UDP dst_port=53 → Ok, Udp,
// app_hint=Some(\"DNS\")" was previously unexercised — all IPv6 tests used TCP.
// This test adds the IPv6 UDP decode path including the DNS port hint.
// ---------------------------------------------------------------------------

/// BC-2.02.005 EC-002: RAW IPv6 / UDP frame decodes to IpAddr::V6 addresses,
/// Protocol::Udp, and app_protocol_hint() returns Some("DNS") for dst_port=53.
#[test]
fn test_BC_2_02_005_ec002_raw_ipv6_udp_dns_hint() {
    // BC-2.02.005 canonical test vector: RAW/IPv6/UDP dst_port=53
    let src_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 1];
    let dst_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 2];

    let data = make_raw_ipv6_udp(src_segs, dst_segs, 55555, 53, b"dns-query");

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::RAW)
        .expect("BC-2.02.005 EC-002: RAW IPv6/UDP frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    // BC-2.02.005 postcondition 2: src_ip is IpAddr::V6
    match parsed.src_ip {
        IpAddr::V6(addr) => assert_eq!(
            addr,
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1),
            "BC-2.02.005 EC-002: src_ip must be 2001:db8::1"
        ),
        IpAddr::V4(_) => panic!("BC-2.02.005 EC-002: src_ip must be IpAddr::V6, got V4"),
    }

    // BC-2.02.005 postcondition 3: dst_ip is IpAddr::V6
    match parsed.dst_ip {
        IpAddr::V6(addr) => assert_eq!(
            addr,
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 2),
            "BC-2.02.005 EC-002: dst_ip must be 2001:db8::2"
        ),
        IpAddr::V4(_) => panic!("BC-2.02.005 EC-002: dst_ip must be IpAddr::V6, got V4"),
    }

    // Protocol::Udp
    assert_eq!(
        parsed.protocol,
        Protocol::Udp,
        "BC-2.02.005 EC-002: protocol must be Udp"
    );

    // TransportInfo::Udp with correct ports
    match &parsed.transport {
        TransportInfo::Udp { src_port, dst_port } => {
            assert_eq!(
                *src_port, 55555,
                "BC-2.02.005 EC-002: src_port must be 55555"
            );
            assert_eq!(*dst_port, 53, "BC-2.02.005 EC-002: dst_port must be 53");
        }
        other => panic!("BC-2.02.005 EC-002: expected TransportInfo::Udp, got: {other:?}"),
    }

    // BC-2.02.002 invariant 2 applied to IPv6: dst_port=53 triggers DNS hint
    assert_eq!(
        parsed.app_protocol_hint(),
        Some("DNS"),
        "BC-2.02.005 EC-002: app_protocol_hint must return Some(\"DNS\") for IPv6 UDP dst_port=53"
    );

    // BC-2.02.005 postcondition 6: packet_len == data.len()
    assert_eq!(
        parsed.packet_len,
        data.len(),
        "BC-2.02.005 EC-002: packet_len must equal data.len()"
    );
}

// ---------------------------------------------------------------------------
// m1 — BC-2.02.005 EC-003: RAW IPv6 / ICMPv6
//
// ICMPv6 is previously unexercised. The decoder must surface Protocol::Icmp
// and TransportInfo::None (no port/flag fields) — not panic or error.
// ---------------------------------------------------------------------------

/// BC-2.02.005 EC-003: RAW IPv6 / ICMPv6 Echo Request decodes to IpAddr::V6
/// addresses, Protocol::Icmp, and TransportInfo::None.
#[test]
fn test_BC_2_02_005_ec003_raw_ipv6_icmpv6_protocol_icmp() {
    let src_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 10];
    let dst_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 20];

    let data = make_raw_ipv6_icmpv6(src_segs, dst_segs);

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::RAW)
        .expect("BC-2.02.005 EC-003: RAW IPv6/ICMPv6 frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    // src/dst must be IpAddr::V6
    assert!(
        matches!(parsed.src_ip, IpAddr::V6(_)),
        "BC-2.02.005 EC-003: src_ip must be IpAddr::V6"
    );
    assert!(
        matches!(parsed.dst_ip, IpAddr::V6(_)),
        "BC-2.02.005 EC-003: dst_ip must be IpAddr::V6"
    );

    // BC-2.02.005 EC-003 expected behavior: Protocol::Icmp
    assert_eq!(
        parsed.protocol,
        Protocol::Icmp,
        "BC-2.02.005 EC-003: ICMPv6 must surface as Protocol::Icmp"
    );

    // TransportInfo::None — ICMPv6 has no port or flag tuple in the decoder
    assert!(
        matches!(parsed.transport, TransportInfo::None),
        "BC-2.02.005 EC-003: ICMPv6 transport must be TransportInfo::None"
    );

    // app_protocol_hint() returns None because TransportInfo::None has no ports
    assert_eq!(
        parsed.app_protocol_hint(),
        None,
        "BC-2.02.005 EC-003: app_protocol_hint must return None for ICMPv6"
    );
}

// ---------------------------------------------------------------------------
// m2 — BC-2.02.001 EC-002: TCP SYN-ACK — both syn and ack absolutely asserted.
//
// The TCP_SYN_ACK constant was previously dead code. This test exercises it
// with positive absolute assertions (not equality-only regression coverage).
// ---------------------------------------------------------------------------

/// BC-2.02.001 EC-002: TCP SYN-ACK frame has syn=true AND ack=true absolutely.
///
/// This test positively asserts both flags are true, not merely that RAW==IPV4.
/// Using TCP_SYN_ACK constant so it is no longer dead code.
#[test]
fn test_BC_2_02_001_ec002_tcp_syn_ack_flags() {
    let data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        80,    // server src port
        55000, // client dst port
        1,     // seq
        1,     // ack_num = 1 (acknowledging the SYN)
        TCP_SYN_ACK,
        &[],
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("BC-2.02.001 EC-002: SYN-ACK frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            // Positive absolute assertions — a regression dropping a flag fails here.
            assert!(*syn, "BC-2.02.001 EC-002: syn must be true on SYN-ACK");
            assert!(*ack, "BC-2.02.001 EC-002: ack must be true on SYN-ACK");
            assert!(!*fin, "BC-2.02.001 EC-002: fin must be false on SYN-ACK");
            assert!(!*rst, "BC-2.02.001 EC-002: rst must be false on SYN-ACK");
        }
        other => panic!("BC-2.02.001 EC-002: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// m2 — BC-2.02.001 invariant 2: TCP RST flag positively asserted.
//
// TCP_RST constant was previously dead code. This test exercises it with a
// positive assertion that rst=true and all other control flags are false.
// ---------------------------------------------------------------------------

/// BC-2.02.001 invariant 2: TCP RST frame has rst=true absolutely,
/// with syn, ack, and fin all false.
///
/// Using TCP_RST constant so it is no longer dead code.
#[test]
fn test_BC_2_02_001_tcp_rst_flag_positively_asserted() {
    let data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        45002,
        80,
        9999,
        0,
        TCP_RST, // RST only
        &[],
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("BC-2.02.001 invariant 2: RST frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            assert!(*rst, "BC-2.02.001 invariant 2: rst must be true");
            assert!(!*syn, "BC-2.02.001 invariant 2: syn must be false on RST");
            assert!(!*ack, "BC-2.02.001 invariant 2: ack must be false on RST");
            assert!(!*fin, "BC-2.02.001 invariant 2: fin must be false on RST");
        }
        other => panic!("BC-2.02.001 invariant 2: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// m2 — BC-2.02.001 invariant 2: TCP FIN flag positively asserted.
//
// TCP_FIN constant was previously dead code. This test exercises it with a
// positive assertion that fin=true (with ack, as FIN is normally sent with ACK).
// ---------------------------------------------------------------------------

/// BC-2.02.001 invariant 2: TCP FIN-ACK frame has fin=true and ack=true absolutely.
///
/// Using TCP_FIN constant so it is no longer dead code.
#[test]
fn test_BC_2_02_001_tcp_fin_flag_positively_asserted() {
    let data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        45003,
        80,
        7777,
        1,
        TCP_FIN | TCP_ACK, // FIN-ACK
        &[],
    );

    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::ETHERNET)
        .expect("BC-2.02.001 invariant 2: FIN-ACK frame must decode successfully")
    else {
        panic!("expected IP DecodedFrame")
    };

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            assert!(*fin, "BC-2.02.001 invariant 2: fin must be true");
            assert!(*ack, "BC-2.02.001 invariant 2: ack must be true on FIN-ACK");
            assert!(
                !*syn,
                "BC-2.02.001 invariant 2: syn must be false on FIN-ACK"
            );
            assert!(
                !*rst,
                "BC-2.02.001 invariant 2: rst must be false on FIN-ACK"
            );
        }
        other => panic!("BC-2.02.001 invariant 2: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// m3 — BC-2.02.004 postcondition 1: error-path equivalence.
//
// BC-2.02.004 PC1 states both calls return Ok OR both return Err. Only the
// both-Ok branch was tested. This test feeds malformed bytes and asserts
// that DataLink::RAW and DataLink::IPV4 both produce Err — not one-succeeds-
// one-fails, which would indicate divergent code paths for the two variants.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// m-3 — BC-2.02.005 invariant 1: lax IPv6 address-extraction path.
//
// BC-2.02.005 invariant 1 names lax_ip_triple (src/decoder.rs:231-250) as an
// enforcement site for IPv6 address extraction. Every previous IPv6 test in
// this file builds a well-formed frame, so the strict parse always succeeds
// and lax_ip_triple is never reached. This test exercises the lax path by
// inflating the IPv6 payload_length field past the captured byte count,
// causing the strict parser to fail with SliceError::Len and the lax
// fallback to run — exactly the snaplen-truncated-capture scenario described
// in the decoder module doc. The IPv6 src/dst addresses must still be
// recovered as IpAddr::V6 from lax_ip_triple's IPv6 arm (decoder.rs:241-249).
// ---------------------------------------------------------------------------

/// BC-2.02.005 invariant 1: snaplen-truncated IPv6 frame recovers src/dst
/// IpAddr::V6 values via the lax parse path (lax_ip_triple IPv6 arm,
/// decoder.rs:241-249).
///
/// The IPv6 `payload_length` field (bytes 4..6 of the raw frame) is inflated
/// to 1500, far past the actual captured bytes, forcing the strict parser to
/// fail with a length error and the lax fallback to take over.
#[test]
fn test_BC_2_02_005_invariant1_lax_path_recovers_ipv6_addresses() {
    // Build a valid IPv6/TCP frame using the standard builder, then corrupt
    // its payload_length field to simulate a snaplen-truncated capture.
    let src_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 0xaa];
    let dst_segs: [u16; 8] = [0x2001, 0x0db8, 0, 0, 0, 0, 0, 0xbb];
    let src_port: u16 = 44444;
    let dst_port: u16 = 55555;

    let mut data = make_raw_ipv6_tcp(src_segs, dst_segs, src_port, dst_port, 1, TCP_SYN, &[]);

    // IPv6 `payload_length` is a big-endian u16 at bytes 4..6 of the raw frame
    // (fixed IPv6 header layout; no Ethernet prefix on a RAW capture).
    // Inflating it to 1500 (0x05DC) makes the strict parser fail with
    // SliceError::Len, which is the only error class that triggers the lax
    // fallback in decode_packet.
    data[4] = 0x05;
    data[5] = 0xdc; // payload_length = 1500, far past the captured bytes

    // The lax fallback must still recover the IP layer and extract the IPv6
    // src/dst addresses — this is the assertion that exercises lax_ip_triple.
    let DecodedFrame::Ip(parsed) = decode_packet(&data, DataLink::RAW).expect(
        "BC-2.02.005 invariant 1: snaplen-truncated IPv6 frame must decode via lax fallback",
    ) else {
        panic!("expected IP DecodedFrame")
    };

    // BC-2.02.005 invariant 1: lax_ip_triple's IPv6 arm must produce IpAddr::V6.
    match parsed.src_ip {
        IpAddr::V6(addr) => assert_eq!(
            addr,
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0xaa),
            "BC-2.02.005 invariant 1: lax path must recover src_ip as 2001:db8::aa"
        ),
        IpAddr::V4(_) => {
            panic!("BC-2.02.005 invariant 1: lax path returned IpAddr::V4 for an IPv6 frame")
        }
    }
    match parsed.dst_ip {
        IpAddr::V6(addr) => assert_eq!(
            addr,
            Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 0xbb),
            "BC-2.02.005 invariant 1: lax path must recover dst_ip as 2001:db8::bb"
        ),
        IpAddr::V4(_) => {
            panic!("BC-2.02.005 invariant 1: lax path returned IpAddr::V4 for an IPv6 frame")
        }
    }

    // packet_len must still equal the actual captured byte count, not the
    // inflated payload_length value (BC-2.02.005 PC6 / BC-2.02.001 invariant 1).
    assert_eq!(
        parsed.packet_len,
        data.len(),
        "BC-2.02.005 invariant 1: packet_len must equal data.len() on the lax path"
    );
}

/// BC-2.02.004 postcondition 1 (error path): garbage bytes fed to DataLink::RAW
/// and DataLink::IPV4 both produce Err — the error-path equivalence of the
/// single match arm covering both variants.
#[test]
fn test_BC_2_02_004_raw_and_ipv4_both_err_on_garbage() {
    // 20 bytes of garbage — not a valid IP header (version nibble 0xDE ≠ 4 or 6).
    let garbage: &[u8] = &[
        0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
        0x0B, 0x0C, 0x0D, 0x0E, 0x0F,
    ];

    let raw_result = decode_packet(garbage, DataLink::RAW);
    let ipv4_result = decode_packet(garbage, DataLink::IPV4);

    // BC-2.02.004 PC1: both must produce Err for the same input.
    assert!(
        raw_result.is_err(),
        "BC-2.02.004 PC1: DataLink::RAW must return Err for garbage bytes"
    );
    assert!(
        ipv4_result.is_err(),
        "BC-2.02.004 PC1: DataLink::IPV4 must return Err for garbage bytes \
         (same match arm as RAW)"
    );
}
