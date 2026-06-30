---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.1: Pass-1 adversarial reconciliation (F-P1-001/SR-001) — remove hs_carry_abandoned precondition and EC-006 text; there is no sticky abandoned state in the clear-and-recover policy; EC-006 rewritten to reflect that an overflowed (cleared) carry is simply empty at flow close — 2026-06-29"
  - "v1.2: Pass-2 adversarial reconciliation (F-F2-009 MEDIUM) — PC-4 tightened: flow-close discard produces NO ADDITIONAL counter increment by itself; a prior overflow on this flow may have already incremented TlsAnalyzer.handshake_reassembly_overflows (an aggregate, not a per-flow counter); PC-4 must not imply handshake_reassembly_overflows==0 at close — 2026-06-29"
  - "v1.3: Pass-3 adversarial reconciliation (F-P3-LOW) — Related BCs line: stale phrase 'abandoned carry is already empty' → 'overflow-cleared carry is already empty'; no abandoned-direction concept exists in the clear-and-recover policy — 2026-06-29"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.07.040: Truncated Handshake at Flow Close Yields No Finding and No parse_errors Increment

## Description

When `on_flow_close` is called for a flow that has an incomplete handshake message
accumulated in either carry buffer (the handshake length header is present but not all
body bytes have arrived, or even only a partial length header is present), the carry
buffer is silently discarded without emitting any finding and without incrementing
`parse_errors`. This preserves existing truncation semantics for snaplen-truncated
captures (READER cand-05 interaction): a capture truncated mid-handshake is
byte-for-byte indistinguishable from an incomplete fragment at capture end, and both
are treated as "nothing to report" rather than as parse failures. The design ensures
that well-maintained traffic captures do not generate false positive parse errors
merely because the capture ended before a handshake message completed.

## Preconditions

1. `TlsAnalyzer::on_flow_close` is called for an active flow.
2. The flow has zero or more bytes in `client_hs_carry` or `server_hs_carry` that do
   not form a complete handshake message (i.e., `carry_buf.len() < 4` or
   `carry_buf.len() < 4 + body_len`, or `carry_buf.len() == 0`).

## Postconditions

1. `on_flow_close` removes the flow's `TlsFlowState` from `self.flows` via
   `HashMap::remove(flow_key)`, which drops all fields including the carry buffers.
2. `parse_errors` is NOT incremented (the incomplete carry is not a parse error).
3. No `Finding` is pushed to `all_findings` for the incomplete handshake.
4. The flow-close discard of incomplete carry bytes produces NO ADDITIONAL increment to
   `TlsAnalyzer.handshake_reassembly_overflows` — the carry is simply dropped as part
   of the normal `HashMap::remove` flow teardown. A prior overflow event on this flow
   may have already incremented the aggregate counter (before flow close) — that prior
   increment is unaffected and persists on the analyzer. No new counter increment, no
   log message, and no other observable side effect are produced by the flow-close
   carry-bytes discard itself.
5. The behavior is identical whether the carry buffer has 0 bytes, 1–3 bytes (partial
   header), or header-complete but body-incomplete bytes.

## Invariants

1. `parse_errors` is incremented ONLY for records that fail `parse_tls_plaintext`
   (malformed TLS record layer) or whose extension bytes fail `parse_tls_extensions`
   (malformed extension encoding). An incomplete carry buffer at flow close is NOT
   a parse error and MUST NOT increment `parse_errors`.
2. An empty or partial carry buffer at flow close produces zero findings from the
   reassembly layer. This is the "nothing to report" semantics: absence of evidence
   is not evidence of malformation.
3. Snaplen-truncated captures are indistinguishable from incomplete fragments at the
   carry buffer level. Both result in the same zero-finding, zero-error behavior. No
   special EPB `original_len` handling is needed at this layer; the pcapng reader
   (READER cand-05) handles the truncation at the packet level.
4. The `on_flow_close` path for carry buffers introduces no new behavior beyond what
   `HashMap::remove` already provides for the `TlsFlowState` drop — the existing flow
   removal mechanism is sufficient; no explicit carry-clearing is needed.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow closes with exactly 1 byte in `client_hs_carry` (partial type byte) | Carry discarded via flow removal; `parse_errors=0`; no finding |
