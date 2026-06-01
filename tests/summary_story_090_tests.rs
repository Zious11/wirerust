//! STORY-090: Summary data model formalization tests — Wave 27 FINAL
//!
//! Behavioral contracts covered:
//!   BC-2.12.018  Summary::ingest Increments total_packets, total_bytes, hosts, protocols
//!   BC-2.12.019  Summary::ingest Derives Service Name from app_protocol_hint
//!   BC-2.12.020  Summary::unique_hosts Returns Sorted Deduplicated Vec<IpAddr>
//!   BC-2.12.021  Summary Serializes with total_packets/total_bytes/skipped_packets Fields
//!
//! AC↔test-name sync enforced by DF-AC-TEST-NAME-SYNC-001.
//! Namespace isolation enforced by DF-TEST-NAMESPACE-001 (`mod story_090`).

// PG-W17-001 mandates that test fn names EXACTLY match the AC `**Test:**`
// citations (e.g. `test_BC_2_12_018_xxx`).  These names use upper-case BC
// identifiers which Rust flags as non-snake-case.  Suppress the lint for this
// file rather than diverge from the required naming scheme.
#![allow(non_snake_case)]

mod story_090 {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
    use wirerust::reporter::Reporter;
    use wirerust::reporter::json::JsonReporter;
    use wirerust::summary::Summary;

    // -------------------------------------------------------------------------
    // Shared helpers — mirror the construction patterns from summary_tests.rs
    // -------------------------------------------------------------------------

