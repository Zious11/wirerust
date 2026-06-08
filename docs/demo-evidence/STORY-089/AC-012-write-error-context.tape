Output AC-012-write-error-context.gif
Output AC-012-write-error-context.webm

Require wirerust

Set Shell "bash"
Set Width 1200
Set Height 700
Set FontSize 16
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-089"
Enter
Sleep 300ms
Show

Type "# AC-012: Write error -> 'Failed to write JSON output to <path>'"
Enter
Sleep 600ms

Type "wirerust analyze tests/fixtures/tls.pcap --tls --json /nonexistent-dir/out.json 2>&1 | head -3"
Enter
Sleep 4s

Sleep 2s
