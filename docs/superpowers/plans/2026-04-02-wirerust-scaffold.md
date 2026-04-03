# Wirerust Scaffold Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Scaffold wirerust — a fast PCAP forensics and network triage CLI tool in Rust — with working packet parsing, protocol analysis, and multi-format output.

**Architecture:** Pipeline design: Reader (pcap-file) -> Decoder (etherparse) -> Analyzers (trait-based per protocol) -> Reporter (terminal/JSON/CSV/SQLite). Each stage is a trait with concrete implementations. Parallel file processing via rayon.

**Tech Stack:** Rust 2024 edition, clap 4 (derive), etherparse, pcap-file, serde/serde_json, csv, rusqlite, owo-colors, indicatif, anyhow, rayon, chrono.

---

## File Structure

```
wirerust/
├── Cargo.toml
├── src/
│   ├── main.rs              — Entry point, CLI dispatch
│   ├── cli.rs               — Clap structs and enums
│   ├── reader.rs            — PCAP/PCAPNG file reading, packet iteration
│   ├── decoder.rs           — Packet decoding via etherparse, ParsedPacket type
│   ├── analyzer/
│   │   ├── mod.rs           — ProtocolAnalyzer trait, AnalysisEngine
│   │   ├── dns.rs           — DNS analyzer
│   │   ├── http.rs          — HTTP analyzer
│   │   └── tls.rs           — TLS analyzer (ClientHello parsing)
│   ├── findings.rs          — Finding, Verdict, Confidence, ThreatCategory types
│   ├── reporter/
│   │   ├── mod.rs           — Reporter trait
│   │   ├── terminal.rs      — Colored terminal output
│   │   ├── json.rs          — JSON export
│   │   └── csv.rs           — CSV export
│   └── summary.rs           — Host/service/protocol aggregation
├── tests/
│   ├── cli_tests.rs         — CLI argument parsing tests
│   ├── reader_tests.rs      — PCAP reading tests
│   ├── decoder_tests.rs     — Packet decoding tests
│   ├── analyzer_tests.rs    — Protocol analyzer tests
│   └── fixtures/            — Test pcap files
│       └── small.pcap       — Minimal test capture
└── docs/
    └── superpowers/
        └── plans/
            └── 2026-04-02-wirerust-scaffold.md
```

---

### Task 1: Dependencies and Cargo.toml

**Files:**
- Modify: `Cargo.toml`

- [ ] **Step 1: Write Cargo.toml with all dependencies**

```toml
[package]
name = "wirerust"
version = "0.1.0"
edition = "2024"
description = "Fast PCAP forensics and network triage CLI tool"
license = "MIT"

[dependencies]
clap = { version = "4", features = ["derive"] }
etherparse = "0.16"
pcap-file = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
csv = "1"
anyhow = "1"
owo-colors = "4"
indicatif = "0.17"
chrono = { version = "0.4", features = ["serde"] }
rayon = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check`
Expected: Compiles with no errors (warnings about unused are fine).

- [ ] **Step 3: Commit**

```bash
git add Cargo.toml Cargo.lock
git commit -m "feat: add core dependencies for wirerust"
```

---

### Task 2: Findings Types

**Files:**
- Create: `src/findings.rs`
- Modify: `src/main.rs` (add mod declaration)

- [ ] **Step 1: Write the failing test**

Create `tests/findings_tests.rs`:

```rust
use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};

#[test]
fn test_finding_creation() {
    let finding = Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "Port scan detected from 10.0.0.1".into(),
        evidence: vec!["50 SYN packets to sequential ports in 2s".into()],
        mitre_technique: Some("T1046".into()),
        source_ip: Some("10.0.0.1".parse().unwrap()),
        timestamp: None,
    };
    assert_eq!(finding.verdict, Verdict::Likely);
    assert_eq!(finding.confidence, Confidence::High);
    assert!(finding.mitre_technique.is_some());
}

#[test]
fn test_finding_display() {
    let finding = Finding {
        category: ThreatCategory::C2,
        verdict: Verdict::Inconclusive,
        confidence: Confidence::Medium,
        summary: "Periodic beaconing pattern".into(),
        evidence: vec![],
        mitre_technique: None,
        source_ip: None,
        timestamp: None,
    };
    let display = format!("{finding}");
    assert!(display.contains("C2"));
    assert!(display.contains("INCONCLUSIVE"));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test findings_tests`
Expected: FAIL — module `findings` not found.

- [ ] **Step 3: Write findings.rs**

Create `src/findings.rs`:

```rust
use std::fmt;
use std::net::IpAddr;

use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Likely,
    Unlikely,
    Inconclusive,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Verdict::Likely => write!(f, "LIKELY"),
            Verdict::Unlikely => write!(f, "UNLIKELY"),
            Verdict::Inconclusive => write!(f, "INCONCLUSIVE"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Confidence {
    High,
    Medium,
    Low,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Confidence::High => write!(f, "HIGH"),
            Confidence::Medium => write!(f, "MEDIUM"),
            Confidence::Low => write!(f, "LOW"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ThreatCategory {
    Reconnaissance,
    LateralMovement,
    C2,
    Exfiltration,
    CredentialAccess,
    Execution,
    Persistence,
    Anomaly,
}

impl fmt::Display for ThreatCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub category: ThreatCategory,
    pub verdict: Verdict,
    pub confidence: Confidence,
    pub summary: String,
    pub evidence: Vec<String>,
    pub mitre_technique: Option<String>,
    pub source_ip: Option<IpAddr>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<DateTime<Utc>>,
}

impl fmt::Display for Finding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{cat}] {verdict} ({conf}) — {summary}",
            cat = self.category,
            verdict = self.verdict,
            conf = self.confidence,
            summary = self.summary,
        )
    }
}
```

- [ ] **Step 4: Wire up lib.rs for integration tests**

Create `src/lib.rs`:

```rust
pub mod findings;
```

Update `src/main.rs`:

```rust
use anyhow::Result;

mod cli;

fn main() -> Result<()> {
    println!("wirerust — PCAP forensics tool");
    Ok(())
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test findings_tests`
Expected: 2 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/findings.rs src/lib.rs src/main.rs tests/findings_tests.rs
git commit -m "feat: add Finding, Verdict, Confidence, ThreatCategory types"
```

---

### Task 3: CLI Argument Parsing

**Files:**
- Create: `src/cli.rs`
- Modify: `src/lib.rs` (add mod)
- Create: `tests/cli_tests.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/cli_tests.rs`:

```rust
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
        Commands::Analyze { targets, threats, dns, .. } => {
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
        "--json",
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test cli_tests`
Expected: FAIL — module `cli` not found.

- [ ] **Step 3: Write cli.rs**

Create `src/cli.rs`:

```rust
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
```

- [ ] **Step 4: Add cli module to lib.rs**

Update `src/lib.rs`:

```rust
pub mod cli;
pub mod findings;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test cli_tests`
Expected: 4 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/cli.rs src/lib.rs tests/cli_tests.rs
git commit -m "feat: add CLI argument parsing with clap derive"
```

---

### Task 4: PCAP/PCAPNG Reader

**Files:**
- Create: `src/reader.rs`
- Create: `tests/reader_tests.rs`
- Create: `tests/fixtures/` (generate a test pcap)
- Modify: `src/lib.rs`

- [ ] **Step 1: Generate a minimal test pcap fixture**

We need a tiny pcap file for tests. Create a script that writes one:

Create `tests/gen_fixture.rs` (a standalone binary):

Actually, let's use raw bytes. Create `tests/fixtures/` and write a minimal valid pcap with one TCP packet programmatically in the test itself.

Write the failing test in `tests/reader_tests.rs`:

```rust
use std::io::Cursor;

use wirerust::reader::PcapSource;

// Minimal valid pcap file: global header + 1 packet (Ethernet + IPv4 + TCP)
fn minimal_pcap_bytes() -> Vec<u8> {
    let mut buf = Vec::new();

    // Global header (24 bytes)
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes()); // magic
    buf.extend_from_slice(&2u16.to_le_bytes());           // version major
    buf.extend_from_slice(&4u16.to_le_bytes());           // version minor
    buf.extend_from_slice(&0i32.to_le_bytes());           // thiszone
    buf.extend_from_slice(&0u32.to_le_bytes());           // sigfigs
    buf.extend_from_slice(&65535u32.to_le_bytes());       // snaplen
    buf.extend_from_slice(&1u32.to_le_bytes());           // network (Ethernet)

    // Packet: Ethernet(14) + IPv4(20) + TCP(20) = 54 bytes
    let packet_data: Vec<u8> = vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00,                         // ethertype: IPv4
        // IPv4 header (20 bytes)
        0x45, 0x00, 0x00, 0x28,             // version/IHL, DSCP, total length=40
        0x00, 0x01, 0x00, 0x00,             // identification, flags/fragment
        0x40, 0x06, 0x00, 0x00,             // TTL=64, protocol=TCP, checksum
        0x0a, 0x00, 0x00, 0x01,             // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02,             // dst: 10.0.0.2
        // TCP header (20 bytes)
        0x00, 0x50, 0x04, 0xd2,             // src port 80, dst port 1234
        0x00, 0x00, 0x00, 0x01,             // seq number
        0x00, 0x00, 0x00, 0x00,             // ack number
        0x50, 0x02, 0xff, 0xff,             // data offset=5, SYN, window
        0x00, 0x00, 0x00, 0x00,             // checksum, urgent pointer
    ];

    let captured_len = packet_data.len() as u32;

    // Packet header (16 bytes)
    buf.extend_from_slice(&1000u32.to_le_bytes());         // ts_sec
    buf.extend_from_slice(&0u32.to_le_bytes());            // ts_usec
    buf.extend_from_slice(&captured_len.to_le_bytes());    // incl_len
    buf.extend_from_slice(&captured_len.to_le_bytes());    // orig_len

    // Packet data
    buf.extend_from_slice(&packet_data);

    buf
}

#[test]
fn test_read_pcap_packets() {
    let data = minimal_pcap_bytes();
    let cursor = Cursor::new(data);
    let source = PcapSource::from_pcap_reader(cursor).unwrap();
    let packets = source.packets;
    assert_eq!(packets.len(), 1);
    assert_eq!(packets[0].timestamp_secs, 1000);
    assert_eq!(packets[0].data.len(), 54);
}

#[test]
fn test_empty_pcap_no_packets() {
    let mut buf = Vec::new();
    // Global header only, no packets
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());

    let cursor = Cursor::new(buf);
    let source = PcapSource::from_pcap_reader(cursor).unwrap();
    assert!(source.packets.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test reader_tests`
Expected: FAIL — module `reader` not found.

- [ ] **Step 3: Write reader.rs**

Create `src/reader.rs`:

```rust
use std::io::Read;

use anyhow::{Context, Result};
use pcap_file::pcap::PcapReader;

#[derive(Debug, Clone)]
pub struct RawPacket {
    pub timestamp_secs: u32,
    pub timestamp_usecs: u32,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct PcapSource {
    pub packets: Vec<RawPacket>,
}

impl PcapSource {
    pub fn from_pcap_reader<R: Read>(reader: R) -> Result<Self> {
        let mut pcap_reader =
            PcapReader::new(reader).context("Failed to parse pcap header")?;

        let mut packets = Vec::new();

        while let Some(raw_packet) = pcap_reader.next_packet() {
            let raw_packet = raw_packet.context("Failed to read packet")?;
            packets.push(RawPacket {
                timestamp_secs: raw_packet.timestamp.as_secs() as u32,
                timestamp_usecs: raw_packet.timestamp.subsec_nanos() / 1000,
                data: raw_packet.data.into_owned(),
            });
        }

        Ok(PcapSource { packets })
    }

    pub fn from_file(path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(path)
            .with_context(|| format!("Failed to open {}", path.display()))?;
        let reader = std::io::BufReader::new(file);
        Self::from_pcap_reader(reader)
    }
}
```

- [ ] **Step 4: Add reader module to lib.rs**

Update `src/lib.rs`:

```rust
pub mod cli;
pub mod findings;
pub mod reader;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test reader_tests`
Expected: 2 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/reader.rs src/lib.rs tests/reader_tests.rs
git commit -m "feat: add PCAP file reader with packet iteration"
```

---

### Task 5: Packet Decoder

**Files:**
- Create: `src/decoder.rs`
- Create: `tests/decoder_tests.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/decoder_tests.rs`:

```rust
use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{decode_packet, Protocol, TransportInfo};

fn make_tcp_packet() -> Vec<u8> {
    vec![
        // Ethernet header (14 bytes)
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, // dst mac
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, // src mac
        0x08, 0x00,                         // ethertype: IPv4
        // IPv4 header (20 bytes)
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x01, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        0xc0, 0xa8, 0x01, 0x0a,             // src: 192.168.1.10
        0xc0, 0xa8, 0x01, 0x01,             // dst: 192.168.1.1
        // TCP header (20 bytes)
        0xc0, 0x01, 0x00, 0x50,             // src port 49153, dst port 80
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x02, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x00,
    ]
}

fn make_udp_packet() -> Vec<u8> {
    vec![
        // Ethernet header
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
        0x08, 0x00,
        // IPv4 header (20 bytes), protocol=UDP (0x11)
        0x45, 0x00, 0x00, 0x1c,
        0x00, 0x01, 0x00, 0x00,
        0x40, 0x11, 0x00, 0x00,
        0x0a, 0x00, 0x00, 0x01,             // src: 10.0.0.1
        0x0a, 0x00, 0x00, 0x02,             // dst: 10.0.0.2
        // UDP header (8 bytes)
        0xd9, 0x03, 0x00, 0x35,             // src port 55555, dst port 53
        0x00, 0x08, 0x00, 0x00,             // length=8, checksum
    ]
}

#[test]
fn test_decode_tcp_packet() {
    let data = make_tcp_packet();
    let parsed = decode_packet(&data).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));

    match parsed.transport {
        TransportInfo::Tcp { src_port, dst_port, .. } => {
            assert_eq!(src_port, 49153);
            assert_eq!(dst_port, 80);
        }
        _ => panic!("Expected TCP"),
    }

    assert_eq!(parsed.protocol, Protocol::Tcp);
}

