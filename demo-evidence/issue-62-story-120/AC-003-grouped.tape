# AC-003 / FindingsRender::Grouped (--mitre flag)
# Demonstrates the Grouped enum variant: findings are sorted by MITRE ATT&CK
# tactic with section headers and em-dash + technique names.
# Dispatched via exhaustive match arm — no if-chain.

Output AC-003-grouped.gif
Output AC-003-grouped.webm

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

Type "# FindingsRender::Grouped (--mitre flag)"
Enter
Sleep 0.4s
Type "# Tactic headers appear; each finding gets em-dash + technique name"
Enter
Sleep 0.4s
Type "./target/release/wirerust --no-color analyze --modbus --reassemble --mitre tests/fixtures/local-samples/modbus-large.pcap 2>&1 | sed -n '/^FINDINGS/,/^ANALYZER/p' | head -35"
Enter
Sleep 8s

Sleep 2s
