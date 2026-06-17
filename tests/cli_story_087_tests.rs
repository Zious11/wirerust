//! STORY-087: Output Format Flags and Reassembly Configuration Flags — Wave 24
//!
//! Formalizes 16 tests for BC-2.12.004, BC-2.12.005, BC-2.12.007
//! (AC-001..AC-012 + EC-001..EC-003, EC-005).
//!
//! Behavioral contracts covered:
//!   BC-2.12.004  --output-format json Parses to Some(OutputFormat::Json)
//!   BC-2.12.005  Reassembly CLI Flags: --reassemble/--no-reassemble, depth,
//!                memcap, and five anomaly-threshold flags
//!   BC-2.12.007  --reassemble and --no-reassemble are Mutually Exclusive
//!
//! implementation_strategy: brownfield-formalization
//! tdd_mode: strict
//! All 16 tests pass; STORY-087 implementation is complete.
//!
//! Placement: dedicated file per DF-TEST-NAMESPACE-001 to avoid name collisions
//! with tests in other test files. All STORY-087 tests are wrapped in
//! `mod story_087`.
//!
//! DF-AC-TEST-NAME-SYNC-001: test function names EXACTLY match the AC `Test:`
//! citations in STORY-087.md. Upper-case BC identifiers in function names are
//! suppressed via #![allow(non_snake_case)].

