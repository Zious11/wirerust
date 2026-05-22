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
/// Canonical test vector (BC-2.04.003 §Canonical Test Vectors row 1):
/// new(1.1.1.1, 5000, 2.2.2.2, 80) →
/// FlowKey { lower: (1.1.1.1, 5000), upper: (2.2.2.2, 80) }
///
/// (1.1.1.1, 5000) < (2.2.2.2, 80) because 1.1.1.1 < 2.2.2.2 as IPv4 addresses.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_lower_endpoint_stored_correctly() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

    let key = FlowKey::new(ip_a, 5000, ip_b, 80);

    assert_eq!(
        key.lower_ip(),
        ip_a,
        "lower_ip must be the smaller IP (1.1.1.1 < 2.2.2.2)"
    );
    assert_eq!(
        key.lower_port(),
        5000,
        "lower_port must be paired with lower_ip (tuple-pair comparison)"
    );
    assert_eq!(
        key.upper_ip(),
        ip_b,
        "upper_ip must be the larger IP (2.2.2.2)"
    );
    assert_eq!(
        key.upper_port(),
        80,
        "upper_port must be paired with upper_ip (tuple-pair comparison)"
    );
}

/// AC-009 (BC-2.04.003 postcondition 2)
/// Postcondition: FlowKey::new(ip_a, port_a, ip_b, port_b) ==
/// FlowKey::new(ip_b, port_b, ip_a, port_a) for all valid inputs.
///
/// Canonical test vectors (BC-2.04.003 §Canonical Test Vectors rows 1–2):
///
/// - new(1.1.1.1, 5000, 2.2.2.2, 80) == new(2.2.2.2, 80, 1.1.1.1, 5000)
///
/// Both must map to FlowKey { lower: (1.1.1.1, 5000), upper: (2.2.2.2, 80) }.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_flow_key_is_commutative() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

    let key_ab = FlowKey::new(ip_a, 5000, ip_b, 80);
    let key_ba = FlowKey::new(ip_b, 80, ip_a, 5000);

    assert_eq!(
        key_ab, key_ba,
        "FlowKey::new must be commutative: A->B and B->A must produce identical keys"
    );
    // Both must have the same canonical ordering.
    assert_eq!(key_ba.lower_ip(), ip_a);
    assert_eq!(key_ba.lower_port(), 5000);
    assert_eq!(key_ba.upper_ip(), ip_b);
    assert_eq!(key_ba.upper_port(), 80);
}

/// AC-010 (BC-2.04.003 invariant 1)
/// Invariant: ordering is TUPLE-PAIR comparison (ip_a, port_a) <=
/// (ip_b, port_b), NOT independent per-field sorting.
///
/// The distinguishing case is identical IPs with different ports — independent
/// per-field sorting would sort by IP (tie) then by port (same result as tuple
/// comparison in this degenerate case), but for clarity we also test a case
/// where ports are in the "opposite" order from what independent sorting would
/// predict if it sorted ports separately: same IP, port_a=55000, port_b=443.
/// Under independent sorting, IP tie → sort by port → 443 < 55000, so 443 is
/// lower. Under tuple-pair: (ip, 55000) vs (ip, 443): 443 < 55000, so the
/// 443 side is lower. Both orderings agree here, so the real distinguishing
/// test is the canonical test vector from the BC.
///
/// Canonical test vector (BC-2.04.003 §Canonical Test Vectors rows 3–4):
/// new(1.1.1.1, 443, 1.1.1.1, 55000) → lower_port=443, upper_port=55000.
/// new(1.1.1.1, 55000, 1.1.1.1, 443) → same key (commutativity + tuple ordering).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_tuple_pair_ordering_not_independent_field() {
    let ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

    // Tuple-pair: (ip, 443) < (ip, 55000) → 443 is lower.
    let key1 = FlowKey::new(ip, 443, ip, 55000);
    assert_eq!(key1.lower_ip(), ip);
    assert_eq!(
        key1.lower_port(),
        443,
        "tuple-pair ordering: lower port (443) wins"
    );
    assert_eq!(key1.upper_ip(), ip);
    assert_eq!(key1.upper_port(), 55000);

    // Same key from the reversed argument order (commutativity check).
    let key2 = FlowKey::new(ip, 55000, ip, 443);
    assert_eq!(
        key1, key2,
        "same-IP different-port keys must be commutative"
    );
    assert_eq!(
        key2.lower_port(),
        443,
        "reversed input must produce same canonical ordering"
    );
    assert_eq!(key2.upper_port(), 55000);
}

