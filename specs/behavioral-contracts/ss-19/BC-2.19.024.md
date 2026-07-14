---
document_type: behavioral-contract
level: L3
version: "1.3"
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
  - "v1.2: D-438 follow-on (human-mandated) — first-frame N(S) guard: last_ns_c2s/last_ns_s2c promoted from u16 to Option<u16> (SS-19 shard v1.6, ADR-013 Decision 6). None sentinel replaces the false u16-default-0 assumption. First observed I-frame sets Some(ns) baseline and emits NO finding (handles mid-capture start where first N(S) is arbitrary). Gap check runs only on Some(prev) state. Preconditions, postconditions, Invariant 3, edge cases, and test vectors updated throughout. 2026-07-14"
  - "v1.3: F2 first-frame-guard-review LOW-1/LOW-2: Description now states the None-baseline first-frame exception; VP-047 property text updated to Option<u16> last_ns state. Prose-only; no arithmetic/postcondition change. 2026-07-14"
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
input-hash: "a153144"
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
detection path described in ADR-013 Decision 6. On the first observed I-frame per direction
(state `None`), the baseline `Some(ns)` is recorded and no finding is emitted
(mid-capture correctness).

## Preconditions

1. An I-format APCI frame has been parsed and N(S) extracted (per BC-2.19.023).
2. `Iec104FlowState::last_ns_c2s` (C2S direction) or `last_ns_s2c` (S2C direction) is `Option<u16>`, initialized to `None` when flow state is first created. At call time the field is either:
   - `None` — no I-frame has been observed yet in this direction (fresh flow or mid-capture start), OR
   - `Some(prev)` — a prior I-frame baseline has been established.
3. [Gap-check path only — requires `Some(prev)` state] `(current_ns.wrapping_sub(prev) & 0x7FFF) > 12` (15-bit modular gap exceeds k=12).

## Postconditions

**Path A — First-frame (state `None`):**
1. The directional field (`last_ns_c2s` or `last_ns_s2c`) is set to `Some(current_ns)`.
2. NO finding is emitted. Baseline is established; the first observed N(S) is arbitrary on mid-capture starts and must never generate a desync finding.

**Path B — Subsequent-frame, gap ≤ k=12 (state `Some(prev)`, gap ≤ 12):**
1. The directional field is updated to `Some(current_ns)`.
2. No finding emitted.

**Path C — Subsequent-frame, gap > k=12 (state `Some(prev)`, `(current_ns.wrapping_sub(prev) & 0x7FFF) > 12`):**
1. T1692.001 "Unauthorized Message: Command Message" finding emitted (confidence Possible).
2. Finding message includes: current N(S), prev N(S) (`prev`), and the gap value.
3. The directional field is updated to `Some(current_ns)`.

## Invariants

1. **15-bit modular arithmetic**: gap calculation must apply `& 0x7FFF` after `wrapping_sub` to correctly mask to the 15-bit N(S) range — `u16::wrapping_sub` wraps at 2^16 (65536), not 2^15 (32768). Without the mask, values in the upper 15-bit range produce false-positive gaps.
2. **k=12 constant**: the k-value is fixed at 12 for MVP. Future enhancement: configurable via `--iec104-k-window`.
3. **First-frame `None` sentinel (mid-capture correctness)**: `last_ns_c2s` and `last_ns_s2c` are initialized to `None`. The first observed I-frame in each direction unconditionally sets `Some(ns)` and emits NO finding. This is the primary correctness guard for mid-session packet captures — the analyzer's primary use case — where the first observed N(S) is an arbitrary value (not necessarily 0) and any gap relative to an assumed zero baseline would be a false positive.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First I-frame on fresh flow (any N(S), state `None`) | State `None` → `Some(N(S))`; NO finding unconditionally (baseline established) |
| EC-002 | Gap = 12 (exactly k) on subsequent frame (`Some(prev)`) | No finding (≤ k is allowed) |
| EC-003 | Gap = 13 (k+1) on subsequent frame (`Some(prev)`) | T1692.001 Possible |
| EC-004 | Sequence wrap: state `Some(32767)`, current_ns=1 | `(1u16.wrapping_sub(32767) & 0x7FFF)` = `(32770 & 0x7FFF)` = 2; no finding; state → `Some(1)` |
| EC-005 | Gap = 32767 (massive jump / replay) on subsequent frame | T1692.001 Possible |
| EC-006 | Mid-capture start: first N(S)=5000 (state `None`) | Baseline set to `Some(5000)`; no finding. Next: N(S)=5001 → gap 1, no finding (`Some(5001)`). Next: N(S)=5020 → gap 19 > 12 → T1692.001 Possible |

## Canonical Test Vectors

| State (last_ns_dir) | current_ns | Gap (15-bit) | Expected |
|--------------------|------------|--------------|----------|
| `None` (first frame) | 0 | N/A | No finding; state → `Some(0)` |
| `None` (first frame) | 5000 | N/A | No finding; state → `Some(5000)` |
| `Some(5000)` | 5001 | 1 | No finding; state → `Some(5001)` |
| `Some(5001)` | 5020 | 19 | T1692.001 Possible; state → `Some(5020)` |
| `Some(0)` | 12 | 12 | No finding |
| `Some(0)` | 13 | 13 | T1692.001 Possible |
| `Some(32767)` | 1 | 2 | No finding |
| `Some(100)` | 114 | 14 | T1692.001 Possible |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in gap detection; wrapping arithmetic produces no overflow for any (`Option<u16>` last_ns state, u16 current_ns) | cargo-fuzz: `fuzz_iec104_parser` |

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

- `src/analyzer/iec104.rs` — `match last_ns_dir { None => { *last_ns_dir = Some(current_ns); /* baseline — no finding */ } Some(prev) => { let gap = current_ns.wrapping_sub(prev) & 0x7FFF; if gap > 12 { emit T1692_001(Possible); } *last_ns_dir = Some(current_ns); } }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 6`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser` (no-panic for all gap computation paths)
