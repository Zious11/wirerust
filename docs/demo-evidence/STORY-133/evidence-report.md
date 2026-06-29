# Demo Evidence Report — STORY-133

**Story:** MITRE ICS Technique Seeding: T0858/T0816/T1693.001/IcsExecution + VP-007 Atomic Update
**Story ID:** STORY-133
**Wave:** 59
**Product type:** Pure-core library (no CLI/UI surface — MITRE catalog data functions only)
**Recording tool:** VHS 0.11.0 (terminal recordings of `cargo test --test enip_analyzer_tests`)
**Recorded:** 2026-06-25
**Test result at recording time:** 10 passed / 0 failed / 0 ignored (mod mitre_seeding)

---

## AC Coverage Map

| AC | Title | Test filter used | Artifact (GIF) | Artifact (WebM) | Tape |
|----|-------|-----------------|---------------|----------------|------|
| AC-133-001 | `technique_info("T0858")` returns correct ICS technique metadata | `mitre_seeding::test_technique_info_t0858` | `AC-001-002-003-technique-info.gif` | `AC-001-002-003-technique-info.webm` | `AC-001-002-003-technique-info.tape` |
| AC-133-002 | `technique_info("T0816")` returns correct ICS technique metadata | `mitre_seeding::test_technique_info_t0816` | `AC-001-002-003-technique-info.gif` | `AC-001-002-003-technique-info.webm` | `AC-001-002-003-technique-info.tape` |
| AC-133-003 | `technique_info("T1693.001")` returns staged technique metadata | `mitre_seeding::test_technique_info_t1693_001` | `AC-001-002-003-technique-info.gif` | `AC-001-002-003-technique-info.webm` | `AC-001-002-003-technique-info.tape` |
| AC-133-004 | `MitreTactic::IcsExecution` variant exists with correct Display and tactic_id | `mitre_seeding::test_ics_execution_tactic_display`, `test_ics_execution_tactic_id` | `AC-004-ics-execution-tactic.gif` | `AC-004-ics-execution-tactic.webm` | `AC-004-ics-execution-tactic.tape` |
| AC-133-005 | `SEEDED` array grows from 25 to 28 entries | `mitre_seeding::test_seeded_count_is_28` | `AC-005-006-007-counts-regression.gif` | `AC-005-006-007-counts-regression.webm` | `AC-005-006-007-counts-regression.tape` |
| AC-133-006 | `EMITTED_IDS` set grows from 17 to 20; T1693.001 NOT in EMITTED | `mitre_seeding::test_emitted_count_is_20`, `test_t1693_001_not_emitted`, `test_t0846_in_emitted` | `AC-005-006-007-counts-regression.gif` | `AC-005-006-007-counts-regression.webm` | `AC-005-006-007-counts-regression.tape` |
| AC-133-007 | All existing mitre consistency tests continue to pass | `cargo test mitre` (full regression) | `AC-005-006-007-counts-regression.gif` | `AC-005-006-007-counts-regression.webm` | `AC-005-006-007-counts-regression.tape` |

---

## Recordings Detail

### AC-001-002-003-technique-info

Demonstrates `technique_info()` arms for T0858, T0816, and T1693.001 added in this story
(VP-007 Step 1). All three return `Some(TechniqueInfo { ... })` with correct name, tactic,
and description. T1693.001 is SEEDED but its absence from EMITTED_IDS is verified separately
in the AC-005/006 recording.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests mitre_seeding::test_technique_info` with grep filter
- `test_technique_info_t0858`: id="T0858", name="Change Operating Mode",
  tactic=MitreTactic::IcsExecution, description references ODVA EtherNet/IP PLC operating mode
- `test_technique_info_t0816`: id="T0816", name="Device Restart/Shutdown",
  tactic=MitreTactic::IcsInhibitResponseFunction, description references CIP Reset service (0x05)
- `test_technique_info_t1693_001`: id="T1693.001", name="Modify Firmware: System Firmware",
  tactic=MitreTactic::IcsInhibitResponseFunction, description marks staged/seeded for v0.12.0
- All 3 technique_info tests pass green

**Tests in recording:**
- `test_technique_info_t0858`
- `test_technique_info_t0816`
- `test_technique_info_t1693_001`

---

### AC-004-ics-execution-tactic

Demonstrates the new `MitreTactic::IcsExecution` enum variant (VP-007 Step 5).
Rust's exhaustiveness checker guarantees the variant appears in all match arms
(`Display`, `tactic_id()`); `cargo check` would fail at compile time if any arm were missing.

**What the recording shows:**
- Runs `cargo test --test enip_analyzer_tests mitre_seeding::test_ics_execution` with grep filter
- `test_ics_execution_tactic_display`: `MitreTactic::IcsExecution.to_string()` == `"Execution (ICS)"`
- `test_ics_execution_tactic_id`: `MitreTactic::IcsExecution.tactic_id()` == `"TA0104"`
- Both tests pass green

**Tests in recording:**
- `test_ics_execution_tactic_display`
- `test_ics_execution_tactic_id`

---

### AC-005-006-007-counts-regression

Demonstrates catalog count invariants (VP-007 Steps 2–4) and full mitre regression
(VP-007 Step 6). Shows SEEDED=28, EMITTED=20, T1693.001 staged-only, T0846 promoted to
emitted, then the full `cargo test mitre` suite passing with no regressions.

**What the recording shows:**
- First run: filters for `test_seeded`, `test_emitted`, `test_t1693`, `test_t0846` tests
  - `test_seeded_count_is_28`: `SEEDED.len()` == 28 and `SEEDED_TECHNIQUE_ID_COUNT` == 28
  - `test_emitted_count_is_20`: `EMITTED_IDS.len()` == 20
  - `test_t1693_001_not_emitted`: `EMITTED_IDS.contains("T1693.001")` == false
  - `test_t0846_in_emitted`: `EMITTED_IDS.contains("T0846")` == true
  - All 4 tests pass green
- Second run: `cargo test mitre` — full regression suite across all mitre_* tests
  - All mitre consistency tests pass; 0 regressions

**Tests in recording (counts):**
- `test_seeded_count_is_28`
- `test_emitted_count_is_20`
- `test_t1693_001_not_emitted`
- `test_t0846_in_emitted`

**Full mitre regression (all mitre_* tests):**
- Includes `test_t0858_t0816_and_t0846_tactic_id_resolution` (authoritative TA-id pin table)
- Includes all prior mitre consistency tests unchanged

---

## VP-007 Kani Drift-Guard Note

AC-133 VP-007 Kani harnesses run at Phase F6 via `cargo kani`, not `cargo test`.
They are not exercised by these recordings. The unit test suite (10/10 green) satisfies
the Phase F3/F4 acceptance bar. Kani verification is tracked separately per STORY-133
and the VP-007 drift-guard obligation in `.factory/STATE.md`.

---

## Artifacts

```
docs/demo-evidence/STORY-133/
  AC-001-002-003-technique-info.tape              (VHS script)
  AC-001-002-003-technique-info.gif               (112 KB)
  AC-001-002-003-technique-info.webm              ( 92 KB)
  AC-004-ics-execution-tactic.tape                (VHS script)
  AC-004-ics-execution-tactic.gif                 (105 KB)
  AC-004-ics-execution-tactic.webm                ( 89 KB)
  AC-005-006-007-counts-regression.tape           (VHS script)
  AC-005-006-007-counts-regression.gif            (2.1 MB)
  AC-005-006-007-counts-regression.webm           (828 KB)
  evidence-report.md                              (this file)
```

## Coverage Summary

| AC | Evidenced | Path(s) demonstrated |
|----|-----------|----------------------|
| AC-133-001 | Yes | T0858 → name "Change Operating Mode", tactic IcsExecution, ODVA description |
| AC-133-002 | Yes | T0816 → name "Device Restart/Shutdown", tactic IcsInhibitResponseFunction, CIP Reset description |
| AC-133-003 | Yes | T1693.001 → name "Modify Firmware: System Firmware", tactic IcsInhibitResponseFunction; staged-only |
| AC-133-004 | Yes | IcsExecution Display "Execution (ICS)"; tactic_id "TA0104" |
| AC-133-005 | Yes | SEEDED.len()==28, SEEDED_TECHNIQUE_ID_COUNT==28; T0858/T0816/T1693.001 added |
| AC-133-006 | Yes | EMITTED_IDS.len()==20; T1693.001 not in EMITTED; T0846 in EMITTED |
| AC-133-007 | Yes | Full `cargo test mitre` suite — 0 regressions |
