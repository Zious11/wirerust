# VHS Demo — STORY-122 AC-006
# Mode 3: --no-collapse → {Flat, Expanded}
# Demonstrates: flat render path, one line per finding (no collapse),
# no (xN) suffix, individual finding entries preserved.
# Byte-identical to v0.9.0 FindingsRender::FlatExpanded behavior.

Output AC-006-mode3-flat-expanded.gif
Output AC-006-mode3-flat-expanded.webm

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
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-122"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 500ms
Show

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-122/target/release/wirerust analyze tests/fixtures/modbus-write.pcap --all --no-color --no-collapse"
Enter
Sleep 8s
Sleep 2s
