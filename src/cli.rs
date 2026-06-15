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

use crate::analyzer::dnp3::DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT;

/// Value parser for usize arguments that must be >= 1 (0 is rejected).
/// Used for `--reassembly-depth` and `--reassembly-memcap`.
fn parse_nonzero_usize(s: &str) -> Result<usize, String> {
    let v: usize = s.parse().map_err(|e| format!("invalid value '{s}': {e}"))?;
    if v == 0 {
        return Err("0 is not in 1..".to_string());
    }
    Ok(v)
}

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
    #[arg(long, global = true, default_value_t = 10, value_parser = parse_nonzero_usize)]
    pub reassembly_depth: usize,

    /// Global reassembly memory cap in MB (default: 1024)
    #[arg(long, global = true, default_value_t = 1024, value_parser = parse_nonzero_usize)]
    pub reassembly_memcap: usize,

    /// Override the overlapping-segment anomaly threshold (default: 50).
    /// Per flow direction; accepted range 0–255 (Snort's
    /// `overlap_limit` range). See LESSON-P2.05 in the reassembly config.
    #[arg(long, global = true, value_parser = clap::value_parser!(u32).range(0..=255))]
    pub overlap_threshold: Option<u32>,

    /// Override the small-segment anomaly threshold (default: 100).
    /// Length of a consecutive run of undersized segments, per flow
    /// direction, above which the anomaly fires. Accepted range 0–2048
    /// (Snort's `small_segments` count range). See LESSON-P2.05.
    #[arg(long, global = true, value_parser = clap::value_parser!(u32).range(0..=2048))]
    pub small_segment_threshold: Option<u32>,

    /// Override the small-segment payload-size cutoff in bytes
    /// (default: 16). A segment shorter than this counts as "small";
    /// 0 disables small-segment detection. Accepted range 0–2048
    /// (Snort's `small_segments` size range). See LESSON-P2.05.
    #[arg(long, global = true, value_parser = clap::value_parser!(u16).range(0..=2048))]
    pub small_segment_max_bytes: Option<u16>,

    /// Override the ports exempt from small-segment detection
    /// (default: 23,513 — telnet, rlogin). Comma-separated; a flow is
    /// exempt if either endpoint port matches. See LESSON-P2.05.
    #[arg(long, global = true, value_delimiter = ',')]
    pub small_segment_ignore_ports: Option<Vec<u16>>,

    /// Override the out-of-window anomaly threshold (default: 100).
    /// Per flow direction; see LESSON-P2.05.
    #[arg(long, global = true)]
    pub out_of_window_threshold: Option<u32>,

    /// Idle flow timeout in seconds. Flows silent for longer than this value
    /// are expired and removed from the flow table. Default: 300.
    /// Minimum: 1 (0 is rejected).
    #[arg(long, global = true, default_value_t = 300, value_parser = clap::value_parser!(u64).range(1..))]
    pub flow_timeout: u64,

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

        /// Analyze Modbus TCP traffic (port 502, requires stream reassembly)
        /// (BC-2.14.023 — default-off; included by --all)
        #[arg(long)]
        modbus: bool,

        /// Per-flow write-burst threshold: fires T0806+T1692.001 when more than N
        /// write-class FCs are observed within any 1-second window (BC-2.14.024).
        /// Default: 20. Must be >= 1.
        #[arg(long, default_value_t = 20)]
        modbus_write_burst_threshold: u32,

        /// Per-flow sustained-rate threshold: fires T0806+T1692.001 when the average
        /// write-FC rate exceeds M writes/second over a contiguous window of >= 2s
        /// (BC-2.14.024). Default: 10. Must be >= 1.
        #[arg(long, default_value_t = 10)]
        modbus_write_sustained_threshold: u32,

        /// Analyze DNP3 TCP traffic (port 20000, requires stream reassembly)
        /// (BC-2.15.021 — default-off; included by --all)
        #[arg(long)]
        dnp3: bool,

        /// Per-flow direct-operate burst threshold: fires T1692.001 when more than N
        /// Control-class FCs are observed within the detection window (BC-2.15.010 /
        /// BC-2.15.017). Default: 10.
        #[arg(long, default_value_t = DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT)]
        dnp3_direct_operate_threshold: u32,

        /// Analyze ARP traffic for spoofing, GARP anomalies, malformed frames, and
        /// L2/L3 sender-MAC mismatch (BC-2.16.011 — default-off; included by --all).
        #[arg(long)]
        arp: bool,

        /// D1 spoof escalation threshold: number of MAC rebinds within
        /// ARP_FLAP_WINDOW_SECS (60 s) before a HIGH severity finding is emitted.
        /// Default: 3. Set to 1 to fire HIGH on the very first rebind.
        /// BC-2.16.012 primary deliverable (STORY-114).
        #[arg(long, default_value_t = 3)]
        arp_spoof_threshold: u32,

        /// D3 storm rate threshold: frames/second per source MAC above which a
        /// MEDIUM/Anomaly storm finding is emitted. Default: 50 (wirerust engineering
        /// default — not derived from any external standard). ICS/OT operators with
        /// PLCs or RTUs should typically lower this to 5–20/s (BC-2.16.013).
        /// BC-2.16.013 primary deliverable (STORY-115).
        #[arg(long, default_value_t = 50)]
        arp_storm_rate: u32,
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
