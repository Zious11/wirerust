---
document_type: story
story_id: "STORY-020"
epic_id: "E-2"
version: "2.2"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.014.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.015.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.016.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.017.md
input-hash: "8e77b20"
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
nfr:
  - NFR-RES-006
  - NFR-RES-008
implementation_strategy: brownfield-formalization
---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 2.1 | 2026-06-01 | story-writer | DF-SIBLING-SWEEP-001 story-body mod.rs re-anchor to HEAD e0451ef (Phase-5 anchor-class closure): AC-013 PATH 1 cite 227-232→256-261, PATH 2 cite 176-179→205-208; Architecture Mapping total_memory-insert 337-340→367-368, total_memory-flush 525-527→554-556, max_flows check 225-235→248-271, memcap check 176-179→205-208; Token Budget ranges updated; Task 8 code-review range 227-232→256-261; File Structure verify ranges updated. |
| 2.0 | 2026-05-29 | state-manager | status reconciled to completed per sprint-state.yaml (merge_commit 8cb907e wave 9); F-DRIFT3B-001/PG-W16-002. |
| 1.9 | 2026-05-26 | story-writer | Wave 9 wave-level adv pass-3 fixes (5TH CONSECUTIVE CYCLE of sibling-regression pattern — W9-D8 codification critical): F-W9P3-001 PC-N → PC-5 (resolves placeholder left by pass-2 F-W9P2-003 burst); F-W9P3-003 EC-012 memcap=12 → memcap=4 (matches actual test value; was arithmetically impossible against described 5-byte buffer). |
| 1.8 | 2026-05-26 | story-writer | Wave 9 wave-level adv pass-2 F-W9P2-003 (sibling-regression of pass-1 F-W9P1-002): added AC-014 tracing to BC-2.04.015 v1.5 PC-7 + BC-2.04.016 sibling PC (data-loss-on-MemoryPressure-eviction); added EC-012 to Edge Cases for the canonical 5+5 byte test vector; AC-014 test target name aligned with parallel test-writer dispatch. |
| 1.7 | 2026-05-26 | story-writer | Wave 9 wave-level adv pass-1 F-W9P1-003 (sibling-discipline regression in spec hierarchy): Architecture Mapping anchor `lifecycle.rs:51` → `lifecycle.rs:60` (line 51 is capture, line 60 is decrement; STORY-019 inserted let-else at 42-50 shifting decrement down). Resolves W9-D9 deferral by being explicit at the spec/BC level rather than just drift-item-tracked. Also swept and corrected two secondary stale anchors: Token Budget table and File Structure Requirements both cited `lifecycle.rs:51` — updated to `lifecycle.rs:60`. |
| 1.6 | 2026-05-26 | story-writer | Wave 9 Ph3 STORY-020 adv pass-5 fix: F-PASS5-003 (HIGH) — revised EC-005 row to remove false 'protects Established sessions' claim (test setup is SynSent flow, not Established); clarified that dual-conjunction termination is state-independent (mechanical); preserved general DESIGN INTENT reference per BC-2.04.015 Inv 4. |
| 1.5 | 2026-05-26 | story-writer | Wave 9 Ph3 STORY-020 adv pass-4 fixes: F-PASS4-001 (HIGH, sibling-regression of pass-3 F-005) — story line 84 stale test name updated to renamed function; LOW — Task #5 added 'Closed' state to AC-012 mix; Task #8 reworded for PATH 1 code-review vs PATH 2 behavioral-test bifurcation per v1.4 AC-013 |
| 1.4 | 2026-05-26 | story-writer | Wave 9 Ph3 STORY-020 adv pass-3 fix: F-002 (HIGH) — AC-005 re-scoped to honestly describe the no-op-eviction case; added NOTE acknowledging that 'evict_flows runs+still-full' is structurally unreachable under v1.3 Inv 4; F-001 (HIGH) — AC-013 wording revised to acknowledge PATH 1 structural verification (code review at mod.rs:227-232) vs PATH 2 behavioral verification; F-006 (MED, sibling-regression of pass-2 F-002) — Task 7 wording aligned with AC-005 v1.4 phrasing |
| 1.3 | 2026-05-26 | story-writer | Wave 9 Ph3 STORY-020 adv pass-2 fix: F-002 (HIGH) — clarified AC-005 wording to make 'after eviction' explicit (eviction may be a no-op per v1.3 Inv 4); AC-005 + EC-005 jointly characterize rejection path (EC-005: no-op case; AC-005: structural rejection). F-003 (HIGH) requires test rewrite (test-writer parallel), not story revision; AC-013 wording unchanged. |
| 1.2 | 2026-05-26 | story-writer | Wave 9 Ph3 STORY-020 adv-prep fix: revised EC-005 to match BC-2.04.015 v1.3 PC-5 implementation reality (evict_flows is no-op at max_flows without memcap pressure; packet dropped, not evicted); added EC-011 for dual-pressure case; AC-005 already consistent. Coordinated with BC-2.04.015 v1.3 EC-004 revision in same burst. |
| 1.1 | 2026-05-21 | story-writer | Initial release |

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
- When `self.flows.len() >= config.max_flows` and a new flow packet arrives, `evict_flows` is called. Under v1.3 Invariant 4 dual-conjunction termination, `evict_flows` exits as a no-op when only max_flows pressure exists (no memcap pressure). After the no-op eviction, the table remains at capacity, `get_or_create_flow` returns `false`, and the packet is dropped.
NOTE: The 'evict_flows runs and frees ≥1 slot but table still at capacity' scenario is structurally unreachable under v1.3 Invariant 4 — once memcap pressure exists to permit Established eviction, successful eviction brings total ≤ memcap (PC-5) and reduces flows.len, so the post-eviction-still-full case cannot be constructed. AC-005 and EC-005 jointly cover the only reachable rejection-after-evict_flows path: the no-op case.
- **Test:** `test_BC_2_04_015_new_flow_dropped_after_no_op_eviction_under_max_flows_only_pressure()`

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
- Both eviction triggers reach the same `evict_flows` function:
  - max_flows trigger via `get_or_create_flow` (src/reassembly/mod.rs:256-261) is verified by code review — the unconditional `self.evict_flows(handler)` call at this line is the witness; the call is structurally observable in source but not behaviorally observable when evict_flows exits as a no-op (PATH 1 / no-op case).
  - memcap trigger via `process_packet` (src/reassembly/mod.rs:205-208) is verified by test — when total > memcap, evict_flows is called and (under valid configs) evicts at least one flow, witnessed via CloseReason::MemoryPressure on close_events (PATH 2 / observable case).
  The test (`test_BC_2_04_015_both_eviction_paths_use_same_function`) verifies PATH 2 behaviorally; PATH 1 is verified by code review against the cited source line.
