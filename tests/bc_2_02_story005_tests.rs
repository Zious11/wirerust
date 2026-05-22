//! STORY-005 Phase 3 TDD — Packet Decoding: packet_len Semantics and TCP Flag/Sequence Extraction
//!
//! Formalizes behavioral contracts:
//!   BC-2.02.014 — packet_len is Set to Total Frame Length, Not Just Payload Length
//!   BC-2.02.015 — Extract TCP Control Flags and Sequence Number into TransportInfo::Tcp
//!
//! Strategy: brownfield-formalization. Production code in src/decoder.rs already exists
//! and is expected to satisfy all ACs. Every test is written against the BC postcondition
//! or invariant it exercises.
//!
//! Test naming convention: test_BC_S_SS_NNN_<assertion>()
//! The BC-based naming pattern uses uppercase letters (BC-S.SS.NNN) which violates
//! Rust's snake_case convention. `#![allow(non_snake_case)]` satisfies both the factory
//! naming mandate and CI's `-D warnings`.
#![allow(non_snake_case)]

use pcap_file::DataLink;
use wirerust::decoder::{TransportInfo, decode_packet};

// ---------------------------------------------------------------------------
// Frame builders — synthetic packet bytes constructed inline.
//
// Reuses the same hand-crafted Ethernet II / IPv4 / TCP layout established
// in bc_2_02_story002_tests.rs. No external fixture files are used.
// ---------------------------------------------------------------------------

/// Build a minimal Ethernet II / IPv4 / TCP frame.
///
/// Layout:
///   Ethernet header   : 14 bytes
///   IPv4 header       : 20 bytes  (IHL=5, no options)
///   TCP header        : 20 bytes  (data-offset=5)
///   Payload           : `payload` bytes (may be empty)
///
/// `flags` is the TCP flags byte (RFC 793 offset 13):
///   CWR ECE URG ACK PSH RST SYN FIN
///   Bit positions (LSB=0): FIN=0 SYN=1 RST=2 PSH=3 ACK=4 URG=5 ECE=6 CWR=7
#[allow(clippy::too_many_arguments)]
fn make_eth_ipv4_tcp(
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    ack_num: u32,
    flags: u8,
    payload: &[u8],
) -> Vec<u8> {
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

/// Build an Ethernet II / IPv4 / TCP frame truncated to `captured_len` bytes,
/// but with the IPv4 total_length field still advertising the full (untruncated)
/// size. This triggers etherparse's strict parser to fail with a `SliceError::Len`
/// error, causing decode_packet to fall back to the lax parse path.
///
/// The `on_wire_payload_len` controls what the IP header says about total size;
/// `captured_len` is how many bytes we actually hand to decode_packet.
fn make_snaplen_truncated_tcp(
    src_ip: [u8; 4],
    dst_ip: [u8; 4],
    src_port: u16,
    dst_port: u16,
    seq: u32,
    flags: u8,
    on_wire_payload_len: u16,
    captured_len: usize,
) -> Vec<u8> {
    // The IP total_length field claims the full on-wire size (may be larger than
    // what we actually capture). etherparse strict parser sees the mismatch and
    // returns SliceError::Len, triggering the lax fallback path in decode_packet.
    let ip_total: u16 = 20 + 20 + on_wire_payload_len;

    let mut frame = Vec::new();

    // Ethernet II header (14 bytes)
    frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]);
    frame.extend_from_slice(&[0x08, 0x00]);

    // IPv4 header (20 bytes) — total_length is the full on-wire length
    frame.push(0x45);
    frame.push(0x00);
    frame.extend_from_slice(&ip_total.to_be_bytes()); // claims full length
    frame.extend_from_slice(&[0x00, 0x01]);
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.push(0x40); // TTL = 64
    frame.push(0x06); // protocol = TCP
    frame.extend_from_slice(&[0x00, 0x00]);
    frame.extend_from_slice(&src_ip);
    frame.extend_from_slice(&dst_ip);

    // TCP header (20 bytes, data offset = 5)
    frame.extend_from_slice(&src_port.to_be_bytes());
    frame.extend_from_slice(&dst_port.to_be_bytes());
    frame.extend_from_slice(&seq.to_be_bytes());
    frame.extend_from_slice(&0u32.to_be_bytes()); // ack number
    frame.push(0x50); // data offset = 5
    frame.push(flags);
    frame.extend_from_slice(&[0xff, 0xff]); // window
    frame.extend_from_slice(&[0x00, 0x00]); // checksum
    frame.extend_from_slice(&[0x00, 0x00]); // urgent pointer

    // Fill payload bytes up to the full on-wire size
    let full_payload = vec![0xAA; on_wire_payload_len as usize];
    frame.extend_from_slice(&full_payload);

    // Truncate to captured_len — this is what a snaplen-limited capture sees
    frame.truncate(captured_len);
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
const TCP_FIN_ACK: u8 = TCP_FIN | TCP_ACK;
const TCP_ALL_FLAGS: u8 = TCP_FIN | TCP_SYN | TCP_RST | TCP_ACK;

