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

# BC-2.14.011: Exception Response Attributed to Originating Request FC via (Transaction ID, Unit ID) Lookup

## Description

When `on_data` processes a validated Modbus ADU in the server-to-client direction where
`classify_fc(h.function_code) == Exception` (i.e. `h.function_code >= 0x80`), the original
request FC is recovered as `original_fc = h.function_code & 0x7F` and the `(h.transaction_id,
h.unit_id)` key is looked up in `ModbusFlowState.pending`. If a matching pending entry exists,
the exception is attributed to the original request — the pending entry is removed and the
exception is correlated with the original FC and timestamp. When the original FC is Write-class,
the exception is forensically significant as evidence of an attempted (but rejected/failed)
write command — a T0855 attribution signal (finding emission handled in BC-2.14.013+, the
parallel burst). `ModbusFlowState.exception_count` is incremented for every exception response
regardless of attribution success.

## Preconditions

1. A Modbus ADU has been parsed and passed the 3-point validity gate.
2. `direction == Direction::ServerToClient` — exception responses come from the server.
3. `classify_fc(h.function_code) == FunctionCodeClass::Exception` — `h.function_code >= 0x80`.
4. `original_fc = h.function_code & 0x7F` — the original request FC has been recovered.
5. The ADU has `h.length >= 2` so the exception code byte at `data[8]` is readable.

## Postconditions

