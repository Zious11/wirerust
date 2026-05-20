---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
extracted_from: src/reassembly/lifecycle.rs
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

# BC-2.04.023: Truncated Segment Emits Anomaly/Inconclusive/Low Finding

## Description

When `insert_segment` returns `InsertResult::Truncated` (the segment was accepted but
truncated because it crossed the `max_depth` byte boundary), the reassembly engine calls
`generate_truncated_finding` which emits an `Anomaly/Inconclusive/Low` finding with no MITRE
technique. The summary names the flow and the evidence cites the `max_depth` value. The
finding is subject to the MAX_FINDINGS cap. The truncated bytes are stored and will be
delivered (truncated) to the handler; the `depth_exceeded` flag is set on the direction.

## Preconditions

1. `insert_segment` returns `InsertResult::Truncated`.
2. `generate_truncated_finding` is called from the `InsertResult::Truncated` match arm.
3. `self.findings.len() < MAX_FINDINGS`.

## Postconditions

1. If under MAX_FINDINGS cap: Finding pushed with:
   - category: Anomaly
   - verdict: Inconclusive
   - confidence: Low
   - mitre_technique: None
   - summary: "Stream depth exceeded on flow <key>"
   - evidence: ["Max depth N bytes reached"] where N = config.max_depth
   - source_ip: Some(packet.src_ip)
2. If at MAX_FINDINGS cap: `stats.dropped_findings` increments; no Finding pushed.
3. The truncated segment data IS inserted (gap bytes up to max_depth are stored).
4. `stats.segments_inserted` increments (Truncated counts as a partial insertion).

## Invariants

1. A Truncated result means the segment was accepted with fewer bytes than submitted; the
   `depth_exceeded` flag on the direction is set to `true` by `insert_segment`.
2. Once `depth_exceeded == true`, all subsequent inserts for that direction return
   `DepthExceeded` (not Truncated). There is at most one Truncated event per direction.
3. This finding uses `source_ip` but NOT a `direction` field (None for direction).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Segment exactly at depth limit (no truncation) | InsertResult::Inserted, no Truncated finding |
| EC-002 | Segment crosses depth limit by 1 byte | Truncated; finding emitted; truncated bytes stored |
| EC-003 | Segment arrives after depth already exceeded | InsertResult::DepthExceeded (not Truncated) |
| EC-004 | Truncated at MAX_FINDINGS | Finding dropped; dropped_findings++ |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| max_depth=100; insert 110-byte segment when 95 bytes reassembled | Truncated to 5 bytes; finding emitted | happy-path |
| max_depth=100; insert after depth exceeded | DepthExceeded; no Truncated finding | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Truncated finding emitted exactly once per direction | unit: two segments that both cross depth; first is Truncated, second is DepthExceeded |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- depth truncation signaling is part of the bounded-resource forensic contract |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap) |
| Architecture Module | SS-04 (reassembly/lifecycle.rs:120-136, generate_truncated_finding; mod.rs:383-385, Truncated match arm) |
| Stories | S-TBD |
| Origin BC | BC-RAS-023 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.041 -- depends on (depth truncation in insert_segment)
- BC-2.04.024 -- related to (MAX_FINDINGS cap can suppress this finding)

## Architecture Anchors

- `src/reassembly/lifecycle.rs:120-136` -- generate_truncated_finding
- `src/reassembly/mod.rs:383-385` -- Truncated match arm: segments_inserted++, generate_truncated_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/lifecycle.rs:120-136` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: cap check before push in generate_truncated_finding
- **type constraint**: InsertResult::Truncated is a distinct enum variant

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.findings and stats.dropped_findings |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |

## Refactoring Notes

No refactoring needed.
