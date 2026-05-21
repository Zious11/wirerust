---
document_type: prd-supplement-interface-definitions
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/prd.md
  - .factory/specs/architecture/api-surface.md
  - src/cli.rs
  - src/main.rs
input-hash: "592d3cb"
traces_to: .factory/specs/prd.md
---

# Interface Definitions: wirerust

> PRD supplement -- extracted from PRD Section 3.
> Referenced by: implementer, test-writer, devops-engineer.
> Brownfield: this document describes the CLI surface as it exists in develop HEAD.

## CLI Interface

```
wirerust -- Fast PCAP forensics and network triage CLI

Usage: wirerust [OPTIONS] <COMMAND>

Commands:
  analyze  Analyze PCAP files for threats and anomalies
  summary  Generate a triage summary of PCAP files
  help     Print this message or the help of the given subcommand(s)

Options:
      --no-color
          Disable colored output [global]
      --output-format <OUTPUT_FORMAT>
          Output format (default: terminal table) [global]
          [possible values: json, csv]
      --json [<FILE>]
          Write JSON output to file (or stdout if no path given).
          Mutually exclusive with --csv. [global]
      --csv [<FILE>]
          Write CSV output to file (or stdout if no path given).
          Emits the findings table only. [global]
      --reassemble
          Force TCP stream reassembly on [global; conflicts with --no-reassemble]
      --no-reassemble
          Force TCP stream reassembly off (quick scan) [global]
      --reassembly-depth <REASSEMBLY_DEPTH>
          Per-direction stream reassembly limit in MB (default: 10) [global]
      --reassembly-memcap <REASSEMBLY_MEMCAP>
          Global reassembly memory cap in MB (default: 1024) [global]
      --overlap-threshold <OVERLAP_THRESHOLD>
          Override the overlapping-segment anomaly threshold (default: 50).
          Per flow direction. Range: 0-255. [global]
      --small-segment-threshold <SMALL_SEGMENT_THRESHOLD>
          Override the small-segment anomaly threshold (default: 100).
          Per flow direction. Range: 0-2048. [global]
      --small-segment-max-bytes <SMALL_SEGMENT_MAX_BYTES>
          Override the small-segment payload-size cutoff in bytes (default: 16).
          0 disables small-segment detection. Range: 0-2048. [global]
      --small-segment-ignore-ports <PORTS>
          Override the ports exempt from small-segment detection
          (default: 23,513). Comma-separated. [global]
      --out-of-window-threshold <OUT_OF_WINDOW_THRESHOLD>
          Override the out-of-window anomaly threshold (default: 100).
          Per flow direction. [global]
  -h, --help
          Print help
  -V, --version
          Print version

wirerust-analyze:
Analyze PCAP files for threats and anomalies

Usage: wirerust analyze [OPTIONS] <TARGET>...

Arguments:
  <TARGET>...  PCAP files or directories to analyze

Options:
      --dns    Analyze DNS traffic
      --http   Analyze HTTP traffic
      --tls    Analyze TLS handshakes
      --mitre  Group findings by MITRE ATT&CK tactic and show technique names
  -a, --all    Run all analyzers (equivalent to --dns --http --tls)
  -h, --help   Print help

wirerust-summary:
Generate a triage summary of PCAP files

Usage: wirerust summary [OPTIONS] <TARGET>...

Arguments:
  <TARGET>...  PCAP files or directories to summarize

Options:
      --hosts  Include per-host breakdown of source/destination IPs
  -h, --help   Print help
```

### Flag Type Constraints

