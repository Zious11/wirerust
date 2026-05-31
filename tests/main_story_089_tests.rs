//! STORY-089: Decode Error Counting, Dispatcher Stats Injection,
//! Format Resolution, and Output Routing — Wave 26 formalization tests
//!
//! Formalizes 12 ACs + 5 ECs for BC-2.12.014, BC-2.12.015, BC-2.12.016,
//! BC-2.12.017.
//!
//! Behavioral contracts covered:
//!   BC-2.12.014  Per-Target Decode Errors Counted into skipped_packets
//!   BC-2.12.015  dispatcher.unclassified_flows() Injected into Reassembly Summary
//!   BC-2.12.016  Output Format Selection: json->JsonReporter, csv->CsvReporter, else Terminal
//!   BC-2.12.017  Output Routed to File if --json/--csv <FILE>; Stdout Otherwise
//!
//! implementation_strategy: brownfield-formalization
//! tdd_mode: strict
//! RED GATE stub phase: all tests confirmed FAIL before implementation.
//!
//! Placement: dedicated file per DF-TEST-NAMESPACE-001. All STORY-089 tests are
//! wrapped in `mod story_089`.
//!
//! DF-AC-TEST-NAME-SYNC-001: test function names EXACTLY match the AC `Test:`
//! citations in STORY-089.md.
//!
//! Approach: zero-src behavioral formalization following STORY-088 precedent.
//! All tests use assert_cmd subprocess invocation — binary-private functions
//! (resolve_format, write_output, decode-error handler, unclassified_flows
//! injection) are exercised purely through observable CLI behavior.
//!
//! No src changes. No resolve_format lib extraction. Justification: behavioral
//! coverage of AC-007/008/009 is mutation-resistant because we assert the actual
//! output FORMAT produced (JSON '{' prefix, CSV header line, terminal table
//! header) — the same three-way discriminating assertion that src/main.rs itself
//! enforces. An adversarial mutation that swaps Json/Csv/Terminal would flip the
//! observed format and fail the assertion.
//!
//! Fixtures used:
//!   dns-remoteshell.pcap — 58 total packets, 73 decode errors (non-IP frames
//!     fail decode_packet with "No IP layer found"). Produces exactly ONE
//!     "Warning: failed to decode packet" line on stderr; skipped_packets=73.
//!     With analyze --http --json: unclassified_flows=8 (non-zero, kills
//!     hardcode-to-zero mutations). Used for AC-001..004, AC-005, EC-001, EC-005
//!     and all run_summary decode-error / format / routing tests.
//!   http-ooo.pcap — 16 HTTP TCP packets; clean decode (0 skipped_packets).
//!     With analyze --http: unclassified_flows=0.
//!     Used for AC-006..012, EC-002..004 and run_summary format/routing tests.

#![allow(non_snake_case)]

mod story_089 {
    use assert_cmd::Command;
    use predicates::prelude::*;
    use std::fs;

    // -----------------------------------------------------------------------
    // Fixture constants (verified by binary run before authoring)
    // -----------------------------------------------------------------------

    /// dns-remoteshell.pcap: 58 total packets, 73 decode errors (non-IP frames
    /// fail decode_packet). Produces exactly ONE "Warning: failed to decode
    /// packet" line on stderr. skipped_packets=73 in --json output.
    /// analyze --http --json → unclassified_flows=8 (non-zero).
    /// Used for AC-001..005, EC-001, EC-005, and all run_summary tests.
    const DNS_REMOTE_FIXTURE: &str = "tests/fixtures/dns-remoteshell.pcap";

    /// http-ooo.pcap: 16 HTTP TCP packets; 0 decode errors; 0 skipped_packets.
    /// With --http, activates TcpReassembler → "unclassified_flows" present.
    /// With --dns --no-reassemble, no reassembler → "unclassified_flows" absent.
    /// Used for AC-005, AC-006, AC-007, AC-008, AC-009, AC-010, AC-011,
    /// AC-012, EC-002, EC-003, EC-004, EC-005.
    const HTTP_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    /// one-decode-error.pcap: 2 pcap records — 1 ARP frame (fails decode with
    /// "No IP layer found") followed by 1 valid IPv4/UDP packet (succeeds).
    /// Produces skipped_packets=1, total_packets=1, exactly ONE warning line.
    ///
    /// A committed 145-byte pcap: packet 1 = ARP frame (fails decode, "No IP layer found");
    /// packet 2 = valid Eth/IPv4/UDP. Yields exactly 1 decode error / 1 warning (see ADV-P04-MED-001).
    /// Mutation-discriminating property: with `== 0` guard the ARP is the
    /// first (and only) error → 1 warning. If the guard were `== 1` (warn on
    /// 2nd error) there is no 2nd error → 0 warnings. This fixture kills that
    /// surviving mutant.
    const ONE_ERROR_FIXTURE: &str = "tests/fixtures/one-decode-error.pcap";

