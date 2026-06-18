# STORY-118 AC-018 / AC-028
# --no-collapse opt-out: 25 identical findings show as 25 individual lines
# Shows: pre-v0.8.0 behavior restored, no (xN) suffix anywhere

Output AC-018-no-collapse-opt-out.gif
Output AC-018-no-collapse-opt-out.webm

Set Shell "bash"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 50ms

Hide
Sleep 500ms
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/feat-259-finding-collapse && clear"
Enter
Sleep 1500ms
Show

# Show the --no-collapse opt-out behavior (one line per finding, no (xN) suffix)
Type "# --no-collapse: restores one-line-per-finding (pre-v0.8.0 behavior)"
Enter
Sleep 500ms
Type "./target/release/wirerust analyze --http --no-color --no-collapse /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-118/empty_ua_flood.pcap"
Enter
Sleep 5s
