Output AC-006-008-arp-decode-routing.gif
Output AC-006-008-arp-decode-routing.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112 && BINARY=./target/release/wirerust"
Enter
Sleep 1s
Show

# AC-006 / AC-008: ARP frames decoded — dns-remoteshell.pcap skipped_packets = 69
# (was 73 before STORY-112; 4 ARP frames now route to DecodedFrame::Arp)
Type "# AC-006/AC-008: dns-remoteshell.pcap — skipped_packets=69 (4 ARP now decoded)"
Enter
Sleep 0.5s
Type "$BINARY --output-format json --no-color analyze tests/fixtures/dns-remoteshell.pcap"
Enter
Sleep 3s
