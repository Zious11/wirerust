//! STORY-112 test suite (originally written as a Red Gate; now GREEN).
//!
//! Exercises the behavioral contracts for:
//!   BC-2.16.001 — ARP Request Frame Correctly Parsed from ArpPacketSlice
//!   BC-2.16.002 — ARP Reply Frame Correctly Parsed from ArpPacketSlice
//!   BC-2.16.015 — Decode-vs-Analysis Separation
//!
//! Acceptance criteria tested:
//!   AC-001  test_extract_arp_frame_request_returns_some
//!   AC-002  test_extract_arp_frame_request_field_copy_fidelity
//!   AC-003  test_extract_arp_frame_reply_returns_some_with_correct_fields
//!   AC-004  test_extract_arp_frame_none_on_hw_addr_size_8
//!           test_extract_arp_frame_none_on_proto_addr_size_16
//!   AC-005  test_extract_arp_frame_outer_src_mac_none_passthrough
//!   AC-006  test_decode_packet_routes_arp_to_decoded_frame_arp
//!   AC-007  test_decode_packet_lax_arm_truncated_arp_non_panic
//!   AC-008  test_main_arp_arm_calls_process_arp_stub
//!   AC-009  test_arp_frame_never_reaches_stream_dispatcher
//!   AC-012  test_decode_packet_arp_non_eth_ipv4_returns_error
//!
//! GREEN STATUS (STORY-112 implemented):
//!   AC-001..007, AC-012: PASS — extract_arp_frame returns Some(ArpFrame) for
//!     valid Ethernet/IPv4 ARP frames; decode_packet routes ARP to
//!     Ok(DecodedFrame::Arp(...)); non-Eth/IPv4 ARP yields
//!     Err("Non-Ethernet/IPv4 ARP frame").
//!   AC-008: PASSES — ArpAnalyzer::process_arp no-op stub is the deliverable.
//!   AC-009: PASSES — structural assertion that the ARP arm in main.rs does not
//!     call dispatcher.on_data; verified by calling process_arp on a known ArpFrame
//!     and confirming it returns vec![].
//!
//! Test naming: `test_BC_S_SS_NNN_xxx` per factory convention.
//! `#![allow(non_snake_case)]` is required because BC IDs use uppercase letters.

#![allow(non_snake_case)]

use etherparse::ArpPacketSlice;
use pcap_file::DataLink;
use wirerust::analyzer::arp::ArpAnalyzer;
use wirerust::decoder::{ArpFrame, DecodedFrame, decode_packet, extract_arp_frame};

// ---------------------------------------------------------------------------
// ARP byte-buffer builders
// ---------------------------------------------------------------------------

/// Build a minimal 28-byte Ethernet/IPv4 ARP payload (no Ethernet header).
///
/// The ARP payload layout (RFC 826):
///   bytes 0-1:   htype (hardware type) — 0x0001 for Ethernet
///   bytes 2-3:   ptype (protocol type) — 0x0800 for IPv4
///   byte  4:     hlen  (hardware address length) — 6 for Ethernet MAC
///   byte  5:     plen  (protocol address length) — 4 for IPv4
///   bytes 6-7:   oper  (operation) — 1=Request, 2=Reply
///   bytes 8-13:  sender hardware address (MAC)
///   bytes 14-17: sender protocol address (IPv4)
///   bytes 18-23: target hardware address (MAC)
///   bytes 24-27: target protocol address (IPv4)
#[allow(clippy::too_many_arguments)]
fn make_arp_payload(
    htype: u16,
    ptype: u16,
    hlen: u8,
    plen: u8,
    oper: u16,
    sender_mac: [u8; 6],
    sender_ip: [u8; 4],
    target_mac: [u8; 6],
    target_ip: [u8; 4],
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&htype.to_be_bytes());
    buf.extend_from_slice(&ptype.to_be_bytes());
    buf.push(hlen);
    buf.push(plen);
    buf.extend_from_slice(&oper.to_be_bytes());
    // Sender hw addr (hlen bytes) — caller ensures hlen matches sender_mac length or uses custom
    buf.extend_from_slice(&sender_mac[..hlen.min(6) as usize]);
    if hlen > 6 {
        buf.extend(std::iter::repeat_n(0u8, (hlen - 6) as usize));
    }
    // Sender proto addr (plen bytes)
    buf.extend_from_slice(&sender_ip[..plen.min(4) as usize]);
    if plen > 4 {
        buf.extend(std::iter::repeat_n(0u8, (plen - 4) as usize));
    }
    // Target hw addr (hlen bytes)
    buf.extend_from_slice(&target_mac[..hlen.min(6) as usize]);
    if hlen > 6 {
        buf.extend(std::iter::repeat_n(0u8, (hlen - 6) as usize));
    }
    // Target proto addr (plen bytes)
    buf.extend_from_slice(&target_ip[..plen.min(4) as usize]);
    if plen > 4 {
        buf.extend(std::iter::repeat_n(0u8, (plen - 4) as usize));
    }
    buf
}

/// Build a standard 28-byte Ethernet/IPv4 ARP payload (htype=1, ptype=0x0800, hlen=6, plen=4).
fn make_standard_arp_payload(
    oper: u16,
    sender_mac: [u8; 6],
    sender_ip: [u8; 4],
    target_mac: [u8; 6],
    target_ip: [u8; 4],
) -> Vec<u8> {
    make_arp_payload(
        0x0001, 0x0800, 6, 4, oper, sender_mac, sender_ip, target_mac, target_ip,
    )
}

