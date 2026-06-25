---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified:
  - "v1.1 F3 story-convergence: flows_analyzed increment site added (F-P6-001 dead-counter fix)"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.017: on_flow_close Removes Flow State and Updates Aggregate Counters

## Description

`EnipAnalyzer::on_flow_close(flow_key)` is called when the TCP flow identified by `flow_key`
closes. The function removes the `EnipFlowState` entry from the per-flow `HashMap` and updates
aggregate-level counters (`total_pdu_count`, `parse_errors`, `command_distribution`) by
folding the closed flow's state into the analyzer's aggregate fields. This mirrors the DNP3
`on_flow_close` pattern. After this call, no further `on_data` calls will arrive for this
flow_key.

## Preconditions

1. `flow_key` identifies a flow previously tracked in `EnipAnalyzer.flows`.
2. `EnipAnalyzer::on_flow_close(flow_key)` is called after the TCP session teardown.

## Postconditions

1. `self.flows.remove(&flow_key)` — `EnipFlowState` for this flow is removed from the map.
2. `self.total_pdu_count += flow.pdu_count` — PDU count from closed flow folded into aggregate.
3. `self.parse_errors += flow.parse_errors` — lifetime parse error count folded into aggregate.
4. For each `(cmd, count)` in `flow.command_counts`:
   `self.command_distribution.entry(cmd).or_insert(0) += count`.
5. If `flow_key` is not found in `self.flows` (already closed or never opened): no-op (no panic).
6. **`self.flows_analyzed += 1`** — the count of distinct TCP flows that have been fully analyzed
   (closed and drained) is incremented. This is the only increment site for `flows_analyzed`;
   it fires for every successful `flows.remove` (Postcondition 5 no-op exception: unknown
   flow_key does NOT increment `flows_analyzed`). Reported by `summarize()` as
   `enip_summary.flows_analyzed` (BC-2.17.021). Mirrors the DNP3/Modbus closed-flow counting
   pattern — a flow is "analyzed" when its lifetime state has been folded into aggregates.
7. Findings already in `self.all_findings` are not affected by flow close.

## Invariants

1. **No double-free**: `HashMap::remove` returns `Option`; missing key is handled gracefully
   (no panic). An absent flow_key on `on_flow_close` is silently ignored.
2. **Aggregate counters are additive**: `total_pdu_count`, `parse_errors`, `command_distribution`,
   and `flows_analyzed` grow monotonically; they are never decremented by flow close.
3. **flows_analyzed is the ONLY increment site**: `EnipAnalyzer.flows_analyzed` is incremented
   exclusively here, on successful flow removal (`HashMap::remove` returns `Some`). It is
   NOT incremented on first-PDU, not in `on_data`, and not in `summarize()`. This mirrors the
   DNP3/Modbus "closed-flow count" pattern and ensures `flows_analyzed` equals the number of
   TCP sessions that have been fully processed and torn down.
4. **Findings persist**: all findings emitted during the flow's lifetime remain in
   `self.all_findings` after the flow is closed.
5. **Memory reclamation**: `EnipFlowState` (including its carry Vec<u8>) is dropped by Rust's
   ownership rules when removed from the HashMap — no explicit memory management required.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow with zero PDUs (immediately closed) | `flow.pdu_count=0` folded; aggregate unchanged |
| EC-002 | Flow with is_non_enip=true | State removed; parse_errors folded in (captures error from is_non_enip trigger) |
| EC-003 | `on_flow_close` called for unknown flow_key | No-op; no panic |
| EC-004 | Two flows closed sequentially | Aggregate totals reflect sum of both flows |

## Canonical Test Vectors

| Flow state before close | Expected aggregate delta |
|------------------------|------------------------|
| `pdu_count=10, parse_errors=2, command_counts={0x006F: 8, 0x0063: 2}` | `total_pdu_count += 10; parse_errors += 2; command_distribution[0x006F] += 8; command_distribution[0x0063] += 2; flows_analyzed += 1` |
| `pdu_count=0, parse_errors=0` | `flows_analyzed += 1` (even zero-PDU flows that opened are counted) |
| `is_non_enip=true, parse_errors=1` | `parse_errors += 1; flows_analyzed += 1` |
| Unknown flow_key (not in self.flows) | no-op; `flows_analyzed` NOT incremented |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Flow removal, aggregate fold, no-panic on missing key: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — flow-close lifecycle management is required for correct aggregate statistics in `summarize()` and for reclaiming per-flow memory after TCP session teardown |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (EnipAnalyzer design) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — lifecycle management; no finding emission) |

## Related BCs

- BC-2.17.021 — composes with (aggregate counters updated here, including flows_analyzed, are consumed by summarize())
- BC-2.17.016 — depends on (carry memory released on flow close)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipAnalyzer::on_flow_close(flow_key)` — StreamHandler trait method
- `src/analyzer/enip.rs` — `EnipAnalyzer.flows: HashMap<FlowKey, EnipFlowState>`
- `src/analyzer/enip.rs` — `EnipAnalyzer.total_pdu_count: u64`, `.parse_errors: u64`, `.command_distribution: HashMap<u16, u64>`
- `src/analyzer/enip.rs` — `EnipAnalyzer.flows_analyzed: u64` — **sole increment site: `if let Some(flow) = self.flows.remove(&flow_key) { self.flows_analyzed += 1; ... }`**

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — lifecycle management; effectful shell; unit test)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (EnipAnalyzer aggregate counter design); enip-architecture-delta.md §4.2 |
| **Confidence** | high — mirrors established DNP3/Modbus pattern |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | removes from flows HashMap; mutates total_pdu_count, parse_errors, command_distribution, flows_analyzed |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell |
