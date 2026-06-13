---
artifact: architecture-section
section: module-decomposition
traces_to: ARCH-INDEX.md
version: "1.6"
status: verified
producer: architect
timestamp: 2026-05-20T00:00:00Z
modified:
  - date: 2026-06-08
    actor: spec-steward
    reason: "Phase-6 gate close: status draft→verified."
  - date: 2026-06-10
    actor: architect
    reason: "Remap revoked ATT&CK-ICS v19 IDs in C-22 description: T0855→T1692.001, T0856→T1692.002 (issue #222)."
  - date: 2026-06-10
    actor: architect
    reason: "C-22 over-inclusion correction: removed T1692.002 (catalogue-only, never emitted) from Modbus analyzer findings list (issue #222)."
  - date: 2026-06-12
    actor: architect
    reason: "F2 delta ARP security analyzer: C-23 added (src/analyzer/arp.rs, SS-16, pure core); C-5 description updated to reflect DecodedFrame return type post-etherparse-0.20 migration."
  - date: 2026-06-13
    actor: architect
    reason: "Corpus-wide consistency audit remediation (CD-6/CD-7): Component Inventory preamble updated to reflect current 24-component count including C-21 StreamDispatcher, C-22 ModbusAnalyzer, C-23 ArpAnalyzer, and C-24 Dnp3Analyzer; C-24 DNP3 analyzer row added (analyzer/dnp3.rs, SS-15, shipped v0.6.0; non-chronological C-ID documented)."
  - date: 2026-06-13
    actor: architect
    reason: "ARP-F2 Pass-14 remediation (A-08/A-06): C-16 mitre.rs description extended with T0888 (Modbus recon emitter, ADR-005 D12) and T1691.001/T0827 (DNP3/STORY-109); seeded count 23/target 25 documented. C-22 modbus.rs findings list extended with T0888. Version bump 1.4→1.5."
  - date: 2026-06-13
    actor: architect
    reason: "O-01 closure propagation: Architecture Debt by Component table row updated Open→CLOSED (21/22 sites wired STORY-097/098/099+STORY-102..110; BC-2.04.054 summary finding timestamp:None by design). Version bump 1.5→1.6."
---

# Module Decomposition

## Component Inventory

All 20 components from the ingestion pass plus C-21 (StreamDispatcher, added by ADR 0001), C-22 (ModbusAnalyzer, added F2 issue #7), C-23 (ArpAnalyzer, added F2 issue #9), and C-24 (Dnp3Analyzer, shipped v0.6.0 — see note on C-24 below for non-chronological C-ID assignment). 24 components total.

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
| C-5 | src/decoder.rs | SS-02 | `decode_packet`: link-type whitelist gate + L2-L4 header parse via etherparse 0.20; returns `Result<DecodedFrame>` where `DecodedFrame::Ip(ParsedPacket)` is the IP path and `DecodedFrame::Arp(ArpFrame)` is the new ARP path (ADR-008). Also defines `ArpFrame` struct. `app_protocol_hint` unchanged. | Pure core (no I/O; takes `&[u8]`, returns `Result<DecodedFrame>`) |

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
| C-16 | src/mitre.rs | SS-10 | `MitreTactic` enum; `MitreMatrix` enum (Enterprise/Ics); `technique_info` static match; `technique_name`, `technique_tactic`, `technique_matrix`, `all_tactics_in_report_order` — extended with T0836/T0814/T0806/T0835/T0831/T0888 for SS-14 and T1691.001/T0827 for SS-15 (STORY-109); 23 SEEDED IDs (target: 25 when ARP ships) | Pure core |
| C-17 | src/summary.rs | SS-12 | `Summary`: per-packet accumulator; `ingest`, `unique_hosts`, serialization | Pure core |
| C-22 | src/analyzer/modbus.rs | SS-14 | `ModbusAnalyzer`: `StreamHandler` + `StreamAnalyzer`; per-flow `HashMap<FlowKey, ModbusFlowState>`; MBAP parse + 3-point validity gate; function-code classification; transaction correlation table; write-burst rate detection; findings for T1692.001/T0836/T0814/T0806/T0835/T0831/T0888 (T0888 = recon FC 0x11/0x12/0x2B, Remote System Information Discovery; ADR-005 D12) | Pure core |
| C-23 | src/analyzer/arp.rs | SS-16 | `ArpAnalyzer`: direct `process_arp(&ArpFrame)` method (not ProtocolAnalyzer/StreamAnalyzer); binding table (HashMap<[u8;4], BindingEntry>, LRU-bounded); D1 spoof, D2 GARP, D3 storm, D11 malformed, D12 L2/L3 mismatch detection; T0830+T1557.002 findings (ADR-008) | Pure core |
| C-24 | src/analyzer/dnp3.rs | SS-15 | `Dnp3Analyzer`: `StreamHandler`; carry-buffer + CRC-block-skip parse; FIR=1-only app-layer extract; function-code classification; ICS MITRE findings T1691.001/T0827/T0836/T0814; per-flow master-address tracking (MAX_MASTER_ADDRS); VP-023 Kani obligation (ADR-007). **Note — non-chronological C-ID:** DNP3 shipped before ARP (v0.6.0 vs v0.7.0-planned) but C-IDs are assigned by factory-registration order; C-22 (Modbus) and C-23 (ARP) were registered first. DNP3 receives C-24 by registration sequence, not deployment sequence. Do not renumber C-23 — it is cited in arp-architecture-delta, ARCH-INDEX, module-criticality, and BC-INDEX. | Pure core |

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
| O-01: Finding.timestamp universally None | C-12, C-13, C-6, C-15 | CLOSED (21/22 sites wired; BC-2.04.054 summary finding timestamp:None by design — STORY-097/098/099 + STORY-102..110) |
| O-06: Weak-cipher evidence vec unbounded | C-13 | Open; NFR-RES-023 |
