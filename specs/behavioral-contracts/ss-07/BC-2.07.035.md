---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/analyzer/tls.rs
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: v0.1.0-brownfield
modified:
  - "v0.1.0: VP back-reference back-fill (P8-DEFER) — 2026-05-21"
  - "v1.3: re-point evidence from inferred to dedicated on_flow_close tests (F-S058-P5-001/P6-001) — 2026-05-29"
  - "v1.4: PG-ARP-F2-007 ss-07 full re-anchor — on_flow_close 841-843→841-843 — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.035: on_flow_close Drops Per-Flow TlsFlowState

## Description

When `StreamHandler::on_flow_close` is called for a flow, `TlsAnalyzer` removes the
corresponding `TlsFlowState` from the `flows` HashMap via `self.flows.remove(flow_key)`.
This frees the `client_buf` and `server_buf` memory for that flow. All counts and
findings accumulated during the flow's lifetime are preserved in the aggregate maps
and `all_findings` -- only the per-flow state is dropped.

## Preconditions

1. `on_flow_close` is called with a `flow_key` that exists in `self.flows`.
   (If the key is absent, `remove` is a no-op.)

## Postconditions

1. `self.flows.remove(flow_key)` is called; the TlsFlowState is dropped.
2. The `client_buf` and `server_buf` memory for that flow is freed.
3. `sni_counts`, `ja3_counts`, `ja3s_counts`, `version_counts`, `cipher_counts`,
   `handshakes_seen`, `parse_errors`, and `all_findings` are all UNCHANGED.
4. `flows.len()` decreases by 1 (if the key was present).

## Invariants

1. Per-flow state cleanup is the ONLY operation in `on_flow_close`; no analysis is
   performed at close time.
2. The `_reason` parameter (CloseReason) is ignored by TlsAnalyzer.
3. A flow that completes both hellos has `TlsFlowState.done() == true` at close time;
   this state persists until the remove.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | on_flow_close for a key not in flows | No-op (HashMap::remove returns None); no panic |
| EC-002 | on_flow_close for a flow that never saw any data | TlsFlowState removed; no aggregate state change |
| EC-003 | Reopening same FlowKey after close | New TlsFlowState created fresh (via or_insert_with on next on_data) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Process ClientHello on flow A; close flow A; check flows | flows.len() == 0; sni_counts still has entry from flow A | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | on_flow_close removes TlsFlowState and preserves aggregates | unit: test_on_flow_close_drops_state_preserves_aggregates; test_on_flow_close_absent_key_no_panic |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md -- per-flow state cleanup on close is part of TLS analysis lifecycle management |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs:841-843, C-13) |
| Stories | STORY-058 |
| Origin BC | BC-TLS-035 (pass-3 ingestion corpus, HIGH confidence -- dedicated unit tests: test_on_flow_close_drops_state_preserves_aggregates, test_on_flow_close_absent_key_no_panic) |

## Related BCs

- BC-2.07.001 -- related to (on_data creates TlsFlowState; on_flow_close drops it)
- BC-2.07.003 -- related to (done() state is in the flow state being dropped)

## Architecture Anchors

- `src/analyzer/tls.rs:841-843` -- `on_flow_close` implementation
- `tests/tls_analyzer_tests.rs` -- test_on_flow_close_drops_state_preserves_aggregates (pc1-4: flow removed, aggregates preserved, flows.len()-1)
- `tests/tls_analyzer_tests.rs` -- test_on_flow_close_absent_key_no_panic (inv1-2/EC-001: absent key → no panic, _reason ignored, state unchanged)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/analyzer/tls.rs:841-843` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `self.flows.remove(flow_key)` at tls.rs:841-843
- **dedicated unit tests**: test_on_flow_close_drops_state_preserves_aggregates (exercises pc1-4: flow removed, aggregates preserved, flows.len()-1); test_on_flow_close_absent_key_no_panic (exercises inv1-2/EC-001: absent key → no panic, _reason ignored, state unchanged)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flows HashMap |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
