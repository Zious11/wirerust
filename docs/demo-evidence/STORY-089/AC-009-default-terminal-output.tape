Output AC-009-default-terminal-output.gif
Output AC-009-default-terminal-output.webm

Require wirerust

Set Shell "bash"
Set Width 1200
Set Height 700
Set FontSize 16
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-089"
Enter
Sleep 300ms
Show

Type "# AC-009: No format flags -> terminal WIRERUST TRIAGE REPORT"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/tls.pcap --tls 2>/dev/null | head -6"
Enter
Sleep 4s

Sleep 2s
