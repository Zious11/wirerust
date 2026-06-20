---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified:
  - "v1.1: F2 Burst-A remediation per ADR-009 rev 4 PO dispatch — (1) VP-027 added to Verification Properties. (2) Postcondition 5 corrected: EPB with interface_id referencing an EMPTY table → E-INP-009 (not E-INP-008); EPB with interface_id OOB on a NON-EMPTY table → E-INP-010 (not E-INP-008). (3) Added explicit AC: interface_id MUST be bounds-checked before any indexing. (4) Added guard-before-allocate AC (SEC-004): captured_len vs. block_total_length - 32 check MUST precede any data allocation. (5) Corrected and named EPB body-relative fixed overhead = 20 bytes (EPB_FIXED_OVERHEAD_BYTES); outer 12-byte raw header is separate; validation: captured_len <= block_total_length - 32. (6) Added no-panic AC (SEC-005). (7) Added boundary edge cases (captured_len = btl-32 valid; btl-31 invalid). (8) Clarified raw-block path: timestamp is raw split ticks fed to BC-2.01.014 — NOT the crate's Duration. (9) Updated EC-005 to reflect empty-table vs OOB distinction. — 2026-06-19"
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
pcapng. On the raw-block path (ADR-009 Decision 1, rev 4), wirerust reads EPB fixed fields
directly from `RawBlock` body bytes: `interface_id: u32`, `ts_high: u32`, `ts_low: u32`,
`captured_len: u32`, `original_len: u32` (20 bytes of body-relative fixed overhead,
`EPB_FIXED_OVERHEAD_BYTES = 20`; outer 12-byte block header is separate). The raw split ticks
`(ts_high, ts_low)` are passed to the BC-2.01.014 pure-core helper together with the
per-interface `if_tsresol` to produce `(ts_sec: u32, ts_usecs: u32)` for `RawPacket`. The
`captured_len` field bounds the data slice and MUST be validated against
`block_total_length - 32` (12 outer + 20 body-fixed) before any allocation.

## Preconditions

1. At least one IDB has been parsed; the interface table is non-empty (BC-2.01.011).
2. The block type reads `0x00000006` (after byte-order correction from SHB).
3. The RawBlock body contains at least `EPB_FIXED_OVERHEAD_BYTES = 20` bytes.
4. `block_total_length` is the value reported by the crate's block framing layer (crate
   rejects `block_total_length < 12` before handing any block to the caller).

## Postconditions

1. The raw 64-bit timestamp in ticks is formed as
   `ticks: u64 = (ts_high as u64) << 32 | (ts_low as u64)` from the raw block body fields.
   This is the RAW split-ticks value, NOT the crate's `Duration` (which hard-codes nanoseconds
   and NEVER applies `if_tsresol` — confirmed at `enhanced_packet.rs:46-48,65`).
2. `ticks` is converted to `(ts_sec, ts_usecs)` by calling the BC-2.01.014 pure-core helper
   with `(ts_high, ts_low, if_tsresol)` where `if_tsresol` comes from the IDB for
   `interface_id` (defaulting to `6u8` when absent from the IDB).
3. Packet data is copied from the EPB body bounded by `captured_len` bytes (not
   `original_len`). If `captured_len < original_len`, the packet is snaplen-truncated; the
   `data` field carries only the captured bytes.
4. The resulting `RawPacket` is appended to the `PcapSource.packets` vector in EPB encounter
   order.
5. An EPB whose `interface_id` is evaluated against an EMPTY interface table (no IDB has been
   seen in the current section) returns `Err` mapping to E-INP-009. An EPB whose `interface_id`
   is out of range on a NON-EMPTY interface table returns `Err` mapping to E-INP-010 with
   context string `"EPB interface_id={id} out of range (table size={n})"`.
6. An EPB where `captured_len > block_total_length - 32` returns `Err` mapping to E-INP-010
   (packet-data truncation / block-length inconsistency).
