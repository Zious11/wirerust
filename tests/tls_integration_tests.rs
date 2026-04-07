use wirerust::analyzer::tls::TlsAnalyzer;
use wirerust::decoder::decode_packet;
use wirerust::dispatcher::StreamDispatcher;
use wirerust::reader::PcapSource;
use wirerust::reassembly::handler::StreamAnalyzer;
use wirerust::reassembly::{ReassemblyConfig, TcpReassembler};

/// Run the full pipeline: PCAP → decoder → reassembler → dispatcher → TLS analyzer
fn analyze_pcap(path: &str) -> TlsAnalyzer {
    let source = PcapSource::from_file(std::path::Path::new(path)).unwrap();
    let config = ReassemblyConfig::default();
    let mut reasm = TcpReassembler::new(config);
    let mut dispatcher = StreamDispatcher::new(None, Some(TlsAnalyzer::new()));

    for raw in &source.packets {
        if let Ok(parsed) = decode_packet(&raw.data, source.datalink) {
            reasm.process_packet(&parsed, raw.timestamp_secs, &mut dispatcher);
        }
    }
    reasm.finalize(&mut dispatcher);

    // Move the TLS analyzer out of the dispatcher
    dispatcher.tls.unwrap()
}

#[test]
fn test_tls12_pcap_sni_and_ja3() {
    let tls = analyze_pcap("tests/fixtures/tls12-aes256gcm.pcap");

    // Should have parsed at least 1 ClientHello
    assert!(tls.handshake_count() >= 1);
    assert_eq!(tls.parse_error_count(), 0);

    // SNI should be "localhost"
    assert_eq!(*tls.sni_counts().get("localhost").unwrap(), 1);

    // JA3 should be computed (32-char hex)
    assert_eq!(tls.ja3_counts().len(), 1);
    let ja3 = tls.ja3_counts().keys().next().unwrap();
    assert_eq!(ja3.len(), 32);

    // JA3S should be computed
    assert!(!tls.ja3s_counts().is_empty());

    // TLS version should be 771 (0x0303 = TLS 1.2)
    assert!(tls.version_counts().contains_key(&0x0303));

    // No findings for modern cipher suites
    assert!(tls.findings().is_empty());
}

#[test]
fn test_tls13_pcap_version_and_ja3() {
    let tls = analyze_pcap("tests/fixtures/tls13-rfc8446.pcap");

    // Should have parsed 2 ClientHellos (2 connections in this capture)
    assert_eq!(tls.handshake_count(), 2);
    assert_eq!(tls.parse_error_count(), 0);

    // TLS 1.3 ClientHello legacy_version is 0x0303 (771)
    // This is correct per JA3 spec — use header version, not supported_versions
    assert!(tls.version_counts().contains_key(&0x0303));

    // 2 unique JA3 hashes
    assert_eq!(tls.ja3_counts().len(), 2);

    // JA3S hashes computed
    assert_eq!(tls.ja3s_counts().len(), 2);

    // No findings — modern ciphers
    assert!(tls.findings().is_empty());
}

#[test]
fn test_ssl30_pcap_generates_findings() {
    let tls = analyze_pcap("tests/fixtures/tls.pcap");

    // SSL 3.0 (version 0x0300 = 768) should be detected
    assert!(tls.version_counts().contains_key(&0x0300));

    // Should generate findings for deprecated protocol AND weak ciphers
    let findings = tls.findings();
    assert!(
        !findings.is_empty(),
        "SSL 3.0 traffic should generate security findings"
    );

    // At least one finding about deprecated protocol
    assert!(
        findings
            .iter()
            .any(|f| f.summary.contains("deprecated protocol")),
        "Should flag SSL 3.0 as deprecated"
    );

    // At least one finding about weak ciphers (export ciphers in this capture)
    assert!(
        findings.iter().any(|f| f.summary.contains("weak cipher")),
        "Should flag export cipher suites"
    );
}

#[test]
fn test_summarize_has_all_required_fields() {
    let tls = analyze_pcap("tests/fixtures/tls12-aes256gcm.pcap");
    let summary = tls.summarize();

    assert_eq!(summary.analyzer_name, "TLS");
    let detail = &summary.detail;

    // All required keys present
    assert!(detail.contains_key("top_snis"), "missing top_snis");
    assert!(detail.contains_key("ja3_hashes"), "missing ja3_hashes");
    assert!(detail.contains_key("ja3s_hashes"), "missing ja3s_hashes");
    assert!(detail.contains_key("tls_versions"), "missing tls_versions");
    assert!(
        detail.contains_key("cipher_suites"),
        "missing cipher_suites"
    );
    assert!(detail.contains_key("parse_errors"), "missing parse_errors");
}
