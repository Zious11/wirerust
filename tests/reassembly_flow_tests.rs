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
/// Postcondition: after set_initiator(src_ip, src_port) (called by apply_handshake_flags
/// for a SYN), flow.initiator == Some((src_ip, src_port)).
/// Canonical test vector: SYN from 1.1.1.1:5000 → initiator=(1.1.1.1, 5000).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_syn_sets_initiator() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // Simulate SYN processing: set_initiator called with src endpoint.
    flow.set_initiator(ip_client, 5000);

    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "BC-2.04.004 post-1: initiator must be set to the SYN source endpoint"
    );
    // Confirm via direction: only ClientToServer if initiator matches.
    assert_eq!(
        flow.direction(ip_server, 80),
        Direction::ServerToClient,
        "BC-2.04.004 post-1: server endpoint must yield ServerToClient direction"
    );
}

/// AC-002 (BC-2.04.004 postcondition 2)
/// Postcondition: after processing a SYN, the ClientToServer direction
/// has isn == Some(tcp.seq), and base_offset == 1 (ISN+1 is first data byte).
/// Canonical test vector: SYN seq=1000 → c2s.isn=Some(1000), base_offset=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_syn_sets_client_isn() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // Simulate SYN processing: set_initiator, then set_isn on the c2s direction.
    flow.set_initiator(ip_client, 5000);
    let dir = flow.direction(ip_client, 5000);
    flow.get_direction_mut(dir).set_isn(1000);

    assert_eq!(
        flow.client_to_server.isn,
        Some(1000),
        "BC-2.04.004 post-2: ClientToServer ISN must be set to tcp.seq=1000"
    );
    assert_eq!(
        flow.client_to_server.base_offset, 1,
        "BC-2.04.004 post-2: base_offset must be 1 (ISN+1 is first data byte)"
    );
}

/// AC-003 (BC-2.04.004 postcondition 3)
/// Postcondition: after on_syn(), flow.state == FlowState::SynSent.
/// Canonical test vector: New flow, on_syn() → state=SynSent.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_syn_transitions_to_synsent() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 5000, ip_b, 80), 0);

    assert_eq!(
        flow.state,
        FlowState::New,
        "precondition: flow starts in New state"
    );
    flow.on_syn();
    assert_eq!(
        flow.state,
        FlowState::SynSent,
        "BC-2.04.004 post-3: on_syn() from New must transition to SynSent"
    );
}

/// AC-004 (BC-2.04.004 invariants 1-2)
/// Invariant: set_initiator and set_isn are idempotent — a retransmitted SYN
/// does not change the stored initiator or ISN.
/// Canonical test vector: two SYNs from same source → ISN and initiator unchanged.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_retransmitted_syn_is_idempotent() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // First SYN — seq=1000.
    flow.set_initiator(ip_client, 5000);
    let dir = flow.direction(ip_client, 5000);
    flow.get_direction_mut(dir).set_isn(1000);
    flow.on_syn();

    let isn_after_first = flow.client_to_server.isn;
    let state_after_first = flow.state;

    // Retransmitted SYN — seq=1001 (different seq, but set_isn must not override).
    flow.set_initiator(ip_client, 5000); // idempotent
    let dir2 = flow.direction(ip_client, 5000);
    flow.get_direction_mut(dir2).set_isn(1001); // idempotent — must not change
    flow.on_syn(); // on_syn in SynSent state → no-op

    assert_eq!(
        flow.client_to_server.isn, isn_after_first,
        "BC-2.04.004 inv-2: set_isn must be idempotent; retransmit must not change ISN"
    );
    assert_eq!(
        flow.state, state_after_first,
        "BC-2.04.004 inv-3: on_syn in SynSent is a no-op; state must remain SynSent"
    );
    // Initiator also unchanged (direction still maps correctly).
    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "BC-2.04.004 inv-1: set_initiator is idempotent; initiator unchanged after retransmit"
    );
}

/// AC-005 (BC-2.04.005 postcondition 1)
/// Postcondition: after processing a SYN+ACK, flow.initiator == Some((dst_ip, dst_port))
/// — the DESTINATION of the SYN+ACK is the initiator (the original SYN sender).
/// Canonical test vector: SYN from C, SYN+ACK from S → initiator = C (dst of SYN+ACK).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_syn_ack_sets_initiator_to_dst() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // SYN+ACK from server (src=server, dst=client).
    // apply_handshake_flags calls set_initiator(packet.dst_ip, tcp.dst_port).
    flow.set_initiator(ip_client, 5000); // dst of SYN+ACK is the initiator
    flow.on_syn_ack();

    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "BC-2.04.005 post-1: the DESTINATION of the SYN+ACK (client) must be the initiator"
    );
    assert_eq!(
        flow.direction(ip_server, 80),
        Direction::ServerToClient,
        "BC-2.04.005 post-1: server (src of SYN+ACK) must be ServerToClient"
    );
}

/// AC-006 (BC-2.04.005 postconditions 2-3)
/// Postcondition: after processing SYN+ACK, the server-to-client direction has
/// isn == Some(tcp.seq), base_offset == 1, and flow.state == FlowState::Established.
/// Canonical test vector: SYN from C (seq=1000), SYN+ACK from S (seq=2000) →
///   s2c.isn=Some(2000), state=Established.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_syn_ack_establishes_flow() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // First process SYN to set initiator.
    flow.set_initiator(ip_client, 5000);
    let c2s_dir = flow.direction(ip_client, 5000);
    flow.get_direction_mut(c2s_dir).set_isn(1000);
    flow.on_syn();

    // Now process SYN+ACK (src=server, dst=client, seq=2000).
    // apply_handshake_flags: set_initiator(dst=client) — already set, idempotent.
    // Then: direction(src=server) → ServerToClient; set_isn(2000) on s2c; on_syn_ack().
    let s2c_dir = flow.direction(ip_server, 80);
    flow.get_direction_mut(s2c_dir).set_isn(2000);
    flow.on_syn_ack();

    assert_eq!(
        flow.server_to_client.isn,
        Some(2000),
        "BC-2.04.005 post-2: s2c ISN must be set to SYN+ACK seq=2000"
    );
    assert_eq!(
        flow.server_to_client.base_offset, 1,
        "BC-2.04.005 post-2: s2c base_offset must be 1 (ISN+1)"
    );
    assert_eq!(
        flow.state,
        FlowState::Established,
        "BC-2.04.005 post-3: on_syn_ack() from SynSent must transition to Established"
    );
}

