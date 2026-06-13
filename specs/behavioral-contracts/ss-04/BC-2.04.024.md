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
  - "v1.3: DF-SIBLING-SWEEP-001 HS-043 re-anchor: guard check sites mod.rs:432,466,495 → mod.rs:461,495,524 (check_anomaly_thresholds guards). — 2026-06-01"
  - "v1.4: PATCH — Pass-19 B-09 anchor fix: MAX_FINDINGS const mod.rs:54→:56; guard sites mod.rs:461,495,524→:479,515,546 (F2 timestamp wiring shifted content). No functional postcondition change. — 2026-06-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.024: Total Findings Capped at MAX_FINDINGS=10000; Excess Silently Dropped

## Description

The reassembly engine's `findings: Vec<Finding>` is hard-capped at `MAX_FINDINGS = 10,000`.
Guard checks at multiple emission points return early if `self.findings.len() >= MAX_FINDINGS`.
When a finding is dropped, `ReassemblyStats.dropped_findings: u64` is incremented, making the
cap observable. The finalize bypass (BC-2.04.054) is the only exception to this cap.

Note: `HttpAnalyzer.all_findings` and `TlsAnalyzer.all_findings` are NOT subject to this cap.
Only the reassembly engine enforces MAX_FINDINGS.

## Preconditions

1. The reassembly engine has been processing packets that generate findings.
2. `self.findings.len() >= MAX_FINDINGS` (= 10,000).
3. A new finding would normally be emitted.

## Postconditions

1. The new finding is NOT added to `self.findings`.
2. `ReassemblyStats.dropped_findings` is incremented by 1.
3. No error is returned to the caller; processing continues normally.
4. The existing 10,000 findings remain unchanged.

## Invariants

1. `self.findings.len() <= MAX_FINDINGS + 1` (the +1 is only possible via finalize bypass).
2. `dropped_findings` is monotonically non-decreasing.
3. The cap is per-reassembler-instance, per-run (not persistent across runs).
4. HttpAnalyzer and TlsAnalyzer findings are NOT counted toward this cap.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | findings.len() == 9999 (one below cap) | Finding is added; findings.len() becomes 10000 |
| EC-002 | findings.len() == 10000 (at cap) | Finding is dropped; dropped_findings++ |
| EC-003 | finalize() called when at cap | Segment-limit summary finding is added unconditionally (findings.len() becomes 10001) |
| EC-004 | Multiple findings dropped consecutively | dropped_findings reflects all dropped counts |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| 10001 conflict events with MAX_FINDINGS=10000 | findings.len()==10000, dropped_findings==1 | happy-path |
| finalize() when findings at cap | findings.len()==10001 (one bypass); dropped_findings unchanged by finalize | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-003 | findings.len() never exceeds MAX_FINDINGS + 1 | proptest: generate arbitrary number of findings |
| VP-003 | dropped_findings == (total_findings_attempted - min(total_attempts, MAX_FINDINGS)) | unit |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- MAX_FINDINGS cap is the primary resource-bounding mechanism for the reassembly engine |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap with cap-bypass for finalize) |
| Architecture Module | SS-04 (reassembly/mod.rs:56,479,515,546, C-6; reassembly/lifecycle.rs:101,121, C-15) |
| Stories | STORY-021 |
| Origin BC | BC-RAS-024 (pass-3 ingestion corpus, MEDIUM confidence -- not directly tested) |

## Related BCs

- BC-2.04.054 -- related to (finalize bypasses this cap unconditionally)
- BC-2.04.025 -- related to (segment-limit summary finding uses the bypass)

## Architecture Anchors

- `src/reassembly/mod.rs:56` -- `const MAX_FINDINGS: usize = 10_000`
- `src/reassembly/mod.rs:479,515,546` -- guard check sites in check_anomaly_thresholds
- `src/reassembly/lifecycle.rs:101,121` -- guard check sites in generate_conflicting_overlap_finding and generate_truncated_finding
- `src/reassembly/stats.rs` -- dropped_findings: u64 field

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:56` (const), `mod.rs:479,515,546` (check_anomaly_thresholds guards), `lifecycle.rs:101,121` (generate_* guards) |
| **Confidence** | medium |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: two guard forms across 5 sites -- (a) inverted early-return form `if self.findings.len() >= MAX_FINDINGS { self.stats.dropped_findings += 1; return; }` at `lifecycle.rs:101` and `lifecycle.rs:121`; (b) positive conditional form `if self.findings.len() < MAX_FINDINGS { self.findings.push(...); } else { self.stats.dropped_findings += 1; }` at `mod.rs:479`, `mod.rs:515`, and `mod.rs:546`

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.findings and stats.dropped_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |
