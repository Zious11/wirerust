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
/// (other_ip, other_port) as (lower_ip, lower_port) using TUPLE-PAIR
/// comparison (BC-2.04.003 invariant 1).
///
/// Two vectors are exercised:
///
/// Vector 1 — canonical (BC-2.04.003 §Canonical Test Vectors row 1):
/// new(1.1.1.1, 5000, 2.2.2.2, 80) →
/// FlowKey { lower: (1.1.1.1, 5000), upper: (2.2.2.2, 80) }
/// IPs differ so both tuple-pair and per-field agree here.
///
/// Vector 2 — field-crossing discriminator:
/// new(2.2.2.2, 80, 1.1.1.1, 5000) — IP order and port order cross:
///
/// - 1.1.1.1 < 2.2.2.2 (lower IP is 1.1.1.1)
/// - 80 < 5000         (lower port by value is 80)
///
/// Tuple-pair: (1.1.1.1, 5000) < (2.2.2.2, 80) → lower = (1.1.1.1, 5000),
/// lower_port = 5000.
///
/// A buggy per-field sort would yield lower_ip=1.1.1.1, lower_port=80
/// (fabricating an endpoint that exists in neither input).
/// Asserting lower_port == 5000 (NOT 80) is the discriminating claim.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_lower_endpoint_stored_correctly() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));

    // Vector 1: canonical BC test vector — IPs alone determine ordering.
    let key_v1 = FlowKey::new(ip_a, 5000, ip_b, 80);
    assert_eq!(
        key_v1.lower_ip(),
        ip_a,
        "lower_ip must be the smaller IP (1.1.1.1 < 2.2.2.2)"
    );
    assert_eq!(
        key_v1.lower_port(),
        5000,
        "lower_port must be paired with lower_ip via tuple-pair comparison"
    );
    assert_eq!(
        key_v1.upper_ip(),
        ip_b,
        "upper_ip must be the larger IP (2.2.2.2)"
    );
    assert_eq!(
        key_v1.upper_port(),
        80,
        "upper_port must be paired with upper_ip via tuple-pair comparison"
    );

    // Vector 2: field-crossing discriminator — IP order and port order cross.
    // Endpoint A = (2.2.2.2, 80), Endpoint B = (1.1.1.1, 5000).
    // Tuple-pair: (1.1.1.1, 5000) < (2.2.2.2, 80) → lower = (1.1.1.1, 5000).
    // Per-field (buggy): lower_ip=1.1.1.1, lower_port=80 — fabricated endpoint.
    // Asserting lower_port == 5000 discriminates tuple-pair from per-field sort.
    let key_v2 = FlowKey::new(ip_b, 80, ip_a, 5000);
    assert_eq!(
        key_v2.lower_ip(),
        ip_a,
        "field-crossing: tuple-pair places 1.1.1.1 as lower_ip"
    );
    assert_eq!(
        key_v2.lower_port(),
        5000,
        "field-crossing: lower_port must be 5000 (paired with 1.1.1.1), NOT 80 — \
         a per-field sort would wrongly yield 80 here"
    );
    assert_eq!(
        key_v2.upper_ip(),
        ip_b,
        "field-crossing: tuple-pair places 2.2.2.2 as upper_ip"
    );
    assert_eq!(
        key_v2.upper_port(),
        80,
        "field-crossing: upper_port must be 80 (paired with 2.2.2.2)"
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
/// AC-010's stated criterion requires "a case that distinguishes the two
/// orderings." The discriminating vector is one where the IP order and the
/// port order cross:
///
///   Endpoint A = (2.2.2.2, 80)   — larger IP, smaller port
///   Endpoint B = (1.1.1.1, 5000) — smaller IP, larger port
///
/// Tuple-pair comparison:
///   (1.1.1.1, 5000) < (2.2.2.2, 80) because 1.1.1.1 < 2.2.2.2
///   → lower = (1.1.1.1, 5000), lower_port = 5000
///
/// A buggy per-field sort (sort IPs independently, sort ports independently):
///   lower_ip = min(2.2.2.2, 1.1.1.1) = 1.1.1.1
///   lower_port = min(80, 5000) = 80
///   → fabricated endpoint (1.1.1.1, 80), which exists in NEITHER input
///   → lower_port would be 80, not 5000
///
/// Asserting lower_port == 5000 (not 80) genuinely fails against a
/// per-field-sort implementation while passing against the correct
/// tuple-pair implementation (BC-2.04.003 invariant 1).
///
/// A same-IP case (BC-2.04.003 §Canonical Test Vectors rows 3–4) is also
/// retained for completeness; that case does not discriminate the orderings
/// on its own but is a valid canonical vector.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_003_tuple_pair_ordering_not_independent_field() {
    let ip_lo = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)); // smaller IP
    let ip_hi = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2)); // larger IP

    // ---- Discriminating field-crossing vector (AC-010 requirement) ----
    // Endpoint A = (2.2.2.2, 80): larger IP, smaller port.
    // Endpoint B = (1.1.1.1, 5000): smaller IP, larger port.
    // Tuple-pair: (1.1.1.1, 5000) < (2.2.2.2, 80) → lower = (1.1.1.1, 5000).
    let key_cross = FlowKey::new(ip_hi, 80, ip_lo, 5000);
    assert_eq!(
        key_cross.lower_ip(),
        ip_lo,
        "field-crossing: tuple-pair places 1.1.1.1 as lower_ip (IP ordering wins)"
    );
    assert_eq!(
        key_cross.lower_port(),
        5000,
        "field-crossing: lower_port must be 5000 (paired with 1.1.1.1) — \
         a per-field sort would wrongly yield 80 here, distinguishing the two orderings"
    );
    assert_eq!(
        key_cross.upper_ip(),
        ip_hi,
        "field-crossing: 2.2.2.2 is the upper endpoint"
    );
    assert_eq!(
        key_cross.upper_port(),
        80,
        "field-crossing: upper_port must be 80 (paired with 2.2.2.2)"
    );

    // Commutativity: reversed argument order must produce the identical key.
    let key_cross_rev = FlowKey::new(ip_lo, 5000, ip_hi, 80);
    assert_eq!(
        key_cross, key_cross_rev,
        "field-crossing vector must be commutative"
    );

    // ---- Same-IP canonical vector (BC-2.04.003 §Canonical Test Vectors rows 3–4) ----
    // Retained as a canonical BC vector; does not on its own discriminate orderings.
    let ip = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let key_same_ip = FlowKey::new(ip, 443, ip, 55000);
    assert_eq!(key_same_ip.lower_ip(), ip);
    assert_eq!(
        key_same_ip.lower_port(),
        443,
        "same-IP: tuple-pair ordering — lower port (443) wins"
    );
    assert_eq!(key_same_ip.upper_ip(), ip);
    assert_eq!(key_same_ip.upper_port(), 55000);

    let key_same_ip_rev = FlowKey::new(ip, 55000, ip, 443);
    assert_eq!(
        key_same_ip, key_same_ip_rev,
        "same-IP different-port keys must be commutative"
    );
    assert_eq!(
        key_same_ip_rev.lower_port(),
        443,
        "same-IP reversed: must produce same canonical ordering"
    );
    assert_eq!(key_same_ip_rev.upper_port(), 55000);
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

