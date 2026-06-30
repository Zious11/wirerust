---
document_type: story
story_id: STORY-141
title: "Modbus Per-Direction Carry Buffer + Saturating Window Monotonicity (DRIFT-MODBUS-DIRECTION-001 / DRIFT-MODBUS-CLOCK-001)"
epic_id: E-14
wave: 64
points: 8
phase: f3
tdd_mode: strict
status: ready
feature_id: issue-316-enip-v0.11.0
github_issue: 316
subsystems: [SS-14]
target_module: analyzer/modbus
depends_on: []
blocks: []
behavioral_contracts:
  - BC-2.14.002
  - BC-2.14.016
  - BC-2.14.017
  - BC-2.14.019
verification_properties:
  - VP-037
  - VP-038
assumption_validations: []
risk_mitigations: []
ruling: RULING-MODBUS-SIBLING-001
inputs:
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.002.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.016.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.017.md
  - .factory/specs/behavioral-contracts/ss-14/BC-2.14.019.md
  - .factory/specs/architecture/decisions/ADR-005-binary-ics-protocol-integration-modbus-tcp.md
  - .factory/cycles/feature-enip-v0.11.0/RULING-MODBUS-SIBLING-001-carry-and-clock.md
input-hash: "827c8da"
---

# STORY-141: Modbus Per-Direction Carry Buffer + Saturating Window Monotonicity (DRIFT-MODBUS-DIRECTION-001 / DRIFT-MODBUS-CLOCK-001)

## Narrative

**As a** security analyst relying on wirerust Modbus/ICS detections for OT threat detection,
**I want** the Modbus analyzer to correctly isolate carry-buffer state per TCP direction and
apply backwards-clock-safe window expiry arithmetic,
**so that** bidirectional Modbus flows do not produce phantom findings or suppress legitimate
detections (DRIFT-MODBUS-DIRECTION-001), adversarially injected out-of-order timestamps cannot
abort in-progress burst windows (DRIFT-MODBUS-CLOCK-001) — unblocking the v0.11.0 release.

## Behavioral Contracts

| BC ID | Version | Title | Story Role |
|-------|---------|-------|-----------|
| BC-2.14.002 | v2.0 | MBAP Header Rejected for ADU Shorter than 8 Bytes | Per-direction carry split (`carry_c2s`/`carry_s2c`); direction isolation invariant; EC-007 direction non-contamination |
| BC-2.14.016 | v2.3 | Coordinated Write Sequence (T0831, 5-second window) | `saturating_sub` window-expiry; EC-013 backwards-clock T0831 no-reset |
| BC-2.14.017 | v2.7 | Write-Rate Exceeding Either Burst or Sustained Threshold (T0806) | `saturating_sub` burst/sustained windows; `>=` KEPT at sustained gate; EC-012 backwards-clock burst no-reset |
| BC-2.14.019 | v1.5 | Exception Response Anomaly (T0888, 10-second window) | `saturating_sub` exception-burst window; EC-009 backwards-clock no-reset |

## Acceptance Criteria

### AC-141-001: `ModbusFlowState` has `carry_c2s` and `carry_s2c`; `carry` field removed
**Traces to:** BC-2.14.002 v2.0 Precondition 3, Postcondition 1, Invariant 1, Invariant 4, EC-007

The single `carry: Vec<u8>` field is removed from `ModbusFlowState`. Two separate fields are added:
- `carry_c2s: Vec<u8>` — partial ADUs from the TCP client (master → device, ClientToServer direction)
- `carry_s2c: Vec<u8>` — partial ADUs from the TCP server (device → master, ServerToClient direction)

The `on_data` signature is UNCHANGED — `direction: Direction` is already present in the existing Modbus
`on_data` interface (lines 1018–1025). No call-site sweep of `dispatcher.rs` is required.

Within `on_data`, a directional carry reference is selected at call entry:
```rust
let active_carry = match direction {
    Direction::ClientToServer => &mut flow.carry_c2s,
    Direction::ServerToClient => &mut flow.carry_s2c,
};
```
This `active_carry` is used for ALL carry operations: prepend-to-buf (lines 1043–1056),
MBAP-partial stash (line 1084), ADU-partial stash (line 1124), and cap-check guards (lines 1080, 1120).

Invariant: `carry_c2s` and `carry_s2c` are NEVER mixed. No frame-walk loop ever prepends bytes from
one direction into the other. Prevents DRIFT-MODBUS-DIRECTION-001 (RULING-MODBUS-SIBLING-001 §1.2).

