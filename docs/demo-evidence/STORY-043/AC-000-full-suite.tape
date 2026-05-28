Output AC-000-full-suite.gif
Output AC-000-full-suite.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-043"
Enter
Sleep 500ms
Show

Type "cargo test --test http_analyzer_tests bc_2_06_043_formalization -- --nocapture 2>&1"
Enter
Sleep 8s
