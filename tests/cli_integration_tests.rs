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

// ---- F5/MEDIUM-1: CLI grouping construction-site mapping e2e guard ----
//
// The http-ooo.pcap fixture with `--all` produces 5 identical "HTTP/1.1
// request without Host header" findings (same category, verdict, confidence,
// summary). These tests exercise the REAL binary wiring at src/main.rs
// (the grouping_from_flag construction site) through end-to-end invocation.
//
// Together with the unit test `test_bc_2_11_030_grouping_flag_polarity` in
// src/main.rs, these tests form a non-tautological regression guard: a swap
// of Grouping::Grouped / Grouping::Flat in `grouping_from_flag` would:
//   - fail the unit test (wrong return value from helper), AND
//   - fail the e2e tests below (wrong output format from binary).

/// BC-2.11.030 PC-2 e2e guard:
/// `analyze <fixture> --all --mitre` must emit MITRE tactic section headers
/// (`## ` prefix) and collapse N≥2 identical-key findings to a single group
/// line with `(xN)` suffix.
#[test]
fn mitre_flag_emits_tactic_headers_and_collapse_suffix() {
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--all", "--mitre"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf-8 stdout");

    assert!(
        stdout.contains("## "),
        "BC-2.11.030 PC-2: `--mitre` must emit MITRE tactic section headers ('## '); \
         got stdout:\n{stdout}"
    );
    // The fixture produces ≥2 identical-key HTTP findings; collapsed output carries (xN).
    assert!(
        stdout.contains("(x"),
        "BC-2.11.030 PC-2: `--mitre` must collapse ≥2 identical findings with '(xN)' suffix; \
         got stdout:\n{stdout}"
    );
}

/// BC-2.11.030 PC-3 e2e guard:
/// `analyze <fixture> --all --mitre --no-collapse` must emit MITRE tactic
/// section headers (`## ` prefix) but NO `(xN)` collapse suffix on any line.
#[test]
fn mitre_no_collapse_emits_tactic_headers_without_collapse_suffix() {
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", FIXTURE, "--all", "--mitre", "--no-collapse"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf-8 stdout");

    assert!(
        stdout.contains("## "),
        "BC-2.11.030 PC-3: `--mitre --no-collapse` must emit MITRE tactic headers ('## '); \
         got stdout:\n{stdout}"
    );
    // Strip ANSI codes before checking — the terminal reporter may emit color codes.
    // Simple approach: look for the literal "(x" which only appears in collapse suffixes.
    assert!(
        !stdout.contains("(x"),
        "BC-2.11.030 PC-3: `--mitre --no-collapse` must NOT emit any '(xN)' collapse suffix; \
         got stdout:\n{stdout}"
    );
}

// O-1: Named MITRE tactic header assertion and (xN) collapse on modbus fixture
//
// The http-ooo.pcap fixture only produces "## Uncategorized" because its HTTP
// findings carry no MITRE technique IDs. The `## ` assertion in
// `mitre_flag_emits_tactic_headers_and_collapse_suffix` is therefore satisfied
// by the Uncategorized bucket alone — it does not verify that a NAMED tactic
// header (Discovery, Command and Control, etc.) is correctly emitted.
//
// modbus-write.pcap produces findings with T1046 (→ Discovery) technique IDs,
// yielding a `## Discovery` tactic header and a collapsed N=2 group.
// This test pins a NAMED tactic header so that a bug suppressing tactic-name
// resolution (while still emitting `## Uncategorized`) would be caught.

// ---- F-PASS-A-001: --mitre help-text collapse regression guard ----
//
// The --mitre clap doc-comment must mention collapse behavior so that
// `wirerust analyze --help` agrees with README and --no-collapse docs.
// This test pins those keywords so a revert to the terse pre-STORY-119
// one-liner is caught immediately.

