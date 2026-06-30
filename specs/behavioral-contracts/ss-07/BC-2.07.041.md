---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-06-29T00:00:00Z
phase: 1a
origin: greenfield
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-07
capability: CAP-07
lifecycle_status: active
introduced: fix-tls-clienthello-frag
modified:
  - "v1.1: Pass-1 adversarial reconciliation (F-P1-001/SR-001) — remove hs_carry_abandoned flag references from Postconditions 3, Invariant 3, and EC-002; replace with clear-and-recover counter semantics; the isolation guarantee is unchanged — 2026-06-29"
  - "v1.2: Fix burst 11 (F-COMP-002 MED) — Verification Properties table cross-flow isolation row re-pointed from proptest_vp039_direction_isolation (covers cross-DIRECTION, not cross-FLOW) to dedicated unit test test_BC_2_07_041_cross_flow_isolation (architect authoring in VP-039); proptest_vp039_direction_isolation retained for cross-DIRECTION property only; over-claim removed — 2026-06-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.041: Handshake Carry Buffers Are Per-Flow and Per-Direction Isolated

## Description

Each active flow has its own independent `client_hs_carry` and `server_hs_carry`
vectors stored inside `TlsFlowState`, which is keyed by `FlowKey` in `TlsAnalyzer`'s
`flows: HashMap<FlowKey, TlsFlowState>`. Bytes from flow A's carry buffers are never
read, written, or cleared in response to data arriving on flow B. Bytes from the
client direction of a flow are never mixed into the server direction's carry buffer
and vice versa. This mirrors the VP-014 cross-flow isolation invariant established for
`HttpAnalyzer` and extends it to the TLS handshake carry layer.

## Preconditions

1. Two or more flows are active simultaneously in `TlsAnalyzer` (each keyed by a
   distinct `FlowKey`).
2. `on_data` is called for one flow with data for a given direction while another
   flow's carry buffers may contain partial handshake data.

## Postconditions

1. Bytes appended to `flow_A.client_hs_carry` do not appear in `flow_B.client_hs_carry`
   or `flow_B.server_hs_carry` for any `flow_B != flow_A`.
2. Bytes appended to `flow_A.client_hs_carry` (ClientToServer) do not appear in
   `flow_A.server_hs_carry` (ServerToClient), and vice versa.
3. The `handshake_reassembly_overflows` counter is per-`TlsAnalyzer` instance (an
   aggregate statistic across all flows), but the carry buffer cleared during an
   overflow is per-flow and per-direction — the clear affects only the flow whose carry
   overflowed. There is no per-flow abandoned flag.
4. Dispatching a ClientHello from `flow_A.client_hs_carry` does not mutate any field
   of `flow_B` (`handshakes_seen`, `ja3_counts`, `sni_counts`, findings, etc. are all
   per-flow state inside `TlsFlowState`).
5. The cross-flow isolation guarantee is structurally enforced by the `HashMap<FlowKey,
   TlsFlowState>` keying: every `on_data` call receives a `flow_key` parameter, and
   all carry buffer access paths dereference only that key.

## Invariants

1. Carry buffer identity is keyed by `FlowKey`; no carry-buffer access path
   dereferences a `FlowKey` other than the one passed to the current `on_data` call.
   This mirrors VP-014's cross-flow isolation invariant for carry buffers.
2. `Direction::ClientToServer` data is appended only to `client_hs_carry`;
   `Direction::ServerToClient` data is appended only to `server_hs_carry`. The
   direction is a parameter to `on_data`; the carry selection must match it exactly.
3. There is no `hs_carry_abandoned` flag in `TlsFlowState`. On carry overflow, the
   buffer for the affected direction is cleared (BC-2.07.039) and processing continues;
   no per-flow or per-direction state is set that would propagate to another flow. The
   isolation guarantee is therefore stronger under clear-and-recover: there is no flag
   that could accidentally be read for the wrong flow.
4. After `on_flow_close(flow_key_A)`, all carry buffer state for `flow_A` is dropped;
   `flow_B`'s carry buffer state is unaffected.
5. Global counters (`handshakes_seen`, `version_counts`, `ja3_counts`, `ja3s_counts`,
   `sni_counts`, `cipher_counts`, `all_findings`, `parse_errors`) are per-`TlsAnalyzer`
   instance and are intentionally shared across flows (they are aggregate statistics,
   not per-flow). This is NOT a violation of isolation: isolation applies to the carry
   buffers and per-flow state (`TlsFlowState`), not to the aggregate counters.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two concurrent flows both deliver fragmented ClientHellos simultaneously | Each flow's carry buffers accumulate independently; each ClientHello dispatched separately; sni_counts and ja3_counts each incremented once per flow; no cross-contamination |
