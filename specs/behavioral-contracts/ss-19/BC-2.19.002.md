---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.1: F-P2-M3 — MITRE Techniques tactic corrected: (IcsImpactIcs) is not a real tactic enum; changed to (IcsInhibitResponseFunction / TA0107) per src/mitre.rs technique_info T0814 arm. 2026-07-13"
  - "v1.2: F-P5-H1 — bad-start-byte carry semantics reconciled to ADR-013 Decision 3 step 3: carry is NOT discarded on bad start byte; caller advances 1 byte (resync scan to next 0x68 candidate). PC-3, EC-001, Inv-2, and Architecture Anchor updated. 2026-07-14"
  - "v1.3: D-470-B-002 — stale claim that frame-walk caller emits T0814 for bad start byte removed; superseded by BC-2.19.026 PC-4 (authoritative frame-walk BC), STORY-172 AC-172-004, and delivered code (Iec104Analyzer::on_data bad-start-byte arm advances pos by 1 with no finding). H1 title corrected from 'Returns None and Emits Anomaly When Start Byte ≠ 0x68' to 'Returns None for Start Byte ≠ 0x68'. Description, PC-2, PC-4, EC-001, EC-004, Canonical Test Vectors, MITRE Techniques, Architecture Anchor, BC-INDEX row, and PRD §2.19.A RTM row corrected. 2026-07-18"
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
input-hash: "0e684c9"
---

# BC-2.19.002: `parse_apci_header` Returns None for Start Byte ≠ 0x68

## Description

When a 6-or-more-byte input is provided but the first byte is not `0x68`, `parse_apci_header`
returns `None`. The start byte `0x68` is the IEC-104 frame-sync anchor defined in IEC
60870-5-104 §5.1. Any other first byte on a port-2404 flow indicates either framing corruption
or a misclassified flow. The frame-walk caller advances 1 byte to resync to the next 0x68
candidate; no finding is emitted for the bad-start-byte path (see BC-2.19.026 PC-4 and
ADR-013 Decision 3 step 3). This is the start-byte-reject path; the length-reject path is
BC-2.19.001.

## Preconditions

1. `data` is a `&[u8]` slice with `data.len() >= 6`.
2. `data[0] != 0x68` — first byte is not the IEC-104 start byte.
3. The flow was classified to `DispatchTarget::Iec104` via Rule 8 (port 2404).

## Postconditions

1. `parse_apci_header(data)` returns `None`.
2. The frame-walk caller does NOT emit a finding for the bad-start-byte path; no T0814 is emitted (see BC-2.19.026 PC-4 — the authoritative frame-walk BC).
3. The caller advances the frame-walk cursor 1 byte (resync scan to next 0x68 candidate) per ADR-013 Decision 3 step 3; the carry buffer is NOT discarded (discarding would lose a valid 0x68 later in the stream).
4. The function is pure; `parse_apci_header` itself does not emit findings, and the caller emits no finding for this path.

## Invariants

1. **Start-byte uniqueness**: `0x68` is the sole valid IEC-104 start byte per IEC 60870-5-104 §5.1.
2. **Advance-resync**: an invalid start byte causes the caller to advance 1 byte (resync scan to next 0x68 candidate) per ADR-013 Decision 3 step 3; the carry buffer is NOT discarded — discarding would lose a valid 0x68 at the next byte offset.
3. **Purity**: `parse_apci_header` itself does not emit findings; finding emission is the caller's responsibility.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `data[0] == 0x00`, rest valid | Returns `None`; no finding emitted; advances 1 byte (carry NOT cleared) |
| EC-002 | `data[0] == 0x68` (correct) | Proceeds to LEN validation; see BC-2.19.003/004/005 |
| EC-003 | `data[0] == 0x68` in an HTTP response body classified to port 2404 | Port-2404 traffic arriving here is already IEC-104-classified; `0x68` proceeds normally |
| EC-004 | All 255 non-0x68 first-byte values | All return `None`; no finding emitted; advance 1 byte each |

## Canonical Test Vectors

| Input (hex) | Expected result | Category |
|-------------|----------------|---------|
| `[0x00, 0x04, 0x07, 0x00, 0x00, 0x00]` | `None`; no finding; advance 1 byte | reject: bad start byte |
| `[0xFF, 0x04, 0x07, 0x00, 0x00, 0x00]` | `None`; no finding; advance 1 byte | reject: bad start byte |
| `[0x68, 0x04, 0x07, 0x00, 0x00, 0x00]` | `Some(ApciHeader{...})` | accept: correct start byte |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-044 | `parse_apci_header` never panics on any input; returns `None` for `data[0] != 0x68` when `len >= 6` | Kani: `verify_parse_apci_header_safety` |
| VP-047 | No panic on arbitrary byte input at `on_data` entry point | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — start-byte validation is the first field check in APCI parsing and a core part of the IEC-104 analysis capability |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 step 3 |
| Feature | feature-iec104 |
| MITRE Techniques | None — bad-start-byte path emits no finding; T0814 for malformed-LEN is covered by BC-2.19.026 (authoritative frame-walk BC) |

## Related BCs

- BC-2.19.001 — composes with (prior rejection: `data.len() < 6`)
- BC-2.19.003 — composes with (next: LEN < 4 rejection)
- BC-2.19.005 — composes with (accept path after all guards pass)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn parse_apci_header(data: &[u8]) -> Option<ApciHeader>`: `if data[0] != 0x68 { return None; }`
- `src/analyzer/iec104.rs` — `Iec104Analyzer::on_data` frame-walk loop: advances 1 byte (resync scan to next 0x68 candidate); NO finding emitted for bad-start-byte path; carry NOT cleared per ADR-013 Decision 3 step 3
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3` step 3

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

- VP-044 — `verify_parse_apci_header_safety`
- VP-047 — `fuzz_iec104_parser`
