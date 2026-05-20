---
artifact: architecture-section
section: api-surface
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# API Surface

## External API (binary CLI)

wirerust has no network API, no library API published to crates.io, and no IPC
interface. The only external API is the CLI and its stdout/exit-code contract.

### CLI Subcommands

```
wirerust analyze [OPTIONS] <TARGET>...
wirerust summary [OPTIONS] <TARGET>...
```

### Global Flags (apply to all subcommands)

| Flag | Type | Default | Notes |
|------|------|---------|-------|
| `--no-color` | bool | false | BC-2.12.003 |
| `--output-format <FMT>` | `OutputFormat` | None (terminal) | BC-2.12.004; enum values: `json`, `csv` |
| `--json [<FILE>]` | `Option<Option<PathBuf>>` | None | BC-2.12.017; mutually exclusive with --csv |
| `--csv [<FILE>]` | `Option<Option<PathBuf>>` | None | BC-2.12.017 |
| `--reassemble` | bool | false | BC-2.12.005; mutually exclusive with --no-reassemble |
| `--no-reassemble` | bool | false | BC-2.12.005 |
| `--reassembly-depth <N>` | usize (MB) | 10 | BC-2.12.005; per-direction stream limit |
| `--reassembly-memcap <N>` | usize (MB) | 1024 | BC-2.12.005; global reassembly memory cap |
| `--overlap-threshold <N>` | u32 (0-255) | config default (50) | BC-2.12.005; overlapping-segment anomaly threshold |
| `--small-segment-threshold <N>` | u32 (0-2048) | config default (100) | BC-2.12.005; consecutive small-segment run threshold |
| `--small-segment-max-bytes <N>` | u16 (0-2048) | config default (16) | BC-2.12.005; payload-size cutoff for "small" segment |
| `--small-segment-ignore-ports <LIST>` | `Vec<u16>` (comma-sep) | config default (23,513) | BC-2.12.005; ports exempt from small-segment detection |
| `--out-of-window-threshold <N>` | u32 | config default (100) | BC-2.12.005; out-of-window anomaly threshold |

### analyze Flags

| Flag | Type | Default | BC |
|------|------|---------|-----|
| `<TARGET>...` | `Vec<PathBuf>` | required | BC-2.12.001 |
| `--dns` | bool | false | BC-2.12.001 |
| `--http` | bool | false | BC-2.12.001 |
| `--tls` | bool | false | BC-2.12.001 |
| `-a` / `--all` | bool | false | BC-2.12.008 |
| `--mitre` | bool | false | BC-2.12.004 |

### summary Flags

| Flag | Type | Default | BC |
|------|------|---------|-----|
| `<TARGET>...` | `Vec<PathBuf>` | required | BC-2.12.001 |
| `--hosts` | bool | false | BC-2.12.001 |

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (findings may be empty) |
| 1 | Error (file not found, parse failure, I/O error) |

### Output Contract

- `--output-format terminal` (default): human-readable, ANSI-colored, MITRE-grouped
  when `--mitre` is passed; escapes C0/DEL/C1/backslash per ADR 0003
- `--output-format json` / `--json <FILE>`: RFC 8259-compliant JSON; raw bytes for
  non-control Unicode; C0 control bytes escaped per serde_json; BTreeMap key order
  deterministic
- `--output-format csv` / `--csv <FILE>`: CSV-injection neutralized; file or stdout


## Internal API (Rust traits)

### StreamHandler (L2<->L3 interface, defined in reassembly/handler.rs)

```rust
pub trait StreamHandler {
    fn on_data(&mut self, flow_key: &FlowKey, direction: Direction,
               data: &[u8], offset: u64);
    fn on_flow_close(&mut self, flow_key: &FlowKey, reason: CloseReason);
}
```

Implemented by: `StreamDispatcher`, (transitively) `HttpAnalyzer`, `TlsAnalyzer`.

### StreamAnalyzer (stream-level analyzer interface)

```rust
pub trait StreamAnalyzer: StreamHandler {
    fn name(&self) -> &'static str;
    fn summarize(&self) -> AnalysisSummary;
    fn findings(&self) -> Vec<Finding>;
}
```

Implemented by: `HttpAnalyzer`, `TlsAnalyzer`.

### ProtocolAnalyzer (packet-level analyzer interface, defined in analyzer/mod.rs)

```rust
pub trait ProtocolAnalyzer {
    fn name(&self) -> &'static str;
    fn can_decode(&self, packet: &ParsedPacket) -> bool;
    fn analyze(&mut self, packet: &ParsedPacket) -> Vec<Finding>;
    fn summarize(&self) -> AnalysisSummary;
}
```

Implemented by: `DnsAnalyzer`.

### Reporter (output interface, defined in reporter/mod.rs)

```rust
pub trait Reporter {
    fn report(&mut self, findings: &[Finding], summaries: &[AnalysisSummary],
              summary: &Summary) -> Result<()>;
}
```

Implemented by: `JsonReporter`, `TerminalReporter`, `CsvReporter`.


## Key Public Functions (L1 Ingest Layer)

| Function | File | Signature | Notes |
|----------|------|-----------|-------|
| `decode_packet` | decoder.rs | `pub fn decode_packet(data: &[u8], datalink: DataLink) -> Result<ParsedPacket>` | Link-type whitelist gate + L2-L4 header parse. Data-first argument order. Used by integration tests and VP-008 fuzz target. Accepts ETHERNET, RAW, IPV4, IPV6, LINUX_SLL; rejects all other link types with Err. |


## Key Public Structs (L3 Domain)

| Struct | File | Key Fields |
|--------|------|-----------|
| `Finding` | findings.rs | `category: ThreatCategory`, `verdict: Verdict`, `confidence: Confidence`, `mitre_technique: Option<String>`, `summary: String` (raw), `evidence: Vec<String>` (raw), `timestamp: Option<...>` (always None; O-01) |
| `AnalysisSummary` | analyzer/mod.rs | `analyzer_name: String`, `packets_analyzed: u64`, `detail: BTreeMap<String, serde_json::Value>` |
| `FlowKey` | reassembly/flow.rs | `lower_ip: IpAddr`, `lower_port: u16`, `upper_ip: IpAddr`, `upper_port: u16` (canonically ordered per INV-1) |
| `ParsedPacket` | decoder.rs | `src_ip`, `dst_ip`, `protocol: Protocol`, `transport: TransportInfo`, `payload: Vec<u8>`, `packet_len: u32`, timestamp fields |
| `ReassemblyConfig` | reassembly/config.rs | `max_flows`, `memcap`, `max_depth`, `flow_timeout_secs`, threshold fields |

## No Network Interface

wirerust has zero network-facing API surface. There are no HTTP endpoints, no sockets,
no RPC interfaces, and no IPC channels. The CLI stdin/stdout/stderr are the complete
external interface. This property is an architectural invariant enabling the
"offline forensic tool" guarantee (KD-001).
