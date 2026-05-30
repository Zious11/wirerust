//! STORY-077: TerminalReporter formalization tests — Wave 21
//!
//! Formalizes 14 tests for BC-2.11.006 through BC-2.11.012 (AC-001..AC-014).
//!
//! Behavioral contracts covered:
//!   BC-2.11.006  TerminalReporter Shows Skipped: N Packets Only When N > 0
//!   BC-2.11.007  TerminalReporter Escapes C0+DEL+C1+Backslash in Finding Summary and Evidence
//!   BC-2.11.008  TerminalReporter Escape Preserves Printable ASCII and UTF-8
//!   BC-2.11.009  TerminalReporter Escapes C1 Codepoints U+0080-U+009F; U+00A0 Preserved
//!   BC-2.11.010  TerminalReporter Escapes Both Summary AND Each Evidence Line
//!   BC-2.11.011  TerminalReporter Escapes Analyzer-Summary Detail Values
//!   BC-2.11.012  TerminalReporter End-to-End: C1 CSI in Path-Traversal Finding Escaped
//!
//! implementation_strategy: brownfield-formalization
//! All tests are expected to PASS at the Green Gate because the production
//! implementation already satisfies all ACs. Any FAIL indicates a real gap.
//!
//! NOTE: `escape_for_terminal` is a private function in src/reporter/terminal.rs
//! and cannot be called directly from integration tests. All AC-003..AC-010
//! tests that target its behavior exercise it indirectly through
//! `TerminalReporter::render`, using controlled Finding/AnalysisSummary inputs
//! and asserting on the rendered String. This is the correct integration-test
//! approach per ADR 0003.
//!
//! VP-012 deferred to Phase-6 (formal verification of the escape predicate
//! requires symbolic execution tooling not yet integrated into the pipeline).
//!
//! Intended JSON↔Terminal C1 asymmetry preserved: JsonReporter passes C1
//! codepoints through as raw UTF-8 (per BC-2.11.005 / RFC 8259 scope);
//! TerminalReporter escapes them (BC-2.11.007, BC-2.11.009). Both behaviors
//! are correct and intentional per ADR 0003.

// PG-W17-001 mandates that test fn names EXACTLY match the AC `**Test:**`
// citations (e.g. `test_BC_2_11_006_skipped_packets_zero_no_line`).  These
// names use upper-case BC identifiers which Rust flags as non-snake-case.
// Suppress the lint for this file rather than diverge from the naming scheme.
#![allow(non_snake_case)]

use wirerust::analyzer::AnalysisSummary;
use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reporter::Reporter;
use wirerust::reporter::terminal::TerminalReporter;
use wirerust::summary::Summary;

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Minimal Finding with no optional fields set.
fn make_finding(summary: impl Into<String>) -> Finding {
    Finding {
        category: ThreatCategory::Anomaly,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: summary.into(),
        evidence: vec![],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
        direction: None,
    }
}

/// TerminalReporter with color and all optional sections disabled.
fn plain_reporter() -> TerminalReporter {
    TerminalReporter {
        use_color: false,
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
    }
}

/// Render the given findings and analyzer summaries against an empty Summary.
fn render(findings: &[Finding], analyzers: &[AnalysisSummary]) -> String {
    plain_reporter().render(&Summary::new(), findings, analyzers)
}

// Per DF-TEST-NAMESPACE-001: all STORY-077 tests are grouped inside a
// dedicated `mod story_077` wrapper to prevent test-function name collisions
// with other stories' BC-prefixed names.
mod story_077 {
    use super::*;

    // -----------------------------------------------------------------------
    // BC-2.11.006 — skipped_packets conditional display
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.006 postcondition 2): When `Summary.skipped_packets = 0`,
    /// the rendered output does NOT contain the string "Skipped:".
    #[test]
    fn test_BC_2_11_006_skipped_packets_zero_no_line() {
        let mut summary = Summary::new();
        summary.skipped_packets = 0;
        let out = plain_reporter().render(&summary, &[], &[]);
        assert!(
            !out.contains("Skipped:"),
            "BC-2.11.006 pc2: 'Skipped:' must not appear when skipped_packets = 0; got: {out}"
        );
    }

    /// AC-002 (BC-2.11.006 postcondition 1): When `Summary.skipped_packets = 5`,
    /// the output contains `"Skipped: 5 packets (decode errors)"`.
    #[test]
    fn test_BC_2_11_006_skipped_packets_nonzero_line_present() {
        let mut summary = Summary::new();
        summary.skipped_packets = 5;
        let out = plain_reporter().render(&summary, &[], &[]);
        assert!(
            out.contains("Skipped: 5 packets (decode errors)"),
            "BC-2.11.006 pc1: expected 'Skipped: 5 packets (decode errors)' in output; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.007 — C0 + DEL + backslash escaping
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.11.007 postcondition 1): ESC (0x1B) in a Finding summary
    /// does NOT appear as a raw byte in the rendered output; it is escaped via
    /// `char::escape_default` (i.e., appears as `\u{1b}` in the output).
    #[test]
    fn test_BC_2_11_007_esc_byte_escaped() {
        let f = make_finding("payload: \x1b[31mRED\x1b[0m");
        let out = render(&[f], &[]);
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.007 pc1: raw ESC byte (0x1b) must not appear in terminal output; got: {out:?}"
        );
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.007 pc1: escaped form '\\u{{1b}}' must appear in output; got: {out}"
        );
    }

    /// AC-004 (BC-2.11.007 postcondition 2): DEL (0x7F) in a Finding summary
    /// is escaped; raw 0x7F does NOT appear in the rendered output.
    #[test]
    fn test_BC_2_11_007_del_escaped() {
        let f = make_finding("before\x7fafter");
        let out = render(&[f], &[]);
        assert!(
            !out.as_bytes().contains(&0x7f),
            "BC-2.11.007 pc2: raw DEL byte (0x7f) must not appear in terminal output; got: {out:?}"
        );
        assert!(
            out.contains("\\u{7f}"),
            "BC-2.11.007 pc2: escaped form '\\u{{7f}}' must appear in output; got: {out}"
        );
    }

    /// AC-005 (BC-2.11.007 postcondition 4): Backslash (0x5C) in a Finding
    /// summary is converted to `\\` (double-backslash); the raw single backslash
    /// does NOT pass through.
    #[test]
    fn test_BC_2_11_007_backslash_escaped() {
        let f = make_finding("path: C:\\Users\\victim");
        let out = render(&[f], &[]);
        // The escaped output must contain double-backslash for each original backslash.
        assert!(
            out.contains("C:\\\\Users\\\\victim"),
            "BC-2.11.007 pc4: backslash must be doubled to '\\\\\\\\' in output; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.008 — printable ASCII and UTF-8 preservation
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.11.008 postcondition 1): Printable ASCII characters
    /// (0x20-0x7E, excluding backslash 0x5C) pass through `escape_for_terminal`
    /// unchanged and appear verbatim in the rendered output.
    #[test]
    fn test_BC_2_11_008_printable_ascii_preserved() {
        // Full printable ASCII range excluding backslash (0x5c).
        let ascii: String = (0x20u8..=0x7eu8)
            .filter(|&b| b != b'\\')
            .map(|b| b as char)
            .collect();
        let f = make_finding(ascii.clone());
        let out = render(&[f], &[]);
        assert!(
            out.contains(&ascii),
            "BC-2.11.008 pc1: printable ASCII (excl. backslash) must pass through unchanged; got: {out}"
        );
    }

    /// AC-007 (BC-2.11.008 postcondition 2): Cyrillic, emoji, and other
    /// non-ASCII Unicode codepoints at U+00A0 and above pass through
    /// `escape_for_terminal` unchanged.
    #[test]
    fn test_BC_2_11_008_cyrillic_and_emoji_preserved() {
        let f = make_finding("host: пример.рф 🦀");
        let out = render(&[f], &[]);
        assert!(
            out.contains("пример.рф"),
            "BC-2.11.008 pc2: Cyrillic must pass through unchanged; got: {out}"
        );
        assert!(
            out.contains("🦀"),
            "BC-2.11.008 pc2: emoji must pass through unchanged; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.009 — C1 range escaping and NBSP preservation
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.11.009 postcondition 1): All codepoints in U+0080-U+009F
    /// (C1 range, inclusive) are replaced by `char::escape_default` output.
    /// Spot-checks U+0085 (NEL → `\u{85}`) and U+009B (CSI → `\u{9b}`).
    #[test]
    fn test_BC_2_11_009_c1_range_escaped() {
        let nel_finding = make_finding("line1\u{85}line2");
        let out_nel = render(&[nel_finding], &[]);
        assert!(
            out_nel.contains("\\u{85}"),
            "BC-2.11.009 pc1: U+0085 (NEL) must be escaped to '\\u{{85}}'; got: {out_nel}"
        );
        assert!(
            !out_nel.as_bytes().windows(2).any(|w| w == [0xc2, 0x85]),
            "BC-2.11.009 pc1: raw U+0085 UTF-8 bytes (0xC2 0x85) must not appear; got: {out_nel:?}"
        );

        let csi_finding = make_finding("before\u{9b}31mafter");
        let out_csi = render(&[csi_finding], &[]);
        assert!(
            out_csi.contains("\\u{9b}"),
            "BC-2.11.009 pc1: U+009B (CSI) must be escaped to '\\u{{9b}}'; got: {out_csi}"
        );
        assert!(
            !out_csi.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.009 pc1: raw U+009B UTF-8 bytes (0xC2 0x9B) must not appear; got: {out_csi:?}"
        );
    }

    /// AC-009 (BC-2.11.009 postcondition 2): U+00A0 (NBSP, Non-Breaking Space)
    /// is NOT escaped; it passes through as-is in the rendered output.
    #[test]
    fn test_BC_2_11_009_nbsp_u00a0_preserved() {
        let f = make_finding("word\u{a0}word");
        let out = render(&[f], &[]);
        assert!(
            out.contains("word\u{a0}word"),
            "BC-2.11.009 pc2: U+00A0 (NBSP) must not be escaped; got: {out:?}"
        );
        // Must NOT appear as the escaped form.
        assert!(
            !out.contains("\\u{a0}"),
            "BC-2.11.009 pc2: U+00A0 must not appear as '\\u{{a0}}' in output; got: {out}"
        );
    }

    /// AC-010 (BC-2.11.009 invariant 2): The C1 boundary is inclusive — U+0080
    /// escapes, U+009F escapes, U+00A0 does NOT escape. All three boundary
    /// values are verified in a single test.
    #[test]
    fn test_BC_2_11_009_c1_boundary_inclusive() {
        // U+0080 — first C1 codepoint — must be escaped.
        let f_low = make_finding("\u{80}");
        let out_low = render(&[f_low], &[]);
        assert!(
            out_low.contains("\\u{80}"),
            "BC-2.11.009 inv2: U+0080 (C1 lower boundary) must be escaped; got: {out_low}"
        );

        // U+009F — last C1 codepoint — must be escaped.
        let f_high = make_finding("\u{9f}");
        let out_high = render(&[f_high], &[]);
        assert!(
            out_high.contains("\\u{9f}"),
            "BC-2.11.009 inv2: U+009F (C1 upper boundary) must be escaped; got: {out_high}"
        );

        // U+00A0 — first codepoint past C1 — must NOT be escaped.
        let f_nbsp = make_finding("\u{a0}");
        let out_nbsp = render(&[f_nbsp], &[]);
        assert!(
            !out_nbsp.contains("\\u{a0}"),
            "BC-2.11.009 inv2: U+00A0 (past C1) must not be escaped; got: {out_nbsp}"
        );
        assert!(
            out_nbsp.as_bytes().windows(2).any(|w| w == [0xc2, 0xa0]),
            "BC-2.11.009 inv2: U+00A0 must appear as raw UTF-8 (0xC2 0xA0); got: {out_nbsp:?}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.010 — escaping applied to summary AND each evidence entry
    // -----------------------------------------------------------------------

    /// AC-011 (BC-2.11.010 postcondition 1): `TerminalReporter::render` applies
    /// escaping to `Finding.summary` — raw C0/DEL/C1 bytes in the summary do
    /// not appear in the rendered output.
    #[test]
    fn test_BC_2_11_010_summary_is_escaped() {
        let mut f = make_finding("attacker\u{1b}[31mRED");
        f.evidence = vec!["clean evidence".to_string()];
        let out = render(&[f], &[]);
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.010 pc1: raw ESC in summary must be escaped; got: {out:?}"
        );
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.010 pc1: escaped form '\\u{{1b}}' must appear from summary; got: {out}"
        );
    }

    /// AC-012 (BC-2.11.010 postcondition 2): `TerminalReporter::render` applies
    /// escaping to EACH entry in `Finding.evidence` independently — raw
    /// C0/DEL/C1 bytes in evidence do not appear in the rendered output.
    #[test]
    fn test_BC_2_11_010_evidence_each_entry_is_escaped() {
        let mut f = make_finding("clean summary");
        f.evidence = vec![
            "ev1: \x1b[32mGREEN".to_string(),
            "ev2: \u{9b}31mC1-CSI".to_string(),
        ];
        let out = render(&[f], &[]);

        // No raw ESC byte.
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.010 pc2: raw ESC in evidence[0] must be escaped; got: {out:?}"
        );
        // No raw C1 CSI bytes.
        assert!(
            !out.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.010 pc2: raw C1 CSI in evidence[1] must be escaped; got: {out:?}"
        );
        // Both escaped forms must appear.
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.010 pc2: escaped ESC from evidence[0] must appear; got: {out}"
        );
        assert!(
            out.contains("\\u{9b}"),
            "BC-2.11.010 pc2: escaped C1 CSI from evidence[1] must appear; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.011 — analyzer-summary detail value escaping
    // -----------------------------------------------------------------------

    /// AC-013 (BC-2.11.011 postcondition 1): `TerminalReporter::render` applies
    /// escaping to each value in `AnalysisSummary.detail`. A C1 CSI byte
    /// (U+009B) in a detail value is escaped to `\u{9b}` in the output.
    #[test]
    fn test_BC_2_11_011_analyzer_detail_c1_escaped() {
        let mut detail = std::collections::BTreeMap::new();
        detail.insert(
            "top_snis".to_string(),
            serde_json::json!("attacker\u{9b}31mCSI.example.com"),
        );
        let asummary = AnalysisSummary {
            analyzer_name: "TLS".to_string(),
            packets_analyzed: 1,
            detail,
        };
        let out = render(&[], &[asummary]);

        // No raw C1 CSI bytes in the analyzer summary section.
        assert!(
            !out.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.011 pc1: raw C1 CSI in detail value must be escaped; got: {out:?}"
        );
        assert!(
            out.contains("\\u{9b}"),
            "BC-2.11.011 pc1: escaped form '\\u{{9b}}' must appear from detail value; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.012 — end-to-end C1 CSI in path-traversal finding
    // -----------------------------------------------------------------------

    /// AC-014 (BC-2.11.012 postcondition 1): End-to-end: an HTTP path-traversal
    /// `Finding` whose `summary` contains U+009B produces terminal output where
    /// U+009B appears as `\u{9b}`, not as raw 0xC2 0x9B bytes.
    ///
    /// This constructs the Finding directly (without driving HttpAnalyzer) to
    /// keep the test isolated to the reporter layer per the architecture mapping
    /// in STORY-077 (reporter/terminal — pure function, no I/O).
    #[test]
    fn test_BC_2_11_012_http_finding_c1_end_to_end() {
        // Simulate the path-traversal finding summary that HttpAnalyzer would
        // produce when the URI contains U+009B CSI (0xC2 0x9B in UTF-8).
        let mut summary_with_c1 = "Path traversal detected: GET /../../etc/passwd".to_string();
        summary_with_c1.push('\u{9b}'); // U+009B CSI appended
        summary_with_c1.push_str("31mHACKED");

        let f = Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: summary_with_c1.clone(),
            evidence: vec![],
            mitre_technique: None,
            source_ip: None,
            timestamp: None,
            direction: None,
        };

        // Forensic preservation: the raw Finding struct must hold the C1 byte.
        assert!(
            f.summary.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.012: Finding.summary must preserve raw C1 CSI for forensics; got: {:?}",
            f.summary
        );

        let out = render(&[f], &[]);

        // Terminal output must not contain raw C1 CSI bytes.
        assert!(
            !out.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.012 pc1: raw C1 CSI (0xC2 0x9B) must not appear in terminal output; got: {out:?}"
        );

        // The escaped form must appear instead.
        assert!(
            out.contains("\\u{9b}"),
            "BC-2.11.012 pc1: U+009B must appear as '\\u{{9b}}' in terminal output; got: {out}"
        );
    }
}
