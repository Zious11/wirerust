Output AC-140-001-carry-direction-isolation.gif
Output AC-140-001-carry-direction-isolation.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "export PATH=/Users/zious/.cargo/bin:$PATH && cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/dnp3-direction-clock"
Enter
Sleep 1s
Show

Type "# AC-140-001: carry_c2s / carry_s2c direction isolation (DRIFT-DNP3-DIRECTION-001)"
Enter
Sleep 400ms

Type "cargo test --test dnp3_detection_tests test_ac140_001 2>&1 | grep -E 'test |result'"
Enter
Sleep 8s

Sleep 2s
