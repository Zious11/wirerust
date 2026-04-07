# wirerust

Fast PCAP forensics and network triage CLI tool written in Rust.

Inspired by [pcapper](https://github.com/SackOfHacks/pcapper) — reimagined for speed: zero-copy packet parsing, single static binary, and designed for incident response jumpkits.

## Features

- **One-pass triage** — hosts, services, protocols, and threat signals from pcap files
- **Protocol analysis** — DNS and HTTP traffic analysis with extensible analyzer framework
- **HTTP forensics** — stream-level HTTP/1.x parsing with detection for path traversal, web shells, unusual methods, and anomalies
- **TCP stream reassembly** — forensic-grade reassembly engine with first-wins overlap policy, configurable depth/memory limits
- **Multi-link-type support** — Ethernet, Raw IP, IPv4, IPv6, and Linux Cooked (SLL) pcap formats
- **Threat detection** — finding system with verdict/confidence scoring and MITRE ATT&CK mapping
- **Multiple outputs** — colored terminal, JSON export
- **Fast** — Rust + etherparse zero-copy parsing, built for multi-GB captures

## Install

```bash
cargo install --path .
```

Or build from source:

```bash
git clone https://github.com/Zious11/wirerust.git
cd wirerust
cargo build --release
# Binary at target/release/wirerust
```

## Usage

### Analyze a PCAP file

```bash
# Quick triage with DNS analysis
wirerust analyze capture.pcap --dns

# HTTP analysis (auto-enables TCP reassembly)
wirerust analyze capture.pcap --http

# Run all analyzers
wirerust analyze capture.pcap --all

# JSON output
wirerust analyze capture.pcap --all --output-format json

# Multiple files or directories
wirerust analyze *.pcap /path/to/pcaps/ --all
```

### Generate a summary

```bash
wirerust summary capture.pcap
wirerust summary /path/to/pcaps/ --output-format json
```

### Options

```
wirerust [OPTIONS] <COMMAND>

Commands:
  analyze   Analyze PCAP files for threats and anomalies
  summary   Generate a triage summary of PCAP files

Options:
  -v, --verbose              Enable verbose output
      --no-color             Disable colored output
      --output-format <FMT>  Output format: json, csv
      --reassemble           Force TCP stream reassembly on
      --no-reassemble        Force TCP stream reassembly off
      --reassembly-depth N   Per-direction stream limit in MB (default: 10)
      --reassembly-memcap N  Global memory cap in MB (default: 1024)
  -h, --help                 Print help
  -V, --version              Print version
```

### Analyze flags

```
--threats    Run threat detection
--dns        Analyze DNS traffic
--http       Analyze HTTP traffic (auto-enables reassembly)
--tls        Analyze TLS handshakes (coming soon)
--beacon     Detect C2 beaconing patterns (coming soon)
-a, --all    Run all analyzers
-f, --filter BPF filter expression
```

## Architecture

```
PCAP file → Reader → Decoder → Analyzers → Reporter
               ↓         ↓          ↓
           DataLink  ParsedPacket  Findings
                         ↓
                   Reassembly Engine → StreamAnalyzers (HTTP)
                         ↓
                      Summary
```

| Component | Crate | Purpose |
|-----------|-------|---------|
| Reader | `pcap-file` | Parse pcap files (5 link types) |
| Decoder | `etherparse` | Zero-copy packet parsing |
| HTTP Parser | `httparse` | HTTP/1.x request/response parsing |
| Reassembly | (built-in) | TCP stream reassembly engine |
| CLI | `clap` | Argument parsing |
| Output | `owo-colors`, `serde_json` | Terminal + JSON |

## Supported Link Types

| Type | ID | Status |
|------|-----|--------|
| Ethernet | 1 | Supported |
| Raw IP | 101 | Supported |
| Linux Cooked (SLL) | 113 | Supported |
| IPv4 | 228 | Supported |
| IPv6 | 229 | Supported |
| pcapng | — | Not yet supported |

## Extending

Add a new protocol analyzer by implementing the `ProtocolAnalyzer` trait (per-packet) or `StreamAnalyzer` trait (reassembled streams):

```rust
use wirerust::analyzer::ProtocolAnalyzer;
use wirerust::decoder::ParsedPacket;
use wirerust::findings::Finding;

impl ProtocolAnalyzer for MyAnalyzer {
    fn name(&self) -> &'static str { "MyProtocol" }
    fn can_decode(&self, packet: &ParsedPacket) -> bool { /* port check */ }
    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding> { /* logic */ }
    fn summarize(&self) -> AnalysisSummary { /* stats */ }
}
```

## Roadmap

See [open issues](https://github.com/Zious11/wirerust/issues) for planned features:

- TLS analyzer (JA3/JA4 fingerprinting)
- C2 beaconing detection
- CSV and SQLite export
- MITRE ATT&CK mapping
- Parallel file processing
- ICS/OT protocols (Modbus, DNP3)
- pcapng format support

## License

[MIT](LICENSE)
