Output AC-001-004-first-decode-error-warns-once.gif
Output AC-001-004-first-decode-error-warns-once.webm

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

Type "# AC-001/004: First decode error warns ONCE; 73 errors -> exactly 1 warning on stderr"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/dns-remoteshell.pcap --dns 2>&1 | head -6"
Enter
Sleep 4s

Sleep 2s
