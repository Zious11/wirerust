Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-003-test-evidence.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-113/AC-003-test-evidence.webm"

Set Theme "Dracula"
Set Width 1200
Set Height 700
Set Padding 20
Set FontSize 13

# AC-003/009/011 (D2/D11/D12) unit test evidence — 21 BC_2_16 tests all pass
Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-113"
Enter
Sleep 500ms
Show

Type "cargo test arp --color always 2>&1 | grep -E 'test analyzer::arp|21 passed'"
Enter
Sleep 15s
