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

# BC-2.19.022: Reserved or Invalid TypeID Emits T0814 Anomaly

## Description

IEC 60870-5-101/104 defines TypeIDs 1–127 for standard information objects; TypeID 0 is
undefined; TypeIDs 128–255 are private-use or reserved. When a parsed ASDU has TypeID=0
or a TypeID in [128, 255], the analyzer emits T0814 "Denial of Service" (ICS) with
confidence Possible. This catch-all prevents silent fallthrough for unrecognized TypeIDs
that are not in the explicit handling sets (45–51, 100/101/103, 105). TypeIDs in [1, 44]
or [53, 99] or [102, 104] or [106, 127] that are not explicitly handled are logged without
a finding (handled silently as future TypeIDs).

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id == 0` OR `asdu.type_id >= 128`.

## Postconditions

1. T0814 "Denial of Service" finding emitted (confidence Possible).
2. Finding includes the invalid TypeID value.
3. No further ASDU processing (type_id-specific fields are not decoded).

## Invariants

1. **Reserved-TypeID scope**: only TypeID=0 and 128–255 emit T0814 from this BC; unrecognized but defined TypeIDs in [1, 127] are silently logged (future-proof).
2. **Fail-closed on reserved**: reserved TypeIDs in production traffic indicate implementation errors or fuzzing attacks.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=0 | T0814 Possible |
| EC-002 | TypeID=128 | T0814 Possible |
| EC-003 | TypeID=255 | T0814 Possible |
| EC-004 | TypeID=44 (in defined but unhandled range) | No finding (silently logged) |

## Canonical Test Vectors

| TypeID | Expected |
|--------|----------|
| 0 | T0814 Possible |
| 128 | T0814 Possible |
| 255 | T0814 Possible |
| 44 | (no finding) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on reserved TypeID | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — reserved/invalid TypeID detection prevents silent fallthrough on malformed or adversarially crafted ASDU frames |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" — Possible |

## Related BCs

- BC-2.19.019 — composes with (TypeIDs 45–51: control commands)
- BC-2.19.021 — composes with (TypeIDs 100/101/103: interrogation)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if asdu.type_id == 0 || asdu.type_id >= 128 { emit T0814(Possible); }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
