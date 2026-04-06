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

    let mut decode_errors = 0;
    for raw in &source.packets {
        if let Err(e) = decode_packet(&raw.data, source.datalink) {
            if decode_errors == 0 {
                panic!("Failed to decode packet in tls.pcap: {e}");
            }
            decode_errors += 1;
        }
    }
    assert_eq!(
        decode_errors, 0,
        "Expected no decode errors in tls.pcap, got {decode_errors}"
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

    let mut decode_errors = 0;
    for raw in &source.packets {
        if let Err(e) = decode_packet(&raw.data, source.datalink) {
            if decode_errors == 0 {
                panic!("Failed to decode packet in segmented.pcap: {e}");
            }
            decode_errors += 1;
        }
    }
    assert_eq!(
        decode_errors, 0,
        "Expected no decode errors in segmented.pcap, got {decode_errors}"
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

    let mut decode_errors = 0;
    for raw in &source.packets {
        if let Err(e) = decode_packet(&raw.data, source.datalink) {
            if decode_errors == 0 {
                panic!("Failed to decode packet in http-ooo.pcap: {e}");
            }
            decode_errors += 1;
        }
    }
    assert_eq!(
        decode_errors, 0,
        "Expected no decode errors in http-ooo.pcap, got {decode_errors}"
    );
}
