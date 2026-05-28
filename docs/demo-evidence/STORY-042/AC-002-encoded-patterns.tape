Output docs/demo-evidence/STORY-042/AC-002-encoded-patterns.gif
Output docs/demo-evidence/STORY-042/AC-002-encoded-patterns.webm

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

Type "cargo test --test http_analyzer_tests 'bc_2_06_story042_formalization::test_BC_2_06_005_encoded_traversal_four_patterns' -- --nocapture 2>&1"
Enter
Sleep 15s
