Output docs/demo-evidence/HS-043/AC-002-default-timeout-no-expiry.gif
Output docs/demo-evidence/HS-043/AC-002-default-timeout-no-expiry.webm

Require wirerust

Set FontFamily "Menlo"
Set FontSize 14
Set Width 1200
Set Height 600
Set Padding 20
Set Theme "Dracula"
Set Framerate 24

# AC-002: Contrast — default 300s timeout means no flow is expired.
# flows_expired=0 proves the knob controls expiry, not a side-effect.

Hide
Type "cd /Users/zious/Documents/GITHUB/wirerust/.worktrees/hs043-flow-expiry"
Enter
Type "clear"
Enter
Show

Type "# Default --flow-timeout 300: 6s gap does NOT trigger expiry"
Enter
Sleep 500ms

Type "wirerust analyze tests/fixtures/flow-expiry.pcap --all --output-format json"
Enter
Wait+Screen@10s /flows_expired/
Sleep 2s
