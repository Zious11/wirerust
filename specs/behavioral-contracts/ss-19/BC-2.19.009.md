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

# BC-2.19.009: `classify_frame_format` Totality — UFormat for All Remaining CF1 Values (VP-046)

## Description

`classify_frame_format(cf1: u8) -> FrameFormat` is a total function over all 256 u8 values.
When CF1 bits 1:0 are `0b11`, the frame is U-format (Unnumbered). This is the residual case
after I-format (bit0=0) and S-format (bits1:0=0b01) are matched. U-format frames carry
STARTDT/STOPDT/TESTFR control commands via CF1 values defined in ADR-013 Decision 5.
This BC establishes totality (no missing cases) and is the primary target of VP-046
proptest `classify_oracle` verification.

## Preconditions

1. `cf1` is any u8 value.
2. `cf1 & 0x01 != 0` AND `cf1 & 0x03 != 0x01` — not I-format and not S-format, therefore bits1:0=0b11.

## Postconditions

1. `classify_frame_format(cf1)` returns `FrameFormat::UFormat`.
2. The caller reads CF1 as a U-command code and dispatches to U-format session state machine (BC-2.19.010..014).
3. No ASDU body parsing is attempted for U-format frames (LEN is always 4).

## Invariants

1. **Totality**: the three-way partition {bit0=0, bits1:0=0b01, bits1:0=0b11} is exhaustive and mutually exclusive for all 256 u8 values. VP-046 proptest verifies via an independent oracle function (`proptest_vp046_frame_format_totality`), unrelated to the dispatcher `classify_oracle` (Decision 9).
2. **U-format LEN constraint**: IEC-104 §5.4 mandates LEN=4 for U-frames; LEN≠4 → T0814.
3. **Canonical U-frame CF1 values**: STARTDT-act=0x07, STARTDT-con=0x0B, STOPDT-act=0x13, STOPDT-con=0x23, TESTFR-act=0x43, TESTFR-con=0x83. Non-canonical CF1 in U-format → T0814 per BC-2.19.014.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `cf1 == 0x07` (STARTDT-act) | UFormat → session_started=true per BC-2.19.010 |
| EC-002 | `cf1 == 0x13` (STOPDT-act) | UFormat → T0881 per BC-2.19.011/012 |
| EC-003 | `cf1 == 0x43` (TESTFR-act) | UFormat → keepalive, no finding per BC-2.19.013 |
| EC-004 | `cf1 == 0xFF` (non-canonical) | UFormat → T0814 per BC-2.19.014 |
| EC-005 | All 64 CF1 values with bits1:0=0b11 | All return UFormat |

## Canonical Test Vectors

| Input CF1 | Expected | Notes |
|-----------|----------|-------|
| `0x07` | `UFormat` | STARTDT-act (canonical) |
| `0x0B` | `UFormat` | STARTDT-con (canonical) |
| `0x13` | `UFormat` | STOPDT-act (canonical) |
| `0x03` | `UFormat` | non-canonical U CF1 |
| `0xFF` | `UFormat` | non-canonical U CF1 |
| `0x00` | `IFormat` | exhaustive check: bit0=0 → NOT U |
| `0x01` | `SFormat` | exhaustive check: bits1:0=0b01 → NOT U |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-046 | `classify_frame_format` is total over all 256 u8 values; every value maps to exactly one of {IFormat, SFormat, UFormat}; verified by an independent proptest oracle (not the dispatcher `classify_oracle`) | proptest: `proptest_vp046_frame_format_totality` — 256-value exhaustive sweep |
| VP-047 | No panic via fuzz harness | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — totality of frame format classification is a correctness requirement for the IEC-104 session state machine and ASDU dispatch |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 4 (VP-046 frame-format oracle obligation) |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure classification; downstream BCs handle MITRE mapping) |

## Related BCs

- BC-2.19.007 — composes with (I-format partition)
- BC-2.19.008 — composes with (S-format partition)
- BC-2.19.010..014 — depends on (U-format session state machine)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn classify_frame_format`: `else { FrameFormat::UFormat }` (exhaustive residual)
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 4` — three-way partition; VP-046 independent proptest oracle obligation

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-046 — `proptest_vp046_frame_format_totality` (U-format residual partition; exhaustive 256-value sweep with oracle)
- VP-047 — `fuzz_iec104_parser`
