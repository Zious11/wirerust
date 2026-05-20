---
artifact: L2-domain-spec
project: wirerust
status: complete
generated: 2026-05-20
reconciled: 2026-05-20
mode: brownfield-descriptive
source: .factory/semport/wirerust/ (21 ingestion artifacts; canonical = pass-8-deep-synthesis.md)
reconciled_against: develop HEAD aa2ece9 (PRs #69-#98; 30-lesson remediation cycle)
shards:
  - capabilities/cap-01-pcap-ingestion.md
  - capabilities/cap-02-link-type-gating.md
  - capabilities/cap-03-packet-decoding.md
  - capabilities/cap-04-tcp-reassembly.md
  - capabilities/cap-05-content-first-dispatch.md
  - capabilities/cap-06-http-analysis.md
  - capabilities/cap-07-tls-analysis.md
  - capabilities/cap-08-dns-analysis.md
  - capabilities/cap-09-finding-emission.md
  - capabilities/cap-10-mitre-mapping.md
  - capabilities/cap-11-reporting-output.md
  - capabilities/cap-12-cli-orchestration.md
  - entities/ent-01-ingestion-decoding.md
  - entities/ent-02-reassembly-flow.md
  - entities/ent-03-dispatch-analysis.md
  - entities/ent-04-findings-output.md
  - entities/ent-05-enums-value-objects.md
  - invariants/inv-01-core-invariants.md
  - domain-debt.md
---

# wirerust L2 Domain Specification

## 1. System Identity

wirerust is an **offline, single-binary, single-pass forensic triage tool** that ingests
classic-pcap captures and emits structured findings about HTTP/TLS/DNS traffic plus TCP
stream-reassembly anomalies. It has no network I/O, no async runtime, no unsafe blocks, and
no process-to-process state.

Product identity: "trustworthy forensic data preservation + display-layer safety." Raw
attacker-controlled bytes survive intact through every layer to JSON output; the terminal
renderer is the sole owner of escape logic.

**Codebase metrics (verified against source; source: pass-8-deep-synthesis.md):**

| Metric | Value |
|---|---|
| Rust source files (src/) | 24 |
| Source LOC | 3,868 |
| Test LOC | 6,021 |
| Total #[test] functions | ~282 as of develop@aa2ece9 (264 in tests/ + 18 inline: 11 in reporter/terminal.rs + 7 in analyzer/tls.rs; exact count is commit-sensitive and should be re-verified against current tree) |
| Components | 21 (C-1..C-20 + C-21); 24 source files map to 21 components because 2 reassembly sub-files (config.rs, stats.rs) are unnumbered data-only modules, and csv.rs (reporter/csv.rs) is unnumbered. dispatcher.rs = C-21 (added by ADR 0001 after the C-1..C-20 count was set). The 7 reassembly sub-files (mod, flow, segment, handler, lifecycle, config, stats) map to C-6, C-7, C-8, C-9, C-15 (config.rs and stats.rs unnumbered). |
| Layers | 5 (L0..L4) |
| Behavioral contracts catalogued | 218 ingested / 217 active (212 ingestion-derived + 5 pass-4 CsvReporter) |
| Domain entities | 41 |
| Semantic enums | 14 |
| NFRs | 79 |
| Direct prod deps | 14 (includes rayon = "1", declared but unused in src/ as of this writing) |
| ADRs | 4 (0001/0002/0003/0004) |
| MSRV (declared in Cargo.toml) | rust-version = "1.91" (P0.01 / #69) |


## 2. Architecture Overview (5-layer pipeline)

```
L0 Entry    main.rs / lib.rs / cli.rs           (C-1, C-2, C-3)
            Parse CLI; own per-target packet loop; stdout-only output

L1 Ingest   reader.rs / decoder.rs              (C-4, C-5)
            Link-type whitelist gate; produce ParsedPackets

L2 Stream   reassembly/{mod,flow,segment,       (C-6, C-7, C-8, C-9, C-15)
            handler,lifecycle}.rs               config.rs and stats.rs unnumbered
            TCP stream state; MAX_FINDINGS cap
            dispatcher.rs                       (C-21)
            Content-first dispatch; routes to HTTP/TLS analyzers

L3 Domain   analyzer/{mod,dns,http,tls}.rs      (C-10, C-11, C-12, C-13)
            findings.rs + mitre.rs + summary.rs (C-14, C-16, C-17)
            Three analyzers; Finding schema; MITRE catalog

L4 Output   reporter/{mod,json,terminal}.rs     (C-18, C-19, C-20)
            reporter/csv.rs                     (unnumbered)
            Terminal (with escaping), JSON, and CSV renderers
```

The file-level DAG is acyclic. One module-group cycle exists: analyzer <-> reassembly via
the StreamAnalyzer trait (L2 imports L3 types). Accepted by ADR 0002.


## 3. Four ADRs (pinned decisions)

| ADR | Date | Decision | What it locks |
|---|---|---|---|
| 0001 | 2026-04-07 | Content-first stream dispatch | 5-byte content signature wins over port; ports 80/443/8080/8443 are fallback only |
| 0002 | 2026-04-07 | Modular protocol analyzer pattern | Two-trait split (ProtocolAnalyzer / StreamHandler+StreamAnalyzer); MAX_FINDINGS/MAX_MAP_ENTRIES cardinality |
| 0003 | 2026-04-09 | Reporting pipeline layering | Analyzers store raw bytes; only TerminalReporter escapes; JsonReporter delegates to serde |
| 0004 | 2026-05-14 | Process-wide warning atomics | Single AtomicBool per one-shot warning site; no mutex; three sites: ISN_MISSING_WARNED (segment.rs:16), FINALIZE_SKIPPED_WARNED (mod.rs:70), CLOSE_FLOW_MISSING_WARNED (lifecycle.rs:31) |


## 4. Capability Index

| Cap-ID | Name | Layer | SS | Shard |
|---|---|---|---|---|
| CAP-01 | PCAP file ingestion | L1 | SS-1 | cap-01-pcap-ingestion.md |
| CAP-02 | Link-type gating | L1 | SS-2 | cap-02-link-type-gating.md |
| CAP-03 | Packet decoding (L2-L4) | L1 | SS-3 | cap-03-packet-decoding.md |
| CAP-04 | TCP stream reassembly | L2 | SS-4 | cap-04-tcp-reassembly.md |
| CAP-05 | Content-first protocol dispatch | L2 | SS-5 | cap-05-content-first-dispatch.md |
| CAP-06 | HTTP traffic analysis | L3 | SS-6 | cap-06-http-analysis.md |
| CAP-07 | TLS traffic analysis | L3 | SS-7 | cap-07-tls-analysis.md |
| CAP-08 | DNS traffic analysis | L3 | SS-8 | cap-08-dns-analysis.md |
| CAP-09 | Forensic finding emission | L3 | SS-9 | cap-09-finding-emission.md |
| CAP-10 | MITRE ATT&CK mapping | L3 | SS-10 | cap-10-mitre-mapping.md |
| CAP-11 | Reporting and output | L4 | SS-11 | cap-11-reporting-output.md |
| CAP-12 | CLI orchestration / entry point | L0 | SS-12 | cap-12-cli-orchestration.md |

**CAP-12 note:** CAP-12 is the L0 cross-cutting orchestration capability. It coordinates
CAP-01..CAP-11 without reimplementing any of them. Components: C-1 (main.rs), C-2 (lib.rs),
C-3 (cli.rs), C-16 (summary.rs). Subsystem SS-12 (CLI / entry-point) maps exclusively to
CAP-12. Behavioral contracts BC-CLI-* and BC-SUM-* are anchored here, not to CAP-11.


## 5. Entity / Enum Index (41 entities, 14 semantic enums)

| Shard | Contents |
|---|---|
| ent-01-ingestion-decoding.md | E-1 Cli, E-2 Commands, E-3 OutputFormat, E-4 RawPacket, E-5 PcapSource, E-6 Protocol, E-7 TransportInfo, E-8 ParsedPacket |
| ent-02-reassembly-flow.md | E-9 FlowKey, E-10 FlowState, E-11 FlowDirection, E-12 TcpFlow, E-13 InsertResult, E-14 Direction, E-15 CloseReason, E-18 ReassemblyConfig, E-19 ReassemblyStats, E-20 TcpReassembler |
| ent-03-dispatch-analysis.md | E-16 StreamHandler (trait), E-17 StreamAnalyzer (trait), E-21 StreamDispatcher, E-22 DispatchTarget, E-29 ProtocolAnalyzer (trait), E-30 DnsAnalyzer, E-31 HttpAnalyzer, E-32 HttpFlowState, E-33 TlsAnalyzer, E-34 TlsFlowState, E-35 SniValue, E-40 ParsedRequest, E-41 ParsedResponse |
| ent-04-findings-output.md | E-23 Verdict, E-24 Confidence, E-25 ThreatCategory, E-26 Finding, E-27 MitreTactic, E-28 AnalysisSummary, E-36 Summary, E-37 Reporter (trait), E-38 TerminalReporter, E-39 JsonReporter |
| ent-05-enums-value-objects.md | All 14 semantic enums; all 12 value objects (VO-1..VO-12) |


## 6. Invariant Index

| Shard | Contents |
|---|---|
| inv-01-core-invariants.md | INV-1 FlowKey canonical ordering; INV-2 Content-first dispatch precedence; INV-3 First-wins overlap policy; INV-4 Raw-data/display-layer separation; INV-5 SNI 4-way classification; INV-6 MAX_FINDINGS cap; INV-7 Finalize-once latch; INV-8 Poison-monotonic HTTP; INV-9 MITRE technique-ID format |


## 7. Domain Glossary

| Term | Definition |
|---|---|
| pcap (classic) | PCAP format with global header; NOT pcapng. wirerust reads only classic pcap. |
| FlowKey | Canonically-ordered (ip_a:port_a <= ip_b:port_b) tuple identifying a TCP connection. |
| Finding | The core output type: a structured forensic observation with category, verdict, confidence, MITRE technique, raw summary, and raw evidence strings. |
| content-first dispatch | Protocol identification by inspecting the first 5 bytes of reassembled TCP data before consulting port numbers (ADR 0001). |
| first-wins overlap | Policy for conflicting TCP segments: the bytes already in the buffer win; the new segment's conflicting bytes are treated as evidence of evasion. |
| poisoned flow | An HTTP flow where parse errors exceeded POISON_THRESHOLD (3 consecutive errors); subsequent bytes are silently counted but not parsed. |
| MAX_FINDINGS | Hard cap of 10,000 on the reassembly engine's findings vec. Analyzer findings (HttpAnalyzer.all_findings, TlsAnalyzer.all_findings) are separately unbounded. |
| SniValue | 4-way enum classifying an SNI hostname: Ascii (clean), AsciiWithControl (C0/DEL present), NonAsciiUtf8 (non-ASCII bytes, valid UTF-8), NonUtf8 (invalid UTF-8). |
| JA3 / JA3S | MD5 fingerprints derived from TLS ClientHello / ServerHello fields, with GREASE values filtered per RFC 8701. |
| escape_for_terminal | The sole C0+DEL+non-CR-LF C1+backslash escape function; only TerminalReporter calls it. |
| finalize() | Explicit cleanup method on TcpReassembler that closes remaining flows and emits the segment-limit summary finding. Must be called by the caller; impl Drop emits a warning eprintln if dropped without calling finalize (added P0.03 / #72). |


## 8. Cross-Reference to Corpus IDs

The following corpus identifiers from the ingestion passes are used throughout the shards:

- BC-RAS-*, BC-DSP-*, BC-HTTP-*, BC-TLS-*, BC-DNS-*, BC-MIT-*, BC-FND-*, BC-RPT-*, BC-CLI-*, BC-SUM-*: Behavioral contract IDs (pass-3 corpus, 218 total)
- C-1..C-21: Component IDs (C-1..C-20 from pass-1 architecture; C-21 = dispatcher.rs added by ADR 0001)
- NFR-PERF/SEC/REL/OBS/RES/MNT/PORT/SUP/COMPAT-NNN: NFR IDs (pass-4 catalog, 79 total)
- NFR-VIO-001..010: Violation IDs (pass-4)
- ADR 0001/0002/0003: Architecture Decision Records (docs/adr/)
- E-1..E-41: Entity IDs (pass-2 domain model)
- VO-1..VO-12: Value object IDs (pass-2 domain model)
- LESSON-P0/P1/P2/P3.*: Action items from pass-8 synthesis

### Subsystem-to-Capability mapping (SS -> CAP)

| Subsystem | Description | Capability |
|---|---|---|
| SS-12 | CLI / entry-point (C-1 main.rs, C-2 lib.rs, C-3 cli.rs, C-16 summary.rs) | CAP-12 |

This table is partial -- it lists only SS-12 here because CAP-12 is the capability added to
resolve adversarial-review finding H-4. Other SS -> CAP mappings are maintained in the
architecture index (ARCH-INDEX). BC-CLI-* and BC-SUM-* are anchored to CAP-12, not CAP-11.


## 9. Known Limitations / Domain Debt

See shard: `domain-debt.md`

This section exists because wirerust is a shipped codebase with observable gaps. The spec
must describe what the system does today. Known gaps are recorded as debt, not silently
omitted or presented as intended behavior.
