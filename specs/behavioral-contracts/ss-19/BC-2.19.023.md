---
document_type: behavioral-contract
level: L3
version: "1.2"
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
  - "v1.1: F-P2-H1 remediation — VP-044 over-scope: extract_ns/extract_nr are not parse_apci_header; Invariant 2 re-anchored to VP-047 (cargo-fuzz); VP-044 Verification Properties row and VP Anchor removed. 2026-07-13"
  - "v1.2: D-438 follow-on (human-mandated) — Postcondition 3 updated: last_ns_c2s/last_ns_s2c type is now Option<u16> (SS-19 shard v1.6, ADR-013 Decision 6). State transition is None → Some(ns) on first I-frame, Some(prev) → Some(ns) on subsequent frames. Extraction arithmetic is unchanged. EC-003 unescaped-pipe pre-existing table bug fixed. 2026-07-14"
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

# BC-2.19.023: N(S)/N(R) 15-Bit Sequence Numbers Extracted Correctly from I/S Frame CF1–CF4

## Description

I-format and S-format APCI frames include 15-bit send sequence counter N(S) and 15-bit
receive sequence counter N(R) encoded in the four control-field octets CF1–CF4 using
little-endian bit layout. N(S) occupies bits 14:0 of the 16-bit value formed by CF1 (MS
byte, high 7 bits as CF2>>0) and CF2; N(R) occupies the same layout in CF3–CF4.
The extraction formulas are:
  `N(S) = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)` (I-frame only)
  `N(R) = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)` (I- and S-frame)
N(S) is tracked per-direction in `Iec104FlowState::last_ns_c2s` (C2S) and `last_ns_s2c`
(S2C) as `Option<u16>` (initialized to `None`; transitions to `Some(ns)` on first I-frame,
then `Some(prev)` → `Some(ns)` on subsequent frames) and is used in BC-2.19.024 to detect
sequence-number desynchronization attacks. N(R) is extracted from I/S-format frames but is
not separately stored in Iec104FlowState.

## Preconditions

1. Frame is I-format (CF1 bit 0 = 0) or S-format (CF1 bits1:0 = 0b01).
2. APCI header has been fully parsed with CF1–CF4 available.

## Postconditions

1. For I-format: `ns = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7)` — range [0, 32767].
2. For I/S-format: `nr = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7)` — range [0, 32767].
3. `Iec104FlowState::last_ns_c2s` (C2S direction) or `last_ns_s2c` (S2C direction) — type `Option<u16>` — transitions `None → Some(ns)` on the first observed I-frame in that direction, and `Some(prev) → Some(ns)` on all subsequent I-frames, selected by the `direction` parameter. The extraction arithmetic (`((cf1 as u16) >> 1) | ((cf2 as u16) << 7)`) is unchanged.
4. N(R) (`nr`) is computed and available transiently but is not stored — `Iec104FlowState` has no `last_nr` field.

## Invariants

1. **15-bit range**: N(S) and N(R) are in [0, 32767] — modular arithmetic wraps at 32768.
2. **No overflow**: the 15-bit extraction arithmetic cannot overflow (all intermediate values are u16 within bounds) — verified by VP-047 (cargo-fuzz: `fuzz_iec104_parser` no-panic harness covers extraction paths; `extract_ns`/`extract_nr` are not `parse_apci_header` and are outside VP-044 Kani scope).
3. **Wrap-around handling**: sequence number 32767 followed by 0 is valid wrapping behavior.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CF1=0x02, CF2=0x00 | N(S)=1 |
| EC-002 | CF1=0x00, CF2=0x80 | N(S)=0x4000 = 16384 |
| EC-003 | CF1=0xFE, CF2=0xFF | N(S) = (0xFE >> 1) \| (0xFF << 7) = 0x7F \| 0x7F80 = 0x7FFF = 32767 |
| EC-004 | Wrap: previous N(S)=32767, current N(S)=0 | valid wrap; no anomaly |

## Canonical Test Vectors

| CF1 | CF2 | CF3 | CF4 | Expected N(S) | Expected N(R) |
|-----|-----|-----|-----|--------------|--------------|
| `0x02` | `0x00` | `0x02` | `0x00` | 1 | 1 |
| `0x00` | `0x00` | `0x02` | `0x00` | 0 | 1 |
| `0xFE` | `0xFF` | `0xFE` | `0xFF` | 32767 | 32767 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in sequence number extraction; N(S)/N(R) arithmetic produces no overflow for any u8 CF1–CF4 input | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — accurate N(S)/N(R) extraction is required for the sequence-number desynchronization detection capability |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 6 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — pure extraction; T1692.001 emitted by BC-2.19.024 on gap detection) |

## Related BCs

- BC-2.19.007 — depends on (I-format gate)
- BC-2.19.008 — depends on (S-format gate)
- BC-2.19.024 — depends on (gap > k=12 → T1692.001 desync finding)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `let ns: u16 = ((cf1 as u16) >> 1) | ((cf2 as u16) << 7);`
- `src/analyzer/iec104.rs` — `let nr: u16 = ((cf3 as u16) >> 1) | ((cf4 as u16) << 7);`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 6`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (no-panic for all N(S)/N(R) extraction paths)
