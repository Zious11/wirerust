---
document_type: story
story_id: "STORY-016"
epic_id: "E-2"
version: "1.1"
status: draft
producer: story-writer
timestamp: 2026-05-21T00:00:00Z
phase: 2
inputs:
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.035.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.036.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.038.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.043.md
  - .factory/specs/behavioral-contracts/ss-04/BC-2.04.047.md
input-hash: "[md5-pending]"
traces_to: .factory/specs/prd.md
points: "8"
depends_on: [STORY-015]
blocks: [STORY-017, STORY-018]
behavioral_contracts: [BC-2.04.035, BC-2.04.036, BC-2.04.038, BC-2.04.043, BC-2.04.047]
verification_properties: [VP-002, VP-010]
priority: "P0"
cycle: v0.1.0-brownfield
wave: 4
target_module: reassembly
subsystems: [SS-04]
estimated_days: "2"
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
implementation_strategy: brownfield-verify
---

> **tdd_mode:** strict — all ACs must be backed by tests.

> **Execute:** `/vsdd-factory:deliver-story STORY-016`

# STORY-016: Overlap Detection — Duplicate Retransmissions, Partial Overlap, and buffered_bytes Accounting

## Narrative
- **As a** forensic analyst
- **I want** the TCP reassembly engine to correctly classify TCP segment overlaps — benign retransmissions (Duplicate), partial overlaps with gap-fill (PartialOverlap), and multi-segment full coverage — while maintaining accurate byte accounting and not counting adjacent segments as overlapping
- **So that** the engine correctly deduplicates normal retransmissions without false positives, fills gaps precisely with first-wins policy, and the `buffered_bytes` counter stays accurate for memcap eviction

## Behavioral Contracts

| BC | Title | Role in Story |
|----|-------|---------------|
| BC-2.04.035 | Identical Retransmission Returns Duplicate; Does Not Double-Count | Duplicate deduplication |
| BC-2.04.036 | First-Wins Overlap: Gap Bytes Added, Existing Bytes Preserved | PartialOverlap gap-fill |
| BC-2.04.038 | Multi-Segment Full Coverage Returns Duplicate or ConflictingOverlap | Union-coverage path |
| BC-2.04.043 | Adjacent Segments at Exact Boundary Do Not Count as Overlap | Adjacency boundary |
| BC-2.04.047 | buffered_bytes Mirrors Segment Size Sum After All Operations | Memory accounting invariant |

## Acceptance Criteria

### AC-001 (traces to BC-2.04.035 postcondition 1)
- When a segment is re-inserted with the same byte range and IDENTICAL bytes as an existing segment, `insert_segment` returns `InsertResult::Duplicate`.
- **Test:** `test_BC_2_04_035_identical_retransmission_returns_duplicate()`

### AC-002 (traces to BC-2.04.035 postcondition 2-3)
- After a `Duplicate` result, `self.segments` is unchanged and `self.buffered_bytes` is unchanged.
- **Test:** `test_BC_2_04_035_duplicate_does_not_change_buffer()`

### AC-003 (traces to BC-2.04.035 postcondition 4)
- `self.overlap_count` increments by 1 even for a `Duplicate` result.
- **Test:** `test_BC_2_04_035_duplicate_increments_overlap_count()`

### AC-004 (traces to BC-2.04.036 postcondition 1)
- When a segment partially overlaps existing bytes AND has gap bytes, `insert_segment` returns `InsertResult::PartialOverlap`.
- **Test:** `test_BC_2_04_036_partial_overlap_returns_partial_overlap()`

### AC-005 (traces to BC-2.04.036 postcondition 2-3)
- After `PartialOverlap`, only the gap bytes are inserted. Existing bytes at the overlapping positions are NOT modified (first-wins, INV-3).
- **Test:** `test_BC_2_04_036_partial_overlap_preserves_existing_bytes()`

### AC-006 (traces to BC-2.04.036 postcondition 4)
- After `PartialOverlap`, `buffered_bytes` increases by the total number of gap bytes inserted (not the full segment length).
- **Test:** `test_BC_2_04_036_partial_overlap_buffered_bytes_gap_only()`

### AC-007 (traces to BC-2.04.036 postcondition 5)
- `overlap_count` increments by 1 for a `PartialOverlap` result.
- **Test:** `test_BC_2_04_036_partial_overlap_increments_overlap_count()`

### AC-008 (traces to BC-2.04.038 postcondition 1)
- When the new segment's byte range is fully covered by the UNION of two or more existing segments AND all bytes match, `insert_segment` returns `InsertResult::Duplicate`.
- **Test:** `test_BC_2_04_038_multi_segment_full_coverage_matching_returns_duplicate()`

### AC-009 (traces to BC-2.04.038 postcondition 2)
- When the new segment's byte range is fully covered by the union of two or more existing segments AND at least one byte differs, `insert_segment` returns `InsertResult::ConflictingOverlap`.
- **Test:** `test_BC_2_04_038_multi_segment_full_coverage_conflicting_returns_conflicting()`

### AC-010 (traces to BC-2.04.043 postcondition 1)
- When a new segment starts exactly where an existing segment ends (`new_start == existing_end`), `insert_segment` returns `InsertResult::Inserted` (not `PartialOverlap`).
- **Test:** `test_BC_2_04_043_adjacent_segment_returns_inserted_not_overlap()`

