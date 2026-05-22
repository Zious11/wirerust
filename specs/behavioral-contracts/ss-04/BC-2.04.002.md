---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: Wave 5 Ph3 per-story adversarial fix Min-1: synced Traceability anchor 186-190 → 187-190 to match the BC's own Architecture Anchors and STORY-012 v1.3 — author: product-owner — 2026-05-22"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.002: Non-TCP Packets Skipped; packets_skipped_non_tcp Increments

## Description

`TcpReassembler::process_packet` checks `packet.protocol` at the top of the hot path. Any
packet whose protocol is not `Protocol::Tcp` is immediately discarded with no flow state
created or modified. The counter `ReassemblyStats.packets_skipped_non_tcp` is incremented
for every such skip. The `packets_processed` counter increments unconditionally before the
protocol check, so every call is counted; the TCP subset is tracked by `packets_tcp` which
increments only when the protocol check passes.

## Preconditions

1. `TcpReassembler::process_packet` is called with a `ParsedPacket`.
2. `packet.protocol != Protocol::Tcp` (e.g., UDP, ICMP, Protocol::Other(_)).

## Postconditions

1. `stats.packets_processed` increments by 1.
2. `stats.packets_skipped_non_tcp` increments by 1.
3. `stats.packets_tcp` does NOT change.
4. No flow is created, modified, or evicted.
5. No findings are emitted.
6. The handler receives no `on_data` or `on_flow_close` callbacks.

## Invariants

1. `packets_processed >= packets_tcp + packets_skipped_non_tcp` for all time (packets_processed
   may exceed the sum if the TransportInfo::Tcp extraction fails after protocol == Tcp).
2. The skip is silent; no error is returned or logged.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | UDP packet | Skipped; packets_skipped_non_tcp++ |
| EC-002 | ICMP packet (Protocol::Icmp) | Skipped; packets_skipped_non_tcp++ |
| EC-003 | Protocol::Other(n) packet | Skipped; packets_skipped_non_tcp++ |
| EC-004 | Protocol::Tcp but TransportInfo is not Tcp (decoder inconsistency) | extract_tcp_context returns None; NOT counted in packets_skipped_non_tcp (separate None path) |
| EC-005 | All packets non-TCP | flows.is_empty(), findings.is_empty() after all packets processed |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Single UDP ParsedPacket | packets_processed=1, packets_skipped_non_tcp=1, packets_tcp=0 | happy-path |
| 5 UDP + 3 TCP packets | packets_processed=8, packets_skipped_non_tcp=5, packets_tcp=3 | happy-path |
| ICMP packet | packets_processed=1, packets_skipped_non_tcp=1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | packets_skipped_non_tcp increments exactly once per non-TCP call | unit: single UDP packet |
| — | No flow state created for non-TCP input | unit: assert flows.is_empty() after N non-TCP packets |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- this BC defines the boundary filter that restricts reassembly to TCP streams only |
| L2 Domain Invariants | None directly |
| Architecture Module | SS-04 (reassembly/mod.rs:187-190, extract_tcp_context, C-6) |
| Stories | STORY-012 |
| Origin BC | BC-RAS-002 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.003 -- depends on (FlowKey only built when this check passes)
- BC-2.04.028 -- related to (packets_skipped_non_tcp surfaced in summarize)

## Architecture Anchors

- `src/reassembly/mod.rs:140-145` -- process_packet entry; packets_processed++, extract_tcp_context call
- `src/reassembly/mod.rs:187-190` -- extract_tcp_context: packets_skipped_non_tcp++ for non-TCP

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:187-190` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if packet.protocol != Protocol::Tcp` with immediate increment and return

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (simple counter mutation) |

## Refactoring Notes

No refactoring needed. The counter mutation is minimal and the guard is a single check.
