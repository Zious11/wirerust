Output AC-001-arp-help-unbounded.gif
Output AC-001-arp-help-unbounded.webm

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

Type "# AC-001: --arp long_help documents unbounded findings (BC-2.16.016 PC-4)"
Enter
Sleep 400ms

Type "./target/release/wirerust analyze --help 2>&1 | grep -B 3 -A 10 'UNBOUNDED'"
Enter
Sleep 3s

Sleep 2s
