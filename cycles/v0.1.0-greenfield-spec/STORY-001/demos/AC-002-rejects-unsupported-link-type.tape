Output AC-002-rejects-unsupported-link-type.gif
Output AC-002-rejects-unsupported-link-type.webm

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

Type "# AC-002: IEEE 802.11 (link 105) returns Err with 'Unsupported pcap link type' + 'IEEE802_11'"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_001_rejects_unsupported_link_type -- --nocapture 2>&1"
Enter
Sleep 5s
