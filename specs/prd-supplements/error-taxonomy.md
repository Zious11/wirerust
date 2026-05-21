---
document_type: prd-supplement-error-taxonomy
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
inputs:
  - .factory/specs/prd.md
  - src/reader.rs
  - src/decoder.rs
  - src/main.rs
  - src/reassembly/mod.rs
  - src/reassembly/segment.rs
  - src/analyzer/http.rs
  - src/analyzer/tls.rs
  - src/analyzer/dns.rs
input-hash: "592d3cb"
traces_to: .factory/specs/prd.md
---

# Error Taxonomy: wirerust

> PRD supplement -- extracted from PRD Section 5.
> Referenced by: implementer, test-writer.
> Brownfield: error paths catalogued from current develop HEAD source.

## Severity Definitions

| Severity | Meaning | Exit Code Impact | User Signal |
|----------|---------|-----------------|-------------|
| `broken` | Cannot continue; analysis aborts | Non-zero exit (1) | Error message via anyhow chain to stderr |
| `degraded` | Partial result possible; analysis continues | Exit 0 | Warning on stderr; `skipped_packets` or `parse_errors` counter incremented |
| `cosmetic` | Analyst-visible anomaly in output format or minor signal | Exit 0 | One-shot eprintln or finding emitted |

## Error Categories

| Category Code | Category | Description |
|--------------|----------|-------------|
| `INP` | Input / File | File access, pcap format validation, link-type rejection |
| `DEC` | Decoder | Packet-level decode failures (malformed L2/L3/L4 headers) |
| `RAS` | Reassembly | TCP stream reassembly state-machine edge cases and resource limits |
| `ANA` | Analyzer | Protocol-level parse failures (HTTP, TLS, DNS) |
| `OUT` | Output | File write failures for --json/--csv file paths |
| `CFG` | Configuration | Mutually exclusive flag combinations rejected by clap |


## Error Catalog

### INP: Input / File Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-INP-001 | Input | `broken` | 1 | `src/reader.rs:56-60` | `Unsupported pcap link type: <type>. Supported: Ethernet (1), Raw IP (101), Linux Cooked (113), IPv4 (228), IPv6 (229)` | BC-2.01.001, BC-2.02.008 | Surfaced via anyhow chain. `<type>` is `DataLink` Debug repr (e.g. `UNKNOWN(166)`) |
| E-INP-002 | Input | `broken` | 1 | `src/reader.rs:46` | `Failed to parse pcap header: <underlying>` | BC-2.01.006 | `pcap_file::pcap::PcapReader::new` failure; wrong magic number, truncated file, or pcapng format |
| E-INP-003 | Input | `broken` | 1 | `src/reader.rs:70` | `Failed to read packet: <underlying>` | BC-2.01.007 | Per-packet `next_raw_packet()` failure; corrupt or truncated payload |
| E-INP-004 | Input | `broken` | 1 | `src/reader.rs:86-87` | `Failed to open <path>: <os-error>` | BC-2.12.012 | `std::fs::File::open` failure; file not found, permission denied |
| E-INP-005 | Input | `broken` | 1 | `src/main.rs:147`, `src/main.rs:260` | `Failed to read <path>: <underlying>` | BC-2.12.012 | Wraps E-INP-001..003; `with_context` adds file path. Surfaced via `PcapSource::from_file` in the capture loop |
| E-INP-006 | Input | `broken` | 1 | `src/main.rs:359` | `Target not found: <target>` | BC-2.12.012 | `anyhow::bail!` when target path is neither file nor directory |
| E-INP-007 | Input | `degraded` | 0 | `src/main.rs:165-173` | `Warning: failed to decode packet (<error>). Further errors counted silently.` | BC-2.12.014 | Printed to stderr ONCE per run; subsequent decode errors are counted into `Summary.skipped_packets` silently. Only the first decode error per run produces a message. |

### DEC: Decoder Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-DEC-001 | Decoder | `degraded` | 0 | `src/decoder.rs` (decode_packet) | (no message -- Result::Err returned to caller) | BC-2.02.007 | `etherparse::SlicedPacket::from_ethernet` / `from_ip` / `from_linux_sll` (selected by `datalink` match) fails for genuine structural corruption (bad header version, bad IHL, bad TCP data-offset). NOT triggered by snaplen-length truncation (see E-DEC-002). Propagates as anyhow::Error to caller (main.rs E-INP-007 path). |
| E-DEC-002 | Decoder | `degraded` | 0 | `src/decoder.rs` (lax fallback) | (no message -- continues with degraded ParsedPacket) | BC-2.02.003 | Strict parser returns `SliceError::Len` -> lax (`LaxSlicedPacket`) fallback triggered. Packet decoded with clamped lengths. This is NOT an error from the caller's perspective; it produces a valid ParsedPacket. |
| E-DEC-003 | Decoder | `degraded` | 0 | `src/decoder.rs` | `No IP layer found` | BC-2.02.009 | anyhow error returned when neither IPv4 nor IPv6 is found in the parsed packet. Counted as skipped. |

