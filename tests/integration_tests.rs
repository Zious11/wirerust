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
    /// 0x88B8 = 35000 decimal (verified: 8*16^3 + 8*16^2 + 11*16 + 8 = 34816+2048+176+8 = wait
    /// 0x88B8: 0x8000=32768, 0x0800=2048, 0x00B0=176, 0x0008=8 → 32768+2048+176+8 = 35000 ✓).
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
}
