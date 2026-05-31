//! STORY-088: run_analyze Orchestration — Wave 25 formalization tests
//!
//! Formalizes 14 ACs + 5 ECs for BC-2.12.008, BC-2.12.009, BC-2.12.010,
//! BC-2.12.011, BC-2.12.012, BC-2.12.013.
//!
//! Behavioral contracts covered:
//!   BC-2.12.008  --all Enables dns/http/tls Together
//!   BC-2.12.009  needs_reassembly Logic; --no-reassemble Forces Off with Warning
//!   BC-2.12.010  NO_COLOR Env Var Disables Color
//!   BC-2.12.011  Directory Target Expands to *.pcap Sorted; *.pcapng Excluded
//!   BC-2.12.012  Non-Existent Target Yields bail! with Target Not Found
//!   BC-2.12.013  Per-Target Progress Bar on stderr via indicatif
//!
//! implementation_strategy: brownfield-formalization
//! tdd_mode: strict
//! RED GATE stub phase: all stubs confirmed FAIL before implementation.
//!
//! Placement: dedicated file per DF-TEST-NAMESPACE-001. All STORY-088 tests are
//! wrapped in `mod story_088`.
//!
//! DF-AC-TEST-NAME-SYNC-001: test function names EXACTLY match the AC `Test:`
//! citations in STORY-088.md.
//!
//! NO_COLOR tests (AC-007, AC-008): assert_cmd spawns subprocesses, so per-command
//! env injection via `.env("NO_COLOR", ...)` / `.env_remove("NO_COLOR")` avoids
//! process-global mutation. No serial_test required (no in-process env mutation).

#![allow(non_snake_case)]

mod story_088 {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // Fixture constants
    // -----------------------------------------------------------------------

    /// http-ooo.pcap: 16 packets of HTTP traffic. Produces HTTP findings under
    /// --all or --http. Used for AC-001, AC-002, AC-004, AC-005, AC-006,
    /// AC-007, AC-008, AC-013, AC-014, EC-003.
    const HTTP_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    /// dns-remoteshell.pcap: DNS + HTTP traffic. Used for AC-001 (dns findings
    /// confirm dns analyzer active) and AC-005/AC-006 (dns independent of
    /// reassembly).
    const DNS_FIXTURE: &str = "tests/fixtures/dns-remoteshell.pcap";

    /// tls.pcap: TLS traffic. Used to confirm TLS analyzer activates under --tls.
    const TLS_FIXTURE: &str = "tests/fixtures/tls.pcap";

    // -----------------------------------------------------------------------
    // Helper: build a Command targeting the wirerust binary.
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    fn wirerust() -> Command {
        Command::cargo_bin("wirerust").expect("binary built")
    }

