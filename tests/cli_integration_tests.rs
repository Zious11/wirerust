//! End-to-end CLI integration tests that spawn the `wirerust` binary.
//!
//! These tests exercise the `--json <FILE>` file-output path (LESSON-P0.04)
//! and the `--csv` reporter (LESSON-P2.03). They were the first activated
//! use of the `assert_cmd` + `predicates` + `tempfile` dev-dependencies
//! (previously dead per BC-ABS-009).

use assert_cmd::Command;

/// Smallest consumed pcap fixture in the tree (1,209 B). Produces a small but
/// non-empty findings set when analyzed with `--all`.
const FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

#[test]
fn json_file_output_writes_json_content_to_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_path = tmp.path().join("out.json");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            FIXTURE,
            "--all",
            "--json",
            out_path.to_str().expect("utf-8 path"),
        ])
        .assert()
        .success();

    let written = std::fs::read_to_string(&out_path).expect("output file exists");
    assert!(
        written.contains("\"summary\""),
        "JSON output must contain a 'summary' key; got first 200 chars: {}",
        &written[..200.min(written.len())]
    );
    assert!(
        written.contains("\"findings\""),
        "JSON output must contain a 'findings' key; got first 200 chars: {}",
        &written[..200.min(written.len())]
    );
}

#[test]
fn json_file_output_overrides_terminal_output_format() {
    // --output-format is not given; --json <FILE> alone should force JSON
    // (i.e. the file is written even without --output-format json on the CLI).
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_path = tmp.path().join("forced.json");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            FIXTURE,
            "--all",
            "--json",
            out_path.to_str().expect("utf-8 path"),
        ])
        .assert()
        .success();

    let written = std::fs::read_to_string(&out_path).expect("output file exists");
    // The terminal reporter never emits raw `{`-prefixed JSON; if `--json
    // <FILE>` did not force JSON, the file would either be terminal-table
    // text or absent.
    assert!(
        written.trim_start().starts_with('{'),
        "file must contain JSON object, got prefix: {:?}",
        &written[..50.min(written.len())]
    );
}

// ---- LESSON-P2.03: CSV reporter ----

#[test]
fn csv_flag_with_path_writes_csv_findings_table() {
    // `--csv <FILE>` writes the findings table to the file. The
    // fixture analyzed with `--all` produces a non-empty findings set,
    // so the CSV has the header row plus at least one data row.
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_path = tmp.path().join("out.csv");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            FIXTURE,
            "--all",
            "--csv",
            out_path.to_str().expect("utf-8 path"),
        ])
        .assert()
        .success();

    let written = std::fs::read_to_string(&out_path).expect("output file exists");
    let mut lines = written.lines();
    // Fixed header row, exact column order.
    assert_eq!(
        lines.next(),
        Some(
            "category,verdict,confidence,summary,evidence,mitre_techniques,source_ip,direction,timestamp"
        ),
        "CSV must start with the fixed header row"
    );
    assert!(
        lines.next().is_some(),
        "CSV must contain at least one finding row for the --all analysis of the fixture"
    );
}

#[test]
fn csv_flag_without_path_writes_csv_to_stdout() {
    // `--csv` with no path emits the CSV table to stdout. The terminal
    // reporter never emits the CSV header line, so its presence proves
    // the CSV reporter ran.
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--all", "--csv"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf-8 stdout");
    assert!(
        stdout.starts_with("category,verdict,confidence,summary,"),
        "stdout must begin with the CSV header, got: {:?}",
        &stdout[..60.min(stdout.len())]
    );
}

#[test]
fn output_format_csv_emits_csv() {
    // `--output-format csv` is honored identically to `--csv`.
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--all", "--output-format", "csv"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf-8 stdout");
    assert!(
        stdout.starts_with("category,verdict,confidence,"),
        "--output-format csv must produce CSV output"
    );
}

#[test]
fn summary_subcommand_also_supports_csv() {
    // The CSV reporter is wired for both subcommands. `summary` has no
    // findings, so the CSV is just the header row.
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["summary", FIXTURE, "--csv"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf-8 stdout");
    assert!(
        stdout.starts_with("category,verdict,confidence,"),
        "summary --csv must produce the CSV header"
    );
}

#[test]
fn json_and_csv_flags_are_mutually_exclusive() {
    // clap `conflicts_with` must reject `--json` + `--csv` together.
    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--json", "--csv"])
        .assert()
        .failure();
}
