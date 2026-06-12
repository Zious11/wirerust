# Red Gate Log — STORY-108: DNP3 Direct Detection Emissions

**Date:** 2026-06-11
**Story:** STORY-108 — DNP3 Direct Detection Emissions (T1692.001, T0814, T0836, co-emission, summarize)
**Phase:** 3 TDD — Red Gate (Step b)
**Test file:** `tests/dnp3_detection_tests.rs`
**Branch:** `feature/story-108-dnp3-direct-detections`

## Red Gate Result

```
test result: FAILED. 0 passed; 24 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

**Red Gate: PASSED — all 24 new tests fail, none pass vacuously.**

## Build Status

```
cargo build --all-targets → Finished `dev` profile [unoptimized + debuginfo] — CLEAN
```

The test file compiles without errors or warnings.

## Test Inventory

| Test Function | AC/EC | Failure Mode | Stub Hit |
|---|---|---|---|
| `test_direct_operate_count_increments_on_control_fc` | AC-001 | Assertion: `direct_operate_count==0`, expected 1 | `detect_control_class_burst` (not yet called from on_data) |
| `test_t1692_001_emitted_at_threshold_plus_one` | AC-002 | Assertion: `all_findings.len()==0`, expected 1 | `detect_control_class_burst` |
| `test_t1692_001_one_shot_guard` | AC-003 | Assertion: `t1692_count==0`, expected 1 | `detect_control_class_burst` |
| `test_t1692_001_window_expiry_resets_counter` | AC-004 | Assertion: `all_findings.len()==0`, expected 1 | `detect_control_class_burst` |
| `test_t0814_emitted_per_occurrence_cold_restart` | AC-005a | Assertion: `all_findings.len()==0`, expected 1 | `detect_restart` |
| `test_t0814_emitted_per_occurrence_warm_restart` | AC-005b | Assertion: `all_findings.len()==0`, expected 1 | `detect_restart` |
| `test_initialize_data_not_restart` | AC-005c | Assertion: `all_findings.len()==0`, expected 1 (for the COLD_RESTART pre-condition) | `detect_restart` |
| `test_t0836_emitted_for_write_fc` | AC-006a | Assertion: `all_findings.len()==0`, expected 1 | `detect_write` |
| `test_write_fc_not_t1692` | AC-006b | Assertion: `t0836_count==0`, expected 20 | `detect_write` |
| `test_co_emission_ordering_t0814_before_derived` | AC-007 | Assertion: `all_findings.len() < 2`, expected >= 2 | `detect_restart` |
| `test_max_findings_cap_preserves_first_finding` | AC-008 | Assertion: `all_findings.len()==MAX_FINDINGS-1`, expected MAX_FINDINGS | `detect_restart` |
| `test_max_findings_counters_updated_when_capped` | AC-009 | Assertion: `restart_event_count==0`, expected 1 | `detect_restart` |
| `test_summarize_function_code_distribution` | AC-010 | `todo!()` panic in `summarize()` at `src/analyzer/dnp3.rs:460` | `summarize()` stub |
| `test_summarize_zero_flows` | AC-011 | `todo!()` panic in `summarize()` at `src/analyzer/dnp3.rs:460` | `summarize()` stub |
| `test_BC_2_15_010_threshold_is_strictly_greater_not_gte` | AC-012 | Assertion: `direct_operate_count==0`, expected 10 | `detect_control_class_burst` |
| `test_EC_001_direct_operate_nr_counts_toward_threshold` | EC-001 | Assertion: `t1692_count==0`, expected 1 | `detect_control_class_burst` |
| `test_EC_002_no_finding_at_exact_threshold` | EC-002 | Assertion: `direct_operate_count==0`, expected 10 | `detect_control_class_burst` |
| `test_EC_005_two_cold_restarts_restart_event_count_is_2` | EC-005 | Assertion: `t0814_count==0`, expected 2 | `detect_restart` |
| `test_EC_006_cap_restart_counter_still_increments` | EC-006 | Assertion: `restart_event_count==0`, expected 1 | `detect_restart` |
| `test_EC_007_control_then_write_separate_findings_never_cotagged` | EC-007 | Assertion: `t1692 == false`, expected true | `detect_control_class_burst`, `detect_write` |
| `test_EC_008_wrapping_sub_out_of_order_timestamp_no_panic` | EC-008 | Assertion: `t1692 == false`, expected true | `detect_control_class_burst` |
| `test_BC_2_15_020_summarize_control_operation_counts_per_flow` | AC-010 | `todo!()` panic in `summarize()` | `summarize()` stub |
| `test_BC_2_15_020_summarize_does_not_push_findings` | AC-010 | `todo!()` panic in `summarize()` | `summarize()` stub |
| `test_BC_2_15_020_summarize_includes_parse_errors` | AC-011 | `todo!()` panic in `summarize()` | `summarize()` stub |

## Failure Mode Summary

- **20 tests**: Assertion failures — production detection logic not yet wired into `on_data`
  (counters stay 0, `all_findings` stays empty, because `detect_control_class_burst`,
  `detect_restart`, `detect_write` are not called from the `on_data` frame-walk loop).
- **4 tests**: `todo!()` panic — `summarize()` method is an unimplemented stub at
  `src/analyzer/dnp3.rs:460`.

## Vacuity Check

Two tests that could have passed vacuously were hardened:
- `test_EC_002_no_finding_at_exact_threshold`: adds `direct_operate_count == 10` assertion
  (fails: counter stays 0 without implementation)
- `test_initialize_data_not_restart`: adds a COLD_RESTART pre-condition assertion that
  must also pass (fails: `detect_restart` not wired)

No test passes vacuously. All 24 tests fail for meaningful behavioral reasons.

## Handoff to Implementer

Implement the following in `src/analyzer/dnp3.rs` to make tests pass (one at a time):

1. **Task 3** (`detect_control_class_burst`): Wire into `on_data` frame-walk. Increment
   `direct_operate_count`, seed `window_start_ts`, check window expiry with `wrapping_sub`,
   emit T1692.001 when `count > threshold && !emitted && !expired`.
2. **Task 4** (`detect_restart`): Wire into `on_data`. Push T0814 per COLD_RESTART/WARM_RESTART.
   Increment `restart_event_count` unconditionally (even when capped).
3. **Task 5** (`detect_write`): Wire into `on_data`. Push T0836 for FC=0x02. T0836 only.
4. **Task 6** (MAX_FINDINGS cap): Check `self.all_findings.len() < MAX_FINDINGS` before each push.
5. **Task 7** (co-emission ordering): Direct findings (T0814/T1692.001) pushed before derived.
6. **Task 8** (`summarize()`): Implement the `todo!()`. Return `AnalysisSummary` with
   `function_code_distribution`, `control_operation_counts`, `total_frames`,
   `total_parse_errors`, `flows_analyzed` in `detail`. Present even for zero flows.

Architecture compliance rules (from STORY-108):
- Use `wrapping_sub` for ALL u32 timestamp arithmetic.
- `restart_event_count` incremented unconditionally (never gated by cap).
- `detect_write` emits T0836 ONLY — never T1692.001.
- `detect_control_class_burst` emits T1692.001 ONLY — never T0836.
- No dependency on `crate::analyzer::modbus`.
