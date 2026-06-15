# VHS Demo — STORY-115 AC-001 through AC-015 (unit + integration tests)
# Shows cargo test storm run: 13 unit + 1 integration test passing

Output AC-TEST-storm-unit-tests.gif
Output AC-TEST-storm-unit-tests.webm

Require cargo

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-115"
Enter
Sleep 300ms
Type "clear"
Enter
Sleep 300ms
Show

# Show storm-specific unit tests (AC-001 through AC-010, and AC-013/AC-014)
Type "cargo test --all-targets test_storm 2>&1 | grep -E 'test test_storm|test result.*passed' | head -20"
Enter
Sleep 5s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# Show CLI flag tests (AC-011, AC-012)
Type "cargo test --all-targets test_cli_arp_storm test_storm_rate_flag 2>&1 | grep -E 'test story_115|test result.*passed' | head -10"
Enter
Sleep 5s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# Show end-to-end integration test (AC-015)
Type "cargo test --all-targets test_integration_arp_storm 2>&1 | grep -E 'test story_115_integration|test result.*passed' | head -5"
Enter
Sleep 8s

Sleep 2s
