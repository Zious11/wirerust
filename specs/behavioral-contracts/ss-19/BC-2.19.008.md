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
input-hash: "f5a97d3"
---

# BC-2.19.008: `classify_frame_format` Returns SFormat When CF1 Bits 1:0 = 0b01

## Description

`classify_frame_format(cf1: u8) -> FrameFormat` returns `FrameFormat::SFormat` when the
two least-significant bits of CF1 are `0b01` (i.e., `cf1 & 0x03 == 0x01`). S-format
(Supervisory) frames carry no ASDU. They are used solely for acknowledgement of received
I-format frames via the N(R) counter in CF3–CF4. S-format frames are the IEC-104 equivalent
of TCP ACK-only segments in the application layer. The analyzer records N(R) but emits no
finding for a normal S-frame.

## Preconditions

1. `cf1` is the first control-field octet of a validated APCI frame (LEN=4).
2. `cf1 & 0x03 == 0x01` (bits 1:0 = 0b01).

## Postconditions

1. `classify_frame_format(cf1)` returns `FrameFormat::SFormat`.
2. Caller reads N(R) from CF3–CF4: `N(R) = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)`.
3. No ASDU body parsing is attempted for an S-frame (LEN is always 4).
4. No finding is emitted for a well-formed S-frame.

## Invariants

1. **S-format LEN constraint**: IEC-104 §5.3 mandates LEN=4 for S-frames; any S-frame with LEN≠4 violates the spec and the caller must emit T0814.
2. **Acknowledgement-only**: S-format frames contain only N(R) (acknowledgement number); no data payload.
3. **Bit pattern exclusivity**: `cf1 & 0x03 == 0x01` is the exclusive S-format indicator (bit0=1, bit1=0).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `cf1 == 0x01` (minimal S-frame indicator) | SFormat |
| EC-002 | `cf1 == 0x03` (bits1:0=0b11) | NOT SFormat — UFormat (see BC-2.19.009) |
| EC-003 | S-frame with LEN≠4 | SFormat returned but caller emits T0814 |
| EC-004 | All 64 CF1 values with bits1:0=0b01 (0x01,0x05,...,0xFD) | All return SFormat |

## Canonical Test Vectors

| Input CF1 | Expected | Category |
|-----------|----------|---------|
| `0x01` | `SFormat` | S-frame: N(R)=0 |
| `0x05` | `SFormat` | S-frame: N(R) from CF3/CF4 |
| `0x00` | `IFormat` | NOT S-frame (bit0=0) |
| `0x03` | `UFormat` | NOT S-frame (bits1:0=0b11) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-046 | For all 64 CF1 values where `cf1 & 0x03 == 0x01`, returns `SFormat`; totality over all 256 u8 values | proptest: `proptest_vp046_frame_format_totality` |
| VP-047 | No panic via fuzz harness | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — S-format discrimination is required to correctly route supervisor frames (which carry no ASDU) away from the ASDU parsing path |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 4 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure classification; T0814 emitted by caller on LEN constraint violation) |

## Related BCs

- BC-2.19.007 — composes with (I-format: bit0=0)
- BC-2.19.009 — composes with (U-format: bits1:0=0b11; totality proof)
- BC-2.19.023 — depends on (N(R) acknowledgement counter extraction)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn classify_frame_format`: `else if cf1 & 0x03 == 0x01 { FrameFormat::SFormat }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 4`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-046 — `proptest_vp046_frame_format_totality` (S-format partition: bits1:0=0b01)
- VP-047 — `fuzz_iec104_parser`
