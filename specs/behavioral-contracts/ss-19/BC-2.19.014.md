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

# BC-2.19.014: Non-Canonical U-Frame CF1 Emits T0814 Anomaly (CVE-2026-1773 Pattern)

## Description

IEC-104 defines exactly six canonical U-frame CF1 values: STARTDT-act=0x07,
STARTDT-con=0x0B, STOPDT-act=0x13, STOPDT-con=0x23, TESTFR-act=0x43, TESTFR-con=0x83.
Any other CF1 value in a U-format frame (bits1:0=0b11) is non-canonical and undefined
by the spec. Such frames are associated with the CVE-2026-1773 class of protocol fuzzing
attacks where non-canonical U-frame injections are used to confuse IEC-104 implementations.
The analyzer emits T0814 "Denial of Service" (ICS) with confidence Possible.

## Preconditions

1. A valid U-format APCI frame has been parsed (CF1 bits1:0 = 0b11, per BC-2.19.009).
2. CF1 is NOT one of {0x07, 0x0B, 0x13, 0x23, 0x43, 0x83}.

## Postconditions

1. T0814 "Denial of Service" finding emitted with confidence Possible.
2. The finding message includes the CF1 value (hex) for analyst inspection.
3. `Iec104FlowState` is not modified (no session state change from an invalid U-frame).
4. The frame is otherwise silently discarded.

## Invariants

1. **Fail-closed**: non-canonical U-frame does not advance session state.
2. **CVE-2026-1773 coverage**: this BC covers the specific attack pattern of injecting reserved U-frame CF1 values to confuse stateful IEC-104 implementations.
3. **Canonical set exclusivity**: the six canonical CF1 values are fixed by IEC 60870-5-104 §5.4 and do not change at runtime.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CF1=0x03 (bits1:0=0b11, non-canonical) | T0814 Possible |
| EC-002 | CF1=0xFF (all bits set) | T0814 Possible |
| EC-003 | CF1=0x0F (bits1:0=0b11, non-canonical) | T0814 Possible |
| EC-004 | All 58 non-canonical U-frame CF1 values (64 total minus 6 canonical) | All emit T0814 Possible |

## Canonical Test Vectors

| CF1 | Expected | Notes |
|-----|----------|-------|
| `0x03` | T0814 Possible | non-canonical U-frame |
| `0xFF` | T0814 Possible | non-canonical U-frame (all bits set) |
| `0x07` | no finding | canonical STARTDT-act (BC-2.19.010) |
| `0x43` | no finding | canonical TESTFR-act (BC-2.19.013) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on any non-canonical U-frame CF1 | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — non-canonical U-frame detection (CVE-2026-1773) is a key ICS threat indicator in the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 5 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" — Possible confidence; associated with CVE-2026-1773 |

## Related BCs

- BC-2.19.013 — composes with (canonical TESTFR: no finding)
- BC-2.19.010..013 — composes with (all canonical U-frame handlers; this is the else-branch)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `else { emit T0814(Possible, cf1); }` (after all canonical CF1 matches)
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 5`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
