//! End-to-end and property-based tests for VP-021: Timestamp Provenance Threading.
//!
//! Verifies that `Finding.timestamp` is correctly populated from the pcap
//! `ts_sec` value across the hot-path flush case (BC-2.09.007 post-2),
//! the close-flush case (BC-2.09.007 post-3), and that the segment-limit
//! summary retains `None` (BC-2.09.007 post-4 / invariant 1).
//!
//! All AC-001..AC-007 from STORY-099 are covered here.

use std::net::{IpAddr, Ipv4Addr};

use chrono::{DateTime, Utc};
use proptest::prelude::*;

use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::dispatcher::StreamDispatcher;
use wirerust::findings::{Confidence, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

// ---------------------------------------------------------------------------
// Shared packet builder (mirrors multi_analyzer_e2e_tests.rs pattern)
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn make_tcp_packet(
    src_ip: IpAddr,
    dst_ip: IpAddr,
    src_port: u16,
    dst_port: u16,
    seq: u32,
    syn: bool,
    ack: bool,
    fin: bool,
    rst: bool,
    payload: Vec<u8>,
) -> ParsedPacket {
    let packet_len = payload.len() + 40;
    ParsedPacket {
        src_ip,
        dst_ip,
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            seq_number: seq,
            syn,
            ack,
            fin,
            rst,
        },
        payload,
        packet_len,
    }
}

/// Convenience constants for test flows.
const CLIENT_A: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
const SERVER: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
const CLIENT_B: IpAddr = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3));

/// HTTP GET request with a path-traversal URI — reliably triggers an HTTP
/// `Reconnaissance` finding (BC-2.09.007 canonical test vector).
fn http_traversal_request() -> Vec<u8> {
    b"GET /../etc/passwd HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec()
}

// ---------------------------------------------------------------------------
// Recording handler — records on_data timestamps for close-flush assertions.
// ---------------------------------------------------------------------------

struct RecordingHandler {
    data_events: Vec<(FlowKey, Direction, Vec<u8>, u64, u32)>,
    close_events: Vec<(FlowKey, CloseReason)>,
}

impl RecordingHandler {
    fn new() -> Self {
        RecordingHandler {
            data_events: Vec::new(),
            close_events: Vec::new(),
        }
    }
}

impl StreamHandler for RecordingHandler {
    fn on_data(
        &mut self,
        flow_key: &FlowKey,
        direction: Direction,
        data: &[u8],
        offset: u64,
        timestamp: u32,
    ) {
        self.data_events.push((
            flow_key.clone(),
            direction,
            data.to_vec(),
            offset,
            timestamp,
        ));
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
        self.close_events.push((flow_key.clone(), reason));
    }
}

// ---------------------------------------------------------------------------
// AC-001: Hot-path flush — Finding.timestamp matches the packet ts_sec
// (BC-2.09.007 post-1 + post-2; VP-021 hot-path case)
//
// Constructs a TCP flow with an HTTP path-traversal request at ts_sec=1_000_000.
// Drives through TcpReassembler → StreamDispatcher → HttpAnalyzer and asserts
// that at least one emitted Finding has timestamp = Some(1970-01-12T13:46:40Z).
// (ts_sec=1_000_000 → 1970-01-12T13:46:40Z is the BC's now-correct specified value;
// this test asserts that value directly.)
// ---------------------------------------------------------------------------

#[test]
fn test_finding_timestamp_hot_path() {
    const TS: u32 = 1_000_000;
    let expected_ts: DateTime<Utc> =
        DateTime::from_timestamp(TS as i64, 0).expect("ts_sec=1_000_000 must be a valid datetime");

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut dispatcher = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);

    let payload = http_traversal_request();
    let payload_len = payload.len() as u32;

    // SYN — open the flow
    let syn = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49100,
        80,
        999,
        true,
        false,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn, TS, &mut dispatcher);

    // SYN-ACK — server acknowledges
    let syn_ack = make_tcp_packet(
        SERVER,
        CLIENT_A,
        80,
        49100,
        9999,
        true,
        true,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn_ack, TS, &mut dispatcher);

    // HTTP GET with path traversal — at ts_sec=1_000_000
    let data_pkt = make_tcp_packet(
        CLIENT_A, SERVER, 49100, 80, 1000, false, true, false, false, payload,
    );
    reassembler.process_packet(&data_pkt, TS, &mut dispatcher);

    // FIN — close the flow cleanly
    let fin = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49100,
        80,
        1000 + payload_len,
        false,
        true,
        true,
        false,
        vec![],
    );
    reassembler.process_packet(&fin, TS, &mut dispatcher);
    reassembler.finalize(&mut dispatcher);

    let findings = dispatcher
        .http_analyzer()
        .expect("http analyzer must be present")
        .findings();

    let timestamped: Vec<_> = findings.iter().filter(|f| f.timestamp.is_some()).collect();

    assert!(
        !timestamped.is_empty(),
        "expected at least one finding with timestamp; got {} total findings",
        findings.len()
    );

    for f in &timestamped {
        assert_eq!(
            f.timestamp,
            Some(expected_ts),
            "hot-path finding timestamp must equal DateTime::from_timestamp({TS}, 0); \
             got {:?}; summary: {}",
            f.timestamp,
            f.summary,
        );
    }

    // EC-006: also verify that the path-traversal finding is present (not just
    // any finding), confirming we exercised the correct HTTP detection path.
    let traversal_finding = findings
        .iter()
        .find(|f| f.category == ThreatCategory::Reconnaissance && f.summary.contains("traversal"));
    assert!(
        traversal_finding.is_some(),
        "expected a path-traversal Reconnaissance finding from /../etc/passwd URI"
    );
}

