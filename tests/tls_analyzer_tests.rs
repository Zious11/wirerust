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
    build_client_hello_raw_sni(sni.as_bytes(), cipher_ids)
}

/// Build a minimal TLS ClientHello record with arbitrary raw bytes for the SNI hostname.
/// Used for tests that exercise non-UTF-8 / malformed SNI handling.
fn build_client_hello_raw_sni(sni_bytes: &[u8], cipher_ids: &[u16]) -> Vec<u8> {
    build_client_hello_with_sni_list(&[sni_bytes], cipher_ids)
}

/// Build a minimal TLS ClientHello record with an arbitrary number of SNI hostname
/// entries (zero or more). Used for edge-case tests covering empty SNI lists,
/// multi-name SNI lists, and other scenarios the single-hostname helpers don't reach.
///
/// Passing `sni_entries: &[]` produces a `ServerNameList` of length 0 — this is a
/// technically RFC-violating wire form (RFC 6066 §3 requires `server_name_list<1..2^16-1>`)
/// but is used to exercise the analyzer's defensive `list.first() == None` branch in
/// `extract_sni`. The current `tls_parser` crate accepts it at parse time.
fn build_client_hello_with_sni_list(sni_entries: &[&[u8]], cipher_ids: &[u16]) -> Vec<u8> {
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000)
    // The ServerNameList contents: for each entry, 1 byte host_name type + 2 byte name length + name bytes.
    // Use checked conversions so a future test passing an out-of-range SNI (e.g. a
    // ~65,532-byte single hostname — u16::MAX minus 3 bytes of ServerName overhead
    // per entry — requested in issue #52) panics with a clear message rather than
    // silently wrapping through `as u16` and producing a malformed ClientHello.
    let mut sni_list_data = Vec::new();
    for entry in sni_entries {
        let name_len = u16::try_from(entry.len())
            .expect("SNI hostname entry exceeds u16::MAX; can't encode as RFC 6066 HostName");
        sni_list_data.push(0x00); // host_name type
        sni_list_data.extend_from_slice(&name_len.to_be_bytes());
        sni_list_data.extend_from_slice(entry);
    }
    let sni_list_len =
        u16::try_from(sni_list_data.len()).expect("ServerNameList payload exceeds u16::MAX");
    let sni_ext_len = sni_list_len
        .checked_add(2)
        .expect("SNI extension length would overflow u16");
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&sni_ext_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_data);

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

    let ciphers_byte_len = cipher_ids
        .len()
        .checked_mul(2)
        .expect("cipher_ids list byte length overflows usize");
    let ciphers_len = u16::try_from(ciphers_byte_len)
        .expect("cipher_ids list exceeds u16::MAX bytes (max 32,767 suites)");
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression

    let ext_len = u16::try_from(extensions.len())
        .expect("ClientHello extensions block exceeds u16::MAX bytes");
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
    let hs_len = u16::try_from(handshake.len())
        .expect("TLS handshake body exceeds u16::MAX bytes; record fragmentation not implemented");
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
    assert_eq!(
        findings[0].category,
        wirerust::findings::ThreatCategory::Anomaly
    );
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
    assert_eq!(
        findings[0].confidence,
        wirerust::findings::Confidence::Medium
    );
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
    assert!(
        detail["top_snis"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("example.com"))
    );
    assert!(detail.contains_key("ja3_hashes"));
    assert!(detail.contains_key("ja3s_hashes"));
    assert!(detail.contains_key("tls_versions"));
    assert!(detail.contains_key("cipher_suites"));
    assert_eq!(detail["parse_errors"], 0);
}

