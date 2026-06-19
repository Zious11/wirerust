//! Terminal-table reporter with ADR 0003 control-byte escaping.
//!
//! Renders the capture summary, per-host breakdown (gated by
//! `show_hosts_breakdown` per LESSON-P1.03), protocols, services,
//! findings, and per-analyzer detail tables for TTY output. The
//! `render: FindingsRender` field is a struct-of-two-orthogonal-enums
//! (`Grouping` × `Collapse`) that selects the FINDINGS rendering path:
//! `Grouping::Grouped` (MITRE tactic grouping, `--mitre`) or
//! `Grouping::Flat` (flat list); `Collapse::Collapsed` (default) or
//! `Collapse::Expanded` (`--no-collapse`). All four combinations are valid.
//! MITRE-tactic grouping reorganizes the FINDINGS section by MITRE ATT&CK
//! tactic and expands each finding's MITRE line with the technique name from
//! [`crate::mitre`].
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

/// Ascending sort rank for [`Verdict`]: lower value → appears first in a sorted bucket.
///
/// BC-2.11.014 / BC-2.11.033 PC-5: both `render_findings_grouped` and
/// `render_findings_grouped_collapsed` share this single source (BC-014 single-source rule).
fn verdict_rank(v: Verdict) -> u8 {
    match v {
        Verdict::Likely => 0,
        Verdict::Possible => 1,
        Verdict::Inconclusive => 2,
        Verdict::Unlikely => 3,
    }
}

/// Ascending sort rank for [`Confidence`]: lower value → appears first in a sorted bucket.
///
/// BC-2.11.014 / BC-2.11.033 PC-6: both grouped render functions share this single source.
fn confidence_rank(c: Confidence) -> u8 {
    match c {
        Confidence::High => 0,
        Confidence::Medium => 1,
        Confidence::Low => 2,
    }
}

/// Maximum number of representative evidence lines rendered per collapsed group.
///
/// BC-2.11.027 invariant 1: K = 3 is a hardcoded named constant, not a magic number.
/// The collapse pass inspects the first `min(N, COLLAPSE_EVIDENCE_SAMPLES)` members
/// of the group and emits `evidence[0]` from each inspected member (if non-empty).
/// The window is positional and does NOT slide past empty-evidence members.
///
const COLLAPSE_EVIDENCE_SAMPLES: usize = 3;

/// Collapse key: the four semantic fields that determine group membership.
///
/// Two findings belong to the same collapsed group when they share the same
/// `(category, verdict, confidence, summary)` four-tuple. The remaining fields
/// (`evidence`, `mitre_techniques`, `source_ip`, `timestamp`, `direction`) are
/// per-instance and are intentionally excluded from the key.
///
/// BC-2.11.025 invariant 7: `PartialEq` only — NO `Hash` derive (ThreatCategory,
/// Verdict, and Confidence do not derive Hash in v0.8.0). The collapse accumulator
/// is `Vec<(CollapseKey, Vec<&Finding>)>` with linear-scan equality matching.
///
#[derive(PartialEq, Eq)]
struct CollapseKey {
    category: crate::findings::ThreatCategory,
    verdict: Verdict,
    confidence: Confidence,
    summary: String,
}

/// Grouping axis for the FINDINGS section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Grouping {
    Grouped,
    Flat,
}

/// Collapse axis for the FINDINGS section.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Collapse {
    Collapsed,
    Expanded,
}

/// Rendering mode for the FINDINGS section of [`TerminalReporter`].
/// No [`Default`] is derived — deliberate, consistent with STORY-120.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FindingsRender {
    pub grouping: Grouping,
    pub collapse: Collapse,
}

pub struct TerminalReporter {
    pub use_color: bool,
    /// When true, render a per-host breakdown section listing each
    /// unique source/destination IP observed in the capture. Wired
    /// from the `summary` subcommand's `--hosts` flag — see
    /// LESSON-P1.03 in the brownfield-ingest Phase C synthesis. The
    /// always-present `Hosts: N` count line in the header is shown
    /// regardless; this gate only controls the expanded itemized list.
    pub show_hosts_breakdown: bool,
    /// Render mode for the FINDINGS section. A struct-of-two-orthogonal-enums:
    /// `grouping` (Grouped vs Flat) and `collapse` (Collapsed vs Expanded).
    /// All four combinations are valid (no combination is illegal).
    /// ADR-0003 Binding Rule 5; BC-2.11.013 / BC-2.11.026 / BC-2.11.027 / BC-2.11.028.
    pub render: FindingsRender,
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
        let mut proto_vec: Vec<_> = summary.protocol_counts().iter().collect();
        proto_vec.sort_by(|a, b| {
            b.1.cmp(a.1)
                .then_with(|| format!("{:?}", a.0).cmp(&format!("{:?}", b.0)))
        });
        for (proto, count) in &proto_vec {
            out.push_str(&format!("  {proto:?}: {count}\n"));
        }
        out.push('\n');