### RAS: Reassembly Errors / Signals

These are not exit-code-1 errors; they are internal state signals that produce
findings or one-shot warnings. They are catalogued here for implementer completeness.

| Error Code | Category | Severity | Exit Code | Source Location | Signal Type | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|-------------|--------|-------|
| E-RAS-001 | Reassembly | `cosmetic` | 0 | `src/reassembly/segment.rs:16, 54-55` | One-shot stderr: `wirerust: insert_segment called with no ISN set` | BC-2.04.032, BC-2.04.048 | Guarded by `ISN_MISSING_WARNED: AtomicBool` (segment.rs:16). `eprintln!` fires at most ONCE per process (segment.rs:54-55). |
| E-RAS-002 | Reassembly | `cosmetic` | 0 | `src/reassembly/lifecycle.rs:31, 44-47` | One-shot stderr: `wirerust: close_flow called for non-existent key: <key> (reason: <reason>)` | BC-2.04.029 | Guarded by `CLOSE_FLOW_MISSING_WARNED: AtomicBool` (lifecycle.rs:31). `eprintln!` fires at most ONCE (lifecycle.rs:44-47). `<reason>` is the `CloseReason` Debug repr. Indicates a structural invariant violation (flow table inconsistency). |
| E-RAS-003 | Reassembly | `degraded` | 0 | `src/reassembly/mod.rs:432, 466, 495`; `src/reassembly/lifecycle.rs:101, 121` | Silent drop + counter increment | BC-2.04.024 | When `self.findings.len() >= MAX_FINDINGS (10,000)`, further per-flow findings are silently dropped. The `finalize()` summary finding unconditionally bypasses this cap. |
| E-RAS-004 | Reassembly | `cosmetic` | 0 | `src/reassembly/mod.rs:107-118` | `assert!` panic (programmer error only) | BC-2.04.001 | `TcpReassembler::new` panics if any config field is 0 or invalid. Only triggered by programmer misconfiguration (CLI range validators prevent zero from reaching this point in production). |
| E-RAS-005 | Reassembly | `degraded` | 0 | `src/reassembly/segment.rs` | `InsertResult::SegmentLimitReached` returned | BC-2.04.044..046 | When `max_segments_per_direction` (default 10,000) is reached, new segments return `SegmentLimitReached`. Tracked via `segments_segment_limit`. A summary finding is emitted by `finalize()`. |

### ANA: Analyzer Parse Errors

| Error Code | Category | Severity | Exit Code | Source Location | Signal Type | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|-------------|--------|-------|
| E-ANA-001 | Analyzer | `degraded` | 0 | `src/analyzer/http.rs:405, 463` | `parse_errors` counter incremented | BC-2.06.013, BC-2.06.015 | Non-HTTP bytes or incomplete HTTP headers that fail `httparse`. No finding emitted for individual parse errors. After `POISON_THRESHOLD=3` consecutive errors in one direction, that direction is "poisoned" (E-ANA-002). |
| E-ANA-002 | Analyzer | `degraded` | 0 | `src/analyzer/http.rs:406-415` (request), `464-473` (response) | Direction poisoned; `poisoned_bytes_skipped` counter incremented | BC-2.06.015..017 | HTTP direction poisoning: after 3 consecutive parse errors, subsequent bytes in that direction are skipped. Per-direction and per-flow. Cleared on `on_flow_close`. |
| E-ANA-003 | Analyzer | `degraded` | 0 | `src/analyzer/tls.rs:643-653` | `parse_errors` incremented; per-direction buffer cleared | BC-2.07.004, BC-2.07.029 | TLS record payload exceeds `MAX_RECORD_PAYLOAD=18,432` bytes or body parse fails. Buffer is cleared; analysis continues on next record. |
| E-ANA-004 | Analyzer | `degraded` | 0 | `src/analyzer/tls.rs` | `parse_errors` incremented | BC-2.07.029 | TLS record body parsing failure (bad handshake structure, truncated extension). Buffer continues. |
| E-ANA-005 | Analyzer | `cosmetic` | 0 | `src/analyzer/dns.rs` | (none -- counts as query or response) | BC-2.08.002 | DNS parse error is implicit: malformed DNS is silently counted as a query or response based on the QR-bit only; no per-packet parse error counter exists for DNS. |
| E-ANA-006 | Analyzer | `cosmetic` | 0 | `src/analyzer/http.rs:375-389` | New map key silently dropped | BC-2.06.024 | HTTP per-map cardinality (`MAX_MAP_ENTRIES=50,000`): new keys past the cap are dropped; existing keys still increment. Affects: `methods`, `hosts`, `user_agents`, `status_codes`. |
| E-ANA-007 | Analyzer | `cosmetic` | 0 | `src/analyzer/tls.rs:372-375` (increment helper), `387, 416, 494, 549, 564, 568` (call sites) | New map key silently dropped | BC-2.07.028 | TLS per-map cardinality (`MAX_MAP_ENTRIES=50,000`): same behavior. Affects: `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`. SNI anomaly findings still fire even when `sni_counts` is at capacity. |
| E-ANA-008 | Analyzer | `cosmetic` | 0 | `src/analyzer/http.rs:391-392` | URI silently dropped | BC-2.06.025 | HTTP URI list cap: `MAX_URIS=10,000`; further URIs silently dropped from the `uris` list. Detection rules continue to run on dropped URIs. |

