//! STORY-003 Phase 3 TDD — Packet Decoding: Linux SLL, No-Panic Safety,
//! Non-IP Frame Rejection.
//!
//! Strategy: brownfield-formalization. Every test maps to one or more ACs
//! from STORY-003.md and a clause from BC-2.02.006 through BC-2.02.009.
//!
//! On the first run (Red Gate), the implementation already exists so all tests
//! EXCEPT `test_VP_008_fuzz_harness_exists` pass (brownfield-confirm). That
//! test was the genuine Red Gate for AC-011; the cargo-fuzz harness has since
//! been created at `fuzz/fuzz_targets/fuzz_decode_packet.rs` and all 21 tests
//! now pass.
//!
//! Test naming convention: `test_BC_S_SS_NNN_<assertion>()` and
//! `test_VP_NNN_<assertion>()` — uppercase letters violate Rust's snake_case
//! lint.  The allow attribute below satisfies CI's `-D warnings` while
//! preserving the factory-mandated naming for full BC/VP traceability.
#![allow(non_snake_case)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pcap_file::DataLink;
use wirerust::decoder::{DecodedFrame, Protocol, TransportInfo, decode_packet};

// ---------------------------------------------------------------------------
// Frame-construction helpers
//
// Every helper builds a byte vector inline — no external fixtures — so the
// tests are self-contained and the exact byte layout is auditable here.
// ---------------------------------------------------------------------------

/// Build a minimal Linux SLL (cooked-capture) frame carrying an IPv4/TCP
/// payload.
///
/// Layout:
///   [0..2]    packet type       (0x0000 = sent by us)
///   [2..4]    ARPHRD_ETHER      (0x0001)
///   [4..6]    link-addr length  (0x0006)
///   [6..14]   link-layer addr   (6 bytes + 2 bytes padding)
///   [14..16]  protocol type     (0x0800 = IPv4)   <── SLL_HEADER_LEN boundary
///   [16..36]  IPv4 header       (20 bytes, total_length = 40)
///   [36..56]  TCP header        (20 bytes)
///
/// Addresses: src 10.0.0.1, dst 10.0.0.2.  Ports: 49153 → 80.  Flags: SYN.
fn make_sll_ipv4_tcp() -> Vec<u8> {
    let mut pkt = Vec::with_capacity(56);
    // SLL header (16 bytes)
    pkt.extend_from_slice(&[0x00, 0x00]); // packet type: sent by us
    pkt.extend_from_slice(&[0x00, 0x01]); // ARPHRD_ETHER
    pkt.extend_from_slice(&[0x00, 0x06]); // link-addr length
    pkt.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00]); // addr (padded)
    pkt.extend_from_slice(&[0x08, 0x00]); // EtherType: IPv4
    // IPv4 header (20 bytes) — total_length=40 covers exactly 20+20 bytes
    pkt.extend_from_slice(&[
        0x45, 0x00, 0x00, 0x28, // ver/IHL, DSCP, total_length=40
        0x00, 0x01, 0x00, 0x00, // identification, flags/frag
        0x40, 0x06, 0x00, 0x00, // TTL=64, proto=TCP(6), checksum
        0x0a, 0x00, 0x00, 0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
    ]);
    // TCP header (20 bytes) — data-offset=5, SYN
    pkt.extend_from_slice(&[
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, // seq number
        0x00, 0x00, 0x00, 0x00, // ack number
        0x50, 0x02, 0xff, 0xff, // data-offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent
    ]);
    pkt
}

/// Build an SLL frame carrying an IPv6/TCP payload.
///
/// Layout:
///   [0..16]   SLL header   (EtherType 0x86DD = IPv6)
///   [16..56]  IPv6 header  (40 bytes)
///   [56..76]  TCP header   (20 bytes)
///
/// Addresses: src 2001:db8::1, dst 2001:db8::2.  Ports: 49153 → 443.
fn make_sll_ipv6_tcp() -> Vec<u8> {
    let mut pkt = Vec::with_capacity(76);
    // SLL header (16 bytes)
    pkt.extend_from_slice(&[0x00, 0x00]); // packet type
    pkt.extend_from_slice(&[0x00, 0x01]); // ARPHRD_ETHER
    pkt.extend_from_slice(&[0x00, 0x06]); // link-addr length
    pkt.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00]);
    pkt.extend_from_slice(&[0x86, 0xdd]); // EtherType: IPv6
    // IPv6 header (40 bytes) — payload_length=20 covers exactly the TCP header
    pkt.extend_from_slice(&[
        0x60, 0x00, 0x00, 0x00, // version=6, traffic class, flow label
        0x00, 0x14, 0x06, 0x40, // payload_length=20, next-header=TCP(6), hop-limit=64
        // src: 2001:db8::1
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, // dst: 2001:db8::2
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
    ]);
    // TCP header (20 bytes)
    pkt.extend_from_slice(&[
        0xc0, 0x01, 0x01, 0xbb, // src 49153, dst 443
        0x00, 0x00, 0x00, 0x01, // seq
        0x00, 0x00, 0x00, 0x00, // ack
        0x50, 0x02, 0xff, 0xff, // data-offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00,
    ]);
    pkt
}

