use std::net::IpAddr;
use wirerust::analyzer::http::HttpAnalyzer;
use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reassembly::flow::FlowKey;
use wirerust::reassembly::handler::{Direction, StreamAnalyzer, StreamHandler};
use wirerust::reporter::Reporter;
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
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
        mitre_techniques: vec!["T1046".into()],
        source_ip: None,
        timestamp: None,
        direction: None,
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
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    }];
    let output = reporter.render(&summary, &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = &parsed["findings"][0];
    let obj = finding.as_object().expect("finding must be a JSON object");
    assert!(
        !obj.contains_key("mitre_techniques"),
        "absent mitre_techniques must be omitted (Vec::is_empty), got: {finding}"
    );
    assert!(
        !obj.contains_key("source_ip"),
        "absent source_ip must be omitted, got: {finding}"
    );
    assert!(
        !obj.contains_key("timestamp"),
        "absent timestamp must be omitted, got: {finding}"
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
        mitre_techniques: vec!["T1036".into()],
        source_ip: Some(IpAddr::V4(std::net::Ipv4Addr::new(10, 0, 0, 1))),
        timestamp: None, // intentionally None — verifies mixed presence
        direction: None,
    }];
    let output = reporter.render(&summary, &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = &parsed["findings"][0];
    let obj = finding.as_object().expect("finding must be a JSON object");
    // STORY-100 BC-2.09.006: mitre_techniques is now a JSON array.
    assert_eq!(
        obj.get("mitre_techniques"),
        Some(&serde_json::json!(["T1036"]))
    );
    assert_eq!(obj.get("source_ip"), Some(&serde_json::json!("10.0.0.1")));
    assert!(
        !obj.contains_key("timestamp"),
        "the still-None timestamp must remain omitted"
    );
}

// ---- LESSON-P2.03: CSV reporter ----

#[test]
fn test_csv_reporter_emits_header_and_one_row_per_finding() {
    use wirerust::reporter::csv::CsvReporter;

    let reporter = CsvReporter;
    let findings = vec![
        Finding {
            category: ThreatCategory::Reconnaissance,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "first finding".into(),
            evidence: vec!["ev-a".into(), "ev-b".into()],
            mitre_techniques: vec!["T1046".into()],
            source_ip: None,
            timestamp: None,
            direction: None,
        },
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: "second finding".into(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        },
    ];
    let out = reporter.render(&Summary::new(), &findings, &[]);
    let lines: Vec<&str> = out.lines().collect();
    // STORY-101 BC-2.11.020: column 6 header is mitre_techniques (not mitre_technique).
    assert_eq!(
        lines[0],
        "category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp"
    );
    // Header + 2 data rows.
    assert_eq!(lines.len(), 3, "expected header + 2 rows, got: {out}");
    assert!(lines[1].contains("first finding"));
    assert!(lines[1].contains("T1046"));
    // Multi-value evidence is flattened into one cell with "; ".
    assert!(lines[1].contains("ev-a; ev-b"));
    assert!(lines[2].contains("second finding"));
}

#[test]
fn test_csv_reporter_empty_findings_emits_header_only() {
    use wirerust::reporter::csv::CsvReporter;

    let reporter = CsvReporter;
    let out = reporter.render(&Summary::new(), &[], &[]);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 1, "empty findings → header row only");
    assert!(lines[0].starts_with("category,verdict,confidence,"));
}

#[test]
fn test_csv_reporter_neutralizes_formula_injection() {
    // LESSON-P2.03 security: a Finding summary controlled by an
    // attacker that begins with a spreadsheet formula-trigger char
    // (`=`, `+`, `-`, `@`) must be neutralized with a leading `'` so
    // that opening the CSV in Excel/Sheets does not execute it.
    use wirerust::reporter::csv::CsvReporter;

    let reporter = CsvReporter;
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        // Classic CSV-injection payload.
        summary: "=cmd|'/c calc'!A1".into(),
        evidence: vec!["@SUM(1+1)".into()],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    }];
    let out = reporter.render(&Summary::new(), &findings, &[]);
    // Parse the CSV back and check the field values directly.
    let mut rdr = csv::Reader::from_reader(out.as_bytes());
    let record = rdr
        .records()
        .next()
        .expect("one data row")
        .expect("valid CSV record");
    let summary_cell = &record[3];
    let evidence_cell = &record[4];
    assert!(
        summary_cell.starts_with('\''),
        "formula-trigger summary must be neutralized with a leading quote, got: {summary_cell:?}"
    );
    assert!(
        evidence_cell.starts_with('\''),
        "formula-trigger evidence must be neutralized, got: {evidence_cell:?}"
    );
}

#[test]
fn test_csv_reporter_ignores_summary_and_analyzer_data() {
    // LESSON-P2.03 documented scope: the CSV reporter renders the
    // findings table ONLY. A populated Summary / AnalysisSummary must
    // not leak into the CSV output.
    use wirerust::analyzer::AnalysisSummary;
    use wirerust::reporter::csv::CsvReporter;

    let mut summary = Summary::new();
    summary.skipped_packets = 999;
    let mut detail: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();
    detail.insert(
        "sentinel_key".to_string(),
        serde_json::json!("sentinel_val"),
    );
    let analyzer = AnalysisSummary {
        analyzer_name: "sentinel_analyzer".to_string(),
        packets_analyzed: 7,
        detail,
    };

    let reporter = CsvReporter;
    let out = reporter.render(&summary, &[], &[analyzer]);
    assert!(!out.contains("999"), "summary data must not appear in CSV");
    assert!(
        !out.contains("sentinel_key") && !out.contains("sentinel_analyzer"),
        "analyzer-summary data must not appear in CSV; got: {out}"
    );
}

// ---- LESSON-P2.08: direction tag on Finding ----

#[test]
fn test_json_finding_emits_direction_when_set() {
    // LESSON-P2.08: a Finding with `direction: Some(Direction::*)` must
    // surface the value in JSON under a `"direction"` key. Verifies the
    // serde Serialize wiring on the Direction enum.
    use wirerust::reassembly::handler::Direction;

    let reporter = JsonReporter;
    let summary = Summary::new();
    let findings = vec![
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: "client-side finding".into(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: Some(Direction::ClientToServer),
        },
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: "server-side finding".into(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: Some(Direction::ServerToClient),
        },
    ];
    let output = reporter.render(&summary, &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        parsed["findings"][0]["direction"],
        serde_json::json!("ClientToServer")
    );
    assert_eq!(
        parsed["findings"][1]["direction"],
        serde_json::json!("ServerToClient")
    );
}

#[test]
fn test_json_finding_omits_direction_when_none() {
    // LESSON-P2.08 companion to P1.02: absent direction must be
    // omitted from the JSON object (skip_serializing_if), not
    // emitted as `null`.
    let reporter = JsonReporter;
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::Medium,
        summary: "directionless finding".into(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    }];
    let output = reporter.render(&summary, &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();
    let finding = parsed["findings"][0].as_object().expect("object");
    assert!(
        !finding.contains_key("direction"),
        "absent direction must be omitted from JSON, got: {}",
        parsed["findings"][0]
    );
}

// ---- LESSON-P2.09: deterministic JSON map ordering ----

