Output AC-004-no-reassemble-http-warning.gif
Output AC-004-no-reassemble-http-warning.webm

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

Type "# AC-004: --no-reassemble + --http emits warning on stderr"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze tests/fixtures/http-ooo.pcap --http --no-reassemble"
Enter
Sleep 3s
