use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
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
            ack: false,
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
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Data packets
    let p1 = make_tcp_packet(client, 12345, server, 80, 1001, b"aaa", false, false, false);
    reassembler.process_packet(&p1, 2, &mut handler);

    let p2 = make_tcp_packet(client, 12345, server, 80, 1004, b"bbb", false, false, false);
    reassembler.process_packet(&p2, 3, &mut handler);

    let p3 = make_tcp_packet(client, 12345, server, 80, 1007, b"ccc", false, false, false);
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
    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    // Send packets [1, 3, 2]
    let p1 = make_tcp_packet(client, 12345, server, 80, 1001, b"aaa", false, false, false);
    reassembler.process_packet(&p1, 2, &mut handler);

    let p3 = make_tcp_packet(client, 12345, server, 80, 1007, b"ccc", false, false, false);
    reassembler.process_packet(&p3, 3, &mut handler);
    assert_eq!(handler.data_events.len(), 1); // only p1 flushed

    let p2 = make_tcp_packet(client, 12345, server, 80, 1004, b"bbb", false, false, false);
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
        client, 12345, server, 80, 5000, b"hello", false, false, false,
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

    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 1, &mut handler);

    let data = make_tcp_packet(
        client, 12345, server, 80, 1001, b"data", false, false, false,
    );
    reassembler.process_packet(&data, 2, &mut handler);

    let rst = make_tcp_packet(server, 80, client, 12345, 2000, &[], false, false, true);
    reassembler.process_packet(&rst, 3, &mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Rst);
}

#[test]
fn test_finalize_flushes_remaining() {
    let config = ReassemblyConfig::default();
    let mut reassembler = TcpReassembler::new(config);
    let mut handler = RecordingHandler::new();

    let client = [10, 0, 0, 1];
    let server = [10, 0, 0, 2];

    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
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

    let syn = make_tcp_packet(client, 12345, server, 80, 1000, &[], true, false, false);
    reassembler.process_packet(&syn, 100, &mut handler);

    // Expire at time 200 (100 seconds later, > 10s timeout)
    reassembler.expire_flows(200, &mut handler);

    assert_eq!(handler.close_events.len(), 1);
    assert_eq!(handler.close_events[0].1, CloseReason::Timeout);

    let stats = reassembler.stats();
    assert_eq!(stats.flows_expired, 1);
}
