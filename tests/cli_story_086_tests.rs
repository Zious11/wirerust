//! STORY-086: CLI Subcommand Parsing formalization tests — Wave 23
//!
//! Formalizes 15 tests for BC-2.12.001, BC-2.12.002, BC-2.12.003, BC-2.12.006
//! (AC-001..AC-010 + EC-001..EC-005).
//!
//! Behavioral contracts covered:
//!   BC-2.12.001  analyze Subcommand Parses Positional Targets and All Flags
//!   BC-2.12.002  summary Subcommand Parses Targets and --hosts Flag
//!   BC-2.12.003  Global Flag --no-color Parsed and Stored
//!   BC-2.12.006  Multiple Positional Targets Accepted in analyze
//!
//! implementation_strategy: brownfield-formalization
//! tdd_mode: strict
//! RED GATE stub phase: all 15 stubs confirmed FAIL before implementation.
//!
//! Placement: dedicated file per DF-TEST-NAMESPACE-001 to avoid name collisions
//! with the 14 informal tests in tests/cli_tests.rs. All STORY-086 tests are
//! wrapped in `mod story_086`.
//!
//! PG-W17-001 / DF-AC-TEST-NAME-SYNC-001: test function names EXACTLY match the
//! AC `Test:` citations in STORY-086.md. Upper-case BC identifiers in function
//! names are suppressed via #![allow(non_snake_case)].

