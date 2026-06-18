# AC-003 / FindingsRender::FlatCollapsed — default mode (no flags)
# Demonstrates the FlatCollapsed enum variant: identical findings are collapsed
# into counted groups with (xN) suffix. This is the v0.9.0 default behavior,
# now dispatched via exhaustive match arm instead of if-chain.

Output AC-003-flat-collapsed.gif
Output AC-003-flat-collapsed.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 620
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0
Set Framerate 30

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-120"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 0.3s
Show

Type "# FindingsRender::FlatCollapsed (default -- no flags)"
Enter
Sleep 0.4s
Type "# Repeated findings are collapsed with (xN) suffix"
Enter
Sleep 0.4s
Type "./target/release/wirerust --no-color analyze --modbus --reassemble tests/fixtures/local-samples/modbus-large.pcap 2>&1 | sed -n '/^FINDINGS/,/^ANALYZER/p' | head -35"
Enter
Sleep 8s

Sleep 2s
