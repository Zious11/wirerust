---
document_type: dependency-graph
version: "1.4"
status: draft
producer: story-writer
phase: 3
timestamp: 2026-05-21T00:00:00Z
modified: "2026-06-17: Feature #253 (issue #253) — added STORY-116..117 linear chain (E-17 ARP VLAN/QinQ/MACsec offset hardening). total_stories 67→69 (product; STORY-091 tooling separate). total_edges 91→93 (+2: +1 intra (STORY-116→117) + 1 cross-epic (STORY-115→116)). number_of_waves 44→46."
total_stories: 69
total_edges: 93
intra_epic_edges: 74
cross_epic_edges: 19
number_of_waves: 46
acyclic: true
traces_to:
  - .factory/stories/epics.md
  - .factory/specs/architecture/dependency-graph.md
  - .factory/specs/architecture/module-decomposition.md
  - .factory/specs/verification-properties/VP-INDEX.md
---

# wirerust Story Dependency Graph

> **Brownfield context:** wirerust is a single-crate offline pcap forensic triage CLI.
> All 69 product stories formalize behavioral contracts for existing and new shipped code
> (48 greenfield + F2/F7/F8/F9 feature additions across E-14, E-15, E-16).
> Cross-epic dependencies reflect the architecture pipeline layering
> (L1 Ingest -> L2 Stream -> L3 Domain -> L4 Output -> L0 Entry) defined in
> `architecture/dependency-graph.md` and `architecture/module-decomposition.md`.

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total stories | 69 (product; +STORY-091 tooling = 70) |
| Total dependency edges | 93 |
| Intra-epic edges | 74 |
| Cross-epic edges | 19 |
| Number of parallel waves | 46 |
| Graph is acyclic | Yes (Kahn topological sort verified; STORY-097→098→099 extend acyclic order; STORY-106→107→108→109→110 extend further; STORY-111→112→113→114→115 extend further; STORY-115→116→117 extend further) |
| Total story points | 460 (product; +5 tooling = 465) |

---

## Architecture Subsystem Boundary Rules

Dependencies in this graph respect the layer rules from
`architecture/dependency-graph.md`:

| From Layer | May depend on | Must not depend on |
|-----------|---------------|-------------------|
| L0 Entry (SS-12, SS-13) | Everything | (no restriction) |
| L1 Ingest (SS-01, SS-02) | types only | L3 analyzers, L4 reporters |
| L2 Stream (SS-04) | L3 types via handler.rs traits | L4 reporters |
| L3 Domain (SS-05..08) | findings.rs, mitre.rs | L2 internals, L4 reporters |
| L4 Output (SS-11) | L3 findings/mitre/summary, L2 via summarize | L1 ingest, L2 internals |

> **Accepted L2<->L3 cycle:** `reassembly/handler.rs` defines `StreamHandler`/`StreamAnalyzer`
> traits (L2); `analyzer/http.rs` and `analyzer/tls.rs` implement them (L3). This cycle
> is accepted per ADR 0002. It does not affect story ordering because E-7 (the Finding
> data model) is independently buildable and E-4/E-5 both depend on E-3 (which includes
> handler.rs) and E-7 via their root stories.

---

## Dependencies (Edge List)

### Intra-Epic Edges (74 edges)

#### Epic E-1: PCAP Ingestion and Packet Decoding

| From | To | Justification |
|------|----|---------------|
| STORY-001 | STORY-002 | STORY-001 establishes `PcapSource` and `RawPacket`; STORY-002 decodes IPv4 and requires those types |
| STORY-001 | STORY-003 | STORY-001 establishes `PcapSource` and `RawPacket`; STORY-003 decodes IPv6 and requires those types |
| STORY-001 | STORY-004 | STORY-001 establishes link-type dispatch; STORY-004 builds Linux SLL support on the same dispatch gate |
| STORY-002 | STORY-005 | STORY-005 (integration) exercises Ethernet+IPv4 decode established in STORY-002 |
| STORY-003 | STORY-005 | STORY-005 exercises IPv6 decode established in STORY-003 |
| STORY-004 | STORY-005 | STORY-005 exercises Linux SLL decode established in STORY-004 |

#### Epic E-2: TCP Stream Reassembly Engine

| From | To | Justification |
|------|----|---------------|
| STORY-011 | STORY-012 | STORY-012 segment insertion builds on `TcpFlow` + `FlowKey` defined in STORY-011 |
| STORY-012 | STORY-013 | STORY-013 flush logic builds on `insert_segment` + `BTreeMap` buffer from STORY-012 |
| STORY-013 | STORY-014 | STORY-014 overlap detection requires `flush_contiguous` from STORY-013 |
| STORY-013 | STORY-015 | STORY-015 lifecycle (RST/FIN) also depends on `flush_contiguous` from STORY-013 (confirmed in STORY-015 `depends_on` frontmatter) |
| STORY-013 | STORY-019 | STORY-019 resource pressure management requires flow table from STORY-013 |
| STORY-014 | STORY-015 | STORY-015 lifecycle (RST/FIN) builds on overlap detection from STORY-014 |
| STORY-014 | STORY-019 | STORY-019 uses flow lifecycle events established in STORY-014 |
| STORY-015 | STORY-016 | STORY-016 evasion detection requires completed lifecycle state machine from STORY-015 |
| STORY-015 | STORY-017 | STORY-017 retransmission accounting builds on overlap logic from STORY-015 |
| STORY-015 | STORY-018 | STORY-018 out-of-order handling requires `flush_contiguous` + lifecycle from STORY-015 |
| STORY-016 | STORY-017 | STORY-017 retransmission accounting includes evasion markers from STORY-016 |
| STORY-016 | STORY-018 | STORY-018 out-of-order handling includes evasion context from STORY-016 |
| STORY-019 | STORY-020 | STORY-020 memory ceiling enforcement builds on flow expiry from STORY-019 |
| STORY-017 | STORY-021 | STORY-021 reassembly statistics require retransmission counters from STORY-017 |
| STORY-018 | STORY-021 | STORY-021 statistics require out-of-order counters from STORY-018 |
| STORY-019 | STORY-021 | STORY-021 statistics require resource-pressure counters from STORY-019 |
| STORY-020 | STORY-021 | STORY-021 statistics require memory-ceiling counters from STORY-020 |

#### Epic E-3: Content-First Protocol Dispatch

| From | To | Justification |
|------|----|---------------|
| STORY-031 | STORY-032 | STORY-032 cache and retry logic builds on `classify()` core from STORY-031 |
| STORY-031 | STORY-033 | STORY-033 unclassified flow reporting requires both classify (STORY-031) and cache (STORY-032) |
| STORY-032 | STORY-033 | STORY-033 reporting requires cache/retry state from STORY-032 |

#### Epic E-4: HTTP Traffic Analysis and Threat Detection

| From | To | Justification |
|------|----|---------------|
| STORY-041 | STORY-042 | STORY-042 path-traversal detection builds on request parser from STORY-041 |
| STORY-041 | STORY-043 | STORY-043 web-shell/admin-probe detection uses parsed URI + method from STORY-041 |
| STORY-041 | STORY-044 | STORY-044 method/header anomalies use parsed headers established in STORY-041 |
| STORY-041 | STORY-045 | STORY-045 parse-error isolation requires request buffer model from STORY-041 |
| STORY-044 | STORY-045 | STORY-045 poisoning logic interacts with header-anomaly detection from STORY-044 |
| STORY-042 | STORY-046 | STORY-046 integration requires path-traversal findings from STORY-042 |
| STORY-043 | STORY-046 | STORY-046 integration requires web-shell/admin findings from STORY-043 |
| STORY-044 | STORY-046 | STORY-046 integration requires method/header anomaly findings from STORY-044 |
| STORY-045 | STORY-046 | STORY-046 integration requires parse-error isolation from STORY-045 |
| STORY-041 | STORY-046 | STORY-046 integration-test baseline requires core parser from STORY-041 |

