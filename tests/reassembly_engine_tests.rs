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
