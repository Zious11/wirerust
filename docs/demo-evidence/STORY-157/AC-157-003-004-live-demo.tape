Output docs/demo-evidence/STORY-157/AC-157-003-004-live-demo.gif
Output docs/demo-evidence/STORY-157/AC-157-003-004-live-demo.webm

Require python3

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 500
Set Padding 20

Sleep 1s
Type "# AC-157-003: inputs: [] (inline compact) -> d41d8cd"
Enter
Sleep 0.5s
Type "python3 bin/compute-input-hash /tmp/story157-demo/story-empty-inline.md"
Enter
Sleep 2s
Type "# AC-157-004: empty multiline inputs block -> d41d8cd"
Enter
Sleep 0.5s
Type "python3 bin/compute-input-hash /tmp/story157-demo/story-empty-block.md"
Enter
Sleep 2s
