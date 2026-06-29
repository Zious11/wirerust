# STORY-137 AC-003: Byte-walk resync on unknown command + oversized-frame-skip
#
# Demonstrates:
#   - test_byte_walk_resync_to_valid_frame_same_segment: garbage byte + valid frame in one
#     segment → valid frame processed same call (EC-012)
#   - test_oversize_frame_skip_then_valid_frame_processed: oversized frame + trailing valid
#     frame → trailing frame processed; is_non_enip=false (EC-010)
#   - test_byte_walk_resync_invalid_command: lone unknown command → byte-walk, no process_pdu
#   - test_oversize_frame_skip_continue: oversized declared frame → frame-skip, continue
#   - test_oversize_frame_does_not_set_non_enip: oversized frame → is_non_enip stays false
#   - test_non_enip_not_latched_at_carry_cap: carry-cap boundary → is_non_enip NOT set by
#     normal carry stash (RULING-137-002)
#   - test_non_enip_flag_permanent: once is_non_enip=true, on_data is immediate no-op
#
# Traces to: BC-2.17.016 Postcondition 1; BC-2.17.016 Invariant 4

Output AC-003-byte-walk-resync.gif
Output AC-003-byte-walk-resync.webm

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

Type "# STORY-137 AC-003: byte-walk resync + oversized-frame-skip (BC-2.17.016 Post-1)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'frame_walk::test_byte_walk|frame_walk::test_oversize|frame_walk::test_non_enip' 2>&1 | grep -E 'frame_walk::(test_byte_walk|test_oversize|test_non_enip)|result'"
Enter
Wait
Sleep 2s
