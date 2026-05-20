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

# BC-2.01.005: Convert PCAP Record Timestamp to (timestamp_secs: u32, timestamp_usecs: u32)

## Description

Each pcap record carries a timestamp as a `Duration`. The reader splits this into two `u32`
fields: `timestamp_secs` (Duration::as_secs() cast as u32) and `timestamp_usecs`
(Duration::subsec_micros()). The as-u32 cast is lossy for timestamps beyond 2106 but this is
an accepted limitation of the pcap format's own u32 epoch design.

## Preconditions

1. A pcap packet record has been read from the file.
2. The record timestamp is a valid Duration from the pcap_file crate.

## Postconditions

1. `RawPacket.timestamp_secs` equals `pcap_packet.timestamp.as_secs() as u32`.
2. `RawPacket.timestamp_usecs` equals `pcap_packet.timestamp.subsec_micros()`.
3. No panic from the as-u32 cast (Rust u64-as-u32 is defined truncation, not panic).

## Invariants

1. Timestamps are preserved as-read; no normalization or wall-clock correction is applied.
2. The Y2106 wrap (when u64 seconds exceed u32::MAX = 4,294,967,295) is not detected or
   reported. This is a known accepted limitation.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | timestamp_secs = 1000, timestamp_usecs = 500 | RawPacket { timestamp_secs: 1000, timestamp_usecs: 500 } |
| EC-002 | Timestamp at u32::MAX (year 2106) | as u32 wraps to 0; no error |
| EC-003 | Timestamp with sub-millisecond precision | subsec_micros preserves microseconds; nanoseconds truncated |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcap_packet.timestamp = 1000s 500us | timestamp_secs=1000, timestamp_usecs=500 | happy-path |
| timestamp = 0s 0us | timestamp_secs=0, timestamp_usecs=0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | timestamp_secs matches pcap record seconds field | unit: craft pcap bytes with known timestamp |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- timestamp extraction is part of RawPacket production during ingestion |
| L2 Domain Invariants | None (O-01: timestamps are read but never threaded to Finding constructors) |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | S-TBD |
| Origin BC | BC-RDR-005 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.01.002 -- composes with (timestamp split is part of the per-packet read loop)
- BC-2.09.001 -- related to (Finding.timestamp is always None regardless; O-01)

## Architecture Anchors

- `src/reader.rs:43-44` -- timestamp split: `.as_secs() as u32` and `.subsec_micros()`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:43-44` |
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
