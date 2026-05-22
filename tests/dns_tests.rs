//! Behavioral-contract test suite for STORY-066: DNS Traffic Statistics.
//!
//! Covers BC-2.08.001 (port-53 dispatch), BC-2.08.002 (QR-bit counting),
//! BC-2.08.003 (summarize output), and BC-2.08.004 (never-emit contract).
//!
//! Test naming follows the factory convention: test_BC_S_SS_NNN_xxx or the
//! story-spec-prescribed verbatim names (W1.4 decision).

use std::net::IpAddr;

use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_packet(transport: TransportInfo, payload: Vec<u8>) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4("127.0.0.1".parse().unwrap()),
        dst_ip: IpAddr::V4("127.0.0.2".parse().unwrap()),
        protocol: match &transport {
            TransportInfo::Udp { .. } => Protocol::Udp,
            TransportInfo::Tcp { .. } => Protocol::Tcp,
            TransportInfo::None => Protocol::Icmp,
        },
        transport,
        payload,
        packet_len: 64,
    }
}

fn make_tcp(src_port: u16, dst_port: u16) -> TransportInfo {
    TransportInfo::Tcp {
        src_port,
        dst_port,
        seq_number: 0,
        syn: false,
        ack: false,
        fin: false,
        rst: false,
    }
}

/// 12-byte DNS query payload (QR=0: payload[2] & 0x80 == 0).
fn dns_query_payload() -> Vec<u8> {
    let mut p = vec![0u8; 12];
    p[2] = 0x00; // QR=0, standard query
    p
}

/// 12-byte DNS response payload (QR=1: payload[2] & 0x80 != 0).
fn dns_response_payload() -> Vec<u8> {
    let mut p = vec![0u8; 12];
    p[2] = 0x80; // QR=1, standard response
    p
}

fn udp_dst53_query_packet() -> ParsedPacket {
    make_packet(
        TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        dns_query_payload(),
    )
}

fn udp_src53_response_packet() -> ParsedPacket {
    make_packet(
        TransportInfo::Udp {
            src_port: 53,
            dst_port: 12345,
        },
        dns_response_payload(),
    )
}

// ---------------------------------------------------------------------------
// AC-001 (BC-2.08.001 postcondition 1)
// can_decode returns true for src==53 or dst==53, TCP or UDP.
// ---------------------------------------------------------------------------

/// AC-001 / BC-2.08.001 postcondition 1:
/// `DnsAnalyzer::can_decode` returns `true` for any packet where `src_port==53`
/// OR `dst_port==53`, regardless of whether the transport is TCP or UDP.
#[test]
fn test_dns_can_decode_port_53_tcp_and_udp() {
    let a = DnsAnalyzer::new();

    // UDP src=12345, dst=53 (query from client to server)
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        vec![],
    );
    assert!(a.can_decode(&pkt), "UDP dst=53 must decode");

    // UDP src=53, dst=12345 (response from server to client)
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 53,
            dst_port: 12345,
        },
        vec![],
    );
    assert!(a.can_decode(&pkt), "UDP src=53 must decode");

    // TCP dst=53 (DNS-over-TCP query)
    let pkt = make_packet(make_tcp(54321, 53), vec![]);
    assert!(a.can_decode(&pkt), "TCP dst=53 must decode");

    // TCP src=53 (DNS-over-TCP response)
    let pkt = make_packet(make_tcp(53, 54321), vec![]);
    assert!(a.can_decode(&pkt), "TCP src=53 must decode");
}

// ---------------------------------------------------------------------------
// AC-002 (BC-2.08.001 postcondition 2)
// can_decode returns false for TransportInfo::None.
// ---------------------------------------------------------------------------

/// AC-002 / BC-2.08.001 postcondition 2:
/// `can_decode` returns `false` for `TransportInfo::None` (ICMP / unknown transport).
#[test]
fn test_dns_can_decode_false_for_icmp() {
    let a = DnsAnalyzer::new();
    let pkt = make_packet(TransportInfo::None, vec![]);
    assert!(!a.can_decode(&pkt), "TransportInfo::None must not decode");
}

// ---------------------------------------------------------------------------
// AC-003 (BC-2.08.001 postcondition 3)
// can_decode returns false when neither port is 53.
// ---------------------------------------------------------------------------

