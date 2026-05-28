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
    let typed: Vec<(u8, &[u8])> = sni_entries.iter().map(|e| (0x00u8, *e)).collect();
    build_client_hello_with_typed_sni_list(&typed, cipher_ids)
}

/// Build a minimal TLS ClientHello with SNI entries where each entry has an explicit
/// NameType byte. Used for testing non-zero NameType values (issue #52 case 1).
///
/// Each tuple is `(name_type, name_bytes)`. For `host_name(0)` the bytes are an
/// ASCII hostname per RFC 6066 §3; for other NameType values the bytes are the
/// opaque ServerName payload for that type. The `(255)` in the RFC enum reserves
/// slots for future types — a non-zero NameType exercises tls_parser's handling
/// of unknown ServerName types.
fn build_client_hello_with_typed_sni_list(
    sni_entries: &[(u8, &[u8])],
    cipher_ids: &[u16],
) -> Vec<u8> {
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000)
    // The ServerNameList contents: for each entry, 1 byte NameType + 2 byte name length + name bytes.
    // Use checked conversions so a future test passing an out-of-range SNI (e.g. a
    // ~65,532-byte single hostname — u16::MAX minus 3 bytes of ServerName overhead
    // per entry — requested in issue #52) panics with a clear message rather than
    // silently wrapping through `as u16` and producing a malformed ClientHello.
    let mut sni_list_data = Vec::new();
    for (name_type, entry) in sni_entries {
        let name_len = u16::try_from(entry.len())
            .expect("SNI entry payload exceeds u16::MAX; can't encode as ServerName");
        sni_list_data.push(*name_type);
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

// ---------------------------------------------------------------------------
// Issue #52: SNI edge-case coverage for non-zero NameType, large SNI,
// and trailing bytes in ServerNameList
// ---------------------------------------------------------------------------

/// Build a ClientHello with a raw, hand-crafted SNI extension (arbitrary bytes).
/// Used for tests that need malformed or non-standard SNI framing that the
/// structured builders can't produce (e.g., trailing bytes in ServerNameList).
fn build_client_hello_with_raw_sni_ext(raw_sni_ext_data: &[u8], cipher_ids: &[u16]) -> Vec<u8> {
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000) with caller-supplied raw data
    let ext_data_len =
        u16::try_from(raw_sni_ext_data.len()).expect("raw SNI ext data exceeds u16::MAX");
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&ext_data_len.to_be_bytes());
    extensions.extend_from_slice(raw_sni_ext_data);

    // Supported Groups extension (type 0x000a)
    extensions.extend_from_slice(&[0x00, 0x0a, 0x00, 0x06, 0x00, 0x04, 0x00, 0x1d, 0x00, 0x17]);

    // EC Point Formats extension (type 0x000b)
    extensions.extend_from_slice(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);

    // Build ClientHello body
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0

    let ciphers_byte_len = cipher_ids
        .len()
        .checked_mul(2)
        .expect("cipher_ids list byte length overflows usize");
    let ciphers_len =
        u16::try_from(ciphers_byte_len).expect("cipher_ids list exceeds u16::MAX bytes");
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression

    let ext_len =
        u16::try_from(extensions.len()).expect("ClientHello extensions block exceeds u16::MAX");
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
    let hs_len = u16::try_from(handshake.len()).expect("handshake body exceeds u16::MAX bytes");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);

    record
}

