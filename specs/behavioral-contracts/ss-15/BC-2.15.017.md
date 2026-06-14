---
document_type: behavioral-contract
level: L3
version: "1.4"
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
  - "v1.3: F3 story-anchor back-fill; renamed DNPXX_→DNP3_ (erroneous; REVERTED in v1.4 — DNPXX_ is the canonical shipped name per src/analyzer/dnp3.rs:169). — 2026-06-14"
  - "v1.4: REVERT erroneous Pass-22 rename — restore canonical shipped constant name DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT (matches src/analyzer/dnp3.rs:169 + STORY-110); prior v1.3 'DNP3_' rename was a spec<->code mis-anchor (F3-convergence Pass-24 FIX-1). — 2026-06-14"
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

# BC-2.15.017: --dnp3-direct-operate-threshold CLI Flag Controls Control-Command Detection Window

## Description

The `--dnp3-direct-operate-threshold` CLI flag (type: `u32`) sets the per-flow detection
window for Control-class FC bursts (BC-2.15.010). When `direct_operate_count` exceeds this
value within the detection window, one T1692.001 finding is emitted. The flag mirrors
`--modbus-write-burst-threshold` in structure and CLI position. The default value is proposed
as **10** (human to confirm at F2-GATE). The flag is optional; if omitted, the compiled
default applies.

**[F2-GATE: human to confirm default]**
Proposed default: `--dnp3-direct-operate-threshold 10`. See BC-2.15.010 Invariants for
the rationale. The Modbus write-burst default was 20/1s (short 1-second window, fast attack).
DNP3 control commands are operationally slower (SBO round-trips can take seconds; relay dwell
times are device-specific). A value of 10 within a 60-second window captures commissioning-speed
attacks while tolerating legitimate maintenance. If the target environment has very low control
activity, a value as low as 3–5 may be more appropriate.

## Preconditions

1. The user invokes `wirerust analyze` with `--dnp3-direct-operate-threshold <N>` where N is a
   valid `u32` (0..=4_294_967_295).
2. OR the user omits the flag, in which case the compiled default (`DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`) is used.

## Postconditions

1. `Dnp3Analyzer.direct_operate_threshold` is set to the parsed `u32` value (or default).
2. BC-2.15.010's threshold check uses exactly this value: `flow.direct_operate_count > self.direct_operate_threshold`.
3. Setting `--dnp3-direct-operate-threshold 0` means any single Control-class FC immediately
   triggers T1692.001 (count=1 > 0 is true on the very first frame).
4. Setting `--dnp3-direct-operate-threshold 4294967295` (u32::MAX) means T1692.001 never fires
   (the counter can never exceed u32::MAX without overflow, which is impossible for a single flow's
   frame count).
5. The flag value is echoed in the T1692.001 finding summary string (per BC-2.15.010 postcondition 3).

## Invariants

1. **Flag mirrors Modbus pattern**: `--dnp3-direct-operate-threshold` follows the same CLI
   structure as `--modbus-write-burst-threshold` per ADR-007 Decision 6.
2. **Type: u32**: the threshold is a `u32`, matching `direct_operate_count: u32` in `Dnp3FlowState`.
   Overflow is impossible: the counter is bounded by the number of frames processed, which is
   bounded by the pcap file size.
3. **No validation of semantic reasonableness**: the CLI accepts any `u32` value including 0
   (hypersensitive) and u32::MAX (disabled). Documentation describes recommended ranges.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `--dnp3-direct-operate-threshold 0` | Every Control-class FC emits T1692.001 (count=1 > 0 on first frame) |
| EC-002 | `--dnp3-direct-operate-threshold 1` | Second Control-class FC emits T1692.001 |
| EC-003 | Flag omitted | Default value applies (proposed: 10) |
| EC-004 | `--dnp3-direct-operate-threshold 4294967295` | T1692.001 never fires (counter can't exceed u32::MAX) |

## Canonical Test Vectors

| Flag value | Control FCs received | Expected outcome |
|-----------|---------------------|-----------------|
| 10 (default) | 10 FCs | No finding (10 > 10 is false) |
| 10 (default) | 11 FCs | T1692.001 finding emitted |
| 0 | 1 FC | T1692.001 finding emitted immediately |
| 5 | 6 FCs | T1692.001 finding emitted |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | CLI flag parsing: standard Clap integration; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 |
| Capability Anchor Justification | CAP-15 ("DNP3/ICS Analysis") per ARCH-INDEX.md §SS-15 — the threshold CLI flag makes the DNP3/ICS analyzer tunable to the sensitivity requirements of specific OT environments, a key usability requirement for a threat-detection tool in industrial deployments where control-command rates vary widely by segment type |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence — flag controls detection sensitivity; does not affect frame classification) |
| Architecture Module | SS-15 (analyzer/dnp3.rs, C-24); ADR-007 Decision 6; SS-12 (CLI, cli.rs) |
| Stories | STORY-110 |
| Feature | issue-008-dnp3-analyzer |
| MITRE Techniques | (none — CLI configuration BC; detection is in BC-2.15.010) |

## Related BCs

- BC-2.15.010 — depends on (this flag directly controls BC-2.15.010's threshold check)

## Architecture Anchors

- `src/cli.rs` — `Commands::Analyze.dnp3_direct_operate_threshold: u32` field with `#[arg(long, default_value_t = DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT)]`
- `src/analyzer/dnp3.rs` — `Dnp3Analyzer.direct_operate_threshold: u32`
- `.factory/phase-f2-spec-evolution/dnp3-architecture-delta.md §10` — CLI delta; `DNPXX_DIRECT_OPERATE_THRESHOLD_DEFAULT`
- `.factory/specs/architecture/decisions/ADR-007-binary-ics-protocol-integration-dnp3-tcp.md §Decision 6`

## Story Anchor

STORY-110

## VP Anchors

(none — CLI integration; no formal proof target)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-007 Decision 6; dnp3-architecture-delta.md §10 (CLI delta); dnp3-research.md §5.1 (threshold rationale [JUDGMENT]) |
| **Confidence** | high (flag structure) / medium (default value pending F2-GATE confirmation) |
| **Extraction Date** | 2026-06-10 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none (flag parsed at startup; stored in struct) |
| **Global state access** | reads `self.direct_operate_threshold` in `on_data` |
| **Deterministic** | yes — same threshold + same frames = same findings |
| **Thread safety** | single-threaded |
| **Overall classification** | configuration / effectful shell consumer |
