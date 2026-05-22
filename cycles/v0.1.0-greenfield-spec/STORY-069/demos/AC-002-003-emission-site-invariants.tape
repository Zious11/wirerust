Output AC-002-003-emission-site-invariants.gif
Output AC-002-003-emission-site-invariants.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 13
Set Width 1100
Set Height 500
Set Padding 20
Set Theme "Dracula"
Set Shell "bash"

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-069"
Enter
Sleep 500ms
Show

Type "cargo test --test reporter_tests test_timestamp_always_none_in_all_emission_sites"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_source_ip_set_at_reassembly_sites"
Enter
Sleep 5s

Type "cargo test --test reporter_tests test_source_ip_none_at_http_tls_sites"
Enter
Sleep 5s
Sleep 2s
