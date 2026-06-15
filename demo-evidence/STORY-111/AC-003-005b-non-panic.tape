Output AC-003-005b-non-panic.gif
Output AC-003-005b-non-panic.webm

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 13
Set Width 1200
Set Height 700
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-111"
Enter
Sleep 1s
Show

Type "# AC-003: non-IP/non-ARP frame returns No-IP-layer error (unchanged path)"
Enter
Sleep 0.3s
Type "# AC-005b: ARP-shaped input does not panic (non-panicking placeholder)"
Enter
Sleep 0.3s
Type "cargo test test_decode_non_ip_non_arp_frame_returns_no_ip_error test_decode_arp_shaped_input_does_not_panic 2>&1 | grep -E '(test |result)'"
Enter
Sleep 10s
