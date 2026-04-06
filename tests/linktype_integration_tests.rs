use pcap_file::DataLink;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;

#[test]
fn test_ethernet_pcap_tls() {
    let source = PcapSource::from_file(std::path::Path::new("tests/fixtures/tls.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::ETHERNET);
    assert_eq!(source.packets.len(), 58, "tls.pcap packet count");

    for raw in &source.packets {
        decode_packet(&raw.data, source.datalink).expect("all packets in tls.pcap should decode");
    }
}

#[test]
fn test_raw_ip_pcap_segmented() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/segmented.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::RAW);
    assert_eq!(source.packets.len(), 20, "segmented.pcap packet count");

    for raw in &source.packets {
        decode_packet(&raw.data, source.datalink)
            .expect("all packets in segmented.pcap should decode");
    }
}

#[test]
fn test_ipv4_pcap_http_ooo() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/http-ooo.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::IPV4);
    assert_eq!(source.packets.len(), 16, "http-ooo.pcap packet count");

    for raw in &source.packets {
        decode_packet(&raw.data, source.datalink)
            .expect("all packets in http-ooo.pcap should decode");
    }
}