/// AC-007 (BC-2.04.005 invariant 3)
/// Invariant: on_syn_ack() transitions from New directly to Established (server-first
/// capture: SYN+ACK without prior SYN).
/// Canonical test vector: SYN+ACK on New flow → state=Established; initiator=dst.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_syn_ack_without_prior_syn() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    assert_eq!(
        flow.state,
        FlowState::New,
        "precondition: no prior SYN seen"
    );

    // SYN+ACK without prior SYN: set_initiator(dst=client), on_syn_ack().
    flow.set_initiator(ip_client, 5000); // initiator = dst of SYN+ACK
    flow.on_syn_ack();

    assert_eq!(
        flow.state,
        FlowState::Established,
        "BC-2.04.005 inv-3: on_syn_ack() from New must transition directly to Established"
    );
    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "BC-2.04.005 inv-3: initiator must be set to the SYN+ACK destination"
    );
}

/// AC-008 (BC-2.04.050 postcondition — all 9 transitions)
/// Each of the 9 rows in the BC-2.04.050 state transition table is verified
/// individually within this single test function.
///
/// Rows:
///   1. on_syn()                New         → SynSent
///   2. on_syn()                SynSent     → SynSent (no-op guard; state unchanged)
///   3. on_syn_ack()            SynSent     → Established
///   4. on_syn_ack()            New         → Established (server-first)
///   5. on_data_without_syn()   New         → Established + partial=true
///   6. on_fin() (first)        Established → Closing
///   7. on_fin() (first)        SynSent     → Closing
///   8. on_fin() (second, fin_count >= 2) any → Closed
///   9. on_rst()                any         → Closed
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_state_machine_all_transitions() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let key = FlowKey::new(ip_a, 1000, ip_b, 80);

    // ---- Row 1: on_syn() New → SynSent ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        assert_eq!(flow.state, FlowState::New);
        flow.on_syn();
        assert_eq!(
            flow.state,
            FlowState::SynSent,
            "BC-2.04.050 row-1: on_syn() from New must → SynSent"
        );
    }

    // ---- Row 2: on_syn() SynSent → SynSent (no-op guard) ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn(); // New → SynSent
        assert_eq!(flow.state, FlowState::SynSent);
        flow.on_syn(); // must be no-op
        assert_eq!(
            flow.state,
            FlowState::SynSent,
            "BC-2.04.050 row-2: on_syn() from SynSent must stay in SynSent (no-op guard)"
        );
    }

    // ---- Row 3: on_syn_ack() SynSent → Established ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn(); // New → SynSent
        assert_eq!(flow.state, FlowState::SynSent);
        flow.on_syn_ack();
        assert_eq!(
            flow.state,
            FlowState::Established,
            "BC-2.04.050 row-3: on_syn_ack() from SynSent must → Established"
        );
    }

    // ---- Row 4: on_syn_ack() New → Established (server-first) ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        assert_eq!(flow.state, FlowState::New);
        flow.on_syn_ack();
        assert_eq!(
            flow.state,
            FlowState::Established,
            "BC-2.04.050 row-4: on_syn_ack() from New must → Established (server-first)"
        );
    }

    // ---- Row 5: on_data_without_syn() New → Established + partial=true ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        assert_eq!(flow.state, FlowState::New);
        assert!(!flow.partial);
        flow.on_data_without_syn();
        assert_eq!(
            flow.state,
            FlowState::Established,
            "BC-2.04.050 row-5: on_data_without_syn() from New must → Established"
        );
        assert!(
            flow.partial,
            "BC-2.04.050 row-5: on_data_without_syn() must set partial=true"
        );
    }

    // ---- Row 6: on_fin() (first) Established → Closing ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack(); // New → Established
        assert_eq!(flow.state, FlowState::Established);
        flow.on_fin();
        assert_eq!(
            flow.state,
            FlowState::Closing,
            "BC-2.04.050 row-6: first on_fin() from Established must → Closing"
        );
        // F-6: assert fin_count == 1 so the "fin_count becomes 1" row text is
        // genuinely verified, not just narrated. A buggy implementation that
        // never increments fin_count would still reach Closing if it used an
        // independent boolean flag, but would fail this assertion.
        assert_eq!(
            flow.fin_count(),
            1,
            "BC-2.04.050 row-6: fin_count() must be 1 after the first on_fin() from Established"
        );
    }

    // ---- Row 7: on_fin() (first) SynSent → Closing ----
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn(); // New → SynSent
        assert_eq!(flow.state, FlowState::SynSent);
        flow.on_fin();
        assert_eq!(
            flow.state,
            FlowState::Closing,
            "BC-2.04.050 row-7: first on_fin() from SynSent must → Closing"
        );
        // F-6: assert fin_count == 1 — same rationale as row-6 above.
        assert_eq!(
            flow.fin_count(),
            1,
            "BC-2.04.050 row-7: fin_count() must be 1 after the first on_fin() from SynSent"
        );
    }

    // ---- Row 8: on_fin() (second, fin_count >= 2) any → Closed ----
    // BC-2.04.050 claims "any state" for the second FIN. We exercise the
    // path from at least two distinct prior states so the "any" claim is
    // not just asserted but verified. fin_count() is now observable, so
    // we confirm fin_count == 2 in each sub-case (not merely narrated).

    // Row 8a: second FIN from Closing (canonical path — Established → Closing → Closed).
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack(); // New → Established
        flow.on_fin(); // Established → Closing; fin_count=1
        assert_eq!(flow.state, FlowState::Closing);
        flow.on_fin(); // fin_count=2 → Closed
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.050 row-8a: second on_fin() from Closing (fin_count >= 2) must → Closed"
        );
        assert_eq!(
            flow.fin_count(),
            2,
            "BC-2.04.050 row-8a: fin_count() must be exactly 2 after two on_fin() calls"
        );
    }

    // Row 8b: second FIN from SynSent (FIN arrives before full handshake — "any" coverage).
    // First FIN on SynSent → Closing (row-7 path); second FIN → Closed.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn(); // New → SynSent
        assert_eq!(flow.state, FlowState::SynSent);
        flow.on_fin(); // SynSent → Closing; fin_count=1
        assert_eq!(flow.state, FlowState::Closing);
        flow.on_fin(); // fin_count=2 → Closed
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.050 row-8b: second on_fin() from Closing (prior SynSent path) \
             must → Closed — exercises 'any state' claim with SynSent as origin"
        );
        assert_eq!(
            flow.fin_count(),
            2,
            "BC-2.04.050 row-8b: fin_count() must be exactly 2 after two on_fin() calls"
        );
    }

    // Row 8c: second FIN on a New flow — discriminates a buggy "state == Closing" guard.
    //
    // F-5: rows 8a and 8b both pass through Closing before the second FIN. A
    // regressed guard like `if fin_count >= 2 && state == FlowState::Closing { Closed }`
    // would pass both. This sub-case never reaches Closing: the first FIN leaves the
    // flow in New (EC-009 — state stays New because the Closing guard covers only
    // Established and SynSent), so the second FIN must fire the `fin_count >= 2` branch
    // from New directly to Closed. A buggy state == Closing guard would fail here.
    //
    // References: BC-2.04.050 row-8 ("any → Closed"), EC-009.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        assert_eq!(
            flow.state,
            FlowState::New,
            "row-8c precondition: flow starts in New"
        );
        // First FIN: fin_count becomes 1; state stays New (EC-009 — no Closing guard for New).
        flow.on_fin();
        assert_eq!(
            flow.state,
            FlowState::New,
            "BC-2.04.050 row-8c precondition: first on_fin() from New must leave state=New \
             (EC-009: Closing guard only covers Established and SynSent)"
        );
        assert_eq!(
            flow.fin_count(),
            1,
            "BC-2.04.050 row-8c precondition: fin_count() must be 1 after first on_fin() from New"
        );
        // Second FIN: fin_count becomes 2 → `if fin_count >= 2` must fire → Closed.
        // A buggy `if fin_count >= 2 && state == FlowState::Closing` would NOT fire here
        // (state is New, not Closing) and the flow would stay in New, failing this assertion.
        flow.on_fin();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.050 row-8c: second on_fin() from New (fin_count >= 2) must → Closed \
             even though state was never Closing — discriminates a buggy 'state==Closing' guard"
        );
        assert_eq!(
            flow.fin_count(),
            2,
            "BC-2.04.050 row-8c: fin_count() must be exactly 2 after two on_fin() calls on New flow"
        );
    }

    // ---- Row 9: on_rst() any → Closed ----
    {
        // Test from Established (representative of "any").
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack(); // New → Established
        assert_eq!(flow.state, FlowState::Established);
        flow.on_rst();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.050 row-9: on_rst() from any state must → Closed"
        );
    }
}

