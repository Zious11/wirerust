use std::net::IpAddr;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamAnalyzer, StreamHandler};
use wirerust::reporter::Reporter;
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::terminal::TerminalReporter;
use wirerust::summary::Summary;

#[test]
fn test_json_reporter_produces_valid_json() {
    let reporter = JsonReporter;
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "Test finding".into(),
        evidence: vec!["evidence line".into()],
        mitre_technique: Some("T1046".into()),
        source_ip: None,
        timestamp: None,
    }];
    let analyzer_summaries = vec![];

    let output = reporter.render(&summary, &findings, &analyzer_summaries);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed.get("summary").is_some());
    assert!(parsed.get("findings").is_some());
    let findings_arr = parsed["findings"].as_array().unwrap();
    assert_eq!(findings_arr.len(), 1);
    assert_eq!(findings_arr[0]["summary"], "Test finding");
}

#[test]
fn test_json_finding_omits_absent_optional_fields_symmetrically() {
    // LESSON-P1.02 / NFR OBS-010: a `Finding` whose three `Option<_>`
    // fields are all `None` must emit a JSON object with *none* of
    // those three keys present. Previously `timestamp` had
    // `skip_serializing_if = "Option::is_none"` while `mitre_technique`
    // and `source_ip` did not, producing asymmetric mixed-shape output
    // (`mitre_technique: null` and `source_ip: null` appeared as keys
    // while `timestamp` was omitted).
    let reporter = JsonReporter;
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "P1.02 symmetry test".into(),
        evidence: vec![],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    }];
    let output = reporter.render(&summary, &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = &parsed["findings"][0];
    let obj = finding.as_object().expect("finding must be a JSON object");
    assert!(
        !obj.contains_key("mitre_technique"),
        "absent mitre_technique must be omitted, got: {}",
        finding
    );
    assert!(
        !obj.contains_key("source_ip"),
        "absent source_ip must be omitted, got: {}",
        finding
    );
    assert!(
        !obj.contains_key("timestamp"),
        "absent timestamp must be omitted, got: {}",
        finding
    );
}

#[test]
fn test_json_finding_emits_present_optional_fields() {
    // LESSON-P1.02 companion: present `Option::Some(_)` values must
    // still be serialized as JSON object members. Confirms that the
    // skip_serializing_if attribute only suppresses None, not Some.
    let reporter = JsonReporter;
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "P1.02 presence test".into(),
        evidence: vec![],
        mitre_technique: Some("T1036".into()),
        source_ip: Some(IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1))),
        timestamp: None, // intentionally None — verifies mixed presence
    }];
    let output = reporter.render(&summary, &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = &parsed["findings"][0];
    let obj = finding.as_object().expect("finding must be a JSON object");
    assert_eq!(
        obj.get("mitre_technique"),
        Some(&serde_json::json!("T1036"))
    );
    assert_eq!(obj.get("source_ip"), Some(&serde_json::json!("10.0.0.1")));
    assert!(
        !obj.contains_key("timestamp"),
        "the still-None timestamp must remain omitted"
    );
}

// ---- LESSON-P1.03: terminal --hosts gates per-host breakdown section ----

fn make_summary_with_two_hosts() -> Summary {
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};

    let mut summary = Summary::new();
    summary.ingest(&ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port: 12345,
            dst_port: 80,
            seq_number: 1,
            syn: false,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 60,
    });
    summary
}

#[test]
fn test_terminal_hosts_breakdown_off_by_default() {
    // LESSON-P1.03 / P1.04: without `--hosts`, the terminal output
    // must keep its existing compact `Hosts: N` count line and not
    // emit a HOSTS section. This preserves back-compat for callers
    // who built parsers against the pre-P1.03 output.
    let summary = make_summary_with_two_hosts();
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&summary, &[], &[]);
    assert!(
        out.contains("Hosts: 2"),
        "header count line must remain unconditional; got: {out}"
    );
    assert!(
        !out.contains("HOSTS"),
        "HOSTS section must be hidden when show_hosts_breakdown is false; got: {out}"
    );
}

#[test]
fn test_terminal_hosts_breakdown_lists_each_host_when_enabled() {
    // LESSON-P1.03: with `--hosts` (show_hosts_breakdown == true),
    // the terminal output emits a HOSTS section listing every unique
    // src/dst IP, in `Summary::unique_hosts()`'s sorted order.
    let summary = make_summary_with_two_hosts();
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: true,
    };
    let out = reporter.render(&summary, &[], &[]);
    assert!(
        out.contains("HOSTS"),
        "HOSTS section must be present when show_hosts_breakdown is true; got: {out}"
    );
    assert!(
        out.contains("10.0.0.1"),
        "host 10.0.0.1 must appear in the breakdown; got: {out}"
    );
    assert!(
        out.contains("10.0.0.2"),
        "host 10.0.0.2 must appear in the breakdown; got: {out}"
    );
    // Sorted: 10.0.0.1 must precede 10.0.0.2 in the output.
    let p1 = out.find("10.0.0.1").expect("10.0.0.1 in output");
    let p2 = out.find("10.0.0.2").expect("10.0.0.2 in output");
    assert!(p1 < p2, "hosts must be listed in sorted order");
}

#[test]
fn test_json_reporter_includes_skipped_packets() {
    let reporter = JsonReporter;
    let mut summary = Summary::new();
    summary.skipped_packets = 42;

    let output = reporter.render(&summary, &[], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed["summary"]["skipped_packets"], 42);
}

#[test]
fn test_json_reporter_skipped_packets_zero_by_default() {
    let reporter = JsonReporter;
    let summary = Summary::new();

    let output = reporter.render(&summary, &[], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed["summary"]["skipped_packets"], 0);
}

#[test]
fn test_terminal_reporter_shows_skipped_when_nonzero() {
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    };
    let mut summary = Summary::new();
    summary.skipped_packets = 5;

    let output = reporter.render(&summary, &[], &[]);
    assert!(
        output.contains("Skipped: 5 packets"),
        "Terminal output should show skipped count, got: {output}"
    );
}

#[test]
fn test_terminal_reporter_hides_skipped_when_zero() {
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    };
    let summary = Summary::new();

    let output = reporter.render(&summary, &[], &[]);
    assert!(
        !output.contains("Skipped"),
        "Terminal output should NOT show 'Skipped' when zero, got: {output}"
    );
}

#[test]
fn test_terminal_reporter_escapes_esc_bytes_in_summary() {
    // Regression: a Finding whose summary contains an ESC byte must not
    // propagate the raw byte to terminal output, where it would be
    // interpreted as an ANSI escape sequence. Per ADR 0003, the terminal
    // reporter is responsible for this escaping.
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    };
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "attacker payload: пример\x1b[31mRED\x1b[0m".into(),
        evidence: vec!["raw evidence: \x1b[32mGREEN".into()],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    }];

    let output = reporter.render(&summary, &findings, &[]);

    assert!(
        !output.as_bytes().contains(&0x1b),
        "terminal output must not contain raw ESC (0x1b) bytes, got: {output:?}"
    );
    assert!(
        output.contains("\\u{1b}[31mRED"),
        "terminal output should contain escaped form of ESC sequence in summary, got: {output}"
    );
    assert!(
        output.contains("\\u{1b}[32mGREEN"),
        "terminal output should contain escaped form in evidence line, got: {output}"
    );
    assert!(
        output.contains("пример"),
        "terminal output should preserve Cyrillic in the summary, got: {output}"
    );
}

