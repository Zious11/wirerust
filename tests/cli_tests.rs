use clap::Parser;
use wirerust::cli::{Cli, Commands, OutputFormat};

#[test]
fn test_analyze_subcommand() {
    let cli = Cli::parse_from([
        "wirerust",
        "analyze",
        "capture.pcap",
        "--threats",
        "--dns",
        "--verbose",
    ]);
    assert!(cli.verbose);
    match cli.command {
        Commands::Analyze {
            targets,
            threats,
            dns,
            ..
        } => {
            assert_eq!(targets, vec![std::path::PathBuf::from("capture.pcap")]);
            assert!(threats);
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
