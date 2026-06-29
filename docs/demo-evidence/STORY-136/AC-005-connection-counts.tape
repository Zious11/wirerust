# STORY-136 AC-136-005: open_connection_count and close_connection_count tracked in flow state
#
# AC-005: EnipFlowState.open_connection_count increments on each ForwardOpen (or LargeForwardOpen)
#          request; close_connection_count increments on each ForwardClose request.
#          Counts persist regardless of MAX_FINDINGS cap — feed STORY-138 session summary.
#
# Tests:
#   connection_lifecycle::test_connection_counts_tracked
# Traces: BC-2.17.015 Invariant 3

Output AC-005-connection-counts.gif
Output AC-005-connection-counts.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-136-enip-lifecycle"
Enter
Wait
Show

Type "# AC-136-005: open_connection_count / close_connection_count tracked in EnipFlowState"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'connection_lifecycle::test_connection_counts_tracked' 2>&1 | grep -E 'test_connection_counts|ok|FAILED|result'"
Enter
Wait
Sleep 2s
