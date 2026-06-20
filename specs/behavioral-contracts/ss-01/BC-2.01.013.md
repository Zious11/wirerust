---
document_type: behavioral-contract
level: L3
version: "1.2"
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
  - "v1.1: F2 Burst-A remediation per ADR-009 rev 4 PO dispatch — (1) Corrected SPB body-relative fixed overhead to 4 bytes (original_len: u32 only; H-2 fix — was incorrectly stated as 20 bytes in the Description and Postcondition 1). (2) Corrected minimum block_total_length to 16 bytes (12 outer + 4 body-fixed); available padded-data bytes = block_total_length - 16. (3) Added explicit note: RawBlock `data` includes padding — caller MUST compute captured_len = min(original_len, snaplen) and strip accordingly. (4) Added SPB-without-IDB case as E-INP-009 (empty interface table; do NOT index idb[0] unguarded — H-4). (5) Added no-panic AC (SEC-005). (6) Removed incorrect 'block_total_length - 20' formula from Postcondition 1 (20 was the EPB overhead, not SPB). — 2026-06-19"
  - "v1.2: Pass-2 remediation per ADR-009 rev 5 (I-4, I-11) — (I-4) EC-001 corrected: data bound changed from min(original_len, block_body_available) to min(original_len, snaplen, block_body_available) — consistent with PC1/Invariant-2 which both specify the three-way minimum. (I-11) Added Test: citations to all ACs. Added HS-107 holdout reference in Verification Properties. — 2026-06-19"
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
carries raw packet data and an `original_len: u32` field but no per-packet timestamp, no
interface ID, and no options. `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len`
field only; the outer 12-byte block header is separate). On the raw-block path, the crate
exposes `data: Cow<[u8]>` which is the ENTIRE remaining block body after the 4-byte
`original_len` field — this slice INCLUDES padding bytes; the crate performs NO
`captured_len` computation and NO snaplen clamping. The caller MUST compute
`captured_len = min(original_len, snaplen_from_idb[0])` and strip the padding accordingly.
SPBs are rare in practice (Wireshark does not emit them) but are legal per the pcapng
specification. Timestamp fields on `RawPacket` are always set to zero for SPBs.

## Preconditions

1. The SHB has been parsed; byte order is established.
2. The block type reads `0x00000003`.
3. The interface table is checked before accessing `idb[0]`; if the table is empty, the
   call returns `Err` mapping to E-INP-009 (SPB-without-IDB).
4. `block_total_length` is at least 16 bytes (12 outer block header + 4 body-fixed for
   `original_len`; minimum legal SPB with 0 bytes of padded data).

## Postconditions

1. The raw `data` slice from `RawBlock` is the block body after `original_len` (4 bytes),
   padded to a 4-byte boundary. The available padded-data bytes =
   `block_total_length - 16` (12-byte outer header + 4-byte `original_len` field).
   `captured_len = min(original_len, snaplen_from_idb[0])`. The data slice MUST be
   truncated to `captured_len` bytes (stripping padding).
2. `original_len` is noted but NOT used to extend the data slice beyond the
   padded block body (a malformed file could claim `original_len` larger than available
   block data; the padded block body is the authoritative bound).
3. A `RawPacket` is produced with `timestamp_secs = 0` and `timestamp_usecs = 0`.
4. The `RawPacket` is appended to `PcapSource.packets` in block-encounter order.
5. An SPB encountered when the interface table is EMPTY (no IDB has been seen) returns `Err`
   mapping to E-INP-009. The caller MUST guard the `idb[0]` access; an unchecked index on an
   empty table is NOT permitted (H-4 fix).
6. A truncated SPB (block shorter than 16 bytes, i.e., insufficient bytes for the
   `original_len` field) returns `Err` mapping to E-INP-010.

## Acceptance Criteria

- **AC-001 (snaplen from idb[0] — guarded):** wirerust MUST look up `snaplen` from the
  interface table at index 0. This access MUST be guarded: if the interface table is empty,
  return `Err` mapping to E-INP-009 rather than indexing an empty Vec.
  **Test:** `test_BC_2_01_013_snaplen_lookup_guarded`
- **AC-002 (padding strip):** The raw `data` slice from the crate INCLUDES padding bytes to
  the 4-byte boundary. wirerust MUST compute `captured_len = min(original_len, snaplen)` and
  slice to `captured_len` bytes before populating `RawPacket.data`. Handing the padded slice
  to downstream decoders verbatim is prohibited.
  **Test:** `test_BC_2_01_013_padding_strip`
- **AC-003 (no-panic, SEC-005):** This block parser MUST return `Err` for any malformed or
  truncated input; `unwrap()`, `expect()`, and `panic!()` are prohibited in the SPB parse path.
  **Test:** `test_BC_2_01_013_no_panic_malformed`
- **AC-004 (SPB_FIXED_OVERHEAD_BYTES = 4):** The named constant `SPB_FIXED_OVERHEAD_BYTES`
  MUST equal 4 (body-relative; `original_len: u32` only). This constant MUST NOT be confused
  with `EPB_FIXED_OVERHEAD_BYTES = 20`.
  **Test:** `test_BC_2_01_013_fixed_overhead_constant`

