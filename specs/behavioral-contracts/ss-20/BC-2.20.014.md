---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-09-06T00:00:00Z
phase: f2
origin: greenfield
extracted_from: null
traces_to: .factory/specs/domain/domain-spec.md
subsystem: SS-20
capability: CAP-20
lifecycle_status: active
introduced: feature-s7comm
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md
  - .factory/specs/architecture/ARCH-INDEX.md
input-hash: "cf116b5"
---

# BC-2.20.014: Carry Buffer Bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535`; Overflow Triggers Clear-and-Resync With One T0814 Per Direction

## Description

Per ADR-014 Decision 8, the residual partial-frame carry left after the frame-walk
loop (BC-2.20.013) is bounded at `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535` — derived
from the TPKT `length` field's own maximum representable value (`u16::MAX`, RFC 1006
§5), **not** from COTP's single-byte Length Indicator (max 254). This is two orders of
magnitude larger than every prior binary-ICS carry cap (IEC-104: 255 bytes; DNP3: 292
bytes; ENIP: 600 bytes) because it reflects S7comm's actual on-wire ceiling — classic
block-download PDUs are the traffic class most likely to approach it — not an
under-specified default. When the residual exceeds this bound, the offending
direction's carry is cleared, the walk resyncs to the next `0x03` version-byte
candidate (BC-2.20.015), and exactly one T0814 (Denial of Service,
`ThreatCategory::Anomaly` / `Verdict::Possible` / `Confidence::Medium`) finding is
emitted per flow direction, guarded by a dedicated carry-overflow dedup flag distinct
from any malformed-length dedup flag.

## Preconditions

1. The frame-walk loop (BC-2.20.013) has completed its extraction pass for the current
   `on_data` call, leaving a residual partial-frame tail.
2. `residual.len() > MAX_S7_ISO_ON_TCP_CARRY_BYTES` (i.e. `> 65,535`).

## Postconditions

1. `carry[direction]` is cleared (set to empty) — the oversized residual is discarded,
   not truncated or partially retained.
2. The walk resyncs: it scans the discarded bytes (or continues scanning subsequent
   incoming bytes) for the next `0x03` version-byte candidate, advancing 1 byte at a
   time (BC-2.20.015) — this is a fresh-start resync, **not** a permanent desync latch;
   the flow remains tracked and subsequent valid frames are parsed normally.
3. Exactly one T0814 finding is emitted for this direction, with
   `verdict: Possible`, `confidence: Medium`, `threat_category: Anomaly`, guarded by a
   per-direction dedup flag (e.g. `carry_overflow_reported_c2s` /
   `carry_overflow_reported_s2c` on `S7commFlowState`) so that repeated overflow events
   in the same direction on the same flow do not each produce a new finding.
4. This dedup flag is **separate** from any malformed-TPKT/COTP-length dedup flag —
   each anomaly class has its own suppression flag, mirroring the IEC-104 precedent
   (BC-2.19.025/026's distinct `carry_overflow_reported_*` vs. `malformed_len_reported_*`
   flags).

## Invariants

1. **Bound derivation is exact, not heuristic**: `65,535 == u16::MAX`, the maximum value
   the TPKT `length` field can ever represent. Any legitimately conformant single-frame
   residual can never exceed this bound (see Edge Cases EC-002) — the bound is
   defense-in-depth against adversarial or corrupted input, unreachable for conformant
   traffic, mirroring IEC-104's "conformant residual ≤ 254 bytes so the bound is
   fail-closed defense-in-depth" characterization (BC-2.19.025 F-172-201).
2. **Clear, not truncate**: on overflow, the entire residual is discarded — there is no
   attempt to salvage a prefix of it, since a >65,535-byte undelimited residual has no
   reliable frame boundary to preserve.
3. **Resync, not desync**: the flow is never permanently abandoned; a single T0814 is
   emitted and normal parsing resumes as soon as a valid `0x03` candidate is found and
   yields a parseable frame.
4. **Per-direction, per-flow scope**: the dedup flag lives on `S7commFlowState`, so a
   fresh flow (new TCP connection) gets a fresh overflow-reporting opportunity.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `residual.len() == 65,535` exactly (at the bound, not over it) | No overflow — this is the legitimate maximum-length single-frame case (BC-2.20.004 EC-002); bound comparison is strict `>`, not `>=` |
| EC-002 | A conformant TPKT frame declaring `length = 65,535` arrives split across many small TCP segments, with the carry temporarily holding up to `65,534` bytes of a still-incomplete frame | Never triggers overflow — the bound exactly accommodates the maximum legitimate single-frame residual; this is the load-bearing off-by-one correctness property named in the ADR (the carry cap must be `>=` the max declarable frame size, not merely "large") |
| EC-003 | An adversarial stream that never presents a valid TPKT header, causing the carry to grow past 65,535 bytes of accumulated garbage | Triggers exactly one T0814 on first crossing the bound; carry cleared; resync begins |
| EC-004 | A second overflow event occurs in the same direction on the same flow shortly after the first | No second T0814 emitted (dedup flag already set); carry is still cleared and resync still occurs each time |
| EC-005 | An overflow in `c2s` direction does not suppress or affect overflow detection in `s2c` on the same flow | Independent dedup flags per direction |

## Canonical Test Vectors

| Scenario | Input | Expected Behavior | Category |
|----------|-------|--------------------|---------|
| At-bound, legitimate | Residual of exactly 65,535 bytes from a valid `length=65,535` frame still incomplete | No overflow; carry retains all 65,535 bytes; no finding | legit: exact ceiling |
| Over-bound, adversarial | Residual of 65,536 bytes with no valid TPKT frame boundary found | Carry cleared; resync to next `0x03`; exactly one T0814 (Anomaly/Possible/Medium) for this direction | non-conformant: overflow |
| Repeated over-bound, same direction | Two consecutive overflow events, same flow, same direction | First event: T0814 emitted, dedup flag set. Second event: no finding emitted, dedup flag already set; carry still cleared and resync still occurs | non-conformant: dedup guard |

## Verification Properties

| Property | Proof Method (planned) |
|----------|-------------------------|
| `MAX_S7_ISO_ON_TCP_CARRY_BYTES = 65,535` exactly accommodates the maximum single-frame residual (`u16::MAX`); the overflow comparison is strict (`>`, not `>=`) so no conformant frame ever triggers a false overflow | Kani P0 (bound arithmetic per ADR-014 Decision 9) — VP-NNN allocation deferred to the F2 INTEGRATE sub-burst |
| Per-direction dedup guarantees at most one T0814 emission per direction per flow for repeated overflow events | proptest P1 — VP-NNN allocation deferred |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 |
| Capability Anchor Justification | CAP-20 ("ISO-on-TCP Framing (TPKT/COTP)") per domain/capabilities/cap-20-iso-on-tcp-framing.md §CAP-20 — the carry-buffer bound is the DoS-resistance property of the ISO-on-TCP framing layer, protecting against unbounded per-flow memory growth |
| L2 Domain Invariants | INV-2 (Content-First Dispatch Precedence); bounded-resource design invariant (mirrors SS-19's `MAX_IEC104_CARRY_BYTES` treatment in ARCH-INDEX Bounded-Resource Design) |
| Architecture Module | SS-20/SS-21 boundary; `S7commFlowState.carry_c2s`/`carry_s2c`, `MAX_S7_ISO_ON_TCP_CARRY_BYTES` constant (planned) |
| ADR | ADR-014 Decision 8 |
| Stories | (TBD — story-writer assigns in F3) |
| Feature | feature-s7comm |
| MITRE Techniques | T0814 (Denial of Service) — emitted on carry-overflow, `Verdict::Possible`, `Confidence::Medium`, `ThreatCategory::Anomaly` |

## Related BCs

- BC-2.20.004 — depends on (the `length == 65,535` maximum this bound is derived from)
- BC-2.20.013 — composes with (this BC bounds the residual BC-2.20.013 leaves in carry)
- BC-2.20.015 — composes with (the resync mechanism invoked after clearing the carry)

## Architecture Anchors

- `src/analyzer/iso_on_tcp.rs` or `S7commFlowState` (planned) — `const MAX_S7_ISO_ON_TCP_CARRY_BYTES: usize = 65_535;`
- `S7commFlowState.carry_overflow_reported_c2s: bool` / `carry_overflow_reported_s2c: bool` (planned) — per-direction dedup flags, distinct from any malformed-length dedup flag
- `docs/adr/0014-s7comm-iso-on-tcp-stream-dispatch-and-parser-design.md §Decision 8` — "Carry-buffer sizing" and "Carry-overflow reaction" subsections, full derivation and reaction spec

## Story Anchor

STORY-186

## VP Anchors

- VP-050 (proptest P1) — TPKT/COTP Carry-Buffer Residual-Bound Reassembly, Overflow
  Isolation, and 1-Byte Resync; registered F2 INTEGRATE sub-burst per VP-INDEX.md
  v2.48; traces BC-2.20.013..015 (supersedes this BC's own speculative separate
  "Kani P0" note — the registered VP-050 is a single proptest VP covering both the
  bound-arithmetic and dedup-flag sub-properties)
- VP-055 (cargo-fuzz P1) — S7comm/ISO-on-TCP combined parse-chain no-panic fuzz
  (`fuzz_s7comm_parser`); registered representative-subset source_bc includes this BC

## Purity Classification

| Property | Assessment |
|----------|-----------|
| **I/O operations** | none |
| **Global state access** | per-flow mutable state (`S7commFlowState` carry buffer and dedup flags) |
| **Deterministic** | yes — given the same sequence of `on_data` calls |
| **Thread safety** | flow state is per-flow |
| **Overall classification** | stateful orchestration; the bound-comparison arithmetic itself is a pure, Kani-provable sub-property |