#[test]
fn test_non_utf8_sni_emits_finding_and_counts_under_hex_key() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // 0xFF / 0xFE are invalid as standalone UTF-8 start bytes — guarantees
    // from_utf8 fails. Mix in some ASCII so the lossy form is recognizable.
    let raw_sni: &[u8] = &[0xff, 0xfe, b'a', b'.', b'c', b'o', b'm'];
    let record = build_client_hello_raw_sni(raw_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // Parse error counter must NOT be incremented — the record itself parsed,
    // only the SNI hostname bytes failed UTF-8 decoding.
    assert_eq!(analyzer.parse_error_count(), 0);

    // sni_counts should be keyed on a tagged hex form, not on a lossy string.
    // This guarantees distinct byte sequences don't collide.
    let expected_key = "<non-utf8:fffe612e636f6d>";
    assert_eq!(
        *analyzer.sni_counts().get(expected_key).unwrap(),
        1,
        "expected sni_counts to contain hex-tagged key {expected_key}"
    );

    // Exactly one finding, the non-UTF-8 SNI anomaly.
    let findings = analyzer.findings();
    let non_utf8_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .collect();
    assert_eq!(
        non_utf8_findings.len(),
        1,
        "expected exactly one non-UTF-8 SNI finding, got: {findings:?}"
    );
    let f = non_utf8_findings[0];
    assert_eq!(f.category, wirerust::findings::ThreatCategory::Anomaly);
    assert_eq!(f.verdict, wirerust::findings::Verdict::Inconclusive);
    assert_eq!(f.confidence, wirerust::findings::Confidence::Low);
    assert!(f.summary.contains("RFC 6066"));
    // Hex evidence is the lossless representation of the raw bytes.
    assert!(
        f.evidence.iter().any(|e| e.contains("fffe612e636f6d")),
        "expected hex evidence to contain raw byte sequence, got: {:?}",
        f.evidence
    );
}

#[test]
fn test_ascii_sni_does_not_emit_non_utf8_finding() {
    // Regression: a normal ASCII hostname must not trip the non-UTF-8 finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(*analyzer.sni_counts().get("example.com").unwrap(), 1);
    let non_utf8_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(non_utf8_findings, 0);
}

#[test]
fn test_non_utf8_sni_preserves_raw_bytes_in_summary() {
    // Per ADR 0003: the Finding struct is the data layer — it stores the
    // raw post-from_utf8_lossy bytes from the attacker's SNI, including
    // any ASCII control codes. Terminal-safety is the reporter's job, not
    // the analyzer's. This test enforces that contract: raw ESC must
    // survive to the struct; downstream rendering tests (in reporter
    // tests) verify the terminal reporter escapes it on display.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // 0xff makes from_utf8 fail; 0x1b [ 3 1 m is the ANSI "red" CSI sequence;
    // "pwnd" is the visible payload an attacker would inject.
    let raw_sni: &[u8] = &[0xff, 0x1b, b'[', b'3', b'1', b'm', b'p', b'w', b'n', b'd'];
    let record = build_client_hello_raw_sni(raw_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let findings = analyzer.findings();
    let f = findings
        .iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("expected non-UTF-8 SNI finding");

    // The summary MUST contain the raw ESC byte — the analyzer does not
    // escape. Forensic preservation is a load-bearing property of the
    // data layer (ADR 0003).
    assert!(
        f.summary.as_bytes().contains(&0x1b),
        "summary must preserve raw ESC byte for forensics, got: {:?}",
        f.summary
    );
    // And it must NOT contain the Debug-formatted escape form (which
    // would indicate a regression to construction-site escaping).
    assert!(
        !f.summary.contains("\\u{1b}"),
        "summary must not contain Debug-formatted escape (regression to construction-site), got: {}",
        f.summary
    );

    // Hex evidence is unchanged — that's the lossless record.
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("ff1b5b33316d70776e64")),
        "expected raw bytes in hex evidence, got: {:?}",
        f.evidence
    );
}

// ── SNI edge-case pin tests (issue #50) ──────────────────────────────────────
//
// These tests pin the analyzer's behavior for SNI inputs that the existing
// tests don't reach. Each one documents the *current* behavior so a future
// refactor can't silently change it.