## Invariants

1. SPB timestamps are always zero — there is no per-packet timestamp in the SPB format.
   Downstream consumers (reassembly, findings timestamp) receive zero-timestamps for SPBs.
2. Packet data is bounded by `min(original_len, snaplen)` and further by the available padded
   block body (`block_total_length - 16`); no out-of-bounds read is possible.
3. SPB parsing shares the same `RawPacket` output type as EPB and classic-pcap parsing.
4. `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len: u32` only). The minimum
   SPB `block_total_length` is 16 bytes (12 outer + 4 body-fixed).
5. The pcapng specification requires that a file using SPBs must have exactly one IDB;
   wirerust enforces this by using `snaplen` from interface index 0, with an explicit guard
   for the empty-table case.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | SPB with `original_len > block body data` (truncated on disk) | Data slice bounded to `min(original_len, snaplen, block_body_available)` bytes; `RawPacket.data.len() < original_len` |
| EC-002 | SPB where `original_len` exactly matches `snaplen` | Data sliced to `captured_len = original_len = snaplen`; no truncation |
| EC-003 | SPB in file with multiple IDBs (spec violation) | Guard only checks `idb.is_empty()`; if non-empty, uses `idb[0].snaplen`; no panic; proceeds |
| EC-004 | SPB with zero-byte data section (`original_len = 0`) | `RawPacket { data: vec![] }` produced |
| EC-005 | SPB body shorter than 4 bytes (truncated `original_len` field) | `Err` mapping to E-INP-010 |
| EC-006 | SPB encountered before any IDB (empty interface table) | `Err` mapping to E-INP-009 (guard fires before any idb[0] access) |
| EC-007 | `original_len` > `snaplen`; block body contains full `original_len` data + padding | `captured_len = snaplen`; data sliced to `snaplen` bytes (snaplen wins) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SPB with 64 bytes of Ethernet frame data, `original_len=64`, `snaplen=65535` | `RawPacket { timestamp_secs: 0, timestamp_usecs: 0, data.len(): 64 }` | happy-path |
| SPB with `original_len=1500`, block body 64 padded bytes, `snaplen=65535` | `data.len() == 64` (bounded by block body) | edge-case |
| SPB with `original_len=1500`, `snaplen=100`, block body 1500 + padding | `data.len() == 100` (snaplen wins) | edge-case |
| SPB before any IDB (empty interface table) | `Err` (E-INP-009) | error |
| Truncated SPB (`block_total_length = 14`, < 16 minimum) | `Err` (E-INP-010) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | SPB always produces timestamp (0, 0) | unit: parse SPB; assert timestamp_secs=0, timestamp_usecs=0 |
| — | SPB data length bounded by min(original_len, snaplen, block_body) | unit: SPB with original_len > block body; assert data.len() <= block body available bytes |
| — | SPB-without-IDB returns E-INP-009, not panic | unit: SPB with empty interface table; assert Err(E-INP-009); no panic |
| — | SPB padding stripped before RawPacket | unit: SPB with original_len not 4-byte aligned; assert data.len() == original_len (not padded length) |
| — | Covered under VP-028 (cargo-fuzz) for full no-panic coverage | fuzz: fuzz SPB bytes, assert no panic (F6 hardening deliverable) |
| HS-107 | SPB holdout scenario: real-world pcapng file with SPB blocks validates end-to-end ingestion correctness and no false positives/negatives on SPB-carrying captures | holdout evaluation (Phase 4); see `.factory/specs/holdout-scenarios/HS-107.md` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- SPB parsing is an alternative packet-extraction path within pcapng ingestion; its `RawPacket` output is the same artifact as EPB and classic-pcap parsing under CAP-01 |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 rev 4 Decision 2 (SPB block coverage), Decision 9 (snaplen enforcement is caller-side), Decision 10 (panic surface) |

## Related BCs

- BC-2.01.011 -- depends on (SPB uses snaplen from interface 0 IDB; empty-table guard prevents H-4 panic)
- BC-2.01.012 -- sibling (EPB is the timestamp-bearing alternative to SPB; same RawPacket output)
- BC-2.01.015 -- related to (unknown blocks are skipped; SPB is a known block that must be parsed)

## Architecture Anchors

- ADR-009 rev 4 Decision 2: `SPB_FIXED_OVERHEAD_BYTES = 4` (body-relative: `original_len: u32` only); minimum `block_total_length = 16`; caller derives `captured_len = min(original_len, snaplen)` and strips padding
- `simple_packet.rs:19-37` (pcap-file 2.0.0 source): `data: Cow<[u8]>` includes padding; no `captured_len` field; no snaplen clamp in crate
- pcapng spec IETF draft §Simple-Packet-Block: `original_len` field only; on-disk payload is padded to 4-byte boundary

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads block bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O during block reading) |