7. No EPB is silently dropped on parse error — the error propagates immediately.

## Acceptance Criteria

- **AC-001 (interface_id bounds-check before indexing):** The `interface_id` field MUST be
  checked against the current interface table size before any indexing operation. The check
  MUST distinguish empty-table (→ E-INP-009) from out-of-range-on-non-empty-table (→ E-INP-010).
  An unchecked array index on `interface_id` is NOT permitted.
- **AC-002 (guard-before-allocate, SEC-004):** The validation `captured_len <=
  block_total_length - 32` MUST be performed BEFORE any memory allocation for packet data.
  Allocating based on an attacker-controlled `captured_len` without first checking the block
  boundary is prohibited.
- **AC-003 (no-panic, SEC-005):** This block parser MUST return `Err` for any malformed or
  truncated input; `unwrap()`, `expect()`, and `panic!()` are prohibited in the EPB parse path.
  The crate's own field-level guards (`slice.len() < 20` → `Err`) enforce this at the framing
  layer; wirerust MUST NOT bypass them.
- **AC-004 (raw-block path):** The raw split ticks `(ts_high, ts_low)` MUST be read from the
  `RawBlock` body and passed to the BC-2.01.014 helper. wirerust MUST NOT consume
  `EnhancedPacketBlock::timestamp` (the crate's `Duration` type) — that field hard-codes
  nanosecond resolution and discards the raw ticks, making tsresol-correct conversion
  impossible.

## Invariants

1. Packet order in `PcapSource.packets` matches EPB encounter order in the block stream.
2. `captured_len` MUST be used for data slicing, never `original_len`. Using
   `original_len` would read past the actual bytes in the block.
3. The `RawPacket` struct produced by EPB parsing is structurally identical to the struct
   produced by classic-pcap parsing; no new fields are added.
4. An EPB's `interface_id` must resolve to an already-seen IDB; forward references (EPB before
   any IDB) produce E-INP-009 — a pcapng structural violation.
5. `EPB_FIXED_OVERHEAD_BYTES = 20` (body-relative: interface_id:4 + ts_high:4 + ts_low:4 +
   captured_len:4 + original_len:4). The outer 12-byte block header
   (block_type:4 + block_total_length:4 + trailing_total_length:4) is NOT included in this
   constant. The combined minimum block size is therefore 32 bytes (12 + 20).
6. The `captured_len` field is NOT retained on the parsed type (`data.len()` recovers it).
   `original_len` IS retained on the RawPacket.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `captured_len == original_len` (no truncation) | Data copied in full; normal case |
