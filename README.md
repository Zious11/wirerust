# wirerust

Fast PCAP forensics and network triage CLI tool written in Rust.

Inspired by [pcapper](https://github.com/SackOfHacks/pcapper) — reimagined for speed: zero-copy packet parsing, single static binary, and designed for incident response jumpkits.

## Features

- **One-pass triage** — hosts, services, protocols, and threat signals from pcap files
- **Protocol analysis** — DNS, HTTP, TLS, Modbus, DNP3, ARP, EtherNet/IP (CIP), and IEC 60870-5-104 traffic analysis with extensible analyzer framework
- **HTTP forensics** — stream-level HTTP/1.x parsing with detection for path traversal, web shells, unusual methods, and anomalies; `dropped_map_entries` JSON counter surfaces silently-dropped entries when per-analyzer counter maps reach their cap
- **TLS forensics** — ClientHello/ServerHello parsing, SNI extraction, JA3/JA3S fingerprinting, weak cipher and deprecated SSL 2.0/3.0 detection; multi-record handshake-message reassembly (SNI/JA3/JA3S fragmentation-evasion closed; per-direction carry buffer with `buffer_saturation_drops` overflow telemetry); `dropped_map_entries` JSON counter surfaces silently-dropped entries when per-analyzer counter maps reach their cap
- **Modbus TCP forensics** — ICS/OT threat detection on port 502; parses MBAP header and function codes; detects 7 MITRE ATT&CK for ICS techniques (T1692.001, T0836, T0835, T0831, T0806, T0814, T0888); configurable write-burst and sustained-rate thresholds; `dropped_transactions` JSON counter surfaces silently-dropped transactions when the per-flow transaction map reaches its cap; enabled via `--modbus`
- **DNP3 TCP forensics** — ICS/OT threat detection on port 20000; parses IEEE Std 1815-2012 data-link frames; detects MITRE ATT&CK for ICS techniques T1692.001, T1691.001, T0827, T0814, and T0836; anomaly detection for broadcast control, unsolicited responses, and malformed frames; `dropped_findings`, `master_addrs_dropped`, and `pending_requests_evicted` JSON counters surface cap-related telemetry; enabled via `--dnp3`
- **ARP security forensics** — link-layer and OT network threat detection; detects ARP spoofing / cache poisoning, gratuitous ARP anomalies, ARP storms, malformed ARP frames, and L2/L3 sender-MAC mismatch; MITRE attribution to T0830 and T1557.002; enabled via `--arp`
- **EtherNet/IP CIP forensics** — ICS/OT threat detection on port 44818; parses ODVA EtherNet/IP encapsulation header (24-byte, little-endian) and Common Packet Format item walk with CIP service extraction; detects 6 MITRE ATT&CK for ICS techniques (T0836 write-burst, T0846 remote system discovery, T0814 malformed-frame/crash-probe, T0888 error-burst/identity-read, T0858 controller stop, T0816 device reset); configurable write-burst and error-burst thresholds; emits `enip_summary` JSON key; references ADR-010; enabled via `--enip`
- **IEC 60870-5-104 forensics** — ICS/OT threat detection on port 2404; parses APCI header and ASDU structure; detects MITRE ATT&CK for ICS techniques T1692.001, T0836, T0827, T0881, T0814; U-frame session state machine with STARTDT/STOPDT/TESTFR tracking; N(S) sequence desync detection; `dropped_findings` and `flows_analyzed` JSON counters; enabled via `--iec104`
- **TCP stream reassembly** — forensic-grade reassembly engine with first-wins overlap policy, configurable depth/memory/window limits
- **Multi-link-type support** — Ethernet, Raw IP, IPv4, IPv6, and Linux Cooked (SLL) in both
  classic pcap and pcapng captures
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

# JSON output to terminal
wirerust analyze capture.pcap --all --output-format json

# Write JSON findings to a file (--json with an optional path)
wirerust analyze capture.pcap --all --json findings.json

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
  analyze    Analyze PCAP files for threats and anomalies
  summary    Generate a triage summary of PCAP files
  protocols  List the protocol coverage catalog
  help       Print this message or the help of the given subcommand(s)

Options:
      --no-color                           Disable colored output
      --output-format <FMT>                Output format: json, csv
      --json [<FILE>]                      Write JSON output to FILE (or stdout if no path given); mutually exclusive with --csv
      --csv [<FILE>]                       Write CSV output to FILE (or stdout if no path given); emits findings table only
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
--arp-storm-rate N                     ARP storm rate (frames/second per source MAC at or above which a storm finding is emitted; default: 50; engineering default — not derived from any external standard; see Known Limitations)
--enip                                 Analyze EtherNet/IP TCP traffic (port 44818, default-off; included in --all)
--enip-write-burst-threshold N         CIP write-burst threshold in SetAttribute requests/1s window (default: 50)
--enip-error-burst-threshold N         CIP error-burst threshold in error responses/10s window (default: 5)
--iec104                               Analyze IEC 60870-5-104 TCP traffic (port 2404, default-off; included in --all)
--no-collapse                          Disable collapsing of repeated findings in both flat and grouped (--mitre) terminal output. By default, collapse is enabled in both modes. When --mitre is used, collapse groups identical findings within each MITRE tactic bucket with a (xN) count suffix. Pass --no-collapse to restore one-line-per-finding output in both modes. Has no effect on --json, --csv, or --output-format json|csv output.
--mitre                                Group findings by MITRE ATT&CK tactic and show technique names; collapses identical findings within each tactic bucket with a (xN) count suffix by default (pass --no-collapse to disable)
-a, --all                              Run all analyzers
--coverage-gaps                        Enable per-port unclassified traffic gap detection (opt-in; excluded from --all)
```

### List protocol coverage

```bash
# Show all known protocols (supported and planned)
wirerust protocols

# Show only protocols wirerust actively dissects
wirerust protocols --supported

# Show only protocols not yet dissected
wirerust protocols --unsupported

# Machine-readable output (uses the global --json flag)
wirerust protocols --json
```

The `protocols` subcommand prints a filterable table of all known ICS/IT protocols and their
coverage status. Filter flags:

- `--all` — show all protocols (default when no filter flag is given)
- `--supported` — show only protocols that wirerust actively dissects
- `--unsupported` — show only protocols that wirerust does not yet dissect

Combine with the global `--json` flag for machine-readable output.

### Coverage gap detection

Pass `--coverage-gaps` to `wirerust analyze` to include a `coverage_gaps` object in JSON
output (rendered as the `CoverageGapsSummary` section in terminal output). Each observed
protocol port is classified with one of three states: `known-supported` (an active analyzer
handles that port), `known-unsupported` (the catalog has an entry for the protocol but no
dissector exists), or `unknown` (no catalog entry matched the port). This flag is deliberately
excluded from `--all` to avoid silent behavioral drift for downstream JSON consumers
(ADR-012, Decision 8).

```bash
wirerust analyze capture.pcap --all --coverage-gaps
```

## Architecture

```
PCAP file → Reader → Decoder → Analyzers → Reporter
               ↓         ↓          ↓
           DataLink  ParsedPacket  Findings
                         ↓         ↓
                         │    ArpAnalyzer (packet-level)
                         ↓
                   Reassembly Engine → StreamDispatcher → StreamAnalyzers (HTTP, TLS, Modbus, DNP3, EtherNet/IP, IEC-104)
                         ↓
                      Summary
```

| Component | Crate / Module | Purpose |
|-----------|----------------|---------|
| Reader | `pcap-file` | Parse classic pcap and pcapng files (5 link types; both formats) |
| Decoder | `etherparse` | Zero-copy packet parsing (IP + ARP frames) |
| HTTP Parser | `httparse` | HTTP/1.x request/response parsing |
| TLS Parser | `tls-parser` | TLS handshake parsing, JA3/JA3S |
| Modbus Analyzer | (built-in) | Modbus TCP ICS/OT threat detection (port 502) |
| DNP3 Analyzer | (built-in) | DNP3 TCP ICS/OT threat detection (port 20000) |
| ENIP/CIP Analyzer | (built-in) | EtherNet/IP TCP ICS/OT threat detection (port 44818) |
| IEC-104 Analyzer | (built-in) | IEC 60870-5-104 TCP ICS/OT threat detection (port 2404) |
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
| EtherNet/IP TCP | 44818 | `--enip` | off | T0836, T0846, T0814, T0888, T0858, T0816 |
| IEC 60870-5-104 TCP | 2404 | `--iec104` | off | T1692.001, T0836, T0827, T0881, T0814 |
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

JSON output counters (present in the DNP3 analyzer's `detail` object in JSON output, at
`analyzers[i].detail`, when using `--json` / `--output-format json`):
- `dropped_findings` — count of findings silently dropped after the `MAX_FINDINGS = 10 000`
  per-analyzer cap was reached; a non-zero value means some detection events were not emitted
- `master_addrs_dropped` — count of new master addresses silently ignored after the
  `MAX_MASTER_ADDRS = 64` per-flow cap was full
- `pending_requests_evicted` — count of pending control-request entries evicted by LRU when
  the `MAX_PENDING_REQUESTS = 256` per-flow table was full; an evicted entry cannot match a
  RESPONSE, so a T1691.001 block-command finding may be suppressed

### ARP Security Analyzer

The ARP analyzer (`--arp`) processes ARP frames at the packet level (no TCP reassembly
required). It maintains a bounded IP→MAC binding table (LRU cap: 65 536 entries) and a
bounded per-source-MAC rate counter table (LRU cap: 4 096 entries).

Detections emitted:

| Detection | Technique | Tactic | Trigger |
|-----------|-----------|--------|---------|
| ARP spoofing (D1) | T0830, T1557.002 | Collection (ICS), Credential Access | MAC rebind for an existing IP→MAC binding; escalates from MEDIUM to HIGH after `--arp-spoof-threshold` rebinds within 60s |
| Gratuitous ARP (D2) | — | Anomaly | Unsolicited GARP frame observed; escalates to MEDIUM and co-emits a D1 finding when the announced MAC conflicts with an established binding |
| ARP storm (D3) | — [^1] | Anomaly | Source MAC ARP rate reaches `--arp-storm-rate` frames/second or more |
| Malformed ARP frame (D11) | — | Anomaly | Frame fails both strict and lax/snaplen-truncated ARP parse |
| L2/L3 sender-MAC mismatch (D12) | T0830, T1557.002 | Collection (ICS), Credential Access | Ethernet source MAC differs from ARP sender hardware address |

CLI flags:
- `--arp` — enable ARP analysis (also included in `-a`/`--all`; default-off)
- `--arp-spoof-threshold N` — MAC-rebind escalation threshold within the 60s window (default: 3)
- `--arp-storm-rate N` — frames/second per source MAC at or above which a storm finding is emitted (default: 50)

JSON output counters (present in the ARP analyzer's `detail` object in JSON output, at
`analyzers[i].detail`, when using `--json` / `--output-format json`):
- `bindings_evicted` — count of IP→MAC binding-table LRU evictions (table cap: 65 536 entries); a
  non-zero value means the oldest bindings were dropped to stay within the memory bound
- `storm_counters_evicted` — count of per-MAC storm-counter-table LRU evictions (table cap:
  4 096 entries); a non-zero value means the oldest MAC rate-counters were dropped

[^1]: D3 storm findings emit `mitre_techniques: []` (no technique attributed). T0814 attribution
is pending validation per DF-VALIDATION-001 / BC-2.16.008 Invariant 3.

### EtherNet/IP CIP Analyzer

The EtherNet/IP analyzer (`--enip`) processes TCP streams on port 44818. It parses the ODVA
EtherNet/IP encapsulation header (24 bytes, little-endian), walks the Common Packet Format (CPF)
item list, and extracts CIP service codes from Unconnected Data Items (type 0x00B2). The analyzer
is dispatched as Rule 7 in the stream dispatcher — after content-signature rules (TLS, HTTP) and
port-based rules for TLS, HTTP, Modbus, and DNP3 — so TLS or HTTP traffic on port 44818 routes
correctly before reaching this rule. Architecture: ADR-010.

Detections emitted:

| Detection | Technique | Tactic | Trigger |
|-----------|-----------|--------|---------|
| ListIdentity broadcast | T0846 | Discovery | ENIP command 0x0063 (ListIdentity); one finding per flow |
| CIP Identity attribute read | T0888 | Discovery | GetAttributeSingle/All/List to Identity Object (class 0x01) |
| CIP error-response burst | T0888 | Discovery | CIP error responses exceed `--enip-error-burst-threshold` (default 5) within 10s window; one finding per window |
| CIP write-class burst | T0836 | Impair Process Control | SetAttribute* requests exceed `--enip-write-burst-threshold` (default 50) within 1s window; one finding per window |
| ENIP structural anomaly | T0814 | Inhibit Response Function | >= 3 malformed/invalid ENIP frames in 300s window; verdict Possible / confidence Low |
| CIP Stop service | T0858 | Execution | CIP service 0x07 (Stop) request; fires per occurrence |
| CIP Reset service | T0816 | Inhibit Response Function | CIP service 0x05 (Reset) request; fires per occurrence |
| CIP ForwardOpen/ForwardClose | — | Anomaly | CIP connection-lifecycle events (0x54 / 0x5B / 0x4E); `mitre_techniques: []` — no dedicated ICS technique |

The JSON output includes an `enip_summary` object with seven canonical fields:
`command_distribution`, `total_pdu_count`, `parse_errors`, `write_count`,
`error_count`, `flows_analyzed`, and `dropped_findings`.

CLI flags:
- `--enip` — enable EtherNet/IP TCP analysis (also included in `-a`/`--all`; default-off)
- `--enip-write-burst-threshold N` — CIP write-burst threshold in SetAttribute requests per 1s
  window (default: 50); strict `>` semantics — fires on the (N+1)th write
- `--enip-error-burst-threshold N` — CIP error-burst threshold in error responses per 10s window
  (default: 5); strict `>` semantics — fires on the (N+1)th error

### IEC 60870-5-104 Analyzer

The IEC-104 analyzer (`--iec104`) processes TCP streams on port 2404. It parses the 6-byte
APCI header and ASDU structure; classifies I-format, S-format, and U-format frames; and
maintains a per-flow U-frame session state machine tracking STARTDT/STOPDT/TESTFR command
sequences. The analyzer is dispatched as Rule 8 in the stream dispatcher — after
content-signature rules (TLS, HTTP) and port-based rules for TLS, HTTP, Modbus, DNP3, and
EtherNet/IP — so TLS or HTTP traffic on port 2404 routes correctly before reaching this
rule. Architecture: ADR-013.

Detections emitted:

| Detection | Technique | Tactic | Trigger |
|-----------|-----------|--------|---------|
| Switching control command (C_SC/C_DC/C_RC) | T1692.001 | Impair Process Control | TypeID 45–47 observed on passive monitor |
| Set-point/bitstring write command (C_SE/C_BO) | T1692.001, T0836 | Impair Process Control | TypeID 48–51 observed on passive monitor |
| Reset Process command (C_RP_NA_1) | T0827 | Impact | TypeID 105 observed; verdict Likely |
| STOPDT-act (service stop) | T0881 | Inhibit Response Function | U-frame CF1=0x13; verdict Possible (session was started) or Likely (no prior STARTDT on this flow) |
| Non-canonical U-frame CF1 | T0814 | Inhibit Response Function | U-frame CF1 not in canonical set {0x07,0x0B,0x13,0x23,0x43,0x83}; potential CVE-2026-1773 denial-of-service attack |
| N(S) sequence desync | T1692.001 | Impair Process Control | I-format N(S) gap > 12 (15-bit modular); possible replay injection or adversarial manipulation |
| Malformed LEN byte | T0814 | Inhibit Response Function | 0x68 start byte followed by LEN outside [4, 253]; one finding per direction per flow |
| Carry-buffer overflow | T0814 | Inhibit Response Function | Directional carry buffer exceeds 255 bytes; adversarial or non-conformant byte sequence |
| Reserved/invalid TypeID | T0814 | Inhibit Response Function | TypeID=0 (undefined) or TypeID in [128, 255] (private-use/reserved range) |

CLI flags:
- `--iec104` — enable IEC 60870-5-104 TCP analysis (also included in `-a`/`--all`; default-off)

JSON output counters (present in the IEC-104 analyzer's `detail` object in JSON output, at
`analyzers[i].detail`, when using `--json` / `--output-format json`):
- `dropped_findings` — count of findings silently dropped after the `MAX_FINDINGS = 10 000`
  per-analyzer cap was reached; a non-zero value means some detection events were not emitted
- `flows_analyzed` — count of flows processed (closed flows plus still-open flows at summarize time)

## Supported Capture Formats

Both classic pcap (libpcap) and pcapng files are supported. Format is detected
automatically by a magic-byte probe on the first four bytes of the file — no
file extension is required.

**Classic pcap** (.pcap) — all four byte-order and timestamp-resolution variants
(big/little-endian, microsecond/nanosecond) are accepted. Snaplen-truncated captures
(e.g. `tcpdump -s 96`) are read correctly.

**pcapng** (.pcapng) — the reader parses SHB, IDB, EPB, and SPB blocks. NRB, ISB,
SJE, DSB (Decryption Secrets), OPB, and unrecognized block types are silently
skipped. Up to 65,535 Interface Description Blocks are accepted per file; all
interfaces in a single file must share the same link type. Multi-section files
(a second SHB block) are not supported — use `mergecap` or `editcap` to
re-save as a single-section file. The all-in-memory model imposes a 4 GiB
per-file size cap (E-INP-014).

### Supported Link Types

The following link-layer types are supported in both classic pcap and pcapng files:

| Type | ID | Status |
|------|-----|--------|
| Ethernet | 1 | Supported |
| Raw IP | 101 | Supported |
| Linux Cooked (SLL) | 113 | Supported |
| IPv4 | 228 | Supported |
| IPv6 | 229 | Supported |

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

## Known Limitations

### Uncalibrated detection-threshold defaults

Three families of detection thresholds ship with engineering defaults that have not been validated against
a labelled ICS/OT traffic corpus. They were accepted as reasonable starting points on
2026-07-08 and may produce false positives or false negatives on unusual networks.
Other detector thresholds (e.g. Modbus and EtherNet/IP write-burst limits, the ARP spoof rebind
threshold) are likewise engineering defaults; the three families below are those with the least
external calibration evidence.

**Reassembly anomaly thresholds** — the TCP reassembly engine emits anomaly findings when
overlapping-segment counts exceed 50 per flow direction (`--overlap-threshold`, default 50),
when consecutive runs of small segments (under 16 bytes) exceed 100 (`--small-segment-threshold`,
default 100; `--small-segment-max-bytes`, default 16), or when out-of-window segments exceed
100 per flow direction (`--out-of-window-threshold`, default 100). No NIDS ships enabled,
directly-comparable count-based defaults for these detectors; these values are conservative
engineering estimates.

**ARP storm rate** — the ARP storm detector (`--arp`) fires when a source MAC sends 50 or more
ARP frames per second (at or above `--arp-storm-rate`, default 50). This value is not derived from any
external standard. Typical OT/ICS segments are quiet, so operators may lower this to 5–20 to
catch low-rate storms; conversely, environments with chatty PLCs or RTUs that legitimately probe
at high rates should raise it above their baseline to avoid false positives.

**DNP3 direct-operate burst threshold** — the DNP3 analyzer (`--dnp3`) fires a control-command
burst finding when more than 10 Control-class function codes arrive within the 60-second
detection window (`--dnp3-direct-operate-threshold`, default 10). This value was chosen to
tolerate routine maintenance while catching commissioning-speed attacks; quiet OT segments may
need a lower value (3–5). Note: the threshold counts per-flow control events across both
captured directions; a unidirectional mirror-tap deployment captures only one direction, which
halves the observable FC rate — operators on mirror taps may need to halve the threshold to
compensate.

### MACsec-Modified ARP limitation

The ARP analyzer correctly decodes VLAN-tagged (802.1Q), double-tagged QinQ (802.1ad), and
MACsec-Unmodified (unencrypted, 802.1AE) ARP frames by computing `arp_offset = 14 + Σ
header_len()` over all link-extension headers without hardcoding (D-078, BC-2.16.015 /
BC-2.16.009, STORY-116 / STORY-117). Two residual boundaries remain: (1) MACsec-Modified
(encrypted) frames carry opaque ciphertext — the ARP parse path is unreachable by construction,
so no ARP findings are produced for those frames; (2) real-world MACsec-over-ARP capture
behavior is documented-unverified because no public MACsec-over-ARP PCAP fixture exists
(EC-007 / EC-009(c), STORY-117, E-17, CWE-693).

## Roadmap

See [open issues](https://github.com/Zious11/wirerust/issues) for planned features:

- C2 beaconing detection
- SQLite export
- Parallel file processing

## License

[MIT](LICENSE)
