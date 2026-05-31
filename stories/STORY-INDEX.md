---
document_type: story-index
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
total_stories: 48
total_waves: 27
total_points: 282
traces_to:
  - .factory/stories/dependency-graph.md
  - .factory/stories/epics.md
  - .factory/cycles/v0.1.0-greenfield-spec/wave-schedule.md
---

# wirerust Story Index

> **Authoritative story registry for the v0.1.0-greenfield-spec cycle.**
> All 48 stories formalize behavioral contracts for the existing shipped wirerust
> codebase. Status `draft` = not yet dispatched. Wave assignments are from the
> authoritative dependency-graph.md (longest-path / Kahn topological sort).

---

## Index Table

| Story ID | Title | Epic | Wave | Points | Status | Dependencies |
|----------|-------|------|------|--------|--------|--------------|
| STORY-001 | PCAP File Ingestion — Link-Type Gating, Eager Packet Load, and Error Surfaces | E-1 | 1 | 5 | completed | — |
| STORY-002 | Packet Decoding — Ethernet, RAW/IPV4, and IPv6 Link-Layer Paths | E-1 | 2 | 5 | completed | STORY-001 |
| STORY-003 | Packet Decoding — Linux SLL, No-Panic Safety, and Non-IP Frame Rejection | E-1 | 2 | 5 | completed | STORY-001 |
| STORY-004 | Packet Decoding — ICMP, Protocol::Other, and app_protocol_hint Port Table | E-1 | 2 | 3 | completed | STORY-001 |
| STORY-005 | Packet Decoding — packet_len Semantics and TCP Flag/Sequence Extraction | E-1 | 3 | 3 | completed | STORY-002, STORY-003, STORY-004 |
| STORY-011 | TcpReassembler Constructor Validation and FlowKey Canonicalization | E-2 | 4 | 5 | completed | STORY-005 |
| STORY-012 | Non-TCP Packet Filter, Statistics Summary, and bytes_reassembled Accounting | E-2 | 5 | 5 | completed | STORY-011 |
| STORY-013 | TCP Three-Way Handshake State Machine and Direction Tagging | E-2 | 6 | 8 | completed | STORY-012 |
| STORY-014 | Mid-Stream Join, ISN Management, and IsnMissing Guard | E-2 | 7 | 5 | completed | STORY-013 |
| STORY-015 | In-Order Delivery, Out-of-Order Buffering, and Bidirectional Direction Tagging | E-2 | 8 | 8 | completed | STORY-013, STORY-014 |
| STORY-016 | Overlap Detection — Duplicate Retransmissions, Partial Overlap, and buffered_bytes Accounting | E-2 | 9 | 8 | completed | STORY-015 |
| STORY-017 | Conflict and Evasion Detection — T1036 Findings and One-Shot Anomaly Latches | E-2 | 10 | 8 | completed | STORY-015, STORY-016 |
| STORY-018 | Resource Bounds — Depth Truncation, Out-of-Window Rejection, and Segment Limit Enforcement | E-2 | 10 | 8 | completed | STORY-015, STORY-016 |
| STORY-019 | Flow Lifecycle — RST Close, FIN Close, Timeout Expiry, and Missing-Key Warning | E-2 | 8 | 5 | completed | STORY-013, STORY-014 |
| STORY-020 | Memory Management — total_memory Accounting and LRU Eviction Policies | E-2 | 9 | 8 | completed | STORY-019 |
| STORY-021 | Finalize Lifecycle, MAX_FINDINGS Cap, and Segment-Limit Summary Finding | E-2 | 11 | 5 | completed | STORY-017, STORY-018, STORY-019, STORY-020 |
| STORY-031 | Content-First Classification — TLS Signature, HTTP Method Prefix, Port Fallback | E-3 | 12 | 5 | completed | STORY-021 |
| STORY-032 | Classification Caching and DispatchTarget::None Retry Budget | E-3 | 13 | 5 | completed | STORY-031 |
| STORY-033 | Flow Lifecycle — Close, Unclassified Counter, No-Op Dispatcher | E-3 | 14 | 3 | completed | STORY-031, STORY-032 |
| STORY-041 | HTTP/1.1 Request/Response Parsing and Core Statistics | E-4 | 15 | 8 | completed | STORY-033, STORY-071 |
| STORY-042 | URI-Based Threat Detections — Path Traversal, Web Shell, Admin Panel | E-4 | 16 | 5 | completed | STORY-041 |
| STORY-043 | Header and Method Anomaly Detections — Method, Host, URI Length, User-Agent | E-4 | 16 | 5 | completed | STORY-041 |
| STORY-044 | Parse-Error Isolation and Poisoning State Machine | E-4 | 16 | 8 | completed | STORY-041 |
| STORY-045 | Flow Lifecycle, Cross-Flow Isolation, and Buffer/Map Caps | E-4 | 17 | 5 | completed | STORY-041, STORY-044 |
| STORY-046 | HTTP Analyzer Summary Output | E-4 | 18 | 3 | completed | STORY-041, STORY-042, STORY-043, STORY-044, STORY-045 |
| STORY-051 | JA3 and JA3S Computation — GREASE Filtering and String Format | E-5 | 15 | 5 | completed | STORY-033, STORY-071 |
| STORY-052 | ClientHello Parsing — Handshake Counting, Version/JA3 Tracking, and Done Short-Circuit | E-5 | 16 | 8 | completed | STORY-051 |
| STORY-053 | ServerHello Parsing — JA3S Fingerprinting and Cipher/Version Tracking | E-5 | 17 | 5 | completed | STORY-051, STORY-052 |
| STORY-054 | Cipher and Protocol Weakness Findings — Weak Ciphers, Deprecated SSL Versions, and Baseline Zero-Finding | E-5 | 18 | 8 | completed | STORY-052, STORY-053 |
| STORY-055 | SNI Classification Arms 1 and 2 — Clean ASCII Baseline and C0/DEL Control-Byte Detection | E-5 | 17 | 8 | completed | STORY-052 |
| STORY-056 | SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte Preservation | E-5 | 18 | 8 | completed | STORY-055 |
| STORY-057 | SNI Edge Cases — Empty Lists, Empty Hostnames, Multi-Name, NameType, Trailing Bytes, Large SNI, and Count-Cap Decoupling | E-5 | 19 | 8 | draft | STORY-055, STORY-056 |
| STORY-058 | Buffer Management, Record Parsing Infrastructure, Flow Lifecycle, and summarize Output | E-5 | 18 | 8 | completed | STORY-052, STORY-053 |
| STORY-066 | DNS Traffic Statistics — Port-53 Dispatch, QR-Bit Counting, and Never-Emit Contract | E-6 | 4 | 5 | completed | STORY-005 |
| STORY-069 | Finding Struct, Verdict/Confidence Display, and Finding Display Format | E-7 | 1 | 5 | completed | — |
| STORY-070 | Raw-Data Contract and JSON Serialization Symmetry (skip_serializing_if) | E-7 | 2 | 5 | completed | STORY-069 |
| STORY-071 | MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order | E-7 | 3 | 8 | completed | STORY-069, STORY-070 |
| STORY-076 | JsonReporter — Structure, skipped_packets, and RFC 8259 Byte Handling | E-8 | 20 | 5 | draft | STORY-046, STORY-057, STORY-058, STORY-066, STORY-071 |
| STORY-077 | TerminalReporter — escape_for_terminal, skipped_packets, and End-to-End C1 Safety | E-8 | 21 | 8 | draft | STORY-076 |
| STORY-078 | TerminalReporter — MITRE Grouping, Section Order, and Colorization | E-8 | 22 | 8 | draft | STORY-077 |
| STORY-079 | CsvReporter — Fixed 9-Column Schema, CSV-Injection Neutralization, and Evidence Join | E-8 | 21 | 5 | draft | STORY-076 |
| STORY-080 | CsvReporter — Reporter Trait Compliance and Optional Field Encoding | E-8 | 22 | 3 | draft | STORY-079 |
| STORY-086 | CLI Subcommand Parsing — analyze, summary, --no-color, Multiple Targets | E-9 | 23 | 5 | completed | STORY-080 |
| STORY-087 | Output Format Flags and Reassembly Configuration Flags | E-9 | 24 | 5 | draft | STORY-086 |
| STORY-088 | run_analyze Orchestration — Analyzer Enablement, Reassembly Logic, Target Expansion, Progress Bar | E-9 | 25 | 8 | draft | STORY-086, STORY-087 |
| STORY-089 | Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing | E-9 | 26 | 5 | draft | STORY-086, STORY-087, STORY-088 |
| STORY-090 | Summary Data Model — ingest, Service Hints, unique_hosts, Serialization | E-9 | 27 | 5 | draft | STORY-086, STORY-088, STORY-089 |
| STORY-096 | Absent Behavior Contracts — Removed Flags Rejected by clap | E-10 | 24 | 3 | draft | STORY-086 |

