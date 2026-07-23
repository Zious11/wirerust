---
document_type: story
level: ops
story_id: STORY-180
title: "IEC-104 Timed Control Command Detection: TypeIDs 58–64 (BC-2.19.029 + BC-2.19.030 + BC-2.19.022 v1.1 Regression Guard)"
epic_id: E-22
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-07-23T00:00:00Z
phase: f3
traces_to: .factory/specs/prd.md
points: 5
depends_on: [STORY-174]
blocks: []
behavioral_contracts:
  - BC-2.19.029
  - BC-2.19.030
  - BC-2.19.022
verification_properties: [VP-047]
priority: P1
cycle: feature-iec104
wave: 85
target_module: analyzer/iec104
subsystems: [SS-19]
estimated_days: null
assumption_validations: []
risk_mitigations: []
tdd_mode: strict
feature_id: feature-iec104
inputs:
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.029.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.030.md
  - .factory/specs/behavioral-contracts/ss-19/BC-2.19.022.md
  - .factory/planning/iec104-timed-cmd-gap-validation.md
  - .factory/specs/architecture/ss-19-iec104-analysis.md
  - docs/adr/0013-iec104-stream-dispatch-and-parser-design.md
input-hash: "c0fad6c"
---

# STORY-180: IEC-104 Timed Control Command Detection: TypeIDs 58–64 (BC-2.19.029 + BC-2.19.030 + BC-2.19.022 v1.1 Regression Guard)

**Epic:** E-22 (IEC-104 Passive Analyzer)
**Status:** draft
**Wave:** 85
**Points:** 5
**Priority:** P1

## Narrative

**As a** security analyst using wirerust to detect adversarial ICS/SCADA commands,
**I want** the IEC-104 analyzer to emit findings for the time-tagged (CP56Time2a) variants
of control command TypeIDs — C_SC_TA_1 (58), C_DC_TA_1 (59), C_RC_TA_1 (60),
C_SE_TA_1 (61), C_SE_TB_1 (62), C_SE_TC_1 (63), C_BO_TA_1 (64) — with the same
T1692.001 / T0836 attribution and evidence as their untimed twins (45–51),
**so that** an attacker who selects the time-tagged TypeID to evade detection (TypeIDs
58–64 previously fell silently through the `_` catch-all arm) is correctly attributed to
T1692.001 "Unauthorized Message: Command Message" (ICS), with T0836 "Modify Parameter"
co-emitted for the set-point and bitstring variants.

This story closes the evasion gap documented in
`.factory/planning/iec104-timed-cmd-gap-validation.md` (IEC104-TIMED-CMD-GAP-001,
CONFIRMED / HIGH confidence, 2026-07-23) by implementing BC-2.19.029 (arm 58..=60,
parity with untimed arm 45..=47) and BC-2.19.030 (arm 61..=64, parity with untimed arm
48..=51), and by narrowing the BC-2.19.022 v1.1 silent-range comment to reflect that
TypeIDs 58–64 are no longer in the silently-logged set.

## Behavioral Contracts

| BC ID | Title | Story Role |
|-------|-------|-----------|
| BC-2.19.029 | Time-Tagged Switching Control Commands (TypeIDs 58–60) Emit T1692.001 | New detection arm: timed switching control commands (C_SC_TA/C_DC_TA/C_RC_TA) → T1692.001 Possible, parity with arm 45..=47 |
| BC-2.19.030 | Time-Tagged Set-Point + Bitstring Commands (TypeIDs 61–64) Emit T1692.001 + T0836 | New detection arm: timed set-point/bitstring writes (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA) → T1692.001 + T0836 Possible, parity with arm 48..=51 |
| BC-2.19.022 | Reserved or Invalid TypeID Emits T0814 Anomaly (v1.1) | Regression guard: v1.1 narrows silently-logged range from 52–99 to {52–57, 65–99}; existing neighbor-silence tests must be updated to exclude 58–64; code comment at iec104.rs:912–914 must be narrowed |

## Acceptance Criteria

### AC-180-001: TypeIDs 58–60 emit exactly one T1692.001 Possible finding with CASDU and first_ioa evidence
- Given an I-format ASDU with TypeID in {58, 59, 60} (C_SC_TA_1, C_DC_TA_1, C_RC_TA_1)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then exactly one T1692.001 "Unauthorized Message: Command Message" finding is emitted
  with `Verdict::Possible`, `Confidence::Medium`, `ThreatCategory::Impact`
- The finding's evidence vector includes `CASDU=<value>` and, when `asdu.first_ioa` is
  `Some(ioa)`, also includes `first_ioa=<ioa>` — identical evidence parity to the untimed
  arm 45..=47 (lines 752–758 of the current implementation)
