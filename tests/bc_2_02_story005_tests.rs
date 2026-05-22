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
use wirerust::decoder::{ParsedPacket, TransportInfo, decode_packet};

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

/// Build an Ethernet II / IPv4 / TCP frame whose IP total_length field claims
/// `on_wire_payload_len` bytes of payload but the slice is truncated to
/// `captured_len` bytes before being returned.
///
/// The IP total_length over-run causes etherparse's strict parser to return
/// `SliceError::Len`, triggering the lax fallback path inside `decode_packet`.
/// This models a tcpdump `-s <snaplen>` capture where packets are truncated.
#[allow(clippy::too_many_arguments)]
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
    // IP total_length claims full on-wire size; the captured slice is shorter.
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

    // Full on-wire payload (will be truncated below)
    let full_payload = vec![0xAA; on_wire_payload_len as usize];
    frame.extend_from_slice(&full_payload);

    // Truncate to the captured snaplen — this is what decode_packet receives
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
// All four tracked flags: FIN | SYN | RST | ACK (PSH=0x08 and URG=0x20 excluded per BC-2.02.015 invariant 3)
const TCP_ALL_FOUR: u8 = TCP_FIN | TCP_SYN | TCP_RST | TCP_ACK;

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
/// This exercises the strict parse path (well-formed frames, no snaplen truncation).
///
/// Also includes a variable-header-size sub-case (IPv4 IHL=6, 24-byte IPv4 header)
/// that exercises BC-2.02.014 invariant 1 ("never a header-dependent partial length"):
/// a decoder bug that computed `packet_len = data.len() - variable_header_len` instead
/// of `data.len()` would pass the fixed-54-byte-header cases but fail here. The
/// non-standard 24-byte IPv4 header (IHL=6 = 20 base + 4 NOP option bytes) shifts the
/// total header to 14+24+20 = 58 bytes, breaking any header-offset-dependent formula.
#[test]
fn test_BC_2_02_014_packet_len_equals_data_len() {
    // BC-2.02.014 canonical test vectors: 0-byte, 10-byte, 100-byte payloads
    let cases: &[(&[u8], &str)] = &[
        (&[], "empty payload (0 bytes)"),
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
            TCP_ACK,
            payload,
        );
        let expected_len = data.len();

        let parsed = decode_packet(&data, DataLink::ETHERNET)
            .unwrap_or_else(|e| panic!("decode failed for {label}: {e}"));

        // BC-2.02.014 postcondition 1: packet_len == data.len()
        assert_eq!(
            parsed.packet_len, expected_len,
            "AC-001: packet_len ({}) must equal data.len() ({}) for {label}",
            parsed.packet_len, expected_len
        );
    }

    // -----------------------------------------------------------------------
    // Variable-header-size sub-case: IPv4 IHL=6 (24-byte IPv4 header).
    //
    // Exercises BC-2.02.014 invariant 1 ("never a header-dependent partial
    // length"). A buggy decoder computing `packet_len = data.len() -
    // variable_header_len` (where variable_header_len adapts to IHL) would
    // yield 40 (the payload length), not 98 (data.len()). This sub-case
    // catches that class of bug, which the fixed-54-byte-header cases above
    // cannot detect.
    //
    // Frame layout:
    //   Ethernet header      : 14 bytes
    //   IPv4 header (IHL=6)  : 24 bytes (20 base + 4 NOP option bytes)
    //   TCP header           : 20 bytes  (data-offset=5)
    //   Payload              : 40 bytes
    //   Total                : 98 bytes
    //
    // IPv4 options: 3× NOP (0x01) + End-of-options (0x00) = 4 bytes.
    // IPv4 total_length = 24 + 20 + 40 = 84.
    // -----------------------------------------------------------------------
    {
        let tcp_payload = [0xCC; 40];
        let ip_options = [0x01u8, 0x01, 0x01, 0x00]; // NOP, NOP, NOP, End-of-options
        let ip_header_len: usize = 20 + ip_options.len(); // 24 bytes
        let ip_total: u16 = (ip_header_len + 20 + tcp_payload.len()) as u16; // 84

        let mut frame = Vec::new();

        // Ethernet II header (14 bytes)
        frame.extend_from_slice(&[0xff, 0xff, 0xff, 0xff, 0xff, 0xff]); // dst MAC
        frame.extend_from_slice(&[0x00, 0x11, 0x22, 0x33, 0x44, 0x55]); // src MAC
        frame.extend_from_slice(&[0x08, 0x00]); // EtherType: IPv4

        // IPv4 header (24 bytes: IHL=6 → first nibble of first byte = 0x46)
        frame.push(0x46); // version=4, IHL=6 (24 bytes)
        frame.push(0x00); // DSCP/ECN
        frame.extend_from_slice(&ip_total.to_be_bytes()); // total length = 84
        frame.extend_from_slice(&[0x00, 0x01]); // identification
        frame.extend_from_slice(&[0x00, 0x00]); // flags / fragment offset
        frame.push(0x40); // TTL = 64
        frame.push(0x06); // protocol = TCP
        frame.extend_from_slice(&[0x00, 0x00]); // checksum (zero; etherparse does not verify)
        frame.extend_from_slice(&[192u8, 168, 1, 10]); // source IP
        frame.extend_from_slice(&[10u8, 0, 0, 1]); // destination IP
        frame.extend_from_slice(&ip_options); // 4 bytes of IPv4 options (NOP×3 + EOL)

        // TCP header (20 bytes, data-offset=5)
        frame.extend_from_slice(&12345u16.to_be_bytes()); // src port
        frame.extend_from_slice(&80u16.to_be_bytes()); // dst port
        frame.extend_from_slice(&1u32.to_be_bytes()); // sequence number
        frame.extend_from_slice(&0u32.to_be_bytes()); // acknowledgement number
        frame.push(0x50); // data offset = 5, reserved = 0
        frame.push(TCP_ACK); // flags
        frame.extend_from_slice(&[0xff, 0xff]); // window size
        frame.extend_from_slice(&[0x00, 0x00]); // checksum
        frame.extend_from_slice(&[0x00, 0x00]); // urgent pointer

        // Payload (40 bytes)
        frame.extend_from_slice(&tcp_payload);

        let expected_len = frame.len(); // must be 98

        let parsed = decode_packet(&frame, DataLink::ETHERNET).unwrap_or_else(|e| {
            panic!(
                "AC-001 variable-header sub-case: IPv4-with-options frame (IHL=6, {expected_len} bytes) \
                 must decode successfully, but got: {e}"
            )
        });

        // BC-2.02.014 postcondition 1 + invariant 1: packet_len == data.len() == 98,
        // not 40 (payload length) nor 84 (ip_total) nor any header-derived partial value.
        assert_eq!(
            parsed.packet_len, expected_len,
            "AC-001 variable-header sub-case: packet_len ({}) must equal data.len() ({expected_len}) \
             for an IPv4-with-options frame — any header-offset-dependent bug would yield a \
             different value (e.g. 40 for payload-length or 84 for ip_total)",
            parsed.packet_len
        );
    }
}

