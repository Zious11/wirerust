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

# BC-2.14.009: Request PDU Inserted into Per-Flow Pending Table Keyed on (Transaction ID, Unit ID)

## Description

When `on_data` processes a validated Modbus ADU in the client-to-server direction (request
direction — `direction == Direction::ClientToServer`, i.e., the TCP flow's client side is
sending toward port 502), the `(transaction_id, unit_id)` pair is used as a key to insert
an entry into `ModbusFlowState.pending`. The entry maps `(txn_id, unit_id) -> (request_fc,
timestamp)`, recording the requesting function code and the pcap-relative timestamp at which
the request was observed. This table enables response matching (BC-2.14.010) and exception
attribution (BC-2.14.011). Insertion is subject to the MAX_PENDING_TRANSACTIONS bound
(BC-2.14.012).

## Preconditions

1. A Modbus ADU has been parsed and passed the 3-point validity gate (BC-2.14.001 through
   BC-2.14.004).
2. The ADU's `direction == Direction::ClientToServer` — the TCP connection's client side is
   sending data toward the server on port 502.
3. `classify_fc(h.function_code)` is NOT `Exception` — exception FCs in the client direction
   are unusual and are not inserted into the pending table.
4. `ModbusFlowState.pending.len() < MAX_PENDING_TRANSACTIONS` (256) — the table is not full.
   If full, the entry is silently dropped per BC-2.14.012.

## Postconditions

1. A new entry is inserted into `ModbusFlowState.pending`:
   - Key: `(h.transaction_id, h.unit_id)` — a `(u16, u8)` tuple.
   - Value: `(h.function_code, timestamp)` — a `(u8, u32)` tuple where `timestamp` is the
     pcap-relative `u32` timestamp passed to `on_data` at the current call site.
2. If a pending entry with the same `(transaction_id, unit_id)` key already exists (i.e., a
   duplicate Transaction ID is reused before the previous response arrives), the new entry
   OVERWRITES the old one via `HashMap::insert`. `ModbusAnalyzer.duplicate_inflight_txn` is
   incremented by 1 to track how many pending entries were overwritten (attribution loss).
   This counter is available for internal diagnostics. See Invariant 6.
3. `ModbusFlowState.pdu_count` is incremented by 1 (all valid ADUs, regardless of direction
   or FC class, increment the PDU counter).
4. `ModbusFlowState.last_ts` is updated to `timestamp`.
5. If `classify_fc(h.function_code) == Write`, `ModbusFlowState.window_write_count` and
   `ModbusFlowState.write_count` are incremented (rate detection, per BC-2.14.007).
6. The aggregate `ModbusAnalyzer.total_pdu_count` is incremented by 1.

## Invariants

1. **Key invariant** (Modbus spec and ADR-005 §2.3): the pending table is keyed on
   `(transaction_id, unit_id)` — NOT on transaction_id alone. Unit ID is included because
   a Modbus TCP gateway may route to multiple serial slaves, each with a distinct Unit ID,
   and Transaction IDs may be reused across different slaves.
2. **Timestamp provenance**: the `timestamp` stored in the pending entry is the pcap-relative
   `u32` from `on_data`. This is the same provenance model as BC-2.09.007 (Finding.timestamp).
3. **Direction determination**: `Direction::ClientToServer` is the TCP connection direction
   provided by the reassembler's `StreamHandler::on_data` parameter. It reflects which TCP
   endpoint sent the data, not a port analysis on the packet. Port 502 being the destination
   port on a `ClientToServer` packet is the expected normal state; the direction enum is the
   authoritative signal per ADR-005 §2.7.
4. **No finding emitted on insert**: inserting a request into the pending table is a state
   transition, not a detection event. Findings are emitted when the FC class of the request
   triggers a detection rule (Write-class findings in BC-2.14.013+) or when the response is
   received (BC-2.14.010 and BC-2.14.011).
5. **Overwrite on key collision**: `HashMap::insert` for a duplicate key is safe — the value
   is updated to the latest request. This is correct behavior: if a client reuses a Transaction
   ID before the response arrives (pipeline ID exhaustion), the latest request governs.
6. **`duplicate_inflight_txn` counter on `ModbusAnalyzer`** (INTERNAL — declared in
   architecture-delta.md §2.2): this u64 counter is incremented whenever
   `pending.insert((txn_id, unit_id), ...)` overwrites an existing entry (i.e., the old entry
   is replaced before a response was seen). It tracks attribution loss: the original request's
   FC is silently discarded when overwritten. The counter is INTERNAL and is NOT surfaced as
   a summarize() key; BC-2.14.021's six-key contract is unchanged (this is not a 7th key).
   If elevated, it indicates either a misbehaving client or an attacker exhausting Transaction
   IDs to suppress attribution of subsequent commands. Note: `HashMap::insert` returns
   `Some(old_value)` when an overwrite occurs; the code increments `duplicate_inflight_txn`
   when `insert` returns `Some(_)`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First ADU on a new flow (pending table is empty) | Entry inserted with key `(txn_id, unit_id)`, value `(fc, ts)`; `pdu_count = 1` |
