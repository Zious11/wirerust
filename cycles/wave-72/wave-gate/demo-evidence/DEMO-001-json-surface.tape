# Wave-72 Integration Demo: DEMO-001 — Combined JSON Surface
# Evidences: STORY-160 (BC-2.11.036 lowercase/snake_case + BC-2.11.037 schema_version + BC-2.11.001 v1.9)
#
# RECORDING NOTE: VHS preferred but transcript is primary evidence (see wave-71 VHS limitation note —
# wirerust unicode box-drawing output fills VHS 0.11.0 viewport before Wait+Screen matches).
# This tape is retained as a runnable reference with <repo> placeholder.

Output DEMO-001-json-surface.gif
Output DEMO-001-json-surface.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set WaitTimeout 60s

# Setup: navigate to repo root
Hide
Type "cd <repo>"
Enter
Sleep 500ms
Show

Type "# Wave-72 Demo 1 — JSON envelope (STORY-160: schema_version + lowercase enums)"
Enter
Sleep 400ms

Type "./target/release/wirerust analyze --all --output-format json tests/fixtures/modbus-write.pcap 2>&1 | python3 -m json.tool | grep -E '\"schema_version\"|\"findings\"|\"analyzers\"|\"mitre_'"
Enter
Wait+Screen /schema_version/
Sleep 800ms

Type "# Verify top-level keys and schema_version = \"2\" (BC-2.11.001 v1.9 + BC-2.11.037)"
Enter
Sleep 300ms

Type "./target/release/wirerust analyze --all --output-format json tests/fixtures/modbus-write.pcap 2>&1 | python3 -c \"import json,sys; d=json.load(sys.stdin); print(sorted(d.keys())); print('schema_version:', d['schema_version'])\""
Enter
Wait+Screen /schema_version/
Sleep 800ms

Type "# Verify findings use lowercase/snake_case (BC-2.11.036)"
Enter
Sleep 300ms

Type "./target/release/wirerust analyze --all --output-format json tests/fixtures/modbus-write.pcap 2>&1 | python3 -c \"import json,sys; d=json.load(sys.stdin); [print(f['verdict'], f['confidence'], f['category']) for f in d['findings']]\""
Enter
Wait+Screen /inconclusive/
Sleep 2s
