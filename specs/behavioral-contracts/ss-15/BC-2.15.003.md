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

# BC-2.15.003: DEST/SOURCE Addresses Decoded Little-Endian from Fixed Offsets 4–7

## Description

When `parse_dnp3_dl_header(data)` returns `Some(Dnp3DlHeader)`, the DESTINATION field is
decoded as `u16::from_le_bytes([data[4], data[5]])` and the SOURCE field is decoded as
`u16::from_le_bytes([data[6], data[7]])`. DNP3 link-layer addresses are little-endian per
IEEE 1815-2012 — the low byte arrives first on the wire. This BC isolates the LE decoding
correctness property targeted by VP-023 Sub-property A field-decode assertions.

## Preconditions

1. `data` is a `&[u8]` slice with `data.len() >= 10`.
2. `parse_dnp3_dl_header(data)` returns `Some(h)` (BC-2.15.001 postcondition).

## Postconditions

1. `h.destination == u16::from_le_bytes([data[4], data[5]])`.
   - Low byte: `data[4]`; high byte: `data[5]`.
   - Wire bytes `[0x03, 0x00]` → destination address 0x0003 (outstation 3).
   - Wire bytes `[0xFF, 0xFF]` → destination address 0xFFFF (broadcast "no confirmation").
   - Wire bytes `[0xFD, 0xFF]` → destination address 0xFFFD (broadcast, confirmation required).
2. `h.source == u16::from_le_bytes([data[6], data[7]])`.
   - Low byte: `data[6]`; high byte: `data[7]`.
   - Wire bytes `[0x01, 0x00]` → source address 0x0001 (master 1).
3. Big-endian decode of the same bytes would produce the WRONG address:
   `u16::from_be_bytes([0x03, 0x00])` = 0x0300 ≠ 0x0003. The implementation MUST use
   `from_le_bytes`, not `from_be_bytes` or native-endian conversion.

## Invariants

1. **Little-endian is the wire format** [SPEC: IEEE 1815-2012; dnp3-research.md §1.1]:
   "DESTINATION: little-endian (low byte first)" and "SOURCE: little-endian (low byte first)."
   This is one of the few places where DNP3 differs from Modbus (which uses big-endian).
2. **Fixed offsets**: DEST is always at bytes 4–5, SOURCE always at bytes 6–7, regardless of
   frame length, control flags, or function code. [SPEC]
3. **Kani proof**: VP-023 Sub-property A asserts
   `h.destination == u16::from_le_bytes([data[4], data[5]])` and
   `h.source == u16::from_le_bytes([data[6], data[7]])` symbolically over all 10-byte inputs.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `[data[4], data[5]] = [0x01, 0x00]` — address 1 LE | `destination = 0x0001` (not 0x0100) |
| EC-002 | `[data[4], data[5]] = [0x00, 0x01]` — address 256 LE | `destination = 0x0100` (not 0x0001) |
| EC-003 | `[data[4], data[5]] = [0xFF, 0xFF]` — broadcast 0xFFFF LE | `destination = 0xFFFF` — broadcast no-confirm |
| EC-004 | `[data[4], data[5]] = [0xFD, 0xFF]` — broadcast 0xFFFD LE | `destination = 0xFFFD` — broadcast confirm-required |
| EC-005 | `[data[4], data[5]] = [0xFE, 0xFF]` — broadcast 0xFFFE LE | `destination = 0xFFFE` — broadcast optional-confirm |
| EC-006 | `[data[6], data[7]] = [0x00, 0x00]` — source address 0 | `source = 0x0000` — valid address 0 |
| EC-007 | `[data[6], data[7]] = [0xFF, 0xEF]` — source 0xEFFF | `source = 0xEFFF` (low byte 0xFF, high byte 0xEF; LE) |

## Canonical Test Vectors

| Wire bytes (positions 4–7 of a 10-byte header) | Expected destination | Expected source | Category |
|------------------------------------------------|---------------------|----------------|----------|
| dest: `03 00`, src: `01 00` | 0x0003 (outstation 3) | 0x0001 (master 1) | happy-path: typical master→outstation control |
| dest: `FF FF`, src: `01 00` | 0xFFFF (broadcast) | 0x0001 (master 1) | happy-path: broadcast no-confirm |
| dest: `FD FF`, src: `02 00` | 0xFFFD (broadcast) | 0x0002 (master 2) | happy-path: broadcast confirm-required |
| dest: `00 00`, src: `00 00` | 0x0000 | 0x0000 | edge-case: zero addresses |
| dest: `00 01`, src: `FF EF` | 0x0100 | 0xEFFF | edge-case: LE vs BE disambiguation (0x0100, not 1) |

**Full 10-byte frame with annotated LE decode:**
```
Bytes:  05 64 0E C4  03 00  01 00  88 C5
                     ┌────┐ ┌────┐
                     DEST   SRC    CRC
LE decode:
  DEST = from_le_bytes([0x03, 0x00]) = 0x0003  (outstation 3)
  SRC  = from_le_bytes([0x01, 0x00]) = 0x0001  (master 1)
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-023 | Sub-property A (field decode): `h.destination == u16::from_le_bytes([data[4], data[5]])` and `h.source == u16::from_le_bytes([data[6], data[7]])` for all symbolic 10-byte inputs | Kani: symbolic `[u8; 12]` with `len >= 10`; explicit assertions in `verify_parse_dnp3_dl_header_safety` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — correct LE address decoding is required by all downstream detection BCs (broadcast anomaly BC-2.15.018, unauthorized control source attribution in BC-2.15.010) which depend on accurate DEST/SRC address values |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — address correctness ensures findings are attributed to the correct source address) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — pure parse function; no finding emission) |

## Related BCs

- BC-2.15.001 — depends on (this BC's postconditions are only reachable when BC-2.15.001's precondition `data.len() >= 10` holds)
- BC-2.15.004 — composes with (validity gate checks `start1`, `start2`, `length` — not DEST/SRC)
- BC-2.15.018 — depends on (broadcast anomaly detection depends on correctly decoded `destination` value)
- BC-2.15.010 — depends on (unauthorized control detection uses `source` + `control.DIR` for attribution)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `fn parse_dnp3_dl_header` — LE decode via `u16::from_le_bytes([data[4], data[5]])` and `u16::from_le_bytes([data[6], data[7]])`
- `.factory/research/dnp3-research.md §1.1` — "DESTINATION: little-endian (low byte first)" / "SOURCE: little-endian (low byte first)" [SPEC]
- `.factory/specs/verification-properties/vp-023-dnp3-parse-safety.md` — Sub-property A field-decode assertion: `h.destination == u16::from_le_bytes([data[4], data[5]])`

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-023 — DNP3 Data-Link Frame Parse Safety and Function-Code Classification (Sub-property A: field decode correctness)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-research.md §1.1; dnp3-architecture-delta.md §3 (struct field comments); ADR-007 Decision 2 |
| **Confidence** | high — IEEE 1815-2012 and all concurring references (RACOM, Wireshark, Chipkin) confirm little-endian DEST/SOURCE; dnp3-research.md §1.1 cited verbatim |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — same bytes always produce same LE addresses |
| **Thread safety** | Send + Sync (pure function, no state) |
| **Overall classification** | pure core — VP-023 Kani target (Sub-A, field decode) |