// ---------------------------------------------------------------------------
// AC-002 / BC-2.02.014 postcondition 2
//
// packet_len is set to the full frame length (data.len()) on BOTH the strict
// parse path (decoder.rs:142-146) and the lax parse path (decoder.rs:161).
// Neither path uses IP header total_length or TCP segment length for this field.
// ---------------------------------------------------------------------------

/// BC-2.02.014 postcondition 2: packet_len is set on both the strict and lax parse paths.
///
/// Strict path: well-formed frame where IP total_length matches captured bytes.
/// Lax path: snaplen-truncated frame (IP total_length > captured bytes) that forces
/// the lax fallback in decode_packet. In both cases packet_len must equal data.len(),
/// not the IP header's total_length field.
#[test]
fn test_BC_2_02_014_packet_len_set_on_both_strict_and_lax_paths() {
    // --- Strict parse path (decoder.rs:142-146) ---
    let strict_data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        1234,
        5678,
        100,
        0,
        TCP_SYN,
        b"strict-path-payload",
    );
    let strict_expected = strict_data.len();

    let strict_parsed = decode_packet(&strict_data, DataLink::ETHERNET)
        .expect("AC-002: strict-path frame must decode successfully");

    // BC-2.02.014 postcondition 2 (strict path): packet_len == data.len()
    assert_eq!(
        strict_parsed.packet_len, strict_expected,
        "AC-002: strict path — packet_len ({}) must equal data.len() ({})",
        strict_parsed.packet_len, strict_expected
    );

    // --- Lax parse path (decoder.rs:161) ---
    // Frame claims 1460 bytes of payload (on-wire) but only 100 bytes are captured.
    // Ethernet(14) + IPv4(20) + TCP(20) = 54-byte header; captured_len = 100 bytes.
    let lax_data = make_snaplen_truncated_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        1234,
        5678,
        100,
        TCP_ACK,
        1460, // on_wire_payload_len (IP total_length = 20 + 20 + 1460 = 1500)
        100,  // captured_len (snaplen = 100)
    );
    let lax_expected = lax_data.len(); // must be 100

    let lax_parsed = decode_packet(&lax_data, DataLink::ETHERNET)
        .expect("AC-002: lax-path truncated frame must decode successfully via lax fallback");

    // BC-2.02.014 postcondition 2 (lax path): packet_len == captured data.len(), not IP total_length
    assert_eq!(
        lax_parsed.packet_len, lax_expected,
        "AC-002: lax path — packet_len ({}) must equal captured data.len() ({}), not IP total_length",
        lax_parsed.packet_len, lax_expected
    );
}

