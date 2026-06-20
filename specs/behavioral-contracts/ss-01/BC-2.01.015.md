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
  - "v1.1: F-07 completeness delta — explicitly enumerate all pcap-file Block variants that fall through to the skip path (NRB, ISB, DSB, SystemdJournalExport, obsolete Packet Block 0x2, Unknown); note that obsolete Packet Block 0x2 carries packet data but is treated as out-of-scope/skipped; add AC to prevent omitted match arm at implementation. — 2026-06-19"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.015: Unknown pcapng Block Types Are Silently Skipped via block-total-length

## Description

The pcapng specification allows any block type, with the guarantee that every block begins
with a 4-byte block-type field followed by a 4-byte block-total-length field. wirerust MUST
skip any block whose type is not SHB (`0x0A0D0D0A`), IDB (`0x00000001`), EPB (`0x00000006`),
or SPB (`0x00000003`) by reading and discarding exactly `block_total_length - 8` bytes
(the block body after the two fixed header fields). Neither a warning nor an error is emitted.
Parse state — interface table, byte order, packet list — is unchanged.

## Preconditions

1. The SHB has been parsed; byte order is established.
2. The current stream position is at the start of a block.
3. The block-type field is not one of the four known types.
4. `block_total_length` is parseable (at least 8 bytes remain in the stream).

## Postconditions

1. `block_total_length - 8` bytes are consumed from the stream (skipping the block body).
2. No error is returned.
3. No warning is emitted to stderr.
4. No packet is added to `PcapSource.packets`.
5. The interface table is unchanged.
6. Parsing continues with the next block.
7. If `block_total_length < 8` (impossible per spec but defensively checked), returns `Err`
   mapping to E-INP-010 (block-length inconsistency).
8. If the stream ends before `block_total_length - 8` bytes are consumed, returns `Err`
   mapping to E-INP-008 (truncated block).

## Acceptance Criteria

- **AC-001:** The implementation MUST have explicit match arms (or equivalent dispatch) for
  ALL of the following pcap-file `Block` enum variants that are NOT SHB/IDB/EPB/SPB. Every
  variant listed below MUST be handled by the skip path; none may be an omitted (implicit
  default-panic) arm:
  - `NameResolutionBlock` (NRB, type `0x00000004`) — name-to-IP mappings; no packet data; silently skipped.
  - `InterfaceStatisticsBlock` (ISB, type `0x00000005`) — interface capture statistics; no packet data; silently skipped.
  - `DecryptionSecretsBlock` (DSB, type `0x0000000A`) — TLS key log material; no packet data; silently skipped.
  - `SystemdJournalExportBlock` (type `0x00000009`) — journal entries; no packet data; silently skipped.
  - Obsolete Packet Block (OPB, type `0x00000002`) — carries captured packet data **but** is
    an obsolete/deprecated block type superseded by EPB; wirerust treats it as out-of-scope and
    skips it silently. **Implementation note:** OPB packet data is intentionally NOT ingested.
    Captures relying solely on OPB (very old tcpdump versions) will yield zero packets from
    those blocks.
  - All other unknown / future block types — silently skipped via `block_total_length`.
- **AC-002:** For each variant above, the skip MUST NOT emit any warning, error, or finding.
  Parse state (interface table, byte order, packet list) MUST be unchanged after the skip.
- **AC-003:** Skipping an OPB does NOT cause EPB interpretation to be affected; `PcapSource.packets`
  reflects only EPB- and SPB-sourced packets.

## Invariants

1. The skip is performed solely using the `block_total_length` field; no knowledge of the
   block's internal structure is required.
2. The following pcap-file Block variants are ALL covered by this skip contract (see AC-001
   for the complete list with type codes): NameResolutionBlock (NRB), InterfaceStatisticsBlock
   (ISB), DecryptionSecretsBlock (DSB), SystemdJournalExportBlock, obsolete Packet Block
   (OPB, type `0x00000002`), and all unknown/future block types. No match arm for these
   variants may be left omitted in the implementation.
3. The skip MUST NOT emit any diagnostic to stderr or stdout.
4. All four known block types (SHB, IDB, EPB, SPB) MUST be handled by their own parsing
   branches; they MUST NOT fall through to the skip path.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Interface Statistics Block (ISB, type `0x00000005`) | Silently skipped; no warning; no packet added |
| EC-002 | Obsolete Packet Block (OPB, type `0x00000002`) | Silently skipped; packet data in OPB NOT ingested; EPB parsing unaffected |
| EC-003 | Block with future type value (e.g. `0x00000007`) | Silently skipped |
| EC-004 | `block_total_length = 8` (block with empty body) | 0 bytes skipped; no error |
| EC-005 | `block_total_length = 4` (illegal; less than minimum 8) | `Err` (E-INP-010) |
| EC-006 | Stream truncated mid-skip (stream ends before skip completes) | `Err` (E-INP-008) |
| EC-007 | Multiple consecutive unknown blocks | Each skipped independently; no cumulative error |
| EC-008 | Name Resolution Block (NRB, type `0x00000004`) | Silently skipped; name resolution data NOT ingested; no warning |
| EC-009 | Decryption Secrets Block (DSB, type `0x0000000A`) | Silently skipped; TLS key material NOT used; no warning |
| EC-010 | Systemd Journal Export Block (type `0x00000009`) | Silently skipped; journal data NOT ingested; no warning |
| EC-011 | pcapng file containing OPB blocks before and after EPBs | OPBs silently skipped; only EPBs produce packets; packet list is EPB-derived only |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| pcapng file with ISB before final EPB | ISB skipped silently; final EPB produces `RawPacket` | happy-path |
| Block with type `0xDEADBEEF`, `block_total_length=20` | 12 bytes consumed, no error, no packet | edge-case |
| Block with `block_total_length=6` | `Err` (E-INP-010) | error |
| Unknown block followed by EPB | Unknown block skipped; EPB parsed normally | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | ISB does not produce a packet or an error | unit: pcapng file with ISB; assert no error, packets unchanged |
| — | No stderr output on unknown block | unit: capture stderr during parse; assert empty |
| — | Truncated unknown block returns Err, not Ok | unit: craft truncated unknown block; assert Err |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md |
| Capability Anchor Justification | CAP-01 ("PCAP File Ingestion") per domain/capabilities/cap-01-pcap-ingestion.md -- block-type skip is a robustness property of the ingestion pipeline; the ability to traverse unknown blocks is required to successfully read all packets from a well-formed pcapng file that contains optional blocks |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-01 (reader.rs, C-4) |
| Stories | STORY-126 |
| ADR Reference | ADR-009 Decision 2 ("Unknown block types MUST be silently skipped using the block-total-length field; neither a warning nor an error is emitted") |

## Related BCs

- BC-2.01.012 -- related (EPB is a known block; must NOT fall to skip path)
- BC-2.01.013 -- related (SPB is a known block; must NOT fall to skip path)
- BC-2.01.017 -- related (block-level errors map to E-INP-008/010; skip path never errors for well-formed unknown blocks)

## Architecture Anchors

- pcapng spec IETF draft §General-Block-Structure: every block has 4-byte type + 4-byte total-length at start
- ADR-009 Decision 2: "Unknown block types MUST be silently skipped using the block-total-length field"
- `pcap_file::pcapng` block walker (docs.rs/pcap-file/2.0.0): unknown block variant in `Block` enum

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | reads and discards bytes from stream |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | effectful shell (I/O only) |