/// AC-009 (BC-2.04.050 invariant 1)
/// Invariant: on_syn() is a no-op when flow is already in SynSent, Established,
/// Closing, or Closed state. All four non-New states verified.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_on_syn_no_op_when_not_new() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let key = FlowKey::new(ip_a, 1000, ip_b, 80);

    // SynSent: on_syn() must not advance state.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn();
        assert_eq!(flow.state, FlowState::SynSent);
        flow.on_syn();
        assert_eq!(
            flow.state,
            FlowState::SynSent,
            "BC-2.04.050 inv-1: on_syn() in SynSent must be a no-op"
        );
    }

    // Established: on_syn() must not change state.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack(); // New → Established
        assert_eq!(flow.state, FlowState::Established);
        flow.on_syn();
        assert_eq!(
            flow.state,
            FlowState::Established,
            "BC-2.04.050 inv-1: on_syn() in Established must be a no-op"
        );
    }

    // Closing: on_syn() must not change state.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack(); // New → Established
        flow.on_fin(); // Established → Closing
        assert_eq!(flow.state, FlowState::Closing);
        flow.on_syn();
        assert_eq!(
            flow.state,
            FlowState::Closing,
            "BC-2.04.050 inv-1: on_syn() in Closing must be a no-op"
        );
    }

    // Closed: on_syn() must not change state.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_rst(); // New → Closed
        assert_eq!(flow.state, FlowState::Closed);
        flow.on_syn();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.050 inv-1: on_syn() in Closed must be a no-op"
        );
    }
}

/// AC-010 (BC-2.04.050 invariant 4)
/// Invariant: fin_count uses saturating_add(1) to prevent u8 overflow at 255.
/// After 256 on_fin() calls, fin_count() must return 255 — not 0 (wrapping_add
/// regression) and not a panic (plain `+` overflow-checks regression).
///
/// Discrimination:
/// - saturating_add: fin_count() == 255 after 256 calls  ← correct
/// - wrapping_add:   fin_count() == 0  after 256 calls   ← would fail assertion
/// - plain `+`:      panic under dev-profile overflow-checks ← would fail test
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_fin_count_saturates_at_255() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    // Use a flow where state transitions to Established first, then drive fin_count high.
    // After fin_count >= 2, state=Closed; on_fin() continues to increment (saturating).
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);
    flow.on_syn_ack(); // New → Established

    // Call on_fin() 256 times — fin_count must saturate at u8::MAX (255), not wrap.
    for _ in 0..256u32 {
        flow.on_fin();
    }

    // BC-2.04.050 inv-4: fin_count() must be exactly 255 (saturated at u8::MAX).
    // A wrapping_add regression would yield 0; a plain-`+` regression would panic.
    assert_eq!(
        flow.fin_count(),
        255,
        "BC-2.04.050 inv-4: fin_count() must be 255 after 256 on_fin() calls \
         (saturating_add at u8::MAX — wrapping_add would give 0)"
    );
    // State must also be Closed (fin_count >= 2 triggered early).
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "BC-2.04.050 inv-4: state must be Closed after many on_fin() calls"
    );
}