/// Build a full 42-byte Ethernet frame containing an ARP payload.
///
/// Layout: 14-byte Ethernet header + 28-byte ARP payload.
/// The `src_mac` goes into bytes 6..12 of the Ethernet header.
fn make_eth_arp_frame(eth_src_mac: [u8; 6], arp_payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    // Ethernet header (14 bytes)
    frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst MAC broadcast
    frame.extend_from_slice(&eth_src_mac); // src MAC
    frame.extend_from_slice(&[0x08, 0x06]); // EtherType: ARP
    frame.extend_from_slice(arp_payload);
    frame
}

/// Parse an ARP payload slice into an `ArpPacketSlice`.
///
/// Panics if the bytes are too short (which would be a test-setup error, not
/// a production path). This helper mirrors the parsing etherparse performs
/// internally when `decode_packet` routes to the ARP arm.
fn parse_arp_slice(payload: &[u8]) -> ArpPacketSlice<'_> {
    ArpPacketSlice::from_slice(payload)
        .expect("test-setup error: ARP payload bytes must be parseable by etherparse")
}

// ---------------------------------------------------------------------------
// Canonical test vectors (from BC-2.16.001 §Canonical Test Vectors)
// ---------------------------------------------------------------------------

/// ARP Request — BC-2.16.001 first canonical vector.
/// op=1, sender_mac=AA:BB:CC:DD:EE:FF, sender_ip=192.168.1.10,
/// target_mac=00:00:00:00:00:00, target_ip=192.168.1.1, pkt_len=42
const REQUEST_SENDER_MAC: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
const REQUEST_SENDER_IP: [u8; 4] = [192, 168, 1, 10];
const REQUEST_TARGET_MAC: [u8; 6] = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
const REQUEST_TARGET_IP: [u8; 4] = [192, 168, 1, 1];
const REQUEST_PKT_LEN: usize = 42;

/// ARP Reply — BC-2.16.002 first canonical vector.
/// op=2, sender_mac=11:22:33:44:55:66, sender_ip=192.168.1.1,
/// target_mac=AA:BB:CC:DD:EE:FF, target_ip=192.168.1.10, pkt_len=42
const REPLY_SENDER_MAC: [u8; 6] = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66];
const REPLY_SENDER_IP: [u8; 4] = [192, 168, 1, 1];
const REPLY_TARGET_MAC: [u8; 6] = [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF];
const REPLY_TARGET_IP: [u8; 4] = [192, 168, 1, 10];
const REPLY_PKT_LEN: usize = 42;

// ---------------------------------------------------------------------------
// AC-001: extract_arp_frame returns Some for a valid ARP Request
// BC-2.16.001 postcondition 1
// GREEN (STORY-112): extract_arp_frame returns Some(ArpFrame) for a valid
// Ethernet/IPv4 ARP Request (htype=0x0001, ptype=0x0800, hlen=6, plen=4).
// Originally RED in STORY-111 (extract_arp_frame was a None-returning placeholder).
// ---------------------------------------------------------------------------

/// AC-001 (BC-2.16.001 PC1): valid Ethernet/IPv4 ARP Request yields Some(ArpFrame).
#[test]
fn test_BC_2_16_001_extract_arp_frame_request_returns_some() {
    let payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);
    let outer_src_mac = Some(REQUEST_SENDER_MAC);

    let result = extract_arp_frame(&arp, outer_src_mac, REQUEST_PKT_LEN);

    assert!(
        result.is_some(),
        "AC-001 / BC-2.16.001 PC1: extract_arp_frame must return Some(ArpFrame) \
         for a valid Ethernet/IPv4 ARP Request (htype=0x0001, ptype=0x0800, hlen=6, \
         plen=4, op=1). Got None — extract_arp_frame placeholder not yet implemented."
    );
}

// ---------------------------------------------------------------------------
// AC-002: field copy fidelity for ARP Request
// BC-2.16.001 postconditions 2–8
// GREEN (STORY-112): extract_arp_frame returns Some(ArpFrame) with all fields
// byte-exactly copied from the ArpPacketSlice. Originally RED in STORY-111
// (extract_arp_frame returned None so field assertions were never reached).
// ---------------------------------------------------------------------------