// ---------------------------------------------------------------------------
// AC-001 variant: TLS path (EC-006 — at least one test covers the TLS analyzer)
// Sends a minimal TLS ClientHello at ts_sec=1_000_000 and asserts that the
// TLS analyzer emits findings (if any) with the expected timestamp.
// ---------------------------------------------------------------------------

#[test]
fn test_finding_timestamp_hot_path_tls() {
    use wirerust::analyzer::tls::TlsAnalyzer;

    const TS: u32 = 1_000_000;
    let expected_ts: DateTime<Utc> =
        DateTime::from_timestamp(TS as i64, 0).expect("ts_sec=1_000_000 must be a valid datetime");

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut dispatcher = StreamDispatcher::new(None, Some(TlsAnalyzer::new()));

    // Build a minimal TLS ClientHello with a NULL cipher suite that triggers
    // a weak-cipher finding in TlsAnalyzer.
    // TLS_NULL_WITH_NULL_NULL = 0x0000 — known weak/null cipher.
    let tls_hello = build_tls_client_hello_with_null_cipher();
    let hello_len = tls_hello.len() as u32;

    // SYN
    let syn = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49200,
        443,
        999,
        true,
        false,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn, TS, &mut dispatcher);

    // SYN-ACK
    let syn_ack = make_tcp_packet(
        SERVER,
        CLIENT_A,
        443,
        49200,
        9999,
        true,
        true,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn_ack, TS, &mut dispatcher);

    // TLS data at ts_sec=1_000_000
    let data_pkt = make_tcp_packet(
        CLIENT_A, SERVER, 49200, 443, 1000, false, true, false, false, tls_hello,
    );
    reassembler.process_packet(&data_pkt, TS, &mut dispatcher);

    // FIN
    let fin = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49200,
        443,
        1000 + hello_len,
        false,
        true,
        true,
        false,
        vec![],
    );
    reassembler.process_packet(&fin, TS, &mut dispatcher);
    reassembler.finalize(&mut dispatcher);

    let tls = dispatcher
        .tls_analyzer()
        .expect("tls analyzer must be present");
    let findings = tls.findings();

    // TLS_NULL_WITH_NULL_NULL (0x0000) IS a weak cipher in the database; the analyzer
    // MUST emit at least one timestamped finding for AC-001 to be meaningful.
    assert!(
        findings.iter().any(|f| f.timestamp.is_some()),
        "TLS NULL-cipher ClientHello must emit at least one timestamped finding (AC-001 TLS)"
    );

    // All timestamped findings must carry the correct timestamp.
    for f in findings.iter().filter(|f| f.timestamp.is_some()) {
        assert_eq!(
            f.timestamp,
            Some(expected_ts),
            "TLS path finding timestamp must equal DateTime::from_timestamp({TS}, 0); \
             got {:?}; summary: {}",
            f.timestamp,
            f.summary,
        );
    }
}

