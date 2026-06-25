---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.002: EnipHeader Field Contracts — Fixed Little-Endian Offsets for 24-Byte Input

## Description

`parse_enip_header(data: &[u8]) -> Option<EnipHeader>` returns `Some(EnipHeader{...})` when
`data.len() >= 24`. All 6 struct fields are decoded from fixed, non-overlapping byte offsets
using little-endian byte order (per ODVA EtherNet/IP specification) except for
`sender_context`, which is an opaque 8-byte copy. The field layout is: command (2 LE, bytes
0–1), length (2 LE, bytes 2–3), session_handle (4 LE, bytes 4–7), status (4 LE, bytes 8–11),
sender_context (8 opaque, bytes 12–19), options (4 LE, bytes 20–23). This is the accept-path
contract; the reject path for short inputs is BC-2.17.001.

## Preconditions

1. `data` is a `&[u8]` slice of reassembled TCP bytes from `StreamHandler::on_data`.
2. `data.len() >= 24` — at least one complete ENIP encapsulation header is present.
3. No alignment assumptions are required: all fields use explicit `u16::from_le_bytes` /
   `u32::from_le_bytes` / array copy — host endianness is irrelevant.

## Postconditions

1. `parse_enip_header(data)` returns `Some(EnipHeader{...})`.
2. `EnipHeader.command        = u16::from_le_bytes([data[0], data[1]])` — ENIP command code.
3. `EnipHeader.length         = u16::from_le_bytes([data[2], data[3]])` — payload byte count
   after the 24-byte header (max 65,511 due to TCP framing; the u16 field allows 65,535).
4. `EnipHeader.session_handle = u32::from_le_bytes([data[4], data[5], data[6], data[7]])` —
   session handle (0 for commands that do not require a registered session).
5. `EnipHeader.status         = u32::from_le_bytes([data[8], data[9], data[10], data[11]])` —
   encapsulation status (0x00000000 = success; non-zero = error response).
6. `EnipHeader.sender_context = [data[12], data[13], data[14], data[15], data[16], data[17],
   data[18], data[19]]` — 8-byte opaque context (copied verbatim; not decoded).
7. `EnipHeader.options        = u32::from_le_bytes([data[20], data[21], data[22], data[23]])` —
   options field (must be 0x00000000 in standard implementations).
8. Bytes beyond index 23 are not read by this function (the frame-walk loop handles them).
9. The function is pure: no I/O, no state mutation, no panics for any input.

## Invariants

1. **Little-endian ENIP header**: all multi-byte fields in the ENIP encapsulation header are
   little-endian per ODVA EtherNet/IP specification. CPF items (parsed separately) also use
   little-endian — both layers are LE. [SPEC: ADR-010 Decision 2]
2. **Fixed offsets**: byte offsets 0–23 are normative ODVA. They do not vary with command
   type, session state, or payload content. The parse is unconditional over all 24 bytes.
3. **sender_context is opaque**: the 8-byte sender_context field is copied verbatim as
   `[u8; 8]`. It is not decoded as a number or compared against any known pattern.
4. **Purity**: `parse_enip_header` is a pure-core Kani target (VP-032 Sub-A). It reads
   exactly bytes 0–23 and returns an `EnipHeader` struct. No heap allocation beyond the struct.
