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
input-hash: "99e4a9b"
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
| BC-2.15.009 | v2.0 | is_non_dnp3 Desync-Safe Bail — Flow Silenced on Initial-Delivery No-Sync | Precondition 3 bail condition widened: `carry_c2s.is_empty() && carry_s2c.is_empty()` replaces `active_carry!(flow, direction).is_empty()`; EC-010/EC-011 direction-isolation desync scenarios added |

## Acceptance Criteria

### AC-142-001: `dnp3.rs:363` desync-latch condition widened to both-carries-empty
**Traces to:** BC-2.15.009 v2.0 Precondition 3; RULING-DNP3-DESYNC-001 §2.1

The desync-latch block at `dnp3.rs:363` (post-STORY-140 code) is changed from:

```rust
if active_carry!(flow, direction).is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

to:

```rust
if flow.carry_c2s.is_empty()
    && flow.carry_s2c.is_empty()
    && data.len() >= 2
    && (data[0] != 0x05 || data[1] != 0x64)
{
    flow.is_non_dnp3 = true;
    return;
}
```

**Exact change:** Replace `active_carry!(flow, direction).is_empty()` with
`flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`. The rest of the bail body
(`flow.is_non_dnp3 = true; return;`) and the outer condition (`data.len() >= 2 && (data[0] != 0x05 || data[1] != 0x64)`)
are UNCHANGED. No other logic changes.

This is a one-line predicate change. No new fields, no structural changes.

**Correctness (RULING-DNP3-DESYNC-001 §2.3):**
- Case 1 (genuine non-DNP3 flow, c2s first): both carries empty → condition fires → `is_non_dnp3 = true`. Correct.
- Case 2 (established c2s, junk s2c WHILE `carry_c2s` non-empty): `carry_c2s.is_empty() = false` → BOTH check fails → condition does NOT fire → `is_non_dnp3` remains false. Correct — established c2s stream preserved.
- Case 2 (established c2s, junk s2c WHILE `carry_c2s` transiently empty between clean frames): both carries empty → condition fires. This matches pre-STORY-140 single-carry semantics (accepted residual limitation per §2.3).

**Test:** `tests/dnp3_detection_tests.rs::desync_latch::test_ac142_001_one_line_condition_change`
— Structural assertion that `active_carry!(flow, direction)` does NOT appear in the desync-latch
condition path; and that `carry_c2s.is_empty() && carry_s2c.is_empty()` does. Implemented as a
compilation test (the new condition compiles; a test that directly exercises the condition path
is covered by AC-142-002). (traces to BC-2.15.009 v2.0 Precondition 3)

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

The fix does NOT regress the case where a flow is genuinely non-DNP3 (both carries empty, junk first delivery).

Test scenario:
1. First delivery in c2s direction: non-DNP3 junk `[0xDE, 0xAD, 0xBE, 0xEF]` (both `carry_c2s` and
   `carry_s2c` empty). Assert: `flow.is_non_dnp3 == true` (both-carries-empty AND no sync word → latch fires).
2. Subsequent `on_data` call with valid DNP3 sync bytes: returns immediately at the
   `if flow.is_non_dnp3 { return; }` bail. Assert: `frame_count == 0` (flow permanently silenced).

Additionally, all existing DNP3 tests pass — `cargo test --all-targets` green.

**Test:** `tests/dnp3_detection_tests.rs::desync_latch::test_ac142_003_true_non_dnp3_still_latches`
(traces to BC-2.15.009 v2.0 Precondition 3, EC-011)

## Architecture Mapping

| Component | Location | Role | Pure/Effectful |
|-----------|----------|------|----------------|
| `is_non_dnp3` desync-latch condition | `src/analyzer/dnp3.rs:363` (post-STORY-140) | One-line predicate change: `active_carry!(flow, direction).is_empty()` → `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()` | Effectful (mutates `is_non_dnp3`) |
| `tests/dnp3_detection_tests.rs` | `tests/dnp3_detection_tests.rs` | `mod desync_latch { ... }` with 3 named tests | Test |

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
| EC-002 | Both carries empty, first delivery is junk c2s (genuine non-DNP3 flow) | `carry_c2s.is_empty() = true`, `carry_s2c.is_empty() = true`, no sync word → bail fires → `is_non_dnp3 = true` (BC-2.15.009 v2.0 Precondition 3) |
| EC-003 | `carry_c2s` drained to empty after complete c2s frame, then junk s2c | Both empty → bail fires (matches pre-STORY-140 single-carry behavior — accepted residual limitation per RULING-DNP3-DESYNC-001 §2.3) |

## Tasks

- [ ] In `src/analyzer/dnp3.rs`, find the desync-latch block at line 363 (post-STORY-140)
- [ ] Change `active_carry!(flow, direction).is_empty()` to `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`
- [ ] No other changes to `on_data` or `Dnp3FlowState` fields
- [ ] Add `mod desync_latch` to `tests/dnp3_detection_tests.rs` with 3 tests:
  - [ ] `test_ac142_001_one_line_condition_change`
  - [ ] `test_ac142_002_regression_established_c2s_preserved_on_junk_s2c` (RED → GREEN)
  - [ ] `test_ac142_003_true_non_dnp3_still_latches`
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
    test_ac142_001_one_line_condition_change
    test_ac142_002_regression_established_c2s_preserved_on_junk_s2c  // RED before fix, GREEN after
    test_ac142_003_true_non_dnp3_still_latches
}
```

