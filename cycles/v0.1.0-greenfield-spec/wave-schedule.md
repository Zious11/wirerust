---
document_type: wave-schedule
level: ops
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/stories/STORY-INDEX.md
  - .factory/stories/dependency-graph.md
traces_to: .factory/stories/STORY-INDEX.md
cycle: v0.1.0-greenfield-spec
---

# Wave Schedule: wirerust v0.1.0-greenfield-spec

> **Brownfield context:** wirerust is a single-crate offline pcap forensic triage CLI.
> All 48 stories formalize behavioral contracts for existing shipped code.
> Waves are computed using longest-path (critical-path) method from the acyclic
> dependency graph in `dependency-graph.md`. Kahn topological sort confirmed acyclic.

---

## Summary

| Metric | Value |
|--------|-------|
| Total stories | 48 |
| Total waves | 27 |
| Max parallelism | 4 stories (Wave 16, Wave 18) |
| Total story points | 282 |
| Estimated agent spawns | 48 (one per story, strict TDD mode) |
| Graph is acyclic | Yes (Kahn verified) |

---

## Wave Plan

### Wave 1 — 2 stories | Epics: E-1, E-7 | No dependencies

| Story | Epic | Points | Description |
|-------|------|--------|-------------|
| STORY-001 | E-1 | 5 | PCAP File Ingestion — Link-Type Gating, Eager Packet Load, and Error Surfaces |
| STORY-069 | E-7 | 5 | Finding Struct, Verdict/Confidence Display, and Finding Display Format |

> Both are foundation stories with no dependencies. E-1 and E-7 can proceed in
> parallel since the Finding data model has no runtime dependency on the pipeline.

---

### Wave 2 — 4 stories | Epics: E-1, E-7 | Depends on Wave 1

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-002 | E-1 | 5 | Packet Decoding — Ethernet, RAW/IPV4, and IPv6 Link-Layer Paths | STORY-001 |
| STORY-003 | E-1 | 5 | Packet Decoding — Linux SLL, No-Panic Safety, and Non-IP Frame Rejection | STORY-001 |
| STORY-004 | E-1 | 3 | Packet Decoding — ICMP, Protocol::Other, and app_protocol_hint Port Table | STORY-001 |
| STORY-070 | E-7 | 5 | Raw-Data Contract and JSON Serialization Symmetry (skip_serializing_if) | STORY-069 |

> STORY-002, STORY-003, STORY-004 are parallel link-layer decode stories.
> STORY-070 proceeds as soon as STORY-069 is done.

---

### Wave 3 — 2 stories | Epics: E-1, E-7 | Depends on Wave 2

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-005 | E-1 | 3 | Packet Decoding — packet_len Semantics and TCP Flag/Sequence Extraction | STORY-002, STORY-003, STORY-004 |
| STORY-071 | E-7 | 8 | MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order | STORY-069, STORY-070 |

> STORY-005 (E-1 integration) and STORY-071 (MITRE table) proceed in parallel.

---

### Wave 4 — 2 stories | Epics: E-2, E-6 | Depends on Wave 3

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-011 | E-2 | 5 | TcpReassembler Constructor Validation and FlowKey Canonicalization | STORY-005 |
| STORY-066 | E-6 | 5 | DNS Traffic Statistics — Port-53 Dispatch, QR-Bit Counting, and Never-Emit Contract | STORY-005 |

> Cross-epic: STORY-005 (E-1) feeds both STORY-011 (E-2 TCP reassembly) and
> STORY-066 (E-6 DNS, packet-level bypass of E-2/E-3). Both proceed in parallel.

---

### Wave 5 — 1 story | Epic: E-2 | Depends on Wave 4

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-012 | E-2 | 5 | Non-TCP Packet Filter, Statistics Summary, and bytes_reassembled Accounting | STORY-011 |

---

### Wave 6 — 1 story | Epic: E-2 | Depends on Wave 5

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-013 | E-2 | 8 | TCP Three-Way Handshake State Machine and Direction Tagging | STORY-012 |

---

### Wave 7 — 1 story | Epic: E-2 | Depends on Wave 6

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-014 | E-2 | 5 | Mid-Stream Join, ISN Management, and IsnMissing Guard | STORY-013 |

---

### Wave 8 — 2 stories | Epic: E-2 | Depends on Wave 7

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-015 | E-2 | 8 | In-Order Delivery, Out-of-Order Buffering, and Bidirectional Direction Tagging | STORY-013, STORY-014 |
| STORY-019 | E-2 | 5 | Flow Lifecycle — RST Close, FIN Close, Timeout Expiry, and Missing-Key Warning | STORY-013, STORY-014 |

> STORY-015 (delivery logic) and STORY-019 (flow lifecycle) touch different
> parts of SS-04 and proceed in parallel once STORY-014 is done.

---

### Wave 9 — 2 stories | Epic: E-2 | Depends on Wave 8

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-016 | E-2 | 8 | Overlap Detection — Duplicate Retransmissions, Partial Overlap, and buffered_bytes Accounting | STORY-015 |
| STORY-020 | E-2 | 8 | Memory Management — total_memory Accounting and LRU Eviction Policies | STORY-019 |

> STORY-016 (overlap detection) and STORY-020 (memory management) proceed in parallel.

---

### Wave 10 — 2 stories | Epic: E-2 | Depends on Wave 9

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-017 | E-2 | 8 | Conflict and Evasion Detection — T1036 Findings and One-Shot Anomaly Latches | STORY-015, STORY-016 |
| STORY-018 | E-2 | 8 | Resource Bounds — Depth Truncation, Out-of-Window Rejection, and Segment Limit Enforcement | STORY-015, STORY-016 |

> STORY-017 (evasion detection) and STORY-018 (resource bounds) proceed in parallel.

---

### Wave 11 — 1 story | Epic: E-2 | Depends on Wave 10

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-021 | E-2 | 5 | Finalize Lifecycle, MAX_FINDINGS Cap, and Segment-Limit Summary Finding | STORY-017, STORY-018, STORY-019, STORY-020 |

> E-2 completes here. Cross-epic edge E-2 -> E-3 now enabled.

---

### Wave 12 — 1 story | Epic: E-3 | Depends on Wave 11

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-031 | E-3 | 5 | Content-First Classification — TLS Signature, HTTP Method Prefix, Port Fallback | STORY-021 |

> Cross-epic: STORY-021 (E-2 stats/summary) feeds STORY-031 (E-3 StreamDispatcher).

---

### Wave 13 — 1 story | Epic: E-3 | Depends on Wave 12

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-032 | E-3 | 5 | Classification Caching and DispatchTarget::None Retry Budget | STORY-031 |

---

### Wave 14 — 1 story | Epic: E-3 | Depends on Wave 13

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-033 | E-3 | 3 | Flow Lifecycle — Close, Unclassified Counter, No-Op Dispatcher | STORY-031, STORY-032 |

> E-3 completes here. Cross-epic edges E-3 -> E-4 and E-3 -> E-5 now enabled.
> Also requires STORY-071 (E-7) which completed in Wave 3.

---

### Wave 15 — 2 stories | Epics: E-4, E-5 | Depends on Waves 3 + 14

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-041 | E-4 | 8 | HTTP/1.1 Request/Response Parsing and Core Statistics | STORY-033, STORY-071 |
| STORY-051 | E-5 | 5 | JA3 and JA3S Computation — GREASE Filtering and String Format | STORY-033, STORY-071 |

> E-4 and E-5 root stories both require STORY-033 (E-3 dispatch) and STORY-071
> (E-7 MITRE types). They proceed in parallel once both predecessors are done.

---

### Wave 16 — 4 stories | Epics: E-4, E-5 | Depends on Wave 15

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-042 | E-4 | 5 | URI-Based Threat Detections — Path Traversal, Web Shell, Admin Panel | STORY-041 |
| STORY-043 | E-4 | 5 | Header and Method Anomaly Detections — Method, Host, URI Length, User-Agent | STORY-041 |
| STORY-044 | E-4 | 8 | Parse-Error Isolation and Poisoning State Machine | STORY-041 |
| STORY-052 | E-5 | 8 | ClientHello Parsing — Handshake Counting, Version/JA3 Tracking, and Done Short-Circuit | STORY-051 |

> Maximum parallelism wave: 4 stories from 2 epics proceed in parallel.

---

### Wave 17 — 3 stories | Epics: E-4, E-5 | Depends on Wave 16

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-045 | E-4 | 5 | Flow Lifecycle, Cross-Flow Isolation, and Buffer/Map Caps | STORY-041, STORY-044 |
| STORY-053 | E-5 | 5 | ServerHello Parsing — JA3S Fingerprinting and Cipher/Version Tracking | STORY-051, STORY-052 |
| STORY-055 | E-5 | 8 | SNI Classification Arms 1 and 2 — Clean ASCII Baseline and C0/DEL Control-Byte Detection | STORY-052 |

---

### Wave 18 — 4 stories | Epics: E-4, E-5 | Depends on Wave 17

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-046 | E-4 | 3 | HTTP Analyzer Summary Output | STORY-041, STORY-042, STORY-043, STORY-044, STORY-045 |
| STORY-054 | E-5 | 8 | Cipher and Protocol Weakness Findings — Weak Ciphers, Deprecated SSL Versions, and Baseline Zero-Finding | STORY-052, STORY-053 |
| STORY-056 | E-5 | 8 | SNI Classification Arms 3 and 4 — Non-ASCII UTF-8 and Non-UTF-8 Byte Preservation | STORY-055 |
| STORY-058 | E-5 | 8 | Buffer Management, Record Parsing Infrastructure, Flow Lifecycle, and summarize Output | STORY-052, STORY-053 |

> Maximum parallelism wave: 4 stories, 2 from E-4 (STORY-046 closes E-4) and 2 from E-5.

---

### Wave 19 — 1 story | Epic: E-5 | Depends on Wave 18

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-057 | E-5 | 8 | SNI Edge Cases — Empty Lists, Empty Hostnames, Multi-Name, NameType, Trailing Bytes, Large SNI, and Count-Cap Decoupling | STORY-055, STORY-056 |

> E-5 completes after Wave 19. Cross-epic edges E-4/E-5/E-6/E-7 -> E-8 now enabled.

---

### Wave 20 — 1 story | Epic: E-8 | Depends on Waves 4 + 19 + (E-7 Wave 3)

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-076 | E-8 | 5 | JsonReporter — Structure, skipped_packets, and RFC 8259 Byte Handling | STORY-046, STORY-057, STORY-058, STORY-066, STORY-071 |

> Cross-epic: STORY-076 requires completed findings from E-4 (STORY-046), E-5
> (STORY-057, STORY-058), E-6 (STORY-066), and E-7 (STORY-071).

---

### Wave 21 — 2 stories | Epic: E-8 | Depends on Wave 20

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-077 | E-8 | 8 | TerminalReporter — escape_for_terminal, skipped_packets, and End-to-End C1 Safety | STORY-076 |
| STORY-079 | E-8 | 5 | CsvReporter — Fixed 9-Column Schema, CSV-Injection Neutralization, and Evidence Join | STORY-076 |

> STORY-077 (terminal escape) and STORY-079 (CSV reporter) proceed in parallel.

---

### Wave 22 — 2 stories | Epic: E-8 | Depends on Wave 21

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-078 | E-8 | 8 | TerminalReporter — MITRE Grouping, Section Order, and Colorization | STORY-077 |
| STORY-080 | E-8 | 3 | CsvReporter — Reporter Trait Compliance and Optional Field Encoding | STORY-079 |

> E-8 completes after Wave 22. Cross-epic edge E-8 -> E-9 now enabled.

---

### Wave 23 — 1 story | Epic: E-9 | Depends on Wave 22

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-086 | E-9 | 5 | CLI Subcommand Parsing — analyze, summary, --no-color, Multiple Targets | STORY-080 |

> Cross-epic: STORY-086 requires the complete Reporter layer from E-8 (via STORY-080).
> STORY-080 provides transitive coverage of all upstream epics E-1..E-7.

---

### Wave 24 — 2 stories | Epics: E-9, E-10 | Depends on Wave 23

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-087 | E-9 | 5 | Output Format Flags and Reassembly Configuration Flags | STORY-086 |
| STORY-096 | E-10 | 3 | Absent Behavior Contracts — Removed Flags Rejected by clap | STORY-086 |

> STORY-087 (flag parsing) and STORY-096 (absent-behavior tests) both require only
> the base CLI from STORY-086 and proceed in parallel. STORY-096 closes E-10.

---

### Wave 25 — 1 story | Epic: E-9 | Depends on Wave 24

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-088 | E-9 | 8 | run_analyze Orchestration — Analyzer Enablement, Reassembly Logic, Target Expansion, Progress Bar | STORY-086, STORY-087 |

---

### Wave 26 — 1 story | Epic: E-9 | Depends on Wave 25

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-089 | E-9 | 5 | Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing | STORY-086, STORY-087, STORY-088 |

---

### Wave 27 — 1 story | Epic: E-9 | Depends on Wave 26

| Story | Epic | Points | Description | Blocked By |
|-------|------|--------|-------------|------------|
| STORY-090 | E-9 | 5 | Summary Data Model — ingest, Service Hints, unique_hosts, Serialization | STORY-086, STORY-088, STORY-089 |

> E-9 completes. All 48 stories delivered. Full pipeline validated end-to-end.

---

## Wave Summary Table

| Wave | Stories | Story IDs | Points | Max Parallel |
|------|---------|-----------|--------|-------------|
| 1 | 2 | STORY-001, STORY-069 | 10 | 2 |
| 2 | 4 | STORY-002, STORY-003, STORY-004, STORY-070 | 18 | 4 |
| 3 | 2 | STORY-005, STORY-071 | 11 | 2 |
| 4 | 2 | STORY-011, STORY-066 | 10 | 2 |
| 5 | 1 | STORY-012 | 5 | 1 |
| 6 | 1 | STORY-013 | 8 | 1 |
| 7 | 1 | STORY-014 | 5 | 1 |
| 8 | 2 | STORY-015, STORY-019 | 13 | 2 |
| 9 | 2 | STORY-016, STORY-020 | 16 | 2 |
| 10 | 2 | STORY-017, STORY-018 | 16 | 2 |
| 11 | 1 | STORY-021 | 5 | 1 |
| 12 | 1 | STORY-031 | 5 | 1 |
| 13 | 1 | STORY-032 | 5 | 1 |
| 14 | 1 | STORY-033 | 3 | 1 |
| 15 | 2 | STORY-041, STORY-051 | 13 | 2 |
| 16 | 4 | STORY-042, STORY-043, STORY-044, STORY-052 | 26 | 4 |
| 17 | 3 | STORY-045, STORY-053, STORY-055 | 18 | 3 |
| 18 | 4 | STORY-046, STORY-054, STORY-056, STORY-058 | 27 | 4 |
| 19 | 1 | STORY-057 | 8 | 1 |
| 20 | 1 | STORY-076 | 5 | 1 |
| 21 | 2 | STORY-077, STORY-079 | 13 | 2 |
| 22 | 2 | STORY-078, STORY-080 | 11 | 2 |
| 23 | 1 | STORY-086 | 5 | 1 |
| 24 | 2 | STORY-087, STORY-096 | 8 | 2 |
| 25 | 1 | STORY-088 | 8 | 1 |
| 26 | 1 | STORY-089 | 5 | 1 |
| 27 | 1 | STORY-090 | 5 | 1 |
| **TOTAL** | **48** | | **282** | |

---

## Critical Path

The longest chain of dependent stories:

```
STORY-001 (W1, 5pts)
  -> STORY-002/003/004 (W2, parallel)
    -> STORY-005 (W3, 3pts)
      -> STORY-011 (W4, 5pts)
        -> STORY-012 (W5, 5pts)
          -> STORY-013 (W6, 8pts)
            -> STORY-014 (W7, 5pts)
              -> STORY-015 (W8, 8pts)
                -> STORY-016 (W9, 8pts)
                  -> STORY-017/018 (W10, parallel, 8pts each)
                    -> STORY-021 (W11, 5pts)
                      -> STORY-031 (W12, 5pts)
                        -> STORY-032 (W13, 5pts)
                          -> STORY-033 (W14, 3pts)
                            -> STORY-041/051 (W15, parallel)
                              -> STORY-052 (W16, 8pts, on E-5 chain)
                                -> STORY-055 (W17, 8pts)
                                  -> STORY-056 (W18, 8pts)
                                    -> STORY-057 (W19, 8pts)
                                      -> STORY-076 (W20, 5pts)
                                        -> STORY-077/079 (W21, parallel)
                                          -> STORY-078/080 (W22, parallel)
                                            -> STORY-086 (W23, 5pts)
                                              -> STORY-087/096 (W24, parallel)
                                                -> STORY-088 (W25, 8pts)
                                                  -> STORY-089 (W26, 5pts)
                                                    -> STORY-090 (W27, 5pts)
```

**Critical path length: 27 waves**
**Critical path total points (longest chain): ~150pts** (E-2 chain through E-5 to E-9)

---

## Pipeline Overlap Plan

| Parallel Activity | When | Precondition |
|------------------|------|--------------|
| E-7 (STORY-069, 070, 071) | Waves 1-3 | Independent of E-1; proceed immediately |
| E-6 (STORY-066) | Wave 4 | Requires STORY-005 (E-1 complete) |
| E-2 formalization | Waves 4-11 | Requires STORY-005; sequential within E-2 |
| E-4 and E-5 in parallel | Waves 15-18/19 | Requires STORY-033 + STORY-071 |
| E-8 (Reporting) | Waves 20-22 | Requires E-4, E-5, E-6, E-7 all complete |
| E-10 (Absent behaviors) | Wave 24 | Requires STORY-086 (E-9 base CLI only) |