#### Epic E-5: TLS Traffic Analysis and Fingerprinting

| From | To | Justification |
|------|----|---------------|
| STORY-051 | STORY-052 | STORY-052 ServerHello/JA3S builds on JA3 compute functions from STORY-051 |
| STORY-051 | STORY-053 | STORY-053 SNI extraction requires ClientHello parse from STORY-051 |
| STORY-052 | STORY-053 | STORY-053 SNI anomaly uses handshake parse state from STORY-052 |
| STORY-052 | STORY-054 | STORY-054 cipher/protocol findings use extension parse from STORY-052 |
| STORY-053 | STORY-054 | STORY-054 combines SNI findings (STORY-053) with cipher findings |
| STORY-052 | STORY-055 | STORY-055 buffer management builds on handshake state from STORY-052 |
| STORY-052 | STORY-058 | STORY-058 TLS summary requires full handshake data from STORY-052 |
| STORY-053 | STORY-058 | STORY-058 summary includes SNI classification results from STORY-053 |
| STORY-055 | STORY-056 | STORY-056 weak-cipher finding requires buffer health from STORY-055 |
| STORY-055 | STORY-057 | STORY-057 deprecated-protocol finding builds on buffer model from STORY-055 |
| STORY-056 | STORY-057 | STORY-057 deprecated-protocol finding follows weak-cipher (same detection sweep) |

#### Epic E-7: Forensic Finding Data Model and MITRE Mapping

| From | To | Justification |
|------|----|---------------|
| STORY-069 | STORY-070 | STORY-070 skip_serializing_if / raw-data contract builds on `Finding` struct from STORY-069 |
| STORY-069 | STORY-071 | STORY-071 MITRE lookup table requires `ThreatCategory` + technique IDs from STORY-069 |
| STORY-070 | STORY-071 | STORY-071 `MitreTactic` enum resolved via technique_id carried in `Finding` from STORY-070 |

#### Epic E-8: Reporting and Output Formats

| From | To | Justification |
|------|----|---------------|
| STORY-076 | STORY-077 | STORY-077 TerminalReporter escape logic builds on `Reporter` trait from STORY-076 |
| STORY-076 | STORY-079 | STORY-079 CsvReporter structure builds on `Reporter` trait from STORY-076 |
| STORY-077 | STORY-078 | STORY-078 MITRE tactic grouping + colorization builds on escape logic from STORY-077 |
| STORY-079 | STORY-080 | STORY-080 CSV injection neutralization builds on column layout from STORY-079 |

#### Epic E-9: CLI, Entry Point, and Analysis Orchestration

| From | To | Justification |
|------|----|---------------|
| STORY-086 | STORY-087 | STORY-087 reassembly flags build on base `Cli`/`Commands` from STORY-086 |
| STORY-086 | STORY-088 | STORY-088 output format flags build on `Cli` structure from STORY-086 |
| STORY-087 | STORY-088 | STORY-088 includes `ReassemblyConfig` parsing established in STORY-087 |
| STORY-086 | STORY-089 | STORY-089 multi-target + progress-bar logic requires base `Cli` from STORY-086 |
| STORY-087 | STORY-089 | STORY-089 multi-target run uses reassembly config from STORY-087 |
| STORY-088 | STORY-089 | STORY-089 output format dispatch requires output-format flags from STORY-088 |
| STORY-086 | STORY-090 | STORY-090 integration requires base CLI wiring from STORY-086 |
| STORY-088 | STORY-090 | STORY-090 integration exercises output-format dispatch from STORY-088 |
| STORY-089 | STORY-090 | STORY-090 end-to-end integration requires multi-target + pipeline from STORY-089 |

#### Epic E-12: Pcap Timestamp Provenance (issue #100)

| From | To | Justification |
|------|----|---------------|
| STORY-097 | STORY-098 | STORY-098 (emission-site wiring) requires the `on_data timestamp: u32` parameter established in STORY-097; trait-break compile dependency |
| STORY-098 | STORY-099 | STORY-099 (E2E VP-021 verification) requires the implementation from STORY-097 (trait) + STORY-098 (emission sites) to be complete before the integration tests can be written and run |

#### Epic E-15: DNP3/ICS Analyzer (issue #8)

| From | To | Justification |
|------|----|---------------|
| STORY-106 | STORY-107 | STORY-107 (per-flow state + carry buffer) requires `parse_dnp3_dl_header`, `compute_dnp3_frame_len`, `is_valid_dnp3_frame_header`, and `classify_dnp3_fc` pure functions from STORY-106; these types/functions are the basis for all carry-buffer consumption and state tracking |
| STORY-107 | STORY-108 | STORY-108 (direct detection emissions) requires the `Dnp3FlowState` struct with `direct_operate_count`, `restart_event_count`, `pending_requests`, `carry`, `master_addrs_seen`, `frame_count`, and `parse_errors` from STORY-107; no detection can fire without per-flow state (note: T0836 is per-occurrence, no write counter exists; the carry field is `carry`, not `carry_buf`) |
| STORY-108 | STORY-109 | STORY-109 (correlated/derived detections) requires STORY-108's full detection surface (T1692.001, T0814 restart, T0836) plus the `direct_operate_count` / `restart_event_count` windowed counters needed for T0827 correlation; also requires the established `MAX_FINDINGS` cap pattern |
| STORY-109 | STORY-110 | STORY-110 (dispatcher integration + CLI) wires the complete `Dnp3Analyzer` (all detections live, mitre.rs seeded) into `StreamDispatcher`; it cannot do so before the analyzer surface is finalized; also confirms VP-007 catalog counts (SEEDED=23, EMITTED=15) which requires STORY-109's seeding |

#### Epic E-16: ARP Security Analyzer (issue #9)

| From | To | Justification |
|------|----|---------------|
| STORY-111 | STORY-112 | STORY-112 (`ArpAnalyzer` struct + Kani VP-024 Sub-A harnesses) requires `DecodedFrame::Arp(ArpFrame)` type and the revised `decode_packet` → `Result<DecodedFrame>` return type established by STORY-111; compile-break dependency — the struct field types reference `ArpFrame` |
| STORY-112 | STORY-113 | STORY-113 (D2 GARP, D11 malformed, D12 mismatch, binding table, summarize) requires the `ArpAnalyzer` stub skeleton and `process_arp` method signature from STORY-112; the binding-table types (`BindingEntry`, `HashMap<[u8;4], BindingEntry>`) are introduced in STORY-113 itself |
| STORY-113 | STORY-114 | STORY-114 (D1 spoof detection emissions + VP-007 atomic update SEEDED 23→25 / EMITTED 15→17) requires the complete `ArpAnalyzer` detection surface from STORY-113 (binding table, classify_garp, D2/D11/D12 detectors) before D1 findings can be emitted and MITRE catalog seeded; also requires `summarize()` key layout from STORY-113 for VP-007 catalog alignment |
| STORY-114 | STORY-115 | STORY-115 (D3 storm detection + `--arp-storm-rate` CLI flag + `storm_findings` summary key) finalizes the complete `ArpAnalyzer` (all detections live); it cannot do so before the analyzer's full detection surface and emission contract are finalized in STORY-114; also wires the VALUE of the existing BC-2.16.010 `storm_findings` key (canonical key 8, declared by STORY-113) in `summarize()` (ARP bypasses `classify()`/`StreamDispatcher` — arp-architecture-delta §4.4) |

#### Epic E-17: ARP Decoder VLAN/QinQ/MACsec Offset Hardening (issue #253)

| From | To | Justification |
|------|----|---------------|
| STORY-116 | STORY-117 | STORY-117 (MACsec observe-only probe + documented limitation AC) requires STORY-116's QinQ fixture infrastructure — the test module scaffolding and pcap fixture loading patterns established in STORY-116 are reused by STORY-117; also a logical sequencing: VLAN/QinQ offset coverage must land before MACsec observe-only probe to avoid test file conflicts on the same `tests/arp_offset_*.rs` module |

---

### Cross-Epic Edges (19 edges)

> **Note:** The E-17 intra-epic edge (STORY-116 → STORY-117) is listed under Intra-Epic Edges above. The E-16 → E-17 boundary edge (STORY-115 → STORY-116) appears in the table below.

These edges reflect the architecture pipeline layers defined in
`architecture/dependency-graph.md` and `architecture/module-decomposition.md`.

| From | To | Epic | Subsystem Boundary | Justification |
|------|----|------|--------------------|---------------|
| STORY-005 | STORY-011 | E-1 -> E-2 | SS-01/02 -> SS-04 | SS-01/02 decoder's `ParsedPacket` is the input type consumed by `TcpReassembler.process_packet()` in SS-04; L1 Ingest feeds L2 Stream |
| STORY-005 | STORY-066 | E-1 -> E-6 | SS-01/02 -> SS-08 | SS-08 `DnsAnalyzer.analyze()` receives `ParsedPacket` directly from the L1 ingest pipeline (packet-level, not stream-level; bypasses E-2/E-3) |
| STORY-021 | STORY-031 | E-2 -> E-3 | SS-04 -> SS-05 | SS-04 reassembly emits stream data via the `StreamHandler`/`StreamAnalyzer` trait interface defined in `reassembly/handler.rs`; SS-05 `StreamDispatcher` implements `StreamHandler` and consumes that interface |
| STORY-033 | STORY-041 | E-3 -> E-4 | SS-05 -> SS-06 | SS-05 `StreamDispatcher.on_data()` routes classified TCP streams to `HttpAnalyzer` (SS-06); STORY-041 builds `HttpAnalyzer` which must conform to the `StreamAnalyzer` trait from STORY-033 |
| STORY-033 | STORY-051 | E-3 -> E-5 | SS-05 -> SS-07 | SS-05 `StreamDispatcher.on_data()` routes classified TCP streams to `TlsAnalyzer` (SS-07); STORY-051 builds `TlsAnalyzer` which must conform to the `StreamAnalyzer` trait from STORY-033 |
| STORY-071 | STORY-041 | E-7 -> E-4 | SS-09/10 -> SS-06 | SS-06 `HttpAnalyzer` emits `Finding` structs using `ThreatCategory` + `MitreTactic` types established by E-7; STORY-041 requires those types before any finding can be constructed |
| STORY-071 | STORY-051 | E-7 -> E-5 | SS-09/10 -> SS-07 | SS-07 `TlsAnalyzer` emits `Finding` structs using `ThreatCategory` + `MitreTactic` types established by E-7; STORY-051 requires those types before any finding can be constructed |
| STORY-046 | STORY-076 | E-4 -> E-8 | SS-06 -> SS-11 | SS-11 reporters consume `Vec<Finding>` produced by `HttpAnalyzer`; STORY-076 (`JsonReporter`) requires the complete `Finding` contract validated by STORY-046 |
| STORY-057 | STORY-076 | E-5 -> E-8 | SS-07 -> SS-11 | SS-11 reporters consume `Vec<Finding>` produced by `TlsAnalyzer` stream stories; STORY-057 is the last stream-finding story in E-5 |
| STORY-058 | STORY-076 | E-5 -> E-8 | SS-07 -> SS-11 | SS-11 reporters consume the `TlsAnalyzer` summary struct established in STORY-058 |
| STORY-066 | STORY-076 | E-6 -> E-8 | SS-08 -> SS-11 | SS-11 reporters include DNS statistics in their `analyzers` output section; STORY-076 requires the `DnsAnalyzer.summarize()` contract from STORY-066 |
| STORY-071 | STORY-076 | E-7 -> E-8 | SS-09/10 -> SS-11 | SS-11 `TerminalReporter` groups findings by `MitreTactic` (from SS-10) and renders `Verdict`/`Confidence` display tokens (from SS-09); STORY-076 requires the fully-specified `Finding` + `MitreTactic` types |
| STORY-080 | STORY-086 | E-8 -> E-9 | SS-11 -> SS-12 | SS-12 `run_analyze()` (L0 Entry) selects and dispatches to the correct `Reporter` (JsonReporter/TerminalReporter/CsvReporter) established in E-8; transitive coverage of E-1..E-7 is already guaranteed via STORY-080's ancestry |
| STORY-086 | STORY-096 | E-9 -> E-10 | SS-12 -> SS-13 | SS-13 absent-behavior tests verify that removed flags are rejected by the `Cli` struct defined in STORY-086; the test vehicle is the CLI binary |
| STORY-100 | STORY-106 | E-13 -> E-15 | SS-10 -> SS-15 | SS-15 `Dnp3Analyzer` emits `Finding` structs using `mitre_techniques: Vec<String>` (multi-tag schema) established by E-13 (STORY-100); STORY-106 defines the `Dnp3Analyzer` type which depends on the post-migration Finding struct |
| STORY-105 | STORY-110 | E-14 -> E-15 | SS-05 (Modbus Rule 5) -> SS-05 (DNP3 Rule 6) | STORY-110 adds `DispatchTarget::Dnp3` as Rule 6 in `classify()`, placed after Rule 5 (Modbus/502) from STORY-105; Rule 6 ordering is only meaningful once Rule 5 is established — and the VP-004 oracle must include the Modbus arm before the DNP3 arm can be added correctly |
| STORY-105 | STORY-109 | E-14 -> E-15 | SS-10 mitre.rs | STORY-109 adds T1691.001 and T0827 to `SEEDED_TECHNIQUE_IDS` and the kani_proofs `EMITTED_IDS` set in `src/mitre.rs`; the Modbus seeding in STORY-105 sets the catalog counts to 21/13; STORY-109's obligation is 21→23 / 13→15 — this is a logical dependency on the prior state of mitre.rs |
| STORY-110 | STORY-111 | E-15 -> E-16 | SS-02 (decoder) | STORY-111 migrates etherparse 0.16→0.20 and introduces `DecodedFrame::Arp(ArpFrame)` in `src/decoder.rs`; it also revises BC-2.02.009 to add the third decode path; the migration touches both `src/decoder.rs` and `src/dispatcher.rs` (Cargo.toml etherparse version bump affects both files) and must follow STORY-110's finalized `src/dispatcher.rs` changes (DNP3 Rule 6 in place) to avoid merge conflicts on the same file — file-level sequencing constraint, not a classify() ordering requirement |
| STORY-115 | STORY-116 | E-16 -> E-17 | SS-16 (ARP lax-path) | STORY-116 adds VLAN/QinQ offset regression tests that exercise `extract_arp_frame` and the ARP lax-path in `src/decoder.rs`; STORY-115 must be merged first because it finalizes all ARP decode-time logic in `src/decoder.rs` and `src/main.rs` that STORY-116's fixture tests exercise; file-sequencing + behavioral contract completeness (BC-2.16.009 EC-008, BC-2.16.015 PC-7a) — offset hardening stories test code that must be fully landed first |

---

## Independent Groups (Wave Schedule)

Waves are computed as `wave(story) = max(wave(dependency)) + 1` (longest-path /
critical-path method). Stories in the same wave have no dependency between them
and can be dispatched in parallel.

> **Graph is acyclic:** Kahn's algorithm processes all 69 product stories. No cycle detected.

### Wave 1 — 2 stories | Epics: E-1, E-7

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-001 | E-1 | 5 | SS-01 | PCAP File Ingestion — Link-Type Gating, Eager Packet Load, and Error Surfaces |
| STORY-069 | E-7 | 5 | SS-09 | Finding Struct, Verdict/Confidence Display, and Finding Display Format |

