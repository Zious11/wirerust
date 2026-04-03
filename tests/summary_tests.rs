use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::summary::Summary;

fn make_parsed(src: [u8; 4], dst: [u8; 4], src_port: u16, dst_port: u16) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            syn: false,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 54,
    }
}

#[test]
fn test_summary_host_counting() {
    let packets = vec![
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 3], 12346, 443),
        make_parsed([10, 0, 0, 2], [10, 0, 0, 1], 80, 12345),
    ];

    let mut summary = Summary::new();
    for pkt in &packets {
        summary.ingest(pkt);
    }

    assert_eq!(summary.total_packets, 3);
    assert_eq!(summary.unique_hosts().len(), 3); // 10.0.0.1, .2, .3
}

#[test]
fn test_summary_protocol_breakdown() {
    let packets = vec![
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12346, 80),
    ];

    let mut summary = Summary::new();
    for pkt in &packets {
        summary.ingest(pkt);
    }

    let proto_counts = summary.protocol_counts();
    assert_eq!(*proto_counts.get(&Protocol::Tcp).unwrap(), 2);
}

#[test]
fn test_summary_service_detection() {
    let packets = vec![
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12346, 443),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12347, 443),
    ];

    let mut summary = Summary::new();
    for pkt in &packets {
        summary.ingest(pkt);
    }

    let services = summary.service_counts();
    assert_eq!(*services.get("HTTP").unwrap(), 1);
    assert_eq!(*services.get("TLS").unwrap(), 2);
}
