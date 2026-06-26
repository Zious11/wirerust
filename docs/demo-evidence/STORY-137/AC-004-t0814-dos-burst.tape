# STORY-137 AC-004: T0814 malformed-frame DoS burst detection — windowed, threshold 3
#
# Demonstrates:
#   - test_t0814_fires_at_threshold: 3 malformed frames → exactly one T0814 Anomaly/Possible/Low
#   - test_t0814_does_not_fire_below_threshold: 2 malformed frames → no T0814 (EC-005/006)
#   - test_t0814_one_shot_guard_per_window: 4th malformed frame → no second T0814 (guard set)
#   - test_t0814_refire_after_window_reset: 300s window expires → fresh 3-frame burst fires new T0814
#   - test_parse_errors_not_reset_on_window_expiry: parse_errors (lifetime) NOT reset at 300s
#   - test_t0814_non_enip_not_set_at_threshold: T0814 fires → is_non_enip stays false
#   - test_t0814_fires_on_third_byte_walk_reject: T0814 fires on 3rd structural reject (byte-walk)
#   - test_invalid_frame_increments_malformed_count: structural reject → both counters increment
#   - test_valid_frame_no_malformed_count: valid frame → parse_errors/malformed_in_window unchanged
#
# Traces to: BC-2.17.018 Postconditions 1–5; Invariants 1/3/4

Output AC-004-t0814-dos-burst.gif
Output AC-004-t0814-dos-burst.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-137-enip-frame-walk"
Enter
Wait
Show

Type "# STORY-137 AC-004: T0814 DoS burst detection — windowed threshold 3 (BC-2.17.018)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'frame_walk::test_t0814|frame_walk::test_valid_frame|frame_walk::test_invalid_frame|frame_walk::test_parse_errors' 2>&1 | grep -E 'frame_walk::(test_t0814|test_valid_frame|test_invalid_frame|test_parse_errors)|result'"
Enter
Wait
Sleep 2s
