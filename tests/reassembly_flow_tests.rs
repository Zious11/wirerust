use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

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
    assert_eq!(key_ab.lower_ip(), IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(key_ab.lower_port(), 12345);
    assert_eq!(key_ab.upper_ip(), IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
    assert_eq!(key_ab.upper_port(), 80);
}

#[test]
fn test_flow_key_same_ip_different_ports() {
    let ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));

    let key1 = FlowKey::new(ip, 80, ip, 12345);
    let key2 = FlowKey::new(ip, 12345, ip, 80);

    assert_eq!(key1, key2);
    assert_eq!(key1.lower_port(), 80);
    assert_eq!(key1.upper_port(), 12345);
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
    assert!(dir.segments_is_empty());
    assert_eq!(dir.reassembled_bytes, 0);
    assert_eq!(dir.buffered_bytes(), 0);
    assert!(!dir.fin_seen);
    assert!(!dir.rst_seen);
    assert!(!dir.depth_exceeded);
}

// ---------------------------------------------------------------------------
// STORY-011: BC-2.04.003 — FlowKey canonical commutative ordering
//            BC-2.04.049 — FlowKey Display uses U+2192 arrow
//
// AC-008 through AC-012, EC-008..EC-010, and the VP-001 proptest for
// FlowKey commutativity.  Test names are prescribed by the story spec
// (W1.4 decision); each test's doc comment cites the BC postcondition
// or invariant being exercised.
// ---------------------------------------------------------------------------

// ---- RED GATE stubs — all bodies are panic!("RED GATE: ...") so every
//      test fails before implementation verification begins.
// ---- After RED GATE is verified, stubs are replaced with real assertions.

/// AC-008 (BC-2.04.003 postcondition 1)
/// Postcondition: FlowKey::new stores the endpoint where (ip, port) <=
/// (other_ip, other_port) as (lower_ip, lower_port) using tuple-pair
/// comparison.
///
/// Canonical test vector: new(1.1.1.1, 5000, 2.2.2.2, 80) →
/// FlowKey { lower: (1.1.1.1, 5000), upper: (2.2.2.2, 80) }
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_lower_endpoint_stored_correctly() {
    panic!("RED GATE: AC-008 not yet verified");
}

/// AC-009 (BC-2.04.003 postcondition 2)
/// Postcondition: FlowKey::new(ip_a, port_a, ip_b, port_b) ==
/// FlowKey::new(ip_b, port_b, ip_a, port_a) for all valid inputs.
///
/// Canonical test vector: new(2.2.2.2, 80, 1.1.1.1, 5000) must equal
/// new(1.1.1.1, 5000, 2.2.2.2, 80).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_flow_key_is_commutative() {
    panic!("RED GATE: AC-009 not yet verified");
}

/// AC-010 (BC-2.04.003 invariant 1)
/// Invariant: ordering is TUPLE-PAIR comparison (ip_a, port_a) <=
/// (ip_b, port_b), NOT independent per-field sorting.
///
/// Distinguishing case: same IP, port_a=443, port_b=55000.
/// Canonical test vector: new(1.1.1.1, 443, 1.1.1.1, 55000) →
/// lower_port=443, upper_port=55000.
/// new(1.1.1.1, 55000, 1.1.1.1, 443) must produce the same key.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_tuple_pair_ordering_not_independent_field() {
    panic!("RED GATE: AC-010 not yet verified");
}

/// AC-011 (BC-2.04.049 postcondition 1)
/// Postcondition: format!("{}", flow_key) contains U+2192 (UTF-8 bytes
/// 0xE2 0x86 0x92) and does NOT contain ASCII "->" (0x2D 0x3E).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_049_display_uses_unicode_arrow() {
    panic!("RED GATE: AC-011 not yet verified");
}

/// AC-012 (BC-2.04.049 invariant 1)
/// Invariant: the lower (canonically-ordered) endpoint appears on the left
/// side of the U+2192 arrow and the upper endpoint on the right.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_049_display_canonical_order() {
    panic!("RED GATE: AC-012 not yet verified");
}

// ---------------------------------------------------------------------------
// EC-008..EC-010 — edge-case tests from STORY-011 edge-case table
// ---------------------------------------------------------------------------

/// EC-008 (BC-2.04.003 edge case)
/// Same IP on both sides, port_a < port_b: tuple-pair ordering must
/// select port_a as the lower endpoint.
/// Scenario: ip=1.1.1.1, port_a=22, port_b=50000.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_ec008_same_ip_different_ports_lower_port_wins() {
    panic!("RED GATE: EC-008 not yet verified");
}

/// EC-009 (BC-2.04.003 edge case)
/// IPv4 vs IPv6 in FlowKey: IpAddr PartialOrd places IPv4 < IPv6, so the
/// IPv4 endpoint must become the lower endpoint.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_ec009_ipv4_lower_than_ipv6() {
    panic!("RED GATE: EC-009 not yet verified");
}

/// EC-010 (BC-2.04.049 edge case)
/// FlowKey Display with an IPv6 endpoint must NOT add RFC-3986 brackets;
/// IPv6 addresses render in plain colon-separated notation via IpAddr::fmt.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_049_ec010_display_ipv6_no_rfc3986_brackets() {
    panic!("RED GATE: EC-010 not yet verified");
}

// ---------------------------------------------------------------------------
// Task 6 / VP-001: property-based test for FlowKey commutativity
// Uses proptest to generate random (ip, port) pairs and verify that
// FlowKey::new(a, pa, b, pb) == FlowKey::new(b, pb, a, pa) for all inputs.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod proptest_flowkey {
    use super::*;
    use proptest::prelude::*;

    /// Generate a random IPv4 address as four bytes.
    fn arb_ipv4() -> impl Strategy<Value = IpAddr> {
        (any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())
            .prop_map(|(a, b, c, d)| IpAddr::V4(Ipv4Addr::new(a, b, c, d)))
    }

    /// Generate a random IPv6 address as sixteen bytes.
    fn arb_ipv6() -> impl Strategy<Value = IpAddr> {
        any::<[u8; 16]>()
            .prop_map(|bytes| IpAddr::V6(Ipv6Addr::from(bytes)))
    }

    /// Generate either an IPv4 or IPv6 address (50/50).
    fn arb_ip() -> impl Strategy<Value = IpAddr> {
        prop_oneof![arb_ipv4(), arb_ipv6()]
    }

    proptest! {
        #![proptest_config(proptest::test_runner::Config {
            cases: 1024,
            ..Default::default()
        })]

        /// VP-001 (BC-2.04.003 postcondition 2 / invariant 1)
        /// Property: FlowKey::new is commutative for all valid (ip, port) pairs.
        /// FlowKey::new(ip_a, port_a, ip_b, port_b) must equal
        /// FlowKey::new(ip_b, port_b, ip_a, port_a) regardless of ordering.
        ///
        /// RED GATE stub: prop_assert!(false) so this test always fails before
        /// implementation verification begins.
        #[test]
        #[allow(non_snake_case)]
        fn test_BC_2_04_003_proptest_flowkey_commutativity(
            _ip_a in arb_ip(),
            _port_a in any::<u16>(),
            _ip_b in arb_ip(),
            _port_b in any::<u16>(),
        ) {
            // RED GATE stub: always fails — replace with real commutativity assertion
            prop_assert!(false, "RED GATE: VP-001 proptest not yet verified");
        }
    }
}
