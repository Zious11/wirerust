Output EC-001-005-edge-cases.gif
Output EC-001-005-edge-cases.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 13
Set Width 1100
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-069"
Enter
Sleep 500ms
Show

Type "cargo test --test reporter_tests test_bc_2_09_001_ec001_empty_evidence_is_valid"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_bc_2_09_002_ec002_empty_summary_display"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_bc_2_09_002_ec003_esc_byte_in_summary_preserved_in_display"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_bc_2_09_001_ec004_direction_some_server_to_client"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_bc_2_09_002_ec005_reconnaissance_category_display"
Enter
Sleep 5s
Sleep 2s
