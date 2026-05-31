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
    // Observable: ANALYZER: DNS appears even with --no-reassemble, AND the
    // DNS analyzer produces NON-ZERO activity (dns_queries > 0) confirming
    // per-packet analysis ran — not merely that the section header was emitted.
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.12.009 postcondition 6; invariant 4): The dns_analyzer is
    /// constructed independently of reassembly and performs per-packet analysis.
    /// Observable: `--dns --no-reassemble` on dns-remoteshell.pcap produces:
    ///   1. The "ANALYZER: DNS" section header in stdout.
    ///   2. "dns_queries: 6" — non-zero, confirming actual DNS packet analysis
    ///      occurred (dns-remoteshell.pcap contains 6 DNS queries / 12 responses).
    ///
    /// This strengthened form catches the adversarial mutation: if DNS per-packet
    /// analysis were gated behind `!skip_reassembly`, dns_queries would be 0 and
    /// the `dns_queries: 6` assertion would fail.
    ///
    /// Fixture: tests/fixtures/dns-remoteshell.pcap — confirmed to yield
    /// `dns_queries: 6` under `--dns --no-reassemble` (verified with binary run).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout contains "ANALYZER: DNS" (section header present).
    ///   Positive: stdout contains "dns_queries: 6" (non-zero — analyzer ran).
    ///   Positive: command exits 0.
    ///   Negative: no reassembly warning emitted (--dns does not require reassembly).
    #[test]
    fn test_dns_analyzer_constructed_without_reassembly() {
        let no_reassemble_warning = "Warning: --http/--tls require TCP reassembly";

        // Positive: DNS analyzer runs WITH --no-reassemble, producing non-zero
        // dns_queries from the dns-remoteshell.pcap fixture (6 DNS queries confirmed).
        Command::cargo_bin("wirerust")
            .unwrap()
            .args(["analyze", DNS_FIXTURE, "--dns", "--no-reassemble"])
            .assert()
            .success()
            // Section header present → analyzer was constructed
            .stdout(predicate::str::contains("ANALYZER: DNS"))
            // Non-zero query count → per-packet analysis actually executed,
            // not gated behind reassembly. If mutated to `if !skip_reassembly
            // && enable_dns`, this would read "dns_queries: 0" and fail here.
            .stdout(predicate::str::contains("dns_queries: 6"))
            // No reassembly warning → --dns correctly does not require reassembly
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
    // Observable: run analyze on a tempdir with a.pcap (http.pcap, 1 packet),
    // z.pcap (http-ooo.pcap, 16 packets), and c.pcapng (excluded); assert:
    //   1. command succeeds (pcapng excluded → no reader error)
    //   2. sort order is observable via recent_uris in --json output:
    //      sorted order (a first) → iuident URI appears before /1, /2...
    //      unsorted/reversed (z first) → /1 appears before iuident URI
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.12.011 postcondition 1, 2, 3, 4): `resolve_targets` on a
    /// directory expands to sorted `.pcap` files only and processes them in
    /// sorted alphabetical order. Observable via two mechanisms:
    ///
    ///   1. pcapng-excluded safety: `c.pcapng` excluded → command exits 0 (no
    ///      reader error that would occur if pcapng were passed to the reader).
    ///
    ///   2. Sort-order observable via `--all --json` recent_uris: the HTTP
    ///      analyzer accumulates URIs in processing order. With sorted order
    ///      (a.pcap first), `/v4/iuident.cab?0307011208` (from http.pcap, 1 pkt)
    ///      appears BEFORE `/1` (from http-ooo.pcap, 16 pkts) in recent_uris.
    ///      If `files.sort()` is removed and z.pcap is iterated first (observed
    ///      on macOS APFS when z is created before a), `/1` would appear first
    ///      and the iuident URI assertion order would fail.
    ///
    /// Fixtures:
    ///   a.pcap ← http.pcap       (1 packet;  HTTP HEAD /v4/iuident.cab...)
    ///   z.pcap ← http-ooo.pcap   (16 packets; HTTP PUT/GET /1.../5)
    ///   c.pcapng ← smb3.pcapng   (pcapng; must be excluded)
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0 (pcapng excluded → no reader error).
    ///   Positive: stdout contains "Packets: 17" (1+16; pcapng excluded).
    ///   Positive: stdout (JSON) contains iuident URI before /1 (sort verified).
    ///   Negative: the pcapng file is NOT passed to the reader.
    #[test]
    fn test_resolve_targets_directory_pcap_only_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // Create z.pcap BEFORE a.pcap so that without sort(), read_dir would
        // return [z.pcap, a.pcap] on typical macOS APFS (creation/inode order).
        // With files.sort(), the order is always [a.pcap, z.pcap].
        fs::copy("tests/fixtures/http-ooo.pcap", dir_path.join("z.pcap")).unwrap();
        fs::copy("tests/fixtures/http.pcap", dir_path.join("a.pcap")).unwrap();
        // smb3.pcapng is a pcapng file; if included it would cause a reader error
        fs::copy("tests/fixtures/smb3.pcapng", dir_path.join("c.pcapng")).unwrap();

        let output = Command::cargo_bin("wirerust")
            .unwrap()
            .args([
                "analyze",
                dir_path.to_str().unwrap(),
                "--no-color",
                "--all",
                "--json",
            ])
            .assert()
            .success()
            // 1 (http.pcap) + 16 (http-ooo.pcap) = 17; c.pcapng excluded
            .stdout(predicate::str::contains("\"total_packets\": 17"))
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8_lossy(&output);

        // Sort-order assertion: in sorted order (a.pcap first), the iuident URI
        // from http.pcap appears in recent_uris BEFORE the /1 URI from http-ooo.pcap.
        // If files.sort() is removed, z.pcap (http-ooo.pcap) is iterated first on
        // macOS APFS (created first above), so /1 appears before the iuident URI.
        let iuident_pos = stdout
            .find("/v4/iuident.cab")
            .expect("iuident URI must appear in recent_uris (http.pcap processed)");
        let slash1_pos = stdout
            .find("\"/1\"")
            .expect("/1 URI must appear in recent_uris (http-ooo.pcap processed)");

        assert!(
            iuident_pos < slash1_pos,
            "Sort order violated: /v4/iuident.cab (from a.pcap) must appear before /1 \
            (from z.pcap) in recent_uris; got iuident_pos={iuident_pos}, slash1_pos={slash1_pos}. \
            This fires when files.sort() is removed from resolve_targets."
        );
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
    /// // LIMITATION: indicatif renders the progress bar ONLY when stderr is a TTY.
    /// // Under assert_cmd's piped (non-TTY) stderr, indicatif emits nothing to stderr
    /// // at all — the bar is completely suppressed. As a result, this test CANNOT
    /// // verify that the bar is wired to stderr, that finish_and_clear() is called,
    /// // or that the bar does not render to a real terminal. These aspects of
    /// // BC-2.12.013 (postconditions 3/4) are LOW-confidence — they are true in
    /// // production (TTY) but invisible to a subprocess test harness.
    /// //
    /// // What this test DOES verify — the real user-facing contract: stdout contains
    /// // NO progress/spinner ANSI artifacts regardless of how the bar behaves on
    /// // stderr. This is the observable guarantee: piped/redirected output is
    /// // always clean of escape sequences when --no-color is set.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain `\x1b` (with --no-color, no escapes remain).
    ///   Positive: command exits 0.
    ///   Positive: stdout is clean of ALL escape bytes (progress leak impossible in piped mode).
    #[test]
    fn test_progress_bar_does_not_appear_in_output() {
        // LIMITATION: indicatif renders only to a TTY stderr; under assert_cmd's
        // piped non-TTY stderr the bar emits nothing, so this test verifies the
        // observable stdout-cleanliness guarantee, NOT the stderr placement /
        // finish_and_clear() detail (BC-2.12.013 is LOW-confidence per the BC).
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
    /// // LIMITATION: indicatif renders the progress bar ONLY when stderr is a TTY.
    /// // Under assert_cmd's piped (non-TTY) stderr, indicatif emits nothing to stderr
    /// // at all — the bar is completely suppressed. As a result, this test CANNOT
    /// // verify that run_summary truly lacks a progress bar on a real TTY, or that
    /// // a future developer could not accidentally add one without this test catching
    /// // it in a non-TTY environment.
    /// //
    /// // What this test DOES verify — the real user-facing contract: run_summary's
    /// // stdout contains NO ANSI escape bytes when --no-color is set. This confirms
    /// // the stdout-cleanliness guarantee for piped/redirected usage. The absence of
    /// // a progress bar on stderr in production (TTY) is an architectural constraint
    /// // (run_summary doesn't call ProgressBar) that must be verified by code review,
    /// // not this subprocess test (BC-2.12.013 invariant 4 is LOW-confidence here).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain `\x1b`.
    ///   Positive: command exits 0.
    ///   Positive: stdout is clean of ALL escape bytes (color and progress-bar alike).
    #[test]
    fn test_run_summary_has_no_progress_bar() {
        // LIMITATION: indicatif renders only to a TTY stderr; under assert_cmd's
        // piped non-TTY stderr the bar emits nothing, so this test verifies the
        // observable stdout-cleanliness guarantee, NOT the stderr placement /
        // finish_and_clear() detail (BC-2.12.013 is LOW-confidence per the BC).
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
    // Two distinct-content pcap files written in reverse-alphabetical creation
    // order → resolve_targets must return them sorted [a, b]. Observable via
    // JSON recent_uris ordering: a.pcap (http.pcap, iuident URI) must appear
    // before b.pcap (http-ooo.pcap, /1.../5 URIs) in the HTTP analyzer output.
    // -----------------------------------------------------------------------

    /// EC-005 (BC-2.12.011 EC-002 / STORY-088 EC-005): A directory with
    /// `b.pcap` (http-ooo.pcap, 16 pkts) written BEFORE `a.pcap` (http.pcap,
    /// 1 pkt) returns them sorted `[a.pcap, b.pcap]`. Observable: the HTTP
    /// analyzer's `recent_uris` in JSON output reflects a-first processing —
    /// `/v4/iuident.cab?0307011208` (from a.pcap) appears before `/1` (from
    /// b.pcap). Without `files.sort()`, on macOS APFS b.pcap is iterated first
    /// (created first / lower inode), `/1` appears before the iuident URI, and
    /// the position assertion fails.
    ///
    /// Fixtures:
    ///   b.pcap ← http-ooo.pcap  (created first; HTTP PUT/GET /1.../5)
    ///   a.pcap ← http.pcap      (created second; HTTP HEAD /v4/iuident.cab...)
    ///
    /// Discriminating assertions:
    ///   Positive: command exits 0.
    ///   Positive: stdout (JSON) contains iuident URI BEFORE /1 (a-first order).
    ///   Negative: if sort removed, b-first iteration puts /1 before iuident.
    #[test]
    fn test_EC_005_directory_files_returned_sorted() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();

        // Write b.pcap (http-ooo.pcap) FIRST so that without sort(), read_dir
        // returns [b.pcap, a.pcap] on typical macOS APFS (creation/inode order).
        // With files.sort(), the order is always [a.pcap, b.pcap].
        fs::copy("tests/fixtures/http-ooo.pcap", dir_path.join("b.pcap")).unwrap();
        fs::copy("tests/fixtures/http.pcap", dir_path.join("a.pcap")).unwrap();

        let output = Command::cargo_bin("wirerust")
            .unwrap()
            .args([
                "analyze",
                dir_path.to_str().unwrap(),
                "--no-color",
                "--all",
                "--json",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();

        let stdout = String::from_utf8_lossy(&output);

        // Sort-order assertion: in sorted order (a.pcap first), the iuident URI
        // from http.pcap appears in recent_uris BEFORE the /1 URI from http-ooo.pcap.
        // If files.sort() is removed, b.pcap (http-ooo.pcap) is iterated first
        // on macOS APFS, so /1 appears before the iuident URI.
        let iuident_pos = stdout
            .find("/v4/iuident.cab")
            .expect("iuident URI must appear in recent_uris (http.pcap processed)");
        let slash1_pos = stdout
            .find("\"/1\"")
            .expect("/1 URI must appear in recent_uris (http-ooo.pcap processed)");

        assert!(
            iuident_pos < slash1_pos,
            "Sort order violated: /v4/iuident.cab (from a.pcap) must appear before /1 \
            (from b.pcap) in recent_uris; got iuident_pos={iuident_pos}, slash1_pos={slash1_pos}. \
            This fires when files.sort() is removed from resolve_targets."
        );
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
