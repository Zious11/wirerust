Output AC-001-accepts-all-five-link-types.gif
Output AC-001-accepts-all-five-link-types.webm

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

Type "# AC-001: PcapSource::from_file accepts ETHERNET, RAW, LINUX_SLL, IPV4, IPV6"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_001_accepts_all_five_link_types -- --nocapture 2>&1"
Enter
Sleep 5s
