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

# BC-2.04.025: finalize Emits Segment-Limit Summary Finding When Segments Dropped

## Description

During `finalize`, after all flows are closed, if `stats.segments_segment_limit > 0` the
engine pushes one aggregate `Anomaly/Inconclusive/Medium` finding summarizing the total
number of segments dropped due to the per-direction segment-count limit. This finding is
pushed UNCONDITIONALLY, bypassing the MAX_FINDINGS cap (BC-2.04.054). The summary uses the
`plural_s` helper for grammatically correct singular/plural. The finding carries no source_ip,
no direction, and no MITRE technique.

## Preconditions

1. `finalize` has been called and `finalized` is being set to true.
2. `stats.segments_segment_limit > 0` (at least one segment was rejected due to the limit).

## Postconditions

1. One Finding is pushed to `self.findings`:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Medium
   - mitre_technique: None
   - summary: "N segment[s] dropped due to per-flow segment count limit"
     (singular if N==1: "1 segment dropped...", plural otherwise: "N segments dropped...")
   - evidence: ["Segment count limit prevents BTreeMap overhead explosion",
                "May indicate segmentation-based evasion attempt"]
   - source_ip: None
   - direction: None
2. This push happens even if `findings.len() >= MAX_FINDINGS`.

## Invariants

1. This push is the ONLY path that bypasses the MAX_FINDINGS cap. After this push,
   `findings.len()` may equal `MAX_FINDINGS + 1`.
2. The finding is emitted at most once per finalize call.
3. The `plural_s` helper: returns `""` if count == 1, `"s"` otherwise.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | segments_segment_limit == 1 | Summary: "1 segment dropped due to per-flow segment count limit" |
| EC-002 | segments_segment_limit == 5 | Summary: "5 segments dropped due to per-flow segment count limit" |
| EC-003 | findings.len() == MAX_FINDINGS when finalize runs | Push happens unconditionally; findings.len() becomes MAX_FINDINGS + 1 |
| EC-004 | segments_segment_limit == 0 | No finding emitted (governed by BC-2.04.026) |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| segments_segment_limit=1; finalize | "1 segment dropped..." in findings | happy-path |
| segments_segment_limit=100; finalize | "100 segments dropped..." | happy-path |
| findings at MAX_FINDINGS; finalize with limit=5 | findings.len() == MAX_FINDINGS + 1 | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| — | Singular/plural grammar in summary is correct | unit: limit=1 and limit=2 |
| — | Finding pushed even at MAX_FINDINGS | unit: fill cap; trigger limit; finalize |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- segment-limit summary finding is the forensic signal for BTreeMap overflow protection |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap; finalize bypass), INV-7 (finalize-once latch) |
| Architecture Module | SS-04 (reassembly/mod.rs:571-590, segment-limit block in finalize) |
| Stories | S-TBD |
| Origin BC | BC-RAS-025 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.026 -- inverse (zero counter suppresses the finding)
- BC-2.04.054 -- composes with (finalize bypass of MAX_FINDINGS)
- BC-2.04.012 -- composes with (finalize lifecycle contract)
- BC-2.04.044 -- depends on (segments_segment_limit incremented by SegmentLimitReached)

## Architecture Anchors

- `src/reassembly/mod.rs:571-590` -- segment-limit finding block
- `src/reassembly/mod.rs:66-68` -- plural_s helper

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:571-590` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **assertion**: unconditional self.findings.push() -- no MAX_FINDINGS guard

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed. The unconditional push is intentional and load-bearing.
