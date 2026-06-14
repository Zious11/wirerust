---
document_type: behavioral-contract
level: L3
version: "1.8"
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
  - "v1.3: Wave 7 wave-level adv-pass-2 F-2 HIGH: comprehensive SS-04 anchor sweep (W4.1 axis #3). Corrected Architecture Anchors on_rst reference from flow.rs:257-259 (pre-Wave-6) to flow.rs:264-266 (post-Wave-6 +7 line shift from fin_count addition). — 2026-05-25"
  - "v1.4: Wave 7 wave-level adv-pass-3 F-1 HIGH: mega-sweep (W4.1 axis #4). Fixed mod.rs:272-278 → 273-279 (off-by-one both bounds; line 272 is a blank line; RST block runs 273-279 with closing brace). Applied to Architecture Module table, Architecture Anchors section, and Source Evidence Path. — 2026-05-25"
  - "v1.5: Wave 8 STORY-019 adv-pass-4 F-2 closure (MEDIUM): PC2 enforcement-mode notation — \"any remaining contiguous data flushed in close_flow\" is structurally a defense-in-depth invariant (per-packet flush at mod.rs:162 already drains buffer pre-close), enforced via code-review of the flush_contiguous loop body at lifecycle.rs:52-59. Mirrors BC-2.04.029 v1.4 PC1/PC2/PC3 + BC-2.04.048 v1.3 PC2 + ADR-0004 amendment enforcement-mode pattern. — 2026-05-25"
  - "v1.6: Wave 8 wave-level adv-pass-4 F-1 MEDIUM closure (S-7.01 partial-fix regression within-file): v1.5 PC2 enforcement-mode notation propagated to 4 sibling sections (Description, EC-002, Canonical Test Vector row 3, Verification Property row 3) that retained pre-v1.5 wording asserting close_flow flushes data. All 4 now reflect that under current engine architecture per-packet flush at mod.rs:162 drains contiguous prefix pre-close; non-contiguous remainder is silently dropped (cannot flush behind gap); the close_flow flush loop is structurally a defensive no-op. — 2026-05-26"
  - "v1.7: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:273-279 → mod.rs:305-310 (RST block in apply_handshake_flags); mod.rs:162 → mod.rs:193 (flush_contiguous_data call in process_packet). All prose, EC, Architecture Module, Architecture Anchors, and Source Evidence updated. — 2026-06-01"
  - "v1.8: PG-ARP-F2-007 ss-04-full re-anchor: mod.rs:305-310 → mod.rs:305-310 (RST block); mod.rs:193 → mod.rs:193 (flush_contiguous_data call). All prose, EC, VP, and Traceability/Architecture Anchors updated. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.010: RST Closes Flow Immediately with CloseReason::Rst

## Description

When a TCP RST packet arrives for a flow, the engine calls `flow.on_rst()` to mark the flow
closed, increments `stats.flows_rst`, calls `close_flow(key, CloseReason::Rst, handler)` to remove the flow and notify the handler.
See PC2 enforcement-mode notation: any contiguous prefix has already been delivered by the
per-packet flush at `mod.rs:193` before RST arrives; non-contiguous remainder behind a gap is
silently dropped at close. Then returns
`PostHandshake::FlowClosed` to `process_packet`, which skips all further payload handling for
this packet. The flow is removed from the `flows` HashMap by `close_flow`.

## Preconditions

1. A TCP packet with RST=true arrives.
2. A flow for the packet's FlowKey exists in the engine.

## Postconditions

1. `stats.flows_rst` increments by 1.
2. Any remaining contiguous data in both directions is flushed to the handler via `on_data`
   calls (in `close_flow`). (Enforcement: in the current engine architecture, the per-packet
   flush at `src/reassembly/mod.rs:193` (unconditional `flush_contiguous_data` after every
   `insert_payload_segment`) already delivers all contiguous-prefix data BEFORE any close path
   runs. The `flush_contiguous` loop at `src/reassembly/lifecycle.rs:52-59` inside `close_flow`
   is therefore structurally a defense-in-depth invariant — it CAN deliver if a future refactor
   breaks per-packet flush, but cannot be triggered to deliver under current engine semantics.
   STORY-019 AC-002 verifies the close path runs without error; PC2 is enforced via code-review
   of the close_flow flush loop body's presence (mirrors the BC-2.04.029 v1.4 PC1/PC2/PC3 +
   BC-2.04.048 v1.3 PC2 / ADR-0004 amendment enforcement-mode precedent).)