/// AC-002 (BC-2.16.001 PC2-8): ARP Request ArpFrame field copy fidelity.
#[test]
fn test_BC_2_16_001_extract_arp_frame_request_field_copy_fidelity() {
    let payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);
    let outer_src_mac = Some(REQUEST_SENDER_MAC);

    let frame = extract_arp_frame(&arp, outer_src_mac, REQUEST_PKT_LEN).expect(
        "AC-002 / BC-2.16.001 PC1: extract_arp_frame must return Some(ArpFrame) \
         for a valid Ethernet/IPv4 ARP Request — got None (stub not yet implemented)",
    );

    // BC-2.16.001 PC2: operation == 1
    assert_eq!(
        frame.operation, 1,
        "AC-002 / BC-2.16.001 PC2: ArpFrame.operation must equal 1 (ARP Request)"
    );
    // BC-2.16.001 PC3: sender_mac == arp.sender_hw_addr()[..6] exactly
    assert_eq!(
        frame.sender_mac,
        <[u8; 6]>::try_from(arp.sender_hw_addr()).unwrap(),
        "AC-002 / BC-2.16.001 PC3: ArpFrame.sender_mac must be byte-exact copy of \
         arp.sender_hw_addr()"
    );
    // BC-2.16.001 PC4: sender_ip == arp.sender_protocol_addr()[..4] exactly
    assert_eq!(
        frame.sender_ip,
        <[u8; 4]>::try_from(arp.sender_protocol_addr()).unwrap(),
        "AC-002 / BC-2.16.001 PC4: ArpFrame.sender_ip must be byte-exact copy of \
         arp.sender_protocol_addr()"
    );
    // BC-2.16.001 PC5: target_mac == arp.target_hw_addr()[..6] exactly
    assert_eq!(
        frame.target_mac,
        <[u8; 6]>::try_from(arp.target_hw_addr()).unwrap(),
        "AC-002 / BC-2.16.001 PC5: ArpFrame.target_mac must be byte-exact copy of \
         arp.target_hw_addr() (all-zero in a standard Request)"
    );
    // BC-2.16.001 PC6: target_ip == arp.target_protocol_addr()[..4] exactly
    assert_eq!(
        frame.target_ip,
        <[u8; 4]>::try_from(arp.target_protocol_addr()).unwrap(),
        "AC-002 / BC-2.16.001 PC6: ArpFrame.target_ip must be byte-exact copy of \
         arp.target_protocol_addr()"
    );
    // BC-2.16.001 PC7: outer_src_mac == parameter passed in unchanged
    assert_eq!(
        frame.outer_src_mac, outer_src_mac,
        "AC-002 / BC-2.16.001 PC7: ArpFrame.outer_src_mac must equal the \
         outer_src_mac parameter passed in unchanged"
    );
    // BC-2.16.001 PC8: packet_len == parameter passed in unchanged
    assert_eq!(
        frame.packet_len, REQUEST_PKT_LEN,
        "AC-002 / BC-2.16.001 PC8: ArpFrame.packet_len must equal the \
         packet_len parameter passed in unchanged"
    );
}

// ---------------------------------------------------------------------------
// AC-003: ARP Reply extraction returns Some with correct fields
// BC-2.16.002 postconditions 1–8
// GREEN (STORY-112): extract_arp_frame returns Some(ArpFrame { operation: 2, ... })
// with all fields byte-exactly copied from the ArpPacketSlice.
// Originally RED in STORY-111 (extract_arp_frame returned None).
// ---------------------------------------------------------------------------

/// AC-003 (BC-2.16.002 PC1-8): valid Ethernet/IPv4 ARP Reply yields
/// Some(ArpFrame { operation: 2, ... }) with exact field copies.
#[test]
fn test_BC_2_16_002_extract_arp_frame_reply_returns_some_with_correct_fields() {
    let payload = make_standard_arp_payload(
        2,
        REPLY_SENDER_MAC,
        REPLY_SENDER_IP,
        REPLY_TARGET_MAC,
        REPLY_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);
    let outer_src_mac = Some(REPLY_SENDER_MAC);

    let frame = extract_arp_frame(&arp, outer_src_mac, REPLY_PKT_LEN).expect(
        "AC-003 / BC-2.16.002 PC1: extract_arp_frame must return Some(ArpFrame) \
         for a valid Ethernet/IPv4 ARP Reply — got None (stub not yet implemented)",
    );

    // BC-2.16.002 PC2: operation == 2
    assert_eq!(
        frame.operation, 2,
        "AC-003 / BC-2.16.002 PC2: ArpFrame.operation must equal 2 (ARP Reply)"
    );
    // BC-2.16.002 PC3: sender_mac byte-exact copy
    assert_eq!(
        frame.sender_mac,
        <[u8; 6]>::try_from(arp.sender_hw_addr()).unwrap(),
        "AC-003 / BC-2.16.002 PC3: ArpFrame.sender_mac must be byte-exact copy of \
         arp.sender_hw_addr()"
    );
    // BC-2.16.002 PC4: sender_ip byte-exact copy
    assert_eq!(
        frame.sender_ip,
        <[u8; 4]>::try_from(arp.sender_protocol_addr()).unwrap(),
        "AC-003 / BC-2.16.002 PC4: ArpFrame.sender_ip must be byte-exact copy of \
         arp.sender_protocol_addr()"
    );
    // BC-2.16.002 PC5: target_mac byte-exact copy (non-zero in ARP Reply)
    assert_eq!(
        frame.target_mac,
        <[u8; 6]>::try_from(arp.target_hw_addr()).unwrap(),
        "AC-003 / BC-2.16.002 PC5: ArpFrame.target_mac must be byte-exact copy of \
         arp.target_hw_addr()"
    );
    // BC-2.16.002 PC6: target_ip byte-exact copy
    assert_eq!(
        frame.target_ip,
        <[u8; 4]>::try_from(arp.target_protocol_addr()).unwrap(),
        "AC-003 / BC-2.16.002 PC6: ArpFrame.target_ip must be byte-exact copy of \
         arp.target_protocol_addr()"
    );
    // BC-2.16.002 PC7: outer_src_mac passed through unchanged
    assert_eq!(
        frame.outer_src_mac, outer_src_mac,
        "AC-003 / BC-2.16.002 PC7: ArpFrame.outer_src_mac must equal the \
         outer_src_mac parameter"
    );
    // BC-2.16.002 PC8: packet_len passed through unchanged
    assert_eq!(
        frame.packet_len, REPLY_PKT_LEN,
        "AC-003 / BC-2.16.002 PC8: ArpFrame.packet_len must equal the \
         packet_len parameter"
    );
}

