Output AC-140-008-backwards-ts-no-reset.gif
Output AC-140-008-backwards-ts-no-reset.webm

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

Type "# AC-140-008: backwards-ts packet does not reset 60s T1692.001 window (DRIFT-DNP3-CLOCK-001)"
Enter
Sleep 400ms

Type "cargo test --test dnp3_detection_tests test_ac140_008 2>&1 | grep -E 'test |result'"
Enter
Sleep 8s

Sleep 2s
