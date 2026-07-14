---
document_type: story
story_id: STORY-170
title: "IEC-104 Control Command Detection: TypeIDs 45–51, C_RP, Interrogation, Reserved TypeIDs"
epic_id: E-22
wave: 79
points: 5
phase: f3
tdd_mode: strict
status: draft
feature_id: feature-iec104
subsystems: [SS-19]
target_module: analyzer/iec104
depends_on: [STORY-169]
blocks: [STORY-171, STORY-172]
behavioral_contracts:
  - BC-2.19.019
  - BC-2.19.020
  - BC-2.19.021
  - BC-2.19.022
verification_properties:
  - VP-047
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.019.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.020.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.021.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.022.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "b7c8dd3"
---

# STORY-170: IEC-104 Control Command Detection: TypeIDs 45–51, C_RP, Interrogation, Reserved TypeIDs

## Narrative

**As a** security analyst using wirerust to detect adversarial ICS/SCADA commands,
**I want** the IEC-104 analyzer to emit findings for control command TypeIDs, system reset
commands, interrogation (reconnaissance) commands, and reserved TypeIDs,
**so that** MITRE ICS ATT&CK techniques T1692.001 (control commands), T0836 (modifying
control logic), T0827 (loss of control / system reset / reconnaissance), and T0814
(DoS via reserved TypeID) are detected in passive observation.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.019 | Control Command TypeIDs 45–51: T1692.001 Possible | TypeID-range detection — control commands |
| BC-2.19.020 | C_RP TypeID 105 (Reset Process Command): T0827 Possible | System-reset detection |
| BC-2.19.021 | Interrogation TypeIDs 100/101/103: T0827 Possible (Reconnaissance) | Reconnaissance detection |
| BC-2.19.022 | Reserved TypeIDs: T0814 Possible (Anomaly) | Reserved/unknown TypeID anomaly |

## Acceptance Criteria

### AC-170-001: Control TypeIDs 45–51 emit T1692.001 Possible
**Traces to:** BC-2.19.019 postconditions 1–2
- Given an I-format frame whose ASDU TypeID is in the range [45, 51]
  (TypeIDs 45=C_SC_NA, 46=C_DC_NA, 47=C_RC_NA, 48=C_SE_NA, 49=C_SE_NB, 50=C_SE_NC, 51=C_BO_NA)
- When the control command detection step processes the extracted `AsduHeader`
- Then a T1692.001 "Unauthorized Message: Command Message" finding is emitted with confidence Possible
- T1692.001 represents adversarial use of IEC-104 control commands to manipulate field devices

### AC-170-002: C_RP TypeID 105 emits T0827 Possible
**Traces to:** BC-2.19.020 postconditions 1–2
- Given an I-format frame with TypeID=105 (C_RP_NA_1 — Reset Process Command)
- When the detection step processes the ASDU header
- Then a T0827 "Loss of Control" finding is emitted with confidence Possible
- C_RP resets RTU/IED processes; adversarial use causes equipment malfunction or loss of control

### AC-170-003: Interrogation TypeIDs 100, 101, 103 emit T0827 Possible
**Traces to:** BC-2.19.021 postconditions 1–2
- Given an I-format frame with TypeID in {100, 101, 103}
  (100=C_IC_NA general interrogation, 101=C_CI_NA counter interrogation, 103=C_CS_NA clock sync)
- When the detection step processes the ASDU header
- Then a T0827 "Loss of Control" finding is emitted with confidence Possible
- Interrogation commands enumerate IED state; adversarial use enables network reconnaissance
  before a more targeted attack

### AC-170-004: Reserved/unrecognized TypeIDs emit T0814 Possible
**Traces to:** BC-2.19.022 postconditions 1–2 and invariant 1
- Given an I-format frame with TypeID outside the recognized IEC-104 operational set
  (not in [45–51], not 100, 101, 103, 105, and not standard monitoring TypeIDs [1–44, 52–99, 102])
- When the detection step processes the ASDU header
- Then a T0814 "Denial of Service" finding is emitted with confidence Possible
- Reserved TypeIDs may indicate crafted packets probing IEC-104 implementations for vulnerabilities

### AC-170-005: Standard monitoring TypeIDs produce no finding
**Traces to:** BC-2.19.019 invariant 2 (no false positives on monitoring frames)
- Given an I-format frame with TypeID in the standard monitoring range [1–44]
  (process information in monitoring direction: single/double point, measured values, etc.)
- When the detection step processes the ASDU header
- Then no finding is emitted — monitoring TypeIDs represent normal IEC-104 telemetry

