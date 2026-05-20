---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/lifecycle.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.017: Eviction Sort -- Non-Established First, Then Oldest-Last-Seen

## Description

`evict_flows` builds a candidate list from all current flows and sorts it by a two-key
comparator: first by `is_established` (false < true, so non-Established flows sort FIRST),
then within each group by `last_seen` (ascending, so the oldest-idle-first). This sort
determines eviction order. The loop closes flows from the head of the sorted list until both
termination conditions are met. The sort is computed fresh on every `evict_flows` call.

## Preconditions

1. `evict_flows` is called (from either memcap or max_flows pressure path).
2. `self.flows` has at least one entry.

## Postconditions

1. The candidate vec is sorted: `(is_established=false, last_seen=T)` entries come before
   `(is_established=true, last_seen=T)` entries.
2. Within non-Established: sorted by `last_seen` ascending (oldest first).
3. Within Established: sorted by `last_seen` ascending (oldest first).
4. Eviction iterates from index 0 (highest priority to evict) forward.

## Invariants

1. The sort comparator: `a.1.cmp(&b.1).then(a.2.cmp(&b.2))` where field 1 is `is_established`
   (bool) and field 2 is `last_seen` (u32). `false.cmp(&true) == Less` so non-Established
   sorts first.
2. The sort allocates a fresh Vec on every call -- this is not cached.
3. All states except `FlowState::Established` are treated as "non-Established" for the sort.
   This includes New, SynSent, Closing, and Closed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two flows: Established(last_seen=5) and New(last_seen=10) | New evicted first (non-Established wins over Established even if newer) |
| EC-002 | Two Established flows with different last_seen | Older one evicted first |
| EC-003 | Two New flows with same last_seen | Arbitrary order (stable sort is not guaranteed; both are equally eligible) |
| EC-004 | Single Closing flow + single Established flow | Closing evicted first (non-Established) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| [Established(t=1), New(t=10)] | New evicted first | happy-path |
| [Established(t=1), Established(t=2)] | t=1 evicted first | happy-path |
| [New(t=5), Established(t=1)] | New evicted first despite older Established | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Non-Established always evicted before Established | unit: mixed state flows at capacity; assert first eviction is non-Established |
| VP-TBD | Within group: oldest last_seen evicted first | unit: two same-state flows; different last_seen |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- eviction ordering policy is part of the memory management contract |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/lifecycle.rs:78-84, sort comparator) |
| Stories | S-TBD |
| Origin BC | BC-RAS-017 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.015 -- composes with (this sort is used in max_flows eviction)
- BC-2.04.016 -- composes with (this sort is used in memcap eviction)

## Architecture Anchors

- `src/reassembly/lifecycle.rs:78-84` -- sort_by comparator: a.1.cmp(&b.1).then(a.2.cmp(&b.2))

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/lifecycle.rs:78-84` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: explicit two-key sort comparator in source code

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads flow states (for sort); mutations happen in close_flow |
| **Deterministic** | yes (for fixed input) |
| **Thread safety** | not thread-safe |
| **Overall classification** | mixed (sort allocation + close_flow mutation) |

## Refactoring Notes

No refactoring needed. The sort logic is clear and the two-key comparator is correct.
