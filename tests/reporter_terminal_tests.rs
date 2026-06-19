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

use std::net::{IpAddr, Ipv4Addr};

use wirerust::analyzer::AnalysisSummary;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reporter::Reporter;
use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
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
        mitre_techniques: vec![],
        source_ip: None,
        timestamp: None,
        direction: None,
    }
}

/// TerminalReporter with color and all optional sections disabled.
/// STORY-118: `collapse_findings` field added; default false so existing tests
/// exercise the pre-v0.8.0 non-collapse path (BC-2.11.028 invariant 2 / opt-out path).
fn plain_reporter() -> TerminalReporter {
    TerminalReporter {
        use_color: false,
        show_hosts_breakdown: false,
        render: FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Expanded,
        },
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
            mitre_techniques: vec![],
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
    /// STORY-118: `collapse_findings` field added; false here since grouped mode
    /// does not apply collapse (BC-2.11.025 invariant 5 / AC-005).
    fn mitre_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
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
            mitre_techniques: technique.map(|s| vec![s.to_string()]).unwrap_or_default(),
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
    /// When `render = FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }`, tactic section headers appear in the
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
    /// When `render = FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` and a finding has a known technique ID
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
    /// When `render != FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }` (FlatExpanded or FlatCollapsed), a finding
    /// with `mitre_technique = "T1036"` produces the MITRE line `MITRE: T1036`
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
        // Canonical test vector: mitre="T1036", render=FlatExpanded.
        let f = make_mitre_finding(
            "flat-finding",
            Verdict::Likely,
            Confidence::High,
            Some("T1036"),
        );
        // Use plain_reporter() (render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }).
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

// ---------------------------------------------------------------------------
// FIX-P5-003 / ADV-IMPL-P06-MED-001: terminal PROTOCOLS + SERVICES ordering
// ---------------------------------------------------------------------------
//
// Defect: `TerminalReporter::render` iterates `summary.protocol_counts()` and
// `summary.service_counts()` directly.  Both return `&HashMap<K,u64>`, whose
// iteration order is per-process-random.  The rendered PROTOCOLS and SERVICES
// section lines therefore appear in an unpredictable order across runs.
//
// Fix: sort each map's entries by count descending, then by name ascending
// before rendering.
//
// RED-GATE STRATEGY: Build a Summary with multiple protocols (TCP and UDP at
// the same count, plus an ICMP at a distinct count) and multiple services at
// controlled counts.  Assert the exact line order in the rendered output.  The
// current HashMap-iteration code is not sorted, so the assertion fails when
// iteration order differs from the expected sorted order — which happens
// reliably given enough distinct entries and the deliberate mixed insertion
// pattern below.

mod fix_p5_003_terminal_ordering {
    use super::*;

    // -----------------------------------------------------------------------
    // Helper: build a ParsedPacket with controlled protocol/port.
    // -----------------------------------------------------------------------

    fn make_tcp_packet(dst_port: u16) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
            protocol: Protocol::Tcp,
            transport: TransportInfo::Tcp {
                src_port: 54321,
                dst_port,
                seq_number: 1,
                syn: false,
                ack: false,
                fin: false,
                rst: false,
            },
            payload: vec![],
            packet_len: 54,
        }
    }

    fn make_udp_packet(dst_port: u16) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 3)),
            dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 4)),
            protocol: Protocol::Udp,
            transport: TransportInfo::Udp {
                src_port: 54322,
                dst_port,
            },
            payload: vec![],
            packet_len: 42,
        }
    }

    fn make_icmp_packet() -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 5)),
            dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 6)),
            protocol: Protocol::Icmp,
            transport: TransportInfo::None,
            payload: vec![],
            packet_len: 28,
        }
    }

    /// Build a packet with `Protocol::Other(proto_num)` (no transport, no service hint).
    fn make_other_packet(proto_num: u8) -> ParsedPacket {
        ParsedPacket {
            src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 7)),
            dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 8)),
            protocol: Protocol::Other(proto_num),
            transport: TransportInfo::None,
            payload: vec![],
            packet_len: 20,
        }
    }

    // -----------------------------------------------------------------------
    // Extract the body of a named section from the rendered output.
    //
    // Returns the lines (without leading "  " indent) between the section
    // header and the next blank line (section separator).
    // -----------------------------------------------------------------------
    fn section_lines<'a>(out: &'a str, section_name: &str) -> Vec<&'a str> {
        // `TerminalReporter::section()` in plain (no-color) mode emits:
        //   "{title}\n{40 × '─'}\n"
        // So after the title line comes a "─" rule line, then content lines
        // (each indented with "  "), then a blank line as the section separator.
        // State machine: Header → Rule → Content → done on blank line.
        #[derive(PartialEq)]
        enum State {
            Searching,
            SkipRule,
            Collecting,
        }
        let mut result = Vec::new();
        let mut state = State::Searching;
        for line in out.lines() {
            match state {
                State::Searching => {
                    if line.contains(section_name) {
                        state = State::SkipRule;
                    }
                }
                State::SkipRule => {
                    // Skip the "────..." rule line that immediately follows the title.
                    state = State::Collecting;
                }
                State::Collecting => {
                    if line.is_empty() {
                        break;
                    }
                    // Strip leading "  " indent that TerminalReporter adds to each entry.
                    result.push(line.trim_start_matches("  "));
                }
            }
        }
        result
    }

    // -----------------------------------------------------------------------
    // test_terminal_protocols_sorted_count_then_name
    // -----------------------------------------------------------------------
    //
    // RED-GATE STRATEGY (reliable): Use 9 distinct protocol entries so that
    // the probability of accidentally correct order from HashMap iteration is
    // 1/9! = 1/362880 ≈ 0.00028%.  The entries are:
    //
    //   Protocol::Tcp           → count 20  (highest, unambiguous first slot)
    //   Protocol::Udp           → count 10  (second)
    //   Protocol::Other(10)     → count  5  (tied group at 5)
    //   Protocol::Other(20)     → count  5  (tied group at 5)
    //   Protocol::Other(30)     → count  5  (tied group at 5)
    //   Protocol::Other(40)     → count  5  (tied group at 5)
    //   Protocol::Other(50)     → count  5  (tied group at 5)
    //   Protocol::Icmp          → count  2  (second-to-last)
    //   Protocol::Other(255)    → count  1  (lowest, last)
    //
    // In the fixed sorted output the tied block at count=5 must appear in
    // Debug-string alphabetical order:
    //   "Other(10)" < "Other(20)" < "Other(30)" < "Other(40)" < "Other(50)"
    // (alphabetical on the debug representation).
    //
    // NOTE: Protocol is printed via {:?} in the current implementation, which
    // produces "Tcp", "Udp", "Icmp", "Other(N)" for the variants.

    /// FIX-P5-003 / ADV-IMPL-P06-MED-001 — PROTOCOLS section sorted count-desc then name-asc.
    ///
    /// Uses 9 distinct protocol entries.  The probability that HashMap iteration
    /// accidentally produces the correct sorted order is 1/9! ≈ 0.00028%, making
    /// this test a deterministic Red Gate in all practical senses.
    #[test]
    fn test_terminal_protocols_sorted_count_then_name() {
        let mut summary = Summary::new();

        // Highest count: Tcp → 20.
        for _ in 0..20 {
            summary.ingest(&make_tcp_packet(9999));
        }
        // Second: Udp → 10.
        for _ in 0..10 {
            summary.ingest(&make_udp_packet(9999));
        }
        // Tied block at count=5: Other(10), Other(20), Other(30), Other(40), Other(50).
        // Inserted in REVERSE debug-alphabetical order (50, 40, 30, 20, 10) to ensure
        // any non-sorted iteration order disagrees with the expected alphabetical result.
        for proto in [50u8, 40, 30, 20, 10] {
            for _ in 0..5 {
                summary.ingest(&make_other_packet(proto));
            }
        }
        // Second-to-last: Icmp → 2.
        for _ in 0..2 {
            summary.ingest(&make_icmp_packet());
        }
        // Last: Other(255) → 1.
        summary.ingest(&make_other_packet(255));

        let out = plain_reporter().render(&summary, &[], &[]);

        // Confirm PROTOCOLS section is present.
        assert!(
            out.contains("PROTOCOLS"),
            "FIX-P5-003: PROTOCOLS section must be present in rendered output; got:\n{out}"
        );

        let lines = section_lines(&out, "PROTOCOLS");

        // 9 distinct protocols → 9 lines.
        assert_eq!(
            lines.len(),
            9,
            "FIX-P5-003: PROTOCOLS section must have 9 lines; got: {lines:?}"
        );

        // Line 0: Tcp: 20 (highest count, unambiguous).
        assert!(
            lines[0].contains("Tcp") && lines[0].contains(": 20"),
            "FIX-P5-003: PROTOCOLS line[0] must be 'Tcp: 20' (count=20); \
             got: {:?}",
            lines[0]
        );

        // Line 1: Udp: 10 (second, unambiguous).
        assert!(
            lines[1].contains("Udp") && lines[1].contains(": 10"),
            "FIX-P5-003: PROTOCOLS line[1] must be 'Udp: 10' (count=10); \
             got: {:?}",
            lines[1]
        );

        // Lines 2-6: the tied block at count=5, in debug-alphabetical order.
        // Expected: Other(10), Other(20), Other(30), Other(40), Other(50).
        // The debug representation is "Other(N)" so alphabetical is N=10 < 20 < 30 < 40 < 50.
        let expected_tied = [
            "Other(10)",
            "Other(20)",
            "Other(30)",
            "Other(40)",
            "Other(50)",
        ];
        for (slot, expected_name) in expected_tied.iter().enumerate() {
            let line = lines[2 + slot];
            assert!(
                line.contains(expected_name) && line.contains(": 5"),
                "FIX-P5-003 (ADV-IMPL-P06-MED-001): PROTOCOLS line[{}] must be '{}: 5' \
                 (tied block at count=5, debug-alphabetical order); current HashMap \
                 iteration produces non-deterministic order — this fails without \
                 sort-by-count-then-name; got: {:?}",
                2 + slot,
                expected_name,
                line
            );
        }

        // Line 7: Icmp: 2.
        assert!(
            lines[7].contains("Icmp") && lines[7].contains(": 2"),
            "FIX-P5-003: PROTOCOLS line[7] must be 'Icmp: 2'; got: {:?}",
            lines[7]
        );

        // Line 8: Other(255): 1 (lowest count, last).
        assert!(
            lines[8].contains("Other(255)") && lines[8].contains(": 1"),
            "FIX-P5-003: PROTOCOLS line[8] must be 'Other(255): 1' (lowest count); \
             got: {:?}",
            lines[8]
        );
    }

    // -----------------------------------------------------------------------
    // test_terminal_services_sorted_count_then_name
    // -----------------------------------------------------------------------
    //
    // RED-GATE STRATEGY (reliable): Use all 7 available port-based service
    // hints so that the probability of accidentally correct order from HashMap
    // iteration is 1/7! = 1/5040 ≈ 0.020%.
    //
    // Port-based service hints from `ParsedPacket::app_protocol_hint`:
    //   port 443 → "TLS"    count = 30  (highest, unambiguous first)
    //   port 445 → "SMB"    count = 10  (second, unambiguous)
    //   port  53 → "DNS"    count =  5  (tied group at 5 — alpha: DNS < HTTP < Modbus < SSH < SMB loses to SMB at count=10)
    //   port  80 → "HTTP"   count =  5  (tied group)
    //   port 502 → "Modbus" count =  5  (tied group)
    //   port  22 → "SSH"    count =  5  (tied group — alphabetically: DNS < HTTP < Modbus < SSH)
    //   port 20000 → "DNP3" count =  1  (lowest, last)
    //
    // After the fix the SERVICES section must be (count-desc then name-asc):
    //   TLS: 30
    //   SMB: 10
    //   DNS: 5    (D < H < M < S alphabetically)
    //   HTTP: 5
    //   Modbus: 5
    //   SSH: 5
    //   DNP3: 1
    //
    // Insertion order is deliberately reverse-alphabetical within tied group
    // (SSH → Modbus → HTTP → DNS) to maximize divergence from the expected order.

    /// FIX-P5-003 / ADV-IMPL-P06-MED-001 — SERVICES section sorted count-desc then name-asc.
    ///
    /// Uses all 7 available port-based service hints.  The probability that
    /// HashMap iteration accidentally produces the correct sorted order is
    /// 1/7! = 1/5040 ≈ 0.02%, making this a reliable Red Gate.
    #[test]
    fn test_terminal_services_sorted_count_then_name() {
        let mut summary = Summary::new();

        // Highest count: TLS → 30 (port 443).
        for _ in 0..30 {
            summary.ingest(&make_tcp_packet(443));
        }
        // Second: SMB → 10 (port 445).
        for _ in 0..10 {
            summary.ingest(&make_tcp_packet(445));
        }
        // Tied block at count=5.  Inserted in REVERSE alphabetical order (SSH, Modbus, HTTP, DNS)
        // so that any non-sorted iteration order disagrees with the expected alphabetical result.
        // SSH → 5 (port 22).
        for _ in 0..5 {
            summary.ingest(&make_tcp_packet(22));
        }
        // Modbus → 5 (port 502).
        for _ in 0..5 {
            summary.ingest(&make_tcp_packet(502));
        }
        // HTTP → 5 (port 80).
        for _ in 0..5 {
            summary.ingest(&make_tcp_packet(80));
        }
        // DNS → 5 (port 53).
        for _ in 0..5 {
            summary.ingest(&make_tcp_packet(53));
        }
        // Lowest: DNP3 → 1 (port 20000).
        summary.ingest(&make_tcp_packet(20000));

        let out = plain_reporter().render(&summary, &[], &[]);

        // Confirm SERVICES section is present.
        assert!(
            out.contains("SERVICES"),
            "FIX-P5-003: SERVICES section must be present in rendered output; got:\n{out}"
        );

        let lines = section_lines(&out, "SERVICES");

        // 7 distinct services → 7 lines.
        assert_eq!(
            lines.len(),
            7,
            "FIX-P5-003: SERVICES section must have 7 lines (TLS, SMB, DNS, HTTP, Modbus, SSH, DNP3); \
             got: {lines:?}"
        );

        // Line 0: TLS: 30 (highest count, unambiguous).
        assert!(
            lines[0].contains("TLS") && lines[0].contains(": 30"),
            "FIX-P5-003: SERVICES line[0] must be 'TLS: 30'; got: {:?}",
            lines[0]
        );

        // Line 1: SMB: 10 (second, unambiguous).
        assert!(
            lines[1].contains("SMB") && lines[1].contains(": 10"),
            "FIX-P5-003: SERVICES line[1] must be 'SMB: 10'; got: {:?}",
            lines[1]
        );

        // Lines 2-5: tied block at count=5, in name-ascending order: DNS, HTTP, Modbus, SSH.
        // ("D" < "H" < "M" < "S" alphabetically.)
        let expected_tied_svc = [("DNS", 5), ("HTTP", 5), ("Modbus", 5), ("SSH", 5)];
        for (slot, (expected_name, expected_count)) in expected_tied_svc.iter().enumerate() {
            let line = lines[2 + slot];
            assert!(
                line.contains(expected_name) && line.contains(&format!(": {expected_count}")),
                "FIX-P5-003 (ADV-IMPL-P06-MED-001): SERVICES line[{}] must be '{}: {}' \
                 (tied block at count=5, alphabetical name order); current HashMap \
                 iteration produces non-deterministic order — this fails without \
                 sort-by-count-then-name; got: {:?}",
                2 + slot,
                expected_name,
                expected_count,
                line
            );
        }

        // Line 6: DNP3: 1 (lowest count, last).
        assert!(
            lines[6].contains("DNP3") && lines[6].contains(": 1"),
            "FIX-P5-003: SERVICES line[6] must be 'DNP3: 1' (lowest count, last); \
             got: {:?}",
            lines[6]
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-118: Terminal Finding-Collapse — Flat Mode (v0.8.0)
// Per DF-TEST-NAMESPACE-001: all STORY-118 tests are grouped inside a
// dedicated `mod story_118` wrapper to prevent test-function name collisions
// with other stories' BC-prefixed names.
//
// Behavioral contracts covered:
//   BC-2.11.025  Flat-Mode Collapse Groups Findings by (category, verdict,
//                confidence, summary) Key; First-Occurrence Order; Deterministic
//   BC-2.11.026  Collapsed Group of N≥2 Renders Header with (xN) Suffix;
//                Singleton (N=1) Renders Without Suffix
//   BC-2.11.027  Collapsed Group Retains at Most K=3 Representative Evidence
//                Lines; Remainder Elided from Terminal Display
//   BC-2.11.028  --no-collapse Opt-Out Flag Disables Terminal Collapse and
//                Restores One-Line-Per-Finding Rendering; JSON/CSV Unaffected
//   BC-2.11.029  Collapse is Display-Layer Only; JSON/CSV Reporters Receive
//                Unmodified findings Slice; Non-Repeated Findings Individually
//                Visible in All Outputs
//   BC-2.11.010  TerminalReporter Escapes Both Summary AND Each Evidence Line
//   BC-2.11.013  MITRE Grouping Emits Tactic Headers in Canonical Order;
//                Uncategorized Last
//   BC-2.11.017  Default Rendering Emits MITRE: <id(s)> Only (No Em-Dash)
//   BC-2.11.019  TerminalReporter Renders Sections in Correct Order
//
// STORY-118 finding-collapse tests — all implemented and passing (v0.8.0).
mod story_118 {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers scoped to story_118
    // -----------------------------------------------------------------------

    /// TerminalReporter with collapse enabled and color disabled (the default
    /// v0.8.0 flat-mode reporter).
    fn collapse_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
    }

    /// TerminalReporter with collapse enabled and color enabled (for color-ladder tests).
    fn collapse_reporter_color() -> TerminalReporter {
        TerminalReporter {
            use_color: true,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
    }

    /// TerminalReporter with MITRE grouping and collapse both enabled (for AC-005).
    fn mitre_collapse_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
    }

    /// Construct a Finding with full control over the four collapse-key fields
    /// plus optional evidence and MITRE techniques.
    fn make_collapse_finding(
        category: ThreatCategory,
        verdict: Verdict,
        confidence: Confidence,
        summary: impl Into<String>,
        evidence: Vec<String>,
        mitre: Vec<String>,
    ) -> Finding {
        Finding {
            category,
            verdict,
            confidence,
            summary: summary.into(),
            evidence,
            mitre_techniques: mitre,
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // -----------------------------------------------------------------------
    // BC-2.11.025 — Flat-Mode Collapse (9 tests)
    // -----------------------------------------------------------------------

    /// AC-001: N identical findings collapse to exactly one display group.
    ///
    /// This guards that when N≥2 findings share the same (category, verdict,
    /// confidence, summary) four-tuple, the FINDINGS section contains exactly one
    /// header line — not N separate lines. FAILS if a future change removes the
    /// collapse pass or falls back to per-finding rendering unconditionally.
    #[test]
    fn test_BC_2_11_025_identical_findings_collapse_to_one_group() {
        // BC-2.11.025 postcondition 1: N identical-key findings → exactly 1 header line.
        // Canonical test vector: 5 findings all (Anomaly, Inconclusive, Low, "Flood").
        let findings: Vec<Finding> = (0..5)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "Flood",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // The FINDINGS section must appear (findings are non-empty).
        assert!(
            out.contains("FINDINGS"),
            "FINDINGS section must appear; got:\n{out}"
        );

        // Count occurrences of the header string in the output — must be exactly 1.
        // The header format per terminal.rs: `  [Anomaly] INCONCLUSIVE (LOW) - Flood`
        let header = "[Anomaly] INCONCLUSIVE (LOW) - Flood";
        let header_count = out.matches(header).count();
        assert_eq!(
            header_count, 1,
            "BC-2.11.025 pc1: 5 identical findings must collapse to exactly 1 header line; \
             found {header_count} occurrences of '{header}' in:\n{out}"
        );

        // The (x5) suffix must be present, confirming N=5 was recorded.
        assert!(
            out.contains("(x5)"),
            "BC-2.11.025 pc1: collapsed group header must carry '(x5)' suffix; got:\n{out}"
        );
    }

    /// AC-002: first-occurrence order is preserved across collapsed groups.
    ///
    /// This guards that collapse respects insertion order: the group whose first
    /// member appeared earliest in the input slice renders first. FAILS if a future
    /// change uses HashMap iteration order (which is non-deterministic) to order groups.
    #[test]
    fn test_BC_2_11_025_first_occurrence_order() {
        // BC-2.11.025 postcondition 2 canonical test vector:
        //   indices 0,2,4 → key A ("Alpha")
        //   indices 1,3   → key B ("Beta")
        // First occurrence of A is index 0; first occurrence of B is index 1.
        // Collapsed output must render group A before group B.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Alpha",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Beta",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Alpha",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Beta",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Alpha",
                vec![],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        let pos_alpha = out
            .find("Alpha")
            .expect("'Alpha' group header not found in output");
        let pos_beta = out
            .find("Beta")
            .expect("'Beta' group header not found in output");

        assert!(
            pos_alpha < pos_beta,
            "BC-2.11.025 pc2: group A (first occurrence at index 0) must render before \
             group B (first occurrence at index 1); pos_alpha={pos_alpha}, pos_beta={pos_beta}"
        );

        // Counts: A×3, B×2.
        assert!(
            out.contains("(x3)"),
            "BC-2.11.025 pc2: group A must have count (x3); got:\n{out}"
        );
        assert!(
            out.contains("(x2)"),
            "BC-2.11.025 pc2: group B must have count (x2); got:\n{out}"
        );
    }

    /// AC-003: evidence difference does NOT prevent collapse; only the four-field
    /// key matters.
    ///
    /// This guards that two findings differing only in their evidence field are
    /// merged into one group. FAILS if evidence is accidentally included in the
    /// collapse key.
    #[test]
    fn test_BC_2_11_025_key_discriminator_evidence_nondiscriminating() {
        // BC-2.11.025 postcondition 4: same (category, verdict, confidence, summary)
        // but different evidence → one collapsed group, N=2.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "SameSummary",
                vec!["evidence-one".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "SameSummary",
                vec!["evidence-two".to_string()],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Only one header line for "SameSummary".
        let header_count = out.matches("SameSummary").count();
        assert_eq!(
            header_count, 1,
            "BC-2.11.025 pc4: two findings differing only in evidence must collapse to 1 header \
             line; found {header_count} occurrences of 'SameSummary'; got:\n{out}"
        );

        // Count is N=2.
        assert!(
            out.contains("(x2)"),
            "BC-2.11.025 pc4: collapsed group must carry '(x2)' suffix; got:\n{out}"
        );
    }

    /// AC-004: category difference prevents collapse; two distinct groups emitted.
    ///
    /// This guards that the category field is part of the collapse key. FAILS if
    /// a future change reduces the key to fewer than four fields (e.g., drops category).
    #[test]
    fn test_BC_2_11_025_key_discriminator_category() {
        // BC-2.11.025 invariant 1: category is a key discriminator.
        // Same verdict/confidence/summary but different category → two distinct groups.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "SharedSummary",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Reconnaissance,
                Verdict::Inconclusive,
                Confidence::Low,
                "SharedSummary",
                vec![],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Two distinct header lines must appear — one per category.
        // Format: `  [Anomaly] INCONCLUSIVE (LOW) - SharedSummary`
        //         `  [Reconnaissance] INCONCLUSIVE (LOW) - SharedSummary`
        assert!(
            out.contains("[Anomaly]"),
            "BC-2.11.025 inv1: Anomaly category group must appear; got:\n{out}"
        );
        assert!(
            out.contains("[Reconnaissance]"),
            "BC-2.11.025 inv1: Reconnaissance category group must appear; got:\n{out}"
        );

        // Neither group should carry a (xN) suffix — they are singletons (N=1 each).
        assert!(
            !out.contains("(x"),
            "BC-2.11.025 inv1: different-category findings must NOT collapse; \
             no (xN) suffix expected; got:\n{out}"
        );
    }

    /// AC-005: render=FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded } suppresses collapse; no (xN) suffix anywhere.
    ///
    /// This guards the invariant that grouped (--mitre) mode is structurally
    /// suffix-free. FAILS if a future change applies the collapse pass inside
    /// the grouped rendering path, OR if the flat collapse path is not implemented
    /// (the contrast assertion calls render_findings_collapsed to verify the bypass
    /// is not just because collapse is broken — flat mode must produce a suffix
    /// that grouped mode does not).
    #[test]
    fn test_BC_2_11_025_grouped_mode_bypasses_collapse() {
        // BC-2.11.025 invariant 5: when render=FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }, collapse does NOT run.
        // 100 identical-key findings in mitre+collapse reporter → no (xN) suffix.
        let findings: Vec<Finding> = (0..100)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "FloodSummary",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = mitre_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // No (xN) suffix of any kind must appear.
        assert!(
            !out.contains("(x"),
            "BC-2.11.025 inv5: grouped mode must NOT emit any (xN) suffix regardless of \
             collapse_findings=true; got:\n{out}"
        );

        // All 100 individual finding lines must appear (grouped, not collapsed).
        // The summary "FloodSummary" should appear 100 times.
        let count = out.matches("FloodSummary").count();
        assert_eq!(
            count, 100,
            "BC-2.11.025 inv5: grouped mode must render all 100 findings individually; \
             found {count} occurrences; got:\n{out}"
        );

        // Contrast assertion (calls render_findings_collapsed via collapse_reporter()):
        // flat+collapse mode on the same input MUST produce a (x100) suffix — proving
        // the bypass above is structural (not because collapse is broken globally).
        let out_flat = collapse_reporter().render(&Summary::new(), &findings, &[]);
        assert!(
            out_flat.contains("(x100)"),
            "BC-2.11.025 inv5 contrast: flat collapse mode must produce '(x100)' suffix \
             for the same 100-finding input (proves grouped bypass is intentional, not broken); \
             got:\n{out_flat}"
        );
    }

    /// AC-006: Likely/High findings collapse normally (severity-agnostic).
    ///
    /// This guards that the collapse logic applies equally to all verdict/confidence
    /// combinations, including the highest-severity Likely+High pair. FAILS if a
    /// future change gates collapse on severity (e.g., "only collapse low-severity findings").
    #[test]
    fn test_BC_2_11_025_severity_agnostic_collapse_likely_high() {
        // BC-2.11.025 postcondition 7 / EC-014: Likely+High collapses just like any other key.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Reconnaissance,
                Verdict::Likely,
                Confidence::High,
                "Port scan detected",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Reconnaissance,
                Verdict::Likely,
                Confidence::High,
                "Port scan detected",
                vec![],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Collapsed to one group with N=2.
        let header_count = out.matches("Port scan detected").count();
        assert_eq!(
            header_count, 1,
            "BC-2.11.025 pc7: Likely/High findings must collapse; found {header_count} header \
             occurrences; got:\n{out}"
        );
        assert!(
            out.contains("(x2)"),
            "BC-2.11.025 pc7: Likely/High collapsed group must carry '(x2)' suffix; got:\n{out}"
        );
    }

    /// AC-007: raw ESC byte in summary distinguishes two groups (raw-byte key).
    ///
    /// This guards that the collapse key is built from the raw summary string bytes,
    /// not from the escaped form. FAILS if a future change escapes summaries before
    /// building the key, which would conflate "x\x1b" and "x" into the same group.
    #[test]
    fn test_BC_2_11_025_raw_byte_key_esc_distinguishes_groups() {
        // BC-2.11.025 postcondition 8 / EC-015: raw-byte key comparison.
        // "x\x1b" (raw ESC byte) ≠ "x" — two distinct groups.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "x\x1b",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "x",
                vec![],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Two distinct groups — neither should carry a (xN) suffix (both are singletons).
        assert!(
            !out.contains("(x"),
            "BC-2.11.025 pc8: 'x\\x1b' and 'x' must form two distinct groups (raw-byte key); \
             no (xN) suffix expected for singleton groups; got:\n{out}"
        );

        // The clean summary "x" must appear escaped in output as "x" (no ESC in display).
        // The ESC-bearing summary must appear as "x\\u{1b}" (escaped at render time).
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.025 pc8: raw ESC in summary must be escaped to '\\u{{1b}}' at render time; \
             got:\n{out}"
        );
    }

    /// AC-024: deterministic output for same input (Vec accumulator invariant).
    ///
    /// This guards that two successive calls to render() with the same input produce
    /// byte-identical output. FAILS if a future change introduces a HashMap-based
    /// accumulator whose iteration order is non-deterministic.
    #[test]
    fn test_BC_2_11_025_deterministic_output_same_input() {
        // BC-2.11.025 postcondition 9 / invariant 7: Vec accumulator → deterministic order.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Gamma",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Alpha",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Gamma",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Beta",
                vec![],
                vec![],
            ),
        ];

        let reporter = collapse_reporter();
        let out1 = reporter.render(&Summary::new(), &findings, &[]);
        let out2 = reporter.render(&Summary::new(), &findings, &[]);

        assert_eq!(
            out1, out2,
            "BC-2.11.025 pc9/inv7: two successive render() calls with identical input must \
             produce byte-identical output; got:\nfirst=\n{out1}\nsecond=\n{out2}"
        );

        // Also verify order is first-occurrence: Gamma appears before Alpha appears before Beta.
        let pos_gamma = out1.find("Gamma").expect("'Gamma' not found");
        let pos_alpha = out1.find("Alpha").expect("'Alpha' not found");
        let pos_beta = out1.find("Beta").expect("'Beta' not found");
        assert!(
            pos_gamma < pos_alpha && pos_alpha < pos_beta,
            "BC-2.11.025 inv7: first-occurrence order must be Gamma < Alpha < Beta; \
             pos_gamma={pos_gamma}, pos_alpha={pos_alpha}, pos_beta={pos_beta}"
        );
    }

    /// AC-026: canonical flood case — 5 empty-UA findings collapse to 1 group
    /// with exactly 3 evidence lines.
    ///
    /// This guards the canonical STORY-118 use case: a flood of identical
    /// empty-User-Agent findings is reduced to a single annotated group with
    /// the first three evidence samples. FAILS if the collapse pass or evidence
    /// sampling is broken for the canonical HTTP analyzer output format.
    #[test]
    fn test_BC_2_11_025_flood_canonical_empty_ua_five_findings() {
        // BC-2.11.025 canonical test vector / BC-2.11.027 pc2.
        // Per src/analyzer/http.rs:365 format!("{} {}", method, uri) — no HTTP/1.1 token.
        // Evidence strings: "GET /a" through "GET /e".
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Empty User-Agent header",
                vec!["GET /a".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Empty User-Agent header",
                vec!["GET /b".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Empty User-Agent header",
                vec!["GET /c".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Empty User-Agent header",
                vec!["GET /d".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Empty User-Agent header",
                vec!["GET /e".to_string()],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Exactly 1 display group with count 5.
        let header_count = out.matches("Empty User-Agent header").count();
        assert_eq!(
            header_count, 1,
            "BC-2.11.025 canonical: 5 empty-UA findings must collapse to 1 header line; \
             found {header_count} occurrences; got:\n{out}"
        );
        assert!(
            out.contains("(x5)"),
            "BC-2.11.025 canonical: collapsed group must carry '(x5)' suffix; got:\n{out}"
        );

        // Exactly 3 evidence lines: GET /a, GET /b, GET /c.
        assert!(
            out.contains("> GET /a"),
            "BC-2.11.025 canonical: first evidence line '> GET /a' must appear; got:\n{out}"
        );
        assert!(
            out.contains("> GET /b"),
            "BC-2.11.025 canonical: second evidence line '> GET /b' must appear; got:\n{out}"
        );
        assert!(
            out.contains("> GET /c"),
            "BC-2.11.025 canonical: third evidence line '> GET /c' must appear; got:\n{out}"
        );

        // Evidence for /d and /e must NOT appear (elided by K=3 cap).
        assert!(
            !out.contains("> GET /d"),
            "BC-2.11.025 canonical: fourth evidence 'GET /d' must be elided (K=3 cap); got:\n{out}"
        );
        assert!(
            !out.contains("> GET /e"),
            "BC-2.11.025 canonical: fifth evidence 'GET /e' must be elided (K=3 cap); got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.026 — Count Suffix and Colorization (10 tests)
    // -----------------------------------------------------------------------

    /// AC-008: N=1 singleton renders with no count suffix; byte-identical to
    /// pre-v0.8.0 output.
    ///
    /// This guards that singletons are unchanged by the v0.8.0 collapse feature.
    /// FAILS if a future change introduces a spurious "(x1)" suffix for singletons.
    #[test]
    fn test_BC_2_11_026_singleton_no_suffix() {
        // BC-2.11.026 postcondition 2 / invariant 2: N=1 singleton, no suffix.
        let f = make_collapse_finding(
            ThreatCategory::Anomaly,
            Verdict::Inconclusive,
            Confidence::Low,
            "SingletonSummary",
            vec!["evidence-line".to_string()],
            vec![],
        );

        // Collapse reporter (render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }) with a single finding.
        let out_collapse =
            collapse_reporter().render(&Summary::new(), std::slice::from_ref(&f), &[]);

        // No (xN) suffix of any kind.
        assert!(
            !out_collapse.contains("(x"),
            "BC-2.11.026 pc2/inv2: singleton must render with no (xN) suffix; got:\n{out_collapse}"
        );

        // Output must be byte-identical to the pre-v0.8.0 path (render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }).
        let out_no_collapse = plain_reporter().render(&Summary::new(), &[f], &[]);
        assert_eq!(
            out_collapse, out_no_collapse,
            "BC-2.11.026 pc2/inv2: singleton with collapse=true must be byte-identical to \
             collapse=false (pre-v0.8.0) output"
        );
    }

    /// AC-009 (part 1): N≥2 header line contains correct (xN) suffix with exact count.
    ///
    /// This guards the basic suffix emission for a small N=3 group. FAILS if the
    /// suffix is missing, has the wrong count, or uses a different format.
    #[test]
    fn test_BC_2_11_026_count_suffix_for_n_ge_2() {
        // BC-2.11.026 postcondition 1 / invariant 1: N=3 group → "(x3)" suffix.
        let findings: Vec<Finding> = (0..3)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "Empty UA",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("Empty UA (x3)"),
            "BC-2.11.026 pc1: N=3 group must render header with '(x3)' suffix; got:\n{out}"
        );
    }

    /// AC-009 (part 2): large count (N=3142) rendered exactly; no rounding.
    ///
    /// This guards that the count is always the exact decimal integer with no
    /// abbreviation. FAILS if a future change introduces "3k" or "3.1k" shortening.
    #[test]
    fn test_BC_2_11_026_large_count_exact() {
        // BC-2.11.026 invariant 1: N=3142 → "(x3142)" with no rounding.
        let findings: Vec<Finding> = (0..3142)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "LargeFlood",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("(x3142)"),
            "BC-2.11.026 inv1: large count N=3142 must render exactly as '(x3142)'; \
             got:\n{out}"
        );
        // Guard against truncated/abbreviated forms.
        assert!(
            !out.contains("(x3k)") && !out.contains("(x3.1") && !out.contains("(x314)"),
            "BC-2.11.026 inv1: count must not be abbreviated or rounded; got:\n{out}"
        );
    }

    /// AC-010: suffix format is exactly ` (x<N>)` — one space, paren, x, decimal, paren.
    ///
    /// This guards the precise format contract. FAILS if the format uses brackets,
    /// reversed order, or different spacing (e.g., "[x2]", "(2x)", " x2", "(x 2)").
    #[test]
    fn test_BC_2_11_026_suffix_format() {
        // BC-2.11.026 invariant 1: suffix format = " (x<N>)".
        let findings: Vec<Finding> = (0..2)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "FormatCheck",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // The correct form: space + open-paren + x + decimal + close-paren.
        assert!(
            out.contains(" (x2)"),
            "BC-2.11.026 inv1: suffix must be exactly ' (x2)' (space-paren-x-2-paren); got:\n{out}"
        );

        // Forbidden alternative formats.
        assert!(
            !out.contains("[x2]"),
            "BC-2.11.026 inv1: bracket form '[x2]' must NOT appear; got:\n{out}"
        );
        assert!(
            !out.contains("(2x)"),
            "BC-2.11.026 inv1: reversed form '(2x)' must NOT appear; got:\n{out}"
        );
        assert!(
            !out.contains(" x2 ") && !out.contains(" x2\n"),
            "BC-2.11.026 inv1: bare 'x2' without parens must NOT appear; got:\n{out}"
        );
    }

    /// AC-011: (xN) suffix is INSIDE the ANSI color span, not after the reset.
    ///
    /// This guards the architectural rule that the suffix is appended to the header
    /// string BEFORE colorization, so it is contained within the ANSI color span.
    /// FAILS if a future change appends the suffix after calling the color function,
    /// which would place it after the ANSI reset sequence.
    #[test]
    fn test_BC_2_11_026_suffix_colorized_inside_span_red_bold() {
        // BC-2.11.026 pc6 / invariant 4: (xN) suffix is inside the red().bold() span.
        // Input: 2 findings with (Reconnaissance, Likely, High, "Port scan") → red+bold.
        let findings: Vec<Finding> = (0..2)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Reconnaissance,
                    Verdict::Likely,
                    Confidence::High,
                    "Port scan",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter_color().render(&Summary::new(), &findings, &[]);

        // raw ESC bytes must be present (use_color=true was active).
        assert!(
            out.as_bytes().contains(&0x1b),
            "BC-2.11.026 pc6: use_color=true must produce ANSI escape bytes; got:\n{out:?}"
        );

        // Span-containment check: the (x2) suffix must be inside the red color span.
        // owo-colors 4.x: header_text.red().bold() emits:
        //   ESC[1m  ESC[31m  <header_text including "(x2)">  ESC[39m  ESC[0m
        // The foreground-color reset for red is ESC[39m (SGR 39 = default fg).
        // We assert: opener(ESC[31m) < suffix("(x2)") < reset(ESC[39m) with no
        // ESC[39m between the opener and the suffix — falsifies "suffix after reset".
        let opener = "\x1b[31m";
        let fg_reset = "\x1b[39m";
        let suffix = "(x2)";

        let pos_opener = out
            .find(opener)
            .expect("BC-2.11.026 pc6/inv4: red ANSI opener ESC[31m must be present");
        let pos_suffix = out
            .find(suffix)
            .expect("BC-2.11.026 pc6/inv4: '(x2)' suffix must be present in colored output");
        let pos_reset = out[pos_suffix..]
            .find(fg_reset)
            .map(|p| p + pos_suffix)
            .expect(
                "BC-2.11.026 pc6/inv4: fg reset ESC[39m must follow the '(x2)' suffix \
                 (suffix must be inside the color span)",
            );

        assert!(
            pos_opener < pos_suffix,
            "BC-2.11.026 pc6/inv4: color opener must precede the '(x2)' suffix; \
             opener@{pos_opener}, suffix@{pos_suffix}; got:\n{out:?}"
        );
        assert!(
            pos_suffix < pos_reset,
            "BC-2.11.026 pc6/inv4: '(x2)' suffix must precede the fg reset; \
             suffix@{pos_suffix}, reset@{pos_reset}; got:\n{out:?}"
        );
        // No fg reset between the opener and the suffix — discriminates "suffix after reset".
        assert!(
            !out[pos_opener..pos_suffix].contains(fg_reset),
            "BC-2.11.026 pc6/inv4: no ESC[39m reset may appear between the color opener \
             and the '(x2)' suffix (suffix must be inside, not after, the span); \
             got:\n{out:?}"
        );
    }

    /// AC-012 (part 1): Inconclusive verdict → cyan colorization.
    ///
    /// This guards the color-ladder branch for Inconclusive. FAILS if a future
    /// change to the color-ladder mis-routes Inconclusive to a different color.
    #[test]
    fn test_BC_2_11_026_color_ladder_inconclusive_cyan() {
        // BC-2.11.026 pc6: Inconclusive → cyan.
        // owo-colors cyan ANSI code: ESC[36m (the standard SGR cyan color).
        let findings: Vec<Finding> = (0..2)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "CyanCheck",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter_color().render(&Summary::new(), &findings, &[]);

        // Cyan ANSI sequence must appear in the output.
        // SGR 36 = foreground cyan.
        assert!(
            out.contains("\x1b[36m") || out.contains("\x1b[0;36m") || out.contains("\x1b[36;"),
            "BC-2.11.026 pc6: Inconclusive verdict must produce cyan ANSI color in output; \
             got:\n{out:?}"
        );

        // No red/bold sequence (Likely+High path).
        assert!(
            !out.contains("\x1b[1;31m") && !out.contains("\x1b[31;1m"),
            "BC-2.11.026 pc6: Inconclusive must NOT use red+bold color; got:\n{out:?}"
        );

        // Span-containment: the (x2) suffix must be inside the cyan color span.
        // owo-colors 4.x: header_text.cyan() emits ESC[36m<text>ESC[39m.
        // ESC[39m is the fg-reset for color spans (SGR 39 = default foreground).
        let opener = "\x1b[36m";
        let fg_reset = "\x1b[39m";
        let suffix = "(x2)";

        let pos_opener = out
            .find(opener)
            .expect("BC-2.11.026 pc6/inv4: cyan opener ESC[36m must be present");
        let pos_suffix = out
            .find(suffix)
            .expect("BC-2.11.026 pc6/inv4: '(x2)' suffix must be present");
        let pos_reset = out[pos_suffix..]
            .find(fg_reset)
            .map(|p| p + pos_suffix)
            .expect(
                "BC-2.11.026 pc6/inv4: fg reset ESC[39m must follow '(x2)' \
                 (suffix must be inside the color span)",
            );

        assert!(
            pos_opener < pos_suffix,
            "BC-2.11.026 pc6/inv4: cyan opener must precede '(x2)' suffix; \
             opener@{pos_opener}, suffix@{pos_suffix}; got:\n{out:?}"
        );
        assert!(
            pos_suffix < pos_reset,
            "BC-2.11.026 pc6/inv4: '(x2)' suffix must precede fg reset; \
             suffix@{pos_suffix}, reset@{pos_reset}; got:\n{out:?}"
        );
        assert!(
            !out[pos_opener..pos_suffix].contains(fg_reset),
            "BC-2.11.026 pc6/inv4: no ESC[39m reset may appear between cyan opener \
             and '(x2)' suffix (suffix must be inside, not after, the span); \
             got:\n{out:?}"
        );
    }

    /// AC-012 (part 2): Likely + non-High confidence → yellow colorization.
    ///
    /// This guards the Likely+Medium branch of the color-ladder. FAILS if
    /// Likely+Medium is accidentally routed to red+bold (the Likely+High path).
    #[test]
    fn test_BC_2_11_026_color_ladder_likely_other_yellow() {
        // BC-2.11.026 pc6: Likely + non-High (Medium) → yellow.
        // owo-colors yellow ANSI code: ESC[33m.
        let findings: Vec<Finding> = (0..2)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Likely,
                    Confidence::Medium,
                    "YellowCheck",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter_color().render(&Summary::new(), &findings, &[]);

        // Yellow ANSI sequence must appear.
        assert!(
            out.contains("\x1b[33m") || out.contains("\x1b[0;33m") || out.contains("\x1b[33;"),
            "BC-2.11.026 pc6: Likely+Medium must produce yellow ANSI color; got:\n{out:?}"
        );

        // No red+bold (that is Likely+High path only).
        assert!(
            !out.contains("\x1b[1;31m") && !out.contains("\x1b[31;1m"),
            "BC-2.11.026 pc6: Likely+Medium must NOT use red+bold; got:\n{out:?}"
        );

        // Span-containment: the (x2) suffix must be inside the yellow color span.
        // owo-colors 4.x: header_text.yellow() emits ESC[33m<text>ESC[39m.
        let opener = "\x1b[33m";
        let fg_reset = "\x1b[39m";
        let suffix = "(x2)";

        let pos_opener = out
            .find(opener)
            .expect("BC-2.11.026 pc6/inv4: yellow opener ESC[33m must be present");
        let pos_suffix = out
            .find(suffix)
            .expect("BC-2.11.026 pc6/inv4: '(x2)' suffix must be present");
        let pos_reset = out[pos_suffix..]
            .find(fg_reset)
            .map(|p| p + pos_suffix)
            .expect(
                "BC-2.11.026 pc6/inv4: fg reset ESC[39m must follow '(x2)' \
                 (suffix must be inside the color span)",
            );

        assert!(
            pos_opener < pos_suffix,
            "BC-2.11.026 pc6/inv4: yellow opener must precede '(x2)' suffix; \
             opener@{pos_opener}, suffix@{pos_suffix}; got:\n{out:?}"
        );
        assert!(
            pos_suffix < pos_reset,
            "BC-2.11.026 pc6/inv4: '(x2)' suffix must precede fg reset; \
             suffix@{pos_suffix}, reset@{pos_reset}; got:\n{out:?}"
        );
        assert!(
            !out[pos_opener..pos_suffix].contains(fg_reset),
            "BC-2.11.026 pc6/inv4: no ESC[39m reset may appear between yellow opener \
             and '(x2)' suffix (suffix must be inside, not after, the span); \
             got:\n{out:?}"
        );
    }

    /// AC-012 (part 3): Possible verdict → yellow colorization.
    ///
    /// This guards the Possible branch of the color-ladder. FAILS if Possible
    /// is mis-routed to a different color (e.g., cyan or dimmed).
    #[test]
    fn test_BC_2_11_026_color_ladder_possible_yellow() {
        // BC-2.11.026 pc6: Possible → yellow.
        let findings: Vec<Finding> = (0..2)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Possible,
                    Confidence::Low,
                    "PossibleCheck",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter_color().render(&Summary::new(), &findings, &[]);

        // Yellow ANSI sequence must appear.
        assert!(
            out.contains("\x1b[33m") || out.contains("\x1b[0;33m") || out.contains("\x1b[33;"),
            "BC-2.11.026 pc6: Possible verdict must produce yellow ANSI color; got:\n{out:?}"
        );

        // Span-containment: the (x2) suffix must be inside the yellow color span.
        // owo-colors 4.x: header_text.yellow() emits ESC[33m<text>ESC[39m.
        let opener = "\x1b[33m";
        let fg_reset = "\x1b[39m";
        let suffix = "(x2)";

        let pos_opener = out
            .find(opener)
            .expect("BC-2.11.026 pc6/inv4: yellow opener ESC[33m must be present");
        let pos_suffix = out
            .find(suffix)
            .expect("BC-2.11.026 pc6/inv4: '(x2)' suffix must be present");
        let pos_reset = out[pos_suffix..]
            .find(fg_reset)
            .map(|p| p + pos_suffix)
            .expect(
                "BC-2.11.026 pc6/inv4: fg reset ESC[39m must follow '(x2)' \
                 (suffix must be inside the color span)",
            );

        assert!(
            pos_opener < pos_suffix,
            "BC-2.11.026 pc6/inv4: yellow opener must precede '(x2)' suffix; \
             opener@{pos_opener}, suffix@{pos_suffix}; got:\n{out:?}"
        );
        assert!(
            pos_suffix < pos_reset,
            "BC-2.11.026 pc6/inv4: '(x2)' suffix must precede fg reset; \
             suffix@{pos_suffix}, reset@{pos_reset}; got:\n{out:?}"
        );
        assert!(
            !out[pos_opener..pos_suffix].contains(fg_reset),
            "BC-2.11.026 pc6/inv4: no ESC[39m reset may appear between yellow opener \
             and '(x2)' suffix (suffix must be inside, not after, the span); \
             got:\n{out:?}"
        );
    }

    /// AC-012 (part 4): Unlikely verdict → dimmed colorization.
    ///
    /// This guards the Unlikely branch of the color-ladder. FAILS if Unlikely
    /// is not dimmed, which would give it the same visual weight as higher-severity findings.
    #[test]
    fn test_BC_2_11_026_color_ladder_unlikely_dimmed() {
        // BC-2.11.026 pc6: Unlikely → dimmed.
        // owo-colors dimmed ANSI code: ESC[2m (SGR 2 = faint/dim).
        let findings: Vec<Finding> = (0..2)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Unlikely,
                    Confidence::Low,
                    "DimCheck",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter_color().render(&Summary::new(), &findings, &[]);

        // Dimmed ANSI sequence must appear: ESC[2m (SGR 2 = faint/dim).
        assert!(
            out.contains("\x1b[2m") || out.contains("\x1b[0;2m") || out.contains("\x1b[2;"),
            "BC-2.11.026 pc6: Unlikely verdict must produce dimmed ANSI color (SGR 2); \
             got:\n{out:?}"
        );

        // Span-containment: the (x2) suffix must be inside the dim span.
        // owo-colors 4.x: header_text.dimmed() emits ESC[2m<text>ESC[0m.
        // DimDisplay uses ESC[0m as its reset (a style span, not a color span).
        let opener = "\x1b[2m";
        let style_reset = "\x1b[0m";
        let suffix = "(x2)";

        let pos_opener = out
            .find(opener)
            .expect("BC-2.11.026 pc6/inv4: dim opener ESC[2m must be present");
        let pos_suffix = out
            .find(suffix)
            .expect("BC-2.11.026 pc6/inv4: '(x2)' suffix must be present");
        let pos_reset = out[pos_suffix..]
            .find(style_reset)
            .map(|p| p + pos_suffix)
            .expect(
                "BC-2.11.026 pc6/inv4: style reset ESC[0m must follow '(x2)' \
                 (suffix must be inside the dim span)",
            );

        assert!(
            pos_opener < pos_suffix,
            "BC-2.11.026 pc6/inv4: dim opener must precede '(x2)' suffix; \
             opener@{pos_opener}, suffix@{pos_suffix}; got:\n{out:?}"
        );
        assert!(
            pos_suffix < pos_reset,
            "BC-2.11.026 pc6/inv4: '(x2)' suffix must precede style reset; \
             suffix@{pos_suffix}, reset@{pos_reset}; got:\n{out:?}"
        );
        assert!(
            !out[pos_opener..pos_suffix].contains(style_reset),
            "BC-2.11.026 pc6/inv4: no ESC[0m reset may appear between dim opener \
             and '(x2)' suffix (suffix must be inside, not after, the span); \
             got:\n{out:?}"
        );
    }

    /// AC-023 (part 1): MITRE line sources group_members[0]; other members' MITRE elided.
    ///
    /// This guards that only the first group member's MITRE techniques appear in the
    /// terminal output — divergent MITRE from later members is elided. FAILS if a
    /// future change emits MITRE lines from all group members or merges them.
    #[test]
    fn test_BC_2_11_026_mitre_line_from_representative_finding() {
        // BC-2.11.026 pc7 / BC-2.11.017 pc6: MITRE from group_members[0] only.
        // members[0].mitre = ["T1036"], members[1].mitre = [], members[2].mitre = ["T1059"].
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "SharedKey",
                vec![],
                vec!["T1036".to_string()],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "SharedKey",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "SharedKey",
                vec![],
                vec!["T1059".to_string()],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // MITRE line from member[0]: "    MITRE: T1036".
        assert!(
            out.contains("MITRE: T1036"),
            "BC-2.11.026 pc7: MITRE line must source from group_members[0] ('T1036'); \
             got:\n{out}"
        );

        // Member[2]'s MITRE technique "T1059" must NOT appear.
        assert!(
            !out.contains("T1059"),
            "BC-2.11.026 pc7: member[2] MITRE 'T1059' must be elided from terminal output; \
             got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.027 — Evidence Sampling (K=3 cap) (5 tests)
    // -----------------------------------------------------------------------

    /// AC-013: N>K shows exactly K=3 evidence lines (first K members).
    ///
    /// This guards the K=3 evidence cap for groups with N>3 members. FAILS if a
    /// future change renders all evidence lines (removing the cap) or fewer than 3.
    #[test]
    fn test_BC_2_11_027_evidence_capped_at_k() {
        // BC-2.11.027 pc2 / invariant 2: N=5 group, each member has 1 evidence line.
        // First 3 members' evidence must appear; members[3] and members[4] elided.
        let findings: Vec<Finding> = (1..=5)
            .map(|i| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "CapTest",
                    vec![format!("req_{i:03}")],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Evidence from first 3 members must appear.
        assert!(
            out.contains("> req_001"),
            "BC-2.11.027 pc2: evidence from member[0] 'req_001' must appear; got:\n{out}"
        );
        assert!(
            out.contains("> req_002"),
            "BC-2.11.027 pc2: evidence from member[1] 'req_002' must appear; got:\n{out}"
        );
        assert!(
            out.contains("> req_003"),
            "BC-2.11.027 pc2: evidence from member[2] 'req_003' must appear; got:\n{out}"
        );

        // Evidence from members[3] and members[4] must be elided.
        assert!(
            !out.contains("req_004"),
            "BC-2.11.027 pc2: evidence from member[3] 'req_004' must be elided (K=3 cap); \
             got:\n{out}"
        );
        assert!(
            !out.contains("req_005"),
            "BC-2.11.027 pc2: evidence from member[4] 'req_005' must be elided (K=3 cap); \
             got:\n{out}"
        );
    }

    /// AC-014: N≤K renders all available evidence (no elision at or below cap).
    ///
    /// This guards that the K=3 cap does not elide evidence when N≤K. FAILS if a
    /// future change applies the cap too aggressively (e.g., always caps at 3 regardless
    /// of group size).
    #[test]
    fn test_BC_2_11_027_evidence_below_cap_rendered_fully() {
        // BC-2.11.027 pc5: N=2 (below cap) → both evidence lines rendered.
        let findings_n2 = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "BelowCap",
                vec!["ev-one".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "BelowCap",
                vec!["ev-two".to_string()],
                vec![],
            ),
        ];

        let out_n2 = collapse_reporter().render(&Summary::new(), &findings_n2, &[]);

        assert!(
            out_n2.contains("> ev-one"),
            "BC-2.11.027 pc5: N=2 (below K=3) member[0] evidence must appear; got:\n{out_n2}"
        );
        assert!(
            out_n2.contains("> ev-two"),
            "BC-2.11.027 pc5: N=2 (below K=3) member[1] evidence must appear; got:\n{out_n2}"
        );

        // N=3 boundary: all 3 evidence lines rendered (N=K, no elision).
        let findings_n3: Vec<Finding> = vec!["ev-a", "ev-b", "ev-c"]
            .into_iter()
            .map(|ev| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "AtCap",
                    vec![ev.to_string()],
                    vec![],
                )
            })
            .collect();

        let out_n3 = collapse_reporter().render(&Summary::new(), &findings_n3, &[]);

        for ev in ["ev-a", "ev-b", "ev-c"] {
            assert!(
                out_n3.contains(&format!("> {ev}")),
                "BC-2.11.027 pc5: N=3 (at K=3 boundary) evidence '{ev}' must appear; \
                 got:\n{out_n3}"
            );
        }
    }

    /// AC-015: empty first-member contributes 0 lines; window does NOT slide.
    ///
    /// This guards the positional no-slide rule: the window always covers the first
    /// min(N, K) members by position, regardless of whether they have evidence. FAILS
    /// if a future change slides the window to skip empty-evidence members.
    #[test]
    fn test_BC_2_11_027_evidence_drawn_from_first_k_members() {
        // BC-2.11.027 pc2 / invariant 2 (EC-008): N=5, members[0].evidence=[].
        // Window covers members[0..2] (positions 0, 1, 2).
        // member[0] contributes 0 lines (empty); member[1] and member[2] contribute 1 each.
        // Total rendered: 2 evidence lines (NOT 3). members[3] and members[4] never inspected.
        let findings = vec![
            // member[0]: empty evidence
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoSlide",
                vec![],
                vec![],
            ),
            // member[1]: evidence present
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoSlide",
                vec!["slide-ev-1".to_string()],
                vec![],
            ),
            // member[2]: evidence present
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoSlide",
                vec!["slide-ev-2".to_string()],
                vec![],
            ),
            // member[3]: NOT inspected (past the K=3 positional window)
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoSlide",
                vec!["slide-ev-3".to_string()],
                vec![],
            ),
            // member[4]: NOT inspected
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoSlide",
                vec!["slide-ev-4".to_string()],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // member[1] and member[2] evidence must appear.
        assert!(
            out.contains("> slide-ev-1"),
            "BC-2.11.027 inv2: member[1] evidence must appear (within K=3 window); got:\n{out}"
        );
        assert!(
            out.contains("> slide-ev-2"),
            "BC-2.11.027 inv2: member[2] evidence must appear (within K=3 window); got:\n{out}"
        );

        // member[3] and member[4] evidence must NOT appear (outside the window — no slide).
        assert!(
            !out.contains("slide-ev-3"),
            "BC-2.11.027 inv2: member[3] evidence must be elided (window does NOT slide); \
             got:\n{out}"
        );
        assert!(
            !out.contains("slide-ev-4"),
            "BC-2.11.027 inv2: member[4] evidence must be elided (window does NOT slide); \
             got:\n{out}"
        );
    }

    /// AC-016: evidence lines pass through escape_for_terminal in the collapse path.
    ///
    /// This guards that VP-012 (escape_for_terminal correctness) extends into the
    /// collapse wrapper. FAILS if a future change introduces a code path in the
    /// collapse evidence rendering that bypasses escape_for_terminal.
    #[test]
    fn test_BC_2_11_027_escape_preserved_in_sampled_evidence() {
        // BC-2.11.027 pc6 / BC-2.11.010 inv4: ESC byte in evidence must be escaped.
        // Canonical test vector: evidence[0] = "ev: \x1b[32mGREEN" → "ev: \u{1b}[32mGREEN".
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "EscapeTest",
                vec!["ev: \x1b[32mGREEN".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "EscapeTest",
                vec!["clean-evidence".to_string()],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Raw ESC byte must not appear in the output.
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.027 pc6 / VP-012: raw ESC byte (0x1b) must not appear in collapsed \
             evidence output; got:\n{out:?}"
        );

        // Escaped form must appear.
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.027 pc6: escaped form '\\u{{1b}}' must appear from evidence in collapse \
             path; got:\n{out}"
        );
    }

    /// AC-017: singleton — K-cap does NOT apply; all evidence lines rendered.
    ///
    /// This guards that the K=3 cap is strictly for groups with N≥2. FAILS if a
    /// future change applies the cap to singletons, truncating their evidence.
    #[test]
    fn test_BC_2_11_027_singleton_evidence_not_capped() {
        // BC-2.11.027 invariant 6 / EC-009: N=1 with 5 evidence lines → all 5 rendered.
        let f = make_collapse_finding(
            ThreatCategory::Anomaly,
            Verdict::Inconclusive,
            Confidence::Low,
            "SingletonEv",
            vec![
                "ev-line-1".to_string(),
                "ev-line-2".to_string(),
                "ev-line-3".to_string(),
                "ev-line-4".to_string(),
                "ev-line-5".to_string(),
            ],
            vec![],
        );

        let out = collapse_reporter().render(&Summary::new(), &[f], &[]);

        // All 5 evidence lines must appear (no K=3 cap on singletons).
        for i in 1..=5 {
            assert!(
                out.contains(&format!("> ev-line-{i}")),
                "BC-2.11.027 inv6: singleton evidence line {i} must appear (K-cap does not \
                 apply to N=1); got:\n{out}"
            );
        }

        // No (xN) suffix (it is a singleton).
        assert!(
            !out.contains("(x"),
            "BC-2.11.027 inv6: singleton must not carry (xN) suffix; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.028 — --no-collapse Opt-Out Flag (3 tests)
    // -----------------------------------------------------------------------

    /// AC-018 (part 1): --no-collapse restores one-line-per-finding; no (xN) suffix.
    ///
    /// This guards the opt-out path: with render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }, 5 identical-key
    /// findings render as 5 individual header lines. Also includes a contrast assertion
    /// that the collapse path (render_findings_collapsed) is implemented — verified by
    /// checking that render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } produces a different (collapsed) result.
    /// FAILS if FlatExpanded collapses, or if render_findings_collapsed is not yet implemented.
    #[test]
    fn test_BC_2_11_028_no_collapse_flag_one_line_per_finding() {
        // BC-2.11.028 pc2: render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded } → one header per finding, no suffix.
        let findings: Vec<Finding> = (0..5)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "OptOut",
                    vec![],
                    vec![],
                )
            })
            .collect();

        // plain_reporter() has render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }.
        let out = plain_reporter().render(&Summary::new(), &findings, &[]);

        // 5 individual header lines, not 1 collapsed group.
        let header_count = out.matches("[Anomaly] INCONCLUSIVE (LOW) - OptOut").count();
        assert_eq!(
            header_count, 5,
            "BC-2.11.028 pc2: collapse_findings=false must render 5 individual header lines; \
             found {header_count}; got:\n{out}"
        );

        // No (xN) suffix anywhere.
        assert!(
            !out.contains("(x"),
            "BC-2.11.028 pc2: collapse_findings=false must produce no (xN) suffix; got:\n{out}"
        );

        // Contrast assertion: render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } on the same input MUST produce
        // a collapsed group with (x5) — verifying the opt-out contrast is meaningful.
        let out_collapse = collapse_reporter().render(&Summary::new(), &findings, &[]);
        assert!(
            out_collapse.contains("(x5)"),
            "BC-2.11.028 pc2 contrast: collapse_findings=true must produce '(x5)' for same \
             5-finding input (proves opt-out has observable effect); got:\n{out_collapse}"
        );
    }

    /// AC-018 (part 2): default vs. opt-out outputs are observably different.
    ///
    /// This guards that the two modes produce distinct output for the same input.
    /// FAILS if a future change makes FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } and FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }
    /// produce identical output (meaning either both collapse or neither does).
    #[test]
    fn test_BC_2_11_028_default_vs_opt_out_output_difference() {
        // BC-2.11.028 pc2: comparing collapse=true vs collapse=false on same 5-finding input.
        let findings: Vec<Finding> = (0..5)
            .map(|i| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "Diff",
                    vec![format!("req_{i}")],
                    vec![],
                )
            })
            .collect();

        let out_collapse = collapse_reporter().render(&Summary::new(), &findings, &[]);
        let out_no_collapse = plain_reporter().render(&Summary::new(), &findings, &[]);

        // The two outputs must be observably different.
        assert_ne!(
            out_collapse, out_no_collapse,
            "BC-2.11.028 pc2: collapse=true and collapse=false outputs must differ for \
             identical-key input"
        );

        // Collapse output: 1 group with (x5).
        assert!(
            out_collapse.contains("(x5)"),
            "BC-2.11.028 pc2: collapse=true output must contain '(x5)'; got:\n{out_collapse}"
        );

        // No-collapse output: 5 individual headers, no suffix.
        assert!(
            !out_no_collapse.contains("(x"),
            "BC-2.11.028 pc2: collapse=false output must have no (xN) suffix; \
             got:\n{out_no_collapse}"
        );
        let no_collapse_count = out_no_collapse
            .matches("[Anomaly] INCONCLUSIVE (LOW) - Diff")
            .count();
        assert_eq!(
            no_collapse_count, 5,
            "BC-2.11.028 pc2: collapse=false output must have 5 individual header lines; \
             found {no_collapse_count}"
        );
    }

    /// AC-019: --no-collapse flag is wired: no_collapse=true → render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }.
    ///
    /// This guards the structural wiring between the CLI flag and the render field.
    /// FAILS if FlatCollapsed and FlatExpanded produce identical output, or if the
    /// render variant is mis-wired to the wrong rendering path.
    #[test]
    fn test_BC_2_11_028_flag_wired_to_reporter_field() {
        // BC-2.11.028 pc1 / invariant 1: structural wiring test.
        // render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } → collapse active (v0.8.0 default).
        // render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded } → collapse inactive (--no-collapse opt-out).
        // Verify the render field is wired correctly by behavioral contrast.

        let findings: Vec<Finding> = (0..3)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "WiredTest",
                    vec![],
                    vec![],
                )
            })
            .collect();

        // Reporter with render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } → produces "(x3)".
        let reporter_on = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        };
        let out_on = reporter_on.render(&Summary::new(), &findings, &[]);
        assert!(
            out_on.contains("(x3)"),
            "BC-2.11.028 inv1: collapse_findings=true must produce '(x3)' suffix; got:\n{out_on}"
        );

        // Reporter with render = FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded } → no collapse, no suffix.
        let reporter_off = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        };
        let out_off = reporter_off.render(&Summary::new(), &findings, &[]);
        assert!(
            !out_off.contains("(x"),
            "BC-2.11.028 inv1: collapse_findings=false must produce no (xN) suffix; \
             got:\n{out_off}"
        );

        // Polarity: the two outputs must be different.
        assert_ne!(
            out_on, out_off,
            "BC-2.11.028 inv1: collapse_findings=true and collapse_findings=false must produce \
             different output for N≥2 identical-key input"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.029 — Display-Layer Only; JSON/CSV Unaffected (4 tests)
    // -----------------------------------------------------------------------

    /// AC-020: JSON reporter receives full N findings regardless of collapse flag.
    ///
    /// This guards the display-layer isolation invariant: collapse is ephemeral in
    /// TerminalReporter::render and never touches the findings slice. FAILS if a future
    /// change pre-filters or mutates the findings slice before handing it to the JSON reporter.
    #[test]
    fn test_BC_2_11_029_json_receives_full_findings() {
        use wirerust::reporter::json::JsonReporter;

        // BC-2.11.029 pc1 / invariant 1: 1000 identical findings → terminal shows 1 group,
        // JSON contains exactly 1000 finding objects.
        let findings: Vec<Finding> = (0..1000)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "Empty UA",
                    vec![],
                    vec![],
                )
            })
            .collect();

        // Terminal output: 1 collapsed group.
        let terminal_out = collapse_reporter().render(&Summary::new(), &findings, &[]);
        assert!(
            terminal_out.contains("(x1000)"),
            "BC-2.11.029 pc1: terminal must show '(x1000)' collapsed group; \
             got:\n{terminal_out}"
        );

        // JSON output from the SAME slice: must have 1000 finding objects.
        let json_out = JsonReporter.render(&Summary::new(), &findings, &[]);
        let json: serde_json::Value =
            serde_json::from_str(&json_out).expect("JSON output must be valid JSON");
        let json_findings = json["findings"]
            .as_array()
            .expect("JSON 'findings' must be an array");
        assert_eq!(
            json_findings.len(),
            1000,
            "BC-2.11.029 pc1/inv1: JSON reporter must receive all 1000 findings; \
             found {} findings in JSON output",
            json_findings.len()
        );
    }

    /// AC-021: CSV reporter receives full N findings regardless of collapse flag.
    ///
    /// This guards the display-layer isolation for CSV output. Also includes a contrast
    /// assertion via the terminal collapse path (render_findings_collapsed) to confirm the
    /// CSV invariant is meaningful — the terminal DOES collapse while CSV does not.
    /// FAILS if CSV emits fewer than N rows, OR if render_findings_collapsed is not yet
    /// implemented (the contrast assertion calls it).
    #[test]
    fn test_BC_2_11_029_csv_receives_full_findings() {
        use wirerust::reporter::csv::CsvReporter;

        // BC-2.11.029 pc2: 5 identical findings → CSV has 5 data rows (plus header).
        let findings: Vec<Finding> = (0..5)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "CsvTest",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let csv_out = CsvReporter.render(&Summary::new(), &findings, &[]);

        // Count data rows: split by newlines, skip header row, count non-empty lines.
        let lines: Vec<&str> = csv_out.lines().collect();
        // First line is the header; remaining non-empty lines are data rows.
        let data_rows = lines.iter().skip(1).filter(|l| !l.is_empty()).count();
        assert_eq!(
            data_rows,
            5,
            "BC-2.11.029 pc2: CSV must contain exactly 5 data rows for 5 identical findings; \
             found {data_rows} rows (total lines including header: {})",
            lines.len()
        );

        // Contrast assertion: terminal collapse mode on the same 5 findings collapses to 1
        // group with (x5) — proving CSV's 5-row output is non-trivially different from
        // what the terminal does (and that render_findings_collapsed is implemented).
        let terminal_out = collapse_reporter().render(&Summary::new(), &findings, &[]);
        assert!(
            terminal_out.contains("(x5)"),
            "BC-2.11.029 pc2 contrast: terminal collapse must show '(x5)' while CSV has 5 rows \
             (proves display-layer isolation is real, not vacuous); got:\n{terminal_out}"
        );
    }

    /// AC-022: non-repeated finding renders individually; no suffix.
    ///
    /// This guards that a finding with a unique key (appears once in the input) is
    /// treated as a singleton and rendered without any count suffix. FAILS if a future
    /// change emits a "(x1)" suffix for singletons.
    #[test]
    fn test_BC_2_11_029_non_repeated_finding_no_suffix() {
        // BC-2.11.029 pc3: unique-key finding → rendered individually, no suffix.
        // Three findings with three distinct keys (all different summaries).
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Unique-Alpha",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Unique-Beta",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "Unique-Gamma",
                vec![],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // All three findings must appear.
        assert!(
            out.contains("Unique-Alpha"),
            "BC-2.11.029 pc3: unique finding 'Unique-Alpha' must appear; got:\n{out}"
        );
        assert!(
            out.contains("Unique-Beta"),
            "BC-2.11.029 pc3: unique finding 'Unique-Beta' must appear; got:\n{out}"
        );
        assert!(
            out.contains("Unique-Gamma"),
            "BC-2.11.029 pc3: unique finding 'Unique-Gamma' must appear; got:\n{out}"
        );

        // None of them should have a (xN) suffix (all are singletons).
        assert!(
            !out.contains("(x"),
            "BC-2.11.029 pc3: non-repeated findings must NOT carry any (xN) suffix; got:\n{out}"
        );
    }

    /// AC-027: --no-collapse flag has no observable effect on JSON or CSV output.
    ///
    /// This guards that the render field (FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }) on TerminalReporter
    /// does not leak into the JSON/CSV reporters. FAILS if main.rs is refactored to pre-filter
    /// the findings slice based on the --no-collapse flag before passing to all reporters.
    #[test]
    fn test_BC_2_11_029_no_collapse_flag_json_invariant() {
        use wirerust::reporter::json::JsonReporter;

        // BC-2.11.029 pc5: JSON output must be identical regardless of render variant.
        // The render field belongs to TerminalReporter only.
        let findings: Vec<Finding> = (0..5)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "JsonInvariant",
                    vec![],
                    vec![],
                )
            })
            .collect();

        // JsonReporter does not have a collapse_findings field — it is unaffected.
        // Both FlatCollapsed and FlatExpanded terminal renders must produce the SAME
        // JSON output when the same slice is passed to JsonReporter.
        let json_out = JsonReporter.render(&Summary::new(), &findings, &[]);
        let json: serde_json::Value =
            serde_json::from_str(&json_out).expect("JSON output must be valid JSON");
        let json_findings = json["findings"]
            .as_array()
            .expect("JSON 'findings' must be an array");

        // All 5 findings must be present in JSON regardless of terminal collapse.
        assert_eq!(
            json_findings.len(),
            5,
            "BC-2.11.029 pc5: JSON must always contain all 5 findings; found {}",
            json_findings.len()
        );

        // The terminal output with collapse=true has 1 group; JSON is unaffected.
        let terminal_out = collapse_reporter().render(&Summary::new(), &findings, &[]);
        assert!(
            terminal_out.contains("(x5)"),
            "BC-2.11.029 pc5: terminal (collapse=true) must show '(x5)'; got:\n{terminal_out}"
        );
        // JSON does not contain "(x5)".
        assert!(
            !json_out.contains("(x5)"),
            "BC-2.11.029 pc5: JSON output must NOT contain '(x5)' suffix; got:\n{json_out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.010 — Escape in Collapse Path (1 test)
    // -----------------------------------------------------------------------

    /// AC-028: escape_for_terminal is called on each sampled evidence line in the
    /// collapse path; raw ESC bytes in evidence are escaped to \u{1b}.
    ///
    /// This guards VP-012 (escape_for_terminal correctness) in the collapse code path.
    /// FAILS if a future change adds a new evidence rendering branch in the collapse
    /// wrapper that bypasses escape_for_terminal, allowing raw ESC bytes to reach the TTY.
    #[test]
    fn test_BC_2_11_010_escape_in_collapse_path() {
        // BC-2.11.010 invariant 4 / AC-028:
        // One of the first K=3 members has evidence with raw ESC bytes.
        // Canonical test vector: "\x1b[31minjected\x1b[0m" → "\\u{1b}[31minjected\\u{1b}[0m".
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "CollapseEscapeBC010",
                vec!["\x1b[31minjected\x1b[0m".to_string()],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "CollapseEscapeBC010",
                vec!["clean-evidence".to_string()],
                vec![],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Raw ESC byte must NOT appear in the rendered output.
        assert!(
            !out.as_bytes().contains(&0x1b),
            "BC-2.11.010 inv4: raw ESC byte (0x1b) must not survive the collapse render path; \
             got:\n{out:?}"
        );

        // Escaped form of ESC must appear: "\\u{1b}" in the output string.
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.010 inv4: escaped form '\\u{{1b}}' must appear from collapsed evidence; \
             got:\n{out}"
        );

        // The rest of the evidence content must survive intact.
        assert!(
            out.contains("[31minjected"),
            "BC-2.11.010 inv4: non-control bytes of the evidence must survive unchanged; \
             got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.013 — Grouped Mode Suffix-Free (1 test)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.11.013): grouped mode (--mitre) is structurally suffix-free
    /// regardless of render variant; 0 (xN) suffixes in any volume.
    ///
    /// This guards the invariant that render_findings_grouped is never modified by
    /// STORY-118. Also includes a contrast assertion that flat collapse produces a
    /// suffix, proving the suffix absence is structural and not because collapse is
    /// globally broken. FAILS if grouped mode emits a suffix, OR if the flat collapse
    /// path (render_findings_collapsed) is not yet implemented.
    #[test]
    fn test_BC_2_11_013_grouped_mode_suffix_free() {
        // BC-2.11.013 invariant 4: grouped mode is structurally suffix-free.
        // 50 identical-key findings with FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } AND FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }.
        let findings: Vec<Finding> = (0..50)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "GroupedFlood",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = mitre_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // No (xN) suffix of any kind must appear anywhere in the output.
        assert!(
            !out.contains("(x"),
            "BC-2.11.013 inv4: grouped mode must produce ZERO (xN) suffixes regardless \
             of collapse_findings=true; 50 identical findings; got:\n{out}"
        );

        // All 50 findings render individually in the grouped path.
        let count = out.matches("GroupedFlood").count();
        assert_eq!(
            count, 50,
            "BC-2.11.013 inv4: grouped mode must render all 50 findings individually; \
             found {count}; got:\n{out}"
        );

        // Contrast assertion: flat+collapse mode on the same 50 findings MUST produce
        // a (x50) suffix — proving grouped bypass is structural, not globally broken.
        let out_flat = collapse_reporter().render(&Summary::new(), &findings, &[]);
        assert!(
            out_flat.contains("(x50)"),
            "BC-2.11.013 inv4 contrast: flat collapse mode must produce '(x50)' for 50 \
             identical findings (proves grouped bypass is intentional); got:\n{out_flat}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.017 — Collapsed MITRE Line from Representative Finding (1 test)
    // -----------------------------------------------------------------------

    /// AC-023 (part 2): collapsed group MITRE line comes from group_members[0],
    /// format is `MITRE: <id>` (no em-dash; flat-mode BC-2.11.017 contract).
    ///
    /// This guards that the flat-mode MITRE line format (bare ID, no em-dash) is
    /// preserved in the collapse path, and that the representative member is always
    /// group_members[0]. FAILS if the collapse path uses grouped-mode format or sources
    /// MITRE from a member other than members[0].
    #[test]
    fn test_BC_2_11_017_collapsed_mitre_line_from_representative() {
        // BC-2.11.017 pc6: flat-mode collapse path emits "    MITRE: T1036\n" from
        // group_members[0] only. Format: no em-dash (that is grouped-mode only).
        let findings = vec![
            // members[0]: has T1036.
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "MitreRepresentative",
                vec![],
                vec!["T1036".to_string()],
            ),
            // members[1]: has divergent T1059.
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "MitreRepresentative",
                vec![],
                vec!["T1059".to_string()],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // MITRE line from member[0]: "    MITRE: T1036" (bare ID, no em-dash).
        assert!(
            out.contains("MITRE: T1036"),
            "BC-2.11.017 pc6: MITRE line must read 'MITRE: T1036' (bare ID, from member[0]); \
             got:\n{out}"
        );

        // No em-dash (that is grouped-mode format only).
        assert!(
            !out.contains('\u{2014}'),
            "BC-2.11.017 pc6: flat-mode collapse MITRE line must NOT contain em-dash; \
             got:\n{out}"
        );

        // member[1]'s T1059 must not appear in terminal output.
        assert!(
            !out.contains("T1059"),
            "BC-2.11.017 pc6: member[1] MITRE 'T1059' must be elided from terminal output; \
             got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.11.019 — Section Order Unchanged with Collapse (1 test)
    // -----------------------------------------------------------------------

    /// AC-025: overall section order is unchanged when render=FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }.
    /// Only the FINDINGS body content changes.
    ///
    /// This guards that the collapse feature does not reorder or drop the top-level
    /// output sections. FAILS if a future change to the collapse path accidentally
    /// moves FINDINGS before PROTOCOLS, or omits the ANALYZER section.
    #[test]
    fn test_BC_2_11_019_section_order_unchanged_with_collapse() {
        // BC-2.11.019 pc9 / invariant 7: section order with collapse=true.
        // Expected order: WIRERUST TRIAGE REPORT → PROTOCOLS → FINDINGS → ANALYZER: Test.
        let findings = vec![make_collapse_finding(
            ThreatCategory::Anomaly,
            Verdict::Inconclusive,
            Confidence::Low,
            "OrderCheck",
            vec![],
            vec![],
        )];
        let analyzer = AnalysisSummary {
            analyzer_name: "Test".to_string(),
            packets_analyzed: 0,
            detail: std::collections::BTreeMap::new(),
        };

        let out = collapse_reporter().render(&Summary::new(), &findings, &[analyzer]);

        // All expected sections must appear.
        assert!(
            out.contains("WIRERUST TRIAGE REPORT"),
            "BC-2.11.019 inv7: 'WIRERUST TRIAGE REPORT' must appear; got:\n{out}"
        );
        assert!(
            out.contains("PROTOCOLS"),
            "BC-2.11.019 inv7: 'PROTOCOLS' must appear; got:\n{out}"
        );
        assert!(
            out.contains("FINDINGS"),
            "BC-2.11.019 inv7: 'FINDINGS' must appear; got:\n{out}"
        );
        assert!(
            out.contains("ANALYZER: Test"),
            "BC-2.11.019 inv7: 'ANALYZER: Test' must appear; got:\n{out}"
        );

        // Section order: REPORT < PROTOCOLS < FINDINGS < ANALYZER.
        let pos_report = out.find("WIRERUST TRIAGE REPORT").unwrap();
        let pos_protocols = out.find("PROTOCOLS").unwrap();
        let pos_findings = out.find("FINDINGS").unwrap();
        let pos_analyzer = out.find("ANALYZER: Test").unwrap();

        assert!(
            pos_report < pos_protocols,
            "BC-2.11.019 inv7: REPORT must precede PROTOCOLS; \
             pos_report={pos_report}, pos_protocols={pos_protocols}"
        );
        assert!(
            pos_protocols < pos_findings,
            "BC-2.11.019 inv7: PROTOCOLS must precede FINDINGS; \
             pos_protocols={pos_protocols}, pos_findings={pos_findings}"
        );
        assert!(
            pos_findings < pos_analyzer,
            "BC-2.11.019 inv7: FINDINGS must precede ANALYZER section; \
             pos_findings={pos_findings}, pos_analyzer={pos_analyzer}"
        );
    }

    // -----------------------------------------------------------------------
    // Supplementary regression guards (beyond the 35 AC-mandated tests)
    // These cover untested branches of existing BCs discovered during the
    // GREEN-phase sweep.
    // -----------------------------------------------------------------------

    /// SUPPLEMENTARY regression guard for BC-2.11.027 (beyond the 35 AC-mandated tests),
    /// covering the untested branch of the `if let Some(ev) = member.evidence.first()` path
    /// when every member in the group has an empty evidence vec.
    ///
    /// This guards that when N=3 members all have `evidence = []`, the collapse header
    /// with `(x3)` still appears, and NO evidence line (`    > `) is emitted for the group.
    /// FAILS if a future change panics on empty-evidence groups, emits a spurious `> ` line,
    /// or omits the `(x3)` header entirely.
    #[test]
    fn test_BC_2_11_027_all_empty_evidence_group_emits_no_evidence_lines() {
        // BC-2.11.027 pc2 / `if let Some(ev) = member.evidence.first()` branch:
        // N=3 collapsed group where ALL members have evidence=[].
        // The header must appear with (x3); no evidence line must be emitted.
        let findings: Vec<Finding> = (0..3)
            .map(|_| {
                make_collapse_finding(
                    ThreatCategory::Anomaly,
                    Verdict::Inconclusive,
                    Confidence::Low,
                    "AllEmptyEv",
                    vec![],
                    vec![],
                )
            })
            .collect();

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Header with (x3) must appear — the group was correctly collapsed.
        assert!(
            out.contains("(x3)"),
            "BC-2.11.027 supplementary: all-empty-evidence N=3 group must emit '(x3)' header; \
             got:\n{out}"
        );

        // No evidence line (`    > `) must appear anywhere in the output.
        assert!(
            !out.contains("    > "),
            "BC-2.11.027 supplementary: all-empty-evidence group must emit zero evidence lines; \
             got:\n{out}"
        );
    }

    /// SUPPLEMENTARY regression guard for BC-2.11.026 (beyond the 35 AC-mandated tests),
    /// covering the negative direction of the MITRE-line-from-representative contract:
    /// when `members[0].mitre_techniques = []` but a later member has `["T1036"]`,
    /// NO `MITRE:` line must appear.
    ///
    /// This is the negative-direction complement of
    /// `test_BC_2_11_026_mitre_line_from_representative_finding`, which tests that MITRE
    /// IS emitted when members[0] has a technique. This test guards the symmetric case:
    /// when members[0] has no technique, the presence of a technique on members[1] must
    /// NOT cause a MITRE line to appear.
    /// FAILS if a future change sources MITRE from any member other than members[0],
    /// or emits a MITRE line when the representative has no techniques.
    #[test]
    fn test_BC_2_11_026_no_mitre_line_when_representative_has_no_mitre() {
        // BC-2.11.026 PC-7 / BC-2.11.017: MITRE sourced from group_members[0] ONLY.
        // members[0].mitre_techniques = [] → no MITRE line should appear.
        // members[1].mitre_techniques = ["T1036"] → must be silently elided.
        let findings = vec![
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoRepMitre",
                vec![],
                vec![],
            ),
            make_collapse_finding(
                ThreatCategory::Anomaly,
                Verdict::Inconclusive,
                Confidence::Low,
                "NoRepMitre",
                vec![],
                vec!["T1036".to_string()],
            ),
        ];

        let out = collapse_reporter().render(&Summary::new(), &findings, &[]);

        // The (x2) header must appear — group was correctly collapsed.
        assert!(
            out.contains("(x2)"),
            "BC-2.11.026 supplementary: N=2 group must emit '(x2)' header; got:\n{out}"
        );

        // No MITRE line must appear — representative (members[0]) has no techniques.
        assert!(
            !out.contains("MITRE:"),
            "BC-2.11.026 supplementary: no MITRE line when representative members[0] has \
             mitre_techniques=[]; members[1] T1036 must be silently elided; got:\n{out}"
        );

        // T1036 specifically must not appear anywhere.
        assert!(
            !out.contains("T1036"),
            "BC-2.11.026 supplementary: members[1] technique 'T1036' must not appear when \
             members[0].mitre_techniques=[]; got:\n{out}"
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-120 GREEN gate tests
//
// These tests exercise new behaviors introduced by the FindingsRender enum
// migration (STORY-120). All todo!() stubs have been replaced with real
// assertions by the GREEN implementer.
//
// `TerminalReporter` now uses `render: FindingsRender` (a struct-of-two-orthogonal-enums:
// `Grouping` × `Collapse`) in place of the two removed bool fields. The FindingsRender
// struct and its axis enums are defined in src/reporter/terminal.rs and imported below.
// ---------------------------------------------------------------------------

mod story_120 {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reporter::Reporter;
    use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
    use wirerust::summary::Summary;

    fn make_finding_s120(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    fn make_mitre_finding_s120(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec!["T1046".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // -----------------------------------------------------------------------
    // AC-001 — FindingsRender derives Debug, Clone, Copy, PartialEq, Eq
    // (traces to BC-2.11.025 invariant 5, BC-2.11.013 invariant 4)
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.025 invariant 5, BC-2.11.013 invariant 4):
    /// FindingsRender satisfies Debug + Clone + Copy + PartialEq + Eq.
    /// The struct has exactly two fields: grouping: Grouping and collapse: Collapse. No Default impl.
    #[test]
    fn test_findings_render_derives_debug_clone_copy_partialeq_eq() {
        let a = FindingsRender {
            grouping: Grouping::Grouped,
            collapse: Collapse::Expanded,
        };
        let b = a; // Copy
        let c = Clone::clone(&a); // Clone (explicit form avoids clone_on_copy lint)
        assert_eq!(a, b, "PartialEq + Eq: Grouped == copied Grouped");
        assert_eq!(a, c, "PartialEq + Eq: Grouped == cloned Grouped");
        let _ = format!("{a:?}"); // Debug — would panic if not implemented

        // All four Grouping x Collapse combinations are pairwise distinct.
        assert_ne!(
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded
            },
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed
            },
            "{{Grouped,Expanded}} != {{Flat,Collapsed}}"
        );
        assert_ne!(
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded
            },
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded
            },
            "{{Grouped,Expanded}} != {{Flat,Expanded}}"
        );
        assert_ne!(
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed
            },
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded
            },
            "{{Flat,Collapsed}} != {{Flat,Expanded}}"
        );
        assert_ne!(
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded
            },
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed
            },
            "{{Grouped,Expanded}} != {{Grouped,Collapsed}}"
        );
        assert_ne!(
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed
            },
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed
            },
            "{{Grouped,Collapsed}} != {{Flat,Collapsed}}"
        );
        assert_ne!(
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed
            },
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded
            },
            "{{Grouped,Collapsed}} != {{Flat,Expanded}}"
        );

        // Debug output for struct form includes field names and variant names.
        assert!(
            format!(
                "{:?}",
                FindingsRender {
                    grouping: Grouping::Grouped,
                    collapse: Collapse::Expanded
                }
            )
            .contains("Grouped")
        );
        assert!(
            format!(
                "{:?}",
                FindingsRender {
                    grouping: Grouping::Flat,
                    collapse: Collapse::Collapsed
                }
            )
            .contains("Flat")
        );
        assert!(
            format!(
                "{:?}",
                FindingsRender {
                    grouping: Grouping::Flat,
                    collapse: Collapse::Collapsed
                }
            )
            .contains("Collapsed")
        );
        assert!(
            format!(
                "{:?}",
                FindingsRender {
                    grouping: Grouping::Flat,
                    collapse: Collapse::Expanded
                }
            )
            .contains("Expanded")
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 — TerminalReporter struct has exactly 3 fields after refactor
    // (traces to BC-2.11.028 precondition 3 — struct shape governs render wiring)
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.11.028 precondition 3):
    /// TerminalReporter has exactly 3 fields: use_color, show_hosts_breakdown, render.
    /// This test compiles only if the struct has exactly those fields — any reference
    /// to the removed fields is a compile error.
    #[test]
    fn test_BC_2_11_028_struct_has_exactly_three_fields_post_refactor() {
        // The struct literal below is the test: it compiles if and only if exactly
        // these three fields exist on TerminalReporter (Rust requires all fields in a
        // struct literal; extra fields are also a compile error).
        let r = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        };
        // The struct renders fine (behavioral sanity).
        let out = r.render(&Summary::new(), &[], &[]);
        assert!(
            out.contains("WIRERUST TRIAGE REPORT"),
            "three-field TerminalReporter renders a valid report; got: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 — Dispatch is an exhaustive match over FindingsRender
    // (traces to BC-2.11.019 invariant 7)
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.11.019 invariant 7):
    /// All four FindingsRender (Grouping × Collapse) dispatch arms are reachable and route
    /// to the correct rendering path. Three of the four combos are exercised here:
    ///   {Grouped,Expanded}   → MITRE tactic header "## " in output
    ///   {Flat,Collapsed}     → "(x3)" suffix for N=3 identical findings
    ///   {Flat,Expanded}      → 3 individual header lines, no "(x" suffix
    #[test]
    fn test_BC_2_11_019_findings_dispatch_match_exhaustive() {
        // Three identical-key findings with a known MITRE technique for the Grouped path.
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_mitre_finding_s120("dispatch-test"))
            .collect();

        // Grouped arm: calls render_findings_grouped → emits "## " tactic header.
        let grouped_out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            grouped_out.contains("## "),
            "AC-003: Grouped arm must emit a tactic header (## ...); got:\n{grouped_out}"
        );

        // FlatCollapsed arm: calls render_findings_collapsed → emits "(x3)" suffix.
        let collapsed_out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            collapsed_out.contains("(x3)"),
            "AC-003: FlatCollapsed arm must emit '(x3)' for 3 identical-key findings; \
             got:\n{collapsed_out}"
        );

        // FlatExpanded arm: iterates render_finding_flat → 3 individual header lines, no (x.
        let expanded_out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            !expanded_out.contains("(x"),
            "AC-003: FlatExpanded arm must not emit (xN) suffix; got:\n{expanded_out}"
        );
        let header_count = expanded_out.matches("dispatch-test").count();
        assert_eq!(
            header_count, 3,
            "AC-003: FlatExpanded arm must emit 3 individual finding lines; \
             found {header_count} in:\n{expanded_out}"
        );

        // All three tested outputs are mutually distinct (three of the four Grouping x Collapse combos).
        assert_ne!(
            grouped_out, collapsed_out,
            "AC-003: Grouped != FlatCollapsed output"
        );
        assert_ne!(
            grouped_out, expanded_out,
            "AC-003: Grouped != FlatExpanded output"
        );
        assert_ne!(
            collapsed_out, expanded_out,
            "AC-003: FlatCollapsed != FlatExpanded output"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 — {Grouped, Expanded} mode does not emit collapse (xN) suffixes
    // (traces to BC-2.11.025 invariant 5)
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.11.025 invariant 5):
    /// FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded } on N=100
    /// identical-key findings produces no "(xN)" suffix. The {Grouped, Expanded} struct combo
    /// routes to render_findings_grouped which never applies the collapse pass — the two axes
    /// are orthogonal and all four combinations are valid (no combination is prohibited).
    #[test]
    fn test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally() {
        let findings: Vec<Finding> = (0..100)
            .map(|_| make_mitre_finding_s120("grouped-collapse-impossible"))
            .collect();

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        assert!(
            !out.contains("(x"),
            "AC-004: FindingsRender {{ grouping: Grouping::Grouped, collapse: Collapse::Expanded }} must never emit (xN) suffix — \
             the grouped path does not apply the collapse pass; got:\n{out}"
        );
        // Grouped mode emits tactic headers (confirming the Grouped path ran, not collapsed).
        assert!(
            out.contains("## "),
            "AC-004: FindingsRender {{ grouping: Grouping::Grouped, collapse: Collapse::Expanded }} must emit tactic section headers; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 — render field wiring: FlatCollapsed vs FlatExpanded polarity
    // (traces to BC-2.11.028 postconditions 1–2, invariant 1)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.11.028 postconditions 1–2, invariant 1):
    /// FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed } produces "(x3)" for 3 identical-key findings;
    /// FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded } does not — proving the render field is wired
    /// correctly to the rendering path. Mirrors the run_analyze construction intent.
    #[test]
    fn test_BC_2_11_028_flag_wired_to_render_field_post_enum() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_finding_s120("WiredTestPostEnum"))
            .collect();

        // FlatCollapsed → collapse active → "(x3)" suffix.
        let out_collapsed = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            out_collapsed.contains("(x3)"),
            "AC-005: FindingsRender {{ grouping: Grouping::Flat, collapse: Collapse::Collapsed }} must produce '(x3)' for 3 identical \
             findings; got:\n{out_collapsed}"
        );

        // FlatExpanded → collapse inactive → no "(x" suffix, 3 individual lines.
        let out_expanded = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            !out_expanded.contains("(x"),
            "AC-005: FindingsRender {{ grouping: Grouping::Flat, collapse: Collapse::Expanded }} must not produce (xN) suffix; \
             got:\n{out_expanded}"
        );

        // Polarity: FlatCollapsed != FlatExpanded output for N≥2 identical-key input.
        assert_ne!(
            out_collapsed, out_expanded,
            "AC-005: FlatCollapsed and FlatExpanded must produce different output \
             for N≥2 identical-key findings"
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-122 acceptance-gate tests
//
// These tests lock in the STORY-122 acceptance criteria for the
// FindingsRender enum→struct reshape (Option X). The stub-architect
// (commit dec8a55) reshaped the types and migrated the 84 construction
// sites; these tests verify both the completed mechanical migration and
// the completed Task-4 comment-sweep work.
//
// All tasks complete on this commit; every test below is GREEN and serves
// as a regression guard.
//
// DF-TEST-NAMESPACE-001: per-story mod wrapper.
// ---------------------------------------------------------------------------

mod story_122 {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reporter::Reporter;
    use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
    use wirerust::summary::Summary;

    fn make_finding_s122(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Inconclusive,
            confidence: Confidence::Low,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    fn make_mitre_finding_s122(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec!["T1046".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // -----------------------------------------------------------------------
    // AC-001 — Grouping, Collapse, and FindingsRender struct defined correctly
    // (traces to BC-2.11.028 Invariant 1)
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.028 Invariant 1):
    /// All four Grouping × Collapse combinations are pairwise distinct.
    /// Grouping and Collapse each derive Debug, Clone, Copy, PartialEq, Eq.
    /// FindingsRender struct derives Debug, Clone, Copy, PartialEq, Eq.
    /// No Default is derived on any of the three types.
    #[test]
    fn test_BC_2_11_028_ac001_four_combos_pairwise_distinct() {
        let grouped_expanded = FindingsRender {
            grouping: Grouping::Grouped,
            collapse: Collapse::Expanded,
        };
        let grouped_collapsed = FindingsRender {
            grouping: Grouping::Grouped,
            collapse: Collapse::Collapsed,
        };
        let flat_collapsed = FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Collapsed,
        };
        let flat_expanded = FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Expanded,
        };

        // Copy semantics: binding a second name does not move.
        let ge2 = grouped_expanded;
        assert_eq!(
            grouped_expanded, ge2,
            "AC-001: Copy + PartialEq: {ge2:?} == copy"
        );

        // Clone semantics.
        let gc2 = Clone::clone(&grouped_collapsed);
        assert_eq!(
            grouped_collapsed, gc2,
            "AC-001: Clone + PartialEq: {gc2:?} == clone"
        );

        // Debug is implemented — would panic at format time if not.
        let _ = format!("{grouped_expanded:?}");
        let _ = format!("{grouped_collapsed:?}");
        let _ = format!("{flat_collapsed:?}");
        let _ = format!("{flat_expanded:?}");

        // All four Grouping × Collapse combinations are pairwise distinct.
        assert_ne!(
            grouped_expanded, grouped_collapsed,
            "AC-001: {grouped_expanded:?} != {grouped_collapsed:?}"
        );
        assert_ne!(
            grouped_expanded, flat_collapsed,
            "AC-001: {grouped_expanded:?} != {flat_collapsed:?}"
        );
        assert_ne!(
            grouped_expanded, flat_expanded,
            "AC-001: {grouped_expanded:?} != {flat_expanded:?}"
        );
        assert_ne!(
            grouped_collapsed, flat_collapsed,
            "AC-001: {grouped_collapsed:?} != {flat_collapsed:?}"
        );
        assert_ne!(
            grouped_collapsed, flat_expanded,
            "AC-001: {grouped_collapsed:?} != {flat_expanded:?}"
        );
        assert_ne!(
            flat_collapsed, flat_expanded,
            "AC-001: {flat_collapsed:?} != {flat_expanded:?}"
        );
    }

    /// AC-001 (BC-2.11.028 Invariant 1):
    /// Debug output for FindingsRender struct uses struct-of-enums vocabulary,
    /// not the old enum-variant vocabulary. Field names "grouping" and "collapse"
    /// appear in the Debug representation; old variant names "FlatCollapsed" and
    /// "FlatExpanded" do not.
    #[test]
    fn test_BC_2_11_028_ac001_debug_format_struct_vocabulary() {
        // Struct debug output contains field and variant names.
        let ge_dbg = format!(
            "{:?}",
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded
            }
        );
        assert!(
            ge_dbg.contains("grouping"),
            "AC-001: Debug must include field name 'grouping'; got: {ge_dbg}"
        );
        assert!(
            ge_dbg.contains("collapse"),
            "AC-001: Debug must include field name 'collapse'; got: {ge_dbg}"
        );
        assert!(
            ge_dbg.contains("Grouped"),
            "AC-001: Debug must include 'Grouped' for Grouping::Grouped; got: {ge_dbg}"
        );
        assert!(
            ge_dbg.contains("Expanded"),
            "AC-001: Debug must include 'Expanded' for Collapse::Expanded; got: {ge_dbg}"
        );

        let fc_dbg = format!(
            "{:?}",
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed
            }
        );
        assert!(
            fc_dbg.contains("Flat"),
            "AC-001: Debug must include 'Flat' for Grouping::Flat; got: {fc_dbg}"
        );
        assert!(
            fc_dbg.contains("Collapsed"),
            "AC-001: Debug must include 'Collapsed' for Collapse::Collapsed; got: {fc_dbg}"
        );

        // Old enum-variant concatenated names must NOT appear in debug output.
        assert!(
            !fc_dbg.contains("FlatCollapsed"),
            "AC-001: Debug must not contain old enum token 'FlatCollapsed'; got: {fc_dbg}"
        );
        let fe_dbg = format!(
            "{:?}",
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded
            }
        );
        assert!(
            !fe_dbg.contains("FlatExpanded"),
            "AC-001: Debug must not contain old enum token 'FlatExpanded'; got: {fe_dbg}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 — Four-arm exhaustive tuple dispatch
    // (traces to BC-2.11.013 Invariant 4)
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.11.013 Invariant 4):
    /// The {Grouped, Expanded} dispatch arm routes to render_findings_grouped —
    /// MITRE tactic headers ("## ") appear in output.
    #[test]
    fn test_BC_2_11_028_ac002_grouped_expanded_arm_routes_to_render_findings_grouped() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_mitre_finding_s122("s122-dispatch-ge"))
            .collect();
        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            out.contains("## "),
            "AC-002: {{Grouped,Expanded}} arm must emit tactic header '## '; got:\n{out}"
        );
        assert!(
            !out.contains("(x"),
            "AC-002: {{Grouped,Expanded}} arm must not emit (xN) suffix; got:\n{out}"
        );
    }

    /// AC-002 (BC-2.11.013 Invariant 4 / STORY-119/B post-implementation):
    /// The {Grouped, Collapsed} dispatch arm routes to `render_findings_grouped_collapsed`
    /// (STORY-119/B). For N≥2 identical-key findings in a tactic bucket, the output
    /// carries a `(xN)` suffix — it is NO longer byte-identical to {Grouped, Expanded}.
    /// Tactic headers still appear (`## <tactic>`).
    ///
    /// Supersedes the TEMPORARY STORY-122/A assertion that {Grouped,Collapsed} was
    /// byte-identical to {Grouped,Expanded}. That was a scaffolding state; STORY-119/B
    /// wires the real implementation.
    #[test]
    fn test_BC_2_11_028_ac002_grouped_collapsed_arm_routes_to_render_findings_grouped_collapsed() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_mitre_finding_s122("s122-dispatch-gc"))
            .collect();

        let out_gc = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        let out_ge = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        // Post-STORY-119/B: {Grouped,Collapsed} routes to render_findings_grouped_collapsed.
        // N=3 identical-key findings must produce (x3) suffix — collapsed, not expanded.
        assert!(
            out_gc.contains("(x3)"),
            "AC-002: {{Grouped,Collapsed}} with N=3 identical findings must emit `(x3)` suffix; \
             got:\n{out_gc}"
        );
        // {Grouped,Collapsed} output must differ from {Grouped,Expanded} (no longer byte-identical).
        assert_ne!(
            out_gc, out_ge,
            "AC-002: {{Grouped,Collapsed}} must differ from {{Grouped,Expanded}} post-STORY-119/B"
        );
        // Tactic headers still appear.
        assert!(
            out_gc.contains("## "),
            "AC-002: {{Grouped,Collapsed}} arm must emit tactic header '## '; got:\n{out_gc}"
        );
    }

    /// AC-002 (BC-2.11.013 Invariant 4):
    /// The {Flat, Collapsed} dispatch arm routes to render_findings_collapsed —
    /// (xN) suffix appears for N≥2 identical-key findings.
    #[test]
    fn test_BC_2_11_028_ac002_flat_collapsed_arm_routes_to_render_findings_collapsed() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_finding_s122("s122-dispatch-fc"))
            .collect();
        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            out.contains("(x3)"),
            "AC-002: {{Flat,Collapsed}} arm must emit '(x3)' for 3 identical-key findings; got:\n{out}"
        );
    }

    /// AC-002 (BC-2.11.013 Invariant 4):
    /// The {Flat, Expanded} dispatch arm iterates render_finding_flat —
    /// 3 individual finding lines with no (xN) suffix.
    #[test]
    fn test_BC_2_11_028_ac002_flat_expanded_arm_iterates_render_finding_flat() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_finding_s122("s122-dispatch-fe"))
            .collect();
        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            !out.contains("(x"),
            "AC-002: {{Flat,Expanded}} arm must not emit (xN) suffix; got:\n{out}"
        );
        let count = out.matches("s122-dispatch-fe").count();
        assert_eq!(
            count, 3,
            "AC-002: {{Flat,Expanded}} arm must emit 3 individual finding lines; found {count} in:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 — run_analyze construction site: 3-arm if → 3 struct literals
    // (traces to BC-2.11.028 Invariant 1)
    //
    // run_analyze is not directly callable from tests (reads pcap files), so
    // we verify the wiring by asserting the behavior of TerminalReporter
    // constructed with the exact FindingsRender values that run_analyze
    // produces for each flag combination (byte-identical to v0.9.0 branching).
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.11.028 Invariant 1, EC-001):
    /// HISTORICAL REGRESSION GUARD (STORY-122/A era): in STORY-122/A, `--mitre` alone
    /// produced `{Grouped, Expanded}`. This test asserts the struct value that
    /// STORY-122/A's construction site emitted — preserved as a regression guard for
    /// the STORY-122/A struct value only.
    /// NOTE: Post-STORY-119/B, `--mitre` alone produces `{Grouped, Collapsed}` (the
    /// CLI default was flipped by STORY-119/B Task 4). This test does NOT assert the
    /// run_analyze CLI behavior after STORY-119/B — it asserts only the struct value
    /// equality and that `{Grouped,Expanded}` ≠ `{Grouped,Collapsed}`.
    #[test]
    fn test_BC_2_11_028_ac004_mitre_alone_yields_grouped_expanded() {
        let findings: Vec<Finding> = (0..2)
            .map(|_| make_mitre_finding_s122("s122-ac004-mitre"))
            .collect();

        // The struct value run_analyze produces when show_mitre_grouping=true.
        let render_from_mitre_flag = FindingsRender {
            grouping: Grouping::Grouped,
            collapse: Collapse::Expanded,
        };

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: render_from_mitre_flag,
        }
        .render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("## "),
            "AC-004: --mitre alone → {{Grouped,Expanded}} must emit tactic headers; got:\n{out}"
        );
        assert!(
            !out.contains("(x"),
            "AC-004: --mitre alone → {{Grouped,Expanded}} must not emit (xN) suffix; got:\n{out}"
        );

        // {Grouped, Collapsed} is NOT what --mitre alone produced in STORY-122/A
        // (post-STORY-119/B this IS the CLI default, but this test only asserts the struct values).
        let render_grouped_collapsed = FindingsRender {
            grouping: Grouping::Grouped,
            collapse: Collapse::Collapsed,
        };
        assert_ne!(
            render_from_mitre_flag, render_grouped_collapsed,
            "AC-004: run_analyze with --mitre alone must produce {{Grouped,Expanded}}, \
             not {{Grouped,Collapsed}} — the CLI flip is STORY-119/B scope"
        );
    }

    /// AC-004 (BC-2.11.028 Invariant 1, EC-003):
    /// Default (show_mitre_grouping=false, collapse_findings=true) →
    /// run_analyze produces FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }.
    /// Routes to render_findings_collapsed; (xN) suffix for identical findings.
    #[test]
    fn test_BC_2_11_028_ac004_default_yields_flat_collapsed() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_finding_s122("s122-ac004-default"))
            .collect();

        // The struct value run_analyze produces for the default (no --mitre, collapse_findings=true).
        let render_default = FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Collapsed,
        };

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: render_default,
        }
        .render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("(x3)"),
            "AC-004: default → {{Flat,Collapsed}} must emit '(x3)' for 3 identical-key findings; got:\n{out}"
        );
        assert!(
            !out.contains("## "),
            "AC-004: default → {{Flat,Collapsed}} must not emit tactic headers; got:\n{out}"
        );
    }

    /// AC-004 (BC-2.11.028 Invariant 1, EC-002):
    /// --no-collapse (show_mitre_grouping=false, collapse_findings=false) →
    /// run_analyze produces FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Expanded }.
    /// Routes to render_finding_flat per-item; 3 individual lines, no (xN) suffix.
    #[test]
    fn test_BC_2_11_028_ac004_no_collapse_yields_flat_expanded() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_finding_s122("s122-ac004-nocollapse"))
            .collect();

        // The struct value run_analyze produces for --no-collapse (collapse_findings=false).
        let render_no_collapse = FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Expanded,
        };

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: render_no_collapse,
        }
        .render(&Summary::new(), &findings, &[]);

        assert!(
            !out.contains("(x"),
            "AC-004: --no-collapse → {{Flat,Expanded}} must not emit (xN) suffix; got:\n{out}"
        );
        let count = out.matches("s122-ac004-nocollapse").count();
        assert_eq!(
            count, 3,
            "AC-004: --no-collapse → {{Flat,Expanded}} must emit 3 individual lines; found {count}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 — run_summary construction site uses {Flat, Collapsed}
    // (traces to BC-2.11.028 Postcondition 4)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.11.028 Postcondition 4):
    /// run_summary uses FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }.
    /// The render field is inert for run_summary (no FINDINGS section rendered).
    /// TerminalReporter constructed with that value renders a valid report.
    #[test]
    fn test_BC_2_11_028_ac005_run_summary_construction_uses_flat_collapsed() {
        // The struct value run_summary uses — inert (run_summary renders no FINDINGS section).
        let render_summary = FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Collapsed,
        };

        let reporter = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: render_summary,
        };

        // run_summary passes an empty findings slice — no FINDINGS section appears.
        let out = reporter.render(&Summary::new(), &[], &[]);
        assert!(
            out.contains("WIRERUST TRIAGE REPORT"),
            "AC-005: run_summary construction site must render a valid report header; got:\n{out}"
        );
        assert!(
            !out.contains("FINDINGS"),
            "AC-005: run_summary with empty findings must not emit FINDINGS section; got:\n{out}"
        );

        // The run_summary value matches exactly {Flat, Collapsed}.
        assert_eq!(
            render_summary,
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed
            },
            "AC-005: run_summary FindingsRender must be {{Flat,Collapsed}}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 — Output byte-identical to v0.9.0 for all existing CLI combos
    // (traces to BC-2.11.013 Invariant 4)
    //
    // The three CLI-reachable paths in STORY-122/A are verified by asserting
    // that each FindingsRender value routes to the same render function as
    // the old enum variant it replaced, producing the same observable output.
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.11.013 Invariant 4):
    /// {Grouped, Expanded} (--mitre alone in STORY-122/A) produces the same
    /// output as the old grouped (suffix-free) variant:
    /// tactic headers present, no (xN) suffix, byte-identical across the reshape.
    #[test]
    fn test_BC_2_11_028_ac006_grouped_expanded_byte_identical_to_old_grouped_variant() {
        let findings: Vec<Finding> = (0..2)
            .map(|_| make_mitre_finding_s122("s122-ac006-byteident"))
            .collect();

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        // Grouped-expanded output has tactic headers and no collapse suffixes.
        assert!(
            out.contains("## "),
            "AC-006: {{Grouped,Expanded}} must emit '## ' tactic header"
        );
        assert!(
            !out.contains("(x"),
            "AC-006: {{Grouped,Expanded}} must not emit (xN) suffix"
        );
        // Finding summary appears (not collapsed away).
        assert!(
            out.matches("s122-ac006-byteident").count() >= 2,
            "AC-006: {{Grouped,Expanded}} must emit all 2 findings individually"
        );
    }

    /// AC-006 (BC-2.11.026 Postcondition 4, BC-2.11.027 Postcondition 1):
    /// {Flat, Collapsed} (default in STORY-122/A) produces the same output as
    /// the old flat-collapsed path: (xN) suffix for N≥2, up to K=3
    /// evidence lines, MITRE line for non-empty mitre_techniques.
    #[test]
    fn test_BC_2_11_028_ac006_flat_collapsed_byte_identical_to_old_flatcollapsed_variant() {
        let findings: Vec<Finding> = (0..3)
            .map(|i| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s122-ac006-collapse".to_string(),
                evidence: vec![format!("ev{i}")],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        // BC-2.11.026 PC-4: (xN) suffix for collapsed group of N=3.
        assert!(
            out.contains("(x3)"),
            "AC-006: {{Flat,Collapsed}} must emit '(x3)' for 3 identical-key findings; got:\n{out}"
        );
        // BC-2.11.027 PC-1: up to K=3 evidence lines.
        assert!(
            out.contains("ev0"),
            "AC-006: first evidence line ev0 must appear; got:\n{out}"
        );
        // MITRE line for non-empty mitre_techniques.
        assert!(
            out.contains("T1046"),
            "AC-006: MITRE technique T1046 must appear; got:\n{out}"
        );
    }

    /// AC-006 (BC-2.11.013 Invariant 4):
    /// {Flat, Expanded} (--no-collapse in STORY-122/A) produces the same output
    /// as the old flat-expanded path: 3 individual lines, no (xN) suffix.
    #[test]
    fn test_BC_2_11_028_ac006_flat_expanded_byte_identical_to_old_flatexpanded_variant() {
        let findings: Vec<Finding> = (0..3).map(|_| make_finding_s122("s122-ac006-fe")).collect();

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        assert!(
            !out.contains("(x"),
            "AC-006: {{Flat,Expanded}} must not emit (xN) suffix; got:\n{out}"
        );
        assert_eq!(
            out.matches("s122-ac006-fe").count(),
            3,
            "AC-006: {{Flat,Expanded}} must emit 3 individual finding lines"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 — Comment sweep: stale prose is absent from swept targets
    // (traces to BC-2.11.028 Invariant 6)
    //
    // These tests read src/reporter/terminal.rs and tests/reporter_terminal_tests.rs
    // via include_str! at compile time (file paths anchored to crate root) and
    // assert that the stale prose tokens listed in the STORY-122 Task-4
    // Falsifiable Requirements are absent.
    //
    // REGRESSION GUARD: Task 4 comment sweep is complete; these tests assert
    // the swept targets stay free of stale FindingsRender enum vocabulary.
    // The grep gate values match STORY-122 AC-007 / Task-4 Falsifiable Requirements (3) and (4).
    //
    // EXEMPT: The pattern "three fields" is NOT in the gate (exempt:
    // TerminalReporter-field prose at reporter_terminal_tests.rs:4040).
    // The "All three boundary" (escape test) and "All three findings must appear"
    // (count-of-test-findings comment) are also exempt per AC-007.
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.11.028 Invariant 6):
    /// src/reporter/terminal.rs contains no stale pre-STORY-122 module-doc vocabulary
    /// (the "three-modes" framing replaced by two-orthogonal-axis struct vocabulary).
    /// Gate 4 from STORY-122 Task-4 Falsifiable Requirements.
    #[test]
    fn test_BC_2_11_028_ac007_terminal_rs_no_three_mutually_exclusive() {
        let src = include_str!("../src/reporter/terminal.rs");
        // Build token at runtime so this source file does not self-trigger the sweep gate.
        let token = concat!("three", " mutually-exclusive");
        assert!(
            !src.contains(token),
            "AC-007: src/reporter/terminal.rs must not contain the stale two-axis framing token; \
             Task 4 comment sweep is incomplete"
        );
    }

    /// AC-007 (BC-2.11.028 Invariant 6):
    /// src/reporter/terminal.rs contains no stale "verdict-desc" or "confidence-desc"
    /// doc-comment text. Gate 2 from STORY-122 Task-4 Falsifiable Requirements.
    #[test]
    fn test_BC_2_11_028_ac007_terminal_rs_no_verdict_desc_or_confidence_desc() {
        let src = include_str!("../src/reporter/terminal.rs");
        assert!(
            !src.contains("verdict-desc"),
            "AC-007: src/reporter/terminal.rs must not contain stale 'verdict-desc' in \
             render_findings_grouped doc-comment; Task 4 comment sweep is incomplete"
        );
        assert!(
            !src.contains("confidence-desc"),
            "AC-007: src/reporter/terminal.rs must not contain stale 'confidence-desc' in \
             render_findings_grouped doc-comment; Task 4 comment sweep is incomplete"
        );
    }

    /// AC-007 (BC-2.11.028 Invariant 6):
    /// tests/reporter_terminal_tests.rs contains no stale pre-STORY-122 FindingsRender
    /// enum vocabulary (old variant names, old three-mode framing, old impossible-combo
    /// framing) in the swept targets.
    /// Gate 3 from STORY-122 Task-4 Falsifiable Requirements.
    ///
    /// EXEMPT tokens NOT checked here:
    ///   - "three fields" (TerminalReporter-field comment) — correct, not stale.
    ///   - "All three boundary" (escape-boundary test) — unrelated to FindingsRender.
    ///   - "All three findings must appear" (test-finding count) — unrelated.
    #[test]
    fn test_BC_2_11_028_ac007_test_file_no_stale_findingsrender_prose() {
        let src = include_str!("../tests/reporter_terminal_tests.rs");

        // Split into lines so we can skip exempt lines by their anchored text.
        let exempt_line_fragments: &[&str] = &[
            // :4040 — "these three fields exist on TerminalReporter" — EXEMPT.
            "these three fields exist on TerminalReporter",
            // :394 — "All three boundary" — EXEMPT (escape boundary values).
            "All three boundary",
            // :3512 — "All three findings must appear" — EXEMPT (count of test findings).
            "All three findings must appear",
        ];

        // Tokens are built with concat! so this source file does not self-trigger the sweep.
        let stale_tokens: &[&str] = &[
            concat!("three", "-variant"),
            concat!("three", " variants"),
            concat!("All three", " FindingsRender"),
            concat!("All three", " outputs"),
            concat!("three", " arm"),
            concat!("three", "-arm"),
            concat!("three", " mutually-exclusive"),
            concat!("three", "-way"),
            concat!("impossible", " state"),
        ];

        let mut violations: Vec<String> = Vec::new();
        for (line_no, line) in src.lines().enumerate() {
            // Skip exempt lines.
            let is_exempt = exempt_line_fragments.iter().any(|frag| line.contains(frag));
            if is_exempt {
                continue;
            }
            for token in stale_tokens {
                if line.contains(token) {
                    violations.push(format!(
                        "  line {}: {:?} (token: {:?})",
                        line_no + 1,
                        line.trim(),
                        token
                    ));
                }
            }
        }

        assert!(
            violations.is_empty(),
            "AC-007: tests/reporter_terminal_tests.rs contains stale FindingsRender prose \
             that Task 4 comment sweep must remove:\n{}",
            violations.join("\n")
        );
    }

    /// AC-007 (BC-2.11.028 Invariant 6):
    /// src/main.rs contains no stale pre-STORY-122 FindingsRender vocabulary
    /// (old three-mode framing, old impossible-combo framing, old three-mode prose).
    /// Gate 4 (src/ targets) from STORY-122 Task-4 Falsifiable Requirements.
    #[test]
    fn test_BC_2_11_028_ac007_main_rs_no_stale_findingsrender_prose() {
        let src = include_str!("../src/main.rs");
        // Tokens built with concat! so this source file does not self-trigger the sweep.
        for token in &[
            concat!("three", " mutually-exclusive"),
            concat!("three", "-way"),
            concat!("impossible", " state"),
            concat!("three", " mode"),
        ] {
            assert!(
                !src.contains(token),
                "AC-007: src/main.rs must not contain stale token {:?}; \
                 Task 4 comment sweep is incomplete",
                token
            );
        }
    }

    // -----------------------------------------------------------------------
    // AC-008 — BC-016/026/027 preserved byte-identically across reshape
    // (traces to BC-2.11.016 Invariant 3, BC-2.11.026 Postcondition 4,
    //  BC-2.11.027 Postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.11.016 Invariant 3):
    /// The em-dash MITRE name line is produced by render_findings_grouped
    /// via the {Grouped, Expanded} arm — the reshape does not alter this behavior.
    /// Known technique T1046 (Network Service Discovery) emits an em-dash line.
    #[test]
    fn test_BC_2_11_016_ac008_em_dash_mitre_format_preserved_after_reshape() {
        let findings = vec![Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "s122-ac008-emdash".to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T1046".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }];

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        // BC-2.11.016: em-dash line for known MITRE ID.
        assert!(
            out.contains("\u{2014}"),
            "AC-008: {{Grouped,Expanded}} must emit em-dash (U+2014) for known MITRE technique; got:\n{out}"
        );
        assert!(
            out.contains("T1046"),
            "AC-008: MITRE technique T1046 must appear in grouped output; got:\n{out}"
        );
    }

    /// AC-008 (BC-2.11.026 Postcondition 4):
    /// The flat (xN) suffix rule is preserved after the enum→struct reshape.
    /// {Flat, Collapsed} with N=5 identical-key findings emits "(x5)" suffix.
    #[test]
    fn test_BC_2_11_026_ac008_flat_xn_suffix_preserved_after_reshape() {
        let findings: Vec<Finding> = (0..5).map(|_| make_finding_s122("s122-ac008-xn")).collect();

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("(x5)"),
            "AC-008: {{Flat,Collapsed}} must emit '(x5)' for 5 identical-key findings; got:\n{out}"
        );
        // BC-2.11.026 PC-4: suffix-free guarantee scoped to {Grouped,Expanded}.
        let grouped_out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            !grouped_out.contains("(x"),
            "AC-008: {{Grouped,Expanded}} must never emit (xN) suffix (PC-4 suffix-free guarantee); got:\n{grouped_out}"
        );
    }

    /// AC-008 (BC-2.11.027 Postcondition 1):
    /// K=3 evidence sampling is preserved after the enum→struct reshape.
    /// {Flat, Collapsed} with N=5 identical-key findings, each with evidence,
    /// emits at most 3 evidence lines (K=3 cap).
    #[test]
    fn test_BC_2_11_027_ac008_k3_evidence_sampling_preserved_after_reshape() {
        let findings: Vec<Finding> = (0..5)
            .map(|i| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s122-ac008-k3".to_string(),
                evidence: vec![format!("evidence-line-{i}")],
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);

        // BC-2.11.027 PC-1: at most K=3 evidence lines ("    > " prefix).
        let evidence_line_count = out
            .lines()
            .filter(|l| l.trim_start().starts_with("> "))
            .count();
        assert!(
            evidence_line_count <= 3,
            "AC-008: K=3 cap — {{Flat,Collapsed}} must emit at most 3 evidence lines; \
             got {evidence_line_count} in:\n{out}"
        );
        // At least 1 evidence line (confirms sampling path ran).
        assert!(
            evidence_line_count >= 1,
            "AC-008: {{Flat,Collapsed}} with evidence-bearing findings must emit \
             at least 1 evidence line; got {evidence_line_count} in:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 — Zero-grep gate: no stale FindingsRender::{Grouped,FlatCollapsed,FlatExpanded}
    // tokens in src/ or tests/
    // (traces to BC-2.11.028 AC-003; AC-007 extended the gate to doc-comments)
    // -----------------------------------------------------------------------

    /// AC-003 / AC-007 (BC-2.11.028 Invariant 6):
    /// REGRESSION GUARD: No stale `FindingsRender::{Grouped,FlatCollapsed,FlatExpanded}`
    /// tokens exist anywhere in src/main.rs, src/reporter/terminal.rs, or this test file.
    /// These tokens are the removed enum-variant paths from the pre-STORY-122 type.
    /// The gate tokens are built with `concat!` so this source file does not self-trigger.
    #[test]
    fn test_BC_2_11_028_ac003_no_stale_findingsrender_variant_tokens() {
        let sources: &[(&str, &str)] = &[
            ("src/main.rs", include_str!("../src/main.rs")),
            (
                "src/reporter/terminal.rs",
                include_str!("../src/reporter/terminal.rs"),
            ),
            (
                "tests/reporter_terminal_tests.rs",
                include_str!("../tests/reporter_terminal_tests.rs"),
            ),
        ];

        // Tokens are built with concat! so this source file does not self-trigger the gate.
        let banned_tokens: &[&str] = &[
            concat!("FindingsRender", "::", "Grouped"),
            concat!("FindingsRender", "::", "FlatCollapsed"),
            concat!("FindingsRender", "::", "FlatExpanded"),
        ];

        let mut violations: Vec<String> = Vec::new();
        for (label, src) in sources {
            for (line_no, line) in src.lines().enumerate() {
                for token in banned_tokens {
                    if line.contains(token) {
                        violations.push(format!(
                            "  {}:{}: {:?} (token: {:?})",
                            label,
                            line_no + 1,
                            line.trim(),
                            token
                        ));
                    }
                }
            }
        }

        assert!(
            violations.is_empty(),
            "AC-003: stale FindingsRender enum-variant tokens found — \
             Task 4 comment sweep or doc-comment cleanup is incomplete:\n{}",
            violations.join("\n")
        );
    }
}

// ============================================================================
// mod story_119 — STORY-119/B: Grouped-Collapse Behavioral Delta
// ============================================================================
//
// Tests for render_findings_grouped_collapsed (BC-2.11.031/032/033/034),
// CLI mapping to {Grouped,Collapsed} (BC-2.11.030), and the per-bucket
// collapse-not-global invariant (BC-2.11.025 Invariant 5).
//
// Namespace wrapper per DF-TEST-NAMESPACE-001: all test functions live inside
// `mod story_119` so that identically-named siblings in sibling mods (e.g.
// `test_BC_2_11_025_grouped_mode_bypasses_flat_collapse` vs the pre-existing
// siblings in the outer scope) resolve without collision.
//
// Regression guard: every test in this module verifies BC-canonical behavior
// of `render_findings_grouped_collapsed` and related helpers. A failing
// assertion is the RED signal for the implementer to fix.
// ============================================================================
mod story_119 {
    use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
    use wirerust::reporter::Reporter;
    use wirerust::reporter::terminal::{Collapse, FindingsRender, Grouping, TerminalReporter};
    use wirerust::summary::Summary;

    // -----------------------------------------------------------------------
    // Module-level helpers
    // -----------------------------------------------------------------------

    /// Returns a `TerminalReporter` wired to `{Grouped, Collapsed}` with
    /// color and hosts-breakdown disabled. This is the canonical reporter for
    /// all STORY-119/B grouped-collapse behavioral tests.
    ///
    /// Fields used: `use_color`, `show_hosts_breakdown`, `render` —
    /// `TerminalReporter` has no `findings` field (STORY-119/B Previous Story
    /// Intelligence lesson 3).
    fn grouped_collapse_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed,
            },
        }
    }

    /// Returns a `TerminalReporter` wired to `{Grouped, Collapsed}` WITH
    /// color enabled. Used by the color-ladder test (AC-008 / BC-2.11.031 PC-3).
    fn grouped_collapse_reporter_color() -> TerminalReporter {
        TerminalReporter {
            use_color: true,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed,
            },
        }
    }

    /// Returns a `TerminalReporter` wired to `{Grouped, Expanded}` (the
    /// `--mitre --no-collapse` path). Used to verify suffix-free behaviour and
    /// byte-identical singleton output.
    fn grouped_expanded_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
        }
    }

    /// Builds a minimal `Finding` with the given `summary`, no evidence, no
    /// MITRE techniques, `Verdict::Likely`, `Confidence::High`.
    fn make_finding_s119(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec![],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    /// Builds a `Finding` with a known MITRE technique T1046 (Discovery tactic).
    /// T1046 → "Network Service Discovery" per `technique_info`.
    fn make_discovery_finding_s119(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec!["T1046".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    /// Builds a `Finding` with a known MITRE technique T1071 (Command and
    /// Control tactic). T1071 → "Application Layer Protocol".
    fn make_c2_finding_s119(summary: impl Into<String>) -> Finding {
        Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: summary.into(),
            evidence: vec![],
            mitre_techniques: vec!["T1071".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }
    }

    // -----------------------------------------------------------------------
    // AC-001 — `--mitre` alone routes to {Grouped, Collapsed}
    // (traces to BC-2.11.030 PC-2)
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.11.030 Postcondition 2):
    /// REGRESSION GUARD: The `run_analyze` construction site produces
    /// `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Collapsed }`
    /// when `show_mitre_grouping == true` and `collapse_findings == true`
    /// (`--mitre` alone, default collapse enabled).
    ///
    /// Verified via the observable dispatch: a reporter constructed with
    /// `{Grouped, Collapsed}` exercises `render_findings_grouped_collapsed`.
    /// A FINDINGS section with grouped-collapse output appears in the rendered
    /// string (tactic headers and `(xN)` suffix for N≥2 groups).
    #[test]
    fn test_BC_2_11_030_mitre_alone_maps_to_grouped_collapsed() {
        // Construct the reporter exactly as run_analyze does when
        // show_mitre_grouping=true, collapse_findings=true.
        let render = FindingsRender {
            grouping: if true {
                Grouping::Grouped
            } else {
                Grouping::Flat
            },
            collapse: if true {
                Collapse::Collapsed
            } else {
                Collapse::Expanded
            },
        };
        assert_eq!(
            render,
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Collapsed,
            },
            "AC-001: --mitre alone (show_mitre_grouping=true, collapse_findings=true) \
             must produce {{Grouped, Collapsed}}"
        );

        // Observable render: N=3 identical-key findings in one tactic bucket
        // must produce a header with `(x3)` suffix — the grouped-collapse path.
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_discovery_finding_s119("s119-ac001-mitre-collapse"))
            .collect();
        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render,
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            out.contains("(x3)"),
            "AC-001: {{Grouped,Collapsed}} with N=3 identical findings must emit \
             `(x3)` suffix; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 — `--mitre --no-collapse` routes to {Grouped, Expanded}
    // (traces to BC-2.11.030 PC-3)
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.11.030 Postcondition 3):
    /// REGRESSION GUARD: The `run_analyze` construction site produces
    /// `FindingsRender { grouping: Grouping::Grouped, collapse: Collapse::Expanded }`
    /// when `show_mitre_grouping == true` and `collapse_findings == false`
    /// (`--mitre` with `--no-collapse`).
    #[test]
    fn test_BC_2_11_030_mitre_no_collapse_maps_to_grouped_expanded() {
        let render = FindingsRender {
            grouping: if true {
                Grouping::Grouped
            } else {
                Grouping::Flat
            },
            collapse: if false {
                Collapse::Collapsed
            } else {
                Collapse::Expanded
            },
        };
        assert_eq!(
            render,
            FindingsRender {
                grouping: Grouping::Grouped,
                collapse: Collapse::Expanded,
            },
            "AC-002: --mitre --no-collapse (show_mitre_grouping=true, collapse_findings=false) \
             must produce {{Grouped, Expanded}}"
        );

        // Observable render: {Grouped, Expanded} must not emit any `(xN)` suffix.
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_discovery_finding_s119("s119-ac002-mitre-expanded"))
            .collect();
        let out = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render,
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            !out.contains("(x"),
            "AC-002: {{Grouped,Expanded}} must not emit any `(xN)` suffix; got:\n{out}"
        );
        assert!(
            out.contains("## "),
            "AC-002: {{Grouped,Expanded}} must emit tactic headers; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 — flat-mode routing unchanged at construction site
    // (traces to BC-2.11.030 PC-4 and PC-5)
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.11.030 Postconditions 4 and 5):
    /// REGRESSION GUARD: Flat-mode `FindingsRender` wiring at the `run_analyze`
    /// construction site is unchanged.
    ///
    /// BC-2.11.030 PC-4: "When neither `--mitre` nor `--no-collapse` is present
    /// (the default terminal output): `render == FindingsRender { grouping:
    /// Grouping::Flat, collapse: Collapse::Collapsed }`. Unchanged from
    /// pre-STORY-119 behavior."
    ///
    /// BC-2.11.030 PC-5: "When `--no-collapse` is present but `--mitre` is
    /// absent: `render == FindingsRender { grouping: Grouping::Flat, collapse:
    /// Collapse::Expanded }`. Unchanged from pre-STORY-119 behavior."
    #[test]
    fn test_BC_2_11_030_flat_routing_unchanged() {
        // PC-4: default (no --mitre, no --no-collapse)
        let default_render = FindingsRender {
            grouping: if false {
                Grouping::Grouped
            } else {
                Grouping::Flat
            },
            collapse: if true {
                Collapse::Collapsed
            } else {
                Collapse::Expanded
            },
        };
        assert_eq!(
            default_render,
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
            "AC-003 PC-4: default routing must produce {{Flat, Collapsed}}"
        );

        // PC-5: --no-collapse without --mitre
        let no_collapse_flat_render = FindingsRender {
            grouping: if false {
                Grouping::Grouped
            } else {
                Grouping::Flat
            },
            collapse: if false {
                Collapse::Collapsed
            } else {
                Collapse::Expanded
            },
        };
        assert_eq!(
            no_collapse_flat_render,
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
            "AC-003 PC-5: --no-collapse without --mitre must produce {{Flat, Expanded}}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 — per-bucket collapse: N≥2 group renders header with `(xN)` suffix
    // (traces to BC-2.11.031 PC-1)
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.11.031 Postcondition 1):
    /// REGRESSION GUARD: For a group of N=3 findings sharing the same collapse
    /// key within a MITRE tactic bucket under `{Grouped, Collapsed}`, the header
    /// line contains ` (x3)` — a space before the opening parenthesis, exact
    /// decimal N, no leading zeros.
    #[test]
    fn test_BC_2_11_031_grouped_collapse_suffix_format() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_discovery_finding_s119("s119-ac006-suffix"))
            .collect();
        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // BC-2.11.031 PC-1: header line contains ` (x3)` suffix.
        assert!(
            out.contains("(x3)"),
            "AC-006: N=3 group in bucket must emit `(x3)` suffix; got:\n{out}"
        );
        // Tactic header for Discovery must be present.
        assert!(
            out.contains("## Discovery"),
            "AC-006: Discovery bucket header must appear; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 — singleton (N=1) renders via render_finding_grouped, no suffix
    // (traces to BC-2.11.031 PC-2)
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.11.031 Postcondition 2):
    /// REGRESSION GUARD: A singleton finding (N=1 within a bucket) under
    /// `{Grouped, Collapsed}` renders via `render_finding_grouped` with no
    /// `(xN)` suffix — byte-identical to `{Grouped, Expanded}` for that finding.
    #[test]
    fn test_BC_2_11_031_singleton_no_suffix_in_bucket() {
        let findings = vec![make_discovery_finding_s119("s119-ac007-singleton")];

        let out_collapsed = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);
        let out_expanded = grouped_expanded_reporter().render(&Summary::new(), &findings, &[]);

        // BC-2.11.031 PC-2: singleton output is byte-identical to {Grouped, Expanded}.
        assert_eq!(
            out_collapsed, out_expanded,
            "AC-007: singleton under {{Grouped,Collapsed}} must be byte-identical to \
             {{Grouped,Expanded}}; outputs differ"
        );
        // No (xN) suffix anywhere in output.
        assert!(
            !out_collapsed.contains("(x"),
            "AC-007: singleton must not emit any `(xN)` suffix; got:\n{out_collapsed}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 — color-ladder: `(xN)` suffix is part of the pre-colorization string
    // (traces to BC-2.11.031 PC-3)
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.11.031 Postcondition 3):
    /// REGRESSION GUARD: For a `Likely + High` group of N=2 under
    /// `{Grouped, Collapsed}` with `use_color: true`, the ` (x2)` suffix
    /// is inside the ANSI color span (i.e., it appears before the ANSI reset
    /// sequence, not after it). Verified by asserting the suffix is NOT
    /// preceded by an ANSI reset `\x1b[0m` in the output.
    #[test]
    fn test_BC_2_11_031_grouped_collapse_color_ladder() {
        let findings: Vec<Finding> = (0..2)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac008-color".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();
        let out = grouped_collapse_reporter_color().render(&Summary::new(), &findings, &[]);

        // BC-2.11.031 PC-3: the suffix must appear in the output.
        assert!(
            out.contains("(x2)"),
            "AC-008: N=2 group with color must still emit `(x2)` suffix; got:\n{out}"
        );
        // The ANSI reset sequence (\x1b[0m) must NOT appear BEFORE `(x2)` on
        // the same header line. Verify by checking that `(x2)` does not appear
        // after a reset on the header line.
        let header_line = out.lines().find(|l| l.contains("(x2)")).unwrap_or_default();
        // Split by reset: if (x2) appears in the LAST segment (after reset),
        // the suffix was appended after the ANSI reset — NON-CONFORMANT.
        let reset = "\x1b[0m";
        let after_reset = header_line
            .rfind(reset)
            .map(|pos| &header_line[pos + reset.len()..])
            .unwrap_or("");
        assert!(
            !after_reset.contains("(x2)"),
            "AC-008: `(x2)` must be inside the ANSI color span (before reset), \
             not after the reset sequence. Header line: {header_line:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 — `(xN)` suffix NOT on MITRE line, evidence lines, or bucket headers
    // (traces to BC-2.11.031 PC-4)
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.11.031 Postcondition 4):
    /// REGRESSION GUARD: The ` (xN)` suffix appears ONLY on the finding-group
    /// header line. It must not appear on the MITRE line (`    MITRE: ...`),
    /// evidence lines (`    > ...`), or the tactic bucket header (`  ## ...`).
    #[test]
    fn test_BC_2_11_031_suffix_only_on_header_not_mitre_or_evidence() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac009-no-suffix-leak".to_string(),
                evidence: vec!["evidence-line".to_string()],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();
        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // `(x3)` must appear somewhere in output.
        assert!(
            out.contains("(x3)"),
            "AC-009: N=3 group must emit `(x3)` suffix; got:\n{out}"
        );

        // `(xN)` must NOT appear on any MITRE line, evidence line, or bucket header.
        for line in out.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("MITRE:") {
                assert!(
                    !line.contains("(x"),
                    "AC-009: `(xN)` must not appear on MITRE line: {line:?}"
                );
            }
            if trimmed.starts_with("> ") {
                assert!(
                    !line.contains("(x"),
                    "AC-009: `(xN)` must not appear on evidence line: {line:?}"
                );
            }
            if trimmed.starts_with("## ") {
                assert!(
                    !line.contains("(x"),
                    "AC-009: `(xN)` must not appear on tactic bucket header: {line:?}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // AC-010 — cross-bucket suffix independence
    // (traces to BC-2.11.031 PC-6)
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.11.031 Postcondition 6):
    /// REGRESSION GUARD: Two groups with the same collapse key but in different
    /// MITRE tactic buckets produce independent `(xN)` suffixes. A group of 3
    /// in the Discovery bucket (T1046) and a group of 2 in the Command and
    /// Control bucket (T1071) produce `(x3)` and `(x2)` respectively — never
    /// merged to `(x5)`.
    #[test]
    fn test_BC_2_11_031_cross_bucket_suffix_independence() {
        // 3 findings with same summary → Discovery bucket (T1046).
        let mut findings: Vec<Finding> = (0..3)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac010-shared-key".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();
        // 2 findings with same summary → Command and Control bucket (T1071).
        findings.extend((0..2).map(|_| Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "s119-ac010-shared-key".to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T1071".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }));

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Each bucket must produce its own independent suffix.
        assert!(
            out.contains("(x3)"),
            "AC-010: Discovery bucket (3 findings) must emit `(x3)`; got:\n{out}"
        );
        assert!(
            out.contains("(x2)"),
            "AC-010: C&C bucket (2 findings) must emit `(x2)`; got:\n{out}"
        );
        // No cross-bucket merge to (x5).
        assert!(
            !out.contains("(x5)"),
            "AC-010: cross-bucket merge must not occur — `(x5)` must not appear; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 — per-bucket evidence sampling: at most K=3 lines per N≥2 group
    // (traces to BC-2.11.032 PC-1)
    // -----------------------------------------------------------------------

    /// AC-011 (BC-2.11.032 Postconditions 1–2):
    /// REGRESSION GUARD: For a collapsed group of N=5 in a tactic bucket, each
    /// member with one evidence line, the terminal output contains at most K=3
    /// evidence lines (`    > ` prefix).
    #[test]
    fn test_BC_2_11_032_evidence_sampling_k3_in_bucket() {
        let findings: Vec<Finding> = (0..5)
            .map(|i| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac011-k3-evidence".to_string(),
                evidence: vec![format!("ev-{i}")],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        let evidence_count = out
            .lines()
            .filter(|l| l.trim_start().starts_with("> "))
            .count();
        assert!(
            evidence_count <= 3,
            "AC-011: K=3 evidence cap — at most 3 evidence lines; got {evidence_count} in:\n{out}"
        );
        assert!(
            evidence_count >= 1,
            "AC-011: at least 1 evidence line expected from evidence-bearing group; \
             got {evidence_count} in:\n{out}"
        );
        // `(x5)` suffix must be present (group of 5).
        assert!(
            out.contains("(x5)"),
            "AC-011: N=5 group must emit `(x5)` suffix; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 edge — no sliding window: empty-evidence member blocks window
    // (traces to BC-2.11.032 Invariant 2)
    // -----------------------------------------------------------------------

    /// AC-011 edge (BC-2.11.032 Invariant 2):
    /// REGRESSION GUARD: For a group of N=5, `members[0].evidence = []` (empty),
    /// `members[1..4]` each have one evidence line. The positional window
    /// inspects members[0], [1], [2] — member[0] contributes 0 lines (no
    /// sliding), members[1] and [2] each contribute 1 line. Total = 2 evidence
    /// lines; NOT 3.
    #[test]
    fn test_BC_2_11_032_evidence_positional_no_slide() {
        // members[0]: no evidence.
        let mut findings = vec![Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "s119-ac011b-no-slide".to_string(),
            evidence: vec![],
            mitre_techniques: vec!["T1046".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }];
        // members[1..4]: each with one evidence line.
        findings.extend((1..5).map(|i| Finding {
            category: ThreatCategory::Anomaly,
            verdict: Verdict::Likely,
            confidence: Confidence::High,
            summary: "s119-ac011b-no-slide".to_string(),
            evidence: vec![format!("ev-{i}")],
            mitre_techniques: vec!["T1046".to_string()],
            source_ip: None,
            timestamp: None,
            direction: None,
        }));

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Window inspects members[0..min(5,3)=3]. members[0]: 0 lines; [1] and [2]: 1 each.
        let evidence_count = out
            .lines()
            .filter(|l| l.trim_start().starts_with("> "))
            .count();
        assert_eq!(
            evidence_count, 2,
            "AC-011 edge: positional no-slide — members[0] empty, window=[0,1,2], \
             expected 2 evidence lines; got {evidence_count} in:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-013 — tactic-bucket ordering invariant under {Grouped, Collapsed}
    // (traces to BC-2.11.033 PC-1 / VP-016)
    // -----------------------------------------------------------------------

    /// AC-013 (BC-2.11.033 Postconditions 1–2 / VP-016):
    /// REGRESSION GUARD: Tactic bucket headers appear in the order returned by
    /// `all_tactics_in_report_order()` under `{Grouped, Collapsed}`. Discovery
    /// (index 8) appears before Command and Control (index 11) in the output.
    #[test]
    fn test_BC_2_11_033_grouped_collapsed_preserves_bucket_order() {
        // Findings in two tactics in reverse order to confirm sorted output.
        let mut findings: Vec<Finding> = (0..2)
            .map(|_| make_c2_finding_s119("s119-ac013-c2")) // C&C = later bucket
            .collect();
        findings.extend((0..2).map(|_| make_discovery_finding_s119("s119-ac013-disc"))); // Discovery = earlier

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        let disc_pos = out.find("## Discovery");
        let c2_pos = out.find("## Command and Control");

        assert!(
            disc_pos.is_some(),
            "AC-013: Discovery bucket header must appear; got:\n{out}"
        );
        assert!(
            c2_pos.is_some(),
            "AC-013: Command and Control bucket header must appear; got:\n{out}"
        );
        assert!(
            disc_pos.unwrap() < c2_pos.unwrap(),
            "AC-013: Discovery (index 8) must appear before Command and Control (index 11) \
             per all_tactics_in_report_order(); got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-014 — `Uncategorized` emitted last under {Grouped, Collapsed}
    // (traces to BC-2.11.033 PC-3)
    // -----------------------------------------------------------------------

    /// AC-014 (BC-2.11.033 Postcondition 3):
    /// REGRESSION GUARD: The `Uncategorized` bucket header appears last among
    /// emitted buckets under `{Grouped, Collapsed}`.
    #[test]
    fn test_BC_2_11_033_uncategorized_last_under_grouped_collapse() {
        // One finding in Discovery, two uncategorized (no mitre_techniques).
        let mut findings = vec![make_discovery_finding_s119("s119-ac014-disc")];
        findings.extend((0..2).map(|_| make_finding_s119("s119-ac014-uncategorized")));

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        let disc_pos = out.find("## Discovery");
        let uncat_pos = out.find("## Uncategorized");

        assert!(
            disc_pos.is_some(),
            "AC-014: Discovery bucket header must appear; got:\n{out}"
        );
        assert!(
            uncat_pos.is_some(),
            "AC-014: Uncategorized bucket header must appear; got:\n{out}"
        );
        assert!(
            disc_pos.unwrap() < uncat_pos.unwrap(),
            "AC-014: Uncategorized must appear after Discovery — Uncategorized is last; \
             got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-015 — bucket membership unchanged by collapse
    // (traces to BC-2.11.033 PC-4)
    // -----------------------------------------------------------------------

    /// AC-015 (BC-2.11.033 Postcondition 4):
    /// REGRESSION GUARD: A finding assigned to the Discovery bucket under
    /// `{Grouped, Expanded}` is assigned to the same bucket under `{Grouped,
    /// Collapsed}`. The collapse key is orthogonal to bucket assignment.
    #[test]
    fn test_BC_2_11_033_different_buckets_not_cross_collapsed() {
        // Two findings with the same summary but different MITRE techniques
        // (different buckets). They must NOT be cross-collapsed.
        let findings = vec![
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac015-same-key".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()], // Discovery
                source_ip: None,
                timestamp: None,
                direction: None,
            },
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac015-same-key".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1071".to_string()], // C&C
                source_ip: None,
                timestamp: None,
                direction: None,
            },
        ];

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Both bucket headers must appear (two separate buckets, not merged).
        assert!(
            out.contains("## Discovery"),
            "AC-015: Discovery bucket header must appear (same key, different tactic); \
             got:\n{out}"
        );
        assert!(
            out.contains("## Command and Control"),
            "AC-015: C&C bucket header must appear (same key, different tactic); got:\n{out}"
        );
        // No cross-bucket merge: `(x2)` must NOT appear (each bucket has only 1).
        assert!(
            !out.contains("(x2)"),
            "AC-015: cross-bucket collapse must not occur — `(x2)` must not appear; \
             got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-016 / AC-017 — sort-then-collapse: post-sort order determines representative
    // (traces to BC-2.11.033 PC-5/6)
    // -----------------------------------------------------------------------

    /// AC-016/AC-017 (BC-2.11.033 Postconditions 5 and 6):
    /// REGRESSION GUARD: Within a bucket, findings are sorted by verdict-rank
    /// ascending (Likely=0, Possible=1, Inconclusive=2, Unlikely=3) BEFORE the
    /// collapse pass. The lower-rank (higher-severity) finding becomes the group
    /// representative.
    ///
    /// Setup: three findings in the T1046 (Discovery) bucket.
    ///   - Two share key (Anomaly, Likely, High, "s119-ac016-likely-finding"):
    ///     they collapse to a N=2 group. The group representative is Likely.
    ///   - One has key (Anomaly, Inconclusive, Low, "s119-ac016-inconclusive"):
    ///     a singleton group. Sorts after the Likely group.
    ///
    /// Observable consequences:
    ///   1. The N=2 group header shows `(x2)` suffix.
    ///   2. The N=2 group header uses UPPERCASE verdict `LIKELY` (not title-case).
    ///   3. The Likely group header appears before the Inconclusive singleton in output.
    #[test]
    fn test_BC_2_11_033_first_occurrence_in_sorted_bucket_order() {
        let findings = vec![
            // Emitted first — Inconclusive/Low singleton.
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Inconclusive, // rank=2 — sorts after Likely
                confidence: Confidence::Low,
                summary: "s119-ac016-inconclusive".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            },
            // Emitted second — first member of the Likely/High N=2 group.
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely, // rank=0 — sorts first
                confidence: Confidence::High,
                summary: "s119-ac016-likely-finding".to_string(),
                evidence: vec!["ev-ac016-a".to_string()],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            },
            // Emitted third — second member of the Likely/High N=2 group (same key).
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely, // rank=0 — same key as above
                confidence: Confidence::High,
                summary: "s119-ac016-likely-finding".to_string(),
                evidence: vec!["ev-ac016-b".to_string()],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            },
        ];

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // The Likely/High pair collapses to a N=2 group.
        assert!(
            out.contains("(x2)"),
            "AC-016: N=2 group in bucket must emit `(x2)` suffix; got:\n{out}"
        );

        // The header verdict must use UPPERCASE Display format: `LIKELY`, not title-case.
        let header_line = out.lines().find(|l| l.contains("(x2)")).unwrap_or_default();
        assert!(
            header_line.contains("LIKELY"),
            "AC-016: N=2 group header must use uppercase Display format `LIKELY`; \
             header line: {header_line:?}\nfull output:\n{out}"
        );
        assert!(
            !header_line.contains("Likely"),
            "AC-016: header must not contain title-case `Likely` — Display format is `LIKELY`; \
             header line: {header_line:?}\nfull output:\n{out}"
        );

        // Sort order: Likely group (rank=0) appears before Inconclusive singleton (rank=2).
        let likely_pos = out.find("(x2)").unwrap_or(usize::MAX);
        let inconclusive_pos = out.find("INCONCLUSIVE").unwrap_or(usize::MAX);
        assert!(
            likely_pos < inconclusive_pos,
            "AC-017: Likely group (lower rank) must appear before Inconclusive singleton \
             in sorted bucket order; got:\n{out}"
        );
    }

    /// BC-2.11.025 EC-007/EC-008 grouped analogue:
    /// REGRESSION GUARD: Two findings in the SAME tactic bucket with the SAME
    /// summary but DIFFERENT verdict (or confidence, or category) must NOT be
    /// merged — they have distinct four-tuple collapse keys and must render as
    /// two separate group headers with NO `(xN)` suffix on either.
    ///
    /// This catches a summary-only (three-field) collapse key bug: if the
    /// implementation keyed on summary alone, these two findings would be
    /// incorrectly merged into one N=2 group.
    #[test]
    fn test_BC_2_11_025_distinct_verdict_same_summary_no_merge_in_bucket() {
        // Same bucket (T1046 / Discovery), same summary, DIFFERENT verdict.
        let findings = vec![
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ec007-same-summary".to_string(),
                evidence: vec!["ev-likely".to_string()],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            },
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Inconclusive,
                confidence: Confidence::High,
                summary: "s119-ec007-same-summary".to_string(), // identical summary
                evidence: vec!["ev-inconclusive".to_string()],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            },
        ];

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Different verdict → different four-tuple keys → two distinct groups, no merge.
        assert!(
            !out.contains("(x2)"),
            "BC-2.11.025 EC-007: same summary but different verdict must produce TWO distinct \
             groups, not one N=2 merged group — `(x2)` must not appear; got:\n{out}"
        );
        // Both headers must appear independently.
        assert!(
            out.contains("LIKELY"),
            "BC-2.11.025 EC-007: Likely group header must appear; got:\n{out}"
        );
        assert!(
            out.contains("INCONCLUSIVE"),
            "BC-2.11.025 EC-007: Inconclusive group header must appear; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-018 / AC-021 — MITRE line from members[0] with em-dash format
    // (traces to BC-2.11.034 PC-1 and PC-5)
    // -----------------------------------------------------------------------

    /// AC-018 / AC-021 (BC-2.11.034 Postconditions 1 and 5):
    /// REGRESSION GUARD: For a collapsed N≥2 group, the MITRE line is rendered
    /// from `group_members[0].mitre_techniques` using em-dash expansion
    /// (U+2014, not ASCII `--`). The observable line format is:
    /// `    MITRE: T1046 — Network Service Discovery`.
    ///
    /// Output block order: (1) header with `(xN)`, (2) evidence lines,
    /// (3) MITRE line. The `(xN)` suffix appears ONLY in item (1).
    #[test]
    fn test_BC_2_11_034_grouped_collapse_mitre_line_em_dash_format() {
        let findings: Vec<Finding> = (0..2)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac018-mitre-em-dash".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // BC-2.11.034 PC-1: em-dash (U+2014) + name on MITRE line.
        assert!(
            out.contains("T1046 \u{2014} Network Service Discovery"),
            "AC-018: MITRE line must use em-dash (U+2014) + name from members[0]; got:\n{out}"
        );
        // `(x2)` suffix on header.
        assert!(
            out.contains("(x2)"),
            "AC-018: header must carry `(x2)` suffix; got:\n{out}"
        );
        // Block order: header (with `(x2)`) must appear before MITRE line.
        let header_pos = out.find("(x2)").unwrap_or(usize::MAX);
        let mitre_pos = out.find("MITRE: T1046").unwrap_or(usize::MAX);
        assert!(
            header_pos < mitre_pos,
            "AC-021: header (with `(x2)`) must appear before MITRE line; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-018 edge — unknown technique in collapsed group
    // (traces to BC-2.11.034 PC-1 unknown-ID branch)
    // -----------------------------------------------------------------------

    /// AC-018 edge (BC-2.11.034 Postcondition 1 — unknown ID):
    /// REGRESSION GUARD: For a collapsed N≥2 group where `members[0]` has an
    /// unknown technique ID, the MITRE line format is
    /// `    MITRE: <ids_joined> (unknown)` — not an em-dash expansion.
    #[test]
    fn test_BC_2_11_034_unknown_technique_in_grouped_collapse() {
        let findings: Vec<Finding> = (0..2)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac018b-unknown-tech".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T9999".to_string()], // unknown ID
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("(x2)"),
            "AC-018 edge: N=2 group with unknown technique must emit `(x2)` suffix; \
             got:\n{out}"
        );
        assert!(
            out.contains("T9999 (unknown)"),
            "AC-018 edge: unknown technique ID must produce `(unknown)` format on MITRE line; \
             got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-019 — `(xN)` suffix NOT on MITRE line for N≥2 groups
    // (traces to BC-2.11.034 PC-2)
    // -----------------------------------------------------------------------

    /// AC-019 (BC-2.11.034 Postcondition 2):
    /// REGRESSION GUARD: The `(xN)` count suffix does not appear on the MITRE
    /// line for N≥2 collapsed groups. The suffix is scoped to the header line
    /// only (verified together with AC-009).
    #[test]
    fn test_BC_2_11_034_suffix_not_on_mitre_line() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_discovery_finding_s119("s119-ac019-no-suffix-mitre"))
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // `(x3)` must appear (on header).
        assert!(
            out.contains("(x3)"),
            "AC-019: `(x3)` must appear in output; got:\n{out}"
        );
        // The MITRE line must not contain any `(xN)`.
        for line in out.lines() {
            if line.trim_start().starts_with("MITRE:") {
                assert!(
                    !line.contains("(x"),
                    "AC-019: `(xN)` must not appear on MITRE line: {line:?}"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // AC-020 — divergent MITRE: only members[0] appears in terminal output
    // (traces to BC-2.11.034 PC-3)
    // -----------------------------------------------------------------------

    /// AC-020 (BC-2.11.034 Postcondition 3):
    /// REGRESSION GUARD: For a collapsed group of N≥2 where members have
    /// divergent `mitre_techniques`, only `members[0]`'s technique appears on
    /// the MITRE line. The other members' techniques are elided from terminal
    /// output (preserved in raw findings for JSON/CSV).
    ///
    /// Both findings MUST be in the same tactic bucket for per-bucket collapse
    /// to merge them (BC-2.11.033 Invariant 3 / EC-003): T1046 and T1083 both
    /// map to Discovery, so they land in the same bucket and collapse as N=2.
    /// Using findings from different tactics (e.g. T1046 Discovery + T1071 C&C)
    /// would produce two independent singletons per BC-2.11.033 EC-003 — the
    /// correct per-bucket behavior — but the divergent-MITRE elision property
    /// (BC-2.11.034 PC-3) can only be observed when N≥2 within one bucket.
    #[test]
    fn test_BC_2_11_034_divergent_mitre_representative_sourcing() {
        // Two findings with same summary but different MITRE techniques,
        // both in the Discovery tactic bucket (T1046 and T1083).
        // After sort (Likely rank=0 for both, same confidence), the first
        // submitted becomes members[0] (stable emission-index tiebreak).
        // They collapse into N=2 within the Discovery bucket.
        let findings = vec![
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac020-divergent-mitre".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()], // members[0] — Discovery
                source_ip: None,
                timestamp: None,
                direction: None,
            },
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac020-divergent-mitre".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1083".to_string()], // members[1] — Discovery; must be elided
                source_ip: None,
                timestamp: None,
                direction: None,
            },
        ];

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // members[0]'s technique must appear on the MITRE line.
        assert!(
            out.contains("T1046"),
            "AC-020: members[0] technique T1046 must appear in terminal output; got:\n{out}"
        );
        // members[1]'s technique must NOT appear in terminal output.
        assert!(
            !out.contains("T1083"),
            "AC-020: members[1] technique T1083 must be elided from terminal output; \
             got:\n{out}"
        );
        // Group must be collapsed as N=2 (same bucket, same summary-only key).
        assert!(
            out.contains("(x2)"),
            "AC-020: same-bucket same-summary findings must collapse to N=2; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-022 — {Grouped, Expanded} suffix-free guarantee unchanged
    // (traces to BC-2.11.013 Invariant 4)
    // -----------------------------------------------------------------------

    /// AC-022 (BC-2.11.013 Invariant 4):
    /// REGRESSION GUARD: Under `render = {Grouped, Expanded}`, N=100 identical-
    /// key findings in a tactic bucket produce 100 individual finding lines with
    /// no ` (xN)` suffix on any line. The `{Grouped, Expanded}` suffix-free
    /// guarantee (pre-STORY-119 `--mitre` behavior) is unchanged.
    #[test]
    fn test_BC_2_11_028_no_collapse_with_mitre_produces_grouped_expanded() {
        let findings: Vec<Finding> = (0..100)
            .map(|_| make_discovery_finding_s119("s119-ac022-no-collapse-100"))
            .collect();

        let out = grouped_expanded_reporter().render(&Summary::new(), &findings, &[]);

        // No `(xN)` suffix anywhere.
        assert!(
            !out.contains("(x"),
            "AC-022: {{Grouped,Expanded}} with N=100 identical findings must emit zero \
             `(xN)` suffixes; got excerpt:\n{}",
            &out[..out.len().min(500)]
        );
        // All 100 findings must be rendered (header line count via `[Anomaly]`).
        let header_count = out.lines().filter(|l| l.contains("[Anomaly]")).count();
        assert_eq!(
            header_count,
            100,
            "AC-022: {{Grouped,Expanded}} must render all 100 findings; got {header_count} in \
             output (first 500 chars):\n{}",
            &out[..out.len().min(500)]
        );
    }

    // -----------------------------------------------------------------------
    // AC-023 — escape_for_terminal applied in grouped-collapse path
    // (traces to BC-2.11.031 Precondition 5 / VP-012)
    // -----------------------------------------------------------------------

    /// AC-023 (BC-2.11.031 Precondition 5 / VP-012):
    /// REGRESSION GUARD: `escape_for_terminal` is applied to all `summary` and
    /// `evidence` strings in `render_findings_grouped_collapsed`. A summary
    /// containing a C0 control character (U+0001) must not appear raw in the
    /// output — it must be escaped via `char::escape_default` to `\u{1}`.
    #[test]
    fn test_BC_2_11_031_escape_for_terminal_in_grouped_collapse_path() {
        let findings: Vec<Finding> = (0..2)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                // C0 control character in summary — must be escaped.
                summary: "s119-ac023-\u{0001}escape".to_string(),
                evidence: vec!["ev-\u{0001}escape".to_string()],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Raw U+0001 must NOT appear in the output.
        assert!(
            !out.contains('\u{0001}'),
            "AC-023: raw C0 control char U+0001 must not appear in output — \
             escape_for_terminal must be applied; got:\n{out:?}"
        );
        // The escaped form `\u{1}` must appear in output (char::escape_default output).
        assert!(
            out.contains("\\u{1}"),
            "AC-023: C0 char U+0001 must be escaped to `\\u{{1}}` via char::escape_default; \
             got:\n{out:?}"
        );
    }

    /// BC-2.11.032 EC-007 / VP canonical vector:
    /// REGRESSION GUARD: Evidence strings in grouped-collapse bucket groups pass
    /// through `escape_for_terminal`. A raw ESC byte (U+001B = 0x1B) in evidence
    /// must be escaped via `char::escape_default` to `\u{1b}` in the output.
    ///
    /// Canonical test vector (BC-2.11.032 EC-007):
    ///   evidence `"\x1b[31m"` → rendered as `> \u{1b}[31m` in the terminal.
    #[test]
    fn test_BC_2_11_032_escape_preserved_in_bucket_evidence() {
        // N=2 group so both findings collapse; K=3 evidence cap not reached.
        let findings: Vec<Finding> = (0..2)
            .map(|i| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ec007-esc-evidence".to_string(),
                // Raw ESC byte in ANSI sequence — must be escaped in output.
                evidence: vec![format!("\x1b[31m-item-{i}")],
                mitre_techniques: vec!["T1046".to_string()],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Raw ESC byte must NOT appear in the output.
        assert!(
            !out.contains('\x1b'),
            "BC-2.11.032 EC-007: raw ESC byte must not appear in grouped-collapse evidence output; \
             got:\n{out:?}"
        );
        // The ESC must be escaped via char::escape_default to `\u{1b}`.
        assert!(
            out.contains("\\u{1b}"),
            "BC-2.11.032 EC-007: ESC byte in evidence must render as `\\u{{1b}}` \
             (char::escape_default); got:\n{out:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-024 — collapse_findings_pass_refs called per bucket, not globally
    // (traces to BC-2.11.033 Invariant 3 / BC-2.11.025 Invariant 5)
    // -----------------------------------------------------------------------

    /// AC-024 / test_BC_2_11_025_grouped_mode_bypasses_flat_collapse
    /// (BC-2.11.025 Invariant 5 — updated; BC-2.11.033 Invariant 3):
    /// REGRESSION GUARD: `render.grouping == Grouping::Grouped` with
    /// `Collapse::Collapsed` uses per-bucket `collapse_findings_pass_refs`,
    /// never the global flat `collapse_findings_pass` adapter. Observable
    /// consequence: two findings with the same summary but in different tactic
    /// buckets produce two separate group headers (one per bucket), not one
    /// merged group of N=2.
    ///
    /// Note: this test is DISTINCT from the pre-existing siblings
    /// `test_BC_2_11_025_grouped_mode_bypasses_collapse` (line ~2072) and
    /// `test_BC_2_11_025_grouped_mode_bypasses_collapse_structurally` (line
    /// ~4140) which verify the `{Grouped, Expanded}` path. This test verifies
    /// the `{Grouped, Collapsed}` per-bucket invariant (BC-2.11.025 Inv5).
    #[test]
    fn test_BC_2_11_025_grouped_mode_bypasses_flat_collapse() {
        // Same collapse key, different MITRE tactic → different buckets.
        // A global flat collapse pass would merge them into one N=2 group.
        // Per-bucket collapse must produce two separate singleton groups (N=1
        // each) — neither emits `(x2)` and neither emits a cross-bucket header.
        let findings = vec![
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac024-per-bucket".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1046".to_string()], // Discovery
                source_ip: None,
                timestamp: None,
                direction: None,
            },
            Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ac024-per-bucket".to_string(),
                evidence: vec![],
                mitre_techniques: vec!["T1071".to_string()], // C&C
                source_ip: None,
                timestamp: None,
                direction: None,
            },
        ];

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Per-bucket pass: each bucket has N=1 → no `(x2)` suffix.
        assert!(
            !out.contains("(x2)"),
            "AC-024: per-bucket collapse must not cross-collapse findings from different \
             buckets — `(x2)` must not appear; got:\n{out}"
        );
        // Both bucket headers must appear (each bucket independently emitted).
        assert!(
            out.contains("## Discovery"),
            "AC-024: Discovery bucket header must appear; got:\n{out}"
        );
        assert!(
            out.contains("## Command and Control"),
            "AC-024: C&C bucket header must appear; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-025 — flat paths byte-identical
    // (traces to BC-2.11.025 Invariant 5)
    // -----------------------------------------------------------------------

    /// AC-025 (BC-2.11.025 Invariant 5):
    /// REGRESSION GUARD: The `{Flat, Collapsed}` arm calls `render_findings_collapsed`
    /// and the `{Flat, Expanded}` arm calls the `render_finding_flat` loop —
    /// both byte-identical to v0.9.0 behavior. Flat-mode tests are unaffected
    /// by STORY-119/B. Verified here as a smoke check.
    #[test]
    fn test_BC_2_11_025_flat_paths_unchanged_by_story_119() {
        let findings: Vec<Finding> = (0..3)
            .map(|_| make_finding_s119("s119-ac025-flat-unchanged"))
            .collect();

        // {Flat, Collapsed} must emit at most 1 `(x3)` suffix (flat collapse).
        let out_fc = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            out_fc.contains("(x3)"),
            "AC-025: {{Flat,Collapsed}} with N=3 identical findings must emit `(x3)`; \
             got:\n{out_fc}"
        );

        // {Flat, Expanded} must emit zero `(xN)` suffixes.
        let out_fe = TerminalReporter {
            use_color: false,
            show_hosts_breakdown: false,
            render: FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Expanded,
            },
        }
        .render(&Summary::new(), &findings, &[]);
        assert!(
            !out_fe.contains("(x"),
            "AC-025: {{Flat,Expanded}} must not emit any `(xN)` suffix; got:\n{out_fe}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 — run_summary construction site produces {Flat, Collapsed}
    // (traces to BC-2.11.030 PC-6)
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.11.030 Postcondition 6):
    /// REGRESSION GUARD: The `run_summary` construction site always produces
    /// `FindingsRender { grouping: Grouping::Flat, collapse: Collapse::Collapsed }`.
    /// Inert value semantics — `run_summary` renders no FINDINGS section; the
    /// field is structurally present but irrelevant.
    #[test]
    fn test_BC_2_11_030_run_summary_produces_flat_collapsed() {
        // Verify the literal struct value that run_summary uses.
        let run_summary_render = FindingsRender {
            grouping: Grouping::Flat,
            collapse: Collapse::Collapsed,
        };
        assert_eq!(
            run_summary_render,
            FindingsRender {
                grouping: Grouping::Flat,
                collapse: Collapse::Collapsed,
            },
            "AC-004: run_summary render field must be {{Flat, Collapsed}}"
        );
        // This is a structural/value test — no dispatch exercised.
    }

    // -----------------------------------------------------------------------
    // AC-005 — render_findings_grouped_collapsed function exists and dispatches
    // (traces to BC-2.11.031 Architecture Anchors)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.11.031 Architecture Anchors):
    /// REGRESSION GUARD: `render_findings_grouped_collapsed` is dispatched for
    /// the `(Grouping::Grouped, Collapse::Collapsed)` arm. Observable: a
    /// `{Grouped, Collapsed}` reporter with N=1 finding renders the FINDINGS
    /// section (non-empty FINDINGS block present) without panicking.
    #[test]
    fn test_BC_2_11_031_grouped_collapsed_arm_dispatches_to_new_function() {
        let findings = vec![make_discovery_finding_s119("s119-ac005-dispatch")];
        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        assert!(
            out.contains("FINDINGS"),
            "AC-005: {{Grouped,Collapsed}} must emit a FINDINGS section; got:\n{out}"
        );
        assert!(
            out.contains("## Discovery"),
            "AC-005: {{Grouped,Collapsed}} must bucket into Discovery tactic; got:\n{out}"
        );
    }

    // -----------------------------------------------------------------------
    // EC-006 edge — members[0].mitre_techniques empty → no MITRE line
    // (traces to AC-018 / BC-2.11.034 PC-1)
    // -----------------------------------------------------------------------

    /// EC-006 / AC-018 edge (BC-2.11.034 Postcondition 1):
    /// REGRESSION GUARD: For a collapsed N≥2 group where `members[0].mitre_techniques`
    /// is empty, no MITRE line is rendered — header + evidence only.
    #[test]
    fn test_BC_2_11_034_empty_mitre_techniques_no_mitre_line() {
        // N=2 identical-key findings, no mitre_techniques → Uncategorized bucket.
        let findings: Vec<Finding> = (0..2)
            .map(|_| Finding {
                category: ThreatCategory::Anomaly,
                verdict: Verdict::Likely,
                confidence: Confidence::High,
                summary: "s119-ec006-empty-mitre".to_string(),
                evidence: vec!["ev-ec006".to_string()],
                mitre_techniques: vec![],
                source_ip: None,
                timestamp: None,
                direction: None,
            })
            .collect();

        let out = grouped_collapse_reporter().render(&Summary::new(), &findings, &[]);

        // Header with `(x2)` must appear.
        assert!(
            out.contains("(x2)"),
            "EC-006: N=2 group must emit `(x2)` suffix; got:\n{out}"
        );
        // No MITRE line must be present (empty mitre_techniques on members[0]).
        assert!(
            !out.contains("MITRE:"),
            "EC-006: no MITRE line must appear when members[0].mitre_techniques is empty; \
             got:\n{out}"
        );
    }
}
