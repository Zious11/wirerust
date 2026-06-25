---
document_type: story
story_id: STORY-133
title: "MITRE ICS Technique Seeding: T0858/T0816/T1693.001/IcsExecution + VP-007 Atomic Update"
epic_id: E-20
wave: 59
points: 5
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-analyzer
github_issue: 316
subsystems: [SS-10, SS-17]
target_module: mitre
depends_on: [STORY-131]
behavioral_contracts: []
verification_properties:
  - VP-007
inputs:
  - .factory/specs/architecture/decisions/ADR-010-ethernet-ip-cip-stream-dispatch.md
  - .factory/phase-f2-spec-evolution/enip-architecture-delta.md
input-hash: "350dcf3"
---

# STORY-133: MITRE ICS Technique Seeding: T0858/T0816/T1693.001/IcsExecution + VP-007 Atomic Update

## Narrative

**As a** security analyst reviewing wirerust findings,
**I want** the MITRE ICS catalog in `src/mitre.rs` to include T0858 (Change Operating Mode,
IcsExecution), T0816 (Device Restart/Shutdown, IcsInhibitResponseFunction), and T1693.001
(staged/seeded only for v0.11.0) — plus the new `MitreTactic::IcsExecution` variant —
**so that** the atomic VP-007 obligation for ENIP techniques is satisfied and all downstream
detection BCs (BC-2.17.010/011/012/013/015) can emit findings with correctly populated
MITRE metadata.

## Behavioral Contracts

This story implements the VP-007 atomic obligation, not a single BC. The 6-part atomic burst
is derived from ADR-010 Decision 7 and the enip-architecture-delta §VP-007 section.

| Obligation Step | Description |
|----------------|-------------|
| VP-007 Step 1 | Add `T0858`, `T0816`, `T1693.001` arms to `technique_info()` |
| VP-007 Step 2 | Extend `SEEDED` array: 25 → 28 entries (add T0858, T0816, T1693.001) |
| VP-007 Step 3 | Update `SEEDED_TECHNIQUE_ID_COUNT`: 25 → 28 |
| VP-007 Step 4 | Extend `EMITTED_IDS` set: add T0858, T0816, T0846 (17 → 20); T1693.001 NOT added to EMITTED |
| VP-007 Step 5 | Add `MitreTactic::IcsExecution` variant (Display: "Execution (ICS)", tactic_id: "TA0104") |
| VP-007 Step 6 | Run `cargo test mitre` — all mitre consistency tests pass |

## Acceptance Criteria

### AC-133-001: `technique_info("T0858")` returns correct ICS technique metadata
**Traces to:** VP-007 Step 1 (T0858 arm)
- `technique_info("T0858")` returns `Some(TechniqueInfo { id: "T0858", name: "Change Operating Mode", tactic: MitreTactic::IcsExecution, description: "..." })`
- Tactic is `MitreTactic::IcsExecution` (new variant), NOT any existing variant
- Description references ODVA EtherNet/IP context (PLC operating mode change)
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_technique_info_t0858`

### AC-133-002: `technique_info("T0816")` returns correct ICS technique metadata
**Traces to:** VP-007 Step 1 (T0816 arm)
- `technique_info("T0816")` returns `Some(TechniqueInfo { id: "T0816", name: "Device Restart/Shutdown", tactic: MitreTactic::IcsInhibitResponseFunction, description: "..." })`
- Tactic is `MitreTactic::IcsInhibitResponseFunction` (existing variant)
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_technique_info_t0816`

