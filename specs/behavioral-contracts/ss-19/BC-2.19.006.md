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
  - "v1.1: F-P3-H1 — VP-044 over-scope: is_valid_iec104_frame is not parse_apci_header; re-anchored to VP-047 per ADR-013 Decision 8. 2026-07-14"
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

# BC-2.19.006: `is_valid_iec104_frame` Post-Classification Validity Gate

## Description

`is_valid_iec104_frame(data: &[u8]) -> bool` is a post-classification guard that verifies
the first byte of a port-2404-classified flow is `0x68` and that the second byte (LEN) is
in [4, 253]. It is called on the raw data before `parse_apci_header` is invoked, providing
a lightweight validity gate that compensates for false-positive port-2404 classification
without polluting the `classify()` rule table with a single-byte content signature
(ADR-013 Decision 1). If this gate fails, the data is not an IEC-104 APCI frame.

## Preconditions

1. `data.len() >= 2` (minimum: start byte + LEN byte visible).
2. The flow was dispatched to SS-19 via Rule 8 (port 2404).

## Postconditions

1. Returns `true` iff `data[0] == 0x68` AND `4 <= data[1] <= 253`.
2. Returns `false` for any other first two bytes; no side effects.
3. Caller emits an anomaly finding and discards data if `false` on a non-empty buffer.

## Invariants

1. **Gate scope**: validates only bytes 0 and 1; does not fully parse the APCI header.
2. **Consistency with parse_apci_header**: any input where `is_valid_iec104_frame` returns `true`
   and `data.len() >= 6` will cause `parse_apci_header` to succeed (return `Some`).
3. **False-positive correction**: this gate catches non-IEC-104 TCP flows on port 2404 at
   zero-cost (2-byte check) without adding a primary content-signature rule.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data[0] == 0x68`, `data[1] == 4` | Returns `true` |
| EC-002 | `data[0] == 0x68`, `data[1] == 253` | Returns `true` |
| EC-003 | `data[0] != 0x68` | Returns `false` |
| EC-004 | `data[0] == 0x68`, `data[1] == 3` | Returns `false` (LEN < 4) |
| EC-005 | `data[0] == 0x68`, `data[1] == 254` | Returns `false` (LEN > 253) |
| EC-006 | `data.len() == 1` | Returns `false` (cannot read LEN) |

## Canonical Test Vectors

| Input | Expected | Category |
|-------|----------|---------|
| `[0x68, 0x04, ...]` | `true` | valid IEC-104 frame start |
| `[0x48, 0x04, ...]` | `false` | wrong start byte |
| `[0x68, 0xFF, ...]` | `false` | LEN out of range |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | `is_valid_iec104_frame` never panics for any input; returns `true` iff `data[0] == 0x68` and `4 <= data[1] <= 253`; `is_valid_iec104_frame` is not `parse_apci_header` and is outside VP-044 Kani scope per ADR-013 Decision 8 | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — the validity gate is the first guard applied to port-2404 traffic and a core part of the IEC-104 analysis capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 1 (post-classification gate) |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure gate function) |

## Related BCs

- BC-2.19.001..005 — composes with (is_valid_iec104_frame summarizes the same two-byte checks as parse_apci_header's first two guards)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn is_valid_iec104_frame(data: &[u8]) -> bool`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 1` — validity gate rationale

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (no-panic for all `is_valid_iec104_frame` paths; gate is outside VP-044 Kani scope per ADR-013 Decision 8)
