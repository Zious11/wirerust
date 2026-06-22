# STORY-128 AC-001 / BC-2.01.018 AC-002: Per-file error isolation
# Shows: directory [a-conflict.pcapng (bad, first), b-valid.pcapng (good)]
# -> bad file emits per-file error, good file still processed, batch completes.

Output AC-001-per-file-isolation.gif
Output AC-001-per-file-isolation.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 680
Set FontSize 15
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type `alias wirerust="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-128/target/debug/wirerust"`
Enter
Wait
Type `python3 /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-128/make-fixtures-iso.py`
Enter
Wait
Show

Type `alias wirerust="/Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-128/target/debug/wirerust"`
Enter
Wait

Type "# AC-001: bad file (a-conflict) sorts FIRST -- good file (b-valid) still runs"
Enter
Sleep 300ms

Type "ls /tmp/demo-128-iso/"
Enter
Wait
Sleep 400ms

Type "wirerust analyze /tmp/demo-128-iso/ --no-color"
Enter
Wait
Sleep 300ms
Type "echo exit:$?"
Enter
Wait
Sleep 2s
