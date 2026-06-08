Output AC-010-json-to-file.gif
Output AC-010-json-to-file.webm

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

Type "# AC-010: --json <FILE> writes to file; stdout is empty"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/tls.pcap --tls --json /tmp/wirerust-demo.json 2>/dev/null && echo 'stdout: (empty -- file written)'"
Enter
Sleep 4s

Type "head -2 /tmp/wirerust-demo.json"
Enter
Sleep 2s

Sleep 2s
