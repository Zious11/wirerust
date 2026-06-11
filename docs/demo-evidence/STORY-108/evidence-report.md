---
story_id: STORY-108
bc: BC-2.15.010, BC-2.15.011, BC-2.15.012, BC-2.15.013, BC-2.15.020, BC-2.15.022
date: 2026-06-11
recorder: demo-recorder
---

# STORY-108 Demo Evidence Report

DNP3 Direct Detection Emissions — T1692.001, T0814, T0836, Co-Emission, Summarize (wave 37)

## AC Coverage (12/12)

| AC | BC | Test Name(s) | Assertion | Recording | Result |
|----|----|-------------|-----------|-----------|--------|
| AC-001 | BC-2.15.010 PC1/2 | `test_direct_operate_count_increments_on_control_fc` | direct_operate_count=2 after SELECT+DIRECT_OPERATE; window_start_ts seeded on first FC | [GIF](AC-001-control-fc-counter.gif) / [WEBM](AC-001-control-fc-counter.webm) | PASS |
| AC-002 | BC-2.15.010 PC3 | `test_t1692_001_emitted_at_threshold_plus_one` | 0 findings at count=10; exactly 1 T1692.001 at count=11; summary/source_ip/timestamp correct | [GIF](AC-002-t1692-emission.gif) / [WEBM](AC-002-t1692-emission.webm) | PASS |
| AC-003 | BC-2.15.010 PC3 guard | `test_t1692_001_one_shot_guard` | 16 Control FCs → exactly 1 T1692.001 (one-shot guard); counter=16 | [GIF](AC-003-one-shot-guard.gif) / [WEBM](AC-003-one-shot-guard.webm) | PASS |
| AC-004 | BC-2.15.010 PC4 | `test_t1692_001_window_expiry_resets_counter` | Window 1 fires finding; ts=61 resets counter=1/ts/emitted=false; window 2 fires second finding | [GIF](AC-004-window-expiry.gif) / [WEBM](AC-004-window-expiry.webm) | PASS |
| AC-005 | BC-2.15.011 PC1/2 | `test_t0814_emitted_per_occurrence_cold_restart`, `test_t0814_emitted_per_occurrence_warm_restart`, `test_initialize_data_not_restart` | T0814 per-occurrence for 0x0D/0x0E; confidence=High; FC 0x0F excluded; restart_event_count incremented | [GIF](AC-005-t0814-restart.gif) / [WEBM](AC-005-t0814-restart.webm) | PASS |
| AC-006 | BC-2.15.012 PC1 | `test_t0836_emitted_for_write_fc`, `test_write_fc_not_t1692` | T0836 per WRITE; confidence=Medium; summary/source_ip/timestamp correct; 20 WRITEs=20 T0836, 0 T1692.001 | [GIF](AC-006-t0836-write.gif) / [WEBM](AC-006-t0836-write.webm) | PASS |
| AC-007 | BC-2.15.013 PC2/3 | `test_restart_findings_append_in_observation_order` | COLD then WARM → all_findings[0]=T0814, [1]=T0814; 0 T0827 (STORY-109 deferred) | [GIF](AC-007-co-emission-ordering.gif) / [WEBM](AC-007-co-emission-ordering.webm) | PASS |
| AC-008 | BC-2.15.013 PC4/5 | `test_max_findings_cap_preserves_first_finding` | Pre-fill MAX_FINDINGS-1; COLD_RESTART fills last slot (T0814 preserved); second restart: cap hit, no push; restart_event_count=2 | [GIF](AC-008-max-findings-cap.gif) / [WEBM](AC-008-max-findings-cap.webm) | PASS |
| AC-009 | BC-2.15.022 PC1/3 | `test_max_findings_counters_updated_when_capped` | At MAX_FINDINGS cap: no new finding pushed; restart_event_count=1, frame_count=1, fc_counts[0x0D]=1, fn_code_counts[0x0D]=1 | [GIF](AC-009-counters-when-capped.gif) / [WEBM](AC-009-counters-when-capped.webm) | PASS |
| AC-010 | BC-2.15.020 PC1 | `test_summarize_function_code_distribution`, `test_BC_2_15_020_summarize_control_operation_counts_per_flow`, `test_BC_2_15_020_summarize_does_not_push_findings`, `test_BC_2_15_020_summarize_includes_parse_errors` | 5 DIRECT_OPERATE+3 READ → fn_code_counts={5:5, 1:3}; flows_analyzed>=1; total_frames=8; summarize() does not push findings | [GIF](AC-010-summarize.gif) / [WEBM](AC-010-summarize.webm) | PASS |
| AC-011 | BC-2.15.020 INV4 | `test_summarize_zero_flows` | Zero-flow case: flows_analyzed=0, total_frames=0, total_parse_errors=0, function_code_distribution present (not absent) | [GIF](AC-011-summarize-zero-flows.gif) / [WEBM](AC-011-summarize-zero-flows.webm) | PASS |
| AC-012 | BC-2.15.010 INV | `test_BC_2_15_010_threshold_is_strictly_greater_not_gte` | At count=10 (==threshold): 0 findings; direct_operate_emitted=false; counter=10 | [GIF](AC-012-threshold-semantics.gif) / [WEBM](AC-012-threshold-semantics.webm) | PASS |