---

## Stories by Wave

| Wave | Story IDs | Count | Points |
|------|-----------|-------|--------|
| 1 | STORY-001, STORY-069 | 2 | 10 |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | 4 | 18 |
| 3 | STORY-005, STORY-071 | 2 | 11 |
| 4 | STORY-011, STORY-066 | 2 | 10 |
| 5 | STORY-012 | 1 | 5 |
| 6 | STORY-013 | 1 | 8 |
| 7 | STORY-014 | 1 | 5 |
| 8 | STORY-015, STORY-019 | 2 | 13 |
| 9 | STORY-016, STORY-020 | 2 | 16 |
| 10 | STORY-017, STORY-018 | 2 | 16 |
| 11 | STORY-021 | 1 | 5 |
| 12 | STORY-031 | 1 | 5 |
| 13 | STORY-032 | 1 | 5 |
| 14 | STORY-033 | 1 | 3 |
| 15 | STORY-041, STORY-051 | 2 | 13 |
| 16 | STORY-042, STORY-043, STORY-044, STORY-052 | 4 | 26 |
| 17 | STORY-045, STORY-053, STORY-055 | 3 | 18 |
| 18 | STORY-046, STORY-054, STORY-056, STORY-058 | 4 | 27 |
| 19 | STORY-057 | 1 | 8 |
| 20 | STORY-076 | 1 | 5 |
| 21 | STORY-077, STORY-079 | 2 | 13 |
| 22 | STORY-078, STORY-080 | 2 | 11 |
| 23 | STORY-086 | 1 | 5 |
| 24 | STORY-087, STORY-096 | 2 | 8 |
| 25 | STORY-088 | 1 | 8 |
| 26 | STORY-089 | 1 | 5 |
| 27 | STORY-090 | 1 | 5 |
| **TOTAL** | | **48** | **282** |

