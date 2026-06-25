---
document_type: story-index
version: "2.8"
status: draft
producer: story-writer
timestamp: 2026-06-24T00:00:00Z
phase: f3
total_stories: 91
total_waves: 61
total_points: 592  # = 526 pre-ENIP + 66 (STORY-130..138: 8+8+8+5+8+8+5+8+8=66 pts); wave-table scheduled: 521+66=587; epic-table grand total: 529+66=595; stories: 82+9=91; waves: 57+4=61; STORY-091+STORY-121 remain wave-TBD (excluded from wave-table total)
# ARITHMETIC: 130=8,131=8,132=8,133=5,134=8,135=8,136=5,137=8,138=8 → sum=66 pts (ENIP E-20); total_points: 526+66=592; wave-table scheduled: 521+66=587; epic-table: 529+66=595; total_stories: 82+9=91; total_waves: 57+4=61 (waves 58-61 added).
# v2.8 (2026-06-24): F3 ENIP EtherNet/IP analyzer INTEGRATE sub-burst (issue #316, feature-enip-v0.11.0) — STORY-130..138 added (E-20, waves 58-61, 66 pts). total_stories 82→91. total_waves 57→61. Wave-table scheduled points 521→587. Epic-table total 529→595. Coverage: all 26 BC-2.17.001..026 assigned. Acyclicity verified: STORY-130/131 (no deps, wave 58) → STORY-132 (dep=130, wave 59) + STORY-133 (dep=131, wave 59) → STORY-134/135/136/137 (dep=132+133, wave 60) → STORY-138 (dep=134+135+136+137, wave 61).
# v2.7 (2026-06-22): F3 issue #64 mitre_attack enrichment — STORY-129 added (E-8, wave 57, 5 pts, depends_on=[]). total_stories 81→82. total_waves 56→57. Wave-table scheduled points 516→521. Epic-table total 524→529. STORY-129 covers BC-2.11.035 (new F2 BC for per-finding mitre_attack JSON array).
# v2.6 (2026-06-20): F3-gate remediation — clarity note added for the three point-total scopes (F3-CV-002); no numeric changes.
# v2.5 (2026-06-20): FE-001 pcapng reader support INTEGRATE sub-burst — E-19 added (STORY-123..128, 6 stories, 37 pts, Waves 51-56).
# ARITHMETIC: 484 pre-FE-001 + 37 (5+8+8+8+5+3) = 521 total_points; wave-table scheduled: 479+37=516; epic-table: 487+37=524; stories: 75+6=81; waves: 50+6=56.
# STORY-091 and STORY-121 remain wave-TBD and are excluded from wave-table scheduled total as before.
# v1.5 totals reconciliation (Pass-26 Slice-D remediation):
# Root cause: the "400 pre-ARP" figure in v1.4 was itself 10 low (actual pre-ARP incl STORY-091 = 410).
# 410 pre-ARP + 47 E-16 ARP = 457 grand total. Wave table (excl STORY-091) = 452.
# All 68 per-story index cells verified against story files — zero mismatches found.
# v1.6 (2026-06-16): E-17 ARP QinQ/MACsec offset hardening — STORY-116 + STORY-117 added.
# 457 pre-E17 + 3 (STORY-116) + 5 (STORY-117) = 465 grand total. Waves 45-46 added.
# STORY-091 wave remains TBD (excluded from wave table as before).
# v1.7 (2026-06-17): E-18 Terminal Finding-Collapse (issue #259) — STORY-118 + STORY-119 added.
# v1.8 (2026-06-17): Adversarial Burst 2 remediation — explicit BC tally added to coverage line; no numeric totals changed.
# 465 pre-E18 + 8 (STORY-118) + 8 (STORY-119 deferred/pts included in total) = 481 total points;
# but STORY-119 is deferred/unscheduled — wave-table scheduled total = 468 + 5 (STORY-091 tooling) = 473.
# STORY-119 deferred (8 pts) not in scheduled wave table total. Wave 47 added for STORY-118.
# v1.9 (2026-06-18): Issue #62 FindingsRender enum migration — STORY-120 added (E-8, wave 48, 3 pts).
# 468 pre-#62 + 3 (STORY-120) = 471 product scheduled; + 5 (STORY-091 tooling) = 476 grand scheduled total.
# STORY-119 deferred (8 pts) not in scheduled total. Wave 48 added for STORY-120.
# STORY-119 depends_on updated from [STORY-118] to [STORY-120] in dependency-graph.md.
# v2.0 (2026-06-18): D-103 F3 human gate — STORY-121 added (E-11, wave TBD, 3 pts, draft).
# STORY-121 is the process-gap self-improvement follow-up for D-099/D-100/D-101.
# E-11 members: STORY-091 + STORY-121 (count 2, points 8). Total stories: 73 → 74.
# Wave-table scheduled total unchanged (STORY-121 wave TBD, excluded like STORY-091).
# v2.1 (2026-06-18): F3 full decomposition of STORY-119 — wave ~ → 49; status draft (deferred) → draft.
# STORY-119 now scheduled at wave 49 (depends_on=[STORY-120], wave 48+1).
# Wave count: 48 → 49. Total scheduled points (excl. STORY-091/STORY-121 wave-TBD): 471 → 479.
# Grand total points: 476 → 484 (471 scheduled product + 8 STORY-091/STORY-121 tooling + 5 unscheduled).
# Note: STORY-119 8 pts now IN the wave-table scheduled total (wave 49 added).
# v2.2 (2026-06-18): F3 adversarial round-5 remediation — Fix 2: stale '~46 sites' in STORY-119
# coverage note (line ~284) corrected to '84 sites' (grepped ground-truth per STORY-119 v1.7).
# Fix 4: legend extended — 'completed' (Index Table) = delivered and merged to develop; equivalent
# to Wave-Delivery-Progress table's DELIVERED & CLOSED. No per-row status cell changes.
# v2.3 (2026-06-18): D-120 STORY-119 split — added STORY-122 (E-18, wave 49, 3 pts, A:
# enum→struct reshape + 84-site migration byte-identical). STORY-119 re-scoped to B (wave 50,
# 5 pts, depends_on STORY-122). total_stories 74→75. total_waves 49→50. Points net zero change
# (STORY-119 8→5 + STORY-122 3pts). E-18 epic entry updated (2→3 stories, 16 pts unchanged).
# v2.4 (2026-06-18): F3-resplit round-2 remediation — Fix 4 (Pass C H-1): epic-table TOTAL
# row Count cell corrected 74→75 (per-epic Count column sums to 75; frontmatter total_stories=75).
traces_to:
  - .factory/stories/dependency-graph.md
  - .factory/stories/epics.md
  - .factory/cycles/v0.1.0-greenfield-spec/wave-schedule.md
  - .factory/feature/wave-schedule.md
