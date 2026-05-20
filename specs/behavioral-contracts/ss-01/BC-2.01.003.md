---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reader.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-01
capability: CAP-01
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.003: Accept PCAP with Zero Packets Without Error

## Description

A pcap file containing only the global header and no packet records is valid input. The reader
returns `Ok(PcapSource { packets: vec![], datalink })` without error or panic. This is important
for test fixtures and for handling captures that were stopped before any traffic was recorded.

## Preconditions

1. The pcap file has a valid global header with an accepted link type.
2. The file ends immediately after the global header (zero packet records).

## Postconditions

1. Returns `Ok(PcapSource)` with `packets` equal to an empty `Vec<RawPacket>`.
2. No error is returned.
3. No panic occurs.

## Invariants

1. Empty packet vector is a valid PcapSource state.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File is exactly 24 bytes (pcap global header only) | Ok(PcapSource { packets: [] }) |
| EC-002 | Zero-packet capture with LINUX_SLL link type | Ok -- link type is still validated |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcap with valid header and 0 packets | Ok(PcapSource { packets: [] }) | happy-path |
| minimum valid pcap bytes (24-byte header) | Ok(PcapSource { packets: [] }) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | packets.is_empty() == true for zero-packet pcap | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- this BC covers the degenerate zero-packet case of file ingestion |
| L2 Domain Invariants | None |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | S-TBD |
| Origin BC | BC-RDR-003 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.01.002 -- related to (zero-packet is a special case of the general read loop)

## Architecture Anchors

- `src/reader.rs:40` -- while-let yields immediately on empty packet stream

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:40` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **type constraint**: while-let on PcapReader iterator exits immediately when no records

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell |
