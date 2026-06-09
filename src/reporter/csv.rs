//! CSV reporter — flat tabular rendering of the findings list
//! (LESSON-P2.03).
//!
//! Emits the `findings` slice as an RFC 4180 CSV table, one row per
//! [`Finding`], with a fixed header. This is the analyst-pipe-into-a-
//! spreadsheet output channel.
//!
//! ## Scope: findings only
//!
//! CSV is a flat row-oriented format. The capture-level [`Summary`]
//! (protocol / service / host maps) and the per-analyzer
//! [`AnalysisSummary`] detail blocks are nested and heterogeneous —
//! they do not map cleanly onto a single CSV table. The CSV reporter
//! therefore renders **only** the findings table; callers who need the
//! summary / analyzer data should use the JSON or terminal reporter.
//! This is an intentional, documented limitation, not an oversight.
//!
//! ## CSV-injection neutralization
//!
//! `Finding::summary` and `Finding::evidence` carry attacker-controlled
//! bytes from packet payloads (see ADR 0003). When a CSV is opened in a
//! spreadsheet application (Excel, LibreOffice, Google Sheets), a cell
//! whose value begins with `=`, `+`, `-`, `@`, TAB, or CR is
//! interpreted as a *formula* — a well-known "CSV injection" /
//! "formula injection" vector (OWASP). Because wirerust is a forensics
//! tool that deliberately surfaces hostile input, every field is run
//! through [`neutralize_csv_injection`], which prepends a single quote
//! to any cell starting with a formula-trigger character. This mirrors
//! the terminal reporter's control-byte escaping: the raw [`Finding`]
//! keeps unmodified bytes; the display/export layer sanitizes.

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reporter::Reporter;
use crate::summary::Summary;

/// Prepend a single quote to any value that a spreadsheet would
/// otherwise interpret as a formula. See the module docs for the
/// CSV-injection rationale.
fn neutralize_csv_injection(s: &str) -> String {
    match s.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{s}"),
        _ => s.to_string(),
    }
}

/// CSV reporter. See the module documentation for the findings-only
/// scope and the CSV-injection neutralization policy.
pub struct CsvReporter;

impl Reporter for CsvReporter {
    fn render(
        &self,
        _summary: &Summary,
        findings: &[Finding],
        _analyzer_summaries: &[AnalysisSummary],
    ) -> String {
        let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());

        // Fixed header. Order is stable so downstream parsers can rely
        // on column positions.
        writer
            .write_record([
                "category",
                "verdict",
                "confidence",
                "summary",
                "evidence",
                "mitre_techniques",
                "source_ip",
                "direction",
                "timestamp",
            ])
            .expect("writing CSV header to an in-memory buffer cannot fail");

        for f in findings {
            // `evidence` is a Vec<String>; flatten with "; " so the
            // whole list lives in one cell. The csv crate quotes the
            // cell automatically if it contains the separator, commas,
            // quotes, or newlines (RFC 4180).
            let evidence = f.evidence.join("; ");
            // ADR-006 Decision 13 §13.3: semicolon-join for multi-technique vecs.
            // Empty vec → "" (empty string, NOT "null"/"[]"/"N/A").
            // EC-015 consumer guard: `"".split(';')` in downstream tooling produces
            // `[""]` (one empty element), not `[]`. Consumers MUST guard:
            //   `if cell.is_empty() { return vec![] }` before splitting on ';'.
            let mitre = f.mitre_techniques.join(";");
            let source_ip = f.source_ip.map(|ip| ip.to_string()).unwrap_or_default();
            let direction = f.direction.map(|d| format!("{d:?}")).unwrap_or_default();
            let timestamp = f.timestamp.map(|t| t.to_rfc3339()).unwrap_or_default();

            writer
                .write_record([
                    neutralize_csv_injection(&f.category.to_string()),
                    neutralize_csv_injection(&f.verdict.to_string()),
                    neutralize_csv_injection(&f.confidence.to_string()),
                    neutralize_csv_injection(&f.summary),
                    neutralize_csv_injection(&evidence),
                    neutralize_csv_injection(&mitre),
                    neutralize_csv_injection(&source_ip),
                    neutralize_csv_injection(&direction),
                    neutralize_csv_injection(&timestamp),
                ])
                .expect("writing a CSV record to an in-memory buffer cannot fail");
        }

        let bytes = writer
            .into_inner()
            .expect("flushing the in-memory CSV buffer cannot fail");
        String::from_utf8(bytes).expect("CSV content is built from UTF-8 String inputs")
    }
}
