---
story: STORY-137
phase: red-gate
date: 2026-06-26
agent: test-writer
---

# Red Gate Log — STORY-137: ENIP Frame-Walk Robustness

## Verdict: PASS

All 21 behavioral tests fail via `todo!()` panic at `src/analyzer/enip.rs:583`.
`test_max_enip_carry_bytes_is_600` (GREEN-by-design constant check) passes.
The Red Gate is valid.

## Test Results

### RED (21 tests — fail via `todo!()` panic at `src/analyzer/enip.rs:583`)

| Test Name | BC Reference | Failure Reason |
|-----------|-------------|---------------|
| `test_carry_buffer_partial_header` | BC-2.17.016 PC-2/3; AC-137-001; EC-003 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_carry_buffer_two_frames_one_segment` | BC-2.17.016 PC-1/3; AC-137-001; EC-002 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_carry_buffer_three_segments_one_frame` | BC-2.17.016 PC-2/3; AC-137-001; EC-003 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_carry_buffer_cap_at_600` | BC-2.17.016 PC-4/Inv-4; AC-137-002; EC-004 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_carry_cap_sets_non_enip` | BC-2.17.016 Inv-4; AC-137-002; EC-004 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_t0814_fires_on_carry_overflow_third_malformed` | BC-2.17.018 EC-007; BC-2.17.016 Inv-4; AC-137-002/004 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_byte_walk_resync_invalid_command` | BC-2.17.016 PC-1; AC-137-003; EC-012 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_oversize_frame_skip_continue` | BC-2.17.016 PC-1; AC-137-003; EC-010 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_oversize_frame_does_not_set_non_enip` | BC-2.17.016 Inv-4; AC-137-003; EC-010 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_non_enip_flag_permanent` | BC-2.17.016 Inv-4; AC-137-003; EC-011 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_non_enip_flag_set_at_carry_cap` | BC-2.17.016 PC-4; AC-137-003; EC-004 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_t0814_fires_at_threshold` | BC-2.17.018 PC-1/2/3/4; AC-137-004; EC-007 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_t0814_one_shot_guard_per_window` | BC-2.17.018 PC-4; AC-137-004; EC-008 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_t0814_does_not_fire_below_threshold` | BC-2.17.018; AC-137-004; EC-006 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_t0814_refire_after_window_reset` | BC-2.17.018 PC-5; AC-137-004; EC-009 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_parse_errors_not_reset_on_window_expiry` | BC-2.17.018 Inv-1; AC-137-004; EC-009 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_t0814_non_enip_not_set_at_threshold` | BC-2.17.016 Inv-4; BC-2.17.018 Inv-4; AC-137-004; HS-117 Case D | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_valid_frame_no_malformed_count` | BC-2.17.016 PC-1; BC-2.17.018 PC-1/2; AC-137-005; EC-001 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_invalid_frame_increments_malformed_count` | BC-2.17.018 PC-1/2; AC-137-005; EC-005 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_command_counts_increments_for_unknown_command` | BC-2.17.016 PC-0; BC-2.17.004 Inv-3; AC-137-006 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |
| `test_command_counts_single_site_not_doubled` | BC-2.17.016 PC-0; BC-2.17.024/025; AC-137-006 | panicked: not yet implemented: STORY-137: frame-walk carry buffer [BC-2.17.016] |

### GREEN-by-design (1 test)

| Test Name | BC Reference | Why it passes now |
|-----------|-------------|------------------|
| `test_max_enip_carry_bytes_is_600` | ADR-010 Decision 3; BC-2.17.016 Inv-1 | Compile-time constant assertion; zero branching, no I/O, no helpers |

## Architecture Note

Tests use `analyzer.flows.get(&key)` to read per-flow state after each `on_data` call.
`EnipAnalyzer.flows: HashMap<FlowKey, EnipFlowState>` was added as required infrastructure
(mirrors Dnp3Analyzer pattern per ADR-007). Production code stub (`on_data`) still carries
`todo!()` at line 583 — all 21 behavioral tests hit this panic before reaching any assertion.

## DF Policy Compliance

- **DF-AC-TEST-NAME-SYNC-001 v2:** All 21 test names match STORY-137.md Test Plan exactly (lines 324-344). No drift found.
- **T0814 field assertions (test_t0814_fires_at_threshold):** category=Anomaly, verdict=Possible, confidence=Low; summary substrings "malformed frames" and "possible crash-probe"; source_ip present; timestamp present; mitre_techniques contains "T0814".
- **Two-counter model (BC-2.17.018 Inv-1):** `test_parse_errors_not_reset_on_window_expiry` verifies parse_errors==4 (lifetime) and malformed_in_window==1 (windowed, reset at 300s).
- **is_non_enip latch exclusivity (BC-2.17.016 Inv-4):** Three separate tests assert is_non_enip NOT set on oversize-frame-skip or T0814 threshold; one asserts it IS set on carry overflow.
- **Ordering constraint (BC-2.17.018 EC-007):** `test_t0814_fires_on_carry_overflow_third_malformed` verifies T0814 finding present AND is_non_enip=true, confirming check_t0814 ran before the latch.
- **cargo clippy --all-targets -- -D warnings:** PASS (zero warnings).
- **cargo fmt --check:** PASS (clean).
