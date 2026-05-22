Output AC-003-packet-count-and-order.gif
Output AC-003-packet-count-and-order.webm

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

Type "# AC-003: packets Vec contains exactly N entries in file order"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_002_packet_count_and_order -- --nocapture 2>&1"
Enter
Sleep 5s
