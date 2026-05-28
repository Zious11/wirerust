Output docs/demo-evidence/STORY-042/AC-003-per-request-firing.gif
Output docs/demo-evidence/STORY-042/AC-003-per-request-firing.webm

Require cargo

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Framerate 30

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-042"
Enter
Sleep 1s
Show

Type "cargo test --test http_analyzer_tests 'bc_2_06_story042_formalization::test_path_traversal_fires_per_request' -- --nocapture 2>&1"
Enter
Sleep 15s
