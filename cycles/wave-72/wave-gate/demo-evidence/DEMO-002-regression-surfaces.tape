# Wave-72 Integration Demo: DEMO-002 — Regression Surfaces (Terminal + CSV)
# Evidences: STORY-160 AC-160-005 (terminal Display unchanged) + AC-160-006 (CSV unchanged, no schema_version)
#
# RECORDING NOTE: Transcript is primary evidence (see wave-71 VHS limitation note).
# This tape is retained as a runnable reference with <repo> placeholder.

Output DEMO-002-regression-surfaces.gif
Output DEMO-002-regression-surfaces.webm

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

Type "# Wave-72 Demo 2 — Terminal output: UPPERCASE Display tokens unchanged (AC-160-005)"
Enter
Sleep 400ms

Type "./target/release/wirerust analyze --all --no-color tests/fixtures/modbus-write.pcap 2>&1 | grep -E '\[Anomaly\]|\[Execution\]'"
Enter
Wait+Screen /INCONCLUSIVE/
Sleep 800ms

Type "# Terminal: confirm no schema_version leakage (AC-160-006)"
Enter
Sleep 300ms

Type "./target/release/wirerust analyze --all --no-color tests/fixtures/modbus-write.pcap 2>&1 | grep -c schema_version; echo 'expect: 0'"
Enter
Wait+Screen /expect/
Sleep 800ms

Type "# CSV output: PascalCase categories unchanged, no schema_version (AC-160-006)"
Enter
Sleep 300ms

Type "./target/release/wirerust analyze --all --output-format csv tests/fixtures/modbus-write.pcap 2>&1 | head -4"
Enter
Wait+Screen /Execution/
Sleep 2s