/// AC-011 (BC-2.04.049 postcondition 1)
/// Postcondition: format!("{}", flow_key) contains U+2192 (UTF-8 bytes
/// 0xE2 0x86 0x92) and does NOT contain ASCII "->" (0x2D 0x3E).
///
/// This test asserts exact UTF-8 bytes to catch any future source-edit
/// that replaces the Unicode arrow with its ASCII lookalike.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_049_display_uses_unicode_arrow() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    let ip_b = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
    // Ensure canonical ordering: 1.2.3.4 < 5.6.7.8
    let key = FlowKey::new(ip_a, 80, ip_b, 443);

    let display = format!("{key}");
    let bytes = display.as_bytes();

    // Assert U+2192 (→) is present: UTF-8 bytes 0xE2 0x86 0x92.
    let unicode_arrow: &[u8] = &[0xE2, 0x86, 0x92];
    assert!(
        bytes.windows(3).any(|w| w == unicode_arrow),
        "Display must contain U+2192 (UTF-8: E2 86 92) — got: {display:?}"
    );

    // Assert ASCII "->" (0x2D 0x3E) is NOT present.
    let ascii_arrow: &[u8] = &[0x2D, 0x3E];
    assert!(
        !bytes.windows(2).any(|w| w == ascii_arrow),
        "Display must NOT contain ASCII '->' (0x2D 0x3E) — got: {display:?}"
    );
}

/// AC-012 (BC-2.04.049 invariant 1)
/// Invariant: the lower (canonically-ordered) endpoint appears on the left
/// side of the U+2192 arrow and the upper endpoint on the right.
///
/// BC-2.04.049 §Canonical Test Vectors row 1:
/// FlowKey{lower=1.2.3.4:80, upper=5.6.7.8:443} →
/// "1.2.3.4:80 → 5.6.7.8:443"
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_049_display_canonical_order() {
    let ip_lower = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    let ip_upper = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
    // Arguments in reverse order — FlowKey::new must canonicalize.
    let key = FlowKey::new(ip_upper, 443, ip_lower, 80);

    let display = format!("{key}");
    // The Unicode arrow U+2192 is '→'.
    let arrow = "\u{2192}";
    let parts: Vec<&str> = display.splitn(2, arrow).collect();
    assert_eq!(
        parts.len(),
        2,
        "Display must contain exactly one U+2192 arrow separator"
    );

    let left = parts[0].trim();
    let right = parts[1].trim();

    assert!(
        left.starts_with("1.2.3.4"),
        "lower endpoint (1.2.3.4) must appear on the LEFT of the arrow — got left={left:?}"
    );
    assert!(
        left.ends_with(":80"),
        "lower endpoint port (80) must appear on the LEFT — got left={left:?}"
    );
    assert!(
        right.starts_with("5.6.7.8"),
        "upper endpoint (5.6.7.8) must appear on the RIGHT of the arrow — got right={right:?}"
    );
    assert!(
        right.ends_with(":443"),
        "upper endpoint port (443) must appear on the RIGHT — got right={right:?}"
    );
}

// ---------------------------------------------------------------------------
// EC-008..EC-010 — edge-case tests from STORY-011 edge-case table
// ---------------------------------------------------------------------------

/// EC-008 (BC-2.04.003 edge case — same IP, different ports)
/// Same IP on both sides, port_a=22, port_b=50000.
/// Tuple-pair ordering: (ip, 22) < (ip, 50000) → port 22 must be the lower
/// endpoint. This exercises the tuple-pair comparison when IPs are equal.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_ec008_same_ip_different_ports_lower_port_wins() {
    let ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

    // Forward direction: (ip, 22, ip, 50000)
    let key_fwd = FlowKey::new(ip, 22, ip, 50000);
    assert_eq!(
        key_fwd.lower_port(),
        22,
        "port 22 < 50000: must be lower endpoint"
    );
    assert_eq!(key_fwd.upper_port(), 50000);

    // Reverse direction: (ip, 50000, ip, 22) must produce the same key.
    let key_rev = FlowKey::new(ip, 50000, ip, 22);
    assert_eq!(
        key_fwd, key_rev,
        "same-IP different-port keys must be commutative"
    );
    assert_eq!(
        key_rev.lower_port(),
        22,
        "reversed input must still yield port 22 as lower"
    );
}