    /// Build a TCP ParsedPacket with IPv4 src/dst, arbitrary ports, and the
    /// given `packet_len`.  `seq_number` is fixed; flags are all false.
    fn make_tcp_packet(
        src: [u8; 4],
        dst: [u8; 4],
        src_port: u16,
        dst_port: u16,
        packet_len: usize,
    ) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::from(src)),
            dst_ip: IpAddr::V4(Ipv4Addr::from(dst)),
            protocol: Protocol::Tcp,
            transport: TransportInfo::Tcp {
                src_port,
                dst_port,
                seq_number: 1000,
                syn: false,
                ack: false,
                fin: false,
                rst: false,
            },
            payload: vec![],
            packet_len,
        }
    }

    /// Build a UDP ParsedPacket with IPv4 src/dst, arbitrary ports, and the
    /// given `packet_len`.
    fn make_udp_packet(
        src: [u8; 4],
        dst: [u8; 4],
        src_port: u16,
        dst_port: u16,
        packet_len: usize,
    ) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::from(src)),
            dst_ip: IpAddr::V4(Ipv4Addr::from(dst)),
            protocol: Protocol::Udp,
            transport: TransportInfo::Udp { src_port, dst_port },
            payload: vec![],
            packet_len,
        }
    }

    /// Build a TCP ParsedPacket whose src_ip is IPv6.
    fn make_tcp_packet_ipv6(
        src: Ipv6Addr,
        dst: Ipv6Addr,
        src_port: u16,
        dst_port: u16,
        packet_len: usize,
    ) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V6(src),
            dst_ip: IpAddr::V6(dst),
            protocol: Protocol::Tcp,
            transport: TransportInfo::Tcp {
                src_port,
                dst_port,
                seq_number: 1000,
                syn: false,
                ack: false,
                fin: false,
                rst: false,
            },
            payload: vec![],
            packet_len,
        }
    }

    /// Build a ParsedPacket with no transport information (ICMP-like).
    fn make_icmp_packet(src: [u8; 4], dst: [u8; 4], packet_len: usize) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::from(src)),
            dst_ip: IpAddr::V4(Ipv4Addr::from(dst)),
            protocol: Protocol::Icmp,
            transport: TransportInfo::None,
            payload: vec![],
            packet_len,
        }
    }

    /// Render a Summary via JsonReporter and parse the JSON.
    fn render_and_parse(summary: &Summary) -> serde_json::Value {
        let json_str = JsonReporter.render(summary, &[], &[]);
        serde_json::from_str(&json_str).unwrap_or_else(|e| {
            panic!("JSON parse failed: {e}\nOutput was:\n{json_str}");
        })
    }

    // =========================================================================
    // AC-001  BC-2.12.018 pc1: ingest increments total_packets
    // =========================================================================

    #[test]
    fn test_summary_ingest_increments_total_packets() {
        let mut summary = Summary::new();
        assert_eq!(summary.total_packets, 0);

        let pkt1 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        summary.ingest(&pkt1);
        assert_eq!(summary.total_packets, 1);

        let pkt2 = make_tcp_packet([10, 0, 0, 3], [10, 0, 0, 4], 9999, 9999, 100);
        summary.ingest(&pkt2);
        assert_eq!(summary.total_packets, 2);

        let pkt3 = make_udp_packet([10, 0, 0, 5], [10, 0, 0, 6], 12345, 53, 60);
        summary.ingest(&pkt3);
        assert_eq!(summary.total_packets, 3);
    }

    // =========================================================================
    // AC-002  BC-2.12.018 pc2: ingest increments total_bytes by packet_len
    // =========================================================================

    #[test]
    fn test_summary_ingest_increments_total_bytes() {
        let mut summary = Summary::new();
        assert_eq!(summary.total_bytes, 0);

        let pkt1 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 100);
        summary.ingest(&pkt1);
        assert_eq!(summary.total_bytes, 100);

        let pkt2 = make_tcp_packet([10, 0, 0, 3], [10, 0, 0, 4], 9999, 9999, 250);
        summary.ingest(&pkt2);
        assert_eq!(summary.total_bytes, 350);

        let pkt3 = make_udp_packet([10, 0, 0, 5], [10, 0, 0, 6], 12345, 53, 50);
        summary.ingest(&pkt3);
        assert_eq!(summary.total_bytes, 400);
    }

    // =========================================================================
    // AC-003  BC-2.12.018 pc3: ingest inserts src_ip AND dst_ip into hosts;
    //         duplicate appearances are deduplicated
    // =========================================================================

    #[test]
    fn test_BC_2_12_018_host_counting_src_and_dst() {
        let mut summary = Summary::new();

        // Packet 1: introduces 10.0.0.1 and 10.0.0.2
        let pkt1 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        summary.ingest(&pkt1);
        assert_eq!(summary.unique_hosts().len(), 2);

        // Packet 2: introduces 10.0.0.3; 10.0.0.1 is a duplicate → still 3 unique
        let pkt2 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 3], 12346, 443, 60);
        summary.ingest(&pkt2);
        assert_eq!(summary.unique_hosts().len(), 3);

        // Packet 3: both src and dst are already seen → still 3 unique
        let pkt3 = make_tcp_packet([10, 0, 0, 2], [10, 0, 0, 1], 80, 12345, 54);
        summary.ingest(&pkt3);
        assert_eq!(summary.unique_hosts().len(), 3);

        // Verify the exact IPs present
        let hosts = summary.unique_hosts();
        assert!(hosts.contains(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(hosts.contains(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2))));
        assert!(hosts.contains(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3))));
    }

    // =========================================================================
    // AC-004  BC-2.12.018 pc4: ingest increments protocols[packet.protocol]
    // =========================================================================

    #[test]
    fn test_BC_2_12_018_protocol_breakdown() {
        let mut summary = Summary::new();

        let pkt_tcp1 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        let pkt_tcp2 = make_tcp_packet([10, 0, 0, 3], [10, 0, 0, 4], 9999, 443, 60);
        let pkt_udp = make_udp_packet([10, 0, 0, 5], [10, 0, 0, 6], 12345, 53, 40);
        let pkt_icmp = make_icmp_packet([10, 0, 0, 7], [10, 0, 0, 8], 28);

        summary.ingest(&pkt_tcp1);
        summary.ingest(&pkt_tcp2);
        summary.ingest(&pkt_udp);
        summary.ingest(&pkt_icmp);

        let counts = summary.protocol_counts();
        assert_eq!(counts.get(&Protocol::Tcp), Some(&2));
        assert_eq!(counts.get(&Protocol::Udp), Some(&1));
        assert_eq!(counts.get(&Protocol::Icmp), Some(&1));
    }

    // =========================================================================
    // AC-005  BC-2.12.018 inv2: ingest never touches skipped_packets
    // =========================================================================

    #[test]
    fn test_skipped_packets_not_modified_by_ingest() {
        let mut summary = Summary::new();
        assert_eq!(summary.skipped_packets, 0);

        // Ingest several packets
        for i in 1u8..=5 {
            let pkt = make_tcp_packet([10, 0, 0, i], [10, 0, 0, i + 10], 9999, 80, 54);
            summary.ingest(&pkt);
        }

        // skipped_packets must still be 0 — ingest never sets it
        assert_eq!(summary.skipped_packets, 0);
        assert_eq!(summary.total_packets, 5);

        // Caller can set skipped_packets manually (as main.rs does)
        summary.skipped_packets = 3;
        assert_eq!(summary.skipped_packets, 3);
    }

    // =========================================================================
    // AC-006  BC-2.12.019 pc1: when app_protocol_hint returns Some("HTTP"),
    //         services["HTTP"] is incremented
    // =========================================================================

    #[test]
    fn test_summary_service_detection_http() {
        let mut summary = Summary::new();

        // dst_port=80 triggers app_protocol_hint() → Some("HTTP")
        let pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        summary.ingest(&pkt);

        let services = summary.service_counts();
        assert_eq!(
            services.get("HTTP"),
            Some(&1),
            "services[\"HTTP\"] should be 1"
        );
        assert_eq!(services.len(), 1, "only one service entry expected");
    }

    // =========================================================================
    // AC-007  BC-2.12.019 pc2: when app_protocol_hint returns None (unknown
    //         port), the services map remains unchanged
    // =========================================================================

    #[test]
    fn test_summary_service_detection_none_on_unknown_port() {
        let mut summary = Summary::new();

        // port 9999 is not a known service port → app_protocol_hint() returns None
        let pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 9998, 9999, 54);
        summary.ingest(&pkt);

        let services = summary.service_counts();
        assert!(
            services.is_empty(),
            "services map should be empty for unknown port; got: {services:?}"
        );
    }

    // =========================================================================
    // AC-008  BC-2.12.019 inv2: service detection is port-based (LESSON-P3.01),
    //         not content-based — HTTP on a non-80/443 port is not counted
    // =========================================================================

    #[test]
    fn test_summary_service_is_port_based_not_content_based() {
        let mut summary = Summary::new();

        // HTTP payload bytes on port 8080 — content looks like HTTP but port is not known
        let mut pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 8080, 200);
        pkt.payload = b"GET / HTTP/1.1\r\nHost: example.com\r\n\r\n".to_vec();
        summary.ingest(&pkt);

        // Despite HTTP content, services map must be empty (port 8080 is unknown)
        let services = summary.service_counts();
        assert!(
            services.is_empty(),
            "port-based service detection must not count HTTP content on port 8080; \
             got: {services:?}"
        );
        assert_eq!(summary.total_packets, 1);
    }

    // =========================================================================
    // AC-009  BC-2.12.020 pc1: unique_hosts returns a sorted, deduplicated Vec
    // =========================================================================

    #[test]
    fn test_unique_hosts_sorted_and_deduplicated() {
        let mut summary = Summary::new();

        // Insert in non-sorted order: .3 then .1 then .2
        let pkt1 = make_tcp_packet([10, 0, 0, 3], [10, 0, 0, 1], 9999, 80, 54);
        let pkt2 = make_tcp_packet([10, 0, 0, 2], [10, 0, 0, 1], 9998, 443, 60);
        // pkt2 introduces 10.0.0.2 and re-introduces 10.0.0.1 (duplicate)
        summary.ingest(&pkt1);
        summary.ingest(&pkt2);

        let hosts = summary.unique_hosts();
        // Exactly 3 unique hosts
        assert_eq!(hosts.len(), 3);
        // Sorted ascending
        assert_eq!(hosts[0], IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
        assert_eq!(hosts[1], IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));
        assert_eq!(hosts[2], IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3)));
        // Verify sorted invariant holds across the whole slice
        for window in hosts.windows(2) {
            assert!(
                window[0] < window[1],
                "unique_hosts must be strictly sorted"
            );
        }
    }

    // =========================================================================
    // AC-010  BC-2.12.020 pc3: unique_hosts is empty when no packets ingested
    // =========================================================================

    #[test]
    fn test_unique_hosts_empty_when_no_packets() {
        let summary = Summary::new();
        assert!(
            summary.unique_hosts().is_empty(),
            "unique_hosts() must return an empty Vec when no packets have been ingested"
        );
    }

    // =========================================================================
    // AC-011  BC-2.12.020 inv4: unique_hosts is non-mutating — calling it
    //         multiple times returns identical results
    // =========================================================================

    #[test]
    fn test_unique_hosts_is_non_mutating() {
        let mut summary = Summary::new();
        let pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        summary.ingest(&pkt);

        // Call unique_hosts() three times; results must be identical
        let first = summary.unique_hosts();
        let second = summary.unique_hosts();
        let third = summary.unique_hosts();
        assert_eq!(
            first, second,
            "unique_hosts() must be idempotent (call 1 vs 2)"
        );
        assert_eq!(
            second, third,
            "unique_hosts() must be idempotent (call 2 vs 3)"
        );

        // total_packets must be unchanged — unique_hosts took &self
        assert_eq!(summary.total_packets, 1);
    }

    // =========================================================================
    // AC-012  BC-2.12.021 pc1: JsonReporter serializes a Summary
    //         with total_packets, total_bytes, and skipped_packets as u64
    //         integers in the JSON "summary" object
    // =========================================================================

    #[test]
    fn test_BC_2_12_021_json_includes_skipped_packets() {
        let mut summary = Summary::new();
        let pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 100);
        summary.ingest(&pkt);
        summary.skipped_packets = 7;

        let json = render_and_parse(&summary);
        let summary_block = &json["summary"];

        assert_eq!(
            summary_block["total_packets"],
            serde_json::Value::Number(1u64.into()),
            "total_packets must be a u64 integer in the JSON summary"
        );
        assert_eq!(
            summary_block["total_bytes"],
            serde_json::Value::Number(100u64.into()),
            "total_bytes must be a u64 integer in the JSON summary"
        );
        assert_eq!(
            summary_block["skipped_packets"],
            serde_json::Value::Number(7u64.into()),
            "skipped_packets must be a u64 integer in the JSON summary"
        );
    }

    // =========================================================================
    // AC-013  BC-2.12.021 pc7: protocol keys in the JSON output use
    //         Debug-format ("Tcp", "Udp", "Icmp"), not Display or numeric
    // =========================================================================

    #[test]
    fn test_summary_protocol_keys_use_debug_format() {
        let mut summary = Summary::new();
        let pkt_tcp = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        let pkt_udp = make_udp_packet([10, 0, 0, 3], [10, 0, 0, 4], 12345, 53, 40);
        let pkt_icmp = make_icmp_packet([10, 0, 0, 5], [10, 0, 0, 6], 28);
        summary.ingest(&pkt_tcp);
        summary.ingest(&pkt_udp);
        summary.ingest(&pkt_icmp);

        let json = render_and_parse(&summary);
        let protocols = &json["summary"]["protocols"];

        // Keys must be Debug format: "Tcp", "Udp", "Icmp"
        assert_eq!(
            protocols["Tcp"],
            serde_json::Value::Number(1u64.into()),
            "Tcp key must use Debug format (\"Tcp\", not \"TCP\" or numeric)"
        );
        assert_eq!(
            protocols["Udp"],
            serde_json::Value::Number(1u64.into()),
            "Udp key must use Debug format (\"Udp\", not \"UDP\" or numeric)"
        );
        assert_eq!(
            protocols["Icmp"],
            serde_json::Value::Number(1u64.into()),
            "Icmp key must use Debug format (\"Icmp\", not \"ICMP\" or numeric)"
        );

        // Negative: Display/uppercase variants must NOT appear
        assert!(
            protocols.get("TCP").is_none(),
            "Protocol key must not be all-caps \"TCP\""
        );
        assert!(
            protocols.get("UDP").is_none(),
            "Protocol key must not be all-caps \"UDP\""
        );
    }

    // =========================================================================
    // EC-001  BC-2.12.020 ec2: when src_ip == dst_ip, the host appears only
    //         once in unique_hosts (HashSet deduplication)
    // =========================================================================

    #[test]
    fn test_BC_2_12_020_ec2_src_eq_dst_counted_once() {
        let mut summary = Summary::new();

        // src_ip == dst_ip: both inserts into HashSet are the same address
        let pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 1], 12345, 80, 54);
        summary.ingest(&pkt);

        let hosts = summary.unique_hosts();
        assert_eq!(
            hosts.len(),
            1,
            "when src_ip == dst_ip, unique_hosts() must contain exactly 1 entry; \
             got: {hosts:?}"
        );
        assert_eq!(hosts[0], IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    }

    // =========================================================================
    // EC-002  BC-2.12.020 ec3: unique_hosts with a mix of IPv4 and IPv6
    //         addresses sorts IPv4 before IPv6 (IpAddr Ord places V4 first)
    // =========================================================================

    #[test]
    fn test_BC_2_12_020_ec3_ipv4_sorts_before_ipv6() {
        let mut summary = Summary::new();

        // IPv6 packet — ingested first to ensure sort is stable regardless of
        // insertion order
        let ipv6_src = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        let ipv6_dst = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 2);
        let pkt_v6 = make_tcp_packet_ipv6(ipv6_src, ipv6_dst, 12345, 80, 60);
        summary.ingest(&pkt_v6);

        // IPv4 packet — ingested second
        let pkt_v4 = make_tcp_packet([192, 168, 1, 1], [192, 168, 1, 2], 9999, 443, 54);
        summary.ingest(&pkt_v4);

        let hosts = summary.unique_hosts();
        assert_eq!(hosts.len(), 4, "expected 4 unique hosts (2 IPv4, 2 IPv6)");

        // First two must be IPv4
        assert!(
            matches!(hosts[0], IpAddr::V4(_)),
            "first host must be IPv4; got: {:?}",
            hosts[0]
        );
        assert!(
            matches!(hosts[1], IpAddr::V4(_)),
            "second host must be IPv4; got: {:?}",
            hosts[1]
        );
        // Last two must be IPv6
        assert!(
            matches!(hosts[2], IpAddr::V6(_)),
            "third host must be IPv6; got: {:?}",
            hosts[2]
        );
        assert!(
            matches!(hosts[3], IpAddr::V6(_)),
            "fourth host must be IPv6; got: {:?}",
            hosts[3]
        );
    }

    // =========================================================================
    // EC-003 (story scenario)  BC-2.12.018 pc4: two packets with the same
    //         protocol produce count 2 for that protocol key.
    //
    //         NOTE: This is NOT a BC-2.12.018 edge-case row.  BC-2.12.018's EC
    //         table contains EC-001 (host-dedup across packets), EC-002
    //         (src-as-dst dedup), EC-003 (ICMP protocol counter), and EC-004
    //         (zero-length packet).  The "same protocol accumulates" scenario is
    //         a canonical test vector / postcondition-4 path, not an EC row.
    //         The function is named pc4 to reflect its true anchor.
    // =========================================================================

    #[test]
    fn test_BC_2_12_018_pc4_same_protocol_accumulates() {
        let mut summary = Summary::new();

        let pkt1 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        let pkt2 = make_tcp_packet([10, 0, 0, 3], [10, 0, 0, 4], 9998, 443, 60);
        summary.ingest(&pkt1);
        summary.ingest(&pkt2);

        let counts = summary.protocol_counts();
        assert_eq!(
            counts.get(&Protocol::Tcp),
            Some(&2),
            "two TCP packets must produce protocol count of 2"
        );
        // No other protocols
        assert_eq!(counts.len(), 1, "only Tcp should appear in protocol_counts");
    }

    // =========================================================================
    // EC-004  BC-2.12.018 ec4: a packet with packet_len = 0 leaves
    //         total_bytes unchanged (adds 0)
    // =========================================================================

    #[test]
    fn test_BC_2_12_018_ec4_zero_length_packet_does_not_increment_bytes() {
        let mut summary = Summary::new();

        let pkt = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 0);
        summary.ingest(&pkt);

        assert_eq!(
            summary.total_bytes, 0,
            "packet_len=0 must leave total_bytes at 0"
        );
        // But total_packets still increments
        assert_eq!(summary.total_packets, 1);
    }

    // =========================================================================
    // EC-005  BC-2.12.019 ec4: after two HTTP packets, the JSON output contains
    //         "services": { "HTTP": 2 }
    // =========================================================================

    #[test]
    fn test_BC_2_12_019_ec4_services_http_count_two_in_json() {
        let mut summary = Summary::new();

        // Two HTTP packets (dst_port=80 → app_protocol_hint → Some("HTTP"))
        let pkt1 = make_tcp_packet([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80, 54);
        let pkt2 = make_tcp_packet([10, 0, 0, 3], [10, 0, 0, 4], 9998, 80, 60);
        summary.ingest(&pkt1);
        summary.ingest(&pkt2);

        let json = render_and_parse(&summary);
        let services = &json["summary"]["services"];

        assert_eq!(
            services["HTTP"],
            serde_json::Value::Number(2u64.into()),
            "after two HTTP packets, JSON services[\"HTTP\"] must be 2; \
             got services block: {services}"
        );
    }
}
