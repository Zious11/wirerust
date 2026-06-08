Output AC-009-missing-user-agent-no-finding.gif
Output AC-009-missing-user-agent-no-finding.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 500
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-043"
Enter
Sleep 500ms
Show

Type "cargo test --test http_analyzer_tests bc_2_06_043_formalization::test_missing_user_agent_no_finding -- --nocapture 2>&1"
Enter
Sleep 5s
Sleep 2s
