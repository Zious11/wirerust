---
document_type: behavioral-contract
level: L3
version: "1.1"
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
modified:
  - "v1.1: F8-001 — Invariant 5 added: process_pdu owns pdu_count but NOT command_counts (command_counts is incremented in BC-2.17.016 frame-walk PC-0, before is_valid_enip_frame); Postcondition 5 annotation added clarifying separation of pdu_count vs command_counts increment sites"
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

# BC-2.17.024: pdu_count Incremented Per Processed Frame and Reflected in summarize()

## Description

For each ENIP frame that passes `is_valid_enip_frame` and is fully processed by `process_pdu`,
`EnipFlowState.pdu_count` is incremented by 1. At flow close, this per-flow count is folded
into `EnipAnalyzer.total_pdu_count` (BC-2.17.017). The aggregate `total_pdu_count` is reported
in `enip_summary.total_pdu_count` by `summarize()` (BC-2.17.021). PDU count does NOT include
frames rejected by `is_valid_enip_frame` (those are `parse_errors`, not PDUs). This matches
the DNP3/Modbus pattern of counting only successfully processed protocol data units.

## Preconditions

1. `is_valid_enip_frame(&header)` returned `true` for the current frame.
2. `process_pdu()` is called for the frame.
3. `flow.is_non_enip == false`.

## Postconditions

1. `flow.pdu_count += 1` at the start of (or end of) each `process_pdu` call.
2. The increment occurs exactly once per successfully-processed frame.
3. Frames rejected by `is_valid_enip_frame` do NOT increment `pdu_count`.
4. Frames that do not result in findings (e.g., RegisterSession, IndicateStatus) still
   increment `pdu_count` (pdu_count tracks all processed frames, not just finding-generating frames).
5. **No-finding commands (v0.11.0)**: ListServices (0x0004), ListInterfaces (0x0064),
   IndicateStatus (0x0072), and Cancel (0x0075) are validity-gated (BC-2.17.003),
   classified (BC-2.17.004), and PDU-counted (this BC), but emit NO finding in v0.11.0.
   These commands have no MITRE ICS detection target in the current scope.
6. **process_pdu owns pdu_count; NOT command_counts (F8-001)**: `process_pdu` is responsible
   for incrementing `flow.pdu_count` (this BC). It does NOT increment `flow.command_counts`.
   The canonical `command_counts` increment site is the BC-2.17.016 frame-walk loop (PC-0 in
   `on_data`), which fires before `is_valid_enip_frame` and therefore counts ALL structurally-
   parsed headers including Unknown/invalid-command frames.

## Invariants

1. **PDU = processed frame**: `pdu_count` counts successfully decoded ENIP frames (passed
   validity gate). Malformed frames and carry-stash bytes are not PDUs.
2. **Monotonically increasing**: `pdu_count` never decreases within a flow's lifetime.
3. **Aggregate in total_pdu_count**: `EnipAnalyzer.total_pdu_count` is the sum of all
   closed flows' `pdu_count` values (folded by `on_flow_close`). Open flows at summarize
   time may optionally be included.
4. **No cap on pdu_count**: unlike `all_findings`, `pdu_count` and `total_pdu_count` are
   `u64` counters with no practical overflow. They count ALL processed frames.
5. **process_pdu owns pdu_count; NOT command_counts (F8-001)**: `process_pdu` is the sole
   increment site for `flow.pdu_count`. It does NOT increment `flow.command_counts`. The
   canonical `command_counts` increment site is the BC-2.17.016 frame-walk loop (PC-0 in
   `on_data`), which fires before `is_valid_enip_frame` and counts ALL structurally-parsed
   24-byte headers (valid + Unknown/invalid-command). pdu_count and command_counts have
   different coverage: `pdu_count` counts only validity-gated frames; `command_counts` counts
   all structurally-parsed headers.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Flow with 0 valid frames (all malformed) | `pdu_count = 0` |
| EC-002 | RegisterSession frame (valid, no finding emitted) | `pdu_count += 1` |
| EC-003 | CIP Stop frame (valid, T0858 finding emitted) | `pdu_count += 1` |
| EC-004 | Frame rejected by is_valid_enip_frame | `parse_errors += 1`; `pdu_count` NOT incremented |
| EC-005 | is_non_enip=true (carry overflow) | No PDUs processed; pdu_count unchanged |

## Canonical Test Vectors

| Sequence | pdu_count after | parse_errors after |
|----------|-----------------|-------------------|
| 3 valid frames | 3 | 0 |
| 3 valid + 2 invalid | 3 | 2 |
| is_non_enip triggered on frame 4 | 3 | 1 (overflow) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | PDU count increment semantics: effectful shell; unit test | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 |
| Capability Anchor Justification | CAP-17 ("EtherNet/IP + CIP Analysis") per ARCH-INDEX.md §SS-17 — PDU count is required for the ENIP analyzer summary statistics: operators need to know how many ENIP frames were successfully processed to assess analysis coverage; without this count, a high parse_errors / low pdu_count ratio indicating a mis-classified flow is invisible |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence) |
| Architecture Module | SS-17 (analyzer/enip.rs); ADR-010 Decision 4 (pdu_count field) |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-enip-v0.11.0 (issue #316) |
| MITRE Techniques | (none — statistics BC; no finding emission) |

## Related BCs

- BC-2.17.003 — depends on (is_valid_enip_frame true is the precondition for pdu_count increment)
- BC-2.17.016 — composes with (frame-walk PC-0 is canonical command_counts site; distinct from pdu_count which is owned by process_pdu; F8-001)
- BC-2.17.017 — depends on (pdu_count folded into total_pdu_count on flow close)
- BC-2.17.021 — composes with (total_pdu_count reported in enip_summary)

## Architecture Anchors

- `src/analyzer/enip.rs` — `EnipFlowState.pdu_count: u64`
- `src/analyzer/enip.rs` — `EnipAnalyzer.total_pdu_count: u64`
- `src/analyzer/enip.rs` — `process_pdu`: `flow.pdu_count += 1;` at start of function

## Story Anchor

(TBD — assigned during F3 story decomposition)

## VP Anchors

(none — statistics counter; unit test)

## Source Evidence

| Property | Value |
|----------|-------|
| **Path** | ADR-010 Decision 4 (pdu_count field in EnipFlowState); architecture-delta.md §4.1 (pdu_count in struct) |
| **Confidence** | high — mirrors DNP3/Modbus frame_count pattern |
| **Extraction Date** | 2026-06-24 |

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | mutates flow.pdu_count |
| **Deterministic** | yes |
| **Thread safety** | single-threaded |
| **Overall classification** | effectful shell (counter within process_pdu) |
