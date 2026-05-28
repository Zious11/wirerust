Output docs/demo-evidence/STORY-052/AC-011-legacy-version-only.gif
Output docs/demo-evidence/STORY-052/AC-011-legacy-version-only.webm

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

# AC-011: TlsAnalyzer uses only ch.version.0 (legacy_version) — supported_versions ext not inspected
Type "cargo test --test tls_analyzer_tests test_BC_2_07_032_inv1_supported_versions_not_inspected -- --nocapture 2>&1"
Enter
Sleep 10s
