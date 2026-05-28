Output docs/demo-evidence/STORY-052/AC-008-009-012-stop-after-handshake.gif
Output docs/demo-evidence/STORY-052/AC-008-009-012-stop-after-handshake.webm

Set Theme "Catppuccin Mocha"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Padding 16
Set Framerate 24
Set WindowBar Colorful
Set Shell "zsh"

Require cargo

Hide
Type "source ~/.zshrc 2>/dev/null || true"
Enter
Sleep 1s
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-052"
Enter
Sleep 1s
Show

# AC-008/009/012: done() short-circuit — retransmitted CH and 1MB burst silently discarded
Type "cargo test --test tls_analyzer_tests test_stop_after_handshake -- --nocapture 2>&1"
Enter
Sleep 10s
