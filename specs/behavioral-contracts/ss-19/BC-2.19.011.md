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
    ref: F-P8-M1
    note: >
      STOPDT-con (0x23) must not emit T0881 per ADR-013 Decision 5 (ACT-only MVP).
      EC-002 expected behavior corrected from "T0881 Possible if previously started"
      to no-finding with session state bookkeeping preserved.
      Canonical vector row 0x23 changed from "T0881 Possible" to "none".
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

# BC-2.19.011: STOPDT-act (CF1=0x13) After STARTDT Emits T0881 (Possible) "Service Stop"

## Description

When the U-format session state machine receives CF1=0x13 (STOPDT-act) on a flow where
`session_started == true`, it emits a T0881 "Service Stop" (ICS) finding
with confidence Possible. STOPDT (Stop Data Transfer) transitions a station from the
active controlled state back to the stopped state. In an ICS context, this is a potential
indicator of deliberate service interruption or a legitimate maintenance procedure.
The baseline confidence is Possible because STOPDT appears in normal operational sessions.
The elevated-confidence path is BC-2.19.012 (STOPDT without prior STARTDT).

## Preconditions

1. A valid U-format APCI frame with CF1=0x13 (STOPDT-act) has been parsed.
2. `Iec104FlowState::session_started == true` for this flow.

## Postconditions

1. `Iec104FlowState::session_started` is set to `false`.
2. A T0881 "Service Stop" finding is emitted with confidence Possible.
3. The finding includes the flow's source/destination addresses (5-tuple context).

## Invariants

1. **Single finding per STOPDT**: exactly one T0881 finding per STOPDT-act frame; no dedup needed (each STOPDT is a distinct event).
2. **State reset**: session_started → false after STOPDT-act; subsequent I-frames are anomalous until next STARTDT.
3. **Confidence level**: Possible (not Likely) because STOPDT is standard IEC-104 behavior for planned maintenance.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | STOPDT-act when session_started=true | T0881 Possible; session_started→false |
| EC-002 | STOPDT-con (CF1=0x23) received | Recognized as U-frame; session_started set false; NO finding (STOPDT-con is a benign confirmation; ADR-013 Decision 5 ACT-only MVP) |
| EC-003 | STOPDT-act followed immediately by STARTDT-act | T0881 emitted on STOPDT; session restarted on STARTDT |
| EC-004 | Multiple consecutive STOPDT-acts | Each emits T0881 Possible (each is an independent event) |

## Canonical Test Vectors

| CF1 | Prior session_started | Expected session_started | Finding |
|-----|-----------------------|--------------------------|---------|
| `0x13` | `true` | `false` | T0881 Possible |
| `0x13` | `false` | `false` | T0881 Likely (see BC-2.19.012) |
| `0x23` | `true` | `false` | none |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in session state machine on STOPDT-act | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — STOPDT detection and T0881 mapping is a core ICS threat-detection requirement of the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decisions 5, 10 (VP-007 T0881 atomic obligation) |
| Feature | feature-iec104 |
| MITRE Techniques | T0881 "Service Stop" (IcsInhibitResponseFunction / TA0107) — Possible confidence |

## Related BCs

- BC-2.19.010 — composes with (STARTDT: sets session_started=true)
- BC-2.19.012 — composes with (STOPDT without prior STARTDT → Likely)
- BC-2.10.010 — depends on (T0881 must be in SEEDED_TECHNIQUE_IDS catalog)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if cf1 == 0x13 { state.session_started = false; emit T0881(Possible); }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 5`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 10` — VP-007 T0881 six-part atomic obligation

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
