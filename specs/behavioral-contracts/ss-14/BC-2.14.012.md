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

# BC-2.14.012: Pending Table Bounded to MAX_PENDING_TRANSACTIONS=256; New Requests Dropped When Full

## Description

`ModbusFlowState.pending` is a `HashMap<(u16, u8), (u8, u32)>` bounded to
`MAX_PENDING_TRANSACTIONS = 256` entries per flow. When the pending table has reached
this bound and a new request-direction ADU arrives, the new entry is NOT inserted —
it is silently dropped. Existing entries continue to be matched and removed on response.
The bound prevents unbounded memory growth under pathological conditions (heavy pipelining,
adversarial captures with many simultaneous in-flight transactions). This BC defines the
exact bound semantics, the eviction policy (no eviction — new requests are simply refused),
and the safety guarantee (the analyzer never panics or leaks on a full table). The bound
is a resource invariant analogous to `MAX_FINDINGS = 10,000` for findings.

## Preconditions

1. `ModbusFlowState.pending.len() >= MAX_PENDING_TRANSACTIONS` (= 256).
2. A new request-direction ADU has arrived with a `(transaction_id, unit_id)` key that
   does NOT already exist in the pending table (so the insert would genuinely expand the
   table, not overwrite an existing entry).

## Postconditions

1. The new request entry is NOT inserted into `ModbusFlowState.pending`. The table size
   does NOT exceed `MAX_PENDING_TRANSACTIONS`.
2. The ADU's FC class detection logic (Write counter, rate detection) still executes —
   the bound applies only to pending-table insertion, not to FC classification or
   write-rate counting. A Write-class FC that is dropped from pending still increments
   `window_write_count` and `write_count`.
3. `ModbusFlowState.pdu_count` and `ModbusAnalyzer.total_pdu_count` are incremented
   (the ADU is processed for counting purposes even if not tracked for correlation).
4. `ModbusFlowState.last_ts` is updated to `timestamp`.
5. The analyzer does NOT panic, abort, or produce undefined behavior when the table is full.
6. **Existing entries remain unaffected**: responses for requests that ARE in the pending
   table continue to be matched and removed normally (BC-2.14.010 and BC-2.14.011).
7. When a response arrives for a request that was dropped (pending table was full at request
   time), BC-2.14.010 Case C applies — the response is treated as an orphan response.

## Invariants

1. **Hard upper bound**: `ModbusFlowState.pending.len()` NEVER exceeds `MAX_PENDING_TRANSACTIONS = 256`
   under any input sequence. This is an absolute safety invariant.
2. **No eviction of existing entries**: the bound policy is "refuse new inserts, keep existing
   entries." There is no LRU eviction, no oldest-entry eviction, no random eviction. This
   ensures that in-flight requests that were accepted into the table can still be matched
   and attributed — eviction would lose attribution for partially-tracked transactions.
3. **Write-rate counting is not affected by the bound**: FC classification and write-rate
   detection (BC-2.14.007, and the burst-detection BCs in the 013+ group) operate independently
   of pending-table state. Even if the table is full, new Write-class FCs still count toward
   the rate detector. This is the correct security behavior: an attacker cannot escape write-
   rate detection simply by flooding the pending table first.
4. **`MAX_PENDING_TRANSACTIONS = 256` rationale** (from ADR-005 §2.3): "heavy pipelining or a
   pathological capture" can produce many simultaneous in-flight transactions. 256 is chosen
   as the bound because Modbus Transaction IDs are 16-bit (0..=65535), so a client could in
   theory have up to 65536 in-flight transactions; 256 is a conservative bound that covers
   legitimate pipelined use while bounding memory.
5. **Memory bound per flow**: each pending entry is `(u16, u8) -> (u8, u32)` = 3 bytes key +
   5 bytes value = 8 bytes per entry (excluding HashMap overhead). At 256 entries, the raw
   payload is 256 * 8 = 2 KB per flow, plus HashMap overhead (~80–100 bytes per entry for
   a default Rust HashMap allocation). The total per-flow pending table memory is bounded to
   approximately 25 KB at maximum capacity.
6. **The guard is a pre-insert check, not post-insert trim**: the implementation checks
   `if flow.pending.len() < MAX_PENDING_TRANSACTIONS` before calling `flow.pending.insert(...)`.
   This is a simpler and safer pattern than inserting and then trimming.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Pending table has 255 entries; new request arrives | Entry inserted (table goes to 256 = MAX); no drop |
