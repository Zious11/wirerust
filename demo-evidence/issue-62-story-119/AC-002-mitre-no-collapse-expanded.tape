# VHS Demo — STORY-119 AC-002
# Mode 2: --mitre --no-collapse → grouped + EXPANDED (suffix-free)
# Demonstrates:
#   AC-002: --no-collapse dual-scope: suppresses collapse in grouped mode too
#   The key proof: same "Report Server ID" finding appears TWICE (no (x2) suffix),
#   proving --no-collapse restores one-line-per-finding in grouped mode.
#   Tactic headers (## Discovery, ## Impair Process Control) still present.

Output AC-002-mitre-no-collapse-expanded.gif
Output AC-002-mitre-no-collapse-expanded.webm

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

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-119/target/release/wirerust analyze tests/fixtures/modbus-write.pcap --modbus --no-color --mitre --no-collapse"
Enter
Sleep 8s
Sleep 2s
