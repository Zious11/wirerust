Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-015-016-arp-gate-without.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-015-016-arp-gate-without.webm"

Set Theme "Dracula"
Set Width 1200
Set Height 600
Set Padding 20
Set FontSize 14

# AC-015/016: WITHOUT --arp flag — "analyzers" array is [] (gate enforced)
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113"
Enter
Sleep 500ms
Show

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113/target/release/wirerust analyze tests/fixtures/dns-remoteshell.pcap --json 2>/dev/null"
Enter
Sleep 3s
