---
document_type: story
level: ops
story_id: STORY-170
title: "IEC-104 Control Command Detection: TypeIDs 45–51, C_RP, Interrogation, Reserved TypeIDs"
epic_id: E-22
version: "2.0"
status: delivered
producer: story-writer
timestamp: 2026-07-15T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-169]
blocks: [STORY-171, STORY-172]
behavioral_contracts: [BC-2.19.017, BC-2.19.019, BC-2.19.020, BC-2.19.021, BC-2.19.022]
verification_properties: [VP-047]
priority: P1
cycle: feature-iec104
wave: 79
target_module: analyzer/iec104
subsystems: [SS-19]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-iec104
modified:
  - "v2.0: pre-delivery BC-realignment — AsduHeader→Asdu / extract_asdu_header→parse_asdu (STORY-169 delivered broken-out Asdu struct); added AC-170-007 cot_test [TEST]-tagging tracing BC-2.19.017 inv1; AC-170-001 extended to include T0836 for TypeIDs 48–51 (BC-2.19.019 postcondition 2); AC-170-002 confidence Possible→Likely (BC-2.19.020 v1.1 postcondition 1); AC-170-003 rewritten — interrogation TypeIDs 100/101/103 emit NO finding (BC-2.19.021 postcondition 1, was erroneously T0827 Possible); AC-170-004 scope tightened to TypeID=0 or >=128 (BC-2.19.022 precondition 2); AC-170-006 dispatch table corrected for silently-logged range [52–127]; BC-2.19.017 added to behavioral_contracts and inputs."
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.017.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.019.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.020.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.021.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.022.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
  - .factory/phase-f1-delta-analysis/feature-iec104-research.md
input-hash: "7c3c35c"
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
| BC-2.19.017 | COT Extraction (Cause of Transmission) from ASDU Bytes 2–3 | `cot_test` field — test-frame `[TEST]` tagging for analyst noise reduction |
| BC-2.19.019 | Control Command TypeIDs 45–51 Emit T1692.001; Set-Point + Bitstring TypeIDs 48–51 Also Emit T0836 | TypeID-range detection — switching commands emit T1692.001; set-point/bitstring TypeIDs 48–51 also emit T0836 |
| BC-2.19.020 | C_RP_NA_1 (TypeID 105) Emits T0827 "Loss of Control" Finding | System-reset detection |
| BC-2.19.021 | Interrogation and Clock-Sync Commands (TypeIDs 100, 101, 103) Are Logged Without Findings | Interrogation/clock-sync commands — no finding, logged only |
| BC-2.19.022 | Reserved or Invalid TypeID Emits T0814 Anomaly | TypeID=0 and 128–255 anomaly detection; [1–127] defined-but-unhandled silently logged |

## Acceptance Criteria

### AC-170-001: Control TypeIDs 45–51 emit T1692.001 Possible; set-point/bitstring TypeIDs 48–51 also emit T0836 Possible
- Given an I-format frame whose ASDU TypeID is in the range [45, 51]
  (TypeIDs 45=C_SC_NA, 46=C_DC_NA, 47=C_RC_NA, 48=C_SE_NA, 49=C_SE_NB, 50=C_SE_NC, 51=C_BO_NA)
- When `detect_iec104_threats` processes the parsed `Asdu` (from `parse_asdu`)
- Then T1692.001 "Unauthorized Message: Command Message" is emitted with confidence Possible for all TypeIDs 45–51
- And for set-point/bitstring TypeIDs 48–51 (C_SE_NA, C_SE_NB, C_SE_NC, C_BO_NA), T0836 "Modify Parameter" is also emitted with confidence Possible
- Switching commands 45–47 (C_SC, C_DC, C_RC) emit T1692.001 only — they represent binary control, not parameter writes
(traces to BC-2.19.019 postconditions 1–2)

### AC-170-002: C_RP TypeID 105 emits T0827 Likely
- Given an I-format frame with TypeID=105 (C_RP_NA_1 — Reset Process Command)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then a T0827 "Loss of Control" finding is emitted with confidence Likely
- C_RP resets RTU/IED processes; adversarial use causes equipment malfunction or loss of control
(traces to BC-2.19.020 postconditions 1–2)

### AC-170-003: Interrogation TypeIDs 100, 101, 103 produce no finding (logged at trace level only)
- Given an I-format frame with TypeID in {100, 101, 103}
  (100=C_IC_NA_1 general interrogation, 101=C_CI_NA_1 counter interrogation, 103=C_CS_NA_1 clock sync)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then no security finding is emitted — interrogation and clock-sync are benign administrative commands
- The ASDU is logged at trace level with CASDU and COT recorded, but no Finding is added to the findings buffer
(traces to BC-2.19.021 postconditions 1–3)