| EC-002 | Flow A's client carry overflows and is cleared (BC-2.07.039); Flow B's client carry still active | Flow B continues to accumulate normally; `flow_B.client_hs_carry` is unaffected; `handshake_reassembly_overflows` incremented once (aggregate counter) but does not affect per-flow carry state for Flow B |
| EC-003 | Interleaved delivery: Flow A client-direction, Flow B client-direction, Flow A client-direction (completing A's ClientHello) | After the third on_data call, `flow_A.client_hello_seen=true`; `flow_B.client_hello_seen` unchanged; no direction or flow confusion |
| EC-004 | ClientToServer bytes accidentally passed to server direction code path | Invariant 2 requires the carry selection to match the direction parameter; incorrect direction selection would violate this BC and must be caught in VP-039 Sub-E |
| EC-005 | Flow with only server direction data arriving (no client data) | `client_hs_carry` remains empty; `server_hs_carry` accumulates ServerHello bytes; no cross-direction bleed |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Interleaved fragmented ClientHello (C→S) and ServerHello (S→C) for same flow: first half of ClientHello, then first half of ServerHello, then remainder of ClientHello, then remainder of ServerHello | `client_hello_seen=true`, `server_hello_seen=true`, `parse_errors=0`, `ja3_counts` and `ja3s_counts` each have one entry — no carry cross-contamination | happy-path |
| Two flows: Flow A delivers complete ClientHello in one record; Flow B delivers fragmented ClientHello in two records | After both: each flow has `client_hello_seen=true`; `sni_counts` has entries from both; no cross-flow bleed | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-039 (Sub-E) | Cross-direction: interleaved `ClientToServer` and `ServerToClient` deliveries of fragmented hellos produce `client_hello_seen=true` and `server_hello_seen=true` with no carry-buffer cross-contamination; each carry buffer behaves identically to an independent same-direction run (PC-2/Inv-2) | proptest: `proptest_vp039_direction_isolation` |
| VP-039 (Sub-E) | Cross-flow: Flow A's carry buffer state does not affect Flow B — bytes appended to `flow_A.client_hs_carry` never appear in `flow_B.client_hs_carry` or `flow_B.server_hs_carry`; PC-1/PC-4/Inv-1 verified over two concurrent flows with interleaved `on_data` calls | unit: `test_BC_2_07_041_cross_flow_isolation` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md — per-flow and per-direction isolation is a correctness and security property of the TLS analysis subsystem; carry buffer contamination would produce wrong SNI/JA3 attribution |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs — `HashMap<FlowKey, TlsFlowState>` keying; direction parameter to `on_data`) |
| Finding Source | TLS-CLIENTHELLO-FRAG-001; F1 §6 regression risk #4 (VP-014 TLS carry analog) |
| Analogue | VP-014 (HttpAnalyzer cross-flow isolation; proptest; P1; verified) — same isolation pattern extended to TLS carry buffers |
| Stories | STORY-145 |
| Origin | greenfield (fix-tls-clienthello-frag cycle) |

## Related BCs

- BC-2.07.038 — depends on (carry buffers whose isolation this BC specifies)
- BC-2.07.039 — related to (clear-and-recover overflow clears per-flow carry; covered by this isolation guarantee; no flag leaks between flows)
- BC-2.07.040 — related to (flow close drops one flow's carry; other flows unaffected)
- BC-2.07.002 — composes with (server direction carry buffer for ServerHello reassembly)
- BC-2.07.001 — composes with (client direction carry buffer for ClientHello reassembly)

## Architecture Anchors

- `src/analyzer/tls.rs` — `flows: HashMap<FlowKey, TlsFlowState>` (per-flow state isolation)
- `src/analyzer/tls.rs` — `on_data(flow_key, direction, data, ts)` signature (direction param drives carry selection)
- `src/analyzer/tls.rs` — carry append: `match direction { ClientToServer => &mut state.client_hs_carry, ServerToClient => &mut state.server_hs_carry }`
- `tests/tls_analyzer_tests.rs` — `proptest_vp039_direction_isolation` (cross-DIRECTION isolation, Sub-E proptest)
- `tests/tls_analyzer_tests.rs` — `test_BC_2_07_041_cross_flow_isolation` (cross-FLOW isolation, dedicated unit test, architect authoring in VP-039)

## Story Anchor

STORY-145 (TLS ServerHello Fragmentation Symmetry + Per-Direction Isolation — BC primary; wave 66)

## VP Anchors

VP-039 (Sub-E)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads direction parameter; accesses per-flow TlsFlowState carry fields only through the FlowKey-keyed HashMap |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
