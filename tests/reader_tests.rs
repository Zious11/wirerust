use std::io::Cursor;

use pcap_file::DataLink;
use wirerust::reader::PcapSource;

// Minimal valid pcap file: global header + 1 packet (Ethernet + IPv4 + TCP)
fn minimal_pcap_bytes() -> Vec<u8> {
    let mut buf = Vec::new();

    // Global header (24 bytes)
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes()); // magic
    buf.extend_from_slice(&2u16.to_le_bytes()); // version major
    buf.extend_from_slice(&4u16.to_le_bytes()); // version minor
    buf.extend_from_slice(&0i32.to_le_bytes()); // thiszone
    buf.extend_from_slice(&0u32.to_le_bytes()); // sigfigs
    buf.extend_from_slice(&65535u32.to_le_bytes()); // snaplen
    buf.extend_from_slice(&1u32.to_le_bytes()); // network (Ethernet)

    // Packet: Ethernet(14) + IPv4(20) + TCP(20) = 54 bytes
    let packet_data: Vec<u8> = vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00, // ethertype: IPv4
        // IPv4 header (20 bytes)
        0x45, 0x00, 0x00, 0x28, // version/IHL, DSCP, total length=40
        0x00, 0x01, 0x00, 0x00, // identification, flags/fragment
        0x40, 0x06, 0x00, 0x00, // TTL=64, protocol=TCP, checksum
        0x0a, 0x00, 0x00, 0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
        // TCP header (20 bytes)
        0x00, 0x50, 0x04, 0xd2, // src port 80, dst port 1234
        0x00, 0x00, 0x00, 0x01, // seq number
        0x00, 0x00, 0x00, 0x00, // ack number
        0x50, 0x02, 0xff, 0xff, // data offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00, // checksum, urgent pointer
    ];

    let captured_len = packet_data.len() as u32;

    // Packet header (16 bytes)
    buf.extend_from_slice(&1000u32.to_le_bytes()); // ts_sec
    buf.extend_from_slice(&0u32.to_le_bytes()); // ts_usec
    buf.extend_from_slice(&captured_len.to_le_bytes()); // incl_len
    buf.extend_from_slice(&captured_len.to_le_bytes()); // orig_len

    // Packet data
    buf.extend_from_slice(&packet_data);

    buf
}

#[test]
fn test_read_pcap_packets() {
    let data = minimal_pcap_bytes();
    let cursor = Cursor::new(data);
    let source = PcapSource::from_pcap_reader(cursor).unwrap();
    let packets = source.packets;
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].timestamp_secs, 1000);
    assert_eq!(packets[0].data.len(), 54);
}

#[test]
fn test_empty_pcap_no_packets() {
    let mut buf = Vec::new();
    // Global header only, no packets
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());

    let cursor = Cursor::new(buf);
    let source = PcapSource::from_pcap_reader(cursor).unwrap();
    assert!(source.packets.is_empty());
}

#[test]
fn test_unsupported_link_type_rejected() {
    let mut buf = Vec::new();
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&105u32.to_le_bytes()); // IEEE802_11 (unsupported)

    let cursor = Cursor::new(buf);
    let result = PcapSource::from_pcap_reader(cursor);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Unsupported"),
        "Error should mention 'Unsupported', got: {err_msg}"
    );
}

fn pcap_header_with_linktype(link_type: u32) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&link_type.to_le_bytes());
    buf
}

#[test]
fn test_pcap_source_stores_datalink() {
    let data = minimal_pcap_bytes();
    let cursor = Cursor::new(data);
    let source = PcapSource::from_pcap_reader(cursor).unwrap();
    assert_eq!(source.datalink, DataLink::ETHERNET);
}

#[test]
fn test_reader_accepts_raw_linktype() {
    let buf = pcap_header_with_linktype(101); // RAW
    let source = PcapSource::from_pcap_reader(Cursor::new(buf)).unwrap();
    assert_eq!(source.datalink, DataLink::RAW);
}

#[test]
fn test_reader_accepts_ipv4_linktype() {
    let buf = pcap_header_with_linktype(228); // IPV4
    let source = PcapSource::from_pcap_reader(Cursor::new(buf)).unwrap();
    assert_eq!(source.datalink, DataLink::IPV4);
}

#[test]
fn test_reader_accepts_ipv6_linktype() {
    let buf = pcap_header_with_linktype(229); // IPV6
    let source = PcapSource::from_pcap_reader(Cursor::new(buf)).unwrap();
    assert_eq!(source.datalink, DataLink::IPV6);
}

#[test]
fn test_reader_accepts_linux_sll_linktype() {
    let buf = pcap_header_with_linktype(113); // LINUX_SLL
    let source = PcapSource::from_pcap_reader(Cursor::new(buf)).unwrap();
    assert_eq!(source.datalink, DataLink::LINUX_SLL);
}