#[test]
fn test_decode_udp_dns_packet() {
    let data = make_udp_packet();
    let parsed = decode_packet(&data).unwrap();

    assert_eq!(parsed.src_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(parsed.dst_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)));

    match parsed.transport {
        TransportInfo::Udp { src_port, dst_port } => {
            assert_eq!(src_port, 55555);
            assert_eq!(dst_port, 53);
        }
        _ => panic!("Expected UDP"),
    }

    assert_eq!(parsed.protocol, Protocol::Udp);
    assert_eq!(parsed.app_protocol_hint(), Some("DNS"));
}

#[test]
fn test_decode_invalid_packet() {
    let garbage = vec![0x00, 0x01, 0x02];
    assert!(decode_packet(&garbage).is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test decoder_tests`
Expected: FAIL — module `decoder` not found.

- [ ] **Step 3: Write decoder.rs**

Create `src/decoder.rs`:

```rust
use std::net::IpAddr;

use anyhow::{anyhow, Result};
use etherparse::SlicedPacket;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Other(u8),
}

#[derive(Debug, Clone)]
pub enum TransportInfo {
    Tcp {
        src_port: u16,
        dst_port: u16,
        syn: bool,
        ack: bool,
        fin: bool,
        rst: bool,
    },
    Udp {
        src_port: u16,
        dst_port: u16,
    },
    None,
}

#[derive(Debug, Clone)]
pub struct ParsedPacket {
    pub src_ip: IpAddr,
    pub dst_ip: IpAddr,
    pub protocol: Protocol,
    pub transport: TransportInfo,
    pub payload: Vec<u8>,
    pub packet_len: usize,
}

impl ParsedPacket {
    /// Guess the application-layer protocol based on port numbers.
    pub fn app_protocol_hint(&self) -> Option<&'static str> {
        let dst = match &self.transport {
            TransportInfo::Tcp { dst_port, .. } => *dst_port,
            TransportInfo::Udp { dst_port, .. } => *dst_port,
            TransportInfo::None => return None,
        };
        let src = match &self.transport {
            TransportInfo::Tcp { src_port, .. } => *src_port,
            TransportInfo::Udp { src_port, .. } => *src_port,
            TransportInfo::None => return None,
        };

        match (src, dst) {
            (53, _) | (_, 53) => Some("DNS"),
            (80, _) | (_, 80) => Some("HTTP"),
            (443, _) | (_, 443) => Some("TLS"),
            (22, _) | (_, 22) => Some("SSH"),
            (445, _) | (_, 445) => Some("SMB"),
            (502, _) | (_, 502) => Some("Modbus"),
            (20000, _) | (_, 20000) => Some("DNP3"),
            _ => None,
        }
    }
}

