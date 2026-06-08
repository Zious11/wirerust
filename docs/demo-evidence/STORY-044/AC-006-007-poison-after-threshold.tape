Output docs/demo-evidence/STORY-044/AC-006-007-poison-after-threshold.gif
Output docs/demo-evidence/STORY-044/AC-006-007-poison-after-threshold.webm

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

Type "cargo test --test http_analyzer_tests test_parse_error_poisons_direction_after_threshold test_poison_is_per_direction -- --nocapture"
Enter
Sleep 10s
