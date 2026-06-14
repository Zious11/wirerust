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
  - "v1.3: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:517-533 → mod.rs:574-591 (flush_contiguous_data fn definition). — 2026-06-01"
  - "v1.4: PG-ARP-F2-007 ss-04-full re-anchor: segment.rs:369-381 → segment.rs:369-381 (flush_contiguous); mod.rs:574-591 → mod.rs:574-591 (flush_contiguous_data). — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.007: In-Order Data Flushes Contiguously to Handler

## Description

After each payload segment is inserted, `flush_contiguous_data` drains the head of the
per-direction BTreeMap by walking it from `base_offset` forward, collecting every contiguous
segment. Each contiguous chunk is delivered to the handler as a separate `on_data` call, and
`base_offset` advances by the length of each chunk. Only segments stored at exactly
`base_offset` are consumed; gaps halt the flush. This is the core in-order delivery guarantee.

## Preconditions

1. A direction's segment buffer contains at least one entry at key == `base_offset`.
2. `flush_contiguous_data` is called after `insert_payload_segment`.

## Postconditions

1. All segments stored at consecutive offsets starting from `base_offset` are removed from
   the BTreeMap and delivered as `on_data` callbacks.
2. `base_offset` advances by the total bytes flushed.
3. `stats.bytes_reassembled` increments by the total bytes flushed.
4. `total_memory` decrements by the bytes freed from the buffer.
5. The handler receives one `on_data` call per contiguous chunk, not one per segment.
6. The segment that triggered the flush is included (it was just inserted at `base_offset`).

## Invariants

1. `flush_contiguous` only removes segments at `base_offset`; it never skips gaps or delivers
   out-of-order bytes.
2. After flush, `buffered_bytes` reflects the bytes still in the BTreeMap (only unflushed
   segments remain).
3. `base_offset` is monotonically non-decreasing within a flow direction.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Segment arrives in-order (no gap) | Immediately flushed; no buffering |
| EC-002 | Gap exists before base_offset | flush stops at gap; only segments up to gap are delivered |
| EC-003 | Out-of-order segment fills a gap, making sequence contiguous | Next flush delivers all previously buffered segments plus the new one |
| EC-004 | Empty payload (pure ACK) | No insert; no flush (guard in process_packet skips empty payloads) |
| EC-005 | Multiple contiguous segments all flushed in one call | Delivered as separate on_data calls (one per original segment), all with correct offsets |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Segment at offset 1 (base_offset=1) | Flushed immediately; on_data(key, dir, data, 1); base_offset advances | happy-path |
| Segments at offsets 5,1,3 (out-of-order) | After each insert, only the contiguous prefix is flushed; offset 1 flushed first, etc. | edge-case |
| Gap at offset 3 when segments at 1,2 present | Offsets 1,2 flushed; offset 3 and beyond buffered | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-011 | flush_contiguous delivers only contiguous prefix | unit: insert with gap; assert partial flush |
| VP-011 | base_offset monotonically increases | proptest: random insert order; assert base_offset never decreases |
| VP-011 | bytes_reassembled equals sum of flushed chunks | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- contiguous delivery is the defining contract of stream reassembly |
| L2 Domain Invariants | None directly (base_offset monotonicity is a derived invariant) |
| Architecture Module | SS-04 (reassembly/segment.rs:369-381, flush_contiguous; mod.rs:574-591) |
| Stories | STORY-015 |
| Origin BC | BC-RAS-007 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.008 -- composes with (out-of-order buffering feeds this flush)
- BC-2.04.034 -- composes with (flush_contiguous formal spec)
- BC-2.04.047 -- related to (buffered_bytes tracking)

## Architecture Anchors

- `src/reassembly/segment.rs:369-381` -- FlowDirection::flush_contiguous implementation
- `src/reassembly/mod.rs:574-591` -- flush_contiguous_data: calls flush, dispatches on_data

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/segment.rs:369-381` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: BTreeMap::remove at base_offset; base_offset advances by data.len()
- **type constraint**: BTreeMap key ordering guarantees offset ordering

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates FlowDirection buffer and engine stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful buffer mutation) |

## Refactoring Notes

`flush_contiguous` is pure modulo the BTreeMap mutation -- the core while-loop logic is
suitable for Kani verification.