pub fn decode_packet(data: &[u8]) -> Result<ParsedPacket> {
    let sliced =
        SlicedPacket::from_ethernet(data).map_err(|e| anyhow!("Parse error: {e}"))?;

    let (src_ip, dst_ip, ip_protocol) = match &sliced.net {
        Some(etherparse::NetSlice::Ipv4(ipv4)) => {
            let header = ipv4.header();
            (
                IpAddr::V4(header.source_addr()),
                IpAddr::V4(header.destination_addr()),
                header.protocol(),
            )
        }
        Some(etherparse::NetSlice::Ipv6(ipv6)) => {
            let header = ipv6.header();
            (
                IpAddr::V6(header.source_addr()),
                IpAddr::V6(header.destination_addr()),
                ipv6.header().next_header(),
            )
        }
        None => return Err(anyhow!("No IP layer found")),
    };

    let (protocol, transport) = match &sliced.transport {
        Some(etherparse::TransportSlice::Tcp(tcp)) => {
            (Protocol::Tcp, TransportInfo::Tcp {
                src_port: tcp.source_port(),
                dst_port: tcp.destination_port(),
                syn: tcp.syn(),
                ack: tcp.ack(),
                fin: tcp.fin(),
                rst: tcp.rst(),
            })
        }
        Some(etherparse::TransportSlice::Udp(udp)) => {
            (Protocol::Udp, TransportInfo::Udp {
                src_port: udp.source_port(),
                dst_port: udp.destination_port(),
            })
        }
        Some(etherparse::TransportSlice::Icmpv4(_) | etherparse::TransportSlice::Icmpv6(_)) => {
            (Protocol::Icmp, TransportInfo::None)
        }
        None => (Protocol::Other(ip_protocol.0), TransportInfo::None),
    };

    let payload = match &sliced.transport {
        Some(etherparse::TransportSlice::Tcp(tcp)) => tcp.payload().to_vec(),
        Some(etherparse::TransportSlice::Udp(udp)) => udp.payload().to_vec(),
        _ => Vec::new(),
    };

    Ok(ParsedPacket {
        src_ip,
        dst_ip,
        protocol,
        transport,
        payload,
        packet_len: data.len(),
    })
}
```

- [ ] **Step 4: Add decoder module to lib.rs**

Update `src/lib.rs`:

```rust
pub mod cli;
pub mod decoder;
pub mod findings;
pub mod reader;
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test decoder_tests`
Expected: 3 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/decoder.rs src/lib.rs tests/decoder_tests.rs
git commit -m "feat: add packet decoder with etherparse (TCP/UDP/ICMP)"
```

---

### Task 6: Summary Aggregation

**Files:**
- Create: `src/summary.rs`
- Create: `tests/summary_tests.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/summary_tests.rs`:

```rust
use std::net::{IpAddr, Ipv4Addr};

use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};
use wirerust::summary::Summary;

fn make_parsed(src: [u8; 4], dst: [u8; 4], src_port: u16, dst_port: u16) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::from(src)),
        dst_ip: IpAddr::V4(Ipv4Addr::from(dst)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port,
            dst_port,
            syn: false,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 54,
    }
}

#[test]
fn test_summary_host_counting() {
    let packets = vec![
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 3], 12346, 443),
        make_parsed([10, 0, 0, 2], [10, 0, 0, 1], 80, 12345),
    ];

    let mut summary = Summary::new();
    for pkt in &packets {
        summary.ingest(pkt);
    }

    assert_eq!(summary.total_packets, 3);
    assert_eq!(summary.unique_hosts().len(), 3); // 10.0.0.1, .2, .3
}

#[test]
fn test_summary_protocol_breakdown() {
    let packets = vec![
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12346, 80),
    ];

    let mut summary = Summary::new();
    for pkt in &packets {
        summary.ingest(pkt);
    }

    let proto_counts = summary.protocol_counts();
    assert_eq!(*proto_counts.get(&Protocol::Tcp).unwrap(), 2);
}

#[test]
fn test_summary_service_detection() {
    let packets = vec![
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12345, 80),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12346, 443),
        make_parsed([10, 0, 0, 1], [10, 0, 0, 2], 12347, 443),
    ];

    let mut summary = Summary::new();
    for pkt in &packets {
        summary.ingest(pkt);
    }

    let services = summary.service_counts();
    assert_eq!(*services.get("HTTP").unwrap(), 1);
    assert_eq!(*services.get("TLS").unwrap(), 2);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test summary_tests`
