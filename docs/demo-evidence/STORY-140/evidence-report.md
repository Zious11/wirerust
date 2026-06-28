# Evidence Report — STORY-140

**Story:** DNP3 Per-Direction Carry Buffer + Saturating Window Monotonicity + Operator Pin
**Drift fixes:** DRIFT-DNP3-DIRECTION-001 / DRIFT-DNP3-CLOCK-001 / DRIFT-DNP3-OP-001
**Branch:** dnp3-direction-clock
**Recorded:** 2026-06-28

---

## Summary

All 18 acceptance-criterion tests pass. Three VHS recordings capture the critical AC clusters.
Zero failures in `cargo test --all-targets` (zero regressions). `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` both clean.

---

## Recording Coverage

### AC-140-001: Carry direction isolation — `carry_c2s` / `carry_s2c` split

**Fix:** DRIFT-DNP3-DIRECTION-001 — single `carry: Vec<u8>` replaced with per-direction `carry_c2s` and `carry_s2c`. `on_data` gains `direction: Direction` parameter.

| File | Type |
|------|------|
| `AC-140-001-carry-direction-isolation.gif` | VHS recording |
| `AC-140-001-carry-direction-isolation.webm` | VHS recording (archival) |
| `AC-140-001-carry-direction-isolation.tape` | VHS script source |

**Test command demonstrated:**
```
cargo test --test dnp3_detection_tests test_ac140_001
```

**Test output:**
```
test direction_and_clock::test_ac140_001_carry_c2s_and_carry_s2c_are_independent ... ok
test direction_and_clock::test_ac140_001_carry_direction_isolation_no_splice ... ok
test result: ok. 2 passed; 0 failed
```

**Traces to:** BC-2.15.016 v2.0 Invariant 6, EC-010

---

### AC-140-004 / AC-140-005 / AC-140-010 / AC-140-011: `saturating_sub` + strict `>` operator pin — VP-036 proptest suite

**Fix:** DRIFT-DNP3-CLOCK-001 + DRIFT-DNP3-OP-001 — all 8 `wrapping_sub` window sites replaced with `saturating_sub`; 300s correlation window `>=` pinned to `>`.

| File | Type |
|------|------|
| `AC-140-004-005-saturating-sub-operator-pin.gif` | VHS recording |
| `AC-140-004-005-saturating-sub-operator-pin.webm` | VHS recording (archival) |
| `AC-140-004-005-saturating-sub-operator-pin.tape` | VHS script source |

**Test command demonstrated:**
```
cargo test --test dnp3_detection_tests vp036
```

**Test output:**
```
test vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset ... ok
test vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_a_ec_x2_repro_t1692 ... ok
test vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire ... ok
test vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_c_300s_window_backwards_ts_no_reset ... ok
test vp036_dnp3_window_monotonic_no_spurious_reset::proptest_vp036_sub_c_operator_pin_elapsed_300_not_expired ... ok
test vp036_dnp3_window_monotonic_no_spurious_reset::test_vp036_sub_d_genuine_rollover_no_spurious_reset ... ok
test result: ok. 6 passed; 0 failed
```

**Sub-properties covered:**
- Sub-A: T1692.001 60s window — backwards-ts does not reset (EC-X2 analog, BC-2.15.010 v1.8 PC4)
- Sub-B: T1691.001 10s block timeout — backwards-ts does not spuriously fire (BC-2.15.014 v2.1 PC3)
- Sub-C: T0827/T0814 300s window — backwards-ts no reset + operator pin boundary at elapsed=300 (BC-2.15.015 v2.0 PC3)
- Sub-D: Genuine u32 rollover — `wrapping_sub` old vs. `saturating_sub` new behavior documented

**Traces to:** BC-2.15.010 v1.8 PC4, BC-2.15.014 v2.1 PC3, BC-2.15.015 v2.0 PC3, RULING-DNP3-SIBLING-001 §2.2 §2.3

---

### AC-140-008: Backwards-ts packet does not reset T1692.001 60s detect window

**Fix:** EC-X2 analog for DNP3 — `saturating_sub(50, 100) = 0`, NOT > 60, so window is preserved; T1692.001 fires at count=11 without spurious reset.

| File | Type |
|------|------|
| `AC-140-008-backwards-ts-no-reset.gif` | VHS recording |
| `AC-140-008-backwards-ts-no-reset.webm` | VHS recording (archival) |
| `AC-140-008-backwards-ts-no-reset.tape` | VHS script source |

**Test command demonstrated:**
```
cargo test --test dnp3_detection_tests test_ac140_008
```

**Test output:**
```
test direction_and_clock::test_ac140_008_regression_backwards_ts_t1692_no_reset ... ok
test result: ok. 1 passed; 0 failed
```

**Traces to:** BC-2.15.010 v1.8 PC4, EC-012; RULING-DNP3-SIBLING-001 §8 AC-9

---

## Full AC Coverage Table

