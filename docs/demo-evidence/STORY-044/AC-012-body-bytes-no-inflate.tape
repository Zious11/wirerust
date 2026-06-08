Output docs/demo-evidence/STORY-044/AC-012-body-bytes-no-inflate.gif
Output docs/demo-evidence/STORY-044/AC-012-body-bytes-no-inflate.webm

Require cargo

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 500
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/STORY-044 && clear"
Enter
Sleep 2s
Show

Type "cargo test --test http_analyzer_tests test_body_bytes_do_not_inflate_parse_errors -- --nocapture"
Enter
Sleep 10s
