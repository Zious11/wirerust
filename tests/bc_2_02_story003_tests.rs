//! STORY-003 Phase 3 TDD — Packet Decoding: Linux SLL, No-Panic Safety,
//! Non-IP Frame Rejection.
//!
//! Strategy: brownfield-formalization. Every test maps to one or more ACs
//! from STORY-003.md and a clause from BC-2.02.006 through BC-2.02.009.
//!
//! On the first run the implementation already exists, so all tests EXCEPT
//! `test_VP_008_fuzz_harness_exists` are expected to PASS (brownfield-confirm).
//! `test_VP_008_fuzz_harness_exists` is the genuine Red Gate failure: the
//! cargo-fuzz harness at `fuzz/fuzz_targets/fuzz_decode_packet.rs` does not
//! yet exist and must be created by the implementer.
//!
//! Test naming convention: `test_BC_S_SS_NNN_<assertion>()` and
//! `test_VP_NNN_<assertion>()` — uppercase letters violate Rust's snake_case
//! lint.  The allow attribute below satisfies CI's `-D warnings` while
//! preserving the factory-mandated naming for full BC/VP traceability.
#![allow(non_snake_case)]

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use pcap_file::DataLink;
use wirerust::decoder::{Protocol, TransportInfo, decode_packet};

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
        result.unwrap_err()
    );
    let pkt = result.unwrap();

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
        result.unwrap_err()
    );
    let pkt = result.unwrap();

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
    assert_eq!(pkt.protocol, Protocol::Tcp, "protocol must be Tcp for IPv6/TCP");
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
        result.unwrap_err()
    );
    let pkt = result.unwrap();

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
        0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0xFF, 0x01, 0x02, 0x03, 0x04,
        0x05, 0x06, 0x07, 0x08, 0x09, 0x0A,
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
// Exhaustiveness test: exercise one representative input for each prefix and
// assert that the resulting error matches one of the three and no others.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_007_error_prefix_exhaustiveness() {
    let valid_prefixes = ["Unsupported link type:", "No IP layer found", "Parse error:"];

    let cases: &[(&[u8], DataLink, &str)] = &[
        // Prefix 1: unsupported link type (BC-2.02.008)
        (&[], DataLink::IEEE802_11, "Unsupported link type:"),
        // Prefix 2: valid ARP frame parsed but no IP layer (BC-2.02.009)
        (make_ethernet_arp().leak(), DataLink::ETHERNET, "No IP layer found"),
        // Prefix 3: garbage bytes on a supported link type (BC-2.02.007)
        (&[0xFF, 0x00, 0x01], DataLink::ETHERNET, "Parse error:"),
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

        // Assert no other prefix appears when it shouldn't
        for other_prefix in valid_prefixes {
            if *other_prefix == **expected_prefix {
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
    assert!(
        result.is_err(),
        "IEEE802_11 must be rejected; got Ok"
    );
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
// AC-009 / BC-2.02.009 postcondition 1
//
// A valid Ethernet ARP frame (EtherType 0x0806, no IP layer) returns Err
// containing "No IP layer found".
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_non_ip_frame_rejected() {
    let data = make_ethernet_arp();
    let result = decode_packet(&data, DataLink::ETHERNET);
    assert!(
        result.is_err(),
        "ARP frame must return Err; got Ok"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("No IP layer found"),
        "ARP frame error must contain 'No IP layer found'; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-010 / BC-2.02.009 invariant 1 + invariant 2 + invariant 3
//
// Lax retry is NOT attempted for structurally absent IP layers (non-length
// errors).  On the strict-parse path, if slice.net is None (non-IP frame),
// the error is returned immediately; lax parsing is not invoked.
//
// Tested with an SLL frame whose strict parse succeeds (no length error) but
// carries no IP layer — specifically a 16-byte SLL header with EtherType
// 0x0806 (ARP) and a minimal ARP body.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_lax_path_also_rejects_no_ip() {
    // Construct an SLL frame carrying ARP (no IP layer). The strict parse
    // of the SLL header succeeds (≥ 16 bytes, valid EtherType), but the
    // network layer is None.  Verify the error is "No IP layer found".
    let mut frame = Vec::new();
    // SLL header (16 bytes) — EtherType 0x0806 = ARP
    frame.extend_from_slice(&[0x00, 0x00]); // packet type
    frame.extend_from_slice(&[0x00, 0x01]); // ARPHRD_ETHER
    frame.extend_from_slice(&[0x00, 0x06]); // link-addr length
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x00, 0x00]);
    frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
    // ARP payload (28 bytes)
    frame.extend_from_slice(&[
        0x00, 0x01, 0x08, 0x00, 0x06, 0x04, 0x00, 0x01, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
        0xc0, 0xa8, 0x01, 0x0a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc0, 0xa8, 0x01, 0x01,
    ]);

    let result = decode_packet(&frame, DataLink::LINUX_SLL);
    assert!(
        result.is_err(),
        "SLL ARP frame must return Err; got Ok"
    );
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("No IP layer found"),
        "SLL ARP frame error must be 'No IP layer found'; got: {msg}"
    );
    // The lax path would NOT produce "Parse error:" for this case;
    // the error must be "No IP layer found", confirming the strict
    // path rejected it (not a lax-fallback rejection).
    assert!(
        !msg.contains("Parse error:"),
        "non-IP SLL frame must be rejected on strict path, not via parse error; got: {msg}"
    );
}

// ---------------------------------------------------------------------------
// AC-011 / VP-008
//
// The cargo-fuzz harness MUST exist at fuzz/fuzz_targets/fuzz_decode_packet.rs
// and MUST be non-empty. This test is a compile-check / file-presence assertion.
//
// EXPECTED: FAIL at this stage — the fuzz harness has NOT been created yet.
// This is the genuine Red Gate for AC-011. The implementer must create
// fuzz/fuzz_targets/fuzz_decode_packet.rs (task 8 in STORY-003.md).
// ---------------------------------------------------------------------------
/// Exercises VP-008: decode_packet Never Panics on Arbitrary Input.
/// The fuzz harness is the mandatory P0 implementation vehicle for VP-008.
#[test]
fn test_VP_008_fuzz_harness_exists() {
    // Resolve path relative to the workspace root (CARGO_MANIFEST_DIR).
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let harness_path = manifest_dir.join("fuzz/fuzz_targets/fuzz_decode_packet.rs");

    assert!(
        harness_path.exists(),
        "VP-008 FAIL: fuzz harness not found at {}. \
         The implementer must create this file (STORY-003.md task 8). \
         Run: cargo +nightly fuzz build fuzz_decode_packet",
        harness_path.display()
    );

    let metadata = std::fs::metadata(&harness_path)
        .expect("metadata must be readable if the file exists");
    assert!(
        metadata.len() > 0,
        "VP-008 FAIL: fuzz harness at {} exists but is empty. \
         It must implement a harness that passes arbitrary bytes to \
         decode_packet with each supported DataLink variant.",
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
    let pkt = decode_packet(&data, DataLink::LINUX_SLL)
        .expect("EC-001: SLL IPv4 TCP must decode via strict path");

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
// This overlaps with AC-003 but focuses on the lax path being invoked
// specifically for the Len-error subclass, not any other error.
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_006_ec002_sll_snaplen_lax_invoked() {
    let data = make_sll_ipv4_tcp_snaplen_truncated();
    // The total_length in the IPv4 header is 1500 but only 56 bytes were
    // captured, so the strict parser MUST fail with a length error and the
    // lax fallback MUST be invoked. Verify by checking the result is Ok
    // (the lax path succeeds), not Err (which would mean it wasn't retried).
    let result = decode_packet(&data, DataLink::LINUX_SLL);
    assert!(
        result.is_ok(),
        "EC-002: snaplen-truncated SLL frame must be recovered by lax path; got: {:?}",
        result.unwrap_err()
    );
    // Addresses are preserved because the IP header bytes were captured.
    let pkt = result.unwrap();
    assert_eq!(
        pkt.src_ip,
        IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        "EC-002: src_ip must be recovered by lax path"
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
    assert!(
        result.is_err(),
        "EC-003: 8-byte SLL frame must return Err"
    );
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
// EC-006 — ARP Ethernet frame → Err("No IP layer found")
// (see also AC-009 above; this edge-case test reinforces the exact message)
// ---------------------------------------------------------------------------
#[test]
fn test_BC_2_02_009_ec006_arp_ethernet_no_ip_layer() {
    let data = make_ethernet_arp();
    let result = decode_packet(&data, DataLink::ETHERNET);
    assert!(
        result.is_err(),
        "EC-006: Ethernet ARP frame must return Err"
    );
    let msg = result.unwrap_err().to_string();
    assert_eq!(
        msg, "No IP layer found",
        "EC-006: error message must be exactly 'No IP layer found'; got: {msg}"
    );
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