## Edge Case Coverage (EC-001..EC-008)

| EC | Test Name | Assertion | Result |
|----|-----------|-----------|--------|
| EC-001 | `test_EC_001_direct_operate_nr_counts_toward_threshold` | FC=0x06 (DIRECT_OPERATE_NR) is Control-class; 11 frames → T1692.001; direct_operate_count=11 | PASS |
| EC-002 | `test_EC_002_no_finding_at_exact_threshold` | SELECT (0x03) at exactly count=10 → 0 findings; direct_operate_count=10 | PASS |
| EC-005 | `test_EC_005_two_cold_restarts_restart_event_count_is_2` | Two COLD_RESTARTs → 2 T0814 findings; restart_event_count=2 | PASS |
| EC-006 | `test_EC_006_cap_restart_counter_still_increments` | At MAX_FINDINGS cap + COLD_RESTART → no T0814; restart_event_count=1 | PASS |
| EC-007 | `test_EC_007_control_then_write_separate_findings_never_cotagged` | 11 DIRECT_OPERATE + 1 WRITE → T1692.001 and T0836 are separate findings; no co-tag | PASS |
| EC-008 | `test_EC_008_wrapping_sub_out_of_order_timestamp_no_panic` | ts_start=0xFFFFFFF0; 10 FCs at ts=0..9; wrapping_sub safe; T1692.001 fired (no panic) | PASS |

## Additional Coverage (Adversarial Round P2 — F-108-P2-001)

| Test | Branch | Assertion | Result |
|------|--------|-----------|--------|
| `test_BC_2_15_010_asymmetric_port_master_lower_ip_else_branch` | ELSE (lower_port != 20000) | source_ip=lower_ip=10.0.0.5 (master on ephemeral port) | PASS |
| `test_BC_2_15_010_asymmetric_port_master_upper_ip_if_branch` | IF (lower_port == 20000) | source_ip=upper_ip=10.0.0.9 (master behind outstation) | PASS |

## Full Test Run Output

```
running 26 tests
test story_108::test_BC_2_15_010_threshold_is_strictly_greater_not_gte ... ok
test story_108::test_BC_2_15_010_asymmetric_port_master_lower_ip_else_branch ... ok
test story_108::test_BC_2_15_010_asymmetric_port_master_upper_ip_if_branch ... ok
test story_108::test_BC_2_15_020_summarize_control_operation_counts_per_flow ... ok
test story_108::test_BC_2_15_020_summarize_does_not_push_findings ... ok
test story_108::test_BC_2_15_020_summarize_includes_parse_errors ... ok
test story_108::test_EC_005_two_cold_restarts_restart_event_count_is_2 ... ok
test story_108::test_EC_002_no_finding_at_exact_threshold ... ok
test story_108::test_direct_operate_count_increments_on_control_fc ... ok
test story_108::test_initialize_data_not_restart ... ok
test story_108::test_EC_001_direct_operate_nr_counts_toward_threshold ... ok
test story_108::test_EC_008_wrapping_sub_out_of_order_timestamp_no_panic ... ok
test story_108::test_EC_007_control_then_write_separate_findings_never_cotagged ... ok
test story_108::test_restart_findings_append_in_observation_order ... ok
test story_108::test_summarize_function_code_distribution ... ok
test story_108::test_summarize_zero_flows ... ok
test story_108::test_t0814_emitted_per_occurrence_cold_restart ... ok
test story_108::test_t0814_emitted_per_occurrence_warm_restart ... ok
test story_108::test_t0836_emitted_for_write_fc ... ok
test story_108::test_t1692_001_emitted_at_threshold_plus_one ... ok
test story_108::test_t1692_001_one_shot_guard ... ok
test story_108::test_t1692_001_window_expiry_resets_counter ... ok
test story_108::test_write_fc_not_t1692 ... ok
test story_108::test_max_findings_cap_preserves_first_finding ... ok
test story_108::test_EC_006_cap_restart_counter_still_increments ... ok
test story_108::test_max_findings_counters_updated_when_capped ... ok

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_detection_tests`

## STORY-107 Regression — CLEAN (13/13)