Expected: FAIL — module `summary` not found.

- [ ] **Step 3: Write summary.rs**

Create `src/summary.rs`:

```rust
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;

use serde::Serialize;

use crate::decoder::{ParsedPacket, Protocol, TransportInfo};

#[derive(Debug, Serialize)]
pub struct Summary {
    pub total_packets: u64,
    pub total_bytes: u64,
    hosts: HashSet<IpAddr>,
    protocols: HashMap<Protocol, u64>,
    services: HashMap<String, u64>,
}

impl Summary {
    pub fn new() -> Self {
        Summary {
            total_packets: 0,
            total_bytes: 0,
            hosts: HashSet::new(),
            protocols: HashMap::new(),
            services: HashMap::new(),
        }
    }

    pub fn ingest(&mut self, packet: &ParsedPacket) {
        self.total_packets += 1;
        self.total_bytes += packet.packet_len as u64;
        self.hosts.insert(packet.src_ip);
        self.hosts.insert(packet.dst_ip);
        *self.protocols.entry(packet.protocol).or_insert(0) += 1;

        if let Some(svc) = packet.app_protocol_hint() {
            *self.services.entry(svc.to_string()).or_insert(0) += 1;
        }
    }

    pub fn unique_hosts(&self) -> Vec<IpAddr> {
        let mut hosts: Vec<_> = self.hosts.iter().copied().collect();
        hosts.sort();
        hosts
    }

    pub fn protocol_counts(&self) -> &HashMap<Protocol, u64> {
        &self.protocols
    }

    pub fn service_counts(&self) -> &HashMap<String, u64> {
        &self.services
    }
}
```

- [ ] **Step 4: Add summary module to lib.rs and add Hash derive to Protocol**

Update `src/lib.rs`:

```rust
pub mod cli;
pub mod decoder;
pub mod findings;
pub mod reader;
pub mod summary;
```

In `src/decoder.rs`, update Protocol derive to include `Hash`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Other(u8),
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test --test summary_tests`
Expected: 3 tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/summary.rs src/decoder.rs src/lib.rs tests/summary_tests.rs
git commit -m "feat: add traffic summary aggregation (hosts, protocols, services)"
```

---

### Task 7: Analyzer Trait and DNS Analyzer

**Files:**
- Create: `src/analyzer/mod.rs`
- Create: `src/analyzer/dns.rs`
- Create: `tests/analyzer_tests.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/analyzer_tests.rs`:

```rust
use std::net::{IpAddr, Ipv4Addr};

use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::decoder::{ParsedPacket, Protocol, TransportInfo};

fn make_dns_packet(payload: &[u8]) -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Udp,
        transport: TransportInfo::Udp {
            src_port: 12345,
            dst_port: 53,
        },
        payload: payload.to_vec(),
        packet_len: 60 + payload.len(),
    }
}

fn make_non_dns_packet() -> ParsedPacket {
    ParsedPacket {
        src_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
        dst_ip: IpAddr::V4(Ipv4Addr::new(10, 0, 0, 2)),
        protocol: Protocol::Tcp,
        transport: TransportInfo::Tcp {
            src_port: 12345,
            dst_port: 80,
            syn: true,
            ack: false,
            fin: false,
            rst: false,
        },
        payload: vec![],
        packet_len: 54,
    }
}

#[test]
fn test_dns_analyzer_matches_dns_packets() {
    let analyzer = DnsAnalyzer::new();
    let dns_pkt = make_dns_packet(&[0; 12]); // minimal DNS header
    let non_dns = make_non_dns_packet();

    assert!(analyzer.can_decode(&dns_pkt));
    assert!(!analyzer.can_decode(&non_dns));
}

#[test]
fn test_dns_analyzer_counts_queries() {
    let mut analyzer = DnsAnalyzer::new();
    let pkt = make_dns_packet(&[0; 12]);

    let findings = analyzer.analyze(&pkt);
    // No findings from a single normal query
    assert!(findings.is_empty());

    let summary = analyzer.summarize();
    assert_eq!(summary.packets_analyzed, 1);
    assert!(summary.detail.contains_key("dns_queries"));
}

#[test]
fn test_analyzer_trait_name() {
    let analyzer = DnsAnalyzer::new();
    assert_eq!(analyzer.name(), "DNS");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test analyzer_tests`
Expected: FAIL — module `analyzer` not found.

- [ ] **Step 3: Write the analyzer trait**

Create `src/analyzer/mod.rs`:

```rust
pub mod dns;

use std::collections::HashMap;

use serde::Serialize;

use crate::decoder::ParsedPacket;
use crate::findings::Finding;

#[derive(Debug, Serialize)]
pub struct AnalysisSummary {
    pub analyzer_name: String,
    pub packets_analyzed: u64,
    pub detail: HashMap<String, serde_json::Value>,
}

pub trait ProtocolAnalyzer {
    /// Human-readable name for this analyzer.
    fn name(&self) -> &'static str;

    /// Return true if this analyzer handles the given packet.
    fn can_decode(&self, packet: &ParsedPacket) -> bool;

    /// Process a packet. Returns any findings (threats, anomalies).
    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding>;

    /// Produce a summary after all packets have been processed.
    fn summarize(&self) -> AnalysisSummary;
}
```

