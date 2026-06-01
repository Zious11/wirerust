Output AC-007-AC-010-error-paths.gif
Output AC-007-AC-010-error-paths.webm

Require wirerust

Set Shell "bash"
Set Width 1200
Set Height 700
Set FontSize 16
Set Padding 20
Set Theme "Dracula"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-090"
Enter
Sleep 300ms
Show

Type "# AC-007: non-standard port (UDP/1434, SQL Slammer) → no service hint"
Enter
Sleep 600ms

Type "wirerust summary tests/fixtures/slammer.pcap --no-color 2>&1"
Enter
Sleep 3s

Type "# AC-010: non-existent file → error exit (empty unique_hosts path)"
Enter
Sleep 600ms

Type "wirerust summary tests/fixtures/nonexistent.pcap --no-color 2>&1 || echo 'Error: file not found'"
Enter
Sleep 3s

Sleep 2s
