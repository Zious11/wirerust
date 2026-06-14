---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-06-10T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-15
capability: CAP-15
lifecycle_status: active
introduced: v0.6.0-feature-008
modified:
  - "v1.1: Pass-1 adversarial fix I-2: retitled H1 from 'Most-Specific-Rule Co-Emission Ordering — T0814 and T1692.001 When Both Conditions Present' (misleading — T0814 and T1692.001 cannot co-occur on one FC) to 'Co-Emission Ordering — Direct Finding (T0814/T1692.001) Precedes Derived T0827' (accurate — real co-emission is direct T0814/T1692.001 → derived T0827). Added I-4 dedup rule for broadcast-anomaly (018↔010 T1692.001 double-count). — 2026-06-10"
  - "v1.3: F3 story-anchor back-fill. — 2026-06-14"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/dnp3-architecture-delta.md
  - .factory/research/dnp3-research.md
  - .factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md
input-hash: TBD
---

# BC-2.15.013: Co-Emission Ordering — Direct Finding (T0814/T1692.001) Precedes Derived T0827

## Description

When a single DNP3 frame triggers multiple detection rules simultaneously (e.g., a Control-class
FC to a broadcast destination, or a Control-class FC burst that also happens to be the N-th
restart in a T0827 accumulation window), findings are emitted in most-specific-rule order:
the most specific / highest-severity detection fires first. This BC specifies the ordering
and cap policy when multiple findings would be emitted from the same frame. It mirrors the
Modbus most-specific rule established in BC-2.14.013. ADR-007 Decision 5.

## Preconditions

1. A single FIR=1 DNP3 application frame triggers two or more distinct detection rules
   simultaneously in the same `on_data` call.
2. `self.all_findings.len() < MAX_FINDINGS` at entry to the `on_data` call (cap checked
   before the first finding is pushed).

## Postconditions

**Emission ordering (most-specific-first, within a single on_data call):**
1. T1692.001 (unauthorized control command, BC-2.15.010) is emitted BEFORE T0836 (WRITE, BC-2.15.012)
   if both fire. (Both require distinct FC classes so this cannot occur in a single frame —
   included for completeness.)
2. T0814 (restart, BC-2.15.011) is emitted BEFORE T0827 (derived impact, BC-2.15.015)
   because T0814 is the direct observation and T0827 is the accumulated consequence.
3. T1692.001 (per-frame threshold finding) is emitted BEFORE T0827 (derived impact) for the
   same reason as above.

**Cap policy:**
4. When `self.all_findings.len() == MAX_FINDINGS` after the first finding is pushed (from a
   multi-finding frame), subsequent findings from the same frame are SILENTLY DROPPED. The
   first (most specific) finding is always preserved.
5. `MAX_FINDINGS` is checked before EACH push, not just at frame entry. If the cap is hit
   mid-sequence, the remaining sequence items are skipped.

## Invariants

1. **Most-specific-first ordering**: more specific detection (per-packet direct observation)
   precedes derived/correlated findings (T0827) in the `all_findings` vec.
2. **Cap applied per-push**: the `MAX_FINDINGS` check is `self.all_findings.len() < MAX_FINDINGS`
   evaluated immediately before each `push`. This prevents a burst of multi-finding frames
   from over-filling the vec.
3. **No duplicate technique tags in a single finding**: each `Finding` has one unique
   `mitre_techniques` vec. The co-emission model (ADR-007 Decision 5) for DNP3 does NOT
   merge findings from different BCs into a single Finding object — each rule produces its
   own Finding (unlike the Modbus v2.0 union-tag model). This is because DNP3 detection rules
   fire on distinct FC classes that cannot co-occur on the same application FC.
