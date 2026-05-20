---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.021: Excessive Out-of-Window Segments Emit One-Shot Low Finding

## Description

When the per-direction `out_of_window_count` exceeds `config.out_of_window_alert_threshold`
(default: 100), `check_anomaly_thresholds` emits a single `Anomaly/Inconclusive/Low` finding
(no MITRE technique) and sets `out_of_window_alert_fired = true`. The finding includes a
formatted evidence string citing the `max_receive_window` value. The latch pattern is the
same as for overlap and small-segment alerts (LESSON-P1.01).

## Preconditions

1. `flow_dir.out_of_window_count > config.out_of_window_alert_threshold`.
2. `flow_dir.out_of_window_alert_fired == false`.
3. `check_anomaly_thresholds` is called after an `OutOfWindow` insert result.

## Postconditions

1. `flow_dir.out_of_window_alert_fired = true` (unconditional).
2. If under MAX_FINDINGS cap: Finding pushed with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Low
   - mitre_technique: None
   - summary: "Excessive out-of-window segments (N) on flow <key>"
   - evidence: ["max_receive_window=W bytes; possible misconfiguration, evasion, or capture corruption"]
3. If at MAX_FINDINGS cap: `stats.dropped_findings` increments; no Finding pushed.

## Invariants

1. `out_of_window_count` is incremented by `insert_segment` when `offset > base_offset +
   max_receive_window`. It is a CUMULATIVE counter (not a run counter).
2. The latch fires once per direction per flow.
3. The evidence string format is fixed: `"max_receive_window={window} bytes; possible ..."`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | out_of_window_count == threshold exactly | No alert (strictly greater) |
| EC-002 | out_of_window_count == threshold + 1 | Alert fires |
| EC-003 | Alert fires when findings at cap | Latch set; dropped_findings++ |
| EC-004 | Both directions exceed threshold | Two separate findings (one per direction) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| threshold=3; 4 OOW segments | Alert on 4th; Anomaly/Inconclusive/Low; evidence includes window size | happy-path |
| threshold=3; 100 OOW segments | Single alert; latch prevents rest | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Out-of-window alert fires exactly once per direction | unit |
| VP-TBD | Evidence string contains max_receive_window value | unit: assert evidence[0] contains window bytes |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- out-of-window detection bounds segment buffering and detects potential evasion |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap) |
| Architecture Module | SS-04 (reassembly/mod.rs:489-512, out-of-window threshold block) |
| Stories | S-TBD |
| Origin BC | BC-RAS-021 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.022 -- composes with (per-direction sticky latch)
- BC-2.04.042 -- depends on (out-of-window detection in insert_segment)
- BC-2.04.019 -- related to (same latch pattern)

## Architecture Anchors

- `src/reassembly/mod.rs:489-512` -- out-of-window threshold check and emission
- `src/reassembly/segment.rs:63-67` -- out_of_window_count increment in insert_segment

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:489-512` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if flow_dir.out_of_window_count > threshold && !alert_fired`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow_dir.out_of_window_alert_fired, self.findings, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed.
