use std::net::IpAddr;
use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamAnalyzer, StreamHandler};

fn test_flow_key() -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        49153,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        443,
    )
}

/// Build a minimal TLS ClientHello record with SNI and specified cipher suites.
fn build_client_hello(sni: &str, cipher_ids: &[u16]) -> Vec<u8> {
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000)
    let sni_bytes = sni.as_bytes();
    let sni_list_len = (3 + sni_bytes.len()) as u16;
    let sni_ext_len = 2 + sni_list_len;
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&sni_ext_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_len.to_be_bytes());
    extensions.push(0x00); // host_name type
    extensions.extend_from_slice(&(sni_bytes.len() as u16).to_be_bytes());
    extensions.extend_from_slice(sni_bytes);

    // Supported Groups extension (type 0x000a)
    extensions.extend_from_slice(&[0x00, 0x0a]); // extension type
    extensions.extend_from_slice(&[0x00, 0x06]); // extension data length
    extensions.extend_from_slice(&[0x00, 0x04]); // named group list length
    extensions.extend_from_slice(&[0x00, 0x1d]); // x25519
    extensions.extend_from_slice(&[0x00, 0x17]); // secp256r1

    // EC Point Formats extension (type 0x000b)
    extensions.extend_from_slice(&[0x00, 0x0b]); // extension type
    extensions.extend_from_slice(&[0x00, 0x02]); // extension data length
    extensions.push(0x01); // ec point formats length
    extensions.push(0x00); // uncompressed

    // Build ClientHello body
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0

    let ciphers_len = (cipher_ids.len() * 2) as u16;
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression

    let ext_len = extensions.len() as u16;
    ch_body.extend_from_slice(&ext_len.to_be_bytes());
    ch_body.extend_from_slice(&extensions);

    // Handshake header
    let mut handshake = Vec::new();
    handshake.push(0x01); // ClientHello
    let ch_len = ch_body.len() as u32;
    handshake.push((ch_len >> 16) as u8);
    handshake.push((ch_len >> 8) as u8);
    handshake.push(ch_len as u8);
    handshake.extend_from_slice(&ch_body);

    // TLS record header
    let mut record = Vec::new();
    record.push(0x16); // handshake
    record.extend_from_slice(&[0x03, 0x01]); // TLS 1.0 record version
    let hs_len = handshake.len() as u16;
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

/// Build a minimal TLS ServerHello record.
fn build_server_hello(cipher_id: u16) -> Vec<u8> {
    // Extensions: just renegotiation_info (0xff01) with empty data
    let mut extensions = Vec::new();
    extensions.extend_from_slice(&[0xff, 0x01]); // renegotiation_info
    extensions.extend_from_slice(&[0x00, 0x01]); // extension data length
    extensions.push(0x00); // empty renegotiation info

    // ServerHello body
    let mut sh_body = Vec::new();
    sh_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    sh_body.extend_from_slice(&[0u8; 32]); // random
    sh_body.push(0x00); // session_id length: 0
    sh_body.extend_from_slice(&cipher_id.to_be_bytes()); // selected cipher
    sh_body.push(0x00); // compression: null

    let ext_len = extensions.len() as u16;
    sh_body.extend_from_slice(&ext_len.to_be_bytes());
    sh_body.extend_from_slice(&extensions);

    // Handshake header
    let mut handshake = Vec::new();
    handshake.push(0x02); // handshake type: ServerHello
    let sh_len = sh_body.len() as u32;
    handshake.push((sh_len >> 16) as u8);
    handshake.push((sh_len >> 8) as u8);
    handshake.push(sh_len as u8);
    handshake.extend_from_slice(&sh_body);

    // TLS record header
    let mut record = Vec::new();
    record.push(0x16);
    record.extend_from_slice(&[0x03, 0x03]);
    let hs_len = handshake.len() as u16;
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

#[test]
fn test_parse_client_hello() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(*analyzer.sni_counts().get("example.com").unwrap(), 1);
    assert_eq!(analyzer.ja3_counts().len(), 1);
    assert!(!analyzer.ja3_counts().is_empty());
    assert_eq!(*analyzer.version_counts().get(&0x0303).unwrap(), 1);
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_ja3_grease_filtering() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("test.com", &[0x0a0a, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(analyzer.ja3_counts().len(), 1);
    let ja3_hash = analyzer.ja3_counts().keys().next().unwrap();
    assert_eq!(ja3_hash.len(), 32); // MD5 hex = 32 chars
}

#[test]
fn test_parse_error_counter() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let bad_record = [0x16, 0x03, 0x03, 0x00, 0x05, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
    analyzer.on_data(&fk, Direction::ClientToServer, &bad_record, 0);

    assert_eq!(analyzer.parse_error_count(), 1);
}

#[test]
fn test_normal_request_no_parse_errors() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert!(analyzer.findings().is_empty());
}

#[test]
fn test_parse_server_hello() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    assert_eq!(analyzer.ja3s_counts().len(), 1);
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_weak_cipher_finding_client() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // TLS_RSA_WITH_NULL_SHA (0x0002) — NULL cipher
    let record = build_client_hello("test.com", &[0x0002, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].category, wirerust::findings::ThreatCategory::Anomaly);
    assert_eq!(findings[0].confidence, wirerust::findings::Confidence::High);
    assert!(findings[0].summary.contains("weak cipher"));
}

#[test]
fn test_weak_cipher_finding_server() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // Server selects TLS_RSA_WITH_RC4_128_SHA (0x0005)
    let sh = build_server_hello(0x0005);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    let findings = analyzer.findings();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].confidence, wirerust::findings::Confidence::Medium);
    assert!(findings[0].summary.contains("weak cipher"));
}

#[test]
fn test_normal_handshake_no_findings() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    assert!(analyzer.findings().is_empty());
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_stop_after_handshake() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    // Send encrypted application data (content_type=0x17) — should be ignored
    let mut app_data = vec![0x17, 0x03, 0x03, 0x00, 0x10];
    app_data.extend_from_slice(&[0xAA; 16]);
    analyzer.on_data(&fk, Direction::ServerToClient, &app_data, 0);

    // No parse errors from the encrypted data
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_summarize_output() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

    let summary = analyzer.summarize();
    assert_eq!(summary.analyzer_name, "TLS");
    assert_eq!(summary.packets_analyzed, 1);

    let detail = &summary.detail;
    assert!(detail["top_snis"]
        .as_array()
        .unwrap()
        .contains(&serde_json::json!("example.com")));
    assert!(detail.contains_key("ja3_hashes"));
    assert!(detail.contains_key("ja3s_hashes"));
    assert!(detail.contains_key("tls_versions"));
    assert!(detail.contains_key("cipher_suites"));
    assert_eq!(detail["parse_errors"], 0);
}