/// AC-011 (BC-2.04.051 invariant 1)
/// Invariant: on_rst() unconditionally transitions to Closed from any prior state —
/// New, SynSent, Established, Closing, Closed — without any state guard.
/// All five states verified per BC-2.04.051.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_rst_closes_from_any_state() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let key = FlowKey::new(ip_a, 1000, ip_b, 80);

    // From New.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        assert_eq!(flow.state, FlowState::New);
        flow.on_rst();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.051 inv-1: on_rst() from New must → Closed"
        );
    }

    // From SynSent.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn();
        assert_eq!(flow.state, FlowState::SynSent);
        flow.on_rst();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.051 inv-1: on_rst() from SynSent must → Closed"
        );
    }

    // From Established.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack();
        assert_eq!(flow.state, FlowState::Established);
        flow.on_rst();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.051 inv-1: on_rst() from Established must → Closed"
        );
    }

    // From Closing.
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_syn_ack();
        flow.on_fin(); // Established → Closing
        assert_eq!(flow.state, FlowState::Closing);
        flow.on_rst();
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.051 inv-1: on_rst() from Closing must → Closed"
        );
    }

    // From Closed: RST on already-Closed flow stays Closed (no-op in practice).
    {
        let mut flow = TcpFlow::new(key.clone(), 0);
        flow.on_rst(); // New → Closed
        assert_eq!(flow.state, FlowState::Closed);
        flow.on_rst(); // must not panic; stays Closed
        assert_eq!(
            flow.state,
            FlowState::Closed,
            "BC-2.04.051 inv-1: on_rst() from Closed must stay Closed"
        );
    }
}

/// AC-012 (BC-2.04.052 postconditions 1-2)
/// Postcondition: on_data_without_syn() on a New flow transitions state to
/// Established and sets partial = true.
/// Canonical test vector: New flow, data packet (no SYN) → state=Established, partial=true.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_052_data_without_syn_sets_partial() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    assert_eq!(flow.state, FlowState::New, "precondition: flow must be New");
    assert!(
        !flow.partial,
        "precondition: partial must be false initially"
    );

    flow.on_data_without_syn();

    assert_eq!(
        flow.state,
        FlowState::Established,
        "BC-2.04.052 post-1: on_data_without_syn() must → Established"
    );
    assert!(
        flow.partial,
        "BC-2.04.052 post-2: on_data_without_syn() must set partial = true"
    );
}

/// AC-013 (BC-2.04.052 invariant 1)
/// Invariant: on_data_without_syn() is a no-op when flow is already Established
/// (the guard: `if self.state == FlowState::New` prevents re-transition).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_052_on_data_without_syn_no_op_when_established() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    // Reach Established via normal handshake.
    flow.on_syn();
    flow.on_syn_ack();
    assert_eq!(flow.state, FlowState::Established);
    assert!(!flow.partial, "normal handshake: partial must be false");

    // Call on_data_without_syn() on an already-Established flow.
    flow.on_data_without_syn();

    assert_eq!(
        flow.state,
        FlowState::Established,
        "BC-2.04.052 inv-1: on_data_without_syn() in Established must be a no-op"
    );
    assert!(
        !flow.partial,
        "BC-2.04.052 inv-1: partial must remain false when on_data_without_syn() is a no-op"
    );
}

/// AC-014 (BC-2.04.053 postcondition 1)
/// Postcondition: direction(src_ip, src_port) returns ClientToServer when
/// src_ip:src_port matches the stored initiator.
/// Canonical test vector: initiator=1.2.3.4:1000, direction(1.2.3.4, 1000) → ClientToServer.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_direction_client_to_server_when_src_is_initiator() {
    let ip_initiator = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    let ip_server = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
    let mut flow = TcpFlow::new(FlowKey::new(ip_initiator, 1000, ip_server, 80), 0);

    flow.set_initiator(ip_initiator, 1000);

    assert_eq!(
        flow.direction(ip_initiator, 1000),
        Direction::ClientToServer,
        "BC-2.04.053 post-1: direction must return ClientToServer when src matches initiator"
    );
}

/// AC-015 (BC-2.04.053 postcondition 2)
/// Postcondition: direction(src_ip, src_port) returns ServerToClient when
/// src_ip:src_port does NOT match the stored initiator.
/// Canonical test vector: initiator=1.2.3.4:1000, direction(5.6.7.8, 80) → ServerToClient.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_direction_server_to_client_when_src_is_not_initiator() {
    let ip_initiator = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    let ip_server = IpAddr::V4(Ipv4Addr::new(5, 6, 7, 8));
    let mut flow = TcpFlow::new(FlowKey::new(ip_initiator, 1000, ip_server, 80), 0);

    flow.set_initiator(ip_initiator, 1000);

    assert_eq!(
        flow.direction(ip_server, 80),
        Direction::ServerToClient,
        "BC-2.04.053 post-2: direction must return ServerToClient when src does not match initiator"
    );
}

/// AC-016 (BC-2.04.053 invariant 2)
/// Invariant: when initiator is None, direction() returns ServerToClient as a
/// conservative default regardless of the src argument.
/// Canonical test vector: initiator=None, direction(any) → ServerToClient.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_direction_server_to_client_when_no_initiator() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    // Do NOT call set_initiator — initiator remains None.
    let flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    assert_eq!(
        flow.direction(ip_a, 1000),
        Direction::ServerToClient,
        "BC-2.04.053 inv-2: direction with initiator=None must return ServerToClient (conservative)"
    );
    assert_eq!(
        flow.direction(ip_b, 80),
        Direction::ServerToClient,
        "BC-2.04.053 inv-2: direction with initiator=None must always return ServerToClient"
    );
}

// ---- EC-001..EC-010: edge-case stubs ----

