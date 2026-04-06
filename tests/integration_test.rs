use std::io::Cursor;

use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reporter::Reporter;
use wirerust::reporter::json::JsonReporter;
use wirerust::summary::Summary;

fn minimal_pcap_with_tcp() -> Vec<u8> {
    let mut buf = Vec::new();
    // Global header
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());

    let packet_data: Vec<u8> = vec![
        // Ethernet
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x00,
        // IPv4
        0x45, 0x00, 0x00, 0x28, 0x00, 0x01, 0x00, 0x00, 0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8, 0x01,
        0x0a, 0xc0, 0xa8, 0x01, 0x01, // TCP
        0xc0, 0x01, 0x00, 0x50, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x50, 0x02, 0xff,
        0xff, 0x00, 0x00, 0x00, 0x00,
    ];

    let captured_len = packet_data.len() as u32;
    buf.extend_from_slice(&1000u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&captured_len.to_le_bytes());
    buf.extend_from_slice(&captured_len.to_le_bytes());
    buf.extend_from_slice(&packet_data);
    buf
}

#[test]
fn test_full_pipeline() {
    let data = minimal_pcap_with_tcp();
    let source = PcapSource::from_pcap_reader(Cursor::new(data)).unwrap();

    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();

    for raw in &source.packets {
        let parsed = decode_packet(&raw.data, source.datalink)
            .expect("test fixture packet should decode successfully");
        summary.ingest(&parsed);
        if dns_analyzer.can_decode(&parsed) {
            let findings = dns_analyzer.analyze(&parsed);
            all_findings.extend(findings);
        }
    }

    assert_eq!(summary.total_packets, 1);

    let reporter = JsonReporter;
    let output = reporter.render(&summary, &all_findings, &[dns_analyzer.summarize()]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["summary"]["total_packets"], 1);
}
