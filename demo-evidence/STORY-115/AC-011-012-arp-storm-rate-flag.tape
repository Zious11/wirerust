# VHS Demo — STORY-115 AC-011 + AC-012
# --arp-storm-rate flag: explicit value, default, and without --arp
# Binary: wirerust-s115 (wirerust 0.6.0 from STORY-115 worktree, in ~/bin)

Output AC-011-012-arp-storm-rate-flag.gif
Output AC-011-012-arp-storm-rate-flag.webm

Require wirerust-s115

Set Shell "zsh"
Set FontFamily "Menlo"
Set FontSize 18
Set Width 1200
Set Height 600
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

# AC-011: --arp-storm-rate 10 explicit — storm fires (10 frames/s >= threshold 10)
# Observed: storm_findings: 1 in ARP summary
Type "wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp --arp-storm-rate 10 --no-color"
Enter
Sleep 1s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# AC-011: default rate (50) — no storm fires (10 frames/s < 50)
# Observed: storm_findings: 0 in ARP summary
Type "wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp --no-color"
Enter
Sleep 1s

Hide
Type "clear"
Enter
Sleep 300ms
Show

# AC-012: --arp-storm-rate without --arp — no parse error, no ARP section
Type "wirerust-s115 analyze /tmp/arp_storm_10fps.pcap --arp-storm-rate 25 --no-color"
Enter
Sleep 1s

Sleep 2s