| Flag | Rust Type | Constraints | Default |
|------|-----------|-------------|---------|
| `<TARGET>...` | `Vec<PathBuf>` | required; 1+ file or directory paths | -- |
| `--no-color` | `bool` | global; presence = true | false |
| `--output-format` | `Option<OutputFormat>` | enum: `json` or `csv`; global | None (terminal) |
| `--json` | `Option<Option<PathBuf>>` | global; mutually exclusive with `--csv` | None |
| `--csv` | `Option<Option<PathBuf>>` | global; mutually exclusive with `--json` | None |
| `--reassemble` | `bool` | global; conflicts with `--no-reassemble` | false |
| `--no-reassemble` | `bool` | global | false |
| `--reassembly-depth` | `usize` | MB; global | 10 |
| `--reassembly-memcap` | `usize` | MB; global | 1024 |
| `--overlap-threshold` | `Option<u32>` | range 0..=255; global | None (50 from config default) |
| `--small-segment-threshold` | `Option<u32>` | range 0..=2048; global | None (100 from config default) |
| `--small-segment-max-bytes` | `Option<u16>` | range 0..=2048; global | None (16 from config default) |
| `--small-segment-ignore-ports` | `Option<Vec<u16>>` | comma-delimited u16 list; global | None (23,513 from config) |
| `--out-of-window-threshold` | `Option<u32>` | global | None (100 from config default) |
| `--dns` | `bool` | analyze subcommand only | false |
| `--http` | `bool` | analyze subcommand only | false |
| `--tls` | `bool` | analyze subcommand only | false |
| `--mitre` | `bool` | analyze subcommand only | false |
| `--all` / `-a` | `bool` | analyze subcommand; equivalent to `--dns --http --tls` | false |
| `--hosts` | `bool` | summary subcommand only | false |

### Intentionally Absent Flags (removed in remediation cycle)

The following flags were parsed in earlier versions but unwired. They have been
REMOVED from the CLI surface per LESSON-P1.04 (no unwired flags):
`--threats`, `--beacon`, `--filter <BPF>`, `--verbose`, `--services`.

The remaining intentionally-absent behaviors (things still parsed but explicitly
no-op) are limited to: none at this revision. Every flag in the current `Cli`
struct has a consumer in `src/main.rs`.


## Exit Code Semantics

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | Pipeline completed; findings may be empty; JSON/terminal output written |
| 1 | Error (anyhow propagated) | File not found; unsupported link type; pcap header parse failure; pcap per-packet read error; output file write failure (E-INP-001..006, E-OUT-001..002) |
| 2 | Argument parse error (clap) | Mutually exclusive flags combined (--reassemble + --no-reassemble, --json + --csv); invalid flag value outside allowed range (E-CFG-001..005) |

Note: decode errors on individual packets do NOT produce exit code 1. They are
counted into `Summary.skipped_packets` and the analysis continues. Exit code 1
is reserved for unrecoverable failures that abort the pipeline (missing file,
bad pcap header, I/O write failure). BC-2.12.012 (target not found) produces
exit 1 via `anyhow::bail!`.


## JSON Output Schema

The `--output-format json` / `--json [<FILE>]` path invokes `JsonReporter`
which calls `serde_json::to_string_pretty`. The output is a single JSON object.