#[test]
fn test_output_sanitization_layering_contract() {
    // End-to-end contract test for ADR 0003. A single Finding flows through
    // the data layer and both reporters; all three assertions must hold:
    //   1. The struct itself keeps the raw ESC byte (forensic layer).
    //   2. The terminal reporter escapes the ESC byte (terminal display layer).
    //   3. The JSON reporter escapes via serde's RFC 8259 \u001b form (JSON layer).
    //
    // Any future regression that breaks one of these — e.g., re-introducing
    // construction-site escaping, removing the terminal reporter's helper,
    // or swapping to a JSON crate that doesn't escape control chars — will
    // fail this test.
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "attacker payload: \x1b[31mRED\x1b[0m".into(),
        evidence: vec!["ev: \x1b[32mGREEN".into()],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    };

    // Layer 1: the struct preserves the raw ESC byte (forensic ground truth).
    assert!(
        finding.summary.as_bytes().contains(&0x1b),
        "Finding.summary must preserve raw ESC for forensics"
    );
    assert!(
        finding.evidence[0].as_bytes().contains(&0x1b),
        "Finding.evidence must preserve raw ESC for forensics"
    );

    // Layer 2: terminal reporter escapes on display.
    let terminal_output = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    }
    .render(&Summary::new(), std::slice::from_ref(&finding), &[]);
    assert!(
        !terminal_output.as_bytes().contains(&0x1b),
        "terminal reporter must not emit raw ESC bytes, got: {terminal_output:?}"
    );
    assert!(
        terminal_output.contains("\\u{1b}[31mRED"),
        "terminal reporter should emit the escaped summary form, got: {terminal_output}"
    );
    assert!(
        terminal_output.contains("\\u{1b}[32mGREEN"),
        "terminal reporter should emit the escaped evidence form, got: {terminal_output}"
    );

    // Layer 3: JSON reporter escapes via serde's RFC 8259 \u001b form.
    let json_output = JsonReporter.render(&Summary::new(), std::slice::from_ref(&finding), &[]);
    assert!(
        !json_output.as_bytes().contains(&0x1b),
        "JSON reporter must not emit raw ESC bytes, got: {json_output:?}"
    );
    assert!(
        json_output.contains("\\u001b"),
        "JSON reporter should serialize ESC as \\u001b per RFC 8259, got: {json_output}"
    );
    // Round-trip through serde_json::from_str: the deserialized summary
    // must match the original raw ESC byte. This proves the JSON escape
    // is reversible, which is what downstream tooling relies on.
    let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    let parsed_summary = parsed["findings"][0]["summary"].as_str().unwrap();
    assert_eq!(parsed_summary, finding.summary);
}

#[test]
fn test_json_reporter_preserves_cyrillic_as_readable_unicode() {
    // ADR 0003's primary user-visible behavioral claim: JSON consumers see
    // raw Cyrillic (and other non-ASCII Unicode) hostnames instead of the
    // hex-mangled forms PR #49 produced (`\u{43f}\u{440}...`). This test
    // pins that contract: any future regression that re-introduces
    // construction-site Debug-formatting or swaps to a non-serde JSON
    // writer that pre-escapes non-ASCII will fail here.
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "TLS SNI non-ASCII: пример.рф".into(),
        evidence: vec!["hex: d0bfd180d0b8d0bcd0b5d1802ed180d184".into()],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    };

    let json_output = JsonReporter.render(&Summary::new(), std::slice::from_ref(&finding), &[]);

    // Readable Cyrillic must appear in the JSON output — serde_json preserves
    // non-ASCII Unicode by default and only escapes control characters.
    assert!(
        json_output.contains("пример.рф"),
        "JSON output must contain readable Cyrillic, got: {json_output}"
    );
    // And the old hex-escaped form must NOT appear — that would indicate a
    // regression to construction-site escaping.
    assert!(
        !json_output.contains("\\u{43f}"),
        "JSON output must not contain Debug-formatted Cyrillic escape (regression to construction-site), got: {json_output}"
    );

    // Round-trip: deserialize the JSON and confirm the summary matches
    // byte-for-byte — proves the Cyrillic survives the full trip.
    let parsed: serde_json::Value = serde_json::from_str(&json_output).unwrap();
    let parsed_summary = parsed["findings"][0]["summary"].as_str().unwrap();
    assert_eq!(parsed_summary, finding.summary);
}

#[test]
fn test_terminal_reporter_escapes_control_bytes_in_analyzer_summaries() {
    // Regression: analyzer_summaries detail values can contain
    // attacker-controlled strings (HTTP top_hosts, TLS top_snis, etc.).
    // serde_json::Value's Display impl escapes C0 (per RFC 8259) and DEL but
    // passes C1 codepoints (U+0080-U+009F) through as raw UTF-8 — which
    // is a terminal injection vector on the analyzer summary rendering
    // path. Per ADR 0003, the terminal reporter must escape at the
    // display boundary regardless of what the underlying serializer does.
    use wirerust::analyzer::AnalysisSummary;

    let mut detail = std::collections::HashMap::new();
    detail.insert(
        "top_snis".to_string(),
        serde_json::json!([
            "\u{1b}[31mREDC0\u{1b}[0m", // C0 ESC injection
            "before\u{9b}31mC1after", // C1 CSI injection (valid UTF-8, bypasses serde's RFC 8259 escape)
            "пример.рф",              // legitimate Cyrillic — must survive readably
        ]),
    );
    let analyzer_summary = AnalysisSummary {
        analyzer_name: "TLS".to_string(),
        packets_analyzed: 3,
        detail,
    };

    let output = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    }
    .render(
        &Summary::new(),
        &[],
        std::slice::from_ref(&analyzer_summary),
    );

    // No raw C0 ESC bytes in the output.
    assert!(
        !output.as_bytes().contains(&0x1b),
        "terminal output must not contain raw ESC (0x1b) bytes in analyzer summary section, got: {output:?}"
    );
    // No raw C1 bytes (0xC2 0x9B encodes U+009B in UTF-8).
    assert!(
        !output.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
        "terminal output must not contain raw C1 UTF-8 bytes (0xC2 0x9B) in analyzer summary section, got: {output:?}"
    );
    // Cyrillic must still be readable — the escape must not mangle valid Unicode.
    assert!(
        output.contains("пример.рф"),
        "analyzer summary section must preserve legitimate Cyrillic, got: {output}"
    );
}

// ---------------------------------------------------------------------------
// End-to-end: HttpAnalyzer → reporter pipeline (issue #56)
//
// These tests close the coverage gap identified during the ADR 0003 PR review:
// the existing contract tests above use synthetic Findings, not ones produced
// by the actual HttpAnalyzer. These tests drive HttpAnalyzer::on_data with
// crafted HTTP requests and verify the full pipeline.
//
// Key discovery during issue validation: httparse rejects C0 control bytes
// (including ESC 0x1b) in URIs and header values, but ACCEPTS C1 codepoints
// (U+0080-U+009F) because they encode as high bytes in UTF-8 (e.g., U+009B
// CSI = 0xC2 0x9B). C1 CSI is the real injection vector through httparse.
// ---------------------------------------------------------------------------

fn http_test_flow_key() -> FlowKey {
    FlowKey::new(
        "10.0.0.1".parse::<IpAddr>().unwrap(),
        49153,
        "10.0.0.2".parse::<IpAddr>().unwrap(),
        80,
    )
}

/// Build an HTTP/1.1 request with a path-traversal URI containing C1 CSI
/// (U+009B) — the 8-bit equivalent of ESC[. httparse accepts this because
/// the UTF-8 encoding (0xC2 0x9B) consists of high bytes (≥ 0x80).
fn build_path_traversal_with_c1_csi() -> Vec<u8> {
    let mut buf = b"GET /../../etc/passwd".to_vec();
    buf.extend_from_slice(&[0xC2, 0x9B]); // U+009B CSI
    buf.extend_from_slice(b"31mHACKED HTTP/1.1\r\nHost: target.com\r\n\r\n");
    buf
}

