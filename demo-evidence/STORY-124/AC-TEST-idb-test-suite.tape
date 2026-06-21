Output AC-TEST-idb-test-suite.gif
Output AC-TEST-idb-test-suite.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 18
Set Width 1200
Set Height 800
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "clear"
Enter
Sleep 800ms
Show

Type "echo '=== STORY-124 IDB test suite: 27 tests, all ACs ==='"
Enter
Sleep 500ms

Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-124 && cargo test --test bc_2_01_story124_idb_tests 2>&1"
Enter
Sleep 15s