// ---------------------------------------------------------------------------
// STORY-013: BC-2.04.004, BC-2.04.005, BC-2.04.050, BC-2.04.051,
//            BC-2.04.052, BC-2.04.053
//
// TCP three-way handshake state machine and direction tagging.
// AC-001..AC-016 and EC-001..EC-010 (story spec + BC postconditions/invariants).
// Test names are prescribed by the story spec (W1.4 decision).
// ---------------------------------------------------------------------------

// ---- AC-001 to AC-016: RED GATE stubs ----

/// AC-001 (BC-2.04.004 postcondition 1)
/// Postcondition: after on_syn() from src_ip:src_port, flow.initiator == Some((src_ip, src_port)).
/// set_initiator records the source endpoint as the initiator.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_syn_sets_initiator() {
    panic!("RED GATE: AC-001 not yet verified");
}

/// AC-002 (BC-2.04.004 postcondition 2)
/// Postcondition: after processing a SYN, the ClientToServer direction
/// has isn == Some(tcp.seq).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_syn_sets_client_isn() {
    panic!("RED GATE: AC-002 not yet verified");
}

/// AC-003 (BC-2.04.004 postcondition 3)
/// Postcondition: after processing a SYN, flow.state == FlowState::SynSent.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_syn_transitions_to_synsent() {
    panic!("RED GATE: AC-003 not yet verified");
}