#[test]
fn test_non_zero_name_type_sni_entry() {
    // RFC 6066 §3 defines NameType { host_name(0), (255) }. A non-zero
    // NameType is reserved for future use. This test pins how tls_parser
    // and extract_sni handle an entry with NameType=1 (unknown future type).
    //
    // extract_sni destructures as `let Some((_, hostname)) = list.first()`
    // — it ignores the type field. So if tls_parser includes the entry in
    // the SNI list, it will be treated as a hostname regardless of type.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // NameType=1 with hostname "future.example"
    let record = build_client_hello_with_typed_sni_list(&[(0x01, b"future.example")], &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // Pinned behavior (empirically verified): tls_parser includes non-zero
    // NameType entries in the SNI list, and extract_sni reads the first entry
    // regardless of type (it destructures as `(_, hostname)`, ignoring the type).
    // If a tls_parser upgrade changes this to skip unknown types, this test
    // will fail — documenting the behavioral change.
    assert_eq!(
        *analyzer.sni_counts().get("future.example").unwrap_or(&0),
        1,
        "tls_parser should include non-zero NameType entry in SNI list"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
    assert_eq!(analyzer.handshake_count(), 1);
}

#[test]
fn test_non_zero_name_type_with_valid_first_entry() {
    // Variant: first entry is host_name(0) with a valid hostname, second
    // entry is an unknown NameType=0x02. extract_sni should read only the
    // first entry (host_name) regardless.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_with_typed_sni_list(
        &[(0x00, b"real.example"), (0x02, b"unknown-type")],
        &[0x1301],
    );
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(
        *analyzer.sni_counts().get("real.example").unwrap_or(&0),
        1,
        "first entry (host_name type) must be counted"
    );
    assert!(
        analyzer.sni_counts().get("unknown-type").is_none(),
        "second entry (unknown type) must not be counted"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
}

#[test]
fn test_large_sni_near_record_payload_limit() {
    // A large SNI hostname that fits within MAX_RECORD_PAYLOAD (18,432 bytes).
    // The ClientHello overhead is ~74 bytes, so a 16,000-byte hostname produces
    // a record payload of ~16,074 — well within the limit.
    //
    // This pins size-handling behavior only — the analyzer does not validate
    // DNS label/hostname syntax, so a 16KB string of "A" (not a legal DNS name)
    // is a fine fixture for exercising the record-layer and SNI-parsing paths.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let large_hostname = "A".repeat(16_000);
    let record = build_client_hello(&large_hostname, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(
        *analyzer
            .sni_counts()
            .get(large_hostname.as_str())
            .unwrap_or(&0),
        1,
        "16KB SNI should parse and be counted"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
    assert_eq!(analyzer.handshake_count(), 1);
}

#[test]
fn test_oversized_sni_exceeds_record_payload_limit() {
    // A SNI so large that the TLS record payload exceeds MAX_RECORD_PAYLOAD.
    // The analyzer should reject the record at the record-layer boundary and
    // increment parse_errors. The SNI is never reached.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // MAX_RECORD_PAYLOAD is private in src/analyzer/tls.rs; this constant
    // must stay in sync with the production value (18,432 = TLS 1.2 ciphertext
    // max per RFC 5246).
    const MAX_RECORD_PAYLOAD: usize = 18_432;

    // 18,400 byte hostname + ~74 bytes overhead = ~18,474 > 18,432 limit.
    // Verify the record payload actually exceeds the cap before asserting
    // rejection — catches fixture drift if the builder structure changes.
    let huge_hostname = "B".repeat(18_400);
    let record = build_client_hello(&huge_hostname, &[0x1301]);
    assert!(
        record.len() >= 5,
        "fixture must produce a complete TLS record header"
    );
    let payload_len = u16::from_be_bytes([record[3], record[4]]) as usize;
    assert!(
        payload_len > MAX_RECORD_PAYLOAD,
        "test precondition: record payload must exceed MAX_RECORD_PAYLOAD (got {payload_len})"
    );

    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert!(
        analyzer.sni_counts().is_empty(),
        "oversized record should be rejected before SNI parsing"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "oversized record should count as a parse error"
    );
    assert_eq!(
        analyzer.handshake_count(),
        0,
        "no handshake should be counted from a rejected record"
    );
}

#[test]
fn test_trailing_bytes_in_server_name_list() {
    // If the ServerNameList's outer length field claims more bytes than the
    // sum of its ServerName entries, there are "trailing bytes". This test
    // pins tls_parser's behavior: does it accept or reject the malformed
    // framing? And if it accepts, does extract_sni still read the first
    // entry correctly?
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Build raw SNI extension data:
    //   sni_list_len = actual_entries_len + 4 (lying — 4 extra bytes)
    //   entry: NameType=0x00, name_len=12, "test.example"
    let hostname = b"test.example";
    let name_len =
        u16::try_from(hostname.len()).expect("hostname length must fit in TLS u16 field");
    let mut sni_list_data = Vec::new();
    sni_list_data.push(0x00); // NameType = host_name
    sni_list_data.extend_from_slice(&name_len.to_be_bytes());
    sni_list_data.extend_from_slice(hostname);

    // Lie about list length: claim 4 extra bytes of trailing garbage
    let lying_list_len =
        u16::try_from(sni_list_data.len() + 4).expect("lying list length must fit in u16");
    let mut raw_ext_data = Vec::new();
    raw_ext_data.extend_from_slice(&lying_list_len.to_be_bytes());
    raw_ext_data.extend_from_slice(&sni_list_data);
    raw_ext_data.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // trailing garbage

    let record = build_client_hello_with_raw_sni_ext(&raw_ext_data, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    // Pinned behavior (empirically verified): tls_parser accepts the malformed
    // framing — the trailing garbage bytes are consumed by the length field but
    // don't interfere with parsing the first ServerName entry. If a tls_parser
    // upgrade tightens validation to reject trailing bytes, this test will fail.
    assert_eq!(
        *analyzer.sni_counts().get("test.example").unwrap_or(&0),
        1,
        "first SNI entry should be readable despite trailing bytes in ServerNameList"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
    assert_eq!(analyzer.handshake_count(), 1);
}

// ── issue #54: SNI containing ASCII control bytes (C0 / DEL) ─────────────────
//
// RFC 6066 §3 requires HostName to be ASCII; the DNS preferred hostname syntax
// (RFC 952 / RFC 1123, inherited by RFC 5890 A-label construction) restricts
// to letters, digits, and hyphens. No conforming TLS client should emit C0
// (0x00–0x1F) or DEL (0x7F) in SNI, so the full C0+DEL range is the correct
// detection scope
// (narrower ranges like ESC+DEL alone would miss BEL / CR / LF / tab variants
// used for log-injection, terminal escape, or covert signalling).
//
// Tests below pin each control-byte case to the `AsciiWithControl` variant:
// finding fires with Anomaly / Inconclusive / Low (matching other SNI anomalies),
// sni_counts stores the *raw* hostname (per ADR 0003 — reporter escapes for
// terminal display), and mutually exclusive with the NonAsciiUtf8 path.

/// Helper: build a ClientHello with an SNI containing arbitrary ASCII bytes.
/// Wraps `build_client_hello_raw_sni` — the latter already accepts `&[u8]`,
/// but this name documents intent at the call site.
fn build_client_hello_ascii_bytes(sni_bytes: &[u8], cipher_ids: &[u16]) -> Vec<u8> {
    build_client_hello_raw_sni(sni_bytes, cipher_ids)
}

/// Helper: count findings whose summary mentions the ASCII control-byte anomaly.
fn count_control_findings(analyzer: &TlsAnalyzer) -> usize {
    analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .count()
}

#[test]
fn test_ascii_sni_with_esc_emits_control_finding_and_counts_under_raw_key() {
    // Primary test: a classic ANSI escape sequence embedded in the SNI. This
    // is the terminal-injection shape the ADR 0003 pipeline is designed to
    // contain — the analyzer preserves the raw bytes, the reporter escapes
    // them at render. Here we assert only the analyzer-layer contract.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // "foo\x1b[31m.example" — ESC + CSI 31m (red) sequence.
    let sni: &[u8] = b"foo\x1b[31m.example";
    let record = build_client_hello_ascii_bytes(sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert_eq!(analyzer.handshake_count(), 1);

    // Exactly one control-byte finding.
    assert_eq!(
        count_control_findings(&analyzer),
        1,
        "expected exactly one ASCII control finding, got: {:?}",
        analyzer.findings()
    );
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .expect("expected control finding");
    assert_eq!(f.category, wirerust::findings::ThreatCategory::Anomaly);
    assert_eq!(f.verdict, wirerust::findings::Verdict::Inconclusive);
    assert_eq!(f.confidence, wirerust::findings::Confidence::Low);
    assert!(
        f.summary.contains("RFC 6066"),
        // {:?} uses Debug formatting which escapes control bytes — prevents
        // a failing assertion from corrupting CI logs with the raw ESC / ANSI
        // sequence that the test deliberately constructs.
        "summary should cite RFC 6066, got: {:?}",
        f.summary
    );
    // Per ADR 0003: analyzer finding summary stores the raw hostname
    // (including the ESC byte). Terminal escaping is the reporter's job.
    assert!(
        f.summary.as_bytes().contains(&0x1b),
        "summary must preserve raw ESC byte per ADR 0003, got hex: {:?}",
        f.summary.as_bytes()
    );
    // Hex evidence is the lossless byte-form of the raw SNI.
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("666f6f1b5b33316d2e6578616d706c65")),
        "expected hex evidence to contain raw byte sequence, got: {:?}",
        f.evidence
    );

    // sni_counts stores the raw hostname string (matches the NonAsciiUtf8
    // precedent — distinct ASCII-with-control strings produce distinct keys
    // deterministically, no <ctrl:HEX> tagging is needed).
    assert_eq!(
        *analyzer
            .sni_counts()
            .get(std::str::from_utf8(sni).unwrap())
            .unwrap_or(&0),
        1,
        "sni_counts must key on raw hostname, got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );
}

#[test]
fn test_ascii_sni_with_bel_emits_control_finding() {
    // BEL (0x07) is the canonical "audible" control — not a terminal-escape
    // vector per se, but still a protocol violation and a plausible covert
    // signal.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_ascii_bytes(b"ring\x07bell.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(count_control_findings(&analyzer), 1);
    assert_eq!(
        *analyzer
            .sni_counts()
            .get("ring\x07bell.example")
            .unwrap_or(&0),
        1
    );
}

#[test]
fn test_ascii_sni_with_del_emits_control_finding() {
    // DEL (0x7F) sits at the very top of the 7-bit ASCII range. `char::is_ascii()`
    // returns true — which is why the pre-fix analyzer silently accepted it.
    // DEL is non-printable and non-LDH.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_ascii_bytes(b"host\x7fname.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(count_control_findings(&analyzer), 1);
}

#[test]
fn test_ascii_sni_with_tab_emits_control_finding() {
    // Tab (0x09) is C0 whitespace. Issue #54 flagged tab/CR/LF as an open
    // question; RFC 6066 §3 + DNS preferred-hostname syntax (RFC 952/1123)
    // disallow whitespace in SNI. Include in detection range.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_ascii_bytes(b"left\tright.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(count_control_findings(&analyzer), 1);
}

#[test]
fn test_ascii_sni_with_cr_and_lf_emits_control_finding() {
    // CR (0x0D) and LF (0x0A) are the classic log-injection vector — a
    // logger that prints the SNI unescaped would see spurious new log lines.
    // Both are C0 and must trip the finding.

    // CR alone.
    let mut a1 = TlsAnalyzer::new();
    let fk = test_flow_key();
    a1.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_ascii_bytes(b"a\rb.example", &[0x1301]),
        0,
    );
    assert_eq!(count_control_findings(&a1), 1, "CR must trip finding");

    // LF alone.
    let mut a2 = TlsAnalyzer::new();
    a2.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_ascii_bytes(b"a\nb.example", &[0x1301]),
        0,
    );
    assert_eq!(count_control_findings(&a2), 1, "LF must trip finding");

    // CRLF combined.
    let mut a3 = TlsAnalyzer::new();
    a3.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_ascii_bytes(b"a\r\nb.example", &[0x1301]),
        0,
    );
    assert_eq!(count_control_findings(&a3), 1, "CRLF must trip finding");
}

#[test]
fn test_printable_ascii_sni_emits_no_control_finding() {
    // Regression guard: a plain printable-ASCII hostname must not trigger
    // the new finding. A normal hostname keeps the intent clear.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("www.example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(count_control_findings(&analyzer), 0);
    assert_eq!(
        analyzer
            .findings()
            .iter()
            .filter(|f| f.summary.contains("SNI"))
            .count(),
        0,
        "printable ASCII must produce no SNI findings at all"
    );
}

#[test]
fn test_punycode_a_label_emits_no_control_finding() {
    // Regression guard: `xn--caf-dma.example` is the RFC-compliant A-label
    // encoding of `café.example`. It's pure printable ASCII and must NOT
    // trigger either the non-ASCII finding (covered elsewhere) or the new
    // control-byte finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("xn--caf-dma.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(count_control_findings(&analyzer), 0);
}

#[test]
fn test_non_ascii_sni_does_not_emit_control_finding() {
    // Mutual exclusivity: a non-ASCII UTF-8 SNI (Cyrillic) falls into the
    // `NonAsciiUtf8` branch, NOT `AsciiWithControl`. The `s.is_ascii()`
    // guard on both control-byte arms prevents the non-ASCII path from
    // ever landing in the new variant.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("пример.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(
        count_control_findings(&analyzer),
        0,
        "non-ASCII SNI must land in NonAsciiUtf8, not AsciiWithControl"
    );
    // And the non-ASCII finding still fires.
    assert_eq!(
        analyzer
            .findings()
            .iter()
            .filter(|f| f.summary.contains("non-ASCII characters"))
            .count(),
        1
    );
}

#[test]
fn test_ascii_control_boundary_bytes() {
    // Boundary pins on both ends of the C0+DEL detection range:
    //   - 0x00 (NUL, start of C0) MUST trip — most dangerous injection byte
    //     (C-string truncation, log-parser splitting).
    //   - 0x1F (Unit Separator, end of C0) MUST trip.
    //   - 0x20 (space, first printable) MUST NOT trip — space is disallowed
    //     by DNS preferred hostname syntax (RFC 952/1123), but it's not a
    //     control byte, so this issue deliberately leaves it alone. Broader
    //     LDH compliance is out-of-scope per issue #54's "Out of scope".
    //   - 0x7F (DEL) is the other non-C0 half of the detection range and is
    //     covered separately in `test_ascii_sni_with_del_emits_control_finding`.
    let fk = test_flow_key();

    for (label, byte, expect_trip) in [
        ("0x00 (NUL, start of C0)", 0x00u8, true),
        ("0x1F (end of C0)", 0x1fu8, true),
        ("0x20 (space, first printable)", 0x20u8, false),
    ] {
        let sni = vec![
            b'a', byte, b'b', b'.', b'e', b'x', b'a', b'm', b'p', b'l', b'e',
        ];
        let mut analyzer = TlsAnalyzer::new();
        analyzer.on_data(
            &fk,
            Direction::ClientToServer,
            &build_client_hello_ascii_bytes(&sni, &[0x1301]),
            0,
        );
        let expected = if expect_trip { 1 } else { 0 };
        assert_eq!(
            count_control_findings(&analyzer),
            expected,
            "{label}: expected {expected} control finding(s)",
        );
    }
}

#[test]
fn test_multiple_control_bytes_in_sni_produces_single_finding() {
    // If multiple C0/DEL bytes appear in the same SNI, the analyzer emits
    // ONE finding for the hostname (not one per byte). Evidence still
    // captures the full raw bytes via the hex field.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_ascii_bytes(b"a\x07b\x1bc\x7fd.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

    assert_eq!(
        count_control_findings(&analyzer),
        1,
        "multiple control bytes in one SNI must produce one finding, not N"
    );
    // Hex evidence captures all three bytes losslessly.
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .unwrap();
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("07") && e.contains("1b") && e.contains("7f")),
        "hex evidence must preserve all control bytes, got: {:?}",
        f.evidence
    );
}

