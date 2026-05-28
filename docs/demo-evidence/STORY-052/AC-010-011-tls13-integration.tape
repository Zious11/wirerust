Output docs/demo-evidence/STORY-052/AC-010-011-tls13-integration.gif
Output docs/demo-evidence/STORY-052/AC-010-011-tls13-integration.webm

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

# AC-010/011: TLS 1.3 pcap — legacy_version=0x0303, JA3 uses 771, 0x0304 absent from version_counts
Type "cargo test --test tls_integration_tests test_tls13_pcap_version_and_ja3 -- --nocapture 2>&1"
Enter
Sleep 10s
