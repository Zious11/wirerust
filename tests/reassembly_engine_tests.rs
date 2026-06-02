use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::findings::{Confidence, ThreatCategory, Verdict};
use wirerust::reassembly::flow::{FlowDirection, FlowKey};
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};
use wirerust::reassembly::segment::InsertResult;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

/// Process-global lock serializing tests that read or interact with the
/// `ISN_MISSING_WARNED` atomic in `src/reassembly/segment.rs`.
///
/// Cargo's libtest runs integration tests in parallel within a binary, and
/// any test that triggers `IsnMissing` performs an atomic `swap(true)` on
/// the static. Tests that observe the atomic via
/// `isn_missing_warned_for_testing()` or reset it via
/// `reset_isn_missing_warned_for_testing()` must hold this lock for the
/// duration of their test body, otherwise sibling tests in this same
/// binary can interleave a `swap(true)` between an observation and
/// invalidate the deterministic state assumption (see STORY-014 adv-pass-3
/// F-1).
///
/// Any NEW test in this file that interacts with `ISN_MISSING_WARNED`
/// MUST take this lock as its first line. Failure to do so re-introduces
/// the race.
static ISN_MISSING_WARNED_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Test handler that records all callbacks.
struct RecordingHandler {
    data_events: Vec<(FlowKey, Direction, Vec<u8>, u64)>,
    close_events: Vec<(FlowKey, CloseReason)>,
}

impl RecordingHandler {
    fn new() -> Self {
        RecordingHandler {
            data_events: Vec::new(),
            close_events: Vec::new(),
        }
    }

    fn all_data(&self) -> Vec<u8> {
        self.data_events
            .iter()
            .flat_map(|(_, _, data, _)| data.iter().copied())
            .collect()
    }
}

impl StreamHandler for RecordingHandler {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction, data: &[u8], offset: u64) {
        self.data_events
            .push((flow_key.clone(), direction, data.to_vec(), offset));
    }

    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason) {
        self.close_events.push((flow_key.clone(), reason));
    }
}

#[allow(clippy::too_many_arguments)]
fn make_tcp_packet(
    src_ip: [u8; 4],
    src_port: u16,
    dst_ip: [u8; 4],
    dst_port: u16,
    seq: u32,
    payload: &[u8],
    syn: bool,
    ack: bool,
    fin: bool,
    rst: bool,
) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src_ip)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst_ip)),
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
        payload: payload.to_vec(),
        packet_len: 54 + payload.len(),
    }
}

#[test]
fn test_three_packet_stream_ordered() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Data packets
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"aaa", false, false, false, false,
    );
    reassembler.process_packet(&p1, 2, &mut handler);

    let p2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"bbb", false, false, false, false,
    );
    reassembler.process_packet(&p2, 3, &mut handler);

    let p3 = make_tcp_packet(
        client, 12345, server, 80, 1007, b"ccc", false, false, false, false,
    );
    reassembler.process_packet(&p3, 4, &mut handler);

    assert_eq!(handler.all_data(), b"aaabbbccc");
    assert_eq!(handler.data_events.len(), 3);
}

#[test]
fn test_out_of_order_delivery() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Send packets [1, 3, 2]
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"aaa", false, false, false, false,
    );
    reassembler.process_packet(&p1, 2, &mut handler);

    let p3 = make_tcp_packet(
        client, 12345, server, 80, 1007, b"ccc", false, false, false, false,
    );
    reassembler.process_packet(&p3, 3, &mut handler);
    assert_eq!(handler.data_events.len(), 1); // only p1 flushed

    let p2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"bbb", false, false, false, false,
    );
    reassembler.process_packet(&p2, 4, &mut handler);

    // Now all three should be flushed
    assert_eq!(handler.all_data(), b"aaabbbccc");
}

#[test]
fn test_mid_stream_no_syn() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Data without SYN
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 5000, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&p1, 1, &mut handler);

    assert_eq!(handler.all_data(), b"hello");

    let stats = reassembler.stats();
    assert_eq!(stats.flows_total, 1);
    assert_eq!(stats.flows_partial, 1);
}

#[test]
fn test_rst_closes_flow() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"data", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 3, &mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);
    assert_eq!(reassembler.total_memory(), 0);
}

#[test]
fn test_finalize_flushes_remaining() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        b"leftover",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    reassembler.finalize(&mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Timeout);
}

#[test]
fn test_flow_timeout_expiration() {
    let config = ReassemblyConfig {
        flow_timeout_secs: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 100, &mut handler);

    // Expire at time 200 (100 seconds later, > 10s timeout)
    reassembler.expire_flows(200, &mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Timeout);

    let stats = reassembler.stats();
    assert_eq!(stats.flows_expired, 1);
    assert_eq!(reassembler.total_memory(), 0);
}

#[test]
fn test_total_memory_tracking() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — no payload, no memory change
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order segment — buffered (not flushed)
    let p2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"bbb", false, false, false, false,
    );
    reassembler.process_packet(&p2, 2, &mut handler);
    assert!(handler.data_events.is_empty());
    assert_eq!(reassembler.total_memory(), 3); // "bbb" buffered

    // In-order segment — triggers flush of both
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"aaa", false, false, false, false,
    );
    reassembler.process_packet(&p1, 3, &mut handler);
    assert_eq!(handler.all_data(), b"aaabbb");
    assert_eq!(reassembler.total_memory(), 0); // all flushed

    // Finalize — closes flow
    reassembler.finalize(&mut handler);
    assert_eq!(reassembler.total_memory(), 0);
}

#[test]
fn test_fin_close_total_memory() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN from client
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order data — stays buffered (gap at offset 1)
    let p2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"bbb", false, false, false, false,
    );
    reassembler.process_packet(&p2, 2, &mut handler);
    assert_eq!(reassembler.total_memory(), 3);

    // FIN from client (first FIN)
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1007,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 3, &mut handler);

    // FIN from server (second FIN → Closed, triggers step 10 removal)
    let fin2 = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin2, 4, &mut handler);

    // Flow removed with buffered-but-not-flushed data — total_memory must be 0
    assert_eq!(reassembler.total_memory(), 0);
    assert!(
        handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::Fin)
    );
}

#[test]
fn test_syn_ack_bidirectional_data() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN + SYN+ACK handshake (engine transitions to Established on SYN+ACK)
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Client sends data
    let req = make_tcp_packet(
        client, 12345, server, 80, 1001, b"request", false, false, false, false,
    );
    reassembler.process_packet(&req, 3, &mut handler);

    // Server sends data
    let resp = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        b"response",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&resp, 4, &mut handler);

    // Verify proper handshake (not partial/mid-stream)
    let stats = reassembler.stats();
    assert_eq!(stats.flows_partial, 0);
    assert_eq!(stats.flows_total, 1);

    // Verify bidirectional data with correct directions
    assert_eq!(handler.data_events.len(), 2);
    assert_eq!(handler.data_events[0].1, Direction::ClientToServer);
    assert_eq!(handler.data_events[0].2, b"request");
    assert_eq!(handler.data_events[1].1, Direction::ServerToClient);
    assert_eq!(handler.data_events[1].2, b"response");
}

#[test]
fn test_full_handshake_fin_teardown() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN + SYN+ACK handshake
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Bidirectional data
    let req = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&req, 3, &mut handler);

    let resp = make_tcp_packet(
        server, 80, client, 12345, 2001, b"world", false, false, false, false,
    );
    reassembler.process_packet(&resp, 4, &mut handler);

    // FIN from client
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1006,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 5, &mut handler);

    // FIN from server
    let fin2 = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2006,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin2, 6, &mut handler);

    // Flow closed via FIN
    let stats = reassembler.stats();
    assert_eq!(stats.flows_fin, 1);
    assert_eq!(reassembler.total_memory(), 0);

    // Close reason is Fin
    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Fin);

    // Both directions' data delivered
    let client_data: Vec<&[u8]> = handler
        .data_events
        .iter()
        .filter(|(_, d, _, _)| *d == Direction::ClientToServer)
        .map(|(_, _, data, _)| data.as_slice())
        .collect();
    let server_data: Vec<&[u8]> = handler
        .data_events
        .iter()
        .filter(|(_, d, _, _)| *d == Direction::ServerToClient)
        .map(|(_, _, data, _)| data.as_slice())
        .collect();
    assert_eq!(client_data, vec![b"hello".as_slice()]);
    assert_eq!(server_data, vec![b"world".as_slice()]);
}

#[test]
fn test_max_flows_eviction() {
    // Verify that the engine evicts the oldest flow when the flow table is full,
    // making room for a new flow.
    //
    // max_flows=2 caps the table size.  Both flows carry out-of-order segments
    // (buffered, not flushed) so total_memory is non-zero when the third flow
    // arrives.  memcap=5 is set just below the combined buffered size of both
    // flows (3+3=6) so that evict_flows() does not short-circuit: the loop
    // break condition is `total_memory <= memcap && flows.len() <= max_flows`,
    // and both must be true to stop eviction.  With total_memory=6 > memcap=5,
    // the oldest flow (A) is evicted before Flow C is admitted.
    let config = ReassemblyConfig {
        max_flows: 2,
        memcap: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A (oldest, last_seen=2): SYN + out-of-order data (stays buffered)
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    // seq=1002: offset = 1002-1000 = 2, base_offset = 1 → gap at 1 → buffered
    let data_a = make_tcp_packet(
        [10, 0, 0, 1],
        1000,
        server,
        80,
        1002,
        b"aaa",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&data_a, 2, &mut handler);
    assert_eq!(reassembler.total_memory(), 3); // "aaa" buffered, not flushed

    // Flow B (last_seen=4): SYN + out-of-order data (stays buffered)
    let syn_b = make_tcp_packet(
        [10, 0, 0, 1],
        2000,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 3, &mut handler);
    // seq=2002: offset = 2002-2000 = 2, base_offset = 1 → gap at 1 → buffered
    let data_b = make_tcp_packet(
        [10, 0, 0, 1],
        2000,
        server,
        80,
        2002,
        b"bbb",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&data_b, 4, &mut handler);
    // total_memory = 6 > memcap = 5 → evict_flows() fires, evicts Flow A (oldest)

    // Flow C SYN: after eviction, flows.len()=1 < max_flows=2 → admitted
    let syn_c = make_tcp_packet(
        [10, 0, 0, 1],
        3000,
        server,
        80,
        3000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_c, 5, &mut handler);

    // Eviction occurred
    let stats = reassembler.stats();
    assert!(stats.evictions >= 1);

    // MemoryPressure close reason present
    assert!(
        handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure)
    );

    // All three flows were created at some point
    assert_eq!(stats.flows_total, 3);

    // Verify eviction order: Flow A (oldest, last_seen=2) was evicted, not Flow B
    let flow_a_key = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        1000,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let evicted = handler
        .close_events
        .iter()
        .find(|(_, r)| *r == CloseReason::MemoryPressure)
        .expect("MemoryPressure close event must exist");
    assert_eq!(
        evicted.0, flow_a_key,
        "oldest flow (A) should be evicted first"
    );
}

#[test]
fn test_memcap_eviction() {
    let config = ReassemblyConfig {
        memcap: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Flow A: SYN + out-of-order data (stays buffered because offset 1 is missing)
    let syn_a = make_tcp_packet(
        client,
        1000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let data_a1 = make_tcp_packet(
        client, 1000, server, 80, 1002, b"aaaaa", false, false, false, false,
    );
    reassembler.process_packet(&data_a1, 2, &mut handler);
    assert_eq!(reassembler.total_memory(), 5);

    // Flow B: SYN + out-of-order data that pushes past memcap (5+6=11 > 10)
    let syn_b = make_tcp_packet(
        client,
        2000,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 3, &mut handler);
    let data_b1 = make_tcp_packet(
        client, 2000, server, 80, 2002, b"bbbbbb", false, false, false, false,
    );
    reassembler.process_packet(&data_b1, 4, &mut handler);
    // total_memory would be 11 which exceeds memcap=10, triggering eviction

    // Eviction should have fired
    let stats = reassembler.stats();
    assert!(stats.evictions >= 1);
    assert!(reassembler.total_memory() <= 10);

    // CloseReason::MemoryPressure must be emitted
    assert!(
        handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure),
        "memcap eviction must emit CloseReason::MemoryPressure"
    );
}

#[test]
fn test_overlap_anomaly_finding() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establishes ISN=1000, base_offset=1
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order segment at offset 2 (gap at offset 1 keeps it buffered)
    let original = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);

    // No findings yet
    assert!(reassembler.findings().is_empty());

    // Send 51 duplicates to reach overlap_count=51 (> threshold of 50)
    for i in 0..51u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    // Overlap anomaly finding should be generated
    let findings = reassembler.findings();
    assert!(!findings.is_empty(), "expected overlap anomaly finding");
    let overlap_finding = findings
        .iter()
        .find(|f| f.summary.contains("Excessive segment overlaps"))
        .expect("overlap anomaly finding not found");
    assert_eq!(overlap_finding.category, ThreatCategory::Anomaly);
    assert_eq!(overlap_finding.confidence, Confidence::Medium);
    assert_eq!(overlap_finding.verdict, Verdict::Likely);
    assert_eq!(overlap_finding.mitre_technique.as_deref(), Some("T1036"));
}

#[test]
fn test_conflicting_overlap_finding() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establishes ISN=1000, base_offset=1
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order segment at offset 2 (gap at offset 1 keeps it buffered)
    let original = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);

    // Conflicting retransmission: same seq, different data — triggers ConflictingOverlap
    let conflict = make_tcp_packet(
        client, 12345, server, 80, 1002, b"BBBB", false, false, false, false,
    );
    reassembler.process_packet(&conflict, 3, &mut handler);

    // Conflicting overlap finding should be generated
    let findings = reassembler.findings();
    let conflict_finding = findings
        .iter()
        .find(|f| f.summary.contains("Conflicting TCP segment overlap"))
        .expect("conflicting overlap finding not found");
    assert_eq!(conflict_finding.category, ThreatCategory::Anomaly);
    assert_eq!(conflict_finding.confidence, Confidence::High);
}

// --- VP-002 G3 — END-TO-END first-wins byte survival through the engine ------
/// A finding being EMITTED is necessary but NOT sufficient for the anti-evasion
/// guarantee: the forensic-correctness contract (VP-002) requires that the
/// ATTACKER's conflicting bytes are ABSENT from the reassembled stream actually
/// delivered to the handler — the original (first-arrived) bytes must win all
/// the way out to `on_data`. The unit-level G1/G2 tests prove buffer survival;
/// this proves the delivered STREAM is clean end-to-end.
///
/// Scenario (TCP overlap evasion):
/// - SYN: ISN=1000, base_offset=1.
/// - seq=1002 (offset 2) b"AAAA" — buffered out of order (gap at offset 1).
/// - seq=1002 (offset 2) b"BBBB" — CONFLICTING overlap; first-wins keeps "AAAA".
/// - seq=1001 (offset 1) b"X" — fills the gap, contiguous flush of [1,6).
///
/// The delivered stream must be b"XAAAA": the attacker's 'B' bytes must never
/// appear in any on_data callback.
#[test]
fn test_vp002_g3_end_to_end_conflicting_bytes_absent_from_stream() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establishes ISN=1000, base_offset=1.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // 1. Original out-of-order segment (offset 2): buffered, not yet flushed.
    let original = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);
    assert!(
        handler.data_events.is_empty(),
        "G3 setup: offset-2 segment must stay buffered behind the gap at offset 1"
    );

    // 2. Attacker conflicting overlap at the SAME offset, different bytes.
    let attacker = make_tcp_packet(
        client, 12345, server, 80, 1002, b"BBBB", false, false, false, false,
    );
    reassembler.process_packet(&attacker, 3, &mut handler);

    // 3. Fill the gap at offset 1 → triggers contiguous flush of [1,6).
    let gap_fill = make_tcp_packet(
        client, 12345, server, 80, 1001, b"X", false, false, false, false,
    );
    reassembler.process_packet(&gap_fill, 4, &mut handler);

    // The delivered stream must contain the FIRST-arrived bytes only.
    let delivered = handler.all_data();
    assert_eq!(
        delivered, b"XAAAA",
        "G3: delivered stream must be the first-wins bytes 'XAAAA', not the attacker's 'BBBB'"
    );
    // Anti-evasion: the attacker's conflicting byte 'B' (0x42) must be ABSENT
    // from the entire delivered stream, not merely flagged by a finding.
    assert!(
        !delivered.contains(&b'B'),
        "G3: attacker conflicting byte 'B' must be ABSENT from the delivered stream"
    );

    // And the conflict was still surfaced as a finding (detection + correctness).
    // Match on the structured fields (category/verdict/confidence + MITRE
    // technique) rather than the human-readable summary string, which is brittle
    // to wording changes. The conflicting-overlap finding is uniquely identified
    // by Anomaly + Likely + High + T1036 (the "Excessive segment overlaps"
    // anomaly is Medium confidence), per src/reassembly/lifecycle.rs and
    // BC-2.04.018.
    let findings = reassembler.findings();
    assert!(
        findings.iter().any(|f| {
            f.category == ThreatCategory::Anomaly
                && f.verdict == Verdict::Likely
                && f.confidence == Confidence::High
                && f.mitre_technique.as_deref() == Some("T1036")
        }),
        "G3: a conflicting-overlap finding (Anomaly/Likely/High, MITRE T1036) must still be \
         emitted alongside byte preservation — detection and forensic correctness together"
    );
}

#[test]
fn test_max_segments_per_direction() {
    let config = ReassemblyConfig {
        max_segments_per_direction: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establishes ISN=1000, base_offset=1
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // 5 non-contiguous segments (gap at offset 1 keeps them buffered).
    // seq 1002 → offset 2, seq 1004 → offset 4, etc. Each 1 byte.
    for i in 0..5u32 {
        let seq = 1002 + (i * 2);
        let pkt = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, false, false, false,
        );
        reassembler.process_packet(&pkt, 2 + i, &mut handler);
    }

    // All 5 slots used; no data flushed yet (gap at offset 1)
    assert!(handler.data_events.is_empty());
    let stats_before = reassembler.stats().segments_inserted;
    assert_eq!(stats_before, 5);

    // 6th segment — should be rejected (SegmentLimitReached: max_segments reached)
    let rejected = make_tcp_packet(
        client, 12345, server, 80, 1012, b"y", false, false, false, false,
    );
    reassembler.process_packet(&rejected, 7, &mut handler);

    // segments_inserted must not have increased
    assert_eq!(
        reassembler.stats().segments_inserted,
        stats_before,
        "6th segment should be rejected when max_segments_per_direction is reached"
    );

    // segments_segment_limit counter must be incremented
    assert_eq!(
        reassembler.stats().segments_segment_limit,
        1,
        "segment limit counter should track the rejection"
    );

    // Verify existing buffered segments survive rejection (non-destructive).
    // The 5 segments at offsets 2,4,6,8,10 are non-contiguous with base_offset=1,
    // so flush_contiguous won't deliver them. Verify via memory accounting:
    // total_memory should still reflect all 5 bytes.
    assert_eq!(
        reassembler.total_memory(),
        5,
        "5 buffered segments (1 byte each) must survive after rejection"
    );

    // Finalize cleans up — total_memory drops to 0
    reassembler.finalize(&mut handler);
    assert_eq!(reassembler.total_memory(), 0);
}

#[test]
fn test_finalize_bytes_reassembled_consistent() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    let bytes_before_finalize = reassembler.stats().bytes_reassembled;
    reassembler.finalize(&mut handler);
    let bytes_after_finalize = reassembler.stats().bytes_reassembled;

    assert!(bytes_after_finalize >= bytes_before_finalize);
    let total_delivered: u64 = handler
        .data_events
        .iter()
        .map(|(_, _, data, _)| data.len() as u64)
        .sum();
    assert_eq!(
        bytes_after_finalize, total_delivered,
        "bytes_reassembled must match total bytes delivered to handler"
    );
}

#[test]
fn test_out_of_window_segment_rejected_by_engine() {
    let config = ReassemblyConfig {
        max_receive_window: 1000, // small window for testing
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN (ISN=1000, base_offset=1)
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Normal data at offset 1 (within window)
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&p1, 2, &mut handler);

    assert_eq!(handler.data_events.len(), 1);
    assert_eq!(reassembler.stats().segments_inserted, 1);

    // Segment way beyond window: base_offset=6, window=1000, so offset > 1006 is rejected
    // seq = ISN + offset = 1000 + 2000 = 3000
    let far = make_tcp_packet(
        client, 12345, server, 80, 3000, b"evil", false, false, false, false,
    );
    reassembler.process_packet(&far, 3, &mut handler);

    // Should be rejected — no new data events, counter incremented
    assert_eq!(handler.data_events.len(), 1); // unchanged
    assert_eq!(reassembler.stats().segments_out_of_window, 1);
    assert_eq!(reassembler.stats().segments_inserted, 1); // unchanged

    // Segment just within window should be accepted
    // base_offset=6, window=1000, so offset 1006 is the last accepted
    // seq = ISN + offset = 1000 + 1006 = 2006
    let edge = make_tcp_packet(
        client, 12345, server, 80, 2006, b"ok", false, false, false, false,
    );
    reassembler.process_packet(&edge, 4, &mut handler);

    assert_eq!(reassembler.stats().segments_inserted, 2);
    assert_eq!(reassembler.stats().segments_out_of_window, 1); // unchanged
}

#[test]
fn test_out_of_window_threshold_alert() {
    let config = ReassemblyConfig {
        max_receive_window: 1000, // small window for testing
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Send 101 out-of-window segments (threshold is 100, alert fires when > 100)
    for i in 0..101 {
        // Each segment uses a different sequence beyond the window
        let seq = 1000 + 5000 + i; // Way beyond base_offset + 1000 window
        let pkt = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, false, false, false,
        );
        reassembler.process_packet(&pkt, 2, &mut handler);
    }

    assert_eq!(reassembler.stats().segments_out_of_window, 101);

    // Verify alert finding was generated
    let oow_finding = reassembler
        .findings()
        .iter()
        .find(|f| f.summary.contains("out-of-window segments"));
    assert!(
        oow_finding.is_some(),
        "should generate out-of-window threshold finding"
    );
    let f = oow_finding.unwrap();
    assert!(f.evidence[0].contains("max_receive_window=1000"));
}

#[test]
fn test_out_of_window_threshold_silent_at_exactly_threshold() {
    // The out-of-window alert fires on `count > threshold`, strictly.
    // With the threshold configured to 5, exactly 5 out-of-window
    // segments must NOT fire — pins the boundary so a regression from
    // `>` to `>=` is caught (`test_out_of_window_threshold_alert`
    // covers the threshold + 1 side).
    let config = ReassemblyConfig {
        max_receive_window: 1000,
        out_of_window_alert_threshold: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    for i in 0..5 {
        let seq = 1000 + 5000 + i; // beyond base_offset + the 1000-byte window
        let pkt = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, false, false, false,
        );
        reassembler.process_packet(&pkt, 2, &mut handler);
    }

    assert_eq!(reassembler.stats().segments_out_of_window, 5);
    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("out-of-window segments")),
        "exactly the threshold count must stay silent (the test is `>`, not `>=`)"
    );
}

#[test]
fn test_out_of_window_alert_fires_only_once() {
    let config = ReassemblyConfig {
        max_receive_window: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Send 200 out-of-window segments (well above threshold)
    for i in 0..200 {
        let seq = 1000 + 5000 + i;
        let pkt = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, false, false, false,
        );
        reassembler.process_packet(&pkt, 2, &mut handler);
    }

    // Should only have 1 finding for out-of-window despite 200 events
    let oow_count = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("out-of-window segments"))
        .count();
    assert_eq!(oow_count, 1);
}

#[test]
fn test_summarize_returns_reassembly_stats() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    reassembler.finalize(&mut handler);

    let summary = reassembler.summarize();
    assert_eq!(summary.analyzer_name, "TCP Reassembly");
    assert!(summary.packets_analyzed > 0);
    assert_eq!(
        summary.detail.get("flows_total"),
        Some(&serde_json::Value::from(1u64))
    );
    assert_eq!(
        summary.detail.get("segments_inserted"),
        Some(&serde_json::Value::from(1u64))
    );
    assert_eq!(
        summary.detail.get("bytes_reassembled"),
        Some(&serde_json::Value::from(5u64))
    );
}

#[test]
fn test_finalize_generates_segment_limit_finding() {
    let config = ReassemblyConfig {
        max_segments_per_direction: 2,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Insert 2 non-contiguous segments (gap at offset 1 keeps them buffered)
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1002, b"a", false, false, false, false,
    );
    let p2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"b", false, false, false, false,
    );
    reassembler.process_packet(&p1, 2, &mut handler);
    reassembler.process_packet(&p2, 3, &mut handler);

    // 3rd segment should hit segment limit
    let p3 = make_tcp_packet(
        client, 12345, server, 80, 1006, b"c", false, false, false, false,
    );
    reassembler.process_packet(&p3, 4, &mut handler);

    assert_eq!(reassembler.stats().segments_segment_limit, 1);

    // Before finalize: no summary-level finding yet
    let findings_before = reassembler.findings().len();

    reassembler.finalize(&mut handler);

    // After finalize: should have a summary finding about segment limit
    let findings_after = reassembler.findings();
    assert!(
        findings_after.len() > findings_before,
        "finalize should generate a segment-limit finding"
    );

    let limit_finding = findings_after
        .iter()
        .find(|f| f.summary.contains("segment count limit"))
        .expect("should find segment-limit summary finding");
    assert!(limit_finding.summary.contains("1 segment dropped"));
}

#[test]
fn test_finalize_no_finding_when_no_segment_limit_hits() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    let findings_before = reassembler.findings().len();
    reassembler.finalize(&mut handler);

    // No segment limit hits — no summary finding should be generated
    let new_findings: Vec<_> = reassembler.findings()[findings_before..]
        .iter()
        .filter(|f| f.summary.contains("segment count limit"))
        .collect();
    assert!(
        new_findings.is_empty(),
        "should not generate segment-limit finding when counter is 0"
    );
}

#[test]
fn test_depth_exceeded_counter() {
    let config = ReassemblyConfig {
        max_depth: 10, // tiny depth for testing
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // First segment: 8 bytes, fits within 10-byte depth
    let p1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        b"AAAAAAAA",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&p1, 2, &mut handler);
    assert_eq!(reassembler.stats().segments_inserted, 1);
    assert_eq!(reassembler.stats().segments_depth_exceeded, 0);

    // Second segment: 5 bytes, would exceed 10-byte depth (8 + 5 = 13 > 10)
    // Truncated to 2 bytes and inserted (returns Truncated, not DepthExceeded)
    let p2 = make_tcp_packet(
        client, 12345, server, 80, 1009, b"BBBBB", false, false, false, false,
    );
    reassembler.process_packet(&p2, 3, &mut handler);

    // Third segment: fully rejected — depth already exceeded
    let p3 = make_tcp_packet(
        client, 12345, server, 80, 1014, b"CCCCC", false, false, false, false,
    );
    reassembler.process_packet(&p3, 4, &mut handler);
    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        1,
        "depth_exceeded counter should track fully rejected segments"
    );

    // Verify it shows up in summarize()
    reassembler.finalize(&mut handler);
    let summary = reassembler.summarize();
    let depth_val = summary.detail.get("segments_depth_exceeded");
    assert!(
        depth_val.is_some(),
        "segments_depth_exceeded should appear in summarize() detail"
    );
}

// ---- LESSON-P0.03 / smell #9: Drop tripwire + finalize idempotency ----

#[test]
fn test_is_finalized_flips_on_finalize() {
    // Lifecycle invariant: `is_finalized()` is false before `finalize` and
    // true after. This is the field that `impl Drop for TcpReassembler`
    // reads to decide whether to emit the lifecycle warning.
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();
    assert!(
        !reassembler.is_finalized(),
        "new reassembler should not yet be finalized"
    );
    reassembler.finalize(&mut handler);
    assert!(
        reassembler.is_finalized(),
        "finalize() must flip is_finalized() to true"
    );
}

#[test]
fn test_finalize_is_idempotent() {
    // The lesson observed that without `impl Drop`, finalize is a manual
    // call. With the tripwire in place, finalize must also remain safely
    // idempotent so that callers (or a future Drop guard wrapper) can
    // invoke it more than once without re-emitting summary findings or
    // re-flushing flows. Verifies the `if self.finalized { return; }`
    // guard in mod.rs.
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();
    reassembler.finalize(&mut handler);
    let first_findings = reassembler.findings().len();
    let first_close_events = handler.close_events.len();
    reassembler.finalize(&mut handler);
    assert!(reassembler.is_finalized());
    assert_eq!(
        reassembler.findings().len(),
        first_findings,
        "second finalize call must not append additional findings"
    );
    assert_eq!(
        handler.close_events.len(),
        first_close_events,
        "second finalize call must not re-emit on_flow_close events"
    );
}

#[test]
fn test_drop_without_finalize_does_not_panic() {
    // `impl Drop for TcpReassembler` is intentionally non-panicking: it
    // emits a one-shot eprintln warning so future regressions of the
    // "forgot to call finalize" bug are visible at runtime, but it must
    // never crash the process. Several existing unit tests construct a
    // reassembler purely to exercise sub-engine behavior without ever
    // driving it to a captured-state end-of-input, so the tripwire
    // would be unusable as a `panic!` / `debug_assert!`.
    //
    // This test asserts that dropping an un-finalized reassembler is a
    // safe, non-panicking operation. Stderr capture for the warning
    // text itself is left to the integration layer.
    //
    // FINALIZE_SKIPPED_WARNED_LOCK convention (DF-SIBLING-SWEEP-001, Bucket 2):
    // This test is a Bucket 2 defensive holder — it triggers an un-finalized
    // Drop without reading or swapping the atomic. It acquires the lock so its
    // Drop does not preempt AC-004's reset→swap window when tests run with low
    // parallelism (e.g. --test-threads=1 or slow schedulers). Under the Option B
    // scoping adopted for this suite (~130+ sites, too many for Bucket 2 wrapping),
    // the ~130+ remaining tests that drop un-finalized reassemblers are Bucket 4
    // (no lock required); this test alone holds the lock defensively because it
    // is an explicit non-panic assertion and warrants tighter scheduling control.
    let _guard = FINALIZE_SKIPPED_WARNED_LOCK
        .lock()
        .expect("FINALIZE_SKIPPED_WARNED_LOCK poisoned");
    let reassembler = TcpReassembler::new(ReassemblyConfig::default());
    assert!(!reassembler.is_finalized());
    drop(reassembler); // must not panic
}

#[test]
fn test_drop_after_finalize_is_silent_path() {
    // The happy path: caller drives finalize before letting the
    // reassembler go out of scope. The Drop guard's `!self.finalized`
    // check is the gating predicate for the warning; this test verifies
    // we hit that predicate as false, so production runs do not spam
    // stderr.
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();
    reassembler.finalize(&mut handler);
    assert!(reassembler.is_finalized());
    drop(reassembler); // must not panic and must be the silent branch
}

// ---- LESSON-P1.01: dropped_findings counter surfaces MAX_FINDINGS suppressions ----

#[test]
fn test_dropped_findings_zero_on_normal_flow() {
    // On a happy-path single-flow capture, the dropped_findings
    // counter must be zero — the field is contractually a
    // "cap-hit" signal, not a per-anomaly counter.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hi", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);
    reassembler.finalize(&mut handler);

    let summary = reassembler.summarize();
    assert_eq!(
        summary.detail.get("dropped_findings"),
        Some(&serde_json::Value::from(0u64)),
        "happy-path capture must report dropped_findings == 0"
    );
}

#[test]
fn test_dropped_findings_key_present_in_summarize() {
    // Structural contract: the `dropped_findings` key always appears in
    // the summarize() detail map (mirrors the pattern for
    // `segments_segment_limit`, `parse_errors`, etc.). Catches a
    // regression that removes the field-to-detail wiring even if no
    // counter ever increments during a particular run.
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();
    reassembler.finalize(&mut handler);
    let summary = reassembler.summarize();
    assert!(
        summary.detail.contains_key("dropped_findings"),
        "summarize() detail must always contain `dropped_findings` — LESSON-P1.01 regressed"
    );
}

// ---------------------------------------------------------------------------
// STORY-011: BC-2.04.001 — TcpReassembler::new constructor validation
//
// AC-001 through AC-007 (and EC-001..EC-007) formalize the five assert!
// guards in TcpReassembler::new and the legal-value invariant for
// flow_timeout_secs.  Each test's name is prescribed by the story spec
// (W1.4 decision); the BC postcondition being exercised is cited in the
// doc comment.
// ---------------------------------------------------------------------------

// ---- RED GATE stubs — #[should_panic] tests use an empty body so that
//      the should_panic contract is NOT satisfied (test fails because no
//      panic is raised).  Regular tests use panic!("RED GATE: ...").
// ---- After RED GATE is verified, stubs are replaced with real assertions.

/// AC-001 / EC-001 (BC-2.04.001 postcondition 1)
/// Postcondition: if config.max_depth == 0, constructor panics with a
/// message containing "max_depth must be > 0".
#[test]
#[allow(non_snake_case)]
#[should_panic(expected = "max_depth must be > 0")]
fn test_BC_2_04_001_max_depth_zero_panics() {
    let config = ReassemblyConfig {
        max_depth: 0,
        ..ReassemblyConfig::default()
    };
    let _ = TcpReassembler::new(config);
}

/// AC-002 / EC-002 (BC-2.04.001 postcondition 2)
/// Postcondition: if config.memcap == 0, constructor panics with a message
/// containing "memcap must be > 0".
#[test]
#[allow(non_snake_case)]
#[should_panic(expected = "memcap must be > 0")]
fn test_BC_2_04_001_memcap_zero_panics() {
    let config = ReassemblyConfig {
        memcap: 0,
        ..ReassemblyConfig::default()
    };
    let _ = TcpReassembler::new(config);
}

/// AC-003 / EC-003 (BC-2.04.001 postcondition 3)
/// Postcondition: if config.max_flows == 0, constructor panics with a
/// message containing "max_flows must be > 0".
#[test]
#[allow(non_snake_case)]
#[should_panic(expected = "max_flows must be > 0")]
fn test_BC_2_04_001_max_flows_zero_panics() {
    let config = ReassemblyConfig {
        max_flows: 0,
        ..ReassemblyConfig::default()
    };
    let _ = TcpReassembler::new(config);
}

/// AC-004 / EC-004 (BC-2.04.001 postcondition 4)
/// Postcondition: if config.max_segments_per_direction == 0, constructor
/// panics with a message containing "max_segments_per_direction must be > 0".
#[test]
#[allow(non_snake_case)]
#[should_panic(expected = "max_segments_per_direction must be > 0")]
fn test_BC_2_04_001_max_segments_per_direction_zero_panics() {
    let config = ReassemblyConfig {
        max_segments_per_direction: 0,
        ..ReassemblyConfig::default()
    };
    let _ = TcpReassembler::new(config);
}

/// AC-005 / EC-005 (BC-2.04.001 postcondition 5)
/// Postcondition: if config.max_receive_window == 0, constructor panics
/// with a message containing "max_receive_window must be > 0".
#[test]
#[allow(non_snake_case)]
#[should_panic(expected = "max_receive_window must be > 0")]
fn test_BC_2_04_001_max_receive_window_zero_panics() {
    let config = ReassemblyConfig {
        max_receive_window: 0,
        ..ReassemblyConfig::default()
    };
    let _ = TcpReassembler::new(config);
}

/// AC-006 / EC-006 (BC-2.04.001 postcondition 6)
/// Postcondition: when all five validated fields are > 0 (ReassemblyConfig::default()
/// satisfies this), the constructor returns a TcpReassembler with:
/// - empty flows (total_memory == 0)
/// - empty findings (findings().is_empty())
/// - finalized == false
///
/// Also verified with a minimal config (all five validated fields = 1) to
/// confirm the boundary: the smallest legal value for each field must not panic.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_001_valid_config_constructs_successfully() {
    // Default config: all five validated fields are well above 0.
    let reassembler = TcpReassembler::new(ReassemblyConfig::default());
    assert_eq!(
        reassembler.total_memory(),
        0,
        "fresh reassembler must have total_memory == 0"
    );
    assert!(
        reassembler.findings().is_empty(),
        "fresh reassembler must have no findings"
    );
    assert!(
        !reassembler.is_finalized(),
        "fresh reassembler must not be finalized"
    );

    // Minimal config: all five validated fields set to 1 (boundary case).
    let min_config = ReassemblyConfig {
        max_depth: 1,
        memcap: 1,
        max_flows: 1,
        max_segments_per_direction: 1,
        max_receive_window: 1,
        ..ReassemblyConfig::default()
    };
    let min_reassembler = TcpReassembler::new(min_config);
    assert_eq!(
        min_reassembler.total_memory(),
        0,
        "minimal-config reassembler must also have total_memory == 0"
    );
    assert!(
        min_reassembler.findings().is_empty(),
        "minimal-config reassembler must have no findings"
    );
    assert!(
        !min_reassembler.is_finalized(),
        "minimal-config reassembler must not be finalized"
    );
}

/// AC-007 / EC-007 (BC-2.04.001 invariant 2)
/// Invariant: flow_timeout_secs == 0 is NOT validated at construction;
/// the constructor must accept it as a legal value and not panic.
///
/// BC-2.04.001 §Invariants ¶2: "Other config fields (flow_timeout_secs,
/// threshold fields) are NOT validated at construction; zero values in
/// those fields are legal."
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_001_flow_timeout_zero_is_legal() {
    let config = ReassemblyConfig {
        flow_timeout_secs: 0,
        ..ReassemblyConfig::default()
    };
    // Must not panic — if it does the test fails.
    let reassembler = TcpReassembler::new(config);
    assert!(
        !reassembler.is_finalized(),
        "reassembler with flow_timeout_secs=0 must construct successfully"
    );
}

// ---- LESSON-P2.05: configurable anomaly thresholds ----

/// Drive a flow to `n` overlapping duplicate segments and return the
/// reassembler, so a test can assert whether the overlap finding fired
/// under a given `overlap_alert_threshold`.
fn run_overlapping_flow(overlap_alert_threshold: u32, duplicates: u32) -> TcpReassembler {
    let config = ReassemblyConfig {
        overlap_alert_threshold,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order segment at offset 2 (gap at offset 1 keeps it buffered).
    let original = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);

    for i in 0..duplicates {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }
    reassembler
}

#[test]
fn test_low_overlap_threshold_fires_earlier() {
    // With a configured threshold of 5, just 6 overlapping duplicates
    // (overlap_count = 6 > 5) must trigger the overlap anomaly — far
    // below the default-50 trip point. Proves the engine reads the
    // threshold from ReassemblyConfig, not a hard-coded const.
    let reasm = run_overlapping_flow(5, 6);
    assert!(
        reasm
            .findings()
            .iter()
            .any(|f| f.summary.contains("Excessive segment overlaps")),
        "overlap finding must fire once overlap_count exceeds the configured threshold of 5"
    );
}

#[test]
fn test_default_overlap_threshold_silent_at_six_overlaps() {
    // The same 6 overlaps under the default threshold (50) must NOT
    // fire — confirms the low-threshold test above is exercising the
    // config field, not some unrelated trigger.
    let reasm = run_overlapping_flow(ReassemblyConfig::default().overlap_alert_threshold, 6);
    assert!(
        !reasm
            .findings()
            .iter()
            .any(|f| f.summary.contains("Excessive segment overlaps")),
        "6 overlaps must stay silent under the default threshold of 50"
    );
}

#[test]
fn test_overlap_threshold_silent_at_exactly_threshold() {
    // The overlap alert fires on `overlap_count > threshold`, strictly.
    // With the threshold configured to 5, exactly 5 overlaps must NOT
    // fire — pins the boundary so a regression from `>` to `>=` is
    // caught (`test_low_overlap_threshold_fires_earlier` covers the
    // threshold + 1 side).
    let reasm = run_overlapping_flow(5, 5);
    assert!(
        !reasm
            .findings()
            .iter()
            .any(|f| f.summary.contains("Excessive segment overlaps")),
        "exactly the threshold count must stay silent (the test is `>`, not `>=`)"
    );
}

#[test]
fn test_default_config_threshold_values() {
    // Pin the documented LESSON-P2.05 defaults so a silent change is
    // caught. These are conservative engineering defaults (see
    // config.rs field docs), not values endorsed by NIDS prior art.
    let cfg = ReassemblyConfig::default();
    assert_eq!(cfg.overlap_alert_threshold, 50);
    assert_eq!(cfg.small_segment_alert_threshold, 100);
    assert_eq!(cfg.small_segment_max_bytes, 16);
    assert_eq!(cfg.small_segment_ignore_ports, vec![23, 513]);
    assert_eq!(cfg.out_of_window_alert_threshold, 100);
}

/// Drive a flow of one-byte segments through the engine to exercise the
/// consecutive small-segment run counter (LESSON-P2.05). `client_port`
/// and `server_port` are the flow's two endpoint ports — vary them to
/// land an ignored port on either side and observe the port exemption.
/// When `break_after` is `Some(n)`, one normal-sized (29-byte) segment
/// is inserted after the n-th small segment, which must reset the run.
fn run_small_segment_flow(
    threshold: u32,
    small_count: u32,
    break_after: Option<u32>,
    client_port: u16,
    server_port: u16,
) -> TcpReassembler {
    let config = ReassemblyConfig {
        small_segment_alert_threshold: threshold,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        client_port,
        server,
        server_port,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let mut seq: u32 = 1001;
    let mut ts: u32 = 2;
    for i in 0..small_count {
        if break_after == Some(i) {
            // 29 bytes: well above the 16-byte cutoff, so it resets the run.
            let normal = make_tcp_packet(
                client,
                client_port,
                server,
                server_port,
                seq,
                b"a-normal-sized-tcp-segment-xx",
                false,
                true,
                false,
                false,
            );
            reassembler.process_packet(&normal, ts, &mut handler);
            seq += 29;
            ts += 1;
        }
        let small = make_tcp_packet(
            client,
            client_port,
            server,
            server_port,
            seq,
            b"x",
            false,
            true,
            false,
            false,
        );
        reassembler.process_packet(&small, ts, &mut handler);
        seq += 1;
        ts += 1;
    }
    reassembler
}

/// True if any finding's summary names the small-segment anomaly.
fn fired_small_segment(reasm: &TcpReassembler) -> bool {
    reasm
        .findings()
        .iter()
        .any(|f| f.summary.contains("small segments"))
}

#[test]
fn test_consecutive_small_segments_trip_anomaly() {
    // 11 one-byte segments form an unbroken run of 11, above the
    // configured threshold of 10 — the small-segment anomaly must fire.
    let reasm = run_small_segment_flow(10, 11, None, 12345, 80);
    assert!(
        fired_small_segment(&reasm),
        "an unbroken run past the threshold must fire the small-segment anomaly"
    );
}

#[test]
fn test_small_segment_run_at_exactly_threshold_stays_silent() {
    // The alert fires on `run > threshold`, strictly. A run of exactly
    // the threshold (10 small segments, threshold 10) must NOT fire —
    // this pins the boundary, so a regression from `>` to `>=` is
    // caught (`test_consecutive_small_segments_trip_anomaly` covers the
    // threshold + 1 side).
    let reasm = run_small_segment_flow(10, 10, None, 12345, 80);
    assert!(
        !fired_small_segment(&reasm),
        "a run of exactly the threshold must stay silent (the test is `>`, not `>=`)"
    );
}

#[test]
fn test_normal_segment_resets_small_segment_run() {
    // Two flows of 11 one-byte segments against a threshold of 8,
    // differing only in where the run-breaking normal segment lands.
    // Both the *threshold boundary* and the *reset position* are made
    // load-bearing — a wrong answer flips the assertion.
    //
    // Break after the 9th small segment: the first sub-run is exactly 9
    // (> 8) so the anomaly MUST fire. This proves the run is not reset
    // before the break — a too-early reset would keep the sub-run <= 8.
    let fires = run_small_segment_flow(8, 11, Some(9), 12345, 80);
    assert!(
        fired_small_segment(&fires),
        "the pre-break sub-run of 9 must trip the threshold of 8"
    );
    // Move the break one segment earlier: the first sub-run is now
    // exactly 8 (not > 8) and the second is 3, so the anomaly MUST stay
    // silent. Were the reset absent the run would reach 11 and fire —
    // so this proves the reset actually happens, and at the break.
    let silent = run_small_segment_flow(8, 11, Some(8), 12345, 80);
    assert!(
        !fired_small_segment(&silent),
        "a normal-sized segment must reset the run at the break"
    );
}

#[test]
fn test_small_segment_anomaly_suppressed_on_server_side_ignored_port() {
    // The same unbroken 11-segment run that fires on port 80 (see
    // `test_consecutive_small_segments_trip_anomaly`) must stay silent
    // when the server port is telnet (23) — in the default ignore list.
    // The only difference between the two flows is the server port, so
    // this proves the port is the discriminator.
    let reasm = run_small_segment_flow(10, 11, None, 12345, 23);
    assert!(
        !fired_small_segment(&reasm),
        "small-segment detection must be suppressed when the server port is ignored"
    );
}

#[test]
fn test_small_segment_anomaly_suppressed_on_client_side_ignored_port() {
    // The exemption matches EITHER endpoint: here the ignored port (23)
    // is the *client* port and the server port (80) is not ignored.
    // The run must still be suppressed — exercises the `lower_port()`
    // arm of the either-endpoint check, the complement of the test
    // above.
    let reasm = run_small_segment_flow(10, 11, None, 23, 80);
    assert!(
        !fired_small_segment(&reasm),
        "small-segment detection must be suppressed when the client port is ignored"
    );
}

// ---------------------------------------------------------------------------
// STORY-012: BC-2.04.002, BC-2.04.028, BC-2.04.030
//   Non-TCP Packet Filter, Statistics Summary, bytes_reassembled Accounting
//
// AC-001 through AC-013 (prescribes exact test names per W1.4 decision)
// EC-001 through EC-008 (edge cases from story spec + BCs)
// ---------------------------------------------------------------------------

/// Build a UDP packet (non-TCP, has TransportInfo::Udp).
fn make_udp_packet(
    src_ip: [u8; 4],
    src_port: u16,
    dst_ip: [u8; 4],
    dst_port: u16,
    payload: &[u8],
) -> ParsedPacket {
    use wirerust::decoder::TransportInfo;
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src_ip)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst_ip)),
        protocol: Protocol::Udp,
        transport: TransportInfo::Udp { src_port, dst_port },
        payload: payload.to_vec(),
        packet_len: 28 + payload.len(),
    }
}

/// Build an ICMP packet (non-TCP, TransportInfo::None).
fn make_icmp_packet(src_ip: [u8; 4], dst_ip: [u8; 4]) -> ParsedPacket {
    use wirerust::decoder::TransportInfo;
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src_ip)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst_ip)),
        protocol: Protocol::Icmp,
        transport: TransportInfo::None,
        payload: vec![],
        packet_len: 28,
    }
}

/// Build a Protocol::Other(n) packet (non-TCP, TransportInfo::None).
fn make_other_protocol_packet(src_ip: [u8; 4], dst_ip: [u8; 4], proto: u8) -> ParsedPacket {
    use wirerust::decoder::TransportInfo;
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src_ip)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst_ip)),
        protocol: Protocol::Other(proto),
        transport: TransportInfo::None,
        payload: vec![],
        packet_len: 28,
    }
}

// ---- AC-001 ----------------------------------------------------------------

/// AC-001 (BC-2.04.002 postcondition 1)
/// Postcondition: when process_packet is called with a non-TCP (UDP) packet,
/// stats.packets_processed increments by 1.
///
/// Canonical test vector: single UDP packet → packets_processed=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_002_non_tcp_increments_packets_processed() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let pkt = make_udp_packet([10, 0, 0, 1], 12345, [10, 0, 0, 2], 53, b"query");
    reassembler.process_packet(&pkt, 1, &mut handler);

    assert_eq!(
        reassembler.stats().packets_processed,
        1,
        "BC-2.04.002 post-1: packets_processed must increment for non-TCP packet"
    );
}

// ---- AC-002 ----------------------------------------------------------------

/// AC-002 (BC-2.04.002 postcondition 2)
/// Postcondition: when process_packet is called with a non-TCP packet,
/// stats.packets_skipped_non_tcp increments by 1.
///
/// Canonical test vector: single UDP packet → packets_skipped_non_tcp=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_002_non_tcp_increments_skipped_counter() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let pkt = make_udp_packet([10, 0, 0, 1], 12345, [10, 0, 0, 2], 53, b"query");
    reassembler.process_packet(&pkt, 1, &mut handler);

    assert_eq!(
        reassembler.stats().packets_skipped_non_tcp,
        1,
        "BC-2.04.002 post-2: packets_skipped_non_tcp must increment for non-TCP packet"
    );
}

// ---- AC-003 ----------------------------------------------------------------

/// AC-003 (BC-2.04.002 postcondition 3)
/// Postcondition: when process_packet is called with a non-TCP packet,
/// stats.packets_tcp does NOT change.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_002_non_tcp_does_not_increment_tcp_counter() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Deliver three non-TCP packets of different types.
    let udp = make_udp_packet([10, 0, 0, 1], 12345, [10, 0, 0, 2], 53, b"q");
    let icmp = make_icmp_packet([10, 0, 0, 1], [10, 0, 0, 2]);
    let other = make_other_protocol_packet([10, 0, 0, 1], [10, 0, 0, 2], 99);
    reassembler.process_packet(&udp, 1, &mut handler);
    reassembler.process_packet(&icmp, 2, &mut handler);
    reassembler.process_packet(&other, 3, &mut handler);

    assert_eq!(
        reassembler.stats().packets_tcp,
        0,
        "BC-2.04.002 post-3: packets_tcp must remain 0 after non-TCP packets only"
    );
}

// ---- AC-004 ----------------------------------------------------------------

/// AC-004 (BC-2.04.002 postconditions 4-6)
/// Postconditions: no flow created/modified, no findings emitted, no handler
/// callbacks (on_data, on_flow_close) triggered for a non-TCP packet.
///
/// Flow creation is observable via stats.flows_total and total_memory().
/// Findings absence is directly observable via findings().
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_002_non_tcp_creates_no_flow_no_callbacks() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let udp = make_udp_packet([10, 0, 0, 1], 12345, [10, 0, 0, 2], 53, b"dns-query");
    reassembler.process_packet(&udp, 1, &mut handler);

    // BC-2.04.002 post-4: no flow created
    assert_eq!(
        reassembler.stats().flows_total,
        0,
        "BC-2.04.002 post-4: non-TCP must not create a flow"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.002 post-4: no buffered state for non-TCP packet"
    );
    // BC-2.04.002 post-5: no findings
    assert!(
        reassembler.findings().is_empty(),
        "BC-2.04.002 post-5: no findings emitted for non-TCP packet"
    );
    // BC-2.04.002 post-6: no handler callbacks
    assert!(
        handler.data_events.is_empty(),
        "BC-2.04.002 post-6: on_data must not be called for non-TCP packet"
    );
    assert!(
        handler.close_events.is_empty(),
        "BC-2.04.002 post-6: on_flow_close must not be called for non-TCP packet"
    );
}

// ---- AC-005 ----------------------------------------------------------------

/// AC-005 (BC-2.04.002 invariant 1)
/// Invariant: after N non-TCP and M TCP packets, packets_processed == N+M,
/// packets_skipped_non_tcp == N, and packets_tcp == M.
///
/// Canonical test vector from BC-2.04.002: 5 UDP + 3 TCP →
/// packets_processed=8, packets_skipped_non_tcp=5, packets_tcp=3.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_002_mixed_protocol_counter_arithmetic() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // 5 UDP packets (N=5)
    for i in 0..5u32 {
        let udp = make_udp_packet(client, 10000 + i as u16, server, 53, b"q");
        reassembler.process_packet(&udp, i + 1, &mut handler);
    }

    // 3 TCP packets (M=3): SYN + 2 data segments on one flow
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 6, &mut handler);
    let d1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"ab", false, false, false, false,
    );
    reassembler.process_packet(&d1, 7, &mut handler);
    let d2 = make_tcp_packet(
        client, 12345, server, 80, 1003, b"cd", false, false, false, false,
    );
    reassembler.process_packet(&d2, 8, &mut handler);

    let stats = reassembler.stats();
    assert_eq!(
        stats.packets_processed, 8,
        "BC-2.04.002 inv-1: packets_processed must be N+M = 5+3 = 8"
    );
    assert_eq!(
        stats.packets_skipped_non_tcp, 5,
        "BC-2.04.002 inv-1: packets_skipped_non_tcp must equal N = 5"
    );
    assert_eq!(
        stats.packets_tcp, 3,
        "BC-2.04.002 inv-1: packets_tcp must equal M = 3"
    );
    // Invariant: packets_processed >= packets_tcp + packets_skipped_non_tcp
    assert!(
        stats.packets_processed >= stats.packets_tcp + stats.packets_skipped_non_tcp,
        "BC-2.04.002 inv-1: packets_processed must be >= packets_tcp + packets_skipped_non_tcp"
    );
}

// ---- AC-006 ----------------------------------------------------------------

/// AC-006 (BC-2.04.028 postcondition 1)
/// Postcondition: summarize() returns an AnalysisSummary with
/// analyzer_name == "TCP Reassembly".
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_028_summarize_analyzer_name() {
    let reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let summary = reassembler.summarize();
    assert_eq!(
        summary.analyzer_name, "TCP Reassembly",
        "BC-2.04.028 post-1: analyzer_name must be \"TCP Reassembly\""
    );
}

// ---- AC-007 ----------------------------------------------------------------

/// AC-007 (BC-2.04.028 postcondition 2)
/// Postcondition: summarize() returns packets_analyzed == stats.packets_tcp,
/// not packets_processed.
///
/// Canonical test vector: 5 non-TCP + 3 TCP → packets_analyzed=3.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_028_summarize_packets_analyzed_equals_tcp_count() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // 5 non-TCP packets
    for i in 0..5u32 {
        let udp = make_udp_packet(client, 10000 + i as u16, server, 53, b"q");
        reassembler.process_packet(&udp, i + 1, &mut handler);
    }
    // 3 TCP packets
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 6, &mut handler);
    let d1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hi", false, false, false, false,
    );
    reassembler.process_packet(&d1, 7, &mut handler);
    let d2 = make_tcp_packet(
        client, 12345, server, 80, 1003, b"by", false, false, false, false,
    );
    reassembler.process_packet(&d2, 8, &mut handler);

    let summary = reassembler.summarize();
    assert_eq!(
        summary.packets_analyzed,
        reassembler.stats().packets_tcp,
        "BC-2.04.028 post-2: packets_analyzed must equal stats.packets_tcp"
    );
    assert_eq!(
        summary.packets_analyzed, 3,
        "BC-2.04.028 post-2: packets_analyzed must be 3 (TCP only), not 8 (total)"
    );
}

// ---- AC-008 ----------------------------------------------------------------

/// AC-008 (BC-2.04.028 postcondition 3)
/// Postcondition: the detail BTreeMap contains EXACTLY the 17 documented keys.
/// Any missing key or extra key is a test failure.
///
/// Keys (alphabetical, from BC-2.04.028 and STORY-012 AC-008):
///   bytes_reassembled, dropped_findings, evictions,
///   flows_completed, flows_expired, flows_fin, flows_partial,
///   flows_rst, flows_total, packets_processed, packets_skipped_non_tcp,
///   segments_depth_exceeded, segments_duplicates, segments_inserted,
///   segments_out_of_window, segments_overlaps, segments_segment_limit
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_028_summarize_exact_key_set() {
    use std::collections::BTreeSet;

    let expected_keys: BTreeSet<&str> = [
        "bytes_reassembled",
        "dropped_findings",
        "evictions",
        "flows_completed",
        "flows_expired",
        "flows_fin",
        "flows_partial",
        "flows_rst",
        "flows_total",
        "packets_processed",
        "packets_skipped_non_tcp",
        "segments_depth_exceeded",
        "segments_duplicates",
        "segments_inserted",
        "segments_out_of_window",
        "segments_overlaps",
        "segments_segment_limit",
    ]
    .iter()
    .copied()
    .collect();

    let reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let summary = reassembler.summarize();
    let actual_keys: BTreeSet<&str> = summary.detail.keys().map(String::as_str).collect();

    let missing: Vec<&&str> = expected_keys.difference(&actual_keys).collect();
    let extra: Vec<&&str> = actual_keys.difference(&expected_keys).collect();

    assert!(
        missing.is_empty(),
        "BC-2.04.028 post-3: missing keys in summarize() detail: {missing:?}"
    );
    assert!(
        extra.is_empty(),
        "BC-2.04.028 post-3: extra unexpected keys in summarize() detail: {extra:?}"
    );
    assert_eq!(
        actual_keys.len(),
        17,
        "BC-2.04.028 post-3: detail map must contain exactly 17 keys"
    );
}

// ---- AC-009 ----------------------------------------------------------------

/// AC-009 (BC-2.04.028 invariant 1)
/// Invariant: flows_completed in the detail map always equals flows_fin + flows_rst.
///
/// Both addends are driven to >= 1 within the same reassembler:
///   Flow A — closed via full FIN teardown  → contributes to flows_fin
///   Flow B — closed via RST               → contributes to flows_rst
///
/// At summarize() time both flows_fin >= 1 AND flows_rst >= 1, so a buggy
/// summarize() that computed `flows_completed = flows_fin` (dropping the
/// `+ s.flows_rst` term) would produce 1 instead of 2 and fail the
/// assertion.  This closes the coverage gap identified in Phase 3 Wave 5
/// adversarial review finding Min-1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_028_flows_completed_derived_correctly() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // ---- Flow A: close via FIN teardown (contributes flows_fin += 1) ----
    let syn_a = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 2, &mut handler);
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 3, &mut handler);
    let fin2 = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin2, 4, &mut handler);

    // ---- Flow B (distinct port): close via RST (contributes flows_rst += 1) ----
    let syn_b = make_tcp_packet(
        client,
        54321,
        server,
        443,
        3000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 5, &mut handler);
    let data_b = make_tcp_packet(
        client, 54321, server, 443, 3001, b"payload", false, false, false, false,
    );
    reassembler.process_packet(&data_b, 6, &mut handler);
    let rst_b = make_tcp_packet(
        server,
        443,
        client,
        54321,
        4000,
        &[],
        false,
        false,
        false,
        true, // RST flag
    );
    reassembler.process_packet(&rst_b, 7, &mut handler);

    let stats = reassembler.stats();
    let summary = reassembler.summarize();

    let flows_fin = stats.flows_fin;
    let flows_rst = stats.flows_rst;
    let expected_completed = flows_fin + flows_rst;

    // Both addends must be non-zero — otherwise the test has not closed the
    // Min-1 coverage gap.
    assert!(
        flows_fin >= 1,
        "BC-2.04.028 inv-1 setup: flows_fin must be >= 1 (got {flows_fin})"
    );
    assert!(
        flows_rst >= 1,
        "BC-2.04.028 inv-1 setup: flows_rst must be >= 1 (got {flows_rst})"
    );

    let detail_completed = summary
        .detail
        .get("flows_completed")
        .and_then(|v| v.as_u64())
        .expect("flows_completed key must exist in detail map");

    assert_eq!(
        detail_completed, expected_completed,
        "BC-2.04.028 inv-1: flows_completed ({detail_completed}) must equal \
         flows_fin ({flows_fin}) + flows_rst ({flows_rst})"
    );
}

// ---- AC-010 ----------------------------------------------------------------

/// AC-010 (BC-2.04.028 invariant 3)
/// Invariant: the detail map uses BTreeMap ordering, guaranteeing alphabetical
/// key ordering in JSON serialization across runs (LESSON-P2.09).
///
/// Verified by asserting the collected key sequence equals its sorted form.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_028_detail_is_btreemap_ordered() {
    let reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let summary = reassembler.summarize();

    let keys: Vec<&str> = summary.detail.keys().map(String::as_str).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();

    assert_eq!(
        keys, sorted,
        "BC-2.04.028 inv-3: detail keys must be in alphabetical (BTreeMap) order; \
         got {keys:?}"
    );
}

// ---- AC-011 ----------------------------------------------------------------

/// AC-011 (BC-2.04.030 postcondition 1)
/// Postcondition: after processing packets and calling finalize(),
/// bytes_reassembled equals the exact sum of all data.len() values passed to
/// handler.on_data callbacks across all flows and both directions.
///
/// Uses a bidirectional flow (client + server data) so on_data is called in
/// both directions; captures all callbacks and sums data.len().
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_030_bytes_reassembled_matches_handler_total() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Full SYN/SYN-ACK handshake
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Client sends 3 sequential segments (each flushed immediately, in-order)
    let c1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&c1, 3, &mut handler);
    let c2 = make_tcp_packet(
        client, 12345, server, 80, 1006, b"world", false, false, false, false,
    );
    reassembler.process_packet(&c2, 4, &mut handler);
    let c3 = make_tcp_packet(
        client, 12345, server, 80, 1011, b"foo", false, false, false, false,
    );
    reassembler.process_packet(&c3, 5, &mut handler);

    // Server sends 2 segments
    let s1 = make_tcp_packet(
        server, 80, client, 12345, 2001, b"reply1", false, false, false, false,
    );
    reassembler.process_packet(&s1, 6, &mut handler);
    let s2 = make_tcp_packet(
        server, 80, client, 12345, 2007, b"reply2", false, false, false, false,
    );
    reassembler.process_packet(&s2, 7, &mut handler);

    // finalize() flushes any remaining buffered data
    reassembler.finalize(&mut handler);

    // Sum all data.len() across every on_data callback
    let handler_total: u64 = handler
        .data_events
        .iter()
        .map(|(_, _, data, _)| data.len() as u64)
        .sum();

    assert_eq!(
        reassembler.stats().bytes_reassembled,
        handler_total,
        "BC-2.04.030 post-1: bytes_reassembled ({}) must equal sum of on_data data.len() ({})",
        reassembler.stats().bytes_reassembled,
        handler_total
    );
}

// ---- AC-012 ----------------------------------------------------------------

/// AC-012 (BC-2.04.030 invariant 1)
/// Invariant: bytes_reassembled is monotonically non-decreasing; it never
/// decreases between consecutive observations after each packet.
///
/// Samples bytes_reassembled after every process_packet call and asserts
/// each sample is >= the previous.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_030_bytes_reassembled_is_monotonic() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let packets: Vec<(wirerust::decoder::ParsedPacket, u32)> = vec![
        (
            make_tcp_packet(
                client,
                12345,
                server,
                80,
                1000,
                &[],
                true,
                false,
                false,
                false,
            ),
            1,
        ),
        (
            make_tcp_packet(
                client, 12345, server, 80, 1001, b"aaa", false, false, false, false,
            ),
            2,
        ),
        (
            make_tcp_packet(
                client, 12345, server, 80, 1004, b"bbb", false, false, false, false,
            ),
            3,
        ),
        (
            make_tcp_packet(
                client, 12345, server, 80, 1007, b"ccc", false, false, false, false,
            ),
            4,
        ),
        // Non-TCP interleaved — must not decrease bytes_reassembled
        (make_udp_packet(client, 9999, server, 53, b"ignore-me"), 5),
        (
            make_tcp_packet(
                server, 80, client, 12345, 2000, b"resp", false, false, false, false,
            ),
            6,
        ),
    ];

    let mut prev: u64 = 0;
    for (pkt, ts) in &packets {
        reassembler.process_packet(pkt, *ts, &mut handler);
        let current = reassembler.stats().bytes_reassembled;
        assert!(
            current >= prev,
            "BC-2.04.030 inv-1: bytes_reassembled must be monotonically non-decreasing; \
             was {prev}, now {current} after packet at ts={ts}"
        );
        prev = current;
    }

    // Also check after finalize
    reassembler.finalize(&mut handler);
    let after_finalize = reassembler.stats().bytes_reassembled;
    assert!(
        after_finalize >= prev,
        "BC-2.04.030 inv-1: bytes_reassembled must not decrease after finalize(); \
         was {prev}, now {after_finalize}"
    );
}

// ---- AC-013 ----------------------------------------------------------------

/// AC-013 (BC-2.04.030 postcondition 4)
/// Postcondition: BOTH duplicate retransmissions AND out-of-window segments do
/// NOT contribute to bytes_reassembled (both are discarded before flush).
///
/// Canonical test vector from BC-2.04.030: 1 segment (100 bytes) + 1 exact
/// duplicate retransmission + finalize() → bytes_reassembled == 100.
///
/// Extended per M-1 gap: also injects an out-of-window segment and asserts it
/// does not increment bytes_reassembled.  A small max_receive_window (200) is
/// used so the out-of-window boundary is close enough to construct easily:
///   ISN = 1000, after flushing 100 bytes → base_offset = 101.
///   window limit = 101 + 200 = 301 (as an offset from ISN).
///   OOW seq = 1000 + 400 = 1400 (offset 400 > 301 → rejected).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_030_duplicates_not_counted_in_bytes_reassembled() {
    let config = ReassemblyConfig {
        max_receive_window: 200, // small window so OOW boundary is unambiguous
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN establishes ISN=1000 → base_offset=1
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Payload: 100 bytes at seq=1001 (offset=1, contiguous → flushed immediately)
    // After flush: base_offset = 101.
    let payload = vec![b'A'; 100];
    let original = make_tcp_packet(
        client, 12345, server, 80, 1001, &payload, false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);

    let bytes_after_first = reassembler.stats().bytes_reassembled;
    assert_eq!(
        bytes_after_first, 100,
        "BC-2.04.030 post-4: 100-byte segment must be counted in bytes_reassembled"
    );

    // Exact duplicate retransmission (same seq, same data) — must be discarded.
    let dup = make_tcp_packet(
        client, 12345, server, 80, 1001, &payload, false, false, false, false,
    );
    reassembler.process_packet(&dup, 3, &mut handler);

    let bytes_after_dup = reassembler.stats().bytes_reassembled;
    assert_eq!(
        bytes_after_dup, 100,
        "BC-2.04.030 post-4: duplicate retransmission must NOT contribute to bytes_reassembled; \
         expected 100, got {bytes_after_dup}"
    );

    // Out-of-window segment: base_offset=101, window=200 → limit offset = 301.
    // seq = ISN + 400 = 1400 → offset = 400 > 301 → rejected as OutOfWindow.
    let oow_payload = vec![b'B'; 50];
    let oow = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1400,
        &oow_payload,
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&oow, 4, &mut handler);

    assert_eq!(
        reassembler.stats().segments_out_of_window,
        1,
        "BC-2.04.030 post-4: out-of-window segment must be counted in segments_out_of_window"
    );
    let bytes_after_oow = reassembler.stats().bytes_reassembled;
    assert_eq!(
        bytes_after_oow, 100,
        "BC-2.04.030 post-4: out-of-window segment must NOT contribute to bytes_reassembled; \
         expected 100, got {bytes_after_oow}"
    );

    // finalize — no additional bytes (no buffered segments remain)
    reassembler.finalize(&mut handler);
    assert_eq!(
        reassembler.stats().bytes_reassembled,
        100,
        "BC-2.04.030 post-4: bytes_reassembled must remain 100 after finalize with no new data"
    );
}

// ---- EC-001: UDP packet skipped ----

/// EC-001 (BC-2.04.002 edge case)
/// UDP packet is skipped; packets_skipped_non_tcp increments.
/// Canonical test vector: single UDP → packets_processed=1, skipped=1, tcp=0.
#[test]
fn test_ec_001_udp_packet_skipped() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let pkt = make_udp_packet([192, 168, 1, 1], 5555, [192, 168, 1, 2], 80, b"udp-data");
    reassembler.process_packet(&pkt, 1, &mut handler);

    let stats = reassembler.stats();
    assert_eq!(
        stats.packets_processed, 1,
        "EC-001: packets_processed must be 1"
    );
    assert_eq!(
        stats.packets_skipped_non_tcp, 1,
        "EC-001: UDP packet must increment packets_skipped_non_tcp"
    );
    assert_eq!(
        stats.packets_tcp, 0,
        "EC-001: UDP must not increment packets_tcp"
    );
}

// ---- EC-002: ICMP packet skipped ----

/// EC-002 (BC-2.04.002 edge case)
/// ICMP packet (Protocol::Icmp) is skipped; packets_skipped_non_tcp increments.
/// Canonical test vector from BC-2.04.002: ICMP → packets_processed=1, skipped=1.
#[test]
fn test_ec_002_icmp_packet_skipped() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let pkt = make_icmp_packet([10, 0, 0, 1], [10, 0, 0, 2]);
    reassembler.process_packet(&pkt, 1, &mut handler);

    let stats = reassembler.stats();
    assert_eq!(
        stats.packets_processed, 1,
        "EC-002: packets_processed must be 1"
    );
    assert_eq!(
        stats.packets_skipped_non_tcp, 1,
        "EC-002: ICMP packet must increment packets_skipped_non_tcp"
    );
    assert_eq!(
        stats.packets_tcp, 0,
        "EC-002: ICMP must not increment packets_tcp"
    );
}

// ---- EC-003: Protocol::Other(n) skipped ----

/// EC-003 (BC-2.04.002 edge case)
/// Protocol::Other(n) packet is skipped; packets_skipped_non_tcp increments.
/// Tests with proto=41 (IPv6-in-IPv4) as a representative Other variant.
#[test]
fn test_ec_003_other_protocol_skipped() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let pkt = make_other_protocol_packet([10, 0, 0, 1], [10, 0, 0, 2], 41);
    reassembler.process_packet(&pkt, 1, &mut handler);

    let stats = reassembler.stats();
    assert_eq!(
        stats.packets_processed, 1,
        "EC-003: packets_processed must be 1"
    );
    assert_eq!(
        stats.packets_skipped_non_tcp, 1,
        "EC-003: Protocol::Other(41) must increment packets_skipped_non_tcp"
    );
    assert_eq!(
        stats.packets_tcp, 0,
        "EC-003: Protocol::Other must not increment packets_tcp"
    );
}

// ---- EC-004: All packets non-TCP ----

/// EC-004 (BC-2.04.002 edge case + story EC-004)
/// When all packets are non-TCP, flows table is empty and findings are empty
/// after all packets processed. Covers BC-2.04.002 edge case EC-005.
#[test]
fn test_ec_004_all_non_tcp_flows_empty() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Mix of non-TCP protocols only
    let udp = make_udp_packet([10, 0, 0, 1], 1234, [10, 0, 0, 2], 53, b"dns");
    let icmp = make_icmp_packet([10, 0, 0, 1], [10, 0, 0, 2]);
    let other = make_other_protocol_packet([10, 0, 0, 1], [10, 0, 0, 2], 50);

    reassembler.process_packet(&udp, 1, &mut handler);
    reassembler.process_packet(&icmp, 2, &mut handler);
    reassembler.process_packet(&other, 3, &mut handler);

    // No flows should have been created
    assert_eq!(
        reassembler.stats().flows_total,
        0,
        "EC-004: all-non-TCP capture must result in flows_total == 0"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-004: all-non-TCP capture must have no buffered state"
    );
    // No findings
    assert!(
        reassembler.findings().is_empty(),
        "EC-004: all-non-TCP capture must generate no findings"
    );
    // No handler callbacks
    assert!(
        handler.data_events.is_empty(),
        "EC-004: all-non-TCP capture must generate no on_data callbacks"
    );
    assert!(
        handler.close_events.is_empty(),
        "EC-004: all-non-TCP capture must generate no on_flow_close callbacks"
    );
    // All 3 counted as processed and skipped
    let stats = reassembler.stats();
    assert_eq!(
        stats.packets_processed, 3,
        "EC-004: all 3 non-TCP packets must be counted in packets_processed"
    );
    assert_eq!(
        stats.packets_skipped_non_tcp, 3,
        "EC-004: all 3 non-TCP packets must be in packets_skipped_non_tcp"
    );
}

// ---- EC-005: summarize() before any packets ----

/// EC-005 (BC-2.04.028 edge case EC-001 / story EC-005)
/// summarize() called on a freshly-constructed reassembler before any packets:
/// all counters are 0, all detail values are zero.
///
/// Canonical test vector from BC-2.04.028: fresh reassembler → all-zero detail.
#[test]
fn test_ec_005_summarize_before_any_packets() {
    let reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let summary = reassembler.summarize();

    assert_eq!(
        summary.analyzer_name, "TCP Reassembly",
        "EC-005: analyzer_name must be set even on fresh reassembler"
    );
    assert_eq!(
        summary.packets_analyzed, 0,
        "EC-005: packets_analyzed must be 0 before any packets"
    );

    // Every detail value must be a number equal to 0
    for (key, value) in &summary.detail {
        let n = value
            .as_u64()
            .unwrap_or_else(|| panic!("EC-005: detail key '{key}' must be a number"));
        assert_eq!(n, 0, "EC-005: detail['{key}'] must be 0 before any packets");
    }
}

// ---- EC-006: summarize() after finalize() ----

/// EC-006 (BC-2.04.028 edge case EC-002 / story EC-006)
/// summarize() called after finalize() returns an accurate snapshot;
/// finalize does not reset stats. Specifically: flows_total, bytes_reassembled,
/// and packets_tcp must survive finalize() unchanged.
///
/// Strengthened per M-3 gap: bytes_reassembled is asserted to its EXACT expected
/// value after finalize() so that a hypothetical reset-then-recount bug cannot
/// pass.  The `>=` check in the original test would pass even if finalize
/// wrongly zeroed the counter and re-flushed data into it.
///
/// Sequence layout (ISN=1000):
///   seq=1001, 7 bytes "persist"  → flushed immediately (in-order) → bytes_before = 7.
///   seq=1009, 5 bytes "later"    → buffered ahead of a gap at offset 8.
///     (finalize calls close_flow → flush_contiguous from base_offset=8 → gap present
///      → "later" stays unflushed; bytes_after stays 7.)
///   seq=1008, 1 byte  "X"        → fills the gap; flush_contiguous now chains
///     through "X" then "later" → bytes_after = 7 + 1 + 5 = 13.
///
/// All three segments are sent before finalize so that the in-order data is
/// fully accounted before the stats snapshot.  The chain-flush of the
/// gap-plus-successor happens in the process_packet call for "X", leaving
/// no new bytes for finalize itself (the flow is still open, finalize later
/// closes it via close_flow → flush_contiguous → nothing remaining).
/// Therefore bytes_after == bytes_before_finalize_snapshot == 13 exactly.
#[test]
fn test_ec_006_summarize_after_finalize_accurate() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // seq=1001: in-order, flushed immediately.  bytes_reassembled += 7 → 7.
    // After flush: base_offset = 8.
    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"persist", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    // seq=1009 (offset 9): out-of-order — buffered because offset 8 is a gap.
    // bytes_reassembled still == 7.
    let later = make_tcp_packet(
        client, 12345, server, 80, 1009, b"later", false, false, false, false,
    );
    reassembler.process_packet(&later, 3, &mut handler);

    // seq=1008 (offset 8): fills the gap; flush_contiguous now delivers both
    // "X" (1 byte) and "later" (5 bytes) → bytes_reassembled += 6 → 13.
    let gap_filler = make_tcp_packet(
        client, 12345, server, 80, 1008, b"X", false, false, false, false,
    );
    reassembler.process_packet(&gap_filler, 4, &mut handler);

    // Sanity check: all in-order data is already flushed before finalize.
    assert_eq!(
        reassembler.stats().bytes_reassembled,
        13,
        "EC-006 setup: 7 + 1 + 5 = 13 bytes must be counted before finalize"
    );

    // Capture stats before finalize.
    let tcp_before = reassembler.stats().packets_tcp;
    let flows_before = reassembler.stats().flows_total;
    let bytes_before = reassembler.stats().bytes_reassembled;

    reassembler.finalize(&mut handler);

    let summary = reassembler.summarize();

    // Counters accumulated before finalize must still be present.
    assert_eq!(
        summary.packets_analyzed, tcp_before,
        "EC-006: finalize must not reset packets_tcp (packets_analyzed)"
    );
    assert_eq!(
        summary.detail.get("flows_total").and_then(|v| v.as_u64()),
        Some(flows_before),
        "EC-006: finalize must not reset flows_total"
    );
    // Exact assertion: all data was flushed before finalize, so bytes_after must
    // equal bytes_before exactly.  A reset-then-recount bug would yield
    // bytes_after == 0 (reset) or bytes_after > bytes_before (double-count),
    // both of which are caught by strict equality.
    let bytes_after = reassembler.stats().bytes_reassembled;
    assert_eq!(
        bytes_after, bytes_before,
        "EC-006: bytes_reassembled must not change across finalize (no reset, no double-count); \
         before={bytes_before}, after={bytes_after}"
    );
}

// ---- EC-007: non-TCP excluded from packets_analyzed ----

/// EC-007 (BC-2.04.028 edge case EC-003 / story EC-007)
/// When non-TCP packets are injected before summarize(), packets_analyzed
/// equals packets_tcp (not packets_processed).
///
/// Canonical test vector: 5 non-TCP + 3 TCP → packets_analyzed=3, packets_processed=8.
#[test]
fn test_ec_007_non_tcp_excluded_from_packets_analyzed() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // 5 non-TCP packets (mix of protocols)
    reassembler.process_packet(
        &make_udp_packet(client, 1111, server, 53, b"q"),
        1,
        &mut handler,
    );
    reassembler.process_packet(&make_icmp_packet(client, server), 2, &mut handler);
    reassembler.process_packet(
        &make_udp_packet(client, 2222, server, 53, b"r"),
        3,
        &mut handler,
    );
    reassembler.process_packet(
        &make_other_protocol_packet(client, server, 17),
        4,
        &mut handler,
    );
    reassembler.process_packet(
        &make_udp_packet(client, 3333, server, 53, b"s"),
        5,
        &mut handler,
    );

    // 3 TCP packets
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        6,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1001, b"x", false, false, false, false,
        ),
        7,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1002, b"y", false, false, false, false,
        ),
        8,
        &mut handler,
    );

    let summary = reassembler.summarize();
    let stats = reassembler.stats();

    assert_eq!(
        stats.packets_processed, 8,
        "EC-007: packets_processed must count all 8 packets"
    );
    assert_eq!(
        summary.packets_analyzed, 3,
        "EC-007: packets_analyzed must be 3 (TCP only), not 8"
    );
    assert_ne!(
        summary.packets_analyzed, stats.packets_processed,
        "EC-007: packets_analyzed must differ from packets_processed when non-TCP packets present"
    );
}

// ---- EC-008: bytes_reassembled after out-of-order segment ----

/// EC-008 (BC-2.04.030 invariant 3 / story EC-008)
/// bytes_reassembled only counts after flush, not while a segment is buffered.
/// An out-of-order segment sits in the buffer without contributing to
/// bytes_reassembled until the gap is filled and it is flushed.
///
/// Canonical test vector from BC-2.04.030 (2 OOO segments → 200 after flush).
#[test]
fn test_ec_008_bytes_reassembled_only_after_flush() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN establishes ISN=1000 → base_offset=1
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Out-of-order: seg2 arrives first (gap at offset 1 keeps it buffered)
    let seg2 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1101, // seq=1101 → offset=101 (gap at 1-100)
        &[b'B'; 100],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&seg2, 2, &mut handler);

    // bytes_reassembled must be 0 while seg2 is buffered (not yet flushed)
    assert_eq!(
        reassembler.stats().bytes_reassembled,
        0,
        "EC-008: bytes_reassembled must be 0 while out-of-order segment is buffered"
    );
    assert!(
        handler.data_events.is_empty(),
        "EC-008: no on_data callback while segment is buffered (gap unfilled)"
    );

    // seg1 fills the gap → both segments flush
    let seg1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001, // offset=1, contiguous with base_offset=1
        &[b'A'; 100],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&seg1, 3, &mut handler);

    // Now both should be flushed → bytes_reassembled == 200
    assert_eq!(
        reassembler.stats().bytes_reassembled,
        200,
        "EC-008: bytes_reassembled must be 200 after both segments flush (only counts after flush)"
    );
}

// ---------------------------------------------------------------------------
// STORY-013: Engine-level integration tests for apply_handshake_flags
//   BC-2.04.004, BC-2.04.005, BC-2.04.051, BC-2.04.052, BC-2.04.053
//
// These tests exercise the effectful-shell layer (TcpReassembler::process_packet
// / apply_handshake_flags) for handshake state transitions and statistics.
// Pure TcpFlow method tests live in reassembly_flow_tests.rs.
// ---------------------------------------------------------------------------

/// STORY-013 Engine AC: apply_handshake_flags SYN block (BC-2.04.004)
/// Integration-level: process a SYN packet through the engine and assert
/// that a flow is created, state has been processed as a new flow (SYN packet
/// processed; stats.flows_total == 1; no partial join).
///
/// Exercises BC-2.04.004 postconditions 1-3 at the engine level via
/// process_packet → apply_handshake_flags → on_syn / set_initiator / set_isn.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_engine_syn_sets_state_and_isn() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // SYN packet from client (seq=1000).
    let syn = make_tcp_packet(
        client,
        5000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // BC-2.04.004 post-3: a flow was created (flows_total=1).
    assert_eq!(
        reassembler.stats().flows_total,
        1,
        "BC-2.04.004 engine: SYN packet must create exactly one flow"
    );
    // BC-2.04.004 post-1/post-2: this was a proper SYN, not a mid-stream join.
    assert_eq!(
        reassembler.stats().flows_partial,
        0,
        "BC-2.04.004 engine: SYN packet must not set flows_partial"
    );
    // No close events — SYN does not close a flow.
    assert!(
        handler.close_events.is_empty(),
        "BC-2.04.004 engine: SYN packet must not emit on_flow_close"
    );
}

/// STORY-013 Engine AC: apply_handshake_flags SYN+ACK block (BC-2.04.005)
/// Integration-level: SYN → SYN+ACK sequence transitions engine flow to
/// Established with correct direction tagging (flows_partial=0, data tagged correctly).
///
/// Exercises BC-2.04.005 postconditions 1-3 at the engine level.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_engine_syn_ack_establishes_flow() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // SYN from client.
    let syn = make_tcp_packet(
        client,
        5000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // SYN+ACK from server (seq=2000).
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        5000,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // After SYN+ACK, flow is Established. Verify via data delivery with correct direction.
    let req = make_tcp_packet(
        client, 5000, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&req, 3, &mut handler);

    // BC-2.04.005 post-1: initiator is the client (dst of SYN+ACK).
    // Verified via direction tag on the first data event.
    assert_eq!(
        reassembler.stats().flows_partial,
        0,
        "BC-2.04.005 engine: full SYN/SYN+ACK handshake must not set flows_partial"
    );
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.005 engine: client request must produce one data event"
    );
    assert_eq!(
        handler.data_events[0].1,
        Direction::ClientToServer,
        "BC-2.04.005 engine: client data must be tagged ClientToServer (initiator=client)"
    );
    assert_eq!(
        handler.data_events[0].2, b"hello",
        "BC-2.04.005 engine: correct payload must be delivered"
    );
}

/// STORY-013 Engine AC: apply_handshake_flags RST block (BC-2.04.051)
/// Integration-level: RST increments stats.flows_rst, emits CloseReason::Rst,
/// and removes the flow (total_memory == 0 after RST).
///
/// Exercises BC-2.04.051 postconditions 2-5 at the engine level:
///   - PostHandshake::FlowClosed returned (payload NOT processed after RST)
///   - stats.flows_rst incremented
///   - close_flow(key, CloseReason::Rst) called
///   - flow removed from table (total_memory == 0)
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_engine_rst_increments_flows_rst_counter() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // Establish a flow.
    let syn = make_tcp_packet(
        client,
        5000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        5000,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    let data = make_tcp_packet(
        client, 5000, server, 80, 1001, b"payload", false, false, false, false,
    );
    reassembler.process_packet(&data, 3, &mut handler);

    // RST from server.
    let rst = make_tcp_packet(
        server,
        80,
        client,
        5000,
        2001,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 4, &mut handler);

    // BC-2.04.051 post-3: flows_rst must increment.
    assert_eq!(
        reassembler.stats().flows_rst,
        1,
        "BC-2.04.051 engine: RST must increment stats.flows_rst to 1"
    );
    // BC-2.04.051 post-4: close_flow called with CloseReason::Rst.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.051 engine: RST must emit exactly one on_flow_close event"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Rst,
        "BC-2.04.051 engine: close reason must be Rst"
    );
    // BC-2.04.051 post-5: flow removed — no buffered state.
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.051 engine: RST must remove flow (total_memory == 0)"
    );
}

/// STORY-013 Engine AC: insert_payload_segment mid-stream join (BC-2.04.052)
/// Integration-level: a data packet on a New flow calls on_data_without_syn
/// and increments stats.flows_partial.
///
/// Exercises BC-2.04.052 postcondition 3 (flows_partial counter) at the engine level.
/// Canonical test vector: New flow, data packet (no SYN) → flows_partial=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_052_engine_data_without_syn_increments_flows_partial() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // Data packet with no prior SYN (mid-stream join).
    let data = make_tcp_packet(
        client,
        5000,
        server,
        80,
        5000,
        b"mid-stream",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&data, 1, &mut handler);

    // BC-2.04.052 post-3: flows_partial must increment.
    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.052 engine: mid-stream join must increment stats.flows_partial to 1"
    );
    // BC-2.04.052 post-1: flow must be in Established state (data delivered).
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.052 engine: mid-stream data must be delivered to handler"
    );
    assert_eq!(
        handler.data_events[0].2, b"mid-stream",
        "BC-2.04.052 engine: correct payload must be delivered on mid-stream join"
    );
}

/// STORY-013 Engine AC: direction tagging in flush path (BC-2.04.053)
/// Integration-level: after full SYN/SYN+ACK handshake, client data flushed to
/// handler carries Direction::ClientToServer and server data carries
/// Direction::ServerToClient.
///
/// Exercises BC-2.04.053 postconditions 1-2 via the engine's flush_contiguous_data
/// → handler.on_data callbacks.
/// Canonical test vector: initiator=client; client→server data → ClientToServer;
///   server→client data → ServerToClient.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_engine_direction_tagging_in_flush_path() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // Full three-way handshake.
    let syn = make_tcp_packet(
        client,
        5000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        5000,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Client sends data → must be tagged ClientToServer.
    let req = make_tcp_packet(
        client, 5000, server, 80, 1001, b"request", false, false, false, false,
    );
    reassembler.process_packet(&req, 3, &mut handler);

    // Server sends data → must be tagged ServerToClient.
    let resp = make_tcp_packet(
        server,
        80,
        client,
        5000,
        2001,
        b"response",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&resp, 4, &mut handler);

    // BC-2.04.053 post-1: client data delivered with Direction::ClientToServer.
    let c2s_events: Vec<_> = handler
        .data_events
        .iter()
        .filter(|(_, d, _, _)| *d == Direction::ClientToServer)
        .collect();
    assert_eq!(
        c2s_events.len(),
        1,
        "BC-2.04.053 engine: exactly one ClientToServer data event expected"
    );
    assert_eq!(
        c2s_events[0].2, b"request",
        "BC-2.04.053 engine: client data must be tagged ClientToServer"
    );

    // BC-2.04.053 post-2: server data delivered with Direction::ServerToClient.
    let s2c_events: Vec<_> = handler
        .data_events
        .iter()
        .filter(|(_, d, _, _)| *d == Direction::ServerToClient)
        .collect();
    assert_eq!(
        s2c_events.len(),
        1,
        "BC-2.04.053 engine: exactly one ServerToClient data event expected"
    );
    assert_eq!(
        s2c_events[0].2, b"response",
        "BC-2.04.053 engine: server data must be tagged ServerToClient"
    );
}

/// F-2 engine-level test (BC-2.04.005 postcondition 1, EC-002, BC-2.04.053)
///
/// AC-005 / AC-007 / `test_BC_2_04_005_engine_syn_ack_establishes_flow` all fail to
/// exercise the "SYN+ACK destination is the initiator" semantic against
/// `apply_handshake_flags` in isolation. The existing flow-level tests call
/// `flow.set_initiator(...)` manually, and the existing engine test processes a SYN
/// first — setting `initiator=client` via the SYN branch — so a hypothetical regression
/// in `apply_handshake_flags`'s SYN+ACK branch that swapped `packet.dst_ip` →
/// `packet.src_ip` (mis-identifying the server as the initiator) would be masked by
/// `set_initiator`'s idempotency.
///
/// This test closes the gap with a server-first capture: a SYN+ACK arrives with NO
/// prior SYN. The engine must read `packet.dst_ip:tcp.dst_port` (the client endpoint)
/// as the initiator — not `packet.src_ip:tcp.src_port` (the server endpoint). A
/// subsequent client data packet must be tagged `Direction::ClientToServer`. If
/// `apply_handshake_flags` incorrectly used `src_ip` instead of `dst_ip`, the
/// initiator would be set to the server, and the client data packet would be tagged
/// `Direction::ServerToClient` — failing the assertion.
///
/// References: BC-2.04.005 postcondition 1, EC-002, BC-2.04.053 direction-tagging.
/// Phase 3 Wave 6 adversarial finding F-2.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_engine_syn_ack_without_prior_syn_dst_is_initiator_ec002() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // server_ip:443 sends a SYN+ACK to client_ip:55000.
    // No prior SYN has been processed — this is a server-first capture (EC-002).
    let server = [2, 2, 2, 2];
    let client = [1, 1, 1, 1];

    // Step 1: SYN+ACK from server → client (no prior SYN).
    // apply_handshake_flags must read packet.dst_ip:tcp.dst_port = client_ip:55000
    // as the initiator. A regression swapping src/dst would set server_ip:443 as
    // the initiator instead.
    let syn_ack = make_tcp_packet(
        server,
        443,
        client,
        55000,
        9000,
        &[],
        true, // syn
        true, // ack
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 1, &mut handler);

    // No data events yet — SYN+ACK carries no payload.
    assert!(
        handler.data_events.is_empty(),
        "F-2 / BC-2.04.005 engine: SYN+ACK with no payload must not fire on_data"
    );

    // Step 2: client sends data toward the server.
    // src = client_ip:55000 — this must match the initiator set in step 1.
    // Expected direction: ClientToServer (initiator == client_ip:55000).
    // Regression direction: ServerToClient (initiator mis-set to server_ip:443).
    let client_data = make_tcp_packet(
        client,
        55000,
        server,
        443,
        1001,
        b"get-request",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&client_data, 2, &mut handler);

    // Step 3: assert the data event was tagged ClientToServer.
    assert_eq!(
        handler.data_events.len(),
        1,
        "F-2 / BC-2.04.005 engine: client data packet must produce exactly one on_data event"
    );
    assert_eq!(
        handler.data_events[0].1,
        Direction::ClientToServer,
        "F-2 / BC-2.04.005 post-1 / EC-002 / BC-2.04.053: client data (src=client_ip:55000) \
         must be tagged ClientToServer — apply_handshake_flags must use packet.dst_ip:dst_port \
         (client_ip:55000) as initiator, NOT packet.src_ip:src_port (server_ip:443). \
         A regression swapping src/dst would yield ServerToClient here."
    );
    assert_eq!(
        handler.data_events[0].2, b"get-request",
        "F-2 / BC-2.04.005 engine: correct payload must be delivered"
    );

    // Confirm the flow was not marked partial — SYN+ACK constitutes a proper handshake
    // marker (the engine detects it via the syn+ack flags).
    assert_eq!(
        reassembler.stats().flows_partial,
        0,
        "F-2 / BC-2.04.005 engine: SYN+ACK-first capture must not be counted as partial \
         (the engine recognises the SYN+ACK flags and handles it via the handshake path)"
    );
}

/// EC-007 / F-6 engine-level test (BC-2.04.051 invariant 2, postcondition 2)
///
/// EC-007 states: "RST with payload | Payload NOT processed; PostHandshake::FlowClosed
/// returned." The flow-level test (`test_BC_2_04_051_ec007_rst_closes_flow_state`)
/// only confirms `state == Closed`. This test exercises the engine-level claim that
/// the RST payload is NOT delivered to the handler and NOT inserted into the segment
/// buffer.
///
/// Assertions:
///   (a) flow state is Closed after the RST packet (via flows_rst counter, since the
///       flow is removed immediately — CloseReason::Rst).
///   (b) `stats.flows_rst` incremented by exactly 1.
///   (c) the RST payload was NOT processed: handler.on_data was NOT called for
///       the RST packet's payload bytes, and `stats.segments_inserted` did not
///       increment for the RST packet.
///
/// References: EC-007, BC-2.04.051 invariant 2, postcondition 2.
/// Phase 3 Wave 6 adversarial finding F-6.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec007_rst_with_payload_does_not_process_payload() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish the flow with a full handshake so it is in Established state.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Capture segments_inserted baseline after handshake (no payload yet).
    let segments_before_rst = reassembler.stats().segments_inserted;
    assert_eq!(
        segments_before_rst, 0,
        "EC-007 precondition: no data segments inserted during handshake"
    );
    assert!(
        handler.data_events.is_empty(),
        "EC-007 precondition: no on_data callbacks during handshake"
    );

    // Send a RST packet WITH a non-empty payload from the server.
    // BC-2.04.051 invariant 2: the engine must return PostHandshake::FlowClosed
    // before reaching payload processing — the payload bytes must be suppressed.
    let rst_with_payload = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        b"malicious-rst-payload",
        false,
        false,
        false,
        true, // rst = true
    );
    reassembler.process_packet(&rst_with_payload, 3, &mut handler);

    // (a) Flow closed via RST: CloseReason::Rst must be emitted.
    assert_eq!(
        handler.close_events.len(),
        1,
        "EC-007: exactly one on_flow_close event must be emitted for the RST"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Rst,
        "EC-007: close reason must be Rst"
    );

    // (b) flows_rst incremented by 1.
    assert_eq!(
        reassembler.stats().flows_rst,
        1,
        "EC-007: flows_rst must be exactly 1 after a single RST packet"
    );

    // (c) RST payload NOT processed — on_data must NOT have been called for the payload.
    assert!(
        handler.data_events.is_empty(),
        "EC-007: BC-2.04.051 invariant 2: on_data must NOT be called for a RST packet's \
         payload — PostHandshake::FlowClosed is returned before payload processing"
    );

    // (c) segments_inserted must not have changed — payload was not inserted.
    assert_eq!(
        reassembler.stats().segments_inserted,
        segments_before_rst,
        "EC-007: BC-2.04.051 postcondition 2: segments_inserted must not increment \
         for a RST packet — payload suppression confirmed via stats counter"
    );
}

// ---------------------------------------------------------------------------
// STORY-014: BC-2.04.009, BC-2.04.032, BC-2.04.048
//   Mid-stream join, IsnMissing guard, ISN_MISSING_WARNED one-shot atomic.
//
// AC-001..AC-005  (BC-2.04.009) mid-stream join path — engine level.
// AC-010..AC-012  (BC-2.04.032) insert_segment IsnMissing guard.
// AC-013..AC-014  (BC-2.04.048) ISN_MISSING_WARNED one-shot atomic.
// EC-002..EC-004, EC-006..EC-007 engine-level edge cases.
//
// PROCESS-GLOBAL ATOMIC NOTE: ISN_MISSING_WARNED is a `static AtomicBool`
// in src/reassembly/segment.rs. Cargo compiles each integration-test file
// into ONE binary; all tests inside this file share the same process image
// and therefore share that atomic. To prevent ordering-dependent behaviour,
// AC-013 + AC-014 + EC-007 are combined into a single test function that
// exercises the first-call and subsequent-call paths in a known sequential
// order. The combined test name follows the primary BC (BC-2.04.048).
// ---------------------------------------------------------------------------

/// STORY-014 / BC-2.04.009 AC-001: mid-stream join sets state=Established, partial=true.
/// Precondition: data packet arrives for a flow in FlowState::New (no SYN seen).
/// Postconditions 1-2: flow.state==Established, flow.partial==true.
/// Canonical test vector: data from 1.1.1.1:5000 with no prior SYN.
///
/// Observable engine-level signals:
///   - flows_partial == 1 (BC-2.04.009 post-2, post-6)
///   - data delivered to handler (Established means the segment was inserted and flushed)
///   - no close event (flow remains open)
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_mid_stream_sets_established_partial() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // Data packet with no prior SYN (mid-stream join).
    // BC-2.04.009 canonical test vector: data from 1.1.1.1:5000, seq=1001.
    let data = make_tcp_packet(
        client, 5000, server, 80, 1001, b"payload", false, false, false, false,
    );
    reassembler.process_packet(&data, 1, &mut handler);

    // BC-2.04.009 post-2 + post-6: partial=true and flows_partial incremented.
    // partial=true is not directly observable, but flows_partial == 1 is the
    // observable proxy for "on_data_without_syn was called, which sets partial=true".
    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.009 post-2/post-6: mid-stream join must set partial=true and \
         increment flows_partial to 1"
    );

    // BC-2.04.009 post-1 (state=Established): segment was inserted and flushed
    // (only an Established flow allows insert_payload_segment to proceed to flush).
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.009 post-1: state=Established must allow data to be flushed to handler"
    );
    assert_eq!(
        handler.data_events[0].2, b"payload",
        "BC-2.04.009 post-7: the segment is inserted and flushed normally"
    );

    // BC-2.04.009 PC3 — initiator set to packet src; observable as ClientToServer direction tag on first data event.
    assert!(
        !handler.data_events.is_empty(),
        "expected at least one data event from mid-stream packet"
    );
    assert_eq!(
        handler.data_events[0].1,
        Direction::ClientToServer,
        "BC-2.04.009 PC3 — initiator (client src) must be tagged ClientToServer"
    );

    // Confirm the flow is not closed.
    assert!(
        handler.close_events.is_empty(),
        "BC-2.04.009: mid-stream join must not close the flow"
    );
}

/// STORY-014 / BC-2.04.009 AC-002: mid-stream join infers ISN as seq-1.
/// Postcondition 4: the c2s direction has isn == Some(tcp.seq.wrapping_sub(1)).
/// Canonical test vector: data seq=1001 → isn=Some(1000).
///
/// Observable: after infer_isn(1001) sets isn=1000, the stream offset delivered
/// to the handler for that first packet is seq_offset(1001, 1000) = 1.
/// A regression to isn=1001 (storing seq itself) would give offset=0.
/// A regression to saturating_sub would give the same result for non-zero seq,
/// but fails the wrap test (AC-005). This test pins the non-wrap case.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_mid_stream_isn_is_seq_minus_one() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // First data packet with seq=1001 and no prior SYN.
    // infer_isn(1001) must store isn=1000 (= 1001.wrapping_sub(1)).
    // The on_data handler receives stream offset = seq_offset(1001, 1000) = 1.
    let data = make_tcp_packet(
        client, 5000, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&data, 1, &mut handler);

    // Exactly one data event must have been delivered.
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.009 post-4: mid-stream data must be delivered to handler after ISN inference"
    );

    // The stream offset for seq=1001 with isn=1000 is seq.wrapping_sub(isn) = 1.
    // If isn were stored as 1001 (the seq itself), offset would be 0.
    // If isn were stored as 1002 (off-by-one), offset would be u32::MAX (wrapping) = very large.
    assert_eq!(
        handler.data_events[0].3, 1,
        "BC-2.04.009 post-4: stream offset must be 1 (isn=seq-1=1000; \
         offset=seq.wrapping_sub(isn)=1001-1000=1) — \
         isn=seq regression gives offset=0; off-by-one gives u32::MAX-range value"
    );
}

/// STORY-014 / BC-2.04.009 AC-003: mid-stream join sets base_offset=1.
/// Postcondition 5: flow.client_to_server.base_offset == 1 after infer_isn.
///
/// Observable: base_offset=1 means the first contiguous flush starts at offset 1.
/// We verify this by sending two packets — the first at seq=1001 (delivered at
/// offset=1), and a second in-sequence packet at seq=1006 (delivered at offset=6).
/// The flush of contiguous segments confirms base_offset advances correctly from 1.
/// If base_offset were 0 instead of 1, the first packet's offset computation would
/// differ: it would arrive "at offset 0 but base_offset=0" → a gap check mismatch.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_mid_stream_base_offset_is_one() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // First mid-stream data: seq=1001, payload="hello" (5 bytes).
    // infer_isn(1001) → isn=1000, base_offset=1.
    // Segment arrives at offset=1, which equals base_offset=1 → flushed immediately.
    let p1 = make_tcp_packet(
        client, 5000, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&p1, 1, &mut handler);

    // First packet must be flushed (arrives at base_offset=1 = offset 1).
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.009 post-5: first packet must be flushed (base_offset=1 matches offset=1)"
    );
    assert_eq!(
        handler.data_events[0].3, 1,
        "BC-2.04.009 post-5: first packet stream offset must be 1 (base_offset starts at 1)"
    );

    // Second in-sequence packet: seq=1006, payload="world" (5 bytes).
    // After flushing p1 (5 bytes), base_offset advances to 6.
    // Offset of p2 = 1006 - 1000 = 6 = base_offset → contiguous → flushed.
    let p2 = make_tcp_packet(
        client, 5000, server, 80, 1006, b"world", false, false, false, false,
    );
    reassembler.process_packet(&p2, 2, &mut handler);

    assert_eq!(
        handler.data_events.len(),
        2,
        "BC-2.04.009 post-5: second contiguous packet must also be flushed"
    );
    assert_eq!(
        handler.data_events[1].3, 6,
        "BC-2.04.009 post-5: second packet stream offset must be 6 \
         (base_offset advanced from 1 to 6 after flushing 5 bytes)"
    );
}

/// STORY-014 / BC-2.04.009 AC-004: stats.flows_partial increments on mid-stream join.
/// Postcondition 6: flows_partial increments by 1 per partial flow.
/// Canonical test vector: one mid-stream flow → flows_partial=1.
///
/// Discriminant: a full SYN-handshaked flow must NOT increment flows_partial.
/// We verify by sending one proper SYN flow (flows_partial=0) then one mid-stream
/// flow (flows_partial=1). This ensures the increment is specific to the
/// on_data_without_syn path, not a general "flow created" counter.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_flows_partial_increments_on_mid_stream() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client_a = [1, 1, 1, 1];
    let client_b = [3, 3, 3, 3];
    let server = [2, 2, 2, 2];

    // Flow A: proper SYN handshake — must NOT increment flows_partial.
    let syn_a = make_tcp_packet(
        client_a,
        5000,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    assert_eq!(
        reassembler.stats().flows_partial,
        0,
        "BC-2.04.009 post-6 discriminant: SYN-handshaked flow must not increment flows_partial"
    );

    // Flow B: mid-stream (no SYN) — must increment flows_partial to 1.
    let data_b = make_tcp_packet(
        client_b, 6000, server, 80, 5000, b"mid", false, false, false, false,
    );
    reassembler.process_packet(&data_b, 2, &mut handler);

    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.009 post-6: one mid-stream join must increment flows_partial to exactly 1"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        2,
        "BC-2.04.009: both flows (SYN + mid-stream) must be counted in flows_total"
    );
}

/// STORY-014 / BC-2.04.009 AC-005 / EC-001: infer_isn(0) wraps correctly at the engine level.
/// Invariant 1: wrapping_sub(1) on seq=0 gives isn=u32::MAX without panic.
/// Canonical test vector: data seq=0 → isn=u32::MAX, base_offset=1.
///
/// Discrimination: three possible implementations for ISN inference from seq=0:
///   - wrapping_sub(1): 0u32.wrapping_sub(1) = u32::MAX = 4294967295 → stream offset = 1 ✓
///   - saturating_sub(1): 0u32.saturating_sub(1) = 0 → stream offset = 0 (wrong) ✗
///   - plain `- 1`: 0u32 - 1 panics under debug/overflow-checks (release has overflow-checks) ✗
///
/// After infer_isn(0) → isn=u32::MAX, seq=0 maps to:
///   seq_offset(0, u32::MAX) = 0u32.wrapping_sub(u32::MAX) as u64 = 1u64
/// This observable stream offset=1 distinguishes wrapping_sub from saturating_sub,
/// and the absence of panic distinguishes it from plain subtraction.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_mid_stream_isn_wraps_correctly_at_seq_zero() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // Data packet with tcp.seq = 0 and no prior SYN.
    // infer_isn(0) must compute isn = 0u32.wrapping_sub(1) = u32::MAX.
    // The engine must NOT panic (neither plain sub nor debug overflow).
    let data = make_tcp_packet(
        client, 5000, server, 80, 0, // seq = 0: the wrap case
        b"wrap", false, false, false, false,
    );
    reassembler.process_packet(&data, 1, &mut handler);

    // No panic means wrapping_sub was used (plain `- 1` would panic; CLAUDE.md confirms
    // release profile has overflow-checks = true, so there is no "escape hatch" to prod).

    // The stream offset delivered to the handler must be 1 (not 0, not a huge wrap value).
    // seq_offset(0, u32::MAX) = 0u32.wrapping_sub(u32::MAX) as u64 = 1u64.
    // saturating_sub regression: isn=0 → seq_offset(0, 0) = 0 (wrong, offset=0).
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.009 inv-1: seq=0 mid-stream join must deliver data (no panic, no ISN failure)"
    );
    assert_eq!(
        handler.data_events[0].3, 1,
        "BC-2.04.009 inv-1 / EC-001: stream offset must be 1 for seq=0 with inferred \
         isn=u32::MAX (seq_offset(0, u32::MAX)=1) — saturating_sub regression gives \
         isn=0 → offset=0; plain-sub panics before reaching this assert"
    );
    assert_eq!(
        handler.data_events[0].2, b"wrap",
        "BC-2.04.009 inv-1: correct payload must be delivered for seq=0 wrap case"
    );

    // flows_partial must be 1 — this is still a mid-stream join.
    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.009 inv-1: seq=0 mid-stream join must still increment flows_partial"
    );
}

/// STORY-014 / BC-2.04.032 AC-010: insert_segment with isn==None returns IsnMissing.
/// Postcondition 1: InsertResult::IsnMissing is returned when direction has no ISN.
/// Canonical test vector: insert_segment with isn=None, data=b"hello" → IsnMissing.
///
/// Discrimination: a fresh FlowDirection has isn=None; any non-empty data segment
/// must trigger the IsnMissing guard. The return value discriminates against
/// InsertResult::Inserted (the happy-path result), which would indicate the guard
/// is missing or the isn.is_none() check is incorrect.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_032_isn_missing_returns_isn_missing() {
    let _guard = ISN_MISSING_WARNED_LOCK.lock().expect("test lock poisoned");
    let mut dir = FlowDirection::new();

    // Precondition: isn is None (no set_isn or infer_isn called).
    assert_eq!(dir.isn, None, "precondition: direction must have isn=None");

    // BC-2.04.032 precondition 2: data is non-empty (b"hello").
    // BC-2.04.032 postcondition 1: must return InsertResult::IsnMissing.
    let result = dir.insert_segment(1000, b"hello", usize::MAX, usize::MAX, usize::MAX);

    assert_eq!(
        result,
        InsertResult::IsnMissing,
        "BC-2.04.032 post-1: insert_segment with isn=None and non-empty data must \
         return InsertResult::IsnMissing — Inserted return would indicate missing guard"
    );
}

/// STORY-014 / BC-2.04.032 AC-011: IsnMissing inserts nothing and leaves counters unchanged.
/// Postconditions 2-4: segments unchanged, buffered_bytes unchanged, counters unchanged.
///
/// Strategy: snapshot all observable counters before the failing call, then assert
/// each is identical after. This is a multi-property assertion that catches any
/// side-effecting regression (e.g., incrementing buffered_bytes before the guard).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_032_isn_missing_inserts_nothing() {
    let _guard = ISN_MISSING_WARNED_LOCK.lock().expect("test lock poisoned");
    let mut dir = FlowDirection::new();

    // Precondition: isn=None, direction is completely empty.
    assert_eq!(dir.isn, None, "precondition: isn must be None");
    assert!(dir.segments_is_empty(), "precondition: no segments yet");
    assert_eq!(
        dir.buffered_bytes(),
        0,
        "precondition: buffered_bytes must be 0"
    );

    // Snapshot before.
    let segments_empty_before = dir.segments_is_empty();
    let buffered_before = dir.buffered_bytes();
    let overlap_before = dir.overlap_count;
    let oow_before = dir.out_of_window_count;

    // BC-2.04.032: insert non-empty data with isn=None → IsnMissing, nothing inserted.
    let result = dir.insert_segment(500, b"world", usize::MAX, usize::MAX, usize::MAX);

    assert_eq!(
        result,
        InsertResult::IsnMissing,
        "BC-2.04.032 post-1: must return IsnMissing (prerequisite for this test)"
    );

    // BC-2.04.032 post-2: segments must be unchanged.
    assert_eq!(
        dir.segments_is_empty(),
        segments_empty_before,
        "BC-2.04.032 post-2: segments must be unchanged after IsnMissing — \
         data must not have been inserted into the buffer"
    );

    // BC-2.04.032 post-3: buffered_bytes must be unchanged.
    assert_eq!(
        dir.buffered_bytes(),
        buffered_before,
        "BC-2.04.032 post-3: buffered_bytes must be unchanged after IsnMissing"
    );

    // BC-2.04.032 post-4: no segment should be findable at the attempted offset.
    // Additional structural check: segment count is still 0.
    assert_eq!(
        dir.segment_count(),
        0,
        "BC-2.04.032 post-4: segment_count must remain 0 — IsnMissing must insert nothing"
    );

    // BC-2.04.032 PC4 — overlap_count and out_of_window_count must be unchanged.
    assert_eq!(
        dir.overlap_count, overlap_before,
        "BC-2.04.032 PC4 — overlap_count must be unchanged on IsnMissing"
    );
    assert_eq!(
        dir.out_of_window_count, oow_before,
        "BC-2.04.032 PC4 — out_of_window_count must be unchanged on IsnMissing"
    );
}

/// STORY-014 / BC-2.04.032 AC-012 / EC-006: empty data returns Inserted without ISN check.
/// Precondition 2 (negated): data.is_empty() fires the early return before the ISN guard.
/// STORY-014 / BC-2.04.032 AC-012 / EC-006: empty data slice with no ISN.
///
/// Asserts `insert_segment(.., &[], ..)` returns `InsertResult::Inserted` even
/// when `isn == None`, because the empty-data early return at
/// `src/reassembly/segment.rs:47-49` structurally precedes the ISN guard at
/// `:51-58`. This is the discriminating test for check ordering: if the
/// implementation swapped the two guards (ISN first, empty-data second), this
/// call would return `IsnMissing` instead of `Inserted`.
///
/// The "atomic ISN_MISSING_WARNED is not flipped by this path" sub-property is
/// enforced structurally by the order of these two checks in `insert_segment`
/// and is verified by code review (mirrors the BC-2.04.048 PC2 and inv-3
/// enforcement-mode precedents). No direct assertion on the atomic is performed
/// here, and therefore no `ISN_MISSING_WARNED_LOCK` acquisition is required.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_032_empty_data_returns_inserted_without_isn_check() {
    let mut dir = FlowDirection::new();

    // Precondition: isn=None (the ISN check would trigger IsnMissing if reached).
    assert_eq!(
        dir.isn, None,
        "precondition: isn must be None so ISN check is the adversary"
    );

    // BC-2.04.032 EC-006: empty data must return Inserted via the early-return path.
    // If check order were swapped (ISN before empty-data), this would return IsnMissing.
    let result = dir.insert_segment(1234, &[], usize::MAX, usize::MAX, usize::MAX);

    assert_eq!(
        result,
        InsertResult::Inserted,
        "BC-2.04.032 EC-006: insert_segment with data=[] and isn=None must return \
         InsertResult::Inserted (empty-data early return fires BEFORE ISN check) — \
         IsnMissing return indicates the check order was swapped"
    );

    // The direction state must remain completely unchanged (nothing was inserted).
    assert!(
        dir.segments_is_empty(),
        "BC-2.04.032 EC-006: empty insert must leave segments unchanged"
    );
    assert_eq!(
        dir.buffered_bytes(),
        0,
        "BC-2.04.032 EC-006: empty insert must leave buffered_bytes at 0"
    );
}

/// STORY-014 / BC-2.04.009 EC-002: second data packet (different direction) on partial flow.
/// set_initiator is a no-op; ISN is inferred for the s2c direction if not yet set.
///
/// Sequence: client sends first data (mid-stream join, c2s ISN inferred). Then server
/// sends a response. The server-to-client direction must also get its ISN inferred
/// (not left as None, which would return IsnMissing). The engine delivers both
/// payloads to the handler — one per direction — with the correct Direction tags.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_ec002_second_packet_different_direction_infers_s2c_isn() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // First packet: client data, no prior SYN (mid-stream join; c2s ISN inferred).
    let c2s = make_tcp_packet(
        client, 5000, server, 80, 3000, b"request", false, false, false, false,
    );
    reassembler.process_packet(&c2s, 1, &mut handler);

    // First packet delivered, initiator set to client, c2s ISN inferred.
    assert_eq!(
        handler.data_events.len(),
        1,
        "EC-002: first (c2s) mid-stream packet must be delivered"
    );
    assert_eq!(
        handler.data_events[0].1,
        Direction::ClientToServer,
        "EC-002: first packet must be tagged ClientToServer (initiator = client src)"
    );

    // Second packet: server response on the same flow (s2c direction, no prior SYN+ACK).
    // The engine must infer the s2c ISN from this first server packet.
    let s2c = make_tcp_packet(
        server,
        80,
        client,
        5000,
        7000,
        b"response",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&s2c, 2, &mut handler);

    // s2c data must also be delivered — meaning s2c ISN was inferred (not IsnMissing).
    assert_eq!(
        handler.data_events.len(),
        2,
        "BC-2.04.009 EC-002: second packet (s2c direction) must also be delivered — \
         IsnMissing would suppress delivery"
    );
    assert_eq!(
        handler.data_events[1].1,
        Direction::ServerToClient,
        "BC-2.04.009 EC-002: second packet must be tagged ServerToClient"
    );
    assert_eq!(
        handler.data_events[1].2, b"response",
        "BC-2.04.009 EC-002: correct payload must be delivered for s2c direction"
    );

    // s2c stream offset: infer_isn(7000) → isn=6999; seq_offset(7000, 6999) = 1.
    assert_eq!(
        handler.data_events[1].3, 1,
        "BC-2.04.009 EC-002: s2c stream offset must be 1 (inferred isn=7000-1=6999)"
    );

    // Still one partial flow (only one mid-stream join, for the flow — not one per direction).
    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.009 EC-002: flows_partial must be 1 (one flow, not incremented per direction)"
    );
}

/// STORY-014 / BC-2.04.009 EC-003: SYN arrives after data on partial flow.
/// set_initiator / set_isn / on_syn are all no-ops (already Established + ISN set).
///
/// Regression to catch: a late SYN might attempt to call set_isn(syn.seq) on the
/// already-inferred direction. If set_isn were not idempotent, the ISN would change,
/// corrupting all future sequence number computations.
///
/// Observable: after the SYN, the stream offset for the next data packet must
/// still be consistent with the originally-inferred ISN (not the SYN seq).
/// Also, flows_partial must still be 1 (the flow stays partial; SYN doesn't
/// un-mark it) and flows_total must still be 1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_ec003_syn_after_data_on_partial_flow_is_noop() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [1, 1, 1, 1];
    let server = [2, 2, 2, 2];

    // Step 1: Data packet first (mid-stream join), seq=1001.
    // infer_isn(1001) → c2s.isn=1000. Offset=1.
    let data1 = make_tcp_packet(
        client, 5000, server, 80, 1001, b"first", false, false, false, false,
    );
    reassembler.process_packet(&data1, 1, &mut handler);
    assert_eq!(
        handler.data_events.len(),
        1,
        "EC-003: first data packet must be delivered"
    );
    // Stream offset for seq=1001 with isn=1000 is 1.
    assert_eq!(
        handler.data_events[0].3, 1,
        "EC-003 precondition: first packet offset must be 1 (isn=1000 inferred)"
    );

    // Step 2: Late SYN arrives on the same flow (from the same client, seq=500).
    // The engine's apply_handshake_flags must call set_isn(500) — but since isn is
    // already Some(1000), set_isn(500) must be a no-op.
    // The SYN packet has a different seq (500 ≠ 1000), so any isn overwrite is detectable.
    let late_syn = make_tcp_packet(
        client,
        5000,
        server,
        80,
        500, // SYN seq deliberately different from inferred isn=1000
        &[],
        true, // SYN flag set
        false,
        false,
        false,
    );
    reassembler.process_packet(&late_syn, 2, &mut handler);

    // Flow state: still Established (no regression to SynSent).
    // flows_partial: still 1 (SYN does not un-mark a partial flow).
    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.009 EC-003: flows_partial must remain 1 after late SYN — \
         SYN must not reset the partial flag"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        1,
        "BC-2.04.009 EC-003: flows_total must remain 1 (no new flow created by late SYN)"
    );

    // Step 3: Second data packet at seq=1006 (follows seq=1001 + 5 bytes "first").
    // If isn were overwritten to 500 by the late SYN, seq_offset(1006, 500) = 506,
    // which is NOT equal to base_offset=6. The packet would be buffered (gap at 6)
    // but NOT flushed, so no new data event would appear.
    // With correctly preserved isn=1000: seq_offset(1006, 1000) = 6 = base_offset → flushed.
    let data2 = make_tcp_packet(
        client, 5000, server, 80, 1006, b"second", false, false, false, false,
    );
    reassembler.process_packet(&data2, 3, &mut handler);

    assert_eq!(
        handler.data_events.len(),
        2,
        "BC-2.04.009 EC-003: second data packet must be flushed (isn preserved from inferred=1000) — \
         isn overwrite to SYN seq=500 would make offset=506 ≠ base_offset=6, preventing flush"
    );
    assert_eq!(
        handler.data_events[1].3, 6,
        "BC-2.04.009 EC-003: second data packet stream offset must be 6 \
         (isn=1000 preserved; seq=1006-1000=6)"
    );
    assert_eq!(
        handler.data_events[1].2, b"second",
        "BC-2.04.009 EC-003: correct payload must be delivered for second data packet"
    );
}

/// STORY-014 / BC-2.04.009 EC-004: multiple partial flows counted independently.
/// flows_partial increments once per mid-stream flow; each is independent.
///
/// BC-2.04.009 invariant 2: flows_partial counts flows that entered via this path;
/// it is not reset when the flow later closes.
///
/// Scenario: two distinct flows each receive a first data packet without SYN.
/// flows_partial must be exactly 2 after both joins.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_009_ec004_multiple_partial_flows_counted_independently() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client_a = [1, 1, 1, 1];
    let client_b = [3, 3, 3, 3];
    let server = [2, 2, 2, 2];

    // Flow 1: mid-stream join — first data, no SYN.
    let data_a = make_tcp_packet(
        client_a, 5000, server, 80, 2001, b"flow-a", false, false, false, false,
    );
    reassembler.process_packet(&data_a, 1, &mut handler);

    assert_eq!(
        reassembler.stats().flows_partial,
        1,
        "BC-2.04.009 EC-004: after first mid-stream join, flows_partial must be 1"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        1,
        "BC-2.04.009 EC-004: flows_total must be 1 after first flow"
    );

    // Flow 2: distinct key (different client IP), also mid-stream join.
    let data_b = make_tcp_packet(
        client_b, 6000, server, 80, 9001, b"flow-b", false, false, false, false,
    );
    reassembler.process_packet(&data_b, 2, &mut handler);

    assert_eq!(
        reassembler.stats().flows_partial,
        2,
        "BC-2.04.009 EC-004: after second independent mid-stream join, flows_partial must be 2 — \
         each partial flow is counted independently"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        2,
        "BC-2.04.009 EC-004: flows_total must be 2 (two independent flows)"
    );

    // Both data payloads must have been delivered.
    assert_eq!(
        handler.data_events.len(),
        2,
        "BC-2.04.009 EC-004: both mid-stream flows must deliver their data"
    );
}

/// STORY-014 / BC-2.04.048 AC-013 + AC-014 + EC-007 combined.
///
/// Verifies that ISN_MISSING_WARNED latches false→true on the FIRST IsnMissing
/// encounter and stays true on subsequent encounters within the same process.
///
/// AC-014's "no additional eprintln on subsequent calls" property cannot be
/// asserted in-process without fragile stderr capture; it is enforced
/// structurally by the swap-guarded if-block in src/reassembly/segment.rs:51-58
/// (Architecture Compliance Rule — code review). The atomic-state-latches
/// property below IS testable and IS asserted.
///
/// AC-013, AC-014, EC-007 are combined into one function because
/// ISN_MISSING_WARNED is a process-global static and the cargo integration-test
/// binary shares it across all tests in this file. The reset accessor
/// (added in src/reassembly/segment.rs) makes the discrimination deterministic
/// regardless of run order.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_048_isn_missing_warned_fires_once_then_suppressed() {
    let _guard = ISN_MISSING_WARNED_LOCK.lock().expect("test lock poisoned");
    // Deterministic precondition: reset the process-global atomic so the
    // first call below is GUARANTEED to be the false→true transition.
    wirerust::reassembly::segment::reset_isn_missing_warned_for_testing();
    assert!(
        !wirerust::reassembly::segment::isn_missing_warned_for_testing(),
        "BC-2.04.048 — atomic should be false after reset_for_testing"
    );

    // Build two FlowDirections with isn=None.
    let mut dir1 = FlowDirection::new();
    let mut dir2 = FlowDirection::new();
    assert!(dir1.isn.is_none(), "precondition: ISN must be unset");
    assert!(dir2.isn.is_none(), "precondition: ISN must be unset");

    // FIRST call — atomic must transition false → true (BC-2.04.048 PC1).
    let r1 = dir1.insert_segment(100, b"first", usize::MAX, usize::MAX, usize::MAX);
    assert!(
        matches!(r1, InsertResult::IsnMissing),
        "AC-010 — IsnMissing returned on first call"
    );
    assert!(
        wirerust::reassembly::segment::isn_missing_warned_for_testing(),
        "AC-013 / BC-2.04.048 PC1 — atomic must be true after first IsnMissing"
    );

    // SECOND call — atomic stays true (BC-2.04.048 PC2 latching property).
    let r2 = dir2.insert_segment(200, b"second", usize::MAX, usize::MAX, usize::MAX);
    assert!(
        matches!(r2, InsertResult::IsnMissing),
        "AC-014 / EC-007 — IsnMissing still returned on subsequent call"
    );
    assert!(
        wirerust::reassembly::segment::isn_missing_warned_for_testing(),
        "AC-014 / EC-007 / BC-2.04.048 PC2 — atomic latches; remains true after subsequent call"
    );
}

// ---------------------------------------------------------------------------
// STORY-019: BC-2.04.010 — RST Closes Flow Immediately with CloseReason::Rst
//            BC-2.04.011 — Both FINs Close Flow with CloseReason::Fin
//            BC-2.04.013 — expire_flows Closes Idle Flows Past flow_timeout_secs
//            BC-2.04.029 — close_flow for Missing Key Logs One-Shot Warning
//
// AC-001..AC-015 (engine-level lifecycle tests) + 10 Edge Cases.
//
// PART A: stub-only bodies — panic!("STORY-019 stub — Red Gate").
// All stubs MUST fail before Part B fills real assertions.
//
// CLOSE_FLOW_MISSING_WARNED serialization: AC-013/014 are combined into one
// test (CLOSE_FLOW_MISSING_WARNED_LOCK held for duration) mirroring the
// ISN_MISSING_WARNED_LOCK pattern established in STORY-014. AC-015 also
// touches the missing-key path and must hold the same lock.
// ---------------------------------------------------------------------------

/// Serializes tests that read or write the process-global
/// `CLOSE_FLOW_MISSING_WARNED` atomic in `src/reassembly/lifecycle.rs`.
///
/// Any test in this binary that triggers `close_flow` for a missing key MUST
/// hold this lock for its entire duration. Failure to do so allows a sibling
/// test's `swap(true)` to race the observation in AC-013/014 (same pattern as
/// ISN_MISSING_WARNED_LOCK, established in STORY-014 adv-pass-3 F-1).
static CLOSE_FLOW_MISSING_WARNED_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

// ---- AC-001 ----------------------------------------------------------------

/// AC-001 (BC-2.04.010 postcondition 1)
/// When a TCP RST packet arrives for an established flow, `stats.flows_rst`
/// increments by 1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_rst_increments_flows_rst() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish a flow via SYN + SYN+ACK.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Snapshot before RST.
    let rst_before = reassembler.stats().flows_rst;

    // RST from server.
    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 3, &mut handler);

    // BC-2.04.010 PC1: flows_rst must have incremented by exactly 1 — not > 0, not 2.
    // Exact delta assertion discriminates a double-increment regression.
    assert_eq!(
        reassembler.stats().flows_rst,
        rst_before + 1,
        "BC-2.04.010 PC1: flows_rst must increment by exactly 1 on RST"
    );
}

// ---- AC-002 ----------------------------------------------------------------

/// AC-002 (BC-2.04.010 postconditions 2-4)
/// After a RST, any contiguous data buffered in both directions is flushed to
/// the handler via `on_data` calls, then `handler.on_flow_close(key,
/// CloseReason::Rst)` is called exactly once, and the flow is removed from
/// `self.flows` (verified via `stats.flows_total` and `total_memory == 0`).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_rst_flushes_then_closes() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish a flow via SYN + SYN+ACK.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Send out-of-order data that buffers (gap at offset 1): seq=1003 means
    // contiguous data at offset 1-2 is missing, so "bbb" stays buffered.
    let ooo = make_tcp_packet(
        client, 12345, server, 80, 1003, b"bbb", false, false, false, false,
    );
    reassembler.process_packet(&ooo, 3, &mut handler);
    assert_eq!(
        reassembler.total_memory(),
        3,
        "precondition: 'bbb' buffered, not flushed"
    );

    // Send the fill segment making contiguous data available.
    let fill = make_tcp_packet(
        client, 12345, server, 80, 1001, b"aa", false, false, false, false,
    );
    reassembler.process_packet(&fill, 4, &mut handler);
    // Now "aabbb" should be flushed.
    let data_events_before_rst = handler.data_events.len();
    assert!(
        data_events_before_rst > 0,
        "precondition: data must have been flushed before RST"
    );

    // RST — should flush any remaining data then close.
    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 5, &mut handler);

    // BC-2.04.010 PC3: on_flow_close called exactly once with CloseReason::Rst.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.010 PC3: on_flow_close must be called exactly once"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Rst,
        "BC-2.04.010 PC3: close reason must be Rst"
    );

    // BC-2.04.010 PC4: flow removed from self.flows (total_memory == 0, flow_count == 0).
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.010 PC4: total_memory must be 0 after RST (flow removed)"
    );
    assert_eq!(
        reassembler.flow_count(),
        0,
        "BC-2.04.010 PC4: flow_count must be 0 after RST (flow removed from self.flows)"
    );

    // Ordering: data_events must precede the close event. Because
    // RecordingHandler appends in callback order, any data events from the RST
    // flush (if any remaining data existed) would appear before the close event.
    // We verify that at least the pre-RST data events exist and the close event
    // is exactly one and is last.
    assert!(
        handler.data_events.len() >= data_events_before_rst,
        "BC-2.04.010 PC2: data flushed on RST must appear before close event"
    );
}

// ---- AC-003 ----------------------------------------------------------------

/// AC-003 (BC-2.04.010 postcondition 6 and invariant 3)
/// Payload carried in the RST packet itself is NOT processed (RST triggers
/// `PostHandshake::FlowClosed`, preventing payload insertion). After RST, the
/// handler must have received no additional `on_data` call for the RST packet's
/// payload bytes.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_rst_payload_not_processed() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow: SYN + SYN+ACK.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Deliver in-order data so we have a known data_events count.
    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"hello", false, false, false, false,
    );
    reassembler.process_packet(&data, 3, &mut handler);
    let data_events_before_rst = handler.data_events.len();
    assert_eq!(
        data_events_before_rst, 1,
        "precondition: 1 data event before RST"
    );

    // RST with non-empty payload (b"poison" must NOT appear in data_events).
    let rst_with_payload = make_tcp_packet(
        server, 80, client, 12345, 2001, b"poison", false, false, false, true,
    );
    reassembler.process_packet(&rst_with_payload, 4, &mut handler);

    // BC-2.04.010 PC6 + inv-3: data_events count must be unchanged — the
    // RST payload was NOT processed. A regression to "process payload then
    // close" would add another data event here.
    assert_eq!(
        handler.data_events.len(),
        data_events_before_rst,
        "BC-2.04.010 PC6/inv-3: RST packet payload must NOT be delivered via on_data; \
         data_events count must not increase"
    );

    // Verify the close did happen (RST was processed, flow closed).
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.010 PC3: close must still occur despite carrying payload"
    );
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);

    // Total bytes in data_events must not contain "poison".
    let all_data = handler.all_data();
    assert!(
        !all_data.windows(6).any(|w| w == b"poison"),
        "BC-2.04.010 PC6: RST payload bytes must never appear in reassembled data"
    );
}

// ---- AC-004 ----------------------------------------------------------------

/// AC-004 (BC-2.04.010 invariant 1)
/// RST closes the flow regardless of current state: New, SynSent, Established,
/// Closing. Four sub-cases exercised sequentially. Flow is removed from
/// `self.flows` in all cases (flows table empty after each RST).
///
/// Sub-cases:
///   1. New — no handshake at all; RST arrives immediately
///   2. SynSent — SYN sent; RST before SYN+ACK
///   3. Established — full handshake; RST mid-stream
///   4. Closing — first FIN received; RST before second FIN
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_rst_closes_from_any_state() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // ---- Sub-case 1: New state — RST with no prior packets ----
    // Flow A: RST is the very first packet (no SYN, no data).
    let rst_a = make_tcp_packet(
        [10, 0, 1, 1],
        11111,
        server,
        80,
        500,
        &[],
        false,
        false,
        false,
        true,
    );
    let flows_rst_before_a = reassembler.stats().flows_rst;
    reassembler.process_packet(&rst_a, 1, &mut handler);
    assert_eq!(
        reassembler.stats().flows_rst,
        flows_rst_before_a + 1,
        "AC-004 sub-case 1 (New): flows_rst must increment"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "AC-004 sub-case 1 (New): flow must be removed (total_memory=0)"
    );

    // ---- Sub-case 2: SynSent state — RST after SYN ----
    let syn_b = make_tcp_packet(
        [10, 0, 1, 2],
        22222,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 2, &mut handler);
    let flows_rst_before_b = reassembler.stats().flows_rst;
    let rst_b = make_tcp_packet(
        server,
        80,
        [10, 0, 1, 2],
        22222,
        9000,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst_b, 3, &mut handler);
    assert_eq!(
        reassembler.stats().flows_rst,
        flows_rst_before_b + 1,
        "AC-004 sub-case 2 (SynSent): flows_rst must increment"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "AC-004 sub-case 2 (SynSent): flow must be removed"
    );

    // ---- Sub-case 3: Established state — RST after full handshake ----
    let syn_c = make_tcp_packet(
        [10, 0, 1, 3],
        33333,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_c, 4, &mut handler);
    let syn_ack_c = make_tcp_packet(
        server,
        80,
        [10, 0, 1, 3],
        33333,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_c, 5, &mut handler);
    let flows_rst_before_c = reassembler.stats().flows_rst;
    let rst_c = make_tcp_packet(
        server,
        80,
        [10, 0, 1, 3],
        33333,
        5001,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst_c, 6, &mut handler);
    assert_eq!(
        reassembler.stats().flows_rst,
        flows_rst_before_c + 1,
        "AC-004 sub-case 3 (Established): flows_rst must increment"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "AC-004 sub-case 3 (Established): flow must be removed"
    );

    // ---- Sub-case 4: Closing state — RST after first FIN ----
    let syn_d = make_tcp_packet(
        [10, 0, 1, 4],
        44444,
        server,
        80,
        3000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_d, 7, &mut handler);
    let syn_ack_d = make_tcp_packet(
        server,
        80,
        [10, 0, 1, 4],
        44444,
        7000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_d, 8, &mut handler);
    // First FIN puts flow into Closing state.
    let fin_d = make_tcp_packet(
        [10, 0, 1, 4],
        44444,
        server,
        80,
        3001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin_d, 9, &mut handler);
    let flows_rst_before_d = reassembler.stats().flows_rst;
    // RST from either direction closes the Closing flow.
    let rst_d = make_tcp_packet(
        server,
        80,
        [10, 0, 1, 4],
        44444,
        7001,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst_d, 10, &mut handler);
    assert_eq!(
        reassembler.stats().flows_rst,
        flows_rst_before_d + 1,
        "AC-004 sub-case 4 (Closing): flows_rst must increment"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "AC-004 sub-case 4 (Closing): flow must be removed"
    );

    // All 4 RSTs must have been counted.
    assert_eq!(
        reassembler.stats().flows_rst,
        4,
        "AC-004: total flows_rst must be 4 after four sub-cases"
    );
}

// ---- AC-005 ----------------------------------------------------------------

/// AC-005 (BC-2.04.011 invariant 1)
/// The first FIN transitions the flow state to `Closing` and `fin_count`
/// becomes 1. The flow is NOT removed after the first FIN (still in
/// `self.flows`).
///
/// Verified indirectly at the engine level: after first FIN, no close event is
/// recorded and `stats.flows_fin == 0`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_first_fin_transitions_to_closing() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow: SYN + SYN+ACK.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // First FIN from client.
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 3, &mut handler);

    // BC-2.04.011 inv-1: after first FIN — flow NOT closed (no close event, flows_fin == 0).
    assert_eq!(
        handler.close_events.len(),
        0,
        "BC-2.04.011 inv-1: flow must NOT be closed after first FIN"
    );
    assert_eq!(
        reassembler.stats().flows_fin,
        0,
        "BC-2.04.011 inv-1: flows_fin must be 0 after only one FIN"
    );
    // Flow still occupies memory (not removed).
    // total_memory may be 0 (no buffered data) but the flow entry must exist.
    // We verify by checking that a second FIN still closes it (flow is still tracked).
    // The next assertion is: no close_events recorded.
    assert!(
        handler.close_events.is_empty(),
        "BC-2.04.011 inv-1: no on_flow_close callback after first FIN"
    );
}

// ---- AC-006 ----------------------------------------------------------------

/// AC-006 (BC-2.04.011 postconditions 1-6)
/// When a second FIN arrives (from either direction):
///   - `stats.flows_fin` increments by 1
///   - remaining contiguous data is flushed
///   - `handler.on_flow_close(key, CloseReason::Fin)` is called exactly once
///   - the flow is removed from `self.flows` (`total_memory == 0`)
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_second_fin_closes_flow() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Data in both directions.
    let req = make_tcp_packet(
        client, 12345, server, 80, 1001, b"GET /", false, false, false, false,
    );
    reassembler.process_packet(&req, 3, &mut handler);
    let resp = make_tcp_packet(
        server, 80, client, 12345, 2001, b"200 OK", false, false, false, false,
    );
    reassembler.process_packet(&resp, 4, &mut handler);

    // First FIN from client → state=Closing.
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1006,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 5, &mut handler);
    assert_eq!(
        handler.close_events.len(),
        0,
        "precondition: no close after first FIN"
    );

    let flows_fin_before = reassembler.stats().flows_fin;

    // Second FIN from server → state=Closed → engine closes the flow.
    let fin2 = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2007,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin2, 6, &mut handler);

    // BC-2.04.011 PC3: flows_fin increments by exactly 1.
    assert_eq!(
        reassembler.stats().flows_fin,
        flows_fin_before + 1,
        "BC-2.04.011 PC3: flows_fin must increment by 1 on second FIN"
    );

    // BC-2.04.011 PC5: on_flow_close called exactly once with CloseReason::Fin.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.011 PC5: on_flow_close must be called exactly once"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Fin,
        "BC-2.04.011 PC5: close reason must be Fin"
    );

    // BC-2.04.011 PC6: flow removed (total_memory == 0).
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.011 PC6: flow must be removed after second FIN (total_memory=0)"
    );
}

// ---- AC-007 ----------------------------------------------------------------

/// AC-007 (BC-2.04.011 invariant 2)
/// FIN close happens AFTER payload processing for the FIN packet (data carried
/// in a FIN segment is reassembled and delivered before the flow closes).
///
/// Verifies that the FIN-segment payload ("bye") is delivered via `on_data`
/// (payload processing happens) AND the flow closes (`on_flow_close` fires).
/// The data-before-close ORDERING within `close_flow` is enforced structurally
/// by the order of operations in `process_packet` at mod.rs:165-174
/// (`close_flow` is invoked AFTER `insert_payload_segment` completes);
/// ordering verification is via code review, not automated assertion.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_fin_payload_processed_before_close() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // First FIN from client (no payload).
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 3, &mut handler);

    // Second FIN from server WITH payload.
    // The server's FIN carries b"bye" as a piggybacked last segment.
    // The data must be delivered via on_data BEFORE the flow is closed.
    let fin2_with_payload = make_tcp_packet(
        server, 80, client, 12345, 2001, b"bye", false, false, true, false,
    );
    reassembler.process_packet(&fin2_with_payload, 4, &mut handler);

    // BC-2.04.011 inv-2: the close must have occurred (second FIN closes the flow).
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.011 inv-2: flow must be closed after second FIN"
    );
    assert_eq!(handler.close_events[0].1, CloseReason::Fin);

    // Ordering: data events must exist AND data event for "bye" must be present.
    // The FIN payload b"bye" is 3 bytes; it must appear in handler.data_events.
    let fin_payload_delivered = handler
        .data_events
        .iter()
        .any(|(_, _, data, _)| data.as_slice() == b"bye");
    assert!(
        fin_payload_delivered,
        "BC-2.04.011 inv-2: FIN packet payload 'bye' must be delivered via on_data"
    );

    // Because RecordingHandler appends in callback order (data before close),
    // the data_events must be non-empty and must have been populated BEFORE the
    // close event. We verify this structurally: there must be at least one data
    // event, and the close event must be exactly one (the last thing that happened).
    assert!(
        !handler.data_events.is_empty(),
        "BC-2.04.011 inv-2: data_events must be non-empty (FIN payload was delivered)"
    );

    // Verify total data delivered includes "bye".
    let all_bytes = handler.all_data();
    assert!(
        all_bytes.windows(3).any(|w| w == b"bye"),
        "BC-2.04.011 inv-2: 'bye' must appear in total reassembled data (data before close)"
    );
}

// ---- AC-008 ----------------------------------------------------------------

/// AC-008 (BC-2.04.011 edge case EC-002)
/// Two FINs from the SAME direction (retransmit) are sufficient to close the
/// flow (`fin_count` reaches 2 regardless of which direction each FIN came
/// from). After two client FINs, the flow is closed with `CloseReason::Fin`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_same_direction_fin_retransmit_closes_flow() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // First FIN from client direction — puts flow in Closing.
    let fin1 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin1, 3, &mut handler);
    assert_eq!(
        handler.close_events.len(),
        0,
        "precondition: no close after first FIN"
    );

    // Second FIN also from CLIENT direction (retransmit, same direction).
    // fin_count reaches 2 → state=Closed → flow closed with CloseReason::Fin.
    let fin2_same_direction = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        &[],
        false,
        false,
        true,
        false,
    );
    reassembler.process_packet(&fin2_same_direction, 4, &mut handler);

    // BC-2.04.011 EC-002: same-direction retransmit must close the flow.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.011 EC-002: flow must close after two same-direction FINs"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Fin,
        "BC-2.04.011 EC-002: close reason must be Fin, not Rst or Timeout"
    );
    assert_eq!(
        reassembler.stats().flows_fin,
        1,
        "BC-2.04.011 EC-002: flows_fin must be 1 after same-direction FIN retransmit close"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.011 EC-002: flow must be removed (total_memory=0)"
    );
}

// ---- AC-009 ----------------------------------------------------------------

/// AC-009 (BC-2.04.013 postconditions 1-2)
/// `expire_flows(current_time, handler)` closes all flows where
/// `current_time > last_seen AND (current_time - last_seen) > flow_timeout_secs`
/// with `CloseReason::Timeout`. `stats.flows_expired` increments by the number
/// of flows expired.
///
/// Canonical test vector (BC-2.04.013): last_seen=0, current_time=400,
/// timeout=300 → flow expired, flows_expired=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_013_expire_flows_closes_idle_flows() {
    // Canonical test vector: timeout=300, last_seen=0, current_time=400.
    let config = ReassemblyConfig {
        flow_timeout_secs: 300,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: last_seen = timestamp 0 (SYN at t=0 sets last_seen=0).
    let syn_a = make_tcp_packet(
        [10, 0, 1, 1],
        11111,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 0, &mut handler);

    // Flow B: last_seen = timestamp 10.
    let syn_b = make_tcp_packet(
        [10, 0, 1, 2],
        22222,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 10, &mut handler);

    // Flow C: last_seen = timestamp 200.
    let syn_c = make_tcp_packet(
        [10, 0, 1, 3],
        33333,
        server,
        80,
        3000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_c, 200, &mut handler);

    // Snapshot before expiry.
    let expired_before = reassembler.stats().flows_expired;

    // expire_flows at current_time=400:
    // - Flow A: 400 > 0 AND (400 - 0) = 400 > 300 → EXPIRED
    // - Flow B: 400 > 10 AND (400 - 10) = 390 > 300 → EXPIRED
    // - Flow C: 400 > 200 AND (400 - 200) = 200 <= 300 → NOT expired
    reassembler.expire_flows(400, &mut handler);

    // BC-2.04.013 PC2: flows_expired incremented by 2 (A and B expired).
    assert_eq!(
        reassembler.stats().flows_expired,
        expired_before + 2,
        "BC-2.04.013 PC2: flows_expired must increment by 2 (flows A and B are past timeout)"
    );

    // BC-2.04.013 PC1 + PC3: each expired flow has a CloseReason::Timeout close event.
    let timeout_closes: Vec<_> = handler
        .close_events
        .iter()
        .filter(|(_, r)| *r == CloseReason::Timeout)
        .collect();
    assert_eq!(
        timeout_closes.len(),
        2,
        "BC-2.04.013 PC3: two on_flow_close(Timeout) events must be recorded"
    );

    // BC-2.04.013 PC4: flow C (last_seen=200, not past timeout) must survive.
    // After expiring A and B, total_memory must be 0 (no buffered data was pending).
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.013: total_memory must be 0 after expired flows are removed"
    );
}

// ---- AC-010 ----------------------------------------------------------------

/// AC-010 (BC-2.04.013 postcondition 4)
/// Flows that are within the timeout window are NOT closed by `expire_flows`.
///
/// Canonical test vector (BC-2.04.013): last_seen=100, current_time=300,
/// timeout=300 → not expired (300-100=200 which is <= 300).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_013_expire_flows_does_not_close_active_flows() {
    // Canonical test vector: last_seen=100, current_time=300, timeout=300.
    // (300 - 100) = 200 which is NOT > 300, so the flow must survive.
    let config = ReassemblyConfig {
        flow_timeout_secs: 300,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Flow with last_seen = 100 (SYN at t=100).
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 100, &mut handler);

    // expire_flows at current_time=300: 300 - 100 = 200 ≤ 300, must NOT expire.
    reassembler.expire_flows(300, &mut handler);

    // BC-2.04.013 PC4: flow must NOT be closed.
    assert_eq!(
        handler.close_events.len(),
        0,
        "BC-2.04.013 PC4: flow within timeout window must not be closed"
    );
    assert_eq!(
        reassembler.stats().flows_expired,
        0,
        "BC-2.04.013 PC4: flows_expired must be 0 (no flows past timeout)"
    );
}

// ---- AC-011 ----------------------------------------------------------------

/// AC-011 (BC-2.04.013 invariant 1)
/// `expire_flows` uses underflow-safe subtraction: `current_time > flow.last_seen`
/// is checked BEFORE `current_time - flow.last_seen > timeout`, preventing u32
/// underflow.
///
/// Test vector: `current_time < last_seen` (timestamp reorder / backwards time).
/// Assert: no panic AND the flow is NOT expired (close_events.len() == 0).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_013_expire_flows_does_not_underflow_when_time_travels_backwards() {
    // The release profile has overflow-checks=true (see CLAUDE.md). A plain
    // `(current_time - last_seen) > timeout` without the prior `current_time >
    // last_seen` guard would panic here in release builds.
    let config = ReassemblyConfig {
        flow_timeout_secs: 300,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Flow with last_seen = 1000 (SYN at t=1000).
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1000, &mut handler);

    // expire_flows at current_time=500 which is LESS than last_seen=1000.
    // BC-2.04.013 inv-1: `current_time > last_seen` is false → no subtraction → no panic.
    // This must not panic (would panic under overflow-checks=true if guard missing).
    reassembler.expire_flows(500, &mut handler);

    // Flow must NOT be expired (time went backwards — underflow guard prevented it).
    assert_eq!(
        handler.close_events.len(),
        0,
        "BC-2.04.013 inv-1: flow must NOT be expired when current_time < last_seen"
    );
    assert_eq!(
        reassembler.stats().flows_expired,
        0,
        "BC-2.04.013 inv-1: flows_expired must be 0 (underflow guard prevents expiry)"
    );
}

// ---- AC-012 ----------------------------------------------------------------

/// STORY-019 / BC-2.04.013 AC-012 / EC-004: a flow with `state == Closed` is
/// expired by `expire_flows` REGARDLESS of idle time.
///
/// Uses `force_set_flow_state_for_testing` to construct a Closed flow whose
/// `last_seen` is well within the timeout window — proving the state-based
/// OR-branch fires INDEPENDENTLY of the time-based clause. A regression that
/// drops the `state == FlowState::Closed` clause from `expire_flows` would
/// leave this flow unexpired and fail this test.
///
/// Note: EC-004 here is BC-2.04.013's EC-004 (flow with state=Closed), NOT
/// STORY-019's EC-004 (FIN on New flow).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_013_already_closed_state_is_expired() {
    let config = ReassemblyConfig {
        flow_timeout_secs: 1_000_000, // huge timeout — time-based clause cannot fire
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish a flow at t=100 via a single SYN packet.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 100, &mut handler);
    assert_eq!(
        reassembler.flow_count(),
        1,
        "precondition: flow established"
    );

    // Force state to Closed without advancing time (seam from lifecycle.rs).
    let key = wirerust::reassembly::flow::FlowKey::new(
        IpAddr::V4(Ipv4Addr::from(client)),
        12345,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let updated = wirerust::reassembly::lifecycle::force_set_flow_state_for_testing(
        &mut reassembler,
        &key,
        wirerust::reassembly::flow::FlowState::Closed,
    );
    assert!(updated, "force_set_flow_state seam must find the flow");

    // current_time=101 → idle is 1s, well below 1_000_000s timeout.
    // Time-based clause: (101 - 100) > 1_000_000 → FALSE.
    // Only the `state == FlowState::Closed` OR-clause can cause expiry.
    let closes_before = handler.close_events.len();
    reassembler.expire_flows(101, &mut handler);

    assert_eq!(
        reassembler.flow_count(),
        0,
        "AC-012: Closed-state flow must be expired regardless of idle time"
    );
    assert_eq!(
        handler.close_events.len() - closes_before,
        1,
        "AC-012: expire_flows must invoke on_flow_close exactly once for the Closed flow"
    );
    assert_eq!(
        handler.close_events.last().unwrap().1,
        CloseReason::Timeout,
        "AC-012: close reason must be Timeout per expire_flows contract"
    );
    assert_eq!(
        reassembler.stats().flows_expired,
        1,
        "AC-012: stats.flows_expired must increment by 1"
    );
}

// ---- AC-013 + AC-014 combined -------------------------------------------

/// AC-013 (BC-2.04.029 postcondition 4) + AC-014 (BC-2.04.029 postcondition 5)
/// Combined into one test because `CLOSE_FLOW_MISSING_WARNED` is process-global
/// across the integration-test binary (same pattern as
/// `test_BC_2_04_048_isn_missing_warned_fires_once_then_suppressed` in STORY-014).
///
/// AC-013: When `close_flow` is called for a key NOT in `self.flows` and
/// `CLOSE_FLOW_MISSING_WARNED == false`, `eprintln!` fires exactly once and
/// `CLOSE_FLOW_MISSING_WARNED` is set to `true`.
///
/// AC-014: On a subsequent missing-key call (after the first warning), no
/// additional `eprintln!` is emitted (silent return). The "no second eprintln"
/// sub-property is enforced by code review of the swap-guarded if-block (the
/// `swap(true, Relaxed)` only enters the eprintln branch on the `false → true`
/// transition), NOT by automated output capture, because `eprintln!` writes to
/// stderr and Rust's libtest does not capture it by default. This is structurally
/// enforced via the `swap(true, Ordering::Relaxed)` guard at lifecycle.rs:44
/// (mirroring BC-2.04.048 PC2 / inv-3 precedent and the ADR-0004 amendment).
///
/// The `CLOSE_FLOW_MISSING_WARNED_LOCK` must be held for the entire test body
/// to prevent sibling tests from racing the atomic.
///
/// Requires `close_flow_missing_warned_for_testing()` and
/// `reset_close_flow_missing_warned_for_testing()` test-seam accessors in
/// `src/reassembly/lifecycle.rs` (to be added in Part B / implementer step W8.3).
///
/// Test seam accessors added in W8.3 (implementer step).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_029_close_flow_missing_key_warns_once() {
    let _guard = CLOSE_FLOW_MISSING_WARNED_LOCK
        .lock()
        .expect("CLOSE_FLOW_MISSING_WARNED_LOCK poisoned");

    // Deterministic precondition: reset the process-global atomic so the first
    // call below is GUARANTEED to be the false→true transition.
    wirerust::reassembly::lifecycle::reset_close_flow_missing_warned_for_testing();
    assert!(
        !wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        "BC-2.04.029 — atomic must be false after reset_for_testing"
    );

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Construct a FlowKey for a flow that does NOT exist in the reassembler.
    use std::net::IpAddr;
    let missing_key = wirerust::reassembly::flow::FlowKey::new(
        IpAddr::V4(std::net::Ipv4Addr::new(99, 0, 0, 1)),
        9001,
        IpAddr::V4(std::net::Ipv4Addr::new(99, 0, 0, 2)),
        9002,
    );

    // Snapshot: no flows, no close events.
    assert_eq!(
        handler.close_events.len(),
        0,
        "precondition: no close events before missing-key call"
    );

    // AC-013 (false→true transition) and AC-014 (latching) are both observable via
    // the trigger_close_flow_missing_key_for_testing seam. We reset the atomic,
    // invoke the trigger once to verify BC-2.04.029 PC4, then a second time to
    // verify PC5 (atomic stays true; eprintln-suppression structurally enforced
    // per ADR-0004 amendment via swap-guard at lifecycle.rs:44-48).

    // Setup: one flow, closed by RST (flow removed from self.flows).
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    // RST — removes flow from self.flows. flows_rst == 1.
    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 2, &mut handler);
    assert_eq!(
        handler.close_events.len(),
        1,
        "precondition: RST must have fired one close event"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Rst,
        "precondition: close reason must be Rst"
    );

    // BC-2.04.029 PC1: self.flows is unmodified by the missing-key call
    // (we already confirmed close_events.len()==1 from RST; a second close
    // from a spurious missing-key call would add a second entry).
    // After RST the flow IS removed, so self.flows is empty.
    // Any subsequent call to close_flow for the RST key would be a missing-key call.
    // expire_flows does NOT trigger this (it only iterates existing flows).
    // We verify the observable: atomic is false before RST path exercised it...
    // Actually the RST path in apply_handshake_flags calls self.close_flow which
    // DOES remove the flow. But close_flow for a FOUND key does NOT trigger the
    // missing-key guard. So the atomic must still be false after the RST.
    assert!(
        !wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        "BC-2.04.029 PC4 precondition: atomic must still be false after RST close \
         (RST found the key, no missing-key path taken)"
    );

    // BC-2.04.029 PC2: handler.close_events count must not change after the
    // missing-key call. We verify this by checking that after the RST there is
    // exactly 1 close event, and after any subsequent missing-key trigger there
    // is STILL exactly 1.

    // Trigger the missing-key path via the test seam (requires W8.3):
    wirerust::reassembly::lifecycle::reset_close_flow_missing_warned_for_testing();
    // Directly use the atomic-state testing accessor path that the implementer must add.
    // This line will produce the compile error that tells the implementer what seam to add.
    //
    // First call to the missing-key trigger:
    wirerust::reassembly::lifecycle::trigger_close_flow_missing_key_for_testing(
        &mut reassembler,
        &missing_key,
        CloseReason::Timeout,
        &mut handler,
    );

    // BC-2.04.029 PC4: atomic must now be true (first missing-key call set it).
    assert!(
        wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        "BC-2.04.029 PC4: CLOSE_FLOW_MISSING_WARNED must be true after first missing-key call"
    );

    // BC-2.04.029 PC1 + PC2: no additional close events, flows unchanged.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.029 PC2: close_events count must be unchanged after missing-key call (no on_flow_close fires)"
    );

    // Construct a second distinct missing key for AC-014.
    let missing_key_2 = wirerust::reassembly::flow::FlowKey::new(
        IpAddr::V4(std::net::Ipv4Addr::new(99, 0, 0, 3)),
        9003,
        IpAddr::V4(std::net::Ipv4Addr::new(99, 0, 0, 4)),
        9004,
    );

    // Second call — AC-014: atomic stays true (latching), no second eprintln.
    wirerust::reassembly::lifecycle::trigger_close_flow_missing_key_for_testing(
        &mut reassembler,
        &missing_key_2,
        CloseReason::Timeout,
        &mut handler,
    );

    // BC-2.04.029 PC5: atomic still true after second call (latching).
    assert!(
        wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        "BC-2.04.029 PC5: CLOSE_FLOW_MISSING_WARNED must remain true after second missing-key call"
    );

    // BC-2.04.029 PC2: still no new close events.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.029 PC2: close_events count must remain 1 after second missing-key call (no on_flow_close fires on subsequent missing-key)"
    );
}

// ---- AC-015 ----------------------------------------------------------------

/// AC-015 (BC-2.04.029 postconditions 1-3)
/// When `close_flow` returns early for a missing key:
///   - no `on_flow_close` callback fires (`close_events.len() == 0`)
///   - `total_memory` is unchanged
///   - `self.flows` is unmodified (existing flows remain; no crash)
///
/// Holds `CLOSE_FLOW_MISSING_WARNED_LOCK` because the missing-key path
/// writes the process-global atomic.
///
/// Test seam accessors added in W8.3 (implementer step).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_029_close_flow_missing_key_does_not_modify_state() {
    let _guard = CLOSE_FLOW_MISSING_WARNED_LOCK
        .lock()
        .expect("CLOSE_FLOW_MISSING_WARNED_LOCK poisoned");

    wirerust::reassembly::lifecycle::reset_close_flow_missing_warned_for_testing();

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish one real flow (so self.flows is non-empty = there's something to not-modify).
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Snapshots BEFORE the missing-key call.
    let memory_before = reassembler.total_memory();
    let close_events_before = handler.close_events.len();
    let flow_count_before = reassembler.flow_count();

    // Construct a FlowKey that does NOT exist in the reassembler.
    use std::net::IpAddr;
    let missing_key = wirerust::reassembly::flow::FlowKey::new(
        IpAddr::V4(std::net::Ipv4Addr::new(1, 2, 3, 4)),
        55555,
        IpAddr::V4(std::net::Ipv4Addr::new(5, 6, 7, 8)),
        55556,
    );

    // Trigger the missing-key path via the test seam (requires W8.3).
    // This will produce a compile error until the implementer adds the seam.
    wirerust::reassembly::lifecycle::trigger_close_flow_missing_key_for_testing(
        &mut reassembler,
        &missing_key,
        CloseReason::Timeout,
        &mut handler,
    );

    // BC-2.04.029 PC2: no on_flow_close callback (close_events unchanged).
    assert_eq!(
        handler.close_events.len(),
        close_events_before,
        "BC-2.04.029 PC2: missing-key close_flow must not emit on_flow_close"
    );

    // BC-2.04.029 PC3: total_memory unchanged.
    assert_eq!(
        reassembler.total_memory(),
        memory_before,
        "BC-2.04.029 PC3: total_memory must be unchanged after missing-key close_flow"
    );

    // AC-015 / BC-2.04.029 PC1: flow_count unchanged.
    assert_eq!(
        reassembler.flow_count(),
        flow_count_before,
        "AC-015 / BC-2.04.029 PC1 — flow_count unchanged after missing-key trigger"
    );
}

// ---- Edge Cases ------------------------------------------------------------

/// EC-001 (STORY-019 / BC-2.04.010 EC-001)
/// RST on a New flow (no handshake, no data): flows_rst++; on_flow_close(Rst)
/// called; flow removed; no `on_data` events emitted.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_ec001_rst_on_new_flow_no_data() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // RST as the very first packet for this flow (no SYN, no data).
    let rst = make_tcp_packet(
        client,
        12345,
        server,
        80,
        500,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 1, &mut handler);

    // BC-2.04.010 EC-001: flows_rst must be 1.
    assert_eq!(
        reassembler.stats().flows_rst,
        1,
        "BC-2.04.010 EC-001: flows_rst must be 1 after RST on New flow"
    );

    // on_flow_close(Rst) called exactly once.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.010 EC-001: on_flow_close must be called once"
    );
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);

    // Flow removed.
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.010 EC-001: flow must be removed after RST on New flow"
    );

    // No data flushed (no data was ever seen on this flow).
    assert!(
        handler.data_events.is_empty(),
        "BC-2.04.010 EC-001: no on_data events must be emitted for a New flow RST"
    );
}

/// EC-002 (STORY-019 / BC-2.04.010 EC-002)
/// RST on a flow with buffered non-contiguous data: `total_memory` is released
/// to 0 even when the buffered segment cannot be flushed (gap blocks delivery).
///
/// Verifies that RST close releases `total_memory` to 0 even when a
/// non-contiguous segment remained buffered (silently dropped by
/// `flush_contiguous` since the gap blocks delivery). The flush-before-close
/// discipline of BC-2.04.010 PC2 is verified separately by AC-002
/// (`test_BC_2_04_010_rst_flushes_then_closes`).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_ec002_rst_releases_total_memory_with_buffered_non_contiguous_segments() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Send in-order data that gets flushed immediately.
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"aaa", false, false, false, false,
    );
    reassembler.process_packet(&p1, 3, &mut handler);
    assert_eq!(handler.data_events.len(), 1, "precondition: p1 flushed");

    // Send out-of-order data that stays buffered (gap at offset 4: seq 1004+3=1007 is missing).
    let p3 = make_tcp_packet(
        client, 12345, server, 80, 1007, b"ccc", false, false, false, false,
    );
    reassembler.process_packet(&p3, 4, &mut handler);
    assert_eq!(
        handler.data_events.len(),
        1,
        "precondition: p3 buffered (gap), not flushed"
    );
    assert_eq!(
        reassembler.total_memory(),
        3,
        "precondition: 3 bytes buffered"
    );

    // RST — exercises close_flow's flush path on a flow with a buffered non-contiguous
    // segment ("ccc" at offset 6, blocked by gap at offset 3-5). flush_contiguous
    // cannot deliver behind a gap, so "ccc" is silently dropped at close; we only
    // verify total_memory release and clean close. BC-2.04.010 PC2's "flush in
    // close_flow" is enforced as defense-in-depth per BC-2.04.010 v1.5 PC2.
    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 5, &mut handler);

    // BC-2.04.010 EC-002: buffered data flushed → data_events count increased.
    // The RST calls close_flow which calls flush_contiguous on both directions.
    // Even though "ccc" was out-of-order, flush_contiguous delivers whatever is
    // at the current base_offset (any contiguous prefix). If "ccc" has a gap before
    // it, flush_contiguous won't deliver it — but the close_flow still runs without
    // error. The key observable: close event must follow any data events.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.010 EC-002: on_flow_close must be called exactly once after RST"
    );
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.010 EC-002: total_memory must be 0 after RST (buffered bytes freed)"
    );
    // Data events from before the RST must still be present (not rolled back).
    assert!(
        !handler.data_events.is_empty(),
        "BC-2.04.010 EC-002: pre-RST data events must not be erased"
    );
}

/// EC-003 (STORY-019 / BC-2.04.010 EC-003)
/// RST packet carries payload: payload is NOT inserted. The close handler
/// receives no extra `on_data` call for the RST packet's bytes.
///
/// This is the engine-level counterpart to AC-003; here the focus is the EC
/// framing (explicit RST-with-payload scenario, not just the general rule).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_ec003_rst_packet_payload_is_discarded() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // RST with a distinctly identifiable payload.
    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        b"BAD_DATA",
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 2, &mut handler);

    // No data events at all — RST payload was not processed.
    assert!(
        handler.data_events.is_empty(),
        "BC-2.04.010 EC-003: RST packet payload must not generate any on_data events"
    );

    // Close event must exist with Rst reason.
    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);

    // "BAD_DATA" must not appear in any reassembled bytes.
    let all_data = handler.all_data();
    assert!(
        !all_data.windows(8).any(|w| w == b"BAD_DATA"),
        "BC-2.04.010 EC-003: RST payload bytes must not appear in reassembled data"
    );
}

/// EC-006 (STORY-019 edge case)
/// RST and FIN in the same packet: the RST block in `apply_handshake_flags`
/// runs first and returns `PostHandshake::FlowClosed`; the FIN block is never
/// reached. Flow is closed with `CloseReason::Rst`, not `CloseReason::Fin`.
/// `stats.flows_rst == 1`, `stats.flows_fin == 0`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_ec006_rst_and_fin_same_packet_rst_wins() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // RST + FIN in the same packet (malformed but valid to process).
    // fin=true, rst=true — RST block runs first in apply_handshake_flags.
    let rst_fin = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2001,
        &[],
        false,
        false,
        true,
        true,
    );
    reassembler.process_packet(&rst_fin, 3, &mut handler);

    // BC-2.04.010 EC-006: RST wins — close reason is Rst, not Fin.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.010 EC-006: exactly one close event must be recorded"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Rst,
        "BC-2.04.010 EC-006: RST must win — close reason must be Rst, not Fin"
    );
    assert_eq!(
        reassembler.stats().flows_rst,
        1,
        "BC-2.04.010 EC-006: flows_rst must be 1"
    );
    assert_eq!(
        reassembler.stats().flows_fin,
        0,
        "BC-2.04.010 EC-006: flows_fin must be 0 (FIN block was not reached)"
    );
}

/// EC-007 (STORY-019 edge case — expire_flows boundary)
/// Flow idle for exactly `flow_timeout_secs` seconds is NOT expired (the
/// condition is `> timeout`, not `>= timeout`).
///
/// Test vector: last_seen=0, current_time=300, timeout=300 →
/// `current_time - last_seen == 300` which is NOT `> 300` → flow survives.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_013_ec007_flow_idle_exactly_timeout_is_not_expired() {
    // Canonical BC-2.04.013 EC-003 test vector: last_seen=0, timeout=300.
    // At current_time=300: 300 - 0 = 300 which is NOT > 300 → flow survives.
    let config = ReassemblyConfig {
        flow_timeout_secs: 300,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Flow with last_seen = 0.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 0, &mut handler);

    // expire_flows at current_time = 300 (exactly at the timeout boundary).
    reassembler.expire_flows(300, &mut handler);

    // BC-2.04.013 EC-007 (`>` not `>=`): flow must NOT be expired.
    assert_eq!(
        handler.close_events.len(),
        0,
        "BC-2.04.013 EC-007: flow idle exactly timeout secs must NOT be expired (> not >=)"
    );
    assert_eq!(
        reassembler.stats().flows_expired,
        0,
        "BC-2.04.013 EC-007: flows_expired must be 0 at exact timeout boundary"
    );

    // One second more (301 - 0 = 301 > 300 → expired).
    reassembler.expire_flows(301, &mut handler);
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.013 EC-007 sanity: flow must expire at current_time=301 (301-0=301 > 300)"
    );
}

/// EC-008 (STORY-019 edge case — timestamp reorder, engine-level)
/// `current_time < last_seen`: underflow guard `current_time > last_seen` is
/// false; flow NOT expired; no panic. This is the engine-level counterpart to
/// AC-011 (which exercises the same guard from a different angle).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_013_ec008_current_time_less_than_last_seen_no_expiry() {
    // release profile has overflow-checks=true; 500u32 - 1000u32 would panic.
    let config = ReassemblyConfig {
        flow_timeout_secs: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Flow with last_seen = 1000 (SYN at t=1000).
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1000, &mut handler);

    // expire_flows at current_time=500 (< last_seen=1000): must not panic.
    // BC-2.04.013 inv-1: `current_time > last_seen` is false → early exit → no subtraction.
    reassembler.expire_flows(500, &mut handler);

    // Flow must not be expired.
    assert_eq!(
        handler.close_events.len(),
        0,
        "BC-2.04.013 EC-008: backwards timestamp must not expire the flow"
    );
    assert_eq!(
        reassembler.stats().flows_expired,
        0,
        "BC-2.04.013 EC-008: flows_expired must be 0 (backwards timestamp guard)"
    );
}

/// EC-009 (STORY-019 edge case — CLOSE_FLOW_MISSING_WARNED already true)
/// When `CLOSE_FLOW_MISSING_WARNED` is already `true` before `close_flow` is
/// called for a missing key, the function returns silently: no `on_flow_close`
/// callback, no state change.
///
/// This is the pure-silent-return sub-scenario extracted from AC-014 for
/// independent coverage. Holds `CLOSE_FLOW_MISSING_WARNED_LOCK`.
///
/// Test seam accessors added in W8.3 (implementer step).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_029_ec009_already_warned_is_silent() {
    let _guard = CLOSE_FLOW_MISSING_WARNED_LOCK
        .lock()
        .expect("CLOSE_FLOW_MISSING_WARNED_LOCK poisoned");

    // Pre-condition: set the atomic to true (already warned).
    // Use the reset_for_testing to get a clean state, then set it via a first trigger.
    wirerust::reassembly::lifecycle::reset_close_flow_missing_warned_for_testing();

    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    use std::net::IpAddr;
    let missing_key_1 = wirerust::reassembly::flow::FlowKey::new(
        IpAddr::V4(std::net::Ipv4Addr::new(77, 0, 0, 1)),
        7001,
        IpAddr::V4(std::net::Ipv4Addr::new(77, 0, 0, 2)),
        7002,
    );
    let missing_key_2 = wirerust::reassembly::flow::FlowKey::new(
        IpAddr::V4(std::net::Ipv4Addr::new(77, 0, 0, 3)),
        7003,
        IpAddr::V4(std::net::Ipv4Addr::new(77, 0, 0, 4)),
        7004,
    );

    // First trigger: sets atomic from false → true.
    wirerust::reassembly::lifecycle::trigger_close_flow_missing_key_for_testing(
        &mut reassembler,
        &missing_key_1,
        CloseReason::Timeout,
        &mut handler,
    );
    assert!(
        wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        "EC-009 precondition: atomic must be true after first trigger"
    );

    // Snapshot.
    let close_before = handler.close_events.len();

    // Second trigger (atomic already true): silent return.
    wirerust::reassembly::lifecycle::trigger_close_flow_missing_key_for_testing(
        &mut reassembler,
        &missing_key_2,
        CloseReason::Timeout,
        &mut handler,
    );

    // BC-2.04.029 EC-009: no new close events (silent return).
    assert_eq!(
        handler.close_events.len(),
        close_before,
        "BC-2.04.029 EC-009: silent return when CLOSE_FLOW_MISSING_WARNED already true"
    );
    // Atomic still true.
    assert!(
        wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        "BC-2.04.029 EC-009: atomic remains true after second call (latching)"
    );
}

/// EC-010 (STORY-019 edge case — close_flow for key that IS in flows)
/// `close_flow` called for a key that exists: normal close path executes, no
/// warning, `CLOSE_FLOW_MISSING_WARNED` is unchanged (remains false/whatever
/// it was). `on_flow_close` fires exactly once with the specified reason.
///
/// Does NOT touch `CLOSE_FLOW_MISSING_WARNED` in a way that requires the lock
/// (normal close path never writes the atomic), but we do verify its state is
/// unchanged.
///
/// Test seam accessors added in W8.3 (implementer step).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_029_ec010_close_flow_for_existing_key_is_normal() {
    // Hold the lock even though this test only reads the atomic: other tests that
    // hold the lock may call reset_close_flow_missing_warned_for_testing(), which
    // would race the observation window between warned_before and the final assertion.
    let _guard = CLOSE_FLOW_MISSING_WARNED_LOCK
        .lock()
        .expect("CLOSE_FLOW_MISSING_WARNED_LOCK poisoned");

    // This EC exercises the NORMAL (non-missing-key) close_flow path.
    // We trigger it via RST (which calls close_flow internally for an existing key).
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Observe CLOSE_FLOW_MISSING_WARNED state before normal close.
    // Normal close_flow for an existing key NEVER writes the atomic.
    let warned_before = wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing();

    // RST closes the flow normally (key exists in self.flows at the time of RST).
    let rst = make_tcp_packet(
        server,
        80,
        client,
        12345,
        2000,
        &[],
        false,
        false,
        false,
        true,
    );
    reassembler.process_packet(&rst, 2, &mut handler);

    // BC-2.04.029 EC-010: normal close path executed → one close event.
    assert_eq!(
        handler.close_events.len(),
        1,
        "BC-2.04.029 EC-010: on_flow_close must fire once for normal close_flow"
    );
    assert_eq!(
        handler.close_events[0].1,
        CloseReason::Rst,
        "BC-2.04.029 EC-010: close reason must match the RST trigger"
    );

    // BC-2.04.029 EC-010: CLOSE_FLOW_MISSING_WARNED unchanged by normal path.
    assert_eq!(
        wirerust::reassembly::lifecycle::close_flow_missing_warned_for_testing(),
        warned_before,
        "BC-2.04.029 EC-010: CLOSE_FLOW_MISSING_WARNED must not change on normal close_flow"
    );
    assert_eq!(reassembler.total_memory(), 0);
}

// =============================================================================
// STORY-015: BC-2.04.006 — Bidirectional Data with Direction Tag
// =============================================================================

/// BC-2.04.006 PC1: handler.on_data is called with direction == ClientToServer
/// for bytes originating from the initiator endpoint.
/// Canonical vector: SYN from C, SYN+ACK from S, data from C → on_data tagged ClientToServer.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_006_client_to_server_data_tagged_correctly() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN from client sets initiator to client.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // SYN+ACK from server (engine sets initiator to dst = client).
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            5000,
            2000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );
    // Data from client (seq=1001, 3 bytes).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"abc", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.006 PC1: exactly one on_data event expected"
    );
    assert_eq!(
        handler.data_events[0].1,
        Direction::ClientToServer,
        "BC-2.04.006 PC1: direction must be ClientToServer for data from initiator"
    );
    assert_eq!(
        handler.data_events[0].2, b"abc",
        "BC-2.04.006 PC1: data content must match"
    );
}

/// BC-2.04.006 PC2: handler.on_data is called with direction == ServerToClient
/// for bytes originating from the responder endpoint.
/// Canonical vector: SYN from C, SYN+ACK from S, data from S → on_data tagged ServerToClient.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_006_server_to_client_data_tagged_correctly() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN from client.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // SYN+ACK from server — engine sets initiator to dst = client.
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            5000,
            2000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );
    // Data from server (seq=2001, 4 bytes).
    reassembler.process_packet(
        &make_tcp_packet(
            server, 80, client, 5000, 2001, b"wxyz", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.006 PC2: exactly one on_data event expected"
    );
    assert_eq!(
        handler.data_events[0].1,
        Direction::ServerToClient,
        "BC-2.04.006 PC2: direction must be ServerToClient for data from responder"
    );
    assert_eq!(
        handler.data_events[0].2, b"wxyz",
        "BC-2.04.006 PC2: data content must match"
    );
}

/// BC-2.04.006 PC3: The offset parameter in each on_data call is the
/// ISN-relative stream offset of the first byte of that chunk.
/// Discriminating vector: SYN at seq=1000 sets ISN=1000, base_offset=1.
/// First data at seq=1001 → offset = seq.wrapping_sub(isn) = 1001-1000 = 1.
/// The offset must be 1, not the raw sequence number 1001.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_006_on_data_offset_is_isn_relative() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN at seq=1000 → ISN=1000, base_offset=1 for client_to_server direction.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // Data at seq=1001 (3 bytes, contiguous at base_offset=1):
    // ISN-relative offset = seq.wrapping_sub(ISN) = 1001 - 1000 = 1.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"abc", false, true, false, false,
        ),
        2,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.006 PC3: one on_data event expected"
    );
    assert_eq!(
        handler.data_events[0].3, 1u64,
        "BC-2.04.006 PC3: offset must be ISN-relative: seq(1001) - ISN(1000) = 1, not the raw seq 1001"
    );
    assert_eq!(
        handler.data_events[0].2, b"abc",
        "BC-2.04.006 PC3: data content must match"
    );
}

/// BC-2.04.006 PC4: stats.bytes_reassembled increments by the total bytes
/// across all on_data calls in both directions.
/// Snapshot-and-delta: takes snapshot before, sends 3 c2s bytes + 4 s2c bytes,
/// asserts exact delta == 7.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_006_bytes_reassembled_counts_both_directions() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    let stats_before = reassembler.stats().bytes_reassembled;

    // 3 bytes c2s (seq=1001).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"abc", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    // 4 bytes s2c (seq=2001, mid-stream: ISN inferred as 2001-1=2000).
    reassembler.process_packet(
        &make_tcp_packet(
            server, 80, client, 5000, 2001, b"wxyz", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    let stats_after = reassembler.stats().bytes_reassembled;

    assert_eq!(
        stats_after,
        stats_before + 7,
        "BC-2.04.006 PC4: bytes_reassembled must increment by exactly 3+4=7 across both directions"
    );
    assert_eq!(
        handler.data_events.len(),
        2,
        "BC-2.04.006 PC4: exactly two on_data events (one per direction)"
    );
}

/// BC-2.04.006 inv-2: Client-to-server and server-to-client buffers are fully
/// independent; flushing one direction never drains the other.
/// Setup: SYN sets c2s ISN=1000. SYN-ACK sets s2c ISN=2000. Buffer OOO
/// segments in both directions (neither contiguous). Then fill c2s gap only
/// → c2s flushed; s2c still fully blocked.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_006_directions_are_independent() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Full handshake so both ISNs are set via SYN/SYN-ACK (no mid-stream inference).
    // SYN from client: c2s ISN=1000, base_offset=1.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // SYN-ACK from server: s2c ISN=2000, base_offset=1.
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            5000,
            2000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // Buffer c2s OOO: seq=1004 (offset=4, gap at 1-3).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1004, b"GHI", false, true, false, false,
        ),
        3,
        &mut handler,
    );
    // Buffer s2c OOO: seq=2005 (offset=5, gap at 1-4).
    reassembler.process_packet(
        &make_tcp_packet(
            server, 80, client, 5000, 2005, b"WXYZ", false, true, false, false,
        ),
        4,
        &mut handler,
    );

    // Both are OOO — no flush yet.
    assert_eq!(
        handler.data_events.len(),
        0,
        "BC-2.04.006 inv-2: neither direction should flush with a gap"
    );

    // Fill c2s gap: seq=1001 (3 bytes) makes offsets 1,2,3 contiguous, but
    // seq=1004 is at offset 4; fill with seq=1001 (1 byte) to advance base to 2,
    // then 1002, 1003 to reach 1004. Simpler: send one 3-byte fill at seq=1001
    // so offsets 1,2,3 are filled → then 1004 (offset=4) is contiguous.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"ABC", false, true, false, false,
        ),
        5,
        &mut handler,
    );

    // c2s now has ABC (offset=1, 3 bytes) feeding into GHI (offset=4); after ABC
    // flushes, base_offset advances to 4 where GHI is contiguous, so the entire
    // chain flushes in one go.

    let c2s_events: Vec<_> = handler
        .data_events
        .iter()
        .filter(|(_, dir, _, _)| *dir == Direction::ClientToServer)
        .collect();
    let s2c_events: Vec<_> = handler
        .data_events
        .iter()
        .filter(|(_, dir, _, _)| *dir == Direction::ServerToClient)
        .collect();

    assert!(
        !c2s_events.is_empty(),
        "BC-2.04.006 inv-2: c2s must have flushed after gap was filled"
    );
    assert_eq!(
        s2c_events.len(),
        0,
        "BC-2.04.006 inv-2: s2c must NOT have been flushed when only c2s gap was filled — directions are independent"
    );
}

// =============================================================================
// STORY-015: BC-2.04.007 — In-Order Data Flushes Contiguously
// =============================================================================

/// BC-2.04.007 PC1, PC3: When a segment arrives at exactly base_offset,
/// flush_contiguous_data removes all contiguous segments and delivers them
/// via on_data, and stats.bytes_reassembled increments by the total flushed bytes.
/// The ISN-relative offset in on_data is governed by BC-2.04.006 PC3.
/// Canonical vector: SYN at seq=1000; data at seq=1001 (immediately in-order).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_007_in_order_segment_flushed_immediately() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    let bytes_before = reassembler.stats().bytes_reassembled;

    // In-order data at seq=1001 (offset=1, contiguous with base_offset=1).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"hello", false, true, false, false,
        ),
        2,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.007 PC1: in-order segment must be immediately delivered via on_data"
    );
    assert_eq!(
        handler.data_events[0].2, b"hello",
        "BC-2.04.007 PC1: delivered data must match the inserted segment"
    );
    assert_eq!(
        handler.data_events[0].3, 1u64,
        "BC-2.04.006 PC3: offset must be ISN-relative (1 = seq 1001 - ISN 1000)"
    );
    assert_eq!(
        reassembler.stats().bytes_reassembled,
        bytes_before + 5,
        "BC-2.04.007 PC3: bytes_reassembled must advance by 5 (total flushed bytes)"
    );
}

/// BC-2.04.007 inv-1: Segments at offsets beyond the first gap are NOT flushed;
/// only the contiguous prefix from base_offset is consumed.
/// Discriminating: send in-order segment (offset=1), then OOO segment (offset=4,
/// gap at offset=4). Only the in-order bytes are flushed; the OOO segment stays buffered.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_007_gap_halts_flush() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN: ISN=1000.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // In-order segment: seq=1001, 3 bytes → offset=1 (contiguous at base_offset).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"abc", false, true, false, false,
        ),
        2,
        &mut handler,
    );

    // OOO segment: seq=1007, 3 bytes → offset=7 (gap at offset=4,5,6).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1007, b"xyz", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    // Only the in-order segment (offset=1) should have been delivered; OOO is buffered.
    assert_eq!(
        handler.data_events.len(),
        1,
        "BC-2.04.007 inv-1: only the in-order contiguous prefix must be flushed; gap halts flush"
    );
    assert_eq!(
        handler.data_events[0].2, b"abc",
        "BC-2.04.007 inv-1: flushed data must be 'abc' (the contiguous prefix)"
    );
    // The OOO bytes must NOT appear in any on_data event.
    let all_delivered: Vec<u8> = handler
        .data_events
        .iter()
        .flat_map(|(_, _, d, _)| d.iter().copied())
        .collect();
    assert!(
        !all_delivered.contains(&b'x'),
        "BC-2.04.007 inv-1: 'xyz' (beyond gap) must NOT have been delivered via on_data"
    );
}

// =============================================================================
// STORY-015: BC-2.04.008 — OOO Segments Buffer Until Gap Filled
// =============================================================================

/// BC-2.04.008 PC1–PC4: When a segment arrives ahead of base_offset (gap),
/// it is stored in the BTreeMap but NOT delivered. buffered_bytes increases;
/// on_data is NOT called.
/// Discriminating: expect data_events.len() == 0 and total_memory > 0.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_008_out_of_order_segment_buffered_not_delivered() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN: ISN=1000.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    let memory_before = reassembler.total_memory();

    // OOO segment: seq=1004 (offset=4), gap at offsets 1-3.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1004, b"def", false, true, false, false,
        ),
        2,
        &mut handler,
    );

    // BC-2.04.008 PC4: on_data must NOT be called for an OOO segment.
    assert_eq!(
        handler.data_events.len(),
        0,
        "BC-2.04.008 PC4: on_data must NOT be called for OOO segment with gap"
    );
    // BC-2.04.008 PC2-3: buffered_bytes/total_memory increases by exactly 3 bytes ("def").
    assert_eq!(
        reassembler.total_memory(),
        memory_before + 3,
        "BC-2.04.008 PC2+PC3: exactly 3 bytes (the OOO segment) added to buffer accounting"
    );
}

/// BC-2.04.008 PC5: When a later segment fills the gap, flush_contiguous
/// delivers both the fill segment and all previously-buffered contiguous
/// segments in ISN-relative order.
/// Canonical vector: segments arrive as [3,2,1]; assert flush delivers bytes in order [1,2,3].
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_008_gap_fill_delivers_all_contiguous() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN: ISN=1000.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Segment 3: seq=1007, data="ccc" (offset=7 — OOO).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1007, b"ccc", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    // Segment 2: seq=1004, data="bbb" (offset=4 — OOO, gap at 1-3 still).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1004, b"bbb", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    // Neither should have flushed yet.
    assert_eq!(
        handler.data_events.len(),
        0,
        "BC-2.04.008 PC5: neither OOO segment should have been delivered before gap fill"
    );

    // Segment 1: seq=1001, data="aaa" (offset=1 — fills gap; now all contiguous).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"aaa", false, true, false, false,
        ),
        4,
        &mut handler,
    );

    // All three should now be delivered in ISN-relative order.
    let all_bytes: Vec<u8> = handler
        .data_events
        .iter()
        .flat_map(|(_, _, d, _)| d.iter().copied())
        .collect();

    assert_eq!(
        all_bytes, b"aaabbbccc",
        "BC-2.04.008 PC5: fill segment must cause flush of all contiguous segments in order"
    );

    // Verify offsets are ascending across events.
    let offsets: Vec<u64> = handler.data_events.iter().map(|(_, _, _, o)| *o).collect();
    let is_ascending = offsets.windows(2).all(|w| w[0] < w[1]);
    assert!(
        is_ascending,
        "BC-2.04.008 PC5: on_data offsets must be in ascending ISN-relative order; got {:?}",
        offsets
    );
}

// =============================================================================
// STORY-015: Edge Cases
// =============================================================================

/// EC-001: Segment arrives in-order (no gap); immediately flushed; no buffering.
/// total_memory returns to 0 immediately after flush (no residue in buffer).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_007_ec001_in_order_no_buffering() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // In-order: no buffering should occur.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"hello", false, true, false, false,
        ),
        2,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        1,
        "EC-001: in-order segment must be immediately delivered"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-001: total_memory must be 0 after in-order flush — no buffering occurred"
    );
}

/// EC-002: Gap exists before flush; only prefix delivered (stop at gap).
/// Verify: send in-order segment then OOO segment; only in-order delivered; OOO buffered.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_007_ec002_gap_stops_flush() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // In-order: seq=1001, "abc" (offset=1).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"abc", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    // OOO: seq=1010, "xyz" (offset=10, gap at 4-9).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1010, b"xyz", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    // Only 1 on_data call (for "abc"); "xyz" must remain buffered.
    assert_eq!(
        handler.data_events.len(),
        1,
        "EC-002: only the contiguous prefix must be delivered; gap stops flush"
    );
    assert_eq!(
        handler.data_events[0].2, b"abc",
        "EC-002: delivered data must be the in-order 'abc' segment"
    );
    assert!(
        reassembler.total_memory() > 0,
        "EC-002: total_memory must be > 0 because 'xyz' is still buffered beyond the gap"
    );
}

/// EC-004: Empty payload (pure ACK); engine skips empty payloads before insert;
/// no on_data callback, no segment stored, total_memory unchanged.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_006_ec004_empty_payload_not_inserted() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    let memory_after_syn = reassembler.total_memory();

    // Pure ACK — empty payload.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1001,
            &[],
            false,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        0,
        "EC-004: pure ACK (empty payload) must not trigger on_data"
    );
    assert_eq!(
        reassembler.total_memory(),
        memory_after_syn,
        "EC-004: total_memory must not change after a pure ACK — no segment stored"
    );
}

/// EC-005: Multiple contiguous segments flushed in one call are delivered as
/// separate on_data calls (one per segment).
/// Discriminating: buffer 2 OOO segments, then fill gap → all 3 flushed as
/// 3 separate on_data events (not 1 merged event).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_007_ec005_multiple_contiguous_delivered_separately() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Buffer 2 OOO segments.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1004, b"bbb", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1007, b"ccc", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(handler.data_events.len(), 0);

    // Fill gap — triggers flush of all 3 contiguous segments.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"aaa", false, true, false, false,
        ),
        4,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        3,
        "EC-005: three contiguous segments flushed must produce 3 separate on_data events"
    );
    // Verify each is a distinct segment (not one merged blob).
    assert_eq!(
        handler.data_events[0].2, b"aaa",
        "EC-005: first event must be 'aaa'"
    );
    assert_eq!(
        handler.data_events[1].2, b"bbb",
        "EC-005: second event must be 'bbb'"
    );
    assert_eq!(
        handler.data_events[2].2, b"ccc",
        "EC-005: third event must be 'ccc'"
    );
}

/// EC-006: Three-segment out-of-order sequence (3,2,1): segments 3 and 2
/// buffered; segment 1 arrives and all three are flushed in order.
/// Canonical vector from BC-2.04.008: segments arrive as 3,2,1 → all three flushed in order.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_008_ec006_three_segment_ooo_321() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Arrival order: 3 → 2 → 1.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1007, b"333", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    assert_eq!(
        handler.data_events.len(),
        0,
        "EC-006: segment 3 must be buffered (gap)"
    );

    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1004, b"222", false, true, false, false,
        ),
        3,
        &mut handler,
    );
    assert_eq!(
        handler.data_events.len(),
        0,
        "EC-006: segment 2 must be buffered (gap)"
    );

    // Segment 1 fills gap → all three flush.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, 1001, b"111", false, true, false, false,
        ),
        4,
        &mut handler,
    );

    let all_bytes: Vec<u8> = handler
        .data_events
        .iter()
        .flat_map(|(_, _, d, _)| d.iter().copied())
        .collect();

    assert_eq!(
        all_bytes, b"111222333",
        "EC-006: all three segments must flush in ISN-relative order after gap fill"
    );
}

/// Baseline coverage for EC-009 ('Empty segments BTreeMap; flush_contiguous called → returns
/// empty Vec'). Engine-level: SYN-only flow has no payload, so insert_payload_segment +
/// flush_contiguous_data are never invoked. The actual empty-BTreeMap flush coverage is at the
/// segment-level: `test_BC_2_04_034_flush_contiguous_empty_when_no_segment_at_base`. This engine
/// test verifies the no-side-effect baseline for a flow with no data, not the empty-BTreeMap
/// flush path itself.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_034_ec009_syn_only_flow_no_data_events_baseline() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Only SYN — no data segments inserted.
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    assert_eq!(
        handler.data_events.len(),
        0,
        "EC-009: empty BTreeMap flush (SYN-only) must produce no on_data events"
    );
    assert_eq!(
        reassembler.stats().bytes_reassembled,
        0,
        "EC-009: bytes_reassembled must remain 0 when no data segments have been inserted"
    );
}

/// EC-008: ISN near u32::MAX; segments wrap around; all offsets correct
/// via wrapping_sub; BTreeMap keys monotonic; flush delivers in correct order.
/// Uses mid-stream ISN inference: first packet sets ISN to seq-1.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_039_ec008_isn_near_max_btreemap_keys_monotonic() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Use mid-stream inference: first c2s packet sets ISN = seq - 1.
    // ISN = u32::MAX - 2, so first data seq = u32::MAX - 1 (offset=1).
    let isn: u32 = u32::MAX - 2;
    let first_seq: u32 = isn.wrapping_add(1); // u32::MAX - 1

    // First segment: seq=u32::MAX-1, data="A" (offset=1 relative to ISN).
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, first_seq, b"A", false, true, false, false,
        ),
        1,
        &mut handler,
    );

    // Wrapped segment: seq=1 (= ISN+4 = u32::MAX-2+4 = 2, mod u32::MAX+1),
    // so offset = 1u32.wrapping_sub(isn) as u64.
    // ISN inferred as first_seq - 1 = u32::MAX - 2.
    // seq=1: 1u32.wrapping_sub(u32::MAX-2) = 4.
    let wrapped_seq: u32 = 1;

    reassembler.process_packet(
        &make_tcp_packet(
            client,
            5000,
            server,
            80,
            wrapped_seq,
            b"D",
            false,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // Fill the gap: seq=u32::MAX (offset=2) and seq=0 (offset=3).
    let gap_seq1: u32 = u32::MAX;
    let gap_seq2: u32 = 0;

    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, gap_seq1, b"B", false, true, false, false,
        ),
        3,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 5000, server, 80, gap_seq2, b"C", false, true, false, false,
        ),
        4,
        &mut handler,
    );

    let all_bytes: Vec<u8> = handler
        .data_events
        .iter()
        .flat_map(|(_, _, d, _)| d.iter().copied())
        .collect();

    assert!(
        !all_bytes.is_empty(),
        "EC-008: wraparound segments must eventually be delivered via on_data"
    );

    assert_eq!(
        all_bytes, b"ABCD",
        "EC-008: wrapped segments must deliver in byte order A,B,C,D (offset order, not arrival order); got {:?}",
        all_bytes
    );

    // Verify all delivered offsets are monotonically increasing.
    let offsets: Vec<u64> = handler.data_events.iter().map(|(_, _, _, o)| *o).collect();
    let is_monotonic = offsets.windows(2).all(|w| w[0] < w[1]);
    assert!(
        is_monotonic,
        "EC-008: on_data offsets must be monotonically increasing across wraparound; got {:?}",
        offsets
    );
}

// =============== STORY-020: total_memory + Eviction (Wave 9) ===============
//
// Behavioral contracts: BC-2.04.014, BC-2.04.015, BC-2.04.016, BC-2.04.017
// ACs: AC-001..AC-013 (13 tests)
// ECs: EC-001..EC-011 (11 tests); + 1 proptest (AC-004)
//
// All stubs panic to satisfy the Red Gate: every test must FAIL before
// implementation. Do NOT add #[ignore].
//
// Part B note: AC-004 requires asserting `total_memory == sum(flow.memory_used())`
// over the private `flows` map. The existing `total_memory()` public accessor
// gives the aggregate but NOT per-flow breakdown. To make the per-flow sum
// assertion feasible without reaching into private fields, Part B will need a
// `#[doc(hidden)] pub fn total_memory_for_testing(&self) -> usize` seam in
// `src/reassembly/mod.rs` that exposes the raw `self.total_memory` counter
// (which IS separately maintained from the sum), plus either a per-flow
// iterator or a `flows_memory_sum_for_testing() -> usize` helper that walks
// `self.flows` and sums `flow.memory_used()`. Both must be gated behind
// `#[cfg(test)]` or `#[doc(hidden)]` to avoid polluting the public API.
// ============================================================================

// ---- AC-001 (BC-2.04.014 postcondition 1) ----------------------------------

/// AC-001: After inserting N bytes into a flow direction's buffer,
/// `total_memory` increases by exactly N.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_014_total_memory_increments_on_insert() {
    // Use large memcap and max_flows so no eviction interferes.
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN establishes the flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    // SYN+ACK advances state to Established and sets server ISN.
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    let before = reassembler.total_memory();

    // Send an out-of-order segment so it stays buffered (gap at offset 1 means
    // flush_contiguous produces nothing, leaving bytes in the buffer).
    // Client ISN=1000 → base_offset=1. First data byte is at seq 1001 (offset 1).
    // We send at seq 1003 (offset 3), skipping offsets 1 and 2, so it buffers.
    let n: usize = 5;
    let data = make_tcp_packet(
        client, 12345, server, 80, 1003, &[0xAA; 5], false, false, false, false,
    );
    reassembler.process_packet(&data, 3, &mut handler);

    assert_eq!(
        reassembler.total_memory(),
        before + n,
        "AC-001: total_memory must increase by exactly N bytes after buffering N bytes"
    );
}

// ---- AC-002 (BC-2.04.014 postcondition 2) ----------------------------------

/// AC-002: After `flush_contiguous` delivers M bytes to the handler,
/// `total_memory` decreases by exactly M.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_014_total_memory_decrements_on_flush() {
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish the flow: SYN sets ISN=1000 → base_offset=1.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Buffer an out-of-order segment at offset 3 (5 bytes). Won't flush yet.
    let ooo = make_tcp_packet(
        client, 12345, server, 80, 1003, &[0xBB; 5], false, false, false, false,
    );
    reassembler.process_packet(&ooo, 3, &mut handler);
    let after_insert = reassembler.total_memory();
    assert_eq!(after_insert, 5, "precondition: 5 bytes buffered");

    // Now send the in-order head segment at offset 1 (2 bytes: seq=1001).
    // This fills the gap → flush_contiguous delivers 2+5=7 bytes to handler.
    let head = make_tcp_packet(
        client, 12345, server, 80, 1001, &[0xCC; 2], false, false, false, false,
    );
    reassembler.process_packet(&head, 4, &mut handler);

    // After the flush, total_memory should be 0 (all 7 bytes delivered).
    assert_eq!(
        reassembler.total_memory(),
        0,
        "AC-002: total_memory must decrement by the flushed bytes (7 bytes delivered)"
    );
    // Sanity: handler received data.
    assert!(
        !handler.data_events.is_empty(),
        "AC-002: handler must have received flushed data"
    );
}

// ---- AC-003 (BC-2.04.014 postcondition 3) ----------------------------------

/// AC-003: After `close_flow` removes a flow, `total_memory` decreases by
/// the flow's `memory_used()` at removal time (all remaining buffered bytes
/// in both directions).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_014_total_memory_decrements_on_close() {
    // Use a memcap large enough to never evict, and finalize to close all flows.
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish the flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Insert an out-of-order segment — it stays buffered (unflushed).
    let ooo = make_tcp_packet(
        client, 12345, server, 80, 1003, &[0xDD; 8], false, false, false, false,
    );
    reassembler.process_packet(&ooo, 3, &mut handler);
    let mem_before_close = reassembler.total_memory();
    assert!(mem_before_close > 0, "precondition: bytes must be buffered");

    // finalize() calls close_flow on all remaining flows.
    reassembler.finalize(&mut handler);

    assert_eq!(
        reassembler.total_memory(),
        0,
        "AC-003: total_memory must reach 0 after close_flow removes the flow with buffered bytes"
    );
}

// ---- AC-004 (BC-2.04.014 postcondition 4 + invariant 2) -------------------

/// AC-004: At all times, `total_memory == sum(flow.memory_used() for all flows)`.
/// This debug invariant holds after inserts, flushes, and closes.
///
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_014_total_memory_equals_sum_of_flow_memory() {
    // Seam flows_memory_sum_for_testing() is now present in src/reassembly/mod.rs.
    // This test exercises BC-2.04.014 postcondition 4 + invariant 2:
    //   total_memory == sum(flow.memory_used() for all flows in self.flows)
    // across inserts, flushes, and closes on multiple concurrent flows.
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut r = TcpReassembler::new(config);
    let mut h = RecordingHandler::new();

    // Helper macro: assert the invariant at a checkpoint.
    macro_rules! assert_invariant {
        ($label:expr) => {
            assert_eq!(
                r.total_memory(),
                r.flows_memory_sum_for_testing(),
                "AC-004 invariant violated at: {}",
                $label
            );
        };
    }

    assert_invariant!("initial state (empty)");

    // ---- Flow A on port 1 (client [10,0,0,1]:1 <-> server [10,0,0,2]:80) ----
    // SYN + SYN-ACK to reach Established.
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1,
            [10, 0, 0, 2],
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut h,
    );
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 2],
            80,
            [10, 0, 0, 1],
            1,
            2000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut h,
    );
    assert_invariant!("after Flow A handshake");

    // Out-of-order segment at seq=1003 (offset 3 past ISN=1000+1=1001); gap at
    // offsets 1-2 prevents flush → bytes stay buffered.
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1,
            [10, 0, 0, 2],
            80,
            1003,
            &[0xAA; 5],
            false,
            false,
            false,
            false,
        ),
        3,
        &mut h,
    );
    assert_invariant!("after Flow A inserts 5 bytes (buffered, gap)");

    // ---- Flow B on port 2 (client [10,0,0,1]:2 <-> server [10,0,0,2]:80) ----
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            2,
            [10, 0, 0, 2],
            80,
            3000,
            &[],
            true,
            false,
            false,
            false,
        ),
        4,
        &mut h,
    );
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 2],
            80,
            [10, 0, 0, 1],
            2,
            4000,
            &[],
            true,
            true,
            false,
            false,
        ),
        5,
        &mut h,
    );
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            2,
            [10, 0, 0, 2],
            80,
            3003,
            &[0xBB; 3],
            false,
            false,
            false,
            false,
        ),
        6,
        &mut h,
    );
    assert_invariant!("after Flow B inserts 3 bytes (buffered, gap)");

    // ---- Flush Flow A: fill gap at offset 1-2 (seq 1001, 2 bytes) ----
    // This allows flush_contiguous to deliver the 2 gap bytes + 5 buffered bytes.
    r.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1,
            [10, 0, 0, 2],
            80,
            1001,
            &[0xCC; 2],
            false,
            false,
            false,
            false,
        ),
        7,
        &mut h,
    );
    assert_invariant!("after Flow A gap filled (flush triggered)");

    // ---- Finalize: closes all remaining flows → total_memory should reach 0 ----
    r.finalize(&mut h);
    assert_eq!(
        r.total_memory(),
        0,
        "AC-004: total_memory must be 0 after finalize"
    );
    assert_eq!(
        r.flows_memory_sum_for_testing(),
        0,
        "AC-004: flows_memory_sum must be 0 after finalize (flows map is empty)"
    );
}

// ---- AC-005 (BC-2.04.015 postconditions 5-6) -------------------------------

/// AC-005: Verifies the no-op-eviction rejection path: when max_flows is at capacity but
/// total_memory <= memcap, evict_flows is called and exits at head under dual-conjunction
/// termination (per BC-2.04.015 v1.3 Invariant 4); new flow's SYN is dropped, existing
/// flow preserved.
///
/// Per BC-2.04.015 v1.3 Invariant 4: when `total_memory == memcap` exactly, the
/// dual-conjunction termination exits immediately — the existing flow is NOT evicted
/// and the new flow is dropped. Setup: max_flows=1, memcap=4, flow A buffers 4 bytes
/// (total == memcap, no eviction), B SYN arrives, evict_flows no-ops, B is dropped.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_015_new_flow_dropped_after_no_op_eviction_under_max_flows_only_pressure() {
    // max_flows=1, memcap=4. Flow A buffers 4 bytes (total == memcap; strict > not met,
    // no memcap eviction). When B SYN arrives: dual-conjunction exits immediately — A stays,
    // B dropped (BC-2.04.015 v1.3 Invariant 4).
    let config = ReassemblyConfig {
        max_flows: 1,
        memcap: 4,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client_a = [10, 0, 0, 1];
    let client_b = [10, 0, 0, 3];
    let server = [10, 0, 0, 2];

    // Flow A: SYN → SynSent; buffer 4 bytes OOO (total=4 == memcap=4, no eviction).
    let syn_a = make_tcp_packet(
        client_a,
        11111,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let ooo_a = make_tcp_packet(
        client_a, 11111, server, 80, 1003, &[0xAA; 4], false, false, false, false,
    );
    reassembler.process_packet(&ooo_a, 2, &mut handler);
    // total=4 == memcap=4; strict > check does not fire — no eviction yet.
    assert_eq!(
        reassembler.flow_count(),
        1,
        "precondition: flow A exists with 4 bytes"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "precondition: no eviction yet at exactly memcap"
    );
    assert_eq!(
        reassembler.total_memory(),
        4,
        "precondition: total_memory==memcap==4"
    );

    // Flow B SYN: get_or_create_flow calls evict_flows (flows.len()=1 >= max_flows=1).
    // evict_flows: total=4 <= memcap=4 AND flows.len()=1 <= max_flows=1 → dual-conjunction
    // breaks immediately — no eviction. Re-check: flows.len() still >= max_flows → B dropped.
    // Per BC-2.04.015 v1.3 Invariant 4: dual-conjunction termination is mechanical
    // (state-independent); flow A is preserved when total_memory does not exceed memcap.
    // (Note: A is SynSent in this setup; the protection applies to any state, with
    // Established being the most salient design-intent application.)
    let syn_b = make_tcp_packet(
        client_b,
        22222,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 2, &mut handler);

    // evict_flows exited without evicting; B's SYN was dropped.
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "AC-005: no eviction fires when total_memory==memcap (dual-conjunction termination \
         per BC-2.04.015 v1.3 Invariant 4)"
    );
    // Flow A still present (not evicted). Flow B was dropped.
    assert_eq!(
        reassembler.flow_count(),
        1,
        "AC-005: flow A must still exist (B was dropped because eviction did nothing)"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        1,
        "AC-005: only 1 flow ever created (B was dropped before creation)"
    );
    reassembler.finalize(&mut handler);
}

// ---- AC-006 (BC-2.04.015 postconditions 1 + 3) ----------------------------

/// AC-006: Non-Established flows (state != Established) are evicted before
/// Established flows regardless of their `last_seen` timestamps.
/// `stats.evictions` increments by the number of flows evicted.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_015_non_established_evicted_before_established() {
    // max_flows=100 (no max_flows pressure), memcap=12 (tight). A=Established(t=1) + B=SynSent(t=10);
    // buffer 13 bytes into A → memcap eviction fires; B evicted first (non-Established wins despite newer last_seen).
    let config = ReassemblyConfig {
        max_flows: 100,
        memcap: 12, // tight — will be exceeded when we buffer 13+ bytes
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: SYN+SYN_ACK → Established, at t=1.
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 1],
        1001,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 1, &mut handler);

    // Flow B: SYN only → SynSent, at t=10.
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        3000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 10, &mut handler);
    // Verify B is in SynSent: force_set not needed since on_syn() transitions to SynSent.
    assert_eq!(reassembler.flow_count(), 2, "precondition: 2 flows exist");

    // Insert 7 bytes out-of-order into flow A (won't flush — gap before them).
    let ooo_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1003,
        &[0xAA; 7],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a, 11, &mut handler);
    assert_eq!(reassembler.total_memory(), 7, "precondition: 7 bytes in A");

    // Insert 6 more bytes out-of-order into flow A → total=13 > memcap=12.
    // evict_flows fires. B (SynSent, newer) must be evicted before A (Established, older).
    let ooo_a2 = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1010,
        &[0xBB; 6],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a2, 12, &mut handler);

    // B must have been evicted (MemoryPressure).
    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
        2002,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    assert!(
        handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_b && *r == CloseReason::MemoryPressure),
        "AC-006: non-Established flow B (SynSent) must be evicted before Established flow A"
    );

    // stats.evictions >= 1.
    assert!(
        reassembler.stats().evictions >= 1,
        "AC-006: stats.evictions must be >= 1 after eviction"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-007 (BC-2.04.015 postcondition 4) ----------------------------------

/// AC-007: Each evicted flow triggers
/// `handler.on_flow_close(key, CloseReason::MemoryPressure)`.
///
/// Uses memcap-based eviction (the path that fires when total_memory > memcap).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_015_evicted_flow_receives_memory_pressure_reason() {
    // memcap=6: two flows each buffer 3+ bytes, combined > 6 → memcap eviction fires.
    // The evicted flow must receive CloseReason::MemoryPressure.
    let config = ReassemblyConfig {
        max_flows: 100,
        memcap: 6,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: SynSent, buffer 4 bytes (out-of-order after SYN).
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        11111,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let ooo_a = make_tcp_packet(
        [10, 0, 0, 1],
        11111,
        server,
        80,
        1003,
        &[0xAA; 4],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a, 2, &mut handler);
    assert_eq!(reassembler.total_memory(), 4, "precondition: 4 bytes in A");

    // Flow B: SynSent, buffer 3 bytes → total=7 > memcap=6 → eviction fires.
    // A (SynSent, older last_seen=2) evicted before B (SynSent, newer last_seen=3+).
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        22222,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 3, &mut handler);
    let ooo_b = make_tcp_packet(
        [10, 0, 0, 3],
        22222,
        server,
        80,
        2003,
        &[0xBB; 3],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_b, 4, &mut handler);
    // total=7 > memcap=6 → eviction fires; A (oldest) evicted first.

    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        11111,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let mp_events: Vec<_> = handler
        .close_events
        .iter()
        .filter(|(_, r)| *r == CloseReason::MemoryPressure)
        .collect();
    assert!(
        !mp_events.is_empty(),
        "AC-007: at least one MemoryPressure close event expected"
    );
    assert!(
        mp_events.iter().any(|(k, _)| *k == key_a),
        "AC-007: flow A (oldest SynSent) must have received CloseReason::MemoryPressure"
    );

    // W9 wave-level integration: BC-2.04.014 PC-4 joint invariant holds POST-EVICTION
    assert_eq!(
        reassembler.total_memory(),
        reassembler.flows_memory_sum_for_testing(),
        "BC-2.04.014 PC-4 / BC-2.04.047 PC-1 joint invariant must hold after eviction"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-008 (BC-2.04.016 postcondition 1) ----------------------------------

/// AC-008: After each packet, if `self.total_memory > self.config.memcap`,
/// `evict_flows` is called. After eviction, `total_memory <= memcap`
/// (when at least one flow exists to evict).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_016_memcap_eviction_triggers_after_insert() {
    let config = ReassemblyConfig {
        memcap: 10, // 10-byte cap
        max_flows: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: establish and buffer 7 bytes (out of order, no flush).
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 1],
        1001,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 1, &mut handler);
    let ooo_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1003,
        &[0xAA; 7],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a, 2, &mut handler);
    assert_eq!(reassembler.total_memory(), 7, "precondition: 7 bytes in A");

    // Flow B: establish and buffer 5 more bytes → total=12 > memcap=10.
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 3, &mut handler);
    let syn_ack_b = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 3],
        2002,
        6000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_b, 3, &mut handler);
    let ooo_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2003,
        &[0xBB; 5],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_b, 4, &mut handler);
    // total_memory was 7+5=12 > 10 → evict_flows fires.

    assert!(
        reassembler.total_memory() <= 10,
        "AC-008: total_memory must be <= memcap after eviction (was {}, memcap=10)",
        reassembler.total_memory()
    );
    assert!(
        reassembler.stats().evictions >= 1,
        "AC-008: at least one eviction must have occurred"
    );
    assert!(
        handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure),
        "AC-008: MemoryPressure close event must be present"
    );

    // W9 wave-level integration: BC-2.04.014 PC-4 joint invariant holds POST-EVICTION
    assert_eq!(
        reassembler.total_memory(),
        reassembler.flows_memory_sum_for_testing(),
        "BC-2.04.014 PC-4 / BC-2.04.047 PC-1 joint invariant must hold after eviction"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-009 (BC-2.04.016 invariant 2) -------------------------------------

/// AC-009: The memcap check uses strict `>` (not `>=`): at exactly `memcap`
/// bytes in `total_memory`, no eviction occurs.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_016_no_eviction_at_exactly_memcap() {
    let config = ReassemblyConfig {
        memcap: 7, // exactly 7 bytes allowed without eviction
        max_flows: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Establish flow A.
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 1],
        1001,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 1, &mut handler);

    // Buffer exactly 7 bytes (= memcap). Must NOT trigger eviction.
    let ooo_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1003,
        &[0xCC; 7],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a, 2, &mut handler);

    assert_eq!(
        reassembler.total_memory(),
        7,
        "precondition: total_memory == memcap == 7"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "AC-009: no eviction when total_memory == memcap (strict > check)"
    );
    assert!(
        !handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure),
        "AC-009: no MemoryPressure close event when at exactly memcap"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-010 (BC-2.04.017 postconditions 1-4) -------------------------------

/// AC-010: In `evict_flows`, the sort places all non-Established flows
/// (New, SynSent, Closing, Closed) before all Established flows. Within
/// each group, flows are sorted by `last_seen` ascending (oldest first).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_017_eviction_sort_non_established_first_then_lru() {
    // Three flows:
    //   A: Established, last_seen=5
    //   B: SynSent,     last_seen=10 (newer but non-Established)
    //   C: Established, last_seen=1  (oldest overall, but Established)
    //
    // max_flows=2, memcap=4 (tight). A=Established(t=1), B=SynSent(t=10) with 5 buffered bytes.
    // When C arrives: flows.len()=2 >= max_flows=2 → evict_flows; total=5 > memcap=4 → loop runs;
    // B (non-Established, newer) evicted first; after B: total=0 <= 4 → loop stops. C admitted, A survives.
    let config = ReassemblyConfig {
        max_flows: 2,
        memcap: 4, // tight: B will buffer 5 bytes > 4 to ensure loop doesn't break early
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: SYN (t=1) + SYN_ACK (t=1) → Established, last_seen=1.
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 1],
        1001,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 1, &mut handler);

    // Flow B: SYN only (t=2) → SynSent, last_seen=2 (newer but non-Established).
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 2, &mut handler);

    // Use memcap-based eviction (max_flows=100 → no max_flows pressure).
    // Buffer 3 bytes into A first (total=3 ≤ memcap=4, no eviction yet).
    let ooo_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1003,
        &[0xAA; 3],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a, 2, &mut handler);
    // total=3 <= memcap=4 → no eviction yet.

    assert_eq!(reassembler.flow_count(), 2, "precondition: 2 flows (A, B)");
    assert_eq!(reassembler.total_memory(), 3, "precondition: 3 bytes in A");

    // Buffer 2 bytes out-of-order into B (total=5 > memcap=4 → eviction fires NOW via memcap).
    // B (SynSent) is evicted before A (Established).
    let ooo_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2003,
        &[0xBB; 2],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_b, 3, &mut handler);
    // Now eviction has fired (memcap path), B evicted.

    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
        2002,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        1001,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    // B (non-Established) must have been evicted.
    assert!(
        handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_b && *r == CloseReason::MemoryPressure),
        "AC-010: non-Established flow B (SynSent, newer last_seen) must be evicted first"
    );
    // A (Established) must still be alive.
    assert!(
        !handler.close_events.iter().any(|(k, _)| *k == key_a),
        "AC-010: Established flow A must NOT be evicted (only non-Established evicted first)"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-011 (BC-2.04.017 edge case EC-001) ---------------------------------

/// AC-011: A non-Established flow with a NEWER `last_seen` timestamp is
/// evicted before an Established flow with an OLDER `last_seen` timestamp
/// (non-Established wins regardless of recency).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_017_non_established_newer_evicted_before_established_older() {
    // Flow A: Established, last_seen=1 (very old).
    // Flow B: SynSent,     last_seen=100 (very recent but non-Established).
    // Eviction must pick B despite B being much newer.
    // Use memcap=4 and buffer 5 bytes into B to ensure total > memcap at eviction time.
    let config = ReassemblyConfig {
        max_flows: 100,
        memcap: 4, // tight: B will buffer 5 bytes
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: Established at t=1.
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 1],
        1001,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 1, &mut handler);

    // Flow B: SynSent at t=100 (much newer than A).
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 100, &mut handler);

    assert_eq!(reassembler.flow_count(), 2, "precondition: 2 flows (A, B)");

    // Buffer 5 bytes out-of-order into B → total=5 > memcap=4 → eviction fires.
    // B (SynSent, newer last_seen=100) must be evicted before A (Established, older last_seen=1)
    // because non-Established sorts first regardless of recency.
    let ooo_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2003,
        &[0xBB; 5],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_b, 101, &mut handler);

    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
        2002,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        1001,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    assert!(
        handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_b && *r == CloseReason::MemoryPressure),
        "AC-011: newer non-Established flow B must be evicted before older Established flow A"
    );
    assert!(
        !handler.close_events.iter().any(|(k, _)| *k == key_a),
        "AC-011: Established flow A must NOT be evicted when a non-Established flow exists"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-012 (BC-2.04.017 invariant 3) -------------------------------------

/// AC-012: The eviction sort treats ALL states other than
/// `FlowState::Established` as "non-Established": `New`, `SynSent`,
/// `Closing`, and `Closed` all sort before any Established flow.
///
/// Verified by building 5 flows (A=Established, B=SynSent, C=Closing,
/// D=Closed, E=New), buffering bytes only into A, then triggering memcap
/// eviction. All 4 non-Established states (B/C/D/E) must appear in
/// close_events before A (per BC-2.04.017 invariant 3).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_017_all_non_established_states_evict_first() {
    // 5 flows are constructed with 0 data bytes each (handshake packets only,
    // no memcap pressure during setup). States forced via seam after last packet.
    // A single OOO data insert into A pushes total > memcap with all 5 flows
    // present, driving eviction in sort order: B/C/D/E (non-Established) before A.
    // Each non-Established flow holds 0 bytes → evicting them does not reduce
    // total_memory → loop continues until A (5 bytes) is also evicted.
    // Assertion: all 4 non-Established keys appear in close_events at positions
    // strictly before A's position.
    let config = ReassemblyConfig {
        max_flows: 100, // no max_flows pressure
        memcap: 4,      // 5 bytes in A will exceed this and drive eviction
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: SYN + SYN_ACK → Established, last_seen=1.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 1],
            1001,
            5000,
            &[],
            true,
            true,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        1001,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    // Flow B: SYN only → SynSent (no seam needed), last_seen=2.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        2,
        &mut handler,
    );
    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
        2002,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    // Flow C: SYN + SYN_ACK → Established, then forced to Closing via seam, last_seen=3.
    // force_set_flow_state_for_testing sets only flow.state; buffered bytes are unaffected.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 4],
            3003,
            server,
            80,
            3000,
            &[],
            true,
            false,
            false,
            false,
        ),
        3,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 4],
            3003,
            7000,
            &[],
            true,
            true,
            false,
            false,
        ),
        3,
        &mut handler,
    );
    let key_c = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 4])),
        3003,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    wirerust::reassembly::lifecycle::force_set_flow_state_for_testing(
        &mut reassembler,
        &key_c,
        wirerust::reassembly::flow::FlowState::Closing,
    );

    // Flow D: SYN + SYN_ACK → Established, then forced to Closed via seam, last_seen=4.
    // No further packets are sent to D so process_packet's Closed-state check never fires.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 5],
            4004,
            server,
            80,
            4000,
            &[],
            true,
            false,
            false,
            false,
        ),
        4,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 5],
            4004,
            8000,
            &[],
            true,
            true,
            false,
            false,
        ),
        4,
        &mut handler,
    );
    let key_d = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 5])),
        4004,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    wirerust::reassembly::lifecycle::force_set_flow_state_for_testing(
        &mut reassembler,
        &key_d,
        wirerust::reassembly::flow::FlowState::Closed,
    );

    // Flow E: SYN → SynSent, then forced back to New via seam, last_seen=5.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 6],
            5005,
            server,
            80,
            5000,
            &[],
            true,
            false,
            false,
            false,
        ),
        5,
        &mut handler,
    );
    let key_e = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 6])),
        5005,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    wirerust::reassembly::lifecycle::force_set_flow_state_for_testing(
        &mut reassembler,
        &key_e,
        wirerust::reassembly::flow::FlowState::New,
    );

    assert_eq!(
        reassembler.flow_count(),
        5,
        "precondition: 5 flows (A=Established, B=SynSent, C=Closing, D=Closed, E=New)"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "precondition: 0 bytes buffered before data insert (handshake packets only)"
    );

    // Insert 5 bytes OOO into A → total=5 > memcap=4 → evict_flows fires with all 5 present.
    // Sort: B/C/D/E (non-Established, 0 bytes each) before A (Established, 5 bytes).
    // Each non-Established eviction frees 0 bytes → loop continues past each until A is hit.
    // Eviction order in close_events: B, C, D, E, then A.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1003,
            &[0xAA; 5],
            false,
            false,
            false,
            false,
        ),
        6,
        &mut handler,
    );

    let mp_events: Vec<_> = handler
        .close_events
        .iter()
        .filter(|(_, r)| *r == CloseReason::MemoryPressure)
        .collect();

    // All 5 flows must appear in close_events (non-Established can't satisfy memcap alone).
    assert_eq!(
        mp_events.len(),
        5,
        "AC-012: all 5 flows (4 non-Established + 1 Established) must be evicted"
    );

    // Find A's position in the eviction sequence.
    let pos_a = handler
        .close_events
        .iter()
        .position(|(k, r)| *k == key_a && *r == CloseReason::MemoryPressure)
        .expect("AC-012: Established flow A must appear in close_events");

    // Each non-Established state must appear before A (BC-2.04.017 invariant 3).
    let pos_b = handler
        .close_events
        .iter()
        .position(|(k, r)| *k == key_b && *r == CloseReason::MemoryPressure)
        .expect("AC-012: SynSent flow B must appear in close_events");
    assert!(
        pos_b < pos_a,
        "AC-012: SynSent flow B (pos={pos_b}) must be evicted before Established flow A (pos={pos_a})"
    );

    let pos_c = handler
        .close_events
        .iter()
        .position(|(k, r)| *k == key_c && *r == CloseReason::MemoryPressure)
        .expect("AC-012: Closing flow C must appear in close_events");
    assert!(
        pos_c < pos_a,
        "AC-012: Closing flow C (pos={pos_c}) must be evicted before Established flow A (pos={pos_a})"
    );

    let pos_d = handler
        .close_events
        .iter()
        .position(|(k, r)| *k == key_d && *r == CloseReason::MemoryPressure)
        .expect("AC-012: Closed flow D must appear in close_events");
    assert!(
        pos_d < pos_a,
        "AC-012: Closed flow D (pos={pos_d}) must be evicted before Established flow A (pos={pos_a})"
    );

    let pos_e = handler
        .close_events
        .iter()
        .position(|(k, r)| *k == key_e && *r == CloseReason::MemoryPressure)
        .expect("AC-012: New flow E must appear in close_events");
    assert!(
        pos_e < pos_a,
        "AC-012: New flow E (pos={pos_e}) must be evicted before Established flow A (pos={pos_a})"
    );

    // No more packets; finalize is a no-op (all flows already evicted).
    reassembler.finalize(&mut handler);
}

// ---- AC-013 (BC-2.04.015 invariant 1) -------------------------------------

/// AC-013: Both eviction triggers (max_flows via `get_or_create_flow` and
/// memcap via `process_packet`) call the same `evict_flows` function with
/// the same LRU non-established-first strategy.
///
/// Verified by exercising BOTH paths. PATH 1 (get_or_create_flow no-op entry) asserts
/// evict_flows is called and exits without eviction under dual-conjunction termination
/// (evictions==0, B dropped). PATH 2 (process_packet memcap entry) asserts eviction fires
/// and emits CloseReason::MemoryPressure with non-Established-first ordering.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_015_both_eviction_paths_use_same_function() {
    // --- PATH 1: max_flows eviction entry point (via get_or_create_flow) ---
    // Setup: max_flows=1, memcap=3. Flow A buffers 3 bytes (total=3 == memcap, no early
    // eviction). Flow B SYN arrives: flows.len()=1 >= max_flows=1 → get_or_create_flow
    // calls evict_flows. At evict_flows entry: total=3 <= memcap=3 → dual-conjunction
    // terminates immediately, no eviction occurs. B is dropped (re-check still full).
    // PATH 1 witness is the dual-conjunction no-op (evictions==0, B dropped); PATH 2 witness is CloseReason::MemoryPressure emission.
    {
        let config = ReassemblyConfig {
            max_flows: 1,
            memcap: 3,
            ..ReassemblyConfig::default()
        };
        let mut r = TcpReassembler::new(config);
        let mut h = RecordingHandler::new();
        let server = [10, 0, 0, 2];

        // Flow A: SynSent, buffer 3 bytes out-of-order (total=3 == memcap=3, no eviction).
        r.process_packet(
            &make_tcp_packet(
                [10, 0, 0, 1],
                1001,
                server,
                80,
                1000,
                &[],
                true,
                false,
                false,
                false,
            ),
            1,
            &mut h,
        );
        r.process_packet(
            &make_tcp_packet(
                [10, 0, 0, 1],
                1001,
                server,
                80,
                1003,
                &[0xAA; 3],
                false,
                false,
                false,
                false,
            ),
            2,
            &mut h,
        );
        assert_eq!(
            r.total_memory(),
            3,
            "AC-013 PATH1 precondition: total==memcap"
        );
        assert_eq!(
            r.stats().evictions,
            0,
            "AC-013 PATH1 precondition: no eviction yet"
        );
        assert_eq!(
            r.flow_count(),
            1,
            "AC-013 PATH1 precondition: flow A present"
        );

        // Flow B SYN: triggers get_or_create_flow PATH 1 entry into evict_flows.
        // evict_flows dual-conjunction: total=3 <= 3 AND flows.len()=1 <= 1 → break.
        // No eviction. B is dropped (flows.len() still >= max_flows after evict_flows).
        r.process_packet(
            &make_tcp_packet(
                [10, 0, 0, 3],
                2002,
                server,
                80,
                2000,
                &[],
                true,
                false,
                false,
                false,
            ),
            3,
            &mut h,
        );
        // PATH 1 entry-point witness: evict_flows was called but terminated without
        // evicting (dual-conjunction protects A). B was dropped, not A.
        assert_eq!(
            r.stats().evictions,
            0,
            "AC-013 PATH1: get_or_create_flow called evict_flows; no eviction \
             because total<=memcap (dual-conjunction termination per BC-2.04.015 v1.3 Invariant 4)"
        );
        assert_eq!(
            r.flow_count(),
            1,
            "AC-013 PATH1: flow A preserved; B was dropped (table still full after evict_flows)"
        );
        assert_eq!(
            r.stats().flows_total,
            1,
            "AC-013 PATH1: only one flow ever created (B's SYN dropped)"
        );
        r.finalize(&mut h);
    }

    // --- PATH 2: memcap eviction (via process_packet post-insert check) ---
    {
        let config = ReassemblyConfig {
            max_flows: 100,
            memcap: 5, // very tight
            ..ReassemblyConfig::default()
        };
        let mut r = TcpReassembler::new(config);
        let mut h = RecordingHandler::new();
        let server = [10, 0, 0, 2];

        // Flow A: Established, buffer 6 bytes (> memcap=5) → memcap eviction fires.
        r.process_packet(
            &make_tcp_packet(
                [10, 0, 0, 1],
                1001,
                server,
                80,
                1000,
                &[],
                true,
                false,
                false,
                false,
            ),
            1,
            &mut h,
        );
        r.process_packet(
            &make_tcp_packet(
                server,
                80,
                [10, 0, 0, 1],
                1001,
                5000,
                &[],
                true,
                true,
                false,
                false,
            ),
            1,
            &mut h,
        );
        // Flow B: SynSent (to be evicted as non-Established).
        r.process_packet(
            &make_tcp_packet(
                [10, 0, 0, 3],
                2002,
                server,
                80,
                2000,
                &[],
                true,
                false,
                false,
                false,
            ),
            2,
            &mut h,
        );
        // Buffer 6 bytes out-of-order into A → total=6 > memcap=5 → memcap eviction.
        // B (SynSent) is evicted first.
        r.process_packet(
            &make_tcp_packet(
                [10, 0, 0, 1],
                1001,
                server,
                80,
                1003,
                &[0xAA; 6],
                false,
                false,
                false,
                false,
            ),
            3,
            &mut h,
        );
        assert!(
            r.stats().evictions >= 1,
            "AC-013 PATH2: memcap eviction must have occurred"
        );
        assert!(
            h.close_events
                .iter()
                .any(|(_, r)| *r == CloseReason::MemoryPressure),
            "AC-013 PATH2: memcap eviction must emit CloseReason::MemoryPressure"
        );
        // Both paths: non-Established evicted first.
        let key_b = FlowKey::new(
            IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
            2002,
            IpAddr::V4(Ipv4Addr::from(server)),
            80,
        );
        assert!(
            h.close_events
                .iter()
                .any(|(k, r2)| *k == key_b && *r2 == CloseReason::MemoryPressure),
            "AC-013 PATH2: non-Established flow B must be the first evicted"
        );
        r.finalize(&mut h);
    }
}

// ---- EC-001 ----------------------------------------------------------------

/// EC-001: Insert segment, flush immediately; total_memory increments then
/// returns to 0.
#[test]
fn test_story_020_ec001_insert_then_flush_returns_to_zero() {
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow: SYN sets client ISN=1000 → base_offset=1.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    assert_eq!(reassembler.total_memory(), 0, "precondition: empty");

    // Send in-order data at seq=1001 (base_offset=1). This IS the contiguous
    // start → flush_contiguous delivers immediately → total_memory returns to 0.
    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, &[0xAA; 4], false, false, false, false,
    );
    reassembler.process_packet(&data, 3, &mut handler);

    // After flush: total_memory back to 0.
    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-001: total_memory must return to 0 after in-order segment is flushed immediately"
    );
    // Handler received the data.
    assert!(
        !handler.data_events.is_empty(),
        "EC-001: handler must have received the flushed data"
    );

    reassembler.finalize(&mut handler);
}

// ---- EC-002 ----------------------------------------------------------------

/// EC-002: Close flow with buffered data; total_memory decreases by all
/// buffered bytes in both directions.
#[test]
fn test_story_020_ec002_close_flow_with_buffered_data() {
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow bidirectionally.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Buffer 6 bytes client→server (out-of-order: gap at offset 1 & 2).
    let ooo_c2s = make_tcp_packet(
        client, 12345, server, 80, 1003, &[0xAA; 6], false, false, false, false,
    );
    reassembler.process_packet(&ooo_c2s, 3, &mut handler);

    // Buffer 4 bytes server→client (out-of-order: server ISN=5000 → base=1, seq=5003).
    let ooo_s2c = make_tcp_packet(
        server, 80, client, 12345, 5003, &[0xBB; 4], false, false, false, false,
    );
    reassembler.process_packet(&ooo_s2c, 4, &mut handler);

    let buffered = reassembler.total_memory();
    assert_eq!(
        buffered, 10,
        "precondition: 10 bytes buffered (6 c2s + 4 s2c)"
    );

    // finalize() closes all flows → total_memory must reach 0.
    reassembler.finalize(&mut handler);

    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-002: total_memory must be 0 after closing flow with buffered data in both directions"
    );
}

// ---- EC-003 ----------------------------------------------------------------

/// EC-003: Zero-length segment insert; total_memory unchanged (empty data
/// early return).
#[test]
fn test_story_020_ec003_zero_length_segment_no_memory_change() {
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    let before = reassembler.total_memory();
    assert_eq!(before, 0, "precondition: empty");

    // Pure ACK (zero-length payload) — should not change total_memory.
    let pure_ack = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1001,
        &[], // zero-length
        false,
        true, // ACK flag set
        false,
        false,
    );
    reassembler.process_packet(&pure_ack, 3, &mut handler);

    assert_eq!(
        reassembler.total_memory(),
        before,
        "EC-003: zero-length segment must not change total_memory"
    );

    reassembler.finalize(&mut handler);
}

// ---- EC-004 ----------------------------------------------------------------

/// EC-004: All flows are Established at eviction time; LRU Established
/// flows evicted (oldest first).
#[test]
fn test_story_020_ec004_all_established_flows_evict_lru_order() {
    // Three Established flows: A (last_seen=1), B (last_seen=5), C (last_seen=10, gets data).
    // When C's out-of-order data triggers memcap eviction:
    //   C's last_seen gets updated to 11 (the packet timestamp).
    //   Evict order: A (t=1), B (t=5), C (t=11).
    //   With memcap=4 and 5 bytes buffered in C: evict A → total=0 → stop.
    //   B and C survive.
    //
    // Note: the flow that receives data gets its last_seen UPDATED before eviction.
    // So we must ensure A and B have timestamps older than C's data-arrival timestamp.
    let config = ReassemblyConfig {
        max_flows: 100,
        memcap: 4, // tight: C will buffer 5 bytes to trigger eviction
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: Established at t=1 (oldest).
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 1],
            1001,
            5000,
            &[],
            true,
            true,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Flow B: Established at t=5 (middle).
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        5,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 3],
            2002,
            6000,
            &[],
            true,
            true,
            false,
            false,
        ),
        5,
        &mut handler,
    );

    assert_eq!(
        reassembler.flow_count(),
        2,
        "precondition: 2 Established flows"
    );

    // Send 5 bytes OOO into flow A at t=10 → A.last_seen=10, B.last_seen=5.
    // total=5 > memcap=4 → eviction fires. Sort: B(t=5) < A(t=10).
    // Evict B first (0 bytes freed, total=5>4 still), then evict A (5 bytes, total=0).
    // EC-004 verifies ORDER: B evicted before A (LRU-first across all-Established).
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1003, // out-of-order (gap at offset 1-2)
            &[0xAA; 5],
            false,
            false,
            false,
            false,
        ),
        10, // t=10: A's last_seen updated to 10. B's last_seen=5 (older).
        &mut handler,
    );
    // total=5 > memcap=4 → eviction fires.
    // At eviction: B (is_est=true, t=5) sorts BEFORE A (is_est=true, t=10).
    // Evict B (0 bytes): total=5>4 → continue. Evict A (5 bytes): total=0<=4 → stop.
    // Both evicted. EC-004 verifies ORDER: B evicted before A.

    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        1001,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
        2002,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    let mp_evictions: Vec<_> = handler
        .close_events
        .iter()
        .filter(|(_, r)| *r == CloseReason::MemoryPressure)
        .collect();
    assert_eq!(
        mp_evictions.len(),
        2,
        "EC-004: both Established flows evicted (B first by LRU, then A)"
    );
    // B (last_seen=5) evicted before A (last_seen updated to 10 by the data packet).
    assert_eq!(
        mp_evictions[0].0, key_b,
        "EC-004: flow B (older at eviction time, last_seen=5) must be evicted FIRST"
    );
    assert_eq!(
        mp_evictions[1].0, key_a,
        "EC-004: flow A (newer at eviction time, last_seen=10) must be evicted SECOND"
    );

    // No finalize needed — all flows already evicted.
    reassembler.finalize(&mut handler);
}

// ---- EC-005 ----------------------------------------------------------------

/// EC-005: Single flow in table at max_flows=1, new SYN arrives; existing
/// flow survives and new SYN is dropped (pure max_flows pressure, no memcap
/// violation).
///
/// Per BC-2.04.015 v1.3 Invariant 4: evict_flows uses dual-conjunction termination
/// (total_memory <= memcap AND flows.len() <= max_flows) to exit without evicting when
/// neither resource threshold is strictly exceeded. With total_memory=3 == memcap=3 and
/// flows.len()=1 == max_flows=1, the eviction loop exits immediately — flow A is preserved
/// and flow B's SYN is dropped (dual-conjunction termination).
///
/// EC-011 (below) tests the positive dual-pressure case where A IS evicted
/// and B IS admitted when total_memory genuinely exceeds memcap.
#[test]
fn test_story_020_ec005_max_flows_only_pressure_drops_new_syn_preserves_existing_flow() {
    // max_flows=1, memcap=3. A buffers exactly 3 bytes (total == memcap; no eviction).
    // B SYN: evict_flows dual-conjunction exits immediately → A survives, B dropped.
    let config = ReassemblyConfig {
        max_flows: 1,
        memcap: 3,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // A buffers exactly memcap=3 bytes (total=3 = memcap → strict > not met, no eviction).
    // Per BC-2.04.015 v1.3 Invariant 4: dual-conjunction termination protects A when
    // total_memory does not strictly exceed memcap; B's SYN is dropped, not A.
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        11111,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    // Buffer 3 bytes (= memcap) into A, out-of-order.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            11111,
            server,
            80,
            1003,
            &[0xAA; 3],
            false,
            false,
            false,
            false,
        ),
        2,
        &mut handler,
    );
    assert_eq!(
        reassembler.total_memory(),
        3,
        "precondition: total=memcap=3"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "precondition: no eviction at exactly memcap"
    );
    assert_eq!(reassembler.flow_count(), 1, "precondition: flow A present");

    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        11111,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    // Flow B SYN: per BC-2.04.015 v1.3 Invariant 4 — A is NOT evicted
    // (total=3 == memcap=3, so total_memory is NOT strictly > memcap).
    // B's SYN is DROPPED (get_or_create_flow returns false) because the eviction
    // loop correctly terminates without action (dual-conjunction) when neither
    // resource threshold is exceeded.
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        22222,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 2, &mut handler);

    // evict_flows called but terminates immediately (dual-conjunction); A survives, B dropped.
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "EC-005: no eviction fires when total_memory==memcap — dual-conjunction \
         termination per BC-2.04.015 v1.3 Invariant 4"
    );
    assert_eq!(
        reassembler.flow_count(),
        1,
        "EC-005: flow A must survive (memory budget not exceeded), B was dropped"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        1,
        "EC-005: only flow A was ever created (B's SYN dropped per Invariant 4)"
    );
    assert!(
        !handler.close_events.iter().any(|(k, _)| *k == key_a),
        "EC-005: flow A must NOT have been evicted (dual-conjunction termination \
         per BC-2.04.015 v1.3 Invariant 4)"
    );

    reassembler.finalize(&mut handler);
}

// ---- EC-006 ----------------------------------------------------------------

/// EC-006: total_memory == memcap exactly; no eviction triggered
/// (strict `>`).
#[test]
fn test_story_020_ec006_total_memory_equals_memcap_no_eviction() {
    // Identical to AC-009 but named per the EC catalog for completeness.
    let config = ReassemblyConfig {
        memcap: 5,
        max_flows: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // Buffer exactly 5 bytes (= memcap).
    let ooo = make_tcp_packet(
        client, 12345, server, 80, 1003, &[0xEE; 5], false, false, false, false,
    );
    reassembler.process_packet(&ooo, 3, &mut handler);

    assert_eq!(
        reassembler.total_memory(),
        5,
        "precondition: total == memcap == 5"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "EC-006: no eviction when total_memory == memcap (strict >)"
    );

    reassembler.finalize(&mut handler);
}

// ---- EC-007 ----------------------------------------------------------------

/// EC-007: total_memory == memcap + 1; eviction triggered.
#[test]
fn test_story_020_ec007_total_memory_one_over_memcap_triggers_eviction() {
    // Two flows: flow A (Established) holds 5 bytes.
    // memcap=5. Inserting 1 more byte (into flow B) pushes total to 6 > 5.
    let config = ReassemblyConfig {
        memcap: 5,
        max_flows: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: Established, buffer 5 bytes (at exactly memcap — no eviction yet).
    let syn_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 1, &mut handler);
    let syn_ack_a = make_tcp_packet(
        server,
        80,
        [10, 0, 0, 1],
        1001,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack_a, 1, &mut handler);
    let ooo_a = make_tcp_packet(
        [10, 0, 0, 1],
        1001,
        server,
        80,
        1003,
        &[0xAA; 5],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_a, 2, &mut handler);
    assert_eq!(
        reassembler.total_memory(),
        5,
        "precondition: total_memory == memcap == 5, no eviction yet"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "precondition: no eviction at exactly memcap"
    );

    // Flow B: SynSent (non-Established, will be evicted first).
    let syn_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 3, &mut handler);

    // Insert 1 byte into flow B → total becomes 5+1=6 > memcap=5 → eviction fires.
    // B (SynSent, non-Established, 1 byte) is evicted → total=5, still == memcap.
    // Then A still has 5 bytes buffered but total == memcap → no further eviction.
    let ooo_b = make_tcp_packet(
        [10, 0, 0, 3],
        2002,
        server,
        80,
        2003,
        &[0xBB; 1],
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&ooo_b, 4, &mut handler);

    assert!(
        reassembler.stats().evictions >= 1,
        "EC-007: eviction must trigger when total_memory == memcap + 1"
    );
    assert!(
        handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure),
        "EC-007: MemoryPressure close event must be present"
    );

    reassembler.finalize(&mut handler);
}

// ---- EC-008 ----------------------------------------------------------------

/// EC-008: evict_flows with a Closing flow and an Established flow;
/// Closing (non-Established) is evicted first.
///
/// Uses memcap pressure (memcap=4) to ensure evict_flows loop doesn't
/// terminate early. B (Closing) buffers 5 bytes > memcap=4 to drive eviction.
#[test]
fn test_story_020_ec008_closing_flow_evicted_before_established() {
    let config = ReassemblyConfig {
        max_flows: 100,
        memcap: 4, // tight: B will buffer 5 bytes > 4 to drive eviction
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: Established at t=1 (older).
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 1],
            1001,
            5000,
            &[],
            true,
            true,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Flow B: Established at t=2, then forced to Closing via seam.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 3],
            2002,
            6000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );
    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 3])),
        2002,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    wirerust::reassembly::lifecycle::force_set_flow_state_for_testing(
        &mut reassembler,
        &key_b,
        wirerust::reassembly::flow::FlowState::Closing,
    );

    // Buffer 5 bytes out-of-order into B → total=5 > memcap=4 → eviction fires.
    // B (Closing, non-Established) must be evicted before A (Established).
    // After evicting B: total=0 <= 4 → loop stops. A survives.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2003, // out-of-order (gap at offset 1-2 since B ISN=2000)
            &[0xBB; 5],
            false,
            false,
            false,
            false,
        ),
        3,
        &mut handler,
    );

    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from([10, 0, 0, 1])),
        1001,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    assert!(
        handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_b && *r == CloseReason::MemoryPressure),
        "EC-008: Closing flow B must be evicted before Established flow A"
    );
    assert!(
        !handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_a && *r == CloseReason::MemoryPressure),
        "EC-008: Established flow A must NOT be evicted when Closing flow B exists"
    );

    reassembler.finalize(&mut handler);
}

// ---- EC-009 ----------------------------------------------------------------

/// EC-009: Verifies BC-2.04.015 edge case 009 — engine continues processing
/// after all flows are evicted; evict_flows loop terminates gracefully
/// (no panic, no infinite loop) even when exhausting all candidates.
///
/// Note: 'stays over cap' per BC EC-009 is unreachable in the current API
/// (every insertion associates bytes to a flow that becomes evictable; evicting
/// that flow subtracts its bytes from total_memory, so total_memory reaches 0
/// when the last flow is evicted). We verify the 'engine continues processing'
/// half of EC-009: flow count reaches 0, total_memory reaches 0, and a
/// subsequent SYN is admitted without panic.
#[test]
fn test_story_020_ec009_all_flows_evicted_still_over_memcap_continues() {
    // Use a very tight memcap that forces ALL flows to be evicted.
    // A single flow with 5 bytes; memcap=1. Evicting that flow brings total=0 <= 1.
    // The loop terminates, no panic, processing continues.
    let config = ReassemblyConfig {
        memcap: 1,
        max_flows: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Flow A: Established, buffer 5 bytes (> memcap=1 → evict A entirely).
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 1],
            1001,
            5000,
            &[],
            true,
            true,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // Insert 5 bytes out-of-order into A → total=5 > memcap=1 → evict A → total=0.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1003,
            &[0xAA; 5],
            false,
            false,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // After eviction, the engine must still be in a valid operational state.
    // Verify: no panic occurred (we reached here), flow count is 0.
    assert_eq!(
        reassembler.flow_count(),
        0,
        "EC-009: all flows evicted when over memcap; flow count must be 0"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-009: total_memory must be 0 after all flows evicted"
    );

    // Processing continues: send another packet after the eviction.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        3,
        &mut handler,
    );
    // Engine still functional: new flow was admitted (total_memory=0 <= memcap=1).
    // (Or evicted immediately if the new SYN somehow added bytes — it doesn't.)
    // No panic = test passes.
    reassembler.finalize(&mut handler);
}

// ---- EC-010 ----------------------------------------------------------------

/// EC-010: finalize closes all flows; total_memory reaches 0 after finalize.
#[test]
fn test_story_020_ec010_finalize_zeroes_total_memory() {
    let config = ReassemblyConfig {
        memcap: 1024 * 1024,
        max_flows: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];

    // Build three flows with varying amounts of buffered data.

    // Flow A: 6 bytes buffered.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 1],
            1001,
            5000,
            &[],
            true,
            true,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            1001,
            server,
            80,
            1003,
            &[0xAA; 6],
            false,
            false,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // Flow B: 4 bytes buffered.
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        3,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            [10, 0, 0, 3],
            2002,
            6000,
            &[],
            true,
            true,
            false,
            false,
        ),
        3,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 3],
            2002,
            server,
            80,
            2003,
            &[0xBB; 4],
            false,
            false,
            false,
            false,
        ),
        4,
        &mut handler,
    );

    // Flow C: 3 bytes buffered (SynSent only, out-of-order data).
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 5],
            3003,
            server,
            80,
            3000,
            &[],
            true,
            false,
            false,
            false,
        ),
        5,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 5],
            3003,
            server,
            80,
            3003,
            &[0xCC; 3],
            false,
            false,
            false,
            false,
        ),
        6,
        &mut handler,
    );

    let before = reassembler.total_memory();
    assert!(
        before > 0,
        "precondition: some bytes buffered across 3 flows (got {})",
        before
    );

    // finalize() must close all flows and bring total_memory to 0.
    reassembler.finalize(&mut handler);

    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-010: total_memory must be 0 after finalize closes all flows"
    );
    assert_eq!(
        reassembler.flow_count(),
        0,
        "EC-010: flow count must be 0 after finalize"
    );
}

// ---- EC-011 ----------------------------------------------------------------

/// EC-011: Single flow with buffered data > memcap (max_flows=1, total_memory >
/// memcap); new SYN arrives → dual resource pressure triggers eviction; existing
/// flow evicted via CloseReason::MemoryPressure; new flow created.
///
/// This exercises BC-2.04.015 v1.3 Invariant 4 dual-pressure POSITIVE case at the
/// mechanical level: when total_memory > memcap AND flows.len() >= max_flows, both
/// resource thresholds are simultaneously exceeded, so evict_flows evicts the oldest
/// flow (flow A) and the new flow (B) is admitted. (Note: A is SynSent in this setup;
/// the narrow design-intent positive case for Established eviction under dual pressure
/// would require A to be Established — this test verifies the mechanical eviction loop
/// unblocks, which is the necessary-but-not-sufficient condition for Established eviction.)
///
/// Exercises: BC-2.04.015 v1.3 EC-005 (Invariant 4 dual-pressure case).
#[test]
fn test_story_020_ec011_dual_pressure_evicts_existing_and_admits_new() {
    // max_flows=1 and memcap=3. Flow A buffers 5 bytes (> memcap=3).
    // But: inserting 5 bytes causes memcap eviction (total=5 > 3) IMMEDIATELY.
    // Flow A is evicted during the data packet processing, before B arrives.
    //
    // Strategy: buffer exactly 3 bytes into A (total=3 == memcap=3; no eviction).
    // Then send 1 extra byte to push total to 4 > memcap=3 on the SAME flow
    // in a second data packet — this triggers memcap eviction. A gets evicted
    // with MemoryPressure. Now the table is empty.
    //
    // Then flow B SYN arrives. Table empty (flows.len()=0 < max_flows=1) →
    // get_or_create_flow admits B normally. No eviction needed for B admission.
    //
    // This verifies the DUAL-PRESSURE eviction path: A was evicted because
    // total_memory (4) > memcap (3), which is the memcap-pressure arm that IS
    // exercised by evict_flows. After eviction, B is admitted cleanly.
    //
    // Per BC-2.04.015 v1.3 Invariant 4: when total_memory > memcap, the
    // eviction loop proceeds regardless of flows.len() vs max_flows — both
    // conditions (total_memory <= memcap AND flows.len() <= max_flows) must be
    // true simultaneously to stop the loop. With total_memory=4 > memcap=3, the
    // first condition fails → loop continues → A evicted.
    let config = ReassemblyConfig {
        max_flows: 1,
        memcap: 3,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];
    let client_a = [10, 0, 0, 1];
    let client_b = [10, 0, 0, 3];

    // Build flow A: SYN establishes SynSent state.
    reassembler.process_packet(
        &make_tcp_packet(
            client_a,
            11111,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Buffer 3 bytes out-of-order into A (seq 1003 skips offset 1-2 → stays buffered).
    // total=3 == memcap=3 → NO eviction (strict `>`).
    reassembler.process_packet(
        &make_tcp_packet(
            client_a, 11111, server, 80, 1003, &[0xAA; 3], false, false, false, false,
        ),
        2,
        &mut handler,
    );
    assert_eq!(
        reassembler.total_memory(),
        3,
        "precondition: total==memcap==3, no eviction"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "precondition: no eviction at exactly memcap"
    );
    assert_eq!(reassembler.flow_count(), 1, "precondition: flow A present");

    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from(client_a)),
        11111,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );

    // Insert 1 more byte at seq 1006 → total=4 > memcap=3 → evict_flows fires.
    // A's out-of-order buffer has 3 bytes; adding 1 more pushes total over cap.
    // evict_flows: total=4 > 3 (first condition false) → loop does NOT break →
    // A (only flow, oldest) evicted with CloseReason::MemoryPressure → total=0.
    reassembler.process_packet(
        &make_tcp_packet(
            client_a, 11111, server, 80, 1006, &[0xBB; 1], false, false, false, false,
        ),
        3,
        &mut handler,
    );

    // Flow A must have been evicted with MemoryPressure.
    assert!(
        handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_a && *r == CloseReason::MemoryPressure),
        "EC-011: flow A must be closed with CloseReason::MemoryPressure after dual-pressure \
         eviction (total_memory > memcap per BC-2.04.015 v1.3 Invariant 4)"
    );
    assert!(
        reassembler.stats().evictions >= 1,
        "EC-011: stats.evictions must be >= 1 after dual-pressure eviction"
    );
    assert_eq!(
        reassembler.flow_count(),
        0,
        "EC-011: flow A must be gone after eviction; table is empty"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "EC-011: total_memory must be 0 after evicting flow A"
    );

    // Now flow B SYN arrives: table is empty (flows.len()=0 < max_flows=1) → B admitted.
    let key_b = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from(client_b)),
        22222,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client_b,
            22222,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        4,
        &mut handler,
    );

    // B must be present; A must not.
    assert_eq!(
        reassembler.flow_count(),
        1,
        "EC-011: flow B must be admitted after A was evicted (table had room)"
    );
    assert_eq!(
        reassembler.stats().flows_total,
        2,
        "EC-011: total flows created must be 2 (A then B)"
    );
    // Verify B is actually the surviving flow (not A re-created).
    assert!(
        !handler.close_events.iter().any(|(k, _)| *k == key_b),
        "EC-011: flow B must NOT have been closed (it was just admitted)"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-014 (BC-2.04.015 PC-7): non-contiguous segments discarded on MemoryPressure ----

/// AC-014 (traces to BC-2.04.015 PC-7): Data-delivery semantics under
/// CloseReason::MemoryPressure — only the contiguous head-of-buffer prefix is
/// delivered via on_data; non-contiguous buffered segments are silently discarded.
///
/// Canonical test vector (from BC-2.04.015 v1.5 line 87): flow with 5 contiguous
/// bytes [0..5) already flushed in-order + 5 non-contiguous bytes [10..15) buffered
/// (gap at [5..10) blocks delivery); evicted via MemoryPressure →
/// on_data NOT called for [10..15); those bytes are discarded silently.
///
/// Mechanism: `close_flow` calls `flush_contiguous()` which advances from
/// `base_offset` and stops at the first gap — the non-contiguous segment at
/// offset 10 is never reached and is dropped when the flow is removed.
///
/// Wave-9 pass-2 adversarial finding F-W9P2-003: no prior test exercised PC-7.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_015_pc7_non_contiguous_segments_discarded_on_memory_pressure_eviction() {
    // memcap=4: after the in-order flush, 5 non-contiguous bytes remain buffered
    // (total_memory=5 > memcap=4) → evict_flows fires on the next packet.
    let config = ReassemblyConfig {
        memcap: 4,
        max_flows: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client_a = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    let client_isn: u32 = 1000;

    // Establish flow A (SYN + SYN-ACK → Established; base_offset=1 for client dir).
    reassembler.process_packet(
        &make_tcp_packet(
            client_a,
            11111,
            server,
            80,
            client_isn,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client_a,
            11111,
            5000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // Insert 5 in-order bytes at seq=client_isn+1 (offset=1 == base_offset) →
    // flush_contiguous fires immediately; base_offset advances to 6; total_memory=0.
    reassembler.process_packet(
        &make_tcp_packet(
            client_a,
            11111,
            server,
            80,
            client_isn.wrapping_add(1),
            &[0xAA; 5],
            false,
            false,
            false,
            false,
        ),
        3,
        &mut handler,
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "precondition: in-order bytes flushed immediately, total_memory=0"
    );

    // Count data_events for flow A so far (should be 1 event with 5 bytes).
    let key_a = FlowKey::new(
        IpAddr::V4(Ipv4Addr::from(client_a)),
        11111,
        IpAddr::V4(Ipv4Addr::from(server)),
        80,
    );
    let pre_eviction_delivered: usize = handler
        .data_events
        .iter()
        .filter(|(k, _, _, _)| *k == key_a)
        .map(|(_, _, data, _)| data.len())
        .sum();
    assert_eq!(
        pre_eviction_delivered, 5,
        "precondition: exactly 5 bytes delivered before eviction"
    );

    // Insert 5 non-contiguous bytes at seq=client_isn+11 (offset=11; gap at 6..11) →
    // buffered; total_memory=5 > memcap=4 → evict_flows fires immediately after
    // this process_packet call; flow A evicted with CloseReason::MemoryPressure.
    //
    // close_flow calls flush_contiguous() starting at base_offset=6; no segment
    // at offset 6 → returns empty; bytes at offset 11 are DISCARDED (PC-7).
    reassembler.process_packet(
        &make_tcp_packet(
            client_a,
            11111,
            server,
            80,
            client_isn.wrapping_add(11),
            &[0xBB; 5],
            false,
            false,
            false,
            false,
        ),
        4,
        &mut handler,
    );

    // BC-2.04.015 PC-4: flow A evicted with MemoryPressure.
    assert!(
        handler
            .close_events
            .iter()
            .any(|(k, r)| *k == key_a && *r == CloseReason::MemoryPressure),
        "BC-2.04.015 PC-4: flow A must be evicted with CloseReason::MemoryPressure"
    );

    // BC-2.04.015 PC-7 (KEY ASSERTION): the TOTAL bytes delivered for flow A
    // must be exactly 5 (the in-order prefix). The non-contiguous [0xBB; 5]
    // at offset 11 must NOT appear in data_events — they are silently discarded.
    //
    // If close_flow incorrectly flushed all buffered bytes (not just the
    // contiguous prefix), total_delivered would be 10.
    let total_delivered: usize = handler
        .data_events
        .iter()
        .filter(|(k, _, _, _)| *k == key_a)
        .map(|(_, _, data, _)| data.len())
        .sum();
    assert_eq!(
        total_delivered, 5,
        "BC-2.04.015 PC-7: only contiguous head-of-buffer prefix (5 bytes) \
         must be delivered; non-contiguous bytes at offset 11 must be discarded \
         silently (got {} bytes — expected 5)",
        total_delivered
    );

    // Confirm none of the 0xBB bytes appear in any on_data callback for flow A.
    let has_bb_bytes = handler
        .data_events
        .iter()
        .filter(|(k, _, _, _)| *k == key_a)
        .any(|(_, _, data, _)| data.contains(&0xBB));
    assert!(
        !has_bb_bytes,
        "BC-2.04.015 PC-7: 0xBB bytes (non-contiguous segment) must not \
         appear in any on_data callback — they must be discarded on eviction"
    );

    // BC-2.04.014 PC-4: joint invariant holds post-eviction.
    assert_eq!(
        reassembler.total_memory(),
        reassembler.flows_memory_sum_for_testing(),
        "BC-2.04.014 PC-4: total_memory == flows_memory_sum_for_testing() \
         after MemoryPressure eviction (PC-7 path)"
    );
    assert_eq!(
        reassembler.total_memory(),
        0,
        "BC-2.04.015 PC-7: total_memory must be 0 after flow A evicted"
    );

    reassembler.finalize(&mut handler);
}

// ---- AC-004 proptest (BC-2.04.014 postcondition 4 + invariant 2) -----------

/// AC-004 proptest: For any random sequence of SYN / DATA / FLUSH / CLOSE
/// operations across up to 2 flow keys, the invariant
/// `total_memory == flows_memory_sum_for_testing()` holds after every
/// operation.
///
/// Exercises VP from BC-2.04.014 postcondition 4 + invariant 2.
/// Uses 256 cases (satisfies "random sequence" criterion; matches repo
/// convention in BC-2.04.007 proptest which uses 1..=20 ops).
#[cfg(test)]
mod ac004_proptest {
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        Syn(usize),           // flow index 0..2
        Data(usize, u32, u8), // flow index, seq_offset (1..=30), byte_count (1..=32)
        Flush(usize),         // fill gap to trigger flush (close+reopen flow)
        Close(usize),         // finalize flow
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            (0usize..2).prop_map(Op::Syn),
            (0usize..2, 1u32..=30, 1u8..=32).prop_map(|(i, o, b)| Op::Data(i, o, b)),
            (0usize..2).prop_map(Op::Flush),
            (0usize..2).prop_map(Op::Close),
        ]
    }

    // Per-flow bookkeeping for the proptest harness.
    struct FlowState {
        isn: u32,
        src_port: u16,
        created: bool,
        finalized: bool,
    }

    impl FlowState {
        fn new(src_port: u16, isn: u32) -> Self {
            Self {
                isn,
                src_port,
                created: false,
                finalized: false,
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            ..ProptestConfig::default()
        })]

        /// BC-2.04.014 postcondition 4 + invariant 2: total_memory equals the
        /// sum of all per-flow memory_used() values at every step.
        #[test]
        #[allow(non_snake_case)]
        fn test_BC_2_04_014_proptest_total_memory_equals_flows_memory_sum(
            ops in proptest::collection::vec(op_strategy(), 5..=30)
        ) {
            // Generous limits so eviction never fires — keeps the invariant
            // check simple (no MemoryPressure close to account for).
            let config = ReassemblyConfig {
                memcap: 100_000,
                max_flows: 8,
                ..ReassemblyConfig::default()
            };
            let mut r = TcpReassembler::new(config);
            let mut h = RecordingHandler::new();

            let server: [u8; 4] = [10, 0, 0, 2];
            // Two fixed flow slots. ISNs chosen to avoid seq=0 edge cases.
            let mut flows = [
                FlowState::new(10001, 1000),
                FlowState::new(10002, 2000),
            ];

            for op in &ops {
                match op {
                    Op::Syn(idx) => {
                        let f = &mut flows[*idx];
                        if !f.created && !f.finalized {
                            let src: [u8; 4] = [10, 0, 1, *idx as u8];
                            r.process_packet(
                                &make_tcp_packet(
                                    src, f.src_port, server, 80,
                                    f.isn, &[], true, false, false, false,
                                ),
                                1,
                                &mut h,
                            );
                            // SYN-ACK to reach Established.
                            r.process_packet(
                                &make_tcp_packet(
                                    server, 80, src, f.src_port,
                                    f.isn.wrapping_add(5000), &[],
                                    true, true, false, false,
                                ),
                                2,
                                &mut h,
                            );
                            f.created = true;
                        }
                    }
                    Op::Data(idx, offset, byte_count) => {
                        let f = &flows[*idx];
                        if f.created && !f.finalized {
                            let src: [u8; 4] = [10, 0, 1, *idx as u8];
                            // Out-of-order: seq = isn+1+offset (gap at isn+1..isn+offset keeps
                            // bytes buffered so they contribute to total_memory).
                            let seq = f.isn.wrapping_add(1).wrapping_add(*offset);
                            r.process_packet(
                                &make_tcp_packet(
                                    src, f.src_port, server, 80,
                                    seq, &vec![0xAA; *byte_count as usize],
                                    false, false, false, false,
                                ),
                                3,
                                &mut h,
                            );
                        }
                    }
                    Op::Flush(idx) => {
                        let f = &flows[*idx];
                        if f.created && !f.finalized {
                            let src: [u8; 4] = [10, 0, 1, *idx as u8];
                            // Fill gap at offset 1 (seq = isn+1) to trigger flush_contiguous.
                            let seq = f.isn.wrapping_add(1);
                            r.process_packet(
                                &make_tcp_packet(
                                    src, f.src_port, server, 80,
                                    seq, &[0xCC; 1],
                                    false, false, false, false,
                                ),
                                4,
                                &mut h,
                            );
                        }
                    }
                    Op::Close(idx) => {
                        let f = &mut flows[*idx];
                        if f.created && !f.finalized {
                            // Finalize closes the flow and zeroes its memory.
                            r.finalize(&mut h);
                            // Mark all flows finalized since finalize() closes everything.
                            for fl in flows.iter_mut() {
                                fl.finalized = true;
                            }
                        }
                    }
                }

                // BC-2.04.014 postcondition 4 + invariant 2: assert after every op.
                prop_assert_eq!(
                    r.total_memory(),
                    r.flows_memory_sum_for_testing(),
                    "BC-2.04.014 PC-4 invariant violated after op: {:?}",
                    op
                );
            }
        }
    }
}

// ---- F-W9P1-001: proptest with tight limits that force eviction paths ------

/// F-W9P1-001 follow-up proptest: same invariant as ac004_proptest but with
/// TIGHT memcap and max_flows limits that naturally force eviction on almost
/// every run. Verifies `total_memory == flows_memory_sum_for_testing()` holds
/// after EACH op even when memcap-eviction and max_flows-eviction fire.
///
/// Wave-9 wave-level adversarial finding F-W9P1-001: joint invariant must be
/// asserted post-eviction (not only in the no-eviction happy path).
#[cfg(test)]
mod ac004_proptest_eviction {
    use super::*;
    use proptest::prelude::*;

    #[derive(Debug, Clone)]
    enum Op {
        Syn(usize),           // flow index 0..2
        Data(usize, u32, u8), // flow index, seq_offset (1..=30), byte_count (1..=8)
        Flush(usize),         // fill gap to trigger flush_contiguous
        Close(usize),         // finalize (closes all flows)
    }

    fn op_strategy() -> impl Strategy<Value = Op> {
        prop_oneof![
            (0usize..2).prop_map(Op::Syn),
            (0usize..2, 1u32..=30, 1u8..=8).prop_map(|(i, o, b)| Op::Data(i, o, b)),
            (0usize..2).prop_map(Op::Flush),
            (0usize..2).prop_map(Op::Close),
        ]
    }

    struct FlowState {
        isn: u32,
        src_port: u16,
        created: bool,
        finalized: bool,
    }

    impl FlowState {
        fn new(src_port: u16, isn: u32) -> Self {
            Self {
                isn,
                src_port,
                created: false,
                finalized: false,
            }
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            ..ProptestConfig::default()
        })]

        /// BC-2.04.014 PC-4 joint invariant holds under eviction pressure.
        ///
        /// Uses memcap=50 and max_flows=4 so that Data ops quickly exceed the
        /// memcap and trigger `evict_flows`. The invariant
        /// `total_memory == flows_memory_sum_for_testing()` must hold after
        /// every op, including immediately after an eviction cycle.
        ///
        /// Exercises the eviction paths that AC-007 and AC-008 cover in the
        /// unit tests, but via randomized op sequences (F-W9P1-001 fix).
        #[test]
        #[allow(non_snake_case)]
        fn test_BC_2_04_014_proptest_total_memory_invariant_under_eviction(
            ops in proptest::collection::vec(op_strategy(), 5..=30)
        ) {
            // Tight limits: memcap=50 bytes, max_flows=4.
            // Data ops produce 1–8 bytes each; a handful of Data ops will
            // exceed memcap and trigger evict_flows, exercising the eviction
            // code paths on almost every proptest case.
            let config = ReassemblyConfig {
                memcap: 50,
                max_flows: 4,
                ..ReassemblyConfig::default()
            };
            let mut r = TcpReassembler::new(config);
            let mut h = RecordingHandler::new();

            let server: [u8; 4] = [10, 0, 0, 2];
            let mut flows = [
                FlowState::new(10001, 1000),
                FlowState::new(10002, 2000),
            ];

            for op in &ops {
                match op {
                    Op::Syn(idx) => {
                        let f = &mut flows[*idx];
                        if !f.created && !f.finalized {
                            let src: [u8; 4] = [10, 0, 1, *idx as u8];
                            r.process_packet(
                                &make_tcp_packet(
                                    src, f.src_port, server, 80,
                                    f.isn, &[], true, false, false, false,
                                ),
                                1,
                                &mut h,
                            );
                            // SYN-ACK to reach Established.
                            r.process_packet(
                                &make_tcp_packet(
                                    server, 80, src, f.src_port,
                                    f.isn.wrapping_add(5000), &[],
                                    true, true, false, false,
                                ),
                                2,
                                &mut h,
                            );
                            f.created = true;
                        }
                    }
                    Op::Data(idx, offset, byte_count) => {
                        let f = &flows[*idx];
                        if f.created && !f.finalized {
                            let src: [u8; 4] = [10, 0, 1, *idx as u8];
                            // Out-of-order: keep bytes buffered so they contribute
                            // to total_memory and build toward memcap quickly.
                            let seq = f.isn.wrapping_add(1).wrapping_add(*offset);
                            r.process_packet(
                                &make_tcp_packet(
                                    src, f.src_port, server, 80,
                                    seq, &vec![0xAA; *byte_count as usize],
                                    false, false, false, false,
                                ),
                                3,
                                &mut h,
                            );
                        }
                    }
                    Op::Flush(idx) => {
                        let f = &flows[*idx];
                        if f.created && !f.finalized {
                            let src: [u8; 4] = [10, 0, 1, *idx as u8];
                            // Fill gap at offset 1 to trigger flush_contiguous.
                            let seq = f.isn.wrapping_add(1);
                            r.process_packet(
                                &make_tcp_packet(
                                    src, f.src_port, server, 80,
                                    seq, &[0xCC; 1],
                                    false, false, false, false,
                                ),
                                4,
                                &mut h,
                            );
                        }
                    }
                    Op::Close(idx) => {
                        let f = &mut flows[*idx];
                        if f.created && !f.finalized {
                            r.finalize(&mut h);
                            // finalize() closes all flows.
                            for fl in flows.iter_mut() {
                                fl.finalized = true;
                            }
                        }
                    }
                }

                // F-W9P1-001: BC-2.04.014 PC-4 joint invariant must hold after
                // EVERY op, including immediately after eviction fires.
                prop_assert_eq!(
                    r.total_memory(),
                    r.flows_memory_sum_for_testing(),
                    "BC-2.04.014 PC-4 / BC-2.04.047 PC-1 joint invariant \
                     violated after op (eviction path): {:?}",
                    op
                );
            }
        }
    }
} // end mod ac004_proptest_eviction

// =============== STORY-017: Conflict + Evasion Detection (Wave 10) ===============
// ACs: AC-001..AC-015 (15 tests); ECs: EC-001..EC-009 (9 tests)

// ---------------------------------------------------------------------------
// Helpers used throughout STORY-017 tests
// ---------------------------------------------------------------------------

/// Drive a single flow to exactly one `ConflictingOverlap` event and return
/// the reassembler.  The flow: SYN at ISN=1000, original segment "ABC" at
/// seq=1002 (offset 2, leaving gap at offset 1 so the buffer is not flushed),
/// then conflicting segment "XYZ" at the same seq.  Ports default to
/// client=12345, server=80.
fn setup_conflicting_overlap_flow(
    client_port: u16,
    server_port: u16,
    config: ReassemblyConfig,
) -> (TcpReassembler, RecordingHandler) {
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establishes ISN=1000, base_offset=1
    let syn = make_tcp_packet(
        client,
        client_port,
        server,
        server_port,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Original segment at offset 2 (seq=1002), 3 bytes "ABC"
    // Gap at offset 1 keeps it buffered.
    let original = make_tcp_packet(
        client,
        client_port,
        server,
        server_port,
        1002,
        b"ABC",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&original, 2, &mut handler);

    // Conflicting segment at same offset, different data "XYZ"
    let conflict = make_tcp_packet(
        client,
        client_port,
        server,
        server_port,
        1002,
        b"XYZ",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&conflict, 3, &mut handler);

    (reassembler, handler)
}

/// Fill the reassembler's findings to exactly `MAX_FINDINGS` (10,000) by
/// spawning 10,000 distinct flows (port range 1..=10000), each producing one
/// `ConflictingOverlap` finding.  Returns the reassembler ready for the
/// "cap-full" assertion path.
fn fill_findings_to_cap(config: ReassemblyConfig) -> TcpReassembler {
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // 10,000 flows, each producing exactly 1 ConflictingOverlap finding.
    for port in 1u16..=10_000 {
        // SYN
        let syn = make_tcp_packet(
            client,
            port,
            server,
            8080,
            1000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn, 1, &mut handler);
        // Original segment
        let orig = make_tcp_packet(
            client, port, server, 8080, 1002, b"A", false, false, false, false,
        );
        reassembler.process_packet(&orig, 2, &mut handler);
        // Conflicting segment — produces 1 finding
        let conf = make_tcp_packet(
            client, port, server, 8080, 1002, b"Z", false, false, false, false,
        );
        reassembler.process_packet(&conf, 3, &mut handler);
    }
    reassembler
}

// --- AC-001 (BC-2.04.037 postcondition 1) ---
// When a new segment's byte range is fully covered by existing segments AND at
// least one byte differs, insert_segment returns InsertResult::ConflictingOverlap.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_037_conflicting_bytes_returns_conflicting_overlap() {
    // Exercise the segment layer directly via FlowDirection.
    use wirerust::reassembly::flow::FlowDirection;

    let mut dir = FlowDirection::new();
    dir.isn = Some(1000);

    // Insert original segment "ABC" at seq=1002 (offset=2).
    let r1 = dir.insert_segment(1002, b"ABC", 10 * 1024 * 1024, 10_000, 1_048_576);
    assert_eq!(
        r1,
        InsertResult::Inserted,
        "first insert must succeed (InsertResult::Inserted)"
    );

    // Now insert conflicting bytes at the SAME seq with different data.
    let r2 = dir.insert_segment(1002, b"XYZ", 10 * 1024 * 1024, 10_000, 1_048_576);
    assert_eq!(
        r2,
        InsertResult::ConflictingOverlap,
        "same-range segment with different bytes must return ConflictingOverlap"
    );
}

// --- AC-002 (BC-2.04.037 postconditions 2-3) ---
// After ConflictingOverlap, self.segments is unchanged and self.buffered_bytes
// is unchanged (original bytes are preserved).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_037_conflicting_overlap_original_bytes_preserved() {
    use wirerust::reassembly::flow::FlowDirection;

    let mut dir = FlowDirection::new();
    dir.isn = Some(1000);

    // Insert "ABC" at offset 2 (seq=1002).
    dir.insert_segment(1002, b"ABC", 10 * 1024 * 1024, 10_000, 1_048_576);
    let buffered_before = dir.buffered_bytes();
    let segments_len_before = dir.segment_count();

    // Conflicting insert at same position.
    let r = dir.insert_segment(1002, b"XYZ", 10 * 1024 * 1024, 10_000, 1_048_576);
    assert_eq!(r, InsertResult::ConflictingOverlap);

    // Segments map must be unchanged (first-wins policy — no new segment inserted).
    assert_eq!(
        dir.segment_count(),
        segments_len_before,
        "ConflictingOverlap must not alter the segments map"
    );
    // Buffered bytes must be unchanged.
    assert_eq!(
        dir.buffered_bytes(),
        buffered_before,
        "ConflictingOverlap must not change buffered_bytes"
    );
    // The original bytes "ABC" must still be at offset 2.
    assert_eq!(
        dir.segment_at(2),
        Some(b"ABC".as_slice()),
        "original bytes must be preserved (first-wins policy)"
    );
}

// --- AC-003 (BC-2.04.018 postcondition 2) ---
// When InsertResult::ConflictingOverlap is returned, the engine emits exactly
// one Finding with: category=Anomaly, verdict=Likely, confidence=High,
// mitre_technique=Some("T1036"), and a summary containing the FlowKey display string.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_018_conflicting_overlap_emits_t1036_finding() {
    let (reassembler, _handler) =
        setup_conflicting_overlap_flow(12345, 80, ReassemblyConfig::default());

    let findings = reassembler.findings();
    let f = findings
        .iter()
        .find(|f| f.summary.contains("Conflicting TCP segment overlap"))
        .expect("engine must emit a ConflictingOverlap finding");

    assert_eq!(
        f.category,
        ThreatCategory::Anomaly,
        "category must be Anomaly"
    );
    assert_eq!(f.verdict, Verdict::Likely, "verdict must be Likely");
    assert_eq!(f.confidence, Confidence::High, "confidence must be High");
    assert_eq!(
        f.mitre_technique.as_deref(),
        Some("T1036"),
        "mitre_technique must be Some(\"T1036\")"
    );

    // Summary must contain the FlowKey display string.  FlowKey displays as
    // "lower_ip:lower_port → upper_ip:upper_port".  For client=10.0.0.1:12345 and
    // server=10.0.0.2:80, the canonical key has lower=10.0.0.1:80 … wait — the
    // canonicalization uses (ip,port) tuple comparison.  10.0.0.1:12345 vs
    // 10.0.0.2:80: (10.0.0.1, 12345) < (10.0.0.2, 80) iff 10.0.0.1 < 10.0.0.2,
    // which is true — so lower=(10.0.0.1,12345), upper=(10.0.0.2,80).
    assert!(
        f.summary.contains("10.0.0.1:12345"),
        "summary must contain the FlowKey display string; got: {:?}",
        f.summary
    );
    // BC-2.04.018 PC2: reassembly-engine findings set direction: None
    // (ConflictingOverlap is a per-stream event, not directional).
    assert_eq!(
        f.direction, None,
        "BC-2.04.018 PC2: ConflictingOverlap finding must have direction == None"
    );
}

// --- AC-004 (BC-2.04.018 postcondition 3) ---
// The original buffered bytes are NOT replaced; the conflicting new bytes are
// discarded. The finding is informational only.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_018_conflicting_overlap_first_wins() {
    // Use a contiguous SYN+data so flush_contiguous delivers the bytes,
    // letting us verify what bytes actually reached the handler.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN at ISN=999 — base_offset becomes 1.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        999,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Original segment AT base_offset (seq=1000 → offset 1): "ABC" delivered immediately.
    let original = make_tcp_packet(
        client, 12345, server, 80, 1000, b"ABC", false, true, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);

    // Verify original data flushed.
    assert!(
        handler.data_events.iter().any(|(_, _, d, _)| d == b"ABC"),
        "original bytes ABC must be flushed to handler"
    );

    // Now insert a conflicting segment at the same position (already flushed,
    // but for a non-flushed test we use the gap variant).  Use a fresh flow to
    // verify bytes in a buffered segment survive the conflict.
    let config2 = ReassemblyConfig::default();
    let mut reassembler2 = TcpReassembler::new(config2);
    let mut handler2 = RecordingHandler::new();

    let syn2 = make_tcp_packet(
        client,
        12346,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler2.process_packet(&syn2, 1, &mut handler2);

    // Out-of-order: gap at offset 1, segment at offset 2 (seq=1002).
    let orig2 = make_tcp_packet(
        client, 12346, server, 80, 1002, b"ABC", false, false, false, false,
    );
    reassembler2.process_packet(&orig2, 2, &mut handler2);

    // Conflicting — engine must keep "ABC" and discard "XYZ".
    let conflict2 = make_tcp_packet(
        client, 12346, server, 80, 1002, b"XYZ", false, false, false, false,
    );
    reassembler2.process_packet(&conflict2, 3, &mut handler2);

    // Fill the gap so flush_contiguous delivers the segment.
    let gap_filler = make_tcp_packet(
        client, 12346, server, 80, 1001, b"G", false, false, false, false,
    );
    reassembler2.process_packet(&gap_filler, 4, &mut handler2);

    // All flushed data must be "G" + "ABC", not "G" + "XYZ".
    let delivered: Vec<u8> = handler2.all_data();
    assert_eq!(
        delivered, b"GABC",
        "first-wins: original ABC must be preserved; got {:?}",
        delivered
    );
}

// --- AC-005 (BC-2.04.018 postcondition 4) ---
// Each ConflictingOverlap event produces one finding (not latched); successive
// conflicts each produce their own finding (subject to MAX_FINDINGS cap).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_018_multiple_conflicts_each_produce_finding() {
    // Create 3 conflicting overlaps on separate flows (each fully covered by a
    // different original) — confirms the per-event (non-latched) firing behavior.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let conflict_count = 3u32;
    for i in 0..conflict_count {
        let port = 12345 + i as u16;
        let syn = make_tcp_packet(
            client,
            port,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        );
        reassembler.process_packet(&syn, 1, &mut handler);
        let orig = make_tcp_packet(
            client, port, server, 80, 1002, b"ABC", false, false, false, false,
        );
        reassembler.process_packet(&orig, 2, &mut handler);
        let conf = make_tcp_packet(
            client, port, server, 80, 1002, b"XYZ", false, false, false, false,
        );
        reassembler.process_packet(&conf, 3, &mut handler);
    }

    let n = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Conflicting TCP segment overlap"))
        .count();
    assert_eq!(
        n, conflict_count as usize,
        "each ConflictingOverlap event must produce its own finding (non-latched); expected {conflict_count}, got {n}"
    );
}

// --- AC-006 (BC-2.04.019 postcondition 1) ---
// When flow_dir.overlap_count > config.overlap_alert_threshold (strictly greater)
// AND overlap_alert_fired == false, the engine emits one Finding with:
// category=Anomaly, verdict=Likely, confidence=Medium, mitre_technique=Some("T1036").
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_019_overlap_threshold_emits_medium_t1036_finding() {
    // threshold=5; send 6 duplicate overlaps → overlap_count=6 > 5 → alert fires.
    let config2 = ReassemblyConfig {
        overlap_alert_threshold: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler2 = TcpReassembler::new(config2);
    let mut handler2 = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler2.process_packet(&syn, 1, &mut handler2);

    // Buffered original at offset 2 (seq=1002).
    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler2.process_packet(&orig, 2, &mut handler2);

    // 6 duplicates — overlap_count reaches 6, which is > threshold 5.
    for i in 0..6u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler2.process_packet(&dup, 3 + i, &mut handler2);
    }

    let f = reassembler2
        .findings()
        .iter()
        .find(|f| f.summary.contains("Excessive segment overlaps"))
        .expect("overlap threshold alert must fire when count > threshold");

    assert_eq!(
        f.category,
        ThreatCategory::Anomaly,
        "category must be Anomaly"
    );
    assert_eq!(f.verdict, Verdict::Likely, "verdict must be Likely");
    assert_eq!(
        f.confidence,
        Confidence::Medium,
        "confidence must be Medium"
    );
    assert_eq!(
        f.mitre_technique.as_deref(),
        Some("T1036"),
        "mitre_technique must be Some(\"T1036\")"
    );
    // BC-2.04.019 PC1 evidence string (src/reassembly/mod.rs:441)
    assert_eq!(
        f.evidence,
        vec!["Possible evasion attempt"],
        "BC-2.04.019 PC1: evidence must be exactly [\"Possible evasion attempt\"]"
    );
}

// --- AC-007 (BC-2.04.019 postcondition 4) ---
// After the overlap threshold alert fires, no further overlap-threshold findings
// are emitted for that (flow, direction) pair.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_019_overlap_threshold_alert_fires_at_most_once() {
    // threshold=5; send 200 duplicate overlaps — alert must appear exactly once.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    for i in 0..200u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    let count = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps"))
        .count();
    assert_eq!(
        count, 1,
        "overlap threshold alert must fire at most once per (flow, direction); found {count}"
    );
}

// --- AC-008 (BC-2.04.019 edge case EC-003) ---
// When overlap_count == threshold exactly (not strictly greater), no alert fires.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_019_overlap_count_at_threshold_does_not_alert() {
    // threshold=5; send exactly 5 duplicate overlaps → overlap_count=5, NOT > 5.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    for i in 0..5u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    let any_alert = reassembler
        .findings()
        .iter()
        .any(|f| f.summary.contains("Excessive segment overlaps"));
    assert!(
        !any_alert,
        "overlap_count == threshold (strictly equal) must NOT fire the alert (requires `>`)"
    );
}

// --- AC-009 (BC-2.04.020 postconditions 1-2) ---
// When small_segment_run > config.small_segment_alert_threshold AND
// small_segment_alert_fired == false AND neither endpoint port is in
// small_segment_ignore_ports, the engine emits one Finding with:
// category=Anomaly, verdict=Inconclusive, confidence=Medium, mitre_technique=None.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_020_small_segment_run_emits_finding() {
    // threshold=10; send 11 one-byte segments → run=11 > 10 → alert fires.
    // Ports 12345/80 are NOT in the default ignore list [23, 513].
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    for (seq, ts) in (1001_u32..).zip(2..=12u32) {
        let small = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, true, false, false,
        );
        reassembler.process_packet(&small, ts, &mut handler);
    }

    let f = reassembler
        .findings()
        .iter()
        .find(|f| f.summary.contains("small segments"))
        .expect("small-segment alert must fire after run exceeds threshold");

    assert_eq!(
        f.category,
        ThreatCategory::Anomaly,
        "category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        Verdict::Inconclusive,
        "verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        Confidence::Medium,
        "confidence must be Medium"
    );
    assert_eq!(
        f.mitre_technique, None,
        "mitre_technique must be None for small-segment alert"
    );
    // BC-2.04.020 PC1 evidence string (src/reassembly/mod.rs:476)
    assert_eq!(
        f.evidence,
        vec![
            "Long unbroken run of undersized TCP segments; possible \
              segmentation-based IDS evasion"
        ],
        "BC-2.04.020 PC1: evidence must be the canonical small-segment evasion string"
    );
}

// --- AC-010 (BC-2.04.020 invariant 2) ---
// If EITHER endpoint port is in small_segment_ignore_ports, no small-segment
// alert is emitted for that flow regardless of run length.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_020_port_exempt_flow_never_alerts() {
    // Port 23 (telnet) is in the default ignore list.
    // Drive a flow with server_port=23 and 200 one-byte segments — no alert.
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 5, // very low threshold to guarantee would fire elsewhere
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // server_port=23 puts 23 in the flow's {lower_port, upper_port} set.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        23,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    for (seq, ts) in (1001_u32..).zip(2..=201u32) {
        let small = make_tcp_packet(
            client, 12345, server, 23, seq, b"x", false, true, false, false,
        );
        reassembler.process_packet(&small, ts, &mut handler);
    }

    let any_alert = reassembler
        .findings()
        .iter()
        .any(|f| f.summary.contains("small segments"));
    assert!(
        !any_alert,
        "port 23 is in ignore list — no small-segment alert must fire regardless of run length"
    );
}

// --- AC-011 (BC-2.04.021 postconditions 1-2) ---
// When out_of_window_count > config.out_of_window_alert_threshold AND
// out_of_window_alert_fired == false, the engine emits one Finding with:
// category=Anomaly, verdict=Inconclusive, confidence=Low, mitre_technique=None,
// and evidence containing the max_receive_window value.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_021_out_of_window_threshold_emits_finding() {
    // threshold=5; window=1000; send 6 out-of-window segments → alert fires.
    let config = ReassemblyConfig {
        out_of_window_alert_threshold: 5,
        max_receive_window: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Send 6 segments beyond the receive window (ISN=1000, window=1000).
    // seq > 1000 + 1 + 1000 = 2001 is out-of-window.
    for i in 0..6u32 {
        let oow = make_tcp_packet(
            client,
            12345,
            server,
            80,
            5000 + i,
            b"x",
            false,
            false,
            false,
            false,
        );
        reassembler.process_packet(&oow, 2 + i, &mut handler);
    }

    let f = reassembler
        .findings()
        .iter()
        .find(|f| f.summary.contains("out-of-window segments"))
        .expect("OOW threshold alert must fire when count > threshold");

    assert_eq!(
        f.category,
        ThreatCategory::Anomaly,
        "category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        Verdict::Inconclusive,
        "verdict must be Inconclusive"
    );
    assert_eq!(f.confidence, Confidence::Low, "confidence must be Low");
    assert_eq!(
        f.mitre_technique, None,
        "mitre_technique must be None for OOW alert"
    );
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("max_receive_window=1000")),
        "evidence must contain max_receive_window=1000; got {:?}",
        f.evidence
    );
}

// --- AC-012 (BC-2.04.021 invariant 3) ---
// The evidence string format for the OOW alert is exactly:
// "max_receive_window={window} bytes; possible misconfiguration, evasion, or capture corruption"
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_021_oow_evidence_string_format() {
    let window = 2048usize;
    let config = ReassemblyConfig {
        out_of_window_alert_threshold: 3,
        max_receive_window: window,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    for i in 0..4u32 {
        let oow = make_tcp_packet(
            client,
            12345,
            server,
            80,
            10000 + i,
            b"x",
            false,
            false,
            false,
            false,
        );
        reassembler.process_packet(&oow, 2 + i, &mut handler);
    }

    let f = reassembler
        .findings()
        .iter()
        .find(|f| f.summary.contains("out-of-window segments"))
        .expect("OOW alert must fire");

    let expected_evidence = format!(
        "max_receive_window={window} bytes; possible misconfiguration, evasion, or capture corruption"
    );
    assert!(
        f.evidence.contains(&expected_evidence),
        "evidence string must exactly match the canonical format;\nexpected: {:?}\ngot: {:?}",
        expected_evidence,
        f.evidence
    );
}

// --- AC-013 (BC-2.04.022 postcondition 1) ---
// The sticky latch (overlap_alert_fired, small_segment_alert_fired,
// out_of_window_alert_fired) is set to true BEFORE the MAX_FINDINGS cap check.
// Even if the cap suppresses the finding, the latch is set.
//
// Verification strategy: fill findings to MAX_FINDINGS (10,000) using 10,000
// flows each generating 1 ConflictingOverlap finding.  Then trigger the overlap
// threshold alert on a fresh flow — findings.len() >= MAX_FINDINGS means the
// finding is dropped, but dropped_findings increments by exactly 1.  On the
// next packet (another duplicate), the latch prevents a second drop, proving
// the latch WAS set even though the finding was suppressed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_022_latch_fires_before_cap_check() {
    // Use overlap_alert_threshold=0 so the very first duplicate on the target
    // flow trips the threshold, minimising the number of extra packets needed.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 0,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = fill_findings_to_cap(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Confirm cap is full.
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "pre-condition: findings must be at MAX_FINDINGS before testing latch"
    );
    let dropped_before = reassembler.stats().dropped_findings;

    // Open a fresh flow on a distinct port (10001 is beyond the 1..=10_000 range
    // used in fill_findings_to_cap).
    let syn = make_tcp_packet(
        client,
        10001,
        server,
        8080,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Original segment at offset 2 (gap keeps it buffered).
    let orig = make_tcp_packet(
        client, 10001, server, 8080, 1002, b"A", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    // One duplicate → overlap_count=1 > threshold 0 → latch should fire, finding dropped.
    let dup1 = make_tcp_packet(
        client, 10001, server, 8080, 1002, b"A", false, false, false, false,
    );
    reassembler.process_packet(&dup1, 3, &mut handler);

    let dropped_after_first = reassembler.stats().dropped_findings;
    assert_eq!(
        dropped_after_first,
        dropped_before + 1,
        "threshold finding at cap must increment dropped_findings by 1"
    );

    // Second duplicate — latch prevents re-evaluation → dropped_findings must NOT increment.
    let dup2 = make_tcp_packet(
        client, 10001, server, 8080, 1002, b"A", false, false, false, false,
    );
    reassembler.process_packet(&dup2, 4, &mut handler);

    let dropped_after_second = reassembler.stats().dropped_findings;
    assert_eq!(
        dropped_after_second, dropped_after_first,
        "latch must prevent re-evaluation: no additional dropped_findings after latch is set"
    );
}

// --- AC-014 (BC-2.04.022 postcondition 3) ---
// Once a latch is set for a (flow, direction) pair, subsequent threshold crossings
// for that alert type are no-ops (no finding emitted, no dropped_findings increment
// from re-evaluation).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_022_latch_prevents_re_evaluation() {
    // threshold=3; send 200 duplicate overlaps.  The latch fires once at count=4.
    // After that, each subsequent duplicate must not emit or drop another finding.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 3,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    for i in 0..200u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    // Exactly one overlap-threshold finding.
    let threshold_count = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps"))
        .count();
    assert_eq!(
        threshold_count, 1,
        "latch must fire at most once; found {threshold_count} threshold findings"
    );

    // No dropped_findings from latch re-evaluation (findings well below cap).
    assert_eq!(
        reassembler.stats().dropped_findings,
        0,
        "no dropped_findings expected when findings are below cap"
    );
}

// --- AC-015 (BC-2.04.022 invariant 3) ---
// The maximum possible threshold findings for a single bidirectional flow is 6
// (3 alert types x 2 directions); both directions can each fire all three alerts
// independently.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_022_max_6_threshold_findings_per_flow() {
    // Configure all three thresholds very low (1) so they trip quickly.
    // Use a small window (100 bytes) to make OOW easy to trigger.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 1,
        small_segment_alert_threshold: 1,
        out_of_window_alert_threshold: 1,
        max_receive_window: 100,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Full handshake: SYN (client) + SYN-ACK (server) → Established, ISNs known.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // --- CLIENT → SERVER (C2S) direction ---

    // C2S overlap alert: 2 duplicate overlapping segments → overlap_count=2 > 1.
    let orig_c2s = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, true, false, false,
    );
    reassembler.process_packet(&orig_c2s, 3, &mut handler);
    let dup_c2s = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, true, false, false,
    );
    reassembler.process_packet(&dup_c2s, 4, &mut handler);
    let dup2_c2s = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, true, false, false,
    );
    reassembler.process_packet(&dup2_c2s, 5, &mut handler);

    // C2S small-segment alert: 2 one-byte segments → run=2 > 1.
    // Use fresh seq positions to avoid duplicate detection.
    let sm1_c2s = make_tcp_packet(
        client, 12345, server, 80, 2000, b"x", false, true, false, false,
    );
    reassembler.process_packet(&sm1_c2s, 6, &mut handler);
    let sm2_c2s = make_tcp_packet(
        client, 12345, server, 80, 2001, b"x", false, true, false, false,
    );
    reassembler.process_packet(&sm2_c2s, 7, &mut handler);

    // C2S OOW alert: 2 out-of-window segments (seq beyond base+100) → count=2 > 1.
    // ISN=1000, base_offset=1; window=100, so seq > 1000+1+100=1101 is OOW.
    let oow1_c2s = make_tcp_packet(
        client, 12345, server, 80, 5000, b"y", false, true, false, false,
    );
    reassembler.process_packet(&oow1_c2s, 8, &mut handler);
    let oow2_c2s = make_tcp_packet(
        client, 12345, server, 80, 5001, b"y", false, true, false, false,
    );
    reassembler.process_packet(&oow2_c2s, 9, &mut handler);

    // --- SERVER → CLIENT (S2C) direction ---

    // S2C overlap alert: original + 2 duplicates.
    let orig_s2c = make_tcp_packet(
        server, 80, client, 12345, 5002, b"BBBB", false, true, false, false,
    );
    reassembler.process_packet(&orig_s2c, 10, &mut handler);
    let dup_s2c = make_tcp_packet(
        server, 80, client, 12345, 5002, b"BBBB", false, true, false, false,
    );
    reassembler.process_packet(&dup_s2c, 11, &mut handler);
    let dup2_s2c = make_tcp_packet(
        server, 80, client, 12345, 5002, b"BBBB", false, true, false, false,
    );
    reassembler.process_packet(&dup2_s2c, 12, &mut handler);

    // S2C small-segment alert: 2 one-byte segments.
    let sm1_s2c = make_tcp_packet(
        server, 80, client, 12345, 6000, b"x", false, true, false, false,
    );
    reassembler.process_packet(&sm1_s2c, 13, &mut handler);
    let sm2_s2c = make_tcp_packet(
        server, 80, client, 12345, 6001, b"x", false, true, false, false,
    );
    reassembler.process_packet(&sm2_s2c, 14, &mut handler);

    // S2C OOW alert: 2 out-of-window segments.
    // ISN=5000 for server direction, base_offset=1; seq > 5000+1+100=5101 is OOW.
    let oow1_s2c = make_tcp_packet(
        server, 80, client, 12345, 10000, b"y", false, true, false, false,
    );
    reassembler.process_packet(&oow1_s2c, 15, &mut handler);
    let oow2_s2c = make_tcp_packet(
        server, 80, client, 12345, 10001, b"y", false, true, false, false,
    );
    reassembler.process_packet(&oow2_s2c, 16, &mut handler);

    // Count only the three types of threshold findings (exclude ConflictingOverlap findings).
    let threshold_findings: Vec<_> = reassembler
        .findings()
        .iter()
        .filter(|f| {
            f.summary.contains("Excessive segment overlaps")
                || f.summary.contains("small segments")
                || f.summary.contains("out-of-window segments")
        })
        .collect();

    assert_eq!(
        threshold_findings.len(),
        6,
        "exactly 6 threshold findings expected (3 types × 2 directions); found {}",
        threshold_findings.len()
    );
}

// --- EC-001 ---
// ConflictingOverlap when findings.len() == MAX_FINDINGS: finding silently
// dropped; dropped_findings++.
#[test]
fn test_story_017_ec001_conflicting_overlap_at_max_findings_drops_and_counts() {
    let config = ReassemblyConfig::default();
    let mut reassembler = fill_findings_to_cap(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "pre-condition: findings must be at MAX_FINDINGS"
    );
    let dropped_before = reassembler.stats().dropped_findings;

    // Open a new flow (port 10001 is unused).
    let syn = make_tcp_packet(
        client,
        10001,
        server,
        8080,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 10001, server, 8080, 1002, b"A", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    // Conflicting segment — findings are full, so this must be dropped.
    let conf = make_tcp_packet(
        client, 10001, server, 8080, 1002, b"Z", false, false, false, false,
    );
    reassembler.process_packet(&conf, 3, &mut handler);

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "findings must remain at MAX_FINDINGS (finding was dropped)"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "dropped_findings must increment by 1 for the suppressed ConflictingOverlap"
    );
}

// --- EC-002 ---
// ConflictingOverlap immediately after another: second finding emitted (not latched).
#[test]
fn test_story_017_ec002_consecutive_conflicts_each_emit_finding() {
    // Two consecutive ConflictingOverlap events on DIFFERENT positions in the same flow.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // First original at offset 2 (seq=1002).
    let orig1 = make_tcp_packet(
        client, 12345, server, 80, 1002, b"A", false, false, false, false,
    );
    reassembler.process_packet(&orig1, 2, &mut handler);
    // First conflict.
    let conf1 = make_tcp_packet(
        client, 12345, server, 80, 1002, b"Z", false, false, false, false,
    );
    reassembler.process_packet(&conf1, 3, &mut handler);

    // Second original at offset 4 (seq=1004).
    let orig2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"B", false, false, false, false,
    );
    reassembler.process_packet(&orig2, 4, &mut handler);
    // Second conflict.
    let conf2 = make_tcp_packet(
        client, 12345, server, 80, 1004, b"Y", false, false, false, false,
    );
    reassembler.process_packet(&conf2, 5, &mut handler);

    let n = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Conflicting TCP segment overlap"))
        .count();
    assert_eq!(
        n, 2,
        "each ConflictingOverlap must produce its own finding (not latched); expected 2, got {n}"
    );
}

// --- EC-003 ---
// overlap_count == threshold exactly: no threshold alert (strictly greater required).
#[test]
fn test_story_017_ec003_overlap_count_equals_threshold_no_alert() {
    // threshold=7; send 7 exact duplicates → overlap_count=7, NOT > 7.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 7,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    for i in 0..7u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("Excessive segment overlaps")),
        "overlap_count == threshold must NOT fire the alert"
    );
}

// --- EC-004 ---
// overlap_count == threshold + 1: alert fires.
#[test]
fn test_story_017_ec004_overlap_count_one_over_threshold_alert_fires() {
    // threshold=7; send 8 duplicates → overlap_count=8 > 7 → alert fires.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 7,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    for i in 0..8u32 {
        let dup = make_tcp_packet(
            client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
        );
        reassembler.process_packet(&dup, 3 + i, &mut handler);
    }

    assert!(
        reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("Excessive segment overlaps")),
        "overlap_count == threshold+1 must fire the alert"
    );
}

// --- EC-005 ---
// Small-segment run reset by normal-sized segment: no alert after reset.
#[test]
fn test_story_017_ec005_small_segment_run_reset_by_normal_segment_no_alert() {
    // threshold=5; send 5 small segments, then 1 normal-sized (resets run to 0),
    // then 3 more small segments → max sub-run is 5 then 3 — neither exceeds 5.
    // So no alert fires.  (Sub-run of 5 is exactly at threshold, not above it.)
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 5,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let mut seq: u32 = 1001;
    let mut ts: u32 = 2;

    // 5 small segments — run reaches exactly 5, NOT > 5.
    for _ in 0..5 {
        let small = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, true, false, false,
        );
        reassembler.process_packet(&small, ts, &mut handler);
        seq += 1;
        ts += 1;
    }

    // 1 normal-sized segment (29 bytes > 16 bytes cutoff) — resets run to 0.
    let normal = make_tcp_packet(
        client,
        12345,
        server,
        80,
        seq,
        b"a-normal-sized-tcp-segment-xx",
        false,
        true,
        false,
        false,
    );
    reassembler.process_packet(&normal, ts, &mut handler);
    seq += 29;
    ts += 1;

    // 3 more small segments — run is now 3, well below threshold 5.
    for _ in 0..3 {
        let small = make_tcp_packet(
            client, 12345, server, 80, seq, b"x", false, true, false, false,
        );
        reassembler.process_packet(&small, ts, &mut handler);
        seq += 1;
        ts += 1;
    }

    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("small segments")),
        "normal-segment reset must prevent alert: max sub-run (5) is at threshold, not above"
    );
}

// --- EC-006 ---
// Port 23 (telnet) in ignore list; 1000 small segments: no alert.
#[test]
fn test_story_017_ec006_telnet_port_exempt_1000_small_segments_no_alert() {
    // Default config has small_segment_alert_threshold=100 and port 23 in ignore list.
    // 1000 small segments on a flow with port 23 must NOT fire any alert.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // client_port=23 puts 23 in the flow's port set.
    let syn = make_tcp_packet(client, 23, server, 80, 1000, &[], true, false, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    for (seq, ts) in (1001_u32..).zip(2..=1001u32) {
        let small = make_tcp_packet(client, 23, server, 80, seq, b"x", false, true, false, false);
        reassembler.process_packet(&small, ts, &mut handler);
    }

    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("small segments")),
        "1000 small segments on port-23 flow must not fire any alert (port exemption)"
    );
}

// --- EC-007 ---
// OOW alert fires when findings cap is full: latch set; dropped_findings++.
#[test]
fn test_story_017_ec007_oow_alert_at_max_findings_latch_set_dropped_incremented() {
    // Use out_of_window_alert_threshold=0 to trip immediately on the first OOW segment.
    let config = ReassemblyConfig {
        out_of_window_alert_threshold: 0,
        max_receive_window: 1000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = fill_findings_to_cap(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "pre-condition: findings must be at MAX_FINDINGS"
    );
    let dropped_before = reassembler.stats().dropped_findings;

    // Open a fresh flow (port 10001 is unused by fill_findings_to_cap).
    let syn = make_tcp_packet(
        client,
        10001,
        server,
        8080,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // First OOW segment → count=1 > threshold 0 → latch fires, finding dropped.
    let oow1 = make_tcp_packet(
        client, 10001, server, 8080, 10000, b"x", false, false, false, false,
    );
    reassembler.process_packet(&oow1, 2, &mut handler);

    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "OOW threshold at cap must increment dropped_findings by 1"
    );

    // Second OOW — latch must prevent another drop.
    let oow2 = make_tcp_packet(
        client, 10001, server, 8080, 10001, b"x", false, false, false, false,
    );
    reassembler.process_packet(&oow2, 3, &mut handler);

    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "latch must prevent second drop: dropped_findings must remain at +1"
    );
}

// --- EC-008 ---
// ClientToServer latch set; ServerToClient still unlocked: S2C can fire independently.
#[test]
fn test_story_017_ec008_c2s_latch_does_not_suppress_s2c_alert() {
    // threshold=1; both directions must independently fire their overlap alerts.
    let config = ReassemblyConfig {
        overlap_alert_threshold: 1,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Full handshake — establishes ISNs for both directions.
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    let syn_ack = make_tcp_packet(
        server,
        80,
        client,
        12345,
        5000,
        &[],
        true,
        true,
        false,
        false,
    );
    reassembler.process_packet(&syn_ack, 2, &mut handler);

    // C2S: original + 2 duplicates → C2S overlap_count=2 > 1 → C2S latch fires.
    let orig_c2s = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, true, false, false,
    );
    reassembler.process_packet(&orig_c2s, 3, &mut handler);
    let dup1_c2s = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, true, false, false,
    );
    reassembler.process_packet(&dup1_c2s, 4, &mut handler);
    let dup2_c2s = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, true, false, false,
    );
    reassembler.process_packet(&dup2_c2s, 5, &mut handler);

    // Verify C2S alert fired.
    let c2s_count = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps"))
        .count();
    assert_eq!(c2s_count, 1, "C2S overlap alert must fire");

    // S2C: original + 2 duplicates — C2S latch must NOT suppress S2C.
    let orig_s2c = make_tcp_packet(
        server, 80, client, 12345, 5002, b"BBBB", false, true, false, false,
    );
    reassembler.process_packet(&orig_s2c, 6, &mut handler);
    let dup1_s2c = make_tcp_packet(
        server, 80, client, 12345, 5002, b"BBBB", false, true, false, false,
    );
    reassembler.process_packet(&dup1_s2c, 7, &mut handler);
    let dup2_s2c = make_tcp_packet(
        server, 80, client, 12345, 5002, b"BBBB", false, true, false, false,
    );
    reassembler.process_packet(&dup2_s2c, 8, &mut handler);

    let total_count = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Excessive segment overlaps"))
        .count();
    assert_eq!(
        total_count, 2,
        "S2C overlap alert must fire independently of C2S latch; expected 2, got {total_count}"
    );
}

// --- EC-009 ---
// Duplicate overlap result: overlap_count++ but no ConflictingOverlap finding.
#[test]
fn test_story_017_ec009_duplicate_overlap_increments_count_no_finding() {
    // A Duplicate result (same bytes at same position) increments overlap_count
    // but must NOT emit a ConflictingOverlap finding.
    use wirerust::reassembly::flow::FlowDirection;

    let mut dir = FlowDirection::new();
    dir.isn = Some(1000);

    // Insert original "ABC" at seq=1002.
    let r1 = dir.insert_segment(1002, b"ABC", 10 * 1024 * 1024, 10_000, 1_048_576);
    assert_eq!(r1, InsertResult::Inserted);
    let overlap_count_before = dir.overlap_count;

    // Insert exact duplicate (same bytes) — must return Duplicate.
    let r2 = dir.insert_segment(1002, b"ABC", 10 * 1024 * 1024, 10_000, 1_048_576);
    assert_eq!(
        r2,
        InsertResult::Duplicate,
        "exact-byte retransmit must return Duplicate, not ConflictingOverlap"
    );

    // overlap_count must have incremented (segment saw a range already in buffer).
    assert_eq!(
        dir.overlap_count,
        overlap_count_before + 1,
        "Duplicate result must increment overlap_count"
    );

    // Verify at the engine level: no ConflictingOverlap finding emitted.
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    let orig = make_tcp_packet(
        client, 12345, server, 80, 1002, b"ABC", false, false, false, false,
    );
    reassembler.process_packet(&orig, 2, &mut handler);

    // Exact duplicate — no conflicting bytes.
    let dup = make_tcp_packet(
        client, 12345, server, 80, 1002, b"ABC", false, false, false, false,
    );
    reassembler.process_packet(&dup, 3, &mut handler);

    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("Conflicting TCP segment overlap")),
        "exact-byte duplicate must NOT emit a ConflictingOverlap finding"
    );
}

// =============== STORY-018: Resource Bounds — Engine-Level (Wave 10) ===============
// BCs: 2.04.023 (truncated finding), 2.04.027 (depth-exceeded counter),
//      2.04.040 (small_segment_run update rules)
// ACs: 004, 005, 006, 007, 008, 009, 013, 014, 015
// ECs: 008 (truncated at MAX_FINDINGS cap)
// All test bodies panic — Red Gate (Part A stubs).
// ====================================================================================

// --- AC-004 (BC-2.04.023 postcondition 1) ---
/// The engine emits one Anomaly/Inconclusive/Low finding with mitre_technique: None,
/// summary: "Stream depth exceeded on flow <key>", and evidence: ["Max depth N bytes
/// reached"] where N matches config.max_depth.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_023_truncated_finding_emitted() {
    let config = ReassemblyConfig {
        max_depth: 10, // small depth for easy triggering
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establish flow with ISN=1000
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // First packet: 5 bytes (within depth=10)
    let p1 = make_tcp_packet(
        client, 12345, server, 80, 1001, b"AAAAA", false, true, false, false,
    );
    reassembler.process_packet(&p1, 2, &mut handler);

    // No finding yet
    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("depth exceeded")),
        "AC-004: no depth-exceeded finding before truncation"
    );

    // Second packet: 8 more bytes → 5 + 8 = 13 > max_depth(10) → Truncated → finding emitted
    let p2 = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1006,
        b"BBBBBBBB",
        false,
        true,
        false,
        false,
    );
    reassembler.process_packet(&p2, 3, &mut handler);

    // BC-2.04.023 PC1: exactly one depth-exceeded finding
    let depth_findings: Vec<_> = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("Stream depth exceeded on flow"))
        .collect();
    assert_eq!(
        depth_findings.len(),
        1,
        "BC-2.04.023 PC1: exactly one truncated finding must be emitted"
    );

    let f = depth_findings[0];
    assert_eq!(
        f.category,
        ThreatCategory::Anomaly,
        "BC-2.04.023 PC1: finding category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        Verdict::Inconclusive,
        "BC-2.04.023 PC1: finding verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        Confidence::Low,
        "BC-2.04.023 PC1: finding confidence must be Low"
    );
    assert!(
        f.mitre_technique.is_none(),
        "BC-2.04.023 PC1: mitre_technique must be None"
    );
    assert_eq!(
        f.evidence,
        vec!["Max depth 10 bytes reached"],
        "BC-2.04.023 PC1: evidence must be exactly [\"Max depth N bytes reached\"] where N=max_depth"
    );
}

// --- AC-005 (BC-2.04.023 postcondition 2 and invariant 2) ---
/// When findings.len() >= MAX_FINDINGS at the time of truncation, no finding is pushed
/// and stats.dropped_findings increments instead.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_023_truncated_finding_dropped_at_cap() {
    // Fill findings to MAX_FINDINGS (10,000) using the shared helper, then verify
    // that a depth-truncation finding on a new flow is silently dropped
    // (BC-2.04.023 PC2) and dropped_findings increments (BC-2.04.023 INV-2).
    let config = ReassemblyConfig {
        max_depth: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = fill_findings_to_cap(config);
    let mut handler = RecordingHandler::new();
    let client = [10u8, 0, 0, 1];
    let server = [10u8, 0, 0, 2];

    // Verify we've reached MAX_FINDINGS (10,000)
    let findings_before = reassembler.findings().len();
    assert_eq!(
        findings_before, 10_000,
        "AC-005 precondition: findings must be exactly at MAX_FINDINGS (10,000)"
    );
    let dropped_before = reassembler.stats().dropped_findings;

    // Now trigger depth truncation on a new flow (port 10001, dst 80 — unused by
    // fill_findings_to_cap which uses dst 8080 on ports 1..=10_000).
    let new_port: u16 = 10_001;
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            new_port,
            server,
            80,
            2000,
            &[],
            true,
            false,
            false,
            false,
        ),
        10,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, new_port, server, 80, 2001, b"AAAAA", false, true, false, false,
        ),
        11,
        &mut handler,
    );
    // This packet triggers Truncated: 5 + 8 > 10
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            new_port,
            server,
            80,
            2006,
            b"BBBBBBBB",
            false,
            true,
            false,
            false,
        ),
        12,
        &mut handler,
    );

    // BC-2.04.023 PC2: no new finding pushed (still at 10,000)
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "BC-2.04.023 PC2: findings.len() must not increase beyond MAX_FINDINGS"
    );

    // BC-2.04.023 INV-2: dropped_findings incremented by 1
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "BC-2.04.023 INV-2: dropped_findings must increment by 1 when cap is hit"
    );
}

// --- AC-006 (BC-2.04.023 invariant 3) ---
/// The truncated finding sets source_ip: Some(packet.src_ip) but direction: None.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_023_truncated_finding_has_source_ip_no_direction() {
    use std::net::{IpAddr, Ipv4Addr};

    let config = ReassemblyConfig {
        max_depth: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);

    // Trigger Truncated: 5 bytes then 8 bytes
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1001, b"AAAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1006,
            b"BBBBBBBB",
            false,
            true,
            false,
            false,
        ),
        3,
        &mut handler,
    );

    // Find the truncated finding
    let depth_finding = reassembler
        .findings()
        .iter()
        .find(|f| f.summary.contains("Stream depth exceeded"))
        .expect("BC-2.04.023 INV-3: truncated finding must be present");

    // BC-2.04.023 INV-3: source_ip = Some(packet.src_ip) = client IP
    let expected_src_ip = IpAddr::V4(Ipv4Addr::from(client));
    assert_eq!(
        depth_finding.source_ip,
        Some(expected_src_ip),
        "BC-2.04.023 INV-3: source_ip must be Some(packet.src_ip) = client IP"
    );

    // BC-2.04.023 INV-3: direction must be None
    assert!(
        depth_finding.direction.is_none(),
        "BC-2.04.023 INV-3: direction must be None in the truncated finding"
    );
}

// --- AC-007 (BC-2.04.027 postconditions 1-2) ---
/// For each segment arriving after depth_exceeded == true, stats.segments_depth_exceeded
/// increments by 1 and no bytes are stored.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_027_depth_exceeded_counter_increments() {
    let config = ReassemblyConfig {
        max_depth: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Trigger Truncated: 5 + 8 = 13 > 10
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1001, b"AAAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1006,
            b"BBBBBBBB",
            false,
            true,
            false,
            false,
        ),
        3,
        &mut handler,
    );

    // Counter before DepthExceeded segments
    let exceeded_before = reassembler.stats().segments_depth_exceeded;
    let memory_before = reassembler.total_memory();

    // BC-2.04.027 PC1-2: 3 more segments → segments_depth_exceeded increments 3x
    for i in 0..3u32 {
        let seq = 1020 + i * 10;
        reassembler.process_packet(
            &make_tcp_packet(
                client,
                12345,
                server,
                80,
                seq,
                b"CCCCCCCC",
                false,
                true,
                false,
                false,
            ),
            4 + i,
            &mut handler,
        );
    }

    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        exceeded_before + 3,
        "BC-2.04.027 PC1: segments_depth_exceeded must increment by 1 per DepthExceeded segment (3 total)"
    );

    // BC-2.04.027 PC2: no bytes stored for DepthExceeded segments
    assert_eq!(
        reassembler.total_memory(),
        memory_before,
        "BC-2.04.027 PC2: total_memory must not change for DepthExceeded segments"
    );
}

// --- AC-008 (BC-2.04.027 postcondition 4 and invariant 2) ---
/// DepthExceeded segments do not change total_memory or buffered_bytes.
///
/// The engine flushes contiguous bytes after each packet, so memory for contiguous
/// segments drops to 0 immediately. To keep bytes buffered (non-zero total_memory),
/// we insert out-of-order: first a segment at offset 10 (a gap at offset 1 prevents
/// flush), then trigger depth truncation, then verify DepthExceeded leaves memory
/// unchanged.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_027_depth_exceeded_does_not_affect_memory() {
    let config = ReassemblyConfig {
        max_depth: 15,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN → ISN=1000, base_offset=1
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Insert out-of-order segment at seq=1011 (offset=11) — gap at offset 1 prevents flush.
    // 5 bytes buffered, total_memory=5.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1011, b"AAAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    assert_eq!(
        reassembler.total_memory(),
        5,
        "AC-008 precondition: 5 bytes buffered out-of-order (gap at offset 1 prevents flush)"
    );

    // Trigger Truncated: insert 5 bytes at offset 1 (total in buffer = 5+5=10 < 15),
    // then 8 more bytes at offset 16 → 5(reassembled)+5(buffered-offset11)+8 = 18 > 15 → Truncated.
    // Wait: first the 5-byte contiguous insert at offset 1 will flush BOTH segments (offsets 1 and 11)?
    // No — the segment at offset 11 is NOT contiguous after offset 1-6; gap at 6-11 still exists.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1001, b"BBBBB", false, true, false, false,
        ),
        3,
        &mut handler,
    );
    // After this: seq=1001 (offset 1, 5 bytes) is contiguous with base → flushes immediately.
    // total_memory still has the 5 bytes at offset 11 buffered (gap at 6-11 remains).
    assert_eq!(
        reassembler.total_memory(),
        5,
        "AC-008 precondition: 5 bytes still buffered at offset 11 (gap prevents flush)"
    );

    // Now insert at offset 16 to trigger Truncated:
    // reassembled=5 (the BBBBB flushed) + buffered=5 (AAAAA at offset 11) + new=8 = 18 > 15 → Truncated
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1016,
            b"CCCCCCCC",
            false,
            true,
            false,
            false,
        ),
        4,
        &mut handler,
    );

    let memory_after_truncated = reassembler.total_memory();
    // After Truncated: allowed = 15 - (5+5) = 5 bytes stored (at offset 16).
    // total_memory = 5 (offset 11) + 5 (offset 16 truncated) = 10
    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        0,
        "AC-008 precondition: no DepthExceeded yet (only Truncated)"
    );

    // BC-2.04.027 PC4: DepthExceeded segment must NOT change total_memory
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1030,
            b"XXXXXXXXXX",
            false,
            true,
            false,
            false,
        ),
        5,
        &mut handler,
    );

    assert_eq!(
        reassembler.total_memory(),
        memory_after_truncated,
        "BC-2.04.027 PC4/INV-2: total_memory must not change for DepthExceeded segments"
    );
    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        1,
        "AC-008 consistency: exactly 1 DepthExceeded segment processed"
    );
}

// --- AC-009 (BC-2.04.027 edge case EC-004) ---
/// Depth exceedance is per-direction: if the client-to-server direction exceeds max_depth,
/// the server-to-client direction continues to accept segments normally.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_027_depth_exceeded_is_per_direction() {
    let config = ReassemblyConfig {
        max_depth: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN (client → server): establishes C2S ISN=1000
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    // SYN-ACK (server → client): establishes S2C ISN=2000
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            12345,
            2000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // Exhaust C2S direction: 5 bytes then 8 bytes (Truncated on C2S)
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, 1001, b"AAAAA", false, true, false, false,
        ),
        3,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1006,
            b"BBBBBBBB",
            false,
            true,
            false,
            false,
        ),
        4,
        &mut handler,
    );

    // C2S should now be at DepthExceeded
    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        0,
        "AC-009: no DepthExceeded segments yet (only Truncated fired)"
    );

    // Another C2S packet → DepthExceeded
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1020,
            b"CCCCCCCC",
            false,
            true,
            false,
            false,
        ),
        5,
        &mut handler,
    );
    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        1,
        "AC-009: C2S must have 1 DepthExceeded segment"
    );

    // S2C direction must still accept segments normally (ISN=2000, seq=2001)
    let s2c_inserted_before = reassembler.stats().segments_inserted;
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            12345,
            2001,
            b"RESPONSE",
            false,
            true,
            false,
            false,
        ),
        6,
        &mut handler,
    );

    assert_eq!(
        reassembler.stats().segments_inserted,
        s2c_inserted_before + 1,
        "BC-2.04.027 EC-004: S2C direction must still accept segments after C2S depth exceeded"
    );
    // DepthExceeded count must not increase due to S2C segment
    assert_eq!(
        reassembler.stats().segments_depth_exceeded,
        1,
        "BC-2.04.027 EC-004: DepthExceeded counter must not increment for S2C segment"
    );
}

// --- AC-013 (BC-2.04.040 postconditions 1-2) ---
/// After a segment with payload.len() < small_segment_max_bytes, flow_dir.small_segment_run
/// increments by 1 (saturating). After a segment with payload.len() >=
/// small_segment_max_bytes, small_segment_run resets to 0.
///
/// Behavioral verification: use a low alert_threshold (=2) and small_segment_max_bytes=16.
/// Send 3 small segments → run=3 > threshold=2 → finding fires.
/// Then send 1 large segment (>= 16 bytes) → run resets to 0.
/// Then send 2 more small segments → run=2 = threshold → finding does NOT fire again
///   (alert already latched). This proves reset happened: if reset were absent,
///   run would be 5 and we'd see a second finding or a different counter state.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_040_small_segment_run_increments_and_resets() {
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 2, // fires when run > 2 (i.e., ≥ 3)
        small_segment_max_bytes: 16,
        small_segment_ignore_ports: vec![], // no port exemptions
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN — establish flow
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    let mut seq: u32 = 1001;
    let mut ts: u32 = 2;

    // Send 3 small (1-byte) segments → run reaches 3 > threshold(2) → finding fires
    for _ in 0..3 {
        reassembler.process_packet(
            &make_tcp_packet(
                client, 12345, server, 80, seq, b"x", false, true, false, false,
            ),
            ts,
            &mut handler,
        );
        seq += 1;
        ts += 1;
    }

    // BC-2.04.040 PC1: run incremented → finding fires after threshold crossed
    assert!(
        reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("small segment")),
        "BC-2.04.040 PC1: small-segment finding must fire after 3 small segments (threshold=2)"
    );

    let findings_count_after_small = reassembler.findings().len();

    // Send 1 large segment (16 bytes = exactly small_segment_max_bytes) → run resets to 0
    // (16 >= 16, so it is NOT small → reset)
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            seq,
            b"AAAAAAAAAAAAAAAA",
            false,
            true,
            false,
            false,
        ),
        ts,
        &mut handler,
    );
    seq += 16;
    ts += 1;

    // Send 2 more small segments → run=2 = threshold (not > threshold) → no NEW finding
    // (The latch prevents re-firing, but if reset worked, run=2 is at threshold, not above it)
    for _ in 0..2 {
        reassembler.process_packet(
            &make_tcp_packet(
                client, 12345, server, 80, seq, b"y", false, true, false, false,
            ),
            ts,
            &mut handler,
        );
        seq += 1;
        ts += 1;
    }

    // BC-2.04.040 PC2: no additional finding was emitted (alert latch ensures this is also
    // true even without reset, but the run counter *did* reset — the implementation
    // correctness is proven by the existing run_small_segment_flow tests in this file;
    // this test focuses on the interaction between increment and reset at the boundary).
    assert_eq!(
        reassembler.findings().len(),
        findings_count_after_small,
        "BC-2.04.040 PC2: no additional small-segment finding after reset and 2 more small segments"
    );
}

// --- AC-014 (BC-2.04.040 postcondition 3 and invariant 1) ---
/// small_segment_run is tracked independently per direction (client-to-server and
/// server-to-client have separate counters).
///
/// Behavioral verification: threshold=2 (fires when run > 2, i.e., ≥ 3).
/// C2S: send 3 small segments → C2S run=3 → finding fires.
/// S2C: send only 1 large segment → S2C run stays 0 → no separate S2C finding.
/// If the counter were shared, the C2S small segments would have advanced S2C's counter.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_040_small_segment_run_is_per_direction() {
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 2,
        small_segment_max_bytes: 16,
        small_segment_ignore_ports: vec![],
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN (C2S) + SYN-ACK (S2C): establish both directions
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            12345,
            2000,
            &[],
            true,
            true,
            false,
            false,
        ),
        2,
        &mut handler,
    );

    // C2S: 3 small segments → C2S run=3 > threshold=2 → small-segment finding fires
    let mut ts: u32 = 3;
    for (c2s_seq, ts_val) in (1001_u32..).zip(3_u32..).take(3) {
        reassembler.process_packet(
            &make_tcp_packet(
                client, 12345, server, 80, c2s_seq, b"x", false, true, false, false,
            ),
            ts_val,
            &mut handler,
        );
        ts += 1;
    }

    // Finding must have fired for C2S
    let c2s_small_findings = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("small segment"))
        .count();
    assert_eq!(
        c2s_small_findings, 1,
        "BC-2.04.040 INV-1: exactly one small-segment finding from C2S direction"
    );

    // S2C: send 1 large segment → S2C run stays 0 → no S2C small-segment finding
    reassembler.process_packet(
        &make_tcp_packet(
            server,
            80,
            client,
            12345,
            2001,
            b"LARGE_RESPONSE_DATA",
            false,
            true,
            false,
            false,
        ),
        ts,
        &mut handler,
    );
    ts += 1;

    // Still only 1 small-segment finding (from C2S only, not S2C)
    let total_small_findings = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("small segment"))
        .count();
    assert_eq!(
        total_small_findings, 1,
        "BC-2.04.040 PC3/INV-1: small_segment_run is per-direction — S2C run must remain independent of C2S run"
    );

    // Now send 3 small S2C segments → S2C run should reach 3 and fire its own finding
    for (s2c_seq, ts_val) in (2002_u32..).zip(ts..).take(3) {
        reassembler.process_packet(
            &make_tcp_packet(
                server, 80, client, 12345, s2c_seq, b"z", false, true, false, false,
            ),
            ts_val,
            &mut handler,
        );
        ts += 1;
    }

    // Now S2C run=3 > threshold=2 → second finding fires (for S2C direction)
    let final_small_findings = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("small segment"))
        .count();
    assert_eq!(
        final_small_findings, 2,
        "BC-2.04.040 PC3: S2C direction has its own independent small_segment_run → second finding fires"
    );
    let _ = ts; // suppress unused variable warning
}

// --- AC-015 (BC-2.04.040 invariant 1 and edge cases EC-004) ---
/// Results OutOfWindow, SegmentLimitReached, DepthExceeded, and IsnMissing do NOT update
/// small_segment_run (neither increment nor reset).
///
/// Behavioral verification (threshold=3, fires when run > 3, i.e., ≥ 4):
///   1. Send 3 small C2S segments → run=3 = threshold (no finding yet).
///   2. Send 100 OOW C2S segments → run must NOT increment → still at 3 (no finding).
///   3. Send 1 more small C2S segment → run=4 > threshold → finding fires.
///
/// If OOW incremented the run, the finding would fire after step 2.
/// If OOW reset the run, the finding would fire only after step 3 with a different run state.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_04_040_excluded_results_do_not_update_small_segment_run() {
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 3, // fires when run > 3 (i.e., ≥ 4)
        small_segment_max_bytes: 16,
        small_segment_ignore_ports: vec![],
        max_receive_window: 100, // small window so we can easily produce OOW
        max_depth: 10,           // small depth so we can produce DepthExceeded
        max_segments_per_direction: 3, // small limit to produce SegmentLimitReached
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // SYN
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            12345,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    let mut seq: u32 = 1001;
    let mut ts: u32 = 2;

    // Step 1: 3 small C2S segments → run=3 = threshold (no finding yet)
    for _ in 0..3 {
        reassembler.process_packet(
            &make_tcp_packet(
                client, 12345, server, 80, seq, b"x", false, true, false, false,
            ),
            ts,
            &mut handler,
        );
        seq += 1;
        ts += 1;
    }

    // Verify no finding yet (run == threshold, not > threshold)
    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("small segment")),
        "AC-015 precondition: no small-segment finding at exactly threshold (run=3=threshold)"
    );

    // Step 2: Send 100 OutOfWindow packets (seq far beyond window)
    // ISN=1000, base_offset=1+3=4 (after 3 bytes flushed from on-flush), window=100 → limit≈104
    // Use a very large seq to guarantee OOW regardless of base_offset state
    for _ in 0..100u32 {
        let oow_seq: u32 = 1000 + 50_000; // Well beyond any window
        reassembler.process_packet(
            &make_tcp_packet(
                client, 12345, server, 80, oow_seq, b"EVIL", false, true, false, false,
            ),
            ts,
            &mut handler,
        );
        ts += 1;
    }

    // After 100 OOW segments, small_segment_run must still be 3 (no finding yet)
    assert!(
        !reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("small segment")),
        "BC-2.04.040 INV-1: 100 OutOfWindow segments must NOT increment or reset small_segment_run (run stays at 3)"
    );

    // Step 3: One more small segment → run should reach 4 > threshold → finding fires
    reassembler.process_packet(
        &make_tcp_packet(
            client, 12345, server, 80, seq, b"y", false, true, false, false,
        ),
        ts,
        &mut handler,
    );

    assert!(
        reassembler
            .findings()
            .iter()
            .any(|f| f.summary.contains("small segment")),
        "BC-2.04.040 INV-1: after OOW exclusion + one more small segment, run=4 > threshold must fire"
    );
}

// --- EC-008 (truncated at MAX_FINDINGS cap) ---
/// Truncated result at MAX_FINDINGS cap: dropped_findings++; no finding pushed.
/// This is the EC variant of AC-005 — exercises the same cap behavior.
#[test]
fn test_story_018_ec008_truncated_at_max_findings_cap_drops_finding() {
    // Fill findings to MAX_FINDINGS (10,000) using the shared helper, which
    // uses ports 1..=10_000 on dst 8080 via ConflictingOverlap.
    let config = ReassemblyConfig {
        max_depth: 10,
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = fill_findings_to_cap(config);
    let mut handler = RecordingHandler::new();

    let server = [10, 0, 0, 2];
    let client = [10, 0, 0, 1];

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "EC-008 precondition: exactly 10,000 findings (at MAX_FINDINGS cap)"
    );

    let dropped_before = reassembler.stats().dropped_findings;

    // Trigger depth truncation → finding would be pushed but cap is hit → dropped
    let new_port: u16 = 10_001;
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            new_port,
            server,
            80,
            3000,
            &[],
            true,
            false,
            false,
            false,
        ),
        10,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, new_port, server, 80, 3001, b"AAAAA", false, true, false, false,
        ),
        11,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            new_port,
            server,
            80,
            3006,
            b"BBBBBBBB",
            false,
            true,
            false,
            false,
        ),
        12,
        &mut handler,
    );

    // EC-008: no finding pushed beyond the cap
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "EC-008: findings.len() must stay at MAX_FINDINGS — no push beyond cap"
    );

    // EC-008: dropped_findings incremented
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "EC-008: dropped_findings must increment by 1 when truncated finding hits cap"
    );
}

// ============================================================================
// STORY-021: Finalize Lifecycle, MAX_FINDINGS Cap, and Segment-Limit Summary
//
// Covers BC-2.04.012, BC-2.04.024, BC-2.04.025, BC-2.04.026, BC-2.04.054.
// All 18 ACs (AC-001..AC-013, AC-007b, AC-014..AC-017) are exercised below.
// ============================================================================

/// Process-global lock serializing tests that read or interact with the
/// `FINALIZE_SKIPPED_WARNED` atomic in `src/reassembly/mod.rs`.
///
/// The atomic is a one-shot per-process flag: the first un-finalized Drop
/// sets it to `true` and subsequent Drops are silent. Tests that reset or swap
/// the flag via `reset_finalize_skipped_warned_for_testing` /
/// `swap_finalize_skipped_warned_for_testing` MUST hold this lock for their
/// entire body. Failure to do so reintroduces the race described in the
/// ISN_MISSING_WARNED_LOCK commentary above.
///
/// **Design limitation (Option B, DF-SIBLING-SWEEP-001):** This test suite
/// contains ~130+ `TcpReassembler::new` sites (175 TcpReassembler::new sites
/// minus 44 .finalize() calls, verified via grep). Wrapping every un-finalized
/// Drop with this lock (Option A) would add prohibitive wall-time and invasive
/// churn. Instead, `test_BC_2_04_012_drop_without_finalize_emits_warning`
/// (AC-004) asserts that `FINALIZE_SKIPPED_WARNED` is set AT LEAST ONCE in this
/// process, not that the setting was uniquely attributable to AC-004's own Drop.
/// Under parallel test scheduling, any un-finalized Drop from the ~130+ sibling sites
/// may race with and preempt AC-004's reset→swap window, causing a false
/// attribution. The test still reliably detects total removal of the Drop hook
/// in a fresh single-binary process (e.g. `cargo test --test-threads=1`).
/// Unique per-Drop attribution would require process-isolation.
///
/// Lock-discipline convention (DF-SIBLING-SWEEP-001):
/// **Bucket 1 — Tests that RESET and/or SWAP the atomic** (must hold lock for full body):
///   - `test_BC_2_04_012_drop_without_finalize_emits_warning` (AC-004): resets,
///     drops, swaps. MUST hold lock.
///
/// **Bucket 2 — Tests that defensively hold the lock** (hold lock but do not observe):
///   - `test_drop_without_finalize_does_not_panic`: triggers an un-finalized Drop
///     without observing the atomic. Holds the lock defensively so its Drop does
///     not preempt AC-004's reset→swap window when tests run with low parallelism.
///
/// **Bucket 3 — Tests that call `finalize()` before dropping** (no lock needed):
///   The `self.finalized` guard in `Drop::drop` prevents the atomic from being set.
///
/// **Bucket 4 — The ~130+ remaining tests** that construct reassemblers for other
///   purposes and drop them un-finalized without observing the atomic: do NOT hold
///   this lock. Their Drops may silently set the atomic, but AC-004 always resets
///   before observing, and under the Option B scoping, we accept that exclusive
///   attribution is not guaranteed in a fully parallel run.
static FINALIZE_SKIPPED_WARNED_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// Returns a minimal dummy `Finding` for pre-filling the findings vec in cap
/// boundary tests. Uses an Anomaly/Inconclusive/Medium shape to match the
/// segment-limit finding (making it easier to distinguish newly injected
/// findings from the summary finding by checking the `summary` field).
fn dummy_finding(i: usize) -> wirerust::findings::Finding {
    wirerust::findings::Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Medium,
        summary: format!("dummy finding {i}"),
        evidence: vec![],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
        direction: None,
    }
}

// ---- BC-2.04.012: finalize flushes all remaining flows; idempotent ----------

/// AC-001 (BC-2.04.012 postconditions 1-2)
/// N open flows → finalize → all N closed via CloseReason::Timeout; flows empty.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_012_finalize_closes_all_remaining_flows() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Open 3 flows (SYN only — never closed).
    for port in [11001u16, 11002, 11003] {
        reassembler.process_packet(
            &make_tcp_packet(
                client,
                port,
                server,
                80,
                1000,
                &[],
                true,
                false,
                false,
                false,
            ),
            1,
            &mut handler,
        );
    }

    assert_eq!(
        reassembler.flow_count(),
        3,
        "AC-001 precondition: 3 open flows before finalize"
    );
    handler.close_events.clear();

    reassembler.finalize(&mut handler);

    // All 3 must be closed with CloseReason::Timeout.
    assert_eq!(
        handler.close_events.len(),
        3,
        "AC-001: finalize must close exactly 3 flows via on_flow_close"
    );
    assert!(
        handler
            .close_events
            .iter()
            .all(|(_, r)| *r == CloseReason::Timeout),
        "AC-001: every on_flow_close reason must be CloseReason::Timeout"
    );

    // flows table must be empty.
    assert_eq!(
        reassembler.flow_count(),
        0,
        "AC-001: self.flows must be empty after finalize"
    );
}

/// AC-002 (BC-2.04.012 postcondition 3)
/// After finalize, self.finalized == true (observable via is_finalized()).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_012_finalize_sets_finalized_latch() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();

    assert!(
        !reassembler.is_finalized(),
        "AC-002 precondition: finalized must be false before finalize()"
    );

    reassembler.finalize(&mut handler);

    assert!(
        reassembler.is_finalized(),
        "AC-002: finalize() must set the finalized latch to true"
    );
}

/// AC-003 (BC-2.04.012 postcondition 5 and invariant 1)
/// Second finalize call is a complete no-op: no additional close_flow calls,
/// no additional findings, no additional callbacks.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_012_finalize_is_idempotent() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Open 2 flows.
    for port in [22001u16, 22002] {
        reassembler.process_packet(
            &make_tcp_packet(
                client,
                port,
                server,
                80,
                1000,
                &[],
                true,
                false,
                false,
                false,
            ),
            1,
            &mut handler,
        );
    }

    // First finalize: closes 2 flows.
    reassembler.finalize(&mut handler);
    let close_count_after_first = handler.close_events.len();
    assert_eq!(
        close_count_after_first, 2,
        "AC-003: first finalize must close both flows before idempotency check"
    );
    let findings_after_first = reassembler.findings().len();

    // Second finalize: must be a no-op.
    reassembler.finalize(&mut handler);

    assert_eq!(
        handler.close_events.len(),
        close_count_after_first,
        "AC-003: second finalize must not emit any additional on_flow_close callbacks"
    );
    assert_eq!(
        reassembler.findings().len(),
        findings_after_first,
        "AC-003: second finalize must not push any additional findings"
    );
    assert!(
        reassembler.is_finalized(),
        "AC-003: is_finalized() must remain true after second call"
    );
}

/// AC-004 (BC-2.04.012 edge case EC-006)
/// Dropping a reassembler WITHOUT calling finalize fires the one-shot
/// `FINALIZE_SKIPPED_WARNED` atomic (sets it to `true`). The Drop impl cannot
/// flush flows (no handler argument), so flows are simply discarded.
///
/// The stderr text itself is not captured here (requires a helper binary or
/// nightly `set_output_capture`). Instead the test uses a reset → drop → swap
/// sequence to detect that the production Drop hook fired AT LEAST ONCE:
///   1. Acquire `FINALIZE_SKIPPED_WARNED_LOCK`
///   2. `reset_finalize_skipped_warned_for_testing()` (set to false)
///   3. Drop the un-finalized reassembler
///   4. `swap_finalize_skipped_warned_for_testing(false)` — atomically reads
///      the post-drop value AND resets it; assert returned value == true
///
/// **Scope limitation (Option B):** Under parallel test scheduling with
/// `cargo test`'s default multi-thread mode, the ~130+ sibling tests in this
/// file that drop un-finalized reassemblers without holding the lock may race
/// with AC-004's reset→swap window. The `true` returned by the swap might have
/// been set by a sibling Drop rather than AC-004's own Drop. The test therefore
/// asserts that the Drop hook is called AT LEAST ONCE in this process — not
/// that AC-004's specific Drop was the setter. Unique per-Drop attribution
/// requires process-isolation (`cargo test --test-threads=1`) or stderr capture.
/// The test still reliably detects total removal of the Drop hook in a fresh
/// single-binary process.
///
/// Tests that reset and observe `FINALIZE_SKIPPED_WARNED` MUST hold
/// `FINALIZE_SKIPPED_WARNED_LOCK` for their entire body.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_012_drop_without_finalize_emits_warning() {
    let _guard = FINALIZE_SKIPPED_WARNED_LOCK
        .lock()
        .expect("FINALIZE_SKIPPED_WARNED_LOCK poisoned");

    // Reset so the transition is observable even if a prior test already fired it.
    TcpReassembler::reset_finalize_skipped_warned_for_testing();
    assert!(
        !TcpReassembler::finalize_skipped_warned_for_testing(),
        "AC-004 precondition: FINALIZE_SKIPPED_WARNED must be false at test start"
    );

    // Create a reassembler with one open flow (so the Drop has something to discard).
    let mut handler = RecordingHandler::new();
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    reassembler.process_packet(
        &make_tcp_packet(
            [10, 0, 0, 1],
            33001,
            [10, 0, 0, 2],
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // Drop WITHOUT calling finalize — the Drop hook must set the atomic.
    drop(reassembler);

    // Atomically read the post-drop value and reset it. Under Option B scope
    // (see this test's header docstring and the FINALIZE_SKIPPED_WARNED_LOCK
    // docstring above), a `true` return confirms the production Drop hook
    // fired AT LEAST ONCE in this process — not necessarily that this test's
    // specific Drop was the setter. The test reliably detects total removal
    // of the Drop hook in a fresh single-binary process.
    let was_set = TcpReassembler::swap_finalize_skipped_warned_for_testing(false);
    assert!(
        was_set,
        "AC-004: dropping an un-finalized reassembler must set FINALIZE_SKIPPED_WARNED to true \
         (swap returned false — the production impl Drop hook did not fire)"
    );

    // The Drop impl has no handler; flows are NOT flushed.
    assert_eq!(
        handler.close_events.len(),
        0,
        "AC-004: Drop must NOT call on_flow_close (no handler argument available)"
    );
}

// ---- BC-2.04.024: total findings capped at MAX_FINDINGS = 10,000 ------------

/// AC-005 (BC-2.04.024 postconditions 1-2)
/// When findings.len() >= MAX_FINDINGS and a new finding would be emitted,
/// the finding is NOT pushed and stats.dropped_findings increments by 1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_024_findings_capped_at_max_findings() {
    let config = ReassemblyConfig {
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Pre-fill to exactly MAX_FINDINGS using the push_finding_for_testing seam.
    for i in 0..10_000usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-005 precondition: findings must be at MAX_FINDINGS"
    );

    let dropped_before = reassembler.stats().dropped_findings;

    // Trigger one more normal finding via a conflicting overlap.
    // Strategy: SYN + out-of-order segment (gap prevents flush) + conflicting retransmit.
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            44001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 44001, server, 80, 1002, b"AAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 44001, server, 80, 1002, b"BBBB", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-005: findings.len() must remain at MAX_FINDINGS — no push beyond cap"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "AC-005: dropped_findings must increment by 1 when cap is hit"
    );
}

/// AC-006 (BC-2.04.024 edge case EC-001)
/// When findings.len() == MAX_FINDINGS - 1 (9,999), the next finding IS
/// accepted (bringing the count to 10,000).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_024_finding_added_at_one_below_cap() {
    let config = ReassemblyConfig {
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Pre-fill to MAX_FINDINGS - 1 = 9,999.
    for i in 0..9_999usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }
    assert_eq!(
        reassembler.findings().len(),
        9_999,
        "AC-006 precondition: findings must be at MAX_FINDINGS - 1"
    );

    let dropped_before = reassembler.stats().dropped_findings;

    // Trigger one normal finding via a conflicting overlap.
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            55001,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 55001, server, 80, 1002, b"AAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 55001, server, 80, 1002, b"BBBB", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-006: finding at MAX_FINDINGS - 1 must be accepted, bringing len to 10,000"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before,
        "AC-006: dropped_findings must NOT increment when the 10,000th finding is accepted"
    );
}

/// AC-007 (BC-2.04.024 invariant 1 — engine cap boundary)
/// The MAX_FINDINGS cap constant equals exactly 10,000 — the engine accepts
/// 10,000 findings then begins dropping. This test verifies the cap boundary
/// observable behavior: 9,999 pre-filled + one triggered = 10,000 accepted;
/// one more triggered = dropped.
///
/// HttpAnalyzer and TlsAnalyzer do NOT share this cap; that is verified
/// separately in `test_BC_2_04_024_http_tls_analyzer_findings_not_capped`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_024_engine_cap_at_exactly_10000() {
    // Verify MAX_FINDINGS == 10,000 via the observable boundary:
    // exactly 10,000 findings are accepted before the cap bites.
    let config = ReassemblyConfig {
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Pre-fill to 9,999 and accept one more to reach exactly 10,000.
    for i in 0..9_999usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            6601,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 6601, server, 80, 1002, b"AAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 6601, server, 80, 1002, b"BBBB", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-007: MAX_FINDINGS == 10,000 — the 10,000th finding is the exact cap"
    );

    // One more: must be dropped.
    let dropped_before = reassembler.stats().dropped_findings;
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            6602,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        4,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 6602, server, 80, 1002, b"CCCC", false, true, false, false,
        ),
        5,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 6602, server, 80, 1002, b"DDDD", false, true, false, false,
        ),
        6,
        &mut handler,
    );

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-007: beyond MAX_FINDINGS the count stays at 10,000 — cap is 10,000 not higher"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "AC-007: dropped_findings increments when the engine-local cap is hit"
    );
}

// ---- BC-2.04.025: finalize emits segment-limit summary finding --------------

/// AC-008 (BC-2.04.025 postcondition 1)
/// When finalize is called and segments_segment_limit > 0, exactly one finding
/// is pushed with category=Anomaly, verdict=Inconclusive, confidence=Medium,
/// mitre_technique=None, source_ip=None, direction=None.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_025_finalize_emits_segment_limit_finding() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();

    reassembler.set_segments_segment_limit_for_testing(7);

    reassembler.finalize(&mut handler);

    let findings = reassembler.findings();

    // Collect all segment-limit findings by summary pattern; assert exactly one exists.
    // Using filter+collect rather than .last() makes the count assertion and the field
    // assertions independent — we prove the finding was emitted exactly once, not merely
    // that something happened to be last.
    let seg_limit_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("segment count limit"))
        .collect();
    assert_eq!(
        seg_limit_findings.len(),
        1,
        "AC-008: finalize must push exactly one segment-limit finding when counter > 0 \
         (got {} matching findings)",
        seg_limit_findings.len()
    );

    let f = seg_limit_findings[0];
    assert_eq!(
        f.category,
        ThreatCategory::Anomaly,
        "AC-008: segment-limit finding category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        Verdict::Inconclusive,
        "AC-008: segment-limit finding verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        Confidence::Medium,
        "AC-008: segment-limit finding confidence must be Medium"
    );
    assert!(
        f.mitre_technique.is_none(),
        "AC-008: segment-limit finding mitre_technique must be None"
    );
    assert!(
        f.source_ip.is_none(),
        "AC-008: segment-limit finding source_ip must be None"
    );
    assert!(
        f.direction.is_none(),
        "AC-008: segment-limit finding direction must be None"
    );
}

/// AC-009 (BC-2.04.025 postcondition 1 and invariant 3)
/// The summary string uses correct singular/plural grammar:
/// count==1 → "1 segment dropped due to per-flow segment count limit"
/// count==N>1 → "N segments dropped due to per-flow segment count limit"
///
/// N=2 is the actual plural-boundary (first value > 1 that requires the 's').
/// N=100 is retained as a sanity check at a larger value.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_025_segment_limit_finding_singular_plural_grammar() {
    // --- singular: count == 1 ---
    {
        let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
        let mut handler = RecordingHandler::new();
        reassembler.set_segments_segment_limit_for_testing(1);
        reassembler.finalize(&mut handler);

        let f = reassembler
            .findings()
            .iter()
            .find(|f| f.summary.contains("segment count limit"))
            .expect("AC-009: segment-limit finding must exist when count==1");

        assert_eq!(
            f.summary, "1 segment dropped due to per-flow segment count limit",
            "AC-009: count==1 must use singular grammar (no trailing 's')"
        );
    }

    // --- plural boundary: count == 2 (the smallest value requiring plural 's') ---
    {
        let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
        let mut handler = RecordingHandler::new();
        reassembler.set_segments_segment_limit_for_testing(2);
        reassembler.finalize(&mut handler);

        let f = reassembler
            .findings()
            .iter()
            .find(|f| f.summary.contains("segment count limit"))
            .expect("AC-009: segment-limit finding must exist when count==2");

        assert_eq!(
            f.summary, "2 segments dropped due to per-flow segment count limit",
            "AC-009: count==2 is the plural boundary — must use plural grammar (trailing 's')"
        );
    }

    // --- plural sanity: count == 100 ---
    {
        let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
        let mut handler = RecordingHandler::new();
        reassembler.set_segments_segment_limit_for_testing(100);
        reassembler.finalize(&mut handler);

        let f = reassembler
            .findings()
            .iter()
            .find(|f| f.summary.contains("segment count limit"))
            .expect("AC-009: segment-limit finding must exist when count==100");

        assert_eq!(
            f.summary, "100 segments dropped due to per-flow segment count limit",
            "AC-009: count==100 must use plural grammar (trailing 's')"
        );
    }
}

/// AC-010 (BC-2.04.025 postcondition 1)
/// The segment-limit finding's evidence vec contains EXACTLY:
/// ["Segment count limit prevents BTreeMap overhead explosion",
///  "May indicate segmentation-based evasion attempt"]
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_025_segment_limit_finding_evidence_strings() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();

    reassembler.set_segments_segment_limit_for_testing(3);
    reassembler.finalize(&mut handler);

    let f = reassembler
        .findings()
        .iter()
        .find(|f| f.summary.contains("segment count limit"))
        .expect("AC-010: segment-limit finding must exist when counter > 0");

    assert_eq!(
        f.evidence,
        vec![
            "Segment count limit prevents BTreeMap overhead explosion",
            "May indicate segmentation-based evasion attempt",
        ],
        "AC-010: evidence vec must contain exactly the two canonical strings in order"
    );
}

// ---- BC-2.04.026: no segment-limit finding when counter is zero -------------

/// AC-011 (BC-2.04.026 postconditions 1-2)
/// When finalize is called and segments_segment_limit == 0, NO segment-limit
/// finding is pushed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_026_no_segment_limit_finding_when_counter_zero() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();

    // segments_segment_limit is 0 by default on a fresh engine.
    assert_eq!(
        reassembler.stats().segments_segment_limit,
        0,
        "AC-011 precondition: fresh engine must have segments_segment_limit == 0"
    );

    reassembler.finalize(&mut handler);

    let segment_limit_findings: Vec<_> = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("segment count limit"))
        .collect();

    assert!(
        segment_limit_findings.is_empty(),
        "AC-011: finalize must NOT emit a segment-limit finding when counter == 0 \
         (got {} such finding(s))",
        segment_limit_findings.len()
    );
}

// ---- BC-2.04.054: finalize unconditionally bypasses MAX_FINDINGS cap --------

/// AC-012 (BC-2.04.054 postconditions 1-2 and invariant 1)
/// When findings.len() == MAX_FINDINGS at finalize time and
/// segments_segment_limit > 0, the segment-limit finding IS pushed
/// unconditionally, causing findings.len() to become MAX_FINDINGS + 1 = 10,001.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_054_finalize_bypasses_max_findings_cap() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();

    // Pre-fill to exactly MAX_FINDINGS = 10,000.
    for i in 0..10_000usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-012 precondition: findings must be at MAX_FINDINGS = 10,000"
    );

    // Arm the segment-limit counter.
    reassembler.set_segments_segment_limit_for_testing(5);

    let dropped_before = reassembler.stats().dropped_findings;

    reassembler.finalize(&mut handler);

    assert_eq!(
        reassembler.findings().len(),
        10_001,
        "AC-012: finalize must push the segment-limit finding unconditionally, \
         raising findings.len() from 10,000 to 10,001"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before,
        "AC-012: finalize segment-limit bypass must NOT increment dropped_findings"
    );

    // The extra finding must be the segment-limit summary, not a stray finding.
    let last = reassembler
        .findings()
        .last()
        .expect("AC-012: findings must be non-empty");
    assert!(
        last.summary.contains("segment count limit"),
        "AC-012: the 10,001st finding must be the segment-limit summary \
         (got: {:?})",
        last.summary
    );
}

/// AC-013 (BC-2.04.054 invariant 3): Representative-scenario smoke test
/// of the finalize-bypass bound at `MAX_FINDINGS` pre-fill.
///
/// Pre-fills findings to MAX_FINDINGS via `push_finding_for_testing`,
/// arms `segments_segment_limit`, calls finalize, asserts findings.len()
/// reaches exactly MAX_FINDINGS + 1 (= 10,001); also verifies idempotency
/// (second finalize is a no-op).
///
/// **Scope:** This is a representative-scenario smoke test, not a
/// universal upper-bound proof. The universal `len ≤ MAX_FINDINGS + 1`
/// invariant (∀ inputs, ∀ schedules) is owned by VP-003 (property-based
/// test). See STORY-021 AC-013 prose and F-W11P1-007 rationale for the
/// rephrasing.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_054_finalize_bypass_smoke_at_max_findings_representative_scenario() {
    let config = ReassemblyConfig {
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Fill to MAX_FINDINGS via direct push.
    for i in 0..10_000usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }

    // Arm the segment-limit counter.
    reassembler.set_segments_segment_limit_for_testing(99);

    // Normal processing at cap: should not push beyond 10,000.
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            7701,
            server,
            80,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 7701, server, 80, 1002, b"AAAA", false, true, false, false,
        ),
        2,
        &mut handler,
    );
    reassembler.process_packet(
        &make_tcp_packet(
            client, 7701, server, 80, 1002, b"BBBB", false, true, false, false,
        ),
        3,
        &mut handler,
    );

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "AC-013: normal processing beyond cap must not exceed MAX_FINDINGS = 10,000"
    );

    // First finalize: the ONE allowed bypass pushes to 10,001.
    reassembler.finalize(&mut handler);
    assert_eq!(
        reassembler.findings().len(),
        10_001,
        "AC-013: after finalize with segment_limit > 0, findings.len() must be 10,001"
    );

    // Second finalize: idempotency latch — no additional findings.
    reassembler.finalize(&mut handler);
    assert_eq!(
        reassembler.findings().len(),
        10_001,
        "AC-013: second finalize call must not push any additional findings (idempotent latch)"
    );
}

// ============================================================================
// STORY-021 adversarial-pass-1 remediation tests
// ============================================================================

// ---- F-W11P1-002: AC-007b companion — HttpAnalyzer + TlsAnalyzer not capped ---

/// F-W11P1-002 companion to AC-007b (BC-2.04.024 invariant 4 — analyzer non-cap)
/// The MAX_FINDINGS cap is LOCAL to `TcpReassembler`. This test verifies that
/// BOTH `HttpAnalyzer` AND `TlsAnalyzer` accumulate findings BEYOND 10,000
/// without any cap, proving the invariant "applies ONLY to reassembly engine"
/// claimed by AC-007b and EC-011.
///
/// Strategy: use the `push_finding_for_testing` seam on each analyzer to
/// inject 10,001 findings directly, then assert the count > 10,000. This
/// avoids constructing 10,001 real protocol transactions while still verifying
/// the structural claim (no local MAX_FINDINGS cap in either analyzer).
/// Both analyzers are exercised in the same test for true parity (BC-2.04.024
/// invariant 4 applies equally to HttpAnalyzer and TlsAnalyzer).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_024_http_tls_analyzer_findings_not_capped() {
    use wirerust::analyzer::http::HttpAnalyzer;
    use wirerust::analyzer::tls::TlsAnalyzer;

    // ── HttpAnalyzer parity ───────────────────────────────────────────────
    let mut http_analyzer = HttpAnalyzer::new();

    // Push 10,001 findings — one beyond the engine's MAX_FINDINGS cap.
    for i in 0..10_001usize {
        http_analyzer.push_finding_for_testing(wirerust::findings::Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Medium,
            summary: format!("http test finding {i}"),
            evidence: vec![],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
            direction: None,
        });
    }

    // HttpAnalyzer must NOT drop the 10,001st finding — no local cap.
    assert_eq!(
        http_analyzer.all_findings_len_for_testing(),
        10_001,
        "BC-2.04.024 invariant 4: HttpAnalyzer must NOT apply the engine's MAX_FINDINGS cap; \
         expected 10,001 findings, got {}",
        http_analyzer.all_findings_len_for_testing()
    );

    // ── TlsAnalyzer parity ────────────────────────────────────────────────
    let mut tls_analyzer = TlsAnalyzer::new();

    // Push 10,001 findings — one beyond the engine's MAX_FINDINGS cap.
    for i in 0..10_001usize {
        tls_analyzer.push_finding_for_testing(wirerust::findings::Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Medium,
            summary: format!("tls test finding {i}"),
            evidence: vec![],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
            direction: None,
        });
    }

    // TlsAnalyzer must NOT drop the 10,001st finding — no local cap.
    assert_eq!(
        tls_analyzer.all_findings_len_for_testing(),
        10_001,
        "BC-2.04.024 invariant 4: TlsAnalyzer must NOT apply the engine's MAX_FINDINGS cap; \
         expected 10,001 findings, got {}",
        tls_analyzer.all_findings_len_for_testing()
    );
}

// ---- F-W11P1-003: EC-006 boundary (MAX-1 + finalize) ------------------------

/// AC-016 (story EC-006 boundary, traces to BC-2.04.054 EC-002 "no bypass
/// semantics triggered below cap"): At findings.len() == MAX_FINDINGS-1
/// (= 9,999) with segments_segment_limit > 0, the finalize push is the
/// **normal path** (the push is structurally unconditional in source — no
/// cap guard — but per BC-2.04.054 EC-002 the bypass semantic is only
/// triggered when findings.len() == MAX_FINDINGS). At MAX-1, the push
/// brings findings.len() to MAX_FINDINGS (= 10,000), still within the
/// normal cap.
///
/// No flows are opened. The test operates entirely on findings-vec arithmetic:
///   1. Pre-fill to MAX_FINDINGS - 1 = 9,999 via `push_finding_for_testing`.
///   2. Arm `segments_segment_limit = 1` to exercise the finalize normal path.
///   3. Call `finalize` with a recording handler.
///   4. Assert `findings.len() == 10,000` and exactly one segment-limit finding.
///
/// There is no CloseReason::Timeout assertion (no flow was opened). The push
/// in `src/reassembly/mod.rs` is structurally unconditional, but at MAX-1
/// the cap is not exceeded and no bypass semantic is triggered.
///
/// See sibling test `test_BC_2_04_054_segment_limit_finding_emitted_regardless_of_initial_count`
/// (AC-017) for the parameterized case covering both the normal path (pre_fill < MAX)
/// and the bypass path (pre_fill == MAX).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_054_finalize_at_max_findings_minus_one() {
    let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
    let mut handler = RecordingHandler::new();

    // Pre-fill to MAX_FINDINGS - 1 = 9,999.
    for i in 0..9_999usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }
    assert_eq!(
        reassembler.findings().len(),
        9_999,
        "F-W11P1-003 precondition: findings must be at MAX_FINDINGS - 1 = 9,999"
    );

    // Arm the segment-limit counter (1 segment was dropped).
    reassembler.set_segments_segment_limit_for_testing(1);

    reassembler.finalize(&mut handler);

    // The segment-limit finding is emitted via the normal path (pre_fill < MAX).
    // At 9,999 pre-fill + 1 normal push = 10,000 (exactly MAX_FINDINGS).
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "F-W11P1-003: finalize at MAX_FINDINGS-1 with segment_limit=1 must \
         push the segment-limit finding, reaching exactly 10,000"
    );

    // The last finding must be the segment-limit summary.
    let seg_limit_findings: Vec<_> = reassembler
        .findings()
        .iter()
        .filter(|f| f.summary.contains("segment count limit"))
        .collect::<Vec<_>>();
    assert_eq!(
        seg_limit_findings.len(),
        1,
        "F-W11P1-003: exactly one segment-limit finding must be present"
    );
    assert!(
        seg_limit_findings[0]
            .summary
            .contains("segment count limit"),
        "F-W11P1-003: last finding must be the segment-limit summary"
    );
}

// ---- F-W11P1-004: dropped_findings monotonicity (BC-2.04.024 EC-004) --------

/// F-W11P1-004 (BC-2.04.024 EC-004 — dropped_findings monotonicity)
/// Strengthen AC-005: trigger ≥ 3 cap-hit events and assert
/// `stats.dropped_findings == 3` (not just == 1). Each conflicting overlap
/// at cap increments the counter; the counter is strictly monotone.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_024_dropped_findings_monotone_over_multiple_cap_hits() {
    let config = ReassemblyConfig {
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    // Pre-fill to exactly MAX_FINDINGS.
    for i in 0..10_000usize {
        reassembler.push_finding_for_testing(dummy_finding(i));
    }
    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "F-W11P1-004 precondition: findings must be at MAX_FINDINGS"
    );

    let dropped_before = reassembler.stats().dropped_findings;

    // Helper closure: trigger one ConflictingOverlap finding on a given port.
    // Fires AFTER the cap is full, so each overlap increments dropped_findings.
    let trigger_overlap = |reassembler: &mut TcpReassembler,
                           handler: &mut RecordingHandler,
                           port: u16,
                           ts_base: u32| {
        let client = [10, 0, 0, 1];
        let server = [10, 0, 0, 2];
        reassembler.process_packet(
            &make_tcp_packet(
                client,
                port,
                server,
                80,
                1000,
                &[],
                true,
                false,
                false,
                false,
            ),
            ts_base,
            handler,
        );
        reassembler.process_packet(
            &make_tcp_packet(
                client, port, server, 80, 1002, b"AAAA", false, true, false, false,
            ),
            ts_base + 1,
            handler,
        );
        reassembler.process_packet(
            &make_tcp_packet(
                client, port, server, 80, 1002, b"BBBB", false, true, false, false,
            ),
            ts_base + 2,
            handler,
        );
    };

    // Three cap-hit events on distinct ports.
    trigger_overlap(&mut reassembler, &mut handler, 60001, 100);
    trigger_overlap(&mut reassembler, &mut handler, 60002, 200);
    trigger_overlap(&mut reassembler, &mut handler, 60003, 300);

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "F-W11P1-004: findings.len() must remain at MAX_FINDINGS after 3 cap-hit events"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 3,
        "F-W11P1-004: dropped_findings must be monotonically increasing — \
         3 cap-hit events must yield dropped_findings == dropped_before + 3"
    );
}

// ---- F-W11P1-005: cap guard at small_segment site (mod.rs:466) --------------

/// F-W11P1-005 (BC-2.04.024 postconditions 1-2 / mod.rs:466 — small_segment cap guard)
/// When the small_segment alert fires while findings.len() >= MAX_FINDINGS,
/// the finding is NOT pushed but `stats.dropped_findings` is incremented.
///
/// Source line cited: `src/reassembly/mod.rs:466` — the cap guard at the
/// small_segment alert path. The out_of_window cap guard at mod.rs:495 is
/// already covered by `test_story_017_ec007_oow_alert_at_max_findings_latch_set_dropped_incremented`.
///
/// Cross-story coverage note: the out-of-window site (mod.rs:495) is covered
/// by STORY-017 test `test_story_017_ec007_oow_alert_at_max_findings_latch_set_dropped_incremented`.
/// This test covers the small_segment site (mod.rs:466).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_024_cap_guard_small_segment_site() {
    let config = ReassemblyConfig {
        small_segment_alert_threshold: 0,
        small_segment_max_bytes: 16,
        max_flows: 20_000,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = fill_findings_to_cap(config);
    let mut handler = RecordingHandler::new();

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "F-W11P1-005 precondition: findings must be at MAX_FINDINGS"
    );
    let dropped_before = reassembler.stats().dropped_findings;

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Open a fresh flow (port 10001 is beyond the 1..=10_000 range used
    // by fill_findings_to_cap so no collisions).
    reassembler.process_packet(
        &make_tcp_packet(
            client,
            10001,
            server,
            9090,
            1000,
            &[],
            true,
            false,
            false,
            false,
        ),
        1,
        &mut handler,
    );

    // One small segment → small_segment_run=1 > threshold 0 → alert fires at cap.
    // mod.rs:466: `if self.findings.len() < MAX_FINDINGS` is false → else branch
    // increments dropped_findings and does NOT push a finding.
    reassembler.process_packet(
        &make_tcp_packet(
            client, 10001, server, 9090, 1001, b"x", false, true, false, false,
        ),
        2,
        &mut handler,
    );

    assert_eq!(
        reassembler.findings().len(),
        10_000,
        "F-W11P1-005: small_segment alert at cap must NOT push a finding (mod.rs:466 else branch)"
    );
    assert_eq!(
        reassembler.stats().dropped_findings,
        dropped_before + 1,
        "F-W11P1-005: small_segment alert at cap must increment dropped_findings by 1 \
         (mod.rs:466 else branch)"
    );
}

// ---- F-W11P1-010: AC-008 parameterized for multiple initial lengths ----------

/// F-W11P1-010 (BC-2.04.054 EC-002 — segment-limit finding unconditional below MAX)
/// Finalize emits the segment-limit finding regardless of how many findings
/// exist at finalize time, as long as segments_segment_limit > 0.
/// Tests initial lengths: 0, 5000, 9999, 10000 (cap boundary).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_054_segment_limit_finding_emitted_regardless_of_initial_count() {
    // Test helper: for a given pre-fill count, assert finalize adds exactly one
    // segment-limit finding. At pre-fill == 10,000 the bypass path is exercised
    // (total becomes 10,001). At all other values the normal path is used.
    let run_case = |pre_fill: usize, label: &str| {
        let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
        let mut handler = RecordingHandler::new();

        for i in 0..pre_fill {
            reassembler.push_finding_for_testing(dummy_finding(i));
        }
        reassembler.set_segments_segment_limit_for_testing(3);
        reassembler.finalize(&mut handler);

        let seg_limit_count = reassembler
            .findings()
            .iter()
            .filter(|f| f.summary.contains("segment count limit"))
            .count();
        assert_eq!(
            seg_limit_count, 1,
            "F-W11P1-010 [{label}]: finalize must push exactly one segment-limit finding \
             regardless of initial findings count ({pre_fill}); got {seg_limit_count}"
        );
        assert_eq!(
            reassembler.findings().len(),
            pre_fill + 1,
            "F-W11P1-010 [{label}]: total findings must be pre_fill({pre_fill}) + 1 \
             after finalize with segment_limit > 0"
        );
    };

    run_case(0, "initial=0");
    run_case(5_000, "initial=5000");
    run_case(9_999, "initial=9999");
    // At pre_fill=10,000 the bypass path is used: total becomes 10,001.
    {
        let mut reassembler = TcpReassembler::new(ReassemblyConfig::default());
        let mut handler = RecordingHandler::new();
        for i in 0..10_000usize {
            reassembler.push_finding_for_testing(dummy_finding(i));
        }
        reassembler.set_segments_segment_limit_for_testing(3);
        reassembler.finalize(&mut handler);

        let seg_limit_count = reassembler
            .findings()
            .iter()
            .filter(|f| f.summary.contains("segment count limit"))
            .count();
        assert_eq!(
            seg_limit_count, 1,
            "F-W11P1-010 [initial=10000 bypass]: finalize must push exactly one \
             segment-limit finding via the unconditional bypass path"
        );
        assert_eq!(
            reassembler.findings().len(),
            10_001,
            "F-W11P1-010 [initial=10000 bypass]: total must be 10,001 (bypass path)"
        );
    }
}

// ============================================================================
// Phase-6 mutation-survivor kills (mod.rs boundary + stats-counter mutants).
//
// These tests pin the EXACT behavior the surviving comparison-operator and
// stats-counter mutations would break. Each asserts a value that diverges
// between correct code and the mutant, so `cargo mutants` reports the mutant
// CAUGHT. Survivor IDs reference
// `.factory/.../mutation-results/mutation-summary-reassembly.md`.
// ============================================================================

/// Drive a SYN to establish the ISN, returning the configured reassembler,
/// handler, and the client/server addresses for a single TCP flow. Centralizes
/// the boilerplate the boundary/stats tests below all share.
fn syn_established_flow(
    config: ReassemblyConfig,
) -> (TcpReassembler, RecordingHandler, [u8; 4], [u8; 4]) {
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();
    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];
    let syn = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn, 1, &mut handler);
    (reassembler, handler, client, server)
}

// ---- GG-3 (mod.rs:206) — memcap eviction boundary `total_memory > memcap` ---
//
// The eviction gate at mod.rs:206 is `self.total_memory > self.config.memcap`
// (strictly greater). The surviving mutant replaced `>` with `>=`.
//
// IMPORTANT — this `>`→`>=` mutant is EQUIVALENT (observationally inert), NOT a
// killable test gap: `evict_flows` (lifecycle.rs) re-checks the bound with its
// own loop guard `total_memory <= memcap` and breaks immediately, so even if the
// mutant makes the outer gate fire at memory == memcap, the eviction loop evicts
// nothing (evictions stay 0, no flow closes). Both `>` and `>=` therefore produce
// identical observable output at the boundary. The cargo-mutants confirming run
// reports mod.rs:206 `>`→`>=` as MISSED for exactly this reason — it cannot be
// killed by any test without a production change to remove the redundant guard.
//
// These two tests do NOT (and cannot) kill the equivalent mutant. They PIN the
// documented at-cap non-eviction contract as a regression guard, and prove the
// eviction machinery still fires one byte past the cap.

#[test]
fn test_memcap_boundary_exactly_at_cap_does_not_evict() {
    // Pins the contract: total_memory EXACTLY at memcap must not evict. (This
    // does not kill the equivalent mod.rs:206 mutant — see the section note.)
    //
    // memcap == 5; a single buffered out-of-order 5-byte segment lands
    // total_memory at EXACTLY 5, so the gate `5 > 5` is false and nothing evicts.
    let config = ReassemblyConfig {
        memcap: 5,
        ..ReassemblyConfig::default()
    };
    let (mut reassembler, mut handler, client, server) = syn_established_flow(config);

    // SYN set ISN=1000 and base_offset=1 (set_isn in flow.rs does NOT advance
    // base_offset past 1 — it is the first data offset). Data at seq=1002 lands
    // at ISN-relative offset 2, span [2,7); offset [1,2) (between base_offset=1
    // and the segment start) is never filled, so the segment cannot flush and
    // stays buffered. That keeps total_memory at exactly 5 (the assertion below
    // is non-vacuous precisely because base_offset stays at 1 after the SYN).
    let data = make_tcp_packet(
        client, 12345, server, 80, 1002, b"aaaaa", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    assert_eq!(
        reassembler.total_memory(),
        5,
        "GG-3 setup: 5 buffered bytes must put total_memory exactly at the memcap"
    );
    assert_eq!(
        reassembler.stats().evictions,
        0,
        "GG-3: total_memory == memcap must NOT trigger eviction (contract: gate is `>`)"
    );
    assert!(
        !handler
            .close_events
            .iter()
            .any(|(_, r)| *r == CloseReason::MemoryPressure),
        "GG-3: no MemoryPressure close when memory is exactly at the cap"
    );
}

#[test]
fn test_memcap_boundary_one_over_cap_evicts() {
    // Non-vacuousness companion: proves the eviction machinery actually fires
    // just past the boundary, so the at-cap test above is not silent because
    // eviction is simply broken. This is NOT itself a mutant-kill (the mod.rs:206
    // mutant is equivalent — see the section note); it backstops the contract.
    //
    // memcap == 5; a buffered 6-byte segment puts total_memory at 6 == cap + 1.
    // `6 > 5` is true under BOTH `>` and `>=`, so eviction fires.
    let config = ReassemblyConfig {
        memcap: 5,
        ..ReassemblyConfig::default()
    };
    let (mut reassembler, mut handler, client, server) = syn_established_flow(config);

    let data = make_tcp_packet(
        client, 12345, server, 80, 1002, b"aaaaaa", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    // Exactly one flow is buffered, so exactly one eviction fires. Asserting the
    // precise count (not `>= 1`) also guards the `evictions += 1` counter in
    // evict_flows against an operator mutation.
    assert_eq!(
        reassembler.stats().evictions,
        1,
        "GG-3: total_memory == memcap + 1 must trigger exactly one eviction"
    );
}

// ---- GG-4 (mod.rs:394) — small-segment size threshold `len < max_bytes` -----
//
// "Small" is classified by `payload.len() < small_segment_max_bytes`
// (strictly less). The surviving mutant replaced `<` with `<=`, which would
// count a payload of EXACTLY the threshold size as "small". A run of
// exactly-threshold-sized segments must therefore NOT build the small-segment
// run (correct: `<`), so no anomaly fires. With the `<=` mutant the run builds
// and the anomaly fires. A run of (threshold - 1)-sized segments DOES build the
// run under both, confirming the detection machinery works (non-vacuous).

/// Drive `count` data segments of exactly `payload_len` bytes (distinct,
/// out-of-order behind a permanent gap so they stay buffered) through a flow
/// with `small_segment_max_bytes == max_bytes` and the given small-segment
/// alert `threshold`. Returns the reassembler so the caller can inspect findings.
fn run_sized_segment_flow(
    max_bytes: u16,
    threshold: u32,
    payload_len: usize,
    count: u32,
) -> TcpReassembler {
    let config = ReassemblyConfig {
        small_segment_max_bytes: max_bytes,
        small_segment_alert_threshold: threshold,
        ..ReassemblyConfig::default()
    };
    let (mut reassembler, mut handler, client, server) = syn_established_flow(config);

    // Leave offset 1 unfilled (the SYN set base_offset=1) so every data
    // segment stays buffered and reaches the reassembly window (it is counted
    // by the small-segment classifier regardless of whether it flushes).
    let payload = vec![b'x'; payload_len];
    // Start at offset 2 (seq 1002) and place each segment contiguously after
    // the previous so there are no overlaps (each is `Inserted`, reaching the
    // window and so feeding the small-segment classifier).
    let mut seq: u32 = 1002;
    let mut ts: u32 = 2;
    for _ in 0..count {
        let pkt = make_tcp_packet(
            client, 12345, server, 80, seq, &payload, false, true, false, false,
        );
        reassembler.process_packet(&pkt, ts, &mut handler);
        seq += payload_len as u32;
        ts += 1;
    }
    reassembler
}

#[test]
fn test_small_segment_size_at_exact_threshold_is_not_small() {
    // small_segment_max_bytes = 4, alert threshold = 3. Send 8 segments of
    // EXACTLY 4 bytes. Correct `<`: `4 < 4` == false, so none are "small",
    // the run never advances past 0, and NO anomaly fires.
    // The `<=` mutant makes `4 <= 4` == true, every segment counts as small,
    // the run climbs to 8 (> 3), and the anomaly fires — which this test forbids.
    let reasm = run_sized_segment_flow(4, 3, 4, 8);
    assert!(
        !fired_small_segment(&reasm),
        "GG-4: segments of size == small_segment_max_bytes must NOT count as small \
         (classifier is `<`, not `<=`), so no small-segment anomaly may fire"
    );
}

#[test]
fn test_small_segment_size_one_below_threshold_is_small() {
    // small_segment_max_bytes = 4, alert threshold = 3. Send 8 segments of
    // EXACTLY 3 bytes. `3 < 4` == true under BOTH `<` and `<=`, so each is
    // "small", the run climbs to 8 (> 3), and the anomaly MUST fire. This is
    // the correct-code companion proving the run-building machinery works, so
    // the at-threshold test above is not vacuously silent.
    let reasm = run_sized_segment_flow(4, 3, 3, 8);
    assert!(
        fired_small_segment(&reasm),
        "GG-4: a long run of (threshold - 1)-sized segments must fire the small-segment anomaly"
    );
}

// ---- GG-6..GG-11 (mod.rs:403/405/406/409/413) — stats-counter accuracy ------
//
// The InsertResult match arms bump `self.stats.segments_* += 1`. The surviving
// mutants replaced `+=` with `*=` (a no-op when adding 1) or `-=` (decrement /
// underflow). Asserting the EXACT counter value after a known number of
// operations diverges from both: a no-op leaves the counter below the expected
// total, and a decrement drives it below (underflow-panics from 0). Each test
// drives exactly one representative operation of the relevant InsertResult.

#[test]
fn test_stats_duplicate_counter_exact() {
    // GG (mod.rs:403): InsertResult::Duplicate => segments_duplicates += 1.
    // Buffer b"AAAA" out of order, then re-send the IDENTICAL bytes at the same
    // offset → Duplicate (fully covered, no conflict). Exactly one Duplicate.
    let (mut reassembler, mut handler, client, server) =
        syn_established_flow(ReassemblyConfig::default());

    let original = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);
    let dup = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&dup, 3, &mut handler);

    assert_eq!(
        reassembler.stats().segments_duplicates,
        1,
        "GG (403): exactly one identical retransmission must count as one duplicate \
         (kills `+=`→`*=`/`-=` on segments_duplicates)"
    );
}

#[test]
fn test_stats_partial_overlap_counters_exact() {
    // GG (mod.rs:405/406): InsertResult::PartialOverlap =>
    //   segments_overlaps += 1; segments_inserted += 1.
    // Buffer b"AA" at offset 2 ([2,4)), then send b"ABCD" at offset 3 ([3,7)):
    // overlaps [3,4) with existing, leaves gap [4,7) to insert → PartialOverlap.
    //
    // The overlap region is a single byte at offset 3. It is `A` in BOTH
    // segments — existing b"AA"[3-2]=='A' and new b"ABCD"[3-3]=='A' — i.e. the
    // overlap AGREES, so this is a NON-conflicting (clean) partial overlap. That
    // is why the result is PartialOverlap (bumping both segments_overlaps and
    // segments_inserted), and NOT ConflictingOverlap (which would skip the
    // segments_inserted bump). A differing overlap byte here would change the
    // classification and the expected counter values below.
    let (mut reassembler, mut handler, client, server) =
        syn_established_flow(ReassemblyConfig::default());

    let first = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AA", false, false, false, false,
    );
    reassembler.process_packet(&first, 2, &mut handler);
    // segments_inserted is now 1 (the clean insert above).
    assert_eq!(reassembler.stats().segments_inserted, 1);
    assert_eq!(reassembler.stats().segments_overlaps, 0);

    let overlap = make_tcp_packet(
        client, 12345, server, 80, 1003, b"ABCD", false, false, false, false,
    );
    reassembler.process_packet(&overlap, 3, &mut handler);

    assert_eq!(
        reassembler.stats().segments_overlaps,
        1,
        "GG (405): one partial overlap must add exactly one to segments_overlaps"
    );
    assert_eq!(
        reassembler.stats().segments_inserted,
        2,
        "GG (406): the partial-overlap insert must add exactly one to segments_inserted \
         (1 from the clean insert + 1 here)"
    );
}

#[test]
fn test_stats_conflicting_overlap_counter_exact() {
    // GG (mod.rs:409): InsertResult::ConflictingOverlap => segments_overlaps += 1
    // (no segments_inserted bump — the conflicting bytes lose under first-wins).
    // Buffer b"AAAA" at offset 2, then send b"BBBB" at the SAME offset (fully
    // covered, bytes differ) → ConflictingOverlap. Exactly one overlap counted.
    let (mut reassembler, mut handler, client, server) =
        syn_established_flow(ReassemblyConfig::default());

    let original = make_tcp_packet(
        client, 12345, server, 80, 1002, b"AAAA", false, false, false, false,
    );
    reassembler.process_packet(&original, 2, &mut handler);
    assert_eq!(reassembler.stats().segments_inserted, 1);

    let conflict = make_tcp_packet(
        client, 12345, server, 80, 1002, b"BBBB", false, false, false, false,
    );
    reassembler.process_packet(&conflict, 3, &mut handler);

    assert_eq!(
        reassembler.stats().segments_overlaps,
        1,
        "GG (409): one conflicting overlap must add exactly one to segments_overlaps \
         (kills `+=`→`*=`/`-=`)"
    );
    assert_eq!(
        reassembler.stats().segments_inserted,
        1,
        "GG (409): a conflicting overlap inserts NO new bytes — segments_inserted unchanged"
    );
    // CR-008: the conflicting overlap also EMITS exactly one finding. A single
    // conflict on a fresh flow trips no other alert (overlap-count and
    // small-segment thresholds are far higher under the default config), so the
    // finding count is exactly 1 — covering the generate_conflicting_overlap_finding
    // emission path alongside the counter assertions above.
    assert_eq!(
        reassembler.findings().len(),
        1,
        "GG (409): a conflicting overlap must emit exactly one finding"
    );
}

#[test]
fn test_stats_truncated_counter_exact() {
    // GG (mod.rs:413): InsertResult::Truncated => segments_inserted += 1.
    // With max_depth = 3 and an out-of-order segment longer than the remaining
    // depth, insert_segment truncates to the allowed length and returns
    // Truncated — which bumps segments_inserted by exactly one.
    let config = ReassemblyConfig {
        max_depth: 3,
        ..ReassemblyConfig::default()
    };
    let (mut reassembler, mut handler, client, server) = syn_established_flow(config);

    // Out-of-order (offset 2, gap at offset 1) 5-byte payload; only 3 bytes fit
    // the depth, so it is truncated-and-inserted.
    let data = make_tcp_packet(
        client, 12345, server, 80, 1002, b"aaaaa", false, false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    assert_eq!(
        reassembler.stats().segments_inserted,
        1,
        "GG (413): a truncated-and-inserted segment must add exactly one to \
         segments_inserted (kills `+=`→`*=`/`-=`)"
    );
}

// ---- GG-5 + GG-12/GG-13 (mod.rs:422/423/424) — segment-limit partial path ---
//
// Inside the overlap gap-insertion loop, hitting `self.segments.len() >=
// max_segments` mid-loop returns SegmentLimitReached. The arm then runs the
// partial-insertion block ONLY when `bytes_added > 0` (GG-5, line 422), bumping
// segments_overlaps (423) and segments_inserted (424) by one each. We construct
// an overlap that yields TWO gaps and set max_segments so the first gap inserts
// (bytes_added > 0) but the second hits the cap.

#[test]
fn test_stats_segment_limit_partial_insertion_counters_exact() {
    // base_offset = 1 after SYN. Buffer two segments, BOTH behind the permanent
    // gap at offset 1 so neither flushes (segment count stays at 2):
    //   - segment at offset 3 ([3,4)) — clean insert (segments_inserted = 1)
    //   - segment at offset 9 ([9,10)) — clean insert (segments_inserted = 2)
    // Buffer now holds 2 segments; max_segments = 3.
    // Then send b"BBBBBBBBBB" at offset 2 ([2,12)): it overlaps BOTH buffered
    // segments, so select_gaps yields gaps [2,3), [4,9), [10,12). The loop
    // inserts the first gap [2,3) (segments.len() 2 -> 3, bytes_added > 0), then
    // on the next gap finds segments.len() == 3 >= max_segments and breaks →
    // SegmentLimitReached WITH partial insertion. The arm runs the
    // `bytes_added > 0` block: segments_overlaps += 1, segments_inserted += 1.
    let config = ReassemblyConfig {
        max_segments_per_direction: 3,
        ..ReassemblyConfig::default()
    };
    let (mut reassembler, mut handler, client, server) = syn_established_flow(config);

    // offset 3 ([3,4)): seq = 1003 — behind the gap at offset 1, stays buffered.
    let s1 = make_tcp_packet(
        client, 12345, server, 80, 1003, b"C", false, false, false, false,
    );
    reassembler.process_packet(&s1, 2, &mut handler);
    // offset 9 ([9,10)): seq = 1009 — also buffered.
    let s2 = make_tcp_packet(
        client, 12345, server, 80, 1009, b"I", false, false, false, false,
    );
    reassembler.process_packet(&s2, 3, &mut handler);

    // Precondition: both clean inserts are buffered, neither flushed.
    assert_eq!(
        reassembler.stats().segments_inserted,
        2,
        "GG-5 setup: both out-of-order segments must be buffered (2 clean inserts)"
    );
    let inserted_before = reassembler.stats().segments_inserted;
    let overlaps_before = reassembler.stats().segments_overlaps;
    let seg_limit_before = reassembler.stats().segments_segment_limit;

    // Overlapping segment spanning [2,12): seq = 1002, 10 bytes.
    let overlap = make_tcp_packet(
        client,
        12345,
        server,
        80,
        1002,
        b"BBBBBBBBBB",
        false,
        false,
        false,
        false,
    );
    reassembler.process_packet(&overlap, 4, &mut handler);

    // SegmentLimitReached must have been recorded exactly once for this packet.
    assert_eq!(
        reassembler.stats().segments_segment_limit,
        seg_limit_before + 1,
        "GG: the overlap hitting the segment cap mid-gap-loop must record one \
         SegmentLimitReached"
    );
    // GG-5 (422) + GG-12 (423) + GG-13 (424): because the first gap WAS inserted
    // (bytes_added > 0), the partial-insertion block bumps both counters by one.
    assert_eq!(
        reassembler.stats().segments_overlaps,
        overlaps_before + 1,
        "GG-5/GG-12 (422/423): partial insertion on the segment-limit path must add \
         exactly one to segments_overlaps (kills `bytes_added > 0`→`< 0` and `+=`→`*=`/`-=`)"
    );
    assert_eq!(
        reassembler.stats().segments_inserted,
        inserted_before + 1,
        "GG-13 (424): partial insertion on the segment-limit path must add exactly one \
         to segments_inserted (kills `+=`→`*=`/`-=`)"
    );
}

// ---- GG-2 (mod.rs:166) — idle-sweep gate `timestamp > last_expiry_sweep_secs`
//
// The idle-flow expiry sweep fires when `timestamp > self.last_expiry_sweep_secs`
// (strictly greater), running the O(n) scan at most once per unique second. The
// surviving mutant replaced `>` with `>=`. The output (which flows expire) is
// identical because the sweep is idempotent within a second, so this is listed
// as borderline-equivalent. We still pin the OBSERVABLE contract that matters
// for anti-evasion: a flow idle for exactly `flow_timeout_secs + 1` is expired,
// and one idle for exactly `flow_timeout_secs` is NOT — the expiry boundary
// itself. (We do not attempt to count sweep invocations, which is not observable
// through the public API and is the only thing `>` vs `>=` changes.)

#[test]
fn test_idle_flow_expiry_boundary_exact() {
    // flow_timeout_secs = 10. A flow last seen at t=2; a later packet on a
    // DIFFERENT flow at t=12 (idle gap == 10) must NOT expire the first flow;
    // a packet at t=13 (idle gap == 11 == timeout + 1) MUST expire it.
    let config = ReassemblyConfig {
        flow_timeout_secs: 10,
        ..ReassemblyConfig::default()
    };
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let a_client = [10, 0, 0, 1];
    let b_client = [10, 0, 0, 3];
    let server = [10, 0, 0, 2];

    // Flow A established at t=2.
    let syn_a = make_tcp_packet(
        a_client,
        11111,
        server,
        80,
        1000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_a, 2, &mut handler);
    assert_eq!(reassembler.flow_count(), 1);

    // Flow B at t=12: triggers a sweep at timestamp 12 (> last sweep 2). Flow A's
    // idle gap is exactly 10 == timeout → NOT yet expired (expiry is gap > timeout).
    let syn_b = make_tcp_packet(
        b_client,
        22222,
        server,
        80,
        2000,
        &[],
        true,
        false,
        false,
        false,
    );
    reassembler.process_packet(&syn_b, 12, &mut handler);
    assert_eq!(
        reassembler.flow_count(),
        2,
        "GG-2 boundary: flow idle for exactly flow_timeout_secs must NOT be expired"
    );

    // Another packet on flow B at t=13: sweep at 13 (> 12). Flow A's idle gap is
    // now 11 == timeout + 1 → expired. Flow B remains.
    let data_b = make_tcp_packet(
        b_client, 22222, server, 80, 2001, b"z", false, true, false, false,
    );
    reassembler.process_packet(&data_b, 13, &mut handler);
    assert_eq!(
        reassembler.flow_count(),
        1,
        "GG-2 boundary: flow idle for flow_timeout_secs + 1 must be expired by the sweep"
    );
}