**TDD discipline (strict mode):** Write `test_ac142_002_regression_established_c2s_preserved_on_junk_s2c`
FIRST against the pre-fix code. It must be RED (the test asserts `is_non_dnp3 == false` after the junk s2c
delivery, but the bug makes it true). Then apply the one-line fix. The test turns GREEN. This is the
canonical Red → Green cycle for this story.

`test_ac142_003_true_non_dnp3_still_latches` must be GREEN both before and after the fix (regression guard).

## Previous Story Intelligence

- STORY-107 introduced `Dnp3FlowState.carry: Vec<u8>` and the `is_non_dnp3` desync-latch check.
- STORY-140 (wave 63) split `carry: Vec<u8>` into `carry_c2s`/`carry_s2c` and introduced the
  `active_carry!(flow, direction)` macro. It is after this change that the desync-latch condition
  `active_carry!(flow, direction).is_empty()` became a defect — the single-carry check that correctly
  proxied "unestablished flow" now only checks the CURRENT DIRECTION's carry, which can be empty even
  when the other direction has established bytes buffered.
- DESIGN-CROSS-DIRECTION-STATE.md §2 analyzed this: the carry-direction split changes the semantics of
  `carry.is_empty()` from "no bytes accepted from any direction" to "no bytes accepted from this direction."
  The fix restores the original intent by checking BOTH carries.
- RULING-DNP3-DESYNC-001 §2.2 explicitly rejected Option B (per-direction `is_non_dnp3` flags) as more
  complex and semantically wrong. Option A (both-carries-empty predicate) is the correct one-line fix.
- STORY-142 is a follow-on to STORY-140; it cannot be written or compiled until STORY-140 is merged to
  `develop`. The `active_carry!` macro and `carry_c2s`/`carry_s2c` fields only exist after STORY-140.

## Architecture Compliance Rules

From BC-2.15.009 v2.0 and RULING-DNP3-DESYNC-001 §2:

1. **`active_carry!(flow, direction).is_empty()` is REMOVED from the bail condition (BC-2.15.009 v2.0 Precondition 3):**
   The single-direction check is replaced with `flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`.
2. **No new fields added to `Dnp3FlowState` (RULING-DNP3-DESYNC-001 §2.2 Option A rationale):**
   This is a one-line predicate change only. No `is_non_dnp3_c2s`/`is_non_dnp3_s2c` split.
3. **`is_non_dnp3` remains PER-FLOW (RULING-DNP3-DESYNC-001 §2.2):** "If the flow is not DNP3, both
   directions are not DNP3." The shared latch is correct; the fix is to the bail CONDITION, not the latch.
4. **ENIP and Modbus code are OUT OF SCOPE:** Do NOT touch `src/analyzer/enip.rs` or `src/analyzer/modbus.rs`.
5. **`first_c2s_frame_seen` / `first_s2c_frame_seen` tracking flags are DEFERRED to v0.12.0** (RULING-DNP3-DESYNC-001 §2.3 enhancement path). This story implements only the one-line Wave 64 fix.

## Library & Framework Requirements

- No new `Cargo.toml` dependencies
- All code changes are in `src/analyzer/dnp3.rs` (existing file, no new imports)
- `proptest` is NOT required for this story (no VP is recommended per RULING-DNP3-DESYNC-001 §5;
  the fix is a targeted regression test, not a new property class)

## File Structure Requirements

**Files to modify:**

- `src/analyzer/dnp3.rs`
  - Change one line: the `if active_carry!(flow, direction).is_empty()` condition at line 363 (post-STORY-140)
    to `if flow.carry_c2s.is_empty() && flow.carry_s2c.is_empty()`
- `tests/dnp3_detection_tests.rs`
  - Add `mod desync_latch { ... }` with 3 named tests

**Files NOT to modify:**
- `src/analyzer/modbus.rs`, `src/analyzer/enip.rs` — out of scope
- `src/dispatcher.rs` — no call-site change needed
- Any other DNP3 test file — no existing call-site updates needed (this story only adds new tests)

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/dnp3.rs` change (one-line condition) | ~50 |
| New `mod desync_latch` tests (3 tests) | ~300 |
| BC-2.15.009.md context | ~300 |
| RULING-DNP3-DESYNC-001 (ruling context) | ~400 |
| **Total** | **~1,050** |

Context utilization: ~1,050 tokens / ~200,000 token window = ~0.5%. This is a small, surgical story.

## Dependency Rationale

Wave 64 (same wave as STORY-141; independent parallel story).

**STORY-142 depends on STORY-140** because: the bug this story fixes only exists in the post-STORY-140
codebase. The `active_carry!(flow, direction).is_empty()` condition at `dnp3.rs:363` and the
`carry_c2s`/`carry_s2c` fields that the fix references are introduced by STORY-140. STORY-142 cannot
be compiled until STORY-140 is on `develop`.

**STORY-142 runs in parallel with STORY-141** (Wave 64): STORY-141 touches `src/analyzer/modbus.rs`;
STORY-142 touches `src/analyzer/dnp3.rs`. There is no file overlap.