// ---------------------------------------------------------------------------
// AC-003 / BC-2.02.014 invariant 2
//
// For a snaplen-truncated packet where data.len() < on-wire frame length,
// packet_len equals the captured (truncated) length. No on_wire_len field exists.
// ---------------------------------------------------------------------------

/// BC-2.02.014 invariant 2: snaplen-truncated packet_len equals captured length.
///
/// Constructs a frame with IP total_length advertising 1500 bytes (IP-datagram on-wire
/// length: 20 IP + 20 TCP + 1460 payload) but only 100 bytes are passed to
/// decode_packet (captured). The Ethernet-frame on-wire length would be 1514 bytes
/// (Ethernet header 14 + IP-datagram 1500). packet_len must be 100 (the captured
/// length), not 1500 (IP-datagram on-wire) nor 1514 (Ethernet-frame on-wire).
///
/// No `on_wire_len` field exists on ParsedPacket — this is BC-2.02.014 invariant 2's
/// deliberate design constraint. The exhaustive destructuring match of ParsedPacket
/// below enforces that guarantee at compile time: if an `on_wire_len` field were ever
/// added, this test would fail to compile, turning the invariant into a compiler error.
#[test]
fn test_BC_2_02_014_snaplen_truncated_packet_len() {
    // IP-datagram on-wire length = 20 (IP) + 20 (TCP) + 1460 (payload) = 1500 bytes.
    // Ethernet-frame on-wire length = 14 (Ethernet) + 1500 (IP-datagram) = 1514 bytes.
    // We capture only the first 100 bytes (headers + start of payload).
    let on_wire_payload_len: u16 = 1460;
    let captured_len: usize = 100;

    let data = make_snaplen_truncated_tcp(
        [192, 168, 0, 1],
        [192, 168, 0, 2],
        44444,
        80,
        999,
        TCP_ACK,
        on_wire_payload_len,
        captured_len,
    );

    // Confirm our builder actually produced 100 bytes
    assert_eq!(
        data.len(),
        captured_len,
        "builder must produce exactly {captured_len} bytes"
    );

    let parsed = decode_packet(&data, DataLink::ETHERNET)
        .expect("AC-003: snaplen-truncated frame must decode via lax fallback");

    // BC-2.02.014 invariant 2: packet_len == captured length, not on-wire length.
    // IP-datagram on-wire = 1500; Ethernet-frame on-wire = 1514. packet_len must be
    // the captured length (100), not either on-wire figure.
    let ip_datagram_on_wire: usize = 20 + 20 + on_wire_payload_len as usize; // 1500
    let eth_frame_on_wire: usize = 14 + ip_datagram_on_wire; // 1514
    assert_eq!(
        parsed.packet_len, captured_len,
        "AC-003: packet_len ({}) must equal the captured length ({captured_len}), \
         not IP-datagram on-wire ({ip_datagram_on_wire}) nor Ethernet-frame on-wire ({eth_frame_on_wire})",
        parsed.packet_len,
    );

    // BC-2.02.014 invariant 2: exhaustive destructuring of ParsedPacket proves no
    // `on_wire_len` field exists. If such a field were added to the struct, this match
    // would become non-exhaustive and the test would refuse to compile — making the
    // "no separate on_wire_len field" invariant a compiler-enforced guarantee.
    let ParsedPacket {
        packet_len,
        protocol: _,
        transport: _,
        payload: _,
        src_ip: _,
        dst_ip: _,
    } = parsed;
    assert_eq!(
        packet_len, captured_len,
        "AC-003: exhaustive-match packet_len ({packet_len}) must equal captured length ({captured_len})"
    );
}

