---
document_type: behavioral-contract
level: L3
version: "1.0"
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

# BC-2.17.005: CPF Item-Layer Walk — Bounded Little-Endian Item Iteration

## Description

`parse_cpf_items(payload: &[u8]) -> Vec<CpfItem>` parses the Common Packet Format item list
from the payload of `SendRRData` (0x006F) or `SendUnitData` (0x0070) frames. CPF uses
little-endian byte order (unlike the ENIP encapsulation header which uses big-endian). The
function reads a 2-byte LE item_count, then iterates over item_count items, each preceded by
a 4-byte LE item header (type_id 2 LE + item_length 2 LE). Iteration terminates early on any
bounds violation — the item count declared in the CPF header cannot exceed what the actual
payload bytes support. The function never panics and never reads out of bounds.

## Preconditions

1. `payload` is the byte slice `data[24 .. 24 + header.length]` — the CPF payload after the
   ENIP encapsulation header.
2. `payload.len() >= 0` — an empty payload (header.length == 0) is valid and yields an empty
   item list.
3. This function is called only for `SendRRData` (0x006F) and `SendUnitData` (0x0070)
   commands where a CPF payload is expected.

## Postconditions

1. If `payload.len() < 2`, returns `vec![]` (empty — cannot read item_count).
2. `item_count = u16::from_le_bytes([payload[0], payload[1]])` — LE read; not BE.
3. For each item in `0..item_count`:
   - If `cursor + 4 > payload.len()`: iteration breaks; remaining declared items are skipped.
   - `item.type_id = u16::from_le_bytes([payload[cursor], payload[cursor+1]])` (LE).
   - `item.length  = u16::from_le_bytes([payload[cursor+2], payload[cursor+3]])` (LE).
   - `cursor += 4`.
   - If `cursor + item.length > payload.len()`: iteration breaks; item data is unavailable.
   - `item.data = payload[cursor .. cursor + item.length]`.
   - `cursor += item.length`.
   - Item pushed to result vec.
4. Returns the items successfully parsed before any bounds violation.
5. No panic for any `payload` content or length.
6. The function is pure: no I/O, no state mutation.

## Invariants

1. **CPF is little-endian**: item_count, type_id, and item_length are all read with
   `u16::from_le_bytes`. Big-endian reads would produce incorrect item structures.
   [SPEC: ODVA EtherNet/IP Specification §2-6.1; ADR-010 Decision 2]
2. **Early-break on bounds violation**: the iteration terminates without panic whenever a
   bounds check fails. Partial results (items parsed before the violation) are returned.
   An attacker declaring a large item_count with insufficient payload bytes cannot cause
   more iterations than the payload length supports (minimum 4 bytes per item header).
3. **DoS bound via payload-length cap**: the maximum number of items parseable from a
   `MAX_ENIP_CARRY_BYTES = 600` payload is bounded by `(600 - 2) / 4 = 149` (all items
   zero-length). In practice, connected-data items have non-zero length, further limiting
   the count.
4. **Recognized type_ids**: `0x00B1` (Connected Data Item) and `0x00B2` (Unconnected Data
   Item) are the CIP payload carriers. Other type_ids (NullAddressItem 0x0000,
   ConnectedAddressItem 0x00A1, etc.) are parsed but their data is not further inspected at
   this layer. The caller (`process_pdu`) dispatches on type_id.
5. **Cursor arithmetic**: cursor advances by exactly `4 + item.length` per item. No
   attacker-controlled value is used as a slice index beyond the bounds check.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `payload.len() == 0` | Returns `vec![]` (cannot read item_count) |
