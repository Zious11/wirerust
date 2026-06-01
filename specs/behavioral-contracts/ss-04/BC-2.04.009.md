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
  - "v1.3: Wave 7 wave-level adv-pass-1 F-1: corrected on_data_without_syn anchor from flow.rs:241-246 to flow.rs:248-253 (Wave 6 fin_count addition shifted lines +7; W4.1 recurrence). Verified mod.rs:305-311 → 305-312 anchor against current source. — 2026-05-25"
  - "v1.4: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:305-312 → mod.rs:335-341 (on_data_without_syn block in insert_payload_segment; HS-043 inserted 29 lines at process_packet entry). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.009: Mid-Stream Join Infers ISN from seq-1; Flow Marked Partial

## Description

When a TCP payload segment arrives for a flow whose state is still `New` (no SYN or SYN+ACK
was seen -- a mid-capture join), the engine calls `on_data_without_syn()` which transitions
the state to `Established` and sets `partial = true`. It then sets the initiator to the packet
source and calls `infer_isn(tcp.seq)` which stores `seq - 1` as the ISN, making the first
data byte appear at stream offset 1. `stats.flows_partial` is incremented. All downstream
reassembly logic proceeds identically to a fully-handshaked flow.

## Preconditions

1. A TCP data packet (non-empty payload) arrives.
2. `flow.state == FlowState::New` (no SYN or SYN+ACK seen for this FlowKey).

## Postconditions

1. `flow.state == FlowState::Established`.
2. `flow.partial == true`.
3. `flow.initiator == Some((packet.src_ip, tcp.src_port))` (set via `set_initiator`).
4. The direction for `src_ip:src_port` has `isn == Some(tcp.seq.wrapping_sub(1))`.
5. `flow.client_to_server.base_offset == 1` (per `infer_isn`).
6. `stats.flows_partial` increments by 1.
7. The segment is inserted and flushed normally.

## Invariants

1. ISN inference is `seq.wrapping_sub(1)`, making the first data byte's offset 1 (same as a
   normally-SYN-handshaked flow). Wrapping ensures correct behavior when seq == 0.
2. `flows_partial` counts flows that entered via this path; it is not reset when the flow
   later closes.
3. Once `partial = true`, it stays true for the flow lifetime.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Mid-stream join with seq = 0 (wraps to ISN = u32::MAX) | infer_isn: isn = 0u32.wrapping_sub(1) = u32::MAX; base_offset = 1 |
| EC-002 | Second data packet on partial flow (different direction) | set_initiator no-op; direction = ServerToClient; ISN inferred for s2c if not yet set |
| EC-003 | SYN arrives after data on partial flow | set_initiator no-op; set_isn no-op (already inferred); on_syn no-op (state already Established) |
| EC-004 | Multiple partial flows in same PCAP | flows_partial counts all; each is independent |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Data from 1.1.1.1:5000 with seq=1001, no prior SYN | state=Established; partial=true; isn=1000; flows_partial=1 | happy-path |
| Data with seq=0 | isn=u32::MAX; base_offset=1 | edge-case (wrap) |
| Data then SYN on same flow | Partial flow; SYN processing no-ops on already-set state | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | infer_isn sets isn = seq.wrapping_sub(1) | unit: process single data packet without SYN; assert isn |
| — | flows_partial increments exactly once per mid-join flow | unit: two partial flows; assert flows_partial == 2 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- mid-stream join is required for forensic analysis of captures that begin mid-connection |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:335-341, insert_payload_segment; flow.rs:248-253, on_data_without_syn; flow.rs:143-148, infer_isn) |
| Stories | STORY-014 |
| Origin BC | BC-RAS-009 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.031 -- composes with (ISN inference: infer_isn formal spec)
- BC-2.04.052 -- composes with (on_data_without_syn state transition)
- BC-2.04.004 -- related to (normal SYN path; this is the alternative)

## Architecture Anchors

- `src/reassembly/mod.rs:335-341` -- on_data_without_syn + set_initiator + infer_isn block
- `src/reassembly/flow.rs:248-253` -- on_data_without_syn: state=Established, partial=true
- `src/reassembly/flow.rs:143-148` -- infer_isn: wrapping_sub(1), base_offset=1

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:335-341` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if flow.state == FlowState::New` in insert_payload_segment
- **assertion**: `wrapping_sub(1)` -- explicit wrapping arithmetic

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow state and stats |
| **Deterministic** | yes (wrapping arithmetic is deterministic) |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. The mid-stream join detection is a clean guard at the top of
insert_payload_segment.