#[test]
fn test_sni_extension_with_empty_hostname_list() {
    // Pin: an SNI extension whose ServerNameList contains zero entries should
    // be treated as "no SNI" — extract_sni returns None, no count, no finding.
    // Whether tls_parser ever produces this on the wire is implementation-defined,
    // but the analyzer's branch (`list.first()` returning None) must be safe.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_with_sni_list(&[], &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // Guard against a silent false-positive: if tls_parser ever tightens to
    // reject zero-entry ServerNameList (RFC 6066 §3: server_name_list<1..2^16-1>),
    // parse_tls_extensions would fail and the subsequent assertions would still
    // pass — but for the wrong reason. The branch we actually want to pin is
    // `list.first() == None`, which requires the extension to have parsed
    // successfully as `TlsExtension::SNI(empty_list)`.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "extensions must parse cleanly; an empty-list pin that fires because \
         tls_parser rejected the extension is not pinning the extract_sni branch \
         we think it is"
    );
    assert!(
        analyzer.sni_counts().is_empty(),
        "sni_counts should be untouched when SNI list is empty, got {:?}",
        analyzer.sni_counts()
    );
    assert!(
        analyzer.findings().is_empty(),
        "no findings should fire for an empty SNI list, got {:?}",
        analyzer.findings()
    );
    // The handshake itself still parsed, so handshake_count should advance.
    assert_eq!(analyzer.handshake_count(), 1);
}

#[test]
fn test_sni_with_empty_hostname_bytes() {
    // Pin current behavior for a degenerate, RFC-violating SNI hostname whose
    // HostName bytes are empty (b""). RFC 6066 §3 defines `HostName<1..2^16-1>`,
    // so a zero-byte hostname is not spec-valid wire form. Current tls_parser
    // accepts it anyway; from_utf8 decodes it as Ok(""), so the analyzer counts
    // it under the empty-string key and emits no non-UTF-8 finding. This pin
    // guards the defensive path — the analyzer's existing branch handles the
    // degenerate case without panicking or double-counting.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_raw_sni(b"", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(
        *analyzer.sni_counts().get("").unwrap_or(&0),
        1,
        "expected empty-string SNI key with count 1, got {:?}",
        analyzer.sni_counts()
    );
    let non_utf8_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(non_utf8_findings, 0);
}

#[test]
fn test_valid_utf8_non_ascii_sni_emits_finding() {
    // A SNI value that is valid UTF-8 but contains non-ASCII characters (e.g. a
    // raw U-label "café.example") is an RFC 6066 §3 violation: the spec requires
    // ASCII encoding, with internationalized names sent as A-labels (RFC 5890
    // Punycode `xn--` form). Major TLS clients (rustls, Chrome/BoringSSL,
    // Firefox/NSS, curl/libcurl) all auto-Punycode internationalized hostnames
    // before sending, so a non-ASCII SNI on the wire is a strong indicator of
    // either a buggy custom client (often raw OpenSSL without IDNA prep) or an
    // attacker tool.
    //
    // This test was previously a pin-test for "no finding emitted" pending
    // issue #51; that issue is now implemented and the assertion has been
    // flipped to expect the finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("café.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // Hostname is still counted under its valid-UTF-8 form (no collision risk).
    assert_eq!(
        *analyzer.sni_counts().get("café.example").unwrap_or(&0),
        1,
        "expected café.example to be counted under its UTF-8 form, got {:?}",
        analyzer.sni_counts()
    );

    let non_ascii_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .collect();
    assert_eq!(
        non_ascii_findings.len(),
        1,
        "expected exactly one non-ASCII SNI finding, got: {:?}",
        analyzer.findings()
    );
    let f = &non_ascii_findings[0];
    assert_eq!(f.category, wirerust::findings::ThreatCategory::Anomaly);
    assert_eq!(f.verdict, wirerust::findings::Verdict::Inconclusive);
    assert_eq!(f.confidence, wirerust::findings::Confidence::Low);
    assert!(
        f.summary.contains("RFC 6066"),
        "summary should reference RFC 6066, got: {}",
        f.summary
    );
    assert!(
        f.summary.contains("A-labels"),
        "summary should mention A-labels, got: {}",
        f.summary
    );
    // Hex evidence is the lossless representation of the raw bytes.
    // "café.example" UTF-8 = 63 61 66 c3 a9 2e 65 78 61 6d 70 6c 65
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("636166c3a92e6578616d706c65")),
        "expected raw UTF-8 bytes in hex evidence, got: {:?}",
        f.evidence
    );

    // Critical: this must NOT trip the non-UTF-8 finding (those are different
    // cases — café.example is valid UTF-8, just non-ASCII).
    let non_utf8_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(non_utf8_findings, 0);
}

