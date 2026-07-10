Output docs/demo-evidence/STORY-157/AC-157-010-inline-comment-success.gif
Output docs/demo-evidence/STORY-157/AC-157-010-inline-comment-success.webm

Require python3

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 550
Set Padding 20

Sleep 1s
Type "# AC-157-010: comment suffix stripped; base path resolved"
Enter
Sleep 0.5s
Type "# inputs entry has trailing: '  # RETIRED 2026-06-19: superseded'"
Enter
Sleep 0.5s
Type "WIRERUST_REPO_ROOT=/tmp/story157-demo python3 bin/compute-input-hash /tmp/story157-demo/story-inline-comment.md"
Enter
Sleep 2s
Type "# same hash as clean path without comment:"
Enter
Sleep 0.5s
Type "WIRERUST_REPO_ROOT=/tmp/story157-demo python3 bin/compute-input-hash /tmp/story157-demo/story-nocomment.md"
Enter
Sleep 2s
