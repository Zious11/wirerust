---
document_type: story
story_id: STORY-142
title: "Fix DNP3 is_non_dnp3 Desync-Latch Direction-Contamination (RULING-DNP3-DESYNC-001)"
epic_id: E-15
wave: 64
points: 3
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-v0.11.0
github_issue: 316
subsystems: [SS-15]
target_module: analyzer/dnp3
depends_on: [STORY-140]
blocks: []
behavioral_contracts:
  - BC-2.15.009
verification_properties: []
assumption_validations: []
risk_mitigations: []
ruling: RULING-DNP3-DESYNC-001
inputs:
  - .factory/specs/behavioral-contracts/ss-15/BC-2.15.009.md
  - .factory/cycles/feature-enip-v0.11.0/RULING-DNP3-DESYNC-001-direction-latch.md
input-hash: "bfd7ab2"
---

# STORY-142: Fix DNP3 is_non_dnp3 Desync-Latch Direction-Contamination (RULING-DNP3-DESYNC-001)

## Narrative

**As a** security analyst running wirerust against bidirectional DNP3 captures,
**I want** the DNP3 analyzer's first-delivery desync check to correctly determine flow
establishment from BOTH directional carries — not just the current delivery's carry —
**so that** a junk server-to-client packet cannot permanently silence an already-established
client-to-server DNP3 detection stream (RULING-DNP3-DESYNC-001) — unblocking the v0.11.0 release.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.15.009 | v2.0 | is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync | Precondition 3 bail condition: complete predicate `frame_count == 0 && carry_c2s.is_empty() && carry_s2c.is_empty()` replaces `active_carry!(flow, direction).is_empty()` (ADDENDUM-2026-06-28; both-carries-empty-only form superseded); EC-010/EC-011/EC-012 direction-isolation desync scenarios added |

## Acceptance Criteria

### AC-142-001: `dnp3.rs:372` desync-latch condition widened to complete predicate with `frame_count==0` guard
**Traces to:** BC-2.15.009 v2.0 Precondition 3; RULING-DNP3-DESYNC-001 §2.1, ADDENDUM-2026-06-28

The desync-latch block at `dnp3.rs:372` (post-STORY-140 code) is changed from:

```rust
if active_carry!(flow, direction).is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

to the COMPLETE predicate (ADDENDUM-2026-06-28-frame-count-guard supersedes the
both-carries-empty-only form):

```rust
if flow.frame_count == 0
    && flow.carry_c2s.is_empty()
    && flow.carry_s2c.is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

**Complete predicate (canonical):**
`flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`

**What changed from the initial adjudication:** The initial ruling adopted only the
both-carries-empty condition (`carry_c2s.is_empty() && carry_s2c.is_empty()`). The
ADDENDUM-2026-06-28 supersedes that: carries are transiently drained to empty after every
complete frame, so the both-carries-empty-only form still fires on established flows at
the moment the carry is drained (sub-case ii — the common request→response lifecycle).
The `frame_count == 0` guard is the correct and complete fix. Once `frame_count >= 1`
the flow is unconditionally established and the latch must never fire.

**Correctness (RULING-DNP3-DESYNC-001 §2.3, ADDENDUM-2026-06-28):**
- Case 1 (genuine non-DNP3 flow, c2s first): `frame_count=0`, both carries empty, junk bytes → all four conditions true → condition fires → `is_non_dnp3 = true`. Correct.
- Case 2 sub-case i (established c2s, junk s2c WHILE `carry_c2s` non-empty): `carry_c2s.is_empty() = false` → condition does NOT fire → `is_non_dnp3` remains false. Correct — established c2s stream preserved.
- Case 2 sub-case ii (established c2s, junk s2c AFTER carry_c2s drained to empty post-complete-frame): `frame_count >= 1` → `frame_count == 0` is FALSE → condition does NOT fire. Correct — this is the sub-case the both-carries-empty-only form missed.

**Test:** `tests/dnp3_detection_tests.rs::desync_latch::test_ac142_001_one_line_condition_change`
— Structural/compilation assertion that `active_carry!(flow, direction)` does NOT appear in the
desync-latch condition path, and that the complete predicate (`frame_count == 0 &&
carry_c2s.is_empty() && carry_s2c.is_empty()`) does. The direct behavioral test of sub-case i
is covered by AC-142-002; sub-case ii by AC-142-004. (traces to BC-2.15.009 v2.0 Precondition 3)

### AC-142-002: New regression test — established c2s direction preserved on junk s2c delivery
**Traces to:** BC-2.15.009 v2.0 Precondition 3, EC-010; RULING-DNP3-DESYNC-001 §4 AC-2

This test is **RED against the buggy post-STORY-140 code** (where `active_carry!(flow, direction).is_empty()` causes `carry_s2c.is_empty()` to fire and latch `is_non_dnp3 = true`), and **GREEN after the fix**.

Steps:
1. Deliver partial c2s DNP3 frame: valid sync bytes `[0x05, 0x64, 0x0A, 0xC4, 0x01, 0x00]` (6 bytes — incomplete
   header; full DNP3 link header is 10 bytes), `direction=ClientToServer`. Assert: `carry_c2s.len() > 0`
   (partial frame stashed). `is_non_dnp3 == false` (sync word matched).
2. Deliver non-DNP3 junk bytes `[0xFF, 0xFE, 0x00]` in `direction=ServerToClient`. Assert:
   `flow.is_non_dnp3 == false` (latch did NOT fire because `carry_c2s.is_non_empty()`).
3. Complete the c2s partial frame with the remaining 4 bytes of the link header + appropriate body bytes
   in `direction=ClientToServer`. Assert: `frame_count >= 1`, `parse_errors == 0` (c2s stream was NOT
   silenced; processing continues after the junk s2c packet).

**Test:** `tests/dnp3_detection_tests.rs::desync_latch::test_ac142_002_regression_established_c2s_preserved_on_junk_s2c`
(traces to BC-2.15.009 v2.0 Precondition 3, EC-010)

### AC-142-003: Existing true-non-DNP3 latch test still passes (no regression from fix)
**Traces to:** BC-2.15.009 v2.0 Precondition 3, EC-011; RULING-DNP3-DESYNC-001 §4 AC-3 and AC-5

The fix does NOT regress the case where a flow is genuinely non-DNP3 (both carries empty,
`frame_count=0`, junk first delivery).

Test scenario:
1. First delivery in c2s direction: non-DNP3 junk `[0xDE, 0xAD, 0xBE, 0xEF]` (`frame_count=0`,
   both `carry_c2s` and `carry_s2c` empty). Assert: `flow.is_non_dnp3 == true` (`frame_count==0`,
   both-carries-empty, no sync word → all conditions true → latch fires).
2. Subsequent `on_data` call with valid DNP3 sync bytes: returns immediately at the
   `if flow.is_non_dnp3 { return; }` bail. Assert: `frame_count == 0` (flow permanently silenced).

Additionally, all existing DNP3 tests pass — `cargo test --all-targets` green.

**Test:** `tests/dnp3_detection_tests.rs::desync_latch::test_ac142_003_true_non_dnp3_still_latches`
(traces to BC-2.15.009 v2.0 Precondition 3, EC-011)

### AC-142-004: Sub-case ii — complete c2s frame drained → junk s2c → latch does NOT fire (requires `frame_count==0` guard)
**Traces to:** BC-2.15.009 v2.0 Precondition 3, EC-012; RULING-DNP3-DESYNC-001 ADDENDUM-2026-06-28 §4 AC-5

This test is **RED against the both-carries-empty-only fix** (without `frame_count == 0`) and
**GREEN only with the complete predicate** (`frame_count == 0 && carry_c2s.is_empty() && carry_s2c.is_empty()`).
It covers the sub-case ii failure: the common request→response lifecycle where a complete c2s frame
drains `carry_c2s` to empty before a junk s2c delivery arrives.

Steps:
1. Deliver a complete c2s DNP3 frame (valid sync bytes, full link-layer frame). After parse:
   `carry_c2s` is drained to empty, `frame_count == 1`. Assert: `frame_count == 1`, `is_non_dnp3 == false`.
2. Deliver non-DNP3 junk bytes `[0xFF, 0xFE, 0x00]` in `direction=ServerToClient`.
   At latch check: `frame_count=1`, `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`.
   Assert: `flow.is_non_dnp3 == false` (latch does NOT fire because `frame_count == 1 >= 1` → `frame_count == 0` is FALSE).
   Assert: `frame_count == 1` (unchanged — junk s2c delivery did not decrement it).