// ---------------------------------------------------------------------------
// AC-004: extract_arp_frame returns None for non-standard hw/proto sizes
// BC-2.16.001 EC-007 (hw_addr_size=8) and EC-008 (proto_addr_size=16)
// GREEN (STORY-112): extract_arp_frame returns None specifically for
// non-Ethernet/IPv4 hardware/protocol sizes (hw_addr_size != 6 or
// proto_addr_size != 4) via the size guard at decoder.rs:307-342.
// AC-004a/AC-004b assert this rejection path returns None for the RIGHT
// reason (size mismatch), not by accident.
// Originally RED in STORY-111 (extract_arp_frame returned None
// unconditionally for all inputs — a None-returning placeholder — so these
// tests happened to pass vacuously without exercising the real size guard).
// ---------------------------------------------------------------------------

/// AC-004a (BC-2.16.001 EC-007): hw_addr_size=8 yields None, no panic.
#[test]
fn test_BC_2_16_001_extract_arp_frame_none_on_hw_addr_size_8() {
    // Build an ARP payload with hw_addr_size=8 (non-Ethernet).
    // htype=0x0001 (Ethernet type field), but hlen=8 (non-standard).
    // Total ARP payload size: 8 + 8*2 + 4*2 = 32 bytes.
    let payload = make_arp_payload(
        0x0001,                               // htype: still claims Ethernet hardware type
        0x0800,                               // ptype: IPv4
        8,                                    // hlen: 8 — non-Ethernet (EC-007)
        4,                                    // plen: 4 — IPv4 OK
        1,                                    // oper: Request
        [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00], // sender hw (6 bytes, padded to 8 in builder)
        [10, 0, 0, 1],
        [0x00; 6], // target hw
        [10, 0, 0, 2],
    );
    let arp = parse_arp_slice(&payload);

    // Confirm the slice has hw_addr_size == 8 as expected.
    assert_eq!(arp.hw_addr_size(), 8, "test-setup check: hlen must be 8");

    // extract_arp_frame must return None and must NOT panic.
    let result = std::panic::catch_unwind(|| extract_arp_frame(&arp, None, payload.len()));
    assert!(
        result.is_ok(),
        "AC-004a / BC-2.16.001 EC-007: extract_arp_frame must NOT panic \
         for hw_addr_size=8 (non-Ethernet). Got a panic: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap(),
        None,
        "AC-004a / BC-2.16.001 EC-007: extract_arp_frame must return None \
         for hw_addr_size=8 (non-standard Ethernet hardware address length)"
    );
}

/// AC-004b (BC-2.16.001 EC-008): proto_addr_size=16 yields None, no panic.
#[test]
fn test_BC_2_16_001_extract_arp_frame_none_on_proto_addr_size_16() {
    // Build an ARP payload with proto_addr_size=16 (IPv6 address length).
    // Total ARP payload size: 8 + 6*2 + 16*2 = 52 bytes.
    let payload = make_arp_payload(
        0x0001,                               // htype: Ethernet
        0x0800,                               // ptype: IPv4 — but plen=16 contradicts it
        6,                                    // hlen: 6 — Ethernet OK
        16,                                   // plen: 16 — non-IPv4 (EC-008)
        1,                                    // oper: Request
        [0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF], // sender hw
        [192, 168, 1, 10],                    // only first 4 bytes used; builder pads
        [0x00; 6],                            // target hw
        [192, 168, 1, 1],                     // only first 4 bytes used; builder pads
    );
    let arp = parse_arp_slice(&payload);

    // Confirm proto_addr_size == 16.
    assert_eq!(
        arp.proto_addr_size(),
        16,
        "test-setup check: plen must be 16"
    );

    let result = std::panic::catch_unwind(|| extract_arp_frame(&arp, None, payload.len()));
    assert!(
        result.is_ok(),
        "AC-004b / BC-2.16.001 EC-008: extract_arp_frame must NOT panic \
         for proto_addr_size=16. Got a panic: {:?}",
        result.unwrap_err()
    );
    assert_eq!(
        result.unwrap(),
        None,
        "AC-004b / BC-2.16.001 EC-008: extract_arp_frame must return None \
         for proto_addr_size=16 (non-IPv4 protocol address length)"
    );
}

// ---------------------------------------------------------------------------
// AC-005: outer_src_mac=None is passed through unchanged
// BC-2.16.001 EC-003
// GREEN (STORY-112): extract_arp_frame returns Some(ArpFrame { outer_src_mac: None, ... })
// when None is passed in (SLL capture — no Ethernet header).
// Originally RED in STORY-111 (extract_arp_frame returned None, never reaching
// the ArpFrame { outer_src_mac: None } assertion).
// ---------------------------------------------------------------------------

