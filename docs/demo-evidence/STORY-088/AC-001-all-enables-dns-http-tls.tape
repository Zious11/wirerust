Output AC-001-all-enables-dns-http-tls.gif
Output AC-001-all-enables-dns-http-tls.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-088"
Enter
Sleep 500ms
Show

Type "# AC-001: --all enables dns + http + tls analyzers"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze tests/fixtures/tls.pcap --all"
Enter
Sleep 3s
