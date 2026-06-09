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

# BC-2.14.003: MBAP ADU Rejected When Protocol ID Is Not 0x0000 (3-Point Gate: Protocol Check — Bail-Out Policy)

## Description

The `is_valid_modbus_adu(h: &MbapHeader) -> bool` function — applied by `on_data` immediately
after `parse_mbap_header` returns `Some(_)` — rejects any ADU whose `MbapHeader.protocol_id`
is not `0x0000`. The Modbus.org spec V1.1b3 mandates that the Protocol Identifier field is
ALWAYS `0x0000` for Modbus; a non-zero value indicates non-Modbus binary traffic inadvertently
landing on TCP port 502 (e.g., a different protocol sharing the port). On a protocol_id failure,
`on_data` sets `flow.is_non_modbus = true` on the `ModbusFlowState` and returns immediately —
the entire flow is permanently marked as non-Modbus and all subsequent `on_data` calls for
that flow key bail out at entry without parsing. No offset advance by the attacker-controlled
`6 + h.length` occurs. No `parse_errors` increment occurs for the initial protocol_id failure
(the flow is silently treated as non-Modbus, not as a Modbus parse error). This is the
**desync bail-out policy** per architecture-delta.md §2.4 Decision 6.

## Preconditions

1. `parse_mbap_header(data)` has already returned `Some(h)` for an 8-byte-minimum input (BC-2.14.001).
2. `h.protocol_id != 0x0000` (bytes 2–3 of the ADU are any value other than `[0x00, 0x00]`).
3. `h.length` may be any value (valid or invalid); the Protocol ID check fires before the
   Length check in `is_valid_modbus_adu` (the AND expression short-circuits, but in practice
   both checks are evaluated via `&&` — order within the boolean expression is implementation-
   defined; the observable result is rejection regardless of which sub-check fires first).

## Postconditions

1. `is_valid_modbus_adu(&h)` returns `false`.
2. `flow.is_non_modbus` is set to `true` on the `ModbusFlowState` for this flow key.
3. `on_data` returns immediately after setting `is_non_modbus = true`. No further parsing
   occurs for this call.
4. All subsequent `on_data` calls for this flow key bail out at entry (first statement):
   `if flow.is_non_modbus { return; }` — no ADU parsing, no `classify_fc` call, no pending-table
   insertion, no finding emission.
5. `ModbusAnalyzer.parse_errors` is NOT incremented for the protocol_id failure. The flow is
   silently treated as non-Modbus traffic, not as a Modbus parse error.
6. `ModbusAnalyzer.total_pdu_count` is NOT incremented.
7. The offset is NOT advanced by `6 + h.length` (attacker-controlled value). The function
   returns before offset arithmetic, preventing DoS via crafted Length fields.
8. No `Finding` is emitted — non-Modbus binary on port 502 is not an attack signal.

## Invariants

1. **Protocol ID == 0x0000 is mandatory** (Modbus.org spec V1.1b3, Messaging Guide V1.0b §3.1.3):
   "The Protocol Identifier is used for intra-system multiplexing. The value 0 is used for
   Modbus protocol." Any non-zero value is architecturally non-Modbus.
2. **Bail-out policy (desync safety — Decision 6)**: a protocol_id failure permanently marks the
   flow as non-Modbus (`is_non_modbus = true`). This handles non-Modbus binary protocols
   (e.g., ENIP, DNP3, proprietary) misrouted to port 502 without DoS from a crafted Length field.
   The claim "resync works after a malformed ADU" is WITHDRAWN for protocol_id failures — resync
   is only meaningful within a valid Modbus binary stream; non-Modbus traffic is not recoverable.
3. **No offset advance on protocol_id failure**: unlike the Length-range failure path (BC-2.14.004),
   a protocol_id failure does NOT advance the offset by `6 + h.length`. Advancing by an
   attacker-controlled `h.length` could cause out-of-bounds access or CPU-exhaustion via huge
   crafted Length values. The bail-out return prevents any such arithmetic.
4. **parse_errors is NOT incremented for this path**: protocol_id failures are a non-Modbus
   flow indicator, not a Modbus parse error. The `parse_errors` counter counts ADUs that are
   structurally Modbus but invalid (Length range errors, per BC-2.14.004). A non-Modbus flow
   is a different category.
