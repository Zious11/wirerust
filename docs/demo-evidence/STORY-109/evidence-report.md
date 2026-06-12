---
story_id: STORY-109
bc: BC-2.15.014, BC-2.15.015, BC-2.15.018, BC-2.15.019, BC-2.15.023, BC-2.15.024
date: 2026-06-11
recorder: demo-recorder
---

# STORY-109 Demo Evidence Report

DNP3 Correlated/Derived + Anomaly Detections — T1691.001, T0827, Broadcast, Unsolicited, ENABLE/DISABLE, Malformed (wave 38)

## AC Coverage (14/14)

| AC | BC | Test Name(s) | Assertion | Recording | Result |
|----|----|-------------|-----------|-----------|--------|
| AC-001 | BC-2.15.014 PC1 | `test_block_event_count_increments_unconditionally` | FC in {0x03,0x04,0x05} timeout → block_event_count=1; FC=0x06 (DIRECT_OPERATE_NR) EXCLUDED — block_event_count stays at 1 | [GIF](AC-001-block-event-count.gif) / [WEBM](AC-001-block-event-count.webm) | PASS |
| AC-002 | BC-2.15.014 PC3 | `test_t1691_001_emitted_at_threshold_3_of_300s` | 3 DIRECT_OPERATE timeouts → exactly ONE T1691.001 at 3rd; Possible/Low; exact summary "DNP3 inferred blocked command: 3 requests without response within 10s (dest=0x0003)"; evidence "block_event_count=3 in correlation window; threshold=3" | [GIF](AC-002-t1691-001-emission.gif) / [WEBM](AC-002-t1691-001-emission.webm) | PASS |
| AC-003 | BC-2.15.014 PC2/INV7 | `test_block_events_not_reset_at_120s` | Block at t=0, block at t=150s (both within 300s) → block_event_count=2; NOT reset at 120s; no T1691.001 (count=2 < threshold=3) | [GIF](AC-003-block-events-300s-window.gif) / [WEBM](AC-003-block-events-300s-window.webm) | PASS |
| AC-004 | BC-2.15.015 PC1/2 | `test_t0827_emitted_at_combined_threshold`, `test_t0827_emitted_when_block_crosses_threshold_after_restarts`, `test_t0827_emitted_when_second_block_crosses_threshold_after_one_restart` | Trace B (2 block + 1 restart = 3) → T0827 Impact/Likely/Medium; tactic IcsImpact; T0827 after T0814 in all_findings; loss_of_control_emitted=true. F-P9-001: block-crossing path (Trace C-rev: 2 restart + 1 block; Trace D: 1 restart + 2 blocks) both fire T0827. | [GIF](AC-004-t0827-combined-threshold.gif) / [WEBM](AC-004-t0827-combined-threshold.webm) | PASS |
| AC-005 | BC-2.15.015 PC3 | `test_correlation_window_expiry_resets_six_fields` | Past 300s: ALL SIX windowed fields reset (restart_event_count, block_event_count, block_finding_emitted_this_window, loss_of_control_emitted, malformed_in_window, malformed_anomaly_emitted); parse_errors NOT reset (lifetime) | [GIF](AC-005-six-field-window-reset.gif) / [WEBM](AC-005-six-field-window-reset.webm) | PASS |
| AC-006 | BC-2.15.015 INV7 | `test_t0827_requires_distinct_events` | 2 COLD_RESTART + 0 blocks = combined 2 < threshold=3 → NO T0827; loss_of_control_emitted=false | [GIF](AC-006-t0827-distinct-events.gif) / [WEBM](AC-006-t0827-distinct-events.webm) | PASS |
| AC-007 | BC-2.15.018 PC1 | `test_broadcast_control_anomaly_fires_for_dest_ffff`, `test_broadcast_read_no_anomaly` | DIRECT_OPERATE (0x05) to dest=0xFFFF → Suspicious/Possible/Medium T1692.001; source_ip/timestamp set; direct_operate_count incremented. READ (0x01) to 0xFFFF → NO T1692.001. | [GIF](AC-007-broadcast-anomaly.gif) / [WEBM](AC-007-broadcast-anomaly.webm) | PASS |
| AC-008 | BC-2.15.018 INV4 | `test_broadcast_and_burst_both_retained` | 11 broadcast DIRECT_OPERATE → >1 T1692.001 findings; Suspicious finding (broadcast anomaly) AND Execution finding (burst threshold) both present — no dedup by technique ID | [GIF](AC-008-broadcast-burst-retained.gif) / [WEBM](AC-008-broadcast-burst-retained.webm) | PASS |
| AC-009 | BC-2.15.019 PC1/2/3 | `test_unsolicited_response_anomaly_no_prior_enable`, `test_unsolicited_response_no_anomaly_after_enable` | FC=0x82 without prior ENABLE → Suspicious/Possible/Low T0814; one-shot guard; second UNSOLICITED_RESPONSE no second finding. ENABLE (FC=0x14) first → subsequent FC=0x82 NOT anomalous. | [GIF](AC-009-unsolicited-anomaly.gif) / [WEBM](AC-009-unsolicited-anomaly.webm) | PASS |
| AC-010 | BC-2.15.023 PC1 | `test_disable_unsolicited_emits_t0814_likely_medium` | FC=0x15 → Execution/Likely/Medium T0814; exact summary "DNP3 DISABLE_UNSOLICITED observed: FC 0x15 from src=0x0001 to dest=0x0003 — alarm suppression / event-blinding primitive"; evidence "FC=0x15 dest=0x0003 src=0x0001"; per-occurrence (2nd FC=0x15 → 2nd finding) | [GIF](AC-010-disable-unsolicited.gif) / [WEBM](AC-010-disable-unsolicited.webm) | PASS |
| AC-011 | BC-2.15.023 PC1 | `test_enable_unsolicited_emits_t0814_possible_low` | FC=0x14 → Execution/Possible/Low T0814; evidence "FC=0x14 dest=0x0003 src=0x0001" (dest before src, no parens); per-occurrence (2nd FC=0x14 → 2nd finding) | [GIF](AC-011-enable-unsolicited.gif) / [WEBM](AC-011-enable-unsolicited.webm) | PASS |
| AC-012 | BC-2.15.024 PC2/3 | `test_malformed_anomaly_at_threshold_3_of_300s` | 3 malformed frames (LENGTH<5) → exactly ONE Anomaly/Possible/Low T0814 at 3rd; summary contains "possible Crain-Sistrunk crash-probe" and "(flow 10.0.0.1→10.0.0.2)"; parse_errors=3 (lifetime); malformed_in_window=3 (windowed); malformed_anomaly_emitted=true | [GIF](AC-012-malformed-anomaly.gif) / [WEBM](AC-012-malformed-anomaly.webm) | PASS |
| AC-013 | BC-2.15.024 INV1 | `test_parse_errors_not_reset_at_window_expiry` | 3 malformed frames → parse_errors=3; advance past 300s → parse_errors STILL 3; malformed_in_window=0 (windowed resets); malformed_anomaly_emitted=false | [GIF](AC-013-parse-errors-lifetime.gif) / [WEBM](AC-013-parse-errors-lifetime.webm) | PASS |
| AC-014 | BC-2.15.014 INV8 | `test_pending_request_timeout_wrapping_sub` | Request at ts=u32::MAX-5, trigger at ts=5: wrapping_sub(5, u32::MAX-5)=11 > 10 → block timeout fires; block_event_count=1; NO PANIC (overflow-checks=true safe) | [GIF](AC-014-wrapping-sub.gif) / [WEBM](AC-014-wrapping-sub.webm) | PASS |

