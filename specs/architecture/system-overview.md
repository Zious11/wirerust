---
artifact: architecture-section
section: system-overview
traces_to: ARCH-INDEX.md
version: "1.0"
status: draft
producer: architect
timestamp: 2026-05-20T00:00:00Z
---

# System Overview

## Product Identity

wirerust is an **offline, single-binary, single-pass forensic triage CLI** that ingests
classic-pcap captures and emits structured findings about HTTP, TLS, and DNS traffic plus
TCP stream-reassembly anomalies. Single Rust crate, Rust 2024 edition, MSRV 1.91.

**Product constraints (hard):**
- No network I/O of any kind -- reads local files only
- No async runtime (synchronous pipeline only)
- No unsafe blocks in product source
- No process-to-process state (binary is the complete unit)
- Classic pcap only; pcapng rejected at reader boundary


## 5-Layer Pipeline

```
L0 Entry
  main.rs   C-1  run_analyze() IIFE owns the per-target loop; calls finalize()
  lib.rs    C-2  pub lib entry point for integration tests
  cli.rs    C-3  clap CLI definition; OutputFormat; ReassemblyConfig flags

L1 Ingest
  reader.rs   C-4  PcapSource: reads classic pcap into Vec<RawPacket>
  decoder.rs  C-5  decode_packet: link-type gate + L2-L4 header parse -> ParsedPacket

L2 Stream
  reassembly/
    mod.rs        C-6   TcpReassembler: flow table; process_packet hot path; finalize
    flow.rs       C-7   TcpFlow: per-flow state machine; ISN tracking; direction
    segment.rs    C-8   FlowDirection: per-direction buffer (BTreeMap<u64,Segment>)
    handler.rs    C-9   StreamHandler / StreamAnalyzer traits (L2<->L3 interface)
    lifecycle.rs  C-15  close_flow, on_rst, on_fin, eviction logic
    config.rs     --    ReassemblyConfig thresholds (not a numbered component)
    stats.rs      --    ReassemblyStats counters
  dispatcher.rs   C-21  StreamDispatcher: content-first classify + route to analyzer

L3 Domain
  analyzer/
    mod.rs   C-10  analyzer module; ProtocolAnalyzer trait
    dns.rs   C-11  DnsAnalyzer: packet-level; statistics-only; no findings
    http.rs  C-12  HttpAnalyzer: stream-level; 8 finding types; HTTP/1.x
    tls.rs   C-13  TlsAnalyzer: stream-level; ClientHello/ServerHello; JA3/JA3S; SNI
  findings.rs  C-14  Finding struct + Verdict/Confidence/ThreatCategory enums
  mitre.rs     C-16  MITRE ATT&CK catalog (15 technique IDs); tactic lookup
  summary.rs   C-17  Summary: per-packet statistics accumulator for the summary subcommand

L4 Output
  reporter/
    mod.rs      C-18  Reporter trait
    json.rs     C-19  JsonReporter: serde_json; deterministic BTreeMap key order
    terminal.rs C-20  TerminalReporter: escape_for_terminal; MITRE tactic grouping
    csv.rs      --    CsvReporter: CSV-injection neutralization
```

> Note: The ingestion pass identified C-1..C-20. C-21 (StreamDispatcher in dispatcher.rs)
> was added by ADR 0001 after the component count was set. The dispatcher sits at the
> L2/L3 boundary.


## Data Flow (single target)

```
CLI parse
  |
  v
resolve_targets(path)          -- SS-12
  |
  v
PcapSource::from_file(path)    -- SS-01: reads entire pcap into Vec<RawPacket>
  |
  v
for each RawPacket:
  decode_packet(raw)            -- SS-02: link-type gate + L2-L4 parse -> ParsedPacket
    |
    +-- [TCP] process_packet    -- SS-04: reassembly engine; on data-flush -> dispatcher
    |     |
    |     v
    |   StreamDispatcher::on_data  -- SS-05: classify flow; route to HTTP or TLS analyzer
    |     |
    |     +-- [HTTP] HttpAnalyzer::on_data  -- SS-06
    |     +-- [TLS]  TlsAnalyzer::on_data   -- SS-07
    |
    +-- [UDP/53] DnsAnalyzer::analyze       -- SS-08: packet-level; no reassembly
    |
    +-- Summary::ingest(packet)             -- SS-12
  |
  v
reassembler.finalize()          -- SS-04: close remaining flows; emit segment-limit finding
  |
  v
collect findings + summaries    -- SS-09, SS-10
  |
  v
reporter.render(...)            -- SS-11: terminal / JSON / CSV
```


## Key Architectural Properties

**Purity boundary:** The domain layer (L3) is pure in the functional sense for all
analysis functions -- they take data in and return Findings. No I/O, no global mutable
state, no side effects except the `static AtomicBool` warning latches in the reassembly
engine (L2). The effectful shell is confined to L0 (CLI + main.rs), L1 (file I/O via
reader.rs), and L4 (stdout/stderr writers). See `purity-boundary-map.md`.

**Single accepted cycle:** The file-level import DAG has one group cycle: the L2 stream
layer (`reassembly/handler.rs`) imports L3 types (`FlowKey`, `Direction`, `Finding`,
`CloseReason`), and the L3 analyzers implement the L2 traits (`StreamHandler`,
`StreamAnalyzer`). This cycle is accepted by ADR 0002 as the cost of the modular
analyzer pattern. It does not prevent formal verification because both sides are
testable independently.

**Forensic fidelity:** INV-4 (ADR 0003) requires that attacker-controlled bytes survive
all pipeline layers unchanged. `escape_for_terminal` executes only in `TerminalReporter`.
`serde_json` handles JSON escaping. All other layers pass raw `String` (post-`from_utf8_lossy`).