(traces to BC-2.19.029 postconditions 1 and 3)

### AC-180-002: TypeIDs 58–60 do NOT emit T0836 (switching commands are binary control, not parameter writes)
- Given an I-format ASDU with TypeID in {58, 59, 60}
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then T0836 "Modify Parameter" is NOT emitted — timed switching commands (timed single
  command, double command, regulating step) are binary control signals, not ICS parameter
  modifications
- This mirrors BC-2.19.019 Invariant 2 which withholds T0836 from untimed TypeIDs 45–47
(traces to BC-2.19.029 postcondition 2 and invariant 2)

### AC-180-003: TypeIDs 61–64 emit exactly one T1692.001 Possible AND one T0836 Possible finding with CASDU and first_ioa evidence
- Given an I-format ASDU with TypeID in {61, 62, 63, 64} (C_SE_TA_1, C_SE_TB_1, C_SE_TC_1, C_BO_TA_1)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then exactly two findings are emitted:
  1. T1692.001 "Unauthorized Message: Command Message" with `Verdict::Possible`,
     `Confidence::Medium`, `ThreatCategory::Impact`
  2. T0836 "Modify Parameter" with `Verdict::Possible`, `Confidence::Medium`,
     `ThreatCategory::Impact`
- Both findings' evidence vectors include `CASDU=<value>` and, when `asdu.first_ioa` is
  `Some(ioa)`, also include `first_ioa=<ioa>` — identical evidence parity to the untimed
  arm 48..=51 (lines 784–799 of the current implementation)
- T0836 is co-emitted because TypeIDs 61–64 are ICS parameter writes (set-point and
  bitstring output register writes), matching the rationale for the untimed arm
(traces to BC-2.19.030 postconditions 1, 2, and 3)

### AC-180-004: Timed-variant summary wording distinguishes from untimed twin summaries
- For TypeIDs 58–60: the T1692.001 finding `summary` field uses "time-tagged" qualifier and
  names the timed mnemonics (C_SC_TA/C_DC_TA/C_RC_TA), for example:
  `"IEC-104 time-tagged control command TypeID=<N> (C_SC_TA/C_DC_TA/C_RC_TA): time-tagged
  switching control command observed on passive monitor (T1692.001 unauthorized command
  message; BC-2.19.029)"`
- For TypeIDs 61–64: both the T1692.001 and T0836 finding `summary` fields name the timed
  mnemonics (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA), for example:
  `"IEC-104 time-tagged control command TypeID=<N> (C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA):
  time-tagged set-point or bitstring write command (T1692.001; BC-2.19.030)"`
- Neither timed summary string is identical to the corresponding untimed arm's summary —
  analysts must be able to distinguish timed from untimed findings in output
(traces to BC-2.19.029 postcondition 4; BC-2.19.030 postconditions 4 and 5)

### AC-180-005: cot_test=true appends [TEST] suffix to all emitted timed-command findings
- Given an I-format ASDU with TypeID in {58..=64} and `asdu.cot_test == true`
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then the ` [TEST]` suffix is appended to all emitted findings' `summary` fields by the
  existing post-emission loop at `detect_iec104_threats` lines 924–928
  (BC-2.19.017 Invariant 1 / AC-170-007 — no extra wiring needed in the new arms;
  the loop already runs over all findings added during the call)
(traces to BC-2.19.029 postcondition 6; BC-2.19.030 postcondition 7)

### AC-180-006: TypeIDs 52–57 and 65–99 still produce zero findings (BC-2.19.022 v1.1 regression guard)
- Given an I-format ASDU with TypeID in {52, 53, 54, 55, 56, 57} (reserved, below new arms)
  or TypeID in {65, 66, ..., 99} (unhandled, above new arms)
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then no finding is emitted — these TypeIDs remain in the silently-logged set per
  BC-2.19.022 v1.1 invariant 1 and the new arms' invariant 6
- Existing "neighbor-silence" unit tests that previously covered the full 52–99 range MUST
  be updated: assertions for TypeIDs 58–64 must be removed (these now produce findings) and
  assertions for TypeIDs 52–57 and 65–99 must be retained or added explicitly as
  `test_BC_2_19_022_v1_1_type_id_<N>_no_finding` to form the regression guard baseline
(traces to BC-2.19.022 v1.1 invariant 1; BC-2.19.029 invariant 6; BC-2.19.030 invariant 6)

### AC-180-007: The silent-range code comment at src/analyzer/iec104.rs lines 912–914 is narrowed to {52–57, 65–99}
- The code comment at `detect_iec104_threats` lines 912–914 currently names the silent
  range as including 52–99; after this story it MUST read "{52–57, 65–99}" (or
  equivalent) and note that TypeIDs 58–64 were removed from the silently-logged set
- The updated comment MUST state that 58–64 are now handled by BC-2.19.029 (58–60) and
  BC-2.19.030 (61–64), matching the architecture anchor in BC-2.19.022 v1.1
- This is a required implementation task, not optional — the comment is the machine-readable
  contract for the `_` catch-all arm
(traces to BC-2.19.022 v1.1 architecture anchor; BC-2.19.029 invariant 6 note; BC-2.19.030 invariant 6 note)

### AC-180-008: Emission is count-independent — one finding set per ASDU regardless of VSQ object count
- Given an I-format ASDU with TypeID in {58..=64} and `asdu.count == 0`
- When `detect_iec104_threats` processes the parsed `Asdu`
- Then the same finding(s) are still emitted as for `count > 0` — emission is per-ASDU,
  not per-object
(traces to BC-2.19.029 postcondition 5 and invariant 3; BC-2.19.030 postcondition 6 and invariant 3)

## Architecture Mapping

| Component | Module | File | Pure/Effectful |
|-----------|--------|------|---------------|
| `detect_iec104_threats` (new arms 58..=60, 61..=64) | SS-19 threat detection | `src/analyzer/iec104.rs` | Effectful (emits findings) |
| TypeID match arm 58..=60 | SS-19 dispatch table | `src/analyzer/iec104.rs` | Effectful |
| TypeID match arm 61..=64 | SS-19 dispatch table | `src/analyzer/iec104.rs` | Effectful |
| Silent-range comment update (lines 912–914) | SS-19 catch-all arm | `src/analyzer/iec104.rs` | N/A (comment only) |

Subsystem anchor: SS-19 owns this story's scope because TypeID-based threat detection is a
core behavioral detection capability of the IEC-104 passive analyzer per ARCH-INDEX.md
§SS-19. The timed-command detection arms belong to the same `detect_iec104_threats`
function as the existing untimed arms (STORY-170).

