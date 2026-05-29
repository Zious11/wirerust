---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: 2026-05-26 | product-owner | Wave 10 STORY-017 pass-1 F-002 (sibling-regression of BC-2.04.022 v1.4 anchor fix): LESSON-P1.01 anchor mod.rs:420-426 → :413-419 (line range previously contained LESSON-P2.05). Sibling sweep across other SS-04 BCs performed."
  - "v1.4: W10-D3 fix — overlap if-block anchor corrected from 430-449 → 430-450 at all 3 sites (Traceability, Architecture Anchors, Source Evidence). The closing `}` of the if-block is at mod.rs:450, confirmed against source. — 2026-05-28"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.019: Excessive Overlaps Emit One-Shot T1036 Finding

## Description

When the per-direction `overlap_count` exceeds `config.overlap_alert_threshold` (default: 50),
`check_anomaly_thresholds` emits a single `Anomaly/Likely/Medium` finding tagged with MITRE
T1036 and sets `overlap_alert_fired = true` for that direction. The latch prevents any further
overlap-threshold findings for that direction on that flow, regardless of how many additional
overlapping segments arrive. The finding is subject to the MAX_FINDINGS cap.

Note: `overlap_count` counts ALL overlapping segments (including Duplicate, PartialOverlap,
and ConflictingOverlap results). The per-event ConflictingOverlap finding (BC-2.04.018) is a
separate, per-event mechanism. This threshold alert is a cumulative count-based anomaly.

## Preconditions

1. `flow_dir.overlap_count > config.overlap_alert_threshold` (strictly greater).
2. `flow_dir.overlap_alert_fired == false` (latch not yet set).
3. `process_packet` has just called `check_anomaly_thresholds` with this direction.

## Postconditions

1. `flow_dir.overlap_alert_fired` is set to `true` (unconditionally -- even if MAX_FINDINGS
   cap suppresses the finding).
2. If `findings.len() < MAX_FINDINGS`: a Finding is pushed with:
   - category: Anomaly
   - verdict: Likely
   - confidence: Medium
   - mitre_technique: Some("T1036")
   - summary: "Excessive segment overlaps (N) on flow <key>"
   - direction: Some(dir)
3. If `findings.len() >= MAX_FINDINGS`: `stats.dropped_findings` increments; no Finding pushed.
4. No further overlap-threshold findings are emitted for this (flow, direction) pair.

## Invariants

1. The latch fires UNCONDITIONALLY before the cap check. This is LESSON-P1.01: the latch
   prevents re-evaluation on subsequent packets and ensures dropped_findings reflects distinct
   anomalies, not repeated threshold crossings.
2. The threshold is per-direction, per-flow. A finding can fire once in ClientToServer and
   once in ServerToClient on the same flow.
3. `overlap_count` counts all overlapping segments, including benign retransmits.
   `ConflictingOverlap` (first-wins conflicts) increment overlap_count AND emit per-event
   findings via BC-2.04.018.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | overlap_count == threshold exactly | No alert (strictly greater) |
| EC-002 | overlap_count == threshold + 1 | Alert fires |
| EC-003 | overlap_count >> threshold | Still fires exactly once (latch) |
| EC-004 | Alert fires when findings cap is full | Latch set; dropped_findings++; no Finding |
| EC-005 | Separate alert for each direction | ClientToServer latch does not suppress ServerToClient alert |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| overlap_threshold=3; 4 overlapping segments | Alert fires on 4th segment; Anomaly/Likely/Medium/T1036 | happy-path |
| overlap_threshold=3; 10 overlapping segments | Single alert (latch); 9 subsequent overlaps silent | edge-case |
| overlap_threshold=3; 4 overlaps at MAX_FINDINGS | Latch fires; dropped_findings++ | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Overlap alert fires exactly once per direction per flow | unit: exceed threshold multiple times; assert findings.len() == 1 |
| — | Latch fires even when finding is dropped | unit: fill to MAX_FINDINGS; trigger overlap threshold; assert latch=true and dropped++ |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- cumulative overlap detection is part of the evasion detection contract |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap) |
| Architecture Module | SS-04 (reassembly/mod.rs:430-450, check_anomaly_thresholds overlap block) |
| Stories | STORY-017 |
| Origin BC | BC-RAS-019 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.018 -- related to (per-event ConflictingOverlap finding; separate mechanism)
- BC-2.04.022 -- composes with (per-direction sticky latch principle)
- BC-2.04.024 -- related to (MAX_FINDINGS cap can suppress this finding)

## Architecture Anchors

- `src/reassembly/mod.rs:430-450` -- overlap threshold check and finding emission
- `src/reassembly/mod.rs:413-419` -- LESSON-P1.01 comment explaining latch-before-cap design

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:430-450` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if flow_dir.overlap_count > threshold && !flow_dir.overlap_alert_fired`
- **documentation**: LESSON-P1.01 comment explicitly describes the latch-before-cap design

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow_dir.overlap_alert_fired, self.findings, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. The latch-before-cap pattern is explicitly documented and intentional.