### AC-133-003: `technique_info("T1693.001")` returns staged technique metadata
**Traces to:** VP-007 Step 1 (T1693.001 arm)
- `technique_info("T1693.001")` returns `Some(TechniqueInfo { id: "T1693.001", name: "Modify Firmware: System Firmware", tactic: MitreTactic::IcsInhibitResponseFunction, description: "..." })`
- T1693.001 is SEEDED (appears in `SEEDED` array) but NOT emitted — it will not appear in `EMITTED_IDS`
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_technique_info_t1693_001`

### AC-133-004: `MitreTactic::IcsExecution` variant exists with correct Display and tactic_id
**Traces to:** VP-007 Step 5
- `MitreTactic::IcsExecution` is a new variant in the `MitreTactic` enum in `src/mitre.rs`
- `MitreTactic::IcsExecution.to_string()` == `"Execution (ICS)"`
- `MitreTactic::IcsExecution.tactic_id()` == `"TA0104"`
- The variant appears in all match arms that enumerate MitreTactic (Display, tactic_id, any coverage tests)
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_ics_execution_tactic_display`
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_ics_execution_tactic_id`

### AC-133-005: `SEEDED` array grows from 25 to 28 entries
**Traces to:** VP-007 Step 2 and Step 3
- The `SEEDED` constant (or `static`) in `src/mitre.rs` contains exactly 28 technique IDs after this story
- The added IDs are `"T0858"`, `"T0816"`, `"T1693.001"` (in addition to the existing 25)
- `SEEDED_TECHNIQUE_ID_COUNT` constant equals 28
- **Baseline verification note (F3-307):** The pre-ENIP baseline of 25 seeded entries MUST be re-verified against `src/mitre.rs` HEAD at implementation time. STORY-129 (Wave 57) also modified the MITRE catalog — the F4 implementer MUST confirm the live pre-ENIP `SEEDED.len()` count before asserting post-ENIP == 28. If the pre-ENIP count differs from 25, adjust the post-ENIP target accordingly (pre + 3). Test `test_seeded_count_is_28` should be derived from the confirmed live count.
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_seeded_count_is_28`

### AC-133-006: `EMITTED_IDS` set grows from 17 to 20 entries; T1693.001 NOT in EMITTED
**Traces to:** VP-007 Step 4
- `EMITTED_IDS` contains T0858, T0816, T0846 as new additions (17 → 20 total)
- T1693.001 is NOT in `EMITTED_IDS` — it is staged-only for v0.11.0
- T0846 was previously in `SEEDED` but not `EMITTED`; this story promotes T0846 to emitted (first emission will be in STORY-134 BC-2.17.010)
- **Baseline verification note (F3-307):** The pre-ENIP baseline of 17 emitted entries MUST be re-verified against `src/mitre.rs` HEAD at implementation time. STORY-129 (Wave 57) also modified the MITRE catalog — the F4 implementer MUST confirm the live pre-ENIP `EMITTED_IDS.len()` count before asserting post-ENIP == 20. If the pre-ENIP count differs from 17, adjust the post-ENIP target accordingly (pre + 3). Tests `test_emitted_count_is_20` / `seeded==28` should be derived from confirmed live counts, not assumed from spec alone.
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_emitted_count_is_20`
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_t1693_001_not_emitted`
- **Test:** `tests/enip_analyzer_tests.rs::mitre_seeding::test_t0846_in_emitted`

### AC-133-007: All existing mitre consistency tests continue to pass
**Traces to:** VP-007 Step 6
- `cargo test mitre` exits zero after this story is applied
- No existing MITRE test is broken by the addition of new variants/entries
- The `technique_info` exhaustiveness test (if present) is updated to include T0858, T0816, T1693.001
- **Test:** all existing `mitre_*` tests in `tests/` (not newly added; just verified still green)

## Architecture Mapping

| Component | Location | Change |
|-----------|----------|--------|
| `MitreTactic` enum | `src/mitre.rs` | Add `IcsExecution` variant |
| `MitreTactic` Display impl | `src/mitre.rs` | Add arm: `IcsExecution => "Execution (ICS)"` |
| `MitreTactic::tactic_id()` | `src/mitre.rs` | Add arm: `IcsExecution => "TA0104"` |
| `technique_info()` | `src/mitre.rs` | Add arms: T0858, T0816, T1693.001 |
| `SEEDED` array | `src/mitre.rs` | Add: "T0858", "T0816", "T1693.001" (25→28) |
| `SEEDED_TECHNIQUE_ID_COUNT` | `src/mitre.rs` | Change: 25 → 28 |
| `EMITTED_IDS` | `src/mitre.rs` | Add: "T0858", "T0816", "T0846" (17→20) |
| Test mod | `tests/enip_analyzer_tests.rs` | `mod mitre_seeding { ... }` |

**Subsystem anchor justification:** SS-10 owns this story's scope because the primary deliverable is `src/mitre.rs` (MITRE Mapping subsystem = SS-10 per ARCH-INDEX.md Subsystem Registry). SS-17 is also listed because the new techniques (T0858, T0816) are EtherNet/IP-specific (SS-17) and the seeding is a prerequisite for SS-17 detection BCs in Wave 60.

**Why T0846 moves to EMITTED here:** T0846 (Remote System Discovery, ListIdentity) was seeded in a prior story but its first emission is in STORY-134 (BC-2.17.010). The VP-007 atomic requirement is that EMITTED_IDS is consistent with what the codebase actually emits. STORY-133 adds T0846 to EMITTED because STORY-134 (Wave 60) will emit it — the catalog update must precede the first emission. Since STORY-133 is Wave 59 and STORY-134 is Wave 60, this ordering is correct.

