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
  - "v1.1: F-P2-M2 — Architecture Anchor pseudo-code corrected: emit T0827(Possible) → emit T0827(Likely); title/description/postcondition all say Likely, anchor contradicted them. 2026-07-13"
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

# BC-2.19.020: C_RP_NA_1 (TypeID 105) Emits T0827 "Loss of Control" Finding

## Description

When a parsed ASDU has TypeID=105 (C_RP_NA_1 — Reset Process Command), the analyzer
emits a T0827 "Loss of Control" (ICS) finding with confidence Likely. The Reset Process
command is used to return RTU/IED applications to a defined initial state. In an adversarial
context, a Reset Process command sent without authorization can cause controlled equipment
to revert to default states — a potential loss-of-control scenario. The analyzer records
this event to flag potential unauthorized reset attempts.

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id == 105`.

## Postconditions

1. T0827 "Loss of Control" finding emitted with confidence Likely.
2. Finding includes CASDU and first_ioa as target address context.

## Invariants

1. **Single technique**: only T0827 is emitted for TypeID 105 (not T1692.001 — reset is a session management command, not a parameter change).
2. **Count-independent**: finding emitted once per ASDU frame.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=105 with COT=6 (activation) | T0827 Likely |
| EC-002 | TypeID=105 with any COT | T0827 Likely regardless of COT |

## Canonical Test Vectors

| TypeID | Expected |
|--------|----------|
| 105 | T0827 Likely |
| 104 | (no finding from this BC) |
| 106 | (no finding from this BC) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on TypeID=105 ASDU | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — Reset Process (TypeID 105) detection maps to ICS Loss of Control and is a required IEC-104 threat indicator |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | T0827 "Loss of Control" (ICS) — Likely |

## Related BCs

- BC-2.19.019 — composes with (control TypeIDs 45–51)
- BC-2.19.021 — composes with (interrogation commands: benign)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if asdu.type_id == 105 { emit T0827(Likely); }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