// ---------------------------------------------------------------------------
// AC-004 / BC-2.02.015 postcondition 4
//
// For a TCP SYN packet, TransportInfo::Tcp.syn = true and
// TransportInfo::Tcp.ack = false.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 4: TCP SYN packet extracts syn=true, ack=false.
///
/// Uses the canonical test vector from BC-2.02.015: TCP SYN frame.
#[test]
fn test_BC_2_02_015_tcp_syn_flags() {
    let data = make_eth_ipv4_tcp(
        [192, 168, 1, 1],
        [192, 168, 1, 2],
        45000,
        80,
        0x0000_0001,
        0,
        TCP_SYN, // SYN only — ACK not set
        &[],
    );

    let parsed = decode_packet(&data, DataLink::ETHERNET).expect("AC-004: SYN frame must decode");

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            // BC-2.02.015 postcondition 4: syn=true iff SYN bit is set
            assert!(*syn, "AC-004: syn must be true for a SYN packet");
            // BC-2.02.015 postcondition 5 complement: ack=false on pure SYN
            assert!(!*ack, "AC-004: ack must be false for a pure SYN packet");
            // Verify other flags are clean
            assert!(!*fin, "AC-004: fin must be false for a pure SYN packet");
            assert!(!*rst, "AC-004: rst must be false for a pure SYN packet");
        }
        other => panic!("AC-004: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-005 / BC-2.02.015 postcondition 5
//
// For a TCP SYN-ACK packet, syn = true and ack = true.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 5: TCP SYN-ACK packet extracts syn=true, ack=true.
#[test]
fn test_BC_2_02_015_tcp_syn_ack_flags() {
    let data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [192, 168, 1, 1],
        443,
        45000,
        0x0000_0100,
        0x0000_0002, // ack number
        TCP_SYN_ACK, // SYN + ACK
        &[],
    );

    let parsed =
        decode_packet(&data, DataLink::ETHERNET).expect("AC-005: SYN-ACK frame must decode");

    match &parsed.transport {
        TransportInfo::Tcp { syn, ack, .. } => {
            // BC-2.02.015 postcondition 5: both syn and ack are true for SYN-ACK
            assert!(*syn, "AC-005: syn must be true for a SYN-ACK packet");
            assert!(*ack, "AC-005: ack must be true for a SYN-ACK packet");
        }
        other => panic!("AC-005: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-006 / BC-2.02.015 postcondition 7 (RST) and postcondition 6 (FIN)
//
// For a TCP RST packet, rst = true.
// For a TCP FIN-ACK packet, fin = true and ack = true.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postconditions 6+7: RST packet has rst=true; FIN-ACK has fin=true, ack=true.
///
/// Both flag combinations tested in one function per the AC naming mandate.
#[test]
fn test_BC_2_02_015_tcp_rst_and_fin_ack_flags() {
    // --- RST packet ---
    let rst_data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        9000,
        80,
        500,
        0,
        TCP_RST, // RST only
        &[],
    );

    let rst_parsed =
        decode_packet(&rst_data, DataLink::ETHERNET).expect("AC-006: RST frame must decode");

    match &rst_parsed.transport {
        TransportInfo::Tcp {
            rst, syn, ack, fin, ..
        } => {
            // BC-2.02.015 postcondition 7: rst=true iff RST bit is set
            assert!(*rst, "AC-006: rst must be true for a RST packet");
            assert!(!*syn, "AC-006: syn must be false for a pure RST packet");
            assert!(!*ack, "AC-006: ack must be false for a pure RST packet");
            assert!(!*fin, "AC-006: fin must be false for a pure RST packet");
        }
        other => panic!("AC-006: expected TransportInfo::Tcp (RST), got: {other:?}"),
    }

    // --- FIN-ACK packet ---
    let fin_ack_data = make_eth_ipv4_tcp(
        [10, 0, 0, 2],
        [10, 0, 0, 1],
        80,
        9000,
        2000,
        1001,        // ack number
        TCP_FIN_ACK, // FIN + ACK
        &[],
    );

    let fin_ack_parsed = decode_packet(&fin_ack_data, DataLink::ETHERNET)
        .expect("AC-006: FIN-ACK frame must decode");

    match &fin_ack_parsed.transport {
        TransportInfo::Tcp {
            fin, ack, syn, rst, ..
        } => {
            // BC-2.02.015 postcondition 6: fin=true for FIN-ACK
            assert!(*fin, "AC-006: fin must be true for a FIN-ACK packet");
            assert!(*ack, "AC-006: ack must be true for a FIN-ACK packet");
            assert!(!*syn, "AC-006: syn must be false for a FIN-ACK packet");
            assert!(!*rst, "AC-006: rst must be false for a FIN-ACK packet");
        }
        other => panic!("AC-006: expected TransportInfo::Tcp (FIN-ACK), got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-007 / BC-2.02.015 postcondition 3
//
// TransportInfo::Tcp.seq_number equals the 32-bit sequence number from the
// TCP header (from etherparse's to_header().sequence_number API).
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 3: seq_number extracted from TCP header bytes.
///
/// Embeds a known canonical value (0xDEAD_BEEF) in the frame's sequence number
/// field and verifies it is extracted unchanged by decode_packet.
/// Also tests with seq=1 (minimal) to exercise a second canonical vector.
#[test]
fn test_BC_2_02_015_tcp_seq_number_extracted() {
    // BC-2.02.015 canonical test vector: TCP SYN frame, seq = 0xDEAD_BEEF
    let canonical_seq: u32 = 0xDEAD_BEEF;

    let data = make_eth_ipv4_tcp(
        [192, 168, 100, 1],
        [192, 168, 100, 2],
        54321,
        443,
        canonical_seq,
        0,
        TCP_SYN,
        &[],
    );

    let parsed = decode_packet(&data, DataLink::ETHERNET)
        .expect("AC-007: frame with known seq_number must decode");

    match &parsed.transport {
        TransportInfo::Tcp { seq_number, .. } => {
            // BC-2.02.015 postcondition 3: seq_number == raw TCP header sequence number
            assert_eq!(
                *seq_number, canonical_seq,
                "AC-007: seq_number ({seq_number:#010x}) must equal the frame's sequence number ({canonical_seq:#010x})"
            );
        }
        other => panic!("AC-007: expected TransportInfo::Tcp, got: {other:?}"),
    }

    // Second canonical vector: seq = 1 (minimal)
    let data2 = make_eth_ipv4_tcp(
        [192, 168, 100, 1],
        [192, 168, 100, 2],
        54321,
        443,
        1,
        0,
        TCP_SYN,
        &[],
    );
    let parsed2 =
        decode_packet(&data2, DataLink::ETHERNET).expect("AC-007: seq=1 frame must decode");
    match &parsed2.transport {
        TransportInfo::Tcp { seq_number, .. } => {
            assert_eq!(
                *seq_number, 1u32,
                "AC-007: seq_number must be 1 for seq=1 canonical vector"
            );
        }
        other => panic!("AC-007: expected TransportInfo::Tcp (seq=1), got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// AC-008 / BC-2.02.015 postcondition 8
//
// ParsedPacket.payload contains the TCP segment payload bytes (bytes after the
// TCP header). For a pure ACK with no data, payload is an empty Vec.
// ---------------------------------------------------------------------------

/// BC-2.02.015 postcondition 8: TCP payload bytes extracted; pure ACK has empty payload.
///
/// Tests two cases:
///   1. TCP segment with a known 50-byte payload — payload.len() == 50.
///   2. Pure ACK with no data — payload is an empty Vec.
#[test]
fn test_BC_2_02_015_tcp_payload_bytes() {
    // BC-2.02.015 canonical test vector: TCP data frame with 50-byte payload
    let known_payload: Vec<u8> = (0u8..50u8).collect();

    let data_with_payload = make_eth_ipv4_tcp(
        [10, 1, 1, 1],
        [10, 1, 1, 2],
        9000,
        8080,
        42,
        0,
        TCP_ACK,
        &known_payload,
    );

    let parsed_with = decode_packet(&data_with_payload, DataLink::ETHERNET)
        .expect("AC-008: frame with 50-byte payload must decode");

    // BC-2.02.015 postcondition 8: payload == TCP segment data bytes
    assert_eq!(
        parsed_with.payload, known_payload,
        "AC-008: payload must contain the 50-byte TCP segment data"
    );
    assert_eq!(
        parsed_with.payload.len(),
        50,
        "AC-008: payload length must be 50 bytes"
    );

    // BC-2.02.015 EC-006: pure ACK (no data) — payload is empty Vec
    let pure_ack_data = make_eth_ipv4_tcp(
        [10, 1, 1, 1],
        [10, 1, 1, 2],
        9001,
        8080,
        43,
        1,
        TCP_ACK,
        &[], // no payload
    );

    let parsed_ack = decode_packet(&pure_ack_data, DataLink::ETHERNET)
        .expect("AC-008: pure ACK frame must decode");

    // BC-2.02.015 postcondition 8: empty payload for pure ACK
    assert_eq!(
        parsed_ack.payload,
        Vec::<u8>::new(),
        "AC-008: payload must be empty Vec for pure ACK (no TCP data)"
    );
}

// ---------------------------------------------------------------------------
// AC-009 / BC-2.02.015 invariant 3
//
// PSH and URG flags are NOT extracted; they are absent from TransportInfo::Tcp.
// Adding them would require a struct change — this is a deliberate scope constraint.
// ---------------------------------------------------------------------------

/// BC-2.02.015 invariant 3: TransportInfo::Tcp struct has no psh or urg fields.
///
/// Pattern-matches ALL named fields of TransportInfo::Tcp exhaustively. If `psh`
/// or `urg` were added to the struct this test would fail to compile — the
/// exhaustive match would leave new fields unaccounted for, turning scope creep
/// into a compile error. This is the structural enforcement mechanism.
///
/// Also verifies the struct has exactly the seven fields documented in BC-2.02.015:
/// src_port, dst_port, seq_number, syn, ack, fin, rst.
#[test]
fn test_BC_2_02_015_psh_urg_not_in_transport_info() {
    // Build a TCP frame with PSH (0x08) and URG (0x20) set in the flags byte.
    // If the decoder were to extract psh/urg into new fields, they would be true here.
    // The absence of those fields in the struct is what we are asserting.
    let psh_urg_flags: u8 = 0x08 | 0x20 | TCP_ACK; // PSH | URG | ACK

    let data = make_eth_ipv4_tcp(
        [172, 16, 0, 1],
        [172, 16, 0, 2],
        12000,
        8080,
        77,
        78,
        psh_urg_flags,
        b"psh-urg-payload",
    );

    let parsed =
        decode_packet(&data, DataLink::ETHERNET).expect("AC-009: PSH+URG frame must decode");

    // BC-2.02.015 invariant 3: exhaustive struct match — only the seven documented
    // fields exist. If psh/urg are ever added this match becomes non-exhaustive and
    // the test will fail to compile, acting as a scope-guard.
    match &parsed.transport {
        TransportInfo::Tcp {
            src_port,
            dst_port,
            seq_number,
            syn,
            ack,
            fin,
            rst,
        } => {
            // The match itself proves no psh/urg fields exist on the struct.
            // Verify the seven expected fields are present with correct types.
            let _: &u16 = src_port;
            let _: &u16 = dst_port;
            let _: &u32 = seq_number;
            let _: &bool = syn;
            let _: &bool = ack;
            let _: &bool = fin;
            let _: &bool = rst;

            // ACK was set in the flags byte; it must be extracted
            assert!(
                *ack,
                "AC-009: ack must be true (it was set in the flags byte)"
            );
            // SYN, FIN, RST were not set
            assert!(!*syn, "AC-009: syn must be false");
            assert!(!*fin, "AC-009: fin must be false");
            assert!(!*rst, "AC-009: rst must be false");
            // PSH and URG are intentionally absent — there is no field to check.
            // Their absence from the exhaustive match is the structural assertion.
        }
        other => panic!("AC-009: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Edge-case tests EC-001..EC-006 from STORY-005.md
// ---------------------------------------------------------------------------

/// EC-001 (BC-2.02.014 EC-001): 1500-byte Ethernet frame — packet_len == 1500.
///
/// The canonical maximum-transmission-unit Ethernet frame size.
/// Ethernet(14) + IPv4(20) + TCP(20) + payload(1446) = 1500 bytes total.
#[test]
fn test_BC_2_02_014_ec001_1500_byte_frame_packet_len() {
    // EC-001: 1500-byte frame; Eth(14) + IPv4(20) + TCP(20) = 54 header bytes
    // payload must be 1500 - 54 = 1446 bytes
    let payload = vec![0xCC; 1446];

    let data = make_eth_ipv4_tcp(
        [10, 10, 10, 1],
        [10, 10, 10, 2],
        50000,
        443,
        1,
        0,
        TCP_ACK,
        &payload,
    );

    assert_eq!(
        data.len(),
        1500,
        "EC-001: frame builder must produce exactly 1500 bytes"
    );

    let parsed =
        decode_packet(&data, DataLink::ETHERNET).expect("EC-001: 1500-byte frame must decode");

    // BC-2.02.014 EC-001: packet_len == 1500
    assert_eq!(
        parsed.packet_len, 1500,
        "EC-001: packet_len must be 1500 for a 1500-byte Ethernet frame"
    );
}

/// EC-002 (BC-2.02.014 EC-004 + BC-2.02.015 EC-006): 54-byte TCP ACK (no payload).
///
/// Ethernet(14) + IPv4(20) + TCP(20) = 54 bytes with zero payload.
/// packet_len == 54 and payload is an empty Vec.
#[test]
fn test_BC_2_02_014_ec002_54_byte_pure_ack() {
    let data = make_eth_ipv4_tcp(
        [192, 168, 1, 10],
        [10, 0, 0, 1],
        55555,
        80,
        1000,
        2000,
        TCP_ACK, // pure ACK, no payload
        &[],
    );

    assert_eq!(
        data.len(),
        54,
        "EC-002: frame builder must produce exactly 54 bytes for header-only TCP"
    );

    let parsed = decode_packet(&data, DataLink::ETHERNET)
        .expect("EC-002: 54-byte pure ACK frame must decode");

    // BC-2.02.014 EC-004: packet_len == 54
    assert_eq!(
        parsed.packet_len, 54,
        "EC-002: packet_len must be 54 for a 54-byte pure ACK frame"
    );

    // BC-2.02.015 EC-006: payload is empty Vec for pure ACK
    assert_eq!(
        parsed.payload,
        Vec::<u8>::new(),
        "EC-002: payload must be empty Vec for pure ACK frame"
    );
}

/// EC-003 (BC-2.02.014 EC-003): Snaplen-truncated at 100 bytes.
///
/// IP header advertises 1500 bytes on-wire but only 100 bytes are captured.
/// packet_len must equal 100 (the captured length), not 1500.
#[test]
fn test_BC_2_02_014_ec003_snaplen_truncated_at_100() {
    // EC-003: on-wire frame is 1500 bytes (IP total = 1500), but only 100 captured.
    // IP total_length = 14 (Eth is outside IP, so IP total = 20 + 20 + 1460 = 1500)
    // We truncate to 100 bytes (headers + first 46 bytes of payload).
    let data = make_snaplen_truncated_tcp(
        [192, 168, 5, 1],
        [8, 8, 8, 8],
        60000,
        443,
        12345,
        TCP_ACK,
        1460, // on_wire_payload_len → IP total_length = 1500
        100,  // captured_len
    );

    assert_eq!(
        data.len(),
        100,
        "EC-003: builder must produce exactly 100 captured bytes"
    );

    let parsed = decode_packet(&data, DataLink::ETHERNET)
        .expect("EC-003: snaplen-truncated frame must decode via lax fallback");

    // BC-2.02.014 EC-003: packet_len == 100 (captured), not 1500 (on-wire)
    assert_eq!(
        parsed.packet_len, 100,
        "EC-003: packet_len must be 100 (captured length), not 1500 (on-wire length)"
    );
}

/// EC-004 (BC-2.02.015 EC-005): seq_number = 0xFFFFFFFF (max u32, wraparound boundary).
///
/// The sequence number field must be 4294967295 with no overflow or truncation.
#[test]
fn test_BC_2_02_015_ec004_seq_number_max_u32() {
    let max_seq: u32 = 0xFFFF_FFFF; // 4294967295

    let data = make_eth_ipv4_tcp(
        [10, 0, 0, 1],
        [10, 0, 0, 2],
        1024,
        9090,
        max_seq,
        0,
        TCP_SYN,
        &[],
    );

    let parsed =
        decode_packet(&data, DataLink::ETHERNET).expect("EC-004: max seq_number frame must decode");

    match &parsed.transport {
        TransportInfo::Tcp { seq_number, .. } => {
            // BC-2.02.015 EC-005: seq_number == 0xFFFFFFFF; no overflow
            assert_eq!(
                *seq_number, max_seq,
                "EC-004: seq_number must be 0xFFFFFFFF (4294967295) with no overflow or truncation"
            );
            assert_eq!(
                *seq_number, 4_294_967_295u32,
                "EC-004: seq_number must equal 4294967295 (u32::MAX)"
            );
        }
        other => panic!("EC-004: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

/// EC-005 (BC-2.02.015): All four flags simultaneously set.
///
/// Pathological packet with syn=true, ack=true, fin=true, rst=true.
/// All four tracked flags must be extracted correctly regardless of combination.
#[test]
fn test_BC_2_02_015_ec005_all_four_flags_set() {
    let data = make_eth_ipv4_tcp(
        [172, 31, 0, 1],
        [172, 31, 0, 2],
        7777,
        8888,
        0xABCD_1234,
        0,
        TCP_ALL_FOUR, // FIN | SYN | RST | ACK
        &[],
    );

    let parsed =
        decode_packet(&data, DataLink::ETHERNET).expect("EC-005: all-flags-set frame must decode");

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            // EC-005: all four flags must be true simultaneously
            assert!(*syn, "EC-005: syn must be true when all four flags are set");
            assert!(*ack, "EC-005: ack must be true when all four flags are set");
            assert!(*fin, "EC-005: fin must be true when all four flags are set");
            assert!(*rst, "EC-005: rst must be true when all four flags are set");
        }
        other => panic!("EC-005: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

/// EC-006 (BC-2.02.015 EC-007): No flags set (data segment).
///
/// A TCP data segment with a zeroed flags byte. All four extracted flags
/// must be false — the decoder must not invent flag values.
#[test]
fn test_BC_2_02_015_ec006_no_flags_set() {
    let data = make_eth_ipv4_tcp(
        [10, 5, 0, 1],
        [10, 5, 0, 2],
        3000,
        4000,
        200,
        0,
        0x00, // zero flags byte — no flags set
        b"data-segment-no-flags",
    );

    let parsed = decode_packet(&data, DataLink::ETHERNET)
        .expect("EC-006: zero-flags data segment must decode");

    match &parsed.transport {
        TransportInfo::Tcp {
            syn, ack, fin, rst, ..
        } => {
            // EC-006: all four flags must be false when no flags are set
            assert!(!*syn, "EC-006: syn must be false when flags byte is 0x00");
            assert!(!*ack, "EC-006: ack must be false when flags byte is 0x00");
            assert!(!*fin, "EC-006: fin must be false when flags byte is 0x00");
            assert!(!*rst, "EC-006: rst must be false when flags byte is 0x00");
        }
        other => panic!("EC-006: expected TransportInfo::Tcp, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// EC-007 / BC-2.02.014 EC-002: 60-byte minimum Ethernet frame with padding
// ---------------------------------------------------------------------------

/// EC-007 (BC-2.02.014 EC-002): 60-byte minimum Ethernet frame with padding.
///
/// Ethernet mandates a minimum frame body of 60 bytes (excluding the 4-byte FCS
/// that the NIC strips before the OS delivers the frame to the capture buffer).
/// When a real IP datagram is smaller than 46 bytes of Ethernet payload, the NIC
/// or switch pads the frame to the minimum. The pad bytes are NOT part of the IP
/// datagram.
///
/// Frame layout:
///   Ethernet header : 14 bytes
///   IPv4 header     : 20 bytes
///   TCP header      : 20 bytes
///   TCP payload     :  0 bytes  (pure ACK, no data)
///   Ethernet padding:  6 bytes  (injected after the real IP datagram)
///   Total           : 60 bytes
///
/// The IPv4 total_length field is 40 (20 IP + 20 TCP + 0 payload) — it reflects
/// only the real IP datagram; it does NOT count the 6 Ethernet pad bytes.
///
/// BC-2.02.014 invariant on packet_len: packet_len must be 60 (data.len()), not 40.
/// BC-2.02.014 / BC-2.02.015: the 6 Ethernet padding bytes must NOT appear in payload
/// — they are outside the IP datagram boundary and are not TCP payload.
#[test]
fn test_BC_2_02_014_ec007_60_byte_padded_frame() {
    // Build the real IP + TCP headers with zero payload.
    // make_eth_ipv4_tcp sets IP total_length = 20 (IP) + 20 (TCP) + 0 (payload) = 40.
    let mut frame = make_eth_ipv4_tcp(
        [10, 20, 30, 40],
        [10, 20, 30, 41],
        1234,
        80,
        0x0000_0001, // seq
        0,           // ack number
        TCP_ACK,
        &[], // zero TCP payload
    );

    // frame is 54 bytes (14 Eth + 20 IP + 20 TCP). Ethernet minimum body is 46 bytes
    // (60 - 14 Eth header), so we need 60 - 54 = 6 bytes of padding.
    let eth_pad_len = 6usize;
    frame.extend(std::iter::repeat_n(0x00u8, eth_pad_len));

    assert_eq!(
        frame.len(),
        60,
        "EC-007: padded frame must be exactly 60 bytes"
    );

    let parsed = decode_packet(&frame, DataLink::ETHERNET)
        .expect("EC-007: 60-byte padded Ethernet frame must decode");

    // (a) packet_len == 60: equals data.len(), the full padded frame.
    assert_eq!(
        parsed.packet_len, 60,
        "EC-007: packet_len must be 60 (full padded frame length), not 54 (IP datagram)"
    );

    // (b) payload is empty: the 6 Ethernet padding bytes are outside the IP datagram
    // and must NOT be counted as TCP payload. The IP total_length (40) correctly
    // delimits the real datagram; etherparse uses it to bound the TCP payload slice.
    assert_eq!(
        parsed.payload,
        Vec::<u8>::new(),
        "EC-007: payload must be empty — Ethernet padding bytes must not appear in TCP payload"
    );
}
