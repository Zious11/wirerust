use owo_colors::OwoColorize;

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, Verdict};
use crate::reporter::Reporter;
use crate::summary::Summary;

pub struct TerminalReporter {
    pub use_color: bool,
}

impl Reporter for TerminalReporter {
    fn render(
        &self,
        summary: &Summary,
        findings: &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String {
        let mut out = String::new();

        // Header
        out.push_str(&self.section("WIRERUST TRIAGE REPORT"));
        out.push_str(&format!(
            "  Packets: {}  Bytes: {}  Hosts: {}\n",
            summary.total_packets,
            summary.total_bytes,
            summary.unique_hosts().len(),
        ));
        if summary.skipped_packets > 0 {
            let warning = format!("  Skipped: {} packets (decode errors)\n", summary.skipped_packets);
            if self.use_color {
                out.push_str(&warning.yellow().to_string());
            } else {
                out.push_str(&warning);
            }
        }
        out.push('\n');

        // Protocol breakdown
        out.push_str(&self.section("PROTOCOLS"));
        for (proto, count) in summary.protocol_counts() {
            out.push_str(&format!("  {proto:?}: {count}\n"));
        }
        out.push('\n');

        // Services
        let services = summary.service_counts();
        if !services.is_empty() {
            out.push_str(&self.section("SERVICES"));
            for (svc, count) in services {
                out.push_str(&format!("  {svc}: {count}\n"));
            }
            out.push('\n');
        }

        // Findings
        if !findings.is_empty() {
            out.push_str(&self.section("FINDINGS"));
            for f in findings {
                let line = format!(
                    "[{}] {} ({}) - {}",
                    f.category, f.verdict, f.confidence, f.summary
                );
                let colored = if self.use_color {
                    match f.verdict {
                        Verdict::Likely => match f.confidence {
                            Confidence::High => line.red().bold().to_string(),
                            _ => line.yellow().to_string(),
                        },
                        Verdict::Inconclusive => line.cyan().to_string(),
                        Verdict::Unlikely => line.dimmed().to_string(),
                    }
                } else {
                    line
                };
                out.push_str(&format!("  {colored}\n"));
                for ev in &f.evidence {
                    out.push_str(&format!("    > {ev}\n"));
                }
                if let Some(ref t) = f.mitre_technique {
                    out.push_str(&format!("    MITRE: {t}\n"));
                }
            }
            out.push('\n');
        }

        // Analyzer summaries
        for asummary in analyzer_summaries {
            out.push_str(&self.section(&format!("ANALYZER: {}", asummary.analyzer_name)));
            out.push_str(&format!(
                "  Packets analyzed: {}\n",
                asummary.packets_analyzed
            ));
            for (key, val) in &asummary.detail {
                out.push_str(&format!("  {key}: {val}\n"));
            }
            out.push('\n');
        }

        out
    }
}

impl TerminalReporter {
    fn section(&self, title: &str) -> String {
        if self.use_color {
            format!("{}\n{}\n", title.bold().underline(), "─".repeat(40))
        } else {
            format!("{title}\n{}\n", "─".repeat(40))
        }
    }
}
