Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-006-arp-spoof-threshold-flag.gif"
Output "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-114/AC-006-arp-spoof-threshold-flag.webm"

Set Theme "Dracula"
Set FontFamily "Menlo"
Set Width 1200
Set Height 750
Set Padding 20
Set FontSize 14

# AC-006: --arp-spoof-threshold flag accepted (BC-2.16.012)
# Show: (1) help text confirming flag exists with default=3
# (2) analyze with default (flag absent) -> threshold=3 implicit
# (3) analyze with --arp-spoof-threshold 1 -> flag accepted, no error

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114"
Enter
Sleep 500ms
Show

Type "# AC-006: --arp-spoof-threshold flag (BC-2.16.012) — default=3, overridable"
Enter
Sleep 500ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114/target/release/wirerust analyze --help 2>&1 | grep -A3 'arp-spoof-threshold'"
Enter
Sleep 2s

Type "# Run with default threshold (3 implicit)"
Enter
Sleep 300ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114/target/release/wirerust analyze --arp tests/fixtures/dns-remoteshell.pcap --no-color 2>&1 | grep -A12 'ANALYZER: ARP'"
Enter
Sleep 3s

Type "# Run with --arp-spoof-threshold 1 (HIGH on first rebind per AC-005)"
Enter
Sleep 300ms

Type "/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-114/target/release/wirerust analyze --arp --arp-spoof-threshold 1 tests/fixtures/dns-remoteshell.pcap --no-color 2>&1 | grep -A12 'ANALYZER: ARP'"
Enter
Sleep 3s

Sleep 2s