**Why this test matters:** Without the `frame_count == 0` guard, the both-carries-empty
condition fires here (both carries are genuinely empty after the complete frame was consumed),
latching `is_non_dnp3 = true` and permanently silencing the established c2s direction. This is
the COMMON pattern in real DNP3 request→response traffic. See RULING-DNP3-DESYNC-001 §2.3
sub-case ii and ADDENDUM-2026-06-28.

**Test:** `tests/dnp3_detection_tests.rs::desync_latch::test_ac142_004_established_c2s_preserved_on_junk_s2c_after_complete_frame`
(traces to BC-2.15.009 v2.0 Precondition 3, EC-012)

## Architecture Mapping

| Component | Location | Role | Pure/Effectful |
|-----------|----------|------|----------------|
| `is_non_dnp3` desync-latch condition | `src/analyzer/dnp3.rs:372` (post-STORY-140) | Predicate change: `active_carry!(flow, direction).is_empty()` → complete predicate `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()` (ADDENDUM-2026-06-28; both-carries-empty-only form SUPERSEDED) | Effectful (mutates `is_non_dnp3`) |
| `tests/dnp3_detection_tests.rs` | `tests/dnp3_detection_tests.rs` | `mod desync_latch { ... }` with 4 named tests | Test |

**Subsystem anchor:** SS-15 owns this story's scope because the desync-latch condition is inside
`src/analyzer/dnp3.rs`, the DNP3 analyzer. Per ARCH-INDEX SS-15. The change is one line in one file.

**Dependency anchor:** STORY-142 depends on STORY-140 because the `carry_c2s`/`carry_s2c` fields and
`active_carry!` macro that are referenced in the current desync-latch condition (and in the fix) are
introduced by STORY-140. The `active_carry!(flow, direction).is_empty()` condition to be replaced
only exists after STORY-140 is merged. STORY-142 cannot be compiled until STORY-140 is on `develop`.

**Forbidden dependencies:** `src/analyzer/dnp3.rs` MUST NOT depend on any other analyzer module.

**NOT in scope:**
- `src/analyzer/modbus.rs` — see STORY-141
- `src/analyzer/enip.rs` — see STORY-139
- Per-direction `is_non_dnp3` flags (Option B per RULING-DNP3-DESYNC-001 §2.2 — rejected; Option A is correct)
- `first_c2s_frame_seen`/`first_s2c_frame_seen` tracking flags (deferred to v0.12.0 per §2.3 enhancement path)

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Partial c2s sync bytes buffered → `carry_c2s` non-empty → junk s2c delivery | `carry_c2s.is_empty() = false` → bail condition does NOT fire → `is_non_dnp3` remains false → c2s stream continues (BC-2.15.009 v2.0 EC-010) |
| EC-002 | Both carries empty, `frame_count=0`, first delivery is junk c2s (genuine non-DNP3 flow) | `frame_count==0`, `carry_c2s.is_empty()=true`, `carry_s2c.is_empty()=true`, junk bytes → all four conditions true → bail fires → `is_non_dnp3 = true` (BC-2.15.009 v2.0 Precondition 3, EC-011) |
| EC-003 | Complete c2s frame consumed → `carry_c2s` drained to empty, `frame_count=1` → junk s2c delivery | `frame_count==1` → `frame_count == 0` is FALSE → bail does NOT fire → `is_non_dnp3` remains false → c2s stream continues. Sub-case ii — this is the case the both-carries-empty-only form (without `frame_count==0`) MISSED (BC-2.15.009 v2.0 EC-012, ADDENDUM-2026-06-28) |

## Tasks

- [ ] In `src/analyzer/dnp3.rs`, find the desync-latch block at line 372 (post-STORY-140)
- [ ] Replace the entire condition with the COMPLETE predicate per ADDENDUM-2026-06-28:
      `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`
      (the both-carries-empty-only form WITHOUT `frame_count == 0` is SUPERSEDED — do NOT use it)