### AC-011 (traces to BC-2.04.043 postcondition 2)
- `overlap_count` is NOT incremented for adjacent segments.
- **Test:** `test_BC_2_04_043_adjacent_segment_does_not_increment_overlap_count()`

### AC-012 (traces to BC-2.04.047 postcondition 1)
- At all times, `buffered_bytes == sum of self.segments.values().map(|v| v.len())`. This invariant holds after inserts, duplicates, partial overlaps, and flushes.
- **Test:** `test_BC_2_04_047_buffered_bytes_mirrors_segment_size_sum()`

### AC-013 (traces to BC-2.04.047 postcondition 4)
- For `Duplicate`, `ConflictingOverlap`, `OutOfWindow`, and `IsnMissing` results, `buffered_bytes` is unchanged.
- **Test:** `test_BC_2_04_047_buffered_bytes_unchanged_for_non_insert_results()`

### AC-014 (traces to BC-2.04.047 postcondition 5)
- After `flush_contiguous()` flushes N bytes, `buffered_bytes` decreases by exactly N.
- **Test:** `test_BC_2_04_047_buffered_bytes_decrements_on_flush()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| FlowDirection::insert_segment (overlap paths) | src/reassembly/segment.rs | pure-core |
| FlowDirection::flush_contiguous | src/reassembly/segment.rs | pure-core |
| FlowDirection::memory_used (debug_assert) | src/reassembly/flow.rs | pure-core |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Exact same seq, exact same bytes | Duplicate |
| EC-002 | New segment byte range covered by 2 non-contiguous existing segments with matching bytes | Duplicate (via union-coverage path) |
| EC-003 | Same range, one byte differs | ConflictingOverlap (not Duplicate) |
| EC-004 | New segment extends existing at the end (append) | PartialOverlap; tail gap bytes appended |
| EC-005 | New segment extends existing at the start (prepend) | PartialOverlap; head gap bytes prepended |
| EC-006 | New segment spans two existing segments with a gap between | Gap bytes between filled |
| EC-007 | Segment B starts exactly where segment A ends | Inserted; overlap_count not incremented |
| EC-008 | Segment B starts one byte before segment A ends | Overlap detected |
| EC-009 | Three segments covering new range jointly; all bytes match | Duplicate |
| EC-010 | Empty data slice | Returns Inserted (early-return before any overlap checks) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| src/reassembly/segment.rs | pure-core | No I/O; BTreeMap mutation only |
| src/reassembly/flow.rs (memory_used) | pure-core | debug_assert only; reads are immutable |

## Token Budget Estimate (MANDATORY)

| Context Source | Estimated Tokens |
|---------------|-----------------|
| This story spec | ~3,000 |
| BC files (5 BCs) | ~5,000 |
| src/reassembly/segment.rs (overlap paths ~lines 118-212) | ~2,000 |
| src/reassembly/flow.rs (memory_used ~lines 170-177) | ~300 |
| Test files | ~4,000 |
| Tool outputs overhead | ~1,000 |
| **Total** | **~15,300** |
| Agent context window | 200K for Sonnet |
| **Budget usage** | **~7.5%** |

## Tasks (MANDATORY)

1. [ ] Write failing tests for all 14 ACs in `tests/reassembly_segment_tests.rs`
2. [ ] Verify Red Gate: all tests fail before implementation changes
3. [ ] Verify existing implementation satisfies all ACs (brownfield)
4. [ ] Add proptest for buffered_bytes invariant: random insert/flush sequences (AC-012)
5. [ ] Verify adjacency boundary test: new_start == existing_end should NOT trigger overlap
6. [ ] Verify union-coverage path: two existing non-contiguous segments covering a new range
7. [ ] Update STATE.md

## Previous Story Intelligence (MANDATORY)

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-015 | Segment tests in reassembly_segment_tests.rs | BTreeMap key is ISN-relative u64 offset | wrapping_sub cast to u64 is the correct offset arithmetic |
| STORY-014 | ISN must be set before insert_segment | IsnMissing is a programming error, not a user error | ISN_MISSING_WARNED is process-wide |

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| First-wins policy: existing bytes NEVER overwritten | BC-2.04.036 invariant 1; INV-3 | Test: assert existing bytes unchanged after PartialOverlap |
| Half-open interval `new_start < existing_end && new_end > existing_offset` | BC-2.04.043 invariant 1 | Code review: grep overlap check in segment.rs:118 |
| `overlap_count` increments on ALL overlap types | BC-2.04.035 invariant 2 (Duplicate also increments) | Test: assert overlap_count++ for Duplicate, PartialOverlap, ConflictingOverlap |
| `debug_assert` in memory_used() verifies buffered_bytes | BC-2.04.047 invariant 2 | Compile with debug assertions enabled in test builds |

## Library & Framework Requirements (MANDATORY)

| Tool | Version | Purpose |
|------|---------|---------|
| Rust stable toolchain | MSRV 1.85+ | BTreeMap, debug_assert |
| proptest | from Cargo.toml | Property-based buffered_bytes invariant tests |

## File Structure Requirements (MANDATORY)

| File | Action | Purpose |
|------|--------|---------|
| `src/reassembly/segment.rs` | verify (lines 118-212) | Overlap detection and gap-fill logic |
| `src/reassembly/flow.rs` | verify (lines 170-177) | memory_used() with debug_assert |
| `tests/reassembly_segment_tests.rs` | modify | Add AC-001 through AC-014 |