### AC-170-004: TypeID=0 or TypeID in [128, 255] emits T0814 Possible
- Given an I-format frame with TypeID=0 (undefined by IEC 60870-5-104) or TypeID in [128, 255] (private-use/reserved range)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then a T0814 "Denial of Service" finding is emitted with confidence Possible
- TypeIDs in [1, 127] that are not explicitly handled are silently logged — only TypeID=0 and 128–255 trigger T0814
(traces to BC-2.19.022 postconditions 1–2 and invariant 1)

### AC-170-005: Defined-but-unhandled TypeIDs in [1, 127] produce no finding (silently logged)
- Given an I-format frame with TypeID in the defined-but-unhandled range [1, 127] excluding the explicitly
  handled sets ({45–51}, {100, 101, 103}, {105}) — for example TypeIDs 1–44 (monitoring), 52–99, 102, 104, 106–127
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then no finding is emitted — these TypeIDs are silently logged (future-proof design per BC-2.19.022)
(traces to BC-2.19.022 invariant 1)

### AC-170-006: TypeID dispatch is exhaustive — every TypeID produces exactly one outcome
- The TypeID dispatch table must cover all 256 possible TypeID values (0–255) without fallthrough:
  - 0: T0814 Possible (undefined, per BC-2.19.022 precondition 2)
  - 1–44: silently logged, no finding (monitoring direction, per BC-2.19.022 invariant 1)
  - 45–47: T1692.001 Possible only (switching commands C_SC, C_DC, C_RC — per BC-2.19.019 invariant 2)
  - 48–51: T1692.001 Possible + T0836 Possible (set-point/bitstring, per BC-2.19.019 postcondition 2)
  - 52–99: silently logged, no finding (defined range, per BC-2.19.022 invariant 1)
  - 100, 101, 103: no finding, logged (interrogation/clock-sync, per BC-2.19.021 postcondition 1)
  - 102, 104, 106–127: silently logged, no finding (defined range, per BC-2.19.022 invariant 1)
  - 105: T0827 Likely (C_RP reset, per BC-2.19.020 postcondition 1)
  - 128–255: T0814 Possible (private-use/reserved, per BC-2.19.022 precondition 2)
(traces to BC-2.19.022 invariant 1)

### AC-170-007: Control-command findings are tagged [TEST] when cot_test is true
- Given an I-format frame whose ASDU would otherwise produce a finding (TypeID in a detection set)
- And the parsed `Asdu` has `cot_test == true` (bit 7 of COT byte 2 is set, per BC-2.19.017 postcondition 3)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then the emitted finding's `summary` field is appended with ` [TEST]` to tag it as a test-frame finding
- No new `Finding` struct field is required — the `[TEST]` tag is applied to `Finding::summary` (the existing string field)
- This reduces analyst noise: test transmissions are still recorded but visually distinguished from operational findings
(traces to BC-2.19.017 invariant 1)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `detect_iec104_threats` | SS-19 threat detection | `src/analyzer/iec104.rs` | Effectful (emits findings) |
| TypeID dispatch match arm | SS-19 dispatch table | `src/analyzer/iec104.rs` | Effectful |

