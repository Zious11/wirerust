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
  - "v1.1: F-P6-H1 — H1 title label corrected from 'Set-Point TypeIDs 48–51' to 'Set-Point + Bitstring TypeIDs 48–51' (TypeID 51 = C_BO_NA_1 is a bitstring write, not a set-point; set-points are TypeIDs 48–50 only). Body description/postconditions/invariants were already correct. Title-only fix. 2026-07-14"
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

# BC-2.19.019: Control Command TypeIDs 45–51 Emit T1692.001; Set-Point + Bitstring TypeIDs 48–51 Also Emit T0836

## Description

When a parsed ASDU has a TypeID in the set {45, 46, 47, 48, 49, 50, 51}
(C_SC_NA_1, C_DC_NA_1, C_RC_NA_1, C_SE_NA_1, C_SE_NB_1, C_SE_NC_1, C_BO_NA_1),
the analyzer emits T1692.001 "Unauthorized Message: Command Message" (ICS) for all seven
TypeIDs. Additionally, for the set-point and bitstring TypeIDs {48, 49, 50, 51}
(C_SE_NA_1, C_SE_NB_1, C_SE_NC_1, C_BO_NA_1), T0836 "Modify Parameter" is co-emitted.
TypeIDs 45–47 (switching commands C_SC, C_DC, C_RC) emit T1692.001 only — they represent
binary control rather than parameter modification. TypeID 52 is RESERVED in IEC 60870-5-101/104
and is not in the control-command range; C_SE_ND_1 does not exist. On a passive-monitoring
view, all seven TypeIDs indicate an active control session; combined with COT=6 (activation)
they are high-confidence control events.

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id` is in {45, 46, 47, 48, 49, 50, 51}.
3. ASDU parse returned `Some(Asdu)`.

## Postconditions

1. T1692.001 "Unauthorized Message: Command Message" finding emitted (confidence Possible) for all TypeIDs 45–51.
2. T0836 "Modify Parameter" finding emitted (confidence Possible) ONLY for set-point/bitstring TypeIDs 48–51.
3. Findings include CASDU and first_ioa as target address context.
4. COT=6 (activation) in combination with a control TypeID may be noted in the finding message.

## Invariants

1. **T1692.001 for all command TypeIDs**: T1692.001 is emitted for every TypeID in 45–51; it is the command-message indicator.
2. **T0836 only for set-point/bitstring writes**: T0836 is emitted for TypeIDs 48–51 only (parameter/value modification); switching commands 45–47 do not emit T0836.
3. **Count-independent**: each finding is emitted once per ASDU frame regardless of object count.
4. **Passive-only**: the analyzer does NOT block the control command; it only records findings.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=45 (C_SC, single-point switching command) | T1692.001 Possible only (no T0836 — switching, not parameter write) |
| EC-002 | TypeID=51 (C_BO_NA_1, bitstring control) | T1692.001 Possible + T0836 Possible (bitstring = parameter write) |
| EC-003 | TypeID=44 (just below range) | No finding from this BC; see BC-2.19.022 (reserved TypeID) |
| EC-004 | TypeID=52 (RESERVED — not a valid control TypeID) | No finding from this BC; reserved per IEC 60870-5-104 |
| EC-005 | Control TypeID with COT=7 (activation-confirm) | Findings still emitted — the confirm is as interesting as the act |
| EC-006 | TypeID=48 (C_SE_NA_1, set-point normalized) | T1692.001 Possible + T0836 Possible |
| EC-007 | TypeID=47 (C_RC_NA_1, regulating step) | T1692.001 Possible only (step command, not parameter write) |

## Canonical Test Vectors

| TypeID | CASDU | Expected findings |
|--------|-------|-------------------|
| 45 | 1 | T1692.001 Possible only |
| 46 | 1 | T1692.001 Possible only |
| 47 | 1 | T1692.001 Possible only |
| 48 | 1 | T1692.001 Possible + T0836 Possible |
| 51 | 100 | T1692.001 Possible + T0836 Possible |
| 44 | 1 | (none from this BC) |
| 52 | 1 | (none from this BC — reserved TypeID) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on any TypeID 45–51 ASDU | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — control-command detection (TypeIDs 45–51) is the primary ICS threat-detection function of the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | T1692.001 "Unauthorized Message: Command Message" (ICS) — Possible for TypeIDs 45–51; T0836 "Modify Parameter" (ICS) — Possible for TypeIDs 48–51 only |

## Related BCs

- BC-2.19.016 — depends on (TypeID extraction)
- BC-2.19.020 — composes with (TypeID 105: C_RP, different technique)
- BC-2.19.021 — composes with (interrogation commands: no control finding)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if (45..=51).contains(&asdu.type_id) { emit T1692_001(Possible); if (48..=51).contains(&asdu.type_id) { emit T0836(Possible); } }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