5. **`is_non_modbus` field on `ModbusFlowState`**: this boolean field (added per Decision 6 in
   architecture-delta.md §2.3) serves as the permanent non-Modbus sentinel. Once set, all
   subsequent `on_data` calls are no-ops for this flow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Protocol ID = 0x0001 (first non-Modbus value) | `is_non_modbus = true`; `on_data` returns immediately; no parse_errors increment; no offset advance; no finding |
| EC-002 | Protocol ID = 0xFFFF (maximum, clearly non-Modbus) | Same as EC-001 — permanent bail-out on this flow |
| EC-003 | Protocol ID = 0x0000 (valid Modbus) | `is_valid_modbus_adu` proto check passes; Length check may still reject (BC-2.14.004); `is_non_modbus` stays false |
| EC-004 | Non-Modbus ADU is the FIRST PDU on the flow (e.g., ENIP header on port 502) | `is_non_modbus = true` immediately; all subsequent PDUs for this flow are no-ops; no resync |
| EC-005 | Protocol ID != 0x0000 with Length = 65535 (crafted DoS attempt) | Bail-out before offset arithmetic; `6 + 65535` is NEVER computed; flow marked non-Modbus; safe |
| EC-006 | Second call to `on_data` for a flow already marked `is_non_modbus = true` | Bails out at entry (`if flow.is_non_modbus { return; }`); zero work; no counters incremented |

## Canonical Test Vectors

| Input (hex, first 8 bytes shown) | `is_non_modbus` set? | `parse_errors` delta | Offset advanced? | Category |
|----------------------------------|----------------------|---------------------|-----------------|----------|
| `00 01  00 01  00 06  01 03` (proto=0x0001) | yes | +0 | no | happy-path: bail-out on first non-Modbus proto |
| `00 01  FF FF  00 06  01 03` (proto=0xFFFF) | yes | +0 | no | edge-case: max proto ID, permanent bail-out |
| `00 01  00 00  00 06  01 03` (proto=0x0000, len=6) | no | +0 | yes (normal) | regression: valid Modbus NOT rejected by this gate |
| `00 01  00 01  FF FF  01 03` (proto=0x0001, len=65535) | yes | +0 | no (DoS-safe) | security: crafted huge length never used in arithmetic |

**Annotated bail-out vector** (non-Modbus on port 502):
```
Bytes:  00 01  |  00 01  |  00 06  |  01  |  03
Field:  TxnID  |  ProtoID|  Length |  UID |  FC
Value:  0x0001 |  0x0001 |    6    |   1  | 0x03
Action: parse_mbap_header returns Some(h); protocol_id != 0x0000;
        set flow.is_non_modbus = true; return immediately.
        No advance. No parse_errors. Flow permanently silent.
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Sub-property A encompasses `is_valid_modbus_adu` as a pure predicate; no-panic over all `MbapHeader` inputs | Kani: if `is_valid_modbus_adu` is extracted as a pure fn |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC specifies the Protocol ID validity gate that prevents false-positive ICS findings from non-Modbus traffic on port 502, a security-critical property of the ICS analysis capability |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `is_valid_modbus_adu`); ADR-005 §2.4 (3-point gate) |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.001 — depends on (parse_mbap_header must return Some before this gate is applied)
- BC-2.14.002 — sibling (truncation reject path — distinct from validity gate reject)
- BC-2.14.004 — sibling (Length validity gate — same `is_valid_modbus_adu` function, different sub-check)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `is_valid_modbus_adu(h: &MbapHeader) -> bool`: `h.protocol_id == 0x0000 && h.length >= 2 && h.length <= 254`
- `src/analyzer/modbus.rs` — `ModbusFlowState.is_non_modbus: bool` — set to true on protocol_id failure; bail-out guard on entry to `on_data`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.4` — Desync / DoS safety policy: "Protocol-ID failure: set `flow.is_non_modbus = true` and return immediately from `on_data`"
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.3` — `ModbusFlowState` complete field list (is_non_modbus field per Decision 6)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — pure predicate coverage via Kani (if `is_valid_modbus_adu` is extracted)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.4; modbus-tcp-research.md §1 (Protocol ID always 0x0000); modbus-tcp-research.md §7 point 4 (three-point check rationale) |
| **Confidence** | high — Protocol ID == 0x0000 is a hard spec requirement (Modbus.org spec §4.1) |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (pure predicate) |
| **Global state access** | none |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync |
| **Overall classification** | pure core |