## F-P9-001 Regression Coverage (Block-Crossing T0827 Path)

| Test | Trace | Assertion | Result |
|------|-------|-----------|--------|
| `test_t0827_emitted_when_block_crosses_threshold_after_restarts` | Trace C-rev (2 restart + 1 block = 3) | T0827 fires on block-timeout path when restart_event_count=2; block_event_count=1 < BLOCK_CMD_THRESHOLD=3; T1691.001 NOT fired | PASS |
| `test_t0827_emitted_when_second_block_crosses_threshold_after_one_restart` | Trace D (1 restart + 2 blocks = 3) | T0827 fires on 2nd block timeout when restart_event_count=1; block_event_count=2; combined=3 >= threshold | PASS |

## Full Test Run Output

```
running 34 tests
test story_109::test_BC_2_15_015_t0827_summary_exact_pin ... ok
test story_109::test_BC_2_15_018_broadcast_summary_and_evidence_exact_pin ... ok
test story_109::test_BC_2_15_019_unsolicited_summary_and_evidence_exact_pin ... ok
test story_109::test_BC_2_15_023_enable_unsolicited_summary_exact_pin ... ok
test story_109::test_BC_2_15_024_malformed_evidence_exact_pin ... ok
test story_109::test_EC_001_direct_operate_nr_not_tracked_no_block_event ... ok
test story_109::test_EC_002_select_response_within_timeout_no_block_event ... ok
test story_109::test_EC_004_t0827_one_shot_fourth_restart_no_second ... ok
test story_109::test_EC_005_broadcast_read_no_anomaly ... ok
test story_109::test_EC_006_bailed_flow_disable_unsolicited_no_op ... ok
test story_109::test_EC_007_enable_before_unsolicited_suppresses_anomaly ... ok
test story_109::test_EC_009_fourth_malformed_no_second_t0814 ... ok
test story_109::test_EC_010_single_cold_restart_t0814_no_t0827 ... ok
test story_109::test_block_event_count_increments_unconditionally ... ok
test story_109::test_block_events_not_reset_at_120s ... ok
test story_109::test_broadcast_and_burst_both_retained ... ok
test story_109::test_broadcast_control_anomaly_fires_for_dest_ffff ... ok
test story_109::test_broadcast_read_no_anomaly ... ok
test story_109::test_correlation_window_expiry_resets_six_fields ... ok
test story_109::test_disable_unsolicited_emits_t0814_likely_medium ... ok
test story_109::test_enable_unsolicited_emits_t0814_possible_low ... ok
test story_109::test_malformed_anomaly_at_threshold_3_of_300s ... ok
test story_109::test_parse_errors_not_reset_at_window_expiry ... ok
test story_109::test_pending_request_timeout_wrapping_sub ... ok
test story_109::test_story_109_constants_correct_values ... ok
test story_109::test_story_109_ics_impact_tactic_seeded ... ok
test story_109::test_story_109_is_broadcast_destination_helper ... ok
test story_109::test_t0827_emitted_at_combined_threshold ... ok
test story_109::test_t0827_emitted_when_block_crosses_threshold_after_restarts ... ok
test story_109::test_t0827_emitted_when_second_block_crosses_threshold_after_one_restart ... ok
test story_109::test_t0827_requires_distinct_events ... ok
test story_109::test_t1691_001_emitted_at_threshold_3_of_300s ... ok
test story_109::test_unsolicited_response_anomaly_no_prior_enable ... ok
test story_109::test_unsolicited_response_no_anomaly_after_enable ... ok

test result: ok. 34 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_correlation_tests`

