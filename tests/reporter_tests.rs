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
        summary: "attacker payload: \x1b[31mRED\x1b[0m".into(),
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
}
