---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.3: DF-SIBLING-SWEEP-001 HS-043 re-anchor: mod.rs:557-591 → mod.rs:614-648 (finalize fn); mod.rs:558-561 → mod.rs:615-618 (finalized latch); mod.rs:573 → mod.rs:630 (unconditional push). — 2026-06-01"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.04.054: finalize Unconditionally Bypasses MAX_FINDINGS Cap for Segment-Limit Finding

## Description

`TcpReassembler::finalize()` pushes the segment-limit summary finding UNCONDITIONALLY, without
checking `self.findings.len() >= MAX_FINDINGS`. This is the single intentional bypass of the
MAX_FINDINGS cap (INV-6). As a result, after finalize, `self.findings.len()` may equal
`MAX_FINDINGS + 1` (10,001). The bypass ensures this critical lifecycle summary is never lost
even during adversarial flooding scenarios.

## Preconditions

1. finalize() is called on the reassembler.
2. `self.finalized` is false (first call; subsequent calls are no-ops via INV-7 latch).
3. `stats.segments_segment_limit > 0` (the segment-limit finding is only pushed when this > 0;
   see BC-2.04.025).

## Postconditions

1. If `segments_segment_limit > 0`: one Finding is pushed to `self.findings` unconditionally,
   even if `findings.len()` was already at or beyond MAX_FINDINGS.
2. After this push, `self.findings.len()` may equal MAX_FINDINGS + 1 = 10,001.
3. The finding has category Anomaly, verdict Inconclusive, confidence Medium, no MITRE technique.
4. `self.finalized` is set to true.

## Invariants

1. This is the ONLY code path that bypasses the MAX_FINDINGS guard.
2. finalize() is idempotent: the latch at mod.rs:615-618 ensures subsequent calls are no-ops.
3. The maximum possible `findings.len()` after any run is MAX_FINDINGS + 1.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | findings.len() == MAX_FINDINGS when finalize called with segments_segment_limit > 0 | findings.len() becomes MAX_FINDINGS + 1 = 10,001 |
| EC-002 | findings.len() < MAX_FINDINGS when finalize called | Normal push; no bypass semantics triggered |
| EC-003 | segments_segment_limit == 0 when finalize called | No finding pushed; no bypass matters |
| EC-004 | finalize called twice | Second call is a no-op (latch); no additional finding |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| finalize() with segments_segment_limit=1, findings at MAX_FINDINGS | findings.len()==10001 | edge-case |
| finalize() with segments_segment_limit=0 | findings.len() unchanged | edge-case |
| finalize() called twice | No second finding; idempotent | edge-case |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-003 | findings.len() <= MAX_FINDINGS + 1 after finalize (not MAX_FINDINGS + 2) | unit |
| VP-003 | finalize is idempotent (second call produces no additional findings) | unit: test_finalize_flushes_remaining |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md |
| Capability Anchor Justification | CAP-04 ("TCP stream reassembly") per domain/capabilities/cap-04-tcp-reassembly.md -- the finalize bypass is the lifecycle closure contract for the reassembly engine |
| L2 Domain Invariants | INV-6 (MAX_FINDINGS cap -- this BC documents the sole exception), INV-7 (Finalize-once latch) |
| Architecture Module | SS-04 (reassembly/mod.rs:614-648, C-6) |
| Stories | STORY-021 |
| Origin BC | BC-RAS-054 (pass-3 ingestion corpus, HIGH confidence) |

## Related BCs

- BC-2.04.024 -- supersedes for finalize path (bypass applies here)
- BC-2.04.025 -- composes with (segment-limit finding is the content of the bypass push)
- BC-2.04.012 -- depends on (finalize idempotence is part of this contract)

## Architecture Anchors

- `src/reassembly/mod.rs:630` -- unconditional push of finalize segment-limit finding
- `src/reassembly/mod.rs:615-618` -- finalized latch

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | `src/reassembly/mod.rs:630` |
| **Confidence** | high |
| **Extraction Date** | 2026-05-19 |

## Evidence Types Used

- **guard clause**: absence of MAX_FINDINGS check at push site 630 (all other 5 sites have it)
- **assertion**: test_finalize_generates_segment_limit_finding

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates self.findings, self.finalized |
| **Deterministic** | yes |
| **Thread safety** | not thread-safe (&mut self) |
| **Overall classification** | mixed (stateful mutation) |
