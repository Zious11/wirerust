Output AC-001-012-tests.gif
Output AC-001-012-tests.webm

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-112"
Enter
Sleep 1s
Show

# AC-001..012: All 15 STORY-112 ARP tests passing
Type "# AC-001..012: All 15 STORY-112 ARP unit tests"
Enter
Sleep 0.5s
Type "cargo test test_BC_2_16 2>&1 | tail -25"
Enter
Sleep 8s
