Output AC-006-011-verdict-confidence-display.gif
Output AC-006-011-verdict-confidence-display.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 13
Set Width 1100
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-069"
Enter
Sleep 500ms
Show

Type "cargo test --test reporter_tests test_verdict_display_likely"
Enter
Sleep 4s

Type "cargo test --test reporter_tests test_verdict_display_unlikely"
Enter
Sleep 4s

Type "cargo test --test reporter_tests test_verdict_display_inconclusive"
Enter
Sleep 4s

Type "cargo test --test reporter_tests test_confidence_display_high"
Enter
Sleep 4s

Type "cargo test --test reporter_tests test_confidence_display_medium"
Enter
Sleep 4s

Type "cargo test --test reporter_tests test_confidence_display_low"
Enter
Sleep 4s
Sleep 2s
