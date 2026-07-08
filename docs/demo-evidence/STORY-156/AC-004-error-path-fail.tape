Output AC-004-error-path-fail.gif
Output AC-004-error-path-fail.webm

Set Shell "bash"
Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Dracula"
Set Padding 20
Set PlaybackSpeed 1.0

Hide
Type "export PATH=<HOME>/.cargo/bin:$PATH && cd <REPO-ROOT>/.worktrees/STORY-156"
Enter
Sleep 1s
Show

Type "# AC-004 error path: injecting dropped_findings key triggers test failure"
Enter
Sleep 400ms

Type "# (src/analyzer/arp.rs temporarily modified — see evidence-report.md)"
Enter
Sleep 400ms

Type "cargo test test_BC_2_16_016_summarize_has_no_dropped_findings_key 2>&1 | grep -E 'FAILED|BC-2.16.016|test result'"
Enter
Sleep 15s

Sleep 2s