### OUT: Output Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-OUT-001 | Output | `broken` | 1 | `src/main.rs:329-330` | `Failed to write JSON output to <path>: <os-error>` | BC-2.12.017 | `std::fs::write` failure when `--json <FILE>` specifies a path (permission denied, disk full, bad path). |
| E-OUT-002 | Output | `broken` | 1 | `src/main.rs:331-332` | `Failed to write CSV output to <path>: <os-error>` | BC-2.12.017 | Same as E-OUT-001 for `--csv <FILE>`. |

### CFG: Configuration Errors

| Error Code | Category | Severity | Exit Code | Source Location | Message Format | BC Ref | Notes |
|-----------|----------|----------|-----------|----------------|----------------|--------|-------|
| E-CFG-001 | Config | `broken` | 2 | clap (src/cli.rs:62) | `error: the argument '--reassemble' cannot be used with '--no-reassemble'` | BC-2.12.007 | clap enforces `conflicts_with`; prints to stderr with usage hint; exits with code 2 (clap's standard argument error code, NOT exit code 1). |
| E-CFG-002 | Config | `broken` | 2 | clap (src/cli.rs:53) | `error: the argument '--json...' cannot be used with '--csv...'` | BC-2.12.017 | clap `conflicts_with = "csv"` on `--json`. Same exit code 2. |
| E-CFG-003 | Config | `broken` | 2 | clap | `error: invalid value '<VAL>' for '--overlap-threshold <OVERLAP_THRESHOLD>': <VAL> is not in 0..=255` | BC-2.12.005 | clap range validator `value_parser(clap::value_parser!(u32).range(0..=255))`. |
| E-CFG-004 | Config | `broken` | 2 | clap | `error: invalid value '<VAL>' for '--small-segment-threshold <SMALL_SEGMENT_THRESHOLD>': <VAL> is not in 0..=2048` | BC-2.12.005 | clap range validator `value_parser(clap::value_parser!(u32).range(0..=2048))`. |
| E-CFG-005 | Config | `broken` | 2 | clap | `error: invalid value '<VAL>' for '--small-segment-max-bytes <SMALL_SEGMENT_MAX_BYTES>': <VAL> is not in 0..=2048` | BC-2.12.005 | clap range validator `value_parser(clap::value_parser!(u16).range(0..=2048))`. |
| E-CFG-006 | Config | `degraded` | 0 | `src/main.rs:90-93` | `Warning: --http/--tls require TCP reassembly, but --no-reassemble is set. Stream analysis will be skipped.` | BC-2.12.009 | Semantic conflict not enforced by clap; wirerust continues without stream analyzers. Not exit-code 1. |

## Error Handling Strategy Summary

| Layer | Strategy | Rationale |
|-------|----------|-----------|
| File/pcap open | `anyhow::bail!` / `?` propagation -- abort | A missing or unreadable pcap means no analysis is possible for that target. |
| Per-packet decode | Count to `skipped_packets`; continue loop | Single bad packet must not abort the analysis of a valid pcap. |
| Snaplen truncation | Lax fallback -- degrade gracefully | Common in real-world forensic captures; strict rejection would miss valid packets. |
| HTTP parse error | Count to `parse_errors`; optionally poison after threshold | Mid-stream joins produce transient errors; 3 consecutive errors indicate non-HTTP stream. |
| TLS parse / buffer overflow | Count to `parse_errors`; clear buffer; continue | Malformed TLS records should not kill handshake fingerprinting for subsequent records. |
| DNS parse | Implicit QR-bit dispatch; no error counter | DNS is statistics-only; failure to parse one record only affects that record's counter. |
| Findings cap | Silent drop after MAX_FINDINGS=10,000 | Adversarial input could generate unbounded findings; cap prevents memory exhaustion. |
| Output write | `anyhow` `?` -- abort with message | If the analyst-requested output cannot be written, the run is a failure. |
| Clap arg parse | clap exits 2 | Standard UX contract; invalid flags are user errors, not internal errors. |