// ---------------------------------------------------------------------------
// AC-001 / BC-2.02.014 postcondition 1
//
// For any successfully decoded frame, ParsedPacket.packet_len equals data.len()
// — the total byte length of the raw frame slice passed to decode_packet,
// regardless of header sizes or payload content.
// ---------------------------------------------------------------------------

/// BC-2.02.014 postcondition 1: packet_len equals data.len() for any decoded frame.
///
/// Uses three different payload sizes to cover the invariant beyond a single case.
/// This is the strict-parse path (no truncation).
#[test]
fn test_BC_2_02_014_packet_len_equals_data_len() {
    panic!("RED GATE: AC-001 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.02.014 invariant 1
//
// packet_len is set to the full frame length (data.len()) on BOTH the strict
// parse path (decoder.rs:142-146) and the lax parse path (decoder.rs:161).
// Neither path uses IP header total_length or TCP segment length for this field.
// ---------------------------------------------------------------------------

/// BC-2.02.014 invariant 1: packet_len is set on both the strict and lax parse paths.
///
/// Strict path: well-formed frame where IP total_length matches captured bytes.
/// Lax path: snaplen-truncated frame that forces the lax fallback.
/// In both cases packet_len must equal data.len().
#[test]
fn test_BC_2_02_014_packet_len_set_on_both_strict_and_lax_paths() {
    panic!("RED GATE: AC-002 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.02.014 invariant 2
//
// For a snaplen-truncated packet where data.len() < on-wire frame length,
// packet_len equals the captured (truncated) length. No on_wire_len field exists.
// ---------------------------------------------------------------------------

/// BC-2.02.014 invariant 2: snaplen-truncated packet_len equals captured length.
///
/// Constructs a frame with IP total_length advertising 1500 bytes but only
/// 100 bytes captured. The lax path recovers the IP layer; packet_len must
/// be 100 (the captured length), not 1500 (the advertised on-wire length).
#[test]
fn test_BC_2_02_014_snaplen_truncated_packet_len() {
    panic!("RED GATE: AC-003 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.02.015 postcondition 4
//
// For a TCP SYN packet, TransportInfo::Tcp.syn = true and
// TransportInfo::Tcp.ack = false.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 4: TCP SYN packet extracts syn=true, ack=false.
#[test]
fn test_BC_2_02_015_tcp_syn_flags() {
    panic!("RED GATE: AC-004 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.02.015 postcondition 5
//
// For a TCP SYN-ACK packet, syn = true and ack = true.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 5: TCP SYN-ACK packet extracts syn=true, ack=true.
#[test]
fn test_BC_2_02_015_tcp_syn_ack_flags() {
    panic!("RED GATE: AC-005 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.02.015 postcondition 7 (RST) and postcondition 6 (FIN)
//
// For a TCP RST packet, rst = true.
// For a TCP FIN-ACK packet, fin = true and ack = true.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 6 + 7: RST packet has rst=true; FIN-ACK has fin=true, ack=true.
///
/// Tests both flag combinations in one function per the AC naming mandate.
#[test]
fn test_BC_2_02_015_tcp_rst_and_fin_ack_flags() {
    panic!("RED GATE: AC-006 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.02.015 postcondition 3
//
// TransportInfo::Tcp.seq_number equals the 32-bit sequence number from the
// TCP header (from etherparse's to_header().sequence_number API).
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 3: seq_number extracted from TCP header bytes.
///
/// Uses a known canonical value (0xDEAD_BEEF) embedded in the frame bytes
/// and verifies it survives the decode path unchanged.
#[test]
fn test_BC_2_02_015_tcp_seq_number_extracted() {
    panic!("RED GATE: AC-007 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.02.015 postcondition 8
//
// ParsedPacket.payload contains the TCP segment payload bytes (bytes after the
// TCP header). For a pure ACK with no data, payload is an empty Vec.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 8: TCP payload bytes extracted; pure ACK has empty payload.
#[test]
fn test_BC_2_02_015_tcp_payload_bytes() {
    panic!("RED GATE: AC-008 not yet verified");
}

// ---------------------------------------------------------------------------
// AC-009 / BC-2.02.015 invariant 3
//
// PSH and URG flags are NOT extracted; they are absent from TransportInfo::Tcp.
// Adding them would require a struct change — this is a deliberate scope constraint.
// ---------------------------------------------------------------------------

/// BC-2.02.015 invariant 3: TransportInfo::Tcp struct has no psh or urg fields.
///
/// Exercises the struct exhaustively by pattern-matching all named fields. If
/// psh or urg were added to the struct this test would fail to compile (exhaustive
/// match would leave new fields unmatched), making scope creep a compile error.
#[test]
fn test_BC_2_02_015_psh_urg_not_in_transport_info() {
    panic!("RED GATE: AC-009 not yet verified");
}

// ---------------------------------------------------------------------------
// Edge-case tests EC-001..EC-006 from STORY-005.md
// ---------------------------------------------------------------------------

/// EC-001 (BC-2.02.014 EC-001): 1500-byte Ethernet frame — packet_len == 1500.
///
/// The canonical maximum-transmission-unit Ethernet frame size.
/// Exercises the invariant at the upper boundary for standard Ethernet captures.
#[test]
fn test_BC_2_02_014_ec001_1500_byte_frame_packet_len() {
    // EC-001: 1500-byte Ethernet frame; packet_len must equal 1500
    panic!("RED GATE: EC-001 not yet verified");
}

/// EC-002 (BC-2.02.014 EC-004 + BC-2.02.015 EC-006): 54-byte TCP ACK (no payload).
///
/// Ethernet(14) + IPv4(20) + TCP(20) = 54 bytes total.
/// packet_len == 54 and payload is an empty Vec.
#[test]
fn test_BC_2_02_014_ec002_54_byte_pure_ack() {
    // EC-002: 54-byte pure-ACK frame; packet_len == 54, payload is empty Vec
    panic!("RED GATE: EC-002 not yet verified");
}

/// EC-003 (BC-2.02.014 EC-003): Snaplen-truncated at 100 bytes.
///
/// IP header advertises 1500-byte on-wire frame but only 100 bytes are captured.
/// packet_len must equal 100 (the captured length), not 1500.
#[test]
fn test_BC_2_02_014_ec003_snaplen_truncated_at_100() {
    // EC-003: snaplen truncated; packet_len == captured length (100), not on-wire (1500)
    panic!("RED GATE: EC-003 not yet verified");
}

/// EC-004 (BC-2.02.015 EC-005): seq_number = 0xFFFFFFFF (max u32, wraparound boundary).
///
/// The sequence number must be 4294967295 with no overflow or truncation.
#[test]
fn test_BC_2_02_015_ec004_seq_number_max_u32() {
    // EC-004: seq_number == 0xFFFFFFFF; no overflow or truncation
    panic!("RED GATE: EC-004 not yet verified");
}

/// EC-005 (BC-2.02.015): All four flags simultaneously set — syn=true, ack=true, fin=true, rst=true.
///
/// Pathological packet with all four tracked flags set. All must be extracted correctly.
#[test]
fn test_BC_2_02_015_ec005_all_four_flags_set() {
    // EC-005: all four flags set simultaneously; all must be true
    panic!("RED GATE: EC-005 not yet verified");
}

/// EC-006 (BC-2.02.015 EC-007): No flags set (data segment).
///
/// A TCP data segment with no control flags. All four extracted flags must be false.
#[test]
fn test_BC_2_02_015_ec006_no_flags_set() {
    // EC-006: zero flags byte; syn=false, ack=false, fin=false, rst=false
    panic!("RED GATE: EC-006 not yet verified");
}