#[test]
fn ascii_control_sni_finding_sets_mitre_t1027() {
    let esc_hostname = b"foo\x1bbar.example.com";
    let bytes = build_client_hello_ascii_bytes(esc_hostname, &[]);
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&test_flow_key(), Direction::ClientToServer, &bytes, 0);

    let findings = analyzer.findings();
    let control_finding = findings
        .iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .expect("expected an ASCII-control SNI finding");
    assert_eq!(
        control_finding.mitre_technique.as_deref(),
        Some("T1027"),
        "malformed-SNI finding must be mapped to T1027 (Obfuscated Files or Information)",
    );
}

#[test]
fn non_ascii_utf8_sni_finding_sets_mitre_t1027() {
    let bytes = build_client_hello("пример.рф", &[]);
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&test_flow_key(), Direction::ClientToServer, &bytes, 0);

    let findings = analyzer.findings();
    let finding = findings
        .iter()
        .find(|f| f.summary.contains("non-ASCII characters"))
        .expect("expected a non-ASCII SNI finding");
    assert_eq!(
        finding.mitre_technique.as_deref(),
        Some("T1027"),
        "malformed-SNI finding must be mapped to T1027 (Obfuscated Files or Information)",
    );
}

#[test]
fn non_utf8_sni_finding_sets_mitre_t1027() {
    let bytes = build_client_hello_raw_sni(&[b'f', b'o', b'o', 0xc3, b'.', b'c', b'o', b'm'], &[]);
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&test_flow_key(), Direction::ClientToServer, &bytes, 0);

    let findings = analyzer.findings();
    let finding = findings
        .iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("expected a non-UTF-8 SNI finding");
    assert_eq!(
        finding.mitre_technique.as_deref(),
        Some("T1027"),
        "malformed-SNI finding must be mapped to T1027 (Obfuscated Files or Information)",
    );
}

