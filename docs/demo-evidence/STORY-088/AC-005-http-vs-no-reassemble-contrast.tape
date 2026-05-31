Output AC-005-http-vs-no-reassemble-contrast.gif
Output AC-005-http-vs-no-reassemble-contrast.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-088"
Enter
Sleep 500ms
Show

Type "# AC-005a: --http runs HTTP analyzer (ANALYZER: HTTP section present)"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze tests/fixtures/http-ooo.pcap --http"
Enter
Sleep 3s

Type "# AC-005b: --http --no-reassemble skips HTTP (no ANALYZER: HTTP section)"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze tests/fixtures/http-ooo.pcap --http --no-reassemble"
Enter
Sleep 3s
