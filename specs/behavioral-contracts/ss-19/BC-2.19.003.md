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

# BC-2.19.003: `parse_apci_header` Rejects LEN < 4 with T0814 Finding (Malformed Length)

## Description

When start byte is `0x68` and `data.len() >= 6` but LEN octet (byte 1) is less than 4,
`parse_apci_header` returns `None`. LEN counts all bytes after the LEN octet; the minimum
valid value is 4 (the 4 control-field octets in an S- or U-frame with no ASDU). A LEN of
0, 1, 2, or 3 is structurally impossible per IEC 60870-5-104 §5.1 and indicates a malformed
or adversarially crafted frame. The frame-walk caller emits a T0814 "Denial of Service"
finding and advances the cursor past the 2-byte APCI stub.

## Preconditions

1. `data.len() >= 6`, `data[0] == 0x68`.
2. `data[1] < 4` (LEN octet is 0, 1, 2, or 3).

## Postconditions

1. `parse_apci_header(data)` returns `None`.
2. The caller emits T0814 finding (Anomaly/Possible); advances cursor by 2 bytes (start byte + LEN).
3. The carry buffer is NOT stashed — the 2-byte stub is consumed and the walk continues.

## Invariants

1. **Minimum LEN = 4**: the control-field octets (CF1–CF4) always occupy at least 4 bytes per the IEC-104 spec.
2. **Fail-closed**: malformed LEN → T0814 emitted, stub discarded. The analyzer never accumulates bytes after a malformed header.
3. **Cursor advance**: advancing by 2 (not 1) prevents single-byte re-scan loops on adversarial byte streams.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `LEN == 0` | `None` + T0814; advance 2 |
| EC-002 | `LEN == 3` (off-by-one boundary) | `None` + T0814; advance 2 |
| EC-003 | `LEN == 4` (boundary: minimum valid) | Proceeds to LEN > 253 check; see BC-2.19.004/005 |
| EC-004 | Multiple consecutive bad-LEN frames | Each emits one T0814; cursor advances on each |

## Canonical Test Vectors

| Input (hex) | Expected | Category |
|-------------|----------|---------|
| `[0x68, 0x00, ...]` (LEN=0) | `None` + T0814 finding | reject: LEN too small |
| `[0x68, 0x03, ...]` (LEN=3) | `None` + T0814 finding | reject: LEN too small |
| `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` (LEN=4) | `Some(ApciHeader{...})` | accept: LEN valid |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-044 | `parse_apci_header` returns `None` for `LEN < 4` and never panics for any symbolic input | Kani: `verify_parse_apci_header_safety` |
| VP-047 | No panic via `on_data` fuzz harness | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — LEN-field validation is essential to the APCI frame-walk loop correctness and is a core part of IEC-104 passive analysis |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 step 4 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" (caller emits on malformed LEN) |

## Related BCs

- BC-2.19.002 — composes with (prior: start byte ≠ 0x68)
- BC-2.19.004 — composes with (sibling: LEN > 253)
- BC-2.19.005 — composes with (accept path)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn parse_apci_header`: `if data[1] < 4 { return None; }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` step 4

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-044 — `verify_parse_apci_header_safety`
- VP-047 — `fuzz_iec104_parser`
