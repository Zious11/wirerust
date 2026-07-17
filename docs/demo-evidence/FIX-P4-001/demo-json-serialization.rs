// Demo: IEC-104 Finding JSON serialization with direction field (FIX-P4-001)
// This demonstrates the additive "direction" JSON key now present on all IEC-104 findings.
//
// All enum variants and field values are derived from real IEC-104 emit sites in
// src/analyzer/iec104.rs. No fabricated variants are used.

use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reassembly::handler::Direction;
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    println!("=== FIX-P4-001: Direction Key in IEC-104 Finding JSON ===\n");

    // Example 1: N(S) desync finding with ClientToServer direction
    // Real emit site: track_ns_desync() in src/analyzer/iec104.rs (BC-2.19.024)
    let finding_c2s = Finding {
        category: ThreatCategory::Impact,
        verdict: Verdict::Possible,
        confidence: Confidence::Medium,
        summary: "IEC-104 N(S) sequence desync: N(S)=5020 prev=5001 gap=19 > k=12 \
                  — sequence-number desynchronization detected; possible replay injection \
                  or adversarial manipulation \
                  (T1692.001 unauthorized command message; BC-2.19.024)".to_string(),
        evidence: vec![
            "N(S) gap=19 exceeds k=12 window (current_ns=5020, prev_ns=5001)".to_string(),
        ],
        mitre_techniques: vec!["T1692.001".to_string()],
        source_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
        timestamp: None,
        direction: Some(Direction::ClientToServer),
    };

    println!("Example 1: ClientToServer Finding (N(S) desync — T1692.001)");
    println!("--------------------------------------------------------------");
    match serde_json::to_string_pretty(&finding_c2s) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("Serialization error: {}", e),
    }
    println!();

    // Example 2: Malformed LEN finding with ServerToClient direction
    // Real emit site: on_data() frame-walk loop in src/analyzer/iec104.rs (BC-2.19.026)
    let finding_s2c = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Possible,
        confidence: Confidence::Medium,
        summary: "IEC-104 malformed LEN byte: 0x68 start byte followed by \
                  LEN=0x01 (1) outside valid range [4, 253] — \
                  protocol anomaly or adversarial framing attack \
                  (T0814; BC-2.19.026 invariant 5)".to_string(),
        evidence: vec![
            "LEN=1 not in [4, 253]; start byte=0x68 at buffer offset 0".to_string(),
        ],
        mitre_techniques: vec!["T0814".to_string()],
        source_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50))),
        timestamp: None,
        direction: Some(Direction::ServerToClient),
    };

    println!("Example 2: ServerToClient Finding (malformed LEN — T0814)");
    println!("-------------------------------------------------------------");
    match serde_json::to_string_pretty(&finding_s2c) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("Serialization error: {}", e),
    }
    println!();

    // Example 3: Pre-FIX-F5-001/FIX-P4-001 baseline — carry-overflow finding as it appeared
    // before enrichment. Before FIX-F5-001 (source_ip/timestamp) and FIX-P4-001 (direction),
    // all three optional fields were None and their JSON keys were omitted entirely by
    // #[serde(skip_serializing_if = "Option::is_none")]. This illustrates the historical
    // baseline JSON shape and the serde skip behavior.
    // Real emit site: on_data() carry-overflow check in src/analyzer/iec104.rs (BC-2.19.025).
    // Post-enrichment (current) real output has source_ip: Some(...), timestamp: Some(...),
    // direction: Some(direction) — see iec104.rs:1215-1217.
    let finding_pre_enrichment = Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Possible,
        confidence: Confidence::Medium,
        summary: "IEC-104 directional carry residual overflow: carry buffer \
                  exceeded MAX_IEC104_CARRY_BYTES=255 — adversarial or non-conformant \
                  byte sequence; carry cleared and analyzer resyncs on next delivery \
                  (T0814; BC-2.19.025 v1.3 F-172-001)".to_string(),
        evidence: vec!["carry overflow (>255); carry cleared".to_string()],
        mitre_techniques: vec!["T0814".to_string()],
        source_ip: None,      // pre-FIX-F5-001: source_ip was None → key absent from JSON
        timestamp: None,      // pre-FIX-F5-001: timestamp was None → key absent from JSON
        direction: None,      // pre-FIX-P4-001: direction was None → key absent from JSON
    };

    println!("Example 3: Pre-FIX-F5-001/FIX-P4-001 baseline (carry overflow — source_ip/timestamp/direction all absent)");
    println!("--------------------------------------------------------------------------------------");
    match serde_json::to_string_pretty(&finding_pre_enrichment) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("Serialization error: {}", e),
    }
    println!();

    println!("=== Key Points ===");
    println!("1. The 'direction' field appears in JSON when Some(Direction)");
    println!("2. The 'direction' field is omitted in JSON when None (serde skip_serializing_if)");
    println!("3. All IEC-104 analyzers now populate direction on emitted Findings");
    println!("4. JSON consumers can now distinguish client vs server anomalies");
}