## STORY-108 Regression — CLEAN (26/26)

STORY-109 adds correlated/anomaly detection branches on top of STORY-108's direct detections. All STORY-108 per-flow direct detection contracts remain clean:

```
running 26 tests
test story_108::test_BC_2_15_010_threshold_is_strictly_greater_not_gte ... ok
test story_108::test_BC_2_15_010_asymmetric_port_master_lower_ip_else_branch ... ok
test story_108::test_BC_2_15_010_asymmetric_port_master_upper_ip_if_branch ... ok
test story_108::test_BC_2_15_020_summarize_does_not_push_findings ... ok
test story_108::test_BC_2_15_020_summarize_control_operation_counts_per_flow ... ok
test story_108::test_BC_2_15_020_summarize_includes_parse_errors ... ok
test story_108::test_EC_001_direct_operate_nr_counts_toward_threshold ... ok
test story_108::test_EC_002_no_finding_at_exact_threshold ... ok
test story_108::test_EC_005_two_cold_restarts_restart_event_count_is_2 ... ok
test story_108::test_EC_006_cap_restart_counter_still_increments ... ok
test story_108::test_EC_007_control_then_write_separate_findings_never_cotagged ... ok
test story_108::test_EC_008_wrapping_sub_out_of_order_timestamp_no_panic ... ok
test story_108::test_direct_operate_count_increments_on_control_fc ... ok
test story_108::test_initialize_data_not_restart ... ok
test story_108::test_max_findings_cap_preserves_first_finding ... ok
test story_108::test_max_findings_counters_updated_when_capped ... ok
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

test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_detection_tests`

## STORY-107 Regression — CLEAN (14/14)

```
running 14 tests
test story_107::test_BC_2_15_016_is_master_frame_dir_bit ... ok
test story_107::test_BC_2_16_OBS_P11_1_resync_realign_branch_drain_to_next_sync ... ok
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

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_flow_state_tests`

## STORY-106 Regression — CLEAN (36/36)

```
test result: ok. 36 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Command: `cargo test --test dnp3_parse_core_tests`

## Detection Narrative

### T1691.001 — Inferred Block-Command (BC-2.15.014)

Fires when `flow.block_event_count >= BLOCK_CMD_THRESHOLD = 3` within the shared `CORRELATION_WINDOW_SECS = 300s` and `block_finding_emitted_this_window == false`. The block-timeout scan runs on every `on_data` call: any pending request (dest_addr, app_seq) where `now_ts.wrapping_sub(request_ts) > BLOCK_CMD_TIMEOUT_SECS = 10` is removed and `block_event_count += 1`. FC=0x06 (DIRECT_OPERATE_NR) is never inserted into `pending_requests` because it expects no response by design.

The finding carries `mitre_techniques=["T1691.001"]`, `category=Execution`, `verdict=Possible`, `confidence=Low`. T1691.001 is the active replacement for revoked T0803 in ics-attack-19.1.

### T0827 — Loss of Control (BC-2.15.015)

Fires when `restart_event_count + block_event_count >= T0827_THRESHOLD = 3` AND `loss_of_control_emitted == false` within the 300s window. The check runs on both the restart path (after T0814 emission) AND the block-timeout path (F-P9-001 fix: unconditional `maybe_emit_t0827` call on the block-timeout path regardless of `block_event_count` relative to `BLOCK_CMD_THRESHOLD`).

T0827 uses `MitreTactic::IcsImpact` (NEW enum variant added in this story), `category=Impact`, `verdict=Likely`, `confidence=Medium`. T0827 always appears after the triggering direct finding (BC-2.15.013 ordering). One-shot guard per 300s window.

### Broadcast Anomaly (BC-2.15.018)

`is_broadcast_destination(dest) → dest >= 0xFFFD`. Only fires on Control-class FCs (classify_dnp3_fc returns Control). READ (FC=0x01) to broadcast is legitimate and does NOT trigger the anomaly. The broadcast anomaly finding is `Suspicious/Possible/Medium`. The same frame also increments `direct_operate_count`, so when the burst threshold (BC-2.15.010) is later crossed, a second distinct T1692.001 finding (Execution/Likely/Medium) is emitted — both are retained (no dedup by technique ID alone).

### Unsolicited Anomaly (BC-2.15.019)

FC=0x82 (UNSOLICITED_RESPONSE) without prior ENABLE_UNSOLICITED (FC=0x14) on the same flow AND `response_seen == false` AND `unsolicited_anomaly_emitted == false` emits `Suspicious/Possible/Low T0814`. ENABLE_UNSOLICITED sets `enable_unsolicited_seen=true`, suppressing the anomaly for subsequent FC=0x82 frames.

### DISABLE/ENABLE_UNSOLICITED (BC-2.15.023)

Raw FC check (`app_fc == 0x15` or `app_fc == 0x14`), NOT via classify_dnp3_fc. FC=0x15 (DISABLE_UNSOLICITED): `Execution/Likely/Medium T0814`, per-occurrence. FC=0x14 (ENABLE_UNSOLICITED): `Execution/Possible/Low T0814`, per-occurrence. Evidence format: `"FC=0x{fc:02X} dest={dest:#06X} src={src:#06X}"` (dest before src, no parenthetical FC name).

### Malformed Anomaly (BC-2.15.024) — Two-Counter Model

Each structural-reject path increments BOTH `parse_errors += 1` (lifetime monotonic, never reset) AND `malformed_in_window += 1` (windowed, resets at 300s). When `malformed_in_window >= MALFORMED_ANOMALY_THRESHOLD = 3` AND `malformed_anomaly_emitted == false`: `Anomaly/Possible/Low T0814` with summary containing "possible Crain-Sistrunk crash-probe" and the flow fragment `(flow {src_ip}→{dest_ip})`. One-shot guard per window. The two-counter model was introduced by the adversarial F-C-2 finding (BC-2.15.024 v1.1).

### Shared 300s Correlation Window (BC-2.15.015)

Single `correlation_window_start_ts: u32` per flow. The window-expiry handler fires at the TOP of `on_data` before any detection logic (using `wrapping_sub` for safe u32 arithmetic). On expiry: exactly six fields reset — `restart_event_count`, `block_event_count`, `block_finding_emitted_this_window`, `loss_of_control_emitted`, `malformed_in_window`, `malformed_anomaly_emitted`. `parse_errors` is NEVER reset (BC-2.15.024 Invariant 1 is absolute).

## Tape Sources

All VHS tape scripts are committed alongside recordings. Each tape covers one AC.

| Tape | AC |
|------|----|
| [AC-001-block-event-count.tape](AC-001-block-event-count.tape) | AC-001 |
| [AC-002-t1691-001-emission.tape](AC-002-t1691-001-emission.tape) | AC-002 |
| [AC-003-block-events-300s-window.tape](AC-003-block-events-300s-window.tape) | AC-003 |
| [AC-004-t0827-combined-threshold.tape](AC-004-t0827-combined-threshold.tape) | AC-004 |
| [AC-005-six-field-window-reset.tape](AC-005-six-field-window-reset.tape) | AC-005 |
| [AC-006-t0827-distinct-events.tape](AC-006-t0827-distinct-events.tape) | AC-006 |
| [AC-007-broadcast-anomaly.tape](AC-007-broadcast-anomaly.tape) | AC-007 |
| [AC-008-broadcast-burst-retained.tape](AC-008-broadcast-burst-retained.tape) | AC-008 |
| [AC-009-unsolicited-anomaly.tape](AC-009-unsolicited-anomaly.tape) | AC-009 |
| [AC-010-disable-unsolicited.tape](AC-010-disable-unsolicited.tape) | AC-010 |
| [AC-011-enable-unsolicited.tape](AC-011-enable-unsolicited.tape) | AC-011 |
| [AC-012-malformed-anomaly.tape](AC-012-malformed-anomaly.tape) | AC-012 |
| [AC-013-parse-errors-lifetime.tape](AC-013-parse-errors-lifetime.tape) | AC-013 |
| [AC-014-wrapping-sub.tape](AC-014-wrapping-sub.tape) | AC-014 |
