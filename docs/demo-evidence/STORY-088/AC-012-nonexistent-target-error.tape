Output AC-012-nonexistent-target-error.gif
Output AC-012-nonexistent-target-error.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 400
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-088"
Enter
Sleep 500ms
Show

Type "# AC-012: Non-existent target yields error message + exit code 1"
Enter
Sleep 500ms

Type "./target/debug/wirerust analyze /nonexistent/path.pcap"
Enter
Sleep 2s

Type "echo exit:$?"
Enter
Sleep 2s