Dependency anchor: STORY-180 depends on STORY-174 because STORY-174 completed the VP-044/
VP-045/VP-046/VP-047 formal hardening of the IEC-104 foundation. The new detection arms
must be added to a formally-hardened codebase; the existing fuzz harness (VP-047) will
automatically cover the new TypeIDs once the arms are added.

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_asdu` (from STORY-169) | pure-core | Returns `Option<Asdu>` by value; no mutation, no findings |
| `Asdu` struct (from STORY-169) | pure-core | Plain data; no I/O |
| `detect_iec104_threats` arms 58..=60 and 61..=64 | effectful-shell | Consumes `&Asdu`, appends findings to `&mut Vec<Finding>` |
| Post-emission [TEST] loop (existing, lines 924–928) | effectful-shell | Iterates over findings added in the same call; no extra wiring for new arms |

## Tasks

- [ ] Add match arm `58..=60` in `detect_iec104_threats`, slotted ahead of the `_` catch-all
  (same slot order as ADR-013 Decision 3). Arm logic mirrors lines 748–774 (untimed 45..=47):
  - Build evidence `ev` with `CASDU=<value>` plus conditional `first_ioa=<ioa>`
  - Emit T1692.001 "Unauthorized Message: Command Message" Possible with timed-variant summary
  - Do NOT emit T0836 (switching commands are binary control, not parameter writes)
  - Verify no `unsafe` pointer-cast is needed (selection by TypeID, single arm)
- [ ] Add match arm `61..=64` in `detect_iec104_threats`, also ahead of `_`.
  Arm logic mirrors lines 781–830 (untimed 48..=51):
  - Build evidence vectors `ev1` and `ev2` each with `CASDU=<value>` plus conditional `first_ioa=<ioa>`
  - Emit T1692.001 Possible (command-message indicator) with timed-variant summary including
    "C_SE_TA/C_SE_TB/C_SE_TC/C_BO_TA" mnemonics
  - Emit T0836 "Modify Parameter" Possible (parameter/value write) with timed-variant summary
  - Order: T1692.001 arm push first, T0836 second (mirrors untimed arm ordering)
- [ ] Narrow the silent-range code comment at `detect_iec104_threats` lines 912–914:
  - Change `52–99` to `52–57` and `65–99`
  - Add note: "TypeIDs 58–64 were here prior to wave-85-spec-evolution; they are now
    handled by BC-2.19.029 (58–60) and BC-2.19.030 (61–64)."
- [ ] Write unit tests:
  - `test_BC_2_19_029_type_id_58_emits_t1692_001_only` (TypeID=58, no T0836)
  - `test_BC_2_19_029_type_id_59_emits_t1692_001_only` (TypeID=59)
  - `test_BC_2_19_029_type_id_60_emits_t1692_001_only` (TypeID=60)
  - `test_BC_2_19_029_casdu_first_ioa_evidence` (TypeID=58, CASDU=1, first_ioa=Some(100))
  - `test_BC_2_19_029_type_id_60_cot_test_suffix` (cot_test=true → ` [TEST]` in summary)
  - `test_BC_2_19_030_type_id_61_emits_two_findings` (T1692.001 + T0836 Possible)
  - `test_BC_2_19_030_type_id_62_emits_two_findings`
  - `test_BC_2_19_030_type_id_63_emits_two_findings`
  - `test_BC_2_19_030_type_id_64_emits_two_findings` (C_BO_TA_1 bitstring)
  - `test_BC_2_19_030_type_id_64_cot_test_both_findings_tagged` (both summaries get [TEST])
  - `test_BC_2_19_022_v1_1_type_id_52_no_finding` (regression: TypeID=52 silent)
  - `test_BC_2_19_022_v1_1_type_id_57_no_finding` (regression: TypeID=57 silent)
  - `test_BC_2_19_022_v1_1_type_id_65_no_finding` (regression: TypeID=65 silent)
  - `test_BC_2_19_022_v1_1_type_id_99_no_finding` (regression: TypeID=99 silent)
  - Update or remove any existing test that asserts zero findings for TypeIDs 58–64
    (these must now assert one or two findings per the new arms)
- [ ] Verify `cargo test --all-targets` passes with no regressions

## Edge Cases

| ID | Source BC | Description | Expected Behavior |
|----|-----------|-------------|-------------------|
| EC-001 | BC-2.19.029 | TypeID=58 (C_SC_TA_1, timed single command) | T1692.001 Possible only (no T0836) |
| EC-002 | BC-2.19.029 | TypeID=59 (C_DC_TA_1, timed double command) | T1692.001 Possible only |
| EC-003 | BC-2.19.029 | TypeID=60 (C_RC_TA_1, timed regulating step) | T1692.001 Possible only |
| EC-004 | BC-2.19.030 | TypeID=61 (C_SE_TA_1, timed set-point normalized) | T1692.001 Possible + T0836 Possible |
| EC-005 | BC-2.19.030 | TypeID=64 (C_BO_TA_1, timed bitstring of 32 bits) | T1692.001 Possible + T0836 Possible |
| EC-006 | BC-2.19.022 v1.1 | TypeID=57 (RESERVED, upper neighbor below 58..=60 arm) | No finding (silently logged — regression guard) |
| EC-007 | BC-2.19.022 v1.1 | TypeID=65 (unhandled, lower neighbor above 61..=64 arm) | No finding (silently logged — regression guard) |
| EC-008 | BC-2.19.029 | TypeID=58, first_ioa=None | T1692.001 Possible; no first_ioa entry in evidence |
| EC-009 | BC-2.19.029 | TypeID=60, cot_test=true | T1692.001 Possible; summary ends with " [TEST]" |
| EC-010 | BC-2.19.030 | TypeID=64, cot_test=true | T1692.001 Possible + T0836 Possible; BOTH summaries end with " [TEST]" |
| EC-011 | BC-2.19.029 | TypeID=58, asdu.count=0 | T1692.001 still emitted — count-independent (Invariant 3) |
| EC-012 | BC-2.19.019 | TypeID=45 (untimed twin, regression) | T1692.001 Possible only (BC-2.19.019 still handles; no regression) |
| EC-013 | BC-2.19.019 | TypeID=51 (untimed twin, regression) | T1692.001 Possible + T0836 Possible (BC-2.19.019 still handles) |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| BC-2.19.029 (~1,000), BC-2.19.030 (~1,000), BC-2.19.022 (~800) — 3 BC files | ~2,800 |
| iec104-timed-cmd-gap-validation.md (research report) | ~2,500 |
| ss-19-iec104-analysis.md (SS-19 architecture shard) | ~8,000 |
| docs/adr/0013-iec104-stream-dispatch-and-parser-design.md | ~12,000 |
| src/analyzer/iec104.rs (current, grown since STORY-174) | ~20,000 |
| tests/iec104_analyzer_tests.rs (test file delta) | ~2,500 |
| **TOTAL** | **~51,300** |

Agent context window ~200k tokens. This story uses ~26% — within the 20–30% guideline.

## Previous Story Intelligence

**Predecessor chain:** STORY-167 → STORY-168 → STORY-169 → STORY-170 → STORY-171 →
STORY-172 → STORY-173 → STORY-174 (formal hardening).

**Critical knowledge from STORY-170 (untimed detection, the direct parity reference):**

- The detection function is `detect_iec104_threats`. It accepts a `&Asdu` (from
  `parse_asdu`) and a `&mut Vec<Finding>`. The function is called from the effectful
  `on_data` shell AFTER `parse_asdu` returns `Some(asdu)`.
- The existing arms for the untimed equivalents are:
  - `45..=47` (lines 748–774): builds `ev` with CASDU + conditional first_ioa, pushes
    one T1692.001 Possible finding with summary naming "C_SC/C_DC/C_RC"
  - `48..=51` (lines 781–830): builds separate `ev1` and `ev2`, pushes T1692.001 Possible
    then T0836 Possible with summaries naming "C_SE/C_BO"
- The **post-emission [TEST] loop** at lines 924–928 iterates over ALL findings pushed
  during the call and appends ` [TEST]` when `asdu.cot_test == true`. The new arms need
  NO extra wiring for [TEST] tagging — the loop covers them automatically.
- Evidence parity: the new arms must use the same CASDU/first_ioa evidence shape. Do NOT
  invent a new evidence key format.
- The `_` catch-all arm is at lines 915–918 (approximately). The new arms slot BEFORE it.

**From STORY-174 (formal hardening):**
- VP-047 (`fuzz_iec104_parser`) is already wired and covers all TypeIDs including 58–64.
  The fuzz harness will automatically exercise the new arms once they are added — no
  additional fuzz target setup is needed.
- The Kani VP-044 skeleton is in place; no changes needed for this story.

**F3-DECOMPOSITION-BC-FIDELITY lesson (4th recurrence, STORY-170 v2.0):**
Read ALL three BC files before writing any implementation code. Summary line descriptions
of BCs cause drift. The precise postcondition wording (e.g., "CASDU + conditional
first_ioa" evidence shape) was only correct in v2.0 of STORY-170 after reading BC-2.19.019
verbatim. Read BC-2.19.029, BC-2.19.030, and BC-2.19.022 v1.1 verbatim before
implementation.

## Architecture Compliance Rules

From `docs/adr/0013-iec104-stream-dispatch-and-parser-design.md`:

- **ADR-013 Decision 3 (detection arm slot order):** New detection arms must be slotted in
  `detect_iec104_threats` in TypeID ascending order. Arms 58..=60 and 61..=64 slot between
  the existing 48..=51 arm and the 100/101/103 arm, ahead of the `_` catch-all.
- **ADR-013 Decision 8 (pure/effectful split):** `parse_asdu` is pure-core. All finding
  emission happens in the effectful `detect_iec104_threats` or `on_data` shell only.
  No findings may be emitted inside `parse_asdu` or any pure function.
- **ADR-013 Decision 7 (no external ICS libraries):** `iec60870-5`, `wireshark`, `lib60870`
  banned. The new arms are plain match arms with manual evidence construction — no external
  library calls.
- **BC-2.19.022 v1.1 (catch-all comment):** The `_` catch-all arm comment MUST be updated
  as part of this story (AC-180-007). Leaving the comment stale is a correctness violation
  because the comment is the machine-readable contract for the catch-all arm.

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| Rust stdlib | 1.91+ (2024 edition) | match arms, range patterns `58..=60`, `61..=64` |

No new crate dependencies. No Cargo.toml changes.

## File Structure Requirements

| File | Action | Contents |
|------|--------|---------|
| `src/analyzer/iec104.rs` | MODIFY | Add `58..=60 =>` arm (T1692.001 only) and `61..=64 =>` arm (T1692.001 + T0836) ahead of `_` catch-all; narrow silent-range comment (lines 912–914) to `{52–57, 65–99}` with BC-2.19.029/030 note |
| `tests/iec104_analyzer_tests.rs` | MODIFY | Add tests for BC-2.19.029 (AC-180-001/002), BC-2.19.030 (AC-180-003), regression guard (AC-180-006); update any existing test asserting zero findings for TypeIDs 58–64 |

## Forbidden Dependencies

- `iec60870-5`, `wireshark`, `lib60870`, `nom` (ICS parsing libs) — banned per ADR-013 Decision 7
- Finding emission inside `parse_asdu` (pure function) — banned per ADR-013 Decision 8
- Unsafe split-borrow for carry fields — the new detection arms read `asdu` only (immutable borrow); no unsafe pattern needed
