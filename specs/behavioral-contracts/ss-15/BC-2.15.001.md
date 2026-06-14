---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/phase-f2-spec-evolution/dnp3-verification-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
  - .factory/specs/verification-properties/vp-023-dnp3-parse-safety.md
input-hash: TBD
---

# BC-2.15.001: DNP3 DL Header Accepted for Well-Formed 10-Byte-Minimum Frame

## Description

`parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` parses the DNP3 data-link header
from a reassembled TCP byte stream and returns a fully-populated `Dnp3DlHeader` struct when
the input is at least 10 bytes long (8 header bytes + 2 header-CRC bytes). All six struct
fields are decoded from fixed offsets: START1 at byte 0, START2 at byte 1, LENGTH at byte 2,
CONTROL at byte 3, DESTINATION little-endian at bytes 4–5, SOURCE little-endian at bytes 6–7
(bytes 8–9 are the header CRC and are not decoded as struct fields in v1). This is the
happy-path accept contract; validity gating (sync word, LENGTH range) is a separate concern
covered by BC-2.15.004.

## Preconditions

1. `data` is a `&[u8]` slice of reassembled, in-order TCP bytes delivered by the
   `StreamHandler::on_data` call site.
2. `data.len() >= 10` — the minimum complete DNP3 link-layer header is 10 bytes: 8 header
   octets (START1+START2+LENGTH+CONTROL+DEST(2)+SOURCE(2)) + 2 header-CRC octets. [SPEC:
   dnp3-research.md §1.1, §1.3]
3. No alignment or endianness assumptions are made about the host; DEST/SOURCE fields are
   decoded via `u16::from_le_bytes`. [SPEC: dnp3-research.md §1.1 "little-endian"]

## Postconditions

1. `parse_dnp3_dl_header(data)` returns `Some(Dnp3DlHeader { ... })`.
2. `Dnp3DlHeader.start1      = data[0]` — START1 byte, must be 0x05 for a valid sync.
3. `Dnp3DlHeader.start2      = data[1]` — START2 byte, must be 0x64 for a valid sync.
4. `Dnp3DlHeader.length      = data[2]` — LENGTH octet (range 5..=255 for valid frames).
5. `Dnp3DlHeader.control     = data[3]` — CONTROL octet (DIR/PRM/FCB/FCV/link-FC bitfields).
6. `Dnp3DlHeader.destination = u16::from_le_bytes([data[4], data[5]])` — little-endian
   link destination address. [SPEC: dnp3-research.md §1.1]
7. `Dnp3DlHeader.source      = u16::from_le_bytes([data[6], data[7]])` — little-endian
   link source address. [SPEC: dnp3-research.md §1.1]
8. Bytes 8–9 (header CRC) are structurally consumed but not decoded as struct fields (v1
   CRC-skip per ADR-007 Decision 3).
9. The function is pure: it reads only `data`; it never panics; it does not mutate any state.

## Invariants

1. **Little-endian addressing** (per IEEE 1815-2012 §1.1 and dnp3-research.md §1.1):
   DESTINATION and SOURCE fields use `u16::from_le_bytes` exclusively. Big-endian reads
   produce incorrect addresses.
2. **Purity**: `parse_dnp3_dl_header` is a pure core function — no I/O, no global state, no
   side effects. It is a Kani formal-verification target (VP-023 Sub-property A).
3. **Immutability of offsets**: the field offsets (0, 1, 2, 3, 4–5, 6–7, 8–9) are fixed by
   the IEEE 1815 data-link header layout and do not vary with PDU content. [SPEC]
4. **No validity gate inside this function**: `parse_dnp3_dl_header` does NOT check sync word
   or LENGTH range — it only parses. The three-point validity gate (`is_valid_dnp3_frame_header`)
   is the caller's responsibility. This separation keeps the function formally provable over
   all 2^(8*10) possible 10-byte inputs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Input is exactly 10 bytes (minimum frame header) | Returns `Some(Dnp3DlHeader)` — all six fields decoded correctly from the 10-byte slice |
