Output AC-007-EC-005-json-wins-over-output-format.gif
Output AC-007-EC-005-json-wins-over-output-format.webm

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

Type "# AC-007/EC-005: --json wins over --output-format csv (output starts with '{')"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/tls.pcap --tls --json --output-format csv 2>/dev/null | head -2"
Enter
Sleep 4s

Sleep 2s
