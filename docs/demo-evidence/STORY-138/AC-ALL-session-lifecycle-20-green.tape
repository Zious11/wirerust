# STORY-138 AC-ALL: Master green-run — all 20 session_lifecycle tests
#
# Demonstrates: full mod session_lifecycle suite — 20/20 passing.
# Covers all 5 acceptance criteria:
#   AC-138-001 (BC-2.17.025): session handshake classify, pdu_count, no finding
#   AC-138-002 (BC-2.17.017): on_flow_close removes state, folds counters, flows_analyzed++
#   AC-138-003 (BC-2.17.024): pdu_count per valid frame, split-header fix (F-W60-P1-001)
#   AC-138-004 (BC-2.17.022): MAX_FINDINGS=10,000 cap + dropped_findings counter
#   AC-138-005 (BC-2.17.021): summarize() → enip_summary with 7 canonical keys

Output AC-ALL-session-lifecycle-20-green.gif
Output AC-ALL-session-lifecycle-20-green.webm

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

Type "# STORY-138 master: all 20 session_lifecycle tests — BC-2.17.025/017/024/022/021"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests session_lifecycle 2>&1 | grep -E 'session_lifecycle::|result'"
Enter
Wait
Sleep 2s