| EC-002 | `payload.len() == 1` | Returns `vec![]` (cannot read 2-byte item_count) |
| EC-003 | `payload.len() == 2`, `item_count == 0` | Returns `vec![]` (no items declared) |
| EC-004 | `item_count == 0xFFFF` with 8-byte payload | Declares 65535 items but only 1 complete item fits (6 bytes for item header + zero data + 2 for count); iteration breaks after first-or-zero items; no panic |
| EC-005 | Valid `SendRRData` with 2 CPF items (NullAddressItem + UnconnectedData) | Returns `vec![CpfItem{0x0000, []}, CpfItem{0x00B2, <cip-data>}]` |
| EC-006 | `item.length` value would exceed remaining payload | Iteration breaks at that item; earlier items still returned |
| EC-007 | `type_id == 0x00B1` (Connected Data Item) | Item included in result with its data slice; caller dispatches for CIP parse |
| EC-008 | `type_id == 0x00B2` (Unconnected Data Item) | Item included in result with its data slice; caller dispatches for CIP parse |

## Canonical Test Vectors

**Two-item CPF payload (NullAddressItem + UnconnectedData):**
```
Payload (hex): 02 00                   // item_count = 2 (LE)
               00 00  00 00            // item[0]: type=0x0000 (NullAddr), length=0
               B2 00  10 00            // item[1]: type=0x00B2 (UnconnectedData), length=16
               <16 bytes CIP data>
```
Expected: `vec![CpfItem{type_id:0x0000, data:[]}, CpfItem{type_id:0x00B2, data:[16 bytes]}]`

**Truncated item_count (short payload):**
```
Payload (hex): 05                      // only 1 byte — cannot read LE u16
```
Expected: `vec![]`

**Giant declared item_count with tiny payload:**
```
Payload (hex): FF FF  00 00  00 00     // item_count=65535, one 4-byte item header (zero-length item)
```
Expected: `vec![CpfItem{type_id:0x0000, data:[]}]` (one item parsed; break at item 2 due to payload exhaustion)

| Payload content | item_count LE | Expected result | Notes |
|----------------|---------------|----------------|-------|
| `[]` | N/A | `vec![]` | empty |
| `[02 00]` | 2 | `vec![]` | count=2 but no item headers follow |
| Valid 2-item SendRRData | 2 | 2 CpfItems | happy path |
| `[FF FF, 4 bytes item-header]` | 65535 | 1 CpfItem | bounds limits to 1 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Bounds-safe iteration with early-break: not a Kani target (loop with variable iteration count); unit test verifies truncated payloads and giant item_count | unit test |

Note: `parse_cpf_items` is listed in ADR-010 §4.3 as an additional pure-core function NOT
targeted by VP-032 Kani in v0.11.0 (only the four Sub-A/B/C/D functions are Kani targets).
The bounds safety is established by invariants and unit test coverage.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — CPF item parsing is required for all CIP service extraction (which underlies MITRE detections T0858/T0816/T0836/T0888); safe bounded iteration prevents DoS via malformed CPF item_count |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 2 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — CPF parse layer; no finding emission directly) |

## Related BCs

- BC-2.17.002 — depends on (header.length bounds the payload slice passed to this function)
- BC-2.17.003 — depends on (only called for valid ENIP frames: SendRRData/SendUnitData)
- BC-2.17.006 — composes with (CipHeader is parsed from CpfItem.data for type_id 0x00B1/0x00B2)
- BC-2.17.007 — depends on (CIP service classification applies to CipHeader extracted from items)

## Architecture Anchors

- `src/analyzer/enip.rs` — `fn parse_cpf_items(payload: &[u8]) -> Vec<CpfItem>` — pure-core free function
- `src/analyzer/enip.rs` — `struct CpfItem { type_id: u16, data: Vec<u8> }`
- `src/analyzer/enip.rs` — `process_pdu` calls `parse_cpf_items` on `payload = &frame[24..]`
- `.factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md §Decision 2` — CPF item-walk pseudocode

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — additional pure-core function not in VP-032 Kani scope for v0.11.0; covered by unit tests)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 2 (parse_cpf_items pseudocode); ODVA EtherNet/IP Specification §2-6.1 (CPF little-endian item structure) |
| **Confidence** | high — CPF little-endian byte order and item structure are normative ODVA; bounds-safe iteration is an architectural invariant |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same payload always produces same item list |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core (additional, not VP-032 Kani target in v0.11.0) |