#[test]
fn test_json_reporter_emits_deterministic_protocols_ordering() {
    // LESSON-P2.09 / NFR DET-001: top-level map keys in the JSON
    // output (protocols, services) must serialize in deterministic
    // alphabetical order, regardless of underlying HashMap iteration
    // order. Snapshot / golden tests over the JSON depend on this.
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};

    fn make_packet(proto: Protocol, src_port: u16, dst_port: u16) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            protocol: proto,
            transport: match proto {
                Protocol::Tcp => TransportInfo::Tcp {
                    src_port,
                    dst_port,
                    seq_number: 0,
                    syn: false,
                    ack: false,
                    fin: false,
                    rst: false,
                },
                Protocol::Udp => TransportInfo::Udp { src_port, dst_port },
                _ => TransportInfo::None,
            },
            payload: vec![],
            packet_len: 60,
        }
    }

    let mut summary = Summary::new();
    // Insert in a deliberately non-alphabetical order to exercise the sort.
    summary.ingest(&make_packet(Protocol::Udp, 53, 0));
    summary.ingest(&make_packet(Protocol::Tcp, 0, 443));
    summary.ingest(&make_packet(Protocol::Icmp, 0, 0));
    summary.ingest(&make_packet(Protocol::Tcp, 0, 80));

    let reporter = JsonReporter;
    // Render twice; output must be byte-identical (HashMap iteration
    // order would otherwise diverge across calls).
    let out_a = reporter.render(&summary, &[], &[]);
    let out_b = reporter.render(&summary, &[], &[]);
    assert_eq!(
        out_a, out_b,
        "two render() calls on the same summary must produce identical bytes"
    );

    // Protocol keys must appear in alphabetical order.
    let icmp_pos = out_a.find("\"Icmp\"").expect("Icmp key in output");
    let tcp_pos = out_a.find("\"Tcp\"").expect("Tcp key in output");
    let udp_pos = out_a.find("\"Udp\"").expect("Udp key in output");
    assert!(
        icmp_pos < tcp_pos && tcp_pos < udp_pos,
        "protocol keys must be sorted alphabetically (Icmp < Tcp < Udp) — got positions {icmp_pos}, {tcp_pos}, {udp_pos}"
    );

    // Service keys (DNS, HTTP, TLS) must appear alphabetically.
    let dns_pos = out_a.find("\"DNS\"").expect("DNS service in output");
    let http_pos = out_a.find("\"HTTP\"").expect("HTTP service in output");
    let tls_pos = out_a.find("\"TLS\"").expect("TLS service in output");
    assert!(
        dns_pos < http_pos && http_pos < tls_pos,
        "service keys must be sorted alphabetically (DNS < HTTP < TLS) — got positions {dns_pos}, {http_pos}, {tls_pos}"
    );
}

#[test]
fn test_json_reporter_emits_deterministic_analyzer_detail_ordering() {
    // LESSON-P2.09: AnalysisSummary::detail is a BTreeMap, so its
    // keys serialize alphabetically. Catches a regression that would
    // change the field back to HashMap.
    use wirerust::analyzer::AnalysisSummary;

    let mut detail: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();
    detail.insert("zeta_key".to_string(), serde_json::json!(1));
    detail.insert("alpha_key".to_string(), serde_json::json!(2));
    detail.insert("mu_key".to_string(), serde_json::json!(3));
    let analyzer_summary = AnalysisSummary {
        analyzer_name: "test".to_string(),
        packets_analyzed: 0,
        detail,
    };

    let reporter = JsonReporter;
    let out = reporter.render(&Summary::new(), &[], &[analyzer_summary]);
    let alpha = out.find("alpha_key").expect("alpha_key in output");
    let mu = out.find("mu_key").expect("mu_key in output");
    let zeta = out.find("zeta_key").expect("zeta_key in output");
    assert!(
        alpha < mu && mu < zeta,
        "analyzer detail keys must serialize alphabetically (alpha < mu < zeta) — got positions {alpha}, {mu}, {zeta}"
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: true,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
    };
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "attacker payload: пример\x1b[31mRED\x1b[0m".into(),
        evidence: vec!["raw evidence: \x1b[32mGREEN".into()],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
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

/// STORY-070 AC-003 (BC-2.09.005 postcondition 4): JSON output produced by
/// `JsonReporter` contains the raw bytes escaped only per RFC 8259 by serde_json
/// (ESC 0x1B appears as `\u001b` in JSON, not the literal byte). This test is
/// the canonical coverage point for STORY-070 AC-003 — it predates STORY-070 and
/// satisfies the requirement in full. See STORY-070 §AC-003 for traceability.
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
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
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

    let mut detail: std::collections::BTreeMap<String, serde_json::Value> =
        std::collections::BTreeMap::new();
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        mitre_techniques: technique.map(|s| vec![s.to_string()]).unwrap_or_default(),
        source_ip: None,
        timestamp: None,
        direction: None,
    }
}

#[test]
fn mitre_grouping_emits_tactic_headers_in_canonical_order() {
    let findings = vec![
        base_finding_with_mitre(Some("T1499.002"), Verdict::Likely, Confidence::High, "dos"),
        base_finding_with_mitre(Some("T1046"), Verdict::Likely, Confidence::High, "scan"),
        base_finding_with_mitre(Some("T1692.001"), Verdict::Likely, Confidence::High, "ics"),
    ];
    let reporter = TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Grouped, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Grouped, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Grouped, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Grouped, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Grouped, Collapse::Expanded),
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
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Grouped, Collapse::Expanded),
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

// ---------------------------------------------------------------------------
// STORY-069: Finding Struct, Verdict/Confidence Display, and Finding Display
// Format
//
// These tests cover AC-001 through AC-011 and EC-001 through EC-005.
//
// implementation_strategy: brownfield-formalization
// All tests are expected to PASS (brownfield-confirm) because the implementation
// already exists. Any FAIL indicates a real gap where existing code does not
// satisfy the BC.
// ---------------------------------------------------------------------------

// --- AC-001 / BC-2.09.001 postcondition 1 ---

/// AC-001 (BC-2.09.001 postcondition 1): A `Finding` constructed with all
/// required and optional fields compiles and holds the expected values.
#[test]
fn test_finding_construction_with_all_fields() {
    use std::net::{IpAddr, Ipv4Addr};
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reassembly::handler::Direction;

    let ip: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1));
    let f = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "test summary".to_string(),
        evidence: vec!["ev1".to_string(), "ev2".to_string()],
        mitre_techniques: vec!["T1027".to_string()],
        source_ip: Some(ip),
        timestamp: None,
        direction: Some(Direction::ClientToServer),
    };

    assert!(matches!(f.category, ThreatCategory::Anomaly));
    assert!(matches!(f.verdict, Verdict::Likely));
    assert!(matches!(f.confidence, Confidence::High));
    assert_eq!(f.summary, "test summary");
    assert_eq!(f.evidence.len(), 2);
    // STORY-100 BC-2.09.001: mitre_techniques is Vec<String> — full-vec equality.
    assert_eq!(f.mitre_techniques, vec!["T1027".to_string()]);
    assert_eq!(f.source_ip, Some(ip));
    assert!(
        f.timestamp.is_none(),
        "BC-2.09.001 invariant 1: timestamp must be None at all emission sites (O-01)"
    );
    assert!(matches!(f.direction, Some(Direction::ClientToServer)));
}

// --- AC-002 / BC-2.09.001 invariant 1 (in-process file scan) ---

/// Helper: read a source file, asserting it exists. A missing file is a test
/// failure — a silent missing file would cause false-passes on zero-match
/// assertions.
fn read_src_file(manifest_dir: &str, rel_path: &str) -> String {
    let path = std::path::Path::new(manifest_dir)
        .join("src")
        .join(rel_path);
    std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "source file '{}' must exist and be readable — \
             a missing file would cause false-passes on zero-match assertions: {e}",
            path.display()
        )
    })
}