- [ ] **Step 4: Write the DNS analyzer**

Create `src/analyzer/dns.rs`:

```rust
use std::collections::HashMap;

use crate::analyzer::{AnalysisSummary, ProtocolAnalyzer};
use crate::decoder::{ParsedPacket, TransportInfo};
use crate::findings::Finding;

pub struct DnsAnalyzer {
    query_count: u64,
    response_count: u64,
    unique_queried_domains: HashMap<String, u64>,
}

impl DnsAnalyzer {
    pub fn new() -> Self {
        DnsAnalyzer {
            query_count: 0,
            response_count: 0,
            unique_queried_domains: HashMap::new(),
        }
    }

    fn is_dns_port(src: u16, dst: u16) -> bool {
        src == 53 || dst == 53
    }

    fn is_query(payload: &[u8]) -> bool {
        // DNS header: first 2 bytes = transaction ID, byte 2 bit 7 = QR (0=query, 1=response)
        if payload.len() < 12 {
            return false;
        }
        (payload[2] & 0x80) == 0
    }
}

impl ProtocolAnalyzer for DnsAnalyzer {
    fn name(&self) -> &'static str {
        "DNS"
    }

    fn can_decode(&self, packet: &ParsedPacket) -> bool {
        match &packet.transport {
            TransportInfo::Udp { src_port, dst_port } => {
                Self::is_dns_port(*src_port, *dst_port)
            }
            TransportInfo::Tcp { src_port, dst_port, .. } => {
                Self::is_dns_port(*src_port, *dst_port)
            }
            TransportInfo::None => false,
        }
    }

    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding> {
        if Self::is_query(&packet.payload) {
            self.query_count += 1;
        } else {
            self.response_count += 1;
        }

        // TODO: future — extract domain names from DNS payload,
        // detect tunneling (high entropy, long labels), excessive NXDOMAIN, etc.

        Vec::new()
    }

    fn summarize(&self) -> AnalysisSummary {
        let mut detail = HashMap::new();
        detail.insert(
            "dns_queries".to_string(),
            serde_json::json!(self.query_count),
        );
        detail.insert(
            "dns_responses".to_string(),
            serde_json::json!(self.response_count),
        );

        AnalysisSummary {
            analyzer_name: self.name().to_string(),
            packets_analyzed: self.query_count + self.response_count,
            detail,
        }
    }
}
```

- [ ] **Step 5: Add analyzer module to lib.rs**

Update `src/lib.rs`:

```rust
pub mod analyzer;
pub mod cli;
pub mod decoder;
pub mod findings;
pub mod reader;
pub mod summary;
```

- [ ] **Step 6: Run tests**

Run: `cargo test --test analyzer_tests`
Expected: 3 tests PASS.

- [ ] **Step 7: Commit**

```bash
git add src/analyzer/ src/lib.rs tests/analyzer_tests.rs
git commit -m "feat: add ProtocolAnalyzer trait and DNS analyzer"
```

---

### Task 8: Terminal Reporter

**Files:**
- Create: `src/reporter/mod.rs`
- Create: `src/reporter/terminal.rs`
- Create: `src/reporter/json.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Write the failing test**

Create `tests/reporter_tests.rs`:

```rust
use std::collections::HashMap;

use wirerust::findings::{Confidence, Finding, ThreatCategory, Verdict};
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::Reporter;
use wirerust::summary::Summary;

