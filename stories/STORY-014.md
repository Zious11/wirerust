---
document_type: story
story_id: "STORY-014"
epic_id: "E-2"
version: "1.2"
status: in_progress
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.009.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.031.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.032.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.048.md
input-hash: "51df820"
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-013]
blocks: [STORY-015, STORY-019]
behavioral_contracts: [BC-2.04.009, BC-2.04.031, BC-2.04.032, BC-2.04.048]
verification_properties: []
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 7
target_module: reassembly
subsystems: [SS-04]
estimated_days: 1
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-014`

# STORY-014: Mid-Stream Join, ISN Management, and IsnMissing Guard

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to handle captures that start mid-connection by inferring the ISN from the first data packet's sequence number, mark such flows as partial, and safely handle the programming-error case where a segment arrives before any ISN is set
- **So that** forensic analysis can proceed on partial captures without silent data corruption, and programming errors in ISN setup are detected early

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.009 | Mid-Stream Join Infers ISN from seq-1; Flow Marked Partial | Engine-level mid-stream join path |
| BC-2.04.031 | ISN Set on First SYN; Inferred as seq-1 on Data-Without-SYN | FlowDirection ISN management |
| BC-2.04.032 | insert_segment With No ISN Returns IsnMissing; Inserts Nothing | Programming-error defensive guard |
| BC-2.04.048 | ISN_MISSING_WARNED Atomic Prevents Repeated eprintln | One-shot warning pattern |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.009 postcondition 1-2, postcondition 3)
- When a data packet arrives for a flow in `FlowState::New`, the flow state transitions to `Established` and `flow.partial == true`. Additionally, the first emitted data event has `Direction::ClientToServer` — observable proof that BC-2.04.009 PC3 sets the initiator to the packet's source.
- **Test:** `test_BC_2_04_009_mid_stream_sets_established_partial()`

### AC-002 (traces to BC-2.04.009 postcondition 4)
- When a data packet arrives on a `New` flow, the direction for `src_ip:src_port` has `isn == Some(tcp.seq.wrapping_sub(1))`.
- **Test:** `test_BC_2_04_009_mid_stream_isn_is_seq_minus_one()`

### AC-003 (traces to BC-2.04.009 postcondition 5)
- After mid-stream ISN inference, `flow.client_to_server.base_offset == 1`.
- **Test:** `test_BC_2_04_009_mid_stream_base_offset_is_one()`

### AC-004 (traces to BC-2.04.009 postcondition 6)
- `stats.flows_partial` increments by 1 for each mid-stream join flow.
- **Test:** `test_BC_2_04_009_flows_partial_increments_on_mid_stream()`

### AC-005 (traces to BC-2.04.009 invariant 1)
- `infer_isn(0)` uses `wrapping_sub(1)`, resulting in `isn = u32::MAX` without panic or incorrect behavior.
- **Test:** `test_BC_2_04_009_mid_stream_isn_wraps_correctly_at_seq_zero()`

### AC-006 (traces to BC-2.04.031 postcondition 1-2 for set_isn path)
- `set_isn(seq)` sets `self.isn = Some(seq)` and `self.base_offset = 1`.
- **Test:** `test_BC_2_04_031_set_isn_stores_isn_and_base_offset()`

### AC-007 (traces to BC-2.04.031 postcondition 1-2 for infer_isn path)
- `infer_isn(first_seq)` sets `self.isn = Some(first_seq.wrapping_sub(1))` and `self.base_offset = 1`.
- **Test:** `test_BC_2_04_031_infer_isn_stores_seq_minus_one()`

### AC-008 (traces to BC-2.04.031 postcondition 3 — both paths)
- Both `set_isn` and `infer_isn` are idempotent: a second call with a different value is a no-op; the first ISN is preserved.
- **Test:** `test_BC_2_04_031_isn_setters_are_idempotent()`

### AC-009 (traces to BC-2.04.031 invariant 2)
- `infer_isn` handles `first_seq == 0` by wrapping: ISN becomes `u32::MAX` and base_offset becomes 1.
- **Test:** `test_BC_2_04_031_infer_isn_zero_wraps_to_max()`

### AC-010 (traces to BC-2.04.032 postcondition 1)
- When `insert_segment` is called on a direction where `isn == None` with non-empty data, it returns `InsertResult::IsnMissing`.
- **Test:** `test_BC_2_04_032_isn_missing_returns_isn_missing()`

### AC-011 (traces to BC-2.04.032 postcondition 2-4)
- When `InsertResult::IsnMissing` is returned, `self.segments` is unchanged, `self.buffered_bytes` is unchanged, `self.overlap_count` is unchanged, `self.out_of_window_count` is unchanged, and no counters are modified (full BC-2.04.032 PC4 coverage).
- **Test:** `test_BC_2_04_032_isn_missing_inserts_nothing()`

### AC-012 (traces to BC-2.04.032 edge case EC-003)
- When `insert_segment` is called with an empty data slice (`data.is_empty()`) and `isn == None`, it returns `Inserted` (the empty-data early return fires before the ISN check).
- **Test:** `test_BC_2_04_032_empty_data_returns_inserted_without_isn_check()`

### AC-013 + AC-014 + EC-007 (combined — traces to BC-2.04.048 PC1, PC2; EC-007)

The three sub-assertions are combined into one test function because `ISN_MISSING_WARNED` is a process-global static and the cargo integration-test binary shares it across all tests in `reassembly_engine_tests.rs`. The combined test uses the `#[doc(hidden)] reset_isn_missing_warned_for_testing()` accessor to deterministically observe the BC-2.04.048 PC1 `false → true` swap transition.

