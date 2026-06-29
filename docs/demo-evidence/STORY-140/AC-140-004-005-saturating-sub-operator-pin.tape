Output AC-140-004-005-saturating-sub-operator-pin.gif
Output AC-140-004-005-saturating-sub-operator-pin.webm

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

Type "# AC-140-004/005: saturating_sub + strict > operator pin — VP-036 proptest suite"
Enter
Sleep 400ms

Type "cargo test --test dnp3_detection_tests vp036 2>&1 | grep -E 'test |result'"
Enter
Sleep 15s

Sleep 2s
