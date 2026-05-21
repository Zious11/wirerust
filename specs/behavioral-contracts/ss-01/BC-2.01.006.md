---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified:
  - v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.006: Surface PCAP Header Parse Errors with Anyhow Context

## Description

When the pcap_file crate fails to parse the global header, the error is wrapped with anyhow
context string "Failed to parse pcap header" and returned. Callers can check the error message
chain for this string. This covers corrupted files, zero-byte files, and pcapng files (which
have a different magic number).

## Preconditions

1. PcapSource::from_pcap_reader is called with a reader positioned at the start of a file.
2. The file bytes do not constitute a valid classic pcap global header.

## Postconditions

1. Returns `Err(anyhow error)` where the error chain contains "Failed to parse pcap header".
2. No packets are read or returned.
3. No panic occurs.

## Invariants

1. The context string "Failed to parse pcap header" is stable; test tooling may key on it.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero-byte file | Err with "Failed to parse pcap header" |
| EC-002 | Truncated pcap header (< 24 bytes) | Err with "Failed to parse pcap header" |
| EC-003 | pcapng magic bytes | Err with "Failed to parse pcap header" |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| empty file (0 bytes) | Err chain contains "Failed to parse pcap header" | error |
| 10 random bytes | Err chain contains "Failed to parse pcap header" | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Error chain contains "Failed to parse pcap header" for corrupt header | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- this BC covers the error path for corrupted or wrong-format files |
| L2 Domain Invariants | None |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-001 |
| Origin BC | BC-RDR-006 (pass-3 ingestion corpus, MEDIUM confidence -- no direct test) |

## Related BCs

- BC-2.01.001 -- related to (link-type gating runs after successful header parse)

## Architecture Anchors

- `src/reader.rs:46` -- `PcapReader::new(reader).context("Failed to parse pcap header")`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:46` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: anyhow::Context chain is wired; no direct test

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell |
