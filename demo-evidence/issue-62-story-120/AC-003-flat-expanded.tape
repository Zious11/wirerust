# AC-003 / FindingsRender::FlatExpanded (--no-collapse flag)
# Demonstrates the FlatExpanded enum variant: one finding line per raw finding,
# no (xN) suffix, full evidence per finding (pre-v0.8.0 behavior restored).
# Dispatched via exhaustive match arm — no if-chain.

Output AC-003-flat-expanded.gif
Output AC-003-flat-expanded.webm

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

Type "# FindingsRender::FlatExpanded (--no-collapse flag)"
Enter
Sleep 0.4s
Type "# One finding line per raw finding; no (xN) suffix"
Enter
Sleep 0.4s
Type "./target/release/wirerust --no-color analyze --modbus --reassemble --no-collapse tests/fixtures/local-samples/modbus-large.pcap 2>&1 | sed -n '/^FINDINGS/,/^ANALYZER/p' | head -35"
Enter
Sleep 8s

Sleep 2s
