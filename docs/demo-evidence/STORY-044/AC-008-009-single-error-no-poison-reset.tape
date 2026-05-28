Output docs/demo-evidence/STORY-044/AC-008-009-single-error-no-poison-reset.gif
Output docs/demo-evidence/STORY-044/AC-008-009-single-error-no-poison-reset.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-044 && clear"
Enter
Sleep 2s
Show

Type "cargo test --test http_analyzer_tests test_single_error_does_not_poison test_error_count_resets_on_success -- --nocapture"
Enter
Sleep 10s
