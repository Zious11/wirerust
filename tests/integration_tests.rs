//! Integration tests for STORY-152: `wirerust protocols` CLI subcommand.
//!
//! Behavioral contracts covered:
//!   BC-2.12.022 v1.0 — `wirerust protocols` Subcommand Dispatch + `--json` Flag
//!   BC-2.18.001 v1.4 — Terminal Catalog Output (filter flags, [L2], EtherType, footnotes)
//!   BC-2.18.002 v1.1 — JSON Mode Output Schema
//!
//! All tests in `mod story_152` are GREEN regression guards for the `protocols`
//! subcommand introduced in STORY-152. They guard that:
//!   - the subcommand remains registered in clap and exits 0 for valid invocations,
//!   - filter flags (`--supported`, `--unsupported`, `--all`) produce the correct row sets,
//!   - terminal output correctly renders transport, EtherType, and footnotes,
//!   - JSON output conforms to BC-2.18.002 field schema,
//!   - the existing `analyze` subcommand is unaffected by the addition.
//!
//! Harness: `assert_cmd::Command::cargo_bin("wirerust")` (binary process spawning).
//! Catalog access: `wirerust::protocols::{all_protocols, supported_protocols,
//! unsupported_protocols}` imported directly for row-count derivations.
//!
//! DF-CANONICAL-FRAME-HOLDOUT-001: EtherType canonical values asserted with spec citations:
//!   GOOSE   0x88B8 = 35000  — IEC 61850-8-1 §4; IEEE RA "IEC GOOSE"
//!   POWERLINK 0x88AB = 34987 — IEEE RA "ETHERNET Powerlink"; Wireshark ETHERTYPE_EPL_V2;
//!              IETF ietf-ethertypes value 34987
//!   BACnet/IP UDP 47808 = 0xBAC0 — ASHRAE 135-2016 Annex J §J.2.1
//!   Modbus/TCP TCP 502  — IANA + Modbus App Protocol v1.1b3 §4.3.1

mod story_152 {
    #![allow(non_snake_case)]

    use assert_cmd::Command;
    use wirerust::protocols::{all_protocols, supported_protocols, unsupported_protocols};

    /// Smallest pcap fixture used by the existing integration tests (1,209 B).
    /// Shared with `cli_integration_tests.rs` — no new fixture needed.
    const ANALYZE_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    /// Spawn the `wirerust` binary (debug build).
    fn bin() -> Command {
        Command::cargo_bin("wirerust").expect("wirerust binary must be built")
    }

    // -----------------------------------------------------------------------
    // BC-2.12.022 — CLI wiring tests
    // -----------------------------------------------------------------------

    /// BC-2.12.022 PC-1 / Postcondition 6 / Invariant 1
    /// `wirerust protocols` exits 0 and produces non-empty stdout.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed from clap
    /// or if it starts producing empty stdout.
    #[test]
    fn test_BC_2_12_022_protocols_subcommand_exit_0() {
        let output = bin()
            .args(["protocols"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            !stdout.is_empty(),
            "BC-2.12.022 Invariant 1: `wirerust protocols` must produce non-empty stdout; \
             got empty output"
        );
    }

    /// BC-2.12.022 Invariant 2 / EC-006
    /// `wirerust protocols --supported --unsupported` exits non-zero (clap conflict).
    ///
    /// Guards that clap's `conflicts_with` annotation on `--supported` vs `--unsupported`
    /// remains in place. Fails if the `conflicts_with` constraint is removed from the
    /// `protocols` subcommand definition in `src/cli.rs`.
    #[test]
    fn test_BC_2_12_022_mutually_exclusive_flags_error() {
        let assert = bin()
            .args(["protocols", "--supported", "--unsupported"])
            .assert()
            .failure();
        // Must be a clap conflict/usage error, not any arbitrary non-zero exit.
        // clap emits "cannot be used with" for conflicts_with violations.
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).expect("utf-8 stderr");
        assert!(
            stderr.contains("cannot be used with"),
            "BC-2.12.022 Invariant 2: expected a clap conflict error \
             (\"cannot be used with\") for `--supported --unsupported`; \
             got stderr:\n{stderr}"
        );
    }

