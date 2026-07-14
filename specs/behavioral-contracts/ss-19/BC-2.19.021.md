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

# BC-2.19.021: Interrogation and Clock-Sync Commands (TypeIDs 100, 101, 103) Are Logged Without Findings

## Description

TypeIDs 100 (C_IC_NA_1 — General Interrogation), 101 (C_CI_NA_1 — Counter Interrogation),
and 103 (C_CS_NA_1 — Clock Synchronization) are common administrative SCADA commands that
occur in normal operations. The analyzer recognizes and logs these TypeIDs but does not
emit any security finding. They are explicitly excluded from the control-command finding
path (BC-2.19.019) because interrogation and clock-sync are benign operational messages.
Logging-only allows the analyst to see communication patterns without false-positive noise.

## Preconditions

1. A valid I-format ASDU has been parsed.
2. `asdu.type_id` is one of {100, 101, 103}.

## Postconditions

1. No finding is emitted.
2. ASDU is logged at trace level (for diagnostics) but not as a finding.
3. CASDU and COT are recorded in the trace log entry.

## Invariants

1. **No-op for findings**: these TypeIDs explicitly produce no findings.
2. **Coverage gap awareness**: TypeID 102 (C_RD_NA_1, Read Command) is not in this set; its handling is left as a future enhancement (emits no finding by default — falls through to the unrecognized-but-non-reserved path).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | C_IC_NA_1 (TypeID 100) | No finding; logged |
| EC-002 | C_CI_NA_1 (TypeID 101) | No finding; logged |
| EC-003 | C_CS_NA_1 (TypeID 103) | No finding; logged |
| EC-004 | TypeID 102 (C_RD_NA_1) | Falls through; no finding (not covered here) |

## Canonical Test Vectors

| TypeID | Expected finding |
|--------|-----------------|
| 100 | none |
| 101 | none |
| 103 | none |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic on TypeIDs 100, 101, 103 | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — explicit no-finding handling for interrogation and clock-sync commands is required to prevent false positives in the IEC-104 analyzer |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — benign administrative commands) |

## Related BCs

- BC-2.19.019 — composes with (control TypeIDs 45–51: emit findings)
- BC-2.19.022 — composes with (reserved/invalid TypeIDs: emit T0814)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if matches!(asdu.type_id, 100 | 101 | 103) { /* log, no finding */ }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