| AC | Description | Test Name | Result | Demo |
|----|-------------|-----------|--------|------|
| AC-140-001 | carry_c2s / carry_s2c isolation — no splice | `test_ac140_001_carry_direction_isolation_no_splice` | PASS | AC-140-001 gif/webm |
| AC-140-001 | carry_c2s / carry_s2c are independent | `test_ac140_001_carry_c2s_and_carry_s2c_are_independent` | PASS | AC-140-001 gif/webm |
| AC-140-002 | Direction-based source IP replaces port-20000 heuristic (c2s) | `test_ac140_002_direction_based_source_ip` | PASS | full suite |
| AC-140-002 | Direction-based source IP — s2c diverges from port heuristic | `test_ac140_002b_direction_s2c_diverges_from_port_heuristic` | PASS | full suite |
| AC-140-003 | Dispatcher passes direction to on_data | `test_ac140_003_dispatcher_passes_direction` | PASS | full suite |
| AC-140-004 | All 8 wrapping_sub sites replaced — validated through VP-036 | VP-036 proptest suite (6 tests) | PASS | AC-140-004/005 gif/webm |
| AC-140-005 | 300s window operator pin: strict `>` | `test_ac140_005_correlation_window_operator_pin_boundary` | PASS | full suite |
| AC-140-006 | All existing DNP3 tests pass — zero regressions | `cargo test --all-targets` | PASS (0 failures) | — |
| AC-140-007 | Regression — direction isolation confirmed | `test_ac140_007_regression_carry_direction_no_splice` | PASS | full suite |
| AC-140-008 | Regression — backwards-ts no T1692.001 window reset | `test_ac140_008_regression_backwards_ts_t1692_no_reset` | PASS | AC-140-008 gif/webm |
| AC-140-009 | Regression — backwards-ts no T0827 window reset | `test_ac140_009_regression_backwards_ts_t0827_no_reset` | PASS | full suite |
| AC-140-010 | VP-035 proptest — carry direction isolation (genuine proptest) | `proptest_vp035_direction_isolation_frame_count` | PASS | full suite |
| AC-140-010 | VP-035 proptest — independent run equivalence | `proptest_vp035_independent_run_equivalence` | PASS | full suite |
| AC-140-011 | VP-036 Sub-A — 60s backwards-ts no reset | `proptest_vp036_sub_a_direct_operate_60s_backwards_ts_no_reset` | PASS | AC-140-004/005 gif/webm |
| AC-140-011 | VP-036 Sub-A — EC-X2 T1692.001 repro | `proptest_vp036_sub_a_ec_x2_repro_t1692` | PASS | AC-140-004/005 gif/webm |
| AC-140-011 | VP-036 Sub-B — 10s block timeout backwards-ts no fire | `proptest_vp036_sub_b_block_timeout_backwards_ts_no_fire` | PASS | AC-140-004/005 gif/webm |
| AC-140-011 | VP-036 Sub-C — 300s window backwards-ts no reset | `proptest_vp036_sub_c_300s_window_backwards_ts_no_reset` | PASS | AC-140-004/005 gif/webm |
| AC-140-011 | VP-036 Sub-C — operator pin elapsed=300 not expired | `proptest_vp036_sub_c_operator_pin_elapsed_300_not_expired` | PASS | AC-140-004/005 gif/webm |
| AC-140-011 | VP-036 Sub-D — genuine u32 rollover no spurious reset | `test_vp036_sub_d_genuine_rollover_no_spurious_reset` | PASS | AC-140-004/005 gif/webm |
| AC-140-012 | clippy + fmt + full suite green | `cargo clippy --all-targets -- -D warnings` / `cargo fmt --check` / `cargo test --all-targets` | PASS | — |

**Total: 20 tests, 20 PASS, 0 FAIL**

---

## Quality Gates (AC-140-012)

```
cargo clippy --all-targets -- -D warnings  →  0 warnings
cargo fmt --check                          →  0 format drift
cargo test --all-targets                   →  0 failures (across all test binaries)
grep -n 'wrapping_sub' src/analyzer/dnp3.rs  →  no results
grep -n '\.carry[^_]' src/analyzer/dnp3.rs  →  no results
```

---

## Files in this Directory

```
AC-140-001-carry-direction-isolation.gif      — VHS recording: carry direction isolation
AC-140-001-carry-direction-isolation.webm     — VHS recording: carry direction isolation (archival)
AC-140-001-carry-direction-isolation.tape     — VHS script source
AC-140-004-005-saturating-sub-operator-pin.gif   — VHS recording: VP-036 proptest suite
AC-140-004-005-saturating-sub-operator-pin.webm  — VHS recording: VP-036 proptest suite (archival)
AC-140-004-005-saturating-sub-operator-pin.tape  — VHS script source
AC-140-008-backwards-ts-no-reset.gif         — VHS recording: backwards-ts no T1692.001 reset
AC-140-008-backwards-ts-no-reset.webm        — VHS recording: backwards-ts no T1692.001 reset (archival)
AC-140-008-backwards-ts-no-reset.tape        — VHS script source
evidence-report.md                           — this file
```