Sub-assertions:
- **AC-013 (BC-2.04.048 PC1):** After resetting the atomic, the first call to `insert_segment` with `isn == None` flips `ISN_MISSING_WARNED` from `false` to `true`. Observed via the `isn_missing_warned_for_testing()` accessor.
- **AC-014 (BC-2.04.048 PC2, latching state):** A subsequent call with `isn == None` keeps the atomic at `true`. Observed via the accessor.
- **AC-014 (BC-2.04.048 PC2, no second eprintln):** The "no additional `eprintln!` is emitted on subsequent calls" sub-property is enforced **structurally** by the swap-guarded `if`-block in `src/reassembly/segment.rs:51-58` and verified by code review (Architecture Compliance Rule — mirrors the BC-2.04.048 invariant 3 precedent that also relies on code review). In-process stderr capture is fragile and out of scope for this story.
- **EC-007 (already-warned-at-test-start safety):** Pre-reset eliminates EC-007's "ISN_MISSING_WARNED already true on test run" concern by making the test order-independent.

- **Test:** `test_BC_2_04_048_isn_missing_warned_fires_once_then_suppressed()` (combined name; rationale documented in test doc-comment)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| infer_isn / set_isn | src/reassembly/flow.rs | pure-core |
| on_data_without_syn (mid-stream join) | src/reassembly/flow.rs | pure-core |
| insert_payload_segment (mid-stream check) | src/reassembly/mod.rs | effectful-shell |
| insert_segment IsnMissing guard | src/reassembly/segment.rs | effectful-shell (stderr write on first call) |
| ISN_MISSING_WARNED AtomicBool | src/reassembly/segment.rs | effectful-shell (global state) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Mid-stream with seq=0 | ISN=u32::MAX; base_offset=1 |
| EC-002 | Second data packet on partial flow (different direction) | set_initiator no-op; ISN inferred for s2c direction |
| EC-003 | SYN arrives after data on partial flow | set_initiator/set_isn/on_syn all no-ops (already set) |
| EC-004 | Multiple partial flows in same PCAP | flows_partial counts all independently |
| EC-005 | ISN already set via SYN; then data-without-SYN path called | All setters no-ops; state unchanged |
| EC-006 | Empty data slice with no ISN | Returns Inserted (early return before ISN check) |
| EC-007 | ISN_MISSING_WARNED already true on test run | Silent return for subsequent IsnMissing calls |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/flow.rs (set_isn, infer_isn) | pure-core | No I/O; purely in-memory |
| src/reassembly/segment.rs (IsnMissing guard) | effectful-shell | Writes to stderr on first encounter; reads global AtomicBool |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~2,500 |
| BC files (4 BCs) | ~4,000 |
| src/reassembly/flow.rs (ISN section ~lines 136-148) | ~500 |
| src/reassembly/mod.rs (mid-stream path ~lines 305-312) | ~500 |
| src/reassembly/segment.rs (ISN guard ~lines 16, 51-58) | ~400 |
| Test files | ~3,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~11,900** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~6%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 14 ACs in `tests/reassembly_flow_tests.rs` and `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Test ISN_MISSING_WARNED atomic carefully — it is process-wide; reset between tests may require separate process or careful ordering
5. [ ] Test `infer_isn(0)` edge case explicitly
6. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-011 | brownfield-verify; tests in reassembly_flow_tests.rs | FlowKey commutativity tested separately | U+2192 arrow in display is a hidden encoding contract |
| STORY-013 | State machine tests in reassembly_flow_tests.rs | on_data_without_syn is the mid-stream entry point | SYN+ACK sets initiator to DST (not SRC) |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `wrapping_sub` used for ISN arithmetic | BC-2.04.031 invariant 2; BC-2.04.009 invariant 1 | Code review: grep for wrapping_sub in flow.rs |
| ISN_MISSING_WARNED uses `Ordering::Relaxed` | BC-2.04.048 invariant 2 | Code review: AtomicBool::swap call |
| `swap(true)` pattern (not load+store) | BC-2.04.048 invariant 3 | Code review: swap-based one-shot guard |
| No `unsafe` blocks | prd.md §1.2 | cargo clippy |
| **No additional eprintln on subsequent IsnMissing calls** | BC-2.04.048 PC2 | Code review of swap-guarded if-block at `src/reassembly/segment.rs:51-58` (matches BC-2.04.048 invariant 3 precedent) |
| **`#[doc(hidden)]` on test-only accessors** | BC-2.04.048 + brownfield-formalization API hygiene | Code review: `isn_missing_warned_for_testing()` and `reset_isn_missing_warned_for_testing()` both carry `#[doc(hidden)]` |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | `wrapping_sub`, `AtomicBool`, `Ordering::Relaxed` |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/flow.rs` | verify (lines 136-148) | set_isn and infer_isn implementations |
| `src/reassembly/mod.rs` | verify (lines 305-312) | Mid-stream join handling in insert_payload_segment |
| `src/reassembly/segment.rs` | verify (lines 16, 51-58) | ISN_MISSING_WARNED AtomicBool and IsnMissing guard |
| `src/reassembly/segment.rs` | append | Added `#[doc(hidden)] pub fn isn_missing_warned_for_testing()` and `#[doc(hidden)] pub fn reset_isn_missing_warned_for_testing()` accessors at end-of-file (no anchor shift of BC anchors at :16 or :51-58) |
| `tests/reassembly_flow_tests.rs` | modify | Add AC-006 through AC-009 tests |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-005, AC-010 through AC-014 |

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.2 | 2026-05-25 | story-writer | Wave 7 Ph3 adv-pass-1 fixes: F-2 (AC-014 "no second eprintln" sub-property promoted to BC-2.04.048 Architecture Compliance Rule with code-review enforcement, mirroring BC-2.04.048 inv-3 precedent); F-3 (AC-011 amended to explicitly include `overlap_count` and `out_of_window_count` for full BC-2.04.032 PC4 coverage); F-5 (AC-013 + AC-014 + EC-007 collapsed into a single combined-test entry with documented rationale for the process-global atomic ordering constraint; combined test name `test_BC_2_04_048_isn_missing_warned_fires_once_then_suppressed()` formally recorded); F-6 (AC-001 amended to assert first data event has `Direction::ClientToServer`, providing observable proof of BC-2.04.009 PC3 initiator-set property). Added Architecture Compliance Rule rows for swap-guard code review and `#[doc(hidden)]` test-accessor hygiene. Added segment.rs to File Structure Requirements with append-only annotation. |
| 1.1 | 2026-05-21 | story-writer | Initial brownfield-formalization story with full AC traceability to BC-2.04.009, BC-2.04.031, BC-2.04.032, BC-2.04.048. |
