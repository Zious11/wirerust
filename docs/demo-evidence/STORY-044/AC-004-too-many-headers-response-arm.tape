Output docs/demo-evidence/STORY-044/AC-004-too-many-headers-response-arm.gif
Output docs/demo-evidence/STORY-044/AC-004-too-many-headers-response-arm.webm

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

Type "cargo test --test http_analyzer_tests test_too_many_headers_in_response_generates_finding -- --nocapture"
Enter
Sleep 10s
