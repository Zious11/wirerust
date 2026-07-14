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
  - "v1.1: F-P3-H1 — VP-044 over-scope: parse_asdu is covered by VP-047 fuzz, not VP-044 Kani per ADR-013 Decision 8. 2026-07-14"
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

# BC-2.19.015: ASDU Minimum-Length Guard Rejects I-Frame ASDU Body Shorter Than 6 Bytes

## Description

After an I-format APCI frame is parsed and the ASDU body is extracted (LEN - 4 bytes,
since 4 bytes are the CF1..CF4 control octets), `parse_asdu` verifies that the ASDU body
is at least 6 bytes: TypeID (1B) + VSQ (1B) + COT (2B) + CASDU (2B) = 6-byte minimum
header before any Information Object. If the ASDU body is shorter, the function returns
`None` and the caller emits T0814. A 4-byte LEN I-frame has a 0-byte ASDU, which also
fails this check.

## Preconditions

1. Frame format is I-format (`classify_frame_format` returned `IFormat`).
2. ASDU body = `&apci_data[4..]` where `apci_data.len() == header.len as usize`.
3. `asdu_body.len() < 6`.

## Postconditions

1. `parse_asdu(asdu_body)` returns `None`.
2. Caller emits T0814 finding (Anomaly/Possible).
3. No TypeID, VSQ, COT, CASDU, or IOA fields are accessed.

## Invariants

1. **ASDU header minimum**: TypeID(1) + VSQ(1) + COT(2) + CASDU(2) = 6 bytes; any shorter ASDU body cannot contain even a headerless ASDU.
2. **Purity**: `parse_asdu` is a pure-core function — no side effects, no findings emitted internally.
3. **I-frame ASDU gate**: this guard applies only to I-frames (S/U frames have no ASDU body).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | LEN=4 I-frame (0-byte ASDU body) | None + T0814 |
| EC-002 | LEN=9 I-frame (5-byte ASDU body) | None + T0814 (one byte short) |
| EC-003 | LEN=10 I-frame (6-byte ASDU body, no IOA) | Some(Asdu{...}) — minimum valid, no IO |
| EC-004 | LEN=13 I-frame (9-byte ASDU body, one 3-byte IOA) | Some(Asdu{...}) |

## Canonical Test Vectors

| ASDU body len | Expected | Notes |
|---------------|----------|-------|
| 0 | None + T0814 | LEN=4 I-frame |
| 5 | None + T0814 | one byte short |
| 6 | Some(Asdu) | minimum valid |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | `parse_asdu` never panics for any input length; minimum-length guard returns `None` without accessing any ASDU fields; `parse_asdu` is not `parse_apci_header` and is outside VP-044 Kani scope per ADR-013 Decision 8 | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — ASDU minimum-length guard is required to safely parse IEC-104 data objects without OOB access |
| L2 Domain Invariants | INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 3 |
| Feature | feature-iec104 |
| MITRE Techniques | T0814 "Denial of Service" (on reject) |

## Related BCs

- BC-2.19.007 — depends on (I-format gate)
- BC-2.19.016 — depends on (TypeID extraction: requires len >= 6)
- BC-2.19.003/004 — composes with (APCI-level length guards)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn parse_asdu(body: &[u8]) -> Option<Asdu>`: `if body.len() < 6 { return None; }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 3`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (no-panic for all `parse_asdu` paths; parse_asdu is outside VP-044 Kani scope per ADR-013 Decision 8)