```json
{
  "$schema": "https://json-schema.org/draft/2020-12",
  "type": "object",
  "required": ["summary", "findings", "analyzers"],
  "properties": {
    "summary": {
      "type": "object",
      "description": "Capture-level statistics aggregated across all target files",
      "required": ["total_packets", "total_bytes", "skipped_packets", "unique_hosts", "protocols", "services"],
      "properties": {
        "total_packets": {
          "type": "integer",
          "description": "Total packets successfully decoded (decode errors excluded)"
        },
        "total_bytes": {
          "type": "integer",
          "description": "Sum of packet_len fields for all decoded packets"
        },
        "skipped_packets": {
          "type": "integer",
          "description": "Count of packets that failed decode_packet; incremented per-target"
        },
        "unique_hosts": {
          "type": "array",
          "items": { "type": "string" },
          "description": "Sorted deduplicated list of IP addresses (IPv4 and IPv6) seen as src or dst"
        },
        "protocols": {
          "type": "object",
          "description": "Map of protocol name to packet count (e.g. {\"Tcp\": 412, \"Udp\": 88})",
          "additionalProperties": { "type": "integer" }
        },
        "services": {
          "type": "object",
          "description": "Map of service hint name to count, derived from app_protocol_hint() (e.g. {\"HTTP\": 200})",
          "additionalProperties": { "type": "integer" }
        }
      }
    },
    "findings": {
      "type": "array",
      "description": "Ordered list of forensic findings emitted by all analyzers",
      "items": {
        "type": "object",
        "required": ["category", "verdict", "confidence", "summary", "evidence"],
        "properties": {
          "category": {
            "type": "string",
            "description": "ThreatCategory variant",
            "enum": ["Reconnaissance", "LateralMovement", "C2", "Exfiltration", "CredentialAccess", "Persistence", "Execution", "Anomaly"]
          },
          "verdict": {
            "type": "string",
            "enum": ["Likely", "Unlikely", "Inconclusive"]
          },
          "confidence": {
            "type": "string",
            "enum": ["High", "Medium", "Low"]
          },
          "summary": {
            "type": "string",
            "description": "Raw (unescaped) human-readable summary. May contain attacker-controlled bytes. C0 control bytes escaped by serde_json RFC 8259; C1 codepoints pass through as raw UTF-8."
          },
          "evidence": {
            "type": "array",
            "items": { "type": "string" },
            "description": "Raw (unescaped) supporting evidence lines. Same escaping contract as summary."
          },
          "mitre_technique": {
            "type": "string",
            "description": "MITRE ATT&CK technique ID (e.g. T1027). Present only when a clean mapping exists; omitted entirely when None (not null).",
            "pattern": "^T[0-9]{4}(\\.[0-9]{3})?$"
          },
          "source_ip": {
            "type": "string",
            "description": "Source IP address of the packet or flow that triggered this finding, if applicable. Omitted when None."
          },
          "timestamp": {
            "type": "string",
            "format": "date-time",
            "description": "Packet-derived timestamp. Omitted when None. IMPORTANT: currently always absent (domain-debt O-01); all emission sites set timestamp: None."
          },
          "direction": {
            "type": "string",
            "enum": ["ClientToServer", "ServerToClient"],
            "description": "TCP stream direction if the finding came from a stream analyzer. Omitted for UDP/ICMP or engine-level summary findings."
          }
        }
      }
    },
    "analyzers": {
      "type": "array",
      "description": "Per-analyzer summary objects emitted by StreamAnalyzer::summarize() / ProtocolAnalyzer::summarize()",
      "items": {
        "type": "object",
        "required": ["analyzer_name", "packets_analyzed", "detail"],
        "properties": {
          "analyzer_name": {
            "type": "string",
            "description": "E.g. 'tcp_reassembler', 'http', 'tls', 'dns'"
          },
          "packets_analyzed": {
            "type": "integer"
          },
          "detail": {
            "type": "object",
            "description": "Analyzer-specific key-value pairs. Values are serde_json::Value (string, integer, array, or object). Keys and value shapes differ per analyzer (see Per-Analyzer Detail Shapes below).",
            "additionalProperties": true
          }
        }
      }
    }
  }
}
```

### Per-Analyzer Detail Shapes

#### TCP Reassembly

`analyzer_name`: `"TCP Reassembly"` (as returned by `TcpReassembler::summarize()`)

| Key | Value Type | Description |
|-----|-----------|-------------|
| `packets_processed` | integer | Total packets fed to process_packet |
| `packets_skipped_non_tcp` | integer | Non-TCP packets skipped |
| `flows_total` | integer | Total TCP flows seen |
| `flows_partial` | integer | Flows that never reached Established state |
| `flows_fin` | integer | Flows closed by FIN handshake |
| `flows_rst` | integer | Flows closed by RST |
| `flows_completed` | integer | flows_fin + flows_rst (convenience sum) |
| `flows_expired` | integer | Flows closed by timeout or finalize |
| `evictions` | integer | Flows evicted under memcap or max_flows pressure |
| `bytes_reassembled` | integer | Total bytes delivered to StreamHandler |
| `segments_inserted` | integer | Segments successfully inserted |
| `segments_duplicates` | integer | Duplicate segment insertions (already-buffered offset) |
| `segments_overlaps` | integer | Conflicting overlap events (first-wins policy triggered) |
| `segments_out_of_window` | integer | Segments rejected as outside max_receive_window |
| `segments_segment_limit` | integer | Segments dropped because per-direction segment count cap hit |
| `segments_depth_exceeded` | integer | Segments dropped because per-direction byte depth cap hit |
| `dropped_findings` | integer | Findings suppressed because MAX_FINDINGS cap (10,000) was already full |
| `unclassified_flows` | integer | Flows dispatcher could not classify (injected from dispatcher, not from reassembler stats) |

