---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-21
capability: CAP-21
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "cf116b5"
---

# BC-2.21.003: `on_flow_close` Removes `S7commFlowState` and Discards All Carry Bytes

## Description

`S7commAnalyzer::on_flow_close` is called when the reassembly layer (SS-04) tears down
a tracked flow (FIN/RST observed, or idle timeout). Per the established binary-ICS
analyzer precedent (BC-2.19.027 for IEC-104, analogous DNP3/ENIP contracts),
`on_flow_close` removes the flow's `S7commFlowState` entirely — including any
in-progress carry-buffer content — rather than attempting to salvage a partial frame.
No finding is emitted for a flow closing with non-empty carry buffers; an incomplete
final frame at connection teardown is ordinary TCP-connection-lifecycle behavior, not
evidence of malformed traffic.

## Preconditions

1. The reassembly layer signals flow closure for a `FlowKey` previously classified
   `DispatchTarget::S7comm`.
2. `S7commFlowState` may or may not exist for this flow (it may never have been
   created, per BC-2.21.001 Edge Case EC-001, if no bytes were ever delivered).

## Postconditions

1. If `S7commFlowState` exists for the `FlowKey`, it is removed from the analyzer's
   per-flow map.
2. Any bytes remaining in `carry_c2s`/`carry_s2c` at the time of closure are discarded
   with the rest of the struct — no attempt is made to force-parse a partial TPKT/COTP
   frame at teardown.
3. No `Finding` is emitted as a consequence of non-empty carry buffers at closure.
4. If `S7commFlowState` does not exist for the `FlowKey` (flow never received data),
   `on_flow_close` is a no-op.

## Invariants

1. **No memory leak across flow lifecycle**: every `S7commFlowState` created by
   BC-2.21.001 is eventually removed by this BC when its flow closes — bounded by
   `max_flows`/`memcap` (SS-04) between creation and closure, consistent with every
   other binary-ICS analyzer's flow-state lifecycle.
2. **Teardown is not an anomaly signal**: an incomplete final frame at connection
   close is not, by itself, evidence of malformed or adversarial traffic — this
   mirrors BC-2.19.027's identical ruling for IEC-104.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow closes with non-empty `carry_c2s` containing a genuinely truncated (attacker-terminated-mid-frame) TPKT frame | Silently discarded, no finding — indistinguishable at this layer from an ordinary connection reset; distinguishing malicious early termination from benign teardown is out of scope for this BC |
| EC-002 | `on_flow_close` is called twice for the same `FlowKey` (defensive double-close) | Second call is a no-op (state already removed by Postcondition 4's no-op path) |

## Canonical Test Vectors

| Scenario | Expected outcome | Category |
|----------|-------------------|---------|
| Flow with populated `S7commFlowState` (non-empty carry, `classified_protocol: Some(Classic)`) closes | State removed, no `Finding` emitted | happy-path: normal teardown |
| Flow with no `S7commFlowState` (zero bytes ever delivered) closes | No-op | edge-case: never-instantiated state |

## Verification Properties

(No independent VP-NNN — lifecycle contract verified by unit test asserting map
non-membership post-close, mirroring BC-2.19.027's treatment; no proof-harness
candidate.)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 |
| Capability Anchor Justification | CAP-21 ("S7comm Analysis") per domain/capabilities/cap-21-s7comm-analysis.md §CAP-21 — flow-lifecycle teardown is part of the per-flow state ownership this capability establishes |
| L2 Domain Invariants | None directly (flow-lifecycle memory-bound contract, analogous to but not itself a cited domain invariant) |
| Architecture Module | SS-21 (`src/analyzer/s7comm.rs`, planned); SS-04 (reassembly, flow-close signal source) |
| ADR | ADR-014 Decision 1 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | (none) |

## Related BCs

- BC-2.19.027 — composes with (identical IEC-104 lifecycle precedent this BC mirrors)
- BC-2.21.001 — depends on (the struct this BC removes)

## Architecture Anchors

- `src/analyzer/s7comm.rs` (planned) — `impl StreamAnalyzer for S7commAnalyzer { fn on_flow_close(...) }`

## Story Anchor

STORY-186 (moved from its originating BC-2.21.NNN block per the F3 decomposition, per
consistency-audit-verified mapping)

## VP Anchors

(None.)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | removes entry from per-flow map (not global state) |
| **Deterministic** | yes |
| **Thread safety** | single-flow-owner access pattern |
| **Overall classification** | effectful shell (state removal) |