#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-087 tests are grouped inside a dedicated
// `mod story_087` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_087 {
    use clap::Parser;
    use clap::error::ErrorKind;
    use wirerust::cli::{Cli, OutputFormat};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Call `Cli::try_parse_from` with the given argv slice and unwrap the
    /// result, panicking with a helpful message on error.
    #[allow(dead_code)]
    fn parse_ok(args: &[&str]) -> Cli {
        Cli::try_parse_from(args)
            .unwrap_or_else(|e| panic!("Expected successful parse for {args:?}, got error: {e}"))
    }

    /// Call `Cli::try_parse_from` and assert the parse fails, returning the
    /// clap error so the caller can assert the ErrorKind.
    #[allow(dead_code)]
    fn parse_err(args: &[&str]) -> clap::Error {
        Cli::try_parse_from(args).unwrap_err()
    }

    // -----------------------------------------------------------------------
    // AC-001 (BC-2.12.004 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.12.004 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "--output-format", "json", "summary", "x.pcap"])` yields
    /// `output_format = Some(OutputFormat::Json)`.
    ///
    /// Discriminating assertions:
    ///   Positive: output_format == Some(OutputFormat::Json).
    ///   Negative: output_format is NOT None and NOT Some(Csv).
    #[test]
    fn test_output_format_json_flag() {
        let cli = parse_ok(&["wirerust", "--output-format", "json", "summary", "x.pcap"]);
        assert_eq!(
            cli.output_format,
            Some(OutputFormat::Json),
            "--output-format json must yield Some(OutputFormat::Json)"
        );
        assert_ne!(cli.output_format, None, "output_format must not be None");
        assert_ne!(
            cli.output_format,
            Some(OutputFormat::Csv),
            "output_format must not be Some(Csv)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 (BC-2.12.004 postcondition 2)
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.12.004 postcondition 2): `--output-format csv` yields
    /// `output_format = Some(OutputFormat::Csv)`.
    ///
    /// Discriminating assertions:
    ///   Positive: output_format == Some(OutputFormat::Csv).
    ///   Negative: output_format is NOT None and NOT Some(Json).
    #[test]
    fn test_output_format_csv_flag() {
        let cli = parse_ok(&["wirerust", "--output-format", "csv", "summary", "x.pcap"]);
        assert_eq!(
            cli.output_format,
            Some(OutputFormat::Csv),
            "--output-format csv must yield Some(OutputFormat::Csv)"
        );
        assert_ne!(cli.output_format, None, "output_format must not be None");
        assert_ne!(
            cli.output_format,
            Some(OutputFormat::Json),
            "output_format must not be Some(Json)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 (BC-2.12.004 postcondition 3)
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.12.004 postcondition 3): When `--output-format` is absent,
    /// `output_format = None`.
    ///
    /// Discriminating assertions:
    ///   Positive: output_format == None.
    ///   Negative: output_format is NOT Some(_).
    #[test]
    fn test_output_format_absent_is_none() {
        let cli = parse_ok(&["wirerust", "summary", "x.pcap"]);
        assert_eq!(
            cli.output_format, None,
            "output_format must be None when --output-format is absent"
        );
        assert!(
            cli.output_format.is_none(),
            "output_format must not be Some(_)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 (BC-2.12.004 postcondition 4)
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.12.004 postcondition 4): `--output-format xml` causes a
    /// clap parse error (unrecognized variant).
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is InvalidValue (unrecognized enum variant).
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_output_format_invalid_value_rejected() {
        let err = parse_err(&["wirerust", "--output-format", "xml", "summary", "x.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::InvalidValue,
            "--output-format xml must produce InvalidValue error, got: {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-005 (BC-2.12.005 postcondition 3)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.12.005 postcondition 3): When `--reassembly-depth` is
    /// absent, `cli.reassembly_depth = 10` (default).
    ///
    /// Discriminating assertions:
    ///   Positive: reassembly_depth == 10.
    ///   Negative: reassembly_depth != 0 and != 1024 (not confused with memcap).
    #[test]
    fn test_reassembly_depth_default_is_10() {
        let cli = parse_ok(&["wirerust", "summary", "x.pcap"]);
        assert_eq!(
            cli.reassembly_depth, 10,
            "reassembly_depth default must be 10"
        );
        assert_ne!(
            cli.reassembly_depth, 0,
            "reassembly_depth must not default to 0"
        );
        assert_ne!(
            cli.reassembly_depth, 1024,
            "reassembly_depth must not be confused with memcap"
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 (BC-2.12.005 postcondition 4)
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.12.005 postcondition 4): When `--reassembly-memcap` is
    /// absent, `cli.reassembly_memcap = 1024` (default).
    ///
    /// Discriminating assertions:
    ///   Positive: reassembly_memcap == 1024.
    ///   Negative: reassembly_memcap != 10 (not confused with depth).
    #[test]
    fn test_reassembly_memcap_default_is_1024() {
        let cli = parse_ok(&["wirerust", "summary", "x.pcap"]);
        assert_eq!(
            cli.reassembly_memcap, 1024,
            "reassembly_memcap default must be 1024"
        );
        assert_ne!(
            cli.reassembly_memcap, 10,
            "reassembly_memcap must not be confused with depth"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 (BC-2.12.005 postcondition 5)
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.12.005 postcondition 5): Threshold override flags are
    /// `None` when absent and `Some(value)` when provided.
    ///
    /// Discriminating assertions:
    ///   Positive: absent flags → all five threshold fields are None.
    ///   Positive: --overlap-threshold 42 → overlap_threshold == Some(42).
    ///   Negative: absent flags do NOT default to Some(0).
    #[test]
    fn test_reassembly_threshold_flags_default_none() {
        // All absent → all None
        let cli = parse_ok(&["wirerust", "summary", "x.pcap"]);
        assert_eq!(
            cli.overlap_threshold, None,
            "overlap_threshold must be None when absent"
        );
        assert_eq!(
            cli.small_segment_threshold, None,
            "small_segment_threshold must be None when absent"
        );
        assert_eq!(
            cli.small_segment_max_bytes, None,
            "small_segment_max_bytes must be None when absent"
        );
        assert_eq!(
            cli.small_segment_ignore_ports, None,
            "small_segment_ignore_ports must be None when absent"
        );
        assert_eq!(
            cli.out_of_window_threshold, None,
            "out_of_window_threshold must be None when absent"
        );

        // --overlap-threshold 42 → Some(42)
        let cli2 = parse_ok(&["wirerust", "--overlap-threshold", "42", "summary", "x.pcap"]);
        assert_eq!(
            cli2.overlap_threshold,
            Some(42),
            "--overlap-threshold 42 must yield Some(42)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 (BC-2.12.005 postcondition 6)
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.12.005 postcondition 6): `--overlap-threshold 256` is
    /// rejected by clap (out of 0-255 range).
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is ValueValidation (range check).
    ///   Negative: parse does NOT return Ok with value 256.
    #[test]
    fn test_overlap_threshold_out_of_range_rejected() {
        let err = parse_err(&[
            "wirerust",
            "--overlap-threshold",
            "256",
            "summary",
            "x.pcap",
        ]);
        assert_eq!(
            err.kind(),
            ErrorKind::ValueValidation,
            "--overlap-threshold 256 must produce ValueValidation error, got: {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 (BC-2.12.005 invariant 3)
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.12.005 invariant 3): `--small-segment-ignore-ports 23,513`
    /// produces `small_segment_ignore_ports = Some([23, 513])` (comma-delimited
    /// Vec<u16>).
    ///
    /// Discriminating assertions:
    ///   Positive: small_segment_ignore_ports == Some(vec![23, 513]).
    ///   Positive: length is 2; order is preserved (23 before 513).
    ///   Negative: field is NOT None and NOT Some(vec![]).
    #[test]
    fn test_small_segment_ignore_ports_comma_delimited() {
        let cli = parse_ok(&[
            "wirerust",
            "--small-segment-ignore-ports",
            "23,513",
            "summary",
            "x.pcap",
        ]);
        assert_eq!(
            cli.small_segment_ignore_ports,
            Some(vec![23u16, 513u16]),
            "--small-segment-ignore-ports 23,513 must yield Some([23, 513])"
        );
        let ports = cli.small_segment_ignore_ports.as_ref().unwrap();
        assert_eq!(ports.len(), 2, "must parse exactly 2 ports");
        assert_eq!(ports[0], 23, "first port must be 23");
        assert_eq!(ports[1], 513, "second port must be 513");
    }

    // -----------------------------------------------------------------------
    // AC-010 (BC-2.12.007 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.12.007 postcondition 1): `Cli::try_parse_from` with both
    /// `--reassemble` AND `--no-reassemble` returns `Err` with
    /// `ArgumentConflict` error kind.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is ArgumentConflict (not ValueValidation).
    ///   Negative: parse does NOT succeed.
    #[test]
    fn test_reassemble_and_no_reassemble_conflict() {
        let err = parse_err(&[
            "wirerust",
            "--reassemble",
            "--no-reassemble",
            "summary",
            "x.pcap",
        ]);
        assert_eq!(
            err.kind(),
            ErrorKind::ArgumentConflict,
            "--reassemble --no-reassemble must produce ArgumentConflict, got: {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-011 (BC-2.12.007 invariant 1)
    // -----------------------------------------------------------------------

    /// AC-011 (BC-2.12.007 invariant 1): The conflict is symmetric:
    /// `--no-reassemble --reassemble` (reversed order) also returns `Err`.
    ///
    /// Discriminating assertions:
    ///   Positive: reversed order also returns Err.
    ///   Positive: error kind is ArgumentConflict.
    ///   Negative: order does NOT affect whether conflict fires.
    #[test]
    fn test_reassemble_conflict_is_symmetric() {
        let err = parse_err(&[
            "wirerust",
            "--no-reassemble",
            "--reassemble",
            "summary",
            "x.pcap",
        ]);
        assert_eq!(
            err.kind(),
            ErrorKind::ArgumentConflict,
            "reversed --no-reassemble --reassemble must also produce ArgumentConflict, got: {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-012 (BC-2.12.007 edge case EC-003)
    // -----------------------------------------------------------------------

    /// AC-012 (BC-2.12.007 EC-003): `--reassemble` alone parses successfully;
    /// `cli.reassemble = true`.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Ok.
    ///   Positive: reassemble == true.
    ///   Positive: no_reassemble == false.
    ///   Negative: parse does NOT fail with a conflict error.
    #[test]
    fn test_reassemble_alone_parses_ok() {
        let cli = parse_ok(&["wirerust", "--reassemble", "summary", "x.pcap"]);
        assert!(
            cli.reassemble,
            "--reassemble alone must set reassemble = true"
        );
        assert!(
            !cli.no_reassemble,
            "--reassemble alone must leave no_reassemble = false"
        );
    }

    // -----------------------------------------------------------------------
    // EC-001 — --reassembly-depth 0 rejected (FIX-P5-002)
    // -----------------------------------------------------------------------

    // STORY-087 EC-001 (revised FIX-P5-002, ADV-IMPL-P04-MED-001):
    // --reassembly-depth 0 rejected with usage error.
    //
    // Before the fix, clap accepted 0 (no range validator). After the fix,
    // `value_parser = clap::value_parser!(usize).range(1..)` rejects 0 with
    // a ValueValidation error (clap exit code 2) instead of letting the
    // process reach the assert!(max_depth > 0) in reassembly/mod.rs which
    // panics with exit 101.
    //
    #[test]
    fn test_EC_001_reassembly_depth_zero_rejected() {
        let err = parse_err(&["wirerust", "--reassembly-depth", "0", "summary", "x.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::ValueValidation,
            "--reassembly-depth 0 must produce ValueValidation error, got: {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-001b — --reassembly-memcap 0 rejected (FIX-P5-002)
    // -----------------------------------------------------------------------

    // STORY-087 EC-001 (memcap twin, FIX-P5-002, ADV-IMPL-P04-MED-001):
    // --reassembly-memcap 0 rejected with usage error.
    //
    // Mirrors test_EC_001_reassembly_depth_zero_rejected for the sibling flag.
    // After the fix, `value_parser = clap::value_parser!(usize).range(1..)`
    // rejects 0 at parse time.
    //
    #[test]
    fn test_EC_001_reassembly_memcap_zero_rejected() {
        let err = parse_err(&["wirerust", "--reassembly-memcap", "0", "summary", "x.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::ValueValidation,
            "--reassembly-memcap 0 must produce ValueValidation error, got: {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-002 — --small-segment-max-bytes 0 → Some(0)
    // -----------------------------------------------------------------------

    /// EC-002 (STORY-087 EC-002): `--small-segment-max-bytes 0` produces
    /// `small_segment_max_bytes = Some(0)` (disables detection).
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Ok.
    ///   Positive: small_segment_max_bytes == Some(0).
    ///   Negative: field is NOT None (flag was provided).
    #[test]
    fn test_EC_002_small_segment_max_bytes_zero() {
        let cli = parse_ok(&[
            "wirerust",
            "--small-segment-max-bytes",
            "0",
            "summary",
            "x.pcap",
        ]);
        assert_eq!(
            cli.small_segment_max_bytes,
            Some(0u16),
            "--small-segment-max-bytes 0 must yield Some(0)"
        );
        assert!(
            cli.small_segment_max_bytes.is_some(),
            "small_segment_max_bytes must be Some(_), not None"
        );
    }

    // -----------------------------------------------------------------------
    // EC-003 — --overlap-threshold 255 (max) accepted
    // -----------------------------------------------------------------------

    /// EC-003 (STORY-087 EC-003): `--overlap-threshold 255` (the maximum in
    /// range 0-255) is accepted; `overlap_threshold = Some(255)`.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Ok.
    ///   Positive: overlap_threshold == Some(255).
    ///   Negative: parse does NOT fail (255 is within the accepted range).
    #[test]
    fn test_EC_003_overlap_threshold_max_accepted() {
        let cli = parse_ok(&[
            "wirerust",
            "--overlap-threshold",
            "255",
            "summary",
            "x.pcap",
        ]);
        assert_eq!(
            cli.overlap_threshold,
            Some(255u32),
            "--overlap-threshold 255 must be accepted and yield Some(255)"
        );
    }

    // -----------------------------------------------------------------------
    // EC-005 — No reassembly flags → all defaults
    // -----------------------------------------------------------------------

    /// EC-005 (STORY-087 EC-005): When no reassembly flags are provided,
    /// all fields hold their defaults: `reassemble = false`,
    /// `no_reassemble = false`, `reassembly_depth = 10`,
    /// `reassembly_memcap = 1024`.
    ///
    /// Discriminating assertions:
    ///   Positive: reassemble == false.
    ///   Positive: no_reassemble == false.
    ///   Positive: reassembly_depth == 10.
    ///   Positive: reassembly_memcap == 1024.
    ///   Negative: no field is Some(_) when the flag was absent.
    #[test]
    fn test_EC_005_no_reassembly_flags_all_defaults() {
        let cli = parse_ok(&["wirerust", "summary", "x.pcap"]);
        assert!(!cli.reassemble, "reassemble must default to false");
        assert!(!cli.no_reassemble, "no_reassemble must default to false");
        assert_eq!(
            cli.reassembly_depth, 10,
            "reassembly_depth must default to 10"
        );
        assert_eq!(
            cli.reassembly_memcap, 1024,
            "reassembly_memcap must default to 1024"
        );
        assert!(
            cli.overlap_threshold.is_none(),
            "overlap_threshold must be None"
        );
        assert!(
            cli.small_segment_threshold.is_none(),
            "small_segment_threshold must be None"
        );
        assert!(
            cli.small_segment_max_bytes.is_none(),
            "small_segment_max_bytes must be None"
        );
        assert!(
            cli.small_segment_ignore_ports.is_none(),
            "small_segment_ignore_ports must be None"
        );
        assert!(
            cli.out_of_window_threshold.is_none(),
            "out_of_window_threshold must be None"
        );
    }

    // -----------------------------------------------------------------------
    // Integration tests (assert_cmd): 0 causes clap exit-2, NOT panic exit-101
    // -----------------------------------------------------------------------

    // FIX-P5-002, ADV-IMPL-P04-MED-001:
    // `--reassembly-depth 0` on the `analyze` subcommand must be rejected at
    // parse time (clap exit code 2, stderr contains an invalid-value message)
    // and must NOT panic (exit 101) or succeed (exit 0).
    //
    // The fixture http-ooo.pcap contains TCP HTTP traffic; `--http` causes
    // TCP reassembly to be exercised. Before the fix, depth=0 reaches
    // `assert!(max_depth > 0)` in reassembly/mod.rs and panics with exit 101.
    //
    #[test]
    fn test_analyze_reassembly_depth_zero_exits_usage_error() {
        use assert_cmd::Command;
        use predicates::prelude::*;

        const FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

        let assert = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["--reassembly-depth", "0", "analyze", FIXTURE, "--http"])
            .assert();

        // Must exit with clap's usage-error code (2), not success (0) or
        // panic (101).
        assert
            .failure()
            .code(2)
            .stderr(
                predicate::str::contains("invalid value")
                    .or(predicate::str::contains("0 is not in")),
            )
            .stderr(predicate::str::contains("panicked").not());
    }

    // FIX-P5-002, ADV-IMPL-P04-MED-001:
    // `--reassembly-memcap 0` mirror of the depth test above.
    //
    #[test]
    fn test_analyze_reassembly_memcap_zero_exits_usage_error() {
        use assert_cmd::Command;
        use predicates::prelude::*;

        const FIXTURE: &str = "tests/fixtures/http-ooo.pcap";

        let assert = Command::cargo_bin("wirerust")
            .expect("binary built")
            .args(["--reassembly-memcap", "0", "analyze", FIXTURE, "--http"])
            .assert();

        // Must exit with clap's usage-error code (2), not success (0) or
        // panic (101).
        assert
            .failure()
            .code(2)
            .stderr(
                predicate::str::contains("invalid value")
                    .or(predicate::str::contains("0 is not in")),
            )
            .stderr(predicate::str::contains("panicked").not());
    }

    // -----------------------------------------------------------------------
    // Unused import guard: ensure OutputFormat and ErrorKind are referenced
    // so the file compiles even in stub form.
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    fn _type_check_imports() {
        let _: ErrorKind = ErrorKind::ArgumentConflict;
        let _: Option<OutputFormat> = None;
    }
}
