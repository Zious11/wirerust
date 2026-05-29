---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - v1.3: DF-16.A citation fix — corrected broken capabilities.md §CAP-NN citation to per-cap file path — 2026-05-28
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.008: from_file Opens via BufReader and Delegates to from_pcap_reader

## Description

`PcapSource::from_file(path)` is a thin wrapper: it opens the file with
`std::fs::File::open(path)`, wraps it in a `BufReader`, and delegates to
`PcapSource::from_pcap_reader(reader)`. The public API that callers use is `from_file`;
`from_pcap_reader` is the testable inner layer. Integration tests use `from_file` exclusively.

## Preconditions

1. The path points to a readable file.

## Postconditions

1. Equivalent to calling `from_pcap_reader` on a `BufReader<File>` for the same path.
2. If the file cannot be opened, returns `Err` with anyhow context
   `"Failed to open {path}"` (reader.rs:87: `.with_context(|| format!("Failed to open {}", path.display()))`).

## Invariants

1. from_file is a delegation wrapper; all logic is in from_pcap_reader.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File does not exist | Err with context "Failed to open {path}" wrapping the OS "No such file or directory" error |
| EC-002 | File exists but not readable (permissions) | Err with context "Failed to open {path}" wrapping the OS permission error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| valid pcap file path | Same result as from_pcap_reader on same bytes | happy-path |
| non-existent path | Err with context "Failed to open {path}" | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | from_file behavior is identical to from_pcap_reader on same bytes | inferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- from_file is the public entry point for file-path based ingestion |
| L2 Domain Invariants | None |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-001 |
| Origin BC | BC-RDR-008 (pass-3 ingestion corpus, MEDIUM confidence -- inferred from code) |

## Architecture Anchors

- `src/reader.rs:85-90` -- `from_file`: opens `File`, wraps in `BufReader`, delegates to `from_pcap_reader`
- `src/reader.rs:86-87` -- `File::open(path).with_context(|| format!("Failed to open {}", path.display()))` -- file-open error context

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **inferred**: integration tests use from_file exclusively; delegation pattern is standard

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | opens and reads file |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell |
