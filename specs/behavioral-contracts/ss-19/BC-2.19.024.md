---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-07-13T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-19
capability: CAP-19
lifecycle_status: active
introduced: feature-iec104
modified:
  - "v1.1: F-P2-M1 — Description corrected: T1692.001 is 'Unauthorized Message: Command Message' not 'Exploitation of Remote Services'. F-P2-H1 — VP-044 over-scope: gap computation is not parse_apci_header; VP-044 row and anchor removed; re-anchored to VP-047. 2026-07-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "f5a97d3"
---

# BC-2.19.024: N(S) Gap > k=12 Emits T1692.001 Sequence-Desync Finding

## Description

The IEC-104 standard defines a maximum unacknowledged window of k=12 I-frames
(configurable; default k=12 per IEC 60870-5-104 §5.5). When the passive analyzer
observes an N(S) value that is more than k=12 ahead of the last acknowledged N(S)
(i.e., `(current_ns - last_ns) mod 32768 > 12`), it emits a T1692.001
"Unauthorized Message: Command Message" finding with confidence Possible. A large N(S) gap
indicates either packet loss (benign but anomalous), a replay injection (adversarial),
or a desynchronized sequence counter (implementation bug). This is the window-overflow
detection path described in ADR-013 Decision 6.

## Preconditions

1. An I-format APCI frame has been parsed and N(S) extracted (per BC-2.19.023).
2. `Iec104FlowState::last_ns_c2s` (C2S direction) or `last_ns_s2c` (S2C direction) contains the N(S) from the previous I-frame on this flow in the matching direction.
3. `(current_ns.wrapping_sub(last_ns_dir) & 0x7FFF) > 12` (15-bit modular gap exceeds k=12).

## Postconditions

1. T1692.001 "Unauthorized Message: Command Message" finding emitted (confidence Possible).
2. Finding message includes: current N(S), last N(S), and the gap value.
3. The directional field (`last_ns_c2s` or `last_ns_s2c`) is updated to current N(S).

## Invariants

1. **15-bit modular arithmetic**: gap calculation must apply `& 0x7FFF` after `wrapping_sub` to correctly mask to the 15-bit N(S) range — `u16::wrapping_sub` wraps at 2^16 (65536), not 2^15 (32768). Without the mask, values in the upper 15-bit range produce false-positive gaps.
2. **k=12 constant**: the k-value is fixed at 12 for MVP. Future enhancement: configurable via `--iec104-k-window`.
3. **First-frame initial state**: `last_ns_c2s` and `last_ns_s2c` are initialized to `0` (u16 default). The first I-frame has gap = `(current_ns.wrapping_sub(0)) & 0x7FFF` = `current_ns & 0x7FFF`. Since IEC-104 N(S) starts at 0, the first frame gap is 0 — no spurious finding.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First I-frame on new flow (N(S)=0, last_ns=0) | Gap = 0; no finding; last_ns_dir updated to 0 |
| EC-002 | Gap = 12 (exactly k) | No finding (≤ k is allowed) |
| EC-003 | Gap = 13 (k+1) | T1692.001 Possible |
| EC-004 | Sequence wrap: last_ns=32767, current_ns=1 | `(1u16.wrapping_sub(32767) & 0x7FFF)` = `(32770 & 0x7FFF)` = 2; no finding |
| EC-005 | Gap = 32767 (massive jump / replay) | T1692.001 Possible |

## Canonical Test Vectors

| last_ns | current_ns | Gap (15-bit) | Expected |
|---------|------------|--------------|----------|
| 0 | 12 | 12 | No finding |
| 0 | 13 | 13 | T1692.001 Possible |
| 32767 | 1 | 2 | No finding |
| 100 | 114 | 14 | T1692.001 Possible |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in gap detection; wrapping arithmetic produces no overflow for any u16 last_ns/current_ns pair | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — N(S) sequence-desync detection is a key ICS network anomaly indicator in the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 6 |
| Feature | feature-iec104 |
| MITRE Techniques | T1692.001 "Unauthorized Message: Command Message" — Possible (sequence desync) |

## Related BCs

- BC-2.19.023 — depends on (N(S) extraction)
- BC-2.19.019 — composes with (T1692.001 also emitted on control TypeIDs)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `let gap = current_ns.wrapping_sub(last_ns_dir) & 0x7FFF; if gap > 12 { emit T1692_001(Possible); }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 6`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (no-panic for all gap computation paths)
