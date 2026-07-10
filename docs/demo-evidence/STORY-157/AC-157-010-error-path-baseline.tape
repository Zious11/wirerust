Output docs/demo-evidence/STORY-157/AC-157-010-error-path-baseline.gif
Output docs/demo-evidence/STORY-157/AC-157-010-error-path-baseline.webm

Require python3

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 550
Set Padding 20

Sleep 1s
Type "# Before fix: develop-baseline does NOT strip inline comment"
Enter
Sleep 0.5s
Type "# Expected: 'Input file missing' error (path includes comment text)"
Enter
Sleep 0.5s
Type "WIRERUST_REPO_ROOT=/tmp/story157-demo python3 /tmp/input-hash-baseline.py /tmp/story157-demo/story-inline-comment.md"
Enter
Sleep 3s