Per-direction cap: each directional carry is independently bounded at 260 bytes (`MAX_ADU_CARRY_BYTES`).
`is_non_modbus` is set if EITHER directional carry cap overflows (shared latch, correct per §1.4).

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_001_carry_direction_isolation_no_splice`
— Deliver 6-byte partial c2s MBAP prefix (stashed into `carry_c2s`), then deliver complete s2c ADU
(TxnID=0x0006, FC=0x03, 13 bytes) on same FlowKey. Assert: `fn_code_counts[0x03] == 1` (s2c read parsed
correctly), `fn_code_counts[0x06] == 0` (no garbled write), `parse_errors == 0`. This is the EC-X1
direction-isolation regression test (RULING-MODBUS-SIBLING-001 §1.1 repro scenario). (traces to BC-2.14.002 v2.0 Invariant 4, EC-007)

### AC-141-002: `on_data` selects directional carry; no signature change; `dispatcher.rs` untouched
**Traces to:** BC-2.14.002 v2.0 Precondition 3; RULING-MODBUS-SIBLING-001 §1.2

The existing `on_data` receives `direction: Direction` (lines 1018–1025). The fix adds `let active_carry = match direction { ... }` at the start of the carry-use region and replaces all `flow.carry` references with `active_carry`. No change to the `on_data` function signature. No changes to `dispatcher.rs` (Modbus already has direction threading — unlike DNP3 which required a new parameter in STORY-140).

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_002_same_direction_carry_completes_normally`
— Deliver 6-byte partial c2s MBAP prefix (stashed into `carry_c2s`), then deliver the remaining 2 bytes
of the same c2s MBAP header + full PDU bytes on the SAME FlowKey, SAME direction (ClientToServer). Assert:
`total_pdu_count == 1`, `write_count == 1` (FC=0x06 write), `parse_errors == 0`. Confirms same-direction
carry completion works after the split. (traces to BC-2.14.002 v2.0 Postcondition 1)

### AC-141-003: Carry-cap DoS guard applied per-direction; `is_non_modbus` shared latch
**Traces to:** BC-2.14.002 v2.0 Invariant 1 (per-direction); RULING-MODBUS-SIBLING-001 §1.5

Each stash path checks `active_carry.len() + remaining.len() > MAX_ADU_CARRY_BYTES (260)` against the
directional carry independently. `carry_c2s` and `carry_s2c` each have their own 260-byte DoS guard.
`is_non_modbus` remains PER-FLOW: either directional cap overflow sets the shared `is_non_modbus = true`
flag (correct — if the Modbus stream overflows in either direction, the flow is non-Modbus).

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_003_cap_overflow_sets_is_non_modbus`
— Deliver repeated 4-byte sub-MBAP partial chunks in c2s direction until cumulative total exceeds 260 bytes.
Assert: `flow.is_non_modbus == true`; `carry_s2c` is empty and unaffected (cap overflow in one direction
does not corrupt the other direction's carry). (traces to BC-2.14.002 v2.0 Invariant 1)

### AC-141-004: Stale module-level doc-comment at `modbus.rs:112` corrected to `saturating_sub`
**Traces to:** BC-2.14.016 v2.3 Invariant rationale; BC-2.14.017 v2.7 Invariant rationale; RULING-MODBUS-SIBLING-001 §2.4

The module-level doc-comment at `modbus.rs:112` (currently prescribing `wrapping_sub` per
f2-fix-directives §11.5b) is changed to:
```rust
/// All window-duration arithmetic uses `saturating_sub` on the u32 timestamps
/// (RULING-MODBUS-SIBLING-001 §2.3 — replaces wrapping_sub per f2-fix-directives §11.5b).
/// Under saturating_sub, backwards-clock packets (out-of-order pcap delivery or
/// adversarial injection) produce elapsed=0, preserving burst accumulation rather than
/// triggering a spurious window reset. See RULING-MODBUS-SIBLING-001 §2.2.
```

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_004_no_wrapping_sub_in_modbus`
— `grep -n 'wrapping_sub' src/analyzer/modbus.rs` returns no results after the fix. Implemented as a
`#[test]` that calls `std::process::Command::new("grep")` with `wrapping_sub` against the file, asserting
empty output. (traces to BC-2.14.016 v2.3 Invariant, BC-2.14.017 v2.7 Invariant)

### AC-141-005: Line 534 `wrapping_sub` → `saturating_sub` (T0831 5s window expiry)
**Traces to:** BC-2.14.016 v2.3 Postcondition pseudocode window-expiry; RULING-MODBUS-SIBLING-001 §2.2 table row 534

`modbus.rs:534`: `timestamp.wrapping_sub(flow.t0831_window_start_ts) > T0831_WINDOW_SECS` is changed to
`timestamp.saturating_sub(flow.t0831_window_start_ts) > T0831_WINDOW_SECS`.

Semantic: when `now_ts < t0831_window_start_ts` (backwards clock), `saturating_sub` returns 0. 0 is NOT
> 5 → window NOT reset → `t0831_window_write_count` is preserved → coordinated-write detection continues.