5. **Trailing bytes ignored**: data beyond byte 23 is the CPF payload; it is the caller's
   responsibility to index into `data[24 .. 24 + header.length]` for CPF parsing.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data.len() == 24` (exact minimum) | Returns `Some(EnipHeader)` — all 6 fields decoded from bytes 0–23; no out-of-bounds |
| EC-002 | `data.len() == 600` (max carry buffer) | Returns `Some(EnipHeader)` from bytes 0–23; remaining 576 bytes untouched |
| EC-003 | `command = 0x006F` (SendRRData), `length = 0x0020` | `header.command=0x006F`, `header.length=32` — valid explicit messaging frame |
| EC-004 | `status = 0x00000065` (non-zero — error response) | `header.status=0x00000065` decoded; validity gate in BC-2.17.003 may still accept (status is not gated here) |
| EC-005 | `session_handle = 0x00000000` (ListIdentity — no session) | `header.session_handle=0` — valid; ListIdentity does not require a registered session |
| EC-006 | All 24 bytes = `0xFF` | Returns `Some(EnipHeader{command:0xFFFF, length:0xFFFF, ...})` — all fields decoded; validity gate rejects (command 0xFFFF is unknown) |
| EC-007 | `options` field bytes 20–23 are all `0x00` | `header.options=0x00000000` — standard compliant |
| EC-008 | `sender_context` bytes 12–19 contain mixed values | `[u8;8]` copied verbatim; no interpretation |

## Canonical Test Vectors

| Input bytes [0..23] (hex) | Expected `EnipHeader` fields | Category |
|---------------------------|------------------------------|---------|
| `65 00 04 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00` | command=0x0065, length=4, session=0, status=0, context=[0×8], options=0 | ListIdentity response header |
| `6F 00 20 00 04 03 02 01 00 00 00 00 AA BB CC DD EE FF 00 11 00 00 00 00` | command=0x006F, length=32, session=0x01020304, status=0, context=[AA BB CC DD EE FF 00 11], options=0 | SendRRData with session (LE: bytes 4–7 = 04 03 02 01 → 0x01020304) |
| `64 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00 00` | command=0x0064, length=0, session=0, status=0, context=[0×8], options=0 | ListInterfaces command (zero payload) |
| `FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF FF` | command=0xFFFF, length=0xFFFF, session=0xFFFFFFFF, status=0xFFFFFFFF, context=[FF×8], options=0xFFFFFFFF | all-0xFF: Some returned; validity gate rejects (unknown command) |

**Annotated SendRRData vector breakdown (little-endian):**
```
Bytes:  6F 00  20 00  04 03 02 01  00 00 00 00  AA BB CC DD EE FF 00 11  00 00 00 00
Field:  CMD(LE) LEN(LE) SESSION_HANDLE(LE)  STATUS(LE)  SENDER_CONTEXT(8 bytes)  OPTIONS(LE)
Value:  0x006F  32      0x01020304           0            [AA..11]                 0
Note:   bytes[4..8] = [04,03,02,01] → u32::from_le_bytes = 0x01020304
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-032 | Sub-A: `parse_enip_header` returns `Some` for `len >= 24`; `h.command == u16::from_le_bytes([data[0], data[1]])`; `h.length == u16::from_le_bytes([data[2], data[3]])`; `h.status == u32::from_le_bytes([data[8..12]])`; no out-of-bounds for any symbolic 48-byte input | Kani: symbolic `[u8; 48]`, `len ∈ [24, 48]`; asserts all field equalities |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — this BC specifies the accept-path field contract for the ENIP encapsulation header decoder; correct little-endian field extraction is required for all subsequent CIP command classification, CPF parsing, and MITRE detection |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — ENIP flows are only routed after TLS/HTTP content rules fail) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.17.001 — composes with (reject path: `len < 24` returns `None`)
- BC-2.17.003 — depends on (validity gate operates on `EnipHeader.command` from this function)
- BC-2.17.004 — depends on (command classification uses `header.command`)
- BC-2.17.005 — depends on (CPF parse uses `header.length` to bound payload slice)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn parse_enip_header(data: &[u8]) -> Option<EnipHeader>` — pure-core free function
- `src/analyzer/enip.rs` — `struct EnipHeader { command: u16, length: u16, session_handle: u32, status: u32, sender_context: [u8; 8], options: u32 }`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 2` — field-offset specification and pseudocode
- `.factory/specs/verification-properties/vp-032-enip-parse-safety.md §Sub-A` — Kani proof skeleton (field assertions)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-032 Sub-A — ENIP header parse safety (Some path: field-offset assertions)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 2 (parse_enip_header pseudocode and field offsets); ODVA EtherNet/IP Specification Table 2-4 (encapsulation header layout); VP-032 Sub-A skeleton |
| **Confidence** | high — field offsets and little-endian byte order are normative ODVA EtherNet/IP specification; confirmed by Wireshark packet-enip.c dissector |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same EnipHeader |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-032 Sub-A Kani target |
