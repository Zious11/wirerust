Output docs/demo-evidence/STORY-052/AC-007-map-bounds.gif
Output docs/demo-evidence/STORY-052/AC-007-map-bounds.webm

Set Theme "Catppuccin Mocha"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Padding 16
Set Framerate 24
Set WindowBar Colorful
Set Shell "zsh"

Require cargo

Hide
Type "source ~/.zshrc 2>/dev/null || true"
Enter
Sleep 1s
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-052"
Enter
Sleep 1s
Show

# AC-007: version_counts and ja3_counts bounded at MAX_MAP_ENTRIES=50,000
Type "cargo test --test tls_analyzer_tests test_BC_2_07_001_inv2 -- --nocapture 2>&1"
Enter
Sleep 15s