/// AC-003 / BC-2.08.001 postcondition 3:
/// `can_decode` returns `false` when neither src_port nor dst_port is 53.
/// Canonical test vector: UDP src=54, dst=54.
#[test]
fn test_dns_can_decode_false_for_non_dns_port() {
    let a = DnsAnalyzer::new();

    // UDP src=54, dst=54 (canonical test vector from BC-2.08.001)
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 54,
            dst_port: 54,
        },
        vec![],
    );
    assert!(!a.can_decode(&pkt), "UDP port 54/54 must not decode");

    // UDP src=9999, dst=9999 (canonical test vector from BC-2.08.001)
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 9999,
            dst_port: 9999,
        },
        vec![],
    );
    assert!(!a.can_decode(&pkt), "UDP port 9999/9999 must not decode");

    // UDP dst=443 (BC-2.08.001 edge case EC-006)
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 1234,
            dst_port: 443,
        },
        vec![],
    );
    assert!(!a.can_decode(&pkt), "UDP dst=443 must not decode");
}

// ---------------------------------------------------------------------------
// AC-004 (BC-2.08.002 postcondition 1)
// analyze increments query_count when QR=0 and payload.len() >= 12.
// ---------------------------------------------------------------------------

/// AC-004 / BC-2.08.002 postcondition 1:
/// When `payload.len() >= 12` AND `(payload[2] & 0x80) == 0` (QR=0),
/// `analyze` increments `query_count` by 1; `summarize().detail["dns_queries"]`
/// reflects the increment. `analyze` returns an empty `Vec<Finding>`.
#[test]
fn test_dns_analyzer_counts_queries() {
    let mut a = DnsAnalyzer::new();

    let pkt = udp_dst53_query_packet(); // QR=0, len=12
    let findings = a.analyze(&pkt);
    assert!(
        findings.is_empty(),
        "BC-2.08.004: analyze must return empty findings"
    );

    let summary = a.summarize();
    assert_eq!(
        summary.detail["dns_queries"],
        serde_json::json!(1u64),
        "BC-2.08.002 pc1: dns_queries must be 1 after one query packet"
    );
    assert_eq!(
        summary.detail["dns_responses"],
        serde_json::json!(0u64),
        "BC-2.08.002 pc1: dns_responses must be 0"
    );
}

// ---------------------------------------------------------------------------
// AC-005 (BC-2.08.002 postcondition 2)
// analyze increments response_count when QR=1 and payload.len() >= 12.
// ---------------------------------------------------------------------------

/// AC-005 / BC-2.08.002 postcondition 2:
/// When `payload.len() >= 12` AND `(payload[2] & 0x80) != 0` (QR=1),
/// `analyze` increments `response_count` by 1.
#[test]
fn test_dns_analyzer_counts_responses() {
    let mut a = DnsAnalyzer::new();

    let pkt = udp_src53_response_packet(); // QR=1, len=12
    let findings = a.analyze(&pkt);
    assert!(
        findings.is_empty(),
        "BC-2.08.004: analyze must return empty findings"
    );

    let summary = a.summarize();
    assert_eq!(
        summary.detail["dns_responses"],
        serde_json::json!(1u64),
        "BC-2.08.002 pc2: dns_responses must be 1 after one response packet"
    );
    assert_eq!(
        summary.detail["dns_queries"],
        serde_json::json!(0u64),
        "BC-2.08.002 pc2: dns_queries must be 0"
    );
}

// ---------------------------------------------------------------------------
// AC-006 (BC-2.08.002 postcondition 3)
// Short payload (<12 bytes): is_query returns false, response_count increments.
// ---------------------------------------------------------------------------

/// AC-006 / BC-2.08.002 postcondition 3 / invariant 2:
/// When `payload.len() < 12`, `is_query` returns `false` (length guard fires
/// before the QR-bit test), and `response_count` (not `query_count`) is
/// incremented. Canonical test vector: 6-byte payload.
#[test]
fn test_dns_short_payload_counted_as_response() {
    let mut a = DnsAnalyzer::new();

    // 11-byte payload: one byte below the 12-byte minimum (BC-2.08.002 EC-003)
    let short_payload = vec![0u8; 11];
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        short_payload,
    );
    let findings = a.analyze(&pkt);
    assert!(findings.is_empty());

    let summary = a.summarize();
    assert_eq!(
        summary.detail["dns_responses"],
        serde_json::json!(1u64),
        "BC-2.08.002 pc3: short payload must count as response"
    );
    assert_eq!(
        summary.detail["dns_queries"],
        serde_json::json!(0u64),
        "BC-2.08.002 pc3: query_count must remain 0 for short payload"
    );
}

