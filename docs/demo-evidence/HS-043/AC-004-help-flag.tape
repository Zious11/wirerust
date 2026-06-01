Output docs/demo-evidence/HS-043/AC-004-help-flag.gif
Output docs/demo-evidence/HS-043/AC-004-help-flag.webm

Require wirerust

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set Framerate 24

# AC-004: --help shows the new --flow-timeout <secs> global flag (default 300).

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/hs043-flow-expiry"
Enter
Type "clear"
Enter
Show

Type "# --help shows --flow-timeout with default 300 and minimum 1"
Enter
Sleep 500ms

Type "wirerust analyze --help"
Enter
Wait+Screen@10s /flow-timeout/
Sleep 2s
