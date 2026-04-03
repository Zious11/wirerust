# wirerust

Fast PCAP forensics and network triage CLI tool written in Rust.

Inspired by [pcapper](https://github.com/SackOfHacks/pcapper) — reimagined for speed: zero-copy packet parsing, single static binary, and designed for incident response jumpkits.

## Features

- **One-pass triage** — hosts, services, protocols, and threat signals from pcap/pcapng files
- **Protocol analysis** — DNS traffic analysis with extensible analyzer framework
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
  -h, --help                 Print help
  -V, --version              Print version
```

### Analyze flags

```
--threats    Run threat detection
--dns        Analyze DNS traffic
--http       Analyze HTTP traffic (coming soon)
--tls        Analyze TLS handshakes (coming soon)
--beacon     Detect C2 beaconing patterns (coming soon)
-a, --all    Run all analyzers
-f, --filter BPF filter expression
```

## Architecture

```
PCAP file → Reader → Decoder → Analyzers → Reporter
                        ↓           ↓
                    ParsedPacket  Findings
                        ↓
                     Summary
```

| Component | Crate | Purpose |
|-----------|-------|---------|
| Reader | `pcap-file` | Parse pcap/pcapng files |
| Decoder | `etherparse` | Zero-copy packet parsing |
| CLI | `clap` | Argument parsing |
| Output | `owo-colors`, `serde_json` | Terminal + JSON |

## Extending

Add a new protocol analyzer by implementing the `ProtocolAnalyzer` trait:

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

- HTTP and TLS analyzers
- C2 beaconing detection
- CSV and SQLite export
- MITRE ATT&CK mapping
- Parallel file processing
- ICS/OT protocols (Modbus, DNP3)

## License

[MIT](LICENSE)
