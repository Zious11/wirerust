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

# BC-2.01.002: Read All Packets from PCAP as Vec<RawPacket> Preserving Timestamps

## Description

After link-type acceptance, the reader performs an eager in-memory load of all pcap packets
into a `Vec<RawPacket>`. Each `RawPacket` carries the raw frame bytes and a split timestamp
(seconds and microseconds) copied from the pcap record header. The entire file must fit in RAM
because no streaming or lazy-read mode exists. This contract covers `PcapSource::from_pcap_reader`
in `src/reader.rs:69-79`.

## Preconditions

1. The pcap file has passed link-type gating (BC-2.01.001 postconditions hold).
2. A `pcap_file::PcapReader` is positioned at the start of packet records.
3. Sufficient RAM is available to load all packets (no streaming; entire file loaded at once).

## Postconditions

1. Returns `Ok(PcapSource { packets: Vec<RawPacket>, datalink })` where `packets` contains
   one entry per pcap record in file order.
2. Each `RawPacket` carries:
   - `timestamp_secs: u32` -- truncated from pcap timestamp Duration seconds.
   - `timestamp_usecs: u32` -- from `Duration::subsec_micros()`.
   - `data: Vec<u8>` -- raw frame bytes, cloned via `into_owned()`.
3. Packet order matches pcap record order (no sorting or reordering).
4. On any packet read error, returns `Err` with context "Failed to read packet"; previously
   read packets are NOT returned.

## Invariants

1. Eager load: all packets are in memory before `PcapSource` is returned.
2. No deduplication, filtering, or modification of packet bytes.
3. `timestamp_secs` is a `u32`, making it correct until the year 2106 (Y2106 boundary); this
   is an accepted limitation of the pcap u32 timestamp format.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Zero-packet pcap (header only) | Returns Ok(PcapSource { packets: vec![] }) |
| EC-002 | Single-packet pcap | Returns Ok(PcapSource { packets: vec![packet] }) |
| EC-003 | Pcap with snaplen-truncated packets | Packets are stored with truncated data bytes; decoder handles truncation downstream |
| EC-004 | Mid-stream read error (corrupt packet record) | Returns Err wrapping the read error; no partial result |
| EC-005 | Very large capture (multi-GB) | Loaded entirely into RAM; may fail with OOM if RAM is insufficient (NFR-VIO-001) |
| EC-006 | Packet with timestamp_secs at u32::MAX (post-2106) | Stored as-is; lossy cast is not checked |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| single-packet pcap with timestamp_secs=1000, timestamp_usecs=500 | RawPacket { timestamp_secs: 1000, timestamp_usecs: 500, data: <frame bytes> } | happy-path |
| empty pcap (header only, 0 packets) | Ok(PcapSource { packets: [], datalink: ETHERNET }) | edge-case |
| three-packet pcap | packets.len() == 3, order preserved | happy-path |
| truncated packet record bytes | Err containing "Failed to read packet" | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Packet count matches the number of pcap records | unit: craft pcap bytes, assert packets.len() |
| VP-TBD | Timestamps are preserved without modification | unit: assert timestamp_secs and timestamp_usecs from known pcap |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 |
| Capability Anchor Justification | CAP-01 ("PCAP file ingestion") per capabilities.md §CAP-01 -- this BC is the core reading contract that produces the packet vector for downstream processing |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | S-TBD -- filled by story-writer |
| Origin BC | BC-RDR-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.01.001 -- depends on (link-type must be accepted before packet reading)
- BC-2.01.003 -- related to (zero-packet case is a special instance)
- BC-2.01.005 -- composes with (timestamp extraction is part of this read loop)

## Architecture Anchors

- `src/reader.rs:69-79` -- PcapSource::from_pcap_reader packet loop (while-let at :69, context at :70)
- `src/reader.rs:71-74` -- timestamp extraction: ts_frac for MicroSecond / NanoSecond resolution
- `src/reader.rs:75-79` -- RawPacket construction: timestamp_secs, timestamp_usecs, data

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reader.rs:69-79` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: while-let loop exits on None (end of packets)
- **assertion**: test_read_pcap_packets asserts timestamp_secs == 1000

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads file (BufReader) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (file I/O only) |
