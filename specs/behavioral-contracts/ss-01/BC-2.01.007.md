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

# BC-2.01.007: Surface Per-Packet Read Errors with Anyhow Context

## Description

When an individual packet record fails to read (e.g., truncated file, I/O error mid-stream),
the error is wrapped with context "Failed to read packet" and returned. Previously-read packets
are NOT returned as a partial result -- the entire from_pcap_reader call fails.

## Preconditions

1. The pcap global header was successfully parsed.
2. At least one packet record is partially present but cannot be fully read.

## Postconditions

1. Returns `Err` with context "Failed to read packet".
2. No partial packet vector is returned.

## Invariants

1. All-or-nothing semantics: partial reads return Err, not Ok with partial data.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File truncated in the middle of a packet record | Err with "Failed to read packet" |
| EC-002 | I/O error during read | Err wrapping the underlying OS error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcap with valid header, then truncated packet record | Err chain contains "Failed to read packet" | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Truncated mid-stream packet returns Err | unit: craft truncated pcap bytes |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- per-packet error handling is part of the ingestion loop |
| L2 Domain Invariants | None |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | S-TBD |
| Origin BC | BC-RDR-007 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test) |

## Architecture Anchors

- `src/reader.rs:70` -- `raw_packet.context("Failed to read packet")`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:70` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: anyhow::Context chain; no direct test

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell |
