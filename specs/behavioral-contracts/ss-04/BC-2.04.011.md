---
document_type: behavioral-contract
level: L3
version: "1.7"
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
  - "v1.3: Wave 7 wave-level adv-pass-2 F-2 HIGH: comprehensive SS-04 anchor sweep (W4.1 axis #3). Corrected on_fin reference from flow.rs:248-256 (pre-Wave-6, also semantically colliding with on_data_without_syn) to flow.rs:255-262 (post-Wave-6). Fixed in both Traceability Architecture Module row and Architecture Anchors section. — 2026-05-25"
  - "v1.4: Wave 7 wave-level adv-pass-4 F-2 (process-gap): mega-sweep false-CORRECT — closing brace at mod.rs:174 not included in cited range 166-173; corrected to 165-174. — 2026-05-25"
  - "v1.5: Wave 8 wave-level adv-pass-1 F-1 HIGH closure (S-7.01 sibling-BC propagation, W7.2 recurrence #5): PC4 enforcement-mode notation — \"remaining contiguous data flushed in close_flow\" is structurally a defense-in-depth invariant (per-packet flush at mod.rs:162 drains buffer pre-close); enforced via code-review of close_flow flush loop body at lifecycle.rs:52-59. Mirrors BC-2.04.010 v1.5 PC2 + BC-2.04.029 v1.4 + BC-2.04.048 v1.3 / ADR-0004 amendment precedent. — 2026-05-26"
  - "v1.6: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:165-174 → mod.rs:198-205 (FIN-close is_some_and detection block); mod.rs:281-287 → mod.rs:313-319 (FIN flag block in apply_handshake_flags); mod.rs:162 → mod.rs:193 (flush_contiguous_data call in process_packet). — 2026-06-01"
  - "v1.7: PG-ARP-F2-007 ss-04-full re-anchor: mod.rs:198-205 → mod.rs:198-205 (FIN-close detection block); mod.rs:313-319 → mod.rs:313-319 (FIN flag block); mod.rs:193 → mod.rs:193 (flush_contiguous_data call). All prose, Architecture Module, Architecture Anchors, and Source Evidence updated. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.011: Both FINs Close Flow with CloseReason::Fin

## Description

TCP connection teardown requires a FIN from each side. The engine uses `fin_count` (a `u8`
that saturates at 255) to track how many FIN-flagged packets have been seen per flow. When
`fin_count >= 2`, `on_fin()` sets `state = FlowState::Closed`. After payload processing for
the packet that triggered the second FIN, `process_packet` detects `state == Closed` and calls
`close_flow(key, CloseReason::Fin, handler)`, incrementing `stats.flows_fin`. Each individual
FIN also transitions the state toward `Closing`.

## Preconditions

1. A TCP FIN packet arrives for a flow.
2. The flow's `fin_count` is at least 1 before this packet (i.e., this is the second FIN
   seen, possibly from either direction).

## Postconditions

1. `flow.fin_count` is now >= 2.
2. `flow.state == FlowState::Closed` (via `on_fin`).
3. `stats.flows_fin` increments by 1.
4. Any remaining contiguous data in both directions is flushed to handler.
   (Enforcement: in the current engine architecture, the per-packet flush at
   `src/reassembly/mod.rs:193` (unconditional `flush_contiguous_data` after every
   `insert_payload_segment`) already delivers all contiguous-prefix data BEFORE any close path
   runs. The `flush_contiguous` loop at `src/reassembly/lifecycle.rs:52-59` inside `close_flow`
   is therefore structurally a defense-in-depth invariant — it CAN deliver if a future refactor
   breaks per-packet flush, but cannot be triggered to deliver under current engine semantics.
   PC4 is enforced via code-review of the close_flow flush loop body's presence (mirrors
   BC-2.04.010 v1.5 PC2 / BC-2.04.029 v1.4 PC1-PC3 / BC-2.04.048 v1.3 PC2 / ADR-0004
   amendment enforcement-mode precedent).)
5. `handler.on_flow_close(key, CloseReason::Fin)` is called exactly once.
6. The flow is removed from `self.flows`.

## Invariants

1. The first FIN transitions state from `Established` (or `SynSent`) to `Closing`; the second
   FIN transitions from `Closing` to `Closed`.
2. FIN close happens AFTER payload processing for the FIN packet (allowing data carried in the
   FIN segment to be reassembled).
3. `fin_count` uses `saturating_add` -- a flow with more than 255 FINs does not overflow.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First FIN (only one seen so far) | state -> Closing; fin_count=1; flow NOT closed |
| EC-002 | Second FIN from same direction (retransmit) | fin_count reaches 2; state -> Closed; flow closed |
| EC-003 | FIN with payload | Payload inserted and flushed; then FIN-close detected after flush |
| EC-004 | FIN on New flow (no handshake) | on_fin from New: state -> Closing; second FIN -> Closed |
| EC-005 | RST and FIN in same packet | RST block runs first (PostHandshake::FlowClosed); FIN block not reached |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Client FIN, then Server FIN | flows_fin=1; on_flow_close(Fin); flow removed | happy-path |
| Client FIN only (never closed by server) | state=Closing; flow remains open until timeout |  edge-case |
| Client FIN retransmit counts as second FIN | flow closed as if both sides sent FIN | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Second FIN (any direction) closes flow | unit: process FIN from C, FIN from S; assert flow removed |
| — | CloseReason::Fin delivered to handler | unit: capture on_flow_close reason |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- FIN-based flow close is required for correct TCP lifecycle management |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:198-205, FIN-close detection; mod.rs:313-319, FIN flag block; flow.rs:255-262, on_fin) |
| Stories | STORY-019 |
| Origin BC | BC-RAS-011 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.010 -- related to (RST close; alternative close path)
- BC-2.04.050 -- composes with (state machine transitions for FIN)
- BC-2.04.012 -- related to (finalize closes remaining flows including half-closed ones)

## Architecture Anchors

- `src/reassembly/mod.rs:198-205` -- process_packet: if state==Closed after payload, close_flow(Fin)
- `src/reassembly/mod.rs:313-319` -- FIN flag block: set fin_seen, call on_fin
- `src/reassembly/flow.rs:255-262` -- on_fin: fin_count++; state transitions

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:198-205` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `is_some_and(|f| f.state == FlowState::Closed)` after payload processing
- **assertion**: fin_count uses saturating_add

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callback) |

## Refactoring Notes

No refactoring needed. FIN-close detection after payload processing is intentional (allows
FIN+data to be reassembled).