// ── BC-2.07.006 / BC-2.07.007 / BC-2.07.008 STORY-051 formalization tests ────
//
// These tests verify the JA3 and JA3S fingerprint computation contracts through
// the public TlsAnalyzer API (feeding wire-format TLS records). All private
// compute_ja3 / compute_ja3s / is_grease_u16 functions are indirectly exercised.
// Algorithm-level property tests exist in src/analyzer/tls.rs::ja3_property_tests;
// these integration-level tests complete the BC traceability in this file.
//
// Naming follows DF-AC-TEST-NAME-SYNC-001 v1: test_BC_2_07_NNN_<suffix>.
// #[allow(non_snake_case)] is applied per-test so BC-prefixed names compile
// under RUSTFLAGS=-Dwarnings without suppressing the lint on existing tests.

/// Build a minimal TLS ClientHello with no extensions at all.
///
/// Used by JA3 string-format tests that need a predictable JA3 string
/// without SNI / SupportedGroups / ECPointFormats padding from the standard
/// `build_client_hello` helper.
fn build_client_hello_no_extensions(cipher_ids: &[u16]) -> Vec<u8> {
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2 (0x0303 = 771)
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0

    let ciphers_len = u16::try_from(cipher_ids.len() * 2).expect("cipher list too long");
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression
    // No extensions length field at all — ch.ext will be None.

    let mut handshake = Vec::new();
    handshake.push(0x01); // ClientHello
    let ch_len = ch_body.len() as u32;
    handshake.push((ch_len >> 16) as u8);
    handshake.push((ch_len >> 8) as u8);
    handshake.push(ch_len as u8);
    handshake.extend_from_slice(&ch_body);

    let mut record = Vec::new();
    record.push(0x16);
    record.extend_from_slice(&[0x03, 0x01]);
    let hs_len = u16::try_from(handshake.len()).expect("handshake too long");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

/// Build a minimal ServerHello that includes a GREASE extension (type 0x0a0a)
/// plus the standard renegotiation_info (0xff01).
///
/// Used for AC-010: verifying that GREASE extension IDs are filtered from the
/// JA3S extension field while the cipher (even if it is a GREASE value) is NOT
/// filtered.
fn build_server_hello_with_grease_ext(cipher_id: u16) -> Vec<u8> {
    let mut extensions = Vec::new();

    // GREASE extension (type 0x0a0a) with empty data — must be filtered in JA3S.
    extensions.extend_from_slice(&[0x0a, 0x0a]); // GREASE ext type
    extensions.extend_from_slice(&[0x00, 0x01]); // data length = 1
    extensions.push(0x00); // payload

    // renegotiation_info (type 0xff01) — NOT GREASE, appears in ext_ids.
    extensions.extend_from_slice(&[0xff, 0x01]);
    extensions.extend_from_slice(&[0x00, 0x01]);
    extensions.push(0x00);

    let mut sh_body = Vec::new();
    sh_body.extend_from_slice(&[0x03, 0x03]); // TLS 1.2
    sh_body.extend_from_slice(&[0u8; 32]); // random
    sh_body.push(0x00); // session_id length
    sh_body.extend_from_slice(&cipher_id.to_be_bytes());
    sh_body.push(0x00); // compression: null

    let ext_len = u16::try_from(extensions.len()).expect("ext too long");
    sh_body.extend_from_slice(&ext_len.to_be_bytes());
    sh_body.extend_from_slice(&extensions);

    let mut handshake = Vec::new();
    handshake.push(0x02); // ServerHello
    let sh_len = sh_body.len() as u32;
    handshake.push((sh_len >> 16) as u8);
    handshake.push((sh_len >> 8) as u8);
    handshake.push(sh_len as u8);
    handshake.extend_from_slice(&sh_body);

    let mut record = Vec::new();
    record.push(0x16);
    record.extend_from_slice(&[0x03, 0x03]);
    let hs_len = u16::try_from(handshake.len()).expect("handshake too long");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

// ── AC-001 (BC-2.07.006 postconditions 1-2): GREASE bitmask exclusion ─────────
//
// A ClientHello with cipher list [0x0a0a, 0x002f] produces the same JA3 hash
// as a ClientHello with cipher list [0x002f] only. Uses no-extension builder so
// the JA3 string is deterministic without SNI/curves/pf padding.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_006_grease_cipher_excluded_same_hash_as_without_grease() {
    // Canonical test vector from BC-2.07.006:
    // Cipher list [0x0a0a, 0x002f] -> JA3 same as [0x002f] only.
    // Expected JA3 string: "771,47,,," -> MD5 = fde4273625b2ac63bd01d9c500dac91b

    let fk = test_flow_key();

    // Hash for [GREASE + 0x002f]
    let mut a1 = TlsAnalyzer::new();
    a1.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x0a0a, 0x002f]),
        0,
    );
    let hash_with_grease = a1.ja3_counts().keys().next().unwrap().clone();

    // Hash for [0x002f only]
    let mut a2 = TlsAnalyzer::new();
    a2.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x002f]),
        0,
    );
    let hash_without_grease = a2.ja3_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash_with_grease, hash_without_grease,
        "JA3 hash with GREASE cipher 0x0a0a + 0x002f must equal JA3 hash with 0x002f only \
         (BC-2.07.006 postcondition 2)"
    );
    // BC canonical test vector: MD5("771,47,,,") = fde4273625b2ac63bd01d9c500dac91b
    assert_eq!(
        hash_with_grease, "fde4273625b2ac63bd01d9c500dac91b",
        "BC-2.07.006 canonical vector: JA3 of cipher [0x002f] with no extensions must be \
         MD5('771,47,,,') = fde4273625b2ac63bd01d9c500dac91b"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_006_all_grease_cipher_list_produces_empty_cipher_field() {
    // EC-001 from BC-2.07.006: cipher list [0x0a0a] only -> cipher field is ""
    // JA3 string = "771,,," -> MD5 = bddda940f9963577c41d7c28b1a5f65f
    let fk = test_flow_key();

    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x0a0a]),
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();

    // BC canonical vector: MD5("771,,,,") = bddda940f9963577c41d7c28b1a5f65f
    assert_eq!(
        hash, "bddda940f9963577c41d7c28b1a5f65f",
        "BC-2.07.006 EC-001: all-GREASE cipher list must produce JA3 = MD5('771,,,,') \
         = bddda940f9963577c41d7c28b1a5f65f"
    );
}

// ── AC-002 (BC-2.07.006 postcondition 3, invariant 3): GREASE invariance ──────
//
// Non-GREASE values are preserved in original order. Inserting GREASE at any
// position does not change the hash. Proptest coverage is provided in the inline
// ja3_property_tests::compute_ja3_is_grease_invariant in src/analyzer/tls.rs.
// This test provides a deterministic AC-binding integration pin.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_006_grease_inserted_at_front_middle_end_same_hash() {
    // Non-GREASE ciphers [0x002f, 0x0035] in order, with 0x1a1a (canonical GREASE)
    // inserted at front / middle / end: all three must produce the same hash.
    // Expected JA3 string (no-ext builder): "771,47-53,,,"
    // MD5("771,47-53,,,") = 577fbfd57b256f5467f2fe09d1505a26
    let fk = test_flow_key();

    let cases: &[&[u16]] = &[
        &[0x1a1a, 0x002f, 0x0035], // GREASE at front
        &[0x002f, 0x1a1a, 0x0035], // GREASE at middle
        &[0x002f, 0x0035, 0x1a1a], // GREASE at end
        &[0x002f, 0x0035],         // no GREASE (baseline)
    ];

    let hashes: Vec<String> = cases
        .iter()
        .map(|&cipher_ids| {
            let mut a = TlsAnalyzer::new();
            a.on_data(
                &fk,
                Direction::ClientToServer,
                &build_client_hello_no_extensions(cipher_ids),
                0,
            );
            a.ja3_counts().keys().next().unwrap().clone()
        })
        .collect();

    let baseline = &hashes[3]; // [0x002f, 0x0035] no GREASE
    for (i, h) in hashes.iter().enumerate().take(3) {
        assert_eq!(
            h, baseline,
            "case {i}: inserting GREASE cipher 0x1a1a must not change JA3 hash \
             (BC-2.07.006 invariant 3)"
        );
    }
    assert_eq!(
        baseline, "577fbfd57b256f5467f2fe09d1105a26",
        "BC-2.07.007 canonical vector: JA3 of [0x002f, 0x0035] no ext must be \
         MD5('771,47-53,,,') = 577fbfd57b256f5467f2fe09d1105a26"
    );
}

// ── AC-003 (BC-2.07.006 invariant 1): bitmask applies to ciphers and extensions,
//           NOT to EC point format bytes ─────────────────────────────────────────
//
// is_grease_u16 applies to cipher IDs, extension type IDs, and named group IDs.
// EC point format bytes are u8 values and are NOT filtered (they are included as-is).
// We verify this by observing JA3 hashes through the wire-format API.
// The 16 canonical GREASE values are confirmed by the inline unit test
// is_grease_u16_matches_all_canonical_grease_values in src/analyzer/tls.rs.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_006_all_16_canonical_grease_ciphers_produce_empty_cipher_field() {
    // EC-003: all 16 canonical RFC 8701 GREASE values in cipher list -> same JA3
    // as empty cipher list. Verifies is_grease_u16 is applied to cipher IDs.
    // BC canonical: MD5("771,,,,") = bddda940f9963577c41d7c28b1a5f65f
    let all_grease: &[u16] = &[
        0x0a0a, 0x1a1a, 0x2a2a, 0x3a3a, 0x4a4a, 0x5a5a, 0x6a6a, 0x7a7a, 0x8a8a, 0x9a9a, 0xaaaa,
        0xbaba, 0xcaca, 0xdada, 0xeaea, 0xfafa,
    ];
    let fk = test_flow_key();

    let mut a_grease = TlsAnalyzer::new();
    a_grease.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(all_grease),
        0,
    );
    let hash_all_grease = a_grease.ja3_counts().keys().next().unwrap().clone();

    // No ciphers at all should produce the same hash as all-GREASE.
    let mut a_empty = TlsAnalyzer::new();
    a_empty.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[]),
        0,
    );
    let hash_empty = a_empty.ja3_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash_all_grease, hash_empty,
        "all 16 canonical GREASE cipher values must produce same JA3 as empty cipher list \
         (BC-2.07.006 EC-003)"
    );
    assert_eq!(
        hash_all_grease, "bddda940f9963577c41d7c28b1a5f65f",
        "BC-2.07.006 EC-003: JA3 with all GREASE ciphers must be MD5('771,,,,') \
         = bddda940f9963577c41d7c28b1a5f65f"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_006_non_canonical_grease_pattern_0x0a1a_is_filtered() {
    // EC-002: non-canonical 0x0a1a passes the bitmask (0x0a1a & 0x0F0F == 0x0a0a)
    // and is filtered identically to canonical GREASE 0x0a0a.
    // JA3 of [0x0a1a] only == JA3 of [] (cipher field empty).
    let fk = test_flow_key();

    let mut a_noncanon = TlsAnalyzer::new();
    a_noncanon.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x0a1a]),
        0,
    );
    let hash_noncanon = a_noncanon.ja3_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash_noncanon, "bddda940f9963577c41d7c28b1a5f65f",
        "non-canonical GREASE 0x0a1a must be filtered like canonical GREASE \
         (BC-2.07.006 EC-002): expected MD5('771,,,,') = bddda940f9963577c41d7c28b1a5f65f"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_006_ec_point_format_bytes_are_not_filtered() {
    // BC-2.07.006 invariant 1: EC point format bytes are NOT filtered.
    // The standard build_client_hello includes an ECPointFormats extension with
    // byte [0x00] (uncompressed). This byte is included in the JA3 point-format
    // field verbatim — not compared against is_grease_u16.
    //
    // Verify: JA3 with EC point format byte 0x00 is DIFFERENT from JA3 without
    // that extension, confirming point format bytes are preserved (not filtered).
    let fk = test_flow_key();

    // With ECPointFormats extension (pf_str = "0")
    let mut a_with_pf = TlsAnalyzer::new();
    a_with_pf.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("test.com", &[0x002f]), // includes SNI + curves + pf
        0,
    );
    let hash_with_pf = a_with_pf.ja3_counts().keys().next().unwrap().clone();

    // Without any extensions (no pf)
    let mut a_no_ext = TlsAnalyzer::new();
    a_no_ext.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x002f]),
        0,
    );
    let hash_no_ext = a_no_ext.ja3_counts().keys().next().unwrap().clone();

    assert_ne!(
        hash_with_pf, hash_no_ext,
        "JA3 with ECPointFormats (pf_str='0') must differ from JA3 without, \
         confirming point format bytes are included in the hash (BC-2.07.006 invariant 1)"
    );

    // The no-extension hash must equal the canonical "771,47,,," vector.
    assert_eq!(
        hash_no_ext, "fde4273625b2ac63bd01d9c500dac91b",
        "no-extension JA3 for cipher 0x002f must be MD5('771,47,,,') \
         = fde4273625b2ac63bd01d9c500dac91b"
    );
}

// ── AC-004 (BC-2.07.007 postconditions 1-2): 5 fields, first field is version ─
//
// Proptest coverage is in ja3_property_tests::compute_ja3_has_five_fields_and_hex_hash.
// This integration pin uses the public API to confirm the same property.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_ja3_string_has_exactly_four_commas_five_fields() {
    // BC-2.07.007 postcondition 1: JA3 string has exactly 4 commas (5 fields).
    // BC-2.07.007 postcondition 2: first field is decimal version.
    // BC canonical: version=771 (0x0303), ciphers=[0x002f], no ext -> "771,47,,,"
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x002f]),
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();

    // The canonical vector for "771,47,,," has a known hash. We verify indirectly:
    // if the hash is MD5("771,47,,,") the 5-field format holds.
    assert_eq!(
        hash, "fde4273625b2ac63bd01d9c500dac91b",
        "BC-2.07.007 postconditions 1-2: JA3 for version=771, cipher=[0x002f], no ext \
         must be MD5('771,47,,,') confirming 5-field format and decimal version \
         (canonical: fde4273625b2ac63bd01d9c500dac91b)"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_version_zero_companion_no_cipher_produces_known_hash() {
    // BC-2.07.007 EC-002 companion: version=771 with empty ciphers -> "771,,,,"
    // This exercises the "first field is version" postcondition at the API level.
    // The algorithm-level version=0 edge case is covered by the inline proptest
    // (version in any::<u16>()).
    // Canonical: MD5("771,,,,") = bddda940f9963577c41d7c28b1a5f65f
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[]),
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "bddda940f9963577c41d7c28b1a5f65f",
        "BC-2.07.007 EC-002 companion: no-cipher JA3 must be MD5('771,,,,') \
         = bddda940f9963577c41d7c28b1a5f65f confirming first field is version 771"
    );
}

// ── AC-005 (BC-2.07.007 postconditions 3-6): decimal field encoding ───────────
//
// Cipher field: decimal IDs joined by '-'; empty if all GREASE.
// Extension / curves / pf fields: decimal IDs joined by '-'.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_cipher_field_is_decimal_not_hex() {
    // BC-2.07.007 postcondition 3 + invariant 3: ciphers encoded as decimal "47",
    // not hex "0x002f" or name "TLS_RSA_WITH_AES_128_CBC_SHA".
    // 0x002f decimal = 47, 0x0035 decimal = 53.
    // "771,47-53,,," -> MD5 = 577fbfd57b256f5467f2fe09d1105a26.
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x002f, 0x0035]),
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();

    // Canonical: MD5("771,47-53,,,") = 577fbfd57b256f5467f2fe09d1105a26
    assert_eq!(
        hash, "577fbfd57b256f5467f2fe09d1105a26",
        "BC-2.07.007 postcondition 3 + invariant 3: cipher field must be decimal '47-53' \
         (not hex), canonical MD5('771,47-53,,,') = 577fbfd57b256f5467f2fe09d1105a26"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_empty_cipher_field_when_all_grease_or_none() {
    // BC-2.07.007 postcondition 3: if all ciphers are GREASE or none exist,
    // cipher field is "".
    // Both [0x0a0a only] and [] produce JA3 string "771,,,," ->
    // MD5 = bddda940f9963577c41d7c28b1a5f65f.
    let fk = test_flow_key();

    for (label, cipher_ids) in [("all GREASE", &[0x0a0au16][..]), ("empty", &[][..])] {
        let mut analyzer = TlsAnalyzer::new();
        analyzer.on_data(
            &fk,
            Direction::ClientToServer,
            &build_client_hello_no_extensions(cipher_ids),
            0,
        );
        let hash = analyzer.ja3_counts().keys().next().unwrap().clone();
        assert_eq!(
            hash, "bddda940f9963577c41d7c28b1a5f65f",
            "BC-2.07.007 postcondition 3 ({label}): cipher field empty -> \
             MD5('771,,,,') = bddda940f9963577c41d7c28b1a5f65f"
        );
    }
}

// ── AC-006 (BC-2.07.007 postconditions 7-8): MD5 32 lowercase hex chars ───────
//
// Proptest coverage in ja3_property_tests::compute_ja3_has_five_fields_and_hex_hash.
// This pin test verifies the property at the integration level.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_ja3_hash_is_32_lowercase_hex_chars() {
    // BC-2.07.007 postconditions 7-8: hash is MD5 over UTF-8 bytes, 32 lowercase hex.
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("example.com", &[0x002f, 0x0035]),
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash.len(),
        32,
        "JA3 hash must be exactly 32 characters (BC-2.07.007 postcondition 8)"
    );
    assert!(
        hash.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "JA3 hash must be all lowercase hex (BC-2.07.007 postcondition 8), got: {hash}"
    );
}

// ── AC-007 (BC-2.07.007 invariant 2): order-sensitive cipher hashing ──────────
//
// Proptest coverage in ja3_property_tests::compute_ja3_is_order_sensitive.
// This deterministic integration pin confirms the property through the public API.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_cipher_order_produces_different_hashes() {
    // BC-2.07.007 invariant 2: [A, B] and [B, A] produce different hashes.
    // [0x002f, 0x0035] -> "771,47-53,,," -> MD5 = 577fbfd57b256f5467f2fe09d1105a26
    // [0x0035, 0x002f] -> "771,53-47,,," -> MD5 = e570871018118a1c91927ac4f3253bb8
    let fk = test_flow_key();

    let mut a_ab = TlsAnalyzer::new();
    a_ab.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x002f, 0x0035]),
        0,
    );
    let hash_ab = a_ab.ja3_counts().keys().next().unwrap().clone();

    let mut a_ba = TlsAnalyzer::new();
    a_ba.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x0035, 0x002f]),
        0,
    );
    let hash_ba = a_ba.ja3_counts().keys().next().unwrap().clone();

    assert_ne!(
        hash_ab, hash_ba,
        "BC-2.07.007 invariant 2: cipher order [A,B] vs [B,A] must produce different JA3 hashes"
    );
    // Pin exact values from canonical test vectors.
    assert_eq!(
        hash_ab, "577fbfd57b256f5467f2fe09d1105a26",
        "JA3 [0x002f,0x0035] canonical hash"
    );
    assert_eq!(
        hash_ba, "e570871018118a1c91927ac4f3253bb8",
        "JA3 [0x0035,0x002f] canonical hash"
    );
}

