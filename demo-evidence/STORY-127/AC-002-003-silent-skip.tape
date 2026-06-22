Output AC-002-003-silent-skip.gif
Output AC-002-003-silent-skip.webm

Set Shell "bash"
Set Theme "Dracula"
Set Width 1200
Set Height 480
Set FontSize 14
Set FontFamily "Menlo"
Set Padding 20

# AC-002 / BC-2.12.011 PC2: wrong-magic files silently skipped (no error, exit 0).
# AC-003 / BC-2.12.011 PC3 + Inv5: files <4 bytes silently skipped (no panic).
# ERROR PATH demo: these are not routing errors -- they are intentional silent skips.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-127"
Enter
Sleep 1s
Show

Type "bash /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-127/demo-ac002-003-silent-skip.sh"
Enter
Sleep 7s
