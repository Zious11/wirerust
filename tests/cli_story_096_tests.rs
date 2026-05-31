//! STORY-096: Absent Behavior Contracts — Removed Flags Rejected by clap (Wave 24)
//!
//! Formalizes 14 tests for BC-2.13.001, BC-2.13.002, BC-2.13.003, BC-2.13.004
//! (AC-001..AC-010 + EC-001..EC-004).
//!
//! Behavioral contracts covered:
//!   BC-2.13.001  --threats Flag Does Not Exist; clap Rejects It as Unknown Argument
//!   BC-2.13.002  --beacon Flag Does Not Exist; No C2 Beacon Analyzer Exists
//!   BC-2.13.003  --filter <BPF> Flag Does Not Exist; No BPF Filter Applied
//!   BC-2.13.004  --verbose Flag Does Not Exist; No Verbose Logging Mode
//!
//! implementation_strategy: brownfield-formalization
//! tdd_mode: facade — combined scaffold+impl delivery. Tests prove ABSENCE.
//! No Red Gate stub phase: the flags are already absent from src/cli.rs
//! (LESSON-P1.04). All 14 tests pass immediately against the current
//! brownfield codebase, which is the intended proof of absent behavior.
//!
//! Placement: dedicated file per DF-TEST-NAMESPACE-001 to avoid name collisions.
//! All STORY-096 tests are wrapped in `mod story_096`.
//!
//! DF-AC-TEST-NAME-SYNC-001: test function names EXACTLY match the AC `Test:`
//! citations in STORY-096.md. Upper-case BC identifiers in function names are
//! suppressed via #![allow(non_snake_case)].

#![allow(non_snake_case)]