---

## Stories by Epic

| Epic | Story IDs | Count | Points |
|------|-----------|-------|--------|
| E-1: PCAP Ingestion and Packet Decoding | STORY-001, STORY-002, STORY-003, STORY-004, STORY-005 | 5 | 21 |
| E-2: TCP Stream Reassembly Engine | STORY-011, STORY-012, STORY-013, STORY-014, STORY-015, STORY-016, STORY-017, STORY-018, STORY-019, STORY-020, STORY-021 | 11 | 73 |
| E-3: Content-First Protocol Dispatch | STORY-031, STORY-032, STORY-033 | 3 | 13 |
| E-4: HTTP Traffic Analysis and Threat Detection | STORY-041, STORY-042, STORY-043, STORY-044, STORY-045, STORY-046 | 6 | 34 |
| E-5: TLS Traffic Analysis and Fingerprinting | STORY-051, STORY-052, STORY-053, STORY-054, STORY-055, STORY-056, STORY-057, STORY-058 | 8 | 58 |
| E-6: DNS Traffic Statistics | STORY-066 | 1 | 5 |
| E-7: Forensic Finding Data Model and MITRE Mapping | STORY-069, STORY-070, STORY-071 | 3 | 18 |
| E-8: Reporting and Output Formats | STORY-076, STORY-077, STORY-078, STORY-079, STORY-080 | 5 | 29 |
| E-9: CLI, Entry Point, and Analysis Orchestration | STORY-086, STORY-087, STORY-088, STORY-089, STORY-090 | 5 | 28 |
| E-10: Absent Behavior Contracts (Flag Rejection) | STORY-096 | 1 | 3 |
| **TOTAL** | | **48** | **282** |

---

## Wave Delivery Progress

| Wave | Stories | Status | PRs | Merge Commits | Date |
|------|---------|--------|-----|---------------|------|
| 1 | STORY-001, STORY-069 | **DELIVERED & CLOSED** | #106, #105 | b7424b7, 2840caf | 2026-05-22 |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | **DELIVERED & CLOSED** | #109, #110, #107, #108 | 34c592b, 3b2481c, 385e763, 8b514c00 | 2026-05-22 |
| 23 | STORY-086 | **DELIVERED & CLOSED** | #163 | a42e14b | 2026-05-31 |

## Coverage Verification

- Total stories: **48** (matches dependency-graph.md `total_stories: 48`)
- Total waves: **27** (matches dependency-graph.md `number_of_waves: 27`)
- Total points: **282** (matches dependency-graph.md `Total story points: 282`)
- Graph is acyclic: **Yes** (Kahn topological sort verified in dependency-graph.md)
- All 10 epics covered: **Yes**
- All 217 BCs assigned: **Yes** (per dependency-graph.md BC to Stories Matrix)
