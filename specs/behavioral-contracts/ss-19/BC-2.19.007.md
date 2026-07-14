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

# BC-2.19.007: `classify_frame_format` Returns IFormat When CF1 Bit 0 = 0

## Description

`classify_frame_format(cf1: u8) -> FrameFormat` is a pure-core free function that
discriminates among the three IEC-104 frame formats based on the two least-significant bits
of CF1 (ADR-013 Decision 4). When `cf1 & 0x01 == 0` (bit 0 is zero), the frame is an
I-format (Information Transfer) frame. I-format frames carry an ASDU payload and include
15-bit send sequence counter N(S) (in CF1–CF2) and receive sequence counter N(R) (in CF3–CF4).
This is the data path for all SCADA telemetry and control commands.

## Preconditions

1. `cf1` is the first control-field octet of a complete, validated APCI frame.
2. `cf1 & 0x01 == 0` (bit 0 is zero).

## Postconditions

1. `classify_frame_format(cf1)` returns `FrameFormat::IFormat`.
2. The caller extracts N(S) from CF1–CF2 and N(R) from CF3–CF4.
3. The caller passes the remaining APCI bytes (LEN - 4) as the ASDU body to ASDU parsing.

## Invariants

1. **I-format discrimination rule**: bit 0 of CF1 is the authoritative discriminant for I-format (ADR-013 Decision 4).
2. **N(S) extraction**: `N(S) = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)` (15-bit, little-endian).
3. **Mutual exclusivity**: I-format (bit0=0) is disjoint from S-format (bits1:0=0b01) and U-format (bits1:0=0b11) — proven by VP-046.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `cf1 == 0x00` (all zeros) | I-format: N(S)=0 |
| EC-002 | `cf1 == 0x02` (bit0=0, bit1=1) | I-format |
| EC-003 | `cf1 == 0xFE` (all bits set except bit0) | I-format |
| EC-004 | `cf1 == 0x01` (bit0=1) | NOT I-format: see BC-2.19.008 (S) or BC-2.19.009 (U) |
| EC-005 | All 128 even CF1 values (0,2,4,...,254) | All return IFormat |

## Canonical Test Vectors

| Input CF1 | Expected | Category |
|-----------|----------|---------|
| `0x00` | `IFormat` | I-frame: N(S)=0 |
| `0x02` | `IFormat` | I-frame: N(S)=1 |
| `0x7E` | `IFormat` | I-frame: N(S)=63 |
| `0x01` | `SFormat` | NOT I-frame (bit0=1, bit1=0) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-046 | For all 128 even CF1 values (bit0=0), `classify_frame_format` returns `IFormat`; totality over all 256 u8 values | proptest: `proptest_vp046_frame_format_totality` |
| VP-047 | No panic via fuzz harness | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — I-format frame discrimination is the entry point for all ASDU parsing and control-command detection in the IEC-104 analyzer |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 4 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure classification function; downstream BCs emit findings) |

## Related BCs

- BC-2.19.008 — composes with (S-format: bits1:0=0b01)
- BC-2.19.009 — composes with (U-format: bits1:0=0b11; totality)
- BC-2.19.015 — depends on (ASDU minimum-length guard for I-frame ASDU body)
- BC-2.19.023 — depends on (N(S)/N(R) extraction from I-frame control fields)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn classify_frame_format(cf1: u8) -> FrameFormat`: `if cf1 & 0x01 == 0 { FrameFormat::IFormat }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 4`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-046 — `proptest_vp046_frame_format_totality` (I-format partition: bit0=0)
- VP-047 — `fuzz_iec104_parser`
