use std::net::{IpAddr, Ipv4Addr};

use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};

fn make_dns_packet(payload: &[u8]) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Udp,
        transport: TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        payload: payload.to_vec(),
        packet_len: 60 + payload.len(),
    }
}

fn make_non_dns_packet() -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port: 12345,
            dst_port: 80,
            syn: true,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 54,
    }
}

#[test]
fn test_dns_analyzer_matches_dns_packets() {
    let analyzer = DnsAnalyzer::new();
    let dns_pkt = make_dns_packet(&[0; 12]); // minimal DNS header
    let non_dns = make_non_dns_packet();

    assert!(analyzer.can_decode(&dns_pkt));
    assert!(!analyzer.can_decode(&non_dns));
}

#[test]
fn test_dns_analyzer_counts_queries() {
    let mut analyzer = DnsAnalyzer::new();
    let pkt = make_dns_packet(&[0; 12]);

    let findings = analyzer.analyze(&pkt);
    // No findings from a single normal query
    assert!(findings.is_empty());

    let summary = analyzer.summarize();
    assert_eq!(summary.packets_analyzed, 1);
    assert!(summary.detail.contains_key("dns_queries"));
}

#[test]
fn test_analyzer_trait_name() {
    let analyzer = DnsAnalyzer::new();
    assert_eq!(analyzer.name(), "DNS");
}
