# VHS Demo — STORY-115 AC-003 + AC-013
# D3 storm finding: MEDIUM/Anomaly confidence, evidence fields, storm_findings > 0
# Binary: wirerust-s115 (wirerust 0.6.0 from STORY-115 worktree, in ~/bin)

Output AC-003-013-d3-storm-finding.gif
Output AC-003-013-d3-storm-finding.webm

Require wirerust-s115

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 16
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

Hide
Type "export PATH=$HOME/bin:$PATH"
Enter
Sleep 300ms
Type "clear"
Enter
Sleep 300ms
Show

# AC-003 + AC-013: D3 storm finding in JSON
# Shows: confidence=Medium, category=Anomaly, evidence (source_mac, frame_count,
# window_secs, rate_pps), mitre_techniques absent (empty per DF-VALIDATION-001),
# storm_findings=1 in analyzer detail
Type "wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp --arp-storm-rate 10 --json"
Enter
Sleep 2s

Sleep 2s
