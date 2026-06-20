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

# BC-2.01.012: Parse pcapng Enhanced Packet Block (EPB): Packet Data and Timestamp

## Description

The Enhanced Packet Block (EPB, block type `0x00000006`) is the primary packet container in
pcapng. Each EPB carries a 64-bit timestamp split as `(ts_high: u32, ts_low: u32)`, an
interface ID linking it to an IDB (BC-2.01.011), captured-length, original-length, and raw
packet data up to captured-length bytes. The reader must combine `ts_high` and `ts_low` with
the per-interface `if_tsresol` value (BC-2.01.014) to produce `(timestamp_secs: u32,
timestamp_usecs: u32)` for `RawPacket`. The `captured_length` — not `original_length` — is
used to bound the data slice.

## Preconditions

1. At least one IDB has been parsed; the interface table is populated (BC-2.01.011).
2. The block type reads `0x00000006` (after byte-order correction from SHB).
3. The EPB's `interface_id` field references a valid 0-based index in the current section's
   interface table.
4. `block_total_length` is consistent with the sum of fixed fields + packet data + padding.

## Postconditions

1. The 64-bit timestamp is formed as `timestamp_u64 = (ts_high as u64) << 32 | (ts_low as u64)`.
2. `timestamp_u64` is converted to `(ts_sec, ts_usecs)` using the per-interface `if_tsresol`
   as defined in BC-2.01.014.
3. Packet data is copied from the EPB body bounded by `captured_length` (not `original_length`).
   If `captured_length < original_length`, the packet is snaplen-truncated; the `data` field
   carries only the captured bytes.
4. The resulting `RawPacket` is appended to the `PcapSource.packets` vector in EPB encounter
   order.
5. An EPB with `interface_id` referencing an interface index not in the table returns `Err`
   mapping to E-INP-008.
6. An EPB with `captured_length` exceeding `block_total_length` minus the EPB fixed-field
   overhead returns `Err` mapping to E-INP-010.
7. No EPB is silently dropped on parse error — the error propagates immediately.

## Invariants

1. Packet order in `PcapSource.packets` matches EPB encounter order in the block stream.
2. `captured_length` MUST be used for data slicing, never `original_length`. Using
   `original_length` would read past the actual bytes in the block.
3. The `RawPacket` struct produced by EPB parsing is structurally identical to the struct
   produced by classic-pcap parsing; no new fields are added.
4. An EPB's `interface_id` must resolve to an already-seen IDB; forward references (EPB before
   any IDB) are an error per pcapng spec and wirerust enforces this.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `captured_length == original_length` (no truncation) | Data copied in full; normal case |
| EC-002 | `captured_length < original_length` (snaplen-truncated) | Data bounded to `captured_length`; truncated `RawPacket` produced; downstream decoder handles via lax fallback |
| EC-003 | `ts_high = 0, ts_low = 0` | timestamp_secs=0, timestamp_usecs=0; valid zero-epoch packet |
| EC-004 | `ts_high` and `ts_low` combine to a very large u64 (near u64::MAX) | Conversion may saturate `ts_sec` at u32::MAX; documented Y2106 limitation (same as classic pcap) |
| EC-005 | EPB `interface_id = 1` with only one IDB (index 0) | `Err` -- interface 1 not in table |
| EC-006 | `captured_length = 0` (zero-length captured data) | `RawPacket { data: vec![] }`; zero-byte packet is valid |
| EC-007 | EPB body truncated mid-packet-data | `Err` mapping to E-INP-010 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| EPB with `ts_high=0, ts_low=1000000`, `if_tsresol` absent (default µs) | `RawPacket { timestamp_secs: 1, timestamp_usecs: 0 }` | happy-path |
| EPB with `ts_high=0, ts_low=1500000000`, `if_tsresol=0x09` (nanoseconds) | `RawPacket { timestamp_secs: 1, timestamp_usecs: 500000 }` | happy-path |
| EPB with `captured_length=64, original_length=1500` | `RawPacket { data.len() == 64 }` (snaplen-truncated) | edge-case |
| EPB with `interface_id=5`, only IDB at index 0 | `Err` | error |
| EPB body shorter than fixed fields | `Err` (E-INP-010) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | `captured_length` is always used for data slice, never `original_length` | unit: EPB with captured < original; assert data.len() == captured |
| — | Packet order preserved across multiple EPBs | unit: 3-EPB file; assert order matches |
| — | Truncated EPB never panics | fuzz: fuzz EPB bytes, assert no panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- EPB parsing is the primary packet-extraction path for pcapng; the `Vec<RawPacket>` produced by EPB parsing is the output artifact of CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-125 |
| ADR Reference | ADR-009 Decision 2 (EPB coverage), Decision 4 (64-bit timestamp normalization) |

## Related BCs

- BC-2.01.011 -- depends on (interface table populated by IDB parsing; EPB uses interface_id to look up if_tsresol)
- BC-2.01.014 -- composes with (timestamp conversion is delegated to the pure-core helper)
- BC-2.01.002 -- mirrors (classic-pcap analog; same RawPacket output type)

## Architecture Anchors

- `pcap_file::pcapng::blocks::enhanced_packet::EnhancedPacketBlock` (docs.rs/pcap-file/2.0.0) -- EPB struct with `interface_id`, `ts_high`, `ts_low`, `captured_packet_length`, `original_packet_length`, `packet_data`
- pcapng spec IETF draft §Enhanced-Packet-Block: fixed-fields layout; captured vs. original length semantics
- ADR-009 Decision 4: "timestamp_u64 = (ts_high << 32) | ts_low; converted using if_tsresol"

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading) |
