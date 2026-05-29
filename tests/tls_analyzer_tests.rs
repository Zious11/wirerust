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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
        f.mitre_technique, None,
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
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0);

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
    analyzer3.on_data(&fk, Direction::ClientToServer, &record3, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // Server selects TLS_RSA_WITH_RC4_128_MD5 (0x0004) — RC4 triggers is_weak_server_cipher.
    // (is_weak_cipher does NOT include RC4 — AC-005 invariant 1)
    // 0x0004 = TLS_RSA_WITH_RC4_128_MD5 per AC-005/BC-2.07.010 EC-001.
    let sh = build_server_hello(0x0004);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
        f.mitre_technique, None,
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
    analyzer_rc4_client.on_data(&fk, Direction::ClientToServer, &ch_rc4, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello: strong cipher TLS_AES_128_GCM_SHA256 (0x1301), no weak cipher.
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);
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
    analyzer.on_data(&fk, Direction::ServerToClient, &large_app_data, 0);

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
    analyzer.on_data(&fk, Direction::ServerToClient, &[], 0);
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
    // JA3 string = "771,,,," -> MD5 = bddda940f9963577c41d7c28b1a5f65f
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
    );
    analyzer.on_data(
        &fk,
        Direction::ServerToClient,
        &build_server_hello_all_grease_ext(0x002f),
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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);
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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);
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
    analyzer.on_data(&fk, Direction::ClientToServer, &overflow_record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // server_hello_seen must be false before the ServerHello arrives.
    assert!(
        !analyzer.server_hello_seen_for_testing(&fk),
        "AC-001 precondition (BC-2.07.002 pc1): server_hello_seen must be false before ServerHello"
    );

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // version_counts[0x0303] == 1 after ClientHello only.
    assert_eq!(
        *analyzer.version_counts().get(&0x0303).unwrap_or(&0),
        1,
        "AC-002 anchor (BC-2.07.002 pc2): version_counts[0x0303] must be 1 after ClientHello"
    );

    // build_server_hello uses version 0x0303.
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // build_server_hello(0x1301) produces:
    //   version = 0x0303 (771), cipher = 0x1301 (4865),
    //   ext = [renegotiation_info 0xff01 (65281)]
    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with GREASE extension 0x0a0a + renegotiation_info 0xff01.
    let sh = build_server_hello_with_grease_ext(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // Server selects cipher 0xFFFF (unknown / unassigned ID).
    let sh = build_server_hello(0xFFFF);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

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
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with no extensions (sh.ext = None).
    let sh = build_server_hello_no_extensions(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // Server selects TLS_NULL_WITH_NULL_NULL (0x0000).
    let sh = build_server_hello(0x0000);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with version 0x0200 (SSL 2.0), no extensions.
    let sh = build_server_hello_with_version_and_cipher(0x0200, 0x1301, false);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with version 0x0301 (TLS 1.0), cipher 0x1301.
    let sh = build_server_hello_with_version_and_cipher(0x0301, 0x1301, true);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
        analyzer.on_data(&fk, Direction::ClientToServer, &ch_bytes, 0);
        // Cipher IDs 1..=50000. Not all are known to TlsCipherSuite, but
        // that only affects cipher_counts key format, not JA3S computation.
        let sh_bytes = build_server_hello(i as u16);
        analyzer.on_data(&fk, Direction::ServerToClient, &sh_bytes, 0);
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
    analyzer.on_data(&overflow_fk, Direction::ClientToServer, &ch_bytes, 0);
    let overflow_sh = build_server_hello(overflow_cipher);
    analyzer.on_data(&overflow_fk, Direction::ServerToClient, &overflow_sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

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
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer2.on_data(&fk, Direction::ClientToServer, &record2, 0);

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
    a_arm1.on_data(&fk, Direction::ClientToServer, &record_clean, 0);

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
    a_arm2.on_data(&fk, Direction::ClientToServer, &record_ctrl, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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

    // BC-2.07.014 pc2: mitre_technique = Some("T1027").
    assert_eq!(
        f.mitre_technique.as_deref(),
        Some("T1027"),
        "AC-003 (BC-2.07.014 pc2): mitre_technique must be Some(\"T1027\")"
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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
        analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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

    // Discriminating: mitre_technique is T1027 (not T1036 or None).
    let f = analyzer
        .findings()
        .into_iter()
        .find(|f| f.summary.contains("ASCII control characters"))
        .unwrap();
    assert_eq!(
        f.mitre_technique.as_deref(),
        Some("T1027"),
        "BC-2.07.016 EC-003: NUL-byte finding must be T1027"
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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer_ssl30.on_data(&fk, Direction::ClientToServer, &ssl30_record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &record, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with version 0x0300 (SSL 3.0) and a strong cipher (no server-cipher finding).
    // Use TLS_AES_128_GCM_SHA256 (0x1301) as the selected cipher so only the version fires.
    let sh = build_server_hello_with_version(0x0300, 0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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

    // BC-2.07.012 postcondition 1: summary contains "negotiated" (not "uses") and "RFC 7568".
    assert!(
        f.summary.contains("negotiated"),
        "BC-2.07.012 postcondition 1: server summary must use 'negotiated' (server finalizes \
         version); got: {:?}",
        f.summary
    );
    assert!(
        f.summary.contains("SSL 3.0"),
        "BC-2.07.012 postcondition 1: summary must name 'SSL 3.0' (version_name for 0x0300); \
         got: {:?}",
        f.summary
    );
    assert!(
        f.summary.contains("RFC 7568"),
        "BC-2.07.012 postcondition 1: summary must contain 'RFC 7568'; got: {:?}",
        f.summary
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
        f.mitre_technique, None,
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
    analyzer_tls10.on_data(&fk, Direction::ClientToServer, &ch2, 0);
    let sh_tls10 = build_server_hello_with_version(0x0301, 0x1301);
    analyzer_tls10.on_data(&fk, Direction::ServerToClient, &sh_tls10, 0);
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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // SSL 3.0 ServerHello with strong cipher.
    let sh = build_server_hello_with_version(0x0300, 0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with cipher ID 0x1234 — unrecognized by tls_parser.
    let sh = build_server_hello(0x1234);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer_aaaa.on_data(&fk, Direction::ClientToServer, &ch2, 0);
    let sh_aaaa = build_server_hello(0xAAAA);
    analyzer_aaaa.on_data(&fk, Direction::ServerToClient, &sh_aaaa, 0);

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    let sh = build_server_hello(0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
    analyzer_ffff.on_data(&fk, Direction::ClientToServer, &ch2, 0);

    let sh_ffff = build_server_hello(0xFFFF);
    analyzer_ffff.on_data(&fk, Direction::ServerToClient, &sh_ffff, 0);

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
    analyzer_null.on_data(&fk, Direction::ClientToServer, &ch3, 0);

    let sh_null = build_server_hello(0x0000);
    analyzer_null.on_data(&fk, Direction::ServerToClient, &sh_null, 0);

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
    analyzer_ssl2.on_data(&fk, Direction::ClientToServer, &record_ssl2, 0);

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
    analyzer_legacy.on_data(&fk, Direction::ClientToServer, &record_legacy, 0);

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
// ROUTE TO STORY-WRITER / PRODUCT-OWNER: BC-2.07.012 EC-004 ("ServerHello with
// version=0x0200 emits deprecated-protocol finding with version_name='SSL 2.0'")
// and EC-005 ("version below 0x0200 emits finding with version_name='Unknown legacy SSL'")
// are UNREACHABLE under tls-parser 0.12 via ServerHello records. The server-side 0x0200
// and catchall version_name arms cannot be triggered via wire input. EC-004/EC-005 should
// be corrected to document the parse-rejection behavior, or deferred until tls-parser
// is upgraded to a version that accepts these ServerHello variants.

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
    analyzer.on_data(&fk, Direction::ClientToServer, &ch, 0);

    // ServerHello with body version 0x0200 (SSL 2.0).
    let sh = build_server_hello_with_version(0x0200, 0x1301);
    analyzer.on_data(&fk, Direction::ServerToClient, &sh, 0);

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
