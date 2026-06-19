# VHS Demo — STORY-119 AC-003 (optional contrast)
# Mode 3: flat collapsed vs flat expanded — proves flat behavior unchanged
# Demonstrates:
#   Default flat collapsed: (x2) suffix, no tactic headers
#   --no-collapse flat expanded: two separate finding lines, no suffix
# Contrast shown back-to-back in same recording (two commands).

Output AC-003-flat-collapsed-vs-expanded.gif
Output AC-003-flat-collapsed-vs-expanded.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 18
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-119"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "# Flat COLLAPSED (default)"
Enter
Sleep 300ms
Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-119/target/release/wirerust analyze tests/fixtures/modbus-write.pcap --modbus --no-color"
Enter
Sleep 8s
Sleep 1s
Type "# Flat EXPANDED (--no-collapse)"
Enter
Sleep 300ms
Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-119/target/release/wirerust analyze tests/fixtures/modbus-write.pcap --modbus --no-color --no-collapse"
Enter
Sleep 8s
Sleep 2s
