---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - v1.3: Phase 3 per-story adversarial review — corrected Invariant 2: smb3.pcapng IS now used as an active negative-coverage assertion by test_BC_2_01_004_rejects_pcapng delivered in STORY-001 — 2026-05-21
  - v1.4: DF-16.A citation fix — corrected broken capabilities.md §CAP-NN citation to per-cap file path — 2026-05-28
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.004: Reject pcapng-Format Input at Reader Level

## Description

wirerust reads only classic pcap format. A pcapng file (which uses a Section Header Block
magic number, not the classic pcap magic `0xD4C3B2A1` or `0xA1B2C3D4`) causes the pcap_file
crate's header parser to fail. The error is wrapped with anyhow context "Failed to parse pcap
header" and returned without any packets being processed. This is expected and documented
behavior, not a bug.

## Preconditions

1. A file path is provided to PcapSource::from_file or from_pcap_reader.
2. The file begins with pcapng Section Header Block magic bytes (not classic pcap magic).

## Postconditions

1. Returns `Err` with anyhow context containing "Failed to parse pcap header".
2. No packets are read or returned.
3. No panic occurs.

## Invariants

1. pcapng support is explicitly out of scope (README documents this; *.pcapng excluded from
   directory glob by LESSON-P0.02 / #69).
2. The `tests/fixtures/smb3.pcapng` fixture IS used as an active negative-coverage assertion:
   `test_BC_2_01_004_rejects_pcapng` (delivered in STORY-001) passes it to `from_file` and
   asserts the Err result contains "Failed to parse pcap header".

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | File is a valid pcapng (smb3.pcapng) | Err("Failed to parse pcap header") |
| EC-002 | Directory glob for *.pcap in a dir containing .pcapng files | .pcapng files are excluded from the glob; they are never passed to from_file |
| EC-003 | pcapng file passed explicitly via CLI | Returns Err at from_file level |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| tests/fixtures/smb3.pcapng | Err containing "Failed to parse pcap header" | error |
| directory containing only .pcapng files with --analyze dir/ | Zero files resolved (glob excludes .pcapng) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | pcapng file returns Err not Ok | unit: from_file(smb3.pcapng) returns Err |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- rejection of pcapng is a boundary of the ingestion capability (format scope is CAP-01's concern; link-type gating after a successful header parse is CAP-02) |
| L2 Domain Invariants | None |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-001 |
| Origin BC | BC-RDR-004 (pass-3 ingestion corpus; confidence upgraded to HIGH -- test_BC_2_01_004_rejects_pcapng delivered in STORY-001) |

## Related BCs

- BC-2.01.001 -- related to (link-type gating is also a rejection path; pcapng fails before reaching it)
- BC-2.12.011 -- related to (directory glob excludes *.pcapng)

## Architecture Anchors

- `src/reader.rs:46` -- `PcapReader::new(reader).context("Failed to parse pcap header")` -- pcapng rejected here
- `src/main.rs` -- directory glob pattern excludes *.pcapng (LESSON-P0.02 / #69)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:46` |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **documentation**: README:126 declares "pcapng not yet supported"
- **inferred**: pcap_file::PcapReader rejects pcapng magic bytes by design

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell |
