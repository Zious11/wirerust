# STORY-138 AC-003: pdu_count accumulated per valid frame; split-header double-count fix
#
# Demonstrates: flow.pdu_count increments at the start of each process_pdu call (valid frames
#   only). Also demonstrates the F-W60-P1-001 regression fix: a TCP segment boundary that
#   falls exactly after the 24-byte ENIP header (partial carry stash) must produce exactly
#   ONE command_counts increment — on the call that commits the frame, not the stash call.
#
# Tests:
#   test_pdu_count_increments_on_valid_frame             — valid frame → pdu_count=1
#   test_pdu_count_not_incremented_on_invalid_frame      — invalid frame → pdu_count=0
#   test_command_count_accumulates                       — command_counts from frame-walk
#   test_command_counts_no_double_count_on_split_header  — split at byte 24 → count==1 (not 2)
#
# Traces to: BC-2.17.024 Postconditions 1–5, BC-2.17.025 Postcondition 2 (F-W60-P1-001)

Output AC-003-pdu-count.gif
Output AC-003-pdu-count.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-138-session-summary"
Enter
Wait
Show

Type "# AC-003: pdu_count per valid frame + split-header double-count fix (BC-2.17.024 / F-W60-P1-001)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'session_lifecycle::test_pdu_count|session_lifecycle::test_command_count' 2>&1 | grep -E 'session_lifecycle::test_pdu_count|session_lifecycle::test_command_count|result'"
Enter
Wait
Sleep 2s