1. `original_fc = h.function_code & 0x7F` is computed.
2. The `(h.transaction_id, h.unit_id)` key is looked up in `ModbusFlowState.pending`.
3. **Case A — Attributed exception** (matching pending entry found):
   - `pending_entry = pending.remove(&(h.transaction_id, h.unit_id))` removes the entry.
   - `pending_fc = pending_entry.0` (the original request FC from the pending table).
   - **Consistency check**: if `original_fc == pending_fc`, the attribution is confirmed —
     the server is responding to the expected request FC. If `original_fc != pending_fc`, the
     attribution is anomalous (the server's exception FC does not match the pending request FC);
     still treated as an exception for this flow, but the anomaly may be flagged (v1: silently
     accepted; the pending entry is still removed).
   - `ModbusFlowState.exception_count` is incremented by 1.
   - `ModbusAnalyzer.total_exception_count` is incremented by 1.
   - If `classify_fc(original_fc) == Write`: the exception is forensic evidence of an
     attempted write command that the server rejected. A finding may be emitted (handled in
     BC-2.14.013+, T0855 attribution).
4. **Case B — Unattributed exception** (no matching pending entry):
   - `ModbusFlowState.exception_count` is incremented by 1.
   - `ModbusAnalyzer.total_exception_count` is incremented by 1.
   - No pending entry is removed.
   - No attribution finding is emitted (orphan exception — same rationale as BC-2.14.010
     Case C: can occur in half-captures).
5. The exception code byte at `data[8]` is recorded for use in finding emission (BC-2.14.013+).
   Valid exception codes: `{0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x0A, 0x0B}`.
6. `ModbusFlowState.pdu_count` and `ModbusAnalyzer.total_pdu_count` are incremented by 1.
7. `ModbusFlowState.last_ts` is updated to `timestamp`.

## Invariants

1. **Exception attribution purpose**: the primary forensic value of exception attribution is
   correlating a server's "I rejected your command" with the original command that was rejected.
   An exception for a Write-class FC (`0x85`, `0x86`, `0x90`, etc.) is evidence of an attempted
   unauthorized write even though it failed. The exception code (e.g. `0x01` Illegal Function)
   tells the analyst WHY the server rejected it.
2. **Exception count is monotonically non-decreasing**: `exception_count` is incremented for
   every exception response, regardless of whether attribution succeeded. It is an aggregate
   counter observable in `summarize()` (BC-2.14.020).
3. **original_fc recovery is lossless**: `fc & 0x7F` for any `fc` in `[0x80, 0xFF]` yields the
   correct original FC in `[0x00, 0x7F]`. For standard exception FCs (e.g. `0x83 = 0x03 | 0x80`),
   this gives back the correct request FC.
4. **Removal of pending entry on exception**: the pending slot is freed when an exception response
   arrives — the request/response cycle is complete (failed). This prevents slots from being
   held indefinitely by requests that receive exception responses rather than normal responses.
5. **Write-class attribution signal**: exception on a Write-class FC is forensically equivalent
   to the write being attempted. The server's rejection does not remove the threat signal — it
   provides corroborating evidence that a write command was issued and provides the server's
   reaction code.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Exception FC = 0x83 (exception for Read HR 0x03) | `original_fc = 0x03`; Read class; exception_count++; no T0855 attribution (read exception) |
| EC-002 | Exception FC = 0x85 (exception for Write Single Coil 0x05) | `original_fc = 0x05`; Write class; exception_count++; T0855 attribution candidate |
| EC-003 | Exception FC = 0x90 (exception for Write Multiple Registers 0x10) | `original_fc = 0x10`; Write class; exception_count++; T0855 attribution candidate |
| EC-004 | Exception with no matching pending entry (orphan) | exception_count++; no removal; no finding |
| EC-005 | Exception code = 0x01 (Illegal Function) attributed to Write FC | Write FC attempted; server doesn't support it — could indicate probing of device capabilities |
| EC-006 | Exception code = 0x04 (Device Failure) attributed to Write FC | Write may have caused device stress; high forensic significance |
| EC-007 | Exception code = 0x0B (Gateway Target Failed) attributed to any FC | Device behind gateway is silent — potential DoS impact |
| EC-008 | original_fc in pending table != exception's recovered original_fc | FC mismatch anomaly; v1: pending entry still removed; exception_count++; anomaly silently noted |
| EC-009 | Exception FC = 0x80 (original_fc = 0x00, undefined FC) | exception_count++; original FC 0x00 is Unknown class; no write attribution |

## Canonical Test Vectors

| Scenario | Prior pending state | Exception ADU (hex) | Expected result | Category |
|----------|---------------------|---------------------|-----------------|----------|
| Write exception attributed | `(0x0002, 0x01) -> (0x06, ts0)` in pending | `00 02 00 00 00 03 01 86 01` (FC=0x86, code=0x01) | `original_fc=0x06` (Write); entry removed; `exception_count=1`; T0855 attribution (BC-2.14.013+) | happy-path |
| Read exception attributed | `(0x0001, 0x01) -> (0x03, ts0)` in pending | `00 01 00 00 00 03 01 83 02` (FC=0x83, code=0x02) | `original_fc=0x03` (Read); entry removed; `exception_count=1`; no write attribution | happy-path |
| Orphan exception | pending empty | `00 05 00 00 00 03 01 90 04` (FC=0x90, code=0x04) | `exception_count=1`; no removal; no attribution | edge-case |
| Force Listen Only exception (unusual) | any pending state | `00 06 00 00 00 03 01 88 01` (FC=0x88 = exception for 0x08 Diagnostics) | `original_fc=0x08`; Diagnostic class; exception_count++; no write attribution | edge-case |

**Annotated Write Single Reg exception vector** (FC=0x86 = exception for 0x06):
```
Bytes:  00 02  |  00 00  |  00 03  |  01  |  86  |  01
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  | ExCode
Value:  0x0002 |  0x0000 |    3    |   1  | 0x86 | 0x01 (Illegal Function)
Decode: fc=0x86 >= 0x80 → Exception; original_fc = 0x86 & 0x7F = 0x06 (Write Single Reg)
Lookup: pending[(0x0002, 0x01)] = (0x06, ts0) → confirmed write attribution
Action: remove entry; exception_count++; T0855 finding candidate (BC-2.14.013+)
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Exception attribution correctly identifies original_fc for all fc in [0x80,0xFF]; exception_count incremented for all exception responses | Integration test: round-trip (request insert, exception response, verify count and removal) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines exception attribution, which correlates failed write commands with their originating requests, enabling T0855 "failed execution evidence" detection in the ICS analysis capability |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation — exception code byte stored raw) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `on_data` exception-response branch); ADR-005 §2.7 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.006 — depends on (Exception classification and original_fc recovery defined there)
- BC-2.14.009 — depends on (pending entry was inserted by request handling; attribution requires that entry)
- BC-2.14.010 — sibling (normal responses handled there; this BC handles exception responses only)
- BC-2.14.012 — constrains (if pending was full, some requests weren't stored; orphan exceptions occur)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `on_data` exception-response arm: `classify_fc(h.function_code) == Exception` branch in `ServerToClient` direction
- `src/analyzer/modbus.rs` — `original_fc = h.function_code & 0x7F` recovery
- `src/analyzer/modbus.rs` — `flow.pending.remove(&(h.transaction_id, h.unit_id))` on attributed exception
- `src/analyzer/modbus.rs` — `flow.exception_count += 1` and `self.total_exception_count += 1`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.7` — on_data step 6: "emit attribution findings (e.g. exception on write FC → T0855 evidence)"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — exception attribution integration test

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.7; modbus-tcp-research.md §3 (exception response format and codes); research §4 point 2 (exception flag as directionality corroboration) |
| **Confidence** | high — exception FC bit pattern is Modbus spec [SPEC]; exception codes table confirmed (no 0x09) [SPEC] |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (operates on per-flow state) |
| **Deterministic** | yes |
| **Thread safety** | n/a (single-threaded StreamHandler) |
| **Overall classification** | effectful shell (mutates pending HashMap and exception counters) |