#[test]
fn test_cyrillic_sni_emits_non_ascii_finding() {
    // Regression: a U-label using Cyrillic script should trip the non-ASCII
    // finding identically to the Latin-accented case.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("пример.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let non_ascii_count = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .count();
    assert_eq!(
        non_ascii_count,
        1,
        "Cyrillic U-label must emit one non-ASCII finding, got {:?}",
        analyzer.findings()
    );
    assert_eq!(
        *analyzer.sni_counts().get("пример.example").unwrap_or(&0),
        1
    );

    // Per ADR 0003: the data layer stores raw bytes (not Debug-escaped).
    // Find the non-ASCII finding and assert it contains the raw Cyrillic
    // hostname — this directly guards the {hostname:?} → {hostname} rollback
    // on the NonAsciiUtf8 match arm (src/analyzer/tls.rs), which was
    // otherwise only covered structurally by the NonUtf8 branch test.
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-ASCII characters"))
        .expect("expected non-ASCII finding");
    assert!(
        f.summary.contains("пример.example"),
        "summary must contain raw Cyrillic hostname for forensic preservation, got: {}",
        f.summary
    );
    assert!(
        !f.summary.contains("\\u{43f}"),
        "summary must not contain Debug-formatted Cyrillic escape (regression to construction-site), got: {}",
        f.summary
    );
}

#[test]
fn test_emoji_sni_emits_non_ascii_finding() {
    // Regression: a 4-byte UTF-8 emoji codepoint also trips the finding.
    // 🦀 = U+1F980 = 0xF0 0x9F 0xA6 0x80 (4 bytes, all ≥ 0x80).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("🦀.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let non_ascii_count = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .count();
    assert_eq!(
        non_ascii_count,
        1,
        "emoji SNI must emit one non-ASCII finding, got {:?}",
        analyzer.findings()
    );
}

#[test]
fn test_punycode_a_label_does_not_emit_non_ascii_finding() {
    // Regression: a Punycode A-label "xn--caf-dma.example" is the
    // RFC-compliant way to encode "café.example" and is pure ASCII.
    // The analyzer must NOT flag it.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("xn--caf-dma.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    let non_ascii_count = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .count();
    assert_eq!(
        non_ascii_count, 0,
        "Punycode A-label is RFC-compliant ASCII and must not be flagged"
    );
    assert_eq!(
        *analyzer
            .sni_counts()
            .get("xn--caf-dma.example")
            .unwrap_or(&0),
        1
    );
}