Subsystem anchor: SS-19 owns this story's scope because TypeID-based threat detection is
a core behavioral detection capability of the IEC-104 passive analyzer per ARCH-INDEX.md §SS-19.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_asdu` (from STORY-169) | pure-core | No I/O, no state mutation, no finding emission; returns `Option<Asdu>` by value |
| `Asdu` struct (from STORY-169) | pure-core | Plain data; no methods that perform I/O or mutation |
| `detect_iec104_threats` (or inline `on_data` dispatch) | effectful-shell | Calls `parse_asdu`, reads `asdu.type_id` and `asdu.cot_test`, emits findings into `Vec<Finding>` |
| TypeID dispatch match arm | effectful-shell | Selects technique (T1692.001 / T0836 / T0827 / T0814) and appends `[TEST]` tag when `cot_test == true` |

## Tasks

- [ ] Implement `detect_iec104_threats(asdu: &Asdu, findings: &mut Vec<Finding>)` or
  equivalent inline in `on_data`; accepts `Asdu` from STORY-169's `parse_asdu`
- [ ] TypeID dispatch table (exhaustive match arm, no fallthrough):
  - 0 | 128..=255 → emit T0814 Possible (BC-2.19.022)
  - 45..=47 → emit T1692.001 Possible only — switching commands (BC-2.19.019 invariant 2)
  - 48..=51 → emit T1692.001 Possible + T0836 Possible — set-point/bitstring (BC-2.19.019 postconditions 1–2)
  - 105 → emit T0827 Likely (C_RP reset) (BC-2.19.020)
  - 100 | 101 | 103 → log at trace level, no finding (BC-2.19.021)
  - _ → silently log, no finding (defined-but-unhandled TypeIDs [1–127] not in above sets)
- [ ] When `asdu.cot_test == true`, append ` [TEST]` to each emitted finding's `summary` field (BC-2.19.017 invariant 1)
- [ ] Write unit tests: one per AC, named `test_BC_2_19_019_*`, `test_BC_2_19_020_*`, `test_BC_2_19_021_*`, `test_BC_2_19_022_*`, `test_BC_2_19_017_cot_test_*`
- [ ] Verify TypeID=0 (reserved) emits T0814 (EC-001)
- [ ] Verify TypeID=51 emits both T1692.001 and T0836 (EC-004)
- [ ] Verify TypeID=105 emits T0827 with confidence Likely (EC-006)
- [ ] Verify `cot_test=true` control command finding has `[TEST]` in summary (EC-009)
- [ ] Verify `cargo test` passes

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.022 | TypeID=0 (undefined by spec) | T0814 Possible |
| EC-002 | BC-2.19.022 | TypeID=44 (max monitoring, defined-but-unhandled) | No finding (silently logged) |
| EC-003 | BC-2.19.019 | TypeID=45 (min switching control) | T1692.001 Possible only (no T0836) |
| EC-004 | BC-2.19.019 | TypeID=51 (C_BO_NA_1, max control — bitstring) | T1692.001 Possible + T0836 Possible |
| EC-005 | BC-2.19.022 | TypeID=52 (RESERVED per IEC 60870-5-104, above control range) | No finding (silently logged per BC-2.19.022 invariant 1) |
| EC-006 | BC-2.19.020 | TypeID=105 specifically | T0827 **Likely** (not Possible; not T1692.001) |
| EC-007 | BC-2.19.022 | TypeID=102 (C_RD_NA_1, defined but not in any detection set) | No finding (silently logged — falls through to defined-but-unhandled path) |
| EC-008 | BC-2.19.022 | TypeID=255 (max, private-use/reserved) | T0814 Possible |
| EC-009 | BC-2.19.017 | cot_test=true with TypeID=45 (control command test frame) | T1692.001 Possible emitted with ` [TEST]` appended to `Finding::summary` |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,000 |
| BC-2.19.017, BC-2.19.019–022 (5 BCs × ~600 each) | ~3,000 |
| ss-19-iec104-analysis.md (SS-19 shard) | ~8,000 |
| ADR-013 architecture decisions | ~12,000 |
| src/analyzer/iec104.rs (from STORY-169) | ~7,000 |
| Test file delta | ~2,000 |
| TOTAL | ~34,400 |

Agent context window ~200k tokens. This story uses ~17% — within budget.

## Previous Story Intelligence

**Predecessor:** STORY-169 (ASDU parser)
- STORY-169 defines `Asdu` struct (broken-out DUI fields) and `parse_asdu` pure free fn
- This story consumes `asdu.type_id` and `asdu.cot_test` from the parsed `Asdu` to make
  detection decisions — do NOT reference the old `AsduHeader` or `extract_asdu_header` names
- T1692.001 is also emitted in STORY-171 (N(S) gap detection); the same technique ID string
  is used — no conflict as they represent different detection paths
- T0836 "Modify Parameter" applies to TypeIDs 48–51 only (set-point/bitstring), per BC-2.19.019
  postcondition 2 — TypeIDs 45–47 (switching commands) emit T1692.001 only, not T0836

## Architecture Compliance Rules

Extracted from `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:
- **ADR-013 Decision 8**: TypeID dispatch happens in the effectful `on_data` shell after
  `parse_asdu` returns `Some(asdu)`. The extraction is pure; the finding emission is effectful.
- **TypeID ranges**: Control command TypeIDs [45–51] are the primary detection set. TypeIDs
  [1–44] are monitoring direction (single point, measured values) — silently logged, no findings.
- **T0836 resolved**: T0836 "Modify Parameter" is co-emitted ONLY for set-point/bitstring
  TypeIDs 48–51 per BC-2.19.019 postcondition 2. Switching commands 45–47 emit T1692.001 only.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | match arms, range patterns |

No new crate dependencies.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add TypeID dispatch logic (in `on_data` effectful path or extracted `detect_iec104_threats` fn) |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add BC-2.19.017/019–022 unit tests (AC-170-001 through AC-170-007) |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870` — banned (ADR-013 Decision 7)
- Finding emission must NOT occur inside `parse_asdu` (pure) — only in the effectful caller (`detect_iec104_threats` or `on_data`)
