---
document_type: story
story_id: "STORY-020"
epic_id: "E-2"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.014.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.015.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.016.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.017.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-019]
blocks: [STORY-021]
behavioral_contracts: [BC-2.04.014, BC-2.04.015, BC-2.04.016, BC-2.04.017]
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 9
target_module: reassembly
subsystems: [SS-04]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-verify
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-020`

# STORY-020: Memory Management — total_memory Accounting and LRU Eviction Policies

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to maintain an accurate cross-flow `total_memory` counter that drives eviction when either the `memcap` is exceeded or the `max_flows` limit is hit, using a deterministic LRU non-Established-first eviction order that protects active sessions from premature eviction
- **So that** the engine never exceeds configurable memory and flow-count bounds, and the eviction policy preferentially discards incomplete or idle flows before disrupting established sessions

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.014 | total_memory Tracks Buffered Bytes; Decrements on Flush and Close | Cross-flow memory aggregate accounting |
| BC-2.04.015 | Flow Eviction on max_flows Hit Uses LRU Non-Established-First | max_flows eviction trigger and strategy |
| BC-2.04.016 | Memory Pressure Eviction When total_memory Exceeds memcap | memcap eviction trigger |
| BC-2.04.017 | Eviction Sort — Non-Established First, Then Oldest-Last-Seen | Sort comparator for eviction ordering |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.014 postcondition 1)
- After inserting N bytes into a flow direction's buffer, `total_memory` increases by exactly N.
- **Test:** `test_BC_2_04_014_total_memory_increments_on_insert()`

### AC-002 (traces to BC-2.04.014 postcondition 2)
- After `flush_contiguous` delivers M bytes to the handler, `total_memory` decreases by exactly M.
- **Test:** `test_BC_2_04_014_total_memory_decrements_on_flush()`

### AC-003 (traces to BC-2.04.014 postcondition 3)
- After `close_flow` removes a flow, `total_memory` decreases by the flow's `memory_used()` at removal time (all remaining buffered bytes in both directions).
- **Test:** `test_BC_2_04_014_total_memory_decrements_on_close()`

### AC-004 (traces to BC-2.04.014 postcondition 4 and invariant 2)
- At all times, `total_memory == sum(flow.memory_used() for all flows in self.flows)`. This debug invariant holds after inserts, flushes, and closes.
- **Test:** `test_BC_2_04_014_total_memory_equals_sum_of_flow_memory()`

### AC-005 (traces to BC-2.04.015 postcondition 5-6)
- When `self.flows.len() >= config.max_flows` and a new flow packet arrives, `evict_flows` is called. After eviction, if the table is STILL at capacity, `get_or_create_flow` returns `false` and the packet is dropped (no flow created).
- **Test:** `test_BC_2_04_015_new_flow_dropped_when_table_full_after_eviction()`

### AC-006 (traces to BC-2.04.015 postcondition 1 and 3)
- Non-Established flows (state != Established) are evicted before Established flows, regardless of their `last_seen` timestamps. `stats.evictions` increments by the number of flows evicted.
- **Test:** `test_BC_2_04_015_non_established_evicted_before_established()`

### AC-007 (traces to BC-2.04.015 postcondition 4)
- Each evicted flow triggers `handler.on_flow_close(key, CloseReason::MemoryPressure)`.
- **Test:** `test_BC_2_04_015_evicted_flow_receives_memory_pressure_reason()`

### AC-008 (traces to BC-2.04.016 postcondition 1)
- After each packet, if `self.total_memory > self.config.memcap`, `evict_flows` is called. After eviction, `total_memory <= memcap` (when at least one flow exists to evict).
- **Test:** `test_BC_2_04_016_memcap_eviction_triggers_after_insert()`

### AC-009 (traces to BC-2.04.016 invariant 2)
- The memcap check uses strict `>` (not `>=`): at exactly `memcap` bytes in `total_memory`, no eviction occurs.
- **Test:** `test_BC_2_04_016_no_eviction_at_exactly_memcap()`

### AC-010 (traces to BC-2.04.017 postcondition 1-4)
- In `evict_flows`, the sort places all non-Established flows (New, SynSent, Closing, Closed) before all Established flows. Within each group, flows are sorted by `last_seen` ascending (oldest first).
- **Test:** `test_BC_2_04_017_eviction_sort_non_established_first_then_lru()`

### AC-011 (traces to BC-2.04.017 edge case EC-001)
- A non-Established flow with a NEWER `last_seen` timestamp is evicted before an Established flow with an OLDER `last_seen` timestamp (non-Established wins regardless of recency).
- **Test:** `test_BC_2_04_017_non_established_newer_evicted_before_established_older()`

### AC-012 (traces to BC-2.04.017 invariant 3)
- The eviction sort treats ALL states other than `FlowState::Established` as "non-Established": `New`, `SynSent`, `Closing`, and `Closed` all sort before any Established flow.
- **Test:** `test_BC_2_04_017_all_non_established_states_evict_first()`

### AC-013 (traces to BC-2.04.015 invariant 1)
- Both eviction triggers (max_flows via `get_or_create_flow` and memcap via `process_packet`) call the same `evict_flows` function with the same LRU non-established-first strategy.
- **Test:** `test_BC_2_04_015_both_eviction_paths_use_same_function()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| total_memory increment on insert | src/reassembly/mod.rs:337-340 | effectful-shell (mutates self.total_memory) |
| total_memory decrement on flush | src/reassembly/mod.rs:525-527 | effectful-shell |
| total_memory decrement on close | src/reassembly/lifecycle.rs:51 | effectful-shell |
| evict_flows (sort + close loop) | src/reassembly/lifecycle.rs:67-92 | effectful-shell (sort alloc + close callbacks) |
| get_or_create_flow max_flows check | src/reassembly/mod.rs:225-235 | effectful-shell |
| memcap check in process_packet | src/reassembly/mod.rs:176-179 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Insert segment, flush immediately; total_memory | Increments then returns to 0 |
| EC-002 | Close flow with buffered data | total_memory -= all buffered bytes in both directions |
| EC-003 | Zero-length segment insert | total_memory unchanged (empty data early return) |
| EC-004 | All flows are Established at eviction time | LRU Established flows evicted (oldest first) |
| EC-005 | Single flow in table at max_flows=1, new SYN arrives | Existing flow evicted; new flow created |
| EC-006 | total_memory == memcap exactly | No eviction triggered (strict `>`) |
| EC-007 | total_memory == memcap + 1 | Eviction triggered |
| EC-008 | evict_flows with Closing flow + Established flow | Closing (non-Established) evicted first |
| EC-009 | All flows evicted but still over memcap | Loop exits; total_memory stays over cap; processing continues |
| EC-010 | finalize closes all flows | total_memory reaches 0 after finalize |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/mod.rs (total_memory sites, memcap check) | effectful-shell | Mutates self.total_memory, self.flows, self.stats |
| src/reassembly/lifecycle.rs (evict_flows) | effectful-shell | Allocates sort Vec, calls close_flow, invokes callbacks |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (4 BCs) | ~5,000 |
| src/reassembly/mod.rs (total_memory sites ~337-340, ~525-527, memcap check ~176-179, get_or_create_flow ~225-235) | ~2,000 |
| src/reassembly/lifecycle.rs (evict_flows ~67-92, total_memory decrement ~51) | ~1,500 |
| Test files | ~4,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~17,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 13 ACs in `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Add proptest for `total_memory == sum(flow.memory_used())` over random insert/flush/close sequences (AC-004)
5. [ ] Build eviction order test: mix New, SynSent, Closing, Established flows with varying last_seen; assert first eviction picks non-Established with lowest last_seen
6. [ ] Test memcap boundary: `total_memory == memcap` (no eviction) vs `memcap + 1` (eviction triggers)
7. [ ] Test `get_or_create_flow` returns false when table still full after eviction (EC-005 full rejection scenario)
8. [ ] Verify both eviction trigger sites reach `evict_flows` (AC-013)
9. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-019 | close_flow removes flows from HashMap; total_memory adjusted in lifecycle.rs | on_flow_close callback order: flush then close | CLOSE_FLOW_MISSING_WARNED is process-wide |
| STORY-016 | buffered_bytes is the per-direction byte counter | buffered_bytes mirrors segment sum (BC-2.04.047) | buffered_bytes is not the same as total_memory |
| STORY-015 | flush_contiguous decrements buffered_bytes | bytes_reassembled tracks flushed bytes | total_memory must be decremented at the same site as buffered_bytes |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| memcap check fires AFTER flush (not before) | BC-2.04.016 invariant 1 | Code review: check order in process_packet ~176-179 |
| memcap check uses strict `>` (not `>=`) | BC-2.04.016 invariant 2 | Test: AC-009 boundary test at exactly memcap |
| Eviction sort comparator: `a.1.cmp(&b.1).then(a.2.cmp(&b.2))` where field 1=is_established, field 2=last_seen | BC-2.04.017 invariant 1 | Code review: lifecycle.rs:78-84 |
| All states except Established are "non-Established" in sort (includes New, SynSent, Closing, Closed) | BC-2.04.017 invariant 3 | Test: AC-012 covers all four non-Established states |
| evict_flows terminates when `flows.len() <= max_flows AND total_memory <= memcap` | BC-2.04.015 postcondition 5 | Code review: loop termination condition |
| `CloseReason::MemoryPressure` is used for all eviction closes | BC-2.04.015 postcondition 4 | Test: capture on_flow_close reason during eviction |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | HashMap, Vec::sort_by, usize arithmetic |
| proptest | from Cargo.toml | Property-based total_memory invariant test |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/mod.rs` | verify (lines 176-179, 225-235, 337-340, 525-527) | memcap check, max_flows check, total_memory increment/decrement sites |
| `src/reassembly/lifecycle.rs` | verify (lines 51, 67-92) | total_memory close decrement, evict_flows sort + loop |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-013 |