// ── AC-008 (BC-2.07.008 postconditions 1-4): JA3S 3-field format ─────────────
//
// JA3S string has exactly 2 commas (3 fields). Field 1: decimal version.
// Field 2: decimal cipher.0 (SINGLE value). Field 3: GREASE-filtered ext IDs or "".

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_008_ja3s_has_exactly_two_commas_three_fields() {
    // BC-2.07.008 postcondition 1: exactly 2 commas.
    // BC-2.07.008 postconditions 2-4: fields are version, cipher, filtered exts.
    // build_server_hello(0x002f) includes renegotiation_info (0xff01 = 65281).
    // JA3S string = "771,47,65281" -> MD5 = 573a9f3f80037fb40d481e2054def5bb
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    // Need client hello first to open the flow
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("example.com", &[0x002f]),
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello(0x002f),
        0,
    );

    assert_eq!(
        analyzer.ja3s_counts().len(),
        1,
        "one JA3S hash must be recorded"
    );
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // Canonical: "771,47,65281" -> MD5 = 573a9f3f80037fb40d481e2054def5bb
    // This verifies 3-field format (only 2 commas: version, cipher, exts).
    assert_eq!(
        hash, "573a9f3f80037fb40d481e2054def5bb",
        "BC-2.07.008 postconditions 1-4: JA3S for version=771, cipher=0x002f, \
         renegotiation_info ext must be MD5('771,47,65281') = 573a9f3f80037fb40d481e2054def5bb \
         (confirming 3-field format with decimal version and cipher)"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_008_ja3s_grease_extension_filtered_from_ext_field() {
    // BC-2.07.008 postcondition 4: GREASE extension IDs are filtered from ext field.
    // build_server_hello_with_grease_ext includes GREASE ext 0x0a0a (filtered) +
    // renegotiation_info 0xff01 (kept). ext_ids = "65281" (same as without GREASE ext).
    // JA3S string = "771,47,65281" -> MD5 = 573a9f3f80037fb40d481e2054def5bb
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("example.com", &[0x002f]),
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello_with_grease_ext(0x002f),
        0,
    );

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // Same hash as without the GREASE extension (it's filtered).
    assert_eq!(
        hash, "573a9f3f80037fb40d481e2054def5bb",
        "BC-2.07.008 postcondition 4: GREASE extension 0x0a0a must be filtered from JA3S \
         ext field; result must be MD5('771,47,65281') = 573a9f3f80037fb40d481e2054def5bb"
    );
}

// ── AC-009 (BC-2.07.008 postconditions 5-6): JA3S MD5 32 lowercase hex, deterministic
//
// Proptest coverage in ja3_property_tests::compute_ja3s_is_deterministic_and_hex.
// This integration pin verifies the same properties through the public API.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_008_ja3s_hash_is_32_lowercase_hex_and_deterministic() {
    // BC-2.07.008 postconditions 5-6: MD5 hex is 32 lowercase chars; deterministic.
    // Same inputs (same ServerHello bytes) fed twice must produce the same hash.
    let fk1 = test_flow_key();
    let fk2 = FlowKey::new(
        "10.0.0.3".parse::<IpAddr>().unwrap(),
        49154,
        "10.0.0.4".parse::<IpAddr>().unwrap(),
        443,
    );

    let sh_bytes = build_server_hello(0x002f);
    let ch_bytes = build_client_hello("example.com", &[0x002f]);

    let mut a1 = TlsAnalyzer::new();
    a1.on_data(&fk1, Direction::ClientToServer, &ch_bytes, 0);
    a1.on_data(&fk1, Direction::ServerToClient, &sh_bytes, 0);
    let hash1 = a1.ja3s_counts().keys().next().unwrap().clone();

    let mut a2 = TlsAnalyzer::new();
    a2.on_data(&fk2, Direction::ClientToServer, &ch_bytes, 0);
    a2.on_data(&fk2, Direction::ServerToClient, &sh_bytes, 0);
    let hash2 = a2.ja3s_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash1, hash2,
        "BC-2.07.008 postcondition 6: same inputs must produce same JA3S hash (deterministic)"
    );
    assert_eq!(
        hash1.len(),
        32,
        "BC-2.07.008 postcondition 5: JA3S hash must be 32 characters"
    );
    assert!(
        hash1
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "BC-2.07.008 postcondition 5: JA3S hash must be all lowercase hex, got: {hash1}"
    );
}

// ── AC-010 (BC-2.07.008 invariants 1-2): JA3S cipher is single value;
//           GREASE filtering only on extension IDs not cipher ─────────────────
//
// The cipher field in JA3S is a SINGLE decimal value (server selects one cipher).
// Even if the server selects a GREASE value (e.g. 0x0a0a), it is NOT filtered
// from the JA3S cipher field.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_008_ja3s_cipher_field_is_single_value_not_filtered() {
    // BC-2.07.008 invariant 1: cipher field is a SINGLE value (not a list).
    // BC-2.07.008 invariant 2: GREASE filtering does NOT apply to the cipher.
    //
    // If server selects GREASE cipher 0x0a0a (decimal 2570), the JA3S cipher
    // field must be "2570" (not filtered to "").
    // JA3S string = "771,2570,65281" -> MD5 = c4b833c0849ff23c29e04fa13f6e87da
    //
    // This MUST differ from the JA3S hash when server selects non-GREASE 0x002f
    // (which would be "771,47,65281" -> MD5 = 573a9f3f80037fb40d481e2054def5bb).
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("example.com", &[0x1301]),
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello(0x0a0a), // GREASE cipher selected by server
        0,
    );

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // Must NOT be the hash for cipher=0x002f (47) — GREASE cipher is NOT filtered.
    assert_ne!(
        hash, "573a9f3f80037fb40d481e2054def5bb",
        "BC-2.07.008 invariant 2: GREASE cipher 0x0a0a must NOT be filtered from JA3S cipher field"
    );
    // Must be the hash for cipher=0x0a0a (2570) — the actual GREASE value in decimal.
    assert_eq!(
        hash, "c4b833c0849ff23c29e04fa13f6e87da",
        "BC-2.07.008 invariants 1-2: server-selected GREASE cipher 0x0a0a must appear as \
         decimal '2570' in JA3S cipher field: MD5('771,2570,65281') = c4b833c0849ff23c29e04fa13f6e87da"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_008_ja3s_grease_extension_filtered_but_grease_cipher_preserved() {
    // Combine AC-010 scenarios: GREASE extension is filtered, GREASE cipher is not.
    // Server hello with GREASE ext 0x0a0a + renegotiation_info, and GREASE cipher 0x0a0a.
    // ext_ids: 0x0a0a filtered, 0xff01 kept -> "65281"
    // cipher: 0x0a0a NOT filtered -> "2570"
    // JA3S string = "771,2570,65281" -> MD5 = c4b833c0849ff23c29e04fa13f6e87da
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("example.com", &[0x1301]),
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello_with_grease_ext(0x0a0a), // GREASE cipher + GREASE ext
        0,
    );

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash, "c4b833c0849ff23c29e04fa13f6e87da",
        "BC-2.07.008: GREASE ext filtered, GREASE cipher kept -> \
         MD5('771,2570,65281') = c4b833c0849ff23c29e04fa13f6e87da"
    );
}
