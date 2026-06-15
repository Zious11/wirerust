Output FULL-SUITE-1512-tests.gif
Output FULL-SUITE-1512-tests.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112"
Enter
Sleep 1s
Show

# Full suite: 1512 tests pass (AC-001..010, AC-012 coverage via suite)
Type "# Full suite: 1512 tests, 0 failures (AC-001..010, AC-012)"
Enter
Sleep 0.5s
Type "cargo test --all-targets 2>&1 | tail -5"
Enter
Sleep 30s