### AC-170-006: TypeID dispatch is exhaustive — every TypeID produces exactly one outcome
**Traces to:** BC-2.19.022 invariant 1 (exhaustive dispatch)
- The TypeID dispatch table must handle all 256 possible TypeID values (0–255):
  - 0: anomalous (reserved); emit T0814
  - 1–44: monitoring; no finding
  - 45–51: control commands; T1692.001
  - 52–99: other ASDU types; no finding (standard telemetry/control responses)
  - 100, 101, 103: interrogation; T0827
  - 102, 104, 106+: standard/reserved; T0814 for clearly reserved values
  - 105: C_RP reset; T0827
- No TypeID value must fall through unhandled; every arm must be explicit

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `detect_iec104_threats` | SS-19 threat detection | `src/analyzer/iec104.rs` | Effectful (emits findings) |
| TypeID dispatch match arm | SS-19 dispatch table | `src/analyzer/iec104.rs` | Effectful |

Subsystem anchor: SS-19 owns this story's scope because TypeID-based threat detection is
a core behavioral detection capability of the IEC-104 passive analyzer per ARCH-INDEX.md §SS-19.

## Tasks

- [ ] Implement `detect_iec104_threats(header: &AsduHeader, findings: &mut Vec<Finding>)` or
  equivalent inline in `on_data`; accepts `AsduHeader` from STORY-169's `extract_asdu_header`
- [ ] TypeID dispatch table (exhaustive match arm):
  - 45..=51 → emit T1692.001 Possible (BC-2.19.019)
  - 105 → emit T0827 Possible (C_RP reset) (BC-2.19.020)
  - 100 | 101 | 103 → emit T0827 Possible (interrogation) (BC-2.19.021)
  - Reserved/anomalous ranges → emit T0814 Possible (BC-2.19.022)
  - Standard monitoring ranges → no finding (BC-2.19.019 invariant 2)
- [ ] Write unit tests: one per AC, named `test_BC_2_19_019_*`, etc.
- [ ] Verify TypeID=0 (reserved) emits T0814 (edge case EC-001)
- [ ] Verify `cargo test` passes

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.022 | TypeID=0 (reserved by spec) | T0814 Possible |
| EC-002 | BC-2.19.019 | TypeID=44 (max monitoring) | No finding |
| EC-003 | BC-2.19.019 | TypeID=45 (min control) | T1692.001 Possible |
| EC-004 | BC-2.19.019 | TypeID=51 (max control) | T1692.001 Possible |
| EC-005 | BC-2.19.019 | TypeID=52 (above control range) | No finding (standard ASDU type) |
| EC-006 | BC-2.19.020 | TypeID=105 specifically | T0827 Possible (not T1692.001) |
| EC-007 | BC-2.19.021 | TypeID=102 (C_RP_NA_1 counter-read) | No finding or T0814 based on spec placement |
| EC-008 | BC-2.19.022 | TypeID=255 (max, unspecified) | T0814 Possible |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| BC-2.19.019–022 (4 BCs × ~600 each) | ~2,400 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-169) | ~7,000 |
| Test file delta | ~2,000 |
| TOTAL | ~34,400 |

Agent context window ~200k tokens. This story uses ~17% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-169 (ASDU header extraction)
- STORY-169 defines `AsduHeader` struct and `extract_asdu_header` free fn
- This story consumes `AsduHeader::type_id` to make detection decisions
- T1692.001 is also emitted in STORY-171 (N(S) gap detection); the same technique ID string
  is used — no conflict as they represent different detection paths
- T0836 "Modifying Control Logic" may also apply to TypeIDs 45–51 in addition to T1692.001;
  check BC-2.19.019 for the exact technique list

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 8**: TypeID dispatch happens in the effectful `on_data` shell after
  `extract_asdu_header` returns `Some(header)`. The extraction is pure; the finding emission
  is effectful.
- **TypeID ranges**: Control command TypeIDs [45–51] are the primary detection set. TypeIDs
  [1–44] are monitoring direction (single point, measured values) — no findings.
- **T0836 note**: ADR-013 may specify whether T0836 is emitted alongside T1692.001 for
  TypeIDs 45–51. Check the BC before implementing to avoid under- or over-emitting.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | match arms, range patterns |

No new crate dependencies.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add TypeID dispatch logic (in `on_data` effectful path or extracted `detect_iec104_threats` fn) |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.019–022 unit tests |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- Finding emission must NOT occur inside `extract_asdu_header` (pure) — only in the effectful caller
