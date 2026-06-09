---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.001: MBAP Header Accepted for Well-Formed 8-Byte-Minimum ADU

## Description

`parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` parses the 7-byte Modbus Application
Protocol (MBAP) header from a reassembled TCP byte stream and returns a fully-populated
`MbapHeader` struct when the input is at least 8 bytes long (7-byte MBAP header + 1-byte
minimum Function Code). All five fields are decoded big-endian from fixed offsets: Transaction
ID at bytes 0–1, Protocol ID at bytes 2–3, Length at bytes 4–5, Unit ID at byte 6, Function
Code at byte 7. This is the happy-path accept contract; validity gating (Protocol ID,
Length range) is a separate concern covered by BC-2.14.002 through BC-2.14.004.

## Preconditions

1. `data` is a `&[u8]` slice of reassembled, in-order TCP bytes delivered by the
   `StreamHandler::on_data` call site.
2. `data.len() >= 8` — the minimum valid Modbus TCP ADU is 8 bytes (7-byte MBAP header plus
   1-byte Function Code).
3. No alignment or endianness assumptions are made about the host; all fields are decoded
   via `u16::from_be_bytes` / direct index.

## Postconditions

1. `parse_mbap_header(data)` returns `Some(MbapHeader { ... })`.
2. `MbapHeader.transaction_id = u16::from_be_bytes([data[0], data[1]])` — big-endian, offset 0.
3. `MbapHeader.protocol_id    = u16::from_be_bytes([data[2], data[3]])` — big-endian, offset 2.
4. `MbapHeader.length         = u16::from_be_bytes([data[4], data[5]])` — big-endian, offset 4.
5. `MbapHeader.unit_id        = data[6]` — single byte, offset 6.
6. `MbapHeader.function_code  = data[7]` — single byte, offset 7.
7. The function is pure: it reads only `data`; it never panics; it does not mutate any state.
8. Trailing bytes at `data[8..]` are not consumed by `parse_mbap_header` — the caller's ADU
   boundary loop uses `MbapHeader.length` to advance the offset pointer.

## Invariants

1. **Big-endian encoding** (per Modbus.org spec V1.1b3 §4.2 and Messaging Guide V1.0b §3.1.3):
   all multi-byte fields are network byte order. `u16::from_be_bytes` is the exclusive decoding
   mechanism; native-endian reads are incorrect.
2. **Purity**: `parse_mbap_header` is a pure core function — no I/O, no global state, no side
   effects. It is a Kani formal-verification target (VP-022 sub-property A).
3. **Immutability of offsets**: the five field offsets (0–1, 2–3, 4–5, 6, 7) are fixed by the
   Modbus TCP specification and do not vary with PDU content.
4. **No validity gate inside this function**: `parse_mbap_header` does NOT check `protocol_id`
   or `length` — it only parses. The 3-point validity gate (`is_valid_modbus_adu`) is the
   caller's responsibility. This separation keeps the function formally provable over all
   2^(8*8) possible 8-byte inputs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Input is exactly 8 bytes (minimum ADU) | Returns `Some(MbapHeader)` — all five fields decoded correctly from the 8-byte slice |
