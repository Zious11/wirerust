//! Terminal-table reporter with ADR 0003 control-byte escaping.
//!
//! Renders the capture summary, per-host breakdown (gated by
//! `show_hosts_breakdown` per LESSON-P1.03), protocols, services,
//! findings, and per-analyzer detail tables for TTY output. Optional
//! MITRE-tactic grouping (`show_mitre_grouping`) reorganizes the
//! FINDINGS section by MITRE ATT&CK tactic and expands each finding's
//! MITRE line with the technique name from [`crate::mitre`].
//!
//! Per ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`), every
//! attacker-controlled string (Finding `summary`/`evidence`, analyzer
//! detail values) is run through [`escape_for_terminal`] before being
//! written to the output buffer. The pure-data [`Finding`] type carries
//! raw bytes; only this layer escapes them.

use owo_colors::OwoColorize;

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, Verdict};
use crate::mitre::{MitreTactic, all_tactics_in_report_order, technique_name, technique_tactic};
use crate::reporter::Reporter;
use crate::summary::Summary;

/// Escape control bytes (C0 + DEL + C1 + backslash) for safe terminal display.
///
/// Iterates the input string's characters and applies `char::escape_default`
/// when the character matches `char::is_ascii_control()` (C0 + DEL), falls in
/// the C1 range `U+0080..=U+009F`, or is a backslash. All other characters —
/// printable ASCII and valid non-ASCII Unicode (Cyrillic, CJK, emoji) — pass
/// through unchanged.
///
/// **Why C1?** Codepoints U+0080–U+009F (C1 controls like NEL U+0085 and CSI
/// U+009B) can be encoded in valid UTF-8 as two-byte sequences (e.g., U+009B
/// as `0xC2 0x9B`) and survive `String::from_utf8_lossy`. Most modern
/// terminals in UTF-8 mode do NOT interpret these as controls by default, but
/// the DEC S8C1T mode and some legacy terminals can treat U+009B as an 8-bit
/// equivalent of ESC[. Escaping them closes a narrow but real vector.
///
/// **Why not `str::escape_default`?** It routes *every* character through
/// `char::escape_default`, which escapes non-ASCII as `\u{...}` and would
/// mangle a Cyrillic hostname like `пример.рф` into `\u{43f}\u{440}...`.
/// See ADR 0003 (`docs/adr/0003-reporting-pipeline-layering.md`) for the
/// layering rationale and the empirical verification.
fn escape_for_terminal(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        // Escape C0 (0x00-0x1f), DEL (0x7f), C1 (U+0080-U+009F), and backslash.
        // C1 codepoints like U+0085 (NEL) and U+009B (CSI — 8-bit equivalent of
        // ESC[) are valid multi-byte UTF-8 in a `String` but can be interpreted
        // as controls by terminals in 8-bit C1 mode (DEC S8C1T). Escaping them
        // is cheap insurance.
        if c.is_ascii_control() || ('\u{80}'..='\u{9f}').contains(&c) || c == '\\' {
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
    /// When true, regroup the FINDINGS section by MITRE tactic and expand
    /// the per-finding MITRE line to include the technique name.
    pub show_mitre_grouping: bool,
    /// When true, render a per-host breakdown section listing each
    /// unique source/destination IP observed in the capture. Wired
    /// from the `summary` subcommand's `--hosts` flag — see
    /// LESSON-P1.03 in the brownfield-ingest Phase C synthesis. The
    /// always-present `Hosts: N` count line in the header is shown
    /// regardless; this gate only controls the expanded itemized list.
    pub show_hosts_breakdown: bool,
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

        // LESSON-P1.03: optional per-host breakdown, gated by the
        // `summary --hosts` flag. Renders one line per unique
        // source/destination IP observed in the capture, in the
        // sorted-by-address order returned by `Summary::unique_hosts()`.
        // The always-on `Hosts: N` count in the header is kept; this
        // section is the expanded itemized list.
        if self.show_hosts_breakdown {
            let hosts = summary.unique_hosts();
            if !hosts.is_empty() {
                out.push_str(&self.section("HOSTS"));
                for host in &hosts {
                    out.push_str(&format!("  {host}\n"));
                }
                out.push('\n');
            }
        }

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
            if self.show_mitre_grouping {
                self.render_findings_grouped(&mut out, findings);
            } else {
                // Per ADR 0003: the Finding struct stores raw bytes; the
                // terminal reporter is responsible for escaping untrusted
                // content (summary + evidence) before writing to a TTY.
                for f in findings {
                    self.render_finding_flat(&mut out, f);
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
                // Per ADR 0003: analyzer summary detail values can contain
                // attacker-controlled bytes (e.g., top_hosts, top_snis,
                // recent_uris from HTTP/TLS). serde_json's Display impl
                // escapes C0 + DEL per RFC 8259 but passes C1 codepoints
                // (U+0080-U+009F) through as raw UTF-8 — so we still need
                // to run the JSON rendering through escape_for_terminal to
                // close the C1 gap that U+009B (CSI) exploits.
                let escaped_val = escape_for_terminal(&val.to_string());
                out.push_str(&format!("  {key}: {escaped_val}\n"));
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

    /// Emits the shared per-finding prefix: the colored `[category] verdict
    /// (confidence) - summary` header line followed by each evidence line.
    /// Summary and evidence strings are escaped per ADR 0003 before being
    /// written to the TTY. Callers own the trailing MITRE line — see
    /// [`render_finding_flat`] and [`render_finding_grouped`].
    fn render_finding_prefix(&self, out: &mut String, f: &Finding) {
        let escaped_summary = escape_for_terminal(&f.summary);
        let line = format!(
            "[{}] {} ({}) - {}",
            f.category, f.verdict, f.confidence, escaped_summary
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
            let escaped_ev = escape_for_terminal(ev);
            out.push_str(&format!("    > {escaped_ev}\n"));
        }
    }

    /// Renders a single finding in the default flat view. MITRE line, if
    /// present, shows just the technique ID.
    fn render_finding_flat(&self, out: &mut String, f: &Finding) {
        self.render_finding_prefix(out, f);
        if let Some(ref t) = f.mitre_technique {
            out.push_str(&format!("    MITRE: {t}\n"));
        }
    }

    /// Renders a single finding in the `--mitre` grouped view. MITRE line,
    /// if present, expands to `ID — Name` for known IDs and `ID (unknown)`
    /// for IDs absent from [`crate::mitre::technique_name`]. Unknown IDs
    /// still render so they surface in audit trails; the regression test
    /// `known_emitted_technique_ids_resolve_in_lookup` in
    /// `tests/mitre_tests.rs` covers the hand-curated set of IDs we
    /// currently emit (see issue #67 for the trade-off rationale).
    fn render_finding_grouped(&self, out: &mut String, f: &Finding) {
        self.render_finding_prefix(out, f);
        if let Some(ref id) = f.mitre_technique {
            match technique_name(id) {
                Some(name) => out.push_str(&format!("    MITRE: {id} \u{2014} {name}\n")),
                None => out.push_str(&format!("    MITRE: {id} (unknown)\n")),
            }
        }
    }

    /// Renders the FINDINGS section grouped by MITRE tactic. Each tactic
    /// header is `## Tactic Name`; sections appear in
    /// [`all_tactics_in_report_order`] order with an Uncategorized bucket
    /// last that holds findings with no technique or an unknown ID.
    /// Within each bucket, findings sort by verdict-desc, then
    /// confidence-desc, then original emission order (stable).
    fn render_findings_grouped(&self, out: &mut String, findings: &[Finding]) {
        // Bucket by tactic. Attach original index for stable tertiary sort.
        let mut buckets: std::collections::HashMap<Option<MitreTactic>, Vec<(usize, &Finding)>> =
            std::collections::HashMap::new();
        for (i, f) in findings.iter().enumerate() {
            let tactic = f.mitre_technique.as_deref().and_then(technique_tactic);
            buckets.entry(tactic).or_default().push((i, f));
        }

        fn verdict_rank(v: Verdict) -> u8 {
            match v {
                Verdict::Likely => 0,
                Verdict::Inconclusive => 1,
                Verdict::Unlikely => 2,
            }
        }
        fn confidence_rank(c: Confidence) -> u8 {
            match c {
                Confidence::High => 0,
                Confidence::Medium => 1,
                Confidence::Low => 2,
            }
        }

        for (_, items) in buckets.iter_mut() {
            items.sort_by_key(|(idx, f)| {
                (verdict_rank(f.verdict), confidence_rank(f.confidence), *idx)
            });
        }

        for tactic in all_tactics_in_report_order() {
            if let Some(items) = buckets.get(&Some(*tactic)) {
                out.push_str(&format!("  ## {tactic}\n"));
                for (_, f) in items {
                    self.render_finding_grouped(out, f);
                }
            }
        }
        if let Some(items) = buckets.get(&None) {
            out.push_str("  ## Uncategorized\n");
            for (_, f) in items {
                self.render_finding_grouped(out, f);
            }
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

    #[test]
    fn escapes_c1_nel_and_csi() {
        // U+0085 (NEL, Next Line) and U+009B (CSI, 8-bit Control Sequence
        // Introducer). Both are valid multi-byte UTF-8 and survive
        // String::from_utf8_lossy — must be escaped to avoid 8-bit terminal
        // control interpretation.
        assert_eq!(escape_for_terminal("line1\u{85}line2"), "line1\\u{85}line2");
        assert_eq!(
            escape_for_terminal("before\u{9b}31mafter"),
            "before\\u{9b}31mafter"
        );
    }

    #[test]
    fn escapes_c1_range_boundaries() {
        // U+0080 (start of C1) and U+009F (end of C1) must both escape.
        // U+00A0 (NBSP, just past C1) must pass through unchanged — it's a
        // legitimate printable whitespace character, not a control code.
        assert_eq!(escape_for_terminal("\u{80}"), "\\u{80}");
        assert_eq!(escape_for_terminal("\u{9f}"), "\\u{9f}");
        assert_eq!(escape_for_terminal("\u{a0}"), "\u{a0}");
    }
}