        // Services
        let services = summary.service_counts();
        if !services.is_empty() {
            out.push_str(&self.section("SERVICES"));
            let mut svc_vec: Vec<_> = services.iter().collect();
            svc_vec.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
            for (svc, count) in &svc_vec {
                out.push_str(&format!("  {svc}: {count}\n"));
            }
            out.push('\n');
        }

        // Findings
        if !findings.is_empty() {
            out.push_str(&self.section("FINDINGS"));
            match (self.render.grouping, self.render.collapse) {
                (Grouping::Grouped, Collapse::Expanded) => {
                    self.render_findings_grouped(&mut out, findings);
                }
                (Grouping::Grouped, Collapse::Collapsed) => {
                    self.render_findings_grouped_collapsed(&mut out, findings);
                }
                (Grouping::Flat, Collapse::Collapsed) => {
                    self.render_findings_collapsed(&mut out, findings);
                }
                (Grouping::Flat, Collapse::Expanded) => {
                    for f in findings {
                        self.render_finding_flat(&mut out, f);
                    }
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
                Verdict::Possible => line.yellow().to_string(),
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
    /// non-empty, shows comma-space joined technique IDs.
    /// BC-2.11.017: multi-ID renders as `MITRE: T1692.001, T0836`; empty vec → no line.
    fn render_finding_flat(&self, out: &mut String, f: &Finding) {
        self.render_finding_prefix(out, f);
        if !f.mitre_techniques.is_empty() {
            let ids = f.mitre_techniques.join(", ");
            out.push_str(&format!("    MITRE: {ids}\n"));
        }
    }

    /// Renders a single finding in the `--mitre` grouped view. MITRE line,
    /// if non-empty, expands to `ID — Name` for the first ID and `ID (unknown)`
    /// for IDs absent from [`crate::mitre::technique_name`]. Unknown IDs
    /// still render so they surface in audit trails; the regression test
    /// `known_emitted_technique_ids_resolve_in_lookup` in
    /// `tests/mitre_tests.rs` covers the hand-curated set of IDs we
    /// currently emit (see issue #67 for the trade-off rationale).
    fn render_finding_grouped(&self, out: &mut String, f: &Finding) {
        self.render_finding_prefix(out, f);
        if !f.mitre_techniques.is_empty() {
            // For the grouped view, show all IDs joined with comma-space.
            // Expand the first known ID with its name; unknown IDs get "(unknown)".
            let ids = f.mitre_techniques.join(", ");
            // Use the first technique ID for expanded name lookup in grouped view.
            match f
                .mitre_techniques
                .first()
                .and_then(|id| technique_name(id.as_str()))
            {
                Some(name) => out.push_str(&format!("    MITRE: {ids} \u{2014} {name}\n")),
                None => out.push_str(&format!("    MITRE: {ids} (unknown)\n")),
            }
        }
    }

    /// Shared collapse-logic helper: groups a slice of finding references by
    /// `CollapseKey` in first-occurrence order.
    ///
    /// This is the single source of collapse logic (ADR-0003 "Collapse-API Shape";
    /// F2 design-note §5.2.1). `collapse_findings_pass` (the flat-mode wrapper)
    /// delegates to this function. `render_findings_grouped_collapsed` calls this
    /// directly per bucket.
    ///
    /// BC-2.11.025 invariant 7 / postcondition 9: Vec accumulator is canonical —
    /// linear-scan `PartialEq`, no `HashMap`, no `IndexMap`.
    fn collapse_findings_pass_refs<'a>(
        &self,
        refs: &[&'a Finding],
    ) -> Vec<(CollapseKey, Vec<&'a Finding>)> {
        let mut groups: Vec<(CollapseKey, Vec<&'a Finding>)> = Vec::new();
        for f in refs {
            let key = CollapseKey {
                category: f.category,
                verdict: f.verdict,
                confidence: f.confidence,
                summary: f.summary.clone(),
            };
            // Linear-scan PartialEq: find an existing group or create a new one.
            if let Some(pos) = groups.iter().position(|(k, _)| k == &key) {
                groups[pos].1.push(f);
            } else {
                groups.push((key, vec![f]));
            }
        }
        groups
    }

    /// Flat-mode collapse adapter. Collects the `&[Finding]` parameter into
    /// `Vec<&Finding>` and delegates to `collapse_findings_pass_refs`.
    ///
    /// Uses the `findings` PARAMETER — `TerminalReporter` has no `findings` field.
    ///
    /// BC-2.11.025 / ADR-0003 "Collapse-API Shape".
    fn collapse_findings_pass<'a>(
        &self,
        findings: &'a [Finding],
    ) -> Vec<(CollapseKey, Vec<&'a Finding>)> {
        let refs: Vec<&Finding> = findings.iter().collect();
        self.collapse_findings_pass_refs(&refs)
    }

    /// Renders the FINDINGS section in collapsed flat mode.
    ///
    /// Calls `collapse_findings_pass` to build insertion-ordered groups, then for
    /// each group:
    ///   - N == 1 (singleton): delegates to `render_finding_flat` for byte-identical
    ///     output compared to pre-v0.8.0 (no count suffix, all evidence rendered, MITRE
    ///     line if non-empty).
    ///   - N >= 2: renders header with ` (xN)` suffix appended BEFORE colorization (so
    ///     the suffix is inside the ANSI color span), then up to K=3 sampled evidence
    ///     lines (first `min(N, COLLAPSE_EVIDENCE_SAMPLES)` members' `evidence[0]` in
    ///     positional order; the window does NOT slide past empty-evidence members), then
    ///     the MITRE line from `group_members[0]` if non-empty.
    ///
    /// BC-2.11.025 / BC-2.11.026 / BC-2.11.027 / BC-2.11.010.
    fn render_findings_collapsed(&self, out: &mut String, findings: &[Finding]) {
        let groups = self.collapse_findings_pass(findings);
        for (_key, members) in &groups {
            let n = members.len();
            if n == 1 {
                // Singleton: byte-identical to pre-v0.8.0 flat rendering.
                self.render_finding_flat(out, members[0]);
            } else {
                // N >= 2: build header with (xN) suffix, colorize the full string.
                let rep = members[0];
                let escaped_summary = escape_for_terminal(&rep.summary);
                let header_text = format!(
                    "[{}] {} ({}) - {} (x{})",
                    rep.category, rep.verdict, rep.confidence, escaped_summary, n
                );
                let colored = if self.use_color {
                    match rep.verdict {
                        Verdict::Likely => match rep.confidence {
                            Confidence::High => header_text.red().bold().to_string(),
                            _ => header_text.yellow().to_string(),
                        },
                        Verdict::Possible => header_text.yellow().to_string(),
                        Verdict::Inconclusive => header_text.cyan().to_string(),
                        Verdict::Unlikely => header_text.dimmed().to_string(),
                    }
                } else {
                    header_text
                };
                out.push_str(&format!("  {colored}\n"));

                // Evidence sampling: inspect first min(N, K) members positionally.
                // The window does NOT slide past empty-evidence members (BC-2.11.027 inv2).
                let window = members.len().min(COLLAPSE_EVIDENCE_SAMPLES);
                for member in &members[..window] {
                    if let Some(ev) = member.evidence.first() {
                        let escaped_ev = escape_for_terminal(ev);
                        out.push_str(&format!("    > {escaped_ev}\n"));
                    }
                }

                // MITRE line from group_members[0] only, if non-empty (BC-2.11.026 pc7).
                if !rep.mitre_techniques.is_empty() {
                    let ids = rep.mitre_techniques.join(", ");
                    out.push_str(&format!("    MITRE: {ids}\n"));
                }
            }
        }
    }

    /// Renders the FINDINGS section grouped by MITRE tactic. Each tactic
    /// header is `## Tactic Name`; sections appear in
    /// [`all_tactics_in_report_order`] order with an Uncategorized bucket
    /// last that holds findings with no technique or an unknown ID.
    /// Within each bucket, findings sort ascending by verdict-rank
    /// (Likely=0, Possible=1, Inconclusive=2, Unlikely=3), then ascending by
    /// confidence-rank (High=0, Medium=1, Low=2), then original emission order (stable).
    /// BC-2.11.013: tactic bucketing uses `mitre_techniques[0]` (first element).
    fn render_findings_grouped(&self, out: &mut String, findings: &[Finding]) {
        // Bucket by tactic. Attach original index for stable tertiary sort.
        let mut buckets: std::collections::HashMap<Option<MitreTactic>, Vec<(usize, &Finding)>> =
            std::collections::HashMap::new();
        for (i, f) in findings.iter().enumerate() {
            // BC-2.11.013: tactic grouping uses mitre_techniques[0] (first element).
            // Empty vec → None → Uncategorized. Unknown ID → None → Uncategorized.
            let tactic = f
                .mitre_techniques
                .first()
                .map(|id| id.as_str())
                .and_then(technique_tactic);
            buckets.entry(tactic).or_default().push((i, f));
        }

        // verdict_rank / confidence_rank: module-level single-source (BC-014).
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

    /// Renders the FINDINGS section grouped by MITRE tactic WITH per-bucket collapse.
    ///
    /// Per-tactic-bucket grouping identical to `render_findings_grouped` (BC-2.11.013),
    /// then per-bucket sort ascending by (verdict_rank, confidence_rank, emission-index)
    /// (BC-2.11.033 PC-5 / BC-2.11.014), then per-bucket
    /// `collapse_findings_pass_refs` (BC-2.11.033 Invariant 3), then collapsed
    /// group rendering with `(xN)` suffix (BC-2.11.031), K=3 evidence sampling
    /// (BC-2.11.032), and MITRE em-dash line from `members[0]` (BC-2.11.034).
    ///
    /// Singletons (N=1) render via `render_finding_grouped` (byte-identical to
    /// `{Grouped, Expanded}` for that finding).
    fn render_findings_grouped_collapsed(&self, out: &mut String, findings: &[Finding]) {
        // Step 1: bucket by tactic (same as render_findings_grouped, BC-2.11.013).
        let mut buckets: std::collections::HashMap<Option<MitreTactic>, Vec<(usize, &Finding)>> =
            std::collections::HashMap::new();
        for (i, f) in findings.iter().enumerate() {
            let tactic = f
                .mitre_techniques
                .first()
                .map(|id| id.as_str())
                .and_then(technique_tactic);
            buckets.entry(tactic).or_default().push((i, f));
        }

        // Sort each bucket ascending by (verdict_rank, confidence_rank, emission-index).
        // verdict_rank / confidence_rank: module-level single-source (BC-014).
        for (_, items) in buckets.iter_mut() {
            items.sort_by_key(|(idx, f)| {
                (verdict_rank(f.verdict), confidence_rank(f.confidence), *idx)
            });
        }

        // Step 3: render buckets in all_tactics_in_report_order(), Uncategorized last.
        let render_collapsed_bucket = |out: &mut String, items: &[(usize, &Finding)]| {
            // Build sorted &Finding refs and delegate to the shared collapse helper.
            // BC-2.11.033 Inv3 / BC-2.11.025 Inv1 / BC-2.11.031 PC-3: use the shared
            // four-tuple (category, verdict, confidence, summary) collapse key via
            // collapse_findings_pass_refs — no inline summary-only keying.
            let bucket_refs: Vec<&Finding> = items.iter().map(|(_, f)| *f).collect();
            let groups = self.collapse_findings_pass_refs(&bucket_refs);
            for (_key, members) in &groups {
                let n = members.len();
                if n == 1 {
                    // Singleton: byte-identical to {Grouped, Expanded}.
                    self.render_finding_grouped(out, members[0]);
                } else {
                    // N >= 2: build header with (xN) suffix, colorize the full
                    // string (suffix is inside the ANSI color span — BC-2.11.031 PC-3).
                    let rep = members[0];
                    // escape_for_terminal (char::escape_default, U+NN format) —
                    // BC-2.11.031 PC-5 / BC-2.11.032 PC-6 / VP-012 / ADR-0003.
                    let escaped_summary = escape_for_terminal(&rep.summary);
                    // Display format (`{}`) for verdict and confidence → UPPERCASE output
                    // (`LIKELY`, `HIGH`) per BC-2.11.031 PC-1 / canonical vector.
                    let header_text = format!(
                        "[{}] {} ({}) - {} (x{})",
                        rep.category, rep.verdict, rep.confidence, escaped_summary, n
                    );
                    let colored = if self.use_color {
                        match rep.verdict {
                            Verdict::Likely => match rep.confidence {
                                Confidence::High => header_text.red().bold().to_string(),
                                _ => header_text.yellow().to_string(),
                            },
                            Verdict::Possible => header_text.yellow().to_string(),
                            Verdict::Inconclusive => header_text.cyan().to_string(),
                            Verdict::Unlikely => header_text.dimmed().to_string(),
                        }
                    } else {
                        header_text
                    };
                    out.push_str(&format!("  {colored}\n"));

                    // Evidence sampling: first min(N, K) members positionally.
                    // Window does NOT slide past empty-evidence members (BC-2.11.032 Inv2).
                    // escape_for_terminal (char::escape_default) — BC-2.11.032 PC-6 / VP-012.
                    let window = members.len().min(COLLAPSE_EVIDENCE_SAMPLES);
                    for member in &members[..window] {
                        if let Some(ev) = member.evidence.first() {
                            let escaped_ev = escape_for_terminal(ev);
                            out.push_str(&format!("    > {escaped_ev}\n"));
                        }
                    }

                    // MITRE line from members[0] with em-dash name expansion (BC-2.11.034).
                    if !rep.mitre_techniques.is_empty() {
                        let ids = rep.mitre_techniques.join(", ");
                        match rep
                            .mitre_techniques
                            .first()
                            .and_then(|id| technique_name(id.as_str()))
                        {
                            Some(name) => {
                                out.push_str(&format!("    MITRE: {ids} \u{2014} {name}\n"))
                            }
                            None => out.push_str(&format!("    MITRE: {ids} (unknown)\n")),
                        }
                    }
                }
            }
        };

        for tactic in all_tactics_in_report_order() {
            if let Some(items) = buckets.get(&Some(*tactic)) {
                out.push_str(&format!("  ## {tactic}\n"));
                render_collapsed_bucket(out, items);
            }
        }
        if let Some(items) = buckets.get(&None) {
            out.push_str("  ## Uncategorized\n");
            render_collapsed_bucket(out, items);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::escape_for_terminal;
    use proptest::prelude::*;

    // VP-012: escape_for_terminal Correctness (BC-2.11.007..012).
    // proptest harnesses over the full Unicode String space. 1000 cases per
    // property to match the VP-010 sibling convention.
    proptest! {
        #![proptest_config(ProptestConfig { cases: 1000, ..ProptestConfig::default() })]

        // VP-012 property 1 + 5: no dangerous bytes survive.
        //
        // The security goal (ADR 0003) is that no raw C0/DEL/C1 *control*
        // character ever reaches the terminal, and that backslash is neutralized.
        // `escape_for_terminal` uses `char::escape_default`, so an escaped
        // backslash is rendered as the two-char sequence `\\` and a control char
        // as `\u{..}`/`\n`/`\t`/etc. Therefore backslash bytes DO legitimately
        // appear in the output as escape-sequence syntax — a per-char
        // `c != '\\'` check is not the property and is unsatisfiable by design.
        //
        // The faithful, non-tautological property is:
        //   (a) no C0/DEL control char survives,
        //   (b) no C1 codepoint (U+0080-U+009F) survives, and
        //   (c) every backslash in the output is a well-formed escape
        //       *introducer* — it is followed by a valid escape continuation
        //       char (one of `n t r 0 ' " \\ u`). This proves there is no raw or
        //       dangling backslash that could leak a control byte; the only
        //       backslashes present are inert escape syntax.
        #[test]
        fn prop_no_dangerous_bytes_survive(s: String) {
            let escaped = escape_for_terminal(&s);
            let chars: Vec<char> = escaped.chars().collect();
            let mut i = 0;
            while i < chars.len() {
                let c = chars[i];
                prop_assert!(
                    !c.is_ascii_control(),
                    "C0/DEL control char U+{:04X} survived in {:?}",
                    c as u32,
                    escaped
                );
                prop_assert!(
                    !(('\u{80}'..='\u{9f}').contains(&c)),
                    "C1 control char U+{:04X} survived in {:?}",
                    c as u32,
                    escaped
                );
                if c == '\\' {
                    // A backslash must introduce a valid escape sequence: it
                    // cannot be dangling at the end and must be followed by a
                    // recognized escape continuation char. The pair is consumed
                    // together so the second char of an escaped backslash (`\\`)
                    // is not itself mistaken for a dangling backslash.
                    let next = chars.get(i + 1);
                    prop_assert!(
                        next.is_some(),
                        "dangling raw backslash at end of output {:?}",
                        escaped
                    );
                    let n = *next.unwrap();
                    // This is the EXACT continuation set `char::escape_default`
                    // emits for the inputs this function escapes (C0, DEL, C1,
                    // backslash): `\n` `\t` `\r` for those three control chars,
                    // `\\` for backslash, and `\u{..}` (introducer `u`) for every
                    // other control/C1 codepoint. NUL escapes to `\u{0}` (not
                    // `\0`), and quotes are never escaped, so `0`/`'`/`"` can
                    // never appear here. Keeping the set exact (not a superset)
                    // means a future change that emitted a different escape form
                    // would fail this assertion instead of being silently accepted.
                    prop_assert!(
                        matches!(n, 'n' | 't' | 'r' | '\\' | 'u'),
                        "backslash not part of a valid escape sequence (followed by {:?}) in {:?}",
                        n,
                        escaped
                    );
                    // Consume the introducer + its continuation char as a unit.
                    i += 2;
                    continue;
                }
                i += 1;
            }
        }

        // VP-012 property 2: printable ASCII (U+0020-U+007E except backslash)
        // passes through byte-for-byte unchanged.
        #[test]
        fn prop_printable_ascii_unchanged(s: String) {
            let ascii_only: String = s
                .chars()
                .filter(|c| c.is_ascii() && !c.is_ascii_control() && *c != '\\')
                .collect();
            let escaped = escape_for_terminal(&ascii_only);
            prop_assert_eq!(escaped, ascii_only, "printable ASCII was modified");
        }

        // VP-012 property 3: valid non-ASCII Unicode strictly above the C1
        // range (> U+009F) passes through unchanged (Cyrillic, CJK, emoji...).
        #[test]
        fn prop_non_ascii_unicode_above_c1_unchanged(s: String) {
            let unicode_only: String = s
                .chars()
                .filter(|c| !c.is_ascii() && *c > '\u{9f}')
                .collect();
            let escaped = escape_for_terminal(&unicode_only);
            prop_assert_eq!(escaped, unicode_only, "safe non-ASCII Unicode was escaped");
        }

        // VP-012 property 4: escaping is length-conserving and expands exactly
        // (CR-006 — the old `from_utf8(...).is_ok()` check was tautological since
        // `String` is valid UTF-8 by type).
        //
        // Falsifiable contract: the escaped output length, measured in `char`s,
        // equals the sum over the input chars of each char's INDIVIDUAL escaped
        // length, computed by an independent oracle. Because every char's escape
        // form is >= 1 char, this also proves `escaped.chars().count() >=
        // s.chars().count()` (escaping never shrinks). This FAILS if the function
        // ever silently drops a char (total too small), passes a dangerous char
        // through unescaped (escaped char too short vs oracle), or mis-expands a
        // safe char (escaped char too long vs oracle).
        #[test]
        fn prop_escape_is_length_conserving(s: String) {
            // Oracle: per-char escaped length, mirroring escape_for_terminal's
            // branch — escape_default for C0/DEL/C1/backslash, else the char as-is.
            fn expected_escaped_len(c: char) -> usize {
                if c.is_ascii_control() || ('\u{80}'..='\u{9f}').contains(&c) || c == '\\' {
                    c.escape_default().count()
                } else {
                    1
                }
            }
            let expected_total: usize = s.chars().map(expected_escaped_len).sum();
            let escaped = escape_for_terminal(&s);
            let actual_total = escaped.chars().count();

            prop_assert_eq!(
                actual_total,
                expected_total,
                "escaped length {} != oracle-predicted {} for input {:?} -> {:?}",
                actual_total,
                expected_total,
                s,
                escaped
            );
            // Direct never-shrinks corollary (strict superset of byte content).
            prop_assert!(
                actual_total >= s.chars().count(),
                "escaping shrank the output: {} chars in, {} chars out",
                s.chars().count(),
                actual_total
            );
        }
    }

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