#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-086 tests are grouped inside a dedicated
// `mod story_086` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_086 {
    use std::path::PathBuf;

    use clap::Parser;
    use clap::error::ErrorKind;
    #[allow(unused_imports)]
    use wirerust::cli::{Cli, Commands};

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
    // AC-001 (BC-2.12.001 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.12.001 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "cap.pcap"])` returns `Ok` with `cli.command` matching
    /// `Commands::Analyze { targets: [cap.pcap], dns: false, http: false,
    /// tls: false, mitre: false, all: false, .. }`.
    ///
    /// Discriminating assertions:
    ///   Positive: variant is Analyze; targets=[cap.pcap]; all bool fields false.
    ///   Negative: command is NOT Summary.
    #[test]
    fn test_analyze_subcommand_basic_parse() {
        let cli = parse_ok(&["wirerust", "analyze", "cap.pcap"]);
        match cli.command {
            Commands::Analyze {
                targets,
                dns,
                http,
                tls,
                mitre,
                all,
                ..
            } => {
                assert_eq!(targets, vec![PathBuf::from("cap.pcap")]);
                assert!(!dns, "dns should be false");
                assert!(!http, "http should be false");
                assert!(!tls, "tls should be false");
                assert!(!mitre, "mitre should be false");
                assert!(!all, "all should be false");
            }
            Commands::Summary { .. } => panic!("Expected Analyze, got Summary"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-002 (BC-2.12.001 postcondition 3)
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.12.001 postcondition 3): When --dns, --http, --tls, --mitre,
    /// or --all are present, their corresponding struct fields are true; absent
    /// flags remain false.
    ///
    /// Discriminating assertions:
    ///   Positive: --dns → dns=true, http/tls/mitre/all=false.
    ///   Positive: --http --tls → http=true, tls=true, dns=false.
    ///   Positive: --all → all=true, dns/http/tls=false.
    ///   Negative: flags not present → all corresponding fields false.
    #[test]
    fn test_analyze_individual_protocol_flags() {
        // --dns only
        let cli = parse_ok(&["wirerust", "analyze", "--dns", "cap.pcap"]);
        match cli.command {
            Commands::Analyze {
                dns,
                http,
                tls,
                mitre,
                all,
                ..
            } => {
                assert!(dns, "dns should be true");
                assert!(!http, "http should be false");
                assert!(!tls, "tls should be false");
                assert!(!mitre, "mitre should be false");
                assert!(!all, "all should be false");
            }
            _ => panic!("Expected Analyze"),
        }

        // --http --tls
        let cli = parse_ok(&["wirerust", "analyze", "--http", "--tls", "cap.pcap"]);
        match cli.command {
            Commands::Analyze {
                dns,
                http,
                tls,
                all,
                ..
            } => {
                assert!(!dns, "dns should be false");
                assert!(http, "http should be true");
                assert!(tls, "tls should be true");
                assert!(!all, "all should be false");
            }
            _ => panic!("Expected Analyze"),
        }

        // --all only
        let cli = parse_ok(&["wirerust", "analyze", "--all", "cap.pcap"]);
        match cli.command {
            Commands::Analyze {
                dns,
                http,
                tls,
                all,
                ..
            } => {
                assert!(all, "all should be true");
                assert!(!dns, "dns should be false");
                assert!(!http, "http should be false");
                assert!(!tls, "tls should be false");
            }
            _ => panic!("Expected Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-003 (BC-2.12.001 invariant 1)
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.12.001 invariant 1): `Cli::try_parse_from(["wirerust",
    /// "analyze"])` (no targets) returns `Err`; clap surfaces a
    /// required-argument-missing error (ErrorKind::MissingRequiredArgument).
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is MissingRequiredArgument (not UnknownArgument).
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_analyze_requires_at_least_one_target() {
        let err = parse_err(&["wirerust", "analyze"]);
        assert_eq!(
            err.kind(),
            ErrorKind::MissingRequiredArgument,
            "Expected MissingRequiredArgument, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 (BC-2.12.001 invariant 3)
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.12.001 invariant 3): --mitre is a separate flag that sets
    /// mitre=true but does NOT enable any analyzer; dns, http, tls remain false
    /// when only --mitre is passed.
    ///
    /// Discriminating assertions:
    ///   Positive: mitre=true.
    ///   Positive: dns=false, http=false, tls=false, all=false.
    ///   Negative: mitre alone does NOT imply all=true.
    #[test]
    fn test_mitre_flag_does_not_imply_analyzers() {
        let cli = parse_ok(&["wirerust", "analyze", "--mitre", "cap.pcap"]);
        match cli.command {
            Commands::Analyze {
                dns,
                http,
                tls,
                mitre,
                all,
                ..
            } => {
                assert!(mitre, "mitre should be true");
                assert!(!dns, "dns should be false");
                assert!(!http, "http should be false");
                assert!(!tls, "tls should be false");
                assert!(!all, "all should be false — mitre does not imply all");
            }
            _ => panic!("Expected Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-005 (BC-2.12.002 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.12.002 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "summary", "cap.pcap"])` returns `Ok` with
    /// `Commands::Summary { targets: [cap.pcap], hosts: false }`.
    ///
    /// Discriminating assertions:
    ///   Positive: variant is Summary; targets=[cap.pcap]; hosts=false.
    ///   Negative: command is NOT Analyze.
    #[test]
    fn test_summary_subcommand_basic_parse() {
        let cli = parse_ok(&["wirerust", "summary", "cap.pcap"]);
        match cli.command {
            Commands::Summary { targets, hosts } => {
                assert_eq!(targets, vec![PathBuf::from("cap.pcap")]);
                assert!(!hosts, "hosts should be false");
            }
            Commands::Analyze { .. } => panic!("Expected Summary, got Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-006 (BC-2.12.002 postcondition 3)
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.12.002 postcondition 3): --hosts flag sets hosts=true;
    /// absent flag leaves hosts=false.
    ///
    /// Discriminating assertions:
    ///   Positive: --hosts → hosts=true.
    ///   Positive: absent --hosts → hosts=false (BC-2.12.002 EC-001).
    ///   Negative: hosts is plain bool, never Option<bool>.
    #[test]
    fn test_summary_hosts_flag() {
        // --hosts present → true
        let cli = parse_ok(&["wirerust", "summary", "--hosts", "cap.pcap"]);
        match cli.command {
            Commands::Summary { hosts, .. } => {
                // Direct bool comparison — would fail to compile if type were Option<bool>
                assert!(hosts, "hosts should be true when --hosts is provided");
                let _: bool = hosts; // type assertion: must be plain bool
            }
            _ => panic!("Expected Summary"),
        }

        // --hosts absent → false
        let cli = parse_ok(&["wirerust", "summary", "cap.pcap"]);
        match cli.command {
            Commands::Summary { hosts, .. } => {
                let _: bool = hosts; // type assertion: must be plain bool
                assert!(!hosts, "hosts should be false when --hosts is absent");
            }
            _ => panic!("Expected Summary"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-007 (BC-2.12.002 invariant 4)
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.12.002 invariant 4): --services (removed flag) is rejected
    /// by clap with UnknownArgument.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument (LESSON-P1.04 / BC-2.12.002 EC-004).
    ///   Negative: parse does NOT succeed.
    #[test]
    fn test_summary_services_flag_removed() {
        let err = parse_err(&["wirerust", "summary", "--services", "cap.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --services, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 (BC-2.12.003 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.12.003 postcondition 1): --no-color sets cli.no_color=true
    /// whether placed before or after the subcommand (global flag semantics).
    ///
    /// Discriminating assertions:
    ///   Positive: ["wirerust", "--no-color", "analyze", "cap.pcap"] → no_color=true.
    ///   Positive: ["wirerust", "analyze", "--no-color", "cap.pcap"] → no_color=true.
    ///   Positive: ["wirerust", "analyze", "cap.pcap", "--no-color"] → no_color=true.
    ///   Negative: absent --no-color → no_color=false.
    #[test]
    fn test_no_color_flag_global_placement() {
        // Before subcommand (BC-2.12.003 EC-001)
        let cli = parse_ok(&["wirerust", "--no-color", "analyze", "cap.pcap"]);
        assert!(
            cli.no_color,
            "--no-color before subcommand should set no_color=true"
        );

        // After subcommand name, before positional (BC-2.12.003 EC-001 / global = true)
        let cli = parse_ok(&["wirerust", "analyze", "--no-color", "cap.pcap"]);
        assert!(
            cli.no_color,
            "--no-color after subcommand name should set no_color=true"
        );

        // After positional (BC-2.12.003 EC-002 — global flag semantics)
        let cli = parse_ok(&["wirerust", "analyze", "cap.pcap", "--no-color"]);
        assert!(
            cli.no_color,
            "--no-color after positional should set no_color=true"
        );

        // Absent → false
        let cli = parse_ok(&["wirerust", "analyze", "cap.pcap"]);
        assert!(
            !cli.no_color,
            "absent --no-color should leave no_color=false"
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 (BC-2.12.003 invariant 2)
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.12.003 invariant 2): cli.no_color is a plain bool (never
    /// Option<bool>); when absent it is false.
    ///
    /// Discriminating assertions:
    ///   Positive: no_color field is false (not Some(false) — type assertion via
    ///   direct bool comparison, which would fail to compile if the type were
    ///   Option<bool>).
    ///   Positive: no_color==false when --no-color is absent (BC-2.12.003 EC-003).
    #[test]
    fn test_no_color_flag_default_false() {
        let cli = parse_ok(&["wirerust", "analyze", "cap.pcap"]);
        // Type assertion: this line would fail to compile if no_color were Option<bool>
        let no_color: bool = cli.no_color;
        assert!(
            !no_color,
            "no_color must be false when --no-color is absent"
        );
    }

    // -----------------------------------------------------------------------
    // AC-010 (BC-2.12.006 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.12.006 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "a.pcap", "b.pcap", "c.pcap"])` produces
    /// `targets = [a.pcap, b.pcap, c.pcap]` in command-line order; duplicates
    /// are preserved.
    ///
    /// Discriminating assertions:
    ///   Positive: targets length == 3.
    ///   Positive: targets[0]=="a.pcap", targets[1]=="b.pcap", targets[2]=="c.pcap".
    ///   Positive: order preserved (BC-2.12.006 inv3).
    ///   Negative: no deduplication at parse time.
    #[test]
    fn test_multiple_targets_preserve_order_and_duplicates() {
        let cli = parse_ok(&["wirerust", "analyze", "a.pcap", "b.pcap", "c.pcap"]);
        match cli.command {
            Commands::Analyze { targets, .. } => {
                assert_eq!(targets.len(), 3, "should have exactly 3 targets");
                assert_eq!(targets[0], PathBuf::from("a.pcap"));
                assert_eq!(targets[1], PathBuf::from("b.pcap"));
                assert_eq!(targets[2], PathBuf::from("c.pcap"));
            }
            _ => panic!("Expected Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // EC-001 — --all with individual flags
    // -----------------------------------------------------------------------

    /// EC-001: --all flag with individual protocol flags provided simultaneously.
    /// all=true; individual flags also true if provided.
    ///
    /// BC-2.12.001 EC-003: --all flag → all=true; dns/http/tls also set if given.
    ///
    /// Discriminating assertions:
    ///   Positive: all=true.
    ///   Positive: dns=true, http=true (individually provided alongside --all).
    ///   Positive: tls=false (not individually provided).
    #[test]
    fn test_EC_001_all_flag_with_individual_protocol_flags() {
        let cli = parse_ok(&[
            "wirerust", "analyze", "--all", "--dns", "--http", "cap.pcap",
        ]);
        match cli.command {
            Commands::Analyze {
                dns,
                http,
                tls,
                all,
                ..
            } => {
                assert!(all, "all should be true");
                assert!(dns, "dns should be true (individually provided)");
                assert!(http, "http should be true (individually provided)");
                assert!(!tls, "tls should be false (not individually provided)");
            }
            _ => panic!("Expected Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // EC-002 — --mitre alone
    // -----------------------------------------------------------------------

    /// EC-002: --mitre alone; mitre=true, all=false, dns/http/tls=false.
    ///
    /// BC-2.12.001 EC-005 / invariant 3: --mitre does not imply any analyzer.
    ///
    /// Discriminating assertions:
    ///   Positive: mitre=true.
    ///   Positive: all=false, dns=false, http=false, tls=false.
    #[test]
    fn test_EC_002_mitre_alone() {
        let cli = parse_ok(&["wirerust", "analyze", "--mitre", "cap.pcap"]);
        match cli.command {
            Commands::Analyze {
                dns,
                http,
                tls,
                mitre,
                all,
                ..
            } => {
                assert!(mitre, "mitre should be true");
                assert!(!all, "all should be false");
                assert!(!dns, "dns should be false");
                assert!(!http, "http should be false");
                assert!(!tls, "tls should be false");
            }
            _ => panic!("Expected Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // EC-003 — --hosts on analyze subcommand
    // -----------------------------------------------------------------------

    /// EC-003: --hosts passed to the analyze subcommand is a clap error.
    /// --hosts is only declared on Commands::Summary.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument (--hosts not in analyze flags).
    #[test]
    fn test_EC_003_hosts_flag_rejected_on_analyze() {
        let err = parse_err(&["wirerust", "analyze", "--hosts", "cap.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --hosts on analyze, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-004 — --services on summary subcommand
    // -----------------------------------------------------------------------

    /// EC-004: --services passed to summary returns clap UnknownArgument error.
    ///
    /// BC-2.12.002 invariant 4 / EC-004: removed flag triggers clap rejection.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: flag is NOT silently ignored.
    #[test]
    fn test_EC_004_services_flag_rejected_on_summary() {
        let err = parse_err(&["wirerust", "summary", "--services", "cap.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --services on summary, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-005 — Duplicate targets preserved
    // -----------------------------------------------------------------------

    /// EC-005: Duplicate targets `a.pcap a.pcap` → targets=[a.pcap, a.pcap].
    /// No deduplication at parse time.
    ///
    /// BC-2.12.006 postcondition 2 / EC-003: duplicate paths are allowed and
    /// both are stored.
    ///
    /// Discriminating assertions:
    ///   Positive: targets.len()==2.
    ///   Positive: targets[0]==targets[1]=="a.pcap" (exact PathBuf equality).
    ///   Negative: targets.len()!=1 (deduplication did NOT occur).
    #[test]
    fn test_EC_005_duplicate_targets_preserved() {
        let cli = parse_ok(&["wirerust", "analyze", "a.pcap", "a.pcap"]);
        match cli.command {
            Commands::Analyze { targets, .. } => {
                assert_eq!(targets.len(), 2, "duplicate targets must both be stored");
                assert_eq!(targets[0], PathBuf::from("a.pcap"));
                assert_eq!(targets[1], PathBuf::from("a.pcap"));
            }
            _ => panic!("Expected Analyze"),
        }
    }

    // -----------------------------------------------------------------------
    // Unused import guard: ensure PathBuf and ErrorKind are referenced
    // so the file compiles even in stub form.
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    fn _type_check_imports() {
        let _: PathBuf = PathBuf::from("x");
        let _: ErrorKind = ErrorKind::MissingRequiredArgument;
    }
}
