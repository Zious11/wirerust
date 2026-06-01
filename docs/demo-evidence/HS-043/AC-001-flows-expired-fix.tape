Output docs/demo-evidence/HS-043/AC-001-flows-expired-fix.gif
Output docs/demo-evidence/HS-043/AC-001-flows-expired-fix.webm

Require wirerust

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set Framerate 24

# AC-001: Bug fix — expire_flows wired into production path.
# flows_expired=1 demonstrates the idle flow is reclaimed when
# --flow-timeout 5 is set and two TCP flows are 6s apart in the capture.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/hs043-flow-expiry"
Enter
Type "clear"
Enter
Show

Type "# HS-043 fix: expire_flows now called per-packet; --flow-timeout knob wired in"
Enter
Sleep 500ms

Type "wirerust analyze tests/fixtures/flow-expiry.pcap --all --flow-timeout 5 --output-format json"
Enter
Wait+Screen@10s /flows_expired/
Sleep 2s
