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

    // AC-010 (BC-2.07.032 pc2): JA3 is computed using legacy_version (0x0303 = 771),
    // not the supported_versions extension value (0x0304 = TLS 1.3).  The JA3 string
    // format is "<version>,<ciphers>,<exts>,<curves>,<pf>", so the first field of
    // every JA3 string produced from a TLS 1.3 ClientHello must be "771".
    // We verify this against the live JA3 pre-image stored in ja3_counts keys.
    //
    // Note: ja3_counts stores 32-char MD5 hex hashes, NOT the raw JA3 strings.
    // The JA3 string is not separately stored by TlsAnalyzer.  Instead we re-derive
    // the assertion: if the version in version_counts is ONLY 0x0303 (i.e. the
    // supported_versions extension's 0x0304 was NOT recorded), then JA3 must have
    // used 0x0303 as its version field.  We additionally verify that 0x0304 is
    // absent from version_counts — if the analyzer had inspected supported_versions,
    // it would have recorded 0x0304 there.
    assert!(
        !tls.version_counts().contains_key(&0x0304),
        "AC-010 / AC-011 (BC-2.07.032 pc2 + inv1): version_counts must NOT contain \
         0x0304 (TLS 1.3) — TlsAnalyzer uses only ch.version.0 (legacy_version 0x0303), \
         NOT the supported_versions extension value. Got version_counts: {:?}",
        tls.version_counts()
    );

    // JA3S hashes computed
    assert_eq!(tls.ja3s_counts().len(), 2);

    // No findings — modern ciphers
    assert!(tls.findings().is_empty());
}

// STORY-054 Brownfield-Formalization — Integration test strengthening
// AC-006 (BC-2.07.011 postconditions 1-2): full pipeline with real SSL 3.0 PCAP.
#[test]
fn test_ssl30_pcap_generates_findings() {
    // AC-006 / BC-2.07.011 postconditions 1-2: full pipeline produces findings for
    // SSL 3.0 deprecated protocol and weak export ciphers.
    let tls = analyze_pcap("tests/fixtures/tls.pcap");

    // BC-2.07.011 precondition: SSL 3.0 (0x0300 = 768) is present in the capture.
    assert!(
        tls.version_counts().contains_key(&0x0300),
        "AC-006 precondition (BC-2.07.011 pc2): version_counts must contain SSL 3.0 (0x0300)"
    );

    let findings = tls.findings();

    // BC-2.07.011 postcondition 1: at least one finding was generated.
    assert!(
        !findings.is_empty(),
        "AC-006 (BC-2.07.011 pc1): SSL 3.0 traffic must generate security findings"
    );

    // BC-2.07.011 postcondition 1: at least one deprecated-protocol finding.
    let deprecated_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();
    assert!(
        !deprecated_findings.is_empty(),
        "AC-006 (BC-2.07.011 pc1): SSL 3.0 PCAP must produce at least one \
         deprecated-protocol finding; got findings: {:?}",
        findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
    );

    // BC-2.07.011 postcondition 1: the deprecated-protocol finding has correct fields.
    let dep = deprecated_findings[0];
    assert_eq!(
        dep.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-006 (BC-2.07.011 pc1): deprecated-protocol finding must be Anomaly"
    );
    assert_eq!(
        dep.verdict,
        wirerust::findings::Verdict::Likely,
        "AC-006 (BC-2.07.011 pc1): deprecated-protocol finding must be Likely"
    );
    assert_eq!(
        dep.confidence,
        wirerust::findings::Confidence::High,
        "AC-006 (BC-2.07.011 pc1): deprecated-protocol finding must be High confidence"
    );
    assert!(
        dep.summary.contains("RFC 7568"),
        "AC-006 (BC-2.07.011 invariant 2): summary must contain 'RFC 7568'; got: {:?}",
        dep.summary
    );
    assert!(
        dep.summary.contains("SSL 3.0"),
        "AC-006 (BC-2.07.011 pc2): summary must name 'SSL 3.0' for version 0x0300; \
         got: {:?}",
        dep.summary
    );
    assert_eq!(
        dep.mitre_technique, None,
        "AC-006 (BC-2.07.011 pc1): mitre_technique must be None"
    );

    // BC-2.07.009 postcondition 1: at least one weak-cipher finding (export ciphers in capture).
    let weak_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .collect();
    assert!(
        !weak_findings.is_empty(),
        "AC-006 (BC-2.07.009 pc1): SSL 3.0 PCAP with export ciphers must produce at least one \
         weak-cipher finding; got findings: {:?}",
        findings.iter().map(|f| &f.summary).collect::<Vec<_>>()
    );

    // BC-2.07.009 postcondition 1: the weak-cipher finding has correct fields.
    let wk = weak_findings[0];
    assert_eq!(
        wk.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-006 (BC-2.07.009 pc1): weak-cipher finding must be Anomaly"
    );
    assert_eq!(
        wk.confidence,
        wirerust::findings::Confidence::High,
        "AC-006 (BC-2.07.009 pc1): client weak-cipher finding must be High confidence"
    );
    assert_eq!(
        wk.mitre_technique, None,
        "AC-006 (BC-2.07.009 pc1): weak-cipher finding must have mitre_technique=None"
    );
    // BC-2.07.009 postcondition 1 (INV-4): evidence contains readable cipher names (not hex).
    assert!(
        wk.evidence.iter().any(|e| !e.starts_with("0x")),
        "AC-006 (BC-2.07.009 pc1 / INV-4): weak-cipher evidence must contain readable \
         cipher names (not hex IDs); got evidence: {:?}",
        wk.evidence
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
    // LESSON-P1.05: capacity/DoS-class drops are now surfaced
    // separately from parse_errors, so any JSON consumer can
    // distinguish "record was malformed" from "record was over-cap".
    assert!(
        detail.contains_key("truncated_records"),
        "missing truncated_records — LESSON-P1.05 regressed"
    );
    let truncated = detail
        .get("truncated_records")
        .expect("truncated_records key")
        .as_u64()
        .expect("truncated_records is u64");
    assert_eq!(
        truncated, 0,
        "well-formed tls12 fixture must not trigger truncated_records"
    );
}
