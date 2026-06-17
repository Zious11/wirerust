---
document_type: story-index
version: "1.6"
status: draft
producer: story-writer
timestamp: 2026-06-16T00:00:00Z
phase: 3
total_stories: 70
total_waves: 46
total_points: 465
# v1.5 totals reconciliation (Pass-26 Slice-D remediation):
# Root cause: the "400 pre-ARP" figure in v1.4 was itself 10 low (actual pre-ARP incl STORY-091 = 410).
# 410 pre-ARP + 47 E-16 ARP = 457 grand total. Wave table (excl STORY-091) = 452.
# All 68 per-story index cells verified against story files — zero mismatches found.
# v1.6 (2026-06-16): E-17 ARP QinQ/MACsec offset hardening — STORY-116 + STORY-117 added.
# 457 pre-E17 + 3 (STORY-116) + 5 (STORY-117) = 465 grand total. Waves 45-46 added.
# STORY-091 wave remains TBD (excluded from wave table as before).
traces_to:
  - .factory/stories/dependency-graph.md
  - .factory/stories/epics.md
  - .factory/cycles/v0.1.0-greenfield-spec/wave-schedule.md
  - .factory/feature/wave-schedule.md
---

# wirerust Story Index

> **Authoritative story registry for the v0.1.0-greenfield-spec cycle (48 greenfield product + 1 tooling STORY-091 = 49 stories) + Feature Mode F3 additions (3 stories, STORY-097/098/099 for issue #100) + Feature #7 additions (6 stories, STORY-100..105 for issue #7 Modbus Analyzer) + Feature #8 additions (5 stories, STORY-106..110 for issue #8 DNP3/ICS Analyzer) + Feature #9 additions (5 stories, STORY-111..115 for issue #9 ARP Security Analyzer) + E-17 F3 additions (2 stories, STORY-116..117 for issue #253 ARP QinQ/MACsec Offset Hardening).**
> All 48 greenfield stories formalize behavioral contracts for the existing shipped wirerust
> codebase. STORY-097/098/099 are new feature stories for issue #100 (pcap timestamps).
> STORY-100/101 implement E-13 multi-tag Finding schema migration (v0.3.0).
> STORY-102/103/104/105 implement E-14 Modbus TCP analyzer (v0.4.0).
> STORY-106/107/108/109/110 implement E-15 DNP3/ICS analyzer (issue #8).
> STORY-111/112/113/114/115 implement E-16 ARP Security Analyzer (issue #9, v0.7.0).
> Status `draft` = not yet dispatched. Wave assignments are from the
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
| STORY-057 | SNI Edge Cases — Empty Lists, Empty Hostnames, Multi-Name, NameType, Trailing Bytes, Large SNI, and Count-Cap Decoupling | E-5 | 19 | 8 | completed | STORY-055, STORY-056 |
| STORY-058 | Buffer Management, Record Parsing Infrastructure, Flow Lifecycle, and summarize Output | E-5 | 18 | 8 | completed | STORY-052, STORY-053 |
| STORY-066 | DNS Traffic Statistics — Port-53 Dispatch, QR-Bit Counting, and Never-Emit Contract | E-6 | 4 | 5 | completed | STORY-005 |
| STORY-069 | Finding Struct, Verdict/Confidence Display, and Finding Display Format | E-7 | 1 | 5 | completed | — |
| STORY-070 | Raw-Data Contract and JSON Serialization Symmetry (skip_serializing_if) | E-7 | 2 | 5 | completed | STORY-069 |
| STORY-071 | MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order | E-7 | 3 | 8 | completed | STORY-069, STORY-070 |
| STORY-076 | JsonReporter — Structure, skipped_packets, and RFC 8259 Byte Handling | E-8 | 20 | 5 | completed | STORY-046, STORY-057, STORY-058, STORY-066, STORY-071 |
| STORY-077 | TerminalReporter — escape_for_terminal, skipped_packets, and End-to-End C1 Safety | E-8 | 21 | 8 | completed | STORY-076 |
| STORY-078 | TerminalReporter — MITRE Grouping, Section Order, and Colorization | E-8 | 22 | 8 | completed | STORY-077 |
| STORY-079 | CsvReporter — Fixed 9-Column Schema, CSV-Injection Neutralization, and Evidence Join | E-8 | 21 | 5 | completed | STORY-076 |
| STORY-080 | CsvReporter — Reporter Trait Compliance and Optional Field Encoding | E-8 | 22 | 3 | completed | STORY-079 |
| STORY-086 | CLI Subcommand Parsing — analyze, summary, --no-color, Multiple Targets | E-9 | 23 | 5 | completed | STORY-080 |
| STORY-087 | Output Format Flags and Reassembly Configuration Flags | E-9 | 24 | 5 | completed | STORY-086 |
| STORY-088 | run_analyze Orchestration — Analyzer Enablement, Reassembly Logic, Target Expansion, Progress Bar | E-9 | 25 | 8 | completed | STORY-086, STORY-087 |
| STORY-089 | Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing | E-9 | 26 | 5 | completed | STORY-086, STORY-087, STORY-088 |
| STORY-090 | Summary Data Model — ingest, Service Hints, unique_hosts, Serialization | E-9 | 27 | 5 | completed | STORY-086, STORY-088, STORY-089 |
| STORY-096 | Absent Behavior Contracts — Removed Flags Rejected by clap | E-10 | 24 | 3 | completed | STORY-086 |
| STORY-091 | Anchor-Validation Tooling — bin/validate-anchors | E-11 | ~ | 5 | draft | — |
| STORY-097 | Thread Capture-Relative Timestamp Through StreamHandler::on_data | E-12 | 28 | 5 | completed | — |
| STORY-098 | Attach Pcap Timestamp to Emitted Findings | E-12 | 29 | 8 | completed | STORY-097 |
| STORY-099 | Verify Timestamp Provenance End-to-End (VP-021) | E-12 | 30 | 5 | completed | STORY-098 |
| STORY-100 | Multi-Tag Finding Schema Migration (Atomic Type Rename + Catalog Seed) | E-13 | 31 | 13 | completed | — |
| STORY-101 | Multi-Tag Reporter Serialization + JSON Envelope Add-Ons | E-13 | 31 | 8 | completed | STORY-100 |
| STORY-102 | Modbus MBAP Parse + FC Classification (Pure Core) | E-14 | 32 | 8 | completed | STORY-100 |
| STORY-103 | Modbus Flow State + Transaction Correlation | E-14 | 33 | 8 | completed | STORY-102 |
| STORY-104 | Modbus Detection Emissions + Summary | E-14 | 33 | 13 | completed | STORY-103 |
| STORY-105 | Modbus Dispatcher Integration + CLI | E-14 | 34 | 8 | completed | STORY-104 |
| STORY-106 | DNP3 DL/Transport Parse + FC Classify — Pure Core (VP-023 Kani) | E-15 | 35 | 8 | completed | STORY-100 |
| STORY-107 | DNP3 Per-Flow State + Carry Buffer + Pending-Request Bounds | E-15 | 36 | 5 | completed | STORY-106 |
| STORY-108 | DNP3 Direct Detection Emissions — T1692.001, T0814 (Restart), T0836, Co-Emission, Summarize | E-15 | 37 | 13 | completed | STORY-107 |
| STORY-109 | DNP3 Correlated/Derived + Anomaly Detections — T1691.001, T0827, Broadcast, Unsolicited, ENABLE/DISABLE, Malformed | E-15 | 38 | 13 | completed | STORY-108 |
| STORY-110 | DNP3 Dispatcher Integration + CLI Flag — VP-004 Oracle + VP-007 Atomic-Update | E-15 | 39 | 8 | completed | STORY-109 |
| STORY-111 | etherparse 0.20 Migration + DecodedFrame/ArpFrame Types + BC-2.02.009 Revision | E-16 | 40 | 5 | draft | STORY-110 |
| STORY-112 | extract_arp_frame + decode_packet ARP Routing (Both Paths) + ArpAnalyzer Stub + VP-024 Sub-A | E-16 | 41 | 8 | draft | STORY-111 |
| STORY-113 | ArpAnalyzer Full Implementation — Binding Table, GARP (D2), D11, D12, summarize(), --arp Flag, VP-024 Sub-B/C/D | E-16 | 42 | 13 | draft | STORY-112 |
| STORY-114 | D1 ARP Spoof Escalation + GARP-that-Conflicts (D2+D1) + MITRE Attribution + VP-007 5-Part Atomic Update | E-16 | 43 | 13 | draft | STORY-113 |
| STORY-115 | D3 ARP Storm Detection + --arp-storm-rate CLI Flag + storm_findings Summary Key | E-16 | 44 | 8 | draft | STORY-114 |
| STORY-116 | ARP QinQ (Double-Tag) Decoder Offset Coverage | E-17 | 45 | 3 | draft | STORY-115 |
| STORY-117 | ARP MACsec Offset Documented-Limitation Coverage | E-17 | 46 | 5 | draft | STORY-116 |

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
| 28 | STORY-097 | 1 | 5 |
| 29 | STORY-098 | 1 | 8 |
| 30 | STORY-099 | 1 | 5 |
| 31 | STORY-100, STORY-101 | 2 | 21 |
| 32 | STORY-102 | 1 | 8 |
| 33 | STORY-103, STORY-104 | 2 | 21 |
| 34 | STORY-105 | 1 | 8 |
| 35 | STORY-106 | 1 | 8 |
| 36 | STORY-107 | 1 | 5 |
| 37 | STORY-108 | 1 | 13 |
| 38 | STORY-109 | 1 | 13 |
| 39 | STORY-110 | 1 | 8 |
| 40 | STORY-111 | 1 | 5 |
| 41 | STORY-112 | 1 | 8 |
| 42 | STORY-113 | 1 | 13 |
| 43 | STORY-114 | 1 | 13 |
| 44 | STORY-115 | 1 | 8 |
| 45 | STORY-116 | 1 | 3 |
| 46 | STORY-117 | 1 | 5 |
| **TOTAL (excl. STORY-091, wave-TBD)** | | **69** | **460** |

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
| E-11: Tooling and Self-Improvement | STORY-091 | 1 | 5 |
| E-12: Pcap Timestamp Provenance (issue #100) | STORY-097, STORY-098, STORY-099 | 3 | 18 |
| E-13: Multi-Tag Finding Schema Migration (v0.3.0 / issue #7) | STORY-100, STORY-101 | 2 | 21 |
| E-14: Modbus TCP Analyzer (v0.4.0 / issue #7) | STORY-102, STORY-103, STORY-104, STORY-105 | 4 | 37 |
| E-15: DNP3/ICS Analyzer (issue #8) | STORY-106, STORY-107, STORY-108, STORY-109, STORY-110 | 5 | 47 |
| E-16: ARP Security Analyzer (issue #9) | STORY-111, STORY-112, STORY-113, STORY-114, STORY-115 | 5 | 47 |
| E-17: ARP QinQ/MACsec Offset Hardening (issue #253) | STORY-116, STORY-117 | 2 | 8 |
| **TOTAL** | | **70** | **465** |

---

## Wave Delivery Progress

| Wave | Stories | Status | PRs | Merge Commits | Date |
|------|---------|--------|-----|---------------|------|
| 1 | STORY-001, STORY-069 | **DELIVERED & CLOSED** | #106, #105 | b7424b7, 2840caf | 2026-05-22 |
| 2 | STORY-002, STORY-003, STORY-004, STORY-070 | **DELIVERED & CLOSED** | #109, #110, #107, #108 | 34c592b, 3b2481c, 385e763, 8b514c00 | 2026-05-22 |
| 3 | STORY-071, STORY-005 | **DELIVERED & CLOSED** | — | f0b5007 | 2026-05-22 |
| 4 | STORY-011, STORY-066 | **DELIVERED & CLOSED** | — | f628c33 | 2026-05-22 |
| 5 | STORY-012 | **DELIVERED & CLOSED** | — | bbddac6 | 2026-05-22 |
| 6 | STORY-013 | **DELIVERED & CLOSED** | #119 | 3e705b5 | 2026-05-22 |
| 7 | STORY-014 | **DELIVERED & CLOSED** | #120 | bc5d23e | 2026-05-25 |
| 8 | STORY-015, STORY-019 | **DELIVERED & CLOSED** | #122, #123 | 4b9b85f | 2026-05-26 |
| 9 | STORY-016, STORY-020 | **DELIVERED & CLOSED** | #127, #128, #129, #130 | e237747 | 2026-05-26 |
| 10 | STORY-017, STORY-018 | **DELIVERED & CLOSED** | #131, #132, #133 | 211143e | 2026-05-27 |
| 11 | STORY-021 | **DELIVERED & CLOSED** | #134 | 3cd3000 | 2026-05-27 |
| 12 | STORY-031 | **DELIVERED & CLOSED** | #135 | 1435362 | 2026-05-27 |
| 13 | STORY-032 | **DELIVERED & CLOSED** | #136 | 0d9b16d | 2026-05-27 |
| 14 | STORY-033 | **DELIVERED & CLOSED** | #137 | 30cd4a6 | 2026-05-28 |
| 15 | STORY-041, STORY-051 | **DELIVERED & CLOSED** | #138, #139 | cb322dc, 945034d | 2026-05-28 |
| 16 | STORY-042, STORY-043, STORY-044, STORY-052 | **DELIVERED & CLOSED** | #140, #141, #142, #143, #144, #145, #146 | fa17dec | 2026-05-29 |
| 17 | STORY-045, STORY-053, STORY-055 | **DELIVERED & CLOSED** | #150, #149, #151 | 9633b0d | 2026-05-29 |
| 18 | STORY-046, STORY-054, STORY-056, STORY-058 | **DELIVERED & CLOSED** | #152, #153, #154, #155 | 3f87ac3 | 2026-05-29 |
| 19 | STORY-057 | **DELIVERED & CLOSED** | #156 | 616897e | 2026-05-29 |
| 20 | STORY-076 | **DELIVERED & CLOSED** | #157 | e5cb2b1 | 2026-05-29 |
| 21 | STORY-077, STORY-079 | **DELIVERED & CLOSED** | #158, #159 | 41ab24d | 2026-05-30 |
| 22 | STORY-078, STORY-080 | **DELIVERED & CLOSED** | #160, #161, #162 | bf16c0b, 1ecf114, c127c1c | 2026-05-30 |
| 23 | STORY-086 | **DELIVERED & CLOSED** | #163 | a42e14b | 2026-05-31 |
| 24 | STORY-087, STORY-096 | **DELIVERED & CLOSED** | #164, #165 | c2445dc, 9954d44 | 2026-05-31 |
| 25 | STORY-088 | **DELIVERED & CLOSED** | #168 | 5202fe9 | 2026-05-31 |
| 26 | STORY-089 | **DELIVERED & CLOSED** | #169 | 450d33e | 2026-05-31 |
| 27 | STORY-090 | **DELIVERED & CLOSED** | #170 | 6158e6e | 2026-05-31 |
| 28 | STORY-097 | **DELIVERED & CLOSED** | #197 | 2d1c9e2 | 2026-06-08 |
| 29 | STORY-098 | **DELIVERED & CLOSED** | #198 | 3b390b2 | 2026-06-08 |
| 30 | STORY-099 | **DELIVERED & CLOSED** | #199 | 48cbc05 | 2026-06-08 |
| 31 | STORY-100, STORY-101 | **DELIVERED & CLOSED** | #209 | c846b3b | 2026-06-09 |
| 32 | STORY-102 | **DELIVERED & CLOSED** | #211 | 26d58bb | 2026-06-09 |
| 33 | STORY-103, STORY-104 | **DELIVERED & CLOSED** | #212, #213 | d894464, dba... | 2026-06-09 |
| 34 | STORY-105 | **DELIVERED & CLOSED** | #214 | dba5f26 | 2026-06-09 |
| 35 | STORY-106 | **DELIVERED & CLOSED** | #225 | d0f3586 | 2026-06-11 |
| 36 | STORY-107 | **DELIVERED & CLOSED** | #226 | ebb4751 | 2026-06-11 |
| 37 | STORY-108 | **DELIVERED & CLOSED** | #227 | 9c03fde | 2026-06-11 |
| 38 | STORY-109 | **DELIVERED & CLOSED** | #228 | 34443f9 | 2026-06-11 |
| 39 | STORY-110 | **DELIVERED & CLOSED** | #229 | ddfa576 | 2026-06-11 |
| 40 | STORY-111 | draft | — | — | — |
| 41 | STORY-112 | draft | — | — | — |
| 42 | STORY-113 | draft | — | — | — |
| 43 | STORY-114 | draft | — | — | — |
| 44 | STORY-115 | draft | — | — | — |
| 45 | STORY-116 | draft | #258 (test/arp-qinq-macsec-fixtures) | — | — |
| 46 | STORY-117 | draft | #258 (test/arp-qinq-macsec-fixtures) | — | — |

## Coverage Verification

- Total stories: **70** (48 greenfield product + 1 tooling STORY-091 + 3 F3 feature STORY-097/098/099 + 6 Feature-#7 STORY-100..105 + 5 Feature-#8 STORY-106..110 + 5 Feature-#9 STORY-111..115 + 2 E-17 F3 STORY-116..117)
- Total waves: **46** (Waves 40–44 added for Feature #9 ARP; Waves 45–46 added for E-17 QinQ/MACsec hardening; STORY-091 wave TBD)
- Total points: **465** (457 pre-E17 + 3 STORY-116 + 5 STORY-117; wave-total row shows 460 — delta of 5 is STORY-091 at wave TBD excluded from wave table)
- Graph is acyclic: **Yes** (Kahn topological sort verified; Feature-#7 dependency chain: STORY-100 → {STORY-101 ∥ STORY-102} → STORY-103 → STORY-104 → STORY-105; Feature-#8 DNP3 chain: STORY-100 → STORY-106 → STORY-107 → STORY-108 → STORY-109 → STORY-110; Feature-#9 ARP chain: STORY-110 → STORY-111 → STORY-112 → STORY-113 → STORY-114 → STORY-115; E-17 hardening chain: STORY-115 → STORY-116 → STORY-117; no back-edges into existing 68-story graph)
- All 10 product epics + E-11 (Tooling) + E-12 (Pcap Timestamps) + E-13 (Multi-Tag Migration) + E-14 (Modbus) + E-15 (DNP3) + E-16 (ARP) + E-17 (ARP QinQ/MACsec Hardening) covered: **Yes**
- All 219 greenfield BCs assigned + F2 additions + BC-2.09.001/006 (shared, extended in STORY-100) + BC-2.10.005/007/008 (extended in STORY-100) + BC-2.11.001/013/015/017/020/024 (extended in STORY-101) + BC-2.14.001..025 (new Modbus BCs in STORY-102..105) + BC-2.15.001..024 (new DNP3 BCs in STORY-106..110) + BC-2.02.009 (revised in STORY-111) + BC-2.16.001..015 (new ARP BCs in STORY-111..115) + BC-2.16.009 v1.10 EC-009 / BC-2.16.015 v1.9 EC-009 (E-17 MACsec documented-limitation extensions in STORY-116/117): **Yes** (total 283 BCs, 2 updated with E-17 EC-009 clauses)
- PROCESS-GAP-P5-001 dispositioned: **Yes** — STORY-091 created as the S-7.02 cycle-close disposition
- Coverage note: STORY-097/098/099 trace to BC-2.04.055 and BC-2.09.007 (both F2 additions); these 3 stories cover VP-021 (verified @256a490). STORY-100 extends BC-2.09.001 (field rename) and BC-2.10.005/007/008 (catalog seed to 21). STORY-101 extends BC-2.11.001/013/015/017/020/024 (reporter multi-tag). STORY-102..105 cover BC-2.14.001..025 (Modbus TCP analyzer). STORY-106..110 cover BC-2.15.001..024 (DNP3/ICS analyzer); VP-023 Kani lands in STORY-106, VP-004 oracle obligation lands in STORY-110, VP-007 atomic-update obligation (SEEDED 21→23, EMITTED 13→15) lands in STORY-109. STORY-111..115 cover BC-2.16.001..015 (ARP Security Analyzer): STORY-111 covers BC-2.02.009 (revised) + decoder migration; STORY-112 covers BC-2.16.001/002/015 (VP-024 Sub-A Kani); STORY-113 covers BC-2.16.003/005/006/007/009/010/011; STORY-114 covers BC-2.16.004/012/014 (VP-007 SEEDED 23→25 / EMITTED 15→17) + BC-2.16.007 D12-MITRE extension; STORY-115 covers BC-2.16.008/013 + BC-2.16.010 extension. STORY-116 covers BC-2.16.009 EC-008 (QinQ D11) + BC-2.16.015 PC-7b/EC-008 (QinQ offset-22 formula pin) + BC-2.16.015 EC-009(a) (MACsec observe-only probe, no-SCI shape guard). STORY-117 covers BC-2.16.009 EC-009(a) (MACsec offset 22/30 assertion + D11 routing) + BC-2.16.015 EC-009(a/b) (SCI-present spec-backing test; Modified/Encrypted opaque-unreachable security guards); both stories reference VP-024 lifecycle note (append-only, no proof change).
- Release mapping: v0.3.0 ships after Wave 31 gate (STORY-100 + STORY-101 merged); v0.4.0 ships after Wave 34 gate (STORY-102..105 merged); v0.6.0 ships after Wave 39 gate (STORY-106..110 merged); v0.7.0 ships after Wave 44 gate (STORY-111..115 merged); v0.7.1 ships after Wave 46 gate (STORY-116 + STORY-117 merged — E-17 test-and-docs patch). (v0.5.0 was the MITRE-drift-guard fix released separately; DNP3 targets v0.6.0; ARP targets v0.7.0; ARP QinQ/MACsec hardening targets v0.7.1.)
- Existing stories affected by schema migration: STORY-069/070/071/078/079/080 — their MITRE-technique test assertions migrate from `mitre_technique: Option<String>` to `mitre_techniques: Vec<String>` via STORY-100 (see revision notes in each story). STORY-111 revises BC-2.02.009 to add the third decode path (ArpFrame); existing STORY-002/003 passing tests are unaffected.
