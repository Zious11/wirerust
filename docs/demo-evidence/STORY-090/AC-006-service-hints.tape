Output AC-006-service-hints.gif
Output AC-006-service-hints.webm

Require wirerust

Set Shell "bash"
Set Width 1200
Set Height 700
Set FontSize 16
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-090"
Enter
Sleep 300ms
Show

Type "# AC-006: app_protocol_hint DNS+HTTP — dns-remoteshell.pcap (ports 53, 80)"
Enter
Sleep 600ms

Type "wirerust summary tests/fixtures/dns-remoteshell.pcap --no-color 2>&1"
Enter
Sleep 4s

Type "# AC-006: TLS fixture — port 443 → TLS service hint"
Enter
Sleep 600ms

Type "wirerust summary tests/fixtures/tls.pcap --no-color 2>&1"
Enter
Sleep 4s

Sleep 2s
