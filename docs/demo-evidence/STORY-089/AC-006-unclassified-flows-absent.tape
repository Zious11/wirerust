Output AC-006-unclassified-flows-absent.gif
Output AC-006-unclassified-flows-absent.webm

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

Type "# AC-006: --dns --no-reassemble --json -> unclassified_flows key ABSENT"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/dns-remoteshell.pcap --dns --no-reassemble --json 2>/dev/null | python3 -m json.tool | grep unclassified || echo '(key absent -- correct)'"
Enter
Sleep 4s

Sleep 2s
