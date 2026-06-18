# STORY-118 AC-001 / AC-009 / AC-026
# Default-ON collapse: 25 identical empty-UA findings collapse to one group
# Shows: [Anomaly] INCONCLUSIVE (LOW) - Empty User-Agent header (x25)
#        with 3 evidence samples

Output AC-001-flood-collapse-default.gif
Output AC-001-flood-collapse-default.webm

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

# Show the default (collapse ON) behavior
Type "# wirerust v0.8.0: 25 identical findings collapse to one (x25) group"
Enter
Sleep 500ms
Type "./target/release/wirerust analyze --http --no-color /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-118/empty_ua_flood.pcap"
Enter
Sleep 5s
