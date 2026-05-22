Output AC-004-005-finding-display.gif
Output AC-004-005-finding-display.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 13
Set Width 1100
Set Height 420
Set Padding 20
Set Theme "Dracula"
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-069"
Enter
Sleep 500ms
Show

Type "cargo test --test reporter_tests test_finding_display_format"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_finding_display_preserves_raw_summary"
Enter
Sleep 5s
Sleep 2s
