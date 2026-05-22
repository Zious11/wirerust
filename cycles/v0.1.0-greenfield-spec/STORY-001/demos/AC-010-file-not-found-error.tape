Output AC-010-file-not-found-error.gif
Output AC-010-file-not-found-error.webm

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

Type "# AC-010: from_file on non-existent path returns Err('Failed to open') with path"
Enter
Sleep 800ms

Type "cargo test --test bc_2_01_story001_tests test_BC_2_01_008_file_not_found_error -- --nocapture 2>&1"
Enter
Sleep 5s