/// Build an SLL frame carrying an IPv6/UDP payload.
///
/// Layout:
///   [0..16]   SLL header  (EtherType 0x86DD = IPv6)
///   [16..56]  IPv6 header (40 bytes, next-header=UDP(0x11), payload_length=8)
///   [56..64]  UDP header  (8 bytes)
///
/// Addresses: src 2001:db8::1, dst 2001:db8::2.  Ports: 12345 → 53.
fn make_sll_ipv6_udp() -> Vec<u8> {
    let mut pkt = Vec::with_capacity(64);
    // SLL header (16 bytes)
    pkt.extend_from_slice(&[0x00, 0x00]); // packet type: sent by us
    pkt.extend_from_slice(&[0x00, 0x01]); // ARPHRD_ETHER
    pkt.extend_from_slice(&[0x00, 0x06]); // link-addr length
    pkt.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00]);
    pkt.extend_from_slice(&[0x86, 0xdd]); // EtherType: IPv6
    // IPv6 header (40 bytes) — payload_length=8 covers exactly the UDP header
    pkt.extend_from_slice(&[
        0x60, 0x00, 0x00, 0x00, // version=6, traffic class, flow label
        0x00, 0x08, 0x11, 0x40, // payload_length=8, next-header=UDP(0x11), hop-limit=64
        // src: 2001:db8::1
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x01, // dst: 2001:db8::2
        0x20, 0x01, 0x0d, 0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x02,
    ]);
    // UDP header (8 bytes) — src 12345, dst 53, length=8, checksum=0
    pkt.extend_from_slice(&[
        0x30, 0x39, // src port 12345
        0x00, 0x35, // dst port 53
        0x00, 0x08, // length = 8 (header only, no payload)
        0x00, 0x00, // checksum (zeroed)
    ]);
    pkt
}

/// Build an `make_sll_ipv4_tcp()` frame with the IPv4 `total_length` field
/// inflated to 1500 to simulate a snaplen-truncated capture.
///
/// The IPv4 `total_length` field is at SLL frame offset 18..20
/// (16-byte SLL header + 2-byte IPv4 offset into the header).
fn make_sll_ipv4_tcp_snaplen_truncated() -> Vec<u8> {
    let mut pkt = make_sll_ipv4_tcp();
    // Overwrite total_length at byte offset 18 (0-indexed within the full frame)
    pkt[18] = 0x05; // 0x05dc = 1500
    pkt[19] = 0xdc;
    pkt
}

/// Build an SLL/IPv4/TCP frame physically cut mid-TCP-header (only 10 of the
/// 20 TCP header bytes are present) with `total_length` still claiming the
/// full 40-byte IP payload.
///
/// This forces the strict parser to fail with a length error (Len), the lax
/// path to be invoked, the IP layer to be recovered (both addresses intact),
/// and the transport layer to degrade to `Protocol::Other(6)` /
/// `TransportInfo::None` — because the captured TCP header bytes are
/// insufficient for lax parsing to extract ports and flags.
///
/// Layout (46 bytes total):
///   [0..16]   SLL header   (16 bytes, EtherType 0x0800 IPv4)
///   [16..36]  IPv4 header  (20 bytes, total_length = 40 = 20 IP + 20 TCP)
///   [36..46]  TCP header   (10 bytes — physically truncated at byte 46)
///
/// The `total_length` field (offset 18..20 in the full frame) is left at 40
/// (0x00, 0x28) — matching the full original frame — so the strict parser
/// sees total_length=40 but only 10 bytes of TCP are captured, triggering a
/// Len error. The lax path then clamps to the captured bytes.
fn make_sll_ipv4_tcp_mid_header_truncated() -> Vec<u8> {
    // Start with the full 56-byte frame and drop the last 10 TCP bytes.
    let full = make_sll_ipv4_tcp();
    // Keep SLL(16) + IPv4(20) + 10 TCP bytes = 46 bytes.
    // total_length at offset 18..20 is already 0x00, 0x28 (= 40), which
    // claims 20 bytes of TCP — 10 more than are present — producing a Len error.
    full[..46].to_vec()
}

/// Build a valid Ethernet ARP frame (EtherType 0x0806, no IP layer).
///
/// A well-formed ARP request:
///   [0..14]   Ethernet header (dst=broadcast, src, EtherType=ARP)
///   [14..42]  ARP payload (28 bytes)
fn make_ethernet_arp() -> Vec<u8> {
    let mut frame = Vec::with_capacity(42);
    // Ethernet header
    frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst: broadcast
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // src mac
    frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
    // ARP payload (28 bytes — htype/ptype/hlen/plen/oper + sender/target hw+ip)
    frame.extend_from_slice(&[
        0x00, 0x01, // htype: Ethernet
        0x08, 0x00, // ptype: IPv4
        0x06, // hlen: 6
        0x04, // plen: 4
        0x00, 0x01, // oper: request
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // sender hw addr
        0xc0, 0xa8, 0x01, 0x0a, // sender proto addr: 192.168.1.10
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // target hw addr (unknown)
        0xc0, 0xa8, 0x01, 0x01, // target proto addr: 192.168.1.1
    ]);
    frame
}

/// Build an Ethernet frame with a custom/unknown EtherType (0x9000) and no
/// IP layer — exercises BC-2.02.009 EC-003 / STORY-003 EC-007.
fn make_ethernet_custom_ethertype_0x9000() -> Vec<u8> {
    let mut frame = Vec::with_capacity(14 + 4);
    frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // src
    frame.extend_from_slice(&[0x90, 0x00]); // EtherType: 0x9000 (non-IP)
    frame.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // arbitrary payload
    frame
}