/// AC-002 (BC-2.09.001 invariant 1): All 22 Finding emission sites in
/// production source set `timestamp: None`. No site sets `timestamp: Some(...)`.
///
/// Uses in-process `std::fs::read_to_string` — no dependency on grep being on
/// PATH, and a missing file fails the test rather than silently passing.
///
/// Post-STORY-098 (BC-2.09.007): 21 of 22 sites now set `timestamp: Some(DateTime<Utc>)`;
/// exactly 1 site retains `timestamp: None` (the segment-limit summary in `finalize`,
/// which is a post-capture aggregate not tied to any specific packet — see BC-2.09.007
/// invariant 1, postcondition 4).
///
/// This test verifies the post-STORY-098 invariant: exactly 1 None and 21 Some-form
/// occurrences across all emission-site files.
#[test]
fn test_timestamp_always_none_in_all_emission_sites() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    // All four files that contain Finding emission sites.
    let emission_files = [
        "analyzer/http.rs",
        "analyzer/tls.rs",
        "reassembly/mod.rs",
        "reassembly/lifecycle.rs",
    ];

    let mut total_none_count: usize = 0;
    let mut total_some_count: usize = 0;

    for rel_path in &emission_files {
        let src = read_src_file(manifest_dir, rel_path);
        total_none_count += src.matches("timestamp: None").count();
        // Count `DateTime::from_timestamp` as the canonical Some-form pattern
        // used at all 21 populated emission sites (BC-2.09.007 post-1).
        total_some_count += src.matches("DateTime::from_timestamp").count();
    }

    // Post-STORY-098 (BC-2.09.007 invariant 1): exactly 1 None site
    // (the segment-limit summary in finalize) and 21 Some sites.
    assert_eq!(
        total_none_count, 1,
        "BC-2.09.007 invariant 1: expected exactly 1 `timestamp: None` occurrence \
         across emission-site files (segment-limit summary in finalize only), \
         found {total_none_count}."
    );
    assert_eq!(
        total_some_count, 21,
        "BC-2.09.007 invariant 1: expected exactly 21 `DateTime::from_timestamp` \
         occurrences across emission-site files (21 of 22 sites set Some timestamp), \
         found {total_some_count}."
    );
}

// --- AC-003 / BC-2.09.001 invariant 2 (in-process file scan, two tests) ---

/// AC-003a (BC-2.09.001 invariant 2): Reassembly anomaly findings set
/// `source_ip: Some(...)` at EXACTLY 5 sites:
///   reassembly/mod.rs lines 443, 481, 505 (overlap, small-segment, out-of-window)
///   reassembly/lifecycle.rs lines 112, 132 (conflicting-overlap, stream-depth-exceeded)
///
/// Uses in-process file reading; a missing file fails the test.
#[test]
fn test_source_ip_set_at_reassembly_sites() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mod_src = read_src_file(manifest_dir, "reassembly/mod.rs");
    let lifecycle_src = read_src_file(manifest_dir, "reassembly/lifecycle.rs");

    let mod_some_count = mod_src.matches("source_ip: Some").count();
    let lifecycle_some_count = lifecycle_src.matches("source_ip: Some").count();
    let total = mod_some_count + lifecycle_some_count;

    // reassembly/mod.rs must have exactly 3 Some sites.
    assert_eq!(
        mod_some_count, 3,
        "BC-2.09.001 invariant 2: reassembly/mod.rs must have exactly 3 \
         `source_ip: Some(...)` emission sites (overlap, small-segment, out-of-window), \
         found {mod_some_count}"
    );

    // reassembly/lifecycle.rs must have exactly 2 Some sites.
    assert_eq!(
        lifecycle_some_count, 2,
        "BC-2.09.001 invariant 2: reassembly/lifecycle.rs must have exactly 2 \
         `source_ip: Some(...)` emission sites (conflicting-overlap, stream-depth-exceeded), \
         found {lifecycle_some_count}"
    );

    // Total across both files must be exactly 5.
    assert_eq!(
        total, 5,
        "BC-2.09.001 invariant 2: expected exactly 5 reassembly `source_ip: Some(...)` \
         sites across mod.rs + lifecycle.rs, found {total}"
    );
}

/// AC-003b (BC-2.09.001 invariant 2): HTTP and TLS analyzer findings set
/// `source_ip: None` at every emission site — no HTTP or TLS site uses
/// `source_ip: Some(...)`.
///
/// Uses in-process file reading; a missing file fails the test.
#[test]
fn test_source_ip_none_at_http_tls_sites() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let http_src = read_src_file(manifest_dir, "analyzer/http.rs");
    let tls_src = read_src_file(manifest_dir, "analyzer/tls.rs");

    let http_some_count = http_src.matches("source_ip: Some").count();
    assert_eq!(
        http_some_count, 0,
        "BC-2.09.001 invariant 2 violated: analyzer/http.rs must not set \
         source_ip: Some(...) at any emission site, found {http_some_count} occurrence(s)"
    );

    // Positive: exactly 9 HTTP emission sites set source_ip: None.
    let http_none_count = http_src.matches("source_ip: None").count();
    assert_eq!(
        http_none_count, 9,
        "BC-2.09.001 invariant 2: analyzer/http.rs must have exactly 9 \
         `source_ip: None` emission sites, found {http_none_count}. \
         A new emission site was added or removed without updating this test."
    );

    let tls_some_count = tls_src.matches("source_ip: Some").count();
    assert_eq!(
        tls_some_count, 0,
        "BC-2.09.001 invariant 2 violated: analyzer/tls.rs must not set \
         source_ip: Some(...) at any emission site, found {tls_some_count} occurrence(s)"
    );

    // Positive: exactly 7 TLS emission sites set source_ip: None.
    let tls_none_count = tls_src.matches("source_ip: None").count();
    assert_eq!(
        tls_none_count, 7,
        "BC-2.09.001 invariant 2: analyzer/tls.rs must have exactly 7 \
         `source_ip: None` emission sites, found {tls_none_count}. \
         A new emission site was added or removed without updating this test."
    );
}

// --- BC-2.09.001 invariant 3 (direction: HTTP/TLS emission sites all set Some) ---

/// BC-2.09.001 invariant 3: HTTP and TLS analyzer findings all set
/// `direction: Some(...)`. No HTTP or TLS emission site leaves direction as None.
///
/// Per BC-2.09.001 invariant 3: "HTTP and TLS analyzer findings set
/// direction: Some(...)".
///
/// Counts:
///   analyzer/http.rs — 9 emission sites, all `direction: Some`
///   analyzer/tls.rs  — 7 emission sites, all `direction: Some`
///
/// Uses in-process file reading; a missing file fails the test.
#[test]
fn test_direction_some_at_all_http_tls_emission_sites() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let http_src = read_src_file(manifest_dir, "analyzer/http.rs");
    let tls_src = read_src_file(manifest_dir, "analyzer/tls.rs");

    // Negative: no `direction: None` at any HTTP or TLS emission site.
    let http_none_count = http_src.matches("direction: None").count();
    assert_eq!(
        http_none_count, 0,
        "BC-2.09.001 invariant 3 violated: analyzer/http.rs must not set \
         direction: None at any emission site, found {http_none_count} occurrence(s)"
    );

    let tls_none_count = tls_src.matches("direction: None").count();
    assert_eq!(
        tls_none_count, 0,
        "BC-2.09.001 invariant 3 violated: analyzer/tls.rs must not set \
         direction: None at any emission site, found {tls_none_count} occurrence(s)"
    );

    // Positive: exactly 9 HTTP sites and 7 TLS sites set direction: Some.
    let http_some_count = http_src.matches("direction: Some").count();
    assert_eq!(
        http_some_count, 9,
        "BC-2.09.001 invariant 3: analyzer/http.rs must have exactly 9 \
         `direction: Some(...)` emission sites, found {http_some_count}. \
         A new emission site was added or removed without updating this test."
    );

    let tls_some_count = tls_src.matches("direction: Some").count();
    assert_eq!(
        tls_some_count, 7,
        "BC-2.09.001 invariant 3: analyzer/tls.rs must have exactly 7 \
         `direction: Some(...)` emission sites, found {tls_some_count}. \
         A new emission site was added or removed without updating this test."
    );
}

// --- BC-2.09.001 invariant 4 (direction: reassembly emission sites) ---