#### HTTP

`analyzer_name`: `"HTTP"` (as returned by `HttpAnalyzer::summarize()`)

| Key | Value Type | Description |
|-----|-----------|-------------|
| `transactions` | integer | Successfully parsed HTTP responses (each counted as one transaction) |
| `methods` | object | Map of HTTP method string to request count (all parsed methods; capped at MAX_MAP_ENTRIES=50,000) |
| `status_codes` | object | Map of HTTP status code (as string) to response count |
| `top_hosts` | array | Top 20 Host header values by frequency |
| `recent_uris` | array | First 20 of up to MAX_URIS=10,000 URIs buffered (insertion order, not frequency) |
| `user_agents` | object | Map of User-Agent string to count (all seen; capped at MAX_MAP_ENTRIES=50,000) |
| `parse_errors` | integer | Accumulated parse-error increments (request + response combined) |
| `non_http_flows` | integer | Flows poisoned in at least one direction (counted once per flow per direction) |
| `poisoned_bytes_skipped` | integer | Bytes skipped after direction poisoning |

#### TLS

`analyzer_name`: `"TLS"` (as returned by `TlsAnalyzer::summarize()`)

Note: `packets_analyzed` on the TLS summary object equals `handshakes_seen` (ClientHello count), not total TCP segments.

| Key | Value Type | Description |
|-----|-----------|-------------|
| `top_snis` | array | Top 20 SNI hostnames by frequency (sorted descending) |
| `ja3_hashes` | object | Map of JA3 MD5 hex string to count (all seen; capped at MAX_MAP_ENTRIES=50,000) |
| `ja3s_hashes` | object | Map of JA3S MD5 hex string to count (all seen; capped at MAX_MAP_ENTRIES=50,000) |
| `tls_versions` | object | Map of TLS version number (as string) to count |
| `cipher_suites` | object | Map of cipher suite name (or hex fallback) to count |
| `parse_errors` | integer | Accumulated TLS parse-error increments (includes oversized record drops) |
| `truncated_records` | integer | Records dropped because declared payload length exceeded MAX_RECORD_PAYLOAD=18,432 bytes |

#### DNS

`analyzer_name`: `"DNS"` (as returned by `DnsAnalyzer::summarize()`)

| Key | Value Type | Description |
|-----|-----------|-------------|
| `dns_queries` | integer | QR-bit=0 packets |
| `dns_responses` | integer | QR-bit=1 packets |


### JSON Option Field Serialization Contract

All three `Option<_>` fields on `Finding` use `#[serde(skip_serializing_if = "Option::is_none")]`:
- `mitre_technique: Option<String>` -- omitted when None; present as string when Some
- `source_ip: Option<IpAddr>` -- omitted when None; present as string when Some
- `timestamp: Option<DateTime<Utc>>` -- omitted when None (always None today per O-01)
- `direction: Option<Direction>` -- omitted when None

Downstream consumers MUST handle key-absent rather than key-present-but-null for all four
fields. This is a symmetric contract (all four use skip_serializing_if). Per NFR-OBS-010 /
LESSON-P1.02, the asymmetry present in earlier revisions has been corrected.


## Config File Schema