/// EC-001 (BC-2.04.004 edge case — retransmitted SYN)
/// set_initiator and set_isn are no-ops; state stays SynSent after a second SYN
/// (on_syn() guards on FlowState::New — already SynSent is a no-op).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_ec001_retransmitted_syn_state_unchanged() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // First SYN.
    flow.set_initiator(ip_client, 5000);
    let dir = flow.direction(ip_client, 5000);
    flow.get_direction_mut(dir).set_isn(1000);
    flow.on_syn();
    assert_eq!(flow.state, FlowState::SynSent);
    let isn_first = flow.client_to_server.isn;

    // Retransmitted SYN with a different seq — all setters must be no-ops.
    flow.set_initiator(ip_server, 80); // attempt to override initiator — must fail
    let dir2 = flow.direction(ip_client, 5000); // initiator unchanged → still ClientToServer
    flow.get_direction_mut(dir2).set_isn(9999); // attempt to override ISN — must fail
    flow.on_syn(); // SynSent → no-op

    assert_eq!(
        flow.state,
        FlowState::SynSent,
        "EC-001: state must remain SynSent after retransmitted SYN"
    );
    assert_eq!(
        flow.client_to_server.isn, isn_first,
        "EC-001: ISN must be unchanged after retransmitted SYN (set_isn idempotent)"
    );
    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "EC-001: initiator must be unchanged after retransmitted SYN (set_initiator idempotent)"
    );
}

/// EC-002 (BC-2.04.005 edge case — SYN+ACK without prior SYN)
/// SYN+ACK is the first packet: initiator = dst_ip:dst_port (the inferred SYN sender);
/// state transitions directly from New to Established.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_ec002_syn_ack_first_sets_initiator_to_dst() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    assert_eq!(flow.state, FlowState::New);

    // SYN+ACK: src=server, dst=client; set_initiator(dst=client).
    flow.set_initiator(ip_client, 5000);
    flow.on_syn_ack();

    assert_eq!(
        flow.state,
        FlowState::Established,
        "EC-002: SYN+ACK first packet must transition New → Established"
    );
    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "EC-002: initiator must be set to the SYN+ACK destination (inferred SYN sender)"
    );
}

/// EC-003 (BC-2.04.005 edge case — SYN+ACK retransmission)
/// All setters are idempotent; if already Established, on_syn_ack() is a no-op
/// (Established is not in {SynSent, New} so the guard prevents re-transition).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_005_ec003_syn_ack_retransmission_is_idempotent() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // First SYN → SYN+ACK.
    flow.set_initiator(ip_client, 5000);
    let c2s_dir = flow.direction(ip_client, 5000);
    flow.get_direction_mut(c2s_dir).set_isn(1000);
    flow.on_syn();
    let s2c_dir = flow.direction(ip_server, 80);
    flow.get_direction_mut(s2c_dir).set_isn(2000);
    flow.on_syn_ack();
    assert_eq!(flow.state, FlowState::Established);
    let s2c_isn_first = flow.server_to_client.isn;

    // Retransmitted SYN+ACK — set_initiator and set_isn must both be no-ops.
    flow.set_initiator(ip_server, 80); // attempt to override — must fail
    let s2c_dir2 = flow.direction(ip_server, 80);
    flow.get_direction_mut(s2c_dir2).set_isn(9999); // must fail — idempotent
    flow.on_syn_ack(); // Established → not in guard → no-op

    assert_eq!(
        flow.state,
        FlowState::Established,
        "EC-003: on_syn_ack() retransmission from Established must be a no-op"
    );
    assert_eq!(
        flow.server_to_client.isn, s2c_isn_first,
        "EC-003: set_isn idempotent — s2c ISN must be unchanged after retransmit"
    );
    assert_eq!(
        flow.direction(ip_client, 5000),
        Direction::ClientToServer,
        "EC-003: set_initiator idempotent — initiator unchanged after retransmit"
    );
}

/// EC-004 (BC-2.04.004 edge case — SYN with payload)
/// At the TcpFlow level: set_initiator, set_isn, and on_syn all succeed correctly
/// regardless of whether the packet carries a payload. The ISN is set from seq.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_004_ec004_syn_with_payload_sets_isn() {
    let ip_client = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    let ip_server = IpAddr::V4(Ipv4Addr::new(2, 2, 2, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_client, 5000, ip_server, 80), 0);

    // SYN with seq=500 (unusual but valid TCP — RFC allows SYN with data).
    flow.set_initiator(ip_client, 5000);
    let dir = flow.direction(ip_client, 5000);
    flow.get_direction_mut(dir).set_isn(500);
    flow.on_syn();

    assert_eq!(
        flow.client_to_server.isn,
        Some(500),
        "EC-004: ISN must be set from SYN seq even when SYN carries a payload"
    );
    assert_eq!(
        flow.state,
        FlowState::SynSent,
        "EC-004: state must be SynSent after SYN-with-payload"
    );
    assert_eq!(
        flow.client_to_server.base_offset, 1,
        "EC-004: base_offset must be 1 (ISN+1 is first data byte)"
    );
}

/// EC-005 (BC-2.04.051 edge case — RST on New flow)
/// on_rst() from New (no SYN ever seen) must unconditionally set state = Closed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec005_rst_on_new_flow() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    assert_eq!(
        flow.state,
        FlowState::New,
        "precondition: flow starts in New"
    );
    flow.on_rst();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "EC-005: on_rst() from New must unconditionally → Closed"
    );
}

/// EC-006 (BC-2.04.051 edge case — RST on Closing flow)
/// on_rst() from Closing must unconditionally set state = Closed (no guard).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec006_rst_on_closing_flow() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    flow.on_syn_ack(); // New → Established
    flow.on_fin(); // Established → Closing
    assert_eq!(
        flow.state,
        FlowState::Closing,
        "precondition: flow in Closing"
    );

    flow.on_rst();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "EC-006: on_rst() from Closing must unconditionally → Closed"
    );
}

/// EC-007 (BC-2.04.051 invariant 2 — RST with payload)
/// At the TcpFlow level: on_rst() sets state = Closed regardless. The payload
/// suppression (PostHandshake::FlowClosed returned before payload processing)
/// is an engine-level behavior; here the TcpFlow state transition is confirmed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_051_ec007_rst_closes_flow_state() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    flow.on_syn_ack(); // reach Established
    assert_eq!(flow.state, FlowState::Established);

    // Simulate RST packet arriving with payload (payload processing is engine-level;
    // on_rst() itself is the TcpFlow-level primitive).
    flow.on_rst();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "EC-007: on_rst() must set state=Closed even when packet carries a payload"
    );
}

