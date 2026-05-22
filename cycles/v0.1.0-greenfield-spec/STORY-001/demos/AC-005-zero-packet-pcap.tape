Output AC-005-zero-packet-pcap.gif
Output AC-005-zero-packet-pcap.webm

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

Type "# AC-005: header-only (zero-packet) pcap returns Ok with empty Vec, no error"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_003_zero_packet_pcap -- --nocapture 2>&1"
Enter
Sleep 5s
