use std::net::{IpAddr, Ipv4Addr};

use wirerust::reassembly::flow::{FlowDirection, FlowKey, FlowState, TcpFlow};
use wirerust::reassembly::handler::Direction;

#[test]
fn test_flow_key_canonicalization() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let key_ab = FlowKey::new(ip_a, 12345, ip_b, 80);
    let key_ba = FlowKey::new(ip_b, 80, ip_a, 12345);

    assert_eq!(key_ab, key_ba);
    // Tuple ordering: (10.0.0.1, 12345) < (10.0.0.2, 80) since IPs differ
    assert_eq!(key_ab.lower_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(key_ab.lower_port, 12345);
    assert_eq!(key_ab.upper_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
    assert_eq!(key_ab.upper_port, 80);
}

#[test]
fn test_flow_key_same_ip_different_ports() {
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

    let key1 = FlowKey::new(ip, 80, ip, 12345);
    let key2 = FlowKey::new(ip, 12345, ip, 80);

    assert_eq!(key1, key2);
    assert_eq!(key1.lower_port, 80);
    assert_eq!(key1.upper_port, 12345);
}

#[test]
fn test_flow_direction_determines_client_server() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 12345, ip_server, 80), 1000);
    flow.set_initiator(ip_client, 12345);

    assert_eq!(flow.direction(ip_client, 12345), Direction::ClientToServer);
    assert_eq!(flow.direction(ip_server, 80), Direction::ServerToClient);
}

#[test]
fn test_flow_state_transitions() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 12345, ip_b, 80), 1000);
    assert_eq!(flow.state, FlowState::New);

    flow.on_syn();
    assert_eq!(flow.state, FlowState::SynSent);

    flow.on_syn_ack();
    assert_eq!(flow.state, FlowState::Established);

    flow.on_fin();
    assert_eq!(flow.state, FlowState::Closing);

    flow.on_fin();
    assert_eq!(flow.state, FlowState::Closed);
}

#[test]
fn test_flow_rst_from_any_state() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 12345, ip_b, 80), 1000);
    flow.on_syn();
    assert_eq!(flow.state, FlowState::SynSent);

    flow.on_rst();
    assert_eq!(flow.state, FlowState::Closed);
}

#[test]
fn test_mid_stream_pickup() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));

    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 12345, ip_b, 80), 1000);
    flow.on_data_without_syn();
    assert_eq!(flow.state, FlowState::Established);
    assert!(flow.partial);
}

#[test]
fn test_flow_direction_new() {
    let dir = FlowDirection::new();
    assert_eq!(dir.isn, None);
    assert_eq!(dir.base_offset, 0);
    assert!(dir.segments.is_empty());
    assert_eq!(dir.reassembled_bytes, 0);
    assert!(!dir.fin_seen);
    assert!(!dir.rst_seen);
    assert!(!dir.depth_exceeded);
}
