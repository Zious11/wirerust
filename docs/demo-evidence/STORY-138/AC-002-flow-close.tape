# STORY-138 AC-002: on_flow_close removes flow state and folds counters into aggregates
#
# Demonstrates: on_flow_close removes EnipFlowState from flows map, folds pdu_count /
#   parse_errors / command_counts into aggregate counters, increments flows_analyzed,
#   and is a no-op (no panic) for unknown flow keys (BC-2.17.017).
#
# Tests:
#   test_flow_close_removes_state                  — HashMap::remove succeeds; flow gone
#   test_flow_close_folds_pdu_count                — total_pdu_count += flow.pdu_count
#   test_flow_close_folds_parse_errors             — parse_errors (aggregate) += flow.parse_errors
#   test_flow_close_unknown_key_no_panic           — unknown key → no-op, no panic
#   test_flows_analyzed_incremented_on_flow_close  — flows_analyzed == 1 after one close
#
# Traces to: BC-2.17.017 Postconditions 1–6

Output AC-002-flow-close.gif
Output AC-002-flow-close.webm

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

Type "# AC-002: on_flow_close — removes state, folds counters, flows_analyzed++ (BC-2.17.017)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'session_lifecycle::test_flow_close|session_lifecycle::test_flows_analyzed' 2>&1 | grep -E 'session_lifecycle::test_flow_close|session_lifecycle::test_flows_analyzed|result'"
Enter
Wait
Sleep 2s