// ---------------------------------------------------------------------------
// AC-007 (BC-2.08.002 invariant 1)
// Exactly one counter incremented per analyze call.
// ---------------------------------------------------------------------------

/// AC-007 / BC-2.08.002 invariant 1:
/// Exactly one of `query_count` or `response_count` is incremented per call to
/// `analyze`. Their sum always equals the total number of `analyze` calls made.
#[test]
fn test_dns_analyze_increments_exactly_one_counter() {
    let mut a = DnsAnalyzer::new();

    // Mix: 3 queries, 2 responses, 1 short (counted as response)
    for _ in 0..3 {
        a.analyze(&udp_dst53_query_packet());
    }
    for _ in 0..2 {
        a.analyze(&udp_src53_response_packet());
    }
    // Short payload — counted as response
    let short = make_packet(
        TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        vec![0u8; 6],
    );
    a.analyze(&short);

    let summary = a.summarize();
    let dns_q = summary.detail["dns_queries"].as_u64().unwrap();
    let dns_r = summary.detail["dns_responses"].as_u64().unwrap();

    // 6 analyze calls total: 3 queries + 3 responses (2 real + 1 short)
    assert_eq!(
        dns_q + dns_r,
        6,
        "BC-2.08.002 inv1: sum of counters must equal analyze call count"
    );
    assert_eq!(dns_q, 3, "BC-2.08.002 inv1: query_count must be 3");
    assert_eq!(
        dns_r, 3,
        "BC-2.08.002 inv1: response_count must be 3 (2 real + 1 short)"
    );
    assert_eq!(
        summary.packets_analyzed, 6,
        "BC-2.08.003 pc5: packets_analyzed must equal sum"
    );
}

// ---------------------------------------------------------------------------
// AC-008 (BC-2.08.003 postcondition 1)
// summarize().analyzer_name == "DNS"
// ---------------------------------------------------------------------------

/// AC-008 / BC-2.08.003 postcondition 1:
/// `DnsAnalyzer::summarize()` returns an `AnalysisSummary` with
/// `analyzer_name == "DNS"`.
#[test]
fn test_dns_summarize_analyzer_name() {
    let a = DnsAnalyzer::new();
    let summary = a.summarize();
    assert_eq!(
        summary.analyzer_name, "DNS",
        "BC-2.08.003 pc1: analyzer_name must be \"DNS\""
    );
}

// ---------------------------------------------------------------------------
// AC-009 (BC-2.08.003 postcondition 5)
// summarize().packets_analyzed == query_count + response_count
// ---------------------------------------------------------------------------

/// AC-009 / BC-2.08.003 postcondition 5:
/// `summarize().packets_analyzed` equals `query_count + response_count` (total
/// packets seen by `analyze`). Canonical test vector: 2 queries + 1 response.
#[test]
fn test_dns_summarize_packets_analyzed_is_sum() {
    let mut a = DnsAnalyzer::new();

    a.analyze(&udp_dst53_query_packet()); // query
    a.analyze(&udp_dst53_query_packet()); // query
    a.analyze(&udp_src53_response_packet()); // response

    let summary = a.summarize();
    assert_eq!(
        summary.packets_analyzed, 3,
        "BC-2.08.003 pc5: packets_analyzed must be 3 (2 queries + 1 response)"
    );
    let dns_q = summary.detail["dns_queries"].as_u64().unwrap();
    let dns_r = summary.detail["dns_responses"].as_u64().unwrap();
    assert_eq!(
        summary.packets_analyzed,
        dns_q + dns_r,
        "BC-2.08.003 pc5: packets_analyzed must equal dns_queries + dns_responses"
    );
}

// ---------------------------------------------------------------------------
// AC-010 + AC-011 (BC-2.08.003 postconditions 2 and 3)
// detail["dns_queries"] and detail["dns_responses"] are correct JSON numbers.
// ---------------------------------------------------------------------------