- [ ] No other changes to `on_data` or `Dnp3FlowState` fields
- [ ] Add `mod desync_latch` to `tests/dnp3_detection_tests.rs` with 4 tests:
  - [ ] `test_ac142_001_one_line_condition_change`
  - [ ] `test_ac142_002_regression_established_c2s_preserved_on_junk_s2c` (RED → GREEN)
  - [ ] `test_ac142_003_true_non_dnp3_still_latches`
  - [ ] `test_ac142_004_established_c2s_preserved_on_junk_s2c_after_complete_frame` (RED under both-carries-empty-only; GREEN under complete predicate)
- [ ] Run `cargo test dnp3` — all desync_latch tests pass
- [ ] Run `cargo test --all-targets` — full test suite green (no regressions)
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] Run `cargo fmt --check` — zero format drift
- [ ] Run `bin/compute-input-hash --write .factory/stories/STORY-142.md` to populate `input-hash`

## Test Plan

**Test file:** `tests/dnp3_detection_tests.rs`

**New test module added by this story:**

```
mod desync_latch {
    test_ac142_001_one_line_condition_change                                        // compilation/structural guard
    test_ac142_002_regression_established_c2s_preserved_on_junk_s2c               // sub-case i: partial-carry guard; RED before fix, GREEN after
    test_ac142_003_true_non_dnp3_still_latches                                     // true non-DNP3 regression guard
    test_ac142_004_established_c2s_preserved_on_junk_s2c_after_complete_frame     // sub-case ii: frame_count guard; RED under both-carries-empty-only, GREEN under complete predicate
}
```

**Test numbering convention (implemented):**
- 001: Structural/compilation assertion — `active_carry!` absent, complete predicate present
- 002: Sub-case i behavioral test — partial c2s carry in flight → junk s2c → latch does NOT fire
- 003: Regression guard — genuine non-DNP3 flow still latches immediately
- 004: Sub-case ii behavioral test — complete c2s frame consumed (carry drained) → junk s2c → latch does NOT fire (requires `frame_count == 0` guard; RED under both-carries-empty-only)

**TDD discipline (strict mode):** Write `test_ac142_002` and `test_ac142_004` FIRST against the
pre-fix (buggy) code. Both must be RED. `test_ac142_004` is additionally RED against the
intermediate both-carries-empty-only form — only the complete predicate with `frame_count == 0`
makes it GREEN. `test_ac142_003` must be GREEN both before and after the fix (regression guard).

`test_ac142_001` encodes the structural invariant: after the fix, `active_carry!(flow, direction)`
must NOT appear in the desync-latch condition path.

## Previous Story Intelligence

- STORY-107 introduced `Dnp3FlowState.carry: Vec<u8>` and the `is_non_dnp3` desync-latch check.
- STORY-140 (wave 63) split `carry: Vec<u8>` into `carry_c2s`/`carry_s2c` and introduced the
  `active_carry!(flow, direction)` macro. It is after this change that the desync-latch condition
  `active_carry!(flow, direction).is_empty()` became a defect — the single-carry check that correctly
  proxied "unestablished flow" now only checks the CURRENT DIRECTION's carry, which can be empty even
  when the other direction has established bytes buffered.
- DESIGN-CROSS-DIRECTION-STATE.md §2 analyzed this: the carry-direction split changes the semantics of
  `carry.is_empty()` from "no bytes accepted from any direction" to "no bytes accepted from this direction."
  The initial adjudication (RULING-DNP3-DESYNC-001 pre-ADDENDUM) widened to both-carries-empty, but the
  ADDENDUM-2026-06-28 revealed that carries are transiently drained after complete frames, making the
  both-carries-empty-only condition still insufficient. The complete fix adds `frame_count == 0` as the
  definitive "flow is genuinely unestablished" proxy.
- RULING-DNP3-DESYNC-001 §2.2 explicitly rejected Option B (per-direction `is_non_dnp3` flags) as more
  complex and semantically wrong. Option A (both-carries-empty predicate) is the correct one-line fix.
- STORY-142 is a follow-on to STORY-140; it cannot be written or compiled until STORY-140 is merged to
  `develop`. The `active_carry!` macro and `carry_c2s`/`carry_s2c` fields only exist after STORY-140.

## Architecture Compliance Rules

From BC-2.15.009 v2.0 and RULING-DNP3-DESYNC-001 §2 (as amended by ADDENDUM-2026-06-28):