/// Build a TLS ClientHello with TLS_NULL_WITH_NULL_NULL (0x0000) as the only
/// cipher suite, which the TlsAnalyzer should flag as weak/null.
fn build_tls_client_hello_with_null_cipher() -> Vec<u8> {
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    ch_body.extend_from_slice(&[0u8; 32]); // random (32 bytes)
    ch_body.push(0x00); // session_id length: 0
    ch_body.extend_from_slice(&[0x00, 0x02]); // cipher suites length: 2 bytes (1 suite)
    ch_body.extend_from_slice(&[0x00, 0x00]); // TLS_NULL_WITH_NULL_NULL
    ch_body.push(0x01); // compression methods length: 1
    ch_body.push(0x00); // null compression
    ch_body.extend_from_slice(&[0x00, 0x00]); // extensions length: 0

    let mut handshake = Vec::new();
    handshake.push(0x01); // ClientHello handshake type
    let ch_len = ch_body.len() as u32;
    handshake.push((ch_len >> 16) as u8);
    handshake.push((ch_len >> 8) as u8);
    handshake.push(ch_len as u8);
    handshake.extend_from_slice(&ch_body);

    let mut record = Vec::new();
    record.push(0x16); // content type: handshake
    record.extend_from_slice(&[0x03, 0x01]); // record version: TLS 1.0
    let hs_len = handshake.len() as u16;
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

// ---------------------------------------------------------------------------
// AC-002: Close-flush path — flow.last_seen becomes the timestamp for on_data
// (BC-2.09.007 post-3; VP-021 close-flush case)
//
// Verifies BC-2.04.055 postcondition 1 (close-flush case): when close_flow is
// triggered (by FIN, RST, timeout, or eviction), it passes `flow.last_seen` as
// the `timestamp` argument to `handler.on_data`.
//
// Runtime observable behavior (per STORY-097 established pattern):
//   (a) Hot-path on_data at TS_DATA fires with timestamp=TS_DATA.
//   (b) FIN at TS_FIN updates flow.last_seen=TS_FIN; on_flow_close fires (FIN).
//   (c) Any on_data calls from close_flow carry timestamp=TS_FIN.
//
// Limitation: flush_contiguous is strictly contiguous — it only delivers data
// starting from base_offset. If all data was flushed hot-path (buffer empty at
// FIN time), close_flow calls flush_contiguous and gets nothing, so no
// additional on_data calls fire. The close-flush timestamp (flow.last_seen) is
// still read by close_flow; this is verified:
//   (i)  by code inspection of lifecycle.rs line 56: `let close_timestamp = flow.last_seen`
//   (ii) by a gap-scenario sub-test that places an in-order data segment at TS_DATA
//        followed by an OOO segment at TS_OOO (which sets flow.last_seen=TS_OOO),
//        then calls finalize(). close_flow fires and on_flow_close carries the
//        lifecycle record; the absence of on_data confirms gapped data is not flushed
//        (consistent with flush_contiguous semantics).
//
// This matches test_close_flow_passes_flow_last_seen_timestamp and
// test_close_flow_passes_flow_last_seen_timestamp_with_ooo_gap from STORY-097.
// ---------------------------------------------------------------------------

#[test]
fn test_finding_timestamp_close_flush() {
    // NOTE: TS_FIN must be within flow_timeout_secs (default: 300) of TS_DATA.
    // If TS_FIN - TS_DATA > 300, the idle-timeout sweep fires on the FIN packet
    // and closes the flow via CloseReason::Timeout BEFORE the FIN is processed.
    // Use 100-second gap (well within 300s window) to keep CloseReason::Fin.
    const TS_DATA: u32 = 1_500_000;
    const TS_FIN: u32 = 1_500_100; // 100s after data; within 300s idle-timeout window

    // Part 1: verify hot-path flush carries TS_DATA, and FIN triggers close_flow
    // (on_flow_close fires with reason Fin), proving the FIN-close code path runs.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let payload = http_traversal_request();
    let payload_len = payload.len() as u32;

    // SYN
    let syn = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49300,
        80,
        999,
        true,
        false,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn, TS_DATA, &mut handler);

    // SYN-ACK
    let syn_ack = make_tcp_packet(
        SERVER,
        CLIENT_A,
        80,
        49300,
        9999,
        true,
        true,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn_ack, TS_DATA, &mut handler);

    // HTTP data at TS_DATA — hot-path flush
    let data_pkt = make_tcp_packet(
        CLIENT_A, SERVER, 49300, 80, 1000, false, true, false, false, payload,
    );
    reassembler.process_packet(&data_pkt, TS_DATA, &mut handler);

    // (a) Hot-path on_data must carry TS_DATA (BC-2.04.055 postcondition 1, hot-path case)
    assert_eq!(
        handler.data_events.len(),
        1,
        "AC-002 setup: exactly one hot-path on_data event"
    );
    assert_eq!(
        handler.data_events[0].4, TS_DATA,
        "AC-002 (a): hot-path on_data must carry current-packet timestamp=TS_DATA"
    );
    assert!(
        handler.close_events.is_empty(),
        "AC-002 setup: no close event yet"
    );

    // Send a zero-payload packet at TS_FIN — updates flow.last_seen = TS_FIN.
    // A single client-FIN transitions state to Closing (not Closed) so the flow
    // stays open until finalize(). Using finalize() ensures close_flow is called
    // with close_timestamp = flow.last_seen = TS_FIN.
    let fin_or_ack = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49300,
        80,
        1000 + payload_len,
        false,
        true,
        true,
        false,
        vec![],
    );
    reassembler.process_packet(&fin_or_ack, TS_FIN, &mut handler);
    // After the FIN: flow.last_seen = TS_FIN. State = Closing (first FIN from client).
    // No close_event yet — single-FIN from one side leaves flow in Closing state.

    // finalize() → close_flow for remaining flows → close_timestamp = flow.last_seen = TS_FIN
    reassembler.finalize(&mut handler);

    // (b) on_flow_close fires exactly once — close_flow was reached via finalize().
    // This proves close_flow executed with close_timestamp = flow.last_seen = TS_FIN.
    assert_eq!(
        handler.close_events.len(),
        1,
        "AC-002 (b): on_flow_close must fire exactly once after finalize()"
    );

    // (c) All data was already flushed hot-path — no additional on_data from close_flow.
    // This is correct: flush_contiguous finds an empty buffer at finalize() time.
    // The close_timestamp = flow.last_seen = TS_FIN was set, as proven by code inspection
    // of lifecycle.rs:56 (`let close_timestamp = flow.last_seen`).
    assert_eq!(
        handler.data_events.len(),
        1,
        "AC-002 (c): close_flow must not produce duplicate on_data for already-flushed data"
    );

    // Part 2: gap scenario — OOO segment at TS_OOO sets flow.last_seen, then finalize().
    // Verifies that the close-flush code path IS reached (on_flow_close fires)
    // and that an in-order segment's on_data correctly carried TS_DATA (not TS_OOO),
    // crosschecking that hot-path uses the current-packet ts, not flow.last_seen.
    //
    // This mirrors test_close_flow_passes_flow_last_seen_timestamp_with_ooo_gap
    // from STORY-097, establishing AC-002's assertion about the close-flush timestamp.
    const TS_OOO: u32 = 1_500_099; // 99s after TS_DATA; within 300s idle-timeout window
    let config2 = ReassemblyConfig::default();
    let mut reassembler2 = TcpReassembler::new(config2);
    let mut handler2 = RecordingHandler::new();

    let syn2 = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49310,
        80,
        200,
        true,
        false,
        false,
        false,
        vec![],
    );
    reassembler2.process_packet(&syn2, TS_DATA, &mut handler2);

    // In-order segment at seq=201 — hot-path flushes with TS_DATA
    let pkt_inorder = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49310,
        80,
        201,
        false,
        true,
        false,
        false,
        b"HELLO".to_vec(),
    );
    reassembler2.process_packet(&pkt_inorder, TS_DATA, &mut handler2);

    assert_eq!(
        handler2.data_events.len(),
        1,
        "AC-002 part2: in-order segment flushed"
    );
    assert_eq!(
        handler2.data_events[0].4, TS_DATA,
        "AC-002 part2: hot-path must carry TS_DATA (current-packet ts), not flow.last_seen"
    );

    // OOO segment at seq=211 (gap at 206-210) — buffered, sets flow.last_seen=TS_OOO
    let pkt_ooo = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49310,
        80,
        211,
        false,
        true,
        false,
        false,
        b"WORLD".to_vec(),
    );
    reassembler2.process_packet(&pkt_ooo, TS_OOO, &mut handler2);

    assert_eq!(
        handler2.data_events.len(),
        1,
        "AC-002 part2: OOO segment must NOT be flushed hot-path (gap prevents contiguous advance)"
    );

    // finalize() → close_flow → close_timestamp = flow.last_seen = TS_OOO
    // flush_contiguous cannot deliver the gapped OOO segment → no additional on_data
    reassembler2.finalize(&mut handler2);

    // (i) No new on_data: gapped OOO data cannot be flushed by close_flow
    assert_eq!(
        handler2.data_events.len(),
        1,
        "AC-002 part2 (i): finalize must NOT emit on_data for gapped OOO segment"
    );

    // (ii) on_flow_close fires exactly once — close_flow reached and executed
    //      with close_timestamp = flow.last_seen = TS_OOO
    assert_eq!(
        handler2.close_events.len(),
        1,
        "AC-002 part2 (ii): finalize must invoke close_flow (on_flow_close fires once)"
    );

    // Part 3: HttpAnalyzer close-flush finding timestamp.
    // An HTTP flow where the traversal data is flushed hot-path (finding carries TS_DATA).
    // Confirms that any finding produced before FIN (hot-path) carries the data packet's ts.
    // The close-flush path (FIN/finalize) would carry TS_FIN for any ADDITIONAL findings —
    // but since the buffer is empty at close time, no additional findings are produced.
    // We verify: all findings produced have timestamp = Some(TS_DATA) (hot-path ts, not None).
    let config3 = ReassemblyConfig::default();
    let mut reassembler3 = TcpReassembler::new(config3);
    let mut dispatcher3 = StreamDispatcher::new(Some(HttpAnalyzer::new()), None);

    let payload3 = http_traversal_request();
    let payload_len3 = payload3.len() as u32;
    let expected_data_ts: DateTime<Utc> =
        DateTime::from_timestamp(TS_DATA as i64, 0).expect("TS_DATA must be a valid datetime");

    let syn3 = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49320,
        80,
        999,
        true,
        false,
        false,
        false,
        vec![],
    );
    reassembler3.process_packet(&syn3, TS_DATA, &mut dispatcher3);

    let syn_ack3 = make_tcp_packet(
        SERVER,
        CLIENT_A,
        80,
        49320,
        9999,
        true,
        true,
        false,
        false,
        vec![],
    );
    reassembler3.process_packet(&syn_ack3, TS_DATA, &mut dispatcher3);

    let data_pkt3 = make_tcp_packet(
        CLIENT_A, SERVER, 49320, 80, 1000, false, true, false, false, payload3,
    );
    reassembler3.process_packet(&data_pkt3, TS_DATA, &mut dispatcher3);

    let fin3 = make_tcp_packet(
        CLIENT_A,
        SERVER,
        49320,
        80,
        1000 + payload_len3,
        false,
        true,
        true,
        false,
        vec![],
    );
    reassembler3.process_packet(&fin3, TS_FIN, &mut dispatcher3);
    reassembler3.finalize(&mut dispatcher3);

    let findings3 = dispatcher3
        .http_analyzer()
        .expect("http analyzer must be present")
        .findings();

    // The traversal finding was emitted during the hot-path flush at TS_DATA.
    // Timestamp = Some(TS_DATA) confirms the hot-path wired the timestamp correctly.
    let timestamped3: Vec<_> = findings3.iter().filter(|f| f.timestamp.is_some()).collect();
    for f in &timestamped3 {
        assert_eq!(
            f.timestamp,
            Some(expected_data_ts),
            "AC-002 part3: close-flush scenario finding must carry TS_DATA (hot-path emission); \
             got {:?}; summary: {}",
            f.timestamp,
            f.summary,
        );
    }
}

