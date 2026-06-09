//! JSON reporter — machine-readable rendering for downstream tooling.
//!
//! Emits a `{ "summary": {...}, "findings": [...], "analyzers": [...],
//! "mitre_domain": "ics-attack", "mitre_attack_version": "ics-attack-19.1" }`
//! object (BC-2.11.001). Per STORY-100 / BC-2.09.006, `mitre_techniques`
//! is a JSON array (empty vec → key absent via `Vec::is_empty` skip).
//!
//! No escaping is performed here — per ADR 0003, raw bytes flow through
//! the `Finding` summary/evidence fields and are escaped only at the
//! terminal display layer. Consumers of JSON output should expect
//! attacker-controlled byte sequences in those fields.

use serde_json::json;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reporter::Reporter;
use crate::summary::Summary;

/// ATT&CK for ICS domain identifier — constant, not dynamic.
const MITRE_DOMAIN: &str = "ics-attack";

// F4: RESOLVED — pinned to ATT&CK for ICS v19.1 (released 2026-04-28).
// Canonical STIX bundle: ics-attack-19.1.json. All emitted ICS technique IDs
// (T0888, T0855, T0836, T0835, T0831, T0814, T0806) confirmed valid and active
// in v19.1. See .factory/research/attack-ics-version-pin.md for full validation.
const MITRE_ATTACK_VERSION: &str = "ics-attack-19.1";

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn render(
        &self,
        summary: &Summary,
        findings: &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String {
        // LESSON-P2.09 / NFR DET-001: every map serialized into the
        // JSON output goes through a `BTreeMap` first so the key
        // order is deterministic (alphabetical) and snapshot/golden
        // tests stay stable across runs and target platforms. The
        // `Protocol` keys also need the non-string-to-string
        // conversion they always did.
        let protocols: std::collections::BTreeMap<String, u64> = summary
            .protocol_counts()
            .iter()
            .map(|(k, v)| (format!("{k:?}"), *v))
            .collect();
        let services: std::collections::BTreeMap<String, u64> = summary
            .service_counts()
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();

        let output = json!({
            "summary": {
                "total_packets": summary.total_packets,
                "total_bytes": summary.total_bytes,
                "skipped_packets": summary.skipped_packets,
                "unique_hosts": summary.unique_hosts(),
                "protocols": protocols,
                "services": services,
            },
            "findings": findings,
            "analyzers": analyzer_summaries,
            "mitre_domain": MITRE_DOMAIN,
            "mitre_attack_version": MITRE_ATTACK_VERSION,
        });
        serde_json::to_string_pretty(&output).unwrap()
    }
}
