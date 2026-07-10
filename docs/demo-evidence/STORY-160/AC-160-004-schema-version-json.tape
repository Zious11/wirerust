Output docs/demo-evidence/STORY-160/AC-160-004-schema-version-json.gif
Output docs/demo-evidence/STORY-160/AC-160-004-schema-version-json.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20

# AC-160-004: Every JSON report envelope includes "schema_version": "2" (BC-2.11.037).
# Part 1: Live CLI run piped through jq showing schema_version field present and = "2".
# Part 2: BC-2.11.037 unit tests all pass.

Hide
Type "cd <REPO-ROOT>/.worktrees/STORY-160"
Enter
Sleep 1s
Show

Type "./target/release/wirerust analyze tests/fixtures/dns-remoteshell.pcap --json 2>/dev/null | jq '{schema_version:.schema_version,findings_count:(.findings|length)}'"
Enter
Sleep 2s

Type "cargo test --test reporter_json_tests test_BC_2_11_037_schema_version 2>&1"
Enter
Sleep 5s