/// AC-010 / BC-2.08.003 postcondition 2:
/// `detail["dns_queries"]` is a `serde_json::Value::Number` equal to `query_count`.
/// AC-011 / BC-2.08.003 postcondition 3:
/// `detail["dns_responses"]` is a `serde_json::Value::Number` equal to `response_count`.
/// (Both ACs share this test per W1.4 decision in the story spec.)
#[test]
fn test_dns_summarize_detail_keys() {
    let mut a = DnsAnalyzer::new();

    // 3 queries, 1 response (canonical test vector from BC-2.08.003)
    for _ in 0..3 {
        a.analyze(&udp_dst53_query_packet());
    }
    a.analyze(&udp_src53_response_packet());

    let summary = a.summarize();

    // AC-010: dns_queries value is a JSON number equal to query_count (3)
    let queries = &summary.detail["dns_queries"];
    assert!(
        queries.is_number(),
        "BC-2.08.003 pc2: dns_queries must be a JSON number"
    );
    assert_eq!(
        queries.as_u64(),
        Some(3),
        "BC-2.08.003 pc2: dns_queries must equal 3"
    );

    // AC-011: dns_responses value is a JSON number equal to response_count (1)
    let responses = &summary.detail["dns_responses"];
    assert!(
        responses.is_number(),
        "BC-2.08.003 pc3: dns_responses must be a JSON number"
    );
    assert_eq!(
        responses.as_u64(),
        Some(1),
        "BC-2.08.003 pc3: dns_responses must equal 1"
    );
}

// ---------------------------------------------------------------------------
// AC-012 (BC-2.08.003 postcondition 4)
// detail BTreeMap has exactly two keys.
// ---------------------------------------------------------------------------

/// AC-012 / BC-2.08.003 postcondition 4:
/// The `detail` `BTreeMap` contains exactly two keys: `"dns_queries"` and
/// `"dns_responses"` — no other keys.
#[test]
fn test_dns_summarize_exactly_two_detail_keys() {
    let a = DnsAnalyzer::new();
    let summary = a.summarize();

    assert_eq!(
        summary.detail.len(),
        2,
        "BC-2.08.003 pc4: detail must have exactly 2 keys"
    );

    let keys: Vec<&str> = summary.detail.keys().map(|s| s.as_str()).collect();
    // BTreeMap is sorted lexicographically: "dns_queries" < "dns_responses"
    assert_eq!(
        keys,
        vec!["dns_queries", "dns_responses"],
        "BC-2.08.003 inv1: keys must be in lexicographic order"
    );
}

// ---------------------------------------------------------------------------
// AC-013 (BC-2.08.004 postcondition 1)
// analyze() always returns empty Vec<Finding>.
// ---------------------------------------------------------------------------

/// AC-013 / BC-2.08.004 postcondition 1:
/// `DnsAnalyzer::analyze(packet)` unconditionally returns an empty `Vec<Finding>`.
/// Verified for a query, a response, and a short (malformed) payload.
#[test]
fn test_dns_analyze_always_returns_empty_findings() {
    let mut a = DnsAnalyzer::new();

    let query_findings = a.analyze(&udp_dst53_query_packet());
    assert!(
        query_findings.is_empty(),
        "BC-2.08.004 pc1: query packet must produce no findings"
    );

    let response_findings = a.analyze(&udp_src53_response_packet());
    assert!(
        response_findings.is_empty(),
        "BC-2.08.004 pc1: response packet must produce no findings"
    );

    // Short/malformed payload
    let short = make_packet(
        TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        vec![0u8; 6],
    );
    let short_findings = a.analyze(&short);
    assert!(
        short_findings.is_empty(),
        "BC-2.08.004 pc1: short payload must produce no findings"
    );
}

// ---------------------------------------------------------------------------
// AC-014 (BC-2.08.004 invariant 1)
// High volume — 1000 packets — never produces findings.
// ---------------------------------------------------------------------------

/// AC-014 / BC-2.08.004 invariant 1:
/// The never-emit contract holds unconditionally. 1000 `analyze` calls all
/// return `vec![]`. Also verifies `query_count == 1000` in the summary.
#[test]
fn test_dns_high_volume_no_findings() {
    let mut a = DnsAnalyzer::new();

    for _ in 0..1000 {
        let findings = a.analyze(&udp_dst53_query_packet());
        assert!(
            findings.is_empty(),
            "BC-2.08.004 inv1: analyze must never return findings"
        );
    }

    let summary = a.summarize();
    assert_eq!(
        summary.detail["dns_queries"].as_u64(),
        Some(1000),
        "BC-2.08.004 inv1: 1000 queries must be counted"
    );
    assert_eq!(
        summary.detail["dns_responses"].as_u64(),
        Some(0),
        "BC-2.08.004 inv1: dns_responses must remain 0"
    );
}

// ---------------------------------------------------------------------------
// EC-001 (BC-2.08.001 edge case): UDP src=53, dst=12345 — can_decode true
// ---------------------------------------------------------------------------

