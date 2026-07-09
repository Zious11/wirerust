Output docs/demo-evidence/STORY-160/AC-160-005-terminal-display-unchanged.gif
Output docs/demo-evidence/STORY-160/AC-160-005-terminal-display-unchanged.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-005: fmt::Display implementations are NOT modified; terminal tokens (LIKELY, HIGH)
# and ThreatCategory PascalCase remain unchanged. serde rename_all affects only Serialize.
# Test: test_BC_2_11_036_terminal_display_unchanged
# Also shows: CLI terminal mode output — no schema_version field visible.

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "cargo test --test reporter_json_tests test_BC_2_11_036_terminal_display_unchanged 2>&1"
Enter
Sleep 5s

Type "./target/release/wirerust analyze tests/fixtures/dns-remoteshell.pcap 2>/dev/null | head -16"
Enter
Sleep 3s
