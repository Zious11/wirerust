---
artifact: architecture-section
section: module-decomposition
traces_to: ARCH-INDEX.md
version: "1.1"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-08
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified."
---

# Module Decomposition

## Component Inventory

All 20 components from the ingestion pass plus C-21 (StreamDispatcher, added by ADR 0001).

### L0 Entry Layer

| C-ID | File | SS-ID | Role | Purity |
|------|------|-------|------|--------|
| C-1 | src/main.rs | SS-12 | `run_analyze()` IIFE: per-target loop, finalize() guarantee, reporter dispatch | Effectful shell |
| C-2 | src/lib.rs | SS-12 | Public crate entry point; re-exports for integration tests | Effectful shell |
| C-3 | src/cli.rs | SS-12 | clap CLI definition; `Cli`, `Commands`, `OutputFormat`; `ReassemblyConfig` flag parsing | Pure (data only) |

### L1 Ingest Layer

| C-ID | File | SS-ID | Role | Purity |
|------|------|-------|------|--------|
| C-4 | src/reader.rs | SS-01 | `PcapSource::from_file` / `from_pcap_reader`; reads pcap into `Vec<RawPacket>` | Effectful (file I/O) |
| C-5 | src/decoder.rs | SS-02 | `decode_packet`: link-type whitelist gate + L2-L4 header parse via etherparse; `app_protocol_hint` | Pure core (no I/O; takes `&[u8]`, returns `Result<ParsedPacket>`) |

### L2 Stream Layer

| C-ID | File | SS-ID | Role | Purity |
|------|------|-------|------|--------|
| C-6 | src/reassembly/mod.rs | SS-04 | `TcpReassembler`: flow table (`HashMap<FlowKey,TcpFlow>`), `process_packet`, `finalize`, `expire_flows`, `summarize` | Mixed: pure segment math; effectful: `static AtomicBool` latches, `eprintln!` tripwires |
| C-7 | src/reassembly/flow.rs | SS-04 | `TcpFlow`, `FlowKey`, `FlowState` state machine, `direction()` | Pure core |
| C-8 | src/reassembly/segment.rs | SS-04 | `FlowDirection`: per-direction `BTreeMap<u64, Vec<u8>>` buffer; `insert_segment`, `flush_contiguous` | Pure core |
| C-9 | src/reassembly/handler.rs | SS-04 | `StreamHandler` / `StreamAnalyzer` trait definitions (the L2<->L3 interface) | Pure (trait definitions only) |
| C-15 | src/reassembly/lifecycle.rs | SS-04 | `close_flow`, eviction logic, `on_rst`, `on_fin` | Mixed: pure flow-table mutations; `static AtomicBool` warning latch for missing-key |
| -- | src/reassembly/config.rs | SS-04 | `ReassemblyConfig` struct; threshold fields with research-documented defaults | Pure (data) |
| -- | src/reassembly/stats.rs | SS-04 | `ReassemblyStats` counters struct; `summarize` mapping | Pure (data) |
| C-21 | src/dispatcher.rs | SS-05 | `StreamDispatcher`: `classify()`, `routes: HashMap<FlowKey, DispatchTarget>`, `classification_attempts: HashMap<FlowKey, u32>`, `max_classification_attempts: u32` (LESSON-P2.11 retry-budget cap), `on_data`, `on_flow_close` | Pure core (deterministic routing logic; HashMap state) |

### L3 Domain Layer

| C-ID | File | SS-ID | Role | Purity |
|------|------|-------|------|--------|
| C-10 | src/analyzer/mod.rs | SS-05 | `ProtocolAnalyzer` trait; `AnalysisSummary` struct; module re-exports -- shared analyzer infrastructure consumed by all three protocol analyzers (SS-06/07/08) and routed through by SS-05 per ADR 0002 | Pure (trait definition) |
| C-11 | src/analyzer/dns.rs | SS-08 | `DnsAnalyzer`: `ProtocolAnalyzer`; packet-level QR-bit dispatch; statistics-only (emits no findings) | Pure core |
| C-12 | src/analyzer/http.rs | SS-06 | `HttpAnalyzer`: `StreamAnalyzer`; HTTP/1.x request+response parse; 8 finding types; poison logic | Pure core |
| C-13 | src/analyzer/tls.rs | SS-07 | `TlsAnalyzer`: `StreamAnalyzer`; ClientHello/ServerHello; JA3/JA3S; SNI 4-way; weak cipher; deprecated protocol | Pure core |
| C-14 | src/findings.rs | SS-09 | `Finding`, `Verdict`, `Confidence`, `ThreatCategory`; `#[derive(Serialize)]`; `Display` impls | Pure (data model) |
| C-16 | src/mitre.rs | SS-10 | `MitreTactic` enum; `MitreMatrix` enum (Enterprise/Ics); `technique_info` static match; `technique_name`, `technique_tactic`, `technique_matrix`, `all_tactics_in_report_order` — extended with T0836/T0814/T0806/T0835/T0831 for SS-14 | Pure core |
| C-17 | src/summary.rs | SS-12 | `Summary`: per-packet accumulator; `ingest`, `unique_hosts`, serialization | Pure core |
| C-22 | src/analyzer/modbus.rs | SS-14 | `ModbusAnalyzer`: `StreamHandler` + `StreamAnalyzer`; per-flow `HashMap<FlowKey, ModbusFlowState>`; MBAP parse + 3-point validity gate; function-code classification; transaction correlation table; write-burst rate detection; findings for T0855/T0836/T0814/T0806/T0835/T0831 | Pure core |

### L4 Output Layer

| C-ID | File | SS-ID | Role | Purity |
|------|------|-------|------|--------|
| C-18 | src/reporter/mod.rs | SS-11 | `Reporter` trait | Pure (trait definition) |
| C-19 | src/reporter/json.rs | SS-11 | `JsonReporter`: `serde_json`; `BTreeMap` for deterministic key order | Pure (returns owned String; no I/O -- caller in main.rs writes) |
| C-20 | src/reporter/terminal.rs | SS-11 | `TerminalReporter`: `escape_for_terminal`; MITRE tactic grouping; colorization; 11 inline unit tests | Pure (returns owned String; no I/O -- caller in main.rs writes) |
| -- | src/reporter/csv.rs | SS-11 | `CsvReporter`: CSV-injection neutralization; renders findings to in-memory String | Pure (returns owned String; no I/O -- caller in main.rs writes) |


## Architecture Debt by Component

| Smell / Debt | Affected Component | Status |
|---|---|---|
| Smell #1: mod.rs god-module | C-6 (reassembly/mod.rs, ~691 LOC) | Partially closed; config/stats/lifecycle extracted; hot path remains |
| Smell #4: L2<->L3 trait cycle | C-9 (handler.rs) <-> C-12/C-13 (analyzers) | Advisory; accepted by ADR 0002 |
| Smell #5: DnsAnalyzer::analyze returns empty Vec | C-11 | By design; statistics-only |
| Smell #6: StreamDispatcher pub field exposure | C-21 | Low severity; unchanged |
| Smell #7: pcap_file::DataLink leaks across crate boundary | C-5 | Low severity; unchanged |
| Smell #10: Loose TLS gate (data[2] unchecked) | C-21 (classify function) | Theoretical; no test exercises misroute |
| O-01: Finding.timestamp universally None | C-12, C-13, C-6, C-15 | Open; medium severity |
| O-06: Weak-cipher evidence vec unbounded | C-13 | Open; NFR-RES-023 |
