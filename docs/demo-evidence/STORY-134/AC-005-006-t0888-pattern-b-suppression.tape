# STORY-134 AC-134-005 / AC-134-006: T0888 Pattern B error-burst + is_non_enip suppression
#
# AC-005 (formerly AC-134-004): T0888 Pattern B fires when total_error_count > threshold (default 5)
#   - Exactly 5 errors → NO finding (strict > semantics, BC-2.17.014 Invariant 3)
#   - 6th error → ONE Finding (verdict=Possible, confidence=Medium, mitre=["T0888"])
#   - flow.error_rate_emitted = true (one-shot guard; 7th error emits NO additional finding)
#   - threshold=0: first error (count=1 > 0) fires immediately
#
# AC-006 (formerly AC-134-005): is_non_enip suppresses all detections
#   - flow.is_non_enip=true: NO T0846 or T0888 findings regardless of frame content
#
# Also covers: AC-134-006 — aggregate error_count increments across flows
#
# Tests:
#   recon::test_t0888_pattern_b_fires_at_threshold_plus_one
#   recon::test_t0888_pattern_b_one_shot_guard
#   recon::test_t0888_pattern_b_no_fire_at_threshold
#   recon::test_t0888_pattern_b_threshold_zero
#   recon::test_non_enip_flow_suppresses_recon
#   recon::test_aggregate_error_count_increments
# Traces: BC-2.17.014 Pattern B; BC-2.17.010 Precondition 2; BC-2.17.008 Postcondition 2b

Output AC-005-006-t0888-pattern-b-suppression.gif
Output AC-005-006-t0888-pattern-b-suppression.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-134-enip-recon"
Enter
Wait
Show

Type "# AC-134-005/006: T0888 Pattern B error-burst + is_non_enip suppression + aggregate counter"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'recon::test_t0888_pattern_b|recon::test_non_enip|recon::test_aggregate' 2>&1 | grep -E 'test_t0888_pattern_b|test_non_enip|test_aggregate|ok|FAILED|result'"
Enter
Wait
Sleep 2s
