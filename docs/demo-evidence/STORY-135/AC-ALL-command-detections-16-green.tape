# STORY-135 Full command_detections suite — 16/16 green
#
# Covers all 16 tests in mod command_detections across AC-135-001 through AC-135-005:
#   T0858 Stop (0x07) — 4 tests
#   T0816 Reset (0x05) — 3 tests
#   T0836 write-burst — 6 tests
#   is_non_enip suppression — 1 test
#   write_count accumulation + aggregate counter — 2 tests
#
# This tape is the master green-run evidence.
# Traces: BC-2.17.011/012/013; AC-135-001..005

Output AC-ALL-command-detections-16-green.gif
Output AC-ALL-command-detections-16-green.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 760
Set FontSize 14
Set Padding 24
Set Shell "zsh"
Set TypingSpeed 60ms

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-135-enip-command-detections"
Enter
Wait
Show

Type "# STORY-135: All 16 command_detections tests (T0858/T0816/T0836)"
Enter
Sleep 300ms

Type "cargo test --test enip_analyzer_tests command_detections 2>&1 | grep -E 'test command_detections|ok|FAILED|result:'"
Enter
Wait
Sleep 2s
