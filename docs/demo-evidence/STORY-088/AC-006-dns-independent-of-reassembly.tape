Output AC-006-dns-independent-of-reassembly.gif
Output AC-006-dns-independent-of-reassembly.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 500
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-088"
Enter
Sleep 500ms
Show

Type "# AC-006: DNS runs independently even with --no-reassemble"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze tests/fixtures/dns-remoteshell.pcap --dns --no-reassemble"
Enter
Sleep 3s
