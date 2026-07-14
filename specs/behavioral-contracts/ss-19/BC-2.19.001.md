---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "a153144"
---

# BC-2.19.001: `parse_apci_header` Returns None for Input Shorter Than 6 Bytes

## Description

`parse_apci_header(data: &[u8]) -> Option<ApciHeader>` is the pure-core entry function for
IEC-104 APCI header parsing. The APCI header is exactly 6 bytes: start byte `0x68`, one LEN
octet, and four control-field octets (CF1–CF4). When `data.len() < 6`, the function returns
`None` immediately without accessing any bytes. No partial header is decoded. This is the
length-reject path; the accept path is BC-2.19.005.

## Preconditions

1. `data` is a `&[u8]` slice of reassembled, in-order TCP bytes (per `StreamHandler::on_data`).
2. `data.len() < 6` — fewer bytes than the minimum 6-byte APCI header.
3. No alignment assumptions affect this path: zero bytes are read.

## Postconditions

1. `parse_apci_header(data)` returns `None`.
2. No bytes in `data` are accessed; no panics are possible.
3. The function is pure: no I/O, no global state mutation, no side effects.
4. The frame-walk caller treats `None` as "incomplete header" and stashes `data` into the
   carry buffer (bounded at `MAX_IEC104_CARRY_BYTES = 255`).

## Invariants

1. **Minimum-header guard**: the 6-byte minimum is fixed by IEC 60870-5-104 §5.1
   (start byte + LEN + 4 control octets). It is not configurable.
2. **Purity**: `parse_apci_header` is a pure-core free function — VP-044 Kani target.
   No state mutation occurs inside this function.
3. **No partial decode**: the function does not attempt to decode the start byte or LEN field
   when `data.len() < 6`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data.len() == 0` (empty slice) | Returns `None` — no bytes accessed, no panic |
| EC-002 | `data.len() == 1` | Returns `None` — one byte, not accessed |
| EC-003 | `data.len() == 5` (one byte short) | Returns `None` — 5 bytes, none accessed |
| EC-004 | `data.len() == 6` (exactly minimum) | Proceeds to start-byte and LEN validation; see BC-2.19.005 |
| EC-005 | `data = [0x68, 0x04, 0x07, 0x00, 0x00]` (5 bytes, looks like start of valid frame) | Returns `None` — length check fires before any field access |

## Canonical Test Vectors

| Input (hex bytes, total length) | Expected result | Category |
|---------------------------------|----------------|---------|
| `[]` (0 bytes) | `None` | reject: empty |
| `[0x68, 0x04, 0x07, 0x00, 0x00]` (5 bytes) | `None` | reject: one byte short |
| `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` (6 bytes) | `Some(ApciHeader{start:0x68,len:4,cf1:0x07,...})` | accept: exact minimum — see BC-2.19.005 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-044 | Sub-A: `parse_apci_header(data)` returns `None` for all inputs with `len < 6`; never panics for any symbolic input up to BOUND=260; for any returned `Some(h)`, `h.len + 2` is in [6, 255] with no integer overflow | Kani: `verify_parse_apci_header_safety` |
| VP-047 | No panic on arbitrary byte input at `on_data` entry point (fuzz harness covering this code path) | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — this BC defines the length-reject path for the APCI header parser, which is the foundational entry function for all IEC-104 analysis in SS-19 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — IEC-104 flows are only routed after TLS/HTTP content rules fail; this BC fires only on port-2404-classified flows per ADR-013 Decision 1) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decisions 3, 8 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.19.002 — composes with (next rejection: start byte ≠ 0x68 when len ≥ 6)
- BC-2.19.003 — composes with (LEN < 4 rejection path)
- BC-2.19.004 — composes with (LEN > 253 rejection path)
- BC-2.19.005 — composes with (accept path: `data.len() >= 6`, start byte valid, LEN valid)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn parse_apci_header(data: &[u8]) -> Option<ApciHeader>` pure-core free function
- `src/analyzer/iec104.rs` — frame-walk loop in `Iec104Analyzer::on_data()`: `if buf.len() - cursor < 6 { carry = buf[cursor..]; break }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` — frame-walk loop pseudocode step 2
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 8` — pure-core free-fn design and VP-044 Kani skeleton

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-044 — `verify_parse_apci_header_safety`: arithmetic safety, no panic, `len < 6` → None
- VP-047 — `fuzz_iec104_parser`: cargo-fuzz no-panic harness for `on_data` entry point

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same result |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-044 Kani target |
