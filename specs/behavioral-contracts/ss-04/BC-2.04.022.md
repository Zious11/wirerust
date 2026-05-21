---
document_type: behavioral-contract
level: L3
version: "1.2"
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
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.022: Per-Direction Alert Fires At Most Once Per Flow (Sticky Latch)

## Description

All three anomaly threshold alerts (overlap, small-segment, out-of-window) use a per-direction
sticky latch (`overlap_alert_fired`, `small_segment_alert_fired`, `out_of_window_alert_fired`)
that is set to `true` the moment the threshold is first crossed. The latch is tested BEFORE
the MAX_FINDINGS cap check -- it is set unconditionally. Once set, it prevents any subsequent
threshold crossing (even after thousands more packets) from attempting to emit another finding
or incrementing dropped_findings again. This is LESSON-P1.01 (LESSON-P0.03 in older docs).

## Preconditions

1. A threshold for one of the three anomaly types has just been crossed for the first time
   on a specific (flow, direction) pair.
2. The corresponding alert latch is currently `false`.

## Postconditions

1. The alert latch is set to `true` before any other action.
2. A finding is emitted if under MAX_FINDINGS cap (or dropped_findings++ if at cap).
3. On all subsequent packets for this (flow, direction): the threshold guard short-circuits
   immediately on `!flow_dir.XXX_alert_fired`, performing no further work.
4. `dropped_findings` is incremented at most ONCE per (flow, direction, alert type), even if
   the threshold is crossed thousands more times while at cap.

## Invariants

1. The latch is monotonically false-to-true; it never resets within a flow's lifetime.
2. The latch fires regardless of whether the finding is actually emitted (cap may suppress it).
3. The worst case per bidirectional flow is 6 findings (3 alert types x 2 directions).
4. `dropped_findings` counts DISTINCT anomalies lost (not packets). A flow that trips all 3
   alerts in 1 direction at cap contributes at most 3 to dropped_findings, not thousands.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Threshold crossed at finding slot 9999 (under cap) | Finding emitted; latch set |
| EC-002 | Threshold crossed exactly at MAX_FINDINGS | Latch set; dropped_findings++; no finding |
| EC-003 | Same threshold crossed 10000 times | Latch set on first crossing; 9999 subsequent crossings are no-ops |
| EC-004 | ClientToServer latch set; ServerToClient still unlocked | S2C can still fire independently |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Overlap: threshold=3; 10000 overlapping segments | Exactly 1 overlap-threshold finding; latch=true | happy-path |
| At MAX_FINDINGS; threshold crossed | Latch=true; dropped_findings++ exactly once | edge-case |
| Both directions cross overlap threshold | Exactly 2 overlap-threshold findings (one per direction) | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Latch-before-cap: latch=true even when finding suppressed | unit: fill to cap; trigger threshold; assert latch=true and dropped_findings==1 |
| — | Three alert types each fire at most once per direction per flow | proptest: generate stream with all three threshold crossings |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- the sticky latch is the resource-bounding mechanism for the three anomaly detectors |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap; dropped_findings observability) |
| Architecture Module | SS-04 (reassembly/mod.rs:420-512, check_anomaly_thresholds; flow.rs:86-108, FlowDirection fields) |
| Stories | STORY-017 |
| Origin BC | BC-RAS-022 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.019 -- composes with (overlap latch)
- BC-2.04.020 -- composes with (small-segment latch)
- BC-2.04.021 -- composes with (out-of-window latch)
- BC-2.04.024 -- depends on (MAX_FINDINGS cap interacts with latch)

## Architecture Anchors

- `src/reassembly/mod.rs:420-426` -- LESSON-P1.01 comment explaining design
- `src/reassembly/flow.rs:100-105` -- latch fields: overlap_alert_fired, small_segment_alert_fired, out_of_window_alert_fired

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:430,465,489` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if count > threshold && !alert_fired` at each threshold check
- **documentation**: LESSON-P1.01 explicitly documents latch-before-cap design intent

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates latch fields, self.findings, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. The latch-before-cap pattern is correct, documented, and intentional.