/// Build an HTTP/1.1 request with C1 CSI in the Host header value.
fn build_request_with_c1_in_host() -> Vec<u8> {
    let mut buf = b"GET /index HTTP/1.1\r\nHost: evil".to_vec();
    buf.extend_from_slice(&[0xC2, 0x9B]); // U+009B CSI
    buf.extend_from_slice(b"31m.com\r\nUser-Agent: Mozilla/5.0\r\n\r\n");
    buf
}

#[test]
fn test_http_finding_c1_csi_escaped_by_terminal_reporter() {
    // End-to-end: HttpAnalyzer produces a path-traversal finding whose
    // summary contains a raw C1 CSI (U+009B). The terminal reporter must
    // escape it. This exercises the real injection path — httparse accepts
    // C1 in URIs because the UTF-8 encoding uses high bytes.
    let mut analyzer = HttpAnalyzer::new();
    let fk = http_test_flow_key();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_path_traversal_with_c1_csi(),
        0,
    );

    let findings = analyzer.findings();
    assert!(
        !findings.is_empty(),
        "path traversal with C1 CSI should produce at least one finding"
    );

    // The finding's raw summary must contain the C1 CSI bytes (forensic preservation).
    let traversal_finding = findings
        .iter()
        .find(|f| f.summary.contains("Path traversal"))
        .expect("expected a path-traversal finding");
    assert!(
        traversal_finding
            .summary
            .as_bytes()
            .windows(2)
            .any(|w| w == [0xC2, 0x9B]),
        "Finding.summary must preserve raw C1 CSI for forensics, got: {:?}",
        traversal_finding.summary
    );

    // Render through terminal reporter — no raw C1 bytes in output.
    let output = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    }
    .render(&Summary::new(), &findings, &[]);
    assert!(
        !output.as_bytes().windows(2).any(|w| w == [0xC2, 0x9B]),
        "terminal output must not contain raw C1 CSI (0xC2 0x9B), got: {output:?}"
    );
    assert!(
        output.contains("\\u{9b}"),
        "terminal output should contain the escaped form of C1 CSI, got: {output}"
    );
}

#[test]
fn test_http_finding_c1_csi_in_json_reporter() {
    // The JSON reporter renders findings from HttpAnalyzer. serde_json does
    // NOT escape C1 codepoints (RFC 8259 only mandates C0; serde_json also escapes DEL), so the
    // raw C1 CSI UTF-8 bytes pass through. This test verifies the JSON
    // round-trip preserves the C1 byte — downstream tools can reconstruct
    // the original payload.
    let mut analyzer = HttpAnalyzer::new();
    let fk = http_test_flow_key();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_path_traversal_with_c1_csi(),
        0,
    );

    let findings = analyzer.findings();
    let json_output = JsonReporter.render(&Summary::new(), &findings, &[]);

    // JSON must be valid.
    let parsed: serde_json::Value =
        serde_json::from_str(&json_output).expect("JSON output must be valid");

    // Round-trip: the deserialized finding summary must contain the C1 CSI
    // codepoint, proving the JSON encoding preserved it.
    let json_findings = parsed["findings"].as_array().unwrap();
    let traversal = json_findings
        .iter()
        .find(|f| {
            f["summary"]
                .as_str()
                .is_some_and(|s| s.contains("Path traversal"))
        })
        .expect("expected path-traversal finding in JSON");
    let summary_str = traversal["summary"].as_str().unwrap();
    assert!(
        summary_str.as_bytes().windows(2).any(|w| w == [0xC2, 0x9B]),
        "JSON round-trip must preserve raw C1 CSI in finding summary, got: {summary_str:?}"
    );
}

