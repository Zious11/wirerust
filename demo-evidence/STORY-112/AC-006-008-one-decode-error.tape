Output AC-006-008-one-decode-error.gif
Output AC-006-008-one-decode-error.webm

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

# AC-006 / AC-008 (error path): one-decode-error.pcap now 0 skipped_packets, 0 warnings
Type "# AC-006/AC-008 (error path): one-decode-error.pcap — 0 skipped_packets, 0 warnings"
Enter
Sleep 0.5s
Type "$BINARY --output-format json --no-color analyze tests/fixtures/one-decode-error.pcap"
Enter
Sleep 3s