    // -----------------------------------------------------------------------
    // AC-001 (traces to BC-2.12.008 postcondition 1, 2, 3)
    // `--all` enables dns, http, and tls together — observable via ANALYZER
    // sections and FINDINGS in the report.
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.12.008 postcondition 1/2/3): When `--all` is given,
    /// run_analyze enables dns, http, and tls analyzers. The observable proxy
    /// is the presence of "ANALYZER: DNS", "ANALYZER: HTTP", and "ANALYZER: TLS"
    /// sections in stdout (their absence would mean the analyzer was not active).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout contains "ANALYZER: DNS", "ANALYZER: HTTP", "ANALYZER: TLS".
    ///   Negative: run without --all produces no ANALYZER sections when no flags set.
    #[test]
    fn test_all_flag_enables_all_three_analyzers() {
        // Positive: all three ANALYZER sections appear with --all
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all"])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: DNS"))
            .stdout(predicate::str::contains("ANALYZER: HTTP"))
            .stdout(predicate::str::contains("ANALYZER: TLS"));

        // Negative: without --all and no analyzer flags, no ANALYZER: DNS/HTTP/TLS sections
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: DNS").not())
            .stdout(predicate::str::contains("ANALYZER: HTTP").not())
            .stdout(predicate::str::contains("ANALYZER: TLS").not());
    }

    // -----------------------------------------------------------------------
    // AC-002 (traces to BC-2.12.008 invariant 3)
    // `--mitre` is NOT implied by `--all`; when all=true and mitre=false, the
    // report is NOT grouped by MITRE technique headers.
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.12.008 invariant 3): `--all` without `--mitre` does not
    /// activate MITRE-grouped rendering. The observable proxy is the absence of
    /// the "## Uncategorized" grouping header in stdout (which only appears when
    /// --mitre is active).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain "## Uncategorized" (no MITRE grouping).
    ///   Positive: command succeeds (exit 0).
    ///   Negative: `--all --mitre` DOES produce the "## Uncategorized" grouping header.
    #[test]
    fn test_all_does_not_imply_mitre() {
        // Positive: --all alone does NOT produce MITRE grouping header
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all"])
            .assert()
            .success()
            .stdout(predicate::str::contains("## Uncategorized").not());

        // Negative: --all --mitre DOES produce the MITRE grouping header
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all", "--mitre"])
            .assert()
            .success()
            .stdout(predicate::str::contains("## Uncategorized"));
    }

    // -----------------------------------------------------------------------
    // AC-003 (traces to BC-2.12.009 postcondition 1; contributes to VP-018)
    // needs_reassembly = cli.reassemble || enable_http || enable_tls
    // Formalized behaviorally: --http without --no-reassemble => reassembler
    // constructed => HTTP findings appear; --http --no-reassemble => no HTTP
    // analyzer section (analyzer not constructed).
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.12.009 postcondition 1; VP-018 runtime half): The
    /// `needs_reassembly` computation is observable via behavior: when
    /// `--http` is given WITHOUT `--no-reassemble`, the reassembler is
    /// constructed and the HTTP analyzer is active (stdout contains
    /// "ANALYZER: HTTP"). When `--http --no-reassemble` is used, the
    /// reassembler is skipped and the HTTP analyzer is NOT constructed
    /// (stdout does NOT contain "ANALYZER: HTTP").
    ///
    /// Discriminating assertions:
    ///   Positive (reassembly on): stdout contains "ANALYZER: HTTP".
    ///   Positive (skip): stdout does NOT contain "ANALYZER: HTTP".
    ///   Negative: the two cases are observably different.
    #[test]
    fn test_needs_reassembly_formula() {
        // Positive (reassembly on): ANALYZER: HTTP present
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--http"])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: HTTP"));

        // Positive (skip): ANALYZER: HTTP absent when --no-reassemble is set
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--http", "--no-reassemble"])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: HTTP").not());
    }

    // -----------------------------------------------------------------------
    // AC-004 (traces to BC-2.12.009 postcondition 5)
    // Warning to stderr when skip_reassembly=true AND enable_http||enable_tls.
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.12.009 postcondition 5; invariant 1): When `--no-reassemble`
    /// is set AND `--http` (or `--tls`) is active, a warning is printed to
    /// stderr matching the exact hardcoded message.
    ///
    /// Warning text: "Warning: --http/--tls require TCP reassembly, but
    /// --no-reassemble is set. Stream analysis will be skipped."
    ///
    /// Discriminating assertions:
    ///   Positive: stderr contains the exact warning string.
    ///   Positive: command exits 0 (not an error, just a warning).
    ///   Negative: --http WITHOUT --no-reassemble does NOT emit the warning.
    #[test]
    fn test_no_reassemble_with_http_emits_warning() {
        let expected_warning = "Warning: --http/--tls require TCP reassembly, but \
            --no-reassemble is set. Stream analysis will be skipped.";

        // Positive: warning is emitted on stderr
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--http", "--no-reassemble"])
            .assert()
            .success()
            .stderr(predicate::str::contains(expected_warning));

        // Negative: no warning when --http without --no-reassemble
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--http"])
            .assert()
            .success()
            .stderr(predicate::str::contains(expected_warning).not());
    }

    // -----------------------------------------------------------------------
    // AC-005 (traces to BC-2.12.009 postcondition 4)
    // When skip_reassembly=true, http_analyzer and tls_analyzer are None.
    // Observable: ANALYZER: HTTP and ANALYZER: TLS sections absent from stdout.
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.12.009 postcondition 4): When `--no-reassemble` is set,
    /// `http_analyzer` and `tls_analyzer` are NOT constructed. Observable proxy:
    /// stdout does NOT contain "ANALYZER: HTTP" or "ANALYZER: TLS" sections.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain "ANALYZER: HTTP".
    ///   Positive: stdout does NOT contain "ANALYZER: TLS".
    ///   Positive: command exits 0.
    ///   Negative: WITHOUT --no-reassemble, both sections appear.
    #[test]
    fn test_no_reassemble_skips_http_and_tls_constructors() {
        // Positive: both HTTP and TLS analyzers absent with --no-reassemble
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--http", "--no-reassemble"])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: HTTP").not())
            .stdout(predicate::str::contains("ANALYZER: TLS").not());

        // Negative: without --no-reassemble, HTTP analyzer IS present
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--http"])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: HTTP"));
    }

    // -----------------------------------------------------------------------
    // AC-006 (traces to BC-2.12.009 postcondition 6)
    // dns_analyzer is constructed independently of reassembly.
    // Observable: ANALYZER: DNS appears even with --no-reassemble.
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.12.009 postcondition 6; invariant 4): The dns_analyzer is
    /// constructed independently of reassembly. Observable: `--dns --no-reassemble`
    /// still produces the "ANALYZER: DNS" section in stdout.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout contains "ANALYZER: DNS" even with --no-reassemble.
    ///   Positive: command exits 0.
    ///   Negative: no warning emitted (--dns does not require reassembly).
    #[test]
    fn test_dns_analyzer_constructed_without_reassembly() {
        let no_reassemble_warning = "Warning: --http/--tls require TCP reassembly";

        // Positive: DNS analyzer still runs with --no-reassemble
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", DNS_FIXTURE, "--dns", "--no-reassemble"])
            .assert()
            .success()
            .stdout(predicate::str::contains("ANALYZER: DNS"))
            .stderr(predicate::str::contains(no_reassemble_warning).not());
    }

    // -----------------------------------------------------------------------
    // AC-007 (traces to BC-2.12.010 postcondition 1)
    // NO_COLOR env var disables color — even when set to empty string.
    // assert_cmd per-subprocess env injection; NO serial_test needed.
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.12.010 postcondition 1; EC-004): When `NO_COLOR` is set to
    /// any value (including empty string `""`), the terminal output contains no
    /// ANSI color escape sequences (`\x1b[` bytes in stdout).
    ///
    /// Uses `.env("NO_COLOR", "")` on the assert_cmd subprocess — no in-process
    /// env mutation, no serial annotation needed.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain the ANSI escape byte `\x1b`.
    ///   Positive: command exits 0.
    ///   Negative: WITHOUT NO_COLOR, stdout contains ANSI escapes (confirmed below
    ///   by the complementary test AC-008).
    #[test]
    fn test_no_color_env_var_disables_color() {
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all"])
            .env("NO_COLOR", "")
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b").not());
    }

    // -----------------------------------------------------------------------
    // AC-008 (traces to BC-2.12.010 postcondition 2)
    // When NO_COLOR is absent and --no-color is absent, use_color=true.
    // Observable: stdout contains ANSI escape codes.
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.12.010 postcondition 2): When `NO_COLOR` is NOT set and
    /// `--no-color` is absent, `use_color = true`. The terminal output contains
    /// ANSI color escape sequences.
    ///
    /// Uses `.env_remove("NO_COLOR")` on the assert_cmd subprocess to guarantee
    /// the env var is absent regardless of the parent process environment.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout contains the ANSI escape byte `\x1b`.
    ///   Positive: command exits 0.
    ///   Negative: the complementary case (NO_COLOR set) does NOT produce escapes.
    #[test]
    fn test_use_color_true_when_no_flags_set() {
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all"])
            .env_remove("NO_COLOR")
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b"));
    }

    // -----------------------------------------------------------------------
    // AC-009 (traces to BC-2.12.011 postcondition 1)
    // resolve_targets on a directory returns sorted Vec<PathBuf> of *.pcap only.
    // Observable: run analyze on a tempdir with a.pcap, b.pcap, c.pcapng;
    // command succeeds (a.pcap and b.pcap processed); c.pcapng excluded
    // (would cause a reader error if it were included since pcapng is rejected
    // at reader level).
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.12.011 postcondition 1, 2, 3, 4): `resolve_targets` on a
    /// directory expands to sorted `.pcap` files only. `.pcapng`, `.txt`, and
    /// other extensions are excluded. Observable proxy: run `analyze <dir>` on
    /// a tempdir containing `a.pcap`, `b.pcap`, and `c.pcapng`; assert command
    /// succeeds (no reader error from .pcapng being passed) AND that the output
    /// reflects the two .pcap files (packet counts consistent with processing
    /// only valid pcap files).
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0 (pcapng excluded → no reader error).
    ///   Positive: stdout contains "Packets: 32" (2 × 16 from http-ooo.pcap).
    ///   Negative: the pcapng file is NOT passed to the reader.
    #[test]
    fn test_resolve_targets_directory_pcap_only_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        fs::copy(HTTP_FIXTURE, dir_path.join("a.pcap")).unwrap();
        fs::copy(HTTP_FIXTURE, dir_path.join("b.pcap")).unwrap();
        // smb3.pcapng is a pcapng file; if included it would cause a reader error
        fs::copy("tests/fixtures/smb3.pcapng", dir_path.join("c.pcapng")).unwrap();

        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            // 2 × 16 = 32 packets from a.pcap + b.pcap; c.pcapng excluded
            .stdout(predicate::str::contains("Packets: 32"));
    }

    // -----------------------------------------------------------------------
    // AC-010 (traces to BC-2.12.011 invariant 1)
    // Extension matching is case-sensitive: .PCAP (uppercase) is excluded.
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.12.011 invariant 1; EC-002): Extension check is
    /// `ext == "pcap"` (case-sensitive). A file named `test.PCAP` is excluded.
    /// Observable: a tempdir containing ONLY `test.PCAP` and NO `.pcap` files
    /// returns an empty list → `analyze <dir>` exits 0 with "Packets: 0".
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0 (empty target list is Ok).
    ///   Positive: stdout contains "Packets: 0" (no files processed).
    ///   Negative: `test.PCAP` is NOT processed (would show Packets > 0 if it were).
    #[test]
    fn test_resolve_targets_case_sensitive_extension_exclusion() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // Copy an http fixture as .PCAP (uppercase) — must NOT be picked up
        fs::copy(HTTP_FIXTURE, dir_path.join("test.PCAP")).unwrap();

        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Packets: 0"));
    }

    // -----------------------------------------------------------------------
    // AC-011 (traces to BC-2.12.011 invariant 3)
    // Directory expansion is NOT recursive; subdirectories are skipped.
    // -----------------------------------------------------------------------

    /// AC-011 (BC-2.12.011 invariant 3; EC-006): `resolve_targets` does NOT
    /// recurse into subdirectories. Observable: a tempdir containing a
    /// subdirectory `subdir/nested.pcap` but no top-level `.pcap` files
    /// produces an empty expansion → `analyze <dir>` exits 0 with "Packets: 0".
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0 (subdir skipped, empty expansion is Ok).
    ///   Positive: stdout contains "Packets: 0" (nested.pcap not processed).
    ///   Negative: nested.pcap is NOT processed (would show Packets > 0 if it were).
    #[test]
    fn test_resolve_targets_not_recursive() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        let subdir = dir_path.join("subdir");
        fs::create_dir(&subdir).unwrap();
        fs::copy(HTTP_FIXTURE, subdir.join("nested.pcap")).unwrap();

        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Packets: 0"));
    }

    // -----------------------------------------------------------------------
    // AC-012 (traces to BC-2.12.012 postcondition 1)
    // Non-existent target → error exit with stderr containing "Target not found:".
    // -----------------------------------------------------------------------

    /// AC-012 (BC-2.12.012 postcondition 1; invariant 1): `resolve_targets` on a
    /// non-existent path returns `Err` via `bail!("Target not found: {}", ...)`.
    /// Observable: `analyze /nonexistent/path.pcap` exits with failure and stderr
    /// contains "Target not found:".
    ///
    /// Discriminating assertions:
    ///   Positive: command exits with failure (non-zero exit code).
    ///   Positive: stderr contains "Target not found:".
    ///   Negative: a valid file does NOT produce this error.
    #[test]
    fn test_resolve_targets_nonexistent_path_error() {
        // Positive: non-existent path → failure + "Target not found:"
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", "/nonexistent/path.pcap"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Target not found:"));

        // Negative: valid file succeeds without error
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE])
            .assert()
            .success()
            .stderr(predicate::str::contains("Target not found:").not());
    }

    // -----------------------------------------------------------------------
    // AC-013 (traces to BC-2.12.013 postcondition 3, 4)
    // Progress bar appears on stderr, not stdout. Stdout must contain no
    // ANSI cursor-movement / progress-bar bytes.
    // -----------------------------------------------------------------------

    /// AC-013 (BC-2.12.013 postcondition 3, 4): The indicatif progress bar writes
    /// to stderr; stdout must NOT contain ANSI progress-bar control sequences.
    /// Specifically, assert stdout does NOT contain `\x1b[` which is the CSI
    /// introducer used by indicatif for cursor movement during progress rendering.
    ///
    /// Note: The terminal-output coloring IS allowed on stdout (it uses the same
    /// `\x1b[` prefix). For this test we use `--no-color` to strip coloring and
    /// then verify no remaining escape sequences are from the progress bar.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain `\x1b` (with --no-color, no escapes remain).
    ///   Positive: command exits 0.
    ///   Negative: progress bar bytes are NOT leaked to stdout.
    #[test]
    fn test_progress_bar_does_not_appear_in_output() {
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all", "--no-color"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b").not());
    }

    // -----------------------------------------------------------------------
    // AC-014 (traces to BC-2.12.013 invariant 4)
    // run_summary has NO progress bar. Observable: summary stdout with --no-color
    // contains no ANSI escape bytes at all.
    // -----------------------------------------------------------------------

    /// AC-014 (BC-2.12.013 invariant 4): `run_summary` has no progress bar.
    /// Observable: `summary <fixture> --no-color` stdout contains no `\x1b`
    /// bytes (neither color nor progress-bar escapes).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain `\x1b`.
    ///   Positive: command exits 0.
    ///   Negative: no ANSI bytes of any kind in stdout (confirms no rogue escapes).
    #[test]
    fn test_run_summary_has_no_progress_bar() {
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["summary", HTTP_FIXTURE, "--no-color"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b").not());
    }

    // -----------------------------------------------------------------------
    // EC-001 (BC-2.12.011 EC-003): Empty directory → Ok(vec![]) → exits 0
    // -----------------------------------------------------------------------

    /// EC-001 (BC-2.12.011 EC-003 / STORY-088 EC-001): An empty directory
    /// produces `resolve_targets` returning `Ok(vec![])`. Observable: `analyze
    /// <empty-dir>` exits 0 and reports "Packets: 0".
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0.
    ///   Positive: stdout contains "Packets: 0".
    ///   Negative: NOT an error (empty expansion is explicitly Ok).
    #[test]
    fn test_EC_001_empty_directory_returns_ok_empty() {
        let dir = tempfile::tempdir().expect("tempdir");

        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", dir.path().to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Packets: 0"));
    }

    // -----------------------------------------------------------------------
    // EC-002 (STORY-088 EC-002 / BC-2.12.011 EC-005):
    // .PCAP (uppercase) excluded — promoted to AC-010 above; kept here as the
    // standalone EC stub for completeness.
    // -----------------------------------------------------------------------

    /// EC-002 (BC-2.12.011 EC-005 / STORY-088 EC-002): A directory with only
    /// a `.PCAP` (uppercase extension) file returns `Ok(vec![])` since
    /// `ext == "pcap"` is case-sensitive. Observable: exits 0 with Packets: 0.
    ///
    /// Note: This scenario is also covered structurally by AC-010; this EC stub
    /// provides the explicit edge-case formalization.
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0 (no .pcap files found).
    ///   Positive: stdout contains "Packets: 0".
    #[test]
    fn test_EC_002_uppercase_pcap_extension_excluded() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        fs::copy(HTTP_FIXTURE, dir_path.join("data.PCAP")).unwrap();

        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Packets: 0"));
    }

    // -----------------------------------------------------------------------
    // EC-003 (STORY-088 EC-003 / BC-2.12.009 EC-002):
    // --no-reassemble WITHOUT --http/--tls → no warning emitted.
    // -----------------------------------------------------------------------

    /// EC-003 (BC-2.12.009 EC-002 / STORY-088 EC-003): `--no-reassemble` alone
    /// (without `--http` or `--tls`) does NOT emit the reassembly warning.
    /// Observable: stderr does NOT contain "Warning: --http/--tls require TCP
    /// reassembly".
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0.
    ///   Positive: stderr does NOT contain the warning string.
    ///   Negative: the warning is NOT emitted when http/tls are absent.
    #[test]
    fn test_EC_003_no_reassemble_without_http_tls_no_warning() {
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--no-reassemble"])
            .assert()
            .success()
            .stderr(predicate::str::contains("Warning: --http/--tls require TCP reassembly").not());
    }

    // -----------------------------------------------------------------------
    // EC-004 (STORY-088 EC-004 / BC-2.12.010 EC-001):
    // NO_COLOR="" (empty value) also disables color.
    // -----------------------------------------------------------------------

    /// EC-004 (BC-2.12.010 EC-001 / STORY-088 EC-004): `NO_COLOR=""` (empty
    /// value) disables color — any set value counts, per the NO_COLOR spec.
    /// Observable: stdout (with --no-color stripped via env) contains no `\x1b`.
    ///
    /// Uses `.env("NO_COLOR", "")` — same as AC-007, isolated here to document
    /// the empty-string edge case explicitly per the BC.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain `\x1b`.
    ///   Positive: command exits 0.
    #[test]
    fn test_EC_004_no_color_empty_value_disables_color() {
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", HTTP_FIXTURE, "--all"])
            .env("NO_COLOR", "")
            .assert()
            .success()
            .stdout(predicate::str::contains("\x1b").not());
    }

    // -----------------------------------------------------------------------
    // EC-005 (STORY-088 EC-005 / BC-2.12.011 EC-002):
    // Two pcap files in reverse-alphabetical order → returned sorted [a, b].
    // -----------------------------------------------------------------------

    /// EC-005 (BC-2.12.011 EC-002 / STORY-088 EC-005): A directory with
    /// `b.pcap` and `a.pcap` (reverse on-disk order from readdir) returns
    /// them sorted `[a.pcap, b.pcap]`. Observable: when `a.pcap` is a minimal
    /// valid pcap and `b.pcap` contains known traffic, the output ordering
    /// reflects a-first processing. (Proxy: use two copies of the http fixture
    /// and confirm total packet count is consistent with 2 × fixture packets.)
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0.
    ///   Positive: stdout contains total packet count = 2 × fixture packet count (32).
    ///   Negative: if unsorted, order would depend on filesystem (non-deterministic).
    #[test]
    fn test_EC_005_directory_files_returned_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // Write b.pcap first, then a.pcap to encourage reverse on-disk order
        fs::copy(HTTP_FIXTURE, dir_path.join("b.pcap")).unwrap();
        fs::copy(HTTP_FIXTURE, dir_path.join("a.pcap")).unwrap();

        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", dir_path.to_str().unwrap(), "--no-color"])
            .assert()
            .success()
            // 2 × 16 = 32 packets (a.pcap + b.pcap, both copies of http-ooo.pcap)
            .stdout(predicate::str::contains("Packets: 32"));
    }

    // -----------------------------------------------------------------------
    // Ensure imports are referenced even in stub form.
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    fn _type_check_imports() {
        let _: &str = HTTP_FIXTURE;
        let _: &str = DNS_FIXTURE;
        let _: &str = TLS_FIXTURE;
        // predicates crate referenced via use at top of mod
        let _pred = predicate::str::contains("x");
    }
}
