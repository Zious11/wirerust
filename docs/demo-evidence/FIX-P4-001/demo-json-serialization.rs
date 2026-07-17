// Demo: IEC-104 Finding JSON serialization with direction field (FIX-P4-001)
// This demonstrates the additive "direction" JSON key now present on all IEC-104 findings.

use wirerust::findings::{Confidence, Direction, Finding, ThreatCategory, Verdict};
use chrono::Utc;
use std::net::{IpAddr, Ipv4Addr};

fn main() {
    println!("=== FIX-P4-001: Direction Key in IEC-104 Finding JSON ===\n");

    // Example 1: Finding with ClientToServer direction
    let finding_c2s = Finding {
        category: ThreatCategory::Protocol,
        verdict: Verdict::Anomaly,
        confidence: Confidence::High,
        summary: "IEC-104 N(S) desynchronization detected".to_string(),
        evidence: vec![
            "N(S) gap exceeded threshold (5000 > 12)".to_string(),
            "Sequence tracking failure".to_string(),
        ],
        mitre_techniques: vec!["T0881".to_string()],
        source_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100))),
        timestamp: None,
        direction: Some(Direction::ClientToServer),
    };

    println!("Example 1: ClientToServer Finding");
    println!("------------------------------------");
    match serde_json::to_string_pretty(&finding_c2s) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("Serialization error: {}", e),
    }
    println!();

    // Example 2: Finding with ServerToClient direction
    let finding_s2c = Finding {
        category: ThreatCategory::Protocol,
        verdict: Verdict::Anomaly,
        confidence: Confidence::High,
        summary: "IEC-104 malformed frame detected".to_string(),
        evidence: vec![
            "Invalid LEN field".to_string(),
            "Frame structure violation".to_string(),
        ],
        mitre_techniques: vec!["T0814".to_string()],
        source_ip: Some(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 50))),
        timestamp: None,
        direction: Some(Direction::ServerToClient),
    };

    println!("Example 2: ServerToClient Finding");
    println!("----------------------------------");
    match serde_json::to_string_pretty(&finding_s2c) {
        Ok(json) => println!("{}", json),
        Err(e) => println!("Serialization error: {}", e),
    }
    println!();

    // Example 3: Finding without direction (for comparison - e.g., non-stream source)
    let finding_no_dir = Finding {
        category: ThreatCategory::Protocol,
        verdict: Verdict::Anomaly,
        confidence: Confidence::Medium,
        summary: "Engine-level summary finding".to_string(),
        evidence: vec!["Engine analysis".to_string()],
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: Some(Utc::now()),
        direction: None, // No direction for engine-level findings
    };

    println!("Example 3: Finding without direction (engine-level)");
    println!("---------------------------------------------------");
    match serde_json::to_string_pretty(&finding_no_dir) {
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
