use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::Reporter;
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