| EC-002 | Input is 292 bytes (maximum DNP3 link frame) | Returns `Some(Dnp3DlHeader)` — parses the leading 10 bytes; trailing bytes are not consumed by this function |
| EC-003 | START1=0x05, START2=0x64 (valid sync word) | Fields decoded; validity gate (`is_valid_dnp3_frame_header`) separately confirms sync. This function does not gate. |
| EC-004 | START1=0x00, START2=0x00 (invalid sync) | Returns `Some(Dnp3DlHeader { start1: 0x00, start2: 0x00, ... })` — caller's validity gate rejects |
| EC-005 | DEST bytes = `[0xFF, 0xFF]` (broadcast 0xFFFF) | `destination = 0xFFFF` — little-endian `[0xFF, 0xFF]` = 65535 |
| EC-006 | DEST bytes = `[0xFD, 0xFF]` (broadcast 0xFFFD) | `destination = 0xFFFD` — little-endian `[0xFD, 0xFF]` = 65533 |
| EC-007 | LENGTH = 0x05 (minimum valid) | `length = 5` — validity gate passes (LENGTH >= 5) |
| EC-008 | LENGTH = 0x04 (below minimum) | `length = 4` — decoded as `Some`; validity gate rejects separately |
| EC-009 | All bytes = 0x00 | Returns `Some(Dnp3DlHeader { 0, 0, 0, 0, 0, 0 })` — validity gate rejects (sync mismatch) |

## Canonical Test Vectors

| Input (hex, 10 bytes minimum) | Expected `Dnp3DlHeader` fields | Category |
|-------------------------------|-------------------------------|----------|
| `05 64 05 C0 01 00 03 00 A1 B2` | start1=0x05, start2=0x64, length=5, control=0xC0, dest=0x0001, src=0x0003; CRC bytes 0xA1/0xB2 not decoded | happy-path: minimum-length control frame (5-byte link control, DIR=1, no user data) |
| `05 64 0E C4 03 00 01 00 88 C5 C0 81 00 00 ...` | start1=0x05, start2=0x64, length=14, control=0xC4, dest=0x0003, src=0x0001 | happy-path: DIRECT_OPERATE frame header parse (dest=outstation 3, src=master 1) |
| `05 64 05 44 FF FF 01 00 XX XX` | start1=0x05, start2=0x64, length=5, control=0x44, dest=0xFFFF, src=0x0001 | happy-path: broadcast destination (`[0xFF, 0xFF]` LE = 0xFFFF) |
| `00 00 05 C0 01 00 03 00 00 00` | start1=0x00, start2=0x00, length=5, control=0xC0, dest=0x0001, src=0x0003 | negative: invalid sync; returns `Some` but validity gate rejects |
| `05 64 04 C0 01 00 03 00 00 00` | start1=0x05, start2=0x64, length=4, control=0xC0, dest=0x0001, src=0x0003 | edge-case: LENGTH below minimum 5; returns `Some` but validity gate rejects |

**Annotated canonical vector breakdown** (DIRECT_OPERATE frame header):
```
Bytes:  05    64    0E   C4    03 00    01 00    88 C5
Field:  S1    S2    LEN  CTRL  DEST(LE) SRC(LE)  HDR_CRC
Value:  0x05  0x64  14   0xC4  0x0003   0x0001   (not decoded)
```
Note: DEST `[0x03, 0x00]` little-endian = 0x0003 (outstation address 3);
SRC `[0x01, 0x00]` little-endian = 0x0001 (master address 1).

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property A: `parse_dnp3_dl_header` returns `Some(_)` for `data.len() >= 10`; never panics for any input; all fields decoded from fixed offsets | Kani: symbolic `[u8; 12]` + symbolic `len <= 12` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — this BC defines the foundational parse-accept postconditions for the DNP3 data-link header decoder, which is the entry point for all DNP3 protocol analysis in the ICS/OT analyzer capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — DNP3 flows are only routed after TLS/HTTP content rules fail, ensuring this BC never fires on non-DNP3 flows) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24 Dnp3Analyzer); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.15.002 — composes with (reject path: `data.len() < 10` returns `None`)
- BC-2.15.003 — composes with (LE field decode correctness for DEST/SOURCE)
- BC-2.15.004 — depends on (validity gate uses parsed struct fields from this function)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn parse_dnp3_dl_header(data: &[u8]) -> Option<Dnp3DlHeader>` pure core function
- `src/analyzer/dnp3.rs` — `struct Dnp3DlHeader { start1, start2, length, control, destination, source }`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §3` — function signature and struct layout
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 2` — frame_len formula and header structure

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property A: parse safety, `Some` path)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-architecture-delta.md §3; dnp3-research.md §1.1; ADR-007 Decision 2 |
| **Confidence** | high — field offsets, endianness, and minimum 10-byte size are fixed by IEEE 1815-2012 and confirmed by dnp3-research.md §1 |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same Dnp3DlHeader |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-A) |
