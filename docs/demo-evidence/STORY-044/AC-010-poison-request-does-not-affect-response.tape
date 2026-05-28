Output docs/demo-evidence/STORY-044/AC-010-poison-request-does-not-affect-response.gif
Output docs/demo-evidence/STORY-044/AC-010-poison-request-does-not-affect-response.webm

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

Type "cargo test --test http_analyzer_tests test_poison_request_does_not_affect_response -- --nocapture"
Enter
Sleep 10s
