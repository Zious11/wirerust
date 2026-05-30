---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/flow.rs
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

# BC-2.04.031: ISN Set on First SYN; Inferred as seq-1 on Data-Without-SYN

## Description

A `FlowDirection`'s initial sequence number (ISN) is set via one of two paths: `set_isn(seq)`
when a SYN or SYN+ACK is processed, which stores `seq` and sets `base_offset = 1` (since
data starts at ISN+1); or `infer_isn(first_seq)`, used on mid-stream joins, which stores
`first_seq.wrapping_sub(1)` as the ISN so that `first_seq` maps to offset 1. Both paths are
idempotent: a second call with `isn.is_some()` is a no-op.

## Preconditions

For `set_isn` path:
1. The direction's `isn` field is `None` (first SYN/SYN+ACK seen).
2. `seq` is the TCP sequence number from the SYN/SYN+ACK packet.

For `infer_isn` path:
1. The direction's `isn` field is `None`.
2. `first_seq` is the TCP sequence number of the first data packet seen (no prior SYN).

## Postconditions

For `set_isn(seq)`:
1. `self.isn = Some(seq)`.
2. `self.base_offset = 1` (ISN+1 is the first data byte's offset).

For `infer_isn(first_seq)`:
1. `self.isn = Some(first_seq.wrapping_sub(1))` (inferred ISN is one before first data seq).
2. `self.base_offset = 1`.

Both paths (when isn was already set):
3. No change to `self.isn` or `self.base_offset` (idempotent guard: `if self.isn.is_none()`).

## Invariants

1. After ISN is set (by either path), `base_offset` is always 1. Data at TCP seq `ISN+1`
   maps to buffer offset 1, which is the first consumable byte position.
2. The `wrapping_sub` in `infer_isn` handles the edge case where `first_seq == 0`
   (wraps to `u32::MAX`), ensuring the inferred ISN is never incorrect due to integer underflow.
3. Once set, the ISN is immutable for the direction's lifetime.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | set_isn called twice with different seq values | Second call is a no-op; first ISN preserved |
| EC-002 | infer_isn called with first_seq == 0 | ISN = u32::MAX (wrapping_sub wraps correctly); base_offset = 1 |
| EC-003 | infer_isn called after set_isn | No-op; ISN from SYN preserved |
| EC-004 | SYN seen but not SYN+ACK; server ISN missing when data arrives | insert_payload_segment calls infer_isn for the server direction |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| set_isn(1000) on fresh FlowDirection | isn=Some(1000), base_offset=1 | happy-path |
| infer_isn(500) on fresh FlowDirection | isn=Some(499), base_offset=1 | happy-path |
| set_isn(1000) then set_isn(2000) | isn=Some(1000) (unchanged) | edge-case |
| infer_isn(0) | isn=Some(4294967295), base_offset=1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | set_isn is idempotent: second call preserves first ISN | unit: reassembly_flow_tests |
| — | infer_isn(0) produces ISN=u32::MAX without panic | unit: boundary test |
| — | base_offset == 1 after any ISN-setting call | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- ISN management is fundamental to TCP stream reassembly's offset computation |
| L2 Domain Invariants | INV-1 (FlowKey canonical ordering -- ISN is per-direction within a canonically-ordered flow) |
| Architecture Module | SS-04 (reassembly/flow.rs:136-148, C-7) |
| Stories | STORY-014 |
| Origin BC | BC-RAS-031 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.004 -- depends on (SYN processing calls set_isn for client direction)
- BC-2.04.005 -- depends on (SYN+ACK processing calls set_isn for server direction)
- BC-2.04.009 -- depends on (mid-stream join calls infer_isn)
- BC-2.04.032 -- related to (insert_segment returns IsnMissing when ISN is None)

## Architecture Anchors

- `src/reassembly/flow.rs:136-148` -- set_isn and infer_isn implementations

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/flow.rs:136-148` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if self.isn.is_none()` in both set_isn and infer_isn ensures idempotence
- **type constraint**: `Option<u32>` for isn field enforces the unset state
- **assertion**: reassembly_flow_tests test_flow_direction_new asserts isn=None on construction

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.isn, self.base_offset |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O; purely in-memory mutation) |

## Refactoring Notes

No refactoring needed -- core ISN management logic is pure in-memory mutation. Suitable for Kani verification of idempotence and wrapping arithmetic.