#[test]
fn test_json_reporter_produces_valid_json() {
    let reporter = JsonReporter;
    let summary = Summary::new();
    let findings = vec![Finding {
        category: ThreatCategory::Reconnaissance,
        verdict: Verdict::Likely,
        confidence: Confidence::High,
        summary: "Test finding".into(),
        evidence: vec!["evidence line".into()],
        mitre_technique: Some("T1046".into()),
        source_ip: None,
        timestamp: None,
    }];
    let analyzer_summaries = vec![];

    let output = reporter.render(&summary, &findings, &analyzer_summaries);
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert!(parsed.get("summary").is_some());
    assert!(parsed.get("findings").is_some());
    let findings_arr = parsed["findings"].as_array().unwrap();
    assert_eq!(findings_arr.len(), 1);
    assert_eq!(findings_arr[0]["summary"], "Test finding");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --test reporter_tests`
Expected: FAIL — module `reporter` not found.

- [ ] **Step 3: Write the Reporter trait**

Create `src/reporter/mod.rs`:

```rust
pub mod json;
pub mod terminal;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::summary::Summary;

pub trait Reporter {
    fn render(
        &self,
        summary: &Summary,
        findings: &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String;
}
```

- [ ] **Step 4: Write the JSON reporter**

Create `src/reporter/json.rs`:

```rust
use serde_json::json;

use crate::analyzer::AnalysisSummary;
use crate::findings::Finding;
use crate::reporter::Reporter;
use crate::summary::Summary;

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn render(
        &self,
        summary: &Summary,
        findings: &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String {
        let output = json!({
            "summary": {
                "total_packets": summary.total_packets,
                "total_bytes": summary.total_bytes,
                "unique_hosts": summary.unique_hosts(),
                "protocols": summary.protocol_counts(),
                "services": summary.service_counts(),
            },
            "findings": findings,
            "analyzers": analyzer_summaries,
        });
        serde_json::to_string_pretty(&output).unwrap()
    }
}
```

- [ ] **Step 5: Write the terminal reporter**

Create `src/reporter/terminal.rs`:

```rust
use owo_colors::OwoColorize;

use crate::analyzer::AnalysisSummary;
use crate::findings::{Confidence, Finding, Verdict};
use crate::reporter::Reporter;
use crate::summary::Summary;

pub struct TerminalReporter {
    pub use_color: bool,
}

impl Reporter for TerminalReporter {
    fn render(
        &self,
        summary: &Summary,
        findings: &[Finding],
        analyzer_summaries: &[AnalysisSummary],
    ) -> String {
        let mut out = String::new();

        // Header
        out.push_str(&self.section("WIRERUST TRIAGE REPORT"));
        out.push_str(&format!(
            "  Packets: {}  Bytes: {}  Hosts: {}\n\n",
            summary.total_packets,
            summary.total_bytes,
            summary.unique_hosts().len(),
        ));

        // Protocol breakdown
        out.push_str(&self.section("PROTOCOLS"));
        for (proto, count) in summary.protocol_counts() {
            out.push_str(&format!("  {proto:?}: {count}\n"));
        }
        out.push('\n');

        // Services
        let services = summary.service_counts();
        if !services.is_empty() {
            out.push_str(&self.section("SERVICES"));
            for (svc, count) in services {
                out.push_str(&format!("  {svc}: {count}\n"));
            }
            out.push('\n');
        }

        // Findings
        if !findings.is_empty() {
            out.push_str(&self.section("FINDINGS"));
            for f in findings {
                let line = format!("[{}] {} ({}) - {}", f.category, f.verdict, f.confidence, f.summary);
                let colored = if self.use_color {
                    match f.verdict {
                        Verdict::Likely => {
                            match f.confidence {
                                Confidence::High => line.red().bold().to_string(),
                                _ => line.yellow().to_string(),
                            }
                        }
                        Verdict::Inconclusive => line.cyan().to_string(),
                        Verdict::Unlikely => line.dimmed().to_string(),
                    }
                } else {
                    line
                };
                out.push_str(&format!("  {colored}\n"));
                for ev in &f.evidence {
                    out.push_str(&format!("    > {ev}\n"));
                }
                if let Some(ref t) = f.mitre_technique {
                    out.push_str(&format!("    MITRE: {t}\n"));
                }
            }
            out.push('\n');
        }

        // Analyzer summaries
        for asummary in analyzer_summaries {
            out.push_str(&self.section(&format!("ANALYZER: {}", asummary.analyzer_name)));
            out.push_str(&format!("  Packets analyzed: {}\n", asummary.packets_analyzed));
            for (key, val) in &asummary.detail {
                out.push_str(&format!("  {key}: {val}\n"));
            }
            out.push('\n');
        }

        out
    }
}

impl TerminalReporter {
    fn section(&self, title: &str) -> String {
        if self.use_color {
            format!("{}\n{}\n", title.bold().underline(), "─".repeat(40))
        } else {
            format!("{title}\n{}\n", "─".repeat(40))
        }
    }
}
```

- [ ] **Step 6: Add reporter module to lib.rs**

Update `src/lib.rs`:

```rust
pub mod analyzer;
pub mod cli;
pub mod decoder;
pub mod findings;
pub mod reader;
pub mod reporter;
pub mod summary;
```

- [ ] **Step 7: Run tests**

Run: `cargo test --test reporter_tests`
Expected: 1 test PASS.

- [ ] **Step 8: Commit**

```bash
git add src/reporter/ src/lib.rs tests/reporter_tests.rs
git commit -m "feat: add Reporter trait with terminal and JSON implementations"
```

---

### Task 9: Wire Up main.rs — End-to-End Pipeline

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write an integration test**

Create `tests/integration_test.rs`:

```rust
use std::io::Cursor;

use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::Reporter;
use wirerust::summary::Summary;

fn minimal_pcap_with_tcp() -> Vec<u8> {
    let mut buf = Vec::new();
    // Global header
    buf.extend_from_slice(&0xa1b2c3d4u32.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes());
    buf.extend_from_slice(&4u16.to_le_bytes());
    buf.extend_from_slice(&0i32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&65535u32.to_le_bytes());
    buf.extend_from_slice(&1u32.to_le_bytes());

    let packet_data: Vec<u8> = vec![
        // Ethernet
        0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55,
        0x08, 0x00,
        // IPv4
        0x45, 0x00, 0x00, 0x28,
        0x00, 0x01, 0x00, 0x00,
        0x40, 0x06, 0x00, 0x00,
        0xc0, 0xa8, 0x01, 0x0a,
        0xc0, 0xa8, 0x01, 0x01,
        // TCP
        0xc0, 0x01, 0x00, 0x50,
        0x00, 0x00, 0x00, 0x01,
        0x00, 0x00, 0x00, 0x00,
        0x50, 0x02, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x00,
    ];

    let captured_len = packet_data.len() as u32;
    buf.extend_from_slice(&1000u32.to_le_bytes());
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&captured_len.to_le_bytes());
    buf.extend_from_slice(&captured_len.to_le_bytes());
    buf.extend_from_slice(&packet_data);
    buf
}

#[test]
fn test_full_pipeline() {
    let data = minimal_pcap_with_tcp();
    let source = PcapSource::from_pcap_reader(Cursor::new(data)).unwrap();

    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();

    for raw in &source.packets {
        if let Ok(parsed) = decode_packet(&raw.data) {
            summary.ingest(&parsed);
            if dns_analyzer.can_decode(&parsed) {
                let findings = dns_analyzer.analyze(&parsed);
                all_findings.extend(findings);
            }
        }
    }

    assert_eq!(summary.total_packets, 1);

    let reporter = JsonReporter;
    let output = reporter.render(&summary, &all_findings, &[dns_analyzer.summarize()]);
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["summary"]["total_packets"], 1);
}
```

- [ ] **Step 2: Run integration test to verify it passes**

Run: `cargo test --test integration_test`
Expected: PASS — all pieces wired together.

- [ ] **Step 3: Write main.rs with full CLI dispatch**

Update `src/main.rs`:

```rust
use std::path::Path;

