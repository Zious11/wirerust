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

# BC-2.04.018: Conflicting Overlap Emits Anomaly/Likely/High Finding with MITRE T1036

## Description

When `FlowDirection::insert_segment` returns `InsertResult::ConflictingOverlap` (same byte
range already buffered with DIFFERENT bytes), the reassembly engine emits one
`Anomaly/Likely/High` finding tagged with MITRE T1036 (Masquerading). The original bytes are
preserved (first-wins, INV-3). This is the primary forensic signal for TCP evasion attacks
such as segment-splicing and IDS bypass attempts.

## Preconditions

1. A TCP segment arrives for an established or partial flow.
2. The segment's byte range overlaps with already-buffered bytes for the same flow direction.
3. The overlapping bytes in the new segment DIFFER from the buffered bytes.
4. `self.findings.len() < MAX_FINDINGS` (10,000); otherwise the finding is dropped.

## Postconditions

1. `InsertResult::ConflictingOverlap` is returned from insert_segment.
2. A Finding is emitted with:
   - category: Anomaly
   - verdict: Likely
   - confidence: High
   - mitre_technique: Some("T1036")
   - summary: contains the FlowKey display string (lower_ip:lower_port -> upper_ip:upper_port)
   - direction: None (reassembly-engine findings do not set direction)
3. The original buffered bytes are NOT replaced; the conflicting new bytes are discarded.
4. The flow continues processing normally.

## Invariants

1. First-wins overlap policy (INV-3): original bytes always win. The ConflictingOverlap finding
   is informational -- it does NOT indicate that the new bytes were accepted.
2. This finding is NOT subject to the per-direction one-shot latch. Each ConflictingOverlap
   event produces one finding (subject to MAX_FINDINGS cap).
3. The overlap threshold alert (BC-2.04.019) is a SEPARATE mechanism: it fires when the
   cumulative count of overlapping segments (any kind) exceeds the threshold.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Conflicting overlap when findings.len() == MAX_FINDINGS | Finding is silently dropped; dropped_findings increments |
| EC-002 | Conflicting overlap immediately after another ConflictingOverlap | Second finding emitted (not latched) |
| EC-003 | Partial overlap: some bytes new, some conflicting | PartialOverlap result (not ConflictingOverlap); only new gap bytes added |
| EC-004 | Exact duplicate (same bytes, same range) | Duplicate result; no finding emitted |
| EC-005 | Conflicting overlap on a partially-assembled flow (ISN inferred) | Same behavior; ISN inference does not change overlap policy |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Segment with bytes "BBB" over buffered "AAA" at same offset | InsertResult::ConflictingOverlap; Finding(Anomaly/Likely/High, T1036); "AAA" preserved | happy-path |
| Same-range retransmit with identical bytes | InsertResult::Duplicate; no finding | edge-case |
| Conflicting overlap when finding cap full | No finding emitted; dropped_findings++ | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-002 | ConflictingOverlap always emits exactly one T1036 finding (when under cap) | unit: test_conflicting_overlap_finding |
| VP-002 | Original bytes are preserved after ConflictingOverlap | unit: assert buffered content unchanged |
| VP-002 | No finding when findings.len() >= MAX_FINDINGS | unit: fill to cap, trigger conflict |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per capabilities.md §CAP-04 -- conflicting overlap detection is the forensic core of TCP stream reassembly anomaly detection |
| L2 Domain Invariants | INV-3 (First-wins overlap policy), INV-6 (MAX_FINDINGS cap) |
| Architecture Module | SS-04 (reassembly/mod.rs:372-405, C-6; reassembly/lifecycle.rs:96-120, C-15; reassembly/segment.rs, C-8) |
| Stories | S-TBD |
| Origin BC | BC-RAS-018 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.037 -- depends on (insert_segment ConflictingOverlap result drives this)
- BC-2.04.019 -- related to (overlap threshold alert is a separate cumulative mechanism)
- BC-2.04.024 -- related to (MAX_FINDINGS cap affects whether finding is emitted)
- BC-2.10.007 -- related to (T1036 maps to DefenseEvasion tactic in MITRE lookup)

## Architecture Anchors

- `src/reassembly/mod.rs:372-405` -- InsertResult match block; ConflictingOverlap arm at line 379-382 calls generate_conflicting_overlap_finding
- `src/reassembly/lifecycle.rs:96-120` -- generate_conflicting_overlap_finding: emits Finding(Anomaly/Likely/High, T1036) subject to MAX_FINDINGS cap
- `src/reassembly/segment.rs` -- insert_segment returning ConflictingOverlap
- `tests/reassembly_engine_tests.rs` -- test_conflicting_overlap_finding

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:379-382` (ConflictingOverlap arm), `src/reassembly/lifecycle.rs:96-120` (finding emission) |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **assertion**: test_conflicting_overlap_finding asserts Finding category/verdict/confidence/mitre
- **guard clause**: InsertResult match arm at reassembly engine level

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (operates on in-memory state) |
| **Global state access** | mutates self (reassembler state) |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (takes &mut self) |
| **Overall classification** | mixed (stateful mutation) |
