---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - "v1.3: 2026-05-26 | product-owner | Wave 10 STORY-017 pass-1 F-005 (wrong field range): small_segment_ignore_ports anchor config.rs:62-104 → :90-104 (line 62 is in small_segment_alert_threshold doc-comment; ignore_ports doc-comment starts at 90, declaration at 104)."
  - "v1.4: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:457-488 → mod.rs:486-517 (small-segment threshold block); mod.rs:356-370 → mod.rs:385-399 (small_segment_run counter maintenance). — 2026-06-01"
  - "v1.5: ARP-F2 Pass-14 Burst 4 — Postconditions mitre_technique: None → mitre_techniques: vec![] (shipped Finding struct uses Vec<String>; ADR-006). — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.020: Excessive Small Segments Emit One-Shot Finding

## Description

When the per-direction `small_segment_run` (the current consecutive run of undersized TCP
segments) exceeds `config.small_segment_alert_threshold` (default: 100) AND neither endpoint
port is in `config.small_segment_ignore_ports`, `check_anomaly_thresholds` emits a single
`Anomaly/Inconclusive/Medium` finding (no MITRE technique) and sets `small_segment_alert_fired
= true`. The latch is per-direction, per-flow. Small segment detection uses a CONSECUTIVE run
counter, not a cumulative count -- a normal-sized segment resets the run to zero.

## Preconditions

1. `flow_dir.small_segment_run > config.small_segment_alert_threshold`.
2. `flow_dir.small_segment_alert_fired == false`.
3. Neither `key.lower_port()` nor `key.upper_port()` is in `config.small_segment_ignore_ports`.

## Postconditions

1. `flow_dir.small_segment_alert_fired = true` (unconditional -- latch fires before cap check).
2. If under MAX_FINDINGS cap: Finding pushed with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Medium
   - mitre_techniques: vec![]
   - summary: "Excessive consecutive small segments (N) on flow <key>"
3. If at MAX_FINDINGS cap: `stats.dropped_findings` increments; no Finding pushed.

## Invariants

1. `small_segment_run` is a CONSECUTIVE run counter. Any segment with payload >=
   `small_segment_max_bytes` resets it to 0. Empty payloads (pure ACKs) do NOT affect the run.
2. The port-exemption check uses `any(|&p| p == key.lower_port() || p == key.upper_port())`.
   If EITHER endpoint port is in the ignore list, the alert is permanently suppressed for that
   flow regardless of run length.
3. Segments with InsertResult::OutOfWindow, SegmentLimitReached, DepthExceeded, or IsnMissing
   do NOT contribute to or reset the run -- they are "turned away before the window."
4. The latch is per-direction per-flow (same LESSON-P1.01 design as overlap latch).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Run of 100 small segments then one large | Run resets to 0 on large segment; no alert |
| EC-002 | Run of 101 small segments | Alert fires on 101st |
| EC-003 | Port 23 (telnet) flow; 1000 small segments | No alert (port-exempt) |
| EC-004 | Port 514 (not in ignore list); 101 small segments | Alert fires |
| EC-005 | Alert already fired; 1000 more small segments | No additional findings |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| threshold=5; 6 small segments (payload < 16 bytes) | Alert fires on 6th; Anomaly/Inconclusive/Medium | happy-path |
| threshold=5; 5 small + 1 large + 5 small | No alert (run reset by large segment) | edge-case |
| Port 23; any number of small segments | No alert | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Alert fires exactly once per direction per flow | unit |
| — | Normal-sized segment resets run counter | unit: run_of_5_small + 1_large; assert no alert |
| — | Port-exempt flows never alert | unit: port=23; 1000 small segments |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- small-segment detection is part of the IDS evasion detection contract |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap) |
| Architecture Module | SS-04 (reassembly/mod.rs:486-517, small-segment threshold block) |
| Stories | STORY-017 |
| Origin BC | BC-RAS-020 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.022 -- composes with (per-direction sticky latch)
- BC-2.04.040 -- composes with (small_segment_run counter update logic)
- BC-2.04.019 -- related to (overlap threshold alert; same latch pattern)

## Architecture Anchors

- `src/reassembly/mod.rs:486-517` -- small-segment threshold check and emission
- `src/reassembly/mod.rs:385-399` -- small_segment_run counter maintenance in insert_payload_segment
- `src/reassembly/config.rs:90-104` -- small_segment_ignore_ports field (doc-comment + declaration)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:486-517` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if flow_dir.small_segment_run > threshold && !alert_fired && !port_exempt`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow_dir.small_segment_alert_fired, self.findings, self.stats |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. The port-exemption scan runs last (`&&` short-circuit), minimizing
overhead for the common non-alert case.
