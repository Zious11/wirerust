Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-015-016-arp-teardrop.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-015-016-arp-teardrop.webm"

Set Theme "Dracula"
Set Width 1200
Set Height 750
Set Padding 20
Set FontSize 14

# AC-015/016: teardrop.cap — 5 ARP frames, 2 bindings, 4 requests + 1 reply
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113"
Enter
Sleep 500ms
Show

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113/target/release/wirerust analyze tests/fixtures/teardrop.cap --arp --json 2>/dev/null"
Enter
Sleep 3s