/// AC-004 (BC-2.04.004 invariants 1-2)
/// Invariant: set_initiator and set_isn are idempotent — a retransmitted SYN
/// does not change the stored initiator or ISN.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_retransmitted_syn_is_idempotent() {
    panic!("RED GATE: AC-004 not yet verified");
}

/// AC-005 (BC-2.04.005 postcondition 1)
/// Postcondition: after processing a SYN+ACK, flow.initiator == Some((dst_ip, dst_port))
/// — the DESTINATION of the SYN+ACK is the initiator.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_syn_ack_sets_initiator_to_dst() {
    panic!("RED GATE: AC-005 not yet verified");
}

/// AC-006 (BC-2.04.005 postconditions 2-3)
/// Postcondition: after processing SYN+ACK, the server-to-client direction has
/// isn == Some(tcp.seq) and flow.state == FlowState::Established.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_syn_ack_establishes_flow() {
    panic!("RED GATE: AC-006 not yet verified");
}

/// AC-007 (BC-2.04.005 invariant 3)
/// Invariant: a SYN+ACK received without a prior SYN (mid-capture) still
/// transitions the flow from New directly to Established.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_syn_ack_without_prior_syn() {
    panic!("RED GATE: AC-007 not yet verified");
}

/// AC-008 (BC-2.04.050 postcondition — all 9 transitions)
/// Each of the 9 rows in the BC-2.04.050 state transition table is verified
/// individually within this single test function.
///
/// Rows:
///   1. on_syn()                New       → SynSent
///   2. on_syn()                SynSent   → SynSent (no-op guard)
///   3. on_syn_ack()            SynSent   → Established
///   4. on_syn_ack()            New       → Established (server-first)
///   5. on_data_without_syn()   New       → Established + partial=true
///   6. on_fin() (first)        Established → Closing
///   7. on_fin() (first)        SynSent   → Closing
///   8. on_fin() (second, fin_count >= 2) any → Closed
///   9. on_rst()                any       → Closed
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_state_machine_all_transitions() {
    panic!("RED GATE: AC-008 not yet verified");
}

/// AC-009 (BC-2.04.050 invariant 1)
/// Invariant: on_syn() is a no-op when flow is already in SynSent, Established,
/// Closing, or Closed state.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_on_syn_no_op_when_not_new() {
    panic!("RED GATE: AC-009 not yet verified");
}

/// AC-010 (BC-2.04.050 invariant 4)
/// Invariant: fin_count uses saturating_add(1) to prevent u8 overflow at 255.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_fin_count_saturates_at_255() {
    panic!("RED GATE: AC-010 not yet verified");
}

/// AC-011 (BC-2.04.051 invariant 1)
/// Invariant: on_rst() transitions to Closed from any prior state (New, SynSent,
/// Established, Closing, Closed) with no state guard.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_rst_closes_from_any_state() {
    panic!("RED GATE: AC-011 not yet verified");
}

/// AC-012 (BC-2.04.052 postconditions 1-2)
/// Postcondition: on_data_without_syn() on a New flow transitions state to
/// Established and sets partial = true.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_052_data_without_syn_sets_partial() {
    panic!("RED GATE: AC-012 not yet verified");
}

