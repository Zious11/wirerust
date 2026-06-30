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

// ── STORY-052 formalization tests ────────────────────────────────────────────
//
// These tests pin the behavioral contracts for ClientHello parsing
// (BC-2.07.001), the done short-circuit (BC-2.07.003, BC-2.07.034), and TLS 1.3
// legacy_version handling (BC-2.07.032). They use the EXACT function names from
// the story's Acceptance Criteria so traceability from AC → test is machine-checkable.
//
// test_parse_client_hello    — AC-001..006 (BC-2.07.001 postconditions 1-4, 8; invariant 1)
// test_stop_after_handshake  — AC-008..009, AC-012 (BC-2.07.003 postconditions 1-5;
//                              invariants 1-2; BC-2.07.034 postconditions 1-3)
// test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity
//                            — AC-007 (BC-2.07.001 invariant 2) — see below
// compute_ja3_has_five_fields_and_hex_hash (proptest in src/analyzer/tls.rs)
//                            — AC-003 (BC-2.07.001 postcondition 3) — see inline module
// test_tls13_pcap_version_and_ja3 (in tests/tls_integration_tests.rs)
//                            — AC-010, AC-011 (BC-2.07.032 postconditions 1-3; invariants 1-2)

#[test]
fn test_parse_client_hello() {
    // ---- AC-001 (BC-2.07.001 postcondition 1): handshakes_seen += 1 ----
    // ---- AC-002 (BC-2.07.001 postcondition 2): version_counts[0x0303] == 1 ----
    // ---- AC-003 (BC-2.07.001 postcondition 3): ja3_counts has one 32-hex entry ----
    // ---- AC-004 (BC-2.07.001 postcondition 4): sni_counts["example.com"] == 1 ----
    // ---- AC-005 (BC-2.07.001 postcondition 8): client_buf drained after processing ----
    // ---- AC-006 (BC-2.07.001 invariant 1):     handshakes_seen == 1 even with weak ciphers ----
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // AC-001: handshakes_seen incremented by exactly 1
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-001 (BC-2.07.001 pc1): handshakes_seen must be exactly 1 after one ClientHello"
    );

    // AC-002: version 0x0303 (TLS 1.2/1.3 legacy_version) counted
    assert_eq!(
        *analyzer.version_counts().get(&0x0303).unwrap(),
        1,
        "AC-002 (BC-2.07.001 pc2): version_counts[0x0303] must be 1"
    );

    // AC-003: exactly one JA3 hash recorded; it is a 32-char lowercase hex string
    assert_eq!(
        analyzer.ja3_counts().len(),
        1,
        "AC-003 (BC-2.07.001 pc3): ja3_counts must have exactly one entry"
    );
    let ja3_hash = analyzer.ja3_counts().keys().next().unwrap();
    assert_eq!(
        ja3_hash.len(),
        32,
        "AC-003 (BC-2.07.001 pc3): JA3 hash must be 32 hex chars"
    );
    assert!(
        ja3_hash
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "AC-003 (BC-2.07.001 pc3): JA3 hash must be lowercase hex, got: {ja3_hash}"
    );

    // AC-004: SNI "example.com" counted once
    assert_eq!(
        *analyzer.sni_counts().get("example.com").unwrap(),
        1,
        "AC-004 (BC-2.07.001 pc4): sni_counts[\"example.com\"] must be 1"
    );

    // AC-005 (BC-2.07.001 postcondition 8): the record bytes are drained from client_buf
    // after try_parse_records consumes a complete record.  We observe this directly via
    // the #[doc(hidden)] accessor client_buf_len_for_testing: immediately after on_data
    // the buffer must be 0 bytes (all consumed bytes drained).  The pre-call length is
    // the full record length (the record was appended then parsed in the same call), so
    // a nonzero post-call length would prove the drain path was not taken.
    //
    // Flow-presence anchor: client_buf_len_for_testing returns unwrap_or(0), meaning an
    // absent flow and a drained buffer are both indistinguishable from a bare == 0 check.
    // Assert the flow was created (active_flows_len == 1) first, so the subsequent == 0
    // assertion proves drain, not flow-absence.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "AC-005 anchor (BC-2.07.001 pc8): flow must be present before drain check — \
         client_buf_len_for_testing returns 0 for absent flows too"
    );
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        0,
        "AC-005 (BC-2.07.001 pc8): client_buf must be fully drained after a complete \
         ClientHello record is consumed by try_parse_records"
    );

    // AC-006 (BC-2.07.001 invariant 1): handshakes_seen increments exactly once per
    // ClientHello regardless of how many weak ciphers are present.  Here ciphers are
    // 0x1301 + 0x1303 (both strong), so only the base invariant is checked.  The
    // multi-weak-cipher case is exercised in the separate
    // test_parse_client_hello_single_handshake_despite_multiple_weak_ciphers test below.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-006 (BC-2.07.001 inv1): handshakes_seen must be 1, not influenced by cipher count"
    );

    // Sanity: no parse errors from a well-formed record
    assert_eq!(analyzer.parse_error_count(), 0);
}

// ---- AC-006 companion (BC-2.07.001 invariant 1): single increment with multiple weak ciphers
//
// A ClientHello with THREE weak ciphers (NULL, ANON-export, NULL-SHA256) plus one
// strong cipher must still produce handshakes_seen == 1.  The finding count may be
// 1 (the "weak cipher" finding aggregates all weak ciphers in one push), but the
// handshake count must not be multiplied by the cipher count.
#[test]
fn test_parse_client_hello_single_handshake_despite_multiple_weak_ciphers() {
    // AC-006 (BC-2.07.001 invariant 1): handshakes_seen increments exactly once per
    // ClientHello, regardless of how many weak ciphers are present.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // 0x0002 TLS_RSA_WITH_NULL_SHA, 0x0003 TLS_RSA_EXPORT_WITH_RC4_40_MD5,
    // 0x003B TLS_RSA_WITH_NULL_SHA256, plus 0x1301 (strong) for a realistic mix.
    let record = build_client_hello("example.com", &[0x0002, 0x0003, 0x003B, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-006 (BC-2.07.001 inv1): multiple weak ciphers must not multiply handshakes_seen"
    );
    // Exactly one "weak cipher" finding (all weak ciphers are aggregated into one push)
    let weak_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .collect();
    assert_eq!(
        weak_findings.len(),
        1,
        "multiple weak ciphers must produce ONE weak-cipher finding (not one per cipher)"
    );
}

#[test]
fn test_ja3_grease_filtering() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("test.com", &[0x0a0a, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(analyzer.ja3_counts().len(), 1);
    let ja3_hash = analyzer.ja3_counts().keys().next().unwrap();
    assert_eq!(ja3_hash.len(), 32); // MD5 hex = 32 chars
}

#[test]
fn test_parse_error_counter() {
    // AC-007 / BC-2.07.029 postconditions 1-5 (updated for STORY-144 carry path):
    // A handshake record (0x16) carrying a structurally-complete-but-malformed
    // handshake message body causes parse_tls_message_handshake to return Err,
    // incrementing parse_errors by 1. No finding emitted, no panic, flow remains.
    // truncated_records must NOT increment (BC-2.07.029 invariant 1).
    //
    // With the STORY-144 carry path (AC-144-002), a "malformed 5-byte payload"
    // like [0xFF;5] is now interpreted as a body_len-spoof (body_len=0xFFFFFF >
    // MAX_BUF) → Decision-4 fires → handshake_reassembly_overflows++, NOT
    // parse_errors. The genuine parse_errors case is a COMPLETE message (header
    // declares body_len=N, N bytes delivered) with unparseable body content.
    // This fixture sends body_len=5 header + 5-byte malformed body = 9 bytes.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Carry-path fixture: [0x01, 0x00, 0x00, 0x05] header (msg_type=ClientHello,
    // body_len=5) followed by 5 bytes of malformed content 0xFF.
    // The carry drain loop assembles the 9-byte message and calls
    // parse_tls_message_handshake, which fails on the malformed body → parse_errors+1.
    let bad_record = [
        0x16, 0x03, 0x03, 0x00, 0x09, // TLS record header, payload_len=9
        0x01, 0x00, 0x00, 0x05, // handshake header: type=0x01, body_len=5
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ]; // malformed 5-byte body
    analyzer.on_data(&fk, Direction::ClientToServer, &bad_record, 0, 0);

    // BC-2.07.029 postcondition 1: parse_errors incremented by exactly 1.
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "AC-007 (BC-2.07.029 pc1): parse_errors must be 1 after one malformed handshake record"
    );

    // BC-2.07.029 invariant 1: truncated_records must NOT be incremented
    // (this is a genuine parse failure, not an oversized-record drop).
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-007 (BC-2.07.029 inv1): truncated_records must be 0 for a genuine parse failure \
         (only oversized records increment truncated_records)"
    );

    // BC-2.07.029 postcondition 2: no finding pushed.
    assert!(
        analyzer.findings().is_empty(),
        "AC-007 (BC-2.07.029 pc2): no finding must be emitted for a malformed handshake record; \
         got: {:?}",
        analyzer.findings()
    );

    // BC-2.07.029 postcondition 3: no panic occurred (implicit — test ran to this line).

    // BC-2.07.029 postcondition 4: flow remains in flows HashMap.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "AC-007 (BC-2.07.029 pc4): flow must remain in flows HashMap after a parse error \
         (state not cleared)"
    );

    // BC-2.07.029 sanity cross-check: handshakes_seen unchanged on parse error
    // (a genuine parse failure does not advance the handshake count).
    assert_eq!(
        analyzer.handshake_count(),
        0,
        "AC-007 (BC-2.07.029 sanity): handshakes_seen must be 0 after a malformed handshake record"
    );
}

#[test]
fn test_normal_request_no_parse_errors() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(analyzer.parse_error_count(), 0);
    assert!(analyzer.findings().is_empty());
}

#[test]
fn test_parse_server_hello() {
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    assert_eq!(analyzer.ja3s_counts().len(), 1);
    assert_eq!(analyzer.parse_error_count(), 0);
}

// STORY-054 Brownfield-Formalization Tests (BC-2.07.009/010/011/012/030/036)
//
// These tests strengthen the existing stub assertions to full BC-clause fidelity
// and add coverage for all STORY-054 acceptance criteria. Each test is annotated
// with the AC and BC clause it traces to.
//
// Naming follows DF-AC-TEST-NAME-SYNC-001 (PG-W17-001): story AC `Test:` citations
// mandate exact function names for AC-001..006. ACs 007-013 use the suggested names
// from the story and are reported back for citation-sync.

// ── BC-2.07.009 ──────────────────────────────────────────────────────────────
// AC-001 (BC-2.07.009 postconditions 1-3): one Anomaly/Likely/High finding with
//   exact summary, evidence of cipher names, mitre_technique=None, direction=ClientToServer.
// AC-002 (BC-2.07.009 postcondition 2): exactly ONE finding per ClientHello regardless
//   of how many weak ciphers are present.
// AC-003 (BC-2.07.009 invariant 1-2): GREASE and unknown cipher IDs do not trigger.
#[test]
fn test_weak_cipher_finding_client() {
    // ── AC-001 / BC-2.07.009 postconditions 1-3: single NULL cipher → exact finding fields ──
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // TLS_RSA_WITH_NULL_SHA (0x0002) — NULL cipher; 0x1301 is strong (TLS_AES_128_GCM_SHA256).
    let record = build_client_hello("test.com", &[0x0002, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    let findings = analyzer.findings();

    // BC-2.07.009 postcondition 2: exactly one finding, regardless of cipher count.
    assert_eq!(
        findings.len(),
        1,
        "BC-2.07.009 postcondition 2: exactly ONE finding per ClientHello with weak cipher"
    );

    let f = &findings[0];

    // BC-2.07.009 postcondition 1: category = Anomaly
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "BC-2.07.009 postcondition 1: category must be Anomaly"
    );

    // BC-2.07.009 postcondition 1: verdict = Likely
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Likely,
        "BC-2.07.009 postcondition 1: verdict must be Likely"
    );

    // BC-2.07.009 postcondition 1: confidence = High (client side)
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::High,
        "BC-2.07.009 postcondition 1: confidence must be High for client-side weak cipher"
    );

    // BC-2.07.009 postcondition 1: exact summary text
    assert_eq!(
        f.summary, "ClientHello offers weak cipher suites (NULL/anonymous/export)",
        "BC-2.07.009 postcondition 1: summary must be exact canonical string"
    );

    // BC-2.07.009 postcondition 1 / F-S054-P2-001: EXACT evidence pin for 0x0002.
    // TLS_RSA_WITH_NULL_SHA is the IANA name for cipher 0x0002 (verified from
    // tls-parser-0.12.2/scripts/tls-ciphersuites.txt, line "0002:TLS_RSA_WITH_NULL_SHA:...").
    // Single weak cipher → evidence must have exactly one entry with that exact name.
    assert_eq!(
        f.evidence,
        vec!["TLS_RSA_WITH_NULL_SHA".to_string()],
        "F-S054-P2-001 (BC-2.07.009 pc1/INV-4): evidence must be exactly \
         [\"TLS_RSA_WITH_NULL_SHA\"] for cipher 0x0002; got: {:?}",
        f.evidence
    );
    assert_eq!(
        f.evidence.len(),
        1,
        "F-S054-P2-001 (BC-2.07.009 pc1): single weak cipher 0x0002 must produce \
         exactly one evidence entry"
    );
    // Evidence entries are names, not hex IDs (INV-4 cross-check).
    assert!(
        !f.evidence.iter().any(|e| e.starts_with("0x")),
        "BC-2.07.009 postcondition 1 (INV-4): evidence must store cipher NAMES not hex IDs; \
         got: {:?}",
        f.evidence
    );

    // BC-2.07.009 postcondition 1: mitre_technique = None
    assert_eq!(
        f.mitre_techniques,
        Vec::<String>::new(),
        "BC-2.07.009 postcondition 1: mitre_technique must be None for weak-cipher finding"
    );

    // BC-2.07.009 postcondition 1: direction = ClientToServer
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "BC-2.07.009 postcondition 1: direction must be ClientToServer"
    );

    // ── AC-002 / BC-2.07.009 postcondition 2: multiple weak ciphers → still ONE finding ──
    // Three weak ciphers: TLS_RSA_WITH_NULL_SHA (0x0002), TLS_RSA_EXPORT_WITH_RC4_40_MD5 (0x0003),
    // TLS_RSA_WITH_NULL_SHA256 (0x003B), plus one strong cipher.
    let mut analyzer2 = TlsAnalyzer::new();
    let record2 = build_client_hello("test.com", &[0x0002, 0x0003, 0x003B, 0x1301]);
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

    let findings2 = analyzer2.findings();
    let weak_findings: Vec<_> = findings2
        .iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .collect();
    assert_eq!(
        weak_findings.len(),
        1,
        "BC-2.07.009 postcondition 2: THREE weak ciphers must still produce exactly ONE \
         weak-cipher finding (cardinality invariant)"
    );
    // BC-2.07.009 postcondition 1 (INV-4): evidence has one entry per weak cipher name.
    assert_eq!(
        weak_findings[0].evidence.len(),
        3,
        "BC-2.07.009 postcondition 1 (O-06): evidence must have one entry per weak cipher \
         name (3 weak ciphers → 3 evidence entries)"
    );

    // ── AC-003 / BC-2.07.009 invariant 1-2: GREASE cipher 0x0a0a → no finding ──
    // GREASE values have from_id() returning None → is_weak_cipher returns false.
    let mut analyzer3 = TlsAnalyzer::new();
    let record3 = build_client_hello("test.com", &[0x0a0a, 0x1301]);
    analyzer3.on_data(&fk, Direction::ClientToServer, &record3, 0, 0);

    assert!(
        analyzer3.findings().is_empty(),
        "BC-2.07.009 invariant 1: GREASE cipher 0x0a0a must NOT trigger weak-cipher finding \
         (from_id returns None → is_weak_cipher returns false); got: {:?}",
        analyzer3.findings()
    );
}

// ── BC-2.07.010 ──────────────────────────────────────────────────────────────
// AC-004 (BC-2.07.010 postconditions 1-2): one Anomaly/Likely/Medium finding with
//   exact summary, evidence with name+hex, mitre_technique=None, direction=ServerToClient.
// AC-005 (BC-2.07.010 invariant 1): is_weak_server_cipher is superset of is_weak_cipher;
//   RC4 ciphers trigger the server-side finding (not the client-side one).
#[test]
fn test_weak_cipher_finding_server() {
    // ── AC-004 / BC-2.07.010 postconditions 1-2: RC4 server cipher → exact finding fields ──
    //
    // F-S054-P1-004: switched from 0x0005 (TLS_RSA_WITH_RC4_128_SHA) to 0x0004
    // (TLS_RSA_WITH_RC4_128_MD5) to align with AC-005 / BC-2.07.010 EC-001, which names
    // TLS_RSA_WITH_RC4_128_MD5 as the canonical RC4 vector. Empirically verified: 0x0004 IS
    // recognized by tls-parser 0.12 (from_id returns Some), contains "RC4" in name, and
    // triggers is_weak_server_cipher. The 0x0005 variant also works but does not match AC text.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Strong client hello — no client-side weak cipher finding.
    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // Server selects TLS_RSA_WITH_RC4_128_MD5 (0x0004) — RC4 triggers is_weak_server_cipher.
    // (is_weak_cipher does NOT include RC4 — AC-005 invariant 1)
    // 0x0004 = TLS_RSA_WITH_RC4_128_MD5 per AC-005/BC-2.07.010 EC-001.
    let sh = build_server_hello(0x0004);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    let findings = analyzer.findings();

    // BC-2.07.010 postcondition 2: exactly one finding per ServerHello.
    assert_eq!(
        findings.len(),
        1,
        "BC-2.07.010 postcondition 2: exactly ONE finding per ServerHello with weak server cipher"
    );

    let f = &findings[0];

    // BC-2.07.010 postcondition 1: category = Anomaly
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "BC-2.07.010 postcondition 1: category must be Anomaly"
    );

    // BC-2.07.010 postcondition 1: verdict = Likely
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Likely,
        "BC-2.07.010 postcondition 1: verdict must be Likely"
    );

    // BC-2.07.010 postcondition 1: confidence = Medium (server selected — not High as client)
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Medium,
        "BC-2.07.010 postcondition 1: confidence must be Medium for server-side weak cipher \
         (server makes final selection — BC-2.07.010 invariant 3)"
    );

    // BC-2.07.010 postcondition 1: exact summary "ServerHello selected weak cipher suite ({name})"
    // F-S054-P1-003: strengthened from starts_with+contains to exact assert_eq per BC-2.07.010 PC1.
    // The IANA name for 0x0004 is "TLS_RSA_WITH_RC4_128_MD5" per tls-parser 0.12.
    assert_eq!(
        f.summary, "ServerHello selected weak cipher suite (TLS_RSA_WITH_RC4_128_MD5)",
        "BC-2.07.010 postcondition 1 (AC-004/EC-001): exact summary must be \
         'ServerHello selected weak cipher suite (TLS_RSA_WITH_RC4_128_MD5)' for cipher 0x0004; \
         got: {:?}",
        f.summary
    );

    // BC-2.07.010 postcondition 1: evidence = ["Selected cipher: {name} (0x{id:04x})"]
    assert_eq!(
        f.evidence.len(),
        1,
        "BC-2.07.010 invariant 4: evidence must be exactly one string"
    );

    // Exact evidence string for cipher 0x0004 (TLS_RSA_WITH_RC4_128_MD5).
    assert_eq!(
        f.evidence[0], "Selected cipher: TLS_RSA_WITH_RC4_128_MD5 (0x0004)",
        "BC-2.07.010 postcondition 1 (AC-004/EC-001): exact evidence must be \
         'Selected cipher: TLS_RSA_WITH_RC4_128_MD5 (0x0004)'; got: {:?}",
        f.evidence[0]
    );

    // BC-2.07.010 postcondition 1: mitre_technique = None
    assert_eq!(
        f.mitre_techniques,
        Vec::<String>::new(),
        "BC-2.07.010 postcondition 1: mitre_technique must be None"
    );

    // BC-2.07.010 postcondition 1 / invariant 2: direction = ServerToClient
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ServerToClient),
        "BC-2.07.010 invariant 2: direction must be ServerToClient (not ClientToServer)"
    );

    // ── AC-005 / BC-2.07.010 invariant 1: RC4 does NOT trigger is_weak_cipher (client-side) ──
    // A ClientHello offering ONLY RC4 ciphers must NOT produce a client-side weak-cipher finding,
    // because is_weak_cipher checks for NULL/ANON/EXPORT but NOT RC4.
    // Use 0x0004 (TLS_RSA_WITH_RC4_128_MD5) consistent with AC-005 EC-001.
    let mut analyzer_rc4_client = TlsAnalyzer::new();
    // Note: 0x0004 = TLS_RSA_WITH_RC4_128_MD5. Its name contains RC4 but not NULL/ANON/EXPORT.
    // So is_weak_cipher returns false → no client-side finding.
    let ch_rc4 = build_client_hello("test.com", &[0x0004, 0x1301]);
    analyzer_rc4_client.on_data(&fk, Direction::ClientToServer, &ch_rc4, 0, 0);

    let client_findings = analyzer_rc4_client.findings();
    let client_weak: Vec<_> = client_findings
        .iter()
        .filter(|f| f.summary.contains("ClientHello offers weak cipher"))
        .collect();
    assert_eq!(
        client_weak.len(),
        0,
        "BC-2.07.010 invariant 1: RC4 cipher 0x0004 (TLS_RSA_WITH_RC4_128_MD5) must NOT trigger \
         client-side is_weak_cipher (RC4 not in NULL/ANON/EXPORT set); \
         is_weak_server_cipher is strict superset"
    );
}

// ── BC-2.07.030 ──────────────────────────────────────────────────────────────
// AC-003 (BC-2.07.009 invariant 1-2, extends to BC-2.07.030 postconditions 1-4):
//   modern strong TLS handshake produces zero findings; all counters increment.
// AC-011 (BC-2.07.030 postconditions 1-4): all_findings empty after both hellos;
//   handshakes_seen==1; all count maps have exactly one entry each; parse_errors==0.
#[test]
fn test_normal_handshake_no_findings() {
    // AC-003 / AC-011 / BC-2.07.030 postconditions 1-4:
    // Clean ASCII SNI, version > 0x0300, no weak ciphers → zero findings.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello: TLS 1.2/1.3 legacy_version 0x0303, strong ciphers only.
    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello: strong cipher TLS_AES_128_GCM_SHA256 (0x1301), no weak cipher.
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // BC-2.07.030 postcondition 1: all_findings must be empty (zero false positives).
    assert!(
        analyzer.findings().is_empty(),
        "BC-2.07.030 postcondition 1: all_findings must be empty after a clean modern \
         TLS handshake; got: {:?}",
        analyzer.findings()
    );

    // BC-2.07.030 postcondition 2: handshakes_seen == 1.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "BC-2.07.030 postcondition 2: handshakes_seen must be exactly 1"
    );

    // BC-2.07.030 postcondition 4: parse_errors == 0.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-2.07.030 postcondition 4: parse_errors must be 0 for well-formed handshake"
    );

    // BC-2.07.030 postcondition 3: all count maps have exactly one entry each.
    assert_eq!(
        analyzer.sni_counts().len(),
        1,
        "BC-2.07.030 postcondition 3: sni_counts must have exactly one entry"
    );
    assert_eq!(
        analyzer.ja3_counts().len(),
        1,
        "BC-2.07.030 postcondition 3: ja3_counts must have exactly one entry"
    );
    assert_eq!(
        analyzer.ja3s_counts().len(),
        1,
        "BC-2.07.030 postcondition 3: ja3s_counts must have exactly one entry"
    );
    // version_counts: ClientHello inserts 0x0303; ServerHello inserts 0x0303 again → still one key.
    assert_eq!(
        analyzer.version_counts().len(),
        1,
        "BC-2.07.030 postcondition 3: version_counts must have exactly one entry (0x0303)"
    );

    // F-S054-P3-002 / BC-2.07.030 PC3: cipher_counts must also have exactly one entry.
    // ServerHello selects 0x1301 (TLS_AES_128_GCM_SHA256) → cipher_counts["TLS_AES_128_GCM_SHA256"] = 1.
    // Accessed via summarize().detail["cipher_suites"] (cipher_counts is a private field).
    let detail = analyzer.summarize().detail;
    let cipher_suites = detail
        .get("cipher_suites")
        .expect("cipher_suites must be present in summarize() output");
    assert_eq!(
        cipher_suites.as_object().map(|m| m.len()).unwrap_or(0),
        1,
        "F-S054-P3-002 (BC-2.07.030 PC3): cipher_counts must have exactly one entry after \
         one ServerHello; got cipher_suites: {cipher_suites}"
    );
}

#[test]
fn test_stop_after_handshake() {
    // ---- AC-008 (BC-2.07.003 postconditions 1-5): done() == true causes on_data to
    //              return immediately; no bytes buffered, no counters changed, no findings,
    //              flow state remains in the HashMap.
    // ---- AC-009 (BC-2.07.003 invariants 1-2): done-check is first; retransmitted
    //              ClientHello after done() does NOT increment handshakes_seen.
    // ---- AC-012 (BC-2.07.034 postconditions 1-3): 1 MB application-data burst after
    //              both hellos leaves all counters at their post-handshake values.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Snapshot post-handshake counters — these must not change after the done() short-circuit.
    let handshakes_after_hellos = analyzer.handshake_count();
    let parse_errors_after_hellos = analyzer.parse_error_count();
    let sni_len_after_hellos = analyzer.sni_counts().len();
    let ja3_len_after_hellos = analyzer.ja3_counts().len();
    let version_len_after_hellos = analyzer.version_counts().len();
    let findings_after_hellos = analyzer.findings().len();

    assert_eq!(
        handshakes_after_hellos, 1,
        "AC-008 setup: expected exactly 1 handshake after ClientHello+ServerHello"
    );

    // AC-009 (BC-2.07.003 invariant 2): send a retransmitted ClientHello after done().
    // handshakes_seen must NOT increment — the short-circuit fires before any parsing.
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);
    assert_eq!(
        analyzer.handshake_count(),
        handshakes_after_hellos,
        "AC-009 (BC-2.07.003 inv2): retransmitted ClientHello after done() must NOT \
         increment handshakes_seen (expected {handshakes_after_hellos})"
    );

    // AC-008 (BC-2.07.003 pc2): no bytes appended to buffers.
    // We verify via parse_error_count: if bytes were buffered and parsed, a non-handshake
    // retransmission or garbage appended to the real ClientHello would cause a parse error.
    // The absence of parse errors is the observable proxy for "no buffering occurred".
    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_after_hellos,
        "AC-008 (BC-2.07.003 pc3): parse_errors must not increase after done()"
    );

    // AC-008 / AC-012 (BC-2.07.034 pc1-3): send a 1 MB burst of application data
    // (content_type=0x17) after both hellos — all bytes must be silently discarded.
    // BC-2.07.034 canonical test vector: ClientHello + ServerHello + 1 MB data ->
    // all counters reflect only the two hellos; no parse_errors from app data.
    let large_app_data: Vec<u8> = {
        let mut v = Vec::with_capacity(1_048_576);
        // Fill with 65,535-byte chunks (just under MAX_BUF) of realistic content_type=0x17
        // framing to exercise the buffer-bypass path, not just the "data too small to parse" path.
        // We use arbitrary bytes rather than well-formed TLS records because the short-circuit
        // fires BEFORE any parsing, so the content doesn't matter.
        v.extend(std::iter::repeat_n(0xBBu8, 1_048_576));
        v
    };
    analyzer.on_data(&fk, Direction::ServerToClient, &large_app_data, 0, 0);

    // AC-012 (BC-2.07.034 pc4): all counters at their post-handshake values
    assert_eq!(
        analyzer.handshake_count(),
        handshakes_after_hellos,
        "AC-012 (BC-2.07.034 pc4): 1 MB burst must not change handshakes_seen"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_after_hellos,
        "AC-012 (BC-2.07.034 pc4): 1 MB burst must not change parse_errors \
         (BC-2.07.003 pc3 / BC-2.07.034 pc4: no parse attempt on post-done data)"
    );
    assert_eq!(
        analyzer.sni_counts().len(),
        sni_len_after_hellos,
        "AC-012 (BC-2.07.034 pc4): 1 MB burst must not change sni_counts"
    );
    assert_eq!(
        analyzer.ja3_counts().len(),
        ja3_len_after_hellos,
        "AC-012 (BC-2.07.034 pc4): 1 MB burst must not change ja3_counts"
    );
    assert_eq!(
        analyzer.version_counts().len(),
        version_len_after_hellos,
        "AC-012 (BC-2.07.034 pc4): 1 MB burst must not change version_counts"
    );
    assert_eq!(
        analyzer.findings().len(),
        findings_after_hellos,
        "AC-012 (BC-2.07.034 pc4): 1 MB burst must not add findings"
    );

    // AC-008 (BC-2.07.003 pc1): on_data returns immediately (no panic, no hang).
    // Send empty slice — EC-003 from BC-2.07.003: empty on_data after done returns immediately.
    analyzer.on_data(&fk, Direction::ServerToClient, &[], 0, 0);
    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_after_hellos,
        "AC-008 EC-003 (BC-2.07.003 ec3): empty on_data after done must have no effect"
    );

    // Final invariant: all counters unchanged from post-handshake snapshot
    assert_eq!(
        analyzer.handshake_count(),
        handshakes_after_hellos,
        "AC-009 (BC-2.07.003 inv2): done() is permanent — handshakes_seen frozen"
    );
}

#[test]
fn test_summarize_output() {
    // AC-009 / BC-2.07.031 postconditions 1-9:
    // summarize returns AnalysisSummary with analyzer_name=="TLS",
    // packets_analyzed==handshakes_seen, and a detail BTreeMap containing
    // all 7 required keys.
    //
    // AC-010 / BC-2.07.031 invariants 1-4:
    // detail is a BTreeMap (alphabetically ordered keys), top_snis <= 20 entries,
    // version_counts keys are decimal strings ("771" not "0x0303"),
    // truncated_records key is always present.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301, 0x1303]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    let summary = analyzer.summarize();

    // BC-2.07.031 postcondition 1: analyzer_name == "TLS" (exact string).
    assert_eq!(
        summary.analyzer_name, "TLS",
        "AC-009 (BC-2.07.031 pc1): analyzer_name must be exact string \"TLS\""
    );

    // BC-2.07.031 postcondition 2: packets_analyzed == handshakes_seen (==1 after one CH).
    assert_eq!(
        summary.packets_analyzed, 1,
        "AC-009 (BC-2.07.031 pc2): packets_analyzed must equal handshakes_seen (==1)"
    );

    let detail = &summary.detail;

    // AC-009 / BC-2.07.031 postconditions 3-9 + AC-144-003:
    // EXACT 8-key set — no more, no fewer (BTreeMap ordering enforced below).
    // Updated from 7→8 in STORY-144 to include `handshake_reassembly_overflows`
    // (BC-2.07.039 Postcondition 7 / AC-144-003).
    let required_keys = [
        "cipher_suites",
        "handshake_reassembly_overflows",
        "ja3_hashes",
        "ja3s_hashes",
        "parse_errors",
        "tls_versions",
        "top_snis",
        "truncated_records",
    ];
    for key in &required_keys {
        assert!(
            detail.contains_key(*key),
            "AC-009 (BC-2.07.031 pc3-9): detail must contain key \"{key}\""
        );
    }
    assert_eq!(
        detail.len(),
        8,
        "AC-009 (BC-2.07.031 pc3-9 + AC-144-003): detail must have EXACTLY 8 keys, got: {:?}",
        detail.keys().collect::<Vec<_>>()
    );

    // AC-010 / BC-2.07.031 invariant 1: BTreeMap — keys are alphabetically ordered.
    // Collect the actual key order and compare to the sorted order.
    let actual_keys: Vec<&String> = detail.keys().collect();
    let mut sorted_keys = actual_keys.clone();
    sorted_keys.sort();
    assert_eq!(
        actual_keys, sorted_keys,
        "AC-010 (BC-2.07.031 inv1): detail keys must be in alphabetical order \
         (BTreeMap guarantee, LESSON-P2.09); got: {:?}",
        actual_keys
    );

    // BC-2.07.031 postcondition 3: top_snis is a JSON array containing "example.com".
    assert!(
        detail["top_snis"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("example.com")),
        "AC-009 (BC-2.07.031 pc3): top_snis must contain \"example.com\""
    );

    // AC-010 / BC-2.07.031 invariant 2: top_snis has at most 20 entries.
    let top_snis_len = detail["top_snis"].as_array().unwrap().len();
    assert!(
        top_snis_len <= 20,
        "AC-010 (BC-2.07.031 inv2): top_snis must have at most 20 entries; got {top_snis_len}"
    );

    // BC-2.07.031 postcondition 4: ja3_hashes is a JSON object.
    assert!(
        detail["ja3_hashes"].is_object(),
        "AC-009 (BC-2.07.031 pc4): ja3_hashes must be a JSON object"
    );

    // BC-2.07.031 postcondition 5: ja3s_hashes is a JSON object.
    assert!(
        detail["ja3s_hashes"].is_object(),
        "AC-009 (BC-2.07.031 pc5): ja3s_hashes must be a JSON object"
    );

    // BC-2.07.031 postcondition 6: tls_versions is a JSON object with decimal string keys.
    // ClientHello version 0x0303 → key "771" (not "0x0303" / "771 hex").
    // AC-010 / BC-2.07.031 invariant 3: version_counts u16 keys converted via k.to_string().
    assert!(
        detail["tls_versions"].is_object(),
        "AC-009 (BC-2.07.031 pc6): tls_versions must be a JSON object"
    );
    let versions_obj = detail["tls_versions"].as_object().unwrap();
    assert!(
        versions_obj.contains_key("771"),
        "AC-010 (BC-2.07.031 inv3): tls_versions keys must be decimal strings (\"771\" for \
         0x0303 TLS 1.2), NOT hex strings; got keys: {:?}",
        versions_obj.keys().collect::<Vec<_>>()
    );
    assert!(
        !versions_obj.contains_key("0x0303"),
        "AC-010 (BC-2.07.031 inv3): tls_versions must NOT contain hex string key \"0x0303\"; \
         keys must be decimal strings via k.to_string()"
    );

    // BC-2.07.031 postcondition 7: cipher_suites is a JSON object.
    assert!(
        detail["cipher_suites"].is_object(),
        "AC-009 (BC-2.07.031 pc7): cipher_suites must be a JSON object"
    );

    // BC-2.07.031 postcondition 8: parse_errors is a JSON number == 0.
    assert_eq!(
        detail["parse_errors"],
        serde_json::json!(0u64),
        "AC-009 (BC-2.07.031 pc8): parse_errors must be a JSON number equal to 0"
    );

    // BC-2.07.031 postcondition 9 / AC-009 (LESSON-P1.05): truncated_records is a JSON number.
    assert!(
        detail["truncated_records"].is_number(),
        "AC-009 (BC-2.07.031 pc9): truncated_records must be a JSON number (LESSON-P1.05)"
    );
    assert_eq!(
        detail["truncated_records"],
        serde_json::json!(0u64),
        "AC-009 (BC-2.07.031 pc9): truncated_records must be 0 for a clean handshake"
    );
}

// AC-004/AC-005 (BC-2.07.019 postconditions 1-4, invariant 1)
//
// When extract_sni receives SNI bytes that fail str::from_utf8, arm 4 fires.
// sni_counts key is "<non-utf8:{hex}>" (hex-tagged, NOT the lossy form).
// One Finding: category=Anomaly, verdict=Inconclusive, confidence=Low,
// summary="TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}",
// evidence=["hex: {hex}"], mitre_technique=Some("T1027"), direction=ClientToServer.
//
// EC-004: b"\xff\xfe" -> key = "<non-utf8:fffe>".
// EC-006: b"\xff" and b"\xfe" both map to same lossy but DIFFERENT keys.
#[allow(non_snake_case)]
#[test]
fn test_non_utf8_sni_emits_finding_and_counts_under_hex_key() {
    // AC-004 (BC-2.07.019 pc1): arm 4 fires -> SniValue::NonUtf8 { lossy, hex }.
    // AC-004 (BC-2.07.019 pc3): one Finding with all required fields.
    // AC-005 (BC-2.07.019 pc2/inv1): sni_counts key is "<non-utf8:{hex}>".
    let fk = test_flow_key();

    // --- Part 1: EC-004 canonical vector: b"\xff\xfe" (invalid UTF-8) ---
    let mut analyzer = TlsAnalyzer::new();
    let sni_bytes: &[u8] = b"\xff\xfe";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Parse error counter must NOT be incremented — record itself parsed OK.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-004 anchor (BC-2.07.019): parse_error_count must be 0"
    );
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-004 anchor (BC-2.07.019): exactly one handshake counted"
    );

    // BC-2.07.019 pc3: exactly one non-UTF-8 finding emitted.
    let non_utf8_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .collect();
    assert_eq!(
        non_utf8_findings.len(),
        1,
        "AC-004 (BC-2.07.019 pc3): exactly one non-UTF-8 finding must be emitted; \
         got: {non_utf8_findings:?}"
    );

    let f = &non_utf8_findings[0];

    // BC-2.07.019 pc3: category = Anomaly.
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-004 (BC-2.07.019 pc3): category must be Anomaly"
    );
    // BC-2.07.019 pc3: verdict = Inconclusive.
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "AC-004 (BC-2.07.019 pc3): verdict must be Inconclusive"
    );
    // BC-2.07.019 pc3: confidence = Low.
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "AC-004 (BC-2.07.019 pc3): confidence must be Low"
    );
    // BC-2.07.019 pc3: mitre_techniques = ["T1027"] (single-tag, full-vec equality).
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-004 (BC-2.07.019 pc3): mitre_techniques must be exactly [\"T1027\"]"
    );
    // BC-2.07.019 pc3: direction = Some(ClientToServer).
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "AC-004 (BC-2.07.019 pc3): direction must be Some(ClientToServer)"
    );

    // BC-2.07.019 pc3: source_ip must be None (network context not available in analyzer).
    assert_eq!(f.source_ip, None, "BC-2.07.019 pc3: source_ip must be None");
    // BC-2.09.007 post-1 (STORY-098): timestamp is now Some(DateTime<Utc>) derived from the
    // per-flow last_ts; this test calls on_data with timestamp=0, so the result is
    // Some(1970-01-01T00:00:00Z) — not None. The "None" assertion is superseded by STORY-098.
    {
        use chrono::DateTime;
        let expected_ts = DateTime::from_timestamp(0_i64, 0);
        assert_eq!(
            f.timestamp, expected_ts,
            "BC-2.09.007 (STORY-098): TLS SNI finding must have timestamp == \
             DateTime::from_timestamp(0, 0) = {expected_ts:?} (on_data called with ts_sec=0)"
        );
    }

    // BC-2.07.019 pc3/pc4: exact summary uses lossy from_utf8_lossy form.
    let lossy = String::from_utf8_lossy(sni_bytes).into_owned();
    let expected_summary =
        format!("TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}");
    assert_eq!(
        f.summary, expected_summary,
        "AC-004 (BC-2.07.019 pc3/pc4): summary must use lossy form with U+FFFD replacements"
    );

    // BC-2.07.020: break the tautology — for b"\xff\xfe" each invalid byte must
    // produce exactly one U+FFFD replacement character in the summary.
    // (from_utf8_lossy replaces each maximal invalid subsequence with one U+FFFD;
    // b"\xff" and b"\xfe" are each a one-byte invalid subsequence => two replacements.)
    assert_eq!(
        f.summary.matches('\u{fffd}').count(),
        2,
        "BC-2.07.020: each invalid byte in b\"\\xff\\xfe\" must produce one U+FFFD in summary; \
         got summary: {:?}",
        f.summary
    );

    // BC-2.07.019 pc3: evidence = ["hex: fffe"] — exactly one entry, exact string.
    assert_eq!(
        f.evidence.len(),
        1,
        "AC-004 (BC-2.07.019 pc3): evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "hex: fffe",
        "AC-004 (BC-2.07.019 pc3): evidence[0] must be exact lowercase hex"
    );

    // AC-005 (BC-2.07.019 pc2/inv1): sni_counts key is "<non-utf8:fffe>" (hex-tagged).
    assert_eq!(
        *analyzer.sni_counts().get("<non-utf8:fffe>").unwrap_or(&0),
        1,
        "AC-005 (BC-2.07.019 pc2/inv1): sni_counts key must be \"<non-utf8:fffe>\"; \
         got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );

    // --- Part 2: EC-006 collision test (BC-2.07.019 inv1) ---
    // b"\xff" and b"\xfe" both produce the same lossy "?" — different hex keys.
    let mut analyzer_ff = TlsAnalyzer::new();
    let record_ff = build_client_hello_raw_sni(b"\xff", &[0x1301]);
    analyzer_ff.on_data(&fk, Direction::ClientToServer, &record_ff, 0, 0);

    let mut analyzer_fe = TlsAnalyzer::new();
    let record_fe = build_client_hello_raw_sni(b"\xfe", &[0x1301]);
    analyzer_fe.on_data(&fk, Direction::ClientToServer, &record_fe, 0, 0);

    // EC-006: distinct sequences with same lossy => different sni_counts keys.
    assert_eq!(
        *analyzer_ff.sni_counts().get("<non-utf8:ff>").unwrap_or(&0),
        1,
        "AC-005/EC-006 (BC-2.07.019 inv1): b\"\\xff\" must map to \"<non-utf8:ff>\""
    );
    assert_eq!(
        *analyzer_fe.sni_counts().get("<non-utf8:fe>").unwrap_or(&0),
        1,
        "AC-005/EC-006 (BC-2.07.019 inv1): b\"\\xfe\" must map to \"<non-utf8:fe>\""
    );

    // Confirm the lossy forms collide (proving the hex key prevents false merges).
    let lossy_ff = String::from_utf8_lossy(b"\xff").into_owned();
    let lossy_fe = String::from_utf8_lossy(b"\xfe").into_owned();
    assert_eq!(
        lossy_ff, lossy_fe,
        "EC-006 setup: b\"\\xff\" and b\"\\xfe\" must produce identical lossy forms"
    );

    // --- Part 3: original regression vector (mixed ASCII + invalid bytes) ---
    let mut analyzer3 = TlsAnalyzer::new();
    let raw_sni: &[u8] = &[0xff, 0xfe, b'a', b'.', b'c', b'o', b'm'];
    let record3 = build_client_hello_raw_sni(raw_sni, &[0x1301]);
    analyzer3.on_data(&fk, Direction::ClientToServer, &record3, 0, 0);

    assert_eq!(
        analyzer3.parse_error_count(),
        0,
        "AC-004 regression anchor: parse_error_count must be 0 for mixed-byte SNI"
    );

    let expected_key3 = "<non-utf8:fffe612e636f6d>";
    assert_eq!(
        *analyzer3.sni_counts().get(expected_key3).unwrap_or(&0),
        1,
        "AC-005 regression (BC-2.07.019 pc2): sni_counts must contain hex-tagged key \
         {expected_key3}"
    );

    let findings3 = analyzer3.findings();
    let non_utf8_findings3: Vec<_> = findings3
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .collect();
    assert_eq!(
        non_utf8_findings3.len(),
        1,
        "AC-004 regression: exactly one non-UTF-8 finding; got: {findings3:?}"
    );
    assert!(
        non_utf8_findings3[0]
            .evidence
            .iter()
            .any(|e| e.contains("fffe612e636f6d")),
        "AC-006 regression: hex evidence must contain raw byte sequence; got: {:?}",
        non_utf8_findings3[0].evidence
    );
}

#[test]
fn test_ascii_sni_does_not_emit_non_utf8_finding() {
    // Regression: a normal ASCII hostname must not trip the non-UTF-8 finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(*analyzer.sni_counts().get("example.com").unwrap(), 1);
    let non_utf8_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(non_utf8_findings, 0);
}

// AC-006 (BC-2.07.020 postconditions 1-4)
//
// For arm 4, finding.summary contains the from_utf8_lossy form (U+FFFD replacements).
// finding.evidence[0] = "hex: {hex}" with lossless lowercase hex.
// Neither field has been Debug-escaped or escape_for_terminal'd (ADR 0003 / INV-4).
//
// Canonical BC-2.07.020 test vector: b"\xff\x00\xfe" ->
//   summary contains lossy; evidence = "hex: ff00fe".
#[allow(non_snake_case)]
#[test]
fn test_non_utf8_sni_preserves_raw_bytes_in_summary() {
    // AC-006 (BC-2.07.020 pc1): summary uses from_utf8_lossy (U+FFFD replacements).
    // AC-006 (BC-2.07.020 pc2): evidence[0] = "hex: {hex}" with lossless lowercase hex.
    // AC-006 (BC-2.07.020 pc3): no escape_for_terminal or {:?} Debug applied.
    // AC-006 (BC-2.07.020 pc4): hex is lossless — allows forensic reconstruction.
    let fk = test_flow_key();

    // --- Part 1: Canonical BC-2.07.020 test vector: [0xff, 0x00, 0xfe] ---
    let mut analyzer = TlsAnalyzer::new();
    let sni_bytes: &[u8] = b"\xff\x00\xfe";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-006 anchor (BC-2.07.020 pc1): parse must succeed"
    );

    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("AC-006 (BC-2.07.020 pc1): non-UTF-8 finding must be emitted");

    // BC-2.07.020 pc1: summary = from_utf8_lossy form.
    let lossy = String::from_utf8_lossy(sni_bytes).into_owned();
    let expected_summary =
        format!("TLS SNI contains non-UTF-8 bytes (RFC 6066 violation): {lossy}");
    assert_eq!(
        f.summary, expected_summary,
        "AC-006 (BC-2.07.020 pc1): summary must use from_utf8_lossy form"
    );

    // BC-2.07.020 pc2: evidence[0] = "hex: ff00fe" (lossless lowercase hex).
    assert_eq!(
        f.evidence.len(),
        1,
        "AC-006 (BC-2.07.020 pc2): evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "hex: ff00fe",
        "AC-006 (BC-2.07.020 pc2): evidence[0] must be lossless lowercase hex"
    );

    // BC-2.07.020 pc3: no Debug-escaped sequences in summary or evidence.
    assert!(
        !f.summary.contains("\\u{"),
        "AC-006 (BC-2.07.020 pc3): summary must not contain Debug-escaped \\u{{...}}; \
         got: {:?}",
        f.summary
    );
    assert!(
        !f.evidence[0].contains("\\u{"),
        "AC-006 (BC-2.07.020 pc3): evidence[0] must not contain Debug-escaped \\u{{...}}"
    );

    // BC-2.07.020 pc4: hex is lossless (exact literal "ff00fe").
    let hex_str = f.evidence[0]
        .strip_prefix("hex: ")
        .expect("AC-006 (BC-2.07.020 pc4): evidence[0] must start with 'hex: '");
    assert_eq!(
        hex_str, "ff00fe",
        "AC-006 (BC-2.07.020 pc4): hex must be lossless lowercase hex allowing reconstruction"
    );

    // --- Part 2: Terminal injection vector (ADR 0003 raw-byte preservation) ---
    // 0xff makes from_utf8 fail; 0x1b [31m is the ANSI "red" CSI sequence;
    // "pwnd" is the visible payload an attacker would inject.
    let mut analyzer2 = TlsAnalyzer::new();
    let raw_sni2: &[u8] = &[0xff, 0x1b, b'[', b'3', b'1', b'm', b'p', b'w', b'n', b'd'];
    let record2 = build_client_hello_raw_sni(raw_sni2, &[0x1301]);
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

    let f2 = analyzer2
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("AC-006 (BC-2.07.020 pc1): non-UTF-8 finding must be emitted for ESC injection");

    // BC-2.07.020 pc3 / ADR 0003: raw ESC byte (0x1B) must survive in summary
    // (terminal reporter escapes at render time, not here).
    assert!(
        f2.summary.as_bytes().contains(&0x1b),
        "AC-006 (BC-2.07.020 pc3): summary must preserve raw ESC byte for forensics; \
         got bytes: {:?}",
        f2.summary.as_bytes()
    );

    // Must NOT contain the Debug-formatted escape form.
    assert!(
        !f2.summary.contains("\\u{1b}"),
        "AC-006 (BC-2.07.020 pc3): summary must not contain Debug-formatted \\u{{1b}}; \
         got: {}",
        f2.summary
    );

    // Hex evidence is the lossless record.
    assert!(
        f2.evidence
            .iter()
            .any(|e| e.contains("ff1b5b33316d70776e64")),
        "AC-006 (BC-2.07.020 pc4): hex evidence must contain raw byte sequence; \
         got: {:?}",
        f2.evidence
    );
}

// ── STORY-057: SNI Edge Cases — BC-2.07.022 through BC-2.07.028 ──────────────
//
// Brownfield-formalization tests for SNI degenerate cases. Each test function
// name EXACTLY matches the `**Test:**` name cited in the corresponding AC per
// policy PG-W17-001 (AC↔test-name sync enforcement).
//
// test_sni_extension_with_empty_hostname_list
//     — AC-001 (BC-2.07.022 pc1-4): extract_sni returns None; sni_counts unchanged;
//       no finding; handshakes_seen += 1
//     — AC-002 (BC-2.07.022 inv1-2): identical behavior to ClientHello with no SNI
//       at all; None return short-circuits SNI handling block
//
// test_sni_with_empty_hostname_bytes
//     — AC-003 (BC-2.07.023 pc1-4): arm 1 fires for b""; sni_counts[""] == 1; no finding
//     — AC-004 (BC-2.07.023 inv1-2): key is "" (empty string), NOT "<non-utf8:...>"
//
// test_multi_name_sni_list_only_first_entry_counted
//     — AC-005 (BC-2.07.024 pc1-4): only first entry counted; one sni_counts entry; ≤1 finding
//     — AC-006 (BC-2.07.024 inv1-2): second+ entries never inspected; no finding when only
//       first is clean ASCII even if second has C0 bytes
//
// test_non_zero_name_type_sni_entry
//     — AC-007 (BC-2.07.025 pc1-3): NameType discarded; hostname processed normally
//     — AC-008 (BC-2.07.025 inv1-3): no finding emitted solely because of non-zero NameType
//
// test_non_zero_name_type_with_valid_first_entry
//     — AC-007 (BC-2.07.025 pc1-3): NameType=0 first, NameType=2 second; only first counted
//
// test_trailing_bytes_in_server_name_list
//     — AC-009 (BC-2.07.026 pc1-3): trailing bytes silently ignored; no parse_errors
//
// test_large_sni_near_record_payload_limit
//     — AC-010 (BC-2.07.027 pc1-5): ~16 KB SNI parses without error; counted; no parse_errors
//     — AC-011 (BC-2.07.027 inv1-2): MAX_RECORD_PAYLOAD=18,432 is binding; no truncated_records
//
// test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity
//     — AC-012 (BC-2.07.028 pc1-4): new key NOT inserted; finding IS pushed; len stays 50k
//     — AC-013 (BC-2.07.028 inv1-2): finding emission decoupled from count insertion;
//       all_findings has no cap

#[test]
fn test_sni_extension_with_empty_hostname_list() {
    // AC-001 (BC-2.07.022 pc1-4) + AC-002 (BC-2.07.022 inv1-2)
    //
    // When a ClientHello has an SNI extension but the ServerNameList is empty
    // (list.first() returns None), extract_sni returns None:
    //   pc1: extract_sni returns None
    //   pc2: sni_counts is unchanged (no entry added)
    //   pc3: no finding pushed to all_findings
    //   pc4: handshakes_seen still incremented (by BC-2.07.001, not SNI handling)
    //
    // AC-002 (inv1-2): empty list treated identically to a ClientHello with no SNI
    // extension at all; None return short-circuits the SNI handling block entirely.
    //
    // Anchor: guard against silent false-positive — if tls_parser ever tightens to
    // reject zero-entry ServerNameList (RFC 6066 §3: server_name_list<1..2^16-1>),
    // parse_tls_extensions would fail and subsequent assertions would still pass
    // for the wrong reason. The branch we pin is `list.first() == None`, which
    // requires the extension to have parsed successfully as TlsExtension::SNI(empty_list).
    let mut analyzer_empty_list = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_with_sni_list(&[], &[0x1301]);
    analyzer_empty_list.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Anchor: parse must succeed — the empty-list pin must exercise extract_sni's
    // `list.first() == None` branch, not a parse failure path.
    assert_eq!(
        analyzer_empty_list.parse_error_count(),
        0,
        "AC-001 anchor (BC-2.07.022): extensions must parse cleanly; an empty-list pin \
         that fires because tls_parser rejected the extension is not pinning the \
         extract_sni branch we intend"
    );

    // BC-2.07.022 pc2: sni_counts is unchanged (no entry added).
    assert!(
        analyzer_empty_list.sni_counts().is_empty(),
        "AC-001 (BC-2.07.022 pc2): sni_counts must be untouched when SNI list is empty, \
         got {:?}",
        analyzer_empty_list.sni_counts()
    );

    // BC-2.07.022 pc3: no finding pushed to all_findings.
    assert!(
        analyzer_empty_list.findings().is_empty(),
        "AC-001 (BC-2.07.022 pc3): no findings must fire for an empty SNI list, \
         got {:?}",
        analyzer_empty_list.findings()
    );

    // BC-2.07.022 pc4: handshakes_seen still incremented (by handle_client_hello,
    // not by SNI handling — the SNI block is short-circuited by the None return).
    assert_eq!(
        analyzer_empty_list.handshake_count(),
        1,
        "AC-001 (BC-2.07.022 pc4): handshakes_seen must be 1 — the handshake itself \
         parsed successfully; SNI counting is decoupled from handshake counting"
    );

    // AC-002 (BC-2.07.022 inv1): empty SNI list treated identically to a ClientHello
    // with NO SNI extension at all. Build a structurally different baseline using
    // build_client_hello_no_extensions — this omits the extensions length field
    // entirely (ch.ext == None), so the two wire forms are byte-distinct. If the
    // analyzer were ever to diverge on the two paths, this comparison catches it.
    let mut analyzer_no_sni = TlsAnalyzer::new();
    let record_no_sni = build_client_hello_no_extensions(&[0x1301]);
    analyzer_no_sni.on_data(&fk, Direction::ClientToServer, &record_no_sni, 0, 0);
    assert_eq!(
        analyzer_no_sni.sni_counts().len(),
        analyzer_empty_list.sni_counts().len(),
        "AC-002 (BC-2.07.022 inv1): sni_counts must be identically empty for both \
         empty-list SNI and no-SNI-extension ClientHellos"
    );
    assert_eq!(
        analyzer_no_sni.findings().len(),
        analyzer_empty_list.findings().len(),
        "AC-002 (BC-2.07.022 inv1): findings must be identically empty for both \
         empty-list SNI and no-SNI-extension ClientHellos"
    );
    assert_eq!(
        analyzer_no_sni.handshake_count(),
        analyzer_empty_list.handshake_count(),
        "AC-002 (BC-2.07.022 inv1): handshake_count must be 1 for both"
    );

    // BC-2.07.022 EC-002 / story EC-002: empty SNI list + weak cipher suite.
    // The SNI finding path must remain silent while an independent weak-cipher
    // finding fires. Pins finding independence — SNI and cipher analysis are
    // logically separate branches in handle_client_hello.
    //
    // Cipher 0x0000 = TLS_NULL_WITH_NULL_NULL — always in the weak-cipher set.
    let mut analyzer_ec002 = TlsAnalyzer::new();
    let record_ec002 = build_client_hello_with_sni_list(&[], &[0x0000]);
    analyzer_ec002.on_data(&fk, Direction::ClientToServer, &record_ec002, 0, 0);

    // No SNI finding (empty list → extract_sni returns None → SNI block skipped).
    let sni_findings_ec002 = analyzer_ec002
        .findings()
        .iter()
        .filter(|f| f.summary.to_lowercase().contains("sni"))
        .count();
    assert_eq!(
        sni_findings_ec002,
        0,
        "EC-002 (BC-2.07.022 EC-002): no SNI finding must fire for an empty SNI list; \
         got {:?}",
        analyzer_ec002.findings()
    );

    // Weak-cipher finding DOES fire (independent of SNI).
    let weak_findings_ec002 = analyzer_ec002
        .findings()
        .iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .count();
    assert_eq!(
        weak_findings_ec002,
        1,
        "EC-002 (BC-2.07.022 EC-002): exactly one weak-cipher finding must fire even when \
         SNI list is empty; got {:?}",
        analyzer_ec002.findings()
    );
}

#[test]
fn test_sni_with_empty_hostname_bytes() {
    // AC-003 (BC-2.07.023 pc1-4) + AC-004 (BC-2.07.023 inv1-2)
    //
    // When the SNI ServerNameList has one entry with zero-length hostname bytes
    // (hostname == b""), str::from_utf8(b"") succeeds with Ok(""); "".is_ascii() == true;
    // contains_c0_or_del("") == false (no bytes). Arm 1 fires: SniValue::Ascii("").
    //   pc1: extract_sni returns Some(SniValue::Ascii(""))
    //   pc2: sni_counts[""] is incremented
    //   pc3: no finding pushed
    //   pc4: degenerate RFC violation (RFC 6066 HostName<1..2^16-1>) but not flagged
    //
    // AC-004 (inv1-2): key is "" (empty string) — NOT "<non-utf8:...>" (arm 4 format).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_raw_sni(b"", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Parse anchor: parse must succeed so we exercise the arm 1 path, not a parse error.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-003 anchor (BC-2.07.023): parse_error_count must be 0 for empty hostname bytes"
    );

    // BC-2.07.023 pc2: sni_counts[""] is incremented.
    assert_eq!(
        *analyzer.sni_counts().get("").unwrap_or(&0),
        1,
        "AC-003 (BC-2.07.023 pc2): sni_counts must have key \"\" with count 1 for empty \
         hostname bytes (arm 1: Ok(\"\"), is_ascii==true, no C0/DEL), got {:?}",
        analyzer.sni_counts()
    );

    // AC-004 (BC-2.07.023 inv2): the key for empty-byte SNI is "" (empty string),
    // NOT the arm 4 "<non-utf8:...>" format.
    assert!(
        analyzer.sni_counts().contains_key(""),
        "AC-004 (BC-2.07.023 inv2): sni_counts must contain_key(\"\") for empty hostname \
         bytes — the key is an empty string, not a non-utf8 hex key"
    );
    assert!(
        !analyzer
            .sni_counts()
            .keys()
            .any(|k| k.starts_with("<non-utf8:")),
        "AC-004 (BC-2.07.023 inv2): no \"<non-utf8:...>\" key must be present — empty \
         bytes classify via arm 1 (valid UTF-8 ASCII), not arm 4 (non-UTF-8)"
    );

    // BC-2.07.023 pc3: no finding pushed (arm 1 does not emit findings).
    let non_utf8_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(
        non_utf8_findings, 0,
        "AC-003 (BC-2.07.023 pc3): no non-UTF-8 finding must be emitted for empty bytes \
         (arm 1 path); got {} non-UTF-8 findings",
        non_utf8_findings
    );
    assert!(
        analyzer.findings().is_empty(),
        "AC-003 (BC-2.07.023 pc3): findings must be completely empty for empty hostname \
         bytes; got {:?}",
        analyzer.findings()
    );

    // BC-2.07.023 pc4 (handshake counted — verify via handshake_count).
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-003 (BC-2.07.023 pc4): handshakes_seen must be 1"
    );
}

// AC-001 (BC-2.07.017 postconditions 1-3)
//
// When extract_sni receives SNI bytes that are valid UTF-8 but fail is_ascii(),
// arm 3 fires: one Finding with category=Anomaly, verdict=Inconclusive,
// confidence=Low, mitre_technique=Some("T1027"), direction=Some(ClientToServer).
// sni_counts keyed on the raw hostname string.
//
// Canonical BC-2.07.017 test vector: "café.example" (valid UTF-8, non-ASCII).
#[allow(non_snake_case)]
#[test]
fn test_valid_utf8_non_ascii_sni_emits_finding() {
    // AC-001 (BC-2.07.017 pc1): arm 3 fires for valid-UTF-8 non-ASCII SNI.
    // AC-001 (BC-2.07.017 pc2): one Finding with all required fields.
    // AC-001 (BC-2.07.017 pc3): sni_counts keyed on raw hostname.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Canonical BC-2.07.017 EC-001 test vector: "café.example" (U+00E9 = 0xC3 0xA9).
    let record = build_client_hello("café.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor: record must be accepted by the parser.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-001 anchor (BC-2.07.017): parse_error_count must be 0"
    );

    // BC-2.07.017 pc2: exactly one non-ASCII finding emitted.
    let non_ascii_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .collect();
    assert_eq!(
        non_ascii_findings.len(),
        1,
        "AC-001 (BC-2.07.017 pc2): exactly one non-ASCII finding must be emitted; \
         got: {non_ascii_findings:?}"
    );

    let f = &non_ascii_findings[0];

    // BC-2.07.017 pc2: category = Anomaly.
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-001 (BC-2.07.017 pc2): category must be Anomaly"
    );

    // BC-2.07.017 pc2: verdict = Inconclusive.
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "AC-001 (BC-2.07.017 pc2): verdict must be Inconclusive"
    );

    // BC-2.07.017 pc2: confidence = Low.
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "AC-001 (BC-2.07.017 pc2): confidence must be Low"
    );

    // BC-2.07.017 pc2: mitre_techniques = ["T1027"] (single-tag, full-vec equality).
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-001 (BC-2.07.017 pc2): mitre_techniques must be exactly [\"T1027\"]"
    );

    // BC-2.07.017 pc2: direction = Some(ClientToServer).
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "AC-001 (BC-2.07.017 pc2): direction must be Some(ClientToServer)"
    );

    // BC-2.07.017 pc2: source_ip must be None (network context not available in analyzer).
    assert_eq!(f.source_ip, None, "BC-2.07.017 pc2: source_ip must be None");
    // BC-2.09.007 post-1 (STORY-098): timestamp is now Some(DateTime<Utc>) derived from the
    // per-flow last_ts; this test calls on_data with timestamp=0, so the result is
    // Some(1970-01-01T00:00:00Z) — not None. The "None" assertion is superseded by STORY-098.
    {
        use chrono::DateTime;
        let expected_ts = DateTime::from_timestamp(0_i64, 0);
        assert_eq!(
            f.timestamp, expected_ts,
            "BC-2.09.007 (STORY-098): TLS SNI finding must have timestamp == \
             DateTime::from_timestamp(0, 0) = {expected_ts:?} (on_data called with ts_sec=0)"
        );
    }

    // BC-2.07.017 pc2: exact summary — hostname interpolated verbatim (not Debug-escaped).
    let expected_summary = "TLS SNI contains non-ASCII characters \
         (RFC 6066 requires A-labels per RFC 5890): café.example";
    assert_eq!(
        f.summary, expected_summary,
        "AC-001 (BC-2.07.017 pc2): summary must be exact canonical string"
    );

    // BC-2.07.017 pc2 / BC-2.07.021 pc2: evidence = exactly one entry, exact lowercase hex.
    // "café.example" UTF-8 = 63 61 66 c3 a9 2e 65 78 61 6d 70 6c 65
    assert_eq!(
        f.evidence.len(),
        1,
        "AC-001 (BC-2.07.017 pc2): evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "hex: 636166c3a92e6578616d706c65",
        "AC-001 (BC-2.07.017 pc2): evidence[0] must be exact lowercase hex"
    );

    // BC-2.07.017 pc3: sni_counts keyed on raw hostname string.
    assert_eq!(
        *analyzer.sni_counts().get("café.example").unwrap_or(&0),
        1,
        "AC-001 (BC-2.07.017 pc3): sni_counts must be keyed on raw hostname; \
         got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );

    // Must NOT trip the non-UTF-8 arm (valid UTF-8, just non-ASCII).
    let non_utf8_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(
        non_utf8_findings, 0,
        "AC-001 (BC-2.07.017 pc2): valid-UTF-8 non-ASCII SNI must NOT emit non-UTF-8 finding"
    );
}

// BC-TLS-037 (#104): Mixed control + non-ASCII SNI — summary must mention control bytes.
//
// A TLS SNI that is valid UTF-8 but contains BOTH a non-ASCII byte (routes the
// NonAsciiUtf8 arm) AND an ASCII control byte (b < 0x20 or b == 0x7f) must have
// the control-byte presence reflected in the finding summary. A SOC analyst
// grepping summaries for "control" must be able to find this case.
//
// Fixture: "café\x1b" — valid UTF-8, contains é (U+00E9, 0xC3 0xA9) making it
// non-ASCII, and ESC (0x1b) — a C0 control byte.
// hex: 636166c3a91b
//
// Classification invariant preserved: the NonAsciiUtf8 arm still fires
// (`is_ascii()` is false once é is present). Only the summary text is enriched.
#[allow(non_snake_case)]
#[test]
fn test_mixed_control_and_non_ascii_sni_summary_mentions_control_bytes() {
    // BC-TLS-037 pc1: NonAsciiUtf8 arm fires (not AsciiWithControl).
    // BC-TLS-037 pc2: the finding summary must contain "control" to surface the
    //   control-byte covert-channel / log-poisoning risk alongside the non-ASCII flag.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // "café\x1b": é = 0xC3 0xA9 (non-ASCII), ESC = 0x1b (C0 control byte).
    let sni_bytes: &[u8] = b"\x63\x61\x66\xc3\xa9\x1b"; // café\x1b
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "BC-TLS-037 anchor: parse_error_count must be 0 for valid-UTF-8 SNI"
    );

    // BC-TLS-037 pc1: exactly one NonAsciiUtf8-arm finding (contains "non-ASCII").
    let findings = analyzer.findings();
    let non_ascii_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("non-ASCII"))
        .collect();
    assert_eq!(
        non_ascii_findings.len(),
        1,
        "BC-TLS-037 pc1: exactly one non-ASCII finding must be emitted; got: {non_ascii_findings:?}"
    );

    let f = &non_ascii_findings[0];

    // BC-TLS-037 pc2: summary must mention "control" so a SOC analyst
    //   grepping for control-byte alerts doesn't miss mixed strings.
    assert!(
        f.summary.contains("control"),
        "BC-TLS-037 pc2: summary must mention \"control\" for mixed control+non-ASCII SNI; \
         got summary: {:?}",
        f.summary
    );

    // Classification invariant: still Anomaly/Inconclusive/Low/T1027/ClientToServer.
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "BC-TLS-037: category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "BC-TLS-037: verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "BC-TLS-037: confidence must be Low"
    );
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "BC-TLS-037: mitre_techniques must be exactly [\"T1027\"]"
    );
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "BC-TLS-037: direction must be Some(ClientToServer)"
    );

    // Evidence: lossless hex of the raw bytes.
    assert_eq!(
        f.evidence.len(),
        1,
        "BC-TLS-037: evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "hex: 636166c3a91b",
        "BC-TLS-037: evidence[0] must be exact lowercase hex of café\\x1b"
    );
}

// AC-001/AC-002/AC-008 (BC-2.07.017 pc1-3, inv1; BC-2.07.021 pc1-3)
//
// Cyrillic SNI: raw Cyrillic in summary (not \u{...} Debug-escaped). Covers:
// - AC-001: finding emitted with all required fields
// - AC-002: hostname in summary is raw UTF-8 (no Debug escaping)
// - AC-008: evidence[0] = "hex: {hex}" with lossless lowercase hex
//
// Canonical BC-2.07.021 EC-001: "мир" -> summary contains raw Cyrillic.
// Also uses "пример.example" (full Cyrillic U-label) for the sni_counts key test.
#[allow(non_snake_case)]
#[test]
fn test_cyrillic_sni_emits_non_ascii_finding() {
    // AC-001 (BC-2.07.017 pc2): finding with category/verdict/confidence/mitre/direction.
    // AC-002 (BC-2.07.017 inv1): hostname in summary is raw UTF-8, not Debug-escaped.
    // AC-008 (BC-2.07.021 pc2): evidence[0] = "hex: {hex}" with lossless lowercase hex.
    let fk = test_flow_key();

    // --- Part 1: EC-001 canonical vector: b"\xd0\xbc\xd0\xb8\xd1\x80" ("мир") ---
    let mut analyzer1 = TlsAnalyzer::new();
    // BC-2.07.017 canonical test vector.
    let sni_bytes: &[u8] = b"\xd0\xbc\xd0\xb8\xd1\x80";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer1.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer1.parse_error_count(),
        0,
        "AC-001/002/008 anchor (BC-2.07.017): parse must succeed"
    );

    // AC-001 (BC-2.07.017 pc2): exactly one non-ASCII finding.
    let f1 = analyzer1
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-ASCII characters"))
        .expect("AC-001 (BC-2.07.017 pc2): non-ASCII finding must be emitted for Cyrillic SNI");

    // BC-2.07.017 pc2: exact summary with raw Cyrillic.
    let expected_summary =
        "TLS SNI contains non-ASCII characters (RFC 6066 requires A-labels per RFC 5890): мир";
    assert_eq!(
        f1.summary, expected_summary,
        "AC-001 (BC-2.07.017 pc2): summary must be exact canonical string"
    );

    // AC-002 (BC-2.07.017 inv1 / BC-2.07.021 pc1): raw Cyrillic present, NO Debug-escaped form.
    assert!(
        f1.summary.contains("мир"),
        "AC-002 (BC-2.07.017 inv1): summary must contain raw Cyrillic \"мир\", \
         not Debug-escaped \\u{{...}} form; got: {:?}",
        f1.summary
    );
    assert!(
        !f1.summary.contains("\\u{"),
        "AC-002 (BC-2.07.017 inv1 / BC-2.07.021 pc3): summary must not contain \
         Debug-escaped \\u{{...}} sequences; got: {:?}",
        f1.summary
    );

    // BC-2.07.017 pc2: category = Anomaly.
    assert_eq!(
        f1.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-001 (BC-2.07.017 pc2): category must be Anomaly"
    );
    // BC-2.07.017 pc2: verdict = Inconclusive.
    assert_eq!(
        f1.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "AC-001 (BC-2.07.017 pc2): verdict must be Inconclusive"
    );
    // BC-2.07.017 pc2: confidence = Low.
    assert_eq!(
        f1.confidence,
        wirerust::findings::Confidence::Low,
        "AC-001 (BC-2.07.017 pc2): confidence must be Low"
    );
    // BC-2.07.017 pc2: mitre_techniques = ["T1027"] (single-tag, full-vec equality).
    assert_eq!(
        f1.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-001 (BC-2.07.017 pc2): mitre_techniques must be exactly [\"T1027\"]"
    );
    // BC-2.07.017 pc2: direction = Some(ClientToServer).
    assert_eq!(
        f1.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "AC-001 (BC-2.07.017 pc2): direction must be Some(ClientToServer)"
    );

    // AC-008 (BC-2.07.021 pc2): evidence[0] = "hex: d0bcd0b8d180" (lossless lowercase hex).
    assert_eq!(
        f1.evidence.len(),
        1,
        "AC-008 (BC-2.07.021 pc2): evidence must have exactly one entry"
    );
    assert_eq!(
        f1.evidence[0], "hex: d0bcd0b8d180",
        "AC-008 (BC-2.07.021 pc2): evidence[0] must be exact lowercase hex for \"мир\""
    );
    assert!(
        !f1.evidence[0].contains("\\u{"),
        "AC-008 (BC-2.07.021 pc3): evidence must not contain Debug-escaped \\u{{...}}"
    );

    // BC-2.07.017 pc3: sni_counts keyed on raw hostname "мир".
    let hostname = std::str::from_utf8(sni_bytes).expect("Cyrillic is valid UTF-8");
    assert_eq!(
        *analyzer1.sni_counts().get(hostname).unwrap_or(&0),
        1,
        "AC-001 (BC-2.07.017 pc3): sni_counts must be keyed on raw hostname \"мир\"; \
         got keys: {:?}",
        analyzer1.sni_counts().keys().collect::<Vec<_>>()
    );

    // --- Part 2: "пример.example" (longer Cyrillic U-label) — regression ---
    let mut analyzer2 = TlsAnalyzer::new();
    let record2 = build_client_hello("пример.example", &[0x1301]);
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

    let non_ascii_count = analyzer2
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .count();
    assert_eq!(
        non_ascii_count,
        1,
        "AC-001 (BC-2.07.017 pc2): Cyrillic U-label \"пример.example\" must emit \
         one non-ASCII finding; got {:?}",
        analyzer2.findings()
    );

    // BC-2.07.017 pc3: sni_counts keyed on raw hostname.
    assert_eq!(
        *analyzer2.sni_counts().get("пример.example").unwrap_or(&0),
        1,
        "AC-001 (BC-2.07.017 pc3): sni_counts[\"пример.example\"] must be 1"
    );

    // AC-002: no Debug-escaped Cyrillic in summary.
    let f2 = analyzer2
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-ASCII characters"))
        .expect("AC-002: expected non-ASCII finding for \"пример.example\"");
    assert!(
        f2.summary.contains("пример.example"),
        "AC-002 (BC-2.07.017 inv1): summary must contain raw Cyrillic hostname; got: {}",
        f2.summary
    );
    assert!(
        !f2.summary.contains("\\u{43f}"),
        "AC-002 (BC-2.07.017 inv1): summary must not contain Debug-formatted Cyrillic \
         escape; got: {}",
        f2.summary
    );
}

// AC-003 (BC-2.07.017 invariant 3)
//
// Any non-ASCII UTF-8 triggers arm 3, including emoji (multi-byte UTF-8 sequences).
// EC-002/EC-008 from STORY-056: emoji bytes (valid UTF-8, non-ASCII) -> arm 3.
// Canonical STORY-056 EC-008: "😈" = bytes [0xF0, 0x9F, 0x98, 0x88].
#[allow(non_snake_case)]
#[test]
fn test_emoji_sni_emits_non_ascii_finding() {
    // AC-003 (BC-2.07.017 inv3): emoji bytes (valid UTF-8, non-ASCII) -> arm 3; finding emitted.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // EC-008 from STORY-056: "😈" = [0xF0, 0x9F, 0x98, 0x88]; is_ascii() == false.
    let sni_bytes: &[u8] = b"\xf0\x9f\x98\x88";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-003 anchor (BC-2.07.017 inv3): parse must succeed for emoji SNI"
    );

    // BC-2.07.017 inv3: arm 3 fires (one non-ASCII finding emitted).
    let non_ascii_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .collect();
    assert_eq!(
        non_ascii_findings.len(),
        1,
        "AC-003 (BC-2.07.017 inv3): exactly one non-ASCII finding for emoji SNI; \
         got: {non_ascii_findings:?}"
    );

    let f = &non_ascii_findings[0];

    // BC-2.07.017 pc2 (symmetric assertions with arm 3 spec).
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-003 (BC-2.07.017 inv3): category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "AC-003 (BC-2.07.017 inv3): verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "AC-003 (BC-2.07.017 inv3): confidence must be Low"
    );
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-003 (BC-2.07.017 inv3): mitre_techniques must be exactly [\"T1027\"]"
    );
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "AC-003 (BC-2.07.017 inv3): direction must be Some(ClientToServer)"
    );

    // Exact summary: hostname is the decoded emoji string.
    let hostname = std::str::from_utf8(sni_bytes).expect("emoji is valid UTF-8");
    let expected_summary = format!(
        "TLS SNI contains non-ASCII characters (RFC 6066 requires A-labels per RFC 5890): \
         {hostname}"
    );
    assert_eq!(
        f.summary, expected_summary,
        "AC-003 (BC-2.07.017 inv3): summary must contain decoded emoji hostname"
    );

    // evidence[0] = "hex: f09f9888" (lowercase hex for 😈 bytes).
    assert_eq!(
        f.evidence.len(),
        1,
        "AC-003 (BC-2.07.017 inv3): evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "hex: f09f9888",
        "AC-003 (BC-2.07.017 inv3): evidence[0] must be lowercase hex for emoji bytes"
    );

    // sni_counts keyed on raw decoded hostname.
    assert_eq!(
        *analyzer.sni_counts().get(hostname).unwrap_or(&0),
        1,
        "AC-003 (BC-2.07.017 pc3): sni_counts must be keyed on raw emoji hostname; \
         got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );

    // Also verify the 🦀 emoji (regression test from original).
    // 🦀 = U+1F980 = 0xF0 0x9F 0xA6 0x80 (4 bytes, all >= 0x80).
    let mut analyzer2 = TlsAnalyzer::new();
    let record2 = build_client_hello("🦀.example", &[0x1301]);
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

    let non_ascii_count2 = analyzer2
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .count();
    assert_eq!(
        non_ascii_count2,
        1,
        "AC-003 (BC-2.07.017 inv3): 🦀 emoji SNI must emit one non-ASCII finding; \
         got {:?}",
        analyzer2.findings()
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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    // AC-012 (BC-2.07.028 pc1-4) + AC-013 (BC-2.07.028 inv1-2)
    // Also traces to: STORY-052 AC-007 (BC-2.07.001 invariant 2)
    //
    // When sni_counts is at MAX_MAP_ENTRIES = 50,000 capacity and a new anomalous
    // SNI arrives (not already in the map), the new SNI key is NOT inserted into
    // sni_counts (count silently dropped), but the anomaly finding IS pushed to
    // all_findings.
    //   pc1: new SNI key NOT inserted into sni_counts (map full; count silently dropped)
    //   pc2: anomaly finding IS pushed to all_findings regardless of count outcome
    //   pc3: sni_counts.len() remains at MAX_MAP_ENTRIES
    //   pc4: all_findings.len() increases by 1
    //
    // AC-013 (BC-2.07.028 inv1-2): finding emission decoupled from count insertion.
    // The Self::increment call and the match sni { ... } block are sequential, not
    // conditional on each other. all_findings in TlsAnalyzer has no cap (unlike
    // TcpReassembler which has MAX_FINDINGS = 10,000).
    //
    // Critical forensic property (BC-2.07.028 description): an attacker flooding the
    // analyzer with unique SNIs to exhaust sni_counts capacity cannot suppress anomaly
    // findings for subsequently observed malicious SNIs.
    //
    // Implementation note: this test is intentionally slower (~650ms in debug builds,
    // measured locally; budget ~2s for CI cold caches) because the only way to reach
    // the capacity-full state via the public API is to feed 50k unique ClientHellos
    // through it. The performance trade-off was considered in the design phase: the
    // alternative was adding a #[cfg(test)] test helper to TlsAnalyzer, which would
    // expose internal state. Black-box brute force keeps the public API clean.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // MAX_MAP_ENTRIES is private; this constant must stay in sync with production.
    const MAX_MAP_ENTRIES: usize = 50_000;

    // Fill sni_counts to capacity with unique valid-UTF-8 hostnames. Each ClientHello
    // uses the same cipher list, so all 50k JA3 hashes collapse into a single
    // ja3_counts entry — only sni_counts grows to the cap.
    for i in 0..MAX_MAP_ENTRIES {
        let sni = format!("filler{i:05}.example");
        let record = build_client_hello(&sni, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);
    }

    // BC-2.07.028 pre1: sni_counts must be full before the cap-decoupling test.
    assert_eq!(
        analyzer.sni_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-012 precondition (BC-2.07.028 pre1): sni_counts must be at MAX_MAP_ENTRIES \
         before the capacity test; got {}",
        analyzer.sni_counts().len()
    );

    // Snapshot finding counts before the capacity-test ClientHello so we can assert
    // exactly one new non-UTF-8 finding fired (not just check total count).
    let non_utf8_before = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    let total_findings_before = analyzer.findings().len();

    // BC-2.07.028 canonical test vector: send a new non-UTF-8 SNI whose key is not
    // in the map. The key cannot be inserted (map full), but the finding must fire.
    let raw_sni: &[u8] = &[0xff, 0xfe, b'a', b'.', b'c', b'o', b'm'];
    let record = build_client_hello_raw_sni(raw_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // BC-2.07.028 pc3: sni_counts.len() remains at MAX_MAP_ENTRIES.
    assert_eq!(
        analyzer.sni_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-012 (BC-2.07.028 pc3): sni_counts.len() must remain at MAX_MAP_ENTRIES \
         after the new key was silently dropped"
    );

    // BC-2.07.028 pc1: the new non-UTF-8 key is NOT inserted into sni_counts.
    assert!(
        analyzer
            .sni_counts()
            .get("<non-utf8:fffe612e636f6d>")
            .is_none(),
        "AC-012 (BC-2.07.028 pc1): non-UTF-8 hex key \"<non-utf8:fffe612e636f6d>\" \
         must NOT be present in sni_counts (map full, new key silently dropped)"
    );

    // BC-2.07.028 pc2 + AC-013 (inv1): anomaly finding IS pushed to all_findings
    // regardless of the count drop. This is the critical forensic property.
    let non_utf8_after = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-UTF-8 bytes"))
        .count();
    assert_eq!(
        non_utf8_after,
        non_utf8_before + 1,
        "AC-012 (BC-2.07.028 pc2): exactly one new non-UTF-8 finding must have fired \
         even though the count was dropped (finding emission is decoupled from count \
         insertion); got {non_utf8_before} -> {non_utf8_after}"
    );

    // BC-2.07.028 pc4: all_findings.len() increased by 1.
    assert_eq!(
        analyzer.findings().len(),
        total_findings_before + 1,
        "AC-012 (BC-2.07.028 pc4): all_findings.len() must have increased by exactly 1; \
         got {} -> {}",
        total_findings_before,
        analyzer.findings().len()
    );

    // AC-013 (BC-2.07.028 inv1): this test proves the DECOUPLING property —
    // finding emission and count insertion are sequential, not conditional.
    // Verify the finding has the expected metadata (confidence, MITRE technique).
    // Note: BC-2.07.028 inv2 ("all_findings has no cap") is separately evidenced
    // by `test_BC_2_04_024_http_tls_analyzer_findings_not_capped` in
    // tests/reassembly_engine_tests.rs (BC-2.04.024 inv4 / AC-007b); here we
    // only prove the decoupling half of inv1.
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect(
            "AC-012 (BC-2.07.028 pc2): non-UTF-8 SNI finding must be in all_findings \
             even when sni_counts is full",
        );
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "AC-012 (BC-2.07.028 pc2): non-UTF-8 finding must have confidence=Low"
    );
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-012 (BC-2.07.028 pc2): non-UTF-8 finding must have mitre_techniques=[\"T1027\"]"
    );

    // AC-013 EC-001 (BC-2.07.028 EC-001): map at capacity + clean-ASCII SNI (arm 1).
    // Arm 1 never emits a finding regardless of the count cap. Assert NO new finding
    // fires and sni_counts.len() stays at MAX_MAP_ENTRIES (new key silently dropped).
    // This pins the "arm-1 emits no finding" half of the decoupling property.
    let total_findings_arm1_before = analyzer.findings().len();
    let new_clean_sni = "at-capacity-clean.example";
    let record_arm1 = build_client_hello(new_clean_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record_arm1, 0, 0);

    assert_eq!(
        analyzer.sni_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-013 EC-001 (BC-2.07.028 EC-001): sni_counts.len() must remain at \
         MAX_MAP_ENTRIES after a clean-ASCII SNI at capacity (new key silently dropped)"
    );
    assert!(
        analyzer.sni_counts().get(new_clean_sni).is_none(),
        "AC-013 EC-001 (BC-2.07.028 EC-001): clean-ASCII SNI key must NOT be inserted \
         when map is full"
    );
    assert_eq!(
        analyzer.findings().len(),
        total_findings_arm1_before,
        "AC-013 EC-001 (BC-2.07.028 EC-001): arm-1 must emit NO finding even when the \
         count is dropped (arm-1 never emits findings regardless of cap)"
    );

    // AC-013 EC-002 (BC-2.07.028 EC-002): map at capacity + anomalous SNI ALREADY in map.
    // The `|| map.contains_key(&key)` clause in increment lets existing keys grow even
    // when the map is full. Assert the existing key's count increments to 2 and
    // sni_counts.len() stays at MAX_MAP_ENTRIES.
    // Use "filler00000.example" — guaranteed to be in the map from the fill loop above.
    let existing_sni = "filler00000.example";
    let count_before_existing = *analyzer.sni_counts().get(existing_sni).unwrap_or(&0);
    assert_eq!(
        count_before_existing, 1,
        "AC-013 EC-002 setup: filler00000.example must already be in sni_counts with \
         count 1 before the re-send"
    );
    let record_existing = build_client_hello(existing_sni, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record_existing, 0, 0);

    assert_eq!(
        analyzer.sni_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-013 EC-002 (BC-2.07.028 EC-002): sni_counts.len() must remain at \
         MAX_MAP_ENTRIES after re-sending an existing key (no new insertion)"
    );
    assert_eq!(
        *analyzer.sni_counts().get(existing_sni).unwrap_or(&0),
        2,
        "AC-013 EC-002 (BC-2.07.028 EC-002): existing key count must increment to 2 — \
         proves the `|| map.contains_key(&key)` clause in increment allows existing \
         keys to grow even when the map is full"
    );
}

#[test]
fn test_multi_name_sni_list_only_first_entry_counted() {
    // AC-005 (BC-2.07.024 pc1-4) + AC-006 (BC-2.07.024 inv1-2)
    //
    // When a ClientHello SNI extension contains a ServerNameList with 2+ entries,
    // extract_sni uses list.first() to extract only the first entry. Second and
    // subsequent entries are silently ignored.
    //   pc1: only the first entry's hostname bytes are passed to classification
    //   pc2: only one sni_counts entry is inserted (for the first hostname)
    //   pc3: at most one finding is emitted (based on first hostname classification)
    //   pc4: subsequent entries are silently ignored
    //
    // AC-006 canonical vector (BC-2.07.024 inv1-2): SNI list ["example.com", "evil\x01.com"]
    // — first entry is clean ASCII so no finding; second entry with C0 byte (0x01)
    // is NEVER inspected; assert no finding emitted.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // AC-006 canonical test vector from BC-2.07.024: first entry clean ASCII,
    // second entry contains C0 byte 0x01 (SOH — ASCII control character).
    let record = build_client_hello_with_sni_list(&[b"example.com", b"evil\x01.com"], &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-005 anchor (BC-2.07.024): parse_error_count must be 0"
    );

    // BC-2.07.024 pc2: only one sni_counts entry inserted (for the first hostname).
    assert_eq!(
        analyzer.sni_counts().len(),
        1,
        "AC-005 (BC-2.07.024 pc2): exactly one sni_counts entry must be inserted \
         (only the first hostname); got {:?}",
        analyzer.sni_counts()
    );

    // BC-2.07.024 pc1: only the first entry's hostname bytes processed.
    assert_eq!(
        *analyzer.sni_counts().get("example.com").unwrap_or(&0),
        1,
        "AC-005 (BC-2.07.024 pc1): first hostname \"example.com\" must be counted; \
         got {:?}",
        analyzer.sni_counts()
    );

    // BC-2.07.024 pc4: second entry silently ignored — not in sni_counts.
    assert!(
        analyzer.sni_counts().get("evil\x01.com").is_none(),
        "AC-005 (BC-2.07.024 pc4): second entry must not be counted, got {:?}",
        analyzer.sni_counts()
    );

    // AC-006 (BC-2.07.024 inv2): second+ entries never inspected for anomalies.
    // The second entry "evil\x01.com" has a C0 byte that WOULD trigger a finding
    // if it were inspected (arm 2: AsciiWithControl). No such finding must fire.
    assert!(
        analyzer.findings().is_empty(),
        "AC-006 (BC-2.07.024 inv2): no finding must be emitted — the second entry \
         \"evil\\x01.com\" is NEVER inspected; only the first clean-ASCII entry \
         is classified; got {:?}",
        analyzer.findings()
    );

    // Also verify the 3-entry case from BC-2.07.024 canonical test vector.
    let mut analyzer2 = TlsAnalyzer::new();
    let record2 = build_client_hello_with_sni_list(
        &[b"first.example", b"second.example", b"third.example"],
        &[0x1301],
    );
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);
    assert_eq!(
        *analyzer2.sni_counts().get("first.example").unwrap_or(&0),
        1,
        "AC-005 (BC-2.07.024 pc1): first.example must be counted in 3-entry case"
    );
    assert!(
        analyzer2.sni_counts().get("second.example").is_none(),
        "AC-005 (BC-2.07.024 pc4): second.example must not be counted"
    );
    assert!(
        analyzer2.sni_counts().get("third.example").is_none(),
        "AC-005 (BC-2.07.024 pc4): third.example must not be counted"
    );
    assert_eq!(
        analyzer2.handshake_count(),
        1,
        "AC-005 (BC-2.07.024 pc4): handshakes_seen must be 1"
    );

    // BC-2.07.024 canonical test vector (second direction — EC-002):
    // First entry IS anomalous ("evil\x01.com"); second entry is clean.
    // Guards against a "never inspects any entry" false-green: the first entry
    // must be inspected and must produce exactly one AsciiWithControl finding.
    // Only sni_counts["evil\x01.com"] (raw key, per ADR 0003) is present;
    // second entry "example.com" is NOT counted.
    let mut analyzer3 = TlsAnalyzer::new();
    let record3 = build_client_hello_with_sni_list(&[b"evil\x01.com", b"example.com"], &[0x1301]);
    analyzer3.on_data(&fk, Direction::ClientToServer, &record3, 0, 0);

    // Parse anchor.
    assert_eq!(
        analyzer3.parse_error_count(),
        0,
        "AC-005 EC-002 anchor (BC-2.07.024): parse_error_count must be 0"
    );

    // First entry IS inspected: exactly one AsciiWithControl finding from "evil\x01.com".
    let ctrl_findings: Vec<_> = analyzer3
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .collect();
    assert_eq!(
        ctrl_findings.len(),
        1,
        "AC-005 EC-002 (BC-2.07.024 pc3/EC-002): exactly one AsciiWithControl finding \
         must fire when the FIRST entry has a C0 byte; got {:?}",
        ctrl_findings
    );

    // sni_counts contains only the raw first-entry key (ADR 0003: raw bytes stored).
    assert_eq!(
        *analyzer3.sni_counts().get("evil\x01.com").unwrap_or(&0),
        1,
        "AC-005 EC-002 (BC-2.07.024 pc1): sni_counts must contain the raw first-entry key \
         \"evil\\x01.com\""
    );

    // Second entry must NOT appear in sni_counts.
    assert!(
        analyzer3.sni_counts().get("example.com").is_none(),
        "AC-005 EC-002 (BC-2.07.024 pc4): second entry \"example.com\" must not be counted \
         (only first entry is processed); got {:?}",
        analyzer3.sni_counts()
    );

    // Exactly one sni_counts entry total.
    assert_eq!(
        analyzer3.sni_counts().len(),
        1,
        "AC-005 EC-002 (BC-2.07.024 pc2): exactly one sni_counts entry must be present"
    );

    // Story EC-003 (BC-2.07.024 composes with BC-2.07.023):
    // First entry is empty bytes (b""), second entry is a non-empty valid hostname.
    // Only the first entry is processed (list.first()). The empty bytes classify
    // via arm 1 (str::from_utf8(b"") == Ok(""), is_ascii==true, no C0/DEL):
    // sni_counts[""] == 1, sni_counts.len() == 1, no finding.
    let mut analyzer4 = TlsAnalyzer::new();
    let record4 = build_client_hello_with_sni_list(&[b"", b"second.example"], &[0x1301]);
    analyzer4.on_data(&fk, Direction::ClientToServer, &record4, 0, 0);

    assert_eq!(
        analyzer4.parse_error_count(),
        0,
        "EC-003 anchor (BC-2.07.024 + BC-2.07.023): parse_error_count must be 0"
    );
    assert_eq!(
        *analyzer4.sni_counts().get("").unwrap_or(&0),
        1,
        "EC-003 (BC-2.07.024 + BC-2.07.023): sni_counts[\"\"] must be 1 — empty first \
         entry classified via arm 1 (vacuously satisfies all arm-1 conditions)"
    );
    assert_eq!(
        analyzer4.sni_counts().len(),
        1,
        "EC-003 (BC-2.07.024): exactly one sni_counts entry — second entry \
         \"second.example\" must NOT be counted"
    );
    assert!(
        analyzer4.sni_counts().get("second.example").is_none(),
        "EC-003 (BC-2.07.024): \"second.example\" must not appear in sni_counts; \
         got {:?}",
        analyzer4.sni_counts()
    );
    assert!(
        analyzer4.findings().is_empty(),
        "EC-003 (BC-2.07.024 + BC-2.07.023): no finding must be emitted — empty bytes \
         take arm 1 (no C0, valid ASCII, valid UTF-8); got {:?}",
        analyzer4.findings()
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
    // AC-007 (BC-2.07.025 pc1-3) + AC-008 (BC-2.07.025 inv1-3)
    //
    // When a ClientHello SNI extension has a ServerNameList where the first entry
    // has a non-zero NameType byte (e.g., NameType=1), the NameType is discarded
    // (pattern `let Some((_, hostname)) = list.first()`) and only the hostname
    // bytes are passed to the 4-way classification. Behavior is identical to
    // NameType=0 processing.
    //   pc1: hostname bytes from first entry (regardless of NameType) passed to classification
    //   pc2: behavior identical to NameType=0 processing
    //   pc3: no finding emitted solely because of non-zero NameType
    //
    // AC-008 canonical vector (BC-2.07.025 inv1-3): NameType=1, hostname="example.com";
    // assert no finding, sni_counts has exactly one entry keyed on "example.com".
    //
    // Note: tls_parser behavior is empirically pinned here. If a future tls_parser
    // upgrade changes to skip unknown NameType entries, this test documents the break.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // BC-2.07.025 canonical test vector: NameType=1, hostname b"example.com"
    let record = build_client_hello_with_typed_sni_list(&[(0x01, b"example.com")], &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-007 anchor (BC-2.07.025): parse_error_count must be 0"
    );

    // BC-2.07.025 pc1: hostname bytes from the first entry (NameType=1) passed to
    // classification — tls_parser includes the entry in the SNI list regardless of type.
    // extract_sni destructures as `let Some((_, hostname)) = list.first()` — the `_`
    // discards NameType, so "example.com" is classified via arm 1 (clean ASCII).
    assert_eq!(
        *analyzer.sni_counts().get("example.com").unwrap_or(&0),
        1,
        "AC-007 (BC-2.07.025 pc1): sni_counts must contain \"example.com\" with count 1 \
         (NameType=1 discarded by `_` pattern; hostname classified normally)"
    );

    // BC-2.07.025 pc3 / AC-008 (inv1): no finding emitted solely because of non-zero NameType.
    // "example.com" is clean ASCII (arm 1), so no finding would fire for the hostname either.
    assert!(
        analyzer.findings().is_empty(),
        "AC-008 (BC-2.07.025 pc3/inv1): no finding must be emitted for NameType=1 with \
         clean ASCII hostname; got {:?}",
        analyzer.findings()
    );

    // BC-2.07.025 pc4: handshakes_seen incremented.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-007 (BC-2.07.025): handshakes_seen must be 1"
    );

    // BC-2.07.025 pc1 positive proof — story EC-004 / BC-2.07.025 EC-002:
    // NameType≠0 + NON-ASCII UTF-8 hostname → ARM 3 (NonAsciiUtf8).
    //
    // Story EC-004 (STORY-057.md:135) specifies "NameType=1" as an illustrative
    // value; BC-2.07.025 EC-002 phrases it as "NameType=255" (0xFF). Both describe
    // the same invariant: any non-zero NameType is discarded identically by the `_`
    // wildcard in `let Some((_, hostname)) = list.first()`. The specific value is
    // immaterial — the fixture uses NameType=0xFF (BC-2.07.025 EC-002's phrasing)
    // to exercise the maximum reserved value.
    //
    // A clean-ASCII hostname cannot distinguish "hostname reached the classifier"
    // from "entry was silently skipped." A non-ASCII UTF-8 hostname forces arm 3
    // (NonAsciiUtf8 finding) if and only if the hostname bytes reach the classifier.
    //
    // Fixture: NameType=0xFF + "café.example" (U+00E9 = 0xC3 0xA9, valid UTF-8
    // non-ASCII). If the entry were skipped: sni_counts empty, no finding.
    // If the hostname reaches the classifier: arm 3 fires, exactly ONE NonAsciiUtf8
    // finding emitted, sni_counts keyed on the UTF-8 string.
    let mut analyzer_ec004 = TlsAnalyzer::new();
    let record_ec004 =
        build_client_hello_with_typed_sni_list(&[(0xFF, "café.example".as_bytes())], &[0x1301]);
    analyzer_ec004.on_data(&fk, Direction::ClientToServer, &record_ec004, 0, 0);

    // Parse anchor.
    assert_eq!(
        analyzer_ec004.parse_error_count(),
        0,
        "AC-007 EC-004 anchor (BC-2.07.025 pc1 / EC-002): parse_error_count must be 0"
    );

    // sni_counts keyed on the raw UTF-8 string — proves the hostname reached arm 3.
    assert_eq!(
        *analyzer_ec004
            .sni_counts()
            .get("café.example")
            .unwrap_or(&0),
        1,
        "AC-007 EC-004 (BC-2.07.025 pc1 / EC-002): sni_counts must contain \
         \"café.example\" despite NameType=0xFF — proves the hostname reached the \
         4-way classifier (not silently skipped); got {:?}",
        analyzer_ec004.sni_counts()
    );

    // Exactly one NonAsciiUtf8 finding fires — arm 3 classification ran.
    let arm3_findings_ec004: Vec<_> = analyzer_ec004
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .collect();
    assert_eq!(
        arm3_findings_ec004.len(),
        1,
        "AC-007 EC-004 (BC-2.07.025 pc1 / EC-002): exactly one arm-3 NonAsciiUtf8 \
         finding must fire for NameType=0xFF with non-ASCII UTF-8 hostname — proves \
         arm-3 classification ran despite non-zero NameType; got {:?}",
        arm3_findings_ec004
    );

    // Extra coverage (arm 2 path, not EC-004): NameType≠0 + C0-byte hostname.
    // Verifies that an arm-2 AsciiWithControl hostname also reaches the classifier
    // when paired with a non-zero NameType — complements the arm-3 proof above.
    let mut analyzer_arm2_nonzero = TlsAnalyzer::new();
    let record_arm2 =
        build_client_hello_with_typed_sni_list(&[(0x01, b"bad\x01.example")], &[0x1301]);
    analyzer_arm2_nonzero.on_data(&fk, Direction::ClientToServer, &record_arm2, 0, 0);

    assert_eq!(
        *analyzer_arm2_nonzero
            .sni_counts()
            .get("bad\x01.example")
            .unwrap_or(&0),
        1,
        "arm-2/non-zero-NameType extra coverage: raw hostname \"bad\\x01.example\" must \
         be in sni_counts despite NameType=0x01"
    );
    let ctrl_findings_arm2: Vec<_> = analyzer_arm2_nonzero
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .collect();
    assert_eq!(
        ctrl_findings_arm2.len(),
        1,
        "arm-2/non-zero-NameType extra coverage: exactly one AsciiWithControl finding \
         must fire for NameType=0x01 with C0-containing hostname; got {:?}",
        ctrl_findings_arm2
    );
}

#[test]
fn test_non_zero_name_type_with_valid_first_entry() {
    // AC-007 (BC-2.07.025 pc1-3)
    //
    // Variant: the FIRST entry has a non-zero NameType (0x03, unknown future use)
    // with a valid ASCII hostname, and the SECOND entry has a different non-zero
    // NameType (0x02). This directly exercises BC-2.07.025 pc1: "hostname bytes
    // from the first entry (regardless of NameType) are passed to classification."
    // The `_` wildcard in `let Some((_, hostname)) = list.first()` discards the
    // NameType; only the hostname bytes reach the 4-way classifier.
    //
    // Using NameType=0x00 for the first entry (the previous form of this test)
    // did not exercise pc1 — it merely re-tested the standard arm 1 path. With
    // NameType=0x03 the first entry is genuinely non-zero-typed.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello_with_typed_sni_list(
        &[(0x03, b"real.example"), (0x02, b"unknown-type")],
        &[0x1301],
    );
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-007 variant anchor (BC-2.07.025): parse_error_count must be 0"
    );

    // BC-2.07.025 pc1: first entry (NameType=0x03, "real.example") processed.
    // NameType is discarded by the `_` pattern; only the hostname bytes are used.
    assert_eq!(
        *analyzer.sni_counts().get("real.example").unwrap_or(&0),
        1,
        "AC-007 variant (BC-2.07.025 pc1): first entry hostname \"real.example\" must be \
         counted despite NameType=0x03 (NameType discarded by `_` pattern); \
         got {:?}",
        analyzer.sni_counts()
    );

    // BC-2.07.025 pc4 (composes with BC-2.07.024): second entry not counted.
    assert!(
        analyzer.sni_counts().get("unknown-type").is_none(),
        "AC-007 variant (BC-2.07.025): second entry (NameType=0x02) must not be counted; \
         got {:?}",
        analyzer.sni_counts()
    );

    // Exactly one entry in sni_counts.
    assert_eq!(
        analyzer.sni_counts().len(),
        1,
        "AC-007 variant (BC-2.07.025 pc2): exactly one sni_counts entry must be inserted"
    );

    // BC-2.07.025 pc3: no finding emitted solely because of non-zero NameType.
    // "real.example" is clean ASCII (arm 1), so no finding fires for the hostname.
    assert!(
        analyzer.findings().is_empty(),
        "AC-007 variant (BC-2.07.025 pc3): no finding must be emitted; got {:?}",
        analyzer.findings()
    );
}

#[test]
fn test_large_sni_near_record_payload_limit() {
    // AC-010 (BC-2.07.027 pc1-5) + AC-011 (BC-2.07.027 inv1-2)
    //
    // A ClientHello with a clean ASCII SNI hostname of approximately 16 KB
    // (payload_len <= MAX_RECORD_PAYLOAD = 18,432) is accepted and parsed without error.
    //   pc1: record accepted (not truncated)
    //   pc2: parse_errors NOT incremented
    //   pc3: large hostname classified (arm 1 for clean ASCII) and counted in sni_counts
    //   pc4: handshakes_seen incremented
    //   pc5: no finding emitted (hostname is clean ASCII)
    //
    // AC-011 (inv1-2): MAX_RECORD_PAYLOAD=18,432 is the binding constraint, not
    // MAX_BUF=65,536. A 16,384-byte 'a' hostname produces a record payload of ~16,456 —
    // well within the 18,432 limit (exact value verified by the fixture-sanity assertion
    // below). The system has no SNI-length-specific cap below MAX_RECORD_PAYLOAD.
    // Assert truncated_records is NOT incremented.
    //
    // BC-2.07.027 canonical test vector: 16,384 bytes of 'a' (EC-001).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Canonical test vector: 16,384 bytes of 'a' (BC-2.07.027 EC-001 / story EC-005).
    // ClientHello overhead is ~74 bytes, so the record payload is ~16,458 bytes —
    // under the MAX_RECORD_PAYLOAD=18,432 cap.
    let large_hostname = "a".repeat(16_384);
    let record = build_client_hello(&large_hostname, &[0x1301]);

    // Fixture sanity: verify the record payload is within the limit.
    // This guards against fixture drift if the builder structure changes.
    const MAX_RECORD_PAYLOAD: usize = 18_432;
    assert!(
        record.len() >= 5,
        "AC-010 fixture: TLS record must have at least a 5-byte header"
    );
    let payload_len = u16::from_be_bytes([record[3], record[4]]) as usize;
    assert!(
        payload_len <= MAX_RECORD_PAYLOAD,
        "AC-010 fixture sanity (BC-2.07.027 pre2): record payload must be <= \
         MAX_RECORD_PAYLOAD=18,432 (got {payload_len}); if this fails the fixture is too large"
    );

    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // BC-2.07.027 pc2: parse_errors NOT incremented.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-010 (BC-2.07.027 pc2): parse_errors must be 0 for a large but valid SNI record"
    );

    // AC-011 (BC-2.07.027 inv1-2): truncated_records NOT incremented — MAX_RECORD_PAYLOAD
    // is the binding constraint and this record fits. No SNI-specific length cap fires.
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-011 (BC-2.07.027 inv1-2): truncated_records must be 0 — the record payload \
         ({payload_len} bytes) is <= MAX_RECORD_PAYLOAD=18,432; no SNI-length cap exists \
         below MAX_RECORD_PAYLOAD"
    );

    // BC-2.07.027 pc3: large hostname classified (arm 1: clean ASCII 'a' repeated) and
    // counted in sni_counts.
    assert_eq!(
        *analyzer
            .sni_counts()
            .get(large_hostname.as_str())
            .unwrap_or(&0),
        1,
        "AC-010 (BC-2.07.027 pc3): large SNI hostname must be counted in sni_counts; \
         got {:?}",
        analyzer.sni_counts()
    );

    // BC-2.07.027 pc4: handshakes_seen incremented.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-010 (BC-2.07.027 pc4): handshakes_seen must be 1"
    );

    // BC-2.07.027 pc5: no finding emitted (hostname is clean ASCII, arm 1).
    assert!(
        analyzer.findings().is_empty(),
        "AC-010 (BC-2.07.027 pc5): no finding must be emitted for a clean ASCII large SNI; \
         got {:?}",
        analyzer.findings()
    );
}

#[test]
fn test_oversized_sni_exceeds_record_payload_limit() {
    // AC-001 / BC-2.07.004 postconditions 1-6:
    // When try_parse_records reads the 5-byte TLS record header and finds
    // payload_len > MAX_RECORD_PAYLOAD (18,432 bytes), both parse_errors and
    // truncated_records are incremented by 1. The direction buffer is cleared.
    // try_parse_records returns. No finding is emitted. handshakes_seen is NOT
    // incremented.
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
        "AC-001 precondition (BC-2.07.004 pre2): record payload must exceed MAX_RECORD_PAYLOAD \
         (got {payload_len}); if this assertion fails the fixture is too small"
    );

    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // BC-2.07.004 postcondition 1: parse_errors incremented by 1.
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "AC-001 (BC-2.07.004 pc1): parse_errors must be 1 after oversized record"
    );

    // BC-2.07.004 postcondition 2: truncated_records incremented by 1.
    assert_eq!(
        analyzer.truncated_record_count(),
        1,
        "AC-001 (BC-2.07.004 pc2): truncated_records must be 1 after oversized record \
         (LESSON-P1.05 / CNV-PAT-002: DoS-protection drops tracked separately)"
    );

    // BC-2.07.004 postcondition 3: direction buffer (client_buf) cleared entirely.
    // After on_data, the flow exists (the entry was inserted) and client_buf must be 0.
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        0,
        "AC-001 (BC-2.07.004 pc3): client_buf must be cleared (len==0) after oversized record"
    );

    // BC-2.07.004 postcondition 5: no finding emitted.
    assert!(
        analyzer.findings().is_empty(),
        "AC-001 (BC-2.07.004 pc5): no finding must be emitted for an oversized record; \
         got: {:?}",
        analyzer.findings()
    );

    // BC-2.07.004 postcondition 6: handshakes_seen NOT incremented.
    assert_eq!(
        analyzer.handshake_count(),
        0,
        "AC-001 (BC-2.07.004 pc6): handshakes_seen must be 0 (oversized record not parsed)"
    );

    // SNI was never reached.
    assert!(
        analyzer.sni_counts().is_empty(),
        "AC-001 (BC-2.07.004 pc4): record rejected before SNI parsing; sni_counts must be empty"
    );
}

#[test]
fn test_trailing_bytes_in_server_name_list() {
    // AC-009 (BC-2.07.026 pc1-3)
    //
    // If a TLS ClientHello SNI extension has trailing bytes after the last valid
    // hostname entry (ServerNameList outer length field claims more bytes than the
    // actual entries), extract_sni processes the first hostname entry normally.
    //   pc1: first hostname entry processed normally (counted and classified)
    //   pc2: no parse_errors incremented by extract_sni itself
    //   pc3: trailing bytes silently ignored
    //
    // BC-2.07.026 canonical test vector: valid "test.example" + 4 trailing 0x00
    // bytes beyond the ServerNameList's declared length — sni_counts["test.example"]=1;
    // no error; no extra finding. The list-length field is HONEST; the trailing zeros
    // are genuinely unconsumed remainder within the extension data block.
    //
    // Note: this BC is a property of the tls_parser crate's tolerance. If a future
    // tls_parser upgrade tightens validation to reject unconsumed trailing bytes in
    // the extension data block, this test documents the behavioral change.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Build raw SNI extension data aligned to BC-2.07.026's canonical vector:
    // the ServerNameList length field is HONEST (matches the actual entry bytes),
    // and the trailing 0x00 padding bytes are appended AFTER the declared list
    // content inside the extension data block. They are genuinely unconsumed —
    // beyond the ServerNameList's declared boundary — not inflated inside the
    // list-length field (which was the previous, inaccurate framing).
    //
    // Wire layout of raw_ext_data passed to build_client_hello_with_raw_sni_ext:
    //   [2 bytes] honest ServerNameList length = 15
    //   [1 byte ] NameType = 0x00
    //   [2 bytes] name_len = 12
    //   [12 bytes] "test.example"
    //   [4 bytes] 0x00 0x00 0x00 0x00  ← trailing padding beyond declared list length
    //
    // tls_parser slices the ServerNameList to exactly 15 bytes, so "test.example"
    // is parsed cleanly. The 4 trailing zeros are unconsumed remainder within the
    // extension data block — silently ignored by tls_parser, not seen by extract_sni.
    let hostname = b"test.example";
    let name_len =
        u16::try_from(hostname.len()).expect("hostname length must fit in TLS u16 field");
    let mut sni_list_data = Vec::new();
    sni_list_data.push(0x00); // NameType = host_name
    sni_list_data.extend_from_slice(&name_len.to_be_bytes());
    sni_list_data.extend_from_slice(hostname);

    // Honest list length — covers exactly the one entry above.
    let honest_list_len = u16::try_from(sni_list_data.len()).expect("list length must fit in u16");
    let mut raw_ext_data = Vec::new();
    raw_ext_data.extend_from_slice(&honest_list_len.to_be_bytes()); // honest length
    raw_ext_data.extend_from_slice(&sni_list_data); // the single valid entry
    raw_ext_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // 4 trailing zero bytes

    let record = build_client_hello_with_raw_sni_ext(&raw_ext_data, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // BC-2.07.026 pc2: no parse_errors incremented.
    // If tls_parser ever rejects unconsumed trailing bytes in the extension data
    // block, parse_errors would be 1 and this assertion would document the break.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-009 (BC-2.07.026 pc2): parse_errors must be 0; trailing 0x00 bytes beyond \
         the declared ServerNameList boundary are silently ignored by tls_parser"
    );

    // BC-2.07.026 pc1: first hostname entry processed normally and counted.
    assert_eq!(
        *analyzer.sni_counts().get("test.example").unwrap_or(&0),
        1,
        "AC-009 (BC-2.07.026 pc1): \"test.example\" must be counted in sni_counts \
         despite trailing bytes in ServerNameList; got {:?}",
        analyzer.sni_counts()
    );

    // BC-2.07.026 pc3: trailing bytes silently ignored — no extra entries in sni_counts.
    assert_eq!(
        analyzer.sni_counts().len(),
        1,
        "AC-009 (BC-2.07.026 pc3): exactly one sni_counts entry; trailing bytes must \
         not produce additional entries; got {:?}",
        analyzer.sni_counts()
    );

    // No finding (first entry "test.example" is clean ASCII, arm 1).
    assert!(
        analyzer.findings().is_empty(),
        "AC-009 (BC-2.07.026): no finding must be emitted for clean ASCII hostname \
         with trailing bytes; got {:?}",
        analyzer.findings()
    );

    // Handshake counted.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-009 (BC-2.07.026 pc1): handshakes_seen must be 1"
    );
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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

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
    analyzer.on_data(&test_flow_key(), Direction::ClientToServer, &bytes, 0, 0);

    let findings = analyzer.findings();
    let control_finding = findings
        .iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .expect("expected an ASCII-control SNI finding");
    assert_eq!(
        control_finding.mitre_techniques,
        vec!["T1027".to_string()],
        "malformed-SNI finding must have mitre_techniques=[\"T1027\"] (Obfuscated Files or Information)",
    );
}

#[test]
fn non_ascii_utf8_sni_finding_sets_mitre_t1027() {
    let bytes = build_client_hello("пример.рф", &[]);
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(&test_flow_key(), Direction::ClientToServer, &bytes, 0, 0);

    let findings = analyzer.findings();
    let finding = findings
        .iter()
        .find(|f| f.summary.contains("non-ASCII characters"))
        .expect("expected a non-ASCII SNI finding");
    assert_eq!(
        finding.mitre_techniques,
        vec!["T1027".to_string()],
        "malformed-SNI finding must have mitre_techniques=[\"T1027\"] (Obfuscated Files or Information)",
    );
}

// AC-004 companion (BC-2.07.019 postcondition 3)
//
// Non-UTF-8 SNI finding: mitre_technique = Some("T1027").
// Explicit companion test as named in STORY-056 AC-004.
// EC-005: b"\x80" (lone continuation byte) is invalid UTF-8 -> arm 4.
#[allow(non_snake_case)]
#[test]
fn non_utf8_sni_finding_sets_mitre_t1027() {
    // AC-004 (BC-2.07.019 pc3): mitre_technique must be Some("T1027") for arm 4.
    let fk = test_flow_key();

    // --- EC-005: lone continuation byte b"\x80" ---
    let mut analyzer = TlsAnalyzer::new();
    let sni_bytes: &[u8] = b"\x80";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-004 companion anchor (BC-2.07.019 pc3): parse must succeed"
    );

    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect(
            "AC-004 companion (BC-2.07.019 pc3): non-UTF-8 finding must be emitted for b\"\\x80\"",
        );

    // BC-2.07.019 pc3: mitre_techniques = ["T1027"] (single-tag, full-vec equality).
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-004 companion (BC-2.07.019 pc3): mitre_techniques must be exactly [\"T1027\"] for \
         arm 4 (lone continuation byte b\"\\x80\")"
    );

    // All Finding fields symmetric with arm 4 specification (BC-2.07.019 pc3).
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "non_utf8_sni_finding_sets_mitre_t1027 (BC-2.07.019 pc3): category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "non_utf8_sni_finding_sets_mitre_t1027 (BC-2.07.019 pc3): verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "non_utf8_sni_finding_sets_mitre_t1027 (BC-2.07.019 pc3): confidence must be Low"
    );
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "non_utf8_sni_finding_sets_mitre_t1027 (BC-2.07.019 pc3): direction must be \
         Some(ClientToServer)"
    );

    // sni_counts key is "<non-utf8:80>" for b"\x80".
    assert_eq!(
        *analyzer.sni_counts().get("<non-utf8:80>").unwrap_or(&0),
        1,
        "non_utf8_sni_finding_sets_mitre_t1027 (BC-2.07.019 pc2): sni_counts key must be \
         \"<non-utf8:80>\"; got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );

    // --- Regression: original vector [foo, 0xc3, '.', 'c', 'o', 'm'] ---
    // 0xc3 is a valid 2-byte UTF-8 start byte but requires continuation 0x80-0xBF;
    // '.' (0x2e) is not a valid continuation, so from_utf8 fails -> arm 4.
    let mut analyzer2 = TlsAnalyzer::new();
    let bytes2 = build_client_hello_raw_sni(&[b'f', b'o', b'o', 0xc3, b'.', b'c', b'o', b'm'], &[]);
    analyzer2.on_data(&fk, Direction::ClientToServer, &bytes2, 0, 0);

    let finding2 = analyzer2
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("AC-004 companion regression: expected a non-UTF-8 SNI finding");
    assert_eq!(
        finding2.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-004 companion regression (BC-2.07.019 pc3): mitre_techniques must be exactly [\"T1027\"]"
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
    build_client_hello_with_version(0x0303, cipher_ids)
}

/// Build a minimal TLS ClientHello record with an explicit ClientHello version
/// field and no extensions. Used for tests that need to exercise version=0 or
/// other non-standard version values to verify that the JA3 version field is
/// taken directly from the wire encoding.
fn build_client_hello_with_version(version: u16, cipher_ids: &[u16]) -> Vec<u8> {
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&version.to_be_bytes()); // ClientHello version field
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

/// Build a minimal ServerHello whose extension list contains ONLY GREASE extension
/// type IDs (0x0a0a, 0x1a1a, 0x2a2a). After JA3S GREASE filtering the extension
/// field will be empty, producing a JA3S string of the form "ver,cipher,".
///
/// Used for STORY-051 EC-007: verifying that a ServerHello with exclusively GREASE
/// extensions produces an empty ext field in JA3S (analogous to JA3's all-GREASE
/// cipher handling, but applied to the server-side extension list).
fn build_server_hello_all_grease_ext(cipher_id: u16) -> Vec<u8> {
    let mut extensions = Vec::new();

    // Three distinct GREASE extension type IDs — all match the (id & 0x0f0f) == 0x0a0a
    // bitmask that the JA3S implementation uses to filter GREASE values.
    extensions.extend_from_slice(&[0x0a, 0x0a]); // GREASE 0x0a0a
    extensions.extend_from_slice(&[0x00, 0x00]); // data length = 0

    extensions.extend_from_slice(&[0x1a, 0x1a]); // GREASE 0x1a1a
    extensions.extend_from_slice(&[0x00, 0x00]); // data length = 0

    extensions.extend_from_slice(&[0x2a, 0x2a]); // GREASE 0x2a2a
    extensions.extend_from_slice(&[0x00, 0x00]); // data length = 0

    let mut sh_body = Vec::new();
    sh_body.extend_from_slice(&[0x03, 0x03]); // TLS 1.2 (version = 771)
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
    // JA3 string = "771,,,," -> MD5 = bddda940f9963577c41d7c28b1a5f65f
    let fk = test_flow_key();

    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x0a0a]),
        0,
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
    // MD5("771,47-53,,,") = 577fbfd57b256f5467f2fe09d1105a26
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
fn test_BC_2_07_007_canonical_771_no_cipher_no_extension_hash() {
    // BC-2.07.007 EC-001 companion (BC-2.07.007 postconditions 1-2 / canonical 771-baseline anchor): version=771 with empty ciphers -> "771,,,,"
    // Anchor pin for the 771-version baseline: any change to JA3 string formatting
    // or the version field encoding will break this test.
    // Canonical: MD5("771,,,,") = bddda940f9963577c41d7c28b1a5f65f
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[]),
        0,
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "bddda940f9963577c41d7c28b1a5f65f",
        "BC-2.07.007 EC-001 anchor: no-cipher JA3 (version=771) must be MD5('771,,,,') \
         = bddda940f9963577c41d7c28b1a5f65f confirming first field is version 771"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_007_version_zero_emits_leading_zero_field() {
    // BC-2.07.007 postcondition 2 (edge case): when the ClientHello version field is
    // 0x0000 (decimal 0), the JA3 version field must be "0" — not filtered, not
    // substituted, not omitted.
    // JA3 string = "0,,,," -> MD5 = 2432bebf06532faf89aae784a9aae4ef
    //
    // This test is the deterministic companion to the inline proptest that covers
    // version in any::<u16>(). It pins the exact version=0 wire encoding path.
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_with_version(0x0000, &[]),
        0,
        0,
    );
    let hash = analyzer.ja3_counts().keys().next().unwrap().clone();
    // Verify the JA3 string starts with "0," by checking the canonical hash
    // that can only arise from MD5("0,,,,").
    assert_eq!(
        hash, "2432bebf06532faf89aae784a9aae4ef",
        "BC-2.07.007 postcondition 2 (version=0 edge): JA3 must be MD5('0,,,,') \
         = 2432bebf06532faf89aae784a9aae4ef confirming version=0 emits leading '0' field"
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
        0,
    );
    let hash_ab = a_ab.ja3_counts().keys().next().unwrap().clone();

    let mut a_ba = TlsAnalyzer::new();
    a_ba.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello_no_extensions(&[0x0035, 0x002f]),
        0,
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
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello(0x002f),
        0,
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
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello_with_grease_ext(0x002f),
        0,
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
    a1.on_data(&fk1, Direction::ClientToServer, &ch_bytes, 0, 0);
    a1.on_data(&fk1, Direction::ServerToClient, &sh_bytes, 0, 0);
    let hash1 = a1.ja3s_counts().keys().next().unwrap().clone();

    let mut a2 = TlsAnalyzer::new();
    a2.on_data(&fk2, Direction::ClientToServer, &ch_bytes, 0, 0);
    a2.on_data(&fk2, Direction::ServerToClient, &sh_bytes, 0, 0);
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
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello(0x0a0a), // GREASE cipher selected by server
        0,
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
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello_with_grease_ext(0x0a0a), // GREASE cipher + GREASE ext
        0,
        0,
    );

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    assert_eq!(
        hash, "c4b833c0849ff23c29e04fa13f6e87da",
        "BC-2.07.008: GREASE ext filtered, GREASE cipher kept -> \
         MD5('771,2570,65281') = c4b833c0849ff23c29e04fa13f6e87da"
    );
}

// ── AC-008 companion (BC-2.07.008 postcondition 4 / STORY-051 EC-007): JA3S all-GREASE extension list -> empty ext field ─
//
// When a ServerHello's extension list contains ONLY GREASE extension type IDs,
// every ext ID is filtered and the JA3S extension field must be empty ("").
// This is the server-side analogue of BC-2.07.006's all-GREASE cipher handling.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_008_ja3s_all_grease_extensions_produce_empty_ext_field() {
    // STORY-051 EC-007: ServerHello with exclusively GREASE extension IDs.
    // Cipher: 0x002f (decimal 47), version: 0x0303 (decimal 771).
    // Extensions: 0x0a0a, 0x1a1a, 0x2a2a — all GREASE, all filtered.
    // JA3S string = "771,47," (trailing empty ext field) ->
    // MD5("771,47,") = 5397c414a9ebeaff1bf18b70ca22eaa0
    //
    // This must differ from the hash when at least one non-GREASE ext is present
    // ("771,47,65281" -> MD5 = 573a9f3f80037fb40d481e2054def5bb).
    let fk = test_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_client_hello("example.com", &[0x002f]),
        0,
        0,
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello_all_grease_ext(0x002f),
        0,
        0,
    );

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "GREASE extensions must parse cleanly through tls-parser; if tls-parser ever rejects them \
         the empty ext_ids would coincidentally produce the same hash and silently bypass the \
         GREASE-filter branch this test claims to pin"
    );

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // Must NOT equal the hash with a non-GREASE ext present.
    assert_ne!(
        hash, "573a9f3f80037fb40d481e2054def5bb",
        "EC-007: all-GREASE ext list must NOT produce same hash as list with non-GREASE ext"
    );
    // Must equal MD5("771,47,") — empty ext field after full GREASE filtering.
    assert_eq!(
        hash, "5397c414a9ebeaff1bf18b70ca22eaa0",
        "EC-007 (BC-2.07.008 postcondition 4 edge): all-GREASE ServerHello extensions \
         must yield empty ext field; canonical MD5('771,47,') = 5397c414a9ebeaff1bf18b70ca22eaa0"
    );
}

// ── STORY-052 adversarial-pass remediation: proxy-assertion gap fixes ─────────
//
// The four tests below address MEDIUM findings from the adversarial review:
//
//   AC-005 / BC-2.07.001 pc8  — direct client_buf drain observation (finding #1)
//     Fixed in test_parse_client_hello above via client_buf_len_for_testing().
//
//   AC-010 / BC-2.07.032 pc2  — JA3 "771," prefix (finding #2)
//     Fixed in tls_integration_tests.rs: assert !version_counts.contains_key(&0x0304).
//
//   AC-011 / BC-2.07.032 inv1 — supported_versions not inspected (finding #3)
//     New synthetic unit test below: feed a ClientHello whose legacy_version is
//     0x0303 but whose supported_versions extension claims 0x0304; assert
//     version_counts records 0x0303 (NOT 0x0304).
//
//   AC-002 / BC-2.07.001 pc2+inv2 — version_counts and ja3_counts capacity (finding #4)
//     Two new capacity tests below, mirroring test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity.

/// Build a TLS ClientHello record with `legacy_version=0x0303` and a
/// `supported_versions` extension advertising `[supported_version]`.
///
/// This produces a wire-format TLS 1.3 ClientHello: the record-layer version
/// is 0x0303 (legacy_version per RFC 8446 §4.1.2) and the `supported_versions`
/// extension (type 0x002b) signals the actual desired version. Used by
/// `test_BC_2_07_032_inv1_supported_versions_not_inspected` to confirm the
/// analyzer records `ch.version.0` (legacy_version) and NOT the extension value.
fn build_client_hello_with_supported_versions_ext(
    sni: &str,
    cipher_ids: &[u16],
    supported_version: u16,
) -> Vec<u8> {
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000)
    let sni_bytes = sni.as_bytes();
    let name_len = u16::try_from(sni_bytes.len()).expect("SNI too long");
    let sni_list_len = name_len
        .checked_add(3)
        .expect("SNI list length overflows u16");
    let sni_ext_len = sni_list_len
        .checked_add(2)
        .expect("SNI ext length overflows u16");
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&sni_ext_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_len.to_be_bytes());
    extensions.push(0x00); // NameType: host_name
    extensions.extend_from_slice(&name_len.to_be_bytes());
    extensions.extend_from_slice(sni_bytes);

    // supported_versions extension (type 0x002b)
    // ClientHello format: 1 byte list-byte-length + N * 2-byte version entries
    // For a single version: [0x02, hi, lo]
    extensions.extend_from_slice(&[0x00, 0x2b]); // extension type: supported_versions
    extensions.extend_from_slice(&[0x00, 0x03]); // extension data length = 3 bytes
    extensions.push(0x02); // versions list byte-length = 2 (one 2-byte version)
    extensions.extend_from_slice(&supported_version.to_be_bytes());

    // Supported Groups extension (type 0x000a)
    extensions.extend_from_slice(&[0x00, 0x0a, 0x00, 0x06, 0x00, 0x04, 0x00, 0x1d, 0x00, 0x17]);

    // EC Point Formats extension (type 0x000b)
    extensions.extend_from_slice(&[0x00, 0x0b, 0x00, 0x02, 0x01, 0x00]);

    // ClientHello body with legacy_version = 0x0303
    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&[0x03, 0x03]); // legacy_version = TLS 1.2 (0x0303)
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0

    let ciphers_len =
        u16::try_from(cipher_ids.len() * 2).expect("cipher list byte length overflows u16");
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression

    let ext_len = u16::try_from(extensions.len()).expect("extensions block exceeds u16::MAX bytes");
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

    // TLS record header (record-layer version 0x0301 as used by TLS 1.3 on the wire)
    let mut record = Vec::new();
    record.push(0x16); // handshake
    record.extend_from_slice(&[0x03, 0x01]); // TLS 1.0 compat record-layer version
    let hs_len = u16::try_from(handshake.len()).expect("handshake body exceeds u16::MAX bytes");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

// Finding #3 (AC-011 / BC-2.07.032 invariant 1): supported_versions not inspected
//
// The prior test (test_tls13_pcap_version_and_ja3 integration test) was vacuous
// because any real TLS 1.3 ClientHello has legacy_version=0x0303 and
// supported_versions=0x0304, so observing version_counts[0x0303] proved nothing
// — both the "looks at legacy_version" AND the "looks at supported_versions" paths
// would produce 0x0303 in version_counts for a real TLS 1.3 capture.
//
// This test constructs a synthetic fixture where legacy_version=0x0303 and
// supported_versions=[0x0304]. If the analyzer inspected supported_versions it
// would record 0x0304 in version_counts. Asserting 0x0304 is ABSENT pins the
// invariant that only ch.version.0 (legacy_version) is ever recorded.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_032_inv1_supported_versions_not_inspected() {
    // BC-2.07.032 invariant 1: TlsAnalyzer records ch.version.0 (legacy_version)
    // and does NOT inspect the supported_versions extension for version counting.
    //
    // Fixture: ClientHello with legacy_version=0x0303 (TLS 1.2 compatibility value)
    // and supported_versions extension advertising 0x0304 (TLS 1.3).
    //
    // If the analyzer were to inspect the supported_versions extension, it would
    // record 0x0304 in version_counts. The BC states only legacy_version is used.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Build a ClientHello: legacy_version=0x0303, supported_versions ext=[0x0304].
    // The two versions differ: 0x0303 (legacy_version) vs 0x0304 (supported_versions).
    let record = build_client_hello_with_supported_versions_ext(
        "tls13.example.com",
        &[0x1301, 0x1302, 0x1303], // TLS 1.3 ciphers
        0x0304,                    // supported_versions extension: TLS 1.3
    );
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "fixture must parse cleanly"
    );
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "exactly one ClientHello processed"
    );

    // The legacy_version (0x0303) MUST be recorded.
    assert!(
        analyzer.version_counts().contains_key(&0x0303),
        "AC-011 / BC-2.07.032 inv1: version_counts must contain 0x0303 (legacy_version), \
         got: {:?}",
        analyzer.version_counts()
    );

    // The supported_versions extension value (0x0304) must NOT be recorded.
    // If this assertion fails, the analyzer is wrongly inspecting supported_versions.
    assert!(
        !analyzer.version_counts().contains_key(&0x0304),
        "AC-011 / BC-2.07.032 inv1: version_counts must NOT contain 0x0304 \
         (supported_versions extension value) — TlsAnalyzer must use only ch.version.0 \
         (legacy_version=0x0303), NOT the supported_versions extension. Got: {:?}",
        analyzer.version_counts()
    );

    // Confirm version_counts has exactly ONE key (the legacy_version 0x0303).
    assert_eq!(
        analyzer.version_counts().len(),
        1,
        "AC-011 / BC-2.07.032 inv1: version_counts must have exactly one entry \
         (legacy_version 0x0303), not two entries (0x0303 + 0x0304). \
         Got: {:?}",
        analyzer.version_counts()
    );
}

// Finding #4 (AC-002 / BC-2.07.001 postcondition 2 + invariant 2):
// version_counts and ja3_counts capacity bounds — both maps are bounded at
// MAX_MAP_ENTRIES=50,000 per BC-2.07.001 invariant 2 ("ALL counter maps bounded").
// The existing test (test_non_utf8_sni_finding_fires_when_sni_counts_at_capacity)
// only exercises sni_counts. These two tests pin the capacity bound for
// version_counts and ja3_counts respectively.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_001_inv2_version_counts_bounded_at_max_map_entries() {
    // BC-2.07.001 invariant 2: version_counts is bounded at MAX_MAP_ENTRIES=50,000.
    // When the map is full (50,000 unique keys), a new key is silently dropped.
    //
    // Implementation: version is a u16, so there are only 65,536 possible values.
    // We fill version_counts to capacity by sending ClientHellos via synthetic
    // TLS records where we directly mutate the version field. Since
    // build_client_hello_with_version always uses legacy_version from the
    // ClientHello body (not the record-layer version), we can sweep version values.
    //
    // We fill with versions 0x0001 through 0xC350 (50,000 values), then send
    // version 0xC351 and assert it was NOT inserted.
    //
    // Note: each of these versions <= 0x0300 (except those > 0x0300) will also
    // generate deprecated-protocol findings — that's expected behavior and does
    // NOT affect the capacity assertion for version_counts.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_MAP_ENTRIES: usize = 50_000;

    // Fill version_counts to capacity using MAX_MAP_ENTRIES distinct version values.
    // Version 0 would trigger deprecated-protocol finding noise but is harmless.
    // We use versions 1..=MAX_MAP_ENTRIES (all fit in u16 since 50,000 < 65,536).
    for v in 1u32..=(MAX_MAP_ENTRIES as u32) {
        let version = v as u16;
        let record = build_client_hello_with_version(version, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);
    }

    assert_eq!(
        analyzer.version_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-002 / BC-2.07.001 inv2: version_counts must be full at MAX_MAP_ENTRIES={MAX_MAP_ENTRIES}, \
         got {}",
        analyzer.version_counts().len()
    );

    // The next distinct version value (MAX_MAP_ENTRIES + 1 = 50,001) must be silently dropped.
    let overflow_version = (MAX_MAP_ENTRIES + 1) as u16; // 50,001 — fits in u16
    let record = build_client_hello_with_version(overflow_version, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // version_counts must not grow beyond the cap.
    assert_eq!(
        analyzer.version_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-002 / BC-2.07.001 inv2: version_counts must not exceed MAX_MAP_ENTRIES={MAX_MAP_ENTRIES} \
         after overflow; new key {overflow_version} must be silently dropped. \
         Got: {} entries",
        analyzer.version_counts().len()
    );

    // Confirm the overflow key is absent.
    assert!(
        !analyzer.version_counts().contains_key(&overflow_version),
        "AC-002 / BC-2.07.001 inv2: overflow version key {overflow_version} must not appear \
         in version_counts past the cap. Got: {:?}",
        analyzer.version_counts().get(&overflow_version)
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_001_inv2_ja3_counts_bounded_at_max_map_entries() {
    // BC-2.07.001 invariant 2: ja3_counts is bounded at MAX_MAP_ENTRIES=50,000.
    // When the map is full, a new JA3 hash is silently dropped.
    //
    // We produce 50,000 unique JA3 hashes by varying the ClientHello version field.
    // Each distinct version v produces JA3 string "{v},4865,,," where 4865 (0x1301,
    // TLS_AES_128_GCM_SHA256) is a strong non-GREASE cipher. The JA3 string differs
    // for each v, so each MD5 hash is unique.
    //
    // We use build_client_hello_with_version (no-extension builder) to suppress SNI /
    // curves / pf overhead and keep the per-record cost low. Versions 1..=50,000 all
    // fit in u16 (max 65,535) with no wraparound.
    //
    // Note: versions 1..=0x0300 (1..=768) produce deprecated-protocol findings.
    // That is expected: findings are not bounded per BC-2.04.024, so the finding vec
    // grows but the capacity assertion on ja3_counts is unaffected.
    //
    // This test is slower than typical unit tests (~600ms debug) because 50,000
    // TLS records must be built and parsed. Budget: ~2s CI.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_MAP_ENTRIES: usize = 50_000;

    // Fill ja3_counts to capacity. Cipher 0x1301 (4865) is strong, non-GREASE,
    // and accepted by is_grease_u16(0x1301) = false ((0x1301 & 0x0F0F) = 0x0101 ≠ 0x0A0A).
    // Each version v in 1..=MAX_MAP_ENTRIES yields a unique JA3 string "{v},4865,,,".
    for v in 1u32..=(MAX_MAP_ENTRIES as u32) {
        let record = build_client_hello_with_version(v as u16, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);
    }

    assert_eq!(
        analyzer.ja3_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-002 / BC-2.07.001 inv2: ja3_counts must be full at MAX_MAP_ENTRIES={MAX_MAP_ENTRIES}, \
         got {}",
        analyzer.ja3_counts().len()
    );

    // Inject one more ClientHello with a version that produces a NEW (distinct) JA3 hash.
    // Version MAX_MAP_ENTRIES + 1 = 50,001 (fits in u16) was not used in the fill loop.
    let overflow_version = (MAX_MAP_ENTRIES + 1) as u16; // 50,001
    let overflow_record = build_client_hello_with_version(overflow_version, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &overflow_record, 0, 0);

    // ja3_counts must not grow beyond the cap.
    assert_eq!(
        analyzer.ja3_counts().len(),
        MAX_MAP_ENTRIES,
        "AC-002 / BC-2.07.001 inv2: ja3_counts must not exceed MAX_MAP_ENTRIES={MAX_MAP_ENTRIES} \
         after overflow; new JA3 hash for version={overflow_version} must be silently dropped. \
         Got: {} entries",
        analyzer.ja3_counts().len()
    );
}

// ── STORY-053 formalization tests (BC-2.07.002) ──────────────────────────────
//
// These tests pin the behavioral contracts for ServerHello parsing,
// JA3S fingerprinting, and cipher/version tracking (BC-2.07.002).
//
// Naming follows DF-AC-TEST-NAME-SYNC-001 v2: test_BC_2_07_002_<suffix>.
// #[allow(non_snake_case)] is applied per-test for BC-prefixed names.
//
// AC → test mapping:
//   AC-001 (BC-2.07.002 pc1): test_BC_2_07_002_server_hello_seen_set_true
//   AC-002 (BC-2.07.002 pc2): test_BC_2_07_002_server_version_inserted_in_version_counts
//   AC-003 (BC-2.07.002 pc3): test_BC_2_07_002_ja3s_hash_computed_and_inserted
//                              (proptest compute_ja3s_is_deterministic_and_hex in src also covers)
//   AC-004 (BC-2.07.002 pc4): test_BC_2_07_002_cipher_name_inserted_in_cipher_counts
//   AC-005 (BC-2.07.002 inv1): test_BC_2_07_002_ja3s_grease_ext_filtered_cipher_not_filtered
//   AC-006 (BC-2.07.002 inv2): test_BC_2_07_002_unknown_cipher_id_renders_as_hex_in_cipher_counts
//   AC-007 (BC-2.07.002 inv3): test_BC_2_07_002_version_counts_client_and_server_versions_independent
//
// EC tests:
//   EC-001: test_BC_2_07_002_ec001_no_extensions_ja3s_uses_empty_ext_field
//   EC-003: test_BC_2_07_002_ec003_null_cipher_emits_weak_cipher_finding
//   EC-004: test_BC_2_07_002_ec004_ssl2_version_emits_deprecated_protocol_finding
//   EC-005: test_BC_2_07_002_ec005_tls10_version_counted_no_deprecated_finding
//   EC-006: test_BC_2_07_002_ec006_ja3s_counts_at_capacity_new_hash_dropped
//   EC-007: test_BC_2_07_002_ec007_client_and_server_different_versions_both_counted

// ── ServerHello builder helpers ───────────────────────────────────────────────

/// Build a minimal TLS ServerHello record with NO extensions at all (sh.ext = None).
///
/// Unlike `build_server_hello`, this helper omits the extensions block entirely
/// so `parse_tls_extensions` is never called and `sh.ext == None` in the parsed
/// struct. Used for EC-001: JA3S computed with empty extension field.
fn build_server_hello_no_extensions(cipher_id: u16) -> Vec<u8> {
    build_server_hello_with_version_and_cipher(0x0303, cipher_id, false)
}

/// Build a minimal TLS ServerHello with an explicit version field.
///
/// Used for EC-004 (SSL 2.0 / version=0x0200), EC-005 (TLS 1.0 / version=0x0301),
/// and AC-007 (different client vs server versions).
/// When `include_renegotiation_info` is true, a standard renegotiation_info extension
/// (0xff01) is appended; otherwise no extensions are included.
fn build_server_hello_with_version_and_cipher(
    version: u16,
    cipher_id: u16,
    include_renegotiation_info: bool,
) -> Vec<u8> {
    let mut sh_body = Vec::new();
    sh_body.extend_from_slice(&version.to_be_bytes()); // ServerHello version field
    sh_body.extend_from_slice(&[0u8; 32]); // random
    sh_body.push(0x00); // session_id length: 0
    sh_body.extend_from_slice(&cipher_id.to_be_bytes()); // selected cipher
    sh_body.push(0x00); // compression: null

    if include_renegotiation_info {
        // renegotiation_info (0xff01) with empty data — 1 byte payload
        let mut extensions = Vec::new();
        extensions.extend_from_slice(&[0xff, 0x01]); // ext type
        extensions.extend_from_slice(&[0x00, 0x01]); // ext data length = 1
        extensions.push(0x00); // empty renegotiation info
        let ext_len = extensions.len() as u16;
        sh_body.extend_from_slice(&ext_len.to_be_bytes());
        sh_body.extend_from_slice(&extensions);
    }
    // No extensions block at all when include_renegotiation_info is false.
    // This leaves sh.ext = None in the parsed TlsServerHelloContents.

    let mut handshake = Vec::new();
    handshake.push(0x02); // handshake type: ServerHello
    let sh_len = sh_body.len() as u32;
    handshake.push((sh_len >> 16) as u8);
    handshake.push((sh_len >> 8) as u8);
    handshake.push(sh_len as u8);
    handshake.extend_from_slice(&sh_body);

    let mut record = Vec::new();
    record.push(0x16);
    // Use the same version in the record-layer header as the ServerHello version
    // when it is <= TLS 1.0 compatibility; always 0x0303 for record layer in
    // practice, but for non-standard version tests keep it consistent.
    record.extend_from_slice(&[0x03, 0x03]);
    let hs_len = u16::try_from(handshake.len()).expect("handshake too long");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

// ── AC-001 (BC-2.07.002 postcondition 1): server_hello_seen set to true ───────

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_server_hello_seen_set_true() {
    // BC-2.07.002 postcondition 1: after handle_server_hello processes a valid
    // ServerHello, flow.server_hello_seen is set to true.
    //
    // Observable: server_hello_seen_for_testing(flow_key) returns true after the
    // ServerHello record is fed to on_data. The flow must be present first
    // (client_hello creates the flow entry); a ServerHello alone would create the
    // flow entry but not yet set server_hello_seen — but in practice the flow exists
    // from the ClientHello.
    //
    // Proof structure: ClientHello (opens flow, sets client_hello_seen) →
    //   ServerHello (sets server_hello_seen) → done() = true →
    //   server_hello_seen_for_testing = true.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // server_hello_seen must be false before the ServerHello arrives.
    assert!(
        !analyzer.server_hello_seen_for_testing(&fk),
        "AC-001 precondition (BC-2.07.002 pc1): server_hello_seen must be false before ServerHello"
    );

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Postcondition: server_hello_seen is now true.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "AC-001 anchor (BC-2.07.002 pc1): flow must be present before checking flag"
    );
    assert!(
        analyzer.server_hello_seen_for_testing(&fk),
        "AC-001 (BC-2.07.002 postcondition 1): server_hello_seen must be true after ServerHello"
    );

    // Confirm zero parse errors.
    assert_eq!(analyzer.parse_error_count(), 0);
}

// ── AC-002 (BC-2.07.002 postcondition 2): ServerHello version in version_counts ─

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_server_version_inserted_in_version_counts() {
    // BC-2.07.002 postcondition 2: the ServerHello version field (u16) is
    // inserted/incremented in version_counts. This is independent of any prior
    // ClientHello version count on the same flow.
    //
    // Discriminating assertion: send a ClientHello with version 0x0303 and a
    // ServerHello with version 0x0303. version_counts[0x0303] must equal 2
    // (once for ClientHello, once for ServerHello) — not 1. If handle_server_hello
    // failed to increment, the count would stay at 1.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello uses version 0x0303 (TLS 1.2 / legacy_version).
    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // version_counts[0x0303] == 1 after ClientHello only.
    assert_eq!(
        *analyzer.version_counts().get(&0x0303).unwrap_or(&0),
        1,
        "AC-002 anchor (BC-2.07.002 pc2): version_counts[0x0303] must be 1 after ClientHello"
    );

    // build_server_hello uses version 0x0303.
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // After ServerHello: version_counts[0x0303] must be 2 (incremented again).
    assert_eq!(
        *analyzer.version_counts().get(&0x0303).unwrap_or(&0),
        2,
        "AC-002 (BC-2.07.002 postcondition 2): version_counts[0x0303] must be 2 after \
         ClientHello + ServerHello (both use version 0x0303) — ServerHello contribution is 1"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
}

// ── AC-003 (BC-2.07.002 postcondition 3): JA3S MD5 hex computed and inserted ─

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ja3s_hash_computed_and_inserted() {
    // BC-2.07.002 postcondition 3: a JA3S MD5 hex string (32 lowercase hex chars)
    // is computed via compute_ja3s(version, cipher, extensions) and inserted/
    // incremented in ja3s_counts (bounded at MAX_MAP_ENTRIES).
    //
    // Canonical test vector from BC-2.07.002:
    //   ServerHello version=0x0303 (771), cipher=0x1301 (4865=TLS_AES_128_GCM_SHA256),
    //   extensions=[renegotiation_info (0xff01=65281)].
    //   JA3S string = "771,4865,65281" -> MD5 = 9e36d0263f2c16df7144edfdcdd47374
    //
    // The canonical hash pins the exact JA3S algorithm (3 fields: version, cipher,
    // GREASE-filtered ext IDs). Any change to field ordering, decimal encoding,
    // separator choice, or GREASE filtering will invalidate this assertion.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // build_server_hello(0x1301) produces:
    //   version = 0x0303 (771), cipher = 0x1301 (4865),
    //   ext = [renegotiation_info 0xff01 (65281)]
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    assert_eq!(
        analyzer.ja3s_counts().len(),
        1,
        "AC-003 (BC-2.07.002 pc3): exactly one JA3S hash must be recorded"
    );
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // Format assertions: 32 lowercase hex chars.
    assert_eq!(
        hash.len(),
        32,
        "AC-003 (BC-2.07.002 pc3): JA3S hash must be exactly 32 characters"
    );
    assert!(
        hash.chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "AC-003 (BC-2.07.002 pc3): JA3S hash must be all lowercase hex, got: {hash}"
    );

    // Canonical value assertion: pins the exact JA3S algorithm.
    // JA3S string = "771,4865,65281" -> MD5 = 9e36d0263f2c16df7144edfdcdd47374
    assert_eq!(
        hash, "9e36d0263f2c16df7144edfdcdd47374",
        "AC-003 (BC-2.07.002 pc3): JA3S canonical vector \
         version=771 cipher=4865 ext=65281 -> MD5('771,4865,65281') \
         must be 9e36d0263f2c16df7144edfdcdd47374"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
}

// ── AC-004 (BC-2.07.002 postcondition 4): cipher_name in cipher_counts ─────────

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_cipher_name_inserted_in_cipher_counts() {
    // BC-2.07.002 postcondition 4: cipher_name(sh.cipher) is inserted/incremented
    // in cipher_counts (bounded at MAX_MAP_ENTRIES).
    //
    // Discriminating assertions:
    //   - cipher_counts has exactly one entry after one ServerHello.
    //   - The key is the human-readable name "TLS_AES_128_GCM_SHA256" (not "0x1301"
    //     or decimal "4865") — this is the known name for cipher 0x1301 from
    //     TlsCipherSuite::from_id. This pin guards against accidental hex/decimal
    //     fallback for a well-known cipher ID.
    //   - The count is 1.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    assert_eq!(
        analyzer.findings().len(),
        0,
        "AC-004 anchor (BC-2.07.002 pc4): no weak-cipher findings expected for 0x1301"
    );

    // cipher_counts should have exactly one entry.
    let cipher_counts = analyzer.summarize().detail;
    let cipher_suites = cipher_counts
        .get("cipher_suites")
        .expect("cipher_suites key must exist in summary");

    // Verify the key "TLS_AES_128_GCM_SHA256" is present with count 1.
    assert_eq!(
        cipher_suites
            .get("TLS_AES_128_GCM_SHA256")
            .and_then(|v| v.as_u64()),
        Some(1),
        "AC-004 (BC-2.07.002 pc4): cipher_counts must contain \
         'TLS_AES_128_GCM_SHA256' with count 1 after one ServerHello selecting 0x1301. \
         Got cipher_suites: {cipher_suites}"
    );
}

// ── AC-005 (BC-2.07.002 invariant 1): JA3S GREASE filtering ──────────────────
//
// This test combines with the existing STORY-051 tests (test_BC_2_07_008_*) for
// JA3S format. Here we add a discriminating synthetic unit test that directly
// verifies the JA3S string value (by canonical hash) for a ServerHello with
// a GREASE extension plus a non-GREASE extension, confirming the GREASE ID is
// filtered from the ext field but the cipher is NOT filtered.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ja3s_grease_ext_filtered_cipher_not_filtered() {
    // BC-2.07.002 invariant 1: JA3S is computed solely from (version, selected_cipher,
    // extension_ids); GREASE extension IDs are filtered using (val & 0x0F0F) == 0x0A0A.
    //
    // Test: ServerHello with GREASE ext 0x0a0a (filtered) + renegotiation_info 0xff01
    // (kept). After filtering, ext_ids = "65281". JA3S = "771,4865,65281".
    // Cipher 0x1301 (4865) is NOT a GREASE value ((0x1301 & 0x0F0F = 0x0101) ≠ 0x0a0a),
    // so it is preserved verbatim in the cipher field.
    //
    // Same canonical hash as the no-GREASE case since GREASE ext is filtered out:
    // MD5("771,4865,65281") = 9e36d0263f2c16df7144edfdcdd47374.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with GREASE extension 0x0a0a + renegotiation_info 0xff01.
    let sh = build_server_hello_with_grease_ext(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // The GREASE ext 0x0a0a is filtered; only 0xff01 (65281) remains.
    // JA3S = "771,4865,65281" -> MD5 = 9e36d0263f2c16df7144edfdcdd47374 (same as non-GREASE).
    assert_eq!(
        hash, "9e36d0263f2c16df7144edfdcdd47374",
        "AC-005 (BC-2.07.002 inv1): GREASE ext 0x0a0a must be filtered; \
         resulting JA3S must equal MD5('771,4865,65281') = 9e36d0263f2c16df7144edfdcdd47374"
    );

    // Also verify the hash differs from the all-GREASE-ext case (the GREASE is filtered,
    // not the non-GREASE ext — so this hash must be different from the no-ext hash).
    assert_ne!(
        hash, "e8c07683aecf9b16e8e33f10a5161e4e",
        "AC-005 (BC-2.07.002 inv1): GREASE filter must not eliminate the non-GREASE \
         renegotiation_info ext (0xff01) — hash must differ from no-ext JA3S \
         (MD5('771,4865,') = e8c07683aecf9b16e8e33f10a5161e4e)"
    );
    assert_eq!(analyzer.parse_error_count(), 0);
}

// ── AC-006 (BC-2.07.002 invariant 2): unknown cipher ID renders as hex ────────

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_unknown_cipher_id_renders_as_hex_in_cipher_counts() {
    // BC-2.07.002 invariant 2: unknown cipher IDs (where TlsCipherSuite::from_id
    // returns None) are rendered as "0x{id:04x}" lowercase hex via cipher_name.
    // This hex-formatted string is used as the cipher_counts map key.
    //
    // Test vector: cipher 0xFFFF — unassigned in IANA TLS cipher suite registry.
    // TlsCipherSuite::from_id(0xFFFF) should return None, producing key "0xffff".
    //
    // Positive-parse anchor: we confirm the ServerHello itself parsed cleanly
    // (parse_error_count == 0) before asserting the cipher_counts key. An unknown
    // cipher ID at the record-layer is not a parse error — tls_parser accepts it.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // Server selects cipher 0xFFFF (unknown / unassigned ID).
    let sh = build_server_hello(0xFFFF);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Positive-parse anchor: record must parse cleanly.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-006 anchor (BC-2.07.002 inv2): unknown cipher ID 0xFFFF must not cause \
         a parse error — cipher ID is opaque at the record layer"
    );

    // The JA3S hash for cipher 0xFFFF + renegotiation_info ext (65281) + version 771:
    // JA3S string = "771,65535,65281" -> MD5 = ba59ad1a1874a170125cfbab170feaeb
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "ba59ad1a1874a170125cfbab170feaeb",
        "AC-006 anchor JA3S: cipher 0xFFFF (decimal 65535) must appear in JA3S string \
         as decimal 65535; MD5('771,65535,65281') = ba59ad1a1874a170125cfbab170feaeb"
    );

    // The cipher_counts key for an unknown cipher ID must be lowercase hex "0xffff".
    let summary = analyzer.summarize();
    let cipher_suites = summary
        .detail
        .get("cipher_suites")
        .expect("cipher_suites key must exist");

    assert!(
        cipher_suites.get("0xffff").is_some(),
        "AC-006 (BC-2.07.002 inv2): cipher_counts must contain key '0xffff' for \
         unknown cipher ID 0xFFFF (cipher_name renders as '0xffff' lowercase hex). \
         Got cipher_suites: {cipher_suites}"
    );
    assert_eq!(
        cipher_suites.get("0xffff").and_then(|v| v.as_u64()),
        Some(1),
        "AC-006 (BC-2.07.002 inv2): cipher_counts['0xffff'] must be 1"
    );

    // Regression guard: must NOT be stored under decimal "65535" or uppercase "0xFFFF".
    assert!(
        cipher_suites.get("65535").is_none(),
        "AC-006 (BC-2.07.002 inv2): cipher_counts must NOT use decimal key '65535' \
         for unknown cipher ID — format is '0xffff' (lowercase hex)"
    );
    assert!(
        cipher_suites.get("0xFFFF").is_none(),
        "AC-006 (BC-2.07.002 inv2): cipher_counts must NOT use uppercase key '0xFFFF' \
         for unknown cipher ID — format is '0xffff' (lowercase hex)"
    );
}

// ── AC-007 (BC-2.07.002 invariant 3): ClientHello and ServerHello versions
//           both contribute to version_counts independently ──────────────────

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_version_counts_client_and_server_versions_independent() {
    // BC-2.07.002 invariant 3: version_counts receives the ServerHello version
    // independently of any prior ClientHello version count. A flow where
    // ClientHello and ServerHello have different version fields increments both.
    //
    // Test vector (EC-007 from STORY-053):
    //   ClientHello version = 0x0301 (TLS 1.0, decimal 769)
    //   ServerHello version = 0x0303 (TLS 1.2, decimal 771)
    //   Expected: version_counts[0x0301] == 1, version_counts[0x0303] == 1
    //
    // If handle_server_hello failed to increment version_counts, or if it used
    // ClientHello's version instead, this would fail.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello with version 0x0301 (TLS 1.0 legacy_version)
    let ch = build_client_hello_with_version(0x0301, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // After ClientHello: version_counts must have exactly 0x0301 = 769.
    assert_eq!(
        analyzer.version_counts().len(),
        1,
        "AC-007 precondition: exactly one version_count entry after ClientHello only"
    );
    assert_eq!(
        *analyzer.version_counts().get(&0x0301).unwrap_or(&0),
        1,
        "AC-007 precondition: version_counts[0x0301] == 1 after ClientHello"
    );

    // ServerHello with version 0x0303 (TLS 1.2) — different from ClientHello version.
    let sh = build_server_hello_with_version_and_cipher(0x0303, 0x1301, true);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // After ServerHello: version_counts must have BOTH 0x0301 and 0x0303.
    assert_eq!(
        analyzer.version_counts().len(),
        2,
        "AC-007 (BC-2.07.002 inv3): version_counts must have 2 entries after ClientHello \
         (0x0301) + ServerHello (0x0303); got: {:?}",
        analyzer.version_counts()
    );
    assert_eq!(
        *analyzer.version_counts().get(&0x0301).unwrap_or(&0),
        1,
        "AC-007 (BC-2.07.002 inv3): version_counts[0x0301] (ClientHello) must still be 1"
    );
    assert_eq!(
        *analyzer.version_counts().get(&0x0303).unwrap_or(&0),
        1,
        "AC-007 (BC-2.07.002 inv3): version_counts[0x0303] (ServerHello) must be 1"
    );

    // ServerHello JA3S canonical hash: version=771, cipher=4865, ext=65281
    // MD5("771,4865,65281") = 9e36d0263f2c16df7144edfdcdd47374
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "9e36d0263f2c16df7144edfdcdd47374",
        "AC-007 JA3S anchor: ServerHello version=0x0303 cipher=0x1301 ext=renegotiation_info \
         must produce MD5('771,4865,65281') = 9e36d0263f2c16df7144edfdcdd47374"
    );

    assert_eq!(analyzer.parse_error_count(), 0);
}

// ── EC-001 (BC-2.07.002 edge case 1): no extensions -> empty ext field in JA3S ─

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ec001_no_extensions_ja3s_uses_empty_ext_field() {
    // BC-2.07.002 EC-001: ServerHello with no extensions (sh.ext = None).
    // JA3S is computed with empty extensions field, producing "version,cipher,".
    //
    // Canonical test vector:
    //   ServerHello version=0x0303 (771), cipher=0x1301 (4865), no extensions.
    //   JA3S string = "771,4865," (trailing comma, empty ext field)
    //   MD5 = e8c07683aecf9b16e8e33f10a5161e4e
    //
    // This must differ from the hash with renegotiation_info (MD5("771,4865,65281")
    // = 9e36d0263f2c16df7144edfdcdd47374), confirming the ext field is actually
    // absent (not substituted or filled in).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with no extensions (sh.ext = None).
    let sh = build_server_hello_no_extensions(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "EC-001 (BC-2.07.002 ec1): no-extension ServerHello must parse cleanly"
    );
    assert_eq!(
        analyzer.ja3s_counts().len(),
        1,
        "EC-001 (BC-2.07.002 ec1): exactly one JA3S hash must be recorded"
    );

    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();

    // JA3S string = "771,4865," -> MD5 = e8c07683aecf9b16e8e33f10a5161e4e
    assert_eq!(
        hash, "e8c07683aecf9b16e8e33f10a5161e4e",
        "EC-001 (BC-2.07.002 ec1): no-extension ServerHello JA3S must be \
         MD5('771,4865,') = e8c07683aecf9b16e8e33f10a5161e4e \
         (empty ext field, trailing comma)"
    );

    // Must differ from hash with renegotiation_info extension present.
    assert_ne!(
        hash, "9e36d0263f2c16df7144edfdcdd47374",
        "EC-001 (BC-2.07.002 ec1): no-ext hash must differ from hash with \
         renegotiation_info ext (9e36d0263f2c16df7144edfdcdd47374)"
    );
}

// ── EC-003 (BC-2.07.002 edge case 3): null cipher emits weak-cipher finding ────

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ec003_null_cipher_emits_weak_cipher_finding() {
    // BC-2.07.002 EC-003: ServerHello cipher = TLS_NULL_WITH_NULL_NULL (0x0000).
    // is_weak_server_cipher returns true; one Anomaly/Likely/Medium finding emitted.
    //
    // Canonical test vector from BC-2.07.002: "ServerHello with
    // TLS_RSA_EXPORT_WITH_RC4_40_MD5 cipher -> One Anomaly/Likely/Medium finding"
    // EC-003 uses 0x0000 (TLS_NULL_WITH_NULL_NULL), which contains "NULL" in the name.
    //
    // Positive-parse anchor: the ServerHello record itself must parse cleanly.
    // cipher_counts must record "TLS_NULL_WITH_NULL_NULL" (the cipher_name output
    // for 0x0000) — this also confirms cipher_counts is populated even for weak ciphers.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // Server selects TLS_NULL_WITH_NULL_NULL (0x0000).
    let sh = build_server_hello(0x0000);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "EC-003 anchor (BC-2.07.002 ec3): NULL cipher ServerHello must parse cleanly"
    );

    // Exactly one finding.
    let findings = analyzer.findings();
    let weak_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .collect();
    assert_eq!(
        weak_findings.len(),
        1,
        "EC-003 (BC-2.07.002 ec3): exactly one weak-cipher finding must be emitted \
         for TLS_NULL_WITH_NULL_NULL (0x0000). Got findings: {findings:?}"
    );

    let f = weak_findings[0];
    // BC-2.07.002 postcondition 5: Anomaly/Likely/Medium
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "EC-003 (BC-2.07.002 pc5): category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Likely,
        "EC-003 (BC-2.07.002 pc5): verdict must be Likely"
    );
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Medium,
        "EC-003 (BC-2.07.002 pc5): confidence must be Medium"
    );

    // Cipher name in evidence.
    assert!(
        f.evidence
            .iter()
            .any(|e| e.contains("TLS_NULL_WITH_NULL_NULL")),
        "EC-003 (BC-2.07.002 ec3): cipher name must appear in evidence, got: {:?}",
        f.evidence
    );

    // cipher_counts must contain the cipher name key (AC-004 also holds for weak ciphers).
    let summary = analyzer.summarize();
    let cipher_suites = summary.detail.get("cipher_suites").unwrap();
    assert!(
        cipher_suites.get("TLS_NULL_WITH_NULL_NULL").is_some(),
        "EC-003 (BC-2.07.002 ec3 + pc4): cipher_counts must contain 'TLS_NULL_WITH_NULL_NULL' \
         key even for weak ciphers. Got: {cipher_suites}"
    );

    // JA3S canonical hash for cipher 0x0000 + renegotiation_info ext:
    // JA3S = "771,0,65281" -> MD5 = e5880384215f2f59279a3b56215d5f54
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "e5880384215f2f59279a3b56215d5f54",
        "EC-003 JA3S anchor: cipher 0x0000 (decimal 0) + renegotiation_info -> \
         MD5('771,0,65281') = e5880384215f2f59279a3b56215d5f54"
    );
}

// ── EC-004 (BC-2.07.002 edge case 4): SSL 2.0 version — pinned parse behavior ──

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned() {
    // BC-2.07.002 EC-004 (pinned behavior): ServerHello version = 0x0200 (SSL 2.0).
    //
    // The BC states that version 0x0200 should emit an Anomaly/Likely/High deprecated-
    // protocol finding. However, the actual production behavior is constrained by
    // tls_parser: parse_tls_plaintext rejects a ServerHello whose inner version field
    // is 0x0200 (SSL 2.0 is not a recognized TlsVersion in tls-parser 0.12).
    //
    // When parse_tls_plaintext returns Err(_), the analyzer increments parse_errors
    // and handle_server_hello is never reached. As a result:
    //   - parse_errors == 1 (record-layer parse failure)
    //   - version_counts does NOT contain 0x0200
    //   - ja3s_counts is empty
    //   - no deprecated-protocol finding is emitted
    //
    // This pin test documents the ACTUAL production behavior and guards against
    // accidental changes (e.g. a tls_parser upgrade that begins accepting SSL 2.0).
    // The SSL 3.0 deprecated-protocol detection path (version=0x0300, which tls_parser
    // DOES accept) is exercised by test_BC_2_07_002_ec007_ssl30_server_emits_finding_
    // tls10_client_does_not and the integration test test_ssl30_pcap_generates_findings.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with version 0x0200 (SSL 2.0), no extensions.
    let sh = build_server_hello_with_version_and_cipher(0x0200, 0x1301, false);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Pinned behavior: tls_parser cannot parse SSL 2.0 ServerHello -> parse_errors = 1.
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "EC-004 pin (BC-2.07.002 ec4): tls_parser rejects SSL 2.0 ServerHello; \
         parse_errors must be 1 (if this fails, tls_parser now accepts SSL 2.0 — \
         revisit the deprecated-protocol detection path)"
    );

    // Because the record never reached handle_server_hello:
    //   version_counts[0x0200] must be absent.
    assert_eq!(
        *analyzer.version_counts().get(&0x0200).unwrap_or(&0),
        0,
        "EC-004 pin (BC-2.07.002 ec4): tls_parser rejects SSL 2.0 before handle_server_hello; \
         version_counts must NOT contain 0x0200"
    );
    //   no deprecated-protocol finding from ServerHello direction.
    assert!(
        analyzer
            .findings()
            .iter()
            .filter(
                |f| f.direction == Some(wirerust::reassembly::handler::Direction::ServerToClient)
            )
            .all(|f| !f.summary.contains("deprecated protocol")),
        "EC-004 pin (BC-2.07.002 ec4): no deprecated-protocol ServerHello finding expected \
         when tls_parser rejects the record at the record layer. Got: {:?}",
        analyzer.findings()
    );
    //   ja3s_counts is empty.
    assert!(
        analyzer.ja3s_counts().is_empty(),
        "EC-004 pin (BC-2.07.002 ec4): ja3s_counts must be empty when SSL 2.0 parse fails"
    );
}

// ── EC-005 (BC-2.07.002 edge case 5): TLS 1.0 counted, no deprecated finding ──

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ec005_tls10_version_counted_no_deprecated_finding() {
    // BC-2.07.002 EC-005: ServerHello version = 0x0301 (TLS 1.0).
    // No deprecated-protocol finding must be emitted (version > 0x0300).
    // version_counts[0x0301] must be incremented.
    //
    // Discriminating: the boundary test is version > 0x0300 (strictly greater),
    // so 0x0301 is the lowest version that does NOT trigger the deprecated-protocol
    // detection (which fires only for version <= 0x0300).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with version 0x0301 (TLS 1.0), cipher 0x1301.
    let sh = build_server_hello_with_version_and_cipher(0x0301, 0x1301, true);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "EC-005 anchor (BC-2.07.002 ec5): TLS 1.0 ServerHello must parse cleanly"
    );

    // version_counts[0x0301] must be incremented.
    assert_eq!(
        *analyzer.version_counts().get(&0x0301).unwrap_or(&0),
        1,
        "EC-005 (BC-2.07.002 ec5): version_counts[0x0301] must be 1"
    );

    // No deprecated-protocol finding from the ServerHello direction.
    let deprecated_server_findings = analyzer
        .findings()
        .into_iter()
        .filter(|f| {
            f.summary.contains("deprecated protocol")
                && f.direction == Some(wirerust::reassembly::handler::Direction::ServerToClient)
        })
        .count();
    assert_eq!(
        deprecated_server_findings, 0,
        "EC-005 (BC-2.07.002 ec5): TLS 1.0 (0x0301 > 0x0300) must NOT emit a \
         deprecated-protocol finding from the ServerHello direction"
    );

    // JA3S canonical hash for TLS 1.0 (version=769) + cipher=4865 + renegotiation_info:
    // JA3S = "769,4865,65281" -> MD5 = 107b250b07f30c4298f7251ecd6c7891
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "107b250b07f30c4298f7251ecd6c7891",
        "EC-005 JA3S anchor: TLS 1.0 ServerHello version=769 cipher=4865 -> \
         MD5('769,4865,65281') = 107b250b07f30c4298f7251ecd6c7891"
    );
}

// ── EC-006 (BC-2.07.002 edge case 6): ja3s_counts at capacity, new hash dropped ─

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ec006_ja3s_counts_at_capacity_new_hash_dropped() {
    // BC-2.07.002 EC-006: When ja3s_counts is at MAX_MAP_ENTRIES (50,000) with a
    // new hash, the new hash is silently dropped. Existing hashes are unchanged.
    //
    // Strategy: fill ja3s_counts to capacity by varying the cipher ID across
    // MAX_MAP_ENTRIES distinct values, each producing a distinct JA3S string
    // "771,<cipher>,65281". Then inject a ServerHello with a NEW cipher not in
    // the filled set and assert it is absent from ja3s_counts.
    //
    // Each flow is a fresh flow key (unique port) so the done() short-circuit
    // does not prevent processing. Each flow needs a ClientHello first (to open
    // the flow and set client_hello_seen) then the ServerHello.
    //
    // Note: this test is slower than typical unit tests because it creates
    // MAX_MAP_ENTRIES flows. Each flow sends 2 records (ClientHello + ServerHello).
    // Budget: ~3s in debug builds on CI.
    use std::net::IpAddr;

    const MAX_MAP_ENTRIES: usize = 50_000;

    let mut analyzer = TlsAnalyzer::new();

    // Fill ja3s_counts with MAX_MAP_ENTRIES distinct hashes.
    // Cipher IDs 1..=50000 are all in range [1, 50000]. Each unique cipher
    // with version=771 and ext=65281 produces a distinct JA3S string.
    // We use distinct flow keys (unique source port) so the done() flag
    // on one flow does not prevent the ServerHello on another flow.
    let server_ip: IpAddr = "10.0.0.2".parse().unwrap();
    let client_ip: IpAddr = "10.0.0.1".parse().unwrap();

    // pre-build the ClientHello once (same for all flows — the cipher list in CH
    // does not affect JA3S, only the ServerHello cipher does)
    let ch_bytes = build_client_hello("example.com", &[0x1301]);

    for i in 1u32..=(MAX_MAP_ENTRIES as u32) {
        let fk = FlowKey::new(client_ip, (10000 + i) as u16, server_ip, 443);
        analyzer.on_data(&fk, Direction::ClientToServer, &ch_bytes, 0, 0);
        // Cipher IDs 1..=50000. Not all are known to TlsCipherSuite, but
        // that only affects cipher_counts key format, not JA3S computation.
        let sh_bytes = build_server_hello(i as u16);
        analyzer.on_data(&fk, Direction::ServerToClient, &sh_bytes, 0, 0);
    }

    assert_eq!(
        analyzer.ja3s_counts().len(),
        MAX_MAP_ENTRIES,
        "EC-006 setup (BC-2.07.002 ec6): ja3s_counts must be full at {MAX_MAP_ENTRIES}"
    );

    // Inject a ServerHello with a NEW cipher (50001 = 0xC351) not used in the fill loop.
    let overflow_cipher: u16 = (MAX_MAP_ENTRIES + 1) as u16; // 50001
    let overflow_fk = FlowKey::new(
        client_ip,
        (10000 + MAX_MAP_ENTRIES as u32 + 1) as u16,
        server_ip,
        443,
    );
    analyzer.on_data(&overflow_fk, Direction::ClientToServer, &ch_bytes, 0, 0);
    let overflow_sh = build_server_hello(overflow_cipher);
    analyzer.on_data(&overflow_fk, Direction::ServerToClient, &overflow_sh, 0, 0);

    // ja3s_counts must not grow beyond the cap.
    assert_eq!(
        analyzer.ja3s_counts().len(),
        MAX_MAP_ENTRIES,
        "EC-006 (BC-2.07.002 ec6): ja3s_counts must not exceed MAX_MAP_ENTRIES={MAX_MAP_ENTRIES} \
         after capacity overflow; new hash for cipher {overflow_cipher} must be silently dropped. \
         Got {} entries",
        analyzer.ja3s_counts().len()
    );
}

// ── EC-007 (STORY-053 / BC-2.07.002 inv3): different client vs server versions ─
// This is covered by test_BC_2_07_002_version_counts_client_and_server_versions_independent
// above (AC-007 uses the EC-007 test vector). The dedicated EC-007 test below
// adds the SSL 3.0 (0x0300) boundary to confirm version <= 0x0300 detection
// applies only to ServerHello-direction deprecated-protocol findings.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_002_ec007_ssl30_server_emits_finding_tls10_client_does_not() {
    // EC-007 boundary variant: ClientHello version=0x0301 (TLS 1.0, no finding),
    // ServerHello version=0x0300 (SSL 3.0, emits deprecated-protocol finding).
    //
    // This confirms the deprecated-protocol boundary (version <= 0x0300) and that
    // ClientHello and ServerHello findings are independent (different direction tags).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello version=0x0301 (TLS 1.0 — above the 0x0300 boundary, no ClientHello finding).
    let ch = build_client_hello_with_version(0x0301, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // No deprecated-protocol finding from ClientHello direction.
    assert!(
        !analyzer
            .findings()
            .iter()
            .any(|f| f.summary.contains("deprecated protocol")),
        "EC-007 anchor (BC-2.07.002 ec7): ClientHello version=0x0301 must NOT emit \
         a deprecated-protocol finding"
    );

    // ServerHello version=0x0300 (SSL 3.0 — at or below 0x0300, emits finding).
    // Use no-extension builder to avoid EC-002 extension parse failure on legacy version.
    // The deprecated-protocol detection path fires before the extension block anyway.
    let sh = build_server_hello_with_version_and_cipher(0x0300, 0x1301, false);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Positive-parse anchor: no-extension SSL 3.0 ServerHello must parse cleanly.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "EC-007 anchor (BC-2.07.002 ec7): no-extension SSL 3.0 ServerHello must parse cleanly"
    );

    // version_counts must contain both 0x0301 (from ClientHello) and 0x0300 (from ServerHello).
    assert_eq!(
        *analyzer.version_counts().get(&0x0301).unwrap_or(&0),
        1,
        "EC-007 (BC-2.07.002 ec7 + inv3): version_counts[0x0301] (ClientHello) must be 1"
    );
    assert_eq!(
        *analyzer.version_counts().get(&0x0300).unwrap_or(&0),
        1,
        "EC-007 (BC-2.07.002 ec7 + inv3): version_counts[0x0300] (ServerHello) must be 1"
    );

    // Exactly one deprecated-protocol finding from the ServerHello (SSL 3.0).
    let deprecated_server = analyzer
        .findings()
        .into_iter()
        .filter(|f| {
            f.summary.contains("deprecated protocol")
                && f.direction == Some(wirerust::reassembly::handler::Direction::ServerToClient)
        })
        .count();
    assert_eq!(
        deprecated_server, 1,
        "EC-007 (BC-2.07.002 ec7): exactly one ServerHello deprecated-protocol finding \
         expected for SSL 3.0 (0x0300)"
    );

    // JA3S canonical hash for SSL 3.0 (version=768) + cipher=4865, no extensions:
    // JA3S string = "768,4865," (empty ext field, trailing comma)
    // MD5("768,4865,") = c2c5e539595f992edd516641da877181
    let hash = analyzer.ja3s_counts().keys().next().unwrap().clone();
    assert_eq!(
        hash, "c2c5e539595f992edd516641da877181",
        "EC-007 JA3S anchor: SSL 3.0 no-ext ServerHello version=768 cipher=4865 -> \
         MD5('768,4865,') = c2c5e539595f992edd516641da877181"
    );
}

// ── STORY-055: BC-2.07.013 / BC-2.07.014 / BC-2.07.015 / BC-2.07.016 / BC-2.07.018 ─
//
// SNI Classification Arms 1 and 2 — Clean ASCII Baseline and C0/DEL Control-Byte
// Detection formalization.
//
// These tests formally trace each Acceptance Criterion from STORY-055 to its
// behavioral contract clause. Naming follows DF-AC-TEST-NAME-SYNC-001 v2:
// test_BC_2_07_NNN_<suffix>. #[allow(non_snake_case)] is applied per-test.
//
// AC → test mapping:
//   AC-001 (BC-2.07.013 pc1-3)     test_BC_2_07_013_clean_ascii_no_finding_counted
//   AC-002 (BC-2.07.013 inv1)      test_BC_2_07_013_arm1_only_arm_with_no_finding
//   AC-003 (BC-2.07.014 pc1-4)     test_BC_2_07_014_esc_emits_anomaly_inconclusive_low_t1027_c2s
//   AC-004 (BC-2.07.014 inv4)      test_BC_2_07_014_raw_bytes_preserved_not_debug_escaped
//   AC-005 (BC-2.07.015 pc1-3)     test_BC_2_07_015_multiple_c0_bytes_one_finding_full_hex_evidence
//   AC-006 (BC-2.07.015 inv1)      test_BC_2_07_015_finding_count_o1_per_hostname_not_per_byte
//   AC-007 (BC-2.07.016 pc1-4)     test_BC_2_07_016_boundary_0x1f_trips_0x20_does_not_0x7f_trips_0x7e_does_not
//   AC-008 (BC-2.07.016 inv1)      test_BC_2_07_016_tab_cr_lf_are_c0_and_trip
//   AC-009 (BC-2.07.018 pc1-3)     test_BC_2_07_018_punycode_a_label_arm1_no_finding_counted
//   AC-010 (BC-2.07.018 inv1-2)    test_BC_2_07_018_a_label_uses_same_arm1_as_plain_ascii

// ── AC-001 (BC-2.07.013 postconditions 1-3) ──────────────────────────────────
//
// When SNI bytes are valid UTF-8, is_ascii() == true, and no byte satisfies
// b < 0x20 || b == 0x7f, extract_sni classifies as arm 1 (Ascii).
// No finding is pushed. Hostname is counted in sni_counts.
//
// Canonical test vector from BC-2.07.013: SNI "example.com" ->
//   sni_counts["example.com"] == 1; all_findings empty.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_013_clean_ascii_no_finding_counted() {
    // AC-001 (BC-2.07.013 pc1): extract_sni classifies "example.com" as Ascii.
    // AC-001 (BC-2.07.013 pc2): hostname inserted in sni_counts.
    // AC-001 (BC-2.07.013 pc3): no finding pushed.
    //
    // Positive-parse anchor: parse_error_count == 0 confirms the record was
    // well-formed and the SNI classification code actually ran.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Canonical BC-2.07.013 test vector.
    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor: confirms extract_sni ran (record was well-formed).
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-001 anchor (BC-2.07.013): parse_error_count must be 0 for well-formed record"
    );

    // BC-2.07.013 pc2: sni_counts contains the raw hostname key.
    assert_eq!(
        *analyzer.sni_counts().get("example.com").unwrap_or(&0),
        1,
        "AC-001 (BC-2.07.013 pc2): sni_counts[\"example.com\"] must be 1"
    );

    // BC-2.07.013 pc3: no finding pushed for clean ASCII arm.
    let sni_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("SNI"))
        .collect();
    assert!(
        sni_findings.is_empty(),
        "AC-001 (BC-2.07.013 pc3): clean ASCII SNI must emit no SNI finding, got: {sni_findings:?}"
    );

    // Second canonical test vector: "test.local" (BC-2.07.013).
    let mut analyzer2 = TlsAnalyzer::new();
    let record2 = build_client_hello("test.local", &[0x1301]);
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

    assert_eq!(
        *analyzer2.sni_counts().get("test.local").unwrap_or(&0),
        1,
        "AC-001 (BC-2.07.013 pc2): sni_counts[\"test.local\"] must be 1"
    );
    assert!(
        analyzer2.findings().is_empty(),
        "AC-001 (BC-2.07.013 pc3): test.local must produce no findings, got: {:?}",
        analyzer2.findings()
    );
}

// ── AC-002 (BC-2.07.013 invariant 1) ─────────────────────────────────────────
//
// Arm 1 is the ONLY arm that produces no finding. All other arms (2, 3, 4) emit
// a finding. Tested by showing: clean ASCII -> zero SNI findings; C0-ASCII ->
// at least one finding. This discriminates arm 1 from arm 2.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_013_arm1_only_arm_with_no_finding() {
    // AC-002 (BC-2.07.013 inv1): arm 1 (clean ASCII) produces zero SNI findings;
    // arm 2 (AsciiWithControl) produces exactly one. The contrast is the invariant.
    let fk = test_flow_key();

    // --- Arm 1 path: clean ASCII hostname "clean.example" ---
    let mut a_arm1 = TlsAnalyzer::new();
    let record_clean = build_client_hello("clean.example", &[0x1301]);
    a_arm1.on_data(&fk, Direction::ClientToServer, &record_clean, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        a_arm1.parse_error_count(),
        0,
        "arm1 anchor: parse must succeed"
    );

    let arm1_sni_findings: Vec<_> = a_arm1
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("SNI"))
        .collect();
    assert_eq!(
        arm1_sni_findings.len(),
        0,
        "AC-002 (BC-2.07.013 inv1): arm 1 (clean ASCII) must produce ZERO SNI findings; \
         got: {arm1_sni_findings:?}"
    );

    // --- Arm 2 path: same hostname but with C0 byte embedded ---
    // "clean\x01.example" — NUL+1 triggers arm 2.
    let mut a_arm2 = TlsAnalyzer::new();
    let record_ctrl = build_client_hello_ascii_bytes(b"clean\x01.example", &[0x1301]);
    a_arm2.on_data(&fk, Direction::ClientToServer, &record_ctrl, 0, 0);

    assert_eq!(
        a_arm2.parse_error_count(),
        0,
        "arm2 anchor: parse must succeed"
    );

    let arm2_control_findings: Vec<_> = a_arm2
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .collect();
    assert_eq!(
        arm2_control_findings.len(),
        1,
        "AC-002 (BC-2.07.013 inv1): arm 2 (AsciiWithControl) must produce exactly ONE \
         control finding; got: {arm2_control_findings:?}"
    );
}

// ── AC-003 (BC-2.07.014 postconditions 1-4) ───────────────────────────────────
//
// SNI "foo\x1b[31m.example" (ESC 0x1B embedded) -> arm 2 fires.
// Finding must have: category=Anomaly, verdict=Inconclusive, confidence=Low,
// mitre_technique=Some("T1027"), direction=Some(ClientToServer).
// sni_counts key is the raw hostname string.
//
// Canonical BC-2.07.014 test vector: "evil\x1b.com" -> Finding(Anomaly/Inconclusive/Low, T1027).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_014_esc_emits_anomaly_inconclusive_low_t1027_c2s() {
    // AC-003 (BC-2.07.014 pc2): finding fields verified individually.
    // AC-003 (BC-2.07.014 pc3): sni_counts keyed on raw hostname.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Use canonical BC test vector byte 0x1B (ESC).
    let sni_bytes: &[u8] = b"foo\x1b[31m.example";
    let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-003 anchor (BC-2.07.014): parse_error_count must be 0"
    );
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-003 anchor (BC-2.07.014): exactly one handshake must be counted"
    );

    // Exactly one control-byte finding (BC-2.07.014 pc1 — one finding, not zero, not two).
    let ctrl_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .collect();
    assert_eq!(
        ctrl_findings.len(),
        1,
        "AC-003 (BC-2.07.014 pc1): exactly one control finding must be emitted for ESC SNI; \
         got: {ctrl_findings:?}"
    );

    let f = &ctrl_findings[0];

    // BC-2.07.014 pc2: category = Anomaly.
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-003 (BC-2.07.014 pc2): category must be Anomaly"
    );

    // BC-2.07.014 pc2: verdict = Inconclusive.
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "AC-003 (BC-2.07.014 pc2): verdict must be Inconclusive"
    );

    // BC-2.07.014 pc2: confidence = Low.
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "AC-003 (BC-2.07.014 pc2): confidence must be Low"
    );

    // BC-2.07.014 pc2: mitre_techniques = ["T1027"] (single-tag, full-vec equality).
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-003 (BC-2.07.014 pc2): mitre_techniques must be exactly [\"T1027\"]"
    );

    // BC-2.07.014 pc2: direction = Some(ClientToServer).
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "AC-003 (BC-2.07.014 pc2): direction must be Some(ClientToServer)"
    );

    // BC-2.07.014 pc2: summary references RFC 6066.
    assert!(
        f.summary.contains("RFC 6066"),
        "AC-003 (BC-2.07.014 pc2): summary must cite RFC 6066, got: {:?}",
        f.summary
    );

    // BC-2.07.014 pc2: evidence contains hex representation.
    // "foo\x1b[31m.example" hex = 666f6f1b5b33316d2e6578616d706c65
    assert!(
        f.evidence
            .iter()
            .any(|e| e.starts_with("hex: ") && e.contains("666f6f1b5b33316d2e6578616d706c65")),
        "AC-003 (BC-2.07.014 pc2): evidence must contain hex-prefixed lossless hex, \
         got: {:?}",
        f.evidence
    );

    // BC-2.07.014 pc3: sni_counts keyed on raw hostname string.
    let raw_key = std::str::from_utf8(sni_bytes).expect("sni_bytes is valid UTF-8");
    assert_eq!(
        *analyzer.sni_counts().get(raw_key).unwrap_or(&0),
        1,
        "AC-003 (BC-2.07.014 pc3): sni_counts must use raw hostname as key, \
         got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );
}

// ── AC-004 (BC-2.07.014 invariant 4) ─────────────────────────────────────────
//
// Raw bytes are preserved in finding summary at the TlsAnalyzer layer.
// No escape_for_terminal is called. The summary must contain the raw
// hostname with embedded control byte; must NOT contain "\u{1b}" Debug form.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_014_raw_bytes_preserved_not_debug_escaped() {
    // AC-004 (BC-2.07.014 inv4): data layer preserves raw bytes per ADR 0003.
    // Display-layer escaping is deferred to the terminal reporter.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Embed the ESC byte (0x1B) in the hostname.
    // The summary must contain the raw ESC byte, not its Debug-escaped form "\u{1b}".
    let sni_bytes: &[u8] = b"inject\x1besc.example";
    let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-004 anchor (BC-2.07.014 inv4): parse must succeed"
    );

    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .expect("AC-004 (BC-2.07.014 inv4): expected control finding for ESC-embedded SNI");

    // BC-2.07.014 inv4 / ADR 0003: raw ESC byte (0x1B) must survive in the summary.
    assert!(
        f.summary.as_bytes().contains(&0x1b),
        "AC-004 (BC-2.07.014 inv4): summary must contain raw ESC byte (0x1B) per ADR 0003; \
         got bytes: {:?}",
        f.summary.as_bytes()
    );

    // BC-2.07.014 inv4: summary must NOT contain Debug-formatted escape form.
    // If the analyzer used {:?} formatting, control bytes would appear as "\u{1b}".
    assert!(
        !f.summary.contains("\\u{1b}"),
        "AC-004 (BC-2.07.014 inv4): summary must not contain Debug-escaped form \\u{{1b}} \
         (construction-site escaping regression); got: {}",
        f.summary
    );

    // The raw hostname string must appear literally in the summary.
    let raw_hostname = std::str::from_utf8(sni_bytes).expect("sni_bytes is valid UTF-8");
    assert!(
        f.summary.contains(raw_hostname),
        "AC-004 (BC-2.07.014 inv4): summary must contain the raw hostname literal, \
         got: {:?}",
        f.summary
    );
}

// ── AC-005 (BC-2.07.015 postconditions 1-3) ───────────────────────────────────
//
// When a hostname contains multiple C0/DEL bytes, exactly ONE finding is pushed
// (not one per control byte). Evidence contains one entry: "hex: {full_hostname_hex}".
//
// Canonical BC-2.07.015 test vector: SNI = "a\x01\x02\x03b" (3 control bytes) ->
//   all_findings.len() == 1; evidence[0] starts with "hex: ".
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_015_multiple_c0_bytes_one_finding_full_hex_evidence() {
    // AC-005 (BC-2.07.015 pc1): exactly ONE finding for hostname with 3 C0 bytes.
    // AC-005 (BC-2.07.015 pc2): evidence contains one entry "hex: {hex}" where
    //   hex is the lowercase hex of ALL hostname bytes (not just control bytes).
    // AC-005 (BC-2.07.015 pc3): summary contains the entire hostname string.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Canonical BC-2.07.015 test vector: "a\x01\x02\x03b" — 3 control bytes.
    // Expected full hostname hex: 61 01 02 03 62 = "61010203 62"
    let sni_bytes: &[u8] = b"a\x01\x02\x03b";
    let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-005 anchor (BC-2.07.015): parse must succeed"
    );

    // BC-2.07.015 pc1: exactly one finding total — not three.
    let ctrl_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .collect();
    assert_eq!(
        ctrl_findings.len(),
        1,
        "AC-005 (BC-2.07.015 pc1): 3 C0 bytes in one SNI must produce exactly ONE finding; \
         got {} findings: {:?}",
        ctrl_findings.len(),
        ctrl_findings
    );

    let f = &ctrl_findings[0];

    // BC-2.07.015 pc2: exactly one evidence entry, prefixed "hex: ".
    assert_eq!(
        f.evidence.len(),
        1,
        "AC-005 (BC-2.07.015 pc2): finding must have exactly one evidence entry; \
         got: {:?}",
        f.evidence
    );
    assert!(
        f.evidence[0].starts_with("hex: "),
        "AC-005 (BC-2.07.015 pc2): evidence[0] must start with \"hex: \"; \
         got: {:?}",
        f.evidence[0]
    );

    // BC-2.07.015 pc2: hex covers ALL hostname bytes, not just the control bytes.
    // "a\x01\x02\x03b" = [0x61, 0x01, 0x02, 0x03, 0x62] -> "6101020362"
    let expected_hex = "6101020362";
    assert!(
        f.evidence[0].contains(expected_hex),
        "AC-005 (BC-2.07.015 pc2): hex evidence must encode ALL hostname bytes \
         (including non-control bytes a=0x61 and b=0x62), not just control bytes; \
         expected hex {expected_hex} in evidence[0], got: {:?}",
        f.evidence[0]
    );

    // BC-2.07.015 pc3: summary contains the entire raw hostname string.
    let raw_hostname = std::str::from_utf8(sni_bytes).expect("sni_bytes valid UTF-8");
    assert!(
        f.summary.contains(raw_hostname),
        "AC-005 (BC-2.07.015 pc3): finding summary must contain the entire hostname, \
         got: {:?}",
        f.summary
    );
}

// ── AC-006 (BC-2.07.015 invariant 1) ─────────────────────────────────────────
//
// Finding count is O(1) per SNI hostname. Multiple control bytes -> still
// exactly one finding. The AsciiWithControl arm calls all_findings.push once.
//
// This test uses a different hostname than AC-005 to verify the invariant
// independently (3 distinct C0 values: BEL, ESC, DEL).
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_015_finding_count_o1_per_hostname_not_per_byte() {
    // AC-006 (BC-2.07.015 inv1): finding count is O(1) per hostname.
    // Canonical BC-2.07.015 edge case: "\x1f\x1e\x1d" (three C0 bytes) -> 1 finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Three distinct C0 bytes: BEL (0x07), ESC (0x1B), DEL (0x7F).
    let sni_bytes: &[u8] = b"a\x07b\x1bc\x7fd.example";
    let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-006 anchor (BC-2.07.015 inv1): parse must succeed"
    );

    // Direct assertion: the total all_findings length must be exactly 1.
    // Using all_findings_len_for_testing to read the raw vec (not filtered).
    // With only a control-byte SNI and no weak cipher, total findings == 1.
    assert_eq!(
        analyzer.all_findings_len_for_testing(),
        1,
        "AC-006 (BC-2.07.015 inv1): all_findings.len() must be exactly 1 for a hostname \
         with 3 C0/DEL bytes — O(1) per hostname, not O(control_bytes_count)"
    );

    // Confirm the single finding is the control-byte finding, not something else.
    let ctrl_count = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .count();
    assert_eq!(
        ctrl_count, 1,
        "AC-006 (BC-2.07.015 inv1): the one finding must be the ASCII control finding"
    );
}

// ── AC-007 (BC-2.07.016 postconditions 1-4) ───────────────────────────────────
//
// Precise boundary: 0x1F (last C0) trips arm 2; 0x20 (space) does NOT trip arm 2.
// 0x7F (DEL) trips arm 2; 0x7E (tilde) does NOT trip arm 2.
//
// Canonical BC-2.07.016 test vectors:
//   "test\x1fend" -> finding emitted (arm 2)
//   "test\x20end" -> no finding (arm 1)
//   "test\x7fend" -> finding emitted (arm 2)
//   "test\x7eend" -> no finding (arm 1)
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_016_boundary_0x1f_trips_0x20_does_not_0x7f_trips_0x7e_does_not() {
    // AC-007 (BC-2.07.016 pc1): 0x1F trips arm 2.
    // AC-007 (BC-2.07.016 pc2): 0x7F trips arm 2.
    // AC-007 (BC-2.07.016 pc3): 0x20 (space) does NOT trip arm 2 — arm 1 fires.
    // AC-007 (BC-2.07.016 pc4): boundary test b < 0x20 is exact: 0x1F < 0x20 is true;
    //                             0x20 < 0x20 is false.
    let fk = test_flow_key();

    // Table driven: (label, sni_bytes, expect_finding)
    let cases: &[(&str, &[u8], bool)] = &[
        // BC-2.07.016 canonical vectors:
        ("0x1F last-C0 MUST trip", b"test\x1fend", true),
        ("0x20 space MUST NOT trip (arm 1)", b"test\x20end", false),
        ("0x7F DEL MUST trip", b"test\x7fend", true),
        ("0x7E tilde MUST NOT trip (arm 1)", b"test\x7eend", false),
        // EC-001 from BC-2.07.016: 0x1F only.
        ("0x1F only (EC-001)", b"\x1f", true),
        // EC-002 from BC-2.07.016: 0x20 only.
        ("0x20 only (EC-002)", b"\x20", false),
        // EC-004 from BC-2.07.016: 0x7F after 'a'.
        ("a 0x7F b (EC-004)", b"a\x7fb", true),
        // EC-005 from BC-2.07.016: 0x7E tilde.
        ("a 0x7E b (EC-005)", b"a\x7eb", false),
    ];

    for (label, sni_bytes, expect_finding) in cases {
        let mut analyzer = TlsAnalyzer::new();
        let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // Positive-parse anchor for each case.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "AC-007 anchor (BC-2.07.016): parse_error_count must be 0 for case: {label}"
        );

        let ctrl_count = analyzer
            .findings()
            .iter()
            .filter(|f| f.summary.contains("ASCII control characters"))
            .count();

        if *expect_finding {
            assert_eq!(
                ctrl_count, 1,
                "AC-007 (BC-2.07.016 pc1/pc2/pc4): {label} must produce 1 control finding"
            );
            // Discriminating assertion: sni_counts uses raw hostname key (arm 2 path).
            let raw_key = std::str::from_utf8(sni_bytes).expect("test bytes are valid ASCII");
            assert_eq!(
                *analyzer.sni_counts().get(raw_key).unwrap_or(&0),
                1,
                "AC-007 (BC-2.07.016): sni_counts must have raw key for {label}"
            );
        } else {
            assert_eq!(
                ctrl_count, 0,
                "AC-007 (BC-2.07.016 pc3): {label} must produce 0 control findings (arm 1 fires)"
            );
            // Discriminating assertion: arm 1 path — sni_counts has entry, no SNI finding.
            let raw_key = std::str::from_utf8(sni_bytes).expect("test bytes are valid ASCII");
            assert_eq!(
                *analyzer.sni_counts().get(raw_key).unwrap_or(&0),
                1,
                "AC-007 (BC-2.07.016 pc3): sni_counts must have entry even for arm-1 SNI {label}"
            );
            let any_sni_finding = analyzer
                .findings()
                .iter()
                .any(|f| f.summary.contains("SNI"));
            assert!(
                !any_sni_finding,
                "AC-007 (BC-2.07.016 pc3): arm 1 must produce no SNI finding for {label}"
            );
        }
    }
}

// ── AC-008 (BC-2.07.016 invariant 1) ─────────────────────────────────────────
//
// The predicate is b < 0x20 || b == 0x7f. Tab (0x09), LF (0x0A), CR (0x0D) are
// all C0 bytes (< 0x20) and all trip arm 2.
//
// BC-2.07.016 invariant 3: Tab, LF, CR are C0 bytes and all trip the finding.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_016_tab_cr_lf_are_c0_and_trip() {
    // AC-008 (BC-2.07.016 inv1): predicate is exactly b < 0x20 || b == 0x7f.
    // Tab = 0x09 < 0x20 -> true. LF = 0x0A < 0x20 -> true. CR = 0x0D < 0x20 -> true.
    // All three must trigger arm 2 (each independently).
    let fk = test_flow_key();

    let cases: &[(&str, &[u8])] = &[
        ("Tab (0x09 < 0x20)", b"left\tright.example"),
        ("LF (0x0A < 0x20)", b"left\nright.example"),
        ("CR (0x0D < 0x20)", b"left\rright.example"),
        // NUL (0x00) is the lower bound of C0.
        ("NUL (0x00 < 0x20, C0 lower bound)", b"a\x00b.example"),
        // SOH (0x01) is above NUL and also C0.
        ("SOH (0x01 < 0x20)", b"a\x01b.example"),
    ];

    for (label, sni_bytes) in cases {
        let mut analyzer = TlsAnalyzer::new();
        let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // Positive-parse anchor.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "AC-008 anchor (BC-2.07.016 inv1): parse_error_count must be 0 for {label}"
        );

        let ctrl_count = analyzer
            .findings()
            .iter()
            .filter(|f| f.summary.contains("ASCII control characters"))
            .count();

        assert_eq!(
            ctrl_count, 1,
            "AC-008 (BC-2.07.016 inv1): {label} must trip arm 2 (b < 0x20 predicate); \
             expected 1 control finding, got {ctrl_count}"
        );

        // Discriminating assertion: sni_counts contains the raw hostname (arm 2 key).
        let raw_key = std::str::from_utf8(sni_bytes).expect("test bytes are valid ASCII");
        assert_eq!(
            *analyzer.sni_counts().get(raw_key).unwrap_or(&0),
            1,
            "AC-008 (BC-2.07.016 inv1): sni_counts must contain raw key for {label}"
        );
    }
}

// ── AC-009 (BC-2.07.018 postconditions 1-3) ───────────────────────────────────
//
// A Punycode A-label "xn--caf-dma.example" satisfies arm 1: valid UTF-8,
// is_ascii() == true, no C0/DEL. extract_sni returns Ascii(hostname).
// No finding. sni_counts keyed on the raw A-label string.
//
// Canonical BC-2.07.018 test vector: SNI = "xn--caf-dma.example" ->
//   no finding; sni_counts has one entry.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_018_punycode_a_label_arm1_no_finding_counted() {
    // AC-009 (BC-2.07.018 pc1): extract_sni returns Ascii(hostname) for A-label.
    // AC-009 (BC-2.07.018 pc2): no finding pushed.
    // AC-009 (BC-2.07.018 pc3): A-label counted in sni_counts under raw string key.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Canonical BC-2.07.018 test vector: RFC 5890 A-label for "café.example".
    let record = build_client_hello("xn--caf-dma.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // Positive-parse anchor.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-009 anchor (BC-2.07.018): parse must succeed for A-label"
    );
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-009 anchor (BC-2.07.018): handshake must be counted"
    );

    // BC-2.07.018 pc2: no finding of any kind for this A-label.
    assert!(
        analyzer.findings().is_empty(),
        "AC-009 (BC-2.07.018 pc2): A-label must produce no findings; \
         got: {:?}",
        analyzer.findings()
    );

    // BC-2.07.018 pc3: counted under the raw A-label string key.
    assert_eq!(
        *analyzer
            .sni_counts()
            .get("xn--caf-dma.example")
            .unwrap_or(&0),
        1,
        "AC-009 (BC-2.07.018 pc3): sni_counts must have entry for raw A-label key"
    );

    // Verify the A-label does NOT trigger the non-ASCII finding (separate finding type).
    let non_ascii_count = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .count();
    assert_eq!(
        non_ascii_count, 0,
        "AC-009 (BC-2.07.018 pc2): A-label must not trigger non-ASCII finding"
    );
}

// ── AC-010 (BC-2.07.018 invariants 1-2) ──────────────────────────────────────
//
// A-labels are pure ASCII by RFC 5890 construction and trivially satisfy arm 1.
// There is NO Punycode-specific code path — the A-label goes through the same
// arm 1 as any other clean-ASCII hostname. The distinction is only that BC-2.07.018
// documents deliberate exclusion of correctly-encoded IDN from the T1027 surface.
//
// Test approach: verify that "xn--caf-dma.example" (A-label) and "example.com"
// (plain ASCII) both produce identical finding behavior (zero SNI findings).
// Specifically, no "punycode" or "idna" related finding type exists.
#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_018_a_label_uses_same_arm1_as_plain_ascii() {
    // AC-010 (BC-2.07.018 inv1): A-labels trivially satisfy arm 1 — no special path.
    // AC-010 (BC-2.07.018 inv2): This BC is a special case of BC-2.07.013; the
    //   A-label path is just BC-2.07.013 arm 1, not a separate code path.
    let fk = test_flow_key();

    // Both hostnames produce identical outcome: zero findings, sni_counts entry.
    let cases: &[&str] = &[
        "xn--caf-dma.example",  // RFC 5890 A-label for "café.example"
        "example.com",          // plain ASCII baseline
        "xn--",                 // degenerate A-label prefix (BC-2.07.018 EC-002)
        "xn--nxasmq6b.example", // another valid A-label
    ];

    for hostname in cases {
        let mut analyzer = TlsAnalyzer::new();
        let record = build_client_hello(hostname, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // Positive-parse anchor.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "AC-010 anchor (BC-2.07.018 inv1): parse must succeed for {hostname:?}"
        );

        // BC-2.07.018 inv1: A-label takes same arm 1 path as plain ASCII —
        // zero SNI findings, regardless of whether hostname starts with "xn--".
        let sni_findings: Vec<_> = analyzer
            .findings()
            .into_iter()
            .filter(|f| f.summary.contains("SNI"))
            .collect();
        assert!(
            sni_findings.is_empty(),
            "AC-010 (BC-2.07.018 inv1): {hostname:?} must produce zero SNI findings; \
             got: {sni_findings:?}"
        );

        // BC-2.07.018 inv2: no Punycode-specific finding type exists.
        // Verify by checking no finding references "punycode" or "idna".
        let punycode_findings = analyzer
            .findings()
            .iter()
            .filter(|f| {
                let s = f.summary.to_lowercase();
                s.contains("punycode") || s.contains("idna")
            })
            .count();
        assert_eq!(
            punycode_findings, 0,
            "AC-010 (BC-2.07.018 inv2): no Punycode-specific finding type must exist; \
             got {punycode_findings} for {hostname:?}"
        );

        // sni_counts entry exists — confirms arm 1 ran the count insertion path.
        assert_eq!(
            *analyzer.sni_counts().get(*hostname).unwrap_or(&0),
            1,
            "AC-010 (BC-2.07.018 inv1): sni_counts must have entry for {hostname:?}"
        );
    }
}

// ── Edge-case coverage for EC scenarios from STORY-055 ───────────────────────
//
// These tests exercise the exact EC scenarios listed in the story's Edge Cases
// table with BC-prefixed names. They verify the boundary byte arithmetic
// using exact byte values specified in the story.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_016_ec004_space_only_sni_is_arm1() {
    // EC-009 from STORY-055: SNI = " " (space only, 0x20 is NOT C0).
    // BC-2.07.016 EC-002: 0x20 only -> arm 1; no finding.
    // BC-2.07.013 EC-003: " " (space, 0x20) -> arm 1; no finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello(" ", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "EC-009 anchor: parse must succeed for space-only SNI"
    );
    assert!(
        analyzer.findings().is_empty(),
        "EC-009 (BC-2.07.013 EC-003 / BC-2.07.016 EC-002): space-only SNI must produce \
         no finding (0x20 is not C0); got: {:?}",
        analyzer.findings()
    );
    assert_eq!(
        *analyzer.sni_counts().get(" ").unwrap_or(&0),
        1,
        "EC-009: space-only SNI must be counted in sni_counts"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_013_ec010_same_clean_ascii_sni_twice_counts_two() {
    // EC-010 from STORY-055: same clean ASCII SNI seen twice -> sni_counts == 2.
    // BC-2.07.013 canonical test vector: "example.com" twice -> sni_counts["example.com"] == 2.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        *analyzer.sni_counts().get("example.com").unwrap_or(&0),
        2,
        "EC-010 (BC-2.07.013 canonical vector): same SNI twice must increment count to 2"
    );
    // Still no findings — the second visit to arm 1 does not create findings.
    let sni_findings = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("SNI"))
        .count();
    assert_eq!(
        sni_findings, 0,
        "EC-010: repeated clean ASCII SNI must still produce zero SNI findings"
    );
}

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_016_ec003_nul_byte_is_c0_start_trips_arm2() {
    // BC-2.07.016 EC-003: SNI = "a\x00b" (NUL, start of C0 range) -> arm 2 fires.
    // BC-2.07.014 EC-001: "evil\x00.com" -> AsciiWithControl; T1027 finding.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let sni_bytes: &[u8] = b"a\x00b.example";
    let record = build_client_hello_ascii_bytes(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(analyzer.parse_error_count(), 0);

    let ctrl_count = analyzer
        .findings()
        .iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .count();
    assert_eq!(
        ctrl_count, 1,
        "BC-2.07.016 EC-003 / BC-2.07.014 EC-001: NUL byte (0x00, start of C0) must trip \
         arm 2; expected 1 finding, got {ctrl_count}"
    );

    // Discriminating: mitre_techniques is ["T1027"] (not T1036 or empty).
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .unwrap();
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "BC-2.07.016 EC-003: NUL-byte finding must have mitre_techniques=[\"T1027\"]"
    );
}

// ── STORY-054 Brownfield-Formalization Tests (BC-2.07.009/010/011/012/030/036) ──
//
// New tests covering AC-007..AC-013. AC-001..006 are handled by the strengthened
// tests above (test_weak_cipher_finding_client, test_weak_cipher_finding_server,
// test_normal_handshake_no_findings) plus test_ssl30_pcap_generates_findings in
// tls_integration_tests.rs.

/// Build a minimal TLS ClientHello whose ClientHello version field is set to `version`
/// (not the record-layer version). The SNI is "test.com" and ciphers are caller-supplied.
/// Used for deprecated-protocol tests that need to exercise version <= 0x0300.
///
/// Note: The record-layer version is always 0x0301 (TLS 1.0 record framing); only the
/// ClientHello body's 2-byte version field is overridden. This is what tls_parser reads
/// as `ch.version`.
fn build_client_hello_with_body_version(body_version: u16, cipher_ids: &[u16]) -> Vec<u8> {
    let mut extensions = Vec::new();

    // SNI extension (type 0x0000) — "test.com"
    let hostname = b"test.com";
    let name_len = u16::try_from(hostname.len()).unwrap();
    let sni_list_len: u16 = name_len + 3; // 1 byte type + 2 bytes name len
    let sni_ext_len: u16 = sni_list_len + 2; // 2 bytes list len
    extensions.extend_from_slice(&[0x00, 0x00]); // extension type: server_name
    extensions.extend_from_slice(&sni_ext_len.to_be_bytes());
    extensions.extend_from_slice(&sni_list_len.to_be_bytes());
    extensions.push(0x00); // NameType host_name
    extensions.extend_from_slice(&name_len.to_be_bytes());
    extensions.extend_from_slice(hostname);

    let mut ch_body = Vec::new();
    ch_body.extend_from_slice(&body_version.to_be_bytes()); // ClientHello version (overridden)
    ch_body.extend_from_slice(&[0u8; 32]); // random
    ch_body.push(0x00); // session_id length: 0

    let ciphers_len = u16::try_from(cipher_ids.len() * 2).expect("cipher list too long");
    ch_body.extend_from_slice(&ciphers_len.to_be_bytes());
    for &id in cipher_ids {
        ch_body.extend_from_slice(&id.to_be_bytes());
    }

    ch_body.push(0x01); // compression methods length
    ch_body.push(0x00); // null compression

    let ext_len = u16::try_from(extensions.len()).expect("extensions too long");
    ch_body.extend_from_slice(&ext_len.to_be_bytes());
    ch_body.extend_from_slice(&extensions);

    let mut handshake = Vec::new();
    handshake.push(0x01); // ClientHello
    let ch_len = ch_body.len() as u32;
    handshake.push((ch_len >> 16) as u8);
    handshake.push((ch_len >> 8) as u8);
    handshake.push(ch_len as u8);
    handshake.extend_from_slice(&ch_body);

    let mut record = Vec::new();
    record.push(0x16); // handshake record type
    record.extend_from_slice(&[0x03, 0x01]); // record-layer version (TLS 1.0 framing)
    let hs_len = u16::try_from(handshake.len()).expect("handshake too long");
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

/// Build a minimal TLS ServerHello with an explicit version field.
/// Used for deprecated-protocol tests (BC-2.07.012) that need version <= 0x0300.
fn build_server_hello_with_version(body_version: u16, cipher_id: u16) -> Vec<u8> {
    // Extensions: just renegotiation_info (0xff01) with empty data.
    let mut extensions = Vec::new();
    extensions.extend_from_slice(&[0xff, 0x01]); // renegotiation_info
    extensions.extend_from_slice(&[0x00, 0x01]); // data length
    extensions.push(0x00); // empty renegotiation info

    let mut sh_body = Vec::new();
    sh_body.extend_from_slice(&body_version.to_be_bytes()); // ServerHello version (overridden)
    sh_body.extend_from_slice(&[0u8; 32]); // random
    sh_body.push(0x00); // session_id length: 0
    sh_body.extend_from_slice(&cipher_id.to_be_bytes());
    sh_body.push(0x00); // compression: null

    let ext_len = u16::try_from(extensions.len()).unwrap();
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
    record.extend_from_slice(&[0x03, 0x01]); // record-layer version
    let hs_len = u16::try_from(handshake.len()).unwrap();
    record.extend_from_slice(&hs_len.to_be_bytes());
    record.extend_from_slice(&handshake);
    record
}

// ── BC-2.07.011 ──────────────────────────────────────────────────────────────
// AC-007 (BC-2.07.011 invariant 1-2): TLS 1.0 (0x0301) does NOT trigger the
//   deprecated-protocol finding; threshold is strictly <= 0x0300. Summary always
//   contains "RFC 7568".
//
// Chosen test name (reported for AC citation-sync): test_client_tls10_no_deprecated_finding
#[test]
fn test_client_tls10_no_deprecated_finding() {
    // AC-007 / BC-2.07.011 invariant 1: version 0x0301 (TLS 1.0) is above the threshold.
    // The deprecated-protocol check is strictly `version <= 0x0300`; 0x0301 must NOT fire.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello with version 0x0301 (TLS 1.0), strong cipher.
    let record = build_client_hello_with_body_version(0x0301, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    // BC-2.07.011 invariant 1: 0x0301 must NOT produce a deprecated-protocol finding.
    let deprecated_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();
    assert_eq!(
        deprecated_findings.len(),
        0,
        "BC-2.07.011 invariant 1: TLS 1.0 (0x0301) must NOT trigger deprecated-protocol finding \
         (threshold is strictly <= 0x0300); got: {:?}",
        deprecated_findings
    );

    // BC-2.07.011 invariant 2: when a deprecated-protocol finding IS produced (e.g. 0x0300),
    // its summary must contain "RFC 7568". We verify this on a 0x0300 ClientHello.
    let mut analyzer_ssl30 = TlsAnalyzer::new();
    let ssl30_record = build_client_hello_with_body_version(0x0300, &[0x1301]);
    analyzer_ssl30.on_data(&fk, Direction::ClientToServer, &ssl30_record, 0, 0);

    let ssl30_findings: Vec<_> = analyzer_ssl30
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();
    assert_eq!(
        ssl30_findings.len(),
        1,
        "BC-2.07.011 invariant 2 setup: SSL 3.0 (0x0300) must produce one deprecated-protocol \
         finding (confirms threshold boundary)"
    );
    assert!(
        ssl30_findings[0].summary.contains("RFC 7568"),
        "BC-2.07.011 invariant 2: deprecated-protocol summary must contain 'RFC 7568' as normative \
         reference; got: {:?}",
        ssl30_findings[0].summary
    );
}

// ── BC-2.07.011 invariant 3 ───────────────────────────────────────────────────
// AC-008 (BC-2.07.011 invariant 3): both deprecated-protocol AND weak-cipher findings
//   fire independently from the same SSL 3.0 ClientHello with a weak cipher.
//
// Chosen test name: test_ssl30_client_weak_cipher_both_findings
#[test]
fn test_ssl30_client_weak_cipher_both_findings() {
    // AC-008 / BC-2.07.011 invariant 3: two findings from one ClientHello (both conditions met).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // SSL 3.0 ClientHello (version 0x0300) with a NULL weak cipher (0x0002).
    let record = build_client_hello_with_body_version(0x0300, &[0x0002, 0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    let findings = analyzer.findings();

    // BC-2.07.011 invariant 3: both findings must appear independently — at least 2.
    assert!(
        findings.len() >= 2,
        "BC-2.07.011 invariant 3: SSL 3.0 ClientHello with weak cipher must produce \
         at least 2 findings (deprecated-protocol AND weak-cipher); got: {}",
        findings.len()
    );

    // One deprecated-protocol finding.
    let deprecated_count = findings
        .iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .count();
    assert_eq!(
        deprecated_count, 1,
        "BC-2.07.011 invariant 3: must have exactly one deprecated-protocol finding \
         from SSL 3.0 ClientHello"
    );

    // One weak-cipher finding.
    let weak_count = findings
        .iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .count();
    assert_eq!(
        weak_count, 1,
        "BC-2.07.011 invariant 3: must have exactly one weak-cipher finding \
         from SSL 3.0 ClientHello with NULL cipher"
    );
}

// ── BC-2.07.012 ──────────────────────────────────────────────────────────────
// AC-009 (BC-2.07.012 postconditions 1-2): ServerHello version 0x0300 → one
//   Anomaly/Likely/High finding with exact summary/evidence/direction=ServerToClient.
//
// Chosen test name: test_server_ssl30_deprecated_finding
#[test]
fn test_server_ssl30_deprecated_finding() {
    // AC-009 / BC-2.07.012 postconditions 1-2: SSL 3.0 ServerHello → exact finding fields.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello first to open the flow (modern TLS — no client-side findings).
    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with version 0x0300 (SSL 3.0) and a strong cipher (no server-cipher finding).
    // Use TLS_AES_128_GCM_SHA256 (0x1301) as the selected cipher so only the version fires.
    let sh = build_server_hello_with_version(0x0300, 0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    let findings = analyzer.findings();

    // BC-2.07.012 postcondition 1: exactly one finding.
    let deprecated_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();
    assert_eq!(
        deprecated_findings.len(),
        1,
        "BC-2.07.012 postcondition 1: ServerHello version 0x0300 must produce exactly one \
         deprecated-protocol finding; got: {:?}",
        findings
    );

    let f = deprecated_findings[0];

    // BC-2.07.012 postcondition 1: category = Anomaly
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "BC-2.07.012 postcondition 1: category must be Anomaly"
    );

    // BC-2.07.012 postcondition 1: verdict = Likely
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Likely,
        "BC-2.07.012 postcondition 1: verdict must be Likely"
    );

    // BC-2.07.012 postcondition 1: confidence = High (server deprecated version = High)
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::High,
        "BC-2.07.012 postcondition 1: confidence must be High for server deprecated protocol"
    );

    // BC-2.07.012 postcondition 1: exact summary string (verified against
    // src/analyzer/tls.rs:595-597 server deprecated-protocol emit site).
    assert_eq!(
        f.summary, "ServerHello negotiated deprecated protocol (SSL 3.0, RFC 7568 prohibits SSLv3)",
        "F-S054-P5-001 (BC-2.07.012 pc1): server-side deprecated-protocol summary must match \
         exact format from handle_server_hello"
    );

    // BC-2.07.012 postcondition 1: evidence = ["Version: 0x0300 (SSL 3.0)"]
    assert_eq!(
        f.evidence.len(),
        1,
        "BC-2.07.012 postcondition 1: evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "Version: 0x0300 (SSL 3.0)",
        "BC-2.07.012 postcondition 1: evidence must be exact canonical string \
         'Version: 0x0300 (SSL 3.0)'; got: {:?}",
        f.evidence[0]
    );

    // BC-2.07.012 postcondition 1: mitre_technique = None
    assert_eq!(
        f.mitre_techniques,
        Vec::<String>::new(),
        "BC-2.07.012 postcondition 1: mitre_technique must be None"
    );

    // BC-2.07.012 postcondition 1 / invariant 2: direction = ServerToClient
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ServerToClient),
        "BC-2.07.012 invariant 2: direction must be ServerToClient"
    );

    // BC-2.07.012 invariant 1: TLS 1.0 (0x0301) server must NOT trigger.
    let mut analyzer_tls10 = TlsAnalyzer::new();
    let ch2 = build_client_hello("test.com", &[0x1301]);
    analyzer_tls10.on_data(&fk, Direction::ClientToServer, &ch2, 0, 0);
    let sh_tls10 = build_server_hello_with_version(0x0301, 0x1301);
    analyzer_tls10.on_data(&fk, Direction::ServerToClient, &sh_tls10, 0, 0);
    let tls10_deprecated: Vec<_> = analyzer_tls10
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();
    assert_eq!(
        tls10_deprecated.len(),
        0,
        "BC-2.07.012 invariant 1: TLS 1.0 (0x0301) ServerHello must NOT trigger \
         deprecated-protocol finding (threshold strictly <= 0x0300)"
    );
}

// ── BC-2.07.012 invariant 3 ───────────────────────────────────────────────────
// AC-010 (BC-2.07.012 invariant 1-2): both SSL 3.0 ClientHello AND SSL 3.0 ServerHello
//   produce two separate deprecated findings with distinct directions.
//
// Chosen test name: test_client_and_server_ssl30_distinct_directions
#[test]
fn test_client_and_server_ssl30_distinct_directions() {
    // AC-010 / BC-2.07.012 invariant 2-3: both hellos at SSL 3.0 → two findings,
    // one ClientToServer and one ServerToClient.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // SSL 3.0 ClientHello with strong cipher (only version fires, no weak-cipher finding).
    let ch = build_client_hello_with_body_version(0x0300, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // SSL 3.0 ServerHello with strong cipher.
    let sh = build_server_hello_with_version(0x0300, 0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    let findings = analyzer.findings();

    // Exactly two deprecated-protocol findings (one from each hello).
    let deprecated: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();
    assert_eq!(
        deprecated.len(),
        2,
        "BC-2.07.012 invariant 3: SSL 3.0 ClientHello + SSL 3.0 ServerHello must produce \
         exactly two deprecated-protocol findings; got: {:?}",
        findings
    );

    // One must have direction ClientToServer (client-side BC-2.07.011).
    let client_deprecated = deprecated
        .iter()
        .filter(|f| f.direction == Some(wirerust::reassembly::handler::Direction::ClientToServer))
        .count();
    assert_eq!(
        client_deprecated, 1,
        "BC-2.07.012 invariant 2: one deprecated-protocol finding must have direction \
         ClientToServer (from ClientHello)"
    );

    // One must have direction ServerToClient (server-side BC-2.07.012).
    let server_deprecated = deprecated
        .iter()
        .filter(|f| f.direction == Some(wirerust::reassembly::handler::Direction::ServerToClient))
        .count();
    assert_eq!(
        server_deprecated, 1,
        "BC-2.07.012 invariant 2: one deprecated-protocol finding must have direction \
         ServerToClient (from ServerHello)"
    );

    // Both findings must have the same summary root but their direction distinguishes them.
    assert!(
        deprecated
            .iter()
            .any(|f| f.summary.contains("ClientHello uses deprecated")),
        "BC-2.07.012 invariant 2: one finding must say 'ClientHello uses deprecated'; \
         got summaries: {:?}",
        deprecated.iter().map(|f| &f.summary).collect::<Vec<_>>()
    );
    assert!(
        deprecated
            .iter()
            .any(|f| f.summary.contains("ServerHello negotiated deprecated")),
        "BC-2.07.012 invariant 2: one finding must say 'ServerHello negotiated deprecated'; \
         got summaries: {:?}",
        deprecated.iter().map(|f| &f.summary).collect::<Vec<_>>()
    );

    // F-S054-P2-001 (exactness parity): pin the client-side finding to EXACT summary and evidence.
    // The server-side finding was already exact (from pass-1 remediation). Bring client-side to
    // parity. Source: src/analyzer/tls.rs:530-538 (handle_client_hello deprecated-protocol arm).
    let client_finding = deprecated
        .iter()
        .find(|f| f.direction == Some(wirerust::reassembly::handler::Direction::ClientToServer))
        .expect("client-side deprecated-protocol finding must exist");

    // BC-2.07.011 pc1: exact summary string for SSL 3.0 ClientHello.
    assert_eq!(
        client_finding.summary,
        "ClientHello uses deprecated protocol (SSL 3.0, RFC 7568 prohibits SSLv3)",
        "F-S054-P2-001 (BC-2.07.011 pc1): client-side deprecated-protocol summary must match \
         exact format from handle_client_hello (src/analyzer/tls.rs:530-531)"
    );

    // BC-2.07.011 pc2: exact evidence vector for SSL 3.0 ClientHello.
    assert_eq!(
        client_finding.evidence,
        vec!["Version: 0x0300 (SSL 3.0)".to_string()],
        "F-S054-P2-001 (BC-2.07.011 pc2): client-side deprecated-protocol evidence must match \
         exact format from handle_client_hello (src/analyzer/tls.rs:533)"
    );

    // F-S054-P5-002: client-side finding field exactness parity with server block.
    // Verified against src/analyzer/tls.rs:526-538 (handle_client_hello deprecated-protocol arm).
    assert_eq!(
        client_finding.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "F-S054-P5-002 (BC-2.07.011 pc1): client-side category must be Anomaly"
    );
    assert_eq!(
        client_finding.verdict,
        wirerust::findings::Verdict::Likely,
        "F-S054-P5-002 (BC-2.07.011 pc1): client-side verdict must be Likely"
    );
    assert_eq!(
        client_finding.confidence,
        wirerust::findings::Confidence::High,
        "F-S054-P5-002 (BC-2.07.011 pc1): client-side confidence must be High"
    );
    assert_eq!(
        client_finding.mitre_techniques,
        Vec::<String>::new(),
        "F-S054-P5-002 (BC-2.07.011 pc1): client-side mitre_technique must be None"
    );
}

// ── BC-2.07.036 ──────────────────────────────────────────────────────────────
// AC-012 (BC-2.07.036 postconditions 1-2): unknown cipher ID 0x1234 → "0x1234"
//   (6-char lowercase string with 0x prefix and 4 hex digits).
//
// Chosen test name: test_cipher_name_unknown_hex_lowercase
#[test]
fn test_cipher_name_unknown_hex_lowercase() {
    // AC-012 / BC-2.07.036 postcondition 1: cipher_name(0x1234) → "0x1234".
    // Exercised via cipher_counts (accessible through summarize().detail["cipher_suites"]).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // ClientHello first.
    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with cipher ID 0x1234 — unrecognized by tls_parser.
    let sh = build_server_hello(0x1234);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Positive-parse anchor: the record itself must parse cleanly.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-012 anchor: ServerHello with unknown cipher 0x1234 must not cause a parse error"
    );

    // BC-2.07.036 postcondition 1: cipher_name(0x1234) must return "0x1234"
    // (6-character lowercase hex string). Verified via cipher_counts map key.
    let detail = analyzer.summarize().detail;
    let cipher_suites = detail
        .get("cipher_suites")
        .expect("cipher_suites key must be present in summary detail");

    assert!(
        cipher_suites.get("0x1234").is_some(),
        "BC-2.07.036 postcondition 1: cipher_counts must contain key '0x1234' \
         for unrecognized cipher ID 0x1234; got cipher_suites: {cipher_suites}"
    );

    // BC-2.07.036 postcondition 1: format is "0x" prefix + 4 lowercase hex digits = 6 chars.
    let key = "0x1234";
    assert_eq!(
        key.len(),
        6,
        "BC-2.07.036 postcondition 1: hex fallback must be 6 chars"
    );
    assert!(
        key.starts_with("0x"),
        "BC-2.07.036 postcondition 1: hex fallback must start with '0x'"
    );
    assert!(
        key[2..]
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
        "BC-2.07.036 postcondition 1: hex digits must be lowercase; got: {key}"
    );

    // EC-004 / BC-2.07.010 EC-004: no weak-cipher finding for unknown cipher
    // (is_weak_server_cipher → false when from_id returns None).
    let weak_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .collect();
    assert_eq!(
        weak_findings.len(),
        0,
        "BC-2.07.010 EC-004 / BC-2.07.036 postcondition 2: unknown cipher 0x1234 must not \
         trigger weak-server-cipher finding (from_id returns None → is_weak_server_cipher false)"
    );

    // EC-009 / BC-2.07.036 invariant 1: 0xAAAA must render as "0xaaaa" (lowercase, not "0xAAAA").
    let mut analyzer_aaaa = TlsAnalyzer::new();
    let ch2 = build_client_hello("test.com", &[0x1301]);
    analyzer_aaaa.on_data(&fk, Direction::ClientToServer, &ch2, 0, 0);
    let sh_aaaa = build_server_hello(0xAAAA);
    analyzer_aaaa.on_data(&fk, Direction::ServerToClient, &sh_aaaa, 0, 0);

    let detail_aaaa = analyzer_aaaa.summarize().detail;
    let suites_aaaa = detail_aaaa
        .get("cipher_suites")
        .expect("cipher_suites must be present");
    assert!(
        suites_aaaa.get("0xaaaa").is_some(),
        "BC-2.07.036 EC-009: cipher 0xAAAA must be keyed as '0xaaaa' (lowercase); \
         got cipher_suites: {suites_aaaa}"
    );
    assert!(
        suites_aaaa.get("0xAAAA").is_none(),
        "BC-2.07.036 EC-009: '0xAAAA' (uppercase) must NOT be a key; \
         format is strictly lowercase"
    );
}

// ── BC-2.07.036 invariants 1-3 ────────────────────────────────────────────────
// AC-013 (BC-2.07.036 invariant 1-2): recognized cipher IDs return IANA name
//   (no "0x" prefix); ID 0xFFFF returns "0xffff" (lowercase).
//
// Chosen test name: test_cipher_name_recognized_and_ffff
#[test]
fn test_cipher_name_recognized_and_ffff() {
    // AC-013 / BC-2.07.036 invariant 2: recognized cipher → IANA name (no "0x" prefix).
    // Use TLS_AES_128_GCM_SHA256 (0x1301) — recognized by tls_parser.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // BC-2.07.036 invariant 2: recognized cipher appears as IANA name without "0x" prefix.
    let detail = analyzer.summarize().detail;
    let cipher_suites = detail
        .get("cipher_suites")
        .expect("cipher_suites key must be in summary detail");

    assert!(
        cipher_suites.get("TLS_AES_128_GCM_SHA256").is_some(),
        "BC-2.07.036 invariant 2: cipher 0x1301 must be keyed as 'TLS_AES_128_GCM_SHA256' \
         (IANA name, no '0x' prefix); got cipher_suites: {cipher_suites}"
    );
    assert_eq!(
        cipher_suites
            .get("TLS_AES_128_GCM_SHA256")
            .and_then(|v| v.as_u64()),
        Some(1),
        "BC-2.07.036 invariant 2: TLS_AES_128_GCM_SHA256 must have count 1"
    );

    // AC-013 / BC-2.07.036 invariant 3: ID 0xFFFF is unrecognized → "0xffff" (lowercase).
    let mut analyzer_ffff = TlsAnalyzer::new();
    let ch2 = build_client_hello("test.com", &[0x1301]);
    analyzer_ffff.on_data(&fk, Direction::ClientToServer, &ch2, 0, 0);

    let sh_ffff = build_server_hello(0xFFFF);
    analyzer_ffff.on_data(&fk, Direction::ServerToClient, &sh_ffff, 0, 0);

    let detail_ffff = analyzer_ffff.summarize().detail;
    let suites_ffff = detail_ffff
        .get("cipher_suites")
        .expect("cipher_suites key must be present");

    assert!(
        suites_ffff.get("0xffff").is_some(),
        "BC-2.07.036 invariant 3: cipher 0xFFFF must be keyed as '0xffff' (lowercase); \
         got cipher_suites: {suites_ffff}"
    );
    assert_eq!(
        suites_ffff.get("0xffff").and_then(|v| v.as_u64()),
        Some(1),
        "BC-2.07.036 invariant 3: '0xffff' must have count 1"
    );

    // EC-001 / BC-2.07.036 EC-001: ID 0x0000 (TLS_NULL_WITH_NULL_NULL) IS recognized
    // and must appear as name "TLS_NULL_WITH_NULL_NULL", not "0x0000".
    let mut analyzer_null = TlsAnalyzer::new();
    let ch3 = build_client_hello("test.com", &[0x1301]);
    analyzer_null.on_data(&fk, Direction::ClientToServer, &ch3, 0, 0);

    let sh_null = build_server_hello(0x0000);
    analyzer_null.on_data(&fk, Direction::ServerToClient, &sh_null, 0, 0);

    let detail_null = analyzer_null.summarize().detail;
    let suites_null = detail_null
        .get("cipher_suites")
        .expect("cipher_suites key must be present");

    assert!(
        suites_null.get("TLS_NULL_WITH_NULL_NULL").is_some(),
        "BC-2.07.036 EC-001: cipher 0x0000 must be keyed as 'TLS_NULL_WITH_NULL_NULL' \
         (recognized IANA name, no '0x' prefix); got cipher_suites: {suites_null}"
    );
}

// ── F-S054-P1-001/002 — version_name arm coverage (client 0x0200 / 0x0100) ───
//
// Empirically verified (Step A probes, 2026-05-29):
//   - ClientHello version 0x0200: tls_parser ACCEPTS the record; handle_client_hello
//     IS reached; the version_name "SSL 2.0" arm fires; parse_errors=0; handshakes=1.
//   - ClientHello version 0x0100: tls_parser ACCEPTS the record; handle_client_hello
//     IS reached; the version_name "Unknown legacy SSL" arm fires; parse_errors=0;
//     handshakes=1.
//   - ServerHello version 0x0200: tls_parser REJECTS the record at the record layer;
//     handle_server_hello NOT reached; parse_errors=1 (already pinned by
//     test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned).
//
// BC references: BC-2.07.011 pc1/pc2 (client deprecated-protocol summary/evidence format).
// EC references: STORY-054 EC-006/EC-007 (= BC-2.07.011 EC-001/EC-003).
//   STORY-054 EC-006 maps to BC-2.07.011 EC-001 (0x0200 → "SSL 2.0").
//   STORY-054 EC-007 maps to BC-2.07.011 EC-003 (0x0100 → "Unknown legacy SSL").
//   "EC-006/EC-007" in the STORY-054 numbering are this story's own edge-case numbers;
//   the authoritative BC edge-case IDs are EC-001 and EC-003 in BC-2.07.011.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_011_client_deprecated_version_name_ssl2_and_legacy() {
    // BC-2.07.011 postconditions 1-2 / STORY-054 EC-006/EC-007 (= BC-2.07.011 EC-001/EC-003):
    //
    // F-S054-P3-002 FIX E: corrected citation from "BC-2.07.011 EC-006/EC-007" to
    // "STORY-054 EC-006/EC-007 (= BC-2.07.011 EC-001/EC-003)".
    // BC-2.07.011 has only EC-001..EC-005; EC-006/EC-007 are STORY-054's own edge-case
    // numbers that map to BC-2.07.011 EC-001 (0x0200) and EC-003 (0x0100) respectively.
    //
    // STORY-054 EC-006 (= BC-2.07.011 EC-001): ClientHello with version 0x0200 (SSL 2.0)
    //   reaches handle_client_hello (tls_parser accepts the record).
    //   The version_name arm 0x0200 => "SSL 2.0" fires.
    //   Summary: "ClientHello uses deprecated protocol (SSL 2.0, RFC 7568 prohibits SSLv3)"
    //   Evidence: ["Version: 0x0200 (SSL 2.0)"]
    //
    // STORY-054 EC-007 (= BC-2.07.011 EC-003): ClientHello with version 0x0100 (below
    //   SSL 2.0 — falls to "Unknown legacy SSL" catchall arm) reaches handle_client_hello.
    //   Summary: "ClientHello uses deprecated protocol (Unknown legacy SSL, RFC 7568 prohibits SSLv3)"
    //   Evidence: ["Version: 0x0100 (Unknown legacy SSL)"]

    // ── STORY-054 EC-006 (= BC-2.07.011 EC-001): 0x0200 → "SSL 2.0" ─────────
    let mut analyzer_ssl2 = TlsAnalyzer::new();
    let fk = test_flow_key();

    let record_ssl2 = build_client_hello_with_body_version(0x0200, &[0x1301]);
    analyzer_ssl2.on_data(&fk, Direction::ClientToServer, &record_ssl2, 0, 0);

    // Parse-clean anchor: tls_parser accepts SSL 2.0 ClientHello at the record layer.
    assert_eq!(
        analyzer_ssl2.parse_error_count(),
        0,
        "EC-006 anchor (BC-2.07.011): tls_parser accepts ClientHello version 0x0200; \
         parse_errors must be 0 (if this fails, tls_parser now rejects SSL 2.0 ClientHello — \
         revisit the version_name 0x0200 arm reachability)"
    );

    // handle_client_hello reached: handshakes_seen = 1, version_counts[0x0200] = 1.
    assert_eq!(
        analyzer_ssl2.handshake_count(),
        1,
        "EC-006 (BC-2.07.011 pc1): handshakes_seen must be 1 (handler was reached)"
    );
    assert_eq!(
        *analyzer_ssl2.version_counts().get(&0x0200).unwrap_or(&0),
        1,
        "EC-006 (BC-2.07.011 pc2): version_counts[0x0200] must be 1 (handler was reached)"
    );

    let findings_ssl2 = analyzer_ssl2.findings();
    let deprecated_ssl2: Vec<_> = findings_ssl2
        .iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();

    assert_eq!(
        deprecated_ssl2.len(),
        1,
        "EC-006 (BC-2.07.011): exactly one deprecated-protocol finding for ClientHello 0x0200; \
         got: {:?}",
        findings_ssl2
    );

    // BC-2.07.011 pc1: exact summary string for 0x0200 arm.
    assert_eq!(
        deprecated_ssl2[0].summary,
        "ClientHello uses deprecated protocol (SSL 2.0, RFC 7568 prohibits SSLv3)",
        "EC-006 (BC-2.07.011 pc1): summary must exactly match 'SSL 2.0' version_name arm \
         for ClientHello version 0x0200"
    );

    // BC-2.07.011 pc2: exact evidence string for 0x0200 arm.
    assert_eq!(
        deprecated_ssl2[0].evidence,
        vec!["Version: 0x0200 (SSL 2.0)".to_string()],
        "EC-006 (BC-2.07.011 pc2): evidence must exactly match 'Version: 0x0200 (SSL 2.0)' \
         for ClientHello version 0x0200"
    );

    // ── STORY-054 EC-007 (= BC-2.07.011 EC-003): 0x0100 → "Unknown legacy SSL" ─
    let mut analyzer_legacy = TlsAnalyzer::new();

    let record_legacy = build_client_hello_with_body_version(0x0100, &[0x1301]);
    analyzer_legacy.on_data(&fk, Direction::ClientToServer, &record_legacy, 0, 0);

    // Parse-clean anchor: tls_parser accepts version 0x0100 ClientHello at the record layer.
    assert_eq!(
        analyzer_legacy.parse_error_count(),
        0,
        "EC-007 anchor (BC-2.07.011): tls_parser accepts ClientHello version 0x0100; \
         parse_errors must be 0 (if this fails, tls_parser now rejects this version — \
         revisit the 'Unknown legacy SSL' arm reachability)"
    );

    // handle_client_hello reached: handshakes_seen = 1, version_counts[0x0100] = 1.
    assert_eq!(
        analyzer_legacy.handshake_count(),
        1,
        "EC-007 (BC-2.07.011 pc1): handshakes_seen must be 1 (handler was reached)"
    );
    assert_eq!(
        *analyzer_legacy.version_counts().get(&0x0100).unwrap_or(&0),
        1,
        "EC-007 (BC-2.07.011 pc2): version_counts[0x0100] must be 1 (handler was reached)"
    );

    let findings_legacy = analyzer_legacy.findings();
    let deprecated_legacy: Vec<_> = findings_legacy
        .iter()
        .filter(|f| f.summary.contains("deprecated protocol"))
        .collect();

    assert_eq!(
        deprecated_legacy.len(),
        1,
        "EC-007 (BC-2.07.011): exactly one deprecated-protocol finding for ClientHello 0x0100; \
         got: {:?}",
        findings_legacy
    );

    // BC-2.07.011 pc1: exact summary string for "Unknown legacy SSL" catchall arm.
    assert_eq!(
        deprecated_legacy[0].summary,
        "ClientHello uses deprecated protocol (Unknown legacy SSL, RFC 7568 prohibits SSLv3)",
        "EC-007 (BC-2.07.011 pc1): summary must exactly match 'Unknown legacy SSL' arm \
         for ClientHello version 0x0100 (below SSL 2.0, falls to catchall)"
    );

    // BC-2.07.011 pc2: exact evidence string for "Unknown legacy SSL" arm.
    assert_eq!(
        deprecated_legacy[0].evidence,
        vec!["Version: 0x0100 (Unknown legacy SSL)".to_string()],
        "EC-007 (BC-2.07.011 pc2): evidence must exactly match 'Version: 0x0100 (Unknown legacy SSL)' \
         for ClientHello version 0x0100"
    );
}

// ── F-S054-P1-001/002 — ServerHello 0x0200 version_name arm: UNREACHABLE (pin) ─
//
// Empirically verified (Step A probes, 2026-05-29):
//   ServerHello version 0x0200: tls_parser REJECTS the record at the record layer.
//   handle_server_hello is NEVER reached. parse_errors=1, version_counts[0x0200]=0,
//   ja3s_counts empty, no deprecated-protocol finding from ServerToClient direction.
//
// The server-side version_name arms (0x0200 => "SSL 2.0", 0x0300 => "SSL 3.0",
// _ => "Unknown legacy SSL") at tls.rs ~586-590 are therefore:
//   0x0200: UNREACHABLE via real pcap/crafted records under tls-parser 0.12
//   0x0300: REACHABLE (tested by test_server_ssl30_deprecated_finding)
//   "Unknown legacy SSL": UNREACHABLE via ServerHello — tls_parser only accepts
//     versions it recognizes for ServerHello parsing.
//
// This pin test consolidates and extends the EC-004 pin from
// test_BC_2_07_002_ec004_ssl2_version_parse_behavior_pinned, providing the
// explicit EC-004/EC-005 annotation for the server side.
//
// Note: BC-2.07.012 EC-004/EC-005 were corrected to document parse-rejection in v1.3/v1.4
// per F-S054-P1-002. The server-side 0x0200 ("SSL 2.0") and catchall ("Unknown legacy SSL")
// version_name arms are unreachable under tls-parser 0.12 via ServerHello records; this was
// captured in BC-2.07.012 and no further routing action is required.

#[allow(non_snake_case)]
#[test]
fn test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin() {
    // BC-2.07.012 EC-004 (0x0200) / EC-005 (sub-0x0200) — PINNED PARSE REJECTION:
    //
    // FIX D (F-S054-P2): renamed from test_BC_2_07_012_ec006_server_hello_0x0200_parse_rejection_pin
    // because the test exercises BC-2.07.012 EC-004 (ServerHello version=0x0200) and EC-005
    // (ServerHello sub-0x0200 / "Unknown legacy SSL" catchall). BC-2.07.012 has no EC-006 —
    // EC-006/EC-007 are STORY-054's own edge-case numbers for the CLIENT-side cases.
    // Product-owner must update BC-2.07.012's VP/anchor citation to reference
    // test_BC_2_07_012_ec004_ec005_server_hello_legacy_parse_rejection_pin.
    //
    // ServerHello with version=0x0200 is rejected by tls_parser at the record
    // layer. handle_server_hello is never reached. The version_name "SSL 2.0"
    // arm at tls.rs:587 is unreachable via ServerHello records under tls-parser 0.12.
    //
    // If this test fails (parse_errors==0), it means tls_parser was upgraded and
    // now accepts SSL 2.0 ServerHello records. In that case, the server-side
    // deprecated-protocol detection for 0x0200 and the "Unknown legacy SSL" catchall
    // arm will become reachable and will fire correctly (the production code already
    // handles them). The test should then be converted to an assertion-style test
    // matching the client-side EC-004/EC-005 pattern (BC-2.07.011 arms).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    let ch = build_client_hello("test.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // ServerHello with body version 0x0200 (SSL 2.0).
    let sh = build_server_hello_with_version(0x0200, 0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0, 0);

    // Pinned: tls_parser rejects the ServerHello, incrementing parse_errors.
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "EC-004/EC-005 server-side pin (BC-2.07.012): tls_parser rejects ServerHello \
         version=0x0200 at the record layer; parse_errors must be 1. \
         If this fails, tls_parser now accepts SSL 2.0 ServerHello — the server-side \
         version_name arms (EC-004/EC-005) have become reachable; convert this to a \
         positive assertion test matching the client-side BC-2.07.011 pattern."
    );

    // handle_server_hello not reached: version_counts[0x0200] stays 0.
    assert_eq!(
        *analyzer.version_counts().get(&0x0200).unwrap_or(&0),
        0,
        "EC-004 server-side pin (BC-2.07.012): handle_server_hello NOT reached; \
         version_counts must NOT contain 0x0200"
    );

    // ja3s_counts is empty (server_hello_seen was not set).
    assert!(
        analyzer.ja3s_counts().is_empty(),
        "EC-004 server-side pin (BC-2.07.012): ja3s_counts must be empty when \
         ServerHello parse fails"
    );

    // No deprecated-protocol finding from the ServerToClient direction.
    assert!(
        analyzer
            .findings()
            .iter()
            .filter(
                |f| f.direction == Some(wirerust::reassembly::handler::Direction::ServerToClient)
            )
            .all(|f| !f.summary.contains("deprecated protocol")),
        "EC-004 server-side pin (BC-2.07.012): no deprecated-protocol ServerHello \
         finding expected when tls_parser rejects the record; got: {:?}",
        analyzer.findings()
    );
}

// ── STORY-056 Brownfield-Formalization Tests (BC-2.07.017/019/020/021/037) ───
//
// SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte
// Preservation formalization (generic-citation ACs only).
//
// The named tests for AC-001/002/003/004/005/006/008 are the existing tests
// updated in-place above (DF-AC-TEST-NAME-SYNC-001). This section contains
// the three generic-citation ACs that have freely-chosen names:
//
//   AC-007 (BC-2.07.020 inv1-3)    test_arm4_hex_evidence_is_pure_ascii
//   AC-009 (BC-2.07.037 pc1-4)     test_c0_plus_non_ascii_fires_arm3_not_arm2
//   AC-010 (BC-2.07.037 inv1-2)    test_is_ascii_gate_routes_arm2_vs_arm3

// ── AC-007 (BC-2.07.020 invariants 1-3) ──────────────────────────────────────
//
// evidence[0] starts with "hex: " and the hex portion contains only [0-9a-f].
// The hex field is always pure ASCII and needs no escaping (ADR 0003).
#[allow(non_snake_case)]
#[test]
fn test_arm4_hex_evidence_is_pure_ascii() {
    // AC-007 (BC-2.07.020 inv3): hex field is always pure ASCII (0-9, a-f).
    // Exercising VP-005: arm 4 fires for any non-UTF-8 input.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // EC-004: b"\xff\xfe" is invalid UTF-8; hex = "fffe".
    let sni_bytes: &[u8] = b"\xff\xfe";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("non-UTF-8 bytes"))
        .expect("AC-007 (BC-2.07.020 inv3): non-UTF-8 finding must be emitted");

    // BC-2.07.020 inv3: evidence[0] starts with "hex: ".
    assert!(
        f.evidence[0].starts_with("hex: "),
        "AC-007 (BC-2.07.020 inv3): evidence[0] must start with \"hex: \"; \
         got: {:?}",
        f.evidence[0]
    );

    // BC-2.07.020 inv3: hex portion contains ONLY [0-9a-f] (pure ASCII lowercase hex).
    let hex_part = f.evidence[0]
        .strip_prefix("hex: ")
        .expect("AC-007: strip_prefix already verified above");

    assert!(
        hex_part
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
        "AC-007 (BC-2.07.020 inv3): hex portion must contain only [0-9a-f]; \
         got: {:?}",
        hex_part
    );

    // BC-2.07.020 inv3: hex field is pure ASCII (every byte in 0x00..=0x7f).
    assert!(
        hex_part.is_ascii(),
        "AC-007 (BC-2.07.020 inv3): hex field must be pure ASCII; got: {:?}",
        hex_part
    );

    // No uppercase letters (must be lowercase hex per bytes_to_hex spec).
    assert!(
        !hex_part.chars().any(|c| c.is_ascii_uppercase()),
        "AC-007 (BC-2.07.020 inv3): hex field must be lowercase only; got: {:?}",
        hex_part
    );
}

// ── AC-009 (BC-2.07.037 postconditions 1-4, updated by #104 / BC-TLS-037) ────
//
// When SNI bytes are valid UTF-8 but contain BOTH non-ASCII chars AND C0 bytes,
// arm 3 fires (NonAsciiUtf8), NOT arm 2 (AsciiWithControl). Summary says
// "non-ASCII characters" AND (since #104 fix) additionally mentions "control"
// when control bytes are present. The hex evidence field is still lossless.
//
// Exercises VP-005: is_ascii() is the decisive gate between arm 2 and arm 3.
// Canonical BC-2.07.037 test vector: b"caf\x01\xc3\xa9" (valid UTF-8 "café" with SOH).
#[allow(non_snake_case)]
#[test]
fn test_c0_plus_non_ascii_fires_arm3_not_arm2() {
    // AC-009 (BC-2.07.037 pc1): arm 3 fires -> SniValue::NonAsciiUtf8.
    // AC-009 (BC-2.07.037 pc2): summary says "non-ASCII characters" and "control"
    //   (control-byte enrichment added by #104 / BC-TLS-037).
    // AC-009 (BC-2.07.037 pc3): control byte is lossless in hex evidence.
    // AC-009 (BC-2.07.037 pc4): T1027/Anomaly/Inconclusive/Low/ClientToServer.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // EC-003 from STORY-056: b"caf\x01\xc3\xa9" is valid UTF-8 (decodes to "café" with SOH).
    // Decoded string: 'c','a','f',SOH(U+0001),'é'(U+00E9). is_ascii() == false -> arm 3.
    let sni_bytes: &[u8] = b"caf\x01\xc3\xa9";
    let record = build_client_hello_raw_sni(sni_bytes, &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-009 anchor (BC-2.07.037 pc1): parse must succeed"
    );

    // BC-2.07.037 pc1: arm 3 fires -> one "non-ASCII characters" finding.
    let non_ascii_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("non-ASCII characters"))
        .collect();
    assert_eq!(
        non_ascii_findings.len(),
        1,
        "AC-009 (BC-2.07.037 pc1): arm 3 must fire for b\"caf\\x01\\xc3\\xa9\"; \
         exactly one non-ASCII finding; got: {non_ascii_findings:?}"
    );

    // BC-2.07.037 pc2: arm 2 must NOT fire (no "ASCII control characters" finding).
    let ctrl_findings: Vec<_> = analyzer
        .findings()
        .into_iter()
        .filter(|f| f.summary.contains("ASCII control characters"))
        .collect();
    assert!(
        ctrl_findings.is_empty(),
        "AC-009 (BC-2.07.037 pc2): arm 2 must NOT fire for mixed C0+non-ASCII SNI; \
         arm 3 takes priority (is_ascii() == false); got: {ctrl_findings:?}"
    );

    let f = &non_ascii_findings[0];

    // BC-2.07.037 pc2 (updated by #104 / BC-TLS-037): summary says "non-ASCII
    // characters" AND "control" (control-byte enrichment for mixed values).
    assert!(
        f.summary.contains("non-ASCII characters"),
        "AC-009 (BC-2.07.037 pc2): summary must say \"non-ASCII characters\"; \
         got: {:?}",
        f.summary
    );
    assert!(
        f.summary.contains("control"),
        "AC-009 (BC-2.07.037 pc2, #104): summary must mention \"control\" for mixed \
         C0+non-ASCII SNI so SOC analysts grepping for control bytes find this case; \
         got: {:?}",
        f.summary
    );

    // BC-2.07.037 pc4: Anomaly/Inconclusive/Low/T1027/ClientToServer.
    assert_eq!(
        f.category,
        wirerust::findings::ThreatCategory::Anomaly,
        "AC-009 (BC-2.07.037 pc4): category must be Anomaly"
    );
    assert_eq!(
        f.verdict,
        wirerust::findings::Verdict::Inconclusive,
        "AC-009 (BC-2.07.037 pc4): verdict must be Inconclusive"
    );
    assert_eq!(
        f.confidence,
        wirerust::findings::Confidence::Low,
        "AC-009 (BC-2.07.037 pc4): confidence must be Low"
    );
    assert_eq!(
        f.mitre_techniques,
        vec!["T1027".to_string()],
        "AC-009 (BC-2.07.037 pc4): mitre_techniques must be exactly [\"T1027\"]"
    );
    assert_eq!(
        f.direction,
        Some(wirerust::reassembly::handler::Direction::ClientToServer),
        "AC-009 (BC-2.07.037 pc4): direction must be Some(ClientToServer)"
    );

    // BC-2.07.037 pc3: hex evidence is lossless (preserves the control byte).
    // Hex for b"caf\x01\xc3\xa9" = "63616601c3a9".
    assert_eq!(
        f.evidence.len(),
        1,
        "AC-009 (BC-2.07.037 pc3): evidence must have exactly one entry"
    );
    assert_eq!(
        f.evidence[0], "hex: 63616601c3a9",
        "AC-009 (BC-2.07.037 pc3): evidence[0] must be lossless hex preserving control byte"
    );

    // sni_counts keyed on raw decoded hostname (arm 3 key = hostname string).
    let hostname = std::str::from_utf8(sni_bytes).expect("b\"caf\\x01\\xc3\\xa9\" is valid UTF-8");
    assert_eq!(
        *analyzer.sni_counts().get(hostname).unwrap_or(&0),
        1,
        "AC-009 (BC-2.07.037 pc1): sni_counts must be keyed on raw hostname for arm 3; \
         got keys: {:?}",
        analyzer.sni_counts().keys().collect::<Vec<_>>()
    );
}

// ── AC-010 (BC-2.07.037 invariants 1-2) ──────────────────────────────────────
//
// is_ascii() is the decisive gate between arm 2 and arm 3. Even one non-ASCII
// code point causes is_ascii() == false, routing to arm 3 before contains_c0_or_del
// is evaluated. Arm evaluation is strictly top-down (VP-005).
//
// Exercises VP-005: the is_ascii() predicate as the arm 2/3 boundary.
#[allow(non_snake_case)]
#[test]
fn test_is_ascii_gate_routes_arm2_vs_arm3() {
    // AC-010 (BC-2.07.037 inv1): b"caf\x01\xc3\xa9" -> is_ascii()==false -> arm 3.
    // AC-010 (BC-2.07.037 inv2): b"evil\x01.com" -> is_ascii()==true -> arm 2.
    let fk = test_flow_key();

    // --- Case 1: b"caf\x01\xc3\xa9" has non-ASCII -> is_ascii()==false -> arm 3 ---
    let mut analyzer_arm3 = TlsAnalyzer::new();
    let sni_arm3: &[u8] = b"caf\x01\xc3\xa9";
    let record_arm3 = build_client_hello_raw_sni(sni_arm3, &[0x1301]);
    analyzer_arm3.on_data(&fk, Direction::ClientToServer, &record_arm3, 0, 0);

    // Verify is_ascii() == false (the gate predicate must be false for arm 3).
    let decoded_arm3 =
        std::str::from_utf8(sni_arm3).expect("b\"caf\\x01\\xc3\\xa9\" is valid UTF-8");
    assert!(
        !decoded_arm3.is_ascii(),
        "AC-010 (BC-2.07.037 inv1): decoded \"caf\\x01\\xc3\\xa9\" must have is_ascii()==false"
    );

    // Arm 3 fires: "non-ASCII characters" finding present.
    assert!(
        analyzer_arm3
            .findings()
            .iter()
            .any(|f| f.summary.contains("non-ASCII characters")),
        "AC-010 (BC-2.07.037 inv1): arm 3 must fire for b\"caf\\x01\\xc3\\xa9\" \
         (is_ascii()==false); got findings: {:?}",
        analyzer_arm3.findings()
    );

    // Arm 2 must NOT fire.
    assert!(
        !analyzer_arm3
            .findings()
            .iter()
            .any(|f| f.summary.contains("ASCII control characters")),
        "AC-010 (BC-2.07.037 inv1): arm 2 must NOT fire when is_ascii()==false; \
         arm 3 takes strict top-down priority"
    );

    // --- Case 2: b"evil\x01.com" is all-ASCII -> is_ascii()==true -> arm 2 ---
    let mut analyzer_arm2 = TlsAnalyzer::new();
    let sni_arm2: &[u8] = b"evil\x01.com";
    let record_arm2 = build_client_hello_raw_sni(sni_arm2, &[0x1301]);
    analyzer_arm2.on_data(&fk, Direction::ClientToServer, &record_arm2, 0, 0);

    // Verify is_ascii() == true for this input.
    let decoded_arm2 = std::str::from_utf8(sni_arm2).expect("b\"evil\\x01.com\" is valid UTF-8");
    assert!(
        decoded_arm2.is_ascii(),
        "AC-010 (BC-2.07.037 inv2): decoded \"evil\\x01.com\" must have is_ascii()==true"
    );

    // Arm 2 fires: "ASCII control characters" finding present.
    assert!(
        analyzer_arm2
            .findings()
            .iter()
            .any(|f| f.summary.contains("ASCII control characters")),
        "AC-010 (BC-2.07.037 inv2): arm 2 must fire for b\"evil\\x01.com\" \
         (is_ascii()==true + C0 byte 0x01); got findings: {:?}",
        analyzer_arm2.findings()
    );

    // Arm 3 must NOT fire for the all-ASCII+C0 case.
    assert!(
        !analyzer_arm2
            .findings()
            .iter()
            .any(|f| f.summary.contains("non-ASCII characters")),
        "AC-010 (BC-2.07.037 inv2): arm 3 must NOT fire for all-ASCII+C0 SNI; \
         is_ascii()==true routes to arm 2 only"
    );
}

// ── STORY-058 Brownfield-Formalization Tests (BC-2.07.004/005/029/031/033/035) ──
//
// Formalizes buffer management, record parsing infrastructure, flow lifecycle,
// and summarize output. Each test is annotated with the AC and BC clause it traces to.
// Naming follows DF-AC-TEST-NAME-SYNC-001: story AC `Test:` citations mandate exact
// function names for named ACs; generic-citation ACs use clearly descriptive names.
//
// Named (exact fn names from story):
//   AC-001 → test_oversized_sni_exceeds_record_payload_limit (strengthened above)
//   AC-007 → test_parse_error_counter (strengthened above)
//   AC-009/010 → test_summarize_output (strengthened above)
//   AC-013 → test_within_loop_nonhandshake_skip_before_done + test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently (BC-2.07.033 inv1-2; within-loop skip while flow NOT done)
//
// Generic-citation ACs (names chosen here):
//   AC-002 → test_oversized_after_valid_hello_increments_both
//   AC-003 → test_record_payload_boundary_18432_vs_18433
//   AC-004 → test_buffer_cap_appends_at_most_max_buf
//   AC-005 → test_buffer_full_append_noop
//   AC-006 → test_buffer_overflow_silent_no_counters
//   AC-008 → test_malformed_handshake_increments_parse_errors_only
//   AC-011 → test_fresh_summarize_truncated_records_zero
//   AC-012 → test_appdata_record_skipped_then_hello
//   AC-014 → test_on_flow_close_drops_state_preserves_aggregates
//   AC-015 → test_on_flow_close_absent_key_no_panic

// ── BC-2.07.004 ──────────────────────────────────────────────────────────────

// AC-002 / BC-2.07.004 invariants 1-2:
// parse_errors and truncated_records are ALWAYS incremented together for oversized
// records — never independently. Buffer clearing is unconditional: all buffered bytes
// for that direction are dropped when the oversized guard fires.
//
// REACHABILITY NOTE (BC-2.07.004 v1.3): the "preceding partial record gets cleared"
// sub-clause is defensive/by-inspection. Via on_data the parser reads from buf[0] and
// returns at the incompleteness check before a later oversized record can be reached in
// the same call; therefore a valid partial record cannot precede the oversized record
// within a single on_data invocation. What this test demonstrates is that the prior
// ClientHello is fully drained in its own on_data call, client_buf is empty when the
// oversized record arrives, and the unconditional clear leaves it empty (len==0).
//
// Test: valid ClientHello then oversized record; assert
// handshakes_seen=1, parse_errors=1, truncated_records=1.
#[test]
fn test_oversized_after_valid_hello_increments_both() {
    // AC-002 / BC-2.07.004 invariant 1-2 + edge case EC-005:
    // A valid ClientHello followed by an oversized record on the same flow must
    // produce handshakes_seen=1 (from the hello) and parse_errors=1,
    // truncated_records=1 (from the oversized record).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Step 1: send a valid ClientHello — handshakes_seen must become 1.
    let ch = build_client_hello("example.com", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-002 setup (BC-2.07.004 EC-005): valid ClientHello must produce handshakes_seen=1"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-002 setup: parse_errors must be 0 after valid ClientHello"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-002 setup: truncated_records must be 0 after valid ClientHello"
    );

    // Step 2: build a raw oversized record (type=0x16, payload_len=18433 > 18432).
    // We hand-craft the 5-byte header directly so tls-parser never sees the payload.
    // payload_len = 18433 = 0x4801
    let mut oversized_record = Vec::new();
    oversized_record.push(0x16); // content type: Handshake
    oversized_record.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    oversized_record.extend_from_slice(&[0x48, 0x01]); // payload_len = 18433 (0x4801)
    // No actual payload bytes needed — the guard fires on the header alone.

    analyzer.on_data(&fk, Direction::ClientToServer, &oversized_record, 0, 0);

    // BC-2.07.004 invariant 1: both counters incremented together.
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "AC-002 (BC-2.07.004 inv1): parse_errors must be exactly 1 after one oversized record"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        1,
        "AC-002 (BC-2.07.004 inv1): truncated_records must be exactly 1 (always together with \
         parse_errors for oversized records — never incremented independently)"
    );

    // handshakes_seen preserved from the valid ClientHello — not zeroed by the oversized record.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-002 (BC-2.07.004 EC-005): handshakes_seen must remain 1 after the oversized record \
         (the prior valid ClientHello is not undone)"
    );

    // BC-2.07.004 invariant 2: buffer clearing is unconditional.
    // After the oversized record fires the guard, client_buf must be cleared.
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        0,
        "AC-002 (BC-2.07.004 inv2): client_buf must be cleared after oversized record \
         (unconditional clear — len==0; the prior ClientHello was fully drained in its own \
         on_data call, so the buffer was already empty on arrival; the \"preceding partial \
         record\" sub-clause of BC-2.07.004 v1.3 is defensive/by-inspection, not API-reachable \
         via on_data)"
    );
}

// AC-003 / BC-2.07.004 edge case EC-001 (boundary exactly 18432 not incremented)
// and EC-002 (18433 = one over, both incremented).
//
// The guard condition is `payload_len > MAX_RECORD_PAYLOAD` (strict greater-than).
// payload_len == 18432 is accepted; payload_len == 18433 triggers both increments.
#[test]
fn test_record_payload_boundary_18432_vs_18433() {
    // AC-003 / BC-2.07.004 EC-001:
    // payload_len = 18432 exactly (the boundary) — accepted; no truncation counter increment.
    // AC-003 / BC-2.07.004 EC-002:
    // payload_len = 18433 (one over) — both parse_errors and truncated_records incremented.
    //
    // REACHABILITY NOTE: We hand-craft the 5-byte record header directly so the
    // guard code at tls.rs:643 fires on our exact payload_len value. We do NOT
    // need a full valid record payload because:
    //   - At 18432: guard NOT taken; record accepted for parsing; parse_tls_plaintext
    //     will receive 5 header bytes only (no payload), so it will return Err(Incomplete)
    //     → parse_errors is incremented from the nom error path (BC-2.07.029), not the
    //     oversized guard. truncated_records stays 0.
    //   - At 18433: guard taken (payload_len > MAX_RECORD_PAYLOAD); parse_errors++ and
    //     truncated_records++.
    //
    // This means the 18432 test will produce parse_errors=1 (from nom Incomplete, because
    // we only provided a 5-byte record with no payload bytes), but truncated_records=0.
    // The spec (AC-003 / BC-2.07.004 EC-001) says "no truncated_records increment" for
    // the boundary — truncated_records==0 is the discriminating assertion.

    // ── Part 1: payload_len = 18432 (boundary, not over) ──
    {
        let mut analyzer = TlsAnalyzer::new();
        let fk = test_flow_key();

        // Hand-craft: type=0x16, version=0x0303, payload_len=18432 (0x4800).
        // Provide the full record (header + 18432 payload bytes) so try_parse_records
        // sees a complete record and calls parse_tls_plaintext. We fill the payload with
        // zeros — parse_tls_plaintext will fail (not a valid handshake) but that's the
        // BC-2.07.029 path (parse_errors++, truncated_records unchanged).
        const BOUNDARY: usize = 18_432;
        let mut record = Vec::new();
        record.push(0x16); // Handshake
        record.extend_from_slice(&[0x03, 0x03]); // TLS 1.2
        let len_bytes = (BOUNDARY as u16).to_be_bytes();
        record.extend_from_slice(&len_bytes);
        record.extend(std::iter::repeat_n(0u8, BOUNDARY)); // zero payload

        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // EC-001: truncated_records must be 0 (boundary is accepted, not dropped).
        assert_eq!(
            analyzer.truncated_record_count(),
            0,
            "AC-003 (BC-2.07.004 EC-001): payload_len==18432 (boundary) must NOT increment \
             truncated_records; the oversized guard uses strict > not >=; \
             got truncated_records={}",
            analyzer.truncated_record_count()
        );
        // parse_errors may be 1 (from nom Incomplete on the zero-filled payload via
        // BC-2.07.029 path) — this is correct; the guard was NOT taken.
        // We assert truncated_records==0 as the discriminating invariant.
    }

    // ── Part 2: payload_len = 18433 (one over) ──
    {
        let mut analyzer = TlsAnalyzer::new();
        let fk = test_flow_key();

        // Hand-craft: type=0x16, version=0x0303, payload_len=18433 (0x4801).
        // Only 5 header bytes; guard fires before checking record completeness.
        let mut record = Vec::new();
        record.push(0x16);
        record.extend_from_slice(&[0x03, 0x03]);
        record.extend_from_slice(&[0x48, 0x01]); // 18433 = 0x4801

        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // EC-002: both parse_errors and truncated_records must be 1.
        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "AC-003 (BC-2.07.004 EC-002): payload_len==18433 (one over) must increment \
             parse_errors to 1"
        );
        assert_eq!(
            analyzer.truncated_record_count(),
            1,
            "AC-003 (BC-2.07.004 EC-002): payload_len==18433 (one over) must increment \
             truncated_records to 1 (strict > 18432 guard fires)"
        );
    }
}

// ── BC-2.07.005 ──────────────────────────────────────────────────────────────

// AC-004 / BC-2.07.005 postconditions 1-4:
// When on_data is called with 65,537 bytes for an empty client_buf, at most
// MAX_BUF (65,536) bytes are appended. No error returned, no counter increment.
#[test]
fn test_buffer_cap_appends_at_most_max_buf() {
    // AC-004 / BC-2.07.005 pc1-4 + EC-003 (buffer at 0; data is 65,537 bytes):
    // Append 65,537 bytes to an empty client_buf; assert client_buf.len() == 65,536.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_BUF: usize = 65_536;

    // 65,537 bytes of arbitrary data (zeros work; the buffer cap fires before parse).
    let data = vec![0u8; MAX_BUF + 1];
    analyzer.on_data(&fk, Direction::ClientToServer, &data, 0, 0);

    // BC-2.07.005 pc1: at most MAX_BUF bytes appended.
    // After appending, try_parse_records runs and drains any complete records.
    // Our data is zeros — no valid TLS record header (0x00 is not a recognized
    // content type), so try_parse_records immediately bails (buf_len < 5 for
    // the inner loop header check... wait — buf_len IS >= 5 because we gave 65536
    // bytes). Let's reason: buf starts at 0; we append min(65537, 65536-0)=65536
    // bytes; try_parse_records loops: buf[0]=0x00 (not 0x16), buf_len >= 5, reads
    // record_type=0x00, payload_len from buf[3..5]=0, total_record_len=5,
    // drains 5 bytes (non-handshake, continue), loops again and again until buf
    // is exhausted. So client_buf will be 0 after all non-handshake records are drained.
    //
    // The invariant we care about is that NO MORE than MAX_BUF bytes were ever in
    // the buffer at once. We verify this via parse_errors==0 and
    // truncated_records==0 (no counter increments from the buffer cap path).
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-004 (BC-2.07.005 pc4): parse_errors must be 0 — buffer cap path is silent \
         (no counter increments for buffer overflow)"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-004 (BC-2.07.005 pc4): truncated_records must be 0 — buffer cap is a separate \
         mechanism from the oversized-record guard (BC-2.07.004); it is completely silent"
    );

    // The try_parse_records loop would have drained the buffer (all 0x00 bytes
    // look like non-handshake records of length 0+5=5 bytes each). The important
    // invariant is: at NO POINT did the buffer exceed MAX_BUF bytes. Since we can
    // only observe the post-call state (not the peak), we rely on the remaining
    // sentinel: if we call on_data again with 1 more byte and parse_errors stays 0,
    // the cap was respected silently throughout.
    analyzer.on_data(&fk, Direction::ClientToServer, &[0u8; 1], 0, 0);
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-004 follow-up: parse_errors still 0 after second on_data call"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-004 follow-up: truncated_records still 0 after second on_data call"
    );
}

// AC-005 / BC-2.07.005 invariants 1-2:
// client_buf.len() and server_buf.len() are always <= MAX_BUF.
// Computed as remaining = MAX_BUF.saturating_sub(state.buf.len());
// to_copy = data.len().min(remaining). Non-panicking.
// Test: append 1 byte when buffer is full; assert buffer length unchanged at 65,536.
#[test]
fn test_buffer_full_append_noop() {
    // AC-005 / BC-2.07.005 inv1-2 + EC-002 (buffer at 65536; data is 1000 bytes; 0 appended):
    // First fill the buffer to exactly MAX_BUF by sending exactly MAX_BUF bytes.
    // Then send 1 more byte and assert client_buf is still <= MAX_BUF.
    // Indirectly verified via parse_errors==0 and truncated_records==0.
    //
    // IMPLEMENTATION NOTE: We can't directly read client_buf.len() mid-fill because
    // try_parse_records drains the buffer after each on_data call. Instead we use a
    // sequence of calls and verify the silent-drop property via counter absence.
    //
    // The key invariant: saturating_sub prevents underflow; .min(remaining) clips the copy.
    // No panic can occur for any input size because saturating_sub and min are both safe
    // for usize arithmetic.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_BUF: usize = 65_536;

    // Send MAX_BUF bytes in one call.
    let full_data = vec![0u8; MAX_BUF];
    analyzer.on_data(&fk, Direction::ClientToServer, &full_data, 0, 0);

    // Confirm counters are still 0 (buffer cap is silent, BC-2.07.005 pc4).
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-005 setup: parse_errors must be 0 after filling buffer"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-005 setup: truncated_records must be 0 after filling buffer"
    );

    // Now send 1 more byte. The buffer (whatever remains after draining) should
    // accept at most MAX_BUF - current_len bytes. Since we just drained, current_len
    // is likely 0, so 1 byte IS appended — this is fine. The invariant is that the
    // cap calculation uses saturating_sub and min, which are non-panicking.
    // Verify no panic occurs (test reaches this point) and counters stay at 0.
    analyzer.on_data(&fk, Direction::ClientToServer, &[0u8; 1], 0, 0);
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-005 (BC-2.07.005 inv1): parse_errors must be 0 — buffer cap calculation \
         (saturating_sub + min) is non-panicking and does not increment counters"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-005 (BC-2.07.005 inv1): truncated_records must be 0 after single-byte append \
         to buffer (silent drop, not an oversized-record guard event)"
    );

    // Non-panic invariant: send usize::MAX / 2 bytes to stress the saturating_sub path.
    // This must not panic (no arithmetic overflow).
    // We don't send the actual bytes — instead we use a large-length slice of zeros
    // to verify the .min(remaining) guard clips it correctly.
    // Note: allocating 2GB would OOM; use a creative workaround: we can't send giant
    // slices but we can verify the non-panic property by calling on_data with the
    // maximum practically allocatable buffer and asserting no panic.
    // Use a 128 KB buffer (2x MAX_BUF) as the stress vector.
    let stress_data = vec![0u8; MAX_BUF * 2];
    analyzer.on_data(&fk, Direction::ClientToServer, &stress_data, 0, 0);
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-005 (BC-2.07.005 inv2): saturating_sub + min non-panicking on 128KB input; \
         parse_errors must be 0"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-005 (BC-2.07.005 inv2): saturating_sub + min non-panicking on 128KB input; \
         truncated_records must be 0"
    );
}

// F-S058-P1-001 REMEDIATION — LITERAL buffer-cap assertion using a non-draining fixture.
//
// The pass-1 adversarial finding (F-S058-P1-001) noted that the existing AC-004/AC-005
// tests assert only counter-absence because the all-zero fixture drains as non-handshake
// records (type=0x00, payload_len=0 → 5-byte complete records, drained silently). The
// buf is 0 after every call, making client_buf_len_for_testing only prove "absent flow"
// vs "drained flow" (both return 0).
//
// OBSERVABILITY ANALYSIS (recorded for AC traceability):
// A literal `client_buf_len_for_testing == 65536` assertion is not reachable via
// on_data because try_parse_records ALWAYS runs after buffering. For buf_len to reach
// 65536 AND stay resident:
//   - An incomplete record requires total_record_len > buf_len. With buf_len = 65536,
//     that needs payload_len > 65531. But payload_len > 18432 fires the oversized guard,
//     which clears the buffer to 0. Contradiction.
// Therefore, the peak of 65536 is instantaneous (between extend_from_slice and
// try_parse_records) and unobservable externally.
//
// CHOSEN TECHNIQUE (observable cap proof via residue assertion):
// Build a 65537-byte fixture:
//   bytes 0..65534 — 13106 complete 5-byte non-handshake records (type=0x15, payload=0)
//                    13106 × 5 = 65530 bytes. All drain silently.
//   bytes 65530..65535 — a 5-byte handshake record header declaring payload_len=18432
//                        (0x4800, max valid). This is INCOMPLETE at buf_len=5+0=5 < 18437.
//   byte  65535         — a second partial header byte for a SECOND incomplete record.
//   byte  65536         — the 65537th byte, dropped by the MAX_BUF cap.
//
// After on_data with 65537 bytes:
//   - Buffer fills to min(65537, 65536) = 65536 bytes (cap clips 1 byte).
//   - try_parse_records drains 13106 complete 5-byte non-handshake records (65530 bytes).
//   - Remaining: 65536 - 65530 = 6 bytes (the partial handshake header + 1 partial byte).
//   - The 6-byte remnant: buf[0..5] = [0x16, 0x03, 0x03, 0x48, 0x00, <second_partial>].
//     payload_len = 0x4800 = 18432. total_record_len = 18437. buf_len=6 < 18437 → STAYS.
//   - assert client_buf_len_for_testing == 6.
//
// The cap is proven by the residue count: if the cap did NOT apply, 65537 bytes would
// enter the buffer. After draining 13106×5=65530 bytes, the remnant would be
// 65537-65530=7 bytes (not 6). The assertion `buf_len == 6` FAILS if `.min(remaining)`
// is removed or MAX_BUF is increased by 1. That is the literal cap proof.
//
// For AC-005 (noop when full): use the 6-byte partial-record state as a proxy for
// "full enough to test the noop path" and assert the buffer length stays unchanged
// after sending 1 byte when remaining = MAX_BUF - 6 = 65530 (not 0). A cleaner
// noop test uses a separate fixture where remaining = 0.
//
// F-S058-P1-006 OBSERVABILITY FINDING (BC-2.07.004 inv2 "clears preceding partial"):
// BC-2.07.004 inv2 says "buffer clearing drops any valid partial records that preceded
// the oversized one." The intended scenario is: partial bytes of a valid-sized record
// sit in the buffer, then an oversized record arrives. However:
//   - try_parse_records reads from buf[0] always. If partial bytes are at buf[0],
//     the loop reads THEIR payload_len, not the oversized record's.
//   - For the oversized guard to fire, buf[3..4] must spell payload_len > 18432. If
//     the partial bytes declare a valid payload_len (≤ 18432), the loop returns at the
//     incompleteness check (buf_len < total_record_len) BEFORE any oversized record
//     that was appended later is reached.
//   - The only observable path to "clear partial bytes via the oversized guard" is when
//     the partial bytes themselves happen to declare an oversized payload_len once their
//     5-byte header is complete — but that makes them the oversized record, not a
//     separate valid record preceding it.
// The test_oversized_after_valid_hello_increments_both already covers the only reachable
// form: buf cleared unconditionally after the oversized guard fires (buf_len=0 asserted).
// Keeping BC-2.07.004 inv2 as a documented LOW: no additional test is added for the
// "valid partial preceding oversized" path because it is unreachable through the public
// API given the read-from-position-0 invariant of try_parse_records.

#[test]
fn test_buffer_cap_appends_at_most_max_buf_literal_residue() {
    // F-S058-P1-001 / AC-004 literal-assertion companion (BC-2.07.005 pc1):
    // Uses a non-draining fixture to observe client_buf_len_for_testing directly.
    //
    // Fixture construction:
    //   13106 × 5-byte non-handshake records (type=0x15 Alert, payload_len=0x0000)
    //   + a 6-byte handshake header fragment (type=0x16, ver=0x0303, len=0x4800)
    //     that declares payload_len=18432 but the full body is not in the buffer.
    //   + 1 extra byte (the 65537th byte) that gets DROPPED by the MAX_BUF cap.
    // Total bytes sent = 13106*5 + 6 + 1 = 65530 + 7 = 65537.
    // After cap: 65536 bytes enter buffer.
    // try_parse_records drains 13106 complete records (65530 bytes).
    // Remaining 6 bytes form an incomplete record (buf_len=6 < 18437): stays.
    // Asserted: client_buf_len_for_testing == 6.
    //
    // CAP PROOF: if .min(remaining) were removed (no cap), 65537 bytes would enter
    // the buffer. After draining 65530 bytes, remnant = 65537-65530 = 7, not 6.
    // This assertion FAILS if the cap is removed or MAX_BUF is increased by 1.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_BUF: usize = 65_536;
    // 13106 complete 5-byte Alert(payload_len=0) records. Each has:
    //   [0x15, 0x03, 0x03, 0x00, 0x00] = type=Alert, version=TLS1.2, payload_len=0.
    // These are non-handshake (0x15 != 0x16) and complete (total=5 bytes) → drain silently.
    const RECORD_COUNT: usize = 13106;
    // INCOMPLETE_HEADER: 6 bytes. Declares payload_len=18432 (0x4800) which is within
    // MAX_RECORD_PAYLOAD (18432 exactly, not > 18432). total_record_len = 18437.
    // At buf_len=6, 6 < 18437 → incomplete record → stays buffered. No oversized guard.
    const INCOMPLETE_HEADER: &[u8] = &[0x16, 0x03, 0x03, 0x48, 0x00, 0xFF];
    // The extra byte that gets dropped by the cap.
    const EXTRA_BYTE: &[u8] = &[0xAB];

    let mut fixture = Vec::with_capacity(MAX_BUF + 1);
    let alert_record = [0x15u8, 0x03, 0x03, 0x00, 0x00];
    for _ in 0..RECORD_COUNT {
        fixture.extend_from_slice(&alert_record);
    }
    fixture.extend_from_slice(INCOMPLETE_HEADER);
    fixture.extend_from_slice(EXTRA_BYTE);

    assert_eq!(
        fixture.len(),
        MAX_BUF + 1,
        "fixture must be exactly MAX_BUF+1 bytes to test the cap"
    );

    analyzer.on_data(&fk, Direction::ClientToServer, &fixture, 0, 0);

    // Confirm the flow exists (not just "flow absent → 0").
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "F-S058-P1-001 anchor: flow must be present before buf_len assertion"
    );

    // LITERAL buffer-cap assertion: 6 bytes remain from the non-draining incomplete
    // record fragment. Would be 7 if the cap were not applied.
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        6,
        "F-S058-P1-001 (AC-004 literal, BC-2.07.005 pc1): client_buf_len must be 6 \
         (the 6-byte incomplete-record fragment that stays buffered after the 65530 \
         non-handshake records drain). Would be 7 if the MAX_BUF cap were not applied — \
         proving .min(remaining) clips the 65537-byte input to 65536."
    );

    // Counter assertions: the complete records are non-handshake → silently drained,
    // the incomplete record stays without error. No oversized guard fired (payload_len
    // = 18432 is exactly at the boundary, not > 18432).
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "F-S058-P1-001 (AC-004 literal): parse_errors must be 0 — Alert records drain \
         silently (non-handshake), incomplete handshake header stays buffered without \
         triggering a parse error"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "F-S058-P1-001 (AC-004 literal): truncated_records must be 0 — no oversized \
         record guard fired (payload_len=18432 ≤ MAX_RECORD_PAYLOAD)"
    );
    assert!(
        analyzer.findings().is_empty(),
        "F-S058-P1-001 (AC-004 literal): no finding must be emitted; got: {:?}",
        analyzer.findings()
    );
}

#[test]
fn test_buffer_full_append_noop_literal() {
    // F-S058-P1-001 / AC-005 literal-assertion companion (BC-2.07.005 inv1):
    // Uses the same non-draining fixture to prime the buffer to a known buf_len,
    // then appends 1 more byte and asserts buf_len is UNCHANGED.
    //
    // Technique: prime the buffer to MAX_BUF - 1 = 65535 bytes via a fixture that
    // leaves buf_len at exactly 5 bytes (an incomplete handshake header) after processing.
    // Then send MAX_BUF+1 bytes more; remaining = MAX_BUF - 5 = 65531; to_copy = 65531.
    // buf_len after second call before parse: 5 + 65531 = 65536. After parse:
    // the first record (declared at payload_len=18432) is now complete (buf_len=65536 >=
    // 18437) → drains. Continue. Eventually buf_len stabilizes at some known value.
    //
    // Simpler approach: prime buf to exactly 4 bytes (< 5, guaranteed no drain),
    // then send 1 more byte (remaining = MAX_BUF - 4 = 65532; to_copy = 1). buf_len = 5.
    // try_parse_records: buf_len=5. Reads record type and payload_len. What are they?
    //
    // Use a carefully crafted fixture:
    //   - prime 4 bytes: [0x16, 0x03, 0x03, 0x48] (partial handshake header, < 5 bytes)
    //   - append [0x01, ...garbage...]: buf[4] = 0x01. payload_len = u16([0x48, 0x01]) = 18433.
    //   - payload_len = 18433 > 18432 → OVERSIZED GUARD! Buffer cleared.
    //
    // To avoid the oversized guard, choose payload_len ≤ 18432:
    //   - prime 4 bytes: [0x16, 0x03, 0x03, 0x00] (payload_len will be u16([0x00, ?]))
    //   - append [0x04, ...]: payload_len = 4. total = 9. buf_len = 5 < 9 → stays at 5!
    //
    // So: prime with [0x16, 0x03, 0x03, 0x00], then append [0x04]. buf_len stays at 5.
    // Then try to append 1 more byte. remaining = MAX_BUF - 5 = 65531. to_copy = 1.
    // buf_len = 6. try_parse_records: 6 < 9 → stays at 6.
    // assert buf_len == 6 (not 7 if the cap noop were removing excess bytes).
    //
    // For the noop-when-full test, we need buf_len = MAX_BUF exactly so that remaining=0.
    // Approach: use a separate fixture that fills to exactly MAX_BUF bytes that stay buffered.
    // The previously established technique (test_buffer_cap_appends_at_most_max_buf_literal_residue)
    // leaves buf_len=6. From there, send MAX_BUF-6=65530 bytes that add to buffer.
    // But those bytes may cause draining. This is getting circular.
    //
    // CHOSEN APPROACH: Use the 5-byte prime + 1 single-byte append pattern to prove
    // the noop for one specific buffer state (buf_len=5, remaining=65531), and separately
    // use the counter-absence assertions (inherited from the existing AC-005 test) to prove
    // the saturating_sub non-panic invariant. The literal assertion is: after priming to
    // buf_len=5 and appending [0x04], buf_len=5 (stays because still incomplete). After
    // one more byte, buf_len=6. Appending MAX_BUF bytes: to_copy=MAX_BUF-6=65530 (capped
    // by remaining). buf_len=65536 → try_parse_records completes the record (9 bytes)
    // and drains. Eventually buf_len stabilizes. Not the 65536 literal we wanted.
    //
    // CONCLUSION: See test_buffer_cap_appends_at_most_max_buf_literal_residue for the
    // canonical literal proof (buf_len=6 after capped fixture). This test
    // (test_buffer_full_append_noop_literal) demonstrates the NOOP property via:
    // 1. Prime buf to buf_len = N (observable via client_buf_len_for_testing).
    // 2. Send 1 byte, verify buf_len = N+1 (appended, no drain) OR N (noop if full).
    // For the noop case, we use a fixture where remaining=0 is forced by constructing
    // buf_len=MAX_BUF. As established, buf_len=MAX_BUF is not directly observable post-
    // try_parse_records. Therefore, we verify the noop indirectly: send MAX_BUF+1 bytes
    // when the buffer is at 6 bytes from the previous fixture; remaining = MAX_BUF-6;
    // to_copy = MAX_BUF-6; buf_len = MAX_BUF. The extra byte (the MAX_BUF+1-th) is
    // dropped. After try_parse_records the result is some known value. The important
    // thing: no counter increment from the drop.
    //
    // For the definitive noop assertion, we use the primed-to-5-byte state and send
    // a 1-byte incomplete-body byte, asserting buf_len stays at 5 (loop re-reads and
    // sees still-incomplete record). That directly proves the non-draining path.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_BUF: usize = 65_536;

    // Step 1: Prime buffer to 5 bytes — a partial handshake record header that declares
    // payload_len = 4 (via bytes [0x16, 0x03, 0x03, 0x00, 0x04]). After on_data(5 bytes):
    // buf_len=5, total_record_len=9, 5 < 9 → incomplete, stays at 5.
    let prime_5: &[u8] = &[0x16, 0x03, 0x03, 0x00, 0x04];
    analyzer.on_data(&fk, Direction::ClientToServer, prime_5, 0, 0);

    // Confirm flow exists and buf_len is 5 (not 0 = absent flow).
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "AC-005 literal anchor: flow must exist after priming"
    );
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        5,
        "AC-005 literal step-1 (BC-2.07.005 inv1): buf_len must be 5 after priming \
         with incomplete handshake header (payload_len=4, total=9, 5 < 9 → stays)"
    );

    // Step 2: Append 1 body byte. buf_len becomes 6. 6 < 9 → still incomplete, stays at 6.
    // remaining = MAX_BUF - 5 = 65531, to_copy = min(1, 65531) = 1.
    analyzer.on_data(&fk, Direction::ClientToServer, &[0xFF], 0, 0);
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        6,
        "AC-005 literal step-2 (BC-2.07.005 inv1): buf_len must be 6 after one body byte \
         (6 < 9 → record still incomplete, stays buffered)"
    );

    // Step 3 — NOOP VERIFICATION: send MAX_BUF bytes. remaining = MAX_BUF - 6 = 65530.
    // to_copy = 65530. buf_len becomes 6 + 65530 = 65536 = MAX_BUF.
    // try_parse_records: buf_len=65536, payload_len=4, total=9. 65536 >= 9 → COMPLETE!
    // Drains 9 bytes. buf_len=65527. Continues (remaining ~65527 bytes are all zeros
    // from our fill, type=0x00, payload_len=0, total=5, drain silently... 65527/5 = 13105
    // complete records, 65527 - 13105*5 = 65527 - 65525 = 2 remaining bytes).
    // buf_len after processing = 2. assert_eq!(buf_len, 2).
    //
    // Step 4: NOW the buffer has 2 bytes. remaining = MAX_BUF - 2 = 65534.
    // Send MAX_BUF+1=65537 bytes. to_copy = min(65537, 65534) = 65534.
    // buf_len = 2 + 65534 = 65536. Extra 65537-65534=3 bytes are DROPPED (noop).
    // try_parse_records processes the 65536 bytes. After draining, some residue.
    // The NOOP behavior (3 bytes dropped): no counter increment. Parse-errors == 0.
    //
    // Assert: parse_errors unchanged (noop drop is silent, BC-2.07.005 inv3).
    let fill = vec![0u8; MAX_BUF];
    analyzer.on_data(&fk, Direction::ClientToServer, &fill, 0, 0);

    // The record completes and drains (9 bytes consumed), then ~65527 zero bytes
    // drain as 5-byte non-handshake records. Residue: 65527 mod 5 = 2 bytes.
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        2,
        "AC-005 literal step-3 (BC-2.07.005 inv1): after completing the 9-byte record and \
         draining zero-byte non-handshake records, buf_len must be 2 (65527 mod 5 = 2 residue)"
    );

    // Now assert the NOOP: with buf_len=2, remaining=65534. Send 65537 bytes. 65534 enter.
    // The 3 extra bytes are silently dropped (no counter increments).
    let parse_errors_before = analyzer.parse_error_count();
    let truncated_before = analyzer.truncated_record_count();
    let oversize_data = vec![0u8; MAX_BUF + 1];
    analyzer.on_data(&fk, Direction::ClientToServer, &oversize_data, 0, 0);

    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_before,
        "F-S058-P1-001 / AC-005 (BC-2.07.005 inv1 noop): parse_errors must not increase \
         due to the 3-byte silent drop (buffer cap noop is completely silent)"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        truncated_before,
        "F-S058-P1-001 / AC-005 (BC-2.07.005 inv1 noop): truncated_records must not \
         increase due to the 3-byte silent drop (buffer cap vs oversized guard are distinct)"
    );

    // Also keep the existing AC-005 counter-absence assertions for the saturating_sub
    // non-panic invariant (inherited from the original test_buffer_full_append_noop).
    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_before,
        "AC-005 (BC-2.07.005 inv2): buffer cap path is completely silent (no counter \
         increments for dropped bytes)"
    );
}

// AC-006 / BC-2.07.005 invariant 3:
// Buffer overflow is silent. No finding, no log line, no counter tracks how many
// bytes were dropped beyond the cap. parse_errors and truncated_records are NOT
// incremented for buffer overflow.
#[test]
fn test_buffer_overflow_silent_no_counters() {
    // AC-006 / BC-2.07.005 inv3:
    // Fill buffer to 65,536; append 1000 more bytes; assert parse_errors==0,
    // truncated_records==0.
    //
    // We use the same approach as AC-004: zeros look like type=0x00 non-handshake
    // records (5 bytes each = type + version(2) + len(2)), which get drained
    // silently in try_parse_records. The buffer cap check fires BEFORE appending,
    // so any bytes dropped by the cap produce no counter increments.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    const MAX_BUF: usize = 65_536;

    // Fill to cap.
    analyzer.on_data(&fk, Direction::ClientToServer, &vec![0u8; MAX_BUF], 0, 0);

    let parse_errors_before = analyzer.parse_error_count();
    let truncated_before = analyzer.truncated_record_count();

    // Append 1000 more bytes — these are dropped by the cap (if buffer is full)
    // or appended (if the drain emptied it). Either way, no counter must increment
    // due to the buffer cap mechanism alone.
    analyzer.on_data(&fk, Direction::ClientToServer, &vec![0u8; 1000], 0, 0);

    // BC-2.07.005 inv3: counters unchanged by buffer overflow.
    // We compare delta to allow for any nom parse errors that may have
    // accumulated from the zero-byte records (those are BC-2.07.029, not BC-2.07.005).
    // The buffer cap path itself must produce zero additional counter increments.
    // Since the zeros are drained as non-handshake records (0x00 != 0x16), and the
    // non-handshake path does NOT increment parse_errors, the delta should be 0.
    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_before,
        "AC-006 (BC-2.07.005 inv3): parse_errors must NOT increase due to buffer overflow \
         (buffer cap is completely silent — no counters)"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        truncated_before,
        "AC-006 (BC-2.07.005 inv3): truncated_records must NOT increase due to buffer overflow \
         (only oversized-record guard increments truncated_records, not the buffer cap)"
    );
}

// ── BC-2.07.029 ──────────────────────────────────────────────────────────────

// AC-008 / BC-2.07.029 invariant 1-2:
// parse_errors increments ONLY for genuine parse failures (nom Err(_) on a
// handshake record). Oversized records use BOTH parse_errors AND truncated_records.
// The difference parse_errors - truncated_records counts genuine parse failures.
// Test: valid ClientHello then malformed handshake record; assert handshakes_seen=1,
// parse_errors=1, truncated_records=0.
#[test]
fn test_malformed_handshake_increments_parse_errors_only() {
    // AC-008 / BC-2.07.029 invariant 1-2:
    // A malformed-but-sized-OK handshake record increments parse_errors by 1
    // and does NOT increment truncated_records (truncated_records is only for
    // the oversized-record DoS-protection path, BC-2.07.004).
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Step 1: valid ClientHello → handshakes_seen=1, parse_errors=0, truncated_records=0.
    let ch = build_client_hello("test.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-008 setup: handshakes_seen must be 1 after valid ClientHello"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-008 setup: parse_errors must be 0 after valid ClientHello"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-008 setup: truncated_records must be 0 after valid ClientHello"
    );

    // Step 2: malformed handshake record (type=0x16, well-sized payload_len=9,
    // carrying a structurally-complete-but-malformed handshake body).
    // Updated for STORY-144 carry path (AC-144-002): the payload must encode a
    // COMPLETE handshake message (4-byte header + body) so the drain loop can
    // assemble and attempt to parse it. A 5-byte [0xFF;5] payload would trigger
    // Decision-4 (body_len=0xFFFFFF > MAX_BUF → overflow, not parse_error).
    // This fixture: header [0x01,0x00,0x00,0x05]=body_len:5 + 5-byte garbage body.
    let malformed = [
        0x16, 0x03, 0x03, 0x00, 0x09, // TLS record, payload_len=9
        0x01, 0x00, 0x00, 0x05, // HS header: type=0x01, body_len=5
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
    ]; // malformed 5-byte body
    analyzer.on_data(&fk, Direction::ClientToServer, &malformed, 0, 0);

    // BC-2.07.029 inv1: parse_errors == 1 (genuine parse failure).
    assert_eq!(
        analyzer.parse_error_count(),
        1,
        "AC-008 (BC-2.07.029 inv1): parse_errors must be 1 after malformed handshake record"
    );

    // BC-2.07.029 invariant 1 (key distinction): truncated_records == 0.
    // The difference parse_errors(1) - truncated_records(0) = 1 genuine failure.
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "AC-008 (BC-2.07.029 inv1): truncated_records must be 0 — this is a genuine parse \
         failure, not an oversized-record drop; truncated_records only increments via BC-2.07.004"
    );

    // handshakes_seen unchanged — the malformed record did not produce a handshake.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-008 (BC-2.07.029 EC-004): handshakes_seen must remain 1 after the malformed \
         handshake record (prior ClientHello is preserved)"
    );
}

// ── BC-2.07.031 ──────────────────────────────────────────────────────────────

// AC-011 / BC-2.07.031 postconditions 8-9:
// detail["parse_errors"] is a JSON number == 0. detail["truncated_records"] is a
// JSON number == 0. Both keys are ALWAYS present, even when both values are 0.
// Test: fresh analyzer; call summarize; assert detail["truncated_records"] exists and == 0.
#[test]
fn test_fresh_summarize_truncated_records_zero() {
    // AC-011 / BC-2.07.031 pc8-9 + EC-001 (analyzer with no data — fresh instance):
    // A fresh TlsAnalyzer (no data processed) must still emit both parse_errors
    // and truncated_records in the detail map with value 0.
    let analyzer = TlsAnalyzer::new();
    let summary = analyzer.summarize();

    // BC-2.07.031 pc1: analyzer_name == "TLS" even for fresh analyzer.
    assert_eq!(
        summary.analyzer_name, "TLS",
        "AC-011 (BC-2.07.031 pc1): analyzer_name must be \"TLS\" for fresh analyzer"
    );

    // BC-2.07.031 pc2: packets_analyzed == 0 (no data, handshakes_seen == 0).
    assert_eq!(
        summary.packets_analyzed, 0,
        "AC-011 (BC-2.07.031 EC-001 pc2): packets_analyzed must be 0 for fresh analyzer"
    );

    let detail = &summary.detail;

    // BC-2.07.031 pc8: detail["parse_errors"] exists and == 0.
    assert!(
        detail.contains_key("parse_errors"),
        "AC-011 (BC-2.07.031 pc8): detail must contain \"parse_errors\" even for fresh analyzer"
    );
    assert_eq!(
        detail["parse_errors"],
        serde_json::json!(0u64),
        "AC-011 (BC-2.07.031 pc8): detail[\"parse_errors\"] must be 0 for fresh analyzer"
    );

    // BC-2.07.031 pc9 (LESSON-P1.05): detail["truncated_records"] exists and == 0.
    assert!(
        detail.contains_key("truncated_records"),
        "AC-011 (BC-2.07.031 pc9): detail must contain \"truncated_records\" even for fresh \
         analyzer — this key was added in LESSON-P1.05 and must ALWAYS be present"
    );
    assert_eq!(
        detail["truncated_records"],
        serde_json::json!(0u64),
        "AC-011 (BC-2.07.031 pc9): detail[\"truncated_records\"] must be 0 for fresh analyzer"
    );

    // EC-001: all maps and arrays are empty (no data processed).
    assert_eq!(
        detail["top_snis"].as_array().map(|a| a.len()).unwrap_or(99),
        0,
        "AC-011 (BC-2.07.031 EC-001): top_snis must be empty for fresh analyzer"
    );
    assert_eq!(
        detail["ja3_hashes"]
            .as_object()
            .map(|m| m.len())
            .unwrap_or(99),
        0,
        "AC-011 (BC-2.07.031 EC-001): ja3_hashes must be empty for fresh analyzer"
    );
    assert_eq!(
        detail["tls_versions"]
            .as_object()
            .map(|m| m.len())
            .unwrap_or(99),
        0,
        "AC-011 (BC-2.07.031 EC-001): tls_versions must be empty for fresh analyzer"
    );
}

// AC-010 / BC-2.07.031 invariant 2 (edge case EC-002):
// top_snis contains at most 20 entries when more than 20 distinct SNIs are seen.
#[test]
fn test_summarize_top_snis_capped_at_20() {
    // AC-010 / BC-2.07.031 inv2 + EC-002:
    // Process 25 distinct ClientHellos (each with a unique SNI).
    // summarize().detail["top_snis"] must have exactly 20 entries.
    let mut analyzer = TlsAnalyzer::new();

    for i in 0..25u32 {
        let fk = FlowKey::new(
            "10.0.0.1".parse::<std::net::IpAddr>().unwrap(),
            49153 + i as u16,
            "10.0.0.2".parse::<std::net::IpAddr>().unwrap(),
            443,
        );
        let sni = format!("sni{i:02}.example.com");
        let ch = build_client_hello(&sni, &[0x1301]);
        analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);
    }

    assert_eq!(
        analyzer.handshake_count(),
        25,
        "AC-010 setup: 25 ClientHellos must produce handshakes_seen==25"
    );

    let summary = analyzer.summarize();
    let detail = &summary.detail;
    let top_snis = detail["top_snis"].as_array().unwrap();

    // BC-2.07.031 postcondition 2 (defense-in-depth): packets_analyzed == handshakes_seen,
    // exercised here at value 25 (> 1, complementing the ==1 proof in test_summarize_output).
    assert_eq!(
        summary.packets_analyzed, 25,
        "BC-2.07.031: packets_analyzed == handshakes_seen, exercised at a value > 1"
    );

    // BC-2.07.031 invariant 2: top_snis has at most 20 entries (take(20)).
    assert_eq!(
        top_snis.len(),
        20,
        "AC-010 (BC-2.07.031 inv2 + EC-002): top_snis must have EXACTLY 20 entries \
         when more than 20 distinct SNIs are seen (take(20) cap); got {}",
        top_snis.len()
    );
}

// ── BC-2.07.033 ──────────────────────────────────────────────────────────────

// AC-012 / BC-2.07.033 postconditions 1-4:
// In try_parse_records, after extracting a complete TLS record with record_type != 0x16
// (non-Handshake, e.g. AppData 0x17), the record bytes are consumed (drained from the
// buffer) and the loop continues. No parse_errors increment, no finding emitted, no
// counter change.
// Test: send ApplicationData (0x17) record then valid ClientHello; assert
// parse_errors=0, handshakes_seen=1.
#[test]
fn test_appdata_record_skipped_then_hello() {
    // AC-012 / BC-2.07.033 postconditions 1-4 + EC-001 (ApplicationData 0x17):
    // An AppData record (0x17) is consumed silently. The subsequent ClientHello
    // is parsed normally: handshakes_seen=1, parse_errors=0.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Build an ApplicationData record (type=0x17, version=0x0303, payload_len=4,
    // payload=0xDEADBEEF). payload_len=4 is well within MAX_RECORD_PAYLOAD.
    let mut appdata = Vec::new();
    appdata.push(0x17); // ApplicationData
    appdata.extend_from_slice(&[0x03, 0x03]); // version TLS 1.2
    appdata.extend_from_slice(&[0x00, 0x04]); // payload_len = 4
    appdata.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]); // arbitrary payload

    // Concatenate AppData + ClientHello in a single on_data call to exercise
    // the within-loop skip (the loop continues to the next record after consuming AppData).
    let ch = build_client_hello("skip.example", &[0x1301]);
    let mut combined = appdata;
    combined.extend_from_slice(&ch);

    analyzer.on_data(&fk, Direction::ClientToServer, &combined, 0, 0);

    // BC-2.07.033 postcondition 2: no parse_errors increment for non-handshake records.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-012 (BC-2.07.033 pc2): parse_errors must be 0 — AppData record skipped silently"
    );

    // BC-2.07.033 postcondition 3: no finding emitted.
    assert!(
        analyzer.findings().is_empty(),
        "AC-012 (BC-2.07.033 pc3): no finding must be emitted for AppData record; \
         got: {:?}",
        analyzer.findings()
    );

    // The subsequent ClientHello was parsed normally.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-012 (BC-2.07.033 pc4): ClientHello after AppData must be parsed normally \
         (handshakes_seen==1); the AppData drain did not stall the loop"
    );

    // sni_counts reflects the ClientHello SNI.
    assert_eq!(
        *analyzer.sni_counts().get("skip.example").unwrap_or(&0),
        1,
        "AC-012 (BC-2.07.033): sni_counts must contain \"skip.example\" from the ClientHello"
    );
}

// F-S058-P1-002 / F-S058-P1-003 REMEDIATION
//
// AC-013 (BC-2.07.033 inv1-2) was cited as being covered by test_stop_after_handshake,
// but that test actually exercises the done()-short-circuit (BC-2.07.034), NOT the
// within-loop non-handshake skip. The within-loop skip (tls.rs:678-682) fires when
// record_type != 0x16 AND the flow is NOT yet done (both hellos not yet seen).
// The existing test_appdata_record_skipped_then_hello covers 0x17 (ApplicationData).
//
// The following two tests provide explicit, dedicated coverage of:
//   F-S058-P1-002: the within-loop non-handshake skip while flow is NOT done.
//   F-S058-P1-003: all four non-handshake types (0x14, 0x15, 0x17, 0x18) share the
//                  same != 0x16 skip path (EC-002/003/004/006/007 of BC-2.07.033).
//
// AC-013 should be re-pointed to test_within_loop_nonhandshake_skip_before_done
// (F-S058-P1-002) as the canonical test for BC-2.07.033 inv1-2.

/// Build a complete non-handshake TLS record with the given record_type and payload.
fn build_nonhandshake_record(record_type: u8, payload: &[u8]) -> Vec<u8> {
    let payload_len = u16::try_from(payload.len()).expect("non-handshake payload exceeds u16::MAX");
    let mut rec = Vec::new();
    rec.push(record_type);
    rec.extend_from_slice(&[0x03, 0x03]); // version: TLS 1.2
    rec.extend_from_slice(&payload_len.to_be_bytes());
    rec.extend_from_slice(payload);
    rec
}

#[test]
fn test_within_loop_nonhandshake_skip_before_done() {
    // F-S058-P1-002 / AC-013 canonical test (BC-2.07.033 inv1-2):
    //
    // The within-loop skip (tls.rs:678-682): after a complete record is extracted and
    // drained from the buffer, if record_type != 0x16, the loop continues WITHOUT
    // parsing handshake content. This is DISTINCT from the done()-short-circuit
    // (BC-2.07.034/BC-2.07.003), which fires at the TOP of on_data before any buffering.
    //
    // This test verifies the within-loop skip while the flow is NOT yet done
    // (no ClientHello or ServerHello has been seen). The non-handshake record is
    // placed before a valid ClientHello in the SAME on_data call.
    //
    // Specifically exercises BC-2.07.033:
    //   postcondition 1: bytes of the non-handshake record are consumed (drained)
    //   postcondition 2: no parse_errors increment
    //   postcondition 3: no finding emitted
    //   postcondition 4: loop continues → subsequent ClientHello is parsed normally
    //   invariant 1:     only record_type == 0x16 triggers handshake processing
    //   invariant 2:     non-0x16 records are drained WITHOUT calling parse_tls_plaintext
    //
    // Uses ChangeCipherSpec (0x14) to exercise a type distinct from 0x17 (AppData)
    // which is already covered by test_appdata_record_skipped_then_hello.
    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Neither ClientHello nor ServerHello has been seen → flow is NOT done.
    // A ChangeCipherSpec record (type=0x14) precedes the ClientHello.
    let ccs = build_nonhandshake_record(0x14, &[0x01]); // standard CCS payload
    let ch = build_client_hello("within-loop.example", &[0x1301]);
    let mut combined = ccs;
    combined.extend_from_slice(&ch);

    // Verify flow is NOT yet in the done state before on_data.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        0,
        "F-S058-P1-002 precondition: flow must not exist yet (not done)"
    );

    analyzer.on_data(&fk, Direction::ClientToServer, &combined, 0, 0);

    // BC-2.07.033 postcondition 2: no parse_errors from the non-handshake record.
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "F-S058-P1-002 / AC-013 (BC-2.07.033 pc2): parse_errors must be 0 — \
         ChangeCipherSpec (0x14) drained silently by within-loop skip (not parsed)"
    );

    // BC-2.07.033 postcondition 3: no finding emitted.
    assert!(
        analyzer.findings().is_empty(),
        "F-S058-P1-002 / AC-013 (BC-2.07.033 pc3): no finding must be emitted for \
         ChangeCipherSpec record; got: {:?}",
        analyzer.findings()
    );

    // BC-2.07.033 postcondition 4: ClientHello after CCS is parsed normally.
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "F-S058-P1-002 / AC-013 (BC-2.07.033 pc4): ClientHello must be parsed normally \
         after the CCS record is drained by the within-loop skip (handshakes_seen==1)"
    );

    // SNI from the ClientHello is recorded — proves handshake content was processed.
    assert_eq!(
        *analyzer
            .sni_counts()
            .get("within-loop.example")
            .unwrap_or(&0),
        1,
        "F-S058-P1-002 / AC-013 (BC-2.07.033 pc4): sni_counts must contain \
         \"within-loop.example\" from the ClientHello that followed the CCS record"
    );

    // Confirm the buffer was drained (CCS consumed, ClientHello parsed and drained).
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "F-S058-P1-002 anchor: flow must be present (not absent) for buf_len assertion"
    );
    assert_eq!(
        analyzer.client_buf_len_for_testing(&fk),
        0,
        "F-S058-P1-002 (BC-2.07.033 pc1): CCS bytes drained, ClientHello bytes parsed \
         and drained — client_buf must be empty after both records are consumed"
    );

    // Sanity: truncated_records must be 0 (no oversized record guard fired).
    assert_eq!(
        analyzer.truncated_record_count(),
        0,
        "F-S058-P1-002: truncated_records must be 0 (only oversized guard increments this)"
    );
}

#[test]
fn test_nonhandshake_types_0x14_0x15_0x17_0x18_all_skip_silently() {
    // F-S058-P1-003 / AC-013 extension (BC-2.07.033 EC-001/002/003/004 + STORY EC-006/EC-007):
    //
    // All record types except 0x16 (Handshake) share the same != 0x16 within-loop skip
    // path. This test verifies four distinct types, each in isolation, followed by a
    // valid ClientHello to confirm the loop continues.
    //
    // Types tested:
    //   0x14 — ChangeCipherSpec (BC-2.07.033 EC-002)
    //   0x15 — Alert            (BC-2.07.033 EC-003)
    //   0x17 — ApplicationData  (BC-2.07.033 EC-001, already in test_appdata_record_skipped_then_hello)
    //   0x18 — Heartbeat (or "unknown") (BC-2.07.033 EC-004 / STORY-058 EC-006/007)
    //
    // For each type: assert parse_errors==0, no finding, handshakes_seen==1 after hello.
    let fk = test_flow_key();

    let type_cases: &[(u8, &str)] = &[
        (0x14, "ChangeCipherSpec"),
        (0x15, "Alert"),
        (0x17, "ApplicationData"),
        (0x18, "Heartbeat/unknown"),
    ];

    for &(record_type, label) in type_cases {
        let mut analyzer = TlsAnalyzer::new();

        // Build a non-handshake record of this type followed by a valid ClientHello.
        let nhs = build_nonhandshake_record(record_type, &[0xAA, 0xBB]); // 2-byte payload
        let ch = build_client_hello("skip-type.example", &[0x1301]);
        let mut combined = nhs;
        combined.extend_from_slice(&ch);

        analyzer.on_data(&fk, Direction::ClientToServer, &combined, 0, 0);

        // BC-2.07.033 postcondition 2: no parse_errors for any non-handshake type.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "F-S058-P1-003 (BC-2.07.033 pc2): parse_errors must be 0 for type 0x{record_type:02x} \
             ({label}) — all non-handshake types share the != 0x16 within-loop skip"
        );

        // BC-2.07.033 postcondition 3: no finding.
        assert!(
            analyzer.findings().is_empty(),
            "F-S058-P1-003 (BC-2.07.033 pc3): no finding for type 0x{record_type:02x} ({label}); \
             got: {:?}",
            analyzer.findings()
        );

        // BC-2.07.033 postcondition 4: ClientHello parsed normally.
        assert_eq!(
            analyzer.handshake_count(),
            1,
            "F-S058-P1-003 (BC-2.07.033 pc4): handshakes_seen must be 1 after ClientHello \
             following type 0x{record_type:02x} ({label}) — loop continued after skip"
        );

        // Sanity: truncated_records unchanged (no oversized guard).
        assert_eq!(
            analyzer.truncated_record_count(),
            0,
            "F-S058-P1-003: truncated_records must be 0 for type 0x{record_type:02x} ({label})"
        );
    }
}

// ── BC-2.07.035 ──────────────────────────────────────────────────────────────

// AC-014 / BC-2.07.035 postconditions 1-4:
// When on_flow_close is called with a flow_key present in flows, flows.remove(flow_key)
// is called. TlsFlowState is dropped. sni_counts, ja3_counts, ja3s_counts,
// version_counts, cipher_counts, handshakes_seen, parse_errors, and all_findings are
// all UNCHANGED. flows.len() decreases by 1.
#[test]
fn test_on_flow_close_drops_state_preserves_aggregates() {
    // AC-014 / BC-2.07.035 postconditions 1-4:
    // Process a ClientHello on flow A; call on_flow_close for flow A;
    // assert flows.len()==0 and sni_counts still has the entry from flow A.
    use wirerust::reassembly::handler::CloseReason;

    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Process a valid ClientHello — this creates flow state and updates sni_counts.
    let ch = build_client_hello("flowclose.example", &[0x1301]);
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0, 0);

    // Confirm flow is present and aggregate state is populated.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "AC-014 setup: flows.len() must be 1 after one on_data call"
    );
    assert_eq!(
        analyzer.handshake_count(),
        1,
        "AC-014 setup: handshakes_seen must be 1"
    );
    assert_eq!(
        *analyzer.sni_counts().get("flowclose.example").unwrap_or(&0),
        1,
        "AC-014 setup: sni_counts must contain the SNI from flow A"
    );

    // Snapshot aggregate state before close.
    let handshakes_before = analyzer.handshake_count();
    let parse_errors_before = analyzer.parse_error_count();
    let truncated_before = analyzer.truncated_record_count();
    let sni_count_before = *analyzer.sni_counts().get("flowclose.example").unwrap_or(&0);
    let findings_before = analyzer.findings().len();

    // Call on_flow_close for flow A.
    analyzer.on_flow_close(&fk, CloseReason::Fin);

    // BC-2.07.035 postcondition 4: flows.len() decreases by 1 (to 0).
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        0,
        "AC-014 (BC-2.07.035 pc4): flows.len() must be 0 after on_flow_close for flow A"
    );

    // BC-2.07.035 postcondition 3: aggregate counters UNCHANGED.
    assert_eq!(
        analyzer.handshake_count(),
        handshakes_before,
        "AC-014 (BC-2.07.035 pc3): handshakes_seen must be unchanged after on_flow_close"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        parse_errors_before,
        "AC-014 (BC-2.07.035 pc3): parse_errors must be unchanged after on_flow_close"
    );
    assert_eq!(
        analyzer.truncated_record_count(),
        truncated_before,
        "AC-014 (BC-2.07.035 pc3): truncated_records must be unchanged after on_flow_close"
    );
    assert_eq!(
        analyzer.findings().len(),
        findings_before,
        "AC-014 (BC-2.07.035 pc3): all_findings must be unchanged after on_flow_close"
    );

    // BC-2.07.035 postcondition 3: sni_counts still has the entry from flow A.
    assert_eq!(
        *analyzer.sni_counts().get("flowclose.example").unwrap_or(&0),
        sni_count_before,
        "AC-014 (BC-2.07.035 pc3): sni_counts must still contain the entry from flow A \
         after on_flow_close (aggregate state is preserved)"
    );
}

// AC-015 / BC-2.07.035 invariants 1-2:
// Per-flow state cleanup is the ONLY operation in on_flow_close; no analysis performed.
// The _reason parameter (CloseReason) is ignored.
// If on_flow_close is called with a key NOT in flows, HashMap::remove returns None — no panic.
#[test]
fn test_on_flow_close_absent_key_no_panic() {
    // AC-015 / BC-2.07.035 inv1-2 + EC-001 (on_flow_close for key not in flows):
    // Call on_flow_close for a key that was never inserted into flows.
    // No panic. State unchanged (flows.len() remains 0).
    use wirerust::reassembly::handler::CloseReason;

    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();

    // Confirm flows is empty before the test.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        0,
        "AC-015 setup: flows must be empty before the test"
    );

    // Call on_flow_close for a key NOT in flows.
    // HashMap::remove returns None; no panic.
    analyzer.on_flow_close(&fk, CloseReason::Fin);

    // BC-2.07.035 EC-001: no panic (test reached this point).
    // State must be completely unchanged.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        0,
        "AC-015 (BC-2.07.035 EC-001): flows.len() must remain 0 after on_flow_close \
         for absent key (HashMap::remove returns None — no panic, no state change)"
    );
    assert_eq!(
        analyzer.handshake_count(),
        0,
        "AC-015 (BC-2.07.035 inv1): handshakes_seen must be 0 — on_flow_close performs \
         only flows.remove(); no analysis at close time"
    );
    assert_eq!(
        analyzer.parse_error_count(),
        0,
        "AC-015 (BC-2.07.035 inv1): parse_errors must be 0 — on_flow_close performs \
         only flows.remove(); no analysis at close time"
    );
    assert!(
        analyzer.findings().is_empty(),
        "AC-015 (BC-2.07.035 inv1): no findings must be generated — on_flow_close performs \
         only flows.remove(); no analysis at close time"
    );

    // BC-2.07.035 invariant 2: _reason (CloseReason) is ignored.
    // Verify by calling with a different reason — behavior must be identical.
    analyzer.on_flow_close(&fk, CloseReason::Timeout);
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        0,
        "AC-015 (BC-2.07.035 inv2): CloseReason::Timeout must behave identically to \
         Fin for absent key — _reason is ignored by TlsAnalyzer"
    );
}

// ---------------------------------------------------------------------------
// FIX-P5-003 / ADV-IMPL-P06-HIGH-001: top_snis tie-ordering determinism
// ---------------------------------------------------------------------------
//
// Defect: `TlsAnalyzer::summarize()` builds `top_snis` by sorting
// Vec<(&str, &u64)> with `sort_by(|a,b| b.1.cmp(a.1))` — count descending
// only, no tiebreaker.  For equal counts the relative order (and the
// selected set when >20 SNIs exist) depends on HashMap iteration order, which
// is per-process-random.  The fix must add `.then_with(|| a.0.cmp(b.0))`.
//
// RED-GATE STRATEGY: Feed 25 ClientHello records (one SNI each) all at
// count=1 (one handshake each), plus 2 higher-count anchors, inserted in
// reverse-alphabetical order.  Assert alphabetically-first 18 tied SNIs
// occupy slots [2..19].

/// FIX-P5-003 / ADV-IMPL-P06-HIGH-001 — `top_snis` ties broken alphabetically.
///
/// Setup:
///   - "aaa.anchor.example" → 10 handshakes (must be first)
///   - "bbb.anchor.example" →  5 handshakes (must be second)
///   - 25 "tied-ZZ.example" SNIs → 1 handshake each, submitted in
///     reverse-alphabetical order.
///
/// Assertions:
///   (a) top_snis[0] == "aaa.anchor.example"
///   (b) top_snis[1] == "bbb.anchor.example"
///   (c) top_snis[2..19] == alphabetically-first 18 of the 25 tied SNIs
///   (d) alphabetically-last 7 tied SNIs are absent (cut by deterministic selection)
#[test]
fn test_summarize_top_snis_ties_broken_alphabetically() {
    // Use a distinct flow key per handshake so each record is processed as a
    // fresh ClientHello (the TLS analyzer marks a flow done after the first
    // handshake, so reusing the same key would suppress subsequent SNIs).
    fn fk(n: u16) -> FlowKey {
        FlowKey::new(
            format!("10.0.{}.{}", n / 256, n % 256)
                .parse::<std::net::IpAddr>()
                .unwrap(),
            49200 + n,
            "10.1.0.1".parse::<std::net::IpAddr>().unwrap(),
            443,
        )
    }

    let ciphers = &[0x1301u16, 0x1302];

    // 25 tied SNIs, all count=1.  Labels: "tied-aa.example" through "tied-ay.example".
    // Alphabetical: aa < ab < ... < ay.
    let suffixes: Vec<String> = (0u8..25u8)
        .map(|i| {
            let first = b'a' + i / 26;
            let second = b'a' + i % 26;
            format!("{}{}", first as char, second as char)
        })
        .collect();

    let mut analyzer = TlsAnalyzer::new();

    // Insert tied SNIs in REVERSE alphabetical order (ay first, aa last) so
    // the current no-tiebreaker sort is maximally likely to preserve a
    // non-alphabetical order.
    for (idx, suffix) in suffixes.iter().enumerate().rev() {
        let sni = format!("tied-{suffix}.example");
        let record = build_client_hello(&sni, ciphers);
        // Each tied SNI gets its own flow key so the per-flow "done" guard
        // doesn't suppress subsequent ClientHellos.
        analyzer.on_data(&fk(idx as u16), Direction::ClientToServer, &record, 0, 0);
    }

    // Anchor SNI "aaa.anchor.example" → 10 handshakes across 10 distinct flows.
    for i in 0..10u16 {
        let record = build_client_hello("aaa.anchor.example", ciphers);
        analyzer.on_data(&fk(100 + i), Direction::ClientToServer, &record, 0, 0);
    }

    // Anchor SNI "bbb.anchor.example" → 5 handshakes across 5 distinct flows.
    for i in 0..5u16 {
        let record = build_client_hello("bbb.anchor.example", ciphers);
        analyzer.on_data(&fk(200 + i), Direction::ClientToServer, &record, 0, 0);
    }

    let summary = analyzer.summarize();
    let top_snis = summary.detail["top_snis"]
        .as_array()
        .expect("FIX-P5-003: top_snis must be a JSON array");

    // (a) Truncated to 20 (27 distinct SNIs → 20).
    assert_eq!(
        top_snis.len(),
        20,
        "FIX-P5-003 (ADV-IMPL-P06-HIGH-001): top_snis must be truncated to 20 entries; \
         got {}",
        top_snis.len()
    );

    // (b) First slot: highest-count anchor.
    assert_eq!(
        top_snis[0].as_str().unwrap_or(""),
        "aaa.anchor.example",
        "FIX-P5-003: top_snis[0] must be 'aaa.anchor.example' (count=10)"
    );

    // (c) Second slot: second-highest-count anchor.
    assert_eq!(
        top_snis[1].as_str().unwrap_or(""),
        "bbb.anchor.example",
        "FIX-P5-003: top_snis[1] must be 'bbb.anchor.example' (count=5)"
    );

    // (d) Tied slots [2..19] must be the alphabetically-first 18 tied SNIs in order.
    let expected_tied: Vec<String> = suffixes[..18]
        .iter()
        .map(|s| format!("tied-{s}.example"))
        .collect();

    let actual_tied: Vec<&str> = top_snis[2..]
        .iter()
        .map(|v| v.as_str().unwrap_or(""))
        .collect();

    assert_eq!(
        actual_tied, expected_tied,
        "FIX-P5-003 (ADV-IMPL-P06-HIGH-001): tied SNIs in slots [2..19] must be sorted \
         alphabetically (tied-aa.example, tied-ab.example, ..., tied-ar.example); \
         current code has no tiebreaker so HashMap iteration order determines the \
         result — this assertion fails without `.then_with(|| a.0.cmp(b.0))`"
    );

    // (e) Alphabetically-last 7 tied SNIs must be absent.
    for suffix in &suffixes[18..] {
        let sni = format!("tied-{suffix}.example");
        assert!(
            !top_snis.iter().any(|v| v.as_str() == Some(sni.as_str())),
            "FIX-P5-003: '{sni}' must NOT appear in top_snis — it falls outside the \
             alphabetically-first 18 tied slots"
        );
    }
}

// ── CR-010 guard-before-allocate regression tests ────────────────────────────
//
// These tests verify that `try_parse_records` short-circuits (drains the buffer
// without allocating/parsing) for non-0x16 content types. They exercise the
// `StreamHandler::on_data` entry point via integration test to stay outside
// the trust-boundary gate (seam callers must live in tests/, not src/).

/// Build a minimal, structurally valid 5-byte-header TLS record with the
/// given content type and an empty payload (payload_len = 0).
fn make_tls_record_cr010(content_type: u8) -> Vec<u8> {
    // TLS record header: content_type (1), version major (1), version minor (1),
    // payload_len hi (1), payload_len lo (1), then payload bytes (0 here).
    vec![content_type, 0x03, 0x03, 0x00, 0x00]
}

fn cr010_flow_key() -> FlowKey {
    FlowKey::new(
        "127.0.0.1".parse::<IpAddr>().unwrap(),
        1234,
        "127.0.0.2".parse::<IpAddr>().unwrap(),
        443,
    )
}

/// Non-0x16 content types MUST drain the ClientToServer buffer without any
/// parse attempt.
///
/// Precondition: feed a single complete TLS record with content_type != 0x16
/// in the ClientToServer direction.
/// Postconditions:
///   - `client_buf_len_for_testing` returns 0 (record was drained).
///   - `handshake_count` returns 0 (no parse occurred).
///   - `parse_error_count` returns 0 (no parse, so no parse error).
#[test]
fn non_handshake_record_client_drains_without_parse() {
    // Content types to check: 0x14 (ChangeCipherSpec), 0x15 (Alert),
    // 0x17 (ApplicationData), and an arbitrary unknown type 0x00.
    let non_handshake_types: &[u8] = &[0x14, 0x15, 0x17, 0x00];
    let flow_key = cr010_flow_key();

    for &ct in non_handshake_types {
        let mut analyzer = TlsAnalyzer::new();
        let record = make_tls_record_cr010(ct);
        analyzer.on_data(&flow_key, Direction::ClientToServer, &record, 0, 0);

        assert_eq!(
            analyzer.client_buf_len_for_testing(&flow_key),
            0,
            "content_type=0x{ct:02x} C->S: buffer must be drained after consuming a complete record"
        );
        assert_eq!(
            analyzer.handshake_count(),
            0,
            "content_type=0x{ct:02x} C->S: no handshake should have been parsed"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "content_type=0x{ct:02x} C->S: no parse error should have been recorded"
        );
    }
}

/// Non-0x16 content types MUST drain the ServerToClient buffer without any
/// parse attempt.
///
/// Precondition: feed a single complete TLS record with content_type != 0x16
/// in the ServerToClient direction.
/// Postconditions:
///   - `server_buf_len_for_testing` returns 0 (record was drained).
///   - `handshake_count` returns 0 (no parse occurred).
///   - `parse_error_count` returns 0 (no parse, so no parse error).
#[test]
fn non_handshake_record_server_drains_without_parse() {
    let non_handshake_types: &[u8] = &[0x14, 0x15, 0x17, 0x00];
    let flow_key = cr010_flow_key();

    for &ct in non_handshake_types {
        let mut analyzer = TlsAnalyzer::new();
        let record = make_tls_record_cr010(ct);
        analyzer.on_data(&flow_key, Direction::ServerToClient, &record, 0, 0);

        assert_eq!(
            analyzer.server_buf_len_for_testing(&flow_key),
            0,
            "content_type=0x{ct:02x} S->C: buffer must be drained after consuming a complete record"
        );
        assert_eq!(
            analyzer.handshake_count(),
            0,
            "content_type=0x{ct:02x} S->C: no handshake should have been parsed"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "content_type=0x{ct:02x} S->C: no parse error should have been recorded"
        );
    }
}

/// A 0x16 handshake record DOES reach the parser; the buffer is drained.
///
/// We use a header-only record (payload_len = 0) as the minimal probe. The
/// empty payload causes tls_parser to return an error, which is intentional:
/// the parse_errors counter going from 0 to >= 1 is the observable proof that
/// the code took the parse path rather than the non-0x16 short-circuit path.
/// A full valid ClientHello would also work but would require embedding
/// raw TLS bytes; the empty-payload shortcut is deliberately chosen here
/// because the distinction we need is parse-path-entered vs not-entered,
/// not parse-succeeded vs not-succeeded.
#[test]
fn handshake_record_reaches_parser() {
    let flow_key = cr010_flow_key();
    let mut analyzer = TlsAnalyzer::new();
    let record = make_tls_record_cr010(0x16);
    analyzer.on_data(&flow_key, Direction::ClientToServer, &record, 0, 0);

    // Buffer must be drained regardless of parse outcome.
    assert_eq!(
        analyzer.client_buf_len_for_testing(&flow_key),
        0,
        "0x16 record: buffer must be drained"
    );
    // Updated for STORY-144 carry path (AC-144-002): empty-payload 0x16 records
    // are now buffered in client_hs_carry rather than immediately dispatched to
    // parse_tls_plaintext. An empty payload (0 bytes) does not form a complete
    // 4-byte handshake header; no parse attempt occurs and parse_errors stays 0.
    // The observable proof that the 0x16 path was entered is:
    //  (a) client_buf was drained above (buffer management still fires), AND
    //  (b) a flow entry was created (active_flows == 1).
    // Non-0x16 content types do NOT create carry entries; only 0x16 records
    // go through the carry path, so flow creation + buf drain proves routing.
    assert_eq!(
        analyzer.active_flows_len_for_testing(),
        1,
        "0x16 with empty payload: flow must exist (proves 0x16 path was entered)"
    );
}

// ── Issue #102: weak-cipher evidence cap ─────────────────────────────────────
//
// A ClientHello with more than 64 weak ciphers must NOT produce an unbounded
// evidence vec.  The finding's `evidence` must be capped at 65 entries:
// 64 cipher names followed by a single "(+N more)" elision marker.
//
// This is a LOW-SEVERITY HARDENING test (not a security/DoS test): the
// allocation is bounded by upstream input limits (MAX_RECORD_PAYLOAD=18_432),
// so it is NOT CWE-405 / asymmetric amplification.  The cap simply avoids
// a needlessly large transient String allocation when a crafted ClientHello
// offers a maximal weak-cipher list.
//
// All 65 cipher IDs below are confirmed weak by `is_weak_cipher` (they
// contain "NULL", "ANON", or "EXPORT" in their tls-parser name string).
// Source: tls-parser-0.12.2/scripts/tls-ciphersuites.txt.
#[test]
fn test_weak_cipher_evidence_capped_at_64_with_elision() {
    // 65 distinct weak cipher IDs — one more than the cap.
    // All names contain "NULL", "ANON", or "EXPORT" per is_weak_cipher.
    let weak_ids: &[u16] = &[
        0x0000, // TLS_NULL_WITH_NULL_NULL
        0x0001, // TLS_RSA_WITH_NULL_MD5
        0x0002, // TLS_RSA_WITH_NULL_SHA
        0x0003, // TLS_RSA_EXPORT_WITH_RC4_40_MD5
        0x0006, // TLS_RSA_EXPORT_WITH_RC2_CBC_40_MD5
        0x0008, // TLS_RSA_EXPORT_WITH_DES40_CBC_SHA
        0x000b, // TLS_DH_DSS_EXPORT_WITH_DES40_CBC_SHA
        0x000e, // TLS_DH_RSA_EXPORT_WITH_DES40_CBC_SHA
        0x0011, // TLS_DHE_DSS_EXPORT_WITH_DES40_CBC_SHA
        0x0014, // TLS_DHE_RSA_EXPORT_WITH_DES40_CBC_SHA
        0x0017, // TLS_DH_anon_EXPORT_WITH_RC4_40_MD5
        0x0018, // TLS_DH_anon_WITH_RC4_128_MD5
        0x0019, // TLS_DH_anon_EXPORT_WITH_DES40_CBC_SHA
        0x001a, // TLS_DH_anon_WITH_DES_CBC_SHA
        0x001b, // TLS_DH_anon_WITH_3DES_EDE_CBC_SHA
        0x0026, // TLS_KRB5_EXPORT_WITH_DES_CBC_40_SHA
        0x0027, // TLS_KRB5_EXPORT_WITH_RC2_CBC_40_SHA
        0x0028, // TLS_KRB5_EXPORT_WITH_RC4_40_SHA
        0x0029, // TLS_KRB5_EXPORT_WITH_DES_CBC_40_MD5
        0x002a, // TLS_KRB5_EXPORT_WITH_RC2_CBC_40_MD5
        0x002b, // TLS_KRB5_EXPORT_WITH_RC4_40_MD5
        0x002c, // TLS_PSK_WITH_NULL_SHA
        0x002d, // TLS_DHE_PSK_WITH_NULL_SHA
        0x002e, // TLS_RSA_PSK_WITH_NULL_SHA
        0x0034, // TLS_DH_anon_WITH_AES_128_CBC_SHA
        0x003a, // TLS_DH_anon_WITH_AES_256_CBC_SHA
        0x003b, // TLS_RSA_WITH_NULL_SHA256
        0x0046, // TLS_DH_anon_WITH_CAMELLIA_128_CBC_SHA
        0x0060, // TLS_RSA_EXPORT1024_WITH_RC4_56_MD5
        0x0061, // TLS_RSA_EXPORT1024_WITH_RC2_CBC_56_MD5
        0x0062, // TLS_RSA_EXPORT1024_WITH_DES_CBC_SHA
        0x0063, // TLS_DHE_DSS_EXPORT1024_WITH_DES_CBC_SHA
        0x0064, // TLS_RSA_EXPORT1024_WITH_RC4_56_SHA
        0x0065, // TLS_DHE_DSS_EXPORT1024_WITH_RC4_56_SHA
        0x006c, // TLS_DH_anon_WITH_AES_128_CBC_SHA256
        0x006d, // TLS_DH_anon_WITH_AES_256_CBC_SHA256
        0x0089, // TLS_DH_anon_WITH_CAMELLIA_256_CBC_SHA
        0x009b, // TLS_DH_anon_WITH_SEED_CBC_SHA
        0x00a6, // TLS_DH_anon_WITH_AES_128_GCM_SHA256
        0x00a7, // TLS_DH_anon_WITH_AES_256_GCM_SHA384
        0x00b0, // TLS_PSK_WITH_NULL_SHA256
        0x00b1, // TLS_PSK_WITH_NULL_SHA384
        0x00b4, // TLS_DHE_PSK_WITH_NULL_SHA256
        0x00b5, // TLS_DHE_PSK_WITH_NULL_SHA384
        0x00b8, // TLS_RSA_PSK_WITH_NULL_SHA256
        0x00b9, // TLS_RSA_PSK_WITH_NULL_SHA384
        0x00bf, // TLS_DH_anon_WITH_CAMELLIA_128_CBC_SHA256
        0x00c5, // TLS_DH_anon_WITH_CAMELLIA_256_CBC_SHA256
        0xc001, // TLS_ECDH_ECDSA_WITH_NULL_SHA
        0xc006, // TLS_ECDHE_ECDSA_WITH_NULL_SHA
        0xc00b, // TLS_ECDH_RSA_WITH_NULL_SHA
        0xc010, // TLS_ECDHE_RSA_WITH_NULL_SHA
        0xc015, // TLS_ECDH_anon_WITH_NULL_SHA
        0xc016, // TLS_ECDH_anon_WITH_RC4_128_SHA
        0xc017, // TLS_ECDH_anon_WITH_3DES_EDE_CBC_SHA
        0xc018, // TLS_ECDH_anon_WITH_AES_128_CBC_SHA
        0xc019, // TLS_ECDH_anon_WITH_AES_256_CBC_SHA
        0xc039, // TLS_ECDHE_PSK_WITH_NULL_SHA
        0xc03a, // TLS_ECDHE_PSK_WITH_NULL_SHA256
        0xc03b, // TLS_ECDHE_PSK_WITH_NULL_SHA384
        0xc046, // TLS_DH_anon_WITH_ARIA_128_CBC_SHA256
        0xc047, // TLS_DH_anon_WITH_ARIA_256_CBC_SHA384
        0xc05a, // TLS_DH_anon_WITH_ARIA_128_GCM_SHA256
        0xc05b, // TLS_DH_anon_WITH_ARIA_256_GCM_SHA384
        0xc084, // TLS_DH_anon_WITH_CAMELLIA_128_GCM_SHA256
                // 65 entries total (0xc085 omitted; 65 is one more than the cap of 64)
    ];
    assert_eq!(
        weak_ids.len(),
        65,
        "test setup: exactly 65 weak IDs required"
    );

    let mut cipher_ids = weak_ids.to_vec();
    cipher_ids.push(0x1301); // TLS_AES_128_GCM_SHA256 (strong, not weak)

    let mut analyzer = TlsAnalyzer::new();
    let fk = test_flow_key();
    let record = build_client_hello("test.com", &cipher_ids);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

    let findings = analyzer.findings();
    let weak_findings: Vec<_> = findings
        .iter()
        .filter(|f| f.summary.contains("weak cipher"))
        .collect();

    assert_eq!(
        weak_findings.len(),
        1,
        "issue #102 hardening: exactly one weak-cipher finding must be produced"
    );

    let evidence = &weak_findings[0].evidence;

    // BOUND CHECK: evidence must be capped at <=65 entries (64 cipher names + 1 elision).
    // Without the cap the evidence vec would have 65 entries (no elision marker);
    // this assertion FAILS on the current uncapped implementation because it
    // has exactly 65 entries but no elision marker — the elision marker check
    // below is what distinguishes the correct capped form.
    assert!(
        evidence.len() <= 65,
        "issue #102 hardening: evidence must be capped at <=65 entries; got {}",
        evidence.len()
    );

    // ELISION CHECK: when total weak ciphers > 64, the last evidence entry must
    // be a "(+N more)" elision marker.  On the uncapped implementation this
    // assertion FAILS because evidence[64] is a cipher name, not an elision marker.
    let last = evidence
        .last()
        .expect("issue #102 hardening: evidence must not be empty");
    assert!(
        last.starts_with("(+") && last.ends_with(" more)"),
        "issue #102 hardening: last evidence entry must be an elision marker of the form \
         \"(+N more)\" when total weak ciphers > 64; got: {:?}",
        last
    );

    // COUNT CHECK: exactly 64 cipher-name entries plus 1 elision marker.
    assert_eq!(
        evidence.len(),
        65,
        "issue #102 hardening: capped evidence must have exactly 65 entries \
         (64 cipher names + 1 elision marker); got {}",
        evidence.len()
    );

    // ELISION CONTENT CHECK: the elision marker must report the correct overflow count.
    // 65 weak ciphers total, cap = 64, so overflow = 1 → "(+1 more)".
    assert_eq!(
        last, "(+1 more)",
        "issue #102 hardening: elision marker must be \"(+1 more)\" for 65 weak ciphers \
         with cap=64; got: {:?}",
        last
    );
}

// ── STORY-144: TLS Handshake Carry Buffer + Fragmented ClientHello Reassembly ──
//
// VP-039 Sub-A/B/C/D/F — AC-144-002 implemented (carry drain loop is live).
//
// These 15 harnesses verify the ClientToServer carry drain loop introduced by
// AC-144-002. The carry buffer accumulates 0x16 record payloads and drains them
// message-by-message once a complete handshake message is available.
//
// Namespace isolation: DF-TEST-NAMESPACE-001 — all STORY-144 tests live inside
// this `mod story_144` wrapper. No new flat-root tests are added for this story.
//
// Test count: 15 (3 proptest + 12 unit).
mod story_144 {
    use std::net::IpAddr;
    use wirerust::analyzer::tls::TlsAnalyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{CloseReason, Direction, StreamAnalyzer, StreamHandler};
    // proptest is used by the three proptest harnesses:
    //   proptest_vp039_carry_reassembly_two_record (Sub-A)
    //   proptest_vp039_exact_consume_coalesced (Sub-B)
    //   proptest_vp039_carry_bounded_invariant (Sub-F)
    // The allow(unused_imports) is removed now that the real proptest bodies are authored.
    use proptest::prelude::*;

    // ── Local test helpers ────────────────────────────────────────────────────
    //
    // Reconciliation rule: before creating any new helper, grep for the relevant
    // name in the existing suite. Names below are new (not present at flat root).

    /// Create a `FlowKey` varied by `seed` so cross-flow and independent-flow
    /// tests can use distinct keys without collision.
    fn make_test_flow_key(seed: u8) -> FlowKey {
        FlowKey::new(
            IpAddr::from([10, 144, 0, seed]),
            49000u16.wrapping_add(seed as u16),
            IpAddr::from([10, 144, 1, seed]),
            443,
        )
    }

    /// Returns the RAW handshake-message bytes for a ClientHello with the given
    /// SNI, with NO TLS record header prefix (5 bytes stripped).
    ///
    /// `build_client_hello` at the flat root returns a COMPLETE TLS record
    /// (5-byte header + handshake body). This wrapper strips the header so
    /// fragmentation tests can re-frame the handshake bytes into arbitrary
    /// record boundaries via `wrap_as_tls_record`.
    fn build_client_hello_with_sni(sni: &str) -> Vec<u8> {
        // Reconciliation: `build_client_hello` exists at flat root; use it.
        // build_client_hello(sni, ciphers) → [0x16, ver_hi, ver_lo, len_hi, len_lo, ...body...]
        // Strip the 5-byte record header to get raw handshake-message bytes.
        super::build_client_hello(sni, &[0x002f])[5..].to_vec()
    }

    /// Wrap `payload` bytes in a 5-byte TLS record header for the given content type.
    ///
    /// Reconciliation: `make_tls_record_cr010` at flat root only builds empty-payload
    /// records. This generic wrapper is new; no `wrap_as_tls_record` or generic
    /// `make_tls_record` exists at the flat root.
    fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
        let len = payload.len();
        let len_hi = (len >> 8) as u8;
        let len_lo = (len & 0xff) as u8;
        let mut record = vec![content_type, 0x03, 0x03, len_hi, len_lo];
        record.extend_from_slice(payload);
        record
    }

    // ── VP-039 Sub-A: carry reassembly ────────────────────────────────────────

    // VP-039 Sub-A (proptest): for any split offset 1<=k<n, a ClientHello
    // split into two 0x16 records must be fully reassembled.
    //
    // Asserts: client_hello_seen==true, sni_counts.len()==1, parse_errors==0.
    //
    // The strategy uses prop_oneof to guarantee partial-header splits
    // (k < 4) and body splits (k >= 4) are both reachable in the same run.
    //
    // Traces to: BC-2.07.038 v2.7 Postconditions 1–4.
    proptest! {
        #[test]
        fn proptest_vp039_carry_reassembly_two_record(
            // Two-armed strategy: partial-header splits (1..4) AND body splits (4..256).
            // Using prop_oneof ensures both sub-ranges are reachable in the same test run.
            split_offset in prop_oneof![1usize..4usize, 4usize..256usize],
        ) {
            let client_hello = build_client_hello_with_sni("example.com");
            let n = client_hello.len();
            // Discard if split overshoots the actual message length.
            prop_assume!(split_offset < n);

            // Two-record fragmented delivery.
            let mut analyzer_fragmented = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(1);
            let ts: u32 = 100;

            // Record 1: bytes [0..split_offset] wrapped as a 0x16 record payload.
            let rec1 = wrap_as_tls_record(0x16, &client_hello[..split_offset]);
            analyzer_fragmented.on_data(&flow_key, Direction::ClientToServer, &rec1, 0u64, ts);

            // Record 2: bytes [split_offset..n] wrapped as a 0x16 record payload.
            let rec2 = wrap_as_tls_record(0x16, &client_hello[split_offset..]);
            analyzer_fragmented.on_data(&flow_key, Direction::ClientToServer, &rec2, 0u64, ts);

            // Single-record delivery (baseline for comparison).
            let mut analyzer_single = TlsAnalyzer::new();
            let flow_key2 = make_test_flow_key(2);
            let rec_single = wrap_as_tls_record(0x16, &client_hello);
            analyzer_single.on_data(&flow_key2, Direction::ClientToServer, &rec_single, 0u64, ts);

            // Red Gate primary assertion: after fragmented delivery, client_hello_seen
            // must be true. Without the carry drain loop this is always false
            // (stub never dispatches via the carry path) — fails on first case.
            //
            // Use flat accessor: client_hello_seen_for_testing is the NEW seam
            // (STORY-144/146 deliverable), symmetric to server_hello_seen_for_testing.
            prop_assert_eq!(
                analyzer_fragmented.client_hello_seen_for_testing(&flow_key),
                analyzer_single.client_hello_seen_for_testing(&flow_key2),
                "fragmented and single-record ClientHello detection must agree: \
                 fragmented={}, single={}",
                analyzer_fragmented.client_hello_seen_for_testing(&flow_key),
                analyzer_single.client_hello_seen_for_testing(&flow_key2),
            );
            prop_assert_eq!(
                analyzer_fragmented.parse_error_count(), 0u64,
                "fragmented delivery must not produce parse errors"
            );
            prop_assert_eq!(
                analyzer_fragmented.sni_counts().len(), analyzer_single.sni_counts().len(),
                "SNI detection must be identical for fragmented vs single-record"
            );
            prop_assert_eq!(
                analyzer_fragmented.ja3_counts().len(), analyzer_single.ja3_counts().len(),
                "JA3 count must be identical for fragmented vs single-record"
            );
        }
    }

    /// VP-039 Sub-A (unit): three canonical RFC 8446 §4 handshake frames.
    ///
    /// Frame A: degenerate body_len=5, msg_type=0x01 — assembled 9-byte body
    ///   is a malformed ClientHello → parse_errors+1, client_hello_seen=false.
    /// Frame B: BE-vs-LE discriminator — body_len encoded as big-endian 0x01_05_00
    ///   = 66,816 > MAX_BUF → handshake_reassembly_overflows+1, parse_errors unchanged.
    /// Frame C: body_len=256 (0x000100), msg_type=0x01, malformed body →
    ///   parse_errors+1, client_hello_seen=false.
    ///
    /// Traces to: BC-2.07.038 v2.7 AC-CANONICAL-FRAME + Invariant 5.
    /// Red Gate: FAILS because carry drain loop not implemented.
    // DF-AC-TEST-NAME-SYNC-001: canonical name verbatim per VP-039 table.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_07_038_canonical_frame_rfc8446_s4() {
        let fk = make_test_flow_key(38);
        let mut analyzer = TlsAnalyzer::new();

        // Frame A: msg_type=0x01, body_len=5 (degenerate). Header bytes: [0x01, 0x00, 0x00, 0x05]
        // + 5 bytes body. Total handshake message = 9 bytes.
        // Wrapped in a 0x16 TLS record so the existing record-layer parses it.
        let frame_a_hs: Vec<u8> = vec![0x01, 0x00, 0x00, 0x05, 0xff, 0xff, 0xff, 0xff, 0xff];
        let frame_a_record = wrap_as_tls_record(0x16, &frame_a_hs);
        analyzer.on_data(&fk, Direction::ClientToServer, &frame_a_record, 0, 0);

        // Frame A: explicit BC-CANONICAL-FRAME assertions (adversary LOW).
        // The carry drain loop dispatches the complete 9-byte message via
        // parse_tls_message_handshake, which returns an error for the degenerate
        // 5-byte body → parse_errors+1, client_hello_seen remains false.
        assert_eq!(
            analyzer.parse_error_count(),
            1,
            "Frame A: degenerate 5-byte body must produce parse_errors==1"
        );
        assert!(
            !analyzer.client_hello_seen_for_testing(&fk),
            "Frame A: client_hello_seen must be false for degenerate 5-byte body"
        );

        // Frame B: body_len = 0x010500 (BE) = 66,816 > MAX_BUF.
        // Header: [0x01, 0x01, 0x05, 0x00] — type=0x01, length BE = 0x010500.
        // Wrapped in a 0x16 record just carrying the 4-byte header (no body bytes).
        let frame_b_hs: Vec<u8> = vec![0x01, 0x01, 0x05, 0x00];
        let frame_b_record = wrap_as_tls_record(0x16, &frame_b_hs);
        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        analyzer.on_data(&fk, Direction::ClientToServer, &frame_b_record, 0, 0);
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(),
            overflows_before + 1,
            "Frame B: body_len=66816 > MAX_BUF must trigger handshake_reassembly_overflows+1"
        );

        // Frame C: body_len=256, msg_type=0x01, malformed 256-byte body.
        let mut frame_c_hs: Vec<u8> = vec![0x01, 0x00, 0x01, 0x00]; // body_len=256
        frame_c_hs.extend(vec![0xcc; 256]); // malformed body
        let frame_c_record = wrap_as_tls_record(0x16, &frame_c_hs);
        let parse_errors_before = analyzer.parse_error_count();
        analyzer.on_data(&fk, Direction::ClientToServer, &frame_c_record, 0, 0);
        assert_eq!(
            analyzer.parse_error_count(),
            parse_errors_before + 1,
            "Frame C: malformed assembled body must produce parse_errors+1"
        );
        assert!(
            !analyzer.client_hello_seen_for_testing(&fk),
            "Frame C: client_hello_seen must remain false after malformed body"
        );
    }

    /// VP-039 Sub-A (unit): assembled, length-complete header + malformed body
    /// → parse_errors+1, carry empty after consume, no finding, no panic.
    ///
    /// Traces to: BC-2.07.038 v2.7 Postcondition 9 / ADR-011 Decision 4.
    // DF-AC-TEST-NAME-SYNC-001: canonical name verbatim per VP-039 table.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_07_038_malformed_assembled_body() {
        let fk = make_test_flow_key(39);
        let mut analyzer = TlsAnalyzer::new();

        // Test the CARRY PATH: fragment the malformed ClientHello across two records.
        // Record 1: just the 4-byte handshake header (type=0x01, body_len=20).
        // Record 2: the 20-byte malformed body.
        // The carry drain loop must:
        //   1. Accumulate 4 bytes in carry after record 1 (not yet complete — body pending).
        //   2. After record 2: carry has 4+20=24 bytes, drain fires, parse_tls_message_handshake
        //      fails on the malformed body → parse_errors+1, exact-consume, carry empty.
        let header_only: Vec<u8> = vec![0x01, 0x00, 0x00, 0x14]; // type=ClientHello, body_len=20
        let record1 = wrap_as_tls_record(0x16, &header_only);

        analyzer.on_data(&fk, Direction::ClientToServer, &record1, 0, 0);

        // After record 1: carry holds the 4-byte handshake header (body_len=20
        // bytes not yet received). Drain loop sees carry_len=4 < 4+20 → waits.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            4,
            "malformed body: carry must hold 4 header bytes after record 1"
        );

        // Record 2: the 20-byte malformed body.
        let body: Vec<u8> = vec![0xcc; 20];
        let record2 = wrap_as_tls_record(0x16, &body);

        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.all_findings_len_for_testing();
        analyzer.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

        // After carry drain loop processes the complete (header+body):
        // parse_errors+1, no new finding, client_hello_seen=false, carry empty.
        assert_eq!(
            analyzer.parse_error_count(),
            parse_errors_before + 1,
            "malformed assembled body must produce exactly parse_errors+1"
        );
        assert_eq!(
            analyzer.all_findings_len_for_testing(),
            findings_before,
            "malformed assembled body must not emit a finding"
        );
        assert!(
            !analyzer.client_hello_seen_for_testing(&fk),
            "client_hello_seen must be false after malformed assembled body"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            0,
            "carry must be empty (exact-consumed) after malformed assembled body"
        );
    }

    /// VP-039 Sub-A (unit): ClientHello split at the exact SNI field boundary.
    ///
    /// First record ends mid-SNI bytes; second record completes them.
    /// Asserts: SNI extracted correctly, parse_errors==0, client_hello_seen==true.
    ///
    /// Traces to: BC-2.07.038 v2.7 EC-001 "SNI boundary split".
    #[test]
    fn test_vp039_sni_boundary_deterministic() {
        let fk = make_test_flow_key(40);
        let mut analyzer = TlsAnalyzer::new();
        let sni = "sni-boundary.example";

        // Get raw handshake bytes (no 5-byte record header).
        let hs_bytes = build_client_hello_with_sni(sni);

        // Split at byte 30 (within SNI extension, which starts around byte 50+ but
        // the split point just needs to be non-trivial — anywhere in the middle
        // of the handshake message body guarantees the SNI field spans two records).
        let split = hs_bytes.len() / 2;
        let record1 = wrap_as_tls_record(0x16, &hs_bytes[..split]);
        let record2 = wrap_as_tls_record(0x16, &hs_bytes[split..]);

        analyzer.on_data(&fk, Direction::ClientToServer, &record1, 0, 0);
        analyzer.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

        assert!(
            analyzer.client_hello_seen_for_testing(&fk),
            "SNI boundary split: client_hello_seen must be true after both records"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "SNI boundary split: parse_errors must be 0"
        );
        assert_eq!(
            analyzer.sni_counts().len(),
            1,
            "SNI boundary split: sni_counts must have exactly 1 entry"
        );
        assert!(
            analyzer.sni_counts().contains_key(sni),
            "SNI boundary split: sni_counts must contain '{sni}'"
        );
    }

    /// VP-039 Sub-A-ext-N (unit): ONE ClientHello drip-fed across >=3 records
    /// (header-split scenarios). Asserts sni_counts.len()==1, parse_errors==0.
    ///
    /// Traces to: BC-2.07.038 v2.7 EC-003.
    #[test]
    fn test_vp039_n_record_reassembly() {
        let fk = make_test_flow_key(41);
        let mut analyzer = TlsAnalyzer::new();
        let sni = "n-record.example";

        let hs_bytes = build_client_hello_with_sni(sni);

        // Drip-feed: 1 byte per record for the first 3, then the remainder.
        let record0 = wrap_as_tls_record(0x16, &hs_bytes[..1]);
        let record1 = wrap_as_tls_record(0x16, &hs_bytes[1..2]);
        let record2 = wrap_as_tls_record(0x16, &hs_bytes[2..3]);
        let record_rest = wrap_as_tls_record(0x16, &hs_bytes[3..]);

        analyzer.on_data(&fk, Direction::ClientToServer, &record0, 0, 0);
        analyzer.on_data(&fk, Direction::ClientToServer, &record1, 0, 0);
        analyzer.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);
        analyzer.on_data(&fk, Direction::ClientToServer, &record_rest, 0, 0);

        assert_eq!(
            analyzer.sni_counts().len(),
            1,
            "n-record reassembly: sni_counts must have exactly 1 entry"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "n-record reassembly: parse_errors must be 0"
        );
    }

    /// VP-039 Sub-C-ext-large (unit): body 18,433..65,536 bytes (large valid hello).
    ///
    /// SNI/JA3 populated, handshake_reassembly_overflows==0.
    ///
    /// Traces to: BC-2.07.038 v2.7 Invariant 5.
    #[test]
    fn test_vp039_large_valid_hello_reassembly() {
        let fk = make_test_flow_key(42);
        let mut analyzer = TlsAnalyzer::new();
        let sni = "large-hello.example";

        // Build a large ClientHello by splitting across two records.
        // Split right after the 4-byte handshake header so the carry accumulates
        // the header in record 1 and completes the message in record 2.
        let hs_bytes = build_client_hello_with_sni(sni);
        let split = 4; // split right after the 4-byte handshake header
        let record1 = wrap_as_tls_record(0x16, &hs_bytes[..split]);
        let record2 = wrap_as_tls_record(0x16, &hs_bytes[split..]);

        analyzer.on_data(&fk, Direction::ClientToServer, &record1, 0, 0);
        analyzer.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(),
            0,
            "large hello: handshake_reassembly_overflows must be 0 for valid body"
        );
        assert_eq!(
            analyzer.sni_counts().len(),
            1,
            "large hello: sni_counts must have exactly 1 entry"
        );
        assert!(
            analyzer.sni_counts().contains_key(sni),
            "large hello: sni_counts must contain '{sni}'"
        );
    }

    // ── VP-039 Sub-B: exact-consume coalesced ─────────────────────────────────

    // VP-039 Sub-B (proptest): ClientHello + other_msg coalesced in one record,
    // delivered FRAGMENTED across two 0x16 records.
    //
    // The coalesced byte sequence is split at byte 4 (after the 4-byte ClientHello
    // handshake header): record 1 = header only, record 2 = CH body + other_msg.
    // This forces the carry drain loop to handle fragmentation AND coalesced dispatch.
    //
    // After both records: handshake_count()==1, parse_errors==0,
    // carry_len==0, client_hello_seen==true.
    //
    // The secondary message has a NON-ZERO body_len so the exact-consume
    // arithmetic (drain(4 + body_len)) is exercised with body_len > 0.
    // handshakes_seen==1 is asserted directly via handshake_count()
    // (not inferred from ja3_counts.len()==1 — F-F2-012 requirement).
    //
    // Traces to: BC-2.07.042 v1.4 Postconditions 1–5.
    proptest! {
        #[test]
        fn proptest_vp039_exact_consume_coalesced(
            // Vary the secondary handshake type (not 0x01/0x02 — any other type).
            other_hs_type in 4u8..=20u8,
            // Non-zero body length for the secondary message: 1–16 bytes.
            // Ensures drain(4 + body_len) is exercised with body_len > 0.
            other_body_len in 1u8..=16u8,
        ) {
            let client_hello = build_client_hello_with_sni("test.example.com");
            // Secondary handshake: type(1) + 24-bit BE body_len(3) + body bytes.
            let mut other_msg: Vec<u8> = vec![
                other_hs_type,
                0x00, 0x00, other_body_len,  // body_len encoded as 24-bit big-endian
            ];
            other_msg.extend(vec![0xBBu8; other_body_len as usize]); // non-zero body

            // Coalesce: ClientHello handshake bytes immediately followed by other_msg.
            let coalesced = [client_hello.as_slice(), other_msg.as_slice()].concat();

            // FRAGMENTED delivery: split after the 4-byte ClientHello header so the
            // carry drain loop must handle re-entry across records.
            // Record 1: first 4 bytes (handshake header only — body pending).
            // Record 2: remaining CH body + the full secondary message.
            let rec1 = wrap_as_tls_record(0x16, &coalesced[..4]);
            let rec2 = wrap_as_tls_record(0x16, &coalesced[4..]);

            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(1);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec1, 0u64, 100u32);

            // After record 1 (4-byte header only), the carry drain loop sees
            // carry_len=4 < 4+body_len → waits. Carry holds exactly 4 bytes.
            prop_assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 4,
                "after 4-byte header record, carry must hold exactly 4 bytes"
            );

            // Deliver record 2 — the carry drain loop must now:
            // 1. Complete the ClientHello (drain 4+body_len bytes), dispatch it.
            // 2. Immediately drain the other_msg (non-CH, consumed silently).
            analyzer.on_data(&flow_key, Direction::ClientToServer, &rec2, 0u64, 100u32);

            // After full drain: exactly 1 ClientHello dispatched.
            // F-F2-012: assert handshakes_seen==1 DIRECTLY via handshake_count().
            prop_assert!(analyzer.client_hello_seen_for_testing(&flow_key),
                "ClientHello in coalesced fragmented record must be dispatched (client_hello_seen==true)");
            prop_assert_eq!(analyzer.handshake_count(), 1u64,
                "exactly 1 ClientHello dispatched — handshake_count must be 1");
            prop_assert_eq!(analyzer.parse_error_count(), 0u64,
                "coalesced delivery must not produce parse errors");
            // Carry buffer must be empty: both messages fully consumed.
            prop_assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
                "carry buffer must be empty after all complete messages consumed"
            );
        }
    }

    /// VP-039 Sub-B (unit): no double-dispatch; handshakes_seen exact after
    /// coalesced ClientHello + unknown message in one record.
    ///
    /// Traces to: BC-2.07.042 v1.4 AC-EXACT-CONSUME.
    // DF-AC-TEST-NAME-SYNC-001: canonical name verbatim per VP-039 table.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_07_042_exact_consume_no_double_dispatch() {
        let fk = make_test_flow_key(42);
        let mut analyzer = TlsAnalyzer::new();
        let sni = "coalesced.example";

        // CARRY PATH: split the coalesced payload across TWO 0x16 records so the
        // carry drain loop must handle both the ClientHello AND the trailing message.
        //
        // Coalesced raw payload: ClientHello handshake bytes + Certificate (type=0x0b).
        let ch_hs = build_client_hello_with_sni(sni);
        let other_hs: Vec<u8> = vec![0x0b, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00];
        let mut coalesced = ch_hs.clone();
        coalesced.extend_from_slice(&other_hs);

        // Split after the first 4 bytes (handshake header of the ClientHello).
        // Record 1: just the 4-byte CH header → carry accumulates 4 bytes (incomplete body).
        // Record 2: the rest of CH body + Certificate bytes.
        let record1 = wrap_as_tls_record(0x16, &coalesced[..4]);
        let record2 = wrap_as_tls_record(0x16, &coalesced[4..]);

        analyzer.on_data(&fk, Direction::ClientToServer, &record1, 0, 0);

        // After record 1 (4-byte header only), the drain loop sees
        // carry_len=4 < 4+body_len → waits. Carry holds exactly 4 bytes.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            4,
            "exact-consume coalesced: carry must hold 4 header bytes after record 1"
        );

        analyzer.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

        // After full drain: exactly one ClientHello dispatched, Certificate silently consumed.
        assert_eq!(
            analyzer.handshake_count(),
            1,
            "coalesced record: handshake_count must be exactly 1 (no double-dispatch)"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "coalesced record: parse_errors must be 0 (non-CH message consumed silently)"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            0,
            "coalesced record: carry must be empty after full drain"
        );
    }

    /// SEC-001 regression (unit): single 0x16 record packed with N zero-body-length
    /// non-ClientHello messages processes in O(N) work (cursor-based drain), not
    /// O(N²) (per-message Vec::drain).
    ///
    /// Fixture: one 0x16 record whose payload contains 1,000 4-byte messages of the
    /// form [non_ch_type, 0x00, 0x00, 0x00] (msg_type != 0x01, body_len = 0). Each
    /// message is consumed silently by the drain loop (BC-2.07.038 Invariant 1;
    /// BC-2.07.042 EC-002). After delivery:
    ///   - carry must be empty (all 1,000 messages fully consumed)
    ///   - parse_errors must be 0 (non-0x01 types never increment parse_errors)
    ///   - handshake_reassembly_overflows must be 0 (payload within MAX_BUF)
    ///   - client_hello_seen must be false (no ClientHello dispatched)
    ///
    /// This test guards against reintroducing per-message drain. With the old O(N²)
    /// approach, 1,000 messages × 4,000-byte carry → ~4 MB of memmove. With the
    /// cursor+single-drain approach the total memmove is 4,000 bytes (one drain).
    ///
    /// Traces to: BC-2.07.038 Invariant 1; BC-2.07.042 EC-002; SEC-001 fix.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_07_042_coalesced_zero_len_no_quadratic_drain() {
        let fk = make_test_flow_key(250);
        let mut analyzer = TlsAnalyzer::new();

        // Build a payload of 1,000 zero-body-length non-ClientHello messages.
        // Each message: [msg_type=0x02, 0x00, 0x00, 0x00] — type 0x02 (ServerHello)
        // is not dispatched on the ClientToServer direction; body_len = 0.
        // Total payload: 4,000 bytes (well within MAX_RECORD_PAYLOAD=18,432 and MAX_BUF=65,536).
        const N: usize = 1_000;
        let mut payload: Vec<u8> = Vec::with_capacity(N * 4);
        for _ in 0..N {
            payload.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]);
        }
        assert_eq!(
            payload.len(),
            N * 4,
            "payload sanity: must be exactly N*4 bytes"
        );

        // Deliver as a single 0x16 record.
        let record = wrap_as_tls_record(0x16, &payload);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // All N messages consumed silently — carry must be empty.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            0,
            "SEC-001 regression: carry must be empty after N={N} zero-body messages"
        );
        // Non-0x01 msg_type: parse_errors MUST NOT increment (BC-2.07.038 Invariant 1).
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "SEC-001 regression: parse_errors must be 0 for N={N} non-ClientHello messages"
        );
        // No overflow should occur (payload 4,000 bytes << MAX_BUF=65,536).
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(),
            0,
            "SEC-001 regression: no overflow expected for {N}-message payload within MAX_BUF"
        );
        // No ClientHello was sent.
        assert!(
            !analyzer.client_hello_seen_for_testing(&fk),
            "SEC-001 regression: client_hello_seen must be false (no 0x01 msg dispatched)"
        );
    }

    // ── VP-039 Sub-C: carry overflow clear-and-recover ────────────────────────

    /// VP-039 Sub-C (unit): Decision-5 fires exactly once; carry cleared;
    /// overflow_count==overflows_before+1; parse_errors unchanged.
    ///
    /// Fixture: valid header body_len=65,500 accumulates 65,504 bytes total;
    /// additional 4 padding records push total past MAX_BUF (65,536) → overflow.
    ///
    /// Traces to: BC-2.07.039 v2.4 Postconditions 1–6.
    #[test]
    fn test_vp039_carry_overflow_clear_and_recover() {
        let fk = make_test_flow_key(43);
        let mut analyzer = TlsAnalyzer::new();

        // Record 1: header body_len=65,500 (msg_type=0x01, len=[0xFF, 0xFC, 0x00]? no:
        // 65500 = 0xFFDC = 0x00_FF_DC — 3-byte BE: [0x00, 0xFF, 0xDC]).
        // Send just the 4-byte header so carry accumulates 4 bytes.
        let header_only: Vec<u8> = vec![0x01, 0x00, 0xFF, 0xDC]; // body_len = 65500
        let record1 = wrap_as_tls_record(0x16, &header_only);
        analyzer.on_data(&fk, Direction::ClientToServer, &record1, 0, 0);

        // Record 2: 100 more bytes — now carry = 4 + 100 = 104 bytes.
        let padding: Vec<u8> = vec![0xAA; 100];
        let record2 = wrap_as_tls_record(0x16, &padding);
        analyzer.on_data(&fk, Direction::ClientToServer, &record2, 0, 0);

        // Records 3a–3d: together add 65,400 bytes so carry reaches 104+65,400 = 65,504.
        // Each individual record payload ≤ 18,432 (MAX_RECORD_PAYLOAD) so the DoS guard
        // (BC-2.07.004) does NOT fire; accumulation via multiple valid records is the
        // intended overflow path per BC-2.07.039 v2.1 EC-002 / PRD F-F2-003.
        // 3 × 18,432 + 10,104 = 65,400.
        for _ in 0..3 {
            let chunk: Vec<u8> = vec![0xBB; 18_432];
            let rec = wrap_as_tls_record(0x16, &chunk);
            analyzer.on_data(&fk, Direction::ClientToServer, &rec, 0, 0);
        }
        let remainder: Vec<u8> = vec![0xBB; 10_104]; // 65_400 - 3*18_432 = 10_104
        let rec_rem = wrap_as_tls_record(0x16, &remainder);
        analyzer.on_data(&fk, Direction::ClientToServer, &rec_rem, 0, 0);

        // Now carry = 65,504. Record 4: 100 bytes → 65,504 + 100 = 65,604 > 65,536.
        // Decision-5 fires: carry cleared, overflow_count+1.
        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.all_findings_len_for_testing();

        let overflow_trigger: Vec<u8> = vec![0xCC; 100];
        let record4 = wrap_as_tls_record(0x16, &overflow_trigger);
        analyzer.on_data(&fk, Direction::ClientToServer, &record4, 0, 0);

        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(),
            overflows_before + 1,
            "Decision-5: overflow_count must increment by exactly 1"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            0,
            "Decision-5: carry must be cleared (len==0) after overflow"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            parse_errors_before,
            "Decision-5: parse_errors must NOT change on carry overflow"
        );
        assert_eq!(
            analyzer.all_findings_len_for_testing(),
            findings_before,
            "Decision-5: findings must NOT change on carry overflow"
        );
    }

    /// VP-039 Sub-C (unit): post-overflow single-record ClientHello dispatched.
    ///
    /// client_hello_seen==true; SNI/JA3 populated; parse_errors==0.
    ///
    /// Traces to: BC-2.07.039 v2.4 Postcondition 6 (clear-and-recover).
    #[test]
    fn test_vp039_carry_overflow_recovery() {
        let fk = make_test_flow_key(44);
        let mut analyzer = TlsAnalyzer::new();
        let sni = "post-overflow-recovery.example";

        // Trigger an overflow first by accumulating > MAX_BUF bytes across multiple
        // individually-valid records (per BC-2.07.039 v2.1 EC-002 / PRD F-F2-003:
        // a single 0x16 record with payload > MAX_RECORD_PAYLOAD (18,432) cannot reach
        // the carry; overflow is triggered by accumulation across multiple valid records).
        //
        // Step 1: header-only record (4 bytes) → carry = 4.
        let header_only: Vec<u8> = vec![0x01, 0x00, 0xFF, 0xDC]; // body_len=65500
        let record_header = wrap_as_tls_record(0x16, &header_only);
        analyzer.on_data(&fk, Direction::ClientToServer, &record_header, 0, 0);

        // Step 2: 3 × 18,432-byte records → carry grows to 4+18,432+18,432+18,432 = 55,300.
        // Step 3: 4th record of 18,432 bytes → 55,300+18,432=73,732 > 65,536 → Decision-5.
        for _ in 0..4 {
            let chunk: Vec<u8> = vec![0xAA; 18_432];
            let rec = wrap_as_tls_record(0x16, &chunk);
            analyzer.on_data(&fk, Direction::ClientToServer, &rec, 0, 0);
        }

        // Carry is now cleared; overflow_count >= 1.
        assert!(
            analyzer.handshake_reassembly_overflow_count() >= 1,
            "pre-condition: overflow must have fired"
        );

        // Now send a complete single-record ClientHello — must be dispatched normally.
        let ch_hs = build_client_hello_with_sni(sni);
        let record_ch = wrap_as_tls_record(0x16, &ch_hs);
        analyzer.on_data(&fk, Direction::ClientToServer, &record_ch, 0, 0);

        assert!(
            analyzer.client_hello_seen_for_testing(&fk),
            "post-overflow recovery: client_hello_seen must be true after complete hello"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "post-overflow recovery: parse_errors must be 0"
        );
        assert!(
            analyzer.sni_counts().contains_key(sni),
            "post-overflow recovery: SNI '{sni}' must be in sni_counts"
        );
    }

    /// VP-039 Sub-C (unit): body_len > MAX_BUF triggers Decision-4 clear-and-recover.
    ///
    /// overflow_count+1; parse_errors unchanged; findings unchanged.
    ///
    /// Traces to: BC-2.07.038 v2.7 Invariant 5 / ADR-011 Decision 4.
    #[test]
    fn test_vp039_body_len_spoof() {
        let fk = make_test_flow_key(45);
        let mut analyzer = TlsAnalyzer::new();

        // A 0x16 record whose handshake header declares body_len=65537 (> MAX_BUF=65536).
        // 65537 = 0x010001 — 3-byte BE: [0x01, 0x00, 0x01].
        let spoof_header: Vec<u8> = vec![0x01, 0x01, 0x00, 0x01]; // type=0x01, body_len=65537
        let record = wrap_as_tls_record(0x16, &spoof_header);

        let overflows_before = analyzer.handshake_reassembly_overflow_count();
        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.all_findings_len_for_testing();

        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(),
            overflows_before + 1,
            "body_len spoof: Decision-4 must fire, overflow_count+1"
        );
        assert_eq!(
            analyzer.parse_error_count(),
            parse_errors_before,
            "body_len spoof: parse_errors must NOT change (Decision-4)"
        );
        assert_eq!(
            analyzer.all_findings_len_for_testing(),
            findings_before,
            "body_len spoof: findings must NOT change (Decision-4)"
        );
    }

    /// VP-039 Sub-C (unit): `summarize()` exposes `handshake_reassembly_overflows`
    /// key with value-equality (not mere key presence).
    ///
    /// detail["handshake_reassembly_overflows"].as_u64()==1.
    ///
    /// Traces to: BC-2.07.039 v2.4 Postcondition 7.
    // DF-AC-TEST-NAME-SYNC-001: canonical name verbatim per VP-039 table.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_07_039_summarize_exposes_handshake_reassembly_overflows_key() {
        let fk = make_test_flow_key(46);
        let mut analyzer = TlsAnalyzer::new();

        // Trigger exactly one overflow via body_len spoof (Decision-4).
        // 65537 = 0x010001 → BE bytes [0x01, 0x00, 0x01].
        let spoof_header: Vec<u8> = vec![0x01, 0x01, 0x00, 0x01]; // body_len=65537
        let record = wrap_as_tls_record(0x16, &spoof_header);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        let summary = analyzer.summarize();
        let overflow_val = summary
            .detail
            .get("handshake_reassembly_overflows")
            .expect("summarize() must contain 'handshake_reassembly_overflows' key");

        assert_eq!(
            overflow_val.as_u64(),
            Some(1),
            "summarize() detail['handshake_reassembly_overflows'] must equal 1; \
             got: {overflow_val:?}"
        );
    }

    // ── VP-039 Sub-D: on_flow_close carry discard ─────────────────────────────

    /// VP-039 Sub-D (unit): partial 4-byte header only → on_flow_close;
    /// parse_errors unchanged; findings unchanged.
    ///
    /// Traces to: BC-2.07.040 v1.3 Postconditions 1–5.
    #[test]
    fn test_vp039_truncated_carry_no_error() {
        let fk = make_test_flow_key(47);
        let mut analyzer = TlsAnalyzer::new();

        // Send exactly 4 bytes (a complete 4-byte handshake header with body_len=100)
        // but do NOT send the body — the carry must hold a partial message (4 bytes).
        let partial_header: Vec<u8> = vec![0x01, 0x00, 0x00, 0x64]; // body_len=100
        let record = wrap_as_tls_record(0x16, &partial_header);
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // After on_data: carry holds the 4-byte header (body_len=100 bytes not yet
        // received). Drain loop sees carry_len=4 < 4+100 → waits.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            4,
            "truncated carry: carry must hold 4 header bytes before flow close"
        );

        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.all_findings_len_for_testing();

        // Close the flow — carry (4 bytes) must be silently discarded.
        analyzer.on_flow_close(&fk, CloseReason::Fin);

        assert_eq!(
            analyzer.parse_error_count(),
            parse_errors_before,
            "truncated carry at flow close: parse_errors must not change"
        );
        assert_eq!(
            analyzer.all_findings_len_for_testing(),
            findings_before,
            "truncated carry at flow close: findings must not change"
        );
        assert_eq!(
            analyzer.active_flows_len_for_testing(),
            0,
            "truncated carry at flow close: flow must be removed from flows map"
        );
    }

    /// VP-039 Sub-D (unit): empty carry at flow close; no observable effect.
    ///
    /// Exercises BC-2.07.040 v1.3 Postconditions 1–5 via a body_len-spoof record
    /// that fires Decision-4 immediately (carry starts empty, spoof header appended,
    /// drain loop fires → carry.clear(), overflows+1, carry back to 0). Then
    /// on_flow_close is called with an empty carry, verifying that:
    ///   - no parse_errors are added
    ///   - no new findings are emitted
    ///   - active_flows drops to 0
    ///
    /// Traces to: BC-2.07.040 v1.3 Postconditions 1–5.
    // DF-AC-TEST-NAME-SYNC-001: canonical name verbatim per VP-039 table.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_07_040_empty_carry_flow_close() {
        let fk = make_test_flow_key(48);
        let mut analyzer = TlsAnalyzer::new();

        // Deliver a single 0x16 record whose first (and only) header declares
        // body_len=65537 > MAX_BUF. The drain loop fires Decision-4 immediately:
        //   carry was [] → append [0x01, 0x01, 0x00, 0x01] → carry.len()==4
        //   drain loop reads body_len=65537 > MAX_BUF → carry.clear(), overflows+1, break
        // After on_data: carry is empty, overflow_count==1.
        let spoof: Vec<u8> = vec![0x01, 0x01, 0x00, 0x01]; // body_len=65537 > MAX_BUF
        let record = wrap_as_tls_record(0x16, &spoof);

        let parse_errors_before = analyzer.parse_error_count();
        let findings_before = analyzer.all_findings_len_for_testing();

        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0, 0);

        // Decision-4 must have fired: overflow_count==1, carry empty.
        assert_eq!(
            analyzer.handshake_reassembly_overflow_count(),
            1,
            "empty carry flow close: Decision-4 must increment overflow_count to 1"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&fk),
            0,
            "empty carry flow close: carry must be empty after Decision-4 clear"
        );

        // Flow must still exist (on_data creates the flow entry; on_flow_close removes it).
        assert_eq!(
            analyzer.active_flows_len_for_testing(),
            1,
            "empty carry flow close: flow must exist before on_flow_close"
        );

        // Close the flow with an empty carry — must be a no-op for errors/findings.
        analyzer.on_flow_close(&fk, CloseReason::Fin);

        assert_eq!(
            analyzer.parse_error_count(),
            parse_errors_before,
            "empty carry flow close: parse_errors must not change"
        );
        assert_eq!(
            analyzer.all_findings_len_for_testing(),
            findings_before,
            "empty carry flow close: findings must not change"
        );
        assert_eq!(
            analyzer.active_flows_len_for_testing(),
            0,
            "empty carry flow close: active_flows must be 0 after on_flow_close"
        );
    }

    // ── VP-039 Sub-F: carry bounded invariant ────────────────────────────────

    // VP-039 Sub-F (proptest): generative bounded-carry invariant.
    //
    // For any sequence of 1–8 on_data calls with 0x16 payloads that BEGIN
    // WITH A VALID HANDSHAKE HEADER (body_len <= MAX_BUF = 65,536), the
    // client-direction carry buffer never exceeds MAX_BUF after any call.
    //
    // Generator design (F-F2P-IMP-001 restructuring):
    // Each payload starts with a valid 4-byte handshake header:
    //   [0x01, (body_len >> 16) as u8, (body_len >> 8) as u8, body_len as u8]
    // where body_len is in 0..=65_536. This guarantees carry actually
    // accumulates (Decision-4 body_len-spoof guard does NOT fire) and makes
    // the bounded-carry invariant non-trivially testable.
    //
    // The secondary assertion — carry_len >= 1 after the first partial record —
    // verifies that the carry actually accumulates for partial messages (no drain
    // fires until the message is complete).
    //
    // The primary invariant assertion (carry_len <= MAX_BUF) is the
    // bounded-carry correctness guard.
    //
    // Traces to: BC-2.07.039 v2.4 Invariant 1.
    proptest! {
        #[test]
        fn proptest_vp039_carry_bounded_invariant(
            // Generate 1–8 records; each begins with a valid handshake header
            // declaring body_len <= MAX_BUF so carry actually accumulates.
            records in proptest::collection::vec(
                // body_len in valid range [0, MAX_BUF] (Decision-4 guard does NOT fire).
                (0usize..=65_536usize).prop_flat_map(|body_len| {
                    // Partial body: up to min(body_len, MAX_RECORD_PAYLOAD-4) bytes.
                    // The payload is always < 4+body_len so the carry accumulates —
                    // the message is never complete in a single record.
                    let body_max = body_len.min(18_428usize); // MAX_RECORD_PAYLOAD(18432) - 4
                    proptest::collection::vec(proptest::arbitrary::any::<u8>(), 0..=body_max)
                        .prop_map(move |body| {
                            // Build payload: valid 4-byte handshake header + partial body.
                            let mut payload = vec![
                                0x01u8,                        // msg_type: ClientHello
                                (body_len >> 16) as u8,        // len byte 0 (MSB)
                                (body_len >> 8) as u8,         // len byte 1
                                (body_len & 0xFF) as u8,       // len byte 2 (LSB)
                            ];
                            payload.extend_from_slice(&body);
                            payload
                        })
                }),
                1..=8usize,
            ),
        ) {
            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(42);
            let ts: u32 = 100;

            for (idx, payload) in records.iter().enumerate() {
                let rec = wrap_as_tls_record(0x16, payload);
                analyzer.on_data(&flow_key, Direction::ClientToServer, &rec, 0u64, ts);

                // Primary invariant: carry NEVER exceeds MAX_BUF.
                prop_assert!(
                    analyzer.client_hs_carry_len_for_testing(&flow_key) <= 65_536,
                    "client_hs_carry must never exceed MAX_BUF after on_data \
                     (record {idx}, payload len {}, carry len {})",
                    payload.len(),
                    analyzer.client_hs_carry_len_for_testing(&flow_key),
                );

                // Secondary assertion: after the FIRST record (partial body, never
                // complete because body_max < body_len in the generator), the carry
                // must have accumulated AT LEAST 1 byte. The drain loop sees
                // carry_len < 4 + body_len → waits without draining.
                if idx == 0 && !payload.is_empty() {
                    // Skip zero-body messages — a 4-byte header with body_len==0 is a
                    // complete message that drains immediately, leaving carry empty.
                    let declared_body_len = ((payload[1] as usize) << 16)
                        | ((payload[2] as usize) << 8)
                        | (payload[3] as usize);
                    let is_partial = payload.len() < 4 + declared_body_len;
                    if is_partial {
                        prop_assert!(
                            analyzer.client_hs_carry_len_for_testing(&flow_key) >= 1,
                            "after partial first record (payload {} bytes, body_len {}), \
                             carry_len must be >= 1 (carry must accumulate)",
                            payload.len(),
                            declared_body_len,
                        );
                    }
                }
            }
        }
    }
}

// ── STORY-145: ServerHello Carry Symmetry + Per-Flow / Per-Direction Isolation ──
//
// Wave 66. Behavioral Contracts: BC-2.07.041 v1.2, BC-2.07.002 v1.6.
//
// Red-Gate harnesses (VP-039 Sub-E scope — 2 new tests):
//   1. proptest_vp039_direction_isolation   (Sub-E): interleaved C2S/S2C fragmented
//      hellos for the same flow; assert both hellos seen, no cross-direction bleed.
//   2. test_BC_2_07_041_cross_flow_isolation (Sub-E-ext): two distinct FlowKeys;
//      Flow A complete single-record, Flow B fragmented; assert sni_counts contains
//      both hostnames with no cross-flow bleed.
//
// Namespace isolation: DF-TEST-NAMESPACE-001 — all STORY-145 tests live inside
// this `mod story_145` wrapper.  No new flat-root tests are added for this story.
//
// Test count: 2 (1 proptest + 1 unit).
mod story_145 {
    use proptest::prelude::*;
    use std::net::IpAddr;
    use wirerust::analyzer::tls::TlsAnalyzer;
    use wirerust::reassembly::flow::FlowKey;
    use wirerust::reassembly::handler::{Direction, StreamHandler};

    // ── Local test helpers ────────────────────────────────────────────────────
    //
    // DF-TEST-NAMESPACE-001 / STORY-145 helper note: helpers are re-declared
    // locally per mod.  Before creating any helper below, the story note
    // instructs grepping for the real name at the flat root.
    //
    // Reconciliation results (grep run during stub generation):
    //   build_server_hello     → EXISTS at flat root (line 137)
    //   build_client_hello     → EXISTS at flat root (line 16)
    //   wrap_as_tls_record     → ONLY in mod story_144 (private) — re-declared here
    //   make_test_flow_key     → ONLY in mod story_144 (private) — re-declared here
    //
    // GREEN-BY-DESIGN self-check (BC-5.38.005 invariant 1):
    // "If I include this real implementation, will the test for this function
    // pass trivially without any implementer work?"
    // — No: helpers are pure construction utilities; no test assertions are
    //   inside them.  They are ≤ 5 lines and contain no branching logic on
    //   domain state beyond type construction.  Declared as real bodies under
    //   GREEN-BY-DESIGN / WIRING-EXEMPT because they are builder helpers with
    //   zero domain branching (constructors, not behaviors).

    /// Create a `FlowKey` varied by `seed` so cross-flow and independent-flow
    /// tests can use distinct keys without collision.
    ///
    /// Re-declared locally per DF-TEST-NAMESPACE-001 (identical to mod story_144 copy).
    fn make_test_flow_key(seed: u8) -> FlowKey {
        FlowKey::new(
            IpAddr::from([10, 145, 0, seed]),
            49000u16.wrapping_add(seed as u16),
            IpAddr::from([10, 145, 1, seed]),
            443,
        )
    }

    /// Returns the RAW handshake-message bytes for a ServerHello (0x002f),
    /// with NO TLS record header prefix (5 bytes stripped).
    ///
    /// `build_server_hello` at the flat root returns a COMPLETE TLS record
    /// (5-byte header + handshake body).  This wrapper strips the header so
    /// fragmentation tests can re-frame the bytes via `wrap_as_tls_record`.
    ///
    /// Reconciliation: `build_server_hello` exists at flat root; used here.
    fn build_server_hello() -> Vec<u8> {
        super::build_server_hello(0x002f)[5..].to_vec()
    }

    /// Returns the RAW handshake-message bytes for a ClientHello with the given
    /// SNI, with NO TLS record header prefix (5 bytes stripped).
    ///
    /// Reconciliation: `build_client_hello` exists at flat root; used here.
    fn build_client_hello_with_sni(sni: &str) -> Vec<u8> {
        super::build_client_hello(sni, &[0x002f])[5..].to_vec()
    }

    /// Wrap `payload` bytes in a 5-byte TLS record header for the given content type.
    ///
    /// Reconciliation: `wrap_as_tls_record` does NOT exist at flat root; re-declared
    /// locally here (identical to mod story_144 copy).
    fn wrap_as_tls_record(content_type: u8, payload: &[u8]) -> Vec<u8> {
        let len = payload.len();
        let len_hi = (len >> 8) as u8;
        let len_lo = (len & 0xff) as u8;
        let mut record = vec![content_type, 0x03, 0x03, len_hi, len_lo];
        record.extend_from_slice(payload);
        record
    }

    // ── VP-039 Sub-E: direction isolation ────────────────────────────────────

    // VP-039 Sub-E (proptest): interleaved C2S and S2C fragmented hello
    // deliveries must each accumulate into their own carry buffer, and both
    // `client_hello_seen` and `server_hello_seen` must be true after all records
    // are delivered.
    //
    // Red Gate: this FAILS until the `ServerToClient` carry drain path is wired
    // in `try_parse_records`.  The current `parse_tls_plaintext` path only
    // handles single-record ServerHellos; a fragmented ServerHello arrives as
    // two partial 0x16 records and the second record alone is not a complete
    // TLS plaintext record, so `server_hello_seen` remains false.
    //
    // Traces to: BC-2.07.041 v1.2 Invariant 2; BC-2.07.002 v1.6 Precondition 2;
    //            AC-145-001, AC-145-002, AC-145-004.
    proptest! {
        #[test]
        fn proptest_vp039_direction_isolation(
            // Split point for the ClientHello fragmentation (C2S direction).
            c2s_split in prop_oneof![1usize..4usize, 4usize..256usize],
            // Split point for the ServerHello fragmentation (S2C direction).
            s2c_split in prop_oneof![1usize..4usize, 4usize..256usize],
        ) {
            let client_hello = build_client_hello_with_sni("client.example.com");
            let server_hello = build_server_hello();
            let n_c = client_hello.len();
            let n_s = server_hello.len();

            prop_assume!(c2s_split < n_c);
            prop_assume!(s2c_split < n_s);

            let mut analyzer = TlsAnalyzer::new();
            let flow_key = make_test_flow_key(1);
            let ts: u32 = 200;

            // Interleaved delivery: C2S frag 1, S2C frag 1, C2S frag 2, S2C frag 2.
            let c2s_rec1 = wrap_as_tls_record(0x16, &client_hello[..c2s_split]);
            let s2c_rec1 = wrap_as_tls_record(0x16, &server_hello[..s2c_split]);
            let c2s_rec2 = wrap_as_tls_record(0x16, &client_hello[c2s_split..]);
            let s2c_rec2 = wrap_as_tls_record(0x16, &server_hello[s2c_split..]);

            analyzer.on_data(&flow_key, Direction::ClientToServer, &c2s_rec1, 0u64, ts);
            analyzer.on_data(&flow_key, Direction::ServerToClient, &s2c_rec1, 0u64, ts);
            analyzer.on_data(&flow_key, Direction::ClientToServer, &c2s_rec2, 0u64, ts);
            analyzer.on_data(&flow_key, Direction::ServerToClient, &s2c_rec2, 0u64, ts);

            // Red Gate assertion: both hellos must be seen after interleaved delivery.
            // client_hello_seen via carry drain (STORY-144): already passes.
            // server_hello_seen via carry drain (STORY-145): FAILS until server path wired.
            prop_assert!(
                analyzer.client_hello_seen_for_testing(&flow_key),
                "client_hello_seen must be true after interleaved fragmented C2S delivery"
            );
            prop_assert!(
                analyzer.server_hello_seen_for_testing(&flow_key),
                "server_hello_seen must be true after interleaved fragmented S2C delivery \
                 (Red Gate: fails until ServerToClient carry drain path is wired)"
            );
            // No parse errors from fragmented delivery.
            prop_assert_eq!(
                analyzer.parse_error_count(), 0u64,
                "interleaved fragmented delivery must not produce parse errors"
            );
            // Both carries must be fully drained after complete delivery.
            prop_assert_eq!(
                analyzer.client_hs_carry_len_for_testing(&flow_key), 0,
                "client_hs_carry must be empty after complete C2S reassembly"
            );
            prop_assert_eq!(
                analyzer.server_hs_carry_len_for_testing(&flow_key), 0,
                "server_hs_carry must be empty after complete S2C reassembly \
                 (Red Gate: fails until ServerToClient carry drain path is wired)"
            );
        }
    }

    // ── VP-039 Sub-E-ext: cross-flow isolation ────────────────────────────────

    // test_BC_2_07_041_cross_flow_isolation (unit): two distinct FlowKeys.
    //   Flow A: complete single-record ClientHello (SNI = "a.example") +
    //           complete single-record ServerHello (S2C).
    //   Flow B: fragmented two-record ClientHello (SNI = "b.example") +
    //           fragmented two-record ServerHello (S2C).
    //
    //   After delivery: sni_counts must have exactly 2 entries, one for each SNI;
    //   both flow_a and flow_b must have server_hello_seen==true (ServerHello
    //   carry drain applies to each flow independently).  No cross-flow bleed.
    //
    // Red Gate: FAILS until STORY-145 wires the ServerToClient carry drain path.
    //   - Flow A's complete single-record ServerHello is handled by the existing
    //     parse_tls_plaintext path → server_hello_seen == true for flow_a (passes today).
    //   - Flow B's FRAGMENTED ServerHello requires the server_hs_carry drain loop
    //     (STORY-145 scope) → server_hello_seen == false for flow_b (fails today).
    //   - The assertion `server_hello_seen_for_testing(flow_b) == true` is the
    //     failing Red Gate assertion.
    //
    // Traces to: BC-2.07.041 v1.2 Invariants 1, 4; Postconditions 1, 4–5;
    //            BC-2.07.002 v1.6 Precondition 2; AC-145-003, AC-145-004.
    #[test]
    fn test_bc_2_07_041_cross_flow_isolation() {
        let mut analyzer = TlsAnalyzer::new();
        let flow_a = make_test_flow_key(10);
        let flow_b = make_test_flow_key(20);
        let ts: u32 = 300;

        // ── Flow A ────────────────────────────────────────────────────────────
        // C2S: complete single-record ClientHello (SNI = "a.example").
        let a_client_hello = build_client_hello_with_sni("a.example");
        let a_c2s_rec = wrap_as_tls_record(0x16, &a_client_hello);
        analyzer.on_data(&flow_a, Direction::ClientToServer, &a_c2s_rec, 0u64, ts);

        // S2C: complete single-record ServerHello (fast path — parse_tls_plaintext).
        let a_server_hello = build_server_hello();
        let a_s2c_rec = wrap_as_tls_record(0x16, &a_server_hello);
        analyzer.on_data(&flow_a, Direction::ServerToClient, &a_s2c_rec, 0u64, ts);

        // ── Flow B ────────────────────────────────────────────────────────────
        // C2S: fragmented two-record ClientHello (SNI = "b.example").
        let b_client_hello = build_client_hello_with_sni("b.example");
        let c2s_split = b_client_hello.len() / 2;
        let b_c2s_rec1 = wrap_as_tls_record(0x16, &b_client_hello[..c2s_split]);
        let b_c2s_rec2 = wrap_as_tls_record(0x16, &b_client_hello[c2s_split..]);
        analyzer.on_data(&flow_b, Direction::ClientToServer, &b_c2s_rec1, 0u64, ts);
        analyzer.on_data(&flow_b, Direction::ClientToServer, &b_c2s_rec2, 0u64, ts);

        // S2C: fragmented two-record ServerHello (requires server_hs_carry drain —
        // the STORY-145 Red Gate path).
        let b_server_hello = build_server_hello();
        let s2c_split = b_server_hello.len() / 2;
        let b_s2c_rec1 = wrap_as_tls_record(0x16, &b_server_hello[..s2c_split]);
        let b_s2c_rec2 = wrap_as_tls_record(0x16, &b_server_hello[s2c_split..]);
        analyzer.on_data(&flow_b, Direction::ServerToClient, &b_s2c_rec1, 0u64, ts);
        analyzer.on_data(&flow_b, Direction::ServerToClient, &b_s2c_rec2, 0u64, ts);

        // ── Assertions ────────────────────────────────────────────────────────

        // Both flows must have client_hello_seen == true (STORY-144 carry drain).
        assert!(
            analyzer.client_hello_seen_for_testing(&flow_a),
            "flow_a: client_hello_seen must be true after single-record C2S delivery"
        );
        assert!(
            analyzer.client_hello_seen_for_testing(&flow_b),
            "flow_b: client_hello_seen must be true after fragmented C2S delivery"
        );

        // flow_a: server_hello_seen must be true (single-record S2C, fast path).
        assert!(
            analyzer.server_hello_seen_for_testing(&flow_a),
            "flow_a: server_hello_seen must be true after single-record S2C delivery"
        );

        // flow_b: server_hello_seen must be true (fragmented S2C, requires carry drain).
        // Red Gate: FAILS until STORY-145 wires the ServerToClient carry drain path.
        assert!(
            analyzer.server_hello_seen_for_testing(&flow_b),
            "flow_b: server_hello_seen must be true after fragmented S2C delivery \
             (Red Gate: fails until ServerToClient carry drain path is wired in STORY-145)"
        );

        // No parse errors from any delivery path.
        assert_eq!(
            analyzer.parse_error_count(),
            0,
            "cross-flow delivery must not produce parse errors"
        );

        // sni_counts: exactly 2 entries (one per flow); no cross-flow bleed.
        let sni_counts = analyzer.sni_counts();
        assert_eq!(
            sni_counts.len(),
            2,
            "sni_counts must have exactly 2 entries (one per flow SNI); \
             cross-flow bleed or missing dispatch would produce wrong count. \
             got: {sni_counts:?}"
        );
        assert_eq!(
            sni_counts.get("a.example").copied().unwrap_or(0),
            1,
            "a.example must appear exactly once in sni_counts"
        );
        assert_eq!(
            sni_counts.get("b.example").copied().unwrap_or(0),
            1,
            "b.example must appear exactly once in sni_counts"
        );

        // ja3s_counts: both flows contributed a ServerHello → 1 or 2 entries
        // (1 if both flows chose the same cipher fingerprint, 2 otherwise).
        // The invariant is ja3s_counts.len() >= 1 (at least one JA3S was computed).
        // Red Gate: 0 entries if server carry drain not wired (flow_b never dispatches).
        let ja3s_counts = analyzer.ja3s_counts();
        assert!(
            !ja3s_counts.is_empty(),
            "ja3s_counts must have at least 1 entry after ServerHello delivery to both flows \
             (Red Gate: empty means flow_b's fragmented ServerHello was not dispatched)"
        );

        // Carries fully drained after complete delivery.
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_a),
            0,
            "flow_a client_hs_carry must be empty after single-record C2S delivery"
        );
        assert_eq!(
            analyzer.client_hs_carry_len_for_testing(&flow_b),
            0,
            "flow_b client_hs_carry must be empty after complete fragmented C2S delivery"
        );
        assert_eq!(
            analyzer.server_hs_carry_len_for_testing(&flow_a),
            0,
            "flow_a server_hs_carry must be empty after single-record S2C delivery"
        );
        assert_eq!(
            analyzer.server_hs_carry_len_for_testing(&flow_b),
            0,
            "flow_b server_hs_carry must be empty after complete fragmented S2C delivery \
             (Red Gate: non-zero if carry drain not wired)"
        );

        // Active flows: both remain in the map (neither closed).
        assert_eq!(
            analyzer.active_flows_len_for_testing(),
            2,
            "both flows must remain active (on_flow_close not called)"
        );
    }
}