// ---------------------------------------------------------------------------
// AC-001 / BC-2.02.006 postcondition 1 + postcondition 3
//
// A valid SLL IPv4 TCP frame decodes to Ok(ParsedPacket) with correct
// IP addresses, protocol=Tcp, and TransportInfo::Tcp containing the ports.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_linux_sll_ipv4_tcp() {
    let data = make_sll_ipv4_tcp();
    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_ok(),
        "SLL IPv4 TCP frame must decode to Ok; got: {:?}",
        result.as_ref().unwrap_err()
    );
    let DecodedFrame::Ip(pkt) = result.unwrap() else {
        panic!("expected IP frame")
    };

    // BC-2.02.006 postcondition 1: IP addresses populated correctly
    assert_eq!(
        pkt.src_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        "src_ip must be 10.0.0.1"
    );
    assert_eq!(
        pkt.dst_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        "dst_ip must be 10.0.0.2"
    );

    // BC-2.02.006 postcondition 3: TCP protocol classification
    assert_eq!(pkt.protocol, Protocol::Tcp, "protocol must be Tcp");
    match &pkt.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(*src_port, 49153, "src_port must be 49153");
            assert_eq!(*dst_port, 80, "dst_port must be 80");
        }
        other => panic!("transport must be Tcp; got {other:?}"),
    }

    // BC-2.02.006 postcondition 2: packet_len == data.len()
    assert_eq!(
        pkt.packet_len,
        data.len(),
        "packet_len must equal data.len()"
    );
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.02.006 postcondition 1
//
// An SLL frame with an IPv6 TCP payload decodes to IpAddr::V6 addresses.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_linux_sll_ipv6_tcp() {
    let data = make_sll_ipv6_tcp();
    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_ok(),
        "SLL IPv6 TCP frame must decode to Ok; got: {:?}",
        result.as_ref().unwrap_err()
    );
    let DecodedFrame::Ip(pkt) = result.unwrap() else {
        panic!("expected IP frame")
    };

    // src/dst must be V6 variants
    assert!(
        matches!(pkt.src_ip, IpAddr::V6(_)),
        "src_ip must be IpAddr::V6 for an IPv6 frame; got {:?}",
        pkt.src_ip
    );
    assert!(
        matches!(pkt.dst_ip, IpAddr::V6(_)),
        "dst_ip must be IpAddr::V6 for an IPv6 frame; got {:?}",
        pkt.dst_ip
    );

    let expected_src = IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1));
    let expected_dst = IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 2));
    assert_eq!(pkt.src_ip, expected_src, "src_ip must be 2001:db8::1");
    assert_eq!(pkt.dst_ip, expected_dst, "dst_ip must be 2001:db8::2");
    assert_eq!(
        pkt.protocol,
        Protocol::Tcp,
        "protocol must be Tcp for IPv6/TCP"
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.006 canonical test vector: SLL/IPv6/UDP happy path
//
// BC-2.02.006 postcondition 4 and its canonical test vector table include
// "SLL/IPv6/UDP frame → Ok(ParsedPacket { protocol: Udp })". This test
// exercises that vector: a Linux SLL frame carrying an IPv6 UDP payload
// decodes to IpAddr::V6 source/destination addresses, Protocol::Udp, and
// TransportInfo::Udp with the correct port numbers.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_linux_sll_ipv6_udp() {
    let data = make_sll_ipv6_udp();
    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_ok(),
        "SLL IPv6 UDP frame must decode to Ok; got: {:?}",
        result.as_ref().unwrap_err()
    );
    let DecodedFrame::Ip(pkt) = result.unwrap() else {
        panic!("expected IP frame")
    };

    // Both addresses must be V6 variants.
    assert!(
        matches!(pkt.src_ip, IpAddr::V6(_)),
        "src_ip must be IpAddr::V6; got {:?}",
        pkt.src_ip
    );
    assert!(
        matches!(pkt.dst_ip, IpAddr::V6(_)),
        "dst_ip must be IpAddr::V6; got {:?}",
        pkt.dst_ip
    );
    let expected_src = IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1));
    let expected_dst = IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 2));
    assert_eq!(pkt.src_ip, expected_src, "src_ip must be 2001:db8::1");
    assert_eq!(pkt.dst_ip, expected_dst, "dst_ip must be 2001:db8::2");

    // BC-2.02.006 postcondition 4: UDP payload gives Protocol::Udp.
    assert_eq!(pkt.protocol, Protocol::Udp, "protocol must be Udp");
    match &pkt.transport {
        TransportInfo::Udp { src_port, dst_port } => {
            assert_eq!(*src_port, 12345, "src_port must be 12345");
            assert_eq!(*dst_port, 53, "dst_port must be 53");
        }
        other => panic!("transport must be Udp; got {other:?}"),
    }

    // BC-2.02.006 postcondition 2: packet_len == data.len()
    assert_eq!(
        pkt.packet_len,
        data.len(),
        "packet_len must equal data.len()"
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.02.006 invariant 2
//
// A snaplen-truncated SLL frame (IPv4 total_length inflated) triggers the
// lax fallback path. The lax path strips the 16-byte SLL header and calls
// LaxSlicedPacket::from_ether_type to recover the IP layer.
//
// Observable behavior: decode_packet returns Ok, and the IP addresses are
// correctly extracted from the captured bytes.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_linux_sll_snaplen_truncated_lax_recovery() {
    let data = make_sll_ipv4_tcp_snaplen_truncated();

    // The strict parser must fail (total_length 1500 >> captured bytes 56)
    // and the lax path must recover the IP layer.
    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_ok(),
        "snaplen-truncated SLL frame must recover via lax path; got: {:?}",
        result.as_ref().unwrap_err()
    );
    let DecodedFrame::Ip(pkt) = result.unwrap() else {
        panic!("expected IP frame")
    };

    // IP addresses must be recovered despite truncation
    assert_eq!(
        pkt.src_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        "lax path must recover src_ip 10.0.0.1"
    );
    assert_eq!(
        pkt.dst_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        "lax path must recover dst_ip 10.0.0.2"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.02.006 invariant 3
//
// An SLL frame shorter than 16 bytes fails the strict parse with a non-Len
// error (structural — the SLL header itself is incomplete). decode_packet
// returns Err immediately; the lax fallback is NOT invoked.
//
// Observable: result is Err; the error message is "Parse error: ..." (not
// "Unsupported link type:" and not "No IP layer found").
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_linux_sll_sub_16_bytes_rejected() {
    // 15 bytes — one byte short of a complete SLL header.
    let data: Vec<u8> = vec![
        0x00, 0x00, // packet type
        0x00, 0x01, // ARPHRD_ETHER
        0x00, 0x06, // link-addr length
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00, // addr bytes (8 bytes)
        0x08, // first byte of EtherType only — truncated at byte 15
    ];
    assert_eq!(data.len(), 15, "pre-condition: frame is 15 bytes");

    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_err(),
        "a 15-byte SLL frame must return Err (sub-header length)"
    );
    let msg = result.unwrap_err().to_string();
    // BC-2.02.006 invariant 3: the error comes from the strict parser — it is
    // "Parse error: ..." not "No IP layer found" and not "Unsupported link type:".
    assert!(
        msg.contains("Parse error:"),
        "sub-16-byte SLL frame must produce 'Parse error:'; got: {msg}"
    );
    // The lax fallback would produce "Parse error: SLL header truncated" —
    // that same prefix is fine; what must NOT appear is "No IP layer found"
    // (which would mean the lax path was invoked and found no IP layer).
    assert!(
        !msg.contains("No IP layer found"),
        "lax path must NOT be invoked for sub-16-byte SLL frame; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.02.007 postcondition 1 + invariant 3
//
// Calling decode_packet with random bytes returns Err (no panic) with the
// message matching one of the three valid BC-2.02.007 prefixes.
//
// Test design note: using DataLink::RAW (raw IP) forces all bytes through
// the `SlicedPacket::from_ip` path, where random bytes reliably fail at IP
// header parsing and produce "Parse error:" directly — eliminating the
// complication that some 20-byte sequences can pass Ethernet header parsing
// (14 bytes of MAC/EtherType) and then produce "No IP layer found" instead.
// Both are valid Err responses per BC-2.02.007; "Parse error:" is the only
// prefix that is guaranteed for all random inputs on the RAW path.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_007_random_bytes_no_panic() {
    // 20 deterministic bytes that cannot form a valid IPv4/IPv6 header.
    // Version nibble 0xDE >> 4 = 0xD (= 13) — neither 4 nor 6 — so the
    // strict IP parser immediately rejects this as a structural error.
    // Canonical test vector from BC-2.02.007 (adapted to RAW path).
    let random_bytes: &[u8] = &[
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0xFF, 0x01, 0x02, 0x03, 0x04, 0x05,
        0x06, 0x07, 0x08, 0x09, 0x0A,
    ];

    // The test runner catches panics as test failures — a panic inside
    // decode_packet surfaces here as a test failure, satisfying the no-panic
    // contract observability requirement.
    let result = decode_packet(random_bytes, DataLink::RAW);
    assert!(
        result.is_err(),
        "random bytes must return Err, not Ok; got Ok"
    );
    let msg = result.unwrap_err().to_string();
    // On the RAW path with a non-IP version nibble, the error is guaranteed
    // to be "Parse error: ..." (structural failure, no lax retry).
    assert!(
        msg.contains("Parse error:"),
        "random-bytes error on RAW path must contain 'Parse error:'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-005 (ETHERNET variant) / BC-2.02.007 postcondition 1 + invariant 3
//
// Canonical test vector from BC-2.02.007: "Random 20 bytes with ETHERNET →
// Err (no panic)". The preceding test uses DataLink::RAW to guarantee a
// specific "Parse error:" prefix; this test exercises the canonical ETHERNET
// path and accepts EITHER "Parse error:" OR "No IP layer found" — both are
// correct non-panicking Err responses per BC-2.02.007. Some 20-byte inputs
// pass Ethernet header parsing (14 bytes MAC + EtherType) and produce "No IP
// layer found" if the EtherType is unknown/non-IP; others fail at the link
// layer and produce "Parse error:". Either outcome satisfies the contract.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_007_random_bytes_ethernet_no_panic() {
    // Same 20 canonical bytes, now fed to DataLink::ETHERNET.
    // The EtherType at bytes 12..14 is 0xBE, 0xEF (= 0xBEEF) — an unknown
    // non-IP EtherType — so etherparse parses the Ethernet header cleanly
    // and returns net=None, yielding "No IP layer found". Either way the call
    // must return Err without panicking.
    let random_bytes: &[u8] = &[
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0xFF, 0x01, 0x02, 0x03, 0x04, 0x05,
        0x06, 0x07, 0x08, 0x09, 0x0A,
    ];

    let result = decode_packet(random_bytes, DataLink::ETHERNET);
    assert!(
        result.is_err(),
        "random bytes with ETHERNET must return Err, not Ok; got Ok"
    );
    let msg = result.unwrap_err().to_string();
    // Both prefixes are valid per BC-2.02.007 postcondition 1 and invariant 1.
    assert!(
        msg.contains("Parse error:") || msg.contains("No IP layer found"),
        "ETHERNET random-bytes error must be 'Parse error:' or 'No IP layer found'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.02.007 postcondition 1 + invariant 3
//
// Calling decode_packet with an empty slice returns Err (no panic).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_007_empty_slice_no_panic() {
    // Canonical test vector: data.len() == 0 (BC-2.02.007 EC-001)
    let result = decode_packet(&[], DataLink::ETHERNET);
    assert!(
        result.is_err(),
        "empty slice must return Err, not Ok; got Ok"
    );
    // An empty frame cannot form a valid Ethernet or IP header — the error
    // must be "Parse error: ..." from the parse path, not from the link-type
    // gate.
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Parse error:"),
        "empty-slice error must contain 'Parse error:'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.02.007 invariant 1
//
// The only three error message prefixes ever produced by decode_packet are:
//   1. "Unsupported link type:"
//   2. "No IP layer found"
//   3. "Parse error:"
//
// Representative check: exercise one canonical input for each prefix and
// verify (a) the expected prefix is present and (b) neither of the other two
// appears in the same message. This does NOT prove universal exhaustiveness —
// that is a BC invariant, enforced by code review of every `anyhow!` call in
// decoder.rs, not by a unit test. The test's purpose is to confirm that each
// prefix is reachable via a known input and that the prefixes are mutually
// exclusive at the message level.
//
// STORY-111 reconciliation (BC-2.02.009 v1.6): ARP frames (EtherType 0x0806)
// no longer return "No IP layer found" — they now return a transitional
// Err("ARP extraction not yet implemented") from the STORY-111 non-panicking
// placeholder. The "Prefix 2: No IP layer found" case is now exercised using
// a non-IP non-ARP frame (custom EtherType 0x9000), which continues to return
// "No IP layer found" per BC-2.02.009 postcondition 3 (Path 3, unchanged).
// ARP-specific assertion moved to separate tests with the transitional Err.
// (BC-2.02.009 v1.6; STORY-111 transitional scope.)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_007_error_prefix_representative_check() {
    let valid_prefixes = [
        "Unsupported link type:",
        "No IP layer found",
        "Parse error:",
    ];

    // Hold owned Vec<u8> values so borrows into the cases slice live long
    // enough — no .leak() required.
    // NOTE: ARP frame (make_ethernet_arp()) was previously used here for
    // "Prefix 2" but ARP now returns the transitional Err("ARP extraction not
    // yet implemented") per BC-2.02.009 v1.6 / STORY-111. Replaced with a
    // non-IP non-ARP frame (custom EtherType 0x9000) which still returns
    // "No IP layer found" (BC-2.02.009 Path 3, unchanged).
    let non_ip_non_arp_frame = make_ethernet_custom_ethertype_0x9000();
    let garbage: Vec<u8> = vec![0xFF, 0x00, 0x01];

    let cases: &[(&[u8], DataLink, &str)] = &[
        // Prefix 1: unsupported link type (BC-2.02.008)
        (&[], DataLink::IEEE802_11, "Unsupported link type:"),
        // Prefix 2: non-IP non-ARP frame → "No IP layer found" (BC-2.02.009 Path 3)
        // ARP frames (EtherType 0x0806) are now handled by the ARP arm and return
        // the transitional Err("ARP extraction not yet implemented") in STORY-111
        // (BC-2.02.009 v1.6; full behavior asserted in STORY-112).
        (
            &non_ip_non_arp_frame,
            DataLink::ETHERNET,
            "No IP layer found",
        ),
        // Prefix 3: garbage bytes on a supported link type (BC-2.02.007)
        (&garbage, DataLink::ETHERNET, "Parse error:"),
    ];

    for (data, datalink, expected_prefix) in cases {
        let result = decode_packet(data, *datalink);
        assert!(
            result.is_err(),
            "case '{expected_prefix}' must return Err; got Ok"
        );
        let msg = result.unwrap_err().to_string();

        // Assert the error matches the expected prefix
        assert!(
            msg.contains(expected_prefix),
            "expected prefix '{expected_prefix}' in error; got: {msg}"
        );

        // Assert neither of the other two prefixes appears in this message
        for other_prefix in valid_prefixes {
            if other_prefix == *expected_prefix {
                continue;
            }
            assert!(
                !msg.contains(other_prefix),
                "error for case '{expected_prefix}' must not contain '{other_prefix}'; got: {msg}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.02.008 postcondition 1 + postcondition 2
//
// Calling decode_packet with DataLink::IEEE802_11 (outside the whitelist)
// returns Err containing "Unsupported link type:" without reading any bytes.
// The result is independent of `data`.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_008_unsupported_link_type_error() {
    // The data content is irrelevant — the link-type gate fires before any
    // byte access (BC-2.02.008 postcondition 4).
    let data: &[u8] = &[0x00; 64];
    let result = decode_packet(data, DataLink::IEEE802_11);
    assert!(result.is_err(), "IEEE802_11 must be rejected; got Ok");
    let msg = result.unwrap_err().to_string();

    // BC-2.02.008 postcondition 2: message contains the prefix AND the debug
    // representation of the rejected variant.
    assert!(
        msg.contains("Unsupported link type:"),
        "error must contain 'Unsupported link type:'; got: {msg}"
    );
    // The Debug representation of IEEE802_11 in pcap_file::DataLink
    assert!(
        msg.contains("IEEE802_11"),
        "error must identify 'IEEE802_11'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-009 / BC-2.02.009 postcondition 3 (Path 3: non-IP non-ARP unchanged)
//
// STORY-112 update (BC-2.16.015 AC-006): The STORY-111 transitional behavior
// (Err("ARP extraction not yet implemented")) is now superseded. A valid
// Ethernet/IPv4 ARP frame (hlen=6, plen=4) now produces Ok(DecodedFrame::Arp)
// per BC-2.16.015 postcondition 1 and STORY-112 AC-006. The old transitional
// Err assertion is replaced with the STORY-112 real-routing assertion.
// (BC-2.02.009 v1.6 → STORY-112 final behavior.)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_non_ip_frame_rejected() {
    let data = make_ethernet_arp();
    // STORY-112 AC-006 / BC-2.16.015 PC1: valid Ethernet/IPv4 ARP frames now
    // produce Ok(DecodedFrame::Arp) — not Err (supersedes STORY-111 transitional).
    let result = decode_packet(&data, DataLink::ETHERNET);
    // Must not panic (VP-008 / BC-2.02.009 Invariant 5 — no panic guarantee persists).
    // Result is Ok(DecodedFrame::Arp) for a valid Eth/IPv4 ARP frame.
    let decoded = result.expect(
        "STORY-112 AC-006: valid Ethernet/IPv4 ARP frame must return Ok(DecodedFrame::Arp)",
    );
    // Must be the Arp variant — not Ip.
    match decoded {
        DecodedFrame::Arp(_) => {} // correct
        DecodedFrame::Ip(_) => {
            panic!("STORY-112 AC-006: Ethernet ARP frame must produce DecodedFrame::Arp, not Ip")
        }
    }
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.02.009 Invariant 1 — strict-path ARP routing for SLL frames
//
// STORY-111 reconciliation (BC-2.02.009 v1.6): this test previously asserted
// that an SLL frame carrying ARP (EtherType 0x0806) returns Err("No IP layer
// found"). The prior understanding was that SlicedPacket::from_linux_sll would
// set slice.net=None for an ARP EtherType. In etherparse 0.20, SLL ARP frames
// yield Some(NetSlice::Arp(_)), so the strict ARP dispatch arm fires.
//
// Under BC-2.02.009 v1.6, the STORY-111 ARP arm routes the SLL/ARP frame to
// the non-panicking extract_arp_frame placeholder, which returns None →
// transitional Err("ARP extraction not yet implemented"). The old "No IP layer
// found" assertion is superseded for ARP frames. Non-panic is the primary
// invariant. Full behavioral assertion (Ok(DecodedFrame::Arp) or final Err
// string for non-Eth/IPv4) belongs to STORY-112.
//
// NOTE on the lax-path None arm (decoder.rs): the analysis in the prior comment
// block regarding the lax-path None arm's structural unreachability remains
// valid for non-ARP non-IP frames. For ARP frames, the lax path would yield
// Some(LaxNetSlice::Arp(_)), not None, and is handled by the lax ARP arm
// added in STORY-111. (BC-2.02.009 v1.6; STORY-111 transitional scope.)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_strict_path_sll_arp_no_ip() {
    // Construct an SLL frame carrying ARP. In etherparse 0.20, the strict
    // parser yields Some(NetSlice::Arp(_)) for this frame — the STORY-111
    // ARP dispatch arm in decode_packet fires and calls extract_arp_frame.
    let mut frame = Vec::new();
    // SLL header (16 bytes) — EtherType 0x0806 = ARP
    frame.extend_from_slice(&[0x00, 0x00]); // packet type
    frame.extend_from_slice(&[0x00, 0x01]); // ARPHRD_ETHER
    frame.extend_from_slice(&[0x00, 0x06]); // link-addr length
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00]);
    frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
    // ARP payload (28 bytes)
    frame.extend_from_slice(&[
        0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0xc0,
        0xa8, 0x01, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x01, 0x01,
    ]);

    // Must not panic (VP-008 / BC-2.02.009 Invariant 5 — no-panic guarantee persists).
    let result = std::panic::catch_unwind(|| decode_packet(&frame, DataLink::LINUX_SLL));
    assert!(
        result.is_ok(),
        "STORY-112: SLL ARP frame must NOT panic (VP-008 / BC-2.02.009 Invariant 5); \
         got a panic"
    );
    let inner = result.unwrap();
    // STORY-112 update (BC-2.16.015 AC-006): A valid Ethernet/IPv4 ARP frame
    // (hlen=6, plen=4) via SLL now produces Ok(DecodedFrame::Arp) — supersedes
    // the STORY-111 transitional Err("ARP extraction not yet implemented").
    // outer_src_mac is None for SLL captures (no Ethernet2 link header).
    let decoded =
        inner.expect("STORY-112: SLL ARP frame (hlen=6, plen=4) must return Ok(DecodedFrame::Arp)");
    match decoded {
        DecodedFrame::Arp(f) => {
            // outer_src_mac is None for SLL (no Ethernet2 header in SLL captures).
            assert_eq!(
                f.outer_src_mac, None,
                "STORY-112: SLL ARP frame outer_src_mac must be None (no Ethernet2 header)"
            );
        }
        DecodedFrame::Ip(_) => {
            panic!("STORY-112: SLL ARP frame must produce DecodedFrame::Arp, not Ip")
        }
    }
}

// ---------------------------------------------------------------------------
// AC-011 / VP-008
//
// The cargo-fuzz harness MUST exist at fuzz/fuzz_targets/fuzz_decode_packet.rs
// and MUST be non-empty. This test is a compile-check / file-presence assertion.
//
// Red Gate: this test was the genuine Red Gate for AC-011 (harness absent).
// The harness has been created at fuzz/fuzz_targets/fuzz_decode_packet.rs
// (task 8 in STORY-003.md) and this test now passes.
// ---------------------------------------------------------------------------
/// Exercises VP-008: decode_packet Never Panics on Arbitrary Input.
/// The fuzz harness is the mandatory P0 implementation vehicle for VP-008.
#[test]
fn test_VP_008_fuzz_harness_exists() {
    // CARGO_MANIFEST_DIR is the wirerust crate root (the directory containing
    // Cargo.toml). There is no workspace; fuzz/ is a nested non-workspace
    // sub-crate that lives inside the crate root — hence the harness resolves
    // at <crate-root>/fuzz/fuzz_targets/fuzz_decode_packet.rs.
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let harness_path = manifest_dir.join("fuzz/fuzz_targets/fuzz_decode_packet.rs");

    assert!(
        harness_path.exists(),
        "VP-008 FAIL: fuzz harness not found at {}. \
         The implementer must create this file (STORY-003.md task 8). \
         Run: cargo +nightly fuzz build fuzz_decode_packet",
        harness_path.display()
    );

    let metadata =
        std::fs::metadata(&harness_path).expect("metadata must be readable if the file exists");
    assert!(
        metadata.len() > 0,
        "VP-008 FAIL: fuzz harness at {} exists but is empty. \
         It must implement a harness that passes arbitrary bytes to \
         decode_packet with each supported DataLink variant.",
        harness_path.display()
    );

    // Lightweight content check: confirm the harness references the two key
    // symbols that any valid implementation must contain. A file with only junk
    // bytes passes the non-empty check above but would fail here. The full
    // compile check is separately gated by the CI fuzz-build job
    // (`cargo +nightly fuzz build fuzz_decode_packet`).
    let content = std::fs::read_to_string(&harness_path)
        .expect("fuzz harness must be valid UTF-8 and readable");
    assert!(
        content.contains("decode_packet"),
        "VP-008 FAIL: fuzz harness at {} does not reference 'decode_packet'. \
         The harness must call wirerust::decoder::decode_packet with arbitrary input.",
        harness_path.display()
    );
    assert!(
        content.contains("fuzz_target!"),
        "VP-008 FAIL: fuzz harness at {} does not contain 'fuzz_target!'. \
         The harness must use the libfuzzer_sys::fuzz_target! macro.",
        harness_path.display()
    );
    // Verify the harness still exercises at least one unsupported DataLink
    // variant (widened in adversary pass 2 to cover the rejection arm in
    // decode_packet). IEEE802_11 is the representative unsupported variant;
    // its absence would indicate a regression of that coverage.
    assert!(
        content.contains("IEEE802_11"),
        "VP-008 FAIL: fuzz harness at {} does not reference 'IEEE802_11'. \
         The harness must exercise unsupported DataLink variants to cover the \
         'Unsupported link type:' rejection arm (BC-2.02.008 / VP-008).",
        harness_path.display()
    );
}

// ===========================================================================
// Edge-case tests (EC-001 through EC-007)
// These map to the Edge Cases table in STORY-003.md.
// ===========================================================================

// ---------------------------------------------------------------------------
// EC-001 — SLL frame with IPv4 TCP decoded correctly via SlicedPacket::from_linux_sll
// (see also AC-001 above; this edge-case test adds a dedicated invariant assertion
//  on the strict-path struct fields to complement AC-001's happy-path assertions)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_ec001_sll_ipv4_tcp_strict_path() {
    let data = make_sll_ipv4_tcp();
    let DecodedFrame::Ip(pkt) = decode_packet(&data, DataLink::LINUX_SLL)
        .expect("EC-001: SLL IPv4 TCP must decode via strict path")
    else {
        panic!("expected IP DecodedFrame")
    };

    // Protocol::Tcp is the key invariant from the strict path.
    assert_eq!(pkt.protocol, Protocol::Tcp, "EC-001: protocol must be Tcp");
    // Verify SYN flag is captured
    match &pkt.transport {
        TransportInfo::Tcp { syn, .. } => {
            assert!(*syn, "EC-001: SYN flag must be true");
        }
        other => panic!("EC-001: expected Tcp transport; got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// EC-002 — SLL frame snaplen-truncated (Len error) → lax path invoked
//
// Uses a frame that is physically cut mid-TCP-header so the strict parser
// fails with a Len error (total_length=40 claims 20 TCP bytes but only 10
// are present). The lax fallback is invoked and recovers the IP layer, but
// cannot reconstruct the TCP header — the decode degrades to
// Protocol::Other(6) / TransportInfo::None. This outcome is ONLY reachable
// via the lax path, which distinguishes it from any strict-path success: the
// strict path would either return Ok with full TCP detail or return Err
// (never Ok with Protocol::Other for a TCP-EtherType frame on the strict
// path). The degraded-protocol assertion is therefore direct observable
// evidence that the lax fallback ran.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_ec002_sll_snaplen_lax_invoked() {
    let data = make_sll_ipv4_tcp_mid_header_truncated();
    // total_length=40 but only 10 TCP bytes captured → strict Len error →
    // lax fallback invoked.
    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_ok(),
        "EC-002: mid-TCP-header-truncated SLL frame must be recovered by lax path; got: {:?}",
        result.as_ref().unwrap_err()
    );
    let DecodedFrame::Ip(pkt) = result.unwrap() else {
        panic!("expected IP DecodedFrame")
    };

    // IP layer was captured in full — addresses must be correct.
    assert_eq!(
        pkt.src_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        "EC-002: lax path must recover src_ip 10.0.0.1"
    );
    assert_eq!(
        pkt.dst_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        "EC-002: lax path must recover dst_ip 10.0.0.2"
    );

    // Transport header was NOT fully captured — lax path degrades to
    // Protocol::Other(6) + TransportInfo::None. This is the lax-path-specific
    // behavior documented in the decoder module: "the lax parser recovers the
    // IP layer but not the transport layer". A strict-path success would have
    // returned Protocol::Tcp with port detail, so this assertion is direct
    // evidence that the lax fallback — not the strict path — produced the result.
    assert_eq!(
        pkt.protocol,
        Protocol::Other(6),
        "EC-002: incomplete TCP header must degrade to Protocol::Other(6) on lax path"
    );
    assert!(
        matches!(pkt.transport, TransportInfo::None),
        "EC-002: incomplete TCP header must yield TransportInfo::None on lax path"
    );
}

// ---------------------------------------------------------------------------
// EC-003 — SLL frame < 16 bytes → non-Len error → immediate Err, NO lax retry
// (see also AC-004 above)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_ec003_sll_sub_16_bytes_no_lax_retry() {
    // 8-byte frame — far below the 16-byte SLL header minimum.
    let data: &[u8] = &[0x00, 0x00, 0x00, 0x01, 0x00, 0x06, 0x00, 0x11];
    let result = decode_packet(data, DataLink::LINUX_SLL);
    assert!(result.is_err(), "EC-003: 8-byte SLL frame must return Err");
    // Must not be "No IP layer found" (that would mean lax was tried and failed).
    let msg = result.unwrap_err().to_string();
    assert!(
        !msg.contains("No IP layer found"),
        "EC-003: lax path must NOT be invoked for sub-16-byte SLL; got: {msg}"
    );
    assert!(
        msg.contains("Parse error:"),
        "EC-003: error must be 'Parse error:'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// EC-004 — Empty data slice → Err("Parse error: ...")
// (see also AC-006 above; this test uses LINUX_SLL rather than ETHERNET)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_007_ec004_empty_data_sll() {
    let result = decode_packet(&[], DataLink::LINUX_SLL);
    assert!(
        result.is_err(),
        "EC-004: empty slice with LINUX_SLL must return Err"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Parse error:"),
        "EC-004: empty-slice error must be 'Parse error:'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// EC-005 — DataLink::IEEE802_11 → Err("Unsupported link type: IEEE802_11")
// (see also AC-008 above; this test verifies with an empty data slice to
//  confirm that no bytes are read before the rejection)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_008_ec005_ieee802_11_rejected() {
    // Empty data proves the rejection happens before any byte access.
    let result = decode_packet(&[], DataLink::IEEE802_11);
    assert!(
        result.is_err(),
        "EC-005: IEEE802_11 must be rejected even with empty data"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Unsupported link type:") && msg.contains("IEEE802_11"),
        "EC-005: must produce 'Unsupported link type: ... IEEE802_11'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// EC-006 — ARP Ethernet frame (EtherType 0x0806) — STORY-111 transitional behavior
//
// STORY-111 reconciliation (BC-2.02.009 v1.6): this test previously asserted
// that an Ethernet ARP frame returns Err with the EXACT message "No IP layer found".
// Under BC-2.02.009 v1.6, ARP frames are routed to the ARP dispatch arm in
// decode_packet. The STORY-111 non-panicking placeholder (extract_arp_frame)
// always returns None, so the call returns the transitional
// Err("ARP extraction not yet implemented"). The old "No IP layer found" assertion
// is superseded. Non-panic is the primary invariant; Ok(DecodedFrame::Arp(...))
// is asserted in STORY-112 AC-006 once extract_arp_frame is fully implemented.
// (BC-2.02.009 v1.6 / STORY-111 transitional scope.)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_ec006_arp_ethernet_no_ip_layer() {
    let data = make_ethernet_arp();
    // STORY-112 update (BC-2.16.015 AC-006): a valid Ethernet/IPv4 ARP frame
    // (hlen=6, plen=4) now returns Ok(DecodedFrame::Arp) — the STORY-111
    // transitional Err("ARP extraction not yet implemented") is superseded.
    let result = decode_packet(&data, DataLink::ETHERNET);
    let decoded = result.expect(
        "EC-006 (STORY-112): valid Ethernet/IPv4 ARP frame must return Ok(DecodedFrame::Arp)",
    );
    match decoded {
        DecodedFrame::Arp(_) => {} // correct per BC-2.16.015 AC-006
        DecodedFrame::Ip(_) => {
            panic!("EC-006 (STORY-112): Ethernet ARP frame must produce DecodedFrame::Arp, not Ip")
        }
    }
}

// ---------------------------------------------------------------------------
// EC-007 — Custom EtherType 0x9000 → Err("No IP layer found")
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_ec007_custom_ethertype_no_ip_layer() {
    let data = make_ethernet_custom_ethertype_0x9000();
    let result = decode_packet(&data, DataLink::ETHERNET);
    assert!(
        result.is_err(),
        "EC-007: custom EtherType 0x9000 frame must return Err"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("No IP layer found"),
        "EC-007: custom EtherType error must be 'No IP layer found'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// BC-2.02.006 EC-005 — exactly-16-byte SLL frame (header only, no IP payload)
//
// BC-2.02.006 EC-005 specifies: "SLL frame exactly 16 bytes (no payload) →
// strict parse fails; likely 'No IP layer found'". The frame is a complete,
// valid SLL header with EtherType 0x0800 (IPv4) but zero payload bytes — the
// IP layer is structurally absent because there are no bytes after the header.
// The call must return a non-panicking Err; either "No IP layer found" or
// "Parse error:" is accepted (both are valid BC-2.02.007 prefixes).
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_linux_sll_exactly_16_bytes_no_payload() {
    // A syntactically complete SLL header (16 bytes) with EtherType 0x0800
    // (IPv4) and no bytes following it.
    let data: &[u8] = &[
        0x00, 0x00, // packet type: sent by us
        0x00, 0x01, // ARPHRD_ETHER
        0x00, 0x06, // link-addr length
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00, // addr (padded)
        0x08, 0x00, // EtherType: IPv4
    ];
    assert_eq!(data.len(), 16, "pre-condition: frame is exactly 16 bytes");

    let result = decode_packet(data, DataLink::LINUX_SLL);
    assert!(
        result.is_err(),
        "exactly-16-byte SLL frame must return Err (no IP payload present)"
    );
    // Accept either valid BC-2.02.007 error prefix — both are non-panicking Err.
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("No IP layer found") || msg.contains("Parse error:"),
        "exactly-16-byte SLL error must be 'No IP layer found' or 'Parse error:'; got: {msg}"
    );
}