#[test]
fn test_http_analyzer_summary_c1_csi_escaped_by_terminal_reporter() {
    // End-to-end: HttpAnalyzer accumulates a Host header containing C1 CSI
    // into its top_hosts summary. When rendered through the terminal
    // reporter's analyzer-summary section, the C1 must be escaped.
    let mut analyzer = HttpAnalyzer::new();
    let fk = http_test_flow_key();
    analyzer.on_data(
        &fk,
        Direction::ClientToServer,
        &build_request_with_c1_in_host(),
        0,
    );

    // Verify the host made it into the analyzer summary.
    let analyzer_summary = analyzer.summarize();
    let top_hosts_str = analyzer_summary.detail["top_hosts"].to_string();
    assert!(
        top_hosts_str
            .as_bytes()
            .windows(2)
            .any(|w| w == [0xC2, 0x9B]),
        "analyzer summary top_hosts must contain raw C1 CSI, got: {top_hosts_str:?}"
    );

    // Render through terminal reporter — no raw C1 bytes in output.
    let output = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    }
    .render(
        &Summary::new(),
        &[],
        std::slice::from_ref(&analyzer_summary),
    );
    assert!(
        !output.as_bytes().windows(2).any(|w| w == [0xC2, 0x9B]),
        "terminal output must not contain raw C1 CSI in analyzer summary section, got: {output:?}"
    );
    assert!(
        output.contains("\\u{9b}"),
        "terminal output should contain the escaped form of C1 CSI in analyzer summary, got: {output}"
    );
}

// ---------------------------------------------------------------------------
// MITRE grouping tests
// ---------------------------------------------------------------------------

fn base_finding_with_mitre(
    technique: Option<&str>,
    verdict: Verdict,
    confidence: Confidence,
    summary: &str,
) -> Finding {
    Finding {
        category: ThreatCategory::Anomaly,
        verdict,
        confidence,
        summary: summary.to_string(),
        evidence: vec![],
        mitre_technique: technique.map(|s| s.to_string()),
        source_ip: None,
        timestamp: None,
    }
}

#[test]
fn mitre_grouping_emits_tactic_headers_in_canonical_order() {
    let findings = vec![
        base_finding_with_mitre(Some("T1499.002"), Verdict::Likely, Confidence::High, "dos"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "scan"),
        base_finding_with_mitre(Some("T0855"), Verdict::Likely, Confidence::High, "ics"),
    ];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: true,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    // Anchor on the `## ` header prefix so future summary/evidence text
    // containing the tactic word cannot confuse the ordering check.
    let discovery_pos = out.find("## Discovery").expect("missing Discovery header");
    let impact_pos = out.find("## Impact").expect("missing Impact header");
    let ics_pos = out
        .find("## Impair Process Control")
        .expect("missing ICS header");
    assert!(
        discovery_pos < impact_pos,
        "Discovery must come before Impact"
    );
    assert!(impact_pos < ics_pos, "Impact must come before ICS tactics");
}

#[test]
fn mitre_grouping_sorts_within_tactic_by_verdict_then_confidence() {
    let findings = vec![
        base_finding_with_mitre(Some("T1046"), Verdict::Unlikely, Confidence::High, "third"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::Medium, "second"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "first"),
        base_finding_with_mitre(
            Some("T1046"),
            Verdict::Inconclusive,
            Confidence::Low,
            "fourth_ish",
        ),
    ];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: true,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    let p1 = out.find("first").expect("first missing");
    let p2 = out.find("second").expect("second missing");
    let p3 = out.find("fourth_ish").expect("fourth_ish missing");
    let p4 = out.find("third").expect("third missing");
    assert!(
        p1 < p2 && p2 < p3 && p3 < p4,
        "verdict/confidence sort wrong: {out}"
    );
}

