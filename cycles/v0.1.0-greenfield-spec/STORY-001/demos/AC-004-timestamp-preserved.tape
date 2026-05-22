Output AC-004-timestamp-preserved.gif
Output AC-004-timestamp-preserved.webm

Require cargo

Set FontSize 14
Set Width 1100
Set Height 480
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

Type "# AC-004: timestamp_secs/usecs preserved for microsecond + nanosecond pcap"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests timestamp_preserved -- --nocapture 2>&1"
Enter
Sleep 5s