- **Test:** `test_BC_2_04_015_both_eviction_paths_use_same_function()`

### AC-014 (traces to BC-2.04.015 PC-7 + BC-2.04.016 PC-5)

When a flow with both contiguous head-of-buffer bytes and non-contiguous buffered
segments is evicted via CloseReason::MemoryPressure (either max_flows or memcap trigger),
`handler.on_data` is called for the contiguous prefix only; the non-contiguous segments
are silently discarded (no on_data callback fires for them, no error returned).

This applies identically to BC-2.04.015 (max_flows trigger) and BC-2.04.016 (memcap
trigger) since both call the same `evict_flows → close_flow(_, MemoryPressure, _)`
codepath.

**Test:** `test_BC_2_04_015_pc7_non_contiguous_segments_discarded_on_memory_pressure_eviction()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| total_memory increment on insert | src/reassembly/mod.rs:367-368 | effectful-shell (mutates self.total_memory) |
| total_memory decrement on flush | src/reassembly/mod.rs:554-556 | effectful-shell |
| total_memory decrement on close | src/reassembly/lifecycle.rs:60 | effectful-shell |
| evict_flows (sort + close loop) | src/reassembly/lifecycle.rs:67-92 | effectful-shell (sort alloc + close callbacks) |
| get_or_create_flow max_flows check | src/reassembly/mod.rs:248-271 | effectful-shell |
| memcap check in process_packet | src/reassembly/mod.rs:205-208 | effectful-shell |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Insert segment, flush immediately; total_memory | Increments then returns to 0 |
| EC-002 | Close flow with buffered data | total_memory -= all buffered bytes in both directions |
| EC-003 | Zero-length segment insert | total_memory unchanged (empty data early return) |
| EC-004 | All flows are Established at eviction time | LRU Established flows evicted (oldest first) |
| EC-005 | Single flow in table (`max_flows=1`, `total_memory <= memcap`), new SYN arrives | evict_flows exits immediately (both PC-5 termination conditions already satisfied); packet dropped, no flow created. max_flows-only pressure without memcap pressure is insufficient to trigger eviction; the existing flow is preserved (regardless of flow state — the dual-conjunction termination is mechanical, not state-specific). Per BC-2.04.015 v1.3 Invariant 4 DESIGN INTENT, this protection has its most salient application to Established sessions (which under pure LRU would be evictable by flow-count pressure alone). |
| EC-006 | total_memory == memcap exactly | No eviction triggered (strict `>`) |
| EC-007 | total_memory == memcap + 1 | Eviction triggered |
| EC-008 | evict_flows with Closing flow + Established flow | Closing (non-Established) evicted first |
| EC-009 | All flows evicted but still over memcap | Loop exits; total_memory stays over cap; processing continues |
| EC-010 | finalize closes all flows | total_memory reaches 0 after finalize |
| EC-011 | Single flow with buffered data > memcap (`max_flows=1`, `total_memory > memcap`), new SYN arrives | Dual pressure triggers eviction; existing flow evicted via CloseReason::MemoryPressure; new flow created |
| EC-012 | Flow with 5 contiguous + 5 non-contiguous bytes; memcap=4; eviction triggers | handler.on_data delivers first 5 bytes only; non-contiguous bytes [10..15) silently discarded; CloseReason::MemoryPressure emitted (data-loss-on-eviction documented in BC-2.04.015 PC-7 + BC-2.04.016 sibling PC) |

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
| src/reassembly/mod.rs (total_memory sites ~367-368, ~554-556, memcap check ~205-208, get_or_create_flow ~248-271) | ~2,000 |
| src/reassembly/lifecycle.rs (evict_flows ~67-92, total_memory decrement ~60) | ~1,500 |
| Test files | ~4,500 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~17,000** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 14 ACs in `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Add proptest for `total_memory == sum(flow.memory_used())` over random insert/flush/close sequences (AC-004)
5. [ ] Build eviction order test: mix New, SynSent, Closing, Closed, Established flows with varying last_seen; assert first eviction picks non-Established with lowest last_seen
6. [ ] Test memcap boundary: `total_memory == memcap` (no eviction) vs `memcap + 1` (eviction triggers)
7. [ ] Test `get_or_create_flow` returns false when `evict_flows` is a no-op (max_flows-only pressure, total_memory ≤ memcap) per BC-2.04.015 v1.3 Invariant 4 (EC-005 no-op rejection scenario; AC-005 also exercises this — see AC-005 note re: 'structural rejection' being unreachable)
8. [ ] Verify PATH 1 entry point structurally (code review at mod.rs:256-261) and PATH 2 behaviorally (CloseReason::MemoryPressure emission)
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
| `src/reassembly/mod.rs` | verify (lines 205-208, 248-271, 367-368, 554-556) | memcap check, max_flows check, total_memory increment/decrement sites |
| `src/reassembly/lifecycle.rs` | verify (lines 60, 67-92) | total_memory close decrement, evict_flows sort + loop |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-014 |