/// BC-2.09.001 invariant 4: Reassembly anomaly findings in reassembly/mod.rs
/// (overlap, small-segment, out-of-window) set `direction: Some(dir)`.
/// Reassembly lifecycle findings (reassembly/lifecycle.rs) and the
/// segment-limit summary finding (reassembly/mod.rs) set `direction: None`.
///
/// Counts (per BC-2.09.001 invariant 4):
///   reassembly/mod.rs      — 3 × `direction: Some(dir)`, 1 × `direction: None`
///   reassembly/lifecycle.rs — 0 × `direction: Some`,      2 × `direction: None`
///
/// Uses in-process file reading; a missing file fails the test.
#[test]
fn test_direction_at_reassembly_emission_sites() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let mod_src = read_src_file(manifest_dir, "reassembly/mod.rs");
    let lifecycle_src = read_src_file(manifest_dir, "reassembly/lifecycle.rs");

    // reassembly/mod.rs: exactly 3 Some sites (the three anomaly findings).
    let mod_some_count = mod_src.matches("direction: Some").count();
    assert_eq!(
        mod_some_count, 3,
        "BC-2.09.001 invariant 4: reassembly/mod.rs must have exactly 3 \
         `direction: Some(...)` emission sites (overlap, small-segment, out-of-window), \
         found {mod_some_count}"
    );

    // reassembly/mod.rs: exactly 1 None site (the segment-limit summary finding).
    let mod_none_count = mod_src.matches("direction: None").count();
    assert_eq!(
        mod_none_count, 1,
        "BC-2.09.001 invariant 4: reassembly/mod.rs must have exactly 1 \
         `direction: None` emission site (segment-limit summary), \
         found {mod_none_count}"
    );

    // reassembly/lifecycle.rs: no Some — lifecycle findings set direction: None.
    let lifecycle_some_count = lifecycle_src.matches("direction: Some").count();
    assert_eq!(
        lifecycle_some_count, 0,
        "BC-2.09.001 invariant 4 violated: reassembly/lifecycle.rs must not set \
         direction: Some(...) at any emission site, found {lifecycle_some_count} occurrence(s)"
    );

    // reassembly/lifecycle.rs: exactly 2 None sites.
    let lifecycle_none_count = lifecycle_src.matches("direction: None").count();
    assert_eq!(
        lifecycle_none_count, 2,
        "BC-2.09.001 invariant 4: reassembly/lifecycle.rs must have exactly 2 \
         `direction: None` emission sites (conflicting-overlap, stream-depth-exceeded), \
         found {lifecycle_none_count}"
    );
}

// --- AC-004 / BC-2.09.002 postcondition 1 ---

/// AC-004 (BC-2.09.002 postcondition 1): `format!("{finding}")` produces exactly
/// `"[Anomaly] LIKELY (HIGH) \u{2014} test"` for the canonical test vector.
#[test]
fn test_finding_display_format() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "test".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let formatted = format!("{finding}");
    assert_eq!(
        formatted, "[Anomaly] LIKELY (HIGH) \u{2014} test",
        "BC-2.09.002 postcondition 1: Finding Display must match canonical test vector"
    );
}

// --- AC-005 / BC-2.09.002 postcondition 5 ---

/// AC-005 (BC-2.09.002 postcondition 5): `Finding::Display` includes `summary`
/// as-is — no escaping applied, raw bytes preserved (ADR 0003).
#[test]
fn test_finding_display_preserves_raw_summary() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

    let raw_summary = "payload \x1b[31mRED\x1b[0m".to_string();
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: raw_summary.clone(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let formatted = format!("{finding}");
    assert!(
        formatted.as_bytes().contains(&0x1b),
        "BC-2.09.002 postcondition 5 / ADR 0003: Finding::Display must preserve raw ESC byte \
         in summary without escaping; got: {formatted:?}"
    );
    assert!(
        formatted.contains(&raw_summary as &str),
        "BC-2.09.002: summary must appear verbatim in Display output; got: {formatted:?}"
    );
}

// --- AC-006 / BC-2.09.003 postcondition 1 ---

/// AC-006 (BC-2.09.003 postcondition 1): `Verdict::Likely` displays as `"LIKELY"`.
#[test]
fn test_verdict_display_likely() {
    use wirerust::findings::Verdict;

    assert_eq!(
        format!("{}", Verdict::Likely),
        "LIKELY",
        "BC-2.09.003: Verdict::Likely must display as 'LIKELY'"
    );
}

// --- AC-007 / BC-2.09.003 postcondition 2 ---

/// AC-007 (BC-2.09.003 postcondition 2): `Verdict::Unlikely` displays as
/// `"UNLIKELY"`.
#[test]
fn test_verdict_display_unlikely() {
    use wirerust::findings::Verdict;

    assert_eq!(
        format!("{}", Verdict::Unlikely),
        "UNLIKELY",
        "BC-2.09.003: Verdict::Unlikely must display as 'UNLIKELY'"
    );
}

// --- AC-008 / BC-2.09.003 postcondition 3 ---

/// AC-008 (BC-2.09.003 postcondition 3): `Verdict::Inconclusive` displays as
/// `"INCONCLUSIVE"`.
#[test]
fn test_verdict_display_inconclusive() {
    use wirerust::findings::Verdict;

    assert_eq!(
        format!("{}", Verdict::Inconclusive),
        "INCONCLUSIVE",
        "BC-2.09.003: Verdict::Inconclusive must display as 'INCONCLUSIVE'"
    );
}

// --- AC-009 / BC-2.09.004 postcondition 1 ---

/// AC-009 (BC-2.09.004 postcondition 1): `Confidence::High` displays as `"HIGH"`.
#[test]
fn test_confidence_display_high() {
    use wirerust::findings::Confidence;

    assert_eq!(
        format!("{}", Confidence::High),
        "HIGH",
        "BC-2.09.004: Confidence::High must display as 'HIGH'"
    );
}

// --- AC-010 / BC-2.09.004 postcondition 2 ---

/// AC-010 (BC-2.09.004 postcondition 2): `Confidence::Medium` displays as
/// `"MEDIUM"`.
#[test]
fn test_confidence_display_medium() {
    use wirerust::findings::Confidence;

    assert_eq!(
        format!("{}", Confidence::Medium),
        "MEDIUM",
        "BC-2.09.004: Confidence::Medium must display as 'MEDIUM'"
    );
}

// --- AC-011 / BC-2.09.004 postcondition 3 ---

/// AC-011 (BC-2.09.004 postcondition 3): `Confidence::Low` displays as `"LOW"`.
#[test]
fn test_confidence_display_low() {
    use wirerust::findings::Confidence;

    assert_eq!(
        format!("{}", Confidence::Low),
        "LOW",
        "BC-2.09.004: Confidence::Low must display as 'LOW'"
    );
}

// ---------------------------------------------------------------------------
// Edge cases EC-001 through EC-005 (STORY-069 edge case table)
// ---------------------------------------------------------------------------

/// EC-001 (BC-2.09.001 EC-001 / STORY-069 EC-001): `evidence = vec![]` is a
/// valid Finding; reporters handle empty evidence list gracefully.
/// Test name follows snake_case per Rust convention; BC identifier is bc_2_09_001.
#[test]
fn test_bc_2_09_001_ec001_empty_evidence_is_valid() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reporter::Reporter;
    use wirerust::reporter::json::JsonReporter;
    use wirerust::summary::Summary;

    let f = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "no evidence".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    assert!(f.evidence.is_empty(), "EC-001: evidence vec must be empty");

    let out = JsonReporter.render(&Summary::new(), &[f], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(
        parsed["findings"][0]["evidence"]
            .as_array()
            .map(|a| a.len()),
        Some(0),
        "EC-001: JSON reporter must emit empty evidence array without panicking"
    );
}

