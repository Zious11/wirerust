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
    ///
    /// Discriminating assertions:
    ///   - Negative: "Skipped:" absent from output (core postcondition).
    ///   - Positive: "Packets:" header ALWAYS present (BC-2.11.006 inv2).
    #[test]
    fn test_BC_2_11_006_skipped_packets_zero_no_line() {
        // BC-2.11.006 pc2: conditional guard `if summary.skipped_packets > 0`
        // at terminal.rs:94 must prevent the warning line from being emitted.
        let mut summary = Summary::new();
        summary.skipped_packets = 0;
        let out = plain_reporter().render(&summary, &[], &[]);

        // Invariant 2: the Packets/Bytes/Hosts header line is always present.
        assert!(
            out.contains("Packets:"),
            "BC-2.11.006 inv2: 'Packets:' header must always be present; got: {out}"
        );

        // Postcondition 2: the skipped warning must be entirely absent.
        assert!(
            !out.contains("Skipped:"),
            "BC-2.11.006 pc2: 'Skipped:' must not appear when skipped_packets = 0; got: {out}"
        );
    }

    /// AC-002 (BC-2.11.006 postcondition 1): When `Summary.skipped_packets = 5`,
    /// the output contains `"Skipped: 5 packets (decode errors)"`.
    ///
    /// Discriminating assertions:
    ///   - Positive: exact phrase "Skipped: 5 packets (decode errors)" present (pc1).
    ///   - Positive: raw "Skipped:" substring also present (prefix check).
    #[test]
    fn test_BC_2_11_006_skipped_packets_nonzero_line_present() {
        // BC-2.11.006 pc1: canonical test vector from BC.
        let mut summary = Summary::new();
        summary.skipped_packets = 5;
        let out = plain_reporter().render(&summary, &[], &[]);

        // Exact phrase match — BC canonical vector: "  Skipped: N packets (decode errors)\n".
        assert!(
            out.contains("Skipped: 5 packets (decode errors)"),
            "BC-2.11.006 pc1: expected 'Skipped: 5 packets (decode errors)' in output; got: {out}"
        );

        // Coarse prefix check.
        assert!(
            out.contains("Skipped:"),
            "BC-2.11.006 pc1: 'Skipped:' prefix must be present; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.007 — C0 + DEL + backslash escaping
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.11.007 postcondition 1): ESC (0x1B) in a Finding summary
    /// does NOT appear as a raw byte in the rendered output; it is escaped via
    /// `char::escape_default` (i.e., appears as `\u{1b}` in the output).
    ///
    /// Discriminating assertions:
    ///   - Negative: raw 0x1B byte absent from output bytes.
    ///   - Positive: "\u{1b}" escaped form present.
    ///   - Positive: surrounding printable ASCII ("[31mRED") preserved.
    #[test]
    fn test_BC_2_11_007_esc_byte_escaped() {
        // BC-2.11.007 pc1: canonical test vector: "evil\x1b[31mtext" → "evil\u{1b}[31mtext".
        let f = make_finding("payload: \x1b[31mRED\x1b[0m");
        let out = render(&[f], &[]);

        // Raw ESC byte must not appear.
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.007 pc1: raw ESC byte (0x1b) must not appear in terminal output; got: {out:?}"
        );

        // Escaped form must appear.
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.007 pc1: escaped form '\\u{{1b}}' must appear in output; got: {out}"
        );

        // Surrounding printable ASCII must survive unchanged.
        assert!(
            out.contains("[31mRED"),
            "BC-2.11.007 pc5: printable ASCII around ESC must be preserved; got: {out}"
        );
    }

    /// AC-004 (BC-2.11.007 postcondition 2): DEL (0x7F) in a Finding summary
    /// is escaped; raw 0x7F does NOT appear in the rendered output.
    ///
    /// Discriminating assertions:
    ///   - Negative: raw 0x7F byte absent.
    ///   - Positive: "\u{7f}" escaped form present.
    ///
    /// Domain note: JsonReporter does NOT escape DEL (above C0 range per RFC 8259).
    /// TerminalReporter DOES escape DEL via `c.is_ascii_control()` (0x7F satisfies
    /// this predicate). This tests the TerminalReporter-specific behavior.
    #[test]
    fn test_BC_2_11_007_del_escaped() {
        // BC-2.11.007 pc2: DEL (0x7F) must be escaped to \u{7f}.
        let f = make_finding("before\x7fafter");
        let out = render(&[f], &[]);

        // Raw DEL byte must not appear.
        assert!(
            !out.as_bytes().contains(&0x7f),
            "BC-2.11.007 pc2: raw DEL byte (0x7f) must not appear in terminal output; \
             TerminalReporter escapes DEL unlike JsonReporter; got: {out:?}"
        );

        // Escaped form must appear.
        assert!(
            out.contains("\\u{7f}"),
            "BC-2.11.007 pc2: escaped form '\\u{{7f}}' must appear in output; got: {out}"
        );
    }

    /// AC-005 (BC-2.11.007 postcondition 4): Backslash (0x5C) in a Finding
    /// summary is converted to `\\` (double-backslash); the raw single backslash
    /// does NOT pass through.
    ///
    /// Discriminating assertions:
    ///   - Positive: double-backslash form "C:\\\\Users\\\\victim" appears.
    ///   - Implicit negative: single-backslash form between letters is gone.
    #[test]
    fn test_BC_2_11_007_backslash_escaped() {
        // BC-2.11.007 pc4: backslash → double-backslash.
        // Canonical BC test vector: "back\\slash" → "back\\\\slash".
        let f = make_finding("path: C:\\Users\\victim");
        let out = render(&[f], &[]);

        // Double-backslash form must appear — each raw backslash becomes two.
        // In Rust string literal: "C:\\\\Users\\\\victim" is C:\\Users\\victim in the
        // rendered String (four backslash chars in the source = two literal backslashes).
        assert!(
            out.contains("C:\\\\Users\\\\victim"),
            "BC-2.11.007 pc4: each backslash must be doubled; expected 'C:\\\\Users\\\\victim'; \
             got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.008 — printable ASCII and UTF-8 preservation
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.11.008 postcondition 1): Printable ASCII characters
    /// (0x20-0x7E, excluding backslash 0x5C) pass through `escape_for_terminal`
    /// unchanged and appear verbatim in the rendered output.
    ///
    /// Discriminating assertions:
    ///   - Positive: full printable ASCII string (excl. backslash) present verbatim.
    ///   - Negative: no spurious \u{20} or \u{7e} escape sequences for printable chars.
    #[test]
    fn test_BC_2_11_008_printable_ascii_preserved() {
        // BC-2.11.008 pc1: exercise the full printable ASCII range excluding backslash.
        // Canonical test vector: "hello world 123 !@#" → identical.
        let ascii: String = (0x20u8..=0x7eu8)
            .filter(|&b| b != b'\\')
            .map(|b| b as char)
            .collect();
        let f = make_finding(ascii.clone());
        let out = render(&[f], &[]);

        // The entire fixture string must appear unchanged.
        assert!(
            out.contains(&ascii),
            "BC-2.11.008 pc1: printable ASCII (excl. backslash) must pass through unchanged; \
             got: {out}"
        );

        // No spurious escape sequences for printable chars.
        assert!(
            !out.contains("\\u{20}") && !out.contains("\\u{7e}"),
            "BC-2.11.008 pc1: printable ASCII must NOT be escape-encoded; got: {out}"
        );
    }

    /// AC-007 (BC-2.11.008 postcondition 2): Cyrillic, emoji, and other
    /// non-ASCII Unicode codepoints at U+00A0 and above pass through
    /// `escape_for_terminal` unchanged.
    ///
    /// Discriminating assertions:
    ///   - Positive: Cyrillic string present as raw UTF-8.
    ///   - Positive: emoji present as raw UTF-8.
    ///   - Negative: no \u{43f} Debug-format escape (construction-site regression guard).
    ///   - Negative: no \u{1f980} escape for the emoji codepoint.
    #[test]
    fn test_BC_2_11_008_cyrillic_and_emoji_preserved() {
        // BC-2.11.008 pc2 canonical test vectors:
        //   "пример.рф" (Cyrillic) → identical
        //   "crab 🦀 rust" (emoji) → identical
        let f = make_finding("host: пример.рф 🦀");
        let out = render(&[f], &[]);

        // Cyrillic must be present as readable UTF-8.
        assert!(
            out.contains("пример.рф"),
            "BC-2.11.008 pc2: Cyrillic 'пример.рф' must pass through unchanged; got: {out}"
        );

        // Emoji must be present as raw UTF-8.
        assert!(
            out.contains("🦀"),
            "BC-2.11.008 pc2: emoji '🦀' must pass through unchanged; got: {out}"
        );

        // Guard against construction-site Debug-format escaping regression.
        assert!(
            !out.contains("\\u{43f}"),
            "BC-2.11.008 pc2: Cyrillic 'п' must not appear as '\\u{{43f}}' escape; got: {out}"
        );

        // Guard against emoji being escape-encoded.
        assert!(
            !out.contains("\\u{1f980}"),
            "BC-2.11.008 pc2: crab emoji must not appear as '\\u{{1f980}}' escape; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.009 — C1 range escaping and NBSP preservation
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.11.009 postcondition 1): All codepoints in U+0080-U+009F
    /// (C1 range, inclusive) are replaced by `char::escape_default` output.
    /// Exercises U+0085 (NEL → `\u{85}`) and U+009B (CSI → `\u{9b}`).
    ///
    /// CRITICAL domain asymmetry: JsonReporter PASSES C1 through as raw UTF-8
    /// (BC-2.11.005 pc1). TerminalReporter ESCAPES C1 (BC-2.11.007/009). These
    /// assertions are the OPPOSITE of reporter_json_tests.rs test_BC_2_11_005_*.
    ///
    /// Discriminating assertions:
    ///   - Positive: "\u{85}" escaped form for NEL.
    ///   - Negative: raw 0xC2 0x85 byte pair absent (NEL as UTF-8).
    ///   - Positive: "\u{9b}" escaped form for CSI.
    ///   - Negative: raw 0xC2 0x9B byte pair absent (CSI as UTF-8).
    #[test]
    fn test_BC_2_11_009_c1_range_escaped() {
        // BC-2.11.009 pc1: canonical test vectors:
        //   "line1\u{85}line2" → "line1\u{85}line2"  (escape notation in output)
        //   "before\u{9b}31mafter" → "before\u{9b}31mafter"  (escape notation in output)

        // --- NEL (U+0085) ---
        let nel_finding = make_finding("line1\u{85}line2");
        let out_nel = render(&[nel_finding], &[]);

        assert!(
            out_nel.contains("\\u{85}"),
            "BC-2.11.009 pc1: NEL (U+0085) must be escaped to '\\u{{85}}'; got: {out_nel}"
        );
        assert!(
            !out_nel.as_bytes().windows(2).any(|w| w == [0xc2, 0x85]),
            "BC-2.11.009 pc1: raw U+0085 UTF-8 bytes (0xC2 0x85) must not appear; got: {out_nel:?}"
        );

        // --- CSI (U+009B) ---
        let csi_finding = make_finding("before\u{9b}31mafter");
        let out_csi = render(&[csi_finding], &[]);

        assert!(
            out_csi.contains("\\u{9b}"),
            "BC-2.11.009 pc1: CSI (U+009B) must be escaped to '\\u{{9b}}'; got: {out_csi}"
        );
        assert!(
            !out_csi.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.009 pc1: raw U+009B UTF-8 bytes (0xC2 0x9B) must not appear; got: {out_csi:?}"
        );
    }

    /// AC-009 (BC-2.11.009 postcondition 2): U+00A0 (NBSP, Non-Breaking Space)
    /// is NOT escaped; it passes through as-is in the rendered output.
    ///
    /// Discriminating assertions:
    ///   - Positive: raw U+00A0 byte pair (0xC2 0xA0) present (not escaped).
    ///   - Negative: "\u{a0}" escaped form NOT present.
    ///   - Positive: "word\u{a0}word" sequence present as-is.
    #[test]
    fn test_BC_2_11_009_nbsp_u00a0_preserved() {
        // BC-2.11.009 pc2: canonical test vector: "\u{a0}" → "\u{a0}" (raw bytes).
        // U+00A0 is ONE past the C1 range (U+009F is the last C1 codepoint).
        let f = make_finding("word\u{a0}word");
        let out = render(&[f], &[]);

        // The full sequence with raw NBSP must be present.
        assert!(
            out.contains("word\u{a0}word"),
            "BC-2.11.009 pc2: U+00A0 (NBSP) must pass through as raw UTF-8; got: {out:?}"
        );

        // The escaped form must NOT appear.
        assert!(
            !out.contains("\\u{a0}"),
            "BC-2.11.009 pc2: U+00A0 must NOT appear as '\\u{{a0}}'; it is above C1 range; \
             got: {out}"
        );
    }

    /// AC-010 (BC-2.11.009 invariant 2): The C1 boundary is inclusive — U+0080
    /// escapes, U+009F escapes, U+00A0 does NOT escape. All three boundary
    /// values are verified.
    ///
    /// Discriminating assertions (three boundary values):
    ///   - U+0080 (first C1): "\u{80}" present; raw 0xC2 0x80 absent.
    ///   - U+009F (last C1): "\u{9f}" present; raw 0xC2 0x9F absent.
    ///   - U+00A0 (first past C1): raw 0xC2 0xA0 present; "\u{a0}" absent.
    #[test]
    fn test_BC_2_11_009_c1_boundary_inclusive() {
        // BC-2.11.009 inv2: inclusive boundary test.
        // Canonical test vectors:
        //   "\u{80}" → "\\u{80}"  (escaped)
        //   "\u{9f}" → "\\u{9f}"  (escaped)
        //   "\u{a0}" → "\u{a0}"   (raw bytes, not escaped)

        // --- U+0080: first C1 codepoint — must escape ---
        let f_low = make_finding("\u{80}");
        let out_low = render(&[f_low], &[]);
        assert!(
            out_low.contains("\\u{80}"),
            "BC-2.11.009 inv2: U+0080 (C1 lower boundary) must be escaped to '\\u{{80}}'; \
             got: {out_low}"
        );
        assert!(
            !out_low.as_bytes().windows(2).any(|w| w == [0xc2, 0x80]),
            "BC-2.11.009 inv2: raw U+0080 bytes (0xC2 0x80) must not appear; got: {out_low:?}"
        );

        // --- U+009F: last C1 codepoint — must escape ---
        let f_high = make_finding("\u{9f}");
        let out_high = render(&[f_high], &[]);
        assert!(
            out_high.contains("\\u{9f}"),
            "BC-2.11.009 inv2: U+009F (C1 upper boundary) must be escaped to '\\u{{9f}}'; \
             got: {out_high}"
        );
        assert!(
            !out_high.as_bytes().windows(2).any(|w| w == [0xc2, 0x9f]),
            "BC-2.11.009 inv2: raw U+009F bytes (0xC2 0x9F) must not appear; got: {out_high:?}"
        );

        // --- U+00A0: first codepoint past C1 — must NOT escape ---
        let f_nbsp = make_finding("\u{a0}");
        let out_nbsp = render(&[f_nbsp], &[]);
        assert!(
            !out_nbsp.contains("\\u{a0}"),
            "BC-2.11.009 inv2: U+00A0 (past C1) must NOT be escaped; got: {out_nbsp}"
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
    ///
    /// Discriminating assertions:
    ///   - Negative: raw ESC byte absent from output.
    ///   - Positive: escaped form "\u{1b}" present in output.
    ///   - Positive: clean evidence passes through (isolates summary-only escape).
    #[test]
    fn test_BC_2_11_010_summary_is_escaped() {
        // BC-2.11.010 pc1: summary with ESC; evidence is clean.
        // Canonical test vector: summary="\x1b ESC", evidence=["clean"].
        let mut f = make_finding("attacker\u{1b}[31mRED");
        f.evidence = vec!["clean evidence".to_string()];
        let out = render(&[f], &[]);

        // Raw ESC must not appear (summary was escaped).
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.010 pc1: raw ESC in summary must be escaped; got: {out:?}"
        );

        // Escaped form must appear.
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.010 pc1: escaped form '\\u{{1b}}' must appear from summary; got: {out}"
        );

        // Clean evidence must be preserved unchanged.
        assert!(
            out.contains("clean evidence"),
            "BC-2.11.010 pc1: clean evidence must be preserved; got: {out}"
        );
    }

    /// AC-012 (BC-2.11.010 postcondition 2): `TerminalReporter::render` applies
    /// escaping to EACH entry in `Finding.evidence` independently — raw
    /// C0/DEL/C1 bytes in evidence do not appear in the rendered output.
    ///
    /// Discriminating assertions:
    ///   - Two evidence entries with different control bytes (ESC + CSI).
    ///   - Negative: raw ESC absent; raw CSI byte pair absent.
    ///   - Positive: escaped form for each entry independently.
    #[test]
    fn test_BC_2_11_010_evidence_each_entry_is_escaped() {
        // BC-2.11.010 pc2: EC-011 scenario — control bytes in both entries.
        // Canonical test vector: evidence=["\x1b", "\u{9b}"] → both escaped.
        let mut f = make_finding("clean summary");
        f.evidence = vec![
            "ev1: \x1b[32mGREEN".to_string(),
            "ev2: \u{9b}31mC1-CSI".to_string(),
        ];
        let out = render(&[f], &[]);

        // evidence[0]: ESC must be escaped.
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.010 pc2: raw ESC in evidence[0] must be escaped; got: {out:?}"
        );

        // evidence[1]: C1 CSI must be escaped.
        assert!(
            !out.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.010 pc2: raw C1 CSI bytes in evidence[1] must be escaped; got: {out:?}"
        );

        // Both escaped forms must appear in the output.
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
    /// escaping to each value in `AnalysisSummary.detail` (via
    /// `escape_for_terminal(&val.to_string())`). A C1 CSI byte (U+009B) in a
    /// detail value is escaped to `\u{9b}` in the output.
    ///
    /// Discriminating assertions:
    ///   - Negative: raw C1 CSI bytes absent from analyzer section.
    ///   - Positive: "\u{9b}" escaped form present.
    ///   - Positive: key name "top_snis" unchanged (keys are NOT escaped per pc3).
    ///
    /// Rationale (BC-2.11.011 inv2): serde_json's Display impl escapes C0 per
    /// RFC 8259 but passes C1 codepoints through as raw UTF-8. The terminal
    /// reporter must re-escape to close the C1 gap.
    #[test]
    fn test_BC_2_11_011_analyzer_detail_c1_escaped() {
        // BC-2.11.011 pc1: canonical test vector:
        //   detail["top_snis"] = Value::String("\u{9b}31m") → "top_snis: \u{9b}31m".
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

        // Raw C1 CSI bytes must not appear.
        assert!(
            !out.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.011 pc1: raw C1 CSI in detail value must be escaped; got: {out:?}"
        );

        // Escaped form must appear.
        assert!(
            out.contains("\\u{9b}"),
            "BC-2.11.011 pc1: escaped form '\\u{{9b}}' must appear from detail value; got: {out}"
        );

        // Key name must not be escaped (pc3: keys are program-controlled).
        assert!(
            out.contains("top_snis"),
            "BC-2.11.011 pc3: key name 'top_snis' must appear unchanged; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.012 — end-to-end C1 CSI in path-traversal finding
    // -----------------------------------------------------------------------

    /// AC-014 (BC-2.11.012 postcondition 1): End-to-end — an HTTP path-traversal
    /// `Finding` whose `summary` contains U+009B produces terminal output where
    /// U+009B appears as `\u{9b}`, not as raw 0xC2 0x9B bytes.
    ///
    /// Constructs the Finding directly (not via HttpAnalyzer) to isolate the
    /// reporter layer per STORY-077 architecture mapping (pure function, no I/O).
    ///
    /// Discriminating assertions:
    ///   - Positive (INV-4): raw C1 bytes in Finding.summary before rendering.
    ///   - Negative: raw 0xC2 0x9B absent from terminal output (pc2).
    ///   - Positive: "\u{9b}" escaped form present in terminal output (pc1).
    #[test]
    fn test_BC_2_11_012_http_finding_c1_end_to_end() {
        // BC-2.11.012 pc1: canonical test vector:
        //   HTTP path-traversal Finding with summary="URI: /\u{9b}31m../etc/passwd"
        //   → output contains "URI: /\u{9b}31m../etc/passwd" (escape notation).
        //
        // BC-2.11.012 inv3: confirms no early escaping happened in the analyzer
        // or Finding constructor — raw C1 must still be in the struct.
        let mut summary_with_c1 = "Path traversal detected: GET /../../etc/passwd".to_string();
        summary_with_c1.push('\u{9b}'); // U+009B CSI
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

        // INV-4 / BC-2.11.012 inv3: forensic preservation — raw C1 in the struct.
        assert!(
            f.summary.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.012 inv3 / INV-4: Finding.summary must preserve raw C1 CSI for forensics; \
             got: {:?}",
            f.summary
        );

        let out = render(&[f], &[]);

        // Terminal output must not contain raw C1 CSI bytes (pc2).
        assert!(
            !out.as_bytes().windows(2).any(|w| w == [0xc2, 0x9b]),
            "BC-2.11.012 pc2: raw C1 CSI (0xC2 0x9B) must not appear in terminal output; \
             got: {out:?}"
        );

        // Escaped form must appear (pc1).
        assert!(
            out.contains("\\u{9b}"),
            "BC-2.11.012 pc1: U+009B must appear as '\\u{{9b}}' in terminal output; got: {out}"
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-078: TerminalReporter — MITRE Grouping, Section Order, Colorization
// Per DF-TEST-NAMESPACE-001: all STORY-078 tests grouped in `mod story_078`.
// Covers BC-2.11.013 through BC-2.11.019, AC-001..AC-016.
// ---------------------------------------------------------------------------
mod story_078 {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers scoped to story_078
    // -----------------------------------------------------------------------

    /// TerminalReporter with MITRE grouping enabled and color disabled.
    fn mitre_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_mitre_grouping: true,
            show_hosts_breakdown: false,
        }
    }

    /// Construct a Finding with the given verdict, confidence, and optional MITRE technique.
    fn make_mitre_finding(
        summary: impl Into<String>,
        verdict: Verdict,
        confidence: Confidence,
        technique: Option<&str>,
    ) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict,
            confidence,
            summary: summary.into(),
            evidence: vec![],
            mitre_technique: technique.map(|s| s.to_string()),
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // -----------------------------------------------------------------------
    // BC-2.11.013 — MITRE Grouping Emits Tactic Headers in Canonical Order;
    //               Uncategorized Last
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.013 postcondition 2):
    /// When `show_mitre_grouping = true`, tactic section headers appear in the
    /// order returned by `all_tactics_in_report_order()`.  Only sections with at
    /// least one finding are emitted.
    ///
    /// Discriminating assertions:
    ///   - Positive: "## Defense Evasion" and "## Command and Control" both present
    ///     (two distinct tactics exercised).
    ///   - Positive: "Defense Evasion" appears BEFORE "Command and Control" in the
    ///     output (kill-chain order: DefenseEvasion index 6 < CommandAndControl
    ///     index 11 in all_tactics_in_report_order()).
    ///   - Negative: a tactic not represented by any finding is absent.
    ///
    /// pc2: sections appear in all_tactics_in_report_order() order.
    /// inv3: sections skipped when empty.
    #[test]
    fn test_BC_2_11_013_tactic_headers_in_canonical_order() {
        // T1036 → DefenseEvasion; T1071 → CommandAndControl
        let defense_evasion_finding =
            make_mitre_finding("masq", Verdict::Likely, Confidence::High, Some("T1036"));
        let c2_finding = make_mitre_finding("c2", Verdict::Likely, Confidence::High, Some("T1071"));
        let out =
            mitre_reporter().render(&Summary::new(), &[defense_evasion_finding, c2_finding], &[]);

        // Both section headers must appear.
        assert!(
            out.contains("## Defense Evasion"),
            "BC-2.11.013 pc2: '## Defense Evasion' section header must appear; got:\n{out}"
        );
        assert!(
            out.contains("## Command and Control"),
            "BC-2.11.013 pc2: '## Command and Control' section header must appear; got:\n{out}"
        );

        // Kill-chain order: DefenseEvasion (index 6) before CommandAndControl (index 11).
        let pos_de = out
            .find("## Defense Evasion")
            .expect("Defense Evasion not found");
        let pos_c2 = out
            .find("## Command and Control")
            .expect("Command and Control not found");
        assert!(
            pos_de < pos_c2,
            "BC-2.11.013 pc2: 'Defense Evasion' (kill-chain index 6) must appear before \
             'Command and Control' (kill-chain index 11); pos_de={pos_de}, pos_c2={pos_c2}"
        );

        // inv3: a tactic with no findings must not appear.
        assert!(
            !out.contains("## Reconnaissance"),
            "BC-2.11.013 inv3: '## Reconnaissance' must be absent (no findings in that tactic); \
             got:\n{out}"
        );
    }

    /// AC-002 (BC-2.11.013 postcondition 4):
    /// The `## Uncategorized` section is always the LAST section in the grouped output.
    ///
    /// Discriminating assertions:
    ///   - Positive: "## Uncategorized" present.
    ///   - Positive: "## Defense Evasion" present (named tactic).
    ///   - Positive: "## Defense Evasion" appears BEFORE "## Uncategorized" (last).
    ///
    /// pc4: Uncategorized is the last bucket.
    #[test]
    fn test_BC_2_11_013_uncategorized_last() {
        // T1036 → DefenseEvasion; None → Uncategorized.
        let known = make_mitre_finding("known", Verdict::Likely, Confidence::High, Some("T1036"));
        let uncategorized = make_mitre_finding("unknown", Verdict::Likely, Confidence::High, None);
        let out = mitre_reporter().render(&Summary::new(), &[known, uncategorized], &[]);

        // Both sections must appear.
        assert!(
            out.contains("## Defense Evasion"),
            "BC-2.11.013 pc2: '## Defense Evasion' must appear; got:\n{out}"
        );
        assert!(
            out.contains("## Uncategorized"),
            "BC-2.11.013 pc4: '## Uncategorized' must appear; got:\n{out}"
        );

        // Uncategorized must appear AFTER all named tactic sections.
        let pos_named = out
            .find("## Defense Evasion")
            .expect("Defense Evasion not found");
        let pos_uncat = out
            .find("## Uncategorized")
            .expect("Uncategorized not found");
        assert!(
            pos_named < pos_uncat,
            "BC-2.11.013 pc4: '## Uncategorized' must be the last section; \
             pos_named={pos_named}, pos_uncat={pos_uncat}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.014 — Within Tactic Bucket: Sort by Verdict, Confidence,
    //               Emission Order
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.11.014 postcondition 1):
    /// Within a MITRE tactic bucket, findings with lower verdict rank (Likely=0)
    /// appear before those with higher rank (Inconclusive=1, Unlikely=2).
    ///
    /// Discriminating assertions:
    ///   - Both findings in same tactic (DefenseEvasion via T1036).
    ///   - "LIKELY" appears BEFORE "INCONCLUSIVE" in the output.
    ///
    /// pc1: verdict rank ascending (Likely first).
    #[test]
    fn test_BC_2_11_014_sort_by_verdict_within_bucket() {
        // Emit Inconclusive first, Likely second — sorted output must flip them.
        let inconclusive = make_mitre_finding(
            "inconclusive-finding",
            Verdict::Inconclusive,
            Confidence::High,
            Some("T1036"),
        );
        let likely = make_mitre_finding(
            "likely-finding",
            Verdict::Likely,
            Confidence::Low,
            Some("T1036"),
        );
        let out = mitre_reporter().render(&Summary::new(), &[inconclusive, likely], &[]);

        // Canonical test vector: Likely/Low at i=1 renders before Inconclusive/High at i=0.
        let pos_likely = out
            .find("likely-finding")
            .expect("likely-finding not found");
        let pos_inconclusive = out
            .find("inconclusive-finding")
            .expect("inconclusive-finding not found");
        assert!(
            pos_likely < pos_inconclusive,
            "BC-2.11.014 pc1: Likely (rank 0) must render before Inconclusive (rank 1); \
             pos_likely={pos_likely}, pos_inconclusive={pos_inconclusive}"
        );
    }

    /// AC-004 (BC-2.11.014 postcondition 2):
    /// Among findings with the same verdict, findings with lower confidence rank
    /// (High=0) appear before those with higher rank (Medium=1, Low=2).
    ///
    /// Discriminating assertions:
    ///   - Both Likely; Low-confidence emitted first; High-confidence must render first.
    ///
    /// pc2: confidence rank ascending (High first) within same verdict.
    #[test]
    fn test_BC_2_11_014_sort_by_confidence_within_same_verdict() {
        // Emit Likely/Low first, Likely/High second — sort must flip them.
        let low_conf = make_mitre_finding(
            "low-confidence",
            Verdict::Likely,
            Confidence::Low,
            Some("T1036"),
        );
        let high_conf = make_mitre_finding(
            "high-confidence",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let out = mitre_reporter().render(&Summary::new(), &[low_conf, high_conf], &[]);

        // Canonical test vector: Likely/High at i=1 renders before Likely/Low at i=0.
        let pos_high = out
            .find("high-confidence")
            .expect("high-confidence not found");
        let pos_low = out
            .find("low-confidence")
            .expect("low-confidence not found");
        assert!(
            pos_high < pos_low,
            "BC-2.11.014 pc2: Likely/High (confidence rank 0) must render before \
             Likely/Low (confidence rank 2); pos_high={pos_high}, pos_low={pos_low}"
        );
    }

    /// AC-005 (BC-2.11.014 postcondition 3):
    /// Among findings with the same verdict and confidence, original emission order
    /// (slice index) is preserved (stable sort).
    ///
    /// Discriminating assertions:
    ///   - Three findings with identical verdict/confidence.
    ///   - Rendered order must match emission order: alpha, beta, gamma.
    ///
    /// pc3: stable tertiary sort by original emission index.
    /// inv3: sort_by_key (Rust std) is stable.
    #[test]
    fn test_BC_2_11_014_stable_emission_order_on_tie() {
        // Canonical test vector: [Likely/High at i=0, Likely/High at i=1] → i=0 first.
        let alpha = make_mitre_finding(
            "alpha-first",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let beta = make_mitre_finding(
            "beta-second",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let gamma = make_mitre_finding(
            "gamma-third",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let out = mitre_reporter().render(&Summary::new(), &[alpha, beta, gamma], &[]);

        let pos_alpha = out.find("alpha-first").expect("alpha-first not found");
        let pos_beta = out.find("beta-second").expect("beta-second not found");
        let pos_gamma = out.find("gamma-third").expect("gamma-third not found");
        assert!(
            pos_alpha < pos_beta && pos_beta < pos_gamma,
            "BC-2.11.014 pc3/inv3: emission order must be stable on tie; \
             expected alpha < beta < gamma; got pos_alpha={pos_alpha}, \
             pos_beta={pos_beta}, pos_gamma={pos_gamma}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.015 — No-Technique or Unknown-ID Findings Land in Uncategorized
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.11.015 postcondition 1):
    /// Findings with `mitre_technique = None` appear under `## Uncategorized`.
    ///
    /// Discriminating assertions:
    ///   - Positive: "## Uncategorized" present.
    ///   - Positive: "none-technique-finding" present under Uncategorized.
    ///   - Negative: No MITRE line for a None-technique finding (pc4).
    ///
    /// pc1: None technique → Uncategorized bucket.
    /// pc4: No MITRE line for None technique.
    #[test]
    fn test_BC_2_11_015_none_technique_uncategorized() {
        // EC-001: mitre_technique = None.
        let f = make_mitre_finding(
            "none-technique-finding",
            Verdict::Likely,
            Confidence::High,
            None,
        );
        let out = mitre_reporter().render(&Summary::new(), &[f], &[]);

        // Uncategorized must be the only tactic section.
        assert!(
            out.contains("## Uncategorized"),
            "BC-2.11.015 pc1: '## Uncategorized' must appear for None-technique finding; \
             got:\n{out}"
        );

        // The finding summary must appear in the output.
        assert!(
            out.contains("none-technique-finding"),
            "BC-2.11.015 pc1: finding summary must appear under Uncategorized; got:\n{out}"
        );

        // pc4: No MITRE line for None technique (render_finding_grouped skips MITRE line
        // when mitre_technique is None — the `if let Some(ref id)` guard at terminal.rs:239).
        assert!(
            !out.contains("MITRE:"),
            "BC-2.11.015 pc4: no MITRE line must appear for None-technique finding; got:\n{out}"
        );
    }

    /// AC-007 (BC-2.11.015 postcondition 2, 3):
    /// Findings with an unrecognized technique ID appear under `## Uncategorized`
    /// with the MITRE line reading `MITRE: T9999 (unknown)`.
    ///
    /// Discriminating assertions:
    ///   - Positive: "## Uncategorized" present.
    ///   - Positive: "MITRE: T9999 (unknown)" present (pc3).
    ///   - Negative: No named tactic section for T9999.
    ///
    /// pc2: unknown ID → Uncategorized.
    /// pc3: MITRE line reads "<id> (unknown)" for unrecognized IDs.
    #[test]
    fn test_BC_2_11_015_unknown_id_uncategorized_with_label() {
        // EC-002: T9999 is not in the catalog.
        let f = make_mitre_finding(
            "unknown-id-finding",
            Verdict::Likely,
            Confidence::High,
            Some("T9999"),
        );
        let out = mitre_reporter().render(&Summary::new(), &[f], &[]);

        // Uncategorized must appear.
        assert!(
            out.contains("## Uncategorized"),
            "BC-2.11.015 pc2: '## Uncategorized' must appear for unknown-ID finding; got:\n{out}"
        );

        // MITRE line must carry the (unknown) label.
        assert!(
            out.contains("MITRE: T9999 (unknown)"),
            "BC-2.11.015 pc3: MITRE line must read 'MITRE: T9999 (unknown)'; got:\n{out}"
        );

        // No spurious named tactic section.
        assert!(
            !out.contains("## Reconnaissance"),
            "BC-2.11.015 pc2: no named tactic section must appear for unknown ID; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.016 — MITRE Grouping Expands Per-Finding Line with Em-Dash
    //               and Name
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.11.016 postcondition 1):
    /// When `show_mitre_grouping = true` and a finding has a known technique ID
    /// (e.g., "T1036"), the MITRE line reads `MITRE: T1036 \u{2014} Masquerading`
    /// (U+2014 em-dash, not ASCII "--").
    ///
    /// Discriminating assertions:
    ///   - Positive: "MITRE: T1036 \u{2014} Masquerading" (exact em-dash + name).
    ///   - Positive: raw em-dash bytes (0xE2 0x80 0x94) present in output bytes.
    ///
    /// pc1: MITRE line = "MITRE: <id> \u{2014} <name>".
    /// inv1: em-dash literal at terminal.rs:241.
    #[test]
    fn test_BC_2_11_016_known_id_em_dash_and_name() {
        // EC-001: T1036 → "Masquerading" per technique_name catalog.
        let f = make_mitre_finding(
            "masq-finding",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let out = mitre_reporter().render(&Summary::new(), &[f], &[]);

        // Exact MITRE line format with U+2014 em-dash (— is the literal em-dash char).
        assert!(
            out.contains("MITRE: T1036 \u{2014} Masquerading"),
            "BC-2.11.016 pc1: MITRE line must read 'MITRE: T1036 — Masquerading' (em-dash); \
             got:\n{out}"
        );

        // Byte-level assertion: U+2014 is encoded as 0xE2 0x80 0x94 in UTF-8.
        let em_dash_bytes: &[u8] = &[0xe2, 0x80, 0x94];
        assert!(
            out.as_bytes().windows(3).any(|w| w == em_dash_bytes),
            "BC-2.11.016 inv1: em-dash bytes [0xE2, 0x80, 0x94] must be present in output bytes"
        );
    }

    /// AC-009 (BC-2.11.016 invariant 1):
    /// The separator is U+2014 (EM DASH). ASCII `--` (two hyphens) is NOT used
    /// as the separator on the MITRE line.
    ///
    /// Discriminating assertions:
    ///   - Positive: U+2014 em-dash present in the MITRE line.
    ///   - Negative: ASCII "--" does NOT appear immediately adjacent to the
    ///     technique name (i.e., "T1036 -- Masquerading" must NOT be present).
    ///
    /// inv1: em-dash U+2014 is the separator, not ASCII "--".
    #[test]
    fn test_BC_2_11_016_separator_is_em_dash_not_ascii_hyphen() {
        let f = make_mitre_finding(
            "sep-check",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let out = mitre_reporter().render(&Summary::new(), &[f], &[]);

        // Em-dash must be present.
        assert!(
            out.contains('\u{2014}'),
            "BC-2.11.016 inv1: U+2014 em-dash must be present as the separator; got:\n{out}"
        );

        // ASCII double-hyphen must NOT be used as the separator.
        // "T1036 -- Masquerading" would indicate a regression to ASCII hyphens.
        assert!(
            !out.contains("T1036 -- Masquerading"),
            "BC-2.11.016 inv1: ASCII '--' must NOT be the separator; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.017 — Default Rendering Emits MITRE: <id> Only (No Em-Dash)
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.11.017 postcondition 1):
    /// When `show_mitre_grouping = false` (default), a finding with
    /// `mitre_technique = "T1036"` produces the MITRE line `MITRE: T1036`
    /// with no em-dash, no technique name, and no `(unknown)` label.
    ///
    /// Discriminating assertions:
    ///   - Positive: "MITRE: T1036" present.
    ///   - Negative: U+2014 em-dash absent.
    ///   - Negative: "Masquerading" (technique name) absent.
    ///   - Negative: "(unknown)" absent.
    ///
    /// pc1: flat mode → "MITRE: <id>" only.
    /// inv1/2: render_finding_flat never calls technique_name() / technique_tactic().
    #[test]
    fn test_BC_2_11_017_default_mode_bare_mitre_id() {
        // Canonical test vector: mitre="T1036", show_mitre_grouping=false.
        let f = make_mitre_finding(
            "flat-finding",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        // Use plain_reporter() (show_mitre_grouping = false).
        let out = plain_reporter().render(&Summary::new(), &[f], &[]);

        // Bare MITRE ID must be present.
        assert!(
            out.contains("MITRE: T1036"),
            "BC-2.11.017 pc1: 'MITRE: T1036' must appear in flat mode; got:\n{out}"
        );

        // Em-dash must be absent (no grouping expansion in flat mode).
        assert!(
            !out.contains('\u{2014}'),
            "BC-2.11.017 pc1: U+2014 em-dash must NOT appear in flat mode; got:\n{out}"
        );

        // Technique name "Masquerading" must be absent.
        assert!(
            !out.contains("Masquerading"),
            "BC-2.11.017 pc1: technique name must NOT appear in flat mode; got:\n{out}"
        );

        // "(unknown)" label must be absent even for unknown IDs in flat mode.
        assert!(
            !out.contains("(unknown)"),
            "BC-2.11.017 pc1: '(unknown)' label must NOT appear in flat mode; got:\n{out}"
        );
    }

    /// AC-011 (BC-2.11.017 postcondition 3):
    /// In default mode, no `## TacticName` or `## Uncategorized` section headers
    /// appear in the output.
    ///
    /// Discriminating assertions:
    ///   - Negative: "## Defense Evasion" absent.
    ///   - Negative: "## Uncategorized" absent.
    ///   - Positive: finding summary present (rendered flat, not dropped).
    ///
    /// pc3: no tactic headers in flat mode.
    #[test]
    fn test_BC_2_11_017_default_mode_no_tactic_headers() {
        let f = make_mitre_finding(
            "flat-no-header",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        let out = plain_reporter().render(&Summary::new(), &[f], &[]);

        // No tactic headers in flat mode.
        assert!(
            !out.contains("## Defense Evasion"),
            "BC-2.11.017 pc3: '## Defense Evasion' must NOT appear in flat mode; got:\n{out}"
        );
        assert!(
            !out.contains("## Uncategorized"),
            "BC-2.11.017 pc3: '## Uncategorized' must NOT appear in flat mode; got:\n{out}"
        );

        // Finding must still be rendered.
        assert!(
            out.contains("flat-no-header"),
            "BC-2.11.017: finding summary must still appear in flat mode; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.018 — TerminalReporter Colorization: use_color = false
    // -----------------------------------------------------------------------

    /// AC-012 (BC-2.11.018 postcondition 5):
    /// When `use_color = false`, no ANSI escape codes appear in the rendered
    /// output for any verdict/confidence combination.
    ///
    /// Discriminating assertions (four verdict/confidence combos tested):
    ///   - Likely/High (would be red bold if colored).
    ///   - Likely/Medium (would be yellow).
    ///   - Inconclusive/High (would be cyan).
    ///   - Unlikely/Low (would be dimmed).
    ///   - Negative: "\x1b[" (ANSI CSI introducer) absent from all outputs.
    ///
    /// pc5: plain text only when use_color = false; no ANSI codes.
    #[test]
    fn test_BC_2_11_018_no_ansi_codes_when_color_disabled() {
        let combos = [
            (Verdict::Likely, Confidence::High, "likely-high"),
            (Verdict::Likely, Confidence::Medium, "likely-medium"),
            (Verdict::Inconclusive, Confidence::High, "inconclusive-high"),
            (Verdict::Unlikely, Confidence::Low, "unlikely-low"),
        ];

        for (verdict, confidence, label) in combos {
            let f = make_mitre_finding(label, verdict, confidence, Some("T1036"));
            // Use plain_reporter (use_color = false).
            let out = plain_reporter().render(&Summary::new(), &[f], &[]);

            // ANSI CSI introducer "\x1b[" must be entirely absent.
            assert!(
                !out.contains("\x1b["),
                "BC-2.11.018 pc5: ANSI escape '\\x1b[' must not appear when use_color=false; \
                 label={label}; got:\n{out:?}"
            );

            // Finding summary must appear (sanity check that rendering happened).
            assert!(
                out.contains(label),
                "BC-2.11.018: finding summary '{label}' must appear in output; got:\n{out}"
            );
        }
    }

    // -----------------------------------------------------------------------
    // BC-2.11.019 — TerminalReporter Renders Sections in Correct Order
    // -----------------------------------------------------------------------

    /// AC-013 (BC-2.11.019 postcondition 1):
    /// The `WIRERUST TRIAGE REPORT` header section is always the first section
    /// in the output.
    ///
    /// Discriminating assertions:
    ///   - Positive: "WIRERUST TRIAGE REPORT" present.
    ///   - Positive: header appears at the very start of the output (byte offset 0
    ///     or as the first non-empty line).
    ///
    /// pc1: header always first.
    #[test]
    fn test_BC_2_11_019_header_is_first_section() {
        let out = plain_reporter().render(&Summary::new(), &[], &[]);

        assert!(
            out.contains("WIRERUST TRIAGE REPORT"),
            "BC-2.11.019 pc1: 'WIRERUST TRIAGE REPORT' must appear in output; got:\n{out}"
        );

        // Header must be the first text in the output — byte offset must be 0.
        assert!(
            out.starts_with("WIRERUST TRIAGE REPORT"),
            "BC-2.11.019 pc1: 'WIRERUST TRIAGE REPORT' must be the first content in output; \
             got:\n{out}"
        );

        // PROTOCOLS (always present) must come AFTER the header.
        let pos_header = out.find("WIRERUST TRIAGE REPORT").unwrap();
        let pos_protocols = out.find("PROTOCOLS").expect("PROTOCOLS not found");
        assert!(
            pos_header < pos_protocols,
            "BC-2.11.019 pc1: header must appear before PROTOCOLS; \
             pos_header={pos_header}, pos_protocols={pos_protocols}"
        );
    }

    /// AC-014 (BC-2.11.019 postcondition 4 / invariant 2):
    /// The FINDINGS section appears only when `findings` is non-empty; it is
    /// entirely absent when `findings.is_empty()`.
    ///
    /// Discriminating assertions:
    ///   - When findings empty: "FINDINGS" absent.
    ///   - When findings non-empty: "FINDINGS" present.
    ///
    /// pc4: FINDINGS section absent when findings.is_empty().
    /// inv2: section is entirely absent (not just empty).
    #[test]
    fn test_BC_2_11_019_findings_section_absent_when_empty() {
        // Empty findings → FINDINGS section absent.
        let out_empty = plain_reporter().render(&Summary::new(), &[], &[]);
        assert!(
            !out_empty.contains("FINDINGS"),
            "BC-2.11.019 pc4/inv2: 'FINDINGS' section must be absent when findings empty; \
             got:\n{out_empty}"
        );

        // Non-empty findings → FINDINGS section present.
        let f = make_mitre_finding("a-finding", Verdict::Likely, Confidence::High, None);
        let out_nonempty = plain_reporter().render(&Summary::new(), &[f], &[]);
        assert!(
            out_nonempty.contains("FINDINGS"),
            "BC-2.11.019 pc4: 'FINDINGS' must appear when findings non-empty; got:\n{out_nonempty}"
        );
    }

    /// AC-016 (BC-2.11.019 invariant 3):
    /// SERVICES section is absent entirely when `service_counts()` returns an
    /// empty map.
    ///
    /// Discriminating assertions:
    ///   - Summary with no ingested packets → empty service map → "SERVICES" absent.
    ///   - Guard: "PROTOCOLS" still present (always-on section, BC-2.11.019 inv5).
    ///
    /// inv3: SERVICES absent when service_counts() is empty.
    #[test]
    fn test_BC_2_11_019_services_section_absent_when_empty() {
        // Summary::new() has empty service map.
        let out = plain_reporter().render(&Summary::new(), &[], &[]);

        assert!(
            !out.contains("SERVICES"),
            "BC-2.11.019 inv3: 'SERVICES' section must be absent when service_counts() empty; \
             got:\n{out}"
        );

        // PROTOCOLS is always present.
        assert!(
            out.contains("PROTOCOLS"),
            "BC-2.11.019 inv5: 'PROTOCOLS' must always be present; got:\n{out}"
        );
    }

    /// AC-015 (BC-2.11.019 postcondition 5):
    /// ANALYZER sections appear last, one per `AnalysisSummary` element, in
    /// slice order.
    ///
    /// Discriminating assertions:
    ///   - Two analyzers "DNS" and "HTTP" in slice order.
    ///   - Both "ANALYZER: DNS" and "ANALYZER: HTTP" present.
    ///   - "ANALYZER: DNS" appears before "ANALYZER: HTTP" (slice order).
    ///   - Both appear AFTER "FINDINGS" (which is after PROTOCOLS).
    ///
    /// pc5: analyzer sections last in slice order.
    #[test]
    fn test_BC_2_11_019_analyzer_sections_last_in_slice_order() {
        let f = make_mitre_finding("some-finding", Verdict::Likely, Confidence::High, None);
        let dns_summary = AnalysisSummary {
            analyzer_name: "DNS".to_string(),
            packets_analyzed: 10,
            detail: std::collections::BTreeMap::new(),
        };
        let http_summary = AnalysisSummary {
            analyzer_name: "HTTP".to_string(),
            packets_analyzed: 20,
            detail: std::collections::BTreeMap::new(),
        };
        let out = plain_reporter().render(&Summary::new(), &[f], &[dns_summary, http_summary]);

        // Both analyzer sections must appear.
        assert!(
            out.contains("ANALYZER: DNS"),
            "BC-2.11.019 pc5: 'ANALYZER: DNS' must appear; got:\n{out}"
        );
        assert!(
            out.contains("ANALYZER: HTTP"),
            "BC-2.11.019 pc5: 'ANALYZER: HTTP' must appear; got:\n{out}"
        );

        // Slice order: DNS (index 0) before HTTP (index 1).
        let pos_dns = out.find("ANALYZER: DNS").expect("ANALYZER: DNS not found");
        let pos_http = out
            .find("ANALYZER: HTTP")
            .expect("ANALYZER: HTTP not found");
        assert!(
            pos_dns < pos_http,
            "BC-2.11.019 pc5: 'ANALYZER: DNS' (slice 0) must appear before \
             'ANALYZER: HTTP' (slice 1); pos_dns={pos_dns}, pos_http={pos_http}"
        );

        // Analyzer sections must appear after FINDINGS.
        let pos_findings = out.find("FINDINGS").expect("FINDINGS not found");
        assert!(
            pos_findings < pos_dns,
            "BC-2.11.019 pc5: 'FINDINGS' must appear before analyzer sections; \
             pos_findings={pos_findings}, pos_dns={pos_dns}"
        );
    }
}
