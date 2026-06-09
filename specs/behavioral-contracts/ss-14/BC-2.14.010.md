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

# BC-2.14.010: Response PDU Matched Against Pending Table and Entry Removed on FC Echo Match

## Description

When `on_data` processes a validated Modbus ADU in the server-to-client direction (response
direction — `direction == Direction::ServerToClient`), it looks up `(h.transaction_id,
h.unit_id)` in `ModbusFlowState.pending`. A normal (non-exception) response echoes the
original request FC exactly: `h.function_code == pending_fc`. On a successful match, the
pending entry is removed from the table, closing the request/response pair. If the FC in
the response does NOT match the pending FC and is NOT an exception FC, it is an FC mismatch
anomaly. If no entry exists in the pending table for the incoming `(txn_id, unit_id)`, it
is an orphan response (possible in half-captures or under heavy pipelining).

## Preconditions

1. A Modbus ADU has been parsed and passed the 3-point validity gate.
2. `direction == Direction::ServerToClient` — the TCP connection's server side is sending
   data toward the client.
3. `classify_fc(h.function_code) != Exception` — exception responses are handled by
   BC-2.14.011, not this BC.
4. `ModbusFlowState.pending` contains at least one entry for the flow's current state.

## Postconditions

1. The function looks up `(h.transaction_id, h.unit_id)` in `ModbusFlowState.pending`.
2. **Case A — Match found, FC echo correct**:
   `pending.get(&(h.transaction_id, h.unit_id))` returns `Some((pending_fc, _))` AND
   `h.function_code == pending_fc`.
   - The entry is removed from `pending` via `pending.remove(&(h.transaction_id, h.unit_id))`.
   - `ModbusFlowState.pdu_count` is incremented by 1.
   - `ModbusFlowState.last_ts` is updated to `timestamp`.
   - `ModbusAnalyzer.total_pdu_count` is incremented by 1.
   - No finding is emitted for a normal successful response.
