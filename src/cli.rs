use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Csv,
}

#[derive(Parser, Debug)]
#[command(
    name = "wirerust",
    about = "Fast PCAP forensics and network triage CLI",
    version
)]
pub struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Output format (default: terminal table)
    #[arg(long, global = true, value_enum)]
    pub output_format: Option<OutputFormat>,

    /// Write JSON output to file
    #[arg(long, global = true)]
    pub json: Option<Option<PathBuf>>,

    /// Write CSV output to file
    #[arg(long, global = true)]
    pub csv: Option<Option<PathBuf>>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Analyze PCAP files for threats and anomalies
    Analyze {
        /// PCAP files or directories to analyze
        #[arg(required = true)]
        targets: Vec<PathBuf>,

        /// Run threat detection
        #[arg(long)]
        threats: bool,

        /// Analyze DNS traffic
        #[arg(long)]
        dns: bool,

        /// Analyze HTTP traffic
        #[arg(long)]
        http: bool,

        /// Analyze TLS handshakes
        #[arg(long)]
        tls: bool,

        /// Detect C2 beaconing patterns
        #[arg(long)]
        beacon: bool,

        /// Run all analyzers
        #[arg(short, long)]
        all: bool,

        /// BPF filter expression
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Generate a triage summary of PCAP files
    Summary {
        /// PCAP files or directories to summarize
        #[arg(required = true)]
        targets: Vec<PathBuf>,

        /// Include per-host breakdown
        #[arg(long)]
        hosts: bool,

        /// Include service/port breakdown
        #[arg(long)]
        services: bool,
    },
}
