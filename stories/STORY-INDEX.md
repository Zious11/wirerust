---
document_type: story-index
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
---

# wirerust Story Index

> Auto-generated index. Covers E-1 (PCAP Ingestion and Packet Decoding), E-6 (DNS Traffic Statistics),
> E-7 (Forensic Finding Data Model and MITRE Mapping), E-8 (Reporting and Output Formats),
> E-9 (CLI, Entry Point, and Analysis Orchestration), and E-10 (Absent Behavior Contracts).

## E-1: PCAP Ingestion and Packet Decoding (SS-01, SS-02)

| ID | Title | Epic | Wave | Points | Status | Dependencies |
|----|-------|------|------|--------|--------|-------------|
| STORY-001 | PCAP File Ingestion — Link-Type Gating, Eager Packet Load, and Error Surfaces | E-1 | 1 | 5 | draft | none |
| STORY-002 | Packet Decoding — Ethernet, RAW/IPV4, and IPv6 Link-Layer Paths | E-1 | 2 | 5 | draft | STORY-001 |
| STORY-003 | Packet Decoding — Linux SLL, No-Panic Safety, and Non-IP Frame Rejection | E-1 | 2 | 5 | draft | STORY-001 |
| STORY-004 | Packet Decoding — ICMP, Protocol::Other, and app_protocol_hint Port Table | E-1 | 2 | 3 | draft | STORY-001 |
| STORY-005 | Packet Decoding — packet_len Semantics and TCP Flag/Sequence Extraction | E-1 | 3 | 3 | draft | STORY-002, STORY-003, STORY-004 |

**E-1 Total:** 5 stories, 21 points, 3 waves

## E-8: Reporting and Output Formats (SS-11)

| ID | Title | Epic | Wave | Points | Status | Dependencies |
|----|-------|------|------|--------|--------|-------------|
| STORY-076 | JsonReporter — Structure, skipped_packets, and RFC 8259 Byte Handling | E-8 | 1 | 5 | draft | none |
| STORY-077 | TerminalReporter — escape_for_terminal, skipped_packets, and End-to-End C1 Safety | E-8 | 2 | 8 | draft | STORY-076 |
| STORY-078 | TerminalReporter — MITRE Grouping, Section Order, and Colorization | E-8 | 3 | 8 | draft | STORY-077 |
| STORY-079 | CsvReporter — Fixed 9-Column Schema, CSV-Injection Neutralization, and Evidence Join | E-8 | 2 | 5 | draft | STORY-076 |
| STORY-080 | CsvReporter — Reporter Trait Compliance and Optional Field Encoding | E-8 | 3 | 3 | draft | STORY-079 |

**E-8 Total:** 5 stories, 29 points, 3 waves

## E-6: DNS Traffic Statistics (SS-08)

| ID | Title | Epic | Wave | Points | Status | Dependencies |
|----|-------|------|------|--------|--------|-------------|
| STORY-066 | DNS Traffic Statistics — Port-53 Dispatch, QR-Bit Counting, and Never-Emit Contract | E-6 | 1 | 5 | draft | none |

**E-6 Total:** 1 story, 5 points, 1 wave

## E-7: Forensic Finding Data Model and MITRE Mapping (SS-09, SS-10)

| ID | Title | Epic | Wave | Points | Status | Dependencies |
|----|-------|------|------|--------|--------|-------------|
| STORY-069 | Finding Struct, Verdict/Confidence Display, and Finding Display Format | E-7 | 1 | 5 | draft | none |
| STORY-070 | Raw-Data Contract and JSON Serialization Symmetry (skip_serializing_if) | E-7 | 2 | 5 | draft | STORY-069 |
| STORY-071 | MITRE ATT&CK Mapping — Tactic Display, Catalog Lookup, all_tactics_in_report_order | E-7 | 3 | 8 | draft | STORY-069, STORY-070 |

**E-7 Total:** 3 stories, 18 points, 3 waves

## E-9: CLI, Entry Point, and Analysis Orchestration (SS-12)

| ID | Title | Epic | Wave | Points | Status | Dependencies |
|----|-------|------|------|--------|--------|-------------|
| STORY-086 | CLI Subcommand Parsing — analyze, summary, --no-color, Multiple Targets | E-9 | 1 | 5 | draft | none |
| STORY-087 | Output Format Flags and Reassembly Configuration Flags | E-9 | 2 | 5 | draft | STORY-086 |
| STORY-088 | run_analyze Orchestration — Analyzer Enablement, Reassembly Logic, Target Expansion, Progress Bar | E-9 | 3 | 8 | draft | STORY-086, STORY-087 |
| STORY-089 | Decode Error Counting, Dispatcher Stats Injection, Format Resolution, and Output Routing | E-9 | 4 | 5 | draft | STORY-086, STORY-087, STORY-088 |
| STORY-090 | Summary Data Model — ingest, Service Hints, unique_hosts, Serialization | E-9 | 5 | 5 | draft | STORY-086, STORY-088, STORY-089 |

**E-9 Total:** 5 stories, 28 points, 5 waves

## E-10: Absent Behavior Contracts (Flag Rejection) (SS-13)

| ID | Title | Epic | Wave | Points | Status | Dependencies |
|----|-------|------|------|--------|--------|-------------|
| STORY-096 | Absent Behavior Contracts — Removed Flags Rejected by clap | E-10 | 1 | 3 | draft | none |

**E-10 Total:** 1 story, 3 points, 1 wave

---

## BC Coverage Summary

### E-1 Coverage (23 BCs)

| BC | Story |
|----|-------|
| BC-2.01.001 | STORY-001 |
| BC-2.01.002 | STORY-001 |
| BC-2.01.003 | STORY-001 |
| BC-2.01.004 | STORY-001 |
| BC-2.01.005 | STORY-001 |
| BC-2.01.006 | STORY-001 |
| BC-2.01.007 | STORY-001 |
| BC-2.01.008 | STORY-001 |
| BC-2.02.001 | STORY-002 |
| BC-2.02.002 | STORY-002 |
| BC-2.02.003 | STORY-002 |
| BC-2.02.004 | STORY-002 |
| BC-2.02.005 | STORY-002 |
| BC-2.02.006 | STORY-003 |
| BC-2.02.007 | STORY-003 |
| BC-2.02.008 | STORY-003 |
| BC-2.02.009 | STORY-003 |
| BC-2.02.010 | STORY-004 |
| BC-2.02.011 | STORY-004 |
| BC-2.02.012 | STORY-004 |
| BC-2.02.013 | STORY-004 |
| BC-2.02.014 | STORY-005 |
| BC-2.02.015 | STORY-005 |

**E-1 Coverage: 23 / 23 BCs covered**

### E-8 Coverage (24 BCs)

| BC | Story |
|----|-------|
| BC-2.11.001 | STORY-076 |
| BC-2.11.002 | STORY-076 |
| BC-2.11.003 | STORY-076 |
| BC-2.11.004 | STORY-076 |
| BC-2.11.005 | STORY-076 |
| BC-2.11.006 | STORY-077 |
| BC-2.11.007 | STORY-077 |
| BC-2.11.008 | STORY-077 |
| BC-2.11.009 | STORY-077 |
| BC-2.11.010 | STORY-077 |
| BC-2.11.011 | STORY-077 |
| BC-2.11.012 | STORY-077 |
| BC-2.11.013 | STORY-078 |
| BC-2.11.014 | STORY-078 |
| BC-2.11.015 | STORY-078 |
| BC-2.11.016 | STORY-078 |
| BC-2.11.017 | STORY-078 |
| BC-2.11.018 | STORY-078 |
| BC-2.11.019 | STORY-078 |
| BC-2.11.020 | STORY-079 |
| BC-2.11.021 | STORY-079 |
| BC-2.11.022 | STORY-079 |
| BC-2.11.023 | STORY-080 |
| BC-2.11.024 | STORY-080 |

**E-8 Coverage: 24 / 24 BCs covered**

### E-6 Coverage (4 BCs)

| BC | Story |
|----|-------|
| BC-2.08.001 | STORY-066 |
| BC-2.08.002 | STORY-066 |
| BC-2.08.003 | STORY-066 |
| BC-2.08.004 | STORY-066 |

**E-6 Coverage: 4 / 4 BCs covered**

### E-7 Coverage (15 BCs)

| BC | Story |
|----|-------|
| BC-2.09.001 | STORY-069 |
| BC-2.09.002 | STORY-069 |
| BC-2.09.003 | STORY-069 |
| BC-2.09.004 | STORY-069 |
| BC-2.09.005 | STORY-070 |
| BC-2.09.006 | STORY-070 |
| BC-2.10.001 | STORY-071 |
| BC-2.10.002 | STORY-071 |
| BC-2.10.003 | STORY-071 |
| BC-2.10.004 | STORY-071 |
| BC-2.10.005 | STORY-071 |
| BC-2.10.006 | STORY-071 |
| BC-2.10.007 | STORY-071 |
| BC-2.10.008 | STORY-071 |
| BC-2.10.009 | STORY-071 |

**E-7 Coverage: 15 / 15 BCs covered**

### E-9 Coverage (21 BCs)

| BC | Story |
|----|-------|
| BC-2.12.001 | STORY-086 |
| BC-2.12.002 | STORY-086 |
| BC-2.12.003 | STORY-086 |
| BC-2.12.004 | STORY-087 |
| BC-2.12.005 | STORY-087 |
| BC-2.12.006 | STORY-086 |
| BC-2.12.007 | STORY-087 |
| BC-2.12.008 | STORY-088 |
| BC-2.12.009 | STORY-088 |
| BC-2.12.010 | STORY-088 |
| BC-2.12.011 | STORY-088 |
| BC-2.12.012 | STORY-088 |
| BC-2.12.013 | STORY-088 |
| BC-2.12.014 | STORY-089 |
| BC-2.12.015 | STORY-089 |
| BC-2.12.016 | STORY-089 |
| BC-2.12.017 | STORY-089 |
| BC-2.12.018 | STORY-090 |
| BC-2.12.019 | STORY-090 |
| BC-2.12.020 | STORY-090 |
| BC-2.12.021 | STORY-090 |

