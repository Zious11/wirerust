Output AC-011-EC-004-json-stdout.gif
Output AC-011-EC-004-json-stdout.webm

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

Type "# AC-011/EC-004: --json (no path) -> JSON printed to stdout"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/tls.pcap --tls --json 2>/dev/null | head -2"
Enter
Sleep 4s

Sleep 2s