/// F-PASS-A-001 regression guard:
/// `wirerust analyze --help` must document that `--mitre` collapses
/// identical findings within each tactic bucket with a `(xN)` count suffix
/// by default, and that `--no-collapse` disables it.
///
/// The test scopes to only the `--mitre` entry text (between `--mitre` and the
/// next `--no-collapse` flag entry) so that keywords found in the `--no-collapse`
/// description do not produce a false pass when `--mitre` reverts to the stale
/// terse one-liner.
///
/// Mutation-fail verification (performed during F-PASS-A-001 fix):
///   Reverted `--mitre` doc-comment to "Group findings by MITRE ATT&CK tactic
///   and show technique names" (single terse line, no "collapse" / "(x").
///   Confirmed the binary renders the terse text under `--mitre` with no
///   "collapse" or "(x" in the scoped slice up to `--no-collapse`.
///   Test failed with: assertion `--mitre` help text must mention 'collapse'.
///   Restored new doc-comment: test passes.
#[test]
fn mitre_help_text_mentions_collapse_behavior() {
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(output).expect("utf-8 stdout");

    // Locate the --mitre entry and extract only the text up to the sibling
    // --no-collapse entry.  This scoping prevents the richer --no-collapse
    // description from satisfying the assertions when --mitre reverts to the
    // stale one-liner (the bug F-PASS-A-001 guards against).
    let mitre_pos = help
        .find("--mitre\n")
        .expect("F-PASS-A-001: `--mitre` flag must appear in `analyze --help` output");
    let after_mitre = &help[mitre_pos..];
    // Clap renders the next flag with leading spaces + "--no-collapse"; slice before it.
    let no_collapse_pos = after_mitre
        .find("--no-collapse")
        .expect("F-PASS-A-001: `--no-collapse` must appear after `--mitre` in help output");
    let mitre_entry = &after_mitre[..no_collapse_pos];

    assert!(
        mitre_entry.contains("collapse"),
        "F-PASS-A-001: `--mitre` help text (before --no-collapse entry) must mention \
         'collapse'; got mitre entry:\n{mitre_entry}"
    );
    assert!(
        mitre_entry.contains("(x"),
        "F-PASS-A-001: `--mitre` help text (before --no-collapse entry) must mention \
         '(x' count suffix; got mitre entry:\n{mitre_entry}"
    );
}

/// O-1 (BC-2.11.030 PC-2 / BC-2.11.033 PC-3):
/// REGRESSION GUARD: `analyze modbus-write.pcap --all --mitre` must emit a
/// NAMED MITRE tactic header `## Discovery` (produced by T1046 findings in the
/// fixture). The existing `mitre_flag_emits_tactic_headers_and_collapse_suffix`
/// test only asserts `## ` which is satisfied by `## Uncategorized`. This test
/// pins the named-tactic path: if tactic-name resolution is broken, `## Discovery`
/// would not appear and this test fails.
///
/// Also asserts that the collapsed N=2 group in the Discovery bucket carries an
/// `(xN)` suffix (the fixture produces 2 Modbus recon findings with the same key).
///
/// FAIL mode: disable `technique_tactic` lookup so all findings fall into
/// Uncategorized. The `## Discovery` assertion fails; the existing `## `
/// assertion in the companion test would still pass.
#[test]
fn mitre_named_tactic_header_emitted_for_modbus_fixture() {
    // modbus-write.pcap produces T1046 findings that map to the Discovery tactic.
    // Verified by running: cargo run -- analyze tests/fixtures/modbus-write.pcap --all --mitre
    // Output includes:
    //   ## Discovery
    //   [Anomaly] INCONCLUSIVE (MEDIUM) - Modbus recon: Report Server ID ... (x2)
    //   ## Impair Process Control
    const MODBUS_FIXTURE: &str = "tests/fixtures/modbus-write.pcap";

    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", MODBUS_FIXTURE, "--all", "--mitre"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).expect("utf-8 stdout");

    // Named tactic header must appear — NOT just `## Uncategorized`.
    assert!(
        stdout.contains("## Discovery"),
        "O-1 (BC-2.11.030 PC-2): `--mitre` must emit the named MITRE tactic header \
         '## Discovery' for T1046 findings in modbus-write.pcap; \
         got stdout:\n{stdout}"
    );

    // The Discovery bucket contains N=2 identical-key Modbus recon findings;
    // collapsed output must carry an `(xN)` suffix on the group header.
    assert!(
        stdout.contains("(x"),
        "O-1 (BC-2.11.030 PC-2): `--mitre` on modbus-write.pcap must collapse \
         N≥2 identical-key Modbus recon findings with '(xN)' suffix; \
         got stdout:\n{stdout}"
    );
}

// ---- F-A-001: CLI help must not expose internal factory provenance IDs ----
//
// Clap renders `///` doc-comment text into `--help` output. Internal factory
// provenance tags (BC-*, STORY-*, LESSON-*) embedded in `///` comments appear
// verbatim in `wirerust analyze --help` and `wirerust summary --help`, which is
// confusing to end-users. This test guards the sweep performed in
// src/cli.rs: all provenance IDs must live in `//` (non-doc) comments so they
// are present as source annotations but do NOT reach the rendered help text.
//
// FAIL MODE VERIFICATION (done manually before committing): temporarily
// restoring one `///` line with "BC-2.11.028" caused this test to fail with:
//   assertion failed: `--help` must not contain "BC-"; found in stdout
// Then removing the ID restored the green state.