| EC-002 | Pending table has 256 entries; new request with new (txn_id, unit_id) arrives | Entry NOT inserted; table stays at 256; write-rate still counted if Write-class FC |
| EC-003 | Pending table has 256 entries; new request with an EXISTING (txn_id, unit_id) key | Entry OVERWRITES existing (HashMap::insert behavior for existing key); table stays at 256; this is a key-reuse scenario (see BC-2.14.009 EC-002), NOT a new insert |
| EC-004 | Pending table at 256; response arrives for an existing entry | Entry removed; table drops to 255; subsequent new request can now be inserted |
| EC-005 | Pending table at 256; response arrives for an existing entry; immediately followed by new request | After removal, table at 255; new request inserts normally (255 < 256) |
| EC-006 | Adversarial: client sends 1000 requests with unique (txn_id, unit_id) pairs without any responses | Pending table caps at 256; entries 257..1000 are dropped; analyzer does not panic; pdu_count reaches 1000; write-rate counted for Write-class FCs throughout |
| EC-007 | Response for a dropped request arrives | BC-2.14.010 Case C (orphan response) — no matching pending entry; silently accepted |
| EC-008 | Flow close while pending table has 256 entries | `on_flow_close` removes the flow from `ModbusAnalyzer.flows`; the `ModbusFlowState` (including the pending HashMap) is dropped; no findings emitted for pending entries (per ADR-005 §2.7: "orphaned transactions at close are not flagged") |

## Canonical Test Vectors

| Scenario | State before | Operation | State after | Category |
|----------|-------------|-----------|-------------|----------|
| Insert at cap − 1 | pending.len() = 255 | New request arrives | pending.len() = 256; entry inserted | happy-path |
| Drop at cap | pending.len() = 256 | New request (unique key) | pending.len() = 256; entry NOT inserted; pdu_count++ | happy-path: drop at bound |
| Drop at cap, Write-class FC | pending.len() = 256 | New Write FC request (unique key) | pending.len() = 256; window_write_count++; write_count++ (rate detection continues) | important: write counting is not gated by table-full |
| Remove via response after cap | pending.len() = 256 | Response for existing entry | pending.len() = 255; entry removed | happy-path: table drains normally |
| Adversarial flood: 300 unique requests | pending.len() = 0 | 300 sequential requests (all unique keys, no responses) | pending.len() = 256 after request 256; requests 257–300 dropped; no panic | adversarial bound test |

**Implementation guard** (pseudocode):
```rust
// In on_data, request branch:
if flow.pending.len() < MAX_PENDING_TRANSACTIONS {
    flow.pending.insert((h.transaction_id, h.unit_id), (h.function_code, timestamp));
}
// NOTE: FC classification and write-rate counting happen REGARDLESS of the guard above.
// The guard only blocks pending-table insertion.
```

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-022 | `pending.len()` never exceeds `MAX_PENDING_TRANSACTIONS = 256` under any input sequence; no panic on full table | Integration test / property-based test: feed 1000 unique-key requests without responses; assert `pending.len() <= 256` and no panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 |
| Capability Anchor Justification | CAP-14 ("Modbus/ICS Analysis") per ARCH-INDEX.md §SS-14 — this BC defines the pending-table memory bound that prevents the ICS analyzer from being resource-exhausted by adversarial captures or heavy pipelining, ensuring the analyzer is safe to run on untrusted network captures |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap with bounded-resource design — this BC applies the same bounded-resource pattern to the pending table that INV-6 applies to findings) |
| Architecture Module | SS-14 (analyzer/modbus.rs C-22 `ModbusFlowState.pending`; `MAX_PENDING_TRANSACTIONS = 256`); ADR-005 §2.3 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-007-modbus-analyzer |

## Related BCs

- BC-2.14.009 — constrained by (pending insert is subject to this bound)
- BC-2.14.010 — related to (response removal reduces table size; drops via cap cause orphan responses)
- BC-2.14.011 — related to (exception attribution for requests that were dropped fails gracefully as orphan exceptions)

## Architecture Anchors

- `src/analyzer/modbus.rs` — `const MAX_PENDING_TRANSACTIONS: usize = 256`
- `src/analyzer/modbus.rs` — `ModbusFlowState.pending: HashMap<(u16, u8), (u8, u32)>`
- `src/analyzer/modbus.rs` — guard: `if flow.pending.len() < MAX_PENDING_TRANSACTIONS { flow.pending.insert(...); }`
- `.factory/phase-f2-spec-evolution/architecture-delta.md §2.3` — "MAX_PENDING_TRANSACTIONS = 256: when the pending table reaches 256 entries... new request entries are NOT inserted"
- `.factory/specs/architecture/ARCH-INDEX.md §127` — "MAX_PENDING_TRANSACTIONS = 256 per Modbus flow (transaction correlation table)"

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

- VP-022 — pending-table bound property-based test

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | architecture-delta.md §2.3; ARCH-INDEX.md §Bounded-Resource Design; modbus-tcp-research.md §7 point 5 ("Duplicate Transaction IDs / orphan responses can be normal under heavy pipelining") |
| **Confidence** | high — bound value and drop-not-evict policy are explicitly specified in the architecture delta and ARCH-INDEX |
| **Extraction Date** | 2026-06-09 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none |
| **Deterministic** | yes — table full + unique key always results in drop |
| **Thread safety** | n/a (single-threaded StreamHandler) |
| **Overall classification** | effectful shell (HashMap state mutation; bounded by constant) |
