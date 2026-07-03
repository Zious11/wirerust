//! Integration tests for STORY-152: `wirerust protocols` CLI subcommand.
//!
//! Behavioral contracts covered:
//!   BC-2.12.022 v1.0 — `wirerust protocols` Subcommand Dispatch + `--json` Flag
//!   BC-2.18.001 v1.4 — Terminal Catalog Output (filter flags, [L2], EtherType, footnotes)
//!   BC-2.18.002 v1.1 — JSON Mode Output Schema
//!
//! RED-GATE: All tests in `mod story_152` that assert `.success()` FAIL against
//! the current binary — the `protocols` subcommand does not yet exist, so clap
//! exits non-zero with "unrecognized subcommand 'protocols'" for every invocation.
//!
//! Exceptions (PASS in Red-Gate — these are regression guards):
//!   - `test_BC_2_12_022_analyze_unaffected` — exercises existing `analyze` behavior;
//!     asserts output does NOT gain a `protocols` key (always true today).
//!   - `test_BC_2_12_022_mutually_exclusive_flags_error` — expects non-zero exit;
//!     binary already exits non-zero (wrong subcommand) so this passes for the
//!     wrong reason in Red-Gate. It becomes a true behavioral guard post-implementation.
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
    /// RED-GATE FAIL: the `protocols` subcommand does not exist → clap exits
    /// non-zero → `.assert().success()` assertion fails.
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
    /// RED-GATE NOTE: PASSES even before implementation — the binary exits non-zero
    /// for "unrecognized subcommand 'protocols'" rather than "mutually exclusive flags".
    /// This test becomes a true behavioral guard after implementation; it would fail if
    /// the `conflicts_with` annotation were removed from the `protocols` subcommand.
    #[test]
    fn test_BC_2_12_022_mutually_exclusive_flags_error() {
        bin()
            .args(["protocols", "--supported", "--unsupported"])
            .assert()
            .failure();
    }

    /// BC-2.12.022 Postcondition 3 / EC-002
    /// `wirerust protocols --supported` output has exactly 7 protocol rows
    /// (one per `supported_protocols()` entry).
    ///
    /// Row count is derived robustly by matching stdout lines against protocol
    /// names from `supported_protocols()`, not by counting header/footnote lines.
    ///
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
        let row_count = stdout
            .lines()
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE NOTE: PASSES before implementation — `analyze` already works and
    /// naturally does not emit a `"protocols"` key. This test is a REGRESSION GUARD:
    /// it fails only if the `protocols` subcommand implementation breaks `analyze`
    /// or accidentally injects a `"protocols"` key into the analyze output.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
        let row_count = stdout
            .lines()
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
        assert!(
            stdout.contains("S7comm"),
            "BC-2.18.001 PC-6: port-102 footnote must name 'S7comm'; stdout:\n{stdout}"
        );
        // S7comm-plus (or equivalent notation).
        assert!(
            stdout.contains("S7comm-plus") || stdout.contains("S7comm+"),
            "BC-2.18.001 PC-6: port-102 footnote must name 'S7comm-plus'; stdout:\n{stdout}"
        );
        // IEC 61850 MMS.
        assert!(
            stdout.contains("IEC 61850 MMS") || stdout.contains("MMS"),
            "BC-2.18.001 PC-6: port-102 footnote must name 'IEC 61850 MMS'; stdout:\n{stdout}"
        );
        // ICCP or ICCP/TASE.2.
        assert!(
            stdout.contains("ICCP") || stdout.contains("TASE.2"),
            "BC-2.18.001 PC-6: port-102 footnote must name 'ICCP' or 'ICCP/TASE.2'; \
             stdout:\n{stdout}"
        );
    }

    /// BC-2.18.001 v1.4 Postcondition 4 / EC-004
    /// The GOOSE row in `wirerust protocols --unsupported` contains `[L2]` in the
    /// transport column (IEC 61850 GOOSE is a `transport=LinkLayer` entry).
    ///
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
    /// RED-GATE FAIL: clap rejects `protocols` → `.assert().success()` fails.
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
}