/// EC-008 (BC-2.04.050 edge case — both FINs from same direction / retransmit)
/// fin_count reaches 2 via two on_fin() calls (both from same direction, i.e.,
/// a retransmit); fin_count >= 2 → state transitions to Closed.
///
/// With the fin_count() accessor now available, fin_count == 2 is explicitly
/// asserted (previously it was only narrated in this comment). This makes the
/// "fin_count = 2" claim verifiable, not just descriptive.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_ec008_both_fins_same_direction_closes_flow() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    flow.on_syn_ack(); // New → Established

    // First FIN from client direction: fin_count = 1, state → Closing.
    flow.on_fin();
    assert_eq!(flow.state, FlowState::Closing);
    assert_eq!(
        flow.fin_count(),
        1,
        "EC-008: fin_count() must be 1 after the first on_fin() call"
    );

    // Second FIN from same direction (retransmit): fin_count = 2 → Closed.
    flow.on_fin();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "EC-008: second on_fin() (fin_count >= 2) must → Closed even from same direction"
    );
    assert_eq!(
        flow.fin_count(),
        2,
        "EC-008: fin_count() must be exactly 2 after two on_fin() calls — \
         confirms the 'fin_count = 2' claim is verifiable, not just narrated"
    );
}

/// EC-009 (BC-2.04.050 edge case — FIN on New flow)
/// on_fin() from New state: fin_count = 1 but state remains New because the
/// Closing guard only applies to Established and SynSent. The flow does NOT
/// transition to Closing from New (only fin_count >= 2 would force Closed).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_050_ec009_fin_on_new_flow() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    assert_eq!(flow.state, FlowState::New);
    // First FIN on a New flow: fin_count becomes 1 but state is not in
    // {Established, SynSent} — the Closing guard is not triggered.
    flow.on_fin();

    // The on_fin() implementation: saturating_add(1) → fin_count=1;
    // then: fin_count >= 2 is false; else-if Established || SynSent is false.
    // So state remains New.
    assert_eq!(
        flow.state,
        FlowState::New,
        "EC-009: on_fin() from New must not transition to Closing (guard only covers \
         Established and SynSent)"
    );
}

// ---------------------------------------------------------------------------
// STORY-014: BC-2.04.031, BC-2.04.009 (partial)
//   ISN management — set_isn / infer_isn unit tests.
//   AC-006..AC-009, EC-001 (seq=0 wrap), EC-005 (ISN already set via SYN).
//
// These tests operate directly on FlowDirection and require no engine-level
// setup. They belong here because set_isn / infer_isn are pure-core methods
// on the same types exercised by STORY-011 and STORY-013 tests above.
//
// NOTE: ISN_MISSING_WARNED accessor — AC-013/AC-014 (BC-2.04.048) in
// reassembly_engine_tests.rs need to observe the ISN_MISSING_WARNED
// AtomicBool. That static is currently `static` (not `pub`). Part B /
// the implementer will need to add:
//   pub fn isn_missing_warned_for_testing() -> bool {
//       ISN_MISSING_WARNED.load(Ordering::Relaxed)
//   }
// in src/reassembly/segment.rs and re-export from the crate's pub API.
// ---------------------------------------------------------------------------

/// STORY-014 / BC-2.04.031 AC-006: set_isn stores isn and sets base_offset=1.
/// Postconditions: self.isn = Some(seq), self.base_offset = 1.
/// Canonical test vector: set_isn(1000) → isn=Some(1000), base_offset=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_031_set_isn_stores_isn_and_base_offset() {
    let mut dir = FlowDirection::new();

    // Precondition: fresh direction has no ISN.
    assert_eq!(
        dir.isn, None,
        "precondition: isn must be None before set_isn"
    );
    assert_eq!(
        dir.base_offset, 0,
        "precondition: base_offset must be 0 before set_isn"
    );

    // BC-2.04.031 postcondition 1-2: set_isn(1000) → isn=Some(1000), base_offset=1.
    dir.set_isn(1000);

    assert_eq!(
        dir.isn,
        Some(1000),
        "BC-2.04.031 post-1: set_isn(1000) must set isn = Some(1000)"
    );
    assert_eq!(
        dir.base_offset, 1,
        "BC-2.04.031 post-2: set_isn must set base_offset = 1 (ISN+1 is first data byte)"
    );
}

/// STORY-014 / BC-2.04.031 AC-007: infer_isn stores seq-1 and sets base_offset=1.
/// Postconditions: self.isn = Some(first_seq.wrapping_sub(1)), self.base_offset = 1.
/// Canonical test vector: infer_isn(500) → isn=Some(499), base_offset=1.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_031_infer_isn_stores_seq_minus_one() {
    let mut dir = FlowDirection::new();

    // Precondition: fresh direction has no ISN.
    assert_eq!(
        dir.isn, None,
        "precondition: isn must be None before infer_isn"
    );
    assert_eq!(
        dir.base_offset, 0,
        "precondition: base_offset must be 0 before infer_isn"
    );

    // BC-2.04.031 postcondition 1-2 (infer path): infer_isn(500) → isn=Some(499), base_offset=1.
    // Canonical test vector from BC-2.04.031 §Canonical Test Vectors row 2.
    dir.infer_isn(500);

    assert_eq!(
        dir.isn,
        Some(499),
        "BC-2.04.031 post-1 (infer): infer_isn(500) must set isn = Some(499) = 500.wrapping_sub(1)"
    );
    assert_eq!(
        dir.base_offset, 1,
        "BC-2.04.031 post-2 (infer): infer_isn must set base_offset = 1"
    );
}