**E-9 Coverage: 21 / 21 BCs covered**

### E-10 Coverage (4 BCs)

| BC | Story |
|----|-------|
| BC-2.13.001 | STORY-096 |
| BC-2.13.002 | STORY-096 |
| BC-2.13.003 | STORY-096 |
| BC-2.13.004 | STORY-096 |

**E-10 Coverage: 4 / 4 BCs covered**

---

## Dependency Graph (Topological Sort)

### E-1 Wave Schedule

```
Wave 1: STORY-001 (no deps)
Wave 2: STORY-002, STORY-003, STORY-004 (all depend on STORY-001 only; parallel)
Wave 3: STORY-005 (depends on STORY-002, STORY-003, STORY-004)
```

Acyclicity verified: STORY-001 -> STORY-002/003/004 -> STORY-005. No cycles.

### E-8 Wave Schedule

```
Wave 1: STORY-076 (no deps)
Wave 2: STORY-077, STORY-079 (both depend on STORY-076 only; parallel)
Wave 3: STORY-078 (depends on STORY-077), STORY-080 (depends on STORY-079) -- parallel
```

Acyclicity verified: STORY-076 -> STORY-077/079 -> STORY-078/080. No cycles.

### E-6 Wave Schedule

```
Wave 1: STORY-066 (no deps)
```

Acyclicity verified: single story, trivially acyclic.

### E-7 Wave Schedule

```
Wave 1: STORY-069 (no deps -- defines Finding struct, Display impls)
Wave 2: STORY-070 (depends on STORY-069 -- adds serde skip_serializing_if)
Wave 3: STORY-071 (depends on STORY-069, STORY-070 -- MITRE mapping uses Finding types)
```

Acyclicity verified: STORY-069 -> STORY-070 -> STORY-071. No cycles.

### E-9 Wave Schedule

```
Wave 1: STORY-086 (no deps -- subcommand parsing, global flags)
Wave 2: STORY-087 (depends on STORY-086 -- output format + reassembly flags)
Wave 3: STORY-088 (depends on STORY-086, STORY-087 -- run_analyze orchestration)
Wave 4: STORY-089 (depends on STORY-086, STORY-087, STORY-088 -- decode errors, resolve_format, write_output)
Wave 5: STORY-090 (depends on STORY-086, STORY-088, STORY-089 -- Summary data model)
```

Acyclicity verified: STORY-086 -> 087 -> 088 -> 089 -> 090 (linear chain with no back-edges). No cycles.

Dependency anchor justifications:
- STORY-087 depends on STORY-086: `OutputFormat` enum and reassembly flag types are declared in `cli.rs` which STORY-086 establishes.
- STORY-088 depends on STORY-086+087: `run_analyze` uses `Commands::Analyze` (STORY-086) and `cli.reassemble`/`cli.no_reassemble` (STORY-087).
- STORY-089 depends on STORY-086+087+088: `resolve_format` and `write_output` consume `cli.json`/`cli.csv` (STORY-087) and integrate with the packet loop structure established in STORY-088.
- STORY-090 depends on STORY-086+088+089: `Summary::ingest` is called from the packet loop (STORY-088) and `skipped_packets` is assigned after it (STORY-089).

### E-10 Wave Schedule

```
Wave 1: STORY-096 (no deps -- verifies absence of removed flags)
```

Acyclicity verified: single story, trivially acyclic.

---

## Plan-Failure Scan Results

### New Stories (E-6, E-7, E-9, E-10)

- No "TBD" or "TODO" in any AC across STORY-066, STORY-069..071, STORY-086..090, STORY-096
- No "implement later" placeholders
- All six mandatory context-engineering sections present in every story:
  - Token Budget Estimate: present in all 10 new stories
  - Tasks: present in all 10 new stories
  - Previous Story Intelligence: present in all 10 new stories (first stories note N/A)
  - Architecture Compliance Rules: present in all 10 new stories
  - Library & Framework Requirements: present in all 10 new stories
  - File Structure Requirements: present in all 10 new stories
- No story exceeds 13 points (max: 8 points for STORY-071 and STORY-088)
- No story estimated context exceeds 20% of 200K token window (max: ~12% for STORY-088)
- All dependencies are forward-directed (acyclic) — verified per epic above
- Every BC traces to at least one AC in its story
- Every AC traces back to a specific BC postcondition, precondition, or invariant with clause citation
- STORY-096 correctly identifies as absent-behavior formalization (brownfield-formalization strategy)

**Status: PASS — no plan failures detected for new stories**
