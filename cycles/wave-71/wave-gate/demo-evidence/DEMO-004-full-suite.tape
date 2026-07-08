# Wave-71 Integration Demo: DEMO-004 — Full Test Suite (2,378 passed)
# Evidences: complete regression gate — all 2,378 tests pass on merged develop
#
# RECORDING NOTE: VHS preferred but failed due to long test runtime (15+ seconds)
# and awk $N syntax not supported in VHS Type strings.
# Use DEMO-004-full-suite.txt transcript as primary evidence artifact.

Output DEMO-004-full-suite.gif
Output DEMO-004-full-suite.webm

Set FontFamily "Menlo"
Set Theme "Dracula"
Set Width 1200
Set Height 500
Set FontSize 14
Set Padding 20
Set Timeout 120000

# Setup: navigate to repo root
Hide
Type "cd <repo>"
Enter
Sleep 500ms
Show

Type "# Wave-71 Demo 4 — full regression gate on merged develop"
Enter
Sleep 400ms

Type "cargo test --all-targets 2>&1 | grep -c 'test result: ok'"
Enter
Wait+Screen /91/
Sleep 2s