// ---- v0.9.1 doc-bug regression guard: --no-collapse flag naming ----
//
// Pre-0.9.1 the --no-collapse clap doc-comment said "Has no effect on --output
// json or --output csv." but `--output` does not exist; the real flags are
// `--json`, `--csv`, and `--output-format`. This test pins the corrected wording
// so any revert to the bogus substring is caught immediately.
//
// FAIL MODE VERIFICATION (performed before committing 0.9.1 fix):
//   Temporarily restored "Has no effect on --output json or --output csv." in
//   src/cli.rs and ran: cargo test no_collapse_help_names_real_output_flags
//   Test failed with:
//     assertion failed: `--no-collapse` help must NOT contain bogus `--output json`
//   Then `--output csv` assertion also failed.
//   Restored corrected wording: test passes.
//   Assertion (b) was verified green throughout (--json appears in sibling flags).

/// REGRESSION GUARD (v0.9.1): `wirerust analyze --help` `--no-collapse` entry
/// must NOT reference the non-existent `--output json` / `--output csv` flags,
/// and MUST reference at least one of the real output flags (`--json`, `--csv`,
/// `--output-format`).
///
/// Scoping: clap renders flags as `\n      --flag-name\n          description`.
/// The test finds the `--no-collapse` FLAG LINE (a line whose content, when
/// trimmed, equals `--no-collapse`) and extracts the description lines that
/// follow until the next flag line.  This prevents the `(pass --no-collapse to
/// disable)` text in the sibling `--mitre` entry from tainting the assertions.
///
/// Fail-mode verification (run before committing 0.9.1 fix):
///   Restored "Has no effect on --output json or --output csv." in src/cli.rs,
///   ran cargo test no_collapse_help_names_real_output_flags.
///   Test failed with:
///     v0.9.1-reg: `--no-collapse` help must NOT contain bogus `--output json`
///   and `--output csv` assertion also failed.
///   Restored corrected wording: both (a) assertions pass, (b) passes.
#[test]
fn no_collapse_help_names_real_output_flags() {
    let output = Command::cargo_bin("wirerust")
        .expect("binary built")
        .args(["analyze", "--help"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let help = String::from_utf8(output).expect("utf-8 stdout");

    // Find the line where --no-collapse IS the flag (trimmed line == "--no-collapse"),
    // not a reference to it inside another flag's description.
    let nc_flag_line_start = help
        .lines()
        .enumerate()
        .find(|(_, line)| line.trim() == "--no-collapse")
        .map(|(i, _)| {
            // Byte offset of the start of this line.
            help.lines()
                .take(i)
                .map(|l| l.len() + 1) // +1 for the \n
                .sum::<usize>()
        })
        .expect("v0.9.1-reg: `--no-collapse` flag line must appear in `analyze --help`");

    let after_nc_line = &help[nc_flag_line_start..];

    // The description is on the lines that follow the flag line.
    // Collect lines until we hit the next flag line (trimmed starts with "--").
    let mut nc_block = String::new();
    let mut saw_flag_line = false;
    for line in after_nc_line.lines() {
        if line.trim() == "--no-collapse" {
            saw_flag_line = true;
            nc_block.push_str(line);
            nc_block.push('\n');
            continue;
        }
        if saw_flag_line {
            // Stop at the next sibling flag line.
            if line.trim_start().starts_with("--") && line.trim() != "--no-collapse" {
                break;
            }
            nc_block.push_str(line);
            nc_block.push('\n');
        }
    }

    // (a) must NOT contain the bogus non-existent flag spelling.
    assert!(
        !nc_block.contains("--output json"),
        "v0.9.1-reg: `--no-collapse` help must NOT contain bogus `--output json`; \
         got block:\n{nc_block}"
    );
    assert!(
        !nc_block.contains("--output csv"),
        "v0.9.1-reg: `--no-collapse` help must NOT contain bogus `--output csv`; \
         got block:\n{nc_block}"
    );

    // (b) must reference at least one real output flag.
    let references_real_flag = nc_block.contains("--json")
        || nc_block.contains("--csv")
        || nc_block.contains("--output-format");
    assert!(
        references_real_flag,
        "v0.9.1-reg: `--no-collapse` help must reference a real output flag \
         (`--json`, `--csv`, or `--output-format`); got block:\n{nc_block}"
    );
}

/// REGRESSION GUARD: `wirerust analyze --help` must not contain any of the
/// internal factory provenance IDs `BC-`, `STORY-`, or `LESSON-`.
///
/// Each subcommand's help text is tested independently so failures name the
/// exact surface that regressed.
#[test]
fn help_output_contains_no_internal_provenance_ids() {
    for subcommand in &["analyze", "summary"] {
        let output = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args([subcommand, "--help"])
            // clap exits 0 for --help
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");

        for id_prefix in &["BC-", "STORY-", "LESSON-"] {
            assert!(
                !stdout.contains(id_prefix),
                "`wirerust {subcommand} --help` must not contain \"{id_prefix}\"; \
                 found in stdout:\n{stdout}"
            );
        }
    }
}