use anyhow::{Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};

mod cli;

use wirerust::analyzer::dns::DnsAnalyzer;
use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::decoder::decode_packet;
use wirerust::reader::PcapSource;
use wirerust::reporter::json::JsonReporter;
use wirerust::reporter::terminal::TerminalReporter;
use wirerust::reporter::Reporter;
use wirerust::summary::Summary;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let use_color = !cli.no_color && std::env::var("NO_COLOR").is_err();

    match &cli.command {
        cli::Commands::Analyze { targets, dns, all, .. } => {
            run_analyze(targets, *dns || *all, use_color, &cli)?;
        }
        cli::Commands::Summary { targets, .. } => {
            run_summary(targets, use_color, &cli)?;
        }
    }

    Ok(())
}

fn run_analyze(
    targets: &[std::path::PathBuf],
    enable_dns: bool,
    use_color: bool,
    cli: &cli::Cli,
) -> Result<()> {
    let mut summary = Summary::new();
    let mut dns_analyzer = DnsAnalyzer::new();
    let mut all_findings = Vec::new();

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)
                .with_context(|| format!("Failed to read {}", path.display()))?;

            let pb = ProgressBar::new(source.packets.len() as u64);
            pb.set_style(
                ProgressStyle::with_template("[{elapsed_precise}] {bar:40} {pos}/{len} packets")?
            );

            for raw in &source.packets {
                if let Ok(parsed) = decode_packet(&raw.data) {
                    summary.ingest(&parsed);
                    if enable_dns && dns_analyzer.can_decode(&parsed) {
                        let findings = dns_analyzer.analyze(&parsed);
                        all_findings.extend(findings);
                    }
                }
                pb.inc(1);
            }
            pb.finish_and_clear();
        }
    }

    let analyzer_summaries = if enable_dns {
        vec![dns_analyzer.summarize()]
    } else {
        vec![]
    };

    let output = match cli.output_format {
        Some(cli::OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
        _ => {
            let reporter = TerminalReporter { use_color };
            reporter.render(&summary, &all_findings, &analyzer_summaries)
        }
    };

    println!("{output}");
    Ok(())
}

fn run_summary(
    targets: &[std::path::PathBuf],
    use_color: bool,
    cli: &cli::Cli,
) -> Result<()> {
    let mut summary = Summary::new();

    for target in targets {
        let pcap_files = resolve_targets(target)?;
        for path in &pcap_files {
            let source = PcapSource::from_file(path)?;
            for raw in &source.packets {
                if let Ok(parsed) = decode_packet(&raw.data) {
                    summary.ingest(&parsed);
                }
            }
        }
    }

    let output = match cli.output_format {
        Some(cli::OutputFormat::Json) => {
            let reporter = JsonReporter;
            reporter.render(&summary, &[], &[])
        }
        _ => {
            let reporter = TerminalReporter { use_color };
            reporter.render(&summary, &[], &[])
        }
    };

    println!("{output}");
    Ok(())
}

fn resolve_targets(target: &Path) -> Result<Vec<std::path::PathBuf>> {
    if target.is_file() {
        return Ok(vec![target.to_path_buf()]);
    }
    if target.is_dir() {
        let mut files = Vec::new();
        for entry in std::fs::read_dir(target)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "pcap" || ext == "pcapng" {
                        files.push(path);
                    }
                }
            }
        }
        files.sort();
        return Ok(files);
    }
    anyhow::bail!("Target not found: {}", target.display());
}
```

- [ ] **Step 4: Verify it builds and runs**

Run: `cargo build`
Expected: Compiles.

Run: `cargo run -- --help`
Expected: Shows help text with `analyze` and `summary` subcommands.

- [ ] **Step 5: Run all tests**

Run: `cargo test`
Expected: All tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs tests/integration_test.rs
git commit -m "feat: wire up end-to-end pipeline with CLI dispatch"
```

---

### Task 10: Push to GitHub

- [ ] **Step 1: Run full test suite one final time**

Run: `cargo test`
Expected: All tests pass.

Run: `cargo clippy -- -W clippy::all`
Expected: No errors (warnings acceptable for now).

- [ ] **Step 2: Push**

```bash
git push origin master
```
