---
document_type: behavioral-contract
level: L3
version: "1.2"
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
modified: []
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

# BC-2.15.022: MAX_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth

## Description

`Dnp3Analyzer` enforces a hard upper bound (`MAX_FINDINGS`) on the total number of `Finding`
objects accumulated in `self.all_findings`. When the cap is reached, no new findings are pushed
for any subsequent detection (T1692.001, T0814, T0836, T1691.001, T0827, broadcast anomaly,
unsolicited anomaly). State counters (`direct_operate_count`, `restart_event_count`, etc.)
continue to be updated even when findings are capped, so correlation-window logic (T0827) remains
accurate if the cap is relaxed in a future configuration. This mirrors the Modbus MAX_FINDINGS
pattern from BC-2.14.022.

## Preconditions

1. `Dnp3Analyzer.all_findings.len() >= MAX_FINDINGS` at the time a detection rule would otherwise push a new `Finding`.

## Postconditions

1. No new `Finding` is pushed to `self.all_findings`.
2. `self.all_findings.len()` remains at `MAX_FINDINGS` (does not grow beyond the cap).
3. Per-flow counters (`direct_operate_count`, `restart_event_count`, `block_event_count`,
   `fc_counts`, `fn_code_counts`, `frame_count`) ARE still updated — they track protocol
   activity regardless of the cap.
4. The `direct_operate_emitted`, `loss_of_control_emitted`, and `unsolicited_anomaly_emitted`
   one-shot guards are NOT set when a finding is dropped due to the cap (the guard prevents
   duplicate findings, but if the first firing was dropped, a future window should be able to
   emit when the flow progresses — this is an acceptable edge case; see EC-002 below).

## Invariants

1. **MAX_FINDINGS is a shared constant**: `MAX_FINDINGS` is the same constant used by all
   analyzers (Modbus, HTTP, TLS). It is not DNP3-specific. [ADR-007 Decision 2; mirrors BC-2.14.022]
2. **Cap check is per-push**: `self.all_findings.len() < MAX_FINDINGS` is evaluated immediately
   before each `push` call, not once at frame entry. This allows a multi-finding frame to push
   partial findings up to the cap (most-specific first, per BC-2.15.013).
3. **Counters always updated**: protocol statistics (FC counts, frame counts, etc.) are updated
   regardless of the cap. This ensures `summarize()` (BC-2.15.020) reflects actual traffic volume
   even when findings are suppressed.
4. **DoS bound**: `MAX_FINDINGS` is the primary defense against an adversary flooding the DNP3
   port with valid-looking frames to exhaust analyzer memory. The cap ensures `O(1)` memory
   regardless of traffic volume.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `all_findings.len() == MAX_FINDINGS`; COLD_RESTART arrives | No T0814 finding pushed; `restart_event_count` still incremented |
| EC-002 | Cap reached on first T1692.001 in a window; `direct_operate_emitted` NOT set; next window arrives | When cap is not reached in next window, T1692.001 can fire again (guard was not set because first fire was dropped) |
| EC-003 | Cap reached mid-multi-finding sequence (T0814 pushed, T0827 not) | T0814 in vec (most-specific first per BC-2.15.013); T0827 dropped |
| EC-004 | Zero findings cap (`MAX_FINDINGS = 0`) | No findings ever pushed; counters still updated; theoretical edge case (not a supported config) |

## Canonical Test Vectors

| State | Event | Expected |
|-------|-------|---------|
| `all_findings.len() == MAX_FINDINGS - 1` | COLD_RESTART | T0814 pushed (cap allows); `len == MAX_FINDINGS` |
| `all_findings.len() == MAX_FINDINGS` | COLD_RESTART | No push; `restart_event_count++` |
| `all_findings.len() == MAX_FINDINGS - 1` | COLD_RESTART (Nth, T0827 threshold crossed) | T0814 pushed (len→MAX); T0827 skipped (cap) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Cap enforcement: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — the MAX_FINDINGS bound is a safety requirement of the DNP3/ICS analyzer capability; without it, an adversary could exhaust analyzer memory by sending a large number of valid-looking DNP3 control commands on port 20000 |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — DoS protection applies to all flows routed to the DNP3 analyzer) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 2 |
| Stories | TBD (F3 decomposition) |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — safety/cap BC; no finding emission) |

## Related BCs

- BC-2.15.010 through BC-2.15.015 — all depend on (MAX_FINDINGS cap guard is a precondition for each detection BC)
- BC-2.15.013 — composes with (co-emission ordering ensures most-specific finding is pushed first before cap is reached)

## Architecture Anchors

- `src/analyzer/dnp3.rs` — `const MAX_FINDINGS: usize` (shared constant)
- `src/analyzer/dnp3.rs` — cap check: `if self.all_findings.len() < MAX_FINDINGS { self.all_findings.push(...); }`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §8` — "MAX_FINDINGS / DoS-bound"
- BC-2.14.022 (Modbus: same pattern)

## Story Anchor

TBD (F3 story decomposition)

## VP Anchors

(none — cap enforcement; no formal proof target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | dnp3-architecture-delta.md §8 ("MAX_FINDINGS / DoS-bound; co-emission ordering & cap (mirror Modbus most-specific rule)"); BC-2.14.022 (Modbus precedent) |
| **Confidence** | high — architectural safety requirement; mirrors established Modbus pattern |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads all_findings.len(); conditionally pushes to all_findings |
| **Deterministic** | yes — same frame sequence produces same cap behavior |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (cap guard within on_data) |