| EC-002 | `captured_len < original_len` (snaplen-truncated) | Data bounded to `captured_len`; truncated `RawPacket` produced; downstream decoder handles via lax fallback |
| EC-003 | `ts_high = 0, ts_low = 0` | `timestamp_secs=0, timestamp_usecs=0`; valid zero-epoch packet |
| EC-004 | `ts_high` and `ts_low` combine to a very large u64 (near u64::MAX) | BC-2.01.014 saturating arithmetic handles; `ts_sec` saturates at u32::MAX; no panic |
| EC-005 | EPB `interface_id = 0` with EMPTY interface table (no IDB seen yet) | `Err` mapping to E-INP-009 (empty-table path) |
| EC-006 | EPB `interface_id = 1` with only one IDB (index 0) in non-empty table | `Err` mapping to E-INP-010 (OOB on non-empty table); context: `"EPB interface_id=1 out of range (table size=1)"` |
| EC-007 | EPB `interface_id = u32::MAX` with any non-empty table | `Err` mapping to E-INP-010 (OOB on non-empty table) |
| EC-008 | `captured_len = 0` (zero-length captured data) | `RawPacket { data: vec![] }`; zero-byte packet is valid |
| EC-009 | `captured_len = block_total_length - 32` (maximum valid) | Exactly valid; data slice occupies the entire block body minus fixed fields; `Ok(RawPacket)` |
| EC-010 | `captured_len = block_total_length - 31` (one byte over maximum) | `Err` mapping to E-INP-010 (captured_len exceeds block boundary) |
| EC-011 | EPB body shorter than 20 bytes (truncated fixed fields) | `Err` mapping to E-INP-010 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| EPB with `ts_high=0, ts_low=1000000`, `if_tsresol` absent (default µs) | `RawPacket { timestamp_secs: 1, timestamp_usecs: 0 }` | happy-path |
| EPB with `ts_high=0, ts_low=1500000000`, `if_tsresol=0x09` (nanoseconds) | `RawPacket { timestamp_secs: 1, timestamp_usecs: 500000 }` | happy-path |
| EPB with `captured_len=64, original_len=1500` | `RawPacket { data.len() == 64 }` (snaplen-truncated) | edge-case |
| EPB with `interface_id=0`, empty interface table (no IDB) | `Err` mapping to E-INP-009 | error |
| EPB with `interface_id=5`, one IDB (index 0 only) | `Err` mapping to E-INP-010; context includes `"interface_id=5 out of range (table size=1)"` | error |
| EPB with `captured_len = block_total_length - 32` | `Ok(RawPacket)` with `data.len() == captured_len` | boundary-valid |
| EPB with `captured_len = block_total_length - 31` | `Err` mapping to E-INP-010 | boundary-invalid |
| EPB body shorter than 20 bytes | `Err` mapping to E-INP-010 | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-027 | EPB parse safety: no panic; interface_id bounds-check before table index (empty-table → E-INP-009, OOB-non-empty → E-INP-010); captured_len guard precedes allocation; returns Err for all invalid inputs | Kani: `#[kani::proof]` over EPB byte sequences with symbolic interface_id and captured_len |
| — | `captured_len` is always used for data slice, never `original_len` | unit: EPB with captured < original; assert data.len() == captured |
| — | Packet order preserved across multiple EPBs | unit: 3-EPB file; assert order matches |
| — | Raw split ticks routed to BC-2.01.014 (not crate Duration) | unit: EPB with `if_tsresol=6` known-µs ticks; assert timestamp 1000× correct (regression guard for crate's ns-hardcode bug) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- EPB parsing is the primary packet-extraction path for pcapng; the `Vec<RawPacket>` produced by EPB parsing is the output artifact of CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-125 |
| ADR Reference | ADR-009 rev 4 Decision 1 (raw-block path), Decision 2 (EPB coverage), Decision 4 (64-bit timestamp normalization via pure-core helper), Decision 8 (forward-progress), Decision 10 (panic surface) |

## Related BCs

- BC-2.01.011 -- depends on (interface table populated by IDB parsing; EPB uses interface_id to look up if_tsresol)
- BC-2.01.014 -- composes with (raw split ticks passed to timestamp conversion helper)
- BC-2.01.002 -- mirrors (classic-pcap analog; same RawPacket output type)

## Architecture Anchors

- ADR-009 rev 4 Decision 1: raw-block path (`RawBlock` / `next_raw_block`); EPB fixed fields read from raw body: interface_id:4, ts_high:4, ts_low:4, captured_len:4, original_len:4
- ADR-009 rev 4 Decision 4: `EPB_FIXED_OVERHEAD_BYTES = 20` (body-relative); validation `captured_len <= block_total_length - 32`
- `enhanced_packet.rs:46-48,65` (pcap-file 2.0.0 source): `Duration::from_nanos` hard-codes ns, never applies `if_tsresol` — confirms wirerust MUST NOT use `EnhancedPacketBlock::timestamp`
- pcapng spec IETF draft §Enhanced-Packet-Block: fixed-fields layout; captured vs. original length semantics

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream (raw-block path) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading); timestamp sub-computation is pure-core (BC-2.01.014) |
