//! BC-2.16.016 test suite: ARP Findings Output is Unbounded — No MAX_FINDINGS Cap
//!
//! Exercises BC-2.16.016 Postcondition 4:
//!   The CLI `--help` text for `--arp` MUST document that ARP findings output
//!   is UNBOUNDED (no cap). Operators analyzing adversarial captures with massive
//!   ARP-storm or ARP-spoof events must be informed that findings output can grow
//!   proportionally to the number of triggering frames, without any platform-imposed
//!   bound.
//!
//! Test coverage:
//!   test_BC_2_16_016_cli_help_documents_arp_findings_unbounded
//!     — RED GATE: asserts the word "unbounded" appears in the `--arp` flag's help
//!       text in `wirerust analyze --help`. FAILS before the long_help is added
//!       to `src/cli.rs` (PC-015 doc fix). PASSES after implementation.
//!
//! Canonical pattern: `assert_cmd::Command::cargo_bin("wirerust")` with
//! `["analyze", "--help"]`, capture stdout, find `--arp` entry, assert keyword.
//! This pattern is established by `cli_integration_tests.rs` (mitre_help_text_*
//! tests) and the `no_collapse_help_names_real_output_flags` test.
//!
//! DF-TEST-NAMESPACE-001: all tests wrapped in `mod bc_2_16_016`.
//! DF-AC-TEST-NAME-SYNC-001: function name follows BC-S.SS.NNN convention.
//! DF-CANONICAL-FRAME-HOLDOUT-001: not applicable (CLI test, no frame synthesis).
//!
//! Run via:
//!   cargo test --test bc_2_16_016_arp_tests

#![allow(non_snake_case)]

mod bc_2_16_016 {
    use assert_cmd::Command;

    // -----------------------------------------------------------------------
    // BC-2.16.016 PC4 — RED GATE: --arp help text must document unbounded findings
    // -----------------------------------------------------------------------

    /// BC-2.16.016 Postcondition 4 (RED GATE):
    /// `wirerust analyze --help` must document that ARP findings output is
    /// UNBOUNDED — specifically, the `--arp` flag's help entry must contain
    /// the word "unbounded" (case-insensitive).
    ///
    /// **This test FAILS before the PC-015 implementation** because `src/cli.rs`
    /// lines 194–198 define `--arp` with a short one-line `///` doc-comment that
    /// mentions spoofing and GARP detection but says nothing about findings being
    /// unbounded. The word "unbounded" is absent from the rendered help output.
    ///
    /// **Red Gate assertion**: after running `wirerust analyze --help`, this test
    /// locates the `--arp` flag entry in the output (the text between `--arp` and
    /// the next `--arp-spoof-threshold` sibling flag) and asserts it contains
    /// "unbounded". It will FAIL on current code and PASS after the long_help
    /// is added to the `--arp` arg in `src/cli.rs`.
    ///
    /// BC-2.16.016 reference:
    ///   Postcondition 4: "The CLI `--help` text for `--arp` MUST document the
    ///   absence of a findings cap."
    ///   Invariant 4 (Security awareness): "A malicious pcap with millions of ARP
    ///   spoof or storm events can cause unbounded Vec<Finding> growth..."
    ///   Architecture Anchor: `src/cli.rs` lines 194–213 — `--arp` flag definition;
    ///   `long_help` MUST document unbounded findings behavior (PC-015 doc fix).
    #[test]
    fn test_BC_2_16_016_cli_help_documents_arp_findings_unbounded() {
        let output = Command::cargo_bin("wirerust")
            .expect("wirerust binary must be built — run `cargo build` first")
            .args(["analyze", "--help"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let help = String::from_utf8(output)
            .expect("BC-2.16.016 PC4: wirerust analyze --help output must be valid UTF-8");

        // Locate the `--arp` flag entry in the help output.
        // Clap renders the flag as `--arp` followed by its description on the
        // next indented line(s). We find the text between `--arp` and the next
        // sibling `--arp-spoof-threshold` flag to scope the assertion tightly,
        // preventing a false pass if another flag's description happens to
        // mention "unbounded".
        let arp_flag_pos = help.find("--arp\n").or_else(|| {
            // Clap may render `--arp` with trailing spaces before a newline
            // if the flag has no value description. Try a broader search.
            help.find("--arp ").filter(|&p| {
                // Make sure it's the `--arp` standalone flag, not --arp-spoof-threshold
                // or --arp-storm-rate. Check that it's not followed immediately by '-'.
                help.get(p + 5..)
                    .map(|s| !s.starts_with('-'))
                    .unwrap_or(false)
            })
        });

        let arp_flag_pos = arp_flag_pos.expect(
            "BC-2.16.016 PC4: `--arp` flag must appear in `wirerust analyze --help` output. \
             If this fails, the --arp flag was removed or renamed.",
        );

        // Extract the text from `--arp` to the next sibling `--arp-spoof-threshold`.
        let after_arp = &help[arp_flag_pos..];
        let next_sibling_pos = after_arp.find("--arp-spoof-threshold").expect(
            "BC-2.16.016 PC4: `--arp-spoof-threshold` must appear after `--arp` in help \
             output (used to scope the --arp entry text).",
        );
        let arp_entry = &after_arp[..next_sibling_pos];

        // BC-2.16.016 PC4: the `--arp` help entry must document that findings are unbounded.
        // The word "unbounded" is the canonical keyword specified in BC-2.16.016 PC4
        // and scope.md §PC-015 Fix Classification item 1.
        //
        // RED GATE: this assertion FAILS on the current codebase because `src/cli.rs`
        // `--arp` doc-comment contains:
        //   "Analyze ARP traffic for spoofing, GARP anomalies, malformed frames, and
        //    L2/L3 sender-MAC mismatch. Default-off; included by --all."
        // The word "unbounded" is absent. After the PC-015 doc fix adds a long_help
        // mentioning unbounded findings, this assertion PASSES.
        assert!(
            arp_entry.to_lowercase().contains("unbounded"),
            "BC-2.16.016 PC4 (RED GATE): the `--arp` flag help text (between `--arp` and \
             `--arp-spoof-threshold` in `wirerust analyze --help`) must contain the word \
             'unbounded' to document that ARP findings output is not capped. \
             This is the PC-015 doc fix. Currently FAILING because long_help is absent \
             from `src/cli.rs` lines 194-198. \
             `--arp` help entry:\n{}",
            arp_entry
        );
    }
}