| EC-002 | Same `(txn_id, unit_id)` seen twice before response (ID reuse) | Second insert overwrites first — safe overwrite via HashMap::insert |
| EC-003 | Transaction ID = 0x0000 (connection start) | Valid key; inserted normally |
| EC-004 | Unit ID = 0xFF (broadcast) | Valid key component; broadcast requests can be tracked |
| EC-005 | Write-class FC request | Inserted into pending AND `window_write_count` incremented; both actions happen in the same on_data call |
| EC-006 | Read-class FC request | Inserted into pending; `window_write_count` NOT incremented |
| EC-007 | Exception FC in client direction (unusual) | NOT inserted into pending table (precondition 3); counted in pdu_count |
| EC-008 | Pending table at MAX_PENDING_TRANSACTIONS - 1 (one slot left) | Entry inserted normally; next insert after this will be dropped (BC-2.14.012) |

## Canonical Test Vectors

| Scenario | Input ADU (hex) | Expected pending table delta | Notes |
|----------|-----------------|------------------------------|-------|
| Read HR request, fresh flow | `00 01 00 00 00 06 01 03 00 00 00 0A` | Insert `(0x0001, 0x01) -> (0x03, ts)` | FC=0x03 Read; no write counter |
| Write Single Reg request | `00 02 00 00 00 06 01 06 00 14 01 F4` | Insert `(0x0002, 0x01) -> (0x06, ts)`; `window_write_count++` | FC=0x06 Write; write counter +1 |
| Write Multiple Reg request | `00 03 00 00 00 0B 02 10 00 00 00 02 04 00 64 00 C8` | Insert `(0x0003, 0x02) -> (0x10, ts)`; `window_write_count++` | FC=0x10 Write; bulk write |
| Duplicate txn_id before response | ADU with `txn_id=0x0001, unit=0x01, fc=0x03` arrives again | `(0x0001, 0x01) -> (0x03, ts2)` overwrites existing entry | Safe overwrite |

**Annotated Read HR request vector** (most common SCADA request):
```
Bytes:  00 01  |  00 00  |  00 06  |  01  |  03  | 00 00   00 0A
Field:  TxnID  |  ProtoID|  Length |  UID |  FC  | addr=0  qty=10
Value:  0x0001 |  0x0000 |    6    |   1  | 0x03 | read 10 holding regs from addr 0
Action: Parse, validate, classify Read, insert (0x0001,0x01)->(0x03,ts) into pending
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | Pending table insertion is guarded by MAX_PENDING_TRANSACTIONS; no unbounded growth | Integration test (property-based: insert 256 entries, verify 257th is dropped) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the request tracking mechanism that enables transaction correlation (response matching and exception attribution), which is a core component of the ICS analysis capability's threat detection |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation — timestamp stored raw as u32; no formatting at insert time) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `on_data` request-direction branch, `ModbusFlowState.pending`); ADR-005 §2.3 and §2.7 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.005 — depends on (classify_fc result determines FC class before insert)
- BC-2.14.007 — depends on (Write-class detection triggers write counter increment alongside pending insert)
- BC-2.14.010 — composes with (pending insert is the precondition for response matching)
- BC-2.14.011 — composes with (pending insert is the precondition for exception attribution)
- BC-2.14.012 — constrains (MAX_PENDING_TRANSACTIONS cap bounds this table)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `ModbusFlowState.pending: HashMap<(u16, u8), (u8, u32)>` — key=(txn_id, unit_id), value=(request_fc, timestamp)
- `src/analyzer/modbus.rs` — `on_data` request branch: `Direction::ClientToServer` arm inserts into `flow.pending`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.3` — ModbusFlowState layout
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.7` — on_data logic step 5: "For requests: insert into pending table (if table is not full)"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — pending table bound integration test (MAX_PENDING_TRANSACTIONS guard)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.3 and §2.7; modbus-tcp-research.md §4 (transaction correlation table recommendation) |
| **Confidence** | high — Transaction ID echo-on-response is a Modbus spec requirement [SPEC]; (txn_id, unit_id) key design is from ADR-005 |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (operates on per-flow state via `&mut ModbusFlowState`) |
| **Deterministic** | yes — same input + state always produces same state transition |
| **Thread safety** | n/a (single-threaded StreamHandler invocation) |
| **Overall classification** | effectful shell (mutates per-flow HashMap state) |