/// AC-013 (BC-2.04.052 invariant 1)
/// Invariant: on_data_without_syn() is a no-op when flow is already Established.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_052_on_data_without_syn_no_op_when_established() {
    panic!("RED GATE: AC-013 not yet verified");
}

/// AC-014 (BC-2.04.053 postcondition 1)
/// Postcondition: direction(src_ip, src_port) returns ClientToServer when
/// src_ip:src_port matches the stored initiator.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_direction_client_to_server_when_src_is_initiator() {
    panic!("RED GATE: AC-014 not yet verified");
}

/// AC-015 (BC-2.04.053 postcondition 2)
/// Postcondition: direction(src_ip, src_port) returns ServerToClient when
/// src_ip:src_port does NOT match the stored initiator.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_direction_server_to_client_when_src_is_not_initiator() {
    panic!("RED GATE: AC-015 not yet verified");
}

/// AC-016 (BC-2.04.053 invariant 2)
/// Invariant: when initiator is None, direction() returns ServerToClient as a
/// conservative default.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_direction_server_to_client_when_no_initiator() {
    panic!("RED GATE: AC-016 not yet verified");
}

// ---- EC-001..EC-010: edge-case stubs ----

/// EC-001 (BC-2.04.004 edge case — retransmitted SYN)
/// set_initiator/set_isn no-ops; state stays SynSent after a second SYN.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_ec001_retransmitted_syn_state_unchanged() {
    panic!("RED GATE: EC-001 not yet verified");
}

/// EC-002 (BC-2.04.005 edge case — SYN+ACK without prior SYN)
/// initiator = dst_ip:dst_port; state → Established from New.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_ec002_syn_ack_first_sets_initiator_to_dst() {
    panic!("RED GATE: EC-002 not yet verified");
}

/// EC-003 (BC-2.04.005 edge case — SYN+ACK retransmission)
/// All setters idempotent; if already Established, on_syn_ack() is a no-op.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_ec003_syn_ack_retransmission_is_idempotent() {
    panic!("RED GATE: EC-003 not yet verified");
}

/// EC-004 (BC-2.04.004 edge case — SYN with payload)
/// ISN is set; the payload would be processed separately by the engine.
/// At the TcpFlow level: set_initiator + set_isn + on_syn all succeed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_ec004_syn_with_payload_sets_isn() {
    panic!("RED GATE: EC-004 not yet verified");
}

/// EC-005 (BC-2.04.051 edge case — RST on New flow)
/// state = Closed; the flow level confirms unconditional close from New.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec005_rst_on_new_flow() {
    panic!("RED GATE: EC-005 not yet verified");
}

/// EC-006 (BC-2.04.051 edge case — RST on Closing flow)
/// state = Closed; on_rst() from Closing is unconditional.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec006_rst_on_closing_flow() {
    panic!("RED GATE: EC-006 not yet verified");
}

/// EC-007 (BC-2.04.051 invariant 2 — RST with payload)
/// At the TcpFlow level: on_rst() sets state = Closed regardless.
/// Payload suppression is tested at engine level; here we confirm state transition.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec007_rst_closes_flow_state() {
    panic!("RED GATE: EC-007 not yet verified");
}

/// EC-008 (BC-2.04.050 edge case — both FINs from same direction)
/// fin_count reaches 2 via two on_fin() calls; flow transitions to Closed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_ec008_both_fins_same_direction_closes_flow() {
    panic!("RED GATE: EC-008 not yet verified");
}

/// EC-009 (BC-2.04.050 edge case — FIN on New flow)
/// on_fin() from New state: fin_count = 1; but New is not in {Established, SynSent},
/// so state does NOT transition to Closing (no-op for state; fin_count increments).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_ec009_fin_on_new_flow() {
    panic!("RED GATE: EC-009 not yet verified");
}

/// EC-010 (BC-2.04.053 invariant 2 — initiator = None)
/// When initiator is None, direction() returns ServerToClient (conservative default).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_ec010_direction_none_initiator_returns_server_to_client() {
    panic!("RED GATE: EC-010 not yet verified");
}