`>` operator is unchanged (consistent with all other non-sustained windows; see §2.3).

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_005_t0831_backwards_clock_no_reset`
— Deliver holding-register write at ts=100 (`t0831_window_start_ts=100`, `t0831_window_write_count=1`), then
deliver holding-register write at ts=50 (backwards). Assert: `saturating_sub(50, 100) = 0`; NOT > 5 → window
NOT reset; `t0831_window_write_count >= 2`; T0831 co-tag fires. Based on BC-2.14.016 EC-013. (traces to BC-2.14.016 v2.3 Postcondition window-expiry, EC-013)

### AC-141-006: Line 595 `wrapping_sub` → `saturating_sub` (T0806 1s burst window expiry)
**Traces to:** BC-2.14.017 v2.7 Postcondition burst window pseudocode; RULING-MODBUS-SIBLING-001 §2.2 table row 595

`modbus.rs:595`: `timestamp.wrapping_sub(flow.window_start_ts) > WRITE_BURST_WINDOW_SECS` is changed to
`timestamp.saturating_sub(flow.window_start_ts) > WRITE_BURST_WINDOW_SECS`.

`>` operator is unchanged.

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_006_burst_backwards_clock_no_reset`
— Deliver 20 write FCs at ts=100 (`window_write_count=20`, `window_start_ts=100`), then 1 write FC at ts=50
(backwards), then 1 write FC at ts=100. Assert: `window_write_count >= 21` (no reset on backwards-ts call);
T0806 burst finding fires (21 > threshold=20). This is the EC-X2 formalized regression
(`scratch_EC_X2_explicit_window_state_via_process_pdu` scenario from RULING-MODBUS-SIBLING-001 §2.1). (traces to BC-2.14.017 v2.7 Postcondition burst window, EC-012)

### AC-141-007: Line 670 `wrapping_sub` → `saturating_sub`, `>=` PRESERVED (T0806 sustained >=2s gate)
**Traces to:** BC-2.14.017 v2.7 Postcondition sustained window pseudocode; RULING-MODBUS-SIBLING-001 §2.3

`modbus.rs:670`: `timestamp.wrapping_sub(flow.sustained_window_start_ts) >= WRITE_SUSTAINED_WINDOW_SECS`
is changed to `timestamp.saturating_sub(flow.sustained_window_start_ts) >= WRITE_SUSTAINED_WINDOW_SECS`.

**The `>=` operator is KEPT.** This is intentional: the sustained detector's `>=` is the minimum-duration
gate ("the window must have run AT LEAST 2 seconds"). This is semantically different from expiry-reset
operators (which use `>` to mean "window has expired"). The `>=` is NOT changed to `>`.