    /// BC-2.12.022 Postcondition 3 / EC-002
    /// `wirerust protocols --supported` output has exactly 7 protocol rows
    /// (one per `supported_protocols()` entry).
    ///
    /// Row count is derived robustly by matching stdout lines against protocol
    /// names from `supported_protocols()`, not by counting header/footnote lines.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_12_022_protocols_supported_filter() {
        let output = bin()
            .args(["protocols", "--supported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let supported = supported_protocols();
        // Exclude lines starting with "NOTE:" for robustness against footnote changes.
        let row_count = stdout
            .lines()
            .filter(|line| !line.starts_with("NOTE:"))
            .filter(|line| supported.iter().any(|p| line.contains(p.name)))
            .count();
        assert_eq!(
            row_count, 7,
            "BC-2.12.022 EC-002: `wirerust protocols --supported` must produce exactly \
             7 protocol rows (== supported_protocols().len()); got {row_count}.\n\
             stdout:\n{stdout}"
        );
    }

    /// BC-2.12.022 Postcondition 5 / EC-004
    /// `wirerust protocols --json` stdout is valid JSON containing a `"protocols"` array.
    ///
    /// The global `--json` flag (no path) routes output to stdout. Parsed with `serde_json`
    /// — no shell-out to `jq`.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_12_022_protocols_json_flag() {
        let output = bin()
            .args(["protocols", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.022 PC-5: `wirerust protocols --json` must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        assert!(
            json.get("protocols").is_some(),
            "BC-2.12.022 PC-5: JSON output must contain a top-level 'protocols' key; \
             got JSON: {json}"
        );
        assert!(
            json["protocols"].is_array(),
            "BC-2.12.022 PC-5: JSON 'protocols' value must be an array; \
             got: {:?}",
            json["protocols"]
        );
    }

    /// BC-2.12.022 Postcondition 7 / Invariant 7
    /// `wirerust analyze <fixture>` is unaffected by the addition of the `protocols`
    /// subcommand: it still exits 0, still contains `"summary"` and `"findings"` keys,
    /// and does NOT contain a `"protocols"` key.
    ///
    /// Regression guard for the existing `analyze` subcommand: fails if the `protocols`
    /// implementation breaks `analyze` output or accidentally injects a `"protocols"` key
    /// into `analyze --json` output.
    #[test]
    fn test_BC_2_12_022_analyze_unaffected() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--all", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.022 Invariant 7: `wirerust analyze --json` must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        assert!(
            json.get("summary").is_some(),
            "analyze JSON output must still contain 'summary' key after adding protocols \
             subcommand; BC-2.12.022 Invariant 7"
        );
        assert!(
            json.get("findings").is_some(),
            "analyze JSON output must still contain 'findings' key; BC-2.12.022 Invariant 7"
        );
        assert!(
            json.as_object()
                .is_some_and(|obj| !obj.contains_key("protocols")),
            "BC-2.12.022 Invariant 7: `wirerust analyze` JSON output must NOT contain a \
             'protocols' key after adding the protocols subcommand"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.18.001 — Terminal renderer tests
    // -----------------------------------------------------------------------

    /// BC-2.18.001 v1.4 Postcondition 1 / Invariant 2
    /// `wirerust protocols --all` prints exactly 30 protocol rows (== `all_protocols().len()`).
    ///
    /// Row count is derived by matching stdout lines against protocol names from
    /// `all_protocols()` — header lines and footnotes are not counted.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_all_row_count() {
        let output = bin()
            .args(["protocols", "--all"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let all = all_protocols();
        // Exclude lines starting with "NOTE:" so the port-102 footnote (which names
        // protocol names per AC-152-004) does not inflate the data-row count.
        let row_count = stdout
            .lines()
            .filter(|line| !line.starts_with("NOTE:"))
            .filter(|line| all.iter().any(|p| line.contains(p.name)))
            .count();
        let expected = all.len(); // == 30
        assert_eq!(
            row_count, expected,
            "BC-2.18.001 Invariant 2: `wirerust protocols --all` must print exactly \
             {expected} protocol rows (== all_protocols().len()); got {row_count}.\n\
             stdout:\n{stdout}"
        );
    }

    /// BC-2.18.001 v1.4 Postconditions 2 + 3 / EC-001 / EC-002
    /// `wirerust protocols --supported` output contains ONLY the 7 supported protocols
    /// and NONE of the unsupported ones.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_supported_filter() {
        let output = bin()
            .args(["protocols", "--supported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let supported = supported_protocols();
        let unsupported = unsupported_protocols();

        // All 7 supported protocol names must appear.
        for proto in &supported {
            assert!(
                stdout.contains(proto.name),
                "BC-2.18.001 PC-2: `wirerust protocols --supported` must contain '{}'; \
                 stdout:\n{stdout}",
                proto.name
            );
        }
        // No unsupported protocol names may appear.
        for proto in &unsupported {
            assert!(
                !stdout.contains(proto.name),
                "BC-2.18.001 PC-2: `wirerust protocols --supported` must NOT contain \
                 '{}' (unsupported entry); stdout:\n{stdout}",
                proto.name
            );
        }
    }

    /// BC-2.18.001 v1.4 Postcondition 6 / Invariant 3
    /// `wirerust protocols --unsupported` stdout contains the TCP/102 collision note
    /// (S7comm, S7comm-plus, IEC 61850 MMS, and ICCP/TASE.2 all appear in the set).
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_port102_footnote() {
        let output = bin()
            .args(["protocols", "--unsupported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // The footnote must reference TCP/102 (exact form may vary; "102" is the minimum signal).
        assert!(
            stdout.contains("TCP/102") || stdout.contains("port 102"),
            "BC-2.18.001 PC-6: `wirerust protocols --unsupported` must include the port-102 \
             collision footnote (S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2 all share \
             TCP/102); stdout:\n{stdout}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 6 / Invariant 3 (conditional footnote)
    /// `wirerust protocols --supported` stdout does NOT contain the port-102 footnote:
    /// none of the four TCP/102 protocols (S7comm, S7comm-plus, IEC 61850 MMS,
    /// ICCP/TASE.2) appear in the supported set.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_port102_footnote_absent_supported() {
        let output = bin()
            .args(["protocols", "--supported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            !stdout.contains("TCP/102"),
            "BC-2.18.001 PC-6 conditional: `wirerust protocols --supported` must NOT contain \
             the 'TCP/102' port-102 collision footnote (none of the four TCP/102 protocols \
             are in the supported set); stdout:\n{stdout}"
        );
        assert!(
            !stdout.contains("NOTE: TCP/102"),
            "BC-2.18.001 PC-6 conditional: `wirerust protocols --supported` must NOT \
             contain the NOTE: TCP/102 footnote line; stdout:\n{stdout}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 6 (footnote names all four)
    /// The port-102 collision footnote in `--unsupported` output names all four protocols:
    /// S7comm, S7comm-plus, IEC 61850 MMS, and ICCP (or ICCP/TASE.2).
    ///
    /// Source: ISO/IEC standards — S7comm (Siemens RFC), IEC 61850 MMS (IEC 61850-8-1),
    /// ICCP/TASE.2 (IEC 60870-6). All four share TCP/102 (ISO on TCP / TPKT framing,
    /// RFC 1006). The footnote must name each to satisfy BC-2.18.001 PC-6 exact text.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_port102_footnote_names_all_four() {
        let output = bin()
            .args(["protocols", "--unsupported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // Locate the footnote line specifically — names must appear IN THE FOOTNOTE,
        // not just anywhere in stdout (the same names also appear as data rows).
        // This guards against a regression where the footnote omits the names but
        // the data rows still satisfy a bare stdout.contains() check.
        let footnote_line = stdout
            .lines()
            .find(|line| line.starts_with("NOTE: TCP/102"))
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.18.001 PC-6: expected a footnote line starting with \
                     'NOTE: TCP/102' in --unsupported output; stdout:\n{stdout}"
                )
            });
        assert!(
            footnote_line.contains("S7comm"),
            "BC-2.18.001 PC-6: footnote must name 'S7comm'; footnote line: {footnote_line:?}"
        );
        // S7comm-plus (or equivalent notation).
        assert!(
            footnote_line.contains("S7comm-plus") || footnote_line.contains("S7comm+"),
            "BC-2.18.001 PC-6: footnote must name 'S7comm-plus'; \
             footnote line: {footnote_line:?}"
        );
        // IEC 61850 MMS.
        assert!(
            footnote_line.contains("IEC 61850 MMS") || footnote_line.contains("MMS"),
            "BC-2.18.001 PC-6: footnote must name 'IEC 61850 MMS'; \
             footnote line: {footnote_line:?}"
        );
        // ICCP or ICCP/TASE.2.
        assert!(
            footnote_line.contains("ICCP") || footnote_line.contains("TASE.2"),
            "BC-2.18.001 PC-6: footnote must name 'ICCP' or 'ICCP/TASE.2'; \
             footnote line: {footnote_line:?}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 4 / EC-004
    /// The GOOSE row in `wirerust protocols --unsupported` contains `[L2]` in the
    /// transport column (IEC 61850 GOOSE is a `transport=LinkLayer` entry).
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_l2_transport_indicator() {
        let output = bin()
            .args(["protocols", "--unsupported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let goose_line = stdout
            .lines()
            .find(|line| line.contains("GOOSE"))
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.18.001 EC-004: GOOSE entry must appear in --unsupported output; \
                     stdout:\n{stdout}"
                )
            });
        assert!(
            goose_line.contains("[L2]"),
            "BC-2.18.001 PC-4: GOOSE row must contain '[L2]' transport indicator; \
             got line: {goose_line:?}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 7 / Invariant 4
    /// `wirerust protocols --unsupported` stdout contains a note that L2/LinkLayer protocols
    /// never appear in the dynamic gap report (`CoverageGapsSummary`).
    ///
    /// The note may be a table footer, footnote, or column annotation. The test checks
    /// for the presence of "gap" (a keyword that must appear in any well-formed note
    /// about gap-report invisibility).
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_l2_note_present() {
        let output = bin()
            .args(["protocols", "--unsupported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let stdout_lower = stdout.to_lowercase();
        // The note must communicate that L2 entries are undetectable in gap reports.
        // "gap" is the minimum distinguishing keyword; a longer phrase is also acceptable.
        let has_note = stdout_lower.contains("gap report")
            || stdout_lower.contains("gap reports")
            || stdout_lower.contains("coveragegap")
            || (stdout_lower.contains("gap") && stdout.contains("[L2]"));
        assert!(
            has_note,
            "BC-2.18.001 PC-7: `wirerust protocols --unsupported` must include a note about \
             L2/LinkLayer entries not appearing in gap reports; stdout:\n{stdout}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 5; DF-CANONICAL-FRAME-HOLDOUT-001
    /// GOOSE row in `wirerust protocols --unsupported` contains `0x88B8 (35000)`.
    ///
    /// Canonical value source: IEC 61850-8-1 §4 ("EtherType 0x88B8");
    /// IEEE Registration Authority EtherType registry entry "IEC GOOSE" (decimal 35000).
    /// 0x88B8 = 32768 + 2048 + 176 + 8 = 35000 (IEC 61850-8-1 §4; IEEE RA "IEC GOOSE").
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_goose_ethertype_display() {
        let output = bin()
            .args(["protocols", "--unsupported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let goose_line = stdout
            .lines()
            .find(|line| line.contains("GOOSE"))
            .unwrap_or_else(|| {
                panic!(
                    "DF-CANONICAL-FRAME-HOLDOUT-001: GOOSE entry must appear in \
                     --unsupported output; stdout:\n{stdout}"
                )
            });
        // 0x88B8 (case-insensitive hex)
        assert!(
            goose_line.contains("0x88B8") || goose_line.contains("0x88b8"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: GOOSE row must display EtherType '0x88B8' \
             (IEC 61850-8-1 §4; IEEE RA registry \"IEC GOOSE\"); \
             got line: {goose_line:?}"
        );
        // Decimal 35000
        assert!(
            goose_line.contains("35000"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: GOOSE row must display decimal 35000 \
             (0x88B8 = 35000; IEC 61850-8-1 §4; IEEE RA registry \"IEC GOOSE\"); \
             got line: {goose_line:?}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 5; DF-CANONICAL-FRAME-HOLDOUT-001
    /// POWERLINK row in `wirerust protocols --unsupported` contains `0x88AB (34987)`.
    ///
    /// Canonical value source: IEEE Registration Authority EtherType registry entry
    /// "ETHERNET Powerlink" (0x88AB, decimal 34987); confirmed by Wireshark
    /// `epan/etypes.h` constant `ETHERTYPE_EPL_V2 = 0x88AB`; IETF `ietf-ethertypes`
    /// YANG module value 34987. The obsolete V1 value 0x3E3F (16191) must not appear.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_powerlink_ethertype_display() {
        let output = bin()
            .args(["protocols", "--unsupported"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let powerlink_line = stdout
            .lines()
            .find(|line| line.to_lowercase().contains("powerlink"))
            .unwrap_or_else(|| {
                panic!(
                    "DF-CANONICAL-FRAME-HOLDOUT-001: POWERLINK entry must appear in \
                     --unsupported output; stdout:\n{stdout}"
                )
            });
        // 0x88AB (case-insensitive)
        assert!(
            powerlink_line.contains("0x88AB") || powerlink_line.contains("0x88ab"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: POWERLINK row must display EtherType '0x88AB' \
             (IEEE RA registry \"ETHERNET Powerlink\"; Wireshark ETHERTYPE_EPL_V2); \
             got line: {powerlink_line:?}"
        );
        // Decimal 34987
        assert!(
            powerlink_line.contains("34987"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: POWERLINK row must display decimal 34987 \
             (0x88AB = 34987; IEEE RA registry; IETF ietf-ethertypes value 34987); \
             got line: {powerlink_line:?}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 5 (ARP ethertype=None → `—`)
    /// The ARP row in `wirerust protocols --all` displays `—` (em dash, U+2014) in the
    /// EtherType column. ARP has `ethertype: None` in `KNOWN_PROTOCOLS`.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_001_arp_ethertype_dash() {
        let output = bin()
            .args(["protocols", "--all"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // Locate the ARP data row. ARP is unique in KNOWN_PROTOCOLS by name.
        // Filter for data rows (lines containing "IT" or "ICS" to exclude headers).
        let arp_line = stdout
            .lines()
            .find(|line| {
                // Match a data row containing "ARP" alongside a category marker.
                line.contains("ARP")
                    && (line.contains("ICS") || line.contains("IT") || line.contains("[L2]"))
            })
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.18.001 PC-5: ARP data row must appear in --all output; \
                     stdout:\n{stdout}"
                )
            });
        // Em dash U+2014 (—) in EtherType column for entries with ethertype=None.
        assert!(
            arp_line.contains('\u{2014}') || arp_line.contains("—"),
            "BC-2.18.001 PC-5: ARP row EtherType column must be '—' (em dash U+2014; \
             ARP has ethertype=None in KNOWN_PROTOCOLS); got line: {arp_line:?}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.18.002 — JSON renderer tests
    // -----------------------------------------------------------------------

    /// BC-2.18.002 v1.1 Postcondition 6 / Invariant 1
    /// `wirerust protocols --all --json` output is valid JSON; `"protocols"` array
    /// length equals `all_protocols().len()` (== 30).
    ///
    /// No shell-out to `jq` — parsed directly with `serde_json`.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_002_json_schema_valid() {
        let output = bin()
            .args(["protocols", "--all", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.18.002 PC-6: `wirerust protocols --all --json` must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        let arr = json["protocols"]
            .as_array()
            .expect("BC-2.18.002 PC-2: 'protocols' field must be a JSON array");
        let expected_len = all_protocols().len(); // == 30
        assert_eq!(
            arr.len(),
            expected_len,
            "BC-2.18.002 Invariant 1: 'protocols' array length must equal \
             all_protocols().len() ({expected_len}); got {}",
            arr.len()
        );
    }

    /// BC-2.18.002 v1.1 Invariant 2
    /// Every JSON entry with `"port_detectable": false` has `"canonical_ports": []`.
    ///
    /// Invariant 2 holds one-way: `port_detectable: false` ⇒ `canonical_ports: []`.
    /// (ARP is the counterexample to the converse: ARP has empty ports, ethertype=null,
    /// port_detectable=false — the iff was relaxed in BC v1.1.)
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_002_l2_entries_no_ports() {
        let output = bin()
            .args(["protocols", "--all", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("valid JSON (BC-2.18.002 PC-6)");
        let arr = json["protocols"]
            .as_array()
            .expect("'protocols' must be an array");
        for entry in arr {
            let name = entry["name"].as_str().unwrap_or("(unnamed)");
            if !entry["port_detectable"]
                .as_bool()
                .expect("port_detectable must be boolean")
            {
                let ports = entry["canonical_ports"]
                    .as_array()
                    .expect("canonical_ports must be an array");
                assert!(
                    ports.is_empty(),
                    "BC-2.18.002 Invariant 2: entry '{}' has port_detectable=false but \
                     canonical_ports={:?} (must be [])",
                    name,
                    ports
                );
            }
        }
    }

    /// BC-2.18.002 v1.1 Invariant 1 / EC-001
    /// `wirerust protocols --supported --json` produces a `"protocols"` array with
    /// exactly 7 elements (== `supported_protocols().len()`).
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_002_supported_flag_matches_function() {
        let output = bin()
            .args(["protocols", "--supported", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("valid JSON (BC-2.18.002 PC-6)");
        let arr = json["protocols"]
            .as_array()
            .expect("'protocols' must be an array");
        let expected_len = supported_protocols().len(); // == 7
        assert_eq!(
            arr.len(),
            expected_len,
            "BC-2.18.002 EC-001: `wirerust protocols --supported --json` 'protocols' array \
             must have {expected_len} entries (== supported_protocols().len()); got {}",
            arr.len()
        );
    }

    /// BC-2.18.002 v1.1 EC-003; DF-CANONICAL-FRAME-HOLDOUT-001
    /// GOOSE entry in `wirerust protocols --unsupported --json` has:
    ///   `"ethertype": 35000`, `"transport": "LinkLayer"`, `"category": "ICS"`
    ///
    /// Canonical value source: IEC 61850-8-1 §4; IEEE RA registry "IEC GOOSE"
    /// (0x88B8 = 35000 decimal). Category must be "ICS" not "L2" — ADR-012 Decision 7.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_002_goose_json_canonical() {
        let output = bin()
            .args(["protocols", "--unsupported", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("valid JSON (BC-2.18.002 PC-6)");
        let arr = json["protocols"]
            .as_array()
            .expect("'protocols' must be an array");
        let goose = arr
            .iter()
            .find(|e| e["name"].as_str().is_some_and(|n| n.contains("GOOSE")))
            .unwrap_or_else(|| {
                panic!(
                    "DF-CANONICAL-FRAME-HOLDOUT-001: GOOSE entry must appear in \
                     --unsupported JSON output; full array:\n{arr:?}"
                )
            });
        // ethertype = 35000 (0x88B8; IEC 61850-8-1 §4; IEEE RA "IEC GOOSE")
        assert_eq!(
            goose["ethertype"],
            serde_json::json!(35000),
            "DF-CANONICAL-FRAME-HOLDOUT-001: GOOSE 'ethertype' must be 35000 \
             (0x88B8; IEC 61850-8-1 §4; IEEE RA registry \"IEC GOOSE\")"
        );
        // transport = "LinkLayer"
        assert_eq!(
            goose["transport"],
            serde_json::json!("LinkLayer"),
            "BC-2.18.002 EC-003: GOOSE 'transport' must be \"LinkLayer\""
        );
        // category = "ICS" (NOT "L2" — ADR-012 Decision 7; ProtocolCategory has two variants)
        assert_eq!(
            goose["category"],
            serde_json::json!("ICS"),
            "BC-2.18.002 EC-003 / ADR-012 Decision 7: GOOSE 'category' must be \"ICS\" \
             (not \"L2\" — ProtocolCategory has exactly two variants: ICS, IT)"
        );
    }

    /// BC-2.18.002 v1.1 EC-004; DF-CANONICAL-FRAME-HOLDOUT-001
    /// BACnet/IP entry in `wirerust protocols --unsupported --json` has:
    ///   `"transport": "UDP"`, `"canonical_ports": [47808]`
    ///
    /// Canonical value source: ASHRAE 135-2016 Annex J §J.2.1
    /// "UDP Port Number 47808 (0xBAC0)". Port 47808 ∉ SUPPORTED_PORTS, so
    /// BACnet/IP is unsupported.
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_002_bacnet_json_canonical() {
        let output = bin()
            .args(["protocols", "--unsupported", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("valid JSON (BC-2.18.002 PC-6)");
        let arr = json["protocols"]
            .as_array()
            .expect("'protocols' must be an array");
        let bacnet = arr
            .iter()
            .find(|e| e["name"].as_str().is_some_and(|n| n.contains("BACnet")))
            .unwrap_or_else(|| {
                panic!(
                    "DF-CANONICAL-FRAME-HOLDOUT-001: BACnet/IP entry must appear in \
                     --unsupported JSON output; full array:\n{arr:?}"
                )
            });
        // transport = "UDP"
        assert_eq!(
            bacnet["transport"],
            serde_json::json!("UDP"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: BACnet/IP 'transport' must be \"UDP\" \
             (ASHRAE 135-2016 Annex J §J.2.1 — UDP-only canonical model)"
        );
        // canonical_ports = [47808] (0xBAC0; ASHRAE 135-2016 Annex J §J.2.1)
        assert_eq!(
            bacnet["canonical_ports"],
            serde_json::json!([47808]),
            "DF-CANONICAL-FRAME-HOLDOUT-001: BACnet/IP 'canonical_ports' must be [47808] \
             (0xBAC0; ASHRAE 135-2016 Annex J §J.2.1)"
        );
    }

    /// BC-2.18.002 v1.1 EC-005; DF-CANONICAL-FRAME-HOLDOUT-001
    /// Modbus/TCP entry in `wirerust protocols --supported --json` has:
    ///   `"transport": "TCP"`, `"canonical_ports": [502]`, `"supported": true`
    ///
    /// Canonical value source: IANA port registry + Modbus Application Protocol
    /// Specification v1.1b3 §4.3.1 "Well-Known TCP Port 0+502".
    ///
    /// Regression guard: fails if the `protocols` subcommand is removed or stops exiting 0.
    #[test]
    fn test_BC_2_18_002_modbus_json_canonical() {
        let output = bin()
            .args(["protocols", "--supported", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("valid JSON (BC-2.18.002 PC-6)");
        let arr = json["protocols"]
            .as_array()
            .expect("'protocols' must be an array");
        let modbus = arr
            .iter()
            .find(|e| e["name"].as_str().is_some_and(|n| n.contains("Modbus")))
            .unwrap_or_else(|| {
                panic!(
                    "DF-CANONICAL-FRAME-HOLDOUT-001: Modbus/TCP entry must appear in \
                     --supported JSON output; full array:\n{arr:?}"
                )
            });
        // transport = "TCP"
        assert_eq!(
            modbus["transport"],
            serde_json::json!("TCP"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: Modbus/TCP 'transport' must be \"TCP\""
        );
        // canonical_ports = [502] (IANA + Modbus App Protocol v1.1b3 §4.3.1)
        assert_eq!(
            modbus["canonical_ports"],
            serde_json::json!([502]),
            "DF-CANONICAL-FRAME-HOLDOUT-001: Modbus/TCP 'canonical_ports' must be [502] \
             (IANA port registry + Modbus App Protocol v1.1b3 §4.3.1)"
        );
        // supported = true
        assert_eq!(
            modbus["supported"],
            serde_json::json!(true),
            "DF-CANONICAL-FRAME-HOLDOUT-001: Modbus/TCP 'supported' must be true \
             (port 502 ∈ SUPPORTED_PORTS)"
        );
    }

    // -----------------------------------------------------------------------
    // Coverage gap tests (Pass-1 findings)
    // -----------------------------------------------------------------------

    /// BC-2.12.022 EC-152-4 — spurious positional argument rejected
    /// `wirerust protocols somefile.pcap` exits non-zero; clap rejects the
    /// unexpected positional argument (the `protocols` subcommand accepts no
    /// positional args — only `--all`, `--supported`, `--unsupported`, `--json`).
    #[test]
    fn test_BC_2_12_022_protocols_spurious_positional_error() {
        bin()
            .args(["protocols", "somefile.pcap"])
            .assert()
            .failure();
    }

    /// BC-2.18.002 v1.1 EC-152-10; DF-CANONICAL-FRAME-HOLDOUT-001
    /// ARP entry in `wirerust protocols --supported --json` has:
    ///   `"transport": "LinkLayer"`, `"ethertype": null`,
    ///   `"canonical_ports": []`, `"supported": true`
    ///
    /// ARP is in the supported set via the explicit `|| p.name == "ARP"` branch
    /// in `supported_protocols()` (BC-2.18.003 Invariant 3). Its L2 transport means
    /// it has no canonical TCP/UDP ports and no EtherType value in KNOWN_PROTOCOLS.
    #[test]
    fn test_BC_2_18_002_arp_json_canonical() {
        let output = bin()
            .args(["protocols", "--supported", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value =
            serde_json::from_str(&stdout).expect("valid JSON (BC-2.18.002 PC-6)");
        let arr = json["protocols"]
            .as_array()
            .expect("'protocols' must be an array");
        let arp = arr
            .iter()
            .find(|e| e["name"].as_str().is_some_and(|n| n == "ARP"))
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.18.002 EC-152-10: ARP must appear in --supported JSON output \
                     (ARP is in supported_protocols() via || p.name == \"ARP\"); \
                     full array:\n{arr:?}"
                )
            });
        assert_eq!(
            arp["transport"],
            serde_json::json!("LinkLayer"),
            "BC-2.18.002 EC-152-10: ARP 'transport' must be \"LinkLayer\""
        );
        assert_eq!(
            arp["ethertype"],
            serde_json::Value::Null,
            "BC-2.18.002 EC-152-10: ARP 'ethertype' must be null (ethertype=None in \
             KNOWN_PROTOCOLS)"
        );
        assert_eq!(
            arp["canonical_ports"],
            serde_json::json!([]),
            "BC-2.18.002 EC-152-10: ARP 'canonical_ports' must be [] (LinkLayer entry, \
             no TCP/UDP port)"
        );
        assert_eq!(
            arp["supported"],
            serde_json::json!(true),
            "BC-2.18.002 EC-152-10: ARP 'supported' must be true \
             (in supported_protocols() via || p.name == \"ARP\")"
        );
    }

    /// BC-2.12.022 Invariant 3 — default (no flag) equals `--all`
    /// `wirerust protocols` with no filter flag prints the same 30 data rows as
    /// `wirerust protocols --all` (BC-2.12.022 Invariant 3: default == --all).
    ///
    /// Row counts exclude lines starting with "NOTE:" so the port-102 footnote
    /// (which names protocol names per AC-152-004) does not inflate the count.
    #[test]
    fn test_BC_2_12_022_default_equals_all() {
        let all = all_protocols();

        // Count data rows in `--all` output (canonical reference).
        let output_all = bin()
            .args(["protocols", "--all"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout_all = String::from_utf8(output_all).expect("utf-8 stdout");
        let count_all = stdout_all
            .lines()
            .filter(|line| !line.starts_with("NOTE:"))
            .filter(|line| all.iter().any(|p| line.contains(p.name)))
            .count();

        // Count data rows in default (no-flag) output.
        let output_default = bin()
            .args(["protocols"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout_default = String::from_utf8(output_default).expect("utf-8 stdout");
        let count_default = stdout_default
            .lines()
            .filter(|line| !line.starts_with("NOTE:"))
            .filter(|line| all.iter().any(|p| line.contains(p.name)))
            .count();

        assert_eq!(
            count_default, count_all,
            "BC-2.12.022 Invariant 3: `wirerust protocols` (no flag) must print the same \
             number of data rows as `wirerust protocols --all`; \
             got default={count_default}, --all={count_all}"
        );
        assert_eq!(
            count_all,
            all.len(),
            "BC-2.12.022 Invariant 3: both default and --all must print exactly {} data rows \
             (== all_protocols().len()); got {count_all}",
            all.len()
        );
    }

    /// BC-2.18.002 v1.1 Invariant 1 / PC-4
    /// `wirerust protocols --json --all` `"protocols"` array preserves the declaration
    /// order of `all_protocols()` (names must appear in the same sequence).
    ///
    /// Invariant 1: JSON output is a faithful, order-preserving serialisation of the
    /// catalog. Shuffling the array would break deterministic holdout comparisons and
    /// the BC-2.18.002 PC-4 ordering guarantee.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_18_002_json_declaration_order() {
        let output = bin()
            .args(["protocols", "--json", "--all"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.18.002 PC-6: `wirerust protocols --json --all` must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        let arr = json["protocols"]
            .as_array()
            .expect("BC-2.18.002 PC-2: 'protocols' field must be a JSON array");

        // Extract the name sequence from JSON output.
        let json_names: Vec<&str> = arr
            .iter()
            .map(|e| {
                e["name"]
                    .as_str()
                    .expect("BC-2.18.002: every protocols entry must have a string 'name' field")
            })
            .collect();

        // Build the expected sequence from all_protocols() declaration order.
        let catalog_names: Vec<&str> = all_protocols().iter().map(|p| p.name).collect();

        assert_eq!(
            json_names, catalog_names,
            "BC-2.18.002 Invariant 1 / PC-4: JSON 'protocols' array must preserve \
             all_protocols() declaration order; \
             json_names={json_names:?}, catalog_names={catalog_names:?}"
        );
    }

    /// BC-2.12.022 output-routing: `protocols --json=<PATH>` writes the JSON
    /// to the given file and does NOT emit the terminal table header to stdout.
    ///
    /// Regression guard for the wave-68 F-W68-01 silent-failure fix: previously
    /// `--json=<PATH>` printed JSON to stdout and silently wrote no file.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_12_022_json_path_writes_file() {
        let tmp = std::env::temp_dir().join(format!(
            "wirerust_test_protocols_{}.json",
            std::process::id()
        ));
        // Clean up any leftover from a previous run.
        let _ = std::fs::remove_file(&tmp);

        let json_arg = format!("--json={}", tmp.display());
        let output = bin()
            .args(["protocols", "--all", &json_arg])
            .assert()
            .success()
            .get_output()
            .clone();

        // The file must exist and contain valid JSON with a "protocols" array of 30 entries.
        assert!(
            tmp.exists(),
            "BC-2.12.022: `protocols --json=<PATH>` must create the output file at {tmp:?}"
        );
        let contents =
            std::fs::read_to_string(&tmp).expect("should be able to read the written JSON file");
        let _ = std::fs::remove_file(&tmp); // clean up

        let json: serde_json::Value = serde_json::from_str(&contents).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.022: file written by `protocols --json=<PATH>` must be valid JSON; \
                 parse error: {e}\ncontents:\n{contents}"
            )
        });
        let arr = json["protocols"]
            .as_array()
            .expect("BC-2.12.022: JSON file must have a top-level 'protocols' array");
        let expected = all_protocols().len();
        assert_eq!(
            arr.len(),
            expected,
            "BC-2.12.022: 'protocols' array must have {expected} entries (--all); got {}",
            arr.len()
        );

        // When JSON is routed to a file, stdout must be completely empty.
        let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
        assert!(
            stdout.trim().is_empty(),
            "BC-2.12.022: stdout must be empty when --json=<PATH> routes output to a file; \
             got:\n{stdout}"
        );
    }

    /// BC-2.12.022 output-routing: `protocols --output-format json` emits
    /// parseable JSON with a `"protocols"` array to stdout.
    ///
    /// Regression guard for the wave-68 F-W68-01 fix: previously
    /// `--output-format json` was silently ignored and the terminal table was
    /// printed instead.
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_12_022_output_format_json() {
        let output = bin()
            .args(["protocols", "--output-format", "json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.022: `protocols --output-format json` must produce valid JSON on \
                 stdout; parse error: {e}\nstdout:\n{stdout}"
            )
        });
        assert!(
            json.get("protocols").is_some(),
            "BC-2.12.022: JSON output from `--output-format json` must contain a \
             top-level 'protocols' key; stdout:\n{stdout}"
        );
        assert!(
            json["protocols"].is_array(),
            "BC-2.12.022: 'protocols' value must be a JSON array; stdout:\n{stdout}"
        );
    }

    /// BC-2.12.022 output-routing: `protocols --csv` and
    /// `protocols --output-format csv` exit non-zero and report the
    /// unsupported format on stderr.
    ///
    /// Regression guard for the wave-68 F-W68-01 fix: previously these flags
    /// silently fell back to the terminal table (no-op, no error).
    #[allow(non_snake_case)]
    #[test]
    fn test_BC_2_12_022_csv_rejected() {
        // --csv flag variant.
        let output_csv_flag = bin()
            .args(["protocols", "--csv"])
            .assert()
            .failure()
            .get_output()
            .clone();
        let stderr_csv_flag = String::from_utf8(output_csv_flag.stderr).expect("utf-8 stderr");
        assert!(
            stderr_csv_flag.to_lowercase().contains("csv"),
            "BC-2.12.022: `protocols --csv` must mention 'csv' in stderr error message; \
             stderr:\n{stderr_csv_flag}"
        );

        // --output-format csv variant.
        let output_fmt_csv = bin()
            .args(["protocols", "--output-format", "csv"])
            .assert()
            .failure()
            .get_output()
            .clone();
        let stderr_fmt_csv = String::from_utf8(output_fmt_csv.stderr).expect("utf-8 stderr");
        assert!(
            stderr_fmt_csv.to_lowercase().contains("csv"),
            "BC-2.12.022: `protocols --output-format csv` must mention 'csv' in stderr \
             error message; stderr:\n{stderr_fmt_csv}"
        );
    }
}

// ---------------------------------------------------------------------------
// STORY-154: `--coverage-gaps` flag + CoverageGapsSummary tri-state report.
//
// BCs: BC-2.12.023 v1.2 (flag opt-in, analyze --all independence, section wiring)
//      BC-2.12.024 v1.1 (L2 caveat, port-102 collision note, tri-state classification)
//
// GREEN STATUS:
//   All 20 tests pass. `--coverage-gaps` is fully implemented and wired.
//   Crafted gap fixtures (gap-tcp102.pcap, gap-udp47808.pcap, gap-tcp47808.pcap,
//   gap-tcp9600.pcap, gap-tcp53.pcap) were generated during the Green phase and
//   reside in tests/fixtures/. TCP gap tests pass --http to build the reassembler
//   (analyzer-present guard, BC-2.05.010); UDP gap tests need no analyzer flag
//   (decode-loop path, ADR-012 Dec 10).
// ---------------------------------------------------------------------------
mod story_154 {
    #![allow(non_snake_case)]

    use assert_cmd::Command;

    /// Default fixture for tests that only need a valid, parseable pcap.
    /// Smallest fixture in the suite (1,209 B); exercises TCP flows on port 80.
    const ANALYZE_FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

    /// Fixture with Modbus TCP/502 traffic — used by test_BC_2_12_024_tcp_502_absent_from_gap_report
    /// to verify that Rule 5 of classify() routes port-502 flows to DispatchTarget::Modbus
    /// before the None-target arm can fire (EC-154-11).
    const MODBUS_FIXTURE: &str = "tests/fixtures/modbus-write.pcap";

    // F4-FIXTURE-NEED-001 crafted gap fixtures (generated by STORY-154 implementation).
    // Each fixture is a minimal pcap (1–2 packets) with traffic on the target port.
    // TCP fixtures contain a single SYN packet; finalize() closes the flow via Timeout
    // and increments unclassified_port_counts. UDP fixture contains a single UDP datagram.

    /// Single TCP SYN to port 102 (S7comm/MMS/ISO-on-TCP collision port).
    /// With --http: reassembly + HttpAnalyzer present → analyzer-present guard passes.
    const GAP_TCP102_FIXTURE: &str = "tests/fixtures/gap-tcp102.pcap";

    /// Single UDP datagram to port 47808 (BACnet/IP; ASHRAE 135-2016 Annex J §J.2.1).
    /// UDP gap detection does not require --http (decode-loop path; no reassembler needed).
    const GAP_UDP47808_FIXTURE: &str = "tests/fixtures/gap-udp47808.pcap";

    /// Single TCP SYN to port 47808 (transport mismatch: BACnet/IP is UDP-only in catalog).
    /// With --http: reassembly + HttpAnalyzer present → flow classified as Unknown.
    const GAP_TCP47808_FIXTURE: &str = "tests/fixtures/gap-tcp47808.pcap";

    /// Single TCP SYN to port 9600 (no catalog match → unknown).
    /// With --http: reassembly + HttpAnalyzer present → gap entry created.
    const GAP_TCP9600_FIXTURE: &str = "tests/fixtures/gap-tcp9600.pcap";

    /// Single TCP SYN to port 53 (DNS is UDP-only in catalog → TCP/53 is unknown).
    /// With --http: reassembly + HttpAnalyzer present → gap entry created.
    const GAP_TCP53_FIXTURE: &str = "tests/fixtures/gap-tcp53.pcap";

    fn bin() -> Command {
        Command::cargo_bin("wirerust").expect("wirerust binary must be built")
    }

    // -----------------------------------------------------------------------
    // BC-2.12.023 — flag opt-in semantics, analyze --all independence
    // -----------------------------------------------------------------------

    /// BC-2.12.023 Invariant 1 / EC-154-2:
    /// `wirerust analyze --all` does NOT produce a `CoverageGapsSummary` section.
    /// The `--all` and `--coverage-gaps` flags are independent.
    ///
    /// GREEN REGRESSION GUARD: `--all` never implies `--coverage-gaps` (Invariant 1).
    /// Fails if `--all` is changed to also trigger `--coverage-gaps`.
    #[test]
    fn test_BC_2_12_023_all_without_coverage_gaps() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--all"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            !stdout.contains("CoverageGapsSummary"),
            "BC-2.12.023 Invariant 1: `analyze --all` must NOT produce a \
             CoverageGapsSummary section; `--all` and `--coverage-gaps` are independent; \
             stdout:\n{stdout}"
        );
    }

    /// BC-2.12.023 Invariant 1 / BC-2.12.023 PC-1 — combined:
    /// `wirerust analyze --all --coverage-gaps` exits 0 and produces a `CoverageGapsSummary`
    /// section. The `--all` selector and `--coverage-gaps` reporter are orthogonal; combining
    /// them must work: `--all` routes all traffic to enabled analyzers while `--coverage-gaps`
    /// independently appends the CoverageGapsSummary after all Findings.
    ///
    /// STORY-154-ALL-COVERAGEGAPS-TEST-001: exercises the combination that was previously
    /// untested. Uses GAP_TCP9600_FIXTURE with `--all` (which enables `--http`, satisfying the
    /// analyzer-present guard per BC-2.05.010) and `--coverage-gaps` (which emits the section).
    /// Port 9600 flows to DispatchTarget::None → CoverageGapsSummary shows TCP/9600 as "unknown".
    ///
    /// Fails if: (1) `--all` and `--coverage-gaps` conflict in clap config,
    /// (2) CoverageGapsSummary section is absent, or (3) TCP/9600 row is missing from the report.
    #[test]
    fn test_BC_2_12_023_all_with_coverage_gaps_combination() {
        let output = bin()
            .args(["analyze", GAP_TCP9600_FIXTURE, "--all", "--coverage-gaps"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // CoverageGapsSummary must appear: --coverage-gaps flag was passed.
        assert!(
            stdout.contains("CoverageGapsSummary"),
            "STORY-154-ALL-COVERAGEGAPS-TEST-001: `analyze --all --coverage-gaps` must \
             produce a CoverageGapsSummary section; stdout:\n{stdout}"
        );
        // TCP/9600 must appear as an "unknown" gap entry: --all enables HTTP (satisfying
        // the analyzer-present guard), and port 9600 has no catalog match → unknown state.
        let tcp9600_row_is_unknown = stdout
            .lines()
            .any(|l| l.contains("TCP/9600") && l.ends_with("unknown"));
        assert!(
            tcp9600_row_is_unknown,
            "STORY-154-ALL-COVERAGEGAPS-TEST-001: `analyze --all --coverage-gaps` must show \
             TCP/9600 as 'unknown' in CoverageGapsSummary; stdout:\n{stdout}"
        );
    }

    /// BC-2.12.023 Invariant 5 / EC-154-6:
    /// `wirerust protocols --coverage-gaps` exits non-zero (clap error).
    /// `--coverage-gaps` is only valid on the `analyze` subcommand.
    ///
    /// GREEN REGRESSION GUARD: `--coverage-gaps` is scoped to `analyze` only.
    /// Fails if `--coverage-gaps` is accidentally added to the `protocols` subcommand.
    #[test]
    fn test_BC_2_12_023_protocols_coverage_gaps_error() {
        bin()
            .args(["protocols", "--coverage-gaps"])
            .assert()
            .failure();
    }

    /// BC-2.12.023 PC-1 / EC-154-1:
    /// `wirerust analyze` without `--coverage-gaps` produces no `CoverageGapsSummary`.
    ///
    /// GREEN REGRESSION GUARD: `--coverage-gaps` is strictly opt-in (flag independence).
    /// Fails if `analyze` is changed to render CoverageGapsSummary without the flag.
    #[test]
    fn test_BC_2_12_023_no_coverage_gaps_no_section() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            !stdout.contains("CoverageGapsSummary"),
            "BC-2.12.023 PC-2: `analyze` without `--coverage-gaps` must NOT render \
             CoverageGapsSummary; stdout:\n{stdout}"
        );
    }

    /// BC-2.12.023 PC-1 / AC-154-002:
    /// `wirerust analyze <pcap> --coverage-gaps` with known-unclassified traffic
    /// produces a `CoverageGapsSummary` section with at least one entry (AC-154-002 ≥1).
    ///
    /// GREEN REGRESSION GUARD: uses GAP_TCP9600_FIXTURE (TCP/9600, no catalog match)
    /// with --http so the reassembler is built and the analyzer-present guard fires
    /// (BC-2.05.010). Port 9600 → DispatchTarget::None → unknown gap entry rendered.
    /// Fails if: --coverage-gaps section disappears, gap counting regresses, or
    /// TCP/9600 is inadvertently added to the classify() routing table.
    #[test]
    fn test_BC_2_12_023_coverage_gaps_counts_unclassified() {
        let output = bin()
            .args(["analyze", GAP_TCP9600_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            stdout.contains("CoverageGapsSummary"),
            "BC-2.12.023 PC-1: --coverage-gaps must produce CoverageGapsSummary section; \
             stdout:\n{stdout}"
        );
        // AC-154-002: at least one gap entry must be present (count= line for TCP/9600).
        assert!(
            stdout.contains("count="),
            "BC-2.12.023 / AC-154-002: CoverageGapsSummary must contain ≥1 gap entry \
             (TCP/9600 → unknown); stdout:\n{stdout}"
        );
    }

    /// BC-2.12.023 PC-1 / Invariant 3 / AC-154-003:
    /// `--coverage-gaps` produces a `CoverageGapsSummary` named section in the output.
    ///
    /// GREEN REGRESSION GUARD: the section header must appear whenever --coverage-gaps
    /// is passed (Invariant 3), even when the pcap has no unclassified traffic.
    #[test]
    fn test_BC_2_12_023_coverage_gaps_flag_produces_section() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--coverage-gaps"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            stdout.contains("CoverageGapsSummary"),
            "BC-2.12.023 PC-1 / Invariant 3: --coverage-gaps must append \
             CoverageGapsSummary named section after all Findings; \
             stdout:\n{stdout}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.12.023 / BC-2.12.024 — JSON mode
    // -----------------------------------------------------------------------

    /// BC-2.12.023 PC-3 / AC-154-007 / EC-154-5:
    /// `--json --coverage-gaps` produces a JSON object with a `"coverage_gaps"` key.
    ///
    /// GREEN REGRESSION GUARD: JSON mode must include the top-level `"coverage_gaps"` key
    /// whenever --coverage-gaps is set. Fails if the JSON serialization path drops the field.
    /// NOTE: `--json` is a top-level Cli flag; it can appear anywhere in the args.
    #[test]
    fn test_BC_2_12_023_json_coverage_gaps_key() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--coverage-gaps", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.023 PC-3: --json --coverage-gaps must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        assert!(
            json.get("coverage_gaps").is_some(),
            "BC-2.12.023 PC-3: JSON output with --coverage-gaps must have a \
             top-level 'coverage_gaps' key; stdout:\n{stdout}"
        );
    }

    // -----------------------------------------------------------------------
    // BC-2.12.024 — L2 caveat, port-102 footnote, tri-state classification
    // -----------------------------------------------------------------------

    /// BC-2.12.024 PC-1 / Invariant 1 / AC-154-004 / EC-001 (BC-2.12.024):
    /// `CoverageGapsSummary` ALWAYS includes the L2/multicast structural caveat,
    /// even when the entries array is empty.
    ///
    /// GREEN REGRESSION GUARD: the L2/multicast caveat text must appear unconditionally
    /// whenever --coverage-gaps is set (Invariant 1), regardless of pcap content.
    /// Fails if the caveat is made conditional on gap entries existing.
    #[test]
    fn test_BC_2_12_024_l2_caveat_always_present() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--coverage-gaps"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // Check for a stable substring of L2_CAVEAT_TEXT (AC-154-004).
        assert!(
            stdout.contains("Layer-2 protocols") || stdout.contains("TCP and UDP flows"),
            "BC-2.12.024 PC-1 / Invariant 1: CoverageGapsSummary must always include \
             the L2/multicast caveat text; stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-2 / Invariant 2 / AC-154-005 / EC-003 (BC-2.12.024):
    /// Port-102 collision footnote present when TCP/102 has a non-zero count.
    /// Names all four protocols: S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2.
    ///
    /// Uses GAP_TCP102_FIXTURE (single SYN to port 102). --http enables TCP reassembly
    /// + HttpAnalyzer (analyzer-present guard); port-102 flow → DispatchTarget::None
    ///   (classify() has no port-102 rule) → gap counter incremented → PORT_102_NOTE rendered.
    #[test]
    fn test_BC_2_12_024_port102_footnote_on_tcp102_traffic() {
        let output = bin()
            .args(["analyze", GAP_TCP102_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // PORT_102_NOTE must appear adjacent to TCP/102 entry (BC-2.12.024 PC-2).
        assert!(
            stdout.contains("S7comm") || stdout.contains("ISO-on-TCP"),
            "BC-2.12.024 PC-2: port-102 collision footnote must appear when TCP/102 \
             count > 0; stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-3 / Invariant 2 / AC-154-005 / EC-001 (BC-2.12.024):
    /// Port-102 footnote is row-conditional: absent when TCP/102 count is zero.
    ///
    /// GREEN REGRESSION GUARD: uses GAP_TCP9600_FIXTURE (TCP/9600) with --http so the
    /// reassembler is built and gap machinery IS active (analyzer-present guard fires).
    /// A TCP/9600 gap entry IS rendered (confirms the section is live), while the
    /// PORT_102_NOTE strings ("ISO-on-TCP", "S7comm-plus") must remain absent — proving
    /// the footnote is row-conditional, not unconditionally suppressed.
    /// Fails if: port-102 footnote appears without TCP/102 traffic, or gap machinery
    /// is inactive (vacuous absence).
    #[test]
    fn test_BC_2_12_024_port102_footnote_absent_without_tcp102() {
        // GAP_TCP9600_FIXTURE has TCP/9600 traffic and no TCP/102 flows.
        // --http builds the reassembler so gap counting is active (analyzer-present guard).
        let output = bin()
            .args(["analyze", GAP_TCP9600_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // Gap machinery is active: a TCP/9600 entry must appear (non-vacuous anchor).
        assert!(
            stdout.contains("count="),
            "BC-2.12.024 PC-3 non-vacuity: gap section must have ≥1 entry (TCP/9600) \
             proving gap machinery is active; stdout:\n{stdout}"
        );
        // PORT_102_NOTE must be absent (no TCP/102 traffic in fixture).
        assert!(
            !stdout.contains("ISO-on-TCP") && !stdout.contains("S7comm-plus"),
            "BC-2.12.024 PC-3 / Invariant 2: port-102 collision footnote must NOT appear \
             when TCP/102 count == 0; stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-2 / DF-CANONICAL-FRAME-HOLDOUT-001:
    /// The port-102 collision footnote (PORT_102_NOTE constant) names all four
    /// protocols sharing TCP/102 via ISO-on-TCP/TPKT framing (RFC 1006):
    ///   S7comm, S7comm-plus, IEC 61850 MMS, ICCP/TASE.2.
    ///
    /// Uses GAP_TCP102_FIXTURE with --http. See test_BC_2_12_024_port102_footnote_on_tcp102_traffic.
    #[test]
    fn test_BC_2_12_024_port102_note_names_all_four() {
        let output = bin()
            .args(["analyze", GAP_TCP102_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // DF-CANONICAL-FRAME-HOLDOUT-001: PORT_102_NOTE must name all four protocols.
        assert!(
            stdout.contains("S7comm"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: PORT_102_NOTE must name 'S7comm'; \
             stdout:\n{stdout}"
        );
        assert!(
            stdout.contains("S7comm-plus"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: PORT_102_NOTE must name 'S7comm-plus'; \
             stdout:\n{stdout}"
        );
        assert!(
            stdout.contains("IEC 61850 MMS"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: PORT_102_NOTE must name 'IEC 61850 MMS'; \
             stdout:\n{stdout}"
        );
        assert!(
            stdout.contains("ICCP"),
            "DF-CANONICAL-FRAME-HOLDOUT-001: PORT_102_NOTE must name 'ICCP' (TASE.2); \
             stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-4 / DF-CANONICAL-FRAME-HOLDOUT-001 / EC-002 (BC-2.12.024):
    /// `(Udp, 47808)` classifies as `known-unsupported` with name "BACnet/IP".
    /// BACnet/IP uses UDP port 0xBAC0 = 47808 per ASHRAE 135-2016 Annex J §J.2.1.
    ///
    /// Uses GAP_UDP47808_FIXTURE (single UDP datagram to port 47808). UDP gap detection
    /// runs in the decode loop regardless of --http; no --http flag needed.
    #[test]
    fn test_BC_2_12_024_bacnet_known_unsupported() {
        let output = bin()
            .args(["analyze", GAP_UDP47808_FIXTURE, "--coverage-gaps"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // DF-CANONICAL-FRAME-HOLDOUT-001: state known-unsupported; name BACnet/IP.
        assert!(
            stdout.contains("known-unsupported"),
            "BC-2.12.024 PC-4 / DF-CANONICAL-FRAME-HOLDOUT-001: (Udp, 47808) entry \
             must show state 'known-unsupported'; stdout:\n{stdout}"
        );
        assert!(
            stdout.contains("BACnet/IP"),
            "BC-2.12.024 PC-4 / DF-CANONICAL-FRAME-HOLDOUT-001: (Udp, 47808) entry \
             must name 'BACnet/IP' (ASHRAE 135-2016 Annex J §J.2.1); stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-4 / EC-004 (BC-2.12.024):
    /// `(Tcp, 9600)` classifies as `unknown` (no catalog entry for this port).
    ///
    /// Uses GAP_TCP9600_FIXTURE (single SYN to port 9600). --http enables TCP reassembly
    /// + HttpAnalyzer (analyzer-present guard); port-9600 flow → DispatchTarget::None → unknown.
    #[test]
    fn test_BC_2_12_024_unknown_port_state() {
        let output = bin()
            .args(["analyze", GAP_TCP9600_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            stdout.contains("unknown"),
            "BC-2.12.024 PC-4: (Tcp, 9600) must show state 'unknown'; \
             stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-4 / EC-009 (BC-2.12.024):
    /// `(Tcp, 47808)` classifies as `unknown` (transport mismatch — BACnet/IP is
    /// catalogued as Udp/47808 only; a Tcp observation of the same port has no
    /// matching catalog entry).
    ///
    /// Uses GAP_TCP47808_FIXTURE (single SYN to port 47808). --http enables TCP reassembly
    /// + HttpAnalyzer; port-47808 TCP flow → DispatchTarget::None → lookup_protocol_state
    ///   finds no Tcp/47808 catalog entry → Unknown (not KnownUnsupported).
    #[test]
    fn test_BC_2_12_024_tcp_47808_is_unknown() {
        let output = bin()
            .args(["analyze", GAP_TCP47808_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // State must be "unknown", NOT "known-unsupported" (transport mismatch).
        assert!(
            stdout.contains("unknown"),
            "BC-2.12.024 EC-009: (Tcp, 47808) must be 'unknown' (transport mismatch \
             — BACnet/IP is UDP-only in catalog); stdout:\n{stdout}"
        );
        assert!(
            !stdout.contains("known-unsupported"),
            "BC-2.12.024 EC-009: (Tcp, 47808) must NOT be 'known-unsupported'; \
             stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-4 / EC-010 (BC-2.12.024) / EC-154-14:
    /// `(Tcp, 53)` classifies as `unknown`. DNS is catalogued as UDP/53 only;
    /// TCP is not a supported DNS transport in the protocol catalog.
    ///
    /// Forward-note: STORY-154-DNS53-TCP-GAP-001 — DNS/53 TCP gap is a structural
    /// limitation of the current catalog (DNS over TCP exists in RFC 7766 but is
    /// not catalogued; this test guards the correct "unknown" classification of the
    /// TCP/53 observation against possible future catalog confusion).
    ///
    /// Uses GAP_TCP53_FIXTURE (single SYN to port 53). --http enables TCP reassembly
    /// + HttpAnalyzer; port-53 TCP flow → DispatchTarget::None → catalog lookup finds
    ///   only Udp/53 (DNS); no Tcp/53 entry → Unknown (EC-154-14 / STORY-154-DNS53-TCP-GAP-001).
    #[test]
    fn test_BC_2_12_024_tcp_53_is_unknown() {
        let output = bin()
            .args(["analyze", GAP_TCP53_FIXTURE, "--coverage-gaps", "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // State must be "unknown" — DNS catalog entry is Udp/53 only;
        // TCP/53 has no catalog match → Unknown (EC-154-14 / STORY-154-DNS53-TCP-GAP-001).
        assert!(
            stdout.contains("unknown"),
            "BC-2.12.024 EC-010 / EC-154-14: (Tcp, 53) must be 'unknown' — DNS is \
             UDP-only in catalog (STORY-154-DNS53-TCP-GAP-001); stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-4 / EC-154-11 / F-F3P14-001:
    /// `(Tcp, 502)` is ABSENT from `CoverageGapsSummary` under normal operation.
    /// `classify()` Rule 5 always routes port-502 flows to `DispatchTarget::Modbus`
    /// before the `None`-target arm fires; therefore port 502 never enters
    /// `unclassified_port_counts` via the analyze pipeline.
    ///
    /// GREEN REGRESSION GUARD: uses `modbus-write.pcap` (TCP/502 Modbus traffic)
    /// WITH `--modbus` so the reassembler IS built and `classify()` actually processes
    /// the TCP/502 flow → routes to `DispatchTarget::Modbus` (Rule 5 fires) → the
    /// None-target arm never fires for port 502 → no gap entry. This is non-vacuous:
    /// if Rule 5 regressed and port 502 were mis-routed to None, it would appear as a
    /// gap entry and this test would fail.
    #[test]
    fn test_BC_2_12_024_tcp_502_absent_from_gap_report() {
        let output = bin()
            .args(["analyze", MODBUS_FIXTURE, "--coverage-gaps", "--modbus"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        // Port 502 must NOT appear as a gap entry: Modbus is supported, and
        // classify() routes it before the None-target arm fires (EC-154-11).
        // We check that "502" does not appear as an entry in CoverageGapsSummary.
        // (The section header itself does not contain "502".)
        assert!(
            !stdout.contains("\"502\"")
                && !stdout.contains("port: 502")
                && !stdout.contains("TCP/502"),
            "BC-2.12.024 EC-154-11: (Tcp, 502) must be ABSENT from CoverageGapsSummary; \
             classify() Rule 5 routes Modbus flows before the None-target arm fires; \
             stdout:\n{stdout}"
        );
    }

    /// BC-2.12.024 PC-5 / AC-154-007:
    /// `--json --coverage-gaps` produces a JSON object where `"coverage_gaps"."caveat_l2"`
    /// is a non-null, non-empty string.
    ///
    /// GREEN REGRESSION GUARD: the L2 caveat must be serialized into the JSON output
    /// as a non-empty string at `coverage_gaps.caveat_l2`. Fails if the field is dropped,
    /// nulled, or emptied during JSON serialization.
    #[test]
    fn test_BC_2_12_024_json_has_caveat_field() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--coverage-gaps", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.024 PC-5: --json --coverage-gaps must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        let caveat = json
            .get("coverage_gaps")
            .and_then(|cg| cg.get("caveat_l2"))
            .and_then(|c| c.as_str());
        assert!(
            caveat.is_some_and(|s| !s.is_empty()),
            "BC-2.12.024 PC-5: JSON 'coverage_gaps.caveat_l2' must be a non-null, \
             non-empty string; stdout:\n{stdout}"
        );
    }

    // -----------------------------------------------------------------------
    // Pass-2 MEDIUM-1: JSON per-entry schema (BC-2.12.024 PC-5 / AC-154-007)
    // -----------------------------------------------------------------------

    /// BC-2.12.024 PC-5 / AC-154-007:
    /// JSON entry for (UDP, 47808) has the correct field types and values:
    ///   transport == "UDP", port == 47808 (integer), count >= 1,
    ///   state == "known-unsupported", name == "BACnet/IP", no collision_note.
    ///
    /// This test is NON-VACUOUS: it fails if `name` is omitted, `state` is wrong,
    /// or `port` is serialized as a string instead of an integer.
    ///
    /// Uses GAP_UDP47808_FIXTURE (single UDP datagram to port 47808). UDP gap detection
    /// runs in the decode loop; no --http flag needed.
    #[test]
    fn test_BC_2_12_024_json_entry_bacnet_schema() {
        let output = bin()
            .args(["analyze", GAP_UDP47808_FIXTURE, "--coverage-gaps", "--json"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.024 PC-5 / AC-154-007: --json --coverage-gaps must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        let entries = json["coverage_gaps"]["entries"]
            .as_array()
            .expect("coverage_gaps.entries must be a JSON array");
        let entry = entries
            .iter()
            .find(|e| e["port"].as_u64() == Some(47808))
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.12.024 / AC-154-007: entries[] must contain an element with port==47808; \
                     entries: {entries:?}"
                )
            });
        assert_eq!(
            entry["transport"].as_str(),
            Some("UDP"),
            "AC-154-007: entry.transport must be \"UDP\"; entry: {entry}"
        );
        // port MUST be an integer (not a string) per AC-154-007 schema.
        assert_eq!(
            entry["port"].as_u64(),
            Some(47808),
            "AC-154-007: entry.port must be integer 47808; entry: {entry}"
        );
        assert!(
            entry["count"].as_u64().unwrap_or(0) >= 1,
            "AC-154-007: entry.count must be >= 1; entry: {entry}"
        );
        assert_eq!(
            entry["state"].as_str(),
            Some("known-unsupported"),
            "AC-154-007: entry.state must be \"known-unsupported\" for (UDP, 47808); \
             entry: {entry}"
        );
        assert_eq!(
            entry["name"].as_str(),
            Some("BACnet/IP"),
            "AC-154-007: entry.name must be \"BACnet/IP\" for (UDP, 47808); entry: {entry}"
        );
        assert!(
            entry.get("collision_note").is_none(),
            "AC-154-007: (UDP, 47808) must NOT have a collision_note field; entry: {entry}"
        );
    }

    /// BC-2.12.024 PC-5 / AC-154-007:
    /// JSON entry for (TCP, 102) has state == "known-unsupported", a collision_note
    /// string naming the four ISO-on-TCP protocols, and NO name field.
    ///
    /// This test is NON-VACUOUS: it fails if collision_note is missing, the four
    /// protocol names are absent, or a spurious name field is present.
    ///
    /// Uses GAP_TCP102_FIXTURE with --http (analyzer-present guard; BC-2.05.010).
    #[test]
    fn test_BC_2_12_024_json_entry_port102_collision_note() {
        let output = bin()
            .args([
                "analyze",
                GAP_TCP102_FIXTURE,
                "--coverage-gaps",
                "--json",
                "--http",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.024 PC-5 / AC-154-007: --json --coverage-gaps must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        let entries = json["coverage_gaps"]["entries"]
            .as_array()
            .expect("coverage_gaps.entries must be a JSON array");
        let entry = entries
            .iter()
            .find(|e| e["port"].as_u64() == Some(102))
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.12.024 / AC-154-007: entries[] must contain an element with port==102; \
                     entries: {entries:?}"
                )
            });
        assert_eq!(
            entry["state"].as_str(),
            Some("known-unsupported"),
            "AC-154-007: (TCP, 102) state must be \"known-unsupported\"; entry: {entry}"
        );
        // collision_note must be present and name all four protocols (AC-154-007).
        let note = entry["collision_note"].as_str().unwrap_or_else(|| {
            panic!("AC-154-007: (TCP, 102) entry must have a collision_note string; entry: {entry}")
        });
        assert!(
            note.contains("S7comm"),
            "AC-154-007: collision_note must name 'S7comm'; note: {note}"
        );
        assert!(
            note.contains("IEC 61850 MMS"),
            "AC-154-007: collision_note must name 'IEC 61850 MMS'; note: {note}"
        );
        assert!(
            note.contains("ICCP"),
            "AC-154-007: collision_note must name 'ICCP'; note: {note}"
        );
        // name field must be absent for TCP/102 (AC-154-007: collision ports omit name).
        assert!(
            entry.get("name").is_none(),
            "AC-154-007: (TCP, 102) must NOT have a name field (collision port); \
             entry: {entry}"
        );
    }

    /// BC-2.12.024 PC-5 / AC-154-007:
    /// JSON entry for (TCP, 9600) has state == "unknown" and NO name field.
    ///
    /// This test is NON-VACUOUS: it fails if state is anything other than "unknown"
    /// (e.g. if TCP/9600 is inadvertently added to the catalog), or if a spurious
    /// name field appears.
    ///
    /// Uses GAP_TCP9600_FIXTURE with --http (analyzer-present guard; BC-2.05.010).
    #[test]
    fn test_BC_2_12_024_json_entry_unknown_state() {
        let output = bin()
            .args([
                "analyze",
                GAP_TCP9600_FIXTURE,
                "--coverage-gaps",
                "--json",
                "--http",
            ])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        let json: serde_json::Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!(
                "BC-2.12.024 PC-5 / AC-154-007: --json --coverage-gaps must produce valid JSON; \
                 parse error: {e}\nstdout:\n{stdout}"
            )
        });
        let entries = json["coverage_gaps"]["entries"]
            .as_array()
            .expect("coverage_gaps.entries must be a JSON array");
        let entry = entries
            .iter()
            .find(|e| e["port"].as_u64() == Some(9600))
            .unwrap_or_else(|| {
                panic!(
                    "BC-2.12.024 / AC-154-007: entries[] must contain an element with port==9600; \
                     entries: {entries:?}"
                )
            });
        assert_eq!(
            entry["state"].as_str(),
            Some("unknown"),
            "AC-154-007: (TCP, 9600) state must be \"unknown\"; entry: {entry}"
        );
        // Unknown entries have no name field (AC-154-007).
        assert!(
            entry.get("name").is_none(),
            "AC-154-007: (TCP, 9600) must NOT have a name field (unknown state); \
             entry: {entry}"
        );
    }

    // -----------------------------------------------------------------------
    // Pass-2 LOW-1: empty-entries render branch (EC-154-4 / EC-154-7)
    // -----------------------------------------------------------------------

    /// BC-2.12.024 EC-154-4 / EC-154-7:
    /// When `--coverage-gaps` finds no unclassified port gaps, the terminal output
    /// renders the empty-state message "No unclassified port gaps detected." AND
    /// the L2 caveat text is unconditionally present.
    ///
    /// Uses ANALYZE_FIXTURE (http-ooo.pcap: TCP/80 HTTP traffic — all flows are
    /// classified; the HTTP analyzer handles port 80, so zero gap entries are produced).
    /// This is the same fixture used by `test_BC_2_12_024_l2_caveat_always_present`.
    #[test]
    fn test_BC_2_12_024_empty_entries_message() {
        let output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--coverage-gaps"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let stdout = String::from_utf8(output).expect("utf-8 stdout");
        assert!(
            stdout.contains("No unclassified port gaps detected."),
            "BC-2.12.024 EC-154-4: empty-state message must be rendered when entries \
             is empty; stdout:\n{stdout}"
        );
        // L2 caveat is unconditional (EC-154-7) — it must appear even on empty entries.
        assert!(
            stdout.contains("Layer-2 protocols") || stdout.contains("TCP and UDP flows"),
            "BC-2.12.024 EC-154-7: L2 caveat text must be present even when entries \
             is empty; stdout:\n{stdout}"
        );
    }

    // -----------------------------------------------------------------------
    // Pass-2 LOW-2: purely-additive (AC-154-003 / AC-154-008 / Rule 3)
    // -----------------------------------------------------------------------

    /// AC-154-003 / AC-154-008 / Architecture Compliance Rule 3:
    /// The CoverageGapsSummary section is PURELY ADDITIVE — the Findings +
    /// AnalysisSummary output produced without `--coverage-gaps` is a byte-identical
    /// prefix of the output produced with `--coverage-gaps`.
    ///
    /// Concretely: `stdout(--http)` must be a byte-identical prefix of
    /// `stdout(--http --coverage-gaps)`. The CoverageGapsSummary block is appended
    /// after all prior output; no existing line is modified or reordered.
    ///
    /// Uses ANALYZE_FIXTURE + --http to produce stable multi-section output (Findings,
    /// TCP Reassembly analyzer, HTTP analyzer) that is deterministic across runs.
    /// Fails if any Findings or AnalysisSummary section is altered when --coverage-gaps
    /// is added, proving that the flag is strictly additive.
    #[test]
    fn test_BC_2_12_023_coverage_gaps_purely_additive() {
        let without_output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--http"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let with_output = bin()
            .args(["analyze", ANALYZE_FIXTURE, "--http", "--coverage-gaps"])
            .assert()
            .success()
            .get_output()
            .stdout
            .clone();
        let without = String::from_utf8(without_output).expect("utf-8 stdout (without)");
        let with_gaps = String::from_utf8(with_output).expect("utf-8 stdout (with)");
        // The CoverageGapsSummary block must be appended — not interleaved — so
        // the without-flag output is a byte-identical prefix of the with-flag output.
        assert!(
            with_gaps.starts_with(without.as_str()),
            "AC-154-003 / AC-154-008 / Rule 3: output with --coverage-gaps must start \
             with the byte-identical prefix from the run without --coverage-gaps;\n\
             WITHOUT length: {}, WITH length: {}\n\
             First differing byte at: {}\n\
             WITHOUT (last 200 bytes): {:?}\n\
             WITH    (first beyond):   {:?}",
            without.len(),
            with_gaps.len(),
            without
                .bytes()
                .zip(with_gaps.bytes())
                .position(|(a, b)| a != b)
                .map(|i| i.to_string())
                .unwrap_or_else(|| "none (one is prefix of other)".to_string()),
            &without[without.len().saturating_sub(200)..],
            &with_gaps[without.len().min(with_gaps.len())
                ..with_gaps
                    .len()
                    .min(without.len().saturating_add(200))
                    .min(with_gaps.len())]
        );
        // Sanity: the with-flag output must be strictly longer (it has the section).
        assert!(
            with_gaps.len() > without.len(),
            "AC-154-003: output with --coverage-gaps must be longer than without; \
             without={}, with={}",
            without.len(),
            with_gaps.len()
        );
    }
}