/// EC-009 (BC-2.04.003 edge case — IPv4 vs IPv6)
/// BC-2.04.003 §Preconditions ¶2: "IPv4 < IPv6 in Rust's PartialOrd."
/// FlowKey with one IPv4 and one IPv6 endpoint must place the IPv4 endpoint
/// as the lower endpoint.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_ec009_ipv4_lower_than_ipv6() {
    let ip_v4 = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let ip_v6 = IpAddr::V6(Ipv6Addr::new(0x2001, 0x0db8, 0, 0, 0, 0, 0, 1));

    // IPv4 < IPv6 in IpAddr PartialOrd; IPv4 endpoint must be lower.
    let key = FlowKey::new(ip_v4, 8080, ip_v6, 443);
    assert_eq!(
        key.lower_ip(),
        ip_v4,
        "IPv4 address must be the lower endpoint (IPv4 < IPv6 in Rust PartialOrd)"
    );
    assert_eq!(key.lower_port(), 8080);
    assert_eq!(key.upper_ip(), ip_v6);
    assert_eq!(key.upper_port(), 443);

    // Commutativity: reversed arguments must produce the same key.
    let key_rev = FlowKey::new(ip_v6, 443, ip_v4, 8080);
    assert_eq!(key, key_rev, "IPv4/IPv6 mixed key must be commutative");
}

/// EC-010 (BC-2.04.049 edge case — IPv6 Display without RFC-3986 brackets)
/// BC-2.04.049 §Edge Cases EC-002: "IpAddr::fmt does NOT add RFC-3986
/// brackets; IPv6 addresses render bracket-free in this format."
///
/// Verifies that an IPv6 endpoint in the display string does not contain
/// "[" or "]" characters, and does contain the expected colon-separated
/// IPv6 address notation.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_049_ec010_display_ipv6_no_rfc3986_brackets() {
    // Use ::1 (loopback) as the IPv6 address — easy to reason about display.
    let ip_v4 = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_v6 = IpAddr::V6(Ipv6Addr::LOCALHOST); // ::1

    // IPv4 < IPv6, so ip_v4 is the lower endpoint.
    let key = FlowKey::new(ip_v4, 1234, ip_v6, 80);
    let display = format!("{key}");

    // No RFC-3986 brackets in the display output.
    assert!(
        !display.contains('['),
        "FlowKey Display must not add '[' RFC-3986 bracket — got: {display:?}"
    );
    assert!(
        !display.contains(']'),
        "FlowKey Display must not add ']' RFC-3986 bracket — got: {display:?}"
    );

    // The IPv6 address ::1 must appear in colon-separated form.
    // Rust's IpAddr::fmt for ::1 renders as "::1".
    assert!(
        display.contains("::1"),
        "IPv6 ::1 must appear as '::1' (no brackets) — got: {display:?}"
    );

    // The U+2192 arrow must still be present.
    let unicode_arrow: &[u8] = &[0xE2, 0x86, 0x92];
    assert!(
        display.as_bytes().windows(3).any(|w| w == unicode_arrow),
        "Display with IPv6 endpoint must still use U+2192 arrow — got: {display:?}"
    );
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
        any::<[u8; 16]>().prop_map(|bytes| IpAddr::V6(Ipv6Addr::from(bytes)))
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
        ///
        /// BC-2.04.003 §Verification Properties VP-001: "FlowKey::new is commutative:
        /// new(a,pa,b,pb) == new(b,pb,a,pa) for all (a,pa,b,pb). Proof method:
        /// proptest: generate random IP+port pairs."
        ///
        /// Generates 1024 random (IpAddr, port) pairs including both IPv4 and IPv6
        /// addresses. For each pair, constructs FlowKey in both argument orders and
        /// asserts equality. Also verifies that both keys share the same lower/upper
        /// field values, confirming the canonical ordering is deterministic.
        #[test]
        #[allow(non_snake_case)]
        fn test_BC_2_04_003_proptest_flowkey_commutativity(
            ip_a in arb_ip(),
            port_a in any::<u16>(),
            ip_b in arb_ip(),
            port_b in any::<u16>(),
        ) {
            let key_ab = FlowKey::new(ip_a, port_a, ip_b, port_b);
            let key_ba = FlowKey::new(ip_b, port_b, ip_a, port_a);

            // BC-2.04.003 postcondition 2: commutativity.
            prop_assert_eq!(
                &key_ab,
                &key_ba,
                "FlowKey::new must be commutative for ({}:{}, {}:{})",
                ip_a, port_a, ip_b, port_b
            );

            // Both keys must agree on lower/upper fields (structural equality
            // beyond the PartialEq impl, as a redundant sanity check).
            prop_assert_eq!(key_ab.lower_ip(), key_ba.lower_ip());
            prop_assert_eq!(key_ab.lower_port(), key_ba.lower_port());
            prop_assert_eq!(key_ab.upper_ip(), key_ba.upper_ip());
            prop_assert_eq!(key_ab.upper_port(), key_ba.upper_port());
        }
    }
}
