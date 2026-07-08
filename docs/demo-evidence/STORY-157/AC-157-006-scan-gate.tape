Output docs/demo-evidence/STORY-157/AC-157-006-scan-gate.gif
Output docs/demo-evidence/STORY-157/AC-157-006-scan-gate.webm

Require python3

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 650
Set Padding 20

Sleep 1s
Type "# AC-157-006: --scan completes MATCH=110 STALE=0"
Enter
Sleep 0.5s
Type "python3 bin/compute-input-hash --scan 2>&1 | tail -25"
Enter
Sleep 8s
