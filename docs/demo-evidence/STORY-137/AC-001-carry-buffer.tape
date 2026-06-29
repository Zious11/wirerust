# STORY-137 AC-001: Carry buffer accumulates partial ENIP frames across TCP segments
#
# Demonstrates: EnipFlowState.carry holds leftover bytes from partial TCP segments.
#   - test_carry_buffer_partial_header: split after 10 bytes → carry holds 10 bytes
#   - test_carry_buffer_two_frames_one_segment: two full frames in one segment → both processed
#   - test_carry_buffer_three_segments_one_frame: frame split across 3 segments → processed on 3rd
#   - test_multi_call_carry_residue_counting: multi-call residue → carry accumulates, parse_errors count
#   - test_max_enip_carry_bytes_is_600: MAX_ENIP_CARRY_BYTES compile-time constant = 600
#
# Traces to: BC-2.17.016 Postconditions 1–3

Output AC-001-carry-buffer.gif
Output AC-001-carry-buffer.webm

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

Type "# STORY-137 AC-001: carry buffer — partial frames across TCP segments (BC-2.17.016)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'frame_walk::test_carry_buffer|frame_walk::test_max_enip' 2>&1 | grep -E 'frame_walk::test_carry|frame_walk::test_max|result'"
Enter
Wait
Sleep 2s