/// EC-001 / BC-2.08.001:
/// UDP src=53, dst=12345 (DNS response packet from server to client):
/// `can_decode` must return `true` because src_port == 53.
#[test]
fn test_dns_ec001_udp_src53_can_decode() {
    let a = DnsAnalyzer::new();
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 53,
            dst_port: 12345,
        },
        vec![],
    );
    assert!(a.can_decode(&pkt), "EC-001: UDP src=53 must decode");
}

// ---------------------------------------------------------------------------
// EC-002 (BC-2.08.001 edge case): TCP dst=53 — can_decode true
// ---------------------------------------------------------------------------

/// EC-002 / BC-2.08.001:
/// TCP dst=53 (DNS-over-TCP query): `can_decode` must return `true`.
#[test]
fn test_dns_ec002_tcp_dst53_can_decode() {
    let a = DnsAnalyzer::new();
    let pkt = make_packet(make_tcp(54321, 53), vec![]);
    assert!(a.can_decode(&pkt), "EC-002: TCP dst=53 must decode");
}

// ---------------------------------------------------------------------------
// EC-003 (BC-2.08.002 edge case): payload.len() == 0 — response_count++
// ---------------------------------------------------------------------------

/// EC-003 / BC-2.08.002:
/// `payload.len() == 0`: `is_query` returns `false` (length guard fires);
/// `response_count` is incremented, `query_count` stays at 0.
#[test]
fn test_dns_ec003_empty_payload_counted_as_response() {
    let mut a = DnsAnalyzer::new();

    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        vec![],
    );
    a.analyze(&pkt);

    let summary = a.summarize();
    assert_eq!(
        summary.detail["dns_responses"].as_u64(),
        Some(1),
        "EC-003: zero-length payload must be counted as response"
    );
    assert_eq!(
        summary.detail["dns_queries"].as_u64(),
        Some(0),
        "EC-003: query_count must remain 0 for empty payload"
    );
}

// ---------------------------------------------------------------------------
// EC-004 (BC-2.08.002 edge case): payload[2] == 0xFF — response_count++
// ---------------------------------------------------------------------------

/// EC-004 / BC-2.08.002:
/// `payload[2] == 0xFF` (QR=1 plus all other flag bits set): `response_count`
/// is incremented because the QR bit (bit 7) is set.
#[test]
fn test_dns_ec004_all_flags_set_counted_as_response() {
    let mut a = DnsAnalyzer::new();

    let mut payload = vec![0u8; 12];
    payload[2] = 0xFF; // QR=1 + all other flags set
    let pkt = make_packet(
        TransportInfo::Udp {
            src_port: 53,
            dst_port: 12345,
        },
        payload,
    );
    a.analyze(&pkt);

    let summary = a.summarize();
    assert_eq!(
        summary.detail["dns_responses"].as_u64(),
        Some(1),
        "EC-004: payload[2]==0xFF (QR=1) must count as response"
    );
    assert_eq!(
        summary.detail["dns_queries"].as_u64(),
        Some(0),
        "EC-004: query_count must remain 0"
    );
}

// ---------------------------------------------------------------------------
// EC-005 (BC-2.08.003 edge case): zero analyze() calls — all counts == 0
// ---------------------------------------------------------------------------

/// EC-005 / BC-2.08.003 postcondition 1 + invariant 4:
/// With no calls to `analyze()`, `summarize()` returns `packets_analyzed=0`,
/// `dns_queries=0`, `dns_responses=0`. Calling `summarize` twice returns the
/// same values (invariant 4: summarize does NOT reset counters).
#[test]
fn test_dns_ec005_zero_packets_summarize() {
    let a = DnsAnalyzer::new();

    let s1 = a.summarize();
    assert_eq!(
        s1.packets_analyzed, 0,
        "EC-005: packets_analyzed must be 0 with no analyze calls"
    );
    assert_eq!(
        s1.detail["dns_queries"].as_u64(),
        Some(0),
        "EC-005: dns_queries must be 0"
    );
    assert_eq!(
        s1.detail["dns_responses"].as_u64(),
        Some(0),
        "EC-005: dns_responses must be 0"
    );

    // Call summarize a second time — counters must not have been reset
    let s2 = a.summarize();
    assert_eq!(
        s2.packets_analyzed, s1.packets_analyzed,
        "EC-005 / BC-2.08.003 inv4: summarize must not reset counters"
    );
    assert_eq!(
        s2.detail["dns_queries"], s1.detail["dns_queries"],
        "EC-005: second summarize must match first"
    );
    assert_eq!(
        s2.detail["dns_responses"], s1.detail["dns_responses"],
        "EC-005: second summarize must match first"
    );
}