| EC-002 | Input is 260 bytes (maximum ADU) | Returns `Some(MbapHeader)` — parses the leading 8 bytes; trailing 252 bytes are not consumed by this function |
| EC-003 | Transaction ID = 0x0000 (connection start) | `transaction_id = 0` — valid; servers echo it unchanged |
| EC-004 | Transaction ID = 0xFFFF (maximum) | `transaction_id = 65535` — valid; BE decoding `[0xFF, 0xFF]` = 65535 |
| EC-005 | Protocol ID = 0x0000 (valid Modbus) | `protocol_id = 0` — caller will pass `is_valid_modbus_adu`; this function does not gate |
| EC-006 | Protocol ID = 0x0001 (non-Modbus) | `protocol_id = 1` — returned as `Some`; caller's `is_valid_modbus_adu` rejects it |
| EC-007 | Length = 2 (minimum) | `length = 2` — decoded; validity gate is caller responsibility |
| EC-008 | Length = 253 (maximum) | `length = 253` — decoded; validity gate is caller responsibility |
| EC-009 | Unit ID = 0xFF (broadcast address) | `unit_id = 255` — valid; broadcast address per Modbus spec |
| EC-010 | Function Code = 0x00 (undefined) | `function_code = 0` — returned; `classify_fc` (BC-2.14.005) will classify as Unknown |
| EC-011 | All bytes = 0x00 | Returns `Some(MbapHeader { 0, 0, 0, 0, 0 })` — valid parse; validity gate rejects `length=0` separately |

## Canonical Test Vectors

| Input (hex, 8+ bytes) | Expected `MbapHeader` fields | Category |
|----------------------|------------------------------|----------|
| `00 01 00 00 00 06 01 03 00 00 00 0A` | txn=0x0001, proto=0x0000, len=6, unit=0x01, fc=0x03 | happy-path: Read Holding Registers request (6-byte PDU: addr=0x0000, qty=0x000A) |
| `00 2A 00 00 00 06 FF 06 00 14 01 F4` | txn=0x002A, proto=0x0000, len=6, unit=0xFF, fc=0x06 | happy-path: Write Single Register to broadcast unit, reg=0x0014, val=0x01F4 |
| `FF FF 00 00 00 02 01 83` | txn=0xFFFF, proto=0x0000, len=2, unit=0x01, fc=0x83 | happy-path: Exception response for FC 0x03 (high bit set) |
| `00 05 00 00 00 06 02 10 00 00 00 02 04 00 64 00 C8` | txn=0x0005, proto=0x0000, len=6, unit=0x02, fc=0x10 | happy-path: Write Multiple Registers request header parsed; len field parsed correctly |
| `00 01 00 00 00 06 01 03 00 00 00 0A` (260-byte padded) | txn=0x0001, proto=0x0000, len=6, unit=0x01, fc=0x03 | edge-case: extra trailing bytes do not affect the 8-byte parse |

**Annotated canonical vector breakdown** (Read Holding Registers request):
```
Bytes:  00 01  |  00 00  |  00 06  |  01  |  03  | 00 00 00 0A
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  |  data (addr=0x0000, qty=0x000A)
Value:  0x0001 |  0x0000 |    6    |   1  | 0x03 |  request payload
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property A: `parse_mbap_header` returns `None` for `data.len() < 8`; returns `Some(_)` for `data.len() >= 8`; never panics over all inputs | Kani: symbolic `&[u8]` of all lengths 0..16 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the foundational parse-accept postconditions for the MBAP header decoder, which is the entry point for all Modbus protocol analysis in the ICS/OT analyzer capability |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation — parsed fields are raw; no formatting at parse time) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22); ADR-005 §2.4 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.002 — composes with (reject path: `data.len() < 8` returns `None`)
- BC-2.14.003 — composes with (post-parse Protocol ID validity gate)
- BC-2.14.004 — composes with (post-parse Length validity gate)
- BC-2.14.005 — depends on (function_code field from this parse is input to `classify_fc`)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `parse_mbap_header(data: &[u8]) -> Option<MbapHeader>` pure core function
- `src/analyzer/modbus.rs` — `MbapHeader` struct (fields: transaction_id: u16, protocol_id: u16, length: u16, unit_id: u8, function_code: u8)
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.4` — MBAP parse model and wire layout

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — Modbus MBAP Parse Safety and Function-Code Boundary Classification (sub-property A)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.4; modbus-tcp-research.md §1 |
| **Confidence** | high — field offsets and sizes are fixed by Modbus.org spec V1.1b3 and Messaging Guide V1.0b §3.1.3 |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same MbapHeader |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-022 Kani target |
