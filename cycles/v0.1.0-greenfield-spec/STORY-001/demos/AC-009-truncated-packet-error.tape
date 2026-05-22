Output AC-009-truncated-packet-error.gif
Output AC-009-truncated-packet-error.webm

Require cargo

Set FontSize 14
Set Width 1100
Set Height 420
Set Theme "Dracula"
Set Padding 18
Set Framerate 24
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-001"
Enter
Sleep 1s
Type "clear"
Enter
Sleep 300ms
Show

Type "# AC-009: truncated mid-stream packet returns Err('Failed to read packet'), no partial Vec"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_007_truncated_packet_error -- --nocapture 2>&1"
Enter
Sleep 5s
