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

# BC-2.19.018: CASDU and First IOA Extraction from ASDU Bytes 4–8

## Description

`parse_asdu` extracts the Common Address of ASDU (CASDU) from `asdu_body[4..6]`
(2-byte little-endian, structure address 1–65535) and, if count > 0, the first
Information Object Address (IOA) from `asdu_body[6..9]` (3-byte little-endian, 24-bit
IOA 0–16777215). The CASDU identifies the Remote Terminal Unit (RTU) or Intelligent
Electronic Device (IED). The IOA identifies the individual data point on that device.
These fields are recorded as the target address context in findings.

## Preconditions

1. `asdu_body.len() >= 6` (per BC-2.19.015).
2. TypeID, VSQ, COT already extracted (per BC-2.19.016, BC-2.19.017).
3. For IOA extraction: `asdu.count > 0` AND `asdu_body.len() >= 9`.

## Postconditions

1. `asdu.casdu = u16::from_le_bytes([asdu_body[4], asdu_body[5]])` (CASDU, 16-bit LE).
2. If `count > 0` and `asdu_body.len() >= 9`:
   `asdu.first_ioa = u32::from_le_bytes([asdu_body[6], asdu_body[7], asdu_body[8], 0])` (IOA, 24-bit LE, zero-extended to u32).
3. If `count == 0` or `asdu_body.len() < 9`: `first_ioa = None`.

## Invariants

1. **CASDU=0 is undefined** per IEC 60870-5-104; it is extracted without rejection (anomaly flagging is out of MVP scope).
2. **IOA 3-byte LE**: 24-bit little-endian; maximum value = 0xFFFFFF = 16777215.
3. **First-IOA only**: for MVP, only the first IOA is extracted; multi-object iteration is a future enhancement.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | count=0 | No IOA extraction; first_ioa=None |
| EC-002 | ASDU body exactly 6 bytes, count=1 | IOA would need bytes 6–8 but body is only 6 bytes → first_ioa=None (truncated) |
| EC-003 | CASDU=0 | Extracted as casdu=0; no rejection |
| EC-004 | CASDU=65535, IOA=0xFFFFFF | Maximum values extracted correctly |

## Canonical Test Vectors

| asdu_body[4..6] (CASDU) | asdu_body[6..9] (IOA) | Expected CASDU | Expected first_ioa |
|------------------------|----------------------|----------------|--------------------|
| `[0x01, 0x00]` | `[0x01, 0x00, 0x00]` | 1 | Some(1) |
| `[0xFF, 0xFF]` | `[0xFF, 0xFF, 0xFF]` | 65535 | Some(16777215) |
| `[0x00, 0x00]` | (count=0) | 0 | None |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in CASDU/IOA extraction for any asdu_body length | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — CASDU and IOA extraction provide the device-address context required for actionable IEC-104 findings |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure extraction) |

## Related BCs

- BC-2.19.017 — composes with (COT extraction from bytes 2–3)
- BC-2.19.019 — depends on (TypeID + CASDU/IOA used in control-command findings)

## Architecture Anchors

- `src/analyzer/iec104.rs` — CASDU: `u16::from_le_bytes([body[4], body[5]])` IOA: `u32::from_le_bytes([body[6], body[7], body[8], 0])`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
