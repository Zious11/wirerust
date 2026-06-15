Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-015-016-arp-gate-with.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-015-016-arp-gate-with.webm"

Set Theme "Dracula"
Set Width 1200
Set Height 750
Set Padding 20
Set FontSize 14

# AC-015/016: WITH --arp flag — "analyzers" array contains ARP entry with 11 keys
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113"
Enter
Sleep 500ms
Show

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113/target/release/wirerust analyze tests/fixtures/dns-remoteshell.pcap --arp --json 2>/dev/null"
Enter
Sleep 3s
