---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-19T00:00:00Z
phase: F2
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-01
capability: CAP-01
lifecycle_status: active
introduced: v0.10.0-pcapng
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.013: Parse pcapng Simple Packet Block (SPB): Packet Data Without Timestamp

## Description

The Simple Packet Block (SPB, block type `0x00000003`) is a compact packet container that
carries raw packet data and an original-packet-length field but no per-packet timestamp, no
interface ID, and no options. SPBs are rare in practice (Wireshark does not emit them) but
are legal per the pcapng specification. wirerust MUST parse SPBs to completion: extract the
packet data bounded by the lesser of `block_total_length - 20` (SPB overhead) and the IDB
`snaplen` for interface 0 (the only interface legal when SPBs are present). Timestamp fields
on `RawPacket` are set to zero for SPBs.

## Preconditions

1. The SHB and at least one IDB have been parsed.
2. The block type reads `0x00000003`.
3. The pcapng specification requires that a file using SPBs must have exactly one IDB;
   wirerust enforces this by using the `snaplen` from interface index 0.
4. `block_total_length` is at least 20 bytes (minimum SPB size: block-type 4 + total-len 4
   + original-packet-length 4 + trailing-total-len 4 + at least some data bytes or 0-pad).

## Postconditions

1. The packet data is extracted up to `min(block_total_length - 20, idb[0].snaplen)` bytes.
2. `original_packet_length` is noted but NOT used to extend the data slice beyond
   `captured_length` (same snaplen discipline as EPB).
3. A `RawPacket` is produced with `timestamp_secs = 0` and `timestamp_usecs = 0`.
4. The `RawPacket` is appended to `PcapSource.packets` in block-encounter order.
5. An SPB in a file with multiple IDBs (which is illegal per spec) does not panic; it uses
   interface index 0 and proceeds.
6. A truncated SPB (block shorter than minimum) returns `Err` mapping to E-INP-010.

## Invariants

1. SPB timestamps are always zero — there is no per-packet timestamp in the SPB format.
   Downstream consumers (reassembly, findings timestamp) receive zero-timestamps for SPBs.
2. Packet data is bounded by the lesser of SPB body size and interface-0 snaplen; no
   out-of-bounds read is possible.
3. SPB parsing shares the same `RawPacket` output type as EPB and classic-pcap parsing.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SPB with `original_packet_length > block body data` (truncated) | Data slice bounded to block body; `RawPacket.data.len() < original_packet_length` |
| EC-002 | SPB where block body exactly matches snaplen | Data copied in full; no truncation |
| EC-003 | SPB in file with multiple IDBs (spec violation) | Uses interface 0 snaplen; no panic; proceeds |
| EC-004 | SPB with zero-byte data section | `RawPacket { data: vec![] }` produced |
| EC-005 | SPB body shorter than 4 bytes (truncated original-length field) | `Err` mapping to E-INP-010 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SPB with 64 bytes of Ethernet frame data | `RawPacket { timestamp_secs: 0, timestamp_usecs: 0, data.len(): 64 }` | happy-path |
| SPB with `original_packet_length=1500`, block data 64 bytes | `data.len() == 64` (truncated to SPB body) | edge-case |
| Truncated SPB (8 bytes total) | `Err` (E-INP-010) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SPB always produces timestamp (0, 0) | unit: parse SPB; assert timestamp_secs=0, timestamp_usecs=0 |
| — | SPB data length bounded by block body | unit: SPB with original_len > block body; assert data.len() <= block body |
| — | Truncated SPB never panics | fuzz: fuzz SPB bytes, assert no panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SPB parsing is an alternative packet-extraction path within pcapng ingestion; its `RawPacket` output is the same artifact as EPB and classic-pcap parsing under CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 Decision 2 (SPB block coverage) |

## Related BCs

- BC-2.01.011 -- depends on (SPB uses snaplen from interface 0 IDB)
- BC-2.01.012 -- sibling (EPB is the timestamp-bearing alternative to SPB; same RawPacket output)
- BC-2.01.015 -- related to (unknown blocks are skipped; SPB is a known block that must be parsed)

## Architecture Anchors

- `pcap_file::pcapng::blocks::simple_packet::SimplePacketBlock` (docs.rs/pcap-file/2.0.0) -- SPB struct
- pcapng spec IETF draft §Simple-Packet-Block: overhead = 20 bytes (block-type 4 + total-len 4 + orig-pkt-len 4 + data + padding + trailing-total-len 4)
- ADR-009 Decision 2: "SPB (packet data, no timestamp)"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading) |
