---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-06-24T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-17
capability: CAP-17
lifecycle_status: active
introduced: v0.11.0-feature-enip
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
  - .factory/research/enip-mitre-ics-tagging.md
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/specs/verification-properties/vp-032-enip-parse-safety.md
input-hash: TBD
---

# BC-2.17.022: MAX_FINDINGS DoS Bound — Finding Cap Prevents Unbounded all_findings Growth

## Description

`EnipAnalyzer` enforces a hard upper bound (`MAX_FINDINGS = 10_000`) on the total number of
`Finding` objects accumulated in `self.all_findings`. When the cap is reached, no new findings
are pushed for any subsequent detection (T0846, T0858, T0816, T0836, T0888, T0814, ForwardOpen
anomaly). State counters (`write_count_in_window`, `error_counts_in_window`, `command_counts`,
`pdu_count`, `parse_errors`, `malformed_in_window`) continue to be updated even when findings
are capped. `self.dropped_findings` is incremented for each suppressed finding. This mirrors
the Modbus and DNP3 MAX_FINDINGS pattern.

## Preconditions

1. `EnipAnalyzer.all_findings.len() >= MAX_FINDINGS` at the time a detection rule would
   otherwise push a new `Finding`.

## Postconditions

1. No new `Finding` is pushed to `self.all_findings`.
2. `self.all_findings.len()` remains at `MAX_FINDINGS`.
3. `self.dropped_findings += 1`.
4. Per-flow state counters (write_count_in_window, malformed_in_window, error_counts_in_window,
   command_counts, pdu_count, parse_errors) ARE still updated regardless of the cap.
5. The `write_burst_emitted`, `error_rate_emitted`, and `malformed_anomaly_emitted` one-shot
   guards are NOT set when a finding is dropped due to the cap (allowing future windows to
   retry emission if the cap is not full — consistent with BC-2.15.022 EC-002 pattern).

## Invariants

1. **MAX_FINDINGS = 10_000**: shared constant matching Modbus and DNP3 (`MAX_FINDINGS` in
   `src/analyzer/enip.rs`). Same value as all other analyzers.
2. **Cap check is per-push**: `self.all_findings.len() < MAX_FINDINGS` is evaluated immediately
   before each `push`. Multi-finding events (rare in v0.11.0) push up to the cap.
3. **Counters always updated**: state tracking continues regardless of cap. This ensures
   `summarize()` (BC-2.17.021) reflects actual traffic volume.
4. **dropped_findings counter**: incremented for every suppressed push. Reported in
   `enip_summary.dropped_findings` to alert operators to high-volume flows.
5. **DoS bound**: `MAX_FINDINGS` is the primary memory defense. An adversary flooding port
   44818 with valid-looking ENIP frames cannot exhaust analyzer memory beyond
   `MAX_FINDINGS × sizeof(Finding)`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `all_findings.len() == MAX_FINDINGS - 1`; CIP Stop arrives | T0858 pushed; len == MAX_FINDINGS |
| EC-002 | `all_findings.len() == MAX_FINDINGS`; CIP Stop arrives | No push; `dropped_findings += 1`; `write_burst_emitted` NOT set |
| EC-003 | Cap reached mid-burst; write_burst_emitted NOT set | Next write burst in new window can fire T0836 (if cap has room by then — guard not set) |
| EC-004 | Cap reached at count 10000; 50 more ListIdentity frames | 50 T0846 findings suppressed; `dropped_findings = 50` |

## Canonical Test Vectors

| all_findings.len() | Event | Pushed? | dropped_findings delta |
|--------------------|----- |--------|------------------------|
| MAX_FINDINGS - 1 | CIP Reset | Yes (one more fits) | 0 |
| MAX_FINDINGS | CIP Reset | No | +1 |
| MAX_FINDINGS | ListIdentity × 5 | None of 5 pushed | +5 |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | Cap enforcement, dropped_findings counter: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — the MAX_FINDINGS bound is a required safety property: without it, an adversary flooding port 44818 with valid ENIP frames (ListIdentity, ForwardOpen, CIP Stop) could exhaust analyzer memory; the cap ensures O(1) bounded memory |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (MAX_FINDINGS constant) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — safety cap BC; no finding emission) |

## Related BCs

- BC-2.17.010 through BC-2.17.018 — all depend on (MAX_FINDINGS cap guard is a precondition for each detection BC)
- BC-2.17.021 — composes with (dropped_findings reported in summarize() output)

## Architecture Anchors

- `src/analyzer/enip.rs` — `const MAX_FINDINGS: usize = 10_000`
- `src/analyzer/enip.rs` — cap check: `if self.all_findings.len() < MAX_FINDINGS { self.all_findings.push(...); } else { self.dropped_findings += 1; }`
- `src/analyzer/enip.rs` — `EnipAnalyzer.dropped_findings: u64`
- BC-2.14.022 (Modbus precedent) and BC-2.15.022 (DNP3 precedent)

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — cap enforcement; no formal proof target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (MAX_FINDINGS constant); architecture-delta.md §4.2 (MAX_FINDINGS = 10,000); BC-2.14.022 + BC-2.15.022 (Modbus/DNP3 precedent) |
| **Confidence** | high — shared architectural safety requirement; mirrors proven Modbus and DNP3 patterns |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | reads all_findings.len(); conditionally pushes or increments dropped_findings |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (cap guard within on_data) |
