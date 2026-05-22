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
  - v1.3: Phase 3 per-story adversarial review — corrected Architecture Anchor line range: timestamp conversion block ends at :73, not :74 (line 74 is blank in reader.rs) — 2026-05-21
  - v1.4: Phase 3 per-story adversarial review pass 5 — corrected false claim in v1.3 changelog: line 74 is NOT blank; it is the closing `};` of the match expression. Architecture Anchor corrected to 71-74 (full match span: let-binding at :71, two arms at :72-73, closing brace at :74) — 2026-05-21
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.005: Convert PCAP Record Timestamp to (timestamp_secs: u32, timestamp_usecs: u32)

## Description

Each pcap record exposes a seconds field (`ts_sec`) and a fractional field (`ts_frac`). The
reader copies `ts_sec` directly into `timestamp_secs: u32` and derives `timestamp_usecs: u32`
from `ts_frac`: used as-is for microsecond-resolution files, divided by 1_000 for nanosecond-
resolution files (reader.rs:71-73). No `Duration` API is involved. The u32 field is already
the pcap format's own type; no cast is required beyond the match-arm value.

## Preconditions

1. A pcap packet record has been read from the file.
2. The record's `ts_sec` (u32) and `ts_frac` (u32) fields are populated by the pcap_file crate from the pcap record header.

## Postconditions

1. `RawPacket.timestamp_secs` equals `raw_packet.ts_sec` -- the pcap record's seconds field
   copied directly (reader.rs:76).
2. `RawPacket.timestamp_usecs` equals `raw_packet.ts_frac` when `ts_resolution` is
   `TsResolution::MicroSecond`, or `raw_packet.ts_frac / 1_000` when
   `TsResolution::NanoSecond` (reader.rs:71-73).
3. Both fields are `u32` as declared in the pcap format; no additional cast is performed.

## Invariants

1. Timestamps are preserved as-read; no normalization or wall-clock correction is applied.
2. `ts_sec` is a u32 in the pcap format; values beyond 2106 (u32::MAX = 4,294,967,295)
   are not possible without format corruption. The field is copied as-is; no wrapping
   behavior is applied by wirerust.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ts_sec=1000, ts_frac=500 (microsecond file) | RawPacket { timestamp_secs: 1000, timestamp_usecs: 500 } |
| EC-002 | ts_sec=u32::MAX (maximum possible pcap value) | timestamp_secs=4294967295; no error (u32 is the native pcap type) |
| EC-003 | Nanosecond-resolution file (ts_frac=500_000) | timestamp_usecs = 500_000 / 1_000 = 500; sub-microsecond precision is discarded (reader.rs:73) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcap record: ts_sec=1000, ts_frac=500, MicroSecond resolution | RawPacket { timestamp_secs: 1000, timestamp_usecs: 500 } | happy-path |
| pcap record: ts_sec=0, ts_frac=0, MicroSecond resolution | RawPacket { timestamp_secs: 0, timestamp_usecs: 0 } | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | timestamp_secs matches pcap record seconds field | unit: craft pcap bytes with known timestamp |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- timestamp extraction is part of RawPacket production during ingestion |
| L2 Domain Invariants | None (O-01: timestamps are read but never threaded to Finding constructors) |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-001 |
| Origin BC | BC-RDR-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.01.002 -- composes with (timestamp split is part of the per-packet read loop)
- BC-2.09.001 -- related to (Finding.timestamp is always None regardless; O-01)

## Architecture Anchors

- `src/reader.rs:71-74` -- timestamp conversion: `let timestamp_usecs = match ts_resolution {` at :71, two match arms at :72-73, closing `};` at :74
- `src/reader.rs:76-77` -- `timestamp_secs: raw_packet.ts_sec`, `timestamp_usecs` from ts_frac conversion

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:71-77` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: test_read_pcap_packets asserts timestamp_secs == 1000

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell |
