# STORY-138 AC-004: MAX_FINDINGS=10,000 hard cap with dropped_findings counter
#
# Demonstrates: MAX_FINDINGS = 10_000 hard cap on all_findings. When the cap is reached,
#   no new Finding is pushed; dropped_findings increments; per-flow counters (pdu_count,
#   command_counts) still update. One-shot guards are NOT set on cap-suppressed findings.
#
# Tests:
#   test_max_findings_cap                    — all_findings.len() stays at MAX_FINDINGS
#   test_dropped_findings_incremented_at_cap — dropped_findings++ when cap reached
#   test_stats_accumulate_past_max_findings  — pdu_count/counters continue past cap
#
# Traces to: BC-2.17.022 Postconditions 1–5, Invariants 1–5

Output AC-004-dos-guard.gif
Output AC-004-dos-guard.webm

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

Type "# AC-004: MAX_FINDINGS cap + dropped_findings counter (BC-2.17.022)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests 'session_lifecycle::test_max_findings|session_lifecycle::test_dropped_findings|session_lifecycle::test_stats_accumulate' 2>&1 | grep -E 'session_lifecycle::test_max_findings|session_lifecycle::test_dropped_findings|session_lifecycle::test_stats_accumulate|result'"
Enter
Wait
Sleep 2s