wirerust has NO configuration file. All behavior is controlled by CLI flags. There is
no TOML/YAML/JSON config file support. The only persistent state is `Cargo.lock`
(build artifact).

Runtime defaults are hardcoded in `src/reassembly/config.rs` (`ReassemblyConfig::default()`):

| Setting | Default | CLI Override |
|---------|---------|--------------|
| max_depth | 10 MB (10 * 1024 * 1024 bytes) | `--reassembly-depth <MB>` |
| memcap | 1 GB (1024 * 1024 * 1024 bytes) | `--reassembly-memcap <MB>` |
| flow_timeout_secs | 300 s | -- (not yet CLI-overridable) |
| max_flows | 100,000 | -- (not yet CLI-overridable) |
| max_segments_per_direction | 10,000 | -- (not yet CLI-overridable) |
| max_receive_window | 1,048,576 bytes (1 MB) | -- (not yet CLI-overridable) |
| overlap_alert_threshold | 50 | `--overlap-threshold <N>` |
| small_segment_alert_threshold | 100 | `--small-segment-threshold <N>` |
| small_segment_max_bytes | 16 | `--small-segment-max-bytes <N>` |
| small_segment_ignore_ports | [23, 513] | `--small-segment-ignore-ports <PORTS>` |
| out_of_window_alert_threshold | 100 | `--out-of-window-threshold <N>` |


## Flag Interactions

| Flag A | Flag B | Interaction | Resolution |
|--------|--------|-------------|------------|
| `--reassemble` | `--no-reassemble` | conflicts (clap `conflicts_with`) | Clap rejects the combination at parse time; exits with error before any analysis runs |
| `--json [<FILE>]` | `--csv [<FILE>]` | mutually exclusive (clap `conflicts_with`) | Clap rejects the combination at parse time |
| `--http` or `--tls` | `--no-reassemble` | semantic conflict (not clap-enforced) | Warning printed to stderr: "Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped." Analysis continues without stream analyzers. BC-2.12.009 |
| `--all` | `--dns`, `--http`, `--tls` | `--all` is OR-expanded to `dns || all`, `http || all`, `tls || all` | No conflict; `--all` is additive. BC-2.12.008 |
| `--output-format json` | `--json [<FILE>]` | `--json` takes precedence over `--output-format` | `resolve_format()` checks `cli.json.is_some()` first; `--output-format` is ignored when `--json` is set. BC-2.12.017 |
| `--output-format csv` | `--csv [<FILE>]` | `--csv` takes precedence over `--output-format` | Same precedence rule; `--csv` wins. BC-2.12.017 |
| `--no-color` | `NO_COLOR` env var | either disables color | `use_color = !cli.no_color && std::env::var("NO_COLOR").is_err()`. BC-2.12.003, BC-2.12.010 |
| `--json [<FILE>]` with no `<FILE>` path | stdout | no path given = stdout | `--json` accepts `Option<Option<PathBuf>>`; `Some(None)` = format JSON to stdout. BC-2.12.017 |
| `--csv [<FILE>]` with no `<FILE>` path | stdout | no path given = stdout | Same semantics as `--json`. |
| `--reassembly-depth <MB>` | `--no-reassemble` | depth ignored when reassembly disabled | Config is constructed only when `needs_reassembly && !skip_reassembly`; the CLI default still parses but the value is discarded. |

### Output Format Precedence (highest to lowest)

1. `--json [<FILE>]` forces `OutputFormat::Json`
2. `--csv [<FILE>]` forces `OutputFormat::Csv`
3. `--output-format <fmt>` honored as-is
4. Default: `TerminalReporter`


## stderr vs stdout Contract

| Stream | Content |
|--------|---------|
| stdout | All output from Reporter::render (terminal, JSON, or CSV). Writing to file when `--json <FILE>` or `--csv <FILE>` includes a path bypasses stdout entirely (uses `std::fs::write`). |
| stderr | Progress bar (indicatif); first decode-error warning; --no-reassemble conflict warning; one-shot ISN-missing warning; one-shot close-flow-missing warning |