> **Rationale:** E-1 and E-7 are independently buildable. E-7's data model (pure types)
> has no runtime dependency on anything in the pipeline.

### Wave 2 — 4 stories | Epics: E-1, E-7

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-002 | E-1 | 5 | SS-02 | Packet Decoding — Ethernet, RAW/IPV4, and IPv6 Link-Layer Paths |
| STORY-003 | E-1 | 5 | SS-02 | Packet Decoding — Linux SLL, No-Panic Safety, and Non-IP Frame Rejection |
| STORY-004 | E-1 | 3 | SS-02 | Packet Decoding — ICMP, Protocol::Other, and app_protocol_hint Port Table |
| STORY-070 | E-7 | 5 | SS-09 | Raw-Data Contract and JSON Serialization Symmetry (skip_serializing_if) |

### Wave 3 — 2 stories | Epics: E-1, E-7

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-005 | E-1 | 3 | SS-01/02 | Packet Decoding — packet_len Semantics and TCP Flag/Sequence Extraction |
| STORY-071 | E-7 | 8 | SS-10 | MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order |

### Wave 4 — 2 stories | Epics: E-2, E-6

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-011 | E-2 | 5 | SS-04 | TcpReassembler Constructor Validation and FlowKey Canonicalization |
| STORY-066 | E-6 | 5 | SS-08 | DNS Traffic Statistics — Port-53 Dispatch, QR-Bit Counting, and Never-Emit Contract |

> **Note:** STORY-066 (DNS, packet-level) and STORY-011 (TCP reassembly) are both
> unblocked once STORY-005 completes. They can proceed in parallel.

### Wave 5 — 1 story | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-012 | E-2 | 5 | SS-04 | Non-TCP Packet Filter, Statistics Summary, and bytes_reassembled Accounting |

### Wave 6 — 1 story | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-013 | E-2 | 8 | SS-04 | TCP Three-Way Handshake State Machine and Direction Tagging |

### Wave 7 — 1 story | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-014 | E-2 | 5 | SS-04 | Mid-Stream Join, ISN Management, and IsnMissing Guard |

### Wave 8 — 2 stories | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-015 | E-2 | 8 | SS-04 | In-Order Delivery, Out-of-Order Buffering, and Bidirectional Direction Tagging |
| STORY-019 | E-2 | 5 | SS-04 | Flow Lifecycle — RST Close, FIN Close, Timeout Expiry, and Missing-Key Warning |

> **Note:** STORY-015 and STORY-019 share only STORY-013/STORY-014 as common ancestors.
> They touch different parts of SS-04 (`lifecycle.rs` vs resource management) and can
> run in parallel.

### Wave 9 — 2 stories | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-016 | E-2 | 8 | SS-04 | Overlap Detection — Duplicate Retransmissions, Partial Overlap, and buffered_bytes Accounting |
| STORY-020 | E-2 | 8 | SS-04 | Memory Management — total_memory Accounting and LRU Eviction Policies |

### Wave 10 — 2 stories | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-017 | E-2 | 8 | SS-04 | Conflict and Evasion Detection — T1036 Findings and One-Shot Anomaly Latches |
| STORY-018 | E-2 | 8 | SS-04 | Resource Bounds — Depth Truncation, Out-of-Window Rejection, and Segment Limit Enforcement |

### Wave 11 — 1 story | Epic: E-2

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-021 | E-2 | 5 | SS-04 | Finalize Lifecycle, MAX_FINDINGS Cap, and Segment-Limit Summary Finding |

### Wave 12 — 1 story | Epic: E-3

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-031 | E-3 | 5 | SS-05 | Content-First Classification — TLS Signature, HTTP Method Prefix, Port Fallback |

### Wave 13 — 1 story | Epic: E-3

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-032 | E-3 | 5 | SS-05 | Classification Caching and DispatchTarget::None Retry Budget |

### Wave 14 — 1 story | Epic: E-3

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-033 | E-3 | 3 | SS-05 | Flow Lifecycle — Close, Unclassified Counter, No-Op Dispatcher |

### Wave 15 — 2 stories | Epics: E-4, E-5

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-041 | E-4 | 8 | SS-06 | HTTP/1.1 Request/Response Parsing and Core Statistics |
| STORY-051 | E-5 | 5 | SS-07 | JA3 and JA3S Computation — GREASE Filtering and String Format |

> **Note:** E-4 and E-5 root stories both depend on STORY-033 and STORY-071.
> They can proceed in parallel once both predecessors are complete.

### Wave 16 — 4 stories | Epics: E-4, E-5

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-042 | E-4 | 5 | SS-06 | URI-Based Threat Detections — Path Traversal, Web Shell, Admin Panel |
| STORY-043 | E-4 | 5 | SS-06 | Header and Method Anomaly Detections — Method, Host, URI Length, User-Agent |
| STORY-044 | E-4 | 8 | SS-06 | Parse-Error Isolation and Poisoning State Machine |
| STORY-052 | E-5 | 8 | SS-07 | ClientHello Parsing — Handshake Counting, Version/JA3 Tracking, and Done Short-Circuit |

### Wave 17 — 3 stories | Epics: E-4, E-5

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-045 | E-4 | 5 | SS-06 | Flow Lifecycle, Cross-Flow Isolation, and Buffer/Map Caps |
| STORY-053 | E-5 | 5 | SS-07 | ServerHello Parsing — JA3S Fingerprinting and Cipher/Version Tracking |
| STORY-055 | E-5 | 8 | SS-07 | SNI Classification Arms 1 and 2 — Clean ASCII Baseline and C0/DEL Control-Byte Detection |

### Wave 18 — 4 stories | Epics: E-4, E-5

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-046 | E-4 | 3 | SS-06 | HTTP Analyzer Summary Output |
| STORY-054 | E-5 | 8 | SS-07 | Cipher and Protocol Weakness Findings — Weak Ciphers, Deprecated SSL Versions, and Baseline Zero-Finding |
| STORY-056 | E-5 | 8 | SS-07 | SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte Preservation |
| STORY-058 | E-5 | 8 | SS-07 | Buffer Management, Record Parsing Infrastructure, Flow Lifecycle, and summarize Output |

### Wave 19 — 1 story | Epic: E-5

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-057 | E-5 | 8 | SS-07 | SNI Edge Cases — Empty Lists, Empty Hostnames, Multi-Name, NameType, Trailing Bytes, Large SNI, and Count-Cap Decoupling |

### Wave 20 — 1 story | Epic: E-8

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-076 | E-8 | 5 | SS-11 | JsonReporter — Structure, skipped_packets, and RFC 8259 Byte Handling |

### Wave 21 — 2 stories | Epic: E-8

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-077 | E-8 | 8 | SS-11 | TerminalReporter — escape_for_terminal, skipped_packets, and End-to-End C1 Safety |
| STORY-079 | E-8 | 5 | SS-11 | CsvReporter — Fixed 9-Column Schema, CSV-Injection Neutralization, and Evidence Join |

### Wave 22 — 2 stories | Epic: E-8

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-078 | E-8 | 8 | SS-11 | TerminalReporter — MITRE Grouping, Section Order, and Colorization |
| STORY-080 | E-8 | 3 | SS-11 | CsvReporter — Reporter Trait Compliance and Optional Field Encoding |

### Wave 23 — 1 story | Epic: E-9

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-086 | E-9 | 5 | SS-12 | CLI Subcommand Parsing — analyze, summary, --no-color, Multiple Targets |

### Wave 24 — 2 stories | Epics: E-9, E-10

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-087 | E-9 | 5 | SS-12 | Output Format Flags and Reassembly Configuration Flags |
| STORY-096 | E-10 | 3 | SS-13 | Absent Behavior Contracts — Removed Flags Rejected by clap |