---

# wirerust Story Index

> **Authoritative story registry for the v0.1.0-greenfield-spec cycle (48 greenfield product + 1 tooling STORY-091 = 49 stories) + Feature Mode F3 additions (3 stories, STORY-097/098/099 for issue #100) + Feature #7 additions (6 stories, STORY-100..105 for issue #7 Modbus Analyzer) + Feature #8 additions (5 stories, STORY-106..110 for issue #8 DNP3/ICS Analyzer) + Feature #9 additions (5 stories, STORY-111..115 for issue #9 ARP Security Analyzer) + E-17 F3 additions (2 stories, STORY-116..117 for issue #253 ARP QinQ/MACsec Offset Hardening) + E-18 F3 additions (3 stories, STORY-118/STORY-119/STORY-122 for issue #259 Terminal Finding-Collapse — D-120 split: STORY-122=A reshape, STORY-119=B behavioral) + Issue #62 F3 addition (1 story, STORY-120 for issue #62 FindingsRender Enum Migration v0.9.0) + D-103 process-gap follow-up (1 story, STORY-121 for E-11 self-improvement — F1/F2 numeric self-audit checklist) + FE-001 pcapng reader support (6 stories, STORY-123..128 for E-19 pcapng capture-format reader support, Waves 51–56) + Issue #64 F3 addition (1 story, STORY-129 for issue #64 per-finding mitre_attack JSON enrichment, Wave 57, BC-2.11.035) + E-20 EtherNet/IP ENIP/CIP Analyzer (9 stories, STORY-130..138 for issue #316 feature-enip-v0.11.0, Waves 58–61, BC-2.17.001..026).**
> All 48 greenfield stories formalize behavioral contracts for the existing shipped wirerust
> codebase. STORY-097/098/099 are new feature stories for issue #100 (pcap timestamps).
> STORY-100/101 implement E-13 multi-tag Finding schema migration (v0.3.0).
> STORY-102/103/104/105 implement E-14 Modbus TCP analyzer (v0.4.0).
> STORY-106/107/108/109/110 implement E-15 DNP3/ICS analyzer (issue #8).
> STORY-111/112/113/114/115 implement E-16 ARP Security Analyzer (issue #9, v0.7.0).
> Status `draft` = not yet dispatched. Status `pending` = fully decomposed, predecessor(s) merged, queued for dispatch (F4). Status `completed` (Index Table) = delivered and merged to develop; equivalent to the Wave-Delivery-Progress table's **DELIVERED & CLOSED**. Wave assignments are from the
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
| STORY-121 | F1/F2 Story-Input Analysis Docs — Mandatory Numeric Self-Audit + Consuming-Surface Sweep Checklist | E-11 | ~ | 3 | draft | — |
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
| STORY-118 | Terminal Finding-Collapse — Flat Mode (v0.8.0) | E-18 | 47 | 8 | completed | — |
| STORY-119 | Terminal Finding-Collapse — Grouped Mode / --mitre (B: render path + CLI flip) | E-18 | 50 | 5 | completed | STORY-122 |
| STORY-120 | TerminalReporter FindingsRender Enum Migration (v0.9.0) | E-8 | 48 | 3 | completed | — |
| STORY-122 | FindingsRender enum→struct reshape + construction-site migration (byte-identical) | E-18 | 49 | 3 | completed | STORY-120 |
| STORY-123 | pcapng Format Detection (Magic-Byte Probe) and SHB Parse | E-19 | 51 | 5 | completed | — |
| STORY-124 | IDB Parse (Link Type + if_tsresol), Interface Whitelist, and Multi-IDB Agreement | E-19 | 52 | 8 | completed | STORY-123 |
| STORY-125 | EPB Parse and 64-bit Timestamp Normalization (Kani VP-025 + VP-027) | E-19 | 53 | 8 | completed | STORY-123, STORY-124 |
| STORY-126 | SPB Parse, Explicit Block-Skip Dispatch (F-07), and Error-Surface Contract | E-19 | 54 | 8 | completed | STORY-123, STORY-124 |
| STORY-127 | Magic-Byte Glob (resolve_targets Content Detection) and E2E Corpus Wiring | E-19 | 55 | 5 | completed | STORY-123, STORY-124, STORY-125, STORY-126 |
| STORY-128 | main.rs Per-File Error Isolation Loop (Catch-and-Continue) | E-19 | 56 | 3 | completed | STORY-127 |
| STORY-129 | Emit Per-Finding `mitre_attack` Array in JSON Output | E-8 | 57 | 5 | completed | — |
| STORY-130 | EtherNet/IP Pure-Core Parse: ENIP Header, Command Classification, Frame Validity, and Kani VP-032 | E-20 | 58 | 8 | completed | — |
| STORY-131 | EtherNet/IP StreamDispatcher Integration, CLI Flags, and TCP Reassembly Wiring | E-20 | 58 | 8 | completed | — |
| STORY-132 | CPF Item Walk, CIP Header Parse, and CIP Request Path Extraction | E-20 | 59 | 8 | completed | STORY-130 |
| STORY-133 | MITRE ICS Technique Seeding: T0858/T0816/T1693.001/IcsExecution + VP-007 Atomic Update | E-20 | 59 | 5 | completed | STORY-131 |
| STORY-134 | ENIP Recon Detections: T0846 ListIdentity, T0888 Identity Read / Error Burst, and CIP Error Accumulation | E-20 | 60 | 8 | draft | STORY-132, STORY-133 |
| STORY-135 | ENIP Command Detections: T0858 Mode Change, T0816 Device Reset, and T0836 Write-Attribute Burst | E-20 | 60 | 8 | draft | STORY-132, STORY-133 |
| STORY-136 | ENIP Connection Lifecycle: ForwardOpen/ForwardClose Detection | E-20 | 60 | 5 | draft | STORY-132, STORY-133 |
| STORY-137 | ENIP Frame Walk Robustness: Carry Buffer, Non-ENIP Detection, and T0814 DoS Burst | E-20 | 60 | 8 | draft | STORY-132, STORY-133 |
| STORY-138 | ENIP Session Lifecycle, Statistics, DoS Guard, and Analyzer Summary | E-20 | 61 | 8 | draft | STORY-134, STORY-135, STORY-136, STORY-137 |

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
| 47 | STORY-118 | 1 | 8 |
| 48 | STORY-120 | 1 | 3 |
| 49 | STORY-122 | 1 | 3 |
| 50 | STORY-119 | 1 | 5 |
| 51 | STORY-123 | 1 | 5 |
| 52 | STORY-124 | 1 | 8 |
| 53 | STORY-125 | 1 | 8 |
| 54 | STORY-126 | 1 | 8 |
| 55 | STORY-127 | 1 | 5 |
| 56 | STORY-128 | 1 | 3 |
| 57 | STORY-129 | 1 | 5 |
| 58 | STORY-130, STORY-131 | 2 | 16 |
| 59 | STORY-132, STORY-133 | 2 | 13 |
| 60 | STORY-134, STORY-135, STORY-136, STORY-137 | 4 | 29 |
| 61 | STORY-138 | 1 | 8 |
| **TOTAL (excl. STORY-091 wave-TBD, STORY-121 wave-TBD)** | | **89** | **587** |

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
| E-8: Reporting and Output Formats | STORY-076, STORY-077, STORY-078, STORY-079, STORY-080, STORY-120, STORY-129 | 7 | 37 |
| E-9: CLI, Entry Point, and Analysis Orchestration | STORY-086, STORY-087, STORY-088, STORY-089, STORY-090 | 5 | 28 |
| E-10: Absent Behavior Contracts (Flag Rejection) | STORY-096 | 1 | 3 |
| E-11: Tooling and Self-Improvement | STORY-091, STORY-121 | 2 | 8 |
| E-12: Pcap Timestamp Provenance (issue #100) | STORY-097, STORY-098, STORY-099 | 3 | 18 |
| E-13: Multi-Tag Finding Schema Migration (v0.3.0 / issue #7) | STORY-100, STORY-101 | 2 | 21 |
| E-14: Modbus TCP Analyzer (v0.4.0 / issue #7) | STORY-102, STORY-103, STORY-104, STORY-105 | 4 | 37 |
| E-15: DNP3/ICS Analyzer (issue #8) | STORY-106, STORY-107, STORY-108, STORY-109, STORY-110 | 5 | 47 |
| E-16: ARP Security Analyzer (issue #9) | STORY-111, STORY-112, STORY-113, STORY-114, STORY-115 | 5 | 47 |
| E-17: ARP QinQ/MACsec Offset Hardening (issue #253) | STORY-116, STORY-117 | 2 | 8 |
| E-18: Terminal Finding-Collapse (issue #259, v0.8.0) | STORY-118, STORY-122, STORY-119 | 3 | 16 |
| E-19: pcapng Capture-Format Reader Support (FE-001) — **COMPLETE (6/6 MERGED, D-184)** | STORY-123, STORY-124, STORY-125, STORY-126, STORY-127, STORY-128 | 6 | 37 |
| E-20: EtherNet/IP (ENIP/CIP) Analyzer (issue #316, feature-enip-v0.11.0) | STORY-130, STORY-131, STORY-132, STORY-133, STORY-134, STORY-135, STORY-136, STORY-137, STORY-138 | 9 | 66 |
| **TOTAL** | | **91** | **595** |

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
| 47 | STORY-118 | **DELIVERED & CLOSED** | #264 | 5f7cd1b | 2026-06-17 |
| 48 | STORY-120 | **DELIVERED & CLOSED** | #267 | f851995 | 2026-06-18 |
| 49 | STORY-122 | **DELIVERED & CLOSED** | #268 | 8696448 | 2026-06-19 |
| 50 | STORY-119 | **DELIVERED & CLOSED** | #269 | 181d5e2 | 2026-06-19 |
| 51 | STORY-123 | **DELIVERED & CLOSED** | #281 | e4b940b | 2026-06-20 |
| 52 | STORY-124 | **DELIVERED & CLOSED** | #282 | 2f762fda | 2026-06-20 |
| 53 | STORY-125 | **DELIVERED & CLOSED** | #283 | 2c8f2a7 | 2026-06-20 |
| 54 | STORY-126 | **DELIVERED & CLOSED** | #284 | 56a10e9 | 2026-06-20 |
| 55 | STORY-127 | **DELIVERED & CLOSED** | #285 | e802b2e | 2026-06-20 |
| 56 | STORY-128 | **DELIVERED & CLOSED** | #286 | e75a797 | 2026-06-21 |
| 57 | STORY-129 | **DELIVERED & CLOSED** | #306 | 2fa6606 | 2026-06-23 |
| 58 | STORY-130, STORY-131 | **DELIVERED & CLOSED** | #317, #318 | edce3bd | 2026-06-25 |
| 59 | STORY-132, STORY-133 | **DELIVERED & CLOSED** | #319, #320 | 16d3ce7, 7f040de | 2026-06-25 |
| 60 | STORY-134, STORY-135, STORY-136, STORY-137 | draft | — | — | — |
| 61 | STORY-138 | draft | — | — | — |

## Coverage Verification

- Total stories: **91** (48 greenfield product + 1 tooling STORY-091 + 3 F3 feature STORY-097/098/099 + 6 Feature-#7 STORY-100..105 + 5 Feature-#8 STORY-106..110 + 5 Feature-#9 STORY-111..115 + 2 E-17 F3 STORY-116..117 + 3 E-18 F3 STORY-118/STORY-119/STORY-122 + 1 Issue-#62 F3 STORY-120 + 1 D-103 process-gap STORY-121 + 6 E-19 FE-001 pcapng STORY-123..128 + 1 Issue-#64 F3 mitre_attack STORY-129 + 9 E-20 ENIP/CIP STORY-130..138)
- Total waves: **61** (Waves 40–44 added for Feature #9 ARP; Waves 45–46 added for E-17 QinQ/MACsec hardening; Wave 47 added for E-18 finding-collapse STORY-118; Wave 48 added for Issue-#62 STORY-120; Wave 49 added for STORY-122 (D-120 split A: enum→struct reshape); Wave 50 added for STORY-119/B (D-120 split B: render path + CLI flip, 2026-06-18); Waves 51–56 added for E-19 FE-001 pcapng reader support (STORY-123..128, 2026-06-20); Wave 57 added for Issue-#64 STORY-129 (mitre_attack JSON enrichment, 2026-06-22); Waves 58–61 added for E-20 ENIP/CIP analyzer (STORY-130..138, 2026-06-24); STORY-091 wave TBD; STORY-121 wave TBD)
- Total points: **592** (526 pre-ENIP + 66 E-20 STORY-130..138; wave-table scheduled row shows 587 — excl. STORY-091+STORY-121 wave-TBD; epic table shows 595)
  > **Point-scope key (three counts, none wrong):** (1) `total_points: 592` in frontmatter = all 89 scheduled product stories + STORY-091 tooling (5 pts); excludes STORY-121 (wave-TBD, 3 pts). (2) Wave-table "TOTAL" row = **587** = the 89 scheduled product stories only; excludes both STORY-091 and STORY-121 (both wave-TBD). (3) Epic-table "TOTAL" row = **595** = all 91 stories including both STORY-091 (5 pts) and STORY-121 (3 pts). The three counts differ solely by which wave-TBD tooling stories are included.
- Graph is acyclic: **Yes** (Kahn topological sort verified; Feature-#7 dependency chain: STORY-100 → {STORY-101 ∥ STORY-102} → STORY-103 → STORY-104 → STORY-105; Feature-#8 DNP3 chain: STORY-100 → STORY-106 → STORY-107 → STORY-108 → STORY-109 → STORY-110; Feature-#9 ARP chain: STORY-110 → STORY-111 → STORY-112 → STORY-113 → STORY-114 → STORY-115; E-17 hardening chain: STORY-115 → STORY-116 → STORY-117; E-18/E-8 collapse chain: STORY-118 (no predecessor) → STORY-120 (no predecessor) → STORY-122 (wave 49) → STORY-119/B (wave 50); E-19 pcapng chain: STORY-123 (no predecessor) → STORY-124 → {STORY-125 ∥ STORY-126} → STORY-127 → STORY-128; STORY-129 (depends_on=[], wave 57, isolated vertex); E-20 ENIP chain: {STORY-130 ∥ STORY-131} (wave 58, no deps) → {STORY-132 (dep=130) ∥ STORY-133 (dep=131)} (wave 59) → {STORY-134 ∥ STORY-135 ∥ STORY-136 ∥ STORY-137} (dep=132+133, wave 60) → STORY-138 (dep=134+135+136+137, wave 61); no back-edges into existing graph; 91 = 89 product stories + STORY-091 tooling + STORY-121 tooling)
- All 10 product epics + E-11 (Tooling, 2 stories: STORY-091 + STORY-121) + E-12 (Pcap Timestamps) + E-13 (Multi-Tag Migration) + E-14 (Modbus) + E-15 (DNP3) + E-16 (ARP) + E-17 (ARP QinQ/MACsec Hardening) + E-18 (Terminal Finding-Collapse) + E-19 (pcapng Capture-Format Reader Support) + E-20 (EtherNet/IP ENIP/CIP Analyzer) covered: **Yes**
- All 219 greenfield BCs assigned + F2 additions + BC-2.09.001/006 (shared, extended in STORY-100) + BC-2.10.005/007/008 (extended in STORY-100) + BC-2.11.001/013/015/017/020/024 (extended in STORY-101) + BC-2.14.001..025 (new Modbus BCs in STORY-102..105) + BC-2.15.001..024 (new DNP3 BCs in STORY-106..110) + BC-2.02.009 (revised in STORY-111) + BC-2.16.001..015 (new ARP BCs in STORY-111..115) + BC-2.16.009 v1.10 EC-009 / BC-2.16.015 v1.9 EC-009 (E-17 MACsec documented-limitation extensions in STORY-116/117) + BC-2.11.025/026/027/028/029 (new E-18 collapse BCs in STORY-118; BC-2.11.010/013/017/019 extended/versioned — not new BCs) + BC-2.11.030/031/032/033/034 (5 new grouped-mode-collapse BCs in STORY-119/B) + BC-2.11.035 (new issue-#64 mitre_attack enrichment BC in STORY-129) + BC-2.17.001..026 (26 new ENIP/CIP BCs in STORY-130..138, excl. STORY-133 VP-007 obligation): **Yes** (total 320 BCs; explicit tally: 219 greenfield + 25 Modbus + 24 DNP3 + 15 ARP + 5 E-18 flat-collapse + 5 E-18 grouped-collapse + 1 issue-#64 mitre_attack + 26 ENIP = 320; E-17 + all extensions = versioning of existing BCs, +0; D-120 split adds no new BCs)
- PROCESS-GAP-P5-001 dispositioned: **Yes** — STORY-091 created as the S-7.02 cycle-close disposition
- Coverage note: STORY-097/098/099 trace to BC-2.04.055 and BC-2.09.007 (both F2 additions); these 3 stories cover VP-021 (verified @256a490). STORY-100 extends BC-2.09.001 (field rename) and BC-2.10.005/007/008 (catalog seed to 21). STORY-101 extends BC-2.11.001/013/015/017/020/024 (reporter multi-tag). STORY-102..105 cover BC-2.14.001..025 (Modbus TCP analyzer). STORY-106..110 cover BC-2.15.001..024 (DNP3/ICS analyzer); VP-023 Kani lands in STORY-106, VP-004 oracle obligation lands in STORY-110, VP-007 atomic-update obligation (SEEDED 21→23, EMITTED 13→15) lands in STORY-109. STORY-111..115 cover BC-2.16.001..015 (ARP Security Analyzer): STORY-111 covers BC-2.02.009 (revised) + decoder migration; STORY-112 covers BC-2.16.001/002/015 (VP-024 Sub-A Kani); STORY-113 covers BC-2.16.003/005/006/007/009/010/011; STORY-114 covers BC-2.16.004/012/014 (VP-007 SEEDED 23→25 / EMITTED 15→17) + BC-2.16.007 D12-MITRE extension; STORY-115 covers BC-2.16.008/013 + BC-2.16.010 extension. STORY-116 covers BC-2.16.009 EC-008 (QinQ D11) + BC-2.16.015 PC-7b/EC-008 (QinQ offset-22 formula pin) + BC-2.16.015 EC-009(a) (MACsec observe-only probe, no-SCI shape guard). STORY-117 covers BC-2.16.009 EC-009(a) (MACsec offset 22/30 assertion + D11 routing) + BC-2.16.015 EC-009(a/b) (SCI-present spec-backing test; Modified/Encrypted opaque-unreachable security guards); both stories reference VP-024 lifecycle note (append-only, no proof change). STORY-118 covers BC-2.11.025/026/027/028/029 (5 new E-18 flat-collapse BCs) + extensions to BC-2.11.010/013/017/019 (collapse-path interaction clauses). VP-012 (escape_for_terminal) extended by STORY-118 (collapse evidence path). STORY-122 (wave 49, D-120 split A): FindingsRender enum→struct reshape (84 sites); 4-arm dispatch with TEMPORARY {Grouped,Collapsed} arm; byte-identical output; VP-016 (existing tactic-order tests continue to pass). STORY-119/B (wave 50, D-120 split B): covers BC-2.11.030/031/032/033/034 (5 new grouped-mode-collapse BCs) + BC-2.11.013/014/016/025/026/027/028 (re-anchored to struct-of-enums vocabulary); render_findings_grouped_collapsed new function; VP-012 (grouped-collapse escape path, AC-023) + VP-016 (tactic order preserved under grouped-collapse, test_BC_2_11_033_grouped_collapsed_preserves_bucket_order). STORY-129 (wave 57, issue #64): covers BC-2.11.035 (new per-finding mitre_attack JSON enrichment); adds FindingJsonDto wrapper + technique_tactic_id catalog accessor; 10 unit tests (AC-1..AC-10, DF-AC-TEST-NAME-SYNC-001); extends vp007_catalog_drift_guard for tactic_id mapping; no predecessor. STORY-130..138 cover BC-2.17.001..026 (26 new ENIP/CIP BCs, SS-17 EtherNet/IP analyzer): STORY-130 covers BC-2.17.001/002/003/004 (pure-core ENIP header parse + VP-032 Kani); STORY-131 covers BC-2.17.019/020/023/026 (StreamDispatcher integration + CLI flags, wave 58); STORY-132 covers BC-2.17.005/006/007/009 (CPF item walk + CIP header parse + path extraction, wave 59) + VP-032 Sub-D Kani (classify_cip_service totality + partition harnesses — F4-01 remediation, input-hash 9df8cea); STORY-133 carries VP-007 atomic obligation (T0858/T0816/T1693.001/IcsExecution seeding, wave 59; no BC in frontmatter — obligation driven by ADR-010 Decision 7); STORY-134 covers BC-2.17.010/008/014 (recon detections T0846+T0888+error-burst, wave 60); STORY-135 covers BC-2.17.011/013/012 (command detections T0858+T0816+T0836, wave 60); STORY-136 covers BC-2.17.015 (ForwardOpen/ForwardClose lifecycle, wave 60); STORY-137 covers BC-2.17.016/018 (frame walk robustness + carry buffer + T0814 DoS burst, wave 60); STORY-138 covers BC-2.17.025/017/021/022/024 (session lifecycle + stats + MAX_FINDINGS guard + summary, wave 61). VP-032 (ENIP parse safety Kani) split: Sub-A/B/C anchored in STORY-130; Sub-D (classify_cip_service) anchored in STORY-132 (F4-01).
- Release mapping: v0.3.0 ships after Wave 31 gate (STORY-100 + STORY-101 merged); v0.4.0 ships after Wave 34 gate (STORY-102..105 merged); v0.6.0 ships after Wave 39 gate (STORY-106..110 merged); v0.7.0 ships after Wave 44 gate (STORY-111..115 merged); v0.7.1 ships after Wave 46 gate (STORY-116 + STORY-117 merged — E-17 test-and-docs patch); **v0.8.0 ships after Wave 47 gate (STORY-118 merged with ADR-0003 Display-Layer Aggregation section)**; **v0.9.0 ships after Wave 48 gate (STORY-120 merged, Cargo.toml version bumped to 0.9.0, cargo-semver-checks struct_field_missing documented)**; **v0.9.x/v0.10.0-pre ships after Wave 49 gate (STORY-122 merged — FindingsRender enum→struct reshape, byte-identical, no externally observable behavior change)**; **v0.10.0 ships after Wave 50 gate (STORY-119/B merged — grouped-mode collapse, --mitre default-collapse behavior change)**; **v0.11.0 ships after Wave 61 gate (STORY-130..138 merged — EtherNet/IP ENIP/CIP analyzer, issue #316)**. Note: STORY-129 (mitre_attack enrichment, issue #64, Wave 57) was folded into v0.11.0 pre-releases and merged as part of the ENIP preparation. (v0.5.0 was the MITRE-drift-guard fix released separately; DNP3 targets v0.6.0; ARP targets v0.7.0; ARP QinQ/MACsec hardening targets v0.7.1.)
- Existing stories affected by schema migration: STORY-069/070/071/078/079/080 — their MITRE-technique test assertions migrate from `mitre_technique: Option<String>` to `mitre_techniques: Vec<String>` via STORY-100 (see revision notes in each story). STORY-111 revises BC-2.02.009 to add the third decode path (ArpFrame); existing STORY-002/003 passing tests are unaffected.
