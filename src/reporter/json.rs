//! JSON reporter — machine-readable rendering for downstream tooling.
//!
//! Emits a `{ "summary": {...}, "findings": [...], "analyzers": [...] }`
//! object. Per LESSON-P1.02 / NFR OBS-010, all three `Option<_>` fields on
//! [`Finding`] use `#[serde(skip_serializing_if = "Option::is_none")]`, so
//! the JSON shape is symmetric: absent values are omitted, present
//! values are emitted under their key.
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
        });
        serde_json::to_string_pretty(&output).unwrap()
    }
}