/// STORY-014 / BC-2.04.031 AC-008: both set_isn and infer_isn are idempotent.
/// Postcondition 3 (both paths): second call is a no-op; first ISN preserved.
/// Canonical test vectors: set_isn(1000) then set_isn(2000) → isn=Some(1000).
///
/// Covers three cross-setter scenarios:
///   (a) set_isn then set_isn — second call uses a clearly different value (2000 ≠ 100).
///   (b) infer_isn then infer_isn — second call uses a clearly different value (800 ≠ 500),
///       which would produce a different inferred ISN (799 ≠ 499).
///   (c) set_isn then infer_isn — first setter (set_isn) must win.
///
/// Using obviously different second-call values prevents a no-op that masks the
/// idempotency bug by coincidence (e.g., same arg twice would pass even if isn
/// were simply overwritten each time).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_031_isn_setters_are_idempotent() {
    // (a) set_isn then set_isn: second call is a no-op.
    // Canonical test vector: set_isn(100) then set_isn(200) → isn=Some(100).
    // Discriminant: second value is 200 (≠ 100), so any overwrite would yield Some(200).
    {
        let mut dir = FlowDirection::new();
        dir.set_isn(100);
        assert_eq!(
            dir.isn,
            Some(100),
            "(a) first set_isn(100) must store Some(100)"
        );
        dir.set_isn(200); // must be a no-op
        assert_eq!(
            dir.isn,
            Some(100),
            "BC-2.04.031 post-3 (a): second set_isn(200) must be a no-op; \
             first ISN (100) must be preserved — overwrite regression would give Some(200)"
        );
        assert_eq!(
            dir.base_offset, 1,
            "(a) base_offset must remain 1 after idempotent set_isn"
        );
    }

    // (b) infer_isn then infer_isn: second call is a no-op.
    // infer_isn(500) stores 499; infer_isn(800) would store 799 if not idempotent.
    {
        let mut dir = FlowDirection::new();
        dir.infer_isn(500);
        assert_eq!(
            dir.isn,
            Some(499),
            "(b) first infer_isn(500) must store Some(499)"
        );
        dir.infer_isn(800); // must be a no-op; 800.wrapping_sub(1)=799 would overwrite
        assert_eq!(
            dir.isn,
            Some(499),
            "BC-2.04.031 post-3 (b): second infer_isn(800) must be a no-op; \
             first ISN (499) must be preserved — overwrite regression would give Some(799)"
        );
        assert_eq!(
            dir.base_offset, 1,
            "(b) base_offset must remain 1 after idempotent infer_isn"
        );
    }

    // (c) set_isn then infer_isn (cross-setter): set_isn wins; infer_isn is a no-op.
    // set_isn(300) stores 300; infer_isn(700) would store 699 if the isn.is_none() guard
    // were absent or incorrect.
    {
        let mut dir = FlowDirection::new();
        dir.set_isn(300);
        assert_eq!(dir.isn, Some(300), "(c) set_isn(300) must store Some(300)");
        dir.infer_isn(700); // must be a no-op; 700.wrapping_sub(1)=699 would overwrite
        assert_eq!(
            dir.isn,
            Some(300),
            "BC-2.04.031 post-3 (c): infer_isn after set_isn must be a no-op; \
             first setter (set_isn=300) must win — cross-setter overwrite regression \
             would give Some(699)"
        );
        assert_eq!(
            dir.base_offset, 1,
            "(c) base_offset must remain 1 after cross-setter no-op"
        );
    }
}

/// STORY-014 / BC-2.04.031 AC-009 / EC-001 (seq=0 wrap):
/// infer_isn(0) wraps correctly: ISN = 0u32.wrapping_sub(1) = u32::MAX, base_offset=1.
/// Invariant 2: wrapping_sub handles first_seq==0 without integer underflow.
/// Canonical test vector: infer_isn(0) → isn=Some(4294967295), base_offset=1.
///
/// Discrimination: saturating_sub(1) on 0 yields 0, not u32::MAX.
/// Plain `- 1` on 0 panics in debug/overflow-checked release builds.
/// Only wrapping_sub(1) correctly yields u32::MAX (4294967295).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_031_infer_isn_zero_wraps_to_max() {
    let mut dir = FlowDirection::new();

    // infer_isn(0): 0u32.wrapping_sub(1) must equal u32::MAX (4294967295).
    dir.infer_isn(0);

    assert_eq!(
        dir.isn,
        Some(u32::MAX),
        "BC-2.04.031 inv-2 / EC-001: infer_isn(0) must yield isn=Some(u32::MAX) via \
         wrapping_sub — saturating_sub would give Some(0); plain sub would panic under \
         overflow-checks (release profile has overflow-checks=true per CLAUDE.md)"
    );
    assert_eq!(
        dir.base_offset, 1,
        "BC-2.04.031 post-2: base_offset must be 1 even when seq=0 wraps"
    );
}

/// STORY-014 / BC-2.04.031 EC-005: ISN already set via set_isn; infer_isn is then
/// a no-op (isn.is_some() guard). State unchanged after second-call attempt.
///
/// Scenario: SYN arrives first (sets ISN via set_isn), then engine calls infer_isn
/// on the same direction (e.g., after a mid-stream path executes for a second packet).
/// Both isn and base_offset must remain from the original set_isn call.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_031_ec005_infer_isn_no_op_when_isn_already_set() {
    let mut dir = FlowDirection::new();

    // SYN path: set_isn(1000) — ISN locked in from the SYN sequence number.
    dir.set_isn(1000);
    assert_eq!(
        dir.isn,
        Some(1000),
        "precondition: set_isn(1000) must store Some(1000)"
    );
    assert_eq!(
        dir.base_offset, 1,
        "precondition: base_offset must be 1 after set_isn"
    );

    // Mid-stream infer_isn called on the same direction — must be a no-op.
    // infer_isn(2000) would store 1999 if the guard fails; 1999 ≠ 1000 is the discriminant.
    dir.infer_isn(2000);

    assert_eq!(
        dir.isn,
        Some(1000),
        "BC-2.04.031 EC-005: infer_isn after set_isn must be a no-op; \
         SYN-derived ISN (1000) must be preserved — regression would give Some(1999)"
    );
    assert_eq!(
        dir.base_offset, 1,
        "BC-2.04.031 EC-005: base_offset must remain 1 after no-op infer_isn"
    );
}