/// EC-002 (BC-2.09.002 EC-003 / STORY-069 EC-002): `summary = ""` — Display
/// renders `"[Anomaly] LIKELY (HIGH) \u{2014} "` (em-dash then space, summary empty).
#[test]
fn test_bc_2_09_002_ec002_empty_summary_display() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

    let f = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: String::new(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let formatted = format!("{f}");
    assert_eq!(
        formatted, "[Anomaly] LIKELY (HIGH) \u{2014} ",
        "EC-002: empty summary must produce trailing em-dash space with nothing after"
    );
}

/// EC-003 (BC-2.09.002 EC-002 / STORY-069 EC-003): `summary` contains ESC byte
/// (0x1B) — ESC byte appears literally in Display output (no escaping applied).
#[test]
fn test_bc_2_09_002_ec003_esc_byte_in_summary_preserved_in_display() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

    let f = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "before\x1bafter".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let formatted = format!("{f}");
    assert!(
        formatted.as_bytes().contains(&0x1b),
        "EC-003: ESC byte (0x1B) must appear literally in Finding Display output; \
         got: {formatted:?}"
    );
    assert!(
        formatted.contains("before\x1bafter"),
        "EC-003: raw summary with ESC byte must appear verbatim in Display; got: {formatted:?}"
    );
}

/// EC-004 (STORY-069 EC-004): `direction = Some(ServerToClient)` — field holds
/// value; Display does not render the direction field.
#[test]
fn test_bc_2_09_001_ec004_direction_some_server_to_client() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reassembly::handler::Direction;

    let f = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "directional".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: Some(Direction::ServerToClient),
    };

    assert!(
        matches!(f.direction, Some(Direction::ServerToClient)),
        "EC-004: direction field must hold Some(ServerToClient)"
    );

    // BC-2.09.002: Display format is [Category] VERDICT (CONFIDENCE) — summary;
    // direction is not part of the template.
    let formatted = format!("{f}");
    assert!(
        !formatted.contains("ServerToClient"),
        "EC-004: Finding Display must not render the direction field; got: {formatted}"
    );
}

/// EC-005 (STORY-069 EC-005): `category = Reconnaissance` — Display renders
/// `"[Reconnaissance] ..."` using the Debug variant name.
#[test]
fn test_bc_2_09_002_ec005_reconnaissance_category_display() {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

    let f = Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "scan".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let formatted = format!("{f}");
    assert_eq!(
        formatted, "[Reconnaissance] INCONCLUSIVE (LOW) \u{2014} scan",
        "EC-005: Reconnaissance category must render as '[Reconnaissance]' \
         (Debug variant name per BC-2.09.002 postcondition 2)"
    );
}

// ---------------------------------------------------------------------------
// STORY-070 (BC-2.09.005 / BC-2.09.006): Raw-Data Contract and JSON
// Serialization Symmetry (skip_serializing_if)
//
// NOTE: The STORY-069 block above also uses AC-001..AC-011 / EC-001..EC-005
// numbering (for BC-2.09.001..BC-2.09.004). The two blocks are DISTINCT:
//   STORY-069 AC-001..AC-011 → BC-2.09.001 (Finding struct) / BC-2.09.002
//             (Display) / BC-2.09.003 (Verdict) / BC-2.09.004 (Confidence)
//   STORY-070 AC-001..AC-011 → BC-2.09.005 (raw-data contract) /
//             BC-2.09.006 (JSON skip_serializing_if symmetry)
//
// implementation_strategy: brownfield-formalization
// All tests are expected to PASS (brownfield-confirm) because the serde
// skip_serializing_if attributes already exist on all four Option fields.
// Any FAIL indicates a real gap where existing code does not satisfy the BC.
// ---------------------------------------------------------------------------

// --- AC-001 / BC-2.09.005 postcondition 1 ---

/// AC-001 (BC-2.09.005 postcondition 1): `Finding.summary` contains raw
/// post-`from_utf8_lossy` bytes without any additional escaping at construction
/// time. This is verified by driving a real analyzer construction site
/// (HttpAnalyzer) rather than constructing a Finding directly, so the test
/// exercises the actual analyzer code path rather than just `String` behavior.
///
/// httparse accepts C1 control bytes (e.g., U+009B CSI, encoded as 0xC2 0x9B)
/// in URIs because they are high bytes; it rejects C0 bytes (0x00-0x1F). The
/// test uses C1 CSI — a genuine control byte that `String::from_utf8_lossy`
/// passes through unchanged — to verify the raw-data contract on a real analyzer
/// path. The property under test is that the analyzer does NOT call
/// `escape_for_terminal` or any other byte transformation: the control byte
/// present in the attacker-controlled input must appear verbatim in the Finding.
///
/// Canonical test vector: path-traversal request with C1 CSI in the URI
/// (same payload as `test_http_finding_c1_csi_escaped_by_terminal_reporter`).
#[test]
fn test_finding_summary_preserves_raw_c1_bytes() {
    // Build an HTTP request whose URI contains a C1 CSI byte (U+009B, 0xC2 0x9B).
    // httparse accepts this because the UTF-8 encoding uses high bytes (>= 0x80).
    // The path-traversal prefix "/../" triggers the path-traversal Finding, which
    // puts the URI (via truncate_uri) into Finding.summary via from_utf8_lossy.
    let mut request = b"GET /../../etc/passwd".to_vec();
    request.extend_from_slice(&[0xC2, 0x9B]); // U+009B CSI — control byte httparse accepts
    request.extend_from_slice(b"31mHACKED HTTP/1.1\r\nHost: target.com\r\n\r\n");

    let mut analyzer = HttpAnalyzer::new();
    let fk = http_test_flow_key();
    analyzer.on_data(&fk, Direction::ClientToServer, &request, 0, 0);

    let findings = analyzer.findings();
    assert!(
        !findings.is_empty(),
        "AC-001: path-traversal request must produce at least one Finding"
    );

    // The path-traversal Finding puts the raw URI bytes into summary via
    // from_utf8_lossy. The C1 CSI byte (0xC2 0x9B) must appear verbatim —
    // no escape_for_terminal, no Debug-format, no other transformation.
    let traversal = findings
        .iter()
        .find(|f| f.summary.contains("Path traversal"))
        .expect("AC-001: expected a path-traversal Finding with the C1 CSI URI");

    assert!(
        traversal
            .summary
            .as_bytes()
            .windows(2)
            .any(|w| w == [0xC2, 0x9B]),
        "BC-2.09.005 postcondition 1: Finding.summary must contain the raw C1 CSI bytes \
         (0xC2 0x9B) without any escaping at the analyzer construction site; \
         got: {:?}",
        traversal.summary
    );
    // Confirm the escaped form is NOT present (would indicate construction-site escaping).
    assert!(
        !traversal.summary.contains("\\u{9b}"),
        "BC-2.09.005 postcondition 1: Finding.summary must not contain the escape-form \
         '\\u{{9b}}'; the analyzer must not call escape_for_terminal; \
         got: {:?}",
        traversal.summary
    );
}

// --- BC-2.09.005 postcondition 2 (evidence raw-byte preservation) ---

