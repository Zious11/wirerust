use pcap_file::DataLink;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;

#[test]
fn test_ethernet_pcap_tls() {
    let source = PcapSource::from_file(std::path::Path::new("tests/fixtures/tls.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::ETHERNET);
    assert!(
        source.packets.len() > 0,
        "tls.pcap should have packets, got 0"
    );

    let mut decoded_count = 0;
    for raw in &source.packets {
        if decode_packet(&raw.data, source.datalink).is_ok() {
            decoded_count += 1;
        }
    }
    assert!(
        decoded_count > 0,
        "Should decode at least some packets from tls.pcap, got 0"
    );
}

#[test]
fn test_raw_ip_pcap_segmented() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/segmented.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::RAW);
    assert!(
        source.packets.len() > 0,
        "segmented.pcap should have packets, got 0"
    );

    let mut decoded_count = 0;
    for raw in &source.packets {
        if decode_packet(&raw.data, source.datalink).is_ok() {
            decoded_count += 1;
        }
    }
    assert!(
        decoded_count > 0,
        "Should decode at least some packets from segmented.pcap, got 0"
    );
}

#[test]
fn test_ipv4_pcap_http_ooo() {
    let source =
        PcapSource::from_file(std::path::Path::new("tests/fixtures/http-ooo.pcap")).unwrap();
    assert_eq!(source.datalink, DataLink::IPV4);
    assert!(
        source.packets.len() > 0,
        "http-ooo.pcap should have packets, got 0"
    );

    let mut decoded_count = 0;
    for raw in &source.packets {
        if decode_packet(&raw.data, source.datalink).is_ok() {
            decoded_count += 1;
        }
    }
    assert!(
        decoded_count > 0,
        "Should decode at least some packets from http-ooo.pcap, got 0"
    );
}