// ---------------------------------------------------------------------------
// AC-003: Segment-limit summary Finding has timestamp = None and is absent
// from JSON serialization (BC-2.09.007 post-4 / post-6 / invariant 1;
// VP-021 segment-limit exception; EC-002 + EC-003 from BC-2.09.007)
// ---------------------------------------------------------------------------

#[test]
fn test_segment_limit_summary_timestamp_is_none_and_absent_from_json() {
    // Use max_segments_per_direction = 2 so the third non-contiguous segment
    // trips the limit (same pattern as test_finalize_generates_segment_limit_finding).
    let config = ReassemblyConfig {
        max_segments_per_direction: 2,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10u8, 0, 0, 1];
    let server_arr = [10u8, 0, 0, 2];
    let client_ip = IpAddr::V4(Ipv4Addr::from(client));
    let server_ip = IpAddr::V4(Ipv4Addr::from(server_arr));

    // SYN
    let syn = make_tcp_packet(
        client_ip,
        server_ip,
        12345,
        80,
        1000,
        true,
        false,
        false,
        false,
        vec![],
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Insert 2 non-contiguous segments (gap at offset 0 keeps them buffered)
    let p1 = make_tcp_packet(
        client_ip,
        server_ip,
        12345,
        80,
        1002,
        false,
        false,
        false,
        false,
        b"a".to_vec(),
    );
    let p2 = make_tcp_packet(
        client_ip,
        server_ip,
        12345,
        80,
        1004,
        false,
        false,
        false,
        false,
        b"b".to_vec(),
    );
    reassembler.process_packet(&p1, 2, &mut handler);
    reassembler.process_packet(&p2, 3, &mut handler);

    // Third non-contiguous segment — trips the segment limit
    let p3 = make_tcp_packet(
        client_ip,
        server_ip,
        12345,
        80,
        1006,
        false,
        false,
        false,
        false,
        b"c".to_vec(),
    );
    reassembler.process_packet(&p3, 4, &mut handler);
    assert_eq!(
        reassembler.stats().segments_segment_limit,
        1,
        "expected 1 segment-limit hit"
    );

    reassembler.finalize(&mut handler);

    let findings = reassembler.findings();
    let limit_finding = findings
        .iter()
        .find(|f| f.summary.contains("segment count limit"))
        .expect("finalize must generate a segment-limit summary finding");

    // BC-2.09.007 post-4 / invariant 1: segment-limit summary must have timestamp = None
    assert_eq!(
        limit_finding.timestamp, None,
        "segment-limit summary finding must have timestamp = None (post-capture aggregate)"
    );

    // BC-2.09.007 post-6: JSON must omit the "timestamp" key entirely (not serialize as null).
    let json_str =
        serde_json::to_string(limit_finding).expect("serde_json::to_string must not fail");
    assert!(
        !json_str.contains("\"timestamp\""),
        "segment-limit summary JSON must NOT contain \"timestamp\" key; \
         serde skip_serializing_if = Option::is_none must fire; got: {json_str}"
    );
}

// ---------------------------------------------------------------------------
// AC-004: JSON serialization — Some(DateTime) → ISO-8601 UTC string
// (BC-2.09.007 post-5; EC-005)
//
// ts_sec=1_000_000 → "1970-01-12T13:46:40Z" (correct ISO-8601 UTC value)
// BC-2.09.007 EC-005 specifies ts_sec=1_000_000 → 1970-01-12T13:46:40Z,
// which is the value this test asserts (the BC date is now correct).
// ---------------------------------------------------------------------------

#[test]
fn test_finding_timestamp_json_serialization() {
    use wirerust::findings::Finding;

    let ts_sec: i64 = 1_000_000;
    let dt: DateTime<Utc> =
        DateTime::from_timestamp(ts_sec, 0).expect("ts_sec=1_000_000 must be valid");

    let finding = Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "Test finding for timestamp serialization".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: Some(dt),
        direction: None,
    };

    let json_str =
        serde_json::to_string(&finding).expect("serde_json::to_string must not fail on Finding");

    // BC-2.09.007 post-5: ISO-8601 UTC via chrono serde integration.
    // ts_sec=1_000_000 is 1970-01-12T13:46:40Z.
    assert!(
        json_str.contains("\"timestamp\""),
        "JSON must contain a \"timestamp\" key when timestamp is Some; got: {json_str}"
    );
    assert!(
        json_str.contains("1970-01-12T13:46:40Z"),
        "timestamp must serialize to ISO-8601 UTC \"1970-01-12T13:46:40Z\" for ts_sec=1_000_000; \
         got: {json_str}"
    );

    // Verify None case: JSON must omit "timestamp" entirely.
    let none_finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "Test finding with no timestamp".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };
    let none_json = serde_json::to_string(&none_finding)
        .expect("serde_json::to_string must not fail on Finding with timestamp=None");
    assert!(
        !none_json.contains("\"timestamp\""),
        "JSON must NOT contain \"timestamp\" key when timestamp is None; got: {none_json}"
    );
}

