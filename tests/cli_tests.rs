use clap::Parser;
use wirerust::cli::{Cli, Commands, OutputFormat};

#[test]
fn test_analyze_subcommand() {
    // LESSON-P1.04: `--threats` and `--verbose` were removed as part
    // of the "no unwired CLI flags" sweep; this test now exercises
    // only flags that have wired effects in `src/main.rs`.
    let cli = Cli::parse_from(["wirerust", "analyze", "capture.pcap", "--dns", "--no-color"]);
    assert!(cli.no_color);
    match cli.command {
        Commands::Analyze { targets, dns, .. } => {
            assert_eq!(targets, vec![std::path::PathBuf::from("capture.pcap")]);
            assert!(dns);
        }
        _ => panic!("Expected Analyze command"),
    }
}

#[test]
fn test_summary_subcommand() {
    let cli = Cli::parse_from([
        "wirerust",
        "summary",
        "capture.pcap",
        "--output-format",
        "json",
    ]);
    match cli.command {
        Commands::Summary { targets, .. } => {
            assert_eq!(targets.len(), 1);
        }
        _ => panic!("Expected Summary command"),
    }
    assert_eq!(cli.output_format, Some(OutputFormat::Json));
}

#[test]
fn test_reassembly_flags() {
    let cli = Cli::parse_from([
        "wirerust",
        "analyze",
        "test.pcap",
        "--reassemble",
        "--reassembly-depth",
        "20",
        "--reassembly-memcap",
        "2048",
    ]);
    assert!(cli.reassemble);
    assert_eq!(cli.reassembly_depth, 20);
    assert_eq!(cli.reassembly_memcap, 2048);
}

#[test]
fn test_no_reassemble_flag() {
    let cli = Cli::parse_from(["wirerust", "analyze", "test.pcap", "--no-reassemble"]);
    assert!(cli.no_reassemble);
}

#[test]
fn test_no_color_flag() {
    let cli = Cli::parse_from(["wirerust", "--no-color", "analyze", "test.pcap"]);
    assert!(cli.no_color);
}

#[test]
fn test_multiple_targets() {
    let cli = Cli::parse_from([
        "wirerust",
        "analyze",
        "one.pcap",
        "two.pcapng",
        "/path/to/dir",
    ]);
    match cli.command {
        Commands::Analyze { targets, .. } => {
            assert_eq!(targets.len(), 3);
        }
        _ => panic!("Expected Analyze command"),
    }
}

#[test]
fn test_mitre_flag_parses_on_analyze() {
    let cli = Cli::parse_from(["wirerust", "analyze", "capture.pcap", "--mitre"]);
    match cli.command {
        Commands::Analyze { mitre, .. } => assert!(mitre),
        _ => panic!("Expected Analyze command"),
    }
}

#[test]
fn test_mitre_flag_defaults_false() {
    let cli = Cli::parse_from(["wirerust", "analyze", "capture.pcap"]);
    match cli.command {
        Commands::Analyze { mitre, .. } => assert!(!mitre),
        _ => panic!("Expected Analyze command"),
    }
}

// ---- LESSON-P1.03 / P1.04: CLI flag truth ----

#[test]
fn test_summary_hosts_flag_parses_and_is_bound() {
    // LESSON-P1.03: `--hosts` on `summary` was previously unwired.
    // It now toggles a per-host breakdown in the terminal reporter
    // and must remain a real, bound field on the Summary variant.
    let cli = Cli::parse_from(["wirerust", "summary", "capture.pcap", "--hosts"]);
    match cli.command {
        Commands::Summary { targets, hosts } => {
            assert_eq!(targets, vec![std::path::PathBuf::from("capture.pcap")]);
            assert!(hosts, "--hosts must set Summary.hosts to true");
        }
        _ => panic!("Expected Summary command"),
    }
}

#[test]
fn test_summary_hosts_flag_defaults_false() {
    let cli = Cli::parse_from(["wirerust", "summary", "capture.pcap"]);
    match cli.command {
        Commands::Summary { hosts, .. } => {
            assert!(!hosts, "Summary.hosts must default to false");
        }
        _ => panic!("Expected Summary command"),
    }
}

// ---- LESSON-P2.05: configurable reassembly anomaly thresholds ----

#[test]
fn test_threshold_flags_parse() {
    let cli = Cli::parse_from([
        "wirerust",
        "--overlap-threshold",
        "10",
        "--small-segment-threshold",
        "256",
        "--small-segment-max-bytes",
        "32",
        "--small-segment-ignore-ports",
        "23,513,9000",
        "--out-of-window-threshold",
        "25",
        "analyze",
        "capture.pcap",
    ]);
    assert_eq!(cli.overlap_threshold, Some(10));
    assert_eq!(cli.small_segment_threshold, Some(256));
    assert_eq!(cli.small_segment_max_bytes, Some(32));
    assert_eq!(cli.small_segment_ignore_ports, Some(vec![23, 513, 9000]));
    assert_eq!(cli.out_of_window_threshold, Some(25));
}

#[test]
fn test_threshold_flags_default_to_none() {
    // Absent flags must be None so main.rs leaves the
    // ReassemblyConfig::default() value untouched.
    let cli = Cli::parse_from(["wirerust", "analyze", "capture.pcap"]);
    assert_eq!(cli.overlap_threshold, None);
    assert_eq!(cli.small_segment_threshold, None);
    assert_eq!(cli.small_segment_max_bytes, None);
    assert_eq!(cli.small_segment_ignore_ports, None);
    assert_eq!(cli.out_of_window_threshold, None);
}

#[test]
fn test_reassembly_threshold_flags_reject_out_of_range_values() {
    // LESSON-P2.05 adversarial-review follow-up: the reassembly
    // threshold flags enforce their documented sane ranges, so an
    // out-of-range value is rejected at parse time rather than silently
    // producing a nonsensical detector configuration.
    for (flag, bad) in [
        ("--overlap-threshold", "300"),        // range 0..=255
        ("--small-segment-threshold", "5000"), // range 0..=2048
        ("--small-segment-max-bytes", "5000"), // range 0..=2048
    ] {
        let result = Cli::try_parse_from(["wirerust", flag, bad, "analyze", "x.pcap"]);
        assert!(
            result.is_err(),
            "`{flag} {bad}` is out of range and must be rejected"
        );
    }
}

#[test]
fn test_removed_unwired_flags_are_rejected() {
    // LESSON-P1.04: --threats, --verbose, --beacon, --filter, and
    // --services were removed in the "no unwired CLI flags" sweep
    // (Phase C synthesis). Re-introducing any of them without
    // wiring a consumer in main.rs would regress the convention;
    // this test asserts clap rejects each as unknown.
    for flag in [
        "--threats",
        "--verbose",
        "--beacon",
        "--filter",
        "--services",
    ] {
        let result = Cli::try_parse_from(["wirerust", "analyze", "x.pcap", flag]);
        assert!(
            result.is_err(),
            "removed flag `{flag}` must not parse — LESSON-P1.04 regressed"
        );
    }
}
