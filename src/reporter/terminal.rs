use owo_colors::OwoColorize;

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, Verdict};
use crate::reporter::Reporter;
use crate::summary::Summary;

/// Escape control bytes (C0 + DEL + backslash) for safe terminal display.
///
/// Iterates the input string's characters and applies `char::escape_default`
/// only when the character matches `char::is_ascii_control()` or is a
/// backslash. All other characters — printable ASCII and valid non-ASCII
/// Unicode (Cyrillic, CJK, emoji) — pass through unchanged.
///
/// Why not `str::escape_default`? It routes *every* character through
/// `char::escape_default`, which escapes non-ASCII as `\u{...}` and
/// would mangle a Cyrillic hostname like `пример.рф` into
/// `\u{43f}\u{440}...`. See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`)
/// for the layering rationale and the empirical verification.
fn escape_for_terminal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_control() || c == '\\' {
            for e in c.escape_default() {
                out.push(e);
            }
        } else {
            out.push(c);
        }
    }
    out
}

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
            let warning = format!(
                "  Skipped: {} packets (decode errors)\n",
                summary.skipped_packets
            );
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
                // Per ADR 0003: the Finding struct stores raw bytes; the
                // terminal reporter is responsible for escaping untrusted
                // content (summary + evidence) before writing to a TTY.
                let safe_summary = escape_for_terminal(&f.summary);
                let line = format!(
                    "[{}] {} ({}) - {}",
                    f.category, f.verdict, f.confidence, safe_summary
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
                    let safe_ev = escape_for_terminal(ev);
                    out.push_str(&format!("    > {safe_ev}\n"));
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

#[cfg(test)]
mod tests {
    use super::escape_for_terminal;

    #[test]
    fn escapes_esc_byte() {
        assert_eq!(
            escape_for_terminal("\x1b[31mRED\x1b[0m"),
            "\\u{1b}[31mRED\\u{1b}[0m"
        );
    }

    #[test]
    fn escapes_bel_and_del() {
        assert_eq!(
            escape_for_terminal("ring\x07bye\x7f"),
            "ring\\u{7}bye\\u{7f}"
        );
    }

    #[test]
    fn escapes_tab_newline_cr_as_short_forms() {
        // char::escape_default uses short escapes for these three.
        assert_eq!(
            escape_for_terminal("tab\there\nnewline\rreturn"),
            "tab\\there\\nnewline\\rreturn"
        );
    }

    #[test]
    fn escapes_backslash() {
        assert_eq!(escape_for_terminal("a\\b"), "a\\\\b");
    }

    #[test]
    fn preserves_printable_ascii() {
        assert_eq!(
            escape_for_terminal("hello world 123 !@#"),
            "hello world 123 !@#"
        );
    }

    #[test]
    fn preserves_cyrillic() {
        assert_eq!(escape_for_terminal("пример.рф"), "пример.рф");
    }

    #[test]
    fn preserves_emoji() {
        assert_eq!(escape_for_terminal("crab 🦀 rust"), "crab 🦀 rust");
    }

    #[test]
    fn mixed_content_escapes_only_dangerous_bytes() {
        // Cyrillic + ESC injection + emoji — Cyrillic and emoji must survive,
        // only the ESC sequence should be escaped.
        assert_eq!(
            escape_for_terminal("пример\x1b[31m🦀"),
            "пример\\u{1b}[31m🦀"
        );
    }

    #[test]
    fn empty_string_is_empty() {
        assert_eq!(escape_for_terminal(""), "");
    }
}