3. `handler.on_flow_close(key, CloseReason::Rst)` is called exactly once.
4. The flow is removed from `self.flows`.
5. `total_memory` decrements by the bytes freed from the flow's buffers.
6. Payload processing for the RST packet itself is skipped (even if the RST packet carries
   data, that data is not processed).

## Invariants

1. RST closes the flow regardless of current state (New, SynSent, Established, Closing).
   `on_rst()` unconditionally sets `state = FlowState::Closed`.
2. `close_flow` is idempotent in effect: if the key is not found (already closed), a one-shot
   warning is emitted and the call returns without error (per BC-2.04.029).
3. `PostHandshake::FlowClosed` return value prevents any payload from the RST packet being
   processed after the flow is removed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | RST on New flow (no data seen) | Flow closed; no data flushed; on_flow_close(Rst) called |
| EC-002 | RST on Established flow with buffered data | Contiguous data already delivered pre-RST via per-packet flush at `mod.rs:193`; non-contiguous buffered data silently dropped at close (cannot flush behind gap); on_flow_close(Rst); total_memory released. See PC2 enforcement-mode. |
| EC-003 | RST with payload | Payload ignored; close happens; payload not inserted |
| EC-004 | RST on already-missing key | close_flow emits one-shot warning; no panic |
| EC-005 | Both SYN+ACK and RST in same packet (malformed) | Both blocks execute: on_syn_ack then on_rst; RST wins (state=Closed) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Established flow receives RST | flows_rst=1; on_flow_close(Rst); flow removed | happy-path |
| New flow receives RST (no handshake) | flows_rst=1; on_flow_close(Rst); flow removed | edge-case |
| Flow with 100 bytes contiguous + gap + 50 bytes non-contiguous receives RST | 100 bytes already delivered pre-RST via per-packet flush; 50 bytes silently dropped at close (gap blocks flush_contiguous); on_flow_close(Rst); total_memory=0 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | RST always results in flow removal | unit: process RST; assert flows.is_empty() |
| — | CloseReason::Rst delivered to handler | unit: capture on_flow_close reason |
| — | Buffered data delivered before on_flow_close (defense-in-depth invariant) | code review: close_flow flush loop body at `lifecycle.rs:52-59` (mirrors BC-2.04.029 v1.4 PC1/PC2/PC3 + BC-2.04.048 v1.3 PC2 / ADR-0004 amendment enforcement-mode precedent; per-packet flush at mod.rs:193 already drains contiguous prefix pre-close, so this VP cannot be unit-tested through close_flow path) |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- RST handling is a required part of the TCP flow lifecycle |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:305-310, RST block; lifecycle.rs:36-62, close_flow) |
| Stories | STORY-019 |
| Origin BC | BC-RAS-010 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.011 -- related to (FIN-based close; alternative close path)
- BC-2.04.012 -- related to (finalize: third close path)
- BC-2.04.051 -- composes with (RST state transition)
- BC-2.04.029 -- related to (missing-key warning in close_flow)

## Architecture Anchors

- `src/reassembly/mod.rs:305-310` -- RST block in apply_handshake_flags
- `src/reassembly/lifecycle.rs:36-62` -- close_flow: flush + remove + on_flow_close
- `src/reassembly/flow.rs:264-266` -- on_rst: unconditional state=Closed

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:305-310` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if tcp.rst` block with explicit counter increment and close_flow call
- **type constraint**: PostHandshake enum ensures process_packet stops after RST

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.flows, self.stats, self.total_memory |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation + callback) |

## Refactoring Notes

No refactoring needed. RST handling is clean and isolated in apply_handshake_flags.
