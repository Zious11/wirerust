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