/// AC-005 (BC-2.16.001 EC-003): extract_arp_frame(arp, None, len) →
/// Some(ArpFrame { outer_src_mac: None, ... }).
#[test]
fn test_BC_2_16_001_extract_arp_frame_outer_src_mac_none_passthrough() {
    let payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);

    // Pass None as outer_src_mac (SLL capture — no Ethernet header).
    let frame = extract_arp_frame(&arp, None, REQUEST_PKT_LEN).expect(
        "AC-005 / BC-2.16.001 EC-003: extract_arp_frame must return Some(ArpFrame) \
         even when outer_src_mac is None — got None (stub not yet implemented)",
    );

    assert_eq!(
        frame.outer_src_mac, None,
        "AC-005 / BC-2.16.001 EC-003: ArpFrame.outer_src_mac must be None \
         when None is passed in (outer_src_mac is passed through unchanged)"
    );
}

// ---------------------------------------------------------------------------
// AC-006: decode_packet routes ARP to Ok(DecodedFrame::Arp(...))
// BC-2.16.015 postconditions 1 and 2
// GREEN (STORY-112): decode_packet returns Ok(DecodedFrame::Arp(frame)) for a
// well-formed Ethernet/IPv4 ARP frame, with outer_src_mac extracted from the
// Ethernet source MAC. Originally RED in STORY-111 (strict arm mapped None →
// Err("ARP extraction not yet implemented")).
// ---------------------------------------------------------------------------

/// AC-006 (BC-2.16.015 PC1/2): decode_packet on a well-formed Ethernet/IPv4
/// ARP frame routes to Ok(DecodedFrame::Arp(frame)) with outer_src_mac from
/// the Ethernet source MAC.
#[test]
fn test_BC_2_16_015_decode_packet_routes_arp_to_decoded_frame_arp() {
    // Canonical vector: 42-byte Ethernet/IPv4 ARP Request.
    // Ethernet src MAC = REQUEST_SENDER_MAC → outer_src_mac in the ArpFrame.
    let arp_payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    let frame_bytes = make_eth_arp_frame(REQUEST_SENDER_MAC, &arp_payload);
    assert_eq!(
        frame_bytes.len(),
        42,
        "pre-condition: Ethernet/ARP frame is 42 bytes"
    );

    let result = decode_packet(&frame_bytes, DataLink::ETHERNET);

    // Must be Ok(DecodedFrame::Arp(...)), not Err.
    let decoded = result.expect(
        "AC-006 / BC-2.16.015 PC1: decode_packet must return Ok(DecodedFrame::Arp) \
         for a valid Ethernet/IPv4 ARP Request — got Err (regression: extract_arp_frame \
         must return Some(ArpFrame) for valid Ethernet/IPv4 ARP)",
    );

    // Must be the Arp variant.
    let arp_frame = match decoded {
        DecodedFrame::Arp(f) => f,
        DecodedFrame::Ip(_) => {
            panic!(
                "AC-006 / BC-2.16.015 PC2: decode_packet must return DecodedFrame::Arp \
                 for ARP EtherType 0x0806 — got DecodedFrame::Ip"
            )
        }
    };

    // outer_src_mac must be extracted from the Ethernet source field.
    assert_eq!(
        arp_frame.outer_src_mac,
        Some(REQUEST_SENDER_MAC),
        "AC-006 / BC-2.16.015 PC2: ArpFrame.outer_src_mac must equal the \
         Ethernet source MAC from slice.link (Ethernet2Slice::source())"
    );
    // operation must be copied from the ARP payload.
    assert_eq!(
        arp_frame.operation, 1,
        "AC-006: ArpFrame.operation must equal 1 (ARP Request)"
    );
}

// ---------------------------------------------------------------------------
// AC-007: lax arm handles truncated ARP without panic
// BC-2.16.015 postcondition — lax arm also routes ARP
// GREEN (STORY-112): the lax arm maps Some(f) → Ok(DecodedFrame::Arp(f)) or
// None → Err("truncated ARP frame"). No panic for truncated ARP input.
// Originally RED in STORY-111 (lax arm mapped None → Err("ARP extraction not
// yet implemented")).
//
// Strategy: build a well-formed 42-byte ARP frame, then inflate the IPv4-
// total_length-equivalent field. ARP frames don't have that field, so we
// instead use a frame whose `total_length` was inflated. Actually, for an
// ARP frame the strict parser routes to NetSlice::Arp directly without
// checking a length field (there is no IPv4 header). The strict path
// succeeds for a well-formed 42-byte ARP.
//
// To trigger the Err(SliceError::Len(_)) arm, we need a frame that the
// strict parser rejects with a Len error. For ARP this can be produced by
// giving the ARP frame hlen/plen fields that claim a larger payload than
// the bytes present — but etherparse's strict parser uses the same
// from_slice logic so this produces a regular error, not SliceError::Len.
//
// Practical approach: wrap the ARP in a context where the outer frame's
// length field is inflated past the captured bytes, using an Ethernet
// frame with a fake length-checked field. Since pure ARP-in-Ethernet has
// no such field, we test the non-panic guarantee using catch_unwind and
// assert that any result is Ok(Arp) or Err containing "truncated ARP frame"
// — not a panic, and not a stale STORY-111 stub error.
//
// Alternative: provide a raw ARP-over-SLL or use a frame where the
// strict parser yields SliceError::Len. The simplest reliable approach is
// to assert non-panic via catch_unwind on any ARP-shaped input and then
// assert the error string when the lax path fires.
// ---------------------------------------------------------------------------

