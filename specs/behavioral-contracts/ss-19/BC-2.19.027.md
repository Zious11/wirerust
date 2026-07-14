---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "a153144"
---

# BC-2.19.027: on_flow_close Removes Iec104FlowState and Discards Carry Bytes

## Description

`Iec104Analyzer::on_flow_close(flow_id: FlowId)` is called by the stream dispatcher when a
TCP flow terminates. It removes the `Iec104FlowState` entry for the flow from the analyzer's
state map, discarding any unprocessed carry bytes in `carry_c2s` and `carry_s2c`. This is
the standard flow lifecycle teardown path. After `on_flow_close`, the flow's state is
permanently deleted — no state can leak to a future flow with the same flow ID.

## Preconditions

1. A flow with `flow_id` is currently tracked in the analyzer's flow state map.
2. `on_flow_close(flow_id)` is called (triggered by TCP FIN/RST observation or session timeout).

## Postconditions

1. `Iec104FlowState` for `flow_id` is removed from the state map.
2. `carry_c2s` and `carry_s2c` for this flow are dropped (memory freed).
3. Any subsequent `on_data` calls for the same flow ID will find no existing state (a fresh state will be created if the flow restarts).
4. No finding is emitted for normal flow close.

## Invariants

1. **No state leak**: after `on_flow_close`, no bytes or state from this flow persist in the analyzer.
2. **Idempotent**: calling `on_flow_close` on an already-removed flow_id is a no-op (returns Ok or ignores if not found).
3. **No spurious findings**: flow close is a normal event; no T0814 or other finding is emitted.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Normal TCP FIN | State removed; carry discarded |
| EC-002 | TCP RST (abrupt close) | Same: state removed, carry discarded |
| EC-003 | Flow close with non-empty carry | Carry silently discarded |
| EC-004 | `on_flow_close` called for unknown flow_id | No-op; no panic |

## Canonical Test Vectors

| Scenario | Expected |
|----------|----------|
| Flow with 100 bytes in carry_c2s → FIN | State removed; 100 bytes discarded; no finding |
| Fresh flow (no state) → FIN | No-op |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in on_flow_close for any flow_id | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — per-flow state teardown on flow close is a core lifecycle requirement for the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 2 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — lifecycle management) |

## Related BCs

- BC-2.19.025 — depends on (carry buffer lifecycle ends here)
- BC-2.19.026 — depends on (frame-walk loop state is part of what's torn down)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `fn on_flow_close(&mut self, flow_id: FlowId) { self.flows.remove(&flow_id); }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 2`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
