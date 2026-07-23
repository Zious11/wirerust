---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified:
  - "v1.1: wave-85-spec-evolution (2026-07-23) — narrow silently-logged range: 52–99 → {52–57} and {65–99}, excluding TypeIDs 58–64 which are now handled by BC-2.19.029 (58–60) and BC-2.19.030 (61–64). Updated Description, Invariant 1, and code comment reference. No change to Precondition 2 or Postconditions (T0814 still only for TypeID=0 and 128–255). inputs: replaced with [] / d41d8cd per PG-HASH-HOOK-DIVERGENCE workaround."
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: []
input-hash: "d41d8cd"
---

# BC-2.19.022: Reserved or Invalid TypeID Emits T0814 Anomaly

## Description

IEC 60870-5-101/104 defines TypeIDs 1–127 for standard information objects; TypeID 0 is
undefined; TypeIDs 128–255 are private-use or reserved. When a parsed ASDU has TypeID=0
or a TypeID in [128, 255], the analyzer emits T0814 "Denial of Service" (ICS) with
confidence Possible. This catch-all prevents silent fallthrough for unrecognized TypeIDs
that are not in the explicit handling sets.

The currently handled sets (TypeIDs that have explicit arms in `detect_iec104_threats`) are:
- 45–47 (C_SC/C_DC/C_RC, switching commands — BC-2.19.019)
- 48–51 (C_SE/C_BO, set-point/bitstring — BC-2.19.019)
- 58–60 (C_SC_TA/C_DC_TA/C_RC_TA, timed switching — BC-2.19.029)
- 61–64 (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA, timed set-point/bitstring — BC-2.19.030)
- 100, 101, 103 (interrogation/clock-sync — BC-2.19.021)
- 105 (C_RP_NA_1 reset — BC-2.19.020)
- 0 and 128–255 (this BC — T0814)

TypeIDs in [1, 44], [52, 57], [65, 99], [102], [104], or [106, 127] that are not explicitly
handled are silently logged without a finding (handled silently as defined-but-unhandled
TypeIDs that do not warrant anomaly attribution).

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id == 0` OR `asdu.type_id >= 128`.

## Postconditions

1. T0814 "Denial of Service" finding emitted (confidence Possible).
2. Finding includes the invalid TypeID value.
3. No further ASDU processing (type_id-specific fields are not decoded).

## Invariants

1. **Reserved-TypeID scope**: only TypeID=0 and 128–255 emit T0814 from this BC; unrecognized
   but defined TypeIDs in [1, 127] are silently logged (future-proof). The silently-logged
   defined TypeIDs are: [1, 44] (monitoring direction), [52, 57] (reserved; formerly part of
   the broader [52, 99] range before BC-2.19.029/030 were introduced), [65, 99] (process
   information in control direction and other defined types), 102 (C_RD_NA_1 read data), 104
   (delay acquisition command), [106, 127] (parameter activation and other defined types).
   TypeIDs 58–64 were previously silently logged; they are now handled by BC-2.19.029 (58–60)
   and BC-2.19.030 (61–64) and are therefore no longer in the silently-logged set.
2. **Fail-closed on reserved**: reserved TypeIDs in production traffic indicate implementation
   errors or fuzzing attacks.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=0 | T0814 Possible |
| EC-002 | TypeID=128 | T0814 Possible |
| EC-003 | TypeID=255 | T0814 Possible |
| EC-004 | TypeID=44 (in defined but unhandled range) | No finding (silently logged) |
| EC-005 | TypeID=52 (RESERVED, neighbor below timed switching arm) | No finding (silently logged per Invariant 1; BC-2.19.029 regression guard) |
| EC-006 | TypeID=57 (RESERVED, upper neighbor below timed switching arm) | No finding (silently logged) |
| EC-007 | TypeID=65 (unhandled, lower neighbor above timed set-point arm) | No finding (silently logged per Invariant 1; BC-2.19.030 regression guard) |
| EC-008 | TypeID=99 (unhandled, upper bound of unhandled range above timed arms) | No finding (silently logged) |

## Canonical Test Vectors

| TypeID | Expected |
|--------|----------|
| 0 | T0814 Possible |
| 128 | T0814 Possible |
| 255 | T0814 Possible |
| 44 | (no finding — silently logged) |
| 52 | (no finding — silently logged; was previously in [52, 99]; now split at 58) |
| 65 | (no finding — silently logged; was previously in [52, 99]; now split at 64) |

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
| Architecture Module | SS-19 (src/analyzer/iec104.rs `detect_iec104_threats`); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" — Possible |

## Related BCs

- BC-2.19.019 — composes with (TypeIDs 45–51: control commands)
- BC-2.19.021 — composes with (TypeIDs 100/101/103: interrogation)
- BC-2.19.029 — narrowed by (v1.1: TypeIDs 58–60 removed from silently-logged set; now handled)
- BC-2.19.030 — narrowed by (v1.1: TypeIDs 61–64 removed from silently-logged set; now handled)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `0 | 128..=255 => { emit T0814(Possible) }` (unchanged from feature-iec104 delivery)
- `src/analyzer/iec104.rs` — `_` catch-all comment updated: "Defined-but-unhandled TypeIDs in [1, 127] not covered by the arms above: TypeIDs 1–44 (monitoring direction), 52–57, 65–99, 102 (C_RD_NA_1), 104, 106–127. TypeIDs 58–64 were here prior to wave-85-spec-evolution; they are now handled by BC-2.19.029 (58–60) and BC-2.19.030 (61–64)."
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition; original feature-iec104 delivery)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