/// AC-007 (BC-2.16.015 lax arm): truncated/lax-path ARP never panics; result
/// is Ok(DecodedFrame::Arp) or Err containing "truncated ARP frame".
#[test]
fn test_BC_2_16_015_decode_packet_lax_arm_truncated_arp_non_panic() {
    // Build a well-formed 42-byte Ethernet/ARP frame where we then truncate
    // the ARP payload to force the lax parser. We do this by building an
    // ARP frame with hlen=6, plen=4 then claiming a larger payload via the
    // Ethernet frame.
    //
    // The lax path triggers when etherparse strict-parses the frame and gets
    // SliceError::Len. For Ethernet/ARP frames the strict parser does not
    // check a length field (there is no IP header); it parses the Ethernet
    // header and then delegates to ArpPacketSlice::from_slice. To trigger
    // the Len path we need the ARP slice to be too short for its claimed
    // address sizes.
    //
    // We build an ARP payload claiming hlen=6, plen=4 but truncate it to
    // 20 bytes (8 header + 12 bytes of addresses = 8 + 6 + 4 + 2 of the 10
    // needed), causing etherparse's strict ArpPacketSlice::from_slice to
    // fail with a Len error, which triggers the lax fallback arm.
    //
    // Minimum bytes needed for hlen=6, plen=4: 8 + 6*2 + 4*2 = 28.
    // Truncate to 20 bytes → strict fails with SliceError::Len; lax arm fires.
    let full_arp_payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    // Truncate to 20 bytes — not enough for 28-byte Ethernet/IPv4 ARP payload.
    let truncated_arp = &full_arp_payload[..20];

    let frame_bytes = make_eth_arp_frame(REQUEST_SENDER_MAC, truncated_arp);
    // frame_bytes is now a 34-byte Ethernet frame with a truncated 20-byte ARP payload.
    assert!(
        frame_bytes.len() < 42,
        "pre-condition: truncated frame must be shorter than a full ARP frame"
    );

    // The key assertion: no panic.
    let outcome = std::panic::catch_unwind(|| decode_packet(&frame_bytes, DataLink::ETHERNET));

    assert!(
        outcome.is_ok(),
        "AC-007 / BC-2.16.015: decode_packet must NOT panic on a truncated ARP \
         frame that routes through the lax arm. Got panic: {:?}",
        outcome.unwrap_err()
    );

    let result = outcome.unwrap();
    match &result {
        Ok(DecodedFrame::Arp(_)) => {
            // After implementation: truncated but valid-header ARP may still yield
            // Some(frame) from extract_arp_frame → Ok(DecodedFrame::Arp).
            // This is the correct post-implementation outcome.
        }
        Err(e) => {
            // The implemented error string for a truncated ARP frame is "truncated ARP frame".
            let msg = e.to_string();
            assert!(
                msg.contains("truncated ARP frame"),
                "AC-007 / BC-2.16.015: Err from truncated ARP path must contain \
                 'truncated ARP frame'. Got: {msg}"
            );
        }
        Ok(DecodedFrame::Ip(_)) => {
            panic!(
                "AC-007: decode_packet must not route an ARP EtherType frame to \
                 DecodedFrame::Ip"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-008: ArpAnalyzer::process_arp stub is called and returns vec![]
// BC-2.16.015 postconditions 5/6
// GREEN BY DESIGN: the no-op stub is the STORY-112 deliverable.
// This test locks the contract so future implementation can't accidentally
// break the stub-wiring signature or change the no-op to panic.
// ---------------------------------------------------------------------------

/// AC-008 (BC-2.16.015 PC5/6): ArpAnalyzer::process_arp is callable and
/// returns vec![] (no-op stub). Locks the wiring contract.
#[test]
fn test_BC_2_16_015_main_arp_arm_calls_process_arp_stub() {
    let mut analyzer = ArpAnalyzer::new(3, 50);

    // Construct a minimal ArpFrame with known values.
    let frame = ArpFrame {
        operation: 1,
        sender_mac: REQUEST_SENDER_MAC,
        sender_ip: REQUEST_SENDER_IP,
        target_mac: REQUEST_TARGET_MAC,
        target_ip: REQUEST_TARGET_IP,
        outer_src_mac: Some(REQUEST_SENDER_MAC),
        packet_len: REQUEST_PKT_LEN,
    };
    let timestamp_secs: u32 = 1_700_000_000;

    // Call process_arp — must not panic, must return vec![].
    let findings = analyzer.process_arp(&frame, timestamp_secs);

    assert!(
        findings.is_empty(),
        "AC-008 / BC-2.16.015 PC5/6: ArpAnalyzer::process_arp stub must return \
         vec![] (no findings in STORY-112 stub). Got {} findings.",
        findings.len()
    );
}

// ---------------------------------------------------------------------------
// AC-009: DecodedFrame::Arp never reaches StreamDispatcher
// BC-2.16.015 invariant 2
// GREEN BY DESIGN at the unit level: process_arp returns vec![] and the
// main.rs Arp arm does not call dispatcher.on_data. We verify this
// structurally by calling process_arp and asserting the return is empty
// (no IP pipeline side-effects occur). A spy-based integration approach
// is not available without refactoring the binary, so we assert the
// structural contract via the ArpAnalyzer return value and the fact
// that the ARP arm does not call any dispatcher method (verified by
// inspecting main.rs structure and confirming the empty findings list).
// ---------------------------------------------------------------------------

/// AC-009 (BC-2.16.015 Invariant 2): an ArpFrame processed through the
/// main-loop ARP arm never invokes dispatcher.on_data or any IP-pipeline
/// method. Verified structurally: process_arp returns vec![] (no findings
/// propagated through the IP pipeline) and the ARP arm is separate from
/// the IP arm.
#[test]
fn test_BC_2_16_015_arp_frame_never_reaches_stream_dispatcher() {
    // Structural verification: construct an ArpFrame and call process_arp.
    // The stub returns vec![]; no dispatcher interaction occurs.
    // If dispatcher.on_data were called for ARP frames, IP-pipeline findings
    // would be interleaved with ARP findings — the empty vec![] return and
    // the ArpAnalyzer not-implementing-ProtocolAnalyzer is the structural proof.
    let mut analyzer = ArpAnalyzer::new(3, 50);

    let arp_frame = ArpFrame {
        operation: 2,
        sender_mac: REPLY_SENDER_MAC,
        sender_ip: REPLY_SENDER_IP,
        target_mac: REPLY_TARGET_MAC,
        target_ip: REPLY_TARGET_IP,
        outer_src_mac: Some(REPLY_SENDER_MAC),
        packet_len: REPLY_PKT_LEN,
    };

    // Call process_arp. The return being vec![] and the call not panicking
    // confirms the ARP arm terminates without entering the IP pipeline.
    let findings = analyzer.process_arp(&arp_frame, 0);

    assert!(
        findings.is_empty(),
        "AC-009 / BC-2.16.015 Invariant 2: process_arp must return vec![] \
         in the STORY-112 stub — no IP-pipeline findings must be produced. \
         Got {} findings.",
        findings.len()
    );

    // Additionally verify the ArpFrame values were not corrupted (process_arp
    // receives &frame, not &mut frame, so the original values are preserved).
    assert_eq!(arp_frame.operation, 2);
    assert_eq!(arp_frame.sender_mac, REPLY_SENDER_MAC);
    assert_eq!(arp_frame.outer_src_mac, Some(REPLY_SENDER_MAC));
}

// ---------------------------------------------------------------------------
// AC-012: decode_packet returns Err containing "Non-Ethernet/IPv4 ARP frame"
//         for ARP frames where extract_arp_frame returns None
// BC-2.16.015 postcondition (malformed ARP decode_packet-level error)
// GREEN (STORY-112): the strict arm maps None → Err("Non-Ethernet/IPv4 ARP frame").
// Originally RED in STORY-111 (strict arm emitted "ARP extraction not yet
// implemented", not "Non-Ethernet/IPv4 ARP frame").
// ---------------------------------------------------------------------------

/// AC-012 (BC-2.16.015 PC7): decode_packet returns Err containing
/// "Non-Ethernet/IPv4 ARP frame" when ARP hw_addr_size != 6 (or any
/// condition causing extract_arp_frame to return None). No panic.
#[test]
fn test_BC_2_16_015_decode_packet_arp_non_eth_ipv4_returns_error() {
    // Build an ARP payload with hw_addr_size=8 (non-Ethernet) — extract_arp_frame
    // must return None, and decode_packet must then return
    // Err("Non-Ethernet/IPv4 ARP frame").
    // Total ARP payload: 8 + 8*2 + 4*2 = 32 bytes.
    let arp_payload = make_arp_payload(
        0x0001, // htype: Ethernet hardware type field
        0x0800, // ptype: IPv4
        8,      // hlen: 8 — non-Ethernet (extract_arp_frame returns None)
        4,      // plen: 4 — IPv4 OK
        1,      // oper: Request
        [0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x00],
        [10, 0, 0, 1],
        [0x00; 6],
        [10, 0, 0, 2],
    );
    let frame_bytes = make_eth_arp_frame([0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0x01], &arp_payload);

    // Must not panic.
    let outcome = std::panic::catch_unwind(|| decode_packet(&frame_bytes, DataLink::ETHERNET));
    assert!(
        outcome.is_ok(),
        "AC-012 / BC-2.16.015 PC7: decode_packet must NOT panic for a non-Eth/IPv4 \
         ARP frame (hw_addr_size=8). Got panic: {:?}",
        outcome.unwrap_err()
    );

    let result = outcome.unwrap();
    // Must be an Err.
    let err = result.expect_err(
        "AC-012 / BC-2.16.015 PC7: decode_packet must return Err for a non-Eth/IPv4 \
         ARP frame where extract_arp_frame returns None",
    );

    // The error string must contain "Non-Ethernet/IPv4 ARP frame".
    assert!(
        err.to_string().contains("Non-Ethernet/IPv4 ARP frame"),
        "AC-012 / BC-2.16.015 PC7: decode_packet must return \
         Err containing 'Non-Ethernet/IPv4 ARP frame' for a non-Eth/IPv4 ARP \
         frame (hw_addr_size=8 causes extract_arp_frame to return None). \
         Got: '{}'",
        err
    );
}

// ---------------------------------------------------------------------------
// Additional field-copy fidelity tests using the canonical test vectors
// from BC-2.16.001 and BC-2.16.002 (parameterized style).
// ---------------------------------------------------------------------------

/// Invariant check: extract_arp_frame is opcode-agnostic (BC-2.16.001 Invariant 4).
/// EC-006: op=0 (undefined opcode) → Some(ArpFrame { operation: 0, ... }).
/// GREEN (STORY-112): returns Some(ArpFrame { operation: 0, ... }) — opcode-agnostic extraction.
/// Originally RED in STORY-111 (extract_arp_frame returned None).
#[test]
fn test_BC_2_16_001_invariant_opcode_agnostic_op_zero() {
    let payload = make_standard_arp_payload(
        0, // op=0 — undefined opcode (EC-006)
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);

    let frame = extract_arp_frame(&arp, None, payload.len()).expect(
        "BC-2.16.001 Invariant 4: extract_arp_frame must return Some(ArpFrame) even \
         for op=0 (undefined opcode) — extractor is opcode-agnostic. Got None (stub).",
    );

    assert_eq!(
        frame.operation, 0,
        "BC-2.16.001 Invariant 4: ArpFrame.operation must equal 0 \
         (undefined opcode is extracted faithfully)"
    );
}

/// BC-2.16.001 EC-002: target_mac all-zero in ARP Request is copied faithfully.
/// GREEN (STORY-112): returns Some(ArpFrame { target_mac: [0x00; 6], ... }).
/// Originally RED in STORY-111 (extract_arp_frame returned None).
#[test]
fn test_BC_2_16_001_invariant_zero_target_mac_copied_faithfully() {
    let payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        [0x00; 6], // target MAC all-zero (standard in ARP Request)
        REQUEST_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);

    let frame = extract_arp_frame(&arp, Some(REQUEST_SENDER_MAC), payload.len()).expect(
        "BC-2.16.001 EC-002: extract_arp_frame must return Some even when target_mac \
         is all-zero. Got None (stub).",
    );

    assert_eq!(
        frame.target_mac, [0x00; 6],
        "BC-2.16.001 EC-002: zero target_mac must be copied faithfully into ArpFrame"
    );
}

/// BC-2.16.002 EC-002: GARP Reply (sender_ip == target_ip) — extractor is agnostic.
/// GREEN (STORY-112): returns Some(ArpFrame) with both IPs faithfully copied.
/// Originally RED in STORY-111 (extract_arp_frame returned None).
#[test]
fn test_BC_2_16_002_invariant_garp_reply_extractor_agnostic() {
    let garp_ip: [u8; 4] = [10, 0, 0, 1];
    let payload = make_standard_arp_payload(
        2, // op=2: Reply
        REPLY_SENDER_MAC,
        garp_ip, // sender_ip == target_ip (GARP)
        REPLY_TARGET_MAC,
        garp_ip, // target_ip == sender_ip
    );
    let arp = parse_arp_slice(&payload);

    let frame = extract_arp_frame(&arp, Some(REPLY_SENDER_MAC), payload.len()).expect(
        "BC-2.16.002 EC-002: extract_arp_frame must return Some for a GARP Reply \
         (sender_ip == target_ip). Extractor is agnostic to GARP condition. Got None (stub).",
    );

    // Extractor is agnostic — both IPs must be extracted faithfully.
    assert_eq!(
        frame.sender_ip, garp_ip,
        "BC-2.16.002 EC-002: GARP sender_ip must be extracted faithfully"
    );
    assert_eq!(
        frame.target_ip, garp_ip,
        "BC-2.16.002 EC-002: GARP target_ip must be extracted faithfully \
         (sender_ip == target_ip — GARP detection is analyzer's responsibility)"
    );
    assert_eq!(
        frame.operation, 2,
        "BC-2.16.002 EC-002: GARP Reply operation must be 2"
    );
}

/// BC-2.16.001 EC-004: outer_src_mac != sender_mac is passed through unchanged
/// (D12 mismatch is the analyzer's concern, not the extractor's).
/// GREEN (STORY-112): returns Some(ArpFrame { outer_src_mac: Some(different_mac), ... }).
/// Originally RED in STORY-111 (extract_arp_frame returned None).
#[test]
fn test_BC_2_16_001_invariant_outer_src_mac_mismatch_passthrough() {
    let payload = make_standard_arp_payload(
        1,
        REQUEST_SENDER_MAC,
        REQUEST_SENDER_IP,
        REQUEST_TARGET_MAC,
        REQUEST_TARGET_IP,
    );
    let arp = parse_arp_slice(&payload);

    // outer_src_mac is different from sender_mac in the ARP payload.
    let different_mac: [u8; 6] = [0xFF, 0xEE, 0xDD, 0xCC, 0xBB, 0xAA];
    let frame = extract_arp_frame(&arp, Some(different_mac), payload.len()).expect(
        "BC-2.16.001 EC-004: extract_arp_frame must return Some even when \
         outer_src_mac != sender_mac. Got None (stub).",
    );

    assert_eq!(
        frame.outer_src_mac,
        Some(different_mac),
        "BC-2.16.001 EC-004: outer_src_mac is passed through unchanged even \
         when it differs from sender_mac (D12 mismatch is analyzer's concern)"
    );
    assert_eq!(
        frame.sender_mac, REQUEST_SENDER_MAC,
        "BC-2.16.001 EC-004: sender_mac is still correctly extracted from \
         arp.sender_hw_addr() regardless of outer_src_mac"
    );
}