1. **Complete predicate required — both-carries-empty-only form is SUPERSEDED (BC-2.15.009 v2.0 Precondition 3, ADDENDUM-2026-06-28):**
   The final predicate is `flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`.
   The `active_carry!(flow, direction).is_empty()` single-direction check is removed entirely.
   The intermediate both-carries-empty-only form (`carry_c2s.is_empty() && carry_s2c.is_empty()` without `frame_count == 0`) was the initial adjudication but is INCOMPLETE and MUST NOT be used — it still fires on established flows after a complete frame drains the carry buffer (sub-case ii).
2. **`frame_count == 0` is load-bearing (RULING-DNP3-DESYNC-001 ADDENDUM-2026-06-28):**
   `frame_count` is incremented on every successful complete-frame parse in any direction. Once
   `frame_count >= 1` the flow is unconditionally established and the latch MUST NOT fire regardless
   of carry state. This guard is not redundant with the carries-empty check.
3. **No new fields added to `Dnp3FlowState` (RULING-DNP3-DESYNC-001 §2.2 Option A rationale):**
   This is a predicate change only. No `is_non_dnp3_c2s`/`is_non_dnp3_s2c` split. `frame_count`
   already exists on `Dnp3FlowState` after STORY-140.
4. **`is_non_dnp3` remains PER-FLOW (RULING-DNP3-DESYNC-001 §2.2):** "If the flow is not DNP3, both
   directions are not DNP3." The shared latch is correct; the fix is to the bail CONDITION, not the latch.
5. **ENIP and Modbus code are OUT OF SCOPE:** Do NOT touch `src/analyzer/enip.rs` or `src/analyzer/modbus.rs`.
6. **`first_c2s_frame_seen` / `first_s2c_frame_seen` tracking flags are NO LONGER NEEDED (superseded):**
   RULING-DNP3-DESYNC-001 §2.3 noted these as an enhancement path. The `frame_count == 0` guard
   (ADDENDUM-2026-06-28) makes that approach unnecessary — `frame_count` already provides the precise
   "flow is genuinely unestablished" proxy. These flags are removed from the enhancement backlog.

## Library & Framework Requirements

- No new `Cargo.toml` dependencies
- All code changes are in `src/analyzer/dnp3.rs` (existing file, no new imports)
- `proptest` is NOT required for this story (no VP is recommended per RULING-DNP3-DESYNC-001 §5;
  the fix is a targeted regression test, not a new property class)

## File Structure Requirements

**Files to modify:**

- `src/analyzer/dnp3.rs`
  - Change the desync-latch condition at line 372 (post-STORY-140) to the COMPLETE predicate:
    `if flow.frame_count == 0 && flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty() && data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`
    (ADDENDUM-2026-06-28; both-carries-empty-only WITHOUT `frame_count == 0` is SUPERSEDED)
- `tests/dnp3_detection_tests.rs`
  - Add `mod desync_latch { ... }` with 4 named tests (001–004)

**Files NOT to modify:**
- `src/analyzer/modbus.rs`, `src/analyzer/enip.rs` — out of scope
- `src/dispatcher.rs` — no call-site change needed
- Any other DNP3 test file — no existing call-site updates needed (this story only adds new tests)

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/dnp3.rs` change (complete predicate) | ~60 |
| New `mod desync_latch` tests (4 tests) | ~400 |
| BC-2.15.009.md context | ~300 |
| RULING-DNP3-DESYNC-001 (ruling context, including ADDENDUM) | ~500 |
| **Total** | **~1,260** |

Context utilization: ~1,260 tokens / ~200,000 token window = ~0.6%. This is a small, surgical story.

## Dependency Rationale

Wave 64 (same wave as STORY-141; independent parallel story).

**STORY-142 depends on STORY-140** because: the bug this story fixes only exists in the post-STORY-140
codebase. The `active_carry!(flow, direction).is_empty()` condition at `dnp3.rs:372` and the
`carry_c2s`/`carry_s2c` fields and `frame_count` field that the complete predicate references are
introduced by STORY-140. STORY-142 cannot be compiled until STORY-140 is on `develop`.

**STORY-142 runs in parallel with STORY-141** (Wave 64): STORY-141 touches `src/analyzer/modbus.rs`;
STORY-142 touches `src/analyzer/dnp3.rs`. There is no file overlap.
