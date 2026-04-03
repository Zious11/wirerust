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
        // Convert Protocol (non-string) keys to strings for JSON compatibility
        let protocols: std::collections::HashMap<String, u64> = summary
            .protocol_counts()
            .iter()
            .map(|(k, v)| (format!("{k:?}"), *v))
            .collect();

        let output = json!({
            "summary": {
                "total_packets": summary.total_packets,
                "total_bytes": summary.total_bytes,
                "unique_hosts": summary.unique_hosts(),
                "protocols": protocols,
                "services": summary.service_counts(),
            },
            "findings": findings,
            "analyzers": analyzer_summaries,
        });
        serde_json::to_string_pretty(&output).unwrap()
    }
}
