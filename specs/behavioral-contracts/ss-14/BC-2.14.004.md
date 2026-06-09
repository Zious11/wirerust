---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-09T00:00:00Z
phase: 1a
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-14
capability: CAP-14
lifecycle_status: active
introduced: v0.3.0-feature-007
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/architecture-delta.md
  - .factory/research/modbus-tcp-research.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
input-hash: TBD
---

# BC-2.14.004: MBAP ADU Rejected When Length Is Outside [2, 254] (3-Point Gate: Length Check)

## Description

The `is_valid_modbus_adu(h: &MbapHeader) -> bool` function (applied by `on_data` after
`parse_mbap_header`) rejects any ADU whose `MbapHeader.length` is less than 2 or greater than
254. The Modbus.org spec V1.1b3 defines `Length` as the byte count of Unit ID + PDU: minimum
is 1 (Unit ID) + 1 (FC-only PDU) = 2; maximum is 1 (Unit ID) + 253 (maximum PDU: 1 FC byte +
252 data bytes) = 254. Note: the PDU max is 253 bytes (FC + data), so Length max = 1 + 253 =
254; maximum Modbus TCP ADU = 6 (prefix) + 254 = 260 bytes. Values outside [2, 254] indicate
either a malformed frame or a non-Modbus protocol on port 502.
Rejected ADUs increment `parse_errors`; no findings are emitted. This check is also the primary
security gate against oversized-ADU injection attacks (Broadcom IDS signature ASID 20676
flags frames exceeding 260 bytes — the 3-point Length gate fires earlier at the 254 Length
limit, which corresponds exactly to the 260-byte ADU maximum).

## Preconditions

1. `parse_mbap_header(data)` has returned `Some(h)` for an 8-byte-minimum input (BC-2.14.001).
2. `h.protocol_id == 0x0000` (the Protocol ID check in `is_valid_modbus_adu` passes — or
   fails; the gate returns `false` as soon as any sub-check fails). In practice this BC's
   behavior is observable whenever `h.length < 2` or `h.length > 254`, regardless of the
   Protocol ID check result.
3. `h.length` is outside the valid range: `h.length < 2` OR `h.length > 254`.

## Postconditions

1. `is_valid_modbus_adu(&h)` returns `false`.
2. `ModbusAnalyzer.parse_errors` is incremented by 1.
3. The `on_data` ADU parsing loop **breaks** immediately after incrementing `parse_errors`.
   The rest of the current segment is discarded (waiting for the next `on_data` call).
   This is the safe default that prevents attacker-controlled offset arithmetic.
4. The offset is NOT advanced by `6 + (h.length as usize)`. The `h.length` value is malformed
   (outside [2, 254]) and must not be used in arithmetic — advancing by it could skip past
   valid later PDUs (if length is too small) or cause an oversized jump (if length is huge).
5. `ModbusAnalyzer.total_pdu_count` is NOT incremented.
6. No `Finding` is emitted — a malformed Length is treated as parse corruption, not an attack
   signal.

**Order of operations** (unambiguous, testable `parse_errors` delta):
- `is_valid_modbus_adu` returns `false` for this ADU → `parse_errors += 1` → `break`.
- The `parse_errors` increment happens on the validity-gate failure path (not on truncation).
- Truncation (ADU boundary check firing, per BC-2.14.002) → `break` without `parse_errors++`.
- Validity-gate failure (this BC) → `parse_errors++` then `break`.
- These two paths are MUTUALLY EXCLUSIVE for any single ADU: if the frame is shorter than 8
  bytes, `parse_mbap_header` returns `None` (truncation, BC-2.14.002) and the validity gate
  is never reached.

## Invariants

1. **Valid Length range [2, 254]** (Modbus.org spec V1.1b3, §4.1 ADU constraints):
   - Minimum: `Length = 2` = Unit ID (1 byte) + minimum PDU of 1 byte (FC alone).
   - Maximum: `Length = 254` = Unit ID (1 byte) + maximum PDU of 253 bytes.
     PDU maximum = FC (1 byte) + data (252 bytes) = 253 bytes; Length = 1 + 253 = 254.
     (Earlier V1 used 249; V1.1b3 harmonized PDU max to 253 bytes per RS-485 constraint.)
   - Maximum Modbus TCP ADU byte count: `6 (MBAP prefix) + 254 = 260 bytes` — confirmed.
     (The 6-byte MBAP prefix = TxnID(2)+ProtoID(2)+Length(2); the 7th byte Unit ID is
     already counted inside `Length`. So ADU = 6 + Length = 6 + 254 = 260.)
2. **Length = 0 and Length = 1 are illegal**: `Length = 0` would mean no Unit ID and no PDU;
   `Length = 1` would mean Unit ID with no FC — neither is valid per the spec.
3. **Length = 255..=65535 are illegal**: the `u16` field can represent up to 65535 but any
   value > 254 is non-spec. The check `h.length <= 254` gates this.
4. **parse_errors semantics (this path only)**: this counter is incremented by 1 for every ADU
   that has a valid 8-byte-minimum header (parse_mbap_header returns Some) but fails the
   Length-range check. Protocol_id failures (BC-2.14.003) do NOT increment parse_errors
   (they set is_non_modbus and bail out). The two failure types are distinguishable by behavior:
   a protocol_id failure permanently silences the flow; a Length failure breaks the loop and
   waits for the next segment.
