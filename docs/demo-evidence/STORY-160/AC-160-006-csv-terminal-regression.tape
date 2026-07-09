Output docs/demo-evidence/STORY-160/AC-160-006-csv-terminal-regression.gif
Output docs/demo-evidence/STORY-160/AC-160-006-csv-terminal-regression.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-006: CSV and terminal reporters are unaffected; no schema_version column in
# CSV; ThreatCategory CSV renders as PascalCase Debug repr (BC-2.11.036, BC-2.11.037).
# Tests: test_BC_2_11_037_schema_version_absent_from_csv
#        test_BC_2_11_037_schema_version_absent_from_terminal
#        test_BC_2_11_036_csv_category_unchanged
# Also shows: --csv header row has no schema_version column.

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "cargo test --test reporter_json_tests -- test_BC_2_11_037_schema_version_absent test_BC_2_11_036_csv_category_unchanged 2>&1"
Enter
Sleep 5s

Type "./target/release/wirerust analyze tests/fixtures/dns-remoteshell.pcap --csv 2>/dev/null"
Enter
Sleep 2s
