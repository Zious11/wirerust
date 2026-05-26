---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: Wave 9 wave-level adv pass-2 F-W9P2-002 (sibling-regression of pass-1 F-W9P1-002): added PC-5 data-delivery semantics under MemoryPressure eviction; added canonical test vector row for memcap-trigger data-loss case — 2026-05-26"
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
5. **Data-delivery semantics under CloseReason::MemoryPressure eviction:** On `CloseReason::MemoryPressure`,
   only the contiguous head-of-buffer prefix is delivered to the handler via `on_data` callback
   (via `flush_contiguous()` per direction). Non-contiguous buffered segments (segments held waiting
   on a gap fill) are DISCARDED — no `on_data` callback fires for them. The number of discarded bytes
   equals `flow.memory_used() at close time − bytes delivered by flush_contiguous()`. Forensic
   analysts should account for this potential data loss when evictions occur under memory pressure:
   an eviction-heavy capture may have non-contiguous segments dropped that would otherwise have been
   reassembled if memory budget had been larger. See BC-2.04.015 PC-7 for the full forensic-implications
   note. Both BC-2.04.015 (max_flows trigger) and BC-2.04.016 (memcap trigger) call the same
   `evict_flows → close_flow(_, MemoryPressure, _)` codepath, so the data-loss semantic is identical
   regardless of which trigger path initiated eviction.

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
| flow with 5 contiguous bytes [0..5) + 5 non-contiguous bytes [10..15) buffered (gap at [5..10)); memcap=4 so total_memory > memcap; eviction triggers via process_packet path | handler.on_data called for contiguous prefix [0..5) only; bytes [10..15) discarded silently; on_flow_close called with CloseReason::MemoryPressure | data-loss-on-memcap-eviction (sibling of BC-2.04.015 case) |

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

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-05-20 | product-owner | Initial brownfield extraction |
| 1.2 | 2026-05-21 | product-owner | VP back-reference back-fill (P8-DEFER) |
| 1.3 | 2026-05-26 | product-owner | Wave 9 wave-level adv pass-2 F-W9P2-002 (sibling-regression of pass-1 F-W9P1-002): added PC-5 documenting data-delivery semantics under MemoryPressure eviction (sibling of BC-2.04.015 v1.5 PC-7; same evict_flows codepath; non-contiguous segments discarded). Added canonical test vector row for memcap-trigger data-loss case. |
| 1.4 | 2026-05-26 | product-owner | Wave 9 wave-level adv pass-3 F-W9P3-002 (5TH CONSECUTIVE CYCLE of sibling-regression — W9-D8 codification critical): canonical test vector memcap=12 → memcap=4 (matches actual test value; was arithmetically impossible against described 5-byte buffer because contiguous prefix is flushed before memcap check). |
