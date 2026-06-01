---
document_type: behavioral-contract
level: L3
version: "1.6"
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
  - "v1.3: Pass-4 sibling sweep: corrected if-count-guard citation 571 → 572 (off-by-one against current source; line 571 is the let count = ... binding, line 572 is the if count > 0 { guard) — 2026-05-27"
  - "v1.4: W11-D1 fix — replaced bare `—` VP placeholder with explicit N/A marker. No VP in VP-INDEX covers the zero-count suppression path; this is a unit-test-sufficient boundary condition on the same guard as BC-2.04.025. — 2026-05-28"
  - "v1.5: F-DRIFT2A-001 — fixed stale domain/capabilities/cap-04-tcp-reassembly.md citation to domain/capabilities/cap-04-tcp-reassembly.md in L2 Capability and Capability Anchor Justification rows. — 2026-05-29"
  - "v1.6: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:572 → mod.rs:629 (`if count > 0` guard in finalize). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.026: finalize Does NOT Emit Segment-Limit Finding When Counter is Zero

## Description

This BC is the inverse guard of BC-2.04.025. When `finalize` runs and
`stats.segments_segment_limit == 0`, no segment-limit summary finding is emitted. The `if
count > 0` guard in the finalize segment-limit block ensures the Finding is only present when
at least one segment was actually dropped. This prevents spurious anomaly findings in clean
captures where no segment limit was hit.

## Preconditions

1. `finalize` is called.
2. `stats.segments_segment_limit == 0` (no segments were ever dropped due to the limit).

## Postconditions

1. No segment-limit summary Finding is added to `self.findings` during finalize.
2. `findings` after finalize contains only findings generated during packet processing (from
   flow closures and anomaly thresholds).
3. `finalized == true`.

## Invariants

1. The guard `if count > 0` is evaluated after `self.finalized = true` and after all flow
   closures. It is the last conditional in finalize.
2. A clean capture (well-behaved TCP, no evasion) produces zero reassembly findings: no
   overlap, small-segment, out-of-window, truncated, or segment-limit findings.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Normal PCAP with well-behaved TCP streams | segments_segment_limit=0; no summary finding |
| EC-002 | segments_segment_limit was 0 but then incremented during finalize | Impossible -- segments_segment_limit is only incremented in process_packet, not finalize |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Clean PCAP; finalize called | findings.is_empty() (or contains only flow-generated findings); no segment-limit finding | happy-path |
| segments_segment_limit=0; finalize | No segment-limit finding in findings | happy-path |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| N/A (no formal VP — zero-count suppression is a unit-test-sufficient boundary condition; no proptest/Kani harness exists for this guard) | No segment-limit finding when limit==0 | unit: finalize with zero limit; assert no such finding |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP Stream Reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP Stream Reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- suppressing spurious findings on clean captures is part of the forensic correctness contract |
| L2 Domain Invariants | INV-7 (finalize-once latch) |
| Architecture Module | SS-04 (reassembly/mod.rs:629, `if count > 0` guard) |
| Stories | STORY-021 |
| Origin BC | BC-RAS-026 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.025 -- inverse (this BC is the false-branch of the same guard)
- BC-2.04.012 -- composes with (finalize lifecycle)

## Architecture Anchors

- `src/reassembly/mod.rs:629` -- `if count > 0 { self.findings.push(...) }`

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:629` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-20 |

## Evidence Types Used

- **guard clause**: `if count > 0` before the push in finalize

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads self.stats; no mutation in the zero-count path |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe |
| **Overall classification** | pure (zero-count path) |

## Refactoring Notes

No refactoring needed.
