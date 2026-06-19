# VHS Demo — STORY-122 AC-006
# Mode 2: default (no flags) → {Flat, Collapsed}
# Demonstrates: flat render path, (x2) collapse suffix on repeated findings,
# no tactic group headers, abbreviated MITRE IDs without em-dash names.
# Byte-identical to v0.9.0 FindingsRender::FlatCollapsed behavior.

Output AC-006-mode2-default-flat-collapsed.gif
Output AC-006-mode2-default-flat-collapsed.webm

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

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-122/target/release/wirerust analyze tests/fixtures/modbus-write.pcap --all --no-color"
Enter
Sleep 8s
Sleep 2s