/// EC-010 (BC-2.04.053 invariant 2 — initiator = None)
/// When initiator is None (no SYN/SYN+ACK/data-without-SYN ever seen), direction()
/// returns ServerToClient as the conservative default for any src argument.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_053_ec010_direction_none_initiator_returns_server_to_client() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    // initiator remains None — no set_initiator call.
    let flow = TcpFlow::new(FlowKey::new(ip_a, 9999, ip_b, 80), 0);

    // Any endpoint queried with initiator=None must return ServerToClient.
    assert_eq!(
        flow.direction(ip_a, 9999),
        Direction::ServerToClient,
        "EC-010: initiator=None → direction() must return ServerToClient (conservative default)"
    );
    assert_eq!(
        flow.direction(ip_b, 80),
        Direction::ServerToClient,
        "EC-010: initiator=None → direction() always returns ServerToClient for any src"
    );
}

// ---------------------------------------------------------------------------
// STORY-019: flow-level state-transition stubs for on_fin / on_rst
//
// These complement the engine-level tests in reassembly_engine_tests.rs.
// They exercise TcpFlow::on_fin and TcpFlow::on_rst directly (pure state
// machine, no engine or handler involved).
//
// AC-005 / BC-2.04.011 invariant 1 — first FIN → Closing
// AC-008 / BC-2.04.011 EC-002    — same-direction FIN retransmit → Closed
// EC-004 (STORY-019)             — FIN on New flow state path
// EC-005 (STORY-019)             — FIN + data ordering at flow level
//
// PART A: stub-only bodies — panic!("STORY-019 stub — Red Gate").
// ---------------------------------------------------------------------------

/// AC-005 flow-level (BC-2.04.011 invariant 1)
/// `on_fin()` called once on an Established flow: state becomes Closing,
/// `fin_count` becomes 1. Flow is not yet Closed.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_flow_first_fin_state_becomes_closing() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    // Reach Established via SYN+ACK.
    flow.on_syn_ack();
    assert_eq!(
        flow.state,
        FlowState::Established,
        "precondition: flow must be Established before first FIN"
    );

    // First FIN.
    flow.on_fin();

    // BC-2.04.011 inv-1: state becomes Closing, fin_count becomes 1.
    assert_eq!(
        flow.state,
        FlowState::Closing,
        "BC-2.04.011 inv-1: first on_fin() from Established must → Closing"
    );
    assert_eq!(
        flow.fin_count(),
        1,
        "BC-2.04.011 inv-1: fin_count() must be exactly 1 after first on_fin()"
    );
    // Flow is NOT yet Closed.
    assert_ne!(
        flow.state,
        FlowState::Closed,
        "BC-2.04.011 inv-1: flow must NOT be Closed after first FIN (still in Closing)"
    );
}

/// AC-008 flow-level (BC-2.04.011 EC-002 — same-direction retransmit)
/// `on_fin()` called twice from an Established state (simulating same-direction
/// retransmit at the flow level): after the second call `fin_count >= 2` and
/// `state == FlowState::Closed`.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_flow_same_direction_two_fins_reach_closed() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    // Reach Established.
    flow.on_syn_ack();
    assert_eq!(flow.state, FlowState::Established);

    // First FIN: Established → Closing, fin_count=1.
    flow.on_fin();
    assert_eq!(
        flow.state,
        FlowState::Closing,
        "AC-008 precondition: state must be Closing after first FIN"
    );
    assert_eq!(
        flow.fin_count(),
        1,
        "AC-008 precondition: fin_count must be 1"
    );

    // Second FIN from same direction (retransmit): fin_count=2 → Closed.
    flow.on_fin();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "BC-2.04.011 EC-002: second on_fin() (same direction) must → Closed (fin_count >= 2)"
    );
    assert_eq!(
        flow.fin_count(),
        2,
        "BC-2.04.011 EC-002: fin_count() must be exactly 2 after two on_fin() calls"
    );
}

/// EC-004 flow-level (STORY-019 — FIN on New flow)
/// `on_fin()` called on a New flow (no handshake): for state=New, the first
/// FIN leaves state=New (Closing guard only applies to Established/SynSent).
/// Second `on_fin()` reaches fin_count=2 → state=Closed.
///
/// NOTE: This test mirrors BC-2.04.050 row-8c behavior: on_fin() from New does
/// NOT transition to Closing on the first call (the Closing guard only covers
/// Established and SynSent). The fin_count >= 2 check fires on the second call.
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_011_flow_fin_on_new_state_transitions() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    assert_eq!(
        flow.state,
        FlowState::New,
        "precondition: flow starts in New"
    );

    // First FIN from New: fin_count=1, state stays New (guard only covers Established/SynSent).
    flow.on_fin();
    assert_eq!(
        flow.state,
        FlowState::New,
        "EC-004: first on_fin() from New must leave state=New (Closing guard only for Established/SynSent)"
    );
    assert_eq!(
        flow.fin_count(),
        1,
        "EC-004: fin_count() must be 1 after first on_fin() from New"
    );

    // Second FIN: fin_count=2 → fin_count >= 2 fires → Closed.
    flow.on_fin();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "EC-004: second on_fin() from New (fin_count >= 2) must → Closed"
    );
    assert_eq!(
        flow.fin_count(),
        2,
        "EC-004: fin_count() must be 2 after second on_fin()"
    );
}

/// EC-004 flow-level RST counterpart (STORY-019 / BC-2.04.010 invariant 1)
/// `on_rst()` called on a Closing flow: state becomes Closed unconditionally
/// (on_rst has no state guard, mirrors the established pattern from STORY-013).
#[test]
#[allow(non_snake_case)]
fn test_BC_2_04_010_flow_rst_on_closing_state_becomes_closed() {
    let ip_a = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let ip_b = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2));
    let mut flow = TcpFlow::new(FlowKey::new(ip_a, 1000, ip_b, 80), 0);

    // Reach Closing via Established → first FIN.
    flow.on_syn_ack(); // New → Established
    flow.on_fin(); // Established → Closing
    assert_eq!(
        flow.state,
        FlowState::Closing,
        "precondition: flow must be in Closing before RST"
    );

    // RST from Closing: must unconditionally → Closed (no state guard).
    flow.on_rst();
    assert_eq!(
        flow.state,
        FlowState::Closed,
        "BC-2.04.010 inv-1: on_rst() from Closing must → Closed unconditionally"
    );
}
