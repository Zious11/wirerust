use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
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
    let reporter = TerminalReporter { use_color: false };
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
    let reporter = TerminalReporter { use_color: false };
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
    let reporter = TerminalReporter { use_color: false };
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
    let terminal_output = TerminalReporter { use_color: false }.render(
        &Summary::new(),
        std::slice::from_ref(&finding),
        &[],
    );
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

    let json_output = JsonReporter.render(
        &Summary::new(),
        std::slice::from_ref(&finding),
        &[],
    );

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
