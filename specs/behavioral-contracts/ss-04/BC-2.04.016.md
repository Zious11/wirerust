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

# BC-2.04.016: Memory Pressure Eviction When total_memory Exceeds memcap

## Description

After each packet's payload segment is inserted and flushed, `process_packet` checks
`if self.total_memory > self.config.memcap`. If true, `evict_flows` is called to shed flows
until total_memory returns to within bounds. This is the memcap-triggered eviction path,
distinct from the max_flows-triggered path in `get_or_create_flow`. Both paths call the same
`evict_flows` function with the same LRU non-established-first strategy.

## Preconditions

1. Segment bytes have just been inserted, causing `total_memory` to exceed `memcap`.
2. `process_packet` has completed payload insertion and flushing.

## Postconditions

1. `evict_flows` closes flows with `CloseReason::MemoryPressure` until `total_memory <=
   memcap` AND `flows.len() <= max_flows`.
2. `stats.evictions` increments by the number of flows evicted.
3. After eviction, `total_memory <= memcap` (assuming at least one flow exists to evict).
4. If all flows are evicted and total_memory is still > memcap (impossible in practice but
   theoretically: segment just inserted is not yet in a flow), the loop terminates with no
   more candidates.

## Invariants

1. The memcap check fires AFTER flush (bytes just inserted may have been partially flushed
   already; the remaining buffered bytes are what triggers the check).
2. The memcap check uses strict `>`, not `>=`. At exactly `memcap` bytes, no eviction occurs.
3. The eviction loop terminates when either condition is satisfied or candidates list is
   exhausted.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | total_memory == memcap exactly | No eviction (> not >=) |
| EC-002 | total_memory == memcap + 1 | Eviction triggered |
| EC-003 | Single large segment pushes total_memory above memcap | Eviction triggered; flow(s) evicted until under cap |
| EC-004 | Eviction brings total_memory to exactly memcap | Loop stops (condition now false) |
| EC-005 | No flows to evict but still over memcap | Loop exits; total_memory stays over cap; processing continues |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| memcap=1000; insert 1001 bytes across 2 flows | evict_flows called; at least one flow evicted; evictions++ | happy-path |
| memcap=1000; insert exactly 1000 bytes | No eviction | edge-case |
| memcap=1000; insert 1001 bytes; only one flow | That flow evicted; total_memory drops to 0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | total_memory <= memcap after eviction (when flows exist to evict) | unit: configure small memcap; insert large segment |
| — | Eviction uses MemoryPressure reason | unit: capture on_flow_close reason during eviction |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- memcap eviction bounds total memory use, a core resource constraint |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:176-179, memcap check; lifecycle.rs:67-92, evict_flows) |
| Stories | STORY-020 |
| Origin BC | BC-RAS-016 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.015 -- composes with (same evict_flows; max_flows trigger)
- BC-2.04.017 -- composes with (eviction sort order)
- BC-2.04.014 -- depends on (total_memory is the trigger metric)

## Architecture Anchors

- `src/reassembly/mod.rs:176-179` -- memcap check and evict_flows call
- `src/reassembly/lifecycle.rs:67-92` -- evict_flows implementation

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:176-179` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.total_memory > self.config.memcap` after flush

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.total_memory, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callbacks) |

## Refactoring Notes

No refactoring needed. The two eviction trigger sites (max_flows, memcap) are cleanly
separated and both delegate to the same evict_flows function.