#[test]
fn test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity() {
    // Pin: when sni_counts is at MAX_MAP_ENTRIES (50,000), the increment for a
    // new key is silently dropped, but the non-UTF-8 finding must STILL fire.
    // The finding push and the count increment are independent — a refactor
    // that nests one inside the other would silently break this invariant
    // and forensic visibility into anomalous SNI in long-running captures
    // would degrade past the cap.
    //
    // This test is intentionally slower than the others (~650ms in debug builds,
    // measured locally; budget ~2s for CI cold caches) because the only way to
    // reach the capacity-full state via the public API is to feed 50k unique
    // ClientHellos through it. The performance trade-off was discussed in the
    // design phase: the alternative was adding a #[cfg(test)] test helper to
    // TlsAnalyzer, which would expose internal state. Black-box brute force
    // keeps the public API clean.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // MAX_MAP_ENTRIES is private; this constant must stay in sync.
    const MAX_MAP_ENTRIES: usize = 50_000;

    // Fill sni_counts to capacity with unique valid-UTF-8 hostnames. Each
    // ClientHello uses the same cipher list, so all 50k JA3 hashes collapse
    // into a single ja3_counts entry — only sni_counts grows. The single
    // per-flow state handles all 50k records in sequence via buffer draining;
    // client_hello_seen stays true after the first iteration but that's
    // harmless because only `done()` (requiring both hellos) short-circuits
    // on_data, and we never send a ServerHello.
    for i in 0..MAX_MAP_ENTRIES {
        let sni = format!("filler{i:05}.example");
        let record = build_client_hello(&sni, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);
    }

    assert_eq!(
        analyzer.sni_counts().len(),
        MAX_MAP_ENTRIES,
        "expected sni_counts to be full at MAX_MAP_ENTRIES, got {}",
        analyzer.sni_counts().len()
    );

    // Snapshot the non-UTF-8 finding count before the capacity-test ClientHello
    // so we can assert exactly one new non-UTF-8 finding fired. Filtering by
    // category (not total length) keeps the assertion robust: a future refactor
    // that adds an unrelated finding on every ClientHello wouldn't make this
    // test pass vacuously.
    let non_utf8_before = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();

    // Send one more ClientHello with a non-UTF-8 SNI. Its key cannot fit in
    // sni_counts (the map is full and the key is new), but the finding push
    // must still happen.
    let raw_sni: &[u8] = &[0xff, 0xfe, b'a', b'.', b'c', b'o', b'm'];
    let record = build_client_hello_raw_sni(raw_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // sni_counts should be unchanged in size — the new non-UTF-8 key was dropped.
    assert_eq!(
        analyzer.sni_counts().len(),
        MAX_MAP_ENTRIES,
        "non-UTF-8 SNI must not have been inserted past the cap"
    );
    assert!(
        analyzer
            .sni_counts()
            .get("<non-utf8:fffe612e636f6d>")
            .is_none(),
        "non-UTF-8 hex key must not be present in sni_counts past the cap"
    );

    // The non-UTF-8 finding must still have fired — this is the real invariant
    // the test guards: finding emission is independent of count insertion.
    let non_utf8_after = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(
        non_utf8_after,
        non_utf8_before + 1,
        "expected exactly one new non-UTF-8 finding past the cap, \
         got {non_utf8_before} -> {non_utf8_after}"
    );
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("expected non-UTF-8 SNI finding even when sni_counts is full");
    assert_eq!(f.confidence, wirerust::findings::Confidence::Low);
}

#[test]
fn test_multi_name_sni_list_only_first_entry_counted() {
    // Pin: when an SNI extension contains multiple ServerName entries,
    // the analyzer reads only the first one. This matches the prior behavior
    // (extract_sni uses `list.first()`) and matches what nearly all real-world
    // TLS clients do — the spec allows multiple but no major client emits >1.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_with_sni_list(
        &[b"first.example", b"second.example", b"third.example"],
        &[0x1301],
    );
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(
        *analyzer.sni_counts().get("first.example").unwrap_or(&0),
        1,
        "expected first hostname to be counted, got {:?}",
        analyzer.sni_counts()
    );
    assert!(
        analyzer.sni_counts().get("second.example").is_none(),
        "second hostname must not be counted, got {:?}",
        analyzer.sni_counts()
    );
    assert!(
        analyzer.sni_counts().get("third.example").is_none(),
        "third hostname must not be counted, got {:?}",
        analyzer.sni_counts()
    );
}