#[test]
fn mitre_grouping_buckets_none_and_unknown_under_uncategorized() {
    let findings = vec![
        base_finding_with_mitre(None, Verdict::Likely, Confidence::High, "no_id_finding"),
        base_finding_with_mitre(
            Some("T9999"),
            Verdict::Likely,
            Confidence::High,
            "unknown_id_finding",
        ),
        base_finding_with_mitre(
            Some("T1046"),
            Verdict::Likely,
            Confidence::High,
            "known_finding",
        ),
    ];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: true,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    let uncat_pos = out
        .find("## Uncategorized")
        .expect("missing Uncategorized section");
    let no_id_pos = out.find("no_id_finding").expect("missing no-id finding");
    let unknown_pos = out
        .find("unknown_id_finding")
        .expect("missing unknown-id finding");
    let known_pos = out.find("known_finding").expect("missing known finding");
    assert!(
        known_pos < uncat_pos,
        "Uncategorized must come after known tactics"
    );
    assert!(uncat_pos < no_id_pos && uncat_pos < unknown_pos);
    assert!(
        out.contains("T9999 (unknown)"),
        "unknown ID must render with '(unknown)' label"
    );
}

#[test]
fn mitre_grouping_expands_per_finding_line_with_technique_name() {
    let findings = vec![base_finding_with_mitre(
        Some("T1046"),
        Verdict::Likely,
        Confidence::High,
        "scan",
    )];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: true,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    assert!(
        out.contains("MITRE: T1046 \u{2014} Network Service Discovery"),
        "expected em-dash-expanded MITRE line, got: {out}",
    );
}

#[test]
fn default_rendering_unchanged_when_mitre_flag_off() {
    let findings = vec![base_finding_with_mitre(
        Some("T1046"),
        Verdict::Likely,
        Confidence::High,
        "scan",
    )];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    assert!(out.contains("MITRE: T1046"));
    assert!(
        !out.contains("\u{2014}"),
        "em-dash should not appear in default render"
    );
    assert!(!out.contains("## Uncategorized"));
}

#[test]
fn mitre_grouping_preserves_emission_order_when_verdict_and_confidence_tie() {
    // All four findings are in the same tactic with identical verdict +
    // confidence — the tertiary emission-order key is the only thing
    // that can distinguish them.
    let findings = vec![
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "alpha"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "bravo"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "charlie"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "delta"),
    ];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: true,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    let pa = out.find("alpha").expect("alpha missing");
    let pb = out.find("bravo").expect("bravo missing");
    let pc = out.find("charlie").expect("charlie missing");
    let pd = out.find("delta").expect("delta missing");
    assert!(
        pa < pb && pb < pc && pc < pd,
        "stable tiebreaker should preserve emission order: {out}",
    );
}

#[test]
fn mitre_grouping_keeps_known_and_unknown_ids_in_separate_buckets() {
    // Enterprise T1046 is Discovery; T1046.999 isn't seeded and so falls
    // to Uncategorized. A finding with a known ID and one with a typo'd
    // variant of the same family must not end up together.
    let findings = vec![
        base_finding_with_mitre(
            Some("T1046"),
            Verdict::Likely,
            Confidence::High,
            "known_discovery",
        ),
        base_finding_with_mitre(
            Some("T1046.999"),
            Verdict::Likely,
            Confidence::High,
            "typo_variant",
        ),
    ];
    let reporter = TerminalReporter {
        use_color: false,
        show_mitre_grouping: true,
        show_hosts_breakdown: false,
    };
    let out = reporter.render(&Summary::new(), &findings, &[]);
    let discovery_pos = out.find("## Discovery").expect("Discovery header missing");
    let uncat_pos = out
        .find("## Uncategorized")
        .expect("Uncategorized header missing");
    let known_pos = out.find("known_discovery").expect("known finding missing");
    let typo_pos = out.find("typo_variant").expect("typo finding missing");
    assert!(
        discovery_pos < known_pos && known_pos < uncat_pos,
        "known_discovery must render under the Discovery header",
    );
    assert!(
        uncat_pos < typo_pos,
        "typo_variant must render under Uncategorized",
    );
    assert!(
        out.contains("T1046.999 (unknown)"),
        "typo variant must render with '(unknown)' label",
    );
}
