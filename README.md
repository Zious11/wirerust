# wirerust

Fast PCAP forensics and network triage CLI tool written in Rust.

Inspired by [pcapper](https://github.com/SackOfHacks/pcapper) — reimagined for speed: zero-copy packet parsing, single static binary, and designed for incident response jumpkits.

## Features

- **One-pass triage** — hosts, services, protocols, and threat signals from pcap files
- **Protocol analysis** — DNS, HTTP, TLS, Modbus, DNP3, and ARP traffic analysis with extensible analyzer framework
- **HTTP forensics** — stream-level HTTP/1.x parsing with detection for path traversal, web shells, unusual methods, and anomalies
- **TLS forensics** — ClientHello/ServerHello parsing, SNI extraction, JA3/JA3S fingerprinting, weak cipher and deprecated SSL 2.0/3.0 detection
- **Modbus TCP forensics** — ICS/OT threat detection on port 502; parses MBAP header and function codes; detects 7 MITRE ATT&CK for ICS techniques (T1692.001, T0836, T0835, T0831, T0806, T0814, T0888); configurable write-burst and sustained-rate thresholds; enabled via `--modbus`
- **DNP3 TCP forensics** — ICS/OT threat detection on port 20000; parses IEEE Std 1815-2012 data-link frames; detects MITRE ATT&CK for ICS techniques T1692.001, T1691.001, T0827, T0814, and T0836; anomaly detection for broadcast control, unsolicited responses, and malformed frames; enabled via `--dnp3`
- **ARP security forensics** — link-layer and OT network threat detection; detects ARP spoofing / cache poisoning, gratuitous ARP anomalies, ARP storms, malformed ARP frames, and L2/L3 sender-MAC mismatch; MITRE attribution to T0830 and T1557.002; enabled via `--arp`
- **TCP stream reassembly** — forensic-grade reassembly engine with first-wins overlap policy, configurable depth/memory/window limits
- **Multi-link-type support** — Ethernet, Raw IP, IPv4, IPv6, and Linux Cooked (SLL) pcap formats
- **Threat detection** — finding system with verdict/confidence scoring and MITRE ATT&CK mapping
- **Multiple outputs** — colored terminal, JSON export, CSV export
- **Fast** — Rust + etherparse zero-copy L2–L4 parsing with single-pass decoding; the full capture is loaded into memory, so available RAM determines the practical file-size limit

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
wirerust summary capture.pcap --hosts       # include per-host IP breakdown
wirerust summary /path/to/pcaps/ --output-format json
```

The `--hosts` flag expands the `Hosts: N` count in the terminal reporter into an itemized
per-host breakdown of source and destination IPs. The JSON reporter always emits the full
`unique_hosts` array regardless of this flag.

### Options

```
wirerust [OPTIONS] <COMMAND>

Commands:
  analyze   Analyze PCAP files for threats and anomalies
  summary   Generate a triage summary of PCAP files

Options:
      --no-color                           Disable colored output
      --output-format <FMT>                Output format: json, csv
      --reassemble                         Force TCP stream reassembly on
      --no-reassemble                      Force TCP stream reassembly off
      --reassembly-depth N                 Per-direction stream limit in MB (default: 10)
      --reassembly-memcap N                Global memory cap in MB (default: 1024)
      --overlap-threshold N                Overlapping-segment anomaly threshold per flow direction (default: 50; range 0–255)
      --small-segment-threshold N          Consecutive small-segment run length threshold (default: 100; range 0–2048)
      --small-segment-max-bytes N          Payload-size cutoff in bytes below which a segment is "small" (default: 16; range 0–2048; 0 disables)
      --small-segment-ignore-ports <LIST>  Comma-separated ports exempt from small-segment detection (default: 23,513)
      --out-of-window-threshold N          Out-of-window segment anomaly threshold per flow direction (default: 100)
      --flow-timeout N                     Idle flow expiry in seconds (default: 300; minimum: 1)
  -h, --help                               Print help
  -V, --version                            Print version
```

### Analyze flags

```
--dns                                  Analyze DNS traffic
--http                                 Analyze HTTP traffic (auto-enables reassembly)
--tls                                  Analyze TLS handshakes (SNI, JA3/JA3S, weak ciphers, deprecated SSL)
--modbus                               Analyze Modbus TCP traffic (port 502, default-off; included in --all)
--modbus-write-burst-threshold N       Burst detection threshold in writes/1s window (default: 20)
--modbus-write-sustained-threshold N   Sustained-rate threshold in writes/s over >=2s (default: 10)
--dnp3                                 Analyze DNP3 TCP traffic (port 20000, default-off; included in --all)
--dnp3-direct-operate-threshold N      Direct-operate burst threshold per flow (default: 10)
--arp                                  Analyze ARP traffic (spoofing, GARP, storms, malformed, MAC mismatch; default-off; included in --all)
--arp-spoof-threshold N                MAC-rebind escalation threshold per IP within 60s window (default: 3)
--arp-storm-rate N                     ARP storm frames/second per source MAC threshold (default: 50)
--no-collapse                          Disable collapsing of repeated findings in both flat and grouped (--mitre) terminal output. By default, collapse is enabled in both modes. When --mitre is used, collapse groups identical findings within each MITRE tactic bucket with a (xN) count suffix. Pass --no-collapse to restore one-line-per-finding output in both modes. Has no effect on --output json or --output csv.
--mitre                                Group findings by MITRE ATT&CK tactic and show technique names; collapses identical findings within each tactic bucket with a (xN) count suffix by default (pass --no-collapse to disable)
-a, --all                              Run all analyzers
```

## Architecture

```
PCAP file → Reader → Decoder → Analyzers → Reporter
               ↓         ↓          ↓
           DataLink  ParsedPacket  Findings
                         ↓         ↓
                         │    ArpAnalyzer (packet-level)
                         ↓
                   Reassembly Engine → StreamDispatcher → StreamAnalyzers (HTTP, TLS, Modbus, DNP3)
                         ↓
                      Summary
```

| Component | Crate / Module | Purpose |
|-----------|----------------|---------|
| Reader | `pcap-file` | Parse pcap files (5 link types) |
| Decoder | `etherparse` | Zero-copy packet parsing (IP + ARP frames) |
| HTTP Parser | `httparse` | HTTP/1.x request/response parsing |
| TLS Parser | `tls-parser` | TLS handshake parsing, JA3/JA3S |
| Modbus Analyzer | (built-in) | Modbus TCP ICS/OT threat detection (port 502) |
| DNP3 Analyzer | (built-in) | DNP3 TCP ICS/OT threat detection (port 20000) |
| ARP Analyzer | (built-in) | Link-layer ARP spoofing and anomaly detection |
| Reassembly | (built-in) | TCP stream reassembly engine |
| CLI | `clap` | Argument parsing |
| Output | `owo-colors`, `serde_json`, `csv` | Terminal + JSON + CSV |

## Supported Protocol Analyzers

| Protocol | Port | Flag | Default | MITRE ATT&CK Techniques Detected |
|----------|------|------|---------|-----------------------------------|
| DNS | 53/UDP | `--dns` | off | — |
| HTTP/1.x | 80, 8080 | `--http` | off | T1071.001, T1505.003, T1083 |
| TLS | 443, 8443 | `--tls` | off | T1040, T1573, T1036, T1027 |
| Modbus TCP | 502 | `--modbus` | off | T1692.001, T0836, T0835, T0831, T0806, T0814, T0888 |
| DNP3 TCP | 20000 | `--dnp3` | off | T1692.001, T1691.001, T0827, T0814, T0836 |
| ARP | link-layer | `--arp` | off | T0830, T1557.002 |

### DNP3 TCP Analyzer

The DNP3 analyzer (`--dnp3`) processes TCP streams on port 20000 per IEEE Std 1815-2012. It is
dispatched as Rule 6 in the stream dispatcher — after content-signature rules (TLS, HTTP) and
port-based rules for TLS, HTTP, and Modbus — so it never misclassifies TLS or HTTP traffic.

Detections emitted:

| Detection | Technique | Tactic | Trigger |
|-----------|-----------|--------|---------|
| Direct-operate burst | T1692.001 | Impair Process Control | Control-class FCs exceed `--dnp3-direct-operate-threshold` (default 10) within the 60s detection window |
| Unexpected master source | T1692.001 | Impair Process Control | Control-class FC from a source address not in the established master set for this flow |
| Broadcast control command | T1692.001 | Impair Process Control | Control-class FC addressed to a DNP3 broadcast destination |
| Restart command (cold/warm) | T0814 | Inhibit Response Function | COLD_RESTART (FC 0x0D) or WARM_RESTART (FC 0x0E) observed; verdict Likely / confidence High |
| DISABLE_UNSOLICITED command | T0814 | Inhibit Response Function | FC 0x15 observed — alarm suppression / event-blinding primitive; verdict Likely / confidence Medium |
| ENABLE_UNSOLICITED command | T0814 | Inhibit Response Function | FC 0x14 observed — unsolicited reporting control; verdict Possible / confidence Low |
| WRITE command | T0836 | Impair Process Control | WRITE FC (0x02) observed |
| Block command (unanswered) | T1691.001 | Inhibit Response Function | Control-class requests with no RESPONSE within 10s, >= 3 events in 300s window |
| Loss of Control | T0827 | Impact (ICS) | Combined restart + block-command events >= 3 in 300s window |
| Malformed frame anomaly | T0814 | Inhibit Response Function | >= 3 parse-invalid frames within the 300s correlation window; verdict Possible / confidence Low |
| Unsolicited response anomaly | T0814 | Inhibit Response Function | UNSOLICITED_RESPONSE (FC 0x82) on a flow where ENABLE_UNSOLICITED was never sent; verdict Possible / confidence Low |

CLI flags:
- `--dnp3` — enable DNP3 TCP analysis (also included in `-a`/`--all`; default-off)
- `--dnp3-direct-operate-threshold N` — direct-operate burst threshold per flow, default 10

### ARP Security Analyzer

The ARP analyzer (`--arp`) processes ARP frames at the packet level (no TCP reassembly
required). It maintains a bounded IP→MAC binding table (LRU cap: 65 536 entries) and a
bounded per-source-MAC rate counter table (LRU cap: 4 096 entries).

Detections emitted:

| Detection | Technique | Tactic | Trigger |
|-----------|-----------|--------|---------|
| ARP spoofing (D1) | T0830, T1557.002 | Adversary-in-the-Middle | MAC rebind for an existing IP→MAC binding; escalates from MEDIUM to HIGH after `--arp-spoof-threshold` rebinds within 60s |
| Gratuitous ARP (D2) | — | Anomaly | Unsolicited GARP frame observed; escalates to MEDIUM and co-emits a D1 finding when the announced MAC conflicts with an established binding |
| ARP storm (D3) | — [^1] | Anomaly | Source MAC ARP rate exceeds `--arp-storm-rate` frames/second |
| Malformed ARP frame (D11) | — | Anomaly | Frame fails both strict and lax/snaplen-truncated ARP parse |
| L2/L3 sender-MAC mismatch (D12) | T0830, T1557.002 | Adversary-in-the-Middle | Ethernet source MAC differs from ARP sender hardware address |

CLI flags:
- `--arp` — enable ARP analysis (also included in `-a`/`--all`; default-off)
- `--arp-spoof-threshold N` — MAC-rebind escalation threshold within the 60s window (default: 3)
- `--arp-storm-rate N` — frames/second per source MAC above which a storm finding is emitted (default: 50)

[^1]: D3 storm findings emit `mitre_techniques: []` (no technique attributed). T0814 attribution
is pending validation per DF-VALIDATION-001 / BC-2.16.008 Invariant 3.

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

### Test naming convention

Tests use **prose-style names that state the asserted behavior**, not just
the function under test. The name should read as a claim that is true when
the test passes:

```rust
// good — states the behavior being asserted
fn test_detect_empty_host_header() { ... }
fn mitre_grouping_emits_tactic_headers_in_canonical_order() { ... }
fn test_low_overlap_threshold_fires_earlier() { ... }

// avoid — names the symbol, not the behavior
fn test_host_header() { ... }
fn test_mitre() { ... }
```

Guidelines:

- Prefer `<subject>_<verb>_<expected outcome>` — e.g.
  `test_drop_without_finalize_does_not_panic`.
- A `test_` prefix is common but not required; an integration test whose
  name already reads as a sentence (`mitre_grouping_emits_…`) may omit it.
- One behavior per test. If the name needs an "and", it is probably two
  tests.
- When a test exists to guard a specific lesson or regression, reference it
  in the test body comment (e.g. `// LESSON-P2.05: …`), not in the name.

This is a documentation of existing practice — the test suite already
follows it; new tests should match.

## Roadmap

See [open issues](https://github.com/Zious11/wirerust/issues) for planned features:

- C2 beaconing detection
- SQLite export
- Parallel file processing
- pcapng format support

## License

[MIT](LICENSE)