**T1693.001 staged-only rationale:** T1693.001 (Modify Firmware: System Firmware) is seeded for catalog completeness and future roadmap but no v0.11.0 detection BC emits it. It must NOT be added to `EMITTED_IDS` in this story or any v0.11.0 story — adding it to EMITTED without a corresponding emitter would cause the EMITTED/SEEDED consistency test to fail.

## VP-007 Atomic Update Details

ADR-010 Decision 7 specifies the VP-007 atomic obligation as a 6-part burst that must all land in one story. Splitting across stories would leave `EMITTED_IDS` inconsistent with `technique_info` arms.

**Step 1 — Add `technique_info` arms:**
```rust
"T0858" => Some(TechniqueInfo {
    id: "T0858",
    name: "Change Operating Mode",
    tactic: MitreTactic::IcsExecution,
    description: "Adversary commands EtherNet/IP device to change operating mode \
                   (e.g., Run→Program), disrupting or enabling unauthorized control.",
}),
"T0816" => Some(TechniqueInfo {
    id: "T0816",
    name: "Device Restart/Shutdown",
    tactic: MitreTactic::IcsInhibitResponseFunction,
    description: "Adversary uses CIP Reset service (0x05) to restart or shutdown \
                   a device via EtherNet/IP.",
}),
"T1693.001" => Some(TechniqueInfo {
    id: "T1693.001",
    name: "Modify Firmware: System Firmware",
    tactic: MitreTactic::IcsInhibitResponseFunction,
    description: "Staged/seeded for v0.12.0; no v0.11.0 emitter.",
}),
```

**Step 2 — Extend SEEDED:**
```rust
static SEEDED: &[&str] = &[
    // ... existing 25 entries ...
    "T0858", "T0816", "T1693.001",  // ENIP additions (v0.11.0)
];
```

**Step 3 — Update count:**
```rust
const SEEDED_TECHNIQUE_ID_COUNT: usize = 28;
```

**Step 4 — Extend EMITTED_IDS:**
```rust
static EMITTED_IDS: &[&str] = &[
    // ... existing 17 entries ...
    "T0858", "T0816", "T0846",  // ENIP additions (v0.11.0); T1693.001 NOT here
];
```

**Step 5 — Add MitreTactic variant:**
```rust
pub enum MitreTactic {
    // ... existing variants ...
    IcsExecution,  // TA0104
}
impl fmt::Display for MitreTactic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // ... existing arms ...
            MitreTactic::IcsExecution => write!(f, "Execution (ICS)"),
        }
    }
}
impl MitreTactic {
    pub fn tactic_id(&self) -> &'static str {
        match self {
            // ... existing arms ...
            MitreTactic::IcsExecution => "TA0104",
        }
    }
}
```