| EC-002 | Flow closes with 3 bytes in `client_hs_carry` (type + 2 of 3 length bytes — incomplete header) | Same: `parse_errors=0`; no finding |
| EC-003 | Flow closes with complete 4-byte header but body not yet arrived (e.g., header says body_len=500, only 4 bytes present) | Same: `parse_errors=0`; no finding |
| EC-004 | Flow closes with empty carry buffers on both directions | No-op beyond normal flow removal; no change to any counter |
| EC-005 | Flow closes mid-handshake due to snaplen truncation in EPB (pcapng EPB `original_len > captured_len`) | Indistinguishable from EC-001/002/003 at this layer; same zero-error, zero-finding outcome |
| EC-006 | Flow closes after a prior overflow event (carry was cleared by BC-2.07.039 clear-and-recover path) | Carry is empty (was cleared at overflow time); flow removal is a no-op for the carry; `parse_errors=0`; no finding; behavior identical to EC-004 (empty carry at close) — there is no abandoned flag |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Build a ClientHello, deliver first 50% as a 0x16 record, then call `on_flow_close` without delivering the remainder | `client_hello_seen=false`, `sni_counts.is_empty()=true`, `ja3_counts.is_empty()=true`, `parse_errors=0` | edge-case |
| Deliver exactly 3 bytes (partial header) as a 0x16 record, then call `on_flow_close` | `parse_errors=0`; no findings; `active_flows_len_for_testing()` decremented by 1 | edge-case |
| Normal flow close after complete ClientHello (carry is empty at close) | Existing behavior unchanged; `active_flows_len_for_testing()` decremented by 1 | regression |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-039 (Sub-D) | `on_flow_close` called with an incomplete carry buffer (partial length header present) does not increment `parse_errors` and does not emit a finding | unit: `test_vp039_truncated_carry_no_error` |
| — | Empty carry at flow close produces no observable effect beyond flow removal | unit: `test_BC_2_07_040_empty_carry_flow_close` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md |
| Capability Anchor Justification | CAP-07 ("TLS traffic analysis") per domain/capabilities/cap-07-tls-analysis.md — truncation-safety at flow close is a correctness property of the TLS analysis subsystem, preventing false-positive parse errors on snaplen-limited captures |
| L2 Domain Invariants | INV-4 (raw-data/display-layer separation) |
| Architecture Module | SS-07 (analyzer/tls.rs — `on_flow_close` via `HashMap::remove`; existing state-drop mechanism) |
| Finding Source | TLS-CLIENTHELLO-FRAG-001 validation §Q5 (snaplen-truncated captures / READER cand-05 interaction) |
| RFC Authority | RFC 5246 §6.2.1 (fragmented messages; truncation at capture end = incomplete fragment) |
| Stories | STORY-144 |
| Origin | greenfield (fix-tls-clienthello-frag cycle) |

## Related BCs

- BC-2.07.038 — depends on (carry buffer that may be incomplete at flow close)
- BC-2.07.039 — related to (overflow-cleared carry is already empty; truncated incomplete carry is distinct — not overflow-cleared, just never completed)
- BC-2.07.035 — composes with (on_flow_close drops TlsFlowState including carry fields; this BC specifies carry-specific truncation semantics)
- BC-2.07.029 — related to (bad record body → parse_errors++; truncated carry at close → parse_errors unchanged; these are distinct code paths)

## Architecture Anchors

- `src/analyzer/tls.rs` — `on_flow_close`: `self.flows.remove(flow_key)` drops `TlsFlowState` including carry fields; no explicit carry-clearing needed
- `tests/tls_analyzer_tests.rs` — `test_vp039_truncated_carry_no_error`
- `tests/tls_analyzer_tests.rs` — `active_flows_len_for_testing()` test seam (verifies flow removal)

## Story Anchor

STORY-144 (TLS Carry Buffer + ClientHello Fragmentation Reassembly — BC primary; wave 65)

## VP Anchors

VP-039 (Sub-D)

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | removes `TlsFlowState` from `self.flows` HashMap |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed |
