use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{Protocol, TransportInfo, decode_packet};

fn make_tcp_packet() -> Vec<u8> {
    vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00, // ethertype: IPv4
        // IPv4 header (20 bytes)
        0x45, 0x00, 0x00, 0x28, 0x00, 0x01, 0x00, 0x00, 0x40, 0x06, 0x00, 0x00, 0xc0, 0xa8, 0x01,
        0x0a, // src: 192.168.1.10
        0xc0, 0xa8, 0x01, 0x01, // dst: 192.168.1.1
        // TCP header (20 bytes)
        0xc0, 0x01, 0x00, 0x50, // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x50, 0x02, 0xff, 0xff, 0x00, 0x00, 0x00,
        0x00,
    ]
}

fn make_udp_packet() -> Vec<u8> {
    vec![
        // Ethernet header
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x08, 0x00,
        // IPv4 header (20 bytes), protocol=UDP (0x11)
        0x45, 0x00, 0x00, 0x1c, 0x00, 0x01, 0x00, 0x00, 0x40, 0x11, 0x00, 0x00, 0x0a, 0x00, 0x00,
        0x01, // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02, // dst: 10.0.0.2
        // UDP header (8 bytes)
        0xd9, 0x03, 0x00, 0x35, // src port 55555, dst port 53
        0x00, 0x08, 0x00, 0x00, // length=8, checksum
    ]
}

#[test]
fn test_decode_tcp_packet() {
    let data = make_tcp_packet();
    let parsed = decode_packet(&data).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));

    match parsed.transport {
        TransportInfo::Tcp {
            src_port, dst_port, ..
        } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }

    assert_eq!(parsed.protocol, Protocol::Tcp);
}

#[test]
fn test_decode_udp_dns_packet() {
    let data = make_udp_packet();
    let parsed = decode_packet(&data).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));

    match parsed.transport {
        TransportInfo::Udp { src_port, dst_port } => {
            assert_eq!(src_port, 55555);
            assert_eq!(dst_port, 53);
        }
        _ => panic!("Expected UDP"),
    }

    assert_eq!(parsed.protocol, Protocol::Udp);
    assert_eq!(parsed.app_protocol_hint(), Some("DNS"));
}

#[test]
fn test_decode_invalid_packet() {
    let garbage = vec![0x00, 0x01, 0x02];
    assert!(decode_packet(&garbage).is_err());
}
