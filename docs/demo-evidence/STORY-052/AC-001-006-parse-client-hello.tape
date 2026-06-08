Output docs/demo-evidence/STORY-052/AC-001-006-parse-client-hello.gif
Output docs/demo-evidence/STORY-052/AC-001-006-parse-client-hello.webm

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

# Setup (hidden) — navigate to worktree, source shell profile for cargo PATH
Hide
Type "source ~/.zshrc 2>/dev/null || true"
Enter
Sleep 1s
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-052"
Enter
Sleep 1s
Show

# AC-001..006: ClientHello parsing — handshake counting, version/JA3 tracking, SNI, buffer drain
Type "cargo test --test tls_analyzer_tests test_parse_client_hello -- --nocapture 2>&1"
Enter
Sleep 10s