3. **Case B — Match found, FC echo incorrect** (FC mismatch):
   `pending.get(...)` returns `Some((pending_fc, _))` AND `h.function_code != pending_fc`
   (and `h.function_code < 0x80` so it's not an exception).
   - The pending entry is removed (the pair is considered closed regardless; the anomaly is noted).
   - `pdu_count` and `total_pdu_count` are incremented.
   - This is an anomalous condition; in v1, it is silently counted (no finding for v1 scope —
     the FC-mismatch detection is a P2 enhancement beyond the approved 6 MITRE techniques).
4. **Case C — No match (orphan response)**:
   `pending.get(...)` returns `None`.
   - `pdu_count` and `total_pdu_count` are incremented.
   - No pending entry is removed.
   - In v1, orphan responses are silently accepted (can occur legitimately in half-captures
     or under packet loss). No finding emitted.
5. In all cases, `ModbusFlowState.last_ts` is updated to `timestamp`.

## Invariants

1. **Transaction ID + Unit ID are the correlation key** (Modbus spec): the server echoes both
   the Transaction ID and Unit ID unchanged in its response. The pair `(txn_id, unit_id)` is
   the minimum correlation granularity; relying on Transaction ID alone could confuse responses
   for different slaves in a gateway scenario.
2. **Pending entry removal on match**: entries are removed from the pending table on response,
   not merely marked. This prevents the table from growing unboundedly under normal operation
   (each request-response pair consumes and then releases one slot).
3. **Orphan responses are not errors**: they occur legitimately when a capture starts mid-flow
   (no request was observed) or when a request was evicted from the pending table due to the
   MAX_PENDING_TRANSACTIONS cap (BC-2.14.012). Emitting findings for every orphan response
   would produce excessive false positives on in-progress captures.
4. **Exception responses are NOT handled here**: FC values `>= 0x80` in the response direction
   are classified as `Exception` by BC-2.14.006 and routed to BC-2.14.011 for attribution.
   This BC handles only non-exception normal responses.
5. **No write counter increment for responses**: response ADUs never increment `window_write_count`.
   Write detection is based on the request direction only (the client commands the write;
   the server's response confirms or rejects it).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Normal Read HR response with matching pending entry | Entry removed; pdu_count++; no finding |
| EC-002 | Normal Write HR response (FC=0x10) with matching pending entry | Entry removed; pdu_count++; no finding; write was already counted when request was received |
| EC-003 | Orphan response (no matching pending entry) | pdu_count++; no removal; no finding (v1: silent) |
| EC-004 | FC mismatch (response FC 0x03 when pending FC was 0x01) | Entry removed; pdu_count++; no finding in v1 (P2 enhancement) |
| EC-005 | Response arrives for a request whose pending entry was evicted (table was full) | Same as EC-003: orphan response, no matching entry |
| EC-006 | Capture starts mid-flow; first PDU is a response | Orphan; silently counted |
| EC-007 | Response matches a pending Write-class FC correctly (FC=0x06 echoed) | Entry removed; confirms write command was acknowledged by device |
| EC-008 | Two concurrent pending entries for different Unit IDs, same Transaction ID | Each keyed separately by (txn_id, unit_id); responses for each resolved independently |

## Canonical Test Vectors

| Scenario | Request ADU (preceding state) | Response ADU (hex) | Expected result | Category |
|----------|------------------------------|--------------------|-----------------|----------|
| Normal Read HR round-trip | Insert `(0x0001, 0x01) -> (0x03, ts0)` | `00 01 00 00 00 0F 01 03 0C 00 64 00 C8 01 2C 01 90 00 C8 00 FA` (FC=0x03, data=6 regs) | Entry `(0x0001, 0x01)` removed; no finding | happy-path |
| Normal Write SR confirmation | Insert `(0x0002, 0x01) -> (0x06, ts0)` | `00 02 00 00 00 06 01 06 00 14 01 F4` (FC=0x06 echo) | Entry `(0x0002, 0x01)` removed; no finding | happy-path |
| Orphan response (no pending) | pending table empty | `00 03 00 00 00 0F 01 03 0C ...` | Nothing removed; pdu_count++; no finding | edge-case |

**Annotated Read HR response vector**:
```
Bytes:  00 01  |  00 00  |  00 0F  |  01  |  03  | 0C  | 00 64  00 C8  01 2C  01 90  00 C8  00 FA
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  | bct | reg1   reg2   reg3   reg4   reg5   reg6
Value:  0x0001 |  0x0000 |   15    |   1  | 0x03 | 12B | 100    200    300    400    200    250
Action: Lookup (0x0001, 0x01) in pending → found (0x03, ts0) → FC match → remove entry
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Response matching removes pending entry on FC echo; pending table size decrements on successful match | Integration test (round-trip: insert request, process response, verify pending is empty) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the response matching mechanism that closes request/response pairs in the transaction correlation table, enabling BC-2.14.011 exception attribution and ensuring bounded pending-table growth |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `on_data` response-direction branch, `ModbusFlowState.pending`); ADR-005 §2.7 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.009 — depends on (pending table is populated by request insertion; this BC consumes from it)
- BC-2.14.011 — sibling (exception responses in the same direction are handled there, not here)
- BC-2.14.012 — constrains (MAX_PENDING_TRANSACTIONS; if table was full, some requests weren't inserted, making orphan responses more likely)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `on_data` response branch: `Direction::ServerToClient`, non-exception arm
- `src/analyzer/modbus.rs` — `flow.pending.remove(&(h.transaction_id, h.unit_id))` on match
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.7` — on_data logic step 6: "For responses: look up pending by (transaction_id, unit_id), validate FC echo, remove entry"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — response-matching integration test

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.7; modbus-tcp-research.md §4 (transaction correlation: "the server echoes Transaction ID, Unit ID, and FC") |
| **Confidence** | high — echo semantics are Modbus spec-defined [SPEC]; orphan-response handling is a judgment call [JUDGMENT] |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (operates on per-flow state) |
| **Deterministic** | yes — same pending state + ADU produces same table delta |
| **Thread safety** | n/a (single-threaded StreamHandler) |
| **Overall classification** | effectful shell (mutates per-flow HashMap state) |
