# Changelog

All notable changes to wirerust are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Version numbers follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.2.0] - 2026-06-09

### Added

- **Finding timestamp provenance** — every `Finding` now carries a
  `capture_ts` field populated with the pcap capture-relative timestamp of
  the packet that triggered the finding. The timestamp is threaded from the
  pcap reader through `StreamHandler::on_data` all the way to each Finding
  emission site in the TLS and HTTP analyzers. It is surfaced as an RFC 3339
  string in JSON output and as a new `timestamp` column in CSV output
  (#100; PRs #197, #198, #199; BC-2.04.055, BC-2.09.007, VP-021).
  Segment-limit summary findings intentionally carry no timestamp (correct
  by design).

### Fixed

- SNI control-byte summary now correctly surfaces control bytes in the
  human-readable finding for mixed control + non-ASCII values (#104, PR #194).
- Weak-cipher evidence vector is capped at 64 entries with an elision marker
  to prevent unbounded growth on adversarial captures (#102, PR #195).

### CI / Build / Supply-chain

- Migrated release workflow actions from Node 20 to Node 24 with fresh
  SHA-pinned refs (`upload-artifact` v7.0.1, `download-artifact` v8.0.1,
  `softprops/action-gh-release` v3.0.0); added Dependabot tracking for
  workflow actions (PR #192).
- SHA-pinned all remaining CI actions (`actions/checkout`, `rust-cache`,
  `cargo-deny`, `amannn/action-semantic-pull-request`) and added the
  **action-pin-gate** enforcement job that fails CI if any action ref is
  not a 40-char hex SHA (PR #196).
- Test and spec hardening for timestamp provenance: exact-value assertions
  replacing approximate checks, stale doc-comment corrections (PRs #200, #201).

## [0.1.0] - 2026-06-08

### Added

**Core pipeline**

- PCAP reader supporting five link types: Ethernet (1), Raw IP (101), Linux
  Cooked / SLL (113), IPv4 (228), and IPv6 (229). Snaplen-truncated captures
  (e.g. `tcpdump -s 96`) are accepted via the unvalidated raw-record path.
  pcapng is not supported.
- Zero-copy L2–L4 packet decoding via `etherparse`. The full capture is loaded
  into memory as a `Vec<RawPacket>` before analysis; available RAM determines
  the practical file-size limit.
- Single-pass analysis pipeline: Reader → Decoder → Analyzers → Reporter,
  producing host/service/protocol summaries and threat findings in one pass.
- Directory expansion: pass a directory path and wirerust processes every
  `.pcap` file found within it (`.pcapng` files are excluded).

**TCP stream reassembly engine**

- Forensic-grade TCP stream reassembly with a first-wins overlap policy
  (earlier-arriving data wins on byte conflicts).
- Configurable per-direction depth limit (`--reassembly-depth`, default 10 MB)
  and global memory cap (`--reassembly-memcap`, default 1024 MB).
- Evasion and anomaly detection: overlapping-segment counting
  (`--overlap-threshold`, default 50 per flow direction), consecutive
  small-segment detection (`--small-segment-threshold`, default 100 run
  length; `--small-segment-max-bytes`, default 16 B), and out-of-window
  segment counting (`--out-of-window-threshold`, default 100).
- Interactive-protocol port exemption from small-segment detection (default:
  ports 23 and 513; overridable via `--small-segment-ignore-ports`).
- Idle-flow expiry: flows silent longer than `--flow-timeout` seconds
  (default 300) are evicted from the flow table.
- Reassembly statistics surfaced in all output formats: bytes reassembled,
  segment-limit drops, overlap count, out-of-window count, and small-segment
  count.

**Protocol analyzers**

- DNS analyzer: traffic statistics including query/response counts,
  top queried hostnames, and query-type distribution.
- HTTP/1.x analyzer (requires TCP reassembly): stream-level request and
  response parsing with detection for path traversal sequences, web-shell
  indicators, unusual HTTP methods, missing or empty Host headers, and other
  header anomalies. Parse-error isolation prevents one poisoned stream from
  affecting other flows.
- TLS analyzer: ClientHello and ServerHello parsing; SNI extraction and
  classification (clean ASCII, ASCII control bytes C0/DEL, valid non-ASCII
  UTF-8, non-UTF-8 bytes); JA3 and JA3S fingerprinting with GREASE
  value filtering; weak cipher detection; deprecated SSL 2.0 and 3.0
  detection.
- Stream dispatcher: content-first protocol classification (TLS record
  signature, HTTP prefix, then port-based fallback) with classification
  caching and a configurable retry budget (`max_classification_attempts`).

**Threat detection and MITRE ATT&CK**

- Finding system with verdict, confidence score, source IP, direction tag,
  and optional MITRE ATT&CK technique ID.
- Static MITRE ATT&CK catalog mapping technique IDs (T-format) to tactic and
  technique name, consumed by the terminal reporter when `--mitre` is passed.
- `--mitre` flag groups terminal output by ATT&CK tactic with technique names
  displayed alongside each finding.

**Output formats and CLI**

- Colored terminal reporter with MITRE tactic grouping, top-SNI and top-host
  tables, reassembly statistics section, and skipped-packet accounting.
  Deterministic tie-ordering for top-SNI and top-host tables.
- JSON reporter: structured output with deterministic field ordering,
  `skipped_packets` counter, and `dropped_findings` counter. `#[non_exhaustive]`
  on public enums for forward compatibility.
- CSV reporter: 9-column findings table (tactic, verdict, confidence,
  source IP, destination IP, port, protocol, description, MITRE technique).
  CSV-injection neutralization applied to all string fields. Evidence strings
  joined with a pipe separator.
- Output routing: `--output-format json|csv` writes to stdout; `--json [FILE]`
  and `--csv [FILE]` write to a file (or stdout if no path is given).
  `--json` and `--csv` are mutually exclusive.
- `analyze` subcommand with `--dns`, `--http`, `--tls`, `--mitre`, and
  `-a/--all` flags. HTTP analysis automatically enables TCP reassembly.
- `summary` subcommand with optional `--hosts` flag for a per-host IP
  breakdown. Outputs total packets, bytes, protocol distribution, and
  service-hint counts.
- `--no-color` flag disables ANSI color globally.
- Zero, non-integer, or out-of-range values for `--reassembly-depth` and
  `--reassembly-memcap` are rejected at argument-parse time.

**Observability**

- `dropped_findings` counter tracks findings discarded when the per-analyzer
  cap is reached; surfaced in JSON output.
- `skipped_packets` counter tracks packets skipped during decode; surfaced in
  all output formats.
- `truncated_records` counter tracks snaplen-truncated records; surfaced in
  JSON output.
- Criterion micro-benchmarks for hot paths in the decoder and reassembly
  engine.

### Security

- Bumped `indicatif` from 0.17 to 0.18 to transitively drop the unmaintained
  `number_prefix` crate (RUSTSEC-2025-0119).
- `cargo audit` and `cargo deny` supply-chain checks added to CI.
- Release profile enables `overflow-checks = true` so integer overflows are
  caught in release builds.
- Output sanitization in the terminal reporter guards against C1 control bytes
  in packet-derived strings.

[Unreleased]: https://github.com/Zious11/wirerust/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/Zious11/wirerust/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/Zious11/wirerust/releases/tag/v0.1.0
