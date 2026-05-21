---
document_type: behavioral-contract
level: L3
version: "1.2"
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
calls `evict_flows`. If after eviction the table is still at capacity, the new packet is
dropped (returns `false`). The eviction strategy (in `lifecycle.rs::evict_flows`) sorts all
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
6. If the table is still at capacity after eviction, `get_or_create_flow` returns `false`
   and the packet is dropped (no flow created).

## Invariants

1. `evict_flows` is called both for `max_flows` pressure (in `get_or_create_flow`) and for
   `memcap` pressure (in `process_packet` after each packet). Both call the same function.
2. The sort is computed fresh on each evict_flows call (not cached).
3. `CloseReason::MemoryPressure` distinguishes eviction closes from RST/FIN/Timeout closes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | All flows are Established | LRU Established flows evicted; oldest first |
| EC-002 | All flows are New (no SYN seen) | All are non-Established; LRU evicted first |
| EC-003 | After eviction, table still at capacity | get_or_create_flow returns false; packet dropped |
| EC-004 | Single flow in table, max_flows=1, new flow arrives | The one flow is evicted; new flow created |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| max_flows=2; two Established flows; new SYN | Oldest Established evicted; new flow created | happy-path |
| max_flows=1; one New flow + one Established; new SYN | New (non-Established) evicted first | happy-path |
| max_flows=1; one flow evicted but still at cap | Packet dropped; flows.len() == 1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Non-Established evicted before Established | unit: mix of flow states at max_flows; verify eviction order |
| — | Oldest (lowest last_seen) evicted within each group | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- flow table eviction is required to bound concurrent flow count |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:225-235, get_or_create_flow; lifecycle.rs:67-92, evict_flows) |
| Stories | S-TBD |
| Origin BC | BC-RAS-015 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.016 -- composes with (same evict_flows called for memcap pressure)
- BC-2.04.017 -- composes with (sort order in evict_flows)
- BC-2.04.014 -- related to (total_memory drives eviction stop condition)

## Architecture Anchors

- `src/reassembly/mod.rs:225-235` -- get_or_create_flow: max_flows check + evict_flows call
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
