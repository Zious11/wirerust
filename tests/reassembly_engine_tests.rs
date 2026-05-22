use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::findings::{Confidence, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{CloseReason, Direction, StreamHandler};
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

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
/// Verified after a full FIN teardown (flows_fin=1, flows_rst=0 → flows_completed=1).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_028_flows_completed_derived_correctly() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    // Establish flow and close it via FIN teardown
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

    let stats = reassembler.stats();
    let summary = reassembler.summarize();

    let flows_fin = stats.flows_fin;
    let flows_rst = stats.flows_rst;
    let expected_completed = flows_fin + flows_rst;

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
