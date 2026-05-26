---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/segment.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-04
capability: CAP-04
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: Wave 8 STORY-015 adv-pass-5 F-1 HIGH closure: corrected Canonical Test Vectors table to be mathematically consistent with EC-001 and the actual test arithmetic — row 1 changed seq=u32::MAX-1 (was incorrectly claimed offset=2; correct value is offset=1); row 2 retained; row 3 rewritten to use 3 segments matching tests/reassembly_segment_tests.rs:651-712 wraparound-flush test (data labels A/X/B at offsets 1/2/3, flush sequence A → X+B). Row-1 anchor mirrors tests/reassembly_segment_tests.rs:612-616 (test_BC_2_04_039_sequence_wraparound_correct_offsets). — 2026-05-26"
  - "v1.4: Wave 8 wave-level adv-pass-6 F-1 MEDIUM closure (S-7.01 partial-fix-regression within-document): row 3 of Canonical Test Vectors was mathematically self-consistent post-v1.3 but did not correspond to any actual test — the cited line range (612-616) verifies row 1 only. v1.4 rewrites row 3 to describe the actual 3-segment scenario from test_BC_2_04_039_flush_delivers_wrapped_segments_in_order (lines 651-712). — 2026-05-26"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.039: TCP Sequence Wraparound Across 32-bit Boundary Reassembles Correctly

## Description

TCP sequence numbers are 32-bit unsigned values that wrap around at `u32::MAX`. The
`seq_offset` function in `segment.rs` computes `seq.wrapping_sub(isn) as u64`, which
correctly handles wraparound: a sequence number that wraps around `0` after `u32::MAX` still
produces the correct monotonically-increasing u64 offset relative to the ISN.

## Preconditions

1. `self.isn` is `Some(isn)` where `isn` is near `u32::MAX`.
2. Subsequent TCP sequence numbers may wrap around `0` (i.e., after `u32::MAX + 1 = 0`).
3. All other insertion constraints are met (in-window, within depth, etc.).

## Postconditions

1. `seq_offset(seq, isn) = seq.wrapping_sub(isn) as u64` produces the correct byte offset
   even when `seq < isn` due to wraparound.
2. Segment data is stored at the correct offset in `self.segments`.
3. `flush_contiguous` correctly delivers wrapped segments in the right order because
   BTreeMap keys are u64 and the offsets are monotonically increasing after wraparound.
4. No finding is emitted for wraparound (it is a normal TCP behavior, not an anomaly).

## Invariants

1. `wrapping_sub` on u32 values, cast to u64, produces a correct offset in the range
   `[0, u32::MAX as u64]` for any seq relative to isn. This is the standard TCP sequence
   arithmetic guarantee.
2. The offset space is effectively the `u32` sequence space projected onto `u64`; no
   sequence number can produce an offset larger than `u32::MAX`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | ISN = u32::MAX - 2, seq = u32::MAX, data = "A" | offset = 2; inserted at key 2 |
| EC-002 | ISN = u32::MAX - 2, seq = 0 (wrapped), data = "B" | offset = 3 (wrapping_sub 0 - (u32::MAX-2) = 3) |
| EC-003 | ISN = u32::MAX - 2, seq = 1, data = "C" | offset = 4; segment delivered after A and B |
| EC-004 | Segments arrive out-of-order across wraparound boundary | Buffered correctly; flush_contiguous delivers in offset order |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| ISN=u32::MAX-2, seq=u32::MAX-1 (offset=1), data=b"A" | Inserted at offset 1 | happy-path |
| ISN=u32::MAX-2, seq=0 (wrapped, offset=3), data=b"B" | Inserted at offset 3 | edge-case |
| ISN=u32::MAX-2, segments at seq=u32::MAX-1 (offset=1, data=b"A"), seq=0 (wrapped, offset=3, data=b"B"), seq=u32::MAX (bridge, offset=2, data=b"X"); B inserted first, then A, then X | flush sequence: A delivered alone after A inserted (gap at offset=2 blocks B); X+B delivered together after bridge fills the gap — mirrors `tests/reassembly_segment_tests.rs:651-712` | wraparound-flush ordering |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-015 | seq.wrapping_sub(isn) produces correct monotonic offset across u32::MAX boundary | Kani |
| VP-015 | flush_contiguous delivers wrapped segments in correct byte order | Kani |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- correct 32-bit wraparound arithmetic is a correctness requirement for any TCP reassembly implementation |
| L2 Domain Invariants | INV-3 (First-wins overlap policy -- wraparound does not change overlap semantics) |
| Architecture Module | SS-04 (reassembly/segment.rs:31-34, C-8) |
| Stories | STORY-015 |
| Origin BC | BC-RAS-039 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.031 -- depends on (ISN is the reference point for all seq_offset computations)
- BC-2.04.033 -- related to (single segment insertion is the base case; wraparound is an edge case of it)

## Architecture Anchors

- `src/reassembly/segment.rs:31-34` -- seq_offset function: `seq.wrapping_sub(isn) as u64`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:31-34` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_sequence_wraparound verifies correct reassembly across the 32-bit boundary
- **type constraint**: wrapping_sub is the standard Rust arithmetic for TCP sequence wraparound

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | none (seq_offset is a pure function) |
| **Deterministic** | yes |
| **Thread safety** | Send + Sync (pure function) |
| **Overall classification** | pure |

## Refactoring Notes

No refactoring needed -- the seq_offset function is already extracted as a pure standalone function. Ideal candidate for Kani proof: `forall seq: u32, isn: u32 => seq.wrapping_sub(isn) as u64 == correct_offset(seq, isn)`.
