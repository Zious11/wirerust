//! Command-line interface definition (clap derive).
//!
//! Surfaces two subcommands: `analyze` (full pipeline with selectable
//! per-protocol analyzers + MITRE grouping) and `summary` (capture-level
//! triage view with optional per-host breakdown). Global flags govern
//! output channel (`--json`, `--csv`, `--output-format`), reassembly
//! limits (`--reassembly-depth`, `--reassembly-memcap`), and color.
//!
//! See the [`Cli`] struct comment for the "no unwired flags" convention
//! (LESSON-P1.04).

use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Json,
    Csv,
}

/// CLI surface.
///
/// LESSON-P1.04 ("no unwired CLI flags" convention): every flag and
/// option here must be consumed somewhere in `src/main.rs`. Declaring a
/// flag in clap that has no behavioral effect misleads users who read
/// `--help` and write scripts against the surface. The brownfield-ingest
/// Phase C synthesis identified 5 unwired flags (`--verbose`,
/// `--threats`, `--beacon`, `--filter` on `analyze`; `--services` on
/// `summary`) which have been removed in this PR. A 6th — `--hosts` on
/// `summary` — was previously unwired and has been *wired* to gate a
/// per-host breakdown in the terminal reporter (LESSON-P1.03).
///
/// When adding a new flag here, verify the consumer path in `main.rs`
/// in the same change. CI does not yet enforce this — see DEBT register.
#[derive(Parser, Debug)]
#[command(
    name = "wirerust",
    about = "Fast PCAP forensics and network triage CLI",
    version
)]
pub struct Cli {
    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Output format (default: terminal table)
    #[arg(long, global = true, value_enum)]
    pub output_format: Option<OutputFormat>,

    /// Write JSON output to file (or stdout if no path given).
    /// Mutually exclusive with --csv.
    #[arg(long, global = true, conflicts_with = "csv")]
    pub json: Option<Option<PathBuf>>,

    /// Write CSV output to file (or stdout if no path given).
    /// Emits the findings table only — see the CSV reporter docs.
    #[arg(long, global = true)]
    pub csv: Option<Option<PathBuf>>,

    /// Force TCP stream reassembly on
    #[arg(long, global = true, conflicts_with = "no_reassemble")]
    pub reassemble: bool,

    /// Force TCP stream reassembly off (quick scan)
    #[arg(long, global = true)]
    pub no_reassemble: bool,

    /// Per-direction stream reassembly limit in MB (default: 10)
    #[arg(long, global = true, default_value_t = 10)]
    pub reassembly_depth: usize,

    /// Global reassembly memory cap in MB (default: 1024)
    #[arg(long, global = true, default_value_t = 1024)]
    pub reassembly_memcap: usize,

    /// Override the overlapping-segment anomaly threshold (default: 50).
    /// Per flow direction; see LESSON-P2.05 in the reassembly config.
    #[arg(long, global = true)]
    pub overlap_threshold: Option<u32>,

    /// Override the small-segment anomaly threshold (default: 2048).
    /// The default is permissive — lower it (low hundreds) to catch
    /// fine-grained segmentation evasion. See LESSON-P2.05.
    #[arg(long, global = true)]
    pub small_segment_threshold: Option<u32>,

    /// Override the out-of-window anomaly threshold (default: 100).
    /// Per flow direction; see LESSON-P2.05.
    #[arg(long, global = true)]
    pub out_of_window_threshold: Option<u32>,

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

        /// Analyze DNS traffic
        #[arg(long)]
        dns: bool,

        /// Analyze HTTP traffic
        #[arg(long)]
        http: bool,

        /// Analyze TLS handshakes
        #[arg(long)]
        tls: bool,

        /// Group findings by MITRE ATT&CK tactic and show technique names
        #[arg(long)]
        mitre: bool,

        /// Run all analyzers
        #[arg(short, long)]
        all: bool,
    },

    /// Generate a triage summary of PCAP files
    Summary {
        /// PCAP files or directories to summarize
        #[arg(required = true)]
        targets: Vec<PathBuf>,

        /// Include per-host breakdown of source/destination IPs
        /// (LESSON-P1.03 — previously a no-op flag; now wired to
        /// expand the terminal output's `Hosts: N` count into an
        /// itemized list. The JSON reporter has always emitted the
        /// full `unique_hosts` array independently of this flag.)
        #[arg(long)]
        hosts: bool,
    },
}
