use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

#[test]
fn test_finding_creation() {
    let finding = Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "Port scan detected from 10.0.0.1".into(),
        evidence: vec!["50 SYN packets to sequential ports in 2s".into()],
        mitre_technique: Some("T1046".into()),
        source_ip: Some("10.0.0.1".parse().unwrap()),
        timestamp: None,
    };
    assert_eq!(finding.verdict, Verdict::Likely);
    assert_eq!(finding.confidence, Confidence::High);
    assert!(finding.mitre_technique.is_some());
}

#[test]
fn test_finding_display() {
    let finding = Finding {
        category: ThreatCategory::C2,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Medium,
        summary: "Periodic beaconing pattern".into(),
        evidence: vec![],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    };
    let display = format!("{finding}");
    assert!(display.contains("C2"));
    assert!(display.contains("INCONCLUSIVE"));
}
