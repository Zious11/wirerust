---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.3: F-002 remediation — Description corrected to document allowed==0 → DepthExceeded path; DF-SIBLING-SWEEP-001 — 2026-05-26"
  - "v1.4: W10-D13 fix — added forensic/security note to Postconditions documenting silent discard of bytes beyond max_depth. Intentional behavior; analyst-facing implication documented for parity with BC-2.04.015 PC-7 data-loss note style. — 2026-05-28"
  - "v1.5: F-DRIFT2A-001 — fixed stale domain/capabilities/cap-04-tcp-reassembly.md citation to domain/capabilities/cap-04-tcp-reassembly.md in L2 Capability and Capability Anchor Justification rows. — 2026-05-29"
  - "v1.6: PG-ARP-F2-007 ss-04-full re-anchor: segment.rs:229-253 → segment.rs:229-253 (depth check + truncation); lifecycle.rs:135-158 → lifecycle.rs:135-158 (generate_truncated_finding); segment.rs:230-235 → segment.rs:230-235. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.041: Depth Truncation: Segment Crossing max_depth is Truncated

## Description

When a new segment would cause `reassembled_bytes + buffered_bytes + segment.len()` to
exceed `max_depth` **AND the remaining capacity `allowed = max_depth.saturating_sub(reassembled_bytes + buffered_bytes)` is > 0**,
the segment is TRUNCATED to `allowed` bytes before insertion. The truncated portion is inserted
and `InsertResult::Truncated` is returned. `depth_exceeded` is set to `true`. The engine match
arm then emits an `Anomaly/Inconclusive/Low` finding (via `generate_truncated_finding`).
Subsequent segments to the same direction return `DepthExceeded` without truncation.
**When `allowed == 0` (the buffer is already at `max_depth`), the segment is fully rejected
with `InsertResult::DepthExceeded` (see EC-004); no bytes are stored.**

## Preconditions

1. `self.isn` is `Some(isn)`.
2. `self.depth_exceeded` is `false` (first time exceeding the limit).
3. `remaining_depth = max_depth.saturating_sub(self.reassembled_bytes)` is > 0.
4. `self.reassembled_bytes + self.buffered_bytes + data.len() > max_depth` (exceeds limit).
5. The allowed capacity `allowed = max_depth.saturating_sub(reassembled_bytes + buffered_bytes)` is > 0.

## Postconditions

1. Returns `InsertResult::Truncated`.
2. `data` is truncated to `allowed` bytes: `segment_data.truncate(allowed)`.
3. The truncated segment is inserted at its computed offset.
4. `self.buffered_bytes` increases by `allowed` (not by the full `data.len()`).
5. `self.depth_exceeded = true`.
6. The engine emits one `Anomaly/Inconclusive/Low` finding with no MITRE technique.
7. **Forensic/security note — silent data loss past depth limit:** Bytes beyond `allowed`
   (i.e., `data[allowed..]`) are permanently discarded with no callback and no additional
   finding. After the Truncated result, all subsequent segments to the same direction return
   `DepthExceeded` and are also silently discarded. Forensic analysts should account for this
   potential data loss: a stream that hit `max_depth` may have had payload bytes beyond the cap
   that were never delivered to the analysis layer. This is intentional bounded-resource
   behavior; the analyst-facing implication mirrors the non-contiguous discard documented in
   BC-2.04.015 PC-7 for `CloseReason::MemoryPressure` eviction.

## Invariants

1. After Truncated is returned, `depth_exceeded` is permanently `true` for this direction.
   Subsequent calls return `DepthExceeded` immediately (not Truncated again).
2. The truncation ensures the total stream bytes never exceed `max_depth` per direction.
3. `Truncated` and `DepthExceeded` are distinct results: `Truncated` means "partial data was
   accepted"; `DepthExceeded` means "no data accepted because limit was already hit".

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Segment arrives when depth exactly at limit (remaining=0) | DepthExceeded (not Truncated); depth_exceeded may have already been true |
| EC-002 | Segment that fits entirely within remaining capacity | Inserted (not Truncated) |
| EC-003 | After Truncated, next segment arrives | DepthExceeded (depth_exceeded=true blocks further insertion) |
| EC-004 | Truncation leaves 0 allowed bytes (edge: buffered+reassembled == max_depth) | DepthExceeded at the inner check (allowed == 0) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| max_depth=100, reassembled=90, buffered=5, data=20 bytes | Truncated; 5 bytes accepted; depth_exceeded=true | happy-path |
| max_depth=100, reassembled=0, buffered=0, data=50 bytes | Inserted (fits); no truncation | edge-case |
| max_depth=100, reassembled=100, data=10 bytes | DepthExceeded (remaining=0) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Truncated: only allowed bytes stored, not full payload | unit: test_depth_limit_truncation |
| — | depth_exceeded=true after Truncated result | unit |
| — | Second segment after Truncated returns DepthExceeded | unit: test_depth_exceeded_counter |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP Stream Reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP Stream Reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- depth truncation is the per-direction resource bound enforcement of TCP reassembly |
| L2 Domain Invariants | INV-6 (bounded-resource design -- max_depth is the per-direction memory cap) |
| Architecture Module | SS-04 (reassembly/segment.rs:229-253, C-8) |
| Stories | STORY-018 |
| Origin BC | BC-RAS-041 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.023 -- composes with (engine emits truncated finding when this result is returned)
- BC-2.04.027 -- related to (segments_depth_exceeded tracks subsequent DepthExceeded results)

## Architecture Anchors

- `src/reassembly/segment.rs:229-253` -- depth check, truncation, and depth_exceeded flag
- `src/reassembly/lifecycle.rs:135-158` -- generate_truncated_finding (called by engine on Truncated result)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:229-253` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: test_depth_limit_truncation asserts Truncated result and correct partial storage
- **guard clause**: `if remaining_depth == 0 { return DepthExceeded }` at segment.rs:230-235

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.segments, self.buffered_bytes, self.depth_exceeded |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | pure (no I/O) |
