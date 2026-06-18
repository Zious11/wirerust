# STORY-118 AC-020 / AC-027
# JSON output unaffected: collapse is display-only
# Shows: 25 JSON finding objects emitted regardless of collapse setting

Output AC-020-json-unaffected.gif
Output AC-020-json-unaffected.webm

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

# JSON always emits all N findings; collapse is display-layer only
Type "# AC-020: JSON output unaffected -- collapse is display-layer only"
Enter
Sleep 500ms
Type "# Step 1: default terminal output (collapse ON) shows (x25)"
Enter
Sleep 300ms
Type "./target/release/wirerust analyze --http --no-color /Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-118/empty_ua_flood.pcap 2>&1 | grep -A3 FINDINGS"
Enter
Sleep 4s
Type "# Step 2: JSON emits all 25 objects (display-layer only)"
Enter
Sleep 300ms
Type "/Users/zious/Documents/GITHUB/wirerust/.factory/demo-evidence/STORY-118/count_json_findings.sh"
Enter
Sleep 3s
