Output BC-parity-summary-subcommand.gif
Output BC-parity-summary-subcommand.webm

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

Type "# BC-2.12.014/016/017 parity: summary subcommand resolves format + counts decode errors"
Enter
Sleep 600ms

Type "wirerust summary tests/fixtures/dns-remoteshell.pcap --json 2>/dev/null | python3 -m json.tool | grep skipped"
Enter
Sleep 4s

Sleep 2s