/// BC-2.09.005 postcondition 2: `Finding.evidence` carries raw post-`from_utf8_lossy`
/// bytes with the same guarantee as `summary`. An evidence entry containing ESC
/// byte 0x1B must store the literal 0x1B byte at construction time — not any
/// escaped form.
///
/// This mirrors `test_finding_summary_preserves_raw_c1_bytes` but exercises the
/// `evidence` field, which BC-2.09.005 §Postconditions explicitly covers as
/// carrying the same raw-byte guarantee (postcondition 2).
#[test]
fn test_finding_evidence_preserves_raw_c0_bytes() {
    // Simulate what an analyzer produces via from_utf8_lossy on attacker-
    // controlled payload bytes containing ESC (0x1B) in an evidence entry.
    let raw_bytes: Vec<u8> = b"header: \x1b[31mINJECTED\x1b[0m".to_vec();
    let evidence_entry = String::from_utf8_lossy(&raw_bytes).into_owned();

    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "payload with injected evidence".to_string(),
        evidence: vec![evidence_entry.clone()],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    // Postcondition 2: the literal 0x1B byte is present in evidence[0].
    assert!(
        finding.evidence[0].as_bytes().contains(&0x1b),
        "BC-2.09.005 postcondition 2: Finding.evidence[0] must contain the literal ESC byte \
         (0x1B) without any additional escaping at construction time; got: {:?}",
        finding.evidence[0]
    );
    // Must NOT contain the escaped string form.
    assert!(
        !finding.evidence[0].contains("\\u{1b}"),
        "BC-2.09.005 postcondition 2: Finding.evidence[0] must not contain the escape-form \
         '\\u{{1b}}'; raw bytes must pass through; got: {:?}",
        finding.evidence[0]
    );
    // Round-trip: the field must equal the from_utf8_lossy output exactly.
    assert_eq!(
        finding.evidence[0], evidence_entry,
        "BC-2.09.005 postcondition 2: Finding.evidence[0] must equal the from_utf8_lossy output"
    );
}

// --- AC-002 / BC-2.09.005 invariant 1 ---

/// AC-002 (BC-2.09.005 invariant 1): `escape_for_terminal` is defined and
/// invoked exclusively within `src/reporter/terminal.rs` (module-containment
/// property). BC-2.09.005 v1.3 establishes three call sites inside that file
/// (render_finding_prefix ×2, analyzer-summary rendering ×1); the property
/// enforced here is that no OTHER source file references the function on a
/// non-comment line. This prevents any analyzer from calling escape_for_terminal
/// at a Finding construction site — which would violate ADR 0003 and corrupt
/// forensic byte fidelity.
///
/// Uses in-process `std::fs::read_to_string` (hardened pattern from STORY-069)
/// — no dependency on grep being on PATH; a missing file panics the test rather
/// than silently passing.
///
/// Robustness: the scan filters out comment and doc lines (those whose trimmed
/// form starts with `//`) before counting occurrences. This prevents a future
/// doc comment mentioning the function name from producing a false failure.
/// Only non-comment code lines are checked, which is where call sites live.
#[test]
fn test_escape_for_terminal_contained_to_terminal_module() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let src_root = std::path::Path::new(manifest_dir).join("src");

    // Walk the src tree and accumulate all .rs files that are NOT terminal.rs.
    let mut violations: Vec<String> = Vec::new();
    let mut scanned_files: usize = 0;
    let mut found_terminal_rs = false;

    for entry in walkdir_rs_files(&src_root) {
        let path = &entry;
        // Track whether the walk reached src/reporter/terminal.rs — this proves
        // the walk descended into the reporter subdirectory and was not vacuous.
        if path.ends_with("reporter/terminal.rs") {
            found_terminal_rs = true;
            continue;
        }
        scanned_files += 1;
        let content = std::fs::read_to_string(path).unwrap_or_else(|e| {
            panic!(
                "BC-2.09.005 invariant 1: source file '{}' must be readable: {e}",
                path.display()
            )
        });
        // Count only non-comment lines — strip lines whose trimmed form begins
        // with `//` (line comments and doc comments). This makes the test
        // resilient to doc comments that mention the function name without
        // calling it, avoiding false positives from future documentation edits.
        let call_site_count = content
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .filter(|line| line.contains("escape_for_terminal"))
            .count();
        if call_site_count > 0 {
            violations.push(format!(
                "'{}': {} non-comment occurrence(s)",
                path.strip_prefix(manifest_dir).unwrap_or(path).display(),
                call_site_count
            ));
        }
    }

    // Positive-coverage assertion 1: the walk must have found and skipped
    // src/reporter/terminal.rs, proving it descended into the reporter directory.
    // A vacuous walk (e.g. src_root resolves to a non-existent path) would leave
    // this false, turning a silent false-pass into an explicit failure.
    assert!(
        found_terminal_rs,
        "BC-2.09.005 invariant 1: walk did not find src/reporter/terminal.rs — \
         the src/ tree was not scanned correctly. Check that CARGO_MANIFEST_DIR \
         resolves to the crate root and src/reporter/terminal.rs exists."
    );

    // Positive-coverage assertion 2: at least 10 other .rs files must have been
    // scanned. The codebase currently has many more; this floor ensures a partial
    // or empty walk cannot silently pass the violation check.
    assert!(
        scanned_files >= 10,
        "BC-2.09.005 invariant 1: only {scanned_files} .rs file(s) scanned (expected >= 10). \
         The src/ walk is incomplete — a structural change may have broken the file discovery."
    );

    assert!(
        violations.is_empty(),
        "BC-2.09.005 invariant 1 violated: `escape_for_terminal` referenced outside \
         src/reporter/terminal.rs on non-comment lines. ADR 0003 mandates the function \
         is defined and invoked exclusively within that file. Violations:\n{}",
        violations.join("\n")
    );
}

/// Walk all `.rs` files under `root`, returning their absolute paths.
/// Used by `test_escape_for_terminal_contained_to_terminal_module` — written as
/// a free function so the logic is easy to read and the test remains concise.
fn walkdir_rs_files(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut results = Vec::new();
    collect_rs_files(root, &mut results);
    results
}

