Output AC-009-unique-hosts.gif
Output AC-009-unique-hosts.webm

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

Type "# AC-009: unique_hosts() sorted deduplicated list via --hosts flag (BC-2.12.020)"
Enter
Sleep 600ms

Type "wirerust summary tests/fixtures/dns-remoteshell.pcap --hosts --no-color 2>&1"
Enter
Sleep 4s

Sleep 2s