5. **Break loop (safe default)**: the `break` policy discards the rest of the current TCP
   segment but does NOT mark the flow as non-Modbus. If the next `on_data` call starts with a
   valid ADU, parsing resumes normally. This is appropriate for a valid Modbus stream that
   contains one malformed ADU (e.g., a network glitch), unlike the protocol_id case where
   the entire stream is non-Modbus.
6. **No attacker-controlled advance**: the malformed `h.length` value (< 2 or > 254) is never
   used to advance the offset. The `break` is the security-relevant escape path per
   architecture-delta.md §2.4 Decision 6.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `length = 0` | `is_valid_modbus_adu` returns `false` (0 < 2); `parse_errors++`; no crash |
| EC-002 | `length = 1` | `false` (1 < 2); `parse_errors++` |
| EC-003 | `length = 2` (minimum valid) | `true` (at the lower bound); ADU is processed normally — NOT rejected by Length gate |
| EC-004 | `length = 254` (maximum valid) | `true` (at the upper bound); ADU is processed normally — NOT rejected by Length gate |
| EC-005 | `length = 255` (first invalid above max) | `false` (255 > 254); `parse_errors++`; no finding |
| EC-006 | `length = 65535` (u16::MAX) | `false` (> 254); `parse_errors++`; loop `break`s; segment discarded; NO `6+65535` arithmetic |
| EC-007 | `length = 6` with `data.len() = 12` (exact fit) | `true` (length in range); adu_size = 12; `offset + 12 <= data.len()` so loop continues; ADU processed |
| EC-008 | `length = 255` (first invalid above max) with a valid ADU following in the same segment | `parse_errors++`; `break`; segment discarded; valid following ADU is NOT parsed in this call (it will be delivered again on the next `on_data` or is lost); the break-loop policy favors security over throughput |

## Canonical Test Vectors

| Input (hex, first 8 bytes) | `h.length` | `is_valid_modbus_adu` | `parse_errors` delta | Category |
|----------------------------|-----------|----------------------|---------------------|----------|
| `00 01 00 00 00 00 01 03` (len=0) | 0 | `false` | +1 | edge-case: zero length |
| `00 01 00 00 00 01 01 03` (len=1) | 1 | `false` | +1 | edge-case: below minimum |
| `00 01 00 00 00 02 01 03` (len=2) | 2 | `true` | +0 | regression: minimum valid NOT rejected |
| `00 01 00 00 00 FE 01 03` (len=254, 0xFE) | 254 | `true` | +0 | regression: maximum valid NOT rejected |
| `00 01 00 00 00 FF 01 03` (len=255, 0xFF) | 255 | `false` | +1 | edge-case: first above maximum |
| `00 01 00 00 FF FF 01 03` (len=65535, 0xFFFF) | 65535 | `false` | +1 | edge-case: u16::MAX — no OOB |

**Annotated invalid-length vector** (length = 0):
```
Bytes:  00 01  |  00 00  |  00 00  |  01  |  03
Field:  TxnID  |  ProtoID|  Length |  UID |  FC
Value:  0x0001 |  0x0000 |    0    |   1  | 0x03
Gate:   —      |  PASS   |  REJECT |  —   |  —
```

**Annotated valid-length vector** (length = 6, standard Read Holding Registers):
```
Bytes:  00 01  |  00 00  |  00 06  |  01  |  03  | 00 00 00 0A
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  | addr(2) qty(2)
Value:  0x0001 |  0x0000 |    6    |   1  | 0x03 | —
Gate:   —      |  PASS   |  PASS   |  —   |  —
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | `is_valid_modbus_adu` is a pure predicate; no-panic over all `MbapHeader.length` values (u16::MIN..=u16::MAX) | Kani: symbolic `h.length: u16 = kani::any()` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC specifies the Length validity gate that enforces spec-compliant ADU sizing and prevents oversized-ADU injection from producing false ICS findings |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `is_valid_modbus_adu`); ADR-005 §2.4 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.001 — depends on (parse_mbap_header must return Some(h) before this gate applies)
- BC-2.14.002 — sibling (truncation reject — distinct: short input returns None before gate)
- BC-2.14.003 — sibling (Protocol ID gate — same function, different sub-check)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `is_valid_modbus_adu`: `h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254`
- `src/analyzer/modbus.rs` — on_data loop: Length-range failure path: `self.parse_errors += 1; break;` — no offset advance
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.4` — Desync / DoS safety policy: "Length-range failure: increment parse_errors and break the parsing loop. Do not advance by the malformed Length value."

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — pure predicate `is_valid_modbus_adu` coverage

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.4; modbus-tcp-research.md §1 (Length range [2,253], spec V1.1b3 ADU bounds); modbus-tcp-research.md §1 point 1 (Broadcom ASID 20676 flags >260-byte frames) |
| **Confidence** | high — [2, 254] range is spec-defined; PDU max = 253 bytes (FC+data), Length = UnitID(1)+PDU(253) = 254; V1.1b3 harmonized PDU max from 249 to 253 bytes |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
