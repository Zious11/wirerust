# STORY-138 AC-005: summarize() produces enip_summary with BC-2.17.021 canonical schema
#
# Demonstrates: EnipAnalyzer::summarize() produces an enip_summary JSON object with
#   exactly 7 canonical keys: command_distribution, total_pdu_count, parse_errors
#   (NOT total_parse_errors), write_count, error_count, flows_analyzed, dropped_findings.
#   Zero-flow case produces a valid JSON object with all counts at 0.
#
# Tests:
#   test_summarize_produces_enip_summary     — 7 canonical keys present in output
#   test_summary_parse_errors_key_canonical  — key is "parse_errors" NOT "total_parse_errors"
#   test_summary_zero_flow_case              — 0 flows → all counts 0; JSON object present
#   test_summary_dropped_findings            — dropped_findings reflected in summary
#   test_summary_flows_analyzed_nonzero      — flows_analyzed >= 1 after one flow closed
#
# Traces to: BC-2.17.021 Postconditions 1–4, Invariants 1–4

Output AC-005-summarize.gif
Output AC-005-summarize.webm

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

Type "# AC-005: summarize() → enip_summary with canonical schema (BC-2.17.021)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'session_lifecycle::test_summarize|session_lifecycle::test_summary' 2>&1 | grep -E 'session_lifecycle::test_summarize|session_lifecycle::test_summary|result'"
Enter
Wait
Sleep 2s
