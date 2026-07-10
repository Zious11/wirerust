Output AC-004-summarize-pass.gif
Output AC-004-summarize-pass.webm

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

Type "# AC-004: summarize() never emits dropped_findings key (BC-2.16.016 PC-2/3)"
Enter
Sleep 400ms

Type "cargo test test_BC_2_16_016_summarize_has_no_dropped_findings_key 2>&1 | grep -E 'running|ok|test result'"
Enter
Sleep 15s

Sleep 2s
