# VHS Demo — STORY-119 AC-001 + AC-006 + AC-011 + AC-018
# Mode 1: --mitre alone → grouped + COLLAPSED (NEW DEFAULT)
# Demonstrates:
#   AC-001: --mitre defaults to collapsed grouped render
#   AC-006: (xN) count suffix appears on repeated findings (x2 visible)
#   AC-011: K=3 evidence lines per group (at most 3 > lines per group)
#   AC-018: MITRE line uses em-dash format "T0888 — Remote System Information Discovery"
# The key proof: "Report Server ID (FC 0x11) from unit 1 (x2)" with tactic header "## Discovery"

Output AC-001-mitre-grouped-collapsed.gif
Output AC-001-mitre-grouped-collapsed.webm

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

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-119/target/release/wirerust analyze tests/fixtures/modbus-write.pcap --modbus --no-color --mitre"
Enter
Sleep 8s
Sleep 2s