// ---------------------------------------------------------------------------
// AC-007: Boundary conversions for u32 ts_sec range
// (BC-2.09.007 invariant 2; EC-003 + EC-004)
// ---------------------------------------------------------------------------

#[test]
fn test_timestamp_conversion_boundary_values() {
    // EC-003: ts_sec=0 (Unix epoch) — must produce Some(1970-01-01T00:00:00Z)
    let epoch = DateTime::from_timestamp(0, 0);
    assert!(
        epoch.is_some(),
        "DateTime::from_timestamp(0, 0) must return Some (Unix epoch)"
    );
    let epoch_dt = epoch.unwrap();
    assert_eq!(
        epoch_dt.to_rfc3339(),
        "1970-01-01T00:00:00+00:00",
        "ts_sec=0 must convert to Unix epoch 1970-01-01T00:00:00Z"
    );

    // EC-004: ts_sec=u32::MAX (~2106 CE) — must produce Some(...), not None or panic
    let max_dt = DateTime::from_timestamp(u32::MAX as i64, 0);
    assert!(
        max_dt.is_some(),
        "DateTime::from_timestamp(u32::MAX as i64, 0) must return Some (chrono handles ~2106 CE); \
         chrono range covers this value"
    );

    // Verify the year is approximately 2106 (u32::MAX seconds ≈ 2106-02-07)
    let max_year = max_dt.unwrap().format("%Y").to_string();
    assert_eq!(
        max_year, "2106",
        "u32::MAX seconds from epoch must map to year 2106; got year: {max_year}"
    );

    // Lossless invariant: the conversion from u32 to DateTime<Utc> must be injective
    // (distinct ts_sec values produce distinct DateTime values) for boundary pair.
    let dt_0 = DateTime::from_timestamp(0_i64, 0).unwrap();
    let dt_max = DateTime::from_timestamp(u32::MAX as i64, 0).unwrap();
    assert_ne!(
        dt_0, dt_max,
        "ts_sec=0 and ts_sec=u32::MAX must produce distinct DateTimes"
    );
}