STORY-108 adds detection branches to `src/analyzer/dnp3.rs` (on top of STORY-107's carry-buffer/frame-walk). STORY-107's per-flow bounds contracts remain clean:

```
running 13 tests
test story_107::test_BC_2_15_016_is_master_frame_dir_bit ... ok
test story_107::test_EC_001_partial_frame_in_carry ... ok
test story_107::test_EC_002_two_complete_frames_one_call ... ok
test story_107::test_EC_003_carry_291_plus_2_overflow ... ok
test story_107::test_EC_004_bailed_flow_is_noop ... ok
test story_107::test_EC_005_pending_requests_tie_break_eviction ... ok
test story_107::test_EC_006_invalid_length_byte_increments_parse_errors ... ok
test story_107::test_carry_buffer_cap_at_292 ... ok
test story_107::test_carry_buffer_frame_consumption ... ok
test story_107::test_carry_drain_boundary_min_frame ... ok
test story_107::test_frame_count_increments ... ok
test story_107::test_master_addrs_cap_at_64 ... ok
test story_107::test_pending_requests_eviction_at_256 ... ok

test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_flow_state_tests`

## STORY-106 Regression — CLEAN (36/36)

```
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_parse_core_tests`

## Detection Narrative

### T1692.001 — Unauthorized Control Command Burst (BC-2.15.010)

The detection fires when `direct_operate_count > direct_operate_threshold` (default 10) within a 60-second rolling window. The check is strict `>` (not `>=`), so 10 FCs never fire; the 11th does. The one-shot guard (`direct_operate_emitted=true`) prevents repeated findings within the same window. Window expiry resets all three fields: count=1 (incoming FC seeds the new window), ts=now_ts, emitted=false. All four Control-class FCs (0x03 SELECT, 0x04 OPERATE, 0x05 DIRECT_OPERATE, 0x06 DIRECT_OPERATE_NR) count toward the threshold. `wrapping_sub` prevents panics with out-of-order pcap timestamps when `overflow-checks=true`.

AC-002 pins the exact finding format: `mitre_techniques=["T1692.001"]`, `category=Execution`, `verdict=Likely`, `confidence=Medium`, summary matches BC-2.15.010 PC3 format string, `source_ip=Some(master_ip)` resolved from the DNP3 port-20000 heuristic, `timestamp=Some(DateTime::from_timestamp(ts))`.

### T0814 — Device Restart (BC-2.15.011)

Per-occurrence detection: every COLD_RESTART (0x0D) or WARM_RESTART (0x0E) on a FIR=1 frame pushes one T0814 finding with `confidence=High`. No threshold or one-shot guard — each restart is individually actionable. `restart_event_count` is incremented unconditionally even when `all_findings` is at MAX_FINDINGS (feeds T0827 accumulator in STORY-109). FC 0x0F (INITIALIZE_DATA) is Management-class and does not trigger T0814.

### T0836 — Modify Parameter (BC-2.15.012)

Per-occurrence detection: every WRITE (FC=0x02) on a FIR=1 frame pushes one T0836 finding with `confidence=Medium`. T0836 only — NOT also T1692.001 (ADR-007 Decision 5 separation: DNP3 WRITE is Write-class, not Control-class). 20 WRITEs → 20 T0836 findings, 0 T1692.001 findings.

### MAX_FINDINGS DoS Bound (BC-2.15.022)

The cap check `self.all_findings.len() < MAX_FINDINGS` is evaluated immediately before each push. When the cap is hit mid-sequence, the first finding is preserved and subsequent findings are silently dropped. All per-flow counters (direct_operate_count, restart_event_count, fc_counts, fn_code_counts, frame_count) continue to update regardless of cap — they are never gated behind the findings cap.

### summarize() — FC Distribution (BC-2.15.020)

`Dnp3Analyzer::summarize()` aggregates `fn_code_counts` (global FC distribution), per-flow `direct_operate_count`, `total_frames`, `total_parse_errors`, and `flows_analyzed` into the output `AnalyzerSummary`. The zero-flow case returns an empty-but-present `function_code_distribution` object (not absent). `summarize()` never pushes new findings (invariant 3).

## Tape Sources

All VHS tape scripts are committed alongside recordings. Each tape covers one AC.

| Tape | AC |
|------|----|
| [AC-001-control-fc-counter.tape](AC-001-control-fc-counter.tape) | AC-001 |
| [AC-002-t1692-emission.tape](AC-002-t1692-emission.tape) | AC-002 |
| [AC-003-one-shot-guard.tape](AC-003-one-shot-guard.tape) | AC-003 |
| [AC-004-window-expiry.tape](AC-004-window-expiry.tape) | AC-004 |
| [AC-005-t0814-restart.tape](AC-005-t0814-restart.tape) | AC-005 |
| [AC-006-t0836-write.tape](AC-006-t0836-write.tape) | AC-006 |
| [AC-007-co-emission-ordering.tape](AC-007-co-emission-ordering.tape) | AC-007 |
| [AC-008-max-findings-cap.tape](AC-008-max-findings-cap.tape) | AC-008 |
| [AC-009-counters-when-capped.tape](AC-009-counters-when-capped.tape) | AC-009 |
| [AC-010-summarize.tape](AC-010-summarize.tape) | AC-010 |
| [AC-011-summarize-zero-flows.tape](AC-011-summarize-zero-flows.tape) | AC-011 |
| [AC-012-threshold-semantics.tape](AC-012-threshold-semantics.tape) | AC-012 |
