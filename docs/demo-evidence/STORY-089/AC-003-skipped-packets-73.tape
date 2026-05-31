Output AC-003-skipped-packets-73.gif
Output AC-003-skipped-packets-73.webm

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

Type "# AC-003: skipped_packets == total_decode_errors (expect 73)"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/dns-remoteshell.pcap --dns --json 2>/dev/null | python3 -m json.tool | grep skipped"
Enter
Sleep 4s

Sleep 2s
