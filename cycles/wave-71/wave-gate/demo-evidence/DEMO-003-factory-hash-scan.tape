# Wave-71 Integration Demo: DEMO-003 — Factory Tooling Hash Scan
# Evidences: STORY-157 (hash tooling) — MATCH=110 STALE=0 confirms no spec drift
#
# RECORDING NOTE: VHS preferred but failed due to output scrolling past viewport.
# Use DEMO-003-factory-hash-scan.txt transcript as primary evidence artifact.

Output DEMO-003-factory-hash-scan.gif
Output DEMO-003-factory-hash-scan.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 450
Set FontSize 14
Set Padding 20
Set WaitTimeout 30s

# Setup: navigate to repo root and set WIRERUST_REPO_ROOT
Hide
Type "cd <repo>"
Enter
Sleep 300ms
Type "export WIRERUST_REPO_ROOT=<repo>"
Enter
Sleep 200ms
Show

Type "# Wave-71 Demo 3 — STORY-157 input-hash tooling: scan all stories"
Enter
Sleep 400ms

Type "bin/compute-input-hash --scan | tail -12"
Enter
Wait+Screen /MATCH=110/
Sleep 2s
