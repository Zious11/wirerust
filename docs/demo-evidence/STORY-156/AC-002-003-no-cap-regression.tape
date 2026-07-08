Output AC-002-003-no-cap-regression.gif
Output AC-002-003-no-cap-regression.webm

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

Type "# AC-002+003: 10,001 ARP findings, no MAX_FINDINGS cap (BC-2.16.016)"
Enter
Sleep 400ms

Type "cargo test bc_2_16_016_arp_findings_vec_has_no_cap 2>&1 | grep -E 'running|ok|test result'"
Enter
Sleep 15s

Sleep 2s