fn collect_rs_files(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

// --- AC-003 / BC-2.09.005 postcondition 4 ---
// test_output_sanitization_layering_contract already exists above (line 594).
// It covers this AC: JSON output contains \u001b (not literal ESC) for a
// Finding whose summary contains 0x1B. No duplicate needed.

// --- AC-004 / BC-2.09.005 invariant 3 ---

/// AC-004 (BC-2.09.005 invariant 3): Invalid UTF-8 sequences in `summary` or
/// `evidence` are replaced by U+FFFD (replacement character) via
/// `String::from_utf8_lossy`; no panic occurs.
///
/// Canonical test vector: TLS finding with non-UTF-8 SNI (invalid UTF-8 bytes).
#[test]
fn test_non_utf8_bytes_in_summary_replaced_with_fffd() {
    // 0x80 and 0xFF are invalid as standalone UTF-8 bytes.
    let bad_bytes: Vec<u8> = vec![b'S', b'N', b'I', b':', 0x80, 0xFF, b'!'];
    let summary = String::from_utf8_lossy(&bad_bytes).into_owned();

    // Must not panic — from_utf8_lossy replaces invalid sequences with U+FFFD.
    let finding = Finding {
        category: ThreatCategory::C2,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: summary.clone(),
        evidence: vec![String::from_utf8_lossy(&[0xfe, 0xfe]).into_owned()],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    // U+FFFD replacement character must appear where invalid bytes were.
    assert!(
        finding.summary.contains('\u{FFFD}'),
        "BC-2.09.005 invariant 3: invalid UTF-8 bytes must be replaced by U+FFFD; \
         got summary: {:?}",
        finding.summary
    );
    assert!(
        finding.evidence[0].contains('\u{FFFD}'),
        "BC-2.09.005 invariant 3: invalid UTF-8 bytes in evidence must be replaced by U+FFFD; \
         got evidence[0]: {:?}",
        finding.evidence[0]
    );

    // No panic — the finding serializes without incident.
    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_out).expect("JSON must be valid even with U+FFFD in summary");
    let summary_str = parsed["findings"][0]["summary"].as_str().unwrap();
    assert!(
        summary_str.contains('\u{FFFD}'),
        "BC-2.09.005 invariant 3: U+FFFD must survive JSON round-trip; got: {summary_str:?}"
    );
}

// --- AC-005 / BC-2.09.006 postcondition 2 ---

/// AC-005 (BC-2.09.006 postcondition 2 / STORY-100): When `mitre_techniques = vec![]`,
/// the JSON object has NO `"mitre_techniques"` key (Vec::is_empty skip).
///
/// Canonical test vector: Finding { mitre_techniques: vec![], ... }.
#[test]
fn test_none_mitre_technique_absent_from_json() {
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "test".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let obj = parsed["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    // STORY-100 BC-2.09.006: empty mitre_techniques → key absent from JSON.
    assert!(
        !obj.contains_key("mitre_techniques"),
        "BC-2.09.006 postcondition 2: empty mitre_techniques must produce NO key in JSON \
         (Vec::is_empty skip); got: {}",
        parsed["findings"][0]
    );
    // Also ensure the OLD scalar key is completely absent.
    assert!(
        !obj.contains_key("mitre_technique"),
        "BC-2.09.006 invariant 4: old scalar key 'mitre_technique' must not appear"
    );
}

// --- AC-006 / BC-2.09.006 postcondition 2 ---

/// AC-006 (BC-2.09.006 postcondition 2): When `source_ip = None`, the JSON
/// object has NO `"source_ip"` key.
#[test]
fn test_none_source_ip_absent_from_json() {
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "test".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let obj = parsed["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    assert!(
        !obj.contains_key("source_ip"),
        "BC-2.09.006 postcondition 2: source_ip=None must produce NO key in JSON; got: {}",
        parsed["findings"][0]
    );
}

// --- AC-007 / BC-2.09.006 postcondition 2 ---

/// AC-007 (BC-2.09.006 postcondition 2): When `direction = None`, the JSON
/// object has NO `"direction"` key.
#[test]
fn test_none_direction_absent_from_json() {
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "test".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let obj = parsed["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    assert!(
        !obj.contains_key("direction"),
        "BC-2.09.006 postcondition 2: direction=None must produce NO key in JSON; got: {}",
        parsed["findings"][0]
    );
}

// --- AC-008 / BC-2.09.006 postcondition 2 + invariant 2 ---

/// AC-008 (BC-2.09.006 postcondition 2 + invariant 2): When `timestamp = None`
/// (always, per domain-debt O-01), the JSON object has NO `"timestamp"` key in
/// any produced Finding.
///
/// Covers all four emission-site categories:
///   1. HTTP analyzer findings (direction: Some, source_ip: None)
///   2. TLS analyzer findings (direction: Some, source_ip: None)
///   3. Reassembly anomaly findings (direction: Some, source_ip: Some) — per
///      BC-2.09.001 invariant 4: overlap/small-segment/out-of-window findings
///      set both direction and source_ip.
///   4. Reassembly lifecycle findings (direction: None, source_ip: None or Some)
#[test]
fn test_timestamp_absent_from_all_finding_json() {
    // Construct four distinct Finding shapes (synthesized — all set timestamp: None
    // per BC-2.09.006 invariant 2 / O-01), one per emission-site category.
    let findings = vec![
        // 1. HTTP-analyzer shape: direction Some, source_ip None.
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "http-finding".to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T1190".to_string()],
            source_ip: None,
            timestamp: None,
            direction: Some(Direction::ClientToServer),
        },
        // 2. TLS-analyzer shape: direction Some, source_ip None.
        Finding {
            category: ThreatCategory::C2,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "tls-finding".to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T1573".to_string()],
            source_ip: None,
            timestamp: None,
            direction: Some(Direction::ClientToServer),
        },
        // 3. Reassembly-anomaly shape: direction Some, source_ip Some.
        // Per BC-2.09.001 invariant 4: overlap/small-segment/out-of-window
        // anomaly findings in reassembly/mod.rs set both fields.
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::Medium,
            summary: "TCP segment overlap anomaly".to_string(),
            evidence: vec!["overlap at seq 1000".to_string()],
            mitre_techniques: vec![],
            source_ip: Some("10.0.0.1".parse().unwrap()),
            timestamp: None,
            direction: Some(Direction::ClientToServer),
        },
        // 4. Reassembly-lifecycle shape: direction None, source_ip None.
        // Per BC-2.09.001 invariant 4: lifecycle findings set direction: None.
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Unlikely,
            confidence: Confidence::Low,
            summary: "reassembly-lifecycle-finding".to_string(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        },
    ];

    let json_out = JsonReporter.render(&Summary::new(), &findings, &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let arr = parsed["findings"].as_array().expect("findings array");

    for (i, finding_val) in arr.iter().enumerate() {
        let obj = finding_val
            .as_object()
            .unwrap_or_else(|| panic!("finding[{i}] must be a JSON object"));
        assert!(
            !obj.contains_key("timestamp"),
            "BC-2.09.006 postcondition 2 + invariant 2 (O-01): \
             findings[{i}] must have NO 'timestamp' key (timestamp is always None); \
             got: {finding_val}"
        );
    }
}

// --- AC-009 / BC-2.09.006 postcondition 1 ---

/// AC-009 (BC-2.09.006 postcondition 1 / STORY-100): When `mitre_techniques = vec!["T1036"]`,
/// the JSON object contains `"mitre_techniques": ["T1036"]` (array, not scalar).
///
/// STORY-100 intended break: JSON format changes from scalar string to array.
#[test]
fn test_some_mitre_technique_present_in_json() {
    let finding = Finding {
        category: ThreatCategory::LateralMovement,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "masquerading".to_string(),
        evidence: vec![],
        mitre_techniques: vec!["T1036".to_string()],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");

    // STORY-100 BC-2.09.006: singleton vec → JSON array ["T1036"], not scalar "T1036".
    assert_eq!(
        parsed["findings"][0]["mitre_techniques"],
        serde_json::json!(["T1036"]),
        "BC-2.09.006 postcondition 1: mitre_techniques=vec![\"T1036\"] must produce \
         JSON array '[\"T1036\"]'"
    );
}

// --- AC-010 / BC-2.09.006 postcondition 1 ---

/// AC-010 (BC-2.09.006 postcondition 1): When `direction = Some(ClientToServer)`,
/// the JSON object contains `"direction": "ClientToServer"`.
///
/// Canonical test vector: Finding { direction: Some(ClientToServer), ... }.
#[test]
fn test_some_direction_present_in_json() {
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::Medium,
        summary: "directional-finding".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: Some(Direction::ClientToServer),
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");

    assert_eq!(
        parsed["findings"][0]["direction"],
        serde_json::json!("ClientToServer"),
        "BC-2.09.006 postcondition 1: direction=Some(ClientToServer) must produce \
         '\"direction\": \"ClientToServer\"' in JSON"
    );
}

// --- AC-011 / BC-2.09.006 invariant 3 ---

/// AC-011 (BC-2.09.006 invariant 3): Reassembly-engine findings with
/// `direction: None` (lifecycle, segment-limit-summary) produce JSON with no
/// `"direction"` key.
///
/// These findings come from `reassembly/lifecycle.rs` (conflicting-overlap,
/// stream-depth-exceeded) and the segment-limit summary in `reassembly/mod.rs`.
/// All set direction: None and must produce no JSON key for that field.
#[test]
fn test_reassembly_lifecycle_finding_no_direction_in_json() {
    // Synthesized to match the shape emitted by reassembly/lifecycle.rs.
    let lifecycle_finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::Medium,
        summary: "Stream depth limit exceeded".to_string(),
        evidence: vec!["stream-depth: 65536".to_string()],
        mitre_techniques: vec![],
        source_ip: Some("10.0.0.1".parse().unwrap()),
        timestamp: None,
        direction: None, // lifecycle findings always set direction: None
    };

    let json_out = JsonReporter.render(&Summary::new(), &[lifecycle_finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let obj = parsed["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    assert!(
        !obj.contains_key("direction"),
        "BC-2.09.006 invariant 3: reassembly lifecycle/segment-limit findings must have \
         no 'direction' key in JSON (direction is always None for these); got: {}",
        parsed["findings"][0]
    );
    // Verify source_ip IS present (these findings do set source_ip).
    assert!(
        obj.contains_key("source_ip"),
        "BC-2.09.006: reassembly lifecycle findings must include source_ip when Some; \
         got: {}",
        parsed["findings"][0]
    );
}

// ---------------------------------------------------------------------------
// STORY-070 Edge Cases EC-001 through EC-005
// ---------------------------------------------------------------------------

/// EC-001 (STORY-070): Full pipeline — Finding with ESC in summary:
///   - JSON output contains `\u001b` (RFC 8259 escape), not the literal 0x1B byte.
///   - Terminal output contains the display-escaped form `\u{1b}`.
///   - Finding.summary has the literal 0x1B byte (forensic preservation).
///
/// This is the three-layer ADR 0003 contract in one end-to-end test.
/// Note: test_output_sanitization_layering_contract above covers these three
/// layers in detail; this test restates them in the EC-001 naming.
#[test]
fn test_story_070_ec001_full_pipeline_esc_in_uri() {
    let raw_bytes = b"GET /\x1b[31mEVIL HTTP/1.1";
    let summary = String::from_utf8_lossy(raw_bytes).into_owned();

    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: summary.clone(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    // Layer 1: struct preserves literal 0x1B.
    assert!(
        finding.summary.as_bytes().contains(&0x1b),
        "EC-001: Finding.summary must contain literal 0x1B byte; got: {:?}",
        finding.summary
    );

    // Layer 2: terminal reporter escapes to \u{1b} form.
    let terminal_out = TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender::new(Grouping::Flat, Collapse::Expanded),
    }
    .render(&Summary::new(), std::slice::from_ref(&finding), &[]);
    assert!(
        !terminal_out.as_bytes().contains(&0x1b),
        "EC-001: terminal output must not contain raw ESC byte; got: {terminal_out:?}"
    );
    assert!(
        terminal_out.contains("\\u{1b}"),
        "EC-001: terminal output must contain '\\u{{1b}}' escape form; got: {terminal_out}"
    );

    // Layer 3: JSON reporter escapes via serde RFC 8259 \u001b form.
    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    assert!(
        !json_out.as_bytes().contains(&0x1b),
        "EC-001: JSON output must not contain raw ESC byte; got: {json_out:?}"
    );
    assert!(
        json_out.contains("\\u001b"),
        "EC-001: JSON output must contain '\\u001b' (RFC 8259); got: {json_out}"
    );
}

/// EC-002 (STORY-070 / BC-2.09.006 EC-002): Finding with
/// `source_ip = Some(IpAddr::V4(1.2.3.4))` — JSON has `"source_ip": "1.2.3.4"`.
#[test]
fn test_story_070_ec002_source_ip_some_in_json() {
    use std::net::{IpAddr, Ipv4Addr};

    let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));
    let finding = Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "probe".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: Some(ip),
        timestamp: None,
        direction: None,
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");

    assert_eq!(
        parsed["findings"][0]["source_ip"],
        serde_json::json!("1.2.3.4"),
        "EC-002: source_ip=Some(V4(1.2.3.4)) must serialize as '1.2.3.4'"
    );
}

/// EC-003 (STORY-070 / STORY-100): The fields `mitre_techniques` (non-empty vec),
/// `source_ip`, and `direction` are all set — all three keys are present in JSON.
/// STORY-100 intended break: `mitre_techniques` is now a JSON array `["T1190"]`, not scalar.
#[test]
fn test_story_070_ec003_three_some_option_fields_present_in_json() {
    use std::net::{IpAddr, Ipv4Addr};

    let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "three-some-optional-fields".to_string(),
        evidence: vec![],
        mitre_techniques: vec!["T1190".to_string()],
        source_ip: Some(ip),
        // timestamp is always None (O-01) — setting it here would violate the
        // domain invariant. Only the three realistically-Some fields are tested.
        timestamp: None,
        direction: Some(Direction::ClientToServer),
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let obj = parsed["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    // STORY-100: mitre_techniques is now a JSON array.
    assert!(
        obj.contains_key("mitre_techniques"),
        "EC-003: non-empty mitre_techniques must produce 'mitre_techniques' JSON key; got: {}",
        parsed["findings"][0]
    );
    assert!(
        obj.contains_key("source_ip"),
        "EC-003: source_ip=Some(...) must produce a JSON key; got: {}",
        parsed["findings"][0]
    );
    assert!(
        obj.contains_key("direction"),
        "EC-003: direction=Some(...) must produce a JSON key; got: {}",
        parsed["findings"][0]
    );
    // timestamp is always None per O-01; its absence is expected.
    assert!(
        !obj.contains_key("timestamp"),
        "EC-003: timestamp=None must produce NO JSON key; got: {}",
        parsed["findings"][0]
    );
    // STORY-100: value is now JSON array, not scalar string.
    assert_eq!(
        obj["mitre_techniques"],
        serde_json::json!(["T1190"]),
        "EC-003: mitre_techniques value must be JSON array ['T1190']"
    );
    assert_eq!(
        obj["direction"],
        serde_json::json!("ClientToServer"),
        "EC-003: direction value must be 'ClientToServer'"
    );
}

/// EC-004 (STORY-070): All four Option fields are None — zero of the four
/// keys appear in JSON.
#[test]
fn test_story_070_ec004_all_four_option_fields_none_all_absent_from_json() {
    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "no-optional-fields".to_string(),
        evidence: vec![],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    let json_out = JsonReporter.render(&Summary::new(), &[finding], &[]);
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let obj = parsed["findings"][0]
        .as_object()
        .expect("finding must be a JSON object");

    // STORY-100: check mitre_techniques (new name) and also old scalar key absent.
    for key in &[
        "mitre_techniques",
        "mitre_technique",
        "source_ip",
        "timestamp",
        "direction",
    ] {
        assert!(
            !obj.contains_key(*key),
            "EC-004: all optional fields empty/None — '{}' must be absent from JSON \
             (no key, not null); got: {}",
            key,
            parsed["findings"][0]
        );
    }
}

/// EC-005 (STORY-070): `evidence = vec!["raw\x00bytes"]` — JSON encodes the
/// null byte via serde (as `\u0000`); `finding.evidence[0]` contains the
/// literal `\x00` byte (forensic preservation).
#[test]
fn test_story_070_ec005_null_byte_in_evidence_preserved_and_encoded_in_json() {
    let raw_evidence = "raw\x00bytes".to_string();

    let finding = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Low,
        summary: "null-byte-test".to_string(),
        evidence: vec![raw_evidence.clone()],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    };

    // The struct preserves the literal null byte (forensic layer).
    assert!(
        finding.evidence[0].as_bytes().contains(&0x00),
        "EC-005: Finding.evidence[0] must contain the literal null byte (0x00); \
         got: {:?}",
        finding.evidence[0]
    );

    // JSON reporter encodes the null byte via serde (RFC 8259 \u0000 form).
    let json_out = JsonReporter.render(&Summary::new(), std::slice::from_ref(&finding), &[]);
    assert!(
        !json_out.as_bytes().contains(&0x00),
        "EC-005: JSON output must not contain a raw null byte; got bytes: {:?}",
        &json_out.as_bytes()[..json_out.len().min(200)]
    );
    assert!(
        json_out.contains("\\u0000"),
        "EC-005: JSON output must encode null byte as '\\u0000' (serde RFC 8259); \
         got: {json_out}"
    );

    // Round-trip: the deserialized evidence must match the original.
    let parsed: serde_json::Value = serde_json::from_str(&json_out).expect("valid JSON");
    let evidence_str = parsed["findings"][0]["evidence"][0]
        .as_str()
        .expect("evidence[0] must be a string");
    assert_eq!(
        evidence_str, raw_evidence,
        "EC-005: JSON round-trip must recover the original evidence string with literal null byte"
    );
}
