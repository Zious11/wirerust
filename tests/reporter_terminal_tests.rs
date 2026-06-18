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
        show_mitre_grouping: false,
        show_hosts_breakdown: false,
        collapse_findings: false,
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
            show_mitre_grouping: true,
            show_hosts_breakdown: false,
            collapse_findings: false,
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
            show_mitre_grouping: false,
            show_hosts_breakdown: false,
            collapse_findings: true,
        }
    }

    /// TerminalReporter with collapse enabled and color enabled (for color-ladder tests).
    fn collapse_reporter_color() -> TerminalReporter {
        TerminalReporter {
            use_color: true,
            show_mitre_grouping: false,
            show_hosts_breakdown: false,
            collapse_findings: true,
        }
    }

    /// TerminalReporter with MITRE grouping and collapse both enabled (for AC-005).
    fn mitre_collapse_reporter() -> TerminalReporter {
        TerminalReporter {
            use_color: false,
            show_mitre_grouping: true,
            show_hosts_breakdown: false,
            collapse_findings: true,
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

    /// AC-005: show_mitre_grouping=true suppresses collapse; no (xN) suffix anywhere.
    ///
    /// This guards the invariant that grouped (--mitre) mode is structurally
    /// suffix-free. FAILS if a future change applies the collapse pass inside
    /// the grouped rendering path, OR if the flat collapse path is not implemented
    /// (the contrast assertion calls render_findings_collapsed to verify the bypass
    /// is not just because collapse is broken — flat mode must produce a suffix
    /// that grouped mode does not).
    #[test]
    fn test_BC_2_11_025_grouped_mode_bypasses_collapse() {
        // BC-2.11.025 invariant 5: when show_mitre_grouping=true, collapse does NOT run.
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

        // Collapse reporter (collapse_findings=true) with a single finding.
        let out_collapse =
            collapse_reporter().render(&Summary::new(), std::slice::from_ref(&f), &[]);

        // No (xN) suffix of any kind.
        assert!(
            !out_collapse.contains("(x"),
            "BC-2.11.026 pc2/inv2: singleton must render with no (xN) suffix; got:\n{out_collapse}"
        );

        // Output must be byte-identical to the pre-v0.8.0 path (collapse_findings=false).
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
    /// This guards the opt-out path: with collapse_findings=false, 5 identical-key
    /// findings render as 5 individual header lines. Also includes a contrast assertion
    /// that the collapse path (render_findings_collapsed) is implemented — verified by
    /// checking that collapse_findings=true produces a different (collapsed) result.
    /// FAILS if collapse_findings=false collapses, or if render_findings_collapsed is not
    /// yet implemented.
    #[test]
    fn test_BC_2_11_028_no_collapse_flag_one_line_per_finding() {
        // BC-2.11.028 pc2: collapse_findings=false → one header per finding, no suffix.
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

        // plain_reporter() has collapse_findings=false.
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

        // Contrast assertion: collapse_findings=true on the same input MUST produce
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
    /// FAILS if a future change makes collapse_findings=true and collapse_findings=false
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

    /// AC-019: --no-collapse flag is wired: no_collapse=true → collapse_findings=false.
    ///
    /// This guards the structural wiring between the CLI flag and the reporter field.
    /// FAILS if the collapse_findings field does not exist, or if its polarity is
    /// reversed (collapse_findings: no_collapse instead of !no_collapse).
    #[test]
    fn test_BC_2_11_028_flag_wired_to_reporter_field() {
        // BC-2.11.028 pc1 / invariant 1: structural wiring test.
        // collapse_findings=true → collapse active (v0.8.0 default).
        // collapse_findings=false → collapse inactive (--no-collapse opt-out).
        // Verify the field exists and the logic is correct by behavioral contrast.

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

        // Reporter with collapse_findings = true → produces "(x3)".
        let reporter_on = TerminalReporter {
            use_color: false,
            show_mitre_grouping: false,
            show_hosts_breakdown: false,
            collapse_findings: true,
        };
        let out_on = reporter_on.render(&Summary::new(), &findings, &[]);
        assert!(
            out_on.contains("(x3)"),
            "BC-2.11.028 inv1: collapse_findings=true must produce '(x3)' suffix; got:\n{out_on}"
        );

        // Reporter with collapse_findings = false → no collapse, no suffix.
        let reporter_off = TerminalReporter {
            use_color: false,
            show_mitre_grouping: false,
            show_hosts_breakdown: false,
            collapse_findings: false,
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
    /// This guards that the collapse_findings field on TerminalReporter does not leak
    /// into the JSON/CSV reporters. FAILS if main.rs is refactored to pre-filter the
    /// findings slice based on the --no-collapse flag before passing to all reporters.
    #[test]
    fn test_BC_2_11_029_no_collapse_flag_json_invariant() {
        use wirerust::reporter::json::JsonReporter;

        // BC-2.11.029 pc5: JSON output must be identical regardless of collapse_findings.
        // The collapse_findings field belongs to TerminalReporter only.
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
        // Both collapse=true and collapse=false terminal renders must produce the SAME
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
    /// regardless of `collapse_findings` flag; 0 (xN) suffixes in any volume.
    ///
    /// This guards the invariant that render_findings_grouped is never modified by
    /// STORY-118. Also includes a contrast assertion that flat collapse produces a
    /// suffix, proving the suffix absence is structural and not because collapse is
    /// globally broken. FAILS if grouped mode emits a suffix, OR if the flat collapse
    /// path (render_findings_collapsed) is not yet implemented.
    #[test]
    fn test_BC_2_11_013_grouped_mode_suffix_free() {
        // BC-2.11.013 invariant 4: grouped mode is structurally suffix-free.
        // 50 identical-key findings with collapse_findings=true AND show_mitre_grouping=true.
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

    /// AC-025: overall section order is unchanged when collapse_findings=true.
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
