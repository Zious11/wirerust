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
  - "v1.1: F-P2-L1 — Invariant 1 reserved-TypeID upper bound reconciled: '128–135 undefined/reserved' corrected to '128–255 undefined/reserved/private-use' to match BC-2.19.022 precondition (type_id >= 128 triggers T0814) and ADR-013. 2026-07-13"
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

# BC-2.19.016: TypeID and VSQ Extraction from ASDU Bytes 0–1

## Description

`parse_asdu` extracts the Type Identification (TypeID, 1 byte) from `asdu_body[0]` and
the Variable Structure Qualifier (VSQ, 1 byte) from `asdu_body[1]`. The TypeID identifies
the type of information object (single-point, double-point, control, etc.). The VSQ encodes
the SQ bit (sequence qualifier, bit 7) and the count of information objects (bits 6:0, range
0–127). Both fields are required for any further ASDU processing. This is a pure extraction
contract with no side effects; callers use these fields to drive TypeID-specific finding
emission (BC-2.19.019..022).

## Preconditions

1. `asdu_body.len() >= 6` (minimum-length guard passed per BC-2.19.015).
2. `asdu_body[0]` is the TypeID byte.
3. `asdu_body[1]` is the VSQ byte.

## Postconditions

1. `asdu.type_id = asdu_body[0]` (u8, any value 1–255; 0 is undefined per spec).
2. `asdu.sq = (asdu_body[1] & 0x80) != 0` (SQ bit: sequence qualifier).
3. `asdu.count = asdu_body[1] & 0x7F` (number of information objects, 0–127).
4. If `asdu.count == 0`, no Information Objects are present (valid but unusual).

## Invariants

1. **TypeID range**: TypeID 0 is undefined; TypeIDs 128–255 are undefined/reserved/private-use per IEC 60870-5-101/104. TypeIDs in range 128–255 (and TypeID 0) trigger T0814 via BC-2.19.022 (reserved-TypeID finding path). TypeIDs in the defined range [1, 127] that are not explicitly handled are extracted without rejection (BC-2.19.022 handles only 0 and 128–255).
2. **VSQ SQ bit**: SQ=1 means information objects are in a contiguous sequence (single IOA with multiple elements); SQ=0 means each element has its own IOA.
3. **Count bound**: count is 0–127 (7 bits). A count that would require more bytes than the ASDU body allows must be caught by the IOA iterator, not here.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | TypeID=45 (C_SC_NA_1) | Extracted; downstream BC-2.19.019 emits T1692.001 |
| EC-002 | TypeID=0 (undefined) | Extracted (type_id=0); BC-2.19.022 emits T0814 |
| EC-003 | VSQ=0x80 (SQ=1, count=0) | sq=true, count=0; no IOA iteration |
| EC-004 | VSQ=0x01 (SQ=0, count=1) | sq=false, count=1; one IOA expected |

## Canonical Test Vectors

| asdu_body[0] (TypeID) | asdu_body[1] (VSQ) | Expected type_id | Expected sq | Expected count |
|----------------------|--------------------|-----------------|------------|----------------|
| `0x01` | `0x01` | 1 | false | 1 |
| `0x2D` (45) | `0x01` | 45 | false | 1 |
| `0xFF` | `0x80` | 255 | true | 0 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in TypeID/VSQ extraction | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — TypeID and VSQ extraction is the entry point for all ASDU type-specific analysis and control-command detection |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure extraction; TypeID-driven findings in BC-2.19.019..022) |

## Related BCs

- BC-2.19.015 — depends on (ASDU minimum length guard)
- BC-2.19.017 — composes with (COT extraction from bytes 2–3)
- BC-2.19.019..022 — depends on (TypeID value drives finding emission)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `Asdu { type_id: body[0], sq: (body[1] & 0x80) != 0, count: body[1] & 0x7F, ... }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