> **Note:** STORY-096 can proceed in parallel with STORY-087 since both only require
> the base CLI from STORY-086.

### Wave 25 — 1 story | Epic: E-9

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-088 | E-9 | 8 | SS-12 | run_analyze Orchestration — Analyzer Enablement, Reassembly Logic, Target Expansion, Progress Bar |

### Wave 26 — 1 story | Epic: E-9

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-089 | E-9 | 5 | SS-12 | Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing |

### Wave 27 — 1 story | Epic: E-9

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-090 | E-9 | 5 | SS-12 | Summary Data Model — ingest, Service Hints, unique_hosts, Serialization |

### Wave 28 — 1 story | Epic: E-12

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-097 | E-12 | 5 | SS-04, SS-05 | Thread Capture-Relative Timestamp Through StreamHandler::on_data |

### Wave 29 — 1 story | Epic: E-12

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-098 | E-12 | 8 | SS-06, SS-07, SS-04, SS-09 | Attach Pcap Timestamp to Emitted Findings |

### Wave 30 — 1 story | Epic: E-12

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-099 | E-12 | 5 | SS-09, SS-04, SS-06, SS-07 | Verify Timestamp Provenance End-to-End (VP-021) |

### Wave 31 — 2 stories | Epic: E-13

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-100 | E-13 | 13 | SS-09, SS-10, SS-06, SS-07, SS-04 | Multi-Tag Finding Schema Migration (Atomic Type Rename + Catalog Seed) |
| STORY-101 | E-13 | 8 | SS-11 | Multi-Tag Reporter Serialization + JSON Envelope Add-Ons |

> **Note:** STORY-100 and STORY-101 can be parallelized once STORY-100's type-rename PR merges. Wave 31 gate closes the v0.3.0 release.

### Wave 32 — 1 story | Epic: E-14

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-102 | E-14 | 8 | SS-14 | Modbus MBAP Parse + FC Classification (Pure Core) |

> **Note:** STORY-102 depends on STORY-100 (multi-tag Finding schema). Root of the E-14 Modbus chain.

### Wave 33 — 2 stories | Epic: E-14

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-103 | E-14 | 8 | SS-14 | Modbus Flow State + Transaction Correlation |
| STORY-104 | E-14 | 13 | SS-14 | Modbus Detection Emissions + Summary |

> **Note:** STORY-104 depends on STORY-103 (flow state required for detection emissions).

### Wave 34 — 1 story | Epic: E-14

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-105 | E-14 | 8 | SS-14, SS-05, SS-12 | Modbus Dispatcher Integration + CLI |

> **Release gate:** v0.4.0 ships after Wave 34 gate (STORY-102..105 all PRs merged, `cargo test --all-targets` green).

### Wave 35 — 1 story | Epic: E-15

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-106 | E-15 | 8 | SS-15 | DNP3 DL/Transport Parse + FC Classify — Pure Core (VP-023 Kani) |

> **Note:** STORY-106 depends on STORY-100 (multi-tag Finding schema). It is the root of the E-15 chain and can proceed once Wave 31 completes.

### Wave 36 — 1 story | Epic: E-15

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-107 | E-15 | 5 | SS-15 | DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds |

### Wave 37 — 1 story | Epic: E-15

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-108 | E-15 | 13 | SS-15 | DNP3 Direct Detection Emissions — T1692.001, T0814 (Restart), T0836, Co-Emission, Summarize |

### Wave 38 — 1 story | Epic: E-15

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-109 | E-15 | 13 | SS-15 | DNP3 Correlated/Derived + Anomaly Detections — T1691.001, T0827, Broadcast, Unsolicited, ENABLE/DISABLE, Malformed |

> **VP-007 obligation:** T1691.001 + T0827 added to SEEDED/EMITTED in this wave. SEEDED 21→23, EMITTED 13→15.

### Wave 39 — 1 story | Epics: E-15, E-3

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-110 | E-15 | 8 | SS-05, SS-15 | DNP3 Dispatcher Integration + CLI Flag — VP-004 Oracle + VP-007 Atomic-Update |

> **VP-004 obligation:** `classify_oracle` gains port-20000 arm in this wave. `verify_content_first_precedence_exhaustive` Kani proof updated.

### Wave 40 — 1 story | Epic: E-16

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-111 | E-16 | 5 | SS-02 | etherparse 0.20 Migration + DecodedFrame/ArpFrame Types + BC-2.02.009 Revision |

