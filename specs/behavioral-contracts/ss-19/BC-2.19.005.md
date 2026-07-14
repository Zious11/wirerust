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
input-hash: "a153144"
---

# BC-2.19.005: `parse_apci_header` Returns Some(ApciHeader) for Valid 6-byte APCI Input (Happy Path)

## Description

When `data.len() >= 6`, `data[0] == 0x68`, and `4 <= data[1] <= 253`, `parse_apci_header`
returns `Some(ApciHeader)` containing the parsed start byte, LEN, and four control-field
octets (CF1–CF4). This is the accept path. The caller then checks whether a complete APDU
is available (data.len() >= LEN + 2); if not, bytes are stashed in carry. The returned
ApciHeader is used by `classify_frame_format(cf1)` (BC-2.19.007..009) to determine I/S/U
format, and by the frame-walk loop to calculate the total APDU extent (LEN + 2 bytes).

## Preconditions

1. `data.len() >= 6`.
2. `data[0] == 0x68` (valid start byte).
3. `4 <= data[1] <= 253` (valid LEN range).

## Postconditions

1. Returns `Some(ApciHeader { start: 0x68, len: data[1], cf1: data[2], cf2: data[3], cf3: data[4], cf4: data[5] })`.
2. Total APDU frame size = `data[1] as usize + 2` (in range [6, 255]).
3. No findings emitted; no state mutation.
4. If `data.len() < data[1] + 2`: caller stashes `data` in carry and returns.
5. If `data.len() >= data[1] + 2`: caller extracts the full APDU and passes CF1 to `classify_frame_format`.

## Invariants

1. **Result bounds**: `h.len + 2` is always in [6, 255] — proven by VP-044 (Kani no-overflow property B+C).
2. **No overflow**: `LEN as u8` fits in [4..=253]; `LEN as usize + 2` fits in [6..=255]; no integer overflow possible for any valid LEN.
3. **Purity**: the function reads exactly 6 bytes and returns; no state, no I/O.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `LEN == 4`, `data.len() == 6` (minimum valid, complete frame) | Returns `Some`; complete S- or U-frame |
| EC-002 | `LEN == 4`, `data.len() == 5` | BC-2.19.001 fires first (len < 6); not reachable here |
| EC-003 | `LEN == 253`, `data.len() == 255` (maximum valid, complete frame) | Returns `Some`; full 255-byte APDU |
| EC-004 | `LEN == 100`, `data.len() == 50` (partial: header present, body incomplete) | Returns `Some`; caller stashes to carry; APDU body arrives in next on_data call |
| EC-005 | Two complete APDUs back-to-back in one slice | First APDU processed, cursor advanced by LEN+2; second APDU processed in next iteration |

## Canonical Test Vectors

| Input (bytes) | Expected result | Category |
|---------------|----------------|---------|
| `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` (U-frame STARTDT-act) | `Some(ApciHeader{start:0x68,len:4,cf1:0x07,cf2:0,cf3:0,cf4:0})` | accept: U-frame minimum |
| `[0x68, 0x04, 0x01, 0x00, 0x00, 0x00]` (S-frame) | `Some(ApciHeader{start:0x68,len:4,cf1:0x01,...})` | accept: S-frame |
| `[0x68, 0x0E, 0x00, 0x00, 0x00, 0x00, ...]` (I-frame, LEN=14) | `Some(ApciHeader{start:0x68,len:14,cf1:0x00,...})` | accept: I-frame |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-044 | Property B: for any returned `Some(h)`, `h.len + 2` is in [6, 255] with no overflow; Property C: `h.len >= 4`, `h.len <= 253` | Kani: `verify_parse_apci_header_safety` |
| VP-047 | No panic on arbitrary fuzz input including valid APCI frames | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — the accept path of parse_apci_header is the entry point for all successful APCI frame processing in the IEC-104 analyzer |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decisions 3, 8 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — successful parse; findings emitted by downstream processing) |

## Related BCs

- BC-2.19.001 — composes with (precondition: len >= 6)
- BC-2.19.002 — composes with (precondition: start byte == 0x68)
- BC-2.19.003 — composes with (precondition: LEN >= 4)
- BC-2.19.004 — composes with (precondition: LEN <= 253)
- BC-2.19.006 — composes with (is_valid_iec104_frame post-classification gate)
- BC-2.19.007 — depends on (classify_frame_format uses cf1 from this result)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn parse_apci_header(data: &[u8]) -> Option<ApciHeader>`: returns `Some(ApciHeader{...})` after all guards pass
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` step 6
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 8` — VP-044 Kani property B: total frame in [6,255]

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-044 — `verify_parse_apci_header_safety` (Properties B+C: bounds on returned header)
- VP-047 — `fuzz_iec104_parser`