Semantic for backwards-ts: `saturating_sub(backwards_ts, start) = 0`; 0 NOT `>=` 2 → minimum-duration
gate not met → no spurious sustained burst on the backwards-ts call itself. Correct behavior.

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_007_sustained_operator_ge_preserved`
— Deliver 25 write FCs at ts=100 (`sustained_window_start_ts=100`), then 1 write FC at ts=50 (backwards).
Assert: `saturating_sub(50, 100) = 0`; 0 NOT `>=` 2 → minimum-duration gate not met on this backwards-ts
call → no spurious sustained burst. Deliver 1 more write at ts=102: elapsed=2 → gate met → rate check fires.
Assert: sustained finding emitted (sustained_window_write_count > threshold over 2s). Confirms `>=` preserved
and correct semantics under backwards-ts. (traces to BC-2.14.017 v2.7 Postcondition sustained window)

### AC-141-008: Line 820 `wrapping_sub` → `saturating_sub` (T0888 10s exception window expiry)
**Traces to:** BC-2.14.019 v1.5 Postcondition pseudocode; RULING-MODBUS-SIBLING-001 §2.2 table row 820

`modbus.rs:820`: `timestamp.wrapping_sub(*flow.exception_window_start_ts.get(&exc_code).unwrap_or(&timestamp)) > EXCEPTION_WINDOW_SECS`
is changed to `timestamp.saturating_sub(...) > EXCEPTION_WINDOW_SECS`.

`>` operator is unchanged.

**Test:** `tests/modbus_detection_tests.rs::direction_and_clock::test_ac141_008_exception_backwards_clock_no_reset`
— Deliver 5 exception responses for FC=0x83 at ts=100 (`exception_window_start_ts[0x83]=100`), then 1
exception at ts=50 (backwards). Assert: `saturating_sub(50, 100) = 0`; NOT > 10 → window NOT reset;
`exception_window_counts[0x83] >= 6`; burst detection continues. (traces to BC-2.14.019 v1.5 Postcondition, EC-009 amended)

### AC-141-009: All existing Modbus tests pass with the carry-split and saturating_sub changes
**Traces to:** BC-2.14.002 v2.0 Precondition 3; BC-2.14.016 v2.3; BC-2.14.017 v2.7; BC-2.14.019 v1.5; RULING-MODBUS-SIBLING-001 §9 AC-7

All existing tests in all Modbus test files (`tests/modbus_detection_tests.rs`,
`tests/modbus_parse_tests.rs`, `tests/modbus_e2e_tests.rs`,
`tests/bc_2_14_103_modbus_correlation_tests.rs`, `tests/bc_2_14_105_modbus_dispatch_tests.rs`)
pass after the carry-split and saturating_sub changes.

**Test:** `cargo test --all-targets` — zero regressions. (traces to BC-2.14.002 v2.0, BC-2.14.017 v2.7)

### AC-141-010: AC-006 of STORY-104 corrected — `saturating_sub` replaces `wrapping_sub` regression test
**Traces to:** BC-2.14.017 v2.7; RULING-MODBUS-SIBLING-001 §4.5

STORY-104 AC-006 mandated `wrapping_sub` with regression test `test_window_elapsed_uses_wrapping_sub`.
Per RULING-MODBUS-SIBLING-001 §4.5 and BC-2.14.016/017/019 v-bumps, this AC is superseded.

The test is replaced with `test_window_elapsed_uses_saturating_sub` which verifies `saturating_sub` semantics:
- A backwards-ts write (ts < window_start_ts) does NOT reset `window_write_count`.
- Specifically: deliver a write at `ts=0xFFFFFF00` (near u32::MAX), seed window_start=`0xFFFFFF00`. Deliver
  write at `ts=0x00000100` (has wrapped, is numerically smaller → backwards). Assert: `saturating_sub(0x00000100, 0xFFFFFF00) = 0`; NOT > threshold → window NOT reset; `window_write_count` incremented to 2. No panic.

The old test `test_window_elapsed_uses_wrapping_sub` is removed; the new test is
`test_window_elapsed_uses_saturating_sub` in `tests/modbus_detection_tests.rs::direction_and_clock`.
(traces to BC-2.14.017 v2.7 Postcondition burst window, EC-012)

### AC-141-011: VP-037 proptest — carry direction isolation (GENUINE proptest)
**Traces to:** VP-037; BC-2.14.002 v2.0 direction-isolation Invariant; RULING-MODBUS-SIBLING-001 §5

VP-037 proptest harness is implemented as a passing proptest suite asserting that per-direction
`fn_code_counts` from an interleaved (alternating direction) sequence match independent single-direction
control runs.

**CRITICAL: these MUST be genuine proptests using `proptest::prelude::*` with generated strategies.**
NOT deterministic point tests masquerading as proptests. The lesson from STORY-139 F-139-002 applies
directly here. The `proptest!` macro must drive strategy generation; test failure must be reproducible
via the seed.

**VP-037 tests** (mod `vp037_modbus_carry_direction_isolation` in `tests/modbus_detection_tests.rs`):
- `proptest_vp037_direction_isolation_fn_code_counts` — strategy generates `split_offset in 0usize..6`
  (partial MBAP prefix length, 0..6 bytes before the full 8-byte MBAP); builds complete and partial
  Modbus ADUs (FC=0x03 Read Holding Registers for s2c, FC=0x06 Write for c2s); delivers partial c2s
  split at `split_offset`, full s2c ADU, then remaining c2s bytes; asserts `fn_code_counts[0x03] == 1`
  and `fn_code_counts[0x06] == 1` and `parse_errors == 0` for all values of `split_offset`. Establishes
  carry-isolation as a structural behavioral invariant.
- `proptest_vp037_independent_run_equivalence` — strategy generates `split_offset in 0usize..6`;
  verifies that the interleaved run `fn_code_counts` and `parse_errors` equal the sum of independent
  same-direction control runs (no splice: interleaved == c2s-only + s2c-only).

**Test:** `tests/modbus_detection_tests.rs::vp037_modbus_carry_direction_isolation::proptest_vp037_direction_isolation_fn_code_counts`
**Test:** `tests/modbus_detection_tests.rs::vp037_modbus_carry_direction_isolation::proptest_vp037_independent_run_equivalence`
(traces to VP-037; BC-2.14.002 v2.0 direction-isolation Invariant)

### AC-141-012: VP-038 proptest — window monotonic no-spurious-reset (GENUINE proptest)
**Traces to:** VP-038; BC-2.14.016 v2.3 EC-013; BC-2.14.017 v2.7 EC-012; BC-2.14.019 v1.5 EC-009; RULING-MODBUS-SIBLING-001 §5

VP-038 proptest harness is implemented as passing proptest suites covering all four windowed detections.

**CRITICAL: genuine proptests only.** `prop_assume!(backwards_ts <= window_start)` is the correct
technique for constraining the strategy domain — NOT replacing with a deterministic fixed scenario.

**VP-038 tests** (mod `vp038_modbus_window_monotonic_no_spurious_reset` in `tests/modbus_detection_tests.rs`):
- `proptest_vp038_sub_a_t0831_backwards_ts_no_reset` — Sub-A: T0831 5s window; strategy generates
  `(window_start in 1u32..u32::MAX, backwards_ts in 0u32..=u32::MAX)` with
  `prop_assume!(backwards_ts <= window_start)`; asserts `saturating_sub(backwards_ts, window_start) == 0` →
  NOT > 5 → no reset; `t0831_window_write_count` preserved.
- `proptest_vp038_sub_b_burst_backwards_ts_no_reset` — Sub-B: T0806 1s burst window; strategy generates
  `(window_start in 1u32..u32::MAX, burst_count in 2u64..200, backwards_ts in 0u32..=u32::MAX)` with
  `prop_assume!(backwards_ts <= window_start)`; asserts `saturating_sub(backwards_ts, window_start) == 0`
  → NOT > 1 → window NOT reset; `window_write_count` preserved; `burst_count + 1 > threshold` → burst fires
  on next forward-ts event.
- `proptest_vp038_sub_c_sustained_backwards_ts_no_spurious_fire` — Sub-C: T0806 sustained >=2s gate;
  generates `(window_start in 1u32..u32::MAX, backwards_ts in 0u32..=u32::MAX)` with
  `prop_assume!(backwards_ts <= window_start)`; asserts `saturating_sub(backwards_ts, window_start) == 0` →
  NOT `>=` 2 → minimum-duration gate not met on backwards-ts call alone (note `>=` preserved, not `>`).
- `proptest_vp038_sub_d_exception_backwards_ts_no_reset` — Sub-D: T0888 10s exception window; strategy
  generates `(window_start in 1u32..u32::MAX, backwards_ts in 0u32..=u32::MAX)` with
  `prop_assume!(backwards_ts <= window_start)`; asserts `saturating_sub(backwards_ts, window_start) == 0`
  → NOT > 10 → window NOT reset; exception count preserved.
- `test_vp038_sub_e_genuine_rollover_no_spurious_reset` — Deterministic: `window_start = u32::MAX - 5`,
  `now_ts = 4`; `saturating_sub(4, u32::MAX-5) = 0`; no spurious reset on any window (vs. `wrapping_sub`
  which gives 10, spuriously firing the 10s exception window). Documents old vs. new semantics.

**Test:** `tests/modbus_detection_tests.rs::vp038_modbus_window_monotonic_no_spurious_reset::proptest_vp038_sub_a_t0831_backwards_ts_no_reset`
**Test:** `tests/modbus_detection_tests.rs::vp038_modbus_window_monotonic_no_spurious_reset::proptest_vp038_sub_b_burst_backwards_ts_no_reset`
**Test:** `tests/modbus_detection_tests.rs::vp038_modbus_window_monotonic_no_spurious_reset::proptest_vp038_sub_c_sustained_backwards_ts_no_spurious_fire`
**Test:** `tests/modbus_detection_tests.rs::vp038_modbus_window_monotonic_no_spurious_reset::proptest_vp038_sub_d_exception_backwards_ts_no_reset`
**Test:** `tests/modbus_detection_tests.rs::vp038_modbus_window_monotonic_no_spurious_reset::test_vp038_sub_e_genuine_rollover_no_spurious_reset`
(traces to VP-038; BC-2.14.016 v2.3 EC-013, BC-2.14.017 v2.7 EC-012, BC-2.14.019 v1.5 EC-009)

### AC-141-013: `cargo clippy`, `cargo fmt`, `cargo test --all-targets` all green
**Traces to:** BC-2.14.002 v2.0 (no `carry` singular field); BC-2.14.016/017/019 (no `wrapping_sub` in window paths)

After all changes:
- `cargo clippy --all-targets -- -D warnings` — zero warnings. No `wrapping_sub` in windowed
  comparison paths in `modbus.rs`.
- `cargo fmt --check` — zero format drift.
- `cargo test --all-targets` — full test suite green.
- `grep -n 'wrapping_sub' src/analyzer/modbus.rs` — returns no results (all 4 sites replaced).
- `grep -n '\.carry[^_]' src/analyzer/modbus.rs` — returns no results (singular `carry` field absent;
  only `carry_c2s` and `carry_s2c` remain).

**Test:** implicit — CI gate. (traces to BC-2.14.002 v2.0, BC-2.14.016/017/019)

## Architecture Mapping

| Component | Location | Role | Pure/Effectful |
|-----------|----------|------|----------------|
| `ModbusFlowState.carry_c2s: Vec<u8>` | `src/analyzer/modbus.rs` | Per-direction carry buffer (c2s); replaces `carry: Vec<u8>` (line 170) | Effectful (mutated by on_data) |
| `ModbusFlowState.carry_s2c: Vec<u8>` | `src/analyzer/modbus.rs` | Per-direction carry buffer (s2c); replaces `carry: Vec<u8>` | Effectful (mutated by on_data) |
| `ModbusAnalyzer::on_data(...)` | `src/analyzer/modbus.rs` | MBAP parse + detection loop entry; `direction: Direction` already present — no signature change | Effectful shell |
| `crate::reassembly::handler::Direction` | `src/reassembly/handler.rs` | Direction enum; already imported in `modbus.rs` — no new import needed | Pure enum |
| `tests/modbus_detection_tests.rs` | `tests/modbus_detection_tests.rs` | `mod direction_and_clock { ... }` + `mod vp037_modbus_carry_direction_isolation { ... }` + `mod vp038_modbus_window_monotonic_no_spurious_reset { ... }` | Test |

**Subsystem anchor:** SS-14 owns this story's scope because all carry-buffer split, direction routing,
and saturating_sub window expiry fixes are localized to `src/analyzer/modbus.rs`. Per ARCH-INDEX SS-14.
No changes to `dispatcher.rs` are required (direction is already threaded in Modbus on_data per §1.2).

**Dependency anchor:** STORY-141 has `depends_on: []` because Modbus is fully on the `develop` branch
with no pending prerequisites. STORY-139 (ENIP carry-split) and STORY-140 (DNP3 carry-split) established
the pattern but are independent analyzers. The Modbus fix requires no merged predecessor — it mirrors
the established pattern but stands alone.

**Forbidden dependencies:** `src/analyzer/modbus.rs` MUST NOT depend on any other analyzer module
(`enip`, `dnp3`, `arp`, `http`, `tls`, etc.). If this module gains a cross-analyzer dependency
(other than shared types from `src/reassembly/`), the build MUST fail.

**NOT in scope:**
- `src/analyzer/enip.rs` — ENIP was fixed in STORY-139. Do NOT touch.
- `src/analyzer/dnp3.rs` — DNP3 carry-split was fixed in STORY-140. Do NOT touch.
- Other register-level findings beyond BC-2.14.002/016/017/019 (parked per RULING-MODBUS-SIBLING-001).
- No signature change to `on_data` — direction is already available; this is purely an internal field split.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Partial c2s MBAP (< 8 bytes) stashed in `carry_c2s`; next call is s2c direction | `carry_s2c` (empty) prepended to s2c data → clean s2c parse; `carry_c2s` retains c2s partial unchanged — no splice (BC-2.14.002 v2.0 EC-007) |
| EC-002 | Both directions have partial ADUs stashed simultaneously | `carry_c2s.len() > 0` AND `carry_s2c.len() > 0` concurrently; each direction's next delivery completes its own ADU independently |
| EC-003 | c2s carry-cap overflow (> 260 bytes accumulated via repeated sub-8-byte deliveries) | `active_carry.len() + remaining.len() > 260` → `is_non_modbus = true`; `carry_s2c` unaffected (RULING-MODBUS-SIBLING-001 §1.5) |
| EC-004 | 20 writes at ts=100, 1 write at ts=50 (backwards), 1 write at ts=100 | `saturating_sub(50, 100) = 0`; NOT > 1 → window NOT reset; `window_write_count=21`; burst fires at 21 > 20 (BC-2.14.017 v2.7 EC-012) |
| EC-005 | Holding-register write at ts=100, holding-register write at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 5 → T0831 window NOT reset; `t0831_window_write_count` incremented; co-tag fires (BC-2.14.016 v2.3 EC-013) |
| EC-006 | Exception responses at ts=100, exception at ts=50 (backwards) | `saturating_sub(50, 100) = 0`; NOT > 10 → T0888 exception window NOT reset; accumulation preserved (BC-2.14.019 v1.5 EC-009 amended) |
| EC-007 | Genuine u32 rollover: `window_start = u32::MAX - 5`, `now_ts = 4` | `saturating_sub(4, u32::MAX-5) = 0`; no spurious reset on any window (vs. `wrapping_sub` which gives 10) |

## Tasks

- [ ] In `ModbusFlowState`: remove `carry: Vec<u8>` (line 170); add `carry_c2s: Vec<u8>` and `carry_s2c: Vec<u8>`
- [ ] In `on_data` at the carry-use region: add
  `let active_carry = match direction { Direction::ClientToServer => &mut flow.carry_c2s, Direction::ServerToClient => &mut flow.carry_s2c };`
- [ ] Replace ALL `flow.carry` references in `on_data` with `active_carry`:
  - [ ] `flow.carry.clear()` (line 1056) → `active_carry.clear()`
  - [ ] Prepend buf: `flow.carry` → `active_carry` (lines 1043–1053)
  - [ ] Cap-check at line 1080: `flow.carry.len()` → `active_carry.len()`
  - [ ] Stash at line 1084: `flow.carry.extend_from_slice(remaining)` → `active_carry.extend_from_slice(remaining)`
  - [ ] Cap-check at line 1120: `flow.carry.len()` → `active_carry.len()`
  - [ ] Stash at line 1124: `flow.carry.extend_from_slice(remaining)` → `active_carry.extend_from_slice(remaining)`
- [ ] Update module-level doc-comment at line 112 per AC-141-004
- [ ] Replace `wrapping_sub` with `saturating_sub` at all 4 window-expiry sites:
  - [ ] Line 534: `timestamp.saturating_sub(flow.t0831_window_start_ts) > T0831_WINDOW_SECS`
  - [ ] Line 595: `timestamp.saturating_sub(flow.window_start_ts) > WRITE_BURST_WINDOW_SECS`
  - [ ] Line 670: `timestamp.saturating_sub(flow.sustained_window_start_ts) >= WRITE_SUSTAINED_WINDOW_SECS` (**keep `>=`**)
  - [ ] Line 820: `timestamp.saturating_sub(*flow.exception_window_start_ts.get(&exc_code).unwrap_or(&timestamp)) > EXCEPTION_WINDOW_SECS`
- [ ] Add `mod direction_and_clock` to `tests/modbus_detection_tests.rs` with AC-141-001..013 named tests (including AC-141-010 correcting STORY-104 AC-006's `test_window_elapsed_uses_wrapping_sub`)
- [ ] Add `mod vp037_modbus_carry_direction_isolation` with genuine VP-037 proptest harnesses (2 tests)
- [ ] Add `mod vp038_modbus_window_monotonic_no_spurious_reset` with genuine VP-038 proptest + deterministic harnesses (5 tests)
- [ ] Remove `test_window_elapsed_uses_wrapping_sub` from `tests/modbus_detection_tests.rs`; replace with `test_window_elapsed_uses_saturating_sub` in `mod direction_and_clock`
- [ ] Run `cargo test modbus` — all direction_and_clock + VP proptest tests pass
- [ ] Run `cargo test --all-targets` — full test suite green (no regressions)
- [ ] Run `cargo clippy --all-targets -- -D warnings` — zero warnings
- [ ] Run `cargo fmt --check` — zero format drift
- [ ] Run `grep -n 'wrapping_sub' src/analyzer/modbus.rs` — no results
- [ ] Run `grep -n '\.carry[^_]' src/analyzer/modbus.rs` — no results (singular `carry` gone)
- [ ] Run `bin/compute-input-hash --write .factory/stories/STORY-141.md` to populate `input-hash`

## Test Plan

**Test file:** `tests/modbus_detection_tests.rs`

**New test modules added by this story:**

```
mod direction_and_clock {
    test_ac141_001_carry_direction_isolation_no_splice
    test_ac141_002_same_direction_carry_completes_normally
    test_ac141_003_cap_overflow_sets_is_non_modbus
    test_ac141_004_no_wrapping_sub_in_modbus
    test_ac141_005_t0831_backwards_clock_no_reset
    test_ac141_006_burst_backwards_clock_no_reset
    test_ac141_007_sustained_operator_ge_preserved
    test_ac141_008_exception_backwards_clock_no_reset
    test_ac141_009_all_existing_tests_pass  (validated via cargo test --all-targets)
    test_ac141_010_window_elapsed_uses_saturating_sub  (replaces test_window_elapsed_uses_wrapping_sub)
    test_ac141_013_no_wrapping_sub_grep  (implicit CI gate)
}

mod vp037_modbus_carry_direction_isolation {
    proptest_vp037_direction_isolation_fn_code_counts
    proptest_vp037_independent_run_equivalence
}

mod vp038_modbus_window_monotonic_no_spurious_reset {
    proptest_vp038_sub_a_t0831_backwards_ts_no_reset
    proptest_vp038_sub_b_burst_backwards_ts_no_reset
    proptest_vp038_sub_c_sustained_backwards_ts_no_spurious_fire
    proptest_vp038_sub_d_exception_backwards_ts_no_reset
    test_vp038_sub_e_genuine_rollover_no_spurious_reset
}
```

**GENUINE proptest discipline (lessons from STORY-139 F-139-002):** All `proptest!` blocks
MUST use generated strategies via `proptest::prelude::*`. `prop_assume!(backwards_ts <= window_start)`
is the correct technique. A proptest that calls `assert_eq!(50u32.saturating_sub(100), 0)` as its
only assertion is NOT a genuine proptest.

**TDD discipline (strict mode):** Implementer writes `todo!()` stubs for new field additions first.
Existing tests fail due to field rename (RED gate: `flow.carry` references break at compile time).
Implements the direction split and saturating_sub fixes to turn them GREEN one-by-one.

## Previous Story Intelligence

- STORY-102 introduced the MBAP parse function `parse_mbap_header` and the pure-core FC classification.
- STORY-103 introduced `ModbusFlowState` with the single `carry: Vec<u8>` (line 170) and the frame-walk
  loop structure in `on_data`. Also introduced `write_count`, window fields, `exception_window_*` maps,
  and `is_non_modbus`. The carry split in this story directly replaces the single carry from STORY-103.
- STORY-104 introduced all seven detection rules (T0806 burst/sustained, T0831, T0814, T0888, T0888 recon)
  and the `wrapping_sub` window arithmetic in the four window-expiry paths (lines 534/595/670/820). This story
  corrects STORY-104 AC-006 (which mandated `wrapping_sub`) per RULING-MODBUS-SIBLING-001 §4.5.
- STORY-105 delivered the dispatcher integration. `on_data` already receives `direction: Direction`.
  No dispatcher changes are needed — unlike STORY-140 (DNP3), the Modbus on_data already has direction threading.
- STORY-139 (wave 62) fixed the identical bug pattern in `src/analyzer/enip.rs`.
- STORY-140 (wave 63) fixed the identical bug pattern in `src/analyzer/dnp3.rs`, including direction threading.
  Read STORY-140 carefully — STORY-141 mirrors the same pattern but is SIMPLER because Modbus already has
  direction threading. The differences are: no `on_data` signature change, no call-site sweep of `dispatcher.rs`,
  no `resolve_master_ip` removal, and 4 window sites (vs. 8 in DNP3). The carry-cap boundary is also different
  from DNP3: 260 bytes for Modbus (vs. 292 for DNP3).
- The `Direction` type from `crate::reassembly::handler::Direction` is already imported in `modbus.rs`.

## Architecture Compliance Rules

From ADR-005 (ICS Protocol Integration), BC-2.14.002 v2.0, BC-2.14.016 v2.3, BC-2.14.017 v2.7, BC-2.14.019 v1.5:

1. **`carry: Vec<u8>` is REMOVED (BC-2.14.002 v2.0):** Any code referencing `flow.carry` (singular) after
   this story is a regression. The fields are `carry_c2s` and `carry_s2c`. The `grep -n '\.carry[^_]'`
   check in Tasks enforces this.
2. **Direction isolation is structurally enforced (BC-2.14.002 v2.0 direction-isolation Invariant):** The
   `match direction { ... }` arm selects exactly one of the two carry buffers. No conditional path may read
   `carry_c2s` when processing `ServerToClient` or `carry_s2c` when processing `ClientToServer`.
3. **`saturating_sub` is the ONLY permitted window arithmetic (RULING-MODBUS-SIBLING-001 §2.2):**
   `wrapping_sub` MUST NOT appear in any window-expiry comparison in `modbus.rs`. Use
   `timestamp.saturating_sub(window_start_ts)` at all 4 window paths.
4. **`>=` at line 670 is KEPT (RULING-MODBUS-SIBLING-001 §2.3):** The sustained-window minimum-duration
   gate uses `>=` (intentional; means "at least 2 seconds"). Do NOT change to `>`. All other windows use `>`.
5. **Detection counters remain PER-FLOW (RULING-MODBUS-SIBLING-001 §1.3):** `write_count`, `pdu_count`,
   `exception_count`, `pending`, `window_write_count`, `window_start_ts`, and all other non-carry fields
   remain per-flow shared (not split by direction). ONLY `carry_c2s`/`carry_s2c` are per-direction.
6. **No signature change to `on_data`:** The `direction: Direction` parameter is already present. No call-site
   sweep of `dispatcher.rs` is required. This is simpler than DNP3 (STORY-140).
7. **ENIP and DNP3 code are OUT OF SCOPE:** Do NOT touch `src/analyzer/enip.rs`, `src/analyzer/dnp3.rs`,
   or their test files.
8. **The module-level doc-comment at line 112 must be corrected per §2.4:** It prescribes `wrapping_sub`
   (stale since f2-fix-directives). Correct it to prescribe `saturating_sub`.

## Library & Framework Requirements

- `crate::reassembly::handler::Direction` — existing type; already imported in `modbus.rs`; no new dependency
- `proptest` crate — already used in the test suite (VP-037/VP-038 harnesses use `proptest::prelude::*`)
- No new `Cargo.toml` dependencies

## File Structure Requirements

**Files to modify:**

- `src/analyzer/modbus.rs`
  - Remove `carry: Vec<u8>` from `ModbusFlowState`
  - Add `carry_c2s: Vec<u8>`, `carry_s2c: Vec<u8>` to `ModbusFlowState`
  - Add `let active_carry = match direction { ... }` in `on_data` carry-use region
  - Replace all `flow.carry` references with `active_carry`
  - Replace all `wrapping_sub` with `saturating_sub` at lines 534, 595, 670, 820
  - Keep `>=` at line 670 (sustained minimum-duration gate)
  - Correct module-level doc-comment at line 112
- `tests/modbus_detection_tests.rs`
  - Remove `test_window_elapsed_uses_wrapping_sub` (from STORY-104 AC-006)
  - Add `mod direction_and_clock { ... }` with AC-141-001..010 named tests (including replacement `test_ac141_010_window_elapsed_uses_saturating_sub`)
  - Add `mod vp037_modbus_carry_direction_isolation { ... }` with VP-037 proptest harnesses
  - Add `mod vp038_modbus_window_monotonic_no_spurious_reset { ... }` with VP-038 proptest harnesses

**Files NOT to modify:**
- `src/dispatcher.rs` — no call-site change needed (Modbus on_data already has direction)
- `src/analyzer/enip.rs`, `src/analyzer/dnp3.rs` — out of scope
- `docs/adr/0005` — ADR-005 amendment (if any) is an F4 action, not F3

## Token Budget Estimate

| Section | Estimated tokens |
|---------|-----------------|
| `src/analyzer/modbus.rs` changes (carry split, active_carry, saturating_sub at 4 sites, doc-comment) | ~600 |
| New `mod direction_and_clock` tests (10 tests) | ~700 |
| `mod vp037_modbus_carry_direction_isolation` proptest harnesses (2 tests + helper) | ~400 |
| `mod vp038_modbus_window_monotonic_no_spurious_reset` proptest + deterministic tests (5 tests) | ~450 |
| BC files (4 BCs) | ~800 |
| ADR-005 (reference context) | ~200 |
| RULING-MODBUS-SIBLING-001 (ruling context) | ~500 |
| **Total** | **~3,650** |

Context utilization: ~3,650 tokens / ~200,000 token window = ~1.8%. Well within the 20-30% per-story budget.

## Dependency Rationale

Wave 64 (new wave added for this post-STORY-140 Modbus fix story).

**STORY-141 has no `depends_on`** because the Modbus analyzer is fully on `develop` and requires no merged
predecessor. STORY-139/140 established and confirmed the pattern for ENIP and DNP3 respectively, but Modbus
is an independent analyzer module. The `direction: Direction` threading in `on_data` was established by
STORY-105 (the original Modbus dispatcher integration), not by STORY-139/140.

**STORY-141 runs in parallel with STORY-142** (Wave 64): STORY-141 touches `src/analyzer/modbus.rs`; STORY-142
touches `src/analyzer/dnp3.rs`. There is no file overlap. Wave scheduling places them in the same wave for
delivery efficiency, with no ordering constraint between them.
