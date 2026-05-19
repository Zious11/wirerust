//! End-to-end CLI integration tests that spawn the `wirerust` binary.
//!
//! These tests exercise the `--json <FILE>` file-output path (LESSON-P0.04) and
//! the loud-bail behavior on `--csv` / `--output-format csv`. They are the
//! first activated use of the `assert_cmd` + `predicates` + `tempfile`
//! dev-dependencies (previously dead per BC-ABS-009).

use assert_cmd::Command;
use predicates::str::contains;

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

#[test]
fn csv_flag_bails_with_clear_message() {
    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--csv"])
        .assert()
        .failure()
        .stderr(contains("--csv output is not yet implemented"));
}

#[test]
fn csv_flag_with_path_bails_with_clear_message() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let out_path = tmp.path().join("would-not-be-written.csv");

    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args([
            "analyze",
            FIXTURE,
            "--csv",
            out_path.to_str().expect("utf-8 path"),
        ])
        .assert()
        .failure()
        .stderr(contains("--csv output is not yet implemented"));

    assert!(
        !out_path.exists(),
        "CSV bail must not create the requested output file"
    );
}

#[test]
fn output_format_csv_bails_with_clear_message() {
    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--output-format", "csv"])
        .assert()
        .failure()
        .stderr(contains("--output-format csv is not yet implemented"));
}

#[test]
fn summary_subcommand_also_honors_csv_bail() {
    // The bail must fire for both subcommands, not just `analyze`.
    Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["summary", FIXTURE, "--csv"])
        .assert()
        .failure()
        .stderr(contains("--csv output is not yet implemented"));
}
