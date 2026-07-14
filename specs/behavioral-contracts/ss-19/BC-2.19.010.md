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
  - when: "2026-07-14"
    by: product-owner
    ref: F-P8-L1
    note: >
      PC-4 reworded: STARTDT-con (0x0B) can set session_started=true when observed
      without a prior STARTDT-act (consistent with EC-003 and the canonical test
      vector showing false→true on 0x0B). Previous wording only described the
      "already started" idempotent case, omitting the cold-start case.
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

# BC-2.19.010: STARTDT-act (CF1=0x07) Sets session_started=true in Iec104FlowState

## Description

When the U-format session state machine receives a U-frame with CF1=0x07 (STARTDT-act),
it sets `Iec104FlowState::session_started = true` for the flow. STARTDT (Start Data Transfer)
is the IEC-104 handshake that initiates the controlled state where I-format data frames may
be exchanged. Prior to STARTDT, a conformant IEC-104 station must not transmit I-frames.
The analyzer records this transition as a normal operational event — no finding is emitted
for STARTDT-act.

## Preconditions

1. A valid U-format APCI frame has been parsed (CF1 & 0x03 == 0x03).
2. CF1 == 0x07 (STARTDT-act).
3. `Iec104FlowState` exists for this flow (created on first on_data call).

## Postconditions

1. `Iec104FlowState::session_started` is set to `true`.
2. Subsequent I-format frames are accepted for ASDU parsing.
3. No finding is emitted (STARTDT is expected operational behavior).
4. STARTDT-con (CF1=0x0B) is also recognized; in the normal act→con ordering session_started is already true (no change), but a con observed without a prior act sets session_started=true (per EC-003 and the canonical vector).

## Invariants

1. **Idempotent**: receiving STARTDT-act when session_started is already true is valid (retransmit); state remains true.
2. **Session gate**: I-format frames received before STARTDT-act should note the anomaly but are still parsed (passive-only analyzer does not enforce STOPDT state on the remote).
3. **State ownership**: `Iec104FlowState` is per-flow; session_started is direction-agnostic (applies to the connection as a whole).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | STARTDT-act received on a fresh flow | session_started: false → true; no finding |
| EC-002 | Duplicate STARTDT-act | session_started remains true; no finding |
| EC-003 | STARTDT-con (CF1=0x0B) received | Recognized as U-frame; session_started already true or set true; no finding |
| EC-004 | I-frame before STARTDT-act | Parsed but anomaly noted (passive analyzer; not T0814 unless other violation) |

## Canonical Test Vectors

| Input CF1 | Prior session_started | Expected session_started | Finding Emitted |
|-----------|----------------------|--------------------------|-----------------|
| `0x07` | `false` | `true` | none |
| `0x07` | `true` | `true` | none |
| `0x0B` | `false` | `true` | none |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in session state machine for all U-frame CF1 values | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — STARTDT handshake tracking is an essential part of IEC-104 session state management in the passive analyzer |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 5 |
| Feature | feature-iec104 |
| MITRE Techniques | (none — normal session startup) |

## Related BCs

- BC-2.19.009 — depends on (U-format classification gate)
- BC-2.19.011 — composes with (STOPDT: session shutdown detection)
- BC-2.19.012 — composes with (STOPDT without STARTDT: anomaly)
- BC-2.19.013 — composes with (TESTFR: keepalive)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `Iec104FlowState::session_started: bool`
- `src/analyzer/iec104.rs` — U-frame dispatch: `if cf1 == 0x07 { state.session_started = true; }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 5` — STARTDT/STOPDT MVP

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
