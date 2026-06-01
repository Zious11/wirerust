---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/mod.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 9 STORY-020 EC-004 correction and EC-005 addition — 2026-05-26"
  - "v1.4: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:225-235 → mod.rs:248-271 (get_or_create_flow fn). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.015: Flow Eviction on max_flows Hit Uses LRU Non-Established-First

## Description

When a new flow would be created but `self.flows.len() >= config.max_flows`, `get_or_create_flow`
calls `evict_flows`. If after eviction the table is still at capacity (under v1.3 Invariant 4,
this only occurs when evict_flows ran as a no-op), the new packet is dropped (returns `false`). The eviction strategy (in `lifecycle.rs::evict_flows`) sorts all
current flows: non-Established flows come first (sorted by `last_seen` ascending), then
Established flows (sorted by `last_seen` ascending). Flows are closed in that order until
both `total_memory <= memcap` AND `flows.len() <= max_flows`.

## Preconditions

1. `self.flows.len() >= config.max_flows` when a new flow arrival occurs.
2. `evict_flows` is called by `get_or_create_flow`.

## Postconditions

1. Non-Established flows (state != Established) are evicted before Established flows.
2. Within each group, oldest `last_seen` flows are evicted first.
3. `stats.evictions` increments by the number of flows evicted.
4. Each evicted flow triggers `close_flow(key, CloseReason::MemoryPressure, handler)`.
5. Eviction stops as soon as `flows.len() <= max_flows` AND `total_memory <= memcap`.
6. If the table is still at capacity after eviction (under v1.3 Invariant 4, this only occurs
   when evict_flows ran as a no-op), `get_or_create_flow` returns `false` and the packet is
   dropped (no flow created).
7. **Data-delivery semantics under CloseReason::MemoryPressure eviction:** On `CloseReason::MemoryPressure`,
   only the contiguous head-of-buffer prefix is delivered to the handler via `on_data` callback
   (via `flush_contiguous()` per direction). Non-contiguous buffered segments (segments held waiting
   on a gap fill) are DISCARDED — no `on_data` callback fires for them. The number of discarded bytes
   equals `flow.memory_used() at close time − bytes delivered by flush_contiguous()`. Forensic
   analysts should account for this potential data loss when evictions occur under memory pressure:
   an eviction-heavy capture may have non-contiguous segments dropped that would otherwise have been
   reassembled if memory budget had been larger.

## Invariants

1. `evict_flows` is called both for `max_flows` pressure (in `get_or_create_flow`) and for
   `memcap` pressure (in `process_packet` after each packet). Both call the same function.
2. The sort is computed fresh on each evict_flows call (not cached).
3. `CloseReason::MemoryPressure` distinguishes eviction closes from RST/FIN/Timeout closes.
4. DESIGN INTENT: `evict_flows` uses dual-conjunction termination (`total_memory <= memcap AND flows.len() <= max_flows`) deliberately. This protects Established sessions from eviction under max_flows-only pressure when memory budget is ample. Only paired resource pressure (both memcap and max_flows exhausted simultaneously) can evict an Established flow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All flows are Established | LRU Established flows evicted; oldest first |
| EC-002 | All flows are New (no SYN seen) | All are non-Established; LRU evicted first |
| EC-003 | After eviction, table still at capacity (under v1.3 Invariant 4, this only occurs when evict_flows ran as a no-op) | get_or_create_flow returns false; packet dropped |
| EC-004 | Single flow in table (`max_flows=1`, `total_memory <= memcap`), new flow arrives | evict_flows exits immediately (both PC-5 termination conditions already satisfied); packet dropped (no flow created). max_flows-only pressure without memcap pressure is insufficient to trigger eviction; this protects Established sessions from being dislodged by mere flow-count pressure when memory budget is ample. |
| EC-005 | Single flow in table with buffered data exceeding memcap (`max_flows=1`, `total_memory > memcap`), new SYN arrives | memcap pressure triggers eviction; existing flow evicted via CloseReason::MemoryPressure; new flow created. Both max_flows AND memcap pressure together can evict Established sessions. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| max_flows=2; two Established flows; new SYN | Oldest Established evicted; new flow created | happy-path |
| max_flows=1; one New flow + one Established; new SYN | New (non-Established) evicted first | happy-path |
| max_flows=1; total ≤ memcap; new SYN | evict_flows no-op (dual-conjunction terminates at head per Inv 4); packet dropped; existing flow preserved | edge-case |
| flow with 5 contiguous bytes [0..5) + 5 non-contiguous bytes [10..15) buffered (gap at [5..10)); evicted via MemoryPressure | handler.on_data called with first 5 bytes; bytes [10..15) discarded silently; on_flow_close called with CloseReason::MemoryPressure | data-loss-on-eviction |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Non-Established evicted before Established | unit: mix of flow states at max_flows; verify eviction order |
| — | Oldest (lowest last_seen) evicted within each group | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- flow table eviction is required to bound concurrent flow count |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:248-271, get_or_create_flow; lifecycle.rs:67-92, evict_flows) |
| Stories | STORY-020 |
| Origin BC | BC-RAS-015 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.016 -- composes with (same evict_flows called for memcap pressure)
- BC-2.04.017 -- composes with (sort order in evict_flows)
- BC-2.04.014 -- related to (total_memory drives eviction stop condition)

## Architecture Anchors

- `src/reassembly/mod.rs:248-271` -- get_or_create_flow: max_flows check + evict_flows call
- `src/reassembly/lifecycle.rs:67-92` -- evict_flows: sort + close loop

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/lifecycle.rs:67-92` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.flows.len() >= self.config.max_flows` in get_or_create_flow
- **assertion**: sort by (is_established ASC, last_seen ASC) -- explicit Ord in evict_flows

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats, self.total_memory |
| **Deterministic** | yes (for a fixed set of flows and timestamps) |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callbacks) |

## Refactoring Notes

The sort-then-evict pattern allocates a Vec on every call. For very large flow tables this
could be optimized, but it is not on the hot path (eviction is rare in normal traffic).

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-05-20 | product-owner | Initial brownfield extraction |
| 1.2 | 2026-05-21 | product-owner | VP back-reference back-fill (P8-DEFER) |
| 1.3 | 2026-05-26 | product-owner | Wave 9 STORY-020 implementer investigation: revised EC-004 to match actual implementation (evict_flows is no-op when only max_flows pressure exists without memcap pressure); added EC-005 for dual-pressure case; added DESIGN INTENT invariant (Invariant 4). PC-5 termination condition is correct; EC-004 was the misstatement. |
| 1.4 | 2026-05-26 | product-owner | Wave 9 STORY-020 adv pass-4 F-PASS4-002 (HIGH, sibling-regression of pass-3 F-002): Canonical Test Vector row at line 77 revised — the 'one flow evicted but still at cap' scenario is structurally unreachable under v1.3 Invariant 4 (evicting brings flows.len to 0 < 1); replaced with the actually-reachable no-op-eviction path. Description/PC-6/EC-003 'still at capacity after eviction' wording clarified with Inv 4 parenthetical. |
| 1.5 | 2026-05-26 | product-owner | Wave 9 wave-level adv pass-1 F-W9P1-002: added PC-7 documenting data-delivery semantics under MemoryPressure eviction (non-contiguous buffered segments are silently discarded; flush_contiguous delivers only head-of-buffer prefix); added canonical test vector row showing data-loss case. |
