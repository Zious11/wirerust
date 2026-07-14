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
  - "v1.1: F-P5-M1 — VP attribution corrected: forward-progress/loop-termination routed to VP-047; VP-044 scope restated as LEN-bounds-only (parse_apci_header pure-core arithmetic). Description and Invariant 3 updated. 2026-07-14"
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

# BC-2.19.004: `parse_apci_header` Rejects LEN > 253 with T0814 Finding (Malformed Length)

## Description

When start byte is `0x68`, `data.len() >= 6`, and LEN ≥ 4, but LEN > 253, `parse_apci_header`
returns `None`. The IEC 60870-5-104 specification caps the maximum APDU at 255 bytes (LEN + 2
≤ 255 ⟹ LEN ≤ 253). A LEN of 254 or 255 exceeds the spec maximum, indicating a malformed or
adversarially crafted frame. The frame-walk caller emits a T0814 "Denial of Service" finding
and advances the cursor by 2 bytes. Together with BC-2.19.003, these two contracts define the
complete LEN bounds check that provides forward-progress guarantees: loop termination proven by VP-047; the LEN bounds themselves proven by VP-044.

## Preconditions

1. `data.len() >= 6`, `data[0] == 0x68`, `data[1] >= 4`.
2. `data[1] > 253` (LEN is 254 or 255).

## Postconditions

1. `parse_apci_header(data)` returns `None`.
2. Caller emits T0814 finding (Anomaly/Possible); advances cursor by 2 bytes.
3. Carry buffer is not stashed for this iteration.

## Invariants

1. **Maximum LEN = 253**: from IEC 60870-5-104 §5.1 (APDU ≤ 255 bytes; LEN excludes start byte + LEN octet itself).
2. **Combined LEN bounds**: `4 ≤ LEN ≤ 253` is the complete valid range (BC-2.19.003 + this contract).
3. **Loop termination (VP-047)**: the frame-walk loop advances 2 bytes on the malformed-LEN path (2-byte APCI stub advance, ADR-013 Decision 3 step 4).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `LEN == 254` | `None` + T0814; advance 2 |
| EC-002 | `LEN == 255` | `None` + T0814; advance 2 |
| EC-003 | `LEN == 253` (maximum valid) | Proceeds to complete-frame check; see BC-2.19.005 |
| EC-004 | `LEN == 4` (minimum valid) | Proceeds to complete-frame check (no ASDU body); see BC-2.19.005 |

## Canonical Test Vectors

| Input (hex) | Expected | Category |
|-------------|----------|---------|
| `[0x68, 0xFE, ...]` (LEN=254) | `None` + T0814 | reject: LEN too large |
| `[0x68, 0xFF, ...]` (LEN=255) | `None` + T0814 | reject: LEN too large |
| `[0x68, 0xFD, 0x01, 0x00, 0x00, 0x00]` (LEN=253, full data) | `Some(ApciHeader{len:253,...})` if full frame available | accept: max LEN |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-044 | Property C: for any returned `Some(h)`, `h.len ≤ 253` and `h.len + 2 ≤ 255` with no integer overflow | Kani: `verify_parse_apci_header_safety` |
| VP-047 | No panic via fuzz harness | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — upper LEN bound is a core APCI validity constraint for the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 step 4 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" (caller emits on malformed LEN) |

## Related BCs

- BC-2.19.003 — composes with (LEN < 4 lower-bound check)
- BC-2.19.005 — composes with (accept path: 4 ≤ LEN ≤ 253)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn parse_apci_header`: `if data[1] > 253 { return None; }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` step 4
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 8` — VP-044 Kani property C: `h.len <= 253`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-044 — `verify_parse_apci_header_safety` (Property C: LEN ≤ 253, total frame ≤ 255)
- VP-047 — `fuzz_iec104_parser`