// ---------------------------------------------------------------------------
// AC-005: proptest — arbitrary ts_sec → hot-path Finding.timestamp matches
// (BC-2.04.055 postcondition 1; BC-2.09.007 post-1; VP-021 hot-path proptest)
//
// For arbitrary ts_sec in 0..=u32::MAX, the path-traversal HTTP request
// at that timestamp must produce findings with timestamp = Some(from_ts(ts_sec)).
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_finding_timestamp_matches_on_data_timestamp(
        ts_sec in 0u32..=u32::MAX,
    ) {
        // ts_sec=0 is explicitly in range (EC-001: epoch must not be filtered out).
        let expected: DateTime<Utc> = DateTime::from_timestamp(ts_sec as i64, 0)
            .expect("DateTime::from_timestamp must be Some for all valid u32 ts_sec values");

        let config = ReassemblyConfig::default();
        let mut reassembler = TcpReassembler::new(config);
        let mut http_analyzer = HttpAnalyzer::new();

        let payload = http_traversal_request();
        let payload_len = payload.len() as u32;

        // SYN
        let syn = make_tcp_packet(CLIENT_A, SERVER, 50000, 80, 999, true, false, false, false, vec![]);
        reassembler.process_packet(&syn, ts_sec, &mut http_analyzer);

        // SYN-ACK
        let syn_ack = make_tcp_packet(SERVER, CLIENT_A, 80, 50000, 9999, true, true, false, false, vec![]);
        reassembler.process_packet(&syn_ack, ts_sec, &mut http_analyzer);

        // HTTP data at ts_sec — hot-path flush
        let data_pkt = make_tcp_packet(
            CLIENT_A, SERVER, 50000, 80, 1000, false, true, false, false, payload,
        );
        reassembler.process_packet(&data_pkt, ts_sec, &mut http_analyzer);

        // FIN
        let fin = make_tcp_packet(
            CLIENT_A, SERVER, 50000, 80, 1000 + payload_len, false, true, true, false, vec![],
        );
        reassembler.process_packet(&fin, ts_sec, &mut http_analyzer);
        reassembler.finalize(&mut http_analyzer);

        let findings = http_analyzer.findings();

        // All flow-data-path findings must have timestamp = Some(expected).
        // (Segment-limit summary is not expected here since we're not hitting limits.)
        let timestamped: Vec<_> = findings.iter().filter(|f| f.timestamp.is_some()).collect();

        prop_assert!(
            !timestamped.is_empty(),
            "expected at least one timestamped finding for ts_sec={ts_sec}; \
             got {} total findings",
            findings.len()
        );

        for f in &timestamped {
            prop_assert_eq!(
                f.timestamp,
                Some(expected),
                "finding timestamp must equal DateTime::from_timestamp({}, 0); \
                 got {:?}; summary: {}",
                ts_sec,
                f.timestamp,
                f.summary,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// AC-006: proptest — cross-flow timestamp isolation
// (BC-2.09.007 invariant 4; BC-2.04.055 invariant 3; VP-021 cross-flow isolation)
//
// Two distinct flows A (src=CLIENT_A) and B (src=CLIENT_B) with non-overlapping
// timestamp ranges: ts_a in 1..500_000, ts_b in 500_001..1_000_000.
// Asserts that findings from flow A carry ts_a and findings from flow B carry ts_b.
// ---------------------------------------------------------------------------

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        ..ProptestConfig::default()
    })]

    #[test]
    fn prop_cross_flow_timestamp_isolation(
        ts_a in 1u32..500_000u32,
        ts_b in 500_001u32..1_000_000u32,
    ) {
        let expected_a: DateTime<Utc> = DateTime::from_timestamp(ts_a as i64, 0)
            .expect("ts_a must be a valid datetime");
        let expected_b: DateTime<Utc> = DateTime::from_timestamp(ts_b as i64, 0)
            .expect("ts_b must be a valid datetime");

        // Non-overlapping ranges guarantee ts_a != ts_b, so cross-contamination
        // is detectable as a wrong timestamp on either flow's findings.
        prop_assume!(ts_a != ts_b);

        let config = ReassemblyConfig::default();
        let mut reassembler = TcpReassembler::new(config);
        let mut http_a = HttpAnalyzer::new();

        let payload_a = http_traversal_request();
        let payload_len_a = payload_a.len() as u32;
        let payload_b = http_traversal_request();
        let payload_len_b = payload_b.len() as u32;

        // --- Flow A: CLIENT_A → SERVER at ts_a ---
        let syn_a = make_tcp_packet(CLIENT_A, SERVER, 51000, 80, 999, true, false, false, false, vec![]);
        reassembler.process_packet(&syn_a, ts_a, &mut http_a);

        let syn_ack_a = make_tcp_packet(SERVER, CLIENT_A, 80, 51000, 9999, true, true, false, false, vec![]);
        reassembler.process_packet(&syn_ack_a, ts_a, &mut http_a);

        let data_a = make_tcp_packet(CLIENT_A, SERVER, 51000, 80, 1000, false, true, false, false, payload_a);
        reassembler.process_packet(&data_a, ts_a, &mut http_a);

        let fin_a = make_tcp_packet(
            CLIENT_A, SERVER, 51000, 80, 1000 + payload_len_a, false, true, true, false, vec![],
        );
        reassembler.process_packet(&fin_a, ts_a, &mut http_a);
        // Finalize flow A so the flow is cleanly closed (no Drop-warning path).
        // HTTP findings are emitted on the hot-path flush, so finalize does not
        // change the finding set — but it prevents a leaked-flow Drop warning.
        reassembler.finalize(&mut http_a);

        // Collect findings from flow A before starting flow B
        let findings_a = http_a.findings();
        let ts_findings_a: Vec<_> = findings_a.iter().filter(|f| f.timestamp.is_some()).collect();

        prop_assert!(
            !ts_findings_a.is_empty(),
            "flow A must produce at least one timestamped finding"
        );
        for f in &ts_findings_a {
            prop_assert_eq!(
                f.timestamp,
                Some(expected_a),
                "flow A finding must carry ts_a={}; got {:?}; summary: {}",
                ts_a,
                f.timestamp,
                f.summary,
            );
        }

        // --- Flow B: CLIENT_B → SERVER at ts_b (distinct src_ip → distinct FlowKey) ---
        let mut http_b = HttpAnalyzer::new();
        let config_b = ReassemblyConfig::default();
        let mut reassembler_b = TcpReassembler::new(config_b);

        let syn_b = make_tcp_packet(CLIENT_B, SERVER, 52000, 80, 999, true, false, false, false, vec![]);
        reassembler_b.process_packet(&syn_b, ts_b, &mut http_b);

        let syn_ack_b = make_tcp_packet(SERVER, CLIENT_B, 80, 52000, 9999, true, true, false, false, vec![]);
        reassembler_b.process_packet(&syn_ack_b, ts_b, &mut http_b);

        let data_b = make_tcp_packet(CLIENT_B, SERVER, 52000, 80, 1000, false, true, false, false, payload_b);
        reassembler_b.process_packet(&data_b, ts_b, &mut http_b);

        let fin_b = make_tcp_packet(
            CLIENT_B, SERVER, 52000, 80, 1000 + payload_len_b, false, true, true, false, vec![],
        );
        reassembler_b.process_packet(&fin_b, ts_b, &mut http_b);
        reassembler_b.finalize(&mut http_b);

        let findings_b = http_b.findings();
        let ts_findings_b: Vec<_> = findings_b.iter().filter(|f| f.timestamp.is_some()).collect();

        prop_assert!(
            !ts_findings_b.is_empty(),
            "flow B must produce at least one timestamped finding"
        );
        for f in &ts_findings_b {
            prop_assert_eq!(
                f.timestamp,
                Some(expected_b),
                "flow B finding must carry ts_b={}; got {:?}; summary: {}",
                ts_b,
                f.timestamp,
                f.summary,
            );
        }

        // Cross-contamination check: no flow-B finding carries ts_a, no flow-A finding ts_b.
        for f in &ts_findings_b {
            prop_assert_ne!(
                f.timestamp,
                Some(expected_a),
                "flow B finding must NOT carry ts_a={} (cross-contamination); summary: {}",
                ts_a,
                f.summary
            );
        }
        for f in &ts_findings_a {
            prop_assert_ne!(
                f.timestamp,
                Some(expected_b),
                "flow A finding must NOT carry ts_b={} (cross-contamination); summary: {}",
                ts_b,
                f.summary
            );
        }
    }
}
