# STORY-137 AC-006: command_counts single canonical site — ALL parsed headers incl. Unknown
#
# Demonstrates:
#   - test_command_counts_increments_for_unknown_command: frame with unknown cmd 0xFF00 →
#     command_counts[0xFF00]==1 AND pdu_count==0 (process_pdu NOT called)
#   - test_command_counts_single_site_not_doubled: valid known-cmd frame →
#     command_counts[cmd]==1 (not 2 — no duplicate increment in process_pdu)
#
# Traces to: BC-2.17.016 Postcondition 0 (command_counts increment site);
#            BC-2.17.004 Invariant 3 (Unknown bucket countable)

Output AC-006-command-counts.gif
Output AC-006-command-counts.webm

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

Type "# STORY-137 AC-006: command_counts single canonical site (BC-2.17.016 PC-0 / BC-2.17.004 Inv-3)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'frame_walk::test_command_counts' 2>&1 | grep -E 'frame_walk::test_command_counts|result'"
Enter
Wait
Sleep 2s
