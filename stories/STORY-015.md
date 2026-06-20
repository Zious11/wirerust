---
document_type: story
story_id: "STORY-015"
epic_id: "E-2"
version: "1.3"
status: completed
producer: story-writer
timestamp: 2026-06-08T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.006.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.007.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.008.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.033.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.034.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.039.md
input-hash: "f41080c"
traces_to: .factory/specs/prd.md
points: 8
depends_on: [STORY-013, STORY-014]
blocks: [STORY-016, STORY-017, STORY-018]
behavioral_contracts: [BC-2.04.006, BC-2.04.007, BC-2.04.008, BC-2.04.033, BC-2.04.034, BC-2.04.039]
verification_properties: [VP-011, VP-015]
priority: "P0"
cycle: v0.1.0-greenfield-spec
wave: 8
target_module: reassembly
subsystems: [SS-04]
estimated_days: 2
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
nfr:
  - NFR-REL-002
  - NFR-REL-003
implementation_strategy: brownfield-formalization
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-015`

# STORY-015: In-Order Delivery, Out-of-Order Buffering, and Bidirectional Direction Tagging

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to deliver reassembled data to protocol analyzers in correct ISN-relative order, buffer out-of-order segments until gaps are filled, tag data with the correct direction, and correctly handle TCP sequence number wraparound
- **So that** protocol analyzers receive complete application-layer payloads in the right order with accurate ClientToServer/ServerToClient labels

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.006 | Bidirectional Data Delivered with Correct Direction Tag | Direction tagging in on_data callback |
| BC-2.04.007 | In-Order Data Flushes Contiguously to Handler | flush_contiguous_data contract |
| BC-2.04.008 | Out-of-Order Segments Buffer Until Gap Filled Then Flush | OOO buffering and gap-fill flush |
| BC-2.04.033 | Single Segment Insertion Returns Inserted; Stored Under Offset Key | Base-case segment insertion |
| BC-2.04.034 | flush_contiguous Consumes Segments from base_offset in Order | flush_contiguous implementation |
| BC-2.04.039 | TCP Sequence Wraparound Across 32-bit Boundary Reassembles Correctly | Wraparound arithmetic |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.006 postcondition 1)
- `handler.on_data` is called with `direction == ClientToServer` for bytes originating from the initiator endpoint.
- **Test:** `test_BC_2_04_006_client_to_server_data_tagged_correctly()`

### AC-002 (traces to BC-2.04.006 postcondition 2)
- `handler.on_data` is called with `direction == ServerToClient` for bytes originating from the responder endpoint.
- **Test:** `test_BC_2_04_006_server_to_client_data_tagged_correctly()`

### AC-003 (traces to BC-2.04.006 postcondition 3)
- The `offset` parameter in each `on_data` call is the ISN-relative stream offset of the first byte of that chunk.
- **Test:** `test_BC_2_04_006_on_data_offset_is_isn_relative()`

### AC-004 (traces to BC-2.04.006 postcondition 4)
- `stats.bytes_reassembled` increments by the total bytes across all `on_data` calls in both directions.
- **Test:** `test_BC_2_04_006_bytes_reassembled_counts_both_directions()`

### AC-005 (traces to BC-2.04.006 invariant 2)
- Client-to-server and server-to-client buffers are fully independent; flushing one direction never drains the other.
- **Test:** `test_BC_2_04_006_directions_are_independent()`

### AC-006 (traces to BC-2.04.007 postcondition 1-2)
- When a segment arrives at exactly `base_offset`, `flush_contiguous_data` removes all contiguous segments starting from `base_offset` and delivers them via `on_data`. `base_offset` advances by the total bytes flushed.
- **Test:** `test_BC_2_04_007_in_order_segment_flushed_immediately()`

### AC-007 (traces to BC-2.04.007 invariant 1)
- Segments stored at offsets beyond the first gap are NOT flushed; only the contiguous prefix from `base_offset` is consumed.
- **Test:** `test_BC_2_04_007_gap_halts_flush()`

### AC-008 (traces to BC-2.04.007 invariant 3)
- `base_offset` is monotonically non-decreasing; it never decreases.
- **Test:** `test_BC_2_04_007_base_offset_is_monotonic()`

### AC-009 (traces to BC-2.04.008 postcondition 1-4)
- When a segment arrives ahead of `base_offset` (gap), it is stored in the BTreeMap but NOT delivered to the handler. `buffered_bytes` increases by the segment length. `on_data` is NOT called.
- **Test:** `test_BC_2_04_008_out_of_order_segment_buffered_not_delivered()`

### AC-010 (traces to BC-2.04.008 postcondition 5)
- When a later segment fills the gap, `flush_contiguous` delivers both the fill segment and all previously-buffered contiguous segments in ISN-relative order.
- **Test:** `test_BC_2_04_008_gap_fill_delivers_all_contiguous()`

### AC-011 (traces to BC-2.04.033 postcondition 1-2)
- When a non-overlapping, in-window segment is inserted into an empty buffer, `insert_segment` returns `InsertResult::Inserted` and stores the segment under its ISN-relative offset key.
- **Test:** `test_BC_2_04_033_single_segment_insert_returns_inserted()`

### AC-012 (traces to BC-2.04.033 postcondition 3)
- `buffered_bytes` increases by `data.len()` after a successful single-segment insertion.
- **Test:** `test_BC_2_04_033_buffered_bytes_increments_after_insert()`

### AC-013 (traces to BC-2.04.034 postcondition 2-3)
- `flush_contiguous()` decrements `buffered_bytes` and advances `base_offset` by exactly the total flushed bytes.
- **Test:** `test_BC_2_04_034_flush_contiguous_accounting()`

### AC-014 (traces to BC-2.04.034 postcondition 4)
- When no segment exists at `base_offset`, `flush_contiguous()` returns an empty Vec and does not modify `base_offset`.
- **Test:** `test_BC_2_04_034_flush_contiguous_empty_when_no_segment_at_base()`

### AC-015 (traces to BC-2.04.034 postcondition 3)
- `flush_contiguous()` returns segments in ascending offset order.
- **Test:** `test_BC_2_04_034_flush_contiguous_returns_ordered_segments()`

### AC-016 (traces to BC-2.04.039 postcondition 1)
- `seq.wrapping_sub(isn) as u64` correctly computes the monotonically-increasing byte offset even when the TCP sequence number wraps around `u32::MAX`. Specifically, ISN near `u32::MAX` and subsequent segments with seq < ISN (due to wraparound) produce correct, increasing u64 offsets.
- **Test:** `test_BC_2_04_039_sequence_wraparound_correct_offsets()`

### AC-017 (traces to BC-2.04.039 postcondition 3)
- After wraparound, `flush_contiguous` delivers wrapped segments in the correct byte order.
- **Test:** `test_BC_2_04_039_flush_delivers_wrapped_segments_in_order()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| FlowDirection::insert_segment | src/reassembly/segment.rs | pure-core |
| FlowDirection::flush_contiguous | src/reassembly/segment.rs | pure-core |
| flush_contiguous_data | src/reassembly/mod.rs | effectful-shell (callback invocation) |
| seq_offset | src/reassembly/segment.rs | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Segment arrives in-order (no gap) | Immediately flushed; no buffering |
| EC-002 | Gap exists before base_offset | flush stops at gap; only prefix delivered |
| EC-003 | Out-of-order segment fills a gap | Next flush delivers buffered segments plus new one |
| EC-004 | Empty payload (pure ACK) | Engine guard skips empty payloads before insert |
| EC-005 | Multiple contiguous segments flushed in one call | Delivered as separate on_data calls (one per segment) |
| EC-006 | Three-segment out-of-order sequence (3,2,1) | 3,2 buffered; 1 arrives; all three flushed in order |
| EC-007 | Gap never filled (flow closed) | close_flow::flush_contiguous delivers up to gap |
| EC-008 | ISN near u32::MAX; segments wrap around | All offsets correct via wrapping_sub; BTreeMap keys monotonic |
| EC-009 | Empty segments BTreeMap; flush_contiguous called | Returns empty Vec; base_offset unchanged |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/segment.rs | pure-core | No I/O; purely in-memory BTreeMap operations |
| src/reassembly/mod.rs (flush_contiguous_data) | effectful-shell | Invokes handler.on_data callback; mutates stats |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (6 BCs) | ~6,000 |
| src/reassembly/segment.rs (insert + flush ~lines 31-248) | ~2,500 |
| src/reassembly/mod.rs (flush_contiguous_data ~lines 517-533) | ~800 |
| Test files | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~17,300** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~8.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 17 ACs in `tests/reassembly_segment_tests.rs` and `tests/reassembly_engine_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Add property-based test: `base_offset` never decreases for any sequence of inserts/flushes (AC-008)
5. [ ] Add explicit wraparound test with ISN=u32::MAX-2 and segments at seq=u32::MAX-1 and seq=0 (AC-016, AC-017)
6. [ ] Verify `reassembled_bytes` increments in flush_contiguous (separate from `base_offset`)
7. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-013 | State machine tests live in reassembly_flow_tests.rs | Engine-level integration in reassembly_engine_tests.rs | SYN+ACK sets initiator to DST (not SRC) |
| STORY-014 | ISN inference uses wrapping_sub(1) | infer_isn is idempotent; set_isn is idempotent | ISN_MISSING_WARNED is process-wide — test isolation needed |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| `seq.wrapping_sub(isn) as u64` for sequence arithmetic | BC-2.04.039 invariant 1 | Code review: grep for wrapping_sub in segment.rs |
| BTreeMap for segment storage (guarantees ordered iteration) | BC-2.04.034 invariant (BTreeMap key ordering) | Type check: `BTreeMap<u64, Vec<u8>>` in FlowDirection |
| `base_offset` never decreases | BC-2.04.007 invariant 3; BC-2.04.034 invariant 3 | proptest: monotonicity assertion |
| No `unsafe` blocks | prd.md §1.2 | cargo clippy |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | BTreeMap, wrapping arithmetic |
| proptest | from Cargo.toml | Property-based tests for monotonicity |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/segment.rs` | verify (lines 31-248) | seq_offset, insert_segment, flush_contiguous |
| `src/reassembly/mod.rs` | verify (lines 517-533) | flush_contiguous_data with on_data callback |
| `tests/reassembly_segment_tests.rs` | modify | Add AC-011 through AC-017 (segment-level tests) |
| `tests/reassembly_engine_tests.rs` | modify | Add AC-001 through AC-010 (engine-level tests) |

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| v1.1 | 2026-05-21 | story-writer | Initial brownfield story release |
| v1.2 | 2026-05-26 | story-writer | Wave 8 wave-level adv-pass-3 F-3 closure: bumped `status: draft` → `in_progress` to match STORY-019's transient-state convention (S-7.01 sibling-discipline). State-manager will advance both to `done` on Wave 8 close (W8.12). |
