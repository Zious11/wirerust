Output AC-009-010-011-test-suite.gif
Output AC-009-010-011-test-suite.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 12
Set Width 1000
Set Height 580
Set Theme "Dracula"
Set Padding 20

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-125"
Enter
Type "clear"
Enter
Show

Type "# STORY-125: 20 EPB/timestamp tests -- AC-009 1000x bug, AC-010/011 tsresol"
Enter
Sleep 400ms
Type "cargo test --test bc_2_01_story125_epb_tests --no-color 2>&1 | tail -28"
Enter
Sleep 25s
