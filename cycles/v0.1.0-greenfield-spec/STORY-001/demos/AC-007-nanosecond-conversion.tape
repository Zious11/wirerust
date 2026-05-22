Output AC-007-nanosecond-conversion.gif
Output AC-007-nanosecond-conversion.webm

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

Type "# AC-007: ns pcap ts_frac=500_000 -> timestamp_usecs=500 (integer div / 1000)"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_005_nanosecond_resolution_conversion -- --nocapture 2>&1"
Enter
Sleep 5s
