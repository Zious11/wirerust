# Red Gate Log — STORY-115

**Branch:** worktree-issue-9-story-115-arp-d3-storm
**Commit:** d068c48
**Date:** 2026-06-15
**Agent:** Test Writer

## Result: RED GATE VERIFIED

All new STORY-115 behavioral tests fail before implementation. Existing tests (68 unit + 3 CLI) pass unaffected.

## Failing Tests (12 unit + 1 integration = 13 total)

| Test | AC | Failure Reason |
|------|----|----------------|
| `analyzer::arp::story_114::story_115::test_storm_first_observation_no_finding` | AC-001 | `storm_counters.len()=0` — insert_storm_counter_lru uncalled; D3 not wired into process_arp |
| `analyzer::arp::story_114::story_115::test_storm_in_window_increments_count` | AC-002 | `storm_counters.get(STORM_MAC)` returns None — detect_storm not wired |
| `analyzer::arp::story_114::story_115::test_storm_finding_emitted_at_threshold` | AC-003 | No D3 finding emitted for 50 frames at ts=100; detect_storm is a todo!() stub |
| `analyzer::arp::story_114::story_115::test_storm_one_shot_guard_prevents_second_finding` | AC-004 | No storm finding emitted at all (detect_storm uncalled) |
| `analyzer::arp::story_114::story_115::test_storm_window_expiry_resets_counter` | AC-005 | `storm_counters` empty; window reset logic not reachable |
| `analyzer::arp::story_114::story_115::test_storm_same_second_denominator_is_1` | AC-006 | No storm finding; denominator formula not implemented |
| `analyzer::arp::story_114::story_115::test_storm_49_below_threshold_50_at_threshold` | AC-007 | 50 frames produce no storm finding; detect_storm uncalled |
| `analyzer::arp::story_114::story_115::test_storm_window_boundary_60_in_window_61_expired` | AC-008 | `storm_counters` empty; boundary comparison not implemented |
| `analyzer::arp::story_114::story_115::test_storm_counter_cap_enforced` | AC-010 | `storm_counters.len()=0` after 4097 MACs; insert_storm_counter_lru uncalled |
| `analyzer::arp::story_114::story_115::test_storm_custom_rate_10` | AC-011 | No storm finding at rate=10; detect_storm not wired with self.storm_rate |
| `analyzer::arp::story_114::story_115::test_summarize_storm_findings_key_non_zero_after_detection` | AC-013 | `storm_findings=0` in summarize(); counter not incremented |
| `analyzer::arp::story_114::story_115::test_d3_finding_has_empty_mitre_techniques` | AC-014 | No D3 finding emitted to assert against |
| `story_115_integration::test_integration_arp_storm_end_to_end` | AC-015 | `findings: []` in JSON output; no D3 storm finding produced via full pipeline |

## Tests That Pass Already (AC-009 + AC-011 x2 + AC-012)

| Test | AC | Status | Reason |
|------|----|--------|--------|
| `test_storm_late_burst_suppression_accepted_limitation` | AC-009 | PASS | Asserts NO finding; vacuously correct against unimplemented stub; continues correct after implementation (accepted limitation per BC-2.16.008 Invariant 2) |
| `story_115_cli::test_cli_arp_storm_rate_parsed` | AC-011 | PASS | `arp_storm_rate` flag already declared in src/cli.rs scaffold (commit 234b739) |
| `story_115_cli::test_cli_arp_storm_rate_default_50` | AC-011 | PASS | `default_value_t = 50` already present in cli.rs scaffold |
| `story_115_cli::test_storm_rate_flag_accepted_without_arp_flag` | AC-012 | PASS | Flag has no `requires("arp")` constraint |

## Files Created / Modified

| File | Action |
|------|--------|
| `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-115/src/analyzer/arp.rs` | Added `mod story_115` sub-module inside `mod story_114` (both are `#[cfg(test)]`) containing 12 unit tests for AC-001..014 |
| `/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-115/tests/bc_2_16_story115_arp_tests.rs` | Created — CLI tests (AC-011, AC-012) and integration test (AC-015) |

## AC → Test Function Map

| AC | Test Function | Location |
|----|---------------|----------|
| AC-001 | `test_storm_first_observation_no_finding` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-002 | `test_storm_in_window_increments_count` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-003 | `test_storm_finding_emitted_at_threshold` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-004 | `test_storm_one_shot_guard_prevents_second_finding` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-005 | `test_storm_window_expiry_resets_counter` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-006 | `test_storm_same_second_denominator_is_1` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-007 | `test_storm_49_below_threshold_50_at_threshold` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-008 | `test_storm_window_boundary_60_in_window_61_expired` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-009 | `test_storm_late_burst_suppression_accepted_limitation` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-010 | `test_storm_counter_cap_enforced` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-011 | `test_cli_arp_storm_rate_parsed`, `test_cli_arp_storm_rate_default_50`, `test_storm_custom_rate_10` | integration + unit |
| AC-012 | `test_storm_rate_flag_accepted_without_arp_flag` | `tests/bc_2_16_story115_arp_tests.rs::story_115_cli` |
| AC-013 | `test_summarize_storm_findings_key_non_zero_after_detection` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-014 | `test_d3_finding_has_empty_mitre_techniques` | `src/analyzer/arp.rs::story_114::story_115` |
| AC-015 | `test_integration_arp_storm_end_to_end` | `tests/bc_2_16_story115_arp_tests.rs::story_115_integration` |