> **Note:** STORY-111 depends on STORY-110 (cross-epic: `src/dispatcher.rs` and Cargo.toml etherparse migration must follow STORY-110's finalized DNP3 changes to avoid merge conflicts; file-sequencing constraint). Root of the E-16 strictly-linear chain.

### Wave 41 — 1 story | Epic: E-16

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-112 | E-16 | 8 | SS-02, SS-16 | extract_arp_frame + decode_packet ARP Routing (Both Paths) + ArpAnalyzer Stub + VP-024 Sub-A |

> **VP-024 Sub-A obligation:** 3 Kani harnesses (safety, eth_ipv4_correctness, none_on_bad_size) land in this wave.

### Wave 42 — 1 story | Epic: E-16

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-113 | E-16 | 13 | SS-16 | ArpAnalyzer Full Implementation — Binding Table, GARP (D2), D11, D12, summarize(), --arp Flag, VP-024 Sub-B/C/D |

> **VP-024 Sub-B/C/D obligations:** `verify_classify_garp_total` Kani (Sub-B), `test_binding_table_last_write_wins` proptest (Sub-C), `verify_binding_table_cap` Kani (Sub-D) land in this wave.

### Wave 43 — 1 story | Epic: E-16

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-114 | E-16 | 13 | SS-16 | D1 ARP Spoof Escalation + GARP-that-Conflicts (D2+D1) + MITRE Attribution + VP-007 5-Part Atomic Update |

> **VP-007 obligation:** ARP techniques added to `SEEDED_TECHNIQUE_IDS` and `EMITTED_IDS` in `src/mitre.rs`; SEEDED 23→25, EMITTED 15→17. `vp007_catalog_drift_guard` must pass.

### Wave 44 — 1 story | Epics: E-16

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-115 | E-16 | 8 | SS-02, SS-16 | D3 ARP Storm Detection + `--arp-storm-rate` CLI Flag + `storm_findings` Summary Key |

> **Release gate:** v0.7.0 ships after Wave 44 gate (STORY-111..115 all PRs merged, `cargo test --all-targets` green). `--arp` CLI flag (STORY-113, BC-2.16.011) gates ARP analysis at decode-time — `main.rs` routes the `DecodedFrame::Arp` arm to `ArpAnalyzer`; ARP bypasses `classify()`/`StreamDispatcher` (arp-architecture-delta §4.4).

### Wave 45 — 1 story | Epic: E-17

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-116 | E-17 | 3 | SS-16 | ARP QinQ (Double-Tag) Decoder Offset Coverage — QinQ 22-byte offset (EC-008) regression + MACsec observe-only probe |

> **Note:** STORY-116 depends on STORY-115 (cross-epic: E-16 ARP decode-time logic must be fully shipped before offset regression tests can be authored against it). Root of the E-17 linear chain. tdd_mode: facade — delivers test files only, no production code change.

### Wave 46 — 1 story | Epic: E-17

| Story | Epic | Points | Subsystem | Description |
|-------|------|--------|-----------|-------------|
| STORY-117 | E-17 | 5 | SS-16 | ARP MACsec Observe-Only Probe + Documented Limitation (EC-009) |

> **Release gate:** v0.7.1 ships after Wave 46 gate (STORY-116 + STORY-117 PRs merged, `cargo test --all-targets` green). tdd_mode: facade — delivers test files only, no production code change.

---

## Topological Order (Full Sequence)

```
STORY-001 -> STORY-069 -> STORY-002 -> STORY-003 -> STORY-004 -> STORY-070 ->
STORY-005 -> STORY-071 -> STORY-011 -> STORY-066 -> STORY-012 -> STORY-013 ->
STORY-014 -> STORY-015 -> STORY-019 -> STORY-016 -> STORY-020 -> STORY-017 ->
STORY-018 -> STORY-021 -> STORY-031 -> STORY-032 -> STORY-033 -> STORY-041 ->
STORY-051 -> STORY-042 -> STORY-043 -> STORY-044 -> STORY-052 -> STORY-045 ->
STORY-053 -> STORY-055 -> STORY-046 -> STORY-054 -> STORY-058 -> STORY-056 ->
STORY-057 -> STORY-076 -> STORY-077 -> STORY-079 -> STORY-078 -> STORY-080 ->
STORY-086 -> STORY-087 -> STORY-096 -> STORY-088 -> STORY-089 -> STORY-090 ->
[Wave 28-30: STORY-097 -> STORY-098 -> STORY-099] ->
[Wave 31: STORY-100 ∥ STORY-101] ->
[Wave 32: STORY-102] -> [Wave 33: STORY-103 ∥ STORY-104] -> [Wave 34: STORY-105] ->
STORY-106 -> STORY-107 -> STORY-108 -> STORY-109 -> STORY-110 ->
STORY-111 -> STORY-112 -> STORY-113 -> STORY-114 -> STORY-115 ->
STORY-116 -> STORY-117
```

> **Cycle check:** All 69 product nodes processed by Kahn's algorithm. No node remained
> in the queue with non-zero in-degree after processing. Graph is acyclic.
> E-15 chain (STORY-106→107→108→109→110) is strictly linear; STORY-106 depends on
> STORY-100 (cross-epic), STORY-110 depends on STORY-105 (cross-epic for VP-004 oracle
> ordering). E-16 chain (STORY-111→112→113→114→115) is strictly linear; STORY-111
> depends on STORY-110 (cross-epic: dispatcher file ordering constraint). E-17 chain
> (STORY-116→117) is strictly linear; STORY-116 depends on STORY-115 (cross-epic:
> E-16 ARP decode-time logic must be shipped before offset regression tests). No
> back-edges into the existing 69-story graph.

---

## Acyclicity Proof

Kahn's algorithm processes nodes by removing zero-in-degree nodes from the graph
iteratively. Result:

- Initial zero-in-degree nodes: STORY-001, STORY-069 (Wave 1)
- Each wave removes its stories and decrements successor in-degrees
- Final output: all 69 product stories processed, queue empty, no cycle detected
- Any cycle would leave unprocessed nodes with non-zero in-degree — none found
- E-15 extension (STORY-106→107→108→109→110) is a linear tail appended after Wave 34;
  it shares two cross-epic edges (STORY-100→106, STORY-105→110) that add in-degrees
  only to E-15 nodes — no existing node gains a new in-degree, so no cycle is possible
- E-16 extension (STORY-111→112→113→114→115) is a linear tail appended after Wave 39;
  it has one cross-epic edge (STORY-110→111) that adds in-degree only to the E-16 root
  node — no existing node gains a new in-degree, so no cycle is possible
- E-17 extension (STORY-116→117) is a linear tail appended after Wave 44;
  it has one cross-epic edge (STORY-115→116) that adds in-degree only to the E-17 root
  node — no existing node gains a new in-degree, so no cycle is possible

---

## BC to Stories Traceability Matrix

| BC Range | Story | Epic | Subsystem |
|----------|-------|------|-----------|
| BC-2.01.001..008 | STORY-001 | E-1 | SS-01 |
| BC-2.02.001..005 | STORY-002 | E-1 | SS-02 |
| BC-2.02.006..009 | STORY-003 | E-1 | SS-02 |
| BC-2.02.010..013 | STORY-004 | E-1 | SS-02 |
| BC-2.02.014..015 | STORY-005 | E-1 | SS-01/02 |
| BC-2.04.001, BC-2.04.003, BC-2.04.049 | STORY-011 | E-2 | SS-04 |
| BC-2.04.002, BC-2.04.028, BC-2.04.030 | STORY-012 | E-2 | SS-04 |
| BC-2.04.004, BC-2.04.005, BC-2.04.050..053 | STORY-013 | E-2 | SS-04 |
| BC-2.04.009, BC-2.04.031, BC-2.04.032, BC-2.04.048 | STORY-014 | E-2 | SS-04 |
| BC-2.04.006..008, BC-2.04.033, BC-2.04.034, BC-2.04.039 | STORY-015 | E-2 | SS-04 |
| BC-2.04.035, BC-2.04.036, BC-2.04.038, BC-2.04.043, BC-2.04.047 | STORY-016 | E-2 | SS-04 |
| BC-2.04.018..022, BC-2.04.037 | STORY-017 | E-2 | SS-04 |
| BC-2.04.023, BC-2.04.027, BC-2.04.040..042, BC-2.04.044..046 | STORY-018 | E-2 | SS-04 |
| BC-2.04.010, BC-2.04.011, BC-2.04.013, BC-2.04.029 | STORY-019 | E-2 | SS-04 |
| BC-2.04.014..017 | STORY-020 | E-2 | SS-04 |
| BC-2.04.012, BC-2.04.024..026, BC-2.04.054 | STORY-021 | E-2 | SS-04 |
| BC-2.05.001..003 | STORY-031 | E-3 | SS-05 |
| BC-2.05.004..006 | STORY-032 | E-3 | SS-05 |
| BC-2.05.007..009 | STORY-033 | E-3 | SS-05 |
| BC-2.06.001..004, BC-2.06.026 | STORY-041 | E-4 | SS-06 |
| BC-2.06.005..007, BC-2.06.012 | STORY-042 | E-4 | SS-06 |
| BC-2.06.008..011 | STORY-043 | E-4 | SS-06 |
| BC-2.06.013..018, BC-2.06.020 | STORY-044 | E-4 | SS-06 |
| BC-2.06.019, BC-2.06.021, BC-2.06.022, BC-2.06.024, BC-2.06.025 | STORY-045 | E-4 | SS-06 |
| BC-2.06.023 | STORY-046 | E-4 | SS-06 |
| BC-2.07.006..008 | STORY-051 | E-5 | SS-07 |
| BC-2.07.001, BC-2.07.003, BC-2.07.032, BC-2.07.034 | STORY-052 | E-5 | SS-07 |
| BC-2.07.002 | STORY-053 | E-5 | SS-07 |
| BC-2.07.009..012, BC-2.07.030, BC-2.07.036 | STORY-054 | E-5 | SS-07 |
| BC-2.07.013..016, BC-2.07.018 | STORY-055 | E-5 | SS-07 |
| BC-2.07.017, BC-2.07.019..021, BC-2.07.037 | STORY-056 | E-5 | SS-07 |
| BC-2.07.022..028 | STORY-057 | E-5 | SS-07 |
| BC-2.07.004, BC-2.07.005, BC-2.07.029, BC-2.07.031, BC-2.07.033, BC-2.07.035 | STORY-058 | E-5 | SS-07 |
| BC-2.08.001..004 | STORY-066 | E-6 | SS-08 |
| BC-2.09.001..004 | STORY-069 | E-7 | SS-09 |
| BC-2.09.005..006 | STORY-070 | E-7 | SS-09 |
| BC-2.10.001..009 | STORY-071 | E-7 | SS-10 |
| BC-2.11.001..005 | STORY-076 | E-8 | SS-11 |
| BC-2.11.006..012 | STORY-077 | E-8 | SS-11 |
| BC-2.11.013..019 | STORY-078 | E-8 | SS-11 |
| BC-2.11.020..022 | STORY-079 | E-8 | SS-11 |
| BC-2.11.023..024 | STORY-080 | E-8 | SS-11 |
| BC-2.12.001..003, BC-2.12.006 | STORY-086 | E-9 | SS-12 |
| BC-2.12.004, BC-2.12.005, BC-2.12.007 | STORY-087 | E-9 | SS-12 |
| BC-2.12.008..013 | STORY-088 | E-9 | SS-12 |
| BC-2.12.014..017 | STORY-089 | E-9 | SS-12 |
| BC-2.12.018..021 | STORY-090 | E-9 | SS-12 |
| BC-2.13.001..004 | STORY-096 | E-10 | SS-13 |
| BC-2.04.055 | STORY-097, STORY-098, STORY-099 | E-12 | SS-04 |
| BC-2.09.007 | STORY-098, STORY-099 | E-12 | SS-09 |
| BC-2.15.001..005 | STORY-106 | E-15 | SS-15 |
| BC-2.15.006..009 | STORY-106 | E-15 | SS-15 |
| BC-2.15.016 | STORY-107 | E-15 | SS-15 |
| BC-2.15.010, BC-2.15.011, BC-2.15.012, BC-2.15.013, BC-2.15.020, BC-2.15.022 | STORY-108 | E-15 | SS-15 |
| BC-2.15.014, BC-2.15.015, BC-2.15.018, BC-2.15.019, BC-2.15.023, BC-2.15.024 | STORY-109 | E-15 | SS-15 |
| BC-2.15.017, BC-2.15.021 | STORY-110 | E-15 | SS-05, SS-15 |
| BC-2.02.009 (revised v1.6) | STORY-111 | E-16 / E-1 | SS-02 |
| BC-2.16.001, BC-2.16.002, BC-2.16.015 | STORY-112 | E-16 | SS-16 |
| BC-2.16.003, BC-2.16.005, BC-2.16.006, BC-2.16.007, BC-2.16.009, BC-2.16.010, BC-2.16.011 | STORY-113 | E-16 | SS-16 |
| BC-2.16.004, BC-2.16.012, BC-2.16.014 (+BC-2.16.007 D12-MITRE extension) | STORY-114 | E-16 | SS-16 |
| BC-2.16.008, BC-2.16.013 (+BC-2.16.010 extension) | STORY-115 | E-16 | SS-02, SS-16 |
| BC-2.16.009 (EC-008 QinQ offset 22, EC-009 MACsec observe-only probe — v1.10 additions) | STORY-116, STORY-117 | E-17 | SS-16 |
| BC-2.16.015 (PC-7a QinQ offset 22, EC-008, EC-009 — v1.9 additions) | STORY-116, STORY-117 | E-17 | SS-16 |

**Coverage: 283 / 283 BCs assigned (219 pre-feature + 25 Modbus BC-2.14.001..025 + 24 DNP3 BC-2.15.001..024 across STORY-106..110 + 15 ARP BC-2.16.001..015 across STORY-112..115; BC-2.02.009 is revised, not a new BC). E-17 (STORY-116..117) adds regression-test coverage to BC-2.16.009 and BC-2.16.015 (v1.10 / v1.9 EC-008/EC-009 additions); no new BCs introduced — the E-17 stories deepen coverage of two BCs already counted in the 283 total.**

---

## VP to Stories Matrix

| VP | Title | Module | Stories Exercising It | BC Source |
|----|-------|--------|----------------------|-----------|
| VP-001 | FlowKey Canonical Ordering | reassembly/flow.rs | STORY-011, STORY-013 | BC-2.04.003 (STORY-011), BC-2.04.053 (STORY-013) |
| VP-002 | First-Wins Overlap Policy | reassembly/segment.rs | STORY-016, STORY-017 | BC-2.04.035, BC-2.04.036, BC-2.04.038, BC-2.04.043 (STORY-016); BC-2.04.018, BC-2.04.037 (STORY-017) |
| VP-003 | MAX_FINDINGS Cap with Finalize Bypass | reassembly/mod.rs | STORY-021 | BC-2.04.024, BC-2.04.054 |
| VP-004 | Content-First Dispatch Precedence | dispatcher.rs | STORY-031, STORY-032 | BC-2.05.001..003 (STORY-031); BC-2.05.004..006 (STORY-032) |
| VP-005 | SNI 4-Way Ordered Classification | analyzer/tls.rs | STORY-055, STORY-056 | BC-2.07.013..016 (STORY-055); BC-2.07.017, BC-2.07.019, BC-2.07.037 (STORY-056) |
| VP-006 | HTTP Poison Monotonicity | analyzer/http.rs | STORY-044 | BC-2.06.015, BC-2.06.016, BC-2.06.017 |
| VP-007 | MITRE Technique ID Format and Catalog Completeness | mitre.rs | STORY-071 | BC-2.10.005..008 |
| VP-008 | decode_packet Never Panics on Arbitrary Input | decoder.rs | STORY-003 | BC-2.02.007, BC-2.02.008, BC-2.02.009 |
| VP-009 | FlowState Machine Validity | reassembly/flow.rs | STORY-013 | BC-2.04.004, BC-2.04.005, BC-2.04.050..052 |
| VP-010 | buffered_bytes Invariant | reassembly/segment.rs | STORY-012, STORY-016 | BC-2.04.030 (STORY-012); BC-2.04.047 (STORY-016) |
| VP-011 | flush_contiguous Monotonicity | reassembly/segment.rs | STORY-015 | BC-2.04.007, BC-2.04.008, BC-2.04.034 |
| VP-012 | escape_for_terminal Correctness | reporter/terminal.rs | STORY-077 | BC-2.11.007..012 |
| VP-013 | JA3 GREASE Filter Correctness | analyzer/tls.rs | STORY-051 | BC-2.07.006..008 |
| VP-014 | HttpAnalyzer Cross-Flow Isolation | analyzer/http.rs | STORY-045 | BC-2.06.019, BC-2.06.021 |
| VP-015 | TCP Sequence Number Wraparound | reassembly/segment.rs | STORY-015 | BC-2.04.039 |
| VP-016 | MITRE Tactic Grouping Order | reporter/terminal.rs | STORY-071, STORY-078 | BC-2.10.003, BC-2.10.004 (STORY-071); BC-2.11.013..015 (STORY-078) |
| VP-017 | JsonReporter Key-Order Determinism | reporter/json.rs | STORY-076 | BC-2.11.001, BC-2.11.003 |
| VP-018 | CLI Reassemble / No-Reassemble Mutual Exclusion | cli.rs | STORY-087, STORY-088 | BC-2.12.007 (STORY-087); BC-2.12.009 (STORY-088) |
| VP-019 | DNS Analyzer Is Statistics-Only (Never Emits Findings) | analyzer/dns.rs | STORY-066 | BC-2.08.001..004 |
| VP-020 | CSV Injection Neutralization | reporter/csv.rs | STORY-079 | BC-2.11.021 |
| VP-021 | Timestamp Provenance End-to-End | reassembly/handler.rs, findings.rs | STORY-097, STORY-098, STORY-099 | BC-2.04.055, BC-2.09.007 |
| VP-023 | DNP3 Parse Safety (4 Kani sub-properties) | analyzer/dnp3.rs | STORY-106 | BC-2.15.001..007 (Kani-provable: parse header, LE decode, validity gate, FC totality, FC set membership, frame_len arithmetic, frame_len bounds). BC-2.15.008 (FIR=1 gating) and BC-2.15.009 (desync bail) are unit-test-only — NOT VP-023 Kani obligations. |
| VP-004 (E-15 arm) | Content-First Dispatch Precedence — port-20000 oracle arm | dispatcher.rs | STORY-110 | BC-2.15.021 |
| VP-007 (E-15 atomic update) | MITRE Technique ID Catalog Completeness — T1691.001 + T0827 seeding | mitre.rs | STORY-109 | BC-2.15.014 (T1691.001), BC-2.15.015 (T0827 correlation) |
| VP-008 (E-16 carry-forward) | decode_packet Never Panics — updated for DecodedFrame return type | decoder.rs | STORY-111 | BC-2.02.009 invariant 5 |
| VP-024 | ARP Frame Parse Safety and Binding-Table Invariant — 4 sub-property groups: Sub-A=3 Kani (verify_extract_arp_frame_safety, verify_extract_arp_frame_eth_ipv4_correctness, verify_extract_arp_frame_none_on_bad_size); Sub-B=1 Kani (verify_classify_garp_total); Sub-C=1 proptest (test_binding_table_last_write_wins — NOT Kani); Sub-D=1 Kani (verify_binding_table_cap) | analyzer/arp.rs + decoder.rs | STORY-112, STORY-113, STORY-116, STORY-117 | BC-2.16.001 (Sub-A safety), BC-2.16.002 (Sub-A correctness), BC-2.16.003 (Sub-B classify_garp_total), BC-2.16.005 (Sub-C last_write_wins), BC-2.16.006 (Sub-D binding_table_cap); STORY-116/117 exercise VP-024 offset-arithmetic correctness for VLAN/QinQ/MACsec paths (BC-2.16.009 EC-008, BC-2.16.015 PC-7a) |
| VP-007 (E-16 atomic update) | MITRE Technique ID Catalog Completeness — ARP technique seeding | mitre.rs | STORY-114 | BC-2.10.005, BC-2.10.008 (catalog SEEDED 23→25 / EMITTED 15→17); BC-2.16.004 (ARP-spoof emission trigger) |

---

## Epic-Level Dependency Summary

```
E-1 (SS-01/02)
  |
  +----> E-2 (SS-04) [via STORY-005 -> STORY-011]
  |        |
  |        +----> E-3 (SS-05) [via STORY-021 -> STORY-031]
  |                 |
  |                 +----> E-4 (SS-06) [via STORY-033 -> STORY-041]
  |                 |        |
  |                 |        +----> E-8 (SS-11) [via STORY-046 -> STORY-076]
  |                 |
  |                 +----> E-5 (SS-07) [via STORY-033 -> STORY-051]
  |                          |
  |                          +----> E-8 (SS-11) [via STORY-057/058 -> STORY-076]
  |
  +----> E-6 (SS-08) [via STORY-005 -> STORY-066; packet-level, bypasses E-2/E-3]
           |
           +----> E-8 (SS-11) [via STORY-066 -> STORY-076]

E-7 (SS-09/10) [independent chain, no upstream deps]
  |
  +----> E-4 (SS-06) [via STORY-071 -> STORY-041]
  +----> E-5 (SS-07) [via STORY-071 -> STORY-051]
  +----> E-8 (SS-11) [via STORY-071 -> STORY-076]

E-8 (SS-11)
  |
  +----> E-9 (SS-12) [via STORY-080 -> STORY-086]
           |
           +----> E-10 (SS-13) [via STORY-086 -> STORY-096]

E-13 (SS-09/10/11 extensions)
  |
  +----> E-14 (SS-14) [via STORY-100 -> STORY-102; multi-tag schema required by Modbus]
  |
  +----> E-15 (SS-15) [via STORY-100 -> STORY-106; multi-tag schema required by DNP3]

E-14 (SS-14 Modbus, SS-05 Rule 5)
  |
  +----> E-15 (SS-15, SS-05 Rule 6) [via STORY-105 -> STORY-110; Rule 5 must precede Rule 6 in classify()]

E-15 (SS-15 DNP3) — linear chain:
  STORY-106 -> STORY-107 -> STORY-108 -> STORY-109 -> STORY-110
  (SS-15 pure core -> SS-15 state -> SS-15 direct detections -> SS-15 correlated detections -> SS-05/SS-15 dispatcher)

E-15 (SS-15 DNP3)
  |
  +----> E-16 (SS-16 ARP) [via STORY-110 -> STORY-111; file-sequencing: etherparse migration touches src/dispatcher.rs, must follow STORY-110's finalized DNP3 changes]

E-16 (SS-16 ARP) — linear chain:
  STORY-111 -> STORY-112 -> STORY-113 -> STORY-114 -> STORY-115
  (SS-02 decoder migration -> SS-16 struct+Kani -> SS-16 detections+binding-table -> SS-16 emissions+VP-007 -> SS-16 D3 storm detection+--arp-storm-rate+storm_findings)

E-16 (SS-16 ARP)
  |
  +----> E-17 (SS-16 offset hardening) [via STORY-115 -> STORY-116; ARP decode-time code must be fully shipped before offset regression tests land]

E-17 (SS-16 ARP VLAN/QinQ/MACsec) — linear chain:
  STORY-116 -> STORY-117
  (SS-16 VLAN+QinQ fixture coverage -> SS-16 MACsec observe-only probe)
```

---

## Gap Register

No story-decomposition gaps identified. All 283 BCs are covered (219 pre-feature + 25 Modbus BC-2.14.001..025 + 24 DNP3 BC-2.15.001..024 across STORY-106..110 + 15 ARP BC-2.16.001..015 across STORY-112..115; BC-2.02.009 is revised in STORY-111, not a new BC).
All L2 domain capabilities (CAP-NNN) are covered by at least one story.
All cross-epic architectural dependencies are captured in this graph.

E-15 DNP3 specific gap notes:
- BC-2.15.017 (CLI threshold flag) and BC-2.15.021 (dispatcher Rule 6) are co-located in STORY-110; they share the same implementation scope (dispatcher + CLI wiring) and cannot be independently delivered.

E-16 ARP specific gap notes:
- BC-2.16.010 (summarize key layout) is primarily owned by STORY-113 but extended in STORY-115 (storm_findings key added). This is not a gap — the BC covers the full key set; STORY-113 establishes the initial layout and STORY-115 extends it in-place.
- STORY-111 revises BC-2.02.009 (not a new BC); the revision adds a third decode path (DecodedFrame::Arp) and does not break the existing two paths. VP-008 fuzz harness must be updated in STORY-111 per F3-OBL-STORY111-VP008.
- T0814 MITRE tag in STORY-115 (D3 storm) is deferred per DF-VALIDATION-001; STORY-115 covers BC-2.16.008 (storm detection) without the MITRE tag until validated.
- BC-2.15.015 (single 300s window reset owner) and BC-2.15.014 (block inference emission) are co-located in STORY-109; the reset and the emission are inseparable behaviors.

E-17 ARP offset hardening specific gap notes:
- BC-2.16.009 and BC-2.16.015 are primarily owned by STORY-113 and STORY-112 respectively; STORY-116 and STORY-117 extend coverage to EC-008 (QinQ offset) and EC-009 (MACsec observe-only) additions introduced in BC-2.16.009 v1.10 / BC-2.16.015 v1.9. No new BCs; no gap — deeper clause coverage on existing BCs.

| Gap ID | Level | Source | Justification | Resolution Target |
|--------|-------|--------|---------------|-------------------|
| (none) | — | — | — | — |

---

## Notes on E-9 Transitive Coverage

STORY-086's single cross-epic dep on STORY-080 (E-8 leaf) provides transitive
coverage of all upstream epics:

```
STORY-080 -> STORY-079 -> STORY-076
STORY-076 <- STORY-046 (E-4) <- STORY-041 <- STORY-033 (E-3) <- STORY-021 (E-2) <- STORY-011 <- STORY-005 (E-1)
STORY-076 <- STORY-057/058 (E-5)
STORY-076 <- STORY-066 (E-6)
STORY-076 <- STORY-071 (E-7)
```

STORY-086's authoritative `depends_on` is `[STORY-080]` (single cross-epic edge).
Transitively, STORY-080 carries full coverage of E-1..E-8 as shown above.
The dependency-graph.md is authoritative; individual story frontmatter `depends_on`
fields are the canonical edge set.