4. **Broadcast-anomaly (BC-2.15.018) + burst-threshold (BC-2.15.010) dedup rule**: a
   broadcast Control FC BOTH emits a BC-2.15.018 anomaly finding (T1692.001,
   `Suspicious/Possible/Medium`) AND increments `direct_operate_count` (which can later
   trigger BC-2.15.010's burst finding, also T1692.001). Both findings are RETAINED — they
   represent distinct observations (broadcast destination anomaly vs. sustained burst
   pattern) and appear under the same technique ID. The BC-2.15.018 finding is emitted
   first (at the point of observation, before the count threshold check in the same
   `on_data` call). This means a single broadcast Control FC can contribute at most TWO
   T1692.001 findings to `all_findings` in a session (one from BC-2.15.018, one from
   BC-2.15.010 when the burst threshold is later crossed). Implementers must not deduplicate
   based on technique ID alone; findings are distinct by their category/verdict/summary.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A COLD_RESTART frame that is also the Nth event pushing past T0827 threshold | Two findings emitted in order: T0814 first, T0827 second. If cap reached after T0814, T0827 dropped. |
| EC-002 | `all_findings.len() == MAX_FINDINGS - 1` and two findings would be emitted | First finding (T0814) is pushed (len → MAX); second finding (T0827) is skipped (cap). |
| EC-003 | `all_findings.len() == MAX_FINDINGS` at frame entry | Neither finding emitted; both are skipped. Counters still updated. |
| EC-004 | A Control-class FC in the same window as a WRITE (impossible in one frame) | WRITE maps to `Write` class, Control to `Control` class; they cannot co-occur in one App FC byte; this edge case cannot arise |
| EC-005 | Broadcast Control FC (dest=0xFFFF) when `direct_operate_count` later crosses burst threshold | Two distinct T1692.001 findings in session: [1] BC-2.15.018 broadcast-anomaly finding (Suspicious/Possible/Medium) emitted on the frame; [2] BC-2.15.010 burst finding (Execution/Likely/Medium) emitted when threshold crossed. Both are retained — they are distinct observations. |

## Canonical Test Vectors

| Scenario | Expected findings (in order) |
|----------|------------------------------|
| COLD_RESTART (Nth) → T0827 threshold crossed | [1] T0814 finding, [2] T0827 finding |
| COLD_RESTART (Nth), cap at MAX_FINDINGS−1 | [1] T0814 finding (cap consumed); T0827 dropped |
| Control burst (Nth) → T0827 threshold crossed | [1] T1692.001 finding, [2] T0827 finding |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Ordering and cap policy: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — the co-emission ordering and cap policy ensures that the most operationally meaningful findings are preserved when the MAX_FINDINGS cap is reached during sustained adversarial traffic against DNP3 outstations |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — findings ordering mirrors the most-specific rule pattern established across all analyzers) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24 `on_data`); ADR-007 Decision 5 |
| Stories | STORY-108 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (policy BC — governs emission ordering; techniques are emitted by BC-2.15.010, 011, 012, 015) |

## Related BCs

- BC-2.15.010 — composes with (T1692.001 emitted first in ordering; burst finding retained alongside broadcast finding)
- BC-2.15.011 — composes with (T0814 emitted before T0827)
- BC-2.15.015 — composes with (T0827 emitted last, as derived consequence)
- BC-2.15.018 — composes with (broadcast-anomaly T1692.001 finding retained alongside BC-2.15.010 burst finding; both are distinct per Invariant 4)
- BC-2.15.022 — depends on (MAX_FINDINGS cap defines the limit this BC manages)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `Dnp3Analyzer::on_data` — multi-finding emission sequence within a single call
- `src/analyzer/dnp3.rs` — `MAX_FINDINGS` constant (shared with other analyzers)
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` — "co-emission ordering & cap (mirror Modbus most-specific rule)"

## Story Anchor

STORY-108

## VP Anchors

(none — ordering policy; effectful shell logic)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 5; dnp3-architecture-delta.md §8 ("co-emission ordering & cap mirror Modbus most-specific rule"); BC-2.14.013 (Modbus precedent) |
| **Confidence** | high — architectural policy decision |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads/writes all_findings |
| **Deterministic** | yes — same frame sequence produces same ordering |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (ordering policy within on_data) |
