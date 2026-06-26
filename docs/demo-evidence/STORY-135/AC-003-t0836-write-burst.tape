# STORY-135 AC-135-003: CIP write-attribute burst within 1s window emits T0836 (Modify Parameter)
#
# AC-003: SetAttribute requests (0x02/0x04/0x10) accumulating in write_count_in_window within
#          a 1-second sliding window; fires when count strictly exceeds enip_write_burst_threshold
#          (default 50) AND write_burst_emitted==false
#          → ONE Finding (category=Execution, verdict=Likely, confidence=Medium,
#            mitre_techniques=["T0836"],
#            summary="CIP write-class service burst: {count} SetAttribute operations in 1s window
#            (threshold {threshold}) — possible parameter modification attack (T0836)")
#   - Strict > semantics: 51st write fires at default threshold=50 (EC-008)
#   - One-shot guard: 52nd write in same window → no additional finding (EC-009)
#   - Window reset after 1s → T0836 can re-fire in new window (EC-010)
#   - threshold=0: first write fires (EC-007)
#
# Tests:
#   command_detections::test_t0836_burst_fires_at_threshold_plus_one
#   command_detections::test_t0836_burst_one_shot_guard
#   command_detections::test_t0836_no_fire_at_threshold
#   command_detections::test_t0836_threshold_zero_fires_on_first_write
#   command_detections::test_t0836_window_resets_after_1s
#   command_detections::test_t0836_custom_threshold
# Traces: BC-2.17.012; EC-007/008/009/010

Output AC-003-t0836-write-burst.gif
Output AC-003-t0836-write-burst.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-135-enip-command-detections"
Enter
Wait
Show

Type "# AC-135-003: T0836 write-burst (1s window, strict >, one-shot guard) (BC-2.17.012)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'command_detections::test_t0836' 2>&1 | grep -E 'test_t0836|ok|FAILED|result'"
Enter
Wait
Sleep 2s