**Step 6 — Verify:**
```bash
cargo test mitre
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `technique_info("T0858")` | Returns Some with IcsExecution tactic |
| EC-002 | `technique_info("T0816")` | Returns Some with IcsInhibitResponseFunction |
| EC-003 | `technique_info("T1693.001")` | Returns Some (seeded catalog entry) |
| EC-004 | `technique_info("T0846")` | Still returns Some (existing entry, now also in EMITTED) |
| EC-005 | `MitreTactic::IcsExecution.to_string()` | "Execution (ICS)" |
| EC-006 | `SEEDED.len()` | 28 |
| EC-007 | `EMITTED_IDS.len()` | 20 |
| EC-008 | `EMITTED_IDS.contains("T1693.001")` | false |
| EC-009 | `EMITTED_IDS.contains("T0846")` | true |

## Tasks

- [ ] Add `IcsExecution` to `MitreTactic` enum in `src/mitre.rs`
- [ ] Add `IcsExecution => "Execution (ICS)"` arm to `MitreTactic` Display impl
- [ ] Add `IcsExecution => "TA0104"` arm to `MitreTactic::tactic_id()` method
- [ ] Add `"T0858"` arm to `technique_info()` match (tactic: IcsExecution)
- [ ] Add `"T0816"` arm to `technique_info()` match (tactic: IcsInhibitResponseFunction)
- [ ] Add `"T1693.001"` arm to `technique_info()` match (name: "Modify Firmware: System Firmware", tactic: IcsInhibitResponseFunction)
- [ ] Append `"T0858"`, `"T0816"`, `"T1693.001"` to `SEEDED` array
- [ ] Update `SEEDED_TECHNIQUE_ID_COUNT` to 28
- [ ] Append `"T0858"`, `"T0816"`, `"T0846"` to `EMITTED_IDS` (NOT T1693.001)
- [ ] Add `mod mitre_seeding { ... }` test wrapper to `tests/enip_analyzer_tests.rs`
- [ ] Run `cargo test mitre` — all mitre tests pass (existing + new)
- [ ] Run `cargo check` — zero errors (no match arm exhaustiveness warnings for new variant)
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings

## Test Plan

**Test file:** `tests/enip_analyzer_tests.rs`
**Test module:** `mod mitre_seeding { ... }`

```
mitre_seeding::test_technique_info_t0858
mitre_seeding::test_technique_info_t0816
mitre_seeding::test_technique_info_t1693_001
mitre_seeding::test_ics_execution_tactic_display
mitre_seeding::test_ics_execution_tactic_id
mitre_seeding::test_seeded_count_is_28
mitre_seeding::test_emitted_count_is_20
mitre_seeding::test_t1693_001_not_emitted
mitre_seeding::test_t0846_in_emitted
```

Plus all existing `cargo test mitre` tests (must remain green).

## Previous Story Intelligence

STORY-133 depends on STORY-131 (which wires the `EnipAnalyzer` to the dispatcher). The MITRE seeding itself is independent of the parse layer (STORY-130) and the CPF/CIP parse (STORY-132), but it must be complete before Wave 60 detection stories (STORY-134, STORY-135) can emit T0858/T0816/T0846 findings with valid MITRE metadata.

Reference: STORY-129 (issue #64, MITRE enrichment, Wave 57) for the pattern of adding technique_info arms and updating SEEDED/EMITTED arrays. The VP-007 atomic burst pattern is documented in STORY-110 for the DNP3 equivalent.

**Critical ordering rule:** All 6 VP-007 steps in this story MUST be in a single commit. Splitting the SEEDED update from the EMITTED update would leave the consistency invariant (`EMITTED ⊆ SEEDED`) temporarily broken. The implementer must stage all 6 changes together before `cargo test mitre`.

## Architecture Compliance Rules

From ADR-010 Decision 7 and VP-007:

1. **Atomicity (ADR-010 Decision 7):** All 6 steps of the VP-007 burst MUST be in one story and ideally one commit. If split across commits, `cargo test mitre` must pass after EACH commit (the intermediate state must not be inconsistent).
2. **T1693.001 is NOT emitted in v0.11.0:** Adding T1693.001 to `EMITTED_IDS` would cause `cargo test mitre` to fail (no emitter exists). It must appear only in `SEEDED` and `technique_info`.
3. **T0846 promotion:** T0846 moves from SEEDED-only to also EMITTED in this story. This is correct because STORY-134 (Wave 60) will be the first story to emit T0846. The catalog must list it as emitted before the emitter story is implemented, not after.
4. **`MitreTactic::IcsExecution` must appear in ALL match arms:** Rust's exhaustiveness checker will flag missing arms. The Display impl, `tactic_id()`, and any other `match self` on `MitreTactic` must all gain the new arm. `cargo check` will catch this.
5. **SEEDED_TECHNIQUE_ID_COUNT must equal SEEDED.len():** If there is a compile-time assert checking this (common pattern in MITRE consistency tests), update the constant. If the assert is a test, `cargo test mitre` will catch a mismatch.

## Library & Framework Requirements

No new crate dependencies. Only `src/mitre.rs` is modified (plus the new test block in `tests/enip_analyzer_tests.rs`).

## File Structure Requirements

**Files to modify:**
- `src/mitre.rs` — add IcsExecution variant, technique_info arms, SEEDED/EMITTED updates
- `tests/enip_analyzer_tests.rs` — add `mod mitre_seeding { ... }` block

**Files NOT touched:**
- `src/analyzer/enip.rs` — no detection logic in this story
- `src/dispatcher.rs` — already done by STORY-131
- `src/cli.rs` — already done by STORY-131

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/mitre.rs` changes (6 steps) | ~300 |
| `tests/enip_analyzer_tests.rs` mitre_seeding mod (9 tests) | ~300 |
| **Total** | **~600** |

## Dependency Rationale

STORY-133 is Wave 59 (depends on STORY-131 for `EnipAnalyzer` to exist). The MITRE seeding is a prerequisite for Wave 60 detection stories which need T0858/T0816/T0846 in `technique_info()` to pass the MITRE metadata validation (any existing `cargo test mitre` coverage test would fail if an emitted technique ID is not in the catalog).
