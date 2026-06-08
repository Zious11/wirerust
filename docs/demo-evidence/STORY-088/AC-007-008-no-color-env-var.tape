Output AC-007-008-no-color-env-var.gif
Output AC-007-008-no-color-env-var.webm

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

Type "# AC-008: Default output uses ANSI color codes (escape sequences visible via cat -v)"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze tests/fixtures/dns-remoteshell.pcap --dns | cat -v | head -6"
Enter
Sleep 3s

Type "# AC-007: NO_COLOR= disables all ANSI escape sequences"
Enter
Sleep 500ms

Type "NO_COLOR= ./target/debug/wirerust analyze tests/fixtures/dns-remoteshell.pcap --dns | cat -v | head -6"
Enter
Sleep 3s