// Per DF-TEST-NAMESPACE-001: all STORY-096 tests are grouped inside a dedicated
// `mod story_096` wrapper to prevent test-function name collisions with other
// stories' BC-prefixed names.
mod story_096 {
    use clap::Parser;
    use clap::error::ErrorKind;
    use wirerust::cli::Cli;

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
    // AC-001 (BC-2.13.001 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-001 (BC-2.13.001 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "--threats", "test.pcap"])` returns `Err` with
    /// `ErrorKind::UnknownArgument`.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument (not some other kind).
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_threats_flag_rejected_by_clap() {
        let err = parse_err(&["wirerust", "analyze", "--threats", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --threats, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-002 (BC-2.13.001 invariant 1)
    // -----------------------------------------------------------------------

    /// AC-002 (BC-2.13.001 invariant 1): No `threats`-related field declaration
    /// exists in `src/cli.rs`. The LESSON-P1.04 comment mentions `--threats` by
    /// name, so we assert against field-declaration patterns specifically:
    ///   - no `pub threats` (struct field)
    ///   - no `long = "threats"` (explicit clap long-name override)
    ///   - no `"--threats"` string literal in an `#[arg` context
    ///
    /// This assertion is mutation-resistant: adding `pub threats: bool` to the
    /// struct would cause the test to fail, even though the comment text is
    /// allowed to remain.
    ///
    /// Discriminating assertions:
    ///   Positive: no `pub threats` field in the source.
    ///   Positive: no `long = "threats"` explicit long-name override.
    ///   Negative: the LESSON-P1.04 comment mentioning "--threats" is NOT a
    ///   false positive because we check for field-declaration patterns only.
    #[test]
    fn test_threats_field_absent_from_cli() {
        let src = include_str!("../src/cli.rs");
        assert!(
            !src.contains("pub threats"),
            "src/cli.rs must not declare a `pub threats` field"
        );
        assert!(
            !src.contains("long = \"threats\""),
            "src/cli.rs must not contain an explicit long = \"threats\" override"
        );
        // A field named `threats` with any type annotation, e.g. `threats: bool`
        // or `threats: Option<String>`, would appear as `threats:` followed by a
        // type. The LESSON-P1.04 comment writes "--threats" (with a leading dash),
        // so checking for `\nthreats:` or `    threats:` (indented field) is safe.
        assert!(
            !src.contains("    threats:"),
            "src/cli.rs must not contain an indented `threats:` field declaration"
        );
    }

    // -----------------------------------------------------------------------
    // AC-003 (BC-2.13.002 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-003 (BC-2.13.002 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "--beacon", "test.pcap"])` returns `Err` with
    /// `ErrorKind::UnknownArgument`.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_beacon_flag_rejected_by_clap() {
        let err = parse_err(&["wirerust", "analyze", "--beacon", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --beacon, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-004 (BC-2.13.002 invariant 2)
    // -----------------------------------------------------------------------

    /// AC-004 (BC-2.13.002 invariant 2): No `C2BeaconAnalyzer` or equivalent
    /// beacon analyzer struct exists ANYWHERE in `src/`.
    ///
    /// BC-2.13.002 invariant 2 scopes the absence to ALL of `src/`, not just the
    /// analyzer subtree. A hand-listed `include_str!` set is therefore inadequate:
    /// a `C2BeaconAnalyzer` declared in any unscanned file (e.g. `src/summary.rs`)
    /// or in a NEW file (e.g. `src/analyzer/beacon.rs`) would evade it. Instead we
    /// recursively walk every `*.rs` file under `src/` at test runtime (resolved
    /// via `CARGO_MANIFEST_DIR`) and assert the struct/impl declaration forms are
    /// absent from every file.
    ///
    /// The LESSON-P1.04 comment in `src/cli.rs` mentions `--beacon` as text, and
    /// `src/findings.rs` contains "beaconing" in a doc comment; neither matches the
    /// struct/impl declaration forms we search for, so they are not false positives.
    ///
    /// A positive-coverage guard asserts the walk visited at least the known number
    /// of `src/` files, so an empty or mis-rooted walk cannot silently false-green.
    ///
    /// Discriminating assertions:
    ///   Positive: no `struct BeaconAnalyzer` / `struct C2BeaconAnalyzer` anywhere in src/.
    ///   Positive: no `impl BeaconAnalyzer` / `impl C2BeaconAnalyzer` anywhere in src/.
    ///   Positive (coverage guard): the walk visited >= a non-trivial number of files.
    ///   Negative: doc-comment occurrences of "beaconing"/"--beacon" are NOT matched
    ///   because we search for the struct/impl declaration forms specifically.
    #[test]
    fn test_beacon_analyzer_absent_from_src() {
        use std::path::{Path, PathBuf};

        // Recursively collect every `*.rs` file under `dir`.
        fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
            let entries = std::fs::read_dir(dir)
                .unwrap_or_else(|e| panic!("failed to read dir {}: {e}", dir.display()));
            for entry in entries {
                let path = entry.expect("dir entry").path();
                if path.is_dir() {
                    collect_rs(&path, out);
                } else if path.extension().is_some_and(|ext| ext == "rs") {
                    out.push(path);
                }
            }
        }

        let src_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut rs_files = Vec::new();
        collect_rs(&src_dir, &mut rs_files);

        // Positive-coverage guard: the walk MUST find the full src/ tree. As of this
        // story src/ has 24 `.rs` files; require a conservative lower bound so an
        // empty/mis-rooted walk (which would vacuously pass the absence checks) fails
        // loudly instead.
        assert!(
            rs_files.len() >= 20,
            "coverage guard: expected to walk the full src/ tree (>=20 .rs files), \
             found only {} under {} — an empty/mis-rooted walk must not false-green",
            rs_files.len(),
            src_dir.display()
        );

        for path in &rs_files {
            let src = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
            let rel = path.strip_prefix(&src_dir).unwrap_or(path).display();
            assert!(
                !src.contains("struct BeaconAnalyzer"),
                "src/{rel} must not declare `struct BeaconAnalyzer`"
            );
            assert!(
                !src.contains("struct C2BeaconAnalyzer"),
                "src/{rel} must not declare `struct C2BeaconAnalyzer`"
            );
            assert!(
                !src.contains("impl BeaconAnalyzer"),
                "src/{rel} must not implement `BeaconAnalyzer`"
            );
            assert!(
                !src.contains("impl C2BeaconAnalyzer"),
                "src/{rel} must not implement `C2BeaconAnalyzer`"
            );
        }
    }

    // -----------------------------------------------------------------------
    // AC-005 (BC-2.13.003 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-005 (BC-2.13.003 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "--filter", "tcp", "test.pcap"])` returns `Err` with
    /// `ErrorKind::UnknownArgument`.
    ///
    /// The canonical test vector from BC-2.13.003 uses `"tcp"` as the BPF
    /// expression (treated by clap as an extra positional after the unknown flag).
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_filter_flag_rejected_by_clap() {
        let err = parse_err(&["wirerust", "analyze", "--filter", "tcp", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --filter, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-006 (BC-2.13.003 invariant 2)
    // -----------------------------------------------------------------------

    /// AC-006 (BC-2.13.003 invariant 2): No BPF library exists in `Cargo.toml`.
    /// Specifically: no `pcap` (BPF-capable), `pcap-filter`, `bpf`, `bpf-sys`, or
    /// `libpcap` crate is declared as a dependency.
    ///
    /// Strategy: `include_str!` on `Cargo.toml` and assert that no forbidden crate
    /// is declared as a DEPENDENCY KEY. The `pcap-file` crate (present) provides
    /// only file-reading — it does NOT provide BPF expression evaluation — and must
    /// NOT be flagged.
    ///
    /// The canonical reintroduction vector (BC-2.13.003 Refactoring Notes) is the
    /// `pcap` crate (crates.io: `pcap`), whose `Capture::filter()` compiles and
    /// applies a BPF expression at the kernel level.
    ///
    /// Because a naive substring check is evadable in several directions —
    /// `contains("pcap")` false-positives on `pcap-file`; `contains("\"bpf\"")`
    /// misses an unquoted `bpf = "0.1"` key; and an enumerated set of inline/table
    /// syntaxes misses the idiomatic dotted form `pcap.version = "2.2"` — we instead
    /// parse Cargo.toml line-by-line and detect each forbidden crate by its
    /// DEPENDENCY KEY across ALL of TOML's dependency-declaration syntaxes:
    ///   - inline:        `name = …`, `name= …`         (space or no-space before `=`)
    ///   - dotted:        `name.version = …`, `name.features = …`  (`name.` prefix)
    ///   - table header:  `[dependencies.name]`, `[dependencies.name.features]`,
    ///     and the build/dev variants `[*-dependencies.name…]`
    ///
    /// This structural key match (rather than enumerated literal forms) is the
    /// mutation-resistance fix: any TOML syntax that declares a forbidden crate is
    /// caught, while `pcap-file` (key `pcap-file`, not `pcap`) is never matched
    /// because none of the `pcap`-anchored prefixes (`pcap `, `pcap=`, `pcap.`,
    /// `[dependencies.pcap]`, `[dependencies.pcap.`) is a prefix of `pcap-file`.
    ///
    /// Discriminating assertions:
    ///   Positive: no `pcap` dependency key (BPF-capable `pcap` crate) in any
    ///     inline / dotted / table-header syntax.
    ///   Positive: no `pcap-filter`, `bpf`, `bpf-sys`, or `libpcap` dependency key.
    ///   Negative: `pcap-file` (the present read-only crate) is NOT flagged — key
    ///     matching distinguishes it from `pcap`.
    ///   Negative (sanity guard): `pcap-file` IS present, proving the detector can
    ///     tell the two crates apart.
    #[test]
    fn test_bpf_filter_absent_from_src() {
        let cargo_toml = include_str!("../Cargo.toml");

        // ------------------------------------------------------------------
        // Structural dependency-KEY detector. Returns true if `crate_name` is
        // declared as a dependency in ANY TOML syntax. Anchoring on the exact
        // key followed by ` `, `=`, `.`, or a table-header boundary ensures
        // sibling crates that merely share a prefix (e.g. `pcap-file` vs `pcap`)
        // are NOT matched.
        // ------------------------------------------------------------------
        let declares_dep = |crate_name: &str| -> bool {
            cargo_toml.lines().any(|raw_line| {
                let line = raw_line.trim();
                // inline:  `name =`, `name= `
                line.starts_with(&format!("{crate_name} ="))
                    || line.starts_with(&format!("{crate_name}="))
                    // dotted:  `name.version = …`, `name.optional = …`, etc.
                    || line.starts_with(&format!("{crate_name}."))
                    // table header (and build-/dev- prefixed variants):
                    //   [dependencies.name]   [dependencies.name.features]
                    || line.contains(&format!("dependencies.{crate_name}]"))
                    || line.contains(&format!("dependencies.{crate_name}."))
            })
        };

        // Every known BPF-capable / BPF-binding crate. The fix is uniform across
        // all forbidden keys (DF-SIBLING-SWEEP-001): the same syntactic gap that
        // affected `pcap` is closed for every name in one pass.
        for forbidden in ["pcap", "pcap-filter", "bpf", "bpf-sys", "libpcap"] {
            assert!(
                !declares_dep(forbidden),
                "Cargo.toml must not declare the `{forbidden}` crate as a \
                 dependency (BPF expression evaluation — BC-2.13.003 invariant 2 / \
                 Refactoring Notes reintroduction vector)"
            );
        }

        // ------------------------------------------------------------------
        // Sanity guard: `pcap-file` (read-only) MUST still be present, and the
        // detector must NOT have flagged it above. This proves the key matcher
        // correctly distinguishes `pcap` from `pcap-file`.
        // ------------------------------------------------------------------
        assert!(
            cargo_toml.contains("pcap-file"),
            "Cargo.toml must retain the `pcap-file` read-only dependency (sanity guard)"
        );
        assert!(
            declares_dep("pcap-file"),
            "sanity guard: detector must recognize `pcap-file` as a declared dependency \
             (proving it matches real dependency keys, not just any substring)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-007 (BC-2.13.004 postcondition 1)
    // -----------------------------------------------------------------------

    /// AC-007 (BC-2.13.004 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "--verbose", "test.pcap"])` returns `Err` with
    /// `ErrorKind::UnknownArgument`.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_verbose_flag_rejected_by_clap() {
        let err = parse_err(&["wirerust", "analyze", "--verbose", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --verbose, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-008 (BC-2.13.004 postcondition 1 — short form)
    // -----------------------------------------------------------------------

    /// AC-008 (BC-2.13.004 postcondition 1): `Cli::try_parse_from(["wirerust",
    /// "analyze", "-v", "test.pcap"])` also returns `Err` with
    /// `ErrorKind::UnknownArgument` (short form `-v` is also not declared).
    ///
    /// clap 4 returns `UnknownArgument` for any short flag (`-x`) that is not
    /// declared in the Cli struct or the active subcommand. This is the same
    /// kind used for undeclared long flags.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: parse does NOT return Ok.
    #[test]
    fn test_verbose_short_flag_rejected_by_clap() {
        let err = parse_err(&["wirerust", "analyze", "-v", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for -v (short verbose), got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // AC-009 (BC-2.13.004 invariant 1)
    // -----------------------------------------------------------------------

    /// AC-009 (BC-2.13.004 invariant 1): No `--verbose` or `-v` field declaration
    /// exists in `src/cli.rs`.
    ///
    /// Strategy: assert absence of field-declaration patterns. The LESSON-P1.04
    /// comment mentions `--verbose` by name, so we assert against the field
    /// forms (`pub verbose`, `long = "verbose"`, indented `verbose:` field).
    ///
    /// Discriminating assertions:
    ///   Positive: no `pub verbose` field in the source.
    ///   Positive: no `long = "verbose"` explicit long-name override.
    ///   Positive: no `short = 'v'` short-flag override for a verbose field.
    ///   Negative: the LESSON-P1.04 comment mentioning "--verbose" is NOT
    ///   a false positive because we check for field-declaration patterns only.
    #[test]
    fn test_verbose_field_absent_from_cli() {
        let src = include_str!("../src/cli.rs");
        assert!(
            !src.contains("pub verbose"),
            "src/cli.rs must not declare a `pub verbose` field"
        );
        assert!(
            !src.contains("long = \"verbose\""),
            "src/cli.rs must not contain an explicit long = \"verbose\" override"
        );
        assert!(
            !src.contains("    verbose:"),
            "src/cli.rs must not contain an indented `verbose:` field declaration"
        );
        // Short flag `-v` would be declared as `short = 'v'` on a verbose field;
        // clap 4 derives only auto-short-flags from `#[arg(short, long)]`. The `-a`
        // short for `--all` in `analyze` uses `short` with the `all` field; there
        // is no `-v` because no `verbose` field exists. We assert `short = 'v'` is
        // absent to rule out any verbose field declared with an explicit short.
        assert!(
            !src.contains("short = 'v'"),
            "src/cli.rs must not declare `short = 'v'` (a verbose short flag)"
        );
    }

    // -----------------------------------------------------------------------
    // AC-010 (BC-2.13.001 postcondition 3 / BC-2.13.002 postcondition 3)
    // -----------------------------------------------------------------------

    /// AC-010 (BC-2.13.001 postcondition 3 / BC-2.13.002 postcondition 3):
    /// A valid invocation without any removed flag parses successfully.
    /// `Cli::try_parse_from(["wirerust", "analyze", "test.pcap"])` returns `Ok`.
    ///
    /// This test proves the absent flags do not disturb normal operation: the
    /// removal of --threats, --beacon, --filter, and --verbose does not break
    /// the ordinary CLI surface.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Ok.
    ///   Positive: the parsed command variant is Analyze with target "test.pcap".
    ///   Negative: parse does NOT return Err.
    #[test]
    fn test_valid_invocation_unaffected_by_absent_flags() {
        let cli = parse_ok(&["wirerust", "analyze", "test.pcap"]);
        match cli.command {
            wirerust::cli::Commands::Analyze { targets, .. } => {
                assert_eq!(targets.len(), 1, "exactly one target must be parsed");
                assert_eq!(
                    targets[0],
                    std::path::PathBuf::from("test.pcap"),
                    "target must be test.pcap"
                );
            }
            wirerust::cli::Commands::Summary { .. } => {
                panic!("Expected Analyze, got Summary")
            }
        }
    }

    // -----------------------------------------------------------------------
    // EC-001 (BC-2.13.001 EC-002): --threats before subcommand also rejected
    // -----------------------------------------------------------------------

    /// EC-001 (BC-2.13.001 EC-002): `--threats` placed before the subcommand
    /// is also rejected. clap treats unknown global-position args as errors.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: placement before the subcommand does NOT make the flag valid.
    #[test]
    fn test_EC_001_threats_before_subcommand_rejected() {
        let err = parse_err(&["wirerust", "--threats", "analyze", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --threats before subcommand, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-002: --beacon combined with valid flags errors before analysis
    // -----------------------------------------------------------------------

    /// EC-002: `--beacon` combined with valid flags (e.g., `--dns`) still
    /// produces a clap error before any analysis begins.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err even when other valid flags are present.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: the presence of valid flags does NOT make --beacon accepted.
    #[test]
    fn test_EC_002_beacon_with_valid_flags_still_errors() {
        let err = parse_err(&["wirerust", "analyze", "--dns", "--beacon", "test.pcap"]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --beacon even with --dns present, got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-003: --filter "tcp port 80" rejected
    // -----------------------------------------------------------------------

    /// EC-003: `--filter "tcp port 80"` (space-separated BPF expression as a
    /// single argument) is rejected by clap on `--filter`.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Err.
    ///   Positive: error kind is UnknownArgument.
    ///   Negative: clap does NOT parse `"tcp port 80"` as a positional target.
    #[test]
    fn test_EC_003_filter_with_bpf_expression_rejected() {
        let err = parse_err(&[
            "wirerust",
            "analyze",
            "--filter",
            "tcp port 80",
            "test.pcap",
        ]);
        assert_eq!(
            err.kind(),
            ErrorKind::UnknownArgument,
            "Expected UnknownArgument for --filter \"tcp port 80\", got {:?}",
            err.kind()
        );
    }

    // -----------------------------------------------------------------------
    // EC-004: valid `analyze --http test.pcap` parses Ok
    // -----------------------------------------------------------------------

    /// EC-004: A valid invocation `wirerust analyze --http test.pcap` parses
    /// successfully. None of the four removed flags affect this invocation.
    ///
    /// Discriminating assertions:
    ///   Positive: parse returns Ok.
    ///   Positive: command variant is Analyze with http=true.
    ///   Positive: targets == ["test.pcap"].
    ///   Negative: parse does NOT fail because --threats/--beacon/--filter/
    ///   --verbose are absent (their removal is orthogonal to --http).
    #[test]
    fn test_EC_004_valid_http_invocation_parses_ok() {
        let cli = parse_ok(&["wirerust", "analyze", "--http", "test.pcap"]);
        match cli.command {
            wirerust::cli::Commands::Analyze { http, targets, .. } => {
                assert!(http, "--http must set http=true");
                assert_eq!(targets.len(), 1, "exactly one target must be parsed");
                assert_eq!(
                    targets[0],
                    std::path::PathBuf::from("test.pcap"),
                    "target must be test.pcap"
                );
            }
            wirerust::cli::Commands::Summary { .. } => {
                panic!("Expected Analyze, got Summary")
            }
        }
    }

    // -----------------------------------------------------------------------
    // Unused import guard: ensure ErrorKind is referenced for compilation.
    // -----------------------------------------------------------------------

    #[allow(dead_code)]
    fn _type_check_imports() {
        let _: ErrorKind = ErrorKind::UnknownArgument;
    }
}
