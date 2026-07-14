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

# BC-2.19.017: COT Extraction (Cause of Transmission) from ASDU Bytes 2–3

## Description

`parse_asdu` extracts the Cause of Transmission (COT) from `asdu_body[2..4]` (2 bytes,
little-endian). The COT structure encodes: cause code (bits 5:0 of byte 2, 6-bit value
0–63), P/N bit (bit 6 of byte 2 — positive/negative confirmation), T bit (bit 7 of
byte 2 — test transmission), and originator address (byte 3, 0 = no originator). The
COT cause field drives triage: cause=6 (activation), cause=7 (activation-confirmation),
cause=10 (interrogation request), etc. Test-bit (T=1) frames are informational only.

## Preconditions

1. `asdu_body.len() >= 6` (per BC-2.19.015).
2. TypeID and VSQ already extracted from bytes 0–1 (per BC-2.19.016).

## Postconditions

1. `asdu.cot_cause = asdu_body[2] & 0x3F` (6-bit cause code, 0–63).
2. `asdu.cot_pn = (asdu_body[2] & 0x40) != 0` (P/N flag: positive/negative confirm).
3. `asdu.cot_test = (asdu_body[2] & 0x80) != 0` (T flag: test transmission).
4. `asdu.cot_originator = asdu_body[3]` (u8, originator address; 0 = absent).

## Invariants

1. **Test bit semantics**: T=1 indicates a test frame; findings may be suppressed or tagged `[TEST]` to reduce analyst noise.
2. **COT cause range**: valid cause codes are 1–63; cause=0 is undefined.
3. **Little-endian COT**: bytes 2 and 3 are ordered: byte 2 = cause+flags, byte 3 = originator.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | T=1 (test bit set) | Extract normally; tag finding context as test-transmission |
| EC-002 | Originator=0 | No originator address — valid, common |
| EC-003 | Cause=6 (activation) + P/N=0 | Positive activation request |
| EC-004 | Cause=0 | Extracted as cot_cause=0; no separate rejection |

## Canonical Test Vectors

| byte[2] | byte[3] | Expected cause | Expected P/N | Expected T | Expected originator |
|---------|---------|----------------|-------------|-----------|---------------------|
| `0x06` | `0x00` | 6 | false | false | 0 |
| `0xC6` | `0x01` | 6 | true | true | 1 |
| `0x3F` | `0xFF` | 63 | false | false | 255 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in COT extraction | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — COT extraction provides cause-of-transmission context required for accurate IEC-104 ASDU analysis |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure extraction) |

## Related BCs

- BC-2.19.016 — composes with (TypeID/VSQ extraction)
- BC-2.19.018 — composes with (CASDU and IOA extraction)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `asdu.cot_cause = body[2] & 0x3F; asdu.cot_test = (body[2] & 0x80) != 0; asdu.cot_originator = body[3];`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