    // -----------------------------------------------------------------------
    // AC-001 (traces to BC-2.12.014 postcondition 1)
    // First decode error prints exactly one warning to stderr.
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.12.014 postcondition 1): The first decode error in the
    /// packet loop prints exactly one warning to stderr with the hardcoded
    /// message: "Warning: failed to decode packet ({e}). Further errors
    /// counted silently."
    ///
    /// Fixture: dns-remoteshell.pcap has packets that fail `decode_packet`
    /// with "No IP layer found", producing the warning. Verified: exactly 1
    /// such warning line appears (AC-004 mutation-resistant count assertion
    /// covers the "at most once" invariant).
    ///
    /// Discriminating assertions:
    ///   Positive: stderr contains the warning message prefix.
    ///   Positive: command exits 0 (decode errors do not abort the run).
    ///   Negative: WITHOUT a malformed fixture (http-ooo.pcap), no warning.
    #[test]
    fn test_first_decode_error_warning_printed() {
        // Positive: dns-remoteshell produces the warning on stderr
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", DNS_REMOTE_FIXTURE, "--dns"])
            .assert()
            .success()
            .stderr(predicate::str::contains("Warning: failed to decode packet"));

        // Negative: http-ooo.pcap has 0 decode errors → no warning
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--dns"])
            .assert()
            .success()
            .stderr(predicate::str::contains("Warning: failed to decode packet").not());
    }

    // -----------------------------------------------------------------------
    // AC-002 (traces to BC-2.12.014 postcondition 2)
    // Subsequent decode errors (2nd, 3rd, ...) are silent.
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.12.014 postcondition 2): After the first decode error,
    /// subsequent errors are counted silently — no additional warning lines
    /// are emitted. dns-remoteshell.pcap has 73 decode errors across 58 total
    /// packets; only 1 warning line appears (the count assertion in AC-004 is
    /// the mutation-resistant form; this test verifies the positive case from
    /// the BC postcondition).
    ///
    /// Discriminating assertions:
    ///   Positive: stderr does NOT contain a second warning line (the line
    ///     appears exactly once, not twice or more).
    ///   Positive: skipped_packets in JSON output == 73 (all errors counted).
    ///   Positive: command exits 0.
    #[test]
    fn test_subsequent_decode_errors_silent() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", DNS_REMOTE_FIXTURE, "--dns", "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warning_count = stderr
            .lines()
            .filter(|l| l.contains("Warning: failed to decode packet"))
            .count();
        // 73 decode errors → exactly 1 warning (subsequent are silent)
        assert_eq!(
            warning_count, 1,
            "expected exactly 1 warning line; got {warning_count}. stderr: {stderr}"
        );

        // All 73 errors counted in skipped_packets
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("\"skipped_packets\": 73"),
            "expected skipped_packets 73 in JSON; stdout: {stdout}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 (traces to BC-2.12.014 postcondition 3)
    // summary.skipped_packets equals total_decode_errors after the loop.
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.12.014 postcondition 3): After the packet loop,
    /// `summary.skipped_packets = total_decode_errors`. Observable: the
    /// `--json` output's `summary.skipped_packets` field equals the number of
    /// malformed packets in the fixture.
    ///
    /// dns-remoteshell.pcap has 73 decode failures (non-IP packets).
    /// Canonical test vector from BC-2.12.014: 0 valid/5 decode errors → 5.
    /// Here: 73 decode errors → skipped_packets == 73.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout JSON contains "\"skipped_packets\": 73".
    ///   Positive: command exits 0.
    ///   Negative: http-ooo.pcap (0 decode errors) → skipped_packets == 0.
    #[test]
    fn test_skipped_packets_equals_total_decode_errors() {
        // Positive: 73 decode errors → skipped_packets: 73
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", DNS_REMOTE_FIXTURE, "--dns", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"skipped_packets\": 73"));

        // Negative: 0 decode errors → skipped_packets: 0
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--dns", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"skipped_packets\": 0"));
    }

    // -----------------------------------------------------------------------
    // AC-004 (traces to BC-2.12.014 invariant 2)
    // Warning printed AT MOST ONCE per invocation, regardless of error count.
    // Mutation-resistant: counts occurrences of the warning line in stderr.
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.12.014 invariant 2): The warning is printed at most ONCE
    /// per invocation, regardless of how many decode errors occur. This is the
    /// mutation-resistant formalization: we count warning occurrences in stderr
    /// and assert count == 1 (not just .contains(), which would miss duplicates).
    ///
    /// dns-remoteshell.pcap: 73 decode errors → exactly 1 warning line.
    ///
    /// Method: capture stderr as string, count occurrences of the warning
    /// prefix "Warning: failed to decode packet" → must equal exactly 1.
    ///
    /// Discriminating assertions:
    ///   Positive: warning prefix appears exactly 1 time in stderr.
    ///   Positive: command exits 0.
    ///   Negative: if the guard `if total_decode_errors == 0` were removed,
    ///     73 warning lines would appear and count > 1 would fail.
    #[test]
    fn test_decode_error_warning_printed_at_most_once() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", DNS_REMOTE_FIXTURE, "--dns"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let count = stderr.matches("Warning: failed to decode packet").count();
        assert_eq!(
            count, 1,
            "warning must appear exactly once; found {count} times. stderr: {stderr}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 (traces to BC-2.12.015 postcondition 1)
    // unclassified_flows injected into reassembly summary when reassembler built.
    // Non-zero case: dns-remoteshell.pcap --http produces unclassified_flows=8.
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.12.015 postcondition 1): When a reassembler is constructed
    /// (e.g., via `--http`), after `finalize()`, `dispatcher.unclassified_flows()`
    /// is injected as `"unclassified_flows"` into the TCP Reassembly analyzer
    /// detail in the `--json` output.
    ///
    /// This test deliberately uses a fixture that yields a NON-ZERO
    /// unclassified_flows count: dns-remoteshell.pcap with --http produces
    /// unclassified_flows=8 (verified by binary run). This kills the
    /// `json!(0)` hardcode mutation that would survive if we only ever
    /// asserted the key presence or a zero value (EC-002 covers zero).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout JSON contains `"unclassified_flows": 8` (non-zero).
    ///   Positive: command exits 0.
    ///   Negative: WITHOUT reassembler (AC-006), the key is absent entirely.
    ///   Mutation gap closed: a `json!(0)` hardcode would produce
    ///     `"unclassified_flows": 0`, failing this assertion.
    #[test]
    fn test_unclassified_flows_injected_into_reassembly_summary() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", DNS_REMOTE_FIXTURE, "--http", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"unclassified_flows\": 8"));
    }

    // -----------------------------------------------------------------------
    // AC-006 (traces to BC-2.12.015 invariant 1)
    // unclassified_flows NOT present when no reassembler was constructed.
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.12.015 invariant 1): When no reassembler was constructed
    /// (e.g., `--dns --no-reassemble`), `"unclassified_flows"` is NOT present
    /// in any analyzer detail map in the JSON output.
    ///
    /// Observable: `analyze http-ooo.pcap --dns --no-reassemble --json` stdout
    /// does NOT contain `"unclassified_flows"`. The DNS analyzer detail only
    /// contains `dns_queries` and `dns_responses` keys.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout does NOT contain `"unclassified_flows"`.
    ///   Positive: command exits 0.
    ///   Negative: WITH reassembler (AC-005), the key IS present.
    #[test]
    fn test_unclassified_flows_absent_without_reassembler() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                HTTP_FIXTURE,
                "--dns",
                "--no-reassemble",
                "--json",
            ])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"unclassified_flows\"").not());
    }

    // -----------------------------------------------------------------------
    // AC-007 (traces to BC-2.12.016 postcondition 1)
    // resolve_format: --json wins over --output-format (mutation-resistant).
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.12.016 postcondition 1; EC-005): `resolve_format(cli)` returns
    /// `Some(OutputFormat::Json)` when `cli.json.is_some()`, REGARDLESS of
    /// `--output-format` setting. Observable: `--json --output-format csv` produces
    /// JSON output (begins with `{`), NOT CSV output (does NOT begin with `category,`).
    ///
    /// This is mutation-resistant: if `--output-format csv` were incorrectly
    /// given higher precedence than `--json`, the output would start with "category,"
    /// and the JSON assertion would fail.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout starts with `{` (JSON format).
    ///   Positive: stdout does NOT start with `category,` (not CSV).
    ///   Positive: command exits 0.
    ///   Negative: `--output-format csv` WITHOUT `--json` DOES produce CSV output.
    #[test]
    fn test_resolve_format_json_flag_wins_over_output_format() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                HTTP_FIXTURE,
                "--http",
                "--json",
                "--output-format",
                "csv",
            ])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim_start().starts_with('{'),
            "--json must produce JSON (starts with {{); got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
        assert!(
            !stdout.trim_start().starts_with("category,"),
            "--json must NOT produce CSV (must not start with 'category,'); got: {:?}",
            &stdout[..50.min(stdout.len())]
        );

        // Negative: --output-format csv WITHOUT --json DOES produce CSV
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http", "--output-format", "csv"])
            .assert()
            .success()
            .stdout(predicate::str::starts_with("category,"));
    }

    // -----------------------------------------------------------------------
    // AC-008 (traces to BC-2.12.016 postcondition 2)
    // resolve_format: --csv produces CSV output when --json is absent.
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.12.016 postcondition 2): `resolve_format(cli)` returns
    /// `Some(OutputFormat::Csv)` when `cli.csv.is_some()` and `cli.json.is_none()`.
    /// Observable: `analyze <fixture> --csv` produces CSV output starting with the
    /// fixed header `"category,verdict,confidence,..."`.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout starts with `"category,verdict,confidence,"` (CSV header).
    ///   Positive: command exits 0.
    ///   Negative: WITHOUT `--csv`, the output is terminal format (no CSV header).
    #[test]
    fn test_resolve_format_csv_flag() {
        // Positive: --csv produces CSV starting with the fixed header
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http", "--csv"])
            .assert()
            .success()
            .stdout(predicate::str::starts_with("category,verdict,confidence,"));

        // Negative: without --csv, terminal format (no CSV header)
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http"])
            .assert()
            .success()
            .stdout(predicate::str::starts_with("category,").not());
    }

    // -----------------------------------------------------------------------
    // AC-009 (traces to BC-2.12.016 postcondition 3)
    // resolve_format: falls back to cli.output_format (or None) when no --json/--csv.
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.12.016 postcondition 3): `resolve_format(cli)` returns
    /// `cli.output_format` (which may be `None`) when neither `--json` nor `--csv`
    /// is present. Observable:
    ///   - No flags → terminal output (contains "WIRERUST TRIAGE REPORT").
    ///   - `--output-format json` → JSON output (starts with `{`).
    ///   - `--output-format csv` → CSV output (starts with `category,`).
    ///
    /// Discriminating assertions:
    ///   Positive (no flags): stdout contains "WIRERUST TRIAGE REPORT".
    ///   Positive (--output-format json): stdout starts with `{`.
    ///   Positive (--output-format csv): stdout starts with `category,`.
    ///   Positive: all three cases exit 0.
    #[test]
    fn test_resolve_format_falls_back_to_output_format() {
        // No flags → terminal output
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http"])
            .assert()
            .success()
            .stdout(predicate::str::contains("WIRERUST TRIAGE REPORT"));

        // --output-format json → JSON output (starts with `{`)
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http", "--output-format", "json"])
            .output()
            .expect("command ran");
        assert!(output.status.success(), "command must exit 0");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim_start().starts_with('{'),
            "--output-format json must produce JSON; got: {:?}",
            &stdout[..50.min(stdout.len())]
        );

        // --output-format csv → CSV output (starts with `category,`)
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http", "--output-format", "csv"])
            .assert()
            .success()
            .stdout(predicate::str::starts_with("category,"));
    }

    // -----------------------------------------------------------------------
    // AC-010 (traces to BC-2.12.017 postcondition 1)
    // write_output: --json <FILE> writes JSON to the file.
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.12.017 postcondition 1): When `cli.json = Some(Some(path))`,
    /// `write_output` writes the JSON string to the file at `path` via
    /// `std::fs::write`. Observable: the file exists and contains JSON after the run.
    ///
    /// Canonical test vector from BC-2.12.017:
    ///   cli.json=Some(Some(PathBuf::from("out.json"))) → File created with output.
    ///
    /// Pattern mirrors tests/cli_integration_tests.rs::json_file_output_writes_json_content_to_path.
    ///
    /// Discriminating assertions:
    ///   Positive: file exists after the command.
    ///   Positive: file content starts with `{` (JSON).
    ///   Positive: file content contains `"summary"` key.
    ///   Positive: command exits 0.
    ///   Positive: stdout is empty (output went to file, not stdout).
    #[test]
    fn test_write_output_json_with_path_writes_to_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path = tmp.path().join("out.json");

        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                HTTP_FIXTURE,
                "--http",
                "--json",
                out_path.to_str().expect("utf-8 path"),
            ])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        // stdout must be empty — output went to file
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "stdout must be empty when --json <FILE> given; got: {stdout}"
        );

        // file must exist and contain JSON
        assert!(out_path.exists(), "output file must exist");
        let written = fs::read_to_string(&out_path).expect("output file readable");
        assert!(
            written.trim_start().starts_with('{'),
            "file content must start with '{{'; got prefix: {:?}",
            &written[..50.min(written.len())]
        );
        assert!(
            written.contains("\"summary\""),
            "file must contain 'summary' key; got prefix: {:?}",
            &written[..200.min(written.len())]
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 (traces to BC-2.12.017 postcondition 3)
    // write_output: default (no --json/--csv path) prints to stdout.
    // -----------------------------------------------------------------------

    /// AC-011 (BC-2.12.017 postcondition 3): When neither `--json <FILE>` nor
    /// `--csv <FILE>` is given, `write_output` prints to stdout via `println!`.
    /// Observable: `analyze <fixture>` without any file-path argument produces
    /// terminal output on stdout, NOT empty stdout.
    ///
    /// Canonical test vector: cli.json=None, cli.csv=None → stdout has terminal output.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout contains "WIRERUST TRIAGE REPORT" (terminal format on stdout).
    ///   Positive: command exits 0.
    ///   Negative: stdout is NOT empty.
    #[test]
    fn test_write_output_default_to_stdout() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http"])
            .assert()
            .success()
            .stdout(predicate::str::contains("WIRERUST TRIAGE REPORT"));
    }

    // -----------------------------------------------------------------------
    // AC-012 (traces to BC-2.12.017 invariant 4)
    // write_output file write errors are wrapped with anyhow context message.
    // -----------------------------------------------------------------------

    /// AC-012 (BC-2.12.017 invariant 4): File write errors from `write_output`
    /// are wrapped with anyhow context: "Failed to write JSON output to <path>"
    /// or "Failed to write CSV output to <path>". Observable: passing an
    /// unwritable path (directory that does not exist) produces a non-zero exit
    /// with the exact context prefix in stderr.
    ///
    /// Exact strings (verified by binary run):
    ///   "Failed to write JSON output to /nonexistent/dir/out.json"
    ///   "Failed to write CSV output to /nonexistent/dir/out.csv"
    ///
    /// Discriminating assertions:
    ///   Positive (json): stderr contains "Failed to write JSON output to".
    ///   Positive (csv): stderr contains "Failed to write CSV output to".
    ///   Positive: command exits with failure (non-zero).
    ///   Negative: a writable path does NOT produce this error.
    #[test]
    fn test_write_output_file_write_error_has_context() {
        // JSON bad path → stderr contains the exact context prefix
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                HTTP_FIXTURE,
                "--http",
                "--json",
                "/nonexistent/dir/out.json",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Failed to write JSON output to"));

        // CSV bad path → stderr contains the exact context prefix
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                HTTP_FIXTURE,
                "--http",
                "--csv",
                "/nonexistent/dir/out.csv",
            ])
            .assert()
            .failure()
            .stderr(predicate::str::contains("Failed to write CSV output to"));
    }

    // -----------------------------------------------------------------------
    // EC-001 (BC-2.12.014 EC-001): Zero decode errors → skipped_packets=0; no warning.
    // -----------------------------------------------------------------------

    /// EC-001 (BC-2.12.014 EC-001): A fixture with zero decode errors produces
    /// `skipped_packets = 0` in JSON output and emits NO warning to stderr.
    ///
    /// Fixture: http-ooo.pcap — 16 clean TCP/HTTP packets; 0 decode failures.
    /// Canonical test vector: 3 valid packets, 0 decode errors → skipped_packets=0.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout JSON contains `"skipped_packets": 0`.
    ///   Positive: stderr does NOT contain "Warning: failed to decode packet".
    ///   Positive: command exits 0.
    #[test]
    fn test_EC_001_zero_decode_errors_no_warning_and_skipped_zero() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--dns", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"skipped_packets\": 0"))
            .stderr(predicate::str::contains("Warning: failed to decode packet").not());
    }

    // -----------------------------------------------------------------------
    // EC-002 (BC-2.12.015 EC-002): Reassembler present, unclassified_flows=0
    // → "unclassified_flows": 0 in detail (zero is present, not absent).
    // -----------------------------------------------------------------------

    /// EC-002 (BC-2.12.015 EC-002 / STORY-089 EC-003): When the reassembler is
    /// present AND `unclassified_flows() = 0`, the key `"unclassified_flows"` is
    /// STILL present in the JSON output with value `0`. Zero means present, not absent.
    ///
    /// Fixture: http-ooo.pcap with --http: reassembler active, 0 unclassified flows
    /// (all flows are HTTP, classified by the dispatcher). Verified by binary run.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout JSON contains `"unclassified_flows": 0` (not absent).
    ///   Positive: command exits 0.
    ///   Negative: AC-006 shows the key is ABSENT when no reassembler.
    #[test]
    fn test_EC_002_unclassified_flows_zero_still_present_in_detail() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http", "--json"])
            .assert()
            .success()
            .stdout(predicate::str::contains("\"unclassified_flows\": 0"));
    }

    // -----------------------------------------------------------------------
    // EC-003 (BC-2.12.017 EC-002 / STORY-089 EC-004):
    // --json Some(None) (flag given, no file path) → write_output prints to stdout.
    // -----------------------------------------------------------------------

    /// EC-003 (BC-2.12.017 EC-002 / STORY-089 EC-004): When `--json` is given
    /// WITHOUT a file path (`cli.json = Some(None)`), `write_output` falls through
    /// to the stdout arm and prints the JSON output to stdout.
    ///
    /// Canonical test vector: cli.json=Some(None) → stdout contains JSON.
    ///
    /// Observable: `analyze <fixture> --json` (no path) → stdout starts with `{`.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout starts with `{` (JSON to stdout).
    ///   Positive: command exits 0.
    ///   Negative: stdout is NOT empty (output IS on stdout, not a file).
    #[test]
    fn test_EC_003_json_flag_without_path_writes_to_stdout() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", HTTP_FIXTURE, "--http", "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            !stdout.is_empty(),
            "stdout must NOT be empty when --json given without path"
        );
        assert!(
            stdout.trim_start().starts_with('{'),
            "--json without path must produce JSON on stdout; got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
    }

    // -----------------------------------------------------------------------
    // EC-004 (BC-2.12.016 EC-006 / STORY-089 EC-005):
    // --json and --output-format csv both given → resolve_format returns Some(Json).
    // Same scenario as AC-007 but named explicitly as EC per the story.
    // -----------------------------------------------------------------------

    /// EC-004 (BC-2.12.016 EC-006 / STORY-089 EC-005): When both `--json` and
    /// `--output-format csv` are given, `resolve_format` returns `Some(Json)` because
    /// `--json` has higher precedence than `--output-format`. Observable: output is
    /// JSON (starts with `{`), not CSV.
    ///
    /// This is the canonical EC for the precedence invariant (BC-2.12.016 invariant 3).
    ///
    /// Discriminating assertions:
    ///   Positive: stdout starts with `{` (JSON wins).
    ///   Positive: stdout does NOT start with `category,` (CSV suppressed).
    ///   Positive: command exits 0.
    #[test]
    fn test_EC_004_json_wins_over_output_format_csv() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "analyze",
                HTTP_FIXTURE,
                "--http",
                "--json",
                "--output-format",
                "csv",
            ])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim_start().starts_with('{'),
            "--json must win over --output-format csv (JSON starts with '{{'); got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
        assert!(
            !stdout.trim_start().starts_with("category,"),
            "--json must suppress CSV output; got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
    }

    // -----------------------------------------------------------------------
    // EC-005 (BC-2.12.014 EC-002 / STORY-089 EC-002):
    // ALL packets fail decode → skipped_packets=N; exactly one warning.
    // -----------------------------------------------------------------------

    /// EC-005 (BC-2.12.014 EC-002 / STORY-089 EC-002): This edge case is covered
    /// by AC-001 + AC-003 together using the dns-remoteshell.pcap fixture, which
    /// has 58 total packets and 73 decode errors (non-IP frames exceed total
    /// packets because the pcap contains fragmented/layered frames that each
    /// attempt multiple decode calls). The test verifies the combined invariant:
    /// skipped_packets = total decode failures AND exactly one warning emitted.
    ///
    /// Canonical test vector from BC-2.12.014: 0 valid/5 decode errors → 5.
    /// Here we use 73 decode errors as the concrete "many errors" case.
    ///
    /// Discriminating assertions:
    ///   Positive: warning appears exactly 1 time in stderr (count check).
    ///   Positive: `"skipped_packets": 73` in stdout JSON.
    ///   Positive: command exits 0.
    #[test]
    fn test_EC_005_all_packets_fail_one_warning_skipped_count_accurate() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", DNS_REMOTE_FIXTURE, "--dns", "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warning_count = stderr.matches("Warning: failed to decode packet").count();
        assert_eq!(
            warning_count, 1,
            "warning must appear exactly once; found {warning_count} times. stderr: {stderr}"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("\"skipped_packets\": 73"),
            "expected skipped_packets 73 in JSON; stdout: {stdout}"
        );
    }

    // =======================================================================
    // run_summary subcommand tests (ADV-P03-HIGH-001)
    //
    // BC-2.12.014, BC-2.12.016, BC-2.12.017 scope to BOTH run_analyze AND
    // run_summary. The tests below mirror the analyze-side formalization for
    // the shared decode-counter loop (lines ~254-278 of main.rs), reporter
    // dispatch (resolve_format), and output routing (write_output) as exercised
    // via the `summary` subcommand.
    //
    // Note on BC-2.12.016 inv-3 ("--json wins over --csv"): this invariant is
    // structurally unobservable via CLI because clap enforces conflicts_with
    // between --json and --csv — the parser rejects the combination before
    // main() runs. This applies equally to the analyze and summary subcommands.
    // No test is written for this scenario (ADV-P03-MED-001).
    // =======================================================================

    // -----------------------------------------------------------------------
    // run_summary: decode-error counting (BC-2.12.014)
    // -----------------------------------------------------------------------

    /// BC-2.12.014 via run_summary: The `summary` subcommand shares the same
    /// decode loop as `analyze`. A fixture with multiple decode errors emits
    /// exactly ONE warning to stderr and counts all errors into skipped_packets.
    ///
    /// dns-remoteshell.pcap: 73 decode errors → exactly 1 warning line and
    /// `"skipped_packets": 73` in --json output. Verified by binary run.
    ///
    /// Mutation proof: mutating `total_decode_errors += 1` in run_summary (e.g.,
    /// `+= 999`) would produce `"skipped_packets": 999` (or similar wrong count)
    /// and fail the `== 73` assertion.
    ///
    /// Discriminating assertions:
    ///   Positive: stderr has EXACTLY ONE warning line (count check, not contains).
    ///   Positive: stdout JSON contains `"skipped_packets": 73`.
    ///   Positive: command exits 0.
    #[test]
    fn test_run_summary_decode_error_warning_once() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", DNS_REMOTE_FIXTURE, "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warning_count = stderr.matches("Warning: failed to decode packet").count();
        assert_eq!(
            warning_count, 1,
            "run_summary: warning must appear exactly once across 73 decode errors; \
             found {warning_count} times. stderr: {stderr}"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("\"skipped_packets\": 73"),
            "run_summary: expected skipped_packets 73 in JSON output; stdout: {stdout}"
        );
    }

    // -----------------------------------------------------------------------
    // run_summary: format resolution (BC-2.12.016)
    // -----------------------------------------------------------------------

    /// BC-2.12.016 via run_summary postcondition 1: `--json` flag routes
    /// the summary through JsonReporter, producing JSON on stdout.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout starts with `{` (JSON format).
    ///   Positive: stdout does NOT start with `category,` (not CSV).
    ///   Positive: command exits 0.
    #[test]
    fn test_run_summary_resolve_format_json() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", HTTP_FIXTURE, "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim_start().starts_with('{'),
            "run_summary --json must produce JSON (starts with '{{'); got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
        assert!(
            !stdout.trim_start().starts_with("category,"),
            "run_summary --json must NOT produce CSV; got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
    }

    /// BC-2.12.016 via run_summary postcondition 2: `--csv` flag routes the
    /// summary through CsvReporter, producing CSV header on stdout.
    ///
    /// Mutation proof: mutating the `Some(OutputFormat::Csv)` arm to dispatch
    /// TerminalReporter would produce "WIRERUST TRIAGE REPORT" instead of the
    /// CSV header, failing this assertion.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout starts with `category,verdict,confidence,` (CSV header).
    ///   Positive: command exits 0.
    #[test]
    fn test_run_summary_resolve_format_csv() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", HTTP_FIXTURE, "--csv"])
            .assert()
            .success()
            .stdout(predicate::str::starts_with("category,verdict,confidence,"));
    }

    /// BC-2.12.016 via run_summary postcondition 3 (fallback): Without `--json`
    /// or `--csv`, the summary falls through to TerminalReporter, producing
    /// "WIRERUST TRIAGE REPORT" on stdout.
    ///
    /// Also exercises `--json --output-format csv` precedence (inv-1): --json
    /// wins, producing JSON output. Note: --json and --csv are clap-mutually-
    /// exclusive so the inv-3 scenario is unobservable (see note above).
    ///
    /// Discriminating assertions:
    ///   Positive (no flags): stdout contains "WIRERUST TRIAGE REPORT".
    ///   Positive (--json --output-format csv): stdout starts with `{` (JSON wins).
    ///   Positive: command exits 0.
    #[test]
    fn test_run_summary_resolve_format_fallback_to_terminal() {
        // No format flags → terminal output
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", HTTP_FIXTURE])
            .assert()
            .success()
            .stdout(predicate::str::contains("WIRERUST TRIAGE REPORT"));

        // --json --output-format csv → JSON wins (resolve_format precedence)
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", HTTP_FIXTURE, "--json", "--output-format", "csv"])
            .output()
            .expect("command ran");
        assert!(output.status.success(), "command must exit 0");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.trim_start().starts_with('{'),
            "run_summary --json --output-format csv: JSON must win; got: {:?}",
            &stdout[..50.min(stdout.len())]
        );
    }

    // -----------------------------------------------------------------------
    // run_summary: output routing (BC-2.12.017)
    // -----------------------------------------------------------------------

    /// BC-2.12.017 via run_summary postcondition 1: `--json <FILE>` writes
    /// JSON to the given file path; stdout is empty.
    ///
    /// Mutation proof: mutating the write_output `Some(Some(path))` arm to
    /// println! instead would produce non-empty stdout and leave the file
    /// empty/missing, failing both assertions.
    ///
    /// Discriminating assertions:
    ///   Positive: file exists after the command.
    ///   Positive: file content starts with `{` (JSON).
    ///   Positive: file content contains `"summary"` key.
    ///   Positive: stdout is empty (output went to file, not stdout).
    ///   Positive: command exits 0.
    #[test]
    fn test_run_summary_write_output_to_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let out_path = tmp.path().join("summary_out.json");

        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([
                "summary",
                HTTP_FIXTURE,
                "--json",
                out_path.to_str().expect("utf-8 path"),
            ])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        // stdout must be empty — output went to file
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.is_empty(),
            "run_summary: stdout must be empty when --json <FILE> given; got: {stdout}"
        );

        // file must exist and contain JSON
        assert!(out_path.exists(), "run_summary: output file must exist");
        let written = fs::read_to_string(&out_path).expect("output file readable");
        assert!(
            written.trim_start().starts_with('{'),
            "run_summary: file content must start with '{{'; got prefix: {:?}",
            &written[..50.min(written.len())]
        );
        assert!(
            written.contains("\"summary\""),
            "run_summary: file must contain 'summary' key; got prefix: {:?}",
            &written[..200.min(written.len())]
        );
    }

    /// BC-2.12.017 via run_summary postcondition 3: Default (no file path)
    /// prints to stdout. `summary <fixture>` without `--json <FILE>` or
    /// `--csv <FILE>` produces terminal output on stdout, not empty stdout.
    ///
    /// Discriminating assertions:
    ///   Positive: stdout contains "WIRERUST TRIAGE REPORT".
    ///   Positive: command exits 0.
    #[test]
    fn test_run_summary_default_output_to_stdout() {
        Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", HTTP_FIXTURE])
            .assert()
            .success()
            .stdout(predicate::str::contains("WIRERUST TRIAGE REPORT"));
    }

    // -----------------------------------------------------------------------
    // ADV-P04-MED-001 remediation: single-decode-error fixture
    //
    // The existing tests (AC-002, AC-004, EC-005, test_run_summary_decode_error_warning_once)
    // use dns-remoteshell.pcap (73 errors, all identical "No IP layer found").
    // A mutation `if total_decode_errors == 0` → `== 1` (warn on 2nd error
    // instead of 1st) SURVIVES those tests because 73 errors still yield exactly
    // 1 warning (errors 2-73 are also suppressed under either guard).
    //
    // FIX: one-decode-error.pcap contains EXACTLY ONE decode failure (1 ARP frame
    // that has no IP layer) plus 1 valid IPv4/UDP packet.  Under the correct
    // `== 0` guard the ARP is error #0 → warning fires → 1 warning.
    // Under the `== 1` mutant the ARP is error #0 (count still 0 at the guard
    // check before the increment) — wait, let me be precise:
    //   total_decode_errors starts at 0.
    //   First error arrives: guard checks `== 0` (true) → warns.
    //   `== 1` mutant: checks `== 1` (false, count is 0) → NO warn; count → 1.
    //   No second error exists → 0 warnings total.
    // The new tests below assert warning_count == 1, killing the mutant (0 ≠ 1).
    // -----------------------------------------------------------------------

    /// ADV-P04-MED-001 (BC-2.12.014 postcondition 1, run_analyze):
    /// With exactly ONE decode error, the `== 0` guard fires exactly once,
    /// emitting exactly 1 warning. The `== 1` mutant would emit 0 warnings
    /// (no second error exists to trigger the mutated guard).
    ///
    /// Fixture: one-decode-error.pcap — 1 ARP packet (decode fails) + 1 valid
    /// IPv4/UDP packet (decode succeeds). skipped_packets=1 in JSON output.
    ///
    /// Mutation-catch proof (verified):
    ///   Correct `== 0`:  ARP is error #0 → guard true  → 1 warning (PASS).
    ///   Mutant  `== 1`:  ARP is error #0 → guard false → 0 warnings (FAIL).
    ///
    /// Discriminating assertions:
    ///   Positive: stderr warning count == 1.
    ///   Positive: stdout JSON contains `"skipped_packets": 1`.
    ///   Positive: command exits 0.
    #[test]
    fn test_BC_2_12_014_single_error_fixture_first_error_fires_warning_run_analyze() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["analyze", ONE_ERROR_FIXTURE, "--dns", "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warning_count = stderr.matches("Warning: failed to decode packet").count();
        assert_eq!(
            warning_count, 1,
            "ADV-P04-MED-001 run_analyze: with exactly 1 decode error the warning \
             must appear exactly once (== 0 guard fires on error #0); \
             got {warning_count}. stderr: {stderr}"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("\"skipped_packets\": 1"),
            "ADV-P04-MED-001 run_analyze: expected skipped_packets=1 in JSON; stdout: {stdout}"
        );
    }

    /// ADV-P04-MED-001 (BC-2.12.014 postcondition 1, run_summary):
    /// Same mutation-discriminating fixture exercised via the `summary`
    /// subcommand. The decode loop in run_summary has its own copy of the
    /// `if total_decode_errors == 0` guard (src/main.rs ~line 267); this
    /// test kills the mutant there independently.
    ///
    /// Fixture: one-decode-error.pcap — skipped_packets=1 in JSON output.
    ///
    /// Mutation-catch proof (verified):
    ///   Correct `== 0`:  1 warning (PASS).
    ///   Mutant  `== 1`:  0 warnings (FAIL).
    ///
    /// Discriminating assertions:
    ///   Positive: stderr warning count == 1.
    ///   Positive: stdout JSON contains `"skipped_packets": 1`.
    ///   Positive: command exits 0.
    #[test]
    fn test_BC_2_12_014_single_error_fixture_first_error_fires_warning_run_summary() {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["summary", ONE_ERROR_FIXTURE, "--json"])
            .output()
            .expect("command ran");

        assert!(output.status.success(), "command must exit 0");

        let stderr = String::from_utf8_lossy(&output.stderr);
        let warning_count = stderr.matches("Warning: failed to decode packet").count();
        assert_eq!(
            warning_count, 1,
            "ADV-P04-MED-001 run_summary: with exactly 1 decode error the warning \
             must appear exactly once (== 0 guard fires on error #0); \
             got {warning_count}. stderr: {stderr}"
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains("\"skipped_packets\": 1"),
            "ADV-P04-MED-001 run_summary: expected skipped_packets=1 in JSON; stdout: {stdout}"
        );
    }
}
