Output AC-005-unclassified-flows-present.gif
Output AC-005-unclassified-flows-present.webm

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

Type "# AC-005: unclassified_flows injected into reassembly summary (expect 8)"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/dns-remoteshell.pcap --http --json 2>/dev/null | python3 -m json.tool | grep unclassified"
Enter
Sleep 5s

Sleep 2s
