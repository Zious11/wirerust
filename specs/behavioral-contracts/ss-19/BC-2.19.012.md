---
document_type: behavioral-contract
level: L3
version: "1.0"
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
modified: []
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

# BC-2.19.012: STOPDT-act Without Prior STARTDT Emits T0881 (Likely) "Service Stop"

## Description

When the U-format session state machine receives CF1=0x13 (STOPDT-act) on a flow where
`session_started == false` (i.e., no STARTDT-act was observed prior on this flow), it
emits a T0881 finding with confidence Likely — elevated from Possible. A STOPDT arriving
before STARTDT is anomalous because IEC-104 stations must begin in the STOPPED state;
a STOPDT from a station that never sent STARTDT suggests mid-session observation (capture
started after STARTDT), replay injection, or an out-of-spec device. The distinction between
Possible (BC-2.19.011) and Likely (this BC) allows analyst triage prioritization.

## Preconditions

1. A valid U-format APCI frame with CF1=0x13 (STOPDT-act) has been parsed.
2. `Iec104FlowState::session_started == false` for this flow (no STARTDT-act seen yet).

## Postconditions

1. `Iec104FlowState::session_started` remains `false`.
2. A T0881 "Service Stop" finding is emitted with confidence Likely.
3. The finding includes a note: "STOPDT received without prior STARTDT on this flow".

## Invariants

1. **Confidence escalation**: Likely > Possible — prioritizes analyst attention for unexpected STOPDT.
2. **Capture-start ambiguity**: a pcap started mid-session may miss the STARTDT; Likely confidence acknowledges this rather than emitting a false-positive at Confirmed.
3. **State unchanged**: session_started stays false; no phantom state transition.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | First U-frame on new flow is STOPDT-act | T0881 Likely; session_started remains false |
| EC-002 | Capture begins mid-session (STARTDT missed) | T0881 Likely — correct triage signal without a hard alarm |
| EC-003 | STOPDT-act immediately after STARTDT-con | session_started=true (STARTDT-con already set it); applies BC-2.19.011 (Possible) instead |

## Canonical Test Vectors

| CF1 | Prior session_started | Expected | Finding |
|-----|-----------------------|----------|---------|
| `0x13` | `false` | `false` (unchanged) | T0881 Likely |
| `0x13` | `true` | `false` | T0881 Possible (BC-2.19.011) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-047 | No panic in session state machine on STOPDT-act with session_started=false | cargo-fuzz: `fuzz_iec104_parser` |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 |
| Capability Anchor Justification | CAP-19 ("IEC 60870-5-104 (IEC-104) Passive Analysis") per ARCH-INDEX.md §SS-19 — elevated-confidence STOPDT detection is a key detection-quality differentiator of the IEC-104 passive analyzer |
| L2 Domain Invariants | INV-1 (Protocol State Accuracy), INV-3 (Fail-Closed Finding Emission) |
| Architecture Module | SS-19 (src/analyzer/iec104.rs C-27); ADR-013 Decision 5 |
| Feature | feature-iec104 |
| MITRE Techniques | T0881 "Service Stop" — Likely confidence (elevated from Possible due to missing STARTDT) |

## Related BCs

- BC-2.19.010 — composes with (STARTDT: the missing prior event)
- BC-2.19.011 — composes with (STOPDT after STARTDT: Possible path)
- BC-2.10.010 — depends on (T0881 must be in SEEDED_TECHNIQUE_IDS catalog)

## Architecture Anchors

- `src/analyzer/iec104.rs` — `if cf1 == 0x13 { if !state.session_started { emit T0881(Likely); } else { state.session_started = false; emit T0881(Possible); } }`
- `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md §Decision 5`

## Story Anchor

(TBD — F3 story decomposition)

## VP Anchors

- VP-047 — `fuzz_iec104_parser`
